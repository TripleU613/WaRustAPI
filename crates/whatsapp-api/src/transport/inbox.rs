//! Shared durable incoming-message processing. No network side effects here.
use crate::store::WriteTx;
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::{Jid, Server},
    session::AccountRecord,
    store::{ProtocolStore, Table},
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// Outcome never includes plaintext or sender identifiers in diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingResult {
    /// Authenticated protobuf and ratchet committed together.
    Decrypted,
    /// Already committed, including consumed view-once media.
    Duplicate,
    /// Preserved but unsupported; do not confirm delivery.
    Unsupported,
}

/// Use explicit primary PN routing or the APK's explicit sender_pn LID mapping.
pub fn session_sender(node: &Node) -> Option<Jid> {
    if node.get_attr("participant").is_some() {
        return None;
    }
    let parse = |name| {
        let value = node.get_attr(name)?;
        value
            .as_jid()
            .cloned()
            .or_else(|| Jid::parse(value.as_text()?).ok())
    };
    let sender = parse("from")?;
    match sender.server {
        Server::User if sender.device == 0 => Some(sender),
        Server::Lid if sender.device == 0 => {
            let phone = parse("sender_pn")?;
            (phone.server == Server::User && phone.device == 0).then_some(phone)
        }
        _ => None,
    }
}

/// Comparison 1gz.A00 015a..01ba: peer category, DeviceJid and own UserJid.
/// This selects a route only. `process` additionally requires a previously
/// pinned device identity AND session; it never bootstraps a linked device.
fn own_peer_sender(account: &AccountRecord, node: &Node) -> Option<Jid> {
    if node.get_attr("category").and_then(|v| v.as_text()) != Some("peer")
        || node.get_attr("participant").is_some()
        || node.get_attr("recipient").is_some()
    {
        return None;
    }
    let from = super::receipts::jid(node, "from")?;
    let own = account.own_jid.as_ref()?;
    (from.server == Server::User
        && from.device != 0
        && own.server == Server::User
        && from.to_non_ad() == own.to_non_ad())
    .then_some(from)
}

/// Our own identities as the server reported them in the latest saved login
/// `success` reply (`pn` and `lid` attributes, observed live). `None` until a
/// login capture exists.
pub fn own_identity(store: &ProtocolStore) -> Result<(Option<Jid>, Option<Jid>)> {
    let limits = crate::profile::Limits::default();
    store.read(|tx| {
        let mut cursor = None;
        let mut found: Option<(Option<Jid>, Option<Jid>)> = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                if !key.starts_with(b"inbound:sha256:") {
                    continue;
                }
                let Some(wire): Option<Vec<u8>> = tx.get_json(Table::Checkpoints, &key, 1)? else {
                    continue;
                };
                let Ok(raw) = crate::binary::unpack_frame_payload(&wire, &limits) else {
                    continue;
                };
                let Ok(node) = crate::binary::Decoder::new(
                    &raw,
                    crate::binary::TokenDictionary::comparison_2_26_37_70(),
                    &limits,
                )
                .decode_node() else {
                    continue;
                };
                if node.description != "success" {
                    continue;
                }
                let pn = super::receipts::jid(&node, "pn").map(|j| j.to_non_ad());
                let lid = super::receipts::jid(&node, "lid").map(|j| j.to_non_ad());
                if pn.is_some() || lid.is_some() {
                    found = Some((pn, lid));
                }
            }
            if done {
                return Ok(found.unwrap_or((None, None)));
            }
        }
    })
}

/// Transaction-scoped variant of [`lid_for`].
pub(crate) fn lid_for_in(tx: &WriteTx<'_>, phone: &Jid) -> Result<Option<Jid>> {
    let phone = phone.to_non_ad();
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        let done = rows.len() < 256;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows {
            let Ok(name) = std::str::from_utf8(&key) else {
                continue;
            };
            let Some(lid) = name.strip_prefix("authenticated-lid:") else {
                continue;
            };
            let mapped: Option<Jid> = tx.get_json(Table::Checkpoints, &key, 1)?;
            if mapped.is_some_and(|m| m.to_non_ad() == phone)
                && let Ok(lid) = Jid::parse(lid)
            {
                return Ok(Some(lid.to_non_ad()));
            }
        }
        if done {
            return Ok(None);
        }
    }
}

/// Our own phone-number and LID identities: the `login-identity` checkpoint
/// written at login, else the latest login `success` capture.
pub fn own_identities(store: &ProtocolStore) -> Result<(Option<Jid>, Option<Jid>)> {
    let saved: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, b"login-identity", 1))?;
    if let Some(saved) = saved {
        let parse = |field: &str| saved[field].as_str().and_then(|v| Jid::parse(v).ok());
        return Ok((parse("pn"), parse("lid")));
    }
    own_identity(store)
}

/// Transaction-scoped own identities from the `login-identity` checkpoint
/// (phone-number and LID, non-AD), for attributing quoted-reply participants.
pub(crate) fn own_identities_in(tx: &WriteTx<'_>) -> Result<Vec<Jid>> {
    let saved: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, b"login-identity", 1)?;
    let mut out = Vec::new();
    if let Some(saved) = saved {
        for field in ["pn", "lid"] {
            if let Some(jid) = saved[field].as_str().and_then(|v| Jid::parse(v).ok()) {
                out.push(jid.to_non_ad());
            }
        }
    }
    Ok(out)
}

/// The authenticated LID last seen for a phone-number contact, from incoming
/// messages that carried `from=<lid>` with `sender_pn=<phone>`. `None` when the
/// contact has only ever addressed us by phone number.
pub fn lid_for(store: &ProtocolStore, phone: &Jid) -> Result<Option<Jid>> {
    let phone = phone.to_non_ad();
    store.read(|tx| {
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                let Ok(name) = std::str::from_utf8(&key) else {
                    continue;
                };
                let Some(lid) = name.strip_prefix("authenticated-lid:") else {
                    continue;
                };
                let mapped: Option<Jid> = tx.get_json(Table::Checkpoints, &key, 1)?;
                if mapped.is_some_and(|m| m.to_non_ad() == phone)
                    && let Ok(lid) = Jid::parse(lid)
                {
                    return Ok(Some(lid.to_non_ad()));
                }
            }
            if done {
                return Ok(None);
            }
        }
    })
}

/// Re-run event projection over every authenticated incoming record from its
/// original capture, upgrading opaque placeholders that newer parsers now
/// understand (for example reactions). Returns how many events were published.
/// Offline; never touches sessions, receipts or the network.
pub fn reproject(store: &ProtocolStore) -> Result<usize> {
    let mut published = 0;
    let mut cursor = None;
    loop {
        let rows = store.read(|tx| tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256))?;
        let done = rows.len() < 256;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows {
            if !key.starts_with(b"received:") {
                continue;
            }
            let Some(saved) =
                store.read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, &key, 1))?
            else {
                continue;
            };
            if super::inbox_events::restore(store, &key, &saved)? {
                published += 1;
            }
        }
        if done {
            return Ok(published);
        }
    }
}

/// Persist a raw frame before processing or replying. Deterministic capture keys
/// bound repeated identical redeliveries without losing the original wire data.
pub fn preserve(store: &ProtocolStore, node: &Node) -> Result<Vec<u8>> {
    let wire = crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70())
        .encode(node)?;
    let key = format!("inbound:sha256:{}", hex::encode(Sha256::digest(&wire))).into_bytes();
    store.write(|tx| tx.put_json(Table::Checkpoints, &key, 1, &wire))?;
    Ok(key)
}

/// Authenticate a direct incoming message, atomically advancing its session.
/// Caller must preserve the raw capture first and may confirm delivery only
/// after Decrypted/Duplicate. Unsupported messages must remain unacknowledged.
pub async fn process(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    capture: &[u8],
) -> Result<IncomingResult> {
    if node.description != "message" {
        return Ok(IncomingResult::Unsupported);
    }
    // Group messages carry a `participant` attribute (the sender device JID);
    // `from` is the group JID. Route them to the sender-key path.
    #[cfg(feature = "experimental-signal-crypto")]
    if node.get_attr("participant").is_some() {
        return process_group(store, node, capture).await;
    }
    let peer_category = node.get_attr("category").and_then(|v| v.as_text()) == Some("peer");
    let sender = if peer_category {
        own_peer_sender(account, node)
    } else {
        session_sender(node)
    };
    let Some(sender) = sender else {
        return Ok(IncomingResult::Unsupported);
    };
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.len() <= 128)
    else {
        return Ok(IncomingResult::Unsupported);
    };
    let key = format!("received:{sender}:{id}");
    if let Some(saved) =
        store.read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))?
    {
        super::inbox_events::restore(store, key.as_bytes(), &saved)?;
        return Ok(
            if matches!(saved["status"].as_str(), Some("decrypted" | "consumed")) {
                IncomingResult::Duplicate
            } else {
                IncomingResult::Unsupported
            },
        );
    }
    let encs: Vec<_> = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "enc")
        .collect();
    if encs.len() != 1
        || !matches!(
            encs[0].get_attr("type").and_then(|v| v.as_text()),
            Some("msg" | "pkmsg")
        )
        || encs[0].get_attr("v").and_then(|v| v.as_text()) != Some("2")
    {
        return Ok(IncomingResult::Unsupported);
    }
    let NodeContent::Bytes(ciphertext) = &encs[0].content else {
        return Ok(IncomingResult::Unsupported);
    };
    let peer_key = format!("comparison-direct:{sender}");
    let (session, identity): (Option<Vec<u8>>, Option<Vec<u8>>) = store.read(|tx| {
        Ok((
            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
        ))
    })?;
    let session = session.map(Zeroizing::new);
    if peer_category
        && (session.is_none()
            || identity.is_none()
            || encs[0].get_attr("type").and_then(|v| v.as_text()) != Some("msg"))
    {
        return Ok(IncomingResult::Unsupported);
    }
    let mut used_prekey = None;
    let mut original_prekey = None;
    let (received, pinned_identity) = if encs[0].get_attr("type").and_then(|v| v.as_text())
        == Some("pkmsg")
    {
        let signed = crate::registration::factory::registered_signed_prekey(store)?;
        let prekey_id = crate::signal::direct::incoming_prekey_id(ciphertext)?;
        if let Some(id) = prekey_id {
            original_prekey = store
                .read(|tx| {
                    tx.get_json::<Vec<u8>>(
                        Table::Prekeys,
                        format!("comparison-prekey:{id}").as_bytes(),
                        1,
                    )
                })?
                .map(Zeroizing::new);
        }
        let incoming = crate::signal::direct::receive_prekey(
            account,
            &sender,
            (
                session.as_deref().map(|v| v.as_slice()),
                identity.as_deref(),
            ),
            &signed,
            original_prekey.as_deref().map(|v| v.as_slice()),
            ciphertext,
        )
        .await?;
        used_prekey = incoming.consumed_prekey;
        (incoming.received, incoming.identity)
    } else {
        let (Some(session), Some(identity)) = (&session, &identity) else {
            return Ok(IncomingResult::Unsupported);
        };
        (
            crate::signal::direct::receive(account, &sender, session, identity, ciphertext).await?,
            identity.clone(),
        )
    };
    let route =
        crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70())
            .encode(node)?;
    let key_share = if peer_category {
        crate::signal::app_state::KeyShare::from_peer_message(&received.plaintext)?
    } else {
        None
    };
    // If the direct message carries a SenderKeyDistributionMessage (field 18), process
    // it now so group messages from this sender can decrypt after this commit.
    #[cfg(feature = "experimental-signal-crypto")]
    let skdm_update = {
        use crate::signal::group as g;
        if let Some((group_id, axolotl)) = g::extract_skdm_from_payload(&received.plaintext) {
            if let Ok(group_jid) = Jid::parse(&group_id) {
                if group_jid.is_group() {
                    let skdm_keys = store.read(|tx| g::load_sender_key_bytes(tx, &group_jid, &sender));
                    match skdm_keys {
                        Ok((db_key, prev)) => g::process_skdm(&group_jid, &sender, db_key, prev, &axolotl).await.ok(),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };
    #[cfg(not(feature = "experimental-signal-crypto"))]
    let skdm_update: Option<()> = None;
    store.write(|tx| {
        if peer_category {
            let current_account = AccountRecord::load_in_write(tx)?.ok_or(Error::NotRegistered)?;
            if !current_account.is_registered()
                || current_account.generation != account.generation
                || current_account.signal_registration_id != account.signal_registration_id
                || current_account
                    .signal_identity
                    .as_ref()
                    .map(|key| &key.0.public)
                    != account.signal_identity.as_ref().map(|key| &key.0.public)
                || own_peer_sender(&current_account, node).as_ref() != Some(&sender)
            {
                return Err(Error::invalid(
                    "inbox",
                    "own-device account changed before commit",
                ));
            }
        }
        let current: Option<Vec<u8>> =
            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
        let current_identity: Option<Vec<u8>> =
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        if current.as_deref() != session.as_deref().map(|v| v.as_slice())
            || current_identity != identity
            || tx
                .get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?
                .is_some()
        {
            return Err(Error::invalid(
                "inbox",
                "session or inbox changed; not committed",
            ));
        }
        tx.put_json(
            Table::SignalSessions,
            peer_key.as_bytes(),
            1,
            &*received.session,
        )?;
        tx.put_json(
            Table::PeerIdentities,
            peer_key.as_bytes(),
            1,
            &pinned_identity,
        )?;
        if let Some(id) = used_prekey {
            let prekey_key = format!("comparison-prekey:{id}");
            let current: Option<Vec<u8>> = tx.get_json(Table::Prekeys, prekey_key.as_bytes(), 1)?;
            if current.as_deref() != original_prekey.as_deref().map(|v| v.as_slice()) {
                return Err(Error::invalid(
                    "inbox",
                    "one-time prekey changed before commit",
                ));
            }
            tx.delete(Table::Prekeys, prekey_key.as_bytes())?;
        }
        if let Some(from) = super::receipts::jid(node, "from")
            && from.server == Server::Lid
        {
            let map_key = format!("authenticated-lid:{from}");
            let previous: Option<Jid> = tx.get_json(Table::Checkpoints, map_key.as_bytes(), 1)?;
            if previous.as_ref().is_some_and(|old| old != &sender) {
                return Err(Error::invalid("inbox", "authenticated LID mapping changed"));
            }
            tx.put_json(Table::Checkpoints, map_key.as_bytes(), 1, &sender)?;
        }
        // Commit SKDM sender key update atomically with the direct message (best-effort:
        // a concurrent SKDM write for the same key does not fail this direct message).
        #[cfg(feature = "experimental-signal-crypto")]
        if let Some(ref skdm) = skdm_update {
            let _ = crate::signal::group::commit_skdm(tx, skdm);
        }
        #[cfg(not(feature = "experimental-signal-crypto"))]
        let _ = &skdm_update;
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &serde_json::json!({
            "status":"decrypted", "protobuf":&*received.plaintext, "capture":capture, "route":route,
            "received_at":crate::model::time::Timestamp::now()
        }))?;
        super::inbox_events::publish(tx, key.as_bytes(), node, &received.plaintext)?;
        if let Some(share) = &key_share {
            share.import(tx)?;
        }
        Ok(())
    })?;
    Ok(IncomingResult::Decrypted)
}

/// Handle incoming group message stanzas (`enc type="skmsg"`, `participant` present).
#[cfg(feature = "experimental-signal-crypto")]
async fn process_group(
    store: &ProtocolStore,
    node: &Node,
    capture: &[u8],
) -> Result<IncomingResult> {
    let parse_jid = |name: &str| {
        node.get_attr(name).and_then(|v| {
            v.as_jid()
                .cloned()
                .or_else(|| Jid::parse(v.as_text()?).ok())
        })
    };
    let (Some(group_jid), Some(sender_jid)) = (parse_jid("from"), parse_jid("participant")) else {
        return Ok(IncomingResult::Unsupported);
    };
    if !group_jid.is_group() {
        return Ok(IncomingResult::Unsupported);
    }
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.len() <= 128)
    else {
        return Ok(IncomingResult::Unsupported);
    };
    let encs: Vec<_> = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "enc")
        .collect();
    if encs.len() != 1
        || encs[0].get_attr("type").and_then(|v| v.as_text()) != Some("skmsg")
        || encs[0].get_attr("v").and_then(|v| v.as_text()) != Some("2")
    {
        return Ok(IncomingResult::Unsupported);
    }
    let NodeContent::Bytes(ciphertext) = &encs[0].content else {
        return Ok(IncomingResult::Unsupported);
    };
    // Inbox key: use group JID + message id (participant is the authenticated sender,
    // but the group+id pair uniquely identifies the stanza server-side).
    let key = format!("received:{}:{id}", group_jid);
    if let Some(saved) =
        store.read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))?
    {
        super::inbox_events::restore(store, key.as_bytes(), &saved)?;
        return Ok(
            if matches!(saved["status"].as_str(), Some("decrypted")) {
                IncomingResult::Duplicate
            } else {
                IncomingResult::Unsupported
            },
        );
    }
    let (db_key, previous_bytes) = store
        .read(|tx| crate::signal::group::load_sender_key_bytes(tx, &group_jid, &sender_jid))?;
    let Some(received) = crate::signal::group::receive_group(
        &group_jid,
        &sender_jid,
        db_key,
        previous_bytes,
        ciphertext,
    )
    .await?
    else {
        // No sender key for this sender yet; SKDM has not arrived.
        return Ok(IncomingResult::Unsupported);
    };
    let route =
        crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70())
            .encode(node)?;
    store.write(|tx| {
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?
            .is_some()
        {
            return Ok(IncomingResult::Duplicate);
        }
        crate::signal::group::commit_group(tx, &received)?;
        tx.put_json(
            Table::Checkpoints,
            key.as_bytes(),
            1,
            &serde_json::json!({
                "status": "decrypted",
                "protobuf": &*received.plaintext,
                "capture": capture,
                "route": route,
                "received_at": crate::model::time::Timestamp::now(),
            }),
        )?;
        super::inbox_events::publish(tx, key.as_bytes(), node, &received.plaintext)?;
        Ok(IncomingResult::Decrypted)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn own_device_route_never_bootstraps_unpinned_sessions() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("peer.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
            create: true,
        })
        .unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            registration: crate::session::RegistrationStatus::Registered,
            ..Default::default()
        };
        for kind in ["msg", "pkmsg"] {
            let node = Node::new("message")
                .attr("category", "peer")
                .attr("from", "15555550100:2@s.whatsapp.net")
                .attr("id", kind)
                .children(vec![
                    Node::new("enc")
                        .attr("type", kind)
                        .attr("v", "2")
                        .bytes(vec![1, 2, 3]),
                ]);
            let capture = preserve(&store, &node).unwrap();
            assert_eq!(
                process(&store, &account, &node, &capture).await.unwrap(),
                IncomingResult::Unsupported
            );
        }
        assert!(
            store
                .read(|tx| tx.scan(Table::SignalSessions, 1, 10))
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .read(|tx| tx.scan(Table::PeerIdentities, 1, 10))
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .read(|tx| tx.scan(Table::Inbox, 1, 10))
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn peer_messages_require_exact_own_account_device_route() {
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            ..Default::default()
        };
        let node = Node::new("message").attr("category", "peer");
        assert!(
            own_peer_sender(
                &account,
                &node.clone().attr("from", "15555550100:2@s.whatsapp.net")
            )
            .is_some()
        );
        for sender in [
            "15555550101:2@s.whatsapp.net",
            "15555550100@s.whatsapp.net",
            "12345:2@lid",
            "12345@g.us",
        ] {
            assert!(own_peer_sender(&account, &node.clone().attr("from", sender)).is_none());
        }
        assert!(
            own_peer_sender(
                &account,
                &Node::new("message").attr("from", "15555550100:2@s.whatsapp.net")
            )
            .is_none()
        );
        assert!(
            own_peer_sender(
                &AccountRecord::default(),
                &node.attr("from", "15555550100:2@s.whatsapp.net")
            )
            .is_none()
        );
    }
}
