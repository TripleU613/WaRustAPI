//! Reference-manifest extraction from an APK (docs/design/README.md section 6.1).
//!
//! Computes the APK hash, reads `AndroidManifest.xml`, and fingerprints the
//! first X.509 certificate in the v1 signature block. Output is JSON on
//! stdout; with `--check` the facts are compared against a committed manifest.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
pub struct Extracted {
    pub package: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u64>,
    pub min_sdk: Option<u64>,
    pub target_sdk: Option<u64>,
    pub compile_sdk: Option<u64>,
    pub apk_sha256: String,
    pub apk_size_bytes: u64,
    pub v1_signature_block: Option<String>,
    pub v1_signature_block_sha256: Option<String>,
    pub signing_certificate_sha256: Option<String>,
    pub native_libraries: Vec<String>,
}

#[derive(Deserialize)]
struct Committed {
    reference_client: CommittedClient,
}

#[derive(Deserialize)]
struct CommittedClient {
    package: String,
    version_name: String,
    version_code: u64,
    apk_sha256: String,
    signing_certificate: CommittedCert,
}

#[derive(Deserialize)]
struct CommittedCert {
    sha256: String,
}

/// Parse one DER TLV at `o`; returns (tag, header_len, content_len).
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
pub(crate) fn find_certificate(d: &[u8], o: usize, end: usize, depth: usize) -> Option<&[u8]> {
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

pub fn extract(apk: &Path) -> Result<Extracted, String> {
    let mut f = File::open(apk).map_err(|e| format!("{}: {e}", apk.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut size = 0u64;
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        size += n as u64;
        hasher.update(&buf[..n]);
    }
    let apk_sha256 = hex::encode(hasher.finalize());
    let f = File::open(apk).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| format!("zip: {e}"))?;
    let names = crate::dex::entry_names(&zip)?;
    let mut manifest_bytes = Vec::new();
    zip.by_name("AndroidManifest.xml")
        .map_err(|e| format!("AndroidManifest.xml: {e}"))?
        .read_to_end(&mut manifest_bytes)
        .map_err(|e| e.to_string())?;
    let facts = crate::axml::parse_manifest(&manifest_bytes)?;
    let sig_name = names
        .iter()
        .find(|n| {
            n.starts_with("META-INF/")
                && (n.ends_with(".DSA") || n.ends_with(".RSA") || n.ends_with(".EC"))
        })
        .cloned();
    let (mut sig_hash, mut cert_hash) = (None, None);
    if let Some(name) = &sig_name {
        let mut sig = Vec::new();
        zip.by_name(name)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut sig)
            .map_err(|e| e.to_string())?;
        sig_hash = Some(hex::encode(Sha256::digest(&sig)));
        if let Some(cert) = find_certificate(&sig, 0, sig.len(), 0) {
            let digest = Sha256::digest(cert);
            cert_hash = Some(
                digest
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(":"),
            );
        }
    }
    let mut native_libraries: Vec<String> = names
        .iter()
        .filter(|n| n.starts_with("lib/") && n.ends_with(".so"))
        .cloned()
        .collect();
    native_libraries.sort();
    Ok(Extracted {
        package: facts.package,
        version_name: facts.version_name,
        version_code: facts.version_code,
        min_sdk: facts.min_sdk,
        target_sdk: facts.target_sdk,
        compile_sdk: facts.compile_sdk,
        apk_sha256,
        apk_size_bytes: size,
        v1_signature_block: sig_name,
        v1_signature_block_sha256: sig_hash,
        signing_certificate_sha256: cert_hash,
        native_libraries,
    })
}

pub fn run(apk: &Path, check: Option<&Path>) -> Result<(), String> {
    let e = extract(apk)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&e).map_err(|err| err.to_string())?
    );
    if let Some(path) = check {
        let text =
            std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let c: Committed = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        let mut mismatches = Vec::new();
        if e.package.as_deref() != Some(c.reference_client.package.as_str()) {
            mismatches.push("package");
        }
        if e.version_name.as_deref() != Some(c.reference_client.version_name.as_str()) {
            mismatches.push("version_name");
        }
        if e.version_code != Some(c.reference_client.version_code) {
            mismatches.push("version_code");
        }
        if e.apk_sha256 != c.reference_client.apk_sha256 {
            mismatches.push("apk_sha256");
        }
        if e.signing_certificate_sha256.as_deref()
            != Some(c.reference_client.signing_certificate.sha256.as_str())
        {
            mismatches.push("signing_certificate.sha256");
        }
        if !mismatches.is_empty() {
            return Err(format!("manifest mismatch: {}", mismatches.join(", ")));
        }
        eprintln!("reference-manifest: APK matches {}", path.display());
    }
    Ok(())
}
