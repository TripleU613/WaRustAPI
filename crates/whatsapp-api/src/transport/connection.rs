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
    config.validate()?;
    let account = store
        .read(AccountRecord::load)?
        .ok_or(Error::NotRegistered)?;
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
    let key = account
        .noise_static
        .as_ref()
        .ok_or_else(|| Error::invalid("connection", "missing saved Noise identity"))?;
    let local = StaticKeypair::from_parts(key.0.private, key.0.public);
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
    let payload = super::login::primary_login(&account, config)?;
    store.write(|tx| {
        let previous: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, LOGIN_KEY, 1)?;
        if previous.is_some_and(|v| v["status"] != "success") {
            return Err(Error::invalid("connection", "login already reserved"));
        }
        tx.put_json(
            Table::Checkpoints,
            LOGIN_KEY,
            1,
            &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now()}),
        )
    })?;
    let mut channel = verified.finish(&payload).await?;
    let reply = super::messaging::read_node(&mut channel).await?;
    super::inbox::preserve_in_epoch(store, &reply, epoch)?;
    record_login_reply(store, &reply)?;
    Ok(channel)
}

// One transaction commits the login result, clock sample and own identity.
fn record_login_reply(store: &ProtocolStore, reply: &Node) -> Result<()> {
    let status = if reply.description == "success" {
        "success"
    } else {
        "refused"
    };
    store.write(|tx| {
        tx.put_json(
            Table::Checkpoints,
            LOGIN_KEY,
            1,
            &serde_json::json!({"status":status,"at":Timestamp::now()}),
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
        return Err(Error::Authentication { code: None });
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

fn update_login_identity(
    tx: &mut crate::store::WriteTx<'_>,
    reply: &Node,
) -> Result<Option<crate::model::jid::Jid>> {
    use crate::model::jid::{Jid, Server};
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
pub(crate) type Outbound = (Node, oneshot::Sender<Result<()>>);

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
    /// Write one stanza and wait for the socket outcome. A closed writer is a
    /// consumed connection: the caller must end this epoch, never retry here.
    async fn send(&self, node: &Node) -> Result<()> {
        let (reply, outcome) = oneshot::channel();
        self.outgoing
            .send((node.clone(), reply))
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
        while let Some((node, reply)) = outgoing_rx.recv().await {
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
    super::identity_notifications::application::recover(&store, config.transport.identity_changes)
        .await?;
    // Reply settlement and contact projection are separate durable commits.
    // Repair the gap after a crash without deleting markers or replaying IQs.
    super::contacts::ingest(&store)?;
    let active_id = super::messaging::new_message_id()?;
    link.send(&super::messaging::active_request(&active_id))
        .await?;
    let mut ready = Some(ready);
    let activated_deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let ping_period = Duration::from_millis((config.transport.timeout_ms / 2).clamp(1000, 20000));
    let mut heartbeat = tokio::time::interval(ping_period);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    heartbeat.tick().await;
    let mut pending_ping: Option<String> = None;
    let mut outgoing: Option<super::outgoing::Pending> = None;
    let mut group_outgoing: Option<super::outgoing_group::Pending> = None;
    let mut upload: Option<super::prekeys::PendingUpload> = None;
    let mut upload_rounds: u32 = 0;
    let mut device_query: Option<super::devices::DeviceQuery> = None;
    let mut device_cursor = super::devices::DeviceSyncCursor::default();
    let mut device_context = super::devices::SyncContext::Interactive;
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
        tokio::select! {
            _ = outbox_tick.tick(), if send_outbox && ready.is_none() => {
                if group_outgoing.as_ref().is_some_and(|v|tokio::time::Instant::now()>=v.deadline) {
                    group_outgoing.take().expect("expired group preparation").expire(&store, &mut device_cursor)?;
                }
                if outgoing.as_ref().is_some_and(|v|tokio::time::Instant::now()>=v.deadline) {
                    return Err(Error::invalid("outbox", "key query outcome unknown; reserved, no retry"));
                }
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
                            link.send(&node).await?;
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
            _ = heartbeat.tick(), if ready.is_none() => {
                if upload.as_ref().is_some_and(|u| tokio::time::Instant::now() >= u.deadline) {
                    // The batch stays reserved as outcome_unknown; its keys are never re-sent.
                    tracing::warn!("prekey upload outcome unknown; reservation kept, no retry");
                    upload = None;
                }
                if device_query.as_ref().is_some_and(|q| tokio::time::Instant::now() >= q.deadline) {
                    // Queries are reads: pending users stay pending for a later session.
                    tracing::warn!("device query unanswered; users remain pending");
                    if let Some(query) = device_query.take() { device_cursor.defer(&query); }
                }
                // Authenticated primary-key changes and offline requests can
                // queue work during this connection without a devices notification.
                if device_query.is_none() && group_outgoing.as_ref().is_none_or(|g| !g.devices_pending())
                    && let Some((query,node))=device_cursor.begin_next(&store,device_context)? {
                    device_query=Some(query);
                    link.send(&node).await?;
                }
                for receipt in super::receipts::reserve_pending(&store)? { link.send(&receipt).await?; }
                {
                    let delay_ms = config.transport.action_delay_ms;
                    let actions = super::actions::reserve_pending(&store)?;
                    for (i, action) in actions.into_iter().enumerate() {
                        if i > 0 && delay_ms > 0 {
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        }
                        link.send(&action).await?;
                    }
                }
                // Unanswered keepalive: classified recoverable; the actor decides on reconnect.
                if pending_ping.is_some() { return Err(Error::Timeout(ping_period)); }
                let id = super::messaging::new_message_id()?;
                // Comparison LX/EIY.<init>(VLI), branch 15, 0x006a..009e.
                let ping = Node::new("iq").attr("id", id.as_str()).attr("type", "get")
                    .attr("xmlns", "w:p").attr("to", crate::model::jid::Jid::parse("s.whatsapp.net")?);
                link.send(&ping).await?; pending_ping = Some(id);
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
                super::clock::observe(&store, &node)?;
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
                if node.description == "iq" {
                    if group_outgoing.as_ref().is_some_and(|v|v.matches(&node)) {
                        let next=group_outgoing.take().expect("matched group preparation").settle_with_policy(&store,&account,&node,epoch,&mut device_cursor,config.transport.outgoing_identity_changes).await?;
                        group_outgoing=next.pending;
                        if let Some(pending) = &group_outgoing { pending.observe_device_query(&mut device_cursor); }
                        if let Some(stanza)=next.node { link.send(&stanza).await?; }
                        continue;
                    }
                    if outgoing.as_ref().is_some_and(|v|v.matches(&node)) {
                        if let Some(stanza)=outgoing.take().expect("matched pending query").prepare_with_policy(&store,&account,&node,epoch,config.transport.outgoing_identity_changes).await? {
                            link.send(&stanza).await?;
                        }
                        continue;
                    }
                    let id = node.get_attr("id").and_then(|v| v.as_text());
                    if id == Some(active_id.as_str()) && ready.is_some() {
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
                        for receipt in super::receipts::reserve_pending(&store)? { link.send(&receipt).await?; }
                        {
                            let delay_ms = config.transport.action_delay_ms;
                            let actions = super::actions::reserve_pending(&store)?;
                            for (i, action) in actions.into_iter().enumerate() {
                                if i > 0 && delay_ms > 0 {
                                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                                }
                                link.send(&action).await?;
                            }
                        }
                        if let Some(ready) = ready.take() { let _ = ready.send(()); }
                        // LX/DNq.run: unsent prekeys (or a store that never generated any) upload at start.
                        if super::prekeys::wants_upload(&store)? {
                            maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds).await?;
                        }
                        // SyncDevicesJob default preset: pending users are queried interactively.
                        if let Some((query, node)) = device_cursor.begin_next(&store, device_context)? {
                            device_query = Some(query);
                            link.send(&node).await?;
                        }
                        continue;
                    }
                    // Queued one-shot actions (block list, reports, lookups) settle by iq id.
                    if super::actions::record_reply(&store, &node)? {
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
                    if device_query.as_ref().is_some_and(|q| q.matches(&node)) {
                        let query = device_query.take().expect("matched device query");
                        let summary = device_cursor.settle(&query, &store, &node)?;
                        if summary.verification_errors > 0 {
                            // Matching proves this is our opaque local reservation
                            // ID, not a user address or arbitrary response body.
                            tracing::warn!(query_id = node.get_attr("id").and_then(|v| v.as_text()).unwrap_or("unavailable"), verification_errors = summary.verification_errors, "device candidates refused locally; inspect device-query --id; saved lists and pending work retained");
                        }
                        tracing::info!(accepted = summary.accepted, ingested = summary.ingested, partial_errors = summary.partial_errors, omitted_users = summary.omitted_users, superseded_users = summary.superseded_users, verification_errors = summary.verification_errors, "device query settled");
                        if group_outgoing.as_ref().is_none_or(|g| !g.devices_pending())
                            && let Some((query, sync)) = device_cursor.begin_next(&store, device_context)? {
                            device_query = Some(query);
                            link.send(&sync).await?;
                        }
                        continue;
                    }
                    if upload.as_ref().is_some_and(|u| u.matches(&node)) {
                        let outcome = upload.take().expect("matched pending upload").settle(&store, &node)?;
                        // LX/Bor.A03: list/@count_low=true schedules another legacy upload.
                        if outcome.accepted && outcome.count_low {
                            maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds).await?;
                        }
                        continue;
                    }
                    if id == pending_ping.as_deref() && pending_ping.is_some() {
                        if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") { return Err(Error::invalid("connection", "keepalive refused")); }
                        pending_ping = None; continue;
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
                match super::groups::notification_ack(&node) {
                    Ok(Some(ack)) => link.send(&ack).await?,
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
                        link.send(&sync).await?;
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

                let low = match super::prekeys::low_count(&node).and_then(|low| {
                    // Built before any upload: a count notification without
                    // `id` is refused like a malformed count, not fatal.
                    low.map(|low| Ok((low, super::prekeys::low_count_ack(&node)?)))
                        .transpose()
                }) {
                    Ok(low) => low,
                    Err(Error::InvalidInput { field, reason }) => {
                        // APK 37.71 14z.A09 throws before queueing work; where that
                        // is caught is not traced (prekey-count-notifications.md).
                        // Continuing is local policy; no ACK means redelivery.
                        tracing::warn!(field, reason, "prekey count refused locally; captured without ACK or upload; session continues");
                        None
                    }
                    Err(error) => return Err(error),
                };
                if let Some((low, low_ack)) = low {
                    // Local ACK ordering follows durable capture; native 14z.A02 ACKs after worker processing.
                    // Its first-child legacy flag triggers upload even for an unknown count; PQ-only is inert.
                    link.send(&low_ack).await?;
                    if low.legacy_present {
                        maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds).await?;
                    }
                }
                super::outgoing::acknowledge(&store, &node)?;
                if received == super::inbox::IncomingResult::Undecryptable {
                    match super::receipts::queue_retry_for_incoming(&store, &account, &node) {
                        Ok(0) => tracing::info!(target: "whatsapp_api::incoming", "retry already recorded; no repeat"),
                        Ok(_) => {
                            for receipt in super::receipts::reserve_pending(&store)? {
                                link.send(&receipt).await?;
                            }
                            tracing::info!(target: "whatsapp_api::incoming", "retry queued; reserved receipt batch written; delivery remains unconfirmed");
                        }
                        Err(Error::InvalidInput { field, reason }) => tracing::warn!(field, reason, "retry receipt not queued"),
                        Err(error) => return Err(error),
                    }
                }
                let mut confirmed = false;
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
                            for receipt in super::receipts::reserve_pending(&store)? { link.send(&receipt).await?; }
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

/// One bounded prekey upload for this session: generate when nothing is
/// unsent (`LX/14z.A02` `0174..0190`), reserve a batch, write it. Accounts
/// without Signal material or a registered signed prekey cannot upload and are
/// skipped; the follow-up count is capped per session.
async fn maybe_upload(
    store: &Arc<ProtocolStore>,
    account: &AccountRecord,
    link: &NodeLink,
    upload: &mut Option<super::prekeys::PendingUpload>,
    rounds: &mut u32,
) -> Result<()> {
    if upload.is_some() || *rounds >= super::prekeys::MAX_ROUNDS_PER_SESSION {
        return Ok(());
    }
    if account.signal_identity.is_none() || account.signal_registration_id.is_none() {
        return Ok(());
    }
    let signed = match crate::registration::factory::registered_signed_prekey(store) {
        Ok(signed) => signed,
        Err(_) => {
            tracing::warn!("prekey upload skipped: registered signed prekey unavailable");
            return Ok(());
        }
    };
    if super::prekeys::unsent_ids(store)?.is_empty() {
        super::prekeys::generate(store)?;
    }
    if let Some((pending, node)) = super::prekeys::begin_upload(store, account, &signed)? {
        *rounds += 1;
        *upload = Some(pending);
        link.send(&node).await?;
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
        async fn written(&mut self) -> Outbound {
            self.written_within(Duration::from_secs(5)).await
        }
        async fn written_within(&mut self, window: Duration) -> Outbound {
            tokio::time::timeout(window, self.from_worker.recv())
                .await
                .expect("write within deadline")
                .expect("link open")
        }
        async fn nothing_written(&mut self, window: Duration) {
            assert!(
                tokio::time::timeout(window, self.from_worker.recv())
                    .await
                    .is_err(),
                "unexpected write"
            );
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
        for grouped in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("retry-cut.redb"));
            store
                .write(|tx| {
                    let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                    account.signal_registration_id = Some(0x0102_0304);
                    account.save(tx)
                })
                .unwrap();
            // Like the inbox resilience fixture: no session for this sender, so
            // this is a genuine decryption failure through the production loop.
            let failed = Node::new("message")
                .attr("from", peer())
                .attr("id", "FAILED")
                .attr("type", "text")
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "msg")
                        .bytes(vec![9; 48]),
                ]);
            let failed = if grouped {
                failed
                    .attr("from", "120363000000000001@g.us")
                    .attr("participant", peer().with_device(3))
                    .children(vec![
                        Node::new("enc")
                            .attr("v", "2")
                            .attr("type", "skmsg")
                            .bytes(vec![9; 48]),
                    ])
            } else {
                failed
            };
            let mut fake = start(&store, false, None);
            fake.activate().await;
            fake.deliver(failed.clone()).await;
            let (retry, reply) = fake.written().await;
            assert_eq!(retry.description, "receipt");
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
            fake.stop_cleanly().await;
        }
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
        fake.deliver(
            Node::new("iq")
                .attr("type", "result")
                .attr("id", ping.get_attr("id").and_then(|v| v.as_text()).unwrap()),
        )
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
        fake.deliver(count_notification("pq", "pq_count", "٢").children(vec![
            Node::new("pq_count"),
            Node::new("count").attr("value", "7"),
        ]))
        .await;
        let (ack, write) = fake.written().await;
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("pq"));
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
        // A malformed count is refused locally: captured, no ACK, no upload,
        // and the session keeps receiving (it would otherwise be redelivered
        // and end every later session too).
        let malformed = count_notification("malformed", "pq_count", "2147483648");
        let wire = encode(&malformed).unwrap();
        fake.deliver(malformed).await;
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
        // count_low on the result schedules one more batch of freshly generated keys.
        fake.deliver(result_for(&iq, true)).await;
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
        fake.deliver(
            Node::new("iq")
                .attr("id", ping.get_attr("id").unwrap().as_text().unwrap())
                .attr("type", "result"),
        )
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
            fake.deliver(
                Node::new("iq")
                    .attr("id", node.get_attr("id").unwrap().as_text().unwrap())
                    .attr("type", "result"),
            )
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
        fake.deliver(
            Node::new("iq")
                .attr("id", ping.get_attr("id").unwrap().as_text().unwrap())
                .attr("type", "result"),
        )
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
        fake.deliver(
            Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result"),
        )
        .await;
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
