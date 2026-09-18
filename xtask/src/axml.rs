//! Minimal Android binary XML (AXML) reader: enough to extract the `manifest`
//! and `uses-sdk` attributes. Pure Rust; used only by the lab tooling.

use std::collections::BTreeMap;

const CHUNK_STRING_POOL: u16 = 0x0001;
const CHUNK_START_ELEMENT: u16 = 0x0102;
const TYPE_STRING: u8 = 0x03;

/// Extracted manifest facts.
#[derive(Debug, Default)]
pub struct ManifestFacts {
    pub package: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u64>,
    pub min_sdk: Option<u64>,
    pub target_sdk: Option<u64>,
    pub compile_sdk: Option<u64>,
}

fn u16le(d: &[u8], o: usize) -> Result<u16, String> {
    d.get(o..o + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| "axml: truncated".to_string())
}

fn u32le(d: &[u8], o: usize) -> Result<u32, String> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| "axml: truncated".to_string())
}

fn read_string_pool(d: &[u8], o: usize) -> Result<Vec<String>, String> {
    let header_size = u16le(d, o + 2)? as usize;
    let count = u32le(d, o + 8)? as usize;
    let flags = u32le(d, o + 16)?;
    let strings_start = u32le(d, o + 20)? as usize;
    let utf8 = flags & 0x100 != 0;
    if count > 1 << 20 {
        return Err("axml: absurd string count".into());
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let idx = u32le(d, o + header_size + 4 * i)? as usize;
        let mut p = o + strings_start + idx;
        if utf8 {
            let mut l = *d.get(p).ok_or("axml: truncated")? as usize;
            p += 1;
            if l & 0x80 != 0 {
                p += 1;
            }
            l = *d.get(p).ok_or("axml: truncated")? as usize;
            p += 1;
            if l & 0x80 != 0 {
                l = ((l & 0x7F) << 8) | *d.get(p).ok_or("axml: truncated")? as usize;
                p += 1;
            }
            let bytes = d.get(p..p + l).ok_or("axml: truncated")?;
            out.push(String::from_utf8_lossy(bytes).into_owned());
        } else {
            let mut l = u16le(d, p)? as usize;
            p += 2;
            if l & 0x8000 != 0 {
                l = ((l & 0x7FFF) << 16) | u16le(d, p)? as usize;
                p += 2;
            }
            let bytes = d.get(p..p + 2 * l).ok_or("axml: truncated")?;
            let units: Vec<u16> = bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            out.push(String::from_utf16_lossy(&units));
        }
    }
    Ok(out)
}

/// Parse an `AndroidManifest.xml` blob.
pub fn parse_manifest(d: &[u8]) -> Result<ManifestFacts, String> {
    if d.len() < 8 {
        return Err("axml: too short".into());
    }
    let mut strings: Vec<String> = Vec::new();
    let mut facts = ManifestFacts::default();
    let mut off = 8usize;
    while off + 8 <= d.len() {
        let chunk_type = u16le(d, off)?;
        let size = u32le(d, off + 4)? as usize;
        if size < 8 || off + size > d.len() {
            return Err("axml: bad chunk size".into());
        }
        match chunk_type {
            CHUNK_STRING_POOL => strings = read_string_pool(d, off)?,
            CHUNK_START_ELEMENT => {
                let name_idx = u32le(d, off + 20)? as usize;
                let attr_start = u16le(d, off + 24)? as usize;
                let attr_count = u16le(d, off + 28)? as usize;
                let name = strings.get(name_idx).cloned().unwrap_or_default();
                let mut attrs: BTreeMap<String, (u8, u32)> = BTreeMap::new();
                for i in 0..attr_count {
                    let a = off + 16 + attr_start + i * 20;
                    let aname = u32le(d, a + 4)? as usize;
                    let typ = *d.get(a + 15).ok_or("axml: truncated")?;
                    let val = u32le(d, a + 16)?;
                    if let Some(n) = strings.get(aname) {
                        attrs.insert(n.clone(), (typ, val));
                    }
                }
                let text = |attrs: &BTreeMap<String, (u8, u32)>, k: &str| -> Option<String> {
                    attrs.get(k).and_then(|(t, v)| {
                        if *t == TYPE_STRING {
                            strings.get(*v as usize).cloned()
                        } else {
                            Some(v.to_string())
                        }
                    })
                };
                let num = |attrs: &BTreeMap<String, (u8, u32)>, k: &str| -> Option<u64> {
                    attrs.get(k).map(|(_, v)| u64::from(*v))
                };
                match name.as_str() {
                    "manifest" => {
                        facts.package = text(&attrs, "package");
                        facts.version_name = text(&attrs, "versionName");
                        facts.version_code = num(&attrs, "versionCode");
                        facts.compile_sdk = num(&attrs, "compileSdkVersion");
                    }
                    "uses-sdk" => {
                        facts.min_sdk = num(&attrs, "minSdkVersion");
                        facts.target_sdk = num(&attrs, "targetSdkVersion");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        off += size;
    }
    Ok(facts)
}
