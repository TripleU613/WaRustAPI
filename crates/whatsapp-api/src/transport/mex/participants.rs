//! APK 37.71 ADv.A0D -> 32q.A00 -> 31c, without durable application.
//! Source: docs/protocol/mex-group-converter-evidence.txt,
//! mex-participant-model-evidence.txt and mex-member-map-evidence.txt.

use super::{GroupParticipantRole, ParticipantAddress, ParticipantAddressKind, coercion};
use crate::{Error, Result};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// ADv.A0D distinguishes a null phash result from an ordered (possibly empty) map.
/// APK 0yw.A0X -> A0U skips membership reconciliation when this map is null.
/// Other group updates continue; this type does not apply any state.
pub enum GroupParticipantMap {
    /// APK returned null before inspecting edges; not an empty replacement.
    PhashMatch,
    /// Converted unique primary keys in first-insertion order; later edges replace values.
    Members(Vec<GroupParticipant>),
}

/// APK 31c member slots from the two ADv.A0D branches; private identifiers/content.
/// Debug hides all values. No Serialize implementation exports these implicitly.
pub struct GroupParticipant {
    /// 31c.A03 strict primary UserJid, including valid plain bots/interop.
    pub primary: ParticipantAddress,
    /// 31c.A01 strict plain LID alias; bot branch leaves absent.
    pub lid: Option<ParticipantAddress>,
    /// 31c.A02 strict ordinary PhoneUserJid alias; server singletons are not PN aliases.
    pub pn: Option<ParticipantAddress>,
    /// 31c.A07 converted role string (empty/admin/superadmin).
    pub role: &'static str,
    /// 31c.A05 positive optInt boxed as long; raw value before the consumer
    /// multiplies by 1000. Use storage_metadata for stored millisecond selection.
    pub join_time: Option<i64>,
    /// 31c.A04 present/null distinction preserved.
    pub group_history_sent: Option<bool>,
    /// 31c.A06 optional display_name; bot branch ignores it.
    pub display_name: Option<String>,
}

impl GroupParticipant {
    /// Resolve the APK member-storage scalars using explicit previous state/clock.
    /// Does not reconcile aliases, apply membership, mutate a store or read time.
    pub fn storage_metadata(
        &self,
        previous_join_time_ms: Option<i64>,
        previous_history_state: Option<i32>,
        clock_ms: i64,
    ) -> super::ParticipantStorageMetadata {
        super::participant_storage_metadata(
            self.join_time,
            self.group_history_sent,
            previous_join_time_ms,
            previous_history_state,
            clock_ms,
        )
    }
}

impl std::fmt::Debug for GroupParticipant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupParticipant").finish_non_exhaustive()
    }
}
impl std::fmt::Debug for GroupParticipantMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PhashMatch => f.write_str("GroupParticipantMap::PhashMatch"),
            Self::Members(members) => f
                .debug_struct("GroupParticipantMap::Members")
                .field("count", &members.len())
                .finish_non_exhaustive(),
        }
    }
}

// 3c9.A0E -> optString(__typename).hashCode; ADv.A0D compares only hashes,
// with no String.equals guards. Actual server typename spellings are unknown.
const BOT_MODEL_HASH: u32 = 0xc745_b457;
const USER_MODEL_HASH: u32 = 0x2179_873b;
fn java_hash(text: &str) -> u32 {
    text.encode_utf16().fold(0u32, |v, unit| {
        v.wrapping_mul(31).wrapping_add(u32::from(unit))
    })
}

fn nullable_string(
    value: Option<&super::ordered_json::OrderedJson>,
    name: &'static str,
) -> Result<Option<String>> {
    // LbE.A0E isNull before optString; direct scalar and structured paths
    // retain their distinct AOSP/host-Java spellings and original object order.
    let Some(value) = value.filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    value.platform_string().map(Some).ok_or_else(|| {
        Error::invalid(
            "mex.participant.string_coercion",
            format!("Java string conversion for {name} failed"),
        )
    })
}

fn model_hash(participant: &super::ordered_json::OrderedJson) -> Result<u32> {
    // A0F does not run isNull: explicit JSONObject.NULL spells "null".
    let text = match participant.get("__typename") {
        Some(value) if value.is_null() => Some("null".into()),
        value => nullable_string(value, "__typename")?,
    };
    Ok(java_hash(text.as_deref().unwrap_or_default()))
}

fn primary(participant: &Map<String, Value>, name: &'static str) -> Result<ParticipantAddress> {
    // JSON nonstrings cannot stringify to a supported full/bare UserJid:
    // numbers/bools have no @, structured strings end in JSON delimiters.
    let value = participant
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            Error::invalid(
                "mex.participant.primary",
                format!("missing/nonstring primary address for {name}"),
            )
        })?;
    ParticipantAddress::parse(value).map_err(|_| {
        Error::invalid(
            "mex.participant.primary",
            format!("invalid strict primary UserJid for {name}"),
        )
    })
}

fn alias(
    participant: &Map<String, Value>,
    name: &'static str,
    kind: ParticipantAddressKind,
) -> Result<Option<ParticipantAddress>> {
    match participant.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => {
            let address = ParticipantAddress::parse(value).map_err(|_| {
                Error::invalid(
                    "mex.participant.alias",
                    format!("invalid present {name} alias"),
                )
            })?;
            if address.kind() != kind {
                return Err(Error::invalid(
                    "mex.participant.alias",
                    format!("present {name} alias has the wrong APK JID subtype"),
                ));
            }
            Ok(Some(address))
        }
        _ => Err(Error::invalid(
            "mex.participant.alias",
            format!("invalid nonstring {name} alias"),
        )),
    }
}

pub(super) fn convert(
    participants: &Map<String, Value>,
    ordered: &super::ordered_json::OrderedJson,
) -> Result<GroupParticipantMap> {
    if coercion::boolean(participants.get("participants_phash_match")) {
        return Ok(GroupParticipantMap::PhashMatch);
    }
    let mut members: Vec<GroupParticipant> = Vec::new();
    // Jid.equals compares user, getServer and getType; the parsed address
    // kinds distinguish the eligible subtypes. Hosted canonicalization thus
    // replaces an existing plain key, retaining its original map position.
    let mut positions: HashMap<ParticipantAddress, usize> = HashMap::new();
    let ordered_edges = ordered
        .get("edges")
        .and_then(super::ordered_json::OrderedJson::as_array)
        .unwrap_or_default();
    for (index, edge) in participants
        .get("edges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(index, value)| value.as_object().map(|edge| (index, edge)))
    {
        let Some(participant) = edge.get("participant").and_then(Value::as_object) else {
            continue;
        };
        let ordered_participant = ordered_edges
            .get(index)
            .and_then(|edge| edge.get("participant"))
            .ok_or_else(|| {
                Error::invalid(
                    "mex.participant.string_coercion",
                    "ordered participant source is missing",
                )
            })?;
        let (address, lid, pn, display_name) = match model_hash(ordered_participant)? {
            BOT_MODEL_HASH => {
                // 0hj.A03 catches every strict-parser failure and returns null.
                // Wrong/missing/nonstring bot JID skips; aliases/display ignored.
                let Some(address) = participant
                    .get("jid")
                    .and_then(Value::as_str)
                    .and_then(|v| ParticipantAddress::parse(v).ok())
                    .filter(|a| a.kind() == ParticipantAddressKind::Bot)
                else {
                    continue;
                };
                (address, None, None, None)
            }
            USER_MODEL_HASH => {
                let address = primary(participant, "id")?;
                let lid = alias(participant, "lid", ParticipantAddressKind::Lid)?;
                let display_name =
                    nullable_string(ordered_participant.get("display_name"), "display_name")?;
                let pn = alias(participant, "pn", ParticipantAddressKind::Phone)?;
                (address, lid, pn, display_name)
            }
            _ => continue,
        };
        let member = GroupParticipant {
            primary: address.clone(),
            lid,
            pn,
            display_name,
            role: GroupParticipantRole::from_server_value(edge.get("role").and_then(Value::as_str))
                .member_role(),
            join_time: coercion::join_time(edge.get("join_time")),
            group_history_sent: coercion::nullable_boolean(edge.get("group_history_sent")),
        };
        if let Some(&position) = positions.get(&address) {
            members[position] = member;
        } else {
            positions.insert(address, members.len());
            members.push(member);
        }
    }
    // 32q/31c's filtered interop alias and extra string slots are null in both
    // traced ADv branches. No guessed interop alias or extra-string meaning.
    Ok(GroupParticipantMap::Members(members))
}

#[cfg(test)]
mod tests {
    use super::super::parse_json_response;
    use super::*;
    use crate::binary::Node;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-participant-map.json"
        ))
        .unwrap()
    }
    fn address(value: &ParticipantAddress) -> Value {
        serde_json::json!({"user": value.user(), "server": value.server(), "kind": value.kind()})
    }

    #[test]
    fn hash_dispatch_matches_independent_java_utf16_reference() {
        for case in fixture()["hash_reference"].as_array().unwrap() {
            assert_eq!(
                format!("{:x}", java_hash(case["value"].as_str().unwrap())),
                case["java_unsigned_hex"].as_str().unwrap()
            );
        }
    }

    #[test]
    fn nullable_ordered_member_map_preserves_apk_branches_and_refusals() {
        for case in fixture()["cases"].as_array().unwrap() {
            for child in ["result", "update"] {
                let node = Node::new("iq").children(vec![
                    Node::new(child).bytes(serde_json::to_vec(&case["payload"]).unwrap()),
                ]);
                let response = parse_json_response(&node).unwrap();
                let result = response.group_participant_map();
                if let Some(field) = case["error_field"].as_str() {
                    let error = result.unwrap_err().to_string();
                    assert!(error.contains(field), "{}: {error}", case["name"]);
                    assert!(!error.contains("private-"));
                    if let Some(attribute) = case["error_attribute"].as_str() {
                        assert!(error.contains(attribute), "{}: {error}", case["name"]);
                    }
                    continue;
                }
                let result = result.unwrap();
                assert!(!format!("{result:?}").contains("synthetic-"));
                let output = match result {
                    GroupParticipantMap::PhashMatch => {
                        serde_json::json!({"kind":"phash_match", "members":null})
                    }
                    GroupParticipantMap::Members(members) => {
                        for member in &members {
                            assert_eq!(format!("{member:?}"), "GroupParticipant { .. }");
                        }
                        serde_json::json!({"kind":"members", "members":members.iter().map(|m| serde_json::json!({
                            "primary":address(&m.primary), "lid":m.lid.as_ref().map(address), "pn":m.pn.as_ref().map(address),
                            "role":m.role, "join_time":m.join_time.map(|v| v.to_string()),
                            "group_history_sent":m.group_history_sent, "display_name":m.display_name,
                        })).collect::<Vec<_>>()})
                    }
                };
                assert_eq!(output, case["expected"], "{} / {child}", case["name"]);
                assert!(!format!("{response:?}").contains("synthetic-"));
            }
        }
    }
}
