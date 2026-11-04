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
const SETTLEMENT_SEQUENCE: &[u8] = b"action-settlement-sequence";
/// At most this many queued actions are written per reservation.
pub(crate) const BATCH: usize = 16;

/// Decode the stored action bytes without re-encoding their identity.
pub(crate) fn decode(bytes: &[u8]) -> Result<Node> {
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
    /// Reserved for a socket write, marked before the write so a cut never
    /// resends it; whether it reached the socket is unknown until a reply
    /// arrives (`reserve_pending` reserves up to 16, written one at a time).
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
    settlement_sequence: Option<u64>,
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
    /// Durable receipt order; absent on legacy/unsettled actions. Never inferred.
    pub settlement_sequence: Option<u64>,
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
        settlement_sequence: None,
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

/// Original action plus reply within a feature's atomic settlement transaction.
pub(crate) fn view_in_write(
    tx: &crate::store::WriteTx<'_>,
    id: &str,
) -> Result<Option<(ActionOutcome, Option<Node>, Node)>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let request = decode(&row.wire)?;
    let reply = row.reply.map(|wire| decode(&wire)).transpose()?;
    Ok(Some((
        ActionOutcome {
            settlement_sequence: row.settlement_sequence,
            id: id.to_owned(),
            kind: row.kind,
            label: row.label,
            status: row.status,
            error_code: row.error_code,
            error_text: row.error_text,
        },
        reply,
        request,
    )))
}

pub(crate) fn settled_at_in(tx: &crate::store::WriteTx<'_>, id: &str) -> Result<Option<Timestamp>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    Ok(row.and_then(|row| row.settled_at))
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
            settlement_sequence: row.settlement_sequence,
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
    reserve_limit(store, BATCH)
}

/// Reserve only the next eligible action for the connection actor. All bulk
/// admission guards remain shared; later actions stay queued until their turn.
pub(crate) fn reserve_next(store: &ProtocolStore) -> Result<Option<Node>> {
    Ok(reserve_limit(store, 1)?.pop())
}

fn reserve_limit(store: &ProtocolStore, limit: usize) -> Result<Vec<Node>> {
    store.write(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        while out.len() < limit {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows
                .iter()
                .filter(|(key, _)| key.starts_with(PREFIX.as_bytes()))
            {
                if out.len() == limit {
                    break;
                }
                let Some(mut row): Option<Row> = tx.get_json(Table::Checkpoints, key, 1)? else {
                    continue;
                };
                if row.status != ActionStatus::Queued {
                    continue;
                }
                let node = decode(&row.wire)?;
                if row.kind == super::app_state_sync::HELD_FETCH_ACTION_KIND {
                    let id = node
                        .get_attr("id")
                        .and_then(|v| v.as_text())
                        .ok_or_else(|| Error::invalid("action", "held fetch action has no id"))?;
                    if !super::app_state_sync::held_fetch::reserve_in(tx, id, &row.wire)? {
                        continue;
                    }
                }
                if row.kind == super::app_state_sync::FETCH_ACTION_KIND
                    && super::app_state_sync::server_waiting_in(tx)?
                {
                    continue;
                }
                if row.kind == super::app_state_sync::PATCH_ACTION_KIND {
                    let id = node
                        .get_attr("id")
                        .and_then(|v| v.as_text())
                        .ok_or_else(|| Error::invalid("action", "patch action has no id"))?;
                    if !super::app_state_sync::validate_patch_reservation_in(tx, id)? {
                        continue;
                    }
                }
                out.push(node);
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
    if row.kind == super::app_state_sync::HELD_FETCH_ACTION_KIND
        && !super::app_state_sync::held_fetch::reserve_in(tx, id, &row.wire)?
    {
        return Err(Error::invalid(
            "app_state_sync",
            "selected held fetch is deferred or locally refused; no request reserved",
        ));
    }
    if row.kind == super::app_state_sync::FETCH_ACTION_KIND
        && super::app_state_sync::server_waiting_in(tx)?
    {
        return Err(Error::invalid(
            "app_state_sync",
            "server app-state backoff is active; no fetch reserved",
        ));
    }
    if row.kind == super::app_state_sync::PATCH_ACTION_KIND
        && !super::app_state_sync::validate_patch_reservation_in(tx, id)?
    {
        return Err(Error::invalid(
            "app_state_sync",
            "server app-state backoff is active; no action reserved",
        ));
    }
    row.status = ActionStatus::Sent;
    row.sent_at = Some(Timestamp::now());
    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)?;
    Ok(node)
}

/// Settle a `sent` action from its `iq` reply. Returns `true` when the node
/// matched a queued action (the caller then treats it as consumed).
pub fn record_reply(store: &ProtocolStore, node: &Node) -> Result<bool> {
    // Current APK 0ln.A01 dispatches only result/error; another IQ carrying
    // the same ID is not a definite refusal of the original request.
    if node.description != "iq"
        || !matches!(
            node.get_attr("type").and_then(|v| v.as_text()),
            Some("result" | "error")
        )
    {
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
        row.settlement_sequence = Some(next_settlement_sequence_in(tx)?);
        row.settled_at = Some(Timestamp::now());
        row.reply = Some(encode(node)?);
        super::groups::mark_pending_in(tx, &row.kind, id)?;
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)?;
        Ok(true)
    })
}

fn next_settlement_sequence_in(tx: &mut crate::store::WriteTx<'_>) -> Result<u64> {
    let current: Option<u64> = tx.get_json(Table::Checkpoints, SETTLEMENT_SEQUENCE, 1)?;
    if current == Some(0) {
        return Err(Error::invalid(
            "action",
            "invalid settlement sequence counter",
        ));
    }
    if current.is_none() {
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows
                .iter()
                .filter(|(key, _)| key.starts_with(PREFIX.as_bytes()))
            {
                let row: Row = tx.get_json(Table::Checkpoints, key, 1)?.ok_or_else(|| {
                    Error::invalid("action", "missing action during sequence check")
                })?;
                if row.settlement_sequence.is_some() {
                    return Err(Error::invalid(
                        "action",
                        "settlement sequence counter missing; no order reset",
                    ));
                }
            }
            if done {
                break;
            }
        }
    }
    let next = current
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| Error::invalid("action", "settlement sequence exhausted"))?;
    tx.put_json(Table::Checkpoints, SETTLEMENT_SEQUENCE, 1, &next)?;
    Ok(next)
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
                    settlement_sequence: row.settlement_sequence,
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
                settlement_sequence: row.settlement_sequence,
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

/// Exact stored encoded request for fingerprinting; never a fresh Node encoding.
pub(crate) fn request_wire_in(tx: &crate::store::ReadTx<'_>, id: &str) -> Result<Option<Vec<u8>>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    Ok(row.map(|row| row.wire))
}

/// Same exact request-byte read inside preparation/reservation transactions.
pub(crate) fn request_wire_in_write(
    tx: &crate::store::WriteTx<'_>,
    id: &str,
) -> Result<Option<Vec<u8>>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    Ok(row.map(|row| row.wire))
}

/// Exact immutable wire pair for a feature's settlement journal; never re-encoded.
pub(crate) fn wire_pair_in_write(
    tx: &crate::store::WriteTx<'_>,
    id: &str,
) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
    let row: Option<Row> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
    Ok(row.and_then(|row| row.reply.map(|reply| (row.wire, reply))))
}

/// Original request in an atomic feature-state update.
pub(crate) fn request_in_write(tx: &crate::store::WriteTx<'_>, id: &str) -> Result<Option<Node>> {
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

/// Sequenced settlements across kinds in durable receive order (oldest first),
/// read with their replies in one snapshot. Legacy unknown-order rows remain
/// inspectable through outcomes/settled, but cannot establish projection order.
pub fn settled_for_kinds(
    store: &ProtocolStore,
    kinds: &[&str],
) -> Result<Vec<(ActionOutcome, Option<Node>)>> {
    store.read(|tx| {
        let mut ordered = std::collections::BTreeMap::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                let Some(id) = std::str::from_utf8(&key)
                    .ok()
                    .and_then(|key| key.strip_prefix(PREFIX))
                else {
                    continue;
                };
                let Some(row): Option<Row> = tx.get_json(Table::Checkpoints, &key, 1)? else {
                    continue;
                };
                if !kinds.contains(&row.kind.as_str())
                    || !matches!(row.status, ActionStatus::Ok | ActionStatus::Error)
                {
                    continue;
                }
                let Some(sequence) = row.settlement_sequence else {
                    continue;
                };
                if sequence == 0 {
                    return Err(Error::invalid(
                        "action",
                        "invalid saved settlement sequence",
                    ));
                }
                let reply = row.reply.map(|wire| decode(&wire)).transpose()?;
                let outcome = ActionOutcome {
                    settlement_sequence: Some(sequence),
                    id: id.to_owned(),
                    kind: row.kind,
                    label: row.label,
                    status: row.status,
                    error_code: row.error_code,
                    error_text: row.error_text,
                };
                if ordered.insert(sequence, (outcome, reply)).is_some() {
                    return Err(Error::invalid(
                        "action",
                        "duplicate saved settlement sequence",
                    ));
                }
            }
            if done {
                return Ok(ordered.into_values().collect());
            }
        }
    })
}

/// Settled actions of one kind in reverse identifier order, with replies.
/// This legacy enumeration is not chronological; projection replay must use
/// [`settled_for_kinds`] rather than treating random identifiers as time.
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

    #[test]
    fn settlement_sequence_is_receive_order_not_identifier_or_clock_order() {
        let (_dir, store) = store();
        let mut ids: Vec<_> = ["first-kind", "second-kind"]
            .into_iter()
            .map(|kind| {
                queue(&store, kind, "fixture", |id| {
                    Ok(Node::new("iq").attr("id", id))
                })
                .unwrap()
            })
            .collect();
        ids.sort();
        reserve_pending(&store).unwrap();
        for (index, id) in ids.iter().enumerate() {
            let response = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result");
            assert!(record_reply(&store, &response).unwrap());
            assert!(!record_reply(&store, &response).unwrap());
            store
                .write(|tx| {
                    let mut row: Row = tx
                        .get_json(Table::Checkpoints, key(id).as_bytes(), 1)?
                        .unwrap();
                    row.settled_at = Some(Timestamp(100 - index as i64));
                    tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &row)
                })
                .unwrap();
        }
        let ordered = settled_for_kinds(&store, &["first-kind", "second-kind"]).unwrap();
        assert_eq!(
            ordered
                .iter()
                .map(|(o, _)| o.id.clone())
                .collect::<Vec<_>>(),
            ids
        );
        assert_eq!(
            ordered
                .iter()
                .map(|(o, _)| o.settlement_sequence.unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        store
            .write(|tx| {
                let mut legacy: Row = tx
                    .get_json(Table::Checkpoints, key(&ids[0]).as_bytes(), 1)?
                    .unwrap();
                legacy.settlement_sequence = None;
                tx.put_json(Table::Checkpoints, key(&ids[0]).as_bytes(), 1, &legacy)
            })
            .unwrap();
        assert_eq!(
            settled_for_kinds(&store, &["first-kind", "second-kind"])
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            outcomes(&store).unwrap().len(),
            2,
            "unknown-order legacy row remains inspectable"
        );
    }

    #[test]
    fn missing_or_exhausted_sequence_counter_cannot_reset_order_or_settle() {
        for missing in [false, true] {
            let (_dir, store) = store();
            let first = queue(&store, "fixture", "fixture", |id| {
                Ok(Node::new("iq").attr("id", id))
            })
            .unwrap();
            reserve_pending(&store).unwrap();
            record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", first.as_str())
                    .attr("type", "result"),
            )
            .unwrap();
            let second = queue(&store, "fixture", "fixture", |id| {
                Ok(Node::new("iq").attr("id", id))
            })
            .unwrap();
            reserve_pending(&store).unwrap();
            store
                .write(|tx| {
                    if missing {
                        tx.delete(Table::Checkpoints, SETTLEMENT_SEQUENCE)
                            .map(|_| ())
                    } else {
                        tx.put_json(Table::Checkpoints, SETTLEMENT_SEQUENCE, 1, &u64::MAX)
                    }
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert!(
                record_reply(
                    &store,
                    &Node::new("iq")
                        .attr("id", second.as_str())
                        .attr("type", "result")
                )
                .is_err()
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
        }
    }

    #[test]
    fn matching_id_without_result_or_error_does_not_settle_a_sent_request() {
        let (_dir, store) = store();
        let id = queue(&store, "fixture", "fixture", |id| {
            Ok(Node::new("iq").attr("id", id).attr("type", "get"))
        })
        .unwrap();
        reserve_pending(&store).unwrap();
        for kind in [None, Some("get"), Some("set"), Some("future")] {
            let mut node = Node::new("iq").attr("id", id.as_str());
            if let Some(kind) = kind {
                node = node.attr("type", kind);
            }
            assert!(!record_reply(&store, &node).unwrap());
            assert_eq!(outcomes(&store).unwrap()[0].status, ActionStatus::Sent);
            assert!(reply(&store, &id).unwrap().is_none());
        }
        assert!(
            record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
            )
            .unwrap()
        );
        assert_eq!(outcomes(&store).unwrap()[0].status, ActionStatus::Ok);
    }

    /// App-state answers per collection, so an `iq type=result` can still carry
    /// a refusal. Recording that as success is worse than recording an error,
    /// because the caller is told the write landed when it did not.
    #[test]
    fn a_collection_level_error_is_not_a_success() {
        let (_d, store) = store();
        let id = queue(&store, "classifier-fixture", "patch", |id| {
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
        let id = queue(&store, "classifier-fixture", "patch", |id| {
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
