//! Whole-APK content indexes; cached comparisons never extract or execute files.
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use sha2::{Digest, Sha256};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Content {
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Index {
    format: u32,
    apk_sha256: String,
    apk_bytes: u64,
    #[serde(deserialize_with = "unique_files")]
    files: BTreeMap<String, Content>,
}

fn unique_files<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Content>, D::Error> {
    struct Files;
    impl<'de> serde::de::Visitor<'de> for Files {
        type Value = BTreeMap<String, Content>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("unique APK file names")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut files = BTreeMap::new();
            while let Some((name, content)) = map.next_entry::<String, Content>()? {
                if files.insert(name, content).is_some() {
                    return Err(serde::de::Error::custom("duplicate APK file index key"));
                }
            }
            Ok(files)
        }
    }
    deserializer.deserialize_map(Files)
}

const MAX_APK_BYTES: u64 = 512 * 1024 * 1024;
const MAX_CONTENT_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_INDEX_BYTES: u64 = 32 * 1024 * 1024;
const MAX_FILES: usize = 100_000;

fn hash(reader: &mut impl Read, limit: u64) -> Result<Content, String> {
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0; 65536];
    loop {
        let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        if bytes > limit {
            return Err("artifact content exceeds byte bound".into());
        }
        digest.update(&buffer[..count]);
    }
    Ok(Content {
        bytes,
        sha256: hex::encode(digest.finalize()),
    })
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && !name.contains('\\')
        && !name.chars().any(char::is_control)
        && name.split('/').all(|part| !matches!(part, "" | "." | ".."))
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn index(apk: &Path) -> Result<Index, String> {
    let mut file = File::open(apk).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_APK_BYTES {
        return Err("APK exceeds byte bound".into());
    }
    let artifact = hash(&mut file, MAX_APK_BYTES)?;
    file.rewind().map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("zip: {e}"))?;
    if zip.len() > MAX_FILES {
        return Err("APK exceeds entry bound".into());
    }
    let mut files = BTreeMap::new();
    let mut total = 0;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| format!("zip entry: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = std::str::from_utf8(entry.name_raw())
            .map_err(|_| "entry name is not UTF-8")?
            .to_owned();
        if !valid_name(&name) {
            return Err("unsafe APK file name".into());
        }
        if files.contains_key(&name) {
            return Err(format!("duplicate APK file: {name}"));
        }
        if entry.size() > MAX_APK_BYTES || entry.size() > MAX_CONTENT_BYTES - total {
            return Err("APK expanded content exceeds byte bound".into());
        }
        let content = hash(&mut entry, MAX_APK_BYTES.min(MAX_CONTENT_BYTES - total))?;
        if content.bytes != entry.size() {
            return Err("ZIP entry size differs from content".into());
        }
        total += content.bytes;
        files.insert(name, content);
    }
    Ok(Index {
        format: 1,
        apk_sha256: artifact.sha256,
        apk_bytes: artifact.bytes,
        files,
    })
}

fn read_index(path: &Path) -> Result<Index, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_INDEX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_INDEX_BYTES {
        return Err("index exceeds byte bound".into());
    }
    let index: Index = serde_json::from_slice(&bytes).map_err(|e| format!("index: {e}"))?;
    if index.format != 1
        || !valid_hash(&index.apk_sha256)
        || index.apk_bytes > MAX_APK_BYTES
        || index.files.len() > MAX_FILES
    {
        return Err("invalid APK index metadata".into());
    }
    let mut total = 0u64;
    for (name, content) in &index.files {
        if !valid_name(name) || !valid_hash(&content.sha256) || content.bytes > MAX_APK_BYTES {
            return Err("invalid APK file index".into());
        }
        total += content.bytes;
    }
    if total > MAX_CONTENT_BYTES {
        return Err("index content exceeds byte bound".into());
    }
    Ok(index)
}

fn comparison(old: &Index, new: &Index) -> serde_json::Value {
    let names: std::collections::BTreeSet<_> = old.files.keys().chain(new.files.keys()).collect();
    let mut unchanged = 0;
    let mut files = Vec::new();
    for name in names {
        let before = old.files.get(name);
        let after = new.files.get(name);
        if before == after {
            unchanged += 1;
            continue;
        }
        let status = match (before, after) {
            (None, _) => "added",
            (_, None) => "removed",
            _ => "changed",
        };
        files.push(serde_json::json!({"path":name, "status":status, "old":before, "new":after}));
    }
    serde_json::json!({
        "format":1, "old_apk_sha256":old.apk_sha256, "new_apk_sha256":new.apk_sha256,
        "apk_container_changed":old.apk_sha256 != new.apk_sha256,
        "unchanged_files":unchanged, "files":files,
        "interpretation":"Uncompressed ZIP file content differences. APK signing blocks, ZIP metadata, code semantics and compatibility are not verified. Compare unmodified indexes generated by apk-file-index."
    })
}

pub fn export(apk: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&index(apk)?).map_err(|e| e.to_string())?;
    if json.len() as u64 > MAX_INDEX_BYTES {
        return Err("index exceeds byte bound".into());
    }
    println!("{json}");
    Ok(())
}

pub fn compare(old: &Path, new: &Path) -> Result<(), String> {
    let diff = comparison(&read_index(old)?, &read_index(new)?);
    println!(
        "{}",
        serde_json::to_string_pretty(&diff).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn apk(path: &Path, entries: &[(&str, &[u8])]) {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(
                    *name,
                    zip::write::FileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored),
                )
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        std::fs::write(path, writer.finish().unwrap().into_inner()).unwrap();
    }

    #[test]
    fn content_index_and_cached_diff_distinguish_add_remove_change_and_repacking() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("old.apk");
        let new = dir.path().join("new.apk");
        apk(
            &old,
            &[
                ("classes.dex", b"old"),
                ("assets/same", b"unchanged"),
                ("assets/gone", b"remove"),
            ],
        );
        apk(
            &new,
            &[
                ("assets/same", b"unchanged"),
                ("classes.dex", b"new"),
                ("lib/arm64-v8a/libwa.so", b"add"),
            ],
        );
        let old = index(&old).unwrap();
        let new = index(&new).unwrap();
        assert_eq!(old.files["assets/same"], new.files["assets/same"]);
        let diff = comparison(&old, &new);
        assert_eq!(diff["unchanged_files"], 1);
        assert_eq!(diff["files"][0]["path"], "assets/gone");
        assert_eq!(diff["files"][0]["status"], "removed");
        assert_eq!(diff["files"][1]["status"], "changed");
        assert_eq!(diff["files"][2]["status"], "added");
        let repacked = dir.path().join("repacked.apk");
        apk(
            &repacked,
            &[
                ("assets/gone", b"remove"),
                ("assets/same", b"unchanged"),
                ("classes.dex", b"old"),
            ],
        );
        let diff = comparison(&old, &index(&repacked).unwrap());
        assert_eq!(diff["files"], serde_json::json!([]));
        assert_eq!(diff["apk_container_changed"], true);
    }
    #[test]
    fn archive_index_refuses_duplicate_unsafe_and_corrupt_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.apk");
        apk(&path, &[("classes.dex", b"one"), ("classes.dex", b"two")]);
        assert!(index(&path).unwrap_err().contains("duplicate APK file"));
        for name in [
            "../classes.dex",
            "/classes.dex",
            "assets/./name",
            "assets//name",
            "assets\\name",
        ] {
            apk(&path, &[(name, b"data")]);
            assert!(index(&path).unwrap_err().contains("unsafe APK file name"));
        }
        apk(&path, &[("classes.dex", b"unique-payload")]);
        let mut bytes = std::fs::read(&path).unwrap();
        let position = bytes
            .windows(b"unique-payload".len())
            .position(|b| b == b"unique-payload")
            .unwrap();
        bytes[position] ^= 1;
        std::fs::write(&path, bytes).unwrap();
        assert!(index(&path).is_err());
        assert!(hash(&mut &b"oversize"[..], 3).is_err());
    }

    #[test]
    fn cached_index_refuses_invalid_metadata_paths_and_duplicate_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.json");
        let mut value = serde_json::json!({
            "format":1, "apk_sha256":"a".repeat(64), "apk_bytes":100,
            "files":{"classes.dex":{"bytes":3,"sha256":"b".repeat(64)}}
        });
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(read_index(&path).is_ok());
        for (field, invalid) in [
            ("format", serde_json::json!(2)),
            ("apk_sha256", serde_json::json!("bad")),
            ("apk_bytes", serde_json::json!(MAX_APK_BYTES + 1)),
        ] {
            let mut bad = value.clone();
            bad[field] = invalid;
            std::fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
            assert!(read_index(&path).is_err());
        }
        let content = value["files"]["classes.dex"].clone();
        value["files"] = serde_json::json!({"../classes.dex":content});
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(read_index(&path).is_err());
        let duplicate = format!(
            r#"{{"format":1,"apk_sha256":"{}","apk_bytes":100,"files":{{"classes.dex":{},"classes.dex":{}}}}}"#,
            "a".repeat(64),
            content,
            content
        );
        std::fs::write(&path, duplicate).unwrap();
        assert!(read_index(&path).is_err());
    }
}
