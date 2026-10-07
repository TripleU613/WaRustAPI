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
    // zip 0.6 file_names() iterates names_map keys and hides duplicate names.
    // Inspect actual directory entries without decompressing media.
    let mut found = None;
    for index in 0..zip.len() {
        let member = zip.by_index_raw(index).map_err(|e| e.to_string())?;
        if member.name() == asset_path {
            if found.is_some() {
                return Err("MEX asset has duplicate ZIP names".into());
            }
            found = Some(index);
        }
    }
    let index = found.ok_or("MEX asset is missing")?;
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

/// Summary is redacted; explicit group_numbers opts into numeric content only.
/// Raw JSON/group names/server error content never appear in diagnostics.
pub fn inspect_response(
    input: &Path,
    child: &str,
    group_info: bool,
    group_numbers: bool,
    edge_roles: bool,
    participant_scalars: bool,
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
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
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
        let archive = |names: &[&str]| {
            let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
            for name in names {
                writer
                    .start_file(*name, zip::write::FileOptions::default())
                    .unwrap();
                writer.write_all(asset()).unwrap();
            }
            zip::ZipArchive::new(writer.finish().unwrap()).unwrap()
        };
        let mut single = archive(&[name]);
        assert_eq!(read_asset(&mut single, name).unwrap(), asset());
        let mut duplicate = archive(&[name, name]);
        assert_eq!(duplicate.len(), 2);
        assert_eq!(duplicate.file_names().count(), 1);
        assert_eq!(
            read_asset(&mut duplicate, name).unwrap_err(),
            "MEX asset has duplicate ZIP names"
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
