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
    use std::io::Write;
    use zip::CompressionMethod;
    use zip::write::SimpleFileOptions;

    /// Minimal Android binary XML: a UTF-8 string pool and one `<manifest
    /// package="...">` start element, laid out at the offsets
    /// `parse_manifest_package` reads (ResXMLTree_attrExt after a 16-byte node
    /// header, 20-byte attributes, `TYPE_STRING` 0x03 at byte 15).
    fn binary_manifest(package: &str) -> Vec<u8> {
        let strings = ["manifest", "package", package];
        let mut data = Vec::new();
        let mut offsets = Vec::new();
        for value in strings {
            offsets.push(u32::try_from(data.len()).unwrap());
            let len = u8::try_from(value.len()).unwrap();
            data.extend([len, len]);
            data.extend(value.as_bytes());
            data.push(0);
        }
        while data.len() % 4 != 0 {
            data.push(0);
        }
        let header = 28u16;
        let strings_start = u32::from(header) + 4 * offsets.len() as u32;
        let pool_size = strings_start + data.len() as u32;
        let mut pool = Vec::new();
        pool.extend(0x0001u16.to_le_bytes());
        pool.extend(header.to_le_bytes());
        pool.extend(pool_size.to_le_bytes());
        pool.extend((offsets.len() as u32).to_le_bytes());
        pool.extend(0u32.to_le_bytes()); // style count
        pool.extend(0x100u32.to_le_bytes()); // UTF-8 flag
        pool.extend(strings_start.to_le_bytes());
        pool.extend(0u32.to_le_bytes()); // styles start
        for offset in offsets {
            pool.extend(offset.to_le_bytes());
        }
        pool.extend(data);

        let mut element = Vec::new();
        element.extend(0x0102u16.to_le_bytes());
        element.extend(16u16.to_le_bytes());
        element.extend(56u32.to_le_bytes());
        element.extend(1u32.to_le_bytes()); // line
        element.extend(u32::MAX.to_le_bytes()); // comment
        element.extend(u32::MAX.to_le_bytes()); // namespace
        element.extend(0u32.to_le_bytes()); // name: "manifest"
        element.extend(20u16.to_le_bytes()); // attribute start
        element.extend(20u16.to_le_bytes()); // attribute size
        element.extend(1u16.to_le_bytes()); // attribute count
        element.extend([0; 6]); // id, class and style indexes
        element.extend(u32::MAX.to_le_bytes()); // attribute namespace
        element.extend(1u32.to_le_bytes()); // attribute name: "package"
        element.extend(2u32.to_le_bytes()); // raw value
        element.extend(8u16.to_le_bytes());
        element.push(0);
        element.push(0x03); // TYPE_STRING
        element.extend(2u32.to_le_bytes()); // data: string index 2

        let mut xml = Vec::new();
        xml.extend(0x0003u16.to_le_bytes());
        xml.extend(8u16.to_le_bytes());
        xml.extend(((8 + pool.len() + element.len()) as u32).to_le_bytes());
        xml.extend(pool);
        xml.extend(element);
        xml
    }

    /// Smallest DER shape `find_certificate` accepts:
    /// `SEQUENCE { SEQUENCE { [0] { INTEGER 2 }, INTEGER 1 } }`.
    const CERTIFICATE: [u8; 12] = [
        0x30, 0x0a, 0x30, 0x08, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x01, 0x01,
    ];

    /// A v1 signature block with the certificate nested one level down, as in
    /// PKCS#7 SignedData.
    fn signature_block() -> Vec<u8> {
        let mut block = vec![0x30, 3 + CERTIFICATE.len() as u8, 0x02, 0x01, 0x00];
        block.extend(CERTIFICATE);
        block
    }

    fn write_archive(path: &Path, entries: &[(&str, &[u8], CompressionMethod)]) {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes, method) in entries {
            let options = SimpleFileOptions::default().compression_method(*method);
            zip.start_file(*name, options).unwrap();
            zip.write_all(bytes).unwrap();
        }
        std::fs::write(path, zip.finish().unwrap().into_inner()).unwrap();
    }

    #[test]
    fn opens_archive_with_stored_and_deflated_entries() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("test.apk");
        let manifest = binary_manifest("com.whatsapp");
        // Repetitive enough that deflate really compresses it, so the inflater
        // selected for `zip` is exercised rather than a stored copy.
        let dex: Vec<u8> = (0..64 * 1024).map(|i| (i % 7) as u8).collect();
        let signature = signature_block();
        write_archive(
            &path,
            &[
                (
                    "AndroidManifest.xml",
                    &manifest,
                    CompressionMethod::Deflated,
                ),
                ("classes.dex", &dex, CompressionMethod::Deflated),
                (
                    "res/drawable-hdpi-v4/about_logo.png",
                    b"png",
                    CompressionMethod::Stored,
                ),
                ("META-INF/CERT.RSA", &signature, CompressionMethod::Stored),
            ],
        );

        let material = TokenMaterial::open(&path).unwrap();
        assert_eq!(material.package_name, "com.whatsapp");
        assert_eq!(material.classes_dex, dex);
        assert_eq!(material.about_logo, b"png");
        assert_eq!(material.logo_entry, "res/drawable-hdpi-v4/about_logo.png");
        assert_eq!(material.signature_entry, "META-INF/CERT.RSA");
        assert_eq!(material.signing_certificate_der, CERTIFICATE);
    }

    #[test]
    fn rejects_archive_without_classes_dex() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("test.apk");
        let manifest = binary_manifest("com.whatsapp");
        let signature = signature_block();
        write_archive(
            &path,
            &[
                (
                    "AndroidManifest.xml",
                    &manifest,
                    CompressionMethod::Deflated,
                ),
                (
                    "res/drawable-hdpi/about_logo.png",
                    b"png",
                    CompressionMethod::Stored,
                ),
                ("META-INF/CERT.RSA", &signature, CompressionMethod::Stored),
            ],
        );

        // The manifest parses, so the refusal is the missing entry itself.
        let error = TokenMaterial::open(&path).unwrap_err().to_string();
        assert!(error.contains("classes.dex"), "{error}");
    }

    #[test]
    fn read_entry_refuses_empty_and_oversized_entries() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("test.zip");
        write_archive(
            &path,
            &[
                ("empty", b"", CompressionMethod::Stored),
                ("five", b"12345", CompressionMethod::Deflated),
            ],
        );
        let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();

        let empty = read_entry(&mut archive, "empty", 4)
            .unwrap_err()
            .to_string();
        assert!(empty.contains("expected 1..=4 bytes, got 0"), "{empty}");
        let oversized = read_entry(&mut archive, "five", 4).unwrap_err().to_string();
        assert!(
            oversized.contains("expected 1..=4 bytes, got 5"),
            "{oversized}"
        );
        assert_eq!(read_entry(&mut archive, "five", 5).unwrap(), b"12345");
        let missing = read_entry(&mut archive, "absent", 5)
            .unwrap_err()
            .to_string();
        assert!(missing.contains("absent"), "{missing}");
    }
}
