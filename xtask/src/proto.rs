//! Rust-only protobuf code generation (no `protoc`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const PROTO_DIR: &str = "proto/whatsmeow";
const OUT_DIR: &str = "crates/whatsapp-api/src/proto";

fn proto_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = root.join(PROTO_DIR);
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_dir() {
            for f in fs::read_dir(entry.path()).map_err(|e| e.to_string())? {
                let f = f.map_err(|e| e.to_string())?;
                if f.path().extension().is_some_and(|e| e == "proto") {
                    files.push(f.path());
                }
            }
        }
    }
    files.sort();
    if files.is_empty() {
        return Err("no .proto files found".into());
    }
    Ok(files)
}

fn generate_into(root: &Path, out: &Path) -> Result<(), String> {
    let files = proto_files(root)?;
    let include = root.join(PROTO_DIR);
    let fds = protox::compile(&files, [&include]).map_err(|e| format!("protox: {e}"))?;
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    // Clear stale outputs.
    for entry in fs::read_dir(out).map_err(|e| e.to_string())? {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|e| e == "rs") {
            fs::remove_file(&p).map_err(|e| e.to_string())?;
        }
    }
    let mut config = prost_build::Config::new();
    config.out_dir(out);
    config.skip_protoc_run();
    config.bytes(["."]);
    config.disable_comments(["."]);
    config
        .compile_fds(fds)
        .map_err(|e| format!("prost-build: {e}"))?;

    // Write mod.rs with provenance and module list.
    let mut modules = BTreeMap::new();
    for entry in fs::read_dir(out).map_err(|e| e.to_string())? {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|e| e == "rs") {
            let stem = p.file_stem().unwrap().to_string_lossy().to_string();
            if stem != "mod" {
                modules.insert(stem, p);
            }
        }
    }
    let revision = fs::read_to_string(root.join(PROTO_DIR).join("REVISION"))
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|_| "unknown".into());
    let mut source_hashes = String::new();
    for f in &files {
        let bytes = fs::read(f).map_err(|e| e.to_string())?;
        let rel = f.strip_prefix(root).unwrap_or(f);
        source_hashes.push_str(&format!(
            "//!   {} sha256={}\n",
            rel.display(),
            hex::encode(Sha256::digest(&bytes))
        ));
    }
    let mut mod_rs = String::new();
    mod_rs.push_str("//! Generated protobuf types. **Do not edit by hand.**\n//!\n");
    mod_rs.push_str(
        "//! Regenerate with `cargo run -p xtask -- proto-gen`; CI runs `proto-check`.\n//!\n",
    );
    mod_rs.push_str(&format!(
        "//! Source schemas: tulir/whatsmeow `proto/` @ {revision} (MPL-2.0), evidence `ReferenceOnly`\n//! for the Android primary profile (ledger `schemas.*`).\n//!\n"
    ));
    mod_rs.push_str(&source_hashes);
    mod_rs.push_str("\n#![allow(missing_docs, dead_code, clippy::all, unused_qualifications, missing_debug_implementations)]\n\n");
    for name in modules.keys() {
        mod_rs.push_str(&format!("pub mod {name};\n"));
    }
    fs::write(out.join("mod.rs"), mod_rs).map_err(|e| e.to_string())?;

    // Format with the pinned toolchain's rustfmt so `cargo fmt --check` and
    // `proto-check` agree byte for byte. rustfmt is a toolchain component,
    // not an external dependency.
    let mut files: Vec<PathBuf> = modules.into_values().collect();
    files.push(out.join("mod.rs"));
    let status = std::process::Command::new("rustfmt")
        .args(["--edition", "2024", "--config", "style_edition=2024"])
        .args(&files)
        .status()
        .map_err(|e| {
            format!("rustfmt not available ({e}); install the toolchain's rustfmt component")
        })?;
    if !status.success() {
        return Err("rustfmt failed on generated code".into());
    }
    Ok(())
}

pub fn generate(root: &Path) -> Result<(), String> {
    let out = root.join(OUT_DIR);
    generate_into(root, &out)?;
    eprintln!("generated into {}", out.display());
    Ok(())
}

fn snapshot(dir: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut m = BTreeMap::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|e| e == "rs") {
            m.insert(
                p.file_name().unwrap().to_string_lossy().to_string(),
                fs::read(&p).map_err(|e| e.to_string())?,
            );
        }
    }
    Ok(m)
}

pub fn check(root: &Path) -> Result<(), String> {
    let committed = snapshot(&root.join(OUT_DIR))?;
    let tmp = std::env::temp_dir().join(format!("whatsapp-api-proto-check-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    generate_into(root, &tmp)?;
    let fresh = snapshot(&tmp)?;
    let _ = fs::remove_dir_all(&tmp);
    if committed == fresh {
        eprintln!("proto-check: committed generated code is up to date");
        return Ok(());
    }
    let mut diff = Vec::new();
    for (k, v) in &fresh {
        match committed.get(k) {
            None => diff.push(format!("missing: {k}")),
            Some(c) if c != v => diff.push(format!("changed: {k}")),
            _ => {}
        }
    }
    for k in committed.keys() {
        if !fresh.contains_key(k) {
            diff.push(format!("stale: {k}"));
        }
    }
    Err(format!(
        "generated code is out of date; run `cargo run -p xtask -- proto-gen`:\n  {}",
        diff.join("\n  ")
    ))
}
