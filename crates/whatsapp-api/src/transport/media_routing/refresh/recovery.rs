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
    /// Possible active507 bound in a correlated legacy capture; reset order is unknown.
    pub unbound_active_until: Option<Timestamp>,
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
// Legacy producer (transport.rs before prospective integration) saved `at`
// after wait_for(iq,id) and stored the full original response under this key.
// This only recognizes a refusal observation: never recreate its request or
// adopt its routes, and never apply an unsequenced reset to another deadline.
fn legacy_wait(
    row_key: &[u8],
    saved: &serde_json::Value,
    now: Timestamp,
) -> Result<Option<Timestamp>> {
    match saved["status"].as_str() {
        Some("refused") => {}
        Some("result" | "outcome_unknown") => return Ok(None),
        _ => return Err(invalid("unrecognized legacy media capture status")),
    }
    let id = row_key
        .strip_prefix(b"media-route:")
        .and_then(|id| std::str::from_utf8(id).ok())
        .filter(|id| !id.is_empty() && id.len() <= 128)
        .ok_or_else(|| invalid("invalid legacy media capture key"))?;
    let wire: Vec<u8> = serde_json::from_value(saved["wire"].clone()).map_err(|_| {
        crate::Error::Storage(crate::store::StoreError::Serialization(
            "invalid legacy media capture wire".into(),
        ))
    })?;
    let reply = super::super::super::actions::decode(&wire)?;
    if reply.description != "iq"
        || reply.get_attr("id").and_then(|value| value.as_text()) != Some(id)
    {
        return Err(invalid("legacy media capture correlation mismatch"));
    }
    if reply.get_attr("type").and_then(|value| value.as_text()) != Some("error") {
        return Ok(None);
    }
    let Some(ThrottleDirective::Wait(seconds)) = error_directive(&reply) else {
        return Ok(None);
    };
    let at = saved["at"]
        .as_i64()
        .filter(|at| *at > 0)
        .map(Timestamp)
        .ok_or_else(|| invalid("legacy media refusal lacks receipt time"))?;
    let until = deadline(at, seconds)?;
    Ok((now <= until).then_some(until))
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
    let expected = counter.unwrap_or(0);
    let mut seen_blocks = BTreeMap::<u64, u64>::new();
    let mut sequenced: usize = 0;
    let mut latest_directive: Option<(u64, Option<Timestamp>)> = None;
    let mut unsequenced = 0;
    let mut unbound = 0;
    let mut unbound_active = None;
    let mut active = None;
    tx.fold_checkpoints(ROUTE_PREFIX, (), |(), row_key, bytes| {
        let value = parse_route(bytes)?;
        if value.get("context_version").is_none() {
            unbound += 1;
            if let Some(until) = legacy_wait(row_key, &value, now)? {
                unbound_active =
                    Some(unbound_active.map_or(until, |old: Timestamp| old.max(until)));
            }
            return Ok(());
        }
        let record: Record = serde_json::from_value(value).map_err(|_| invalid_route_data())?;
        validate(&record)?;
        if record.generation != tx.generation() || row_key != key(&record.action_id).as_bytes() {
            return Err(invalid("historical media context mismatch"));
        }
        if record.status == "outcome_unknown" {
            return Ok(());
        }
        let directive = receipt_directive(&record)?;
        if let Some(sequence) = record.settlement_sequence {
            // validate() excludes sequence zero. Allocate bits only for observed
            // blocks, so an erroneous large counter cannot size an allocation.
            if sequence > expected {
                return Err(invalid("media receipt history and counter disagree"));
            }
            let index = sequence - 1;
            let block = seen_blocks.entry(index / 64).or_default();
            let bit = 1u64 << (index % 64);
            if *block & bit != 0 {
                return Err(invalid("duplicate media receipt sequence"));
            }
            *block |= bit;
            sequenced += 1;
            if let Some(directive) = directive {
                let until = match directive {
                    ThrottleDirective::Wait(seconds) => Some(deadline(record.at, seconds)?),
                    ThrottleDirective::Clear => None,
                };
                if latest_directive.is_none_or(|(last, _)| sequence > last) {
                    latest_directive = Some((sequence, until));
                }
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
        Ok(())
    })?;
    // Exactly N distinct receipts in 1..=N proves full coverage. A missing
    // receipt could hold a later wait or clear, so publish nothing on a gap.
    if u64::try_from(sequenced) != Ok(expected) {
        return Err(invalid("media receipt history is incomplete"));
    }
    let replayed = latest_directive.and_then(|(_, until)| until);
    // A standalone saved deadline can predate sequencing. Recovery is not
    // authority to erase/shorten it; only a fresh validated live transition is.
    let preserved = existing.into_iter().chain(replayed).max();
    if preserved != existing
        && let Some(until) = preserved
    {
        tx.put_json(Table::Checkpoints, THROTTLE, 1, &until)?;
    }
    Ok(RecoveryReport {
        sequenced_receipts: sequenced,
        unsequenced_receipts: unsequenced,
        unbound_records: unbound,
        unbound_active_until: unbound_active,
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
    fn legacy(store: &ProtocolStore, id: &str, node: &Node, at: i64) -> serde_json::Value {
        let saved = serde_json::json!({"status":"refused","at":at,"wire":encode(node).unwrap()});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &saved))
            .unwrap();
        saved
    }
    #[test]
    fn legacy_correlated_wait_blocks_without_context_reconstruction_or_timer_restart() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("unbound.redb"));
        let at = Timestamp::now().0;
        let original = legacy(&store, "old", &error("old", "507", "60"), at);
        let report = recover_throttle(&store).unwrap();
        assert_eq!(report.unbound_records, 1);
        assert_eq!(report.unbound_active_until, Some(Timestamp(at + 60000)));
        assert_eq!(report.unsequenced_receipts, 0);
        assert_eq!(throttle_until(&store).unwrap(), None);
        assert!(prepare(&store, "new").is_err());
        assert_eq!(saved(&store, "old"), original);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, OWNER, 1))
                .unwrap()
                .is_none()
        );
        let expired = store
            .write(|tx| recover_in(tx, Timestamp(at + 60001)))
            .unwrap();
        assert_eq!(expired.unbound_active_until, None);
        legacy(&store, "reset", &error("reset", "503", "0"), at + 1);
        assert_eq!(
            recover_throttle(&store).unwrap().unbound_active_until,
            Some(Timestamp(at + 60000))
        );
    }
    #[test]
    fn malformed_legacy_capture_cannot_hide_a_wait_or_change_other_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("bad-legacy.redb"));
        let at = Timestamp::now().0;
        for (id, node, time) in [
            ("wrong", error("foreign", "507", "60"), at),
            ("no-time", error("no-time", "507", "60"), 0),
        ] {
            legacy(&store, id, &node, time);
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert!(recover_throttle(&store).is_err());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
            store
                .write(|tx| tx.delete(Table::Checkpoints, key(id).as_bytes()))
                .unwrap();
        }
        let mut unknown = legacy(&store, "unknown", &error("unknown", "507", "60"), at);
        unknown["status"] = "unknown-format".into();
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key("unknown").as_bytes(), 1, &unknown))
            .unwrap();
        assert!(recover_throttle(&store).is_err());
        store
            .write(|tx| tx.delete(Table::Checkpoints, key("unknown").as_bytes()))
            .unwrap();
        let result = Node::new("iq")
            .attr("id", "result")
            .attr("type", "result")
            .children(vec![
                Node::new("error").attr("code", "507").attr("backoff", "60"),
            ]);
        legacy(&store, "result", &result, at);
        assert_eq!(recover_throttle(&store).unwrap().unbound_active_until, None);
    }

    #[test]
    fn complete_large_route_history_restores_latest_ordered_wait_and_refuses_a_gap() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large-receipts.redb");
        let store = open(&path);
        const COUNT: u64 = 10_001;
        let at = Timestamp::now();
        let expected = deadline(at, 3600).unwrap();
        store
            .write(|tx| {
                tx.put_json(Table::Checkpoints, RECEIPT_SEQUENCE, 1, &COUNT)?;
                tx.put_json(Table::Checkpoints, OWNER, 1, &true)?;
                for sequence in 1..=COUNT {
                    // Reverse lexical order so the newest wait is visited first.
                    let id = format!("fixture-{:05}", COUNT - sequence);
                    let request_wire = encode(&request(&id, None)?)?;
                    let code = if sequence == COUNT { "507" } else { "503" };
                    let record = Record {
                        context_version: 1,
                        generation: tx.generation(),
                        action_id: id.clone(),
                        requested_at: at,
                        at,
                        status: "refused".into(),
                        settlement_sequence: Some(sequence),
                        request_digest: Sha256::digest(&request_wire).into(),
                        request_wire,
                        prior: None,
                        wire: Some(encode(&error(&id, code, "3600"))?),
                        effective_wire: None,
                    };
                    tx.put_json(Table::Checkpoints, key(&id).as_bytes(), 1, &record)?;
                }
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = open(&path);
        let report = recover_throttle(&store).unwrap();
        assert_eq!(report.sequenced_receipts as u64, COUNT);
        assert_eq!(report.preserved_until, Some(expected));
        store
            .write(|tx| {
                tx.delete(Table::Checkpoints, THROTTLE)?;
                tx.delete(Table::Checkpoints, key("fixture-05000").as_bytes())
            })
            .unwrap();
        assert!(recover_throttle(&store).is_err());
        assert_eq!(throttle_until(&store).unwrap(), None);
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
