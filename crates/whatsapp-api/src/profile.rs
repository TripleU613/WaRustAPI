//! Validated protocol profile (README sections 5 and 6).
//!
//! A profile bundles every protocol-dependent fact the library needs, each
//! carrying an [`EvidenceLabel`]. Operations that need a fact whose label is
//! `Unknown` (or whose value is absent) fail with
//! [`Error::ProfileIncomplete`] naming the missing ledger entries.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::evidence::{EvidenceLabel, Evidenced};

/// Embedded copy of the machine-readable reference manifest.
pub const REFERENCE_MANIFEST_JSON: &str = include_str!("../fixtures/reference-manifest.json");

/// Facts about the pinned reference Android client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceClient {
    /// Android package name.
    pub package: String,
    /// Version name as declared in the manifest.
    pub version_name: String,
    /// Version code as declared in the manifest.
    pub version_code: u64,
    /// SHA-256 of the APK (hex).
    pub apk_sha256: String,
    /// SHA-256 fingerprint of the signing certificate (colon-separated hex).
    pub signing_certificate_sha256: String,
}

/// Local product limits. These are *our* bounds, not observed wire limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    /// Maximum accepted frame body length in bytes.
    pub max_frame_len: u32,
    /// Maximum decompressed node payload in bytes.
    pub max_decompressed_len: u32,
    /// Maximum node nesting depth.
    pub max_node_depth: u16,
    /// Maximum attributes per node.
    pub max_attributes: u16,
    /// Maximum children per node.
    pub max_children: u32,
    /// Maximum string/bytes length inside a node.
    pub max_string_len: u32,
    /// Maximum text message body length in bytes.
    pub max_text_len: u32,
    /// Maximum queued outbox entries.
    pub max_outbox_entries: u32,
    /// Maximum retained raw payload for unsupported messages.
    pub max_unsupported_raw_len: u32,
    /// Maximum outstanding correlated requests.
    pub max_pending_requests: u32,
    /// Maximum events returned per batch.
    pub max_event_batch: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_frame_len: 1 << 20,
            max_decompressed_len: 4 << 20,
            max_node_depth: 32,
            max_attributes: 64,
            max_children: 4096,
            max_string_len: 1 << 20,
            max_text_len: 65_536,
            max_outbox_entries: 10_000,
            max_unsupported_raw_len: 64 * 1024,
            max_pending_requests: 256,
            max_event_batch: 1000,
        }
    }
}

/// Framing facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FramingProfile {
    /// Number of big-endian length-prefix bytes.
    pub length_prefix_bytes: Evidenced<Option<u8>>,
    /// Maximum frame length implied by the prefix width / observed limit.
    pub max_frame_len: Evidenced<Option<u32>>,
    /// Connection header sent once before the first frame, if any.
    pub connection_header: Evidenced<Option<Vec<u8>>>,
}

/// Noise facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoiseProfile {
    /// Noise protocol name (for example `Noise_XX_25519_AESGCM_SHA256`).
    pub pattern: Evidenced<Option<String>>,
    /// Prologue bytes.
    pub prologue: Evidenced<Option<Vec<u8>>>,
    /// Trust root public key for the server certificate chain.
    pub trust_root: Evidenced<Option<[u8; 32]>>,
    /// Expected issuer serial of the intermediate certificate.
    pub issuer_serial: Evidenced<Option<u32>>,
}

/// Token dictionary reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenDictionaryRef {
    /// Dictionary version byte.
    pub version: u8,
    /// SHA-256 (hex) of the fixture file.
    pub sha256: String,
    /// Evidence label of the dictionary for this profile.
    pub label: EvidenceLabel,
}

/// Network endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoints {
    /// Chat/socket endpoint (host:port or URL).
    pub chat: Evidenced<Option<String>>,
    /// Registration HTTPS base URL.
    pub registration_base: Evidenced<Option<String>>,
}

/// Wire message-ID format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MessageIdFormat {
    /// Uppercase hex of `n` random bytes.
    UppercaseHexRandom {
        /// Number of random bytes.
        bytes: u8,
    },
}

/// A complete protocol profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolProfile {
    /// Stable identifier, e.g. `android-primary/2.26.36.75`.
    pub id: String,
    /// The reference client.
    pub reference: ReferenceClient,
    /// Framing facts.
    pub framing: FramingProfile,
    /// Noise facts.
    pub noise: NoiseProfile,
    /// Token dictionary.
    pub tokens: TokenDictionaryRef,
    /// Endpoints.
    pub endpoints: Endpoints,
    /// Message-ID format.
    pub message_id_format: Evidenced<Option<MessageIdFormat>>,
    /// Local limits.
    pub limits: Limits,
}

#[derive(Deserialize)]
struct ManifestFile {
    reference_client: ManifestClient,
    token_dictionary: ManifestTokens,
}

#[derive(Deserialize)]
struct ManifestClient {
    package: String,
    version_name: String,
    version_code: u64,
    apk_sha256: String,
    signing_certificate: ManifestCert,
}

#[derive(Deserialize)]
struct ManifestCert {
    sha256: String,
}

#[derive(Deserialize)]
struct ManifestTokens {
    version: u8,
    source_sha256: String,
}

impl ProtocolProfile {
    /// The Android primary-device profile as currently evidenced.
    ///
    /// Everything transport-facing is `ReferenceOnly` (taken from web
    /// multi-device implementations) or `Unknown`. This profile therefore
    /// permits offline work and experimental features only.
    pub fn android_reference() -> Result<Self> {
        let m: ManifestFile = serde_json::from_str(REFERENCE_MANIFEST_JSON)
            .map_err(|e| Error::invalid("reference-manifest.json", e.to_string()))?;
        Ok(Self {
            id: format!("android-primary/{}", m.reference_client.version_name),
            reference: ReferenceClient {
                package: m.reference_client.package,
                version_name: m.reference_client.version_name,
                version_code: m.reference_client.version_code,
                apk_sha256: m.reference_client.apk_sha256,
                signing_certificate_sha256: m.reference_client.signing_certificate.sha256,
            },
            framing: FramingProfile {
                length_prefix_bytes: Evidenced::reference_only(Some(3), "framing.length_prefix"),
                max_frame_len: Evidenced::reference_only(Some(1 << 24), "framing.max_frame_len"),
                connection_header: Evidenced::reference_only(
                    Some(vec![b'W', b'A', 6, m.token_dictionary.version]),
                    "framing.connection_header",
                ),
            },
            noise: NoiseProfile {
                pattern: Evidenced::reference_only(
                    Some("Noise_XX_25519_AESGCM_SHA256".to_owned()),
                    "noise.pattern",
                ),
                prologue: Evidenced::reference_only(
                    Some(vec![b'W', b'A', 6, m.token_dictionary.version]),
                    "noise.prologue",
                ),
                trust_root: Evidenced::reference_only(Some(WEB_MD_TRUST_ROOT), "noise.trust_root"),
                issuer_serial: Evidenced::reference_only(Some(0), "noise.issuer_serial"),
            },
            tokens: TokenDictionaryRef {
                version: m.token_dictionary.version,
                sha256: m.token_dictionary.source_sha256,
                label: EvidenceLabel::ReferenceOnly,
            },
            endpoints: Endpoints {
                chat: Evidenced::new(None, EvidenceLabel::Unknown, "transport.endpoint"),
                registration_base: Evidenced::new(
                    None,
                    EvidenceLabel::Unknown,
                    "registration.endpoints",
                ),
            },
            message_id_format: Evidenced::new(None, EvidenceLabel::Unknown, "messaging.id_format"),
            limits: Limits::default(),
        })
    }

    /// Ledger references of facts that are absent or `Unknown` among those
    /// required to open an authenticated connection.
    pub fn missing_for_connect(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        fn check<T>(missing: &mut Vec<&'static str>, e: &Evidenced<Option<T>>, r: &'static str) {
            if e.value.is_none() || e.label == EvidenceLabel::Unknown {
                missing.push(r);
            }
        }
        check(&mut missing, &self.endpoints.chat, "transport.endpoint");
        check(
            &mut missing,
            &self.framing.length_prefix_bytes,
            "framing.length_prefix",
        );
        check(&mut missing, &self.noise.pattern, "noise.pattern");
        check(&mut missing, &self.noise.prologue, "noise.prologue");
        check(&mut missing, &self.noise.trust_root, "noise.trust_root");
        missing.push("login.client_payload");
        missing
    }

    /// Ledger references of facts missing for primary registration.
    pub fn missing_for_registration(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.endpoints.registration_base.value.is_none()
            || self.endpoints.registration_base.label == EvidenceLabel::Unknown
        {
            missing.push("registration.endpoints");
        }
        missing.push("registration.payload_encoding");
        missing.push("registration.prerequisites");
        missing.push("registration.verification_steps");
        missing
    }

    /// Fail with `ProfileIncomplete` unless connection facts are present.
    pub fn require_connect_facts(&self) -> Result<()> {
        let missing = self.missing_for_connect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(Error::ProfileIncomplete {
                profile: self.id.clone(),
                missing,
            })
        }
    }

    /// Whether every transport-facing fact is production grade.
    pub fn is_production_grade(&self) -> bool {
        self.framing.length_prefix_bytes.label.is_production_grade()
            && self.noise.pattern.label.is_production_grade()
            && self.noise.trust_root.label.is_production_grade()
            && self.endpoints.chat.label.is_production_grade()
            && self.tokens.label.is_production_grade()
    }
}

/// Trust root recorded from web multi-device reference implementations
/// (`ReferenceOnly`; ledger `noise.trust_root`). Signature scheme in those
/// references is XEdDSA over Curve25519, not plain Ed25519.
pub const WEB_MD_TRUST_ROOT: [u8; 32] = [
    0x14, 0x23, 0x75, 0x57, 0x4d, 0x0a, 0x58, 0x71, 0x66, 0xaa, 0xe7, 0x1e, 0xbe, 0x51, 0x64, 0x37,
    0xc4, 0xa2, 0x8b, 0x73, 0xe3, 0x69, 0x5c, 0x6c, 0xe1, 0xf7, 0xf9, 0x54, 0x5d, 0xa8, 0xee, 0x6b,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_profile_loads_and_is_incomplete() {
        let p = ProtocolProfile::android_reference().unwrap();
        assert_eq!(p.reference.package, "com.whatsapp");
        assert_eq!(p.reference.version_name, "2.26.36.75");
        assert_eq!(p.tokens.version, 3);
        assert!(!p.is_production_grade());
        let err = p.require_connect_facts().unwrap_err();
        match err {
            Error::ProfileIncomplete { missing, .. } => {
                assert!(missing.contains(&"transport.endpoint"));
                assert!(missing.contains(&"login.client_payload"));
            }
            other => panic!("unexpected {other:?}"),
        }
        assert!(!p.missing_for_registration().is_empty());
    }
}
