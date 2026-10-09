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
}

// A missing JSON member is malformed, distinct from an explicit null counter.
fn nullable_version<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<i64>, D::Error> {
    Option::<i64>::deserialize(d)
}

/// Redacted relationship of one actual send target's user to current state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserVersion {
    /// SHA-256 of normalized user JID; never a phone number or safety number.
    pub user_ticket: String,
    /// Number of unique actual target devices for this user at reservation.
    pub devices: usize,
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
            version,
        });
    }
    let key = key(id);
    if tx.get(Table::Checkpoints, key.as_bytes(), 1)?.is_some() {
        return Err(invalid("original send version snapshot already exists"));
    }
    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &snapshot)
}

/// Inspect one existing outbox intent without queries, writes or resend decisions.
pub fn inspect(store: &ProtocolStore, id: LocalMessageId) -> Result<SendVersions> {
    store.read(|tx| {
        Outbox::new("comparison-driver", Limits::default())
            .get(tx, id)?
            .ok_or_else(|| invalid("unknown outbox intent"))?;
        let snapshot: Option<Snapshot> = tx
            .get(Table::Checkpoints, key(id).as_bytes(), 1)?
            .map(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|_| invalid("invalid saved version snapshot"))
            })
            .transpose()?;
        let mut result = SendVersions {
            local_id: id,
            captured: snapshot.is_some(),
            users: Vec::new(),
        };
        if let Some(snapshot) = snapshot {
            if snapshot.users.is_empty() {
                return Err(invalid("saved version snapshot is empty"));
            }
            let mut seen = BTreeSet::new();
            for row in snapshot.users {
                if !primary(&row.user) || row.devices == 0 || !seen.insert(row.user.clone()) {
                    return Err(invalid("invalid saved version snapshot user"));
                }
                let current = super::devices::primary_device_version_in(tx, &row.user)?;
                result.users.push(UserVersion {
                    user_ticket: hex::encode(Sha256::digest(row.user.to_string().as_bytes())),
                    devices: row.devices,
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
        assert_eq!(store.journal_head().unwrap(), head);
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
}
