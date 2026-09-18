//! Persistent comparison connection. Explicit opt-in; no hidden reconnects.
use super::{
    ConnectionEpoch,
    comparison::{ComparisonEndpoint, ComparisonGateway},
    handshake::{COMPARISON_CHAIN_POLICY, EncryptedConnection, EncryptedWriter, VerifiedHandshake},
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
            current.save(tx)?;
        }
        Ok(())
    })?;
    if status != "success" {
        return Err(Error::Authentication { code: None });
    }
    Ok(channel)
}

async fn write(writer: &mut EncryptedWriter<tokio::net::TcpStream>, node: &Node) -> Result<()> {
    let bytes = Encoder::new(TokenDictionary::comparison_2_26_37_70()).encode(node)?;
    writer.write(&bytes).await?;
    Ok(())
}

/// Run continuously until explicit stop, EOF, timeout or refusal. Caller owns
/// reconnect policy; this function never replays uncertain chats or resets state.
/// `ready` resolves only after the server accepts connection activation.
pub async fn run(
    store: Arc<ProtocolStore>,
    config: LabRegistrationConfig,
    epoch: ConnectionEpoch,
    mut stop: watch::Receiver<bool>,
    ready: oneshot::Sender<()>,
    send_outbox: bool,
    ticket: Option<crate::model::ids::LocalMessageId>,
) -> Result<()> {
    let mut channel = connect(&store, &config, epoch).await?;
    let account = store
        .read(AccountRecord::load)?
        .ok_or(Error::NotRegistered)?;
    let active_id = super::messaging::new_message_id()?;
    super::messaging::write_node(&mut channel, &super::messaging::active_request(&active_id))
        .await?;
    let (mut reader, mut writer) = channel.into_split()?;
    let (incoming_tx, mut incoming_rx) = mpsc::channel(64);
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
    // Abort-on-drop also covers callers cancelling this whole future.
    struct ReaderTask(tokio::task::JoinHandle<()>);
    impl Drop for ReaderTask {
        fn drop(&mut self) {
            self.0.abort();
        }
    }
    let _reader = ReaderTask(reader_task);
    let mut ready = Some(ready);
    let activated_deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let ping_period = Duration::from_millis((config.transport.timeout_ms / 2).clamp(1000, 20000));
    let mut heartbeat = tokio::time::interval(ping_period);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    heartbeat.tick().await;
    let mut pending_ping: Option<String> = None;
    let mut outgoing: Option<super::outgoing::Pending> = None;
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
                    write(&mut writer,&node).await?;
                }
            }
            changed = stop.changed() => { if changed.is_err() || *stop.borrow() { return Ok(()); } }
            _ = tokio::time::sleep_until(activated_deadline), if ready.is_some() => {
                return Err(Error::invalid("connection", "activation outcome unknown; frames preserved"));
            }
            _ = heartbeat.tick(), if ready.is_none() => {
                for receipt in super::receipts::reserve_pending(&store)? { write(&mut writer, &receipt).await?; }
                // Unanswered keepalive: classified recoverable; the actor decides on reconnect.
                if pending_ping.is_some() { return Err(Error::Timeout(ping_period)); }
                let id = super::messaging::new_message_id()?;
                // Comparison LX/EIY.<init>(VLI), branch 15, 0x006a..009e.
                let ping = Node::new("iq").attr("id", id.as_str()).attr("type", "get")
                    .attr("xmlns", "w:p").attr("to", crate::model::jid::Jid::parse("s.whatsapp.net")?);
                write(&mut writer, &ping).await?; pending_ping = Some(id);
            }
            incoming = incoming_rx.recv() => {
                let node = incoming.ok_or_else(|| Error::invalid("connection", "reader ended"))??;
                let capture = super::inbox::preserve(&store, &node)?;
                if matches!(node.description.as_str(), "failure" | "stream:error") {
                    return Err(Error::invalid("connection", "server ended session; reason preserved encrypted; no reconnect"));
                }
                if node.description == "iq" {
                    if outgoing.as_ref().is_some_and(|v|v.matches(&node)) {
                        if let Some(stanza)=outgoing.take().expect("matched pending query").prepare(&store,&account,&node,epoch).await? {
                            write(&mut writer,&stanza).await?;
                        }
                        continue;
                    }
                    let id = node.get_attr("id").and_then(|v| v.as_text());
                    if id == Some(active_id.as_str()) && ready.is_some() {
                        if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
                            return Err(Error::invalid("connection", "activation refused"));
                        }
                        for receipt in super::receipts::reserve_pending(&store)? { write(&mut writer, &receipt).await?; }
                        if let Some(ready) = ready.take() { let _ = ready.send(()); }
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
                if let Some(ack)=super::notifications::handle(&store, &node)? {write(&mut writer,&ack).await?;}
                super::outgoing::acknowledge(&store, &node)?;
                if (node.description == "receipt" || matches!(received, super::inbox::IncomingResult::Decrypted | super::inbox::IncomingResult::Duplicate))
                    && let Some(confirmation) = super::messaging::received_confirmation(&node) {
                        // Raw capture and session/inbox commit precede this write.
                        // A reconnect may confirm a genuine server redelivery again.
                        write(&mut writer, &confirmation).await?;
                }
            }
        }
    }
}
