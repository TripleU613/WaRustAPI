//! Own sender-key exposure history for the supported comparison sender.
//! APK 37.71 D4x.A00 selects groups with sent_sender_key=1 before scoped
//! 0jc.A19 removal. We conservatively record reservation, since a missing ACK
//! does not establish that the recipient never received the distribution.
use crate::{
    Error, Result,
    model::jid::Jid,
    store::{ReadTx, Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PREFIX: &[u8] = b"sender-key-distribution:";

/// Private snapshot used for optimistic concurrency; addresses stay encrypted.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributionState {
    own: Jid,
    recipients: Vec<Jid>,
    rotation_pending: bool,
    /// Random identity of the current own chain, so a late ACK for an older
    /// chain cannot mark devices of a newer one. 0 in pre-existing states.
    #[serde(default)]
    chain: u64,
    /// Devices the server ACKed a send carrying this chain's distribution to:
    /// 2.26.37.71 `group_participant_device.sent_sender_key = 1`, which
    /// `SendE2EMessageJob.A06` sets only after the ACK (`0bA.A0U/A0V/A0X`,
    /// "…after ack"). Such devices are not sent the distribution again.
    #[serde(default)]
    acked: Vec<Jid>,
}

impl std::fmt::Debug for DistributionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DistributionState")
            .field("recipient_count", &self.recipients.len())
            .field("rotation_pending", &self.rotation_pending)
            .finish_non_exhaustive()
    }
}

impl DistributionState {
    /// Predict the same primary-only invalidation used by the identity archive,
    /// so a fanout stages its new generation before committing that archive.
    pub(super) fn after_primary_changes(&self, peers: &[Jid]) -> Self {
        let mut state = self.clone();
        state.rotation_pending |= peers.iter().any(|peer| {
            peer.device == 0
                && peer.agent == 0
                && matches!(
                    peer.server,
                    crate::model::jid::Server::User | crate::model::jid::Server::Lid
                )
                && state.own.to_non_ad() != peer.to_non_ad()
                && state
                    .recipients
                    .iter()
                    .any(|r| r.to_non_ad() == peer.to_non_ad())
        });
        state
    }
    /// Whether the next newly reserved send must mint a fresh own sender key.
    pub fn rotation_pending(&self) -> bool {
        self.rotation_pending
    }
    /// Whether `device` already holds this chain's sender key (server-ACKed
    /// distribution) and need not be sent it again. Never true while a
    /// rotation is pending: a new chain must reach every device.
    pub fn delivered(&self, device: &Jid) -> bool {
        !self.rotation_pending && self.acked.contains(device)
    }
}

fn new_chain_id() -> Result<u64> {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes)
        .map_err(|_| Error::invalid("group_send", "OS random source unavailable"))?;
    Ok(u64::from_le_bytes(bytes))
}

fn key(chat: &Jid, own: &Jid) -> Vec<u8> {
    let selector = crate::signal::group::sender_key_db_key(chat, own);
    format!(
        "sender-key-distribution:{}",
        hex::encode(Sha256::digest(selector))
    )
    .into_bytes()
}

/// Load the exact own-key exposure snapshot; absent means historical coverage unknown.
pub fn load_in(tx: &ReadTx<'_>, chat: &Jid, own: &Jid) -> Result<Option<DistributionState>> {
    tx.get_json(Table::Checkpoints, &key(chat, own), 1)
}

/// Check the original exposure before a composed identity archive can mark it
/// pending. A later real rotation must not be hidden by our predicted change.
pub(super) fn validate_in(
    tx: &WriteTx<'_>,
    chat: &Jid,
    own: &Jid,
    expected: Option<&DistributionState>,
) -> Result<()> {
    let current: Option<DistributionState> = tx.get_json(Table::Checkpoints, &key(chat, own), 1)?;
    if current.as_ref() != expected {
        return Err(Error::invalid(
            "group_send",
            "sender-key distribution changed before commit",
        ));
    }
    Ok(())
}

/// Reserve exposure atomically with own-key/pairwise state and the send intent.
/// Replacing a pending snapshot starts a fresh chain's recipient history.
pub fn reserve_in(
    tx: &mut WriteTx<'_>,
    chat: &Jid,
    own: &Jid,
    expected: Option<&DistributionState>,
    recipients: &[Jid],
) -> Result<()> {
    let selector = key(chat, own);
    let current: Option<DistributionState> = tx.get_json(Table::Checkpoints, &selector, 1)?;
    if current.as_ref() != expected {
        return Err(Error::invalid(
            "group_send",
            "sender-key distribution changed before commit",
        ));
    }
    let mut state = match current {
        Some(state) if !state.rotation_pending => state,
        _ => DistributionState {
            own: own.clone(),
            recipients: Vec::new(),
            rotation_pending: false,
            chain: new_chain_id()?,
            acked: Vec::new(),
        },
    };
    for peer in recipients {
        if !peer.is_user_like() {
            return Err(Error::invalid(
                "group_send",
                "sender-key recipient must be a user device",
            ));
        }
        if !state.recipients.contains(peer) {
            state.recipients.push(peer.clone());
        }
    }
    state.recipients.sort();
    tx.put_json(Table::Checkpoints, &selector, 1, &state)
}

#[derive(Serialize, Deserialize)]
struct PendingAck {
    chat: Jid,
    own: Jid,
    chain: u64,
    devices: Vec<Jid>,
}

fn pending_key(wire_id: &str) -> Vec<u8> {
    format!("sender-key-distribution-ack:{wire_id}").into_bytes()
}

/// Remember, in the send's reservation transaction, which devices this stanza
/// carries the current chain's distribution to. Call after [`reserve_in`].
pub fn pending_ack_in(
    tx: &mut WriteTx<'_>,
    chat: &Jid,
    own: &Jid,
    wire_id: &str,
    devices: &[Jid],
) -> Result<()> {
    if devices.is_empty() {
        return Ok(());
    }
    let state: DistributionState = tx
        .get_json(Table::Checkpoints, &key(chat, own), 1)?
        .ok_or_else(|| Error::invalid("group_send", "distribution state missing after reserve"))?;
    let record = PendingAck {
        chat: chat.clone(),
        own: own.clone(),
        chain: state.chain,
        devices: devices.to_vec(),
    };
    tx.put_json(Table::Checkpoints, &pending_key(wire_id), 1, &record)
}

/// On the server's successful ACK of `wire_id`, mark its distribution
/// recipients as holding the chain (APK `sent_sender_key = 1` after ack).
/// A rotated or replaced chain since the reservation marks nothing.
/// No record (not a group send, or nothing distributed) is a no-op.
pub fn confirm_ack_in(tx: &mut WriteTx<'_>, wire_id: &str) -> Result<()> {
    let Some(record): Option<PendingAck> =
        tx.get_json(Table::Checkpoints, &pending_key(wire_id), 1)?
    else {
        return Ok(());
    };
    tx.delete(Table::Checkpoints, &pending_key(wire_id))?;
    let selector = key(&record.chat, &record.own);
    let Some(mut state): Option<DistributionState> =
        tx.get_json(Table::Checkpoints, &selector, 1)?
    else {
        return Ok(());
    };
    if state.rotation_pending || state.chain != record.chain {
        return Ok(());
    }
    for device in record.devices {
        if state.recipients.contains(&device) && !state.acked.contains(&device) {
            state.acked.push(device);
        }
    }
    state.acked.sort();
    tx.put_json(Table::Checkpoints, &selector, 1, &state)
}

/// Invalidate only indexed own keys exposed to this changed primary namespace.
/// Does not remove sender records, replay sends, or infer PN/LID equivalence.
pub(super) fn after_primary_change_in(tx: &mut WriteTx<'_>, peer: &Jid) -> Result<()> {
    use crate::model::jid::Server;
    if peer.device != 0 || peer.agent != 0 || !matches!(peer.server, Server::User | Server::Lid) {
        return Ok(());
    }
    // The primary-only guard above preserves agent/device scope. JidKind is
    // wire representation, so compare the validated primary without that kind.
    let peer = &peer.to_non_ad();
    let own = crate::session::AccountRecord::load_in_write(tx)?
        .and_then(|a| a.own_jid)
        .map(|j| j.to_non_ad());
    let own_lid = super::group_send::own_lid_in(tx)?;
    if own.as_ref() == Some(peer) || own_lid.as_ref().map(Jid::to_non_ad).as_ref() == Some(peer) {
        return Ok(());
    }
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if rows.is_empty() {
            break;
        }
        cursor = rows.last().map(|(k, _)| k.clone());
        for (key, value) in rows {
            if !key.starts_with(PREFIX) {
                continue;
            }
            let mut state: DistributionState = serde_json::from_slice(&value).map_err(|_| {
                Error::invalid("group_send", "invalid saved sender-key distribution")
            })?;
            if state.own.to_non_ad() != *peer
                && state.recipients.iter().any(|r| r.to_non_ad() == *peer)
                && !state.rotation_pending
            {
                state.rotation_pending = true;
                tx.put_json(Table::Checkpoints, &key, 1, &state)?;
            }
        }
    }
    Ok(())
}

/// Redacted count for peer-identity inspection; zero does not prove historical coverage.
pub(super) fn pending_for_peer_in(tx: &ReadTx<'_>, peer: &Jid) -> Result<usize> {
    // Match primary encoding variants without folding companions or agents.
    let mut logical_peer = peer.clone();
    logical_peer.kind = crate::model::jid::JidKind::Plain;
    let peer = &logical_peer;
    let mut count = 0;
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if rows.is_empty() {
            break;
        }
        cursor = rows.last().map(|(k, _)| k.clone());
        for (key, value) in rows {
            if key.starts_with(PREFIX) {
                let state: DistributionState = serde_json::from_slice(&value).map_err(|_| {
                    Error::invalid("group_send", "invalid saved sender-key distribution")
                })?;
                if state.rotation_pending && state.recipients.iter().any(|r| r.to_non_ad() == *peer)
                {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([19; 32]))),
            create,
        })
        .unwrap()
    }
    fn jid(raw: &str) -> Jid {
        Jid::parse(raw).unwrap()
    }

    #[test]
    fn acked_distribution_is_not_resent_until_rotation_and_stale_acks_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("acked.redb"), true);
        let own = jid("15555550101@s.whatsapp.net");
        let peer = jid("15555550102:3@s.whatsapp.net");
        let late = jid("15555550103@s.whatsapp.net");
        let group = jid("123456@g.us");
        let load = || store.read(|tx| load_in(tx, &group, &own)).unwrap().unwrap();
        // Reserved but not ACKed: still needs the distribution.
        store
            .write(|tx| {
                reserve_in(tx, &group, &own, None, std::slice::from_ref(&peer))?;
                pending_ack_in(tx, &group, &own, "W1", std::slice::from_ref(&peer))
            })
            .unwrap();
        assert!(!load().delivered(&peer));
        // The ACK marks it; a repeated ACK is a no-op.
        store.write(|tx| confirm_ack_in(tx, "W1")).unwrap();
        assert!(load().delivered(&peer));
        store.write(|tx| confirm_ack_in(tx, "W1")).unwrap();
        // A reservation for a device not yet a recipient adds it unacked.
        let before = load();
        store
            .write(|tx| {
                reserve_in(tx, &group, &own, Some(&before), std::slice::from_ref(&late))?;
                pending_ack_in(tx, &group, &own, "W2", std::slice::from_ref(&late))
            })
            .unwrap();
        assert!(load().delivered(&peer) && !load().delivered(&late));
        // A rotation (identity change) makes nobody delivered...
        store
            .write(|tx| {
                let mut state = load_in_write(tx, &group, &own)?;
                state.rotation_pending = true;
                tx.put_json(Table::Checkpoints, &key(&group, &own), 1, &state)
            })
            .unwrap();
        assert!(!load().delivered(&peer));
        // ...and the next chain is new, so an ACK from the old one marks nothing.
        let pending = load();
        store
            .write(|tx| {
                reserve_in(
                    tx,
                    &group,
                    &own,
                    Some(&pending),
                    std::slice::from_ref(&peer),
                )
            })
            .unwrap();
        store.write(|tx| confirm_ack_in(tx, "W2")).unwrap();
        let fresh = load();
        assert!(!fresh.delivered(&peer) && !fresh.delivered(&late));
        assert_ne!(fresh.chain, before.chain);
    }

    fn load_in_write(tx: &WriteTx<'_>, chat: &Jid, own: &Jid) -> Result<DistributionState> {
        Ok(tx
            .get_json(Table::Checkpoints, &key(chat, own), 1)?
            .expect("state"))
    }

    #[test]
    fn authenticated_archive_invalidates_only_exposed_exact_primary_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("exposure.redb");
        let store = open(&path, true);
        let own = jid("15555550101@s.whatsapp.net");
        let peer = jid("15555550102@s.whatsapp.net");
        let alias = jid("123456@lid");
        let group = jid("123456@g.us");
        let other = jid("456789@g.us");
        store
            .write(|tx| {
                reserve_in(tx, &group, &own, None, &[peer.with_device(3)])?;
                reserve_in(tx, &other, &own, None, std::slice::from_ref(&alias))?;
                tx.put_json(Table::SenderKeys, b"incoming-history", 1, &vec![1u8, 2, 3])?;
                tx.put_json(
                    Table::Checkpoints,
                    b"message:uncertain:send",
                    1,
                    &"outcome_unknown",
                )
            })
            .unwrap();
        // First-seen/equal identities and companions do not trigger this primary path.
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_replacement(tx, &peer, None, None, &[2])?;
                super::super::identity_conflicts::archive_replacement(
                    tx,
                    &peer,
                    Some(&[2]),
                    None,
                    &[2],
                )?;
                after_primary_change_in(tx, &peer.with_device(3))?;
                let mut agent = peer.clone();
                agent.agent = 1;
                after_primary_change_in(tx, &agent)
            })
            .unwrap();
        assert!(
            !store
                .read(|tx| load_in(tx, &group, &own))
                .unwrap()
                .unwrap()
                .rotation_pending()
        );
        let mut encoded_primary = peer.clone();
        encoded_primary.kind = crate::model::jid::JidKind::AgentDevice;
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_replacement(
                    tx,
                    &encoded_primary,
                    Some(&[1]),
                    None,
                    &[2],
                )
            })
            .unwrap();
        drop(store);
        let store = open(&path, false);
        for kind in [
            crate::model::jid::JidKind::Plain,
            crate::model::jid::JidKind::AgentDevice,
            crate::model::jid::JidKind::Interop,
            crate::model::jid::JidKind::Facebook,
        ] {
            let mut encoded = peer.clone();
            encoded.kind = kind;
            assert_eq!(
                store.read(|tx| pending_for_peer_in(tx, &encoded)).unwrap(),
                1
            );
        }
        assert_eq!(
            store
                .read(|tx| pending_for_peer_in(tx, &peer.with_device(3)))
                .unwrap(),
            0
        );
        assert!(
            !store
                .read(|tx| load_in(tx, &other, &own))
                .unwrap()
                .unwrap()
                .rotation_pending()
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::SenderKeys, b"incoming-history", 1))
                .unwrap(),
            Some(vec![1, 2, 3])
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<String>(Table::Checkpoints, b"message:uncertain:send", 1))
                .unwrap(),
            Some("outcome_unknown".into())
        );
        // No exposure metadata is manufactured for historical records.
        assert!(
            store
                .read(|tx| load_in(tx, &jid("999999@g.us"), &own))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn authenticated_lid_change_with_alternate_encoding_queues_rotation_and_refresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lid-primary-kind.redb");
        let store = open(&path, true);
        let own = jid("15555550101@s.whatsapp.net");
        let peer = jid("123456@lid");
        let group = jid("123456@g.us");
        store
            .write(|tx| reserve_in(tx, &group, &own, None, &[peer.with_device(3)]))
            .unwrap();
        let mut encoded = peer.clone();
        encoded.kind = crate::model::jid::JidKind::AgentDevice;
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_bundle_replacement(
                    tx,
                    &encoded,
                    Some(&[1]),
                    None,
                    &[2],
                )
            })
            .unwrap();
        drop(store);
        let store = open(&path, false);
        assert!(
            store
                .read(|tx| load_in(tx, &group, &own))
                .unwrap()
                .unwrap()
                .rotation_pending()
        );
        assert_eq!(
            super::super::devices::pending_users(&store).unwrap(),
            vec![peer.clone()]
        );
        assert_eq!(
            store.read(|tx| pending_for_peer_in(tx, &encoded)).unwrap(),
            1
        );
        assert_eq!(
            store
                .read(|tx| pending_for_peer_in(tx, &jid("123456@s.whatsapp.net")))
                .unwrap(),
            0
        );
    }

    #[test]
    fn login_own_lid_does_not_invalidate_exposure_before_group_checkpoint_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("login-own-rotation.redb");
        let store = open(&path, true);
        let own = jid("111111111111111@lid");
        let foreign = jid("222222222222222@lid");
        let group = jid("123456@g.us");
        let sender = jid("15555550102@s.whatsapp.net");
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"lid":own.to_string()}),
                )?;
                reserve_in(tx, &group, &sender, None, &[own.clone(), foreign.clone()])
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        store.write(|tx| after_primary_change_in(tx, &own)).unwrap();
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        store
            .write(|tx| after_primary_change_in(tx, &foreign))
            .unwrap();
        assert_eq!(
            store.read(|tx| pending_for_peer_in(tx, &foreign)).unwrap(),
            1
        );
        drop(store);
        let store = open(&path, false);
        assert_eq!(
            store.read(|tx| pending_for_peer_in(tx, &foreign)).unwrap(),
            1
        );
    }

    #[test]
    fn own_primary_encoding_variants_do_not_invalidate_exposure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("own-primary-kind.redb");
        let store = open(&path, true);
        let own = jid("15555550101@s.whatsapp.net");
        let lid = jid("123456@lid");
        let group = jid("123456@g.us");
        let other_owner = jid("15555550102@s.whatsapp.net");
        store
            .write(|tx| {
                crate::session::AccountRecord {
                    own_jid: Some(own.clone()),
                    ..Default::default()
                }
                .save(tx)?;
                tx.put_json(
                    Table::Checkpoints,
                    super::super::group_send::OWN_LID_KEY,
                    1,
                    &lid,
                )?;
                reserve_in(tx, &group, &other_owner, None, &[own.clone(), lid.clone()])
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        for primary in [&own, &lid] {
            for kind in [
                crate::model::jid::JidKind::Plain,
                crate::model::jid::JidKind::AgentDevice,
                crate::model::jid::JidKind::Interop,
                crate::model::jid::JidKind::Facebook,
            ] {
                let mut encoded = primary.clone();
                encoded.kind = kind;
                store
                    .write(|tx| after_primary_change_in(tx, &encoded))
                    .unwrap();
            }
        }
        drop(store);
        let store = open(&path, false);
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
    }

    #[test]
    fn aborted_identity_transition_rolls_back_rotation_and_stale_reservation_cannot_clear_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("rollback.redb"), true);
        let own = jid("15555550101@s.whatsapp.net");
        let peer = jid("15555550102@s.whatsapp.net");
        let group = jid("123456@g.us");
        store
            .write(|tx| reserve_in(tx, &group, &own, None, std::slice::from_ref(&peer)))
            .unwrap();
        let before = store.read(|tx| load_in(tx, &group, &own)).unwrap();
        let aborted: Result<()> = store.write(|tx| {
            after_primary_change_in(tx, &peer)?;
            Err(Error::invalid("test", "cut before commit"))
        });
        assert!(aborted.is_err());
        assert_eq!(store.read(|tx| load_in(tx, &group, &own)).unwrap(), before);
        store
            .write(|tx| after_primary_change_in(tx, &peer))
            .unwrap();
        assert!(
            store
                .write(|tx| reserve_in(
                    tx,
                    &group,
                    &own,
                    before.as_ref(),
                    std::slice::from_ref(&peer)
                ))
                .is_err()
        );
        let pending = store.read(|tx| load_in(tx, &group, &own)).unwrap();
        assert!(pending.as_ref().unwrap().rotation_pending());
        store
            .write(|tx| {
                reserve_in(
                    tx,
                    &group,
                    &own,
                    pending.as_ref(),
                    &[jid("15555550103@s.whatsapp.net")],
                )
            })
            .unwrap();
        assert_eq!(store.read(|tx| pending_for_peer_in(tx, &peer)).unwrap(), 0);
        store
            .write(|tx| after_primary_change_in(tx, &peer))
            .unwrap();
        assert!(
            !store
                .read(|tx| load_in(tx, &group, &own))
                .unwrap()
                .unwrap()
                .rotation_pending()
        );
    }

    #[tokio::test]
    async fn pending_rotation_mints_an_independent_chain_without_destroying_old_record() {
        use crate::signal::group::{
            encrypt_group_payload, prepare_own_sender_key, process_skdm, receive_group,
            sender_key_db_key,
        };
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("crypto.redb"), true);
        let own = jid("15555550101@s.whatsapp.net");
        let peer = jid("15555550102@s.whatsapp.net");
        let group = jid("123456@g.us");
        let old = prepare_own_sender_key(&group, &own, None).await.unwrap();
        let db = sender_key_db_key(&group, &own);
        store
            .write(|tx| {
                reserve_in(tx, &group, &own, None, std::slice::from_ref(&peer))?;
                tx.put_json(Table::SenderKeys, &db, 1, &*old.updated_key)
            })
            .unwrap();
        store
            .write(|tx| after_primary_change_in(tx, &peer))
            .unwrap();
        let state = store.read(|tx| load_in(tx, &group, &own)).unwrap().unwrap();
        assert!(state.rotation_pending());
        let fresh = prepare_own_sender_key(&group, &own, None).await.unwrap();
        assert_ne!(*fresh.updated_key, *old.updated_key);
        let skdm =
            crate::signal::group::extract_skdm_from_payload(&fresh.distribution_message).unwrap();
        let (_, bytes) = skdm;
        let installed = process_skdm(&group, &own, db.clone(), None, &bytes)
            .await
            .unwrap();
        let (cipher, _) = encrypt_group_payload(&group, &own, &fresh.updated_key, b"new chain")
            .await
            .unwrap();
        assert!(
            receive_group(&group, &own, Some(old.updated_key.as_slice()), &cipher)
                .await
                .is_err()
        );
        assert!(
            receive_group(
                &group,
                &own,
                Some(installed.updated_key.as_slice()),
                &cipher
            )
            .await
            .unwrap()
            .is_some()
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::SenderKeys, &db, 1))
                .unwrap(),
            Some(old.updated_key.to_vec())
        );
    }
}
