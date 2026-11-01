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
const PENDING_PREFIX: &str = "groups-pending-ingestion:";
const ACTION_KINDS: [&str; 11] = [
    "group-create",
    "group-info",
    "group-refresh",
    "group-subject",
    "group-description",
    "group-add",
    "group-remove",
    "group-promote",
    "group-demote",
    "group-leave",
    "group-invite",
];

/// The action settlement and recovery marker share one commit. This marks only
/// newly received replies, rather than backfilling historical snapshots at login.
pub(crate) fn mark_pending_in(
    tx: &mut crate::store::WriteTx<'_>,
    kind: &str,
    id: &str,
) -> Result<()> {
    if ACTION_KINDS.contains(&kind) {
        tx.put_json(
            Table::Checkpoints,
            format!("{PENDING_PREFIX}{id}").as_bytes(),
            1,
            &true,
        )?;
    }
    Ok(())
}

/// Apply a session's pending group reply once. Unsupported/truncated replies
/// remain inspectable without preventing unrelated receiving; storage errors
/// still stop the session rather than masking a failed durable commit.
pub(crate) fn apply_pending(store: &ProtocolStore, id: &str) -> Result<bool> {
    let pending: Option<bool> = store.read(|tx| {
        tx.get_json(
            Table::Checkpoints,
            format!("{PENDING_PREFIX}{id}").as_bytes(),
            1,
        )
    })?;
    if pending != Some(true) {
        return Ok(false);
    }
    let Some((outcome, reply)) = store.read(|tx| super::actions::view_in(tx, id))? else {
        return Err(invalid("pending group action is missing"));
    };
    if !ACTION_KINDS.contains(&outcome.kind.as_str())
        || !matches!(
            outcome.status,
            super::actions::ActionStatus::Ok | super::actions::ActionStatus::Error
        )
    {
        return Err(invalid("pending group action is not settled"));
    }
    let request = super::actions::request(store, id)?;
    match store.write(|tx| {
        ingest_reply_in(
            tx,
            &outcome.kind,
            &outcome,
            request.as_ref(),
            reply.as_ref(),
        )
    }) {
        Err(Error::InvalidInput {
            field: "group",
            reason,
        }) => {
            tracing::warn!(%reason, "queued group reply remains unapplied; saved response preserved");
            Ok(false)
        }
        result => result,
    }
}

/// Recover only replies marked during session settlement, without transmitting
/// their requests again or replaying already consumed historical action rows.
pub(crate) fn recover_pending(store: &ProtocolStore) -> Result<usize> {
    let ids = store.read(|tx| {
        let mut ids = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                if let Some(id) = std::str::from_utf8(&key)
                    .ok()
                    .and_then(|k| k.strip_prefix(PENDING_PREFIX))
                {
                    ids.push(id.to_owned());
                }
            }
        }
        Ok(ids)
    })?;
    let mut applied = 0;
    for id in ids {
        applied += usize::from(apply_pending(store, &id)?);
    }
    Ok(applied)
}

fn invalid(msg: &'static str) -> Error {
    Error::invalid("group", msg)
}

fn group_server() -> Result<Jid> {
    Jid::parse("g.us")
}

fn require_group(jid: &Jid) -> Result<Jid> {
    jid.require_group_id()?;
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

/// Queue one classic batch query. APK 37.71 EJy selector 4 emits this
/// per_group_dirty_recovery context; 9QE selector 11 emits group@jid.
/// No automatic retry or truncated follow-up is queued by this operation.
pub fn queue_refresh(store: &ProtocolStore, groups: &[Jid]) -> Result<String> {
    queue_refresh_mode(store, groups, false)
}

/// Opt into APK-derived bounded continuation of authenticated truncated results.
/// Initial zero estimates derive round limit 2 in APK 37.71 job A0G 04a2..04f4.
/// Refused, unanswered or unsupported responses never generate another request.
pub fn queue_refresh_follow_up(store: &ProtocolStore, groups: &[Jid]) -> Result<String> {
    queue_refresh_mode(store, groups, true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshContinuation {
    root: String,
    round: u8,
    remaining: Vec<TruncatedGroup>,
}

fn refresh_node(id: &str, groups: &[TruncatedGroup], follow_up: bool) -> Result<Node> {
    let children = groups
        .iter()
        .map(|g| Ok(Node::new("group").attr("jid", Jid::parse(&g.jid)?)))
        .collect::<Result<Vec<_>>>()?;
    Ok(iq(
        id,
        "get",
        group_server()?,
        vec![
            Node::new("query")
                .attr(
                    "context",
                    if follow_up {
                        "per_group_dirty_recovery_truncatable"
                    } else {
                        "per_group_dirty_recovery"
                    },
                )
                .children(children),
        ],
    ))
}

fn queue_refresh_mode(store: &ProtocolStore, groups: &[Jid], follow_up: bool) -> Result<String> {
    if groups.is_empty() || groups.len() > 256 {
        return Err(invalid(
            "batch refresh requires one to 256 groups (local bound)",
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    let mut inputs = Vec::new();
    for group in groups {
        require_group(group)?;
        if group.user.is_empty() || group.is_ad() {
            return Err(invalid("batch refresh requires concrete group identities"));
        }
        if !unique.insert(group.to_string()) {
            return Err(invalid("batch refresh has duplicate groups"));
        }
        inputs.push(TruncatedGroup {
            jid: group.to_string(),
            size: 0,
        });
    }
    store.write(|tx| queue_refresh_in(tx, &inputs, follow_up, "refresh groups"))
}

fn queue_refresh_in(
    tx: &mut crate::store::WriteTx<'_>,
    inputs: &[TruncatedGroup],
    follow_up: bool,
    label: &str,
) -> Result<String> {
    let id = super::actions::queue_in(tx, "group-refresh", label, |id| {
        refresh_node(id, inputs, follow_up)
    })?;
    if follow_up {
        tx.put_json(
            Table::Checkpoints,
            format!("group-refresh-job:{id}").as_bytes(),
            1,
            &RefreshContinuation {
                root: id.clone(),
                round: 0,
                remaining: Vec::new(),
            },
        )?;
    }
    Ok(id)
}

#[derive(Serialize, Deserialize)]
struct DirtyNotification {
    fingerprint: String,
    action: String,
}

/// Acknowledge a `w:gp2` group notification other than `groups_dirty`
/// (that one is applied and ACKed by [`handle_dirty_notification`]).
///
/// **Confirmed, 2.26.37.71:** `Fbq` registers `w:gp2`; its handler `0yv`
/// extends `0yO` (`BaseNotificationHandler.handleAndAckNotification`), which
/// ACKs every notification after `A09`, or through `AAR.A00` when `A09`
/// deferred with `A07(true)` (32 sites in `0yv.A09`, none adding an `error`
/// attribute). `Cp2.A05` writes id/to/class/type/participant.
/// **Observed:** leaving these unacknowledged ended sessions with a
/// `stream:error` echoing `ack class=notification type=w:gp2`.
/// The membership/subject changes they carry are not applied here; the
/// durable encrypted capture keeps them, and group sends fetch membership
/// fresh. Malformed id/from/participant is an error (captured, no ACK).
pub fn notification_ack(node: &Node) -> Result<Option<Node>> {
    if node.child_nodes().len() == 1 && node.child_nodes()[0].description == "groups_dirty" {
        return Ok(None);
    }
    super::notifications::base_ack(node, "w:gp2")
}

/// Apply the supported w:gp2/groups_dirty notification before acknowledging.
/// APK 37.71 0yv.A09 2912..2990/6548..6572 queues zero-estimate truncatable
/// refresh; 0yO.A01 dispatches before ACK. Unknown shapes receive no ACK.
/// Capture must already be durable and the stream authenticated by the caller.
pub fn handle_dirty_notification(store: &ProtocolStore, node: &Node) -> Result<Option<Node>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("w:gp2")
        || node.child_nodes().len() != 1
        || node.child_nodes()[0].description != "groups_dirty"
    {
        return Ok(None);
    }
    // Local input restrictions are stronger than the APK raw-string extraction.
    // Invalid/empty lists stay captured rather than acknowledging an empty job.
    let prepare = || -> Result<_> {
        let from = super::receipts::jid(node, "from")
            .ok_or_else(|| invalid("dirty notification source is missing or invalid"))?;
        let id = node
            .get_attr("id")
            .and_then(|v| v.as_text())
            .filter(|id| !id.is_empty() && id.len() <= 128)
            .ok_or_else(|| invalid("dirty notification ID is missing or outside local bound"))?;
        let participant = if node.get_attr("participant").is_some() {
            Some(
                super::receipts::jid(node, "participant")
                    .ok_or_else(|| invalid("dirty notification participant is invalid"))?,
            )
        } else {
            None
        };
        let children = node.child_nodes()[0].child_nodes();
        if children.len() > 256 || children.iter().any(|g| g.description != "group") {
            return Err(invalid(
                "dirty notification list is excessive or unsupported",
            ));
        }
        let mut unique = std::collections::BTreeSet::new();
        let mut inputs = Vec::new();
        for group in children {
            if group.get_attr("jid").is_none() {
                continue;
            }
            let jid = super::receipts::jid(group, "jid")
                .ok_or_else(|| invalid("dirty notification group identity is invalid"))?;
            require_group(&jid)?;
            if jid.user.is_empty() || jid.is_ad() || !unique.insert(jid.to_string()) {
                return Err(invalid(
                    "dirty notification group identity is empty, device or duplicate",
                ));
            }
            inputs.push(TruncatedGroup {
                jid: jid.to_string(),
                size: 0,
            });
        }
        if inputs.is_empty() {
            return Err(invalid("dirty notification has no supported groups"));
        }
        // CmG.A02/A00 extracts from/id/type/participant; recipient/category/edit
        // belong to its separate receipt wrapper, so are not echoed here.
        let mut ack = Node::new("ack")
            .attr("to", super::receipts::ack_address(from.clone()))
            .attr("id", id)
            .attr("class", "notification")
            .attr("type", "w:gp2");
        if let Some(participant) = &participant {
            ack = ack.attr(
                "participant",
                super::receipts::ack_address(participant.clone()),
            );
        }
        use sha2::{Digest, Sha256};
        let key = format!(
            "groups-dirty:{}",
            hex::encode(Sha256::digest(
                serde_json::to_vec(&(from.to_string(), id))
                    .map_err(|_| invalid("dirty source serialization failed"))?
            ))
        );
        let fingerprint = hex::encode(Sha256::digest(
            serde_json::to_vec(&(inputs.clone(), participant))
                .map_err(|_| invalid("dirty inputs serialization failed"))?,
        ));
        Ok((inputs, ack, key, fingerprint))
    };
    let (inputs, ack, key, fingerprint) = match prepare() {
        Ok(prepared) => prepared,
        Err(Error::InvalidInput { reason, .. }) => {
            tracing::warn!(%reason, "group dirty notification unsupported; captured without ACK");
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let accepted = store.write(|tx| {
        if let Some(saved) =
            tx.get_json::<DirtyNotification>(Table::Checkpoints, key.as_bytes(), 1)?
        {
            // Repeat ACK is safe; never rotate/resend an existing query ID.
            return Ok(saved.fingerprint == fingerprint);
        }
        let action = queue_refresh_in(tx, &inputs, true, "refresh dirty groups")?;
        tx.put_json(
            Table::Checkpoints,
            key.as_bytes(),
            1,
            &DirtyNotification {
                fingerprint,
                action,
            },
        )?;
        Ok(true)
    })?;
    if !accepted {
        tracing::warn!(
            "group dirty notification ID conflicts with saved inputs; captured without ACK"
        );
    }
    Ok(accepted.then_some(ack))
}

fn first_refresh_partition(
    inputs: Vec<TruncatedGroup>,
) -> (Vec<TruncatedGroup>, Vec<TruncatedGroup>) {
    // APK 37.71 job A0G 00de..0126 flushes above participant budget,
    // resetting the accumulator to zero after the inserted first pair.
    // 10000 is our conservative local budget, equal to the APK floor before
    // its unknown feature cap; it does not claim identical heap/feature state.
    let mut first = Vec::new();
    let mut remaining = Vec::new();
    let mut cost = 0u32;
    let mut split = false;
    for group in inputs {
        if !split && cost != 0 && cost + group.size > 10000 {
            split = true;
        }
        if split {
            remaining.push(group);
        } else {
            cost += group.size;
            first.push(group);
        }
    }
    (first, remaining)
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

/// Classification derived from complete group reply child elements.
/// APK 37.71 ADs.A01 uses precedence parent=1, default_sub_group=3,
/// general_chat=6, linked_parent=2, otherwise=0 (0044..0064).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupType {
    /// No classification child in the complete reply (APK value 0).
    Ordinary,
    /// Parent group child (APK value 1).
    Parent,
    /// Linked parent child (APK value 2).
    LinkedSubgroup,
    /// Default subgroup child (APK value 3).
    DefaultSubgroup,
    /// General chat child (APK value 6).
    GeneralChat,
}

fn group_type(node: &Node) -> GroupType {
    // 8kZ.A1Z and 1xi.A1U test child presence, not text or attributes.
    if node.child("parent").is_some() {
        GroupType::Parent
    } else if node.child("default_sub_group").is_some() {
        GroupType::DefaultSubgroup
    } else if node.child("general_chat").is_some() {
        GroupType::GeneralChat
    } else if node.child("linked_parent").is_some() {
        GroupType::LinkedSubgroup
    } else {
        GroupType::Ordinary
    }
}

/// Stored snapshot of a group (`group:<jid>`), built from `group` replies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupSnapshot {
    /// Group JID.
    pub jid: String,
    /// Complete-reply classification; null for historical rows or local stubs.
    #[serde(default)]
    pub group_type: Option<GroupType>,
    /// Typed group JID from `linked_parent@jid`; null if absent or unsupported.
    #[serde(default)]
    pub linked_parent: Option<String>,
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

pub(crate) fn key(jid: &str) -> String {
    format!("group:{jid}")
}

fn require_complete_group(node: &Node) -> Result<()> {
    // APK 37.71 ADs.A0C 002e..0078 returns a size-only A9s for truncated
    // groups, before constructing AAO. Replacing membership would invent removals.
    if let Some(flag) = node.get_attr("truncated") {
        let flag = flag
            .as_text()
            .ok_or_else(|| invalid("invalid group truncated flag"))?;
        if flag.eq_ignore_ascii_case("true") {
            return Err(invalid(
                "truncated group reply requires refresh; existing snapshot preserved",
            ));
        }
        // Local validation is stricter than Java Boolean.parseBoolean, which
        // treats any string other than case-insensitive true as false.
        if !flag.eq_ignore_ascii_case("false") {
            return Err(invalid("invalid group truncated flag"));
        }
    }
    Ok(())
}

/// Parse a complete `group` element; truncated or invalid flags yield no snapshot.
pub fn parse_group(node: &Node) -> Option<GroupSnapshot> {
    if node.description != "group" {
        return None;
    }
    require_complete_group(node).ok()?;
    let jid = attr_string(node, "id")?;
    let jid = if jid.contains('@') {
        jid
    } else {
        format!("{jid}@g.us")
    };
    let mut members: Vec<Member> = Vec::new();
    let mut member_indices = std::collections::HashMap::new();
    for participant in node
        .child_nodes()
        .iter()
        .filter(|c| c.description == "participant")
    {
        // A `<participant>` carrying `error` is one the server refused to add
        // (**observed** 2026-09-22 on a `group create` reply: the snapshot said
        // two members while the next info fetch said one, and the fan-out then
        // had nobody to address). It is not a member, so it is not counted.
        if participant.get_attr("error").is_some() {
            continue;
        }
        // APK 37.71 ADs.A05 002e..0046 casts jid to UserJid and skips null;
        // DeviceJid extends Jid directly, not UserJid. Our user-level snapshot
        // uses the supported user-like namespaces without an agent/device.
        let Some(jid) = attr_string(participant, "jid")
            .and_then(|raw| Jid::parse(&raw).ok())
            .filter(|jid| jid.is_user_like() && !jid.is_ad())
        else {
            continue;
        };
        let member = Member {
            jid: jid.to_string(),
            role: attr_string(participant, "type"),
        };
        // ADs.A05 016c..0172 uses Map.put on 1xf.A1H's LinkedHashMap:
        // last value wins, with the first insertion position retained.
        if let Some(&index) = member_indices.get(&member.jid) {
            members[index] = member;
        } else {
            member_indices.insert(member.jid.clone(), members.len());
            members.push(member);
        }
    }
    Some(GroupSnapshot {
        jid,
        group_type: Some(group_type(node)),
        // APK 37.71 ADs.A03 0000..002a reads linked_parent child and typed
        // group jid. An unsupported/missing identity stays absent; type is
        // independently determined by child presence, not valid parent data.
        linked_parent: node
            .child("linked_parent")
            .and_then(|parent| super::receipts::jid(parent, "jid"))
            .filter(|jid| jid.server == Server::Group && !jid.user.is_empty() && !jid.is_ad())
            .map(|jid| jid.to_string()),
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
/// Truncated replies return an error and preserve the current snapshot.
///
/// A fan-out needs the membership inside the one connection it is allowed, so
/// it issues [`info_request`] itself; the result must still land in the same
/// snapshot the rest of the client reads.
pub fn record_info_reply(store: &ProtocolStore, reply: &Node) -> Result<bool> {
    let Some(node) = reply.child("group") else {
        return Ok(false);
    };
    require_complete_group(node)?;
    let Some(group) = parse_group(node) else {
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
/// actions are only marked seen. State, events and the marker commit atomically.
pub fn ingest(store: &ProtocolStore) -> Result<usize> {
    let mut applied = 0usize;
    for kind in ACTION_KINDS {
        for (outcome, reply) in super::actions::settled(store, kind)? {
            // The stored request is immutable. Load it before the write lock;
            // invite/leave application must not nest a store transaction.
            let request = super::actions::request(store, &outcome.id)?;
            if store
                .write(|tx| ingest_reply_in(tx, kind, &outcome, request.as_ref(), reply.as_ref()))?
            {
                applied += 1;
            }
        }
    }
    Ok(applied)
}

fn ingest_reply_in(
    tx: &mut crate::store::WriteTx<'_>,
    kind: &str,
    outcome: &super::actions::ActionOutcome,
    request: Option<&Node>,
    reply: Option<&Node>,
) -> Result<bool> {
    let marker = format!("{INGESTED_PREFIX}{}", outcome.id);
    let seen: Option<bool> = tx.get_json(Table::Checkpoints, marker.as_bytes(), 1)?;
    if seen.unwrap_or(false) {
        tx.delete(
            Table::Checkpoints,
            format!("{PENDING_PREFIX}{}", outcome.id).as_bytes(),
        )?;
        return Ok(false);
    }
    if outcome.status == super::actions::ActionStatus::Ok
        && let Some(reply) = reply
    {
        apply_reply_in(tx, kind, request, reply)?;
    }
    // Snapshot, journal events and this marker share the same commit. Otherwise
    // concurrent ingestion or an intervening reply can apply a saved reply twice.
    tx.put_json(Table::Checkpoints, marker.as_bytes(), 1, &true)?;
    tx.delete(
        Table::Checkpoints,
        format!("{PENDING_PREFIX}{}", outcome.id).as_bytes(),
    )?;
    Ok(true)
}

fn apply_reply_in(
    tx: &mut crate::store::WriteTx<'_>,
    kind: &str,
    request: Option<&Node>,
    reply: &Node,
) -> Result<()> {
    if kind == "group-refresh" {
        return apply_refresh_in(tx, request, reply);
    }
    if let Some(node) = reply.child("group") {
        require_complete_group(node)?;
        if let Some(group) = parse_group(node) {
            if kind == "group-info" {
                // A correlated IQ id alone does not authorize replacing another
                // group's membership. parse_group also accepts a short numeric id.
                let expected = request.and_then(|r| super::receipts::jid(r, "to"));
                if expected.as_ref() != Jid::parse(&group.jid).ok().as_ref() {
                    return Err(invalid(
                        "group info reply does not name the requested group",
                    ));
                }
            }
            return save_snapshot_in(tx, group);
        }
    }
    if kind == "group-invite" {
        let Some(code) = reply.child("invite").and_then(|n| attr_string(n, "code")) else {
            return Ok(());
        };
        // The request's `to` is the group.
        let Some(request) = request else {
            return Ok(());
        };
        let Some(group) = attr_string(request, "to") else {
            return Ok(());
        };
        let mut snapshot: GroupSnapshot = tx
            .get_json(Table::Checkpoints, key(&group).as_bytes(), 1)?
            .unwrap_or(GroupSnapshot {
                jid: group.clone(),
                group_type: None,
                linked_parent: None,
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
        return tx.put_json(Table::Checkpoints, key(&group).as_bytes(), 1, &snapshot);
    }
    if kind == "group-leave" {
        // The request names the group inside `leave/group@id`.
        if let Some(request) = request
            && let Some(group) = request
                .child("leave")
                .and_then(|l| l.child("group"))
                .and_then(|g| attr_string(g, "id"))
        {
            tx.delete(Table::Checkpoints, key(&group).as_bytes())?;
        }
    }
    Ok(())
}

/// One group whose batch response contains only size, not complete membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruncatedGroup {
    /// Concrete group identity. Stored encrypted; CLI summaries do not print it.
    pub jid: String,
    /// APK 37.71 AX9 permits size 0..=19999; this is not a member list.
    pub size: u32,
}

/// Durable projection result, separate from the outer IQ action status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RefreshOutcome {
    /// Complete snapshots committed in this batch.
    pub complete: usize,
    /// Size-only groups returned by this response; not a member list.
    pub truncated: Vec<TruncatedGroup>,
    /// APK GroupForbidden results (group@error=403); snapshots retained.
    pub forbidden: usize,
    /// APK GroupNotExist results (group@error=404); snapshots retained.
    pub not_found: usize,
    /// Whether this action opted into the bounded continuation chain.
    #[serde(default)]
    pub automatic_follow_up: bool,
    /// Current round, zero for initial or noncontinuing refreshes.
    #[serde(default)]
    pub round: u8,
    /// Next durable query ID, when a recognized result permits continuation.
    #[serde(default)]
    pub next_action: Option<String>,
    /// All groups still needing full results, including unsent partitions.
    #[serde(default)]
    pub pending: Vec<TruncatedGroup>,
    /// Pending work remains but APK-derived round limit 2 was reached.
    #[serde(default)]
    pub exhausted: bool,
}

/// Inspect the projection result of one settled batch, without transmitting.
pub fn refresh_outcome(store: &ProtocolStore, id: &str) -> Result<Option<RefreshOutcome>> {
    store.read(|tx| {
        let report: Option<RefreshOutcome> = tx.get_json(
            Table::Checkpoints,
            format!("group-refresh-result:{id}").as_bytes(),
            1,
        )?;
        Ok(report.map(|mut report| {
            // Pre-continuation reports contain truncated inputs but no pending
            // field. Recover inspection counts without writes or new queries.
            if !report.automatic_follow_up && report.pending.is_empty() {
                report.pending.clone_from(&report.truncated);
            }
            report
        }))
    })
}

fn apply_refresh_in(
    tx: &mut crate::store::WriteTx<'_>,
    request: Option<&Node>,
    reply: &Node,
) -> Result<()> {
    let request = request.ok_or_else(|| invalid("batch refresh request is missing"))?;
    let id =
        attr_string(request, "id").ok_or_else(|| invalid("batch refresh request ID is missing"))?;
    let query = request
        .child("query")
        .ok_or_else(|| invalid("batch refresh query is missing"))?;
    let expected: std::collections::BTreeSet<_> = query
        .child_nodes()
        .iter()
        .filter_map(|group| attr_string(group, "jid"))
        .collect();
    if expected.is_empty() {
        return Err(invalid("batch refresh request has no groups"));
    }
    // APK 37.71 Aov selectors 10/11 decode groups/group with AZM selector 15.
    // Validate correlation/union before any snapshot or outcome write.
    if reply.child_nodes().len() != 1 || reply.child_nodes()[0].description != "groups" {
        return Err(invalid("batch refresh reply requires one groups container"));
    }
    let nodes = reply.child_nodes()[0].child_nodes();
    if nodes.len() != expected.len() {
        return Err(invalid("batch refresh reply omits or adds groups"));
    }
    let mut found = std::collections::BTreeSet::new();
    let mut snapshots = Vec::new();
    let mut outcome = RefreshOutcome::default();
    for node in nodes {
        if node.description != "group" || node.child("error").is_some() {
            return Err(invalid(
                "batch refresh contains an unsupported group result; snapshots preserved",
            ));
        }
        let raw = attr_string(node, "id")
            .ok_or_else(|| invalid("batch refresh group has no identity"))?;
        let jid = if raw.contains('@') {
            raw
        } else {
            format!("{raw}@g.us")
        };
        if !expected.contains(&jid) || !found.insert(jid.clone()) {
            return Err(invalid(
                "batch refresh contains an unexpected or duplicate group",
            ));
        }
        if let Some(error) = node.get_attr("error") {
            // APK 37.71 AXA/AXB.AAK 0070..009c require exact 403/404
            // and construct 9PO; A74 puts the code in the per-group result map.
            match error.as_text() {
                Some("403") => outcome.forbidden += 1,
                Some("404") => outcome.not_found += 1,
                _ => {
                    return Err(invalid(
                        "batch refresh contains an unrecognized group error",
                    ));
                }
            }
        } else if node.get_attr("truncated").and_then(|v| v.as_text()) == Some("true") {
            // AX9.AAK 0064..008e checks primitive long size 0..19999;
            // 0098..00bc requires exact lowercase true before constructing 9PN.
            let size = node
                .get_attr("size")
                .and_then(|v| v.as_text())
                .and_then(|raw| raw.parse::<i64>().ok())
                .filter(|n| (0..=19999).contains(n))
                .ok_or_else(|| invalid("truncated batch group requires size 0..19999"))?;
            outcome.truncated.push(TruncatedGroup {
                jid,
                size: size as u32,
            });
        } else {
            require_complete_group(node)?;
            snapshots.push(
                parse_group(node).ok_or_else(|| invalid("batch refresh group has no identity"))?,
            );
            outcome.complete += 1;
        }
    }
    for snapshot in snapshots {
        save_snapshot_in(tx, snapshot)?;
    }
    let continuation: Option<RefreshContinuation> = tx.get_json(
        Table::Checkpoints,
        format!("group-refresh-job:{id}").as_bytes(),
        1,
    )?;
    if let Some(continuation) = continuation {
        outcome.automatic_follow_up = true;
        outcome.round = continuation.round;
        // A0G 036a..03b8 retains original input order when removing processed
        // groups. Response order does not control the next partition.
        for requested in query.child_nodes() {
            let jid = attr_string(requested, "jid").unwrap_or_default();
            if let Some(group) = outcome.truncated.iter().find(|g| g.jid == jid) {
                outcome.pending.push(group.clone());
            }
        }
        outcome.pending.extend(continuation.remaining);
        // APK A0G 0420..0436 schedules only nonempty pending work with round
        // below the bound; 0474..049a creates a new job with round+1. New read
        // queries are distinct actions; an unanswered action is never replayed.
        if !outcome.pending.is_empty() && continuation.round < 2 {
            let (next, remaining) = first_refresh_partition(outcome.pending.clone());
            let next_id = super::actions::queue_in(tx, "group-refresh", "refresh groups", |id| {
                refresh_node(id, &next, true)
            })?;
            tx.put_json(
                Table::Checkpoints,
                format!("group-refresh-job:{next_id}").as_bytes(),
                1,
                &RefreshContinuation {
                    root: continuation.root,
                    round: continuation.round + 1,
                    remaining,
                },
            )?;
            outcome.next_action = Some(next_id);
        } else {
            outcome.exhausted = !outcome.pending.is_empty();
        }
    } else {
        outcome.pending = outcome.truncated.clone();
    }
    tx.put_json(
        Table::Checkpoints,
        format!("group-refresh-result:{id}").as_bytes(),
        1,
        &outcome,
    )
}

fn save_snapshot(store: &ProtocolStore, group: GroupSnapshot) -> Result<()> {
    store.write(|tx| save_snapshot_in(tx, group))
}

fn save_snapshot_in(tx: &mut crate::store::WriteTx<'_>, mut group: GroupSnapshot) -> Result<()> {
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
    store.read(|tx| get_in(tx, group))
}
pub(crate) fn get_in(tx: &crate::store::ReadTx<'_>, group: &Jid) -> Result<Option<GroupSnapshot>> {
    let group = require_group(group)?.to_string();
    tx.get_json(Table::Checkpoints, key(&group).as_bytes(), 1)
}

pub use super::participant_hash::{
    DeviceParticipantHashVersion, participant_device_hash, participant_hash, participant_hash_v2,
};

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

/// What a sender-key stanza carries, which decides its `type` and the extra
/// markers the recipient needs to render it. Each variant mirrors the direct
/// envelope for the same content (`messaging::direct_typed`,
/// `messaging::direct_media`), because the payload inside `skmsg` is the same
/// `Message` a direct send would carry.
///
/// **Inferred** for groups: the markers go on the top-level `skmsg` `enc`, the
/// one that carries the content. `LX/Cp3->A09` is the shared `enc` builder and
/// emits `mediatype` and `decrypt-fail="hide"` under their own conditions; which
/// of a fan-out's encs those conditions select was not traced. The per-device
/// encs carry only the sender-key distribution, which is not media and not an
/// add-on, so they stay minimal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StanzaContent<'a> {
    /// `type="text"`, nothing else.
    Text,
    /// `type="reaction"`, `enc decrypt-fail="hide"` and `meta
    /// content_type="add_on"`, observed together on the official client's
    /// reactions (see `messaging::direct_typed`).
    Reaction,
    /// `type="media"` and `enc mediatype=<kind>` (`image`, `ptt`), as
    /// `messaging::direct_media` sends.
    Media(&'a str),
}

impl StanzaContent<'_> {
    fn message_type(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Reaction => "reaction",
            Self::Media(_) => "media",
        }
    }
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
///   which holds the literal — **confirmed**; the string has 31 references, so
///   an earlier note that binary-token node names cannot be found by string
///   search was wrong). `SendLiveLocationKeyJob->A0G` builds the same container
///   without obfuscation, which is the easiest place to re-read this shape.
/// * Each child is `<to>` with a `jid` attribute, carrying the enc elements as
///   children — `LX/Cp3->A08` builds that list with element name `"to"` at all
///   three construction sites, and `LX/Cp3->A0A(jid, encList)` sets `jid` then
///   appends each enc.
/// * An outgoing `<enc>` carries `v` and `type` always, and `count`,
///   `mediatype`, `native_flow_name`, `duration`, `decrypt-fail="hide"`,
///   `state="false"` and `session_type="pq"` only under their own conditions
///   (`LX/Cp3->A09`, **confirmed**). None applies to a plain first send, so
///   `v` and `type` alone is the correct minimal case, not an omission.
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
/// `phash` is `groupParticipantHash` verbatim (`A06` `1300`), emitted only when
/// that job field is non-null. It is now computed rather than omitted — see
/// [`participant_hash_v2`], whose construction was solved against a value the
/// server itself returned — and `None` still omits the attribute.
///
/// **Not emitted, because their conditions are unread:** `device_fanout`,
/// `category`, `t`, `expiration`, `recipient`/`recipient_pn`, and the
/// per-`<to>` `peer_recipient_lid` and `eph_setting` (`LX/Cp3->A07` gates the
/// former on `DeviceJid.isPrimary()` and an `LX/0DO` device resolved through an
/// inverse LID map).
///
/// Nothing in this function sends.
pub fn sender_key_stanza(
    chat: &Jid,
    id: &str,
    content: StanzaContent<'_>,
    addressing_mode: Option<&str>,
    phash: Option<&str>,
    targets: &[FanoutTarget],
    skmsg: &[u8],
) -> Result<Node> {
    if id.is_empty() || id.len() > 128 {
        return Err(Error::invalid("group_send", "invalid stanza id"));
    }
    // An empty fan-out is legitimate only when the distribution went out of
    // band: `includeSenderKeysInMessage` is a per-send flag, so a real client
    // does not always carry sender keys in the message.
    if skmsg.is_empty() || skmsg.len() > 1_048_576 {
        return Err(Error::invalid("group_send", "invalid skmsg size"));
    }
    // **Observed live 2026-11-01** (two own accounts, five controlled sends):
    // the server ACKs a group `type=reaction` stanza that carries
    // `<participants>` (sender-key distributions) but delivers it to nobody;
    // its ACK has no `count`. The same reaction with `type=text`, or as a
    // reaction with the distribution sent beforehand as a direct message and
    // no `<participants>`, was delivered and decrypted. Removing `meta`,
    // `decrypt-fail` or reordering `meta` before `enc` did not help. Refuse
    // the dropped shape here so no caller can build it again.
    if content == StanzaContent::Reaction && !targets.is_empty() {
        return Err(Error::invalid(
            "group_send",
            "a group reaction cannot carry sender-key distributions; deliver them out of band first",
        ));
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
        // A `<to>` names a *device*, and a device address goes on the wire as
        // an AD-JID — the form that carries an explicit agent and device byte —
        // even when both are zero. **Observed** 2026-09-22: every `participant`
        // the server stamps on an incoming group stanza decodes as `AgentDevice`
        // with `device=0`, while the group `from` beside it decodes as `Plain`.
        // Our `Jid` only reaches for the AD encoding when the agent or device is
        // non-zero, so a primary device would otherwise be encoded as a plain
        // user JID and the server has no device to deliver that enc to.
        let device = Jid::new_ad(
            target.device.user.clone(),
            target.device.agent,
            target.device.device,
            target.device.server.clone(),
        )?;
        participants.push(Node::new("to").attr("jid", device).children(vec![
                Node::new("enc")
                    .attr("v", "2")
                    .attr("type", target.kind)
                    .bytes(target.ciphertext.clone()),
            ]));
    }
    let mut node = Node::new("message")
        .attr("to", chat.clone())
        .attr("id", id)
        .attr("type", content.message_type());
    if let Some(phash) = phash {
        // APK 37.71 AzR.A03 produces "3:" as well as "2:". Accept the
        // explicit builder input without changing automatic version selection.
        if phash.len() > 32
            || !phash.starts_with(['1', '2', '3'])
            || phash.as_bytes().get(1) != Some(&b':')
        {
            return Err(Error::invalid(
                "group_send",
                "phash is a versioned `<n>:<base64>` value",
            ));
        }
        node = node.attr("phash", phash);
    }
    if let StanzaContent::Media(kind) = content
        && !matches!(kind, "image" | "ptt")
    {
        return Err(Error::invalid("group_send", "unsupported media type"));
    }
    if let Some(mode) = addressing_mode {
        if !matches!(mode, "pn" | "lid") {
            return Err(Error::invalid("group_send", "addressing_mode is pn or lid"));
        }
        node = node.attr("addressing_mode", mode);
    }
    let mut children = Vec::new();
    if !participants.is_empty() {
        children.push(Node::new("participants").children(participants));
    }
    let mut enc = Node::new("enc").attr("v", "2").attr("type", "skmsg");
    match content {
        StanzaContent::Text => {}
        StanzaContent::Reaction => enc = enc.attr("decrypt-fail", "hide"),
        StanzaContent::Media(kind) => enc = enc.attr("mediatype", kind),
    }
    children.push(enc.bytes(skmsg.to_vec()));
    if content == StanzaContent::Reaction {
        children.push(Node::new("meta").attr("content_type", "add_on"));
    }
    Ok(node.children(children))
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
        let node = sender_key_stanza(
            &chat,
            "M1",
            StanzaContent::Text,
            Some("lid"),
            Some("2:79jEAyGD"),
            &targets,
            &[9; 32],
        )
        .unwrap();
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
        // The device suffix on a target must survive into its `<to jid=…>`,
        // and every `<to>` must name a device in the AD form the server uses,
        // including the primary whose agent and device are both zero.
        for (child, device) in participants.child_nodes().iter().zip([1, 0]) {
            let jid = child.get_attr("jid").unwrap().as_jid().unwrap();
            assert_eq!(jid.device, device);
            assert_eq!(
                jid.kind,
                crate::model::jid::JidKind::AgentDevice,
                "a <to> names a device, so it encodes as an AD-JID"
            );
        }
        let skmsg = &children[1];
        assert_eq!(skmsg.description, "enc");
        assert_eq!(skmsg.get_attr("type").unwrap().as_text(), Some("skmsg"));

        // Confirmed from A06 and observed live on this group.
        assert_eq!(
            node.get_attr("addressing_mode").unwrap().as_text(),
            Some("lid")
        );
        // Nothing we could not read is emitted.
        for absent in ["device_fanout", "category", "t", "expiration"] {
            assert!(
                node.get_attr(absent).is_none(),
                "{absent} must not be guessed"
            );
        }
        // Omitting it is a choice the caller makes, not a default.
        assert!(
            sender_key_stanza(
                &chat,
                "M1",
                StanzaContent::Text,
                None,
                None,
                &targets,
                &[9; 32]
            )
            .unwrap()
            .get_attr("addressing_mode")
            .is_none()
        );
        assert!(
            sender_key_stanza(
                &chat,
                "M1",
                StanzaContent::Text,
                Some("web"),
                None,
                &targets,
                &[9; 32]
            )
            .is_err()
        );

        let dict = crate::binary::TokenDictionary::comparison_2_26_37_71();
        let encoded = crate::binary::Encoder::new(dict).encode(&node).unwrap();
        let decoded =
            crate::binary::Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
                .decode_node()
                .unwrap();
        assert_eq!(decoded, node, "the send stanza must survive the wire codec");
    }

    #[test]
    fn a_refused_participant_is_not_a_member() {
        let group = Node::new("group")
            .attr("id", "120363000000000001")
            .children(vec![
                Node::new("participant")
                    .attr("jid", "15551234567@s.whatsapp.net")
                    .attr("type", "superadmin"),
                Node::new("participant")
                    .attr("jid", "15559876543@s.whatsapp.net")
                    .attr("error", "403"),
            ]);
        let snapshot = parse_group(&group).unwrap();
        assert_eq!(snapshot.members.len(), 1);
        assert_eq!(snapshot.members[0].jid, "15551234567@s.whatsapp.net");
    }

    #[test]
    fn member_map_uses_user_identities_and_last_duplicate_role() {
        // Synthetic ADs.A05 branch fixture: typed user JIDs, skipped casts,
        // LinkedHashMap replacement, plus our observed create-error exclusion.
        let alice = Jid::parse("15550000001@s.whatsapp.net").unwrap();
        let bob = Jid::parse("90000000002@lid").unwrap();
        let mut children = vec![
            Node::new("participant")
                .attr("jid", alice.clone())
                .attr("type", "admin"),
            Node::new("participant").attr("jid", bob.clone()),
            Node::new("participant")
                .attr("jid", alice.to_string())
                .attr("type", "superadmin"),
            Node::new("participant")
                .attr("jid", bob.clone())
                .attr("error", "403"),
            Node::new("participant"),
        ];
        for invalid in [
            "",
            "broken",
            "s.whatsapp.net",
            "120363000000000001@g.us",
            "status@broadcast",
            "15550000001:1@s.whatsapp.net",
            "90000000002.1@lid",
        ] {
            children.push(Node::new("participant").attr("jid", invalid));
        }
        let node = Node::new("group")
            .attr("id", "120363000000000001@g.us")
            .children(children);
        let (_dir, s) = store();
        assert!(record_info_reply(&s, &Node::new("iq").children(vec![node])).unwrap());
        let snapshot = get(&s, &Jid::parse("120363000000000001@g.us").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(
            snapshot.members,
            vec![
                Member {
                    jid: alice.to_string(),
                    role: Some("superadmin".into())
                },
                Member {
                    jid: bob.to_string(),
                    role: None
                },
            ]
        );
        let events = s
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("member-map-test")?,
                    crate::model::ids::EventCursor::START,
                    64,
                )
            })
            .unwrap();
        let SyncEvent::GroupCreated(created) = &events.events[0].event else {
            panic!("expected group creation")
        };
        assert_eq!(
            created.participants.len(),
            2,
            "journal must not contain duplicate or invalid members"
        );
    }

    /// A reaction and a media status carry the same markers their direct
    /// envelopes do, on the `skmsg` that holds the content — and only there.
    #[test]
    fn group_notifications_other_than_dirty_are_acked_once() {
        let text = |n: &Node, k: &str| n.get_attr(k).and_then(|v| v.as_text()).map(str::to_owned);
        let note = |child: &str| {
            Node::new("notification")
                .attr("from", "120363000000000001@g.us")
                .attr("type", "w:gp2")
                .attr("id", "2468")
                .attr("participant", "111111111111111@lid")
                .children(vec![Node::new(child)])
        };
        for child in [
            "create",
            "add",
            "remove",
            "subject",
            "promote",
            "unknown_future",
        ] {
            let ack = notification_ack(&note(child)).unwrap().unwrap();
            assert_eq!(ack.description, "ack");
            assert_eq!(text(&ack, "id").as_deref(), Some("2468"));
            assert_eq!(text(&ack, "class").as_deref(), Some("notification"));
            assert_eq!(text(&ack, "type").as_deref(), Some("w:gp2"));
            assert!(ack.get_attr("to").is_some() && ack.get_attr("participant").is_some());
            assert!(ack.get_attr("error").is_none());
        }
        // groups_dirty has its own apply-then-ACK path; never ACKed twice.
        assert!(notification_ack(&note("groups_dirty")).unwrap().is_none());
        assert!(
            notification_ack(&Node::new("notification").attr("type", "psa"))
                .unwrap()
                .is_none()
        );
        let no_id = Node::new("notification")
            .attr("type", "w:gp2")
            .attr("from", "120363000000000001@g.us");
        assert!(notification_ack(&no_id).is_err());
    }

    #[test]
    fn sender_key_stanza_marks_reactions_and_media() {
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let target = FanoutTarget {
            device: Jid::parse("15551234567@s.whatsapp.net").unwrap(),
            kind: "pkmsg",
            ciphertext: vec![1],
        };
        let targets = std::slice::from_ref(&target);
        // The server drops a group reaction that carries distributions
        // (observed live 2026-11-01), so that shape is refused.
        let error = sender_key_stanza(
            &chat,
            "M",
            StanzaContent::Reaction,
            None,
            None,
            targets,
            &[1],
        )
        .unwrap_err();
        assert!(error.to_string().contains("out of band"), "{error}");
        let reaction =
            sender_key_stanza(&chat, "M", StanzaContent::Reaction, None, None, &[], &[1]).unwrap();
        assert!(reaction.child("participants").is_none());
        assert_eq!(
            reaction.get_attr("type").unwrap().as_text(),
            Some("reaction")
        );
        let skmsg = reaction
            .child_nodes()
            .iter()
            .find(|n| n.description == "enc")
            .unwrap();
        assert_eq!(
            skmsg.get_attr("decrypt-fail").unwrap().as_text(),
            Some("hide")
        );
        let meta = reaction.child("meta").unwrap();
        assert_eq!(
            meta.get_attr("content_type").unwrap().as_text(),
            Some("add_on")
        );

        let media = sender_key_stanza(
            &chat,
            "M",
            StanzaContent::Media("image"),
            None,
            None,
            targets,
            &[1],
        )
        .unwrap();
        assert_eq!(media.get_attr("type").unwrap().as_text(), Some("media"));
        let skmsg = media
            .child_nodes()
            .iter()
            .find(|n| n.description == "enc")
            .unwrap();
        assert_eq!(
            skmsg.get_attr("mediatype").unwrap().as_text(),
            Some("image")
        );
        assert!(media.child("meta").is_none());
        let to = &media.child("participants").unwrap().child_nodes()[0];
        assert!(to.child_nodes()[0].get_attr("mediatype").is_none());

        assert!(
            sender_key_stanza(
                &chat,
                "M",
                StanzaContent::Media("video"),
                None,
                None,
                targets,
                &[1]
            )
            .is_err(),
            "no media kind is sent before its direct envelope exists"
        );
    }

    #[test]
    fn sender_key_stanza_refuses_malformed_input() {
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let ok = FanoutTarget {
            device: Jid::parse("15551234567@s.whatsapp.net").unwrap(),
            kind: "pkmsg",
            ciphertext: vec![1],
        };
        // An empty fan-out is allowed (out-of-band distribution) and simply
        // omits `<participants>`; an empty skmsg and an empty id are not.
        let bare = sender_key_stanza(
            &chat,
            "M",
            StanzaContent::Text,
            Some("lid"),
            None,
            &[],
            &[1],
        )
        .unwrap();
        for phash in ["1:79jEAyGD", "2:79jEAyGD", "3:79jEAyGD"] {
            let stanza = sender_key_stanza(
                &chat,
                "M",
                StanzaContent::Text,
                Some("lid"),
                Some(phash),
                &[],
                &[1],
            )
            .unwrap();
            assert_eq!(
                stanza.get_attr("phash").and_then(|value| value.as_text()),
                Some(phash)
            );
        }
        assert!(
            sender_key_stanza(
                &chat,
                "M",
                StanzaContent::Text,
                Some("lid"),
                Some("4:79jEAyGD"),
                &[],
                &[1]
            )
            .is_err()
        );
        assert!(bare.child("participants").is_none());
        assert_eq!(bare.child_nodes().len(), 1);
        assert!(
            sender_key_stanza(
                &chat,
                "M",
                StanzaContent::Text,
                Some("lid"),
                None,
                std::slice::from_ref(&ok),
                &[]
            )
            .is_err()
        );
        assert!(
            sender_key_stanza(
                &chat,
                "",
                StanzaContent::Text,
                Some("lid"),
                None,
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
        assert!(
            sender_key_stanza(
                &chat,
                "M",
                StanzaContent::Text,
                Some("lid"),
                None,
                &[wrong],
                &[1]
            )
            .is_err()
        );
        // Empty per-device ciphertext.
        let empty = FanoutTarget {
            ciphertext: Vec::new(),
            ..ok
        };
        assert!(
            sender_key_stanza(
                &chat,
                "M",
                StanzaContent::Text,
                Some("lid"),
                None,
                &[empty],
                &[1]
            )
            .is_err()
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
    fn complete_group_classification_obeys_apk_child_precedence() {
        // APK 37.71 ADs.A01 0044..0064, A03 0014..002a; synthetic identities.
        // Text "false" still counts as presence; attributes do not classify.
        let group = Node::new("group")
            .attr("id", "120363000000000001")
            .attr("parent", "true")
            .attr("default_sub_group", "true");
        assert_eq!(
            parse_group(&group).unwrap().group_type,
            Some(GroupType::Ordinary)
        );
        let parent = Node::new("linked_parent").attr("jid", "120363000000000002@g.us");
        let cases = [
            (vec![parent.clone()], GroupType::LinkedSubgroup),
            (
                vec![parent.clone(), Node::new("general_chat").text("false")],
                GroupType::GeneralChat,
            ),
            (
                vec![
                    Node::new("general_chat"),
                    parent.clone(),
                    Node::new("default_sub_group"),
                ],
                GroupType::DefaultSubgroup,
            ),
            (
                vec![
                    Node::new("default_sub_group"),
                    parent,
                    Node::new("parent"),
                    Node::new("general_chat"),
                ],
                GroupType::Parent,
            ),
        ];
        for (children, expected) in cases {
            let snapshot = parse_group(&group.clone().children(children)).unwrap();
            assert_eq!(snapshot.group_type, Some(expected));
            assert_eq!(
                snapshot.linked_parent.as_deref(),
                Some("120363000000000002@g.us")
            );
        }
        for raw in [
            None,
            Some("15551234567@s.whatsapp.net"),
            Some("120363000000000002:1@g.us"),
            Some("@g.us"),
        ] {
            let mut parent = Node::new("linked_parent");
            if let Some(raw) = raw {
                parent = parent.attr("jid", raw);
            }
            let snapshot = parse_group(&group.clone().children(vec![parent])).unwrap();
            assert_eq!(snapshot.group_type, Some(GroupType::LinkedSubgroup));
            assert_eq!(snapshot.linked_parent, None);
        }
        assert!(parse_group(&group.attr("truncated", "true")).is_none());
    }

    #[test]
    fn group_classification_survives_reopen_without_guessing_historical_rows() {
        let (dir, s) = store();
        let jid = Jid::parse("120363000000000001@g.us").unwrap();
        let reply = Node::new("iq").children(vec![
            Node::new("group")
                .attr("id", jid.clone())
                .attr("subject", "Example")
                .children(vec![
                    Node::new("general_chat"),
                    Node::new("linked_parent").attr("jid", "120363000000000002@g.us"),
                ]),
        ]);
        assert!(record_info_reply(&s, &reply).unwrap());
        let current = get(&s, &jid).unwrap().unwrap();
        let mut old = serde_json::to_value(&current).unwrap();
        old.as_object_mut().unwrap().remove("group_type");
        old.as_object_mut().unwrap().remove("linked_parent");
        let historical: GroupSnapshot = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(historical.group_type, None);
        assert_eq!(historical.linked_parent, None);
        let old_jid = "120363000000000003@g.us";
        old["jid"] = serde_json::json!(old_jid);
        s.write(|tx| tx.put_json(Table::Checkpoints, key(old_jid).as_bytes(), 1, &old))
            .unwrap();
        drop(s);
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(get(&s, &jid).unwrap(), Some(current));
        let historical = get(&s, &Jid::parse(old_jid).unwrap()).unwrap().unwrap();
        assert_eq!(historical.group_type, None);
        assert_eq!(historical.linked_parent, None);
    }

    #[test]
    fn mixed_refresh_outcome_and_full_snapshot_commit_once_across_restart() {
        let (dir, s) = store();
        let groups: Vec<Jid> = (0..4)
            .map(|n| Jid::parse(&format!("12345-6789{n}@g.us")).unwrap())
            .collect();
        for group in &groups {
            record_info_reply(
                &s,
                &Node::new("iq").children(vec![
                    Node::new("group")
                        .attr("id", group.clone())
                        .attr("subject", "Before"),
                ]),
            )
            .unwrap();
        }
        let id = queue_refresh(&s, &groups).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        // APK 37.71 AX9 size=0..19999 plus exact truncated=true;
        // AXA/AXB classify exact group@error 403/404. Synthetic group IDs.
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("groups").children(vec![
                    Node::new("group")
                        .attr("id", groups[0].clone())
                        .attr("subject", "After"),
                    Node::new("group")
                        .attr("id", groups[1].clone())
                        .attr("truncated", "true")
                        .attr("size", "19999"),
                    Node::new("group")
                        .attr("id", groups[2].clone())
                        .attr("error", "403"),
                    Node::new("group")
                        .attr("id", groups[3].clone())
                        .attr("error", "404"),
                ])]);
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        let (outcome, saved) = s
            .read(|tx| super::super::actions::view_in(tx, &id))
            .unwrap()
            .unwrap();
        let request = super::super::actions::request(&s, &id).unwrap();
        let abort: Result<()> = s.write(|tx| {
            assert!(ingest_reply_in(
                tx,
                "group-refresh",
                &outcome,
                request.as_ref(),
                saved.as_ref()
            )?);
            assert!(
                tx.get_json::<RefreshOutcome>(
                    Table::Checkpoints,
                    format!("group-refresh-result:{id}").as_bytes(),
                    1
                )?
                .is_some()
            );
            Err(invalid("abort mixed projection"))
        });
        assert!(abort.is_err());
        assert!(refresh_outcome(&s, &id).unwrap().is_none());
        assert!(
            list(&s)
                .unwrap()
                .iter()
                .all(|g| g.subject.as_deref() == Some("Before"))
        );
        assert!(apply_pending(&s, &id).unwrap());
        drop(s);
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(
            refresh_outcome(&s, &id).unwrap().unwrap(),
            RefreshOutcome {
                complete: 1,
                truncated: vec![TruncatedGroup {
                    jid: groups[1].to_string(),
                    size: 19999
                }],
                forbidden: 1,
                not_found: 1,
                pending: vec![TruncatedGroup {
                    jid: groups[1].to_string(),
                    size: 19999
                }],
                ..Default::default()
            }
        );
        assert_eq!(
            get(&s, &groups[0]).unwrap().unwrap().subject.as_deref(),
            Some("After")
        );
        for group in &groups[1..] {
            assert_eq!(
                get(&s, group).unwrap().unwrap().subject.as_deref(),
                Some("Before")
            );
        }
        assert_eq!(recover_pending(&s).unwrap(), 0);
        assert_eq!(ingest(&s).unwrap(), 0);
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
        assert_eq!(super::super::actions::reply(&s, &id).unwrap(), Some(reply));
        let events = s
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("mixed-test")?,
                    crate::model::ids::EventCursor::START,
                    64,
                )
            })
            .unwrap();
        assert_eq!(events.events.len(), 5);
    }

    #[test]
    fn follow_up_is_atomic_ordered_bounded_and_once_only_after_reopen() {
        let (dir, mut s) = store();
        let groups: Vec<Jid> = (0..3)
            .map(|n| Jid::parse(&format!("12345-6789{n}@g.us")).unwrap())
            .collect();
        let mut id = queue_refresh_follow_up(&s, &groups).unwrap();
        for round in 0..=2 {
            let queued = super::super::actions::reserve_pending(&s).unwrap();
            assert_eq!(queued.len(), 1);
            assert_eq!(text(&queued[0], "id").as_deref(), Some(id.as_str()));
            let query = queued[0].child("query").unwrap();
            assert_eq!(
                text(query, "context").as_deref(),
                Some("per_group_dirty_recovery_truncatable")
            );
            // Source-derived bounded size, deliberately reverse server ordering.
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![
                    Node::new("groups").children(
                        query
                            .child_nodes()
                            .iter()
                            .rev()
                            .map(|g| {
                                Node::new("group")
                                    .attr("id", text(g, "jid").unwrap())
                                    .attr("truncated", "true")
                                    .attr("size", "7000")
                            })
                            .collect(),
                    ),
                ]);
            super::super::actions::record_reply(&s, &reply).unwrap();
            if round == 0 {
                let (action, saved) = s
                    .read(|tx| super::super::actions::view_in(tx, &id))
                    .unwrap()
                    .unwrap();
                let abort: Result<()> = s.write(|tx| {
                    ingest_reply_in(
                        tx,
                        "group-refresh",
                        &action,
                        Some(&queued[0]),
                        saved.as_ref(),
                    )?;
                    Err(invalid("abort continuation"))
                });
                assert!(abort.is_err());
                assert!(refresh_outcome(&s, &id).unwrap().is_none());
                assert!(
                    super::super::actions::reserve_pending(&s)
                        .unwrap()
                        .is_empty()
                );
            }
            assert!(apply_pending(&s, &id).unwrap());
            let report = refresh_outcome(&s, &id).unwrap().unwrap();
            assert_eq!(report.round, round);
            assert!(report.automatic_follow_up);
            assert_eq!(
                report
                    .pending
                    .iter()
                    .map(|g| g.jid.clone())
                    .collect::<Vec<_>>(),
                groups.iter().map(ToString::to_string).collect::<Vec<_>>()
            );
            assert_eq!(report.exhausted, round == 2);
            assert_eq!(report.next_action.is_some(), round < 2);
            drop(s);
            s = ProtocolStore::open(OpenOptions {
                path: dir.path().join("p.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
                create: false,
            })
            .unwrap();
            assert_eq!(refresh_outcome(&s, &id).unwrap(), Some(report.clone()));
            assert_eq!(recover_pending(&s).unwrap(), 0);
            assert_eq!(ingest(&s).unwrap(), 0);
            if let Some(next) = report.next_action {
                let request = super::super::actions::request(&s, &next).unwrap().unwrap();
                // 7000 + 7000 exceeds local floor: only first group queried.
                assert_eq!(request.child("query").unwrap().child_nodes().len(), 1);
                id = next;
            }
        }
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn follow_up_excludes_completed_errors_and_never_replays_uncertain_queries() {
        let (_dir, s) = store();
        let groups: Vec<Jid> = (0..4)
            .map(|n| Jid::parse(&format!("12345-6789{n}@g.us")).unwrap())
            .collect();
        let id = queue_refresh_follow_up(&s, &groups).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
        assert_eq!(recover_pending(&s).unwrap(), 0);
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("groups").children(vec![
                    Node::new("group")
                        .attr("id", groups[0].clone())
                        .attr("subject", "Complete"),
                    Node::new("group")
                        .attr("id", groups[1].clone())
                        .attr("error", "403"),
                    Node::new("group")
                        .attr("id", groups[2].clone())
                        .attr("error", "404"),
                    Node::new("group")
                        .attr("id", groups[3].clone())
                        .attr("truncated", "true")
                        .attr("size", "0"),
                ])]);
        super::super::actions::record_reply(&s, &reply).unwrap();
        assert!(apply_pending(&s, &id).unwrap());
        let report = refresh_outcome(&s, &id).unwrap().unwrap();
        assert_eq!(
            (report.complete, report.forbidden, report.not_found),
            (1, 1, 1)
        );
        assert_eq!(report.pending.len(), 1);
        let next = report.next_action.unwrap();
        let queued = super::super::actions::reserve_pending(&s).unwrap();
        assert_eq!(queued.len(), 1);
        assert_eq!(
            text(&queued[0].child("query").unwrap().child_nodes()[0], "jid"),
            Some(groups[3].to_string())
        );
        let refusal = Node::new("iq")
            .attr("id", next.as_str())
            .attr("type", "error")
            .children(vec![Node::new("error").attr("code", "403")]);
        super::super::actions::record_reply(&s, &refusal).unwrap();
        apply_pending(&s, &next).unwrap();
        assert!(refresh_outcome(&s, &next).unwrap().is_none());
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn follow_up_stops_on_complete_unknown_and_root_refusal() {
        for case in ["complete", "unknown", "refused"] {
            let (_dir, s) = store();
            let group = Jid::parse("12345-67890@g.us").unwrap();
            let id = queue_refresh_follow_up(&s, std::slice::from_ref(&group)).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let mut node = Node::new("group").attr("id", group);
            if case == "unknown" {
                node = node.attr("error", "500");
            }
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", if case == "refused" { "error" } else { "result" })
                .children(if case == "refused" {
                    vec![Node::new("error").attr("code", "403")]
                } else {
                    vec![Node::new("groups").children(vec![node])]
                });
            super::super::actions::record_reply(&s, &reply).unwrap();
            assert_eq!(apply_pending(&s, &id).unwrap(), case != "unknown");
            let report = refresh_outcome(&s, &id).unwrap();
            assert_eq!(report.is_some(), case == "complete");
            if let Some(report) = report {
                assert_eq!(report.complete, 1);
                assert!(report.pending.is_empty());
                assert!(report.next_action.is_none());
                assert!(!report.exhausted);
            }
            assert!(
                super::super::actions::reserve_pending(&s)
                    .unwrap()
                    .is_empty()
            );
        }
    }

    fn dirty_fixture() -> Node {
        Node::new("notification")
            .attr("from", "s.whatsapp.net")
            .attr("id", "DIRTY1")
            .attr("type", "w:gp2")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("recipient", "15555550101@s.whatsapp.net")
            .attr("category", "ignored")
            .attr("edit", "7")
            .children(vec![Node::new("groups_dirty").children(vec![
                Node::new("group"), // APK logs/skips missing jid.
                Node::new("group").attr("jid", "12345-67890@g.us"),
            ])])
    }

    #[test]
    fn dirty_notifications_queue_before_ack_and_deduplicate_across_restart() {
        let (dir, s) = store();
        let node = dirty_fixture();
        let ack = handle_dirty_notification(&s, &node).unwrap().unwrap();
        for name in ["recipient", "category", "edit"] {
            assert!(ack.get_attr(name).is_none());
        }
        assert_eq!(text(&ack, "participant"), text(&node, "participant"));
        assert_eq!(text(&ack, "to"), Some("s.whatsapp.net".into()));
        let actions = super::super::actions::outcomes(&s).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].label, "refresh dirty groups");
        let request = super::super::actions::reserve_pending(&s).unwrap();
        assert_eq!(request.len(), 1);
        assert_eq!(
            text(request[0].child("query").unwrap(), "context").as_deref(),
            Some("per_group_dirty_recovery_truncatable")
        );
        drop(s);
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(handle_dirty_notification(&s, &node).unwrap(), Some(ack));
        assert_eq!(super::super::actions::outcomes(&s).unwrap().len(), 1);
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
        let changed = Node::new("notification")
            .attr("from", "s.whatsapp.net")
            .attr("id", "DIRTY1")
            .attr("type", "w:gp2")
            .children(vec![Node::new("groups_dirty").children(vec![
                Node::new("group").attr("jid", "12345-67891@g.us"),
            ])]);
        assert!(handle_dirty_notification(&s, &changed).unwrap().is_none());
        assert_eq!(super::super::actions::outcomes(&s).unwrap().len(), 1);
    }

    #[test]
    fn unsupported_dirty_notifications_do_not_ack_or_queue() {
        for case in [
            "empty",
            "bad-jid",
            "duplicate",
            "other-child",
            "missing-id",
            "bad-participant",
        ] {
            let (_dir, s) = store();
            let mut node = Node::new("notification")
                .attr("from", "s.whatsapp.net")
                .attr("type", "w:gp2");
            if case != "missing-id" {
                node = node.attr("id", "fixture");
            }
            if case == "bad-participant" {
                node = node.attr("participant", "user:broken@s.whatsapp.net");
            }
            let group = Node::new("group").attr(
                "jid",
                if case == "bad-jid" {
                    "12345@s.whatsapp.net"
                } else {
                    "12345-67890@g.us"
                },
            );
            let children = match case {
                "empty" => vec![],
                "duplicate" => vec![group.clone(), group],
                "other-child" => vec![Node::new("unknown")],
                _ => vec![group],
            };
            node = node.children(vec![Node::new("groups_dirty").children(children)]);
            assert!(
                handle_dirty_notification(&s, &node).unwrap().is_none(),
                "{case}"
            );
            assert!(super::super::actions::outcomes(&s).unwrap().is_empty());
        }
    }

    #[test]
    fn historical_report_inspection_recovers_pending_count_without_queueing() {
        let (_dir, s) = store();
        s.write(|tx| tx.put_json(Table::Checkpoints, b"group-refresh-result:old", 1,
            &serde_json::json!({"complete":0,"truncated":[{"jid":"12345-67890@g.us","size":100}],"forbidden":0,"not_found":0}))).unwrap();
        let report = refresh_outcome(&s, "old").unwrap().unwrap();
        assert_eq!(report.pending, report.truncated);
        assert!(!report.automatic_follow_up);
        assert!(report.next_action.is_none());
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn truncated_refresh_size_requires_apk_bounds_and_explicit_value() {
        for size in [
            None,
            Some("-1"),
            Some("20000"),
            Some("invalid"),
            Some("0"),
            Some("19999"),
        ] {
            let (_dir, s) = store();
            let group = Jid::parse("12345-67890@g.us").unwrap();
            let id = queue_refresh(&s, std::slice::from_ref(&group)).unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let mut node = Node::new("group")
                .attr("id", group)
                .attr("truncated", "true");
            if let Some(size) = size {
                node = node.attr("size", size);
            }
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("groups").children(vec![node])]);
            super::super::actions::record_reply(&s, &reply).unwrap();
            assert_eq!(
                apply_pending(&s, &id).unwrap(),
                matches!(size, Some("0" | "19999"))
            );
            assert_eq!(
                refresh_outcome(&s, &id).unwrap().is_some(),
                matches!(size, Some("0" | "19999"))
            );
            assert!(list(&s).unwrap().is_empty());
        }
    }

    #[test]
    fn batch_refresh_applies_only_complete_correlated_snapshots_atomically() {
        for case in [
            "complete",
            "truncated",
            "missing",
            "duplicate",
            "unexpected",
            "error",
        ] {
            let (_dir, s) = store();
            let first = Jid::parse("12345-67890@g.us").unwrap();
            let second = Jid::parse("12345-67891@g.us").unwrap();
            for group in [&first, &second] {
                record_info_reply(
                    &s,
                    &Node::new("iq").children(vec![
                        Node::new("group")
                            .attr("id", group.clone())
                            .attr("subject", "Before"),
                    ]),
                )
                .unwrap();
            }
            let before = serde_json::to_value(list(&s).unwrap()).unwrap();
            let id = queue_refresh(&s, &[first.clone(), second.clone()]).unwrap();
            assert!(queue_refresh(&s, &[]).is_err());
            assert!(queue_refresh(&s, &[first.clone(), first.clone()]).is_err());
            assert!(queue_refresh(&s, &[Jid::parse("g.us").unwrap()]).is_err());
            assert!(
                queue_refresh(&s, &[Jid::parse("15555550100@s.whatsapp.net").unwrap()]).is_err()
            );
            let request = super::super::actions::reserve_pending(&s)
                .unwrap()
                .pop()
                .unwrap();
            // Literal APK 37.71 EJy selector 4 / 9QE selector 11 shape.
            assert_eq!(attr_string(&request, "to").as_deref(), Some("g.us"));
            assert_eq!(attr_string(&request, "xmlns").as_deref(), Some("w:g2"));
            let query = request.child("query").unwrap();
            assert_eq!(
                attr_string(query, "context").as_deref(),
                Some("per_group_dirty_recovery")
            );
            assert_eq!(query.child_nodes().len(), 2);
            assert_eq!(
                attr_string(&query.child_nodes()[0], "jid"),
                Some(first.to_string())
            );
            let full = |jid: &Jid| {
                Node::new("group")
                    .attr("id", jid.clone())
                    .attr("subject", "After")
            };
            let mut children = vec![full(&first), full(&second)];
            match case {
                "truncated" => children[1] = full(&second).attr("truncated", "true"),
                "missing" => {
                    children.pop();
                }
                "duplicate" => children[1] = full(&first),
                "unexpected" => children[1] = full(&Jid::parse("12345-67892@g.us").unwrap()),
                "error" => children[1] = full(&second).attr("error", "500"),
                _ => {}
            }
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("groups").children(children)]);
            assert!(super::super::actions::record_reply(&s, &reply).unwrap());
            assert_eq!(
                apply_pending(&s, &id).unwrap(),
                case == "complete",
                "{case}"
            );
            if case == "complete" {
                assert!(
                    list(&s)
                        .unwrap()
                        .iter()
                        .all(|g| g.subject.as_deref() == Some("After"))
                );
                assert!(!apply_pending(&s, &id).unwrap());
                assert_eq!(ingest(&s).unwrap(), 0);
            } else {
                assert_eq!(
                    serde_json::to_value(list(&s).unwrap()).unwrap(),
                    before,
                    "{case}"
                );
                assert!(ingest(&s).is_err());
                assert_eq!(
                    s.read(|tx| tx.get_json::<bool>(
                        Table::Checkpoints,
                        format!("{PENDING_PREFIX}{id}").as_bytes(),
                        1
                    ))
                    .unwrap(),
                    Some(true)
                );
            }
            assert_eq!(super::super::actions::reply(&s, &id).unwrap(), Some(reply));
            assert!(
                super::super::actions::reserve_pending(&s)
                    .unwrap()
                    .is_empty()
            );
            let events = s
                .read(|tx| {
                    tx.read_events(
                        &crate::store::ConsumerId::new("batch-refresh-test")?,
                        crate::model::ids::EventCursor::START,
                        64,
                    )
                })
                .unwrap();
            assert_eq!(
                events.events.len(),
                if case == "complete" { 4 } else { 2 },
                "{case}"
            );
        }
    }

    #[test]
    fn session_pending_projection_survives_restart_and_skips_truncated_reply() {
        let (dir, s) = store();
        let first = Jid::parse("12345-67890@g.us").unwrap();
        let second = Jid::parse("12345-67891@g.us").unwrap();
        let incomplete = queue_info(&s, &first).unwrap();
        let complete = queue_info(&s, &second).unwrap();
        super::super::actions::reserve_pending(&s).unwrap();
        for (id, group) in [
            (
                &incomplete,
                Node::new("group")
                    .attr("id", first.clone())
                    .attr("truncated", "true"),
            ),
            (
                &complete,
                Node::new("group")
                    .attr("id", second.clone())
                    .attr("subject", "Saved before interruption"),
            ),
        ] {
            assert!(
                super::super::actions::record_reply(
                    &s,
                    &Node::new("iq")
                        .attr("id", id.as_str())
                        .attr("type", "result")
                        .children(vec![group])
                )
                .unwrap()
            );
        }
        assert!(
            list(&s).unwrap().is_empty(),
            "only settlement has committed"
        );
        drop(s);
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(recover_pending(&s).unwrap(), 1);
        assert!(get(&s, &first).unwrap().is_none());
        assert_eq!(
            get(&s, &second).unwrap().unwrap().subject.as_deref(),
            Some("Saved before interruption")
        );
        assert_eq!(recover_pending(&s).unwrap(), 0);
        assert!(!apply_pending(&s, &complete).unwrap());
        assert_eq!(
            s.read(|tx| tx.get_json::<bool>(
                Table::Checkpoints,
                format!("{PENDING_PREFIX}{incomplete}").as_bytes(),
                1
            ))
            .unwrap(),
            Some(true)
        );
        assert!(
            super::super::actions::reply(&s, &incomplete)
                .unwrap()
                .is_some()
        );
        assert!(
            super::super::actions::reserve_pending(&s)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn ingestion_snapshot_invite_leave_and_marker_rollback_together() {
        for kind in ["group-info", "group-invite", "group-leave"] {
            let (dir, s) = store();
            let group = Jid::parse("120363000000000001@g.us").unwrap();
            let initial = Node::new("group")
                .attr("id", group.clone())
                .attr("subject", "Before")
                .children(vec![
                    Node::new("participant").attr("jid", "15550000001@s.whatsapp.net"),
                ]);
            record_info_reply(&s, &Node::new("iq").children(vec![initial.clone()])).unwrap();
            let before = serde_json::to_value(get(&s, &group).unwrap()).unwrap();
            let id = match kind {
                "group-info" => queue_info(&s, &group),
                "group-invite" => queue_invite(&s, &group, false),
                _ => queue_leave(&s, &group),
            }
            .unwrap();
            super::super::actions::reserve_pending(&s).unwrap();
            let children = match kind {
                "group-info" => vec![initial.attr("subject", "After")],
                "group-invite" => vec![Node::new("invite").attr("code", "synthetic-code")],
                _ => vec![],
            };
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(children);
            assert!(super::super::actions::record_reply(&s, &reply).unwrap());
            let outcome = super::super::actions::outcomes(&s)
                .unwrap()
                .into_iter()
                .find(|outcome| outcome.id == id)
                .unwrap();
            let request = super::super::actions::request(&s, &id).unwrap();
            let marker = format!("{INGESTED_PREFIX}{id}");
            let failed: Result<()> = s.write(|tx| {
                assert!(ingest_reply_in(
                    tx,
                    kind,
                    &outcome,
                    request.as_ref(),
                    Some(&reply)
                )?);
                assert_eq!(
                    tx.get_json::<bool>(Table::Checkpoints, marker.as_bytes(), 1)?,
                    Some(true)
                );
                assert_eq!(
                    tx.get_json::<bool>(
                        Table::Checkpoints,
                        format!("{PENDING_PREFIX}{id}").as_bytes(),
                        1
                    )?,
                    None
                );
                Err(invalid("test rollback after snapshot and marker writes"))
            });
            assert!(failed.is_err());
            drop(s);
            let reopen = || {
                ProtocolStore::open(OpenOptions {
                    path: dir.path().join("p.redb"),
                    account_id: AccountId::new("a").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
                    create: false,
                })
                .unwrap()
            };
            let s = reopen();
            assert_eq!(
                serde_json::to_value(get(&s, &group).unwrap()).unwrap(),
                before
            );
            assert_eq!(
                s.read(|tx| tx.get_json::<bool>(Table::Checkpoints, marker.as_bytes(), 1))
                    .unwrap(),
                None
            );
            let event_count = |s: &ProtocolStore| {
                s.read(|tx| {
                    tx.read_events(
                        &crate::store::ConsumerId::new("atomic-group-test")?,
                        crate::model::ids::EventCursor::START,
                        64,
                    )
                })
                .unwrap()
                .events
                .len()
            };
            assert_eq!(
                s.read(|tx| tx.get_json::<bool>(
                    Table::Checkpoints,
                    format!("{PENDING_PREFIX}{id}").as_bytes(),
                    1
                ))
                .unwrap(),
                Some(true),
                "pending recovery survives aborted projection"
            );
            assert_eq!(event_count(&s), 1, "aborted subject event must roll back");
            assert_eq!(ingest(&s).unwrap(), 1);
            assert_eq!(
                recover_pending(&s).unwrap(),
                0,
                "CLI ingestion clears session recovery marker"
            );
            let after = get(&s, &group).unwrap();
            match kind {
                "group-info" => assert_eq!(after.unwrap().subject.as_deref(), Some("After")),
                "group-invite" => assert_eq!(
                    after.unwrap().invite_code.as_deref(),
                    Some("synthetic-code")
                ),
                _ => assert!(after.is_none()),
            }
            assert_eq!(event_count(&s), if kind == "group-info" { 2 } else { 1 });
            drop(s);
            let s = reopen();
            assert_eq!(ingest(&s).unwrap(), 0, "committed marker survives restart");
            assert_eq!(event_count(&s), if kind == "group-info" { 2 } else { 1 });
        }
    }

    #[test]
    fn truncated_replies_preserve_members_events_and_saved_action_after_restart() {
        let (dir, s) = store();
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let complete = Node::new("iq").children(vec![
            Node::new("group")
                .attr("id", group.clone())
                .attr("subject", "Original")
                .children(vec![
                    Node::new("participant").attr("jid", "15550000001@s.whatsapp.net"),
                    Node::new("participant").attr("jid", "15550000002@s.whatsapp.net"),
                ]),
        ]);
        assert!(record_info_reply(&s, &complete).unwrap());
        let before = serde_json::to_value(get(&s, &group).unwrap()).unwrap();
        // APK ADs.A0C returns the id/size result before reading full metadata.
        // Even a partial participant child must not become a complete snapshot.
        let partial = Node::new("group")
            .attr("id", group.clone())
            .attr("truncated", "true")
            .attr("size", "2")
            .children(vec![
                Node::new("participant").attr("jid", "15550000001@s.whatsapp.net"),
            ]);
        assert!(parse_group(&partial).is_none());
        let id = queue_info(&s, &group).unwrap();
        let _ = super::super::actions::reserve_pending(&s).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![partial]);
        assert!(
            record_info_reply(&s, &reply)
                .unwrap_err()
                .to_string()
                .contains("existing snapshot preserved")
        );
        assert!(super::super::actions::record_reply(&s, &reply).unwrap());
        assert!(
            ingest(&s)
                .unwrap_err()
                .to_string()
                .contains("requires refresh")
        );
        drop(s);
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
            create: false,
        })
        .unwrap();
        assert!(ingest(&s).is_err(), "unprocessed reply remains diagnosable");
        assert_eq!(
            serde_json::to_value(get(&s, &group).unwrap()).unwrap(),
            before
        );
        assert_eq!(super::super::actions::reply(&s, &id).unwrap(), Some(reply));
        let marker: Option<bool> = s
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("{INGESTED_PREFIX}{id}").as_bytes(),
                    1,
                )
            })
            .unwrap();
        assert!(marker.is_none());
        let events = s
            .read(|tx| {
                tx.read_events(
                    &crate::store::ConsumerId::new("truncated-test")?,
                    crate::model::ids::EventCursor::START,
                    64,
                )
            })
            .unwrap();
        assert_eq!(events.events.len(), 1);
        assert_eq!(events.events[0].event.kind(), "group_created");
    }

    #[test]
    fn truncated_flag_validation_keeps_complete_results_available() {
        let group = Node::new("group").attr("id", "120363000000000001@g.us");
        assert!(parse_group(&group).is_some());
        for flag in ["false", "FALSE"] {
            assert!(parse_group(&group.clone().attr("truncated", flag)).is_some());
        }
        for flag in ["true", "TRUE", "", "unknown", "0"] {
            let node = group.clone().attr("truncated", flag);
            assert!(parse_group(&node).is_none());
            assert!(require_complete_group(&node).is_err());
        }
        let non_text = group.attr("truncated", Jid::parse("g.us").unwrap());
        assert!(parse_group(&non_text).is_none());
        assert!(require_complete_group(&non_text).is_err());
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
