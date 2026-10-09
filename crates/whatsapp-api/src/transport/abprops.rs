//! APK 37.71 Hgb.A00 / IWy.C8f case 1 AB-property read.
//! Saved replies are evidence only: MobileConfig precedence and application
//! are unfinished. This does not set the companion removal gate.
use super::actions;
use crate::{
    Error, Result,
    binary::Node,
    model::jid::Jid,
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};

const MARKER: &[u8] = b"abprops-initial-query";
// 0bq.<clinit> 0050..0064 and 07t.A0J 1ed2..1ee6, APK 2.26.37.71.
const REMOVAL_PROPERTY: i32 = 35661;
// 37.71 07t.A0L 1594..15a6 / 0q2.A02: Boolean counterpart suppression.
const TOKEN_COUNTERPART_PROPERTY: i32 = 15491;
// 37.71 07t.A0X 055a..056c / HxT.A00: Boolean pending-job namespace.
const TOKEN_PENDING_NAMESPACE_PROPERTY: i32 = 14303;

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
    let mut token_counterpart: Option<&str> = None;
    let mut token_pending_namespace: Option<&str> = None;
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
                REMOVAL_PROPERTY => removal = Some(value),
                TOKEN_COUNTERPART_PROPERTY => token_counterpart = Some(value),
                TOKEN_PENDING_NAMESPACE_PROPERTY => token_pending_namespace = Some(value),
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

/// Inspect saved correlated evidence without queueing or applying feature values.
pub fn inspect(store: &ProtocolStore) -> Result<Option<AbPropsInspection>> {
    store.read(|tx| {
        let Some(id) = tx.get_json::<String>(Table::Checkpoints, MARKER, 1)? else {
            return Ok(None);
        };
        let Some((outcome, reply)) = actions::view_in(tx, &id)? else {
            return Err(Error::invalid("abprops", "saved query action is missing"));
        };
        if outcome.kind != "abprops_query" {
            return Err(Error::invalid(
                "abprops",
                "saved marker references another action kind",
            ));
        }
        let mut view = AbPropsInspection {
            action_id: id.clone(),
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
        };
        let Some(reply) = reply else {
            return Ok(Some(view));
        };
        if reply.description != "iq" || text(&reply, "id") != Some(id.as_str()) {
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
        Ok(Some(view))
    })
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
                        ]),
                ),
                &mut evidence,
            )
            .unwrap();
            assert_eq!(evidence.token_counterpart_value, Some(true));
            assert_eq!(evidence.token_pending_namespace_value, Some(false));
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
