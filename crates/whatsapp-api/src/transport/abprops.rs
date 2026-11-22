//! APK 37.71 Hgb.A00 / IWy.C8f case 1 AB-property read.
//! Saved replies are evidence only: MobileConfig precedence and application
//! are unfinished. This does not set the companion removal gate.
//!
//! 43.72 `X.15I.A09` additionally forces the same read (`HBE.A00(true, false,
//! 0)`) when a notification's first child is `<abprops/>` and the sending
//! channel is ready. [`queue_forced`] implements that trigger; see
//! docs/protocol/startup-requests.md.
use super::actions;
use crate::{
    Error, Result,
    binary::Node,
    model::jid::Jid,
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MARKER: &[u8] = b"abprops-initial-query";
/// Marker for a read the server asked for with a notification.
const FORCED_MARKER: &[u8] = b"abprops-forced-query";
// 0bq.<clinit> 0050..0064 and 07t.A0J 1ed2..1ee6, APK 2.26.37.71.
const REMOVAL_PROPERTY: i32 = 35661;
// 37.71 07t.A0L 1594..15a6 / 0q2.A02: Boolean counterpart suppression.
const TOKEN_COUNTERPART_PROPERTY: i32 = 15491;
// 37.71 07t.A0X 055a..056c / HxT.A00: Boolean pending-job namespace.
const TOKEN_PENDING_NAMESPACE_PROPERTY: i32 = 14303;
// 37.71 D1G.Bat 0004..000e (PQKeysUploadABPropsObserver): PQ_KEYS_UPLOAD_CODE.
// Turning it off deletes every Kyber prekey (DeleteKyberPreKeysJob).
const PQ_KEYS_UPLOAD_PROPERTY: i32 = 0x52ce;
// 43.72 08Y.A0Q registry: PQ inventory target (default 200) and upload batch
// (default 10), captured by the 0WJ provider. Values are counts, not booleans.
/// `0x6530` (25904): `context_jid` on the key query. Registered **true** in
/// `08Y.A0H` (`v2 = Boolean.valueOf(true)`), so absence keeps it enabled.
pub const CONTEXT_JID_PROPERTY: i32 = 0x6530;
/// Peer message-edit window in seconds (`152.A00` at 042c..045a multiplies it
/// by 1000 and compares it with the signed stanza-minus-parent time).
pub const PEER_EDIT_WINDOW_PROPERTY: i32 = 2983;
/// `0x5e60` (24160), the second `Can.A00` predicate input, registered false.
pub const PQ_SUPPORT_PROPERTY: i32 = 0x5e60;
/// Generation target for unsent one-time KEM keys (`0Yj.A00`).
pub const PQ_INVENTORY_PROPERTY: i32 = 21200;
/// Upload batch size for one-time KEM keys (`0jc.A1C`).
pub const PQ_BATCH_PROPERTY: i32 = 21201;

/// Queue one initial AB-property read without a fabricated cached hash.
/// Repeats reuse the original action, including refusals and uncertain outcomes.
pub fn queue(store: &ProtocolStore) -> Result<String> {
    store.write(|tx| {
        if let Some(id) = tx.get_json::<String>(Table::Checkpoints, MARKER, 1)? {
            return Ok(id);
        }
        let id = actions::queue_in(tx, "abprops_query", "initial AB-property read", |id| {
            // Hgb.A00 0056..00ea: protocol 1, initial empty hash, xmlns abt.
            // GAZ.A1T selects 1S7 (empty user, server s.whatsapp.net).
            Ok(Node::new("iq")
                .attr("id", id)
                .attr("type", "get")
                .attr("xmlns", "abt")
                .attr("to", Jid::parse("s.whatsapp.net")?)
                .children(vec![
                    Node::new("props").attr("protocol", "1").attr("hash", ""),
                ]))
        })?;
        tx.put_json(Table::Checkpoints, MARKER, 1, &id)?;
        Ok(id)
    })
}

/// `<notification><abprops/></notification>`: the trigger for the APK's forced
/// read. 2.26.43.72 `X.15I.A09` dispatches stanza kind 235 (notification) on the
/// **first child's tag** and compares it to `abprops`; it does not read an outer
/// `type` attribute for this branch. `X.37W` then checks the sending-channel
/// predicate (`X.0AI.A06`) and calls `X.HBE.A00(true, false, 0)`.
pub fn forced_fetch_requested(node: &Node) -> bool {
    node.description == "notification"
        && node
            .child_nodes()
            .first()
            .is_some_and(|child| child.description == "abprops")
}

/// Queue the forced push-config read (`force=true, refresh=false, refresh_id=0`).
///
/// The forced node has the same wire shape as the initial read: the forced
/// branch skips the saved hash, so `props` carries `protocol=1` and an empty
/// hash. A queued-but-unsent read is returned for sending; an uncertain or
/// refused read is never repeated; only a settled successful read allows a new
/// notification-triggered fetch, which is a new IQ rather than a replay.
pub fn queue_forced(store: &ProtocolStore) -> Result<Option<String>> {
    store.write(|tx| {
        if let Some(id) = tx.get_json::<String>(Table::Checkpoints, FORCED_MARKER, 1)? {
            let Some((outcome, _reply, _request)) = actions::view_in_write(tx, &id)? else {
                return Err(Error::invalid(
                    "abprops",
                    "saved forced query action is missing",
                ));
            };
            match outcome.status {
                // Already queued: the caller's drain sends it once.
                actions::ActionStatus::Queued => return Ok(Some(id)),
                // A completed fetch may be refreshed by a later notification.
                actions::ActionStatus::Ok => {}
                // Uncertain or refused: no automatic repeat.
                actions::ActionStatus::Sent | actions::ActionStatus::Error => return Ok(None),
            }
        }
        let id = actions::queue_in(tx, "abprops_query", "forced AB-property read", |id| {
            Ok(Node::new("iq")
                .attr("id", id)
                .attr("type", "get")
                .attr("xmlns", "abt")
                .attr("to", Jid::parse("s.whatsapp.net")?)
                .children(vec![
                    Node::new("props").attr("protocol", "1").attr("hash", ""),
                ]))
        })?;
        tx.put_json(Table::Checkpoints, FORCED_MARKER, 1, &id)?;
        Ok(Some(id))
    })
}

/// Lifecycle of the read, separate from effective runtime configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbPropsStatus {
    /// Queued locally, without a socket reservation.
    Queued,
    /// Reserved once, with no correlated response; never automatically replayed.
    OutcomeUnknown,
    /// Correlated server refusal, without an inferred wait.
    ServerRefused,
    /// Unsupported or ambiguous correlated reply.
    InvalidReply,
    /// Parsed configuration rows; not applied to receive policy.
    ConfigurationReceived,
    /// Full reply with no nonempty hash; APK skips value application.
    NotModified,
}
/// Fixed parser diagnoses; raw values and error text are never returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbPropsFailure {
    /// Invalid IQ correlation/type.
    Envelope,
    /// Missing/duplicate props child or nontext metadata.
    Props,
    /// More than the local 16384-property inspection bound.
    PropertySet,
    /// Nonempty config_code is not a signed Java decimal integer.
    ConfigCode,
    /// Required text config_value is missing or exceeds the local 64 KiB bound.
    ConfigValue,
    /// Sampling-only row lacks parsed event_code/sampling_weight integers.
    Sampling,
}
/// Content-free inspection except explicitly selected returned Booleans.
/// These are not effective APK gates: MobileConfig may supersede them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbPropsInspection {
    /// Last returned signed integer values for token bucket properties
    /// 865/909/996/997. Missing keys were not returned/applied; null means
    /// the last value was invalid. These are evidence, not effective settings.
    #[serde(default)]
    pub token_bucket_values: BTreeMap<i32, Option<i32>>,
    /// Original durable action selector.
    pub action_id: String,
    /// Correlated read lifecycle.
    pub status: AbPropsStatus,
    /// Fixed local parser failure, if any.
    pub failure: Option<AbPropsFailure>,
    /// Recognized outer refusal code; no raw body or inferred wait.
    pub server_error_code: Option<u16>,
    /// Count of configuration prop rows; null before a supported response.
    pub configuration_rows: Option<usize>,
    /// Count of sampling prop rows, not their weights/codes.
    pub sampling_rows: Option<usize>,
    /// APK Boolean.parseBoolean semantics for delta_update, default false.
    pub delta_update: Option<bool>,
    /// Nonempty hash presence only; the hash itself remains private.
    pub hash_present: Option<bool>,
    /// Last returned 35661 value parsed as signed int != 0; never a default.
    pub removal_value: Option<bool>,
    /// Last returned removal value failed conversion; no earlier duplicate fallback.
    pub removal_value_invalid: bool,
    /// Last returned 15491 value; not the effective counterpart-suppression gate.
    #[serde(default)]
    pub token_counterpart_value: Option<bool>,
    /// Invalid last 15491 value; no fallback to an earlier duplicate/default.
    #[serde(default)]
    pub token_counterpart_value_invalid: bool,
    /// Last returned 14303 value; not the effective pending-job namespace gate.
    #[serde(default)]
    pub token_pending_namespace_value: Option<bool>,
    /// Invalid last 14303 value; no fallback to an earlier duplicate/default.
    #[serde(default)]
    pub token_pending_namespace_value_invalid: bool,
    /// Last returned 21198 (`PQ_KEYS_UPLOAD_CODE`) value; whether the server
    /// has this account upload Kyber prekeys. Not the effective gate.
    #[serde(default)]
    pub pq_keys_upload_value: Option<bool>,
    /// Invalid last 21198 value; no fallback to an earlier duplicate/default.
    #[serde(default)]
    pub pq_keys_upload_value_invalid: bool,
}
fn text<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    node.get_attr(name).and_then(|v| v.as_text())
}
fn integer(text: &str) -> Option<i32> {
    super::mex::java_decimal_long(text).and_then(|v| i32::try_from(v).ok())
}
fn parse(reply: &Node, view: &mut AbPropsInspection) -> std::result::Result<(), AbPropsFailure> {
    let mut children = reply
        .child_nodes()
        .iter()
        .filter(|n| n.description == "props");
    let props = children.next().ok_or(AbPropsFailure::Props)?;
    if children.next().is_some() {
        return Err(AbPropsFailure::Props);
    }
    for name in ["hash", "delta_update"] {
        if props.get_attr(name).is_some() && text(props, name).is_none() {
            return Err(AbPropsFailure::Props);
        }
    }
    // IWy.C8f 0098..00dc: Boolean.parseBoolean, not numeric coercion.
    let delta = text(props, "delta_update").is_some_and(|v| v.eq_ignore_ascii_case("true"));
    let hash = text(props, "hash").is_some_and(|v| !v.is_empty());
    let mut count = 0usize;
    let mut sampling = 0usize;
    let mut removal = None;
    let mut token_buckets = BTreeMap::new();
    let mut token_counterpart: Option<&str> = None;
    let mut token_pending_namespace: Option<&str> = None;
    let mut pq_keys_upload: Option<&str> = None;
    for row in props
        .child_nodes()
        .iter()
        .filter(|n| n.description == "prop")
    {
        if count + sampling >= 16384 {
            return Err(AbPropsFailure::PropertySet);
        }
        let code = text(row, "config_code").filter(|v| !v.is_empty());
        if let Some(code) = code {
            let code = integer(code).ok_or(AbPropsFailure::ConfigCode)?;
            let value = text(row, "config_value").ok_or(AbPropsFailure::ConfigValue)?;
            if value.len() > 65536 {
                return Err(AbPropsFailure::ConfigValue);
            }
            count += 1;
            // IWy.C8f 0164 SparseArray.append replaces duplicate IDs; 0Ci.A00
            // converts after collection, so an invalid last row hides earlier ones.
            match code {
                // 43.72 0hu.A01/A02 and 08Y.A0Q: registered integer
                // inputs; inspect returned rows without resolving MobileConfig.
                865 | 909 | 996 | 997 => {
                    token_buckets.insert(code, integer(value));
                }
                REMOVAL_PROPERTY => removal = Some(value),
                TOKEN_COUNTERPART_PROPERTY => token_counterpart = Some(value),
                TOKEN_PENDING_NAMESPACE_PROPERTY => token_pending_namespace = Some(value),
                PQ_KEYS_UPLOAD_PROPERTY => pq_keys_upload = Some(value),
                _ => {}
            }
        } else {
            for name in ["event_code", "sampling_weight"] {
                text(row, name)
                    .and_then(integer)
                    .ok_or(AbPropsFailure::Sampling)?;
            }
            sampling += 1;
        }
    }
    view.configuration_rows = Some(count);
    view.sampling_rows = Some(sampling);
    view.delta_update = Some(delta);
    view.hash_present = Some(hash);
    // IWy.C8f 0340..035c/04c2: a full hashless/empty-hash reply skips values;
    // delta processing does not require a nonempty hash. This is not a cooldown.
    if !delta && !hash {
        view.status = AbPropsStatus::NotModified;
    } else {
        view.status = AbPropsStatus::ConfigurationReceived;
        view.token_bucket_values = token_buckets;
        for (value, parsed, invalid) in [
            (
                removal,
                &mut view.removal_value,
                &mut view.removal_value_invalid,
            ),
            (
                token_counterpart,
                &mut view.token_counterpart_value,
                &mut view.token_counterpart_value_invalid,
            ),
            (
                token_pending_namespace,
                &mut view.token_pending_namespace_value,
                &mut view.token_pending_namespace_value_invalid,
            ),
            (
                pq_keys_upload,
                &mut view.pq_keys_upload_value,
                &mut view.pq_keys_upload_value_invalid,
            ),
        ] {
            if let Some(value) = value {
                // 0Ci.A00 0038..004e, known Boolean A15 membership: int != 0.
                *parsed = integer(value).map(|v| v != 0);
                *invalid = parsed.is_none();
            }
        }
    }
    Ok(())
}

/// Whether the last A/B reply turned `PQ_KEYS_UPLOAD_CODE` (21198) on. Absent
/// or invalid means the `07t.A0L` default, false. MobileConfig is unported.
pub fn pq_keys_upload_enabled(store: &ProtocolStore) -> Result<bool> {
    store.read(pq_keys_upload_enabled_in)
}

/// Read the saved gate within the caller's immutable key/account snapshot.
pub(crate) fn pq_keys_upload_enabled_in(tx: &crate::store::ReadTx<'_>) -> Result<bool> {
    Ok(inspect_in(tx)?.and_then(|view| view.pq_keys_upload_value) == Some(true))
}

/// Read the saved gate in the same transaction that reserves publication bytes.
pub(crate) fn pq_keys_upload_enabled_in_write(tx: &crate::store::WriteTx<'_>) -> Result<bool> {
    let Some(id) = tx.get_json::<String>(Table::Checkpoints, MARKER, 1)? else {
        return Ok(false);
    };
    let Some((outcome, reply, _request)) = actions::view_in_write(tx, &id)? else {
        return Err(Error::invalid("abprops", "saved query action is missing"));
    };
    Ok(inspect_saved(&id, outcome, reply)?.pq_keys_upload_value == Some(true))
}

/// Whether a settled, correlated A/B reply exists. `false` means the variables
/// are still unknown, which is distinct from a settled reply that omits a code
/// (the registered default then applies).
pub(crate) fn settled_reply_in(tx: &crate::store::ReadTx<'_>) -> Result<bool> {
    for key in [FORCED_MARKER, MARKER] {
        let Some(id) = tx.get_json::<String>(Table::Checkpoints, key, 1)? else {
            continue;
        };
        let Some((outcome, reply)) = actions::view_in(tx, &id)? else {
            continue;
        };
        if outcome.kind != "abprops_query" || outcome.status != actions::ActionStatus::Ok {
            continue;
        }
        let Some(reply) = reply else {
            continue;
        };
        if reply.description == "iq"
            && text(&reply, "id") == Some(id.as_str())
            && text(&reply, "type") == Some("result")
            && reply.child("error").is_none()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Can.A04(jid)` for a user-addressable peer: the APK's key-fetch
/// `pqsupport="true"` attribute (writer `X.0v0.A00`) is added only when
///
/// ```text
/// (0DE.A0l(jid) || 0DE.A0T(jid))          // user/LID-style peer
///   && property 21198                      // PQ keys enabled
///   && 0jc.A11()                           // a sent last-resort KEM row exists
///   && property 24160 (0x5e60)             // second gate, default false
/// ```
///
/// A group, status or broadcast address never qualifies.
pub fn pq_support_for(store: &ProtocolStore, peer: &Jid) -> Result<bool> {
    if !peer.is_user_like() {
        return Ok(false);
    }
    if !pq_keys_upload_enabled(store)? {
        return Ok(false);
    }
    if !store.read(super::kem_prekeys::migrated_read_in)? {
        return Ok(false);
    }
    Ok(property_value(store, PQ_SUPPORT_PROPERTY)?.is_some_and(|value| value != 0))
}

/// [`pq_support_for`] inside an existing write transaction.
pub(crate) fn pq_support_for_in_write(tx: &crate::store::WriteTx<'_>, peer: &Jid) -> Result<bool> {
    if !peer.is_user_like() {
        return Ok(false);
    }
    if !pq_keys_upload_enabled_in_write(tx)? {
        return Ok(false);
    }
    if !super::kem_prekeys::migrated_in(tx)? {
        return Ok(false);
    }
    Ok(property_value_in_write(tx, PQ_SUPPORT_PROPERTY)?.is_some_and(|value| value != 0))
}

/// Whether the key query may carry `context_jid` (property 25904, default true).
pub fn context_jid_enabled(store: &ProtocolStore) -> Result<bool> {
    Ok(property_value(store, CONTEXT_JID_PROPERTY)?.is_none_or(|value| value != 0))
}

pub(crate) fn context_jid_enabled_in_write(tx: &crate::store::WriteTx<'_>) -> Result<bool> {
    Ok(property_value_in_write(tx, CONTEXT_JID_PROPERTY)?.is_none_or(|value| value != 0))
}

/// Last returned signed integer for one A/B property code.
///
/// Collection follows the same rules as [`inspect`]: duplicate rows take the
/// last value, an invalid last value hides earlier ones, and a full hashless
/// reply applies nothing. A delta reply contributes only the codes it names;
/// an older settled reply supplies the rest, while a full reply ends the
/// search because it defines the whole set. Only a settled, correlated
/// successful reply counts — queued, reserved-unanswered and refused reads
/// apply nothing. This is not MobileConfig precedence.
pub fn property_value(store: &ProtocolStore, code: i32) -> Result<Option<i32>> {
    store.read(|tx| property_value_in(tx, code))
}

pub(crate) fn property_value_in(tx: &crate::store::ReadTx<'_>, code: i32) -> Result<Option<i32>> {
    let mut candidates = Vec::new();
    for key in [FORCED_MARKER, MARKER] {
        let Some(id) = tx.get_json::<String>(Table::Checkpoints, key, 1)? else {
            continue;
        };
        let Some((outcome, reply)) = actions::view_in(tx, &id)? else {
            continue;
        };
        if outcome.kind != "abprops_query" || outcome.status != actions::ActionStatus::Ok {
            continue;
        }
        let Some(reply) = reply else {
            continue;
        };
        if reply.description != "iq"
            || text(&reply, "id") != Some(id.as_str())
            || text(&reply, "type") != Some("result")
            || reply.child("error").is_some()
        {
            continue;
        }
        candidates.push((outcome.settlement_sequence.unwrap_or(0), reply));
    }
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.0));
    for (_, reply) in candidates {
        match property_in_reply(&reply, code)? {
            PropertyLookup::Named(value) => return Ok(value),
            PropertyLookup::FullWithout => return Ok(None),
            PropertyLookup::DeltaWithout => continue,
        }
    }
    Ok(None)
}

pub(crate) fn property_value_in_write(
    tx: &crate::store::WriteTx<'_>,
    code: i32,
) -> Result<Option<i32>> {
    // The write transaction exposes the same read surface through `ReadTx`
    // conversions used elsewhere; re-read the two markers directly instead of
    // nesting a store read inside the reservation.
    let mut candidates = Vec::new();
    for key in [FORCED_MARKER, MARKER] {
        let Some(id) = tx.get_json::<String>(Table::Checkpoints, key, 1)? else {
            continue;
        };
        let Some((outcome, reply, _request)) = actions::view_in_write(tx, &id)? else {
            continue;
        };
        if outcome.kind != "abprops_query" || outcome.status != actions::ActionStatus::Ok {
            continue;
        }
        let Some(reply) = reply else {
            continue;
        };
        if reply.description != "iq"
            || text(&reply, "id") != Some(id.as_str())
            || text(&reply, "type") != Some("result")
            || reply.child("error").is_some()
        {
            continue;
        }
        candidates.push((outcome.settlement_sequence.unwrap_or(0), reply));
    }
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.0));
    for (_, reply) in candidates {
        match property_in_reply(&reply, code)? {
            PropertyLookup::Named(value) => return Ok(value),
            PropertyLookup::FullWithout => return Ok(None),
            PropertyLookup::DeltaWithout => continue,
        }
    }
    Ok(None)
}

enum PropertyLookup {
    /// The code is named by this reply; `None` means its last value is invalid.
    Named(Option<i32>),
    /// A full reply that does not name the code: the property is unset.
    FullWithout,
    /// A delta reply that does not name the code: an older reply applies.
    DeltaWithout,
}

fn property_in_reply(reply: &Node, code: i32) -> Result<PropertyLookup> {
    let props = reply
        .child("props")
        .ok_or_else(|| Error::invalid("abprops", "saved reply has no unique props child"))?;
    let delta = delta_and_hash(props)?.0;
    let mut seen = None;
    for row in props
        .child_nodes()
        .iter()
        .filter(|node| node.description == "prop")
    {
        let Some(row_code) = text(row, "config_code").filter(|value| !value.is_empty()) else {
            continue;
        };
        let Some(row_code) = integer(row_code) else {
            continue;
        };
        if row_code != code {
            continue;
        }
        // IWy.C8f SparseArray.append: the last duplicate wins, and an invalid
        // last value hides every earlier one.
        seen = Some(text(row, "config_value").and_then(integer));
    }
    Ok(match seen {
        Some(value) => PropertyLookup::Named(value),
        None if delta => PropertyLookup::DeltaWithout,
        // A full reply defines the whole set: an absent code is unset.
        None => PropertyLookup::FullWithout,
    })
}

/// `(delta_update, hash-present)` with the APK's Boolean parse.
fn delta_and_hash(props: &Node) -> Result<(bool, bool)> {
    for name in ["hash", "delta_update"] {
        if props.get_attr(name).is_some() && text(props, name).is_none() {
            return Err(Error::invalid("abprops", "nontext props metadata"));
        }
    }
    Ok((
        text(props, "delta_update").is_some_and(|value| value.eq_ignore_ascii_case("true")),
        text(props, "hash").is_some_and(|value| !value.is_empty()),
    ))
}

/// Inspect saved correlated evidence without queueing or applying feature values.
pub fn inspect(store: &ProtocolStore) -> Result<Option<AbPropsInspection>> {
    store.read(inspect_in)
}

fn inspect_in(tx: &crate::store::ReadTx<'_>) -> Result<Option<AbPropsInspection>> {
    let Some(id) = tx.get_json::<String>(Table::Checkpoints, MARKER, 1)? else {
        return Ok(None);
    };
    let Some((outcome, reply)) = actions::view_in(tx, &id)? else {
        return Err(Error::invalid("abprops", "saved query action is missing"));
    };
    inspect_saved(&id, outcome, reply).map(Some)
}

fn inspect_saved(
    id: &str,
    outcome: actions::ActionOutcome,
    reply: Option<Node>,
) -> Result<AbPropsInspection> {
    if outcome.kind != "abprops_query" {
        return Err(Error::invalid(
            "abprops",
            "saved marker references another action kind",
        ));
    }
    let mut view = AbPropsInspection {
        token_bucket_values: BTreeMap::new(),
        action_id: id.to_owned(),
        status: match outcome.status {
            actions::ActionStatus::Queued => AbPropsStatus::Queued,
            actions::ActionStatus::Sent => AbPropsStatus::OutcomeUnknown,
            _ => AbPropsStatus::InvalidReply,
        },
        failure: None,
        server_error_code: None,
        configuration_rows: None,
        sampling_rows: None,
        delta_update: None,
        hash_present: None,
        removal_value: None,
        removal_value_invalid: false,
        token_counterpart_value: None,
        token_counterpart_value_invalid: false,
        token_pending_namespace_value: None,
        token_pending_namespace_value_invalid: false,
        pq_keys_upload_value: None,
        pq_keys_upload_value_invalid: false,
    };
    let Some(reply) = reply else {
        return Ok(view);
    };
    if reply.description != "iq" || text(&reply, "id") != Some(id) {
        view.failure = Some(AbPropsFailure::Envelope);
    } else if text(&reply, "type") == Some("error")
        || (text(&reply, "type") == Some("result") && reply.child("error").is_some())
    {
        view.status = AbPropsStatus::ServerRefused;
        view.server_error_code = reply
            .child("error")
            .and_then(|n| text(n, "code"))
            .and_then(|s| s.parse().ok());
    } else if text(&reply, "type") != Some("result") {
        view.failure = Some(AbPropsFailure::Envelope);
    } else if let Err(failure) = parse(&reply, &mut view) {
        view.failure = Some(failure);
    }
    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("abprops-fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([28; 32]))),
            create,
        })
        .unwrap()
    }
    fn config(value: &str) -> Node {
        Node::new("prop")
            .attr("config_code", "35661")
            .attr("config_value", value)
    }
    fn reply(id: &str, props: Node) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .children(vec![props])
    }
    fn prop(code: &str, value: &str) -> Node {
        Node::new("prop")
            .attr("config_code", code)
            .attr("config_value", value)
    }
    fn settle(store: &ProtocolStore, id: &str, props: Node) {
        actions::reserve_pending(store).unwrap();
        assert!(
            actions::record_reply(
                store,
                &Node::new("iq")
                    .attr("id", id)
                    .attr("type", "result")
                    .children(vec![props]),
            )
            .unwrap()
        );
    }

    #[test]
    fn context_jid_property_defaults_true_and_follows_the_settled_reply() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("context-jid.redb"), true);
        // 08Y.A0H registers 0x6530 with Boolean.TRUE.
        assert!(context_jid_enabled(&store).unwrap());
        let id = queue(&store).unwrap();
        settle(
            &store,
            &id,
            Node::new("props")
                .attr("hash", "h")
                .children(vec![prop("25904", "0")]),
        );
        assert!(!context_jid_enabled(&store).unwrap());
        let forced = queue_forced(&store).unwrap().unwrap();
        settle(
            &store,
            &forced,
            Node::new("props")
                .attr("hash", "h2")
                .children(vec![prop("25904", "1")]),
        );
        assert!(context_jid_enabled(&store).unwrap());
    }

    #[test]
    fn property_lookup_applies_last_value_and_delta_precedence() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("property.redb"), true);
        // Nothing saved: no value, no default invented here.
        assert_eq!(property_value(&store, 2983).unwrap(), None);
        let id = queue(&store).unwrap();
        // Queued but not settled: applies nothing.
        assert_eq!(property_value(&store, 2983).unwrap(), None);
        let props = Node::new("props").attr("hash", "h").children(vec![
            prop("2983", "900"),
            prop("2983", "1200"),
            prop("21201", "7"),
        ]);
        settle(&store, &id, props);
        assert_eq!(property_value(&store, 2983).unwrap(), Some(1200));
        assert_eq!(property_value(&store, 21201).unwrap(), Some(7));
        assert_eq!(property_value(&store, 21200).unwrap(), None);

        // A later delta names only the code it changes; an unmentioned code
        // keeps the older settled value.
        let forced = queue_forced(&store).unwrap().unwrap();
        assert_ne!(forced, id);
        let delta = Node::new("props")
            .attr("delta_update", "true")
            .children(vec![prop("21201", "3")]);
        settle(&store, &forced, delta);
        assert_eq!(property_value(&store, 21201).unwrap(), Some(3));
        assert_eq!(property_value(&store, 2983).unwrap(), Some(1200));

        // A newer full reply replaces the whole set: unmentioned codes are unset.
        let newer = queue_forced(&store).unwrap().unwrap();
        settle(
            &store,
            &newer,
            Node::new("props")
                .attr("hash", "h2")
                .children(vec![prop("2983", "60")]),
        );
        assert_eq!(property_value(&store, 2983).unwrap(), Some(60));
        assert_eq!(property_value(&store, 21201).unwrap(), None);
    }

    #[test]
    fn property_lookup_hides_invalid_last_values_and_ignores_unsettled_replies() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("property-invalid.redb"), true);
        let id = queue(&store).unwrap();
        settle(
            &store,
            &id,
            Node::new("props")
                .attr("hash", "h")
                .children(vec![prop("2983", "900"), prop("2983", "not-an-int")]),
        );
        // IWy.C8f collects then converts, so the invalid last duplicate hides
        // the earlier valid one.
        assert_eq!(property_value(&store, 2983).unwrap(), None);
        // A refused reply never applies.
        let forced = queue_forced(&store).unwrap().unwrap();
        actions::reserve_pending(&store).unwrap();
        actions::record_reply(
            &store,
            &Node::new("iq")
                .attr("id", forced.as_str())
                .attr("type", "error")
                .children(vec![Node::new("error").attr("code", "400")]),
        )
        .unwrap();
        assert_eq!(property_value(&store, 2983).unwrap(), None);
    }

    #[test]
    fn forced_fetch_is_triggered_by_the_first_child_tag_not_the_type_attribute() {
        let with_child = |children: Vec<Node>| {
            Node::new("notification")
                .attr("type", "abprops")
                .children(children)
        };
        let abprops = Node::new("abprops");
        assert!(forced_fetch_requested(&with_child(vec![abprops.clone()])));
        // The APK reads only the first child (`X.0Tp.A0E(0)`).
        assert!(!forced_fetch_requested(&with_child(vec![
            Node::new("log"),
            abprops.clone(),
        ])));
        assert!(!forced_fetch_requested(&with_child(vec![])));
        assert!(!forced_fetch_requested(&Node::new("abprops")));
        // An outer type attribute alone is not the trigger.
        assert!(!forced_fetch_requested(
            &Node::new("notification").attr("type", "abprops")
        ));
    }

    #[test]
    fn forced_fetch_queues_the_initial_shape_once_and_allows_a_settled_refetch() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("forced.redb"), true);
        let id = queue_forced(&store).unwrap().expect("first forced read");
        let (outcome, _reply) = store
            .read(|tx| actions::view_in(tx, &id))
            .unwrap()
            .expect("queued action");
        let request = actions::request(&store, &id)
            .unwrap()
            .expect("request node");
        assert_eq!(outcome.kind, "abprops_query");
        assert_eq!(outcome.status, actions::ActionStatus::Queued);
        // Forced branch skips the saved hash, so the node matches the initial read.
        assert_eq!(request.description, "iq");
        assert_eq!(
            request.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("abt")
        );
        let props = request.child("props").expect("props child");
        assert_eq!(
            props.get_attr("protocol").and_then(|v| v.as_text()),
            Some("1")
        );
        assert_eq!(props.get_attr("hash").and_then(|v| v.as_text()), Some(""));
        // Still queued: the same action is returned for the caller's drain.
        assert_eq!(queue_forced(&store).unwrap().as_deref(), Some(id.as_str()));
        // Reserved but unanswered: never repeat.
        actions::reserve_pending(&store).unwrap();
        assert_eq!(queue_forced(&store).unwrap(), None);
        // A settled successful read lets a later notification queue a new IQ.
        actions::record_reply(&store, &reply(&id, Node::new("props"))).unwrap();
        let next = queue_forced(&store)
            .unwrap()
            .expect("refetch after success");
        assert_ne!(next, id);
        // A refused read is not repeated either.
        actions::reserve_pending(&store).unwrap();
        actions::record_reply(
            &store,
            &Node::new("iq")
                .attr("id", next.as_str())
                .attr("type", "error")
                .children(vec![Node::new("error").attr("code", "400")]),
        )
        .unwrap();
        assert_eq!(queue_forced(&store).unwrap(), None);
    }

    #[test]
    fn pq_gate_uses_the_callers_snapshot_and_uncommitted_reservation_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pq-snapshot.redb");
        let store = open(&path, true);
        let id = queue(&store).unwrap();
        actions::reserve_pending(&store).unwrap();
        actions::record_reply(
            &store,
            &reply(
                &id,
                Node::new("props").attr("hash", "fixture").children(vec![
                    Node::new("prop")
                        .attr("config_code", "21198")
                        .attr("config_value", "1"),
                ]),
            ),
        )
        .unwrap();
        assert!(pq_keys_upload_enabled(&store).unwrap());
        // A nested store.read here would see the committed true gate, not the
        // caller's pending deletion. Reservation must use its own WriteTx.
        let refused: Result<()> = store.write(|tx| {
            assert!(pq_keys_upload_enabled_in_write(tx)?);
            tx.delete(Table::Checkpoints, MARKER)?;
            assert!(!pq_keys_upload_enabled_in_write(tx)?);
            Err(Error::invalid("fixture", "rollback"))
        });
        assert!(refused.is_err());
        assert!(pq_keys_upload_enabled(&store).unwrap());
        // Hold an old read snapshot while a concurrent generation of local
        // config/key state commits. Digest must not combine the two snapshots.
        store
            .read(|tx| {
                assert!(pq_keys_upload_enabled_in(tx)?);
                store.write(|writer| {
                    writer.delete(Table::Checkpoints, MARKER)?;
                    writer.put_json(Table::Prekeys, b"snapshot-fixture", 1, &true)
                })?;
                assert!(pq_keys_upload_enabled_in(tx)?);
                assert!(
                    tx.get_json::<bool>(Table::Prekeys, b"snapshot-fixture", 1)?
                        .is_none()
                );
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = open(&path, false);
        assert!(!pq_keys_upload_enabled(&store).unwrap());
        assert!(
            store
                .read(|tx| Ok(
                    tx.get_json::<bool>(Table::Prekeys, b"snapshot-fixture", 1)? == Some(true)
                ))
                .unwrap()
        );
    }

    #[test]
    fn once_only_correlated_read_survives_reopen_and_preserves_private_state() {
        for refused in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("read.redb");
            let store = open(&path, true);
            assert!(inspect(&store).unwrap().is_none());
            store
                .write(|tx| {
                    tx.put_json(Table::PeerIdentities, b"private-pin", 1, &vec![7u8; 33])?;
                    tx.put_json(Table::SignalSessions, b"private-session", 1, &vec![8u8; 50])?;
                    tx.put_json(Table::Checkpoints, b"devices:private", 1, &"unchanged")
                })
                .unwrap();
            let id = queue(&store).unwrap();
            let request = actions::request(&store, &id).unwrap().unwrap();
            assert_eq!(text(&request, "xmlns"), Some("abt"));
            assert_eq!(
                request.get_attr("to").and_then(|v| v.as_jid()),
                Some(&Jid::parse("s.whatsapp.net").unwrap())
            );
            let props = request.child("props").unwrap();
            assert_eq!(text(props, "protocol"), Some("1"));
            assert_eq!(text(props, "hash"), Some(""));
            assert!(props.get_attr("refresh_id").is_none());
            assert_eq!(queue(&store).unwrap(), id);
            assert_eq!(
                inspect(&store).unwrap().unwrap().status,
                AbPropsStatus::Queued
            );
            assert_eq!(actions::reserve_pending(&store).unwrap().len(), 1);
            assert_eq!(
                inspect(&store).unwrap().unwrap().status,
                AbPropsStatus::OutcomeUnknown
            );
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
            let node = if refused {
                Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "error")
                    .children(vec![
                        Node::new("error")
                            .attr("code", "503")
                            .attr("text", "private-error"),
                    ])
            } else {
                reply(
                    &id,
                    Node::new("props")
                        .attr("hash", "private-hash")
                        .attr("ab_key", "private-key")
                        .children(vec![
                            config("-1"),
                            Node::new("prop")
                                .attr("config_code", "15491")
                                .attr("config_value", "1"),
                            Node::new("prop")
                                .attr("config_code", "14303")
                                .attr("config_value", "0"),
                            Node::new("prop")
                                .attr("config_code", "865")
                                .attr("config_value", "+604800"),
                            Node::new("prop")
                                .attr("config_code", "909")
                                .attr("config_value", "4"),
                            Node::new("prop")
                                .attr("config_code", "996")
                                .attr("config_value", "604800"),
                            Node::new("prop")
                                .attr("config_code", "997")
                                .attr("config_value", "4"),
                        ]),
                )
            };
            assert!(actions::record_reply(&store, &node).unwrap());
            let view = inspect(&store).unwrap().unwrap();
            assert_eq!(
                view.status,
                if refused {
                    AbPropsStatus::ServerRefused
                } else {
                    AbPropsStatus::ConfigurationReceived
                }
            );
            assert_eq!(view.removal_value, if refused { None } else { Some(true) });
            assert_eq!(
                view.token_counterpart_value,
                if refused { None } else { Some(true) }
            );
            assert_eq!(
                view.token_pending_namespace_value,
                if refused { None } else { Some(false) }
            );
            assert_eq!(
                view.token_bucket_values,
                if refused {
                    BTreeMap::new()
                } else {
                    BTreeMap::from([
                        (865, Some(604800)),
                        (909, Some(4)),
                        (996, Some(604800)),
                        (997, Some(4)),
                    ])
                }
            );
            let encoded = serde_json::to_string(&view).unwrap();
            for secret in ["private-hash", "private-key", "private-error"] {
                assert!(!encoded.contains(secret));
            }
            assert_eq!(queue(&store).unwrap(), id);
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
            store
                .read(|tx| {
                    assert_eq!(
                        tx.get_json::<Vec<u8>>(Table::PeerIdentities, b"private-pin", 1)?,
                        Some(vec![7; 33])
                    );
                    assert_eq!(
                        tx.get_json::<Vec<u8>>(Table::SignalSessions, b"private-session", 1)?,
                        Some(vec![8; 50])
                    );
                    assert_eq!(
                        tx.get_json::<String>(Table::Checkpoints, b"devices:private", 1)?,
                        Some("unchanged".into())
                    );
                    Ok(())
                })
                .unwrap();
            drop(store);
            let store = open(&path, false);
            assert_eq!(inspect(&store).unwrap().unwrap(), view);
            assert_eq!(queue(&store).unwrap(), id);
        }
    }
    fn view() -> AbPropsInspection {
        AbPropsInspection {
            token_bucket_values: BTreeMap::new(),
            action_id: "TEST".into(),
            status: AbPropsStatus::InvalidReply,
            failure: None,
            server_error_code: None,
            configuration_rows: None,
            sampling_rows: None,
            delta_update: None,
            hash_present: None,
            removal_value: None,
            removal_value_invalid: false,
            token_counterpart_value: None,
            token_counterpart_value_invalid: false,
            token_pending_namespace_value: None,
            token_pending_namespace_value_invalid: false,
            pq_keys_upload_value: None,
            pq_keys_upload_value_invalid: false,
        }
    }
    #[test]
    fn token_bucket_evidence_keeps_last_invalid_zero_negative_and_hash_rules() {
        let row = |code: i32, value: &str| {
            Node::new("prop")
                .attr("config_code", code.to_string())
                .attr("config_value", value)
        };
        for (hash, delta, applied) in [
            ("", "false", false),
            ("", "true", true),
            ("h", "false", true),
        ] {
            let mut evidence = view();
            parse(
                &reply(
                    "TEST",
                    Node::new("props")
                        .attr("hash", hash)
                        .attr("delta_update", delta)
                        .children(vec![
                            row(865, "604800"),
                            row(909, "4"),
                            row(996, "0"),
                            row(997, "-1"),
                            row(865, "2147483648"),
                            row(909, " 4"),
                            row(123456, "private-unselected-value"),
                        ]),
                ),
                &mut evidence,
            )
            .unwrap();
            let expected = if applied {
                BTreeMap::from([(865, None), (909, None), (996, Some(0)), (997, Some(-1))])
            } else {
                BTreeMap::new()
            };
            assert_eq!(evidence.token_bucket_values, expected);
            let encoded = serde_json::to_string(&evidence).unwrap();
            assert!(!encoded.contains("private-unselected-value"));
            assert!(!encoded.contains("604800"));
            assert_eq!(
                serde_json::from_str::<AbPropsInspection>(&encoded).unwrap(),
                evidence
            );
        }
    }

    #[test]
    fn token_boolean_evidence_uses_last_values_without_defaults_or_policy_application() {
        let token = |code: i32, value: &str| {
            Node::new("prop")
                .attr("config_code", code.to_string())
                .attr("config_value", value)
        };
        for delta in [false, true] {
            let mut evidence = view();
            parse(
                &reply(
                    "TEST",
                    Node::new("props")
                        .attr("hash", if delta { "" } else { "private-hash" })
                        .attr("delta_update", if delta { "true" } else { "false" })
                        .children(vec![
                            token(15491, "0"),
                            token(15491, "-1"),
                            token(14303, "1"),
                            token(14303, "0"),
                            token(21198, "1"),
                        ]),
                ),
                &mut evidence,
            )
            .unwrap();
            assert_eq!(evidence.token_counterpart_value, Some(true));
            assert_eq!(evidence.token_pending_namespace_value, Some(false));
            assert_eq!(evidence.pq_keys_upload_value, Some(true));
            assert!(!evidence.token_counterpart_value_invalid);
            assert!(!evidence.token_pending_namespace_value_invalid);
            let mut invalid = view();
            parse(
                &reply(
                    "TEST",
                    Node::new("props")
                        .attr("hash", "private-hash")
                        .children(vec![
                            token(15491, "1"),
                            token(15491, "true"),
                            token(14303, "1"),
                            token(14303, "2147483648"),
                        ]),
                ),
                &mut invalid,
            )
            .unwrap();
            assert_eq!(invalid.token_counterpart_value, None);
            assert_eq!(invalid.token_pending_namespace_value, None);
            assert!(invalid.token_counterpart_value_invalid);
            assert!(invalid.token_pending_namespace_value_invalid);
        }
        for props in [
            Node::new("props").attr("hash", "private-hash"),
            Node::new("props").children(vec![token(15491, "1"), token(14303, "1")]),
        ] {
            let mut evidence = view();
            parse(&reply("TEST", props), &mut evidence).unwrap();
            assert!(evidence.token_bucket_values.is_empty());
            assert_eq!(evidence.token_counterpart_value, None);
            assert_eq!(evidence.token_pending_namespace_value, None);
            assert!(!evidence.token_counterpart_value_invalid);
            assert!(!evidence.token_pending_namespace_value_invalid);
        }
    }

    #[test]
    fn legacy_abprops_inspection_has_unknown_token_values() {
        let legacy = r#"{"action_id":"TEST","status":"configuration_received","failure":null,"server_error_code":null,"configuration_rows":1,"sampling_rows":0,"delta_update":false,"hash_present":true,"removal_value":false,"removal_value_invalid":false}"#;
        let evidence: AbPropsInspection = serde_json::from_str(legacy).unwrap();
        assert!(evidence.token_bucket_values.is_empty());
        assert_eq!(evidence.token_counterpart_value, None);
        assert_eq!(evidence.token_pending_namespace_value, None);
        assert!(!evidence.token_counterpart_value_invalid);
        assert!(!evidence.token_pending_namespace_value_invalid);
    }

    #[test]
    fn known_boolean_conversion_matches_independent_java_corpus() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-integrator-reference.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let mut view = view();
            parse(
                &reply(
                    "TEST",
                    Node::new("props")
                        .attr("hash", "fixture-hash")
                        .children(vec![config(case["integrator"].as_str().unwrap())]),
                ),
                &mut view,
            )
            .unwrap();
            assert_eq!(
                view.removal_value,
                case["java_integer"].as_i64().map(|v| v != 0)
            );
            assert_eq!(view.removal_value_invalid, case["java_integer"].is_null());
        }
    }
    #[test]
    fn hashless_full_delta_duplicates_and_sampling_follow_callback_branches() {
        for (hash, delta, expected) in [
            ("", "false", None),
            ("", "TrUe", Some(false)),
            ("fixture", "false", Some(false)),
        ] {
            let mut view = view();
            let node = reply(
                "TEST",
                Node::new("props")
                    .attr("hash", hash)
                    .attr("delta_update", delta)
                    .children(vec![
                        config("1"),
                        config("0"),
                        Node::new("prop")
                            .attr("event_code", "-1")
                            .attr("sampling_weight", "0"),
                    ]),
            );
            parse(&node, &mut view).unwrap();
            assert_eq!(view.removal_value, expected);
            assert_eq!(view.configuration_rows, Some(2));
            assert_eq!(view.sampling_rows, Some(1));
            assert_eq!(
                view.status,
                if hash.is_empty() && delta == "false" {
                    AbPropsStatus::NotModified
                } else {
                    AbPropsStatus::ConfigurationReceived
                }
            );
        }
        let mut view = view();
        parse(
            &reply(
                "TEST",
                Node::new("props")
                    .attr("hash", "fixture")
                    .children(vec![config("1"), config("true")]),
            ),
            &mut view,
        )
        .unwrap();
        assert_eq!(view.removal_value, None);
        assert!(view.removal_value_invalid);
    }
    #[test]
    fn malformed_rows_refuse_without_partial_values() {
        for (props, expected) in [
            (
                Node::new("props").children(vec![
                    Node::new("prop")
                        .attr("config_code", "2147483648")
                        .attr("config_value", "1"),
                ]),
                AbPropsFailure::ConfigCode,
            ),
            (
                Node::new("props").children(vec![Node::new("prop").attr("config_code", "35661")]),
                AbPropsFailure::ConfigValue,
            ),
            (
                Node::new("props").children(vec![Node::new("prop").attr("event_code", "1")]),
                AbPropsFailure::Sampling,
            ),
            (
                Node::new("props").children(vec![config(&"0".repeat(65537))]),
                AbPropsFailure::ConfigValue,
            ),
            (
                Node::new("props").children(vec![config("1"); 16385]),
                AbPropsFailure::PropertySet,
            ),
        ] {
            let mut view = view();
            assert_eq!(parse(&reply("TEST", props), &mut view), Err(expected));
            assert_eq!(view.removal_value, None);
            assert_eq!(view.configuration_rows, None);
        }
        let mut view = view();
        assert_eq!(
            parse(
                &Node::new("iq").children(vec![Node::new("props"), Node::new("props")]),
                &mut view
            ),
            Err(AbPropsFailure::Props)
        );
    }
}
