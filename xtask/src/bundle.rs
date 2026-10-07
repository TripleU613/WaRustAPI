//! Bounded, reproducible research export. Never exports media or executable binaries.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const APK_SHA256: &str = "b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732";
const MAX_BUNDLE_BYTES: usize = 2 * 1024 * 1024;
// Version-scoped obfuscated names. Never apply these selectors to another APK.
const TARGETS: &[(&str, &str)] = &[
    ("registration-paths", "LX/HpO;"),
    ("registration-requests", "LX/HSP;"),
    ("registration-key-bundle", "LX/Cdl;"),
    (
        "registration-response-bridge",
        "Lcom/whatsapp/registration/core/http/KotlinRegistrationBridge;",
    ),
    ("registration-token", "LX/Hre;"),
    ("registration-token-salt", "LX/HQP;"),
    ("noise-handshake", "LX/1N7;"),
    ("noise-patterns", "LX/1NZ;"),
    ("noise-config", "LX/19p;"),
    ("noise-cipher", "LX/1Nj;"),
    ("noise-record-state", "LX/1QR;"),
    ("certificate-root", "LX/Aw1;"),
    ("socket-sequence", "LX/1BH;"),
    ("socket-factory", "LX/1AM;"),
    ("frame-writer", "LX/1NX;"),
    ("frame-reader", "LX/1NV;"),
    ("integer-encoding", "LX/1Lz;"),
    ("node-writer", "LX/1Qe;"),
    ("record-writer", "LX/1Qc;"),
    ("http-chunk-writer", "LX/H9X;"),
    ("http-chunk-reader", "LX/H9U;"),
];

/// A reviewed mapping is specific to one APK; class renames require re-anchoring.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    apk_sha256: String,
    targets: BTreeMap<String, Target>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    class: String,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    rust_files: Vec<String>,
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_ .".contains(&b))
        && name != "."
        && name != ".."
        && !name.contains(' ')
}

fn read_recipe(path: &Path) -> Result<Recipe, String> {
    let recipe: Recipe = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("recipe: {e}"))?;
    if recipe.apk_sha256.len() != 64
        || hex::decode(&recipe.apk_sha256).is_err()
        || recipe.targets.is_empty()
        || recipe.targets.len() > 256
    {
        return Err("recipe requires SHA-256 and one to 256 targets".into());
    }
    for (topic, target) in &recipe.targets {
        if !safe_name(topic)
            || topic.contains('.')
            || !target.class.starts_with('L')
            || !target.class.ends_with(';')
            || target.method.as_ref().is_some_and(|m| m.is_empty())
            || target.rust_files.iter().any(|p| {
                p.is_empty()
                    || Path::new(p).is_absolute()
                    || Path::new(p)
                        .components()
                        .any(|c| !matches!(c, std::path::Component::Normal(_)))
            })
        {
            return Err(format!("invalid recipe target {topic}"));
        }
    }
    Ok(recipe)
}

#[derive(Default, Serialize)]
struct Totals {
    entries: u64,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
}

fn category(name: &str) -> &'static str {
    if name.starts_with("classes") && name.ends_with(".dex") {
        "dex"
    } else if name.starts_with("lib/") && name.ends_with(".so") {
        "native_libraries"
    } else if name == "AndroidManifest.xml" || name == "resources.arsc" {
        "android_metadata"
    } else if name.starts_with("META-INF/") {
        "signature_and_packaging_metadata"
    } else if name.starts_with("res/") || name.starts_with("assets/") {
        "resources_and_assets_excluded"
    } else {
        "other_excluded"
    }
}

fn inventory(apk: &Path) -> Result<serde_json::Value, String> {
    let file = fs::File::open(apk).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut totals: BTreeMap<&str, Totals> = BTreeMap::new();
    let mut code = BTreeMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let kind = category(entry.name());
        let total = totals.entry(kind).or_default();
        total.entries += 1;
        total.compressed_bytes += entry.compressed_size();
        total.uncompressed_bytes += entry.size();
        if kind == "dex" || entry.name() == "AndroidManifest.xml" {
            let mut digest = Sha256::new();
            let mut buffer = [0u8; 65536];
            loop {
                let n = entry.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                digest.update(&buffer[..n]);
            }
            code.insert(
                entry.name().to_owned(),
                serde_json::json!({
                    "bytes": entry.size(), "sha256": hex::encode(digest.finalize())
                }),
            );
        }
    }
    Ok(serde_json::json!({"categories": totals, "code_and_manifest": code}))
}

fn json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn validate_size(files: &BTreeMap<String, Vec<u8>>) -> Result<usize, String> {
    let total: usize = files.values().map(Vec::len).sum();
    if total > MAX_BUNDLE_BYTES {
        return Err(format!(
            "bundle is {total} bytes; limit is {MAX_BUNDLE_BYTES}"
        ));
    }
    Ok(total)
}

pub fn run(
    apk: &Path,
    output: &Path,
    check: bool,
    recipe_path: Option<&Path>,
) -> Result<(), String> {
    if !check && output.exists() {
        return Err("output already exists; choose a new directory or use --check".into());
    }
    let recipe = recipe_path.map(read_recipe).transpose()?;
    let manifest = crate::manifest::extract(apk)?;
    let expected_hash = recipe
        .as_ref()
        .map_or(APK_SHA256, |r| r.apk_sha256.as_str());
    if manifest.apk_sha256 != expected_hash {
        return Err(
            "APK hash differs from the selected recipe (default 2.26.37.70); refusing version-specific selectors"
                .into(),
        );
    }
    let mut files = BTreeMap::new();
    files.insert("manifest.json".into(), json(&manifest)?);
    files.insert("inventory.json".into(), json(&inventory(apk)?)?);
    let tokens = crate::dex::apk_tokens(apk)?;
    files.insert(
        "tokens.json".into(),
        json(&serde_json::json!({
            "evidence": "client-static, comparison only; not installed into active profile",
            "location": tokens.location, "single": tokens.single, "double": tokens.double
        }))?,
    );
    let mut index = BTreeMap::new();
    let defaults: BTreeMap<String, Target> = TARGETS
        .iter()
        .map(|&(topic, class)| {
            (
                topic.into(),
                Target {
                    class: class.into(),
                    method: None,
                    rust_files: Vec::new(),
                },
            )
        })
        .collect();
    let targets = recipe.as_ref().map_or(&defaults, |r| &r.targets);
    for (topic, target) in targets {
        let class = target.class.as_str();
        eprintln!("exporting {topic} ({class})");
        let methods = crate::xref::apk_smali(apk, class, target.method.as_deref())?;
        if methods.is_empty() || methods.iter().any(|m| m.class_name != class) {
            return Err(format!("missing or ambiguous class {class}"));
        }
        let mut text = format!(
            "# APK-derived structural disassembly: {class}\n# APK SHA-256: {expected_hash}\n# Offsets are method-relative bytes. Prototypes are DEX shorties, not full signatures.\n# This is partial annotated bytecode, not compilable smali or recovered Java.\n"
        );
        for method in &methods {
            writeln!(
                text,
                "\n.method {}({}) flags=0x{:x} registers={}",
                method.method_name, method.method_proto, method.access_flags, method.registers
            )
            .unwrap();
            for insn in &method.insns {
                writeln!(
                    text,
                    "  {:04x}..{:04x} op={:02x} {} {}",
                    insn.byte_off,
                    insn.byte_off + insn.width * 2,
                    insn.op,
                    insn.mnemonic,
                    insn.annotation
                )
                .unwrap();
            }
            text.push_str(".end method\n");
        }
        // Empty annotations must not introduce trailing whitespace.
        let text = text
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let name = format!("{topic}.txt");
        let mut entry =
            serde_json::json!({ "class": class, "methods": methods.len(), "file": name });
        if recipe.is_some() {
            entry["method_filter"] = serde_json::json!(target.method);
            entry["rust_files"] = serde_json::json!(target.rust_files);
        }
        index.insert(topic, entry);
        files.insert(name, text.into_bytes());
        validate_size(&files)?;
    }
    files.insert("targets.json".into(), json(&index)?);
    let checksums: BTreeMap<_, _> = files.iter().map(|(name, bytes)| {
        (name.clone(), serde_json::json!({"bytes": bytes.len(), "sha256": hex::encode(Sha256::digest(bytes))}))
    }).collect();
    files.insert("checksums.json".into(), json(&checksums)?);
    let total = validate_size(&files)?;
    if check {
        for (name, expected) in &files {
            let actual = fs::read(output.join(name)).map_err(|e| format!("{name}: {e}"))?;
            if actual != *expected {
                return Err(format!("bundle differs: {name}"));
            }
        }
        for entry in fs::read_dir(output).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !files.contains_key(&entry.file_name().to_string_lossy().into_owned()) {
                return Err(format!(
                    "unexpected bundle entry: {}",
                    entry.path().display()
                ));
            }
        }
    } else {
        fs::create_dir_all(output).map_err(|e| e.to_string())?;
        for (name, bytes) in &files {
            fs::write(output.join(name), bytes).map_err(|e| e.to_string())?;
        }
    }
    println!(
        "{} {} files, {total} bytes (limit {MAX_BUNDLE_BYTES})",
        if check { "Verified" } else { "Exported" },
        files.len()
    );
    Ok(())
}

fn load_bundle(path: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if fs::metadata(path.join("checksums.json"))
        .map_err(|e| e.to_string())?
        .len()
        > MAX_BUNDLE_BYTES as u64
    {
        return Err("bundle checksum index exceeds local bound".into());
    }
    let checksums: BTreeMap<String, serde_json::Value> =
        serde_json::from_slice(&fs::read(path.join("checksums.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let mut files = BTreeMap::new();
    for (name, checksum) in checksums {
        if !safe_name(&name) {
            return Err("unsafe bundle filename".into());
        }
        let size = fs::metadata(path.join(&name))
            .map_err(|e| e.to_string())?
            .len();
        if size > MAX_BUNDLE_BYTES as u64 {
            return Err("bundle file exceeds local bound".into());
        }
        let bytes = fs::read(path.join(&name)).map_err(|e| e.to_string())?;
        if checksum["bytes"].as_u64() != Some(bytes.len() as u64)
            || checksum["sha256"].as_str() != Some(hex::encode(Sha256::digest(&bytes)).as_str())
        {
            return Err(format!("bundle checksum mismatch: {name}"));
        }
        files.insert(name, bytes);
        validate_size(&files)?;
    }
    Ok(files)
}

fn bundle_json(files: &BTreeMap<String, Vec<u8>>, name: &str) -> Result<serde_json::Value, String> {
    serde_json::from_slice(
        files
            .get(name)
            .ok_or_else(|| format!("missing bundle file {name}"))?,
    )
    .map_err(|e| e.to_string())
}

fn topic_body(
    files: &BTreeMap<String, Vec<u8>>,
    entry: &serde_json::Value,
) -> Result<String, String> {
    let file = entry["file"].as_str().ok_or("topic file missing")?;
    let bytes = files
        .get(file)
        .ok_or("topic file absent from verified checksums")?;
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    // Every release changes the provenance header; retain all other differences,
    // including class names, offsets, instructions and unresolved opcodes.
    Ok(text
        .lines()
        .filter(|l| !l.starts_with("# APK SHA-256:"))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn comparison(old: &Path, new: &Path) -> Result<serde_json::Value, String> {
    let old_files = load_bundle(old)?;
    let new_files = load_bundle(new)?;
    let old_targets = bundle_json(&old_files, "targets.json")?;
    let new_targets = bundle_json(&new_files, "targets.json")?;
    let old_targets = old_targets
        .as_object()
        .ok_or("old targets is not an object")?;
    let new_targets = new_targets
        .as_object()
        .ok_or("new targets is not an object")?;
    let topics: std::collections::BTreeSet<_> =
        old_targets.keys().chain(new_targets.keys()).collect();
    let mut changes = Vec::new();
    for topic in topics {
        let before = old_targets.get(topic);
        let after = new_targets.get(topic);
        let status = match (before, after) {
            (Some(a), Some(b)) => {
                if topic_body(&old_files, a)? == topic_body(&new_files, b)? {
                    "unchanged"
                } else {
                    "changed"
                }
            }
            (None, Some(_)) => "added",
            (Some(_), None) => "removed",
            (None, None) => unreachable!(),
        };
        let mut rust_files = std::collections::BTreeSet::new();
        for target in [before, after].into_iter().flatten() {
            if let Some(paths) = target["rust_files"].as_array() {
                for path in paths {
                    if let Some(path) = path.as_str() {
                        rust_files.insert(path);
                    }
                }
            }
        }
        changes.push(serde_json::json!({"topic":topic,"status":status,
            "old_file":before.map(|v| old.join(v["file"].as_str().unwrap_or_default())),
            "new_file":after.map(|v| new.join(v["file"].as_str().unwrap_or_default())),
            "old_class":before.map(|v| &v["class"]),"new_class":after.map(|v| &v["class"]),
            "rust_files":rust_files}));
    }
    let topic_files: std::collections::BTreeSet<_> = old_targets
        .values()
        .chain(new_targets.values())
        .filter_map(|v| v["file"].as_str())
        .collect();
    let artifact_names: std::collections::BTreeSet<_> = old_files
        .keys()
        .chain(new_files.keys())
        .filter(|name| !topic_files.contains(name.as_str()))
        .collect();
    let artifacts: Vec<_> = artifact_names
        .into_iter()
        .map(|name| {
            let status = match (old_files.get(name), new_files.get(name)) {
                (Some(a), Some(b)) => {
                    if a == b {
                        "unchanged"
                    } else {
                        "changed"
                    }
                }
                (None, Some(_)) => "added",
                (Some(_), None) => "removed",
                (None, None) => unreachable!(),
            };
            serde_json::json!({"file":name,"status":status,
            "old_file":old_files.contains_key(name).then(|| old.join(name)),
            "new_file":new_files.contains_key(name).then(|| new.join(name))})
        })
        .collect();
    Ok(serde_json::json!({
        "old_apk":bundle_json(&old_files,"manifest.json")?["apk_sha256"],
        "new_apk":bundle_json(&new_files,"manifest.json")?["apk_sha256"],
        "interpretation":"Structural topic differences, not automatic proof of protocol behavior changes. Unmapped APK code/native behavior is outside this scoped comparison.",
        "topics":changes, "artifacts":artifacts
    }))
}

pub fn compare(old: &Path, new: &Path) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(&comparison(old, new)?).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_bundle(path: &Path, hash: &str, topics: &[(&str, &str)]) {
        fs::create_dir(path).unwrap();
        let mut files = BTreeMap::new();
        let mut targets = BTreeMap::new();
        for (topic, body) in topics {
            let name = format!("{topic}.txt");
            files.insert(
                name.clone(),
                format!("# APK SHA-256: {hash}\n{body}\n").into_bytes(),
            );
            targets.insert(topic, serde_json::json!({"file":name,"class":"LX/Test;","rust_files":["crates/whatsapp-api/src/transport/groups.rs"]}));
        }
        files.insert("targets.json".into(), json(&targets).unwrap());
        files.insert(
            "manifest.json".into(),
            json(&serde_json::json!({"apk_sha256":hash})).unwrap(),
        );
        let sums: BTreeMap<_,_> = files.iter().map(|(name,bytes)| (name.clone(),
            serde_json::json!({"bytes":bytes.len(),"sha256":hex::encode(Sha256::digest(bytes))}))).collect();
        files.insert("checksums.json".into(), json(&sums).unwrap());
        for (name, bytes) in files {
            fs::write(path.join(name), bytes).unwrap();
        }
    }

    #[test]
    fn cached_comparison_separates_provenance_and_code_and_verifies_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("old");
        let new = dir.path().join("new");
        fixture_bundle(
            &old,
            "aaa",
            &[
                ("same", "const-string subject"),
                ("changed", "const/4 1"),
                ("removed", "old"),
            ],
        );
        fixture_bundle(
            &new,
            "bbb",
            &[
                ("same", "const-string subject"),
                ("changed", "const/4 2"),
                ("added", "new"),
            ],
        );
        let report = comparison(&old, &new).unwrap();
        let rows = report["topics"].as_array().unwrap();
        assert_eq!(
            rows.iter()
                .map(|v| (v["topic"].as_str().unwrap(), v["status"].as_str().unwrap()))
                .collect::<Vec<_>>(),
            vec![
                ("added", "added"),
                ("changed", "changed"),
                ("removed", "removed"),
                ("same", "unchanged")
            ]
        );
        assert!(
            report["artifacts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["file"] == "manifest.json" && v["status"] == "changed")
        );
        assert_eq!(
            rows[1]["rust_files"][0],
            "crates/whatsapp-api/src/transport/groups.rs"
        );
        fs::write(new.join("changed.txt"), "tampered").unwrap();
        assert!(
            comparison(&old, &new)
                .unwrap_err()
                .contains("checksum mismatch")
        );
    }

    #[test]
    fn recipes_and_cached_filenames_refuse_unsafe_or_unpinned_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("recipe.json");
        for value in [
            serde_json::json!({"apk_sha256":"bad","targets":{"subject":{"class":"LX/Test;"}}}),
            serde_json::json!({"apk_sha256":"0".repeat(64),"targets":{"../escape":{"class":"LX/Test;"}}}),
            serde_json::json!({"apk_sha256":"0".repeat(64),"targets":{"subject":{"class":"LX/Test;","rust_files":["../escape"]}}}),
            serde_json::json!({"apk_sha256":"0".repeat(64),"targets":{"subject":{"class":"LX/Test;","invented":true}}}),
        ] {
            fs::write(&path, json(&value).unwrap()).unwrap();
            assert!(read_recipe(&path).is_err());
        }
        fs::write(&path,json(&serde_json::json!({"apk_sha256":"0".repeat(64),"targets":{"subject":{"class":"LX/Test;","method":"A09","rust_files":["crates/group.rs"]}}})).unwrap()).unwrap();
        assert!(read_recipe(&path).is_ok());
        let cache = dir.path().join("bad-cache");
        fs::create_dir(&cache).unwrap();
        fs::write(
            cache.join("checksums.json"),
            br#"{"../escape":{"bytes":1,"sha256":"bad"}}"#,
        )
        .unwrap();
        assert!(load_bundle(&cache).unwrap_err().contains("unsafe"));
    }

    #[test]
    fn assets_never_classify_as_code() {
        for path in [
            "res/drawable/icon.png",
            "assets/font.ttf",
            "assets/classes.dex",
            "res/raw/sound.ogg",
        ] {
            assert_eq!(category(path), "resources_and_assets_excluded");
        }
        assert_eq!(category("classes12.dex"), "dex");
        assert_eq!(category("lib/arm64-v8a/libs.so"), "native_libraries");
    }

    #[test]
    fn bundle_budget_is_enforced_before_writing() {
        let mut files = BTreeMap::new();
        files.insert("one.txt".into(), vec![0; MAX_BUNDLE_BYTES]);
        assert_eq!(validate_size(&files).unwrap(), MAX_BUNDLE_BYTES);
        files.insert("two.txt".into(), vec![0]);
        assert!(validate_size(&files).is_err());
    }
}
