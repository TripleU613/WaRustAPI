//! Accounting for retained REA records, separate from bundle-topic coverage.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

#[derive(Default)]
struct Release {
    hash: String,
    records: usize,
    operations: BTreeMap<String, usize>,
    method_bodies: BTreeMap<String, usize>,
    rust_files: BTreeMap<String, usize>,
}

fn hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn evidence_id(value: &str) -> bool {
    value.strip_prefix("ev_").is_some_and(hex_digest)
}

fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| format!("{}: invalid REA JSON: {e}", path.display()))
}

fn record(value: &Value) -> Result<&Value, String> {
    let Some(structured) = value.get("structuredContent") else {
        return Ok(value);
    };
    if value.get("isError").and_then(Value::as_bool) == Some(true) {
        return Err("MCP wrapper contains an error".into());
    }
    let text = value["content"]
        .as_array()
        .ok_or("MCP wrapper has no content array")?
        .iter()
        .filter(|item| item["type"] == "text")
        .map(|item| {
            serde_json::from_str::<Value>(item["text"].as_str().ok_or("invalid MCP text")?)
                .map_err(|_| "MCP text is not JSON")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if text.len() != 1 || text[0] != *structured {
        return Err("MCP text and structuredContent disagree".into());
    }
    Ok(structured)
}

fn paths(base: &Path) -> Result<Vec<PathBuf>, String> {
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut dirs = vec![base.to_owned()];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                return Err(format!(
                    "{}: REA evidence symlinks are not supported",
                    entry.path().display()
                ));
            }
            if kind.is_dir() {
                dirs.push(entry.path());
            } else if entry.path().extension().is_some_and(|ext| ext == "json") {
                out.push(entry.path());
            }
        }
    }
    out.sort();
    Ok(out)
}

fn version(path: &Path, base: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(base).map_err(|e| e.to_string())?;
    let directory = relative
        .components()
        .next()
        .ok_or("missing REA release directory")?;
    let directory = directory
        .as_os_str()
        .to_str()
        .ok_or("non-UTF8 REA directory")?;
    let version = directory.split('-').next().unwrap_or_default();
    if version.split('.').count() != 4
        || version
            .split('.')
            .any(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("REA evidence must live below a four-part APK version directory".into());
    }
    Ok(version.to_owned())
}

// Older retained records do not always carry DEX descriptors. Require the
// recorded class/display-method/overload identity and a real source export,
// without inventing descriptor-level or Java-vs-smali coverage for omissions.
fn method_export_status(normalized: &Value) -> &'static str {
    if normalized["body_status"] == "not_available" {
        return "unavailable";
    }
    let nonempty = |value: &Value| value.as_str().is_some_and(|s| !s.trim().is_empty());
    let identity = nonempty(&normalized["class_name"])
        && nonempty(&normalized["method"]["name"])
        && nonempty(&normalized["method"]["reported_signature"])
        && normalized["method"]["overload_index"].as_u64().is_some();
    let source = nonempty(&normalized["source"]["text"])
        && matches!(
            normalized["representation"].as_str(),
            Some("java" | "smali")
        );
    if normalized["body_status"] == "available"
        && normalized["source"]["status"] == "complete"
        && identity
        && source
    {
        "complete_export"
    } else {
        "partial_or_unknown"
    }
}

/// Version labels come from release directories and are checked against any
/// package declaration and existing bundle manifest. They are not inferred
/// from class names, filesystem-local APK names, or engine versions.
pub(super) fn build(
    root: &Path,
    bundle_hashes: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let base = root.join("docs/protocol/rea");
    let files = paths(&base)?;
    let mut releases: BTreeMap<String, Release> = BTreeMap::new();
    let mut seen: BTreeMap<String, (String, Value)> = BTreeMap::new();
    let mut duplicates = 0;
    let mut excluded = Vec::new();
    let mut ownership = BTreeMap::new();
    for path in files
        .iter()
        .filter(|path| path.file_name().is_some_and(|n| n == "ownership.json"))
    {
        let sidecar = read(path)?;
        let mappings = sidecar["evidence"]
            .as_object()
            .ok_or_else(|| format!("{}: ownership sidecar has no evidence map", path.display()))?;
        for (id, mapping) in mappings {
            if !evidence_id(id) || !hex_digest(mapping["apk_sha256"].as_str().unwrap_or_default()) {
                return Err(format!("{}: invalid ownership identity", path.display()));
            }
            let rust_files = mapping["rust_files"].as_array().ok_or_else(|| {
                format!("{}: ownership rust_files is not an array", path.display())
            })?;
            if rust_files.is_empty() {
                return Err(format!("{}: ownership rust_files is empty", path.display()));
            }
            let mut selected = BTreeSet::new();
            for file in rust_files {
                let file = file.as_str().ok_or("ownership Rust path is not a string")?;
                let file_path = Path::new(file);
                if !file.starts_with("crates/")
                    || !file.ends_with(".rs")
                    || file_path
                        .components()
                        .any(|part| !matches!(part, std::path::Component::Normal(_)))
                    || !root.join(file).is_file()
                {
                    return Err(format!(
                        "{}: invalid or missing ownership Rust path {file}",
                        path.display()
                    ));
                }
                selected.insert(file.to_owned());
            }
            if ownership
                .insert(
                    id.clone(),
                    (mapping["apk_sha256"].as_str().unwrap().to_owned(), selected),
                )
                .is_some()
            {
                return Err(format!(
                    "{}: duplicate ownership declaration for {id}",
                    path.display()
                ));
            }
        }
    }
    for path in files
        .iter()
        .filter(|path| path.file_name().is_none_or(|n| n != "ownership.json"))
    {
        let result = (|| -> Result<(), String> {
            let value = read(path)?;
            let item = record(&value)?;
            // This legacy certificate-parser note is not an REA Evidence record.
            // Keep it visible as excluded instead of silently inventing evidence.
            if item.get("evidence_id").is_none()
                && item["apk_version"].is_string()
                && hex_digest(item["der_sha256"].as_str().unwrap_or_default())
                && item["parsed_certificate"].is_array()
                && item["installed_for_tls"].is_boolean()
            {
                excluded.push(json!({"path": path.strip_prefix(root).unwrap().to_string_lossy(), "reason": "certificate note, not an REA Evidence record"}));
                return Ok(());
            }
            let id = item["evidence_id"]
                .as_str()
                .filter(|id| evidence_id(id))
                .ok_or("missing or invalid evidence_id")?;
            let hash = item["subject"]["digest"]["sha256"]
                .as_str()
                .filter(|hash| hex_digest(hash))
                .ok_or("missing or invalid APK SHA-256")?;
            if item["subject"]["format"] != "apk" || item["authority"] != "shipped-artifact" {
                return Err("expected APK shipped-artifact evidence".into());
            }
            let operation = item["operation"]
                .as_str()
                .filter(|op| !op.is_empty())
                .ok_or("missing operation")?;
            if !item["normalized_result"].is_object() {
                return Err("missing normalized_result object".into());
            }
            let version = version(path, &base)?;
            let declared_version = item["normalized_result"]["version_name"]
                .as_str()
                .or_else(|| item["normalized_result"]["summary"]["version_name"].as_str());
            if declared_version.is_some_and(|declared| declared != version) {
                return Err("APK version declaration disagrees with release directory".into());
            }
            if bundle_hashes
                .get(&version)
                .is_some_and(|expected| expected != hash)
            {
                return Err("REA APK hash disagrees with bundle manifest".into());
            }
            if bundle_hashes
                .iter()
                .any(|(other, expected)| other != &version && expected == hash)
                || releases
                    .iter()
                    .any(|(other, release)| other != &version && release.hash == hash)
            {
                return Err("REA release label disagrees with known APK hash".into());
            }
            let release = releases.entry(version.clone()).or_default();
            if !release.hash.is_empty() && release.hash != hash {
                return Err("REA release appears with two APK hashes".into());
            }
            release.hash = hash.to_owned();
            if let Some((previous_version, previous)) = seen.get(id) {
                if previous_version != &version {
                    return Err("same evidence_id has inconsistent release labels".into());
                }
                if previous != item {
                    return Err("same evidence_id has different retained contents".into());
                }
                duplicates += 1;
                return Ok(());
            }
            seen.insert(id.to_owned(), (version, item.clone()));
            release.records += 1;
            *release.operations.entry(operation.to_owned()).or_default() += 1;
            if operation == "inspect_android_method" {
                let status = method_export_status(&item["normalized_result"]);
                *release.method_bodies.entry(status.to_owned()).or_default() += 1;
            }
            if let Some((expected, rust_files)) = ownership.get(id) {
                if expected != hash {
                    return Err("ownership APK hash disagrees with evidence".into());
                }
                for file in rust_files {
                    *release.rust_files.entry(file.clone()).or_default() += 1;
                }
            }
            Ok(())
        })();
        result.map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if let Some(id) = ownership.keys().find(|id| !seen.contains_key(*id)) {
        return Err(format!("ownership references unretained evidence {id}"));
    }
    Ok(json!({
        "scope": "retained REA APK Evidence records only; distinct from bundle topics and Rust implementation or interoperability coverage",
        "version_source": "release-directory label, checked against package declarations when present and bundle manifests when available",
        "records": seen.len(),
        "duplicate_copies": duplicates,
        "excluded": excluded,
        "ownership_source": "explicit ownership.json sidecars only; prose, directory names and classes do not imply Rust ownership",
        "records_with_explicit_ownership": ownership.len(),
        "releases": releases.iter().map(|(version, r)| json!({
            "version": version, "apk_sha256": r.hash, "records": r.records,
            "operations": r.operations, "method_bodies": r.method_bodies,
            "rust_files": r.rust_files,
        })).collect::<Vec<_>>(),
        "limitations": [
            "An exported method body is not a proven Rust port; fallback and partial source do not establish semantic completeness.",
            "Package summaries, class inventories and reference searches are evidence records, not method or implementation coverage.",
            "Evidence IDs are deduplicated by exact retained record equality; this accounting does not cryptographically reauthenticate REA evidence IDs or execute the APK.",
            "Bundle-topic and REA-record counts can overlap and must not be added into a completion percentage."
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERSION: &str = "2.26.43.72";
    const HASH: &str = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e";
    const ID: &str = "ev_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn method() -> Value {
        json!({
            "evidence_id": ID,
            "subject": {"format": "apk", "digest": {"sha256": HASH}},
            "authority": "shipped-artifact",
            "operation": "inspect_android_method",
            "normalized_result": {
                "class_name": "example.Registration",
                "method": {"name": "waitSeconds", "reported_signature": "waitSeconds(): long", "overload_index": 0},
                "representation": "java",
                "body_status": "available", "source": {"status": "complete", "text": "return 1;"}
            }
        })
    }

    fn save(root: &Path, name: &str, value: &Value) -> PathBuf {
        let path = root
            .join("docs/protocol/rea")
            .join(format!("{VERSION}-test"))
            .join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        path
    }

    fn report(root: &Path) -> Result<Value, String> {
        build(
            root,
            &BTreeMap::from([(VERSION.to_owned(), HASH.to_owned())]),
        )
    }

    #[test]
    fn counts_wrapped_and_direct_evidence_once_and_keeps_inventory_separate() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let item = method();
        save(root, "direct.json", &item);
        save(
            root,
            "wrapped.json",
            &json!({"structuredContent": item, "content": [{"type":"text", "text": item.to_string()}]}),
        );
        let mut summary = method();
        summary["evidence_id"] =
            json!("ev_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        summary["operation"] = json!("inspect_analysis_view");
        summary["normalized_result"] =
            json!({"summary": {"class_count": 106897, "version_name": VERSION}});
        save(root, "summary.json", &summary);
        let actual = report(root).unwrap();
        assert_eq!(actual["records"], 2);
        assert_eq!(actual["duplicate_copies"], 1);
        assert_eq!(
            actual["releases"][0]["operations"]["inspect_analysis_view"],
            1
        );
        assert_eq!(
            actual["releases"][0]["method_bodies"],
            json!({"complete_export": 1})
        );
        assert_eq!(actual["records_with_explicit_ownership"], 0);
    }

    #[test]
    fn refuses_conflicting_duplicates_and_disagreeing_mcp_wrappers() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let item = method();
        save(root, "one.json", &item);
        let mut altered = item.clone();
        altered["normalized_result"]["source"]["text"] = json!("return 2;");
        let path = save(root, "two.json", &altered);
        assert!(
            report(root)
                .unwrap_err()
                .contains("different retained contents")
        );
        std::fs::remove_file(path).unwrap();
        save(
            root,
            "wrapped.json",
            &json!({"structuredContent": item, "content": [{"type":"text", "text": altered.to_string()}]}),
        );
        assert!(report(root).unwrap_err().contains("disagree"));
    }

    #[test]
    fn refuses_wrong_hash_version_and_malformed_records() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut item = method();
        item["subject"]["digest"]["sha256"] = json!("b".repeat(64));
        save(root, "one.json", &item);
        assert!(report(root).unwrap_err().contains("disagrees with bundle"));
        item = method();
        item["normalized_result"]["version_name"] = json!("2.26.37.71");
        save(root, "one.json", &item);
        assert!(report(root).unwrap_err().contains("version declaration"));
        save(
            root,
            "one.json",
            &json!({"operation": "inspect_android_method"}),
        );
        assert!(report(root).unwrap_err().contains("evidence_id"));
    }

    #[test]
    fn requires_explicit_same_artifact_ownership_and_existing_rust_paths() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        save(root, "one.json", &method());
        let source = "crates/whatsapp-api/src/registration/wire.rs";
        let path = root.join(source);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
        let mut ownership =
            json!({"evidence": {ID: {"apk_sha256": HASH, "rust_files": [source,source]}}});
        save(root, "ownership.json", &ownership);
        let actual = report(root).unwrap();
        assert_eq!(actual["records_with_explicit_ownership"], 1);
        assert_eq!(actual["releases"][0]["rust_files"][source], 1);
        ownership["evidence"][ID]["apk_sha256"] = json!("b".repeat(64));
        save(root, "ownership.json", &ownership);
        assert!(report(root).unwrap_err().contains("ownership APK hash"));
        ownership["evidence"][ID]["apk_sha256"] = json!(HASH);
        ownership["evidence"][ID]["rust_files"] = json!(["crates/../outside.rs"]);
        save(root, "ownership.json", &ownership);
        assert!(
            report(root)
                .unwrap_err()
                .contains("invalid or missing ownership")
        );
    }

    #[test]
    fn refuses_relabelled_artifacts_and_dangling_ownership() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let path = save(root, "one.json", &method());
        let wrong = root.join("docs/protocol/rea/2.26.43.73-test/one.json");
        std::fs::create_dir_all(wrong.parent().unwrap()).unwrap();
        std::fs::rename(&path, &wrong).unwrap();
        assert!(report(root).unwrap_err().contains("release label"));
        std::fs::remove_file(wrong).unwrap();
        let source = "crates/whatsapp-api/src/registration/wire.rs";
        let path = root.join(source);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
        save(root, "retained.json", &method());
        let absent = "ev_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        save(
            root,
            "ownership.json",
            &json!({"evidence": {absent: {"apk_sha256": HASH, "rust_files": [source]}}}),
        );
        assert!(report(root).unwrap_err().contains("unretained evidence"));
    }

    #[test]
    fn missing_partial_or_malformed_method_exports_never_count_complete() {
        let mutations = [
            ("missing source", "/source", Value::Null),
            ("missing text", "/source/text", Value::Null),
            ("empty text", "/source/text", json!("")),
            ("whitespace text", "/source/text", json!("  \n")),
            ("non-string text", "/source/text", json!(7)),
            ("partial source", "/source/status", json!("partial")),
            ("missing representation", "/representation", Value::Null),
            ("unknown representation", "/representation", json!("kotlin")),
            ("missing method", "/method", Value::Null),
            ("empty method", "/method/name", json!("")),
            (
                "missing signature",
                "/method/reported_signature",
                Value::Null,
            ),
            ("negative overload", "/method/overload_index", json!(-1)),
            ("missing class", "/class_name", Value::Null),
        ];
        for (name, pointer, replacement) in mutations {
            let dir = tempfile::tempdir().unwrap();
            let mut item = method();
            *item["normalized_result"].pointer_mut(pointer).unwrap() = replacement;
            save(dir.path(), "one.json", &item);
            let actual = report(dir.path()).unwrap();
            assert_eq!(
                actual["releases"][0]["method_bodies"],
                json!({"partial_or_unknown": 1}),
                "{name}"
            );
        }
        // Valid legacy exports lack dex_descriptor; both actual representations
        // remain exportable without claiming that the source is original code.
        let mut item = method();
        item["normalized_result"]["representation"] = json!("smali");
        assert_eq!(
            method_export_status(&item["normalized_result"]),
            "complete_export"
        );
    }

    #[test]
    fn unavailable_bodies_and_legacy_certificate_notes_are_not_code_exports() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut item = method();
        item["normalized_result"]["body_status"] = json!("not_available");
        save(root, "one.json", &item);
        save(
            root,
            "certificate.json",
            &json!({"apk_version":VERSION,"der_sha256":HASH,"parsed_certificate":[],"installed_for_tls":false}),
        );
        let actual = report(root).unwrap();
        assert_eq!(actual["records"], 1);
        assert_eq!(
            actual["releases"][0]["method_bodies"],
            json!({"unavailable":1})
        );
        assert_eq!(actual["excluded"].as_array().unwrap().len(), 1);
    }
}
