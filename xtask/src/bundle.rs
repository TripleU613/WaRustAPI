//! Bounded, reproducible research export. Never exports media or executable binaries.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
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

pub fn run(apk: &Path, output: &Path, check: bool) -> Result<(), String> {
    if !check && output.exists() {
        return Err("output already exists; choose a new directory or use --check".into());
    }
    let manifest = crate::manifest::extract(apk)?;
    if manifest.apk_sha256 != APK_SHA256 {
        return Err(
            "APK hash differs from the 2.26.37.70 comparison; refusing version-specific selectors"
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
    for &(topic, class) in TARGETS {
        eprintln!("exporting {topic} ({class})");
        let methods = crate::xref::apk_smali(apk, class, None)?;
        if methods.is_empty() || methods.iter().any(|m| m.class_name != class) {
            return Err(format!("missing or ambiguous class {class}"));
        }
        let mut text = format!(
            "# APK-derived structural disassembly: {class}\n# APK SHA-256: {APK_SHA256}\n# Offsets are method-relative bytes. Prototypes are DEX shorties, not full signatures.\n# This is partial annotated bytecode, not compilable smali or recovered Java.\n"
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
        index.insert(
            topic,
            serde_json::json!({
                "class": class, "methods": methods.len(), "file": name
            }),
        );
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

#[cfg(test)]
mod tests {
    use super::*;

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
