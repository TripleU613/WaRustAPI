//! Shared durable incoming-message processing. No network side effects here.
use crate::registration::request::IdentityChangePolicy;
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

pub(super) fn remote_identity_policy(
    store: &ProtocolStore,
    account: &AccountRecord,
    peer: &Jid,
    policy: IdentityChangePolicy,
) -> Result<IdentityChangePolicy> {
    // APK 2.26.37.71 0jc.A14 has a separate own-companion safeguard.
    // A remote policy must not replace a session for our own PN/LID.
    if policy == IdentityChangePolicy::Pinned
        || account
            .own_jid
            .as_ref()
            .is_some_and(|own| own.user == peer.user && own.server == peer.server)
    {
        return Ok(IdentityChangePolicy::Pinned);
    }
    let own: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, b"login-identity", 1))?;
    let own_alias = own.as_ref().is_some_and(|saved| {
        ["pn", "lid"].into_iter().any(|field| {
            saved[field]
                .as_str()
                .and_then(|s| Jid::parse(s).ok())
                .is_some_and(|j| j.user == peer.user && j.server == peer.server)
        })
    });
    Ok(if own_alias {
        IdentityChangePolicy::Pinned
    } else {
        policy
    })
}

/// Outcome never includes plaintext or sender identifiers in diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingResult {
    /// Authenticated payload or key control and protocol state committed together.
    Decrypted,
    /// Already committed, including consumed view-once media.
    Duplicate,
    /// Preserved but unsupported; do not confirm delivery.
    Unsupported,
    /// Preserved, and it could not be decrypted at all — a broken or missing
    /// session, not merely content this client does not render.
    ///
    /// Distinguished from [`Self::Unsupported`] because the two need different
    /// recovery: an unsupported envelope stays unacknowledged, while supported
    /// ciphertext without usable keys can request a bounded resend.
    Undecryptable,
}

// Token/string node bodies can be Text; raw binary bodies remain Bytes.
// Treat both opaque byte views identically; structured/empty bodies are not
// supported ciphertext and must never earn a delivery confirmation.
fn ciphertext_bytes(node: &Node) -> Option<&[u8]> {
    let bytes = match &node.content {
        NodeContent::Bytes(bytes) => bytes.as_ref(),
        NodeContent::Text(text) => text.as_bytes(),
        _ => return None,
    };
    (!bytes.is_empty()).then_some(bytes)
}

/// Primary chat identity for direct inbox/event/receipt lookup. Companion
/// session selection separately preserves the actual device and namespace.
pub fn session_sender(node: &Node) -> Option<Jid> {
    // Recipient-qualified synchronization uses a separate outgoing projection.
    if node.get_attr("participant").is_some()
        || node.get_attr("recipient").is_some()
        || node.get_attr("category").and_then(|v| v.as_text()) == Some("peer")
    {
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
        Server::User if sender.agent == 0 => Some(sender.to_non_ad()),
        // Keep companion chat/dedupe identity stable if an optional alias
        // appears or disappears on redelivery. Save PN mapping after auth.
        Server::Lid if sender.device != 0 && sender.agent == 0 => Some(sender.to_non_ad()),
        // APK 37.71 CjN.A04/A00 select the actual ordinary LID route;
        // DEw.CF0 -> DFc carries a nullable phone alias separately.
        Server::Lid if sender.device == 0 && sender.agent == 0 => {
            if node.get_attr("sender_pn").is_some()
                && !parse("sender_pn").is_some_and(|phone| {
                    phone.server == Server::User && phone.device == 0 && phone.agent == 0
                })
            {
                return None;
            }
            Some(sender.to_non_ad())
        }
        _ => None,
    }
}

/// Existing authenticated direct record, retaining its original chat selector.
/// New LID records use the actual namespace; legacy PN rows are never renamed.
pub(super) struct DirectRecord {
    pub chat: Jid,
    pub key: String,
    pub saved: Option<serde_json::Value>,
}

/// Read the canonical row and at most two explicitly attributed legacy PN rows.
/// Capture checks are local deduplication guards, not APK migration behavior.
pub(super) fn direct_record_in(
    node: &Node,
    mut get: impl FnMut(&[u8]) -> Result<Option<serde_json::Value>>,
) -> Result<Option<DirectRecord>> {
    let Some(chat) = session_sender(node) else {
        return Ok(None);
    };
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|id| !id.is_empty() && id.len() <= 128)
    else {
        return Ok(None);
    };
    let key = format!("received:{chat}:{id}");
    let saved = get(key.as_bytes())?;
    let mut selected = DirectRecord { chat, key, saved };
    let Some(from) = super::receipts::jid(node, "from")
        .filter(|from| from.server == Server::Lid && from.device == 0 && from.agent == 0)
    else {
        return Ok(Some(selected));
    };
    let mut phones = Vec::new();
    if let Some(phone) = super::receipts::jid(node, "sender_pn") {
        phones.push(phone.to_non_ad());
    }
    if let Some(value) = get(format!("authenticated-lid:{}", from.to_non_ad()).as_bytes())? {
        let phone: Jid = serde_json::from_value(value)
            .map_err(|_| Error::invalid("inbox", "invalid authenticated LID mapping"))?;
        if phone.server != Server::User || phone.device != 0 || phone.agent != 0 {
            return Err(Error::invalid("inbox", "invalid authenticated LID mapping"));
        }
        if !phones.contains(&phone) {
            phones.push(phone);
        }
    }
    for phone in phones {
        let legacy_key = format!("received:{phone}:{id}");
        let Some(legacy) = get(legacy_key.as_bytes())? else {
            continue;
        };
        // Consumed view-once rows keep their route proof but no plaintext;
        // redelivery must retain their single-use marker across chat layouts.
        if !matches!(legacy["status"].as_str(), Some("decrypted" | "consumed")) {
            continue;
        }
        let wire = if !legacy["route"].is_null() {
            serde_json::from_value::<Vec<u8>>(legacy["route"].clone()).ok()
        } else if let Ok(capture) = serde_json::from_value::<Vec<u8>>(legacy["capture"].clone()) {
            get(&capture)?.and_then(|value| serde_json::from_value(value).ok())
        } else {
            None
        };
        let Some(wire) = wire else {
            return Err(Error::invalid(
                "inbox",
                "legacy direct inbox route unavailable",
            ));
        };
        let limits = crate::profile::Limits::default();
        let raw = crate::binary::unpack_frame_payload(&wire, &limits)?;
        let original = crate::binary::Decoder::new(
            &raw,
            crate::binary::TokenDictionary::comparison_2_26_37_71(),
            &limits,
        )
        .decode_node()?;
        if original.description != "message"
            || original.get_attr("id").and_then(|v| v.as_text()) != Some(id)
            || !super::receipts::jid(&original, "from")
                .is_some_and(|old| super::identity_query::same_primary_address(&old, &from))
            || !super::receipts::jid(&original, "sender_pn")
                .is_some_and(|old| super::identity_query::same_primary_address(&old, &phone))
        {
            continue;
        }
        if selected.saved.is_some() {
            return Err(Error::invalid(
                "inbox",
                "multiple authenticated direct inbox records",
            ));
        }
        selected = DirectRecord {
            chat: phone,
            key: legacy_key,
            saved: Some(legacy),
        };
    }
    Ok(Some(selected))
}

/// Projection of historical records follows the original authenticated alias,
/// never an alias supplied only by a redelivery.
pub(super) fn sender_for_record(key: &[u8], node: &Node) -> Option<Jid> {
    let sender = session_sender(node)?;
    let id = node.get_attr("id")?.as_text()?;
    if sender.server == Server::Lid
        && super::receipts::jid(node, "from")?.device == 0
        && let Some(phone) = super::receipts::jid(node, "sender_pn")
        && phone.server == Server::User
        && phone.device == 0
        && phone.agent == 0
        && format!("received:{phone}:{id}").as_bytes() == key
    {
        return Some(phone);
    }
    Some(sender)
}

/// Share the authenticated alias rules between receive and historical projection.
pub(super) fn authenticated_phone_alias(node: &Node, sender: &Jid) -> Option<Jid> {
    if sender.server == Server::User {
        Some(sender.to_non_ad())
    } else {
        super::receipts::jid(node, "sender_pn")
            .filter(|phone| phone.server == Server::User && phone.device == 0 && phone.agent == 0)
            .map(|phone| phone.to_non_ad())
    }
}

/// Actual non-hosted companion address; never an alias or primary selector.
fn direct_companion_sender(node: &Node) -> Option<Jid> {
    if node.get_attr("participant").is_some() {
        return None;
    }
    super::receipts::jid(node, "from").filter(|peer| {
        peer.device != 0 && peer.agent == 0 && matches!(peer.server, Server::User | Server::Lid)
    })
}

/// The sender's LID when the stanza is LID-addressed (`from=<lid>`, device 0).
fn lid_sender(node: &Node) -> Option<Jid> {
    // DEw.CF0 (APK 37.71) resolves address attributes as typed JIDs; our
    // decoder also preserves textual JIDs. They must choose the same held
    // ratchet (observed LID recovery, 2026-09-28), before normalizing its key.
    super::receipts::jid(node, "from")
        .filter(|j| j.server == Server::Lid && j.device == 0)
        .map(|j| j.to_non_ad())
}

/// Which peer's Signal session decrypts a direct message.
///
/// The session is the one for the address the sender used: a LID-addressed
/// chat used the held LID session (authenticated recovery, 2026-09-28;
/// the old CPQ.A00 citation is not re-anchored for 37.71), and our own sends to
/// that person's LID device (every group fan-out) keyed their session by the
/// LID. Looking a LID sender up by the phone number it maps to found a
/// different ratchet, so every reply they sent over the LID session failed —
/// observed 2026-09-28 on one message redelivered three times over four days,
/// which decrypted as soon as the LID session was used. The phone session
/// stays the fallback for a peer only ever reached by number, and a `pkmsg`
/// from a LID starts its new session under the LID.
fn direct_session_peer(
    phone: &Jid,
    lid: Option<Jid>,
    lid_session_held: bool,
    is_prekey_message: bool,
) -> Jid {
    match lid {
        Some(lid) if lid_session_held || is_prekey_message => lid,
        _ => phone.clone(),
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
/// nor `class`, while status carries `class="status"` without sts;
/// sender type requires own-account context (2.26.37.71 Cp2.A06). Widening this
/// would send a group-shaped receipt for a status update.
pub fn group_sender(node: &Node) -> Option<(Jid, Jid)> {
    let (chat, participant) = sender_key_chat(node)?;
    chat.is_group().then_some((chat, participant))
}

/// Comparison 1gz.A00 015a..01ba: peer category, DeviceJid and own UserJid.
/// This selects a route only. `process` additionally requires a previously
/// pinned device identity AND session; it never bootstraps a linked device.
fn own_peer_sender(account: &AccountRecord, node: &Node, own_lid: Option<&Jid>) -> Option<Jid> {
    if node.get_attr("category").and_then(|v| v.as_text()) != Some("peer")
        || node.get_attr("participant").is_some()
        || node.get_attr("recipient").is_some()
    {
        return None;
    }
    let from = super::receipts::jid(node, "from")?;
    if from.agent != 0 || from.device == 0 || from.user.is_empty() {
        return None;
    }
    let own = match from.server {
        Server::User => account.own_jid.as_ref()?,
        Server::Lid => own_lid?,
        _ => return None,
    };
    (own.server == from.server && from.to_non_ad() == own.to_non_ad()).then_some(from)
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
                    crate::binary::TokenDictionary::comparison_2_26_37_71(),
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
    preserve_observed(store, node, None)
}

/// Persist a network capture and, for terminal stanzas, its local observation.
/// Epochs are actor-local and may repeat after restart; times are wall-clock
/// observations, never server timestamps. Offline preserve does not backfill them.
pub fn preserve_in_epoch(
    store: &ProtocolStore,
    node: &Node,
    epoch: super::ConnectionEpoch,
) -> Result<Vec<u8>> {
    preserve_observed(
        store,
        node,
        Some((epoch, crate::model::time::Timestamp::now())),
    )
}

const LATEST_TERMINAL: &[u8] = b"terminal-latest-capture";
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct StoredTerminalObservation {
    first_observed_at: crate::model::time::Timestamp,
    last_observed_at: crate::model::time::Timestamp,
    last_epoch: super::ConnectionEpoch,
    observations: u64,
}

/// Local terminal-capture provenance; no raw stanza or echoed identifier.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TerminalObservation {
    /// First network observation recorded by this instrumentation.
    pub first_observed_at: crate::model::time::Timestamp,
    /// Most recent network observation; wall clock can move backwards.
    pub last_observed_at: crate::model::time::Timestamp,
    /// Most recently observing actor-local epoch, not globally unique.
    pub last_epoch: super::ConnectionEpoch,
    /// Number of recorded observations, saturating at u64::MAX.
    pub observations: u64,
    /// This capture is the last terminal stanza committed by the instrumentation.
    pub latest_terminal: bool,
}

fn observation_key(capture: &[u8]) -> Option<Vec<u8>> {
    let digest = capture.strip_prefix(b"inbound:sha256:")?;
    if digest.len() != 64 || !digest.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut key = b"terminal-observation:".to_vec();
    key.extend_from_slice(digest);
    Some(key)
}

/// Inspect terminal provenance without creating metadata for old captures.
pub fn terminal_observation(
    store: &ProtocolStore,
    capture: &[u8],
) -> Result<Option<TerminalObservation>> {
    let Some(key) = observation_key(capture) else {
        return Ok(None);
    };
    store.read(|tx| {
        let Some(saved) = tx.get_json::<StoredTerminalObservation>(Table::Checkpoints, &key, 1)?
        else {
            return Ok(None);
        };
        let latest: Option<Vec<u8>> = tx.get_json(Table::Checkpoints, LATEST_TERMINAL, 1)?;
        Ok(Some(TerminalObservation {
            first_observed_at: saved.first_observed_at,
            last_observed_at: saved.last_observed_at,
            last_epoch: saved.last_epoch,
            observations: saved.observations,
            latest_terminal: latest.as_deref() == Some(capture),
        }))
    })
}

fn preserve_observed(
    store: &ProtocolStore,
    node: &Node,
    observed: Option<(super::ConnectionEpoch, crate::model::time::Timestamp)>,
) -> Result<Vec<u8>> {
    let wire = crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
        .encode(node)?;
    let key = format!("inbound:sha256:{}", hex::encode(Sha256::digest(&wire))).into_bytes();
    store.write(|tx| {
        tx.put_json(Table::Checkpoints, &key, 1, &wire)?;
        if matches!(node.description.as_str(), "stream:error" | "failure")
            && let Some((epoch, at)) = observed
        {
            let metadata_key = observation_key(&key).expect("generated SHA-256 capture key");
            let mut saved = tx
                .get_json::<StoredTerminalObservation>(Table::Checkpoints, &metadata_key, 1)?
                .unwrap_or(StoredTerminalObservation {
                    first_observed_at: at,
                    last_observed_at: at,
                    last_epoch: epoch,
                    observations: 0,
                });
            saved.last_observed_at = at;
            saved.last_epoch = epoch;
            saved.observations = saved.observations.saturating_add(1);
            tx.put_json(Table::Checkpoints, &metadata_key, 1, &saved)?;
            tx.put_json(Table::Checkpoints, LATEST_TERMINAL, 1, &key)?;
        }
        Ok(())
    })?;
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
    process_with_policy(store, account, node, capture, IdentityChangePolicy::Pinned).await
}

/// Process using an explicit incoming remote identity policy. Own-device
/// messages always remain pinned; authenticated replacements archive old state
/// and emit an IdentityChanged event in the inbox commit transaction.
pub async fn process_with_policy(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    capture: &[u8],
    policy: IdentityChangePolicy,
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
    match process_direct(store, account, node, capture, policy).await {
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
    policy: IdentityChangePolicy,
) -> Result<IncomingResult> {
    if node.description != "message" {
        return Ok(IncomingResult::Unsupported);
    }
    if node.get_attr("recipient").is_some() {
        return super::own_device::process(store, account, node, capture).await;
    }
    // Group messages carry a `participant` attribute (the sender device JID);
    // `from` is the group JID. Route them to the sender-key path.
    #[cfg(feature = "experimental-signal-crypto")]
    if node.get_attr("participant").is_some() {
        return process_group(store, account, node, capture, policy).await;
    }
    let peer_category = node.get_attr("category").and_then(|v| v.as_text()) == Some("peer");
    let own_peer_lid = if peer_category
        && node.get_attr("participant").is_none()
        && node.get_attr("recipient").is_none()
        && super::receipts::jid(node, "from")
            .is_some_and(|jid| jid.server == Server::Lid && jid.device != 0 && jid.agent == 0)
    {
        super::group_send::own_lid(store)?
    } else {
        None
    };
    let sender = if peer_category {
        own_peer_sender(account, node, own_peer_lid.as_ref())
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
    let (sender, key, saved) = if peer_category {
        let key = format!("received:{sender}:{id}");
        let saved = store.read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?;
        (sender, key, saved)
    } else {
        let Some(record) = store
            .read(|tx| direct_record_in(node, |key| tx.get_json(Table::Checkpoints, key, 1)))?
        else {
            return Ok(IncomingResult::Unsupported);
        };
        (record.chat, record.key, record.saved)
    };
    if let Some(saved) = saved {
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
    let Some(ciphertext) = ciphertext_bytes(encs[0]) else {
        return Ok(IncomingResult::Unsupported);
    };
    // The Signal session is the one for the address the sender used. A
    // LID-addressed chat used the held LID session (authenticated 2026-09-28
    // recovery; the old CPQ.A00 citation is not re-anchored for 37.71), and
    // our own sends to that person's LID device — every group fan-out — keyed
    // their session `comparison-direct:<lid>`. Looking a LID sender up by the
    // phone number it maps to found a different ratchet, so every reply they
    // sent over the LID session failed ("Signal key/session operation failed",
    // observed 2026-09-28 on one message redelivered three times). The phone
    // session stays the fallback for a peer we only ever reached by number.
    // The inbox record itself stays keyed by `sender`.
    let is_prekey_message = encs[0].get_attr("type").and_then(|v| v.as_text()) == Some("pkmsg");
    // Own-device (`category=peer`) prekeys, hybrid or not, are refused below
    // without parsing: that route accepts only `msg` on a held pinned session.
    let hybrid_prekey = !peer_category
        && is_prekey_message
        && crate::signal::direct::incoming_prekey_kem(ciphertext)?.is_some();
    let lid_peer = if peer_category {
        None
    } else {
        lid_sender(node)
    };
    let lid_session_held = match &lid_peer {
        Some(lid) => store
            .read(|tx| {
                tx.get_json::<Vec<u8>>(
                    Table::SignalSessions,
                    format!("comparison-direct:{lid}").as_bytes(),
                    1,
                )
            })?
            .is_some(),
        None => false,
    };
    // APK 37.71 B7H.A06 carries getDevice into B76; companion sessions
    // use the actual from address even when sender_pn names a chat alias.
    let companion_peer = if peer_category {
        None
    } else {
        direct_companion_sender(node)
    };
    if hybrid_prekey && companion_peer.is_some() {
        // Companion ADV admission is classic-only here; checked before it can
        // observe or reserve anything. Local coverage limit, not native refusal.
        tracing::warn!("companion hybrid prekey preserved: companion KEM admission is unported");
        return Ok(IncomingResult::Unsupported);
    }
    let companion_authority = if is_prekey_message && let Some(peer) = &companion_peer {
        match super::companion_admission::prepare(store, account, peer, node, ciphertext)? {
            super::companion_admission::Admission::Ready(authority) => Some(authority),
            super::companion_admission::Admission::Refused(status) => {
                tracing::warn!(
                    status,
                    "direct companion prekey preserved: admission deferred"
                );
                return Ok(IncomingResult::Unsupported);
            }
        }
    } else {
        None
    };
    let session_peer = companion_peer.clone().unwrap_or_else(|| {
        // Chat projection uses the actual LID, independently of the held
        // ratchet. Preserve the explicit PN fallback for ordinary msg when
        // no LID session exists (Signal rollback/crash fixture); pkmsg still
        // starts under the actual LID. No alias is guessed or persisted here.
        let fallback = if !peer_category && sender.server == Server::Lid {
            authenticated_phone_alias(node, &sender).unwrap_or_else(|| sender.clone())
        } else {
            sender.clone()
        };
        direct_session_peer(&fallback, lid_peer, lid_session_held, is_prekey_message)
    });
    let skdm_sender = companion_peer.as_ref().unwrap_or(&sender);
    let peer_key = format!("comparison-direct:{session_peer}");
    let (session, identity): (Option<Vec<u8>>, Option<Vec<u8>>) = store.read(|tx| {
        Ok((
            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
        ))
    })?;
    if is_prekey_message {
        super::identity_conflicts::observe(store, &session_peer, ciphertext, identity.as_deref())?;
    }
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
    let mut used_kem = None;
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
        let previous = (
            session.as_deref().map(|v| v.as_slice()),
            identity.as_deref(),
            if peer_category {
                IdentityChangePolicy::Pinned
            } else {
                remote_identity_policy(store, account, &session_peer, policy)?
            },
        );
        let incoming = if hybrid_prekey {
            let Some(hybrid) = receive_hybrid_prekey(
                store,
                account,
                &session_peer,
                previous,
                &signed,
                original_prekey.as_deref().map(|v| v.as_slice()),
                ciphertext,
            )
            .await?
            else {
                return Ok(IncomingResult::Unsupported);
            };
            used_kem = hybrid.consumed_kem;
            hybrid.incoming
        } else {
            crate::signal::direct::receive_prekey_with_policy(
                account,
                &session_peer,
                previous,
                &signed,
                original_prekey.as_deref().map(|v| v.as_slice()),
                ciphertext,
            )
            .await?
        };
        used_prekey = incoming.consumed_prekey;
        (incoming.received, incoming.identity)
    } else {
        let (Some(session), Some(identity)) = (&session, &identity) else {
            // A `msg` with no session: the sender is encrypting against a
            // ratchet we do not have, which only a resend can fix.
            return Ok(IncomingResult::Undecryptable);
        };
        (
            crate::signal::direct::receive(account, &session_peer, session, identity, ciphertext)
                .await?,
            identity.clone(),
        )
    };
    let route =
        crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
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
                        store.read(|tx| g::load_sender_key_bytes(tx, &group_jid, skdm_sender));
                    match skdm_keys {
                        Ok((db_key, prev)) => {
                            g::process_skdm(&group_jid, skdm_sender, db_key, prev, &axolotl)
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
        if let Some(authority) = &companion_authority {
            authority.commit(tx)?;
        }
        if peer_category {
            let current_account = AccountRecord::load_in_write(tx)?.ok_or(Error::NotRegistered)?;
            let current_own_lid = if sender.server == Server::Lid {
                super::group_send::own_lid_in(tx)?
            } else {
                None
            };
            if !current_account.is_registered()
                || current_account.generation != account.generation
                || current_account.signal_registration_id != account.signal_registration_id
                || current_account
                    .signal_identity
                    .as_ref()
                    .map(|key| &key.0.public)
                    != account.signal_identity.as_ref().map(|key| &key.0.public)
                || own_peer_sender(&current_account, node, current_own_lid.as_ref()).as_ref()
                    != Some(&sender)
            {
                return Err(Error::invalid(
                    "inbox",
                    "own-device account changed before commit",
                ));
            }
        }
        if !peer_category {
            let selected =
                direct_record_in(node, |key| tx.get_json(Table::Checkpoints, key, 1))?
                    .ok_or_else(|| Error::invalid("inbox", "direct route changed before commit"))?;
            if selected.key != key || selected.saved.is_some() {
                return Err(Error::invalid(
                    "inbox",
                    "direct inbox changed before commit",
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
        super::identity_conflicts::archive_replacement(
            tx,
            &session_peer,
            identity.as_deref(),
            session.as_deref().map(|v| v.as_slice()),
            &pinned_identity,
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
        if let Some((id, original)) = &used_kem {
            super::kem_prekeys::consume_in(tx, *id, original)?;
        }
        let phone_alias = authenticated_phone_alias(node, &sender);
        if let Some(from) = super::receipts::jid(node, "from")
            && from.server == Server::Lid
            && let Some(phone) = phone_alias
        {
            let map_key = format!("authenticated-lid:{}", from.to_non_ad());
            let previous: Option<Jid> = tx.get_json(Table::Checkpoints, map_key.as_bytes(), 1)?;
            if previous.as_ref().is_some_and(|old| old != &phone) {
                return Err(Error::invalid("inbox", "authenticated LID mapping changed"));
            }
            tx.put_json(Table::Checkpoints, map_key.as_bytes(), 1, &phone)?;
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
/// same way as a direct chat (`comparison-direct:<selected PN or LID>`). Prefer
/// a held LID session, or establish a mapped LID prekey session. An explicit PN
/// alias supplies contact identity but does not select a different ratchet.
/// An unmapped LID can use its held session; without either it remains skipped.
#[cfg(feature = "experimental-signal-crypto")]
fn participant_session_jid(
    store: &ProtocolStore,
    node: &Node,
    participant: &Jid,
) -> Result<Option<Jid>> {
    // APK 37.71 B7H.A06 carries getDevice into B76; flattening a companion
    // would select another device's ratchet. This selector handles held msg
    // sessions; prekeys use companion_admission separately. 37.71 CoU.A04
    // calls 11T.A0D before
    // CQn.A00 dispatches companion prekeys (peer-identity.md outer trace).
    // Use only a held/pinned non-hosted msg session here, never the primary
    // or an alias fallback.
    if participant.agent != 0 {
        tracing::warn!("group pairwise child preserved: nonzero-agent routing is unsupported");
        return Ok(None);
    }
    if participant.device != 0 {
        if !matches!(participant.server, Server::User | Server::Lid) {
            return Ok(None);
        }
        let direct_kind = node
            .child_nodes()
            .iter()
            .find(|n| {
                n.description == "enc"
                    && n.get_attr("v").and_then(|v| v.as_text()) == Some("2")
                    && matches!(
                        n.get_attr("type").and_then(|v| v.as_text()),
                        Some("msg" | "pkmsg")
                    )
            })
            .and_then(|n| n.get_attr("type").and_then(|v| v.as_text()));
        if direct_kind != Some("msg") {
            tracing::warn!("group companion prekey requires separate ADV admission");
            return Ok(None);
        }
        let key = format!("comparison-direct:{participant}");
        let (session_held, identity_pinned) = store.read(|tx| {
            Ok((
                tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?
                    .is_some(),
                tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1)?
                    .is_some(),
            ))
        })?;
        if !session_held || !identity_pinned {
            tracing::warn!(
                session_held,
                identity_pinned,
                "group companion pairwise child preserved: exact pinned session required"
            );
            return Ok(None);
        }
        return Ok(Some(participant.clone()));
    }
    // Authenticated original-capture recovery (2026-10-07, groups.md) proved
    // that the held LID ratchet decrypts where the PN alias does not. The older
    // CPQ.A00 citation is not established for 37.71; its producer is untraced.
    // participant_pn is a contact alias, not authority to switch ratchets.
    let mut phone = None;
    for attr in ["participant_pn", "sender_pn"] {
        if let Some(value) = super::receipts::jid(node, attr)
            && value.server == Server::User
        {
            phone = Some(value.to_non_ad());
            break;
        }
    }
    if participant.server == Server::User {
        return Ok(phone.or_else(|| Some(participant.to_non_ad())));
    }
    if participant.server != Server::Lid {
        return Ok(None);
    }
    let lid = participant.to_non_ad();
    if phone.is_none() {
        let map_key = format!("authenticated-lid:{lid}");
        phone = store.read(|tx| tx.get_json::<Jid>(Table::Checkpoints, map_key.as_bytes(), 1))?;
    }
    let held = store
        .read(|tx| {
            tx.get_json::<Vec<u8>>(
                Table::SignalSessions,
                format!("comparison-direct:{lid}").as_bytes(),
                1,
            )
        })?
        .is_some();
    let is_prekey = node.child_nodes().iter().any(|n| {
        n.description == "enc"
            && n.get_attr("type").and_then(|v| v.as_text()) == Some("pkmsg")
            && n.get_attr("v").and_then(|v| v.as_text()) == Some("2")
    });
    Ok(match phone {
        Some(phone) => Some(direct_session_peer(&phone, Some(lid), held, is_prekey)),
        None if held => Some(lid),
        None => None,
    })
}

/// Authenticated pairwise hybrid candidate plus the KEM row its commit consumes.
#[cfg(feature = "experimental-signal-crypto")]
struct HybridPrekey {
    incoming: crate::signal::direct::ReceivedPrekey,
    /// Non-last-resort row to delete with the commit, as read before decryption.
    consumed_kem: Option<(u32, super::kem_prekeys::StoredKemPrekey)>,
}

/// Version-4 KEM prekey envelope from a primary PN/LID peer.
///
/// Native 2.26.37.71 `Cnp.A04` promotes a matching current/archived session
/// (version and Alice base key) before any local key lookup; otherwise it loads
/// the signed, optional ordinary and `kyber_prekeys` rows (`0kh.BTS`) and
/// bootstraps (docs/protocol/hybrid-incoming.md). `Ok(None)` preserves the
/// capture unacknowledged for local coverage gaps: no inventory row (this client
/// has no KEM producer), a changed pin on a matching session, or malformed local
/// or KEM input. Nested authentication failures keep the classic
/// `direct_message` classification, and a pinned changed identity is reported as
/// the classic backend's untrusted-identity failure, so both paths retry alike.
#[cfg(feature = "experimental-signal-crypto")]
async fn receive_hybrid_prekey(
    store: &ProtocolStore,
    account: &AccountRecord,
    peer: &Jid,
    previous: (Option<&[u8]>, Option<&[u8]>, IdentityChangePolicy),
    signed: &[u8],
    one_time: Option<&[u8]>,
    ciphertext: &[u8],
) -> Result<Option<HybridPrekey>> {
    let kem_id = crate::signal::direct::incoming_prekey_kem(ciphertext)?
        .ok_or_else(|| Error::invalid("inbox", "hybrid receive requires KEM metadata"))?
        .prekey_id;
    let local = match store.read(|tx| super::kem_prekeys::load(tx, kem_id)) {
        Ok(local) => local,
        Err(Error::InvalidInput {
            field: "kem_prekey_inventory",
            reason,
        }) => {
            tracing::warn!(reason, "hybrid prekey preserved: local KEM row is invalid");
            return Ok(None);
        }
        Err(other) => return Err(other),
    };
    let result = crate::signal::hybrid::receive_prekey(
        account,
        peer,
        previous,
        Some(signed),
        one_time,
        local
            .as_ref()
            .map(super::kem_prekeys::StoredKemPrekey::local),
        ciphertext,
    )
    .await;
    match result {
        Ok(received) => {
            let consumed_kem = match received.consumed_kem {
                Some(id) => Some((
                    id,
                    local
                        .filter(|row| row.id == id && !row.last_resort)
                        .ok_or_else(|| Error::invalid("inbox", "consumed KEM prekey not loaded"))?,
                )),
                None => None,
            };
            Ok(Some(HybridPrekey {
                incoming: received.incoming,
                consumed_kem,
            }))
        }
        Err(Error::IdentityChanged(_)) => Err(Error::invalid(
            "direct_message",
            "Signal peer identity untrusted; saved pin preserved",
        )),
        Err(Error::InvalidInput { field, reason })
            if matches!(
                field,
                "hybrid_session" | "kem_secret_key" | "kem_ciphertext"
            ) =>
        {
            tracing::warn!(
                field,
                reason,
                local_kem_row = local.is_some(),
                "hybrid prekey preserved: not processed"
            );
            Ok(None)
        }
        Err(other) => Err(other),
    }
}

/// A group member's direct Signal session advance, staged for the atomic commit.
/// The SKDM that lets us read a member's group messages arrives inside their
/// ordinary direct session, so reading it moves that session forward.
#[cfg(feature = "experimental-signal-crypto")]
struct MemberSession {
    /// `comparison-direct:<selected PN or LID>` storage key.
    peer_key: String,
    /// Session and identity as read before decryption, for the OCC check.
    previous_session: Option<Vec<u8>>,
    previous_identity: Option<Vec<u8>>,
    /// Advanced session and the identity to pin.
    session: Zeroizing<Vec<u8>>,
    identity: Vec<u8>,
    companion_authority: Option<Box<super::companion_admission::AuthoritySnapshot>>,
}

/// Handle incoming group message stanzas (`participant` present, `from` a group).
///
/// Malformed group stanzas stay Unsupported; supported ciphertext that cannot
/// decrypt is Undecryptable rather than failing the session. The caller propagates
/// errors from `process`, so dropping a live connection over one unreadable stanza would lose the whole
/// receive window. The raw capture is already preserved for offline diagnosis
/// and the stanza stays unacknowledged, so the server may redeliver it.
/// Storage failures still propagate.
#[cfg(feature = "experimental-signal-crypto")]
async fn process_group(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    capture: &[u8],
    policy: IdentityChangePolicy,
) -> Result<IncomingResult> {
    match process_group_inner(store, account, node, capture, policy).await {
        Err(Error::InvalidInput { field, reason }) => {
            // A group stanza that fails in the crypto is undecryptable; one
            // that fails elsewhere is merely unsupported. Only the first is
            // worth asking the sender to repeat.
            // SKDM-bearing group children use the direct Signal decryptor.
            // Its failures need the same bounded retry as sender-key failures.
            let undecryptable =
                matches!(field, "group_crypto" | "group_message" | "direct_message");
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
    policy: IdentityChangePolicy,
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
    let hybrid_prekey = if let Some(enc) = direct_enc
        && enc.get_attr("type").and_then(|v| v.as_text()) == Some("pkmsg")
        && let Some(ciphertext) = ciphertext_bytes(enc)
    {
        crate::signal::direct::incoming_prekey_kem(ciphertext)?.is_some()
    } else {
        false
    };
    if hybrid_prekey && sender_jid.device != 0 {
        // Companion ADV admission is classic-only here. Local coverage limit.
        tracing::warn!(
            "group companion hybrid prekey preserved: companion KEM admission is unported"
        );
        return Ok(IncomingResult::Unsupported);
    }
    let sk_enc = enc_of(&["skmsg"]);
    if direct_enc.is_none() && sk_enc.is_none() {
        return Ok(IncomingResult::Unsupported);
    }
    let selected =
        if group_jid.server == Server::Group {
            Some(store.read(|tx| {
                super::group_inbox::select(node, |k| tx.get(Table::Checkpoints, k, 1))
            })?)
        } else {
            None
        };
    let key = selected
        .as_ref()
        .map(|r| r.key.clone())
        .unwrap_or_else(|| format!("received:{group_jid}:{id}"));
    let saved = match &selected {
        Some(row) => row.saved.clone(),
        None => store
            .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))?,
    };
    if let Some(saved) = saved {
        super::inbox_events::restore(store, key.as_bytes(), &saved)?;
        return Ok(
            if matches!(
                saved["status"].as_str(),
                Some("decrypted" | "sender_key_distributed")
            ) {
                IncomingResult::Duplicate
            } else {
                IncomingResult::Unsupported
            },
        );
    }
    let (db_key, previous_bytes) = store
        .read(|tx| crate::signal::group::load_sender_key_bytes(tx, &group_jid, &sender_jid))?;

    // Step 1: the SKDM-bearing direct child, when present and we can address a
    // session for this member. Advances that member's direct Signal session.
    let mut direct_commit: Option<MemberSession> = None;
    let mut used_prekey = None;
    let mut original_prekey: Option<Zeroizing<Vec<u8>>> = None;
    let mut used_kem = None;
    let mut key_after_skdm: Option<Vec<u8>> = None;
    // A member payload that decrypted but is not a key distribution. In a group
    // stanza with no `skmsg`, that payload *is* the message: the sender
    // addressed it to our device alone. Observed 2026-09-28: the operator's
    // phone sent a `messageHistoryBundle` (Message field 70, `LX/BZc`) this way.
    let mut pairwise_plaintext = None;
    let companion_prekey = sender_jid.device != 0
        && direct_enc
            .is_some_and(|enc| enc.get_attr("type").and_then(|v| v.as_text()) == Some("pkmsg"));
    let companion_authority = if companion_prekey {
        match direct_enc.and_then(ciphertext_bytes) {
            Some(ciphertext) => match super::companion_admission::prepare(
                store,
                account,
                &sender_jid,
                node,
                ciphertext,
            )? {
                super::companion_admission::Admission::Ready(authority) => Some(authority),
                super::companion_admission::Admission::Refused(status) => {
                    tracing::warn!(
                        status,
                        "group companion prekey preserved: admission deferred"
                    );
                    None
                }
            },
            None => None,
        }
    } else {
        None
    };
    let pairwise_peer = if companion_prekey {
        companion_authority.as_ref().map(|_| sender_jid.clone())
    } else {
        participant_session_jid(store, node, &sender_jid)?
    };
    if let Some(enc) = direct_enc
        && let Some(ciphertext) = ciphertext_bytes(enc)
        && let Some(peer) = pairwise_peer
    {
        let peer_key = format!("comparison-direct:{peer}");
        let (session, identity): (Option<Vec<u8>>, Option<Vec<u8>>) = store.read(|tx| {
            Ok((
                tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
                tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
            ))
        })?;
        let is_prekey = enc.get_attr("type").and_then(|v| v.as_text()) == Some("pkmsg");
        if is_prekey {
            super::identity_conflicts::observe(store, &peer, ciphertext, identity.as_deref())?;
        }
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
            let previous = (
                session.as_deref(),
                identity.as_deref(),
                remote_identity_policy(store, account, &peer, policy)?,
            );
            let incoming = if hybrid_prekey {
                let Some(hybrid) = receive_hybrid_prekey(
                    store,
                    account,
                    &peer,
                    previous,
                    &signed,
                    original_prekey.as_deref().map(|v| v.as_slice()),
                    ciphertext,
                )
                .await?
                else {
                    return Ok(IncomingResult::Unsupported);
                };
                used_kem = hybrid.consumed_kem;
                hybrid.incoming
            } else {
                crate::signal::direct::receive_prekey_with_policy(
                    account,
                    &peer,
                    previous,
                    &signed,
                    original_prekey.as_deref().map(|v| v.as_slice()),
                    ciphertext,
                )
                .await?
            };
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
                tracing::info!(
                    fields = ?crate::signal::direct::payload_fields(&received.plaintext)
                        .unwrap_or_default(),
                    "group stanza: member payload is addressed to this device, not a key distribution"
                );
                pairwise_plaintext = Some(received.plaintext.clone());
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
                companion_authority,
            });
        }
    }

    // Step 2: the group message itself, using the key the SKDM above installed
    // when this is the member's first message.
    let effective_key = key_after_skdm.clone().or_else(|| previous_bytes.clone());
    let mut group_plaintext = None;
    let mut final_key = key_after_skdm.clone();
    if let Some(enc) = sk_enc {
        let Some(ciphertext) = ciphertext_bytes(enc) else {
            tracing::warn!("group stanza preserved: skmsg body is not supported ciphertext");
            return Ok(IncomingResult::Unsupported);
        };
        let Some(received) = crate::signal::group::receive_group(
            &group_jid,
            &sender_jid,
            effective_key.as_deref(),
            ciphertext,
        )
        .await?
        else {
            // No sender-key record is a decryption failure, not an unknown
            // message format. The shared durable retry queue already supports
            // group routing; its once-only reservation remains unchanged.
            tracing::warn!(
                "group stanza preserved: sender key missing; decryption requires distribution"
            );
            return Ok(IncomingResult::Undecryptable);
        };
        final_key = Some(received.updated_key.to_vec());
        group_plaintext = Some(received.plaintext);
    }
    // No `skmsg` content but a payload addressed to this device: record that as
    // the message. Before this, such a stanza was left unprocessed and
    // unacknowledged, so the server redelivered it at every login.
    if group_plaintext.is_none() && sk_enc.is_none() {
        group_plaintext = pairwise_plaintext;
    }
    // A previously held key does not authenticate this stanza by itself.
    // Require actual plaintext or a newly applied SKDM before acknowledging;
    // preserve the capture when neither was processed.
    // Say which inputs were missing — a silent refusal here is
    // indistinguishable from the stanza never arriving.
    if key_after_skdm.is_none() && group_plaintext.is_none() {
        tracing::warn!(
            had_direct_child = direct_enc.is_some(),
            direct_child_decrypted = direct_commit.is_some(),
            skdm_installed = key_after_skdm.is_some(),
            had_skmsg_child = sk_enc.is_some(),
            had_stored_sender_key = previous_bytes.is_some(),
            "group stanza not processed: no authenticated payload or new key distribution"
        );
        return Ok(IncomingResult::Unsupported);
    }
    let route =
        crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
            .encode(node)?;
    store.write(|tx| {
        if let Some(selected) = &selected {
            selected.validate_in(tx)?;
        }
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?
            .is_some()
        {
            return Ok(IncomingResult::Duplicate);
        }
        // A payload addressed to this device alone leaves the group key as it
        // was; only a distribution or an `skmsg` advances it.
        if let Some(final_key) = &final_key {
            crate::signal::group::commit_sender_key(
                tx,
                &db_key,
                previous_bytes.as_deref(),
                final_key,
            )?;
        }
        if let Some(member) = &direct_commit {
            if let Some(authority) = &member.companion_authority {
                authority.commit(tx)?;
            }
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
            // The archived peer must be the actual pairwise session selector.
            let peer = Jid::parse(
                member
                    .peer_key
                    .strip_prefix("comparison-direct:")
                    .ok_or_else(|| Error::invalid("inbox", "invalid member session selector"))?,
            )?;
            super::identity_conflicts::archive_replacement(
                tx,
                &peer,
                member.previous_identity.as_deref(),
                member.previous_session.as_deref(),
                &member.identity,
            )?;
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
            if let Some((id, original)) = &used_kem {
                super::kem_prekeys::consume_in(tx, *id, original)?;
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
                    "wire_id": id,
                    "protobuf": &**plaintext,
                    "capture": capture,
                    "route": route,
                    "received_at": crate::model::time::Timestamp::now(),
                }),
            )?;
            super::inbox_events::publish(tx, key.as_bytes(), node, plaintext)?;
        } else {
            // The APK-derived SKDM (Message field 2) installs a key, not user
            // content. A local control marker must share the key/session commit
            // or replaying its pairwise ciphertext falsely requests a retry.
            tx.put_json(
                Table::Checkpoints,
                key.as_bytes(),
                1,
                &serde_json::json!({
                    "status": "sender_key_distributed", "wire_id": id, "capture": capture,
                    "route": route, "received_at": crate::model::time::Timestamp::now(),
                }),
            )?;
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

    #[test]
    fn group_lid_session_selection_preserves_phone_fallback_and_existing_ratchets() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("aliases.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap();
        let phone = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let lid = Jid::parse("123456789@lid").unwrap();
        let node = |kind, mapped| {
            let n = Node::new("message")
                .attr("from", "120363000000000001@g.us")
                .attr("participant", lid.clone())
                .children(vec![
                    Node::new("enc")
                        .attr("type", kind)
                        .attr("v", "2")
                        .bytes(vec![1]),
                ]);
            if mapped {
                n.attr("participant_pn", phone.clone())
            } else {
                n
            }
        };
        assert_eq!(
            participant_session_jid(&store, &node("msg", true), &lid).unwrap(),
            Some(phone.clone())
        );
        assert_eq!(
            participant_session_jid(&store, &node("pkmsg", true), &lid).unwrap(),
            Some(lid.clone())
        );
        assert_eq!(
            participant_session_jid(&store, &node("msg", false), &lid).unwrap(),
            None
        );
        let key = format!("comparison-direct:{lid}");
        store
            .write(|tx| tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &vec![1u8; 20]))
            .unwrap();
        for mapped in [false, true] {
            assert_eq!(
                participant_session_jid(&store, &node("msg", mapped), &lid).unwrap(),
                Some(lid.clone())
            );
        }
        // A malformed mapping cannot convert a group's identity into a peer.
        let bad = node("msg", false).attr("participant_pn", "120363000000000002@g.us");
        assert_eq!(
            participant_session_jid(&store, &bad, &lid).unwrap(),
            Some(lid)
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1))
                .unwrap(),
            Some(vec![1; 20])
        );
    }

    #[test]
    fn companion_group_distribution_never_uses_primary_or_unpinned_session() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("companion.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([30; 32]))),
            create: true,
        })
        .unwrap();
        for raw in ["15555550100:3@s.whatsapp.net", "123456789:3@lid"] {
            let sender = Jid::parse(raw).unwrap();
            let primary = format!("comparison-direct:{}", sender.to_non_ad());
            let companion = format!("comparison-direct:{sender}");
            let node = |kind| {
                Node::new("message")
                    .attr("from", "120363000000000001@g.us")
                    .attr("participant", sender.clone())
                    .attr("participant_pn", "15555550100@s.whatsapp.net")
                    .children(vec![
                        Node::new("enc")
                            .attr("type", kind)
                            .attr("v", "2")
                            .bytes(vec![1]),
                    ])
            };
            store
                .write(|tx| {
                    tx.put_json(Table::SignalSessions, primary.as_bytes(), 1, &vec![1u8; 20])?;
                    tx.put_json(Table::PeerIdentities, primary.as_bytes(), 1, &vec![2u8; 33])
                })
                .unwrap();
            assert_eq!(
                participant_session_jid(&store, &node("msg"), &sender).unwrap(),
                None
            );
            store
                .write(|tx| {
                    tx.put_json(
                        Table::SignalSessions,
                        companion.as_bytes(),
                        1,
                        &vec![3u8; 20],
                    )
                })
                .unwrap();
            assert_eq!(
                participant_session_jid(&store, &node("msg"), &sender).unwrap(),
                None
            );
            store
                .write(|tx| {
                    tx.put_json(
                        Table::PeerIdentities,
                        companion.as_bytes(),
                        1,
                        &vec![4u8; 33],
                    )
                })
                .unwrap();
            assert_eq!(
                participant_session_jid(&store, &node("msg"), &sender).unwrap(),
                Some(sender.clone())
            );
            assert_eq!(
                participant_session_jid(&store, &node("pkmsg"), &sender).unwrap(),
                None
            );
            let mut other = sender.clone();
            other.device = 4;
            assert_eq!(
                participant_session_jid(&store, &node("msg"), &other).unwrap(),
                None
            );
            let state = store
                .read(|tx| {
                    Ok((
                        tx.scan(Table::SignalSessions, 1, 20)?,
                        tx.scan(Table::PeerIdentities, 1, 20)?,
                    ))
                })
                .unwrap();
            let mut hosted = sender.clone();
            hosted.agent = 1;
            assert_eq!(
                participant_session_jid(&store, &node("msg"), &hosted).unwrap(),
                None
            );
            assert_eq!(
                store
                    .read(|tx| Ok((
                        tx.scan(Table::SignalSessions, 1, 20)?,
                        tx.scan(Table::PeerIdentities, 1, 20)?
                    )))
                    .unwrap(),
                state
            );
        }
    }

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

    #[test]
    fn terminal_network_observations_are_atomic_and_offline_replay_is_not_an_observation() {
        use crate::{
            model::{ids::AccountId, time::Timestamp},
            store::{MasterKey, OpenOptions, StaticKeySource},
            transport::ConnectionEpoch,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("observations.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("terminal-fixture").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [43; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let terminal =
            Node::new("stream:error").children(vec![Node::new("ack").attr("id", "private-id")]);
        let key = preserve(&store, &terminal).unwrap();
        assert!(terminal_observation(&store, &key).unwrap().is_none());
        assert!(
            terminal_observation(&store, b"login-result:unknown")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            preserve_observed(
                &store,
                &terminal,
                Some((ConnectionEpoch(7), Timestamp(2000)))
            )
            .unwrap(),
            key
        );
        let first = terminal_observation(&store, &key).unwrap().unwrap();
        assert_eq!(first.first_observed_at, Timestamp(2000));
        assert_eq!(first.observations, 1);
        assert!(first.latest_terminal);
        // Actor epochs and wall clocks can both go backwards after restart;
        // retain actual observation order without inventing global chronology.
        preserve_observed(
            &store,
            &terminal,
            Some((ConnectionEpoch(1), Timestamp(1000))),
        )
        .unwrap();
        let second = terminal_observation(&store, &key).unwrap().unwrap();
        assert_eq!(second.first_observed_at, Timestamp(2000));
        assert_eq!(second.last_observed_at, Timestamp(1000));
        assert_eq!(second.last_epoch, ConnectionEpoch(1));
        assert_eq!(second.observations, 2);
        preserve(&store, &terminal).unwrap();
        assert_eq!(terminal_observation(&store, &key).unwrap().unwrap(), second);
        let failure = Node::new("failure").attr("appeal_token", "private-token");
        let failure_key = preserve_observed(
            &store,
            &failure,
            Some((ConnectionEpoch(2), Timestamp(3000))),
        )
        .unwrap();
        assert!(
            !terminal_observation(&store, &key)
                .unwrap()
                .unwrap()
                .latest_terminal
        );
        assert!(
            terminal_observation(&store, &failure_key)
                .unwrap()
                .unwrap()
                .latest_terminal
        );
        let content =
            serde_json::to_string(&terminal_observation(&store, &failure_key).unwrap()).unwrap();
        assert!(!content.contains("private-token") && !content.contains("private-id"));
        let ordinary = preserve_observed(
            &store,
            &Node::new("iq"),
            Some((ConnectionEpoch(2), Timestamp(4000))),
        )
        .unwrap();
        assert!(terminal_observation(&store, &ordinary).unwrap().is_none());

        let malformed = Node::new("stream:error").attr("code", "fixture-malformed");
        let wire =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&malformed)
                .unwrap();
        let malformed_key =
            format!("inbound:sha256:{}", hex::encode(Sha256::digest(&wire))).into_bytes();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    &observation_key(&malformed_key).unwrap(),
                    1,
                    &"invalid metadata",
                )
            })
            .unwrap();
        assert!(
            preserve_observed(
                &store,
                &malformed,
                Some((ConnectionEpoch(3), Timestamp(5000)))
            )
            .is_err()
        );
        assert!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::Checkpoints, &malformed_key, 1))
                .unwrap()
                .is_none()
        );
        assert!(
            terminal_observation(&store, &failure_key)
                .unwrap()
                .unwrap()
                .latest_terminal
        );
        drop(store);
        let store = open(false);
        assert!(
            terminal_observation(&store, &failure_key)
                .unwrap()
                .unwrap()
                .latest_terminal
        );
        assert_eq!(
            terminal_observation(&store, &key)
                .unwrap()
                .unwrap()
                .observations,
            2
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::Checkpoints, &key, 1))
                .unwrap()
                .unwrap(),
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&terminal)
                .unwrap()
        );
    }

    /// A LID sender's message opens with the LID session when we hold one,
    /// starts one there for a `pkmsg`, and otherwise falls back to the phone
    /// session.
    #[test]
    fn a_lid_sender_uses_the_lid_session() {
        let phone = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let lid = Jid::parse("99990000111@lid").unwrap();
        let node = Node::new("message")
            .attr("from", lid.clone())
            .attr("sender_pn", phone.clone())
            .attr("id", "M");
        assert_eq!(lid_sender(&node), Some(lid.clone()));
        assert_eq!(
            direct_session_peer(&phone, Some(lid.clone()), true, false),
            lid
        );
        assert_eq!(
            direct_session_peer(&phone, Some(lid.clone()), false, true),
            lid
        );
        assert_eq!(direct_session_peer(&phone, Some(lid), false, false), phone);
        let by_number = Node::new("message")
            .attr("from", phone.clone())
            .attr("id", "M");
        assert_eq!(lid_sender(&by_number), None);
        assert_eq!(direct_session_peer(&phone, None, false, true), phone);
    }

    /// A status update is addressed `from=status@broadcast participant=<poster>`
    /// and is sender-key encrypted exactly like a group message, so it must
    /// route through the same decryption path — but it must NOT be treated as a
    /// group by the receipt builder; a remote status carries class=status
    /// without type, while an own-account status requires sender type.
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
        // a receipt with class=status; this peer status has no sender type.
        assert!(group_sender(&status).is_none());
        let receipt = super::super::messaging::received_confirmation(&status)
            .expect("a decrypted status is confirmed, with the status shape");
        assert!(receipt.get_attr("type").is_none());
        assert_eq!(receipt.get_attr("class").unwrap().as_text(), Some("status"));
        // A broadcast list routes the same way as status.
        let list = Node::new("message")
            .attr("from", "12345@broadcast")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "b1");
        assert!(sender_key_chat(&list).is_some());
        assert!(group_sender(&list).is_none());
        assert!(
            super::super::messaging::received_confirmation(&list)
                .unwrap()
                .get_attr("class")
                .is_none()
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
    async fn missing_group_sender_key_is_undecryptable_and_does_not_fail_the_session() {
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
            IncomingResult::Undecryptable
        );
        let text_body = node.clone().children(vec![
            Node::new("enc")
                .attr("v", "2")
                .attr("type", "skmsg")
                .text("\t\t\t\t"),
        ]);
        let text_capture = preserve(&store, &text_body).unwrap();
        assert_eq!(
            process(&store, &account, &text_body, &text_capture)
                .await
                .unwrap(),
            IncomingResult::Undecryptable
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
        // Pairwise group children use direct Signal sessions. A failure here
        // used to be classified Unsupported, silently suppressing retries.
        let peer_key = "comparison-direct:15555550100@s.whatsapp.net";
        store
            .write(|tx| {
                tx.put_json(
                    Table::SignalSessions,
                    peer_key.as_bytes(),
                    1,
                    &vec![1u8; 32],
                )?;
                tx.put_json(
                    Table::PeerIdentities,
                    peer_key.as_bytes(),
                    1,
                    &vec![5u8; 33],
                )
            })
            .unwrap();
        let pairwise = Node::new("message")
            .attr("from", "120363000000000001@g.us")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "grp-pairwise-bad")
            .children(vec![
                Node::new("enc")
                    .attr("type", "msg")
                    .attr("v", "2")
                    .bytes(vec![9, 9, 9, 9]),
            ]);
        let capture = preserve(&store, &pairwise).unwrap();
        assert_eq!(
            process(&store, &account, &pairwise, &capture)
                .await
                .unwrap(),
            IncomingResult::Undecryptable
        );
    }

    #[tokio::test]
    async fn group_same_wire_id_preserves_each_authenticated_participant_after_reopen() {
        use crate::signal::group::{
            commit_skdm, encrypt_group_payload, extract_skdm_from_payload, prepare_own_sender_key,
            process_skdm, sender_key_db_key,
        };
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collision.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("fixture").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [21; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550199@s.whatsapp.net").unwrap()),
            ..Default::default()
        };
        let mut incoming = Vec::new();
        // The same numeric user in PN and LID namespaces is deliberately not
        // conflated. Trusted generated SKDM fixtures isolate inbox crypto from
        // directory admission, which has its own signed-bundle tests.
        for (sender, text) in [
            ("15555550100:3@s.whatsapp.net", "PN collision fixture"),
            ("15555550100:3@lid", "LID collision fixture"),
        ] {
            let sender = Jid::parse(sender).unwrap();
            let own = prepare_own_sender_key(&chat, &sender, None).await.unwrap();
            let (_, bytes) = extract_skdm_from_payload(&own.distribution_message).unwrap();
            let installed = process_skdm(
                &chat,
                &sender,
                sender_key_db_key(&chat, &sender),
                None,
                &bytes,
            )
            .await
            .unwrap();
            assert!(store.write(|tx| commit_skdm(tx, &installed)).unwrap());
            let plain = super::super::group_send::text_payload(text).unwrap();
            let (cipher, _) = encrypt_group_payload(&chat, &sender, &own.updated_key, &plain)
                .await
                .unwrap();
            let node = Node::new("message")
                .attr("from", chat.clone())
                .attr("participant", sender.clone())
                .attr("id", "SHARED-ID")
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "skmsg")
                        .bytes(cipher.to_vec()),
                ]);
            let capture = preserve(&store, &node).unwrap();
            assert_eq!(
                process(&store, &account, &node, &capture).await.unwrap(),
                IncomingResult::Decrypted
            );
            let key = super::super::group_inbox::key(&chat, &sender, "SHARED-ID").unwrap();
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(saved["wire_id"], "SHARED-ID");
            assert_eq!(
                serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()).unwrap(),
                plain
            );
            let mut target =
                super::super::group_references::tests::reference(&chat, &sender, false);
            target.key.id = crate::model::ids::MessageId::new("SHARED-ID").unwrap();
            let content = crate::model::message::MessageContent::Text {
                body: "reply fixture".into(),
                quoted: Some(target),
                style: Default::default(),
            };
            let (quote, _) = super::super::group_references::payload(
                &store,
                &chat,
                account.own_jid.as_ref().unwrap(),
                &content,
            )
            .unwrap();
            let parent = super::super::quotes::decode(&quote)
                .unwrap()
                .quoted
                .unwrap();
            assert_eq!(super::super::quotes::decode(&parent).unwrap().text, text);
            incoming.push((node, capture));
        }
        let receipt = crate::model::receipt::Receipt {
            chat: crate::model::ids::ChatId::from_jid(&chat).unwrap(),
            message_ids: vec![crate::model::ids::MessageId::new("SHARED-ID").unwrap()],
            from: None,
            kind: crate::model::receipt::ReceiptKind::Read,
            server_timestamp: None,
            code: None,
        };
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(
            matches!(super::super::receipts::queue_read(&store, &receipt), Err(Error::InvalidInput { reason, .. }) if reason == "ambiguous group message id; specify participant")
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        for (node, _) in &incoming {
            assert_eq!(
                super::super::receipts::queue_read_for_incoming(&store, node).unwrap(),
                1
            );
            assert_eq!(
                super::super::receipts::queue_read_for_incoming(&store, node).unwrap(),
                0
            );
        }
        let checkpoints = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let ratchets = store.read(|tx| tx.scan(Table::SenderKeys, 1, 100)).unwrap();
        drop(store);
        let store = open(false);
        for (node, capture) in incoming {
            assert_eq!(
                process(&store, &account, &node, &capture).await.unwrap(),
                IncomingResult::Duplicate
            );
            assert_eq!(
                super::super::receipts::queue_read_for_incoming(&store, &node).unwrap(),
                0
            );
        }
        assert_eq!(
            store.read(|tx| tx.scan(Table::SenderKeys, 1, 100)).unwrap(),
            ratchets
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            checkpoints
        );
    }

    #[tokio::test]
    async fn missing_sender_key_recovers_same_capture_after_distribution_without_resetting_retry() {
        use crate::signal::group::{
            commit_skdm, encrypt_group_payload, extract_skdm_from_payload, prepare_own_sender_key,
            process_skdm, sender_key_db_key,
        };
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("late-key.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([21; 32]))),
            create: true,
        })
        .unwrap();
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let sender = Jid::parse("123456:3@lid").unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550199@s.whatsapp.net").unwrap()),
            registration: crate::session::RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            ..Default::default()
        };
        let mine = prepare_own_sender_key(&group, &sender, None).await.unwrap();
        let payload = super::super::group_send::text_payload("late distribution fixture").unwrap();
        let (cipher, _) = encrypt_group_payload(&group, &sender, &mine.updated_key, &payload)
            .await
            .unwrap();
        let node = Node::new("message")
            .attr("from", group.clone())
            .attr("participant", sender.clone())
            .attr("id", "LATE-KEY")
            .children(vec![
                Node::new("enc")
                    .attr("v", "2")
                    .attr("type", "skmsg")
                    .bytes(cipher.to_vec()),
            ]);
        let capture = preserve(&store, &node).unwrap();
        assert_eq!(
            process(&store, &account, &node, &capture).await.unwrap(),
            IncomingResult::Undecryptable
        );
        assert_eq!(
            super::super::receipts::queue_retry_for_incoming(&store, &account, &node).unwrap(),
            1
        );
        assert_eq!(
            super::super::receipts::reserve_pending(&store)
                .unwrap()
                .len(),
            1
        );
        let receipt = super::super::receipts::outcomes(&store).unwrap();
        // Install the late distribution as trusted test input. This tests the
        // group crypto/commit path, not authentication of a new direct SKDM.
        let (_, bytes) = extract_skdm_from_payload(&mine.distribution_message).unwrap();
        let installed = process_skdm(
            &group,
            &sender,
            sender_key_db_key(&group, &sender),
            None,
            &bytes,
        )
        .await
        .unwrap();
        assert!(store.write(|tx| commit_skdm(tx, &installed)).unwrap());
        assert_eq!(
            process(&store, &account, &node, &capture).await.unwrap(),
            IncomingResult::Decrypted
        );
        let ratchet = store.read(|tx| tx.scan(Table::SenderKeys, 1, 10)).unwrap();
        assert_eq!(
            process(&store, &account, &node, &capture).await.unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            store.read(|tx| tx.scan(Table::SenderKeys, 1, 10)).unwrap(),
            ratchet
        );
        assert_eq!(
            super::super::receipts::queue_retry_for_incoming(&store, &account, &node).unwrap(),
            0
        );
        assert_eq!(super::super::receipts::outcomes(&store).unwrap(), receipt);
        assert!(
            super::super::receipts::reserve_pending(&store)
                .unwrap()
                .is_empty()
        );
        // A stored key cannot turn a malformed skmsg body into Decrypted.
        for enc in [
            Node::new("enc").attr("v", "2").attr("type", "skmsg"),
            Node::new("enc")
                .attr("v", "2")
                .attr("type", "skmsg")
                .children(vec![Node::new("bad")]),
            Node::new("enc").attr("v", "2").attr("type", "msg"),
        ] {
            let bad = node.clone().attr("id", "BAD-BODY").children(vec![enc]);
            let saved = preserve(&store, &bad).unwrap();
            assert_eq!(
                process(&store, &account, &bad, &saved).await.unwrap(),
                IncomingResult::Unsupported
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::SenderKeys, 1, 10)).unwrap(),
                ratchet
            );
        }
    }

    #[tokio::test]
    async fn direct_lid_text_and_typed_routes_share_authenticated_session_and_duplicate_marker() {
        use crate::{
            model::ids::AccountId,
            session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
            signal::direct,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        use wacore_libsignal::protocol::{
            DeviceId, GenericSignedPreKey, IdentityKeyPair, KeyPair, PreKeyBundle,
            SignedPreKeyRecord, Timestamp,
        };
        let alice = IdentityKeyPair::generate(&mut rand::rng());
        let bob = IdentityKeyPair::generate(&mut rand::rng());
        let phone = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let lid = Jid::parse("123456789@lid").unwrap();
        let bob_jid = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let account = |identity: &IdentityKeyPair, jid: Jid, id| AccountRecord {
            own_jid: Some(jid),
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(id),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: identity.public_key().public_key_bytes().try_into().unwrap(),
                private: *identity.private_key().serialize(),
            })),
            ..Default::default()
        };
        let alice_account = account(&alice, phone.clone(), 31);
        let bob_account = account(&bob, bob_jid.clone(), 42);
        let signed = KeyPair::generate(&mut rand::rng());
        let signature = alice
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
            .unwrap();
        let bundle = PreKeyBundle::new(
            31,
            DeviceId::new(0),
            None,
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *alice.identity_key(),
        )
        .unwrap();
        let first = direct::prepare_payload(
            &bob_account,
            &lid,
            Some(&bundle),
            None,
            None,
            b"\x0a\x04init",
        )
        .await
        .unwrap();
        let signed_record = SignedPreKeyRecord::new(
            7.into(),
            Timestamp::from_epoch_millis(0),
            &signed,
            &signature,
        )
        .serialize()
        .unwrap();
        let incoming = direct::receive_prekey(
            &alice_account,
            &bob_jid,
            (None, None),
            &signed_record,
            None,
            &first.ciphertext,
        )
        .await
        .unwrap();
        let reply = direct::prepare_payload(
            &alice_account,
            &bob_jid,
            None,
            Some(&incoming.received.session),
            Some(&incoming.identity),
            b"\x0a\x05reply",
        )
        .await
        .unwrap();
        assert_eq!(reply.kind, "msg");
        let typed = Node::new("message")
            .attr("from", lid.clone())
            .attr("sender_pn", phone.clone())
            .attr("id", "LID-ENCODING")
            .children(vec![
                Node::new("enc")
                    .attr("v", "2")
                    .attr("type", "msg")
                    .bytes(reply.ciphertext.clone()),
            ]);
        let text = typed
            .clone()
            .attr("from", lid.to_string())
            .attr("sender_pn", phone.to_string());
        let bare = Node::new("message")
            .attr("from", lid.clone())
            .attr("id", "LID-ENCODING")
            .children(typed.child_nodes().to_vec());
        let username_only = bare.clone().attr("username", "synthetic_user");
        for (node, duplicate, legacy) in [
            (&text, &typed, false),
            (&typed, &text, false),
            (&bare, &typed, false),
            (&typed, &bare, false),
            (&username_only, &bare, false),
            (&typed, &bare, true),
            (&text, &bare, true),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("encoding.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [29; 32],
                    ))),
                    create,
                })
                .unwrap()
            };
            let store = open(true);
            let selector = format!("comparison-direct:{lid}");
            store
                .write(|tx| {
                    tx.put_json(
                        Table::SignalSessions,
                        selector.as_bytes(),
                        1,
                        &*first.session,
                    )?;
                    tx.put_json(
                        Table::PeerIdentities,
                        selector.as_bytes(),
                        1,
                        &first.identity,
                    )
                })
                .unwrap();
            let capture = preserve(&store, node).unwrap();
            if legacy {
                // Genuine authenticated plaintext/advanced ratchet, installed in
                // the former PN layout only in this isolated historical fixture.
                let received = direct::receive(
                    &bob_account,
                    &lid,
                    &first.session,
                    &first.identity,
                    &reply.ciphertext,
                )
                .await
                .unwrap();
                let key = format!("received:{phone}:LID-ENCODING");
                let route = crate::binary::Encoder::new(
                    crate::binary::TokenDictionary::comparison_2_26_37_71(),
                )
                .encode(node)
                .unwrap();
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::SignalSessions,
                            selector.as_bytes(),
                            1,
                            &*received.session,
                        )?;
                        tx.put_json(
                            Table::Checkpoints,
                            format!("authenticated-lid:{lid}").as_bytes(),
                            1,
                            &phone,
                        )?;
                        tx.put_json(Table::Checkpoints, key.as_bytes(), 1,
                        &serde_json::json!({"status":"decrypted", "protobuf":&*received.plaintext,
                            "capture":capture, "route":route}))?;
                        super::super::inbox_events::publish(
                            tx,
                            key.as_bytes(),
                            node,
                            &received.plaintext,
                        )?;
                        Ok(())
                    })
                    .unwrap();
            }
            assert_eq!(
                process(&store, &bob_account, node, &capture).await.unwrap(),
                if legacy {
                    IncomingResult::Duplicate
                } else {
                    IncomingResult::Decrypted
                }
            );
            let record_key = format!(
                "received:{}:LID-ENCODING",
                if legacy { &phone } else { &lid }
            );
            assert!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        record_key.as_bytes(),
                        1
                    ))
                    .unwrap()
                    .is_some()
            );
            let events = store.read(|tx| tx.scan(Table::Inbox, 1, 10)).unwrap();
            assert_eq!(events.len(), 1);
            let before_rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let sessions = store
                .read(|tx| tx.scan(Table::SignalSessions, 1, 10))
                .unwrap();
            let pins = store
                .read(|tx| tx.scan(Table::PeerIdentities, 1, 10))
                .unwrap();
            assert_eq!(sessions.len(), 1);
            assert!(
                store
                    .read(|tx| tx.get_json::<Vec<u8>>(
                        Table::SignalSessions,
                        selector.as_bytes(),
                        1
                    ))
                    .unwrap()
                    .is_some()
            );
            assert!(
                store
                    .read(|tx| tx.get_json::<Vec<u8>>(
                        Table::SignalSessions,
                        format!("comparison-direct:{phone}").as_bytes(),
                        1
                    ))
                    .unwrap()
                    .is_none()
            );
            drop(store);
            let store = open(false);
            let capture = preserve(&store, duplicate).unwrap();
            assert_eq!(
                process(&store, &bob_account, duplicate, &capture)
                    .await
                    .unwrap(),
                IncomingResult::Duplicate
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::SignalSessions, 1, 10))
                    .unwrap(),
                sessions
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::PeerIdentities, 1, 10))
                    .unwrap(),
                pins
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Inbox, 1, 10)).unwrap(),
                events
            );
            // Capture rows can grow on redelivery; authenticated records stay exact.
            let after_rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert_eq!(
                before_rows
                    .iter()
                    .find(|(key, _)| key == record_key.as_bytes()),
                after_rows
                    .iter()
                    .find(|(key, _)| key == record_key.as_bytes())
            );
            assert_eq!(
                super::super::receipts::queue_read_for_incoming(&store, duplicate).unwrap(),
                1
            );
            assert_eq!(
                super::super::receipts::queue_read_for_incoming(&store, node).unwrap(),
                0
            );
            if legacy {
                let mut consumed = serde_json::from_slice::<serde_json::Value>(
                    &after_rows
                        .iter()
                        .find(|(key, _)| key == record_key.as_bytes())
                        .unwrap()
                        .1,
                )
                .unwrap();
                consumed["status"] = "consumed".into();
                consumed.as_object_mut().unwrap().remove("protobuf");
                store
                    .write(|tx| {
                        tx.put_json(Table::Checkpoints, record_key.as_bytes(), 1, &consumed)
                    })
                    .unwrap();
                assert_eq!(
                    process(&store, &bob_account, duplicate, &capture)
                        .await
                        .unwrap(),
                    IncomingResult::Duplicate
                );
                let saved: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, record_key.as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(saved, consumed);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::SignalSessions, 1, 10))
                        .unwrap(),
                    sessions
                );
                assert_eq!(
                    store.read(|tx| tx.scan(Table::Inbox, 1, 10)).unwrap(),
                    events
                );
                assert_eq!(
                    super::super::receipts::queue_read_for_incoming(&store, duplicate).unwrap(),
                    0
                );
                assert!(
                    store
                        .read(|tx| tx.get(
                            Table::Checkpoints,
                            format!("received:{lid}:LID-ENCODING").as_bytes(),
                            1
                        ))
                        .unwrap()
                        .is_none()
                );
            }
        }
    }

    #[test]
    fn legacy_lid_lookup_requires_original_capture_and_refuses_ambiguous_rows() {
        use std::collections::BTreeMap;
        let lid = Jid::parse("123456789@lid").unwrap();
        let phone = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let node = Node::new("message")
            .attr("from", lid.clone())
            .attr("id", "LEGACY");
        let old = node.clone().attr("sender_pn", phone.clone());
        let wire = |node: &Node| {
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(node)
                .unwrap()
        };
        let legacy_key = format!("received:{phone}:LEGACY").into_bytes();
        let canonical_key = format!("received:{lid}:LEGACY").into_bytes();
        let mapping = format!("authenticated-lid:{lid}").into_bytes();
        let legacy = |node: &Node| serde_json::json!({"status":"decrypted", "route":wire(node)});
        let base = BTreeMap::from([
            (mapping, serde_json::to_value(&phone).unwrap()),
            (legacy_key.clone(), legacy(&old)),
        ]);
        let lookup = |rows: &BTreeMap<Vec<u8>, serde_json::Value>| {
            direct_record_in(&node, |key| Ok(rows.get(key).cloned()))
        };
        let record = lookup(&base).unwrap().unwrap();
        assert_eq!(record.chat, phone);
        assert_eq!(record.key.as_bytes(), legacy_key);
        let mut encoded = phone.clone();
        encoded.kind = crate::model::jid::JidKind::AgentDevice;
        let mut rows = base.clone();
        rows.insert(
            legacy_key.clone(),
            legacy(&old.clone().attr("sender_pn", encoded)),
        );
        assert_eq!(lookup(&rows).unwrap().unwrap().chat, phone);
        for foreign in [
            old.clone().attr("from", "987654321@lid"),
            old.clone().attr("id", "OTHER"),
            old.clone().attr("from", lid.with_device(3)),
            old.clone().attr("sender_pn", "15555550102@s.whatsapp.net"),
        ] {
            let mut rows = base.clone();
            rows.insert(legacy_key.clone(), legacy(&foreign));
            assert_eq!(
                lookup(&rows).unwrap().unwrap().key.as_bytes(),
                canonical_key
            );
        }
        let mut rows = base.clone();
        rows.insert(canonical_key, serde_json::json!({"status":"decrypted"}));
        assert!(
            lookup(&rows)
                .err()
                .unwrap()
                .to_string()
                .contains("multiple authenticated direct inbox records")
        );
        let mut rows = base.clone();
        rows.insert(
            legacy_key.clone(),
            serde_json::json!({"status":"decrypted"}),
        );
        assert!(
            lookup(&rows)
                .err()
                .unwrap()
                .to_string()
                .contains("legacy direct inbox route unavailable")
        );
        let mut rows = base.clone();
        rows.insert(b"capture-fixture".to_vec(), serde_json::json!(wire(&old)));
        rows.insert(
            legacy_key,
            serde_json::json!({"status":"decrypted", "capture":b"capture-fixture".to_vec()}),
        );
        assert_eq!(lookup(&rows).unwrap().unwrap().chat, phone);
    }

    #[tokio::test]
    async fn primary_lid_prekey_without_alias_commits_once_after_authentication() {
        use crate::{
            model::ids::AccountId,
            session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
            signal::direct,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        use wacore_libsignal::protocol::{
            DeviceId, GenericSignedPreKey, IdentityKey, IdentityKeyPair, KeyPair, PreKeyBundle,
            PreKeyRecord, SignedPreKeyRecord,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("primary.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("primary-lid-test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [38; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let mut config: crate::registration::request::LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.registration_id = 42;
        config.signed_pre_key_id = 7;
        crate::registration::factory::install_test_material(&store, &config).unwrap();
        let receiver = store.read(AccountRecord::load).unwrap().unwrap();
        let signed = crate::registration::factory::registered_signed_prekey(&store).unwrap();
        let signed = SignedPreKeyRecord::deserialize(&signed).unwrap();
        let one = KeyPair::generate(&mut rand::rng());
        let mut identity = vec![5];
        identity.extend(receiver.signal_identity.as_ref().unwrap().0.public);
        let bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            Some((11.into(), one.public_key)),
            7.into(),
            signed.public_key().unwrap(),
            signed.signature().unwrap().to_vec(),
            IdentityKey::decode(&identity).unwrap(),
        )
        .unwrap();
        let peer = Jid::parse("123456789@lid").unwrap();
        let key = IdentityKeyPair::generate(&mut rand::rng());
        let sender = AccountRecord {
            own_jid: Some(peer.clone()),
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: key.public_key().public_key_bytes().try_into().unwrap(),
                private: *key.private_key().serialize(),
            })),
            ..Default::default()
        };
        let sent = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            Some(&bundle),
            None,
            None,
            b"\x0a\x05hello",
        )
        .await
        .unwrap();
        let prekey = PreKeyRecord::new(11.into(), &one).serialize().unwrap();
        store
            .write(|tx| tx.put_json(Table::Prekeys, b"comparison-prekey:11", 1, &prekey))
            .unwrap();
        let node = Node::new("message")
            .attr("from", peer.clone())
            .attr("id", "BARE-PREKEY")
            .attr("t", "1")
            .children(vec![
                Node::new("enc")
                    .attr("type", sent.kind)
                    .attr("v", "2")
                    .bytes(sent.ciphertext.clone()),
            ]);
        let mut pq_wire = sent.ciphertext.clone();
        pq_wire.extend_from_slice(&[0x38, 0, 0x42, 0]);
        let mut unported_pq = Vec::new();
        for group in [false, true] {
            let mut pq_node = Node::new("message")
                .attr("from", peer.clone())
                .attr("id", "UNPORTED-PQ")
                .children(vec![
                    Node::new("enc")
                        .attr("type", "pkmsg")
                        .attr("v", "2")
                        .bytes(pq_wire.clone()),
                ]);
            if group {
                pq_node = pq_node
                    .attr("from", "12345@g.us")
                    .attr("participant", peer.clone());
            }
            let capture = preserve(&store, &pq_node).unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        Table::ALL
                            .iter()
                            .map(|table| tx.scan(*table, 1, 200))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            let head = store.journal_head().unwrap();
            for policy in [
                IdentityChangePolicy::Pinned,
                IdentityChangePolicy::ApkAuthenticated,
            ] {
                assert_eq!(
                    process_with_policy(&store, &receiver, &pq_node, &capture, policy)
                        .await
                        .unwrap(),
                    IncomingResult::Unsupported
                );
                assert_eq!(snapshot(), before);
                assert_eq!(store.journal_head().unwrap(), head);
            }
            unported_pq.push((pq_node, capture));
        }
        let qualified = node.clone().attr("recipient", "987654321@lid");
        let qualified_capture = preserve(&store, &qualified).unwrap();
        for policy in [
            IdentityChangePolicy::Pinned,
            IdentityChangePolicy::ApkAuthenticated,
        ] {
            assert_eq!(
                process_with_policy(&store, &receiver, &qualified, &qualified_capture, policy)
                    .await
                    .unwrap(),
                IncomingResult::Unsupported
            );
        }
        store
            .read(|tx| {
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:11", 1)?,
                    Some(prekey.clone())
                );
                assert!(tx.scan(Table::SignalSessions, 1, 10)?.is_empty());
                assert!(tx.scan(Table::PeerIdentities, 1, 10)?.is_empty());
                assert!(
                    tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        format!("received:{peer}:BARE-PREKEY").as_bytes(),
                        1
                    )?
                    .is_none()
                );
                assert!(
                    tx.get_json::<Vec<u8>>(Table::Checkpoints, &qualified_capture, 1)?
                        .is_some()
                );
                Ok(())
            })
            .unwrap();
        let mut bad = sent.ciphertext.clone();
        *bad.last_mut().unwrap() ^= 1;
        let damaged = Node::new("message")
            .attr("from", peer.clone())
            .attr("id", "BARE-PREKEY")
            .children(vec![
                Node::new("enc")
                    .attr("type", sent.kind)
                    .attr("v", "2")
                    .bytes(bad),
            ]);
        let capture = preserve(&store, &damaged).unwrap();
        assert_eq!(
            process(&store, &receiver, &damaged, &capture)
                .await
                .unwrap(),
            IncomingResult::Undecryptable
        );
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
                .read(|tx| tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:11", 1))
                .unwrap()
                .is_some()
        );
        let capture = preserve(&store, &node).unwrap();
        assert_eq!(
            process(&store, &receiver, &node, &capture).await.unwrap(),
            IncomingResult::Decrypted
        );
        assert!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:11", 1))
                .unwrap()
                .is_none()
        );
        let sessions = store
            .read(|tx| tx.scan(Table::SignalSessions, 1, 10))
            .unwrap();
        let pins = store
            .read(|tx| tx.scan(Table::PeerIdentities, 1, 10))
            .unwrap();
        let events = store.read(|tx| tx.scan(Table::Inbox, 1, 10)).unwrap();
        assert_eq!(events.len(), 1);
        assert!(super::super::messaging::received_confirmation(&node).is_some());
        assert_eq!(
            super::super::receipts::queue_read_for_incoming(&store, &node).unwrap(),
            1
        );
        drop(store);
        let store = open(false);
        for (pq_node, capture) in &unported_pq {
            let snapshot = || {
                store
                    .read(|tx| {
                        Table::ALL
                            .iter()
                            .map(|table| tx.scan(*table, 1, 200))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            let head = store.journal_head().unwrap();
            assert_eq!(
                process_with_policy(
                    &store,
                    &receiver,
                    pq_node,
                    capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                IncomingResult::Unsupported
            );
            assert_eq!(snapshot(), before);
            assert_eq!(store.journal_head().unwrap(), head);
        }
        let redelivery = node.clone().attr("sender_pn", "15555550101@s.whatsapp.net");
        let capture = preserve(&store, &redelivery).unwrap();
        assert_eq!(
            process(&store, &receiver, &redelivery, &capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::SignalSessions, 1, 10))
                .unwrap(),
            sessions
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::PeerIdentities, 1, 10))
                .unwrap(),
            pins
        );
        assert_eq!(
            store.read(|tx| tx.scan(Table::Inbox, 1, 10)).unwrap(),
            events
        );
        assert_eq!(
            super::super::receipts::queue_read_for_incoming(&store, &redelivery).unwrap(),
            0
        );
        assert!(
            store
                .read(|tx| tx.get_json::<Jid>(
                    Table::Checkpoints,
                    format!("authenticated-lid:{peer}").as_bytes(),
                    1
                ))
                .unwrap()
                .is_none()
        );
        let mut peer_revoke_protocol =
            prost::Message::encode_to_vec(&super::super::chat_settings::ProtocolMessage {
                key: Some(super::super::reactions::Key {
                    id: Some("BARE-PREKEY".into()),
                    from_me: Some(false),
                    ..Default::default()
                }),
                type_: Some(0),
                ..Default::default()
            });
        peer_revoke_protocol.extend_from_slice(b"\x10\x63");
        let mut peer_revoke_body = Vec::new();
        prost::encoding::bytes::encode(12, &peer_revoke_protocol, &mut peer_revoke_body);
        let peer_revocation = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&sent.session),
            Some(&sent.identity),
            &peer_revoke_body,
        )
        .await
        .unwrap();
        let peer_revoke_node = Node::new("message")
            .attr("from", peer.clone())
            .attr("id", "PEER-REVOKE")
            .attr("t", "259201")
            .attr("type", "text")
            .attr("edit", "7")
            .children(vec![
                Node::new("enc")
                    .attr("type", peer_revocation.kind)
                    .attr("v", "2")
                    .bytes(peer_revocation.ciphertext.to_vec()),
            ]);
        let peer_revoke_capture = preserve(&store, &peer_revoke_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &peer_revoke_node, &peer_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        let peer_revoke_events = store
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("peer-revoke")?,
                    crate::model::ids::EventCursor::START,
                    100,
                )
            })
            .unwrap();
        let crate::sync::SyncEvent::MessageUpdated(update) =
            &peer_revoke_events.events.last().unwrap().event
        else {
            panic!("peer revoke missing")
        };
        assert_eq!(update.key.id.as_str(), "BARE-PREKEY");
        assert_eq!(update.key.chat.jid(), &peer);
        assert_eq!(
            update.key.direction,
            crate::model::message::Direction::Incoming
        );
        assert!(
            matches!(&update.change, crate::model::message::MessageChange::Revoked {by} if by.to_jid()==peer)
        );
        assert_eq!(
            process(&store, &receiver, &peer_revoke_node, &peer_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        let old_revocation = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&peer_revocation.session),
            Some(&sent.identity),
            &peer_revoke_body,
        )
        .await
        .unwrap();
        let old_revoke_node = Node::new("message")
            .attr("from", peer.clone())
            .attr("id", "PEER-OLD-REVOKE")
            .attr("t", "259202")
            .attr("type", "text")
            .attr("edit", "7")
            .children(vec![
                Node::new("enc")
                    .attr("type", old_revocation.kind)
                    .attr("v", "2")
                    .bytes(old_revocation.ciphertext.to_vec()),
            ]);
        let old_revoke_capture = preserve(&store, &old_revoke_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &old_revoke_node, &old_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        let old_events = store
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("old-revoke")?,
                    crate::model::ids::EventCursor::START,
                    100,
                )
            })
            .unwrap();
        assert!(matches!(
            old_events.events.last().unwrap().event,
            crate::sync::SyncEvent::UnsupportedMessage(_)
        ));
        assert_eq!(
            process(&store, &receiver, &old_revoke_node, &old_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        // Reuse the authenticated ratchet as a fixture for a separately held own device.
        let peer_device = Jid::parse(&format!("{}:2@lid", peer.user)).unwrap();
        let (held_session, held_identity) = store
            .read(|tx| {
                let key = format!("comparison-direct:{peer}");
                Ok((
                    tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?
                        .unwrap(),
                    tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1)?
                        .unwrap(),
                ))
            })
            .unwrap();
        let reply = direct::prepare_payload(
            &receiver,
            &peer,
            None,
            Some(&held_session),
            Some(&held_identity),
            b"\x0a\x05hello",
        )
        .await
        .unwrap();
        let acknowledged = direct::receive(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            &old_revocation.session,
            &sent.identity,
            &reply.ciphertext,
        )
        .await
        .unwrap();
        store
            .write(|tx| {
                let device_key = format!("comparison-direct:{peer_device}");
                tx.put_json(
                    Table::SignalSessions,
                    device_key.as_bytes(),
                    1,
                    &*reply.session,
                )?;
                tx.put_json(
                    Table::PeerIdentities,
                    device_key.as_bytes(),
                    1,
                    &held_identity,
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"lid":peer.to_string()}),
                )
            })
            .unwrap();
        let synced = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&acknowledged.session),
            Some(&sent.identity),
            b"\xfa\x01\x14\x0a\x0967890@lid\x12\x07\x0a\x05hello",
        )
        .await
        .unwrap();
        let sync_node = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "OWN-LID-SYNC")
            .attr("type", "text")
            .children(vec![
                Node::new("enc")
                    .attr("type", synced.kind)
                    .attr("v", "2")
                    .bytes(synced.ciphertext.to_vec()),
            ]);
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store, &receiver, &sync_node
            )
            .unwrap()
            .is_none()
        );
        for from in ["123456789:3@lid", "999999999:2@lid", "123456789@lid"] {
            let unknown = sync_node.clone().attr("from", from);
            let capture = preserve(&store, &unknown).unwrap();
            assert_eq!(
                process(&store, &receiver, &unknown, &capture)
                    .await
                    .unwrap(),
                IncomingResult::Unsupported
            );
        }
        let mut corrupt = synced.ciphertext.to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        let damaged_sync = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "OWN-LID-DAMAGED")
            .attr("type", "text")
            .children(vec![
                Node::new("enc")
                    .attr("type", "msg")
                    .attr("v", "2")
                    .bytes(corrupt),
            ]);
        let damaged_capture = preserve(&store, &damaged_sync).unwrap();
        let before_damaged = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::SignalSessions, 1, 100)?,
                    tx.scan(Table::PeerIdentities, 1, 100)?,
                ))
            })
            .unwrap();
        assert_eq!(
            process(&store, &receiver, &damaged_sync, &damaged_capture)
                .await
                .unwrap(),
            IncomingResult::Unsupported
        );
        assert_eq!(
            store
                .read(|tx| {
                    Ok((
                        tx.scan(Table::Checkpoints, 1, 100)?,
                        tx.scan(Table::SignalSessions, 1, 100)?,
                        tx.scan(Table::PeerIdentities, 1, 100)?,
                    ))
                })
                .unwrap(),
            before_damaged
        );
        let mismatched = sync_node.clone().attr("recipient", "67891@lid");
        let mismatched_capture = preserve(&store, &mismatched).unwrap();
        assert_eq!(
            process(&store, &receiver, &mismatched, &mismatched_capture)
                .await
                .unwrap(),
            IncomingResult::Unsupported
        );
        let sync_capture = preserve(&store, &sync_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &sync_node, &sync_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        assert_eq!(
            process(&store, &receiver, &sync_node, &sync_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        let edited_redelivery = sync_node.clone().attr("edit", "7");
        let edited_redelivery_capture = preserve(&store, &edited_redelivery).unwrap();
        assert_eq!(
            process(
                &store,
                &receiver,
                &edited_redelivery,
                &edited_redelivery_capture
            )
            .await
            .unwrap(),
            IncomingResult::Unsupported
        );
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store,
                &receiver,
                &edited_redelivery
            )
            .unwrap()
            .is_none()
        );
        let expected_confirmation = Node::new("receipt")
            .attr("to", peer_device.clone())
            .attr("id", "OWN-LID-SYNC")
            .attr("type", "sender")
            .attr("recipient", Jid::parse("67890@lid").unwrap());
        assert_eq!(
            super::super::messaging::received_confirmation_for_account(
                &store, &receiver, &sync_node
            )
            .unwrap(),
            Some(expected_confirmation.clone())
        );
        let batch = store
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("own-sync")?,
                    crate::model::ids::EventCursor::START,
                    100,
                )
            })
            .unwrap();
        let crate::sync::SyncEvent::NewMessage(message) = &batch.events.last().unwrap().event
        else {
            panic!("outgoing text event missing")
        };
        assert_eq!(
            message.key.direction,
            crate::model::message::Direction::Outgoing
        );
        assert_eq!(message.key.chat.jid(), &Jid::parse("67890@lid").unwrap());
        assert!(message.key.sender.is_none());
        assert!(
            matches!(&message.content, crate::model::message::MessageContent::Text { body, .. } if body == "hello")
        );
        store
            .read(|tx| {
                assert!(tx.scan(Table::Outbox, 1, 10)?.is_empty());
                let saved = tx
                    .get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"received-own:67890@lid:OWN-LID-SYNC",
                        1,
                    )?
                    .unwrap();
                assert_eq!(
                    saved["protobuf"],
                    serde_json::json!(
                        b"\xfa\x01\x14\x0a\x0967890@lid\x12\x07\x0a\x05hello".to_vec()
                    )
                );
                Ok(())
            })
            .unwrap();
        #[cfg(feature = "experimental-registration-https")]
        let continued_session = {
            let mut image = Vec::new();
            prost::encoding::string::encode(
                1,
                &"https://mmg.whatsapp.net/fixture".to_owned(),
                &mut image,
            );
            prost::encoding::string::encode(2, &"image/png".to_owned(), &mut image);
            prost::encoding::bytes::encode(4, &vec![1; 32], &mut image);
            prost::encoding::uint64::encode(5, &42, &mut image);
            prost::encoding::bytes::encode(8, &vec![2; 32], &mut image);
            prost::encoding::bytes::encode(9, &vec![3; 32], &mut image);
            let mut inner = Vec::new();
            prost::encoding::bytes::encode(3, &image, &mut inner);
            let mut sent_wrapper = Vec::new();
            prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_wrapper);
            prost::encoding::bytes::encode(2, &inner, &mut sent_wrapper);
            let mut wrapper = Vec::new();
            prost::encoding::bytes::encode(31, &sent_wrapper, &mut wrapper);
            let media = direct::prepare_payload(
                &sender,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&synced.session),
                Some(&sent.identity),
                &wrapper,
            )
            .await
            .unwrap();
            let media_node = Node::new("message")
                .attr("from", peer_device.clone())
                .attr("recipient", "67890@lid")
                .attr("id", "OWN-MEDIA")
                .attr("type", "media")
                .children(vec![
                    Node::new("enc")
                        .attr("type", media.kind)
                        .attr("v", "2")
                        .bytes(media.ciphertext.to_vec()),
                ]);
            let media_capture = preserve(&store, &media_node).unwrap();
            assert_eq!(
                process(&store, &receiver, &media_node, &media_capture)
                    .await
                    .unwrap(),
                IncomingResult::Decrypted
            );
            assert_eq!(
                process(&store, &receiver, &media_node, &media_capture)
                    .await
                    .unwrap(),
                IncomingResult::Duplicate
            );
            assert_eq!(
                super::super::messaging::received_confirmation_for_account(
                    &store,
                    &receiver,
                    &media_node
                )
                .unwrap(),
                Some(
                    Node::new("receipt")
                        .attr("to", peer_device.clone())
                        .attr("id", "OWN-MEDIA")
                        .attr("type", "sender")
                        .attr("recipient", Jid::parse("67890@lid").unwrap())
                )
            );
            let entries = super::super::media::list(&store).unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].kind, super::super::media::MediaKind::Image);
            assert!(
                store
                    .read(|tx| tx.scan(Table::Outbox, 1, 10))
                    .unwrap()
                    .is_empty()
            );
            let before_parent = store
                .read(|tx| {
                    tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"received-own:67890@lid:OWN-MEDIA",
                        1,
                    )
                })
                .unwrap();
            prost::encoding::string::encode(3, &"edited caption".to_owned(), &mut image);
            let mut replacement = Vec::new();
            prost::encoding::bytes::encode(3, &image, &mut replacement);
            let mut protocol = b"\x0a\x0b\x1a\x09OWN-MEDIA\x10\x0e".to_vec();
            prost::encoding::bytes::encode(14, &replacement, &mut protocol);
            prost::encoding::int64::encode(15, &1, &mut protocol);
            let mut body = Vec::new();
            prost::encoding::bytes::encode(12, &protocol, &mut body);
            let mut sent_edit = Vec::new();
            prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_edit);
            prost::encoding::bytes::encode(2, &body, &mut sent_edit);
            let mut wrapper = Vec::new();
            prost::encoding::bytes::encode(31, &sent_edit, &mut wrapper);
            let edited = direct::prepare_payload(
                &sender,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&media.session),
                Some(&sent.identity),
                &wrapper,
            )
            .await
            .unwrap();
            let edit_node = Node::new("message")
                .attr("from", peer_device.clone())
                .attr("recipient", "67890@lid")
                .attr("id", "OWN-IMAGE-EDIT")
                .attr("type", "media")
                .attr("edit", "1")
                .children(vec![
                    Node::new("enc")
                        .attr("type", edited.kind)
                        .attr("v", "2")
                        .bytes(edited.ciphertext.to_vec()),
                ]);
            let edit_capture = preserve(&store, &edit_node).unwrap();
            assert_eq!(
                process(&store, &receiver, &edit_node, &edit_capture)
                    .await
                    .unwrap(),
                IncomingResult::Decrypted
            );
            assert_eq!(
                process(&store, &receiver, &edit_node, &edit_capture)
                    .await
                    .unwrap(),
                IncomingResult::Duplicate
            );
            let events = store
                .read(|tx| {
                    tx.read_events(
                        &crate::store::ConsumerId::new("image-edit")?,
                        crate::model::ids::EventCursor::START,
                        100,
                    )
                })
                .unwrap();
            let crate::sync::SyncEvent::MessageUpdated(update) =
                &events.events.last().unwrap().event
            else {
                panic!("caption edit missing")
            };
            let crate::model::message::MessageChange::Edited(content) = &update.change else {
                panic!("not edited")
            };
            let crate::model::message::MessageContent::Media {
                reference, caption, ..
            } = &**content
            else {
                panic!("not image")
            };
            assert_eq!(caption.as_deref(), Some("edited caption"));
            assert_eq!(
                super::super::media::reference_selector(&store, reference).unwrap(),
                entries[0].id
            );
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"received-own:67890@lid:OWN-MEDIA",
                        1
                    ))
                    .unwrap(),
                before_parent
            );
            assert!(
                super::super::messaging::received_confirmation_for_account(
                    &store, &receiver, &edit_node
                )
                .unwrap()
                .is_some()
            );
            let mut invalid_protocol =
                b"\x0a\x0b\x1a\x09OWN-MEDIA\x10\x0e\x72\x02\x1a\x00".to_vec();
            prost::encoding::int64::encode(15, &2, &mut invalid_protocol);
            let mut body = Vec::new();
            prost::encoding::bytes::encode(12, &invalid_protocol, &mut body);
            let mut sent_empty = Vec::new();
            prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_empty);
            prost::encoding::bytes::encode(2, &body, &mut sent_empty);
            let mut wrapper = Vec::new();
            prost::encoding::bytes::encode(31, &sent_empty, &mut wrapper);
            let invalid = direct::prepare_payload(
                &sender,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&edited.session),
                Some(&sent.identity),
                &wrapper,
            )
            .await
            .unwrap();
            let empty_node = Node::new("message")
                .attr("from", peer_device.clone())
                .attr("recipient", "67890@lid")
                .attr("id", "OWN-CAPTION-EMPTY")
                .attr("type", "media")
                .attr("edit", "1")
                .children(vec![
                    Node::new("enc")
                        .attr("type", invalid.kind)
                        .attr("v", "2")
                        .bytes(invalid.ciphertext.to_vec()),
                ]);
            let empty_capture = preserve(&store, &empty_node).unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        Table::ALL
                            .iter()
                            .map(|t| tx.scan(*t, 1, 200))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            let head = store.journal_head().unwrap();
            assert_eq!(
                process(&store, &receiver, &empty_node, &empty_capture)
                    .await
                    .unwrap(),
                IncomingResult::Unsupported
            );
            assert_eq!(snapshot(), before);
            assert_eq!(store.journal_head().unwrap(), head);
            assert!(
                super::super::messaging::received_confirmation_for_account(
                    &store,
                    &receiver,
                    &empty_node
                )
                .unwrap()
                .is_none()
            );
            invalid.session
        };
        #[cfg(not(feature = "experimental-registration-https"))]
        let continued_session = synced.session.clone();
        let outbox =
            crate::outbox::Outbox::new("own-echo-fixture", crate::profile::Limits::default());
        let request = crate::model::send::SendRequest {
            client_request_id: crate::model::ids::ClientRequestId::new("OWN-ECHO-REQUEST").unwrap(),
            chat: crate::model::ids::ChatId::from_jid(&Jid::parse("67890@lid").unwrap()).unwrap(),
            content: crate::model::message::MessageContent::Text {
                body: "original intent".into(),
                quoted: None,
                style: Default::default(),
            },
        };
        let (ticket, _) = store.write(|tx| outbox.enqueue(tx, &request)).unwrap();
        store
            .write(|tx| {
                let mut entry = outbox.get_mut(tx, ticket.local_id)?.unwrap();
                entry.wire_id = Some(crate::model::ids::MessageId::new("OWN-ECHO")?);
                entry.progress = crate::model::send::AttemptProgress::InFlight;
                entry.facts.push(crate::model::send::DeliveryFact {
                    kind: crate::model::send::DeliveryFactKind::DeliveryUnknown,
                    device: None,
                    observed_at: crate::model::time::Timestamp(123),
                    server_timestamp: None,
                    code: None,
                });
                tx.put_json(Table::Outbox, &ticket.local_id.0, 1, &entry)
            })
            .unwrap();
        let before_echo = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Outbox, 1, 10)?,
                    tx.scan(Table::Idempotency, 1, 10)?,
                ))
            })
            .unwrap();
        let before_echo_head = store.journal_head().unwrap();
        let echo = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&continued_session),
            Some(&sent.identity),
            b"\xfa\x01\x14\x0a\x0967890@lid\x12\x07\x0a\x05hello",
        )
        .await
        .unwrap();
        let echo_node = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "OWN-ECHO")
            .attr("type", "text")
            .children(vec![
                Node::new("enc")
                    .attr("type", echo.kind)
                    .attr("v", "2")
                    .bytes(echo.ciphertext.to_vec()),
            ]);
        let echo_capture = preserve(&store, &echo_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &echo_node, &echo_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        assert_eq!(store.journal_head().unwrap(), before_echo_head);
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan(Table::Outbox, 1, 10)?,
                    tx.scan(Table::Idempotency, 1, 10)?
                )))
                .unwrap(),
            before_echo
        );
        let marker: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Inbox, b"received-own:67890@lid:OWN-ECHO", 1))
            .unwrap()
            .unwrap();
        assert_eq!(marker["kind"], "outgoing_echo");
        assert_eq!(marker["local_id"], serde_json::json!(ticket.local_id));
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store, &receiver, &echo_node
            )
            .unwrap()
            .is_some()
        );
        // Native Cla.A01 moves Message context outside the one-level wrapper.
        let options = vec!["a".to_owned(), "b".to_owned()];
        let creation = super::super::polls::Envelope {
            context: None,
            update: None,
            creation_v3: Some(super::super::polls::PollCreation {
                name: Some("Question".into()),
                options: options
                    .iter()
                    .map(|name| super::super::polls::PollOption {
                        option_name: Some(name.clone()),
                    })
                    .collect(),
                selectable_options_count: Some(1),
            }),
        };
        let mut sent_poll = Vec::new();
        prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_poll);
        prost::encoding::bytes::encode(
            2,
            &prost::Message::encode_to_vec(&creation),
            &mut sent_poll,
        );
        let mut wrapped_poll = Vec::new();
        prost::encoding::bytes::encode(31, &sent_poll, &mut wrapped_poll);
        let context = super::super::polls::ContextInfo {
            message_secret: Some(vec![7; 32]),
        };
        prost::encoding::bytes::encode(
            35,
            &prost::Message::encode_to_vec(&context),
            &mut wrapped_poll,
        );
        let synchronized_poll = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&echo.session),
            Some(&sent.identity),
            &wrapped_poll,
        )
        .await
        .unwrap();
        let poll_node = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "OWN-POLL")
            .attr("t", "1")
            .attr("type", "poll")
            .children(vec![
                Node::new("meta").attr("polltype", "creation"),
                Node::new("enc")
                    .attr("type", synchronized_poll.kind)
                    .attr("v", "2")
                    .bytes(synchronized_poll.ciphertext.to_vec()),
            ]);
        let poll_capture = preserve(&store, &poll_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &poll_node, &poll_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        let poll_batch = store
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("own-poll")?,
                    crate::model::ids::EventCursor::START,
                    100,
                )
            })
            .unwrap();
        let crate::sync::SyncEvent::NewMessage(message) = &poll_batch.events.last().unwrap().event
        else {
            panic!("poll event missing")
        };
        assert_eq!(
            message.key.direction,
            crate::model::message::Direction::Outgoing
        );
        assert_eq!(message.key.chat.jid(), &Jid::parse("67890@lid").unwrap());
        assert!(message.key.sender.is_none());
        assert!(
            matches!(&message.content, crate::model::message::MessageContent::Poll { name, options: actual, selectable_count } if name == "Question" && actual == &options && *selectable_count == 1)
        );
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"received-own:67890@lid:OWN-POLL", 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["protobuf"], serde_json::json!(wrapped_poll));
        let plain =
            super::super::own_device::payload(&wrapped_poll, &Jid::parse("67890@lid").unwrap())
                .unwrap();
        assert_eq!(
            super::super::polls::decode_poll(&plain).unwrap().secret,
            Some([7; 32])
        );
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store, &receiver, &poll_node
            )
            .unwrap()
            .is_some()
        );
        let poll_id = crate::model::ids::MessageId::new("OWN-POLL").unwrap();
        let chat = Jid::parse("67890@lid").unwrap();
        assert_eq!(
            super::super::polls::own_chat_binding(&store, &poll_id, &chat).unwrap(),
            "matched"
        );
        let target = crate::model::message::MessageRef {
            key: crate::model::message::MessageKey {
                chat: crate::model::ids::ChatId::from_jid(&chat).unwrap(),
                sender: None,
                direction: crate::model::message::Direction::Incoming,
                id: poll_id.clone(),
            },
        };
        let vote = super::super::polls::encrypt_vote(
            &[7; 32],
            "OWN-POLL",
            &peer_device.to_non_ad(),
            &chat,
            &["a".into()],
            [3; 12],
        )
        .unwrap();
        let vote_payload =
            super::super::polls::vote_payload(&peer_device.to_non_ad(), &target, vote, 5);
        let vote_node = Node::new("message")
            .attr("from", chat.clone())
            .attr("id", "PEER-POLL-VOTE");
        let before_lookup = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let context = store
            .write(|tx| super::super::inbox_events::poll_context(tx, &vote_node, &vote_payload))
            .unwrap()
            .expect("synchronized poll secret missing");
        let decoded_vote = super::super::polls::decode_vote(&vote_payload).unwrap();
        let hashes = super::super::polls::decrypt_vote(
            &context.secret,
            &context.poll_id,
            &peer_device.to_non_ad(),
            &chat,
            &decoded_vote,
        )
        .unwrap();
        assert_eq!(
            super::super::polls::resolve_options(&hashes, &context.options).0,
            vec!["a".to_owned()]
        );
        assert_eq!(context.target_chat, Some(chat));
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before_lookup
        );
        let own_vote_target = crate::model::message::MessageRef {
            key: crate::model::message::MessageKey {
                chat: crate::model::ids::ChatId::from_jid(&Jid::parse("67890@lid").unwrap())
                    .unwrap(),
                sender: None,
                direction: crate::model::message::Direction::Outgoing,
                id: poll_id.clone(),
            },
        };
        let own_vote = super::super::polls::encrypt_vote(
            &[7; 32],
            "OWN-POLL",
            &peer_device.to_non_ad(),
            &peer_device.to_non_ad(),
            &["b".into()],
            [5; 12],
        )
        .unwrap();
        let inner_vote = super::super::polls::vote_payload(
            &Jid::parse("67890@lid").unwrap(),
            &own_vote_target,
            own_vote,
            6,
        );
        let mut sent_vote = Vec::new();
        prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_vote);
        prost::encoding::bytes::encode(2, &inner_vote, &mut sent_vote);
        let mut wrapped_vote = Vec::new();
        prost::encoding::bytes::encode(31, &sent_vote, &mut wrapped_vote);
        let foreign_vote = super::super::polls::encrypt_vote(
            &[7; 32],
            "OWN-POLL",
            &peer_device.to_non_ad(),
            &Jid::parse("67890@lid").unwrap(),
            &["b".into()],
            [9; 12],
        )
        .unwrap();
        let foreign_inner = super::super::polls::vote_payload(
            &Jid::parse("67890@lid").unwrap(),
            &own_vote_target,
            foreign_vote,
            6,
        );
        let mut bad_sent = Vec::new();
        prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut bad_sent);
        prost::encoding::bytes::encode(2, &foreign_inner, &mut bad_sent);
        let mut bad_wrapped = Vec::new();
        prost::encoding::bytes::encode(31, &bad_sent, &mut bad_wrapped);
        let bad = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&synchronized_poll.session),
            Some(&sent.identity),
            &bad_wrapped,
        )
        .await
        .unwrap();
        let bad_node = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "BAD-OWN-VOTE")
            .attr("type", "poll")
            .children(vec![
                Node::new("enc")
                    .attr("type", bad.kind)
                    .attr("v", "2")
                    .bytes(bad.ciphertext.to_vec()),
            ]);
        let bad_capture = preserve(&store, &bad_node).unwrap();
        let before_bad = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::SignalSessions, 1, 100)?,
                    tx.scan(Table::PeerIdentities, 1, 100)?,
                ))
            })
            .unwrap();
        assert_eq!(
            process(&store, &receiver, &bad_node, &bad_capture)
                .await
                .unwrap(),
            IncomingResult::Unsupported
        );
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::SignalSessions, 1, 100)?,
                    tx.scan(Table::PeerIdentities, 1, 100)?
                )))
                .unwrap(),
            before_bad
        );
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store, &receiver, &bad_node
            )
            .unwrap()
            .is_none()
        );
        let synchronized_vote = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&synchronized_poll.session),
            Some(&sent.identity),
            &wrapped_vote,
        )
        .await
        .unwrap();
        let own_vote_node = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "OWN-VOTE")
            .attr("type", "poll")
            .children(vec![
                Node::new("meta").attr("polltype", "vote"),
                Node::new("enc")
                    .attr("type", synchronized_vote.kind)
                    .attr("v", "2")
                    .bytes(synchronized_vote.ciphertext.to_vec()),
            ]);
        let own_vote_capture = preserve(&store, &own_vote_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &own_vote_node, &own_vote_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        let votes = store
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("own-vote")?,
                    crate::model::ids::EventCursor::START,
                    100,
                )
            })
            .unwrap();
        let crate::sync::SyncEvent::MessageUpdated(update) = &votes.events.last().unwrap().event
        else {
            panic!("own vote event missing")
        };
        assert_eq!(update.key, own_vote_target.key);
        assert!(
            matches!(&update.change,crate::model::message::MessageChange::PollVote{by,selected,unresolved} if by.to_jid().to_non_ad()==peer_device.to_non_ad() && selected==&vec!["b".to_owned()] && unresolved.is_empty())
        );
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store,
                &receiver,
                &own_vote_node
            )
            .unwrap()
            .is_some()
        );
        let mut after_reactions_session = synchronized_vote.session.clone();
        let mut own_reactions = Vec::new();
        for (id, emoji) in [
            ("OWN-REACTION-SET", Some("👍")),
            ("OWN-REACTION-REMOVE", None),
        ] {
            let inner = super::super::reactions::payload(
                &Jid::parse("67890@lid").unwrap(),
                &Jid::parse("67890@lid").unwrap(),
                &own_vote_target,
                emoji,
                7,
            )
            .unwrap();
            let mut sent_reaction = Vec::new();
            prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_reaction);
            prost::encoding::bytes::encode(2, &inner, &mut sent_reaction);
            let mut wrapped = Vec::new();
            prost::encoding::bytes::encode(31, &sent_reaction, &mut wrapped);
            let synchronized = direct::prepare_payload(
                &sender,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&after_reactions_session),
                Some(&sent.identity),
                &wrapped,
            )
            .await
            .unwrap();
            let node = Node::new("message")
                .attr("from", peer_device.clone())
                .attr("recipient", "67890@lid")
                .attr("id", id)
                .attr("type", "reaction")
                .children(vec![
                    Node::new("enc")
                        .attr("type", synchronized.kind)
                        .attr("v", "2")
                        .bytes(synchronized.ciphertext.to_vec()),
                ]);
            let capture = preserve(&store, &node).unwrap();
            assert_eq!(
                process(&store, &receiver, &node, &capture).await.unwrap(),
                IncomingResult::Decrypted
            );
            let events = store
                .read(|tx| {
                    tx.read_events(
                        &crate::store::ConsumerId::new("own-reactions")?,
                        crate::model::ids::EventCursor::START,
                        100,
                    )
                })
                .unwrap();
            let crate::sync::SyncEvent::MessageUpdated(update) =
                &events.events.last().unwrap().event
            else {
                panic!("own reaction missing")
            };
            assert_eq!(update.key, own_vote_target.key);
            assert!(
                matches!(&update.change,crate::model::message::MessageChange::Reaction{by,emoji:actual} if by.to_jid().to_non_ad()==peer_device.to_non_ad() && actual.as_deref()==emoji)
            );
            assert!(
                super::super::messaging::received_confirmation_for_account(
                    &store, &receiver, &node
                )
                .unwrap()
                .is_some()
            );
            after_reactions_session = synchronized.session;
            own_reactions.push((node, capture));
        }
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan(Table::Outbox, 1, 10)?,
                    tx.scan(Table::Idempotency, 1, 10)?
                )))
                .unwrap(),
            before_echo
        );
        let mut after_settings_session = after_reactions_session.clone();
        let mut own_settings = Vec::new();
        for (id, kind, seconds) in [
            (
                "OWN-TIMER-SET",
                super::super::chat_settings::TYPE_EPHEMERAL_SETTING,
                86400,
            ),
            (
                "OWN-TIMER-OFF",
                super::super::chat_settings::TYPE_EPHEMERAL_SETTING,
                0,
            ),
            (
                "OWN-TIMER-SYNC",
                super::super::chat_settings::TYPE_EPHEMERAL_SYNC_RESPONSE,
                604800,
            ),
        ] {
            let inner = prost::Message::encode_to_vec(&super::super::chat_settings::Envelope {
                protocol: Some(super::super::chat_settings::ProtocolMessage {
                    type_: Some(kind),
                    ephemeral_expiration: Some(seconds),
                    ..Default::default()
                }),
                ..Default::default()
            });
            let mut sent_setting = Vec::new();
            prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_setting);
            prost::encoding::bytes::encode(2, &inner, &mut sent_setting);
            let mut wrapped = Vec::new();
            prost::encoding::bytes::encode(31, &sent_setting, &mut wrapped);
            let synchronized = direct::prepare_payload(
                &sender,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&after_settings_session),
                Some(&sent.identity),
                &wrapped,
            )
            .await
            .unwrap();
            let node = Node::new("message")
                .attr("from", peer_device.clone())
                .attr("recipient", "67890@lid")
                .attr("id", id)
                .attr("type", "text")
                .children(vec![
                    Node::new("enc")
                        .attr("type", synchronized.kind)
                        .attr("v", "2")
                        .bytes(synchronized.ciphertext.to_vec()),
                ]);
            let capture = preserve(&store, &node).unwrap();
            assert_eq!(
                process(&store, &receiver, &node, &capture).await.unwrap(),
                IncomingResult::Decrypted
            );
            let events = store
                .read(|tx| {
                    tx.read_events(
                        &crate::store::ConsumerId::new("own-timers")?,
                        crate::model::ids::EventCursor::START,
                        100,
                    )
                })
                .unwrap();
            let crate::sync::SyncEvent::NewMessage(message) = &events.events.last().unwrap().event
            else {
                panic!("own setting event missing")
            };
            assert_eq!(message.key.chat.jid(), &Jid::parse("67890@lid").unwrap());
            assert_eq!(
                message.key.direction,
                crate::model::message::Direction::Outgoing
            );
            assert!(message.key.sender.is_none());
            assert!(
                matches!(message.content,crate::model::message::MessageContent::DisappearingTimer {seconds:actual} if actual==seconds)
            );
            assert!(
                super::super::messaging::received_confirmation_for_account(
                    &store, &receiver, &node
                )
                .unwrap()
                .is_some()
            );
            after_settings_session = synchronized.session;
            own_settings.push((node, capture));
        }
        let mut after_keeps_session = after_settings_session.clone();
        let mut own_keeps = Vec::new();
        for (id, kept) in [("OWN-KEEP", true), ("OWN-UNKEEP", false)] {
            let inner = super::super::chat_settings::keep_payload(
                &Jid::parse("67890@lid").unwrap(),
                &own_vote_target,
                kept,
                19,
            );
            let mut keep_body = prost::Message::encode_to_vec(
                &<super::super::chat_settings::Envelope as prost::Message>::decode(
                    inner.as_slice(),
                )
                .unwrap()
                .keep_in_chat
                .unwrap(),
            );
            keep_body.extend_from_slice(b"\x10\x63");
            let mut inner = Vec::new();
            prost::encoding::bytes::encode(51, &keep_body, &mut inner);
            let mut sent_keep = Vec::new();
            prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_keep);
            prost::encoding::bytes::encode(2, &inner, &mut sent_keep);
            let mut wrapped = Vec::new();
            prost::encoding::bytes::encode(31, &sent_keep, &mut wrapped);
            let synchronized = direct::prepare_payload(
                &sender,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&after_keeps_session),
                Some(&sent.identity),
                &wrapped,
            )
            .await
            .unwrap();
            let node = Node::new("message")
                .attr("from", peer_device.clone())
                .attr("recipient", "67890@lid")
                .attr("id", id)
                .attr("type", "text")
                .children(vec![
                    Node::new("enc")
                        .attr("type", synchronized.kind)
                        .attr("v", "2")
                        .bytes(synchronized.ciphertext.to_vec()),
                ]);
            let capture = preserve(&store, &node).unwrap();
            assert_eq!(
                process(&store, &receiver, &node, &capture).await.unwrap(),
                IncomingResult::Decrypted
            );
            let events = store
                .read(|tx| {
                    tx.read_events(
                        &crate::store::ConsumerId::new("own-keep")?,
                        crate::model::ids::EventCursor::START,
                        100,
                    )
                })
                .unwrap();
            let crate::sync::SyncEvent::MessageUpdated(update) =
                &events.events.last().unwrap().event
            else {
                panic!("own keep event missing")
            };
            assert_eq!(update.key, own_vote_target.key);
            assert!(
                matches!(&update.change,crate::model::message::MessageChange::KeepInChat {by,kept:actual,sender_timestamp,update_id} if by.to_jid().to_non_ad()==peer_device.to_non_ad() && *actual==kept && *sender_timestamp==Some(crate::model::time::Timestamp(19)) && update_id.as_ref().is_some_and(|v|v.as_str()==id))
            );
            assert!(
                super::super::messaging::received_confirmation_for_account(
                    &store, &receiver, &node
                )
                .unwrap()
                .is_some()
            );
            after_keeps_session = synchronized.session;
            own_keeps.push((node, capture));
        }
        let mut own_edits = Vec::new();
        for (index, timestamp, accepted) in [
            (0, 20, true),
            (1, 19, false),
            (2, 20, true),
            (3, 21, true),
            (4, 0, true),
            (5, 22, false),
        ] {
            // Independent ProtocolMessage bytes: key=1, MESSAGE_EDIT=14, replacement=14, timestamp=15.
            let mut target = Vec::new();
            prost::encoding::string::encode(1, &"status@broadcast".to_owned(), &mut target);
            prost::encoding::bool::encode(2, &false, &mut target);
            prost::encoding::string::encode(3, &"OWN-LID-SYNC".to_owned(), &mut target);
            let mut protocol = Vec::new();
            prost::encoding::bytes::encode(1, &target, &mut protocol);
            if index == 4 {
                protocol.extend_from_slice(b"\x10\x00");
            } else {
                protocol.extend_from_slice(b"\x10\x0e");
                prost::encoding::bytes::encode(14, &b"\x0a\x06edited".to_vec(), &mut protocol);
                prost::encoding::int64::encode(15, &timestamp, &mut protocol);
            }
            protocol.extend_from_slice(b"\x10\x63");
            let mut body = Vec::new();
            prost::encoding::bytes::encode(12, &protocol, &mut body);
            let mut sent_edit = Vec::new();
            prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_edit);
            prost::encoding::bytes::encode(2, &body, &mut sent_edit);
            let mut wrapper = Vec::new();
            prost::encoding::bytes::encode(31, &sent_edit, &mut wrapper);
            let synchronized = direct::prepare_payload(
                &sender,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&after_keeps_session),
                Some(&sent.identity),
                &wrapper,
            )
            .await
            .unwrap();
            let node = Node::new("message")
                .attr("from", peer_device.clone())
                .attr("recipient", "67890@lid")
                .attr("id", format!("OWN-EDIT-{index}"))
                .attr("type", "text")
                .attr("edit", "1")
                .attr("t", "259202")
                .children(vec![
                    Node::new("enc")
                        .attr("type", synchronized.kind)
                        .attr("v", "2")
                        .bytes(synchronized.ciphertext.to_vec()),
                ]);
            let capture = preserve(&store, &node).unwrap();
            assert_eq!(
                process(&store, &receiver, &node, &capture).await.unwrap(),
                IncomingResult::Decrypted
            );
            let events = store
                .read(|tx| {
                    tx.read_events(
                        &crate::store::ConsumerId::new("own-edit")?,
                        crate::model::ids::EventCursor::START,
                        100,
                    )
                })
                .unwrap();
            match &events.events.last().unwrap().event {
                crate::sync::SyncEvent::MessageUpdated(update) if accepted => {
                    assert_eq!(update.key.id.as_str(), "OWN-LID-SYNC");
                    assert_eq!(
                        update.key.direction,
                        crate::model::message::Direction::Outgoing
                    );
                    assert!(update.key.sender.is_none());
                    assert_eq!(update.key.chat.jid(), &Jid::parse("67890@lid").unwrap());
                    if index == 4 {
                        assert!(matches!(
                            &update.change,
                            crate::model::message::MessageChange::Revoked { .. }
                        ));
                    } else {
                        assert!(
                            matches!(&update.change, crate::model::message::MessageChange::Edited(content)
                        if matches!(&**content, crate::model::message::MessageContent::Text {body,..} if body=="edited"))
                        );
                    }
                    assert_eq!(
                        update.server_timestamp,
                        Some(crate::model::time::Timestamp(259202000))
                    );
                }
                crate::sync::SyncEvent::UnsupportedMessage(_) if !accepted => {}
                other => panic!("unexpected own edit event: {other:?}"),
            }
            assert!(
                super::super::messaging::received_confirmation_for_account(
                    &store, &receiver, &node
                )
                .unwrap()
                .is_some()
            );
            after_keeps_session = synchronized.session;
            own_edits.push((node, capture));
        }
        let mut edited_sent = Vec::new();
        prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut edited_sent);
        prost::encoding::bytes::encode(2, &b"\x0a\x05hello".to_vec(), &mut edited_sent);
        let mut edited_wrapper = Vec::new();
        prost::encoding::bytes::encode(31, &edited_sent, &mut edited_wrapper);
        let unsupported_edit = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&after_keeps_session),
            Some(&sent.identity),
            &edited_wrapper,
        )
        .await
        .unwrap();
        let edited_node = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "OWN-EDIT-UNPORTED")
            .attr("type", "text")
            .attr("edit", "7")
            .children(vec![
                Node::new("enc")
                    .attr("type", unsupported_edit.kind)
                    .attr("v", "2")
                    .bytes(unsupported_edit.ciphertext.to_vec()),
            ]);
        let edited_capture = preserve(&store, &edited_node).unwrap();
        let before_edit = store
            .read(|tx| {
                Table::ALL
                    .iter()
                    .map(|t| tx.scan(*t, 1, 200))
                    .collect::<Result<Vec<_>>>()
            })
            .unwrap();
        let before_edit_head = store.journal_head().unwrap();
        assert_eq!(
            process(&store, &receiver, &edited_node, &edited_capture)
                .await
                .unwrap(),
            IncomingResult::Unsupported
        );
        assert_eq!(
            store
                .read(|tx| Table::ALL
                    .iter()
                    .map(|t| tx.scan(*t, 1, 200))
                    .collect::<Result<Vec<_>>>())
                .unwrap(),
            before_edit
        );
        assert_eq!(store.journal_head().unwrap(), before_edit_head);
        let mut own_revoke_protocol =
            prost::Message::encode_to_vec(&super::super::chat_settings::ProtocolMessage {
                key: Some(super::super::reactions::Key {
                    id: Some(own_vote_target.key.id.as_str().into()),
                    from_me: Some(false),
                    remote_jid: Some("status@broadcast".into()),
                    ..Default::default()
                }),
                type_: Some(0),
                ..Default::default()
            });
        own_revoke_protocol.extend_from_slice(b"\x10\x63");
        let mut own_revoke_body = Vec::new();
        prost::encoding::bytes::encode(12, &own_revoke_protocol, &mut own_revoke_body);
        let mut sent_revoke = Vec::new();
        prost::encoding::string::encode(1, &"67890@lid".to_owned(), &mut sent_revoke);
        prost::encoding::bytes::encode(2, &own_revoke_body, &mut sent_revoke);
        let mut revoke_wrapper = Vec::new();
        prost::encoding::bytes::encode(31, &sent_revoke, &mut revoke_wrapper);
        let synchronized = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&unsupported_edit.session),
            Some(&sent.identity),
            &revoke_wrapper,
        )
        .await
        .unwrap();
        let own_revoke_node = Node::new("message")
            .attr("from", peer_device.clone())
            .attr("recipient", "67890@lid")
            .attr("id", "OWN-REVOKE")
            .attr("t", "259202")
            .attr("type", "text")
            .attr("edit", "7")
            .children(vec![
                Node::new("enc")
                    .attr("type", synchronized.kind)
                    .attr("v", "2")
                    .bytes(synchronized.ciphertext.to_vec()),
            ]);
        let own_revoke_capture = preserve(&store, &own_revoke_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &own_revoke_node, &own_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        let revoke_events = store
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("own-revoke")?,
                    crate::model::ids::EventCursor::START,
                    100,
                )
            })
            .unwrap();
        let crate::sync::SyncEvent::MessageUpdated(update) =
            &revoke_events.events.last().unwrap().event
        else {
            panic!("own revoke missing")
        };
        assert_eq!(update.key, own_vote_target.key);
        assert!(
            matches!(&update.change,crate::model::message::MessageChange::Revoked {by} if by.to_jid()==peer_device.to_non_ad())
        );
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store,
                &receiver,
                &own_revoke_node
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan(Table::Outbox, 1, 10)?,
                    tx.scan(Table::Idempotency, 1, 10)?
                )))
                .unwrap(),
            before_echo
        );
        let after_revoke_session = synchronized.session;
        let after_poll_head = store.journal_head().unwrap();
        let event_count = batch.events.len()
            + 2 * usize::from(cfg!(feature = "experimental-registration-https"))
            + 17;
        drop(store);
        let store = open(false);
        assert_eq!(
            process(&store, &receiver, &own_revoke_node, &own_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            process(&store, &receiver, &peer_revoke_node, &peer_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            process(&store, &receiver, &old_revoke_node, &old_revoke_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        for (node, capture) in &own_edits {
            assert_eq!(
                process(&store, &receiver, node, capture).await.unwrap(),
                IncomingResult::Duplicate
            );
        }
        for (node, capture) in &own_keeps {
            assert_eq!(
                process(&store, &receiver, node, capture).await.unwrap(),
                IncomingResult::Duplicate
            );
        }
        for (node, capture) in &own_settings {
            assert_eq!(
                process(&store, &receiver, node, capture).await.unwrap(),
                IncomingResult::Duplicate
            );
        }
        for (node, capture) in &own_reactions {
            assert_eq!(
                process(&store, &receiver, node, capture).await.unwrap(),
                IncomingResult::Duplicate
            );
        }
        assert_eq!(
            process(&store, &receiver, &own_vote_node, &own_vote_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            process(&store, &receiver, &poll_node, &poll_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            process(&store, &receiver, &echo_node, &echo_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(store.journal_head().unwrap(), after_poll_head);
        assert_eq!(
            super::super::messaging::received_confirmation_for_account(
                &store, &receiver, &sync_node
            )
            .unwrap(),
            Some(expected_confirmation)
        );
        assert_eq!(
            process(&store, &receiver, &sync_node, &sync_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            store
                .read(|tx| tx.read_events(
                    &crate::store::ConsumerId::new("own-sync-reopen")?,
                    crate::model::ids::EventCursor::START,
                    100
                ))
                .unwrap()
                .events
                .len(),
            event_count
        );
        assert_eq!(
            super::super::chat_clear::clear_chat(&store, &Jid::parse("67890@lid").unwrap())
                .unwrap()
                .cleared_messages,
            18 + 2 * usize::from(cfg!(feature = "experimental-registration-https"))
        );
        #[cfg(feature = "experimental-registration-https")]
        assert!(super::super::media::list(&store).unwrap().is_empty());
        assert_eq!(
            process(&store, &receiver, &poll_node, &poll_capture)
                .await
                .unwrap(),
            IncomingResult::Unsupported
        );
        assert!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    b"received-own:67890@lid:OWN-POLL",
                    1
                ))
                .unwrap()
                .unwrap()["protobuf"]
                .is_null()
        );
        assert_eq!(
            process(&store, &receiver, &sync_node, &sync_capture)
                .await
                .unwrap(),
            IncomingResult::Unsupported
        );
        assert!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    b"received-own:67890@lid:OWN-LID-SYNC",
                    1
                ))
                .unwrap()
                .unwrap()["protobuf"]
                .is_null()
        );
        assert!(
            super::super::messaging::received_confirmation_for_account(
                &store, &receiver, &sync_node
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(
            super::super::polls::own_chat_binding(
                &store,
                &poll_id,
                &Jid::parse("67890@lid").unwrap()
            )
            .unwrap(),
            "unknown_poll"
        );
        let next = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            None,
            Some(&after_revoke_session),
            Some(&sent.identity),
            b"\x0a\x05hello",
        )
        .await
        .unwrap();
        assert_eq!(next.kind, "msg");
        let peer_node = Node::new("message")
            .attr("category", "peer")
            .attr("from", peer_device)
            .attr("id", "OWN-LID-PEER")
            .children(vec![
                Node::new("enc")
                    .attr("type", next.kind)
                    .attr("v", "2")
                    .bytes(next.ciphertext.to_vec()),
            ]);
        let capture = preserve(&store, &peer_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &peer_node, &capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        assert_eq!(
            process(&store, &receiver, &peer_node, &capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
    }

    #[tokio::test]
    async fn hybrid_prekeys_use_local_kem_inventory_and_continue_through_normal_inbox() {
        use crate::{
            model::ids::AccountId,
            session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
            signal::{direct, hybrid, kem},
            store::{MasterKey, OpenOptions, StaticKeySource},
            transport::{kem_prekeys, prekeys},
        };
        use wacore_libsignal::protocol::{
            AliceSignalProtocolParameters, GenericSignedPreKey, IdentityKey, IdentityKeyPair,
            KeyPair, PreKeyRecord, SignedPreKeyRecord, UsePQRatchet,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hybrid-inbox.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("hybrid-inbox-test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [39; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let mut config: crate::registration::request::LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.registration_id = 42;
        config.signed_pre_key_id = 7;
        crate::registration::factory::install_test_material(&store, &config).unwrap();
        let receiver = store.read(AccountRecord::load).unwrap().unwrap();
        let receiver_jid = receiver.own_jid.clone().unwrap();
        let signed = SignedPreKeyRecord::deserialize(
            &crate::registration::factory::registered_signed_prekey(&store).unwrap(),
        )
        .unwrap()
        .public_key()
        .unwrap();
        let mut receiver_identity = vec![5];
        receiver_identity.extend(receiver.signal_identity.as_ref().unwrap().0.public);
        let receiver_identity = IdentityKey::decode(&receiver_identity).unwrap();
        let (kem_public, kem_secret) = kem::generate_key_pair().unwrap();
        // A peer's native first message: signed/optional ordinary EC keys plus
        // an encapsulation to our published KEM key, pending fields retained.
        let initial = |sender: &IdentityKeyPair, one: Option<(u32, &KeyPair)>, kem_id: u32| {
            let mut parameters = AliceSignalProtocolParameters::new(
                sender.clone(),
                KeyPair::generate(&mut rand::rng()),
                receiver_identity,
                signed,
                signed,
                UsePQRatchet::No,
            );
            if let Some((_, one)) = one {
                parameters.set_their_one_time_pre_key(one.public_key);
            }
            let (ciphertext, shared) = kem::encapsulate(&kem_public).unwrap();
            hybrid::initialize_alice(
                &parameters,
                shared.as_bytes().try_into().unwrap(),
                &KeyPair::generate(&mut rand::rng()),
                (31, 42),
                (
                    one.map(|(id, _)| id),
                    7,
                    &direct::IncomingKem {
                        prekey_id: kem_id,
                        ciphertext: &ciphertext,
                    },
                ),
            )
            .unwrap()
        };
        let message = |from: &str, participant: Option<&str>, id: &str, kind: &str, body: &[u8]| {
            let node = Node::new("message")
                .attr("from", Jid::parse(from).unwrap())
                .attr("id", id.to_owned())
                .attr("t", "1")
                .children(vec![
                    Node::new("enc")
                        .attr("type", kind.to_owned())
                        .attr("v", "2")
                        .bytes(body.to_vec()),
                ]);
            match participant {
                Some(participant) => node.attr("participant", Jid::parse(participant).unwrap()),
                None => node,
            }
        };
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    Table::ALL
                        .iter()
                        .map(|table| tx.scan(*table, 1, 500))
                        .collect::<Result<Vec<_>>>()
                })
                .unwrap()
        };
        let row =
            |store: &ProtocolStore, id: u32| store.read(|tx| kem_prekeys::load(tx, id)).unwrap();
        let held = |store: &ProtocolStore, peer: &str| {
            let key = format!("comparison-direct:{peer}");
            store
                .read(|tx| {
                    Ok((
                        tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?,
                        tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1)?,
                    ))
                })
                .unwrap()
        };

        let peer = "123456789@lid";
        let alice = IdentityKeyPair::generate(&mut rand::rng());
        let one = KeyPair::generate(&mut rand::rng());
        let a = initial(&alice, Some((11, &one)), 9);
        let first = a.prepare(b"\x0a\x05hello").unwrap();
        let second = hybrid::HybridSession::from_bytes(&first.session)
            .unwrap()
            .prepare(b"\x0a\x05again")
            .unwrap();
        assert_eq!((first.kind, first.ciphertext[0]), ("pkmsg", 0x44));
        assert_eq!((second.kind, second.ciphertext[0]), ("pkmsg", 0x44));
        let ordinary = PreKeyRecord::new(11.into(), &one).serialize().unwrap();
        store
            .write(|tx| tx.put_json(Table::Prekeys, b"comparison-prekey:11", 1, &ordinary))
            .unwrap();
        let node1 = message(peer, None, "HYBRID-1", "pkmsg", &first.ciphertext);
        let capture1 = preserve(&store, &node1).unwrap();

        // No local KEM row: preserved, unacknowledged and nothing consumed.
        let before = snapshot(&store);
        let head = store.journal_head().unwrap();
        for policy in [
            IdentityChangePolicy::Pinned,
            IdentityChangePolicy::ApkAuthenticated,
        ] {
            assert_eq!(
                process_with_policy(&store, &receiver, &node1, &capture1, policy)
                    .await
                    .unwrap(),
                IncomingResult::Unsupported
            );
            assert_eq!(snapshot(&store), before);
            assert_eq!(store.journal_head().unwrap(), head);
        }

        // Inventory row present: bootstrap consumes the ordinary and KEM keys
        // in the inbox commit, pins the authenticated identity and publishes.
        store
            .write(|tx| {
                kem_prekeys::install_for_test(tx, 9, &kem_public, kem_secret.as_bytes(), false)
                    .map(drop)
            })
            .unwrap();
        assert_eq!(prekeys::summary(&store).unwrap().kem_local, 1);
        assert_eq!(
            process(&store, &receiver, &node1, &capture1).await.unwrap(),
            IncomingResult::Decrypted
        );
        assert!(row(&store, 9).is_none());
        assert!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:11", 1))
                .unwrap()
                .is_none()
        );
        let (session, pin) = held(&store, peer);
        assert!(hybrid::HybridSession::from_bytes(session.as_deref().unwrap()).is_ok());
        assert_eq!(pin.as_deref(), Some(&alice.identity_key().serialize()[..]));
        assert_eq!(
            store.read(|tx| tx.scan(Table::Inbox, 1, 10)).unwrap().len(),
            1
        );
        assert_eq!(
            process(&store, &receiver, &node1, &capture1).await.unwrap(),
            IncomingResult::Duplicate
        );

        // A later pending prekey matches the held session; no key is needed.
        let node2 = message(peer, None, "HYBRID-2", "pkmsg", &second.ciphertext);
        let capture2 = preserve(&store, &node2).unwrap();
        assert_eq!(
            process(&store, &receiver, &node2, &capture2).await.unwrap(),
            IncomingResult::Decrypted
        );

        // Our 0x44 reply clears the peer's pending KEM fields; their next
        // 0x44 msg decrypts through the ordinary inbox msg branch.
        let (session, pin) = held(&store, peer);
        let reply = direct::prepare_payload(
            &receiver,
            &Jid::parse(peer).unwrap(),
            None,
            session.as_deref(),
            pin.as_deref(),
            b"\x0a\x05reply",
        )
        .await
        .unwrap();
        assert_eq!((reply.kind, reply.ciphertext[0]), ("msg", 0x44));
        store
            .write(|tx| {
                tx.put_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &*reply.session,
                )
            })
            .unwrap();
        let alice_account = AccountRecord {
            own_jid: Some(Jid::parse(peer).unwrap()),
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: alice.public_key().public_key_bytes().try_into().unwrap(),
                private: *alice.private_key().serialize(),
            })),
            ..Default::default()
        };
        let acknowledged = direct::receive(
            &alice_account,
            &receiver_jid,
            &second.session,
            &receiver_identity.serialize(),
            &reply.ciphertext,
        )
        .await
        .unwrap();
        assert_eq!(acknowledged.plaintext.as_slice(), b"\x0a\x05reply");
        let third = hybrid::HybridSession::from_bytes(&acknowledged.session)
            .unwrap()
            .prepare(b"\x0a\x05third")
            .unwrap();
        assert_eq!((third.kind, third.ciphertext[0]), ("msg", 0x44));
        let node3 = message(peer, None, "HYBRID-3", "msg", &third.ciphertext);
        let capture3 = preserve(&store, &node3).unwrap();
        assert_eq!(
            process(&store, &receiver, &node3, &capture3).await.unwrap(),
            IncomingResult::Decrypted
        );
        assert_eq!(
            store.read(|tx| tx.scan(Table::Inbox, 1, 10)).unwrap().len(),
            3
        );

        // Native last-resort rows survive successful bootstrap (0kh.BWa).
        let last_resort_peer = "15555550104@s.whatsapp.net";
        store
            .write(|tx| {
                kem_prekeys::install_for_test(tx, 10, &kem_public, kem_secret.as_bytes(), true)
                    .map(drop)
            })
            .unwrap();
        let bob = IdentityKeyPair::generate(&mut rand::rng());
        let last = initial(&bob, None, 10).prepare(b"\x0a\x04last").unwrap();
        let node4 = message(
            last_resort_peer,
            None,
            "HYBRID-LR",
            "pkmsg",
            &last.ciphertext,
        );
        let capture4 = preserve(&store, &node4).unwrap();
        assert_eq!(
            process(&store, &receiver, &node4, &capture4).await.unwrap(),
            IncomingResult::Decrypted
        );
        assert!(row(&store, 10).is_some_and(|row| row.last_resort));
        let summary = prekeys::summary(&store).unwrap();
        assert_eq!((summary.kem_local, summary.kem_last_resort), (1, 1));

        // A changed identity under the pinned policy fails like the classic
        // untrusted-identity path: no session, pin or KEM row changes.
        store
            .write(|tx| {
                kem_prekeys::install_for_test(tx, 12, &kem_public, kem_secret.as_bytes(), false)
                    .map(drop)
            })
            .unwrap();
        let replacement = IdentityKeyPair::generate(&mut rand::rng());
        let changed = initial(&replacement, None, 12)
            .prepare(b"\x0a\x03new")
            .unwrap();
        let node5 = message(peer, None, "HYBRID-CHANGED", "pkmsg", &changed.ciphertext);
        let capture5 = preserve(&store, &node5).unwrap();
        let pinned_before = held(&store, peer);
        assert_eq!(
            process(&store, &receiver, &node5, &capture5).await.unwrap(),
            IncomingResult::Undecryptable
        );
        assert_eq!(held(&store, peer), pinned_before);
        assert!(row(&store, 12).is_some());
        // The explicit APK-authenticated policy archives and replaces the pin.
        assert_eq!(
            process_with_policy(
                &store,
                &receiver,
                &node5,
                &capture5,
                IdentityChangePolicy::ApkAuthenticated
            )
            .await
            .unwrap(),
            IncomingResult::Decrypted
        );
        assert!(row(&store, 12).is_none());
        assert_eq!(
            held(&store, peer).1.as_deref(),
            Some(&replacement.identity_key().serialize()[..])
        );

        // Group pairwise children use the same hybrid path and consumption.
        let member = "15555550105@s.whatsapp.net";
        let carol = IdentityKeyPair::generate(&mut rand::rng());
        let carol_one = KeyPair::generate(&mut rand::rng());
        store
            .write(|tx| {
                tx.put_json(
                    Table::Prekeys,
                    b"comparison-prekey:14",
                    1,
                    &PreKeyRecord::new(14.into(), &carol_one)
                        .serialize()
                        .unwrap(),
                )?;
                kem_prekeys::install_for_test(tx, 13, &kem_public, kem_secret.as_bytes(), false)
                    .map(drop)
            })
            .unwrap();
        let group_first = initial(&carol, Some((14, &carol_one)), 13)
            .prepare(b"\x0a\x05group")
            .unwrap();
        let group_node = message(
            "12345@g.us",
            Some(member),
            "HYBRID-G",
            "pkmsg",
            &group_first.ciphertext,
        );
        let group_capture = preserve(&store, &group_node).unwrap();
        assert_eq!(
            process(&store, &receiver, &group_node, &group_capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        assert!(row(&store, 13).is_none());
        assert!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:14", 1))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            held(&store, member).1.as_deref(),
            Some(&carol.identity_key().serialize()[..])
        );
        assert_eq!(
            process(&store, &receiver, &group_node, &group_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );

        // Companion KEM admission is unported on both routes; nothing changes.
        store
            .write(|tx| {
                kem_prekeys::install_for_test(tx, 15, &kem_public, kem_secret.as_bytes(), false)
                    .map(drop)
            })
            .unwrap();
        let companion = initial(&alice, None, 15).prepare(b"\x0a\x02hi").unwrap();
        for node in [
            message(
                "123456789:3@lid",
                None,
                "HYBRID-COMPANION",
                "pkmsg",
                &companion.ciphertext,
            ),
            message(
                "12345@g.us",
                Some("15555550105:2@s.whatsapp.net"),
                "HYBRID-GC",
                "pkmsg",
                &companion.ciphertext,
            ),
        ] {
            let capture = preserve(&store, &node).unwrap();
            let before = snapshot(&store);
            assert_eq!(
                process(&store, &receiver, &node, &capture).await.unwrap(),
                IncomingResult::Unsupported
            );
            assert_eq!(snapshot(&store), before);
        }

        // Reopening keeps every commit once-only and the unused rows intact.
        let after = snapshot(&store);
        drop(store);
        let store = open(false);
        for (node, capture) in [
            (&node1, &capture1),
            (&node2, &capture2),
            (&node3, &capture3),
        ] {
            assert_eq!(
                process(&store, &receiver, node, capture).await.unwrap(),
                IncomingResult::Duplicate
            );
        }
        assert_eq!(snapshot(&store), after);
        assert!(row(&store, 10).is_some() && row(&store, 15).is_some());
    }

    #[tokio::test]
    async fn sender_key_only_group_control_is_durable_without_user_event_or_read_receipt() {
        for sender in [
            "15555550101@s.whatsapp.net",
            "123456789@lid",
            "15555550101:3@s.whatsapp.net",
            "123456789:3@lid",
        ] {
            assert_sender_key_control_route(sender).await;
        }
    }

    async fn assert_sender_key_control_route(raw_sender: &str) {
        use crate::{
            model::ids::AccountId,
            session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
            signal::{direct, group},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        use wacore_libsignal::protocol::{
            DeviceId, GenericSignedPreKey, IdentityKeyPair, KeyPair, PreKeyBundle,
            SignedPreKeyRecord, Timestamp,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("control.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("fixture").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [22; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let alice = IdentityKeyPair::generate(&mut rand::rng());
        let bob = IdentityKeyPair::generate(&mut rand::rng());
        let alice_jid = Jid::parse(raw_sender).unwrap();
        let bob_jid = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let account = |identity: &IdentityKeyPair, jid: Jid, id| AccountRecord {
            own_jid: Some(jid),
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(id),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: identity.public_key().public_key_bytes().try_into().unwrap(),
                private: *identity.private_key().serialize(),
            })),
            ..Default::default()
        };
        let alice_account = account(&alice, alice_jid.to_non_ad(), 31);
        let bob_account = account(&bob, bob_jid.clone(), 42);
        let signed = KeyPair::generate(&mut rand::rng());
        let signature = alice
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
            .unwrap();
        let bundle = PreKeyBundle::new(
            31,
            DeviceId::new(alice_jid.device.into()),
            None,
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *alice.identity_key(),
        )
        .unwrap();
        let first = direct::prepare_payload(
            &bob_account,
            &alice_jid,
            Some(&bundle),
            None,
            None,
            b"\x0a\x04init",
        )
        .await
        .unwrap();
        let signed_record = SignedPreKeyRecord::new(
            7.into(),
            Timestamp::from_epoch_millis(0),
            &signed,
            &signature,
        )
        .serialize()
        .unwrap();
        let incoming = direct::receive_prekey(
            &alice_account,
            &bob_jid,
            (None, None),
            &signed_record,
            None,
            &first.ciphertext,
        )
        .await
        .unwrap();
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let own_key = group::prepare_own_sender_key(&chat, &alice_jid, None)
            .await
            .unwrap();
        let distribution = direct::prepare_payload(
            &alice_account,
            &bob_jid,
            None,
            Some(&incoming.received.session),
            Some(&incoming.identity),
            &own_key.distribution_message,
        )
        .await
        .unwrap();
        assert_eq!(distribution.kind, "msg");
        let peer_key = format!("comparison-direct:{alice_jid}");
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    peer_key.as_bytes(),
                    1,
                    &first.identity,
                )?;
                tx.put_json(
                    Table::SignalSessions,
                    peer_key.as_bytes(),
                    1,
                    &*first.session,
                )
            })
            .unwrap();
        let node = Node::new("message")
            .attr("from", chat.clone())
            .attr("participant", alice_jid.clone())
            .attr("id", "KEY-ONLY")
            .children(vec![
                Node::new("enc")
                    .attr("v", "2")
                    .attr("type", "msg")
                    .bytes(distribution.ciphertext.to_vec()),
            ]);
        let capture = preserve(&store, &node).unwrap();
        assert_eq!(
            process(&store, &bob_account, &node, &capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        assert_eq!(
            super::super::receipts::queue_read_for_incoming(&store, &node).unwrap(),
            0
        );
        let marker_key = super::super::group_inbox::key(&chat, &alice_jid, "KEY-ONLY").unwrap();
        let marker: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, marker_key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(marker["status"], "sender_key_distributed");
        assert!(marker.get("protobuf").is_none());
        assert!(!marker["route"].is_null() && !marker["capture"].is_null());
        let state = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::SenderKeys, 1, 10)?,
                    tx.scan(Table::SignalSessions, 1, 10)?,
                ))
            })
            .unwrap();
        assert_eq!(state.0.len(), 1);
        drop(store);
        let store = open(false);
        assert_eq!(
            process(&store, &bob_account, &node, &capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan(Table::SenderKeys, 1, 10)?,
                    tx.scan(Table::SignalSessions, 1, 10)?
                )))
                .unwrap(),
            state
        );
        assert_eq!(
            super::super::receipts::queue_read_for_incoming(&store, &node).unwrap(),
            0
        );
        let redelivery = node.clone().children(vec![
            Node::new("enc")
                .attr("v", "2")
                .attr("type", "msg")
                .bytes(vec![9; 48]),
        ]);
        let redelivery_capture = preserve(&store, &redelivery).unwrap();
        assert_eq!(
            process(&store, &bob_account, &redelivery, &redelivery_capture)
                .await
                .unwrap(),
            IncomingResult::Duplicate
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    marker_key.as_bytes(),
                    1
                ))
                .unwrap(),
            Some(marker)
        );
        assert!(super::super::receipts::outcomes(&store).unwrap().is_empty());
        let events = store
            .read_events(
                &crate::store::ConsumerId::new("fixture").unwrap(),
                crate::model::ids::EventCursor::START,
                100,
            )
            .unwrap();
        assert!(events.events.is_empty());
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
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"pn":"15555550100@s.whatsapp.net","lid":"123456789@lid"}),
                )
            })
            .unwrap();
        for (from, kind) in [
            ("15555550100:2@s.whatsapp.net", "msg"),
            ("15555550100:2@s.whatsapp.net", "pkmsg"),
            ("123456789:2@lid", "msg"),
            ("123456789:2@lid", "pkmsg"),
        ] {
            let node = Node::new("message")
                .attr("category", "peer")
                .attr("from", from)
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
            assert_eq!(
                process_with_policy(
                    &store,
                    &account,
                    &node,
                    &capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                IncomingResult::Unsupported
            );
        }
        for own in ["15555550100@s.whatsapp.net", "123456789:2@lid"] {
            assert_eq!(
                remote_identity_policy(
                    &store,
                    &account,
                    &Jid::parse(own).unwrap(),
                    IdentityChangePolicy::ApkAuthenticated
                )
                .unwrap(),
                IdentityChangePolicy::Pinned
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
    fn recipient_routes_do_not_project_as_ordinary_incoming_chats() {
        for from in ["15555550100@s.whatsapp.net", "12345@lid", "12345:2@lid"] {
            let node = Node::new("message").attr("from", from).attr("id", "SYNC");
            assert!(session_sender(&node).is_some());
            for recipient in ["15555550200@s.whatsapp.net", "67890@lid", "invalid", ""] {
                let qualified = node.clone().attr("recipient", recipient);
                assert!(session_sender(&qualified).is_none());
                assert!(
                    direct_record_in(&qualified, |_| panic!("must not read an incoming row"))
                        .unwrap()
                        .is_none()
                );
            }
        }
    }

    #[test]
    fn peer_messages_accept_only_the_saved_own_lid_device() {
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            ..Default::default()
        };
        let own_lid = Jid::parse("12345@lid").unwrap();
        let sender = Jid::parse("12345:2@lid").unwrap();
        let node = Node::new("message")
            .attr("category", "peer")
            .attr("from", sender.clone());
        assert_eq!(
            own_peer_sender(&account, &node, Some(&own_lid)),
            Some(sender)
        );
        assert!(own_peer_sender(&account, &node, None).is_none());
        for from in ["99999:2@lid", "12345@lid", "12345_1:2@lid"] {
            assert!(
                own_peer_sender(&account, &node.clone().attr("from", from), Some(&own_lid))
                    .is_none()
            );
        }
        for node in [
            node.clone().attr("recipient", "99999@lid"),
            node.clone().attr("participant", "99999@lid"),
            Node::new("message").attr("from", "12345:2@lid"),
        ] {
            assert!(own_peer_sender(&account, &node, Some(&own_lid)).is_none());
        }
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
                &node.clone().attr("from", "15555550100:2@s.whatsapp.net"),
                None
            )
            .is_some()
        );
        for sender in [
            "15555550101:2@s.whatsapp.net",
            "15555550100@s.whatsapp.net",
            "12345:2@lid",
            "12345@g.us",
        ] {
            assert!(own_peer_sender(&account, &node.clone().attr("from", sender), None).is_none());
        }
        assert!(
            own_peer_sender(
                &account,
                &Node::new("message").attr("from", "15555550100:2@s.whatsapp.net"),
                None
            )
            .is_none()
        );
        assert!(
            own_peer_sender(
                &AccountRecord::default(),
                &node.attr("from", "15555550100:2@s.whatsapp.net"),
                None
            )
            .is_none()
        );
    }
}
