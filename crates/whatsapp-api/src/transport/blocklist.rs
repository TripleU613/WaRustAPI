//! Block list (`xmlns=blocklist`), traced from comparison APK **2.26.37.71**
//! and recorded in `docs/protocol/chat-actions.md` section 1.
//!
//! Set request `LX/1ep.A05` (confirmed): `<iq to=s.whatsapp.net type=set
//! xmlns=blocklist><item action=block|unblock jid=… [pn_jid=…] [dhash=…]/>
//! [<entry_point source=…/>]</iq>`. `jid` is the contact's LID when the chat is
//! LID-addressed (`super::inbox::lid_for`), otherwise the phone-number JID;
//! `pn_jid` then carries the phone-number JID. `dhash` is the client's saved
//! `block_list_v2_dhash` when non-empty. The `entry_point` child is only sent
//! for a block, `source="chat"` for a CLI-driven block (inferred choice; the
//! attribute is telemetry). Get request `LX/1ep.A0M` (confirmed): `<iq
//! to=s.whatsapp.net type=get xmlns=blocklist><item dhash=…/></iq>`, the
//! `item` present only when a dhash is saved.
//!
//! Reply parser `LX/8nc.A02` (confirmed): the reply's `list` child carries
//! `dhash`; each `item` child has `jid` (fallback `pn_jid`), `action` (default
//! `block`), `active` (default `false`), optional `display_name` and
//! `username`. The client replaces its list with the `action=block` items and
//! saves the new `dhash`. A set reply is not traced past `type=result` plus a
//! possibly-updated `dhash` (`LX/1ep.A08`); this module tolerates the `dhash`
//! attribute on the `iq` itself or on any of its immediate children, since the
//! exact placement is **unknown**.
//!
//! None of this is live-verified; nothing here has been exercised against a
//! live session.
use crate::{
    Error, Result,
    binary::Node,
    model::{jid::Jid, time::Timestamp},
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const BLOCKED_PREFIX: &str = "blocked:";
const DHASH_KEY: &str = "blocklist-dhash";
const INGESTED_PREFIX: &str = "blocklist-ingested:";
const TARGET_PREFIX: &str = "blocklist-target:";

fn invalid(reason: &'static str) -> Error {
    Error::invalid("blocklist", reason)
}

fn blocked_key(jid: &str) -> String {
    format!("{BLOCKED_PREFIX}{jid}")
}
fn target_key(id: &str) -> String {
    format!("{TARGET_PREFIX}{id}")
}
fn ingested_key(id: &str) -> String {
    format!("{INGESTED_PREFIX}{id}")
}

fn saved_dhash(store: &ProtocolStore) -> Result<Option<String>> {
    store.read(|tx| tx.get_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1))
}

fn save_target(store: &ProtocolStore, id: &str, jid: &Jid) -> Result<()> {
    store.write(|tx| tx.put_json(Table::Checkpoints, target_key(id).as_bytes(), 1, jid))
}

fn read_target(store: &ProtocolStore, id: &str) -> Result<Option<Jid>> {
    store.read(|tx| tx.get_json(Table::Checkpoints, target_key(id).as_bytes(), 1))
}

/// Persisted row for one blocked contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockedRow {
    jid: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default = "Timestamp::now")]
    since: Timestamp,
}

/// Redacted operator view of one blocked contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlockedContact {
    /// Contact JID as carried on the wire (LID or phone-number JID).
    pub jid: String,
    /// Display name from a blocklist fetch, when the server supplied one.
    pub display_name: Option<String>,
    /// Username from a blocklist fetch, when the server supplied one.
    pub username: Option<String>,
}

fn build_iq(id: &str, kind: &str, children: Vec<Node>) -> Result<Node> {
    Ok(Node::new("iq")
        .attr("to", Jid::parse("s.whatsapp.net")?)
        .attr("id", id)
        .attr("type", kind)
        .attr("xmlns", "blocklist")
        .children(children))
}

fn queue_set(
    store: &ProtocolStore,
    action: &'static str,
    chat: &Jid,
    label: &str,
) -> Result<String> {
    if !chat.is_user_like() {
        return Err(invalid(
            "block/unblock target must be a phone-number or LID identity",
        ));
    }
    let phone = chat.to_non_ad();
    let lid = super::inbox::lid_for(store, &phone)?;
    let wire_jid = lid.clone().unwrap_or_else(|| phone.clone());
    let dhash = saved_dhash(store)?;
    let mut item = Node::new("item")
        .attr("action", action)
        .attr("jid", wire_jid.clone());
    if lid.is_some() {
        item = item.attr("pn_jid", phone.clone());
    }
    if let Some(dhash) = dhash {
        item = item.attr("dhash", dhash);
    }
    let mut children = vec![item];
    if action == "block" {
        children.push(Node::new("entry_point").attr("source", "chat"));
    }
    let id = super::actions::queue(store, action, label, move |id| {
        build_iq(id, "set", children)
    })?;
    save_target(store, &id, &wire_jid)?;
    Ok(id)
}

/// Durably queue a block of `chat`. `chat` is the contact's phone-number JID;
/// the wire request uses the saved LID instead when one is known.
pub fn queue_block(store: &ProtocolStore, chat: &Jid) -> Result<String> {
    queue_set(store, "block", chat, "block contact")
}

/// Durably queue an unblock of `chat`, same addressing rule as [`queue_block`].
pub fn queue_unblock(store: &ProtocolStore, chat: &Jid) -> Result<String> {
    queue_set(store, "unblock", chat, "unblock contact")
}

/// Durably queue a blocklist fetch. Carries the saved `dhash` when one exists.
pub fn queue_fetch(store: &ProtocolStore) -> Result<String> {
    let dhash = saved_dhash(store)?;
    super::actions::queue(store, "blocklist", "fetch blocklist", move |id| {
        let children = match dhash {
            Some(dhash) => vec![Node::new("item").attr("dhash", dhash)],
            None => Vec::new(),
        };
        build_iq(id, "get", children)
    })
}

/// An attribute that may have decoded as text or as a JID (both occur on the
/// wire: the encoder picks a JID representation for values shaped like one).
fn attr_as_string(node: &Node, name: &str) -> Option<String> {
    let value = node.get_attr(name)?;
    if let Some(text) = value.as_text() {
        return Some(text.to_owned());
    }
    value.as_jid().map(ToString::to_string)
}

fn extract_dhash(node: &Node) -> Option<String> {
    if let Some(v) = node.get_attr("dhash").and_then(|v| v.as_text()) {
        return Some(v.to_owned());
    }
    node.child_nodes().iter().find_map(|c| {
        c.get_attr("dhash")
            .and_then(|v| v.as_text())
            .map(str::to_owned)
    })
}

fn apply_fetch(store: &ProtocolStore, reply: &Node) -> Result<()> {
    let list = reply.child("list");
    let mut current = BTreeSet::new();
    let mut new_rows: Vec<(String, BlockedRow)> = Vec::new();
    let mut dhash = None;
    if let Some(list) = list {
        dhash = list
            .get_attr("dhash")
            .and_then(|v| v.as_text())
            .map(str::to_owned);
        for item in list
            .child_nodes()
            .iter()
            .filter(|c| c.description == "item")
        {
            let Some(jid) = attr_as_string(item, "jid").or_else(|| attr_as_string(item, "pn_jid"))
            else {
                continue;
            };
            let action = item
                .get_attr("action")
                .and_then(|v| v.as_text())
                .unwrap_or("block");
            if action != "block" {
                continue;
            }
            current.insert(jid.clone());
            new_rows.push((
                jid.clone(),
                BlockedRow {
                    jid,
                    display_name: item
                        .get_attr("display_name")
                        .and_then(|v| v.as_text())
                        .map(str::to_owned),
                    username: item
                        .get_attr("username")
                        .and_then(|v| v.as_text())
                        .map(str::to_owned),
                    since: Timestamp::now(),
                },
            ));
        }
    }
    store.write(|tx| {
        if let Some(dhash) = &dhash {
            tx.put_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1, dhash)?;
        }
        for (jid, row) in &new_rows {
            tx.put_json(Table::Checkpoints, blocked_key(jid).as_bytes(), 1, row)?;
        }
        let mut cursor = None;
        let mut stale = Vec::new();
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                if let Ok(name) = std::str::from_utf8(&key)
                    && let Some(jid) = name.strip_prefix(BLOCKED_PREFIX)
                    && !current.contains(jid)
                {
                    stale.push(key);
                }
            }
            if done {
                break;
            }
        }
        for key in stale {
            tx.delete(Table::Checkpoints, &key)?;
        }
        Ok(())
    })
}

fn apply_block_result(store: &ProtocolStore, id: &str, action: &str, reply: &Node) -> Result<()> {
    let Some(target) = read_target(store, id)? else {
        return Ok(());
    };
    let key = blocked_key(&target.to_string());
    let dhash = extract_dhash(reply);
    store.write(|tx| {
        if action == "block" {
            let existing: Option<BlockedRow> =
                tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
            let row = existing.unwrap_or_else(|| BlockedRow {
                jid: target.to_string(),
                display_name: None,
                username: None,
                since: Timestamp::now(),
            });
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)?;
        } else {
            tx.delete(Table::Checkpoints, key.as_bytes())?;
        }
        if let Some(dhash) = &dhash {
            tx.put_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1, dhash)?;
        }
        Ok(())
    })
}

/// Apply every settled fetch/block/unblock action not yet ingested. Returns
/// how many were applied. Idempotent: a settled action is applied at most once.
pub fn ingest(store: &ProtocolStore) -> Result<usize> {
    let mut applied = 0usize;
    for kind in ["blocklist", "block", "unblock"] {
        for (outcome, reply) in super::actions::settled(store, kind)? {
            let marker = ingested_key(&outcome.id);
            let done: Option<bool> =
                store.read(|tx| tx.get_json(Table::Checkpoints, marker.as_bytes(), 1))?;
            if done.unwrap_or(false) {
                continue;
            }
            if outcome.status == super::actions::ActionStatus::Ok
                && let Some(reply) = &reply
            {
                match kind {
                    "blocklist" => apply_fetch(store, reply)?,
                    _ => apply_block_result(store, &outcome.id, kind, reply)?,
                }
                applied += 1;
            }
            store.write(|tx| tx.put_json(Table::Checkpoints, marker.as_bytes(), 1, &true))?;
        }
    }
    Ok(applied)
}

/// Saved blocked contacts, jid-ordered. No network access.
pub fn blocked(store: &ProtocolStore) -> Result<Vec<BlockedContact>> {
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                let Ok(name) = std::str::from_utf8(&key) else {
                    continue;
                };
                if !name.starts_with(BLOCKED_PREFIX) {
                    continue;
                }
                let Some(row): Option<BlockedRow> = tx.get_json(Table::Checkpoints, &key, 1)?
                else {
                    continue;
                };
                out.push(BlockedContact {
                    jid: row.jid,
                    display_name: row.display_name,
                    username: row.username,
                });
            }
            if done {
                out.sort_by(|a, b| a.jid.cmp(&b.jid));
                return Ok(out);
            }
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
    use std::sync::Arc;

    fn store() -> (tempfile::TempDir, ProtocolStore) {
        let dir = tempfile::tempdir().unwrap();
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, s)
    }

    fn phone() -> Jid {
        Jid::parse("15555550101@s.whatsapp.net").unwrap()
    }

    fn attr_text<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
        node.get_attr(name).and_then(|v| v.as_text())
    }

    #[test]
    fn block_request_shape_without_lid() {
        let (_d, s) = store();
        let id = queue_block(&s, &phone()).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        assert_eq!(sent.len(), 1);
        let iq = &sent[0];
        assert_eq!(iq.description, "iq");
        assert_eq!(attr_text(iq, "id"), Some(id.as_str()));
        assert_eq!(attr_text(iq, "type"), Some("set"));
        assert_eq!(attr_text(iq, "xmlns"), Some("blocklist"));
        assert_eq!(
            iq.get_attr("to").and_then(|v| v.as_jid()),
            Some(&Jid::parse("s.whatsapp.net").unwrap())
        );
        let item = iq.child("item").unwrap();
        assert_eq!(attr_text(item, "action"), Some("block"));
        assert_eq!(
            item.get_attr("jid").and_then(|v| v.as_jid()),
            Some(&phone())
        );
        assert!(item.get_attr("pn_jid").is_none(), "no lid on file");
        assert!(item.get_attr("dhash").is_none(), "no saved dhash");
        let entry = iq.child("entry_point").unwrap();
        assert_eq!(attr_text(entry, "source"), Some("chat"));
    }

    #[test]
    fn unblock_request_has_no_entry_point() {
        let (_d, s) = store();
        queue_unblock(&s, &phone()).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        let iq = &sent[0];
        let item = iq.child("item").unwrap();
        assert_eq!(attr_text(item, "action"), Some("unblock"));
        assert!(iq.child("entry_point").is_none());
    }

    #[test]
    fn block_uses_lid_and_pn_jid_when_known() {
        let (_d, s) = store();
        let lid = Jid::parse("987654321@lid").unwrap();
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                format!("authenticated-lid:{lid}").as_bytes(),
                1,
                &phone(),
            )
        })
        .unwrap();
        queue_block(&s, &phone()).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        let item = sent[0].child("item").unwrap();
        assert_eq!(item.get_attr("jid").and_then(|v| v.as_jid()), Some(&lid));
        assert_eq!(
            item.get_attr("pn_jid").and_then(|v| v.as_jid()),
            Some(&phone())
        );
    }

    #[test]
    fn block_carries_saved_dhash() {
        let (_d, s) = store();
        s.write(|tx| tx.put_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1, &"abc123"))
            .unwrap();
        queue_block(&s, &phone()).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        let item = sent[0].child("item").unwrap();
        assert_eq!(attr_text(item, "dhash"), Some("abc123"));
    }

    #[test]
    fn fetch_request_omits_item_without_saved_dhash_and_includes_it_once_saved() {
        let (_d, s) = store();
        queue_fetch(&s).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        let iq = &sent[0];
        assert_eq!(attr_text(iq, "type"), Some("get"));
        assert!(iq.child("item").is_none());

        s.write(|tx| tx.put_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1, &"zz9"))
            .unwrap();
        queue_fetch(&s).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        let item = sent[0].child("item").unwrap();
        assert_eq!(attr_text(item, "dhash"), Some("zz9"));
    }

    fn fetch_reply(id: &str, dhash: &str, items: Vec<Node>) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .children(vec![Node::new("list").attr("dhash", dhash).children(items)])
    }

    #[test]
    fn ingest_fetch_replaces_the_saved_list_and_is_idempotent() {
        let (_d, s) = store();
        let id = queue_fetch(&s).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let a = Jid::parse("15555550111@s.whatsapp.net").unwrap();
        let b = Jid::parse("15555550112@s.whatsapp.net").unwrap();
        let reply = fetch_reply(
            &id,
            "d1",
            vec![
                Node::new("item")
                    .attr("jid", a.clone())
                    .attr("display_name", "Alice"),
                Node::new("item")
                    .attr("jid", b.clone())
                    .attr("action", "unblock"),
            ],
        );
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        // Idempotent: a second ingest applies nothing more.
        assert_eq!(ingest(&s).unwrap(), 0);
        let list = blocked(&s).unwrap();
        assert_eq!(list.len(), 1, "unblock item is not a blocked row");
        assert_eq!(list[0].jid, a.to_string());
        assert_eq!(list[0].display_name.as_deref(), Some("Alice"));
        assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("d1"));

        // A later fetch that drops `a` removes its saved row.
        let id2 = queue_fetch(&s).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply2 = fetch_reply(&id2, "d2", vec![]);
        super::super::actions::record_reply(&s, &reply2).unwrap();
        ingest(&s).unwrap();
        assert!(blocked(&s).unwrap().is_empty());
    }

    #[test]
    fn ingest_block_result_adds_a_row_and_unblock_removes_it() {
        let (_d, s) = store();
        let id = queue_block(&s, &phone()).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .attr("dhash", "after-block");
        super::super::actions::record_reply(&s, &reply).unwrap();
        assert_eq!(ingest(&s).unwrap(), 1);
        let list = blocked(&s).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].jid, phone().to_string());
        assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("after-block"));

        let uid = queue_unblock(&s, &phone()).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let ureply = Node::new("iq").attr("id", uid).attr("type", "result");
        super::super::actions::record_reply(&s, &ureply).unwrap();
        ingest(&s).unwrap();
        assert!(blocked(&s).unwrap().is_empty());
    }

    #[test]
    fn ingest_ignores_error_results_but_still_marks_them_seen() {
        let (_d, s) = store();
        let id = queue_block(&s, &phone()).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id)
            .attr("type", "error")
            .children(vec![Node::new("error").attr("code", "403")]);
        super::super::actions::record_reply(&s, &reply).unwrap();
        assert_eq!(ingest(&s).unwrap(), 0);
        assert!(blocked(&s).unwrap().is_empty());
        assert_eq!(ingest(&s).unwrap(), 0, "already seen");
    }
}
