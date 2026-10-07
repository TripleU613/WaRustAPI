//! APK-derived JSON/XMPP MEX request construction (not a live dispatcher).
//!
//! Release document IDs are caller-supplied extracted data. This module does
//! not substitute classic group IQs, choose Pando/Argo feature branches, or
//! interpret an IQ result as successful GraphQL/group-state application.
//! See `docs/protocol/mex.md` for 2.26.37.71 sources and remaining gaps.

use crate::{
    Error, Result,
    binary::Node,
    model::jid::{Jid, Server},
};

/// Local JSON response budget. This is not an APK/server payload limit.
pub const MAX_JSON_RESPONSE_BYTES: usize = 1024 * 1024;

/// Presence/type of one selected JSON response field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonFieldState {
    /// The field was absent.
    Missing,
    /// Explicit JSON null, accepted by the APK's nullable field getter.
    Null,
    /// The expected object (`data`) or array (`errors`) was present.
    Valid,
    /// The field was present with an unexpected type; its meaning is unknown.
    Invalid,
}

/// Counts/presence only: no group identifiers, content or server error text.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MexResponseSummary {
    /// Selected payload child (`result` takes precedence over `update`).
    pub payload_child: &'static str,
    /// Object-field state; empty object is valid, not automatically success.
    pub data_state: JsonFieldState,
    /// Number of top-level data fields, not participants or complete groups.
    pub data_field_count: usize,
    /// Error-array state; invalid differs from an absent/empty error array.
    pub errors_state: JsonFieldState,
    /// Number of opaque error array entries, without interpreting their schema.
    pub error_count: usize,
}

/// Parsed JSON response parts. Debug output is redacted to the summary.
///
/// Independent field failures preserve the other part. This does not indicate
/// semantic success or implement group-model conversion. The APK's malformed
/// errors fallback contains Java exception strings; this port keeps an explicit
/// invalid state rather than fabricating those strings or treating it as empty.
pub struct MexJsonResponse {
    data: Option<serde_json::Map<String, serde_json::Value>>,
    errors: Option<Vec<serde_json::Value>>,
    summary: MexResponseSummary,
}

/// APK 37.71 QueryGroupInfo callback gate, before the group success callback runs.
/// This is not a success verdict or permission to overwrite a saved snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupInfoDisposition {
    /// A selected field has an unsupported type; Java exception conversion is not modeled.
    UnsupportedParts,
    /// Missing/null data cannot construct the nonnull JSONObject-backed model.
    MissingModel,
    /// I1H's getJSONObject rejects at least one error array entry.
    InvalidErrorEntry,
    /// Group callback returns false after error handling, even with partial data.
    ServerErrors,
    /// Empty model and no errors: the inherited callback does not invoke A04.
    EmptyData,
    /// Nonempty model without errors reaches A04; feature-gated conversion may still fail.
    InvokeGroupCallback,
}

/// Confirmed numeric projection before full group conversion/application.
/// Values may reveal private group timing; Debug hides them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GroupNumericMetadata {
    /// ADv.A04 -> 8kY.A0B, signed parsed creation_time multiplied by 1000.
    /// Storage units are not established by this projection.
    pub creation_scaled: i64,
    /// ADv.A05 -> 8kY.A0B, nullable subject.creation_time multiplied by 1000.
    pub subject_creation_scaled: i64,
    /// ADv.A03 -> 0Gv.A01, unscaled announcement_version.
    pub announcement_version: i64,
}

impl std::fmt::Debug for GroupNumericMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupNumericMetadata")
            .finish_non_exhaustive()
    }
}

// Host Java 21 Character.digit(char,10), checked across all 65536 UTF-16 units.
// Source/reference corpus: fixtures/mex/2.26.37.71-numeric-reference.json.
// This is platform-reference data, not a digit table recovered from the APK.
const BMP_DECIMAL_ZEROES: [u16; 37] = [
    48, 1632, 1776, 1984, 2406, 2534, 2662, 2790, 2918, 3046, 3174, 3302, 3430, 3558, 3664, 3792,
    3872, 4160, 4240, 6112, 6160, 6470, 6608, 6784, 6800, 6992, 7088, 7232, 7248, 42528, 43216,
    43264, 43472, 43504, 43600, 44016, 65296,
];

fn decimal_digit(unit: u16) -> Option<u16> {
    let index = BMP_DECIMAL_ZEROES.partition_point(|&start| start <= unit);
    let zero = *BMP_DECIMAL_ZEROES.get(index.checked_sub(1)?)?;
    let digit = unit - zero;
    (digit < 10).then_some(digit)
}

fn java_decimal_long(text: &str) -> Option<i64> {
    // Long.parseLong iterates UTF-16 chars, so supplementary-plane digit
    // scalars are not accepted as one character. Do not trim or accept radix prefixes.
    let mut units = text.encode_utf16().peekable();
    let negative = match units.peek().copied()? {
        0x2d => {
            units.next();
            true
        }
        0x2b => {
            units.next();
            false
        }
        _ => false,
    };
    let mut value = 0i64;
    let mut any = false;
    // Accumulate negatively to represent MIN_VALUE without positive overflow.
    for unit in units {
        any = true;
        value = value
            .checked_mul(10)?
            .checked_sub(i64::from(decimal_digit(unit)?))?;
    }
    if !any {
        return None;
    }
    if negative {
        Some(value)
    } else {
        value.checked_neg()
    }
}

fn group_long_zero(value: Option<&serde_json::Value>) -> i64 {
    // APK 0Gv.A01 catches NumberFormatException and returns supplied zero.
    // LbE.A0E/A0F -> JSONObject.optString -> JSON.toString/String.valueOf.
    // Wire JSON integral Long values preserve signed precision. Double values
    // stringify with fraction/exponent, and objects/arrays/bools/null cannot
    // parse as decimal long; their numeric result is zero without copying content.
    match value {
        Some(serde_json::Value::String(text)) => java_decimal_long(text).unwrap_or(0),
        Some(serde_json::Value::Number(number)) => number.as_i64().unwrap_or(0),
        _ => 0,
    }
}

impl std::fmt::Debug for MexJsonResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.summary.fmt(f)
    }
}

impl MexJsonResponse {
    /// Parsed data object, if valid. Contains private server/group content.
    pub fn data(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.data.as_ref()
    }

    /// Opaque error entries, if valid; missing/null fields yield an empty slice.
    /// Contains private server error content. Invalid type returns None.
    pub fn errors(&self) -> Option<&[serde_json::Value]> {
        self.errors.as_deref()
    }

    /// Redacted presence/type/count inspection, without a success verdict.
    pub fn summary(&self) -> &MexResponseSummary {
        &self.summary
    }

    /// Project the APK's three confirmed time/version numbers, without applying
    /// a snapshot, parsing membership or claiming complete group validity.
    /// Refuses blocked callback gates or missing/nonobject query data. The
    /// caller's later feature gate and all other converter requirements remain.
    pub fn group_numeric_metadata(&self) -> Result<GroupNumericMetadata> {
        let disposition = self.group_info_disposition();
        if disposition != GroupInfoDisposition::InvokeGroupCallback {
            return Err(Error::invalid(
                "mex.group.numeric_metadata",
                format!("callback gate blocks numeric projection: {disposition:?}"),
            ));
        }
        // 0Xo.A03+000c..0026 requires this JSONObject-backed 2FT.
        let group = self
            .data()
            .and_then(|data| data.get("xwa2_group_query_by_id"))
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                Error::invalid(
                    "mex.group.numeric_metadata",
                    "missing/nonobject xwa2_group_query_by_id",
                )
            })?;
        // 90x.A0J uses nullable optJSONObject, including wrong-type fallback.
        let subject = group.get("subject").and_then(serde_json::Value::as_object);
        Ok(GroupNumericMetadata {
            // ADv.A04/A05 -> 8kY.A0B, literal mul-long/2addr 1000; Java
            // overflow wraps, unlike Rust debug-mode multiplication.
            creation_scaled: group_long_zero(group.get("creation_time")).wrapping_mul(1000),
            subject_creation_scaled: group_long_zero(subject.and_then(|s| s.get("creation_time")))
                .wrapping_mul(1000),
            // ADv.A03 directly invokes 0Gv.A01; A4m stores in distinct A08.
            announcement_version: group_long_zero(group.get("announcement_version")),
        })
    }

    /// Inspect the confirmed group callback gate without running callbacks,
    /// advancing rate guards or applying state. Valid error objects remain
    /// opaque: no error code, retry duration or exception hierarchy is guessed.
    pub fn group_info_disposition(&self) -> GroupInfoDisposition {
        // LaZ independent parser failures have their own exception/fallback
        // paths. Do not equate the local explicit-invalid state with no errors.
        if self.summary.data_state == JsonFieldState::Invalid
            || self.summary.errors_state == JsonFieldState::Invalid
        {
            return GroupInfoDisposition::UnsupportedParts;
        }
        // LaT.A00 -> 2FU.<init> requires a nonnull JSONObject before LZv,
        // even when a valid server error array accompanies missing/null data.
        let Some(data) = &self.data else {
            return GroupInfoDisposition::MissingModel;
        };
        let Some(errors) = &self.errors else {
            return GroupInfoDisposition::UnsupportedParts;
        };
        // Lac.BEE and BaseMexCallback.A03 construct I1H; its constructor
        // +004c..0064 calls JSONArray.getJSONObject for every array entry.
        if errors.iter().any(|entry| !entry.is_object()) {
            return GroupInfoDisposition::InvalidErrorEntry;
        }
        // 9Np.A06 selector 0 always ends at +0028..002c (false), including
        // the 429 timestamp/runnable branch. BaseMexCallback.A03+005e..006a
        // therefore never dispatches partial group data after any error array.
        if !errors.is_empty() {
            return GroupInfoDisposition::ServerErrors;
        }
        // LZv.A02 is JSONObject.length()>0; BaseMexCallback.A03 requires it
        // before A04. The later required xwa2_group_query_by_id conversion is
        // distinct: a nonempty object here is not a proven complete group.
        if data.is_empty() {
            GroupInfoDisposition::EmptyData
        } else {
            GroupInfoDisposition::InvokeGroupCallback
        }
    }
}

/// Parse the APK 37.71 JSON MEX envelope and its independent data/errors fields.
///
/// Source: LaT.A02, Lab/LaZ constructors, provider 0x1763 -> 0yN.A00 ->
/// O4r(selector 0), and O4r.Cf4 selectors 2/1. Leading whitespace does not
/// satisfy LaT's JSON branch discriminator. Argo/Pando remain unsupported.
/// Raw JSON/UTF-8 parse failures never echo payloads into errors. No IQ ID
/// matching, action settlement, callbacks or group-state application occurs.
pub fn parse_json_response(node: &Node) -> Result<MexJsonResponse> {
    use crate::binary::NodeContent;
    // LaT.A02+0000..002e and O4r.Cf4+009e..00c4 prefer result over update.
    let (payload_child, payload) = if let Some(child) = node.child("result") {
        ("result", child)
    } else if let Some(child) = node.child("update") {
        ("update", child)
    } else {
        return Err(Error::invalid(
            "mex.response",
            "missing result/update payload child",
        ));
    };
    let body = match &payload.content {
        NodeContent::Text(text) => text.as_bytes(),
        NodeContent::Bytes(bytes) => bytes.as_ref(),
        _ => return Err(Error::invalid("mex.response", "payload is not text/bytes")),
    };
    if body.len() > MAX_JSON_RESPONSE_BYTES {
        return Err(Error::invalid(
            "mex.response",
            "JSON payload exceeds local 1-MiB limit",
        ));
    }
    // LaT.A02+003c..0058 requires length >=2 and literal first character '{'.
    if body.len() < 2 || body.first() != Some(&b'{') {
        return Err(Error::invalid(
            "mex.response",
            "payload does not select the traced JSON branch; Argo/Pando unsupported",
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(body)
        .map_err(|_| Error::invalid("mex.response", "malformed UTF-8 JSON object"))?;
    let serde_json::Value::Object(mut root) = value else {
        return Err(Error::invalid("mex.response", "expected a JSON object"));
    };
    // O4r.Cf4+002a..0056: absent/null is nullable; present requires JSONObject.
    let (data_state, data) = match root.remove("data") {
        None => (JsonFieldState::Missing, None),
        Some(serde_json::Value::Null) => (JsonFieldState::Null, None),
        Some(serde_json::Value::Object(data)) => (JsonFieldState::Valid, Some(data)),
        Some(_) => (JsonFieldState::Invalid, None),
    };
    // O4r.Cf4+0058..008e: absent/null -> new empty JSONArray; other types
    // throw. LaZ.A00 catches this separately from the data parser.
    let (errors_state, errors) = match root.remove("errors") {
        None => (JsonFieldState::Missing, Some(Vec::new())),
        Some(serde_json::Value::Null) => (JsonFieldState::Null, Some(Vec::new())),
        Some(serde_json::Value::Array(errors)) => (JsonFieldState::Valid, Some(errors)),
        Some(_) => (JsonFieldState::Invalid, None),
    };
    let summary = MexResponseSummary {
        payload_child,
        data_state,
        data_field_count: data.as_ref().map_or(0, serde_json::Map::len),
        errors_state,
        error_count: errors.as_ref().map_or(0, Vec::len),
    };
    Ok(MexJsonResponse {
        data,
        errors,
        summary,
    })
}

/// Query contexts produced by APK 37.71 `0Xo.A02`, including its private-set override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupQueryContext {
    /// User-requested information.
    Interactive,
    /// Logical-ID migration.
    LidMigration,
    /// Subject notification recovery.
    NewSubject,
    /// Acknowledgement group mismatch.
    AckGroupMismatch,
    /// Message send failure.
    SendFailure,
    /// Subgroup conflict recovery.
    SubGroupConflictRecovery,
    /// Invite acceptance conflict recovery.
    InviteCode,
    /// Participant hash mismatch.
    Phash,
    /// Participant change recovery.
    ParticipantChangeRecovery,
    /// Prefetch.
    PreFetch,
    /// A reason outside the recognized mapping.
    Unknown,
    /// APK private group-set override; not inferred from a reason string.
    MissingParticipantIdentification,
}

impl GroupQueryContext {
    /// Exact wire spelling, preserved from APK 37.71 `0Xo.A02` string constants.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Interactive => "INTERACTIVE",
            Self::LidMigration => "LID_MIGRATION",
            Self::NewSubject => "NEW_SUBJECT",
            Self::AckGroupMismatch => "ACK_GROUP_MISMATCH",
            Self::SendFailure => "SEND_FAILURE",
            Self::SubGroupConflictRecovery => "SUB_GROUP_CONFLICT_RECOVERY",
            Self::InviteCode => "INVITE_CODE",
            Self::Phash => "PHASH",
            Self::ParticipantChangeRecovery => "PARTICIPANT_CHANGE_RECOVERY",
            Self::PreFetch => "PRE_FETCH",
            Self::Unknown => "UNKNOWN",
            Self::MissingParticipantIdentification => "MISSING_PARTICIPANT_IDENTIFICATION",
        }
    }

    /// Parse an explicit context; never invent an uppercased context from a reason.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "INTERACTIVE" => Ok(Self::Interactive),
            "LID_MIGRATION" => Ok(Self::LidMigration),
            "NEW_SUBJECT" => Ok(Self::NewSubject),
            "ACK_GROUP_MISMATCH" => Ok(Self::AckGroupMismatch),
            "SEND_FAILURE" => Ok(Self::SendFailure),
            "SUB_GROUP_CONFLICT_RECOVERY" => Ok(Self::SubGroupConflictRecovery),
            "INVITE_CODE" => Ok(Self::InviteCode),
            "PHASH" => Ok(Self::Phash),
            "PARTICIPANT_CHANGE_RECOVERY" => Ok(Self::ParticipantChangeRecovery),
            "PRE_FETCH" => Ok(Self::PreFetch),
            "UNKNOWN" => Ok(Self::Unknown),
            "MISSING_PARTICIPANT_IDENTIFICATION" => Ok(Self::MissingParticipantIdentification),
            _ => Err(Error::invalid(
                "mex.query_context",
                "unrecognized APK context",
            )),
        }
    }
}

/// Construct the confirmed 37.71 JSON/XMPP `QueryGroupInfo` envelope offline.
///
/// `doc_id` is the extracted **server** ID (`164.CFb`), not `client_doc_id`.
/// Local validation is stricter than the APK's assertion helper: it refuses
/// empty/non-decimal/overlong IDs before building a node. No account is opened,
/// IQ ID allocated, rate-guard state advanced, or action queued here.
pub fn group_info_request(
    iq_id: &str,
    group: &Jid,
    doc_id: &str,
    context: GroupQueryContext,
) -> Result<Node> {
    if iq_id.is_empty() || iq_id.len() > 256 || iq_id.chars().any(char::is_control) {
        return Err(Error::invalid(
            "mex.iq_id",
            "expected 1..256 bytes without control characters",
        ));
    }
    // 9QZ.<init>(VLLLL)+005c..0076 checks 1..20 Java characters;
    // requiring decimal ASCII here also prevents using the longer client ID.
    if doc_id.is_empty() || doc_id.len() > 20 || !doc_id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::invalid(
            "mex.doc_id",
            "expected 1..20 decimal ASCII digits",
        ));
    }
    // Reparse because Jid's fields are publicly editable. APK 0Xo.A02 receives
    // a typed group JID, never a user/device JID masquerading as group input.
    let group = Jid::parse(&group.to_string())?;
    if group.server != Server::Group
        || group.user.is_empty()
        || group.agent != 0
        || group.device != 0
    {
        return Err(Error::invalid(
            "mex.group_jid",
            "expected a plain nonempty group JID",
        ));
    }
    // 0Xo.A02 -> J8H.A02 -> GraphQlCallInput.A00/A01 recursively maps
    // the input object. Lai.<init>+003e..005a wraps it in queryId/variables;
    // 1xg.A1P invokes JSONObject.toString, not an encoded query or call name.
    let body = serde_json::json!({
        "queryId": doc_id,
        "variables": {"input": {"group_jid": group.to_string(), "query_context": context.as_str()}}
    })
    .to_string();
    // Lai+0060..007a supplies doc_id to BpQ(selector 3). BpQ+004e..008e
    // places that ID, rather than a random telemetry value, in trace/flow_id.
    let trace = Node::new("trace").children(vec![Node::new("flow_id").text(doc_id)]);
    let query = Node::new("query").attr("query_id", doc_id).text(body);
    // 9QZ.<init>(VLLLL)+000e..0038 and 8kY.A1H/1S7.getServer:
    // destination is the server-only JID. Preserve observed attribute order.
    Ok(Node::new("iq")
        .attr("to", Jid::parse("s.whatsapp.net")?)
        .attr("xmlns", "w:mex")
        .attr("type", "get")
        .attr("id", iq_id)
        .children(vec![trace, query]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{AttributeValue, NodeContent};

    fn numeric_reference() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-numeric-reference.json"
        ))
        .unwrap()
    }

    #[test]
    fn group_numeric_projection_matches_independent_java_reference() {
        for case in numeric_reference()["cases"].as_array().unwrap() {
            let text = case["input"].as_str().unwrap();
            let expected = case["parsed"].as_str().unwrap().parse::<i64>().unwrap();
            let scaled = case["scaled"].as_str().unwrap().parse::<i64>().unwrap();
            let body = serde_json::json!({"data": {"xwa2_group_query_by_id": {
                "creation_time": text, "subject": {"creation_time": text},
                "announcement_version": text
            }}});
            let node = Node::new("iq").children(vec![Node::new("result").text(body.to_string())]);
            let numbers = parse_json_response(&node)
                .unwrap()
                .group_numeric_metadata()
                .unwrap();
            assert_eq!(numbers.creation_scaled, scaled, "{text:?}");
            assert_eq!(numbers.subject_creation_scaled, scaled, "{text:?}");
            assert_eq!(numbers.announcement_version, expected, "{text:?}");
            assert_eq!(java_decimal_long(text).unwrap_or(0), expected);
            assert_eq!(format!("{numbers:?}"), "GroupNumericMetadata { .. }");
        }
        let zeroes: Vec<u16> = numeric_reference()["bmp_decimal_zeroes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as u16)
            .collect();
        // Complete host Character.digit(char,10) reference domain, including
        // surrogate units and nondecimal numeric symbols that must be rejected.
        for unit in 0..=u16::MAX {
            let expected = zeroes
                .iter()
                .find_map(|&zero| unit.checked_sub(zero).filter(|&n| n < 10));
            assert_eq!(decimal_digit(unit), expected, "unit {unit:x}");
        }
    }

    #[test]
    fn numeric_json_coercions_preserve_distinct_fields_and_nullable_subject() {
        for (value, expected) in [
            (serde_json::json!(42), 42),
            (serde_json::json!(-42), -42),
            (serde_json::json!(i64::MAX), i64::MAX),
            (serde_json::json!(i64::MIN), i64::MIN),
            (serde_json::json!(9223372036854775808u64), 0),
            (serde_json::json!(1.0), 0),
            (serde_json::json!(1000.0), 0),
            (serde_json::json!(true), 0),
            (serde_json::json!(null), 0),
            (serde_json::json!(["synthetic-private-value"]), 0),
            (serde_json::json!({"private": "synthetic-private-value"}), 0),
        ] {
            assert_eq!(group_long_zero(Some(&value)), expected);
        }
        assert_eq!(group_long_zero(None), 0);
        for subject in [
            serde_json::json!(null),
            serde_json::json!({}),
            serde_json::json!(3),
            serde_json::json!([]),
        ] {
            let body = serde_json::json!({"data": {"xwa2_group_query_by_id": {
                "creation_time": 42, "subject": subject, "announcement_version": "+7"
            }}});
            let node = Node::new("iq").children(vec![Node::new("result").text(body.to_string())]);
            let numbers = parse_json_response(&node)
                .unwrap()
                .group_numeric_metadata()
                .unwrap();
            assert_eq!(numbers.creation_scaled, 42000);
            assert_eq!(numbers.subject_creation_scaled, 0);
            assert_eq!(numbers.announcement_version, 7);
        }
    }

    #[test]
    fn numeric_projection_refuses_blocked_callbacks_and_wrong_query_shape_without_content() {
        for body in [
            serde_json::json!({"data": {"unrelated": "synthetic-private-value"}}),
            serde_json::json!({"data": {"xwa2_group_query_by_id": null}}),
            serde_json::json!({"data": {"xwa2_group_query_by_id": []}}),
            serde_json::json!({"data": {"xwa2_group_query_by_id": {}}, "errors": [{"message":"synthetic-private-value"}]}),
            serde_json::json!({"data": {"xwa2_group_query_by_id": {}}, "errors": ["synthetic-private-value"]}),
            serde_json::json!({"data": {}}),
            serde_json::json!({}),
        ] {
            let node = Node::new("iq").children(vec![Node::new("result").text(body.to_string())]);
            let error = parse_json_response(&node)
                .unwrap()
                .group_numeric_metadata()
                .unwrap_err();
            assert!(!error.to_string().contains("synthetic-private-value"));
        }
    }

    #[test]
    fn group_callback_corpus_blocks_partial_data_errors_and_only_gates_callback() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-group-callback.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let node = Node::new("iq")
                .children(vec![Node::new("result").text(case["payload"].to_string())]);
            let parsed = parse_json_response(&node).unwrap();
            assert_eq!(
                serde_json::to_value(parsed.group_info_disposition()).unwrap(),
                case["group_info_disposition"]
            );
        }
    }

    #[test]
    fn independent_nullable_field_corpus_preserves_partial_results_and_redacts() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-response-parts.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let body = case["payload"].to_string();
            for child in ["result", "update"] {
                let node = Node::new("iq").children(vec![Node::new(child).text(&body)]);
                let parsed = parse_json_response(&node).unwrap();
                let summary = serde_json::to_value(parsed.summary()).unwrap();
                for field in [
                    "data_state",
                    "errors_state",
                    "data_field_count",
                    "error_count",
                ] {
                    assert_eq!(summary[field], case[field]);
                }
                assert_eq!(summary["payload_child"], child);
                assert_eq!(parsed.data().is_some(), case["data_state"] == "valid");
                assert_eq!(parsed.errors().is_some(), case["errors_state"] != "invalid");
                if let Some(data) = parsed.data() {
                    assert_eq!(
                        &serde_json::Value::Object(data.clone()),
                        &case["payload"]["data"]
                    );
                }
                if case["errors_state"] == "valid" {
                    assert_eq!(
                        parsed.errors().unwrap(),
                        case["payload"]["errors"].as_array().unwrap()
                    );
                }
                let debug = format!("{parsed:?}");
                assert!(!debug.contains("synthetic-private"));
                assert!(!summary.to_string().contains("synthetic-private"));
            }
        }
    }

    #[test]
    fn payload_selection_invalid_json_bounds_and_failures_do_not_echo() {
        let node = |body: &str| Node::new("iq").children(vec![Node::new("result").text(body)]);
        assert!(parse_json_response(&Node::new("iq")).is_err());
        for body in [
            "",
            "{",
            " {\"data\":{}}",
            "[]",
            "{\"private-failed-marker\":",
        ] {
            let error = parse_json_response(&node(body)).unwrap_err().to_string();
            assert!(!error.contains("private-failed-marker"));
        }
        let mixed = Node::new("iq").children(vec![
            Node::new("update").text("{\"data\":{}}"),
            Node::new("result").text("{\"errors\":[\"opaque\"]}"),
        ]);
        assert_eq!(
            parse_json_response(&mixed).unwrap().summary().payload_child,
            "result"
        );
        assert_eq!(
            parse_json_response(&mixed).unwrap().summary().error_count,
            1
        );
        let invalid_first = Node::new("iq").children(vec![
            Node::new("result").text("malformed"),
            Node::new("update").text("{}"),
        ]);
        assert!(parse_json_response(&invalid_first).is_err());
        let oversized = "{".to_owned() + &" ".repeat(MAX_JSON_RESPONSE_BYTES);
        assert!(
            parse_json_response(&node(&oversized))
                .unwrap_err()
                .to_string()
                .contains("limit")
        );
        let utf8 =
            Node::new("iq").children(vec![Node::new("result").bytes(vec![b'{', 0xff, b'}'])]);
        assert!(parse_json_response(&utf8).is_err());
    }

    #[test]
    fn literal_apk_envelope_uses_server_id_at_all_three_locations() {
        // Independent asset selection, retained with APK and asset SHA-256.
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-group-queries.json"
        ))
        .unwrap();
        let document = &fixture["operations"]["QueryGroupInfo"];
        let id = document["doc_id"].as_str().unwrap();
        assert_eq!(id, "37759635623635554");
        let node = group_info_request(
            "fixture-iq",
            &Jid::parse("123-456@g.us").unwrap(),
            id,
            GroupQueryContext::Interactive,
        )
        .unwrap();
        assert_eq!(node.description, "iq");
        assert_eq!(
            node.attributes
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>(),
            ["to", "xmlns", "type", "id"]
        );
        assert_eq!(
            node.get_attr("to"),
            Some(&AttributeValue::Jid(Jid::parse("s.whatsapp.net").unwrap()))
        );
        assert_eq!(
            node.get_attr("xmlns"),
            Some(&AttributeValue::Text("w:mex".into()))
        );
        assert_eq!(
            node.get_attr("type"),
            Some(&AttributeValue::Text("get".into()))
        );
        assert_eq!(
            node.child_nodes()
                .iter()
                .map(|c| c.description.as_str())
                .collect::<Vec<_>>(),
            ["trace", "query"]
        );
        assert_eq!(
            node.child("trace")
                .unwrap()
                .child("flow_id")
                .unwrap()
                .content,
            NodeContent::Text("37759635623635554".into())
        );
        let query = node.child("query").unwrap();
        assert_eq!(
            query.get_attr("query_id"),
            Some(&AttributeValue::Text("37759635623635554".into()))
        );
        let NodeContent::Text(body) = &query.content else {
            panic!("missing JSON body")
        };
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"queryId":"37759635623635554", "variables":{"input":{
            "group_jid":"123-456@g.us", "query_context":"INTERACTIVE"}}})
        );
        assert!(
            group_info_request(
                "fixture-iq",
                &Jid::parse("123-456@g.us").unwrap(),
                document["client_doc_id"].as_str().unwrap(),
                GroupQueryContext::Interactive
            )
            .is_err()
        );
    }

    #[test]
    fn invalid_inputs_never_produce_a_request() {
        let group = Jid::parse("123-456@g.us").unwrap();
        for id in ["", "a123", "123456789012345678901"] {
            assert!(
                group_info_request("fixture-iq", &group, id, GroupQueryContext::Interactive)
                    .is_err()
            );
        }
        for id in ["", "id\n", &"a".repeat(257)] {
            assert!(group_info_request(id, &group, "123", GroupQueryContext::Interactive).is_err());
        }
        assert!(
            group_info_request(
                "fixture-iq",
                &Jid::parse("123@s.whatsapp.net").unwrap(),
                "123",
                GroupQueryContext::Interactive
            )
            .is_err()
        );
        assert!(GroupQueryContext::parse("PARTICIPANT_ME_NOT_INCLUDED_CHANGE_RECOVERY").is_err());
        assert_eq!(
            GroupQueryContext::parse("MISSING_PARTICIPANT_IDENTIFICATION")
                .unwrap()
                .as_str(),
            "MISSING_PARTICIPANT_IDENTIFICATION"
        );
    }
}
