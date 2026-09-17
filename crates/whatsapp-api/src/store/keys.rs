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
        getrandom::fill(&mut b)
            .map_err(|e| Error::Io(std::io::Error::other(format!("entropy source failed: {e}"))))?;
        Ok(Self(b))
    }

    pub(crate) fn as_bytes(&self) -> &[u8; MASTER_KEY_LEN] {
        &self.0
    }

    /// Hex encoding of the key as raw bytes, for an operator tool that must
    /// hand a freshly generated key to a secret store. Callers must keep the
    /// result in a zeroizing container.
    pub fn to_hex_bytes(&self) -> Vec<u8> {
        hex::encode(self.0).into_bytes()
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

/// Key-encryption key derived from the master key and the per-database salt.
/// It only ever wraps the random data/index keys stored in `meta`.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub(crate) struct Kek(pub [u8; 32]);

/// Random per-database keys, stored wrapped under the [`Kek`].
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub(crate) struct DataKeys {
    /// Encrypts record envelopes.
    pub data_key: [u8; 32],
    /// Keys blinded lookup identifiers.
    pub index_key: [u8; 32],
}

impl fmt::Debug for DataKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DataKeys(<redacted>)")
    }
}

/// Domain-separation labels. Changing any of them is a format change.
const INFO_KEK: &[u8] = b"whatsapp-api/store/kek/v1";
const INFO_KEY_ID: &[u8] = b"whatsapp-api/store/key-id/v1";

impl Kek {
    pub(crate) fn derive(master: &MasterKey, salt: &[u8; 32]) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(salt), master.as_bytes());
        let mut kek = [0u8; 32];
        hk.expand(INFO_KEK, &mut kek)
            .expect("HKDF output length is within bounds");
        Self(kek)
    }
}

impl DataKeys {
    /// Generate fresh random keys.
    pub(crate) fn generate() -> Result<Self> {
        let mut data_key = [0u8; 32];
        let mut index_key = [0u8; 32];
        getrandom::fill(&mut data_key)
            .map_err(|e| Error::Io(std::io::Error::other(format!("entropy source failed: {e}"))))?;
        getrandom::fill(&mut index_key)
            .map_err(|e| Error::Io(std::io::Error::other(format!("entropy source failed: {e}"))))?;
        Ok(Self {
            data_key,
            index_key,
        })
    }

    /// 64-byte serialization for wrapping.
    pub(crate) fn to_bytes(&self) -> Zeroizing<[u8; 64]> {
        let mut out = Zeroizing::new([0u8; 64]);
        out[..32].copy_from_slice(&self.data_key);
        out[32..].copy_from_slice(&self.index_key);
        out
    }

    pub(crate) fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != 64 {
            return None;
        }
        let mut data_key = [0u8; 32];
        let mut index_key = [0u8; 32];
        data_key.copy_from_slice(&b[..32]);
        index_key.copy_from_slice(&b[32..]);
        Some(Self {
            data_key,
            index_key,
        })
    }

    /// Identifier of the data key stored in every envelope: the first four
    /// bytes of a domain-separated HKDF output. Stable across master-key
    /// rotations, which only re-wrap the data keys.
    pub(crate) fn key_id(&self) -> u32 {
        let hk = Hkdf::<Sha256>::new(None, &self.data_key);
        let mut id = [0u8; 4];
        hk.expand(INFO_KEY_ID, &mut id)
            .expect("HKDF output length is within bounds");
        u32::from_be_bytes(id)
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
    fn kek_is_deterministic_and_salted() {
        let k = MasterKey::from_bytes([7u8; 32]);
        let a = Kek::derive(&k, &[1u8; 32]);
        let b = Kek::derive(&k, &[1u8; 32]);
        let c = Kek::derive(&k, &[2u8; 32]);
        assert_eq!(a.0, b.0);
        assert_ne!(a.0, c.0);
    }

    #[test]
    fn data_keys_roundtrip_and_key_id_is_stable() {
        let d = DataKeys::generate().unwrap();
        let back = DataKeys::from_bytes(&*d.to_bytes()).unwrap();
        assert_eq!(back.data_key, d.data_key);
        assert_eq!(back.key_id(), d.key_id());
        assert!(DataKeys::from_bytes(&[0u8; 10]).is_none());
    }
}
