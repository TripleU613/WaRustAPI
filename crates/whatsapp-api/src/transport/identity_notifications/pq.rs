//! CQc.A00 signs the unchanged KEMPublicKey.A00 bytes; this is not KEM admission.
use crate::binary::{Node, NodeContent};
use serde::Serialize;
use wacore_libsignal::protocol::IdentityKey;

/// Signature-only result for a selected directory PQ key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PqSignatureStatus {
    /// Required byte bodies are absent, duplicated or structurally unsupported.
    Malformed,
    /// The local inspector bound was exceeded; not an APK key-size rule.
    PayloadTooLarge,
    /// Original bytes verify under the returned candidate identity.
    SignatureVerified,
    /// Original bytes fail signature verification under the candidate identity.
    InvalidSignature,
}

/// Redacted PQ signature diagnostic, never key/session/adoption authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PqSignatureInspection {
    /// Signature-only outcome.
    pub status: PqSignatureStatus,
    /// Original value byte count, not a recognized KEM format or algorithm.
    pub value_bytes: Option<usize>,
    /// Null until crypto verification; independent of the classic signed key.
    pub signature_valid: Option<bool>,
    /// `not_checked` (value absent/oversized), `valid_ml_kem_1024` or
    /// `invalid_ml_kem_1024`: local encoding validation, independent of the
    /// signature. A valid encoding is not trust, admission or adoption.
    pub key_validation: &'static str,
}

fn one<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    let mut matches = node.child_nodes().iter().filter(|n| n.description == name);
    let selected = matches.next()?;
    matches.next().is_none().then_some(selected)
}
fn bytes(node: &Node) -> Option<&[u8]> {
    match &node.content {
        NodeContent::Bytes(value) => Some(value),
        _ => None,
    }
}

pub(super) fn inspect(user: &Node, identity: &IdentityKey) -> Option<PqSignatureInspection> {
    user.child("pqkey")?;
    let mut result = PqSignatureInspection {
        status: PqSignatureStatus::Malformed,
        value_bytes: None,
        signature_valid: None,
        key_validation: "not_checked",
    };
    let Some(node) = one(user, "pqkey") else {
        return Some(result);
    };
    let Some(value) = one(node, "value").and_then(bytes) else {
        return Some(result);
    };
    result.value_bytes = Some(value.len());
    // Native KEMPublicKey keeps the bytes opaque; the ML-KEM encoding check is
    // the same one hybrid::bundle_for applies before any derivation.
    if value.len() <= 65536 {
        result.key_validation = if crate::signal::kem::public_key(value).is_ok() {
            "valid_ml_kem_1024"
        } else {
            "invalid_ml_kem_1024"
        };
    }
    let Some(signature) = one(node, "signature")
        .and_then(bytes)
        .filter(|s| s.len() == 64)
    else {
        return Some(result);
    };
    if value.len() > 65536 {
        result.status = PqSignatureStatus::PayloadTooLarge;
        return Some(result);
    }
    // KEMPublicKey wraps/returns the supplied array; do not prepend the EC type or re-encode it.
    let valid = identity.public_key().verify_signature(value, signature);
    result.signature_valid = Some(valid);
    result.status = if valid {
        PqSignatureStatus::SignatureVerified
    } else {
        PqSignatureStatus::InvalidSignature
    };
    Some(result)
}
