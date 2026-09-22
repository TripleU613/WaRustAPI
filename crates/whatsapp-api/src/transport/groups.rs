//! Group management over `xmlns="w:g2"` (docs/protocol/groups.md): requests
//! are queued on the durable action queue and their replies are ingested into
//! `group:<jid>` snapshots plus `GroupCreated`/`GroupUpdated` journal events.
//! Group messages themselves (sender keys) are a separate slice.
use crate::{
    Error, Result,
    binary::Node,
    model::{
        ids::{ContactId, GroupId},
        jid::{Jid, Server},
        time::Timestamp,
    },
    store::{ProtocolStore, Table},
    sync::{Group, GroupUpdate, SyncEvent},
};
use serde::{Deserialize, Serialize};

const XMLNS: &str = "w:g2";
const INGESTED_PREFIX: &str = "groups-ingested:";

fn invalid(msg: &'static str) -> Error {
    Error::invalid("group", msg)
}

fn group_server() -> Result<Jid> {
    Jid::parse("g.us")
}

fn require_group(jid: &Jid) -> Result<Jid> {
    if jid.server != Server::Group {
        return Err(invalid("not a group JID"));
    }
    Ok(jid.clone())
}

fn require_user(jid: &Jid) -> Result<Jid> {
    if !jid.is_user_like() || jid.device != 0 {
        return Err(invalid(
            "participants must be phone-number or LID identities",
        ));
    }
    Ok(jid.to_non_ad())
}

fn iq(id: &str, kind: &str, to: Jid, children: Vec<Node>) -> Node {
    Node::new("iq")
        .attr("id", id)
        .attr("xmlns", XMLNS)
        .attr("type", kind)
        .attr("to", to)
        .children(children)
}

fn participants(jids: &[Jid]) -> Result<Vec<Node>> {
    if jids.is_empty() || jids.len() > 256 {
        return Err(invalid("between one and 256 participants"));
    }
    jids.iter()
        .map(|j| Ok(Node::new("participant").attr("jid", require_user(j)?)))
        .collect()
}

fn queue(
    store: &ProtocolStore,
    kind: &str,
    label: &str,
    node: impl FnOnce(&str) -> Result<Node>,
) -> Result<String> {
    super::actions::queue(store, kind, label, node)
}

/// `<create subject key><participant/>…</create>` to the group server
/// (`LX/AWf.A00`). Returns the action id; the reply names the new group.
pub fn queue_create(store: &ProtocolStore, subject: &str, members: &[Jid]) -> Result<String> {
    if subject.trim().is_empty() || subject.chars().count() > 100 {
        return Err(invalid("subject must be 1..=100 characters"));
    }
    let members = participants(members)?;
    let key = super::messaging::new_message_id()?;
    let subject = subject.to_owned();
    queue(store, "group-create", "create group", move |id| {
        Ok(iq(
            id,
            "set",
            group_server()?,
            vec![
                Node::new("create")
                    .attr("subject", subject)
                    .attr("key", key)
                    .children(members),
            ],
        ))
    })
}

/// `<query request="interactive"/>` (`GetGroupInfoProtocolHelper.A00`).
pub fn queue_info(store: &ProtocolStore, group: &Jid) -> Result<String> {
    let group = require_group(group)?;
    queue(store, "group-info", "group info", move |id| {
        info_request(&group, id)
    })
}

/// The same group-info query as a plain node, for a caller holding its own
/// connection rather than draining the durable action queue. A fan-out has to
/// know the membership before it can address anything, and it needs the answer
/// inside the one bounded connection it is allowed.
pub fn info_request(group: &Jid, id: &str) -> Result<Node> {
    Ok(iq(
        id,
        "get",
        require_group(group)?,
        vec![Node::new("query").attr("request", "interactive")],
    ))
}

/// `<subject>text</subject>` (`LX/0Xm.A0E`).
pub fn queue_subject(store: &ProtocolStore, group: &Jid, subject: &str) -> Result<String> {
    let group = require_group(group)?;
    if subject.trim().is_empty() || subject.chars().count() > 100 {
        return Err(invalid("subject must be 1..=100 characters"));
    }
    let subject = subject.to_owned();
    queue(store, "group-subject", "set subject", move |id| {
        Ok(iq(
            id,
            "set",
            group,
            vec![Node::new("subject").text(subject)],
        ))
    })
}

/// `<description id [prev]><body>text</body></description>`; an empty text
/// deletes (`delete="true"`). Shape inferred from the public protocol.
pub fn queue_description(
    store: &ProtocolStore,
    group: &Jid,
    text: &str,
    previous_id: Option<&str>,
) -> Result<String> {
    let group = require_group(group)?;
    if text.chars().count() > 2048 {
        return Err(invalid("description must be at most 2048 characters"));
    }
    let text = text.to_owned();
    let previous = previous_id.map(str::to_owned);
    queue(store, "group-description", "set description", move |id| {
        let mut description = Node::new("description");
        if let Some(prev) = &previous {
            description = description.attr("prev", prev.as_str());
        }
        let description = if text.trim().is_empty() {
            description.attr("delete", "true")
        } else {
            description
                .attr("id", super::messaging::new_message_id()?)
                .children(vec![Node::new("body").text(text)])
        };
        Ok(iq(id, "set", group, vec![description]))
    })
}

fn queue_membership(
    store: &ProtocolStore,
    group: &Jid,
    action: &'static str,
    members: &[Jid],
) -> Result<String> {
    let group = require_group(group)?;
    let members = participants(members)?;
    queue(
        store,
        &format!("group-{action}"),
        &format!("{action} participants"),
        move |id| {
            Ok(iq(
                id,
                "set",
                group,
                vec![Node::new(action).children(members)],
            ))
        },
    )
}

/// `<add><participant/>…</add>` (`LX/0Xm.A03`).
pub fn queue_add(store: &ProtocolStore, group: &Jid, members: &[Jid]) -> Result<String> {
    queue_membership(store, group, "add", members)
}
/// `<remove><participant/>…</remove>` (public protocol; `remove` is a token).
pub fn queue_remove(store: &ProtocolStore, group: &Jid, members: &[Jid]) -> Result<String> {
    queue_membership(store, group, "remove", members)
}
/// `<promote>…</promote>` (`LX/9Ms.A0Z`).
pub fn queue_promote(store: &ProtocolStore, group: &Jid, members: &[Jid]) -> Result<String> {
    queue_membership(store, group, "promote", members)
}
/// `<demote>…</demote>` (`LX/9Ms.A0Z`).
pub fn queue_demote(store: &ProtocolStore, group: &Jid, members: &[Jid]) -> Result<String> {
    queue_membership(store, group, "demote", members)
}

/// `<leave><group id/></leave>` to the group server (`LX/0Xm.A0C`).
pub fn queue_leave(store: &ProtocolStore, group: &Jid) -> Result<String> {
    let group = require_group(group)?;
    queue(store, "group-leave", "leave group", move |id| {
        Ok(iq(
            id,
            "set",
            group_server()?,
            vec![Node::new("leave").children(vec![Node::new("group").attr("id", group)])],
        ))
    })
}

/// `<invite/>`: `get` returns the current invite code, `set` revokes it and
/// returns a new one (`get` inferred from `LX/EnG.A00`; `set` public protocol).
pub fn queue_invite(store: &ProtocolStore, group: &Jid, revoke: bool) -> Result<String> {
    let group = require_group(group)?;
    let kind = if revoke { "set" } else { "get" };
    queue(
        store,
        "group-invite",
        if revoke {
            "revoke invite link"
        } else {
            "invite link"
        },
        move |id| Ok(iq(id, kind, group, vec![Node::new("invite")])),
    )
}

/// One participant of a stored snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    /// Participant JID as sent (phone-number or LID).
    pub jid: String,
    /// `admin`, `superadmin` or absent.
    pub role: Option<String>,
}

/// Stored snapshot of a group (`group:<jid>`), built from `group` replies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupSnapshot {
    /// Group JID.
    pub jid: String,
    /// Subject.
    pub subject: Option<String>,
    /// Subject change time (`s_t`, seconds).
    pub subject_time: Option<u64>,
    /// Subject owner (`s_o`).
    pub subject_owner: Option<String>,
    /// Creator (`creator`).
    pub creator: Option<String>,
    /// Creation time (`creation`, seconds).
    pub creation: Option<u64>,
    /// Participants with roles.
    pub members: Vec<Member>,
    /// Current invite code, when fetched.
    pub invite_code: Option<String>,
    /// Last update.
    pub updated_at: Timestamp,
}

fn attr_string(node: &Node, name: &str) -> Option<String> {
    let value = node.get_attr(name)?;
    value
        .as_text()
        .map(str::to_owned)
        .or_else(|| value.as_jid().map(|j| j.to_string()))
}

fn attr_u64(node: &Node, name: &str) -> Option<u64> {
    node.get_attr(name)?.as_text()?.parse().ok()
}

fn key(jid: &str) -> String {
    format!("group:{jid}")
}

/// Parse a `group` element into a snapshot.
pub fn parse_group(node: &Node) -> Option<GroupSnapshot> {
    if node.description != "group" {
        return None;
    }
    let jid = attr_string(node, "id")?;
    let jid = if jid.contains('@') {
        jid
    } else {
        format!("{jid}@g.us")
    };
    let members = node
        .child_nodes()
        .iter()
        .filter(|c| c.description == "participant")
        .filter_map(|c| {
            Some(Member {
                jid: attr_string(c, "jid")?,
                role: attr_string(c, "type"),
            })
        })
        .collect();
    Some(GroupSnapshot {
        jid,
        subject: attr_string(node, "subject"),
        subject_time: attr_u64(node, "s_t"),
        subject_owner: attr_string(node, "s_o"),
        creator: attr_string(node, "creator"),
        creation: attr_u64(node, "creation"),
        members,
        invite_code: None,
        updated_at: Timestamp::now(),
    })
}

/// Save the snapshot from a `w:g2` info reply obtained outside the durable
/// action queue, emitting the same `GroupCreated`/`GroupUpdated` events the
/// queued path does. Returns false when the reply carries no `group` element.
///
/// A fan-out needs the membership inside the one connection it is allowed, so
/// it issues [`info_request`] itself; the result must still land in the same
/// snapshot the rest of the client reads.
pub fn record_info_reply(store: &ProtocolStore, reply: &Node) -> Result<bool> {
    let Some(group) = reply.child("group").and_then(parse_group) else {
        return Ok(false);
    };
    save_snapshot(store, group)?;
    Ok(true)
}

fn to_model(snapshot: &GroupSnapshot) -> Option<Group> {
    let id = GroupId::from_jid(&Jid::parse(&snapshot.jid).ok()?).ok()?;
    let participants = snapshot
        .members
        .iter()
        .filter_map(|m| ContactId::from_jid(&Jid::parse(&m.jid).ok()?).ok())
        .collect();
    Some(Group {
        id,
        subject: snapshot.subject.clone(),
        participants,
        revision: None,
    })
}

/// Apply every settled group action once: `group` replies replace the
/// snapshot (and emit `GroupCreated` for a new one or `GroupUpdated` for a
/// changed subject or membership); `invite` replies save the code; refused
/// actions are only marked seen.
pub fn ingest(store: &ProtocolStore) -> Result<usize> {
    let mut applied = 0usize;
    for kind in [
        "group-create",
        "group-info",
        "group-subject",
        "group-description",
        "group-add",
        "group-remove",
        "group-promote",
        "group-demote",
        "group-leave",
        "group-invite",
    ] {
        for (outcome, reply) in super::actions::settled(store, kind)? {
            let marker = format!("{INGESTED_PREFIX}{}", outcome.id);
            let seen: Option<bool> =
                store.read(|tx| tx.get_json(Table::Checkpoints, marker.as_bytes(), 1))?;
            if seen.unwrap_or(false) {
                continue;
            }
            if outcome.status == super::actions::ActionStatus::Ok
                && let Some(reply) = &reply
            {
                apply_reply(store, kind, &outcome.id, reply)?;
            }
            store.write(|tx| tx.put_json(Table::Checkpoints, marker.as_bytes(), 1, &true))?;
            applied += 1;
        }
    }
    Ok(applied)
}

fn apply_reply(store: &ProtocolStore, kind: &str, action_id: &str, reply: &Node) -> Result<()> {
    if let Some(group) = reply.child("group").and_then(parse_group) {
        return save_snapshot(store, group);
    }
    if kind == "group-invite" {
        let Some(code) = reply.child("invite").and_then(|n| attr_string(n, "code")) else {
            return Ok(());
        };
        // The request's `to` is the group.
        let Some(request) = super::actions::request(store, action_id)? else {
            return Ok(());
        };
        let Some(group) = attr_string(&request, "to") else {
            return Ok(());
        };
        return store.write(|tx| {
            let mut snapshot: GroupSnapshot = tx
                .get_json(Table::Checkpoints, key(&group).as_bytes(), 1)?
                .unwrap_or(GroupSnapshot {
                    jid: group.clone(),
                    subject: None,
                    subject_time: None,
                    subject_owner: None,
                    creator: None,
                    creation: None,
                    members: Vec::new(),
                    invite_code: None,
                    updated_at: Timestamp::now(),
                });
            snapshot.invite_code = Some(code);
            snapshot.updated_at = Timestamp::now();
            tx.put_json(Table::Checkpoints, key(&group).as_bytes(), 1, &snapshot)
        });
    }
    if kind == "group-leave" {
        // The request names the group inside `leave/group@id`.
        if let Some(request) = super::actions::request(store, action_id)?
            && let Some(group) = request
                .child("leave")
                .and_then(|l| l.child("group"))
                .and_then(|g| attr_string(g, "id"))
        {
            store.write(|tx| {
                tx.delete(Table::Checkpoints, key(&group).as_bytes())?;
                Ok(())
            })?;
        }
    }
    Ok(())
}

fn save_snapshot(store: &ProtocolStore, mut group: GroupSnapshot) -> Result<()> {
    store.write(|tx| {
        let previous: Option<GroupSnapshot> =
            tx.get_json(Table::Checkpoints, key(&group.jid).as_bytes(), 1)?;
        if let Some(previous) = &previous {
            group.invite_code = previous.invite_code.clone();
        }
        let Some(model) = to_model(&group) else {
            return Err(invalid("group reply carries an invalid JID"));
        };
        match previous {
            None => {
                tx.append_event(SyncEvent::GroupCreated(model))?;
            }
            Some(previous) => {
                if previous.subject != group.subject {
                    tx.append_event(SyncEvent::GroupUpdated(GroupUpdate::Subject {
                        id: model.id.clone(),
                        subject: group.subject.clone(),
                    }))?;
                }
                let before: std::collections::BTreeSet<_> =
                    previous.members.iter().map(|m| m.jid.clone()).collect();
                let after: std::collections::BTreeSet<_> =
                    group.members.iter().map(|m| m.jid.clone()).collect();
                let added: Vec<ContactId> = after
                    .difference(&before)
                    .filter_map(|j| ContactId::from_jid(&Jid::parse(j).ok()?).ok())
                    .collect();
                let removed: Vec<ContactId> = before
                    .difference(&after)
                    .filter_map(|j| ContactId::from_jid(&Jid::parse(j).ok()?).ok())
                    .collect();
                if !added.is_empty() {
                    tx.append_event(SyncEvent::GroupUpdated(GroupUpdate::ParticipantsAdded {
                        id: model.id.clone(),
                        participants: added,
                    }))?;
                }
                if !removed.is_empty() {
                    tx.append_event(SyncEvent::GroupUpdated(GroupUpdate::ParticipantsRemoved {
                        id: model.id.clone(),
                        participants: removed,
                    }))?;
                }
            }
        }
        tx.put_json(Table::Checkpoints, key(&group.jid).as_bytes(), 1, &group)
    })
}

/// All stored group snapshots.
pub fn list(store: &ProtocolStore) -> Result<Vec<GroupSnapshot>> {
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                if !key.starts_with(b"group:") {
                    continue;
                }
                if let Some(snapshot) = tx.get_json::<GroupSnapshot>(Table::Checkpoints, &key, 1)? {
                    out.push(snapshot);
                }
            }
            if done {
                return Ok(out);
            }
        }
    })
}

/// One stored snapshot.
pub fn get(store: &ProtocolStore, group: &Jid) -> Result<Option<GroupSnapshot>> {
    let group = require_group(group)?.to_string();
    store.read(|tx| tx.get_json(Table::Checkpoints, key(&group).as_bytes(), 1))
}

/// Participant hash (`phash`) for a group send.
///
/// Comparison 2.26.37.71 `LX/1jL->A02(Collection<Jid>) -> String`
/// (**confirmed**): take each JID's raw string, sort the strings with natural
/// ordering, feed them to **SHA-1** concatenated with **no separator**, take the
/// **first 6 bytes** of the 20-byte digest, Base64-encode them with the standard
/// alphabet and `NO_WRAP`, and prefix `"1:"`.
///
/// Self-consistency check: six bytes encode to exactly eight Base64 characters
/// with no padding, which with the two-character prefix is exactly the ten
/// characters `LX/0hp->A06(value, 0, 10, …)` bounds `phash` to in the emitter
/// `LX/BpS`. The width matching the emitter's own bound is independent
/// corroboration that this is the right derivation.
///
/// **The membership set is the caller's problem and is currently unknown.**
/// `A02` does not normalize: it hashes whatever `getRawString()` yields, so the
/// JID form is decided by the caller. The APK has two lazily cached hashes
/// (`A0J` over `A0A.keySet()`, `A0L` over `A0B.keySet()`), matching
/// `SendE2EMessageJob`'s `groupParticipantHash` / `groupParticipantUserHash`
/// pair and its `useParticipantUserHash` flag — but which map is device-keyed
/// versus user-keyed, and whether the sender itself is a member, were **not**
/// established. A wrong membership set produces a wrong six-byte prefix and a
/// refused send, so nothing here picks a set.
pub fn participant_hash(jids: &[String]) -> String {
    use sha1::{Digest, Sha1};
    let mut sorted: Vec<&str> = jids.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    let mut hasher = Sha1::new();
    for jid in &sorted {
        hasher.update(jid.as_bytes());
    }
    let digest = hasher.finalize();
    let mut out = String::from("1:");
    out.push_str(&base64_no_wrap(&digest[..6]));
    out
}

/// Standard-alphabet Base64 without padding-free wrapping, matching Android's
/// `Base64.encodeToString(bytes, NO_WRAP)`. Six input bytes divide evenly into
/// four-character groups, so no padding is produced for this caller.
fn base64_no_wrap(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let take = chunk.len() + 1;
        for i in 0..take {
            out.push(ALPHABET[((n >> (18 - 6 * i)) & 0x3f) as usize] as char);
        }
        for _ in take..4 {
            out.push('=');
        }
    }
    out
}

/// One recipient device and the sender-key distribution ciphertext addressed
/// to it.
#[derive(Debug, Clone)]
pub struct FanoutTarget {
    /// The device this `<to>` addresses.
    pub device: Jid,
    /// `pkmsg` for a new session, `msg` for an established one, exactly as the
    /// direct path reports it.
    pub kind: &'static str,
    /// The encrypted sender-key distribution message for this device.
    pub ciphertext: Vec<u8>,
}

/// Build the outgoing sender-key stanza for a group or broadcast chat.
///
/// Structure, **confirmed** from comparison 2.26.37.71:
///
/// ```text
/// <message to=<chat> id=<id> type=<type>>
///   <participants>
///     <to jid=<device>><enc v="2" type="pkmsg"|"msg">…</enc></to>   × N
///   </participants>
///   <enc v="2" type="skmsg">…</enc>
/// </message>
/// ```
///
/// * `<participants>` wraps an already-built child list (`LX/BpZ.<init>(VL)`,
///   bounded to [1, 0x987356]).
/// * Each child is `<to>` with a `jid` attribute, carrying the enc elements as
///   children — `LX/Cp3->A08` builds that list with element name `"to"` at all
///   three construction sites, and `LX/Cp3->A0A(jid, encList)` sets `jid` then
///   appends each enc.
/// * The enc `type` strings come from `LX/Cp3->A06`, the outgoing mapper:
///   `0=msg, 1=pkmsg, 2=skmsg, 3=frskmsg, 4=msmsg` — the exact inverse of the
///   `LX/C9c->A00` reader the receive path already matches.
///
/// The top-level `skmsg` being a **sibling** of `<participants>` rather than
/// nested inside it was inferred from `LX/Cp3->A08` returning only the `<to>`
/// list; it is now **observed**: one stanza in exactly this shape was
/// transmitted on 2026-09-22 and server-acknowledged. That run also had no
/// `phash`, for a group whose membership had just been fetched — accepted once,
/// which is not the same as generally optional.
///
/// `addressing_mode` is **confirmed** from `SendE2EMessageJob->A06`
/// (2.26.37.71) at `12d0..1322`: the value is `"lid"` when
/// `groupParticipantsManager.A0k(chat)` or `LX/0DE->A0W(chat)` holds and `"pn"`
/// otherwise, and `13d8..13fc` emits it whenever it is non-null *and* the chat
/// JID is not itself the LID form. A `g.us` chat is never that form, so a group
/// send always carries it. **Observed** live on 2026-09-22: every incoming
/// stanza from the operator's group carried `addressing_mode="lid"` with a LID
/// `participant`, so that group is LID-addressed and an outgoing stanza must
/// match it. `None` omits the attribute.
///
/// **Not emitted, because their conditions are unread:** `phash`,
/// `device_fanout`, `category`, `t`, `expiration`,
/// `recipient`/`recipient_pn`, and the per-`<to>` `peer_recipient_lid` and
/// `eph_setting` (`LX/Cp3->A07` gates the former on `DeviceJid.isPrimary()` and
/// an `LX/0DO` device resolved through an inverse LID map). `phash` is
/// `groupParticipantHash` verbatim (`A06` `1300`), emitted only when that job
/// field is non-null; it is the client's claim about the membership set it
/// encrypted for, and we cannot compute it honestly until the membership set
/// question in [`participant_hash`] is settled, so it is left off rather than
/// asserted wrongly. Omitting it forfeits the server's stale-list correction;
/// it does not fabricate one.
///
/// Nothing in this function sends.
pub fn sender_key_stanza(
    chat: &Jid,
    id: &str,
    message_type: &str,
    addressing_mode: Option<&str>,
    targets: &[FanoutTarget],
    skmsg: &[u8],
) -> Result<Node> {
    if id.is_empty() || id.len() > 128 {
        return Err(Error::invalid("group_send", "invalid stanza id"));
    }
    if targets.is_empty() {
        return Err(Error::invalid(
            "group_send",
            "a sender-key send needs at least one recipient device",
        ));
    }
    if skmsg.is_empty() || skmsg.len() > 1_048_576 {
        return Err(Error::invalid("group_send", "invalid skmsg size"));
    }
    let mut participants = Vec::with_capacity(targets.len());
    for target in targets {
        if !matches!(target.kind, "pkmsg" | "msg") {
            return Err(Error::invalid(
                "group_send",
                "a distribution child is pkmsg or msg",
            ));
        }
        if target.ciphertext.is_empty() {
            return Err(Error::invalid(
                "group_send",
                "empty distribution ciphertext",
            ));
        }
        participants.push(
            Node::new("to")
                .attr("jid", target.device.clone())
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", target.kind)
                        .bytes(target.ciphertext.clone()),
                ]),
        );
    }
    let mut node = Node::new("message")
        .attr("to", chat.clone())
        .attr("id", id)
        .attr("type", message_type);
    if let Some(mode) = addressing_mode {
        if !matches!(mode, "pn" | "lid") {
            return Err(Error::invalid("group_send", "addressing_mode is pn or lid"));
        }
        node = node.attr("addressing_mode", mode);
    }
    Ok(node.children(vec![
        Node::new("participants").children(participants),
        Node::new("enc")
            .attr("v", "2")
            .attr("type", "skmsg")
            .bytes(skmsg.to_vec()),
    ]))
}

#[cfg(test)]
mod tests {
    /// The stanza structure confirmed from LX/Cp3: a `<participants>` wrapper
    /// of `<to jid=…>` children each carrying its own `enc`, beside a top-level
    /// `skmsg`. Also checks it survives the wire codec, since it is the shape
    /// that would go out on a live account.
    #[test]
    fn sender_key_stanza_matches_the_traced_structure() {
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let targets = vec![
            FanoutTarget {
                device: Jid::parse("15551234567:1@s.whatsapp.net").unwrap(),
                kind: "pkmsg",
                ciphertext: vec![1, 2, 3],
            },
            FanoutTarget {
                device: Jid::parse("15559876543@s.whatsapp.net").unwrap(),
                kind: "msg",
                ciphertext: vec![4, 5, 6],
            },
        ];
        let node = sender_key_stanza(&chat, "M1", "text", Some("lid"), &targets, &[9; 32]).unwrap();
        assert_eq!(node.description, "message");
        assert_eq!(node.get_attr("to").unwrap().as_jid(), Some(&chat));
        assert_eq!(node.get_attr("id").unwrap().as_text(), Some("M1"));

        let children = node.child_nodes();
        assert_eq!(children.len(), 2, "participants and the skmsg are siblings");
        let participants = &children[0];
        assert_eq!(participants.description, "participants");
        assert_eq!(participants.child_nodes().len(), 2);
        for (child, expected) in participants.child_nodes().iter().zip(["pkmsg", "msg"]) {
            assert_eq!(child.description, "to");
            assert!(child.get_attr("jid").is_some());
            let enc = &child.child_nodes()[0];
            assert_eq!(enc.description, "enc");
            assert_eq!(enc.get_attr("v").unwrap().as_text(), Some("2"));
            assert_eq!(enc.get_attr("type").unwrap().as_text(), Some(expected));
        }
        // The device suffix on a target must survive into its `<to jid=…>`.
        assert_eq!(
            participants.child_nodes()[0]
                .get_attr("jid")
                .unwrap()
                .as_jid()
                .unwrap()
                .device,
            1
        );
        let skmsg = &children[1];
        assert_eq!(skmsg.description, "enc");
        assert_eq!(skmsg.get_attr("type").unwrap().as_text(), Some("skmsg"));

        // Confirmed from A06 and observed live on this group.
        assert_eq!(
            node.get_attr("addressing_mode").unwrap().as_text(),
            Some("lid")
        );
        // Nothing we could not read is emitted.
        for absent in ["phash", "device_fanout", "category", "t", "expiration"] {
            assert!(
                node.get_attr(absent).is_none(),
                "{absent} must not be guessed"
            );
        }
        // Omitting it is a choice the caller makes, not a default.
        assert!(
            sender_key_stanza(&chat, "M1", "text", None, &targets, &[9; 32])
                .unwrap()
                .get_attr("addressing_mode")
                .is_none()
        );
        assert!(sender_key_stanza(&chat, "M1", "text", Some("web"), &targets, &[9; 32]).is_err());

        let dict = crate::binary::TokenDictionary::comparison_2_26_37_70();
        let encoded = crate::binary::Encoder::new(dict).encode(&node).unwrap();
        let decoded =
            crate::binary::Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
                .decode_node()
                .unwrap();
        assert_eq!(decoded, node, "the send stanza must survive the wire codec");
    }

    #[test]
    fn sender_key_stanza_refuses_malformed_input() {
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let ok = FanoutTarget {
            device: Jid::parse("15551234567@s.whatsapp.net").unwrap(),
            kind: "pkmsg",
            ciphertext: vec![1],
        };
        // No recipients, empty skmsg, empty id.
        assert!(sender_key_stanza(&chat, "M", "text", Some("lid"), &[], &[1]).is_err());
        assert!(
            sender_key_stanza(
                &chat,
                "M",
                "text",
                Some("lid"),
                std::slice::from_ref(&ok),
                &[]
            )
            .is_err()
        );
        assert!(
            sender_key_stanza(
                &chat,
                "",
                "text",
                Some("lid"),
                std::slice::from_ref(&ok),
                &[1]
            )
            .is_err()
        );
        // A distribution child is never an skmsg.
        let wrong = FanoutTarget {
            kind: "skmsg",
            ..ok.clone()
        };
        assert!(sender_key_stanza(&chat, "M", "text", Some("lid"), &[wrong], &[1]).is_err());
        // Empty per-device ciphertext.
        let empty = FanoutTarget {
            ciphertext: Vec::new(),
            ..ok
        };
        assert!(sender_key_stanza(&chat, "M", "text", Some("lid"), &[empty], &[1]).is_err());
    }

    /// Vectors computed independently with Python's hashlib and base64, not by
    /// running this implementation: sorted raw JID strings concatenated with no
    /// separator, SHA-1, first six digest bytes, standard Base64, "1:" prefix.
    #[test]
    fn participant_hash_matches_independent_vectors() {
        let jids = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            participant_hash(&jids(&["15551234567@s.whatsapp.net"])),
            "1:XS9Zi8u/"
        );
        let three = jids(&[
            "15551234567@s.whatsapp.net",
            "15559876543@s.whatsapp.net",
            "120363000000000001@g.us",
        ]);
        assert_eq!(participant_hash(&three), "1:Lz4D7Wb2");
        // Sorting is part of the derivation: input order must not matter.
        let shuffled = jids(&[
            "120363000000000001@g.us",
            "15559876543@s.whatsapp.net",
            "15551234567@s.whatsapp.net",
        ]);
        assert_eq!(participant_hash(&shuffled), participant_hash(&three));
        // The emitter bounds phash to ten characters; six digest bytes encode to
        // eight Base64 characters, so the prefixed value is exactly ten.
        assert_eq!(participant_hash(&three).len(), 10);
        assert_eq!(participant_hash(&jids(&[])).len(), 10);
        // Membership changes the hash, which is the point of it.
        assert_ne!(
            participant_hash(&three),
            participant_hash(&jids(&[
                "15551234567@s.whatsapp.net",
                "15559876543@s.whatsapp.net",
            ]))
        );
    }

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

    fn text(node: &Node, name: &str) -> Option<String> {
        attr_string(node, name)
    }

    #[test]
    fn request_shapes_follow_the_documented_builders() {
        let (_d, s) = store();
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let alice = Jid::parse("15550000001@s.whatsapp.net").unwrap();
        let id = queue_create(&s, "Lab", std::slice::from_ref(&alice)).unwrap();
        let req = super::super::actions::request(&s, &id).unwrap().unwrap();
        assert_eq!(text(&req, "to").as_deref(), Some("g.us"));
        assert_eq!(text(&req, "xmlns").as_deref(), Some("w:g2"));
        assert_eq!(text(&req, "type").as_deref(), Some("set"));
        let create = req.child("create").unwrap();
        assert_eq!(text(create, "subject").as_deref(), Some("Lab"));
        assert_eq!(text(create, "key").map(|k| k.len()), Some(32));
        assert_eq!(create.child_nodes().len(), 1);
        assert_eq!(create.child_nodes()[0].description, "participant");

        let id = queue_info(&s, &group).unwrap();
        let req = super::super::actions::request(&s, &id).unwrap().unwrap();
        assert_eq!(text(&req, "type").as_deref(), Some("get"));
        assert_eq!(
            text(req.child("query").unwrap(), "request").as_deref(),
            Some("interactive")
        );

        let id = queue_leave(&s, &group).unwrap();
        let req = super::super::actions::request(&s, &id).unwrap().unwrap();
        assert_eq!(text(&req, "to").as_deref(), Some("g.us"));
        let inner = req.child("leave").unwrap().child("group").unwrap();
        assert_eq!(
            text(inner, "id").as_deref(),
            Some(group.to_string().as_str())
        );

        for (action, f) in [
            (
                "add",
                queue_add as fn(&ProtocolStore, &Jid, &[Jid]) -> Result<String>,
            ),
            ("remove", queue_remove),
            ("promote", queue_promote),
            ("demote", queue_demote),
        ] {
            let id = f(&s, &group, std::slice::from_ref(&alice)).unwrap();
            let req = super::super::actions::request(&s, &id).unwrap().unwrap();
            assert_eq!(
                text(&req, "to").as_deref(),
                Some(group.to_string().as_str())
            );
            assert!(req.child(action).is_some(), "{action}");
        }
        let id = queue_invite(&s, &group, true).unwrap();
        let req = super::super::actions::request(&s, &id).unwrap().unwrap();
        assert_eq!(text(&req, "type").as_deref(), Some("set"));
        assert!(req.child("invite").is_some());
        assert!(queue_create(&s, "", std::slice::from_ref(&alice)).is_err());
        assert!(queue_info(&s, &alice).is_err(), "not a group");
        assert!(queue_add(&s, &group, &[]).is_err());
    }

    #[test]
    fn replies_build_snapshots_and_events_once() {
        let (_d, s) = store();
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let alice = Jid::parse("15550000001@s.whatsapp.net").unwrap();
        let bob = Jid::parse("15550000002@s.whatsapp.net").unwrap();
        let id = queue_create(&s, "Lab", std::slice::from_ref(&alice)).unwrap();
        let _ = super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![
                Node::new("group")
                    .attr("id", group.clone())
                    .attr("subject", "Lab")
                    .attr("creator", alice.clone())
                    .attr("creation", "1700000000")
                    .children(vec![
                        Node::new("participant")
                            .attr("jid", alice.clone())
                            .attr("type", "superadmin"),
                        Node::new("participant").attr("jid", bob.clone()),
                    ]),
            ]);
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        assert_eq!(ingest(&s).unwrap(), 0, "idempotent");
        let saved = get(&s, &group).unwrap().unwrap();
        assert_eq!(saved.subject.as_deref(), Some("Lab"));
        assert_eq!(saved.members.len(), 2);
        assert_eq!(saved.members[0].role.as_deref(), Some("superadmin"));
        assert_eq!(list(&s).unwrap().len(), 1);
        // A later info reply without bob and a new subject emits updates.
        let id = queue_info(&s, &group).unwrap();
        let _ = super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![
                Node::new("group")
                    .attr("id", group.clone())
                    .attr("subject", "Lab 2")
                    .children(vec![Node::new("participant").attr("jid", alice.clone())]),
            ]);
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        let saved = get(&s, &group).unwrap().unwrap();
        assert_eq!(saved.subject.as_deref(), Some("Lab 2"));
        assert_eq!(saved.members.len(), 1);
        // Invite code attaches to the snapshot.
        let id = queue_invite(&s, &group, false).unwrap();
        let _ = super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("invite").attr("code", "ABCDEFG")]);
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        assert_eq!(
            get(&s, &group).unwrap().unwrap().invite_code.as_deref(),
            Some("ABCDEFG")
        );
        // Leaving removes the snapshot.
        let id = queue_leave(&s, &group).unwrap();
        let _ = super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result");
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert_eq!(ingest(&s).unwrap(), 1);
        assert!(get(&s, &group).unwrap().is_none());
        let events = s
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("t")?,
                    crate::model::ids::EventCursor::START,
                    64,
                )
            })
            .unwrap();
        let kinds: Vec<_> = events.events.iter().map(|e| e.event.kind()).collect();
        assert!(kinds.contains(&"group_created"));
        assert!(kinds.contains(&"group_updated"));
    }
}
