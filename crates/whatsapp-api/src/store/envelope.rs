//! At-rest record encryption (README section 11.3).
//!
//! Format (all integers big-endian):
//!
//! ```text
//! u8  format version (=1)
//! u8  algorithm id (1 = XChaCha20-Poly1305)
//! u32 key id
//! [u8; 24] nonce (random, fresh per write)
//! [u8]     ciphertext || 16-byte tag
//! ```
//!
//! Associated data binds the record to its account generation, table, key and
//! record format version using an unambiguous length-delimited encoding, so a
//! valid ciphertext moved to another row fails authentication.

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};

use super::StoreError;

/// Current envelope format version.
pub const ENVELOPE_VERSION: u8 = 1;
/// Algorithm identifier for XChaCha20-Poly1305.
pub const ALG_XCHACHA20_POLY1305: u8 = 1;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
const HEADER_LEN: usize = 1 + 1 + 4 + NONCE_LEN;

/// Record identity authenticated as associated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordAad<'a> {
    /// Account generation the record belongs to.
    pub generation: u32,
    /// Logical table name.
    pub table: &'a str,
    /// Record key bytes.
    pub key: &'a [u8],
    /// Plaintext record format version.
    pub record_version: u16,
}

impl RecordAad<'_> {
    /// Length-delimited encoding: `u16 len || bytes` for each variable field.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + 2 + self.table.len() + 2 + self.key.len() + 2 + 2);
        out.extend_from_slice(b"aad1");
        out.extend_from_slice(&self.generation.to_be_bytes());
        out.extend_from_slice(&(self.table.len() as u16).to_be_bytes());
        out.extend_from_slice(self.table.as_bytes());
        out.extend_from_slice(&(self.key.len() as u16).to_be_bytes());
        out.extend_from_slice(self.key);
        out.extend_from_slice(&self.record_version.to_be_bytes());
        out
    }
}

/// Sealing/opening context for one derived data key.
#[derive(Clone)]
pub(crate) struct Sealer {
    cipher: XChaCha20Poly1305,
    key_id: u32,
}

impl std::fmt::Debug for Sealer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sealer(key_id={})", self.key_id)
    }
}

impl Sealer {
    pub(crate) fn new(data_key: &[u8; 32], key_id: u32) -> Self {
        Self {
            cipher: XChaCha20Poly1305::new(data_key.into()),
            key_id,
        }
    }

    /// Encrypt `plaintext` under `aad` with a fresh random nonce.
    pub(crate) fn seal(
        &self,
        aad: &RecordAad<'_>,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, StoreError> {
        if aad.key.len() > u16::MAX as usize || aad.table.len() > u16::MAX as usize {
            return Err(StoreError::RecordTooLarge);
        }
        let mut nonce = [0u8; NONCE_LEN];
        getrandom::fill(&mut nonce).map_err(|e| StoreError::Entropy(e.to_string()))?;
        let aad_bytes = aad.encode();
        let ct = self
            .cipher
            .encrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: plaintext,
                    aad: &aad_bytes,
                },
            )
            .map_err(|_| StoreError::Crypto("encryption failed"))?;
        let mut out = Vec::with_capacity(HEADER_LEN + ct.len());
        out.push(ENVELOPE_VERSION);
        out.push(ALG_XCHACHA20_POLY1305);
        out.extend_from_slice(&self.key_id.to_be_bytes());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Decrypt an envelope produced by [`Self::seal`] under the same `aad`.
    pub(crate) fn open(&self, aad: &RecordAad<'_>, envelope: &[u8]) -> Result<Vec<u8>, StoreError> {
        if envelope.len() < HEADER_LEN + TAG_LEN {
            return Err(StoreError::MalformedEnvelope("too short"));
        }
        let version = envelope[0];
        if version != ENVELOPE_VERSION {
            return Err(StoreError::UnsupportedEnvelopeVersion(version));
        }
        if envelope[1] != ALG_XCHACHA20_POLY1305 {
            return Err(StoreError::MalformedEnvelope("unknown algorithm"));
        }
        let key_id = u32::from_be_bytes([envelope[2], envelope[3], envelope[4], envelope[5]]);
        if key_id != self.key_id {
            return Err(StoreError::KeyIdMismatch {
                expected: self.key_id,
                found: key_id,
            });
        }
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&envelope[6..HEADER_LEN]);
        let aad_bytes = aad.encode();
        self.cipher
            .decrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: &envelope[HEADER_LEN..],
                    aad: &aad_bytes,
                },
            )
            .map_err(|_| StoreError::AuthenticationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sealer() -> Sealer {
        Sealer::new(&[9u8; 32], 0xC0FFEE)
    }

    #[test]
    fn roundtrip() {
        let s = sealer();
        let aad = RecordAad {
            generation: 1,
            table: "outbox",
            key: b"k1",
            record_version: 1,
        };
        let env = s.seal(&aad, b"hello").unwrap();
        assert_eq!(s.open(&aad, &env).unwrap(), b"hello");
        assert_eq!(env.len(), HEADER_LEN + 5 + TAG_LEN);
    }

    #[test]
    fn nonces_are_fresh() {
        let s = sealer();
        let aad = RecordAad {
            generation: 1,
            table: "t",
            key: b"k",
            record_version: 1,
        };
        let a = s.seal(&aad, b"x").unwrap();
        let b = s.seal(&aad, b"x").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn substitution_into_another_row_fails() {
        let s = sealer();
        let aad1 = RecordAad {
            generation: 1,
            table: "signal_sessions",
            key: b"device-a",
            record_version: 1,
        };
        let env = s.seal(&aad1, b"secret").unwrap();
        for aad2 in [
            RecordAad {
                key: b"device-b",
                ..aad1
            },
            RecordAad {
                table: "prekeys",
                ..aad1
            },
            RecordAad {
                generation: 2,
                ..aad1
            },
            RecordAad {
                record_version: 2,
                ..aad1
            },
        ] {
            assert!(matches!(
                s.open(&aad2, &env),
                Err(StoreError::AuthenticationFailed)
            ));
        }
    }

    #[test]
    fn aad_encoding_is_unambiguous() {
        let a = RecordAad {
            generation: 1,
            table: "ab",
            key: b"c",
            record_version: 1,
        };
        let b = RecordAad {
            generation: 1,
            table: "a",
            key: b"bc",
            record_version: 1,
        };
        assert_ne!(a.encode(), b.encode());
    }

    #[test]
    fn wrong_key_and_tamper_fail() {
        let s = sealer();
        let other = Sealer::new(&[10u8; 32], 0xC0FFEE);
        let aad = RecordAad {
            generation: 1,
            table: "t",
            key: b"k",
            record_version: 1,
        };
        let mut env = s.seal(&aad, b"payload").unwrap();
        assert!(matches!(
            other.open(&aad, &env),
            Err(StoreError::AuthenticationFailed)
        ));
        let last = env.len() - 1;
        env[last] ^= 1;
        assert!(matches!(
            s.open(&aad, &env),
            Err(StoreError::AuthenticationFailed)
        ));
        env[0] = 9;
        assert!(matches!(
            s.open(&aad, &env),
            Err(StoreError::UnsupportedEnvelopeVersion(9))
        ));
        let short = vec![1u8; 10];
        assert!(matches!(
            s.open(&aad, &short),
            Err(StoreError::MalformedEnvelope(_))
        ));
        let mismatched = Sealer::new(&[9u8; 32], 1);
        let env2 = s.seal(&aad, b"p").unwrap();
        assert!(matches!(
            mismatched.open(&aad, &env2),
            Err(StoreError::KeyIdMismatch { .. })
        ));
    }
}
