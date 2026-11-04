//! Side-effect-free comparison collection verification. No network/key activation.
//! APK evidence and remaining application gates: docs/protocol/cross-device-sync.md.
use super::{
    app_state_crypto::{DecryptedMutation, MutationKeys, MutationOperation},
    app_state_hash::StateHash,
};
use crate::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};
pub mod patch;
mod snapshot;
pub mod storage;
pub use snapshot::{CollectionSnapshot, VerifiedCollectionSnapshot};

fn invalid(reason: &'static str) -> Error {
    Error::invalid("app_state_collection", reason)
}

/// An encrypted mutation from a decoded patch. Not authenticated on construction.
pub struct EncryptedMutation {
    /// One-based MAC operation semantics (not protobuf numeric values).
    pub operation: MutationOperation,
    /// Key ID to resolve against authenticated account-device state.
    pub key_id: [u8; 6],
    /// Exact index MAC supplied by the patch.
    pub index_mac: [u8; 32],
    /// IV, padded CBC ciphertext, then 32-byte value MAC.
    pub body: Vec<u8>,
}
impl std::fmt::Debug for EncryptedMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EncryptedMutation(<redacted>)")
    }
}

/// Decoded patch integrity inputs; parsing does not authorize its keys.
pub struct CollectionPatch {
    /// Collection ordering counter, distinct from action schema versions.
    pub version: u64,
    /// Key used for patch and snapshot MAC checks.
    pub key_id: [u8; 6],
    /// Claimed post-update snapshot MAC.
    pub snapshot_mac: [u8; 32],
    /// MAC over snapshot, ordered mutation MACs, version and collection.
    pub patch_mac: [u8; 32],
    /// In original wire order.
    pub mutations: Vec<EncryptedMutation>,
}
impl std::fmt::Debug for CollectionPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CollectionPatch(<redacted>)")
    }
}

/// [`CollectionState`] in a form the store can hold between sessions.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SavedCollectionState {
    /// Collection name.
    pub collection: String,
    /// Version we hold after the server accepted our last patch.
    pub version: u64,
    /// 128-byte LT hash.
    pub hash: Vec<u8>,
    /// index MAC → value MAC.
    pub values: Vec<(Vec<u8>, Vec<u8>)>,
    /// Exact authenticated index JSON → wire index MAC. Absent on legacy rows;
    /// their integrity map remains inspectable but cannot authorize mutations.
    #[serde(default)]
    pub logical_indexes: Option<Vec<(Vec<u8>, Vec<u8>)>>,
}
impl std::fmt::Debug for SavedCollectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SavedCollectionState")
            .field("version", &self.version)
            .field("entries", &self.values.len())
            .field("has_logical_indexes", &self.logical_indexes.is_some())
            .finish_non_exhaustive()
    }
}

/// Verified collection integrity state, not yet persisted or applied to UI data.
#[derive(Clone, PartialEq, Eq)]
pub struct CollectionState {
    collection: String,
    version: u64,
    hash: StateHash,
    values: BTreeMap<[u8; 32], [u8; 32]>,
    logical_indexes: Option<BTreeMap<Vec<u8>, [u8; 32]>>,
}
impl std::fmt::Debug for CollectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CollectionState(<redacted>)")
    }
}

/// Only produced after all MACs pass. Applying actions and committing their
/// state/version atomically remains the caller's separate responsibility.
#[derive(Debug)]
pub struct VerifiedCollectionPatch {
    /// Candidate integrity state. The source state remains unchanged.
    pub state: CollectionState,
    /// Authenticated mutations in wire order, with opaque action protobufs.
    pub mutations: Vec<DecryptedMutation>,
}

impl CollectionState {
    /// Initial empty version-zero candidate. Never use this to reset a saved
    /// account collection; existing accounts must restore their verified state.
    pub fn empty(collection: &str) -> Result<Self> {
        if collection.is_empty() || collection.len() > 128 {
            return Err(invalid("invalid collection name"));
        }
        Ok(Self {
            collection: collection.to_owned(),
            version: 0,
            hash: StateHash::default(),
            values: BTreeMap::new(),
            logical_indexes: Some(BTreeMap::new()),
        })
    }
    /// Current verified collection version.
    pub fn version(&self) -> u64 {
        self.version
    }
    /// Serialize for durable storage between patches: collection name, version,
    /// state hash and the index→value MAC map.
    ///
    /// A second patch has to build on the first, and the server tells us which
    /// version it now holds. Starting from `empty` every time would claim
    /// version 1 for ever and recompute a hash over nothing, which the server
    /// would be right to refuse.
    pub fn to_saved(&self) -> SavedCollectionState {
        SavedCollectionState {
            collection: self.collection.clone(),
            version: self.version,
            hash: self.hash.as_bytes().to_vec(),
            values: self
                .values
                .iter()
                .map(|(index, value)| (index.to_vec(), value.to_vec()))
                .collect(),
            logical_indexes: self.logical_indexes.as_ref().map(|indexes| {
                indexes
                    .iter()
                    .map(|(index, mac)| (index.clone(), mac.to_vec()))
                    .collect()
            }),
        }
    }

    /// The inverse of [`Self::to_saved`], refusing anything mis-sized rather
    /// than silently starting from a wrong hash.
    pub fn from_saved(saved: &SavedCollectionState) -> Result<Self> {
        Self::empty(&saved.collection)?;
        let hash: [u8; 128] = saved
            .hash
            .as_slice()
            .try_into()
            .map_err(|_| invalid("saved state hash is not 128 bytes"))?;
        let mut values = BTreeMap::new();
        for (index, value) in &saved.values {
            let index: [u8; 32] = index
                .as_slice()
                .try_into()
                .map_err(|_| invalid("saved index MAC is not 32 bytes"))?;
            let value: [u8; 32] = value
                .as_slice()
                .try_into()
                .map_err(|_| invalid("saved value MAC is not 32 bytes"))?;
            if values.insert(index, value).is_some() {
                return Err(invalid("duplicate saved wire index MAC"));
            }
        }
        let hash = StateHash::from_bytes(hash);
        let macs: Vec<_> = values.values().copied().collect();
        if StateHash::default().updated(&[], &macs)? != hash {
            return Err(invalid("saved state hash disagrees with value MACs"));
        }
        let logical_indexes = match &saved.logical_indexes {
            Some(saved_indexes) => {
                if saved_indexes.len() != values.len() {
                    return Err(invalid("saved logical index cardinality mismatch"));
                }
                let mut indexes = BTreeMap::new();
                let mut referenced = BTreeSet::new();
                for (index, mac) in saved_indexes {
                    validate_logical_index(index)?;
                    let mac: [u8; 32] = mac
                        .as_slice()
                        .try_into()
                        .map_err(|_| invalid("saved logical index MAC is not 32 bytes"))?;
                    if !values.contains_key(&mac) || !referenced.insert(mac) {
                        return Err(invalid(
                            "saved logical index has missing or repeated wire MAC",
                        ));
                    }
                    if indexes.insert(index.clone(), mac).is_some() {
                        return Err(invalid("duplicate saved logical index"));
                    }
                }
                Some(indexes)
            }
            // An empty map has no missing provenance; retain its version/hash.
            None if values.is_empty() => Some(BTreeMap::new()),
            None => None,
        };
        Ok(Self {
            collection: saved.collection.clone(),
            version: saved.version,
            hash,
            values,
            logical_indexes,
        })
    }

    /// Whether incremental verification/construction has authenticated index
    /// provenance. Legacy integrity-only rows need a matching verified snapshot.
    pub fn has_logical_indexes(&self) -> bool {
        self.logical_indexes.is_some()
    }

    fn require_logical_indexes(&self) -> Result<()> {
        if self.has_logical_indexes() {
            Ok(())
        } else {
            Err(invalid(
                "legacy state lacks authenticated logical indexes; matching authenticated snapshot required",
            ))
        }
    }

    // Current 43.72 0g1.A05/A0I store/select previous value MACs by the
    // decrypted index, which remains stable when mutation keys change.
    // Wire index MACs still authenticate exact bytes; they are not row identity.
    // Evidence: docs/protocol/app-state-logical-index.md.
    fn set_value(
        &mut self,
        index: &[u8],
        wire_index: [u8; 32],
        value: [u8; 32],
    ) -> Result<Option<[u8; 32]>> {
        validate_logical_index(index)?;
        self.require_logical_indexes()?;
        let indexes = self.logical_indexes.as_mut().expect("checked provenance");
        let old_index = indexes.get(index).copied();
        if self.values.contains_key(&wire_index) && old_index != Some(wire_index) {
            return Err(invalid("wire index MAC belongs to another logical index"));
        }
        let previous = match old_index {
            Some(old) => Some(
                self.values
                    .remove(&old)
                    .ok_or_else(|| invalid("logical index has no stored value MAC"))?,
            ),
            None => None,
        };
        indexes.insert(index.to_vec(), wire_index);
        self.values.insert(wire_index, value);
        Ok(previous)
    }

    fn remove_value(&mut self, index: &[u8]) -> Result<[u8; 32]> {
        self.require_logical_indexes()?;
        let wire_index = self
            .logical_indexes
            .as_mut()
            .expect("checked provenance")
            .remove(index)
            .ok_or_else(|| {
                invalid("REMOVE has no saved logical index; resynchronization required")
            })?;
        self.values
            .remove(&wire_index)
            .ok_or_else(|| invalid("logical index has no stored value MAC"))
    }

    /// Current verified rolling hash.
    pub fn hash(&self) -> &StateHash {
        &self.hash
    }
    /// Number of present indexes, without exposing identifiers or MACs.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    /// Whether this candidate has no present indexes.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Verify a strictly sequential patch without mutating this state. The key
    /// resolver must enforce device authorization/fingerprints; a matching ID is
    /// insufficient. Sequential-only and missing-REMOVE refusal are conservative
    /// local policies until broader recovery semantics are implemented.
    pub fn verify_patch<'a>(
        &self,
        patch: &CollectionPatch,
        mut key: impl FnMut(&[u8; 6]) -> Result<&'a MutationKeys>,
    ) -> Result<VerifiedCollectionPatch> {
        self.require_logical_indexes()?;
        if self.version.checked_add(1) != Some(patch.version) {
            return Err(invalid(
                "patch is not the next version; resynchronization required",
            ));
        }
        // Resource bounds are local, not claims about server limits.
        if patch.mutations.len() > 4096 {
            return Err(invalid("patch exceeds local mutation bound"));
        }
        let total = patch
            .mutations
            .iter()
            .try_fold(0_usize, |sum, m| sum.checked_add(m.body.len()))
            .ok_or_else(|| invalid("patch size overflow"))?;
        if total > 16 * 1024 * 1024 {
            return Err(invalid("patch exceeds local byte bound"));
        }
        let mut indexes = BTreeSet::new();
        let mut wire_macs = Vec::new();
        for mutation in &patch.mutations {
            if !indexes.insert(mutation.index_mac) {
                return Err(invalid("duplicate mutation index"));
            }
            if !(64..=1_048_576).contains(&mutation.body.len()) {
                return Err(invalid("invalid mutation size"));
            }
            wire_macs.push(
                mutation.body[mutation.body.len() - 32..]
                    .try_into()
                    .map_err(|_| invalid("missing value MAC"))?,
            );
        }
        let patch_key = key(&patch.key_id)?;
        patch_key.verify_patch(
            &patch.snapshot_mac,
            &wire_macs,
            patch.version,
            &self.collection,
            &patch.patch_mac,
        )?;
        let mut next = self.clone();
        let mut mutations = Vec::new();
        let mut removed = Vec::new();
        let mut added = Vec::new();
        let mut logical_indexes = BTreeSet::new();
        for (mutation, wire_mac) in patch.mutations.iter().zip(wire_macs) {
            let decoded = key(&mutation.key_id)?.decrypt_mutation(
                mutation.operation,
                &mutation.key_id,
                &mutation.body,
                &mutation.index_mac,
            )?;
            if !logical_indexes.insert(decoded.index().to_vec()) {
                return Err(invalid("duplicate logical mutation index"));
            }
            match decoded.operation() {
                MutationOperation::Set => {
                    if let Some(previous) =
                        next.set_value(decoded.index(), mutation.index_mac, wire_mac)?
                    {
                        removed.push(previous);
                    }
                    added.push(wire_mac);
                }
                MutationOperation::Remove => {
                    let previous = next.remove_value(decoded.index())?;
                    removed.push(previous);
                }
            }
            mutations.push(decoded);
        }
        next.hash = self.hash.updated(&removed, &added)?;
        patch_key.verify_snapshot(
            next.hash.as_bytes(),
            patch.version,
            &self.collection,
            &patch.snapshot_mac,
        )?;
        next.version = patch.version;
        Ok(VerifiedCollectionPatch {
            state: next,
            mutations,
        })
    }
}

fn validate_logical_index(index: &[u8]) -> Result<()> {
    let parts: zeroize::Zeroizing<Vec<String>> = zeroize::Zeroizing::new(
        serde_json::from_slice(index)
            .map_err(|_| invalid("invalid authenticated logical index"))?,
    );
    if parts.is_empty() || parts.iter().any(String::is_empty) {
        return Err(invalid(
            "logical index must contain nonempty string components",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixed<const N: usize>(value: &str) -> [u8; N] {
        hex::decode(value).unwrap().try_into().unwrap()
    }
    pub(super) fn patch(remove: bool) -> CollectionPatch {
        // Independent Python/OpenSSL mutation + HKDF/LTHash/HMAC vectors.
        let (body, snapshot, mac) = if remove {
            (
                "000102030405060708090a0b0c0d0e0f2192b6a9dde3b0f3712d6c99dc98976b3a3b4d03d17330a34a3211156d7360a31dc6482aacb38b0e0be76016dc6f7b7673e949ff5098ce118e05a12e44b8fa67",
                "372fa102e3b780fd0fac7ca9f521ebf00480b92e1e812fc867a6b751fde6f168",
                "81f911830690a8c2f52881a8eae26e8de2875da64ffece1cab25db631fff648f",
            )
        } else {
            (
                "000102030405060708090a0b0c0d0e0f2192b6a9dde3b0f3712d6c99dc98976b69e1b9f25155fa8ca9e9f0e206925fcc5f5fd8469f020699365e4750632c893bc24096315f6739aaeba497a176823909",
                "0e326b26aa4969d98e2fcf482d2790bf9933a9e11f093b7c18a17f0da491775d",
                "0960e911ee5e9d0e27b9e9e10b665dee186429aae47f0ed7bd0ade5c00081e8e",
            )
        };
        let id = [0, 1, 0, 0, 0, 2];
        CollectionPatch {
            version: if remove { 2 } else { 1 },
            key_id: id,
            snapshot_mac: fixed(snapshot),
            patch_mac: fixed(mac),
            mutations: vec![EncryptedMutation {
                operation: if remove {
                    MutationOperation::Remove
                } else {
                    MutationOperation::Set
                },
                key_id: id,
                index_mac: fixed(
                    "3d47311df4bc7aa8bd5ef2c16cbe7fc1ea07f18c86a2fb34078d1d416edb22d8",
                ),
                body: hex::decode(body).unwrap(),
            }],
        }
    }
    #[test]
    fn independent_set_remove_sequence_and_atomic_rejection() {
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        let initial = CollectionState::empty("regular").unwrap();
        let set = patch(false);
        let one = initial.verify_patch(&set, |_| Ok(&keys)).unwrap();
        assert_eq!(one.state.len(), 1);
        assert_eq!(one.state.version(), 1);
        assert_eq!(one.mutations.len(), 1);
        let removed = one.state.verify_patch(&patch(true), |_| Ok(&keys)).unwrap();
        assert!(removed.state.is_empty());
        assert_eq!(removed.state.version(), 2);
        assert_eq!(removed.state.hash(), &StateHash::default());
        let mut replacement = patch(false);
        replacement.version = 2;
        replacement.snapshot_mac =
            fixed("3dfbbb3f92d80538705091bf981fa35ba6cf294be0588fcb23b543d0375775f7");
        replacement.patch_mac =
            fixed("49c8f49f929d036e3aa03ab193af559daa0ab7e75bbd22b778264c4ea3278889");
        let replaced = one.state.verify_patch(&replacement, |_| Ok(&keys)).unwrap();
        assert_eq!(replaced.state.len(), 1);
        assert_eq!(replaced.state.hash(), one.state.hash());
        assert_eq!(replaced.state.version(), 2);
        let mut wrong_snapshot = patch(false);
        wrong_snapshot.snapshot_mac = [0; 32];
        wrong_snapshot.patch_mac =
            fixed("29f90db350e8a048e4ffa64ead425aae7b991286eda934f6470fc3a97dc50408");
        // Patch authentication succeeds, but the final snapshot check must fail.
        assert!(
            initial
                .verify_patch(&wrong_snapshot, |_| Ok(&keys))
                .is_err()
        );
        assert!(initial.is_empty());
        assert_eq!(initial.version(), 0);
        assert!(one.state.verify_patch(&set, |_| Ok(&keys)).is_err());
        assert!(initial.verify_patch(&patch(true), |_| Ok(&keys)).is_err());
        assert!(
            CollectionState::empty("critical")
                .unwrap()
                .verify_patch(&set, |_| Ok(&keys))
                .is_err()
        );
        assert!(
            initial
                .verify_patch(&set, |_| Err(invalid("unauthorized key")))
                .is_err()
        );
        for position in 0..set.mutations[0].body.len() {
            let mut bad = patch(false);
            bad.mutations[0].body[position] ^= 1;
            assert!(initial.verify_patch(&bad, |_| Ok(&keys)).is_err());
            assert!(initial.is_empty());
            assert_eq!(initial.version(), 0);
        }
        let mut bad = patch(false);
        bad.snapshot_mac[0] ^= 1;
        assert!(initial.verify_patch(&bad, |_| Ok(&keys)).is_err());
        let mut bad = patch(false);
        bad.mutations.push(patch(false).mutations.remove(0));
        assert!(initial.verify_patch(&bad, |_| Ok(&keys)).is_err());
    }
}

#[cfg(test)]
mod logical_index_tests {
    use super::*;

    const A: [u8; 6] = [0, 1, 0, 0, 0, 1];
    const B: [u8; 6] = [0, 1, 0, 0, 0, 2];
    const INDEX: &[u8] = br#"["fixture","same-index"]"#;

    fn record(
        keys: &MutationKeys,
        id: [u8; 6],
        operation: MutationOperation,
        index: &[u8],
    ) -> (EncryptedMutation, [u8; 32]) {
        let wire = keys
            .encrypt_mutation(operation, &id, index, Some(&[8, 1]), 1, [id[5]; 16])
            .unwrap();
        let mac = wire.value_mac;
        (
            EncryptedMutation {
                operation,
                key_id: id,
                index_mac: wire.index_mac,
                body: wire.body.to_vec(),
            },
            mac,
        )
    }

    fn incoming(
        keys: &MutationKeys,
        id: [u8; 6],
        version: u64,
        mutations: Vec<EncryptedMutation>,
        expected: &StateHash,
    ) -> CollectionPatch {
        let macs: Vec<[u8; 32]> = mutations
            .iter()
            .map(|m| m.body[m.body.len() - 32..].try_into().unwrap())
            .collect();
        let snapshot_mac = keys
            .snapshot_mac(expected.as_bytes(), version, "regular")
            .unwrap();
        let patch_mac = keys
            .patch_mac(&snapshot_mac, &macs, version, "regular")
            .unwrap();
        CollectionPatch {
            version,
            key_id: id,
            snapshot_mac,
            patch_mac,
            mutations,
        }
    }

    fn first(keys: &MutationKeys) -> CollectionState {
        let (mutation, mac) = record(keys, A, MutationOperation::Set, INDEX);
        let expected = StateHash::default().updated(&[], &[mac]).unwrap();
        CollectionState::empty("regular")
            .unwrap()
            .verify_patch(&incoming(keys, A, 1, vec![mutation], &expected), |_| {
                Ok(keys)
            })
            .unwrap()
            .state
    }

    #[test]
    fn cross_key_incoming_set_replaces_old_mac_and_remove_uses_logical_index() {
        let a = MutationKeys::derive(&[1; 32]).unwrap();
        let b = MutationKeys::derive(&[2; 32]).unwrap();
        assert_ne!(a.index_mac(INDEX).unwrap(), b.index_mac(INDEX).unwrap());
        let one = first(&a);
        let before = one.to_saved();
        let (set, mac) = record(&b, B, MutationOperation::Set, INDEX);
        // Native SQL has one row for this logical index: hash only the new MAC.
        let expected = StateHash::default().updated(&[], &[mac]).unwrap();
        let two = one
            .verify_patch(&incoming(&b, B, 2, vec![set], &expected), |_| Ok(&b))
            .unwrap()
            .state;
        assert_eq!(two.len(), 1);
        assert_eq!(two.hash(), &expected);
        assert!(!two.values.contains_key(&a.index_mac(INDEX).unwrap()));
        assert_eq!(one.to_saved(), before);
        let two = CollectionState::from_saved(&two.to_saved()).unwrap();
        let (remove, _) = record(&a, A, MutationOperation::Remove, INDEX);
        let three = two
            .verify_patch(
                &incoming(&a, A, 3, vec![remove], &StateHash::default()),
                |_| Ok(&a),
            )
            .unwrap()
            .state;
        assert!(three.is_empty());
        assert_eq!(three.version(), 3);
        assert_eq!(three.hash(), &StateHash::default());
    }

    #[test]
    fn cross_key_snapshot_mac_covering_both_old_and_new_rows_is_refused_atomically() {
        let a = MutationKeys::derive(&[1; 32]).unwrap();
        let b = MutationKeys::derive(&[2; 32]).unwrap();
        let one = first(&a);
        let before = one.to_saved();
        let (set, mac) = record(&b, B, MutationOperation::Set, INDEX);
        let wrong = one.hash().updated(&[], &[mac]).unwrap();
        assert!(
            one.verify_patch(&incoming(&b, B, 2, vec![set], &wrong), |_| Ok(&b))
                .is_err()
        );
        assert_eq!(one.to_saved(), before);
    }

    #[test]
    fn incoming_duplicate_logical_indexes_under_distinct_keys_are_refused() {
        let a = MutationKeys::derive(&[1; 32]).unwrap();
        let b = MutationKeys::derive(&[2; 32]).unwrap();
        let (one, mac_a) = record(&a, A, MutationOperation::Set, INDEX);
        let (two, mac_b) = record(&b, B, MutationOperation::Set, INDEX);
        let hash = StateHash::default().updated(&[], &[mac_a, mac_b]).unwrap();
        let empty = CollectionState::empty("regular").unwrap();
        let error = empty
            .verify_patch(&incoming(&a, A, 1, vec![one, two], &hash), |id| {
                if *id == A { Ok(&a) } else { Ok(&b) }
            })
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate logical mutation index")
        );
        assert!(empty.is_empty());
    }

    #[test]
    fn legacy_integrity_can_be_inspected_but_cannot_authorize_incremental_work() {
        let keys = MutationKeys::derive(&[1; 32]).unwrap();
        let mut saved = first(&keys).to_saved();
        saved.logical_indexes = None;
        let mut json = serde_json::to_value(&saved).unwrap();
        json.as_object_mut().unwrap().remove("logical_indexes");
        let legacy = CollectionState::from_saved(&serde_json::from_value(json).unwrap()).unwrap();
        assert!(!legacy.has_logical_indexes());
        assert_eq!(legacy.version(), saved.version);
        assert_eq!(legacy.hash().as_bytes().as_slice(), saved.hash);
        assert_eq!(legacy.to_saved().logical_indexes, None);
        assert!(!format!("{saved:?}").contains("fixture"));
        let (set, mac) = record(&keys, A, MutationOperation::Set, INDEX);
        let hash = StateHash::default().updated(&[], &[mac]).unwrap();
        let err = legacy
            .verify_patch(&incoming(&keys, A, 2, vec![set], &hash), |_| Ok(&keys))
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("matching authenticated snapshot required")
        );
        let mutation = patch::OutgoingMutation {
            operation: MutationOperation::Set,
            index: INDEX.to_vec(),
            action: Some(vec![8, 1]),
            version: 1,
        };
        assert!(legacy.build_patch(&keys, A, 0, &[mutation]).is_err());
        let mut empty = CollectionState::empty("regular").unwrap().to_saved();
        empty.version = 17;
        empty.logical_indexes = None;
        let empty = CollectionState::from_saved(&empty).unwrap();
        assert!(empty.has_logical_indexes());
        assert_eq!(empty.version(), 17);
    }

    #[test]
    fn saved_maps_require_deduplication_bijection_and_consistent_hash() {
        let keys = MutationKeys::derive(&[1; 32]).unwrap();
        let (a, mac_a) = record(&keys, A, MutationOperation::Set, INDEX);
        let (b, mac_b) = record(&keys, A, MutationOperation::Set, br#"["fixture","other"]"#);
        let hash = StateHash::default().updated(&[], &[mac_a, mac_b]).unwrap();
        let saved = CollectionState::empty("regular")
            .unwrap()
            .verify_patch(&incoming(&keys, A, 1, vec![a, b], &hash), |_| Ok(&keys))
            .unwrap()
            .state
            .to_saved();
        let mut corruptions = Vec::new();
        let mut bad = saved.clone();
        bad.values.push(bad.values[0].clone());
        corruptions.push(bad);
        let mut bad = saved.clone();
        bad.hash[0] ^= 1;
        corruptions.push(bad);
        let mut bad = saved.clone();
        bad.logical_indexes.as_mut().unwrap().pop();
        corruptions.push(bad);
        let mut bad = saved.clone();
        bad.logical_indexes.as_mut().unwrap()[0].1 = vec![99; 32];
        corruptions.push(bad);
        let mut bad = saved.clone();
        bad.logical_indexes.as_mut().unwrap()[0].1 =
            bad.logical_indexes.as_ref().unwrap()[1].1.clone();
        corruptions.push(bad);
        let mut bad = saved.clone();
        bad.logical_indexes.as_mut().unwrap()[0].0 =
            bad.logical_indexes.as_ref().unwrap()[1].0.clone();
        corruptions.push(bad);
        for index in [b"[]".as_slice(), b"[\"\"]", b"not-json", b"[1]"] {
            let mut bad = saved.clone();
            bad.logical_indexes.as_mut().unwrap()[0].0 = index.to_vec();
            corruptions.push(bad);
        }
        for bad in corruptions {
            assert!(CollectionState::from_saved(&bad).is_err());
        }
        assert_eq!(
            CollectionState::from_saved(&saved).unwrap().to_saved(),
            saved
        );
    }
}
