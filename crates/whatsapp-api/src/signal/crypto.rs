//! Opt-in Signal cryptography adapter (ADR 0005).
//!
//! Uses the upstream implementation, not new cryptographic primitives.
//! This does not establish the Android trust root or enable login/messaging.

use wacore_libsignal::core::curve::PublicKey;

use crate::transport::cert::SignatureVerifier;

/// XEdDSA verification over raw 32-byte Curve25519 public keys.
///
/// The dependency contains AGPL-marked sources; see THIRD_PARTY_NOTICES.md.
#[derive(Debug, Default, Clone, Copy)]
pub struct SignalSignatureVerifier;

impl SignatureVerifier for SignalSignatureVerifier {
    fn verify(&self, public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
        PublicKey::from_djb_public_key_bytes(public_key)
            .is_ok_and(|key| key.verify_signature(message, signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wacore_libsignal::core::curve::PrivateKey;

    // Adapter regression, not an independent Android interoperability vector.
    #[test]
    fn verifies_signatures_and_rejects_tampering() {
        let private = PrivateKey::deserialize(&[42; 32]).unwrap();
        let public = private.public_key().unwrap();
        let raw: [u8; 32] = public.public_key_bytes().try_into().unwrap();
        let message = b"offline certificate details";
        let signature = private
            .calculate_signature(message, &mut rand::rng())
            .unwrap();
        let verifier = SignalSignatureVerifier;
        assert!(verifier.verify(&raw, message, &signature));
        assert!(!verifier.verify(&raw, b"changed details", &signature));
        let mut damaged = signature;
        damaged[7] ^= 1;
        assert!(!verifier.verify(&raw, message, &damaged));
        let other = PrivateKey::deserialize(&[43; 32])
            .unwrap()
            .public_key()
            .unwrap();
        assert!(!verifier.verify(
            other.public_key_bytes().try_into().unwrap(),
            message,
            &signature
        ));
        assert!(!verifier.verify(&raw, message, &[0; 64]));
    }
}
