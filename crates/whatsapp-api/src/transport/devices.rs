//! Contact device-list synchronization (`usync` `devices` version 2), traced
//! from comparison APK **2.26.37.71**; class names below are that build's and
//! match the 2.26.37.70 references for `LX/11Q`, `LX/11T` and `LX/17V`.
//!
//! * Query `LX/0u7.A06` `0e52..0fd4`: `iq xmlns=usync/id/type=get` (no `to`)
//!   wrapping `usync` with `sid`, `index`, `last`, `mode` (`LX/0xb`: full,
//!   delta, query, download), `context` (`LX/0xY`: interactive, background,
//!   notification, message, …) and optional `allow_mutation`; children `query`
//!   (here `devices version="2"`, `0d8c..0db2`), `list` of `user jid=…` and an
//!   optional `side_list`. `SyncDevicesJob.A0G` selects presets from `LX/0xX`:
//!   the default is `INTERACTIVE_MULTI_PROTOCOL_QUERY` (mode `query`, context
//!   `interactive`); notification-driven syncs use context `notification`.
//!   Per-user known `device_hash`/`ts`/`expected_ts` hints (`0558..05b4`) are not
//!   sent; the server then returns full lists.
//! * Result `LX/0u7.C8f` `0a54..0bb8` (`parseDeviceData v2`): `usync/list/user
//!   [jid]/devices/device-list/device[id, key-index, is_hosted]` plus
//!   `devices/key-index-list[ts, expected_ts]` whose bytes are the signed ADV
//!   key-index list; per-user `devices/error[code,text]` marks a partial failure.
//!   Hosted devices are refused for E2EE (`LX/0bN.A0H` "cannot add hosted device
//!   with adv e2ee").
//! * Verification `LX/11T.A07`: the signed list is checked with the contact's
//!   pinned primary identity when one is stored ("local"); the ADV-provided key
//!   branch ("adv") is not implemented here, so contacts without a pinned
//!   identity keep only their primary device. Companion filtering follows
//!   `LX/11T.A08` (`VerifiedIndexList::reconcile_devices`).
//! * Notification `LX/11Q.A09`: `notification type=devices` carries `add` or
//!   `remove` with `device[jid,key-index,lid]` children, `device_hash`,
//!   optional `device_lid_hash` and `key-index-list[ts]`; the client records the
//!   user for a device sync (`LX/101.A02`, `pending_users_to_sync_device`) that
//!   `SyncDevicesJob` performs. Own-account `devices` notifications go through
//!   `LX/17V.A09`/`LX/1Tc.A01` account sync, which is not implemented; they are
//!   captured and acknowledged only.
//!
//! Local policy: a query is a read, so an unanswered query leaves its users
//! pending and a later session asks again. Lists are persisted atomically per
//! user; a raw-ID change or a stale timestamp keeps the saved list. `sid` is a
//! client-chosen opaque identifier like our message ids.
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::{
        jid::{Jid, Server},
        time::Timestamp,
    },
    signal::adv::{DeviceHashVersion, ReconciledDevices, VerifiedIndexList},
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Users per query; the APK batches phonebook syncs far larger, this is local.
pub const MAX_USERS_PER_QUERY: usize = 64;
const PENDING_PREFIX: &str = "device-sync-pending:";
const QUERY_PREFIX: &str = "device-sync:";
const LIST_PREFIX: &str = "devices:";

fn invalid(reason: &'static str) -> Error {
    Error::invalid("devices", reason)
}
fn pending_key(user: &Jid) -> String {
    format!("{PENDING_PREFIX}{user}")
}
fn list_key(user: &Jid) -> String {
    format!("{LIST_PREFIX}{user}")
}
fn query_key(iq_id: &str) -> String {
    format!("{QUERY_PREFIX}{iq_id}")
}
fn primary_of(jid: &Jid) -> Result<Jid> {
    if !matches!(jid.server, Server::User | Server::Lid) || jid.user.is_empty() {
        return Err(invalid(
            "device lists exist for user or LID identities only",
        ));
    }
    Ok(jid.to_non_ad())
}

/// Sync context string (`LX/0xY`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncContext {
    /// Started by the application (`INTERACTIVE_MULTI_PROTOCOL_QUERY`).
    Interactive,
    /// Prompted by a `devices` notification (`NOTIFICATION_MULTI_PROTOCOL_QUERY`).
    Notification,
    /// Needed to send a message (`MESSAGE_MULTI_PROTOCOL_QUERY`).
    Message,
}
impl SyncContext {
    fn as_str(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Notification => "notification",
            Self::Message => "message",
        }
    }
}

/// One advertised device of a contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceEntry {
    /// Transport device JID (`user:device@server`).
    pub jid: Jid,
    /// ADV key index (0 for the primary); never equal to `jid.device`.
    pub key_index: u32,
}
/// Persisted, account-independent device list for one contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceList {
    /// Contact primary identity (non-AD).
    pub owner: Jid,
    /// Retained devices; always includes the primary.
    pub devices: Vec<DeviceEntry>,
    /// Signed list account raw id, once a list was verified.
    pub raw_id: Option<u32>,
    /// Verified list timestamp (`key-index-list/@ts`).
    pub list_timestamp: Option<u64>,
    /// Server's `expected_ts`, if given.
    pub expected_timestamp: Option<u64>,
    /// Companions came from a verified signed list; false means primary only.
    pub companions_verified: bool,
    /// Why companions were not verified, sanitized.
    pub unverified_reason: Option<String>,
    /// `device_hash` from the latest notification, if any.
    pub notified_hash: Option<String>,
    /// Whether the retained set matches `notified_hash` (either hash version).
    pub hash_matches: Option<bool>,
    /// Hosted devices skipped for E2EE.
    pub hosted_skipped: u32,
    /// Local time of this ingest.
    pub fetched_at: Timestamp,
}
/// Pending-user marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pending {
    requested_at: Timestamp,
    notified_hash: Option<String>,
    reason: String,
}
/// Reserved query record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryRecord {
    /// Users asked about.
    pub users: Vec<Jid>,
    /// `outcome_unknown`, `server_acknowledged` or `refused`.
    pub status: String,
    /// Sanitized error code for a refusal.
    pub code: Option<String>,
    /// Users whose lists were stored from this reply.
    pub ingested: u32,
    /// Users with a per-user error in this reply.
    pub partial_errors: u32,
    /// Reservation time.
    pub requested_at: Timestamp,
}

/// Parsed `notification type=devices` (`LX/11Q.A09`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicesNotification {
    /// Contact (or own account) primary the notification is about.
    pub owner: Jid,
    /// `add` (true) or `remove` (false).
    pub added: bool,
    /// Devices listed in the child.
    pub devices: Vec<DeviceEntry>,
    /// `device_hash` attribute.
    pub device_hash: Option<String>,
    /// `key-index-list/@ts`, if present.
    pub list_timestamp: Option<u64>,
}
fn attr_text<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    node.get_attr(name).and_then(|v| v.as_text())
}
fn attr_jid(node: &Node, name: &str) -> Option<Jid> {
    super::receipts::jid(node, name)
}
fn child<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    node.child_nodes().iter().find(|c| c.description == name)
}
fn parse_device_children(list: &Node) -> Result<Vec<DeviceEntry>> {
    let mut out = Vec::new();
    for device in list
        .child_nodes()
        .iter()
        .filter(|c| c.description == "device")
    {
        let jid = attr_jid(device, "jid").ok_or_else(|| invalid("device without jid"))?;
        let key_index = attr_text(device, "key-index")
            .map(|v| v.parse::<u32>().map_err(|_| invalid("invalid key-index")))
            .transpose()?
            .unwrap_or(0);
        out.push(DeviceEntry { jid, key_index });
        if out.len() > 1024 {
            return Err(invalid("too many devices"));
        }
    }
    Ok(out)
}
/// Recognize a device-list notification. `None` for anything else.
pub fn notification(node: &Node) -> Option<DevicesNotification> {
    if node.description != "notification"
        || attr_text(node, "type") != Some("devices")
        || node.get_attr("participant").is_some()
    {
        return None;
    }
    let owner = primary_of(&attr_jid(node, "from")?).ok()?;
    let (added, change) = match (child(node, "add"), child(node, "remove")) {
        (Some(add), _) => (true, add),
        (None, Some(remove)) => (false, remove),
        (None, None) => return None,
    };
    let devices = parse_device_children(change).ok()?;
    let list_timestamp = child(change, "key-index-list")
        .and_then(|c| attr_text(c, "ts"))
        .and_then(|v| v.parse().ok());
    Some(DevicesNotification {
        owner,
        added,
        devices,
        device_hash: attr_text(change, "device_hash").map(|v| v.chars().take(64).collect()),
        list_timestamp,
    })
}
/// Acknowledge a device-list notification after its durable capture
/// (`APK 37.71 LX/Cp2.A05 (0036..0074)` layout: id/to/class/type).
pub fn notification_ack(node: &Node) -> Result<Node> {
    let id = attr_text(node, "id")
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .ok_or_else(|| invalid("devices notification without id"))?;
    let to = attr_jid(node, "from").ok_or_else(|| invalid("devices notification without from"))?;
    Ok(Node::new("ack")
        .attr("to", to)
        .attr("id", id)
        .attr("class", "notification")
        .attr("type", "devices"))
}

/// Record that `user`'s device list should be fetched by the next query
/// (`LX/101.A02`). Own-account notifications are recorded by the caller as a
/// capture only; this refuses the account's own identity.
pub fn request_sync(
    store: &ProtocolStore,
    own: Option<&Jid>,
    user: &Jid,
    notified_hash: Option<&str>,
    reason: &str,
) -> Result<()> {
    let user = primary_of(user)?;
    if own.is_some_and(|own| own.to_non_ad() == user) {
        return Err(invalid(
            "own-account device sync uses account sync, not usync",
        ));
    }
    store.write(|tx| {
        tx.put_json(
            Table::Checkpoints,
            pending_key(&user).as_bytes(),
            1,
            &Pending {
                requested_at: Timestamp::now(),
                notified_hash: notified_hash.map(|v| v.chars().take(64).collect()),
                reason: reason.chars().take(64).collect(),
            },
        )
    })
}
/// Users waiting for a device query, ascending.
pub fn pending_users(store: &ProtocolStore) -> Result<Vec<Jid>> {
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 512)?;
            let done = rows.len() < 512;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                if let Ok(name) = std::str::from_utf8(&key)
                    && let Some(rest) = name.strip_prefix(PENDING_PREFIX)
                    && let Ok(jid) = Jid::parse(rest)
                {
                    out.push(jid);
                }
            }
            if done {
                out.sort();
                return Ok(out);
            }
        }
    })
}

/// A reserved device query awaiting its `iq` reply on this connection.
#[derive(Debug)]
pub struct DeviceQuery {
    iq_id: String,
    users: Vec<Jid>,
    /// After this instant the reply is no longer awaited this session.
    pub deadline: tokio::time::Instant,
}
/// Build one `usync devices` query for pending users. Returns `None` when
/// nothing is pending. The reservation is durable before the socket write.
pub fn begin_query(
    store: &ProtocolStore,
    context: SyncContext,
) -> Result<Option<(DeviceQuery, Node)>> {
    let users: Vec<Jid> = pending_users(store)?
        .into_iter()
        .take(MAX_USERS_PER_QUERY)
        .collect();
    if users.is_empty() {
        return Ok(None);
    }
    let iq_id = super::messaging::new_message_id()?;
    let sid = super::messaging::new_message_id()?;
    store.write(|tx| {
        tx.put_json(
            Table::Checkpoints,
            query_key(&iq_id).as_bytes(),
            1,
            &QueryRecord {
                users: users.clone(),
                status: "outcome_unknown".into(),
                code: None,
                ingested: 0,
                partial_errors: 0,
                requested_at: Timestamp::now(),
            },
        )
    })?;
    let list = users
        .iter()
        .map(|user| Node::new("user").attr("jid", user.clone()))
        .collect();
    let node = Node::new("iq")
        .attr("xmlns", "usync")
        .attr("id", iq_id.as_str())
        .attr("type", "get")
        .children(vec![
            Node::new("usync")
                .attr("sid", sid.as_str())
                .attr("index", "0")
                .attr("last", "true")
                .attr("mode", "query")
                .attr("context", context.as_str())
                .children(vec![
                    Node::new("query").children(vec![Node::new("devices").attr("version", "2")]),
                    Node::new("list").children(list),
                ]),
        ]);
    Ok(Some((
        DeviceQuery {
            iq_id,
            users,
            deadline: tokio::time::Instant::now() + std::time::Duration::from_secs(30),
        },
        node,
    )))
}

/// Outcome of ingesting one reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IngestSummary {
    /// Reply was `iq type=result`.
    pub accepted: bool,
    /// Users whose list was stored.
    pub ingested: u32,
    /// Users with a per-user error.
    pub partial_errors: u32,
}

fn pinned_primary_identity(
    tx: &crate::store::WriteTx<'_>,
    owner: &Jid,
) -> Result<Option<[u8; 32]>> {
    let key = format!("comparison-direct:{owner}");
    let Some(bytes): Option<Vec<u8>> = tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?
    else {
        return Ok(None);
    };
    let raw = match bytes.len() {
        33 if bytes[0] == 0x05 => &bytes[1..],
        32 => &bytes[..],
        _ => return Err(invalid("pinned identity has an unexpected length")),
    };
    Ok(Some(
        raw.try_into().map_err(|_| invalid("pinned identity"))?,
    ))
}

/// Parse one `user` element into a stored list, verifying companions when the
/// contact's primary identity is pinned. Returns `None` for a per-user error.
fn ingest_user(
    tx: &mut crate::store::WriteTx<'_>,
    user: &Node,
    notified_hash: Option<String>,
) -> Result<Option<DeviceList>> {
    let owner = primary_of(&attr_jid(user, "jid").ok_or_else(|| invalid("user without jid"))?)?;
    let devices = child(user, "devices").ok_or_else(|| invalid("user without devices"))?;
    if child(devices, "error").is_some() {
        return Ok(None);
    }
    let mut incoming = BTreeMap::new();
    let mut hosted_skipped = 0u32;
    if let Some(list) = child(devices, "device-list") {
        for device in list
            .child_nodes()
            .iter()
            .filter(|c| c.description == "device")
        {
            if attr_text(device, "is_hosted").is_some_and(|v| v.eq_ignore_ascii_case("true")) {
                hosted_skipped += 1;
                continue;
            }
            let id: u16 = attr_text(device, "id")
                .ok_or_else(|| invalid("device without id"))?
                .parse()
                .map_err(|_| invalid("invalid device id"))?;
            let key_index: u32 = attr_text(device, "key-index")
                .map(|v| v.parse().map_err(|_| invalid("invalid key-index")))
                .transpose()?
                .unwrap_or(0);
            incoming.insert(owner.with_device(id), key_index);
            if incoming.len() > 1024 {
                return Err(invalid("too many devices"));
            }
        }
    }
    incoming.entry(owner.clone()).or_insert(0);
    let saved: Option<DeviceList> =
        tx.get_json(Table::Checkpoints, list_key(&owner).as_bytes(), 1)?;
    let current: BTreeMap<Jid, u32> = saved
        .as_ref()
        .map(|s| {
            s.devices
                .iter()
                .map(|d| (d.jid.clone(), d.key_index))
                .collect()
        })
        .unwrap_or_default();
    let index_list = child(devices, "key-index-list");
    let list_timestamp = index_list
        .and_then(|c| attr_text(c, "ts"))
        .and_then(|v| v.parse::<u64>().ok());
    let expected_timestamp = index_list
        .and_then(|c| attr_text(c, "expected_ts"))
        .and_then(|v| v.parse::<u64>().ok());
    let signed = index_list.and_then(|c| match &c.content {
        NodeContent::Bytes(b) => Some(b.as_ref()),
        _ => None,
    });
    let mut unverified_reason = None;
    let mut raw_id = saved.as_ref().and_then(|s| s.raw_id);
    let mut companions_verified = false;
    let verified = match (signed, list_timestamp, pinned_primary_identity(tx, &owner)?) {
        (Some(bytes), Some(ts), Some(key)) => {
            let minimum = saved.as_ref().and_then(|s| s.list_timestamp).unwrap_or(0);
            match VerifiedIndexList::verify(bytes, &key, ts, minimum) {
                Ok(list) => Some(list),
                Err(_) => {
                    unverified_reason = Some("signed index list failed verification".into());
                    None
                }
            }
        }
        (None, _, _) | (_, None, _) => {
            unverified_reason = Some("no signed index list".into());
            None
        }
        (_, _, None) => {
            unverified_reason = Some("no pinned primary identity".into());
            None
        }
    };
    let retained: ReconciledDevices = match verified {
        Some(list) => {
            let expected = raw_id.unwrap_or_else(|| list.raw_id());
            match list.reconcile_devices(&owner, expected, &current, &incoming) {
                Ok(devices) => {
                    raw_id = Some(list.raw_id());
                    companions_verified = true;
                    devices
                }
                Err(_) => {
                    unverified_reason = Some("reconciliation refused (raw id or indexes)".into());
                    ReconciledDevices::primary_only(&owner)?
                }
            }
        }
        None => ReconciledDevices::primary_only(&owner)?,
    };
    let hash_matches = notified_hash.as_deref().map(|hash| {
        [DeviceHashVersion::V2, DeviceHashVersion::V3]
            .into_iter()
            .any(|version| retained.device_hash(&owner, version).ok().as_deref() == Some(hash))
    });
    let list = DeviceList {
        owner: owner.clone(),
        devices: retained
            .devices()
            .iter()
            .map(|(jid, index)| DeviceEntry {
                jid: jid.clone(),
                key_index: *index,
            })
            .collect(),
        raw_id,
        list_timestamp: if companions_verified {
            list_timestamp
        } else {
            saved.as_ref().and_then(|s| s.list_timestamp)
        },
        expected_timestamp,
        companions_verified,
        unverified_reason,
        notified_hash,
        hash_matches,
        hosted_skipped,
        fetched_at: Timestamp::now(),
    };
    tx.put_json(Table::Checkpoints, list_key(&owner).as_bytes(), 1, &list)?;
    tx.delete(Table::Checkpoints, pending_key(&owner).as_bytes())?;
    Ok(Some(list))
}

impl DeviceQuery {
    /// Only the exact `iq` reply for this query settles it.
    pub fn matches(&self, node: &Node) -> bool {
        node.description == "iq" && attr_text(node, "id") == Some(self.iq_id.as_str())
    }
    /// Apply the reply durably: each user's list is verified, reconciled and
    /// stored in one transaction; pending markers clear only for ingested users.
    pub fn ingest(&self, store: &ProtocolStore, node: &Node) -> Result<IngestSummary> {
        if !self.matches(node) {
            return Err(invalid("reply does not belong to this query"));
        }
        let accepted = attr_text(node, "type") == Some("result");
        let code = child(node, "error")
            .and_then(|c| attr_text(c, "code"))
            .map(|v| v.chars().take(8).collect::<String>());
        store.write(|tx| {
            let key = query_key(&self.iq_id);
            let mut record: QueryRecord = tx
                .get_json(Table::Checkpoints, key.as_bytes(), 1)?
                .ok_or_else(|| invalid("reserved query disappeared"))?;
            if record.status != "outcome_unknown" {
                return Err(invalid("query already settled"));
            }
            let mut summary = IngestSummary {
                accepted,
                ..Default::default()
            };
            if accepted {
                let users = child(node, "usync")
                    .and_then(|u| child(u, "list"))
                    .map(|l| l.child_nodes().to_vec())
                    .unwrap_or_default();
                for user in users.iter().filter(|u| u.description == "user") {
                    let Some(owner) = attr_jid(user, "jid").and_then(|j| primary_of(&j).ok())
                    else {
                        continue;
                    };
                    if !self.users.contains(&owner) {
                        continue;
                    }
                    let notified: Option<Pending> =
                        tx.get_json(Table::Checkpoints, pending_key(&owner).as_bytes(), 1)?;
                    match ingest_user(tx, user, notified.and_then(|p| p.notified_hash))? {
                        Some(_) => summary.ingested += 1,
                        None => summary.partial_errors += 1,
                    }
                }
            }
            record.status = if accepted {
                "server_acknowledged".into()
            } else {
                "refused".into()
            };
            record.code = code.clone();
            record.ingested = summary.ingested;
            record.partial_errors = summary.partial_errors;
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &record)?;
            Ok(summary)
        })
    }
}

/// Saved device list for a contact, without network access.
pub fn device_list(store: &ProtocolStore, user: &Jid) -> Result<Option<DeviceList>> {
    store.read(|tx| device_list_in(tx, user))
}

pub(crate) fn device_list_in(
    tx: &crate::store::ReadTx<'_>,
    user: &Jid,
) -> Result<Option<DeviceList>> {
    let owner = primary_of(user)?;
    tx.get_json(Table::Checkpoints, list_key(&owner).as_bytes(), 1)
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
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([2; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn contact() -> Jid {
        Jid::parse("15555550101@s.whatsapp.net").unwrap()
    }
    fn pin_identity(store: &ProtocolStore, owner: &Jid, key: &[u8; 32]) {
        let mut bytes = vec![0x05];
        bytes.extend_from_slice(key);
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{owner}").as_bytes(),
                    1,
                    &bytes,
                )
            })
            .unwrap();
    }
    fn reply(iq: &Node, users: Vec<Node>) -> Node {
        Node::new("iq")
            .attr("id", attr_text(iq, "id").unwrap())
            .attr("type", "result")
            .children(vec![Node::new("usync").attr("sid", "x").children(vec![
                Node::new("result"),
                Node::new("list").children(users),
            ])])
    }
    fn user_node(owner: &Jid, devices: &[(u16, u32, bool)], list: Option<(u64, Vec<u8>)>) -> Node {
        let mut device_nodes = Vec::new();
        for (id, index, hosted) in devices {
            let mut node = Node::new("device").attr("id", id.to_string());
            if *index > 0 {
                node = node.attr("key-index", index.to_string());
            }
            if *hosted {
                node = node.attr("is_hosted", "true");
            }
            device_nodes.push(node);
        }
        let mut children = vec![Node::new("device-list").children(device_nodes)];
        if let Some((ts, bytes)) = list {
            children.push(
                Node::new("key-index-list")
                    .attr("ts", ts.to_string())
                    .attr("expected_ts", (ts + 5).to_string())
                    .bytes(bytes),
            );
        }
        Node::new("user")
            .attr("jid", owner.clone())
            .children(vec![Node::new("devices").children(children)])
    }

    #[test]
    fn notifications_are_parsed_acknowledged_and_recorded_as_pending() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("pending.redb"));
        let node = Node::new("notification")
            .attr("type", "devices")
            .attr("id", "D1")
            .attr("from", contact())
            .children(vec![
                Node::new("add")
                    .attr("device_hash", "2:abcdef==")
                    .children(vec![
                        Node::new("device")
                            .attr("jid", contact().with_device(3))
                            .attr("key-index", "1"),
                        Node::new("key-index-list").attr("ts", "1700000000"),
                    ]),
            ]);
        let parsed = notification(&node).unwrap();
        assert_eq!(parsed.owner, contact());
        assert!(parsed.added);
        assert_eq!(parsed.devices.len(), 1);
        assert_eq!(parsed.devices[0].key_index, 1);
        assert_eq!(parsed.device_hash.as_deref(), Some("2:abcdef=="));
        assert_eq!(parsed.list_timestamp, Some(1_700_000_000));
        let ack = notification_ack(&node).unwrap();
        assert_eq!(ack.description, "ack");
        assert_eq!(attr_text(&ack, "type"), Some("devices"));
        assert_eq!(attr_text(&ack, "class"), Some("notification"));
        assert_eq!(attr_text(&ack, "id"), Some("D1"));
        for other in [
            Node::new("notification")
                .attr("type", "devices")
                .attr("from", contact()),
            Node::new("notification")
                .attr("type", "encrypt")
                .attr("from", contact())
                .children(vec![Node::new("add")]),
            Node::new("notification")
                .attr("type", "devices")
                .attr("from", contact())
                .attr("participant", "x")
                .children(vec![Node::new("remove")]),
        ] {
            assert!(notification(&other).is_none(), "{other:?}");
        }
        let own = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        assert!(request_sync(&store, Some(&own), &own, None, "test").is_err());
        assert!(
            request_sync(
                &store,
                Some(&own),
                &Jid::parse("g@g.us").unwrap(),
                None,
                "t"
            )
            .is_err()
        );
        request_sync(
            &store,
            Some(&own),
            &contact().with_device(3),
            Some("2:abcdef=="),
            "notification",
        )
        .unwrap();
        assert_eq!(pending_users(&store).unwrap(), vec![contact()]);
    }

    #[test]
    fn query_matches_traced_layout_and_reserves_before_write() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("query.redb"));
        assert!(
            begin_query(&store, SyncContext::Notification)
                .unwrap()
                .is_none()
        );
        let other = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        request_sync(&store, None, &contact(), None, "t").unwrap();
        request_sync(&store, None, &other, None, "t").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Notification)
            .unwrap()
            .unwrap();
        assert_eq!(iq.description, "iq");
        assert_eq!(attr_text(&iq, "xmlns"), Some("usync"));
        assert_eq!(attr_text(&iq, "type"), Some("get"));
        assert!(iq.get_attr("to").is_none(), "usync iq carries no to");
        let usync = child(&iq, "usync").unwrap();
        assert!(attr_text(usync, "sid").is_some_and(|s| !s.is_empty()));
        assert_eq!(attr_text(usync, "index"), Some("0"));
        assert_eq!(attr_text(usync, "last"), Some("true"));
        assert_eq!(attr_text(usync, "mode"), Some("query"));
        assert_eq!(attr_text(usync, "context"), Some("notification"));
        let names: Vec<&str> = usync
            .child_nodes()
            .iter()
            .map(|c| c.description.as_str())
            .collect();
        assert_eq!(names, ["query", "list"]);
        let devices = child(child(usync, "query").unwrap(), "devices").unwrap();
        assert_eq!(attr_text(devices, "version"), Some("2"));
        let users: Vec<Jid> = child(usync, "list")
            .unwrap()
            .child_nodes()
            .iter()
            .filter_map(|u| attr_jid(u, "jid"))
            .collect();
        assert_eq!(users, vec![contact(), other.clone()]);
        assert!(query.matches(&Node::new("iq").attr("id", attr_text(&iq, "id").unwrap())));
        assert!(!query.matches(&Node::new("iq").attr("id", "other")));
        // Pending users stay pending until their list is ingested.
        assert_eq!(pending_users(&store).unwrap().len(), 2);
        let record: QueryRecord = store
            .read(|tx| tx.get_json(Table::Checkpoints, query_key(&query.iq_id).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(record.status, "outcome_unknown");
        assert_eq!(record.users.len(), 2);
    }

    #[test]
    fn replies_verify_companions_with_pinned_identity_and_keep_primary_otherwise() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ingest.redb");
        let store = open(&path);
        let unpinned = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let errored = Jid::parse("15555550103@s.whatsapp.net").unwrap();
        for user in [&contact(), &unpinned, &errored] {
            request_sync(&store, None, user, Some("2:notahash"), "t").unwrap();
        }
        let private = [11u8; 32];
        let (signed, key) =
            crate::signal::adv::test_signed_list(7, 1_700_000_000, 2, &[1, 2], &private);
        pin_identity(&store, &contact(), &key);
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        // Device 3 has signed index 1, device 5 has unsigned index 9, device 7 is hosted.
        let pinned_user = user_node(
            &contact(),
            &[(0, 0, false), (3, 1, false), (5, 9, false), (7, 2, true)],
            Some((1_700_000_000, signed.clone())),
        );
        let unpinned_user = user_node(
            &unpinned,
            &[(0, 0, false), (4, 1, false)],
            Some((1_700_000_000, signed.clone())),
        );
        let errored_user = Node::new("user")
            .attr("jid", errored.clone())
            .children(vec![
                Node::new("devices").children(vec![Node::new("error").attr("code", "404")]),
            ]);
        let node = reply(&iq, vec![pinned_user, unpinned_user, errored_user]);
        assert!(
            query
                .ingest(&store, &Node::new("iq").attr("id", "other"))
                .is_err()
        );
        let summary = query.ingest(&store, &node).unwrap();
        assert_eq!(
            summary,
            IngestSummary {
                accepted: true,
                ingested: 2,
                partial_errors: 1
            }
        );
        assert!(query.ingest(&store, &node).is_err(), "settled once");
        let pinned = device_list(&store, &contact()).unwrap().unwrap();
        assert!(pinned.companions_verified);
        assert_eq!(pinned.raw_id, Some(7));
        assert_eq!(pinned.list_timestamp, Some(1_700_000_000));
        assert_eq!(pinned.expected_timestamp, Some(1_700_000_005));
        assert_eq!(pinned.hosted_skipped, 1);
        let devices: Vec<(u16, u32)> = pinned
            .devices
            .iter()
            .map(|d| (d.jid.device, d.key_index))
            .collect();
        assert_eq!(
            devices,
            vec![(0, 0), (3, 1)],
            "unsigned and hosted devices dropped"
        );
        assert_eq!(pinned.hash_matches, Some(false));
        let fallback = device_list(&store, &unpinned).unwrap().unwrap();
        assert!(!fallback.companions_verified);
        assert_eq!(
            fallback.unverified_reason.as_deref(),
            Some("no pinned primary identity")
        );
        assert_eq!(fallback.devices.len(), 1);
        assert_eq!(fallback.devices[0].jid, unpinned);
        assert!(device_list(&store, &errored).unwrap().is_none());
        let remaining: Vec<String> = pending_users(&store)
            .unwrap()
            .iter()
            .map(|j| j.to_string())
            .collect();
        assert_eq!(
            remaining,
            vec![errored.to_string()],
            "errored user stays pending"
        );
        // Notified hash agreement is reported when it matches the retained set.
        let hash = ReconciledDevices::primary_only(&unpinned)
            .unwrap()
            .device_hash(&unpinned, DeviceHashVersion::V2)
            .unwrap();
        request_sync(&store, None, &unpinned, Some(&hash), "t").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Message).unwrap().unwrap();
        let node = reply(&iq, vec![user_node(&unpinned, &[(0, 0, false)], None)]);
        query.ingest(&store, &node).unwrap();
        assert_eq!(
            device_list(&store, &unpinned)
                .unwrap()
                .unwrap()
                .hash_matches,
            Some(true)
        );

        // A raw-id change or stale timestamp keeps the saved list; a newer signed
        // list replaces it. Restart preserves everything.
        drop(store);
        let store = open(&path);
        let (changed_raw, _) =
            crate::signal::adv::test_signed_list(8, 1_700_000_100, 2, &[1, 2], &private);
        request_sync(&store, None, &contact(), None, "t").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        let node = reply(
            &iq,
            vec![user_node(
                &contact(),
                &[(0, 0, false), (3, 1, false)],
                Some((1_700_000_100, changed_raw)),
            )],
        );
        query.ingest(&store, &node).unwrap();
        let kept = device_list(&store, &contact()).unwrap().unwrap();
        assert!(!kept.companions_verified);
        assert_eq!(kept.raw_id, Some(7));
        assert_eq!(kept.list_timestamp, Some(1_700_000_000));
        let (stale, _) =
            crate::signal::adv::test_signed_list(7, 1_699_999_999, 2, &[1, 2], &private);
        request_sync(&store, None, &contact(), None, "t").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        query
            .ingest(
                &store,
                &reply(
                    &iq,
                    vec![user_node(
                        &contact(),
                        &[(0, 0, false), (3, 1, false)],
                        Some((1_699_999_999, stale)),
                    )],
                ),
            )
            .unwrap();
        assert_eq!(
            device_list(&store, &contact())
                .unwrap()
                .unwrap()
                .unverified_reason
                .as_deref(),
            Some("signed index list failed verification")
        );
        let (newer, _) =
            crate::signal::adv::test_signed_list(7, 1_700_000_200, 3, &[1, 2, 3], &private);
        request_sync(&store, None, &contact(), None, "t").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        query
            .ingest(
                &store,
                &reply(
                    &iq,
                    vec![user_node(
                        &contact(),
                        &[(0, 0, false), (3, 1, false), (9, 3, false)],
                        Some((1_700_000_200, newer)),
                    )],
                ),
            )
            .unwrap();
        let updated = device_list(&store, &contact()).unwrap().unwrap();
        assert!(updated.companions_verified);
        assert_eq!(updated.list_timestamp, Some(1_700_000_200));
        assert_eq!(updated.devices.len(), 3);
        // A refused query records its code and leaves users pending.
        request_sync(&store, None, &contact(), None, "t").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        let error = Node::new("iq")
            .attr("id", attr_text(&iq, "id").unwrap())
            .attr("type", "error")
            .children(vec![Node::new("error").attr("code", "503")]);
        assert_eq!(
            query.ingest(&store, &error).unwrap(),
            IngestSummary::default()
        );
        assert_eq!(
            pending_users(&store).unwrap(),
            vec![contact(), errored.clone()],
            "refused users stay pending alongside the earlier errored user"
        );
        let record: QueryRecord = store
            .read(|tx| tx.get_json(Table::Checkpoints, query_key(&query.iq_id).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            (record.status.as_str(), record.code.as_deref()),
            ("refused", Some("503"))
        );
    }
}
