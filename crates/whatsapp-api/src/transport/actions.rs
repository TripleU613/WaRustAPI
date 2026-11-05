//! Durable queue of one-shot IQ requests ("actions": block list changes, spam
//! reports, contact lookups, disappearing-mode defaults, …).
//!
//! An action is queued offline as an encoded `iq` stanza. The comparison
//! session loop sends queued actions once it is active (and on every keepalive
//! tick), marks them `sent`, and settles them when the matching `iq` reply
//! arrives (`result` → `ok`, `error` → `error` with the server code). A `sent`
//! action whose reply never arrives stays `sent`: nothing here retries, in line
//! with the rest of the transport (uncertain outcomes are preserved, never
//! replayed). Normal receive requires a callback owned by an action locally
//! written on that connection; durable rows do not recreate live ownership.
//! Explicit saved-reply reconciliation remains separate. Rows live in
//! [`Table::Checkpoints`] under `action-outbox:<iq id>`.
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

fn decode_owned(id: &str, wire: &[u8]) -> Result<Node> {
    if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::invalid(
            "action",
            "invalid saved action row identity",
        ));
    }
    let node = decode(wire)?;
    if node.description != "iq" || node.get_attr("id").and_then(|value| value.as_text()) != Some(id)
    {
        return Err(Error::invalid(
            "action",
            "saved action wire does not match its row identity",
        ));
    }
    Ok(node)
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
                // Recheck at reservation: a restriction can arrive after queueing.
                // Keep the original create queued, never mark it sent while held.
                if row.kind == "group-create"
                    && super::notifications::recorded_active_reachout_timelock_in(tx)?.is_some()
                {
                    tracing::info!(
                        "group creation deferred: recorded reach-out restriction remains active"
                    );
                    continue;
                }
                let id = std::str::from_utf8(&key[PREFIX.len()..])
                    .map_err(|_| Error::invalid("action", "invalid saved action row identity"))?;
                let node = decode_owned(id, &row.wire)?;
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
    if row.kind == "group-create"
        && super::notifications::recorded_active_reachout_timelock_in(tx)?.is_some()
    {
        return Err(Error::invalid(
            "group",
            "recorded reach-out restriction remains active; group creation stays queued until an explicit inactive update",
        ));
    }
    let node = decode_owned(id, &row.wire)?;
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

/// Callback claims for actions written by one normal connection invocation.
/// Durable Sent rows are reservations, not callbacks on a later connection.
/// APK43.72 0ln.A02 removes sent-map ownership before callback parsing;
/// selected disconnect handler case3 clears A0H, separately from queued A0I.
#[derive(Default)]
pub(crate) struct LiveReplies(
    std::collections::HashSet<String>,
    std::collections::HashMap<String, tokio::time::Instant>,
);

impl LiveReplies {
    pub(crate) fn record_written(&mut self, node: &Node, started: tokio::time::Instant) {
        if node.description == "iq"
            && let Some(id) = node.get_attr("id").and_then(|value| value.as_text())
        {
            self.0.insert(id.to_owned());
            // APK43.72 HBE.A00 -> 0Sf.A0W/A0A -> 0ln.A08: the initial
            // AB-properties callback has 32000ms from pre-dispatch registration.
            // Other producers' durations are not inferred from this request.
            let seconds = if node.get_attr("type").and_then(|v| v.as_text()) == Some("get")
                && node.get_attr("xmlns").and_then(|v| v.as_text()) == Some("abt")
                && let [props] = node.child_nodes()
                && props.description == "props"
                && props.get_attr("protocol").and_then(|v| v.as_text()) == Some("1")
                && props.get_attr("hash").and_then(|v| v.as_text()) == Some("")
                && props.get_attr("refresh_id").is_none()
            {
                Some(32)
            } else if node.get_attr("type").and_then(|v| v.as_text()) == Some("set")
                && node.get_attr("xmlns").and_then(|v| v.as_text()) == Some("blocklist")
                && node.child_nodes().first().is_some_and(|item| {
                    item.description == "item"
                        && matches!(
                            item.get_attr("action").and_then(|v| v.as_text()),
                            Some("block" | "unblock")
                        )
                })
            {
                // APK43.72 1dH.A05 0638..0650 -> 0Sf.A0W: block/unblock
                // use20000ms. A0M fetch uses zero through A0Q/A06, so it
                // does not inherit this set-request timer.
                Some(20)
            } else if node.get_attr("xmlns").and_then(|v| v.as_text()) == Some("disappearing_mode")
                && match node.get_attr("type").and_then(|v| v.as_text()) {
                    Some("get") => node.child_nodes().is_empty(),
                    Some("set") => matches!(node.child_nodes(), [mode]
                        if mode.description == "disappearing_mode"
                            && mode.get_attr("duration").and_then(|v| v.as_text()).is_some()),
                    _ => false,
                }
            {
                // APK43.72 ChangeDMSettingActivity.A0c 018a..0198 and
                // 0oy.A00 00b4..00c2 both pass20000ms to 0Sf.A0W.
                // This is the account default IQ, not per-chat E2E timers
                // or a disappearing_mode child nested inside usync.
                Some(20)
            } else {
                None
            };
            if let Some(seconds) = seconds {
                self.1.insert(
                    id.to_owned(),
                    started + std::time::Duration::from_secs(seconds),
                );
            }
        }
    }

    pub(crate) fn deadline(&self) -> Option<tokio::time::Instant> {
        self.1.values().copied().min()
    }

    pub(crate) fn expire(&mut self) {
        let now = tokio::time::Instant::now();
        // Timer and response are competing claims. Do not add an age check to
        // record_reply: native A0H cancels message4 before A02 claims ownership.
        self.1.retain(|id, deadline| {
            if *deadline <= now {
                self.0.remove(id);
                false
            } else {
                true
            }
        });
    }

    pub(crate) fn record_reply(&mut self, store: &ProtocolStore, node: &Node) -> Result<bool> {
        if node.description != "iq"
            || !matches!(
                node.get_attr("type").and_then(|value| value.as_text()),
                Some("result" | "error")
            )
        {
            return Ok(false);
        }
        let Some(id) = node.get_attr("id").and_then(|value| value.as_text()) else {
            return Ok(false);
        };
        // Claim once before parsing or durable settlement. A local failure
        // leaves uncertainty; it cannot authorize a second callback attempt.
        if !self.0.remove(id) {
            return Ok(false);
        }
        self.1.remove(id);
        record_reply(store, node)
    }
}

/// Settle a `sent` action from its `iq` reply. Returns `true` when the node
/// matched a queued action (the caller then treats it as consumed).
/// Caller must own a live request callback or explicit captured-reply
/// reconciliation authority; a durable Sent row alone is not a live callback.
/// The normal driver enforces this through LiveReplies.
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
        // A damaged saved request must not settle a different row merely
        // because the incoming ID matches that row's key.
        let request = decode_owned(id, &row.wire)?;
        let kind = node.get_attr("type").and_then(|v| v.as_text());
        // APK43.72 1rA.A03 selects result -> JAj.A02; that About callback
        // ignores the response payload. Do not reinterpret an extension child
        // as a refusal, or extend this producer-specific rule to app-state.
        let about_result = kind == Some("result")
            && row.kind == super::about::ACTION_KIND
            && request.get_attr("xmlns").and_then(|v| v.as_text()) == Some("status")
            && request.get_attr("type").and_then(|v| v.as_text()) == Some("set")
            && matches!(request.child_nodes(), [status] if status.description == "status");
        // An `iq type=result` can still carry a refusal further in. App-state
        // answers per collection: `<sync><collection type="error"><error
        // code="409" text="conflict"/>`, observed live 2026-09-24. Reading only
        // the outer type recorded that as success, which is worse than an error
        // — it is an error the caller is told is fine.
        let inner_error = if about_result {
            None
        } else {
            node.child_nodes()
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
                .or_else(|| node.child("error"))
        };
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
    fn queued_create_rechecks_restriction_at_every_reservation_boundary() {
        let (dir, store) = store();
        let peer = crate::model::jid::Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let create = super::super::groups::queue_create(&store, "Lab", &[peer]).unwrap();
        let original = request(&store, &create).unwrap().unwrap();
        let original_row: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key(&create).as_bytes(), 1))
            .unwrap()
            .unwrap();
        let set_active = |store: &ProtocolStore, active: bool| {
            let update = Node::new("notification")
                .attr("type", "mex")
                .attr("id", "restriction-fixture")
                .attr("from", "s.whatsapp.net")
                .children(vec![
                    Node::new("update")
                        .attr("op_name", "NotificationUserReachoutTimelockUpdate")
                        .bytes(
                            serde_json::to_vec(&serde_json::json!({"data": {
                                "xwa2_notify_account_reachout_timelock": {
                                    "is_active": active, "time_enforcement_ends": "1"
                                }
                            }}))
                            .unwrap(),
                        ),
                ]);
            super::super::notifications::record_reachout_timelock(store, &update).unwrap();
        };
        set_active(&store, true);
        let other = queue(&store, "unrelated-read", "fixture", |id| {
            Ok(Node::new("iq").attr("id", id).attr("type", "get"))
        })
        .unwrap();
        let selected = reserve_pending(&store).unwrap();
        assert_eq!(selected, vec![request(&store, &other).unwrap().unwrap()]);
        assert!(reserve_next(&store).unwrap().is_none());
        let error = store.write(|tx| reserve_one_in(tx, &create)).unwrap_err();
        assert!(error.to_string().contains("stays queued"));
        let held_row: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key(&create).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            held_row, original_row,
            "no reservation or intent rewrite while held"
        );
        let reopen = || {
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("p.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [9; 32],
                ))),
                create: false,
            })
            .unwrap()
        };
        drop(store);
        let store = reopen();
        assert!(reserve_pending(&store).unwrap().is_empty());
        let reopened_row: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key(&create).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            reopened_row, original_row,
            "reopen preserves held intent and restriction"
        );
        set_active(&store, false);
        assert_eq!(reserve_next(&store).unwrap(), Some(original));
        let row: Row = store
            .read(|tx| tx.get_json(Table::Checkpoints, key(&create).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(row.status, ActionStatus::Sent);
        assert!(row.sent_at.is_some());
        // Reopening never turns an uncertain sent action back into a queued one.
        drop(store);
        let store = reopen();
        let reopened_sent: Row = store
            .read(|tx| tx.get_json(Table::Checkpoints, key(&create).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&reopened_sent).unwrap(),
            serde_json::to_value(&row).unwrap()
        );
        assert!(reserve_next(&store).unwrap().is_none());
        // Later restrictions/releases never replay an already-reserved request.
        set_active(&store, true);
        assert!(reserve_pending(&store).unwrap().is_empty());
        set_active(&store, false);
        assert!(reserve_pending(&store).unwrap().is_empty());
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

    #[tokio::test(start_paused = true)]
    async fn initial_abprops_timer_claims_once_from_before_write() {
        use std::time::Duration;
        use tokio::time::{Instant, advance};

        for outcome in ["expire", "result", "error", "overdue_response"] {
            let (_dir, store) = store();
            let id = super::super::abprops::queue(&store).unwrap();
            let request = reserve_next(&store).unwrap().unwrap();
            let generic = queue(&store, "fixture", "fixture", ping).unwrap();
            let generic_request = reserve_next(&store).unwrap().unwrap();
            let started = Instant::now();
            advance(Duration::from_secs(5)).await;
            let mut live = LiveReplies::default();
            live.record_written(&request, started);
            live.record_written(&generic_request, started);
            assert_eq!(live.deadline(), Some(started + Duration::from_secs(32)));
            advance(Duration::from_secs(26)).await;
            live.expire();
            assert!(live.0.contains(&id));
            let reply = Node::new("iq").attr("id", id.as_str()).attr(
                "type",
                if outcome == "error" {
                    "error"
                } else {
                    "result"
                },
            );
            if outcome == "expire" {
                let before = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                advance(Duration::from_secs(1)).await;
                live.expire();
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    before
                );
            } else {
                if outcome == "overdue_response" {
                    // Nominal time alone is not the native expiry claim: a
                    // reply may cancel a timer not yet dispatched by the loop.
                    advance(Duration::from_secs(2)).await;
                }
                assert!(live.record_reply(&store, &reply).unwrap());
                assert!(!live.record_reply(&store, &reply).unwrap());
                advance(Duration::from_secs(32)).await;
                live.expire();
            }
            assert!(live.deadline().is_none());
            assert!(!live.0.contains(&id));
            // No universal32s policy: an unrelated live IQ still owns its reply.
            assert!(
                live.record_reply(
                    &store,
                    &Node::new("iq")
                        .attr("id", generic.as_str())
                        .attr("type", "result")
                )
                .unwrap()
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn block_changes_expire_at_twenty_seconds_but_fetch_has_no_timer() {
        use super::super::blocklist;
        use crate::model::jid::Jid;
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        for block in [false, true] {
            for settle in [false, true] {
                let (_dir, store) = store();
                let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
                let id = if block {
                    blocklist::queue_block(&store, &peer)
                } else {
                    blocklist::queue_unblock(&store, &peer)
                }
                .unwrap();
                let request = reserve_next(&store).unwrap().unwrap();
                let fetch = blocklist::queue_fetch(&store).unwrap();
                let fetch_request = reserve_next(&store).unwrap().unwrap();
                let started = Instant::now();
                let mut live = LiveReplies::default();
                advance(Duration::from_secs(3)).await;
                live.record_written(&request, started);
                live.record_written(&fetch_request, started);
                assert_eq!(live.deadline(), Some(started + Duration::from_secs(20)));
                advance(Duration::from_secs(16)).await;
                live.expire();
                assert!(live.0.contains(&id));
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", if block { "result" } else { "error" });
                if settle {
                    assert!(live.record_reply(&store, &reply).unwrap());
                    assert!(live.deadline().is_none());
                }
                advance(Duration::from_secs(1)).await;
                live.expire();
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert_eq!(
                    outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == id)
                        .unwrap()
                        .status,
                    if !settle {
                        ActionStatus::Sent
                    } else if block {
                        ActionStatus::Ok
                    } else {
                        ActionStatus::Error
                    }
                );
                assert!(live.deadline().is_none());
                advance(Duration::from_secs(40)).await;
                live.expire();
                assert!(
                    live.record_reply(
                        &store,
                        &Node::new("iq")
                            .attr("id", fetch.as_str())
                            .attr("type", "result")
                    )
                    .unwrap()
                );
                assert!(blocklist::blocked(&store).unwrap().is_empty());
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn default_timer_expiry_is_independent_of_other_request_deadlines() {
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        for read in [false, true] {
            for outcome in ["expire", "result", "error"] {
                let (_dir, store) = store();
                let id = if read {
                    queue(&store, "dm-read-fixture", "default read", |id| {
                        Ok(Node::new("iq")
                            .attr("id", id)
                            .attr("type", "get")
                            .attr("xmlns", "disappearing_mode")
                            .attr("to", crate::model::jid::Jid::parse("s.whatsapp.net")?))
                    })
                } else {
                    super::super::chat_settings::queue_default_timer(&store, 86400)
                }
                .unwrap();
                let request = reserve_next(&store).unwrap().unwrap();
                let props = super::super::abprops::queue(&store).unwrap();
                let props_request = reserve_next(&store).unwrap().unwrap();
                let started = Instant::now();
                advance(Duration::from_secs(2)).await;
                let mut live = LiveReplies::default();
                live.record_written(&request, started);
                live.record_written(&props_request, started);
                assert_eq!(live.deadline(), Some(started + Duration::from_secs(20)));
                advance(Duration::from_secs(17)).await;
                live.expire();
                assert!(live.0.contains(&id));
                let reply = Node::new("iq").attr("id", id.as_str()).attr(
                    "type",
                    if outcome == "error" {
                        "error"
                    } else {
                        "result"
                    },
                );
                if outcome != "expire" {
                    assert!(live.record_reply(&store, &reply).unwrap());
                    assert_eq!(live.deadline(), Some(started + Duration::from_secs(32)));
                }
                advance(Duration::from_secs(1)).await;
                live.expire();
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert!(live.0.contains(&props));
                assert_eq!(live.deadline(), Some(started + Duration::from_secs(32)));
                assert_eq!(
                    outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == id)
                        .unwrap()
                        .status,
                    match outcome {
                        "expire" => ActionStatus::Sent,
                        "error" => ActionStatus::Error,
                        _ => ActionStatus::Ok,
                    }
                );
                advance(Duration::from_secs(12)).await;
                live.expire();
                assert!(!live.0.contains(&props));
                assert!(live.deadline().is_none());
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn about_ui_wait_and_outer_reply_type_keep_their_native_meaning() {
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        // APK43.72 SetAboutInfo.A5f posts UI message0 after32000ms, but
        // K57 selector1 does not remove the shared1rj/1rD callback map.
        for text in ["", "Available"] {
            for outcome in ["result", "error"] {
                let (_dir, store) = store();
                let id = super::super::about::queue_set(&store, text).unwrap();
                let request = reserve_next(&store).unwrap().unwrap();
                let abprops = super::super::abprops::queue(&store).unwrap();
                let props_request = reserve_next(&store).unwrap().unwrap();
                let mut live = LiveReplies::default();
                let started = Instant::now();
                live.record_written(&request, started);
                live.record_written(&props_request, started);
                advance(Duration::from_secs(32)).await;
                live.expire();
                assert!(live.0.contains(&id));
                assert!(!live.0.contains(&abprops));
                assert!(live.deadline().is_none());
                advance(Duration::from_secs(28)).await;
                live.expire();
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", outcome)
                    .children(vec![Node::new("error").attr("code", "403")]);
                assert!(live.record_reply(&store, &reply).unwrap());
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert_eq!(
                    outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == id)
                        .unwrap()
                        .status,
                    if outcome == "result" {
                        ActionStatus::Ok
                    } else {
                        ActionStatus::Error
                    }
                );
                assert_eq!(
                    outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == id)
                        .unwrap()
                        .error_code,
                    if outcome == "result" { None } else { Some(403) }
                );
            }
        }
    }

    #[test]
    fn live_reply_claims_do_not_resurrect_from_durable_rows() {
        for outcome in ["result", "error", "disconnect", "local_failure"] {
            let (dir, store) = store();
            let id = queue(&store, "fixture", "fixture", ping).unwrap();
            let request = reserve_next(&store).unwrap().unwrap();
            let reply = Node::new("iq").attr("id", id.as_str()).attr(
                "type",
                if outcome == "error" {
                    "error"
                } else {
                    "result"
                },
            );
            let mut live = LiveReplies::default();
            assert!(
                !live.record_reply(&store, &reply).unwrap(),
                "reservation alone owns no callback"
            );
            live.record_written(&request, tokio::time::Instant::now());
            for wrong in [
                Node::new("iq").attr("id", id.as_str()).attr("type", "get"),
                Node::new("iq").attr("id", id.as_str()).attr("type", "set"),
                Node::new("iq").attr("id", id.as_str()),
                Node::new("message")
                    .attr("id", id.as_str())
                    .attr("type", "result"),
                Node::new("iq").attr("id", "OTHER").attr("type", "result"),
            ] {
                assert!(!live.record_reply(&store, &wrong).unwrap());
                assert!(live.0.contains(&id));
            }
            if outcome == "disconnect" {
                let before = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                drop(live);
                drop(store);
                let reopened = ProtocolStore::open(OpenOptions {
                    path: dir.path().join("p.redb"),
                    account_id: AccountId::new("a").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                    create: false,
                })
                .unwrap();
                assert!(
                    !LiveReplies::default()
                        .record_reply(&reopened, &reply)
                        .unwrap()
                );
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    before
                );
            } else if outcome == "local_failure" {
                store
                    .write(|tx| tx.put_json(Table::Checkpoints, SETTLEMENT_SEQUENCE, 1, &0u64))
                    .unwrap();
                assert!(live.record_reply(&store, &reply).is_err());
                assert!(!live.0.contains(&id));
                store
                    .write(|tx| tx.delete(Table::Checkpoints, SETTLEMENT_SEQUENCE))
                    .unwrap();
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert_eq!(outcomes(&store).unwrap()[0].status, ActionStatus::Sent);
            } else {
                assert!(live.record_reply(&store, &reply).unwrap());
                let before = outcomes(&store).unwrap();
                assert_eq!(
                    before[0].status,
                    if outcome == "error" {
                        ActionStatus::Error
                    } else {
                        ActionStatus::Ok
                    }
                );
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert_eq!(outcomes(&store).unwrap(), before);
            }
        }
    }

    #[test]
    fn saved_wire_identity_mismatch_never_reserves_or_settles() {
        let (_dir, store) = store();
        let other = queue(&store, "fixture", "fixture", ping).unwrap();
        store.write(|tx| reserve_one_in(tx, &other)).unwrap();
        let id = queue(&store, "fixture", "fixture", ping).unwrap();
        let original: Row = store
            .read(|tx| tx.get_json(Table::Checkpoints, key(&id).as_bytes(), 1))
            .unwrap()
            .unwrap();
        for wrong in [
            Node::new("iq")
                .attr("id", other.as_str())
                .attr("type", "get"),
            Node::new("message").attr("id", id.as_str()),
            Node::new("iq"),
        ] {
            let mut row = original.clone();
            row.wire = encode(&wrong).unwrap();
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &row))
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            for error in [
                reserve_pending(&store).unwrap_err(),
                reserve_next(&store).unwrap_err(),
                store.write(|tx| reserve_one_in(tx, &id)).unwrap_err(),
            ] {
                assert!(
                    error
                        .to_string()
                        .contains("saved action wire does not match")
                );
                assert!(!error.to_string().contains(&id));
                assert!(!error.to_string().contains(&other));
            }
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
            row.status = ActionStatus::Sent;
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &row))
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert!(
                record_reply(
                    &store,
                    &Node::new("iq")
                        .attr("id", id.as_str())
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
