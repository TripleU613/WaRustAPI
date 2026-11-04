//! Prospective held-version fetch intents with generation-authenticated local
//! provenance. No result verification, state advancement or flight lease.
//! See docs/protocol/app-state-held-fetch-admission-design.md.
use super::Collection;
use crate::{
    Error, Result,
    model::{ids::AccountGeneration, time::Timestamp},
    signal::app_state_collection::{CollectionState, SavedCollectionState},
    store::{ProtocolStore, Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PREFIX: &str = "app-state-held-fetch-admission:";
const SCHEMA: u16 = 1;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: u16,
    generation: AccountGeneration,
    action_id: String,
    request_digest: [u8; 32],
    baselines: Vec<SavedCollectionState>,
    phase: Phase,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
enum Phase {
    Queued,
    Admitted { reserved_at: Timestamp },
    LocallyRefused { reason: Refusal },
}
#[derive(Serialize, Deserialize)]
enum Refusal {
    BaselineChanged,
}

/// Redacted local admission status; not a socket/server outcome.
#[derive(Debug, Serialize)]
pub struct AdmissionInspection {
    /// Private local record format version.
    pub schema_version: u16,
    /// Generation under which the admission record was authenticated.
    pub generation: AccountGeneration,
    /// `queued`, `admitted` or `locally_refused`.
    pub phase: &'static str,
    /// Action status from the same consistent store read as the phase.
    pub status: super::super::actions::ActionStatus,
    /// Preserved numeric server error code, without arbitrary server text.
    pub error_code: Option<u16>,
    /// Durable local receipt order, absent before settlement.
    pub settlement_sequence: Option<u64>,
    /// Static local refusal reason, never a fabricated server error.
    pub local_reason: Option<&'static str>,
    /// Whether the immutable stored request still matches its sealed fingerprint.
    pub request_digest_matches: bool,
    /// Known collection/version claims only; no hashes, maps or record keys.
    pub collections: Vec<CollectionAdmissionInspection>,
}
/// A known collection and its admitted held-version claim.
#[derive(Debug, Serialize)]
pub struct CollectionAdmissionInspection {
    /// Known static collection name.
    pub collection: &'static str,
    /// Frozen held version, never an advertised dirty version.
    pub version: u64,
}

fn invalid(reason: &'static str) -> Error {
    Error::invalid("held_fetch", reason)
}
fn redact_serialization(error: Error) -> Error {
    match error {
        Error::Storage(crate::store::StoreError::Serialization(_)) => Error::Storage(
            crate::store::StoreError::Serialization("invalid stored held-fetch data".into()),
        ),
        other => other,
    }
}
fn key(id: &str) -> String {
    format!("{PREFIX}{id}")
}
fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn validate_record(
    record: &Record,
    id: &str,
    generation: AccountGeneration,
) -> Result<Vec<Collection>> {
    if record.schema_version != SCHEMA
        || record.generation != generation
        || record.action_id != id
        || record.baselines.is_empty()
        || record.baselines.len() > super::ALL_COLLECTIONS.len()
    {
        return Err(invalid(
            "invalid admission record schema, generation or identity",
        ));
    }
    let mut names = std::collections::BTreeSet::new();
    let mut collections = Vec::new();
    for baseline in &record.baselines {
        let collection = Collection::from_name(&baseline.collection)
            .ok_or_else(|| invalid("admission names no known collection"))?;
        if !names.insert(collection.name()) || baseline.version > i64::MAX as u64 {
            return Err(invalid("duplicate or out-of-range admission baseline"));
        }
        let state = CollectionState::from_saved(baseline)?;
        if !state.has_logical_indexes() {
            return Err(invalid("admission baseline lacks logical provenance"));
        }
        collections.push(collection);
    }
    Ok(collections)
}

fn validate_wire(record: &Record, raw_wire: &[u8], collections: &[Collection]) -> Result<()> {
    if raw_wire.len() > 16 * 1024 * 1024 || digest(raw_wire) != record.request_digest {
        return Err(invalid(
            "stored request differs from its admission fingerprint",
        ));
    }
    let request = super::super::actions::decode(raw_wire)?;
    let versions: Vec<_> = collections
        .iter()
        .zip(&record.baselines)
        .map(|(collection, baseline)| (*collection, Some(baseline.version)))
        .collect();
    // Build only a comparison value. The actual stored/sent request is never
    // changed or re-encoded, and this comparison cannot rebase a stale intent.
    let expected = super::fetch_request_at_versions(&record.action_id, &versions, false)?;
    if request != expected {
        return Err(invalid(
            "stored request is not the sealed held-version intent",
        ));
    }
    Ok(())
}

fn pending_in(tx: &WriteTx<'_>, collections: &[Collection]) -> Result<bool> {
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if rows.is_empty() {
            return Ok(false);
        }
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in &rows {
            if !key.starts_with(super::PENDING_PREFIX.as_bytes()) {
                continue;
            }
            let pending: super::PendingState = tx
                .get_json(Table::Checkpoints, key, 1)?
                .ok_or_else(|| invalid("pending write disappeared"))?;
            if collections
                .iter()
                .any(|c| c.name() == pending.prepared().collection)
            {
                return Ok(true);
            }
        }
    }
}

/// Queue a new explicit incremental fetch from existing authenticated held rows.
/// The action and its admission provenance commit together. Missing checkpoints,
/// incomplete legacy provenance, pending writes and server waits do not create
/// an intent. No key, held state or historical request is modified.
pub fn queue(store: &ProtocolStore, collections: &[Collection]) -> Result<String> {
    store
        .write(|tx| queue_in(tx, collections))
        .map_err(redact_serialization)
}

fn queue_in(tx: &mut WriteTx<'_>, collections: &[Collection]) -> Result<String> {
    if collections.is_empty() || collections.len() > super::ALL_COLLECTIONS.len() {
        return Err(invalid(
            "held fetch needs a bounded nonempty collection list",
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    if collections.iter().any(|c| !unique.insert(c.name())) {
        return Err(invalid("duplicate requested collection"));
    }
    super::require_server_ready_in(tx)?;
    let mut baselines = Vec::new();
    for collection in collections {
        super::ensure_no_pending_collection_in(tx, *collection)?;
        let saved: SavedCollectionState = tx
            .get_json(
                Table::Checkpoints,
                super::held_key(*collection).as_bytes(),
                1,
            )?
            .ok_or_else(|| invalid("held fetch requires an existing collection checkpoint"))?;
        if saved.collection != collection.name() {
            return Err(invalid("held checkpoint names a different collection"));
        }
        let state = CollectionState::from_saved(&saved)?;
        if !state.has_logical_indexes() {
            return Err(invalid(
                "held checkpoint needs authenticated logical-index reconstruction",
            ));
        }
        if saved.version > i64::MAX as u64 {
            return Err(invalid("held version exceeds native signed-long range"));
        }
        baselines.push(state.to_saved());
    }
    let versions: Vec<_> = collections
        .iter()
        .zip(&baselines)
        .map(|(c, b)| (*c, Some(b.version)))
        .collect();
    let id = super::super::actions::queue_in(
        tx,
        super::HELD_FETCH_ACTION_KIND,
        "held app-state fetch",
        |id| super::fetch_request_at_versions(id, &versions, false),
    )?;
    let raw = super::super::actions::request_wire_in_write(tx, &id)?
        .ok_or_else(|| invalid("newly queued held fetch has no stored wire"))?;
    let record = Record {
        schema_version: SCHEMA,
        generation: tx.generation(),
        action_id: id.clone(),
        request_digest: digest(&raw),
        baselines,
        phase: Phase::Queued,
    };
    let names = validate_record(&record, &id, tx.generation())?;
    validate_wire(&record, &raw, &names)?;
    tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &record)?;
    Ok(id)
}

/// Recheck and mark prospective provenance in the action reservation transaction.
/// The caller must mark the matching queued action Sent in this SAME transaction
/// when true. False permits bulk scans to commit a local refusal and continue.
/// A selected caller converting false to Err rolls this transaction back,
/// including its diagnostic; it must not claim that refusal persisted.
pub(crate) fn reserve_in(tx: &mut WriteTx<'_>, id: &str, raw_wire: &[u8]) -> Result<bool> {
    reserve_impl_in(tx, id, raw_wire).map_err(redact_serialization)
}

fn reserve_impl_in(tx: &mut WriteTx<'_>, id: &str, raw_wire: &[u8]) -> Result<bool> {
    let mut record: Record = tx
        .get_json(Table::Checkpoints, key(id).as_bytes(), 1)?
        .ok_or_else(|| invalid("held-fetch action has no admission record"))?;
    let collections = validate_record(&record, id, tx.generation())?;
    validate_wire(&record, raw_wire, &collections)?;
    match record.phase {
        Phase::LocallyRefused { .. } => return Ok(false),
        Phase::Admitted { .. } => {
            return Err(invalid("held fetch was already admitted; no replay"));
        }
        Phase::Queued => {}
    }
    for (collection, baseline) in collections.iter().zip(&record.baselines) {
        let current: Option<SavedCollectionState> = tx.get_json(
            Table::Checkpoints,
            super::held_key(*collection).as_bytes(),
            1,
        )?;
        let unchanged = match current {
            Some(saved) if saved.collection == collection.name() => {
                let state = CollectionState::from_saved(&saved)?;
                state.has_logical_indexes() && state.to_saved() == *baseline
            }
            Some(_) => return Err(invalid("held checkpoint names a different collection")),
            None => false,
        };
        if !unchanged {
            record.phase = Phase::LocallyRefused {
                reason: Refusal::BaselineChanged,
            };
            tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &record)?;
            // This helper cannot know whether its caller will commit: selected
            // refusal or a later failure can still roll back the diagnostic.
            if id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                tracing::warn!(
                    action_id = id,
                    "held fetch not reserved: baseline changed; inspect actions --show; local refusal persists only if the transaction commits"
                );
            } else {
                tracing::warn!(
                    "held fetch not reserved: baseline changed; inspect actions --show; local refusal persists only if the transaction commits"
                );
            }
            return Ok(false);
        }
    }
    if super::server_waiting_in(tx)? || pending_in(tx, &collections)? {
        return Ok(false);
    }
    record.phase = Phase::Admitted {
        reserved_at: Timestamp::now(),
    };
    tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &record)?;
    Ok(true)
}

/// Inspect sealed provenance without printing raw IDs, digests, maps or keys.
/// Legacy fetches have no record and return None; a new-kind action missing its
/// mandatory record is an integrity error, not a retrospective migration.
pub fn inspect(store: &ProtocolStore, id: &str) -> Result<Option<AdmissionInspection>> {
    store
        .read(|tx| {
            let record: Option<Record> = tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1)?;
            let action = super::super::actions::view_in(tx, id)?;
            let Some(record) = record else {
                if action
                    .as_ref()
                    .is_some_and(|(outcome, _)| outcome.kind == super::HELD_FETCH_ACTION_KIND)
                {
                    return Err(invalid("held-fetch action has no admission record"));
                }
                return Ok(None);
            };
            let collections = validate_record(&record, id, tx.generation())?;
            let (outcome, _) = action.ok_or_else(|| invalid("admission has no original action"))?;
            if outcome.kind != super::HELD_FETCH_ACTION_KIND {
                return Err(invalid("admission names a different action kind"));
            }
            use super::super::actions::ActionStatus;
            let (phase, local_reason) = match record.phase {
                Phase::Queued if outcome.status == ActionStatus::Queued => ("queued", None),
                Phase::Admitted { .. } if outcome.status != ActionStatus::Queued => {
                    ("admitted", None)
                }
                Phase::LocallyRefused { .. } if outcome.status == ActionStatus::Queued => {
                    ("locally_refused", Some("baseline_changed"))
                }
                _ => return Err(invalid("action status and admission phase disagree")),
            };
            let raw = super::super::actions::request_wire_in(tx, id)?
                .ok_or_else(|| invalid("admission has no original wire"))?;
            Ok(Some(AdmissionInspection {
                schema_version: record.schema_version,
                generation: record.generation,
                phase,
                status: outcome.status,
                error_code: outcome.error_code,
                settlement_sequence: outcome.settlement_sequence,
                local_reason,
                request_digest_matches: digest(&raw) == record.request_digest,
                collections: collections
                    .iter()
                    .zip(&record.baselines)
                    .map(|(c, b)| CollectionAdmissionInspection {
                        collection: c.name(),
                        version: b.version,
                    })
                    .collect(),
            }))
        })
        .map_err(redact_serialization)
}

#[cfg(test)]
mod tests {
    use super::super::super::actions;
    use super::*;
    use crate::{
        binary::Node,
        model::ids::AccountId,
        signal::app_state_hash::StateHash,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use std::{path::Path, sync::Arc};

    fn open(path: &Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("held-fetch-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([61; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn empty(version: u64) -> SavedCollectionState {
        let mut saved = CollectionState::empty("regular").unwrap().to_saved();
        saved.version = version;
        saved
    }
    fn populated() -> SavedCollectionState {
        // Synthetic already-held state; this test does not claim these MACs
        // came from a native packet or authenticate new mutation material.
        let mut saved = empty(7);
        saved.values = vec![(vec![2; 32], vec![3; 32])];
        saved.hash = StateHash::default()
            .updated(&[], &[[3; 32]])
            .unwrap()
            .as_bytes()
            .to_vec();
        saved.logical_indexes = Some(vec![(br#"["x"]"#.to_vec(), vec![2; 32])]);
        saved
    }
    fn save(store: &ProtocolStore, saved: &SavedCollectionState) {
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    super::super::held_key(Collection::Regular).as_bytes(),
                    1,
                    saved,
                )
            })
            .unwrap();
    }
    fn status(store: &ProtocolStore, id: &str) -> actions::ActionStatus {
        actions::outcomes(store)
            .unwrap()
            .into_iter()
            .find(|o| o.id == id)
            .unwrap()
            .status
    }
    fn raw(store: &ProtocolStore, id: &str) -> Vec<u8> {
        store
            .read(|tx| actions::request_wire_in(tx, id))
            .unwrap()
            .unwrap()
    }
    fn ping(id: &str) -> Result<Node> {
        Ok(Node::new("iq")
            .attr("id", id)
            .attr("type", "get")
            .attr("xmlns", "w:p")
            .attr("to", "s.whatsapp.net"))
    }

    #[test]
    fn queue_and_admission_survive_reopen_without_reencoding_or_replay() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("held.redb");
        let store = open(&path);
        let baseline = populated();
        save(&store, &baseline);
        let id = queue(&store, &[Collection::Regular]).unwrap();
        let wire = raw(&store, &id);
        let request = actions::request(&store, &id).unwrap().unwrap();
        let collection = request.child("sync").unwrap().child("collection").unwrap();
        assert_eq!(collection.get_attr("version").unwrap().as_text(), Some("7"));
        assert!(collection.get_attr("return_snapshot").is_none());
        assert_eq!(inspect(&store, &id).unwrap().unwrap().phase, "queued");
        drop(store);
        let store = open(&path);
        let reserved = actions::reserve_next(&store).unwrap().unwrap();
        assert_eq!(reserved, request);
        assert_eq!(status(&store, &id), actions::ActionStatus::Sent);
        let info = inspect(&store, &id).unwrap().unwrap();
        assert_eq!(info.phase, "admitted");
        assert!(info.request_digest_matches);
        assert_eq!(raw(&store, &id), wire);
        let held: SavedCollectionState = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    super::super::held_key(Collection::Regular).as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(held, baseline);
        drop(store);
        let store = open(&path);
        assert!(actions::reserve_next(&store).unwrap().is_none());
        assert_eq!(inspect(&store, &id).unwrap().unwrap().phase, "admitted");
    }

    #[test]
    fn factory_rollback_leaves_neither_action_nor_admission() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("rollback.redb"));
        save(&store, &empty(0));
        let mut id = None;
        let result: Result<()> = store.write(|tx| {
            id = Some(queue_in(tx, &[Collection::Regular])?);
            Err(invalid("synthetic transaction cut"))
        });
        assert!(result.is_err());
        assert!(actions::outcomes(&store).unwrap().is_empty());
        assert!(inspect(&store, id.as_ref().unwrap()).unwrap().is_none());
    }

    #[test]
    fn reservation_transaction_cut_rolls_back_both_admission_and_sent() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("reservation-cut.redb"));
        save(&store, &empty(0));
        let id = queue(&store, &[Collection::Regular]).unwrap();
        let result: Result<()> = store.write(|tx| {
            actions::reserve_one_in(tx, &id)?;
            Err(invalid("synthetic cut after reservation"))
        });
        assert!(result.is_err());
        assert_eq!(status(&store, &id), actions::ActionStatus::Queued);
        assert_eq!(inspect(&store, &id).unwrap().unwrap().phase, "queued");
        assert!(actions::reserve_next(&store).unwrap().is_some());
    }

    #[test]
    fn missing_or_invalid_legacy_provenance_cannot_create_intent() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("provenance.redb"));
        assert!(queue(&store, &[Collection::Regular]).is_err());
        let mut legacy = populated();
        legacy.logical_indexes = None;
        save(&store, &legacy);
        assert!(queue(&store, &[Collection::Regular]).is_err());
        let mut corrupt = populated();
        corrupt.hash[0] ^= 1;
        save(&store, &corrupt);
        assert!(queue(&store, &[Collection::Regular]).is_err());
        assert!(actions::outcomes(&store).unwrap().is_empty());
        let mut legacy_empty = empty(9);
        legacy_empty.logical_indexes = None;
        save(&store, &legacy_empty);
        let id = queue(&store, &[Collection::Regular]).unwrap();
        assert_eq!(
            inspect(&store, &id).unwrap().unwrap().collections[0].version,
            9
        );
        assert!(actions::reserve_next(&store).unwrap().is_some());
    }

    #[test]
    fn stale_full_baseline_is_permanent_local_refusal_not_server_error() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("stale.redb"));
        let original = populated();
        save(&store, &original);
        let id = queue(&store, &[Collection::Regular]).unwrap();
        let wire = raw(&store, &id);
        let mut changed = original.clone();
        changed.logical_indexes.as_mut().unwrap()[0].0 = br#"["y"]"#.to_vec();
        save(&store, &changed);
        let control = actions::queue(&store, "ping", "control", ping).unwrap();
        let reserved = actions::reserve_pending(&store).unwrap();
        assert_eq!(reserved.len(), 1);
        assert_eq!(
            reserved[0].get_attr("id").unwrap().as_text(),
            Some(control.as_str())
        );
        let info = inspect(&store, &id).unwrap().unwrap();
        assert_eq!(info.phase, "locally_refused");
        assert_eq!(info.local_reason, Some("baseline_changed"));
        assert_eq!(status(&store, &id), actions::ActionStatus::Queued);
        assert_eq!(raw(&store, &id), wire);
        save(&store, &original);
        assert!(
            actions::reserve_next(&store).unwrap().is_none(),
            "old intent is not rebased or revived"
        );
        let fresh = queue(&store, &[Collection::Regular]).unwrap();
        assert_ne!(fresh, id);
        assert!(actions::reserve_next(&store).unwrap().is_some());
    }

    #[test]
    fn pending_write_and_server_wait_recheck_leave_intent_queued() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("wait.redb"));
        let baseline = empty(3);
        save(&store, &baseline);
        let id = queue(&store, &[Collection::Regular]).unwrap();
        let pending = super::super::pending_key("synthetic-pending");
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    pending.as_bytes(),
                    1,
                    &super::super::PendingState::Legacy(baseline),
                )
            })
            .unwrap();
        assert!(actions::reserve_next(&store).unwrap().is_none());
        assert_eq!(inspect(&store, &id).unwrap().unwrap().phase, "queued");
        store
            .write(|tx| {
                tx.delete(Table::Checkpoints, pending.as_bytes())?;
                tx.put_json(
                    Table::Checkpoints,
                    super::super::SERVER_BACKOFF,
                    1,
                    &Timestamp(Timestamp::now().0 + 60_000),
                )
            })
            .unwrap();
        assert!(actions::reserve_next(&store).unwrap().is_none());
        assert_eq!(status(&store, &id), actions::ActionStatus::Queued);
        assert!(queue(&store, &[Collection::Regular]).is_err());
        store
            .write(|tx| tx.delete(Table::Checkpoints, super::super::SERVER_BACKOFF))
            .unwrap();
        assert!(actions::reserve_next(&store).unwrap().is_some());
    }

    #[test]
    fn selected_refusal_rolls_back_diagnostic_and_bulk_persists_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("selected.redb"));
        save(&store, &empty(1));
        let id = queue(&store, &[Collection::Regular]).unwrap();
        save(&store, &empty(2));
        assert!(store.write(|tx| actions::reserve_one_in(tx, &id)).is_err());
        assert_eq!(inspect(&store, &id).unwrap().unwrap().phase, "queued");
        assert!(actions::reserve_next(&store).unwrap().is_none());
        assert_eq!(
            inspect(&store, &id).unwrap().unwrap().phase,
            "locally_refused"
        );
    }

    #[test]
    fn metadata_integrity_faults_do_not_become_missing_or_ready() {
        for mode in ["missing", "digest", "generation", "schema", "admitted"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("corrupt.redb"));
            save(&store, &empty(1));
            let id = queue(&store, &[Collection::Regular]).unwrap();
            store
                .write(|tx| {
                    if mode == "missing" {
                        tx.delete(Table::Checkpoints, key(&id).as_bytes())?;
                        return Ok(());
                    }
                    let mut record: Record = tx
                        .get_json(Table::Checkpoints, key(&id).as_bytes(), 1)?
                        .unwrap();
                    match mode {
                        "digest" => record.request_digest[0] ^= 1,
                        "generation" => record.generation = AccountGeneration(99),
                        "schema" => record.schema_version = 99,
                        "admitted" => {
                            record.phase = Phase::Admitted {
                                reserved_at: Timestamp::now(),
                            }
                        }
                        _ => unreachable!(),
                    }
                    tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &record)
                })
                .unwrap();
            assert!(actions::reserve_next(&store).is_err(), "{mode}");
            assert_eq!(status(&store, &id), actions::ActionStatus::Queued);
            if mode == "digest" {
                assert!(
                    !inspect(&store, &id)
                        .unwrap()
                        .unwrap()
                        .request_digest_matches
                );
            } else {
                assert!(inspect(&store, &id).is_err(), "{mode}");
            }
        }
    }

    #[test]
    fn account_generation_fence_is_an_authentication_error() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("generation.redb"));
        save(&store, &empty(1));
        let id = queue(&store, &[Collection::Regular]).unwrap();
        store.write(|tx| tx.bump_generation()).unwrap();
        assert!(matches!(
            inspect(&store, &id),
            Err(Error::Storage(
                crate::store::StoreError::AuthenticationFailed
            ))
        ));
        assert!(queue(&store, &[Collection::Regular]).is_err());
    }

    #[test]
    fn legacy_fetches_are_not_given_retrospective_admission_and_debug_is_redacted() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("legacy.redb"));
        save(&store, &empty(12));
        let old = super::super::queue_fetch(&store).unwrap();
        let old_wire = raw(&store, &old);
        assert!(inspect(&store, &old).unwrap().is_none());
        let fresh = queue(&store, &[Collection::Regular]).unwrap();
        let info = inspect(&store, &fresh).unwrap().unwrap();
        let debug = format!("{info:?}");
        assert!(!debug.contains(&fresh));
        assert!(!debug.contains("request_digest:"));
        assert!(!debug.contains("logical_indexes"));
        assert_eq!(raw(&store, &old), old_wire);
        assert!(inspect(&store, &old).unwrap().is_none());
    }

    #[test]
    fn inspection_includes_settlement_metadata_without_error_text() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("settled-inspection.redb"));
        save(&store, &empty(1));
        let id = queue(&store, &[Collection::Regular]).unwrap();
        actions::reserve_next(&store).unwrap().unwrap();
        actions::record_reply(
            &store,
            &Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "error")
                .children(vec![
                    Node::new("error")
                        .attr("code", "503")
                        .attr("text", "private-server-detail"),
                ]),
        )
        .unwrap();
        let inspected = inspect(&store, &id).unwrap().unwrap();
        assert_eq!(inspected.phase, "admitted");
        assert_eq!(inspected.status, actions::ActionStatus::Error);
        assert_eq!(inspected.error_code, Some(503));
        assert!(inspected.settlement_sequence.is_some());
        let printed = format!(
            "{inspected:?} {}",
            serde_json::to_string(&inspected).unwrap()
        );
        assert!(!printed.contains("private-server-detail"));
        assert!(!printed.contains(&id));
    }

    #[test]
    fn serialization_faults_keep_storage_type_without_stored_plaintext() {
        fn redacted(error: Error) {
            assert!(matches!(
                &error,
                Error::Storage(crate::store::StoreError::Serialization(_))
            ));
            let printed = format!("{error} {error:?}");
            assert!(!printed.contains("private-phase-detail"));
            assert!(!printed.contains("private-version-detail"));
            assert!(printed.contains("invalid stored held-fetch data"));
        }
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("redacted-fault.redb"));
        save(&store, &empty(1));
        let id = queue(&store, &[Collection::Regular]).unwrap();
        let wire = raw(&store, &id);
        store
            .write(|tx| {
                let record: Record = tx
                    .get_json(Table::Checkpoints, key(&id).as_bytes(), 1)?
                    .unwrap();
                let mut malformed = serde_json::to_value(record).unwrap();
                malformed["phase"] = serde_json::json!({"state": "private-phase-detail"});
                tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &malformed)
            })
            .unwrap();
        redacted(inspect(&store, &id).unwrap_err());
        redacted(store.write(|tx| reserve_in(tx, &id, &wire)).unwrap_err());
        // The factory also reads caller-editable stored state. A wrong JSON
        // type must not echo its string through serde's original diagnostic.
        store
            .write(|tx| {
                let mut malformed = serde_json::to_value(empty(1)).unwrap();
                malformed["version"] = serde_json::json!("private-version-detail");
                tx.put_json(
                    Table::Checkpoints,
                    super::super::held_key(Collection::Regular).as_bytes(),
                    1,
                    &malformed,
                )
            })
            .unwrap();
        redacted(queue(&store, &[Collection::Regular]).unwrap_err());
    }
}
