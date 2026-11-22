//! Read-only privacy-setting fetch (SMAX branch).
//!
//! APK 2.26.43.72 `0sv`: integer property **23463** selects the fetch path
//! (1 → MEX, 2 → shadow, other/absent → SMAX), and `0sv.A02` builds
//! `iq type="get" xmlns="privacy"` to the server with a single empty
//! `<privacy/>` child. The SMAX reply is a first `privacy` child whose
//! `category` children carry text `name`/`value`; a value of `error` is
//! replaced by the first child's `code`, and a category with a missing name,
//! value or code is dropped.
//!
//! This module queues that one read on the durable action queue and inspects
//! the correlated reply. It **does not adopt** any returned setting: category
//! adoption authority (notification/map validation and the pending setter
//! guard) stays in [`super::account_privacy`], and MEX/shadow fetching is not
//! implemented. `action=modify` notifications are a separate envelope.

use super::actions;
use crate::{
    Error, Result,
    binary::Node,
    model::jid::Jid,
    store::{ProtocolStore, ReadTx, Table},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MARKER: &[u8] = b"account-privacy-fetch";
/// Integer property selecting the native privacy fetch path (`0sv.A0M`).
pub const FETCH_MODE_PROPERTY: i32 = 23463;

/// Which native fetch path the server's property selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchMode {
    /// 1: MEX coroutine with selector 24 — not implemented.
    Mex,
    /// 2: shadow MEX+SMAX comparison — not implemented.
    Shadow,
    /// Anything else, including an absent property: classic SMAX.
    Smax,
}

/// Property value to fetch mode; only the SMAX branch is implemented.
pub fn mode(store: &ProtocolStore) -> Result<FetchMode> {
    store.read(mode_in)
}

fn mode_in(tx: &ReadTx<'_>) -> Result<FetchMode> {
    Ok(select_mode(super::abprops::property_value_in(
        tx,
        FETCH_MODE_PROPERTY,
    )?))
}

fn mode_in_write(tx: &crate::store::WriteTx<'_>) -> Result<FetchMode> {
    Ok(select_mode(super::abprops::property_value_in_write(
        tx,
        FETCH_MODE_PROPERTY,
    )?))
}

fn select_mode(value: Option<i32>) -> FetchMode {
    match value {
        Some(1) => FetchMode::Mex,
        Some(2) => FetchMode::Shadow,
        _ => FetchMode::Smax,
    }
}

/// Queue one classic privacy fetch. Repeats return the original action.
/// A selected MEX/shadow path refuses instead of sending the wrong wire form.
pub fn queue(store: &ProtocolStore) -> Result<String> {
    store.write(|tx| {
        if let Some(id) = tx.get_json::<String>(Table::Checkpoints, MARKER, 1)? {
            return Ok(id);
        }
        match mode_in_write(tx)? {
            FetchMode::Smax => {}
            FetchMode::Mex => {
                return Err(Error::invalid(
                    "privacy_fetch",
                    "selected fetch mode is MEX; not implemented",
                ));
            }
            FetchMode::Shadow => {
                return Err(Error::invalid(
                    "privacy_fetch",
                    "selected fetch mode is shadow MEX/SMAX; not implemented",
                ));
            }
        }
        let id = actions::queue_in(tx, "privacy_fetch", "classic privacy fetch", |id| {
            // 0sv.A02 0032..00c0: get/encrypt-style IQ, empty privacy child.
            Ok(Node::new("iq")
                .attr("id", id)
                .attr("type", "get")
                .attr("xmlns", "privacy")
                .attr("to", Jid::parse("s.whatsapp.net")?)
                .children(vec![Node::new("privacy")]))
        })?;
        tx.put_json(Table::Checkpoints, MARKER, 1, &id)?;
        Ok(id)
    })
}

/// Lifecycle and selected values of the saved fetch, never applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchInspection {
    /// `queued`, `outcome_unknown`, `refused`, `invalid_reply` or `received`.
    pub status: String,
    /// Number of category pairs built from the reply (before validation).
    pub categories: usize,
    /// Validated `readreceipts` value, when the reply names one.
    pub read_receipts: Option<super::account_privacy::ReadReceiptValue>,
    /// The reply named `readreceipts` with a value the client does not admit.
    pub read_receipts_invalid: bool,
    /// Last returned category names (no values). Bounded by the local cap.
    pub category_names: Vec<String>,
}

/// Inspect the saved fetch without adopting anything.
pub fn inspect(store: &ProtocolStore) -> Result<Option<FetchInspection>> {
    store.read(inspect_in)
}

fn inspect_in(tx: &ReadTx<'_>) -> Result<Option<FetchInspection>> {
    use actions::ActionStatus;
    let Some(id) = tx.get_json::<String>(Table::Checkpoints, MARKER, 1)? else {
        return Ok(None);
    };
    let Some((outcome, reply)) = actions::view_in(tx, &id)? else {
        return Err(Error::invalid(
            "privacy_fetch",
            "saved fetch action is missing",
        ));
    };
    let mut view = FetchInspection {
        status: match outcome.status {
            ActionStatus::Queued => "queued".into(),
            ActionStatus::Sent => "outcome_unknown".into(),
            ActionStatus::Error => "refused".into(),
            ActionStatus::Ok => "invalid_reply".into(),
        },
        categories: 0,
        read_receipts: None,
        read_receipts_invalid: false,
        category_names: Vec::new(),
    };
    if outcome.status != ActionStatus::Ok {
        return Ok(Some(view));
    }
    let Some(reply) = reply else {
        return Ok(Some(view));
    };
    if reply.description != "iq"
        || reply.get_attr("id").and_then(|v| v.as_text()) != Some(id.as_str())
        || reply.get_attr("type").and_then(|v| v.as_text()) != Some("result")
        || reply.child("error").is_some()
    {
        return Ok(Some(view));
    }
    // FdY.C8f reads the response's first child and requires the privacy tag.
    let Some(first) = reply.child_nodes().first() else {
        view.status = "received".into();
        return Ok(Some(view));
    };
    if first.description != "privacy" {
        return Ok(Some(view));
    }
    let map = categories(first);
    view.status = "received".into();
    view.categories = map.len();
    view.category_names = map.keys().take(256).cloned().collect();
    if let Some(value) = map.get("readreceipts") {
        match super::account_privacy::ReadReceiptValue::parse(value) {
            Ok(value) => view.read_receipts = Some(value),
            Err(_) => view.read_receipts_invalid = true,
        }
    }
    Ok(Some(view))
}

/// Category map with the native `error` substitution; invalid entries drop.
fn categories(privacy: &Node) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for category in privacy
        .child_nodes()
        .iter()
        .filter(|node| node.description == "category")
    {
        let Some(name) = category
            .get_attr("name")
            .and_then(|value| value.as_text())
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(value) = category.get_attr("value").and_then(|v| v.as_text()) else {
            continue;
        };
        let value = if value == "error" {
            category
                .child_nodes()
                .first()
                .and_then(|error| error.get_attr("code"))
                .and_then(|code| code.as_text())
        } else {
            Some(value)
        };
        if let Some(value) = value {
            // Native map: duplicate names replace earlier entries.
            map.insert(name.to_owned(), value.to_owned());
        }
    }
    map
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
            account_id: AccountId::new("privacy-fetch").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([41; 32]))),
            create: true,
        })
        .unwrap()
    }

    fn settle(store: &ProtocolStore, id: &str, props: Node) {
        actions::reserve_pending(store).unwrap();
        actions::record_reply(
            store,
            &Node::new("iq")
                .attr("id", id)
                .attr("type", "result")
                .children(vec![props]),
        )
        .unwrap();
    }

    fn abprops_reply(store: &ProtocolStore, code: &str, value: &str) {
        let action = super::super::abprops::queue(store).unwrap();
        settle(
            store,
            &action,
            Node::new("props").attr("hash", "h").children(vec![
                Node::new("prop")
                    .attr("config_code", code)
                    .attr("config_value", value),
            ]),
        );
    }

    #[test]
    fn fetch_mode_follows_property_23463_and_defaults_to_smax() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("mode.redb"));
        assert_eq!(mode(&store).unwrap(), FetchMode::Smax);
        let id = queue(&store).unwrap();
        assert!(!id.is_empty());
        let request = actions::request(&store, &id).unwrap().unwrap();
        assert_eq!(
            request.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("privacy")
        );
        assert_eq!(
            request.get_attr("type").and_then(|v| v.as_text()),
            Some("get")
        );
        assert_eq!(
            request.get_attr("to").and_then(|v| v.as_jid()),
            Some(&Jid::parse("s.whatsapp.net").unwrap())
        );
        let children = request.child_nodes();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].description, "privacy");
        assert!(children[0].child_nodes().is_empty());
        // Repeats reuse the original action.
        assert_eq!(queue(&store).unwrap(), id);
        abprops_reply(&store, "23463", "1");
        assert_eq!(mode(&store).unwrap(), FetchMode::Mex);
        let other = open(&dir.path().join("mode2.redb"));
        abprops_reply(&other, "23463", "2");
        assert_eq!(mode(&other).unwrap(), FetchMode::Shadow);
    }

    #[test]
    fn mex_or_shadow_fetch_is_refused_instead_of_sending_the_wrong_form() {
        for value in ["1", "2"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join(format!("refuse-{value}.redb")));
            abprops_reply(&store, "23463", value);
            let error = queue(&store).unwrap_err().to_string();
            assert!(error.contains("not implemented"), "{error}");
            assert!(inspect(&store).unwrap().is_none());
        }
    }

    #[test]
    fn reply_categories_are_parsed_without_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("parse.redb"));
        let id = queue(&store).unwrap();
        assert_eq!(inspect(&store).unwrap().unwrap().status, "queued");
        settle(
            &store,
            &id,
            Node::new("privacy").children(vec![
                Node::new("category")
                    .attr("name", "readreceipts")
                    .attr("value", "none"),
                Node::new("category")
                    .attr("name", "last")
                    .attr("value", "all"),
            ]),
        );
        let view = inspect(&store).unwrap().unwrap();
        assert_eq!(view.status, "received");
        assert_eq!(view.categories, 2);
        assert_eq!(
            view.read_receipts,
            Some(super::super::account_privacy::ReadReceiptValue::None)
        );
        assert_eq!(view.category_names, ["last", "readreceipts"]);
        // Nothing was adopted.
        assert!(
            super::super::account_privacy::read_receipts(&store)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn error_categories_and_invalid_values_follow_the_native_map_rules() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("errors.redb"));
        let id = queue(&store).unwrap();
        settle(
            &store,
            &id,
            Node::new("privacy").children(vec![
                // A value of `error` takes the first child's code.
                Node::new("category")
                    .attr("name", "about")
                    .attr("value", "error")
                    .children(vec![Node::new("error").attr("code", "401")]),
                // Missing value: dropped.
                Node::new("category").attr("name", "no-value"),
                // Missing name: dropped.
                Node::new("category").attr("value", "all"),
                // Unsupported readreceipts value: flagged, not adopted.
                Node::new("category")
                    .attr("name", "readreceipts")
                    .attr("value", "sometimes"),
            ]),
        );
        let view = inspect(&store).unwrap().unwrap();
        assert_eq!(view.categories, 2);
        assert_eq!(view.category_names, ["about", "readreceipts"]);
        assert!(view.read_receipts_invalid);
        assert_eq!(view.read_receipts, None);
    }
}
