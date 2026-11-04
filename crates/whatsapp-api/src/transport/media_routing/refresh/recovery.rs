//! Rebuild known waits in durable receipt order; never replay a request or a reset
//! against an active saved deadline whose origin cannot be proved.
use super::*;
use std::collections::BTreeMap;

/// Local recovery observations, not proof of full native retry eligibility.
#[derive(Debug, Serialize)]
pub struct RecoveryReport {
    /// Validated receipts with known, complete local order.
    pub sequenced_receipts: usize,
    /// Settled prospective receipts whose historical order remains unknown.
    pub unsequenced_receipts: usize,
    /// Older unbound route rows outside this prospective recovery contract.
    pub unbound_records: usize,
    /// Latest still-active bound from unsequenced507 evidence; order is unknown.
    pub unsequenced_active_until: Option<Timestamp>,
    /// Preserved/restored saved deadline. Recovery never shortens this field.
    pub preserved_until: Option<Timestamp>,
}
fn receipt_directive(record: &Record) -> Result<Option<ThrottleDirective>> {
    let reply = super::super::super::actions::decode(
        record
            .wire
            .as_ref()
            .ok_or_else(|| invalid("settled media receipt has no wire"))?,
    )?;
    if reply.description != "iq"
        || reply.get_attr("id").and_then(|v| v.as_text()) != Some(record.action_id.as_str())
    {
        return Err(invalid("historical media receipt correlation mismatch"));
    }
    match (
        record.status.as_str(),
        reply.get_attr("type").and_then(|v| v.as_text()),
    ) {
        ("refused", Some("error")) => Ok(error_directive(&reply)),
        ("result", Some("result")) => {
            effective_reply(
                &serde_json::to_value(record)
                    .map_err(|_| invalid("invalid stored media receipt"))?,
            )?
            .ok_or_else(|| invalid("historical media result lacks context"))?;
            Ok(Some(ThrottleDirective::Clear))
        }
        ("unsupported" | "stale", Some("result")) => Ok(None),
        _ => Err(invalid("historical media receipt status mismatch")),
    }
}
fn deadline(at: Timestamp, seconds: u32) -> Result<Timestamp> {
    at.0.checked_add(i64::from(seconds) * 1000)
        .filter(|value| at.0 > 0 && *value > 0)
        .map(Timestamp)
        .ok_or_else(|| invalid("historical media deadline is invalid"))
}
/// Recover receipt-anchored waits without issuing requests. Older unbound rows
/// are counted, not reinterpreted as prospective request authority. Active
/// unsequenced waits remain ambiguous and prevent prepare from creating a query.
pub(super) fn recover_throttle(store: &ProtocolStore) -> Result<RecoveryReport> {
    store
        .write(|tx| recover_in(tx, Timestamp::now()))
        .map_err(redact)
}
fn recover_in(tx: &mut WriteTx<'_>, now: Timestamp) -> Result<RecoveryReport> {
    let existing = valid_deadline(tx.get_json(Table::Checkpoints, THROTTLE, 1)?)?;
    let counter: Option<u64> = tx.get_json(Table::Checkpoints, RECEIPT_SEQUENCE, 1)?;
    if counter == Some(0) {
        return Err(invalid("invalid media receipt counter"));
    }
    let mut ordered = BTreeMap::new();
    let mut cursor = None;
    let mut unsequenced = 0;
    let mut unbound = 0;
    let mut active = None;
    let mut examined = 0;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        let done = rows.len() < 256;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (row_key, bytes) in rows {
            if !row_key.starts_with(b"media-route:") {
                continue;
            }
            examined += 1;
            if examined > 10000 {
                return Err(invalid("media recovery history exceeds local bound"));
            }
            let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| {
                crate::Error::Storage(crate::store::StoreError::Serialization(
                    "invalid stored media refresh data".into(),
                ))
            })?;
            if value.get("context_version").is_none() {
                unbound += 1;
                continue;
            }
            let record: Record = serde_json::from_value(value).map_err(|_| {
                crate::Error::Storage(crate::store::StoreError::Serialization(
                    "invalid stored media refresh data".into(),
                ))
            })?;
            validate(&record)?;
            if record.generation != tx.generation() || row_key != key(&record.action_id).as_bytes()
            {
                return Err(invalid("historical media context mismatch"));
            }
            if record.status == "outcome_unknown" {
                continue;
            }
            let directive = receipt_directive(&record)?;
            if let Some(sequence) = record.settlement_sequence {
                if ordered.insert(sequence, (directive, record.at)).is_some() {
                    return Err(invalid("duplicate media receipt sequence"));
                }
            } else {
                unsequenced += 1;
                if let Some(ThrottleDirective::Wait(seconds)) = directive {
                    let until = deadline(record.at, seconds)?;
                    if now <= until {
                        active = Some(active.map_or(until, |old: Timestamp| old.max(until)));
                    }
                }
            }
        }
        if done {
            break;
        }
    }
    // A missing middle receipt may contain a wait or reset. Never replay a
    // partial history or use timestamps to fill its order.
    let mut last: u64 = 0;
    let mut replayed = None;
    for (sequence, (directive, at)) in &ordered {
        if last.checked_add(1) != Some(*sequence) {
            return Err(invalid("media receipt history is incomplete"));
        }
        last = *sequence;
        match directive {
            Some(ThrottleDirective::Clear) => replayed = None,
            Some(ThrottleDirective::Wait(seconds)) => replayed = Some(deadline(*at, *seconds)?),
            None => {}
        }
    }
    if counter.unwrap_or(0) != last {
        return Err(invalid("media receipt history and counter disagree"));
    }
    // A standalone saved deadline can predate sequencing. Recovery is not
    // authority to erase/shorten it; only a fresh validated live transition is.
    let preserved = existing.into_iter().chain(replayed).max();
    if preserved != existing
        && let Some(until) = preserved
    {
        tx.put_json(Table::Checkpoints, THROTTLE, 1, &until)?;
    }
    Ok(RecoveryReport {
        sequenced_receipts: ordered.len(),
        unsequenced_receipts: unsequenced,
        unbound_records: unbound,
        unsequenced_active_until: active,
        preserved_until: preserved,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("media-recovery-test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn error(id: &str, code: &str, backoff: &str) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "error")
            .children(vec![
                Node::new("error")
                    .attr("code", code)
                    .attr("backoff", backoff),
            ])
    }
    fn saved(store: &ProtocolStore, id: &str) -> serde_json::Value {
        store
            .read(|tx| tx.get_json(Table::Checkpoints, key(id).as_bytes(), 1))
            .unwrap()
            .unwrap()
    }
    #[test]
    fn lost_wait_is_restored_at_original_receipt_and_preparation_stays_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("restore.redb");
        let store = open(&path);
        prepare(&store, "a").unwrap();
        store
            .write(|tx| settle_in(tx, "a", &error("a", "507", "60")))
            .unwrap();
        let expected = throttle_until(&store).unwrap().unwrap();
        store
            .write(|tx| tx.delete(Table::Checkpoints, THROTTLE))
            .unwrap();
        drop(store);
        let store = open(&path);
        assert!(prepare(&store, "b").is_err());
        assert_eq!(throttle_until(&store).unwrap(), Some(expected));
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, key("b").as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        let before = saved(&store, "a");
        let report = recover_throttle(&store).unwrap();
        assert_eq!(report.sequenced_receipts, 1);
        assert_eq!(report.preserved_until, Some(expected));
        assert_eq!(saved(&store, "a"), before);
    }
    #[test]
    fn sequence_overrides_clock_and_id_order_but_recovery_never_clears_unknown_saved_wait() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("order.redb"));
        prepare(&store, "z").unwrap();
        prepare(&store, "a").unwrap();
        store
            .write(|tx| settle_in(tx, "z", &error("z", "507", "60")))
            .unwrap();
        store
            .write(|tx| settle_in(tx, "a", &error("a", "503", "0")))
            .unwrap();
        let mut older = saved(&store, "z");
        older["at"] = (Timestamp::now().0 + 60000).into();
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key("z").as_bytes(), 1, &older))
            .unwrap();
        assert_eq!(recover_throttle(&store).unwrap().preserved_until, None);
        let unknown = Timestamp(Timestamp::now().0 + 90000);
        store
            .write(|tx| tx.put_json(Table::Checkpoints, THROTTLE, 1, &unknown))
            .unwrap();
        assert_eq!(
            recover_throttle(&store).unwrap().preserved_until,
            Some(unknown)
        );
    }
    #[test]
    fn unsequenced_active_wait_is_reported_without_guessing_order_or_restamping() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("legacy.redb"));
        prepare(&store, "old").unwrap();
        store
            .write(|tx| settle_in(tx, "old", &error("old", "507", "60")))
            .unwrap();
        let until = throttle_until(&store).unwrap().unwrap();
        let mut legacy = saved(&store, "old");
        legacy
            .as_object_mut()
            .unwrap()
            .remove("settlement_sequence");
        store
            .write(|tx| {
                tx.put_json(Table::Checkpoints, key("old").as_bytes(), 1, &legacy)?;
                tx.delete(Table::Checkpoints, RECEIPT_SEQUENCE)?;
                tx.delete(Table::Checkpoints, THROTTLE)?;
                Ok(())
            })
            .unwrap();
        let report = recover_throttle(&store).unwrap();
        assert_eq!(report.unsequenced_receipts, 1);
        assert_eq!(report.unsequenced_active_until, Some(until));
        assert_eq!(throttle_until(&store).unwrap(), None);
        assert!(prepare(&store, "new").is_err());
        assert_eq!(saved(&store, "old"), legacy);
        let expired = store
            .write(|tx| recover_in(tx, Timestamp(until.0 + 1)))
            .unwrap();
        assert_eq!(expired.unsequenced_active_until, None);
    }
    #[test]
    fn incomplete_duplicate_or_unauthentic_history_cannot_partly_repair() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("incomplete.redb"));
        prepare(&store, "a").unwrap();
        prepare(&store, "b").unwrap();
        store
            .write(|tx| settle_in(tx, "a", &error("a", "500", "0")))
            .unwrap();
        store
            .write(|tx| settle_in(tx, "b", &error("b", "507", "60")))
            .unwrap();
        let original = saved(&store, "b");
        let mut duplicate = original.clone();
        duplicate["settlement_sequence"] = 1.into();
        store
            .write(|tx| {
                tx.put_json(Table::Checkpoints, key("b").as_bytes(), 1, &duplicate)?;
                tx.delete(Table::Checkpoints, THROTTLE)?;
                Ok(())
            })
            .unwrap();
        assert!(recover_throttle(&store).is_err());
        assert_eq!(throttle_until(&store).unwrap(), None);
        store
            .write(|tx| {
                tx.put_json(Table::Checkpoints, key("b").as_bytes(), 1, &original)?;
                tx.delete(Table::Checkpoints, key("a").as_bytes())?;
                Ok(())
            })
            .unwrap();
        assert!(recover_throttle(&store).is_err());
        assert_eq!(throttle_until(&store).unwrap(), None);
        store.write(|tx| tx.bump_generation()).unwrap();
        assert!(recover_throttle(&store).is_err());
    }
}
