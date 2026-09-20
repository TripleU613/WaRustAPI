//! One selected normal-outbox send, using the public client and private JSON.
use super::*;
use whatsapp_api::{
    model::send::{AttemptProgress, DeliveryFactKind, SendStatus},
    store::{OpenOptions, ProtocolStore},
    transport::messaging::MessageRequest,
};

fn receipt_summary(status: &SendStatus) -> String {
    let has = |kind| status.facts.iter().any(|fact| fact.kind == kind);
    format!(
        "outbox receipts: delivered={} read={} played={} (saved protocol facts, not handset confirmation)",
        has(DeliveryFactKind::Delivered),
        has(DeliveryFactKind::Read),
        has(DeliveryFactKind::Played),
    )
}

// Run before media preparation: a reused text ID must never upload a new file.
// Existing media must already have its account-bound reference; prepare then
// checks the saved upload's file hash/view-once flag without another upload.
fn preflight(store: &ProtocolStore, message: &MessageRequest) -> Result<(), Error> {
    let id = ClientRequestId::new(message.request_id.clone())?;
    let outbox = whatsapp_api::outbox::Outbox::new("default", Default::default());
    let Some(saved) = store.read(|tx| outbox.get_by_request_id(tx, &id))? else {
        return Ok(());
    };
    let peer = message.recipient()?;
    if saved.intent.chat != ChatId::from_jid(&peer)? {
        return Err(Error::IdempotencyConflict(id));
    }
    let matches = match &saved.intent.content {
        MessageContent::Text { body, quoted } => {
            message.image.is_none()
                && message.voice_note.is_none()
                && quoted.is_none()
                && body == message.text.as_str()
        }
        MessageContent::Media {
            reference,
            caption,
            file_name,
        } if message.image.is_some() || message.voice_note.is_some() => {
            caption.is_none()
                && file_name.is_none()
                && *reference
                    == whatsapp_api::transport::outgoing_media::reference(store, &id, &peer)?
        }
        _ => false,
    };
    if !matches {
        return Err(Error::IdempotencyConflict(id));
    }
    Ok(())
}

pub(super) async fn run(
    cli: &Cli,
    path: &std::path::Path,
    message_path: &std::path::Path,
    allow: bool,
) -> Result<(), Error> {
    if !allow {
        return Err(Error::invalid(
            "outbox_send",
            "requires --allow-live-request",
        ));
    }
    let config = signup::config(path)?;
    let message = MessageRequest::read(message_path)?;
    let account_id = AccountId::new(cli.account.clone())?;
    let options = || OpenOptions {
        path: cli.store.clone(),
        account_id: account_id.clone(),
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    };
    let content = {
        let store = ProtocolStore::open(options())?;
        let account = store
            .read(AccountRecord::load)?
            .ok_or(Error::NotRegistered)?;
        if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
            return Err(Error::NotRegistered);
        }
        whatsapp_api::transport::send_path::claim(
            &store,
            &ClientRequestId::new(message.request_id.clone())?,
            whatsapp_api::transport::send_path::SendPath::Outbox,
        )?;
        let lab: Option<serde_json::Value> = store.read(|tx| {
            tx.get_json(
                Table::Checkpoints,
                format!("message:{}:send", message.request_id).as_bytes(),
                1,
            )
        })?;
        if lab.is_some() {
            return Err(Error::invalid(
                "outbox_send",
                "request ID already belongs to a lab send; use its original path, no duplicate transmission",
            ));
        }
        preflight(&store, &message)?;
        if message.image.is_some() || message.voice_note.is_some() {
            whatsapp_api::transport::image_upload::prepare(&store, &message, &config).await?;
            let reference = whatsapp_api::transport::outgoing_media::reference(
                &store,
                &ClientRequestId::new(message.request_id.clone())?,
                &message.recipient()?,
            )?;
            MessageContent::Media {
                reference,
                caption: None,
                file_name: None,
            }
        } else {
            MessageContent::Text {
                body: message.text.to_string(),
                quoted: None,
            }
        }
    };
    let request = SendRequest {
        client_request_id: ClientRequestId::new(message.request_id.clone())?,
        chat: ChatId::from_jid(&message.recipient()?)?,
        content,
    };
    let offline = open_client(cli, false).await?;
    let prepared = async {
        let ticket = offline.send_message(request).await?;
        let status = offline.message_status(&ticket).await?;
        Ok::<_, Error>((ticket, status))
    }
    .await;
    offline.shutdown().await;
    drop(offline);
    let (ticket, status) = prepared?;
    if status.server_accepted() {
        println!("outbox: server_acknowledged (saved); no network request");
        println!("{}", receipt_summary(&status));
        return Ok(());
    }
    if status.progress != AttemptProgress::Queued || status.is_uncertain() {
        return Err(Error::invalid(
            "outbox_send",
            "saved send is failed, cancelled, prepared or uncertain; no replay",
        ));
    }
    {
        let store = ProtocolStore::open(options())?;
        let query: Option<serde_json::Value> = store.read(|tx| {
            tx.get_json(
                Table::Checkpoints,
                format!("outbox-key-query:{}", ticket.local_id).as_bytes(),
                1,
            )
        })?;
        if query.is_some() {
            return Err(Error::invalid(
                "outbox_send",
                "prior key query is reserved; inspect its saved outcome, no retry",
            ));
        }
    }
    let client = WhatsAppClient::builder()
        .store_path(cli.store.clone())
        .account_id(AccountId::new(cli.account.clone())?)
        .key_source(Arc::new(EnvKeySource::new(cli.key_env.clone())))
        .create_store(false)
        .comparison_transport(config)
        .comparison_outbox_ticket(ticket.local_id)
        .build()
        .await?;
    let result = async {
        client.connect(Duration::from_secs(120)).await?;
        println!("outbox: authenticated; one selected durable intent enabled");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        loop {
            let status = client.message_status(&ticket).await?;
            if status.server_accepted() {
                println!("outbox: server_acknowledged; handset delivery/read not yet established");
                println!("{}", receipt_summary(&status));
                return Ok(());
            }
            if status.progress == AttemptProgress::Failed {
                return Err(Error::invalid(
                    "outbox_send",
                    "send failed; saved outcome preserved; no retry",
                ));
            }
            if !matches!(
                client.connection_state().borrow().state,
                whatsapp_api::runtime::ConnectionState::Ready { .. }
            ) {
                return Err(Error::invalid(
                    "outbox_send",
                    "connection ended; inspect saved outcome; no retry",
                ));
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(Error::invalid(
                    "outbox_send",
                    "acknowledgement deadline elapsed; outcome preserved; no retry",
                ));
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
    .await;
    client.shutdown().await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_text_id_is_rejected_before_media_file_or_upload_access() {
        use whatsapp_api::{
            outbox::Outbox,
            store::{MasterKey, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("state.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([8; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let mut message: MessageRequest = serde_json::from_value(serde_json::json!({
            "request_id": "stable-id", "to": "15550000000", "text": "original"
        }))
        .unwrap();
        let outbox = Outbox::new("default", Default::default());
        let request = SendRequest {
            client_request_id: ClientRequestId::new(message.request_id.clone()).unwrap(),
            chat: ChatId::from_jid(&message.recipient().unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "original".into(),
                quoted: None,
            },
        };
        store.write(|tx| outbox.enqueue(tx, &request)).unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        preflight(&store, &message).unwrap();
        let head = store.journal_head().unwrap();
        message.text.clear();
        message.image = Some(dir.path().join("does-not-exist.png"));
        assert!(matches!(
            preflight(&store, &message),
            Err(Error::IdempotencyConflict(_))
        ));
        message.image = None;
        message.voice_note = Some(dir.path().join("does-not-exist.ogg"));
        assert!(matches!(
            preflight(&store, &message),
            Err(Error::IdempotencyConflict(_))
        ));
        message.voice_note = None;
        message.text.push_str("changed");
        assert!(matches!(
            preflight(&store, &message),
            Err(Error::IdempotencyConflict(_))
        ));
        message.text.clear();
        message.text.push_str("original");
        message.to.clear();
        message.to.push_str("15550000001");
        assert!(matches!(
            preflight(&store, &message),
            Err(Error::IdempotencyConflict(_))
        ));
        assert_eq!(store.journal_head().unwrap(), head);
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 0);
        assert_eq!(
            store
                .read(|tx| outbox.get_by_request_id(tx, &request.client_request_id))
                .unwrap()
                .unwrap()
                .intent,
            request
        );
    }

    #[tokio::test]
    async fn live_opt_in_is_required_before_files_or_store_are_opened() {
        let cli = Cli::try_parse_from([
            "whatsapp-cli",
            "outbox-send",
            "--config",
            "missing-config.json",
            "--message",
            "missing-message.json",
        ])
        .unwrap();
        let Cmd::OutboxSend {
            config,
            message,
            allow_live_request,
        } = &cli.cmd
        else {
            panic!("wrong command");
        };
        assert!(!allow_live_request);
        let config = config.as_deref().expect("config given");
        let error = run(&cli, config, message, *allow_live_request)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("requires --allow-live-request"));
    }

    #[test]
    fn selected_send_requires_the_message_path_and_a_config_source() {
        for args in [
            vec!["whatsapp-cli", "outbox-send"],
            vec!["whatsapp-cli", "outbox-send", "--config", "config.json"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
        // `--config` may come from a profile; without one the command must fail
        // before touching anything.
        let cli = Cli::try_parse_from(["whatsapp-cli", "outbox-send", "--message", "message.json"])
            .unwrap();
        let Cmd::OutboxSend { config, .. } = &cli.cmd else {
            panic!("wrong command");
        };
        assert!(config.is_none());
        assert!(cli.config_path(None, None).is_err());
    }
}
