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
    if node.get_attr("participant").is_some()
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
    // Group messages carry a `participant` attribute (the sender device JID);
    // `from` is the group JID. Route them to the sender-key path.
    #[cfg(feature = "experimental-signal-crypto")]
    if node.get_attr("participant").is_some() {
        return process_group(store, account, node, capture, policy).await;
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
        let incoming = crate::signal::direct::receive_prekey_with_policy(
            account,
            &session_peer,
            (
                session.as_deref().map(|v| v.as_slice()),
                identity.as_deref(),
                if peer_category {
                    IdentityChangePolicy::Pinned
                } else {
                    remote_identity_policy(store, account, &session_peer, policy)?
                },
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
            let incoming = crate::signal::direct::receive_prekey_with_policy(
                account,
                &peer,
                (
                    session.as_deref(),
                    identity.as_deref(),
                    remote_identity_policy(store, account, &peer, policy)?,
                ),
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
        // shortcut: broadcast-list class is unported, replace after its ordinary receive dispatch trace.
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
            .children(vec![
                Node::new("enc")
                    .attr("type", sent.kind)
                    .attr("v", "2")
                    .bytes(sent.ciphertext.clone()),
            ]);
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
