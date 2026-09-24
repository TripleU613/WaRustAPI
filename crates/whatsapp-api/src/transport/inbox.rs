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
    /// Preserved, and it could not be decrypted at all — a broken or missing
    /// session, not merely content this client does not render.
    ///
    /// Distinguished from [`Self::Unsupported`] because the two want opposite
    /// answers: unsupported content has arrived and should be acknowledged,
    /// while an undecryptable one should make us ask the sender to send it
    /// again. Conflating them is why several of the operator's messages simply
    /// vanished with no request to resend.
    Undecryptable,
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

/// Sender-key routing: `from` is the chat and `participant` is the authenticated
/// sender device. Groups (`@g.us`) and broadcasts (`status@broadcast` and
/// broadcast lists) both address messages this way and both encrypt them with
/// the Signal sender-key protocol, keyed by the chat JID — comparison
/// 2.26.37.71 `LX/Cnr` builds its sender-key name from the chat JID's raw
/// string, so the two namespaces cannot collide. Returns `None` for direct
/// messages and malformed stanzas.
pub fn sender_key_chat(node: &Node) -> Option<(Jid, Jid)> {
    let parse = |name| {
        let value = node.get_attr(name)?;
        value
            .as_jid()
            .cloned()
            .or_else(|| Jid::parse(value.as_text()?).ok())
    };
    let chat = parse("from")?;
    if !matches!(chat.server, Server::Group | Server::Broadcast) {
        return None;
    }
    let participant = parse("participant")?;
    matches!(participant.server, Server::User | Server::Lid).then_some((chat, participant))
}

/// Groups only. Separate from [`sender_key_chat`] because a group and a
/// broadcast take *different* receipt shapes: a group's carries neither `type`
/// nor `class`, while a status broadcast carries `type="sender"`,
/// `class="status"` and `sts="65535"` (2.26.37.71 `LX/Cp2->A06`). Widening this
/// would send a group-shaped receipt for a status update.
pub fn group_sender(node: &Node) -> Option<(Jid, Jid)> {
    let (chat, participant) = sender_key_chat(node)?;
    chat.is_group().then_some((chat, participant))
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
    // One message we cannot open must not end the session. The group path has
    // degraded like this since the first live group stanza; the direct path
    // still propagated, so a single undecryptable `pkmsg` tore down the whole
    // connection and every stanza queued behind it went unprocessed.
    //
    // Only the Signal layer's own failures are degraded, by their `field`. A
    // transactional failure *after* a successful decryption also surfaces as
    // `InvalidInput`, and that one must keep propagating: it means the advanced
    // session and the inbox write rolled back together, which the caller has to
    // see rather than have reported as an ordinary undecryptable message.
    match process_direct(store, account, node, capture).await {
        Err(Error::InvalidInput { field, reason }) if field == "direct_message" => {
            tracing::warn!(
                field,
                reason,
                "message preserved but not decrypted; session continues"
            );
            Ok(IncomingResult::Undecryptable)
        }
        other => other,
    }
}

async fn process_direct(
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
        return process_group(store, account, node, capture).await;
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
            // A `msg` with no session: the sender is encrypting against a
            // ratchet we do not have, which only a resend can fix.
            return Ok(IncomingResult::Undecryptable);
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
    // If the direct message carries a SenderKeyDistributionMessage (field 2), process
    // it now so group messages from this sender can decrypt after this commit.
    #[cfg(feature = "experimental-signal-crypto")]
    let skdm_update = {
        use crate::signal::group as g;
        if let Some((group_id, axolotl)) = g::extract_skdm_from_payload(&received.plaintext) {
            if let Ok(group_jid) = Jid::parse(&group_id) {
                if group_jid.is_group() {
                    let skdm_keys =
                        store.read(|tx| g::load_sender_key_bytes(tx, &group_jid, &sender));
                    match skdm_keys {
                        Ok((db_key, prev)) => {
                            g::process_skdm(&group_jid, &sender, db_key, prev, &axolotl)
                                .await
                                .ok()
                        }
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
        // Commit the SKDM sender key update atomically with the direct message. A
        // concurrent writer's newer key wins without failing this message; storage
        // errors still propagate. Say so when the update is dropped, so a later
        // group message that cannot decrypt has a recorded reason.
        #[cfg(feature = "experimental-signal-crypto")]
        if let Some(ref skdm) = skdm_update
            && !crate::signal::group::commit_skdm(tx, skdm)?
        {
            tracing::warn!(
                "sender key distribution not stored: another writer changed the key first"
            );
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

/// The participant's direct-session identity. The sender-key distribution for a
/// group arrives inside an ordinary Signal session with that member, keyed the
/// same way as a direct chat (`comparison-direct:<phone>`). A LID participant is
/// mapped through the `authenticated-lid:` checkpoint the direct path writes; an
/// unmapped LID has no pinned session, so its SKDM child is skipped.
#[cfg(feature = "experimental-signal-crypto")]
fn participant_session_jid(
    store: &ProtocolStore,
    node: &Node,
    participant: &Jid,
) -> Result<Option<Jid>> {
    // An explicit phone mapping on the stanza wins when the server supplies one.
    for attr in ["participant_pn", "sender_pn"] {
        if let Some(phone) = node.get_attr(attr).and_then(|v| {
            v.as_jid()
                .cloned()
                .or_else(|| Jid::parse(v.as_text()?).ok())
        }) && phone.server == Server::User
        {
            return Ok(Some(phone.to_non_ad()));
        }
    }
    match participant.server {
        Server::User => Ok(Some(participant.to_non_ad())),
        Server::Lid => {
            let map_key = format!("authenticated-lid:{}", participant.to_non_ad());
            store.read(|tx| tx.get_json::<Jid>(Table::Checkpoints, map_key.as_bytes(), 1))
        }
        _ => Ok(None),
    }
}

/// A group member's direct Signal session advance, staged for the atomic commit.
/// The SKDM that lets us read a member's group messages arrives inside their
/// ordinary direct session, so reading it moves that session forward.
#[cfg(feature = "experimental-signal-crypto")]
struct MemberSession {
    /// `comparison-direct:<phone>` storage key.
    peer_key: String,
    /// Session and identity as read before decryption, for the OCC check.
    previous_session: Option<Vec<u8>>,
    previous_identity: Option<Vec<u8>>,
    /// Advanced session and the identity to pin.
    session: Zeroizing<Vec<u8>>,
    identity: Vec<u8>,
}

/// Handle incoming group message stanzas (`participant` present, `from` a group).
///
/// A malformed or undecryptable group stanza degrades to `Unsupported` rather
/// than failing the session: the caller propagates errors from `process`, and
/// dropping a live connection over one unreadable stanza would lose the whole
/// receive window. The raw capture is already preserved for offline diagnosis
/// and the stanza stays unacknowledged, so the server may redeliver it.
/// Storage failures still propagate.
#[cfg(feature = "experimental-signal-crypto")]
async fn process_group(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    capture: &[u8],
) -> Result<IncomingResult> {
    match process_group_inner(store, account, node, capture).await {
        Err(Error::InvalidInput { field, reason }) => {
            // A group stanza that fails in the crypto is undecryptable; one
            // that fails elsewhere is merely unsupported. Only the first is
            // worth asking the sender to repeat.
            let undecryptable = matches!(field, "group_crypto" | "group_message");
            tracing::warn!(
                field,
                reason,
                undecryptable,
                "group stanza preserved but not processed; not acknowledged"
            );
            Ok(if undecryptable {
                IncomingResult::Undecryptable
            } else {
                IncomingResult::Unsupported
            })
        }
        other => other,
    }
}

/// A group stanza carries one or two `enc` children: a `pkmsg`/`msg` child whose
/// plaintext holds the sender-key distribution message (field 2), and/or an
/// `skmsg` child holding the actual group message. A member's first message
/// carries both, so the SKDM must be applied before that stanza's `skmsg` is
/// decrypted. Both updates commit in one write transaction.
#[cfg(feature = "experimental-signal-crypto")]
async fn process_group_inner(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    capture: &[u8],
) -> Result<IncomingResult> {
    let Some((group_jid, sender_jid)) = sender_key_chat(node) else {
        return Ok(IncomingResult::Unsupported);
    };
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.len() <= 128)
    else {
        return Ok(IncomingResult::Unsupported);
    };
    // Select the enc children by type; a stanza may legitimately carry both.
    let enc_of = |want: &[&str]| {
        node.child_nodes().iter().find(|n| {
            n.description == "enc"
                && n.get_attr("v").and_then(|v| v.as_text()) == Some("2")
                && n.get_attr("type")
                    .and_then(|v| v.as_text())
                    .is_some_and(|t| want.contains(&t))
        })
    };
    let direct_enc = enc_of(&["pkmsg", "msg"]);
    let sk_enc = enc_of(&["skmsg"]);
    if direct_enc.is_none() && sk_enc.is_none() {
        return Ok(IncomingResult::Unsupported);
    }
    // Inbox key: the group and the server-assigned id identify the stanza.
    let key = format!("received:{group_jid}:{id}");
    if let Some(saved) =
        store.read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))?
    {
        super::inbox_events::restore(store, key.as_bytes(), &saved)?;
        return Ok(if matches!(saved["status"].as_str(), Some("decrypted")) {
            IncomingResult::Duplicate
        } else {
            IncomingResult::Unsupported
        });
    }
    let (db_key, previous_bytes) = store
        .read(|tx| crate::signal::group::load_sender_key_bytes(tx, &group_jid, &sender_jid))?;

    // Step 1: the SKDM-bearing direct child, when present and we can address a
    // session for this member. Advances that member's direct Signal session.
    let mut direct_commit: Option<MemberSession> = None;
    let mut used_prekey = None;
    let mut original_prekey: Option<Zeroizing<Vec<u8>>> = None;
    let mut key_after_skdm: Option<Vec<u8>> = None;
    if let Some(enc) = direct_enc
        && let NodeContent::Bytes(ciphertext) = &enc.content
        && let Some(peer) = participant_session_jid(store, node, &sender_jid)?
    {
        let peer_key = format!("comparison-direct:{peer}");
        let (session, identity): (Option<Vec<u8>>, Option<Vec<u8>>) = store.read(|tx| {
            Ok((
                tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
                tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
            ))
        })?;
        let is_prekey = enc.get_attr("type").and_then(|v| v.as_text()) == Some("pkmsg");
        let decrypted = if is_prekey {
            let signed = crate::registration::factory::registered_signed_prekey(store)?;
            if let Some(pid) = crate::signal::direct::incoming_prekey_id(ciphertext)? {
                original_prekey = store
                    .read(|tx| {
                        tx.get_json::<Vec<u8>>(
                            Table::Prekeys,
                            format!("comparison-prekey:{pid}").as_bytes(),
                            1,
                        )
                    })?
                    .map(Zeroizing::new);
            }
            let incoming = crate::signal::direct::receive_prekey(
                account,
                &peer,
                (session.as_deref(), identity.as_deref()),
                &signed,
                original_prekey.as_deref().map(|v| v.as_slice()),
                ciphertext,
            )
            .await?;
            used_prekey = incoming.consumed_prekey;
            Some((incoming.received, incoming.identity))
        } else if let (Some(s), Some(i)) = (&session, &identity) {
            Some((
                crate::signal::direct::receive(account, &peer, s, i, ciphertext).await?,
                i.clone(),
            ))
        } else {
            // A `msg` child with no established session cannot be read; the
            // skmsg below may still decrypt from a previously stored key.
            tracing::warn!(
                have_session = session.is_some(),
                have_identity = identity.is_some(),
                "group stanza: no session for the member, so its key distribution was skipped"
            );
            None
        };
        if let Some((received, pinned)) = decrypted {
            // Compare the SKDM's group id by identity (user and server), not with
            // `Jid` equality: that derives over the wire-encoding `kind` and the
            // agent/device fields too, so a group parsed from the SKDM's string
            // never equals the same group decoded from the stanza in AD form, and
            // the distribution would be silently discarded.
            let same_group = |g: &Jid| g.user == group_jid.user && g.server == group_jid.server;
            let skdm = crate::signal::group::extract_skdm_from_payload(&received.plaintext);
            if skdm.is_none() {
                // Field numbers only; this never exposes message content.
                tracing::warn!(
                    fields = ?crate::signal::direct::payload_fields(&received.plaintext)
                        .unwrap_or_default(),
                    "group stanza: member payload decrypted but carried no key distribution"
                );
            }
            if let Some((skdm_group, axolotl)) = skdm
                && Jid::parse(&skdm_group).is_ok_and(|g| same_group(&g))
            {
                let applied = crate::signal::group::process_skdm(
                    &group_jid,
                    &sender_jid,
                    db_key.clone(),
                    previous_bytes.clone(),
                    &axolotl,
                )
                .await?;
                key_after_skdm = Some(applied.updated_key.to_vec());
            }
            direct_commit = Some(MemberSession {
                peer_key,
                previous_session: session,
                previous_identity: identity,
                session: received.session.clone(),
                identity: pinned,
            });
        }
    }

    // Step 2: the group message itself, using the key the SKDM above installed
    // when this is the member's first message.
    let effective_key = key_after_skdm.clone().or_else(|| previous_bytes.clone());
    let mut group_plaintext = None;
    let mut final_key = key_after_skdm.clone();
    if let Some(enc) = sk_enc
        && let NodeContent::Bytes(ciphertext) = &enc.content
        && let Some(received) = crate::signal::group::receive_group(
            &group_jid,
            &sender_jid,
            effective_key.as_deref(),
            ciphertext,
        )
        .await?
    {
        final_key = Some(received.updated_key.to_vec());
        group_plaintext = Some(received.plaintext);
    }
    // Nothing usable: no sender key yet and no SKDM in this stanza. Preserve the
    // capture without acknowledging it. Say which inputs were missing — a silent
    // refusal here is indistinguishable from the stanza never arriving.
    let Some(final_key) = final_key else {
        tracing::warn!(
            had_direct_child = direct_enc.is_some(),
            direct_child_decrypted = direct_commit.is_some(),
            skdm_installed = key_after_skdm.is_some(),
            had_skmsg_child = sk_enc.is_some(),
            had_stored_sender_key = previous_bytes.is_some(),
            "group stanza not processed: no sender key available and none distributed in it"
        );
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
        crate::signal::group::commit_sender_key(
            tx,
            &db_key,
            previous_bytes.as_deref(),
            &final_key,
        )?;
        if let Some(member) = &direct_commit {
            let peer_key = member.peer_key.as_bytes();
            let current: Option<Vec<u8>> = tx.get_json(Table::SignalSessions, peer_key, 1)?;
            let current_identity: Option<Vec<u8>> =
                tx.get_json(Table::PeerIdentities, peer_key, 1)?;
            if current.as_deref() != member.previous_session.as_deref()
                || current_identity != member.previous_identity
            {
                return Err(Error::invalid(
                    "inbox",
                    "member session changed; group stanza not committed",
                ));
            }
            tx.put_json(Table::SignalSessions, peer_key, 1, &*member.session)?;
            tx.put_json(Table::PeerIdentities, peer_key, 1, &member.identity)?;
            if let Some(pid) = used_prekey {
                let prekey_key = format!("comparison-prekey:{pid}");
                let current: Option<Vec<u8>> =
                    tx.get_json(Table::Prekeys, prekey_key.as_bytes(), 1)?;
                if current.as_deref() != original_prekey.as_deref().map(|v| v.as_slice()) {
                    return Err(Error::invalid(
                        "inbox",
                        "one-time prekey changed before commit",
                    ));
                }
                tx.delete(Table::Prekeys, prekey_key.as_bytes())?;
            }
        }
        // Only the skmsg plaintext is user content; an SKDM-only stanza records
        // the key exchange without publishing a message event.
        if let Some(plaintext) = &group_plaintext {
            tx.put_json(
                Table::Checkpoints,
                key.as_bytes(),
                1,
                &serde_json::json!({
                    "status": "decrypted",
                    "protobuf": &**plaintext,
                    "capture": capture,
                    "route": route,
                    "received_at": crate::model::time::Timestamp::now(),
                }),
            )?;
            super::inbox_events::publish(tx, key.as_bytes(), node, plaintext)?;
        }
        Ok(IncomingResult::Decrypted)
    })
}

#[cfg(test)]
mod resilience_tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };

    /// A message we cannot open must not end the session. Before this, a single
    /// undecryptable `pkmsg` propagated out of `process` and tore down the whole
    /// connection, so every stanza queued behind it went unprocessed — which
    /// looked like the server hanging up on us.
    #[tokio::test]
    async fn an_undecryptable_message_does_not_end_the_session() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap();
        let account = AccountRecord::default();
        // A direct `msg` from a peer we hold no session for: unopenable.
        let node = Node::new("message")
            .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            .attr("id", "M1")
            .attr("type", "text")
            .children(vec![
                Node::new("enc")
                    .attr("v", "2")
                    .attr("type", "msg")
                    .bytes(vec![9; 48]),
            ]);
        let result = process(&store, &account, &node, b"capture:1").await;
        assert!(
            result.is_ok(),
            "an unopenable message must not propagate an error out of process"
        );
        assert_eq!(result.unwrap(), IncomingResult::Undecryptable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A status update is addressed `from=status@broadcast participant=<poster>`
    /// and is sender-key encrypted exactly like a group message, so it must
    /// route through the same decryption path — but it must NOT be treated as a
    /// group by the receipt builder, whose status shape (`class="status"`,
    /// `type="sender"`) is not implemented yet.
    #[test]
    fn status_routes_for_decryption_but_not_as_a_group() {
        let status = Node::new("message")
            .attr("from", "status@broadcast")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "s1");
        let (chat, sender) = sender_key_chat(&status).expect("status must route for decryption");
        assert_eq!(chat.server, Server::Broadcast);
        assert_eq!(chat.user, "status");
        assert_eq!(sender.user, "15555550100");
        // Not a group: it must never get a *group-shaped* receipt. It does get
        // a receipt, but the status one — `type="sender"`, `class="status"`.
        assert!(group_sender(&status).is_none());
        let receipt = super::super::messaging::received_confirmation(&status)
            .expect("a decrypted status is confirmed, with the status shape");
        assert_eq!(receipt.get_attr("type").unwrap().as_text(), Some("sender"));
        assert_eq!(receipt.get_attr("class").unwrap().as_text(), Some("status"));
        // A broadcast list routes the same way as status.
        let list = Node::new("message")
            .attr("from", "12345@broadcast")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "b1");
        assert!(sender_key_chat(&list).is_some());
        assert!(group_sender(&list).is_none());
        // A broadcast list takes the same status-shaped receipt.
        assert_eq!(
            super::super::messaging::received_confirmation(&list)
                .unwrap()
                .get_attr("class")
                .unwrap()
                .as_text(),
            Some("status")
        );
        // A group still routes as a group and still gets its receipt.
        let group = Node::new("message")
            .attr("from", "120363000000000001@g.us")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "g1");
        assert!(sender_key_chat(&group).is_some());
        assert!(group_sender(&group).is_some());
        assert!(super::super::messaging::received_confirmation(&group).is_some());
    }

    #[test]
    fn group_sender_accepts_group_stanzas_and_refuses_everything_else() {
        let group = |from: &str, participant: Option<&str>| {
            let mut n = Node::new("message").attr("from", from).attr("id", "x");
            if let Some(p) = participant {
                n = n.attr("participant", p);
            }
            group_sender(&n)
        };
        // Well-formed: group `from` plus a user participant.
        let ok = group(
            "120363000000000001@g.us",
            Some("15555550100@s.whatsapp.net"),
        );
        let (g, s) = ok.expect("well-formed group stanza");
        assert!(g.is_group());
        assert_eq!(s.user, "15555550100");
        // A LID participant is accepted (its phone is resolved separately).
        assert!(group("120363000000000001@g.us", Some("12345@lid")).is_some());
        // No participant, non-group from, and a group participant are all refused.
        assert!(group("120363000000000001@g.us", None).is_none());
        assert!(
            group(
                "15555550100@s.whatsapp.net",
                Some("15555550101@s.whatsapp.net")
            )
            .is_none()
        );
        assert!(group("120363000000000001@g.us", Some("120363000000000002@g.us")).is_none());
    }

    /// A group stanza we cannot read must not fail the receive session: the
    /// listen loop propagates errors from `process`, so an undecryptable stanza
    /// would otherwise drop a live connection and lose the whole window.
    #[tokio::test]
    async fn undecryptable_group_stanza_is_unsupported_and_does_not_fail_the_session() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("group.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550199@s.whatsapp.net").unwrap()),
            registration: crate::session::RegistrationStatus::Registered,
            ..Default::default()
        };
        // An skmsg with no stored sender key and garbage ciphertext: nothing to
        // decrypt with, and nothing may be written.
        let node = Node::new("message")
            .attr("from", "120363000000000001@g.us")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "grp1")
            .attr("t", "123")
            .children(vec![
                Node::new("enc")
                    .attr("type", "skmsg")
                    .attr("v", "2")
                    .bytes(vec![9, 9, 9, 9]),
            ]);
        let capture = preserve(&store, &node).unwrap();
        assert_eq!(
            process(&store, &account, &node, &capture).await.unwrap(),
            IncomingResult::Unsupported
        );
        // No sender key, session or inbox row was created.
        assert!(
            store
                .read(|tx| tx.scan(Table::SenderKeys, 1, 10))
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .read(|tx| tx.scan(Table::SignalSessions, 1, 10))
                .unwrap()
                .is_empty()
        );
    }

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
