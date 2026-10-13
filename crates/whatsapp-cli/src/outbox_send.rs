//! One selected normal-outbox send, using the public client and private JSON.
use super::*;
use whatsapp_api::{
    model::send::{AttemptProgress, DeliveryFactKind, SendStatus},
    store::{OpenOptions, ProtocolStore},
    transport::messaging::MessageRequest,
};

fn receipt_summary(status: &SendStatus, group: bool) -> String {
    if group {
        let count = |kind| {
            status
                .facts
                .iter()
                .filter(|fact| fact.kind == kind)
                .filter_map(|fact| fact.device.as_ref())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        };
        return format!(
            "outbox device receipts: delivered_devices={} read_devices={} played_devices={} (partial device facts; whole-group delivery and handset confirmation are unverified)",
            count(DeliveryFactKind::Delivered),
            count(DeliveryFactKind::Read),
            count(DeliveryFactKind::Played)
        );
    }
    let has = |kind| status.facts.iter().any(|fact| fact.kind == kind);
    format!(
        "outbox receipts: delivered={} read={} played={} (saved protocol facts, not handset confirmation)",
        has(DeliveryFactKind::Delivered),
        has(DeliveryFactKind::Read),
        has(DeliveryFactKind::Played),
    )
}

/// The normal-outbox reference for a JSON reply (`quote` without an emoji) in
/// a group or direct chat. The participant names the quoted message's author;
/// one of our own identities makes it our outgoing message. In a direct chat
/// any other author must be the contact itself, in the same form as `to`. Reactions and a supplied
/// `quoted_text` stay on the comparison path: the normal path copies the saved
/// parent's text itself and never sends a caller-supplied preview.
fn quoted(
    store: &ProtocolStore,
    account: &AccountRecord,
    message: &MessageRequest,
) -> Result<Option<whatsapp_api::model::message::MessageRef>, Error> {
    use whatsapp_api::model::{
        ids::{DeviceAddress, MessageId},
        message::{Direction, MessageKey, MessageRef},
    };
    let Some(quote) = &message.quote else {
        return Ok(None);
    };
    if quote.is_reaction() || quote.quoted_text.is_some() {
        return Err(Error::invalid(
            "outbox_send",
            "normal JSON quotes are replies without quoted_text; reactions and supplied previews remain on the comparison path",
        ));
    }
    let group = message.group_chat()?;
    let chat = destination(message)?;
    let author = quote.sender()?.to_non_ad();
    let own_lid = whatsapp_api::transport::group_send::own_lid(store)?;
    let ours = [account.own_jid.as_ref().map(Jid::to_non_ad), own_lid]
        .into_iter()
        .flatten()
        .any(|own| own.to_non_ad() == author);
    if group.is_none() && !ours && author != chat.to_non_ad() {
        return Err(Error::invalid(
            "outbox_send",
            "a direct reply quotes the contact (in the same form as `to`) or one of our identities",
        ));
    }
    Ok(Some(MessageRef {
        key: MessageKey {
            chat: ChatId::from_jid(&chat)?,
            sender: if ours {
                None
            } else {
                Some(DeviceAddress::from_jid(&author)?)
            },
            direction: if ours {
                Direction::Outgoing
            } else {
                Direction::Incoming
            },
            id: MessageId::new(quote.message_id.clone())?,
        },
    }))
}

/// JSON this path does not send. Runs before the store opens and before the
/// request ID is claimed for the outbox path, so the same JSON can still go to
/// lab-transport unchanged.
fn refuse_comparison_only(message: &MessageRequest) -> Result<(), Error> {
    if message
        .quote
        .as_ref()
        .is_some_and(|quote| quote.is_reaction() || quote.quoted_text.is_some())
    {
        return Err(Error::invalid(
            "outbox_send",
            "normal JSON quotes are replies without quoted_text; reactions and supplied previews remain on the comparison path",
        ));
    }
    if message.group_chat()?.is_some()
        && (message.view_once || message.fresh_sessions || message.skdm_out_of_band()?)
    {
        return Err(Error::invalid(
            "outbox_send",
            "normal group JSON supports text/uploaded media and replies with existing sessions and in-band distribution; view-once, fresh sessions and direct distribution remain on the comparison path",
        ));
    }
    Ok(())
}

fn destination(message: &MessageRequest) -> Result<Jid, Error> {
    match message.group_chat()? {
        Some(group) => Ok(group),
        None => message.recipient(),
    }
}

// Run before media preparation: a reused text ID must never upload a new file.
// Existing media must already have its account-bound reference; prepare then
// checks the saved upload's file hash/view-once flag without another upload.
fn preflight(
    store: &ProtocolStore,
    message: &MessageRequest,
    quote: Option<&whatsapp_api::model::message::MessageRef>,
) -> Result<(), Error> {
    let id = ClientRequestId::new(message.request_id.clone())?;
    let outbox = whatsapp_api::outbox::Outbox::new("default", Default::default());
    let Some(saved) = store.read(|tx| outbox.get_by_request_id(tx, &id))? else {
        return Ok(());
    };
    let peer = destination(message)?;
    if saved.intent.chat != ChatId::from_jid(&peer)? {
        return Err(Error::IdempotencyConflict(id));
    }
    let matches = match &saved.intent.content {
        MessageContent::Text { body, quoted, .. } => {
            message.image.is_none()
                && message.voice_note.is_none()
                && quoted.as_ref() == quote
                && body == message.text.as_str()
        }
        MessageContent::Media {
            reference,
            caption,
            file_name,
            quoted,
        } if message.image.is_some() || message.voice_note.is_some() => {
            quoted.as_ref() == quote
                && caption.is_none()
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
    let is_group = message.group_chat()?.is_some();
    refuse_comparison_only(&message)?;
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
        let quote = quoted(&store, &account, &message)?;
        preflight(&store, &message, quote.as_ref())?;
        if message.image.is_some() || message.voice_note.is_some() {
            whatsapp_api::transport::image_upload::prepare_outbox(&store, &message, &config)
                .await?;
            let reference = whatsapp_api::transport::outgoing_media::reference(
                &store,
                &ClientRequestId::new(message.request_id.clone())?,
                &destination(&message)?,
            )?;
            MessageContent::Media {
                reference,
                caption: None,
                file_name: None,
                quoted: quote,
            }
        } else {
            MessageContent::Text {
                body: message.text.to_string(),
                quoted: quote,
                style: Default::default(),
            }
        }
    };
    let request = SendRequest {
        client_request_id: ClientRequestId::new(message.request_id.clone())?,
        chat: ChatId::from_jid(&destination(&message)?)?,
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
        println!("{}", receipt_summary(&status, is_group));
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
        if whatsapp_api::transport::outgoing_group::preparation(&store, ticket.local_id)?.is_some()
        {
            return Err(Error::invalid(
                "outbox_send",
                "saved group preparation is reserved or blocked; inspect outbox-group-preparations; no query replay",
            ));
        }
        let query: Option<serde_json::Value> = store.read(|tx| {
            tx.get_json(
                Table::Checkpoints,
                format!("outbox-key-query:{}", ticket.local_id).as_bytes(),
                1,
            )
        })?;
        if query.as_ref().is_some_and(|row| {
            !whatsapp_api::transport::outgoing::identity_query_resumable(
                row,
                config.transport.outgoing_identity_changes,
            )
        }) {
            return Err(Error::invalid(
                "outbox_send",
                "prior key query cannot resume under this policy; inspect outbox-suspended; no query replay",
            ));
        }
    }
    let client = WhatsAppClient::builder()
        .store_path(cli.store.clone())
        .account_id(AccountId::new(cli.account.clone())?)
        .key_source(Arc::new(EnvKeySource::new(cli.key_env.clone())))
        .create_store(false)
        .comparison_transport(config.clone())
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
                println!("{}", receipt_summary(&status, is_group));
                return Ok(());
            }
            if status.progress == AttemptProgress::Queued
                && let Some(suspension) = client.identity_suspension(&ticket).await?
                && (suspension.reason != "identity_changed"
                    || config.transport.outgoing_identity_changes
                        == whatsapp_api::registration::request::IdentityChangePolicy::Pinned)
            {
                return Err(Error::invalid(
                    "outbox_send",
                    format!(
                        "send suspended: {}; inspect outbox-suspended; original intent preserved",
                        suspension.reason
                    ),
                ));
            }
            if status.progress == AttemptProgress::Failed {
                return Err(Error::invalid(
                    "outbox_send",
                    "send failed; saved outcome preserved; no retry",
                ));
            }
            if let Some(group) = client.group_preparation(&ticket).await?
                && group.phase == "blocked" {
                return Err(Error::invalid("outbox_send", format!("group preparation blocked: {}; inspect outbox-group-preparations; original intent preserved", group.blocking_reason.as_deref().unwrap_or("unknown"))));
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
                style: Default::default(),
            },
        };
        store.write(|tx| outbox.enqueue(tx, &request)).unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        preflight(&store, &message, None).unwrap();
        let head = store.journal_head().unwrap();
        message.text.clear();
        message.image = Some(dir.path().join("does-not-exist.png"));
        assert!(matches!(
            preflight(&store, &message, None),
            Err(Error::IdempotencyConflict(_))
        ));
        message.image = None;
        message.voice_note = Some(dir.path().join("does-not-exist.ogg"));
        assert!(matches!(
            preflight(&store, &message, None),
            Err(Error::IdempotencyConflict(_))
        ));
        message.voice_note = None;
        message.text.push_str("changed");
        assert!(matches!(
            preflight(&store, &message, None),
            Err(Error::IdempotencyConflict(_))
        ));
        message.text.clear();
        message.text.push_str("original");
        message.to.clear();
        message.to.push_str("15550000001");
        assert!(matches!(
            preflight(&store, &message, None),
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

    #[test]
    fn json_replies_map_authors_and_refuse_comparison_only_quotes() {
        use whatsapp_api::{
            model::message::Direction,
            outbox::Outbox,
            store::{MasterKey, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("group-reply.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([10; 32]))),
            create: true,
        })
        .unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15550000099@s.whatsapp.net").unwrap()),
            ..Default::default()
        };
        let message = |quote: serde_json::Value| -> MessageRequest {
            serde_json::from_value(serde_json::json!({
                "request_id": "group-reply", "group": "120363000000000001@g.us",
                "text": "reply body", "quote": quote,
            }))
            .unwrap()
        };
        // Another member's message: incoming, attributed to that member.
        let other = message(serde_json::json!({
            "message_id": "PARENT", "participant": "15550000001@s.whatsapp.net"
        }));
        let reference = quoted(&store, &account, &other).unwrap().unwrap();
        assert_eq!(reference.key.direction, Direction::Incoming);
        assert_eq!(
            reference.key.sender.as_ref().unwrap().to_jid(),
            Jid::parse("15550000001@s.whatsapp.net").unwrap()
        );
        // Our own message (any device of our number): outgoing, no sender.
        let own = message(serde_json::json!({
            "message_id": "OWN", "participant": "15550000099:3@s.whatsapp.net"
        }));
        let own_reference = quoted(&store, &account, &own).unwrap().unwrap();
        assert_eq!(own_reference.key.direction, Direction::Outgoing);
        assert!(own_reference.key.sender.is_none());
        // Reactions and caller-supplied previews stay on the comparison path,
        // refused before any store access or send-path claim.
        assert!(refuse_comparison_only(&other).is_ok());
        for quote in [
            serde_json::json!({"message_id": "PARENT", "participant": "15550000001@s.whatsapp.net", "emoji": "x"}),
            serde_json::json!({"message_id": "PARENT", "participant": "15550000001@s.whatsapp.net", "quoted_text": "preview"}),
        ] {
            let mut request = message(quote);
            if request.quote.as_ref().unwrap().is_reaction() {
                request.text.clear();
            }
            assert!(quoted(&store, &account, &request).is_err());
            assert!(refuse_comparison_only(&request).is_err());
        }
        // Direct replies: the contact or us; a third party is refused.
        let direct = |participant: &str| -> MessageRequest {
            serde_json::from_value(serde_json::json!({
                "request_id": "direct-reply", "to": "15550000001@s.whatsapp.net",
                "text": "reply body",
                "quote": {"message_id": "PARENT", "participant": participant},
            }))
            .unwrap()
        };
        let contact = quoted(&store, &account, &direct("15550000001@s.whatsapp.net"))
            .unwrap()
            .unwrap();
        assert_eq!(contact.key.direction, Direction::Incoming);
        assert_eq!(
            contact.key.chat.jid(),
            &Jid::parse("15550000001@s.whatsapp.net").unwrap()
        );
        let mine = quoted(&store, &account, &direct("15550000099@s.whatsapp.net"))
            .unwrap()
            .unwrap();
        assert_eq!(mine.key.direction, Direction::Outgoing);
        assert!(quoted(&store, &account, &direct("15550000002@s.whatsapp.net")).is_err());
        assert!(refuse_comparison_only(&direct("15550000001@s.whatsapp.net")).is_ok());
        // A reused request ID must keep the same quote target.
        let request = SendRequest {
            client_request_id: ClientRequestId::new("group-reply").unwrap(),
            chat: ChatId::from_jid(&destination(&other).unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "reply body".into(),
                quoted: Some(reference.clone()),
                style: Default::default(),
            },
        };
        store
            .write(|tx| Outbox::new("default", Default::default()).enqueue(tx, &request))
            .unwrap();
        preflight(&store, &other, Some(&reference)).unwrap();
        assert!(matches!(
            preflight(&store, &own, Some(&own_reference)),
            Err(Error::IdempotencyConflict(_))
        ));
        assert!(matches!(
            preflight(&store, &other, None),
            Err(Error::IdempotencyConflict(_))
        ));
    }

    #[test]
    fn group_json_uses_its_existing_chat_and_reused_ids_refuse_changed_destinations() {
        use whatsapp_api::{
            outbox::Outbox,
            store::{MasterKey, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("group-json.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap();
        let mut message:MessageRequest=serde_json::from_value(serde_json::json!({"request_id":"group-original","group":"120363000000000001@g.us","text":"saved group text"})).unwrap();
        let request = SendRequest {
            client_request_id: ClientRequestId::new(message.request_id.clone()).unwrap(),
            chat: ChatId::from_jid(&destination(&message).unwrap()).unwrap(),
            content: MessageContent::Text {
                body: message.text.to_string(),
                quoted: None,
                style: Default::default(),
            },
        };
        store
            .write(|tx| Outbox::new("default", Default::default()).enqueue(tx, &request))
            .unwrap();
        preflight(&store, &message, None).unwrap();
        let head = store.journal_head().unwrap();
        message.group = Some("120363000000000002@g.us".into());
        assert!(matches!(
            preflight(&store, &message, None),
            Err(Error::IdempotencyConflict(_))
        ));
        message.group = Some("120363000000000001@g.us".into());
        message.text.push_str(" altered");
        assert!(matches!(
            preflight(&store, &message, None),
            Err(Error::IdempotencyConflict(_))
        ));
        assert_eq!(store.journal_head().unwrap(), head);
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 0);
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
