//! Token material extraction from a ZIP archive (APK or equivalent).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::model::phone::PhoneNumber;
use crate::{Error, Result};

use super::token::{ComparisonTokenMaterial, derive_comparison_registration_token};
use zeroize::Zeroizing;

/// Maximum accepted file size for Android manifest.
const MAX_MANIFEST_BYTES: usize = 8 << 20;
/// Maximum accepted file size for signatures.
const MAX_SIGNATURE_BYTES: usize = 8 << 20;

/// APK-extracted token inputs suitable for derivation.
#[derive(Debug)]
pub struct TokenMaterial {
    /// Android application package name.
    pub package_name: String,
    /// Exact bytes of the selected hdpi `about_logo.png` resource.
    pub about_logo: Vec<u8>,
    /// First Android signing certificate in DER form.
    pub signing_certificate_der: Vec<u8>,
    /// Exact uncompressed bytes of the primary `classes.dex` entry.
    pub classes_dex: Vec<u8>,
    /// ZIP entry name of the logo resource.
    pub logo_entry: String,
    /// ZIP entry name of the signature block.
    pub signature_entry: String,
}

impl TokenMaterial {
    /// Open and extract token material from a ZIP archive (APK or material archive).
    /// Returns an error if required entries are missing or exceed size bounds.
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|error| {
            Error::invalid("token_material", format!("{}: {error}", path.display()))
        })?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|error| Error::invalid("token_material", format!("zip: {error}")))?;

        let manifest = read_entry(&mut archive, "AndroidManifest.xml", MAX_MANIFEST_BYTES)?;
        let package_name = parse_manifest_package(&manifest)?;

        let file_names = archive
            .file_names()
            .map(|name| name.map(|name| name.into_owned()))
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|error| {
                Error::invalid("token_material", format!("zip entry name: {error}"))
            })?;

        let logo_candidates = [
            "res/drawable-hdpi/about_logo.png",
            "res/drawable-hdpi-v4/about_logo.png",
            "res/drawable-xxhdpi-v4/about_logo.png",
        ];

        let logo_entry = logo_candidates
            .iter()
            .find(|name| file_names.contains(&name.to_string()))
            .ok_or_else(|| {
                Error::invalid(
                    "token_material",
                    "APK has none of the token helper's about_logo.png candidates",
                )
            })?
            .to_string();

        let about_logo = read_entry(
            &mut archive,
            &logo_entry,
            super::token::MAX_TOKEN_LOGO_BYTES,
        )?;

        let classes_dex = read_entry(
            &mut archive,
            "classes.dex",
            super::token::MAX_TOKEN_DEX_BYTES,
        )?;

        let signature_entry = file_names
            .iter()
            .find(|name| {
                name.starts_with("META-INF/")
                    && (name.ends_with(".DSA") || name.ends_with(".RSA") || name.ends_with(".EC"))
            })
            .cloned()
            .ok_or_else(|| {
                Error::invalid(
                    "token_material",
                    "APK has no v1 signature block from which to extract a certificate",
                )
            })?;

        let signature = read_entry(&mut archive, &signature_entry, MAX_SIGNATURE_BYTES)?;
        let signing_certificate_der = find_certificate(&signature, 0, signature.len(), 0)
            .ok_or_else(|| {
                Error::invalid(
                    "token_material",
                    "v1 signature block has no recognizable X.509 certificate",
                )
            })?
            .to_vec();

        Ok(Self {
            package_name,
            about_logo,
            signing_certificate_der,
            classes_dex,
            logo_entry,
            signature_entry,
        })
    }

    /// Borrow this material as a reference for token derivation.
    pub fn as_ref(&self) -> ComparisonTokenMaterial<'_> {
        ComparisonTokenMaterial {
            package_name: &self.package_name,
            about_logo: &self.about_logo,
            signing_certificate_der: &self.signing_certificate_der,
            classes_dex: &self.classes_dex,
        }
    }

    /// Derive the registration token for the given phone number.
    pub fn derive(&self, phone: &PhoneNumber) -> Result<Zeroizing<String>> {
        derive_comparison_registration_token(phone, self.as_ref())
    }
}

fn read_entry(archive: &mut zip::ZipArchive<File>, name: &str, maximum: usize) -> Result<Vec<u8>> {
    let entry = archive
        .by_name(name)
        .map_err(|error| Error::invalid("token_material", format!("{name}: {error}")))?;
    let declared = usize::try_from(entry.size())
        .map_err(|_| Error::invalid("token_material", format!("{name}: size overflow")))?;
    if declared == 0 || declared > maximum {
        return Err(Error::invalid(
            "token_material",
            format!("{name}: expected 1..={maximum} bytes, got {declared}"),
        ));
    }
    let mut bytes = Vec::with_capacity(declared);
    entry
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| Error::invalid("token_material", format!("{name}: {error}")))?;
    if bytes.len() > maximum {
        return Err(Error::invalid(
            "token_material",
            format!("{name}: exceeds {maximum} bytes"),
        ));
    }
    Ok(bytes)
}

/// Parse DER TLV (Tag-Length-Value) at position `o`.
/// Returns (tag, header_len, content_len).
fn tlv(d: &[u8], o: usize) -> Option<(u8, usize, usize)> {
    let tag = *d.get(o)?;
    let first = *d.get(o + 1)?;
    if first & 0x80 == 0 {
        return Some((tag, 2, first as usize));
    }
    let n = (first & 0x7F) as usize;
    if n == 0 || n > 4 {
        return None;
    }
    let mut len = 0usize;
    for i in 0..n {
        len = (len << 8) | *d.get(o + 2 + i)? as usize;
    }
    Some((tag, 2 + n, len))
}

/// Depth-first search for the first X.509 v3 Certificate:
/// `SEQUENCE { SEQUENCE { [0] EXPLICIT version, INTEGER serial, ... }, ... }`.
fn find_certificate(d: &[u8], o: usize, end: usize, depth: usize) -> Option<&[u8]> {
    if depth > 16 {
        return None;
    }
    let mut p = o;
    while p < end {
        let (tag, hl, cl) = tlv(d, p)?;
        let content = p + hl;
        let next = content.checked_add(cl)?;
        if next > end {
            return None;
        }
        let constructed = tag & 0x20 != 0;
        if tag == 0x30
            && let Some((t1, h1, c1)) = tlv(d, content)
            && t1 == 0x30
            && let Some((t2, _, _)) = tlv(d, content + h1)
            && t2 == 0xA0
            && content + h1 + c1 <= next
            && let Some((t3, _, _)) = tlv(
                d,
                content + h1 + tlv(d, content + h1)?.1 + tlv(d, content + h1)?.2,
            )
            && t3 == 0x02
        {
            return Some(&d[p..next]);
        }
        if constructed && let Some(found) = find_certificate(d, content, next, depth + 1) {
            return Some(found);
        }
        p = next;
    }
    None
}

/// Minimal Android binary XML reader for manifest package extraction.
struct ManifestFacts {
    package: Option<String>,
}

/// Parse an `AndroidManifest.xml` blob and extract the package name.
fn parse_manifest_package(d: &[u8]) -> Result<String> {
    if d.len() < 8 {
        return Err(Error::invalid(
            "token_material",
            "AndroidManifest.xml: too short",
        ));
    }

    const CHUNK_STRING_POOL: u16 = 0x0001;
    const CHUNK_START_ELEMENT: u16 = 0x0102;
    const TYPE_STRING: u8 = 0x03;

    let mut strings: Vec<String> = Vec::new();
    let mut facts = ManifestFacts { package: None };
    let mut off = 8usize;

    while off + 8 <= d.len() {
        let chunk_type = u16le(d, off)?;
        let size = u32le(d, off + 4)? as usize;
        if size < 8 || off + size > d.len() {
            return Err(Error::invalid(
                "token_material",
                "AndroidManifest.xml: bad chunk size",
            ));
        }
        match chunk_type {
            CHUNK_STRING_POOL => strings = read_string_pool(d, off)?,
            CHUNK_START_ELEMENT => {
                let name_idx = u32le(d, off + 20)? as usize;
                let attr_start = u16le(d, off + 24)? as usize;
                let attr_count = u16le(d, off + 28)? as usize;
                let name = strings.get(name_idx).cloned().unwrap_or_default();
                let mut attrs: std::collections::BTreeMap<String, (u8, u32)> =
                    std::collections::BTreeMap::new();
                for i in 0..attr_count {
                    let a = off + 16 + attr_start + i * 20;
                    let aname = u32le(d, a + 4)? as usize;
                    let typ = *d.get(a + 15).ok_or_else(|| {
                        Error::invalid("token_material", "AndroidManifest.xml: truncated")
                    })?;
                    let val = u32le(d, a + 16)?;
                    if let Some(n) = strings.get(aname) {
                        attrs.insert(n.clone(), (typ, val));
                    }
                }
                if name == "manifest"
                    && let Some((TYPE_STRING, idx)) = attrs.get("package")
                {
                    facts.package = strings.get(*idx as usize).cloned();
                }
            }
            _ => {}
        }
        off += size;
    }

    facts
        .package
        .ok_or_else(|| Error::invalid("token_material", "AndroidManifest.xml has no package name"))
}

fn u16le(d: &[u8], o: usize) -> Result<u16> {
    d.get(o..o + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| Error::invalid("token_material", "AndroidManifest.xml: truncated"))
}

fn u32le(d: &[u8], o: usize) -> Result<u32> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| Error::invalid("token_material", "AndroidManifest.xml: truncated"))
}

fn read_string_pool(d: &[u8], o: usize) -> Result<Vec<String>> {
    let header_size = u16le(d, o + 2)? as usize;
    let count = u32le(d, o + 8)? as usize;
    let flags = u32le(d, o + 16)?;
    let strings_start = u32le(d, o + 20)? as usize;
    let utf8 = flags & 0x100 != 0;

    if count > 1 << 20 {
        return Err(Error::invalid(
            "token_material",
            "AndroidManifest.xml: absurd string count",
        ));
    }

    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let idx = u32le(d, o + header_size + 4 * i)? as usize;
        let mut p = o + strings_start + idx;
        if utf8 {
            let mut l = *d
                .get(p)
                .ok_or_else(|| Error::invalid("token_material", "AndroidManifest.xml: truncated"))?
                as usize;
            p += 1;
            if l & 0x80 != 0 {
                p += 1;
            }
            l = *d
                .get(p)
                .ok_or_else(|| Error::invalid("token_material", "AndroidManifest.xml: truncated"))?
                as usize;
            p += 1;
            if l & 0x80 != 0 {
                l = ((l & 0x7F) << 8)
                    | *d.get(p).ok_or_else(|| {
                        Error::invalid("token_material", "AndroidManifest.xml: truncated")
                    })? as usize;
                p += 1;
            }
            let bytes = d.get(p..p + l).ok_or_else(|| {
                Error::invalid("token_material", "AndroidManifest.xml: truncated")
            })?;
            out.push(String::from_utf8_lossy(bytes).into_owned());
        } else {
            let mut l = u16le(d, p)? as usize;
            p += 2;
            if l & 0x8000 != 0 {
                l = ((l & 0x7FFF) << 16) | u16le(d, p)? as usize;
                p += 2;
            }
            let bytes = d.get(p..p + 2 * l).ok_or_else(|| {
                Error::invalid("token_material", "AndroidManifest.xml: truncated")
            })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_archive_without_classes_dex() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");

        let mut zip_data = Vec::new();
        {
            use std::io::Write;
            use zip::ZipWriter;
            let writer = std::io::Cursor::new(&mut zip_data);
            let mut zip = ZipWriter::new(writer);
            let options = zip::write::SimpleFileOptions::default();

            zip.start_file("AndroidManifest.xml", options).ok();
            zip.write_all(b"\x00\x00\x00\x00\x00\x00\x00\x00").ok();

            zip.start_file("res/drawable-hdpi/about_logo.png", options)
                .ok();
            zip.write_all(b"png").ok();

            zip.start_file("META-INF/CERT.RSA", options).ok();
            zip.write_all(b"\x30\x03\x30\x01\xA0").ok();
        }

        std::fs::write(&archive_path, &zip_data).ok();
        let result = TokenMaterial::open(&archive_path);
        assert!(
            result.is_err(),
            "archive without classes.dex should fail; got: {:?}",
            result
        );
    }
}
