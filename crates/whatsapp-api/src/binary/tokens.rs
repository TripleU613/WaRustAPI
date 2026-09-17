//! Token dictionary and tag constants.
//!
//! The dictionary is loaded from a JSON fixture that carries its own
//! provenance block. See `fixtures/tokens/web-md-v3.json`.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::evidence::EvidenceLabel;

/// Structural tag bytes (`ReferenceOnly`, ledger `binary.tags`).
pub mod tags {
    /// Empty list / absent value.
    pub const LIST_EMPTY: u8 = 0;
    /// First double-byte dictionary page.
    pub const DICTIONARY_0: u8 = 236;
    /// Second double-byte dictionary page.
    pub const DICTIONARY_1: u8 = 237;
    /// Third double-byte dictionary page.
    pub const DICTIONARY_2: u8 = 238;
    /// Fourth double-byte dictionary page.
    pub const DICTIONARY_3: u8 = 239;
    /// Interop JID.
    pub const INTEROP_JID: u8 = 245;
    /// Facebook/Messenger JID.
    pub const FB_JID: u8 = 246;
    /// Agent/device JID.
    pub const AD_JID: u8 = 247;
    /// List with u8 size.
    pub const LIST_8: u8 = 248;
    /// List with u16 size.
    pub const LIST_16: u8 = 249;
    /// user@server JID.
    pub const JID_PAIR: u8 = 250;
    /// Packed uppercase-hex string.
    pub const HEX_8: u8 = 251;
    /// Bytes with u8 length.
    pub const BINARY_8: u8 = 252;
    /// Bytes with 20-bit length.
    pub const BINARY_20: u8 = 253;
    /// Bytes with u32 length.
    pub const BINARY_32: u8 = 254;
    /// Packed decimal/`-`/`.` string.
    pub const NIBBLE_8: u8 = 255;
    /// Maximum characters in a packed string.
    pub const PACKED_MAX: usize = 127;
}

/// Embedded fixture.
pub const WEB_MD_V3_JSON: &str = include_str!("../../fixtures/tokens/web-md-v3.json");

#[derive(Deserialize)]
struct Provenance {
    evidence: EvidenceLabel,
    source: String,
    source_revision: String,
    ledger_ref: String,
}

#[derive(Deserialize)]
struct FixtureFile {
    provenance: Provenance,
    dictionary_version: u8,
    single_byte: Vec<String>,
    double_byte: Vec<Vec<String>>,
}

/// A token dictionary with forward and reverse lookups.
#[derive(Debug)]
pub struct TokenDictionary {
    version: u8,
    label: EvidenceLabel,
    source: String,
    source_revision: String,
    ledger_ref: String,
    fixture_sha256: String,
    single: Vec<String>,
    double: Vec<Vec<String>>,
    single_index: HashMap<String, u8>,
    double_index: HashMap<String, (u8, u8)>,
}

impl TokenDictionary {
    /// Parse a fixture JSON document.
    pub fn from_fixture_json(json: &str) -> Result<Self, String> {
        let f: FixtureFile = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if f.single_byte.len() > tags::DICTIONARY_0 as usize {
            return Err("single-byte token list overlaps structural tags".into());
        }
        if f.double_byte.len() > 4 || f.double_byte.iter().any(|p| p.len() > 256) {
            return Err("double-byte pages exceed 4x256".into());
        }
        let mut single_index = HashMap::with_capacity(f.single_byte.len());
        for (i, t) in f.single_byte.iter().enumerate() {
            if !t.is_empty() {
                single_index.insert(t.clone(), i as u8);
            }
        }
        let mut double_index = HashMap::with_capacity(f.double_byte.len() * 256);
        for (d, page) in f.double_byte.iter().enumerate() {
            for (i, t) in page.iter().enumerate() {
                if !t.is_empty() {
                    double_index.insert(t.clone(), (d as u8, i as u8));
                }
            }
        }
        Ok(Self {
            version: f.dictionary_version,
            label: f.provenance.evidence,
            source: f.provenance.source,
            source_revision: f.provenance.source_revision,
            ledger_ref: f.provenance.ledger_ref,
            fixture_sha256: hex::encode(Sha256::digest(json.as_bytes())),
            single: f.single_byte,
            double: f.double_byte,
            single_index,
            double_index,
        })
    }

    /// The embedded v3 dictionary (content recovered from the pinned Android
    /// APK and identical to the web multi-device references).
    pub fn web_md_v3() -> &'static TokenDictionary {
        static DICT: OnceLock<TokenDictionary> = OnceLock::new();
        DICT.get_or_init(|| {
            TokenDictionary::from_fixture_json(WEB_MD_V3_JSON)
                .expect("embedded token fixture is valid")
        })
    }

    /// Dictionary version byte.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Evidence label for the Android primary profile.
    pub fn label(&self) -> EvidenceLabel {
        self.label
    }

    /// Provenance description.
    pub fn provenance(&self) -> (&str, &str, &str) {
        (&self.source, &self.source_revision, &self.ledger_ref)
    }

    /// SHA-256 (hex) of the fixture document.
    pub fn fixture_sha256(&self) -> &str {
        &self.fixture_sha256
    }

    /// Single-byte token by index.
    pub fn single(&self, index: u8) -> Option<&str> {
        self.single
            .get(index as usize)
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    }

    /// Double-byte token by page and index.
    pub fn double(&self, dictionary: u8, index: u8) -> Option<&str> {
        self.double
            .get(dictionary as usize)?
            .get(index as usize)
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    }

    /// Reverse lookup for a single-byte token.
    pub fn index_of_single(&self, token: &str) -> Option<u8> {
        self.single_index.get(token).copied()
    }

    /// Reverse lookup for a double-byte token.
    pub fn index_of_double(&self, token: &str) -> Option<(u8, u8)> {
        self.double_index.get(token).copied()
    }

    /// Number of single-byte tokens (including the empty index 0).
    pub fn single_len(&self) -> usize {
        self.single.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_dictionary_matches_reference_counts() {
        let d = TokenDictionary::web_md_v3();
        assert_eq!(d.version(), 3);
        assert_eq!(d.label(), EvidenceLabel::Observed);
        assert_eq!(d.single_len(), 236);
        assert_eq!(d.single(3), Some("s.whatsapp.net"));
        assert_eq!(d.single(0), None);
        assert_eq!(d.index_of_single("s.whatsapp.net"), Some(3));
        assert_eq!(d.index_of_double("read-self"), Some((0, 0)));
        assert_eq!(d.double(0, 0), Some("read-self"));
        assert!(d.double(4, 0).is_none());
    }

    #[test]
    fn rejects_dictionary_that_overlaps_tags() {
        let json = r#"{"provenance":{"evidence":"Unknown","source":"","source_revision":"","ledger_ref":""},
            "dictionary_version":1,"single_byte":["x"],"double_byte":[[],[],[],[],[]]}"#;
        assert!(TokenDictionary::from_fixture_json(json).is_err());
    }
}
