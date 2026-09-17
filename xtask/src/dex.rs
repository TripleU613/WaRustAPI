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

/// Recover the binary-node token dictionary from the client.
///
/// The dictionary is initialised by a long, uninterrupted run of
/// `const-string` instructions (opcode `0x1a`, format `1a AA BBBB`, or the
/// jumbo form `0x1b`) in one static initialiser. We scan every dex file for
/// runs of plausible tokens, anchor on the stream-control token
/// `xmlstreamstart` (the only place such a literal appears in a const-string
/// run), and stop at the first repeated string, because a dictionary has no
/// duplicates. The first 235 strings are the single-byte page (index 0 is the
/// empty token) and each following block of 256 is a double-byte page. This
/// heuristic decodes no other bytecode; it is validated by `--check` against a
/// committed fixture.
/// A dictionary recovered from bytecode.
pub struct RecoveredDictionary {
    /// Single-byte page (index 0 is the empty token).
    pub single: Vec<String>,
    /// Double-byte pages of 256 entries each.
    pub double: Vec<Vec<String>>,
    /// Where the run was found (`<dex file> @ <offset>`).
    pub location: String,
}

pub fn apk_tokens(apk: &Path) -> Result<RecoveredDictionary, String> {
    let f = std::fs::File::open(apk).map_err(|e| format!("{}: {e}", apk.display()))?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| format!("zip: {e}"))?;
    let names: Vec<String> = zip
        .file_names()
        .filter(|n| n.starts_with("classes") && n.ends_with(".dex"))
        .map(str::to_owned)
        .collect();
    let mut best: Vec<String> = Vec::new();
    let mut best_where = String::new();
    for name in names {
        let mut d = Vec::new();
        zip.by_name(&name)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut d)
            .map_err(|e| e.to_string())?;
        let strs = strings(&d)?;
        let mut run: Vec<(usize, u32)> = Vec::new();
        let mut i = 0usize;
        let mut last = 0usize;
        let flush =
            |run: &mut Vec<(usize, u32)>, best: &mut Vec<String>, best_where: &mut String| {
                let names: Vec<&str> = run
                    .iter()
                    .map(|&(_, s)| strs[s as usize].as_str())
                    .collect();
                if let Some(start) = names.iter().position(|s| *s == "xmlstreamstart") {
                    let mut seen = std::collections::HashSet::new();
                    let mut candidate: Vec<String> = Vec::new();
                    for s in &names[start..] {
                        if !seen.insert(*s) {
                            break;
                        }
                        candidate.push((*s).to_owned());
                    }
                    if candidate.len() > best.len() {
                        *best = candidate;
                        *best_where = format!("{name} @ 0x{:x}", run[start].0);
                    }
                }
                run.clear();
            };
        while i + 3 < d.len() {
            let (sidx, width) = match d[i] {
                0x1a => (u32::from(u16::from_le_bytes([d[i + 2], d[i + 3]])), 4usize),
                0x1b if i + 5 < d.len() => (
                    u32::from_le_bytes([d[i + 2], d[i + 3], d[i + 4], d[i + 5]]),
                    6usize,
                ),
                _ => {
                    i += 1;
                    continue;
                }
            };
            let Some(s) = strs.get(sidx as usize) else {
                i += 1;
                continue;
            };
            // Dictionary tokens are short printable ASCII without whitespace.
            let plausible =
                !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| (0x21..0x7f).contains(&b));
            if !plausible {
                i += 1;
                continue;
            }
            if !run.is_empty() && i - last > 64 {
                flush(&mut run, &mut best, &mut best_where);
            }
            run.push((i, sidx));
            last = i;
            i += width;
        }
        flush(&mut run, &mut best, &mut best_where);
    }
    if best.len() < 235 {
        return Err(format!(
            "no plausible token run found (longest {} strings)",
            best.len()
        ));
    }
    let mut single = vec![String::new()];
    single.extend(best[..235].iter().cloned());
    let mut double = Vec::new();
    let mut rest = &best[235..];
    while rest.len() >= 256 {
        double.push(rest[..256].to_vec());
        rest = &rest[256..];
    }
    Ok(RecoveredDictionary {
        single,
        double,
        location: best_where,
    })
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
