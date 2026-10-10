//! Send-time primary versions, APK 37.71 0Z5.A0D -> DPo.invoke selector 1.
//! This captures state for the original reservation; it never permits replay.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    Error, Result,
    model::{
        ids::LocalMessageId,
        jid::{Jid, JidKind, Server},
    },
    outbox::Outbox,
    profile::Limits,
    store::{ProtocolStore, Table, WriteTx},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    users: Vec<CapturedUser>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedUser {
    user: Jid,
    devices: usize,
    #[serde(deserialize_with = "nullable_version")]
    version: Option<i64>,
    // Legacy snapshots retain counts only; never reconstruct their addresses.
    #[serde(default)]
    device_ids: Option<Vec<u16>>,
}

// A missing JSON member is malformed, distinct from an explicit null counter.
fn nullable_version<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<i64>, D::Error> {
    Option::<i64>::deserialize(d)
}

/// One actual frozen target device, with its address redacted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceTarget {
    /// SHA-256 of the normalized exact device address.
    pub device_ticket: String,
    /// Original device component; zero identifies the primary device.
    pub device: u16,
}

/// Redacted relationship of one actual send target's user to current state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserVersion {
    /// SHA-256 of normalized user JID; never a phone number or safety number.
    pub user_ticket: String,
    /// Number of unique actual target devices for this user at reservation.
    pub devices: usize,
    /// Exact frozen targets, or null for legacy count-only instrumentation.
    pub device_targets: Option<Vec<DeviceTarget>>,
    /// Known missing counter is null; historical missing snapshots have no rows.
    pub at_send: Option<i64>,
    /// Current tracked counter in this exact namespace, or null when absent.
    pub current: Option<i64>,
    /// Nullable equality only. This is not a resend permission or feature gate.
    pub matches: bool,
}

/// Offline original-send snapshot. Missing historical instrumentation stays unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendVersions {
    /// Original stable local intent selector.
    pub local_id: LocalMessageId,
    /// False for an old/unprepared intent without instrumentation; never backfilled.
    pub captured: bool,
    /// Redacted actual user namespaces; empty when no snapshot is available.
    pub users: Vec<UserVersion>,
    /// Original-device row membership, or null for historical missing instrumentation.
    /// Timestamps are server milliseconds, never local observation time or resend permission.
    pub device_receipts: Option<Vec<DeviceReceipt>>,
}

/// One original target's mutable row membership, with no address exposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceReceipt {
    /// SHA-256 of the exact normalized original device address.
    pub device_ticket: String,
    /// Original device component.
    pub device: u16,
    /// Nullable primary version captured by the original reservation.
    pub at_send: Option<i64>,
    /// Whether the mutable device row currently exists; false retains provenance only.
    pub present: bool,
    /// Nullable current-row version; absent rows have null, independently of at_send.
    pub version: Option<i64>,
    /// Earliest positive server timestamp in milliseconds, or pending/unknown null.
    pub timestamp: Option<i64>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptRows {
    rows: Vec<ReceiptRow>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptRow {
    device: Jid,
    #[serde(deserialize_with = "nullable_version")]
    version: Option<i64>,
    #[serde(deserialize_with = "nullable_version")]
    timestamp: Option<i64>,
    // Old rows predate cleanup and therefore really were present, not unknown.
    #[serde(default = "row_present")]
    present: bool,
    #[serde(default)]
    reinserted: bool,
}
fn row_present() -> bool {
    true
}
fn receipt_key(id: LocalMessageId) -> String {
    format!("outbox-device-receipts:{id}")
}
fn key(id: LocalMessageId) -> String {
    format!("outbox-primary-versions:{id}")
}
fn invalid(reason: &'static str) -> Error {
    Error::invalid("send_versions", reason)
}
fn primary(user: &Jid) -> bool {
    !user.user.is_empty()
        && user.agent == 0
        && user.device == 0
        && user.kind == JidKind::Plain
        && matches!(user.server, Server::User | Server::Lid)
}

/// Called after authenticated replacement hooks, inside the original send's
/// OCC/reservation transaction. Group input includes all plan devices, not only
/// devices requiring a new sender-key distribution. No standalone writer exists.
pub(super) fn capture_in(tx: &mut WriteTx<'_>, id: LocalMessageId, targets: &[Jid]) -> Result<()> {
    if targets.is_empty() {
        return Err(invalid("send targets are empty"));
    }
    let mut users = BTreeMap::<Jid, BTreeSet<Jid>>::new();
    for target in targets {
        if target.user.is_empty()
            || target.agent != 0
            || !matches!(target.server, Server::User | Server::Lid)
        {
            return Err(invalid("unsupported send target namespace"));
        }
        let mut device = target.clone();
        device.kind = JidKind::Plain;
        users.entry(device.to_non_ad()).or_default().insert(device);
    }
    let mut snapshot = Snapshot { users: Vec::new() };
    for (user, devices) in users {
        // APK 37.71 DPo.invoke 0058..0086 binds the device user's version,
        // preserving SQL null. Read after any replacement increment in this tx.
        let version = super::devices::primary_device_version_write_in(tx, &user)?;
        snapshot.users.push(CapturedUser {
            user,
            devices: devices.len(),
            device_ids: Some(devices.iter().map(|d| d.device).collect()),
            version,
        });
    }
    let key = key(id);
    if tx.get(Table::Checkpoints, key.as_bytes(), 1)?.is_some() {
        return Err(invalid("original send version snapshot already exists"));
    }
    // APK 37.71 0Z5.A0D / 0Z6.A0L seeds pending device rows with nullable
    // original versions. Keep these mutable timestamps separate from provenance.
    let rows = expected_receipts(&snapshot).ok_or_else(|| invalid("missing original devices"))?;
    if tx
        .get(Table::Checkpoints, receipt_key(id).as_bytes(), 1)?
        .is_some()
    {
        return Err(invalid("original device receipt rows already exist"));
    }
    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &snapshot)?;
    tx.put_json(
        Table::Checkpoints,
        receipt_key(id).as_bytes(),
        1,
        &ReceiptRows {
            rows: rows
                .into_iter()
                .map(|(device, version)| ReceiptRow {
                    device,
                    version,
                    timestamp: None,
                    present: true,
                    reinserted: false,
                })
                .collect(),
        },
    )
}

fn decode_snapshot(bytes: Option<Vec<u8>>) -> Result<Option<Snapshot>> {
    let snapshot: Option<Snapshot> = bytes
        .map(|bytes| {
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid saved version snapshot"))
        })
        .transpose()?;
    if let Some(snapshot) = &snapshot {
        if snapshot.users.is_empty() {
            return Err(invalid("saved version snapshot is empty"));
        }
        let mut seen = BTreeSet::new();
        let known = snapshot.users[0].device_ids.is_some();
        for row in &snapshot.users {
            if !primary(&row.user) || row.devices == 0 || !seen.insert(&row.user) {
                return Err(invalid("invalid saved version snapshot user"));
            }
            if row.device_ids.is_some() != known {
                return Err(invalid("inconsistent saved device target provenance"));
            }
            if let Some(ids) = &row.device_ids
                && (ids.len() != row.devices || ids.windows(2).any(|w| w[0] >= w[1]))
            {
                return Err(invalid("invalid saved device targets"));
            }
        }
    }
    Ok(snapshot)
}
fn device(user: &Jid, id: u16) -> Jid {
    let mut target = user.clone();
    target.device = id;
    target
}

fn expected_receipts(snapshot: &Snapshot) -> Option<BTreeMap<Jid, Option<i64>>> {
    let mut rows = BTreeMap::new();
    for user in &snapshot.users {
        for id in user.device_ids.as_ref()? {
            rows.insert(device(&user.user, *id), user.version);
        }
    }
    Some(rows)
}
fn decode_receipts(
    bytes: Option<Vec<u8>>,
    snapshot: Option<&Snapshot>,
) -> Result<Option<ReceiptRows>> {
    let Some(bytes) = bytes else {
        return Ok(None);
    };
    let rows: ReceiptRows =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid saved device receipt rows"))?;
    let expected = snapshot
        .and_then(expected_receipts)
        .ok_or_else(|| invalid("device receipt rows lack original provenance"))?;
    let mut seen = BTreeSet::new();
    if rows.rows.len() != expected.len()
        || rows.rows.iter().any(|row| {
            !seen.insert(&row.device)
                || !expected.contains_key(&row.device)
                || (!row.reinserted && expected.get(&row.device) != Some(&row.version))
                || row.timestamp.is_some_and(|t| t <= 0)
                || (!row.present
                    && (row.device.device == 0 || row.timestamp.is_some() || row.reinserted))
                || (row.reinserted
                    && (!row.present || row.device.device == 0 || row.timestamp.is_none()))
        })
    {
        return Err(invalid(
            "device receipt rows contradict original provenance",
        ));
    }
    Ok(Some(rows))
}

/// Caller has already authenticated and attributed the receipt. Exact original
/// namespace only: native counterpart authority/primary cleanup are unported.
/// APK Cmk.A03 direct t -> 0Gv.A01(default 0) -> wrapping mul-long 1000;
/// 7hf.A01 keeps earliest positive time, even for an equal/lower status.
pub(super) fn observe_in(
    tx: &mut WriteTx<'_>,
    id: LocalMessageId,
    actor: &Jid,
    node: &crate::binary::Node,
) -> Result<bool> {
    let snapshot = decode_snapshot(tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?)?;
    let Some(mut rows) = decode_receipts(
        tx.get(Table::Checkpoints, receipt_key(id).as_bytes(), 1)?,
        snapshot.as_ref(),
    )?
    else {
        return Ok(false);
    };
    let timestamp = super::receipts::server_timestamp(node)
        .map(|time| time.as_millis())
        .unwrap_or(0);
    if timestamp <= 0 {
        return Ok(false);
    }
    let mut actor = actor.clone();
    actor.kind = JidKind::Plain;
    let Some(row) = rows.rows.iter_mut().find(|row| row.device == actor) else {
        return Ok(false);
    };
    if !row.present {
        // APK 37.71 0Z5.A0A 034e..0366: an UPDATE with zero matches
        // inserts the current nullable user version, not the original version.
        row.version = super::devices::primary_device_version_write_in(tx, &actor.to_non_ad())?;
        row.present = true;
        row.reinserted = true;
    }
    if row.timestamp.is_some_and(|prior| prior <= timestamp) {
        return Ok(false);
    }
    row.timestamp = Some(timestamp);
    tx.put_json(Table::Checkpoints, receipt_key(id).as_bytes(), 1, &rows)?;
    Ok(true)
}

/// Authenticated companion replacement only. Native 0Z5.A0B excludes primary
/// devices and deletes null-timestamp rows for the changed device. Retain the
/// original target as provenance, separately from mutable row membership.
pub(super) fn after_companion_identity_change_in(tx: &mut WriteTx<'_>, peer: &Jid) -> Result<()> {
    if peer.user.is_empty()
        || peer.device == 0
        || peer.agent != 0
        || !matches!(peer.server, Server::User | Server::Lid)
    {
        return Ok(());
    }
    let mut peer = peer.clone();
    peer.kind = JidKind::Plain;
    let mut cursor = None;
    loop {
        let page = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if page.is_empty() {
            break;
        }
        cursor = page.last().map(|(key, _)| key.clone());
        for (record_key, bytes) in page {
            let Some(selector) = std::str::from_utf8(&record_key)
                .ok()
                .and_then(|name| name.strip_prefix("outbox-device-receipts:"))
            else {
                continue;
            };
            let id: LocalMessageId = selector
                .parse()
                .map_err(|_| invalid("invalid device receipt intent selector"))?;
            let entry = Outbox::new("comparison-driver", Limits::default())
                .get_mut(tx, id)?
                .ok_or_else(|| invalid("device receipt rows lack original outbox intent"))?;
            // Ordinary direct/group rows are established. Status/coexistence
            // predicates are separate 0Z4.A07 branches and remain unported.
            if !matches!(
                entry.intent.chat.jid().server,
                Server::User | Server::Lid | Server::Group
            ) {
                continue;
            }
            let snapshot = decode_snapshot(tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?)?;
            let mut rows = decode_receipts(Some(bytes), snapshot.as_ref())?
                .ok_or_else(|| invalid("missing saved device receipt rows"))?;
            if let Some(row) = rows.rows.iter_mut().find(|row| row.device == peer)
                && row.present
                && row.timestamp.is_none()
            {
                row.present = false;
                tx.put_json(Table::Checkpoints, &record_key, 1, &rows)?;
            }
        }
    }
    Ok(())
}

/// Original targets for receipt attribution, not current membership or resend
/// permission. Missing/legacy count-only instrumentation remains unknown.
pub(super) fn targets_in(tx: &WriteTx<'_>, id: LocalMessageId) -> Result<Option<Vec<Jid>>> {
    let Some(snapshot) = decode_snapshot(tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?)?
    else {
        return Ok(None);
    };
    if snapshot.users[0].device_ids.is_none() {
        return Ok(None);
    }
    Ok(Some(
        snapshot
            .users
            .into_iter()
            .flat_map(|row| {
                row.device_ids
                    .unwrap()
                    .into_iter()
                    .map(move |id| device(&row.user, id))
            })
            .collect(),
    ))
}

/// Inspect one existing outbox intent without queries, writes or resend decisions.
pub fn inspect(store: &ProtocolStore, id: LocalMessageId) -> Result<SendVersions> {
    store.read(|tx| {
        Outbox::new("comparison-driver", Limits::default())
            .get(tx, id)?
            .ok_or_else(|| invalid("unknown outbox intent"))?;
        let snapshot = decode_snapshot(tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?)?;
        let receipts = decode_receipts(
            tx.get(Table::Checkpoints, receipt_key(id).as_bytes(), 1)?,
            snapshot.as_ref(),
        )?;
        let original = snapshot
            .as_ref()
            .and_then(expected_receipts)
            .unwrap_or_default();
        let mut result = SendVersions {
            local_id: id,
            captured: snapshot.is_some(),
            users: Vec::new(),
            device_receipts: receipts.map(|rows| {
                rows.rows
                    .into_iter()
                    .map(|row| DeviceReceipt {
                        device_ticket: hex::encode(Sha256::digest(
                            row.device.to_string().as_bytes(),
                        )),
                        device: row.device.device,
                        at_send: original[&row.device],
                        present: row.present,
                        version: if row.present { row.version } else { None },
                        timestamp: row.timestamp,
                    })
                    .collect()
            }),
        };
        if let Some(snapshot) = snapshot {
            for row in snapshot.users {
                let current = super::devices::primary_device_version_in(tx, &row.user)?;
                let device_targets = row.device_ids.map(|ids| {
                    ids.into_iter()
                        .map(|id| DeviceTarget {
                            device_ticket: hex::encode(Sha256::digest(
                                device(&row.user, id).to_string().as_bytes(),
                            )),
                            device: id,
                        })
                        .collect()
                });
                result.users.push(UserVersion {
                    user_ticket: hex::encode(Sha256::digest(row.user.to_string().as_bytes())),
                    devices: row.devices,
                    device_targets,
                    at_send: row.version,
                    current,
                    matches: row.version == current,
                });
            }
        }
        Ok(result)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId, ClientRequestId},
            message::MessageContent,
            send::SendRequest,
        },
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
            create,
        })
        .unwrap()
    }
    fn enqueue(store: &ProtocolStore) -> LocalMessageId {
        let request = SendRequest {
            client_request_id: ClientRequestId::new("original-version-fixture").unwrap(),
            chat: ChatId::from_jid(&Jid::parse("15555550101@s.whatsapp.net").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "PRIVATE CONTENT".into(),
                quoted: None,
                style: Default::default(),
            },
        };
        store
            .write(|tx| {
                Ok(Outbox::new("comparison-driver", Limits::default())
                    .enqueue(tx, &request)?
                    .0
                    .local_id)
            })
            .unwrap()
    }
    #[test]
    fn companion_cleanup_and_late_reinsertion_preserve_original_send_and_reservations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("companion-cleanup.redb");
        let store = open(&path, true);
        let request = SendRequest {
            client_request_id: ClientRequestId::new("group-companion-cleanup-fixture").unwrap(),
            chat: ChatId::from_jid(&Jid::parse("123456@g.us").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "PRIVATE CONTENT".into(),
                quoted: None,
                style: Default::default(),
            },
        };
        let id = store
            .write(|tx| {
                Ok(Outbox::new("comparison-driver", Limits::default())
                    .enqueue(tx, &request)?
                    .0
                    .local_id)
            })
            .unwrap();
        let pn = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let changed = pn.with_device(2);
        let received = pn.with_device(3);
        let lid = Jid::parse("222222222222222:2@lid").unwrap();
        let node = crate::binary::Node::new("receipt").attr("t", "20");
        store
            .write(|tx| {
                capture_in(
                    tx,
                    id,
                    &[pn.clone(), changed.clone(), received.clone(), lid.clone()],
                )?;
                observe_in(tx, id, &received, &node)?;
                tx.put_json(
                    Table::Checkpoints,
                    b"receipt-outbox:held",
                    1,
                    &serde_json::json!({"state":"uncertain"}),
                )?;
                // Force a later scan page; cleanup must not stop at unrelated entries.
                for i in 0..300 {
                    tx.put_json(
                        Table::Checkpoints,
                        format!("aaa-fixture:{i:03}").as_bytes(),
                        1,
                        &i,
                    )?;
                }
                Ok(())
            })
            .unwrap();
        let original = inspect(&store, id).unwrap();
        let frozen = store
            .read(|tx| {
                Ok((
                    tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?,
                    tx.get(Table::Outbox, &id.0, 1)?,
                    tx.get(Table::Checkpoints, b"receipt-outbox:held", 1)?,
                ))
            })
            .unwrap();
        // First seen / identical key observations do not run cleanup.
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_replacement(
                    tx,
                    &changed,
                    None,
                    None,
                    &[2],
                )?;
                super::super::identity_conflicts::archive_bundle_replacement(
                    tx,
                    &changed,
                    Some(&[2]),
                    None,
                    &[2],
                )
            })
            .unwrap();
        assert_eq!(inspect(&store, id).unwrap(), original);
        let mut encoded = changed.clone();
        encoded.kind = JidKind::Interop;
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_replacement(
                    tx,
                    &encoded,
                    Some(&[1]),
                    None,
                    &[2],
                )
            })
            .unwrap();
        let cleaned = inspect(&store, id).unwrap();
        assert_eq!(cleaned.users, original.users);
        let rows = cleaned.device_receipts.as_ref().unwrap();
        assert_eq!(rows.iter().filter(|r| !r.present).count(), 1);
        assert_eq!(
            rows.iter().filter(|r| r.device == 2 && r.present).count(),
            1
        ); // LID remains separate.
        assert_eq!(
            rows.iter().find(|r| r.device == 3).unwrap().timestamp,
            Some(20000)
        );
        assert!(rows.iter().find(|r| r.device == 0).unwrap().present);
        // Rollback includes cleanup/history/journal; directory and message sources share it.
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap();
        let head = store.journal_head().unwrap();
        let result: Result<()> = store.write(|tx| {
            super::super::identity_conflicts::archive_bundle_replacement(
                tx,
                &lid,
                Some(&[1]),
                None,
                &[2],
            )?;
            Err(invalid("fixture rollback"))
        });
        assert!(result.is_err());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap(),
            before
        );
        assert_eq!(store.journal_head().unwrap(), head);
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_bundle_replacement(
                    tx,
                    &lid,
                    Some(&[1]),
                    None,
                    &[2],
                )
            })
            .unwrap();
        assert_eq!(
            inspect(&store, id)
                .unwrap()
                .device_receipts
                .unwrap()
                .iter()
                .filter(|r| !r.present)
                .count(),
            2
        );
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("primary-device-version:{pn}").as_bytes(),
                    1,
                    &4i64,
                )?;
                assert!(!observe_in(tx, id, &changed, &node.clone().attr("t", "0"))?);
                assert!(observe_in(tx, id, &changed, &node.clone().attr("t", "11"))?);
                tx.put_json(
                    Table::Checkpoints,
                    format!("primary-device-version:{pn}").as_bytes(),
                    1,
                    &5i64,
                )?;
                assert!(observe_in(tx, id, &changed, &node.clone().attr("t", "10"))?);
                super::super::identity_conflicts::archive_bundle_replacement(
                    tx,
                    &changed,
                    Some(&[2]),
                    None,
                    &[3],
                )
            })
            .unwrap();
        let saved = inspect(&store, id).unwrap();
        let row = saved
            .device_receipts
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.device == 2 && r.present)
            .unwrap();
        assert_eq!(row.at_send, None);
        assert_eq!(row.version, Some(4)); // Reinsertion version, not latest receipt's 5.
        assert_eq!(row.timestamp, Some(10000));
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?,
                    tx.get(Table::Outbox, &id.0, 1)?,
                    tx.get(Table::Checkpoints, b"receipt-outbox:held", 1)?
                )))
                .unwrap(),
            frozen
        );
        assert_eq!(
            store.write(|tx| targets_in(tx, id)).unwrap().unwrap().len(),
            4
        );
        drop(store);
        assert_eq!(inspect(&open(&path, false), id).unwrap(), saved);
    }

    #[test]
    fn companion_cleanup_guards_and_legacy_rows_are_not_missing_provenance() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("legacy-cleanup.redb"), true);
        let id = enqueue(&store);
        let pn = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let companion = pn.with_device(2);
        store
            .write(|tx| capture_in(tx, id, &[pn.clone(), companion.clone()]))
            .unwrap();
        let mut legacy: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, receipt_key(id).as_bytes(), 1))
            .unwrap()
            .unwrap();
        for row in legacy["rows"].as_array_mut().unwrap() {
            row.as_object_mut().unwrap().remove("present");
            row.as_object_mut().unwrap().remove("reinserted");
        }
        store
            .write(|tx| tx.put_json(Table::Checkpoints, receipt_key(id).as_bytes(), 1, &legacy))
            .unwrap();
        assert!(
            inspect(&store, id)
                .unwrap()
                .device_receipts
                .unwrap()
                .iter()
                .all(|r| r.present)
        );
        let mut agent = companion.clone();
        agent.agent = 1;
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        for peer in [
            pn.clone(),
            agent,
            Jid::parse("15555550102:2@s.whatsapp.net").unwrap(),
        ] {
            store
                .write(|tx| after_companion_identity_change_in(tx, &peer))
                .unwrap();
        }
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        for (index, field, value) in [
            (0, "present", serde_json::json!(false)),
            (1, "reinserted", serde_json::json!(true)),
            (1, "version", serde_json::json!(3)),
        ] {
            let mut bad = legacy.clone();
            bad["rows"][index][field] = value;
            store
                .write(|tx| tx.put_json(Table::Checkpoints, receipt_key(id).as_bytes(), 1, &bad))
                .unwrap();
            assert!(
                store
                    .write(|tx| after_companion_identity_change_in(tx, &companion))
                    .is_err()
            );
            assert!(inspect(&store, id).is_err());
        }
        store
            .write(|tx| {
                tx.put_json(Table::Checkpoints, receipt_key(id).as_bytes(), 1, &legacy)?;
                after_companion_identity_change_in(tx, &companion)
            })
            .unwrap();
        assert_eq!(
            inspect(&store, id)
                .unwrap()
                .device_receipts
                .unwrap()
                .iter()
                .filter(|r| !r.present)
                .count(),
            1
        );
    }
    #[test]
    fn send_versions_capture_known_null_and_namespace_state_once_and_survive_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("versions.redb");
        let store = open(&path, true);
        let id = enqueue(&store);
        let pn = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let lid = Jid::parse("222222222222222@lid").unwrap();
        let legacy = inspect(&store, id).unwrap();
        assert!(!legacy.captured);
        assert!(legacy.users.is_empty());
        assert!(legacy.device_receipts.is_none());
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("primary-device-version:{lid}").as_bytes(),
                    1,
                    &7i64,
                )?;
                capture_in(
                    tx,
                    id,
                    &[
                        pn.clone(),
                        pn.with_device(0),
                        pn.with_device(3),
                        lid.clone(),
                        lid.with_device(4),
                    ],
                )
            })
            .unwrap();
        let before = inspect(&store, id).unwrap();
        assert!(before.captured);
        assert_eq!(before.users.len(), 2);
        assert!(
            before
                .users
                .iter()
                .all(|u| u.device_targets.as_ref().unwrap().len() == u.devices)
        );
        let targets = store.write(|tx| targets_in(tx, id)).unwrap().unwrap();
        assert_eq!(targets.len(), 4);
        let receipts = before.device_receipts.as_ref().unwrap();
        assert_eq!(receipts.len(), 4);
        assert!(receipts.iter().all(|r| r.timestamp.is_none()));
        assert_eq!(receipts.iter().filter(|r| r.at_send == Some(7)).count(), 2);
        assert!(targets.iter().all(|j| j.kind == JidKind::Plain));
        assert!(
            before
                .users
                .iter()
                .any(|u| u.at_send.is_none() && u.current.is_none() && u.devices == 2 && u.matches)
        );
        assert!(
            before
                .users
                .iter()
                .any(|u| u.at_send == Some(7) && u.devices == 2 && u.matches)
        );
        let text = serde_json::to_string(&before).unwrap();
        for secret in [pn.user.as_str(), lid.user.as_str(), "PRIVATE CONTENT"] {
            assert!(!text.contains(secret));
        }
        let rows = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let head = store.journal_head().unwrap();
        assert!(
            store
                .write(|tx| capture_in(tx, id, std::slice::from_ref(&pn)))
                .is_err()
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            rows
        );
        assert_eq!(store.journal_head().unwrap(), head);
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("primary-device-version:{lid}").as_bytes(),
                    1,
                    &8i64,
                )
            })
            .unwrap();
        drop(store);
        let store = open(&path, false);
        let after = inspect(&store, id).unwrap();
        assert!(
            after
                .users
                .iter()
                .any(|u| u.at_send == Some(7) && u.current == Some(8) && !u.matches)
        );
        assert_eq!(after.device_receipts, before.device_receipts);
        assert_eq!(store.journal_head().unwrap(), head);
    }
    #[test]
    fn send_versions_legacy_counts_remain_unknown_and_corrupt_targets_refuse() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("legacy-targets.redb"), true);
        let id = enqueue(&store);
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let legacy = serde_json::json!({"users":[{"user":peer,"devices":2,"version":null}]});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &legacy))
            .unwrap();
        let head = store.journal_head().unwrap();
        let value = inspect(&store, id).unwrap();
        assert!(value.captured);
        assert_eq!(value.users[0].devices, 2);
        assert!(value.users[0].device_targets.is_none());
        assert!(store.write(|tx| targets_in(tx, id)).unwrap().is_none());
        assert_eq!(store.journal_head().unwrap(), head);
        for ids in [
            serde_json::json!([]),
            serde_json::json!([0]),
            serde_json::json!([3, 0]),
            serde_json::json!([0, 0]),
            serde_json::json!([0, "PRIVATE VALUE"]),
            serde_json::json!([0, 65536]),
        ] {
            let value = serde_json::json!({"users":[{"user":peer,"devices":2,"version":null,"device_ids":ids}]});
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &value))
                .unwrap();
            let error = inspect(&store, id).unwrap_err().to_string();
            assert!(!error.contains("PRIVATE"));
            assert!(!error.contains(&peer.user));
            assert!(store.write(|tx| targets_in(tx, id)).is_err());
        }
    }

    #[test]
    fn send_versions_rollback_and_invalid_targets_never_create_a_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("rollback.redb"), true);
        let id = enqueue(&store);
        let peer = Jid::parse("222222222222222@lid").unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let failed: Result<()> = store.write(|tx| {
            capture_in(tx, id, std::slice::from_ref(&peer))?;
            Err(invalid("fixture rollback"))
        });
        assert!(failed.is_err());
        for targets in [
            vec![],
            vec![Jid::parse("lid").unwrap()],
            vec![Jid::parse("123.1:0@lid").unwrap()],
            vec![Jid::parse("123@g.us").unwrap()],
        ] {
            assert!(store.write(|tx| capture_in(tx, id, &targets)).is_err());
        }
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        assert!(!inspect(&store, id).unwrap().captured);
        assert!(inspect(&store, LocalMessageId([1; 16])).is_err());
    }
    #[test]
    fn send_versions_malformed_rows_and_counters_refuse_without_echoing_private_values() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("malformed.redb"), true);
        let id = enqueue(&store);
        let peer = Jid::parse("222222222222222@lid").unwrap();
        let valid = serde_json::json!({"user":peer,"devices":1,"version":null});
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("version");
        for value in [
            serde_json::json!({"users":[missing]}),
            serde_json::json!({"users":[valid.clone(),valid.clone()]}),
            serde_json::json!({"users":[],"PRIVATE UNKNOWN NAME":"PRIVATE VALUE"}),
        ] {
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &value))
                .unwrap();
            let error = inspect(&store, id).unwrap_err().to_string();
            assert!(!error.contains("PRIVATE"));
        }
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key(id).as_bytes(),
                    1,
                    &serde_json::json!({"users":[valid]}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("primary-device-version:{peer}").as_bytes(),
                    1,
                    &"PRIVATE VALUE",
                )
            })
            .unwrap();
        let error = inspect(&store, id).unwrap_err().to_string();
        assert!(error.contains("invalid saved primary device version"));
        assert!(!error.contains("PRIVATE VALUE"));
    }
    #[test]
    fn java_receipt_time_keeps_received_companion_rows_during_identity_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("java-receipt.redb");
        let store = open(&path, true);
        let id = enqueue(&store);
        let peer = Jid::parse("15555550101:3@s.whatsapp.net").unwrap();
        let pending = peer.with_device(4);
        store
            .write(|tx| capture_in(tx, id, &[peer.clone(), pending.clone()]))
            .unwrap();
        let original = inspect(&store, id).unwrap();
        let node = crate::binary::Node::new("receipt").attr("t", "+２０");
        assert!(store.write(|tx| observe_in(tx, id, &peer, &node)).unwrap());
        assert!(
            store
                .write(|tx| observe_in(tx, id, &peer, &node.clone().attr("t", "١٠")))
                .unwrap()
        );
        let received = inspect(&store, id).unwrap();
        for value in ["𝟜𝟘", " 5", "5 ", "-1", "9223372036854775808"] {
            assert!(
                !store
                    .write(|tx| observe_in(tx, id, &peer, &node.clone().attr("t", value)))
                    .unwrap()
            );
            assert_eq!(inspect(&store, id).unwrap(), received);
        }
        let aborted: Result<()> = store.write(|tx| {
            after_companion_identity_change_in(tx, &pending)?;
            Err(invalid("fixture rollback"))
        });
        assert!(aborted.is_err());
        assert_eq!(inspect(&store, id).unwrap(), received);
        store
            .write(|tx| {
                after_companion_identity_change_in(tx, &peer)?;
                after_companion_identity_change_in(tx, &pending)
            })
            .unwrap();
        let saved = inspect(&store, id).unwrap();
        assert_eq!(saved.users, original.users);
        let rows = saved.device_receipts.as_ref().unwrap();
        let arrived = rows.iter().find(|r| r.device == 3).unwrap();
        assert!(arrived.present);
        assert_eq!(arrived.timestamp, Some(10000));
        assert_eq!(rows.iter().filter(|r| !r.present).count(), 1);
        drop(store);
        assert_eq!(inspect(&open(&path, false), id).unwrap(), saved);
    }
    #[test]
    fn send_versions_receipt_time_is_earliest_positive_exact_device_and_atomic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("receipt-time.redb");
        let store = open(&path, true);
        let id = enqueue(&store);
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        store
            .write(|tx| capture_in(tx, id, &[peer.clone(), peer.with_device(3)]))
            .unwrap();
        let original = inspect(&store, id).unwrap();
        // Cmk.A03 signed seconds and DEX wrapping multiplication; no local clock fallback.
        for value in [
            None,
            Some(""),
            Some("PRIVATE INVALID"),
            Some("0"),
            Some("-1"),
            Some("9223372036854775808"),
            Some("9223372036854775807"),
        ] {
            let mut node = crate::binary::Node::new("receipt");
            if let Some(value) = value {
                node = node.attr("t", value);
            }
            assert!(!store.write(|tx| observe_in(tx, id, &peer, &node)).unwrap());
            assert_eq!(inspect(&store, id).unwrap(), original);
        }
        let node = crate::binary::Node::new("receipt").attr("t", "20");
        for other in [
            peer.with_device(4),
            Jid::parse("222222222222222@lid").unwrap(),
        ] {
            assert!(!store.write(|tx| observe_in(tx, id, &other, &node)).unwrap());
        }
        assert!(store.write(|tx| observe_in(tx, id, &peer, &node)).unwrap());
        assert!(!store.write(|tx| observe_in(tx, id, &peer, &node)).unwrap());
        assert!(
            store
                .write(|tx| observe_in(tx, id, &peer, &node.clone().attr("t", "10")))
                .unwrap()
        );
        assert!(
            !store
                .write(|tx| observe_in(tx, id, &peer, &node.clone().attr("t", "30")))
                .unwrap()
        );
        let saved = inspect(&store, id).unwrap();
        assert_eq!(saved.users, original.users);
        assert_eq!(
            saved
                .device_receipts
                .as_ref()
                .unwrap()
                .iter()
                .filter(|r| r.timestamp == Some(10000))
                .count(),
            1
        );
        assert_eq!(
            saved
                .device_receipts
                .as_ref()
                .unwrap()
                .iter()
                .filter(|r| r.timestamp.is_none())
                .count(),
            1
        );
        let result: Result<()> = store.write(|tx| {
            observe_in(tx, id, &peer.with_device(3), &node)?;
            Err(invalid("fixture rollback"))
        });
        assert!(result.is_err());
        assert_eq!(inspect(&store, id).unwrap(), saved);
        // DEX mul-long wraps this valid negative seconds input to positive 1000.
        let wrapped = node.attr("t", "-9223372036854775807");
        assert!(
            store
                .write(|tx| observe_in(tx, id, &peer.with_device(3), &wrapped))
                .unwrap()
        );
        let saved = inspect(&store, id).unwrap();
        assert_eq!(
            saved
                .device_receipts
                .as_ref()
                .unwrap()
                .iter()
                .filter(|r| r.timestamp == Some(1000))
                .count(),
            1
        );
        drop(store);
        assert_eq!(inspect(&open(&path, false), id).unwrap(), saved);
    }

    #[test]
    fn send_versions_receipt_rows_refuse_corruption_and_do_not_backfill_history() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("receipt-corruption.redb"), true);
        let id = enqueue(&store);
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let node = crate::binary::Node::new("receipt").attr("t", "10");
        assert!(!store.write(|tx| observe_in(tx, id, &peer, &node)).unwrap());
        let snapshot = serde_json::json!({"users":[{"user":peer,"devices":1,"device_ids":[0],"version":null}]});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &snapshot))
            .unwrap();
        // Exact historical targets alone are not evidence that pending rows existed.
        assert!(!store.write(|tx| observe_in(tx, id, &peer, &node)).unwrap());
        assert!(inspect(&store, id).unwrap().device_receipts.is_none());
        let valid = serde_json::json!({"device":peer,"version":null,"timestamp":null});
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("timestamp");
        for rows in [
            serde_json::json!([missing]),
            serde_json::json!([]),
            serde_json::json!([valid.clone(), valid.clone()]),
            serde_json::json!([{"device":peer,"version":1,"timestamp":null}]),
            serde_json::json!([{"device":peer,"version":null,"timestamp":0}]),
            serde_json::json!([{"device":peer.with_device(3),"version":null,"timestamp":null}]),
            serde_json::json!("PRIVATE VALUE"),
        ] {
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        receipt_key(id).as_bytes(),
                        1,
                        &serde_json::json!({"rows":rows}),
                    )
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let error = store
                .write(|tx| observe_in(tx, id, &peer, &node))
                .unwrap_err()
                .to_string();
            assert!(!error.contains("PRIVATE"));
            assert!(!error.contains(&peer.user));
            assert!(inspect(&store, id).is_err());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
        }
    }
}
