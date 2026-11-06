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
    /// A correlated result without a recognized producer-specific refusal.
    /// This does not establish payload validation or feature-state adoption.
    Ok,
    /// A correlated outer error or recognized producer-specific inner refusal.
    Error,
}

/// Local pre-write deferral; distinct from a server refusal or uncertain send.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionDeferReason {
    /// No saved trusted-contact scheduling state exists.
    PrivacyTokenStateMissing,
    /// Saved time is older than the APK's mandatory182-day lower bound.
    PrivacyTokenExpired,
    /// Another action owns this user's current token schedule.
    PrivacyTokenSuperseded,
    /// A queued contact check lacks its original error-correlation record.
    ContactCheckCorrelationMissing,
}

/// Stored row.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Row {
    kind: String,
    label: String,
    wire: Vec<u8>,
    status: ActionStatus,
    created_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deferred_reason: Option<ActionDeferReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token_issue_completion: Option<super::token_issue::TokenIssueCompletion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    contact_projection: Option<super::contacts::ContactProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    usync_refusal: Option<super::contacts::UsyncRefusal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    contact_protocol_refusal: Option<super::contacts::UsyncRefusal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    about_refusal: Option<super::about::AboutRefusal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blocklist_error_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    photo_error: Option<super::profile_picture::PhotoErrorOutcome>,
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
    /// Last local reservation deferral; cleared when the action is reserved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deferred_reason: Option<ActionDeferReason>,
    /// Local timestamp effect recorded atomically with a trusted-contact result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_issue_completion: Option<super::token_issue::TokenIssueCompletion>,
    /// Local contact-status projection result; distinct from the outer IQ outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_projection: Option<super::contacts::ContactProjection>,
    /// Parsed usync outer error; this does not activate a backoff or retry policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usync_refusal: Option<super::contacts::UsyncRefusal>,
    /// Contact protocol error inside an outer result; separate from per-user contact type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_protocol_refusal: Option<super::contacts::UsyncRefusal>,
    /// Ordered native About error codes; legacy absent metadata remains unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about_refusal: Option<super::about::AboutRefusal>,
    /// Native signed blocklist outer-error code; zero includes parser fallback.
    /// Absent for other producers/results and legacy rows; no retry authority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocklist_error_code: Option<i32>,
    /// Native photo error callback diagnostic; legacy absence remains unknown.
    /// Empty lookup metadata is not a cache adoption or retry authorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_error: Option<super::profile_picture::PhotoErrorOutcome>,
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
        deferred_reason: None,
        token_issue_completion: None,
        contact_projection: None,
        usync_refusal: None,
        contact_protocol_refusal: None,
        about_refusal: None,
        blocklist_error_code: None,
        photo_error: None,
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

/// Save contact projection diagnostics in the same transaction as data/marker.
/// Does not change request bytes, settlement order or callback ownership.
pub(crate) fn record_contact_projection_in(
    tx: &mut crate::store::WriteTx<'_>,
    id: &str,
    projection: super::contacts::ContactProjection,
) -> Result<()> {
    let selector = key(id);
    let mut row: Row = tx
        .get_json(Table::Checkpoints, selector.as_bytes(), 1)?
        .ok_or_else(|| Error::invalid("contacts", "projection action is missing"))?;
    if row.kind != super::contacts::ACTION_KIND
        || !matches!(row.status, ActionStatus::Ok | ActionStatus::Error)
    {
        return Err(Error::invalid(
            "contacts",
            "projection requires a settled contact action",
        ));
    }
    row.contact_projection = Some(projection);
    tx.put_json(Table::Checkpoints, selector.as_bytes(), 1, &row)
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
            deferred_reason: row.deferred_reason,
            token_issue_completion: row.token_issue_completion,
            contact_projection: row.contact_projection,
            usync_refusal: row.usync_refusal,
            contact_protocol_refusal: row.contact_protocol_refusal,
            about_refusal: row.about_refusal,
            blocklist_error_code: row.blocklist_error_code,
            photo_error: row.photo_error,
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
            deferred_reason: row.deferred_reason,
            token_issue_completion: row.token_issue_completion,
            contact_projection: row.contact_projection,
            usync_refusal: row.usync_refusal,
            contact_protocol_refusal: row.contact_protocol_refusal,
            about_refusal: row.about_refusal,
            blocklist_error_code: row.blocklist_error_code,
            photo_error: row.photo_error,
            error_code: row.error_code,
            error_text: row.error_text,
        },
        reply,
    )))
}

// Complete, identity-checked producer inventory for legacy token-owner recovery.
pub(crate) fn for_kind_in(
    tx: &crate::store::WriteTx<'_>,
    kind: &str,
) -> Result<Vec<(String, ActionStatus, Node)>> {
    let mut out = Vec::new();
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        let done = rows.len() < 256;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows
            .iter()
            .filter(|(key, _)| key.starts_with(PREFIX.as_bytes()))
        {
            let Some(row): Option<Row> = tx.get_json(Table::Checkpoints, key, 1)? else {
                continue;
            };
            if row.kind != kind {
                continue;
            }
            let id = std::str::from_utf8(&key[PREFIX.len()..])
                .map_err(|_| Error::invalid("action", "invalid saved action row identity"))?;
            out.push((id.to_owned(), row.status, decode_owned(id, &row.wire)?));
        }
        if done {
            return Ok(out);
        }
    }
}

fn prepare_for_reservation_in(
    tx: &mut crate::store::WriteTx<'_>,
    id: &str,
    row: &mut Row,
    node: Node,
) -> Result<Option<Node>> {
    if row.kind == super::contacts::ACTION_KIND {
        if !super::contacts::check_correlation_in(tx, id, &node)? {
            let reason = ActionDeferReason::ContactCheckCorrelationMissing;
            if row.deferred_reason != Some(reason) {
                row.deferred_reason = Some(reason);
                tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, row)?;
                tracing::info!(?reason, "contact check deferred before reservation");
            }
            return Ok(None);
        }
        row.deferred_reason = None;
    }
    if row.kind != super::token_issue::ACTION_KIND {
        return Ok(Some(node));
    }
    let now = super::clock::a04_now_in_write(tx, Timestamp::now().as_millis())? / 1000;
    match super::token_issue::prepare_in(tx, id, &node, now)? {
        super::token_issue::IssueReservation::Ready(prepared) => {
            row.wire = encode(&prepared)?;
            row.deferred_reason = None;
            Ok(Some(prepared))
        }
        super::token_issue::IssueReservation::Deferred(reason) => {
            if row.deferred_reason != Some(reason) {
                row.deferred_reason = Some(reason);
                tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, row)?;
                tracing::info!(
                    ?reason,
                    "trusted-contact action deferred before reservation"
                );
            }
            Ok(None)
        }
    }
}

/// Reserve at most sixteen queued actions for one socket write, marking them
/// `sent` before the write so a crash never resends them.
pub fn reserve_pending(store: &ProtocolStore) -> Result<Vec<Node>> {
    Ok(reserve_limit_owned(store, BATCH)?
        .into_iter()
        .map(|action| action.node)
        .collect())
}

/// Reserve only the next eligible action for the connection actor. All bulk
/// admission guards remain shared; later actions stay queued until their turn.
#[cfg(test)]
pub(crate) fn reserve_next(store: &ProtocolStore) -> Result<Option<Node>> {
    Ok(reserve_next_owned(store)?.map(|action| action.node))
}

pub(crate) struct ReservedAction {
    pub node: Node,
    pub started: tokio::time::Instant,
    pub guard: ActionWriteGuard,
}

#[derive(Clone)]
pub(crate) struct ActionWriteGuard {
    key: Vec<u8>,
    row: zeroize::Zeroizing<Vec<u8>>,
    deadline: Option<tokio::time::Instant>,
}
impl ActionWriteGuard {
    pub(crate) fn check_write_authority(&self, store: &ProtocolStore) -> Result<()> {
        store.read(|tx| {
            let current = zeroize::Zeroizing::new(
                tx.get(Table::Checkpoints, &self.key, 1)?
                    .unwrap_or_default(),
            );
            if *current != *self.row {
                return Err(Error::invalid(
                    "action",
                    "reserved action changed before write",
                ));
            }
            #[derive(Deserialize)]
            struct AdmissionKind {
                kind: String,
            }
            let row: AdmissionKind = serde_json::from_slice(&current)
                .map_err(|_| Error::invalid("action", "invalid reserved action metadata"))?;
            // Reuse reservation admission at the last local write boundary:
            // a restriction/backoff notification can arrive during queueing.
            if row.kind == "group-create"
                && super::notifications::recorded_active_reachout_timelock_read_in(tx)?.is_some()
            {
                return Err(Error::invalid(
                    "action",
                    "recorded reach-out restriction blocks queued group creation",
                ));
            }
            if matches!(
                row.kind.as_str(),
                super::app_state_sync::FETCH_ACTION_KIND
                    | super::app_state_sync::PATCH_ACTION_KIND
                    | super::app_state_sync::HELD_FETCH_ACTION_KIND
            ) && super::app_state_sync::server_waiting_read_in(tx)?
            {
                return Err(Error::invalid(
                    "action",
                    "server app-state backoff blocks queued action",
                ));
            }
            if self
                .deadline
                .is_some_and(|deadline| tokio::time::Instant::now() >= deadline)
            {
                return Err(Error::invalid(
                    "action",
                    "reserved action expired before write",
                ));
            }
            Ok(())
        })
    }
}

pub(crate) fn reserve_next_owned(store: &ProtocolStore) -> Result<Option<ReservedAction>> {
    Ok(reserve_limit_owned(store, 1)?.pop())
}

fn reserve_limit_owned(store: &ProtocolStore, limit: usize) -> Result<Vec<ReservedAction>> {
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
                let Some(node) = prepare_for_reservation_in(tx, id, &mut row, node)? else {
                    continue;
                };
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
                row.status = ActionStatus::Sent;
                row.sent_at = Some(Timestamp::now());
                tx.put_json(Table::Checkpoints, key, 1, &row)?;
                let started = tokio::time::Instant::now();
                let deadline = callback_seconds(&node)
                    .map(|seconds| started + std::time::Duration::from_secs(seconds));
                let row = tx
                    .get(Table::Checkpoints, key, 1)?
                    .ok_or_else(|| Error::invalid("action", "reserved action missing"))?;
                out.push(ReservedAction {
                    node,
                    started,
                    guard: ActionWriteGuard {
                        key: key.clone(),
                        row: zeroize::Zeroizing::new(row),
                        deadline,
                    },
                });
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
    let node = prepare_for_reservation_in(tx, id, &mut row, node)?.ok_or_else(|| {
        Error::invalid(
            "action",
            "selected action is locally deferred; no request reserved",
        )
    })?;
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
    std::collections::HashMap<String, ActionWriteGuard>,
);

fn callback_seconds(node: &Node) -> Option<u64> {
    // APK43.72 HBE.A00 -> 0Sf.A0W/A0A -> 0ln.A08 registers the initial
    // AB-properties callback for32000ms before dispatch. Other producers'
    // durations are selected separately below, not inferred from that request.
    if node.get_attr("type").and_then(|v| v.as_text()) == Some("get")
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
    } else if super::two_factor::is_set_request(node) {
        // APK43.72 1HL.A07 017c..018a -> 0Sf.A0W: 32000ms
        // from registration, not the registration HTTP PIN flow.
        Some(32)
    } else if super::report::is_request(node) {
        // APK43.72 SpamRPCManager.A01 -> MessageClientSmaxWrapper.A00
        // passes32000ms to0Sf.A0F; Cgd selector2 resumes timeout as BWM.
        // docs/protocol/apk/2.26.43.72-report-callback-timer/README.md.
        // Expiry drops the live claim, never the durable uncertain intent.
        Some(32)
    } else if super::identity_query::is_request(node) {
        // APK43.72 FeM.run case16 187a..1888 -> 0Sf.A0W:
        // identity-only callback uses32000ms, not a universal key timer.
        Some(32)
    } else if super::token_issue::is_issue_request(node) {
        // APK43.72 GeneratePrivacyTokenJob.A0L 0144..015c:
        // trusted-contact request299 has its own32000ms timer.
        Some(32)
    } else {
        None
    }
}

impl LiveReplies {
    pub(crate) fn record_written(&mut self, action: &ReservedAction) -> Result<()> {
        let node = &action.node;
        if node.description != "iq" {
            return Ok(());
        }
        let id = node
            .get_attr("id")
            .and_then(|value| value.as_text())
            .ok_or_else(|| Error::invalid("action", "written action lacks callback id"))?;
        if action.guard.key != key(id).as_bytes() {
            return Err(Error::invalid(
                "action",
                "written action callback owner does not match id",
            ));
        }
        let deadline = callback_seconds(node)
            .map(|seconds| action.started + std::time::Duration::from_secs(seconds));
        if deadline != action.guard.deadline {
            return Err(Error::invalid(
                "action",
                "written action callback timer does not match reservation",
            ));
        }
        if self.0.contains(id) {
            return Err(Error::invalid(
                "action",
                "action callback id already active",
            ));
        }
        self.0.insert(id.to_owned());
        self.2.insert(id.to_owned(), action.guard.clone());
        // Same reservation-time base as writer admission; callback completion
        // does not start a fresh duration after the socket write.
        if let Some(deadline) = action.guard.deadline {
            self.1.insert(id.to_owned(), deadline);
        }
        Ok(())
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
                self.2.remove(id);
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
        let owner = self
            .2
            .remove(id)
            .ok_or_else(|| Error::invalid("action", "live callback lacks written reservation"))?;
        record_reply_inner(store, node, Some(&owner))
    }
}

/// Settle a `sent` action from its `iq` reply. Returns `true` when the node
/// matched a queued action (the caller then treats it as consumed).
/// Caller must own a live request callback or explicit captured-reply
/// reconciliation authority; a durable Sent row alone is not a live callback.
/// The normal driver enforces this through LiveReplies.
pub fn record_reply(store: &ProtocolStore, node: &Node) -> Result<bool> {
    record_reply_inner(store, node, None)
}

fn record_reply_inner(
    store: &ProtocolStore,
    node: &Node,
    owner: Option<&ActionWriteGuard>,
) -> Result<bool> {
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
        if let Some(owner) = owner {
            let current = zeroize::Zeroizing::new(
                tx.get(Table::Checkpoints, key.as_bytes(), 1)?
                    .unwrap_or_default(),
            );
            if owner.key != key.as_bytes() || *current != *owner.row {
                return Err(Error::invalid(
                    "action",
                    "written action changed before reply settlement",
                ));
            }
        }
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
        let photo_error = super::profile_picture::error_outcome(&row.kind, &request, node);
        // APK43.72 1rA.A03 selects result -> JAj.A02; that About callback
        // ignores the response payload. Do not reinterpret an extension child
        // as a refusal, or extend this producer-specific rule to app-state.
        let about_result = kind == Some("result")
            && row.kind == super::about::ACTION_KIND
            && super::about::is_set_request(&request);
        // APK43.72 Hyd.CK4 for a nonnull upload validates the first child
        // as picture, reads its attributes and ignores subsequent extensions.
        // Missing/wrong first child and lookup/removal policies remain separate.
        let photo_set_result = kind == Some("result")
            && row.kind == super::profile_picture::SET_KIND
            && request.get_attr("xmlns").and_then(|v| v.as_text()) == Some("w:profile:picture")
            && request.get_attr("type").and_then(|v| v.as_text()) == Some("set")
            && matches!(request.child_nodes(), [picture]
                if picture.description == "picture"
                    && picture.get_attr("type").and_then(|v| v.as_text()) == Some("image")
                    && matches!(&picture.content, crate::binary::NodeContent::Bytes(_)))
            && node
                .child_nodes()
                .first()
                .is_some_and(|child| child.description == "picture");
        // APK43.72 6p1.CK4 selects picture by name, otherwise empty metadata;
        // it never searches error extensions. This is outer callback dispatch,
        // not validation of URL/id or native photo/cache adoption.
        let photo_lookup_result = kind == Some("result")
            && row.kind == super::profile_picture::GET_KIND
            && super::profile_picture::is_lookup_request(&request);
        // APK43.72 FUS.CK4 schedules the originally requested values without
        // reading response payload. Preserve outer-type dispatch for this
        // typed producer, independently of unrelated nested-refusal parsers.
        let two_factor_result = kind == Some("result")
            && row.kind == super::two_factor::ACTION_KIND
            && super::two_factor::is_set_request(&request);
        // APK43.72 8R8.CK4 selector3 uses captured user/timestamp and
        // resolves the job future without parsing the response payload.
        let privacy_token_result = kind == Some("result")
            && row.kind == super::token_issue::ACTION_KIND
            && super::token_issue::is_issue_request(&request);
        // APK43.72 8R7/6p9 select CK4 from the outer result, then parse
        // their list. Unrelated error extensions cannot select Bvj instead.
        let blocklist_reply = matches!(row.kind.as_str(), "blocklist" | "block" | "unblock")
            && super::blocklist::require_request(&request, id, &row.kind).is_ok();
        // APK43.72 1rA.A03 dispatches result to Dxh.A02, which reads
        // per-user errors under list, not root extensions. Keep this scoped to
        // the typed peer directory producer; app-state has different semantics.
        let peer_directory_reply = matches!(kind, Some("result" | "error"))
            && row.kind == "peer_keys"
            && request.get_attr("xmlns").and_then(|v| v.as_text()) == Some("encrypt")
            && request.get_attr("type").and_then(|v| v.as_text()) == Some("get")
            && request.child("key").is_some();
        // APK43.72 FeM.run case16 binds identity/user to FUP case0;
        // 0ln.A01 -> CK4 selects list/user errors instead of root extensions.
        let identity_only_reply = matches!(kind, Some("result" | "error"))
            && row.kind == "peer_identity_query"
            && request.get_attr("xmlns").and_then(|v| v.as_text()) == Some("encrypt")
            && request.get_attr("type").and_then(|v| v.as_text()) == Some("get")
            && request.child("identity").is_some();
        // APK43.72 0v0.A00 binds identityJids/reason=identity key reads
        // to Dxh too. Preserve its selected-device errors and outer dispatch.
        let identity_refresh_peer =
            if matches!(kind, Some("result" | "error")) && row.kind == "identity_refresh" {
                Some(super::identity_notifications::request_peer(&request, id)?)
            } else {
                None
            };
        // An `iq type=result` can still carry a refusal further in. App-state
        // answers per collection: `<sync><collection type="error"><error
        // code="409" text="conflict"/>`, observed live 2026-09-24. Reading only
        // the outer type recorded that as success, which is worse than an error
        // — it is an error the caller is told is fine.
        let inner_error = if about_result
            || photo_set_result
            || photo_lookup_result
            || two_factor_result
            || privacy_token_result
            || (blocklist_reply && kind == Some("result"))
        {
            None
        } else if (blocklist_reply || photo_error.is_some()) && kind == Some("error") {
            // APK43.72 1ud.A01 scans direct error children, skipping
            // missing code. It returns the first coded error, not a nested
            // user error or an earlier uncoded extension (8R7/6p9.Bvj).
            // Hyd.Bvj and 6p1.Bvj select the same 1ud parser for photos.
            node.child_nodes().iter().find(|child| {
                child.description == "error"
                    && child.get_attr("code").and_then(|v| v.as_text()).is_some()
            })
        } else if kind == Some("error")
            && (peer_directory_reply || identity_only_reply || identity_refresh_peer.is_some())
        {
            // APK43.72 1rA.A03 and 0ln.A01 select outer-error
            // callbacks. Nested device errors are not this query's code.
            let mut errors = node
                .child_nodes()
                .iter()
                .filter(|child| child.description == "error");
            let error = errors.next();
            errors.next().is_none().then_some(error).flatten()
        } else if peer_directory_reply {
            // Bind refusal to the original single-peer request. A foreign
            // user's error must not poison this peer's completion/refresh.
            crate::signal::direct::child(&request, "key")
                .ok()
                .and_then(|key| crate::signal::direct::child(key, "user").ok())
                .and_then(|user| super::receipts::jid(user, "jid"))
                .and_then(|peer| super::peer_keys::selected_user(node, &peer).ok())
                .and_then(|user| user.child("error"))
        } else if let Some(peer) = identity_refresh_peer.as_ref() {
            super::peer_keys::selected_user(node, peer)
                .ok()
                .and_then(|user| user.child("error"))
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
                .or_else(|| {
                    (!peer_directory_reply && !identity_only_reply)
                        .then(|| node.child("error"))
                        .flatten()
                })
        };
        if kind == Some("result") && inner_error.is_none() {
            if privacy_token_result {
                row.token_issue_completion =
                    Some(super::token_issue::record_success_in(tx, id, &request)?);
            }
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
        // APK43.72 8R7/6p9.Bvj -> 1ud.A00/A01 -> 0HF.A00(code,0):
        // Java Integer.parseInt, signed32 with zero on missing/invalid code.
        // Keep compatibility u16 diagnostics separate; neither field retries.
        row.blocklist_error_code = (blocklist_reply && kind == Some("error")).then(|| {
            inner_error
                .and_then(|error| error.get_attr("code"))
                .and_then(|value| value.as_text())
                .and_then(super::mex::java_decimal_long)
                .and_then(|value| i32::try_from(value).ok())
                .unwrap_or(0)
        });
        row.photo_error = photo_error;
        row.about_refusal = (row.kind == super::about::ACTION_KIND)
            .then(|| super::about::refusal(&request, node))
            .flatten();
        row.usync_refusal = super::contacts::outer_usync_refusal(&request, node);
        row.contact_protocol_refusal = super::contacts::contact_protocol_refusal(&request, node);
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
                    deferred_reason: row.deferred_reason,
                    token_issue_completion: row.token_issue_completion,
                    contact_projection: row.contact_projection,
                    usync_refusal: row.usync_refusal,
                    contact_protocol_refusal: row.contact_protocol_refusal,
                    about_refusal: row.about_refusal,
                    blocklist_error_code: row.blocklist_error_code,
                    photo_error: row.photo_error,
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
                deferred_reason: row.deferred_reason,
                token_issue_completion: row.token_issue_completion,
                contact_projection: row.contact_projection,
                usync_refusal: row.usync_refusal,
                contact_protocol_refusal: row.contact_protocol_refusal,
                about_refusal: row.about_refusal,
                blocklist_error_code: row.blocklist_error_code,
                photo_error: row.photo_error,
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
                    deferred_reason: row.deferred_reason,
                    token_issue_completion: row.token_issue_completion,
                    contact_projection: row.contact_projection,
                    usync_refusal: row.usync_refusal,
                    contact_protocol_refusal: row.contact_protocol_refusal,
                    about_refusal: row.about_refusal,
                    blocklist_error_code: row.blocklist_error_code,
                    photo_error: row.photo_error,
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

    fn register_written(
        live: &mut LiveReplies,
        store: &ProtocolStore,
        node: &Node,
        started: tokio::time::Instant,
    ) {
        let id = node.get_attr("id").and_then(|v| v.as_text()).unwrap();
        let key = key(id).into_bytes();
        let row = store
            .read(|tx| tx.get(Table::Checkpoints, &key, 1))
            .unwrap()
            .unwrap();
        let action = ReservedAction {
            node: node.clone(),
            started,
            guard: ActionWriteGuard {
                key,
                row: zeroize::Zeroizing::new(row),
                deadline: callback_seconds(node)
                    .map(|seconds| started + std::time::Duration::from_secs(seconds)),
            },
        };
        live.record_written(&action).unwrap();
    }

    #[tokio::test]
    async fn queued_action_rechecks_later_restrictions_and_server_wait_without_mutation() {
        for producer in [
            "group-create",
            super::super::app_state_sync::FETCH_ACTION_KIND,
            super::super::app_state_sync::PATCH_ACTION_KIND,
            super::super::app_state_sync::HELD_FETCH_ACTION_KIND,
            "unrelated",
        ] {
            for gate in ["none", "active", "expired", "cleared", "malformed"] {
                if producer == "group-create" && gate == "malformed" {
                    continue;
                }
                let (dir, db) = store();
                let db = Arc::new(db);
                let id = match producer {
                    "group-create" => super::super::groups::queue_create(
                        &db,
                        "Offline fixture",
                        &[crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap()],
                    ),
                    super::super::app_state_sync::FETCH_ACTION_KIND => {
                        super::super::app_state_sync::queue_fetch(&db)
                    }
                    _ => queue(&db, "fixture", "fixture", ping),
                }
                .unwrap();
                let action = if matches!(
                    producer,
                    super::super::app_state_sync::PATCH_ACTION_KIND
                        | super::super::app_state_sync::HELD_FETCH_ACTION_KIND
                ) {
                    // These fixtures isolate the shared writer gate; their full
                    // producer/key/sidecar admission is covered in app-state tests.
                    let node = request(&db, &id).unwrap().unwrap();
                    let raw = db
                        .write(|tx| {
                            let mut row: Row = tx
                                .get_json(Table::Checkpoints, key(&id).as_bytes(), 1)?
                                .unwrap();
                            row.kind = producer.into();
                            row.status = ActionStatus::Sent;
                            row.sent_at = Some(Timestamp::now());
                            tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &row)?;
                            Ok(tx.get(Table::Checkpoints, key(&id).as_bytes(), 1)?.unwrap())
                        })
                        .unwrap();
                    ReservedAction {
                        node,
                        started: tokio::time::Instant::now(),
                        guard: ActionWriteGuard {
                            key: key(&id).into_bytes(),
                            row: zeroize::Zeroizing::new(raw),
                            deadline: None,
                        },
                    }
                } else {
                    reserve_next_owned(&db).unwrap().unwrap()
                };
                let (_incoming, incoming) = tokio::sync::mpsc::channel(1);
                let (outgoing, mut writes) = tokio::sync::mpsc::channel(1);
                let link = super::super::connection::NodeLink::new(incoming, outgoing);
                let worker = db.clone();
                let task = tokio::spawn(async move { link.send_action(&worker, &action).await });
                let queued = writes.recv().await.unwrap();
                if gate != "none" {
                    if producer == "group-create" || producer == "unrelated" {
                        let update=Node::new("notification").attr("type","mex").attr("id","restriction-fixture").attr("from","s.whatsapp.net")
                            .children(vec![Node::new("update").attr("op_name","NotificationUserReachoutTimelockUpdate")
                                .bytes(serde_json::to_vec(&serde_json::json!({"data":{"xwa2_notify_account_reachout_timelock":{"is_active":gate!="cleared","time_enforcement_ends":if gate=="active" {"20000000000"} else {"1"}}}})).unwrap())]);
                        super::super::notifications::record_reachout_timelock(&db, &update)
                            .unwrap();
                    }
                    if producer != "group-create" {
                        let until = match gate {
                            "active" => i64::MAX,
                            "malformed" => -1,
                            _ => 1,
                        };
                        db.write(|tx| {
                            tx.put_json(
                                Table::Checkpoints,
                                b"app-state-server-backoff-until",
                                1,
                                &Timestamp(until),
                            )
                        })
                        .unwrap();
                    }
                }
                let before = db.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap();
                let head = db.journal_head().unwrap();
                let admitted = producer == "unrelated"
                    || gate == "none"
                    || gate == "cleared"
                    || (producer != "group-create" && gate == "expired");
                let prepared = queued.prepare();
                assert_eq!(prepared.is_some(), admitted, "{producer}/{gate}");
                if let Some((_node, written)) = prepared {
                    written.send(Ok(())).unwrap();
                }
                assert_eq!(task.await.unwrap().is_ok(), admitted);
                assert!(writes.recv().await.is_none());
                assert_eq!(
                    db.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap(),
                    before
                );
                assert_eq!(db.journal_head().unwrap(), head);
                drop(db);
                let reopened = ProtocolStore::open(OpenOptions {
                    path: dir.path().join("p.redb"),
                    account_id: AccountId::new("a").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                    create: false,
                })
                .unwrap();
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    before
                );
                assert_eq!(reopened.journal_head().unwrap(), head);
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn live_action_reply_keeps_written_row_and_rejects_replacement() {
        use std::time::Duration;
        for response in ["result", "error"] {
            for change in [
                "same", "missing", "status", "wire", "kind", "label", "sent_at", "expire",
            ] {
                let (dir, db) = store();
                let peer = crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap();
                let id = super::super::blocklist::queue_block(&db, &peer).unwrap();
                let action = reserve_next_owned(&db).unwrap().unwrap();
                let mut live = LiveReplies::default();
                // Registration does not reload the durable row to create authority.
                for invalid in [
                    ReservedAction {
                        node: Node::new("iq"),
                        started: action.started,
                        guard: action.guard.clone(),
                    },
                    ReservedAction {
                        node: action.node.clone().attr("id", "OTHER"),
                        started: action.started,
                        guard: action.guard.clone(),
                    },
                    ReservedAction {
                        node: action.node.clone(),
                        started: action.started + Duration::from_millis(1),
                        guard: action.guard.clone(),
                    },
                ] {
                    assert!(live.record_written(&invalid).is_err());
                    assert!(live.0.is_empty() && live.1.is_empty() && live.2.is_empty());
                }
                live.record_written(&action).unwrap();
                assert!(live.record_written(&action).is_err());
                let mut replacement_guard = action.guard.clone();
                replacement_guard.row.push(b' ');
                assert!(
                    live.record_written(&ReservedAction {
                        node: action.node.clone(),
                        started: action.started,
                        guard: replacement_guard
                    })
                    .is_err()
                );
                assert!(live.2[&id].row == action.guard.row);
                assert_eq!(live.deadline(), action.guard.deadline);
                if matches!(
                    change,
                    "missing" | "status" | "wire" | "kind" | "label" | "sent_at"
                ) {
                    db.write(|tx| {
                        if change == "missing" {
                            tx.delete(Table::Checkpoints, key(&id).as_bytes())?;
                            return Ok(());
                        }
                        let mut row: Row = tx
                            .get_json(Table::Checkpoints, key(&id).as_bytes(), 1)?
                            .unwrap();
                        match change {
                            "status" => row.status = ActionStatus::Ok,
                            "wire" => {
                                row.wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                                    .encode(&action.node.clone().attr("xmlns", "changed-fixture"))?
                            }
                            "kind" => row.kind = "changed-fixture".into(),
                            "label" => row.label = "changed-fixture".into(),
                            _ => row.sent_at = Some(Timestamp(1)),
                        }
                        tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &row)
                    })
                    .unwrap();
                }
                if change == "expire" {
                    tokio::time::advance(Duration::from_secs(20)).await;
                    live.expire();
                    assert!(live.0.is_empty() && live.1.is_empty() && live.2.is_empty());
                }
                let before = db.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap();
                let head = db.journal_head().unwrap();
                let mut reply = Node::new("iq")
                    .attr("type", response)
                    .attr("id", id.as_str());
                if response == "error" {
                    reply = reply.children(vec![Node::new("error").attr("code", "403")]);
                }
                let result = live.record_reply(&db, &reply);
                if change == "same" {
                    assert!(result.unwrap());
                } else {
                    if change == "expire" {
                        assert!(!result.unwrap());
                    } else {
                        assert!(result.is_err(), "{response}/{change}");
                    }
                    assert_eq!(
                        db.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap(),
                        before
                    );
                    assert_eq!(db.journal_head().unwrap(), head);
                }
                assert!(live.0.is_empty() && live.1.is_empty() && live.2.is_empty());
                assert!(!live.record_reply(&db, &reply).unwrap());
                let after = db.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap();
                drop(db);
                let reopened = ProtocolStore::open(OpenOptions {
                    path: dir.path().join("p.redb"),
                    account_id: AccountId::new("a").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                    create: false,
                })
                .unwrap();
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    after
                );
                assert!(
                    !LiveReplies::default()
                        .record_reply(&reopened, &reply)
                        .unwrap()
                );
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    after
                );
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn queued_action_write_retains_row_and_original_deadline() {
        use std::time::Duration;
        use tokio::time::advance;
        for (timed, delay_ms, change) in [
            (true, 0, "same"),
            (true, 19999, "same"),
            (true, 20000, "same"),
            (true, 20001, "same"),
            (false, 60000, "same"),
            (true, 0, "missing"),
            (true, 0, "status"),
            (true, 0, "wire"),
            (true, 0, "label"),
            (true, 0, "sent_at"),
            (true, 0, "slow_completion"),
        ] {
            let (dir, db) = store();
            let db = Arc::new(db);
            let peer = crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap();
            let id = if timed {
                super::super::blocklist::queue_block(&db, &peer)
            } else {
                super::super::blocklist::queue_fetch(&db)
            }
            .unwrap();
            let action = reserve_next_owned(&db).unwrap().unwrap();
            let started = action.started;
            assert_eq!(
                action.guard.deadline,
                timed.then_some(started + Duration::from_secs(20))
            );
            let (_incoming, incoming) = tokio::sync::mpsc::channel(1);
            let (outgoing, mut writes) = tokio::sync::mpsc::channel(1);
            let link = super::super::connection::NodeLink::new(incoming, outgoing);
            let worker_db = db.clone();
            let task = tokio::spawn(async move { link.send_action(&worker_db, &action).await });
            let queued = writes.recv().await.unwrap();
            advance(Duration::from_millis(delay_ms)).await;
            if !matches!(change, "same" | "slow_completion") {
                db.write(|tx| {
                    if change == "missing" {
                        tx.delete(Table::Checkpoints, key(&id).as_bytes())?;
                        return Ok(());
                    }
                    let mut row: Row = tx
                        .get_json(Table::Checkpoints, key(&id).as_bytes(), 1)?
                        .unwrap();
                    match change {
                        "status" => row.status = ActionStatus::Ok,
                        "wire" => {
                            let node =
                                decode_owned(&id, &row.wire)?.attr("xmlns", "changed-fixture");
                            row.wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                                .encode(&node)?;
                        }
                        "label" => row.label = "changed-fixture".into(),
                        _ => row.sent_at = Some(Timestamp(1)),
                    }
                    tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &row)
                })
                .unwrap();
            }
            let before = db.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap();
            let head = db.journal_head().unwrap();
            let admitted =
                matches!(change, "same" | "slow_completion") && (!timed || delay_ms < 20000);
            let prepared = queued.prepare();
            assert_eq!(prepared.is_some(), admitted, "{timed}/{delay_ms}/{change}");
            if let Some((node, written)) = prepared {
                assert_eq!(
                    node.get_attr("id").and_then(|v| v.as_text()),
                    Some(id.as_str())
                );
                if change == "slow_completion" {
                    advance(Duration::from_secs(21)).await;
                }
                written.send(Ok(())).unwrap();
                let mut replies = LiveReplies::default();
                register_written(&mut replies, &db, &node, started);
                assert_eq!(
                    replies.deadline(),
                    timed.then_some(started + Duration::from_secs(20))
                );
                if change == "slow_completion" {
                    replies.expire();
                    assert!(replies.deadline().is_none());
                    assert!(replies.0.is_empty());
                }
            }
            assert_eq!(task.await.unwrap().is_ok(), admitted);
            assert!(writes.recv().await.is_none());
            assert_eq!(
                db.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap(),
                before
            );
            assert_eq!(db.journal_head().unwrap(), head);
            drop(db);
            let reopened = ProtocolStore::open(OpenOptions {
                path: dir.path().join("p.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                create: false,
            })
            .unwrap();
            assert_eq!(
                reopened
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
            assert_eq!(reopened.journal_head().unwrap(), head);
        }
    }

    #[test]
    fn directory_outer_errors_never_attribute_nested_device_failures() {
        for producer in ["peer_keys", "peer_identity_query"] {
            for roots in [0, 1, 2] {
                for first in [false, true] {
                    let (dir, db) = store();
                    let peer = crate::model::jid::Jid::parse("15555550100@s.whatsapp.net").unwrap();
                    let id = if producer == "peer_keys" {
                        super::super::peer_keys::queue(&db, &peer, false).unwrap()
                    } else {
                        super::super::identity_query::queue(&db, &peer).unwrap()
                    };
                    reserve_pending(&db).unwrap();
                    let selected = Node::new("user").attr("jid", peer.clone()).children(vec![
                        Node::new("error")
                            .attr("code", "480")
                            .attr("text", "NESTED_PRIVATE"),
                    ]);
                    let list = Node::new("list").children(vec![selected]);
                    let mut children = Vec::new();
                    if first {
                        children.push(list.clone());
                    }
                    if roots > 0 {
                        children.push(
                            Node::new("error")
                                .attr("code", "429")
                                .attr("text", "ROOT_CONTEXT"),
                        );
                    }
                    if roots > 1 {
                        children.push(
                            Node::new("error")
                                .attr("code", "500")
                                .attr("text", "SECOND_ROOT"),
                        );
                    }
                    if !first {
                        children.push(list);
                    }
                    let incoming = Node::new("iq")
                        .attr("id", id.as_str())
                        .attr("type", "error")
                        .children(children);
                    assert!(record_reply(&db, &incoming).unwrap());
                    drop(db);
                    let db = ProtocolStore::open(OpenOptions {
                        path: dir.path().join("p.redb"),
                        account_id: AccountId::new("a").unwrap(),
                        key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                        create: false,
                    })
                    .unwrap();
                    let outcome = outcomes(&db).unwrap().remove(0);
                    assert_eq!(outcome.status, ActionStatus::Error);
                    assert_eq!(
                        outcome.error_code,
                        if roots == 1 { Some(429) } else { None },
                        "{producer}/{roots}/{first}"
                    );
                    assert_eq!(
                        outcome.error_text.as_deref(),
                        if roots == 1 {
                            Some("ROOT_CONTEXT")
                        } else {
                            None
                        }
                    );
                    assert!(
                        !serde_json::to_string(&outcome)
                            .unwrap()
                            .contains("NESTED_PRIVATE")
                    );
                    let before = db
                        .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                        .unwrap();
                    let head = db.journal_head().unwrap();
                    if producer == "peer_keys" {
                        let view = super::super::peer_keys::inspect(&db, &peer)
                            .unwrap()
                            .unwrap();
                        assert!(view.fingerprint.is_none());
                        assert!(super::super::peer_keys::queue(&db, &peer, true).is_err());
                        assert_eq!(
                            super::super::peer_keys::queue(&db, &peer, false).unwrap(),
                            id
                        );
                    } else {
                        let view = super::super::identity_query::inspect(&db, &peer)
                            .unwrap()
                            .unwrap();
                        assert!(view.fingerprint.is_none());
                        assert_eq!(super::super::identity_query::queue(&db, &peer).unwrap(), id);
                    }
                    assert!(reserve_pending(&db).unwrap().is_empty());
                    assert_eq!(reply(&db, &id).unwrap(), Some(incoming));
                    assert_eq!(
                        db.read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                            .unwrap(),
                        before
                    );
                    assert_eq!(db.journal_head().unwrap(), head);
                }
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn identity_only_callback_expires_at_native_thirty_two_seconds_without_replay() {
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        for address in ["15555550100@s.whatsapp.net", "222222222222222@lid"] {
            for outcome in [
                "expire_result",
                "expire_error",
                "result",
                "error",
                "overdue_response",
            ] {
                let (dir, db) = store();
                let peer = crate::model::jid::Jid::parse(address).unwrap();
                let id = super::super::identity_query::queue(&db, &peer).unwrap();
                let request = reserve_next(&db).unwrap().unwrap();
                let key_peer = crate::model::jid::Jid::parse("15555550100@s.whatsapp.net").unwrap();
                let keys = super::super::peer_keys::queue(&db, &key_peer, false).unwrap();
                let key_request = reserve_next(&db).unwrap().unwrap();
                let started = Instant::now();
                advance(Duration::from_secs(5)).await;
                let mut live = LiveReplies::default();
                register_written(&mut live, &db, &request, started);
                register_written(&mut live, &db, &key_request, started);
                assert_eq!(live.deadline(), Some(started + Duration::from_secs(32)));
                advance(Duration::from_secs(26)).await;
                live.expire();
                assert!(live.0.contains(&id));
                let incoming = Node::new("iq").attr("id", id.as_str()).attr(
                    "type",
                    if outcome.ends_with("error") {
                        "error"
                    } else {
                        "result"
                    },
                );
                if outcome.starts_with("expire") {
                    let before = db
                        .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                        .unwrap();
                    advance(Duration::from_secs(1)).await;
                    live.expire();
                    assert!(!live.record_reply(&db, &incoming).unwrap());
                    assert_eq!(
                        db.read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                            .unwrap(),
                        before
                    );
                    assert_eq!(
                        outcomes(&db)
                            .unwrap()
                            .into_iter()
                            .find(|row| row.id == id)
                            .unwrap()
                            .status,
                        ActionStatus::Sent
                    );
                } else {
                    if outcome == "overdue_response" {
                        advance(Duration::from_secs(2)).await;
                    }
                    assert!(live.record_reply(&db, &incoming).unwrap());
                    assert!(!live.record_reply(&db, &incoming).unwrap());
                }
                assert!(live.deadline().is_none());
                assert!(!live.0.contains(&id));
                // Full-key producer remains unmapped; no universal encrypt timeout.
                assert!(
                    live.record_reply(
                        &db,
                        &Node::new("iq")
                            .attr("id", keys.as_str())
                            .attr("type", "error")
                    )
                    .unwrap()
                );
                assert_eq!(super::super::identity_query::queue(&db, &peer).unwrap(), id);
                assert!(reserve_pending(&db).unwrap().is_empty());
                let before = db
                    .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                    .unwrap();
                drop(db);
                let db = ProtocolStore::open(OpenOptions {
                    path: dir.path().join("p.redb"),
                    account_id: AccountId::new("a").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                    create: false,
                })
                .unwrap();
                assert!(!LiveReplies::default().record_reply(&db, &incoming).unwrap());
                assert!(reserve_pending(&db).unwrap().is_empty());
                assert_eq!(
                    db.read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                        .unwrap(),
                    before
                );
            }
        }
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
            register_written(&mut live, &store, &request, started);
            register_written(&mut live, &store, &generic_request, started);
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
                register_written(&mut live, &store, &request, started);
                register_written(&mut live, &store, &fetch_request, started);
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
                register_written(&mut live, &store, &request, started);
                register_written(&mut live, &store, &props_request, started);
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
                register_written(&mut live, &store, &request, started);
                register_written(&mut live, &store, &props_request, started);
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

    #[tokio::test(start_paused = true)]
    async fn report_callback_expires_at_native_timeout_without_replay() {
        use super::super::report;
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        for outcome in ["result", "error", "expire"] {
            let (dir, store) = store();
            let id = report::queue_report(
                &store,
                &crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap(),
                "chat",
                Some(true),
            )
            .unwrap();
            let request = reserve_next(&store).unwrap().unwrap();
            let mut live = LiveReplies::default();
            let started = Instant::now();
            register_written(&mut live, &store, &request, started);
            assert_eq!(live.deadline(), Some(started + Duration::from_secs(32)));
            advance(Duration::from_millis(31999)).await;
            live.expire();
            let reply = Node::new("iq").attr("id", id.as_str()).attr(
                "type",
                if outcome == "error" {
                    "error"
                } else {
                    "result"
                },
            );
            if outcome == "expire" {
                advance(Duration::from_millis(1)).await;
                let before = outcomes(&store).unwrap();
                live.expire();
                assert!(live.deadline().is_none());
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert_eq!(outcomes(&store).unwrap(), before);
                assert!(super::reply(&store, &id).unwrap().is_none());
            } else {
                assert!(live.record_reply(&store, &reply).unwrap());
                assert!(!live.record_reply(&store, &reply).unwrap());
            }
            let before = outcomes(&store).unwrap();
            assert_eq!(
                before[0].status,
                match outcome {
                    "result" => ActionStatus::Ok,
                    "error" => ActionStatus::Error,
                    _ => ActionStatus::Sent,
                }
            );
            drop(store);
            let reopened = ProtocolStore::open(OpenOptions {
                path: dir.path().join("p.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                create: false,
            })
            .unwrap();
            assert_eq!(outcomes(&reopened).unwrap(), before);
            assert!(reserve_pending(&reopened).unwrap().is_empty());
        }
        let (_dir, store) = store();
        queue(&store, "report", "wrong producer", ping).unwrap();
        let request = reserve_next(&store).unwrap().unwrap();
        let mut live = LiveReplies::default();
        register_written(&mut live, &store, &request, Instant::now());
        assert!(live.deadline().is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn photo_lookup_result_extensions_do_not_select_refusal_callback() {
        use super::super::profile_picture;
        for shape in [
            "picture_first",
            "metadata_first",
            "no_picture",
            "two_pictures",
        ] {
            let (dir, store) = store();
            let id = profile_picture::queue_get(
                &store,
                &crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap(),
            )
            .unwrap();
            let request = reserve_next(&store).unwrap().unwrap();
            let mut live = LiveReplies::default();
            register_written(&mut live, &store, &request, tokio::time::Instant::now());
            let picture = Node::new("picture")
                .attr("id", "7")
                .attr("url", "https://example.invalid/picture.jpg");
            let mut children = match shape {
                "picture_first" => vec![picture],
                "metadata_first" => vec![Node::new("metadata"), picture],
                "two_pictures" => vec![picture, Node::new("picture").attr("id", "8")],
                _ => vec![Node::new("metadata")],
            };
            children.push(Node::new("error").attr("code", "403"));
            children.push(Node::new("list").children(vec![
                Node::new("user").children(vec![Node::new("error").attr("code", "401")]),
            ]));
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(children);
            assert!(live.record_reply(&store, &reply).unwrap());
            assert!(!live.record_reply(&store, &reply).unwrap());
            let before = outcomes(&store).unwrap();
            assert_eq!(before[0].status, ActionStatus::Ok, "{shape}");
            assert_eq!(before[0].error_code, None);
            assert_eq!(before[0].photo_error, None);
            let saved_reply = super::reply(&store, &id).unwrap();
            assert_eq!(saved_reply, Some(reply));
            drop(store);
            let reopened = ProtocolStore::open(OpenOptions {
                path: dir.path().join("p.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                create: false,
            })
            .unwrap();
            assert_eq!(outcomes(&reopened).unwrap(), before);
            assert!(reserve_pending(&reopened).unwrap().is_empty());
        }
    }

    #[test]
    fn photo_error_selects_first_direct_coded_child() {
        use super::super::profile_picture;
        for upload in [false, true] {
            let (_dir, store) = store();
            let id = if upload {
                profile_picture::queue_set(&store, vec![0xff, 0xd8, 0xff, 0xe0, 1, 2, 0xff, 0xd9])
            } else {
                profile_picture::queue_get(
                    &store,
                    &crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap(),
                )
            }
            .unwrap();
            reserve_next(&store).unwrap().unwrap();
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "error")
                .children(vec![
                    Node::new("list").children(vec![
                        Node::new("user").children(vec![Node::new("error").attr("code", "401")]),
                    ]),
                    Node::new("error"),
                    Node::new("error").attr("code", "404"),
                    Node::new("error").attr("code", "410"),
                ]);
            assert!(record_reply(&store, &reply).unwrap());
            let saved = outcomes(&store).unwrap().remove(0);
            assert_eq!(saved.status, ActionStatus::Error);
            assert_eq!(saved.error_code, Some(404));
            assert_eq!(
                saved.photo_error,
                Some(profile_picture::PhotoErrorOutcome {
                    code: 404,
                    empty_lookup: !upload
                })
            );
        }
    }

    #[test]
    fn photo_error_metadata_preserves_native_codes_and_reopen_without_replay() {
        use super::super::profile_picture::{self, PhotoErrorOutcome};
        for upload in [false, true] {
            for (text, code) in [
                (Some("404"), 404),
                (Some("410"), 410),
                (Some("403"), 403),
                (Some("٤٠٤"), 404),
                (Some("-1"), -1),
                (Some("65536"), 65536),
                (Some("-2147483648"), i32::MIN),
                (Some("2147483647"), i32::MAX),
                (Some("2147483648"), 0),
                (Some(" 404"), 0),
                (Some(""), 0),
                (None, 0),
            ] {
                let (dir, store) = store();
                let id = if upload {
                    profile_picture::queue_set(
                        &store,
                        vec![0xff, 0xd8, 0xff, 0xe0, 1, 2, 0xff, 0xd9],
                    )
                } else {
                    profile_picture::queue_get(
                        &store,
                        &crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap(),
                    )
                }
                .unwrap();
                reserve_next(&store).unwrap().unwrap();
                let mut children = vec![Node::new("error")];
                if let Some(text) = text {
                    children.push(Node::new("error").attr("code", text));
                    children.push(Node::new("error").attr("code", "410"));
                }
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "error")
                    .children(children);
                assert!(record_reply(&store, &reply).unwrap());
                assert!(!record_reply(&store, &reply).unwrap());
                let before = outcomes(&store).unwrap();
                assert_eq!(
                    before[0].photo_error,
                    Some(PhotoErrorOutcome {
                        code,
                        empty_lookup: !upload && matches!(code, 404 | 410)
                    })
                );
                assert_eq!(before[0].status, ActionStatus::Error);
                let raw: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, key(&id).as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                let mut legacy = raw.clone();
                legacy.as_object_mut().unwrap().remove("photo_error");
                let legacy: Row = serde_json::from_value(legacy).unwrap();
                assert!(legacy.photo_error.is_none());
                assert!(
                    serde_json::to_value(legacy)
                        .unwrap()
                        .get("photo_error")
                        .is_none()
                );
                drop(store);
                let reopened = ProtocolStore::open(OpenOptions {
                    path: dir.path().join("p.redb"),
                    account_id: AccountId::new("a").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                    create: false,
                })
                .unwrap();
                assert_eq!(outcomes(&reopened).unwrap(), before);
                assert!(reserve_pending(&reopened).unwrap().is_empty());
                let after: serde_json::Value = reopened
                    .read(|tx| tx.get_json(Table::Checkpoints, key(&id).as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(raw, after);
            }
        }
        // A kind label alone cannot borrow photo error interpretation.
        let (_dir, store) = store();
        let id = queue(&store, profile_picture::GET_KIND, "wrong producer", ping).unwrap();
        reserve_next(&store).unwrap().unwrap();
        record_reply(
            &store,
            &Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "error")
                .children(vec![Node::new("error").attr("code", "404")]),
        )
        .unwrap();
        assert!(outcomes(&store).unwrap()[0].photo_error.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn photo_callbacks_keep_zero_timer_and_upload_extension_semantics() {
        use super::super::profile_picture;
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        for upload in [false, true] {
            for outcome in ["result", "error"] {
                let (_dir, store) = store();
                let id = if upload {
                    profile_picture::queue_set(
                        &store,
                        vec![0xff, 0xd8, 0xff, 0xe0, 1, 2, 0xff, 0xd9],
                    )
                } else {
                    profile_picture::queue_get(
                        &store,
                        &crate::model::jid::Jid::parse("15555550101@s.whatsapp.net").unwrap(),
                    )
                }
                .unwrap();
                let request = reserve_next(&store).unwrap().unwrap();
                let mut live = LiveReplies::default();
                register_written(&mut live, &store, &request, Instant::now());
                assert!(live.deadline().is_none());
                advance(Duration::from_secs(60)).await;
                live.expire();
                let mut children = vec![
                    Node::new("picture")
                        .attr("id", "7")
                        .attr("url", "https://example.invalid/picture.jpg"),
                ];
                if upload || outcome == "error" {
                    children.push(Node::new("error").attr("code", "403"));
                }
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", outcome)
                    .children(children);
                assert!(live.record_reply(&store, &reply).unwrap());
                assert!(!live.record_reply(&store, &reply).unwrap());
                let outcomes = outcomes(&store).unwrap();
                let saved = outcomes.iter().find(|a| a.id == id).unwrap();
                assert_eq!(
                    saved.status,
                    if outcome == "result" {
                        ActionStatus::Ok
                    } else {
                        ActionStatus::Error
                    }
                );
                assert_eq!(
                    saved.error_code,
                    if outcome == "result" { None } else { Some(403) }
                );
            }
        }
        // A mislabeled action must not inherit this callback's reply policy.
        let (_dir, store) = store();
        for kind in [profile_picture::SET_KIND, profile_picture::GET_KIND] {
            let id = queue(&store, kind, "wrong producer", ping).unwrap();
            reserve_next(&store).unwrap().unwrap();
            assert!(
                record_reply(
                    &store,
                    &Node::new("iq")
                        .attr("id", id.as_str())
                        .attr("type", "result")
                        .children(vec![
                            Node::new("picture"),
                            Node::new("error").attr("code", "403")
                        ])
                )
                .unwrap()
            );
            assert_eq!(
                outcomes(&store)
                    .unwrap()
                    .iter()
                    .find(|row| row.id == id)
                    .unwrap()
                    .status,
                ActionStatus::Error
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn two_factor_callback_expiry_and_outer_type_preserve_uncertainty() {
        use super::super::two_factor::{self, TwoFactorChange};
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        for change in [
            TwoFactorChange {
                code: Some("928374"),
                email: None,
            },
            TwoFactorChange {
                code: Some(""),
                email: None,
            },
            TwoFactorChange {
                code: None,
                email: Some("private@example.invalid"),
            },
        ] {
            for outcome in ["expire", "result", "error"] {
                let (_dir, store) = store();
                let id = two_factor::queue_set(&store, &change).unwrap();
                let request = reserve_next(&store).unwrap().unwrap();
                let started = Instant::now();
                advance(Duration::from_secs(5)).await;
                let mut live = LiveReplies::default();
                register_written(&mut live, &store, &request, started);
                assert_eq!(live.deadline(), Some(started + Duration::from_secs(32)));
                advance(Duration::from_secs(26)).await;
                live.expire();
                assert!(live.0.contains(&id));
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr(
                        "type",
                        if outcome == "error" {
                            "error"
                        } else {
                            "result"
                        },
                    )
                    .children(vec![Node::new("error").attr("code", "403")]);
                let before = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                if outcome != "expire" {
                    assert!(live.record_reply(&store, &reply).unwrap());
                }
                advance(Duration::from_secs(1)).await;
                live.expire();
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert!(live.deadline().is_none());
                let outcomes = outcomes(&store).unwrap();
                let saved = outcomes.iter().find(|a| a.id == id).unwrap();
                assert_eq!(
                    saved.status,
                    match outcome {
                        "expire" => ActionStatus::Sent,
                        "result" => ActionStatus::Ok,
                        _ => ActionStatus::Error,
                    }
                );
                assert_eq!(
                    saved.error_code,
                    if outcome == "error" { Some(403) } else { None }
                );
                if outcome == "expire" {
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        before
                    );
                }
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn privacy_token_expiry_keeps_scheduled_state_and_uses_outer_reply_type() {
        use super::super::token_issue;
        use std::time::Duration;
        use tokio::time::{Instant, advance};
        for domain in ["s.whatsapp.net", "lid"] {
            for outcome in ["expire", "result", "error"] {
                let (_dir, store) = store();
                let user = crate::model::jid::Jid::parse(&format!("15555550101@{domain}")).unwrap();
                let scheduled = 1_800_000_000;
                let id = store
                    .write(|tx| token_issue::schedule_after_ack_in(tx, &user, scheduled))
                    .unwrap()
                    .unwrap();
                let request = reserve_next(&store).unwrap().unwrap();
                let started = Instant::now();
                advance(Duration::from_secs(5)).await;
                let mut live = LiveReplies::default();
                register_written(&mut live, &store, &request, started);
                assert_eq!(live.deadline(), Some(started + Duration::from_secs(32)));
                advance(Duration::from_secs(26)).await;
                live.expire();
                assert!(live.0.contains(&id));
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr(
                        "type",
                        if outcome == "error" {
                            "error"
                        } else {
                            "result"
                        },
                    )
                    .children(vec![Node::new("error").attr("code", "403")]);
                let before = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                if outcome != "expire" {
                    assert!(live.record_reply(&store, &reply).unwrap());
                }
                advance(Duration::from_secs(1)).await;
                live.expire();
                assert!(!live.record_reply(&store, &reply).unwrap());
                assert!(live.deadline().is_none());
                let outcomes = outcomes(&store).unwrap();
                let saved = outcomes.iter().find(|a| a.id == id).unwrap();
                assert_eq!(
                    saved.status,
                    match outcome {
                        "expire" => ActionStatus::Sent,
                        "result" => ActionStatus::Ok,
                        _ => ActionStatus::Error,
                    }
                );
                assert_eq!(
                    saved.error_code,
                    if outcome == "error" { Some(403) } else { None }
                );
                assert_eq!(
                    saved.token_issue_completion,
                    if outcome == "result" {
                        Some(token_issue::TokenIssueCompletion::ScheduledTimestampConfirmed)
                    } else {
                        None
                    }
                );
                if outcome == "expire" {
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        before
                    );
                }
                assert!(
                    store
                        .write(|tx| token_issue::schedule_after_ack_in(tx, &user, scheduled + 60))
                        .unwrap()
                        .is_none()
                );
            }
        }
        // Account-privacy setters share xmlns=privacy but not this producer.
        assert_eq!(
            callback_seconds(
                &Node::new("iq")
                    .attr("id", "PRIVACY-SETTING")
                    .attr("xmlns", "privacy")
                    .attr("type", "set")
                    .children(vec![Node::new("privacy")])
            ),
            None
        );
    }

    #[test]
    fn about_refusal_preserves_signed_order_and_malformed_boundary_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("about-refusal.redb");
        let open = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7u8; 32]))),
                create: true,
            })
            .unwrap()
        };
        let store = open();
        let id = super::super::about::queue_set(&store, "Available").unwrap();
        reserve_pending(&store).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "error")
            .children(vec![
                Node::new("error"),
                Node::new("extension").children(vec![Node::new("error").attr("code", "403")]),
                Node::new("error").attr("code", "-1"),
                Node::new("error").attr("code", "٦٥٥٣٦"),
                Node::new("error").attr("code", "+2147483647"),
                Node::new("error").attr("code", "-2147483648"),
                Node::new("error").attr("code", "2147483648"),
                Node::new("error").attr("code", "500"),
            ]);
        assert!(record_reply(&store, &reply).unwrap());
        assert!(!record_reply(&store, &reply).unwrap());
        let expected = super::super::about::AboutRefusal {
            codes: vec![-1, 65536, i32::MAX, i32::MIN],
            malformed_code: true,
        };
        let before = outcomes(&store).unwrap();
        assert_eq!(before[0].status, ActionStatus::Error);
        assert_eq!(before[0].about_refusal, Some(expected.clone()));
        drop(store);
        let store = open();
        assert_eq!(outcomes(&store).unwrap(), before);
        assert!(reserve_pending(&store).unwrap().is_empty());
        let view = serde_json::to_value(&before[0]).unwrap();
        assert_eq!(view["about_refusal"]["codes"][1], 65536);
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
            register_written(&mut live, &store, &request, tokio::time::Instant::now());
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
