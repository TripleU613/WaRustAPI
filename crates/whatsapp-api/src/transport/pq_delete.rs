//! Native PQ prekey deletion (`DeleteKyberPreKeysJob`).
//!
//! APK 2.26.43.72: `FKS.onRandomizedDailyCronNoMessageStore` checks Boolean
//! property **21198** (`C33998Ex8.A04`). When it is false and local Kyber rows
//! still exist (`A0g(A02).A0y()`), it schedules `DeleteKyberPreKeysJob`, which
//! sends `C34869FUb.A01(deleteLegacy=false, deletePq=true)`:
//! `<iq xmlns="encrypt" type="set"><op mode="delete"/><pq_list/></iq>`.
//! On completion `C32659EaO.A00` deletes all `kyber_prekeys`, clears PQ upload
//! bookkeeping and PQ sessions.
//!
//! This module reproduces the request, the local KEM wipe and the trigger. It
//! does **not** delete saved PQ sessions or the `pq_session_upgrade_device_state`
//! equivalents (not modelled here), and it wipes only after a definite server
//! result, not after a terminal refusal: the native code also wipes on a
//! terminal 4xx, but the server deletion is then unconfirmed, so rows are kept
//! and the next trigger retries. That difference is deliberate and local.
//!
//! Deletion is idempotent; the settled 21198 reply is the only automatic
//! trigger, matching the daily cron. An uncertain prekey upload blocks the
//! queue locally so reservation fingerprints cannot dangle.

use super::actions;
use crate::{
    Error, Result,
    store::{ProtocolStore, ReadTx, Table},
};
use serde::{Deserialize, Serialize};

const MARKER: &[u8] = b"pq-delete:v1";

fn invalid(reason: &'static str) -> Error {
    Error::invalid("pq_delete", reason)
}

#[derive(Serialize, Deserialize)]
struct Saved {
    id: String,
    /// Local rows removed after a definite result.
    #[serde(default)]
    deleted: Option<usize>,
    /// A definite refusal was recorded; rows were kept.
    #[serde(default)]
    refused: bool,
}

/// A queued delete request and the inventory it will remove.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedDelete {
    /// Durable action id; `listen` transmits it.
    pub action_id: String,
    /// Local KEM rows present when it was queued.
    pub rows: usize,
}

/// Queue one native PQ delete. Refuses when nothing is stored, when the same
/// request is queued or uncertain, or when a prekey upload is unresolved.
pub fn queue(store: &ProtocolStore) -> Result<QueuedDelete> {
    if super::prekeys::summary(store)?.uncertain > 0 {
        return Err(invalid(
            "a prekey upload is unresolved; refusing to delete its inventory",
        ));
    }
    store.write(|tx| queue_in(tx, true)?.ok_or_else(|| invalid("no local KEM prekeys to delete")))
}

/// Read-only form of the automatic trigger: a settled A/B reply leaves 21198
/// off, local KEM rows exist, and no prekey upload is unresolved. Does not
/// queue anything.
pub fn automatic_trigger_ready(store: &ProtocolStore) -> Result<bool> {
    let disabled =
        store.read(|tx| {
            Ok(super::abprops::settled_reply_in(tx)?
                && !super::abprops::pq_keys_upload_enabled_in(tx)?)
        })?;
    if !disabled {
        return Ok(false);
    }
    if super::prekeys::summary(store)?.uncertain > 0 {
        return Ok(false);
    }
    Ok(!store
        .read(|tx| super::kem_prekeys::all_rows(tx))?
        .is_empty())
}

/// Automatic trigger for the daily-cron condition: a settled A/B reply leaves
/// 21198 off while local KEM rows exist. Returns the queued request, if any.
pub fn queue_if_disabled(store: &ProtocolStore) -> Result<Option<QueuedDelete>> {
    let disabled =
        store.read(|tx| {
            Ok(super::abprops::settled_reply_in(tx)?
                && !super::abprops::pq_keys_upload_enabled_in(tx)?)
        })?;
    if !disabled {
        return Ok(None);
    }
    if super::prekeys::summary(store)?.uncertain > 0 {
        return Ok(None);
    }
    store.write(|tx| queue_in(tx, false))
}

fn queue_in(tx: &mut crate::store::WriteTx<'_>, explicit: bool) -> Result<Option<QueuedDelete>> {
    let rows = super::kem_prekeys::all_rows(tx)?.len();
    if rows == 0 {
        return Ok(None);
    }
    if let Some(saved) = tx.get_json::<Saved>(Table::Checkpoints, MARKER, 1)?
        && let Some((outcome, _reply, _request)) = actions::view_in_write(tx, &saved.id)?
    {
        match outcome.status {
            actions::ActionStatus::Queued | actions::ActionStatus::Sent => {
                if explicit {
                    return Err(invalid(
                        "a previous PQ delete is queued or uncertain; no duplicate request",
                    ));
                }
                return Ok(None);
            }
            _ => {}
        }
    }
    let id = actions::queue_in(tx, "pq_delete", "delete PQ prekeys", |id| {
        super::prekeys::delete_pq_request(id)
    })?;
    tx.put_json(
        Table::Checkpoints,
        MARKER,
        1,
        &Saved {
            id: id.clone(),
            deleted: None,
            refused: false,
        },
    )?;
    Ok(Some(QueuedDelete {
        action_id: id,
        rows,
    }))
}

/// Saved delete lifecycle, without changing anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inspection {
    /// `queued`, `outcome_unknown`, `refused`, `deleted` or `invalid_reply`.
    pub status: String,
    /// Local KEM rows currently stored.
    pub rows: usize,
    /// Rows removed by a definite result.
    pub deleted: Option<usize>,
}

/// Inspect the saved delete request.
pub fn inspect(store: &ProtocolStore) -> Result<Option<Inspection>> {
    store.read(inspect_in)
}

fn inspect_in(tx: &ReadTx<'_>) -> Result<Option<Inspection>> {
    let Some(saved) = tx.get_json::<Saved>(Table::Checkpoints, MARKER, 1)? else {
        return Ok(None);
    };
    let rows = super::kem_prekeys::all_rows(tx)?.len();
    let Some((outcome, _reply)) = actions::view_in(tx, &saved.id)? else {
        return Err(invalid("saved delete action is missing"));
    };
    let status = if saved.deleted.is_some() {
        "deleted"
    } else if saved.refused {
        "refused"
    } else {
        match outcome.status {
            actions::ActionStatus::Queued => "queued",
            actions::ActionStatus::Sent => "outcome_unknown",
            actions::ActionStatus::Error => "refused",
            actions::ActionStatus::Ok => "invalid_reply",
        }
    };
    Ok(Some(Inspection {
        status: status.into(),
        rows,
        deleted: saved.deleted,
    }))
}

/// Settle the saved delete if `id` is its action. A definite result wipes the
/// local KEM rows; a refusal is recorded and keeps them. Returns the number of
/// rows removed when this call performed the wipe.
pub fn settle_reply(store: &ProtocolStore, id: &str) -> Result<Option<usize>> {
    let saved: Option<Saved> = store.read(|tx| tx.get_json(Table::Checkpoints, MARKER, 1))?;
    let Some(saved) = saved.filter(|saved| saved.id == id) else {
        return Ok(None);
    };
    if saved.deleted.is_some() || saved.refused {
        return Ok(None);
    }
    settle_pending(store)
}

/// Wipe KEM rows when the saved delete has a definite result. Idempotent; also
/// run at startup so a crash between reply settlement and the wipe recovers.
pub fn settle_pending(store: &ProtocolStore) -> Result<Option<usize>> {
    store.write(|tx| {
        let Some(mut saved) = tx.get_json::<Saved>(Table::Checkpoints, MARKER, 1)? else {
            return Ok(None);
        };
        if saved.deleted.is_some() || saved.refused {
            return Ok(None);
        }
        let Some((outcome, _reply, _request)) = actions::view_in_write(tx, &saved.id)? else {
            return Ok(None);
        };
        match outcome.status {
            actions::ActionStatus::Ok => {
                let removed = super::kem_prekeys::delete_all_in(tx)?;
                saved.deleted = Some(removed);
                tx.put_json(Table::Checkpoints, MARKER, 1, &saved)?;
                Ok(Some(removed))
            }
            actions::ActionStatus::Error => {
                saved.refused = true;
                tx.put_json(Table::Checkpoints, MARKER, 1, &saved)?;
                Ok(None)
            }
            _ => Ok(None),
        }
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
            path: path.into(),
            account_id: AccountId::new("pq-delete").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([47; 32]))),
            create: true,
        })
        .unwrap()
    }

    fn add_row(store: &ProtocolStore, last_resort: bool) {
        store
            .write(|tx| super::super::kem_prekeys::generate_in(tx, last_resort))
            .unwrap();
    }

    fn settle(store: &ProtocolStore, id: &str, result: bool) {
        actions::reserve_pending(store).unwrap();
        let reply = if result {
            crate::binary::Node::new("iq")
                .attr("id", id)
                .attr("type", "result")
        } else {
            crate::binary::Node::new("iq")
                .attr("id", id)
                .attr("type", "error")
                .children(vec![crate::binary::Node::new("error").attr("code", "500")])
        };
        actions::record_reply(store, &reply).unwrap();
    }

    fn settle_abprops(store: &ProtocolStore, value: &str) {
        let action = super::super::abprops::queue(store).unwrap();
        actions::reserve_pending(store).unwrap();
        actions::record_reply(
            store,
            &crate::binary::Node::new("iq")
                .attr("id", action.as_str())
                .attr("type", "result")
                .children(vec![
                    crate::binary::Node::new("props")
                        .attr("hash", "h")
                        .children(vec![
                            crate::binary::Node::new("prop")
                                .attr("config_code", "21198")
                                .attr("config_value", value),
                        ]),
                ]),
        )
        .unwrap();
    }

    #[test]
    fn delete_request_is_the_native_pq_delete_shape() {
        let node = super::super::prekeys::delete_pq_request("PQ-DEL").unwrap();
        assert_eq!(node.description, "iq");
        assert_eq!(
            node.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert_eq!(node.get_attr("type").and_then(|v| v.as_text()), Some("set"));
        assert!(node.get_attr("id").and_then(|v| v.as_text()) == Some("PQ-DEL"));
        let children = node.child_nodes();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].description, "op");
        assert_eq!(
            children[0].get_attr("mode").and_then(|v| v.as_text()),
            Some("delete")
        );
        assert_eq!(children[1].description, "pq_list");
        assert!(children[1].child_nodes().is_empty());
    }

    #[test]
    fn queue_requires_inventory_and_wipes_only_after_a_result() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("delete.redb"));
        assert!(queue(&store).is_err());
        add_row(&store, true);
        add_row(&store, false);
        let queued = queue(&store).unwrap();
        assert_eq!(queued.rows, 2);
        assert_eq!(inspect(&store).unwrap().unwrap().status, "queued");
        // Queued or uncertain: no duplicate request.
        assert!(queue(&store).is_err());
        actions::reserve_pending(&store).unwrap();
        assert_eq!(inspect(&store).unwrap().unwrap().status, "outcome_unknown");
        assert!(queue(&store).is_err());
        // A definite result wipes every KEM row and stays idempotent.
        settle(&store, &queued.action_id, true);
        assert_eq!(settle_reply(&store, &queued.action_id).unwrap(), Some(2));
        let view = inspect(&store).unwrap().unwrap();
        assert_eq!(view.status, "deleted");
        assert_eq!(view.rows, 0);
        assert_eq!(view.deleted, Some(2));
        assert_eq!(settle_pending(&store).unwrap(), None);
        assert!(queue(&store).is_err());
    }

    #[test]
    fn refusal_keeps_rows_and_allows_a_new_request() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("refused.redb"));
        add_row(&store, true);
        let queued = queue(&store).unwrap();
        settle(&store, &queued.action_id, false);
        assert_eq!(settle_reply(&store, &queued.action_id).unwrap(), None);
        let view = inspect(&store).unwrap().unwrap();
        assert_eq!(view.status, "refused");
        assert_eq!(view.rows, 1);
        assert_eq!(view.deleted, None);
        // Deletion is idempotent, so a refused request may be repeated.
        let again = queue(&store).unwrap();
        assert_ne!(again.action_id, queued.action_id);
    }

    #[test]
    fn automatic_trigger_needs_a_settled_disabled_property_and_rows() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("auto.redb"));
        // No settled A/B reply yet: unknown, never inferred off.
        add_row(&store, true);
        assert_eq!(queue_if_disabled(&store).unwrap(), None);
        // Settled off with local rows: queue once, not twice.
        settle_abprops(&store, "0");
        let queued = queue_if_disabled(&store).unwrap().expect("queued");
        assert_eq!(queued.rows, 1);
        assert_eq!(queue_if_disabled(&store).unwrap(), None);
        assert_eq!(inspect(&store).unwrap().unwrap().status, "queued");
        // Settled on: nothing to do even though rows exist.
        let other = open(&dir.path().join("auto-on.redb"));
        add_row(&other, true);
        settle_abprops(&other, "1");
        assert_eq!(queue_if_disabled(&other).unwrap(), None);
    }
}
