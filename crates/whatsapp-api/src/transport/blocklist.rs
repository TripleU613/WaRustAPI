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
//! saves the new `dhash`. Current43.72 `8R7.CK4` requires a `list` and routes
//! literal `matched=false` into reconciliation rather than the requested change.
//! This adapter holds those results without cache/hash mutation and exposes a
//! reconciliation warning. Set results require the first child to be list and
//! validate it before branching; only its dhash is admitted. Full native migration
//! remains unported.
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
use std::collections::{BTreeMap, BTreeSet};

const BLOCKED_PREFIX: &str = "blocked:";
const DHASH_KEY: &str = "blocklist-dhash";
const RECONCILIATION_KEY: &[u8] = b"blocklist-reconciliation-required";
const INGESTED_PREFIX: &str = "blocklist-ingested:";

fn invalid(reason: &'static str) -> Error {
    Error::invalid("blocklist", reason)
}

fn redact_projection_error(error: Error) -> Error {
    match error {
        Error::Storage(crate::store::StoreError::Serialization(_)) => {
            invalid("invalid saved blocklist projection data")
        }
        other => other,
    }
}

fn blocked_key(jid: &str) -> String {
    format!("{BLOCKED_PREFIX}{jid}")
}
fn ingested_key(id: &str) -> String {
    format!("{INGESTED_PREFIX}{id}")
}

/// Drop the saved `dhash` so the next fetch or set request omits it and the
/// server answers with the full list.
pub fn forget_dhash(store: &ProtocolStore) -> Result<()> {
    store.write(|tx| {
        tx.delete(Table::Checkpoints, DHASH_KEY.as_bytes())?;
        Ok(())
    })
}

fn saved_dhash(store: &ProtocolStore) -> Result<Option<String>> {
    store.read(|tx| tx.get_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1))
}

/// Whether a set result needs reconciliation before the cache can be current.
/// This is local projection state, not a server refusal or permission to resend.
pub fn reconciliation_required(store: &ProtocolStore) -> Result<bool> {
    store
        .read(|tx| tx.get_json(Table::Checkpoints, RECONCILIATION_KEY, 1))
        .map(|saved: Option<bool>| saved.unwrap_or(false))
        .map_err(redact_projection_error)
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

// APK43.72 0Cf.A0q -> 0Fv.A01 checks Java UTF-16 char whitespace or
// space-char, independently verified across all BMP units by BlankUnits.java.
// Rust's Unicode White_Space differs at U+001C..001F and U+0085.
fn blank_hash_unit(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000d | 0x001c..=0x0020 | 0x00a0 | 0x1680
        | 0x2000..=0x200a | 0x2028..=0x2029 | 0x202f | 0x205f | 0x3000)
}

fn blank_hash(hash: &str) -> bool {
    hash.encode_utf16().all(blank_hash_unit)
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
    let dhash = saved_dhash(store)?.filter(|hash| !blank_hash(hash));
    let mut item = Node::new("item")
        .attr("action", action)
        .attr("jid", wire_jid.clone());
    // `LX/1ep.A05` 0x049c..0x0516: the `pn_jid`/`username` attributes are added
    // only on the block branch; an unblock item carries `action`, `jid` and the
    // optional `dhash`. Live 2026-09-20: an unblock sent with `pn_jid` was never
    // answered by the server.
    if lid.is_some() && action == "block" {
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

/// Durably queue a blocklist fetch. Carries a nonempty saved `dhash` verbatim.
pub fn queue_fetch(store: &ProtocolStore) -> Result<String> {
    let dhash = saved_dhash(store)?.filter(|hash| !hash.is_empty());
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

fn text_attr<'a>(node: &'a Node, name: &str) -> Result<Option<&'a str>> {
    node.get_attr(name)
        .map(|value| {
            value
                .as_text()
                .ok_or_else(|| invalid("nontext blocklist metadata"))
        })
        .transpose()
}

fn user_attr(node: &Node, name: &str) -> Result<Option<Jid>> {
    let Some(value) = node.get_attr(name) else {
        return Ok(None);
    };
    let jid = match value.as_jid() {
        Some(jid) => jid.clone(),
        None => Jid::parse(
            value
                .as_text()
                .ok_or_else(|| invalid("invalid blocklist identity"))?,
        )
        .map_err(|_| invalid("invalid blocklist identity"))?,
    };
    if !jid.is_user_like() || jid.is_ad() {
        return Err(invalid(
            "blocklist projection requires a primary user identity",
        ));
    }
    Ok(Some(jid))
}

fn require_request(request: &Node, id: &str, kind: &str) -> Result<()> {
    let request_type = if kind == "blocklist" { "get" } else { "set" };
    let recipient = request.get_attr("to").and_then(|value| {
        value
            .as_jid()
            .cloned()
            .or_else(|| value.as_text().and_then(|text| Jid::parse(text).ok()))
    });
    if request.description != "iq"
        || text_attr(request, "id")? != Some(id)
        || text_attr(request, "xmlns")? != Some("blocklist")
        || text_attr(request, "type")? != Some(request_type)
        || recipient != Some(Jid::parse("s.whatsapp.net")?)
    {
        return Err(invalid("saved blocklist request does not match its action"));
    }
    Ok(())
}

struct Snapshot {
    dhash: Option<String>,
    current: BTreeSet<String>,
    rows: BTreeMap<String, BlockedRow>,
}

fn parse_snapshot(list: &Node) -> Result<Snapshot> {
    if !matches!(
        list.content,
        crate::binary::NodeContent::Empty | crate::binary::NodeContent::Children(_)
    ) {
        return Err(invalid("malformed blocklist snapshot"));
    }
    let dhash = text_attr(list, "dhash")?.map(str::to_owned);
    let mut current = BTreeSet::new();
    let mut new_rows = BTreeMap::new();
    for item in list.child_nodes() {
        // APK43.72 6qD.A02 requires each child to be item via 0Tp.A01.
        // Ignoring unknown tags can turn a malformed list into a cache wipe.
        if item.description != "item" {
            return Err(invalid("unexpected blocklist snapshot child"));
        }
        let action = text_attr(item, "action")?.unwrap_or("block");
        let jid = match user_attr(item, "jid")? {
            Some(jid) => jid,
            None => user_attr(item, "pn_jid")?
                .ok_or_else(|| invalid("blocked snapshot item has no identity"))?,
        }
        .to_string();
        // APK43.72 6qD.A02 writes every item's action==block Boolean with
        // Map.put; 80Z derives membership from the final values. A later
        // non-block item therefore cancels an earlier block in this snapshot.
        if action != "block" {
            current.remove(&jid);
            new_rows.remove(&jid);
            continue;
        }
        current.insert(jid.clone());
        let metadata = |name| -> Result<Option<String>> {
            if item.get_attr(name).is_none() {
                return Ok(None);
            }
            attr_as_string(item, name)
                .map(Some)
                .ok_or_else(|| invalid("invalid blocklist contact metadata"))
        };
        new_rows.insert(
            jid.clone(),
            BlockedRow {
                jid,
                display_name: metadata("display_name")?,
                username: metadata("username")?,
                since: Timestamp::now(),
            },
        );
    }
    Ok(Snapshot {
        dhash,
        current,
        rows: new_rows,
    })
}

fn apply_fetch_in(tx: &mut crate::store::WriteTx<'_>, reply: &Node) -> Result<()> {
    let mut lists = reply
        .child_nodes()
        .iter()
        .filter(|node| node.description == "list");
    // Existing observed no-list response means unchanged, never an empty list.
    let Some(list) = lists.next() else {
        return Ok(());
    };
    if lists.next().is_some() {
        return Err(invalid("ambiguous blocklist snapshot"));
    }
    let Snapshot {
        dhash,
        current,
        rows: new_rows,
    } = parse_snapshot(list)?;
    // APK43.72 1dH.A0O / 8al.run case0 write the admitted snapshot
    // hash even when null. Android putString(null) removes the old value.
    if let Some(dhash) = dhash {
        tx.put_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1, &dhash)?;
    } else {
        tx.delete(Table::Checkpoints, DHASH_KEY.as_bytes())?;
    }
    for (jid, row) in new_rows {
        tx.put_json(Table::Checkpoints, blocked_key(&jid).as_bytes(), 1, &row)?;
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
    // Only an admitted full snapshot repairs an unresolved set reconciliation.
    // A no-list/unchanged reply returned above must retain the warning.
    tx.delete(Table::Checkpoints, RECONCILIATION_KEY)?;
    Ok(())
}

fn apply_block_result_in(
    tx: &mut crate::store::WriteTx<'_>,
    request: &Node,
    action: &str,
    reply: &Node,
) -> Result<bool> {
    let mut items = request
        .child_nodes()
        .iter()
        .filter(|node| node.description == "item");
    let item = items
        .next()
        .ok_or_else(|| invalid("saved block request has no item"))?;
    if items.next().is_some() || text_attr(item, "action")? != Some(action) {
        return Err(invalid("saved block request has an ambiguous action"));
    }
    let target =
        user_attr(item, "jid")?.ok_or_else(|| invalid("saved block request has no target"))?;
    // APK43.72 8R7.CK4 requires A0D's first child to be list and parses
    // it before testing matched. Trailing children cannot supply a hash or
    // change which result branch is selected.
    let list = reply
        .child_nodes()
        .first()
        .filter(|node| node.description == "list")
        .ok_or_else(|| invalid("blocklist set result requires list as its first child"))?;
    let snapshot = parse_snapshot(list)?;
    if text_attr(list, "matched")? == Some("false") {
        tx.put_json(Table::Checkpoints, RECONCILIATION_KEY, 1, &true)?;
        return Ok(false);
    }
    let key = blocked_key(&target.to_string());
    let dhash = snapshot.dhash;
    // APK43.72 7vV.A00 only reaches 1dH.A08 when add/remove of the
    // requested target changes its membership. Its backend Boolean denotes
    // execution success, not a positive affected-row count (7PB.A0Q).
    let changed = if action == "block" {
        let existing: Option<BlockedRow> = tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
        if existing
            .as_ref()
            .is_some_and(|row| row.jid != target.to_string())
        {
            return Err(invalid("saved blocked row names another identity"));
        }
        if existing.is_none() {
            let row = BlockedRow {
                jid: target.to_string(),
                display_name: None,
                username: None,
                since: Timestamp::now(),
            };
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)?;
            true
        } else {
            false
        }
    } else {
        let present = tx.get(Table::Checkpoints, key.as_bytes(), 1)?.is_some();
        if present {
            tx.delete(Table::Checkpoints, key.as_bytes())?;
        }
        present
    };
    if changed {
        if let Some(dhash) = dhash {
            tx.put_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1, &dhash)?;
        } else {
            tx.delete(Table::Checkpoints, DHASH_KEY.as_bytes())?;
        }
    }
    Ok(true)
}

fn ingest_action_in(tx: &mut crate::store::WriteTx<'_>, id: &str, sequence: u64) -> Result<usize> {
    let marker = ingested_key(id);
    if tx
        .get_json::<bool>(Table::Checkpoints, marker.as_bytes(), 1)?
        .unwrap_or(false)
    {
        return Ok(0);
    }
    let (outcome, reply, request) = super::actions::view_in_write(tx, id)?
        .ok_or_else(|| invalid("settled blocklist action disappeared"))?;
    if outcome.settlement_sequence.is_none() {
        return Ok(0);
    }
    if outcome.settlement_sequence != Some(sequence)
        || sequence == 0
        || !matches!(outcome.kind.as_str(), "blocklist" | "block" | "unblock")
        || !matches!(
            outcome.status,
            super::actions::ActionStatus::Ok | super::actions::ActionStatus::Error
        )
    {
        return Err(invalid(
            "settled blocklist action changed during projection",
        ));
    }
    require_request(&request, id, &outcome.kind)?;
    let reply = reply.ok_or_else(|| invalid("settled blocklist action has no saved reply"))?;
    if reply.description != "iq" || text_attr(&reply, "id")? != Some(id) {
        return Err(invalid("saved blocklist reply does not match its action"));
    }
    let applied = if outcome.status == super::actions::ActionStatus::Ok {
        if text_attr(&reply, "type")? != Some("result") {
            return Err(invalid("saved successful blocklist reply is not a result"));
        }
        if outcome.kind == "blocklist" {
            apply_fetch_in(tx, &reply)?;
            1
        } else {
            usize::from(apply_block_result_in(tx, &request, &outcome.kind, &reply)?)
        }
    } else {
        0
    };
    tx.put_json(Table::Checkpoints, marker.as_bytes(), 1, &true)?;
    Ok(applied)
}

/// Apply newly sequenced settled fetch/block/unblock replies in receive order.
/// Projection and once-only marker commit together. Legacy unsequenced replies
/// remain inspectable but are not replayed or marked; chronology is unknown.
/// Unmatched set results are marked inspected without applying the requested
/// change/hash, and retain a reconciliation warning until a later full snapshot.
pub fn ingest(store: &ProtocolStore) -> Result<usize> {
    let mut applied = 0;
    for (outcome, _) in super::actions::settled_for_kinds(store, &["blocklist", "block", "unblock"])
        .map_err(redact_projection_error)?
    {
        let Some(sequence) = outcome.settlement_sequence else {
            continue;
        };
        applied += store
            .write(|tx| ingest_action_in(tx, &outcome.id, sequence))
            .map_err(redact_projection_error)?;
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
    fn blank_hash_units_match_executed_java_reference_for_every_utf16_unit() {
        let expected: Vec<u16> = serde_json::from_str(include_str!(
            "../../../../docs/protocol/apk/2.26.43.72-blocklist-outbound-hash/blank-units.json"
        ))
        .unwrap();
        assert!(expected.windows(2).all(|pair| pair[0] < pair[1]));
        for unit in 0..=u16::MAX {
            assert_eq!(
                blank_hash_unit(unit),
                expected.binary_search(&unit).is_ok(),
                "UTF16 unit {unit:04x}"
            );
        }
        assert!(blank_hash(""));
        assert!(!blank_hash("😀"));
    }

    #[test]
    fn outbound_hash_presence_matches_distinct_get_and_set_predicates() {
        for (hash, get_present, set_present) in [
            (None, false, false),
            (Some(""), false, false),
            (Some(" \t\n"), true, false),
            (Some("\u{a0}"), true, false),
            (Some("\u{1c}"), true, false),
            (Some("\u{85}"), true, true),
            (Some("\u{200b}"), true, true),
            (Some("😀"), true, true),
            (Some(" hash "), true, true),
            (Some("abc"), true, true),
        ] {
            let (_dir, s) = store();
            if let Some(hash) = hash {
                s.write(|tx| tx.put_json(Table::Checkpoints, DHASH_KEY.as_bytes(), 1, &hash))
                    .unwrap();
            }
            let fetch = queue_fetch(&s).unwrap();
            let block = queue_block(&s, &phone()).unwrap();
            let unblock = queue_unblock(&s, &phone()).unwrap();
            let queued = super::super::actions::reserve_pending(&s).unwrap();
            let get = queued
                .iter()
                .find(|n| attr_text(n, "id") == Some(fetch.as_str()))
                .unwrap();
            assert_eq!(get.child("item").is_some(), get_present);
            if get_present {
                assert_eq!(attr_text(get.child("item").unwrap(), "dhash"), hash);
            }
            for id in [&block, &unblock] {
                let set = queued
                    .iter()
                    .find(|n| attr_text(n, "id") == Some(id.as_str()))
                    .unwrap();
                assert_eq!(
                    attr_text(set.child("item").unwrap(), "dhash"),
                    if set_present { hash } else { None }
                );
            }
            assert_eq!(saved_dhash(&s).unwrap().as_deref(), hash);
        }
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
    fn unmatched_set_results_never_apply_the_requested_change_or_hash_after_reopen() {
        for action in ["block", "unblock"] {
            for client_hash in [Some("before"), Some("different"), None] {
                let (dir, s) = store();
                let other = Jid::parse("15555550199@s.whatsapp.net").unwrap();
                let initial = queue_fetch(&s).unwrap();
                super::super::actions::reserve_pending(&s).unwrap();
                let mut items = vec![Node::new("item").attr("jid", other.clone())];
                if action == "unblock" {
                    items.push(Node::new("item").attr("jid", phone()));
                }
                super::super::actions::record_reply(&s, &fetch_reply(&initial, "before", items))
                    .unwrap();
                assert_eq!(ingest(&s).unwrap(), 1);
                let before = blocked(&s).unwrap();
                let cache_bytes = |store: &ProtocolStore| {
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap()
                        .into_iter()
                        .filter(|(key, _)| {
                            key.starts_with(BLOCKED_PREFIX.as_bytes())
                                || key == DHASH_KEY.as_bytes()
                        })
                        .collect::<Vec<_>>()
                };
                let before_cache = cache_bytes(&s);
                let id = if action == "block" {
                    queue_block(&s, &phone())
                } else {
                    queue_unblock(&s, &phone())
                }
                .unwrap();
                super::super::actions::reserve_pending(&s).unwrap();
                let mut list = Node::new("list")
                    .attr("matched", "false")
                    .attr("dhash", "PRIVATE-UNADOPTED-HASH");
                if let Some(hash) = client_hash {
                    list = list.attr("c_dhash", hash);
                }
                if action == "unblock" {
                    list = list.children(vec![Node::new("item").attr("jid", phone())]);
                }
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .children(vec![list]);
                assert!(super::super::actions::record_reply(&s, &reply).unwrap());
                let outcome = super::super::actions::outcomes(&s)
                    .unwrap()
                    .into_iter()
                    .find(|o| o.id == id)
                    .unwrap();
                assert_eq!(outcome.status, super::super::actions::ActionStatus::Ok);
                let head = s.journal_head().unwrap();
                let cut: Result<()> = s.write(|tx| {
                    assert_eq!(
                        ingest_action_in(tx, &id, outcome.settlement_sequence.unwrap())?,
                        0
                    );
                    Err(Error::Cancelled)
                });
                assert!(cut.is_err());
                assert!(!reconciliation_required(&s).unwrap());
                assert_eq!(s.journal_head().unwrap(), head);
                assert!(
                    s.read(|tx| tx.get(Table::Checkpoints, ingested_key(&id).as_bytes(), 1))
                        .unwrap()
                        .is_none()
                );
                assert_eq!(ingest(&s).unwrap(), 0);
                assert!(reconciliation_required(&s).unwrap());
                assert_eq!(blocked(&s).unwrap(), before);
                assert_eq!(cache_bytes(&s), before_cache);
                assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("before"));
                let head = s.journal_head().unwrap();
                assert_eq!(ingest(&s).unwrap(), 0);
                assert_eq!(s.journal_head().unwrap(), head);
                drop(s);
                let s = reopen(dir.path());
                assert_eq!(cache_bytes(&s), before_cache);
                assert!(reconciliation_required(&s).unwrap());
                assert_eq!(ingest(&s).unwrap(), 0);
                assert!(
                    super::super::actions::reserve_pending(&s)
                        .unwrap()
                        .is_empty()
                );
                // An unchanged/no-list fetch cannot repair an unresolved cache.
                let unchanged = queue_fetch(&s).unwrap();
                super::super::actions::reserve_pending(&s).unwrap();
                super::super::actions::record_reply(
                    &s,
                    &Node::new("iq").attr("id", unchanged).attr("type", "result"),
                )
                .unwrap();
                assert_eq!(ingest(&s).unwrap(), 1);
                assert!(reconciliation_required(&s).unwrap());
                // A later explicit supported snapshot replaces the cache and clears the hold.
                let refresh = queue_fetch(&s).unwrap();
                super::super::actions::reserve_pending(&s).unwrap();
                super::super::actions::record_reply(
                    &s,
                    &fetch_reply(
                        &refresh,
                        "reconciled",
                        vec![Node::new("item").attr("jid", other.clone())],
                    ),
                )
                .unwrap();
                assert_eq!(ingest(&s).unwrap(), 1);
                assert!(!reconciliation_required(&s).unwrap());
                assert_eq!(
                    blocked(&s)
                        .unwrap()
                        .iter()
                        .map(|row| row.jid.clone())
                        .collect::<Vec<_>>(),
                    vec![other.to_string()]
                );
                assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("reconciled"));
                drop(s);
                let s = reopen(dir.path());
                assert_eq!(ingest(&s).unwrap(), 0);
                assert!(!reconciliation_required(&s).unwrap());
                assert!(
                    super::super::actions::reserve_pending(&s)
                        .unwrap()
                        .is_empty()
                );
            }
        }
    }

    #[test]
    fn set_result_first_list_admission_preserves_state_and_correlation_after_reopen() {
        for action in ["block", "unblock"] {
            for mode in [
                "missing",
                "wrong_first",
                "unknown_child",
                "unknown_unmatched",
                "invalid_identity",
                "first_only",
                "root_hash_only",
                "unmatched",
            ] {
                let (dir, store) = store();
                let initial = queue_fetch(&store).unwrap();
                super::super::actions::reserve_pending(&store).unwrap();
                let initial_items = if action == "unblock" {
                    vec![Node::new("item").attr("jid", phone())]
                } else {
                    vec![]
                };
                super::super::actions::record_reply(
                    &store,
                    &fetch_reply(&initial, "before", initial_items),
                )
                .unwrap();
                assert_eq!(ingest(&store).unwrap(), 1);
                let id = if action == "block" {
                    queue_block(&store, &phone())
                } else {
                    queue_unblock(&store, &phone())
                }
                .unwrap();
                super::super::actions::reserve_pending(&store).unwrap();
                let good = Node::new("list").attr("dhash", "selected");
                let children = match mode {
                    "missing" => vec![],
                    "wrong_first" => vec![Node::new("PRIVATE-WRONG-TAG"), good],
                    "unknown_child" => vec![good.children(vec![Node::new("PRIVATE-WRONG-TAG")])],
                    "unknown_unmatched" => vec![
                        good.attr("matched", "false")
                            .children(vec![Node::new("PRIVATE-WRONG-TAG")]),
                    ],
                    "invalid_identity" => vec![
                        good.children(vec![Node::new("item").attr("jid", "PRIVATE-INVALID-JID")]),
                    ],
                    "first_only" => vec![
                        good,
                        Node::new("list")
                            .attr("matched", "false")
                            .attr("dhash", "PRIVATE-TRAILING-HASH"),
                    ],
                    "root_hash_only" => vec![Node::new("list")],
                    "unmatched" => vec![good.attr("matched", "false")],
                    _ => unreachable!(),
                };
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .attr("dhash", "PRIVATE-ROOT-HASH")
                    .children(children);
                assert!(super::super::actions::record_reply(&store, &reply).unwrap());
                let before = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                let head = store.journal_head().unwrap();
                let invalid = !matches!(mode, "first_only" | "root_hash_only" | "unmatched");
                if invalid {
                    let error = ingest(&store).unwrap_err().to_string();
                    assert!(!error.contains("PRIVATE"));
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        before
                    );
                } else {
                    assert_eq!(ingest(&store).unwrap(), usize::from(mode != "unmatched"));
                }
                drop(store);
                let store = reopen(dir.path());
                assert_eq!(store.journal_head().unwrap(), head);
                assert!(super::super::actions::reply(&store, &id).unwrap().is_some());
                let outcome = super::super::actions::outcomes(&store)
                    .unwrap()
                    .into_iter()
                    .find(|o| o.id == id)
                    .unwrap();
                assert_eq!(outcome.status, super::super::actions::ActionStatus::Ok);
                if invalid {
                    assert!(ingest(&store).is_err());
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        before
                    );
                    assert_eq!(
                        blocked(&store).unwrap().len(),
                        usize::from(action == "unblock")
                    );
                } else {
                    assert_eq!(ingest(&store).unwrap(), 0);
                    assert_eq!(
                        blocked(&store).unwrap().len(),
                        usize::from(if mode == "unmatched" {
                            action == "unblock"
                        } else {
                            action == "block"
                        })
                    );
                    assert_eq!(
                        reconciliation_required(&store).unwrap(),
                        mode == "unmatched"
                    );
                }
                let expected_hash = match mode {
                    "first_only" => Some("selected"),
                    "root_hash_only" => None,
                    _ => Some("before"),
                };
                assert_eq!(saved_dhash(&store).unwrap().as_deref(), expected_hash);
                assert!(
                    super::super::actions::reserve_pending(&store)
                        .unwrap()
                        .is_empty()
                );
            }
        }
    }

    #[test]
    fn set_hash_updates_only_when_target_membership_changes_after_reopen() {
        for action in ["block", "unblock"] {
            for present in [false, true] {
                for hash in [None, Some("updated"), Some("")] {
                    let (dir, s) = store();
                    let seed = queue_fetch(&s).unwrap();
                    super::super::actions::reserve_pending(&s).unwrap();
                    let other = Node::new("item")
                        .attr("jid", "15555550999@s.whatsapp.net")
                        .attr("display_name", "other");
                    let mut items = vec![other];
                    if present {
                        items.push(
                            Node::new("item")
                                .attr("jid", phone())
                                .attr("display_name", "saved-name"),
                        );
                    }
                    super::super::actions::record_reply(&s, &fetch_reply(&seed, "before", items))
                        .unwrap();
                    assert_eq!(ingest(&s).unwrap(), 1);
                    let cache = |store: &ProtocolStore| {
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap()
                            .into_iter()
                            .filter(|(k, _)| {
                                k.starts_with(BLOCKED_PREFIX.as_bytes())
                                    || k == DHASH_KEY.as_bytes()
                            })
                            .collect::<Vec<_>>()
                    };
                    let before = cache(&s);
                    let id = if action == "block" {
                        queue_block(&s, &phone())
                    } else {
                        queue_unblock(&s, &phone())
                    }
                    .unwrap();
                    super::super::actions::reserve_pending(&s).unwrap();
                    let mut list = Node::new("list");
                    if let Some(hash) = hash {
                        list = list.attr("dhash", hash);
                    }
                    super::super::actions::record_reply(
                        &s,
                        &Node::new("iq")
                            .attr("id", id.as_str())
                            .attr("type", "result")
                            .children(vec![list]),
                    )
                    .unwrap();
                    let head = s.journal_head().unwrap();
                    let changed = present != (action == "block");
                    assert_eq!(ingest(&s).unwrap(), 1);
                    drop(s);
                    let s = reopen(dir.path());
                    assert_eq!(
                        saved_dhash(&s).unwrap().as_deref(),
                        if changed { hash } else { Some("before") }
                    );
                    assert_eq!(
                        blocked(&s).unwrap().len(),
                        1 + usize::from(action == "block")
                    );
                    if !changed {
                        assert_eq!(cache(&s), before);
                    }
                    assert_eq!(s.journal_head().unwrap(), head);
                    assert_eq!(ingest(&s).unwrap(), 0);
                    assert!(
                        super::super::actions::reserve_pending(&s)
                            .unwrap()
                            .is_empty()
                    );
                }
            }
        }
    }

    #[test]
    fn ingest_block_result_adds_a_row_and_unblock_removes_it() {
        let (_d, s) = store();
        let id = queue_block(&s, &phone()).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .children(vec![Node::new("list").attr("dhash", "after-block")]);
        super::super::actions::record_reply(&s, &reply).unwrap();
        assert_eq!(ingest(&s).unwrap(), 1);
        let list = blocked(&s).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].jid, phone().to_string());
        assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("after-block"));

        let uid = queue_unblock(&s, &phone()).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let ureply = Node::new("iq")
            .attr("id", uid)
            .attr("type", "result")
            .children(vec![Node::new("list")]);
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

    #[test]
    fn fetch_reply_without_list_leaves_the_saved_list_alone() {
        let (_d, s) = store();
        let a = Jid::parse("15550000001@s.whatsapp.net").unwrap();
        let id = queue_fetch(&s).unwrap();
        let _ = super::super::actions::reserve_pending(&s).unwrap();
        assert!(
            super::super::actions::record_reply(
                &s,
                &fetch_reply(&id, "h1", vec![Node::new("item").attr("jid", a.clone())])
            )
            .unwrap()
        );
        assert_eq!(ingest(&s).unwrap(), 1);
        assert_eq!(blocked(&s).unwrap().len(), 1);
        // Second fetch carries the dhash; the server answers "unchanged".
        let id2 = queue_fetch(&s).unwrap();
        let _ = super::super::actions::reserve_pending(&s).unwrap();
        let unchanged = Node::new("iq")
            .attr("id", id2.as_str())
            .attr("type", "result");
        assert!(super::super::actions::record_reply(&s, &unchanged).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        assert_eq!(
            blocked(&s).unwrap().len(),
            1,
            "unchanged reply keeps the list"
        );
        forget_dhash(&s).unwrap();
        assert!(saved_dhash(&s).unwrap().is_none());
    }

    fn fixed_id(store: &ProtocolStore, id: &str, desired: &str) -> String {
        use crate::binary::{Encoder, TokenDictionary};
        let request = super::super::actions::request(store, id)
            .unwrap()
            .unwrap()
            .attr("id", desired);
        let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&request)
            .unwrap();
        store
            .write(|tx| {
                let old = format!("action-outbox:{id}");
                let mut row: serde_json::Value =
                    tx.get_json(Table::Checkpoints, old.as_bytes(), 1)?.unwrap();
                row["wire"] = serde_json::json!(wire);
                tx.delete(Table::Checkpoints, old.as_bytes())?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("action-outbox:{desired}").as_bytes(),
                    1,
                    &row,
                )
            })
            .unwrap();
        desired.to_owned()
    }

    fn reopen(dir: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: dir.join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: false,
        })
        .unwrap()
    }

    #[test]
    fn cross_kind_projection_follows_receive_order_after_reopen() {
        let (dir, s) = store();
        let block = fixed_id(&s, &queue_block(&s, &phone()).unwrap(), &"f".repeat(32));
        let unblock = fixed_id(&s, &queue_unblock(&s, &phone()).unwrap(), &"e".repeat(32));
        let fetch = fixed_id(&s, &queue_fetch(&s).unwrap(), &"0".repeat(32));
        super::super::actions::reserve_pending(&s).unwrap();
        // IDs sort in the opposite order to receipt order; kind partitioning
        // would replay this authoritative final fetch before the older unblock.
        for reply in [
            Node::new("iq")
                .attr("type", "result")
                .attr("id", block.as_str())
                .children(vec![Node::new("list")]),
            Node::new("iq")
                .attr("type", "result")
                .attr("id", unblock.as_str())
                .children(vec![Node::new("list")]),
            fetch_reply(
                &fetch,
                "latest",
                vec![Node::new("item").attr("jid", phone())],
            ),
        ] {
            assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        }
        // A stale target sidecar must never override the immutable wire target.
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                format!("blocklist-target:{block}").as_bytes(),
                1,
                &Jid::parse("15555550999@s.whatsapp.net").unwrap(),
            )
        })
        .unwrap();
        drop(s);
        let s = reopen(dir.path());
        assert_eq!(ingest(&s).unwrap(), 3);
        assert_eq!(
            blocked(&s)
                .unwrap()
                .iter()
                .map(|row| row.jid.clone())
                .collect::<Vec<_>>(),
            vec![phone().to_string()]
        );
        assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("latest"));
        assert_eq!(ingest(&s).unwrap(), 0);
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn full_snapshot_without_hash_clears_stale_request_hash_after_reopen() {
        for mode in ["same", "empty", "no_list", "malformed"] {
            let (dir, s) = store();
            let seed = queue_fetch(&s).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            super::super::actions::record_reply(
                &s,
                &fetch_reply(&seed, "stale", vec![Node::new("item").attr("jid", phone())]),
            )
            .unwrap();
            assert_eq!(ingest(&s).unwrap(), 1);
            s.write(|tx| tx.put_json(Table::Checkpoints, RECONCILIATION_KEY, 1, &true))
                .unwrap();
            let id = queue_fetch(&s).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let children = match mode {
                "same" => {
                    vec![Node::new("list").children(vec![Node::new("item").attr("jid", phone())])]
                }
                "empty" => vec![Node::new("list")],
                "no_list" => vec![],
                "malformed" => {
                    vec![Node::new("list").children(vec![Node::new("PRIVATE-INVALID-TAG")])]
                }
                _ => unreachable!(),
            };
            super::super::actions::record_reply(
                &s,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .children(children),
            )
            .unwrap();
            let before = s.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap();
            let head = s.journal_head().unwrap();
            if mode == "malformed" {
                assert!(ingest(&s).is_err());
                assert_eq!(
                    s.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap(),
                    before
                );
            } else {
                assert_eq!(ingest(&s).unwrap(), 1);
            }
            drop(s);
            let s = reopen(dir.path());
            let cleared = matches!(mode, "same" | "empty");
            assert_eq!(
                saved_dhash(&s).unwrap().as_deref(),
                if cleared { None } else { Some("stale") }
            );
            assert_eq!(reconciliation_required(&s).unwrap(), !cleared);
            assert_eq!(blocked(&s).unwrap().len(), usize::from(mode != "empty"));
            assert_eq!(s.journal_head().unwrap(), head);
            if mode == "malformed" {
                assert!(ingest(&s).is_err());
            } else {
                assert_eq!(ingest(&s).unwrap(), 0);
            }
            assert!(
                super::super::actions::reserve_pending(&s)
                    .unwrap()
                    .is_empty()
            );
            // Explicit offline queue controls prove future construction uses
            // the committed checkpoint, without transmitting either request.
            let fetch = queue_fetch(&s).unwrap();
            let block = queue_block(&s, &phone()).unwrap();
            let queued = super::super::actions::reserve_pending(&s).unwrap();
            let fetch = queued
                .iter()
                .find(|n| attr_text(n, "id") == Some(fetch.as_str()))
                .unwrap();
            assert_eq!(fetch.child("item").is_none(), cleared);
            let block = queued
                .iter()
                .find(|n| attr_text(n, "id") == Some(block.as_str()))
                .unwrap();
            assert_eq!(
                attr_text(block.child("item").unwrap(), "dhash"),
                if cleared { None } else { Some("stale") }
            );
        }
    }

    #[test]
    fn snapshot_replay_is_independent_of_id_sort_direction() {
        for (old_id, new_id) in [
            ("0".repeat(32), "f".repeat(32)),
            ("f".repeat(32), "0".repeat(32)),
        ] {
            let (dir, s) = store();
            let old = fixed_id(&s, &queue_fetch(&s).unwrap(), &old_id);
            let new = fixed_id(&s, &queue_fetch(&s).unwrap(), &new_id);
            super::super::actions::reserve_pending(&s).unwrap();
            let stale = Jid::parse("15555550199@s.whatsapp.net").unwrap();
            super::super::actions::record_reply(
                &s,
                &fetch_reply(&old, "old", vec![Node::new("item").attr("jid", stale)]),
            )
            .unwrap();
            super::super::actions::record_reply(
                &s,
                &fetch_reply(&new, "new", vec![Node::new("item").attr("jid", phone())]),
            )
            .unwrap();
            drop(s);
            let s = reopen(dir.path());
            assert_eq!(ingest(&s).unwrap(), 2);
            assert_eq!(blocked(&s).unwrap()[0].jid, phone().to_string());
            assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("new"));
            assert_eq!(ingest(&s).unwrap(), 0);
        }
    }

    #[test]
    fn projection_and_marker_roll_back_together_before_commit() {
        let (dir, s) = store();
        let id = queue_fetch(&s).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        super::super::actions::record_reply(
            &s,
            &fetch_reply(&id, "hash", vec![Node::new("item").attr("jid", phone())]),
        )
        .unwrap();
        let outcome = super::super::actions::outcomes(&s).unwrap().pop().unwrap();
        let result: Result<()> = s.write(|tx| {
            assert_eq!(
                ingest_action_in(tx, &id, outcome.settlement_sequence.unwrap())?,
                1
            );
            // Simulate a cut after projection and marker writes but before the
            // parent store transaction commits. No nested commit can survive.
            Err(Error::Cancelled)
        });
        assert!(result.is_err());
        assert!(blocked(&s).unwrap().is_empty());
        assert!(saved_dhash(&s).unwrap().is_none());
        assert!(
            s.read(|tx| tx.get(Table::Checkpoints, ingested_key(&id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        drop(s);
        let s = reopen(dir.path());
        assert!(blocked(&s).unwrap().is_empty());
        assert!(saved_dhash(&s).unwrap().is_none());
        assert_eq!(ingest(&s).unwrap(), 1);
        assert_eq!(ingest(&s).unwrap(), 0);
    }

    #[test]
    fn snapshot_rejects_unknown_children_without_erasing_cache_or_reconciliation_after_reopen() {
        for mode in [
            "unknown_only",
            "unknown_first",
            "unknown_last",
            "nested_list",
            "valid_empty",
            "valid_item",
        ] {
            let (dir, s) = store();
            let initial = queue_fetch(&s).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            super::super::actions::record_reply(
                &s,
                &fetch_reply(
                    &initial,
                    "before",
                    vec![Node::new("item").attr("jid", phone())],
                ),
            )
            .unwrap();
            assert_eq!(ingest(&s).unwrap(), 1);
            let change = queue_unblock(&s, &phone()).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let unmatched = Node::new("iq")
                .attr("id", change)
                .attr("type", "result")
                .children(vec![Node::new("list").attr("matched", "false")]);
            super::super::actions::record_reply(&s, &unmatched).unwrap();
            assert_eq!(ingest(&s).unwrap(), 0);
            assert!(reconciliation_required(&s).unwrap());
            let fetch = queue_fetch(&s).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let valid = Node::new("item").attr("jid", "15555550199@s.whatsapp.net");
            let unknown =
                Node::new("PRIVATE-UNSUPPORTED-TAG").attr("jid", "15555550999@s.whatsapp.net");
            let children = match mode {
                "unknown_only" => vec![unknown],
                "unknown_first" => vec![unknown, valid],
                "unknown_last" => vec![valid, unknown],
                "nested_list" => vec![Node::new("list").children(vec![valid])],
                "valid_empty" => vec![],
                "valid_item" => vec![valid],
                _ => unreachable!(),
            };
            super::super::actions::record_reply(&s, &fetch_reply(&fetch, "after", children))
                .unwrap();
            let before = s.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap();
            let head = s.journal_head().unwrap();
            let malformed = mode.starts_with("unknown") || mode == "nested_list";
            if malformed {
                let error = ingest(&s).unwrap_err().to_string();
                assert!(error.contains("unexpected blocklist snapshot child"));
                assert!(!error.contains("PRIVATE-UNSUPPORTED-TAG"));
                assert!(!error.contains("15555550999"));
                assert_eq!(
                    s.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap(),
                    before
                );
                assert_eq!(s.journal_head().unwrap(), head);
                assert!(reconciliation_required(&s).unwrap());
            } else {
                assert_eq!(ingest(&s).unwrap(), 1);
                assert!(!reconciliation_required(&s).unwrap());
                assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("after"));
                assert_eq!(
                    blocked(&s).unwrap().len(),
                    usize::from(mode == "valid_item")
                );
            }
            drop(s);
            let s = reopen(dir.path());
            if malformed {
                assert!(ingest(&s).is_err());
                assert_eq!(
                    s.read(|tx| tx.scan(Table::Checkpoints, 1, 100)).unwrap(),
                    before
                );
                assert_eq!(s.journal_head().unwrap(), head);
                assert!(reconciliation_required(&s).unwrap());
                assert_eq!(blocked(&s).unwrap()[0].jid, phone().to_string());
                assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("before"));
                assert!(
                    s.read(|tx| tx.get(Table::Checkpoints, ingested_key(&fetch).as_bytes(), 1))
                        .unwrap()
                        .is_none()
                );
            } else {
                assert_eq!(ingest(&s).unwrap(), 0);
                assert!(!reconciliation_required(&s).unwrap());
            }
            assert!(
                super::super::actions::reserve_pending(&s)
                    .unwrap()
                    .is_empty()
            );
        }
    }

    #[test]
    fn malformed_snapshot_is_refused_without_partial_list_hash_or_marker() {
        let (_dir, s) = store();
        let original = queue_block(&s, &phone()).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        super::super::actions::record_reply(
            &s,
            &Node::new("iq")
                .attr("id", original)
                .attr("type", "result")
                .children(vec![Node::new("list").attr("dhash", "before")]),
        )
        .unwrap();
        ingest(&s).unwrap();
        let before = blocked(&s).unwrap();
        let id = queue_fetch(&s).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        let reply = fetch_reply(
            &id,
            "after",
            vec![
                Node::new("item").attr("jid", "15555550998@s.whatsapp.net"),
                Node::new("item").attr("jid", "private-invalid-target"),
            ],
        );
        super::super::actions::record_reply(&s, &reply).unwrap();
        let error = ingest(&s).unwrap_err().to_string();
        assert!(!error.contains("private-invalid-target"));
        assert!(!error.contains("15555550998"));
        assert_eq!(blocked(&s).unwrap(), before);
        assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("before"));
        assert!(
            s.read(|tx| tx.get(Table::Checkpoints, ingested_key(&id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn repeated_snapshot_identity_uses_last_membership_after_reopen() {
        for (actions, expected) in [
            (vec!["block", "unblock"], false),
            (vec!["block", "block"], true),
            (vec!["unblock", "block"], true),
            (vec!["block", "other"], false),
            (vec!["block", "unblock", "block"], true),
        ] {
            let (dir, s) = store();
            let id = queue_fetch(&s).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let items = actions
                .iter()
                .enumerate()
                .map(|(index, action)| {
                    Node::new("item")
                        .attr("jid", phone().to_string())
                        .attr("action", *action)
                        .attr("display_name", format!("synthetic-{index}"))
                })
                .collect();
            super::super::actions::record_reply(&s, &fetch_reply(&id, "last-write", items))
                .unwrap();
            let journal = s.journal_head().unwrap();
            assert_eq!(ingest(&s).unwrap(), 1);
            assert_eq!(blocked(&s).unwrap().len(), usize::from(expected));
            assert_eq!(saved_dhash(&s).unwrap().as_deref(), Some("last-write"));
            assert_eq!(ingest(&s).unwrap(), 0);
            drop(s);
            let s = ProtocolStore::open(OpenOptions {
                path: dir.path().join("p.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
                create: false,
            })
            .unwrap();
            assert_eq!(blocked(&s).unwrap().len(), usize::from(expected));
            if expected {
                assert_eq!(
                    blocked(&s).unwrap()[0].display_name.as_deref(),
                    Some(format!("synthetic-{}", actions.len() - 1).as_str())
                );
            }
            assert_eq!(ingest(&s).unwrap(), 0);
            assert_eq!(s.journal_head().unwrap(), journal);
            assert!(
                super::super::actions::reserve_pending(&s)
                    .unwrap()
                    .is_empty()
            );
        }
    }

    #[test]
    fn stored_projection_decode_errors_do_not_echo_private_values() {
        let (_dir, s) = store();
        let id = queue_block(&s, &phone()).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        super::super::actions::record_reply(
            &s,
            &Node::new("iq")
                .attr("type", "result")
                .attr("id", id.as_str())
                .children(vec![Node::new("list")]),
        )
        .unwrap();
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                blocked_key(&phone().to_string()).as_bytes(),
                1,
                &serde_json::json!({"jid":phone().to_string(),"since":"private-stored-value"}),
            )
        })
        .unwrap();
        let before = s
            .read(|tx| {
                tx.get(
                    Table::Checkpoints,
                    blocked_key(&phone().to_string()).as_bytes(),
                    1,
                )
            })
            .unwrap();
        let error = ingest(&s).unwrap_err().to_string();
        assert!(error.contains("invalid saved blocklist projection data"));
        assert!(!error.contains("private-stored-value"));
        assert_eq!(
            s.read(|tx| tx.get(
                Table::Checkpoints,
                blocked_key(&phone().to_string()).as_bytes(),
                1
            ))
            .unwrap(),
            before
        );
        assert!(
            s.read(|tx| tx.get(Table::Checkpoints, ingested_key(&id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn legacy_unsequenced_reply_is_not_projected_marked_or_resent() {
        let (dir, s) = store();
        let id = queue_fetch(&s).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        super::super::actions::record_reply(
            &s,
            &fetch_reply(&id, "legacy", vec![Node::new("item").attr("jid", phone())]),
        )
        .unwrap();
        s.write(|tx| {
            let key = format!("action-outbox:{id}");
            let mut row: serde_json::Value =
                tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
            row.as_object_mut().unwrap().remove("settlement_sequence");
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)
        })
        .unwrap();
        drop(s);
        let s = reopen(dir.path());
        assert_eq!(ingest(&s).unwrap(), 0);
        assert!(blocked(&s).unwrap().is_empty());
        assert!(saved_dhash(&s).unwrap().is_none());
        assert!(
            s.read(|tx| tx.get(Table::Checkpoints, ingested_key(&id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        assert!(super::super::actions::reply(&s, &id).unwrap().is_some());
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
    }
}
