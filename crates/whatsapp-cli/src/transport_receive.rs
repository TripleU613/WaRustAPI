//! Offline processing of encrypted captures. Never sends receipts or chats.
use std::sync::Arc;
use whatsapp_api::{
    Error,
    binary::{Decoder, TokenDictionary, unpack_frame_payload},
    model::ids::AccountId,
    profile::Limits,
    session::AccountRecord,
    store::{EnvKeySource, OpenOptions, ProtocolStore, Table},
};

fn is_capture(key: &[u8]) -> bool {
    if key.starts_with(b"inbound:") {
        return true;
    }
    let Ok(key) = std::str::from_utf8(key) else {
        return false;
    };
    let parts: Vec<_> = key.split(':').collect();
    matches!(parts.as_slice(), ["message", _, "pending" | "send-response", index] if !index.is_empty() && index.bytes().all(|b| b.is_ascii_digit()))
}

pub(super) async fn listen(
    cli: &super::Cli,
    path: &std::path::Path,
    allow_live: bool,
    seconds: u64,
    send_outbox: bool,
) -> Result<(), Error> {
    if !allow_live {
        return Err(Error::invalid("listen", "requires --allow-live-request"));
    }
    let config = super::signup::config(path)?;
    let reconnect = config.transport.reconnect;
    let client = whatsapp_api::WhatsAppClient::builder()
        .store_path(cli.store.clone())
        .account_id(AccountId::new(cli.account.clone())?)
        .key_source(Arc::new(EnvKeySource::new(cli.key_env.clone())))
        .create_store(false)
        .comparison_transport(config)
        .comparison_outbox(send_outbox)
        .build()
        .await?;
    if send_outbox {
        // Show exactly what this session may transmit before any connection.
        let pending = whatsapp_api::transport::outgoing::eligible(client.store(), 256)?;
        println!(
            "listen: outbox transmission enabled for {} eligible queued intent(s); unsupported content fails locally",
            pending.len()
        );
        for entry in &pending {
            println!("  {}", pending_line(entry));
        }
    }
    if let Err(error) = client.connect(std::time::Duration::from_secs(120)).await {
        client.shutdown().await;
        return Err(error);
    }
    let mut status = client.connection_state();
    let sends = if send_outbox {
        "queued outbox sends enabled"
    } else {
        "no chat sends"
    };
    if reconnect.max_attempts == 0 {
        println!(
            "listen: authenticated and active; immediate incoming decryption/receipts enabled; no automatic reconnect; {sends}"
        );
    } else {
        println!(
            "listen: authenticated and active; immediate incoming decryption/receipts enabled; bounded reconnect after recoverable loss (up to {} attempts); {sends}",
            reconnect.max_attempts
        );
    }
    let ended = tokio::select! {
        _ = async {
            loop {
                // Connecting/Backoff are transient reconnect states; only a
                // terminal state ends the listener.
                match &status.borrow_and_update().state {
                    whatsapp_api::runtime::ConnectionState::Ready{..}
                    | whatsapp_api::runtime::ConnectionState::Connecting{..} => {}
                    whatsapp_api::runtime::ConnectionState::Backoff{attempt,..} => {
                        println!("listen: connection lost; reconnect attempt {attempt} scheduled; saved state preserved");
                    }
                    _ => break,
                }
                if status.changed().await.is_err() {break;}
            }
        } => true,
        _ = tokio::signal::ctrl_c() => false,
        _ = async {
            if seconds == 0 { std::future::pending::<()>().await; }
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        } => false,
    };
    client.shutdown().await;
    if ended {
        return Err(Error::invalid(
            "listen",
            "connection ended; saved state preserved; reconnect bounds exhausted or failure not recoverable",
        ));
    }
    println!("listen: stopped cleanly; account and sessions preserved");
    Ok(())
}

/// One redacted line per queued intent: identifiers, recipient, content kind
/// and accept time. Message text and media bytes are never printed.
fn pending_summary(entry: &whatsapp_api::outbox::OutboxEntry) -> serde_json::Value {
    let kind = match &entry.intent.content {
        whatsapp_api::model::message::MessageContent::Text { .. } => "text",
        whatsapp_api::model::message::MessageContent::Media { .. } => "media",
        _ => "unsupported",
    };
    serde_json::json!({
        "local_id": entry.local_id.to_string(),
        "client_request_id": entry.client_request_id.as_str(),
        "chat": entry.intent.chat.jid().to_string(),
        "kind": kind,
        "progress": format!("{:?}", entry.progress),
        "accepted_at_unix": entry.accepted_at.as_unix_seconds(),
    })
}
fn pending_line(entry: &whatsapp_api::outbox::OutboxEntry) -> String {
    let v = pending_summary(entry);
    format!(
        "local_id={} request_id={} chat={} kind={} accepted_at_unix={}",
        v["local_id"].as_str().unwrap_or(""),
        v["client_request_id"].as_str().unwrap_or(""),
        v["chat"].as_str().unwrap_or(""),
        v["kind"].as_str().unwrap_or(""),
        v["accepted_at_unix"]
    )
}

/// Queue explicit read receipts through the public client. Offline: the next
/// authenticated session transmits each once and settles it by server ACK.
pub(super) async fn mark_read(
    cli: &super::Cli,
    path: &std::path::Path,
    chat: &str,
    ids: &[String],
) -> Result<(), Error> {
    use whatsapp_api::model::{
        ids::MessageId,
        receipt::{Receipt, ReceiptKind},
    };
    let config = super::signup::config(path)?;
    let receipt = Receipt {
        chat: whatsapp_api::model::ids::ChatId::from_jid(&whatsapp_api::model::jid::Jid::parse(
            chat,
        )?)?,
        message_ids: ids
            .iter()
            .map(|id| MessageId::new(id.clone()))
            .collect::<Result<Vec<_>, _>>()?,
        from: None,
        kind: ReceiptKind::Read,
        server_timestamp: None,
        code: None,
    };
    let client = whatsapp_api::WhatsAppClient::builder()
        .store_path(cli.store.clone())
        .account_id(AccountId::new(cli.account.clone())?)
        .key_source(Arc::new(EnvKeySource::new(cli.key_env.clone())))
        .create_store(false)
        .comparison_transport(config)
        .build()
        .await?;
    let result = client.send_receipt(receipt).await;
    client.shutdown().await;
    result?;
    println!(
        "mark-read: {} read receipt(s) queued durably; the next listen session sends each once (repeat ids are ignored)",
        ids.len()
    );
    Ok(())
}

/// Offline contact device list, optionally recording a sync for the next session.
pub(super) fn devices(cli: &super::Cli, chat: &str, request: bool) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let jid = whatsapp_api::model::jid::Jid::parse(chat)?;
    if request {
        let own = store.read(AccountRecord::load)?.and_then(|a| a.own_jid);
        whatsapp_api::transport::devices::request_sync(&store, own.as_ref(), &jid, None, "cli")?;
        println!("devices: sync recorded; the next listen session queries this contact");
    }
    let list = whatsapp_api::transport::devices::device_list(&store, &jid)?;
    let pending = whatsapp_api::transport::devices::pending_users(&store)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "list": list,
                "pending_users": pending.iter().map(|j| j.to_string()).collect::<Vec<_>>(),
            }))
            .map_err(|e| Error::invalid("devices", e.to_string()))?
        );
        return Ok(());
    }
    match list {
        None => println!("devices: no saved list for this contact"),
        Some(list) => {
            println!(
                "devices: {} device(s); companions_verified={} raw_id={:?} list_ts={:?} hash_matches={:?}",
                list.devices.len(),
                list.companions_verified,
                list.raw_id,
                list.list_timestamp,
                list.hash_matches
            );
            for d in &list.devices {
                println!("  device={} key_index={}", d.jid.device, d.key_index);
            }
            if let Some(reason) = &list.unverified_reason {
                println!("  companions not verified: {reason}");
            }
        }
    }
    println!("devices: {} contact(s) pending a query", pending.len());
    Ok(())
}

/// Offline one-time prekey summary: counts by upload state and batch outcomes.
/// No key material is read or printed.
pub(super) fn prekeys_status(cli: &super::Cli) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let summary = whatsapp_api::transport::prekeys::summary(&store)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&summary)
                .map_err(|e| Error::invalid("prekeys", e.to_string()))?
        );
        return Ok(());
    }
    println!(
        "prekeys: unsent={} uncertain={} uploaded={} (uncertain batches are never re-sent)",
        summary.unsent, summary.uncertain, summary.uploaded
    );
    for (status, count) in &summary.batches {
        println!("  batches {status}: {count}");
    }
    println!(
        "prekeys: an authenticated session uploads unsent keys at start and after a server low-count notification"
    );
    Ok(())
}

/// Offline listing of the queued intents an outbox-enabled session would send.
pub(super) fn outbox_pending(cli: &super::Cli) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let pending = whatsapp_api::transport::outgoing::eligible(&store, 256)?;
    if cli.json {
        let rows: Vec<serde_json::Value> = pending.iter().map(pending_summary).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&rows)
                .map_err(|e| Error::invalid("outbox", e.to_string()))?
        );
        return Ok(());
    }
    println!(
        "outbox-pending: {} eligible queued intent(s); `listen --send-outbox` would transmit these, `cancel --local-id` removes one",
        pending.len()
    );
    for entry in &pending {
        println!("  {}", pending_line(entry));
    }
    Ok(())
}

pub(super) fn open_view_once(cli: &super::Cli) -> Result<(), Error> {
    use std::{
        io::Write,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    // Spawn first so a missing viewer cannot consume the attachment. No custom
    // config, scripts, screenshot bindings, history, plaintext path or master key.
    let mut child = Command::new("mpv")
        .args([
            "--no-config",
            "--load-scripts=no",
            "--input-default-bindings=no",
            "--input-conf=/dev/null",
            "--osc=no",
            "--no-audio",
            "--terminal=no",
            "--save-position-on-quit=no",
            "--resume-playback=no",
            "--keep-open=no",
            "--image-display-duration=30",
            "--title=View once — closes after 30 seconds",
            "-",
        ])
        .env_remove(&cli.key_env)
        .env_remove("WHATSAPP_API_MASTER_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| {
            Error::invalid(
                "viewer",
                "cannot start local mpv viewer; image not consumed",
            )
        })?;
    let outcome = (|| {
        let (ticket, plaintext) =
            whatsapp_api::transport::image_download::consume_view_once_with_ticket(&store)?;
        let mut input = child
            .stdin
            .take()
            .ok_or_else(|| Error::invalid("viewer", "viewer input unavailable; image consumed"))?;
        input.write_all(&plaintext).map_err(|_| {
            Error::invalid("viewer", "viewer input failed; image consumed; no reopen")
        })?;
        drop(input);
        drop(plaintext);
        println!(
            "image: consumed locally; sent to one-use desktop viewer; closes after 30 seconds; no server opened receipt"
        );
        let deadline = Instant::now() + Duration::from_secs(40);
        loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|_| Error::invalid("viewer", "viewer status unavailable; consumed"))?
            {
                return if status.success() {
                    whatsapp_api::transport::receipts::queue_opened(&store, &ticket)?;
                    println!(
                        "image: viewer completed; opened receipt queued for the next authenticated connection"
                    );
                    Ok(())
                } else {
                    Err(Error::invalid(
                        "viewer",
                        "viewer failed; image remains consumed",
                    ))
                };
            }
            if Instant::now() >= deadline {
                return Err(Error::invalid(
                    "viewer",
                    "viewer exceeded deadline; closed; image consumed",
                ));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    })();
    let _ = child.kill();
    let _ = child.wait();
    outcome
}

pub(super) async fn download_view_once(
    cli: &super::Cli,
    path: &std::path::Path,
    allow_live: bool,
) -> Result<(), Error> {
    if !allow_live {
        return Err(Error::invalid(
            "image_download",
            "requires --allow-live-request",
        ));
    }
    let config = super::signup::config(path)?;
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let account = store
        .read(AccountRecord::load)?
        .ok_or_else(|| Error::invalid("image_download", "missing account"))?;
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::invalid(
            "image_download",
            "requires matching registered account",
        ));
    }
    let size = whatsapp_api::transport::image_download::download_view_once(&store, &config).await?;
    println!(
        "image: {size} encrypted bytes downloaded and authenticated; privately stored unopened; no plaintext export or opened receipt"
    );
    Ok(())
}

#[cfg(test)]
fn session_sender(node: &whatsapp_api::binary::Node) -> Option<whatsapp_api::model::jid::Jid> {
    use whatsapp_api::model::jid::{Jid, Server};
    let parse = |name| {
        let value = node.get_attr(name)?;
        value
            .as_jid()
            .cloned()
            .or_else(|| Jid::parse(value.as_text()?).ok())
    };
    if node.get_attr("participant").is_some() {
        return None;
    }
    let sender = parse("from")?;
    match sender.server {
        Server::User => Some(sender),
        // Comparison LX/CmF.A01 0x0096..00c6 reads sender_pn as PhoneUserJid.
        // Only use an explicit mapping to an existing pinned primary session.
        Server::Lid if sender.device == 0 => {
            let phone = parse("sender_pn")?;
            (phone.server == Server::User && phone.device == 0).then_some(phone)
        }
        _ => None,
    }
}

/// Only previously authenticated, atomically committed messages qualify for
/// delivery confirmation. Raw capture alone is not proof of successful delivery.
#[cfg(test)]
pub(super) fn is_saved_message(
    store: &ProtocolStore,
    node: &whatsapp_api::binary::Node,
) -> Result<bool, Error> {
    let Some(sender) = session_sender(node) else {
        return Ok(false);
    };
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.len() <= 128)
    else {
        return Ok(false);
    };
    let key = format!("received:{sender}:{id}");
    let saved: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?;
    Ok(matches!(
        saved
            .as_ref()
            .and_then(|v| v.get("status"))
            .and_then(|v| v.as_str()),
        Some("decrypted" | "consumed")
    ))
}

pub(super) async fn run(cli: &super::Cli, inspect: bool) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let account = store
        .read(AccountRecord::load)?
        .ok_or_else(|| Error::invalid("inbox", "missing account"))?;
    let captures = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
    if captures.len() == 10000 {
        return Err(Error::invalid(
            "inbox",
            "checkpoint scan limit reached; pagination required",
        ));
    }
    let limits = Limits::default();
    let mut count = 0;
    let mut capture_shapes = std::collections::BTreeSet::new();
    for (key, value) in captures {
        if inspect {
            if is_capture(&key) {
                let wire: Vec<u8> = serde_json::from_slice(&value)
                    .map_err(|_| Error::invalid("inbox", "invalid capture"))?;
                let raw = unpack_frame_payload(&wire, &limits)?;
                let node = Decoder::new(&raw, TokenDictionary::comparison_2_26_37_70(), &limits)
                    .decode_node()?;
                let safe = |value: &str| {
                    value.len() <= 32
                        && value
                            .bytes()
                            .all(|b| b.is_ascii_alphabetic() || matches!(b, b'_' | b'-' | b':'))
                };
                if safe(&node.description) {
                    let children: std::collections::BTreeSet<_> = node
                        .child_nodes()
                        .iter()
                        .filter(|n| safe(&n.description))
                        .map(|n| n.description.as_str())
                        .collect();
                    let kind = node
                        .get_attr("type")
                        .and_then(|v| v.as_text())
                        .filter(|v| safe(v))
                        .unwrap_or("none");
                    let offline = node
                        .child("offline")
                        .and_then(|n| n.get_attr("count"))
                        .and_then(|v| v.as_text())
                        .and_then(|v| v.parse::<u32>().ok());
                    capture_shapes.insert(format!("inbox: capture shape tag={}, type={kind}, children={children:?}, offline_count={offline:?}",node.description));
                }
            }
            if key.starts_with(b"received:") {
                let mut saved: serde_json::Value = serde_json::from_slice(&value)
                    .map_err(|_| Error::invalid("inbox", "invalid saved payload"))?;
                if saved["status"] == "consumed" {
                    println!("inbox: view-once attachment consumed; content unavailable");
                    continue;
                }
                let protobuf: zeroize::Zeroizing<Vec<u8>> =
                    serde_json::from_value(saved["protobuf"].take())
                        .map_err(|_| Error::invalid("inbox", "invalid saved protobuf"))?;
                let summary = whatsapp_api::signal::direct::payload_summary(&protobuf)?;
                println!(
                    "inbox: saved payload bytes={}, {summary}; values redacted",
                    protobuf.len()
                );
            }
            continue;
        }
        if !is_capture(&key) {
            continue;
        }
        let bytes: Vec<u8> = serde_json::from_slice(&value)
            .map_err(|_| Error::invalid("inbox", "invalid capture"))?;
        let unpacked = unpack_frame_payload(&bytes, &limits)?;
        let node = Decoder::new(&unpacked, TokenDictionary::comparison_2_26_37_70(), &limits)
            .decode_node()?;
        match whatsapp_api::transport::inbox::process(&store, &account, &node, &key).await? {
            whatsapp_api::transport::inbox::IncomingResult::Decrypted => {
                count += 1;
                println!("inbox: authenticated payload committed; content redacted");
            }
            whatsapp_api::transport::inbox::IncomingResult::Duplicate => {}
            whatsapp_api::transport::inbox::IncomingResult::Unsupported => {}
        }
    }
    for shape in capture_shapes {
        println!("{shape}");
    }
    println!("inbox: {count} newly decrypted payloads; no network requests");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use whatsapp_api::binary::Node;
    use whatsapp_api::model::jid::Jid;

    #[test]
    fn delivery_requires_committed_decrypted_message() {
        use whatsapp_api::store::{MasterKey, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("inbox.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap();
        let node = Node::new("message")
            .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            .attr("id", "offline-test");
        assert!(!is_saved_message(&store, &node).unwrap());
        let key = b"received:15551234567@s.whatsapp.net:offline-test";
        for (status, expected) in [("captured", false), ("decrypted", true), ("consumed", true)] {
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        key,
                        1,
                        &serde_json::json!({"status":status}),
                    )
                })
                .unwrap();
            assert_eq!(is_saved_message(&store, &node).unwrap(), expected);
        }
        assert!(!is_saved_message(&store, &node.attr("id", "other")).unwrap());
    }

    #[test]
    fn captures_include_frames_received_while_sending_but_not_intents() {
        for key in [
            "inbound:abc:0",
            "message:test:pending:2",
            "message:test:send-response:1",
        ] {
            assert!(is_capture(key.as_bytes()));
        }
        for key in [
            "received:sender:id",
            "message:test:send",
            "message:test:upload",
            "message:test:keys",
            "message:test:pending:",
            "message:test:pending:bad",
        ] {
            assert!(!is_capture(key.as_bytes()));
        }
    }

    #[test]
    fn lid_requires_explicit_primary_phone_mapping() {
        let phone = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let lid = Node::new("message").attr("from", Jid::parse("12345@lid").unwrap());
        assert!(session_sender(&lid).is_none());
        assert_eq!(
            session_sender(&lid.clone().attr("sender_pn", phone.clone())),
            Some(phone.clone())
        );
        assert!(session_sender(&lid.clone().attr("sender_pn", "12345@lid")).is_none());
        assert!(
            session_sender(
                &lid.clone()
                    .attr("sender_pn", "15551234567:1@s.whatsapp.net")
            )
            .is_none()
        );
        assert!(
            session_sender(
                &lid.attr("sender_pn", phone.clone())
                    .attr("participant", phone.clone())
            )
            .is_none()
        );
        assert_eq!(
            session_sender(&Node::new("message").attr("from", phone.clone())),
            Some(phone)
        );
        assert!(session_sender(&Node::new("message").attr("from", "12345@g.us")).is_none());
    }

    #[test]
    fn pending_summary_redacts_content_and_reports_kind() {
        use whatsapp_api::{
            model::{
                ids::{ChatId, ClientRequestId, LocalMessageId},
                message::MessageContent,
                send::{AttemptProgress, SendRequest},
                time::Timestamp,
            },
            outbox::OutboxEntry,
        };
        let entry = OutboxEntry {
            local_id: LocalMessageId([7u8; 16]),
            client_request_id: ClientRequestId::new("req-1").unwrap(),
            intent: SendRequest {
                client_request_id: ClientRequestId::new("req-1").unwrap(),
                chat: ChatId::from_jid(&Jid::parse("15555550100@s.whatsapp.net").unwrap()).unwrap(),
                content: MessageContent::Text {
                    body: "private body text".into(),
                    quoted: None,
                },
            },
            semantic_hash: [0; 32],
            wire_id: None,
            progress: AttemptProgress::Queued,
            attempts: Vec::new(),
            facts: Vec::new(),
            accepted_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        let line = pending_line(&entry);
        assert!(line.contains("kind=text"), "{line}");
        assert!(line.contains("request_id=req-1"), "{line}");
        assert!(line.contains("chat=15555550100@s.whatsapp.net"), "{line}");
        assert!(!line.contains("private body"), "{line}");
        let json = pending_summary(&entry);
        assert_eq!(json["progress"], "Queued");
        assert!(
            !serde_json::to_string(&json)
                .unwrap()
                .contains("private body")
        );
    }

    #[test]
    fn receive_cannot_send_and_requires_login() {
        use clap::Parser;
        assert!(
            super::super::Cli::try_parse_from([
                "cli",
                "lab-transport",
                "--config",
                "fixture.json",
                "--receive"
            ])
            .is_err()
        );
        assert!(
            super::super::Cli::try_parse_from([
                "cli",
                "lab-transport",
                "--config",
                "fixture.json",
                "--login",
                "--receive",
                "--message",
                "message.json"
            ])
            .is_err()
        );
        assert!(super::super::Cli::try_parse_from(["cli", "lab-inbox"]).is_ok());
    }
}
