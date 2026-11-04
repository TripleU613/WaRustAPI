//! Supported account_sync privacy category adoption, APK 2.26.43.72.
//! `19N.A09` -> `0oe.A0R` -> `8AX.A00/A04/A03`; base `0v6.A01`
//! ACKs after processing. Refresh/other categories remain unacknowledged.
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::Jid,
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STATE: &[u8] = b"account-privacy:readreceipts";

/// Server category value admitted by current `8AX.A00/A04` for readreceipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadReceiptValue {
    /// Native all (0).
    All,
    /// Native contacts (1); Boolean conversion is false.
    Contacts,
    /// Native none (2).
    None,
    /// Native known (5); Boolean conversion is false.
    Known,
    /// Native off (7).
    Off,
    /// Native on_standard (8).
    OnStandard,
}
impl ReadReceiptValue {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "all" => Ok(Self::All),
            "contacts" => Ok(Self::Contacts),
            "none" => Ok(Self::None),
            "known" => Ok(Self::Known),
            "off" => Ok(Self::Off),
            "on_standard" => Ok(Self::OnStandard),
            _ => Err(invalid("unsupported readreceipts value")),
        }
    }
    /// Current `8AX.A03`: true only for all/on_standard after admission.
    pub const fn enabled(self) -> bool {
        matches!(self, Self::All | Self::OnStandard)
    }
}

/// Last supported server category, without inferring an absent preference.
/// This inspection does not alter explicit local read or read-self policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadReceiptPreference {
    /// Validated server category value.
    pub value: ReadReceiptValue,
}

/// Inspect the durable server preference; absent remains unknown.
pub fn read_receipts(store: &ProtocolStore) -> Result<Option<ReadReceiptPreference>> {
    store.read(|tx| tx.get_json(Table::Checkpoints, STATE, 1))
}

fn invalid(reason: &'static str) -> Error {
    Error::invalid("account_privacy", reason)
}

fn attributes(node: &Node, allowed: &[&str]) -> Result<()> {
    for (i, attr) in node.attributes.iter().enumerate() {
        if !allowed.contains(&attr.name.as_str())
            || node.attributes[..i].iter().any(|old| old.name == attr.name)
        {
            return Err(invalid("unsupported or duplicate attribute"));
        }
    }
    Ok(())
}

struct Update {
    id: String,
    value: ReadReceiptValue,
    ack: Node,
}

fn parse(node: &Node) -> Result<Option<Update>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("account_sync")
    {
        return Ok(None);
    }
    // Local supported route: server-only envelope, exactly one privacy child.
    // Native 19N reads the first child; unknown siblings are never ACKed here.
    attributes(node, &["from", "id", "type", "t"])?;
    let from =
        super::receipts::jid(node, "from").ok_or_else(|| invalid("invalid account-sync source"))?;
    if from != Jid::parse("s.whatsapp.net")? {
        return Err(invalid("unsupported account-sync source"));
    }
    let id = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|id| !id.is_empty() && id.len() <= 128 && !id.chars().any(char::is_control))
        .ok_or_else(|| invalid("invalid notification id"))?;
    let time = node
        .get_attr("t")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|v| v.parse::<i64>().ok());
    if time.is_none() {
        return Err(invalid("invalid notification timestamp"));
    }
    let children = node.child_nodes();
    if children.len() != 1 || children[0].description != "privacy" {
        return Err(invalid("unsupported account-sync child set"));
    }
    let privacy = &children[0];
    attributes(privacy, &["action"])?;
    if let Some(action) = privacy.get_attr("action") {
        let action = action
            .as_text()
            .ok_or_else(|| invalid("invalid privacy action"))?;
        if action == "modify" {
            return Err(invalid("privacy refresh is not implemented"));
        }
    }
    let categories = privacy.child_nodes();
    if categories.is_empty() || categories.len() > 64 {
        return Err(invalid("unsupported privacy category count"));
    }
    let mut value = None;
    for category in categories {
        attributes(category, &["name", "value"])?;
        if category.description != "category"
            || category.get_attr("name").and_then(|v| v.as_text()) != Some("readreceipts")
            || category.content != NodeContent::Empty
        {
            return Err(invalid("unsupported privacy category"));
        }
        value = Some(ReadReceiptValue::parse(
            category
                .get_attr("value")
                .and_then(|v| v.as_text())
                .ok_or_else(|| invalid("invalid category value"))?,
        )?);
    }
    // 19N builds a HashMap with put: repeated supported names use the last value.
    Ok(Some(Update {
        id: id.to_owned(),
        value: value.expect("nonempty categories"),
        ack: Node::new("ack")
            .attr("id", id)
            .attr("to", super::receipts::ack_address(from))
            .attr("class", "notification")
            .attr("type", "account_sync"),
    }))
}

/// Pure wire validation; the receive loop catches only this stage's refusal.
pub(crate) fn validate(node: &Node) -> Result<()> {
    parse(node).map(|_| ())
}

/// Adopt the complete supported child atomically before exposing its ACK.
/// Redelivery re-ACKs without applying an old preference over a newer one.
pub(crate) fn handle(store: &ProtocolStore, node: &Node) -> Result<Option<Node>> {
    let Some(update) = parse(node)? else {
        return Ok(None);
    };
    let key = format!(
        "account-privacy:notification:{}",
        hex::encode(Sha256::digest(update.id.as_bytes()))
    );
    store.write(|tx| {
        if let Some(saved) =
            tx.get_json::<ReadReceiptValue>(Table::Checkpoints, key.as_bytes(), 1)?
        {
            if saved != update.value {
                return Err(invalid(
                    "notification id conflicts with saved privacy value",
                ));
            }
            return Ok(Some(update.ack));
        }
        // No typed Rust privacy setter owns a native pending map yet. Generic
        // actions can still carry a setter, so do not overwrite its intention.
        // This conservative local guard is not a reconstruction of 0oe.A0E.
        for action in super::actions::outcomes_in_write(tx)? {
            let request = super::actions::request_in_write(tx, &action.id)?
                .ok_or_else(|| invalid("saved action request is missing"))?;
            if request.get_attr("xmlns").and_then(|v| v.as_text()) == Some("privacy")
                && request.get_attr("type").and_then(|v| v.as_text()) == Some("set")
                && request.child("privacy").is_some()
            {
                // Older generic rows could label an unexpected IQ type as
                // Error. Only an actual outer refusal discharges this guard.
                if action.status == super::actions::ActionStatus::Error
                    && super::actions::view_in_write(tx, &action.id)?
                        .and_then(|(_, reply, _)| reply)
                        .is_some_and(|reply| {
                            reply.get_attr("type").and_then(|v| v.as_text()) == Some("error")
                        })
                {
                    continue;
                }
                tracing::warn!(
                    "privacy category retained without ACK: setter reconciliation is unowned"
                );
                return Ok(None);
            }
        }
        tx.put_json(
            Table::Checkpoints,
            STATE,
            1,
            &ReadReceiptPreference {
                value: update.value,
            },
        )?;
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &update.value)?;
        Ok(Some(update.ack))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use std::sync::Arc;

    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("privacy-fixture").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn category(value: &str) -> Node {
        Node::new("category")
            .attr("name", "readreceipts")
            .attr("value", value)
    }
    fn notification(id: &str, categories: Vec<Node>) -> Node {
        Node::new("notification")
            .attr("type", "account_sync")
            .attr("from", "s.whatsapp.net")
            .attr("id", id)
            .attr("t", "1700000000")
            .children(vec![Node::new("privacy").children(categories)])
    }
    fn snapshot(store: &ProtocolStore) -> Vec<(Vec<u8>, Vec<u8>)> {
        store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap()
    }

    #[test]
    fn current_category_values_and_boolean_conversion_are_not_read_self_policy() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("values.redb"));
        assert_eq!(read_receipts(&store).unwrap(), None);
        for (i, (value, enabled)) in [
            ("all", true),
            ("contacts", false),
            ("none", false),
            ("known", false),
            ("off", false),
            ("on_standard", true),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                handle(&store, &notification(&i.to_string(), vec![category(value)]))
                    .unwrap()
                    .is_some()
            );
            let saved = read_receipts(&store).unwrap().unwrap();
            assert_eq!(saved.value, ReadReceiptValue::parse(value).unwrap());
            assert_eq!(saved.value.enabled(), enabled);
        }
    }

    #[test]
    fn redelivery_after_reopen_acknowledges_without_replacing_newer_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reopen.redb");
        let store = open(&path);
        let old = notification("old", vec![category("all"), category("none")]);
        let ack = handle(&store, &old).unwrap().unwrap();
        assert_eq!(
            ack,
            Node::new("ack")
                .attr("id", "old")
                .attr("to", Jid::parse("s.whatsapp.net").unwrap())
                .attr("class", "notification")
                .attr("type", "account_sync")
        );
        assert_eq!(
            read_receipts(&store).unwrap().unwrap().value,
            ReadReceiptValue::None
        );
        handle(&store, &notification("new", vec![category("all")])).unwrap();
        let before = snapshot(&store);
        drop(store);
        let store = open(&path);
        assert_eq!(handle(&store, &old).unwrap(), Some(ack));
        assert_eq!(snapshot(&store), before);
        assert_eq!(
            read_receipts(&store).unwrap().unwrap().value,
            ReadReceiptValue::All
        );
        let conflict = notification("old", vec![category("on_standard")]);
        assert!(handle(&store, &conflict).is_err());
        assert_eq!(snapshot(&store), before);
    }

    #[test]
    fn malformed_or_mixed_batches_leave_every_checkpoint_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("invalid.redb"));
        handle(&store, &notification("original", vec![category("none")])).unwrap();
        let good = notification("bad", vec![category("all")]);
        let mut duplicate = good.clone();
        duplicate.attributes.push(duplicate.attributes[0].clone());
        let mut malformed = good.clone();
        malformed
            .attributes
            .iter_mut()
            .find(|a| a.name == "t")
            .unwrap()
            .value = crate::binary::AttributeValue::Bytes(vec![255].into());
        let cases = vec![
            good.clone().attr("from", "15555550100@s.whatsapp.net"),
            good.clone().attr("from", "s.whatsapp.net:1"),
            good.clone()
                .attr("participant", "15555550100@s.whatsapp.net"),
            good.clone().attr("recipient", "15555550100@s.whatsapp.net"),
            good.clone().attr("id", ""),
            good.clone().attr("t", "-1"),
            good.clone().attr("t", "9223372036854775808"),
            duplicate,
            malformed,
            notification("bad", vec![category("all"), category("contact_blacklist")]),
            notification(
                "bad",
                vec![
                    category("all"),
                    Node::new("category")
                        .attr("name", "profile")
                        .attr("value", "all"),
                ],
            ),
            notification(
                "bad",
                vec![
                    category("all"),
                    Node::new("category").attr("name", "readreceipts"),
                ],
            ),
            notification("bad", vec![]),
            notification("bad", vec![category("all").text("unknown")]),
            good.clone().children(vec![
                Node::new("privacy")
                    .attr("action", "modify")
                    .children(vec![category("all")]),
            ]),
            good.children(vec![
                Node::new("privacy").children(vec![category("all")]),
                Node::new("devices"),
            ]),
        ];
        let before = snapshot(&store);
        for case in cases {
            assert!(validate(&case).is_err());
            assert!(handle(&store, &case).is_err());
            assert_eq!(snapshot(&store), before);
        }
    }

    #[test]
    fn generic_privacy_setter_blocks_adoption_including_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pending.redb");
        let store = open(&path);
        let id = super::super::actions::queue(&store, "fixture", "privacy setter", |id| {
            Ok(Node::new("iq")
                .attr("id", id)
                .attr("xmlns", "privacy")
                .attr("type", "set")
                .children(vec![Node::new("privacy").children(vec![category("none")])]))
        })
        .unwrap();
        let update = notification("new", vec![category("all")]);
        let before = snapshot(&store);
        assert_eq!(handle(&store, &update).unwrap(), None);
        assert_eq!(snapshot(&store), before);
        super::super::actions::reserve_pending(&store).unwrap();
        drop(store);
        let store = open(&path);
        let before = snapshot(&store);
        assert_eq!(handle(&store, &update).unwrap(), None);
        assert_eq!(snapshot(&store), before);
        super::super::actions::record_reply(
            &store,
            &Node::new("iq").attr("id", id).attr("type", "result"),
        )
        .unwrap();
        let before = snapshot(&store);
        assert_eq!(handle(&store, &update).unwrap(), None);
        assert_eq!(snapshot(&store), before);
        assert_eq!(read_receipts(&store).unwrap(), None);
    }

    #[test]
    fn only_a_real_outer_refusal_clears_the_unowned_setter_guard() {
        for legacy_misclassification in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("legacy-setter.redb"));
            let id = super::super::actions::queue(&store, "fixture", "setter", |id| {
                Ok(Node::new("iq")
                    .attr("id", id)
                    .attr("type", "set")
                    .attr("xmlns", "privacy")
                    .children(vec![Node::new("privacy")]))
            })
            .unwrap();
            super::super::actions::reserve_pending(&store).unwrap();
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr(
                    "type",
                    if legacy_misclassification {
                        "get"
                    } else {
                        "error"
                    },
                )
                .children(vec![Node::new("error").attr("code", "503")]);
            if legacy_misclassification {
                let wire = crate::binary::Encoder::new(
                    crate::binary::TokenDictionary::comparison_2_26_37_71(),
                )
                .encode(&reply)
                .unwrap();
                store
                    .write(|tx| {
                        let key = format!("action-outbox:{id}");
                        let mut row: serde_json::Value =
                            tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                        row["status"] = serde_json::json!("error");
                        row["reply"] = serde_json::json!(wire);
                        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)
                    })
                    .unwrap();
            } else {
                assert!(super::super::actions::record_reply(&store, &reply).unwrap());
            }
            let before = snapshot(&store);
            let ack = handle(&store, &notification("guard", vec![category("none")])).unwrap();
            assert_eq!(ack.is_none(), legacy_misclassification);
            if legacy_misclassification {
                assert_eq!(snapshot(&store), before);
            } else {
                assert_eq!(
                    read_receipts(&store).unwrap().unwrap().value,
                    ReadReceiptValue::None
                );
            }
        }
    }
}
