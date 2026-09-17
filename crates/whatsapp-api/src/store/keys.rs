//! Master key injection and the at-rest key hierarchy (README section 11.3).
//!
//! The master key lives outside the database. A [`KeySource`] hands it to the
//! store; the store derives a per-database data key and an index key with
//! HKDF-SHA256 using a random per-database salt stored in `meta`. A wrong
//! key is detected through a key-check record and reported as
//! [`crate::Error::WrongStorageKey`], never repaired by generating a new key.

use std::fmt;

use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

use crate::error::{Error, Result};

/// Length of the master key in bytes.
pub const MASTER_KEY_LEN: usize = 32;

/// A 32-byte master key. Zeroized on drop; `Debug` is redacted.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MasterKey([u8; MASTER_KEY_LEN]);

impl MasterKey {
    /// Wrap raw key bytes.
    pub fn from_bytes(bytes: [u8; MASTER_KEY_LEN]) -> Self {
        Self(bytes)
    }

    /// Parse a 64-character hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.len() != MASTER_KEY_LEN * 2 {
            return Err(Error::invalid(
                "master_key",
                format!("expected {} hex characters", MASTER_KEY_LEN * 2),
            ));
        }
        let v = Zeroizing::new(
            hex::decode(s).map_err(|_| Error::invalid("master_key", "not valid hex"))?,
        );
        let mut b = [0u8; MASTER_KEY_LEN];
        b.copy_from_slice(&v);
        Ok(Self(b))
    }

    /// Generate a fresh key from the OS CSPRNG.
    pub fn generate() -> Result<Self> {
        let mut b = [0u8; MASTER_KEY_LEN];
        getrandom::fill(&mut b).map_err(|e| {
            Error::Io(std::io::Error::other(format!("entropy source failed: {e}")))
        })?;
        Ok(Self(b))
    }

    pub(crate) fn as_bytes(&self) -> &[u8; MASTER_KEY_LEN] {
        &self.0
    }
}

impl fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MasterKey(<redacted>)")
    }
}

/// Supplies the master key to a store at open time.
pub trait KeySource: Send + Sync + fmt::Debug {
    /// Obtain the master key. Failures are typed; a missing key must not be
    /// replaced by a generated one.
    fn master_key(&self) -> Result<MasterKey>;
}

/// Reads a hex-encoded master key from an environment variable.
#[derive(Debug, Clone)]
pub struct EnvKeySource {
    variable: String,
}

impl EnvKeySource {
    /// Read from the named environment variable.
    pub fn new(variable: impl Into<String>) -> Self {
        Self {
            variable: variable.into(),
        }
    }
}

impl KeySource for EnvKeySource {
    fn master_key(&self) -> Result<MasterKey> {
        let raw = std::env::var(&self.variable).map_err(|_| {
            Error::invalid(
                "master_key",
                format!("environment variable {} is not set", self.variable),
            )
        })?;
        let raw = Zeroizing::new(raw);
        MasterKey::from_hex(&raw)
    }
}

/// Holds a key in memory. Intended for tests and for callers that obtained
/// the key from their own secret store.
#[derive(Clone)]
pub struct StaticKeySource(MasterKey);

impl StaticKeySource {
    /// Wrap a key.
    pub fn new(key: MasterKey) -> Self {
        Self(key)
    }
}

impl fmt::Debug for StaticKeySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StaticKeySource(<redacted>)")
    }
}

impl KeySource for StaticKeySource {
    fn master_key(&self) -> Result<MasterKey> {
        Ok(self.0.clone())
    }
}

/// Derived per-database keys.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub(crate) struct DerivedKeys {
    /// Encrypts record envelopes.
    pub data_key: [u8; 32],
    /// Keys blinded lookup identifiers.
    pub index_key: [u8; 32],
    /// Identifier of the master key that produced these (first 4 bytes of a
    /// domain-separated hash), stored in every envelope.
    pub key_id: u32,
}

impl fmt::Debug for DerivedKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DerivedKeys(key_id={})", self.key_id)
    }
}

/// Domain-separation labels. Changing any of them is a key rotation.
const INFO_DATA: &[u8] = b"whatsapp-api/store/data-key/v1";
const INFO_INDEX: &[u8] = b"whatsapp-api/store/index-key/v1";
const INFO_KEY_ID: &[u8] = b"whatsapp-api/store/key-id/v1";

impl DerivedKeys {
    pub(crate) fn derive(master: &MasterKey, salt: &[u8; 32]) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(salt), master.as_bytes());
        let mut data_key = [0u8; 32];
        let mut index_key = [0u8; 32];
        let mut id = [0u8; 4];
        // 32-byte outputs are always within HKDF-SHA256's 255*32 limit.
        hk.expand(INFO_DATA, &mut data_key)
            .expect("HKDF output length is within bounds");
        hk.expand(INFO_INDEX, &mut index_key)
            .expect("HKDF output length is within bounds");
        hk.expand(INFO_KEY_ID, &mut id)
            .expect("HKDF output length is within bounds");
        Self {
            data_key,
            index_key,
            key_id: u32::from_be_bytes(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip_and_validation() {
        let k = MasterKey::generate().unwrap();
        let h = hex::encode(k.as_bytes());
        let k2 = MasterKey::from_hex(&h).unwrap();
        assert_eq!(k.as_bytes(), k2.as_bytes());
        assert!(MasterKey::from_hex("abc").is_err());
        assert!(MasterKey::from_hex(&"zz".repeat(32)).is_err());
        assert!(!format!("{k:?}").contains(&h));
    }

    #[test]
    fn derivation_is_deterministic_and_salted() {
        let k = MasterKey::from_bytes([7u8; 32]);
        let a = DerivedKeys::derive(&k, &[1u8; 32]);
        let b = DerivedKeys::derive(&k, &[1u8; 32]);
        let c = DerivedKeys::derive(&k, &[2u8; 32]);
        assert_eq!(a.data_key, b.data_key);
        assert_ne!(a.data_key, c.data_key);
        assert_ne!(a.data_key, a.index_key);
    }
}
