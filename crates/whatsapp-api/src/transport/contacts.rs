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
//!   child, if present, is parsed defensively since its exact shape was not
//!   traced (chat-actions.md section 7 marks it **unknown**).
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::{jid::Jid, time::Timestamp},
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};

const CONTACT_PREFIX: &str = "contact:";
const STATUS_PREFIX: &str = "contact-status:";
const CHECK_REQUEST_PREFIX: &str = "contact-check-req:";
const INGESTED_PREFIX: &str = "contacts-ingested:";
const ACTION_KIND: &str = "contact-check";

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

/// The saved WhatsApp-existence status for one number, if a check ever settled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactStatus {
    /// Whether the number carries a WhatsApp account.
    pub on_whatsapp: bool,
    /// The account's phone-number JID, when returned.
    pub jid: Option<Jid>,
    /// The account's LID, when a `lid` child was present and parsed.
    pub lid: Option<Jid>,
    /// When this status was recorded.
    pub checked_at: Timestamp,
    /// Sanitized server error, set when the whole request was refused
    /// (`iq type=error`, which carries no per-user data to confirm or deny).
    pub error: Option<String>,
}

/// Queue a `usync` existence check for `phone` through the durable one-shot
/// action queue. Returns the action id; nothing is sent until a comparison
/// session drains queued actions (see `super::actions`).
pub fn queue_check(store: &ProtocolStore, phone: &str) -> Result<String> {
    let (digits, canonical) = normalize(phone)?;
    let id = super::actions::queue(store, ACTION_KIND, "contact check", |id| {
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
    store.write(|tx| {
        tx.put_json(
            Table::Checkpoints,
            check_request_key(&id).as_bytes(),
            1,
            &digits,
        )
    })?;
    Ok(id)
}

fn attr_jid(node: &Node, name: &str) -> Option<Jid> {
    let value = node.get_attr(name)?;
    value
        .as_jid()
        .cloned()
        .or_else(|| Jid::parse(value.as_text()?).ok())
}

/// Parse one reply `user` node into `(digits, status)`. Defensive: any
/// unexpected shape is skipped rather than treated as an error.
fn parse_user(user: &Node) -> Option<(String, ContactStatus)> {
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
    let jid = attr_jid(user, "jid");
    let lid = user.child("lid").and_then(|l| attr_jid(l, "jid"));
    Some((
        digits,
        ContactStatus {
            on_whatsapp: jid.is_some(),
            jid,
            lid,
            checked_at: Timestamp::now(),
            error: None,
        },
    ))
}

/// Ingest every settled `contact-check` action not yet processed, saving a
/// [`ContactStatus`] row per resolved number. Idempotent: a
/// `contacts-ingested:<action id>` marker skips actions already ingested.
/// Returns the number of status rows newly recorded.
pub fn ingest(store: &ProtocolStore) -> Result<usize> {
    let mut recorded = 0usize;
    for (outcome, reply) in super::actions::settled(store, ACTION_KIND)? {
        let marker = ingested_key(&outcome.id);
        let newly = store.write(|tx| {
            if tx
                .get_json::<bool>(Table::Checkpoints, marker.as_bytes(), 1)?
                .is_some()
            {
                return Ok(0usize);
            }
            let mut count = 0usize;
            match outcome.status {
                super::actions::ActionStatus::Ok => {
                    if let Some(node) = &reply
                        && let Some(usync) = node.child("usync")
                        && let Some(list) = usync.child("list")
                    {
                        for user in list
                            .child_nodes()
                            .iter()
                            .filter(|n| n.description == "user")
                        {
                            let Some((digits, status)) = parse_user(user) else {
                                continue;
                            };
                            tx.put_json(
                                Table::Checkpoints,
                                status_key(&digits).as_bytes(),
                                1,
                                &status,
                            )?;
                            count += 1;
                        }
                    }
                }
                super::actions::ActionStatus::Error => {
                    let req_key = check_request_key(&outcome.id);
                    if let Some(digits) =
                        tx.get_json::<String>(Table::Checkpoints, req_key.as_bytes(), 1)?
                    {
                        let error = outcome
                            .error_text
                            .clone()
                            .or_else(|| outcome.error_code.map(|c| c.to_string()))
                            .unwrap_or_else(|| "refused".to_owned());
                        tx.put_json(
                            Table::Checkpoints,
                            status_key(&digits).as_bytes(),
                            1,
                            &ContactStatus {
                                on_whatsapp: false,
                                jid: None,
                                lid: None,
                                checked_at: Timestamp::now(),
                                error: Some(error),
                            },
                        )?;
                        count += 1;
                    }
                }
                super::actions::ActionStatus::Queued | super::actions::ActionStatus::Sent => {}
            }
            tx.put_json(Table::Checkpoints, marker.as_bytes(), 1, &true)?;
            Ok(count)
        })?;
        recorded += newly;
    }
    Ok(recorded)
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
        let mut children = vec![Node::new("contact").attr("type", "in").text(phone)];
        if let Some(lid) = lid {
            children.push(Node::new("lid").attr("jid", Jid::parse(lid).unwrap()));
        }
        let mut n = Node::new("user").children(children);
        if let Some(jid) = jid {
            n = n.attr("jid", Jid::parse(jid).unwrap());
        }
        n
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
    fn ingest_records_on_and_off_whatsapp_and_is_idempotent() {
        let (_d, s) = store();
        let id = queue_check(&s, "+15551234567").unwrap();
        assert!(super::super::actions::reserve_pending(&s).unwrap().len() == 1);
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("usync").attr("sid", "x").children(vec![
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
        assert!(saved2.jid.is_none());
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
        let saved = status(&s, "+15551234567").unwrap().unwrap();
        assert!(!saved.on_whatsapp);
        assert_eq!(saved.error.as_deref(), Some("rate-overlimit"));
    }
}
