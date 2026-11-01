//! Selected persisted-query release data and offline Rust envelope inspection.
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use whatsapp_api::{
    binary::{AttributeValue, Node, NodeContent},
    model::jid::Jid,
    transport::mex::{GroupQueryContext, group_info_request},
};

// Local extraction budgets, not WhatsApp server requirements. No bulk export.
const MAX_ASSET: u64 = 4 * 1024 * 1024;
const MAX_EXPORT: u64 = 128 * 1024;
const MAX_OPERATIONS: usize = 64;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Export {
    format_version: u32,
    apk_version: String,
    apk_sha256: String,
    namespace: String,
    asset_path: String,
    asset_sha256: String,
    operations: BTreeMap<String, Operation>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    operation_name: String,
    operation_name_hash: String,
    operation_text_hash: String,
    client_doc_id: String,
    // Decimal string after exact u64 extraction: no f64/JavaScript precision loss.
    doc_id: String,
    schema: String,
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

fn bounded_read(reader: impl Read, maximum: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > maximum {
        return Err("MEX input exceeds local byte limit".into());
    }
    Ok(bytes)
}

fn read_asset<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    asset_path: &str,
) -> Result<Vec<u8>, String> {
    // zip 9 keys entries by name: a duplicate collapses into one entry, so
    // len()/by_index never see it. Only the raw central-directory count does,
    // and it is archive-wide, so refuse any duplicate rather than guess which
    // copy is the asset.
    if zip.has_duplicate_names() {
        return Err("APK has duplicate ZIP entry names".into());
    }
    let index = zip
        .index_for_name_raw(asset_path.as_bytes())
        .ok_or("MEX asset is missing")?;
    let member = zip.by_index(index).map_err(|e| e.to_string())?;
    if member.size() > MAX_ASSET {
        return Err("MEX asset declared size exceeds local limit".into());
    }
    bounded_read(member, MAX_ASSET)
}

fn select(bytes: &[u8], operations: &[String]) -> Result<BTreeMap<String, Operation>, String> {
    if operations.is_empty()
        || operations.len() > MAX_OPERATIONS
        || operations.iter().any(|v| !identifier(v))
    {
        return Err(
            "select 1..64 exact operation names (ASCII identifiers up to 128 bytes)".into(),
        );
    }
    let data: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("MEX JSON: {e}"))?;
    let data = data.as_object().ok_or("MEX asset must be an object")?;
    let mut selected = BTreeMap::new();
    for name in operations {
        let entry = data
            .get(name)
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("MEX operation not found: {name}"))?;
        let text = |key: &str| -> Result<String, String> {
            let value = entry
                .get(key)
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("MEX operation {name}: missing string {key}"))?;
            if value.is_empty() || value.len() > 256 {
                return Err(format!("MEX operation {name}: invalid {key} length"));
            }
            Ok(value.to_owned())
        };
        let operation_name = text("operation_name")?;
        if operation_name != *name {
            return Err(format!(
                "MEX operation {name}: name disagrees with asset key"
            ));
        }
        // 164.CFb reads doc_id; AFf's client_doc_id is a separate feature branch.
        let doc_id = entry
            .get("doc_id")
            .and_then(serde_json::Value::as_u64)
            .filter(|id| *id != 0)
            .ok_or_else(|| format!("MEX operation {name}: doc_id must be a positive exact u64"))?
            .to_string();
        selected.insert(
            name.clone(),
            Operation {
                operation_name,
                operation_name_hash: text("operation_name_hash")?,
                operation_text_hash: text("operation_text_hash")?,
                client_doc_id: text("client_doc_id")?,
                doc_id,
                schema: text("schema")?,
            },
        );
    }
    Ok(selected)
}

pub fn extract(
    apk: &Path,
    namespace: &str,
    operations: &[String],
    check: Option<&Path>,
) -> Result<(), String> {
    if !identifier(namespace) {
        return Err("MEX namespace must be an ASCII identifier up to 128 bytes".into());
    }
    // Validate the selector before spending time hashing the APK.
    if operations.is_empty()
        || operations.len() > MAX_OPERATIONS
        || operations.iter().any(|v| !identifier(v))
    {
        return Err(
            "select 1..64 exact operation names (ASCII identifiers up to 128 bytes)".into(),
        );
    }
    let asset_path = format!("assets/{namespace}_client_persist_ids.json");
    let mut zip = zip::ZipArchive::new(File::open(apk).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let bytes = read_asset(&mut zip, &asset_path)?;
    let operations = select(&bytes, operations)?;
    let facts = crate::manifest::extract(apk)?;
    let export = Export {
        format_version: 1,
        apk_version: facts.version_name.ok_or("APK version name is missing")?,
        apk_sha256: facts.apk_sha256,
        namespace: namespace.to_owned(),
        asset_path,
        asset_sha256: hex::encode(Sha256::digest(&bytes)),
        operations,
    };
    if let Some(path) = check {
        let expected: Export = serde_json::from_slice(&bounded_read(
            File::open(path).map_err(|e| e.to_string())?,
            MAX_EXPORT,
        )?)
        .map_err(|e| e.to_string())?;
        if export != expected {
            return Err("selected MEX export/provenance differs from fixture".into());
        }
        eprintln!("selected MEX export and provenance match");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

fn node_json(node: &Node) -> serde_json::Value {
    let attributes: Vec<_> = node
        .attributes
        .iter()
        .map(|a| {
            let value = match &a.value {
                AttributeValue::Text(s) => serde_json::json!({"text":s}),
                AttributeValue::Jid(j) => serde_json::json!({"jid":j.to_string()}),
                AttributeValue::Bytes(b) => serde_json::json!({"bytes_hex":hex::encode(b)}),
            };
            serde_json::json!({"name":a.name,"value":value})
        })
        .collect();
    let content = match &node.content {
        NodeContent::Children(children) => {
            serde_json::json!({"children":children.iter().map(node_json).collect::<Vec<_>>()})
        }
        NodeContent::Text(s) => serde_json::json!({"text":s}),
        NodeContent::Jid(j) => serde_json::json!({"jid":j.to_string()}),
        NodeContent::Bytes(b) => serde_json::json!({"bytes_hex":hex::encode(b)}),
        NodeContent::Empty => serde_json::Value::Null,
    };
    serde_json::json!({"tag":node.description,"attributes":attributes,"content":content})
}

pub fn preview(queries: &Path, chat: &str, id: &str, context: &str) -> Result<(), String> {
    let export: Export = serde_json::from_slice(&bounded_read(
        File::open(queries).map_err(|e| e.to_string())?,
        MAX_EXPORT,
    )?)
    .map_err(|e| e.to_string())?;
    if export.format_version != 1 || export.namespace != "whatsapp-android-mex" {
        return Err("unsupported MEX export format or group query namespace".into());
    }
    let operation = export
        .operations
        .get("QueryGroupInfo")
        .ok_or("QueryGroupInfo is absent from export")?;
    if operation.operation_name != "QueryGroupInfo" || operation.schema != "whatsapp_mex" {
        return Err("group operation name/schema disagrees with confirmed request".into());
    }
    let node = group_info_request(
        id,
        &Jid::parse(chat).map_err(|e| e.to_string())?,
        &operation.doc_id,
        GroupQueryContext::parse(context).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "offline_only":true,"apk_version":export.apk_version,"apk_sha256":export.apk_sha256,
            "operation":"QueryGroupInfo","node":node_json(&node)
        }))
        .map_err(|e| e.to_string())?
    );
    Ok(())
}

/// Prints only recognized enum and stored-role output, never the supplied text.
pub fn inspect_role(value: Option<&str>) -> Result<(), String> {
    let role = whatsapp_api::transport::mex::GroupParticipantRole::from_server_value(value);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "offline_only": true, "role": role, "member_role": role.member_role(),
        }))
        .map_err(|_| "cannot encode MEX role inspection")?
    );
    Ok(())
}

/// Strict member-primary validation without private address echo or account access.
pub fn inspect_participant_address(value: &str) -> Result<(), String> {
    let address = whatsapp_api::transport::mex::ParticipantAddress::parse(value)
        .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "offline_only": true, "kind": address.kind(),
        }))
        .map_err(|_| "cannot encode participant address inspection")?
    );
    Ok(())
}

fn participant_hash_output(
    body: &[u8],
    version: &str,
    as_set: bool,
) -> Result<serde_json::Value, String> {
    use whatsapp_api::transport::participant_hash::{
        DeviceParticipantHashVersion, participant_device_hash, participant_hash,
    };
    let raw: Vec<String> =
        serde_json::from_slice(body).map_err(|_| "expected a private JSON array of strings")?;
    if raw.len() > 1024 {
        return Err("participant hash input exceeds local entry limit".into());
    }
    let input_count = raw.len();
    let (hash, hashed_count) = if version == "1" {
        let selected: Vec<String> = if as_set {
            raw.into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect()
        } else {
            raw
        };
        (participant_hash(&selected), selected.len())
    } else {
        let version = match version {
            "2" => DeviceParticipantHashVersion::V2,
            "3" => DeviceParticipantHashVersion::V3,
            _ => return Err("expected participant hash version 1, 2 or 3".into()),
        };
        let mut selected = Vec::with_capacity(raw.len());
        for (index, text) in raw.iter().enumerate() {
            selected.push(
                Jid::parse(text)
                    .map_err(|_| format!("invalid model address at input index {index}"))?,
            );
        }
        if as_set {
            selected = selected
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
        }
        (participant_device_hash(&selected, version), selected.len())
    };
    Ok(
        serde_json::json!({"offline_only":true,"version":version,"as_set":as_set,"input_count":input_count,"hashed_count":hashed_count,"hash":hash}),
    )
}

/// Explicit private hash inspection only; never selects a live account scheme.
pub fn inspect_participant_hash(input: &Path, version: &str, as_set: bool) -> Result<(), String> {
    let body = bounded_read(
        File::open(input).map_err(|_| "cannot open participant hash input")?,
        64 * 1024,
    )?;
    let output = participant_hash_output(&body, version, as_set)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|_| "cannot encode participant hash inspection")?
    );
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParticipantStorageInput {
    join_time: Option<i64>,
    group_history_sent: Option<bool>,
    previous_join_time_ms: Option<i64>,
    previous_history_state: Option<i32>,
    clock_ms: i64,
}

fn participant_storage_input_error(body: &[u8]) -> String {
    // Explain the known field/type without exposing serde's private scalar or
    // unknown key text. This diagnostic pass never changes parsing acceptance.
    let Ok(serde_json::Value::Object(fields)) = serde_json::from_slice(body) else {
        return "invalid storage input: expected a UTF-8 JSON object".into();
    };
    if !fields.contains_key("clock_ms") {
        return "invalid storage input: missing required clock_ms".into();
    }
    const NAMES: [&str; 5] = [
        "clock_ms",
        "join_time",
        "previous_join_time_ms",
        "group_history_sent",
        "previous_history_state",
    ];
    if fields.keys().any(|key| !NAMES.contains(&key.as_str())) {
        return "invalid storage input: unknown fields are not accepted".into();
    }
    for name in NAMES {
        let Some(value) = fields.get(name) else {
            continue;
        };
        if name != "clock_ms" && value.is_null() {
            continue;
        }
        let valid = match name {
            "group_history_sent" => value.is_boolean(),
            "previous_history_state" => value.as_i64().is_some_and(|v| i32::try_from(v).is_ok()),
            _ => value.as_i64().is_some(),
        };
        if !valid {
            let kind = match name {
                "group_history_sent" => "boolean or null",
                "previous_history_state" => "signed i32 or null",
                "clock_ms" => "signed i64",
                _ => "signed i64 or null",
            };
            return format!("invalid storage input: {name} must be {kind}");
        }
    }
    "invalid storage input: duplicate fields or invalid typed input".into()
}

fn participant_storage_output(body: &[u8]) -> Result<serde_json::Value, String> {
    // Do not expose serde's offending scalar/string in private input diagnostics.
    let input: ParticipantStorageInput =
        serde_json::from_slice(body).map_err(|_| participant_storage_input_error(body))?;
    let metadata = whatsapp_api::transport::mex::participant_storage_metadata(
        input.join_time,
        input.group_history_sent,
        input.previous_join_time_ms,
        input.previous_history_state,
        input.clock_ms,
    );
    Ok(serde_json::json!({
        "offline_only": true,
        "join_time_ms": metadata.join_time_ms.to_string(),
        "history_state": metadata.history_state,
    }))
}

/// Explicit private numeric output; does not open an account or read wall time.
pub fn inspect_participant_storage(input: &Path) -> Result<(), String> {
    let body = bounded_read(
        File::open(input).map_err(|_| "cannot open storage metadata file")?,
        16 * 1024,
    )?;
    let output = participant_storage_output(&body)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|_| "cannot encode storage metadata")?
    );
    Ok(())
}

/// Summary is redacted; explicit group_numbers opts into numeric content only.
/// Raw JSON/group names/server error content never appear in diagnostics.
pub fn inspect_response(
    input: &Path,
    child: &str,
    group_info: bool,
    group_numbers: bool,
    edge_roles: bool,
    participant_scalars: bool,
    participant_members: bool,
) -> Result<(), String> {
    use whatsapp_api::transport::mex::{MAX_JSON_RESPONSE_BYTES, parse_json_response};
    if !matches!(child, "result" | "update") {
        return Err("expected result or update payload child".into());
    }
    let body = bounded_read(
        File::open(input).map_err(|_| "cannot open MEX payload file")?,
        MAX_JSON_RESPONSE_BYTES as u64,
    )?;
    let node = Node::new("iq").children(vec![Node::new(child).bytes(body)]);
    let parsed = parse_json_response(&node).map_err(|e| e.to_string())?;
    let mut summary = serde_json::to_value(parsed.summary()).map_err(|e| e.to_string())?;
    if group_info {
        summary["group_info_disposition"] =
            serde_json::to_value(parsed.group_info_disposition()).map_err(|e| e.to_string())?;
    }
    if group_numbers {
        let numbers = parsed.group_numeric_metadata().map_err(|e| e.to_string())?;
        // Decimal strings preserve full signed-long precision in JSON consumers.
        summary["group_numeric_metadata"] = serde_json::json!({
            "creation_scaled": numbers.creation_scaled.to_string(),
            "subject_creation_scaled": numbers.subject_creation_scaled.to_string(),
            "announcement_version": numbers.announcement_version.to_string(),
        });
    }
    if edge_roles {
        summary["group_participant_edge_roles"] = serde_json::to_value(
            parsed
                .group_participant_edge_roles()
                .map_err(|e| e.to_string())?,
        )
        .map_err(|_| "cannot encode MEX edge roles")?;
    }
    if participant_scalars {
        let scalars = parsed
            .group_participant_scalars()
            .map_err(|e| e.to_string())?;
        summary["group_participant_scalars"] = serde_json::json!({
            "phash_match": scalars.phash_match,
            "edges": scalars.edges.iter().map(|edge| serde_json::json!({
                "join_time": edge.join_time.map(|n| n.to_string()),
                "group_history_sent": edge.group_history_sent,
            })).collect::<Vec<_>>(),
        });
    }
    if participant_members {
        use whatsapp_api::transport::mex::{GroupParticipantMap, ParticipantAddress};
        let address = |address: &ParticipantAddress| {
            serde_json::json!({
                "user": address.user(), "server": address.server(), "kind": address.kind(),
            })
        };
        summary["group_participant_map"] = match parsed
            .group_participant_map()
            .map_err(|e| e.to_string())?
        {
            GroupParticipantMap::PhashMatch => {
                serde_json::json!({"kind": "phash_match", "members": null})
            }
            GroupParticipantMap::Members(members) => serde_json::json!({
                "kind": "members", "members": members.iter().map(|member| serde_json::json!({
                    "primary": address(&member.primary),
                    "lid": member.lid.as_ref().map(address), "pn": member.pn.as_ref().map(address),
                    "role": member.role, "join_time": member.join_time.map(|v| v.to_string()),
                    "group_history_sent": member.group_history_sent, "display_name": member.display_name,
                })).collect::<Vec<_>>(),
            }),
        };
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn participant_hash_inspection_matches_reference_modes_and_redacts_refusals() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../crates/whatsapp-api/fixtures/mex/2.26.37.71-participant-hash-reference.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let body = serde_json::to_vec(&case["input"]).unwrap();
            for version in ["1", "2", "3"] {
                for as_set in [false, true] {
                    let output = participant_hash_output(&body, version, as_set).unwrap();
                    let key = format!("v{version}{}", if as_set { "_set" } else { "" });
                    assert_eq!(output["hash"], case["expected"][key]);
                    assert!(output.get("input").is_none());
                }
            }
        }
        for (body, version) in [
            (br#"["private-invalid/address@lid"]"#.as_slice(), "3"),
            (br#"{"private-marker":1}"#.as_slice(), "2"),
            (br#"["private-marker", 1]"#.as_slice(), "1"),
            (br#"[]"#.as_slice(), "4"),
        ] {
            let error = participant_hash_output(body, version, false).unwrap_err();
            assert!(!error.contains("private-"));
        }
    }

    #[test]
    fn storage_inputs_are_typed_and_content_free_on_refusal() {
        let output = participant_storage_output(br#"{"clock_ms":99,"group_history_sent":false,"previous_history_state":4,"previous_join_time_ms":23}"#).unwrap();
        assert_eq!(
            output,
            serde_json::json!({"offline_only":true,"join_time_ms":"23","history_state":4})
        );
        assert!(
            participant_storage_output(br#"{}"#)
                .unwrap_err()
                .contains("missing required clock_ms")
        );
        assert!(
            participant_storage_output(br#"{"clock_ms":99,"join_time":"private-content"}"#)
                .unwrap_err()
                .contains("join_time must be signed i64")
        );
        for raw in [
            r#"{"clock_ms":99,"join_time":"private-content"}"#,
            r#"{"clock_ms":99,"group_history_sent":"private-content"}"#,
            r#"{"clock_ms":99,"previous_history_state":2147483648}"#,
            r#"{"clock_ms":99,"clock_ms":100}"#,
            r#"{"clock_ms":99,"private-content":1}"#,
            r#"{}"#,
        ] {
            let error = participant_storage_output(raw.as_bytes()).unwrap_err();
            assert!(error.starts_with("invalid storage input:"));
            assert!(!error.contains("private-content"));
        }
    }

    use super::*;
    use std::io::{Cursor, Write};

    fn asset() -> &'static [u8] {
        // Independently selected APK 37.71 literal numbers, including >2^53.
        br#"{"QueryGroupInfo":{"operation_name":"QueryGroupInfo","operation_name_hash":"3736295641","operation_text_hash":"12111573297758520297","client_doc_id":"373629564112111573297758520297","doc_id":37759635623635554,"schema":"whatsapp_mex"}}"#
    }

    #[test]
    fn exact_server_id_not_lossy_float_or_client_id() {
        let selected =
            select(asset(), &["QueryGroupInfo".into(), "QueryGroupInfo".into()]).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected["QueryGroupInfo"].doc_id, "37759635623635554");
        assert_eq!(
            selected["QueryGroupInfo"].client_doc_id,
            "373629564112111573297758520297"
        );
    }

    #[test]
    fn actual_zip_entries_refuse_hidden_duplicate_names() {
        let name = "assets/whatsapp-android-mex_client_persist_ids.json";
        let bytes = |names: &[&str]| {
            let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
            for name in names {
                writer
                    .start_file(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(asset()).unwrap();
            }
            writer.finish().unwrap().into_inner()
        };
        let archive = |names: &[&str]| zip::ZipArchive::new(Cursor::new(bytes(names))).unwrap();
        let mut single = archive(&[name]);
        assert_eq!(read_asset(&mut single, name).unwrap(), asset());

        let twin = name.replace(".json", ".jsoX");
        let mut raw = bytes(&[name, &twin]);
        assert_eq!(crate::dex::rename_entry(&mut raw, &twin, name), 2);
        let mut duplicate = zip::ZipArchive::new(Cursor::new(raw)).unwrap();
        assert_eq!(duplicate.len(), 1, "zip 9 collapses the duplicate");
        assert!(duplicate.has_duplicate_names());
        assert_eq!(
            read_asset(&mut duplicate, name).unwrap_err(),
            "APK has duplicate ZIP entry names"
        );
        assert_eq!(
            read_asset(&mut archive(&["other.json"]), name).unwrap_err(),
            "MEX asset is missing"
        );
    }

    #[test]
    fn malformed_or_unselected_inputs_refuse() {
        for names in [
            vec![],
            vec!["../QueryGroupInfo".into()],
            vec!["Missing".into()],
            vec!["QueryGroupInfo".into(); 65],
        ] {
            assert!(select(asset(), &names).is_err());
        }
        for replacement in [
            "0",
            "-1",
            "3.7759635623635554e16",
            "18446744073709551616",
            "\"37759635623635554\"",
        ] {
            let data = String::from_utf8(asset().to_vec())
                .unwrap()
                .replace("37759635623635554", replacement);
            assert!(select(data.as_bytes(), &["QueryGroupInfo".into()]).is_err());
        }
        let data = String::from_utf8(asset().to_vec()).unwrap().replace(
            "\"operation_name\":\"QueryGroupInfo\"",
            "\"operation_name\":\"Other\"",
        );
        assert!(select(data.as_bytes(), &["QueryGroupInfo".into()]).is_err());
        assert!(bounded_read(&b"12345"[..], 4).is_err());
    }
}
