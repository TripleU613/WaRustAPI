//! Comparison-APK registration encryption, not device attestation.
//!
//! `RetryingHttpClient.A00` (2.26.37.70) uses a fresh Curve25519 agreement
//! directly as an AES-256-GCM key, a zero nonce, no AAD, and prefixes the raw
//! ephemeral public key. The zero nonce is safe only with a fresh key per seal.
//! Never reuse the persisted account/Noise key here.

use crate::{Error, Result};
use wacore_libsignal::crypto;
use zeroize::Zeroizing;

/// Public server key from `HQQ.<clinit>` in the comparison APK, not a secret.
const SERVER_PUBLIC_KEY: [u8; 32] = [
    0x8e, 0x8c, 0x0f, 0x74, 0xc3, 0xeb, 0xc5, 0xd7, 0xa6, 0x86, 0x5c, 0x6c, 0x3c, 0x84, 0x38, 0x56,
    0xb0, 0x61, 0x21, 0xcc, 0xe8, 0xea, 0x77, 0x4d, 0x22, 0xfb, 0x6f, 0x12, 0x25, 0x12, 0x30, 0x2d,
];

pub(super) fn seal(plaintext: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    seal_for(plaintext, &SERVER_PUBLIC_KEY)
}

fn seal_for(plaintext: &[u8], server: &[u8; 32]) -> Result<Zeroizing<Vec<u8>>> {
    let mut private = Zeroizing::new([0u8; 32]);
    getrandom::fill(private.as_mut())
        .map_err(|_| Error::invalid("registration_envelope", "ephemeral randomness failed"))?;
    let mut basepoint = [0u8; 32];
    basepoint[0] = 9;
    let public = crypto::x25519_agreement(&private, &basepoint)
        .map_err(|_| Error::invalid("registration_envelope", "public key generation failed"))?;
    let key = Zeroizing::new(
        crypto::x25519_agreement(&private, server)
            .map_err(|_| Error::invalid("registration_envelope", "key agreement failed"))?,
    );
    if key.iter().all(|byte| *byte == 0) {
        return Err(Error::invalid(
            "registration_envelope",
            "invalid server public key",
        ));
    }
    let mut out = Zeroizing::new(public.to_vec());
    crypto::aes_256_gcm_encrypt(&key, &[0; 12], &[], plaintext, &mut out)
        .map_err(|_| Error::invalid("registration_envelope", "encryption failed"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wacore_libsignal::core::curve::KeyPair;

    #[test]
    fn envelope_roundtrip_tag_and_fresh_ephemeral_keys() {
        let server = KeyPair::generate(&mut rand::rng());
        let public: [u8; 32] = server.public_key.public_key_bytes().try_into().unwrap();
        let plaintext = b"cc=1&reason=&client_metrics=%7B%22attempts%22%3A1%7D";
        let first = seal_for(plaintext, &public).unwrap();
        let second = seal_for(plaintext, &public).unwrap();
        assert_eq!(first.len(), 32 + plaintext.len() + 16);
        assert_ne!(&first[..32], &second[..32]);
        let ephemeral: [u8; 32] = first[..32].try_into().unwrap();
        let key = Zeroizing::new(
            crypto::x25519_agreement(server.private_key.serialize(), &ephemeral).unwrap(),
        );
        let mut decoded = Zeroizing::new(Vec::new());
        crypto::aes_256_gcm_decrypt(&key, &[0; 12], &[], &first[32..], &mut decoded).unwrap();
        assert_eq!(decoded.as_slice(), plaintext);
        let mut tampered = first;
        tampered[32] ^= 1;
        assert!(
            crypto::aes_256_gcm_decrypt(&key, &[0; 12], &[], &tampered[32..], &mut Vec::new())
                .is_err()
        );
    }

    #[test]
    fn rejects_noncontributory_server_key() {
        assert!(seal_for(b"test", &[0; 32]).is_err());
    }
}
