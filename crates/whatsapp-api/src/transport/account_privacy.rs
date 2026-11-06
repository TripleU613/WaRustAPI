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
    store.read(read_receipts_in)
}

pub(crate) fn read_receipts_in(
    tx: &crate::store::ReadTx<'_>,
) -> Result<Option<ReadReceiptPreference>> {
    tx.get_json(Table::Checkpoints, STATE, 1)
}

/// Keep automatic receipt reservation and its preference gate atomic.
pub(crate) fn read_receipts_in_write(
    tx: &crate::store::WriteTx<'_>,
) -> Result<Option<ReadReceiptPreference>> {
    tx.get_json(Table::Checkpoints, STATE, 1)
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
    // Local supported route: server-only envelope, first event is privacy.
    // APK43.72 19N.A09 selects0Tp.A0D; later event siblings are not applied.
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
    let privacy = node
        .child_nodes()
        .first()
        .filter(|child| child.description == "privacy")
        .ok_or_else(|| invalid("unsupported account-sync child set"))?;
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
    // APK43.72 19N.A09 uses0Tp.A0N("category"). Unrelated child
    // metadata is not a category and cannot adopt a preference.
    for category in categories
        .iter()
        .filter(|child| child.description == "category")
    {
        attributes(category, &["name", "value"])?;
        if category.get_attr("name").and_then(|v| v.as_text()) != Some("readreceipts")
            || category.content != NodeContent::Empty
        {
            return Err(invalid("unsupported privacy category"));
        }
        value = Some(
            category
                .get_attr("value")
                .and_then(|v| v.as_text())
                .ok_or_else(|| invalid("invalid category value"))?,
        );
    }
    // APK43.72 19N.A09 puts raw category strings into a HashMap before
    // 0oe.A0R/8AX validates final values. A replaced unsupported string must
    // not veto the admitted last readreceipts value. Required text/structure
    // checks still apply to every selected category before any state or ACK.
    let value = ReadReceiptValue::parse(
        value.ok_or_else(|| invalid("unsupported privacy category count"))?,
    )?;
    Ok(Some(Update {
        id: id.to_owned(),
        value,
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
                // APK43.72 0oe.A0P inserts pending 7wp before dispatch and
                // binds 6pB.A01=false. 8R5.Bvj -> 6pB.A00 then leaves that
                // pending ownership intact even on a real IQ refusal.
                // Generic action status alone cannot retire the unowned intent.
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
    fn first_account_sync_event_owns_supported_privacy_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("first.redb"));
        let node = notification("first", vec![category("all")]);
        let mut children = node.child_nodes().to_vec();
        let mut categories = children[0].child_nodes().to_vec();
        categories.insert(
            0,
            Node::new("metadata")
                .attr("name", "readreceipts")
                .attr("value", "invalid")
                .text("opaque"),
        );
        children[0] = children[0].clone().children(categories);
        children.push(Node::new("privacy").children(vec![category("none")]));
        children.push(Node::new("devices"));
        let selected = node.children(children);
        validate(&selected).unwrap();
        assert!(handle(&store, &selected).unwrap().is_some());
        assert_eq!(
            read_receipts(&store).unwrap().unwrap().value,
            ReadReceiptValue::All
        );
        let before = snapshot(&store);
        for children in [
            vec![Node::new("privacy").children(vec![Node::new("metadata")])],
            vec![
                Node::new("devices"),
                Node::new("privacy").children(vec![category("none")]),
            ],
            vec![
                Node::new("privacy").children(vec![category("invalid")]),
                Node::new("privacy").children(vec![category("none")]),
            ],
        ] {
            let bad = notification("bad-first", vec![]).children(children);
            assert!(validate(&bad).is_err());
            assert!(handle(&store, &bad).is_err());
            assert_eq!(snapshot(&store), before);
        }
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
                Node::new("devices"),
                Node::new("privacy").children(vec![category("all")]),
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
    fn outer_refusals_do_not_retire_unowned_privacy_setter_intent() {
        for legacy_misclassification in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("legacy-setter.redb");
            let store = open(&path);
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
            assert!(ack.is_none());
            assert_eq!(snapshot(&store), before);
            assert_eq!(read_receipts(&store).unwrap(), None);
            let head = store.journal_head().unwrap();
            drop(store);
            let store = open(&path);
            assert!(
                handle(&store, &notification("guard", vec![category("none")]))
                    .unwrap()
                    .is_none()
            );
            assert_eq!(snapshot(&store), before);
            assert_eq!(store.journal_head().unwrap(), head);

            assert!(
                super::super::actions::reserve_pending(&store)
                    .unwrap()
                    .is_empty()
            );
        }
    }
    #[test]
    fn duplicate_category_values_are_validated_after_last_write_selection() {
        for prefix in ["fixture-unsupported", "contact_blacklist", ""] {
            for final_value in ["all", "contacts", "none", "known", "off", "on_standard"] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("duplicate-values.redb");
                let store = open(&path);
                let old = notification(
                    "old-duplicate",
                    vec![category(prefix), category(final_value)],
                );
                validate(&old).unwrap();
                let ack = handle(&store, &old).unwrap().unwrap();
                assert_eq!(
                    read_receipts(&store).unwrap().unwrap().value,
                    ReadReceiptValue::parse(final_value).unwrap()
                );
                handle(&store, &notification("newer", vec![category("all")])).unwrap();
                let before = snapshot(&store);
                let head = store.journal_head().unwrap();
                drop(store);
                let store = open(&path);
                assert_eq!(handle(&store, &old).unwrap(), Some(ack));
                assert_eq!(snapshot(&store), before);
                assert_eq!(store.journal_head().unwrap(), head);
                assert_eq!(
                    read_receipts(&store).unwrap().unwrap().value,
                    ReadReceiptValue::All
                );
                for categories in [
                    vec![category("all"), category(prefix)],
                    vec![
                        Node::new("category").attr("name", "readreceipts"),
                        category("all"),
                    ],
                    vec![category(prefix).text("unsupported-body"), category("all")],
                    vec![
                        category(prefix).attr("extra", "unsupported"),
                        category("all"),
                    ],
                ] {
                    let bad = notification("bad-duplicate", categories);
                    assert!(validate(&bad).is_err());
                    assert!(handle(&store, &bad).is_err());
                    assert_eq!(snapshot(&store), before);
                    assert_eq!(store.journal_head().unwrap(), head);
                }
            }
        }
    }
}
