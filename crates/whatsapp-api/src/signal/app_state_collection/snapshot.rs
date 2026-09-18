//! APK BTa/BSR/BMY/BMV/BMW/BMX snapshot structures; no downloading or persistence.
use super::*;
use prost::Message;

#[derive(Message)]
struct WireSnapshot {
    #[prost(message, optional, tag = "1")]
    version: Option<Version>,
    #[prost(message, repeated, tag = "2")]
    records: Vec<Record>,
    #[prost(bytes = "vec", optional, tag = "3")]
    mac: Option<Vec<u8>>,
    #[prost(message, optional, tag = "4")]
    key_id: Option<Blob>,
}
#[derive(Message)]
pub(super) struct Version {
    #[prost(uint64, optional, tag = "1")]
    pub(super) value: Option<u64>,
}
#[derive(Message)]
pub(super) struct Blob {
    #[prost(bytes = "vec", optional, tag = "1")]
    value: Option<Vec<u8>>,
}
#[derive(Message)]
pub(super) struct Record {
    #[prost(message, optional, tag = "1")]
    pub(super) index: Option<Blob>,
    #[prost(message, optional, tag = "2")]
    pub(super) value: Option<Blob>,
    #[prost(message, optional, tag = "3")]
    pub(super) key_id: Option<Blob>,
}
pub(super) fn blob(value: Option<Blob>) -> Result<Vec<u8>> {
    value
        .and_then(|v| v.value)
        .ok_or_else(|| invalid("missing snapshot record field"))
}
pub(super) fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N]> {
    value
        .try_into()
        .map_err(|_| invalid("invalid snapshot field width"))
}

/// Structurally decoded snapshot; not trusted until its records and MAC verify.
pub struct CollectionSnapshot {
    version: u64,
    key_id: [u8; 6],
    mac: [u8; 32],
    records: Vec<EncryptedMutation>,
}
impl std::fmt::Debug for CollectionSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CollectionSnapshot(<redacted>)")
    }
}
impl CollectionSnapshot {
    /// Decode an already acquired snapshot protobuf. Local limits precede the
    /// repeated-message allocation; no URL fetch or key lookup occurs here.
    pub fn decode(mut bytes: &[u8]) -> Result<Self> {
        use prost::encoding::{DecodeContext, decode_key, skip_field};
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid("snapshot exceeds local byte bound"));
        }
        let original = bytes;
        let mut count = 0;
        let mut records = 0;
        let mut singletons = BTreeSet::new();
        while !bytes.is_empty() {
            let (tag, wire) =
                decode_key(&mut bytes).map_err(|_| invalid("invalid snapshot protobuf"))?;
            count += 1;
            if tag == 2 {
                records += 1;
            }
            if count > 8192 || records > 4096 {
                return Err(invalid("snapshot exceeds local field bound"));
            }
            if matches!(tag, 1 | 3 | 4) && !singletons.insert(tag) {
                return Err(invalid("ambiguous duplicate snapshot field"));
            }
            skip_field(wire, tag, &mut bytes, DecodeContext::default())
                .map_err(|_| invalid("invalid snapshot protobuf"))?;
        }
        let snapshot =
            WireSnapshot::decode(original).map_err(|_| invalid("invalid snapshot protobuf"))?;
        let version = snapshot
            .version
            .and_then(|v| v.value)
            .ok_or_else(|| invalid("missing snapshot version"))?;
        let key_id = fixed(blob(snapshot.key_id)?)?;
        let mac = fixed(
            snapshot
                .mac
                .ok_or_else(|| invalid("missing snapshot MAC"))?,
        )?;
        let mut records = Vec::new();
        for record in snapshot.records {
            let body = blob(record.value)?;
            if !(64..=1_048_576).contains(&body.len()) {
                return Err(invalid("invalid snapshot record size"));
            }
            records.push(EncryptedMutation {
                operation: MutationOperation::Set,
                key_id: fixed(blob(record.key_id)?)?,
                index_mac: fixed(blob(record.index)?)?,
                body,
            });
        }
        Ok(Self {
            version,
            key_id,
            mac,
            records,
        })
    }
}

/// Verified snapshot candidate and authenticated actions; no durable state change.
#[derive(Debug)]
pub struct VerifiedCollectionSnapshot {
    /// Candidate integrity state.
    pub state: CollectionState,
    /// Authenticated records, not yet applied by action handlers.
    pub mutations: Vec<DecryptedMutation>,
}
impl CollectionState {
    /// Verify a full snapshot against this baseline. Reject rollback and
    /// same-version contradictions. Callers still enforce device/key authority,
    /// snapshot freshness and atomic application before committing the candidate.
    pub fn verify_snapshot<'a>(
        &self,
        snapshot: &CollectionSnapshot,
        mut key: impl FnMut(&[u8; 6]) -> Result<&'a MutationKeys>,
    ) -> Result<VerifiedCollectionSnapshot> {
        if snapshot.version < self.version {
            return Err(invalid("snapshot would roll back collection version"));
        }
        let mut next = CollectionState::empty(&self.collection)?;
        let mut mutations = Vec::new();
        let mut macs = Vec::new();
        for record in &snapshot.records {
            if next.values.contains_key(&record.index_mac) {
                return Err(invalid("duplicate snapshot index"));
            }
            let decoded = key(&record.key_id)?.decrypt_mutation(
                MutationOperation::Set,
                &record.key_id,
                &record.body,
                &record.index_mac,
            )?;
            let mac = fixed(record.body[record.body.len() - 32..].to_vec())?;
            next.values.insert(record.index_mac, mac);
            macs.push(mac);
            mutations.push(decoded);
        }
        next.hash = next.hash.updated(&[], &macs)?;
        key(&snapshot.key_id)?.verify_snapshot(
            next.hash.as_bytes(),
            snapshot.version,
            &self.collection,
            &snapshot.mac,
        )?;
        next.version = snapshot.version;
        if next.version == self.version && next != *self {
            return Err(invalid("snapshot contradicts saved version"));
        }
        Ok(VerifiedCollectionSnapshot {
            state: next,
            mutations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn literal_snapshot() -> Vec<u8> {
        // Independently assembled protobuf tags/lengths around Python/OpenSSL
        // encrypted SET and snapshot-MAC fixtures (version 1, one record).
        let mut bytes = hex::decode("0a0208011282010a220a20").unwrap();
        bytes.extend(
            hex::decode("3d47311df4bc7aa8bd5ef2c16cbe7fc1ea07f18c86a2fb34078d1d416edb22d8")
                .unwrap(),
        );
        bytes.extend(hex::decode("12520a50").unwrap());
        bytes.extend(hex::decode("000102030405060708090a0b0c0d0e0f2192b6a9dde3b0f3712d6c99dc98976b69e1b9f25155fa8ca9e9f0e206925fcc5f5fd8469f020699365e4750632c893bc24096315f6739aaeba497a176823909").unwrap());
        bytes.extend(hex::decode("1a080a060001000000021a20").unwrap());
        bytes.extend(
            hex::decode("0e326b26aa4969d98e2fcf482d2790bf9933a9e11f093b7c18a17f0da491775d")
                .unwrap(),
        );
        bytes.extend(hex::decode("22080a06000100000002").unwrap());
        bytes
    }
    #[test]
    fn snapshot_storage_is_atomic_idempotent_and_rejects_rollback() {
        use super::super::storage::{stage_patch, stage_snapshot};
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("snapshot.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        let snapshot = CollectionSnapshot::decode(&literal_snapshot()).unwrap();
        let rollback: Result<()> = store.write(|tx| {
            assert_eq!(stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys))?, 1);
            Err(Error::Cancelled)
        });
        assert!(rollback.is_err());
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 0);
        store
            .write(|tx| stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys)))
            .unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert_eq!(
            store
                .write(|tx| stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys)))
                .unwrap(),
            1
        );
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 3);
        store
            .write(|tx| {
                stage_patch(tx, "regular", &super::super::tests::patch(true), |_| {
                    Ok(&keys)
                })
            })
            .unwrap();
        assert!(
            store
                .write(|tx| stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys)))
                .is_err()
        );
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 4);
    }

    #[test]
    fn identical_snapshot_repairs_only_missing_records_after_authentication() {
        use super::super::storage::{stage_patch, stage_snapshot};
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("legacy-snapshot.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        let snapshot = CollectionSnapshot::decode(&literal_snapshot()).unwrap();
        let records = format!("app-state-records:{}", hex::encode("regular"));
        let batch = format!("app-state-actions:{}:{:020}", hex::encode("regular"), 1);
        let state = format!("app-state-collection:{}", hex::encode("regular"));
        store
            .write(|tx| {
                stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys))?;
                // Model a legacy store with an already-consumed apply queue.
                assert!(tx.delete(Table::Checkpoints, records.as_bytes())?);
                assert!(tx.delete(Table::Checkpoints, batch.as_bytes())?);
                Ok(())
            })
            .unwrap();
        let baseline = store
            .read(|tx| tx.get(Table::Checkpoints, state.as_bytes(), 1))
            .unwrap();
        let mut tampered = CollectionSnapshot::decode(&literal_snapshot()).unwrap();
        tampered.mac[0] ^= 1;
        assert!(
            store
                .write(|tx| stage_snapshot(tx, "regular", &tampered, |_| Ok(&keys)))
                .is_err()
        );
        assert!(
            store
                .write(
                    |tx| stage_patch(tx, "regular", &super::super::tests::patch(true), |_| Ok(
                        &keys
                    ))
                )
                .is_err()
        );
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 1);
        let rollback: Result<()> = store.write(|tx| {
            stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys))?;
            Err(Error::Cancelled)
        });
        assert!(rollback.is_err());
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 1);
        store
            .write(|tx| stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys)))
            .unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 2);
        assert_eq!(
            store
                .read(|tx| tx.get(Table::Checkpoints, state.as_bytes(), 1))
                .unwrap(),
            baseline
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, batch.as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        store
            .write(|tx| stage_snapshot(tx, "regular", &snapshot, |_| Ok(&keys)))
            .unwrap();
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 2);
        assert_eq!(
            store
                .write(
                    |tx| stage_patch(tx, "regular", &super::super::tests::patch(true), |_| Ok(
                        &keys
                    ))
                )
                .unwrap(),
            2
        );
    }

    #[test]
    fn literal_snapshot_bootstrap_and_rejection() {
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        let bytes = literal_snapshot();
        let snapshot = CollectionSnapshot::decode(&bytes).unwrap();
        let empty = CollectionState::empty("regular").unwrap();
        let verified = empty.verify_snapshot(&snapshot, |_| Ok(&keys)).unwrap();
        assert_eq!(verified.state.version(), 1);
        assert_eq!(verified.state.len(), 1);
        assert_eq!(verified.mutations.len(), 1);
        assert!(empty.is_empty());
        assert_eq!(
            verified
                .state
                .verify_snapshot(&snapshot, |_| Ok(&keys))
                .unwrap()
                .state,
            verified.state
        );
        let mut newer = verified.state.clone();
        newer.version = 2;
        assert!(newer.verify_snapshot(&snapshot, |_| Ok(&keys)).is_err());
        let mut conflict = CollectionState::empty("regular").unwrap();
        conflict.version = 1;
        assert!(conflict.verify_snapshot(&snapshot, |_| Ok(&keys)).is_err());
        assert!(
            CollectionState::empty("critical")
                .unwrap()
                .verify_snapshot(&snapshot, |_| Ok(&keys))
                .is_err()
        );
        let mut corrupt = CollectionSnapshot::decode(&bytes).unwrap();
        corrupt.mac[0] ^= 1;
        assert!(empty.verify_snapshot(&corrupt, |_| Ok(&keys)).is_err());
        let mut duplicate = CollectionSnapshot::decode(&bytes).unwrap();
        duplicate.records.push(
            CollectionSnapshot::decode(&bytes)
                .unwrap()
                .records
                .remove(0),
        );
        assert!(empty.verify_snapshot(&duplicate, |_| Ok(&keys)).is_err());
        assert!(
            empty
                .verify_snapshot(&snapshot, |_| Err(invalid("unauthorized key")))
                .is_err()
        );
    }
    #[test]
    fn structural_limits_and_missing_fields_fail_before_verification() {
        assert!(CollectionSnapshot::decode(&[]).is_err());
        assert!(CollectionSnapshot::decode(b"\x0a\x00").is_err());
        let mut duplicate = literal_snapshot();
        duplicate.extend_from_slice(b"\x0a\x02\x08\x01");
        assert!(CollectionSnapshot::decode(&duplicate).is_err());
        assert!(CollectionSnapshot::decode(&[0x12, 0x00].repeat(4097)).is_err());
        let mut missing = literal_snapshot();
        missing.truncate(missing.len() - 10);
        assert!(CollectionSnapshot::decode(&missing).is_err());
        assert_eq!(
            format!(
                "{:?}",
                CollectionSnapshot::decode(&literal_snapshot()).unwrap()
            ),
            "CollectionSnapshot(<redacted>)"
        );
    }
}
