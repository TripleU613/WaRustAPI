//! Server certificate chain binding for the Noise handshake (README 8.3).
//!
//! The chain structure (`CertChain { intermediate, leaf }`, each with
//! protobuf `details` + 64-byte signature) and the verification order are
//! `ReferenceOnly` (ledger `noise.cert_chain`). The signature scheme used by
//! the references is XEdDSA over Curve25519, which belongs to the Signal
//! backend; this module therefore takes a [`SignatureVerifier`] and never
//! ships an accept-all implementation.

use prost::Message;

use super::TransportError;
use crate::model::time::Timestamp;
use crate::proto::wa_cert::CertChain;
use crate::proto::wa_cert::cert_chain::noise_certificate::Details;

/// Verifies a 64-byte signature over `message` with a 32-byte public key.
pub trait SignatureVerifier: Send + Sync + std::fmt::Debug {
    /// Return `true` only if the signature is valid.
    fn verify(&self, public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool;
}

/// Trust anchors for chain verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainPolicy {
    /// Root public key that must have signed the intermediate certificate.
    pub trust_root: [u8; 32],
    /// Expected `issuer_serial` on the intermediate certificate.
    pub intermediate_issuer_serial: u32,
}

/// Facts established by a successful verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedChain {
    /// Serial of the intermediate certificate.
    pub intermediate_serial: u32,
    /// Serial of the leaf certificate.
    pub leaf_serial: u32,
    /// Leaf key, equal to the remote static key it was checked against.
    pub leaf_key: [u8; 32],
    /// Leaf expiry (seconds since epoch), if declared.
    pub leaf_not_after: Option<u64>,
}

fn reject(reason: impl Into<String>) -> TransportError {
    TransportError::Handshake(reason.into())
}

fn check_validity(details: &Details, now: Timestamp, label: &str) -> Result<(), TransportError> {
    let now_s = now.as_unix_seconds();
    if let Some(nb) = details.not_before
        && i64::try_from(nb).map_or(true, |nb| now_s < nb)
    {
        return Err(reject(format!("{label} certificate not yet valid")));
    }
    if let Some(na) = details.not_after
        && i64::try_from(na).map_or(true, |na| now_s > na)
    {
        return Err(reject(format!("{label} certificate expired")));
    }
    Ok(())
}

fn sig64(sig: &[u8], label: &str) -> Result<[u8; 64], TransportError> {
    sig.try_into()
        .map_err(|_| reject(format!("{label} signature length {} != 64", sig.len())))
}

fn key32(key: &[u8], label: &str) -> Result<[u8; 32], TransportError> {
    key.try_into()
        .map_err(|_| reject(format!("{label} key length {} != 32", key.len())))
}

/// Verify an encoded `CertChain` and bind its leaf key to `remote_static`.
///
/// Order (mirrors the reference implementations): structural completeness,
/// root -> intermediate signature, intermediate issuer serial, intermediate
/// -> leaf signature, intermediate validity, leaf issuer serial, leaf key ==
/// remote static (constant-time), leaf validity.
pub fn verify_chain(
    chain_bytes: &[u8],
    remote_static: &[u8; 32],
    now: Timestamp,
    policy: &ChainPolicy,
    verifier: &dyn SignatureVerifier,
) -> Result<VerifiedChain, TransportError> {
    let chain = CertChain::decode(chain_bytes).map_err(|e| reject(format!("cert chain decode: {e}")))?;
    let intermediate = chain
        .intermediate
        .ok_or_else(|| reject("missing intermediate certificate"))?;
    let leaf = chain.leaf.ok_or_else(|| reject("missing leaf certificate"))?;
    let (int_details_raw, int_sig) = match (intermediate.details, intermediate.signature) {
        (Some(d), Some(s)) => (d, s),
        _ => return Err(reject("intermediate certificate incomplete")),
    };
    let (leaf_details_raw, leaf_sig) = match (leaf.details, leaf.signature) {
        (Some(d), Some(s)) => (d, s),
        _ => return Err(reject("leaf certificate incomplete")),
    };
    let int_sig = sig64(&int_sig, "intermediate")?;
    let leaf_sig = sig64(&leaf_sig, "leaf")?;

    if !verifier.verify(&policy.trust_root, &int_details_raw, &int_sig) {
        return Err(reject("intermediate signature does not verify against trust root"));
    }
    let int_details = Details::decode(&int_details_raw[..])
        .map_err(|e| reject(format!("intermediate details decode: {e}")))?;
    if int_details.issuer_serial != Some(policy.intermediate_issuer_serial) {
        return Err(reject("unexpected intermediate issuer serial"));
    }
    let int_key = key32(
        int_details.key.as_deref().ok_or_else(|| reject("intermediate key missing"))?,
        "intermediate",
    )?;
    if !verifier.verify(&int_key, &leaf_details_raw, &leaf_sig) {
        return Err(reject("leaf signature does not verify against intermediate"));
    }
    check_validity(&int_details, now, "intermediate")?;
    let leaf_details = Details::decode(&leaf_details_raw[..])
        .map_err(|e| reject(format!("leaf details decode: {e}")))?;
    if leaf_details.issuer_serial.is_none() || leaf_details.issuer_serial != int_details.serial {
        return Err(reject("leaf issuer serial does not match intermediate serial"));
    }
    let leaf_key = key32(
        leaf_details.key.as_deref().ok_or_else(|| reject("leaf key missing"))?,
        "leaf",
    )?;
    if !constant_time_eq(&leaf_key, remote_static) {
        return Err(reject("leaf key does not match remote static key"));
    }
    check_validity(&leaf_details, now, "leaf")?;
    Ok(VerifiedChain {
        intermediate_serial: int_details.serial.unwrap_or_default(),
        leaf_serial: leaf_details.serial.unwrap_or_default(),
        leaf_key,
        leaf_not_after: leaf_details.not_after,
    })
}

fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use sha2::{Digest, Sha256};

    use super::*;
    use crate::proto::wa_cert::cert_chain::NoiseCertificate;

    /// Test-only verifier: signature = SHA-256(key || message) repeated twice.
    /// This exercises the chain logic; it is not a real signature scheme.
    #[derive(Debug)]
    struct FakeVerifier;

    fn fake_sign(key: &[u8; 32], msg: &[u8]) -> Vec<u8> {
        let h = Sha256::digest([&key[..], msg].concat());
        [h.as_slice(), h.as_slice()].concat()
    }

    impl SignatureVerifier for FakeVerifier {
        fn verify(&self, public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
            fake_sign(public_key, message) == signature
        }
    }

    const ROOT: [u8; 32] = [1u8; 32];
    const INT_KEY: [u8; 32] = [2u8; 32];
    const LEAF_KEY: [u8; 32] = [3u8; 32];

    fn details(serial: u32, issuer: u32, key: &[u8], nb: u64, na: u64) -> Vec<u8> {
        Details {
            serial: Some(serial),
            issuer_serial: Some(issuer),
            key: Some(Bytes::copy_from_slice(key)),
            not_before: Some(nb),
            not_after: Some(na),
        }
        .encode_to_vec()
    }

    fn chain(int_details: Vec<u8>, int_signer: &[u8; 32], leaf_details: Vec<u8>, leaf_signer: &[u8; 32]) -> Vec<u8> {
        CertChain {
            intermediate: Some(NoiseCertificate {
                signature: Some(Bytes::from(fake_sign(int_signer, &int_details))),
                details: Some(Bytes::from(int_details)),
            }),
            leaf: Some(NoiseCertificate {
                signature: Some(Bytes::from(fake_sign(leaf_signer, &leaf_details))),
                details: Some(Bytes::from(leaf_details)),
            }),
        }
        .encode_to_vec()
    }

    fn policy() -> ChainPolicy {
        ChainPolicy {
            trust_root: ROOT,
            intermediate_issuer_serial: 0,
        }
    }

    fn now() -> Timestamp {
        Timestamp::from_unix_seconds(1_700_000_000)
    }

    #[test]
    fn valid_chain_binds_leaf_to_static() {
        let c = chain(
            details(7, 0, &INT_KEY, 1_600_000_000, 1_800_000_000),
            &ROOT,
            details(9, 7, &LEAF_KEY, 1_600_000_000, 1_800_000_000),
            &INT_KEY,
        );
        let v = verify_chain(&c, &LEAF_KEY, now(), &policy(), &FakeVerifier).unwrap();
        assert_eq!(v.intermediate_serial, 7);
        assert_eq!(v.leaf_serial, 9);
        assert_eq!(v.leaf_key, LEAF_KEY);
    }

    #[test]
    fn valid_looking_chain_that_does_not_bind_to_remote_key_fails() {
        let c = chain(
            details(7, 0, &INT_KEY, 1_600_000_000, 1_800_000_000),
            &ROOT,
            details(9, 7, &LEAF_KEY, 1_600_000_000, 1_800_000_000),
            &INT_KEY,
        );
        let other_static = [4u8; 32];
        let err = verify_chain(&c, &other_static, now(), &policy(), &FakeVerifier).unwrap_err();
        assert!(matches!(err, TransportError::Handshake(ref r) if r.contains("remote static")));
    }

    #[test]
    fn rejects_bad_root_signature_serials_and_expiry() {
        // Intermediate signed by the wrong root.
        let c = chain(
            details(7, 0, &INT_KEY, 1_600_000_000, 1_800_000_000),
            &[9u8; 32],
            details(9, 7, &LEAF_KEY, 1_600_000_000, 1_800_000_000),
            &INT_KEY,
        );
        assert!(verify_chain(&c, &LEAF_KEY, now(), &policy(), &FakeVerifier).is_err());
        // Wrong intermediate issuer serial.
        let c = chain(
            details(7, 5, &INT_KEY, 1_600_000_000, 1_800_000_000),
            &ROOT,
            details(9, 7, &LEAF_KEY, 1_600_000_000, 1_800_000_000),
            &INT_KEY,
        );
        assert!(verify_chain(&c, &LEAF_KEY, now(), &policy(), &FakeVerifier).is_err());
        // Leaf issuer serial does not match intermediate serial.
        let c = chain(
            details(7, 0, &INT_KEY, 1_600_000_000, 1_800_000_000),
            &ROOT,
            details(9, 8, &LEAF_KEY, 1_600_000_000, 1_800_000_000),
            &INT_KEY,
        );
        assert!(verify_chain(&c, &LEAF_KEY, now(), &policy(), &FakeVerifier).is_err());
        // Expired leaf.
        let c = chain(
            details(7, 0, &INT_KEY, 1_600_000_000, 1_800_000_000),
            &ROOT,
            details(9, 7, &LEAF_KEY, 1_600_000_000, 1_650_000_000),
            &INT_KEY,
        );
        assert!(verify_chain(&c, &LEAF_KEY, now(), &policy(), &FakeVerifier).is_err());
        // Leaf signed by a key other than the intermediate's.
        let c = chain(
            details(7, 0, &INT_KEY, 1_600_000_000, 1_800_000_000),
            &ROOT,
            details(9, 7, &LEAF_KEY, 1_600_000_000, 1_800_000_000),
            &[8u8; 32],
        );
        assert!(verify_chain(&c, &LEAF_KEY, now(), &policy(), &FakeVerifier).is_err());
    }

    #[test]
    fn rejects_structural_problems() {
        assert!(verify_chain(b"\xff\xff", &LEAF_KEY, now(), &policy(), &FakeVerifier).is_err());
        let missing_leaf = CertChain {
            intermediate: Some(NoiseCertificate {
                details: Some(Bytes::from_static(b"x")),
                signature: Some(Bytes::from(vec![0u8; 64])),
            }),
            leaf: None,
        }
        .encode_to_vec();
        assert!(verify_chain(&missing_leaf, &LEAF_KEY, now(), &policy(), &FakeVerifier).is_err());
        let short_sig = CertChain {
            intermediate: Some(NoiseCertificate {
                details: Some(Bytes::from_static(b"x")),
                signature: Some(Bytes::from(vec![0u8; 63])),
            }),
            leaf: Some(NoiseCertificate {
                details: Some(Bytes::from_static(b"x")),
                signature: Some(Bytes::from(vec![0u8; 64])),
            }),
        }
        .encode_to_vec();
        assert!(matches!(
            verify_chain(&short_sig, &LEAF_KEY, now(), &policy(), &FakeVerifier),
            Err(TransportError::Handshake(ref r)) if r.contains("signature length")
        ));
    }
}
