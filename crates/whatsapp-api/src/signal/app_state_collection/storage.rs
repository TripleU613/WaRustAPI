//! Transactional integrity state and opaque pending actions. No action is applied
//! here, and callers must still resolve only account/device-authorized keys.
use super::*;
use crate::store::{Table, WriteTx};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};
pub mod actions;

#[derive(Serialize, Deserialize)]
struct SavedState {
    collection: String,
    version: u64,
    hash: Vec<u8>,
    values: Vec<([u8; 32], [u8; 32])>,
}

#[derive(Clone, Serialize, Deserialize)]
struct PendingAction {
    operation: u8,
    index: Vec<u8>,
    value: Option<Vec<u8>>,
    schema_version: i32,
}
impl Drop for PendingAction {
    fn drop(&mut self) {
        self.index.zeroize();
        if let Some(value) = &mut self.value {
            value.zeroize();
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PendingBatch {
    previous_version: u64,
    snapshot: bool,
    actions: Vec<PendingAction>,
}

fn pending_actions(mutations: &[DecryptedMutation]) -> Vec<PendingAction> {
    mutations
        .iter()
        .map(|m| PendingAction {
            operation: match m.operation() {
                MutationOperation::Set => 1,
                MutationOperation::Remove => 2,
            },
            index: m.index().to_vec(),
            value: m.value().map(<[u8]>::to_vec),
            schema_version: m.version(),
        })
        .collect()
}

// Current authenticated mutation records are distinct from the pending apply
// queue. APK 0oQ.A04 reads syncd_mutations separately from pending_mutations.
#[derive(Serialize, Deserialize)]
struct CurrentRecords {
    version: u64,
    actions: Vec<PendingAction>,
}
fn records_key(collection: &str) -> Vec<u8> {
    format!("app-state-records:{}", hex::encode(collection)).into_bytes()
}

fn save_current_records(
    tx: &mut WriteTx<'_>,
    collection: &str,
    batch: &PendingBatch,
    version: u64,
) -> Result<()> {
    let mut records = BTreeMap::new();
    if !batch.snapshot && batch.previous_version != 0 {
        let bytes = Zeroizing::new(
            tx.get(Table::Checkpoints, &records_key(collection), 1)?
                .ok_or_else(|| {
                    invalid("missing authenticated records; verified snapshot required")
                })?,
        );
        let previous: CurrentRecords =
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid authenticated records"))?;
        if previous.version != batch.previous_version {
            return Err(invalid("authenticated record version mismatch"));
        }
        for action in previous.actions {
            if action.operation != 1 || records.insert(action.index.clone(), action).is_some() {
                return Err(invalid("invalid or duplicate authenticated record"));
            }
        }
    }
    for action in &batch.actions {
        match action.operation {
            1 => {
                records.insert(action.index.clone(), action.clone());
            }
            2 if !batch.snapshot => {
                if records.remove(&action.index).is_none() {
                    return Err(invalid("missing authenticated record for removal"));
                }
            }
            _ => return Err(invalid("invalid authenticated record operation")),
        }
    }
    let saved = CurrentRecords {
        version,
        actions: records.into_values().collect(),
    };
    let bytes = Zeroizing::new(
        serde_json::to_vec(&saved)
            .map_err(|_| invalid("authenticated record serialization failed"))?,
    );
    tx.put(Table::Checkpoints, &records_key(collection), 1, &bytes)
}

fn state_key(collection: &str) -> Vec<u8> {
    format!("app-state-collection:{}", hex::encode(collection)).into_bytes()
}
fn batch_key(collection: &str, version: u64) -> Vec<u8> {
    format!(
        "app-state-actions:{}:{version:020}",
        hex::encode(collection)
    )
    .into_bytes()
}

fn load(tx: &WriteTx<'_>, collection: &str) -> Result<CollectionState> {
    let empty = CollectionState::empty(collection)?;
    let Some(bytes) = tx.get(Table::Checkpoints, &state_key(collection), 1)? else {
        return Ok(empty);
    };
    let bytes = Zeroizing::new(bytes);
    let saved: SavedState =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid stored collection"))?;
    if saved.collection != collection || saved.hash.len() != 128 {
        return Err(invalid("stored collection identity or hash mismatch"));
    }
    let mut values = BTreeMap::new();
    for (index, value) in saved.values {
        if values.insert(index, value).is_some() {
            return Err(invalid("duplicate stored collection index"));
        }
    }
    Ok(CollectionState {
        collection: saved.collection,
        version: saved.version,
        hash: StateHash::from_bytes(saved.hash.try_into().map_err(|_| invalid("invalid hash"))?),
        values,
    })
}

fn save(
    tx: &mut WriteTx<'_>,
    state: &CollectionState,
    mutations: &[DecryptedMutation],
    snapshot: bool,
    previous_version: u64,
) -> Result<()> {
    let key = batch_key(&state.collection, state.version);
    if tx.get(Table::Checkpoints, &key, 1)?.is_some() {
        return Err(invalid("pending action batch already exists"));
    }
    let batch = PendingBatch {
        previous_version,
        snapshot,
        actions: pending_actions(mutations),
    };
    save_current_records(tx, &state.collection, &batch, state.version)?;
    let saved = SavedState {
        collection: state.collection.clone(),
        version: state.version,
        hash: state.hash.as_bytes().to_vec(),
        values: state
            .values
            .iter()
            .map(|(index, value)| (*index, *value))
            .collect(),
    };
    let batch_bytes = Zeroizing::new(
        serde_json::to_vec(&batch).map_err(|_| invalid("pending action serialization failed"))?,
    );
    let state_bytes = Zeroizing::new(
        serde_json::to_vec(&saved).map_err(|_| invalid("collection serialization failed"))?,
    );
    tx.put(Table::Checkpoints, &key, 1, &batch_bytes)?;
    tx.put(
        Table::Checkpoints,
        &state_key(&state.collection),
        1,
        &state_bytes,
    )
}

/// Authenticate against the current account-scoped baseline and stage state and
/// pending actions together. The surrounding store transaction must commit for
/// either to become durable. A stale patch fails instead of replacing state.
/// Propagate errors to abort the surrounding transaction; do not swallow them.
/// The resolver must authorize keys for this transaction's account and generation;
/// this function does not activate imported keys or send network requests.
pub fn stage_patch<'a>(
    tx: &mut WriteTx<'_>,
    collection: &str,
    patch: &CollectionPatch,
    key: impl FnMut(&[u8; 6]) -> Result<&'a MutationKeys>,
) -> Result<u64> {
    let baseline = load(tx, collection)?;
    let verified = baseline.verify_patch(patch, key)?;
    save(
        tx,
        &verified.state,
        &verified.mutations,
        false,
        baseline.version,
    )?;
    Ok(verified.state.version)
}

/// Stage an authenticated snapshot without rollback or same-version replacement.
/// An identical current snapshot never duplicates pending actions. It may rebuild
/// missing authenticated records for stores created before record retention.
/// Snapshot freshness and device/key authorization remain caller responsibilities.
/// Propagate errors to abort the surrounding transaction; do not swallow them.
pub fn stage_snapshot<'a>(
    tx: &mut WriteTx<'_>,
    collection: &str,
    snapshot: &CollectionSnapshot,
    key: impl FnMut(&[u8; 6]) -> Result<&'a MutationKeys>,
) -> Result<u64> {
    let baseline = load(tx, collection)?;
    let verified = baseline.verify_snapshot(snapshot, key)?;
    if verified.state != baseline {
        save(
            tx,
            &verified.state,
            &verified.mutations,
            true,
            baseline.version,
        )?;
    } else if tx
        .get(Table::Checkpoints, &records_key(collection), 1)?
        .is_none()
    {
        // Verification above authenticated every record and matched the saved
        // integrity state. Rebuild only the missing cache, not the apply queue,
        // applied cursor, projected settings or collection version.
        save_current_records(
            tx,
            collection,
            &PendingBatch {
                previous_version: baseline.version,
                snapshot: true,
                actions: pending_actions(&verified.mutations),
            },
            baseline.version,
        )?;
    }
    Ok(verified.state.version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };

    #[test]
    fn snapshot_record_replacement_is_atomic_and_missing_history_is_not_reset() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("records.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        })
        .unwrap();
        let action = |index: &[u8]| PendingAction {
            operation: 1,
            index: index.to_vec(),
            value: Some(vec![8, 9]),
            schema_version: 5,
        };
        let snapshot = PendingBatch {
            previous_version: 0,
            snapshot: true,
            actions: vec![action(b"old")],
        };
        store
            .write(|tx| save_current_records(tx, "regular", &snapshot, 3))
            .unwrap();
        let replacement = PendingBatch {
            previous_version: 3,
            snapshot: true,
            actions: vec![action(b"new")],
        };
        let result: Result<()> = store.write(|tx| {
            save_current_records(tx, "regular", &replacement, 7)?;
            Err(Error::Cancelled)
        });
        assert!(result.is_err());
        let read = || {
            store
                .read(|tx| {
                    tx.get_json::<CurrentRecords>(Table::Checkpoints, &records_key("regular"), 1)
                })
                .unwrap()
                .unwrap()
        };
        assert_eq!(read().version, 3);
        assert_eq!(read().actions[0].index, b"old");
        store
            .write(|tx| save_current_records(tx, "regular", &replacement, 7))
            .unwrap();
        assert_eq!(read().version, 7);
        assert_eq!(read().actions.len(), 1);
        assert_eq!(read().actions[0].index, b"new");
        let patch = PendingBatch {
            previous_version: 7,
            snapshot: false,
            actions: vec![action(b"later")],
        };
        assert!(
            store
                .write(|tx| save_current_records(tx, "other", &patch, 8))
                .is_err()
        );
        assert_eq!(read().version, 7);
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 1);
    }

    #[test]
    fn commit_rollback_restart_and_stale_patch_preserve_pending_actions() {
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("collections.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let keys = MutationKeys::derive(&(0..32).collect::<Vec<u8>>()).unwrap();
        let patch = super::super::tests::patch(false);
        let rollback: Result<()> = store.write(|tx| {
            stage_patch(tx, "regular", &patch, |_| Ok(&keys))?;
            Err(Error::Cancelled)
        });
        assert!(rollback.is_err());
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 0);
        assert!(
            store
                .write(|tx| stage_patch(tx, "regular", &patch, |_| Err(Error::Cancelled)))
                .is_err()
        );
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 0);
        store
            .write(|tx| stage_patch(tx, "regular", &patch, |_| Ok(&keys)))
            .unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert!(
            store
                .write(|tx| stage_patch(tx, "regular", &patch, |_| Ok(&keys)))
                .is_err()
        );
        let pending: PendingBatch = store
            .read(|tx| tx.get_json(Table::Checkpoints, &batch_key("regular", 1), 1))
            .unwrap()
            .unwrap();
        assert!(!pending.snapshot);
        assert_eq!(pending.actions.len(), 1);
        assert_eq!(pending.actions[0].operation, 1);
        let current: CurrentRecords = store
            .read(|tx| tx.get_json(Table::Checkpoints, &records_key("regular"), 1))
            .unwrap()
            .unwrap();
        assert_eq!(current.version, 1);
        assert_eq!(current.actions.len(), 1);
        assert_eq!(current.actions[0].index, pending.actions[0].index);
        store
            .write(|tx| {
                assert_eq!(load(tx, "regular")?.version(), 1);
                stage_patch(tx, "regular", &super::super::tests::patch(true), |_| {
                    Ok(&keys)
                })?;
                let state = load(tx, "regular")?;
                assert_eq!(state.version(), 2);
                assert!(state.is_empty());
                assert_eq!(state.hash(), &StateHash::default());
                Ok(())
            })
            .unwrap();
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 4);
        let current: CurrentRecords = store
            .read(|tx| tx.get_json(Table::Checkpoints, &records_key("regular"), 1))
            .unwrap()
            .unwrap();
        assert_eq!(current.version, 2);
        assert!(current.actions.is_empty());
    }
}
