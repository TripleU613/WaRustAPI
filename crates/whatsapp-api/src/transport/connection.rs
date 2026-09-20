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
    binary::{Decoder, Encoder, Node, TokenDictionary, unpack_frame_payload},
    model::time::Timestamp,
    profile::Limits,
    registration::request::LabRegistrationConfig,
    session::AccountRecord,
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
    super::inbox::preserve(store, &reply)?;
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
            let mut current = AccountRecord::load_in_write(tx)?.ok_or(Error::NotRegistered)?;
            current.last_authenticated_at = Some(Timestamp::now());
            // Observed live: `success` carries our `pn` and `lid`. Keep them for
            // message keys and vote encryption; never invent either.
            let pn = super::receipts::jid(&reply, "pn").map(|j| j.to_non_ad());
            let lid = super::receipts::jid(&reply, "lid").map(|j| j.to_non_ad());
            if current.own_jid.is_none() {
                current.own_jid = pn.clone();
            }
            if pn.is_some() || lid.is_some() {
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({
                        "pn": pn.as_ref().map(|j| j.to_string()),
                        "lid": lid.as_ref().map(|j| j.to_string()),
                        "at": Timestamp::now(),
                    }),
                )?;
            }
            current.save(tx)?;
        }
        Ok(())
    })?;
    if status != "success" {
        return Err(Error::Authentication { code: None });
    }
    Ok(channel)
}

/// One outbound stanza and where to report the socket write outcome.
pub(crate) type Outbound = (Node, oneshot::Sender<Result<()>>);

/// Node-level link for one connection epoch. Real sessions adapt the split
/// encrypted transport; tests inject channels and cut them at chosen points.
pub(crate) struct NodeLink {
    incoming: mpsc::Receiver<Result<Node>>,
    outgoing: mpsc::Sender<Outbound>,
}
impl NodeLink {
    pub(crate) fn new(
        incoming: mpsc::Receiver<Result<Node>>,
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
    Ok(Encoder::new(TokenDictionary::comparison_2_26_37_70()).encode(node)?)
}
fn link_from(channel: EncryptedConnection<tokio::net::TcpStream>) -> Result<(NodeLink, LinkTasks)> {
    let (mut reader, mut writer) = channel.into_split()?;
    let (incoming_tx, incoming_rx) = mpsc::channel(64);
    let reader_task = tokio::spawn(async move {
        loop {
            let next = match reader.read().await {
                Ok(Some(bytes)) => {
                    let limits = Limits::default();
                    unpack_frame_payload(&bytes, &limits)
                        .map_err(Error::from)
                        .and_then(|raw| {
                            Decoder::new(&raw, TokenDictionary::comparison_2_26_37_70(), &limits)
                                .decode_node()
                                .map_err(Error::from)
                        })
                }
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
    let mut upload: Option<super::prekeys::PendingUpload> = None;
    let mut upload_rounds: u32 = 0;
    let mut device_query: Option<super::devices::DeviceQuery> = None;
    let mut outbox_tick = tokio::time::interval(Duration::from_secs(1));
    outbox_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        if *stop.borrow() {
            return Ok(());
        }
        tokio::select! {
            _ = outbox_tick.tick(), if send_outbox && ready.is_none() => {
                if outgoing.as_ref().is_some_and(|v|tokio::time::Instant::now()>=v.deadline) {
                    return Err(Error::invalid("outbox", "key query outcome unknown; reserved, no retry"));
                }
                if outgoing.is_none() && let Some((pending,node))=super::outgoing::begin_selected(&store,ticket)? {
                    outgoing=Some(pending);
                    link.send(&node).await?;
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
                    device_query = None;
                }
                for receipt in super::receipts::reserve_pending(&store)? { link.send(&receipt).await?; }
                for action in super::actions::reserve_pending(&store)? { link.send(&action).await?; }
                // Unanswered keepalive: classified recoverable; the actor decides on reconnect.
                if pending_ping.is_some() { return Err(Error::Timeout(ping_period)); }
                let id = super::messaging::new_message_id()?;
                // Comparison LX/EIY.<init>(VLI), branch 15, 0x006a..009e.
                let ping = Node::new("iq").attr("id", id.as_str()).attr("type", "get")
                    .attr("xmlns", "w:p").attr("to", crate::model::jid::Jid::parse("s.whatsapp.net")?);
                link.send(&ping).await?; pending_ping = Some(id);
            }
            incoming = link.incoming.recv() => {
                let node = incoming.ok_or_else(|| Error::invalid("connection", "reader ended"))??;
                let capture = super::inbox::preserve(&store, &node)?;
                super::clock::observe(&store, &node)?;
                if matches!(node.description.as_str(), "failure" | "stream:error") {
                    return Err(Error::invalid("connection", "server ended session; reason preserved encrypted; no reconnect"));
                }
                if node.description == "iq" {
                    if outgoing.as_ref().is_some_and(|v|v.matches(&node)) {
                        if let Some(stanza)=outgoing.take().expect("matched pending query").prepare(&store,&account,&node,epoch).await? {
                            link.send(&stanza).await?;
                        }
                        continue;
                    }
                    let id = node.get_attr("id").and_then(|v| v.as_text());
                    if id == Some(active_id.as_str()) && ready.is_some() {
                        if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
                            return Err(Error::invalid("connection", "activation refused"));
                        }
                        for receipt in super::receipts::reserve_pending(&store)? { link.send(&receipt).await?; }
                        for action in super::actions::reserve_pending(&store)? { link.send(&action).await?; }
                        if let Some(ready) = ready.take() { let _ = ready.send(()); }
                        // LX/DNq.run: unsent prekeys (or a store that never generated any) upload at start.
                        if super::prekeys::wants_upload(&store)? {
                            maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds).await?;
                        }
                        // SyncDevicesJob default preset: pending users are queried interactively.
                        if let Some((query, node)) = super::devices::begin_query(&store, super::devices::SyncContext::Interactive)? {
                            device_query = Some(query);
                            link.send(&node).await?;
                        }
                        continue;
                    }
                    // Queued one-shot actions (block list, reports, lookups) settle by iq id.
                    if super::actions::record_reply(&store, &node)? { continue; }
                    if device_query.as_ref().is_some_and(|q| q.matches(&node)) {
                        let summary = device_query.take().expect("matched device query").ingest(&store, &node)?;
                        tracing::info!(ingested = summary.ingested, partial_errors = summary.partial_errors, "device lists updated");
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
                let received = super::inbox::process(&store, &account, &node, &capture).await?;
                super::receipts::record(&store, &node)?;
                super::receipts::record_ack(&store, &node)?;
                if let Some(ack)=super::notifications::handle(&store, &node)? {link.send(&ack).await?;}
                if let Some(change) = super::devices::notification(&node) {
                    // LX/11Q.A09: acknowledge after capture, then record the contact for a
                    // device sync (LX/101.A02). Own-account changes use account sync, which is
                    // not implemented: captured and acknowledged only.
                    link.send(&super::devices::notification_ack(&node)?).await?;
                    let own = account.own_jid.as_ref().map(|j| j.to_non_ad());
                    if own.as_ref() != Some(&change.owner) {
                        super::devices::request_sync(&store, own.as_ref(), &change.owner, change.device_hash.as_deref(), "notification")?;
                        if device_query.is_none()
                            && let Some((query, sync)) = super::devices::begin_query(&store, super::devices::SyncContext::Notification)?
                        {
                            device_query = Some(query);
                            link.send(&sync).await?;
                        }
                    }
                }
                if let Some(low) = super::prekeys::low_count(&node) {
                    // LX/14z.A09: acknowledge after the durable capture above, then LX/14z.A02:
                    // legacy count triggers generate-if-empty and upload; pq-only takes no action.
                    link.send(&super::prekeys::low_count_ack(&node)?).await?;
                    if low.legacy.is_some() {
                        maybe_upload(&store, &account, &link, &mut upload, &mut upload_rounds).await?;
                    }
                }
                super::outgoing::acknowledge(&store, &node)?;
                if (node.description == "receipt" || matches!(received, super::inbox::IncomingResult::Decrypted | super::inbox::IncomingResult::Duplicate))
                    && let Some(confirmation) = super::messaging::received_confirmation(&node) {
                        // Raw capture and session/inbox commit precede this write.
                        // A reconnect may confirm a genuine server redelivery again.
                        link.send(&confirmation).await?;
                }
            }
        }
    }
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
        to_worker: mpsc::Sender<Result<Node>>,
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
        let (to_worker, incoming) = mpsc::channel(16);
        let (outgoing, from_worker) = mpsc::channel(16);
        let (stop, stop_rx) = watch::channel(false);
        let (ready_tx, ready) = oneshot::channel();
        let task = tokio::spawn(drive(
            Session {
                store: store.clone(),
                config: config(),
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
            id
        }
        async fn deliver(&self, node: Node) {
            self.to_worker.send(Ok(node)).await.unwrap();
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
    }

    #[tokio::test]
    async fn explicit_receipt_write_cut_is_reserved_once_and_settled_by_a_later_ack() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("receipt-cut.redb"));
        queue_read_receipt(&store, "first");
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
        reply
            .send(Err(Error::Transport(TransportError::Io(
                std::io::ErrorKind::ConnectionReset,
            ))))
            .unwrap();
        let err = fake.finished().await.unwrap_err();
        assert!(err.is_recoverable_by_reconnect(), "{err:?}");
        assert_eq!(receipt_status(&store, "first"), "outcome_unknown");

        // Next epoch: no replay, and the server's late ACK settles the reservation.
        let mut fake = start(&store, false, None);
        fake.activate().await;
        fake.nothing_written(Duration::from_millis(300)).await;
        fake.deliver(
            Node::new("ack")
                .attr("id", "first")
                .attr("class", "receipt")
                .attr("type", "read")
                .attr("from", peer()),
        )
        .await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while receipt_status(&store, "first") != "server_acknowledged" {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        fake.stop_cleanly().await;
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
