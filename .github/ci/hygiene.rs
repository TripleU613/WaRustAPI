//! Fast repository checks using only std and runner-provided git/jq/sha256sum.
use std::{fs, path::Path, process::Command};

fn output(program: &str, args: &[&str]) -> Result<String, String> {
    let result = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(format!(
            "{program}: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    String::from_utf8(result.stdout).map_err(|e| e.to_string())
}

fn check() -> Result<(), String> {
    let paths = output("git", &["ls-files", "-z"])?;
    let mut links = 0;
    for name in paths.split('\0').filter(|p| !p.is_empty()) {
        let path = Path::new(name);
        let metadata = fs::symlink_metadata(path).map_err(|e| format!("{name}: {e}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("unexpected tracked symlink: {name}"));
        }
        if metadata.len() > 2 * 1024 * 1024 {
            return Err(format!("tracked file exceeds 2 MiB: {name}"));
        }
        if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("apk" | "dex" | "odex" | "vdex" | "redb" | "key" | "pem")
        ) || path.file_name().is_some_and(|s| s == ".env")
        {
            return Err(format!("private/raw artifact must not be tracked: {name}"));
        }
        if path.extension().is_some_and(|e| e == "md") {
            let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let mut fenced = false;
            for line in text.lines() {
                if line.trim_start().starts_with("```") {
                    fenced = !fenced;
                    continue;
                }
                if fenced {
                    continue;
                }
                for part in line.split("](").skip(1) {
                    let Some((target, _)) = part.split_once(')') else {
                        continue;
                    };
                    let target = target.trim_matches(['<', '>']);
                    if target.contains("://")
                        || target.starts_with("mailto:")
                        || target.starts_with('#')
                    {
                        continue;
                    }
                    let local = target.split('#').next().unwrap_or("").replace("%20", " ");
                    if !path.parent().unwrap().join(&local).exists() {
                        return Err(format!("{name}: broken local link {target}"));
                    }
                    links += 1;
                }
            }
        }
    }
    let mut bundles = Vec::new();
    for entry in fs::read_dir("docs/protocol/apk").map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() && path.join("checksums.json").is_file() {
            bundles.push(path);
        }
    }
    bundles.sort();
    if bundles.is_empty() {
        return Err("no APK research checksum bundles found".into());
    }
    let mut total = 0;
    for bundle in &bundles {
        total += check_bundle(bundle).map_err(|e| format!("{}: {e}", bundle.display()))?;
    }
    for name in ["LICENSE-MIT", "LICENSE-APACHE"] {
        if fs::read(name).map_err(|e| e.to_string())?
            != fs::read(format!("crates/whatsapp-api/{name}")).map_err(|e| e.to_string())?
        {
            return Err(format!("packaged license differs: {name}"));
        }
    }
    println!(
        "Checked tracked-file budgets, {links} local links, package licenses, and {total} bytes across {} APK research checksum bundles.",
        bundles.len()
    );
    Ok(())
}

fn check_bundle(bundle: &Path) -> Result<u64, String> {
    let checksum_file = bundle.join("checksums.json");
    let entries = output(
        "jq",
        &[
            "-r",
            "to_entries[] | [.key, .value.bytes, .value.sha256] | @tsv",
            checksum_file.to_str().unwrap(),
        ],
    )?;
    let mut expected = std::collections::BTreeSet::from(["checksums.json".to_owned()]);
    for row in entries.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.len() != 3 || fields[0].contains(['/', '\\']) || fields[0].starts_with('.') {
            return Err("invalid checksum entry".into());
        }
        let file = bundle.join(fields[0]);
        if fs::metadata(&file)
            .map_err(|e| e.to_string())?
            .len()
            .to_string()
            != fields[1]
        {
            return Err(format!("size mismatch: {}", fields[0]));
        }
        let hash = output("sha256sum", &[file.to_str().unwrap()])?;
        if hash.split_whitespace().next() != Some(fields[2]) {
            return Err(format!("checksum mismatch: {}", fields[0]));
        }
        expected.insert(fields[0].to_owned());
    }
    let mut total = 0;
    let mut actual = std::collections::BTreeSet::new();
    for entry in fs::read_dir(bundle).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        total += entry.metadata().map_err(|e| e.to_string())?.len();
        actual.insert(entry.file_name().to_string_lossy().into_owned());
    }
    if actual != expected || total > 2 * 1024 * 1024 {
        return Err("APK bundle file list or size budget failed".into());
    }
    Ok(total)
}

fn main() {
    if let Err(e) = check() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn every_selected_bundle_refuses_hash_size_and_file_set_changes() {
        let path = std::env::temp_dir().join(format!(
            "warustapi-hygiene-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        let scratch = Scratch(path);
        let original = r#"{"topic.txt":{"bytes":3,"sha256":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}}"#;
        fs::write(scratch.0.join("checksums.json"), original).unwrap();
        fs::write(scratch.0.join("topic.txt"), "abc").unwrap();
        assert!(check_bundle(&scratch.0).is_ok());
        fs::write(scratch.0.join("topic.txt"), "abd").unwrap();
        assert!(
            check_bundle(&scratch.0)
                .unwrap_err()
                .contains("checksum mismatch")
        );
        fs::write(scratch.0.join("topic.txt"), "abcd").unwrap();
        assert!(
            check_bundle(&scratch.0)
                .unwrap_err()
                .contains("size mismatch")
        );
        fs::write(scratch.0.join("topic.txt"), "abc").unwrap();
        fs::write(scratch.0.join("extra.txt"), "unexpected").unwrap();
        assert!(check_bundle(&scratch.0).unwrap_err().contains("file list"));
    }
}
