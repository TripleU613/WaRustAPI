//! Local address book plus an APK-derived `usync` contact-existence check
//! (docs/protocol/chat-actions.md section 7).
//!
//! Two independent things live here:
//!
//! * A purely local address book (`contact:<digits>` rows). Nothing here ever
//!   sends a saved contact anywhere; syncing it to companion devices is the
//!   `contactAction` `w:sync:app:state` mutation described in
//!   docs/protocol/chat-actions.md section 6 (`SyncActionValue.contactAction`,
//!   `LX/BXZ`: fullName, firstName, lidJid, saveOnPrimaryAddressbook, pnJid,
//!   username) and is **not implemented** here.
//! * A `usync` "is this number on WhatsApp" check, queued through the durable
//!   one-shot [`super::actions`] queue and ingested from its settled reply.
//!   The query reuses the exact `usync mode=query context=interactive index=0
//!   last=true` envelope that `transport/devices.rs` already sends
//!   (`LX/0u7.A06`, confirmed), with protocol children `contact`, `lid` and
//!   `disappearing_mode` (chat-actions.md section 7, confirmed vocabulary) and
//!   `<list><user><contact>+E.164</contact></user></list>`. The reply's `user`
//!   carries `jid` when the number is on WhatsApp and a `contact
//!   type="in"|"out"` child whose text is the number (confirmed); a `lid`
//!   child uses `val`, gated by successful `usync/result/lid` metadata
//!   (APK 37.71 `LX/0u7.C8f`; chat-actions.md section 7).
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::{
        jid::{Jid, JidKind},
        time::Timestamp,
    },
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};

const CONTACT_PREFIX: &str = "contact:";
const SYNC_ACTION_KIND: &str = "contact-sync";
const STATUS_PREFIX: &str = "contact-status:";
const CHECK_REQUEST_PREFIX: &str = "contact-check-req:";
const INGESTED_PREFIX: &str = "contacts-ingested:";
pub(crate) const ACTION_KIND: &str = "contact-check";

fn invalid(reason: impl Into<String>) -> Error {
    Error::invalid("contacts", reason)
}

/// Normalize a phone number to (digits-only key, canonical `+E.164` display).
/// Accepts an optional leading `+`; the remainder must be 6-15 ASCII digits.
fn normalize(phone: &str) -> Result<(String, String)> {
    let trimmed = phone.trim();
    let digits = trimmed.strip_prefix('+').unwrap_or(trimmed);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(
            "phone must contain only digits, with an optional leading +",
        ));
    }
    if !(6..=15).contains(&digits.len()) {
        return Err(invalid("phone must have 6 to 15 digits"));
    }
    Ok((digits.to_owned(), format!("+{digits}")))
}

fn contact_key(digits: &str) -> String {
    format!("{CONTACT_PREFIX}{digits}")
}
fn status_key(digits: &str) -> String {
    format!("{STATUS_PREFIX}{digits}")
}
fn check_request_key(action_id: &str) -> String {
    format!("{CHECK_REQUEST_PREFIX}{action_id}")
}
fn ingested_key(action_id: &str) -> String {
    format!("{INGESTED_PREFIX}{action_id}")
}

fn validate_name(name: &str, field: &'static str) -> Result<()> {
    if name.is_empty() || name.chars().count() > 256 || name.chars().any(char::is_control) {
        return Err(Error::invalid(
            field,
            "must be 1..=256 non-control characters",
        ));
    }
    Ok(())
}

/// A saved local contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contact {
    /// Canonical `+E.164` display form (the address-book key is its digits).
    pub phone: String,
    /// Display name.
    pub full_name: String,
    /// Optional given name (mirrors `contactAction.firstName`).
    pub first_name: Option<String>,
    /// When this row was first added.
    pub created_at: Timestamp,
    /// When this row was last edited.
    pub updated_at: Timestamp,
}

/// Add a new local contact. Fails if one already exists for this number.
pub fn add(
    store: &ProtocolStore,
    phone: &str,
    full_name: &str,
    first_name: Option<&str>,
) -> Result<Contact> {
    let (digits, canonical) = normalize(phone)?;
    validate_name(full_name, "full_name")?;
    if let Some(first) = first_name {
        validate_name(first, "first_name")?;
    }
    let key = contact_key(&digits);
    store.write(|tx| {
        if tx
            .get_json::<Contact>(Table::Checkpoints, key.as_bytes(), 1)?
            .is_some()
        {
            return Err(invalid("a contact for this number already exists"));
        }
        let now = Timestamp::now();
        let contact = Contact {
            phone: canonical.clone(),
            full_name: full_name.to_owned(),
            first_name: first_name.map(str::to_owned),
            created_at: now,
            updated_at: now,
        };
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &contact)?;
        Ok(contact)
    })
}

/// Edit an existing local contact's name fields.
pub fn edit(
    store: &ProtocolStore,
    phone: &str,
    full_name: &str,
    first_name: Option<&str>,
) -> Result<Contact> {
    let (digits, _) = normalize(phone)?;
    validate_name(full_name, "full_name")?;
    if let Some(first) = first_name {
        validate_name(first, "first_name")?;
    }
    let key = contact_key(&digits);
    store.write(|tx| {
        let mut contact: Contact = tx
            .get_json(Table::Checkpoints, key.as_bytes(), 1)?
            .ok_or_else(|| invalid("no contact saved for this number"))?;
        contact.full_name = full_name.to_owned();
        contact.first_name = first_name.map(str::to_owned);
        contact.updated_at = Timestamp::now();
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &contact)?;
        Ok(contact)
    })
}

/// Remove a local contact. Returns whether one existed.
pub fn remove(store: &ProtocolStore, phone: &str) -> Result<bool> {
    let (digits, _) = normalize(phone)?;
    let key = contact_key(&digits);
    store.write(|tx| tx.delete(Table::Checkpoints, key.as_bytes()))
}

/// One saved local contact, if any.
pub fn get(store: &ProtocolStore, phone: &str) -> Result<Option<Contact>> {
    let (digits, _) = normalize(phone)?;
    store.read(|tx| tx.get_json(Table::Checkpoints, contact_key(&digits).as_bytes(), 1))
}

/// Every saved local contact, in key order.
pub fn list(store: &ProtocolStore) -> Result<Vec<Contact>> {
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                if !key.starts_with(CONTACT_PREFIX.as_bytes()) {
                    continue;
                }
                if let Some(contact) = tx.get_json(Table::Checkpoints, &key, 1)? {
                    out.push(contact);
                }
            }
            if done {
                return Ok(out);
            }
        }
    })
}

/// Queue a **full** `usync` address-book sync through the durable one-shot
/// action queue. Returns the action id and the number of contacts included;
/// nothing is sent until a session drains queued actions.
///
/// Envelope confirmed from comparison 2.26.37.71 `LX/0u7->A06`: the same
/// `iq xmlns="usync" type="get"` / `<usync sid index last mode context>` /
/// `<query>` / per-`<user><contact>+E.164</contact></user>` structure the
/// existence check already uses — a sync differs from a query only in
/// `mode`/`context`, not in per-user encoding. The mode and context vocabularies
/// are confirmed (`LX/0xb`: `full`, `delta`, `query`, `download`; `LX/0xY`:
/// `metadata`, `registration`, `interactive`, `background`, `notification`,
/// `message`, `add`, `voip`, `snapshot`, `wamo`), and `LX/0xX` pairs them:
/// `INTERACTIVE_FULL` is `full`/`interactive`, while `INTERACTIVE_QUERY` is the
/// `query`/`interactive` pair this module already sends — which is what
/// validates the reading.
///
/// `allow_mutation="true"` is **not** emitted. The attribute exists in the
/// builder, but what enables it is **unknown** (the six-argument constructor
/// hardcodes it false), and guessing it is not worth the risk.
///
/// **Why this exists:** the account had only ever sent one-number
/// `query`/`interactive` existence checks, which are discovery, not
/// address-book registration. Whether registering contacts affects Status
/// delivery is **unknown** — no APK evidence links contact sync to status — so
/// this is a hypothesis test, not an evidence-backed fix. See
/// `docs/protocol/status.md`.
#[cfg(any(
    test,
    all(
        feature = "experimental-signal-crypto",
        feature = "experimental-noise",
        feature = "experimental-transport"
    )
))]
pub fn queue_full_sync(store: &ProtocolStore) -> Result<(String, usize)> {
    let contacts = list(store)?;
    if contacts.is_empty() {
        return Err(invalid("no saved contacts to sync"));
    }
    if contacts.len() > 1024 {
        return Err(invalid("local bound on contacts per sync"));
    }
    let canonical: Vec<String> = contacts.iter().map(|c| c.phone.clone()).collect();
    let count = canonical.len();
    let id = super::actions::queue(store, SYNC_ACTION_KIND, "contact sync", |id| {
        let sid = super::messaging::new_message_id()?;
        let users: Vec<Node> = canonical
            .iter()
            .map(|phone| Node::new("user").children(vec![Node::new("contact").text(phone.clone())]))
            .collect();
        Ok(Node::new("iq")
            .attr("xmlns", "usync")
            .attr("id", id)
            .attr("type", "get")
            .children(vec![
                Node::new("usync")
                    .attr("sid", sid.as_str())
                    .attr("index", "0")
                    .attr("last", "true")
                    .attr("mode", "full")
                    .attr("context", "interactive")
                    .children(vec![
                        Node::new("query").children(vec![
                            Node::new("contact"),
                            Node::new("lid"),
                            Node::new("disappearing_mode"),
                        ]),
                        Node::new("list").children(users),
                    ]),
            ]))
    })?;
    Ok((id, count))
}

/// APK 37.71 0u7.C8f contact@type values, stored in N0z.A04 as 1/2/3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactType {
    /// The APK consumer sets the positive account flag only for this value.
    In,
    /// The server returned `out`, distinct from invalid input.
    Out,
    /// The server returned `invalid`; this is not proof of a ban.
    Invalid,
}

/// The saved WhatsApp-existence status for one number, if a check ever settled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactStatus {
    /// Positive only for an `in` result; inspect contact_type/error for other states.
    /// Historical rows without contact_type retain their original boolean.
    pub on_whatsapp: bool,
    /// Exact recognized server type; null for historical/refused/malformed results.
    #[serde(default)]
    pub contact_type: Option<ContactType>,
    /// The account's phone-number JID, when returned.
    pub jid: Option<Jid>,
    /// The plain account LID from `lid@val`, when its protocol result succeeded.
    pub lid: Option<Jid>,
    /// When this status was recorded.
    pub checked_at: Timestamp,
    /// Sanitized server refusal or fixed local missing/unsupported-contact-type reason.
    /// Neither a refusal nor malformed metadata confirms account absence.
    pub error: Option<String>,
}

/// Queue a `usync` existence check for `phone` through the durable one-shot
/// action queue. Returns the action id; nothing is sent until a comparison
/// session drains queued actions (see `super::actions`).
pub fn queue_check(store: &ProtocolStore, phone: &str) -> Result<String> {
    let (digits, canonical) = normalize(phone)?;
    // Queue bytes and reply-correlation provenance must survive together.
    // A partial preparation is not eligible for later transmission.
    store.write(|tx| {
        let id = super::actions::queue_in(tx, ACTION_KIND, "contact check", |id| {
            let sid = super::messaging::new_message_id()?;
            Ok(Node::new("iq")
                .attr("xmlns", "usync")
                .attr("id", id)
                .attr("type", "get")
                .children(vec![
                    Node::new("usync")
                        .attr("sid", sid.as_str())
                        .attr("index", "0")
                        .attr("last", "true")
                        .attr("mode", "query")
                        .attr("context", "interactive")
                        .children(vec![
                            Node::new("query").children(vec![
                                Node::new("contact"),
                                Node::new("lid"),
                                Node::new("disappearing_mode"),
                            ]),
                            Node::new("list").children(vec![
                                Node::new("user")
                                    .children(vec![Node::new("contact").text(canonical.clone())]),
                            ]),
                        ]),
                ]))
        })?;
        // Labels must never carry a phone number, so the checked number is kept
        // in its own row: a top-level `iq type=error` reply carries no per-user
        // node to recover it from.
        tx.put_json(
            Table::Checkpoints,
            check_request_key(&id).as_bytes(),
            1,
            &digits,
        )?;
        Ok(id)
    })
}

/// Local provenance guard, not a native usync gate or a retry permission.
pub(crate) fn check_correlation_in(
    tx: &crate::store::WriteTx<'_>,
    id: &str,
    request: &Node,
) -> Result<bool> {
    let Some(saved): Option<serde_json::Value> =
        tx.get_json(Table::Checkpoints, check_request_key(id).as_bytes(), 1)?
    else {
        return Ok(false);
    };
    let bad = || invalid("saved contact request correlation does not match its request");
    let saved = saved.as_str().ok_or_else(bad)?;
    let (digits, canonical) = normalize(saved).map_err(|_| bad())?;
    if digits != saved
        || request.description != "iq"
        || request.get_attr("id").and_then(|value| value.as_text()) != Some(id)
        || request.get_attr("xmlns").and_then(|v| v.as_text()) != Some("usync")
        || request.get_attr("type").and_then(|v| v.as_text()) != Some("get")
    {
        return Err(bad());
    }
    let [usync] = request.child_nodes() else {
        return Err(bad());
    };
    if usync.description != "usync" {
        return Err(bad());
    }
    let mut lists = usync
        .child_nodes()
        .iter()
        .filter(|node| node.description == "list");
    let list = lists.next().ok_or_else(bad)?;
    if lists.next().is_some() {
        return Err(bad());
    }
    let [user] = list.child_nodes() else {
        return Err(bad());
    };
    let [contact] = user.child_nodes() else {
        return Err(bad());
    };
    let text = match &contact.content {
        NodeContent::Text(value) => Some(value.as_str()),
        NodeContent::Bytes(value) => std::str::from_utf8(value).ok(),
        _ => None,
    };
    if user.description != "user"
        || contact.description != "contact"
        || text != Some(canonical.as_str())
    {
        return Err(bad());
    }
    Ok(true)
}

fn attr_jid(node: &Node, name: &str) -> Option<Jid> {
    let value = node.get_attr(name)?;
    value
        .as_jid()
        .cloned()
        .or_else(|| Jid::parse(value.as_text()?).ok())
}

// APK 37.71 0u7.A01: non-contact protocols succeed only without an error
// child; optional refresh uses Long.parseLong. A3p.<init> stores that flag.
fn lid_protocol_succeeded(usync: &Node) -> bool {
    let Some(lid) = usync.child("result").and_then(|result| result.child("lid")) else {
        return false;
    };
    lid.child("error").is_none()
        && lid.get_attr("refresh").is_none_or(|value| {
            value
                .as_text()
                .and_then(super::mex::java_decimal_long)
                .is_some()
        })
}

fn parse_lid(node: &Node) -> Option<Jid> {
    let value = node.get_attr("val")?;
    // 0hs.A0A performs a typed class cast, returning null for a wrong subtype.
    // Reuse the traced 0D1/0ZY plain-address validation, including hosted.lid
    // canonicalization. Validate text before Jid::parse loses explicit :0.
    // Observed 2026-10-08: a live lid@val decodes from AD_JID with both
    // components zero. JidKind tracks encoding, not the APK Java subtype;
    // rejecting that encoding discarded a valid primary account LID.
    let text = if let Some(jid) = value.as_jid() {
        if !matches!(jid.kind, JidKind::Plain | JidKind::AgentDevice)
            || jid.agent != 0
            || jid.device != 0
        {
            return None;
        }
        jid.to_string()
    } else {
        value.as_text()?.to_owned()
    };
    let address = super::mex::ParticipantAddress::parse(&text).ok()?;
    if address.kind() != super::mex::ParticipantAddressKind::Lid {
        return None;
    }
    Jid::new(address.user(), crate::model::jid::Server::Lid).ok()
}

/// Parse one reply `user` node into `(digits, status)`. Defensive: any
/// unexpected shape is skipped rather than treated as an error.
fn parse_user(user: &Node, lid_succeeded: bool) -> Option<(String, ContactStatus)> {
    let contact = user.child("contact")?;
    // A wire round trip decodes non-token, non-packed content as raw bytes
    // (`binary/decoder.rs::read_content` never assumes UTF-8 for content, only
    // for attributes), even though it is a plain digit string, so both
    // representations must be accepted here.
    let text = match &contact.content {
        NodeContent::Text(s) => s.clone(),
        NodeContent::Bytes(b) => std::str::from_utf8(b).ok()?.to_owned(),
        _ => return None,
    };
    let (digits, _) = normalize(&text).ok()?;
    // APK 37.71 0u7.C8f 04f0..0570 requires exact case-sensitive type.
    // 0u3.A02 0100..0112/0232..0236 sets the account flag only for value 1.
    let (contact_type, error) = match contact.get_attr("type").and_then(|value| value.as_text()) {
        Some("in") => (Some(ContactType::In), None),
        Some("out") => (Some(ContactType::Out), None),
        Some("invalid") => (Some(ContactType::Invalid), None),
        None if contact.get_attr("type").is_none() => {
            (None, Some("missing_contact_type".to_owned()))
        }
        _ => (None, Some("unsupported_contact_type".to_owned())),
    };
    let jid = attr_jid(user, "jid");
    // APK 37.71 0u7.C8f 0e0c..0e42 checks A3p.A05 before reading
    // user/lid@val as 0ZY. Contact metadata never authorizes identity changes.
    let lid = lid_succeeded
        .then(|| user.child("lid").and_then(parse_lid))
        .flatten();
    Some((
        digits,
        ContactStatus {
            on_whatsapp: contact_type == Some(ContactType::In),
            contact_type,
            jid,
            lid,
            checked_at: Timestamp::now(),
            error,
        },
    ))
}

/// Fixed numeric parse failure in usync refusal metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsyncRefusalFailure {
    /// Present error code is not a Java signed integer.
    Code,
    /// Present backoff is not a Java signed long.
    Backoff,
}

/// APK43.72 usync refusal numeric metadata, not an effective wait policy.
/// Defaults/presence remain distinct; invalid input never creates a guessed wait.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsyncRefusal {
    /// Parsed signed code; absent defaults are zero for outer IQ, -1 for contact protocol.
    pub code: Option<i32>,
    /// Whether an actual code attribute was present.
    pub code_present: bool,
    /// Native signed wrapping seconds*1000, capped above at3600000.
    /// Absent wait is-1 for outer IQ, 3600000 for contact protocol.
    /// Malformed present data is null with failure.
    pub backoff_millis: Option<i64>,
    /// Whether an actual backoff attribute was present.
    pub backoff_present: bool,
    /// Fixed local parse diagnosis, with no raw error text or user data.
    pub failure: Option<UsyncRefusalFailure>,
}

pub(crate) fn outer_usync_refusal(request: &Node, reply: &Node) -> Option<UsyncRefusal> {
    if request.description != "iq"
        || request.get_attr("xmlns").and_then(|value| value.as_text()) != Some("usync")
        || request.get_attr("type").and_then(|value| value.as_text()) != Some("get")
        || request.child("usync").is_none()
        || reply.description != "iq"
        || reply.get_attr("type").and_then(|value| value.as_text()) != Some("error")
    {
        return None;
    }
    let error = reply.child("error");
    let code = error.and_then(|node| node.get_attr("code"));
    let backoff = error.and_then(|node| node.get_attr("backoff"));
    let mut result = UsyncRefusal {
        code: None,
        code_present: code.is_some(),
        backoff_millis: None,
        backoff_present: backoff.is_some(),
        failure: None,
    };
    result.code = if let Some(value) = code {
        value
            .as_text()
            .and_then(super::mex::java_decimal_long)
            .and_then(|value| i32::try_from(value).ok())
    } else {
        Some(0)
    };
    if result.code.is_none() {
        result.failure = Some(UsyncRefusalFailure::Code);
        return Some(result);
    }
    // Bvj's absent wait is-1, unlike A01's protocol-result default7200s.
    // Java mul-long wraps before Math.min; negative results are not a positive wait.
    result.backoff_millis = if let Some(value) = backoff {
        value
            .as_text()
            .and_then(super::mex::java_decimal_long)
            .map(|seconds| seconds.wrapping_mul(1000).min(3_600_000))
    } else {
        Some(-1)
    };
    if result.backoff_millis.is_none() {
        result.failure = Some(UsyncRefusalFailure::Backoff);
    }
    Some(result)
}

// 0pD.CK4 -> A01: selected contact protocol error in an outer result.
// This only retains numeric metadata; integrity/adoption and observer waits are separate.
pub(crate) fn contact_protocol_refusal(request: &Node, reply: &Node) -> Option<UsyncRefusal> {
    if request.description != "iq"
        || request.get_attr("xmlns").and_then(|v| v.as_text()) != Some("usync")
        || request.get_attr("type").and_then(|v| v.as_text()) != Some("get")
        || request
            .child("usync")?
            .child("query")?
            .child("contact")
            .is_none()
        || reply.description != "iq"
        || reply.get_attr("type").and_then(|v| v.as_text()) != Some("result")
    {
        return None;
    }
    let error = reply
        .child("usync")?
        .child("result")?
        .child("contact")?
        .child("error")?;
    let code = error.get_attr("code");
    let backoff = error.get_attr("backoff");
    let mut result = UsyncRefusal {
        code: None,
        code_present: code.is_some(),
        backoff_millis: None,
        backoff_present: backoff.is_some(),
        failure: None,
    };
    // A01 parses backoff before code: absent7200 seconds, signed Java wrap then cap.
    result.backoff_millis = if let Some(value) = backoff {
        value
            .as_text()
            .and_then(super::mex::java_decimal_long)
            .map(|seconds| seconds.wrapping_mul(1000).min(3_600_000))
    } else {
        Some(3_600_000)
    };
    if result.backoff_millis.is_none() {
        result.failure = Some(UsyncRefusalFailure::Backoff);
        return Some(result);
    }
    result.code = if let Some(value) = code {
        value
            .as_text()
            .and_then(super::mex::java_decimal_long)
            .and_then(|value| i32::try_from(value).ok())
    } else {
        Some(-1)
    };
    if result.code.is_none() {
        result.failure = Some(UsyncRefusalFailure::Code);
    }
    Some(result)
}

/// Local projection result, independent of the server IQ's outer outcome.
/// A refused/unsupported projection is not proof of account absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactProjection {
    /// The uniquely matching contact row was projected.
    Applied,
    /// The outer IQ refusal was attributed to its original requested number.
    ServerRefused,
    /// Historical action has no saved original-request correlation.
    MissingRequestCorrelation,
    /// Expected usync/list structure is missing or ambiguous.
    MalformedReply,
    /// No parseable returned user matched the requested number.
    NoMatchingUser,
    /// More than one returned user matched the single requested number.
    AmbiguousUser,
}

fn select_contact_reply(
    reply: Option<&Node>,
    expected: &str,
) -> std::result::Result<ContactStatus, ContactProjection> {
    let reply = reply.ok_or(ContactProjection::MalformedReply)?;
    let mut syncs = reply
        .child_nodes()
        .iter()
        .filter(|node| node.description == "usync");
    let usync = syncs.next().ok_or(ContactProjection::MalformedReply)?;
    if syncs.next().is_some() {
        return Err(ContactProjection::MalformedReply);
    }
    let mut lists = usync
        .child_nodes()
        .iter()
        .filter(|node| node.description == "list");
    let list = lists.next().ok_or(ContactProjection::MalformedReply)?;
    if lists.next().is_some() {
        return Err(ContactProjection::MalformedReply);
    }
    let mut selected = None;
    for user in list
        .child_nodes()
        .iter()
        .filter(|node| node.description == "user")
    {
        if let Some((digits, status)) = parse_user(user, lid_protocol_succeeded(usync))
            && digits == expected
        {
            if selected.is_some() {
                return Err(ContactProjection::AmbiguousUser);
            }
            selected = Some(status);
        }
    }
    selected.ok_or(ContactProjection::NoMatchingUser)
}

/// Ingest every settled `contact-check` action not yet processed, saving a
/// [`ContactStatus`] only for its uniquely matched original number. Idempotent: a
/// `contacts-ingested:<action id>` marker skips actions already ingested.
/// Returns the number of status rows newly recorded.
pub fn ingest(store: &ProtocolStore) -> Result<usize> {
    let mut recorded = 0usize;
    for (listed, _) in super::actions::settled_for_kinds(store, &[ACTION_KIND])? {
        recorded += store.write(|tx| ingest_in(tx, &listed))?;
    }
    Ok(recorded)
}

fn ingest_in(
    tx: &mut crate::store::WriteTx<'_>,
    listed: &super::actions::ActionOutcome,
) -> Result<usize> {
    let marker = ingested_key(&listed.id);
    if tx
        .get_json::<bool>(Table::Checkpoints, marker.as_bytes(), 1)?
        .is_some()
    {
        return Ok(0usize);
    }
    // Enumeration supplies order only. Projection authority is current
    // action/request/reply state held by this write transaction.
    let (outcome, reply, request) = super::actions::view_in_write(tx, &listed.id)?
        .ok_or_else(|| invalid("settled contact action is missing"))?;
    if outcome.kind != ACTION_KIND
        || !matches!(
            outcome.status,
            super::actions::ActionStatus::Ok | super::actions::ActionStatus::Error
        )
        || outcome.settlement_sequence != listed.settlement_sequence
    {
        return Err(invalid("contact settlement changed before projection"));
    }
    let response = reply
        .as_ref()
        .ok_or_else(|| invalid("contact projection reply is missing"))?;
    let response_type = response.get_attr("type").and_then(|v| v.as_text());
    if response.description != "iq"
        || response.get_attr("id").and_then(|v| v.as_text()) != Some(listed.id.as_str())
        || !matches!(response_type, Some("result" | "error"))
        || (outcome.status == super::actions::ActionStatus::Ok && response_type != Some("result"))
    {
        return Err(invalid("contact projection reply is not correlated"));
    }
    let mut count = 0usize;
    let projection = if !check_correlation_in(tx, &outcome.id, &request)? {
        ContactProjection::MissingRequestCorrelation
    } else {
        // The preflight also validates canonical stored digits against
        // the frozen single-user request. Never derive the target from
        // an unsolicited result row or from the current address book.
        let digits: String = tx
            .get_json(
                Table::Checkpoints,
                check_request_key(&outcome.id).as_bytes(),
                1,
            )?
            .ok_or_else(|| invalid("contact correlation disappeared"))?;
        match outcome.status {
            super::actions::ActionStatus::Ok => {
                match select_contact_reply(reply.as_ref(), &digits) {
                    Ok(status) => {
                        tx.put_json(
                            Table::Checkpoints,
                            status_key(&digits).as_bytes(),
                            1,
                            &status,
                        )?;
                        count = 1;
                        ContactProjection::Applied
                    }
                    Err(reason) => reason,
                }
            }
            super::actions::ActionStatus::Error => {
                let error = outcome
                    .error_text
                    .clone()
                    .or_else(|| outcome.error_code.map(|code| code.to_string()))
                    .unwrap_or_else(|| "refused".to_owned());
                tx.put_json(
                    Table::Checkpoints,
                    status_key(&digits).as_bytes(),
                    1,
                    &ContactStatus {
                        on_whatsapp: false,
                        contact_type: None,
                        jid: None,
                        lid: None,
                        checked_at: Timestamp::now(),
                        error: Some(error),
                    },
                )?;
                count = 1;
                ContactProjection::ServerRefused
            }
            _ => return Err(invalid("contact projection requires a settled action")),
        }
    };
    super::actions::record_contact_projection_in(tx, &outcome.id, projection)?;
    tx.put_json(Table::Checkpoints, marker.as_bytes(), 1, &true)?;
    Ok(count)
}

/// Repair only the missing LID of an already-ingested, uniquely attributable
/// successful contact check. Retains the prior row; never queries or changes pins.
/// This is a local migration for the former AD_JID-zero rejection, not APK behavior.
pub fn repair_lid(store: &ProtocolStore, id: &str) -> Result<bool> {
    let refuse = |reason: &str| Error::invalid("contacts.lid_repair", reason);
    store.write(|tx| {
        // Read all authority under the same transaction as the mapping update.
        // A previously decoded reply cannot authorize a changed saved request.
        let (selected, reply, request) =
            super::actions::view_in_write(tx, id)?.ok_or_else(|| refuse("saved_reply_missing"))?;
        let reply = reply.ok_or_else(|| refuse("saved_reply_missing"))?;
        if selected.kind != ACTION_KIND {
            return Err(refuse("action_missing_or_not_contact_check"));
        }
        if selected.status != super::actions::ActionStatus::Ok {
            return Err(refuse("action_not_successful"));
        }
        if !check_correlation_in(tx, id, &request)? {
            return Err(refuse("original_request_phone_missing"));
        }
        if reply.description != "iq"
            || reply.get_attr("id").and_then(|v| v.as_text()) != Some(id)
            || reply.get_attr("type").and_then(|v| v.as_text()) != Some("result")
        {
            return Err(refuse("reply_not_correlated_result"));
        }
        let outcomes = super::actions::outcomes_in_write(tx)?;

        let digits: String = tx
            .get_json(Table::Checkpoints, check_request_key(id).as_bytes(), 1)?
            .ok_or_else(|| refuse("original_request_phone_missing"))?;
        // No timestamp ordering guess or deletion of consumed markers. Even an
        // unsent second check makes attribution ambiguous for this bounded repair.
        for other in &outcomes {
            if other.kind == ACTION_KIND && other.id != id {
                let other_phone: Option<String> = tx.get_json(
                    Table::Checkpoints,
                    check_request_key(&other.id).as_bytes(),
                    1,
                )?;
                if other_phone.as_deref().is_none_or(|phone| phone == digits) {
                    return Err(refuse("ambiguous_contact_history"));
                }
            }
        }
        if tx.get_json::<bool>(Table::Checkpoints, ingested_key(id).as_bytes(), 1)? != Some(true) {
            return Err(refuse("ingestion_marker_missing"));
        }
        let parsed = select_contact_reply(Some(&reply), &digits)
            .map_err(|_| refuse("matching_user_ambiguous_or_malformed"))?;
        let lid = parsed
            .lid
            .clone()
            .ok_or_else(|| refuse("valid_lid_missing"))?;
        let key = status_key(&digits);
        let mut current: ContactStatus = tx
            .get_json(Table::Checkpoints, key.as_bytes(), 1)?
            .ok_or_else(|| refuse("saved_status_missing"))?;
        if current.on_whatsapp != parsed.on_whatsapp
            || current.contact_type != parsed.contact_type
            || current.jid != parsed.jid
            || current.error != parsed.error
        {
            return Err(refuse("saved_status_mismatch"));
        }
        if let Some(existing) = &current.lid {
            return if existing == &lid {
                Ok(false)
            } else {
                Err(refuse("existing_lid_conflict"))
            };
        }
        let history = format!("contact-status-lid-repair:{id}");
        if tx
            .get_json::<ContactStatus>(Table::Checkpoints, history.as_bytes(), 1)?
            .is_some()
        {
            return Err(refuse("repair_history_already_exists"));
        }
        tx.put_json(Table::Checkpoints, history.as_bytes(), 1, &current)?;
        current.lid = Some(lid);
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &current)?;
        Ok(true)
    })
}

/// The saved existence status for one number, if a check ever settled.
pub fn status(store: &ProtocolStore, phone: &str) -> Result<Option<ContactStatus>> {
    let (digits, _) = normalize(phone)?;
    store.read(|tx| tx.get_json(Table::Checkpoints, status_key(&digits).as_bytes(), 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use std::sync::Arc;

    fn attr_text<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
        node.get_attr(name).and_then(|v| v.as_text())
    }

    fn store() -> (tempfile::TempDir, ProtocolStore) {
        let dir = tempfile::tempdir().unwrap();
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("c.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, s)
    }

    fn protocol_refusal_request() -> Node {
        Node::new("iq")
            .attr("xmlns", "usync")
            .attr("type", "get")
            .children(vec![Node::new("usync").children(vec![
                Node::new("query").children(vec![Node::new("contact")]),
            ])])
    }

    fn protocol_refusal_reply(error: Node) -> Node {
        Node::new("iq")
            .attr("type", "result")
            .children(vec![Node::new("usync").children(vec![
                Node::new("result").children(vec![Node::new("contact").children(vec![error])]),
                Node::new("list"),
            ])])
    }

    #[test]
    fn contact_protocol_refusal_numbers_match_independent_jvm_oracle() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/apk/2.26.43.72-contact-protocol-refusal/numeric-vectors.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let mut error = Node::new("error");
            if let Some(value) = case["code_text"].as_str() {
                error = error.attr("code", value);
            }
            if let Some(value) = case["backoff_text"].as_str() {
                error = error.attr("backoff", value);
            }
            let parsed = contact_protocol_refusal(
                &protocol_refusal_request(),
                &protocol_refusal_reply(error),
            )
            .unwrap();
            assert_eq!(serde_json::to_value(parsed.code).unwrap(), case["code"]);
            assert_eq!(
                serde_json::to_value(parsed.backoff_millis).unwrap(),
                case["backoff_millis"]
            );
            assert_eq!(
                serde_json::to_value(parsed.failure).unwrap(),
                case["failure"]
            );
            assert_eq!(parsed.code_present, !case["code_text"].is_null());
            assert_eq!(parsed.backoff_present, !case["backoff_text"].is_null());
        }
    }

    #[test]
    fn contact_protocol_refusal_preserves_scope_first_child_and_redaction() {
        let request = protocol_refusal_request();
        let reply = protocol_refusal_reply(Node::new("error").attr("code", "429"));
        let metadata = contact_protocol_refusal(&request, &reply).unwrap();
        assert_eq!(
            (metadata.code, metadata.backoff_millis),
            (Some(429), Some(3_600_000))
        );
        assert!(outer_usync_refusal(&request, &reply).is_none());
        assert!(contact_protocol_refusal(&Node::new("iq"), &reply).is_none());
        assert!(
            contact_protocol_refusal(&request, &Node::new("iq").attr("type", "error")).is_none()
        );
        let no_error = protocol_refusal_reply(Node::new("refresh").attr("backoff", "99"));
        assert!(contact_protocol_refusal(&request, &no_error).is_none());
        let first =
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("usync").children(vec![
                    Node::new("result").children(vec![Node::new("contact").children(vec![
                        Node::new("error")
                            .attr("code", "503")
                            .attr("backoff", "2")
                            .attr("text", "private-refusal"),
                        Node::new("error")
                            .attr("code", "429")
                            .attr("backoff", "bad"),
                    ])]),
                ])]);
        let metadata = contact_protocol_refusal(&request, &first).unwrap();
        assert_eq!(
            (metadata.code, metadata.backoff_millis),
            (Some(503), Some(2000))
        );
        assert!(
            !serde_json::to_string(&metadata)
                .unwrap()
                .contains("private-refusal")
        );
        let nontext = protocol_refusal_reply(
            Node::new("error").attr("backoff", Jid::parse("15551234567@s.whatsapp.net").unwrap()),
        );
        let metadata = contact_protocol_refusal(&request, &nontext).unwrap();
        assert_eq!(metadata.failure, Some(UsyncRefusalFailure::Backoff));
        assert_eq!(metadata.code, None);
    }

    #[test]
    fn contact_protocol_refusal_survives_settlement_projection_and_reopen() {
        use super::super::actions;
        let (dir, store) = store();
        let id = queue_check(&store, "+15551234567").unwrap();
        actions::reserve_pending(&store).unwrap();
        let reply =
            protocol_refusal_reply(Node::new("error").attr("code", "429")).attr("id", id.as_str());
        assert!(actions::record_reply(&store, &reply).unwrap());
        let action = actions::outcomes(&store).unwrap().pop().unwrap();
        assert_eq!(action.status, actions::ActionStatus::Ok);
        assert_eq!(action.error_code, None);
        assert_eq!(action.usync_refusal, None);
        let metadata = action.contact_protocol_refusal.unwrap();
        assert_eq!(
            (metadata.code, metadata.backoff_millis),
            (Some(429), Some(3_600_000))
        );
        assert_eq!(ingest(&store).unwrap(), 0);
        assert_eq!(
            actions::outcomes(&store).unwrap()[0].contact_projection,
            Some(ContactProjection::NoMatchingUser)
        );
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(!actions::record_reply(&store, &reply).unwrap());
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
        );
        drop(store);
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("c.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(
            actions::outcomes(&store).unwrap()[0].contact_protocol_refusal,
            Some(metadata)
        );
        assert_eq!(ingest(&store).unwrap(), 0);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
    }

    #[test]
    fn usync_outer_refusal_numbers_match_independent_jvm_oracle() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/apk/2.26.43.72-usync-outer-refusal/numeric-vectors.json"
        ))
        .unwrap();
        let request = Node::new("iq")
            .attr("xmlns", "usync")
            .attr("type", "get")
            .children(vec![Node::new("usync")]);
        for case in fixture["cases"].as_array().unwrap() {
            let mut error = Node::new("error");
            if let Some(value) = case["code_text"].as_str() {
                error = error.attr("code", value);
            }
            if let Some(value) = case["backoff_text"].as_str() {
                error = error.attr("backoff", value);
            }
            let reply = Node::new("iq").attr("type", "error").children(vec![error]);
            let parsed = outer_usync_refusal(&request, &reply).unwrap();
            assert_eq!(serde_json::to_value(parsed.code).unwrap(), case["code"]);
            assert_eq!(
                serde_json::to_value(parsed.backoff_millis).unwrap(),
                case["backoff_millis"]
            );
            assert_eq!(
                serde_json::to_value(parsed.failure).unwrap(),
                case["failure"]
            );
            assert_eq!(parsed.code_present, !case["code_text"].is_null());
            assert_eq!(parsed.backoff_present, !case["backoff_text"].is_null());
        }
    }

    #[test]
    fn usync_outer_refusal_keeps_missing_first_child_and_protocol_scope_distinct() {
        let request = Node::new("iq")
            .attr("xmlns", "usync")
            .attr("type", "get")
            .children(vec![Node::new("usync")]);
        let missing = Node::new("iq").attr("type", "error");
        let parsed = outer_usync_refusal(&request, &missing).unwrap();
        assert_eq!((parsed.code, parsed.backoff_millis), (Some(0), Some(-1)));
        assert!(!parsed.code_present && !parsed.backoff_present);
        let reply = Node::new("iq").attr("type", "error").children(vec![
            Node::new("error")
                .attr("code", "503")
                .attr("backoff", "2")
                .attr("text", "private-error"),
            Node::new("error")
                .attr("code", "bad")
                .attr("backoff", "999999"),
        ]);
        let parsed = outer_usync_refusal(&request, &reply).unwrap();
        assert_eq!(
            (parsed.code, parsed.backoff_millis),
            (Some(503), Some(2000))
        );
        assert!(
            !serde_json::to_string(&parsed)
                .unwrap()
                .contains("private-error")
        );
        assert!(outer_usync_refusal(&Node::new("iq").attr("type", "get"), &reply).is_none());
        assert!(
            outer_usync_refusal(
                &request,
                &Node::new("iq").attr("type", "result").children(vec![
                    Node::new("usync").children(vec![Node::new("result").children(vec![
                        Node::new("contact").children(vec![Node::new("error").attr("code", "503")])
                    ])])
                ])
            )
            .is_none()
        );
        let nontext = Node::new("iq").attr("type", "error").children(vec![
            Node::new("error").attr("code", Jid::parse("15551234567@s.whatsapp.net").unwrap()),
        ]);
        let parsed = outer_usync_refusal(&request, &nontext).unwrap();
        assert_eq!(parsed.failure, Some(UsyncRefusalFailure::Code));
        assert_eq!(parsed.backoff_millis, None);
        assert!(
            !serde_json::to_string(&parsed)
                .unwrap()
                .contains("15551234567")
        );
    }

    #[test]
    fn usync_outer_refusal_is_durable_through_projection_and_cannot_resettle() {
        use super::super::actions;
        let (dir, store) = store();
        let id = queue_check(&store, "+15551234567").unwrap();
        actions::reserve_pending(&store).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "error")
            .children(vec![
                Node::new("error").attr("code", "503").attr("backoff", "30"),
            ]);
        assert!(actions::record_reply(&store, &reply).unwrap());
        let metadata = actions::outcomes(&store).unwrap()[0]
            .usync_refusal
            .clone()
            .unwrap();
        assert_eq!(
            (metadata.code, metadata.backoff_millis, metadata.failure),
            (Some(503), Some(30000), None)
        );
        assert_eq!(ingest(&store).unwrap(), 1);
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(!actions::record_reply(&store, &reply).unwrap());
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
        );
        drop(store);
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("c.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
            create: false,
        })
        .unwrap();
        let action = actions::outcomes(&store).unwrap().pop().unwrap();
        assert_eq!(action.status, actions::ActionStatus::Error);
        assert_eq!(action.usync_refusal, Some(metadata));
        assert_eq!(
            action.contact_projection,
            Some(ContactProjection::ServerRefused)
        );
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        assert_eq!(ingest(&store).unwrap(), 0);
    }

    #[test]
    fn contact_projection_is_request_bound_unambiguous_and_once_only_after_reopen() {
        use super::super::actions;
        for case in [
            "matching_with_unrelated",
            "duplicate",
            "missing_origin",
            "malformed",
        ] {
            let (dir, store) = store();
            let id = queue_check(&store, "+15551234567").unwrap();
            let request = actions::reserve_pending(&store).unwrap().pop().unwrap();
            let target = user_node("+15551234567", Some("15551234567@s.whatsapp.net"), None);
            let users = if case == "duplicate" {
                vec![target.clone(), target]
            } else {
                vec![user_node("+15559999999", None, None), target]
            };
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(if case == "malformed" {
                    vec![]
                } else {
                    vec![Node::new("usync").children(vec![Node::new("list").children(users)])]
                });
            actions::record_reply(&store, &reply).unwrap();
            if case == "missing_origin" {
                store
                    .write(|tx| tx.delete(Table::Checkpoints, check_request_key(&id).as_bytes()))
                    .unwrap();
            }
            let expected = match case {
                "duplicate" => ContactProjection::AmbiguousUser,
                "missing_origin" => ContactProjection::MissingRequestCorrelation,
                "malformed" => ContactProjection::MalformedReply,
                _ => ContactProjection::Applied,
            };
            assert_eq!(
                ingest(&store).unwrap(),
                usize::from(case == "matching_with_unrelated")
            );
            assert!(status(&store, "+15559999999").unwrap().is_none());
            assert_eq!(
                status(&store, "+15551234567").unwrap().is_some(),
                case == "matching_with_unrelated"
            );
            assert_eq!(
                actions::outcomes(&store).unwrap()[0].contact_projection,
                Some(expected)
            );
            assert_eq!(
                store
                    .write(|tx| actions::view_in_write(tx, &id))
                    .unwrap()
                    .unwrap()
                    .2,
                request
            );
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            drop(store);
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("c.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
                create: false,
            })
            .unwrap();
            assert_eq!(ingest(&store).unwrap(), 0);
            assert_eq!(
                before,
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap()
            );
            assert_eq!(
                actions::outcomes(&store).unwrap()[0].contact_projection,
                Some(expected)
            );
        }
    }

    #[test]
    fn contact_projection_status_diagnostic_and_marker_are_atomic_under_disk_faults() {
        use super::super::actions;
        use crate::store::full_disk;
        for fsync_only in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("contacts.db");
            let faults = full_disk::Faults::new();
            let store = full_disk::open(&path, "contact-projection", [18; 32], &faults).unwrap();
            let id = queue_check(&store, "+15551234567").unwrap();
            actions::reserve_pending(&store).unwrap();
            actions::record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .children(vec![Node::new("usync").children(vec![
                        Node::new("list").children(vec![user_node(
                            "+15551234567",
                            Some("15551234567@s.whatsapp.net"),
                            None,
                        )]),
                    ])]),
            )
            .unwrap();
            faults.arm_on(INGESTED_PREFIX.as_bytes(), fsync_only);
            assert!(ingest(&store).is_err());
            drop(store);
            faults.set_full(false);
            faults.set_sync_fails(false);
            let store = full_disk::open(&path, "contact-projection", [18; 32], &faults).unwrap();
            let saved = status(&store, "+15551234567").unwrap();
            let marker = store
                .read(|tx| tx.get_json::<bool>(Table::Checkpoints, ingested_key(&id).as_bytes(), 1))
                .unwrap();
            let action = actions::outcomes(&store).unwrap().pop().unwrap();
            assert_eq!(action.status, actions::ActionStatus::Ok);
            assert_eq!(saved.is_some(), marker == Some(true));
            assert_eq!(
                action.contact_projection,
                saved.as_ref().map(|_| ContactProjection::Applied)
            );
            if !fsync_only {
                assert!(saved.is_none());
            }
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
            // Local recovery reprojects the retained reply once, without a query.
            assert_eq!(ingest(&store).unwrap(), usize::from(saved.is_none()));
            assert_eq!(ingest(&store).unwrap(), 0);
        }
    }

    #[test]
    fn single_contact_reply_never_overwrites_an_unrequested_contact() {
        let (_dir, store) = store();
        let other = queue_check(&store, "+15559999999").unwrap();
        super::super::actions::reserve_pending(&store).unwrap();
        let response = |id: &str, phone: &str, registered| {
            Node::new("iq")
                .attr("id", id)
                .attr("type", "result")
                .children(vec![Node::new("usync").children(vec![
                    Node::new("list").children(vec![user_node(
                        phone,
                        if registered {
                            Some("15559999999@s.whatsapp.net")
                        } else {
                            None
                        },
                        None,
                    )]),
                ])])
        };
        super::super::actions::record_reply(&store, &response(&other, "+15559999999", true))
            .unwrap();
        assert_eq!(ingest(&store).unwrap(), 1);
        let previous = status(&store, "+15559999999").unwrap().unwrap();
        let selected = queue_check(&store, "+15551234567").unwrap();
        super::super::actions::reserve_pending(&store).unwrap();
        super::super::actions::record_reply(&store, &response(&selected, "+15559999999", false))
            .unwrap();
        assert_eq!(ingest(&store).unwrap(), 0);
        assert_eq!(status(&store, "+15559999999").unwrap().unwrap(), previous);
        assert_eq!(
            super::super::actions::outcomes(&store)
                .unwrap()
                .into_iter()
                .find(|row| row.id == selected)
                .unwrap()
                .contact_projection,
            Some(ContactProjection::NoMatchingUser)
        );
        assert!(status(&store, "+15551234567").unwrap().is_none());
        assert_eq!(ingest(&store).unwrap(), 0);
    }

    #[test]
    fn missing_contact_correlation_holds_legacy_request_without_blocking_other_actions() {
        use super::super::actions;
        let (_dir, store) = store();
        let id = queue_check(&store, "+15551234567").unwrap();
        store
            .write(|tx| tx.delete(Table::Checkpoints, check_request_key(&id).as_bytes()))
            .unwrap();
        let other = actions::queue(&store, "control", "control", |id| {
            Ok(Node::new("iq").attr("id", id).attr("type", "get"))
        })
        .unwrap();
        let reserved = actions::reserve_pending(&store).unwrap();
        assert_eq!(reserved.len(), 1);
        assert_eq!(
            reserved[0].get_attr("id").and_then(|v| v.as_text()),
            Some(other.as_str())
        );
        let held = actions::outcomes(&store)
            .unwrap()
            .into_iter()
            .find(|row| row.id == id)
            .unwrap();
        assert_eq!(held.status, actions::ActionStatus::Queued);
        assert_eq!(
            held.deferred_reason,
            Some(actions::ActionDeferReason::ContactCheckCorrelationMissing)
        );
        assert!(store.write(|tx| actions::reserve_one_in(tx, &id)).is_err());
        // Restoring explicit original provenance permits one reservation and clears the hold.
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    check_request_key(&id).as_bytes(),
                    1,
                    &"15551234567",
                )
            })
            .unwrap();
        assert!(store.write(|tx| actions::reserve_one_in(tx, &id)).is_ok());
        let held = actions::outcomes(&store)
            .unwrap()
            .into_iter()
            .find(|row| row.id == id)
            .unwrap();
        assert_eq!(held.deferred_reason, None);
        assert_eq!(held.status, actions::ActionStatus::Sent);
    }

    #[test]
    fn mismatched_contact_correlation_refuses_reservation_without_partial_changes_or_echo() {
        use super::super::actions;
        let (_dir, store) = store();
        let id = queue_check(&store, "+15551234567").unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    check_request_key(&id).as_bytes(),
                    1,
                    &"15559999999",
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        for selected in [false, true] {
            let error = if selected {
                store
                    .write(|tx| actions::reserve_one_in(tx, &id))
                    .unwrap_err()
            } else {
                actions::reserve_pending(&store).unwrap_err()
            };
            let diagnostic = error.to_string();
            for private in [id.as_str(), "15551234567", "15559999999"] {
                assert!(!diagnostic.contains(private));
            }
            assert_eq!(
                before,
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap()
            );
        }
    }

    #[test]
    fn contact_request_and_error_correlation_commit_together_under_disk_faults() {
        use crate::store::full_disk;
        for fsync_only in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("contacts.db");
            let faults = full_disk::Faults::new();
            let store = full_disk::open(&path, "contact-atomic", [18; 32], &faults).unwrap();
            faults.arm_on(CHECK_REQUEST_PREFIX.as_bytes(), fsync_only);
            assert!(queue_check(&store, "+15551234567").is_err());
            drop(store);
            faults.set_full(false);
            faults.set_sync_fails(false);
            let store = full_disk::open(&path, "contact-atomic", [18; 32], &faults).unwrap();
            let actions = super::super::actions::outcomes(&store).unwrap();
            if !fsync_only {
                assert!(
                    actions.is_empty(),
                    "failed correlation commit left a sendable orphan"
                );
            }
            assert!(actions.len() <= 1);
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert_eq!(
                rows.iter()
                    .filter(|(key, _)| key.starts_with(CHECK_REQUEST_PREFIX.as_bytes()))
                    .count(),
                actions.len()
            );
            for action in actions {
                assert_eq!(action.status, super::super::actions::ActionStatus::Queued);
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<String>(
                            Table::Checkpoints,
                            check_request_key(&action.id).as_bytes(),
                            1
                        ))
                        .unwrap(),
                    Some("15551234567".to_owned())
                );
            }
        }
    }

    #[test]
    fn normalize_accepts_plus_and_bare_digits() {
        assert_eq!(normalize("+15551234567").unwrap().0, "15551234567");
        assert_eq!(normalize("15551234567").unwrap().1, "+15551234567");
        assert!(normalize("12345").is_err(), "too short");
        assert!(normalize("1234567890123456").is_err(), "too long");
        assert!(normalize("+1555-123-4567").is_err(), "no punctuation");
        assert!(normalize("abc").is_err());
    }

    #[test]
    fn crud_round_trip_and_rejects_duplicates() {
        let (_d, s) = store();
        let c = add(&s, "+15551234567", "Jamie Doe", Some("Jamie")).unwrap();
        assert_eq!(c.phone, "+15551234567");
        assert!(add(&s, "+1 555 123 4567".replace(' ', "").as_str(), "x", None).is_err());
        let listed = list(&s).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            get(&s, "+15551234567").unwrap().unwrap().full_name,
            "Jamie Doe"
        );
        let edited = edit(&s, "+15551234567", "Jamie D.", None).unwrap();
        assert_eq!(edited.full_name, "Jamie D.");
        assert!(edited.updated_at.0 >= edited.created_at.0);
        assert!(edit(&s, "+15559999999", "x", None).is_err());
        assert!(remove(&s, "+15551234567").unwrap());
        assert!(!remove(&s, "+15551234567").unwrap());
        assert!(list(&s).unwrap().is_empty());
    }

    fn user_node(phone: &str, jid: Option<&str>, lid: Option<&str>) -> Node {
        let mut children = vec![
            Node::new("contact")
                .attr("type", if jid.is_some() { "in" } else { "out" })
                .text(phone),
        ];
        if let Some(lid) = lid {
            children.push(Node::new("lid").attr("val", Jid::parse(lid).unwrap()));
        }
        let mut n = Node::new("user").children(children);
        if let Some(jid) = jid {
            n = n.attr("jid", Jid::parse(jid).unwrap());
        }
        n
    }

    /// A full sync must differ from the existence query only in mode/context,
    /// per `LX/0u7->A06`: same iq/usync/query/user structure, same per-user
    /// encoding. `allow_mutation` must not appear, since what enables it is
    /// unknown.
    #[test]
    fn full_sync_matches_the_query_envelope_but_for_mode_and_context() {
        let (_d, store) = store();
        // No contacts: refused rather than sending an empty address book.
        assert!(queue_full_sync(&store).is_err());

        add(&store, "+15551234567", "A", None).unwrap();
        add(&store, "+15557654321", "B", None).unwrap();
        let (id, count) = queue_full_sync(&store).unwrap();
        assert_eq!(count, 2);

        let sent = super::super::actions::reserve_pending(&store).unwrap();
        assert_eq!(sent.len(), 1);
        let node = &sent[0];
        assert_eq!(attr_text(node, "xmlns"), Some("usync"));
        assert_eq!(attr_text(node, "type"), Some("get"));
        assert_eq!(attr_text(node, "id"), Some(id.as_str()));
        let usync = node.child("usync").unwrap();
        assert_eq!(attr_text(usync, "mode"), Some("full"));
        assert_eq!(attr_text(usync, "context"), Some("interactive"));
        assert_eq!(attr_text(usync, "index"), Some("0"));
        assert_eq!(attr_text(usync, "last"), Some("true"));
        assert!(attr_text(usync, "sid").is_some_and(|s| !s.is_empty()));
        // allow_mutation is deliberately absent.
        assert!(usync.get_attr("allow_mutation").is_none());
        // Every saved contact appears once, in the same per-user shape.
        let users = usync.child("list").unwrap().child_nodes();
        assert_eq!(users.len(), 2);
        for user in users {
            assert_eq!(user.description, "user");
            let contact = user.child("contact").unwrap();
            // The queue round-trips a text child back as raw bytes; the
            // live-verified existence query uses the identical per-user
            // encoding, so this is the established representation.
            let bytes = match &contact.content {
                NodeContent::Bytes(b) => b.to_vec(),
                NodeContent::Text(t) => t.clone().into_bytes(),
                other => panic!("contact carried {other:?}"),
            };
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.starts_with('+'), "expected +E.164, got {text}");
        }
    }

    #[test]
    fn usync_query_has_the_documented_shape() {
        let (_d, s) = store();
        let id = queue_check(&s, "+15551234567").unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        assert_eq!(sent.len(), 1);
        let iq = &sent[0];
        assert_eq!(iq.description, "iq");
        assert_eq!(attr_text(iq, "xmlns"), Some("usync"));
        assert_eq!(attr_text(iq, "type"), Some("get"));
        assert_eq!(attr_text(iq, "id"), Some(id.as_str()));
        let usync = iq.child("usync").unwrap();
        assert!(attr_text(usync, "sid").is_some_and(|s| !s.is_empty()));
        assert_eq!(attr_text(usync, "index"), Some("0"));
        assert_eq!(attr_text(usync, "last"), Some("true"));
        assert_eq!(attr_text(usync, "mode"), Some("query"));
        assert_eq!(attr_text(usync, "context"), Some("interactive"));
        let query = usync.child("query").unwrap();
        let names: Vec<&str> = query
            .child_nodes()
            .iter()
            .map(|n| n.description.as_str())
            .collect();
        assert_eq!(names, vec!["contact", "lid", "disappearing_mode"]);
        let list = usync.child("list").unwrap();
        let user = &list.child_nodes()[0];
        assert_eq!(user.description, "user");
        let contact = user.child("contact").unwrap();
        // A round trip through the wire encoding decodes plain digit-and-`+`
        // content as raw bytes, not text (see `parse_user`'s doc comment).
        assert_eq!(
            contact.content,
            NodeContent::Bytes(bytes::Bytes::from_static(b"+15551234567"))
        );
    }

    #[test]
    fn replay_after_restart_preserves_receive_order_over_random_identifier_order() {
        let (dir, store) = store();
        let mut ids = [
            queue_check(&store, "+15551234567").unwrap(),
            queue_check(&store, "+15551234567").unwrap(),
        ];
        ids.sort();
        super::super::actions::reserve_pending(&store).unwrap();
        for (index, id) in ids.iter().enumerate() {
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("usync").children(vec![
                    Node::new("list").children(vec![user_node(
                        "+15551234567",
                        if index == 0 {
                            Some("15551234567@s.whatsapp.net")
                        } else {
                            None
                        },
                        None,
                    )]),
                ])]);
            super::super::actions::record_reply(&store, &reply).unwrap();
        }
        drop(store);
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("c.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(ingest(&store).unwrap(), 2);
        assert!(!status(&store, "+15551234567").unwrap().unwrap().on_whatsapp);
        assert_eq!(ingest(&store).unwrap(), 0);
    }

    #[test]
    fn ingest_records_on_and_off_whatsapp_and_is_idempotent() {
        let (_d, s) = store();
        let id = queue_check(&s, "+15551234567").unwrap();
        assert!(super::super::actions::reserve_pending(&s).unwrap().len() == 1);
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("usync").attr("sid", "x").children(vec![
                Node::new("result").children(vec![Node::new("lid")]),
                Node::new("list").children(vec![user_node(
                    "+15551234567",
                    Some("15551234567@s.whatsapp.net"),
                    Some("999999999999@lid"),
                )]),
            ])]);
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        assert_eq!(ingest(&s).unwrap(), 0, "idempotent on a second pass");
        let saved = status(&s, "+15551234567").unwrap().unwrap();
        assert!(saved.on_whatsapp);
        assert_eq!(saved.contact_type, Some(ContactType::In));
        assert_eq!(
            saved.jid,
            Some(Jid::parse("15551234567@s.whatsapp.net").unwrap())
        );
        assert_eq!(saved.lid, Some(Jid::parse("999999999999@lid").unwrap()));

        let id2 = queue_check(&s, "+15559999999").unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply2 = Node::new("iq")
            .attr("id", id2.as_str())
            .attr("type", "result")
            .children(vec![Node::new("usync").attr("sid", "y").children(vec![
                Node::new("list").children(vec![user_node("+15559999999", None, None)]),
            ])]);
        assert!(super::super::actions::record_reply(&s, &reply2).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        let saved2 = status(&s, "+15559999999").unwrap().unwrap();
        assert!(!saved2.on_whatsapp);
        assert_eq!(saved2.contact_type, Some(ContactType::Out));
        assert!(saved2.jid.is_none());
    }

    #[test]
    fn apk_lid_val_is_typed_and_gated_by_protocol_success() {
        // Literal APK attribute, independent of the old fixture helper's jid.
        let user = Node::new("user")
            .attr("jid", "15551234567@s.whatsapp.net")
            .children(vec![
                Node::new("contact").text("+15551234567"),
                Node::new("lid").attr("val", "999999999999@lid"),
            ]);
        assert_eq!(
            parse_user(&user, true).unwrap().1.lid,
            Some(Jid::parse("999999999999@lid").unwrap())
        );
        assert!(parse_user(&user, false).unwrap().1.lid.is_none());
        for invalid in [
            "999:0@lid",
            "999:1@lid",
            "999.0@lid",
            "999@s.whatsapp.net",
            "999@bot",
            "0@lid",
            "0999@lid",
            "1234567890123456@lid",
            "abc@lid",
        ] {
            assert!(
                parse_lid(&Node::new("lid").attr("val", invalid)).is_none(),
                "{invalid}"
            );
        }
        assert!(parse_lid(&Node::new("lid").attr("jid", "999@lid")).is_none());
        assert_eq!(
            parse_lid(&Node::new("lid").attr("val", "999@hosted.lid")),
            Some(Jid::parse("999@lid").unwrap())
        );
        assert_eq!(
            parse_lid(&Node::new("lid").attr(
                "val",
                Jid::new_ad("999", 0, 0, crate::model::jid::Server::Lid).unwrap()
            )),
            Some(Jid::parse("999@lid").unwrap())
        );
        assert!(
            parse_lid(&Node::new("lid").attr(
                "val",
                Jid::new_ad("999", 0, 1, crate::model::jid::Server::Lid).unwrap()
            ))
            .is_none()
        );
        let metadata =
            |lid| Node::new("usync").children(vec![Node::new("result").children(vec![lid])]);
        assert!(!lid_protocol_succeeded(&Node::new("usync")));
        assert!(lid_protocol_succeeded(&metadata(Node::new("lid"))));
        assert!(!lid_protocol_succeeded(&metadata(
            Node::new("lid").children(vec![Node::new("error").attr("code", "500")])
        )));
        assert!(lid_protocol_succeeded(&metadata(
            Node::new("lid").attr("refresh", "+60")
        )));
        assert!(!lid_protocol_succeeded(&metadata(
            Node::new("lid").attr("refresh", "invalid")
        )));
        assert!(!lid_protocol_succeeded(&metadata(
            Node::new("lid").attr("refresh", "9223372036854775808")
        )));
    }

    #[test]
    fn ingest_keeps_contact_status_but_discards_lid_when_protocol_failed_or_missing() {
        let (_d, s) = store();
        for protocol_result in [
            None,
            Some(Node::new("lid").children(vec![Node::new("error").attr("code", "500")])),
        ] {
            let id = queue_check(&s, "+15551234567").unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let mut children = vec![Node::new("list").children(vec![user_node(
                "+15551234567",
                Some("15551234567@s.whatsapp.net"),
                Some("999999999999@lid"),
            )])];
            if let Some(result) = protocol_result {
                children.push(Node::new("result").children(vec![result]));
            }
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("usync").children(children)]);
            assert!(super::super::actions::record_reply(&s, &reply).unwrap());
            assert_eq!(ingest(&s).unwrap(), 1);
            let saved = status(&s, "+15551234567").unwrap().unwrap();
            assert!(saved.on_whatsapp);
            assert!(saved.lid.is_none());
            assert_eq!(ingest(&s).unwrap(), 0);
        }
    }

    #[test]
    fn contact_type_overrides_jid_presence_and_preserves_invalid() {
        for (wire_type, expected) in [
            ("in", ContactType::In),
            ("out", ContactType::Out),
            ("invalid", ContactType::Invalid),
        ] {
            for jid in [None, Some("15551234567@s.whatsapp.net")] {
                let mut user = Node::new("user").children(vec![
                    Node::new("contact")
                        .attr("type", wire_type)
                        .text("+15551234567"),
                ]);
                if let Some(jid) = jid {
                    user = user.attr("jid", jid);
                }
                let status = parse_user(&user, false).unwrap().1;
                assert_eq!(status.contact_type, Some(expected));
                assert_eq!(status.on_whatsapp, expected == ContactType::In);
                assert!(status.error.is_none());
            }
        }
        for (wire_type, reason) in [
            (None, "missing_contact_type"),
            (Some("IN"), "unsupported_contact_type"),
            (
                Some("private-unrecognized-value"),
                "unsupported_contact_type",
            ),
        ] {
            let mut contact = Node::new("contact").text("+15551234567");
            if let Some(t) = wire_type {
                contact = contact.attr("type", t);
            }
            let user = Node::new("user")
                .attr("jid", "15551234567@s.whatsapp.net")
                .children(vec![contact]);
            let status = parse_user(&user, false).unwrap().1;
            assert!(!status.on_whatsapp);
            assert!(status.contact_type.is_none());
            assert_eq!(status.error.as_deref(), Some(reason));
            assert!(
                !serde_json::to_string(&status)
                    .unwrap()
                    .contains("private-unrecognized-value")
            );
        }
    }

    #[test]
    fn invalid_status_survives_reopen_and_inspection_never_queues() {
        let (d, s) = store();
        assert!(status(&s, "+15551234567").unwrap().is_none());
        assert!(super::super::actions::outcomes(&s).unwrap().is_empty());
        let id = queue_check(&s, "+15551234567").unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("usync").children(vec![
                Node::new("list").children(
                    vec![Node::new("user").attr("jid", "15551234567@s.whatsapp.net")
                    .children(vec![Node::new("contact").attr("type", "invalid")
                        .text("+15551234567")])],
                ),
            ])]);
        super::super::actions::record_reply(&s, &reply).unwrap();
        assert_eq!(ingest(&s).unwrap(), 1);
        drop(s);
        let s = ProtocolStore::open(OpenOptions {
            path: d.path().join("c.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
            create: false,
        })
        .unwrap();
        let before = serde_json::to_value(super::super::actions::outcomes(&s).unwrap()).unwrap();
        for _ in 0..2 {
            let saved = status(&s, "+15551234567").unwrap().unwrap();
            assert_eq!(saved.contact_type, Some(ContactType::Invalid));
            assert!(!saved.on_whatsapp);
        }
        assert_eq!(ingest(&s).unwrap(), 0);
        assert_eq!(
            before,
            serde_json::to_value(super::super::actions::outcomes(&s).unwrap()).unwrap()
        );
    }

    #[test]
    fn missing_primary_wire_lid_repairs_once_with_history_and_ambiguity_guard() {
        let (_d, s) = store();
        let phone = "+15551234567";
        let id = queue_check(&s, phone).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let user = Node::new("user")
            .attr("jid", "15551234567@s.whatsapp.net")
            .children(vec![
                Node::new("contact").attr("type", "in").text(phone),
                Node::new("lid").attr(
                    "val",
                    Jid::new_ad("999999999999", 0, 0, crate::model::jid::Server::Lid).unwrap(),
                ),
            ]);
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("usync").children(vec![
                Node::new("result").children(vec![Node::new("lid")]),
                Node::new("list").children(vec![user]),
            ])]);
        super::super::actions::record_reply(&s, &reply).unwrap();
        assert_eq!(ingest(&s).unwrap(), 1);
        let expected = Jid::parse("999999999999@lid").unwrap();
        assert_eq!(
            status(&s, phone).unwrap().unwrap().lid,
            Some(expected.clone()),
            "wire round-trip AD_JID zero reaches ordinary ingestion"
        );
        // Emulate the prior consumed projection without deleting its marker.
        let mut old = status(&s, phone).unwrap().unwrap();
        old.lid = None;
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                status_key("15551234567").as_bytes(),
                1,
                &old,
            )
        })
        .unwrap();
        let before = serde_json::to_value(super::super::actions::outcomes(&s).unwrap()).unwrap();
        assert!(repair_lid(&s, &id).unwrap());
        assert!(!repair_lid(&s, &id).unwrap());
        let updated = status(&s, phone).unwrap().unwrap();
        assert_eq!(updated.lid, Some(expected));
        assert_eq!(updated.checked_at, old.checked_at);
        let history: ContactStatus = s
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("contact-status-lid-repair:{id}").as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(history, old);
        assert_eq!(
            before,
            serde_json::to_value(super::super::actions::outcomes(&s).unwrap()).unwrap()
        );
        let mut conflict = updated.clone();
        conflict.lid = Some(Jid::parse("888888888888@lid").unwrap());
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                status_key("15551234567").as_bytes(),
                1,
                &conflict,
            )
        })
        .unwrap();
        assert!(repair_lid(&s, &id).is_err());
        assert_eq!(status(&s, phone).unwrap().unwrap(), conflict);
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                status_key("15551234567").as_bytes(),
                1,
                &updated,
            )
        })
        .unwrap();
        let err = repair_lid(&s, "unknown-private-id")
            .unwrap_err()
            .to_string();
        assert!(!err.contains("unknown-private-id"));
        assert!(err.contains("saved_reply_missing"));
        queue_check(&s, phone).unwrap();
        assert!(
            repair_lid(&s, &id).is_err(),
            "a second check blocks ambiguous attribution"
        );
        assert_eq!(status(&s, phone).unwrap().unwrap(), updated);
    }

    #[test]
    fn contact_projection_uses_current_transaction_state_after_enumeration() {
        use super::super::actions;
        use crate::binary::{Encoder, TokenDictionary};
        for case in [
            "replacement",
            "kind",
            "status",
            "sequence",
            "reply_id",
            "reply_type",
            "reply_missing",
        ] {
            let (dir, store) = store();
            let phone = "+15551234567";
            let id = queue_check(&store, phone).unwrap();
            actions::reserve_pending(&store).unwrap();
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("usync").children(vec![
                    Node::new("list").children(vec![user_node(
                        phone,
                        Some("15551234567@s.whatsapp.net"),
                        None,
                    )]),
                ])]);
            actions::record_reply(&store, &reply).unwrap();
            let listed = actions::settled_for_kinds(&store, &[ACTION_KIND])
                .unwrap()
                .pop()
                .unwrap()
                .0;
            store
                .write(|tx| {
                    let key = format!("action-outbox:{id}");
                    let mut row: serde_json::Value =
                        tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                    let wire = |node: &Node| {
                        Encoder::new(TokenDictionary::comparison_2_26_37_71())
                            .encode(node)
                            .unwrap()
                    };
                    match case {
                        "replacement" => {
                            let current =
                                reply
                                    .clone()
                                    .children(vec![Node::new("usync").children(vec![
                            Node::new("list").children(vec![Node::new("user").children(vec![
                                Node::new("contact").attr("type", "out").text(phone)])])])]);
                            row["reply"] = serde_json::json!(wire(&current));
                        }
                        "kind" => row["kind"] = serde_json::json!("peer_keys"),
                        "status" => row["status"] = serde_json::json!("sent"),
                        "sequence" => {
                            row["settlement_sequence"] =
                                serde_json::json!(listed.settlement_sequence.unwrap() + 1)
                        }
                        "reply_id" => {
                            row["reply"] =
                                serde_json::json!(wire(&reply.clone().attr("id", "foreign")))
                        }
                        "reply_type" => {
                            row["reply"] =
                                serde_json::json!(wire(&reply.clone().attr("type", "error")))
                        }
                        "reply_missing" => row["reply"] = serde_json::Value::Null,
                        _ => unreachable!(),
                    }
                    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let result = store.write(|tx| ingest_in(tx, &listed));
            if case == "replacement" {
                assert_eq!(result.unwrap(), 1);
                let saved = status(&store, phone).unwrap().unwrap();
                assert!(
                    !saved.on_whatsapp,
                    "must use current out result, not enumerated in result"
                );
                assert!(saved.jid.is_none());
                assert_eq!(ingest(&store).unwrap(), 0);
            } else {
                let error = result.unwrap_err().to_string();
                assert!(!error.contains(&id));
                assert!(!error.contains("15551234567"));
                assert_eq!(
                    before,
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    "{case}"
                );
            }
            let after = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            drop(store);
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("c.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
                create: false,
            })
            .unwrap();
            if case == "replacement" {
                assert_eq!(ingest(&store).unwrap(), 0);
            } else {
                assert!(store.write(|tx| ingest_in(tx, &listed)).is_err());
            }
            assert_eq!(
                after,
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                "{case} after reopen"
            );
        }
    }

    #[test]
    fn lid_repair_revalidates_saved_request_and_reply_without_mutation_after_reopen() {
        use super::super::actions;
        use crate::binary::{Encoder, TokenDictionary};
        for case in [
            "request_target",
            "request_id",
            "reply_id",
            "reply_type",
            "reply_root",
            "duplicate_usync",
            "duplicate_list",
            "kind",
            "status",
        ] {
            let (dir, store) = store();
            let phone = "+15551234567";
            let id = queue_check(&store, phone).unwrap();
            let request = actions::reserve_pending(&store).unwrap().pop().unwrap();
            let user = user_node(
                phone,
                Some("15551234567@s.whatsapp.net"),
                Some("999999999999@lid"),
            );
            let sync = Node::new("usync").children(vec![
                Node::new("result").children(vec![Node::new("lid")]),
                Node::new("list").children(vec![user.clone()]),
            ]);
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![sync.clone()]);
            actions::record_reply(&store, &reply).unwrap();
            assert_eq!(ingest(&store).unwrap(), 1);
            let mut old = status(&store, phone).unwrap().unwrap();
            old.lid = None;
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        status_key("15551234567").as_bytes(),
                        1,
                        &old,
                    )?;
                    let key = format!("action-outbox:{id}");
                    let mut row: serde_json::Value =
                        tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                    let wire = |node: &Node| {
                        Encoder::new(TokenDictionary::comparison_2_26_37_71())
                            .encode(node)
                            .unwrap()
                    };
                    match case {
                        "request_target" => {
                            let changed =
                                request
                                    .clone()
                                    .children(vec![Node::new("usync").children(vec![
                                        Node::new("list").children(vec![
                                            Node::new("user").children(vec![
                                                Node::new("contact").text("+15559999999"),
                                            ]),
                                        ]),
                                    ])]);
                            row["wire"] = serde_json::json!(wire(&changed));
                        }
                        "request_id" => {
                            row["wire"] =
                                serde_json::json!(wire(&request.clone().attr("id", "foreign")))
                        }
                        "reply_id" => {
                            row["reply"] =
                                serde_json::json!(wire(&reply.clone().attr("id", "foreign")))
                        }
                        "reply_type" => {
                            row["reply"] =
                                serde_json::json!(wire(&reply.clone().attr("type", "error")))
                        }
                        "reply_root" => {
                            let mut changed = reply.clone();
                            changed.description = "message".into();
                            row["reply"] = serde_json::json!(wire(&changed));
                        }
                        "duplicate_usync" => {
                            row["reply"] = serde_json::json!(wire(
                                &reply.clone().children(vec![sync.clone(), sync.clone()])
                            ))
                        }
                        "duplicate_list" => {
                            row["reply"] = serde_json::json!(wire(&reply.clone().children(vec![
                                Node::new("usync").children(vec![
                                    Node::new("result").children(vec![Node::new("lid")]),
                                    Node::new("list").children(vec![user.clone()]),
                                    Node::new("list").children(vec![user.clone()])
                                ])
                            ])))
                        }
                        "kind" => row["kind"] = serde_json::json!("peer_keys"),
                        "status" => row["status"] = serde_json::json!("sent"),
                        _ => unreachable!(),
                    }
                    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            {
                let error = repair_lid(&store, &id).unwrap_err().to_string();
                for private in [id.as_str(), "15551234567", "15559999999", "999999999999"] {
                    assert!(!error.contains(private), "{case}");
                }
                assert_eq!(
                    before,
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    "{case}"
                );
            }
            drop(store);
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("c.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([11u8; 32]))),
                create: false,
            })
            .unwrap();
            assert!(repair_lid(&store, &id).is_err(), "{case} after reopen");
            assert_eq!(
                before,
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap()
            );
        }
    }

    #[test]
    fn historical_status_type_remains_unknown_without_rewriting_boolean() {
        let status: ContactStatus = serde_json::from_value(serde_json::json!({
            "on_whatsapp":true, "jid":null, "lid":null, "checked_at":42, "error":null
        }))
        .unwrap();
        assert!(status.on_whatsapp);
        assert!(status.contact_type.is_none());
    }

    #[test]
    fn ingest_records_a_top_level_refusal_against_the_checked_number() {
        let (_d, s) = store();
        let id = queue_check(&s, "+15551234567").unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "error")
            .children(vec![
                Node::new("error")
                    .attr("code", "429")
                    .attr("text", "rate-overlimit"),
            ]);
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        assert_eq!(
            super::super::actions::outcomes(&s)
                .unwrap()
                .into_iter()
                .find(|row| row.id == id)
                .unwrap()
                .contact_projection,
            Some(ContactProjection::ServerRefused)
        );
        let saved = status(&s, "+15551234567").unwrap().unwrap();
        assert!(!saved.on_whatsapp);
        assert_eq!(saved.error.as_deref(), Some("rate-overlimit"));
    }
}
