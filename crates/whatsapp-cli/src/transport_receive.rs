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
    // A preserved login result. A `<failure>` can carry an appeal token, and a
    // refusal cannot be replayed to fetch one again, so it has to be readable
    // from saved state rather than only from the session that saw it.
    if key.starts_with(b"login-result:") {
        return true;
    }
    let Ok(key) = std::str::from_utf8(key) else {
        return false;
    };
    let parts: Vec<_> = key.split(':').collect();
    matches!(parts.as_slice(), ["message", _, "pending" | "send-response", index] if !index.is_empty() && index.bytes().all(|b| b.is_ascii_digit()))
}

fn capture_protocol_type(value: Option<&str>) -> &str {
    value
        .filter(|value| {
            // APK 37.71 Fbq.<init> registers the literal group notification
            // type w:gp2. Do not widen the general filter to numeric IDs.
            *value == "w:gp2"
                || (value.len() <= 32
                    && value
                        .bytes()
                        .all(|b| b.is_ascii_alphabetic() || matches!(b, b'_' | b'-' | b':')))
        })
        .unwrap_or("none")
}

/// Read-only alias comparisons: missing pin is `None`, malformed input is an
/// error, and a differing identity is `Some(false)`. These do not select a
/// session or authorize replacement; they help distinguish routing from trust.
fn prekey_pin_summary(
    store: &ProtocolStore,
    node: &whatsapp_api::binary::Node,
) -> Result<String, Error> {
    use whatsapp_api::binary::NodeContent;
    let Some(enc) = node.child_nodes().iter().find(|n| {
        n.description == "enc" && n.get_attr("type").and_then(|v| v.as_text()) == Some("pkmsg")
    }) else {
        return Ok(String::new());
    };
    let NodeContent::Bytes(ciphertext) = &enc.content else {
        return Ok(", prekey_pin_inspection=invalid_ciphertext".into());
    };
    let mut comparisons = Vec::new();
    for attribute in ["from", "sender_pn", "participant", "participant_pn"] {
        let Some(peer) = node
            .get_attr(attribute)
            .and_then(|v| {
                v.as_jid()
                    .cloned()
                    .or_else(|| whatsapp_api::model::jid::Jid::parse(v.as_text()?).ok())
            })
            .filter(|j| {
                matches!(
                    j.server,
                    whatsapp_api::model::jid::Server::User | whatsapp_api::model::jid::Server::Lid
                )
            })
        else {
            continue;
        };
        let peer = peer.with_device(peer.device);
        let key = format!("comparison-direct:{peer}");
        let identity: Option<Vec<u8>> =
            store.read(|tx| tx.get_json(Table::PeerIdentities, key.as_bytes(), 1))?;
        let matches = identity
            .map(|pin| {
                whatsapp_api::signal::direct::incoming_prekey_identity_matches(ciphertext, &pin)
            })
            .transpose()?;
        comparisons.push(format!("{attribute}={matches:?}"));
    }
    Ok(format!(", prekey_pin_matches={comparisons:?}"))
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
        whatsapp_api::model::message::MessageContent::Reaction { .. } => "reaction",
        whatsapp_api::model::message::MessageContent::Poll { .. } => "poll",
        whatsapp_api::model::message::MessageContent::PollVote { .. } => "vote",
        whatsapp_api::model::message::MessageContent::DisappearingTimer { .. } => "disappearing",
        whatsapp_api::model::message::MessageContent::ChatPrivacy { .. } => "chat_privacy",
        whatsapp_api::model::message::MessageContent::KeepInChat { .. } => "keep",
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

fn device_query_inspection(
    record: Option<&whatsapp_api::transport::devices::QueryRecord>,
) -> serde_json::Value {
    match record {
        None => serde_json::json!({"offline_only":true,"found":false}),
        Some(record) => serde_json::json!({
            "offline_only":true,"found":true,"status":record.status,
            "requested_users":record.users.len(),"ingested":record.ingested,
            "partial_errors":record.partial_errors,"omitted_users":record.omitted_users,"superseded_users":record.superseded_users,
            "verification_errors":record.verification_failures.as_ref().map(|r| r.values().sum::<u32>()),
            "verification_failures":record.verification_failures,
            "code":record.code.as_ref().filter(|c| !c.is_empty() && c.bytes().all(|b| b.is_ascii_digit())),
        }),
    }
}

pub(super) fn device_query(cli: &super::Cli, id: &str) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let record = whatsapp_api::transport::devices::query_record(&store, id)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&device_query_inspection(record.as_ref()))
            .map_err(|_| Error::invalid("devices", "cannot encode query inspection"))?
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
        if !cli.json {
            println!("devices: sync recorded; a listen session queries this contact");
        }
    }
    let list = whatsapp_api::transport::devices::device_list(&store, &jid)?;
    let authority = whatsapp_api::transport::devices::device_list_authority(&store, &jid)?;
    let pending = whatsapp_api::transport::devices::pending_users(&store)?;
    if cli.json {
        let mut displayed = serde_json::to_value(&list)
            .map_err(|_| Error::invalid("devices", "cannot inspect saved list"))?;
        if let Some(row) = displayed.as_object_mut() {
            row.remove("verified_primary_key_fingerprint");
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "list": displayed,
                "authority": authority,
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
    if let Some(a) = authority {
        println!(
            "devices: companions_current={} binding_known={} primary_pin_present={} primary_pin_matches={:?}",
            a.companions_current, a.binding_known, a.primary_pin_present, a.primary_pin_matches
        );
    }
    println!("devices: {} contact(s) pending a query", pending.len());
    Ok(())
}

/// Mask a phone number for terminal output: only the last three digits show.
fn mask_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(char::is_ascii_digit).collect();
    if digits.len() <= 3 {
        return "*".repeat(digits.len());
    }
    format!(
        "{}{}",
        "*".repeat(digits.len() - 3),
        &digits[digits.len() - 3..]
    )
}

fn contacts_store(cli: &super::Cli) -> Result<ProtocolStore, Error> {
    ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })
}

fn print_contact(cli: &super::Cli, contact: &whatsapp_api::transport::contacts::Contact) {
    if cli.json {
        if let Ok(s) = serde_json::to_string_pretty(contact) {
            println!("{s}");
        }
        return;
    }
    println!(
        "contact: phone={} name={:?} first_name={:?}",
        mask_phone(&contact.phone),
        contact.full_name,
        contact.first_name
    );
}

/// Offline local address book: list/add/edit/remove, and queue/inspect a
/// `usync` WhatsApp-existence check (docs/protocol/chat-actions.md section 7).
pub(super) fn contacts(cli: &super::Cli, action: &super::ContactsCmd) -> Result<(), Error> {
    use whatsapp_api::transport::contacts;
    let store = contacts_store(cli)?;
    match action {
        super::ContactsCmd::List => {
            let all = contacts::list(&store)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&all)
                        .map_err(|e| Error::invalid("contacts", e.to_string()))?
                );
                return Ok(());
            }
            println!("contacts: {} saved", all.len());
            for c in &all {
                print_contact(cli, c);
            }
            Ok(())
        }
        super::ContactsCmd::Add {
            phone,
            name,
            first_name,
        } => {
            let contact = contacts::add(&store, phone, name, first_name.as_deref())?;
            print_contact(cli, &contact);
            Ok(())
        }
        super::ContactsCmd::Edit {
            phone,
            name,
            first_name,
        } => {
            let contact = contacts::edit(&store, phone, name, first_name.as_deref())?;
            print_contact(cli, &contact);
            Ok(())
        }
        super::ContactsCmd::Remove { phone } => {
            let removed = contacts::remove(&store, phone)?;
            if cli.json {
                println!("{}", serde_json::json!({ "removed": removed }));
            } else {
                println!(
                    "contacts: {}",
                    if removed {
                        "removed"
                    } else {
                        "no contact saved for this number"
                    }
                );
            }
            Ok(())
        }
        super::ContactsCmd::PublishAppState => {
            let own = whatsapp_api::transport::inbox::own_identities(&store)?;
            let device_index = own.0.as_ref().map_or(0, |j| j.device);
            let registered: std::collections::BTreeSet<u32> =
                [u32::from(device_index)].into_iter().collect();
            let now = whatsapp_api::model::time::Timestamp::now().0;
            let (id, count) = whatsapp_api::transport::app_state_sync::queue_contact_patch(
                &store,
                device_index,
                0,
                &registered,
                now,
            )?;
            println!(
                "app-state: contactAction patch for {count} contact(s) queued in critical_unblock_low; a session transmits it (action id {id})"
            );
            Ok(())
        }
        super::ContactsCmd::FetchAppState => {
            let id = whatsapp_api::transport::app_state_sync::queue_fetch(&store)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({"action_id": id}))
                        .map_err(|e| Error::invalid("app-state", e.to_string()))?
                );
                return Ok(());
            }
            println!(
                "app-state: read-only fetch of all collections queued; a session transmits it (action id {id})"
            );
            Ok(())
        }
        super::ContactsCmd::Sync => {
            let (id, count) = contacts::queue_full_sync(&store)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "action_id": id,
                        "contacts": count,
                    }))
                    .map_err(|e| Error::invalid("contacts", e.to_string()))?
                );
                return Ok(());
            }
            println!(
                "contacts: full sync of {count} contact(s) queued; a session transmits it (action id {id})"
            );
            Ok(())
        }
        super::ContactsCmd::Check { phone } => {
            let id = contacts::queue_check(&store, phone)?;
            let ingested = contacts::ingest(&store)?;
            let status = contacts::status(&store, phone)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "action_id": id,
                        "newly_ingested": ingested,
                        "status": status,
                    }))
                    .map_err(|e| Error::invalid("contacts", e.to_string()))?
                );
                return Ok(());
            }
            println!(
                "contacts: check queued; a session transmits it; inspect with `actions` (action id {id})"
            );
            match status {
                None => println!("contacts: no saved status yet for {}", mask_phone(phone)),
                Some(status) => println!(
                    "contacts: saved status for {} on_whatsapp={} checked_at={:?} error={:?}",
                    mask_phone(phone),
                    status.on_whatsapp,
                    status.checked_at,
                    status.error
                ),
            }
            Ok(())
        }
    }
}

/// Offline "clear chat": drop locally decrypted message bodies for a direct
/// chat (docs/protocol/chat-actions.md section 6). Local-only; see the
/// `chat_clear` module for what the companion-sync mutation still needs.
pub(super) fn clear_chat(cli: &super::Cli, chat: &str) -> Result<(), Error> {
    let store = contacts_store(cli)?;
    let jid = whatsapp_api::model::jid::Jid::parse(chat)?;
    let summary = whatsapp_api::transport::chat_clear::clear_chat(&store, &jid)?;
    if cli.json {
        println!(
            "{}",
            serde_json::json!({
                "cleared_messages": summary.cleared_messages,
                "cleared_poll_secrets": summary.cleared_poll_secrets,
            })
        );
        return Ok(());
    }
    println!(
        "clear-chat: {} message(s) cleared locally, {} poll secret(s) removed; local-only, companion sync not implemented",
        summary.cleared_messages, summary.cleared_poll_secrets
    );
    Ok(())
}

/// Offline re-projection of authenticated incoming records with current parsers.
pub(super) fn reproject(cli: &super::Cli) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let published = whatsapp_api::transport::inbox::reproject(&store)?;
    println!(
        "reproject: {published} event(s) upgraded from saved authenticated captures; no network requests"
    );
    Ok(())
}

/// Offline list of receipts (read/played/retry) and their outcomes.
pub(super) fn receipts_status(cli: &super::Cli) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let outcomes = whatsapp_api::transport::receipts::outcomes(&store)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&outcomes)
                .map_err(|e| Error::invalid("receipts", e.to_string()))?
        );
        return Ok(());
    }
    println!("receipts: {} receipt(s) queued so far", outcomes.len());
    for r in &outcomes {
        println!(
            "  kind={} status={} message_id={}",
            r.kind, r.status, r.message_id
        );
    }
    Ok(())
}

pub(super) fn peer_keys(
    cli: &super::Cli,
    chat: &str,
    show: bool,
    refresh: bool,
) -> Result<(), Error> {
    let peer = whatsapp_api::model::jid::Jid::parse(chat)
        .map_err(|_| Error::invalid("peer_keys", "invalid chat JID"))?;
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let value = if show {
        serde_json::to_value(whatsapp_api::transport::peer_keys::inspect(&store, &peer)?)
            .map_err(|_| Error::invalid("peer_keys", "cannot encode inspection"))?
    } else {
        let id = whatsapp_api::transport::peer_keys::queue(&store, &peer, refresh)?;
        serde_json::json!({"action_id":id})
    };
    if !cli.json {
        if show && value.is_null() {
            println!("peer keys: no saved directory query; queue once with peer-keys --chat");
        } else if show {
            println!(
                "peer keys: saved directory evidence; diagnostic hashes, not safety numbers; pins preserved"
            );
        } else {
            println!(
                "peer keys: queued or saved action ID; use listen to transmit queued actions once; pins preserved"
            );
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&value)
            .map_err(|_| Error::invalid("peer_keys", "cannot encode result"))?
    );
    Ok(())
}

pub(super) fn peer_identity(cli: &super::Cli, chat: &str) -> Result<(), Error> {
    let peer = whatsapp_api::model::jid::Jid::parse(chat)
        .map_err(|_| Error::invalid("peer_identity", "invalid chat JID"))?;
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let state = whatsapp_api::transport::identity_conflicts::inspect_peer(&store, &peer)?;
    // Both formats are redacted; fingerprints are diagnostic public-key hashes,
    // not the WhatsApp safety-number algorithm or human verification status.
    let json = serde_json::to_string_pretty(&state)
        .map_err(|_| Error::invalid("peer_identity", "serialization failed"))?;
    if !cli.json {
        println!(
            "peer identity: saved primary PN/LID state; diagnostic hashes, not safety numbers"
        );
    }
    println!("{json}");
    Ok(())
}

/// Offline diagnostic observations, without exposing peer addresses or keys.
pub(super) fn identity_conflicts(cli: &super::Cli) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let rows = whatsapp_api::transport::identity_conflicts::list(&store)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows)
                .map_err(|_| Error::invalid("identity_conflict", "serialization failed"))?
        );
    } else {
        println!(
            "identity conflicts: {} unauthenticated observation(s); pins preserved",
            rows.len()
        );
        for row in rows {
            println!(
                "  ticket={} peer_ticket={} pinned={} observed={} observed_at={}",
                row.ticket,
                row.peer_ticket,
                row.pinned_fingerprint,
                row.observed_fingerprint,
                row.observed_at.0
            );
        }
    }
    Ok(())
}

/// Offline list of queued one-shot actions and their server outcomes.
pub(super) fn actions_status(cli: &super::Cli, show: Option<&str>) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    if let Some(id) = show {
        let mask = |text: String| {
            let mut out = String::new();
            let mut run = 0usize;
            let mut pending = String::new();
            for ch in text.chars() {
                if ch.is_ascii_digit() {
                    run += 1;
                    pending.push(ch);
                } else {
                    if run >= 7 {
                        out.push_str(&"*".repeat(run));
                    } else {
                        out.push_str(&pending);
                    }
                    pending.clear();
                    run = 0;
                    out.push(ch);
                }
            }
            if run >= 7 {
                out.push_str(&"*".repeat(run));
            } else {
                out.push_str(&pending);
            }
            out
        };
        match whatsapp_api::transport::actions::request(&store, id)? {
            Some(node) => println!("request: {}", mask(format!("{node:?}"))),
            None => println!("request: unknown action id"),
        }
        match whatsapp_api::transport::actions::reply(&store, id)? {
            Some(node) => println!("reply: {}", mask(format!("{node:?}"))),
            None => println!("reply: none saved"),
        }
        return Ok(());
    }
    let outcomes = whatsapp_api::transport::actions::outcomes(&store)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&outcomes)
                .map_err(|e| Error::invalid("actions", e.to_string()))?
        );
        return Ok(());
    }
    println!("actions: {} queued so far", outcomes.len());
    for a in &outcomes {
        println!(
            "  id={} kind={} label={} status={:?}{}",
            a.id,
            a.kind,
            a.label,
            a.status,
            match (a.error_code, a.error_text.as_deref()) {
                (None, None) => String::new(),
                (code, text) => format!(" error={:?}/{}", code, text.unwrap_or("")),
            }
        );
    }
    Ok(())
}

/// Mask a JID's user part for display, keeping only its first two characters
/// (e.g. a phone number's country-code digits) visible; the server part
/// (`s.whatsapp.net`, `lid`, …) is not identifying and stays visible.
fn mask_jid(jid: &str) -> String {
    let (user, server) = jid.split_once('@').unwrap_or((jid, ""));
    let visible: String = user.chars().take(2).collect();
    let hidden = "*".repeat(user.chars().count().saturating_sub(2));
    if server.is_empty() {
        format!("{visible}{hidden}")
    } else {
        format!("{visible}{hidden}@{server}")
    }
}

const QUEUED_NOTE: &str =
    "queued; `listen --send-outbox` (or any session) transmits it; inspect with `actions`";

/// Durably queue a block of `chat`. Offline: only queues; a session transmits it.
pub(super) fn block(cli: &super::Cli, chat: &str) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let jid = whatsapp_api::model::jid::Jid::parse(chat)?;
    let id = whatsapp_api::transport::blocklist::queue_block(&store, &jid)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({"id": id, "note": QUEUED_NOTE}))
                .map_err(|e| Error::invalid("block", e.to_string()))?
        );
    } else {
        println!("block: id={id}; {QUEUED_NOTE}");
    }
    Ok(())
}

/// Durably queue an unblock of `chat`. Offline: only queues; a session transmits it.
pub(super) fn unblock(cli: &super::Cli, chat: &str) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let jid = whatsapp_api::model::jid::Jid::parse(chat)?;
    let id = whatsapp_api::transport::blocklist::queue_unblock(&store, &jid)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({"id": id, "note": QUEUED_NOTE}))
                .map_err(|e| Error::invalid("unblock", e.to_string()))?
        );
    } else {
        println!("unblock: id={id}; {QUEUED_NOTE}");
    }
    Ok(())
}

/// Apply settled block/unblock/fetch actions and show the saved block list,
/// numbers masked. `refresh` additionally queues a blocklist fetch (offline).
pub(super) fn blocklist_status(cli: &super::Cli, refresh: bool, full: bool) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let applied = whatsapp_api::transport::blocklist::ingest(&store)?;
    if full {
        whatsapp_api::transport::blocklist::forget_dhash(&store)?;
    }
    let fetch_id = if refresh {
        Some(whatsapp_api::transport::blocklist::queue_fetch(&store)?)
    } else {
        None
    };
    let contacts = whatsapp_api::transport::blocklist::blocked(&store)?;
    let masked: Vec<serde_json::Value> = contacts
        .iter()
        .map(|c| {
            serde_json::json!({
                "jid": mask_jid(&c.jid),
                "display_name": c.display_name,
                "username": c.username,
            })
        })
        .collect();
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ingested": applied,
                "fetch_queued_id": fetch_id,
                "fetch_note": fetch_id.as_ref().map(|_| QUEUED_NOTE),
                "blocked": masked,
            }))
            .map_err(|e| Error::invalid("blocklist", e.to_string()))?
        );
        return Ok(());
    }
    println!(
        "blocklist: {applied} settled action(s) applied; {} blocked contact(s)",
        contacts.len()
    );
    if let Some(id) = &fetch_id {
        println!("blocklist: fetch id={id}; {QUEUED_NOTE}");
    }
    for c in &masked {
        println!(
            "  jid={} display_name={} username={}",
            c["jid"].as_str().unwrap_or(""),
            c["display_name"].as_str().unwrap_or(""),
            c["username"].as_str().unwrap_or(""),
        );
    }
    Ok(())
}

/// Durably queue a spam report against `chat`. Offline: only queues; a session
/// transmits it. Not live-verified (see `docs/protocol/chat-actions.md` §2).
pub(super) fn report(
    cli: &super::Cli,
    chat: &str,
    flow: Option<&str>,
    known_chat: Option<bool>,
) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let jid = whatsapp_api::model::jid::Jid::parse(chat)?;
    let id = whatsapp_api::transport::report::queue_report(
        &store,
        &jid,
        flow.unwrap_or(""),
        known_chat,
    )?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({"id": id, "note": QUEUED_NOTE}))
                .map_err(|e| Error::invalid("report", e.to_string()))?
        );
    } else {
        println!("report: id={id}; {QUEUED_NOTE}");
    }
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
pub(super) fn outbox_suspended(cli: &super::Cli) -> Result<(), Error> {
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let rows = whatsapp_api::transport::outgoing::identity_suspended(&store)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows)
                .map_err(|_| Error::invalid("outbox", "cannot encode identity suspensions"))?
        );
    } else {
        println!(
            "outbox-suspended: {} queued send(s) awaiting peer identity policy resolution",
            rows.len()
        );
        for row in rows {
            println!(
                "  {} peer_ticket={} reason={}",
                row.local_id, row.peer_ticket, row.reason
            );
        }
    }
    Ok(())
}

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

fn poll_inspection_summary(poll: &whatsapp_api::transport::polls::Poll) -> String {
    format!(
        "name_len={} options_count={} option_lengths={:?} selectable_count={} secret_present={}",
        poll.name.len(),
        poll.options.len(),
        poll.options.iter().map(String::len).collect::<Vec<_>>(),
        poll.selectable_count,
        poll.secret.is_some(),
    )
}

pub(super) async fn run(
    cli: &super::Cli,
    inspect: bool,
    replay_captures: bool,
    accept_authenticated_identity_changes: bool,
) -> Result<(), Error> {
    let identity_policy = if accept_authenticated_identity_changes {
        whatsapp_api::registration::request::IdentityChangePolicy::ApkAuthenticated
    } else {
        whatsapp_api::registration::request::IdentityChangePolicy::Pinned
    };
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
    if inspect {
        let own = account.own_jid.as_ref().map(|j| j.to_non_ad().to_string());
        for (key, value) in &captures {
            if !key.starts_with(b"received:") {
                continue;
            }
            let Ok(saved) = serde_json::from_slice::<serde_json::Value>(value) else {
                continue;
            };
            let Ok(plain) = serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()) else {
                continue;
            };
            {
                let fields =
                    whatsapp_api::signal::direct::payload_fields(&plain).unwrap_or_default();
                let id = std::str::from_utf8(key)
                    .ok()
                    .and_then(|k| k.rsplit_once(':'))
                    .map(|(_, id)| id.to_owned())
                    .unwrap_or_default();
                println!(
                    "inbox: payload id={id} fields={fields:?} len={}",
                    plain.len()
                );
            }
            {
                let fields =
                    whatsapp_api::signal::direct::payload_fields(&plain).unwrap_or_default();
                if fields.iter().all(|v| matches!(v, 1 | 6 | 35))
                    && let Some(text) = whatsapp_api::transport::quotes::decode(&plain)
                    && text.is_reply()
                {
                    let id = std::str::from_utf8(key)
                        .ok()
                        .and_then(|k| k.rsplit_once(':'))
                        .map(|(_, id)| id.to_owned())
                        .unwrap_or_default();
                    let participant = text
                        .participant
                        .as_deref()
                        .and_then(|p| whatsapp_api::model::jid::Jid::parse(p).ok());
                    let sender = std::str::from_utf8(key)
                        .ok()
                        .and_then(|k| k.strip_prefix("received:"))
                        .and_then(|k| k.rsplit_once(':'))
                        .and_then(|(sender, _)| whatsapp_api::model::jid::Jid::parse(sender).ok());
                    let from_lid = sender
                        .as_ref()
                        .and_then(|s| whatsapp_api::transport::inbox::lid_for(&store, s).ok())
                        .flatten();
                    let (own_pn, own_lid) = whatsapp_api::transport::inbox::own_identities(&store)
                        .unwrap_or((None, None));
                    let same = |a: &Option<whatsapp_api::model::jid::Jid>| matches!((participant.as_ref(), a), (Some(p), Some(a)) if p.to_non_ad() == a.to_non_ad());
                    println!(
                        "inbox: reply id={id} text_len={} stanza_id={:?} participant_server={:?} participant_is_sender_pn={} participant_is_sender_lid={} participant_is_own_pn={} participant_is_own_lid={} quoted_fields={:?}",
                        text.text.len(),
                        text.stanza_id,
                        participant.as_ref().map(|p| format!("{:?}", p.server)),
                        same(&sender),
                        same(&from_lid),
                        same(&own_pn),
                        same(&own_lid),
                        text.quoted
                            .as_deref()
                            .map(|q| whatsapp_api::signal::direct::payload_fields(q)
                                .unwrap_or_default()),
                    );
                }
            }
            if let Some(poll) = whatsapp_api::transport::polls::decode_poll(&plain) {
                let id = std::str::from_utf8(key)
                    .ok()
                    .and_then(|k| k.rsplit_once(':'))
                    .map(|(_, id)| id.to_owned())
                    .unwrap_or_default();
                println!(
                    "inbox: poll id={id} {}; values redacted",
                    poll_inspection_summary(&poll)
                );
            }
            if let Some(vote) = whatsapp_api::transport::polls::decode_vote(&plain) {
                let id = std::str::from_utf8(key)
                    .ok()
                    .and_then(|k| k.rsplit_once(':'))
                    .map(|(_, id)| id.to_owned())
                    .unwrap_or_default();
                let sender_pn = std::str::from_utf8(key)
                    .ok()
                    .and_then(|k| k.strip_prefix("received:"))
                    .and_then(|r| r.rsplit_once(':'))
                    .map(|(s, _)| s.to_owned())
                    .unwrap_or_default();
                let fields = whatsapp_api::transport::polls::vote_key_fields(&plain);
                let (remote, from_me, metadata, ts) = fields
                    .map(|f| {
                        (
                            f.remote_jid,
                            f.from_me,
                            f.metadata_present,
                            f.sender_timestamp_ms,
                        )
                    })
                    .unwrap_or((None, None, false, None));
                let remote_server = remote
                    .as_deref()
                    .and_then(|v| v.rsplit_once('@'))
                    .map(|(_, s)| s.to_owned());
                let remote_is_sender_pn = remote.as_deref() == Some(sender_pn.as_str());
                // Which identity pair decrypts: creator in {pn, lid}, voter in {pn, lid}.
                let sender_jid = whatsapp_api::model::jid::Jid::parse(&sender_pn).ok();
                let sender_lid = sender_jid.as_ref().and_then(|j| {
                    whatsapp_api::transport::inbox::lid_for(&store, j)
                        .ok()
                        .flatten()
                });
                let (own_pn, own_lid) =
                    whatsapp_api::transport::inbox::own_identities(&store).unwrap_or((None, None));
                let poll_key_theirs = format!("received:{sender_pn}:{}", vote.poll_id.as_str());
                let (secret, options): (Option<[u8; 32]>, Vec<String>) = if vote.poll_from_sender {
                    store
                        .read(|tx| {
                            tx.get_json::<serde_json::Value>(
                                Table::Checkpoints,
                                poll_key_theirs.as_bytes(),
                                1,
                            )
                        })
                        .ok()
                        .flatten()
                        .and_then(|saved| {
                            serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()).ok()
                        })
                        .and_then(|p| whatsapp_api::transport::polls::decode_poll(&p))
                        .map(|p| (p.secret, p.options))
                        .unwrap_or((None, vec![]))
                } else {
                    store
                        .read(|tx| {
                            tx.get_json::<serde_json::Value>(
                                Table::Checkpoints,
                                format!("poll-secret:{}", vote.poll_id.as_str()).as_bytes(),
                                1,
                            )
                        })
                        .ok()
                        .flatten()
                        .map(|own| {
                            (
                                serde_json::from_value::<Vec<u8>>(own["secret"].clone())
                                    .ok()
                                    .and_then(|s| <[u8; 32]>::try_from(s).ok()),
                                serde_json::from_value(own["options"].clone()).unwrap_or_default(),
                            )
                        })
                        .unwrap_or((None, vec![]))
                };
                let mut worked = Vec::new();
                if let Some(secret) = secret {
                    let creators: Vec<(&str, Option<whatsapp_api::model::jid::Jid>)> =
                        if vote.poll_from_sender {
                            vec![
                                ("sender_pn", sender_jid.clone()),
                                ("sender_lid", sender_lid.clone()),
                            ]
                        } else {
                            vec![("own_pn", own_pn.clone()), ("own_lid", own_lid.clone())]
                        };
                    let voters = [
                        ("sender_pn", sender_jid.clone()),
                        ("sender_lid", sender_lid.clone()),
                    ];
                    for (cl, c) in &creators {
                        for (vl, v) in &voters {
                            if let (Some(c), Some(v)) = (c, v)
                                && whatsapp_api::transport::polls::decrypt_vote(
                                    &secret,
                                    vote.poll_id.as_str(),
                                    c,
                                    v,
                                    &vote,
                                )
                                .is_ok()
                            {
                                worked.push(format!("creator={cl} voter={vl}"));
                            }
                        }
                    }
                }
                let _ = options;
                println!(
                    "inbox: vote id={id} poll={} poll_from_sender={} remote_server={remote_server:?} remote_is_sender_pn={remote_is_sender_pn} key_from_me={from_me:?} metadata_present={metadata} sender_ts_present={} iv_len={} decrypts_with={worked:?}",
                    vote.poll_id.as_str(),
                    vote.poll_from_sender,
                    ts.is_some(),
                    vote.enc_iv.len()
                );
            }
            if let Some(reaction) = whatsapp_api::transport::reactions::decode(&plain)
                && let Some(key_fields) = whatsapp_api::transport::reactions::key_fields(&plain)
            {
                let sender = std::str::from_utf8(key)
                    .ok()
                    .and_then(|k| k.strip_prefix("received:"))
                    .and_then(|rest| rest.rsplit_once(':'))
                    .map(|(sender, _)| sender.to_owned());
                let fields =
                    whatsapp_api::signal::direct::payload_fields(&plain).unwrap_or_default();
                println!(
                    "inbox: reaction payload fields={:?} target_id={} from_me={} remote_jid_is_sender={:?} remote_jid_is_own_account={:?} remote_jid_server={:?} participant_present={} emoji_len={}",
                    fields,
                    reaction.target_id.as_str(),
                    reaction.target_from_sender,
                    sender
                        .as_deref()
                        .map(|s| key_fields.0.as_deref() == Some(s)),
                    own.as_deref().map(|o| key_fields.0.as_deref() == Some(o)),
                    key_fields
                        .0
                        .as_deref()
                        .and_then(|v| v.rsplit_once('@'))
                        .map(|(_, s)| s.to_owned()),
                    key_fields.1,
                    reaction.emoji.as_deref().map_or(0, str::len)
                );
                if let Some(shape) = whatsapp_api::transport::reactions::shape(&plain) {
                    println!("inbox: reaction shape {shape}");
                }
            }
        }
    }
    let mut replayed = (0usize, 0usize, 0usize);
    if replay_captures {
        for (key, value) in &captures {
            if !is_capture(key) {
                continue;
            }
            let Ok(wire) = serde_json::from_slice::<Vec<u8>>(value) else {
                continue;
            };
            let Ok(raw) = unpack_frame_payload(&wire, &limits) else {
                continue;
            };
            let Ok(node) =
                Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits).decode_node()
            else {
                continue;
            };
            if node.description != "message" {
                continue;
            }
            // The same durable path a live session uses: dedupe, atomic commit,
            // no network. A stanza already committed returns Duplicate untouched.
            match whatsapp_api::transport::inbox::process_with_policy(
                &store,
                &account,
                &node,
                &wire,
                identity_policy,
            )
            .await?
            {
                whatsapp_api::transport::inbox::IncomingResult::Decrypted => replayed.0 += 1,
                whatsapp_api::transport::inbox::IncomingResult::Duplicate => replayed.1 += 1,
                // Offline replay sends nothing, so an undecryptable capture is
                // counted with the unsupported rather than answered.
                whatsapp_api::transport::inbox::IncomingResult::Unsupported
                | whatsapp_api::transport::inbox::IncomingResult::Undecryptable => replayed.2 += 1,
            }
        }
        println!(
            "inbox: replayed preserved captures: {} newly decrypted, {} already committed, {} still unsupported (no network, no receipts sent)",
            replayed.0, replayed.1, replayed.2
        );
    }
    for (key, value) in &captures {
        if inspect {
            if is_capture(key) {
                let wire: Vec<u8> = serde_json::from_slice(value)
                    .map_err(|_| Error::invalid("inbox", "invalid capture"))?;
                let raw = unpack_frame_payload(&wire, &limits)?;
                let node = Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits)
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
                    let kind =
                        capture_protocol_type(node.get_attr("type").and_then(|v| v.as_text()));
                    let offline = node
                        .child("offline")
                        .and_then(|n| n.get_attr("count"))
                        .and_then(|v| v.as_text())
                        .and_then(|v| v.parse::<u32>().ok());
                    let attr_names: Vec<&str> = node
                        .attributes
                        .iter()
                        .map(|a| a.name.as_str())
                        .filter(|n| safe(n))
                        .collect();
                    let child_attrs: Vec<String> = node
                        .child_nodes()
                        .iter()
                        .filter(|n| safe(&n.description))
                        .map(|n| {
                            let names: Vec<&str> = n
                                .attributes
                                .iter()
                                .map(|a| a.name.as_str())
                                .filter(|v| safe(v))
                                .collect();
                            let mut shown = format!("{}{names:?}", n.description);
                            // `meta` flags and an `enc` child's version/type are
                            // protocol routing metadata, not message content, and
                            // knowing which envelope a stanza used is the first
                            // thing needed when one is refused as unsupported.
                            // `offline`/`offline_preview` carry only per-class
                            // queue counts, which are decisive when diagnosing
                            // whether the server ever queued a class of stanza
                            // for this account at all.
                            // An `error` child's code and text are the server's
                            // own refusal reason and carry no identifiers; they
                            // are the whole point of inspecting a refusal.
                            if matches!(
                                n.description.as_str(),
                                "meta" | "enc" | "offline" | "offline_preview" | "error"
                            ) {
                                // These carry protocol flags and versions only
                                // (`v`, `type`, `content_type`, `decrypt-fail`),
                                // never identities, so a short numeric value such
                                // as the envelope version `v=2` is shown. `safe`
                                // rejects digits, which would redact exactly the
                                // version needed to diagnose a refused envelope.
                                let safe_flag = |value: &str| {
                                    value.len() <= 64
                                        && value.bytes().all(|b| {
                                            b.is_ascii_alphanumeric()
                                                || matches!(b, b'_' | b'-' | b' ' | b'.' | b':')
                                        })
                                };
                                let values: Vec<String> = n
                                    .attributes
                                    .iter()
                                    .filter(|a| safe(&a.name))
                                    .map(|a| {
                                        let v = a.value.as_text().unwrap_or("<non-text>");
                                        format!(
                                            "{}={}",
                                            a.name,
                                            if safe_flag(v) { v } else { "<unsafe>" }
                                        )
                                    })
                                    .collect();
                                shown = format!("{}{values:?}", n.description);
                            }
                            shown
                        })
                        .collect();
                    let id8 = node
                        .get_attr("id")
                        .and_then(|v| v.as_text())
                        .map(|v| v.chars().take(8).collect::<String>())
                        .unwrap_or_default();
                    // An `ack` for a fan-out answers with the participant hash
                    // the server expects and how many devices it counted. Both
                    // are the server's own bookkeeping, not identifying, and
                    // without them a send that is acknowledged but undecryptable
                    // on the handset gives no clue which list was wrong.
                    // A `receipt type="retry"` is the recipient telling us it
                    // could not decrypt: `count` is which attempt this is, `v`
                    // the retry protocol version, and `<registration>` carries
                    // their current registration id, which is how a stale
                    // session is detected. The counts and version are not
                    // identifying; the registration id is reported by length.
                    let retry = if node.description == "receipt"
                        && node.get_attr("type").and_then(|v| v.as_text()) == Some("retry")
                    {
                        let child = node.child("retry");
                        let attr = |name: &str| {
                            child
                                .and_then(|c| c.get_attr(name))
                                .and_then(|v| v.as_text())
                                .filter(|v| v.len() <= 8)
                                .map(str::to_owned)
                        };
                        let registration = node.child("registration").map(|c| match &c.content {
                            whatsapp_api::binary::NodeContent::Bytes(b) => b.len(),
                            _ => 0,
                        });
                        // Which member failed matters: a fan-out that gets one
                        // retry out of three addressed devices is a per-device
                        // problem, not a broken stanza. The tail is the same
                        // masking `group list` uses, so the two can be compared
                        // without either printing a full identity.
                        let participant_tail = node
                            .get_attr("participant")
                            .and_then(|v| {
                                v.as_jid().cloned().or_else(|| {
                                    whatsapp_api::model::jid::Jid::parse(v.as_text()?).ok()
                                })
                            })
                            .map(|j| {
                                format!(
                                    "…{}@{:?}/device={}",
                                    j.user
                                        .chars()
                                        .rev()
                                        .take(3)
                                        .collect::<String>()
                                        .chars()
                                        .rev()
                                        .collect::<String>(),
                                    j.server,
                                    j.device
                                )
                            });
                        format!(
                            ", retry_count={:?}, retry_v={:?}, registration_bytes={:?}, participant={:?}",
                            attr("count"),
                            attr("v"),
                            registration,
                            participant_tail
                        )
                    } else {
                        String::new()
                    };
                    let ack = if node.description == "ack" {
                        let show = |name: &str| {
                            node.get_attr(name)
                                .and_then(|v| v.as_text())
                                .filter(|v| v.len() <= 32)
                                .map(str::to_owned)
                        };
                        format!(
                            ", class={:?}, phash={:?}, count={:?}, error={:?}, ack_addressing_mode={:?}, from_server={:?}",
                            show("class"),
                            show("phash"),
                            show("count"),
                            show("error"),
                            show("addressing_mode"),
                            node.get_attr("from")
                                .and_then(|v| v.as_jid())
                                .map(|j| format!("{:?}", j.server))
                        )
                    } else {
                        String::new()
                    };
                    let codes = if matches!(
                        node.description.as_str(),
                        "stream:error" | "failure" | "success"
                    ) {
                        // Server codes carry no personal data, but a `<failure>`
                        // also carries an `appeal_token`, which is an account
                        // credential, and `<success>` carries our own LID and
                        // phone number. Report presence and length, never the
                        // value, and bound everything else rather than trusting
                        // the server not to put a secret in a new attribute.
                        const SECRET: [&str; 5] =
                            ["appeal_token", "token", "challenge", "lid", "pn"];
                        let values: Vec<String> = node
                            .attributes
                            .iter()
                            .map(|a| {
                                if SECRET.contains(&a.name.as_str()) {
                                    return format!("{}=<present, redacted>", a.name);
                                }
                                match a.value.as_text() {
                                    Some(v) if v.len() <= 24 => format!("{}={v}", a.name),
                                    Some(v) => {
                                        format!("{}=<{} chars, redacted>", a.name, v.len())
                                    }
                                    None => format!("{}=<non-text>", a.name),
                                }
                            })
                            .collect();
                        format!(", values={values:?}")
                    } else {
                        String::new()
                    };
                    // How the receive path would route this stanza. Server kinds
                    // and a yes/no are not identifying, and without them a stanza
                    // refused as unsupported gives no clue which guard rejected it.
                    let routing = if node.description == "message" {
                        let server_of = |name: &str| {
                            node.get_attr(name)
                                .and_then(|v| {
                                    v.as_jid().cloned().or_else(|| {
                                        whatsapp_api::model::jid::Jid::parse(v.as_text()?).ok()
                                    })
                                })
                                // `kind` records which wire form the JID arrived
                                // in: `AgentDevice` means an AD-JID (an explicit
                                // *device* address), `Plain` a JID pair (a user).
                                // An outgoing fan-out has to match that choice.
                                .map(|j| format!("{:?}/device={}/{:?}", j.server, j.device, j.kind))
                        };
                        // `addressing_mode` is a closed enum ("pn"/"lid"), not a
                        // value derived from the account, and it decides which JID
                        // form an outgoing sender-key fan-out has to address.
                        format!(
                            ", from_server={:?}, participant_server={:?}, addressing_mode={:?}, routes_as_group={}",
                            server_of("from"),
                            server_of("participant"),
                            node.get_attr("addressing_mode")
                                .and_then(|v| v.as_text())
                                .filter(|v| matches!(*v, "pn" | "lid")),
                            whatsapp_api::transport::inbox::group_sender(&node).is_some()
                        )
                    } else {
                        String::new()
                    };
                    // A disappearing-message timer and the message's age decide
                    // whether a later reply or reaction still has a target.
                    let number = |name: &str| {
                        node.get_attr(name)
                            .and_then(|v| v.as_text())
                            .and_then(|v| v.parse::<i64>().ok())
                    };
                    let sender_tail = node
                        .get_attr("participant")
                        .and_then(|v| v.as_jid())
                        .map(|j| {
                            let raw = j.to_non_ad().to_string();
                            let tail: String = raw
                                .chars()
                                .rev()
                                .take(9)
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect();
                            format!(", participant_tail={tail}")
                        })
                        .unwrap_or_default();
                    let timing = match (number("expiration"), number("t")) {
                        (None, None) => String::new(),
                        (expiration, t) => format!(
                            ", expiration_s={expiration:?}, age_s={:?}",
                            t.map(|t| whatsapp_api::model::time::Timestamp::now().0 / 1000 - t)
                        ),
                    };
                    let pins = match prekey_pin_summary(&store, &node) {
                        Ok(summary) => summary,
                        Err(Error::InvalidInput { .. }) => {
                            ", prekey_pin_inspection=invalid_input".into()
                        }
                        Err(error) => return Err(error),
                    };
                    capture_shapes.insert(format!("inbox: capture shape tag={}, type={kind}, id={id8}, attrs={attr_names:?}{codes}{ack}{retry}{routing}{timing}{sender_tail}{pins}, children={children:?}, child_attrs={child_attrs:?}, offline_count={offline:?}",node.description));
                }
            }
            if key.starts_with(b"received:") {
                let mut saved: serde_json::Value = serde_json::from_slice(value)
                    .map_err(|_| Error::invalid("inbox", "invalid saved payload"))?;
                if saved["status"] == "consumed" {
                    println!("inbox: view-once attachment consumed; content unavailable");
                    continue;
                }
                if saved["status"] == "cleared" {
                    // `clear_chat` strips `protobuf` and keeps the row so
                    // redeliveries still dedupe. Skip it rather than failing the
                    // whole inspection on a row that intentionally has no body.
                    println!("inbox: chat cleared; content unavailable");
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
        if !is_capture(key) {
            continue;
        }
        let bytes: Vec<u8> = serde_json::from_slice(value)
            .map_err(|_| Error::invalid("inbox", "invalid capture"))?;
        let unpacked = unpack_frame_payload(&bytes, &limits)?;
        let node = Decoder::new(&unpacked, TokenDictionary::comparison_2_26_37_71(), &limits)
            .decode_node()?;
        match whatsapp_api::transport::inbox::process_with_policy(
            &store,
            &account,
            &node,
            key,
            identity_policy,
        )
        .await?
        {
            whatsapp_api::transport::inbox::IncomingResult::Decrypted => {
                count += 1;
                println!("inbox: authenticated payload committed; content redacted");
            }
            whatsapp_api::transport::inbox::IncomingResult::Duplicate => {}
            whatsapp_api::transport::inbox::IncomingResult::Unsupported => {}
            whatsapp_api::transport::inbox::IncomingResult::Undecryptable => {
                println!("inbox: a saved payload could not be decrypted; no receipt sent offline");
            }
        }
    }
    for shape in capture_shapes {
        println!("{shape}");
    }
    if inspect {
        // Which request produced which wire id, so an `ack` or `receipt` above
        // (listed by the first eight characters of its id) can be tied to the
        // send that caused it. Request ids are the operator's own labels.
        // Where each received message came from, last four characters only —
        // enough to check a reply or reaction names the right sender.
        for (key, _) in &captures {
            if let Some((sender, id)) = std::str::from_utf8(key)
                .ok()
                .and_then(|k| k.strip_prefix("received:"))
                .and_then(|rest| rest.rsplit_once(':'))
            {
                let tail: String = sender
                    .chars()
                    .rev()
                    .take(9)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                println!(
                    "inbox: received id={} sender_tail={tail}",
                    id.chars().take(8).collect::<String>()
                );
            }
        }
        for (key, value) in &captures {
            let Some(request) = std::str::from_utf8(key)
                .ok()
                .and_then(|k| k.strip_prefix("message:"))
                .and_then(|k| k.strip_suffix(":send"))
            else {
                continue;
            };
            let Ok(saved) = serde_json::from_slice::<serde_json::Value>(value) else {
                continue;
            };
            let wire = saved["wire_id"]
                .as_str()
                .map(|w| w.chars().take(8).collect::<String>());
            println!(
                "inbox: send request={request} wire_id={} status={}",
                wire.as_deref().unwrap_or("?"),
                saved["status"].as_str().unwrap_or("?")
            );
        }
    }
    println!("inbox: {count} newly decrypted payloads; no network requests");
    Ok(())
}

fn mask_number_in(text: &str) -> String {
    // Keep only the last three digits of any long digit run (phone numbers).
    let mut out = String::new();
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if run.len() >= 7 {
            out.push_str(&"*".repeat(run.len() - 3));
            out.push_str(&run[run.len() - 3..]);
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            run.push(ch);
        } else {
            flush(&mut run, &mut out);
            out.push(ch);
        }
    }
    flush(&mut run, &mut out);
    out
}

/// Group management: every subcommand queues one action (a session
/// transmits it) except `list`, which also ingests historical replies and prints snapshots.
fn fanout_inspection(
    plan: &whatsapp_api::transport::group_send::FanoutPlan,
    reveal_hash: bool,
) -> serde_json::Value {
    let mut output = serde_json::json!({
        "offline_only": true,
        "addressing_mode": plan.addressing_mode,
        "recipients": plan.recipients.len(),
        "devices": plan.devices.len(),
        "without_device_list": plan.without_device_list.len(),
        "own_devices_unknown": plan.own_devices_unknown,
        "participant_hash_version": "2",
        "participant_hash_device_count": plan.participant_hash_device_count,
    });
    if reveal_hash {
        output["participant_hash"] = serde_json::json!(plan.participant_hash);
    }
    output
}

pub(super) fn group(cli: &super::Cli, cmd: &super::GroupCmd) -> Result<(), Error> {
    use whatsapp_api::model::jid::Jid;
    use whatsapp_api::transport::groups;
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let jids = |list: &[String]| -> Result<Vec<Jid>, Error> {
        list.iter().map(|j| Jid::parse(j)).collect()
    };
    let queued = |id: String| {
        println!(
            "group: action {id} queued; a session transmits it and applies the reply; `group list` inspects snapshots"
        );
        Ok(())
    };
    match cmd {
        super::GroupCmd::FanoutInspect { chat, own, hash } => {
            let parse = |value: &str| {
                Jid::parse(value)
                    .map_err(|_| Error::invalid("group_send", "invalid inspection address"))
            };
            let plan =
                whatsapp_api::transport::group_send::plan(&store, &parse(chat)?, &parse(own)?)?;
            let output = fanout_inspection(&plan, *hash);
            println!(
                "{}",
                serde_json::to_string_pretty(&output).map_err(|_| Error::invalid(
                    "group_send",
                    "cannot encode fan-out inspection"
                ))?
            );
            Ok(())
        }
        super::GroupCmd::Create {
            subject,
            participants,
        } => queued(groups::queue_create(&store, subject, &jids(participants)?)?),
        super::GroupCmd::Info { chat } => queued(groups::queue_info(&store, &Jid::parse(chat)?)?),
        super::GroupCmd::Refresh { chats, follow_up } => {
            let chats = jids(chats)?;
            queued(if *follow_up {
                groups::queue_refresh_follow_up(&store, &chats)?
            } else {
                groups::queue_refresh(&store, &chats)?
            })
        }
        super::GroupCmd::RefreshStatus { id } => {
            let summary = groups::refresh_outcome(&store, id)?.map(|v| {
                serde_json::json!({
                    "complete": v.complete, "truncated": v.truncated.len(),
                    "forbidden": v.forbidden, "not_found": v.not_found,
                    "automatic_follow_up": v.automatic_follow_up, "round": v.round,
                    "next_action": v.next_action, "pending": v.pending.len(), "exhausted": v.exhausted,
                })
            });
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary)
                        .map_err(|e| Error::invalid("group", e.to_string()))?
                );
            } else if let Some(summary) = summary {
                println!(
                    "group refresh: complete={} truncated={} forbidden={} not_found={} pending={} round={} exhausted={} next={}",
                    summary["complete"],
                    summary["truncated"],
                    summary["forbidden"],
                    summary["not_found"],
                    summary["pending"],
                    summary["round"],
                    summary["exhausted"],
                    summary["next_action"]
                );
            } else {
                println!(
                    "group refresh: no saved projection outcome; inspect actions for pending/refused/unapplied status"
                );
            }
            Ok(())
        }
        super::GroupCmd::Subject { chat, text } => {
            queued(groups::queue_subject(&store, &Jid::parse(chat)?, text)?)
        }
        super::GroupCmd::Description { chat, text, prev } => queued(groups::queue_description(
            &store,
            &Jid::parse(chat)?,
            text,
            prev.as_deref(),
        )?),
        super::GroupCmd::Add { chat, participants } => queued(groups::queue_add(
            &store,
            &Jid::parse(chat)?,
            &jids(participants)?,
        )?),
        super::GroupCmd::Remove { chat, participants } => queued(groups::queue_remove(
            &store,
            &Jid::parse(chat)?,
            &jids(participants)?,
        )?),
        super::GroupCmd::Promote { chat, participants } => queued(groups::queue_promote(
            &store,
            &Jid::parse(chat)?,
            &jids(participants)?,
        )?),
        super::GroupCmd::Demote { chat, participants } => queued(groups::queue_demote(
            &store,
            &Jid::parse(chat)?,
            &jids(participants)?,
        )?),
        super::GroupCmd::Leave { chat } => queued(groups::queue_leave(&store, &Jid::parse(chat)?)?),
        super::GroupCmd::Invite { chat, revoke } => {
            queued(groups::queue_invite(&store, &Jid::parse(chat)?, *revoke)?)
        }
        super::GroupCmd::List => {
            let applied = groups::ingest(&store)?;
            let all = groups::list(&store)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&all)
                        .map_err(|e| Error::invalid("group", e.to_string()))?
                );
                return Ok(());
            }
            println!(
                "groups: {applied} settled action(s) applied; {} group(s) known",
                all.len()
            );
            for g in &all {
                println!(
                    "  {} subject={:?} members={} creator={} invite={}",
                    mask_number_in(&g.jid),
                    g.subject,
                    g.members.len(),
                    g.creator
                        .as_deref()
                        .map(mask_number_in)
                        .unwrap_or_else(|| "-".into()),
                    g.invite_code
                        .as_deref()
                        .map(|c| format!("https://chat.whatsapp.com/{c}"))
                        .unwrap_or_else(|| "-".into())
                );
                for m in &g.members {
                    println!(
                        "    {} {}",
                        mask_number_in(&m.jid),
                        m.role.as_deref().unwrap_or("member")
                    );
                }
            }
            Ok(())
        }
    }
}

/// Two-account lab check: `from` queues one text to `to`'s own number, both
/// sessions run at once, then the sender's delivery facts and the receiver's
/// journal are inspected. Profiles must not share a store or a key variable.
pub(super) async fn lab_pair(
    cli: &super::Cli,
    from: &str,
    to: &str,
    text: &str,
    seconds: u64,
    allow_live: bool,
) -> Result<(), Error> {
    use sha2::Digest;
    use whatsapp_api::model::{
        ids::{ChatId, ClientRequestId, EventCursor},
        jid::Jid,
        message::MessageContent,
        send::SendRequest,
    };
    use whatsapp_api::store::ConsumerId;
    use whatsapp_api::sync::SyncEvent;
    if !allow_live {
        return Err(Error::invalid("lab-pair", "requires --allow-live-request"));
    }
    if from == to {
        return Err(Error::invalid("lab-pair", "from and to must differ"));
    }
    let a = cli.for_profile(from)?;
    let b = cli.for_profile(to)?;
    if a.store == b.store {
        return Err(Error::invalid("lab-pair", "profiles share a store"));
    }
    if a.key_env == b.key_env {
        return Err(Error::invalid("lab-pair", "profiles share a key variable"));
    }
    let cfg_a = a.config_path(None, None)?;
    let cfg_b = b.config_path(None, None)?;
    let open = |c: &super::Cli| {
        whatsapp_api::WhatsAppClient::builder()
            .store_path(c.store.clone())
            .account_id(AccountId::new(c.account.clone()).expect("validated account id"))
            .key_source(Arc::new(EnvKeySource::new(c.key_env.clone())))
            .create_store(false)
            .build()
    };
    let own = |client: &whatsapp_api::WhatsAppClient, name: &str| -> Result<Jid, Error> {
        client
            .store()
            .read(AccountRecord::load)?
            .and_then(|r| r.own_jid)
            .map(|j| j.to_non_ad())
            .ok_or_else(|| {
                Error::invalid(
                    "lab-pair",
                    format!("profile {name} has no registered own jid"),
                )
            })
    };
    // Receiver: note the journal position before the run.
    let client_b = open(&b).await?;
    let own_b = own(&client_b, to)?;
    let consumer = ConsumerId::new("lab-pair")?;
    let engine = client_b.events(consumer.clone());
    let mut before = engine.acknowledged_cursor().await?;
    loop {
        let batch = engine.next_batch(before, 4096).await?;
        if batch.events.is_empty() {
            break;
        }
        before = batch.ack_cursor;
        if !batch.has_more {
            break;
        }
    }
    drop(engine);
    client_b.shutdown().await;
    // Sender: queue the text to the receiver's own number.
    let client_a = open(&a).await?;
    let own_a = own(&client_a, from)?;
    let digest = sha2::Sha256::digest(format!(
        "{text}\n{}",
        whatsapp_api::model::time::Timestamp::now().0
    ));
    let ticket = client_a
        .send_message(SendRequest {
            client_request_id: ClientRequestId::new(format!("pair-{}", hex::encode(&digest[..8])))?,
            chat: ChatId::from_jid(&own_b)?,
            content: MessageContent::Text {
                body: text.to_owned(),
                quoted: None,
                style: Default::default(),
            },
        })
        .await?;
    client_a.shutdown().await;
    println!(
        "lab-pair: queued text from profile {from} to profile {to} (local id {}); running both sessions for {seconds}s",
        ticket.local_id
    );
    let (ra, rb) = tokio::join!(
        listen(&a, &cfg_a, true, seconds, true),
        listen(&b, &cfg_b, true, seconds, false)
    );
    if let Err(e) = &ra {
        println!("lab-pair: sender session ended with error: {e}");
    }
    if let Err(e) = &rb {
        println!("lab-pair: receiver session ended with error: {e}");
    }
    // Sender facts.
    let client_a = open(&a).await?;
    let status = client_a.message_status(&ticket).await?;
    client_a.shutdown().await;
    let facts: Vec<String> = status
        .facts
        .iter()
        .map(|f| format!("{:?}", f.kind))
        .collect();
    println!(
        "lab-pair: sender progress={:?} facts={facts:?}",
        status.progress
    );
    // Receiver journal.
    let client_b = open(&b).await?;
    let engine = client_b.events(consumer);
    let mut cursor: EventCursor = before;
    let mut received = false;
    let mut new_events = 0usize;
    loop {
        let batch = engine.next_batch(cursor, 4096).await?;
        if batch.events.is_empty() {
            break;
        }
        for e in &batch.events {
            new_events += 1;
            if let SyncEvent::NewMessage(m) = &e.event
                && let MessageContent::Text { body, .. } = &m.content
                && body == text
                && m.key.chat.jid().to_non_ad().user == own_a.user
            {
                received = true;
            }
        }
        cursor = batch.ack_cursor;
        if !batch.has_more {
            break;
        }
    }
    drop(engine);
    client_b.shutdown().await;
    println!(
        "lab-pair: receiver saw {new_events} new event(s); text from the sender received={received}"
    );
    if !received {
        return Err(Error::invalid(
            "lab-pair",
            "the receiver did not project the sender's text; inspect both accounts' events",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use whatsapp_api::binary::Node;
    use whatsapp_api::model::jid::Jid;

    #[test]
    fn device_query_inspection_keeps_users_and_unknown_codes_private() {
        use whatsapp_api::{model::time::Timestamp, transport::devices::QueryRecord};
        let mut record = QueryRecord {
            users: vec![Jid::parse("15555550101@s.whatsapp.net").unwrap()],
            status: "refused".into(),
            code: Some("PRIVATE".into()),
            ingested: 0,
            partial_errors: 0,
            omitted_users: Some(1),
            generations: None,
            superseded_users: None,
            verification_failures: None,
            requested_at: Timestamp::now(),
        };
        let out = device_query_inspection(Some(&record));
        assert_eq!(out["requested_users"], 1);
        assert_eq!(out["omitted_users"], 1);
        assert_eq!(out["superseded_users"], serde_json::Value::Null);
        assert!(out.get("generations").is_none());
        assert_eq!(out["verification_errors"], serde_json::Value::Null);
        assert_eq!(out["verification_failures"], serde_json::Value::Null);
        assert_eq!(out["code"], serde_json::Value::Null);
        for private in ["15555550101", "PRIVATE"] {
            assert!(!out.to_string().contains(private));
        }
        record.omitted_users = None;
        record.superseded_users = Some(1);
        record.generations = Some(vec![7]);
        record.verification_failures = Some(std::collections::BTreeMap::from([(
            whatsapp_api::transport::devices::DeviceVerificationFailure::MissingPinnedIdentity,
            1,
        )]));
        record.code = Some("503".into());
        let historical = device_query_inspection(Some(&record));
        assert_eq!(historical["omitted_users"], serde_json::Value::Null);
        assert_eq!(historical["code"], "503");
        assert_eq!(historical["superseded_users"], 1);
        assert!(historical.get("generations").is_none());
        assert_eq!(historical["verification_errors"], 1);
        assert_eq!(
            historical["verification_failures"]["missing_pinned_identity"],
            1
        );
        assert_eq!(
            device_query_inspection(None),
            serde_json::json!({"offline_only":true,"found":false})
        );
    }

    #[test]
    fn fanout_inspection_reports_unknowns_without_identifiers_or_implicit_hash() {
        let plan = whatsapp_api::transport::group_send::FanoutPlan {
            chat: Jid::parse("120363000000000001@g.us").unwrap(),
            own: Jid::parse("111111111111111@lid").unwrap(),
            addressing_mode: "lid",
            recipients: vec![Jid::parse("222222222222222@lid").unwrap()],
            devices: vec![Jid::parse("222222222222222:3@lid").unwrap()],
            without_device_list: vec![Jid::parse("333333333333333@lid").unwrap()],
            own_devices_unknown: true,
            participant_hash: "2:private!".into(),
            participant_hash_device_count: 3,
        };
        let output = fanout_inspection(&plan, false);
        assert_eq!(output["without_device_list"], 1);
        assert_eq!(output["participant_hash_device_count"], 3);
        assert_eq!(output["own_devices_unknown"], true);
        assert!(output.get("participant_hash").is_none());
        let rendered = output.to_string();
        for secret in [
            "120363000000000001",
            "111111111111111",
            "222222222222222",
            "333333333333333",
            "2:private!",
        ] {
            assert!(!rendered.contains(secret));
        }
        let explicit = fanout_inspection(&plan, true);
        assert_eq!(explicit["participant_hash"], "2:private!");
        assert!(!explicit.to_string().contains("111111111111111"));
    }

    #[test]
    fn capture_type_preserves_apk_group_label_without_disclosing_numeric_values() {
        assert_eq!(capture_protocol_type(Some("w:gp2")), "w:gp2");
        for value in ["privacy_token", "devices", "encrypt", "mex", "psa"] {
            assert_eq!(capture_protocol_type(Some(value)), value);
        }
        for value in [
            "15555550100",
            "12345@lid",
            "w:gp3",
            "private123",
            "bad\nvalue",
            "é",
        ] {
            assert_eq!(capture_protocol_type(Some(value)), "none");
        }
        assert_eq!(capture_protocol_type(None), "none");
        assert_eq!(capture_protocol_type(Some(&"a".repeat(33))), "none");
    }

    #[test]
    fn poll_inspection_never_prints_question_options_or_secret() {
        let poll = whatsapp_api::transport::polls::Poll {
            name: "private question".into(),
            options: vec!["private option".into(), "🔒".into()],
            selectable_count: 1,
            secret: Some([0xab; 32]),
        };
        let summary = poll_inspection_summary(&poll);
        for value in ["private question", "private option", "🔒", "abab", "171"] {
            assert!(!summary.contains(value));
        }
        assert!(summary.contains("name_len=16"));
        assert!(summary.contains("options_count=2"));
        assert!(summary.contains("option_lengths=[14, 4]"));
        assert!(summary.contains("secret_present=true"));
        assert!(
            poll_inspection_summary(&whatsapp_api::transport::polls::Poll {
                secret: None,
                ..poll
            })
            .contains("secret_present=false")
        );
    }

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
                    style: Default::default(),
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
