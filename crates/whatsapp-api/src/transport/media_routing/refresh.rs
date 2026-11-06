//! Prospective encrypted media refresh context. No implicit query, replay or retry.
//! Native 0VM.A07/1xY.CK4/0VM.A0A evidence is linked by the parent module.
use super::{invalid, project_media_conn};
use crate::{
    Result,
    binary::{Encoder, Node, TokenDictionary},
    model::{ids::AccountGeneration, time::Timestamp},
    store::{ProtocolStore, Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
mod recovery;
pub use recovery::RecoveryReport;

/// Restore known receipt-anchored waits without sending or replaying a request.
/// Unsequenced observations remain ambiguous; this is not retry authorization.
pub fn recover_throttle(store: &ProtocolStore) -> Result<RecoveryReport> {
    recovery::recover_throttle(store)
}

/// APK43.72 0VM.A07 registers media IQ124 with32000ms through0Sf.A0R.
/// Nominal callback window, not native Handler scheduling/clock equivalence.
pub const REPLY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(32_000);

/// Send one already-reserved media request and capture its correlated reply.
/// The32s window starts before write and never restarts. A shorter configured
/// socket I/O limit still wins. Caller owns preparation, settlement and channel
/// disposal on failure; no automatic refresh, retry or durable reset is added.
pub async fn write_and_wait_captured<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    store: &ProtocolStore,
    channel: &mut super::super::handshake::EncryptedConnection<S>,
    request: &Node,
    id: &str,
) -> Result<(Node, Vec<Node>)> {
    super::super::messaging::write_and_wait_captured_until(
        store,
        channel,
        request,
        "iq",
        id,
        tokio::time::Instant::now() + REPLY_TIMEOUT,
    )
    .await
}

const CURRENT: &[u8] = b"media-routing-current:v1";
const OWNER: &[u8] = b"media-routing-owner:v1";
const THROTTLE: &[u8] = b"media-routing-throttle-until:v1";
const RECEIPT_SEQUENCE: &[u8] = b"media-routing-receipt-sequence:v1";
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Current {
    generation: AccountGeneration,
    action_id: String,
    requested_at: Timestamp,
    wire: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    context_version: u16,
    generation: AccountGeneration,
    action_id: String,
    requested_at: Timestamp,
    at: Timestamp,
    status: String,
    #[serde(default)]
    settlement_sequence: Option<u64>,
    request_wire: Vec<u8>,
    request_digest: [u8; 32],
    prior: Option<Current>,
    wire: Option<Vec<u8>>,
    effective_wire: Option<Vec<u8>>,
}
/// Local settlement, not upload acceptance or complete native lifecycle parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Settlement {
    /// Correlated result projected and installed atomically.
    Applied,
    /// Correlated outer server error; current route is unchanged.
    Refused,
    /// Correlated result outside the supported parser; raw reply is retained.
    Unsupported,
    /// Prior route changed since preparation; current route is unchanged.
    Stale,
}
fn redact(error: crate::Error) -> crate::Error {
    match error {
        crate::Error::Storage(crate::store::StoreError::Serialization(_)) => crate::Error::Storage(
            crate::store::StoreError::Serialization("invalid stored media refresh data".into()),
        ),
        other => other,
    }
}
/// Safe applied-route counts for the existing CLI output. No IDs/auth/hosts.
pub fn current_summary(store: &ProtocolStore) -> Result<Option<(usize, bool)>> {
    store
        .read(|tx| {
            let current: Option<Current> = tx.get_json(Table::Checkpoints, CURRENT, 1)?;
            let Some(current) = current else {
                return Ok(None);
            };
            if current.generation != tx.generation() {
                return Err(invalid("media route generation mismatch"));
            }
            let node = super::super::actions::decode(&current.wire)?;
            validate_conn(&node)?;
            Ok(Some((
                node.child_nodes()
                    .iter()
                    .filter(|n| n.description == "host")
                    .count(),
                node.get_attr("auth").is_some(),
            )))
        })
        .map_err(redact)
}
#[derive(Debug, PartialEq, Eq)]
enum ThrottleDirective {
    Clear,
    Wait(u32),
}
fn integer_attr(node: &Node, name: &str) -> Option<i32> {
    match node.get_attr(name) {
        None => Some(0),
        Some(value) => value
            .as_text()
            .and_then(super::super::mex::java_decimal_long)
            .and_then(|value| i32::try_from(value).ok()),
    }
}
// APK 43.72 1xY.Bvj: skip malformed entries/code0, consume the first valid
// nonzero code+backoff pair. 507 positive seconds cap at 10800; 503 resets.
fn error_directive(reply: &Node) -> Option<ThrottleDirective> {
    for error in reply
        .child_nodes()
        .iter()
        .filter(|node| node.description == "error")
    {
        let Some(code) = integer_attr(error, "code") else {
            continue;
        };
        if code == 0 {
            continue;
        }
        let Some(backoff) = integer_attr(error, "backoff") else {
            continue;
        };
        return match code {
            503 => Some(ThrottleDirective::Clear),
            507 if backoff > 0 => Some(ThrottleDirective::Wait(backoff.min(10800) as u32)),
            _ => None,
        };
    }
    None
}
fn valid_deadline(deadline: Option<Timestamp>) -> Result<Option<Timestamp>> {
    if deadline.is_some_and(|deadline| deadline.0 <= 0) {
        return Err(invalid("invalid stored media throttle deadline"));
    }
    Ok(deadline)
}
/// Saved server-throttle deadline, including expired observations. No wait/retry.
pub fn throttle_until(store: &ProtocolStore) -> Result<Option<Timestamp>> {
    store
        .read(|tx| valid_deadline(tx.get_json(Table::Checkpoints, THROTTLE, 1)?))
        .map_err(redact)
}
fn require_not_throttled_in(tx: &WriteTx<'_>, now: Timestamp) -> Result<()> {
    if let Some(deadline) = valid_deadline(tx.get_json(Table::Checkpoints, THROTTLE, 1)?)?
        && now <= deadline
    {
        return Err(crate::Error::invalid(
            "media_routing",
            format!(
                "server media-route throttle until {} ms; no refresh reserved",
                deadline.0
            ),
        ));
    }
    Ok(())
}
fn retain_throttle_in(
    tx: &mut WriteTx<'_>,
    directive: Option<ThrottleDirective>,
    at: Timestamp,
) -> Result<()> {
    // Opening an existing observation must never turn an auth/schema fault into
    // absence, even on a legitimate native reset transition.
    valid_deadline(tx.get_json(Table::Checkpoints, THROTTLE, 1)?)?;
    match directive {
        Some(ThrottleDirective::Clear) => {
            tx.delete(Table::Checkpoints, THROTTLE)?;
        }
        Some(ThrottleDirective::Wait(seconds)) => {
            let deadline =
                at.0.checked_add(i64::from(seconds) * 1000)
                    .filter(|deadline| *deadline > 0)
                    .ok_or_else(|| invalid("media throttle deadline overflow"))?;
            tx.put_json(Table::Checkpoints, THROTTLE, 1, &Timestamp(deadline))?;
        }
        None => {}
    }
    Ok(())
}

/// Durable local receipt order for one prospective action. None means missing,
/// pending or legacy-unsequenced; it never invents historical receipt order.
pub fn settlement_sequence(store: &ProtocolStore, id: &str) -> Result<Option<u64>> {
    store
        .read(|tx| {
            let record: Option<Record> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
            let Some(record) = record else {
                return Ok(None);
            };
            validate(&record)?;
            if record.action_id != id || record.generation != tx.generation() {
                return Err(invalid("media receipt context mismatch"));
            }
            Ok(record.settlement_sequence)
        })
        .map_err(redact)
}
fn checked_receipt_counter_in(tx: &WriteTx<'_>) -> Result<Option<u64>> {
    let prior: Option<u64> = tx.get_json(Table::Checkpoints, RECEIPT_SEQUENCE, 1)?;
    if prior == Some(0) {
        return Err(invalid("invalid media receipt counter"));
    }
    if prior == Some(u64::MAX) {
        return Err(invalid("media receipt counter exhausted"));
    }
    if prior.is_none() {
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, bytes) in rows {
                if !key.starts_with(b"media-route:") {
                    continue;
                }
                let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| {
                    crate::Error::Storage(crate::store::StoreError::Serialization(
                        "invalid stored media refresh data".into(),
                    ))
                })?;
                if value.get("context_version").is_none() {
                    continue;
                }
                let record: Record = serde_json::from_value(value).map_err(|_| {
                    crate::Error::Storage(crate::store::StoreError::Serialization(
                        "invalid stored media refresh data".into(),
                    ))
                })?;
                validate(&record)?;
                if record.settlement_sequence.is_some() {
                    return Err(invalid("media receipt counter missing; no order reset"));
                }
            }
            if done {
                break;
            }
        }
    }
    Ok(prior)
}
fn next_receipt_sequence_in(tx: &mut WriteTx<'_>) -> Result<u64> {
    let prior = checked_receipt_counter_in(tx)?;
    let next = prior
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| invalid("media receipt counter exhausted"))?;
    tx.put_json(Table::Checkpoints, RECEIPT_SEQUENCE, 1, &next)?;
    Ok(next)
}

fn key(id: &str) -> String {
    format!("media-route:{id}")
}
fn encode(node: &Node) -> Result<Vec<u8>> {
    Ok(Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(node)?)
}
fn conn(node: &Node) -> Result<&Node> {
    let mut nodes = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "media_conn");
    let value = nodes
        .next()
        .ok_or_else(|| invalid("missing media connection"))?;
    if nodes.next().is_some() {
        return Err(invalid("ambiguous media connection"));
    }
    Ok(value)
}
fn validate_conn(node: &Node) -> Result<()> {
    super::is_new_route(node)?;
    if node.get_attr("auth").and_then(|v| v.as_text()).is_none() {
        return Err(invalid("missing media auth"));
    }
    for name in ["ttl", "auth_ttl"] {
        if node
            .get_attr(name)
            .and_then(|v| v.as_text())
            .and_then(super::super::mex::java_decimal_long)
            .is_none()
        {
            return Err(invalid("missing or invalid media lifetime"));
        }
    }
    for host in node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "host")
    {
        if host
            .get_attr("hostname")
            .and_then(|v| v.as_text())
            .is_none()
        {
            return Err(invalid("missing media hostname"));
        }
    }
    Ok(())
}
fn request(id: &str, prior: Option<&Current>) -> Result<Node> {
    let mut node = super::super::messaging::media_route_request(id);
    if let Some(prior) = prior {
        let prior = super::super::actions::decode(&prior.wire)?;
        validate_conn(&prior)?;
        if let Some(last_id) = prior.get_attr("id") {
            let last_id = last_id
                .as_text()
                .ok_or_else(|| invalid("invalid prior route ID"))?;
            node = node.children(vec![Node::new("media_conn").attr("last_id", last_id)]);
        }
    }
    Ok(node)
}
fn validate(record: &Record) -> Result<()> {
    if record.settlement_sequence == Some(0)
        || (record.status == "outcome_unknown" && record.settlement_sequence.is_some())
        || record.context_version != 1
        || record.action_id.is_empty()
        || record.requested_at.0 <= 0
        || record
            .prior
            .as_ref()
            .is_some_and(|p| p.generation != record.generation)
        || <[u8; 32]>::from(Sha256::digest(&record.request_wire)) != record.request_digest
        || super::super::actions::decode(&record.request_wire)?
            != request(&record.action_id, record.prior.as_ref())?
    {
        return Err(invalid("invalid media refresh context"));
    }
    Ok(())
}
fn has_context_rows(rows: Vec<(Vec<u8>, Vec<u8>)>, only_applied: bool) -> Result<bool> {
    if rows.len() == 10000 {
        return Err(invalid("media history scan limit reached"));
    }
    for (key, bytes) in rows {
        if !key.starts_with(b"media-route:") {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| {
            crate::Error::Storage(crate::store::StoreError::Serialization(
                "invalid stored media refresh data".into(),
            ))
        })?;
        if value.get("context_version").is_some() && (!only_applied || value["status"] == "result")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Freeze request bytes and the coordinator's exact prior route atomically.
/// This returns an unsent node. Only the caller's authorized operation may send it.
/// Legacy route history does not seed this coordinator retrospectively.
pub fn prepare(store: &ProtocolStore, id: &str) -> Result<Node> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(invalid("invalid media refresh ID"));
    }
    let recovery = recover_throttle(store)?;
    if let Some(until) = recovery
        .unsequenced_active_until
        .into_iter()
        .chain(recovery.unbound_active_until)
        .max()
    {
        return Err(crate::Error::invalid(
            "media_routing",
            format!(
                "saved media wait with unproved receipt order until {} ms; no refresh reserved",
                until.0
            ),
        ));
    }
    store
        .write(|tx| {
            if tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?.is_some() {
                return Err(invalid("media refresh already reserved"));
            }
            require_not_throttled_in(tx, Timestamp::now())?;
            checked_receipt_counter_in(tx)?;
            let owner: Option<bool> = tx.get_json(Table::Checkpoints, OWNER, 1)?;
            let prior: Option<Current> = tx.get_json(Table::Checkpoints, CURRENT, 1)?;
            if owner.is_none() && has_context_rows(tx.scan(Table::Checkpoints, 1, 10000)?, false)? {
                return Err(invalid(
                    "prospective media history has no coordinator owner",
                ));
            }
            if owner == Some(false) || (prior.is_some() && owner != Some(true)) {
                return Err(invalid("media route has no valid owner"));
            }
            if owner == Some(true)
                && prior.is_none()
                && has_context_rows(tx.scan(Table::Checkpoints, 1, 10000)?, true)?
            {
                return Err(invalid("applied media route pointer is missing"));
            }
            if let Some(current) = &prior {
                let saved: serde_json::Value = tx
                    .get_json(Table::Checkpoints, key(&current.action_id).as_bytes(), 1)?
                    .ok_or_else(|| invalid("prior media route has no original record"))?;
                let projected = effective_reply(&saved)?
                    .ok_or_else(|| invalid("prior media route lacks prospective context"))?;
                if conn(&projected)? != &super::super::actions::decode(&current.wire)?
                    || saved["requested_at"].as_i64() != Some(current.requested_at.0)
                    || saved["action_id"].as_str() != Some(current.action_id.as_str())
                    || saved["generation"].as_u64() != Some(u64::from(current.generation.0))
                {
                    return Err(invalid("prior media route differs from applied record"));
                }
            }
            tx.put_json(Table::Checkpoints, OWNER, 1, &true)?;
            if prior
                .as_ref()
                .is_some_and(|p| p.generation != tx.generation())
            {
                return Err(invalid("media route generation mismatch"));
            }
            let node = request(id, prior.as_ref())?;
            let request_wire = encode(&node)?;
            let requested_at = Timestamp::now();
            let record = Record {
                context_version: 1,
                generation: tx.generation(),
                action_id: id.to_owned(),
                requested_at,
                at: requested_at,
                status: "outcome_unknown".into(),
                settlement_sequence: None,
                request_digest: Sha256::digest(&request_wire).into(),
                request_wire,
                prior,
                wire: None,
                effective_wire: None,
            };
            tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &record)?;
            Ok(node)
        })
        .map_err(redact)
}
fn outcome(status: &str) -> Result<Settlement> {
    match status {
        "result" => Ok(Settlement::Applied),
        "refused" => Ok(Settlement::Refused),
        "unsupported" => Ok(Settlement::Unsupported),
        "stale" => Ok(Settlement::Stale),
        _ => Err(invalid("media refresh is not settled")),
    }
}
/// Preserve a correlated reply and merge against frozen prior hosts atomically.
/// The caller owes an authenticated receive path; this is not network proof.
/// Call in the same transaction as any saved unrelated inbound frames.
pub fn settle_in(tx: &mut WriteTx<'_>, id: &str, reply: &Node) -> Result<Settlement> {
    settle_impl_in(tx, id, reply).map_err(redact)
}
fn settle_impl_in(tx: &mut WriteTx<'_>, id: &str, reply: &Node) -> Result<Settlement> {
    let mut record: Record = tx
        .get_json(Table::Checkpoints, key(id).as_bytes(), 1)?
        .ok_or_else(|| invalid("media reply has no prospective context"))?;
    validate(&record)?;
    if record.action_id != id
        || record.generation != tx.generation()
        || reply.description != "iq"
        || reply.get_attr("id").and_then(|v| v.as_text()) != Some(id)
    {
        return Err(invalid("media reply does not match prospective context"));
    }
    let kind = reply.get_attr("type").and_then(|v| v.as_text());
    if !matches!(kind, Some("result" | "error")) {
        return Err(invalid("media reply is not result or error"));
    }
    let wire = encode(reply)?;
    if record.status != "outcome_unknown" {
        if record.wire.as_ref() != Some(&wire) {
            return Err(invalid("conflicting media settlement"));
        }
        return outcome(&record.status);
    }
    if tx.get_json::<bool>(Table::Checkpoints, OWNER, 1)? != Some(true) {
        return Err(invalid("media refresh lost its coordinator owner"));
    }
    let current: Option<Current> = tx.get_json(Table::Checkpoints, CURRENT, 1)?;
    if current
        .as_ref()
        .is_some_and(|current| current.generation != tx.generation())
    {
        return Err(invalid("media route generation mismatch"));
    }

    if kind == Some("result")
        && current.is_none()
        && record.prior.is_none()
        && has_context_rows(tx.scan(Table::Checkpoints, 1, 10000)?, true)?
    {
        return Err(invalid("applied media route pointer is missing"));
    }
    let result = if kind == Some("error") {
        Settlement::Refused
    } else if current != record.prior {
        Settlement::Stale
    } else {
        let prior = record
            .prior
            .as_ref()
            .map(|p| super::super::actions::decode(&p.wire))
            .transpose()?;
        let projected = (|| {
            let incoming = conn(reply)?;
            validate_conn(incoming)?;
            project_media_conn(prior.as_ref(), incoming)
        })();
        match projected {
            Ok(projected) => {
                record.effective_wire =
                    Some(encode(&reply.clone().children(vec![projected.clone()]))?);
                tx.put_json(
                    Table::Checkpoints,
                    CURRENT,
                    1,
                    &Current {
                        generation: tx.generation(),
                        action_id: id.to_owned(),
                        requested_at: record.requested_at,
                        wire: encode(&projected)?,
                    },
                )?;
                Settlement::Applied
            }
            Err(crate::Error::InvalidInput { .. }) => Settlement::Unsupported,
            Err(error) => return Err(error),
        }
    };
    let received_at = Timestamp::now();
    let directive = if kind == Some("error") {
        error_directive(reply)
    } else if result == Settlement::Applied {
        Some(ThrottleDirective::Clear)
    } else {
        None
    };
    retain_throttle_in(tx, directive, received_at)?;
    record.status = match result {
        Settlement::Applied => "result",
        Settlement::Refused => "refused",
        Settlement::Unsupported => "unsupported",
        Settlement::Stale => "stale",
    }
    .into();
    record.settlement_sequence = Some(next_receipt_sequence_in(tx)?);
    record.at = received_at;
    record.wire = Some(wire);
    tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &record)?;
    Ok(result)
}
/// Resolve the applied current pointer instead of ordering receipt timestamps.
/// Once this coordinator is active, legacy history cannot resurrect a route.
pub(crate) fn active_record(store: &ProtocolStore) -> Result<Option<serde_json::Value>> {
    store
        .read(|tx| {
            let owner: Option<bool> = tx.get_json(Table::Checkpoints, OWNER, 1)?;
            if owner.is_none() {
                if tx.get(Table::Checkpoints, CURRENT, 1)?.is_some()
                    || has_context_rows(tx.scan(Table::Checkpoints, 1, 10000)?, false)?
                {
                    return Err(invalid(
                        "prospective media history has no coordinator owner",
                    ));
                }
                return Ok(None);
            }
            if owner != Some(true) {
                return Err(invalid("invalid media route owner"));
            }
            let current: Current = tx
                .get_json(Table::Checkpoints, CURRENT, 1)?
                .ok_or_else(|| invalid("media refresh has no applied current route"))?;
            if current.generation != tx.generation() {
                return Err(invalid("media route generation mismatch"));
            }
            let saved: serde_json::Value = tx
                .get_json(Table::Checkpoints, key(&current.action_id).as_bytes(), 1)?
                .ok_or_else(|| invalid("current media route has no original record"))?;
            let projected = effective_reply(&saved)?
                .ok_or_else(|| invalid("current media route lacks prospective context"))?;
            if conn(&projected)? != &super::super::actions::decode(&current.wire)?
                || saved["requested_at"].as_i64() != Some(current.requested_at.0)
                || saved["action_id"].as_str() != Some(current.action_id.as_str())
                || saved["generation"].as_u64() != Some(u64::from(current.generation.0))
            {
                return Err(invalid("current media route differs from applied record"));
            }
            Ok(Some(saved))
        })
        .map_err(redact)
}

/// Recompute a projected reply from sealed prospective context. None is legacy;
/// a context-bearing malformed record is an error, never a legacy fallback.
pub(crate) fn effective_reply(saved: &serde_json::Value) -> Result<Option<Node>> {
    if saved.get("context_version").is_none() {
        return Ok(None);
    }
    let record: Record = serde_json::from_value(saved.clone()).map_err(|_| {
        crate::Error::Storage(crate::store::StoreError::Serialization(
            "invalid stored media refresh data".into(),
        ))
    })?;
    validate(&record)?;
    if record.status != "result" {
        return Err(invalid("media refresh is not usable"));
    }
    let reply = super::super::actions::decode(
        record
            .wire
            .as_ref()
            .ok_or_else(|| invalid("missing media reply"))?,
    )?;
    if reply.description != "iq"
        || reply.get_attr("type").and_then(|v| v.as_text()) != Some("result")
        || reply.get_attr("id").and_then(|v| v.as_text()) != Some(record.action_id.as_str())
    {
        return Err(invalid("invalid stored media correlation"));
    }
    let prior = record
        .prior
        .as_ref()
        .map(|p| super::super::actions::decode(&p.wire))
        .transpose()?;
    let incoming = conn(&reply)?;
    validate_conn(incoming)?;
    let projected = reply
        .clone()
        .children(vec![project_media_conn(prior.as_ref(), incoming)?]);
    let effective = super::super::actions::decode(
        record
            .effective_wire
            .as_ref()
            .ok_or_else(|| invalid("missing media projection"))?,
    )?;
    if projected != effective {
        return Err(invalid("media projection differs from frozen context"));
    }
    Ok(Some(effective))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("media-refresh-test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([2; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn reply(id: &str, flag: &str, route_id: &str, auth: &str, host: &str) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .children(vec![
                Node::new("media_conn")
                    .attr("id", route_id)
                    .attr("auth", auth)
                    .attr("ttl", "3600")
                    .attr("auth_ttl", "1800")
                    .attr("is_new", flag)
                    .children(vec![
                        Node::new("host")
                            .attr("hostname", host)
                            .children(vec![Node::new("upload")]),
                    ]),
            ])
    }
    fn saved(store: &ProtocolStore, id: &str) -> serde_json::Value {
        store
            .read(|tx| tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1))
            .unwrap()
            .unwrap()
    }
    fn settle(store: &ProtocolStore, id: &str, node: &Node) -> Settlement {
        store.write(|tx| settle_in(tx, id, node)).unwrap()
    }
    #[test]
    fn rejected_reply_admission_preserves_current_and_pending_route_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("admission.redb");
        let store = open(&path);
        prepare(&store, "old").unwrap();
        settle(
            &store,
            "old",
            &reply("old", "1", "old-route", "old-auth", "old.whatsapp.net"),
        );
        prepare(&store, "pending").unwrap();
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap()
        };
        let before = snapshot(&store);
        let current = current_summary(&store).unwrap();
        let mut rejected = Vec::new();
        for kind in [
            None,
            Some("get"),
            Some("set"),
            Some("RESULT"),
            Some("unexpected"),
        ] {
            let mut node = Node::new("iq")
                .attr("id", "pending")
                .children(vec![Node::new("error").attr("code", "503")]);
            if let Some(kind) = kind {
                node = node.attr("type", kind);
            }
            rejected.push(node);
        }
        rejected.push(reply(
            "unrelated",
            "1",
            "new-route",
            "new-auth",
            "new.whatsapp.net",
        ));
        rejected.push(
            Node::new("message")
                .attr("id", "pending")
                .attr("type", "result"),
        );
        for node in rejected {
            assert!(store.write(|tx| settle_in(tx, "pending", &node)).is_err());
            assert_eq!(snapshot(&store), before);
            assert_eq!(current_summary(&store).unwrap(), current);
        }
        drop(store);
        let store = open(&path);
        assert_eq!(snapshot(&store), before);
        assert_eq!(current_summary(&store).unwrap(), current);
        assert!(prepare(&store, "pending").is_err());
        let valid = reply("pending", "1", "new-route", "new-auth", "new.whatsapp.net");
        assert_eq!(settle(&store, "pending", &valid), Settlement::Applied);
        let after = snapshot(&store);
        assert_eq!(settle(&store, "pending", &valid), Settlement::Applied);
        assert_eq!(snapshot(&store), after);
        let applied: Current = store
            .read(|tx| tx.get_json(Table::Checkpoints, CURRENT, 1))
            .unwrap()
            .unwrap();
        assert_eq!(applied.action_id, "pending");
        let route = super::super::super::actions::decode(&applied.wire).unwrap();
        assert_eq!(
            route.get_attr("id").and_then(|v| v.as_text()),
            Some("new-route")
        );
    }

    #[test]
    fn prospective_first_and_conditional_merge_survive_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("merge.redb");
        let store = open(&path);
        let first = prepare(&store, "first").unwrap();
        assert!(conn(&first).unwrap().get_attr("last_id").is_none());
        assert!(prepare(&store, "first").is_err());
        drop(store);
        let store = open(&path);
        let first_reply = reply("first", "0", "route-one", "old-auth", "old.whatsapp.net");
        assert_eq!(settle(&store, "first", &first_reply), Settlement::Applied);
        assert_eq!(settle(&store, "first", &first_reply), Settlement::Applied);
        let next = prepare(&store, "second").unwrap();
        assert_eq!(
            conn(&next).unwrap().get_attr("last_id").unwrap().as_text(),
            Some("route-one")
        );
        let second = reply(
            "second",
            "0",
            "route-two",
            "new-auth",
            "ignored.whatsapp.net",
        );
        assert_eq!(settle(&store, "second", &second), Settlement::Applied);
        let raw = super::super::super::actions::decode(
            &serde_json::from_value::<Vec<u8>>(saved(&store, "second")["wire"].clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(raw, second);
        let projected = effective_reply(&saved(&store, "second")).unwrap().unwrap();
        assert_eq!(
            conn(&projected)
                .unwrap()
                .get_attr("auth")
                .unwrap()
                .as_text(),
            Some("new-auth")
        );
        assert_eq!(
            conn(&projected).unwrap().child_nodes(),
            conn(&first_reply).unwrap().child_nodes()
        );
        drop(store);
        let store = open(&path);
        assert_eq!(
            effective_reply(&saved(&store, "second")).unwrap().unwrap(),
            projected
        );
        let third = prepare(&store, "third").unwrap();
        assert_eq!(
            conn(&third).unwrap().get_attr("last_id").unwrap().as_text(),
            Some("route-two")
        );
    }
    #[test]
    fn competing_reply_is_retained_stale_without_overwriting_current() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("stale.redb"));
        prepare(&store, "a").unwrap();
        prepare(&store, "b").unwrap();
        assert_eq!(
            settle(
                &store,
                "a",
                &reply("a", "1", "a", "a-auth", "a.whatsapp.net")
            ),
            Settlement::Applied
        );
        assert_eq!(
            settle(
                &store,
                "b",
                &reply("b", "1", "b", "b-auth", "b.whatsapp.net")
            ),
            Settlement::Stale
        );
        assert_eq!(active_record(&store).unwrap().unwrap()["action_id"], "a");
        assert_eq!(saved(&store, "b")["status"], "stale");
        assert!(!saved(&store, "b")["wire"].is_null());
        let next = prepare(&store, "c").unwrap();
        assert_eq!(
            conn(&next).unwrap().get_attr("last_id").unwrap().as_text(),
            Some("a")
        );
    }
    #[test]
    fn outer_transaction_cut_rolls_back_projection_and_receipt() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("cut.redb"));
        prepare(&store, "a").unwrap();
        let before = saved(&store, "a");
        let result: Result<()> = store.write(|tx| {
            assert_eq!(
                settle_in(tx, "a", &reply("a", "1", "a", "auth", "a.whatsapp.net"))?,
                Settlement::Applied
            );
            Err(invalid("synthetic cut"))
        });
        assert!(result.is_err());
        assert_eq!(saved(&store, "a"), before);
        assert_eq!(current_summary(&store).unwrap(), None);
    }
    #[test]
    fn unsupported_results_preserve_installed_route_and_server_wait_across_reopen() {
        for case in [
            "missing_conn",
            "duplicate_conn",
            "missing_auth",
            "invalid_ttl",
            "invalid_auth_ttl",
            "invalid_is_new",
            "valid",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("unsupported-retains-authority.redb");
            let store = open(&path);
            prepare(&store, "seed").unwrap();
            assert_eq!(
                settle(
                    &store,
                    "seed",
                    &reply("seed", "1", "old", "old-auth", "old.whatsapp.net")
                ),
                Settlement::Applied
            );
            prepare(&store, "held").unwrap();
            prepare(&store, "wait").unwrap();
            assert_eq!(
                settle(&store, "wait", &error("wait", "507", "60")),
                Settlement::Refused
            );
            let until = throttle_until(&store).unwrap().unwrap();
            let prior = store
                .read(|tx| tx.get(Table::Checkpoints, CURRENT, 1))
                .unwrap()
                .unwrap();
            let base = reply("held", "1", "new", "new-auth", "new.whatsapp.net");
            let mut incoming = base.child("media_conn").unwrap().clone();
            incoming = match case {
                "missing_auth" => {
                    incoming.attributes.retain(|attr| attr.name != "auth");
                    incoming
                }
                "invalid_ttl" => incoming.attr("ttl", "not-a-long"),
                "invalid_auth_ttl" => incoming.attr("auth_ttl", "9223372036854775808"),
                "invalid_is_new" => incoming.attr("is_new", "2147483648"),
                _ => incoming,
            };
            let children = match case {
                "missing_conn" => vec![],
                "duplicate_conn" => vec![incoming.clone(), incoming],
                _ => vec![incoming],
            };
            let candidate = base.children(children);
            let expected = if case == "valid" {
                Settlement::Applied
            } else {
                Settlement::Unsupported
            };
            assert_eq!(settle(&store, "held", &candidate), expected, "{case}");
            assert_eq!(
                saved(&store, "held")["wire"],
                serde_json::json!(encode(&candidate).unwrap())
            );
            let before_duplicate = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap();
            assert_eq!(settle(&store, "held", &candidate), expected);
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                    .unwrap(),
                before_duplicate
            );
            if case == "valid" {
                assert_eq!(throttle_until(&store).unwrap(), None);
                assert_ne!(
                    store
                        .read(|tx| tx.get(Table::Checkpoints, CURRENT, 1))
                        .unwrap()
                        .unwrap(),
                    prior
                );
            } else {
                assert_eq!(throttle_until(&store).unwrap(), Some(until));
                assert_eq!(
                    store
                        .read(|tx| tx.get(Table::Checkpoints, CURRENT, 1))
                        .unwrap()
                        .unwrap(),
                    prior
                );
                assert!(
                    store
                        .write(|tx| settle_in(
                            tx,
                            "held",
                            &reply("held", "1", "late", "late-auth", "late.whatsapp.net")
                        ))
                        .is_err()
                );
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                        .unwrap(),
                    before_duplicate
                );
            }
            drop(store);
            let reopened = open(&path);
            assert_eq!(
                reopened
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                    .unwrap(),
                before_duplicate
            );
            if case != "valid" {
                assert_eq!(throttle_until(&reopened).unwrap(), Some(until));
                assert_eq!(
                    reopened
                        .read(|tx| tx.get(Table::Checkpoints, CURRENT, 1))
                        .unwrap()
                        .unwrap(),
                    prior
                );
                assert!(prepare(&reopened, "blocked-next").is_err());
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                        .unwrap(),
                    before_duplicate
                );
            }
        }
    }

    #[test]
    fn errors_unsupported_and_conflicting_settlement_preserve_authority() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("refusal.redb"));
        prepare(&store, "a").unwrap();
        let err = Node::new("iq").attr("id", "a").attr("type", "error");
        assert_eq!(settle(&store, "a", &err), Settlement::Refused);
        assert!(
            store
                .write(|tx| settle_in(tx, "a", &reply("a", "1", "a", "auth", "a.whatsapp.net")))
                .is_err()
        );
        prepare(&store, "b").unwrap();
        let malformed = Node::new("iq").attr("id", "b").attr("type", "result");
        assert_eq!(settle(&store, "b", &malformed), Settlement::Unsupported);
        assert_eq!(current_summary(&store).unwrap(), None);
        assert!(effective_reply(&saved(&store, "b")).is_err());
        assert!(
            effective_reply(&serde_json::json!({"status":"result","wire":[]}))
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn orphaned_current_and_missing_owner_cannot_create_new_refresh() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("orphan.redb"));
        prepare(&store, "a").unwrap();
        prepare(&store, "pending-first").unwrap();
        settle(&store, "a", &reply("a", "1", "a", "auth", "a.whatsapp.net"));
        let current: Current = store
            .read(|tx| tx.get_json(Table::Checkpoints, CURRENT, 1))
            .unwrap()
            .unwrap();
        store
            .write(|tx| tx.delete(Table::Checkpoints, CURRENT))
            .unwrap();
        assert!(prepare(&store, "lost-current").is_err());
        assert!(active_record(&store).is_err());
        assert!(
            store
                .write(|tx| settle_in(
                    tx,
                    "pending-first",
                    &reply("pending-first", "1", "new", "auth", "new.whatsapp.net")
                ))
                .is_err()
        );
        store
            .write(|tx| tx.put_json(Table::Checkpoints, CURRENT, 1, &current))
            .unwrap();
        let original = saved(&store, "a");
        let mut wrong_generation = original.clone();
        wrong_generation["generation"] = 17.into();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key("a").as_bytes(),
                    1,
                    &wrong_generation,
                )
            })
            .unwrap();
        assert!(active_record(&store).is_err());
        assert!(prepare(&store, "wrong-generation-source").is_err());
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key("a").as_bytes(), 1, &original))
            .unwrap();
        store
            .write(|tx| tx.delete(Table::Checkpoints, key("a").as_bytes()))
            .unwrap();
        assert!(prepare(&store, "b").is_err());
        assert!(active_record(&store).is_err());
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, key("b").as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        store
            .write(|tx| tx.delete(Table::Checkpoints, OWNER))
            .unwrap();
        assert!(prepare(&store, "c").is_err());
        let pending = open(&dir.path().join("pending-owner.redb"));
        prepare(&pending, "p").unwrap();
        pending
            .write(|tx| tx.delete(Table::Checkpoints, OWNER))
            .unwrap();
        assert!(prepare(&pending, "q").is_err());
        assert!(active_record(&pending).is_err());
        assert!(
            pending
                .write(|tx| settle_in(tx, "p", &reply("p", "1", "p", "auth", "p.whatsapp.net")))
                .is_err()
        );
    }

    fn error(id: &str, code: &str, backoff: &str) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "error")
            .children(vec![
                Node::new("error")
                    .attr("code", code)
                    .attr("backoff", backoff),
            ])
    }
    #[test]
    fn server_507_throttle_is_capped_durable_and_duplicate_receipts_do_not_extend_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("throttle.redb");
        let store = open(&path);
        prepare(&store, "a").unwrap();
        let refusal = error("a", "+507", "2147483647");
        assert_eq!(settle(&store, "a", &refusal), Settlement::Refused);
        let until = throttle_until(&store).unwrap().unwrap();
        let receipt = saved(&store, "a")["at"].as_i64().unwrap();
        assert_eq!(until.0 - receipt, 10800 * 1000);
        assert_eq!(settle(&store, "a", &refusal), Settlement::Refused);
        assert_eq!(throttle_until(&store).unwrap(), Some(until));
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(prepare(&store, "b").is_err());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        drop(store);
        let store = open(&path);
        assert_eq!(throttle_until(&store).unwrap(), Some(until));
        assert!(prepare(&store, "b").is_err());
        store
            .write(|tx| {
                assert!(require_not_throttled_in(tx, until).is_err());
                require_not_throttled_in(tx, Timestamp(until.0 + 1))
            })
            .unwrap();
    }
    #[test]
    fn native_error_selection_and_resets_do_not_create_automatic_queries() {
        let node = Node::new("iq").children(vec![
            Node::new("error").attr("code", "bad"),
            Node::new("error").attr("code", "0"),
            Node::new("error").attr("code", "٥٠٧").attr("backoff", "٢"),
        ]);
        assert_eq!(error_directive(&node), Some(ThrottleDirective::Wait(2)));
        for (code, backoff) in [("500", "60"), ("507", "0"), ("507", "-1"), ("507", "bad")] {
            assert_eq!(error_directive(&error("a", code, backoff)), None);
        }
        let first_other = Node::new("iq").children(vec![
            Node::new("error").attr("code", "500"),
            Node::new("error").attr("code", "507").attr("backoff", "60"),
        ]);
        assert_eq!(error_directive(&first_other), None);
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("reset.redb"));
        for id in ["a", "b", "c"] {
            prepare(&store, id).unwrap();
        }
        settle(&store, "a", &error("a", "507", "60"));
        assert!(throttle_until(&store).unwrap().is_some());
        settle(&store, "b", &error("b", "503", "0"));
        assert_eq!(throttle_until(&store).unwrap(), None);
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert_eq!(
            settle(&store, "b", &error("b", "503", "0")),
            Settlement::Refused
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    THROTTLE,
                    1,
                    &Timestamp(Timestamp::now().0 + 60000),
                )
            })
            .unwrap();
        let later = throttle_until(&store).unwrap();
        assert_eq!(
            settle(&store, "b", &error("b", "503", "0")),
            Settlement::Refused
        );
        assert_eq!(
            throttle_until(&store).unwrap(),
            later,
            "old reset receipt cannot clear a newer wait"
        );
        assert_eq!(
            settle(&store, "c", &reply("c", "1", "c", "auth", "c.whatsapp.net")),
            Settlement::Applied
        );
        assert_eq!(throttle_until(&store).unwrap(), None);
    }

    #[test]
    fn receipt_order_is_durable_across_outcomes_and_exact_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("order.redb");
        let store = open(&path);
        for id in ["z", "a", "m"] {
            prepare(&store, id).unwrap();
            assert_eq!(settlement_sequence(&store, id).unwrap(), None);
        }
        let refused = error("z", "500", "0");
        settle(&store, "z", &refused);
        assert_eq!(settlement_sequence(&store, "z").unwrap(), Some(1));
        let applied = reply("a", "1", "a", "auth", "a.whatsapp.net");
        settle(&store, "a", &applied);
        assert_eq!(settlement_sequence(&store, "a").unwrap(), Some(2));
        settle(&store, "m", &reply("m", "1", "m", "auth", "m.whatsapp.net"));
        assert_eq!(settlement_sequence(&store, "m").unwrap(), Some(3));
        settle(&store, "z", &refused);
        settle(&store, "a", &applied);
        assert_eq!(settlement_sequence(&store, "z").unwrap(), Some(1));
        drop(store);
        let store = open(&path);
        assert_eq!(settlement_sequence(&store, "m").unwrap(), Some(3));
        let counter: u64 = store
            .read(|tx| tx.get_json(Table::Checkpoints, RECEIPT_SEQUENCE, 1))
            .unwrap()
            .unwrap();
        assert_eq!(counter, 3);
    }
    #[test]
    fn receipt_counter_loss_overflow_and_transaction_cut_never_partly_settle() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("counter.redb"));
        prepare(&store, "a").unwrap();
        let node = reply("a", "1", "a", "auth", "a.whatsapp.net");
        let cut: Result<()> = store.write(|tx| {
            settle_in(tx, "a", &node)?;
            Err(invalid("synthetic cut"))
        });
        assert!(cut.is_err());
        assert_eq!(settlement_sequence(&store, "a").unwrap(), None);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, RECEIPT_SEQUENCE, 1))
                .unwrap()
                .is_none()
        );
        settle(&store, "a", &node);
        prepare(&store, "b").unwrap();
        for counter in [None, Some(0), Some(u64::MAX)] {
            store
                .write(|tx| match counter {
                    None => {
                        tx.delete(Table::Checkpoints, RECEIPT_SEQUENCE)?;
                        Ok(())
                    }
                    Some(counter) => tx.put_json(Table::Checkpoints, RECEIPT_SEQUENCE, 1, &counter),
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert!(prepare(&store, "cannot-reserve").is_err());
            assert!(
                store
                    .write(|tx| settle_in(
                        tx,
                        "b",
                        &reply("b", "1", "b", "new-auth", "b.whatsapp.net")
                    ))
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
    fn legacy_settled_records_are_not_retroactively_stamped() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("legacy-order.redb"));
        prepare(&store, "a").unwrap();
        let node = error("a", "500", "0");
        settle(&store, "a", &node);
        let mut legacy = saved(&store, "a");
        legacy
            .as_object_mut()
            .unwrap()
            .remove("settlement_sequence");
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key("a").as_bytes(), 1, &legacy))
            .unwrap();
        assert_eq!(settlement_sequence(&store, "a").unwrap(), None);
        settle(&store, "a", &node);
        assert_eq!(settlement_sequence(&store, "a").unwrap(), None);
        assert_eq!(saved(&store, "a"), legacy);
    }

    #[test]
    fn generation_and_request_digest_faults_do_not_become_legacy() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("fence.redb"));
        prepare(&store, "a").unwrap();
        let mut changed = saved(&store, "a");
        changed["request_digest"] = serde_json::json!(vec![0; 32]);
        assert!(effective_reply(&changed).is_err());
        store.write(|tx| tx.bump_generation()).unwrap();
        assert!(
            store
                .write(|tx| settle_in(tx, "a", &reply("a", "1", "a", "auth", "a.whatsapp.net")))
                .is_err()
        );
        assert!(prepare(&store, "a").is_err());
        assert!(prepare(&store, "new-generation-id").is_err());
    }
}
