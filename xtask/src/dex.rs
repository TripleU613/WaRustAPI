//! Dalvik executable (dex) string-table extraction and structural parsing for
//! static APK analysis.
//!
//! The public helpers (`uleb128`, `mutf8`, `dex_string`) are re-used by
//! `xref.rs` which performs full structural analysis of class/method/code items.

use std::collections::BTreeSet;
use std::io::{Read, Seek};
use std::path::Path;

pub const DEX_MAGIC: &[u8] = b"dex\n";

/// Every entry name in archive order. zip 9 decodes names fallibly (UTF-8,
/// else CP437); an undecodable name is an error, never a silently skipped entry.
pub fn entry_names<R: Read + Seek>(zip: &zip::ZipArchive<R>) -> Result<Vec<String>, String> {
    zip.file_names()
        .map(|name| {
            name.map(|name| name.into_owned())
                .map_err(|e| format!("zip entry name: {e}"))
        })
        .collect()
}

/// The `classes*.dex` entry names in archive order.
pub fn dex_entry_names<R: Read + Seek>(zip: &zip::ZipArchive<R>) -> Result<Vec<String>, String> {
    let mut names = entry_names(zip)?;
    names.retain(|name| name.starts_with("classes") && name.ends_with(".dex"));
    Ok(names)
}

/// Test helper: the zip 9 writer refuses duplicate names, so tests write a
/// same-length twin and rename it in place in both the local and central
/// headers (names are not covered by the CRC). Returns the rename count.
#[cfg(test)]
pub fn rename_entry(raw: &mut [u8], from: &str, to: &str) -> usize {
    assert_eq!(from.len(), to.len());
    let mut renamed = 0;
    for i in 0..=raw.len().saturating_sub(from.len()) {
        if raw[i..].starts_with(from.as_bytes()) {
            raw[i..i + to.len()].copy_from_slice(to.as_bytes());
            renamed += 1;
        }
    }
    renamed
}

fn u32le(d: &[u8], o: usize) -> Result<u32, String> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| "dex: truncated header".to_string())
}

pub fn uleb128(d: &[u8], mut o: usize) -> Result<(u32, usize), String> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let b = *d.get(o).ok_or("dex: truncated uleb128")?;
        o += 1;
        result |= u32::from(b & 0x7F) << shift;
        if b & 0x80 == 0 {
            return Ok((result, o));
        }
        shift += 7;
        if shift > 28 {
            return Err("dex: uleb128 too long".into());
        }
    }
}

/// Decode MUTF-8 (no embedded NUL; surrogate pairs encoded separately).
pub fn mutf8(bytes: &[u8]) -> String {
    let mut units: Vec<u16> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b & 0x80 == 0 {
            units.push(u16::from(b));
            i += 1;
        } else if b & 0xE0 == 0xC0 && i + 1 < bytes.len() {
            units.push((u16::from(b & 0x1F) << 6) | u16::from(bytes[i + 1] & 0x3F));
            i += 2;
        } else if b & 0xF0 == 0xE0 && i + 2 < bytes.len() {
            units.push(
                (u16::from(b & 0x0F) << 12)
                    | (u16::from(bytes[i + 1] & 0x3F) << 6)
                    | u16::from(bytes[i + 2] & 0x3F),
            );
            i += 3;
        } else {
            units.push(0xFFFD);
            i += 1;
        }
    }
    String::from_utf16_lossy(&units)
}

/// Extract all strings from one dex blob.
pub fn strings(d: &[u8]) -> Result<Vec<String>, String> {
    if d.len() < 0x70 || &d[..4] != DEX_MAGIC {
        return Err("not a dex file".into());
    }
    let count = u32le(d, 0x38)? as usize;
    let off = u32le(d, 0x3C)? as usize;
    if count > 4_000_000 {
        return Err("dex: absurd string count".into());
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let item = u32le(d, off + 4 * i)? as usize;
        let (len_units, data) = uleb128(d, item)?;
        // MUTF-8 bytes end at NUL; bound the scan by a generous multiple of
        // the declared UTF-16 length.
        let max = data + (len_units as usize) * 3 + 1;
        let end = d[data..d.len().min(max)]
            .iter()
            .position(|&b| b == 0)
            .map(|p| data + p)
            .ok_or("dex: unterminated string")?;
        out.push(mutf8(&d[data..end]));
    }
    Ok(out)
}

/// Extract the union of string tables from every `classes*.dex` in an APK.
pub fn apk_strings(apk: &Path) -> Result<BTreeSet<String>, String> {
    let f = std::fs::File::open(apk).map_err(|e| format!("{}: {e}", apk.display()))?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| format!("zip: {e}"))?;
    let names = dex_entry_names(&zip)?;
    if names.is_empty() {
        return Err("no classes*.dex in APK".into());
    }
    let mut all = BTreeSet::new();
    for name in names {
        let mut buf = Vec::new();
        zip.by_name(&name)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        for s in strings(&buf)? {
            all.insert(s);
        }
    }
    Ok(all)
}

/// Print strings, optionally filtered by a case-sensitive substring list
/// (any match). Output is one string per line, escaped for control chars.
pub fn run(apk: &Path, contains: &[String]) -> Result<(), String> {
    let all = apk_strings(apk)?;
    let mut n = 0usize;
    for s in &all {
        if contains.is_empty() || contains.iter().any(|c| s.contains(c.as_str())) {
            println!("{}", s.escape_debug());
            n += 1;
        }
    }
    eprintln!("{n} strings ({} total)", all.len());
    Ok(())
}

/// A dictionary recovered from checked initializer array assignments.
pub struct RecoveredDictionary {
    /// Single-byte page (index 0 is the empty token).
    pub single: Vec<String>,
    /// Four double-byte pages of 256 entries each.
    pub double: Vec<Vec<String>>,
    /// DEX file, defining class and anchor instruction offset.
    pub location: String,
}

pub fn apk_tokens(apk: &Path) -> Result<RecoveredDictionary, String> {
    let f = std::fs::File::open(apk).map_err(|e| format!("{}: {e}", apk.display()))?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| format!("zip: {e}"))?;
    let mut names = dex_entry_names(&zip)?;
    names.sort();
    if names.is_empty() {
        return Err("no classes*.dex in APK".into());
    }
    let mut found = None;
    for name in names {
        let mut bytes = Vec::new();
        zip.by_name(&name)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if let Some(dictionary) =
            crate::xref::token_arrays_one(&name, &bytes).map_err(|e| format!("{name}: {e}"))?
        {
            if found.is_some() {
                return Err("multiple token dictionary initializers in APK".into());
            }
            found = Some(dictionary);
        }
    }
    found.ok_or_else(|| "no checked token dictionary initializer found".into())
}

#[derive(serde::Deserialize)]
struct TokenFixture {
    single_byte: Vec<String>,
    double_byte: Vec<Vec<String>>,
}

/// `apk-tokens`: print the recovered dictionary as JSON; with `--check`,
/// compare against a committed fixture and fail on any difference.
pub fn run_tokens(apk: &Path, check: Option<&Path>) -> Result<(), String> {
    let RecoveredDictionary {
        single,
        double,
        location,
    } = apk_tokens(apk)?;
    eprintln!(
        "recovered {} single-byte and {}x256 double-byte tokens from {location}",
        single.len() - 1,
        double.len()
    );
    if let Some(path) = check {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let fx: TokenFixture = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if fx.single_byte != single {
            let first = fx
                .single_byte
                .iter()
                .zip(&single)
                .position(|(a, b)| a != b)
                .map_or_else(|| "length".to_string(), |i| i.to_string());
            return Err(format!(
                "single-byte page differs (first difference at {first})"
            ));
        }
        if fx.double_byte.len() != double.len() {
            return Err(format!(
                "double-byte page count differs: fixture {} vs APK {}",
                fx.double_byte.len(),
                double.len()
            ));
        }
        for (p, (a, b)) in fx.double_byte.iter().zip(&double).enumerate() {
            if a != b {
                let first = a.iter().zip(b).position(|(x, y)| x != y).unwrap_or(0);
                return Err(format!("double-byte page {p} differs at index {first}"));
            }
        }
        eprintln!("apk-tokens: dictionary matches {}", path.display());
        return Ok(());
    }
    let out = serde_json::json!({
        "single_byte": single,
        "double_byte": double,
        "recovered_from": location,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?
    );
    Ok(())
}
