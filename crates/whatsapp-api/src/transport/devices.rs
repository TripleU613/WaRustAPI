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
//! * Verification `LX/11T.A07` selects the local identity or a supplied ADV
//!   key when different. Only the already-pinned key branch is implemented
//!   here; no supplied key replaces identity. Contacts without a pinned
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
//! user. Failed signed candidates preserve the complete prior list and pending
//! revision and are reported as local verification refusals, independently of
//! server errors. Unsigned companion lists refuse; primary-only unsigned reads
//! remain supported. Supplied-key identity application and raw-ID replacement
//! are still unsupported. `sid` is a
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
const GENERATION_PREFIX: &str = "device-sync-generation:";

fn invalid(reason: impl Into<String>) -> Error {
    Error::invalid("devices", reason)
}
fn pending_key(user: &Jid) -> String {
    format!("{PENDING_PREFIX}{user}")
}
pub(crate) fn list_key(user: &Jid) -> String {
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

pub(super) fn same_device(a: &Jid, b: &Jid) -> bool {
    a.user == b.user && a.server == b.server && a.agent == b.agent && a.device == b.device
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
    /// Server `expected_ts` or a newer authenticated known-companion ADV expectation.
    /// Separate from the verified floor (APK 37.71 0bN.A05 / Lcr.A00).
    pub expected_timestamp: Option<u64>,
    /// Locally recorded removals from signed reconciliation; None is legacy unknown.
    /// No key cleanup is implied (APK 37.71 1Uf dead_devices lifecycle).
    #[serde(default)]
    pub pending_removed_devices: Option<Vec<Jid>>,
    /// Devices added by authenticated signed ADV, awaiting list confirmation.
    /// Local provenance, not a server field (APK 37.71 11T.A0D add-only path).
    #[serde(default)]
    pub adv_added_devices: Vec<Jid>,
    /// Account baseline has signed authority; additions have separate ADV provenance.
    pub companions_verified: bool,
    /// SHA-256 of the raw primary public key that verified this signed list.
    /// Private local binding, not an APK field or human verification. Historical
    /// rows stay unknown; never bind an old list to whichever pin exists now.
    #[serde(default)]
    pub verified_primary_key_fingerprint: Option<String>,
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
    // Local revision prevents an old reply erasing a newer notification. The
    // separate counter survives marker deletion, preventing revision reuse.
    #[serde(default)]
    generation: u64,
    requested_at: Timestamp,
    notified_hash: Option<String>,
    reason: String,
}
/// Fixed local verification failures; not WhatsApp server error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceVerificationFailure {
    /// Conflicting rows for one requested user; local ambiguity guard.
    ConflictingUserRows,
    /// Companion entries were advertised without signed index material.
    UnsignedCompanions,
    /// The supplied key-index-list has no byte body.
    InvalidSignedListShape,
    /// Signed material has no valid stanza timestamp.
    MissingTimestamp,
    /// No independently pinned primary key is available for this verifier.
    MissingPinnedIdentity,
    /// Stanza timestamp precedes the saved verified time floor.
    StaleTimestamp,
    /// Signature, protobuf or conservative verifier checks failed.
    InvalidSignedList,
    /// Local raw-ID replacement is unsupported; no reset is performed.
    RawIdMismatch,
    /// Device reconciliation failed its ownership/index guards.
    ReconciliationRefused,
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
    /// Requested users absent from the reply. None for historical records or
    /// an unsettled query; never infer completeness from old aggregate counts.
    #[serde(default)]
    pub omitted_users: Option<u32>,
    /// Local marker revisions captured at reservation, aligned with users.
    /// None for historical rows; these are not APK/server values or identities.
    #[serde(default)]
    pub generations: Option<Vec<u64>>,
    /// Successfully parsed users whose pending work changed before the reply.
    #[serde(default)]
    pub superseded_users: Option<u32>,
    /// Local rejected-user counts by fixed reason. None for old/unsettled rows.
    /// These are not server response codes and contain no addresses or keys.
    #[serde(default)]
    pub verification_failures: Option<BTreeMap<DeviceVerificationFailure, u32>>,
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
    store.write(|tx| request_sync_in(tx, &user, notified_hash, reason))
}
// APK 37.71 0bN.A05 preserves the verified list timestamp; 101.A03
// deduplicates scheduled users. Commit only after nested Signal authentication.
pub(super) fn request_adv_refresh_in(
    tx: &mut crate::store::WriteTx<'_>,
    list: &DeviceList,
    timestamp: u64,
) -> Result<()> {
    let pending: Option<Pending> =
        tx.get_json(Table::Checkpoints, pending_key(&list.owner).as_bytes(), 1)?;
    let raised = timestamp > list.expected_timestamp.unwrap_or(0);
    if raised {
        let mut updated = list.clone();
        updated.expected_timestamp = Some(timestamp);
        tx.put_json(
            Table::Checkpoints,
            list_key(&list.owner).as_bytes(),
            1,
            &updated,
        )?;
    }
    if raised || pending.is_none() {
        request_sync_in(tx, &list.owner, None, "companion_adv")?;
    }
    Ok(())
}

pub(crate) fn request_sync_in(
    tx: &mut crate::store::WriteTx<'_>,
    user: &Jid,
    notified_hash: Option<&str>,
    reason: &str,
) -> Result<()> {
    let previous: Option<Pending> =
        tx.get_json(Table::Checkpoints, pending_key(user).as_bytes(), 1)?;
    let generation_key = format!("{GENERATION_PREFIX}{user}");
    let prior: Option<u64> = tx.get_json(Table::Checkpoints, generation_key.as_bytes(), 1)?;
    let generation = prior
        .unwrap_or(0)
        .max(previous.as_ref().map_or(0, |p| p.generation))
        .checked_add(1)
        .ok_or_else(|| invalid("local device sync revision exhausted"))?;
    tx.put_json(
        Table::Checkpoints,
        generation_key.as_bytes(),
        1,
        &generation,
    )?;
    tx.put_json(
        Table::Checkpoints,
        pending_key(user).as_bytes(),
        1,
        &Pending {
            generation,
            requested_at: Timestamp::now(),
            // A plain refresh request must not erase an observed pending
            // notification hash. No hash is invented when neither exists.
            notified_hash: notified_hash
                .map(|v| v.chars().take(64).collect())
                .or_else(|| previous.and_then(|p| p.notified_hash)),
            reason: reason.chars().take(64).collect(),
        },
    )
}
// APK 37.71 0Z7.A00/A02 stores a version per exact user namespace.
// This key is not a signed device-list timestamp or a PN/LID alias merger.
// Native resend filtering also needs per-message send-time receipt versions
// (receipt-version.md); this current counter alone never authorizes a resend.
fn primary_device_version_key(peer: &Jid) -> String {
    format!("primary-device-version:{}", peer.to_non_ad())
}

fn decode_primary_device_version(bytes: Option<Vec<u8>>) -> Result<Option<i64>> {
    bytes
        .map(|bytes| {
            serde_json::from_slice(&bytes)
                .map_err(|_| Error::invalid("devices", "invalid saved primary device version"))
        })
        .transpose()
}

pub(super) fn primary_device_version_in(
    tx: &crate::store::ReadTx<'_>,
    peer: &Jid,
) -> Result<Option<i64>> {
    if peer.device != 0 || peer.agent != 0 || !matches!(peer.server, Server::Lid | Server::User) {
        return Ok(None);
    }
    decode_primary_device_version(tx.get(
        Table::Checkpoints,
        primary_device_version_key(peer).as_bytes(),
        1,
    )?)
}

pub(super) fn primary_device_version_write_in(
    tx: &crate::store::WriteTx<'_>,
    peer: &Jid,
) -> Result<Option<i64>> {
    decode_primary_device_version(tx.get(
        Table::Checkpoints,
        primary_device_version_key(peer).as_bytes(),
        1,
    )?)
}

/// Local follow-up to an already authenticated primary-key replacement. This
/// shares the pin/session/archive transaction; no old list is rewritten or
/// automatically bound to the replacement key.
pub(super) fn after_primary_identity_change_in(
    tx: &mut crate::store::WriteTx<'_>,
    peer: &Jid,
) -> Result<()> {
    if peer.device != 0 || peer.agent != 0 || !matches!(peer.server, Server::Lid | Server::User) {
        return Ok(());
    }
    // The primary-only guard above preserves agent/device scope. JidKind is
    // wire representation, so compare the validated primary without that kind.
    let peer = &peer.to_non_ad();
    let own = crate::session::AccountRecord::load_in_write(tx)?
        .and_then(|a| a.own_jid)
        .map(|j| j.to_non_ad());
    let own_lid: Option<Jid> =
        tx.get_json(Table::Checkpoints, super::group_send::OWN_LID_KEY, 1)?;
    if own.as_ref() == Some(peer) || own_lid.as_ref().map(Jid::to_non_ad).as_ref() == Some(peer) {
        return Ok(());
    }
    let key = primary_device_version_key(peer);
    let previous = primary_device_version_write_in(tx, peer)?;
    // APK 37.71 0Z7.A02 000a..001c: missing -> 1, otherwise add-long.
    // Java signed-long overflow wraps; this is not a chronological timestamp.
    let version = previous.map_or(1, |value| value.wrapping_add(1));
    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &version)?;
    request_sync_in(tx, peer, None, "authenticated-primary-change")
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
    generations: Vec<u64>,
    /// After this instant the reply is no longer awaited this session.
    pub deadline: tokio::time::Instant,
}
/// Connection-local eligibility: each unchanged pending revision is queried
/// once. Refused/uncertain users stay deferred for a later connection; fresh
/// notifications after successful reads may create new eligible work.
#[derive(Default)]
pub struct DeviceSyncCursor {
    attempted: BTreeMap<Jid, u64>,
    deferred: std::collections::BTreeSet<Jid>,
}
impl std::fmt::Debug for DeviceSyncCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceSyncCursor")
            .field("attempted_users", &self.attempted.len())
            .field("deferred_users", &self.deferred.len())
            .finish()
    }
}
impl DeviceSyncCursor {
    /// Reserve the next eligible local batch, excluding already attempted work.
    pub fn begin_next(
        &mut self,
        store: &ProtocolStore,
        context: SyncContext,
    ) -> Result<Option<(DeviceQuery, Node)>> {
        let pending = pending_users(store)?;
        let users = store.read(|tx| {
            let mut selected = Vec::new();
            for user in pending {
                let pending: Option<Pending> =
                    tx.get_json(Table::Checkpoints, pending_key(&user).as_bytes(), 1)?;
                if let Some(pending) = pending
                    && !self.deferred.contains(&user)
                    && self.attempted.get(&user) != Some(&pending.generation)
                {
                    selected.push(user);
                    if selected.len() == MAX_USERS_PER_QUERY {
                        break;
                    }
                }
            }
            Ok(selected)
        })?;
        let Some((query, node)) = begin_query_for_users(store, context, &users)? else {
            return Ok(None);
        };
        self.observe_query(&query);
        Ok(Some((query, node)))
    }
    pub(crate) fn observe_query(&mut self, query: &DeviceQuery) {
        for (user, revision) in query.users.iter().zip(&query.generations) {
            self.attempted.insert(user.clone(), *revision);
        }
    }
    /// Settle a reply, deferring refused, errored, locally rejected or omitted users. Successful
    /// users with newer pending revisions remain eligible in this connection.
    pub fn settle(
        &mut self,
        query: &DeviceQuery,
        store: &ProtocolStore,
        node: &Node,
    ) -> Result<IngestSummary> {
        let (summary, failed) = query.ingest_internal(store, node)?;
        self.deferred.extend(failed);
        Ok(summary)
    }
    /// A top-level refusal or uncertain read must not be repeated merely because
    /// another notification arrived during this connection.
    pub fn defer(&mut self, query: &DeviceQuery) {
        self.deferred.extend(query.users.iter().cloned());
    }
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
    reserve_query(store, context, users)
}

/// Reserve exactly this pending device-user batch, never unrelated pending
/// contacts. This is local batching over the same APK usync v2 layout; it does
/// not retry a refused query or assert complete own-account device inventory.
pub fn begin_query_for_users(
    store: &ProtocolStore,
    context: SyncContext,
    users: &[Jid],
) -> Result<Option<(DeviceQuery, Node)>> {
    if users.len() > MAX_USERS_PER_QUERY {
        return Err(invalid(
            "selected device query exceeds the local 64-user limit",
        ));
    }
    let users = users
        .iter()
        .map(primary_of)
        .collect::<Result<std::collections::BTreeSet<_>>>()?;
    reserve_query(store, context, users.into_iter().collect())
}

fn reserve_query(
    store: &ProtocolStore,
    context: SyncContext,
    users: Vec<Jid>,
) -> Result<Option<(DeviceQuery, Node)>> {
    store.write(|tx| reserve_query_in(tx, context, users))
}
pub(crate) fn reserve_query_in(
    tx: &mut crate::store::WriteTx<'_>,
    context: SyncContext,
    users: Vec<Jid>,
) -> Result<Option<(DeviceQuery, Node)>> {
    if users.is_empty() {
        return Ok(None);
    }
    let iq_id = super::messaging::new_message_id()?;
    let sid = super::messaging::new_message_id()?;
    let generations = {
        let mut generations = Vec::with_capacity(users.len());
        // Selection and reservation share a transaction: a stale batch cannot
        // silently invent pending work after another query settled its users.
        for user in &users {
            let pending: Option<Pending> =
                tx.get_json(Table::Checkpoints, pending_key(user).as_bytes(), 1)?;
            let pending = pending.ok_or_else(|| {
                invalid("selected device user is not pending; request synchronization first")
            })?;
            generations.push(pending.generation);
        }

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
                omitted_users: None,
                generations: Some(generations.clone()),
                superseded_users: None,
                verification_failures: None,
                requested_at: Timestamp::now(),
            },
        )?;
        generations
    };
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
            generations,
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
    /// Distinct users with a per-user error.
    pub partial_errors: u32,
    /// Requested users not represented by a recognized user row. A result
    /// stanza alone is not a complete answer.
    pub omitted_users: u32,
    /// Parsed lists that did not settle the currently pending revision.
    pub superseded_users: u32,
    /// Distinct users whose candidate was locally refused by verification.
    pub verification_errors: u32,
}

impl IngestSummary {
    /// Require every requested user to have an ingested list before preparing
    /// a send. Background synchronization may retain partial success instead.
    pub fn require_complete(&self) -> Result<()> {
        if !self.accepted {
            return Err(invalid("device query refused; send preparation stopped"));
        }
        if self.partial_errors > 0 {
            return Err(invalid(format!(
                "device query has {} per-user error(s); send preparation stopped",
                self.partial_errors
            )));
        }
        if self.verification_errors > 0 {
            return Err(invalid(format!(
                "device query has {} local verification refusal(s); inspect device-query --id; send preparation stopped",
                self.verification_errors
            )));
        }
        if self.omitted_users > 0 {
            return Err(invalid(format!(
                "device query omitted {} requested user(s); send preparation stopped",
                self.omitted_users
            )));
        }
        if self.superseded_users > 0 {
            return Err(invalid(format!(
                "device query superseded for {} user(s); send preparation stopped",
                self.superseded_users
            )));
        }
        Ok(())
    }
}

/// Inspect a saved query offline. Historical omitted-user information remains
/// None; this never resets or reserves a query.
pub fn query_record(store: &ProtocolStore, id: &str) -> Result<Option<QueryRecord>> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':'))
    {
        return Err(invalid("invalid device query id"));
    }
    store.read(|tx| tx.get_json(Table::Checkpoints, query_key(id).as_bytes(), 1))
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
    parse_primary_key(&bytes).map(Some)
}
fn parse_primary_key(bytes: &[u8]) -> Result<[u8; 32]> {
    let raw = match bytes.len() {
        33 if bytes[0] == 0x05 => &bytes[1..],
        32 => bytes,
        _ => return Err(invalid("pinned identity has an unexpected length")),
    };
    raw.try_into().map_err(|_| invalid("pinned identity"))
}

enum UserIngest {
    Stored,
    ServerError,
    VerificationRefused(DeviceVerificationFailure),
}

/// Apply one candidate only after verification/reconciliation. The exact APK
/// 37.71 Lck.A00 caller (099a..09f2, 0b6c..0cd8) does not apply stale or
/// failed signed candidates. Refusal must preserve the prior list and pending
/// work, not manufacture a successful primary-only replacement.
fn ingest_user(
    tx: &mut crate::store::WriteTx<'_>,
    user: &Node,
    notified_hash: Option<String>,
    clear_pending: bool,
) -> Result<UserIngest> {
    let owner = primary_of(&attr_jid(user, "jid").ok_or_else(|| invalid("user without jid"))?)?;
    let devices = child(user, "devices").ok_or_else(|| invalid("user without devices"))?;
    if child(devices, "error").is_some() {
        return Ok(UserIngest::ServerError);
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
    let server_expected_timestamp = index_list
        .and_then(|c| attr_text(c, "expected_ts"))
        .and_then(|v| v.parse::<u64>().ok());
    // Lcr.A00 0002..0068 retains expected-timestamp state when the new
    // normal baseline remains below it; reaching it clears the old expectation.
    let unresolved_expected = saved
        .as_ref()
        .and_then(|s| s.expected_timestamp)
        .filter(|expected| list_timestamp.is_none_or(|ts| ts < *expected));
    let expected_timestamp = match (server_expected_timestamp, unresolved_expected) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    let signed = index_list.and_then(|c| match &c.content {
        NodeContent::Bytes(b) => Some(b.as_ref()),
        _ => None,
    });
    let mut unverified_reason = None;
    let mut raw_id = saved.as_ref().and_then(|s| s.raw_id);
    let mut companions_verified = false;
    let mut verified_primary_key_fingerprint = None;
    let verified = if index_list.is_some() {
        let Some(bytes) = signed else {
            return Ok(UserIngest::VerificationRefused(
                DeviceVerificationFailure::InvalidSignedListShape,
            ));
        };
        let Some(ts) = list_timestamp else {
            return Ok(UserIngest::VerificationRefused(
                DeviceVerificationFailure::MissingTimestamp,
            ));
        };
        let minimum = saved.as_ref().and_then(|s| s.list_timestamp).unwrap_or(0);
        if ts < minimum {
            return Ok(UserIngest::VerificationRefused(
                DeviceVerificationFailure::StaleTimestamp,
            ));
        }
        let Some(key) = pinned_primary_identity(tx, &owner)? else {
            // The APK's supplied-key branch has additional caller application.
            // This strict pinned-key adapter does not establish that authority.
            return Ok(UserIngest::VerificationRefused(
                DeviceVerificationFailure::MissingPinnedIdentity,
            ));
        };
        match VerifiedIndexList::verify(bytes, &key, ts, minimum) {
            Ok(list) => {
                use sha2::Digest as _;
                // APK 37.71 11T.A07 verifies with this selected primary key.
                // Persist the exact verifier key binding in the same transaction.
                verified_primary_key_fingerprint = Some(hex::encode(sha2::Sha256::digest(key)));
                Some(list)
            }
            Err(_) => {
                return Ok(UserIngest::VerificationRefused(
                    DeviceVerificationFailure::InvalidSignedList,
                ));
            }
        }
    } else {
        // Lck.A00 0a50..0aa8 refuses unsigned companion entries. The separate
        // unsigned primary-only path remains supported; no companion is invented.
        if incoming.len() > 1 {
            return Ok(UserIngest::VerificationRefused(
                DeviceVerificationFailure::UnsignedCompanions,
            ));
        }
        unverified_reason = Some("no signed index list".into());
        None
    };
    let mut adv_added_devices = saved
        .as_ref()
        .map(|s| s.adv_added_devices.clone())
        .unwrap_or_default();
    let retained: ReconciledDevices = match verified {
        Some(list) => {
            if raw_id.is_some_and(|id| id != list.raw_id()) {
                // APK 11T.A04 has reset branches here. Local raw-ID replacement
                // remains unsupported; retaining state does not authorize reset.
                return Ok(UserIngest::VerificationRefused(
                    DeviceVerificationFailure::RawIdMismatch,
                ));
            }
            let expected = raw_id.unwrap_or_else(|| list.raw_id());
            match list.reconcile_devices(&owner, expected, &current, &incoming) {
                Ok(devices) => {
                    raw_id = Some(list.raw_id());
                    companions_verified = true;
                    adv_added_devices.retain(|jid| {
                        devices.devices().get(jid).is_some_and(|index| {
                            !list.contains_index(*index) || incoming.get(jid) != Some(index)
                        })
                    });
                    devices
                }
                Err(_) => {
                    return Ok(UserIngest::VerificationRefused(
                        DeviceVerificationFailure::ReconciliationRefused,
                    ));
                }
            }
        }
        None => {
            adv_added_devices.clear();
            ReconciledDevices::primary_only(&owner)?
        }
    };
    // 1Uf.A06 records removals; A02 drops returned-device markers before
    // cleanup. Our namespace-local registry keeps keys/history untouched.
    let mut pending_removed_devices = saved
        .as_ref()
        .and_then(|s| s.pending_removed_devices.clone());
    if companions_verified {
        let pending = pending_removed_devices.get_or_insert_with(Vec::new);
        for jid in current.keys().filter(|jid| jid.device != 0) {
            if !retained.devices().keys().any(|next| same_device(next, jid))
                && !pending.iter().any(|old| same_device(old, jid))
            {
                pending.push(jid.clone());
            }
        }
        pending.retain(|jid| !retained.devices().keys().any(|next| same_device(next, jid)));
        pending.sort();
    }
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
        adv_added_devices,
        pending_removed_devices,
        verified_primary_key_fingerprint,
        unverified_reason,
        notified_hash,
        hash_matches,
        hosted_skipped,
        fetched_at: Timestamp::now(),
    };
    tx.put_json(Table::Checkpoints, list_key(&owner).as_bytes(), 1, &list)?;
    if clear_pending && unresolved_expected.is_none() {
        tx.delete(Table::Checkpoints, pending_key(&owner).as_bytes())?;
    }
    Ok(UserIngest::Stored)
}

impl DeviceQuery {
    /// Only the exact `iq` reply for this query settles it.
    pub fn matches(&self, node: &Node) -> bool {
        node.description == "iq" && attr_text(node, "id") == Some(self.iq_id.as_str())
    }
    /// Apply the reply durably: each user's list is verified, reconciled and
    /// stored in one transaction; only ingested matching pending revisions clear.
    pub fn ingest(&self, store: &ProtocolStore, node: &Node) -> Result<IngestSummary> {
        self.ingest_internal(store, node)
            .map(|(summary, _)| summary)
    }
    fn ingest_internal(
        &self,
        store: &ProtocolStore,
        node: &Node,
    ) -> Result<(IngestSummary, Vec<Jid>)> {
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
            let mut represented = std::collections::BTreeSet::new();
            let mut ingested = std::collections::BTreeSet::new();
            let mut errored = std::collections::BTreeSet::new();
            let mut superseded = std::collections::BTreeSet::new();
            let mut rejected = BTreeMap::new();
            if accepted {
                let users = child(node, "usync")
                    .and_then(|u| child(u, "list"))
                    .map(|l| l.child_nodes().to_vec())
                    .unwrap_or_default();
                // Preflight duplicates before any mutation: a valid first row
                // must not clear pending work before a conflicting refused row.
                // Exact duplicates are harmless. This is local ambiguity policy,
                // not an established APK duplicate-row dispatch rule.
                let mut selected = BTreeMap::new();
                let mut conflicts = std::collections::BTreeSet::new();
                for user in users.iter().filter(|u| u.description == "user") {
                    let Some(owner) = attr_jid(user, "jid").and_then(|j| primary_of(&j).ok())
                    else {
                        continue;
                    };
                    if !self.users.contains(&owner) {
                        continue;
                    }
                    if let Some(previous) = selected.insert(owner.clone(), user)
                        && previous != user
                    {
                        conflicts.insert(owner);
                    }
                }
                for (owner, user) in selected {
                    represented.insert(owner.clone());
                    if conflicts.contains(&owner) {
                        rejected.insert(owner, DeviceVerificationFailure::ConflictingUserRows);
                        continue;
                    }
                    let notified: Option<Pending> =
                        tx.get_json(Table::Checkpoints, pending_key(&owner).as_bytes(), 1)?;
                    let index = self
                        .users
                        .iter()
                        .position(|u| u == &owner)
                        .expect("checked requested user");
                    let current = notified.as_ref().map(|p| p.generation);
                    let clear_pending = current == self.generations.get(index).copied();
                    match ingest_user(
                        tx,
                        user,
                        notified.and_then(|p| p.notified_hash),
                        clear_pending,
                    )? {
                        UserIngest::Stored => {
                            if !clear_pending {
                                superseded.insert(owner.clone());
                            }
                            ingested.insert(owner.clone());
                        }
                        UserIngest::ServerError => {
                            errored.insert(owner.clone());
                        }
                        UserIngest::VerificationRefused(reason) => {
                            rejected.insert(owner.clone(), reason);
                        }
                    }
                }
            }
            summary.ingested = ingested.len() as u32;
            summary.partial_errors = errored.len() as u32;
            summary.omitted_users = (self.users.len() - represented.len()) as u32;
            summary.superseded_users = superseded.len() as u32;
            summary.verification_errors = rejected.len() as u32;
            let mut reasons = BTreeMap::new();
            for reason in rejected.values() {
                *reasons.entry(*reason).or_insert(0) += 1;
            }
            record.verification_failures = Some(reasons);
            record.superseded_users = Some(summary.superseded_users);
            record.omitted_users = Some(summary.omitted_users);
            record.status = if accepted {
                "server_acknowledged".into()
            } else {
                "refused".into()
            };
            record.code = code.clone();
            record.ingested = summary.ingested;
            record.partial_errors = summary.partial_errors;
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &record)?;
            let failed = if accepted {
                // Omitted users have no settled list either; a new notification
                // must not retry that unresolved read in this connection.
                self.users
                    .iter()
                    .filter(|u| {
                        errored.contains(*u)
                            || rejected.contains_key(*u)
                            || !represented.contains(*u)
                    })
                    .cloned()
                    .collect()
            } else {
                self.users.clone()
            };
            Ok((summary, failed))
        })
    }
}

/// Current pin relationship of a historical saved signed-device list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceListAuthority {
    /// The saved row records the actual verifier key; false for historical rows.
    pub binding_known: bool,
    /// An independently saved primary identity exists in this namespace.
    pub primary_pin_present: bool,
    /// Null if binding or pin is missing; false for a changed key.
    pub primary_pin_matches: Option<bool>,
    /// Signed companions are verified under the currently saved primary key.
    /// This is membership authority, not human verification or companion-key trust.
    pub companions_current: bool,
}
/// Inspect current authority without rewriting cached history or changing pins.
pub fn device_list_authority(
    store: &ProtocolStore,
    user: &Jid,
) -> Result<Option<DeviceListAuthority>> {
    store.read(|tx| {
        let Some(list) = device_list_in(tx, user)? else {
            return Ok(None);
        };
        authority_in(tx, &list).map(Some)
    })
}
pub(crate) fn authority_in(
    tx: &crate::store::ReadTx<'_>,
    list: &DeviceList,
) -> Result<DeviceListAuthority> {
    use sha2::Digest as _;
    let pin: Option<Vec<u8>> = tx.get_json(
        Table::PeerIdentities,
        format!("comparison-direct:{}", list.owner).as_bytes(),
        1,
    )?;
    let fingerprint = pin
        .as_deref()
        .map(parse_primary_key)
        .transpose()?
        .map(|key| hex::encode(sha2::Sha256::digest(key)));
    let matches = list
        .verified_primary_key_fingerprint
        .as_ref()
        .zip(fingerprint.as_ref())
        .map(|(a, b)| a == b);
    Ok(DeviceListAuthority {
        binding_known: list.verified_primary_key_fingerprint.is_some(),
        primary_pin_present: pin.is_some(),
        primary_pin_matches: matches,
        companions_current: list.companions_verified && matches == Some(true),
    })
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
    fn adv_refresh_preserves_verified_floor_and_waits_for_expected_signed_list() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("refresh.redb"));
        let peer = contact();
        let (signed, key) = crate::signal::adv::test_signed_list(7, 8, 2, &[1, 2], &[11; 32]);
        pin_identity(&store, &peer, &key);
        request_sync(&store, None, &peer, None, "seed").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        // This fixture's optional server expected_ts is omitted to isolate the
        // authenticated ADV expectation from server-provided metadata.
        let signed_user = |ts, bytes| {
            let mut node = user_node(&peer, &[(0, 0, false), (3, 1, false)], Some((ts, bytes)));
            let NodeContent::Children(users) = &mut node.content else {
                panic!()
            };
            let NodeContent::Children(devices) = &mut users[0].content else {
                panic!()
            };
            devices[1].attributes.retain(|a| a.name != "expected_ts");
            node
        };
        query
            .ingest(&store, &reply(&iq, vec![signed_user(8, signed)]))
            .unwrap()
            .require_complete()
            .unwrap();
        let mut baseline = device_list(&store, &peer).unwrap().unwrap();
        // A separately authenticated ADV allocation newer than the response's
        // signed current_index must retain its provenance until confirmation.
        baseline.devices.push(DeviceEntry {
            jid: peer.with_device(4),
            key_index: 3,
        });
        baseline.adv_added_devices.push(peer.with_device(4));
        store
            .write(|tx| tx.put_json(Table::Checkpoints, list_key(&peer).as_bytes(), 1, &baseline))
            .unwrap();
        store
            .write(|tx| request_adv_refresh_in(tx, &baseline, 10))
            .unwrap();
        let pending: Pending = store
            .read(|tx| tx.get_json(Table::Checkpoints, pending_key(&peer).as_bytes(), 1))
            .unwrap()
            .unwrap();
        let updated = device_list(&store, &peer).unwrap().unwrap();
        assert_eq!(updated.list_timestamp, Some(8));
        assert_eq!(updated.expected_timestamp, Some(10));
        store
            .write(|tx| request_adv_refresh_in(tx, &updated, 9))
            .unwrap();
        let unchanged: Pending = store
            .read(|tx| tx.get_json(Table::Checkpoints, pending_key(&peer).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(pending.generation, unchanged.generation);
        for ts in [9, 10] {
            let (signed, _) = crate::signal::adv::test_signed_list(
                7,
                ts,
                if ts == 9 { 2 } else { 3 },
                if ts == 9 { &[1, 2] } else { &[1, 2, 3] },
                &[11; 32],
            );
            let mut cursor = DeviceSyncCursor::default();
            let (query, iq) = cursor
                .begin_next(&store, SyncContext::Interactive)
                .unwrap()
                .unwrap();
            let mut response_user = signed_user(ts, signed);
            if ts == 10 {
                let NodeContent::Children(users) = &mut response_user.content else {
                    panic!()
                };
                let NodeContent::Children(devices) = &mut users[0].content else {
                    panic!()
                };
                let NodeContent::Children(entries) = &mut devices[0].content else {
                    panic!()
                };
                entries.push(Node::new("device").attr("id", "4").attr("key-index", "3"));
            }
            cursor
                .settle(&query, &store, &reply(&iq, vec![response_user]))
                .unwrap()
                .require_complete()
                .unwrap();
            assert!(
                cursor
                    .begin_next(&store, SyncContext::Interactive)
                    .unwrap()
                    .is_none()
            );
            let list = device_list(&store, &peer).unwrap().unwrap();
            assert_eq!(list.list_timestamp, Some(ts));
            assert!(list.companions_verified);
            if ts == 9 {
                assert_eq!(list.expected_timestamp, Some(10));
                assert_eq!(list.adv_added_devices, vec![peer.with_device(4)]);
                assert_eq!(pending_users(&store).unwrap(), vec![peer.clone()]);
                let held: Pending = store
                    .read(|tx| tx.get_json(Table::Checkpoints, pending_key(&peer).as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(held.generation, pending.generation);
            } else {
                assert_eq!(list.expected_timestamp, None);
                assert!(list.adv_added_devices.is_empty());
                assert!(pending_users(&store).unwrap().is_empty());
            }
        }
    }

    #[test]
    fn signed_removal_registry_survives_reopen_and_clears_only_on_return() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("removals.redb");
        let store = open(&path);
        let peer = contact();
        let (_, key) = crate::signal::adv::test_signed_list(7, 1, 2, &[1, 2], &[11; 32]);
        pin_identity(&store, &peer, &key);
        let held: Vec<(Vec<u8>, Vec<u8>)> = store
            .read(|tx| tx.scan(Table::PeerIdentities, 1, 100))
            .unwrap();
        for ts in [1, 2, 3] {
            request_sync(&store, None, &peer, None, "fixture").unwrap();
            let (query, iq) = begin_query(&store, SyncContext::Interactive)
                .unwrap()
                .unwrap();
            let (signed, _) = crate::signal::adv::test_signed_list(7, ts, 2, &[1, 2], &[11; 32]);
            let entries: &[(u16, u32, bool)] = if ts == 2 {
                &[(0, 0, false)]
            } else {
                &[(0, 0, false), (3, 1, false)]
            };
            query
                .ingest(
                    &store,
                    &reply(&iq, vec![user_node(&peer, entries, Some((ts, signed)))]),
                )
                .unwrap()
                .require_complete()
                .unwrap();
            let expected = if ts == 2 {
                vec![peer.with_device(3)]
            } else {
                Vec::new()
            };
            assert_eq!(
                device_list(&store, &peer)
                    .unwrap()
                    .unwrap()
                    .pending_removed_devices,
                Some(expected)
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::PeerIdentities, 1, 100))
                    .unwrap(),
                held
            );
        }
        // Repeat removal, then reopen with the existing store key. Registry
        // persistence must not imply key erasure or a replacement identity.
        request_sync(&store, None, &peer, None, "fixture").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        let (signed, _) = crate::signal::adv::test_signed_list(7, 4, 2, &[1, 2], &[11; 32]);
        query
            .ingest(
                &store,
                &reply(
                    &iq,
                    vec![user_node(&peer, &[(0, 0, false)], Some((4, signed)))],
                ),
            )
            .unwrap()
            .require_complete()
            .unwrap();
        drop(store);
        let store = ProtocolStore::open(OpenOptions {
            path: path.clone(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([2; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(
            device_list(&store, &peer)
                .unwrap()
                .unwrap()
                .pending_removed_devices,
            Some(vec![peer.with_device(3)])
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::PeerIdentities, 1, 100))
                .unwrap(),
            held
        );
    }

    #[test]
    fn signed_authority_tracks_its_verifier_not_a_replacement_pin_or_legacy_row() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("authority.redb");
        let store = open(&path);
        let peer = contact();
        let (signed, key) =
            crate::signal::adv::test_signed_list(7, 1_700_000_000, 2, &[1, 2], &[11; 32]);
        pin_identity(&store, &peer, &key);
        request_sync(&store, None, &peer, None, "seed").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        query
            .ingest(
                &store,
                &reply(
                    &iq,
                    vec![user_node(
                        &peer,
                        &[(0, 0, false), (3, 1, false)],
                        Some((1_700_000_000, signed)),
                    )],
                ),
            )
            .unwrap()
            .require_complete()
            .unwrap();
        let baseline = device_list(&store, &peer).unwrap().unwrap();
        let authority = device_list_authority(&store, &peer).unwrap().unwrap();
        assert!(authority.binding_known && authority.companions_current);
        assert_eq!(authority.primary_pin_matches, Some(true));
        // Historical JSON has no independent verifier binding; do not stamp
        // today's pin on old metadata just because its verified flag was true.
        let mut old = serde_json::to_value(&baseline).unwrap();
        old.as_object_mut()
            .unwrap()
            .remove("verified_primary_key_fingerprint");
        old.as_object_mut().unwrap().remove("adv_added_devices");
        old.as_object_mut()
            .unwrap()
            .remove("pending_removed_devices");
        store
            .write(|tx| tx.put_json(Table::Checkpoints, list_key(&peer).as_bytes(), 1, &old))
            .unwrap();
        let legacy = device_list_authority(&store, &peer).unwrap().unwrap();
        assert!(!legacy.binding_known && !legacy.companions_current);
        assert!(legacy.primary_pin_present);
        assert_eq!(legacy.primary_pin_matches, None);
        assert!(
            device_list(&store, &peer)
                .unwrap()
                .unwrap()
                .verified_primary_key_fingerprint
                .is_none()
        );
        store
            .write(|tx| tx.put_json(Table::Checkpoints, list_key(&peer).as_bytes(), 1, &baseline))
            .unwrap();
        let (fresh, new_key) =
            crate::signal::adv::test_signed_list(7, 1_700_000_100, 2, &[1, 2], &[13; 32]);
        let selector = format!("comparison-direct:{peer}");
        let old_pin: Vec<u8> = store
            .read(|tx| tx.get_json(Table::PeerIdentities, selector.as_bytes(), 1))
            .unwrap()
            .unwrap();
        let new_pin = [vec![5u8], new_key.to_vec()].concat();
        let replace = |tx: &mut crate::store::WriteTx<'_>| {
            super::super::identity_conflicts::archive_replacement(
                tx,
                &peer,
                Some(&old_pin),
                None,
                &new_pin,
            )?;
            tx.put_json(Table::PeerIdentities, selector.as_bytes(), 1, &new_pin)
        };
        assert!(
            store
                .write::<()>(|tx| {
                    replace(tx)?;
                    Err(invalid("injected abort after authenticated replacement"))
                })
                .is_err()
        );
        assert!(pending_users(&store).unwrap().is_empty());
        assert!(
            device_list_authority(&store, &peer)
                .unwrap()
                .unwrap()
                .companions_current
        );
        store.write(replace).unwrap();
        let changed = device_list_authority(&store, &peer).unwrap().unwrap();
        assert_eq!(changed.primary_pin_matches, Some(false));
        assert!(!changed.companions_current);
        assert_eq!(device_list(&store, &peer).unwrap().unwrap(), baseline);
        assert_eq!(pending_users(&store).unwrap(), [contact()]);
        drop(store);
        let store = open(&path);
        assert_eq!(
            device_list_authority(&store, &peer).unwrap().unwrap(),
            changed
        );
        assert_eq!(device_list(&store, &peer).unwrap().unwrap(), baseline);
        assert_eq!(pending_users(&store).unwrap(), [contact()]);
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        query
            .ingest(
                &store,
                &reply(
                    &iq,
                    vec![user_node(
                        &peer,
                        &[(0, 0, false), (3, 1, false)],
                        Some((1_700_000_100, fresh)),
                    )],
                ),
            )
            .unwrap()
            .require_complete()
            .unwrap();
        assert!(
            device_list_authority(&store, &peer)
                .unwrap()
                .unwrap()
                .companions_current
        );
        assert_ne!(
            device_list(&store, &peer)
                .unwrap()
                .unwrap()
                .verified_primary_key_fingerprint,
            baseline.verified_primary_key_fingerprint
        );
        assert!(pending_users(&store).unwrap().is_empty());
    }

    #[test]
    fn primary_version_matches_native_signed_long_addition_and_transaction_rollback() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("version-overflow.redb"));
        let peer = contact();
        let key = primary_device_version_key(&peer);
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &i64::MAX))
            .unwrap();
        let mut encoded = peer.clone();
        encoded.kind = crate::model::jid::JidKind::AgentDevice;
        store
            .write(|tx| after_primary_identity_change_in(tx, &encoded))
            .unwrap();
        assert_eq!(
            store
                .read(|tx| primary_device_version_in(tx, &peer))
                .unwrap(),
            Some(i64::MIN)
        );
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let refused: Result<()> = store.write(|tx| {
            after_primary_identity_change_in(tx, &peer)?;
            Err(Error::invalid("test", "refuse commit"))
        });
        assert!(refused.is_err());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        // Corrupted stored type must refuse rather than restart the version.
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &"invalid"))
            .unwrap();
        assert!(
            store
                .write(|tx| after_primary_identity_change_in(tx, &peer))
                .is_err()
        );
    }

    #[test]
    fn primary_refresh_hook_does_not_queue_companion_or_own_lid() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("own-exclusions.redb"));
        let peer = contact();
        store
            .write(|tx| after_primary_identity_change_in(tx, &peer.with_device(3)))
            .unwrap();
        assert!(pending_users(&store).unwrap().is_empty());
        let mut agent = peer.clone();
        agent.agent = 1;
        store
            .write(|tx| after_primary_identity_change_in(tx, &agent))
            .unwrap();
        assert!(pending_users(&store).unwrap().is_empty());
        store
            .write(|tx| {
                crate::session::AccountRecord {
                    own_jid: Some(peer.clone()),
                    ..Default::default()
                }
                .save(tx)?;
                let mut encoded_primary = peer.clone();
                encoded_primary.kind = crate::model::jid::JidKind::AgentDevice;
                after_primary_identity_change_in(tx, &encoded_primary)
            })
            .unwrap();
        assert!(pending_users(&store).unwrap().is_empty());
        let own = Jid::parse("111111111111111@lid").unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    super::super::group_send::OWN_LID_KEY,
                    1,
                    &own,
                )?;
                for primary in [&peer, &own] {
                    for kind in [
                        crate::model::jid::JidKind::Plain,
                        crate::model::jid::JidKind::AgentDevice,
                        crate::model::jid::JidKind::Interop,
                        crate::model::jid::JidKind::Facebook,
                    ] {
                        let mut encoded = primary.clone();
                        encoded.kind = kind;
                        after_primary_identity_change_in(tx, &encoded)?;
                    }
                }
                Ok(())
            })
            .unwrap();
        assert!(pending_users(&store).unwrap().is_empty());
        assert!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
                .iter()
                .all(|(key, _)| !key.starts_with(b"primary-device-version:"))
        );
    }

    #[test]
    fn rejected_candidates_preserve_verified_lists_pins_sessions_and_pending_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("retention.redb");
        let store = open(&path);
        let peer = contact();
        let private = [11u8; 32];
        let (signed, key) =
            crate::signal::adv::test_signed_list(7, 1_700_000_000, 2, &[1, 2], &private);
        pin_identity(&store, &peer, &key);
        request_sync(&store, None, &peer, None, "seed").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        query
            .ingest(
                &store,
                &reply(
                    &iq,
                    vec![user_node(
                        &peer,
                        &[(0, 0, false), (3, 1, false)],
                        Some((1_700_000_000, signed.clone())),
                    )],
                ),
            )
            .unwrap()
            .require_complete()
            .unwrap();
        let baseline = device_list(&store, &peer).unwrap().unwrap();
        assert!(baseline.companions_verified);
        let selector = format!("comparison-direct:{peer}");
        store
            .write(|tx| {
                tx.put_json(
                    Table::SignalSessions,
                    selector.as_bytes(),
                    1,
                    &vec![19u8, 27],
                )
            })
            .unwrap();
        let protected = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    Ok((
                        tx.get_json::<Vec<u8>>(Table::PeerIdentities, selector.as_bytes(), 1)?,
                        tx.get_json::<Vec<u8>>(Table::SignalSessions, selector.as_bytes(), 1)?,
                    ))
                })
                .unwrap()
        };
        let protected_before = protected(&store);
        let (stale, _) =
            crate::signal::adv::test_signed_list(7, 1_699_999_999, 2, &[1, 2], &private);
        let (raw, _) = crate::signal::adv::test_signed_list(8, 1_700_000_100, 2, &[1, 2], &private);
        let mut tampered = signed.clone();
        tampered[3] ^= 1;
        let (foreign_signed, _) =
            crate::signal::adv::test_signed_list(7, 1_700_000_100, 2, &[1, 2], &[13; 32]);
        let with_index = |index: Node| {
            Node::new("user").attr("jid", peer.clone()).children(vec![
                Node::new("devices").children(vec![
                    Node::new("device-list").children(vec![
                        Node::new("device").attr("id", "0"),
                        Node::new("device").attr("id", "3").attr("key-index", "1"),
                    ]),
                    index,
                ]),
            ])
        };
        let cases = vec![
            (
                DeviceVerificationFailure::UnsignedCompanions,
                user_node(&peer, &[(0, 0, false), (3, 1, false)], None),
            ),
            (
                DeviceVerificationFailure::InvalidSignedListShape,
                with_index(Node::new("key-index-list").attr("ts", "1700000000")),
            ),
            (
                DeviceVerificationFailure::MissingTimestamp,
                with_index(Node::new("key-index-list").bytes(signed.clone())),
            ),
            (
                DeviceVerificationFailure::StaleTimestamp,
                user_node(
                    &peer,
                    &[(0, 0, false), (3, 1, false)],
                    Some((1_699_999_999, stale)),
                ),
            ),
            (
                DeviceVerificationFailure::InvalidSignedList,
                user_node(
                    &peer,
                    &[(0, 0, false), (3, 1, false)],
                    Some((1_700_000_000, vec![0xff])),
                ),
            ),
            (
                DeviceVerificationFailure::InvalidSignedList,
                user_node(
                    &peer,
                    &[(0, 0, false), (3, 1, false)],
                    Some((1_700_000_000, tampered)),
                ),
            ),
            (
                DeviceVerificationFailure::InvalidSignedList,
                user_node(
                    &peer,
                    &[(0, 0, false), (3, 1, false)],
                    Some((1_700_000_100, foreign_signed)),
                ),
            ),
            (
                DeviceVerificationFailure::RawIdMismatch,
                user_node(
                    &peer,
                    &[(0, 0, false), (3, 1, false)],
                    Some((1_700_000_100, raw)),
                ),
            ),
            (
                DeviceVerificationFailure::ReconciliationRefused,
                user_node(
                    &peer,
                    &[(0, 0, false), (3, 1, false), (5, 1, false)],
                    Some((1_700_000_000, signed)),
                ),
            ),
        ];
        let mut ids = Vec::new();
        for (reason, candidate) in cases {
            request_sync(&store, None, &peer, Some("2:retained"), "test").unwrap();
            let mut cursor = DeviceSyncCursor::default();
            let (query, iq) = cursor
                .begin_next(&store, SyncContext::Interactive)
                .unwrap()
                .unwrap();
            let summary = cursor
                .settle(
                    &query,
                    &store,
                    &reply(&iq, vec![candidate.clone(), candidate]),
                )
                .unwrap();
            assert_eq!(
                summary.verification_errors, 1,
                "duplicate rows count one user"
            );
            assert_eq!(
                (
                    summary.ingested,
                    summary.partial_errors,
                    summary.omitted_users
                ),
                (0, 0, 0)
            );
            assert!(
                summary.accepted,
                "IQ result is separate from local verification"
            );
            assert!(
                summary
                    .require_complete()
                    .unwrap_err()
                    .to_string()
                    .contains("verification")
            );
            let record = query_record(&store, &query.iq_id).unwrap().unwrap();
            assert_eq!(
                record.verification_failures,
                Some(BTreeMap::from([(reason, 1)]))
            );
            assert_eq!(record.status, "server_acknowledged");
            assert_eq!(
                record.code, None,
                "no server code invented for local refusal"
            );
            assert_eq!(device_list(&store, &peer).unwrap().unwrap(), baseline);
            assert_eq!(protected(&store), protected_before);
            assert_eq!(
                pending_users(&store).unwrap().as_slice(),
                std::slice::from_ref(&peer)
            );
            request_sync(&store, None, &peer, None, "newer-notification").unwrap();
            assert!(
                cursor
                    .begin_next(&store, SyncContext::Notification)
                    .unwrap()
                    .is_none()
            );
            ids.push((query.iq_id, reason));
        }
        drop(store);
        let store = open(&path);
        assert_eq!(device_list(&store, &peer).unwrap().unwrap(), baseline);
        assert_eq!(protected(&store), protected_before);
        assert_eq!(
            pending_users(&store).unwrap().as_slice(),
            std::slice::from_ref(&peer)
        );
        for (id, reason) in ids {
            assert_eq!(
                query_record(&store, &id)
                    .unwrap()
                    .unwrap()
                    .verification_failures,
                Some(BTreeMap::from([(reason, 1)]))
            );
        }
        // A distinct valid later read settles retained work normally, without
        // reviving old reservations or resetting the prior identity/session.
        let (fresh, _) =
            crate::signal::adv::test_signed_list(7, 1_700_000_200, 3, &[1, 2, 3], &private);
        let (query, iq) = DeviceSyncCursor::default()
            .begin_next(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        query
            .ingest(
                &store,
                &reply(
                    &iq,
                    vec![user_node(
                        &peer,
                        &[(0, 0, false), (3, 1, false), (9, 3, false)],
                        Some((1_700_000_200, fresh)),
                    )],
                ),
            )
            .unwrap()
            .require_complete()
            .unwrap();
        assert!(pending_users(&store).unwrap().is_empty());
        assert_eq!(
            device_list(&store, &peer).unwrap().unwrap().devices.len(),
            3
        );
        assert_eq!(protected(&store), protected_before);
    }

    #[test]
    fn conflicting_duplicate_rows_do_not_apply_the_first_candidate_in_either_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("duplicate-conflict.redb");
        let store = open(&path);
        let peer = contact();
        request_sync(&store, None, &peer, None, "seed").unwrap();
        let (query, iq) = begin_query(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        let good = user_node(&peer, &[(0, 0, false)], None);
        query
            .ingest(&store, &reply(&iq, vec![good.clone()]))
            .unwrap()
            .require_complete()
            .unwrap();
        let baseline = device_list(&store, &peer).unwrap().unwrap();
        let bad = user_node(&peer, &[(0, 0, false), (3, 1, false)], None);
        for rows in [vec![good.clone(), bad.clone()], vec![bad.clone(), good]] {
            request_sync(&store, None, &peer, Some("2:pending"), "test").unwrap();
            let (query, iq) = begin_query(&store, SyncContext::Interactive)
                .unwrap()
                .unwrap();
            let summary = query.ingest(&store, &reply(&iq, rows)).unwrap();
            assert_eq!(
                (
                    summary.ingested,
                    summary.verification_errors,
                    summary.partial_errors,
                    summary.omitted_users
                ),
                (0, 1, 0, 0)
            );
            assert!(summary.require_complete().is_err());
            assert_eq!(device_list(&store, &peer).unwrap().unwrap(), baseline);
            assert_eq!(pending_users(&store).unwrap(), [contact()]);
            assert_eq!(
                query_record(&store, &query.iq_id)
                    .unwrap()
                    .unwrap()
                    .verification_failures,
                Some(BTreeMap::from([(
                    DeviceVerificationFailure::ConflictingUserRows,
                    1
                )]))
            );
        }
        drop(store);
        let store = open(&path);
        assert_eq!(device_list(&store, &peer).unwrap().unwrap(), baseline);
        assert_eq!(pending_users(&store).unwrap(), [peer]);
    }

    #[test]
    fn missing_pin_refusal_is_distinct_and_does_not_block_other_successful_users() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("missing-pin.redb"));
        let missing = contact();
        let good = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        for user in [&missing, &good] {
            request_sync(&store, None, user, None, "test").unwrap();
        }
        let mut cursor = DeviceSyncCursor::default();
        let (query, iq) = cursor
            .begin_next(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        let (signed, _) =
            crate::signal::adv::test_signed_list(7, 1_700_000_000, 2, &[1, 2], &[11; 32]);
        let summary = cursor
            .settle(
                &query,
                &store,
                &reply(
                    &iq,
                    vec![
                        user_node(
                            &missing,
                            &[(0, 0, false), (3, 1, false)],
                            Some((1_700_000_000, signed)),
                        ),
                        user_node(&good, &[(0, 0, false)], None),
                    ],
                ),
            )
            .unwrap();
        assert_eq!(
            (
                summary.ingested,
                summary.verification_errors,
                summary.partial_errors
            ),
            (1, 1, 0)
        );
        assert!(summary.require_complete().is_err());
        assert!(device_list(&store, &missing).unwrap().is_none());
        assert!(device_list(&store, &good).unwrap().is_some());
        let failures = query_record(&store, &query.iq_id)
            .unwrap()
            .unwrap()
            .verification_failures
            .unwrap();
        assert_eq!(
            failures,
            BTreeMap::from([(DeviceVerificationFailure::MissingPinnedIdentity, 1)])
        );
        // Hold the locally refused user, while another user's new work advances.
        for user in [&missing, &good] {
            request_sync(&store, None, user, None, "notification").unwrap();
        }
        let (next, _) = cursor
            .begin_next(&store, SyncContext::Notification)
            .unwrap()
            .unwrap();
        assert_eq!(next.users.as_slice(), std::slice::from_ref(&good));
    }

    #[test]
    fn newer_pending_revisions_survive_old_replies_and_deleted_markers_do_not_reuse_revisions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("revisions.redb");
        let store = open(&path);
        request_sync(&store, None, &contact(), Some("2:old"), "notification").unwrap();
        let mut cursor = DeviceSyncCursor::default();
        let (old, iq) = cursor
            .begin_next(&store, SyncContext::Notification)
            .unwrap()
            .unwrap();
        assert_eq!(old.generations, [1]);
        request_sync(&store, None, &contact(), Some("2:new"), "notification").unwrap();
        let summary = cursor
            .settle(
                &old,
                &store,
                &reply(&iq, vec![user_node(&contact(), &[(0, 0, false)], None)]),
            )
            .unwrap();
        assert_eq!(summary.superseded_users, 1);
        assert!(
            summary
                .require_complete()
                .unwrap_err()
                .to_string()
                .contains("superseded")
        );
        assert_eq!(pending_users(&store).unwrap(), [contact()]);
        assert_eq!(
            query_record(&store, &old.iq_id)
                .unwrap()
                .unwrap()
                .superseded_users,
            Some(1)
        );
        let (new, iq) = cursor
            .begin_next(&store, SyncContext::Notification)
            .unwrap()
            .unwrap();
        assert_eq!(new.generations, [2]);
        cursor
            .settle(
                &new,
                &store,
                &reply(&iq, vec![user_node(&contact(), &[(0, 0, false)], None)]),
            )
            .unwrap()
            .require_complete()
            .unwrap();
        assert!(pending_users(&store).unwrap().is_empty());
        request_sync(&store, None, &contact(), Some("2:latest"), "notification").unwrap();
        request_sync(&store, None, &contact(), None, "cli").unwrap();
        let pending: Pending = store
            .read(|tx| tx.get_json(Table::Checkpoints, pending_key(&contact()).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(pending.generation, 4);
        assert_eq!(pending.notified_hash.as_deref(), Some("2:latest"));
        drop(store);
        let store = open(&path);
        let (again, _) = cursor
            .begin_next(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        assert_eq!(again.generations, [4]);
        assert_eq!(
            query_record(&store, &again.iq_id)
                .unwrap()
                .unwrap()
                .generations,
            Some(vec![4])
        );
    }

    #[test]
    fn cursor_defers_actual_failed_users_but_keeps_successful_newer_work_eligible() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("cursor.redb"));
        let failed = contact();
        let success = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        for user in [&failed, &success] {
            request_sync(&store, None, user, None, "test").unwrap();
        }
        let mut cursor = DeviceSyncCursor::default();
        let (query, iq) = cursor
            .begin_next(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        request_sync(&store, None, &success, Some("2:new"), "notification").unwrap();
        let bad = Node::new("user").attr("jid", failed.clone()).children(vec![
            Node::new("devices").children(vec![Node::new("error").attr("code", "404")]),
        ]);
        let summary = cursor
            .settle(
                &query,
                &store,
                &reply(&iq, vec![bad, user_node(&success, &[(0, 0, false)], None)]),
            )
            .unwrap();
        assert_eq!(summary.partial_errors, 1);
        assert_eq!(summary.superseded_users, 1);
        request_sync(&store, None, &failed, Some("2:later"), "notification").unwrap();
        let (next, iq) = cursor
            .begin_next(&store, SyncContext::Notification)
            .unwrap()
            .unwrap();
        assert_eq!(next.users.as_slice(), std::slice::from_ref(&success));
        cursor
            .settle(
                &next,
                &store,
                &reply(&iq, vec![user_node(&success, &[(0, 0, false)], None)]),
            )
            .unwrap();
        assert!(
            cursor
                .begin_next(&store, SyncContext::Notification)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            pending_users(&store).unwrap().as_slice(),
            std::slice::from_ref(&failed)
        );
        let mut later = DeviceSyncCursor::default();
        let (query, _) = later
            .begin_next(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        assert_eq!(query.users, [failed]);
        // The uncertain read is held for this connection even after fresh input.
        later.defer(&query);
        request_sync(&store, None, &contact(), None, "test").unwrap();
        assert!(
            later
                .begin_next(&store, SyncContext::Interactive)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            query_record(&store, &query.iq_id).unwrap().unwrap().status,
            "outcome_unknown"
        );
    }

    #[test]
    fn cursor_holds_omitted_users_despite_newer_notifications() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("omitted.redb"));
        request_sync(&store, None, &contact(), None, "test").unwrap();
        let mut cursor = DeviceSyncCursor::default();
        let (query, iq) = cursor
            .begin_next(&store, SyncContext::Interactive)
            .unwrap()
            .unwrap();
        let summary = cursor.settle(&query, &store, &reply(&iq, vec![])).unwrap();
        assert_eq!(summary.omitted_users, 1);
        assert!(summary.require_complete().is_err());
        request_sync(&store, None, &contact(), Some("2:later"), "notification").unwrap();
        assert!(
            cursor
                .begin_next(&store, SyncContext::Notification)
                .unwrap()
                .is_none()
        );
        assert_eq!(pending_users(&store).unwrap(), [contact()]);
        assert!(
            DeviceSyncCursor::default()
                .begin_next(&store, SyncContext::Interactive)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn scoped_queries_process_three_batches_without_draining_unrelated_pending_users() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("batches.redb"));
        let users: Vec<Jid> = (0..130)
            .map(|i| Jid::parse(&format!("1555555{:04}@s.whatsapp.net", i + 100)).unwrap())
            .collect();
        let unrelated = Jid::parse("15550000000@s.whatsapp.net").unwrap();
        for user in users.iter().chain(std::iter::once(&unrelated)) {
            request_sync(&store, None, user, None, "test").unwrap();
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut batch_sizes = Vec::new();
        for batch in users.chunks(MAX_USERS_PER_QUERY) {
            let (query, iq) = begin_query_for_users(&store, SyncContext::Message, batch)
                .unwrap()
                .unwrap();
            assert_eq!(query.users, batch);
            assert!(ids.insert(query.iq_id.clone()));
            let wire_users = child(child(&iq, "usync").unwrap(), "list")
                .unwrap()
                .child_nodes();
            assert_eq!(wire_users.len(), batch.len());
            assert!(!query.users.contains(&unrelated));
            assert_eq!(
                attr_text(child(&iq, "usync").unwrap(), "context"),
                Some("message")
            );
            let stored = query_record(&store, &query.iq_id).unwrap().unwrap();
            assert_eq!(stored.status, "outcome_unknown");
            assert_eq!(stored.omitted_users, None);
            let response = reply(
                &iq,
                batch
                    .iter()
                    .map(|u| user_node(u, &[(0, 0, false)], None))
                    .collect(),
            );
            let summary = query.ingest(&store, &response).unwrap();
            summary.require_complete().unwrap();
            assert_eq!(summary.ingested as usize, batch.len());
            assert_eq!(summary.omitted_users, 0);
            assert!(query.ingest(&store, &response).is_err());
            batch_sizes.push(batch.len());
        }
        assert_eq!(batch_sizes, [64, 64, 2]);
        assert_eq!(pending_users(&store).unwrap(), [unrelated]);
        assert!(
            users
                .iter()
                .all(|user| device_list(&store, user).unwrap().is_some())
        );
    }

    #[test]
    fn scoped_selection_validates_atomically_and_normalizes_only_user_addresses() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("scope.redb"));
        request_sync(&store, None, &contact(), None, "test").unwrap();
        let before = store
            .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
            .unwrap();
        let absent = Jid::parse("15555550999@s.whatsapp.net").unwrap();
        for batch in [
            vec![contact(), absent],
            vec![contact(); 65],
            vec![Jid::parse("120363000000000001@g.us").unwrap()],
        ] {
            assert!(begin_query_for_users(&store, SyncContext::Message, &batch).is_err());
            assert_eq!(
                store
                    .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 512))
                    .unwrap(),
                before
            );
        }
        assert!(
            begin_query_for_users(&store, SyncContext::Message, &[])
                .unwrap()
                .is_none()
        );
        let (query, _) = begin_query_for_users(
            &store,
            SyncContext::Message,
            &[contact(), contact().with_device(3)],
        )
        .unwrap()
        .unwrap();
        assert_eq!(query.users, [contact()]);
        assert_eq!(
            query_record(&store, &query.iq_id).unwrap().unwrap().users,
            [contact()]
        );
    }

    #[test]
    fn incomplete_or_refused_queries_cannot_use_cached_lists_to_pass_send_preparation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("incomplete.redb");
        let store = open(&path);
        let other = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let targets = vec![contact(), other.clone()];
        for u in &targets {
            request_sync(&store, None, u, None, "test").unwrap();
        }
        let (initial, iq) = begin_query_for_users(&store, SyncContext::Message, &targets)
            .unwrap()
            .unwrap();
        initial
            .ingest(
                &store,
                &reply(
                    &iq,
                    targets
                        .iter()
                        .map(|u| user_node(u, &[(0, 0, false)], None))
                        .collect(),
                ),
            )
            .unwrap()
            .require_complete()
            .unwrap();
        let saved: Vec<_> = targets
            .iter()
            .map(|u| device_list(&store, u).unwrap().unwrap())
            .collect();
        for u in &targets {
            request_sync(&store, None, u, None, "test").unwrap();
        }
        let (refused, iq) = begin_query_for_users(&store, SyncContext::Message, &targets)
            .unwrap()
            .unwrap();
        let summary = refused
            .ingest(
                &store,
                &Node::new("iq")
                    .attr("id", attr_text(&iq, "id").unwrap())
                    .attr("type", "error")
                    .children(vec![Node::new("error").attr("code", "503")]),
            )
            .unwrap();
        assert!(!summary.accepted);
        assert!(
            summary
                .require_complete()
                .unwrap_err()
                .to_string()
                .contains("refused")
        );
        assert_eq!(summary.omitted_users, 2);
        assert_eq!(
            query_record(&store, &refused.iq_id)
                .unwrap()
                .unwrap()
                .status,
            "refused"
        );
        assert_eq!(pending_users(&store).unwrap(), targets);
        assert_eq!(
            targets
                .iter()
                .map(|u| device_list(&store, u).unwrap().unwrap())
                .collect::<Vec<_>>(),
            saved
        );
        // A separate explicit read remains possible; no code resets the refused
        // reservation. Duplicate rows cannot hide the omitted second user.
        let (incomplete, iq) = begin_query_for_users(&store, SyncContext::Message, &targets)
            .unwrap()
            .unwrap();
        let duplicated = user_node(&contact(), &[(0, 0, false)], None);
        let summary = incomplete
            .ingest(&store, &reply(&iq, vec![duplicated.clone(), duplicated]))
            .unwrap();
        assert_eq!(summary.ingested, 1);
        assert_eq!(summary.omitted_users, 1);
        assert!(
            summary
                .require_complete()
                .unwrap_err()
                .to_string()
                .contains("omitted 1")
        );
        assert_eq!(
            pending_users(&store).unwrap().as_slice(),
            std::slice::from_ref(&other)
        );
        let id = incomplete.iq_id.clone();
        drop(store);
        let reopened = open(&path);
        let record = query_record(&reopened, &id).unwrap().unwrap();
        assert_eq!(record.status, "server_acknowledged");
        assert_eq!(record.omitted_users, Some(1));
        assert_eq!(
            pending_users(&reopened).unwrap().as_slice(),
            std::slice::from_ref(&other)
        );
        let (partial, iq) = begin_query_for_users(
            &reopened,
            SyncContext::Message,
            std::slice::from_ref(&other),
        )
        .unwrap()
        .unwrap();
        let error = Node::new("user").attr("jid", other.clone()).children(vec![
            Node::new("devices").children(vec![Node::new("error").attr("code", "404")]),
        ]);
        let summary = partial.ingest(&reopened, &reply(&iq, vec![error])).unwrap();
        assert_eq!(summary.partial_errors, 1);
        assert_eq!(summary.omitted_users, 0);
        assert!(
            summary
                .require_complete()
                .unwrap_err()
                .to_string()
                .contains("per-user error")
        );
        assert_eq!(device_list(&reopened, &other).unwrap().unwrap(), saved[1]);
        assert_eq!(
            query_record(&reopened, &refused.iq_id)
                .unwrap()
                .unwrap()
                .status,
            "refused"
        );
    }

    #[test]
    fn historical_query_records_keep_omission_unknown() {
        let old = serde_json::json!({"users":[],"status":"server_acknowledged","code":null,"ingested":0,"partial_errors":0,"requested_at":0});
        let record: QueryRecord = serde_json::from_value(old).unwrap();
        assert_eq!(record.omitted_users, None);
        assert_eq!(record.generations, None);
        assert_eq!(record.superseded_users, None);
        assert_eq!(record.verification_failures, None);
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
        let unpinned_user = user_node(&unpinned, &[(0, 0, false)], None);
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
                partial_errors: 1,
                omitted_users: 0,
                superseded_users: 0,
                verification_errors: 0,
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
            Some("no signed index list")
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

        // Failed candidates retain the complete prior list across restart.
        // A later valid signed list replaces it without resetting identity.
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
        let summary = query.ingest(&store, &node).unwrap();
        assert_eq!(summary.verification_errors, 1);
        assert!(summary.require_complete().is_err());
        let kept = device_list(&store, &contact()).unwrap().unwrap();
        assert_eq!(kept, pinned);
        assert!(kept.companions_verified);
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
        assert_eq!(device_list(&store, &contact()).unwrap().unwrap(), pinned);
        assert_eq!(
            query_record(&store, &query.iq_id)
                .unwrap()
                .unwrap()
                .verification_failures
                .unwrap()
                .get(&DeviceVerificationFailure::StaleTimestamp),
            Some(&1)
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
            IngestSummary {
                omitted_users: 2,
                ..Default::default()
            }
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
