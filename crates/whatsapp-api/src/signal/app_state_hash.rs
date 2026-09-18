//! APK 2.26.37.70 C6C.A00: 128-byte, little-endian 16-bit additive state hash.
//! This primitive does not authenticate mutations or choose which old MACs to
//! remove. Its caller must supply authenticated changes and verify snapshot MACs.
use crate::{Error, Result};
use sha2::Sha256;
use zeroize::Zeroizing;

/// Rolling collection hash. No collection/version or authorization is implied.
#[derive(Clone, PartialEq, Eq)]
pub struct StateHash([u8; 128]);
impl Default for StateHash {
    fn default() -> Self {
        Self([0; 128])
    }
}
impl std::fmt::Debug for StateHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StateHash(<redacted>)")
    }
}
impl StateHash {
    /// Restore an exact-width hash from authenticated, account-scoped state.
    pub fn from_bytes(bytes: [u8; 128]) -> Self {
        Self(bytes)
    }
    /// Bytes used by the snapshot-MAC check, not a printable status summary.
    pub fn as_bytes(&self) -> &[u8; 128] {
        &self.0
    }

    /// Calculate a candidate without mutating committed state. Replacements
    /// subtract the old value MAC before adding the new one; REMOVE subtracts
    /// the stored old MAC, not its own operation's wire MAC.
    pub fn updated(&self, removed: &[[u8; 32]], added: &[[u8; 32]]) -> Result<Self> {
        let mut candidate = self.clone();
        for (values, add) in [(removed, false), (added, true)] {
            for value_mac in values {
                let mut expanded = Zeroizing::new([0; 128]);
                hkdf::Hkdf::<Sha256>::new(None, value_mac)
                    .expand(b"WhatsApp Patch Integrity", &mut *expanded)
                    .map_err(|_| Error::invalid("app_state_hash", "key expansion failed"))?;
                for (word, delta) in candidate
                    .0
                    .as_chunks_mut::<2>()
                    .0
                    .iter_mut()
                    .zip(expanded.as_chunks::<2>().0)
                {
                    let old = u16::from_le_bytes([word[0], word[1]]);
                    let delta = u16::from_le_bytes([delta[0], delta[1]]);
                    let next = if add {
                        old.wrapping_add(delta)
                    } else {
                        old.wrapping_sub(delta)
                    };
                    word.copy_from_slice(&next.to_le_bytes());
                }
            }
        }
        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(hex: &str) -> [u8; 128] {
        hex::decode(hex).unwrap().try_into().unwrap()
    }
    #[test]
    fn independent_little_endian_wraparound_vectors() {
        // Python stdlib HMAC RFC5869 expansion; independent u16 LE modulo sums.
        let a: [u8; 32] = std::array::from_fn(|i| i as u8);
        let b = [165; 32];
        let zero = StateHash::default();
        let one = zero.updated(&[], &[a]).unwrap();
        assert_eq!(
            one.as_bytes(),
            &fixture(
                "fa29a23a1edb970a8db0b86d1ae94d4461209cbea2dae9dc39ef1ba696d7213042d167b5db383efd034591861867ae2746357ecee3bccd070b1110e0abb6f18c91d687194dd9f7ea2d474f815bed0adb07fd6256915d9bda410982dab7bc7e07f525caf02ef2bc7b9f8d7b70469ab8ada2fc32c2c718728f04d0d340a6a368a2"
            )
        );
        assert_eq!(
            one.updated(&[], &[b]).unwrap().as_bytes(),
            &fixture(
                "2261b9bc089db3ffbc5990db23bd3bd705f8871fd8df5eaf2c3073879cbca18b0784069fad5b70d3723aa2b2cc1a960bcc968d4c0f181fcf08b9777e2368e30d8195ea1d1901e63af3ddb376ba74b5c99e4f40225df03da8442d2cbd4f180cf46d8f1f81c60ce39ff69c654298994ee74dcdcb82c70c942ef649ac8d9822bfa0"
            )
        );
        assert_eq!(
            one.updated(&[b], &[]).unwrap().as_bytes(),
            &fixture(
                "d2f28bb834197b155e07e0ff11155fb1bd48b15d6cd5740a46aec3c490f2a1d47d1ec8cb09160c27944f805a64b3c643c0d36f50b7617b400e69a9413305ff0ba117241581b1089b67b0eb8bfc655fec70aa848ac5caf90c3ee5d8f71f61f01a7dbc756096d79557487e919ef49a2274f72b9901c72450f01256faf3b42411a4"
            )
        );
        assert_eq!(one.updated(&[a], &[]).unwrap(), zero);
        assert_eq!(
            one.updated(&[a], &[b]).unwrap(),
            zero.updated(&[], &[b]).unwrap()
        );
        assert_eq!(
            zero.updated(&[], &[a, b]).unwrap(),
            zero.updated(&[], &[b, a]).unwrap()
        );
        assert_eq!(zero.as_bytes(), &[0; 128]); // candidates never mutate source
        assert_eq!(StateHash::from_bytes(*one.as_bytes()), one);
        assert_eq!(format!("{one:?}"), "StateHash(<redacted>)");
        // Independently generated snapshot MAC binds the derived candidate to
        // its collection/version, not just the arithmetic fixture in isolation.
        let keys = super::super::app_state_crypto::MutationKeys::derive(&a).unwrap();
        let tag = hex::decode("aea859e56044c3218f67103fb585e3e1d7cdc1a1464e4dbdaf1d5d4ffcc0ceb6")
            .unwrap();
        keys.verify_snapshot(one.as_bytes(), 1, "regular", &tag)
            .unwrap();
        assert!(
            keys.verify_snapshot(zero.as_bytes(), 1, "regular", &tag)
                .is_err()
        );
    }
}
