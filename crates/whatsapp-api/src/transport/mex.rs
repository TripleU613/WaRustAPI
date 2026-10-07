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
