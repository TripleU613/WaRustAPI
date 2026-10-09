//! Durable observations of differing public keys, never authority to replace pins.
//! APK 2.26.37.71 LX/0jc.A0K separates identity saving from verified state;
//! LX/0kc.A0I stores mark_as_verified/action_seq. Authenticated replacement is
//! separate; an unauthenticated envelope must remain an observation.
use crate::{
    Error, Result,
    model::{
        jid::{Jid, Server},
        time::Timestamp,
    },
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Redacted, first-seen observation. Fingerprints are SHA-256 of serialized
/// public keys, not WhatsApp safety numbers. No key or message is exposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityConflict {
    /// Stable local selector for this peer/pinned/observed combination.
    pub ticket: String,
    /// Local hash of the session address, without the address itself.
    pub peer_ticket: String,
    /// Public key fingerprint saved when this conflict was observed.
    pub pinned_fingerprint: String,
    /// Public key fingerprint carried by the unauthenticated prekey envelope.
    pub observed_fingerprint: String,
    /// First local observation time in milliseconds since the Unix epoch.
    pub observed_at: Timestamp,
}

pub(super) fn observe(
    store: &ProtocolStore,
    peer: &Jid,
    ciphertext: &[u8],
    pinned: Option<&[u8]>,
) -> Result<bool> {
    let Some(pinned) = pinned else {
        return Ok(false);
    };
    // Malformed envelopes remain the decryptor's responsibility. They cannot
    // create a conflict record, and storage failures must not be swallowed.
    let Ok((old, new)) =
        crate::signal::direct::incoming_prekey_identity_fingerprints(ciphertext, pinned)
    else {
        return Ok(false);
    };
    record(store, peer, pinned, old, new)
}

fn record(
    store: &ProtocolStore,
    peer: &Jid,
    pinned: &[u8],
    old: String,
    new: String,
) -> Result<bool> {
    if old == new {
        return Ok(false);
    }
    let peer_key = format!("comparison-direct:{peer}");
    let peer_ticket = hex::encode(Sha256::digest(peer_key.as_bytes()));
    let ticket = hex::encode(Sha256::digest(
        serde_json::to_vec(&(&peer_ticket, &old, &new))
            .map_err(|_| Error::invalid("identity_conflict", "invalid fingerprint tuple"))?,
    ));
    let key = format!("identity-conflict:{ticket}");
    store.write(|tx| {
        let current: Option<Vec<u8>> =
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        if current.as_deref() != Some(pinned) {
            return Err(Error::invalid(
                "identity_conflict",
                "saved pin changed during observation",
            ));
        }
        if tx
            .get_json::<IdentityConflict>(Table::Checkpoints, key.as_bytes(), 1)?
            .is_some()
        {
            return Ok(false);
        }
        tx.put_json(
            Table::Checkpoints,
            key.as_bytes(),
            1,
            &IdentityConflict {
                ticket,
                peer_ticket,
                pinned_fingerprint: old,
                observed_fingerprint: new,
                observed_at: Timestamp::now(),
            },
        )?;
        Ok(true)
    })
}

/// Inspect durable conflict observations offline. No session, pin, journal,
/// receipt or network state is changed; records do not assert authentication.
pub fn list(store: &ProtocolStore) -> Result<Vec<IdentityConflict>> {
    store.read(|tx| {
        let mut result = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(k, _)| k.clone());
            for (key, value) in rows {
                if key.starts_with(b"identity-conflict:") {
                    result.push(serde_json::from_slice(&value).map_err(|_| {
                        Error::invalid("identity_conflict", "invalid saved observation")
                    })?);
                }
            }
        }
        Ok(result)
    })
}

/// Saved E2EE device-list coverage, not the selected APK fingerprint inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDeviceKeyCoverage {
    /// Number of devices retained in the saved E2EE list, including the primary.
    pub retained_devices: usize,
    /// Retained devices with a valid saved public identity pin.
    pub pinned_devices: usize,
    /// Hashed session selectors for retained devices whose identity pin is absent.
    pub missing_pin_tickets: Vec<String>,
    /// Whether the saved companions came from a verified signed device list.
    pub companions_verified: bool,
    /// Current primary-key relationship of the historical signed list.
    #[serde(default)]
    pub device_authority: Option<super::devices::DeviceListAuthority>,
    /// Hosted devices excluded by the E2EE device-list implementation.
    pub hosted_devices_skipped: u32,
    /// Agreement with a saved notification hash, or null when no hash was tested.
    pub notified_hash_matches: Option<bool>,
}

/// Redacted pin and held-session state for one primary peer address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerPinState {
    /// Native-style primary-change counter for this exact namespace. Null before
    /// a tracked change; not server state, an inventory version or a timestamp.
    #[serde(default)]
    pub primary_device_version: Option<i64>,
    /// Indexed own keys awaiting rotation; zero does not prove historical coverage.
    #[serde(default)]
    pub pending_sender_key_rotations: usize,
    /// Hash of the session lookup key; never a raw contact address.
    pub peer_ticket: String,
    /// SHA-256 of serialized saved public identity, not a safety number.
    pub pinned_fingerprint: Option<String>,
    /// Whether a serialized session is held (not proof of its usability).
    pub has_session: bool,
    /// SHA-256 of the current session's remote public identity, if present.
    pub session_fingerprint: Option<String>,
    /// Null when either the saved pin or current session identity is missing.
    pub session_pin_matches: Option<bool>,
    /// Null without a saved device list. Even full retained-key coverage does
    /// not establish inventory completeness, current scheme, or verification.
    #[serde(default)]
    pub device_key_coverage: Option<PeerDeviceKeyCoverage>,
    /// Local manual claim only; null before the first decision. Not scan/server verification.
    #[serde(default)]
    pub verification: Option<super::peer_verification::PeerVerification>,
}

/// One authenticated incoming LID mapping and its relationship to the PN pin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedPeerIdentity {
    /// Saved pin and current session state for this LID address.
    pub state: PeerPinState,
    /// Null when either pin is missing; true only for byte-identical pins.
    pub pins_agree: Option<bool>,
    /// Saved authenticated history links the PN pin to the current LID pin.
    /// This is local provenance, never human safety-number verification.
    pub authenticated_transition_from_phone_pin: bool,
}

/// One transaction's view of the primary PN address and all mapped primary LIDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerIdentityState {
    /// Phone-number-addressed saved state.
    pub phone: PeerPinState,
    /// Every mapped primary LID; multiple entries are not silently selected.
    pub linked: Vec<LinkedPeerIdentity>,
}

fn pin_state(tx: &crate::store::ReadTx<'_>, peer: &Jid) -> Result<(PeerPinState, Option<Vec<u8>>)> {
    use wacore_libsignal::protocol::{IdentityKey, SessionRecord};
    let key = format!("comparison-direct:{peer}");
    let pin: Option<Vec<u8>> = tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?;
    if let Some(pin) = &pin {
        IdentityKey::decode(pin)
            .map_err(|_| Error::invalid("peer_identity", "invalid saved peer pin"))?;
    }
    let session: Option<Vec<u8>> = tx.get_json(Table::SignalSessions, key.as_bytes(), 1)?;
    let session_identity = if let Some(bytes) = &session {
        let record = SessionRecord::deserialize(bytes)
            .map_err(|_| Error::invalid("peer_identity", "invalid saved peer session"))?;
        record
            .session_state()
            .map(|state| state.remote_identity_key_bytes())
            .transpose()
            .map_err(|_| Error::invalid("peer_identity", "invalid session remote identity"))?
            .flatten()
    } else {
        None
    };
    let fingerprint = |bytes: &[u8]| hex::encode(Sha256::digest(bytes));
    let state = PeerPinState {
        primary_device_version: super::devices::primary_device_version_in(tx, peer)?,
        pending_sender_key_rotations: super::sender_key_distribution::pending_for_peer_in(
            tx, peer,
        )?,
        peer_ticket: hex::encode(Sha256::digest(key.as_bytes())),
        pinned_fingerprint: pin.as_deref().map(fingerprint),
        has_session: session.is_some(),
        session_fingerprint: session_identity.as_deref().map(fingerprint),
        session_pin_matches: pin
            .as_ref()
            .zip(session_identity.as_ref())
            .map(|(a, b)| a == b),
        device_key_coverage: device_key_coverage(tx, peer)?,
        verification: super::peer_verification::inspect_in(tx, peer, pin.as_deref())?,
    };
    Ok((state, pin))
}

fn device_key_coverage(
    tx: &crate::store::ReadTx<'_>,
    peer: &Jid,
) -> Result<Option<PeerDeviceKeyCoverage>> {
    use wacore_libsignal::protocol::IdentityKey;
    let Some(list) = super::devices::device_list_in(tx, peer)? else {
        return Ok(None);
    };
    // APK 37.71 JhJ.A0Z 003a..006a requires nonempty peer identity
    // lookup coverage for every selected device. Our retained E2EE list can
    // be a primary-only fallback and excludes hosted devices; it is not the
    // APK's selected inventory. Diagnose missing pins without computing a
    // purported complete safety number or fetching/replacing any key.
    let mut seen = std::collections::HashSet::new();
    let mut missing_pin_tickets = Vec::new();
    let mut pinned_devices = 0;
    for entry in &list.devices {
        if entry.jid.to_non_ad() != *peer || entry.jid.agent != 0 || !seen.insert(entry.jid.clone())
        {
            return Err(Error::invalid(
                "peer_identity",
                "invalid saved device inventory",
            ));
        }
        let key = format!("comparison-direct:{}", entry.jid);
        let pin: Option<Vec<u8>> = tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?;
        if let Some(pin) = pin {
            IdentityKey::decode(&pin).map_err(|_| {
                Error::invalid("peer_identity", "invalid saved device identity pin")
            })?;
            pinned_devices += 1;
        } else {
            missing_pin_tickets.push(hex::encode(Sha256::digest(key.as_bytes())));
        }
    }
    if list.owner != *peer || !list.devices.iter().any(|entry| entry.jid == *peer) {
        return Err(Error::invalid(
            "peer_identity",
            "saved device inventory lacks its primary owner",
        ));
    }
    missing_pin_tickets.sort();
    Ok(Some(PeerDeviceKeyCoverage {
        retained_devices: list.devices.len(),
        pinned_devices,
        missing_pin_tickets,
        companions_verified: list.companions_verified,
        device_authority: Some(super::devices::authority_in(tx, &list)?),
        hosted_devices_skipped: list.hosted_skipped,
        notified_hash_matches: list.hash_matches,
    }))
}

/// Inspect logical peer identity state without writes, queries or replacement.
/// Store opening can change engine file bytes; this function only reads records.
pub fn inspect_peer(store: &ProtocolStore, phone: &Jid) -> Result<PeerIdentityState> {
    if phone.server != Server::User || phone.device != 0 || phone.agent != 0 {
        return Err(Error::invalid(
            "peer_identity",
            "requires a primary phone-number chat JID",
        ));
    }
    store.read(|tx| inspect_peer_in(tx, phone))
}

pub(crate) fn inspect_peer_in(
    tx: &crate::store::ReadTx<'_>,
    phone: &Jid,
) -> Result<PeerIdentityState> {
    let (phone_state, phone_pin) = pin_state(tx, phone)?;
    let mut lids = Vec::new();
    let mut histories: Vec<serde_json::Value> = Vec::new();
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if rows.is_empty() {
            break;
        }
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, value) in rows {
            if let Some(lid) = std::str::from_utf8(&key)
                .ok()
                .and_then(|key| key.strip_prefix("authenticated-lid:"))
            {
                let mapped: Jid = serde_json::from_slice(&value)
                    .map_err(|_| Error::invalid("peer_identity", "invalid saved LID mapping"))?;
                if mapped.to_non_ad() != *phone {
                    continue;
                }
                let lid = Jid::parse(lid)
                    .map_err(|_| Error::invalid("peer_identity", "invalid saved LID address"))?;
                if lid.server == Server::Lid
                    && lid.device == 0
                    && lid.agent == 0
                    && !lids.contains(&lid)
                {
                    lids.push(lid);
                }
            } else if key.starts_with(b"identity-history:") {
                histories.push(serde_json::from_slice(&value).map_err(|_| {
                    Error::invalid("peer_identity", "invalid saved identity history")
                })?);
            }
        }
    }
    let mut linked = Vec::new();
    for lid in lids {
        let (state, pin) = pin_state(tx, &lid)?;
        let pins_agree = phone_pin.as_ref().zip(pin.as_ref()).map(|(a, b)| a == b);
        let authenticated_transition_from_phone_pin = pins_agree == Some(false)
            && histories.iter().any(|row| {
                let history_peer: Option<Jid> = serde_json::from_value(row["peer"].clone()).ok();
                let previous: Option<Vec<u8>> =
                    serde_json::from_value(row["previous_identity"].clone()).ok();
                history_peer.as_ref() == Some(&lid)
                    && row["policy"] == "apk_authenticated"
                    && matches!(
                        row["authentication"].as_str(),
                        None | Some("incoming_message")
                    )
                    && previous.as_ref() == phone_pin.as_ref()
                    && row["current_fingerprint"].as_str() == state.pinned_fingerprint.as_deref()
            });
        linked.push(LinkedPeerIdentity {
            state,
            pins_agree,
            authenticated_transition_from_phone_pin,
        });
    }
    Ok(PeerIdentityState {
        phone: phone_state,
        linked,
    })
}

/// Commit authenticated replacement history in the same transaction as inbox,
/// session and prekey changes. Call only after OCC checks and authentication.
pub(super) fn archive_replacement(
    tx: &mut crate::store::WriteTx<'_>,
    peer: &Jid,
    previous: Option<&[u8]>,
    previous_session: Option<&[u8]>,
    current: &[u8],
) -> Result<()> {
    archive_identity(
        tx,
        peer,
        previous,
        previous_session,
        current,
        "incoming_message",
    )
}

/// A policy-approved, signed directory response; preserve its different
/// authentication source rather than claiming an incoming message was read.
pub(super) fn archive_bundle_replacement(
    tx: &mut crate::store::WriteTx<'_>,
    peer: &Jid,
    previous: Option<&[u8]>,
    previous_session: Option<&[u8]>,
    current: &[u8],
) -> Result<()> {
    archive_identity(
        tx,
        peer,
        previous,
        previous_session,
        current,
        "signed_directory_bundle",
    )
}

fn archive_identity(
    tx: &mut crate::store::WriteTx<'_>,
    peer: &Jid,
    previous: Option<&[u8]>,
    previous_session: Option<&[u8]>,
    current: &[u8],
    authentication: &str,
) -> Result<()> {
    let Some(previous) = previous.filter(|old| *old != current) else {
        return Ok(());
    };
    super::peer_verification::invalidate_in(tx, peer, current)?;
    super::devices::after_primary_identity_change_in(tx, peer)?;
    super::sender_key_distribution::after_primary_change_in(tx, peer)?;
    super::send_versions::after_companion_identity_change_in(tx, peer)?;
    let observed_at = Timestamp::now();
    let event = tx.append_event(crate::sync::SyncEvent::IdentityChanged(
        crate::sync::IdentityChange {
            device: crate::model::ids::DeviceAddress::from_jid(peer)?,
            previous_fingerprint: Some(hex::encode(Sha256::digest(previous))),
            new_fingerprint: hex::encode(Sha256::digest(current)),
            observed_at,
        },
    ))?;
    // Unique journal event selector preserves repeated A->B->A transitions;
    // a deterministic old/new selector would overwrite distinct ratchets.
    let key = format!("identity-history:{}", hex::encode(event.event_id.0));
    tx.put_json(
        Table::Checkpoints,
        key.as_bytes(),
        1,
        &serde_json::json!({
            "peer":peer, "previous_identity":previous, "previous_session":previous_session,
            "current_fingerprint":hex::encode(Sha256::digest(current)), "at":observed_at,
            "verification":"unverified", "policy":"apk_authenticated",
            "authentication":authentication,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create,
        })
        .unwrap()
    }
    #[test]
    fn primary_version_covers_both_archive_sources_and_preserves_receipt_reservations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("primary-version.redb");
        let store = open(&path, true);
        let pn = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let lid = Jid::parse("222222222222222@lid").unwrap();
        let version = |store: &ProtocolStore, peer: &Jid| {
            store
                .read(|tx| super::super::devices::primary_device_version_in(tx, peer))
                .unwrap()
        };
        let receipt = serde_json::json!({"state":"uncertain","id":"synthetic-stable-receipt"});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, b"receipt-outbox:synthetic", 1, &receipt))
            .unwrap();
        assert_eq!(version(&store, &pn), None);
        store
            .write(|tx| {
                archive_replacement(tx, &pn, None, None, &[2])?;
                archive_bundle_replacement(tx, &pn, Some(&[2]), None, &[2])
            })
            .unwrap();
        assert_eq!(version(&store, &pn), None);
        store
            .write(|tx| archive_replacement(tx, &pn, Some(&[1]), None, &[2]))
            .unwrap();
        assert_eq!(version(&store, &pn), Some(1));
        store
            .write(|tx| archive_bundle_replacement(tx, &pn, Some(&[2]), None, &[1]))
            .unwrap();
        assert_eq!(version(&store, &pn), Some(2));
        assert_eq!(version(&store, &lid), None);
        store
            .write(|tx| archive_bundle_replacement(tx, &lid, Some(&[1]), None, &[2]))
            .unwrap();
        assert_eq!(version(&store, &lid), Some(1));
        assert_eq!(version(&store, &pn), Some(2));
        let head = store.journal_head().unwrap();
        let failed: Result<()> = store.write(|tx| {
            archive_replacement(tx, &pn, Some(&[1]), None, &[3])?;
            Err(Error::invalid("test", "rollback"))
        });
        assert!(failed.is_err());
        assert_eq!(version(&store, &pn), Some(2));
        assert_eq!(store.journal_head().unwrap(), head);
        drop(store);
        let store = open(&path, false);
        assert_eq!(version(&store, &pn), Some(2));
        assert_eq!(version(&store, &lid), Some(1));
        assert_eq!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    b"receipt-outbox:synthetic",
                    1
                ))
                .unwrap(),
            Some(receipt)
        );
    }

    #[test]
    fn retained_device_key_coverage_is_redacted_read_only_and_not_inventory_authority() {
        use super::super::devices::{DeviceEntry, DeviceList};
        use wacore_libsignal::protocol::IdentityKeyPair;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("coverage.redb");
        let store = open(&path, true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let companion = Jid::parse("15555550100:2@s.whatsapp.net").unwrap();
        assert!(
            inspect_peer(&store, &peer)
                .unwrap()
                .phone
                .device_key_coverage
                .is_none()
        );
        let old_state = inspect_peer(&store, &peer).unwrap().phone;
        let mut old_json = serde_json::to_value(&old_state).unwrap();
        old_json
            .as_object_mut()
            .unwrap()
            .remove("device_key_coverage");
        old_json
            .as_object_mut()
            .unwrap()
            .remove("primary_device_version");
        assert_eq!(
            serde_json::from_value::<PeerPinState>(old_json).unwrap(),
            old_state
        );
        let primary_key = IdentityKeyPair::generate(&mut rand::rng())
            .identity_key()
            .serialize()
            .to_vec();
        let companion_key = IdentityKeyPair::generate(&mut rand::rng())
            .identity_key()
            .serialize()
            .to_vec();
        let primary_selector = format!("comparison-direct:{peer}");
        let companion_selector = format!("comparison-direct:{companion}");
        let list_selector = format!("devices:{peer}");
        let mut list = DeviceList {
            owner: peer.clone(),
            devices: vec![DeviceEntry {
                jid: peer.clone(),
                key_index: 0,
            }],
            raw_id: None,
            list_timestamp: None,
            expected_timestamp: None,
            companions_verified: false,
            adv_added_devices: Vec::new(),
            pending_removed_devices: None,
            verified_primary_key_fingerprint: None,
            unverified_reason: Some("no signed list".into()),
            notified_hash: None,
            hash_matches: None,
            hosted_skipped: 1,
            fetched_at: Timestamp(1),
        };
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    primary_selector.as_bytes(),
                    1,
                    &primary_key,
                )?;
                tx.put_json(Table::Checkpoints, list_selector.as_bytes(), 1, &list)
            })
            .unwrap();
        let coverage = inspect_peer(&store, &peer)
            .unwrap()
            .phone
            .device_key_coverage
            .unwrap();
        assert_eq!(coverage.retained_devices, 1);
        assert_eq!(coverage.pinned_devices, 1);
        assert!(!coverage.companions_verified);
        assert_eq!(coverage.hosted_devices_skipped, 1);
        // All retained fallback keys can be present without an authenticated
        // complete device list. Do not expose a "fingerprint ready" assertion.
        assert!(!serde_json::to_string(&coverage).unwrap().contains("ready"));
        list.devices.push(DeviceEntry {
            jid: companion.clone(),
            key_index: 7,
        });
        list.companions_verified = true;
        list.hash_matches = Some(false);
        store
            .write(|tx| tx.put_json(Table::Checkpoints, list_selector.as_bytes(), 1, &list))
            .unwrap();
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan_after(Table::Checkpoints, 1, None, 64)?,
                    tx.scan_after(Table::PeerIdentities, 1, None, 64)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 64)?,
                ))
            })
            .unwrap();
        let inspected = inspect_peer(&store, &peer).unwrap();
        let coverage = inspected.phone.device_key_coverage.as_ref().unwrap();
        assert_eq!(coverage.retained_devices, 2);
        assert_eq!(coverage.pinned_devices, 1);
        assert_eq!(
            coverage.missing_pin_tickets,
            vec![hex::encode(Sha256::digest(companion_selector.as_bytes()))]
        );
        assert_eq!(coverage.notified_hash_matches, Some(false));
        let encoded = serde_json::to_string(&inspected).unwrap();
        assert!(!encoded.contains(&peer.user));
        assert!(!encoded.contains(&hex::encode(&primary_key)));
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan_after(Table::Checkpoints, 1, None, 64)?,
                    tx.scan_after(Table::PeerIdentities, 1, None, 64)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 64)?,
                )))
                .unwrap(),
            before
        );
        drop(store);
        let store = open(&path, false);
        assert_eq!(inspect_peer(&store, &peer).unwrap(), inspected);
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    companion_selector.as_bytes(),
                    1,
                    &companion_key,
                )
            })
            .unwrap();
        let coverage = inspect_peer(&store, &peer)
            .unwrap()
            .phone
            .device_key_coverage
            .unwrap();
        assert_eq!(coverage.pinned_devices, 2);
        assert!(coverage.missing_pin_tickets.is_empty());
        // Neither coverage inspection nor corrupted-pin refusal rotates keys.
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    companion_selector.as_bytes(),
                    1,
                    &vec![0u8; 3],
                )
            })
            .unwrap();
        assert!(inspect_peer(&store, &peer).is_err());
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    companion_selector.as_bytes(),
                    1,
                    &companion_key,
                )?;
                list.devices.push(list.devices[0].clone());
                tx.put_json(Table::Checkpoints, list_selector.as_bytes(), 1, &list)
            })
            .unwrap();
        assert!(inspect_peer(&store, &peer).is_err());
        list.devices = vec![DeviceEntry {
            jid: companion,
            key_index: 7,
        }];
        store
            .write(|tx| tx.put_json(Table::Checkpoints, list_selector.as_bytes(), 1, &list))
            .unwrap();
        assert!(inspect_peer(&store, &peer).is_err());
    }

    #[tokio::test]
    async fn peer_inspection_distinguishes_alias_history_from_pin_and_session_agreement() {
        use crate::session::{AccountRecord, KeyMaterial, RegistrationStatus, SignalIdentityKey};
        use wacore_libsignal::protocol::{DeviceId, IdentityKeyPair, KeyPair, PreKeyBundle};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("inspection.redb");
        let store = open(&path, true);
        let pn = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let lid = Jid::parse("222222222222222@lid").unwrap();
        let mut rng = rand::rng();
        let local = IdentityKeyPair::generate(&mut rng);
        let old = IdentityKeyPair::generate(&mut rng);
        let new = IdentityKeyPair::generate(&mut rng);
        let signed = KeyPair::generate(&mut rng);
        let bundle = |identity: &IdentityKeyPair| {
            let signature = identity
                .private_key()
                .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
                .unwrap();
            PreKeyBundle::new(
                42,
                DeviceId::new(0),
                None,
                7.into(),
                signed.public_key,
                signature.to_vec(),
                *identity.identity_key(),
            )
            .unwrap()
        };
        let account = AccountRecord {
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: local.public_key().public_key_bytes().try_into().unwrap(),
                private: *local.private_key().serialize(),
            })),
            ..Default::default()
        };
        let prior = crate::signal::direct::prepare(
            &account,
            &pn,
            &bundle(&old),
            None,
            None,
            "offline fixture",
        )
        .await
        .unwrap();
        let updated = crate::signal::direct::prepare(
            &account,
            &lid,
            &bundle(&new),
            None,
            None,
            "offline fixture",
        )
        .await
        .unwrap();
        let pn_key = format!("comparison-direct:{pn}");
        let lid_key = format!("comparison-direct:{lid}");
        store
            .write(|tx| {
                tx.put_json(Table::PeerIdentities, pn_key.as_bytes(), 1, &prior.identity)?;
                tx.put_json(Table::SignalSessions, pn_key.as_bytes(), 1, &*prior.session)?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("authenticated-lid:{lid}").as_bytes(),
                    1,
                    &pn,
                )?;
                tx.put_json(
                    Table::PeerIdentities,
                    lid_key.as_bytes(),
                    1,
                    &updated.identity,
                )?;
                tx.put_json(
                    Table::SignalSessions,
                    lid_key.as_bytes(),
                    1,
                    &*updated.session,
                )?;
                // Seed the same history format produced after authenticated receive.
                archive_replacement(
                    tx,
                    &lid,
                    Some(&prior.identity),
                    Some(&prior.session),
                    &updated.identity,
                )?;
                for n in 0..260 {
                    tx.put_json(
                        Table::Checkpoints,
                        format!("a-inspection-unrelated:{n:03}").as_bytes(),
                        1,
                        &true,
                    )?;
                }
                Ok(())
            })
            .unwrap();
        let inspected = inspect_peer(&store, &pn).unwrap();
        assert_eq!(inspected.phone.session_pin_matches, Some(true));
        assert_eq!(inspected.linked.len(), 1);
        assert_eq!(inspected.linked[0].state.session_pin_matches, Some(true));
        assert_eq!(inspected.linked[0].pins_agree, Some(false));
        assert!(inspected.linked[0].authenticated_transition_from_phone_pin);
        let encoded = serde_json::to_string(&inspected).unwrap();
        assert!(!encoded.contains(&pn.user));
        assert!(!encoded.contains(&lid.user));
        let before = store
            .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
            .unwrap();
        assert_eq!(inspect_peer(&store, &pn).unwrap(), inspected);
        assert_eq!(
            store
                .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                .unwrap(),
            before
        );
        drop(store);
        let store = open(&path, false);
        assert_eq!(inspect_peer(&store, &pn).unwrap(), inspected);
        // A pin that disagrees with its own held ratchet is visible separately
        // from PN/LID agreement. A historical transition to another current pin
        // is not qualifying evidence; no alias is silently overwritten.
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    lid_key.as_bytes(),
                    1,
                    &prior.identity,
                )
            })
            .unwrap();
        let agreed = inspect_peer(&store, &pn).unwrap();
        assert_eq!(agreed.linked[0].pins_agree, Some(true));
        assert_eq!(agreed.linked[0].state.session_pin_matches, Some(false));
        assert!(!agreed.linked[0].authenticated_transition_from_phone_pin);
        let second = Jid::parse("333333333333333@lid").unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("authenticated-lid:{second}").as_bytes(),
                    1,
                    &pn,
                )
            })
            .unwrap();
        let multiple = inspect_peer(&store, &pn).unwrap();
        assert_eq!(multiple.linked.len(), 2);
        let missing = multiple
            .linked
            .iter()
            .find(|entry| !entry.state.has_session)
            .unwrap();
        assert_eq!(missing.state.pinned_fingerprint, None);
        assert_eq!(missing.pins_agree, None);
        assert!(!missing.authenticated_transition_from_phone_pin);
        assert!(inspect_peer(&store, &lid).is_err());
        assert!(
            inspect_peer(&store, &Jid::parse("15555550100:2@s.whatsapp.net").unwrap()).is_err()
        );
        let unknown =
            inspect_peer(&store, &Jid::parse("15555550101@s.whatsapp.net").unwrap()).unwrap();
        assert!(!unknown.phone.has_session);
        assert_eq!(unknown.phone.session_pin_matches, None);
        assert!(unknown.linked.is_empty());
    }

    #[tokio::test]
    async fn incoming_prekey_conflict_is_recorded_before_decryption_can_fail() {
        use wacore_libsignal::protocol::{IdentityKeyPair, PreKeySignalMessage, SignalMessage};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("incoming.redb"), true);
        let mut rng = rand::rng();
        let saved = IdentityKeyPair::generate(&mut rng);
        let observed = IdentityKeyPair::generate(&mut rng);
        let receiver = IdentityKeyPair::generate(&mut rng);
        let peer = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let peer_key = format!("comparison-direct:{peer}");
        let pin = saved.identity_key().serialize().to_vec();
        store
            .write(|tx| tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &pin))
            .unwrap();
        let nested = SignalMessage::new(
            3,
            &[1; 32],
            *observed.public_key(),
            0,
            0,
            b"not authenticated plaintext",
            observed.identity_key(),
            receiver.identity_key(),
        )
        .unwrap();
        let wire = PreKeySignalMessage::new(
            3,
            17,
            None,
            7.into(),
            *observed.public_key(),
            *observed.identity_key(),
            nested,
        )
        .unwrap();
        let node = crate::binary::Node::new("message")
            .attr("from", peer)
            .attr("id", "conflict-test")
            .attr("type", "text")
            .children(vec![
                crate::binary::Node::new("enc")
                    .attr("type", "pkmsg")
                    .attr("v", "2")
                    .bytes(wire.serialized().to_vec()),
            ]);
        let capture = super::super::inbox::preserve(&store, &node).unwrap();
        // No registered signed prekey: decryption cannot run, but the observed
        // public identity difference must remain diagnosable and not trusted.
        let _ = super::super::inbox::process(
            &store,
            &crate::session::AccountRecord::default(),
            &node,
            &capture,
        )
        .await;
        let rows = list(&store).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].observed_fingerprint,
            hex::encode(Sha256::digest(observed.identity_key().serialize()))
        );
        let _ = super::super::inbox::process(
            &store,
            &crate::session::AccountRecord::default(),
            &node,
            &capture,
        )
        .await;
        assert_eq!(list(&store).unwrap(), rows);
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::PeerIdentities, peer_key.as_bytes(), 1))
                .unwrap(),
            Some(pin)
        );
        assert_eq!(store.read(|tx| tx.count(Table::SignalSessions)).unwrap(), 0);
    }
    #[test]
    fn observations_survive_restart_without_replacing_pins_or_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("conflict.redb");
        let store = open(&path, true);
        let peer = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let key = format!("comparison-direct:{peer}");
        let pin = vec![5u8; 33];
        store
            .write(|tx| {
                tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &pin)?;
                tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &vec![7u8; 20])
            })
            .unwrap();
        let old = hex::encode(Sha256::digest(&pin));
        let new = hex::encode(Sha256::digest([6u8; 33]));
        assert!(!record(&store, &peer, &pin, old.clone(), old.clone()).unwrap());
        assert!(record(&store, &peer, &pin, old.clone(), new.clone()).unwrap());
        let rows = list(&store).unwrap();
        let output = serde_json::to_string(&rows).unwrap();
        assert!(!output.contains(&peer.user));
        assert_eq!(rows.len(), 1);
        assert!(!observe(&store, &peer, b"malformed", Some(&pin)).unwrap());
        assert!(!observe(&store, &peer, b"malformed", None).unwrap());
        drop(store);
        let store = open(&path, false);
        assert!(!record(&store, &peer, &pin, old.clone(), new.clone()).unwrap());
        assert_eq!(list(&store).unwrap(), rows);
        assert!(record(&store, &peer, &[8u8; 33], old, new).is_err());
        store
            .read(|tx| {
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1)?,
                    Some(pin)
                );
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?,
                    Some(vec![7; 20])
                );
                Ok(())
            })
            .unwrap();
    }

    #[tokio::test]
    async fn authenticated_replacement_is_atomic_for_direct_and_group_messages() {
        use crate::{
            binary::Node,
            registration::request::{IdentityChangePolicy, LabRegistrationConfig},
            session::{AccountRecord, KeyMaterial, RegistrationStatus, SignalIdentityKey},
        };
        use wacore_libsignal::protocol::{
            DeviceId, GenericSignedPreKey, IdentityKey, IdentityKeyPair, PreKeyBundle,
            SessionRecord, SignedPreKeyRecord,
        };
        for group in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("replace.redb");
            let store = open(&path, true);
            let mut config: LabRegistrationConfig =
                serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
            config.registration_id = 42;
            config.signed_pre_key_id = 7;
            crate::registration::factory::install_test_material(&store, &config).unwrap();
            let receiver = store.read(AccountRecord::load).unwrap().unwrap();
            let signed = crate::registration::factory::registered_signed_prekey(&store).unwrap();
            let signed = SignedPreKeyRecord::deserialize(&signed).unwrap();
            let local = receiver.signal_identity.as_ref().unwrap();
            let mut serialized_identity = vec![5];
            serialized_identity.extend(local.0.public);
            let bundle = PreKeyBundle::new(
                42,
                DeviceId::new(0),
                None,
                7.into(),
                signed.public_key().unwrap(),
                signed.signature().unwrap().to_vec(),
                IdentityKey::decode(&serialized_identity).unwrap(),
            )
            .unwrap();
            let mut rng = rand::rng();
            let alice = IdentityKeyPair::generate(&mut rng);
            let old = IdentityKeyPair::generate(&mut rng)
                .identity_key()
                .serialize()
                .to_vec();
            let sender = Jid::parse("15557654321@s.whatsapp.net").unwrap();
            let session_peer = if group {
                Jid::parse("123456789@lid").unwrap()
            } else {
                sender.clone()
            };
            let peer_key = format!("comparison-direct:{session_peer}");
            let old_session = SessionRecord::new_fresh().serialize().unwrap();
            store
                .write(|tx| {
                    tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &old)?;
                    tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &old_session)?;
                    if group {
                        // A separate incompatible PN session must not displace
                        // the sender's LID ratchet merely because its alias exists.
                        tx.put_json(
                            Table::SignalSessions,
                            format!("comparison-direct:{sender}").as_bytes(),
                            1,
                            &vec![4u8; 20],
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
            let sending = AccountRecord {
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(31),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public: alice.public_key().public_key_bytes().try_into().unwrap(),
                    private: *alice.private_key().serialize(),
                })),
                ..Default::default()
            };
            let prepared = crate::signal::direct::prepare(
                &sending,
                &receiver.own_jid.clone().unwrap(),
                &bundle,
                None,
                None,
                "identity test",
            )
            .await
            .unwrap();
            let node = |id: &str, ciphertext: Vec<u8>| {
                let mut n = Node::new("message").attr("id", id).attr("type", "text");
                n = if group {
                    n.attr("from", "120363000000000001@g.us")
                        .attr("participant", session_peer.clone())
                        .attr("participant_pn", sender.clone())
                } else {
                    n.attr("from", sender.clone())
                };
                n.children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "pkmsg")
                        .bytes(ciphertext),
                ])
            };
            let n = node("identity-good", prepared.ciphertext.clone());
            let capture = super::super::inbox::preserve(&store, &n).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &n, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Undecryptable
            );
            let initial_head = store.journal_head().unwrap();
            let mut damaged = prepared.ciphertext.clone();
            let envelope = wacore_libsignal::protocol::PreKeySignalMessage::try_from(
                prepared.ciphertext.as_slice(),
            )
            .unwrap();
            let nested = envelope.message().serialized();
            let start = damaged
                .windows(nested.len())
                .position(|w| w == nested)
                .unwrap();
            damaged[start + nested.len() - 1] ^= 1;
            let bad = node("identity-bad", damaged);
            let bad_capture = super::super::inbox::preserve(&store, &bad).unwrap();
            assert_eq!(
                super::super::inbox::process_with_policy(
                    &store,
                    &receiver,
                    &bad,
                    &bad_capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                super::super::inbox::IncomingResult::Undecryptable
            );
            assert_eq!(store.journal_head().unwrap(), initial_head);
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<Vec<u8>>(
                        Table::PeerIdentities,
                        peer_key.as_bytes(),
                        1
                    ))
                    .unwrap(),
                Some(old.clone())
            );
            assert_eq!(
                super::super::inbox::process_with_policy(
                    &store,
                    &receiver,
                    &n,
                    &capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                super::super::inbox::IncomingResult::Decrypted
            );
            assert_eq!(
                store
                    .read(|tx| super::super::devices::primary_device_version_in(tx, &session_peer))
                    .unwrap(),
                Some(1)
            );
            let head = store.journal_head().unwrap();
            assert!(head > initial_head);
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<Vec<u8>>(
                        Table::PeerIdentities,
                        peer_key.as_bytes(),
                        1
                    ))
                    .unwrap(),
                Some(alice.identity_key().serialize().to_vec())
            );
            let archives = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 256))
                .unwrap();
            let archives: Vec<_> = archives
                .into_iter()
                .filter(|(k, _)| k.starts_with(b"identity-history:"))
                .collect();
            assert_eq!(archives.len(), 1);
            let history: serde_json::Value = serde_json::from_slice(&archives[0].1).unwrap();
            assert_eq!(history["previous_identity"], serde_json::json!(old));
            assert_eq!(history["previous_session"], serde_json::json!(old_session));
            assert_eq!(history["verification"], "unverified");
            // Aborting a replacement transaction rolls back both its archive
            // and its journal event, including a repeated key transition.
            let before = store.journal_head().unwrap();
            let failed: Result<()> = store.write(|tx| {
                archive_replacement(
                    tx,
                    &session_peer,
                    Some(&alice.identity_key().serialize()),
                    Some(&old_session),
                    &old,
                )?;
                Err(Error::invalid("test", "simulated commit refusal"))
            });
            assert!(failed.is_err());
            assert_eq!(store.journal_head().unwrap(), before);
            assert_eq!(
                store
                    .read(|tx| super::super::devices::primary_device_version_in(tx, &session_peer))
                    .unwrap(),
                Some(1)
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 256))
                    .unwrap()
                    .iter()
                    .filter(|(k, _)| k.starts_with(b"identity-history:"))
                    .count(),
                1
            );
            // Duplicate after reopening must not rotate again or append history/events.
            drop(store);
            let store = open(&path, false);
            assert_eq!(
                super::super::inbox::process_with_policy(
                    &store,
                    &receiver,
                    &n,
                    &capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                super::super::inbox::IncomingResult::Duplicate
            );
            assert_eq!(store.journal_head().unwrap(), head);
            assert_eq!(
                store
                    .read(|tx| super::super::devices::primary_device_version_in(tx, &session_peer))
                    .unwrap(),
                Some(1)
            );
        }
    }
}
