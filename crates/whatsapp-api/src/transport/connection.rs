//! Persistent comparison connection. Explicit opt-in; the actor owns reconnect
//! policy. The node-level loop (`drive`) is separate from the encrypted socket so
//! tests can cut the link at exact points without a server or Noise peer.
use super::{
    ConnectionEpoch,
    comparison::{ComparisonEndpoint, ComparisonGateway},
    handshake::{COMPARISON_CHAIN_POLICY, EncryptedConnection, VerifiedHandshake},
    noise::StaticKeypair,
    socket::{SocketLimits, connect_comparison},
};
use crate::{
    Error, Result,
    binary::{Encoder, Node, TokenDictionary},
    model::time::Timestamp,
    registration::request::LabRegistrationConfig,
    session::{AccountRecord, ObservedCredential},
    store::{ProtocolStore, Table},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot, watch};

const LOGIN_KEY: &[u8] = b"comparison-primary-login-v1";

/// Establish one certificate-checked primary session using saved identity only.
pub async fn connect(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
    epoch: ConnectionEpoch,
) -> Result<EncryptedConnection<tokio::net::TcpStream>> {
    // Validate every local login fact before DNS, TCP or Noise work.
    let (local, payload, snapshot) = login_material(store, config)?;
    let endpoint = ComparisonEndpoint::built_in(ComparisonGateway::Primary, 443)
        .map_err(|_| Error::invalid("connection", "invalid built-in endpoint"))?;
    let socket = connect_comparison(
        &endpoint,
        epoch,
        SocketLimits {
            connect_timeout: Duration::from_millis(config.transport.connect_timeout_ms),
            io_timeout: Duration::from_millis(config.transport.timeout_ms),
            ..Default::default()
        },
    )
    .await?;
    let verified = VerifiedHandshake::start(
        socket,
        &local,
        epoch,
        Timestamp::now(),
        &COMPARISON_CHAIN_POLICY,
        &crate::signal::crypto::SignalSignatureVerifier,
    )
    .await?;
    let reservation = reserve_login(store, &snapshot)?;
    let mut channel = verified.finish(&payload).await?;
    let reply = super::messaging::read_node(&mut channel).await?;
    super::inbox::preserve_in_epoch(store, &reply, epoch)?;
    record_login_reply_checked(store, &reply, Some((&snapshot, &reservation)))?;
    Ok(channel)
}

/// In-memory, zeroized account/material snapshot for local login ownership.
/// Does not authorize a request or prove server acceptance; never exposes keys.
pub struct LoginSnapshot {
    account: zeroize::Zeroizing<Vec<u8>>,
    #[cfg(feature = "experimental-signal-crypto")]
    material: Option<zeroize::Zeroizing<Vec<u8>>>,
}

impl std::fmt::Debug for LoginSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginSnapshot").finish_non_exhaustive()
    }
}

impl LoginSnapshot {
    /// Validate present registration material and capture this account in the
    /// caller's read transaction, before any login-side effects.
    pub fn capture_in(tx: &crate::store::ReadTx<'_>, account: &AccountRecord) -> Result<Self> {
        // A consistent pair can still be a different account key. Check the
        // existing registration copy before a socket or login reservation exists.
        // Local invariant, not an APK requirement for Android storage layout.
        #[cfg(feature = "experimental-signal-crypto")]
        crate::registration::factory::check_login_account_in(tx, account)?;
        Ok(Self {
            account: login_account_bytes(account)?,
            #[cfg(feature = "experimental-signal-crypto")]
            material: tx
                .get(
                    Table::Account,
                    crate::registration::factory::MATERIAL_KEY,
                    crate::registration::factory::RECORD_VERSION,
                )?
                .map(zeroize::Zeroizing::new),
        })
    }

    /// Refuse a changed account/material inside the caller's write transaction.
    /// The caller must separately check its reservation and perform settlement
    /// in that same transaction. No locks are held across network awaits.
    pub fn validate_in_write(&self, tx: &crate::store::WriteTx<'_>) -> Result<()> {
        check_login_snapshot(tx, self)
    }
}

fn login_account_bytes(account: &AccountRecord) -> Result<zeroize::Zeroizing<Vec<u8>>> {
    serde_json::to_vec(account)
        .map(zeroize::Zeroizing::new)
        .map_err(|_| Error::invalid("connection", "cannot encode login account snapshot"))
}

fn check_login_snapshot(tx: &crate::store::WriteTx<'_>, snapshot: &LoginSnapshot) -> Result<()> {
    let account = AccountRecord::load_in_write(tx)?.ok_or(Error::NotRegistered)?;
    if login_account_bytes(&account)? != snapshot.account {
        return Err(Error::invalid(
            "connection",
            "login account changed after preflight",
        ));
    }
    #[cfg(feature = "experimental-signal-crypto")]
    {
        use crate::registration::factory::{MATERIAL_KEY, RECORD_VERSION};
        let material = tx
            .get(Table::Account, MATERIAL_KEY, RECORD_VERSION)?
            .map(zeroize::Zeroizing::new);
        if material != snapshot.material {
            return Err(Error::invalid(
                "connection",
                "registration material changed after preflight",
            ));
        }
    }
    Ok(())
}

fn reserve_login(store: &ProtocolStore, snapshot: &LoginSnapshot) -> Result<String> {
    let reservation = super::messaging::new_message_id()?;
    store.write(|tx| {
        check_login_snapshot(tx, snapshot)?;
        let previous: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, LOGIN_KEY, 1)?;
        if previous.is_some_and(|v| v["status"] != "success") {
            return Err(Error::invalid("connection", "login already reserved"));
        }
        tx.put_json(
            Table::Checkpoints,
            LOGIN_KEY,
            1,
            &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now(), "reservation":reservation}),
        )
    })?;
    Ok(reservation)
}

// This seam performs only configuration/store reads and payload construction.
// Keep it before all socket/handshake work in `connect`; no login reservation
// should be written for local facts that cannot form a request.
fn login_material(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
) -> Result<(StaticKeypair, zeroize::Zeroizing<Vec<u8>>, LoginSnapshot)> {
    config.validate()?;
    let (account, snapshot) = store.read(|tx| {
        let account = AccountRecord::load(tx)?.ok_or(Error::NotRegistered)?;
        let snapshot = LoginSnapshot::capture_in(tx, &account)?;
        Ok((account, snapshot))
    })?;
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::NotRegistered);
    }
    let previous: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, LOGIN_KEY, 1))?;
    if previous.is_some_and(|v| v["status"] != "success") {
        return Err(Error::invalid(
            "connection",
            "previous login refused or uncertain; inspect before retry",
        ));
    }
    let payload = super::login::primary_login(&account, config)?;
    let key = account
        .noise_static
        .as_ref()
        .ok_or_else(|| Error::invalid("connection", "missing saved Noise identity"))?;
    let local = StaticKeypair::from_parts(key.0.private, key.0.public);
    local.validate_curve25519()?;
    Ok((local, payload, snapshot))
}

// One transaction commits the login result, clock sample and own identity.
#[cfg(test)]
fn record_login_reply(store: &ProtocolStore, reply: &Node) -> Result<()> {
    record_login_reply_checked(store, reply, None)
}

fn record_login_reply_checked(
    store: &ProtocolStore,
    reply: &Node,
    ownership: Option<(&LoginSnapshot, &str)>,
) -> Result<()> {
    let status = if reply.description == "success" {
        "success"
    } else {
        "refused"
    };
    // `<failure reason=N>`: observed 403 for a banned account (2026-11-01).
    // 405 is the outdated-client code in public multi-device clients
    // (reference only, not traced in the APK); keeping the number is what
    // tells a retired app version apart from any other refusal.
    let code = (status == "refused")
        .then(|| reply.get_attr("reason").and_then(|v| v.as_text()))
        .flatten()
        .filter(|v| v.len() <= 5)
        .and_then(|v| v.parse::<u32>().ok());
    store.write(|tx| {
        if let Some((snapshot, reservation)) = ownership {
            check_login_snapshot(tx, snapshot)?;
            let current: Option<serde_json::Value> =
                tx.get_json(Table::Checkpoints, LOGIN_KEY, 1)?;
            if !current.is_some_and(|v| {
                v["status"] == "outcome_unknown" && v["reservation"].as_str() == Some(reservation)
            }) {
                return Err(Error::invalid(
                    "connection",
                    "login reply no longer owns reservation",
                ));
            }
        }

        tx.put_json(
            Table::Checkpoints,
            LOGIN_KEY,
            1,
            &serde_json::json!({"status":status,"code":code,"at":Timestamp::now()}),
        )?;
        if status == "success" {
            super::clock::observe_login(tx, reply)?;
            let mut current = AccountRecord::load_in_write(tx)?.ok_or(Error::NotRegistered)?;
            current.last_authenticated_at = Some(Timestamp::now());
            let pn = update_login_identity(tx, reply)?;
            if current.own_jid.is_none() {
                current.own_jid = pn;
            }
            current.save(tx)?;
        }
        Ok(())
    })?;
    if status != "success" {
        return Err(Error::Authentication { code });
    }
    Ok(())
}

fn login_user(
    jid: crate::model::jid::Jid,
    server: crate::model::jid::Server,
) -> Option<crate::model::jid::Jid> {
    use crate::model::jid::JidKind;
    (jid.server == server
        && !jid.user.is_empty()
        && jid.agent == 0
        && matches!(jid.kind, JidKind::Plain | JidKind::AgentDevice))
    .then(|| jid.to_non_ad())
}

/// Apply supplied PN/LID fields from an authenticated login success in the
/// caller's settlement transaction. Omitted fields preserve known namespaces;
/// malformed supplied/history fields refuse. The caller owns authentication,
/// capture and reservation checks; this helper sends nothing.
pub fn update_login_identity(
    tx: &mut crate::store::WriteTx<'_>,
    reply: &Node,
) -> Result<Option<crate::model::jid::Jid>> {
    use crate::model::jid::{Jid, Server};
    if reply.description != "success" {
        return Err(Error::invalid(
            "connection",
            "identity update requires login success",
        ));
    }
    // Authenticated success fields are observed live, not a guarantee that both
    // appear every time. Native 08i keeps typed own PN/LID separately; our local
    // partial-update contract is documented in login-identity-updates.md.
    let supplied = |field, server, diagnostic| -> Result<Option<Jid>> {
        if reply.get_attr(field).is_none() {
            return Ok(None);
        }
        super::receipts::jid(reply, field)
            .and_then(|jid| login_user(jid, server))
            .map(Some)
            .ok_or_else(|| Error::invalid("connection", diagnostic))
    };
    let pn = supplied(
        "pn",
        Server::User,
        "success pn is not a known phone-number user",
    )?;
    let lid = supplied("lid", Server::Lid, "success lid is not a known LID user")?;
    if pn.is_none() && lid.is_none() {
        return Ok(None);
    }
    let mut saved = tx
        .get_json::<serde_json::Value>(Table::Checkpoints, b"login-identity", 1)?
        .unwrap_or_else(|| serde_json::json!({"pn":null,"lid":null}));
    let object = saved
        .as_object_mut()
        .ok_or_else(|| Error::invalid("connection", "saved login-identity is not an object"))?;
    for (field, server, diagnostic) in [
        ("pn", Server::User, "saved login-identity pn is malformed"),
        ("lid", Server::Lid, "saved login-identity lid is malformed"),
    ] {
        if let Some(value) = object.get(field).filter(|value| !value.is_null()) {
            let valid = value
                .as_str()
                .and_then(|text| Jid::parse(text).ok())
                .and_then(|jid| login_user(jid, server));
            if valid.is_none() {
                return Err(Error::invalid("connection", diagnostic));
            }
        }
    }
    // Absence never erases an authenticated namespace. Preserve all untouched
    // fields and refuse malformed history rather than silently repairing it.
    if let Some(pn) = &pn {
        object.insert("pn".into(), serde_json::Value::String(pn.to_string()));
    }
    if let Some(lid) = lid {
        object.insert("lid".into(), serde_json::Value::String(lid.to_string()));
    }
    object.insert("at".into(), serde_json::json!(Timestamp::now()));
    tx.put_json(Table::Checkpoints, b"login-identity", 1, &saved)?;
    Ok(pn)
}

/// One outbound stanza and where to report the socket write outcome.
type Written = (Node, oneshot::Sender<Result<()>>);
enum WriteGuard {
    Prekey(Arc<ProtocolStore>, super::prekeys::PendingUpload),
    Devices(Arc<ProtocolStore>, super::devices::DeviceQuery),
    RetryRefetch(Arc<ProtocolStore>, super::retry_answer::RefetchWriteGuard),
    Action(Arc<ProtocolStore>, super::actions::ActionWriteGuard),
    Receipt(Arc<ProtocolStore>, super::receipts::ReceiptWriteGuard),
}
pub(crate) struct Outbound {
    node: Node,
    reply: oneshot::Sender<Result<()>>,
    guard: Option<WriteGuard>,
}
impl Outbound {
    // A refusal consumes the queued write and reports its actual local error.
    // The writer ends the epoch without encoding, writing, or releasing a batch.
    pub(crate) fn prepare(self) -> Option<Written> {
        let admission = match self.guard {
            Some(WriteGuard::Prekey(store, pending)) => pending.check_write_authority(&store),
            Some(WriteGuard::Devices(store, query)) => query.check_write_authority(&store),
            Some(WriteGuard::RetryRefetch(store, pending)) => pending.check_write_authority(&store),
            Some(WriteGuard::Action(store, action)) => action.check_write_authority(&store),
            Some(WriteGuard::Receipt(store, receipt)) => receipt.check_write_authority(&store),
            None => Ok(()),
        };
        if let Err(error) = admission {
            let _ = self.reply.send(Err(error));
            return None;
        }
        Some((self.node, self.reply))
    }
}

/// Node-level link for one connection epoch. Real sessions adapt the split
/// encrypted transport; tests inject channels and cut them at chosen points.
pub(crate) struct NodeLink {
    incoming: mpsc::Receiver<Result<Inbound>>,
    outgoing: mpsc::Sender<Outbound>,
}

/// One item from the reader task.
pub(crate) enum Inbound {
    Node(Node),
    /// A frame that decrypted, so the stream is intact, but whose payload did
    /// not unpack or decode. It carries the decrypted frame for capture, a
    /// fixed error class and, when readable, the stanza's description, so a
    /// terminal stanza still ends the session; there is no ID to acknowledge.
    Undecodable {
        frame: Vec<u8>,
        class: &'static str,
        description: Option<String>,
    },
}

/// Unpack and decode one decrypted frame.
fn inbound_from_frame(frame: &[u8]) -> Inbound {
    match super::inbox::decode_frame(frame) {
        Ok(node) => Inbound::Node(node),
        Err(error) => Inbound::Undecodable {
            frame: frame.to_vec(),
            class: error.class(),
            description: super::inbox::peek_frame_description(frame),
        },
    }
}

impl NodeLink {
    pub(crate) fn new(
        incoming: mpsc::Receiver<Result<Inbound>>,
        outgoing: mpsc::Sender<Outbound>,
    ) -> Self {
        Self { incoming, outgoing }
    }
    // APK43.72 14h.A00 submits the ACK before running the deferred Fe5
    // callback. Reservation has committed before this handoff. Any write error
    // ends the epoch; neither an ACK nor a resend authorizes replay.
    pub(crate) async fn send_retry_answer(
        &self,
        store: &Arc<ProtocolStore>,
        answer: &super::retry_answer::ReservedAnswer,
    ) -> Result<bool> {
        let guard = || {
            answer
                .refetch
                .as_ref()
                .map(|pending| WriteGuard::RetryRefetch(store.clone(), pending.clone()))
        };
        let confirmed = if let Some(confirmation) = &answer.confirmation {
            self.send_guarded(confirmation, guard()).await?;
            true
        } else {
            false
        };
        self.send_guarded(&answer.node, guard()).await?;
        Ok(confirmed)
    }

    /// Write one stanza and wait for the socket outcome. A closed writer is a
    /// consumed connection: the caller must end this epoch, never retry here.
    async fn send(&self, node: &Node) -> Result<()> {
        self.send_guarded(node, None).await
    }
    pub(crate) async fn send_receipt(
        &self,
        store: &Arc<ProtocolStore>,
        receipt: &super::receipts::ReservedReceipt,
    ) -> Result<()> {
        self.send_guarded(
            receipt.node(),
            Some(WriteGuard::Receipt(store.clone(), receipt.guard.clone())),
        )
        .await
    }

    pub(crate) async fn send_action(
        &self,
        store: &Arc<ProtocolStore>,
        action: &super::actions::ReservedAction,
    ) -> Result<()> {
        self.send_guarded(
            &action.node,
            Some(WriteGuard::Action(store.clone(), action.guard.clone())),
        )
        .await
    }

    async fn send_prekey_upload(
        &self,
        store: &Arc<ProtocolStore>,
        pending: super::prekeys::PendingUpload,
        node: &Node,
    ) -> Result<()> {
        self.send_guarded(node, Some(WriteGuard::Prekey(store.clone(), pending)))
            .await
    }
    async fn send_device_query(
        &self,
        store: &Arc<ProtocolStore>,
        query: &super::devices::DeviceQuery,
        node: &Node,
    ) -> Result<()> {
        self.send_guarded(
            node,
            Some(WriteGuard::Devices(store.clone(), query.clone())),
        )
        .await
    }
    async fn send_group_step(
        &self,
        store: &Arc<ProtocolStore>,
        pending: Option<&super::outgoing_group::Pending>,
        node: &Node,
    ) -> Result<()> {
        let guard = pending
            .and_then(|p| p.device_query())
            .map(|query| WriteGuard::Devices(store.clone(), query.clone()));
        self.send_guarded(node, guard).await
    }
    async fn send_guarded(&self, node: &Node, guard: Option<WriteGuard>) -> Result<()> {
        let (reply, outcome) = oneshot::channel();
        self.outgoing
            .send(Outbound {
                node: node.clone(),
                reply,
                guard,
            })
            .await
            .map_err(|_| Error::Transport(super::TransportError::Closed))?;
        outcome
            .await
            .map_err(|_| Error::Transport(super::TransportError::Closed))?
    }
}

/// Reader/writer tasks behind a real link; aborted when the epoch ends.
struct LinkTasks(tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>);
impl Drop for LinkTasks {
    fn drop(&mut self) {
        self.0.abort();
        self.1.abort();
    }
}
fn encode(node: &Node) -> Result<Vec<u8>> {
    Ok(Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(node)?)
}
fn link_from(channel: EncryptedConnection<tokio::net::TcpStream>) -> Result<(NodeLink, LinkTasks)> {
    let (mut reader, mut writer) = channel.into_split()?;
    let (incoming_tx, incoming_rx) = mpsc::channel(64);
    let reader_task = tokio::spawn(async move {
        loop {
            let next = match reader.read().await {
                Ok(Some(bytes)) => Ok(inbound_from_frame(&bytes)),
                // Classified as recoverable: the actor may reconnect within its bounds.
                Ok(None) => Err(Error::Transport(super::TransportError::PeerClosed)),
                Err(e) => Err(e.into()),
            };
            let failed = next.is_err();
            if incoming_tx.send(next).await.is_err() || failed {
                break;
            }
        }
    });
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<Outbound>(16);
    let writer_task = tokio::spawn(async move {
        while let Some(outbound) = outgoing_rx.recv().await {
            let Some((node, reply)) = outbound.prepare() else {
                break;
            };
            let result = match encode(&node) {
                Ok(bytes) => writer.write(&bytes).await.map_err(Error::from),
                Err(e) => Err(e),
            };
            let failed = result.is_err();
            let _ = reply.send(result);
            if failed {
                // A failed write poisons the epoch; later sends see a closed link.
                break;
            }
        }
    });
    Ok((
        NodeLink::new(incoming_rx, outgoing_tx),
        LinkTasks(reader_task, writer_task),
    ))
}

/// Everything one epoch needs besides its link and control channels.
pub(crate) struct Session {
    pub(crate) store: Arc<ProtocolStore>,
    pub(crate) config: LabRegistrationConfig,
    pub(crate) epoch: ConnectionEpoch,
    pub(crate) send_outbox: bool,
    pub(crate) ticket: Option<crate::model::ids::LocalMessageId>,
}

/// Run continuously until explicit stop, EOF, timeout or refusal. Caller owns
/// reconnect policy; this function never replays uncertain chats or resets state.
/// `ready` resolves only after the server accepts connection activation.
pub async fn run(
    store: Arc<ProtocolStore>,
    config: LabRegistrationConfig,
    epoch: ConnectionEpoch,
    stop: watch::Receiver<bool>,
    ready: oneshot::Sender<()>,
    send_outbox: bool,
    ticket: Option<crate::model::ids::LocalMessageId>,
) -> Result<()> {
    let channel = connect(&store, &config, epoch).await?;
    // Abort-on-drop also covers callers cancelling this whole future.
    let (link, _tasks) = link_from(channel)?;
    drive(
        Session {
            store,
            config,
            epoch,
            send_outbox,
            ticket,
        },
        stop,
        ready,
        link,
    )
    .await
}

#[derive(Default)]
// Keep manual action pacing outside select handlers: a whole reserved batch
// previously blocked inbound processing during each inter-action sleep.
struct ActionDrain {
    remaining: usize,
    due: Option<tokio::time::Instant>,
}
impl ActionDrain {
    fn trigger(&mut self) {
        if self.remaining == 0 {
            self.remaining = super::actions::BATCH;
            self.due = Some(tokio::time::Instant::now());
        }
    }
    fn deadline(&self) -> tokio::time::Instant {
        self.due.unwrap_or_else(tokio::time::Instant::now)
    }
    fn sent(&mut self, delay_ms: u64) {
        self.remaining -= 1;
        self.due = (self.remaining > 0)
            .then(|| tokio::time::Instant::now() + Duration::from_millis(delay_ms));
    }
    fn finish(&mut self) {
        self.remaining = 0;
        self.due = None;
    }
}

/// Read the login checkpoint status string, if any checkpoint is saved.
pub fn last_login_status(store: &ProtocolStore) -> Result<Option<String>> {
    store.read(|tx| {
        tx.get_json::<serde_json::Value>(Table::Checkpoints, LOGIN_KEY, 1)?
            .map(|v| {
                v["status"]
                    .as_str()
                    .map(|s| s.to_owned())
                    .ok_or_else(|| Error::invalid("checkpoint", "status field is not a string"))
            })
            .transpose()
    })
}

struct PendingPing {
    id: String,
    deadline: tokio::time::Instant,
    response_claim: bool,
}

impl PendingPing {
    fn claim_reply(&mut self, node: &Node) -> bool {
        if !self.response_claim
            || tokio::time::Instant::now() >= self.deadline
            || !super::messaging::iq_reply_matches(node, &self.id)
        {
            return false;
        }
        // APK43.72 0ln.A02 removes the IQ owner before 0ln.A01/1xa.CK4.
        self.response_claim = false;
        true
    }
}

fn validate_keepalive_result(node: &Node, id: &str) -> std::result::Result<(), &'static str> {
    // APK43.72 1xa.CK4 -> 8B6.A0F/A0J: JID from, request-bound id,
    // required inclusive timestamp bounds and exact result type.
    if node.description != "iq" || node.get_attr("type").and_then(|v| v.as_text()) != Some("result")
    {
        return Err("type");
    }
    let from = node.get_attr("from").ok_or("from")?;
    if from.as_jid().is_none()
        && from
            .as_text()
            .and_then(|v| crate::model::jid::Jid::parse(v).ok())
            .is_none()
    {
        return Err("from");
    }
    if node.get_attr("id").and_then(|v| v.as_text()) != Some(id) {
        return Err("id");
    }
    let timestamp = node
        .get_attr("t")
        .and_then(|v| v.as_text())
        .and_then(super::mex::java_decimal_long)
        .ok_or("t")?;
    if !(1_577_865_600..=4_102_473_600).contains(&timestamp) {
        return Err("t");
    }
    Ok(())
}

fn activation_matches(node: &Node, id: &str, deadline: tokio::time::Instant) -> bool {
    super::messaging::iq_reply_matches(node, id) && tokio::time::Instant::now() < deadline
}

/// The authenticated session loop over a node link. Every durable effect
/// (capture, session/inbox commit, reservation) precedes the socket write it
/// leads to, so a cut at any await leaves state a later epoch can continue from.
pub(crate) async fn drive(
    session: Session,
    mut stop: watch::Receiver<bool>,
    ready: oneshot::Sender<()>,
    mut link: NodeLink,
) -> Result<()> {
    let Session {
        store,
        config,
        epoch,
        send_outbox,
        ticket,
    } = session;
    let account = store
        .read(AccountRecord::load)?
        .ok_or(Error::NotRegistered)?;
    super::groups::recover_pending(&store)?;
    super::app_state_sync::recover_fetch_backoff(&store)?;
    super::identity_notifications::application::recover(&store, config.transport.identity_changes)
        .await?;
    // Reply settlement and contact projection are separate durable commits.
    // Repair the gap after a crash without deleting markers or replaying IQs.
    super::contacts::ingest(&store)?;
    let active_id = super::messaging::new_message_id()?;
    // APK43.72 0Sf.A0A/0ln.A08 register the callback timer before dispatch.
    let activated_deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    link.send(&super::messaging::active_request(&active_id))
        .await?;
    let mut ready = Some(ready);
    let ping_period = Duration::from_millis((config.transport.timeout_ms / 2).clamp(1000, 20000));
    // Maintenance stays on its own local cadence. Native 1A7.A05 restarts
    // periodic ping scheduling from each accepted response, not a fixed tick.
    let mut maintenance_due = tokio::time::Instant::now() + ping_period;
    let mut next_ping_due = maintenance_due;
    let mut pending_ping: Option<PendingPing> = None;
    // Pending jobs stay durable, but callback authority is local to this link.
    let mut retry_refetches = super::retry_answer::ActiveRefetches::default();
    let mut outgoing: Option<super::outgoing::Pending> = None;
    let mut group_outgoing: Option<super::outgoing_group::Pending> = None;
    let mut upload: Option<super::prekeys::PendingUpload> = None;
    let mut upload_rounds: u32 = 0;
    let mut device_query: Option<super::devices::DeviceQuery> = None;
    let mut device_cursor = super::devices::DeviceSyncCursor::default();
    let mut device_context = super::devices::SyncContext::Interactive;
    let mut action_drain = ActionDrain::default();
    let mut action_replies = super::actions::LiveReplies::default();
    let mut outbox_tick = tokio::time::interval(Duration::from_secs(1));
    // Set by a frame whose stanza name could not be read; cleared by the next
    // decoded stanza. A server close right after such a frame may be the end
    // of a session we could not read, so it is not treated as recoverable.
    let mut unreadable_stanza = false;
    outbox_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        if *stop.borrow() {
            return Ok(());
        }
        let ping_deadline = pending_ping
            .as_ref()
            .map(|ping| ping.deadline)
            .unwrap_or_else(tokio::time::Instant::now);
        let upload_deadline = upload
            .as_ref()
            .map(|u| u.deadline)
            .unwrap_or_else(tokio::time::Instant::now);
        let device_deadline = device_query
            .as_ref()
            .map(|q| q.deadline)
            .unwrap_or_else(tokio::time::Instant::now);
        let outgoing_deadline = outgoing
            .as_ref()
            .map(|q| q.deadline)
            .unwrap_or_else(tokio::time::Instant::now);
        let group_deadline = group_outgoing
            .as_ref()
            .map(|q| q.deadline)
            .unwrap_or_else(tokio::time::Instant::now);
        let action_deadline = action_replies.deadline();
        let maintenance_or_ping_due = if pending_ping.is_none() {
            maintenance_due.min(next_ping_due)
        } else {
            maintenance_due
        };
        tokio::select! {
            _ = tokio::time::sleep_until(action_deadline.unwrap_or_else(tokio::time::Instant::now)), if action_deadline.is_some() => {
                action_replies.expire();
            }
            _ = tokio::time::sleep_until(group_deadline), if group_outgoing.is_some() => {
                group_outgoing.take().expect("expired group preparation").expire(&store, &mut device_cursor)?;
            }
            _ = tokio::time::sleep_until(outgoing_deadline), if outgoing.is_some() => {
                return Err(Error::invalid("outbox", "key query outcome unknown; reserved, no retry"));
            }
            // Like 43.72 0ln.A08's ID-specific timer, expiration belongs to
            // each pending request rather than the heartbeat's phase.
            _ = tokio::time::sleep_until(upload_deadline), if upload.is_some() => {
                tracing::warn!("prekey upload outcome unknown; reservation kept, no retry");
                upload = None;
            }
            _ = tokio::time::sleep_until(device_deadline), if device_query.is_some() => {
                tracing::warn!("device query unanswered; users remain pending");
                if let Some(query) = device_query.take() { device_cursor.defer(&query); }
            }
            _ = tokio::time::sleep_until(ping_deadline), if pending_ping.is_some() => {
                return Err(Error::Timeout(ping_period));
            }
            _ = tokio::time::sleep_until(action_drain.deadline()), if ready.is_none() && action_drain.remaining > 0 => {
                if let Some(action) = super::actions::reserve_next_owned(&store)? {
                    link.send_action(&store, &action).await?;
                    action_replies.record_written(&action)?;
                    action_drain.sent(config.transport.action_delay_ms);
                } else {
                    action_drain.finish();
                }
            }
            _ = outbox_tick.tick(), if send_outbox && ready.is_none() => {
                if outgoing.is_none() && group_outgoing.is_none() && let Some(id)=ticket
                    && let Some(stanza)=super::outgoing::resume_identity_selected(&store,&account,id,epoch,config.transport.outgoing_identity_changes).await? {
                    link.send(&stanza).await?;
                    continue;
                }
                if outgoing.is_none() && group_outgoing.is_none() {
                    let selected = match ticket {
                        Some(id) => Some(id),
                        None => super::outgoing::eligible(&store, 1)?.first().map(|v|v.local_id),
                    };
                    if let Some(id) = selected {
                        if let Some((pending,node))=super::outgoing_group::begin_selected(&store,id)? {
                            group_outgoing=Some(pending);
                            link.send_group_step(&store, group_outgoing.as_ref(), &node).await?;
                        } else if let Some((pending,node))=super::outgoing::begin_selected(&store,Some(id))? {
                            outgoing=Some(pending);
                            link.send(&node).await?;
                        }
                    }
                }
            }
            changed = stop.changed() => { if changed.is_err() || *stop.borrow() { return Ok(()); } }
            _ = tokio::time::sleep_until(activated_deadline), if ready.is_some() => {
                return Err(Error::invalid("connection", "activation outcome unknown; frames preserved"));
            }
            _ = tokio::time::sleep_until(maintenance_or_ping_due), if ready.is_none() => {
                if tokio::time::Instant::now() >= maintenance_due {
                    maintenance_due = tokio::time::Instant::now() + ping_period;
                    // Authenticated primary-key changes and offline requests can
                    // queue work during this connection without a devices notification.
                    if device_query.is_none() && group_outgoing.as_ref().is_none_or(|g| !g.devices_pending())
                        && let Some((query,node))=device_cursor.begin_next(&store,device_context)? {
                        device_query=Some(query);
                        link.send_device_query(&store, device_query.as_ref().expect("reserved device query"), &node).await?;
                    }
                    for receipt in super::receipts::reserve_pending_owned(&store)? { link.send_receipt(&store,&receipt).await?; }
                    action_drain.trigger();
                }
                // A maintenance write can delay a due ping. Its response window
                // belongs to its own admission (43.72 0Sf.A06 -> 0ln.A08).
                if pending_ping.is_some() || tokio::time::Instant::now() < next_ping_due { continue; }
                let id = super::messaging::new_message_id()?;
                // Comparison LX/EIY.<init>(VLI), branch 15, 0x006a..009e.
                let ping = Node::new("iq").attr("id", id.as_str()).attr("type", "get")
                    .attr("xmlns", "w:p").attr("to", crate::model::jid::Jid::parse("s.whatsapp.net")?);
                pending_ping = Some(PendingPing { id, deadline: tokio::time::Instant::now() + ping_period, response_claim: true });
                link.send(&ping).await?;
            }
            incoming = link.incoming.recv() => {
                let inbound = match incoming.ok_or_else(|| Error::invalid("connection", "reader ended"))? {
                    Err(Error::Transport(super::TransportError::PeerClosed)) if unreadable_stanza => {
                        return Err(Error::invalid("connection", "server closed the connection after a stanza this client could not read; captured; no reconnect (whatsapp-cli undecodable)"));
                    }
                    other => other?,
                };
                let node = match inbound {
                    Inbound::Node(node) => {
                        unreadable_stanza = false;
                        node
                    }
                    // A redelivered frame this decoder refuses (a newer token,
                    // say) used to end receive at every login, uncaptured.
                    // Kept for inspection and later re-decoding; nothing to ACK.
                    // Continuing is local policy: where the APK catches a
                    // decode error is not traced.
                    Inbound::Undecodable { frame, class, description } => {
                        super::inbox::preserve_undecodable(&store, &frame, class, Some(epoch))?;
                        match description.as_deref() {
                            // The server ended the session, as with a decodable
                            // one below: never reconnect into a refusal.
                            Some("failure" | "stream:error") => {
                                return Err(Error::invalid("connection", "server ended session with a stanza this client could not decode; captured; no reconnect"));
                            }
                            Some(_) => {}
                            None => unreadable_stanza = true,
                        }
                        tracing::warn!(class, "inbound frame did not decode; captured without ACK; session continues");
                        continue;
                    }
                };
                let capture = super::inbox::preserve_in_epoch(&store, &node, epoch)?;
                super::messaging::validate_received_ack(&node)?;
                super::clock::observe(&store, &node)?;
                // APK 43.72 1rA.A03 -> 1lZ.A0t -> 0ZJ.A0q handles
                // get/urn:xmpp:ping even without an id or usable timestamp.
                // 0ZJ.A07 selects writer command 0; 0v0.A00 emits only
                // iq/type=result/to=s.whatsapp.net (no echoed id/namespace).
                if node.description == "iq"
                    && node.get_attr("type").and_then(|v| v.as_text()) == Some("get")
                    && node.get_attr("xmlns").and_then(|v| v.as_text()) == Some("urn:xmpp:ping")
                {
                    let pong = Node::new("iq").attr("type", "result")
                        .attr("to", crate::model::jid::Jid::parse("s.whatsapp.net")?);
                    link.send(&pong).await?;
                    continue;
                }
                if matches!(node.description.as_str(), "failure" | "stream:error") {
                    if node.description == "failure"
                        && let Some(token) = node.get_attr("appeal_token").and_then(|v| v.as_text())
                    {
                        let token_bytes = zeroize::Zeroizing::new(token.as_bytes().to_vec());
                        let now = Timestamp::now();
                        let _ = store.write(|tx| {
                            let mut account = AccountRecord::load_in_write(tx)?.unwrap_or_default();
                            if account.credentials.len() < 64
                                && !account.credentials.iter().any(|c| c.name == "noise_appeal_token")
                            {
                                account.credentials.push(ObservedCredential {
                                    name: "noise_appeal_token".into(),
                                    value: token_bytes,
                                    observed_at: now,
                                    source_step: Some("failure".into()),
                                });
                                AccountRecord::save(&account, tx)?;
                            }
                            Ok(())
                        });
                    }
                    return Err(Error::invalid("connection", "server ended session; reason preserved encrypted; no reconnect"));
                }
                // A reserved group reaction waits for each out-of-band
                // distribution's ACK; the ACK is then processed as usual.
                if node.description == "ack" && group_outgoing.as_ref().is_some_and(|v|tokio::time::Instant::now() < v.deadline && v.matches(&node)) {
                    let next=group_outgoing.take().expect("matched group distribution").settle_with_policy(&store,&account,&node,epoch,&mut device_cursor,config.transport.outgoing_identity_changes).await?;
                    group_outgoing=next.pending;
                    if let Some(stanza)=next.node { link.send_group_step(&store, group_outgoing.as_ref(), &stanza).await?; }
                }
                if node.description == "iq" {
                    // A retry answer waiting for the peer's fresh bundle (retry_answer).
                    match retry_refetches.complete(&store, &account, &node, epoch).await {
                        Ok(Some(resend)) => {
                            if let Some(resend) = resend {
                                link.send(&resend).await?;
                                tracing::info!(target: "whatsapp_api::incoming", "retry receipt answered with a fresh session; same message id");
                            }
                            continue;
                        }
                        Ok(None) => {}
                        Err(Error::InvalidInput { field, reason }) => tracing::warn!(field, reason, "retry refetch not completed"),
                        Err(error) => return Err(error),
                    }
                    if group_outgoing.as_ref().is_some_and(|v|tokio::time::Instant::now() < v.deadline && v.matches(&node)) {
                        let next=group_outgoing.take().expect("matched group preparation").settle_with_policy(&store,&account,&node,epoch,&mut device_cursor,config.transport.outgoing_identity_changes).await?;
                        group_outgoing=next.pending;
                        if let Some(pending) = &group_outgoing { pending.observe_device_query(&mut device_cursor); }
                        if let Some(stanza)=next.node { link.send_group_step(&store, group_outgoing.as_ref(), &stanza).await?; }
                        continue;
                    }
                    if outgoing.as_ref().is_some_and(|v|tokio::time::Instant::now() < v.deadline && v.matches(&node)) {
                        if let Some(stanza)=outgoing.take().expect("matched pending query").prepare_with_policy(&store,&account,&node,epoch,config.transport.outgoing_identity_changes).await? {
                            link.send(&stanza).await?;
                        }
                        continue;
                    }
                    let id = node.get_attr("id").and_then(|v| v.as_text());
                    // A queued late result cannot win readiness by select order.
                    if ready.is_some() && activation_matches(&node, &active_id, activated_deadline) {
                        if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
                            return Err(Error::invalid("connection", "activation refused"));
                        }
                        // Announce availability, as every real client does
                        // (LX/1Rd->A01). Sent only once activation is confirmed:
                        // writing it alongside the activation request would block
                        // on a peer that has already gone away, since nothing
                        // reads a reply for a fire-and-forget presence stanza.
                        link.send(&super::messaging::available_presence()).await?;
                        // A display name queued by `push-name` (0y0 case 3).
                        if let Some(name) = super::presence::take_push_name(&store)? {
                            link.send(&name).await?;
                        }
                        for receipt in super::receipts::reserve_pending_owned(&store)? { link.send_receipt(&store,&receipt).await?; }
                        action_drain.trigger();
                        if let Some(ready) = ready.take() { let _ = ready.send(()); }
                        // LX/DNq.run: unsent prekeys (or a store that never generated any) upload at start.
                        if super::prekeys::wants_upload(&store)? {
                            maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds, PrekeyUploadTrigger::Startup).await?;
                        }
                        // SyncDevicesJob default preset: pending users are queried interactively.
                        if let Some((query, node)) = device_cursor.begin_next(&store, device_context)? {
                            device_query = Some(query);
                            link.send_device_query(&store, device_query.as_ref().expect("reserved device query"), &node).await?;
                        }
                        continue;
                    }
                    // Queued one-shot actions (block list, reports, lookups) settle by iq id.
                    if action_replies.record_reply(&store, &node)? {
                        if let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) {
                            match super::app_state_sync::ingest_action(&store, id) {
                                Ok(_) => {}
                                Err(Error::InvalidInput { field, reason }) => tracing::warn!(field, reason, "app-state reply retained without promotion"),
                                Err(error) => return Err(error),
                            }
                        }
                        super::contacts::ingest(&store)?;
                        if let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) {
                            // APK 37.71 8lQ.A01 separates truncated/full results;
                            // 8lQ.A00 sends full state to 0yw.A0U. Our supported
                            // snapshot projection runs here, not only in CLI list.
                            super::groups::apply_pending(&store, id)?;
                            let application = super::identity_notifications::application::apply(
                                &store, id, config.transport.identity_changes,
                            ).await?;
                            if application != super::identity_notifications::application::Application::Unrelated {
                                tracing::info!(?application, "identity refresh reply considered; notification ACK handled separately");
                            }
                        }
                        continue;
                    }
                    if device_query.as_ref().is_some_and(|q| tokio::time::Instant::now() < q.deadline && q.matches(&node)) {
                        let query = device_query.take().expect("matched device query");
                        let summary = match device_cursor.settle(&query, &store, &node) {
                            Ok(value) => value,
                            Err(error) if super::devices::is_expired(&error) => { tracing::info!("device query reply expired; users deferred"); continue; }
                            Err(error) => return Err(error),
                        };
                        if summary.verification_errors > 0 {
                            // Matching proves this is our opaque local reservation
                            // ID, not a user address or arbitrary response body.
                            tracing::warn!(query_id = node.get_attr("id").and_then(|v| v.as_text()).unwrap_or("unavailable"), verification_errors = summary.verification_errors, "device candidates refused locally; inspect device-query --id; saved lists and pending work retained");
                        }
                        tracing::info!(accepted = summary.accepted, ingested = summary.ingested, partial_errors = summary.partial_errors, omitted_users = summary.omitted_users, superseded_users = summary.superseded_users, verification_errors = summary.verification_errors, "device query settled");
                        if group_outgoing.as_ref().is_none_or(|g| !g.devices_pending())
                            && let Some((query, sync)) = device_cursor.begin_next(&store, device_context)? {
                            device_query = Some(query);
                            link.send_device_query(&store, device_query.as_ref().expect("reserved device query"), &sync).await?;
                        }
                        continue;
                    }
                    if upload.as_ref().is_some_and(|u| tokio::time::Instant::now() < u.deadline && u.matches(&node)) {
                        let Some(outcome) = settle_upload_reply(&store, upload.take().expect("matched pending upload"), &node)? else { continue; };
                        // 43.72 Fcj3/Fc8(7): success demand enters migrated ADD admission.
                        if outcome.accepted && (outcome.count_low || outcome.pq_count_low) {
                            maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds, PrekeyUploadTrigger::Success { legacy: outcome.count_low, pq: outcome.pq_count_low }).await?;
                        }
                        continue;
                    }
                    // APK 43.72 0ln.A01: non-reply IQs cannot settle a ping.
                    if pending_ping.as_mut().is_some_and(|ping| ping.claim_reply(&node)) {
                        if node.get_attr("type").and_then(|v| v.as_text()) == Some("result")
                            && let Err(field) = validate_keepalive_result(&node, id.expect("claimed ping id")) {
                            tracing::warn!(field, "malformed keepalive result ignored; timer remains pending");
                            continue;
                        }
                        // APK43.72 1xa.Bvj and CK4 both schedule Fea(24), which
                        // clears the outstanding ping without a connection refusal.
                        if node.get_attr("type").and_then(|v| v.as_text()) == Some("error") {
                            tracing::info!("keepalive error response received");
                        }
                        pending_ping = None;
                        next_ping_due = tokio::time::Instant::now() + ping_period;
                        continue;
                    }
                }
                let received = super::inbox::process_with_policy(&store, &account, &node, &capture, config.transport.identity_changes).await?;
                // A receipt, privacy-token or prekey-count stanza whose wire
                // content this client refuses is already captured; it stays
                // without ACK but must not end the session. The server
                // redelivers it at every login, so ending here would stop all
                // receiving for good (receive audit F1-F3,
                // docs/protocol/replay-safety-audit.md). Only the pure
                // validation stage is caught: once it passes, a refusal from
                // recording concerns our stored rows (a corrupt outbox entry,
                // an ambiguous wire ID) and ends the session visibly.
                let receipt_recorded = match super::receipts::validate(&node) {
                    Ok(()) => {
                        super::receipts::record(&store, &node)?;
                        true
                    }
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "receipt content refused; captured without ACK; session continues");
                        false
                    }
                    Err(error) => return Err(error),
                };
                super::receipts::record_ack(&store, &node)?;
                // Who has received or viewed a status we posted. Recording it
                // sends nothing: this is their report about our status, not a
                // read receipt we emit for theirs. Only for an accepted receipt,
                // so nothing is recorded from a stanza left unacknowledged.
                if receipt_recorded && let Some(receipt) = super::status_receipts::parse(&node) {
                    super::status_receipts::record(&store, &receipt)?;
                }
                match super::account_privacy::validate(&node) {
                    Ok(()) => {
                        if let Some(ack) = super::account_privacy::handle(&store, &node)? {
                            link.send(&ack).await?;
                        }
                    }
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "account privacy refused; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                match super::notifications::validate(&node) {
                    Ok(()) => {
                        if let Some(ack) = super::notifications::handle(&store, &node)? {
                            link.send(&ack).await?;
                        }
                    }
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "notification content refused; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                if let Some(ack)=super::groups::handle_dirty_notification(&store, &node)? {link.send(&ack).await?;}
                // 2.26.37.71 0yv (BaseNotificationHandler) ACKs every other
                // w:gp2 too; unacknowledged ones ended sessions.
                let mex_ready = match super::notifications::record_reachout_timelock(&store, &node) {
                    Ok(lock) => {
                        if let Some(lock) = lock {
                            tracing::warn!(active = lock.active, ends_unix = lock.ends, "WhatsApp reach-out timelock updated");
                        }
                        true
                    }
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "reach-out update refused; prior restriction preserved; captured without ACK; session continues");
                        false
                    }
                    Err(error) => return Err(error),
                };
                match if mex_ready { super::notifications::mex_ack(&node) } else { Ok(None) } {
                    Ok(Some(ack)) => link.send(&ack).await?,
                    Ok(None) => {}
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "mex notification refused locally; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                // APK43.72 picture handler19k returns to the base ACK;
                // preserve the already committed capture before acknowledging.
                match super::notifications::picture_ack(&node) {
                    Ok(Some(ack)) => link.send(&ack).await?,
                    Ok(None) => {}
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "picture notification refused locally; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                match super::notifications::business_ack(&node) {
                    Ok(Some(ack)) => link.send(&ack).await?,
                    Ok(None) => {}
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "business notification refused locally; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                match super::notifications::disappearing_mode_ack(&node) {
                    Ok(Some(ack)) => link.send(&ack).await?,
                    Ok(None) => {}
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "disappearing_mode notification refused locally; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                // Validate the control envelope before mutating membership. The
                // capture is already durable; commit the supported change before
                // writing ACK, and surface store failures without acknowledging.
                match super::groups::notification_ack(&node) {
                    Ok(Some(ack)) => {
                        super::groups::apply_change_notification(&store, &node)?;
                        link.send(&ack).await?;
                    }
                    Ok(None) => {}
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "group notification refused locally; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                // 2.26.37.71 PSANotificationHandler (17o) ACKs every psa
                // notification; without it the server ends the session with a
                // stream:error echoing the ACK (observed 2026-11-01).
                match super::notifications::psa_ack(&node) {
                    Ok(Some(ack)) => link.send(&ack).await?,
                    Ok(None) => {}
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "psa notification refused locally; captured without ACK; session continues");
                    }
                    Err(error) => return Err(error),
                }
                // The ACK is built first: a devices notification without `id` or
                // `from` cannot be acknowledged, so it is refused (logged, no
                // sync queued, no ACK) rather than ending receive at every login.
                let devices_change = super::devices::notification(&node).and_then(|change| {
                    match super::devices::notification_ack(&node) {
                        Ok(ack) => Some(Ok((change, ack))),
                        Err(Error::InvalidInput { field, reason }) => {
                            tracing::warn!(field, reason, "devices notification refused locally; captured without ACK; session continues");
                            None
                        }
                        Err(error) => Some(Err(error)),
                    }
                }).transpose()?;
                if let Some((change, devices_ack)) = devices_change {
                    // LX/11Q.A09 / LX/101.A02: contact notifications request a device
                    // sync. Locally persist work before ACK. Own-account changes use account sync, which is
                    // not implemented: captured and acknowledged only.
                    let own = account.own_jid.as_ref().map(|j| j.to_non_ad());
                    // Our own LID is ours too (devices.rs excludes it likewise);
                    // never queue a contact device sync for it. Read only for a
                    // LID owner, as inbox.rs does: a damaged own-LID record
                    // must not end receive on every PN devices notification.
                    let own_lid = if change.owner.server == crate::model::jid::Server::Lid {
                        super::group_send::own_lid(&store)?
                    } else {
                        None
                    };
                    if own.as_ref() != Some(&change.owner) && own_lid.as_ref() != Some(&change.owner) {
                        // Capture already committed above. Record pending work
                        // before ACK so a socket cut cannot silently lose it.
                        super::devices::request_sync(&store, own.as_ref(), &change.owner, change.device_hash.as_deref(), "notification")?;
                        device_context = super::devices::SyncContext::Notification;
                    }
                    link.send(&devices_ack).await?;
                    if device_query.is_none()
                        && group_outgoing.as_ref().is_none_or(|g| !g.devices_pending())
                        && let Some((query, sync)) = device_cursor.begin_next(&store, device_context)? {
                        device_query = Some(query);
                        link.send_device_query(&store, device_query.as_ref().expect("reserved device query"), &sync).await?;
                    }
                }

                let identity_ack = match super::identity_notifications::notification_ack(&node) {
                    Ok(ack) => ack,
                    Err(Error::InvalidInput { reason, .. }) => {
                        tracing::warn!(reason, "identity notification unsupported; captured without key read or ACK");
                        None
                    }
                    Err(error) => return Err(error),
                };
                if let Some(ack) = identity_ack {
                    let outcome = super::identity_notifications::queue(&store, &account, &node)?;
                    // 14z identity dispatch does not defer the base ACK. Commit
                    // capture/read-or-skip before write; bundle adoption is separate.
                    // A freshly received duplicate may repeat this idempotent ACK,
                    // never its queued/sent key query or any chat transmission.
                    link.send(&ack).await?;
                    if matches!(outcome, super::identity_notifications::Refresh::Queued(_)) {
                        tracing::info!("identity notification ACK written; key read queued or already saved");
                    } else {
                        tracing::info!(?outcome, "identity notification ACK written; key read skipped; capture preserved");
                    }
                }

                // APK43.72 15v.A09 parses PQ before A07(true). Its LIL
                // failure is caught by 0v6.A01, which dispatches the original
                // non-deferred ACK. No upload work runs for refused content.
                let low = match super::prekeys::low_count(&node) {
                    Ok(low) => low.map(Some),
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "prekey count content refused; capture retained; acknowledge without upload");
                        Some(None)
                    }
                    Err(error) => return Err(error),
                };
                let low = match low
                    .map(|low| Ok((low, super::prekeys::low_count_ack(&node)?)))
                    .transpose()
                {
                    Ok(low) => low,
                    Err(Error::InvalidInput { field, reason }) => {
                        tracing::warn!(field, reason, "prekey count envelope refused locally; captured without ACK or upload; session continues");
                        None
                    }
                    Err(error) => return Err(error),
                };
                if let Some((low, low_ack)) = low {
                    // Local ACK ordering follows durable capture; native 15v.A02 ACKs after worker processing.
                    // 43.72 15v.A02 default PQ-notification flagfalse:
                    // legacy presence chooses migrated ADD or non-migrated SET.
                    link.send(&low_ack).await?;
                    if low.is_some_and(|low| low.legacy_present) {
                        maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds, PrekeyUploadTrigger::Count { legacy: true }).await?;
                    }
                }
                super::outgoing::acknowledge(&store, &node)?;
                // A peer that could not decrypt our direct send asks for it
                // again (2.26.37.71 RetryReceiptHandler; retry_answer). The
                // answer is reserved before this write; refusals are recorded.
                let mut confirmed = false;
                if receipt_recorded {
                    match super::retry_answer::answer_with_confirmation(&store, &account, &node, epoch).await {
                        Ok(Some(answer)) => {
                            confirmed = link.send_retry_answer(&store, &answer).await?;
                            retry_refetches.record_written(&answer)?;
                            tracing::info!(target: "whatsapp_api::incoming", kind = %answer.node.description, "retry receipt answered (message), or its fresh-bundle query sent (iq)");
                        }
                        Ok(None) => {}
                        Err(Error::InvalidInput { field, reason }) => tracing::warn!(field, reason, "retry receipt not answered"),
                        Err(error) => return Err(error),
                    }
                }
                let retry_cause = match received {
                    super::inbox::IncomingResult::MissingSession => Some(super::receipts::RetryCause::MissingSession),
                    super::inbox::IncomingResult::InvalidSenderKeySignature => Some(super::receipts::RetryCause::SenderKeySignature),
                    super::inbox::IncomingResult::SenderKeyFutureCounter => Some(super::receipts::RetryCause::SenderKeyFutureCounter),
                    super::inbox::IncomingResult::MissingSenderKeyChain => Some(super::receipts::RetryCause::MissingSenderKeyChain),
                    super::inbox::IncomingResult::IncomingLocalPrekey => Some(super::receipts::RetryCause::IncomingLocalPrekey),
                    super::inbox::IncomingResult::UnknownSenderKeyVersion => Some(super::receipts::RetryCause::UnknownSenderKeyVersion),
                    _ => None,
                };
                if received == super::inbox::IncomingResult::Undecryptable || retry_cause.is_some() {
                    match super::receipts::queue_retry_for_incoming_with_cause(&store, &account, &node, retry_cause) {
                        Ok(0) => tracing::info!(target: "whatsapp_api::incoming", "retry already recorded; no repeat"),
                        Ok(_) => {
                            for receipt in super::receipts::reserve_pending_owned(&store)? {
                                link.send_receipt(&store,&receipt).await?;
                            }
                            tracing::info!(target: "whatsapp_api::incoming", "retry queued; reserved receipt batch written; delivery remains unconfirmed");
                        }
                        Err(Error::InvalidInput { field, reason }) => tracing::warn!(field, reason, "retry receipt not queued"),
                        Err(error) => return Err(error),
                    }
                }
                if ((node.description == "receipt" && receipt_recorded) || matches!(received, super::inbox::IncomingResult::Decrypted | super::inbox::IncomingResult::Duplicate))
                    && let Some(confirmation) = super::messaging::received_confirmation_for_account(&store, &account, &node)? {
                        // Raw capture and session/inbox commit precede this write.
                        // A reconnect may confirm a genuine server redelivery again.
                        link.send(&confirmation).await?;
                        confirmed = true;
                }
                // Opt-in: mark it read too. A refusal (an unsupported chat form,
                // say) is logged and skipped; it must not end the session.
                let mut marked_read = false;
                if config.transport.mark_read && matches!(received, super::inbox::IncomingResult::Decrypted | super::inbox::IncomingResult::Duplicate) {
                    match super::receipts::queue_read_for_incoming(&store, &node) {
                        Ok(0) => {}
                        Ok(_) => {
                            for receipt in super::receipts::reserve_pending_owned(&store)? { link.send_receipt(&store,&receipt).await?; }
                            marked_read = true;
                        }
                        Err(error) => tracing::warn!(%error, "automatic read receipt may not be queued"),
                    }
                }
                incoming_line(&node, received, confirmed, marked_read);
            }
        }
    }
}

/// One line per incoming stanza, so an operator can see what arrived and what
/// was done with it: element, `type`, which encryption, the inbox outcome, and
/// whether a delivery confirmation or read receipt went back. No identities,
/// no ids, no content. Keepalive replies are left out; there is one every 20 s.
fn incoming_line(
    node: &Node,
    received: super::inbox::IncomingResult,
    confirmed: bool,
    marked_read: bool,
) {
    if node.description == "iq" {
        return;
    }
    let chat = node
        .get_attr("from")
        .and_then(|v| v.as_jid())
        .map(|j| match j.server {
            crate::model::jid::Server::Group => "group",
            crate::model::jid::Server::Broadcast => "broadcast",
            crate::model::jid::Server::User | crate::model::jid::Server::Lid => "direct",
            _ => "other",
        })
        .unwrap_or("server");
    let enc: Vec<&str> = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "enc")
        .filter_map(|n| n.get_attr("type").and_then(|v| v.as_text()))
        .collect();
    tracing::info!(
        target: "whatsapp_api::incoming",
        stanza = %node.description,
        kind = node.get_attr("type").and_then(|v| v.as_text()).unwrap_or("-"),
        chat,
        enc = ?enc,
        outcome = ?received,
        confirmed,
        marked_read,
        "incoming"
    );
}

fn settle_upload_reply(
    store: &ProtocolStore,
    pending: super::prekeys::PendingUpload,
    node: &Node,
) -> Result<Option<super::prekeys::UploadOutcome>> {
    match pending.settle(store, node) {
        Ok(outcome) => Ok(Some(outcome)),
        Err(Error::InvalidInput {
            field: super::prekeys::DEADLINE_FIELD,
            ..
        }) => {
            tracing::info!("prekey reply expired during settlement; uncertainty retained");
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

/// Entry reason is preserved so count notifications and successful replies
/// cannot silently take the generic startup SET path.
#[derive(Clone, Copy)]
enum PrekeyUploadTrigger {
    Startup,
    Success { legacy: bool, pq: bool },
    Count { legacy: bool },
}

/// Reserve before write, with per-session bound and registered-material checks.
async fn maybe_upload(
    store: &Arc<ProtocolStore>,
    account: &AccountRecord,
    link: &NodeLink,
    upload: &mut Option<super::prekeys::PendingUpload>,
    rounds: &mut u32,
    trigger: PrekeyUploadTrigger,
) -> Result<()> {
    if upload.is_some() || *rounds >= super::prekeys::MAX_ROUNDS_PER_SESSION {
        return Ok(());
    }
    if account.signal_identity.is_none() || account.signal_registration_id.is_none() {
        return Ok(());
    }
    let signed =
        match store.read(crate::registration::factory::optional_registered_signed_prekey_in)? {
            Some(signed) => signed,
            None => {
                tracing::warn!("prekey upload skipped: registered signed prekey unavailable");
                return Ok(());
            }
        };
    if matches!(trigger, PrekeyUploadTrigger::Startup)
        && super::prekeys::unsent_ids(store)?.is_empty()
    {
        super::prekeys::generate(store)?;
    }
    let reserved = match trigger {
        PrekeyUploadTrigger::Startup => super::prekeys::begin_upload(store, account, &signed)?,
        PrekeyUploadTrigger::Success { legacy, pq } => {
            super::prekeys::begin_followup_upload(store, account, &signed, legacy, pq)?
        }
        PrekeyUploadTrigger::Count { legacy } => {
            super::prekeys::begin_notification_upload(store, account, &signed, legacy)?
        }
    };
    if let Some((pending, node)) = reserved {
        *rounds += 1;
        *upload = Some(pending.clone());
        link.send_prekey_upload(store, pending, &node).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId, MessageId},
            jid::Jid,
            receipt::{Receipt, ReceiptKind},
        },
        store::{MasterKey, OpenOptions, StaticKeySource},
        transport::TransportError,
    };

    fn open(path: &std::path::Path) -> Arc<ProtocolStore> {
        let store = ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        })
        .unwrap();
        store
            .write(|tx| {
                AccountRecord {
                    own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
                    registration: crate::session::RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)
            })
            .unwrap();
        Arc::new(store)
    }
    fn identity_store(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        })
        .unwrap()
    }

    #[test]
    fn invalid_login_material_is_refused_by_offline_preflight_without_reservation() {
        use crate::session::{KeyMaterial, NoiseStaticKey, RegistrationStatus};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preflight.redb");
        let store = identity_store(&path);
        let config = config();
        let original = AccountRecord {
            phone: Some(config.phone.clone()),
            registration: RegistrationStatus::Registered,
            noise_static: Some(NoiseStaticKey(KeyMaterial {
                public: [5; 32],
                private: [6; 32],
            })),
            credentials: vec![
                ObservedCredential {
                    name: "login".into(),
                    value: zeroize::Zeroizing::new(config.phone.protocol_digits().into_bytes()),
                    observed_at: Timestamp(0),
                    source_step: None,
                },
                ObservedCredential {
                    name: super::super::login::REGISTERED_APP_VERSION.into(),
                    value: zeroize::Zeroizing::new(b"private-malformed-build".to_vec()),
                    observed_at: Timestamp(0),
                    source_step: None,
                },
            ],
            ..Default::default()
        };
        store.write(|tx| original.save(tx)).unwrap();
        let before = store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap();
        let error = match login_material(&store, &config) {
            Ok(_) => panic!("expected preflight refusal"),
            Err(error) => error,
        };
        assert!(matches!(&error, Error::InvalidInput { field: "login", .. }));
        assert!(!error.to_string().contains("private-malformed-build"));
        assert!(last_login_status(&store).unwrap().is_none());
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap(),
            before
        );
        drop(store);
        let store = identity_store(&path);
        assert!(login_material(&store, &config).is_err());
        assert!(last_login_status(&store).unwrap().is_none());
    }

    #[test]
    fn inconsistent_noise_halves_refuse_preflight_without_repair_or_reservation() {
        use crate::session::{KeyMaterial, NoiseStaticKey, RegistrationStatus};
        use wacore_libsignal::core::curve::KeyPair;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("noise-halves.redb");
        let store = identity_store(&path);
        let config = config();
        // Independent registration key producer, not the Noise backend under test.
        let pair = KeyPair::generate(&mut rand::rng());
        let public: [u8; 32] = pair.public_key.public_key_bytes().try_into().unwrap();
        let private = *pair.private_key.serialize();
        let mut original = AccountRecord {
            phone: Some(config.phone.clone()),
            registration: RegistrationStatus::Registered,
            noise_static: Some(NoiseStaticKey(KeyMaterial { public, private })),
            credentials: vec![ObservedCredential {
                name: "login".into(),
                value: zeroize::Zeroizing::new(config.phone.protocol_digits().into_bytes()),
                observed_at: Timestamp(0),
                source_step: None,
            }],
            ..Default::default()
        };
        store.write(|tx| original.save(tx)).unwrap();
        assert!(login_material(&store, &config).is_ok());
        let journal = store.journal_head().unwrap();
        let mut changed_public = public;
        changed_public[10] ^= 1;
        let other = KeyPair::generate(&mut rand::rng());
        for (public, private) in [
            (changed_public, private),
            (public, *other.private_key.serialize()),
            ([0; 32], private),
        ] {
            original.noise_static = Some(NoiseStaticKey(KeyMaterial { public, private }));
            store.write(|tx| original.save(tx)).unwrap();
            let before = store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap();
            let error = match login_material(&store, &config) {
                Ok(_) => panic!("inconsistent pair passed preflight"),
                Err(error) => error,
            };
            assert!(
                error
                    .to_string()
                    .contains("saved Noise keypair is inconsistent")
            );
            assert!(!error.to_string().contains(&hex::encode(private)));
            assert!(!error.to_string().contains(&hex::encode(public)));
            assert!(last_login_status(&store).unwrap().is_none());
            assert_eq!(store.journal_head().unwrap(), journal);
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap(),
                before
            );
        }
        drop(store);
        let store = identity_store(&path);
        assert!(login_material(&store, &config).is_err());
        assert!(last_login_status(&store).unwrap().is_none());
    }

    #[cfg(feature = "experimental-signal-crypto")]
    #[test]
    fn registration_noise_copy_mismatch_refuses_login_before_reservation() {
        use crate::session::{KeyMaterial, NoiseStaticKey};
        use wacore_libsignal::core::curve::KeyPair;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("noise-copies.redb");
        let store = identity_store(&path);
        let config = config();
        crate::registration::factory::install_test_material(&store, &config).unwrap();
        store
            .write(|tx| {
                let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                account.credentials.push(ObservedCredential {
                    name: "login".into(),
                    value: zeroize::Zeroizing::new(config.phone.protocol_digits().into_bytes()),
                    observed_at: Timestamp(0),
                    source_step: None,
                });
                account.save(tx)
            })
            .unwrap();
        assert!(login_material(&store, &config).is_ok());
        // Both halves remain mathematically valid, but this is not the key
        // recorded as published by registration. Pair validation alone misses it.
        let other = KeyPair::generate(&mut rand::rng());
        store
            .write(|tx| {
                let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                account.noise_static = Some(NoiseStaticKey(KeyMaterial {
                    public: other.public_key.public_key_bytes().try_into().unwrap(),
                    private: *other.private_key.serialize(),
                }));
                account.save(tx)
            })
            .unwrap();
        let before = store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap();
        let journal = store.journal_head().unwrap();
        assert!(login_material(&store, &config).is_err());
        assert!(last_login_status(&store).unwrap().is_none());
        assert_eq!(store.journal_head().unwrap(), journal);
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
            before
        );
        drop(store);
        let store = identity_store(&path);
        assert!(login_material(&store, &config).is_err());
        assert!(last_login_status(&store).unwrap().is_none());
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
            before
        );
    }

    #[cfg(feature = "experimental-signal-crypto")]
    #[test]
    fn login_reservation_refuses_account_or_material_replacement_after_preflight() {
        for material_change in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("login-snapshot.redb");
            let store = identity_store(&path);
            let config = config();
            crate::registration::factory::install_test_material(&store, &config).unwrap();
            store
                .write(|tx| {
                    let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                    account.credentials.push(ObservedCredential {
                        name: "login".into(),
                        value: zeroize::Zeroizing::new(config.phone.protocol_digits().into_bytes()),
                        observed_at: Timestamp(0),
                        source_step: None,
                    });
                    account.save(tx)
                })
                .unwrap();
            let (_, _, snapshot) = login_material(&store, &config).unwrap();
            store
                .write(|tx| {
                    if material_change {
                        use crate::registration::factory::MATERIAL_KEY;
                        tx.delete(Table::Account, MATERIAL_KEY)?;
                        Ok(())
                    } else {
                        let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                        account.last_authenticated_at = Some(Timestamp(123));
                        account.save(tx)
                    }
                })
                .unwrap();
            let rows = store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap();
            let journal = store.journal_head().unwrap();
            assert!(
                reserve_login(&store, &snapshot)
                    .unwrap_err()
                    .to_string()
                    .contains("changed after preflight")
            );
            assert!(last_login_status(&store).unwrap().is_none());
            assert_eq!(store.journal_head().unwrap(), journal);
            drop(store);
            let store = identity_store(&path);
            assert!(reserve_login(&store, &snapshot).is_err());
            assert!(last_login_status(&store).unwrap().is_none());
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
                rows
            );
            // A fresh, valid snapshot still reserves once; deletion of optional
            // lab material does not become an unconditional import prohibition.
            let (_, _, current) = login_material(&store, &config).unwrap();
            reserve_login(&store, &current).unwrap();
            assert!(reserve_login(&store, &current).is_err());
        }
    }

    #[cfg(feature = "experimental-signal-crypto")]
    #[test]
    fn stale_login_reply_preserves_current_reservation_account_and_clock() {
        for case in [
            "account", "material", "owner", "settled", "missing", "valid",
        ] {
            for kind in ["success", "failure"] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("login-owner.redb");
                let store = identity_store(&path);
                let config = config();
                crate::registration::factory::install_test_material(&store, &config).unwrap();
                store
                    .write(|tx| {
                        let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                        account.credentials.push(ObservedCredential {
                            name: "login".into(),
                            value: zeroize::Zeroizing::new(
                                config.phone.protocol_digits().into_bytes(),
                            ),
                            observed_at: Timestamp(0),
                            source_step: None,
                        });
                        account.save(tx)
                    })
                    .unwrap();
                let (_, _, snapshot) = login_material(&store, &config).unwrap();
                let owner = reserve_login(&store, &snapshot).unwrap();
                store.write(|tx| {
                    match case {
                        "account" => {
                            let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                            account.last_authenticated_at = Some(Timestamp(123));
                            account.save(tx)?;
                        }
                        "material" => { tx.delete(Table::Account, crate::registration::factory::MATERIAL_KEY)?; }
                        "owner" | "settled" => tx.put_json(Table::Checkpoints, LOGIN_KEY, 1,
                            &serde_json::json!({"status":if case == "settled" {"refused"} else {"outcome_unknown"},
                                "reservation":"different-owner", "code":403}))?,
                        "missing" => { tx.delete(Table::Checkpoints, LOGIN_KEY)?; }
                        _ => {}
                    }
                    Ok(())
                }).unwrap();
                let account = store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap();
                let checkpoints = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                let journal = store.journal_head().unwrap();
                let reply = Node::new(kind)
                    .attr("reason", "403")
                    .attr("t", "1700000000");
                let result = record_login_reply_checked(&store, &reply, Some((&snapshot, &owner)));
                if case == "valid" {
                    if kind == "success" {
                        result.unwrap();
                    } else {
                        assert!(matches!(
                            result,
                            Err(Error::Authentication { code: Some(403) })
                        ));
                    }
                    let settled = store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap();
                    assert!(
                        record_login_reply_checked(&store, &reply, Some((&snapshot, &owner)))
                            .is_err()
                    );
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        settled
                    );
                } else {
                    assert!(matches!(
                        result,
                        Err(Error::InvalidInput {
                            field: "connection",
                            ..
                        })
                    ));
                    assert_eq!(store.journal_head().unwrap(), journal);
                    assert_eq!(
                        store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
                        account
                    );
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        checkpoints
                    );
                    drop(store);
                    let store = identity_store(&path);
                    assert!(
                        record_login_reply_checked(&store, &reply, Some((&snapshot, &owner)))
                            .is_err()
                    );
                    assert_eq!(
                        store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
                        account
                    );
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        checkpoints
                    );
                }
            }
        }
    }

    #[test]
    fn login_identity_partial_success_preserves_known_namespaces_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("login-identities.redb");
        let store = identity_store(&path);
        store.write(|tx| AccountRecord::default().save(tx)).unwrap();
        let pn = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let lid = Jid::parse("111111111111111@lid").unwrap();
        record_login_reply(
            &store,
            &Node::new("success")
                .attr("pn", pn.clone())
                .attr("lid", lid.clone()),
        )
        .unwrap();
        for reply in [
            Node::new("success").attr("pn", pn.clone()),
            Node::new("success").attr("lid", "111111111111111:7@lid"),
            Node::new("success"),
        ] {
            record_login_reply(&store, &reply).unwrap();
            assert_eq!(
                super::super::inbox::own_identities(&store).unwrap(),
                (Some(pn.clone()), Some(lid.clone()))
            );
            assert_eq!(
                super::super::group_send::own_lid(&store).unwrap(),
                Some(lid.clone())
            );
        }
        let saved = store
            .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, b"login-identity", 1))
            .unwrap();
        record_login_reply(&store, &Node::new("success")).unwrap();
        // A refusal keeps its numeric reason (403 banned, observed; 405 for a
        // retired client version in public clients) instead of dropping it.
        for (reason, code) in [("405", Some(405)), ("x", None)] {
            let error = record_login_reply(&store, &Node::new("failure").attr("reason", reason));
            assert!(matches!(error, Err(Error::Authentication { code: c }) if c == code));
        }
        record_login_reply(&store, &Node::new("success")).unwrap();
        assert_eq!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    b"login-identity",
                    1
                ))
                .unwrap(),
            saved
        );
        let head = store.journal_head().unwrap();
        drop(store);
        let store = identity_store(&path);
        assert_eq!(
            super::super::inbox::own_identities(&store).unwrap(),
            (Some(pn.clone()), Some(lid.clone()))
        );
        assert_eq!(
            super::super::group_send::own_lid(&store).unwrap(),
            Some(lid.clone())
        );
        assert_eq!(
            store.read(AccountRecord::load).unwrap().unwrap().own_jid,
            Some(pn)
        );
        assert_eq!(store.journal_head().unwrap(), head);
    }
    fn login_snapshot(store: &ProtocolStore) -> Vec<Vec<(Vec<u8>, Vec<u8>)>> {
        store
            .read(|tx| {
                [
                    Table::Account,
                    Table::Checkpoints,
                    Table::PeerIdentities,
                    Table::SignalSessions,
                    Table::Outbox,
                ]
                .into_iter()
                .map(|table| tx.scan_after(table, 1, None, 256))
                .collect()
            })
            .unwrap()
    }

    #[test]
    fn malformed_login_identity_updates_roll_back_result_clock_and_account() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid-login-identities.redb");
        let store = identity_store(&path);
        store.write(|tx| {
            AccountRecord { own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()), ..Default::default() }.save(tx)?;
            tx.put_json(Table::Checkpoints, b"login-identity", 1, &serde_json::json!({"pn":"15555550100@s.whatsapp.net","lid":"111111111111111@lid","at":1,"retained_metadata":"fixture"}))?;
            tx.put_json(Table::Checkpoints, LOGIN_KEY, 1, &serde_json::json!({"status":"outcome_unknown","at":1}))?;
            tx.put_json(Table::PeerIdentities, b"fixture", 1, &vec![1u8,2,3])?;
            tx.put_json(Table::SignalSessions, b"fixture", 1, &vec![4u8,5,6])
        }).unwrap();
        let before = login_snapshot(&store);
        let head = store.journal_head().unwrap();
        for (field, bad) in [
            ("pn", ""),
            ("pn", "s.whatsapp.net"),
            ("pn", "111111111111111@lid"),
            ("pn", "15555550100.1:0@s.whatsapp.net"),
            ("lid", ""),
            ("lid", "lid"),
            ("lid", "15555550100@s.whatsapp.net"),
            ("lid", "111111111111111.1:0@lid"),
        ] {
            let reply = Node::new("success")
                .attr(field, bad)
                .attr("t", "1791589303");
            let error = record_login_reply(&store, &reply).unwrap_err().to_string();
            assert!(error.contains(if field == "pn" {
                "success pn is not a known phone-number user"
            } else {
                "success lid is not a known LID user"
            }));
            assert!(!error.contains(bad) || bad.is_empty() || bad == "lid");
            assert_eq!(login_snapshot(&store), before);
            assert_eq!(store.journal_head().unwrap(), head);
        }
        let reply = Node::new("success")
            .attr(
                "lid",
                crate::binary::AttributeValue::Bytes(bytes::Bytes::from_static(b"\xff")),
            )
            .attr("t", "1791589303");
        assert!(record_login_reply(&store, &reply).is_err());
        assert_eq!(login_snapshot(&store), before);
        drop(store);
        let store = identity_store(&path);
        assert_eq!(login_snapshot(&store), before);
        assert_eq!(
            last_login_status(&store).unwrap().as_deref(),
            Some("outcome_unknown")
        );
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn malformed_saved_login_identity_is_not_repaired_by_a_new_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("malformed-login-history.redb");
        let store = identity_store(&path);
        store.write(|tx| AccountRecord::default().save(tx)).unwrap();
        for saved in [
            serde_json::json!([]),
            serde_json::json!(false),
            serde_json::json!({"pn":"s.whatsapp.net","lid":null}),
            serde_json::json!({"pn":null,"lid":7}),
            serde_json::json!({"pn":null,"lid":"111111111111111.1:0@lid"}),
        ] {
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, b"login-identity", 1, &saved)?;
                    tx.put_json(
                        Table::Checkpoints,
                        LOGIN_KEY,
                        1,
                        &serde_json::json!({"status":"outcome_unknown","at":1}),
                    )
                })
                .unwrap();
            let before = login_snapshot(&store);
            let reply = Node::new("success")
                .attr("pn", "15555550100@s.whatsapp.net")
                .attr("t", "1791589303");
            assert!(
                record_login_reply(&store, &reply)
                    .unwrap_err()
                    .to_string()
                    .contains("saved login-identity")
            );
            assert_eq!(login_snapshot(&store), before);
        }
    }
    fn config() -> LabRegistrationConfig {
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = zeroize::Zeroizing::new("offline-fixture".into());
        config
    }
    fn peer() -> Jid {
        Jid::parse("15555550101@s.whatsapp.net").unwrap()
    }

    #[tokio::test]
    async fn retry_ack_write_precedes_resend_and_failure_stops_the_pair() {
        for answer_kind in ["message", "iq"] {
            for outcome_kind in ["ack_failed", "resend_failed", "written"] {
                let (_incoming, incoming) = mpsc::channel(1);
                let (outgoing, mut writes) = mpsc::channel(2);
                let link = NodeLink::new(incoming, outgoing);
                let dir = tempfile::tempdir().unwrap();
                let store = Arc::new(open(&dir.path().join("retry-write.db")));
                let task = tokio::spawn(async move {
                    link.send_retry_answer(
                        &store,
                        &super::super::retry_answer::ReservedAnswer {
                            refetch: None,
                            confirmation: Some(
                                Node::new("ack")
                                    .attr("id", "ROOT")
                                    .attr("class", "receipt")
                                    .attr("type", "retry"),
                            ),
                            node: Node::new(answer_kind).attr("id", "W1"),
                        },
                    )
                    .await
                });
                let (first, outcome) = writes.recv().await.unwrap().prepare().unwrap();
                assert_eq!(first.description, "ack");
                assert_eq!(first.get_attr("id").and_then(|v| v.as_text()), Some("ROOT"));
                assert!(matches!(
                    writes.try_recv(),
                    Err(mpsc::error::TryRecvError::Empty)
                ));
                if outcome_kind != "ack_failed" {
                    outcome.send(Ok(())).unwrap();
                    let (second, outcome) = writes.recv().await.unwrap().prepare().unwrap();
                    assert_eq!(second.description, answer_kind);
                    assert_eq!(second.get_attr("id").and_then(|v| v.as_text()), Some("W1"));
                    if outcome_kind == "written" {
                        outcome.send(Ok(())).unwrap();
                        assert!(task.await.unwrap().unwrap());
                    } else {
                        outcome
                            .send(Err(Error::Transport(TransportError::Closed)))
                            .unwrap();
                        assert!(task.await.unwrap().is_err());
                        assert!(writes.recv().await.is_none());
                    }
                } else {
                    outcome
                        .send(Err(Error::Transport(TransportError::Closed)))
                        .unwrap();
                    assert!(task.await.unwrap().is_err());
                    assert!(writes.recv().await.is_none());
                }
            }
        }
    }

    /// The fake server side of one injected link.
    struct Fake {
        to_worker: mpsc::Sender<Result<Inbound>>,
        from_worker: mpsc::Receiver<Outbound>,
        stop: watch::Sender<bool>,
        ready: oneshot::Receiver<()>,
        task: tokio::task::JoinHandle<Result<()>>,
    }
    fn start(
        store: &Arc<ProtocolStore>,
        send_outbox: bool,
        ticket: Option<crate::model::ids::LocalMessageId>,
    ) -> Fake {
        start_with_config(store, send_outbox, ticket, config())
    }
    fn start_with_config(
        store: &Arc<ProtocolStore>,
        send_outbox: bool,
        ticket: Option<crate::model::ids::LocalMessageId>,
        config: LabRegistrationConfig,
    ) -> Fake {
        let (to_worker, incoming) = mpsc::channel(16);
        let (outgoing, from_worker) = mpsc::channel(16);
        let (stop, stop_rx) = watch::channel(false);
        let (ready_tx, ready) = oneshot::channel();
        let task = tokio::spawn(drive(
            Session {
                store: store.clone(),
                config,
                epoch: ConnectionEpoch(1),
                send_outbox,
                ticket,
            },
            stop_rx,
            ready_tx,
            NodeLink::new(incoming, outgoing),
        ));
        Fake {
            to_worker,
            from_worker,
            stop,
            ready,
            task,
        }
    }
    impl Fake {
        async fn written(&mut self) -> Written {
            self.written_within(Duration::from_secs(5)).await
        }
        async fn written_within(&mut self, window: Duration) -> Written {
            tokio::time::timeout(window, self.from_worker.recv())
                .await
                .expect("write within deadline")
                .expect("link open")
                .prepare()
                .expect("writer preflight accepted")
        }
        async fn nothing_written(&mut self, window: Duration) {
            match tokio::time::timeout(window, self.from_worker.recv()).await {
                Err(_) => {}
                Ok(None) => panic!("link closed while checking for unexpected writes"),
                Ok(Some(Outbound { node, .. })) => panic!(
                    "unexpected write: tag={} type={:?} xmlns={:?} children={:?}",
                    node.description,
                    node.get_attr("type").and_then(|v| v.as_text()),
                    node.get_attr("xmlns").and_then(|v| v.as_text()),
                    node.child_nodes()
                        .iter()
                        .map(|child| child.description.as_str())
                        .collect::<Vec<_>>()
                ),
            }
        }
        async fn activate(&mut self) -> String {
            let (active, reply) = self.written().await;
            assert_eq!(active.description, "iq");
            assert_eq!(
                active.get_attr("xmlns").and_then(|v| v.as_text()),
                Some("passive")
            );
            let id = active
                .get_attr("id")
                .and_then(|v| v.as_text())
                .unwrap()
                .to_owned();
            reply.send(Ok(())).unwrap();
            self.deliver(
                Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result"),
            )
            .await;
            // Availability is announced once activation is confirmed, so every
            // test that reasons about the next written stanza must account for
            // it here rather than each re-deriving the session's opening.
            let (presence, presence_reply) = self.written().await;
            assert_eq!(presence.description, "presence");
            assert_eq!(
                presence.get_attr("type").and_then(|v| v.as_text()),
                Some("available")
            );
            presence_reply.send(Ok(())).unwrap();
            id
        }
        async fn deliver(&self, node: Node) {
            self.to_worker.send(Ok(Inbound::Node(node))).await.unwrap();
        }
        /// A decrypted frame, decoded exactly as the real reader task does.
        async fn deliver_frame(&self, frame: &[u8]) {
            self.to_worker
                .send(Ok(inbound_from_frame(frame)))
                .await
                .unwrap();
        }
        async fn cut(&self, error: Error) {
            self.to_worker.send(Err(error)).await.unwrap();
        }
        async fn finished(self) -> Result<()> {
            self.finished_within(Duration::from_secs(5)).await
        }
        async fn finished_within(self, window: Duration) -> Result<()> {
            tokio::time::timeout(window, self.task)
                .await
                .expect("worker ends")
                .expect("worker did not panic")
        }
        async fn stop_cleanly(self) {
            self.stop.send(true).unwrap();
            assert!(self.finished().await.is_ok());
        }
    }
    #[tokio::test(start_paused = true)]
    async fn malformed_ack_stops_epoch_after_capture_and_keeps_reserved_action() {
        use sha2::Digest;
        for class in ["message", "receipt", "notification", "call"] {
            for raw in [
                Some(""),
                Some("2147483648"),
                None,
                Some("call-empty-id"),
                Some("call-missing-type"),
                Some("call-empty-type"),
                Some("call-group-sender"),
            ] {
                if class != "call" && raw.is_some_and(|raw| raw.starts_with("call-")) {
                    continue;
                }
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("ack-reader-error.redb");
                let store = open(&path);
                let action = super::super::actions::queue(&store, "fixture", "fixture", |id| {
                    Ok(Node::new("iq").attr("id", id).attr("type", "get"))
                })
                .unwrap();
                let mut fake = start(&store, false, None);
                fake.activate().await;
                (&mut fake.ready).await.unwrap();
                let (query, written) = fake.written().await;
                assert_eq!(
                    query.get_attr("id").and_then(|v| v.as_text()),
                    Some(action.as_str())
                );
                written.send(Ok(())).unwrap();
                let snapshot = |store: &ProtocolStore| {
                    store
                        .read(|tx| {
                            [
                                Table::Account,
                                Table::Outbox,
                                Table::SignalSessions,
                                Table::PeerIdentities,
                                Table::Checkpoints,
                            ]
                            .into_iter()
                            .map(|table| {
                                Ok(tx
                                    .scan(table, 1, 100)?
                                    .into_iter()
                                    .filter(|(key, _)| !key.starts_with(b"inbound:"))
                                    .collect::<Vec<_>>())
                            })
                            .collect::<Result<Vec<_>>>()
                        })
                        .unwrap()
                };
                let before = snapshot(&store);
                let bad = match raw {
                    Some("call-group-sender") => Node::new("ack")
                        .attr("class", "call")
                        .attr("id", "fixture")
                        .attr("type", "offer")
                        .attr("from", "123456-7@g.us"),
                    Some("call-empty-id") => Node::new("ack")
                        .attr("class", "call")
                        .attr("id", "")
                        .attr("type", "offer")
                        .attr("from", peer()),
                    Some("call-missing-type") => Node::new("ack")
                        .attr("class", "call")
                        .attr("id", "fixture")
                        .attr("from", peer()),
                    Some("call-empty-type") => Node::new("ack")
                        .attr("class", "call")
                        .attr("id", "fixture")
                        .attr("type", "")
                        .attr("from", peer()),
                    Some(raw) => Node::new("ack")
                        .attr("class", class)
                        .attr("id", "bad-ack-fixture")
                        .attr("error", raw),
                    None => Node::new("ack").attr("class", class).attr("error", "459"),
                };
                let capture = format!(
                    "inbound:sha256:{}",
                    hex::encode(sha2::Sha256::digest(encode(&bad).unwrap()))
                );
                fake.to_worker.try_send(Ok(Inbound::Node(bad))).unwrap();
                fake.to_worker
                    .try_send(Ok(Inbound::Node(
                        Node::new("iq")
                            .attr("type", "get")
                            .attr("xmlns", "urn:xmpp:ping"),
                    )))
                    .unwrap();
                let error = tokio::time::timeout(Duration::from_secs(1), &mut fake.task)
                    .await
                    .expect("malformed ACK ends promptly")
                    .unwrap()
                    .unwrap_err();
                assert!(matches!(error, Error::InvalidInput { field: "ack", .. }));
                assert!(!error.is_recoverable_by_reconnect());
                assert!(
                    fake.from_worker.recv().await.is_none(),
                    "no pong or later write"
                );
                assert!(
                    store
                        .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                        .unwrap()
                        .is_some()
                );
                assert_eq!(snapshot(&store), before);
                drop(fake);
                drop(store);
                let reopened = identity_store(&path);
                assert_eq!(snapshot(&reopened), before);
                assert!(
                    super::super::actions::reserve_pending(&reopened)
                        .unwrap()
                        .is_empty()
                );
                assert!(
                    reopened
                        .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                        .unwrap()
                        .is_some()
                );
            }
        }
    }

    #[tokio::test]
    async fn call_ack_known_null_sender_is_captured_without_stopping_receive() {
        use sha2::Digest;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("call-ack-null-sender.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        for (index, sender) in [None, Some("")].into_iter().enumerate() {
            for (kind_index, kind) in [None, Some("")].into_iter().enumerate() {
                let mut call = Node::new("ack").attr("class", "call").attr("id", "");
                if let Some(sender) = sender {
                    call = call.attr("from", sender);
                }
                if let Some(kind) = kind {
                    call = call.attr("type", kind);
                }
                let capture = format!(
                    "inbound:sha256:{}",
                    hex::encode(sha2::Sha256::digest(encode(&call).unwrap()))
                );
                fake.deliver(call).await;
                let barrier = format!("AFTER-NULL-CALL-{index}-{kind_index}");
                fake.deliver(
                    Node::new("receipt")
                        .attr("from", peer())
                        .attr("id", barrier.clone()),
                )
                .await;
                let (ack, written) = fake.written().await;
                assert_eq!(ack.description, "ack");
                assert_eq!(
                    ack.get_attr("id").and_then(|v| v.as_text()),
                    Some(barrier.as_str())
                );
                written.send(Ok(())).unwrap();
                assert!(
                    store
                        .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                        .unwrap()
                        .is_some()
                );
                fake.nothing_written(Duration::from_millis(25)).await;
            }
        }
        fake.stop_cleanly().await;
        assert!(super::super::receipts::outcomes(&store).unwrap().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn about_result_extensions_do_not_become_refusals_in_receive() {
        use super::super::actions::{self, ActionStatus};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("about-reply.redb"));
        let id = super::super::about::queue_set(&store, "Available").unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let (query, written) = fake.written().await;
        assert_eq!(
            query.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        written.send(Ok(())).unwrap();
        fake.deliver(
            Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("error").attr("code", "403")]),
        )
        .await;
        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "AFTER-ABOUT-RESULT"),
        )
        .await;
        let (ack, written) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("AFTER-ABOUT-RESULT")
        );
        written.send(Ok(())).unwrap();
        let outcomes = actions::outcomes(&store).unwrap();
        let outcome = outcomes.iter().find(|a| a.id == id).unwrap();
        assert_eq!(outcome.status, ActionStatus::Ok);
        assert_eq!(outcome.error_code, None);
        fake.nothing_written(Duration::from_millis(1)).await;
        fake.stop_cleanly().await;
    }

    #[tokio::test(start_paused = true)]
    async fn profile_upload_result_extensions_do_not_become_refusals_in_receive() {
        use super::super::actions::{self, ActionStatus};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("photo-reply.redb"));
        let id = super::super::profile_picture::queue_set(
            &store,
            vec![0xff, 0xd8, 0xff, 0xe0, 1, 2, 0xff, 0xd9],
        )
        .unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let (query, written) = fake.written().await;
        assert_eq!(
            query.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        written.send(Ok(())).unwrap();
        fake.deliver(
            Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![
                    Node::new("picture").attr("id", "7"),
                    Node::new("error").attr("code", "403"),
                ]),
        )
        .await;
        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "AFTER-PHOTO-RESULT"),
        )
        .await;
        let (ack, written) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("AFTER-PHOTO-RESULT")
        );
        written.send(Ok(())).unwrap();
        let outcomes = actions::outcomes(&store).unwrap();
        let outcome = outcomes.iter().find(|a| a.id == id).unwrap();
        assert_eq!(outcome.status, ActionStatus::Ok);
        assert_eq!(outcome.error_code, None);
        fake.nothing_written(Duration::from_millis(1)).await;
        fake.stop_cleanly().await;
    }

    #[tokio::test(start_paused = true)]
    async fn expired_token_preflight_writes_no_token_and_keeps_receive_active() {
        use super::super::{actions, token_issue};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("expired-token-preflight.redb"));
        let old = Timestamp::now().as_millis() / 1000 - token_issue::FALLBACK_SECONDS - 60;
        let id = store
            .write(|tx| token_issue::schedule_after_ack_in(tx, &peer(), old))
            .unwrap()
            .unwrap();
        let control = actions::queue(&store, "control", "control", |id| {
            Ok(Node::new("iq")
                .attr("id", id)
                .attr("type", "get")
                .attr("xmlns", "fixture"))
        })
        .unwrap();
        let mut cfg = config();
        cfg.transport.action_delay_ms = 1;
        let mut fake = start_with_config(&store, false, None, cfg);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let (node, written) = fake.written().await;
        assert_eq!(
            node.get_attr("id").and_then(|v| v.as_text()),
            Some(control.as_str())
        );
        written.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(2)).await;
        let outcomes = actions::outcomes(&store).unwrap();
        let held = outcomes.iter().find(|a| a.id == id).unwrap();
        assert_eq!(held.status, actions::ActionStatus::Queued);
        assert_eq!(
            held.deferred_reason,
            Some(actions::ActionDeferReason::PrivacyTokenExpired)
        );
        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "AFTER-TOKEN-HOLD"),
        )
        .await;
        let (ack, written) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("AFTER-TOKEN-HOLD")
        );
        written.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
    }

    #[tokio::test(start_paused = true)]
    async fn timed_action_expiry_preserves_uncertainty_without_stopping_receive() {
        use super::super::actions::{self, ActionStatus};
        for kind in [
            "abprops",
            "block",
            "unblock",
            "dm-default",
            "two-factor",
            "privacy-token",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("abprops-expiry.redb"));
            let id = match kind {
                "abprops" => super::super::abprops::queue(&store),
                "dm-default" => super::super::chat_settings::queue_default_timer(&store, 86400),
                "privacy-token" => store
                    .write(|tx| {
                        super::super::token_issue::schedule_after_ack_in(tx, &peer(), 1_800_000_000)
                    })
                    .map(|id| id.expect("fresh token fixture")),
                "two-factor" => super::super::two_factor::queue_set(
                    &store,
                    &super::super::two_factor::TwoFactorChange {
                        code: Some("928374"),
                        email: None,
                    },
                ),
                "block" => super::super::blocklist::queue_block(&store, &peer()),
                _ => super::super::blocklist::queue_unblock(&store, &peer()),
            }
            .unwrap();
            let mut fake = start(&store, false, None);
            fake.activate().await;
            (&mut fake.ready).await.unwrap();
            let (query, written) = fake.written().await;
            assert_eq!(
                query.get_attr("id").and_then(|v| v.as_text()),
                Some(id.as_str())
            );
            // Include time spent waiting for the local write; completing it must
            // not start a fresh callback budget. No delivery assertion follows expiry.
            tokio::time::advance(Duration::from_secs(33)).await;
            written.send(Ok(())).unwrap();
            let (ping, written) = fake.written().await;
            assert_eq!(
                ping.get_attr("xmlns").and_then(|v| v.as_text()),
                Some("w:p")
            );
            written.send(Ok(())).unwrap();
            fake.deliver(valid_keepalive_reply(
                ping.get_attr("id").and_then(|v| v.as_text()).unwrap(),
            ))
            .await;
            // Let the due expiry branch run before presenting the late reply.
            fake.nothing_written(Duration::from_millis(1)).await;
            fake.deliver(
                Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result"),
            )
            .await;
            fake.deliver(
                Node::new("receipt")
                    .attr("from", peer())
                    .attr("id", "AFTER-ABPROPS-EXPIRY"),
            )
            .await;
            let (ack, written) = fake.written().await;
            assert_eq!(ack.description, "ack");
            assert_eq!(
                ack.get_attr("id").and_then(|v| v.as_text()),
                Some("AFTER-ABPROPS-EXPIRY")
            );
            written.send(Ok(())).unwrap();
            assert_eq!(
                actions::outcomes(&store)
                    .unwrap()
                    .iter()
                    .find(|a| a.id == id)
                    .unwrap()
                    .status,
                ActionStatus::Sent
            );
            fake.nothing_written(Duration::from_millis(1)).await;
            assert!(super::super::blocklist::blocked(&store).unwrap().is_empty());
            fake.stop_cleanly().await;
        }
    }

    #[tokio::test]
    async fn paced_actions_keep_receive_and_stop_responsive_and_reserve_only_one() {
        use super::super::actions::{self, ActionStatus};
        for fail in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("paced-actions.redb"));
            for _ in 0..17 {
                actions::queue(&store, "pace-fixture", "fixture", |id| {
                    Ok(Node::new("iq").attr("id", id).attr("type", "get"))
                })
                .unwrap();
            }
            let mut config = config();
            config.transport.action_delay_ms = 300_000;
            let mut fake = start_with_config(&store, false, None, config.clone());
            fake.activate().await;
            (&mut fake.ready).await.unwrap();
            let (first, written) = fake.written().await;
            let first_id = first
                .get_attr("id")
                .and_then(|v| v.as_text())
                .unwrap()
                .to_owned();
            let outcomes = actions::outcomes(&store).unwrap();
            assert_eq!(
                outcomes
                    .iter()
                    .filter(|a| a.status == ActionStatus::Sent)
                    .count(),
                1
            );
            assert_eq!(
                outcomes
                    .iter()
                    .filter(|a| a.status == ActionStatus::Queued)
                    .count(),
                16
            );
            if fail {
                written
                    .send(Err(Error::Transport(TransportError::PeerClosed)))
                    .unwrap();
                assert!(fake.finished().await.is_err());
                let mut next = start_with_config(&store, false, None, config);
                next.activate().await;
                let (second, written) = next.written().await;
                assert_ne!(
                    second.get_attr("id").and_then(|v| v.as_text()),
                    Some(first_id.as_str())
                );
                let second_id = second
                    .get_attr("id")
                    .and_then(|v| v.as_text())
                    .unwrap()
                    .to_owned();
                written.send(Ok(())).unwrap();
                next.deliver(
                    Node::new("iq")
                        .attr("id", first_id.as_str())
                        .attr("type", "result"),
                )
                .await;
                next.deliver(
                    Node::new("receipt")
                        .attr("from", peer())
                        .attr("id", "AFTER-STALE-ACTION"),
                )
                .await;
                let (ack, reply) = next.written().await;
                assert_eq!(ack.description, "ack");
                assert_eq!(
                    ack.get_attr("id").and_then(|v| v.as_text()),
                    Some("AFTER-STALE-ACTION")
                );
                reply.send(Ok(())).unwrap();
                assert_eq!(
                    actions::outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == first_id)
                        .unwrap()
                        .status,
                    ActionStatus::Sent
                );
                next.deliver(
                    Node::new("iq")
                        .attr("id", second_id.as_str())
                        .attr("type", "result"),
                )
                .await;
                next.deliver(
                    Node::new("receipt")
                        .attr("from", peer())
                        .attr("id", "AFTER-LIVE-ACTION"),
                )
                .await;
                let (ack, reply) = next.written().await;
                assert_eq!(
                    ack.get_attr("id").and_then(|v| v.as_text()),
                    Some("AFTER-LIVE-ACTION")
                );
                reply.send(Ok(())).unwrap();
                assert_eq!(
                    actions::outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == second_id)
                        .unwrap()
                        .status,
                    ActionStatus::Ok
                );
                next.stop_cleanly().await;
            } else {
                fake.deliver(
                    Node::new("iq")
                        .attr("id", first_id.as_str())
                        .attr("type", "result"),
                )
                .await;
                // Reply was queued while the local write awaited completion.
                // Register ownership before the loop processes that fast reply.
                written.send(Ok(())).unwrap();
                tokio::time::timeout(Duration::from_secs(1), async {
                    while actions::outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == first_id)
                        .unwrap()
                        .status
                        != ActionStatus::Ok
                    {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                assert_eq!(
                    actions::outcomes(&store)
                        .unwrap()
                        .iter()
                        .filter(|a| a.status == ActionStatus::Queued)
                        .count(),
                    16
                );
                fake.stop.send(true).unwrap();
                assert!(fake.finished_within(Duration::from_secs(1)).await.is_ok());
            }
        }
    }

    #[test]
    fn action_drain_triggers_do_not_stack_or_reset_an_active_pacing_deadline() {
        let mut drain = ActionDrain::default();
        drain.trigger();
        drain.sent(300_000);
        let due = drain.deadline();
        drain.trigger();
        assert_eq!(drain.remaining, super::super::actions::BATCH - 1);
        assert_eq!(drain.deadline(), due);
        for _ in 1..super::super::actions::BATCH {
            drain.sent(0);
        }
        assert_eq!(drain.remaining, 0);
        assert!(drain.due.is_none());
        drain.trigger();
        assert_eq!(drain.remaining, super::super::actions::BATCH);
        drain.finish();
        assert_eq!(drain.remaining, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn zero_delay_drain_keeps_the_sixteen_action_trigger_budget() {
        use super::super::actions::{self, ActionStatus};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("zero-delay-actions.redb"));
        for _ in 0..17 {
            actions::queue(&store, "pace-fixture", "fixture", |id| {
                Ok(Node::new("iq").attr("id", id).attr("type", "get"))
            })
            .unwrap();
        }
        let mut config = config();
        config.transport.action_delay_ms = 0;
        config.transport.timeout_ms = 40_000;
        let mut fake = start_with_config(&store, false, None, config);
        fake.activate().await;
        for _ in 0..16 {
            let (_, written) = fake.written().await;
            written.send(Ok(())).unwrap();
        }
        fake.nothing_written(Duration::from_millis(5)).await;
        assert_eq!(
            actions::outcomes(&store)
                .unwrap()
                .iter()
                .filter(|a| a.status == ActionStatus::Sent)
                .count(),
            16
        );
        assert_eq!(
            actions::outcomes(&store)
                .unwrap()
                .iter()
                .filter(|a| a.status == ActionStatus::Queued)
                .count(),
            1
        );
        tokio::time::advance(Duration::from_secs(20)).await;
        let (ping, written) = fake.written().await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        written.send(Ok(())).unwrap();
        let (_, written) = fake.written().await;
        written.send(Ok(())).unwrap();
        assert!(
            actions::outcomes(&store)
                .unwrap()
                .iter()
                .all(|a| a.status == ActionStatus::Sent)
        );
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn account_privacy_commits_before_ack_and_failed_write_survives_reopen() {
        use super::super::account_privacy::{ReadReceiptValue, read_receipts};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("privacy-cut.redb");
        let store = open(&path);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let notification = Node::new("notification")
            .attr("type", "account_sync")
            .attr("from", "s.whatsapp.net")
            .attr("id", "PRIVACY-CUT")
            .attr("t", "1700000000")
            .children(vec![Node::new("privacy").children(vec![
                Node::new("category").attr("name", "readreceipts").attr("value", "none")
            ])]);
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&notification).unwrap()))
        );
        fake.deliver(notification.clone()).await;
        let (ack, written) = fake.written().await;
        assert_eq!(
            ack,
            Node::new("ack")
                .attr("id", "PRIVACY-CUT")
                .attr("to", Jid::parse("s.whatsapp.net").unwrap())
                .attr("class", "notification")
                .attr("type", "account_sync")
        );
        assert_eq!(
            read_receipts(&store).unwrap().unwrap().value,
            ReadReceiptValue::None
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        let account = store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap();
        let journal = store.journal_head().unwrap();
        written
            .send(Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "privacy ACK write cut",
            ))))
            .unwrap();
        assert!(matches!(fake.finished().await, Err(Error::Io(_))));
        drop(store);
        let store = Arc::new(identity_store(&path));
        assert_eq!(
            read_receipts(&store).unwrap().unwrap().value,
            ReadReceiptValue::None
        );
        let before = store
            .read(|tx| tx.get(Table::Checkpoints, b"account-privacy:readreceipts", 1))
            .unwrap();
        let mut fresh = start(&store, false, None);
        fresh.activate().await;
        (&mut fresh.ready).await.unwrap();
        fresh.nothing_written(Duration::from_millis(30)).await;
        fresh.deliver(notification).await;
        let (again, written) = fresh.written().await;
        assert_eq!(again, ack);
        assert_eq!(
            store
                .read(|tx| tx.get(Table::Checkpoints, b"account-privacy:readreceipts", 1))
                .unwrap(),
            before
        );
        written.send(Ok(())).unwrap();
        fresh.stop_cleanly().await;
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap(),
            account
        );
        assert_eq!(store.journal_head().unwrap(), journal);
    }

    #[tokio::test]
    async fn unsupported_account_privacy_is_captured_without_ack_or_session_loss() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("privacy-unsupported.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        for child in [
            Node::new("privacy").attr("action", "modify"),
            Node::new("privacy").children(vec![
                Node::new("category")
                    .attr("name", "readreceipts")
                    .attr("value", "all"),
                Node::new("category")
                    .attr("name", "profile")
                    .attr("value", "none"),
            ]),
            Node::new("devices"),
        ] {
            let notification = Node::new("notification")
                .attr("type", "account_sync")
                .attr("from", "s.whatsapp.net")
                .attr("id", "UNSUPPORTED")
                .attr("t", "1700000000")
                .children(vec![child]);
            let capture = format!(
                "inbound:sha256:{}",
                hex::encode(sha2::Sha256::digest(encode(&notification).unwrap()))
            );
            fake.deliver(notification).await;
            fake.nothing_written(Duration::from_millis(30)).await;
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            assert!(
                super::super::account_privacy::read_receipts(&store)
                    .unwrap()
                    .is_none()
            );
        }
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn refused_group_ack_envelope_cannot_change_membership() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("group-envelope.redb");
        let store = open(&path);
        let key = super::super::groups::key("123456-789@g.us");
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key.as_bytes(),
                    1,
                    &serde_json::json!({"jid":"123456-789@g.us", "updated_at":0,
                "members":[{"jid":"15551234567@s.whatsapp.net","role":null}]}),
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let members = vec![Node::new("participant").attr("jid", "15551234567@s.whatsapp.net")];
        let notification = Node::new("notification")
            .attr("type", "w:gp2")
            .attr("from", "123456-789@g.us")
            .children(vec![Node::new("remove").children(members)]);
        for invalid in [
            notification.clone(),
            notification.clone().attr("id", ""),
            notification
                .clone()
                .attr("id", "BAD-PARTICIPANT")
                .attr("participant", ""),
        ] {
            fake.deliver(invalid).await;
            // Processing barrier: refusal must leave receive running without ACK.
            fake.deliver(
                Node::new("iq")
                    .attr("type", "get")
                    .attr("xmlns", "urn:xmpp:ping"),
            )
            .await;
            let (pong, written) = fake.written().await;
            assert_eq!(pong.description, "iq");
            written.send(Ok(())).unwrap();
            assert_eq!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                    .unwrap(),
                before,
                "unusable ACK envelope cannot mutate membership"
            );
        }
        fake.deliver(notification.attr("id", "VALID")).await;
        let (ack, written) = fake.written().await;
        assert_eq!(ack.description, "ack");
        let snapshot: super::super::groups::GroupSnapshot = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert!(
            snapshot.members.is_empty(),
            "valid change commits before ACK write"
        );
        written.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
        drop(store);
        let reopened = open(&path);
        let snapshot: super::super::groups::GroupSnapshot = reopened
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert!(snapshot.members.is_empty());
    }

    #[tokio::test]
    async fn corrupt_group_snapshot_is_not_acknowledged_as_applied() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt-group-notification.redb");
        let store = open(&path);
        let key = super::super::groups::key("123456-789@g.us");
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key.as_bytes(),
                    1,
                    &serde_json::json!({"jid":"123456-789@g.us","members":"corrupt"}),
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let participants = vec![Node::new("participant").attr("jid", "15551234567@s.whatsapp.net")];
        let notification = Node::new("notification")
            .attr("type", "w:gp2")
            .attr("from", "123456-789@g.us")
            .attr("id", "GROUP-CHANGE")
            .children(vec![Node::new("add").children(participants)]);
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&notification).unwrap()))
        );
        fake.deliver(notification).await;
        let outbound = tokio::time::timeout(Duration::from_secs(5), fake.from_worker.recv())
            .await
            .unwrap();
        assert!(
            outbound.is_none(),
            "stored snapshot failure must not produce a success ACK"
        );
        assert!(matches!(fake.finished().await, Err(Error::Storage(_))));
        assert_eq!(
            store
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap(),
            before
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        drop(store);
        let reopened = open(&path);
        assert_eq!(
            reopened
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap(),
            before
        );
        assert!(
            reopened
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn picture_notification_ack_follows_capture_and_survives_redelivery() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("picture-ack.redb");
        let store = open(&path);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let children = vec![
            Node::new("set")
                .attr("jid", "123456-789@g.us")
                .attr("id", "7"),
        ];
        let notification = Node::new("notification")
            .attr("type", "picture")
            .attr("from", "123456-789@g.us")
            .attr("participant", "15551234567@s.whatsapp.net")
            .attr("id", "PICTURE")
            // F0R.A02/A00 does not copy these incoming attributes into Efa.
            .attr("recipient", "15557654321@s.whatsapp.net")
            .attr("edit", "7")
            .attr("error", "500")
            .attr("offline", "1")
            .children(children);
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&notification).unwrap()))
        );
        let expected = Node::new("ack")
            .attr("id", "PICTURE")
            .attr("to", Jid::parse("123456-789@g.us").unwrap())
            .attr("class", "notification")
            .attr("type", "picture")
            .attr(
                "participant",
                Jid::parse("15551234567@s.whatsapp.net").unwrap(),
            );
        for _ in 0..2 {
            fake.deliver(notification.clone()).await;
            let (ack, written) = fake.written().await;
            assert_eq!(ack, expected);
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            written.send(Ok(())).unwrap();
        }
        // APK43.72 123.A02/A04 reconstruct zero-device PN/LID as users;
        // 1r2.A00 writes pairs for zero getters and AD for positive ones.
        for (server, agent, device, expected_peer) in [
            (
                crate::model::jid::Server::User,
                0,
                0,
                Jid::parse("15551234567@s.whatsapp.net").unwrap(),
            ),
            (
                crate::model::jid::Server::Lid,
                0,
                0,
                Jid::parse("15551234567@lid").unwrap(),
            ),
            (
                crate::model::jid::Server::User,
                0,
                2,
                Jid::parse("15551234567:2@s.whatsapp.net").unwrap(),
            ),
            (
                crate::model::jid::Server::Lid,
                0,
                2,
                Jid::parse("15551234567:2@lid").unwrap(),
            ),
        ] {
            let incoming_peer = Jid {
                user: "15551234567".into(),
                server,
                agent,
                device,
                kind: crate::model::jid::JidKind::AgentDevice,
            };
            let incoming = notification
                .clone()
                .attr("from", incoming_peer.clone())
                .attr("participant", incoming_peer);
            fake.deliver(incoming).await;
            let (ack, written) = fake.written().await;
            let expected = Node::new("ack")
                .attr("id", "PICTURE")
                .attr("to", expected_peer.clone())
                .attr("class", "notification")
                .attr("type", "picture")
                .attr("participant", expected_peer);
            assert_eq!(ack, expected);
            written.send(Ok(())).unwrap();
        }
        // An unusable ACK envelope must not terminate receive or fabricate a peer.
        fake.deliver(
            Node::new("notification")
                .attr("type", "picture")
                .attr("id", "BAD"),
        )
        .await;
        fake.deliver(
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        )
        .await;
        let (pong, written) = fake.written().await;
        assert_eq!(pong.description, "iq");
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        written.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(30)).await;
        fake.stop_cleanly().await;
        drop(store);
        let reopened = open(&path);
        assert!(
            reopened
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn native_server_ping_is_answered_before_activation_and_after_capture() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server-ping.redb");
        let store = open(&path);
        let journal = store.journal_head().unwrap();
        let account = store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap();
        let mut fake = start(&store, false, None);
        let (active, written) = fake.written().await;
        let active_id = active.get_attr("id").unwrap().clone();
        written.send(Ok(())).unwrap();
        let ping = Node::new("iq")
            .attr("type", "get")
            .attr("xmlns", "urn:xmpp:ping")
            .attr("id", "DO-NOT-ECHO")
            .attr("from", peer())
            .attr("t", "1700000000");
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&ping).unwrap()))
        );
        let expected = Node::new("iq")
            .attr("type", "result")
            .attr("to", Jid::parse("s.whatsapp.net").unwrap());
        for incoming in [
            ping.clone(),
            ping.clone(),
            // Native routing forwards timestamp default 0 but still pongs:
            // neither id nor a parsed positive t is a response prerequisite.
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping")
                .attr("t", "not-a-time"),
        ] {
            fake.deliver(incoming).await;
            let (pong, written) = fake.written().await;
            assert_eq!(pong, expected);
            assert!(
                fake.ready.try_recv().is_err(),
                "pong does not activate a connection"
            );
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                super::super::clock::inspect(&store)
                    .unwrap()
                    .confirmed_source,
                Some(super::super::clock::ConfirmedSource::ServerPing)
            );
            written.send(Ok(())).unwrap();
        }
        fake.deliver(Node::new("iq").attr("id", active_id).attr("type", "result"))
            .await;
        let (presence, written) = fake.written().await;
        assert_eq!(presence.description, "presence");
        written.send(Ok(())).unwrap();
        (&mut fake.ready).await.unwrap();
        fake.stop_cleanly().await;
        assert_eq!(store.journal_head().unwrap(), journal);
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap(),
            account
        );
        drop(store);
        let store = identity_store(&path);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        assert_eq!(
            super::super::clock::inspect(&store)
                .unwrap()
                .confirmed_source,
            Some(super::super::clock::ConfirmedSource::ServerPing)
        );
    }

    #[tokio::test]
    async fn failed_server_pong_write_keeps_capture_and_is_not_queued_for_reconnect() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server-pong-cut.redb");
        let store = open(&path);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        let ping = Node::new("iq")
            .attr("type", "get")
            .attr("xmlns", "urn:xmpp:ping")
            .attr("id", "PONG-CUT")
            .attr("t", "1700000000");
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&ping).unwrap()))
        );
        fake.deliver(ping).await;
        let (pong, written) = fake.written().await;
        assert_eq!(pong.description, "iq");
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        written
            .send(Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "pong write cut",
            ))))
            .unwrap();
        assert!(matches!(fake.finished().await, Err(Error::Io(_))));
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        drop(store);
        let store = Arc::new(identity_store(&path));
        let mut fresh = start(&store, false, None);
        fresh.activate().await;
        (&mut fresh.ready).await.unwrap();
        // Historical captures are evidence, not a queue of protocol pongs.
        fresh.nothing_written(Duration::from_millis(30)).await;
        fresh.stop_cleanly().await;
    }

    #[tokio::test]
    async fn server_pong_does_not_answer_other_iq_types_or_namespaces() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("no-server-pong.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        for node in [
            Node::new("iq").attr("type", "get").attr("xmlns", "w:p"),
            Node::new("iq")
                .attr("type", "result")
                .attr("xmlns", "urn:xmpp:ping"),
            Node::new("iq")
                .attr("type", "set")
                .attr("xmlns", "urn:xmpp:ping"),
            Node::new("iq").attr("xmlns", "urn:xmpp:ping"),
            Node::new("iq").attr("type", "get"),
            Node::new("message")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        ] {
            fake.deliver(node).await;
            fake.nothing_written(Duration::from_millis(20)).await;
            assert!(!fake.task.is_finished());
        }
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn dirty_notification_ack_cut_preserves_one_query_for_next_session() {
        use super::super::{actions, groups};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("dirty-session.redb"));
        let notification = Node::new("notification")
            .attr("from", "s.whatsapp.net")
            .attr("id", "DIRTY-CUT")
            .attr("type", "w:gp2")
            .children(vec![Node::new("groups_dirty").children(vec![
                Node::new("group").attr("jid", "12345-67890@g.us"),
            ])]);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(notification.clone()).await;
        let (ack, written) = fake.written().await;
        assert_eq!(ack.description, "ack");
        let outcomes = actions::outcomes(&store).unwrap();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].status, actions::ActionStatus::Queued);
        let id = outcomes[0].id.clone();
        // Queue commit precedes the ACK write, which fails ambiguously here.
        written
            .send(Err(Error::Transport(TransportError::PeerClosed)))
            .unwrap();
        assert!(fake.finished().await.is_err());
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (query, written) = fake.written().await;
        assert_eq!(
            query.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        written.send(Ok(())).unwrap();
        fake.deliver(notification).await;
        let (repeat_ack, written) = fake.written().await;
        assert_eq!(repeat_ack, ack);
        written.send(Ok(())).unwrap();
        assert_eq!(actions::outcomes(&store).unwrap().len(), 1);
        fake.deliver(
            Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("groups").children(vec![Node::new("group")
                .attr("id", "12345-67890@g.us").attr("subject", "Recovered")])]),
        )
        .await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while groups::refresh_outcome(&store, &id).unwrap().is_none() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            groups::refresh_outcome(&store, &id)
                .unwrap()
                .unwrap()
                .complete,
            1
        );
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn settled_group_reply_updates_snapshot_during_receive_without_cli_ingestion() {
        use super::super::{actions, groups};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("group-session.redb"));
        let group = Jid::parse("12345-67890@g.us").unwrap();
        let id = groups::queue_info(&store, &group).unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (query, written) = fake.written().await;
        assert_eq!(
            query.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        written.send(Ok(())).unwrap();
        fake.deliver(
            Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![
                    Node::new("group")
                        .attr("id", group.clone())
                        .attr("subject", "Received in session")
                        .children(vec![Node::new("participant").attr("jid", peer())]),
                ]),
        )
        .await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while groups::get(&store, &group).unwrap().is_none() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        let snapshot = groups::get(&store, &group).unwrap().unwrap();
        assert_eq!(snapshot.subject.as_deref(), Some("Received in session"));
        assert_eq!(snapshot.members.len(), 1);
        assert_eq!(
            actions::outcomes(&store).unwrap()[0].status,
            actions::ActionStatus::Ok
        );
        assert_eq!(
            groups::ingest(&store).unwrap(),
            0,
            "CLI has no remaining projection work"
        );
        assert!(
            actions::reserve_pending(&store).unwrap().is_empty(),
            "query is never replayed"
        );
        fake.stop_cleanly().await;
    }

    fn contact_reply(id: &str) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .children(vec![Node::new("usync").children(vec![
                Node::new("result").children(vec![Node::new("lid")]),
                Node::new("list").children(vec![Node::new("user")
                    .attr("jid", "15551234567@s.whatsapp.net").children(vec![
                        Node::new("contact").attr("type", "in").text("+15551234567"),
                        Node::new("lid").attr("val", "999999999999@lid")])]),
            ])])
    }

    #[tokio::test]
    async fn contact_reply_projects_status_in_receive_without_another_query() {
        use super::super::{actions, contacts};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("contacts-live-loop.redb"));
        let id = contacts::queue_check(&store, "+15551234567").unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (query, written) = fake.written().await;
        assert_eq!(
            query.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        written.send(Ok(())).unwrap();
        fake.deliver(contact_reply(&id)).await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while contacts::status(&store, "+15551234567").unwrap().is_none() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        let status = contacts::status(&store, "+15551234567").unwrap().unwrap();
        assert_eq!(status.contact_type, Some(contacts::ContactType::In));
        assert_eq!(status.lid, Some(Jid::parse("999999999999@lid").unwrap()));
        assert_eq!(actions::outcomes(&store).unwrap().len(), 1);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        assert_eq!(contacts::ingest(&store).unwrap(), 0);
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn settled_contact_reply_recovers_projection_on_next_session_without_replay() {
        use super::super::{actions, contacts};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("contacts-crash-gap.redb"));
        let id = contacts::queue_check(&store, "+15551234567").unwrap();
        actions::reserve_pending(&store).unwrap();
        actions::record_reply(&store, &contact_reply(&id)).unwrap();
        assert!(contacts::status(&store, "+15551234567").unwrap().is_none());
        let mut fake = start(&store, false, None);
        fake.activate().await;
        assert!(
            contacts::status(&store, "+15551234567")
                .unwrap()
                .unwrap()
                .on_whatsapp
        );
        assert_eq!(actions::outcomes(&store).unwrap().len(), 1);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        fake.nothing_written(Duration::from_millis(30)).await;
        assert_eq!(contacts::ingest(&store).unwrap(), 0);
        fake.stop_cleanly().await;
    }

    fn queue_read_receipt(store: &ProtocolStore, id: &str) {
        let route = encode(&Node::new("message").attr("id", id).attr("from", peer())).unwrap();
        let key = format!("received:{}:{id}", peer());
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key.as_bytes(),
                    1,
                    &serde_json::json!({"status":"decrypted","route":route}),
                )
            })
            .unwrap();
        super::super::receipts::queue_read(
            store,
            &Receipt {
                chat: ChatId::from_jid(&peer()).unwrap(),
                message_ids: vec![MessageId::new(id).unwrap()],
                from: None,
                kind: ReceiptKind::Read,
                server_timestamp: None,
                code: None,
            },
        )
        .unwrap();
    }
    fn receipt_status(store: &ProtocolStore, id: &str) -> serde_json::Value {
        let key = format!(
            "receipt-outbox:{}:read",
            hex::encode(sha2::Sha256::digest(
                format!("received:{}:{id}", peer()).as_bytes()
            ))
        );
        store
            .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap()["status"]
            .clone()
    }
    use sha2::Digest;

    #[tokio::test]
    async fn non_reply_iqs_do_not_refuse_or_complete_activation() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("activation-nonreply.redb"));
        let account = store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap();
        let journal = store.journal_head().unwrap();
        let mut fake = start(&store, false, None);
        let (active, written) = fake.written().await;
        let id = active.get_attr("id").unwrap().clone();
        written.send(Ok(())).unwrap();
        for kind in [
            None,
            Some("get"),
            Some("set"),
            Some("RESULT"),
            Some("unexpected"),
        ] {
            let mut node = Node::new("iq")
                .attr("id", id.clone())
                .children(vec![Node::new("error").attr("code", "503")]);
            if let Some(kind) = kind {
                node = node.attr("type", kind);
            }
            let capture = format!(
                "inbound:sha256:{}",
                hex::encode(sha2::Sha256::digest(encode(&node).unwrap()))
            );
            fake.deliver(node).await;
            // A subsequent server ping proves the actor processed the preceding
            // stanza without terminating or advancing to availability/startup.
            fake.deliver(
                Node::new("iq")
                    .attr("type", "get")
                    .attr("xmlns", "urn:xmpp:ping"),
            )
            .await;
            let (pong, written) = fake.written().await;
            assert_eq!(pong.description, "iq");
            assert_eq!(
                pong.get_attr("type").and_then(|v| v.as_text()),
                Some("result")
            );
            assert!(matches!(
                fake.ready.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap(),
                account
            );
            assert_eq!(store.journal_head().unwrap(), journal);
            written.send(Ok(())).unwrap();
        }
        fake.deliver(Node::new("iq").attr("id", id).attr("type", "result"))
            .await;
        let (presence, written) = fake.written().await;
        assert_eq!(presence.description, "presence");
        written.send(Ok(())).unwrap();
        (&mut fake.ready).await.unwrap();
        fake.stop_cleanly().await;
    }

    #[tokio::test(start_paused = true)]
    async fn activation_admission_rejects_the_deadline_boundary_and_later_replies() {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        for kind in ["result", "error"] {
            let reply = Node::new("iq").attr("id", "active").attr("type", kind);
            assert!(activation_matches(&reply, "active", deadline));
            assert!(!activation_matches(&reply, "another", deadline));
        }
        tokio::time::advance(Duration::from_secs(30)).await;
        for delay in [Duration::ZERO, Duration::from_millis(1)] {
            tokio::time::advance(delay).await;
            for kind in ["result", "error"] {
                assert!(!activation_matches(
                    &Node::new("iq").attr("id", "active").attr("type", kind),
                    "active",
                    deadline
                ));
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn slow_activation_write_does_not_restart_the_response_deadline() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("activation-write-deadline.redb"));
        let mut fake = start(&store, false, None);
        let (_active, written) = fake.written().await;
        tokio::time::advance(Duration::from_secs(31)).await;
        written.send(Ok(())).unwrap();
        let Fake {
            ready,
            task,
            mut from_worker,
            ..
        } = fake;
        let error = tokio::time::timeout(Duration::from_secs(1), task)
            .await
            .expect("expired activation must end without a fresh30s wait")
            .unwrap()
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput {
                field: "connection",
                ..
            }
        ));
        assert!(!error.is_recoverable_by_reconnect());
        assert!(ready.await.is_err());
        assert!(
            from_worker.try_recv().is_err(),
            "no availability/startup write after expiry"
        );
    }

    #[tokio::test]
    async fn activation_cut_points_end_the_epoch_without_readiness() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("activation.redb"));
        // Write failure on the activation request itself.
        let mut fake = start(&store, false, None);
        let (_active, reply) = fake.written().await;
        reply
            .send(Err(Error::Transport(TransportError::Io(
                std::io::ErrorKind::BrokenPipe,
            ))))
            .unwrap();
        let err = fake.finished().await.unwrap_err();
        assert!(err.is_recoverable_by_reconnect(), "{err:?}");
        // Peer close before the activation result: recoverable, never ready.
        let mut fake = start(&store, false, None);
        let (_active, reply) = fake.written().await;
        reply.send(Ok(())).unwrap();
        fake.cut(Error::Transport(TransportError::PeerClosed)).await;
        let Fake { ready, task, .. } = fake;
        let err = task.await.unwrap().unwrap_err();
        assert!(matches!(err, Error::Transport(TransportError::PeerClosed)));
        assert!(ready.await.is_err(), "readiness must not resolve");
        // Activation refused: terminal, not recoverable.
        let mut fake = start(&store, false, None);
        let (active, reply) = fake.written().await;
        reply.send(Ok(())).unwrap();
        let id = active
            .get_attr("id")
            .and_then(|v| v.as_text())
            .unwrap()
            .to_owned();
        fake.deliver(
            Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "error"),
        )
        .await;
        let err = fake.finished().await.unwrap_err();
        assert!(!err.is_recoverable_by_reconnect(), "{err:?}");
    }

    #[tokio::test]
    async fn server_stream_error_is_captured_and_terminal() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("stream-error.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        tokio::time::timeout(Duration::from_secs(5), &mut fake.ready)
            .await
            .unwrap()
            .unwrap();
        let ended = Node::new("stream:error").children(vec![Node::new("conflict")]);
        let wire = encode(&ended).unwrap();
        fake.deliver(ended).await;
        let err = fake.finished().await.unwrap_err();
        assert!(!err.is_recoverable_by_reconnect(), "{err:?}");
        let key = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(&wire))
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .is_some(),
            "server reason preserved encrypted at rest"
        );
        let observed = super::super::inbox::terminal_observation(&store, key.as_bytes())
            .unwrap()
            .expect("live session loop records terminal provenance");
        assert_eq!(observed.last_epoch, ConnectionEpoch(1));
        assert_eq!(observed.observations, 1);
        assert_eq!(observed.first_observed_at, observed.last_observed_at);
        assert!(observed.latest_terminal);
    }

    #[tokio::test]
    async fn undecryptable_message_requests_once_before_write_cut_and_never_replays() {
        for case in [
            "direct_missing",
            "group_missing",
            "group_empty",
            "signature",
            "future",
            "chain",
            "ordinary_prekey",
            "signed_prekey",
            "group_prekey",
            "unknown_version",
        ] {
            let prekey = matches!(case, "ordinary_prekey" | "signed_prekey" | "group_prekey");
            let grouped = matches!(
                case,
                "group_missing"
                    | "group_empty"
                    | "signature"
                    | "future"
                    | "chain"
                    | "group_prekey"
                    | "unknown_version"
            );
            let bad_signature = case == "signature";
            let future_counter = case == "future";
            let missing_chain = case == "chain";
            let unknown_version = case == "unknown_version";
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("retry-cut.redb");
            let store = if prekey {
                registered_store(&path)
            } else {
                open(&path)
            };
            store
                .write(|tx| {
                    let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                    if !prekey {
                        account.signal_registration_id = Some(0x0102_0304);
                    }
                    account.save(tx)
                })
                .unwrap();
            // Like the inbox resilience fixture: no session for this sender, so
            // this is a genuine decryption failure through the production loop.
            use wacore_libsignal::protocol::{IdentityKeyPair, KeyPair, SignalMessage};
            let mut rng = rand::rng();
            let sender = IdentityKeyPair::generate(&mut rng);
            let receiver = IdentityKeyPair::generate(&mut rng);
            let ratchet = KeyPair::generate(&mut rng);
            let message = SignalMessage::new(
                3,
                &[7; 32],
                ratchet.public_key,
                0,
                0,
                &[9; 48],
                sender.identity_key(),
                receiver.identity_key(),
            )
            .unwrap();
            let failed = Node::new("message")
                .attr("t", "1700000000")
                .attr("from", peer())
                .attr("id", "FAILED")
                .attr("type", "text")
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "msg")
                        .bytes(message.serialized().to_vec()),
                ]);
            let failed = if prekey {
                use crate::session::{KeyMaterial, SignalIdentityKey};
                use wacore_libsignal::protocol::{
                    DeviceId, GenericSignedPreKey, PreKeyBundle, SignedPreKeyRecord,
                };
                let account = store.read(AccountRecord::load).unwrap().unwrap();
                let signed_bytes =
                    crate::registration::factory::registered_signed_prekey(&store).unwrap();
                let signed = SignedPreKeyRecord::deserialize(&signed_bytes).unwrap();
                let identity = wacore_libsignal::protocol::PublicKey::from_djb_public_key_bytes(
                    &account.signal_identity.as_ref().unwrap().0.public,
                )
                .unwrap();
                let one = KeyPair::generate(&mut rng);
                let id: u32 = signed.id().unwrap().into();
                let bundle = PreKeyBundle::new(
                    account.signal_registration_id.unwrap(),
                    DeviceId::new(0),
                    Some((0x123456.into(), one.public_key)),
                    (if case == "signed_prekey" { id + 1 } else { id }).into(),
                    signed.public_key().unwrap(),
                    signed.signature().unwrap().to_vec(),
                    wacore_libsignal::protocol::IdentityKey::new(identity),
                )
                .unwrap();
                let sender_account = AccountRecord {
                    registration: crate::session::RegistrationStatus::Registered,
                    signal_registration_id: Some(91),
                    signal_identity: Some(SignalIdentityKey(KeyMaterial {
                        public: sender.public_key().public_key_bytes().try_into().unwrap(),
                        private: *sender.private_key().serialize(),
                    })),
                    ..Default::default()
                };
                let encrypted = crate::signal::direct::prepare(
                    &sender_account,
                    account.own_jid.as_ref().unwrap(),
                    &bundle,
                    None,
                    None,
                    "local prekey fixture",
                )
                .await
                .unwrap();
                failed.children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "pkmsg")
                        .bytes(encrypted.ciphertext.clone()),
                ])
            } else {
                failed
            };
            let group_ciphertext =
                if bad_signature || future_counter || missing_chain || unknown_version {
                    use crate::signal::group::{
                        commit_skdm, encrypt_group_payload, extract_skdm_from_payload,
                        prepare_own_sender_key, process_skdm, sender_key_db_key,
                    };
                    let group = Jid::parse("120363000000000001@g.us").unwrap();
                    let sender = peer().with_device(3);
                    let mine = prepare_own_sender_key(&group, &sender, None).await.unwrap();
                    let (_, bytes) = extract_skdm_from_payload(&mine.distribution_message).unwrap();
                    let installed = process_skdm(
                        &group,
                        &sender,
                        sender_key_db_key(&group, &sender),
                        None,
                        &bytes,
                    )
                    .await
                    .unwrap();
                    assert!(store.write(|tx| commit_skdm(tx, &installed)).unwrap());
                    if future_counter || missing_chain {
                        use wacore_libsignal::protocol::{SenderKeyMessage, SenderKeyRecord};
                        let own_record = SenderKeyRecord::deserialize(&mine.updated_key).unwrap();
                        let state = own_record.sender_key_state().unwrap();
                        SenderKeyMessage::new(
                            3,
                            if missing_chain {
                                state.chain_id().wrapping_add(1)
                            } else {
                                state.chain_id()
                            },
                            if future_counter { 2001 } else { 0 },
                            vec![0; 1].into_boxed_slice(),
                            &mut rng,
                            &state.signing_key_private().unwrap(),
                        )
                        .unwrap()
                        .serialized()
                        .to_vec()
                    } else {
                        let body =
                            super::super::group_send::text_payload("signature fixture").unwrap();
                        let (cipher, _) =
                            encrypt_group_payload(&group, &sender, &mine.updated_key, &body)
                                .await
                                .unwrap();
                        let mut tampered = cipher.to_vec();
                        if unknown_version {
                            // Parser version rejection precedes signature verification.
                            tampered[0] = 0x43;
                        } else {
                            *tampered.last_mut().unwrap() ^= 1;
                        }
                        tampered
                    }
                } else {
                    vec![9; 48]
                };
            if case == "group_empty" {
                let group = Jid::parse("120363000000000001@g.us").unwrap();
                let sender = peer().with_device(3);
                let key = crate::signal::group::sender_key_db_key(&group, &sender);
                let empty = wacore_libsignal::protocol::SenderKeyRecord::new_empty()
                    .serialize()
                    .unwrap();
                store
                    .write(|tx| tx.put_json(Table::SenderKeys, &key, 1, &empty))
                    .unwrap();
            }
            let held_keys = store.read(|tx| tx.scan(Table::SenderKeys, 1, 10)).unwrap();
            let direct_enc = failed.child("enc").unwrap().clone();
            let failed = if grouped {
                failed
                    .attr("from", "120363000000000001@g.us")
                    .attr(
                        "participant",
                        if prekey {
                            peer()
                        } else {
                            peer().with_device(3)
                        },
                    )
                    .children(vec![if prekey {
                        direct_enc
                    } else {
                        Node::new("enc")
                            .attr("v", "2")
                            .attr("type", "skmsg")
                            .bytes(group_ciphertext)
                    }])
            } else {
                failed
            };
            let mut fake = start(&store, false, None);
            fake.activate().await;
            if prekey {
                let (upload, write) = fake.written().await;
                assert_upload(&upload);
                write.send(Ok(())).unwrap();
                fake.deliver(result_for(&upload, false)).await;
            }
            let held_prekeys = store.read(|tx| tx.scan(Table::Prekeys, 1, 1000)).unwrap();
            fake.deliver(failed.clone()).await;
            let (retry, reply) = fake.written().await;
            assert_eq!(retry.description, "receipt");
            assert_eq!(
                retry
                    .child("retry")
                    .unwrap()
                    .get_attr("error")
                    .and_then(|v| v.as_text()),
                Some(if bad_signature {
                    "5"
                } else if future_counter {
                    "6"
                } else if missing_chain || unknown_version {
                    "4"
                } else if prekey {
                    "3"
                } else {
                    "1"
                })
            );
            assert_eq!(
                retry
                    .child("retry")
                    .unwrap()
                    .get_attr("t")
                    .and_then(|v| v.as_text()),
                Some("1700000000")
            );
            assert_eq!(
                retry.get_attr("type").and_then(|v| v.as_text()),
                Some("retry")
            );
            assert_eq!(
                retry.get_attr("id").and_then(|v| v.as_text()),
                Some("FAILED")
            );
            assert_eq!(
                super::super::receipts::outcomes(&store).unwrap()[0].status,
                "outcome_unknown"
            );
            let wire = encode(&failed).unwrap();
            let capture = format!("inbound:sha256:{}", hex::encode(sha2::Sha256::digest(wire)));
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            reply
                .send(Err(Error::Transport(TransportError::PeerClosed)))
                .unwrap();
            assert!(
                fake.finished()
                    .await
                    .unwrap_err()
                    .is_recoverable_by_reconnect()
            );

            let mut fake = start(&store, false, None);
            fake.activate().await;
            let mut alternate_from = super::super::receipts::jid(&failed, "from").unwrap();
            alternate_from.kind = crate::model::jid::JidKind::AgentDevice;
            fake.deliver(failed.attr("from", alternate_from)).await;
            // Following receipt proves the loop continued after the failed
            // redelivery, without emitting retry or delivery ACK for that message.
            fake.deliver(Node::new("receipt").attr("from", peer()).attr("id", "NEXT"))
                .await;
            let (ack, reply) = fake.written().await;
            assert_eq!(ack.description, "ack");
            assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("NEXT"));
            reply.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_millis(100)).await;
            assert_eq!(super::super::receipts::outcomes(&store).unwrap().len(), 1);
            assert_eq!(
                store.read(|tx| tx.scan(Table::SenderKeys, 1, 10)).unwrap(),
                held_keys
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Prekeys, 1, 1000)).unwrap(),
                held_prekeys
            );
            fake.stop_cleanly().await;
        }
    }

    #[tokio::test]
    async fn authenticated_invalid_application_padding_never_requests_retry_or_confirmation() {
        use crate::signal::group::{
            commit_skdm, encrypt_raw_for_test, extract_skdm_from_payload, prepare_own_sender_key,
            process_skdm, sender_key_db_key,
        };
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("invalid-app-padding.redb"));
        store
            .write(|tx| {
                let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                account.signal_registration_id = Some(31);
                account.save(tx)
            })
            .unwrap();
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let sender = peer().with_device(3);
        let own = prepare_own_sender_key(&group, &sender, None).await.unwrap();
        let (_, bytes) = extract_skdm_from_payload(&own.distribution_message).unwrap();
        let installed = process_skdm(
            &group,
            &sender,
            sender_key_db_key(&group, &sender),
            None,
            &bytes,
        )
        .await
        .unwrap();
        assert!(store.write(|tx| commit_skdm(tx, &installed)).unwrap());
        let keys = store.read(|tx| tx.scan(Table::SenderKeys, 1, 10)).unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        for (id, plain) in [("PAD-ZERO", vec![42, 0]), ("PAD-ENTIRE", vec![2, 2])] {
            let ciphertext = encrypt_raw_for_test(&group, &sender, &own.updated_key, &plain).await;
            fake.deliver(
                Node::new("message")
                    .attr("from", group.clone())
                    .attr("participant", sender.clone())
                    .attr("id", id)
                    .attr("t", "1700000000")
                    .children(vec![
                        Node::new("enc")
                            .attr("v", "2")
                            .attr("type", "skmsg")
                            .bytes(ciphertext),
                    ]),
            )
            .await;
            // This following ACK is a receive-loop barrier. An incorrect retry
            // or delivery confirmation for the invalid payload would arrive first.
            fake.deliver(
                Node::new("receipt")
                    .attr("from", peer())
                    .attr("id", "AFTER-PADDING"),
            )
            .await;
            let (ack, reply) = fake.written().await;
            assert_eq!(ack.description, "ack");
            assert_eq!(
                ack.get_attr("id").and_then(|v| v.as_text()),
                Some("AFTER-PADDING")
            );
            reply.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_millis(100)).await;
            assert!(super::super::receipts::outcomes(&store).unwrap().is_empty());
            assert_eq!(
                store.read(|tx| tx.scan(Table::SenderKeys, 1, 10)).unwrap(),
                keys
            );
            assert!(
                store
                    .read(|tx| tx.scan(Table::Inbox, 1, 10))
                    .unwrap()
                    .is_empty()
            );
        }
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn missing_registration_skips_retry_without_ending_receive() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("retry-missing-registration.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(
            Node::new("message")
                .attr("from", peer())
                .attr("id", "FAILED")
                .children(vec![
                    Node::new("enc").attr("type", "msg").bytes(vec![9; 48]),
                ]),
        )
        .await;
        fake.deliver(Node::new("receipt").attr("from", peer()).attr("id", "NEXT"))
            .await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("NEXT"));
        reply.send(Ok(())).unwrap();
        assert!(super::super::receipts::outcomes(&store).unwrap().is_empty());
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn explicit_receipt_write_cut_is_reserved_once_and_settled_by_a_later_ack() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("receipt-cut.redb"));
        queue_read_receipt(&store, "first");
        queue_read_receipt(&store, "second");
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (receipt, reply) = fake.written().await;
        assert_eq!(receipt.description, "receipt");
        assert_eq!(
            receipt.get_attr("type").and_then(|v| v.as_text()),
            Some("read")
        );
        assert_eq!(
            receipt_status(&store, "first"),
            "outcome_unknown",
            "reserved before the write"
        );
        let root = receipt
            .get_attr("id")
            .unwrap()
            .as_text()
            .unwrap()
            .to_owned();
        let list = receipt.child("list").unwrap();
        assert_eq!(list.child_nodes().len(), 1);
        assert_eq!(receipt_status(&store, "second"), "outcome_unknown");
        reply
            .send(Err(Error::Transport(TransportError::Io(
                std::io::ErrorKind::ConnectionReset,
            ))))
            .unwrap();
        let err = fake.finished().await.unwrap_err();
        assert!(err.is_recoverable_by_reconnect(), "{err:?}");
        assert_eq!(receipt_status(&store, "first"), "outcome_unknown");
        assert_eq!(receipt_status(&store, "second"), "outcome_unknown");

        // Next epoch: no replay, and the server's late ACK settles the reservation.
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.nothing_written(Duration::from_millis(300)).await;
        fake.deliver(
            Node::new("ack")
                .attr("id", root)
                .attr("class", "receipt")
                .attr("type", "read")
                .attr("from", peer()),
        )
        .await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while ["first", "second"]
                .iter()
                .any(|id| receipt_status(&store, id) != "server_acknowledged")
            {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn automatic_read_for_recovered_duplicate_is_once_only_after_write_cut() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("offline-read.redb"));
        let node = Node::new("message")
            .attr("from", peer())
            .attr("id", "OFFLINE-RECOVERED")
            .attr("type", "text");
        let key = format!("received:{}:OFFLINE-RECOVERED", peer());
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key.as_bytes(),
                    1,
                    &serde_json::json!({"status":"decrypted","route":encode(&node)?,
                "protobuf":vec![0x0au8,1,b'x']}),
                )
            })
            .unwrap();
        let mut configured = config();
        configured.transport.mark_read = true;
        let mut fake = start_with_config(&store, false, None, configured.clone());
        fake.activate().await;
        fake.deliver(node.clone()).await;
        let (delivery, reply) = fake.written().await;
        assert_eq!(delivery.description, "receipt");
        assert_ne!(
            delivery.get_attr("type").and_then(|v| v.as_text()),
            Some("read")
        );
        reply.send(Ok(())).unwrap();
        let (read, reply) = fake.written().await;
        assert_eq!(
            read.get_attr("type").and_then(|v| v.as_text()),
            Some("read")
        );
        assert_eq!(
            receipt_status(&store, "OFFLINE-RECOVERED"),
            "outcome_unknown"
        );
        reply
            .send(Err(Error::Transport(TransportError::PeerClosed)))
            .unwrap();
        assert!(fake.finished().await.is_err());
        let mut fake = start_with_config(&store, false, None, configured);
        fake.activate().await;
        fake.deliver(node).await;
        let (delivery, reply) = fake.written().await;
        assert_ne!(
            delivery.get_attr("type").and_then(|v| v.as_text()),
            Some("read")
        );
        reply.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(100)).await;
        assert_eq!(
            receipt_status(&store, "OFFLINE-RECOVERED"),
            "outcome_unknown"
        );
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn unsupported_receipt_type_and_namespaces_stay_captured_without_ack() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("unsupported-receipt.redb"));
        let base = Node::new("receipt")
            .attr("from", peer())
            .attr("id", "RECEIPT-FIXTURE");
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let head = store.journal_head().unwrap();
        for node in [
            base.clone().attr("type", peer()),
            base.clone().attr("from", "15555550100@unknown.example"),
            base.clone().attr("from", "lid"),
        ] {
            let wire = encode(&node).unwrap();
            let capture = format!(
                "inbound:sha256:{}",
                hex::encode(sha2::Sha256::digest(&wire))
            );
            fake.deliver(node).await;
            fake.nothing_written(Duration::from_millis(100)).await;
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            assert_eq!(store.journal_head().unwrap(), head);
        }
        fake.deliver(base.attr("recipient", "120363000000000001@g.us"))
            .await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert!(ack.get_attr("type").is_none());
        assert!(ack.get_attr("recipient").is_none());
        reply.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
    }

    /// Content this client refuses in a receipt or privacy-token notification
    /// is captured without ACK, and the session keeps receiving. Ending it
    /// would make every later session end on the same redelivered stanza.
    #[tokio::test]
    async fn refused_receipt_and_notification_content_does_not_end_receive() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("refused-content.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let head = store.journal_head().unwrap();
        let refused = [
            // A receipt list child that is not `item`.
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "REFUSED-RECEIPT")
                .children(vec![Node::new("list").children(vec![Node::new("bogus")])]),
            // A privacy-token notification without its timestamp.
            Node::new("notification")
                .attr("from", peer())
                .attr("id", "REFUSED-TOKEN")
                .attr("type", "privacy_token")
                .children(vec![Node::new("tokens").children(vec![
                    Node::new("token")
                        .attr("type", "trusted_contact")
                        .bytes(vec![1, 2, 3]),
                ])]),
            // A devices notification without `id` cannot be acknowledged; no
            // device sync is queued for it either.
            Node::new("notification")
                .attr("from", peer())
                .attr("type", "devices")
                .children(vec![Node::new("add").attr("device_hash", "2:x").children(
                    vec![
                        Node::new("device")
                            .attr("jid", peer().with_device(3))
                            .attr("key-index", "1"),
                    ],
                )]),
            // A prekey count notification without `id`: no ACK, no upload.
            Node::new("notification")
                .attr("from", Jid::parse("s.whatsapp.net").unwrap())
                .attr("type", "encrypt")
                .children(vec![Node::new("count").attr("value", "5")]),
            // A count envelope cannot invent an ACK destination.
            Node::new("notification")
                .attr("from", "1:invalid@s.whatsapp.net")
                .attr("id", "COUNT-BAD-SOURCE")
                .attr("type", "encrypt")
                .children(vec![Node::new("count").attr("value", "5")]),
        ];
        for node in refused {
            let wire = encode(&node).unwrap();
            let capture = format!(
                "inbound:sha256:{}",
                hex::encode(sha2::Sha256::digest(&wire))
            );
            fake.deliver(node).await;
            fake.nothing_written(Duration::from_millis(100)).await;
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            assert_eq!(store.journal_head().unwrap(), head);
        }
        assert!(
            super::super::devices::pending_users(&store)
                .unwrap()
                .is_empty()
        );
        // Still receiving: a supported receipt is answered as before.
        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "RECEIPT-FIXTURE")
                .attr("recipient", "120363000000000001@g.us"),
        )
        .await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        reply.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
    }

    /// The status-receipt shape observed live on 2026-09-24 (an empty
    /// `<participants message_id/>`) is recorded and acknowledged. Since the
    /// offline-aggregate branch (2026-10-10) it had been refused as an empty
    /// aggregate, which ended receive at every status receipt. An aggregate
    /// with a refused entry stays unacknowledged and records no status view.
    #[tokio::test]
    async fn observed_status_receipt_is_recorded_and_acknowledged() {
        use super::super::status_receipts;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("status-receipt.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "STATUS-RECEIPT")
                .attr("type", "read")
                .attr("t", "1700000000")
                .children(vec![
                    Node::new("participants").attr("message_id", "OUR-STATUS"),
                ]),
        )
        .await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("STATUS-RECEIPT")
        );
        reply.send(Ok(())).unwrap();
        let views = status_receipts::list(&store, Some("OUR-STATUS")).unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].kind, status_receipts::StatusReceiptKind::Read);

        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "BAD-AGGREGATE")
                .children(vec![
                    Node::new("participants")
                        .attr("message_id", "OUR-MESSAGE")
                        .children(vec![
                            Node::new("user")
                                .attr("jid", peer())
                                .attr("type", "not-a-type")
                                .attr("t", "1700000000"),
                        ]),
                ]),
        )
        .await;
        fake.nothing_written(Duration::from_millis(100)).await;
        assert!(
            status_receipts::list(&store, Some("OUR-MESSAGE"))
                .unwrap()
                .is_empty()
        );
        fake.stop_cleanly().await;
    }

    /// Only the receipt's wire content is refused without ending receive. A
    /// fault in our own stored rows (here a corrupt outbox entry the receipt
    /// lookup scans) still ends the session with the error, rather than
    /// leaving every later receipt silently unacknowledged.
    #[tokio::test]
    async fn corrupt_stored_rows_still_end_receive_on_a_valid_receipt() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("corrupt-outbox.redb"));
        store
            .write(|tx| {
                tx.put_json(
                    Table::Outbox,
                    b"corrupt-row",
                    1,
                    &serde_json::json!("not an outbox entry"),
                )
            })
            .unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "VALID-RECEIPT")
                .attr("t", "1700000000"),
        )
        .await;
        let error = fake.finished().await.unwrap_err();
        assert!(error.to_string().contains("invalid saved outbox entry"));
    }

    /// A devices notification naming our own LID is ours, like our own PN: it
    /// is acknowledged without queueing a contact device sync (receive audit F7).
    #[tokio::test]
    async fn own_lid_devices_notification_queues_no_contact_sync() {
        use super::super::devices::pending_users;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("own-lid-devices.redb"));
        let own_lid = Jid::parse("99999999@lid").unwrap();
        store
            .write(|tx| super::super::group_send::record_own_lid(tx, &own_lid))
            .unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(
            Node::new("notification")
                .attr("type", "devices")
                .attr("id", "OWN-DEVICES")
                .attr("from", own_lid.clone())
                .children(vec![Node::new("add").attr("device_hash", "2:x").children(
                    vec![
                        Node::new("device")
                            .attr("jid", own_lid.with_device(3))
                            .attr("key-index", "1"),
                    ],
                )]),
        )
        .await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("type").and_then(|v| v.as_text()),
            Some("devices")
        );
        reply.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(100)).await;
        assert!(pending_users(&store).unwrap().is_empty());
        fake.stop_cleanly().await;
    }

    /// A decrypted frame the decoder refuses is captured for inspection and
    /// skipped; receive continues. It used to end the session without a
    /// capture, and a redelivered one ended every later session.
    #[tokio::test]
    async fn undecodable_frame_is_captured_and_receive_continues() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("undecodable.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        // Uncompressed frame: a node whose description is a nested JID tag.
        let mut frame = vec![0x00, crate::binary::tags::LIST_8, 1];
        frame.extend(vec![crate::binary::tags::JID_PAIR; 1_000]);
        fake.deliver_frame(&frame).await;
        fake.nothing_written(Duration::from_millis(100)).await;
        let key = format!(
            "inbound-undecodable:sha256:{}",
            hex::encode(sha2::Sha256::digest(&frame))
        );
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["class"], "invalid_node");
        let stored: Vec<u8> = serde_json::from_value(saved["frame"].clone()).unwrap();
        assert_eq!(stored, frame);
        // Redelivery is stored once and still does not end receive.
        fake.deliver_frame(&frame).await;
        fake.nothing_written(Duration::from_millis(100)).await;
        fake.deliver(
            Node::new("receipt")
                .attr("from", peer())
                .attr("id", "AFTER-UNDECODABLE")
                .attr("t", "1700000000"),
        )
        .await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        reply.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
    }

    /// An undecodable `stream:error` or `failure` still ends the session
    /// without reconnecting, as a decodable one does; so does a server close
    /// right after a stanza whose name could not be read. A close after a
    /// decoded stanza stays an ordinary recoverable loss.
    #[tokio::test]
    async fn undecodable_terminal_stanzas_end_the_session_without_reconnect() {
        use crate::binary::tags::{BINARY_8, JID_PAIR, LIST_8};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("undecodable-terminal.redb"));
        // `<stream:error>` whose only child has an undecodable description.
        let mut stream_error = vec![0x00, LIST_8, 2, BINARY_8, 12];
        stream_error.extend_from_slice(b"stream:error");
        stream_error.extend([LIST_8, 1, LIST_8, 1, JID_PAIR, JID_PAIR]);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver_frame(&stream_error).await;
        let error = fake.finished().await.unwrap_err();
        assert!(!error.is_recoverable_by_reconnect());
        assert!(error.to_string().contains("could not decode"), "{error}");
        assert_eq!(
            super::super::inbox::undecodable_frames(&store)
                .unwrap()
                .len(),
            1
        );

        // A stanza whose name itself is unreadable, then the server closes.
        let mut unreadable = vec![0x00, LIST_8, 1];
        unreadable.extend([JID_PAIR; 4]);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver_frame(&unreadable).await;
        fake.cut(Error::Transport(TransportError::PeerClosed)).await;
        let error = fake.finished().await.unwrap_err();
        assert!(!error.is_recoverable_by_reconnect(), "{error}");

        // The same, but a decoded stanza arrives in between: an ordinary loss.
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver_frame(&unreadable).await;
        fake.deliver(Node::new("ib").attr("from", Jid::parse("s.whatsapp.net").unwrap()))
            .await;
        fake.cut(Error::Transport(TransportError::PeerClosed)).await;
        let error = fake.finished().await.unwrap_err();
        assert!(error.is_recoverable_by_reconnect(), "{error}");
    }

    #[tokio::test]
    async fn delivery_confirmation_write_cut_then_redelivery_confirms_again() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("confirm-cut.redb"));
        let incoming = Node::new("receipt")
            .attr("from", peer())
            .attr("id", "OUR-MSG")
            .attr("t", "1700000000");
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(incoming.clone()).await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("OUR-MSG")
        );
        reply
            .send(Err(Error::Transport(TransportError::PeerClosed)))
            .unwrap();
        assert!(fake.finished().await.is_err());
        // Server redelivers the same receipt: fact stays recorded once, ack repeats.
        let keys_before: std::collections::BTreeSet<Vec<u8>> = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 10_000))
            .unwrap()
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(incoming).await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        reply.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
        let keys_after: std::collections::BTreeSet<Vec<u8>> = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 10_000))
            .unwrap()
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        let added: Vec<&Vec<u8>> = keys_after.difference(&keys_before).collect();
        // Only the new activation result's raw capture is new: the redelivered
        // receipt hashes to the same capture key and its fact row is reused.
        assert_eq!(added.len(), 1, "{added:?}");
        assert!(added[0].starts_with(b"inbound:sha256:"));
    }

    /// Disk full on the real session loop: a receipt that cannot be stored is
    /// not acknowledged and the session ends with the storage error (the
    /// handle refuses further writes); after space returns and the store is
    /// reopened, the server's redelivery is stored and acknowledged.
    #[tokio::test]
    async fn a_full_disk_ends_receive_without_an_ack_and_redelivery_confirms_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("full.redb");
        let faults = crate::store::full_disk::Faults::new();
        let open_full =
            || Arc::new(crate::store::full_disk::open(&path, "test", [4; 32], &faults).unwrap());
        let store = open_full();
        store
            .write(|tx| {
                AccountRecord {
                    own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
                    registration: crate::session::RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)
            })
            .unwrap();
        let incoming = Node::new("receipt")
            .attr("from", peer())
            .attr("id", "OUR-MSG")
            .attr("t", "1700000000");
        let records = |store: &ProtocolStore| {
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10_000))
                .unwrap()
                .len()
        };
        let mut fake = start(&store, false, None);
        fake.activate().await;
        // Readiness is signalled after the post-activation reservations; on
        // this single-threaded runtime the worker then runs on until it waits
        // for input, so the fault below meets the receipt's own writes.
        (&mut fake.ready).await.unwrap();
        let before = records(&store);
        faults.set_full(true);
        fake.deliver(incoming.clone()).await;
        let Fake {
            mut from_worker,
            task,
            ..
        } = fake;
        let ended = tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("worker ends")
            .expect("worker did not panic");
        let error = ended.expect_err("a full disk ends the session");
        assert!(error.to_string().contains("simulated full disk"), "{error}");
        assert!(
            from_worker.try_recv().is_err(),
            "nothing written after the failure"
        );
        drop(store);
        faults.set_full(false);
        let store = open_full();
        assert_eq!(records(&store), before, "the receipt left no record");
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.deliver(incoming).await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("OUR-MSG")
        );
        reply.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
        assert!(records(&store) > before, "the redelivery was recorded");
    }

    #[tokio::test(start_paused = true)]
    async fn group_outbox_socket_cuts_keep_every_query_and_ciphertext_reservation_without_replay() {
        use super::super::{group_fanout, outgoing_group};
        use outgoing_group::tests::{device_reply, enqueue, key_reply, membership};
        let outbox =
            crate::outbox::Outbox::new("comparison-driver", crate::profile::Limits::default());
        for cut in ["membership", "devices", "keys", "send"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("normal-group-cut.redb"));
            let f = group_fanout::tests::fixture();
            store.write(|tx| f.alice.save(tx)).unwrap();
            let ticket = enqueue(&store, &f.plan.chat, "original-group-cut");
            let mut fake = start(&store, true, Some(ticket));
            fake.activate().await;
            let mut original_query_ids = vec![];
            loop {
                let (node, written) = fake.written().await;
                let stage = if node.description == "message" {
                    "send"
                } else {
                    match node.get_attr("xmlns").and_then(|v| v.as_text()) {
                        Some("w:g2") => "membership",
                        Some("usync") => "devices",
                        Some("encrypt") => "keys",
                        _ => panic!("unexpected outbound stage"),
                    }
                };
                if node.description == "iq" {
                    original_query_ids.push(
                        node.get_attr("id")
                            .and_then(|v| v.as_text())
                            .unwrap()
                            .to_owned(),
                    );
                }
                if stage == cut {
                    // State is inspected before the fake socket acknowledges
                    // this write, including the final encrypted original wire.
                    let entry = store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap();
                    assert_eq!(
                        entry.progress,
                        if cut == "send" {
                            crate::model::send::AttemptProgress::InFlight
                        } else {
                            crate::model::send::AttemptProgress::Queued
                        }
                    );
                    if cut == "send" {
                        assert_eq!(entry.attempts.len(), 1);
                        assert!(entry.status().is_uncertain());
                    }
                    written
                        .send(Err(Error::Transport(TransportError::PeerClosed)))
                        .unwrap();
                    break;
                }
                written.send(Ok(())).unwrap();
                let reply = match stage {
                    "membership" => membership(&node, &f.plan.own, &f.plan.recipients),
                    "devices" => device_reply(&node),
                    "keys" => key_reply(&node, &f),
                    _ => unreachable!(),
                };
                fake.deliver(reply).await;
            }
            assert!(
                fake.finished()
                    .await
                    .unwrap_err()
                    .is_recoverable_by_reconnect()
            );
            let original = store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap();
            let progress = outgoing_group::preparation(&store, ticket)
                .unwrap()
                .unwrap();
            let mut restarted = start(&store, true, Some(ticket));
            restarted.activate().await;
            if cut == "devices" {
                // Existing background usync can re-read pending users in a new
                // epoch. It cannot resume this group intent or its old query.
                let (query, written) = restarted.written().await;
                assert_eq!(
                    query.get_attr("xmlns").and_then(|v| v.as_text()),
                    Some("usync")
                );
                assert_eq!(
                    query
                        .child("usync")
                        .unwrap()
                        .get_attr("context")
                        .and_then(|v| v.as_text()),
                    Some("interactive")
                );
                assert!(
                    !original_query_ids
                        .iter()
                        .any(|id| Some(id.as_str())
                            == query.get_attr("id").and_then(|v| v.as_text()))
                );
                written.send(Ok(())).unwrap();
                restarted.deliver(device_reply(&query)).await;
            }
            restarted.nothing_written(Duration::from_millis(1200)).await;
            assert_eq!(
                store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap(),
                original
            );
            assert_eq!(
                outgoing_group::preparation(&store, ticket)
                    .unwrap()
                    .unwrap(),
                progress
            );
            if cut == "send" {
                let id = original.wire_id.as_ref().unwrap();
                restarted
                    .deliver(
                        Node::new("ack")
                            .attr("class", "message")
                            .attr("id", id.as_str())
                            .attr("from", f.plan.chat.clone()),
                    )
                    .await;
                tokio::task::yield_now().await;
                let accepted = store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap();
                assert!(accepted.status().server_accepted());
                assert_eq!(accepted.attempts, original.attempts);
                assert!(
                    accepted
                        .facts
                        .iter()
                        .any(|fact| fact.kind
                            == crate::model::send::DeliveryFactKind::DeliveryUnknown)
                );
                restarted
                    .deliver(
                        Node::new("receipt")
                            .attr("id", id.as_str())
                            .attr("from", f.plan.chat.clone())
                            .attr("participant", f.plan.recipients[0].clone()),
                    )
                    .await;
                let (ack, written) = restarted.written().await;
                assert_eq!(ack.description, "ack");
                written.send(Ok(())).unwrap();
                let delivered = store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap();
                assert!(delivered.facts.iter().any(|fact| fact.kind
                    == crate::model::send::DeliveryFactKind::Delivered
                    && fact.device.is_some()));
                assert_eq!(delivered.attempts, original.attempts);
            }
            restarted.stop_cleanly().await;
        }
    }

    #[tokio::test]
    async fn malformed_direct_directory_reply_suspends_only_its_intent() {
        use super::super::{group_fanout, outgoing_group};
        use crate::{
            model::{ids::ClientRequestId, message::MessageContent, send::SendRequest},
            outbox::Outbox,
            profile::Limits,
        };
        for case in [
            "missing_list",
            "foreign_user",
            "missing_identity",
            "short_signature",
            "duplicate_user",
            "root_error",
            "invalid_signature",
            "corrupt_saved_pin",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("malformed-direct.redb");
            let store = open(&path);
            let f = group_fanout::tests::fixture();
            store.write(|tx| f.alice.save(tx)).unwrap();
            let outbox = Outbox::new("comparison-driver", Limits::default());
            let ticket = store
                .write(|tx| {
                    Ok(outbox
                        .enqueue(
                            tx,
                            &SendRequest {
                                client_request_id: ClientRequestId::new("malformed-direct")?,
                                chat: ChatId::from_jid(&peer())?,
                                content: MessageContent::Text {
                                    body: "offline fixture".into(),
                                    quoted: None,
                                    style: Default::default(),
                                },
                            },
                        )?
                        .0
                        .local_id)
                })
                .unwrap();
            let mut fake = start(&store, true, Some(ticket));
            fake.activate().await;
            let (query, written) = fake.written().await;
            written.send(Ok(())).unwrap();
            let original = store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap();
            let valid = outgoing_group::tests::key_reply(&query, &f);
            let mut user = valid.child("list").unwrap().child_nodes()[0].clone();
            if case == "foreign_user" {
                user = user.attr("jid", peer().with_device(9));
            }
            if case == "missing_identity" {
                user = user.clone().children(
                    user.child_nodes()
                        .iter()
                        .filter(|n| n.description != "identity")
                        .cloned()
                        .collect(),
                );
            }
            if matches!(case, "short_signature" | "invalid_signature") {
                let skey = Node::new("skey").children(
                    user.child("skey")
                        .unwrap()
                        .child_nodes()
                        .iter()
                        .map(|n| {
                            if n.description == "signature" {
                                Node::new("signature").bytes(vec![
                                    0;
                                    if case == "short_signature" {
                                        63
                                    } else {
                                        64
                                    }
                                ])
                            } else {
                                n.clone()
                            }
                        })
                        .collect(),
                );
                user = user.clone().children(
                    user.child_nodes()
                        .iter()
                        .map(|n| {
                            if n.description == "skey" {
                                skey.clone()
                            } else {
                                n.clone()
                            }
                        })
                        .collect(),
                );
            }
            let users = if case == "duplicate_user" {
                vec![user.clone(), user]
            } else {
                vec![user]
            };
            let reply = Node::new("iq")
                .attr("type", "result")
                .attr("id", query.get_attr("id").unwrap().clone());
            let reply = if case == "missing_list" {
                reply
            } else if case == "root_error" {
                reply.children(vec![Node::new("error").attr("code", "500")])
            } else {
                reply.children(vec![Node::new("list").children(users)])
            };
            if case == "corrupt_saved_pin" {
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::PeerIdentities,
                            format!("comparison-direct:{}", peer()).as_bytes(),
                            1,
                            &serde_json::json!({"invalid_saved_pin":true}),
                        )
                    })
                    .unwrap();
            }
            fake.deliver(reply.clone()).await;
            if case == "corrupt_saved_pin" {
                assert!(fake.finished().await.is_err());
                assert!(
                    super::super::outgoing::identity_suspension(&store, ticket)
                        .unwrap()
                        .is_none()
                );
                assert_eq!(
                    store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap(),
                    original
                );
                let pin: serde_json::Value = store
                    .read(|tx| {
                        tx.get_json(
                            Table::PeerIdentities,
                            format!("comparison-direct:{}", peer()).as_bytes(),
                            1,
                        )
                    })
                    .unwrap()
                    .unwrap();
                assert_eq!(pin, serde_json::json!({"invalid_saved_pin":true}));
                continue;
            }
            fake.deliver(reply).await;
            fake.deliver(
                Node::new("iq")
                    .attr("type", "get")
                    .attr("xmlns", "urn:xmpp:ping"),
            )
            .await;
            let (pong, written) = fake.written().await;
            assert_eq!(
                pong.get_attr("type").and_then(|v| v.as_text()),
                Some("result"),
                "{case}"
            );
            written.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_millis(20)).await;
            fake.stop_cleanly().await;
            let held = super::super::outgoing::identity_suspension(&store, ticket)
                .unwrap()
                .unwrap();
            assert_eq!(
                held.reason,
                if case == "invalid_signature" {
                    "invalid_signed_bundle"
                } else {
                    "invalid_directory_reply"
                }
            );
            assert_eq!(
                store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap(),
                original
            );
            for table in [
                Table::SignalSessions,
                Table::PeerIdentities,
                Table::SenderKeys,
            ] {
                assert!(store.read(|tx| tx.scan(table, 1, 100)).unwrap().is_empty());
            }
            let key = format!("outbox-key-query:{ticket}");
            let reservation = store
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            drop(store);
            let reopened = identity_store(&path);
            let snapshot = || {
                reopened
                    .read(|tx| {
                        Table::ALL
                            .iter()
                            .map(|table| tx.scan(*table, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before_resume = snapshot();
            let head = reopened.journal_head().unwrap();
            for policy in [
                crate::registration::request::IdentityChangePolicy::Pinned,
                crate::registration::request::IdentityChangePolicy::ApkAuthenticated,
            ] {
                assert!(
                    super::super::outgoing::resume_identity_selected(
                        &reopened,
                        &f.alice,
                        ticket,
                        ConnectionEpoch(99),
                        policy
                    )
                    .await
                    .unwrap()
                    .is_none(),
                    "{case}"
                );
                assert_eq!(snapshot(), before_resume);
                assert_eq!(reopened.journal_head().unwrap(), head);
            }
            assert_eq!(
                super::super::outgoing::identity_suspension(&reopened, ticket)
                    .unwrap()
                    .unwrap(),
                held
            );
            assert_eq!(
                reopened
                    .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                    .unwrap()
                    .unwrap(),
                reservation
            );
            assert!(
                super::super::outgoing::begin_selected(&reopened, Some(ticket))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[tokio::test]
    async fn cancelled_direct_key_reply_keeps_receive_and_original_reservation() {
        use crate::{
            model::{ids::ClientRequestId, message::MessageContent, send::SendRequest},
            outbox::Outbox,
            profile::Limits,
        };
        for outcome in ["result", "error"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("cancelled-direct.redb");
            let store = open(&path);
            let outbox = Outbox::new("comparison-driver", Limits::default());
            let ticket = store
                .write(|tx| {
                    Ok(outbox
                        .enqueue(
                            tx,
                            &SendRequest {
                                client_request_id: ClientRequestId::new("cancelled-direct")?,
                                chat: ChatId::from_jid(&peer())?,
                                content: MessageContent::Text {
                                    body: "offline fixture".into(),
                                    quoted: None,
                                    style: Default::default(),
                                },
                            },
                        )?
                        .0
                        .local_id)
                })
                .unwrap();
            let mut fake = start(&store, true, Some(ticket));
            fake.activate().await;
            let (query, written) = fake.written().await;
            assert_eq!(
                query.get_attr("xmlns").and_then(|v| v.as_text()),
                Some("encrypt")
            );
            written.send(Ok(())).unwrap();
            let key = format!("outbox-key-query:{ticket}");
            let reserved = store
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert!(store.write(|tx| outbox.cancel(tx, ticket)).unwrap());
            let cancelled = store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap();
            // Even an unusable result is ignored before directory parsing once
            // the intent is cancelled. Cancellation does not adopt its keys.
            let reply = Node::new("iq")
                .attr("type", outcome)
                .attr("id", query.get_attr("id").unwrap().clone());
            fake.deliver(reply.clone()).await;
            fake.deliver(reply).await;
            fake.deliver(
                Node::new("iq")
                    .attr("type", "get")
                    .attr("xmlns", "urn:xmpp:ping"),
            )
            .await;
            let (pong, written) = fake.written().await;
            assert_eq!(
                pong.get_attr("type").and_then(|v| v.as_text()),
                Some("result")
            );
            written.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_millis(20)).await;
            fake.stop_cleanly().await;
            assert_eq!(
                store.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap(),
                cancelled
            );
            assert!(cancelled.attempts.is_empty());
            assert_eq!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                    .unwrap()
                    .unwrap(),
                reserved
            );
            for table in [
                Table::SignalSessions,
                Table::PeerIdentities,
                Table::SenderKeys,
            ] {
                assert!(store.read(|tx| tx.scan(table, 1, 100)).unwrap().is_empty());
            }
            drop(store);
            let reopened = identity_store(&path);
            assert_eq!(
                reopened.read(|tx| outbox.get(tx, ticket)).unwrap().unwrap(),
                cancelled
            );
            assert_eq!(
                reopened
                    .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                    .unwrap()
                    .unwrap(),
                reserved
            );
            assert!(
                super::super::outgoing::begin_selected(&reopened, Some(ticket))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn key_query_write_cut_keeps_the_reservation_and_is_not_reselected() {
        use crate::{
            model::{ids::ClientRequestId, message::MessageContent, send::SendRequest},
            outbox::Outbox,
            profile::Limits,
        };
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("query-cut.redb"));
        let ticket = store
            .write(|tx| {
                Ok(Outbox::new("comparison-driver", Limits::default())
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: ClientRequestId::new("cut-send").unwrap(),
                            chat: ChatId::from_jid(&peer()).unwrap(),
                            content: MessageContent::Text {
                                body: "fixture".into(),
                                quoted: None,
                                style: Default::default(),
                            },
                        },
                    )?
                    .0
                    .local_id)
            })
            .unwrap();
        let mut fake = start(&store, true, Some(ticket));
        fake.activate().await;
        let (query, reply) = fake.written().await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        let key = format!("outbox-key-query:{ticket}");
        let reserved = store
            .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .expect("reserved before the write");
        reply
            .send(Err(Error::Transport(TransportError::PeerClosed)))
            .unwrap();
        assert!(
            fake.finished()
                .await
                .unwrap_err()
                .is_recoverable_by_reconnect()
        );
        // Next epoch: the reserved query is never re-sent, the reservation is unchanged.
        let mut fake = start(&store, true, Some(ticket));
        fake.activate().await;
        fake.nothing_written(Duration::from_secs(5)).await;
        assert_eq!(
            store
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap(),
            reserved
        );
        fake.stop_cleanly().await;
    }

    /// Outbox key-query reservation under a failing commit. A full disk: the
    /// reservation does not persist, nothing is written, and after reopen
    /// the query is sent once. A failed `fsync` whose write still landed: the
    /// reservation persists, so after reopen the query is never sent (it is
    /// reserved and unsettled, exactly as after a socket cut).
    #[tokio::test(start_paused = true)]
    async fn key_query_reservation_on_a_failing_disk_never_sends_unreserved_or_twice() {
        use crate::{
            model::{ids::ClientRequestId, message::MessageContent, send::SendRequest},
            outbox::Outbox,
            profile::Limits,
        };
        for fsync_only in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("reserve-fault.redb");
            let faults = crate::store::full_disk::Faults::new();
            let open_faulty = || {
                Arc::new(crate::store::full_disk::open(&path, "test", [4; 32], &faults).unwrap())
            };
            let store = open_faulty();
            let ticket = store
                .write(|tx| {
                    AccountRecord {
                        own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
                        registration: crate::session::RegistrationStatus::Registered,
                        ..Default::default()
                    }
                    .save(tx)?;
                    Ok(Outbox::new("comparison-driver", Limits::default())
                        .enqueue(
                            tx,
                            &SendRequest {
                                client_request_id: ClientRequestId::new("fault-send").unwrap(),
                                chat: ChatId::from_jid(&peer()).unwrap(),
                                content: MessageContent::Text {
                                    body: "fixture".into(),
                                    quoted: None,
                                    style: Default::default(),
                                },
                            },
                        )?
                        .0
                        .local_id)
                })
                .unwrap();
            let key = format!("outbox-key-query:{ticket}");
            // Fail exactly the commit that writes the reservation key; the
            // send-path claim and the post-activation commits before it succeed.
            faults.arm_on(key.as_bytes(), fsync_only);
            let mut fake = start(&store, true, Some(ticket));
            fake.activate().await;
            let Fake {
                mut from_worker,
                task,
                ..
            } = fake;
            let ended = tokio::time::timeout(Duration::from_secs(5), task)
                .await
                .expect("worker ends")
                .expect("worker did not panic");
            let error = ended.expect_err("the failed reservation ends the session");
            let expected = if fsync_only {
                "simulated fsync failure"
            } else {
                "simulated full disk"
            };
            assert!(error.to_string().contains(expected), "{error}");
            assert!(
                from_worker.try_recv().is_err(),
                "no query is written without a committed reservation (fsync_only={fsync_only})"
            );
            drop(store);
            faults.set_full(false);
            faults.set_sync_fails(false);
            let store = open_faulty();
            let reserved = store
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap();
            assert_eq!(reserved.is_some(), fsync_only, "fsync_only={fsync_only}");
            let mut fake = start(&store, true, Some(ticket));
            fake.activate().await;
            if fsync_only {
                // Reserved but never written: unsettled, so never re-sent.
                fake.nothing_written(Duration::from_secs(5)).await;
            } else {
                let (query, reply) = fake.written().await;
                assert_eq!(
                    query.get_attr("xmlns").and_then(|v| v.as_text()),
                    Some("encrypt")
                );
                reply.send(Ok(())).unwrap();
            }
            fake.stop_cleanly().await;
        }
    }

    fn registered_store(path: &std::path::Path) -> Arc<ProtocolStore> {
        let store = ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        })
        .unwrap();
        crate::registration::factory::install_test_material(&store, &config()).unwrap();
        Arc::new(store)
    }
    #[tokio::test]
    async fn queued_device_query_refuses_changed_or_missing_reservation() {
        let dir = tempfile::tempdir().unwrap();
        for damage in ["missing", "users", "revisions", "settled"] {
            let store =
                registered_store(&dir.path().join(format!("queued-device-owner-{damage}.db")));
            super::super::devices::request_sync(&store, None, &peer(), None, "fixture").unwrap();
            let (query, node) = super::super::devices::begin_query(
                &store,
                super::super::devices::SyncContext::Interactive,
            )
            .unwrap()
            .unwrap();
            let key = format!(
                "device-sync:{}",
                node.get_attr("id").unwrap().as_text().unwrap()
            );
            let (_incoming, incoming) = mpsc::channel(1);
            let (outgoing, mut writes) = mpsc::channel(1);
            let link = NodeLink::new(incoming, outgoing);
            let worker_store = store.clone();
            let task =
                tokio::spawn(
                    async move { link.send_device_query(&worker_store, &query, &node).await },
                );
            let queued = writes.recv().await.unwrap();
            store
                .write(|tx| {
                    if damage == "missing" {
                        tx.delete(Table::Checkpoints, key.as_bytes())?;
                    } else {
                        let mut record: super::super::devices::QueryRecord =
                            tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                        match damage {
                            "users" => record.users.clear(),
                            "revisions" => record.generations = None,
                            _ => record.status = "server_acknowledged".into(),
                        }
                        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &record)?;
                    }
                    Ok(())
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            assert!(
                queued.prepare().is_none(),
                "changed device reservation must refuse before encoding"
            );
            let error = task.await.unwrap().unwrap_err();
            assert!(
                !super::super::devices::is_expired(&error),
                "ownership failure must not be hidden as expiry"
            );
            assert!(writes.recv().await.is_none());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                    .unwrap(),
                before
            );
            assert_eq!(
                super::super::devices::pending_users(&store).unwrap().len(),
                1
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn queued_device_query_refuses_encoding_at_its_original_deadline() {
        let dir = tempfile::tempdir().unwrap();
        for delay_ms in [29_999, 30_000, 30_001] {
            let store = registered_store(&dir.path().join(format!("queued-device-{delay_ms}.db")));
            super::super::devices::request_sync(&store, None, &peer(), None, "fixture").unwrap();
            let (query, node) = super::super::devices::begin_query(
                &store,
                super::super::devices::SyncContext::Interactive,
            )
            .unwrap()
            .unwrap();
            let (_incoming, incoming) = mpsc::channel(1);
            let (outgoing, mut writes) = mpsc::channel(1);
            let link = NodeLink::new(incoming, outgoing);
            let worker_store = store.clone();
            let task =
                tokio::spawn(
                    async move { link.send_device_query(&worker_store, &query, &node).await },
                );
            let queued = writes.recv().await.unwrap();
            assert!(matches!(&queued.guard, Some(WriteGuard::Devices(..))));
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            tokio::time::advance(Duration::from_millis(delay_ms)).await;
            let mut encoded = 0;
            if let Some((node, outcome)) = queued.prepare() {
                encode(&node).unwrap();
                encoded += 1;
                outcome.send(Ok(())).unwrap();
            }
            let result = task.await.unwrap();
            if delay_ms < 30_000 {
                assert!(result.is_ok());
                assert_eq!(encoded, 1);
            } else {
                assert!(super::super::devices::is_expired(&result.unwrap_err()));
                assert_eq!(encoded, 0);
            }
            assert!(writes.recv().await.is_none());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                    .unwrap(),
                before
            );
            assert_eq!(
                super::super::devices::pending_users(&store).unwrap().len(),
                1
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn queued_prekey_upload_cannot_start_at_or_after_reply_deadline() {
        let dir = tempfile::tempdir().unwrap();
        for delay_ms in [29_999, 30_000, 30_001] {
            let path = dir.path().join(format!("queued-deadline-{delay_ms}.db"));
            let store = registered_store(&path);
            let account = store.read(AccountRecord::load).unwrap().unwrap();
            let (_incoming, incoming) = mpsc::channel(1);
            let (outgoing, mut writes) = mpsc::channel(1);
            let link = NodeLink::new(incoming, outgoing);
            let worker_store = store.clone();
            let task = tokio::spawn(async move {
                let mut pending = None;
                let mut rounds = 0;
                let result = maybe_upload(
                    &worker_store,
                    &account,
                    &link,
                    &mut pending,
                    &mut rounds,
                    PrekeyUploadTrigger::Startup,
                )
                .await;
                (result, pending.is_some(), rounds)
            });
            let queued = writes.recv().await.unwrap();
            let pending_reply = match &queued.guard {
                Some(WriteGuard::Prekey(_, pending)) => pending.clone(),
                _ => panic!("prekey guard"),
            };
            let reply = Node::new("iq")
                .attr("id", queued.node.get_attr("id").unwrap().as_text().unwrap())
                .attr("type", "result");
            let deadline = pending_reply.deadline;
            let now = tokio::time::Instant::now();
            assert_eq!(deadline.duration_since(now), Duration::from_secs(30));
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        [Table::Account, Table::Prekeys, Table::Checkpoints]
                            .into_iter()
                            .map(|t| tx.scan(t, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            tokio::time::advance(Duration::from_millis(delay_ms)).await;
            assert_eq!(tokio::time::Instant::now() >= deadline, delay_ms >= 30_000);
            let mut encoded = 0;
            if let Some((node, outcome)) = queued.prepare() {
                encode(&node).unwrap();
                encoded += 1;
                outcome.send(Ok(())).unwrap();
            }
            let (result, held, rounds) = task.await.unwrap();
            assert!(held);
            assert_eq!(rounds, 1);
            assert!(writes.recv().await.is_none(), "no replacement write");
            if delay_ms < 30_000 {
                assert!(result.is_ok());
                assert_eq!(encoded, 1);
            } else {
                assert!(
                    result.is_err(),
                    "expired queued upload {delay_ms} must refuse"
                );
                assert_eq!(encoded, 0);
            }
            if delay_ms >= 30_000 {
                assert!(
                    settle_upload_reply(&store, pending_reply, &reply)
                        .unwrap()
                        .is_none()
                );
            }
            assert_eq!(snapshot(&store), before);
            assert_eq!(
                super::super::prekeys::summary(&store).unwrap().uncertain,
                50
            );
            drop(store);
            let store = identity_store(&path);
            assert_eq!(snapshot(&store), before);
        }
    }

    #[tokio::test]
    async fn queued_prekey_upload_rechecks_authority_before_writer_encoding() {
        let dir = tempfile::tempdir().unwrap();
        for case in [
            "revoked",
            "batch-binding",
            "auth-time",
            "legacy-replaced",
            "legacy-missing",
            "legacy-consumed",
            "legacy-owner",
            "kem-replaced",
            "kem-missing",
            "kem-consumed",
            "kem-restored",
        ] {
            let store = registered_store(&dir.path().join(format!("queued-authority-{case}.db")));
            if case.starts_with("kem-") {
                let prop = super::super::abprops::queue(&store).unwrap();
                super::super::actions::reserve_pending(&store).unwrap();
                super::super::actions::record_reply(
                    &store,
                    &Node::new("iq")
                        .attr("id", prop.as_str())
                        .attr("type", "result")
                        .children(vec![Node::new("props").attr("hash", "fixture").children(
                            vec![
                                Node::new("prop")
                                    .attr("config_code", "21198")
                                    .attr("config_value", "1"),
                            ],
                        )]),
                )
                .unwrap();
            }
            let account = store.read(AccountRecord::load).unwrap().unwrap();
            let (_incoming, incoming) = mpsc::channel(1);
            let (outgoing, mut writes) = mpsc::channel(1);
            let link = NodeLink::new(incoming, outgoing);
            let worker_store = store.clone();
            let task = tokio::spawn(async move {
                let mut pending = None;
                let mut rounds = 0;
                let result = maybe_upload(
                    &worker_store,
                    &account,
                    &link,
                    &mut pending,
                    &mut rounds,
                    PrekeyUploadTrigger::Startup,
                )
                .await;
                (result, pending.is_some(), rounds)
            });
            // Hold the actual queued envelope before the shared writer preflight.
            let queued = writes.recv().await.unwrap();
            assert!(matches!(&queued.guard, Some(WriteGuard::Prekey(..))));
            let iq_id = queued
                .node
                .get_attr("id")
                .and_then(|v| v.as_text())
                .unwrap()
                .to_owned();
            store
                .write(|tx| {
                    if case.starts_with("legacy-") || case.starts_with("kem-") {
                        let batch: super::super::prekeys::Batch = tx
                            .get_json(
                                Table::Checkpoints,
                                format!("prekey-upload:{iq_id}").as_bytes(),
                                1,
                            )?
                            .unwrap();
                        if case.starts_with("legacy-") {
                            let id = batch.ids[0];
                            let key = format!("comparison-prekey:{id}");
                            let original: Vec<u8> =
                                tx.get_json(Table::Prekeys, key.as_bytes(), 1)?.unwrap();
                            match case {
                                "legacy-replaced" => {
                                    let pair = wacore_libsignal::core::curve::KeyPair::generate(
                                        &mut rand::rng(),
                                    );
                                    let raw = wacore_libsignal::protocol::PreKeyRecord::new(
                                        id.into(),
                                        &pair,
                                    )
                                    .serialize()
                                    .unwrap();
                                    tx.put_json(Table::Prekeys, key.as_bytes(), 1, &raw)?;
                                }
                                "legacy-missing" => {
                                    tx.delete(Table::Prekeys, key.as_bytes())?;
                                }
                                "legacy-consumed" => {
                                    super::super::prekeys::consume_in(tx, id, &original)?
                                }
                                _ => {
                                    let key = format!("comparison-prekey-meta:{id}");
                                    let mut meta: serde_json::Value = tx
                                        .get_json(Table::Checkpoints, key.as_bytes(), 1)?
                                        .unwrap();
                                    meta["status"] = serde_json::json!("uploaded");
                                    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &meta)?;
                                }
                            }
                        } else {
                            let rows = super::super::kem_prekeys::all_rows(&*tx)?;
                            let mut row = rows
                                .into_iter()
                                .find(|r| !r.last_resort && batch.kem_ids.contains(&r.id))
                                .unwrap();
                            let key = super::super::kem_prekeys::record_key(row.id);
                            match case {
                                "kem-replaced" => {
                                    let (public, secret) =
                                        crate::signal::kem::generate_key_pair().unwrap();
                                    row.public = public;
                                    row.secret = secret.as_bytes().to_vec();
                                    tx.put_json(Table::Prekeys, key.as_bytes(), 1, &row)?;
                                }
                                "kem-missing" => {
                                    tx.delete(Table::Prekeys, key.as_bytes())?;
                                }
                                _ => {
                                    super::super::kem_prekeys::consume_in(tx, row.id, &row)?;
                                    if case == "kem-restored" {
                                        tx.put_json(Table::Prekeys, key.as_bytes(), 1, &row)?;
                                    }
                                }
                            }
                        }
                        return Ok(());
                    }
                    if case == "batch-binding" {
                        let key = format!("prekey-upload:{iq_id}");
                        let mut batch: super::super::prekeys::Batch =
                            tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                        batch.account_authority_sha256.as_mut().unwrap()[0] ^= 1;
                        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &batch)
                    } else {
                        let mut current = AccountRecord::load_in_write(tx)?.unwrap();
                        if case == "revoked" {
                            current.registration = crate::session::RegistrationStatus::Revoked;
                        } else {
                            current.last_authenticated_at = Some(Timestamp::now());
                        }
                        current.save(tx)
                    }
                })
                .unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        [Table::Account, Table::Prekeys, Table::Checkpoints]
                            .into_iter()
                            .map(|t| tx.scan(t, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            let mut encoded = 0;
            if let Some((node, outcome)) = queued.prepare() {
                encode(&node).unwrap();
                encoded += 1;
                outcome.send(Ok(())).unwrap();
            }
            let (result, held, rounds) = task.await.unwrap();
            assert!(held);
            assert_eq!(rounds, 1);
            assert_eq!(snapshot(), before);
            if case == "auth-time" {
                assert!(result.is_ok());
                assert_eq!(encoded, 1);
            } else {
                assert!(result.is_err(), "{case} must stop queued upload");
                assert_eq!(encoded, 0, "writer must refuse before encoding");
            }
        }
    }

    #[tokio::test]
    async fn prekey_upload_distinguishes_absent_and_invalid_registered_material() {
        let dir = tempfile::tempdir().unwrap();
        for case in [
            "absent",
            "corrupt-json",
            "wrong-account",
            "signed-private-mismatch",
        ] {
            let path = dir.path().join(format!("signed-material-{case}.redb"));
            let store = registered_store(&path);
            let mut account = store.read(AccountRecord::load).unwrap().unwrap();
            store
                .write(|tx| {
                    match case {
                        "absent" => {
                            tx.delete(Table::Account, b"registration-lab-material")?;
                        }
                        "corrupt-json" => {
                            tx.put(Table::Account, b"registration-lab-material", 1, b"{")?
                        }
                        "wrong-account" => {
                            account.signal_registration_id = Some(999);
                            account.save(tx)?;
                        }
                        "signed-private-mismatch" => {
                            let mut record: serde_json::Value = tx
                                .get_json(Table::Account, b"registration-lab-material", 1)?
                                .unwrap();
                            record["signed_pre_key"]["private"] =
                                record["identity"]["private"].clone();
                            tx.put_json(Table::Account, b"registration-lab-material", 1, &record)?;
                        }
                        _ => unreachable!(),
                    }
                    Ok(())
                })
                .unwrap();
            let material = store
                .read(|tx| tx.get(Table::Account, b"registration-lab-material", 1))
                .unwrap();
            let keys = store.read(|tx| tx.scan(Table::Prekeys, 1, 1000)).unwrap();
            let mut fake = start(&store, false, None);
            fake.activate().await;
            if case == "absent" {
                fake.nothing_written(Duration::from_millis(100)).await;
                fake.stop_cleanly().await;
            } else {
                let error = fake
                    .finished()
                    .await
                    .expect_err("invalid saved material must surface");
                match case {
                    "corrupt-json" => assert!(matches!(error, Error::Storage(_)), "{error}"),
                    "wrong-account" => {
                        assert!(matches!(error, Error::InvalidInput { .. }), "{error}")
                    }
                    "signed-private-mismatch" => {
                        assert!(
                            matches!(&error, Error::InvalidInput { reason, .. } if reason == "signed prekey pair mismatch"),
                            "{error}"
                        );
                    }
                    _ => unreachable!(),
                }
            }
            assert_eq!(
                store
                    .read(|tx| tx.get(Table::Account, b"registration-lab-material", 1))
                    .unwrap(),
                material
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Prekeys, 1, 1000)).unwrap(),
                keys
            );
        }
    }

    fn assert_upload(iq: &Node) -> Vec<Vec<u8>> {
        assert_eq!(iq.description, "iq");
        assert_eq!(
            iq.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert_eq!(iq.get_attr("type").and_then(|v| v.as_text()), Some("set"));
        iq.child_nodes()
            .iter()
            .find(|c| c.description == "list")
            .unwrap()
            .child_nodes()
            .iter()
            .map(|k| {
                super::super::prekeys::child_bytes(k, "id")
                    .unwrap()
                    .to_vec()
            })
            .collect()
    }
    fn result_for(iq: &Node, count_low: bool) -> Node {
        let id = iq
            .get_attr("id")
            .and_then(|v| v.as_text())
            .unwrap()
            .to_owned();
        let mut list = Node::new("list");
        if count_low {
            list = list.attr("count_low", "true");
        }
        Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![list])
    }
    fn count_notification(id: &str, child: &str, value: &str) -> Node {
        Node::new("notification")
            .attr("type", "encrypt")
            .attr("id", id)
            .attr("from", Jid::parse("s.whatsapp.net").unwrap())
            .children(vec![Node::new(child).attr("value", value)])
    }

    #[tokio::test]
    async fn identity_refresh_reply_is_applied_by_actor_without_outbox_or_identity_ack() {
        use super::super::identity_notifications;
        use crate::session::{KeyMaterial, SignalIdentityKey};
        use wacore_libsignal::protocol::IdentityKeyPair;
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(identity_store(
            &dir.path().join("identity-application-actor.redb"),
        ));
        let local = IdentityKeyPair::generate(&mut rand::rng());
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            registration: crate::session::RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: local.public_key().public_key_bytes().try_into().unwrap(),
                private: *local.private_key().serialize(),
            })),
            ..Default::default()
        };
        let peer = peer();
        let old = IdentityKeyPair::generate(&mut rand::rng());
        store
            .write(|tx| {
                account.save(tx)?;
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &old.identity_key().serialize().to_vec(),
                )
            })
            .unwrap();
        let notification = Node::new("notification")
            .attr("type", "encrypt")
            .attr("from", peer.clone())
            .attr("id", "actor-change")
            .children(vec![Node::new("identity")]);
        let identity_notifications::Refresh::Queued(id) =
            identity_notifications::queue(&store, &account, &notification).unwrap()
        else {
            panic!("query")
        };
        let mut settings = config();
        settings.transport.identity_changes =
            crate::registration::request::IdentityChangePolicy::ApkAuthenticated;
        settings.transport.timeout_ms = 2000;
        let mut fake = start_with_config(&store, false, None, settings);
        fake.activate().await;
        let (query, written) = fake.written().await;
        assert_eq!(
            query.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        written.send(Ok(())).unwrap();
        let new = IdentityKeyPair::generate(&mut rand::rng());
        fake.deliver(identity_notifications::tests::reply(&peer, &id, &new, true))
            .await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while !identity_notifications::inspect(&store, &id)
                .unwrap()
                .unwrap()
                .applied
            {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await
        .unwrap();
        assert!(
            store
                .read(|tx| tx.scan_after(Table::Outbox, 1, None, 32))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1
                ))
                .unwrap(),
            Some(new.identity_key().serialize().to_vec())
        );
        // Native identity ACK construction/lifecycle is not implemented by this chunk.
        fake.nothing_written(Duration::from_millis(50)).await;
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn identity_notification_read_write_cut_never_replays_or_changes_keys() {
        use super::super::{actions, identity_notifications};
        use wacore_libsignal::protocol::IdentityKeyPair;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity-refresh.redb");
        let store = Arc::new(identity_store(&path));
        store
            .write(|tx| {
                AccountRecord {
                    own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
                    registration: crate::session::RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)
            })
            .unwrap();
        let peer = peer().with_device(4);
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        let pin_key = format!("comparison-direct:{peer}");
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    pin_key.as_bytes(),
                    1,
                    &identity.identity_key().serialize().to_vec(),
                )?;
                tx.put_json(
                    Table::SignalSessions,
                    pin_key.as_bytes(),
                    1,
                    &vec![1u8, 2, 3],
                )
            })
            .unwrap();
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    Ok((
                        tx.scan_after(Table::PeerIdentities, 1, None, 128)?,
                        tx.scan_after(Table::SignalSessions, 1, None, 128)?,
                        tx.scan_after(Table::Outbox, 1, None, 128)?,
                    ))
                })
                .unwrap()
        };
        let before = snapshot(&store);
        let head = store.journal_head().unwrap();
        let mut settings = config();
        settings.transport.timeout_ms = 2000;
        let mut fake = start_with_config(&store, false, None, settings.clone());
        fake.activate().await;
        let notification = Node::new("notification")
            .attr("type", "encrypt")
            .attr("id", "fixture-identity-event")
            .attr("from", peer.clone())
            .attr("participant", "15555550100@s.whatsapp.net")
            .children(vec![
                Node::new("identity"),
                Node::new("count").attr("value", "1"),
            ]);
        let wire = encode(&notification).unwrap();
        fake.deliver(notification.clone()).await;
        let (ack, ack_write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("fixture-identity-event")
        );
        // Original capture and queued read are already durable before this ACK.
        assert_eq!(
            actions::outcomes(&store).unwrap()[0].status,
            actions::ActionStatus::Queued
        );
        ack_write.send(Ok(())).unwrap();
        let (query, write) = fake.written().await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        let user = &query.child("key").unwrap().child_nodes()[0];
        assert_eq!(super::super::receipts::jid(user, "jid"), Some(peer.clone()));
        assert_eq!(
            user.get_attr("reason").and_then(|v| v.as_text()),
            Some("identity")
        );
        let query_id = query.get_attr("id").and_then(|v| v.as_text()).unwrap();
        assert_eq!(
            identity_notifications::inspect(&store, query_id)
                .unwrap()
                .unwrap()
                .refresh_guard,
            identity_notifications::RefreshGuard::Matches
        );
        let baseline_key = format!("identity-refresh-snapshot:{query_id}");
        let baseline = store
            .read(|tx| tx.get(Table::Checkpoints, baseline_key.as_bytes(), 1))
            .unwrap();
        assert!(baseline.is_some());
        // Reservation already committed before the first uncertain write.
        write
            .send(Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "fixture cut",
            ))))
            .unwrap();
        assert!(fake.finished().await.is_err());
        let outcomes = actions::outcomes(&store).unwrap();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].kind, "identity_refresh");
        assert_eq!(outcomes[0].status, actions::ActionStatus::Sent);
        let id = outcomes[0].id.clone();
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(&wire))
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        assert_eq!(snapshot(&store), before);
        assert_eq!(store.journal_head().unwrap(), head);
        drop(store);
        let store = Arc::new(identity_store(&path));
        let account = store.read(AccountRecord::load).unwrap().unwrap();
        assert_eq!(
            identity_notifications::queue(&store, &account, &notification).unwrap(),
            identity_notifications::Refresh::Queued(id.clone())
        );
        assert_eq!(
            store
                .read(|tx| tx.get(Table::Checkpoints, baseline_key.as_bytes(), 1))
                .unwrap(),
            baseline
        );
        assert_eq!(
            identity_notifications::inspect(&store, &id)
                .unwrap()
                .unwrap()
                .refresh_guard,
            identity_notifications::RefreshGuard::Matches
        );
        let mut fake = start_with_config(&store, false, None, settings);
        fake.activate().await;
        // Only the ordinary keepalive is written; no refresh replay or ACK.
        let (ping, write) = fake.written().await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        write.send(Ok(())).unwrap();
        fake.deliver(valid_keepalive_reply(
            ping.get_attr("id").and_then(|v| v.as_text()).unwrap(),
        ))
        .await;
        fake.deliver(notification).await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        write.send(Ok(())).unwrap();
        // A fresh server redelivery repeats the control ACK, never the uncertain read.
        fake.nothing_written(Duration::from_millis(100)).await;
        fake.stop.send(true).unwrap();
        fake.finished().await.unwrap();
        assert_eq!(actions::outcomes(&store).unwrap(), outcomes);
        assert_eq!(snapshot(&store), before);
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[tokio::test]
    async fn identity_ack_cut_preserves_queued_read_and_skipped_controls_do_not_read() {
        use super::super::{actions, identity_notifications};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity-ack-cut.redb");
        let store = Arc::new(identity_store(&path));
        let account = identity_notifications::tests::account();
        store.write(|tx| account.save(tx)).unwrap();
        let peer = peer();
        identity_notifications::tests::install_pin(&store, &peer);
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    Ok((
                        tx.scan_after(Table::PeerIdentities, 1, None, 128)?,
                        tx.scan_after(Table::SignalSessions, 1, None, 128)?,
                        tx.scan_after(Table::Outbox, 1, None, 128)?,
                    ))
                })
                .unwrap()
        };
        let before = snapshot(&store);
        let head = store.journal_head().unwrap();
        let mut settings = config();
        settings.transport.timeout_ms = 2000;
        let mut fake = start_with_config(&store, false, None, settings.clone());
        fake.activate().await;
        // Native dispatch ACK is independent of missing/own identity skips.
        for route in ["15555550888@s.whatsapp.net", "15555550999@s.whatsapp.net"] {
            fake.deliver(identity_notifications::tests::notification(
                route,
                "fixture-skipped",
            ))
            .await;
            let (ack, write) = fake.written().await;
            assert_eq!(ack.description, "ack");
            assert!(actions::outcomes(&store).unwrap().is_empty());
            write.send(Ok(())).unwrap();
        }
        // Malformed descriptor stays captured and does not queue a new query.
        fake.deliver(
            identity_notifications::tests::notification(&peer.to_string(), "fixture-malformed")
                .attr("participant", "private:invalid@s.whatsapp.net"),
        )
        .await;
        fake.nothing_written(Duration::from_millis(100)).await;
        let notification =
            identity_notifications::tests::notification(&peer.to_string(), "fixture-cut");
        fake.deliver(notification).await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        let outcomes = actions::outcomes(&store).unwrap();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].status, actions::ActionStatus::Queued);
        let id = outcomes[0].id.clone();
        write
            .send(Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "fixture ACK cut",
            ))))
            .unwrap();
        assert!(fake.finished().await.is_err());
        assert_eq!(snapshot(&store), before);
        assert_eq!(store.journal_head().unwrap(), head);
        drop(store);
        let store = Arc::new(identity_store(&path));
        let mut fake = start_with_config(&store, false, None, settings);
        fake.activate().await;
        // Startup dispatches the original queued read, never reconstructs the ACK.
        let (query, write) = fake.written().await;
        assert_eq!(query.description, "iq");
        assert_eq!(
            query.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        assert_eq!(
            actions::outcomes(&store).unwrap()[0].status,
            actions::ActionStatus::Sent
        );
        write.send(Ok(())).unwrap();
        fake.stop.send(true).unwrap();
        fake.finished().await.unwrap();
        assert_eq!(snapshot(&store), before);
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[tokio::test]
    async fn prekey_count_dispatch_bounds_uploads_and_skips_malformed_counts_without_ending_receive()
     {
        use super::super::prekeys::summary;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("count-dispatch.redb");
        let store = registered_store(&path);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (initial, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        fake.deliver(result_for(&initial, false)).await;
        fake.nothing_written(Duration::from_millis(100)).await;
        fake.deliver(count_notification("ignored", "count", "7").children(vec![
            Node::new("identity"),
            Node::new("count").attr("value", "7"),
        ]))
        .await;
        fake.nothing_written(Duration::from_millis(100)).await;
        // PQ-first dispatch ignores a later legacy child and never uploads.
        let mut pq = count_notification("pq", "pq_count", "٢").children(vec![
            Node::new("pq_count"),
            Node::new("count").attr("value", "7"),
        ]);
        pq.attributes.retain(|attribute| attribute.name != "from");
        fake.deliver(pq).await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("pq"));
        assert!(
            ack.get_attr("to").is_none(),
            "absent source must not fabricate an ACK destination"
        );
        write.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(100)).await;
        assert_eq!(summary(&store).unwrap().uploaded, 50);
        // The native flag is first-child count, not successful numeric parsing.
        fake.deliver(count_notification("legacy", "count", "invalid"))
            .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("legacy"));
        write.send(Ok(())).unwrap();
        let (upload, write) = fake.written().await;
        let ids = assert_upload(&upload);
        assert_eq!(ids.len(), 50);
        write.send(Ok(())).unwrap();
        fake.deliver(result_for(&upload, false)).await;
        fake.nothing_written(Duration::from_millis(100)).await;
        assert_eq!(summary(&store).unwrap().uploaded, 100);
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan_after(Table::Prekeys, 1, None, 256)?,
                    tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                ))
            })
            .unwrap();
        // APK43.72 0v6.A01 catches malformed PQ content before metadata
        // deferral, then dispatches the original ACK without upload work.
        let malformed = count_notification("malformed", "pq_count", "2147483648");
        let wire = encode(&malformed).unwrap();
        fake.deliver(malformed).await;
        let (ack, write) = fake.written().await;
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("malformed")
        );
        assert_eq!(
            ack.get_attr("class").and_then(|v| v.as_text()),
            Some("notification")
        );
        assert_eq!(
            ack.get_attr("type").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        write.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(100)).await;
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(&wire))
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan_after(Table::Prekeys, 1, None, 256)?,
                    tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 256)?
                )))
                .unwrap(),
            before
        );
        // Mixed legacy/PQ refusal cannot fall back to a legacy upload.
        let mixed = count_notification("mixed-malformed", "count", "0").children(vec![
            Node::new("count").attr("value", "0"),
            Node::new("pq_count").attr("value", "not-an-int"),
        ]);
        fake.deliver(mixed).await;
        let (ack, write) = fake.written().await;
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("mixed-malformed")
        );
        write.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(100)).await;
        assert_eq!(summary(&store).unwrap().uploaded, 100);
        // Content refusal does not relax the local ACK-envelope guards.
        let mut missing_id = count_notification("unused", "pq_count", "bad");
        missing_id
            .attributes
            .retain(|attribute| attribute.name != "id");
        fake.deliver(missing_id).await;
        fake.nothing_written(Duration::from_millis(100)).await;
        fake.deliver(
            count_notification("invalid-from", "pq_count", "bad")
                .attr("from", "1:invalid@s.whatsapp.net"),
        )
        .await;
        fake.nothing_written(Duration::from_millis(100)).await;
        // The session is still receiving: a later valid count is answered.
        fake.deliver(count_notification("after", "count", "invalid"))
            .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("after"));
        write.send(Ok(())).unwrap();
        let (upload, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        fake.deliver(result_for(&upload, false)).await;
        fake.nothing_written(Duration::from_millis(100)).await;
        let head = store.journal_head().unwrap();
        fake.stop_cleanly().await;
        drop(store);
        let store = identity_store(&path);
        assert_eq!(summary(&store).unwrap().uploaded, 150);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        assert_eq!(store.journal_head().unwrap(), head);
    }
    #[tokio::test]
    async fn prekeys_upload_at_start_follow_count_low_and_notifications_without_replaying_uncertain_batches()
     {
        use super::super::prekeys::{summary, unsent_ids};
        let dir = tempfile::tempdir().unwrap();
        let store = registered_store(&dir.path().join("prekeys.redb"));
        // Fresh store: activation generates 50 keys and uploads them once.
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (iq, reply) = fake.written().await;
        let first_ids = assert_upload(&iq);
        assert_eq!(first_ids.len(), 50);
        assert_eq!(
            summary(&store).unwrap().uncertain,
            50,
            "reserved before the write"
        );
        reply.send(Ok(())).unwrap();
        // 43.72 legacy success enters A0V: non-migrated accounts do not ADD.
        fake.deliver(result_for(&iq, true)).await;
        fake.nothing_written(Duration::from_millis(200)).await;
        fake.deliver(count_notification("FOLLOW", "count", "0"))
            .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        write.send(Ok(())).unwrap();
        let (second, reply) = fake.written().await;
        let second_ids = assert_upload(&second);
        assert_eq!(second_ids.len(), 50);
        assert!(first_ids.iter().all(|id| !second_ids.contains(id)));
        reply.send(Ok(())).unwrap();
        fake.deliver(result_for(&second, false)).await;
        fake.nothing_written(Duration::from_millis(300)).await;
        let s = summary(&store).unwrap();
        assert_eq!((s.unsent, s.uncertain, s.uploaded), (0, 0, 100));
        // pq-only notification: acknowledged, nothing uploaded.
        fake.deliver(count_notification("PQ1", "pq_count", "2"))
            .await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("type").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        reply.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(300)).await;
        // Legacy count notification: ack, generate, upload; the socket cuts on the upload write.
        fake.deliver(count_notification("N1", "count", "7")).await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("N1"));
        reply.send(Ok(())).unwrap();
        let (third, reply) = fake.written().await;
        let third_ids = assert_upload(&third);
        reply
            .send(Err(Error::Transport(TransportError::PeerClosed)))
            .unwrap();
        assert!(fake.finished().await.is_err());
        let s = summary(&store).unwrap();
        assert_eq!((s.unsent, s.uncertain, s.uploaded), (0, 50, 100));

        // Next epoch: nothing unsent and keys exist, so no startup upload.
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.nothing_written(Duration::from_millis(300)).await;
        // Another notification generates new keys; the uncertain batch is never re-sent.
        fake.deliver(count_notification("N2", "count", "3")).await;
        let (_ack, reply) = fake.written().await;
        reply.send(Ok(())).unwrap();
        let (fourth, reply) = fake.written().await;
        let fourth_ids = assert_upload(&fourth);
        assert_eq!(fourth_ids.len(), 50);
        assert!(third_ids.iter().all(|id| !fourth_ids.contains(id)));
        reply.send(Ok(())).unwrap();
        fake.deliver(result_for(&fourth, false)).await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while summary(&store).unwrap().uploaded != 150 {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        assert!(unsent_ids(&store).unwrap().is_empty());
        assert_eq!(summary(&store).unwrap().uncertain, 50);
        fake.stop_cleanly().await;
    }

    // Upload policy is independent of wall-clock CPU/scheduling delays while
    // synthetic PQ keys are generated; heartbeat timing has separate tests.
    #[tokio::test(start_paused = true)]
    async fn migrated_success_and_count_notifications_write_family_specific_add() {
        let dir = tempfile::tempdir().unwrap();
        let store = registered_store(&dir.path().join("pq-add-actor.redb"));
        let prop = super::super::abprops::queue(&store).unwrap();
        super::super::actions::reserve_pending(&store).unwrap();
        super::super::actions::record_reply(
            &store,
            &Node::new("iq")
                .attr("id", prop.as_str())
                .attr("type", "result")
                .children(vec![Node::new("props").attr("hash", "fixture").children(
                    vec![Node::new("prop").attr("config_code", "21198").attr("config_value", "1")],
                )]),
        )
        .unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (set, write) = fake.written().await;
        assert!(set.child("pq_last_resort_key").is_some());
        write.send(Ok(())).unwrap();
        fake.deliver(
            Node::new("iq")
                .attr("id", set.get_attr("id").unwrap().as_text().unwrap())
                .attr("type", "result")
                .children(vec![Node::new("pq_list").attr("count_low", "true")]),
        )
        .await;
        let (add, write) = fake.written().await;
        assert_eq!(
            add.child_nodes()
                .iter()
                .map(|n| n.description.as_str())
                .collect::<Vec<_>>(),
            ["op", "pq_list"]
        );
        assert_eq!(add.child("pq_list").unwrap().child_nodes().len(), 10);
        assert_eq!(
            add.child("op")
                .unwrap()
                .get_attr("mode")
                .and_then(|v| v.as_text()),
            Some("add")
        );
        write.send(Ok(())).unwrap();
        fake.deliver(result_for(&add, false)).await;
        fake.nothing_written(Duration::from_millis(200)).await;
        // With EMv0x70d7's declared defaultfalse, PQ-only notification is
        // ACKed without a request; result PQ demand above is independent.
        fake.deliver(count_notification("MIG-PQ", "pq_count", "0"))
            .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("MIG-PQ"));
        write.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(200)).await;
        // Optional legacy count can be unparseable: presence still requests ADD.
        fake.deliver(count_notification("MIG-LEGACY", "count", "not-an-int"))
            .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("id").and_then(|v| v.as_text()),
            Some("MIG-LEGACY")
        );
        write.send(Ok(())).unwrap();
        let (add, write) = fake.written().await;
        assert_eq!(
            add.child_nodes()
                .iter()
                .map(|n| n.description.as_str())
                .collect::<Vec<_>>(),
            ["op", "list"]
        );
        assert_eq!(add.child("list").unwrap().child_nodes().len(), 200);
        write.send(Ok(())).unwrap();
        fake.deliver(result_for(&add, false)).await;
        fake.nothing_written(Duration::from_millis(200)).await;
        fake.stop_cleanly().await;
        assert_eq!(
            super::super::prekeys::summary(&store)
                .unwrap()
                .batches
                .get("server_acknowledged"),
            Some(&3)
        );
    }

    #[tokio::test]
    async fn refused_prekey_upload_keeps_count_acks_without_retry_after_reopen() {
        use super::super::prekeys::summary;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("refused-prekey-actor.redb");
        let store = registered_store(&path);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (upload, write) = fake.written().await;
        assert_eq!(assert_upload(&upload).len(), 50);
        write.send(Ok(())).unwrap();
        fake.deliver(
            Node::new("iq")
                .attr("id", upload.get_attr("id").unwrap().as_text().unwrap())
                .attr("type", "error")
                .children(vec![Node::new("error").attr("code", "503")]),
        )
        .await;
        fake.nothing_written(Duration::from_millis(200)).await;
        assert!(summary(&store).unwrap().upload_held_after_refusal);
        let keys = store.read(|tx| tx.scan(Table::Prekeys, 1, 10000)).unwrap();
        for id in ["HELD1", "HELD2"] {
            fake.deliver(count_notification(id, "count", "0")).await;
            let (ack, write) = fake.written().await;
            assert_eq!(ack.description, "ack");
            assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some(id));
            write.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_millis(200)).await;
        }
        fake.stop_cleanly().await;
        assert_eq!(
            store.read(|tx| tx.scan(Table::Prekeys, 1, 10000)).unwrap(),
            keys
        );
        drop(store);
        // Reopen the exact synthetic account without reinstalling material.
        let store = Arc::new(
            ProtocolStore::open(OpenOptions {
                path,
                account_id: AccountId::new("test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
                create: false,
            })
            .unwrap(),
        );
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.nothing_written(Duration::from_millis(200)).await;
        fake.deliver(count_notification("HELD3", "count", "0"))
            .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("HELD3"));
        write.send(Ok(())).unwrap();
        fake.nothing_written(Duration::from_millis(200)).await;
        fake.stop_cleanly().await;
        let state = summary(&store).unwrap();
        assert!(state.upload_held_after_refusal);
        assert_eq!(state.batches.len(), 1);
        assert_eq!(state.batches.get("refused"), Some(&1));
        assert_eq!((state.unsent, state.uncertain, state.uploaded), (50, 0, 0));
        assert_eq!(
            store.read(|tx| tx.scan(Table::Prekeys, 1, 10000)).unwrap(),
            keys
        );
    }

    #[tokio::test(start_paused = true)]
    async fn newly_queued_device_work_is_picked_up_without_a_notification_or_reconnect() {
        use super::super::devices::{pending_users, request_sync};
        let dir = tempfile::tempdir().unwrap();
        let store = registered_store(&dir.path().join("late-device-work.redb"));
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (_, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        // Let activation's initial pending scan finish before adding new work.
        fake.nothing_written(Duration::from_millis(100)).await;
        request_sync(&store, None, &peer, None, "authenticated-primary-change").unwrap();
        let (query, write) = fake.written_within(Duration::from_secs(25)).await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        write.send(Ok(())).unwrap();
        let (ping, write) = fake.written().await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        write.send(Ok(())).unwrap();
        fake.deliver(valid_keepalive_reply(
            ping.get_attr("id").unwrap().as_text().unwrap(),
        ))
        .await;
        fake.deliver(device_reply(&query, vec![peer])).await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while !pending_users(&store).unwrap().is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        fake.nothing_written(Duration::from_millis(300)).await;
        fake.stop_cleanly().await;
    }

    fn device_reply(query: &Node, users: Vec<Jid>) -> Node {
        Node::new("iq")
            .attr(
                "id",
                query.get_attr("id").and_then(|v| v.as_text()).unwrap(),
            )
            .attr("type", "result")
            .children(vec![Node::new("usync").children(vec![
                    Node::new("list").children(
                        users
                            .into_iter()
                            .map(|user| {
                                Node::new("user").attr("jid", user).children(vec![
                                    Node::new("devices").children(vec![
                                        Node::new("device-list")
                                            .children(vec![Node::new("device").attr("id", "0")]),
                                    ]),
                                ])
                            })
                            .collect(),
                    ),
                ])])
    }

    #[tokio::test]
    async fn background_device_batches_advance_without_repeating_a_refused_batch() {
        use super::super::devices::{pending_users, request_sync};
        let dir = tempfile::tempdir().unwrap();
        let store = registered_store(&dir.path().join("device-batches.redb"));
        let users: Vec<Jid> = (0..130)
            .map(|i| Jid::parse(&format!("1555555{:04}@s.whatsapp.net", i + 100)).unwrap())
            .collect();
        for user in &users {
            request_sync(&store, None, user, None, "test").unwrap();
        }
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (upload, write) = fake.written().await;
        assert_eq!(
            upload.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        write.send(Ok(())).unwrap();
        let mut sizes = Vec::new();
        let mut ids = std::collections::BTreeSet::new();
        for index in 0..3 {
            let (query, write) = fake.written().await;
            assert_eq!(
                query.get_attr("xmlns").and_then(|v| v.as_text()),
                Some("usync")
            );
            let list = query.child("usync").unwrap().child("list").unwrap();
            let targets: Vec<Jid> = list
                .child_nodes()
                .iter()
                .map(|u| u.get_attr("jid").unwrap().as_jid().unwrap().clone())
                .collect();
            sizes.push(targets.len());
            assert_eq!(targets, users[index * 64..((index + 1) * 64).min(130)]);
            let id = query.get_attr("id").unwrap().as_text().unwrap().to_owned();
            assert!(ids.insert(id.clone()));
            write.send(Ok(())).unwrap();
            if index == 0 {
                fake.deliver(
                    Node::new("iq")
                        .attr("id", id)
                        .attr("type", "error")
                        .children(vec![Node::new("error").attr("code", "503")]),
                )
                .await;
            } else {
                fake.deliver(device_reply(&query, targets)).await;
            }
        }
        assert_eq!(sizes, [64, 64, 2]);
        tokio::time::timeout(Duration::from_secs(5), async {
            while pending_users(&store).unwrap().len() != 64 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(pending_users(&store).unwrap(), users[..64]);
        fake.nothing_written(Duration::from_millis(300)).await;
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn newer_device_notification_survives_inflight_reply_and_gets_a_fresh_query() {
        use super::super::devices::{pending_users, query_record, request_sync};
        let dir = tempfile::tempdir().unwrap();
        let store = registered_store(&dir.path().join("device-newer.redb"));
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        request_sync(&store, None, &peer, Some("2:old"), "test").unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (_, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        let (old, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        let old_id = old.get_attr("id").unwrap().as_text().unwrap().to_owned();
        fake.deliver(
            Node::new("notification")
                .attr("type", "devices")
                .attr("id", "D-NEW")
                .attr("from", peer.clone())
                .children(vec![
                    Node::new("add")
                        .attr("device_hash", "2:new")
                        .children(vec![Node::new("device").attr("jid", peer.with_device(3))]),
                ]),
        )
        .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        write.send(Ok(())).unwrap();
        fake.deliver(device_reply(&old, vec![peer.clone()])).await;
        let (fresh, write) = fake.written().await;
        assert_eq!(
            fresh.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        assert_eq!(
            fresh
                .child("usync")
                .unwrap()
                .get_attr("context")
                .and_then(|v| v.as_text()),
            Some("notification")
        );
        let fresh_id = fresh.get_attr("id").unwrap().as_text().unwrap().to_owned();
        assert_ne!(old_id, fresh_id);
        assert_eq!(
            query_record(&store, &old_id)
                .unwrap()
                .unwrap()
                .superseded_users,
            Some(1)
        );
        assert_eq!(
            query_record(&store, &fresh_id)
                .unwrap()
                .unwrap()
                .generations,
            Some(vec![2])
        );
        assert_eq!(
            pending_users(&store).unwrap().as_slice(),
            std::slice::from_ref(&peer)
        );
        write.send(Ok(())).unwrap();
        fake.deliver(device_reply(&fresh, vec![peer])).await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while !pending_users(&store).unwrap().is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            query_record(&store, &fresh_id)
                .unwrap()
                .unwrap()
                .superseded_users,
            Some(0)
        );
        fake.nothing_written(Duration::from_millis(300)).await;
        fake.stop_cleanly().await;
    }

    #[tokio::test(start_paused = true)]
    async fn unanswered_device_batch_is_deferred_while_other_users_advance() {
        use super::super::devices::{pending_users, query_record, request_sync};
        let dir = tempfile::tempdir().unwrap();
        let store = registered_store(&dir.path().join("device-timeout.redb"));
        let users: Vec<Jid> = (0..65)
            .map(|i| Jid::parse(&format!("1555555{:04}@s.whatsapp.net", i + 100)).unwrap())
            .collect();
        for user in &users {
            request_sync(&store, None, user, None, "test").unwrap();
        }
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (_, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        let (first, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        let first_id = first.get_attr("id").unwrap().as_text().unwrap().to_owned();
        // Service keepalives while the first read remains unanswered.
        let second = loop {
            let (node, write) = fake.written_within(Duration::from_secs(60)).await;
            write.send(Ok(())).unwrap();
            if node.get_attr("xmlns").and_then(|v| v.as_text()) == Some("usync") {
                break node;
            }
            assert_eq!(
                node.get_attr("xmlns").and_then(|v| v.as_text()),
                Some("w:p")
            );
            fake.deliver(valid_keepalive_reply(
                node.get_attr("id").unwrap().as_text().unwrap(),
            ))
            .await;
        };
        let targets = second
            .child("usync")
            .unwrap()
            .child("list")
            .unwrap()
            .child_nodes();
        assert_eq!(targets.len(), 1);
        assert_eq!(
            targets[0].get_attr("jid").unwrap().as_jid(),
            Some(&users[64])
        );
        assert_ne!(
            second.get_attr("id").unwrap().as_text(),
            Some(first_id.as_str())
        );
        // The same heartbeat writes its ping after the newly selected batch.
        let (ping, write) = fake.written().await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        write.send(Ok(())).unwrap();
        fake.deliver(valid_keepalive_reply(
            ping.get_attr("id").unwrap().as_text().unwrap(),
        ))
        .await;
        fake.deliver(device_reply(&second, vec![users[64].clone()]))
            .await;
        // The timeout and a late old reply do not revive/replay its reservation.
        fake.deliver(device_reply(&first, users[..64].to_vec()))
            .await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while pending_users(&store).unwrap().len() != 64 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            query_record(&store, &first_id).unwrap().unwrap().status,
            "outcome_unknown"
        );
        assert_eq!(pending_users(&store).unwrap(), users[..64]);
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn device_notification_ack_write_cut_keeps_pending_sync_for_restart() {
        use super::super::devices::pending_users;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("device-ack-cut.redb");
        let store = registered_store(&path);
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (_, write) = fake.written().await;
        write.send(Ok(())).unwrap();
        fake.deliver(
            Node::new("notification")
                .attr("type", "devices")
                .attr("id", "D-CUT")
                .attr("from", peer.clone())
                .children(vec![Node::new("add").children(vec![
                    Node::new("device").attr("jid", peer.with_device(3)),
                ])]),
        )
        .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            pending_users(&store).unwrap().as_slice(),
            std::slice::from_ref(&peer)
        );
        write
            .send(Err(Error::Transport(TransportError::PeerClosed)))
            .unwrap();
        assert!(fake.finished().await.is_err());
        let account_before =
            serde_json::to_value(store.read(AccountRecord::load).unwrap()).unwrap();
        drop(store);
        let store = Arc::new(
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
                create: false,
            })
            .unwrap(),
        );
        assert_eq!(
            serde_json::to_value(store.read(AccountRecord::load).unwrap()).unwrap(),
            account_before
        );
        assert_eq!(
            pending_users(&store).unwrap().as_slice(),
            std::slice::from_ref(&peer)
        );
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (query, write) = fake.written().await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        write.send(Ok(())).unwrap();
        fake.deliver(device_reply(&query, vec![peer])).await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while !pending_users(&store).unwrap().is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        fake.nothing_written(Duration::from_millis(300)).await;
        fake.stop_cleanly().await;
    }

    #[tokio::test]
    async fn devices_notifications_are_acknowledged_queried_ingested_and_requeried_after_a_cut() {
        use super::super::devices::{DeviceEntry, device_list, pending_users};
        let dir = tempfile::tempdir().unwrap();
        let store = registered_store(&dir.path().join("devices.redb"));
        let contact = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let private = [13u8; 32];
        let (signed, key) =
            crate::signal::adv::test_signed_list(7, 1_700_000_000, 2, &[1, 2], &private);
        let mut pinned = vec![0x05];
        pinned.extend_from_slice(&key);
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{contact}").as_bytes(),
                    1,
                    &pinned,
                )
            })
            .unwrap();
        let notification = |id: &str| {
            Node::new("notification")
                .attr("type", "devices")
                .attr("id", id)
                .attr("from", contact.clone())
                .children(vec![Node::new("add").attr("device_hash", "2:x").children(
                    vec![
                    Node::new("device")
                        .attr("jid", contact.with_device(3))
                        .attr("key-index", "1"),
                ],
                )])
        };
        let mut fake = start(&store, false, None);
        fake.activate().await;
        // The first upload of prekeys happens at activation for this registered store.
        let (upload, reply) = fake.written().await;
        assert_eq!(
            upload.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        reply.send(Ok(())).unwrap();
        fake.deliver(notification("D1")).await;
        let (ack, reply) = fake.written().await;
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("type").and_then(|v| v.as_text()),
            Some("devices")
        );
        reply.send(Ok(())).unwrap();
        let (query, reply) = fake.written().await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        let usync = &query.child_nodes()[0];
        assert_eq!(
            usync.get_attr("context").and_then(|v| v.as_text()),
            Some("notification")
        );
        // The socket cuts on the query write: the user stays pending.
        reply
            .send(Err(Error::Transport(TransportError::PeerClosed)))
            .unwrap();
        assert!(fake.finished().await.is_err());
        assert_eq!(pending_users(&store).unwrap(), vec![contact.clone()]);
        assert!(device_list(&store, &contact).unwrap().is_none());

        // Next epoch: activation re-queries interactively; the reply is ingested.
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (query, reply) = fake.written().await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        assert_eq!(
            query.child_nodes()[0]
                .get_attr("context")
                .and_then(|v| v.as_text()),
            Some("interactive")
        );
        reply.send(Ok(())).unwrap();
        let result = Node::new("iq")
            .attr(
                "id",
                query.get_attr("id").and_then(|v| v.as_text()).unwrap(),
            )
            .attr("type", "result")
            .children(vec![Node::new("usync").children(vec![
                Node::new("result"),
                Node::new("list").children(
                    vec![Node::new("user").attr("jid", contact.clone()).children(vec![
                    Node::new("devices").children(vec![
                        Node::new("device-list").children(vec![
                            Node::new("device").attr("id", "0"),
                            Node::new("device").attr("id", "3").attr("key-index", "1"),
                        ]),
                        Node::new("key-index-list").attr("ts", "1700000000").bytes(signed),
                    ]),
                ])],
                ),
            ])]);
        fake.deliver(result).await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while device_list(&store, &contact).unwrap().is_none() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        let list = device_list(&store, &contact).unwrap().unwrap();
        assert!(list.companions_verified);
        assert_eq!(
            list.devices,
            vec![
                DeviceEntry {
                    jid: contact.clone(),
                    key_index: 0
                },
                DeviceEntry {
                    jid: contact.with_device(3),
                    key_index: 1
                }
            ]
        );
        assert!(pending_users(&store).unwrap().is_empty());
        fake.nothing_written(Duration::from_millis(300)).await;
        fake.stop_cleanly().await;
    }

    fn valid_keepalive_reply(id: impl Into<crate::binary::AttributeValue>) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .attr("from", Jid::parse("s.whatsapp.net").unwrap())
            .attr("t", "1700000000")
    }

    #[test]
    fn keepalive_result_schema_requires_native_fields_and_timestamp_range() {
        let valid = valid_keepalive_reply("ping");
        assert_eq!(validate_keepalive_result(&valid, "ping"), Ok(()));
        for field in ["from", "id", "t", "type"] {
            let mut missing = valid.clone();
            missing.attributes.retain(|a| a.name != field);
            assert_eq!(validate_keepalive_result(&missing, "ping"), Err(field));
        }
        for (text, accepted) in [
            ("1577865600", true),
            ("4102473600", true),
            ("1577865599", false),
            ("4102473601", false),
            ("+1700000000", true),
            ("١٧٠٠٠٠٠٠٠٠", true),
            ("", false),
            (" 1700000000", false),
            ("1700000000.0", false),
            ("0x6553f100", false),
            ("9223372036854775808", false),
            ("-9223372036854775808", false),
            ("1700000000000", false),
        ] {
            let mut reply = valid.clone();
            reply.attributes.retain(|a| a.name != "t");
            reply = reply.attr("t", text);
            assert_eq!(
                validate_keepalive_result(&reply, "ping").is_ok(),
                accepted,
                "{text}"
            );
        }
        let mut text_from = valid.clone();
        text_from.attributes.retain(|a| a.name != "from");
        assert_eq!(
            validate_keepalive_result(&text_from.clone().attr("from", "s.whatsapp.net"), "ping"),
            Ok(())
        );
        // The selected callback admits Jid, not just the server constant.
        assert_eq!(
            validate_keepalive_result(
                &text_from.clone().attr("from", "120363012345678901@g.us"),
                "ping"
            ),
            Ok(())
        );
        assert_eq!(
            validate_keepalive_result(&text_from.attr("from", "bad jid"), "ping"),
            Err("from")
        );
        assert_eq!(validate_keepalive_result(&valid, "other"), Err("id"));
    }

    #[tokio::test(start_paused = true)]
    async fn delayed_ping_response_restarts_only_ping_schedule_not_maintenance() {
        use super::super::{about, actions};
        for outcome in ["result", "error"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("ping-schedule.redb"));
            let mut fake = start(&store, false, None);
            fake.activate().await;
            let (first, written) = fake.written_within(Duration::from_secs(60)).await;
            written.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_secs(19)).await;
            let id = first.get_attr("id").unwrap().clone();
            let reply = if outcome == "result" {
                valid_keepalive_reply(id)
            } else {
                Node::new("iq").attr("id", id).attr("type", "error")
            };
            let responded_at = tokio::time::Instant::now();
            fake.deliver(reply).await;
            // This response proves the preceding client reply was processed.
            fake.deliver(
                Node::new("iq")
                    .attr("type", "get")
                    .attr("xmlns", "urn:xmpp:ping"),
            )
            .await;
            let (pong, written) = fake.written().await;
            assert_eq!(
                pong.get_attr("type").and_then(|v| v.as_text()),
                Some("result")
            );
            written.send(Ok(())).unwrap();
            let action_id = about::queue_set(&store, "Available").unwrap();
            // The old maintenance tick is still one second away. It must drain
            // the action, while the next client ping waits a full response interval.
            let (action, written) = fake.written_within(Duration::from_secs(2)).await;
            assert_eq!(
                action.get_attr("id").and_then(|v| v.as_text()),
                Some(action_id.as_str())
            );
            written.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_secs(18)).await;
            let (second, written) = fake.written_within(Duration::from_secs(2)).await;
            assert_eq!(
                second.get_attr("xmlns").and_then(|v| v.as_text()),
                Some("w:p")
            );
            assert_ne!(second.get_attr("id"), first.get_attr("id"));
            assert_eq!(
                tokio::time::Instant::now() - responded_at,
                Duration::from_secs(20)
            );
            written.send(Ok(())).unwrap();
            fake.stop_cleanly().await;
            assert_eq!(
                actions::outcomes(&store).unwrap()[0].status,
                actions::ActionStatus::Sent
            );
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
        }
    }

    #[tokio::test(start_paused = true)]
    async fn previous_link_ping_replies_cannot_settle_or_extend_restarted_ping() {
        // APK43.72 0m2 case 3 clears the awaiting-response IQ registry on
        // disconnect. Rust owns claims per drive, independently of saved state.
        for outcome in ["result", "error"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("ping-restart.redb");
            let store = open(&path);
            let mut first = start(&store, false, None);
            first.activate().await;
            let (old_ping, written) = first.written_within(Duration::from_secs(60)).await;
            let old_id = old_ping.get_attr("id").unwrap().clone();
            written.send(Ok(())).unwrap();
            first
                .cut(Error::Transport(TransportError::PeerClosed))
                .await;
            assert!(
                first
                    .finished()
                    .await
                    .unwrap_err()
                    .is_recoverable_by_reconnect()
            );
            drop(store);

            let store = open(&path);
            let mut restarted = start(&store, false, None);
            restarted.activate().await;
            let (current, written) = restarted.written_within(Duration::from_secs(60)).await;
            assert_ne!(current.get_attr("id"), Some(&old_id));
            let admitted_at = tokio::time::Instant::now();
            written.send(Ok(())).unwrap();
            restarted.nothing_written(Duration::from_secs(19)).await;
            let stale = if outcome == "result" {
                valid_keepalive_reply(old_id)
            } else {
                Node::new("iq").attr("id", old_id).attr("type", "error")
            };
            restarted.deliver(stale).await;
            // A separate server ping proves receive processed the stale reply.
            restarted
                .deliver(
                    Node::new("iq")
                        .attr("type", "get")
                        .attr("xmlns", "urn:xmpp:ping"),
                )
                .await;
            let (pong, written) = restarted.written().await;
            assert_eq!(
                pong.get_attr("type").and_then(|v| v.as_text()),
                Some("result")
            );
            written.send(Ok(())).unwrap();
            let error = restarted
                .finished_within(Duration::from_secs(2))
                .await
                .unwrap_err();
            assert!(matches!(error, Error::Timeout(_)), "{error:?}");
            assert_eq!(
                tokio::time::Instant::now() - admitted_at,
                Duration::from_secs(20)
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn malformed_keepalive_result_consumes_callback_without_clearing_timer() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("ping-malformed.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (ping, written) = fake.written_within(Duration::from_secs(60)).await;
        let id = ping.get_attr("id").unwrap().clone();
        written.send(Ok(())).unwrap();
        fake.deliver(
            Node::new("iq")
                .attr("id", id.clone())
                .attr("type", "result"),
        )
        .await;
        // Native one-shot ownership is gone: even a later valid same-ID result
        // or error cannot invoke this callback and clear the outstanding timer.
        fake.deliver(valid_keepalive_reply(id.clone())).await;
        fake.deliver(Node::new("iq").attr("id", id).attr("type", "error"))
            .await;
        fake.deliver(
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        )
        .await;
        let (pong, written) = fake.written().await;
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        written.send(Ok(())).unwrap();
        let error = fake
            .finished_within(Duration::from_secs(60))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Timeout(_)));
    }

    #[tokio::test(start_paused = true)]
    async fn keepalive_error_clears_only_its_live_ping_without_ending_receive() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("ping-error.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let before_clock =
            serde_json::to_value(super::super::clock::inspect(&store).unwrap()).unwrap();
        let (ping, written) = fake.written_within(Duration::from_secs(60)).await;
        let first_id = ping.get_attr("id").unwrap().clone();
        written.send(Ok(())).unwrap();
        let error = Node::new("iq")
            .attr("id", first_id.clone())
            .attr("type", "error")
            .attr("t", "1900000000")
            .children(vec![
                Node::new("error")
                    .attr("code", "503")
                    .attr("text", "fixture-only"),
            ]);
        fake.deliver(error.clone()).await;
        let (next, written) = fake.written_within(Duration::from_secs(60)).await;
        assert_eq!(
            next.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        assert_ne!(next.get_attr("id"), Some(&first_id));
        written.send(Ok(())).unwrap();
        // An old duplicate cannot consume the successor's pending response.
        fake.deliver(error).await;
        fake.deliver(
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        )
        .await;
        let (pong, written) = fake.written().await;
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        written.send(Ok(())).unwrap();
        assert_eq!(
            serde_json::to_value(super::super::clock::inspect(&store).unwrap()).unwrap(),
            before_clock
        );
        // If the stale error cleared the successor, this deadline would not terminate.
        let error = fake
            .finished_within(Duration::from_secs(60))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Timeout(_)));
        assert!(error.is_recoverable_by_reconnect());
    }

    #[tokio::test(start_paused = true)]
    async fn non_reply_iqs_leave_keepalive_pending_until_real_reply_or_timeout() {
        for outcome in ["result", "error", "timeout"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("ping-nonreply.redb"));
            let mut fake = start(&store, false, None);
            fake.activate().await;
            let (ping, written) = fake.written_within(Duration::from_secs(60)).await;
            let id = ping.get_attr("id").unwrap().clone();
            written.send(Ok(())).unwrap();
            for kind in [
                None,
                Some("get"),
                Some("set"),
                Some("RESULT"),
                Some("unexpected"),
            ] {
                let mut node = Node::new("iq")
                    .attr("id", id.clone())
                    .children(vec![Node::new("error").attr("code", "503")]);
                if let Some(kind) = kind {
                    node = node.attr("type", kind);
                }
                let capture = format!(
                    "inbound:sha256:{}",
                    hex::encode(sha2::Sha256::digest(encode(&node).unwrap()))
                );
                fake.deliver(node).await;
                fake.deliver(
                    Node::new("iq")
                        .attr("type", "get")
                        .attr("xmlns", "urn:xmpp:ping"),
                )
                .await;
                let (pong, written) = fake.written().await;
                assert_eq!(
                    pong.get_attr("type").and_then(|v| v.as_text()),
                    Some("result")
                );
                assert!(
                    store
                        .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                        .unwrap()
                        .is_some()
                );
                written.send(Ok(())).unwrap();
            }
            if outcome != "timeout" {
                let reply = if outcome == "result" {
                    valid_keepalive_reply(id)
                } else {
                    Node::new("iq").attr("id", id).attr("type", outcome)
                };
                fake.deliver(reply).await;
            }
            if outcome != "timeout" {
                let (next_ping, written) = fake.written_within(Duration::from_secs(60)).await;
                assert_eq!(
                    next_ping.get_attr("xmlns").and_then(|v| v.as_text()),
                    Some("w:p")
                );
                written.send(Ok(())).unwrap();
                fake.stop_cleanly().await;
            } else {
                let error = fake
                    .finished_within(Duration::from_secs(60))
                    .await
                    .unwrap_err();
                assert!(matches!(error, Error::Timeout(_)));
                assert!(error.is_recoverable_by_reconnect());
            }
        }
    }

    #[tokio::test]
    async fn cancelled_group_preparation_settles_owned_reply_without_followup_or_disconnect() {
        use super::super::{group_fanout, outgoing_group};
        use outgoing_group::tests::{device_reply, enqueue, key_reply, membership};
        for cancel_stage in ["membership", "devices", "keys"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("cancelled-group.redb");
            let store = open(&path);
            let f = group_fanout::tests::fixture();
            store.write(|tx| f.alice.save(tx)).unwrap();
            let ticket = enqueue(&store, &f.plan.chat, "cancelled-group");
            let outbox =
                crate::outbox::Outbox::new("fixture-namespace", crate::profile::Limits::default());
            let mut fake = start(&store, true, Some(ticket));
            fake.activate().await;
            let mut cancelled_entry = None;
            let mut cancelled_query_id = String::new();
            for stage in ["membership", "devices", "keys"] {
                let (query, written) = fake.written().await;
                assert_eq!(
                    query.get_attr("xmlns").and_then(|v| v.as_text()),
                    Some(match stage {
                        "membership" => "w:g2",
                        "devices" => "usync",
                        _ => "encrypt",
                    })
                );
                written.send(Ok(())).unwrap();
                let reply = match stage {
                    "membership" => membership(&query, &f.plan.own, &f.plan.recipients),
                    "devices" => device_reply(&query),
                    _ => key_reply(&query, &f),
                };
                if stage == cancel_stage {
                    assert!(store.write(|tx| outbox.cancel(tx, ticket)).unwrap());
                    cancelled_entry = store.read(|tx| outbox.get(tx, ticket)).unwrap();
                    cancelled_query_id =
                        query.get_attr("id").unwrap().as_text().unwrap().to_owned();
                }
                fake.deliver(reply).await;
                if stage == cancel_stage {
                    break;
                }
            }
            // Prove the same actor still services receive and did not release
            // another preparation query or encrypted message after cancellation.
            fake.deliver(
                Node::new("iq")
                    .attr("type", "get")
                    .attr("xmlns", "urn:xmpp:ping"),
            )
            .await;
            let (pong, written) = fake.written().await;
            assert_eq!(
                pong.get_attr("type").and_then(|v| v.as_text()),
                Some("result")
            );
            written.send(Ok(())).unwrap();
            fake.nothing_written(Duration::from_millis(20)).await;
            fake.stop_cleanly().await;
            let summary = outgoing_group::preparation(&store, ticket)
                .unwrap()
                .unwrap();
            assert_eq!(summary.blocking_reason.as_deref(), Some("intent_cancelled"));
            assert_eq!(
                store.read(|tx| outbox.get(tx, ticket)).unwrap(),
                cancelled_entry
            );
            assert!(cancelled_entry.unwrap().attempts.is_empty());
            if cancel_stage == "membership" {
                let outcome = store
                    .read(|tx| super::super::actions::view_in(tx, &cancelled_query_id))
                    .unwrap()
                    .unwrap()
                    .0;
                assert_eq!(outcome.status, super::super::actions::ActionStatus::Ok);
            }
            if cancel_stage == "devices" {
                assert_eq!(
                    super::super::devices::query_record(&store, &cancelled_query_id)
                        .unwrap()
                        .unwrap()
                        .status,
                    "server_acknowledged"
                );
            }
            if cancel_stage == "keys" {
                let batches = summary.key_batches.unwrap();
                assert_eq!(batches.outcome_unknown, 0);
                assert!(batches.complete);
            }
            for table in [
                Table::SignalSessions,
                Table::PeerIdentities,
                Table::SenderKeys,
            ] {
                assert!(store.read(|tx| tx.scan(table, 1, 100)).unwrap().is_empty());
            }
            assert!(
                outgoing_group::begin_selected(&store, ticket)
                    .unwrap()
                    .is_none()
            );
            drop(store);
            let reopened = identity_store(&path);
            assert_eq!(
                outgoing_group::preparation(&reopened, ticket)
                    .unwrap()
                    .unwrap()
                    .blocking_reason
                    .as_deref(),
                Some("intent_cancelled")
            );
            assert!(
                outgoing_group::begin_selected(&reopened, ticket)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn group_preparation_deadline_cannot_advance_on_a_late_device_result() {
        use super::super::{group_fanout, outgoing_group};
        use outgoing_group::tests::{device_reply, enqueue, membership};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("group-preparation-deadline.redb");
        let store = open(&path);
        let f = group_fanout::tests::fixture();
        store.write(|tx| f.alice.save(tx)).unwrap();
        let ticket = enqueue(&store, &f.plan.chat, "group-deadline");
        let mut fake = start(&store, true, Some(ticket));
        fake.activate().await;
        let (info, written) = fake.written().await;
        assert_eq!(
            info.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:g2")
        );
        written.send(Ok(())).unwrap();
        // Reset the next stage's30s deadline between the1s outbox ticks.
        tokio::time::advance(Duration::from_millis(500)).await;
        fake.deliver(membership(&info, &f.plan.own, &f.plan.recipients))
            .await;
        let (devices, written) = fake.written().await;
        assert_eq!(
            devices.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        written.send(Ok(())).unwrap();
        let reserved = outgoing_group::preparation(&store, ticket)
            .unwrap()
            .unwrap();
        let (ping, written) = fake.written_within(Duration::from_secs(25)).await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        written.send(Ok(())).unwrap();
        fake.deliver(result_for(&ping, false)).await;
        // Service the due outbox tick at30s, then enter its gap before31s.
        tokio::time::advance(Duration::from_secs(10)).await;
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(750)).await;
        tokio::task::yield_now().await;
        let late = device_reply(&devices);
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&late).unwrap()))
        );
        fake.deliver(late).await;
        fake.deliver(
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        )
        .await;
        let (pong, written) = fake.written().await;
        // Old code advanced to an encrypt query instead of this receive barrier.
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        written.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
        let after = outgoing_group::preparation(&store, ticket)
            .unwrap()
            .unwrap();
        assert_eq!(after.phase, "blocked");
        assert_eq!(after.blocking_reason.as_deref(), Some("outcome_unknown"));
        assert_eq!(after.key_queries, reserved.key_queries);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        drop(store);
        let store = identity_store(&path);
        assert_eq!(
            outgoing_group::preparation(&store, ticket)
                .unwrap()
                .unwrap(),
            after
        );
    }

    #[tokio::test(start_paused = true)]
    async fn late_prekey_reply_cannot_settle_between_deadline_and_heartbeat() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prekey-deadline.redb");
        let store = registered_store(&path);
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (upload, written) = fake.written().await;
        assert_eq!(assert_upload(&upload).len(), 50);
        written.send(Ok(())).unwrap();
        let (ping, written) = fake.written_within(Duration::from_secs(25)).await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        written.send(Ok(())).unwrap();
        fake.deliver(result_for(&ping, false)).await;
        tokio::time::advance(Duration::from_secs(11)).await;
        tokio::task::yield_now().await;
        // This result arrives at31s: after its30s deadline but before the40s tick.
        let late = result_for(&upload, false);
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&late).unwrap()))
        );
        fake.deliver(late).await;
        fake.deliver(
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        )
        .await;
        let (pong, written) = fake.written().await;
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        written.send(Ok(())).unwrap();
        fake.stop_cleanly().await;
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        let state = super::super::prekeys::summary(&store).unwrap();
        assert_eq!(state.uncertain, 50);
        assert_eq!(state.uploaded, 0);
        assert_eq!(state.batches.get("outcome_unknown"), Some(&1));
        drop(store);
        let store = identity_store(&path);
        assert_eq!(
            super::super::prekeys::summary(&store).unwrap().uncertain,
            50
        );
    }

    #[tokio::test(start_paused = true)]
    async fn late_device_reply_cannot_settle_between_deadline_and_heartbeat() {
        use super::super::devices::{pending_users, query_record, request_sync};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("device-deadline.redb");
        let store = open(&path);
        request_sync(&store, None, &peer(), None, "test").unwrap();
        let mut fake = start(&store, false, None);
        fake.activate().await;
        let (query, written) = fake.written().await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        let id = query.get_attr("id").unwrap().as_text().unwrap().to_owned();
        let before = query_record(&store, &id).unwrap().unwrap();
        written.send(Ok(())).unwrap();
        let (ping, written) = fake.written_within(Duration::from_secs(25)).await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        written.send(Ok(())).unwrap();
        fake.deliver(result_for(&ping, false)).await;
        tokio::time::advance(Duration::from_secs(11)).await;
        tokio::task::yield_now().await;
        // A late error must not turn the reserved read into a server refusal.
        let late = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "error")
            .children(vec![Node::new("error").attr("code", "503")]);
        let capture = format!(
            "inbound:sha256:{}",
            hex::encode(sha2::Sha256::digest(encode(&late).unwrap()))
        );
        fake.deliver(late).await;
        fake.deliver(
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        )
        .await;
        let (pong, written) = fake.written().await;
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        written.send(Ok(())).unwrap();
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        assert_eq!(query_record(&store, &id).unwrap().unwrap(), before);
        assert_eq!(pending_users(&store).unwrap(), vec![peer()]);
        fake.stop_cleanly().await;
        drop(store);
        let store = identity_store(&path);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        assert_eq!(query_record(&store, &id).unwrap().unwrap(), before);
        assert_eq!(pending_users(&store).unwrap(), vec![peer()]);
    }

    #[tokio::test(start_paused = true)]
    async fn delayed_heartbeat_work_does_not_shorten_the_ping_response_window() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("ping-phase.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        (&mut fake.ready).await.unwrap();
        // Queue after activation so the next heartbeat discovers this read.
        super::super::devices::request_sync(&store, None, &peer(), None, "test").unwrap();
        let (query, written) = fake.written_within(Duration::from_secs(60)).await;
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("usync")
        );
        // The heartbeat has ticked, but its maintenance write blocks the ping.
        tokio::time::advance(Duration::from_secs(19)).await;
        written.send(Ok(())).unwrap();
        let (ping, written) = fake.written().await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        written.send(Ok(())).unwrap();
        let sent_at = tokio::time::Instant::now();
        // At the next heartbeat the ping is only one second old. A separate
        // inbound ping is a receive-loop barrier, not an answer to our ping.
        tokio::time::advance(Duration::from_secs(1)).await;
        fake.deliver(
            Node::new("iq")
                .attr("id", "barrier")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping"),
        )
        .await;
        let (pong, written) = fake.written().await;
        assert_eq!(
            pong.get_attr("type").and_then(|v| v.as_text()),
            Some("result")
        );
        assert!(pong.get_attr("id").is_none());
        written.send(Ok(())).unwrap();
        // Silence still expires at this ping's own 20-second deadline.
        let error = fake
            .finished_within(Duration::from_secs(30))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Timeout(_)));
        assert!(error.is_recoverable_by_reconnect());
        assert_eq!(
            tokio::time::Instant::now() - sent_at,
            Duration::from_secs(20)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn unanswered_keepalive_times_out_as_recoverable_and_answered_keepalive_continues() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("keepalive.redb"));
        let mut fake = start(&store, false, None);
        fake.activate().await;
        // Heartbeats are 20 s apart under the template timeout; paused time skips ahead.
        let (ping, reply) = fake.written_within(Duration::from_secs(60)).await;
        assert_eq!(
            ping.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        reply.send(Ok(())).unwrap();
        let id = ping
            .get_attr("id")
            .and_then(|v| v.as_text())
            .unwrap()
            .to_owned();
        fake.deliver(valid_keepalive_reply(id.as_str())).await;
        let (second, reply) = fake.written_within(Duration::from_secs(60)).await;
        assert_eq!(
            second.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("w:p")
        );
        reply.send(Ok(())).unwrap();
        // No answer before the next heartbeat: typed timeout the actor may retry.
        let err = fake
            .finished_within(Duration::from_secs(60))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Timeout(_)), "{err:?}");
        assert!(err.is_recoverable_by_reconnect());
    }
}
