//! Retained-evidence coverage (docs/protocol/apk-coverage.md): which APK
//! releases the committed research bundles cover, which topics have evidence
//! from more than one release, and which Rust files no retained topic cites.
//!
//! Reads only `docs/protocol/apk/*/manifest.json` and `targets.json`; it needs
//! no APK and proves nothing about unexported code. `docs/protocol/rea`
//! envelopes are not counted here.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// The release the default profile is pinned to (docs/design/README.md).
const PINNED_RELEASE: &str = "2.26.36.75";
const REPORT: &str = "docs/protocol/apk-coverage.json";

pub fn run(root: &Path, check: bool) -> Result<(), String> {
    let report = build(root)?;
    let rendered = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n";
    let path = root.join(REPORT);
    if check {
        let saved = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        if saved != rendered {
            return Err(format!(
                "{REPORT} is stale; run `cargo run -p xtask -- apk-coverage` and commit it"
            ));
        }
        println!("{REPORT} is current");
        return Ok(());
    }
    std::fs::write(&path, rendered).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    print_summary(&report);
    Ok(())
}

fn print_summary(report: &Value) {
    for release in report["releases"].as_array().into_iter().flatten() {
        println!(
            "{}: {} bundles, {} topics",
            release["version"].as_str().unwrap_or("?"),
            release["bundles"],
            release["topics"]
        );
    }
    println!(
        "topics: {} total, {} with evidence from more than one release",
        report["topics"]["total"], report["topics"]["multi_release"]
    );
    println!(
        "rust files: {} cited, {} uncited, {} cited but missing",
        report["rust_files"]["cited"]
            .as_object()
            .map_or(0, |m| m.len()),
        report["rust_files"]["uncited"]
            .as_array()
            .map_or(0, Vec::len),
        report["rust_files"]["missing"]
            .as_array()
            .map_or(0, Vec::len)
    );
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON in {}: {e}", path.display()))
}

fn build(root: &Path) -> Result<Value, String> {
    let base = root.join("docs/protocol/apk");
    let mut bundles: Vec<PathBuf> = std::fs::read_dir(&base)
        .map_err(|e| format!("cannot list {}: {e}", base.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.join("manifest.json").is_file() && path.join("targets.json").is_file())
        .collect();
    bundles.sort();

    // release -> (apk hash, bundle count, topic count)
    let mut releases: BTreeMap<String, (String, usize, usize)> = BTreeMap::new();
    // topic name -> releases with evidence for it
    let mut topics: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // cited Rust file -> number of topics citing it
    let mut cited: BTreeMap<String, usize> = BTreeMap::new();
    for bundle in &bundles {
        let manifest = read_json(&bundle.join("manifest.json"))?;
        let version = manifest["version_name"]
            .as_str()
            .ok_or_else(|| format!("{}: manifest has no version_name", bundle.display()))?
            .to_owned();
        let hash = manifest["apk_sha256"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let targets = read_json(&bundle.join("targets.json"))?;
        let targets = targets
            .as_object()
            .ok_or_else(|| format!("{}: targets.json is not an object", bundle.display()))?;
        let entry = releases
            .entry(version.clone())
            .or_insert((hash.clone(), 0, 0));
        if entry.0 != hash {
            return Err(format!(
                "{}: release {version} appears with two APK hashes",
                bundle.display()
            ));
        }
        entry.1 += 1;
        entry.2 += targets.len();
        for (topic, target) in targets {
            topics
                .entry(topic.clone())
                .or_default()
                .insert(version.clone());
            for file in target["rust_files"].as_array().into_iter().flatten() {
                if let Some(file) = file.as_str() {
                    *cited.entry(file.to_owned()).or_default() += 1;
                }
            }
        }
    }

    let protocol_files = rust_files(&root.join("crates/whatsapp-api/src"), root)?;
    let uncited: Vec<&String> = protocol_files
        .iter()
        .filter(|file| !cited.contains_key(*file))
        .collect();
    let missing: Vec<&String> = cited
        .keys()
        .filter(|file| !root.join(file).is_file())
        .collect();
    let multi: Vec<Value> = topics
        .iter()
        .filter(|(_, releases)| releases.len() > 1)
        .map(|(topic, releases)| json!({"topic": topic, "releases": releases}))
        .collect();

    Ok(json!({
        "source": "docs/protocol/apk/*/{manifest,targets}.json only; docs/protocol/rea envelopes are not counted; unexported APK code is outside this report",
        "pinned_release": {
            "version": PINNED_RELEASE,
            "bundles": releases.get(PINNED_RELEASE).map_or(0, |r| r.1),
        },
        "releases": releases.iter().map(|(version, (hash, bundles, topics))| json!({
            "version": version,
            "apk_sha256": hash,
            "bundles": bundles,
            "topics": topics,
        })).collect::<Vec<_>>(),
        "topics": {
            "total": topics.len(),
            "multi_release": multi.len(),
            "multi_release_topics": multi,
        },
        "rust_files": {
            "scope": "crates/whatsapp-api/src/**/*.rs excluding generated proto code",
            "total": protocol_files.len(),
            "cited": cited.iter().filter(|(file, _)| root.join(file).is_file()).collect::<BTreeMap<_, _>>(),
            "uncited": uncited,
            "missing": missing,
        },
    }))
}

/// Library sources, repository-relative, excluding generated protobuf code.
fn rust_files(dir: &Path, root: &Path) -> Result<BTreeSet<String>, String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in
            std::fs::read_dir(&dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))?
        {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "proto") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| format!("{} is outside the repository", path.display()))?;
                out.insert(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic tree: two releases, one shared topic, one stale citation.
    #[test]
    fn report_counts_releases_shared_topics_and_uncited_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let src = root.join("crates/whatsapp-api/src");
        std::fs::create_dir_all(src.join("proto")).unwrap();
        for file in ["a.rs", "b.rs", "proto/gen.rs"] {
            std::fs::write(src.join(file), "").unwrap();
        }
        let bundle = |name: &str, version: &str, hash: &str, targets: Value| {
            let path = root.join("docs/protocol/apk").join(name);
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                path.join("manifest.json"),
                json!({"version_name": version, "apk_sha256": hash}).to_string(),
            )
            .unwrap();
            std::fs::write(path.join("targets.json"), targets.to_string()).unwrap();
        };
        bundle(
            "old",
            "1.0",
            "aa",
            json!({"shared": {"rust_files": ["crates/whatsapp-api/src/a.rs"]}}),
        );
        bundle(
            "new",
            "2.0",
            "bb",
            json!({
                "shared": {"rust_files": ["crates/whatsapp-api/src/a.rs"]},
                "only-new": {"rust_files": ["crates/whatsapp-api/src/gone.rs"]},
            }),
        );
        let report = build(root).unwrap();
        assert_eq!(report["pinned_release"]["bundles"], 0);
        assert_eq!(report["releases"].as_array().unwrap().len(), 2);
        assert_eq!(report["topics"]["total"], 2);
        assert_eq!(report["topics"]["multi_release"], 1);
        assert_eq!(report["rust_files"]["total"], 2, "generated proto excluded");
        assert_eq!(
            report["rust_files"]["cited"]["crates/whatsapp-api/src/a.rs"],
            2
        );
        assert_eq!(
            report["rust_files"]["uncited"],
            json!(["crates/whatsapp-api/src/b.rs"])
        );
        assert_eq!(
            report["rust_files"]["missing"],
            json!(["crates/whatsapp-api/src/gone.rs"])
        );
        // One release with two hashes is refused, not merged.
        bundle("again", "2.0", "cc", json!({}));
        assert!(build(root).unwrap_err().contains("two APK hashes"));
    }
}
