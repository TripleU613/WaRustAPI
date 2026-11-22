//! One-shot classic privacy setting set (SMAX branch).
//!
//! APK 2.26.43.72 `0sv.A0P` records a pending category value, then Boolean
//! property **23639** selects a MEX branch or the classic SMAX IQ:
//! `iq type="set" xmlns="privacy"` to the server with
//! `<privacy><category name="…" value="…"/></privacy>`. `08Y.A0X` registers
//! 23639 with `Boolean.FALSE`, so SMAX is the default; MEX set is not
//! implemented here.
//!
//! The setter is deliberately one-shot and explicit: the CLI requires both
//! `--allow-live-request` and `--confirm`, the queued action is transmitted by
//! `listen`, and a queued, uncertain or refused setter is never automatically
//! repeated. While a setter action is outstanding, `account_privacy` keeps
//! refusing to adopt incoming category notifications (native pending-ownership
//! behaviour), so the two cannot race.

use super::actions;
use crate::{
    Error, Result,
    binary::Node,
    model::jid::Jid,
    store::{ProtocolStore, ReadTx, Table},
};
use serde::{Deserialize, Serialize};

/// Boolean property selecting the native set path (`0sv.A0P`), default false.
pub const SET_MODE_PROPERTY: i32 = 23639;
const CATEGORY: &str = "readreceipts";

fn marker() -> Vec<u8> {
    format!("account-privacy-set:{CATEGORY}").into_bytes()
}

fn invalid(reason: &'static str) -> Error {
    Error::invalid("privacy_set", reason)
}

/// Which native set path property 23639 selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetMode {
    /// 23639 true: MEX mutation — not implemented.
    Mex,
    /// 23639 false or absent (registered default): classic privacy/set.
    Smax,
}

fn mode(value: Option<i32>) -> SetMode {
    match value {
        Some(value) if value != 0 => SetMode::Mex,
        _ => SetMode::Smax,
    }
}

/// Selected set path for the saved A/B reply.
pub fn selected_mode(store: &ProtocolStore) -> Result<SetMode> {
    store.read(|tx| {
        Ok(mode(super::abprops::property_value_in(
            tx,
            SET_MODE_PROPERTY,
        )?))
    })
}

#[derive(Serialize, Deserialize)]
struct Saved {
    id: String,
    value: String,
}

/// A queued setter and the value it will request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedSet {
    /// Durable action id; `listen` transmits it.
    pub action_id: String,
    /// Requested native category value.
    pub value: super::account_privacy::ReadReceiptValue,
}

/// Queue one explicit classic readreceipts set.
///
/// Refuses when property 23639 selects MEX, when a previous setter is queued or
/// uncertain, or when the requested value is the one the previous setter
/// already asked for (a refused or applied identical repeat is never resent).
/// A *different* value is a new explicit intent: it is allowed after a definite
/// success, and after a refusal it is also allowed because the refusal applies
/// to that request, not to the account. Nothing is retried automatically.
pub fn queue(
    store: &ProtocolStore,
    value: super::account_privacy::ReadReceiptValue,
) -> Result<QueuedSet> {
    store.write(|tx| {
        if mode(super::abprops::property_value_in_write(
            tx,
            SET_MODE_PROPERTY,
        )?) == SetMode::Mex
        {
            return Err(invalid("selected set mode is MEX; not implemented"));
        }
        let requested = value_text(value).to_owned();
        if let Some(saved) = tx.get_json::<Saved>(Table::Checkpoints, &marker(), 1)? {
            let Some((outcome, _reply, _request)) = actions::view_in_write(tx, &saved.id)? else {
                return Err(invalid("saved setter action is missing"));
            };
            match outcome.status {
                actions::ActionStatus::Queued | actions::ActionStatus::Sent => {
                    return Err(invalid("a previous set is queued or uncertain; no retry"));
                }
                actions::ActionStatus::Error if saved.value == requested => {
                    return Err(invalid(
                        "that set was refused; the same value is not resent",
                    ));
                }
                actions::ActionStatus::Error => {}
                actions::ActionStatus::Ok if saved.value == requested => {
                    return Err(invalid("that value is already applied"));
                }
                actions::ActionStatus::Ok => {}
            }
        }
        let id = actions::queue_in(tx, "privacy_set", "classic privacy set", |id| {
            // 0sv.A0P 0100..01cc: one category child, no other payload.
            Ok(Node::new("iq")
                .attr("id", id)
                .attr("type", "set")
                .attr("xmlns", "privacy")
                .attr("to", Jid::parse("s.whatsapp.net")?)
                .children(vec![Node::new("privacy").children(vec![
                    Node::new("category")
                        .attr("name", CATEGORY)
                        .attr("value", requested.clone()),
                ])]))
        })?;
        tx.put_json(
            Table::Checkpoints,
            &marker(),
            1,
            &Saved {
                id: id.clone(),
                value: requested,
            },
        )?;
        Ok(QueuedSet {
            action_id: id,
            value,
        })
    })
}

fn value_text(value: super::account_privacy::ReadReceiptValue) -> &'static str {
    use super::account_privacy::ReadReceiptValue as Value;
    match value {
        Value::All => "all",
        Value::Contacts => "contacts",
        Value::None => "none",
        Value::Known => "known",
        Value::Off => "off",
        Value::OnStandard => "on_standard",
    }
}

/// Saved setter lifecycle, without applying anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetInspection {
    /// `queued`, `outcome_unknown`, `refused`, `applied` or `invalid_reply`.
    pub status: String,
    /// Value the setter requests.
    pub value: super::account_privacy::ReadReceiptValue,
}

/// Inspect the saved setter.
pub fn inspect(store: &ProtocolStore) -> Result<Option<SetInspection>> {
    store.read(inspect_in)
}

fn inspect_in(tx: &ReadTx<'_>) -> Result<Option<SetInspection>> {
    use actions::ActionStatus;
    let Some(saved) = tx.get_json::<Saved>(Table::Checkpoints, &marker(), 1)? else {
        return Ok(None);
    };
    let value = super::account_privacy::ReadReceiptValue::parse(&saved.value)?;
    let Some((outcome, _reply)) = actions::view_in(tx, &saved.id)? else {
        return Err(invalid("saved setter action is missing"));
    };
    Ok(Some(SetInspection {
        status: match outcome.status {
            ActionStatus::Queued => "queued",
            ActionStatus::Sent => "outcome_unknown",
            ActionStatus::Error => "refused",
            ActionStatus::Ok => "applied",
        }
        .into(),
        value,
    }))
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
            account_id: AccountId::new("privacy-set").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([43; 32]))),
            create: true,
        })
        .unwrap()
    }

    fn settle(store: &ProtocolStore, id: &str, result: bool) {
        actions::reserve_pending(store).unwrap();
        let reply = if result {
            Node::new("iq").attr("id", id).attr("type", "result")
        } else {
            Node::new("iq")
                .attr("id", id)
                .attr("type", "error")
                .children(vec![Node::new("error").attr("code", "403")])
        };
        actions::record_reply(store, &reply).unwrap();
    }

    fn abprops_23639(store: &ProtocolStore, value: &str) {
        let action = super::super::abprops::queue(store).unwrap();
        actions::reserve_pending(store).unwrap();
        actions::record_reply(
            store,
            &Node::new("iq")
                .attr("id", action.as_str())
                .attr("type", "result")
                .children(vec![Node::new("props").attr("hash", "h").children(vec![
                        Node::new("prop")
                            .attr("config_code", "23639")
                            .attr("config_value", value),
                    ])]),
        )
        .unwrap();
    }

    #[test]
    fn classic_set_shape_defaults_to_smax_and_repeats_are_refused() {
        use super::super::account_privacy::ReadReceiptValue as Value;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("set.redb"));
        assert_eq!(selected_mode(&store).unwrap(), SetMode::Smax);
        let queued = queue(&store, Value::None).unwrap();
        let request = actions::request(&store, &queued.action_id)
            .unwrap()
            .unwrap();
        assert_eq!(request.description, "iq");
        assert_eq!(
            request.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("privacy")
        );
        assert_eq!(
            request.get_attr("type").and_then(|v| v.as_text()),
            Some("set")
        );
        let privacy = request.child("privacy").expect("privacy child");
        let category = &privacy.child_nodes()[0];
        assert_eq!(
            category.get_attr("name").and_then(|v| v.as_text()),
            Some("readreceipts")
        );
        assert_eq!(
            category.get_attr("value").and_then(|v| v.as_text()),
            Some("none")
        );
        assert_eq!(inspect(&store).unwrap().unwrap().status, "queued");
        // Queued but unsent: no second action.
        assert!(queue(&store, Value::None).is_err());
        assert!(queue(&store, Value::All).is_err());
        // Uncertain: still no repeat.
        actions::reserve_pending(&store).unwrap();
        assert!(queue(&store, Value::None).is_err());
        assert_eq!(inspect(&store).unwrap().unwrap().status, "outcome_unknown");
    }

    #[test]
    fn refused_set_is_not_resent_and_a_different_value_is_a_new_intent() {
        use super::super::account_privacy::ReadReceiptValue as Value;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("refused.redb"));
        let id = queue(&store, Value::None).unwrap().action_id;
        settle(&store, &id, false);
        assert_eq!(inspect(&store).unwrap().unwrap().status, "refused");
        // The same refused value is never resent, but a different explicit
        // value is a new request, not a retry.
        assert!(queue(&store, Value::None).is_err());
        let id = queue(&store, Value::All).unwrap().action_id;
        settle(&store, &id, true);
        assert_eq!(inspect(&store).unwrap().unwrap().status, "applied");
        assert!(queue(&store, Value::All).is_err());
        assert!(queue(&store, Value::None).is_ok());
    }

    #[test]
    fn mex_selected_by_property_23639_is_refused() {
        use super::super::account_privacy::ReadReceiptValue as Value;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("mex.redb"));
        abprops_23639(&store, "1");
        assert_eq!(selected_mode(&store).unwrap(), SetMode::Mex);
        let error = queue(&store, Value::None).unwrap_err().to_string();
        assert!(error.contains("MEX"), "{error}");
        assert!(inspect(&store).unwrap().is_none());
    }
}
