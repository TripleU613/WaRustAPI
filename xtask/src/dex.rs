//! Dalvik executable (dex) string-table extraction for static APK analysis.
//!
//! Only the string table is parsed: header fields `string_ids_size/off`,
//! `string_id_item` offsets and MUTF-8 `string_data_item`s. This is enough to
//! enumerate every string constant in the client, including protocol pattern
//! names, hostnames and URL fragments. Bytecode is not interpreted.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

const DEX_MAGIC: &[u8] = b"dex\n";

fn u32le(d: &[u8], o: usize) -> Result<u32, String> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| "dex: truncated header".to_string())
}

fn uleb128(d: &[u8], mut o: usize) -> Result<(u32, usize), String> {
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
fn mutf8(bytes: &[u8]) -> String {
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
    let names: Vec<String> = zip
        .file_names()
        .filter(|n| n.starts_with("classes") && n.ends_with(".dex"))
        .map(str::to_owned)
        .collect();
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
