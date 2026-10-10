//! Durable queue of one-shot IQ requests ("actions": block list changes, spam
//! reports, contact lookups, disappearing-mode defaults, …).
//!
//! An action is queued offline as an encoded `iq` stanza. The comparison
//! session loop sends queued actions once it is active (and on every keepalive
//! tick), marks them `sent`, and settles them when the matching `iq` reply
//! arrives (`result` → `ok`, `error` → `error` with the server code). A `sent`
//! action whose reply never arrives stays `sent`: nothing here retries, in line
//! with the rest of the transport (uncertain outcomes are preserved, never
//! replayed). Rows live in [`Table::Checkpoints`] under `action-outbox:<iq id>`.
use crate::{
    Error, Result,
    binary::{Encoder, Node, TokenDictionary},
    model::time::Timestamp,
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};

const PREFIX: &str = "action-outbox:";
/// At most this many queued actions are written per reservation.
const BATCH: usize = 16;

fn decode(bytes: &[u8]) -> Result<Node> {
    let limits = crate::profile::Limits::default();
    let raw = crate::binary::unpack_frame_payload(bytes, &limits)?;
    Ok(
        crate::binary::Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits)
            .decode_node()?,
    )
}

fn encode(node: &Node) -> Result<Vec<u8>> {
    Ok(Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(node)?)
}

/// Lifecycle of one queued action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    /// Durably queued; no socket write yet.
    Queued,
    /// Written to a session; the reply has not arrived (outcome unknown).
    Sent,
    /// The server answered `type=result`.
    Ok,
    /// The server answered `type=error`.
    Error,
}

/// Stored row.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Row {
    kind: String,
    label: String,
    wire: Vec<u8>,
    status: ActionStatus,
    created_at: Timestamp,
    #[serde(default)]
    sent_at: Option<Timestamp>,
    #[serde(default)]
    settled_at: Option<Timestamp>,
    #[serde(default)]
    reply: Option<Vec<u8>>,
    #[serde(default)]
    error_code: Option<u16>,
    #[serde(default)]
    error_text: Option<String>,
}

/// Redacted operator view of one action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionOutcome {
    /// The `iq` id, which is also the row id.
    pub id: String,
    /// Feature-defined kind, e.g. `block`, `unblock`, `blocklist`, `report`.
    pub kind: String,
    /// Feature-defined short label (must not contain phone numbers).
    pub label: String,
    /// Lifecycle state.
    pub status: ActionStatus,
    /// Server error code when refused.
    pub error_code: Option<u16>,
    /// Server error text when refused.
    pub error_text: Option<String>,
}

/// Durably queue an `iq` built by `build`, which receives the fresh stanza id
/// it must place in the `id` attribute. Returns the id.
pub fn queue(
    store: &ProtocolStore,
    kind: &str,
    label: &str,
    build: impl FnOnce(&str) -> Result<Node>,
) -> Result<String> {
    // Preserve the public builder contract: callbacks may inspect/update
    // other local state, so run them before acquiring the store's write lock.
    let id = super::messaging::new_message_id()?;
    let node = build(&id)?;
    store.write(|tx| queue_prepared_in(tx, kind, label, id, node))
}

/// Queue atomically with a feature's durable idempotency marker.
pub(crate) fn queue_in(
    tx: &mut crate::store::WriteTx<'_>,
    kind: &str,
    label: &str,
    build: impl FnOnce(&str) -> Result<Node>,
) -> Result<String> {
    let id = super::messaging::new_message_id()?;
    let node = build(&id)?;
    queue_prepared_in(tx, kind, label, id, node)
}

fn queue_prepared_in(
    tx: &mut crate::store::WriteTx<'_>,
    kind: &str,
    label: &str,
    id: String,
    node: Node,
) -> Result<String> {
    if node.description != "iq" || node.get_attr("id").and_then(|v| v.as_text()) != Some(&id) {
        return Err(Error::invalid(
            "action",
            "actions must be iq stanzas carrying the assigned id",
        ));
    }
    let row = Row {
        kind: kind.to_owned(),
        label: label.to_owned(),
        wire: encode(&node)?,
        status: ActionStatus::Queued,
        created_at: Timestamp::now(),
        sent_at: None,
        settled_at: None,
        reply: None,
        error_code: None,
        error_text: None,
    };
    tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &row)?;
    Ok(id)
}

fn key(id: &str) -> String {
    format!("{PREFIX}{id}")
}

/// Successful reply for validating an explicit refresh in the same transaction.
pub(crate) fn completed_reply_in(tx: &crate::store::WriteTx<'_>, id: &str) -> Result<Option<Node>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    row.filter(|row| row.status == ActionStatus::Ok)
        .and_then(|row| row.reply)
        .map(|wire| decode(&wire))
        .transpose()
}

/// One immutable view of an action and its authenticated saved response.
pub(crate) fn view_in(
    tx: &crate::store::ReadTx<'_>,
    id: &str,
) -> Result<Option<(ActionOutcome, Option<Node>)>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let reply = row.reply.map(|wire| decode(&wire)).transpose()?;
    Ok(Some((
        ActionOutcome {
            id: id.to_owned(),
            kind: row.kind,
            label: row.label,
            status: row.status,
            error_code: row.error_code,
            error_text: row.error_text,
        },
        reply,
    )))
}

/// Reserve at most sixteen queued actions for one socket write, marking them
/// `sent` before the write so a crash never resends them.
pub fn reserve_pending(store: &ProtocolStore) -> Result<Vec<Node>> {
    store.write(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        while out.len() < BATCH {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows
                .iter()
                .filter(|(key, _)| key.starts_with(PREFIX.as_bytes()))
            {
                if out.len() == BATCH {
                    break;
                }
                let Some(mut row): Option<Row> = tx.get_json(Table::Checkpoints, key, 1)? else {
                    continue;
                };
                if row.status != ActionStatus::Queued {
                    continue;
                }
                out.push(decode(&row.wire)?);
                row.status = ActionStatus::Sent;
                row.sent_at = Some(Timestamp::now());
                tx.put_json(Table::Checkpoints, key, 1, &row)?;
            }
        }
        Ok(out)
    })
}

/// Compose one feature-owned action reservation with its original intent.
pub(crate) fn reserve_one_in(tx: &mut crate::store::WriteTx<'_>, id: &str) -> Result<Node> {
    let key = key(id);
    let mut row: Row = tx
        .get_json(Table::Checkpoints, key.as_bytes(), 1)?
        .ok_or_else(|| Error::invalid("action", "missing selected action"))?;
    if row.status != ActionStatus::Queued {
        return Err(Error::invalid(
            "action",
            "selected action already reserved; no replay",
        ));
    }
    let node = decode(&row.wire)?;
    row.status = ActionStatus::Sent;
    row.sent_at = Some(Timestamp::now());
    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)?;
    Ok(node)
}

/// Settle a `sent` action from its `iq` reply. Returns `true` when the node
/// matched a queued action (the caller then treats it as consumed).
pub fn record_reply(store: &ProtocolStore, node: &Node) -> Result<bool> {
    if node.description != "iq" {
        return Ok(false);
    }
    let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(false);
    };
    if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Ok(false);
    }
    let key = key(id);
    store.write(|tx| {
        let Some(mut row): Option<Row> = tx.get_json(Table::Checkpoints, key.as_bytes(), 1)? else {
            return Ok(false);
        };
        if row.status != ActionStatus::Sent {
            return Ok(false);
        }
        // An `iq type=result` can still carry a refusal further in. App-state
        // answers per collection: `<sync><collection type="error"><error
        // code="409" text="conflict"/>`, observed live 2026-09-24. Reading only
        // the outer type recorded that as success, which is worse than an error
        // — it is an error the caller is told is fine.
        let inner_error = node
            .child_nodes()
            .iter()
            .flat_map(|child| child.child_nodes())
            .find_map(|child| {
                // An encrypt directory result can carry <list><user><error/>
                // without type=error on user. Preserve that refusal too.
                (child.description == "user"
                    || child.get_attr("type").and_then(|v| v.as_text()) == Some("error"))
                .then(|| child.child("error"))
                .flatten()
            })
            .or_else(|| node.child("error"));
        let kind = node.get_attr("type").and_then(|v| v.as_text());
        if kind == Some("result") && inner_error.is_none() {
            row.status = ActionStatus::Ok;
        } else {
            row.status = ActionStatus::Error;
            if let Some(error) = inner_error {
                row.error_code = error
                    .get_attr("code")
                    .and_then(|v| v.as_text())
                    .and_then(|v| v.parse().ok());
                row.error_text = error
                    .get_attr("text")
                    .and_then(|v| v.as_text())
                    .map(str::to_owned);
            }
        }
        row.settled_at = Some(Timestamp::now());
        row.reply = Some(encode(node)?);
        super::groups::mark_pending_in(tx, &row.kind, id)?;
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)?;
        Ok(true)
    })
}

/// Every action ever queued, in key order, redacted.
pub fn outcomes(store: &ProtocolStore) -> Result<Vec<ActionOutcome>> {
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                let Some(id) = std::str::from_utf8(&key)
                    .ok()
                    .and_then(|k| k.strip_prefix(PREFIX))
                else {
                    continue;
                };
                let Some(row): Option<Row> = tx.get_json(Table::Checkpoints, &key, 1)? else {
                    continue;
                };
                out.push(ActionOutcome {
                    id: id.to_owned(),
                    kind: row.kind,
                    label: row.label,
                    status: row.status,
                    error_code: row.error_code,
                    error_text: row.error_text,
                });
            }
            if done {
                return Ok(out);
            }
        }
    })
}

// Used by bounded local migrations so attribution cannot race a new action.
pub(crate) fn outcomes_in_write(tx: &crate::store::WriteTx<'_>) -> Result<Vec<ActionOutcome>> {
    let mut out = Vec::new();
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        let done = rows.len() < 256;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows {
            let Some(id) = std::str::from_utf8(&key)
                .ok()
                .and_then(|k| k.strip_prefix(PREFIX))
            else {
                continue;
            };
            let Some(row): Option<Row> = tx.get_json(Table::Checkpoints, &key, 1)? else {
                continue;
            };
            out.push(ActionOutcome {
                id: id.to_owned(),
                kind: row.kind,
                label: row.label,
                status: row.status,
                error_code: row.error_code,
                error_text: row.error_text,
            });
        }
        if done {
            return Ok(out);
        }
    }
}

/// The queued request of an action, decoded (for diagnostics).
pub fn request(store: &ProtocolStore, id: &str) -> Result<Option<Node>> {
    store.read(|tx| request_in(tx, id))
}

/// Original immutable request alongside a reply/pin inspection in one snapshot.
pub(crate) fn request_in(tx: &crate::store::ReadTx<'_>, id: &str) -> Result<Option<Node>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    row.map(|r| decode(&r.wire)).transpose()
}

/// The saved reply of a settled action, decoded, for feature-specific parsing.
pub fn reply(store: &ProtocolStore, id: &str) -> Result<Option<Node>> {
    let row: Option<Row> =
        store.read(|tx| tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1))?;
    row.and_then(|r| r.reply)
        .map(|wire| decode(&wire))
        .transpose()
}

/// The settled actions of one kind, newest first, with their decoded replies.
pub fn settled(store: &ProtocolStore, kind: &str) -> Result<Vec<(ActionOutcome, Option<Node>)>> {
    let mut out = Vec::new();
    for outcome in outcomes(store)? {
        if outcome.kind != kind || !matches!(outcome.status, ActionStatus::Ok | ActionStatus::Error)
        {
            continue;
        }
        let node = reply(store, &outcome.id)?;
        out.push((outcome, node));
    }
    out.reverse();
    Ok(out)
}

#[cfg(test)]
mod inner_error_tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };

    fn store() -> (tempfile::TempDir, ProtocolStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap();
        (dir, store)
    }

    /// App-state answers per collection, so an `iq type=result` can still carry
    /// a refusal. Recording that as success is worse than recording an error,
    /// because the caller is told the write landed when it did not.
    #[test]
    fn a_collection_level_error_is_not_a_success() {
        let (_d, store) = store();
        let id = queue(&store, "app-state-patch", "patch", |id| {
            Ok(Node::new("iq").attr("id", id))
        })
        .unwrap();
        reserve_pending(&store).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("sync").children(vec![
                Node::new("collection")
                    .attr("type", "error")
                    .attr("name", "critical_unblock_low")
                    .children(vec![
                        Node::new("error").attr("code", "409").attr("text", "conflict"),
                    ]),
            ])]);
        assert!(record_reply(&store, &reply).unwrap());
        let outcome = outcomes(&store)
            .unwrap()
            .into_iter()
            .find(|o| o.id == id)
            .unwrap();
        assert_eq!(outcome.status, ActionStatus::Error);
        assert_eq!(outcome.error_code, Some(409));
        assert_eq!(outcome.error_text.as_deref(), Some("conflict"));
    }

    /// A plain successful reply must still be a success, including one whose
    /// children carry no `type` at all.
    #[test]
    fn a_clean_result_is_still_ok() {
        let (_d, store) = store();
        let id = queue(&store, "app-state-patch", "patch", |id| {
            Ok(Node::new("iq").attr("id", id))
        })
        .unwrap();
        reserve_pending(&store).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("sync").children(vec![
                Node::new("collection")
                    .attr("name", "critical_unblock_low")
                    .attr("version", "1"),
            ])]);
        assert!(record_reply(&store, &reply).unwrap());
        let outcome = outcomes(&store)
            .unwrap()
            .into_iter()
            .find(|o| o.id == id)
            .unwrap();
        assert_eq!(outcome.status, ActionStatus::Ok);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use std::sync::Arc;

    fn store() -> (tempfile::TempDir, ProtocolStore) {
        let dir = tempfile::tempdir().unwrap();
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, s)
    }

    fn ping(id: &str) -> Result<Node> {
        Ok(Node::new("iq")
            .attr("id", id)
            .attr("type", "get")
            .attr("xmlns", "w:p")
            .attr("to", crate::model::jid::Jid::parse("s.whatsapp.net")?))
    }

    #[test]
    fn queue_reserve_and_settle_once() {
        let (_d, s) = store();
        let id = queue(&s, "ping", "keepalive", ping).unwrap();
        assert_eq!(outcomes(&s).unwrap()[0].status, ActionStatus::Queued);
        let sent = reserve_pending(&s).unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(
            sent[0].get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        // A second reservation writes nothing: outcome unknown, never resent.
        assert!(reserve_pending(&s).unwrap().is_empty());
        assert_eq!(outcomes(&s).unwrap()[0].status, ActionStatus::Sent);
        let unrelated = Node::new("iq").attr("id", "ABC").attr("type", "result");
        assert!(!record_reply(&s, &unrelated).unwrap());
        let reply_node = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "error")
            .children(vec![
                Node::new("error")
                    .attr("code", "401")
                    .attr("text", "not-authorized"),
            ]);
        assert!(record_reply(&s, &reply_node).unwrap());
        assert!(!record_reply(&s, &reply_node).unwrap(), "already settled");
        let outcome = &outcomes(&s).unwrap()[0];
        assert_eq!(outcome.status, ActionStatus::Error);
        assert_eq!(outcome.error_code, Some(401));
        assert_eq!(outcome.error_text.as_deref(), Some("not-authorized"));
        let saved = reply(&s, &id).unwrap().unwrap();
        assert!(saved.child("error").is_some());
        assert_eq!(settled(&s, "ping").unwrap().len(), 1);
        assert!(settled(&s, "other").unwrap().is_empty());
    }

    #[test]
    fn rejects_non_iq_or_wrong_id() {
        let (_d, s) = store();
        assert!(queue(&s, "x", "x", |_| Ok(Node::new("message"))).is_err());
        assert!(queue(&s, "x", "x", |_| Ok(Node::new("iq").attr("id", "other"))).is_err());
        assert!(outcomes(&s).unwrap().is_empty());
    }
}
