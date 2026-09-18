//! Comparison 2.26.37.70 app-state integrity primitives (CI5, Cz6, 1Ng).
//! These do not authenticate a device, activate a stored key, or apply state.
use crate::{Error, Result};
use hkdf::hmac::{Hmac, Mac, digest::KeyInit};
use prost::Message;
use sha2::{Sha256, Sha512};
use zeroize::{Zeroize, Zeroizing};

fn invalid() -> Error {
    Error::invalid(
        "app_state_integrity",
        "invalid input or integrity check failed",
    )
}

/// APK B1e: MAC operation bytes are one-based, unlike protobuf enum values.
#[derive(Debug, Clone, Copy)]
pub enum MutationOperation {
    /// Set/replace a value: MAC byte 1.
    Set,
    /// Remove a value: MAC byte 2.
    Remove,
}

#[derive(Message)]
struct MutationData {
    #[prost(bytes = "vec", optional, tag = "1")]
    index: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "2")]
    value: Option<Vec<u8>>,
    #[prost(bytes = "vec", tag = "3")]
    padding: Vec<u8>,
    #[prost(int32, optional, tag = "4")]
    version: Option<i32>,
}
impl Drop for MutationData {
    fn drop(&mut self) {
        if let Some(index) = &mut self.index {
            index.zeroize();
        }
        if let Some(value) = &mut self.value {
            value.zeroize();
        }
        self.padding.zeroize();
    }
}
#[derive(Message)]
struct ActionHeader {
    #[prost(int64, optional, tag = "1")]
    timestamp: Option<i64>,
}

/// A value/index-authenticated mutation, not yet authorized for state application.
/// The action protobuf stays opaque until its action-specific handler validates it.
pub struct DecryptedMutation {
    operation: MutationOperation,
    index: Zeroizing<Vec<u8>>,
    value: Option<Zeroizing<Vec<u8>>>,
    version: i32,
}
impl std::fmt::Debug for DecryptedMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DecryptedMutation(<redacted>)")
    }
}
impl DecryptedMutation {
    /// Operation authenticated by the value MAC; never infer it from the action.
    pub fn operation(&self) -> MutationOperation {
        self.operation
    }
    /// Exact authenticated index JSON, without normalization.
    pub fn index(&self) -> &[u8] {
        &self.index
    }
    /// Authenticated action protobuf, absent on some REMOVE mutations.
    pub fn value(&self) -> Option<&[u8]> {
        self.value.as_deref().map(|v| v.as_slice())
    }
    /// Action schema version, not the collection's ordering counter.
    pub fn version(&self) -> i32 {
        self.version
    }
}

/// Five separated 32-byte subkeys, with redacted diagnostics and zeroized storage.
pub struct MutationKeys(Zeroizing<[u8; 160]>);
impl std::fmt::Debug for MutationKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MutationKeys(<redacted>)")
    }
}
impl MutationKeys {
    /// Cz6.AKC -> ClC.A03: authenticate before decrypting, then validate the
    /// mutation envelope and exact index MAC. No state writes or key activation.
    pub fn decrypt_mutation(
        &self,
        operation: MutationOperation,
        key_id: &[u8; 6],
        body: &[u8],
        index_mac: &[u8],
    ) -> Result<DecryptedMutation> {
        // Local bound; minimum includes IV, one ciphertext block and value MAC.
        if !(64..=1_048_576).contains(&body.len()) {
            return Err(invalid());
        }
        let (encrypted, tag) = body.split_at(body.len() - 32);
        self.verify_value(operation, key_id, encrypted, tag)?;
        let mut plaintext = Zeroizing::new(Vec::new());
        wacore_libsignal::crypto::aes_256_cbc_decrypt_into(
            &encrypted[16..],
            &self.0[32..64],
            &encrypted[..16],
            &mut plaintext,
        )
        .map_err(|_| invalid())?;
        let mut data = MutationData::decode(plaintext.as_slice()).map_err(|_| invalid())?;
        let version = data.version.ok_or_else(invalid)?;
        let index = Zeroizing::new(data.index.take().ok_or_else(invalid)?);
        // ClC.A02 requires a nonempty JSON array of nonempty index strings.
        // MAC covers original UTF-8 bytes, never a reserialized JSON value.
        let parts: Zeroizing<Vec<String>> =
            Zeroizing::new(serde_json::from_slice(&index).map_err(|_| invalid())?);
        if parts.is_empty() || parts.iter().any(String::is_empty) {
            return Err(invalid());
        }
        self.verify_index(&index, index_mac)?;
        let value = data.value.take().map(Zeroizing::new);
        if matches!(operation, MutationOperation::Set) {
            let raw = value.as_deref().ok_or_else(invalid)?;
            let header = ActionHeader::decode(raw.as_slice()).map_err(|_| invalid())?;
            if header.timestamp.is_none() {
                return Err(invalid());
            }
        } else if let Some(raw) = &value {
            // REMOVE permits no action/timestamp, but present bytes must parse.
            ActionHeader::decode(raw.as_slice()).map_err(|_| invalid())?;
        }
        Ok(DecryptedMutation {
            operation,
            index,
            value,
            version,
        })
    }

    /// CI5: HKDF-SHA256, zero salt, UTF-8 label, 160 bytes. This operation alone
    /// does not establish that the supplied key belongs to the current account.
    pub fn derive(key: &[u8]) -> Result<Self> {
        if key.is_empty() || key.len() > 4096 {
            return Err(invalid());
        }
        let mut bytes = Zeroizing::new([0; 160]);
        hkdf::Hkdf::<Sha256>::new(None, key)
            .expand(b"WhatsApp Mutation Keys", &mut *bytes)
            .map_err(|_| invalid())?;
        Ok(Self(bytes))
    }

    /// Cz6.AKC: verify the exact UTF-8 index bytes, without JSON normalization.
    pub fn verify_index(&self, index: &[u8], expected: &[u8]) -> Result<()> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.0[..32]).map_err(|_| invalid())?;
        mac.update(index);
        mac.verify_slice(expected).map_err(|_| invalid())
    }

    /// Cz6.A01: HMAC-SHA512 truncated to 32 bytes, covering operation, six-byte
    /// key ID, IV+ciphertext, and the eight-byte BE length of operation+key ID.
    /// Call before any CBC decryption. This does not validate the index or patch.
    pub fn verify_value(
        &self,
        operation: MutationOperation,
        key_id: &[u8; 6],
        iv_ciphertext: &[u8],
        expected: &[u8],
    ) -> Result<()> {
        if expected.len() != 32
            || iv_ciphertext.len() < 32
            || !iv_ciphertext.len().is_multiple_of(16)
        {
            return Err(invalid());
        }
        let op = match operation {
            MutationOperation::Set => 1,
            MutationOperation::Remove => 2,
        };
        let mut mac = Hmac::<Sha512>::new_from_slice(&self.0[64..96]).map_err(|_| invalid())?;
        mac.update(&[op]);
        mac.update(key_id);
        mac.update(iv_ciphertext);
        mac.update(&7_u64.to_be_bytes());
        mac.verify_truncated_left(expected).map_err(|_| invalid())
    }

    /// Cz6.ADk: state hash || version (eight-byte BE) || UTF-8 collection name.
    /// The caller must independently compute the state hash and enforce order.
    pub fn verify_snapshot(
        &self,
        state_hash: &[u8; 128],
        version: u64,
        collection: &str,
        expected: &[u8],
    ) -> Result<()> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.0[96..128]).map_err(|_| invalid())?;
        mac.update(state_hash);
        mac.update(&version.to_be_bytes());
        mac.update(collection.as_bytes());
        mac.verify_slice(expected).map_err(|_| invalid())
    }

    /// Cz6.ADf: snapshot MAC || ordered value MACs || version || collection.
    /// A valid MAC is not permission to skip version/fingerprint validation.
    pub fn verify_patch(
        &self,
        snapshot_mac: &[u8; 32],
        value_macs: &[[u8; 32]],
        version: u64,
        collection: &str,
        expected: &[u8],
    ) -> Result<()> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.0[128..160]).map_err(|_| invalid())?;
        mac.update(snapshot_mac);
        for value_mac in value_macs {
            mac.update(value_mac);
        }
        mac.update(&version.to_be_bytes());
        mac.update(collection.as_bytes());
        mac.verify_slice(expected).map_err(|_| invalid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bytes(hex: &str) -> Vec<u8> {
        hex::decode(hex).unwrap()
    }
    #[test]
    fn independent_openssl_mutations_decrypt_only_after_authentication() {
        // Python stdlib HKDF/HMAC plus OpenSSL AES-256-CBC, synthetic key 0..31.
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        let id = [0, 1, 0, 0, 0, 2];
        let index_mac = bytes("3d47311df4bc7aa8bd5ef2c16cbe7fc1ea07f18c86a2fb34078d1d416edb22d8");
        let set = bytes(
            "000102030405060708090a0b0c0d0e0f2192b6a9dde3b0f3712d6c99dc98976b69e1b9f25155fa8ca9e9f0e206925fcc5f5fd8469f020699365e4750632c893bc24096315f6739aaeba497a176823909",
        );
        let remove = bytes(
            "000102030405060708090a0b0c0d0e0f2192b6a9dde3b0f3712d6c99dc98976b3a3b4d03d17330a34a3211156d7360a31dc6482aacb38b0e0be76016dc6f7b7673e949ff5098ce118e05a12e44b8fa67",
        );
        let decoded = keys
            .decrypt_mutation(MutationOperation::Set, &id, &set, &index_mac)
            .unwrap();
        assert_eq!(decoded.index(), br#"["fixture","setting"]"#);
        assert_eq!(decoded.value(), Some(b"\x08\x09".as_slice()));
        assert_eq!(decoded.version(), 1);
        assert_eq!(format!("{decoded:?}"), "DecryptedMutation(<redacted>)");
        let decoded = keys
            .decrypt_mutation(MutationOperation::Remove, &id, &remove, &index_mac)
            .unwrap();
        assert!(decoded.value().is_none());
        for offset in 0..set.len() {
            let mut tampered = set.clone();
            tampered[offset] ^= 1;
            assert!(
                keys.decrypt_mutation(MutationOperation::Set, &id, &tampered, &index_mac)
                    .is_err()
            );
        }
        assert!(
            keys.decrypt_mutation(MutationOperation::Set, &id, &set, &[0; 32])
                .is_err()
        );
        assert!(
            keys.decrypt_mutation(MutationOperation::Remove, &id, &set, &index_mac)
                .is_err()
        );
        assert!(
            keys.decrypt_mutation(MutationOperation::Set, &[0; 6], &set, &index_mac)
                .is_err()
        );
        assert!(
            keys.decrypt_mutation(MutationOperation::Set, &id, &set[..63], &index_mac)
                .is_err()
        );
        assert!(
            keys.decrypt_mutation(MutationOperation::Set, &id, &vec![0; 1_048_577], &index_mac)
                .is_err()
        );
    }

    #[test]
    fn authenticated_but_malformed_mutations_stay_unexposed() {
        let keys = MutationKeys::derive(&[7; 32]).unwrap();
        let id = [0; 6];
        let seal = |index: &[u8], value: Option<Vec<u8>>, version| {
            let data = MutationData {
                index: Some(index.to_vec()),
                value,
                padding: vec![],
                version,
            };
            let raw = Zeroizing::new(data.encode_to_vec());
            let mut ciphertext = Vec::new();
            wacore_libsignal::crypto::aes_256_cbc_encrypt_into(
                &raw,
                &keys.0[32..64],
                &[0; 16],
                &mut ciphertext,
            )
            .unwrap();
            let mut body = vec![0; 16];
            body.extend_from_slice(&ciphertext);
            let mut mac = Hmac::<Sha512>::new_from_slice(&keys.0[64..96]).unwrap();
            mac.update(&[1]);
            mac.update(&id);
            mac.update(&body);
            mac.update(&7_u64.to_be_bytes());
            body.extend_from_slice(&mac.finalize().into_bytes()[..32]);
            let mut mac = Hmac::<Sha256>::new_from_slice(&keys.0[..32]).unwrap();
            mac.update(index);
            (body, mac.finalize().into_bytes())
        };
        for (index, value, version) in [
            (b"[]".as_slice(), Some(vec![8, 9]), Some(1)),
            (b"[\"\"]", Some(vec![8, 9]), Some(1)),
            (b"not-json", Some(vec![8, 9]), Some(1)),
            (b"[\"fixture\"]", Some(vec![8, 9]), None),
            (b"[\"fixture\"]", None, Some(1)),
            (b"[\"fixture\"]", Some(vec![]), Some(1)),
            (b"[\"fixture\"]", Some(vec![8]), Some(1)),
        ] {
            let (body, index_mac) = seal(index, value, version);
            assert!(
                keys.decrypt_mutation(MutationOperation::Set, &id, &body, &index_mac)
                    .is_err()
            );
        }
    }
    #[test]
    fn independent_hkdf_and_mac_vectors_bind_all_inputs() {
        // Generated independently with Python stdlib hashlib/hmac implementing
        // RFC5869 extract+five expand blocks; inputs contain no account data.
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        assert_eq!(
            &keys.0[..],
            bytes(concat!(
                "61387bcf643616a68bd611a45516b3980418323087d78bf08c615645549434b4",
                "900ba2843ba5fb0cee55cf2a4de9503dce68187d3f6b95b420b008bde66f5a20d",
                "0879f6b61f0bcba79faad3f47a8a768fd7fc04a6cc8b3ecefedfa087413226f",
                "69b2e91be6587307c43c29b027a71fdd55be25f07ca726714115430d23093071",
                "693845bdd996652aca9ca0b96d0f081abf29943303c5eb19bdb84f38b24c32ab"
            ))
        );
        let index = bytes("3d47311df4bc7aa8bd5ef2c16cbe7fc1ea07f18c86a2fb34078d1d416edb22d8");
        keys.verify_index(br#"["fixture","setting"]"#, &index)
            .unwrap();
        assert!(
            keys.verify_index(br#"["fixture", "setting"]"#, &index)
                .is_err()
        );
        let value = bytes("975b196ca2eeb2275a07a81659e33c6282802f97257e8a63714f8de10f464fce");
        let id = [0, 1, 0, 0, 0, 2];
        let mut encrypted: Vec<u8> = (0..48).collect();
        keys.verify_value(MutationOperation::Set, &id, &encrypted, &value)
            .unwrap();
        assert!(
            keys.verify_value(MutationOperation::Remove, &id, &encrypted, &value)
                .is_err()
        );
        assert!(
            keys.verify_value(MutationOperation::Set, &[0; 6], &encrypted, &value)
                .is_err()
        );
        for length in [0, 1, 31] {
            assert!(
                keys.verify_value(MutationOperation::Set, &id, &encrypted, &value[..length])
                    .is_err()
            );
        }
        encrypted[0] ^= 1;
        assert!(
            keys.verify_value(MutationOperation::Set, &id, &encrypted, &value)
                .is_err()
        );
        let snapshot: [u8; 32] =
            bytes("4ff60c25aaa36bda18cd221610b28dd872dd3107a7fa435743f03d65d7185121")
                .try_into()
                .unwrap();
        let version = 0x0102030405060708;
        keys.verify_snapshot(&[0; 128], version, "regular", &snapshot)
            .unwrap();
        assert!(
            keys.verify_snapshot(&[1; 128], version, "regular", &snapshot)
                .is_err()
        );
        assert!(
            keys.verify_snapshot(&[0; 128], version + 1, "regular", &snapshot)
                .is_err()
        );
        assert!(
            keys.verify_snapshot(&[0; 128], version, "critical", &snapshot)
                .is_err()
        );
        let patch = bytes("aaad77c99d67d496e9a7c85a29f2d2371b734d497d5483804249f429bdcabc29");
        let values = [value.try_into().unwrap()];
        keys.verify_patch(&snapshot, &values, version, "regular", &patch)
            .unwrap();
        assert!(
            keys.verify_patch(&snapshot, &[], version, "regular", &patch)
                .is_err()
        );
        assert!(
            keys.verify_patch(&snapshot, &values, version + 1, "regular", &patch)
                .is_err()
        );
        assert!(
            keys.verify_patch(&snapshot, &values, version, "critical", &patch)
                .is_err()
        );
        assert!(
            keys.verify_patch(&[0; 32], &values, version, "regular", &patch)
                .is_err()
        );
        let ordered_patch =
            bytes("99838b22fddddb08a3903f7b714e62e361e4de79a5f730d26233f857276036f6");
        keys.verify_patch(
            &snapshot,
            &[values[0], [165; 32]],
            version,
            "regular",
            &ordered_patch,
        )
        .unwrap();
        assert!(
            keys.verify_patch(
                &snapshot,
                &[[165; 32], values[0]],
                version,
                "regular",
                &ordered_patch
            )
            .is_err()
        );
        assert_eq!(format!("{keys:?}"), "MutationKeys(<redacted>)");
    }
}
