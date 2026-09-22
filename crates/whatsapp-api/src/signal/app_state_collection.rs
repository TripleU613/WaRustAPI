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

/// Verified collection integrity state, not yet persisted or applied to UI data.
#[derive(Clone, PartialEq, Eq)]
pub struct CollectionState {
    collection: String,
    version: u64,
    hash: StateHash,
    values: BTreeMap<[u8; 32], [u8; 32]>,
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
        })
    }
    /// Current verified collection version.
    pub fn version(&self) -> u64 {
        self.version
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
        for (mutation, wire_mac) in patch.mutations.iter().zip(wire_macs) {
            let decoded = key(&mutation.key_id)?.decrypt_mutation(
                mutation.operation,
                &mutation.key_id,
                &mutation.body,
                &mutation.index_mac,
            )?;
            match decoded.operation() {
                MutationOperation::Set => {
                    if let Some(previous) = next.values.insert(mutation.index_mac, wire_mac) {
                        removed.push(previous);
                    }
                    added.push(wire_mac);
                }
                MutationOperation::Remove => {
                    let previous = next.values.remove(&mutation.index_mac).ok_or_else(|| {
                        invalid("REMOVE has no saved index; resynchronization required")
                    })?;
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
