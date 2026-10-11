//! Bounded comparison transport experiment; never implicitly selected by send.

use std::{path::Path, sync::Arc, time::Duration};
use whatsapp_api::{
    Error,
    model::{ids::AccountId, time::Timestamp},
    session::AccountRecord,
    signal::crypto::SignalSignatureVerifier,
    store::{EnvKeySource, OpenOptions, ProtocolStore},
    transport::{
        ConnectionEpoch,
        comparison::{ComparisonEndpoint, ComparisonGateway},
        handshake::{COMPARISON_CHAIN_POLICY, EncryptedConnection, VerifiedHandshake},
        noise::StaticKeypair,
        socket::{SocketLimits, connect_comparison},
    },
};

/// Hold an acknowledged connection open briefly and preserve whatever the
/// server sends next.
///
/// A recipient that cannot decrypt a message answers with a `receipt
/// type="retry"`, which names the failing device and its count — the single most
/// informative artifact available when a send is acknowledged but unreadable.
/// Closing the moment the `ack` arrives throws that away, and a later session
/// only sees it after the server has queued it, if at all. Nothing is sent
/// during the window: it does not acknowledge, decrypt or advance anything, it
/// only preserves frames for offline inspection.
async fn after_send_window(
    store: &ProtocolStore,
    channel: &mut EncryptedConnection<tokio::net::TcpStream>,
    request_id: &str,
) -> Result<(), Error> {
    use whatsapp_api::{
        binary::{Encoder, tokens::TokenDictionary},
        store::Table,
    };
    const WINDOW: Duration = Duration::from_secs(45);
    eprintln!(
        "transport: holding the connection for {}s to preserve anything the recipients send back",
        WINDOW.as_secs()
    );
    let deadline = tokio::time::Instant::now() + WINDOW;
    let mut seen = 0usize;
    for index in 0..64 {
        let next = tokio::time::timeout_at(
            deadline,
            whatsapp_api::transport::messaging::read_node(channel),
        )
        .await;
        let node = match next {
            Err(_) => break,
            // A silent window runs the per-read I/O timeout out at the same
            // moment as the window itself, and the inner deadline wins the race.
            // That is the window completing, not the connection failing.
            Ok(Err(whatsapp_api::transport::TransportError::Deadline(_)))
                if tokio::time::Instant::now() + Duration::from_secs(2) >= deadline =>
            {
                break;
            }
            Ok(Err(error)) => {
                eprintln!("transport: post-send window ended: {error}");
                break;
            }
            Ok(Ok(node)) => node,
        };
        let bytes = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&node)?;
        store.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                format!("inbound:after-send-{request_id}:{index}").as_bytes(),
                1,
                &bytes,
            )
        })?;
        seen += 1;
        println!(
            "transport: post-send {} type={:?}; saved encrypted",
            node.description,
            node.get_attr("type").and_then(|v| v.as_text())
        );
    }
    println!("transport: post-send window complete; {seen} frame(s) preserved");
    Ok(())
}

/// What a single lab-transport session is authorized to do. Grouped because the
/// switches are a set the caller chooses together, not independent arguments.
pub(super) struct Session<'a> {
    /// Authorize the one live connection.
    pub allow_live: bool,
    /// Send one primary login after certificate verification.
    pub login: bool,
    /// Permit one further login after a saved `failure`, to tell a temporary
    /// limit from a permanent refusal.
    pub recheck_refusal: bool,
    /// Private message JSON to send.
    pub message: Option<&'a Path>,
    /// Hold a bounded receive window instead of sending.
    pub receive: bool,
    /// Fetch one media-upload route.
    pub media_route: bool,
    /// Set the display name after login.
    pub push_name: Option<&'a str>,
    /// Show one typing indicator after login.
    pub typing: Option<Typing>,
    /// People to subscribe to once a receive window is active.
    pub subscribe: Vec<whatsapp_api::model::jid::Jid>,
}

/// One typing indicator: the state to show, and how long before `paused`.
pub(super) struct Typing {
    pub chat: whatsapp_api::model::jid::Jid,
    pub state: whatsapp_api::transport::presence::ChatState,
    pub hold: Duration,
}

/// The one chat a status post is addressed to.
fn status_broadcast() -> whatsapp_api::model::jid::Jid {
    whatsapp_api::model::jid::Jid::parse("status@broadcast").expect("constant JID")
}

pub(super) async fn run(cli: &super::Cli, path: &Path, session: Session<'_>) -> Result<(), Error> {
    let Session {
        allow_live,
        login,
        recheck_refusal,
        message,
        receive,
        media_route,
        push_name,
        typing,
        subscribe,
    } = session;
    if !allow_live {
        return Err(Error::invalid("transport", "requires --allow-live-request"));
    }
    let config = super::signup::config(path)?;
    let message = message
        .map(whatsapp_api::transport::messaging::MessageRequest::read)
        .transpose()?;
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let account = store
        .read(AccountRecord::load)?
        .ok_or_else(|| Error::invalid("transport", "missing account"))?;
    if let Some(message) = &message {
        whatsapp_api::transport::send_path::claim(
            &store,
            &whatsapp_api::model::ids::ClientRequestId::new(message.request_id.clone())?,
            whatsapp_api::transport::send_path::SendPath::Lab,
        )?;
        let previous: Option<serde_json::Value> = store.read(|tx| {
            tx.get_json(
                whatsapp_api::store::Table::Checkpoints,
                format!("message:{}:send", message.request_id).as_bytes(),
                1,
            )
        })?;
        if let Some(previous) = previous {
            if previous["to"] != message.to.as_str()
                || previous["text"] != message.text.as_str()
                || previous["image_hash"]
                    != whatsapp_api::transport::image_upload::fingerprint(message)?
            {
                return Err(Error::invalid(
                    "message",
                    "request_id already belongs to a different message",
                ));
            }
            if previous["status"] == "server_acknowledged" {
                println!("transport: message already server_acknowledged; not sent again");
                return Ok(());
            }
            return Err(Error::invalid(
                "message",
                "saved send is refused or uncertain; no replay",
            ));
        }
    }
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::invalid(
            "transport",
            "requires matching registered account",
        ));
    }
    let payload = login
        .then(|| whatsapp_api::transport::login::primary_login(&account, &config))
        .transpose()?;
    const LOGIN_KEY: &[u8] = b"comparison-primary-login-v1";
    if login {
        let previous: Option<serde_json::Value> =
            store.read(|tx| tx.get_json(whatsapp_api::store::Table::Checkpoints, LOGIN_KEY, 1))?;
        if previous.as_ref().is_some_and(|v| v["status"] != "success") {
            // A saved `failure` is a definite server answer, and a refusal that
            // is really a temporary limit cannot be told from a permanent one
            // without asking again later. `--recheck-refusal` authorizes exactly
            // one more login for that question and nothing else: it requires a
            // saved `failure`, never an `outcome_unknown` (an ambiguous outcome
            // must not be resolved by guessing), it refuses to carry a message,
            // and the earlier refusal is preserved under its own key rather than
            // overwritten.
            let recheckable = recheck_refusal
                && previous.as_ref().is_some_and(|v| v["status"] == "failure")
                && message.is_none();
            if !recheckable {
                return Err(Error::invalid(
                    "transport",
                    if recheck_refusal {
                        "--recheck-refusal needs a saved `failure` and no --message"
                    } else {
                        "previous login experiment exists; inspect saved result before another attempt"
                    },
                ));
            }
            use whatsapp_api::store::Table;
            let history = format!(
                "login-refusal-history:{}",
                previous
                    .as_ref()
                    .and_then(|v| v["at"].as_i64())
                    .unwrap_or(0)
            );
            let kept = previous.clone();
            store.write(|tx| {
                let existing: Option<serde_json::Value> =
                    tx.get_json(Table::Checkpoints, history.as_bytes(), 1)?;
                if existing.is_none() {
                    tx.put_json(Table::Checkpoints, history.as_bytes(), 1, &kept)?;
                }
                tx.delete(Table::Checkpoints, LOGIN_KEY)?;
                Ok(())
            })?;
            eprintln!(
                "transport: rechecking a saved login refusal ({}); the earlier result is preserved as {history}",
                previous
                    .as_ref()
                    .and_then(|v| v["reason"].as_str())
                    .unwrap_or("no reason")
            );
        }
    }
    let image_payload = if let Some(message) = &message {
        whatsapp_api::transport::image_upload::prepare(&store, message, &config).await?
    } else {
        None
    };
    let key = account
        .noise_static
        .as_ref()
        .ok_or_else(|| Error::invalid("transport", "missing saved Noise key"))?;
    let local = StaticKeypair::from_parts(key.0.private, key.0.public);
    let endpoint = ComparisonEndpoint::built_in(ComparisonGateway::Primary, 443)
        .map_err(|_| Error::invalid("transport", "invalid endpoint"))?;
    let limits = SocketLimits {
        connect_timeout: Duration::from_millis(config.transport.connect_timeout_ms),
        io_timeout: Duration::from_millis(config.transport.timeout_ms),
        ..Default::default()
    };
    let map = |e| Error::invalid("transport", format!("{e}"));
    eprintln!(
        "transport: one raw TCP connection, classic XX, certificate verification enabled; automatic retries disabled"
    );
    let socket = connect_comparison(&endpoint, ConnectionEpoch(1), limits)
        .await
        .map_err(map)?;
    eprintln!("transport: TCP connected; sending framed ClientHello");
    let verified = VerifiedHandshake::start(
        socket,
        &local,
        ConnectionEpoch(1),
        Timestamp::now(),
        &COMPARISON_CHAIN_POLICY,
        &SignalSignatureVerifier,
    )
    .await
    .map_err(map)?;
    println!(
        "transport: server certificate verified; intermediate_serial={}, leaf_serial={}",
        verified.chain().intermediate_serial,
        verified.chain().leaf_serial
    );
    if let Some(payload) = payload {
        use whatsapp_api::{
            binary::{Decoder, tokens::TokenDictionary, unpack_frame_payload},
            profile::Limits,
            store::Table,
        };
        store.write(|tx| {
            let previous: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, LOGIN_KEY, 1)?;
            if previous.as_ref().is_some_and(|v| v["status"] != "success") { return Err(Error::invalid("transport", "login already reserved or refused")); }
            tx.put_json(Table::Checkpoints, LOGIN_KEY, 1, &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now(), "payload_bytes":payload.len()}))
        })?;
        eprintln!(
            "transport: sending one minimal primary login ({} bytes); unknown hardware fields omitted",
            payload.len()
        );
        let mut channel = verified.finish(&payload).await.map_err(map)?;
        let frame = channel.read().await.map_err(map)?.ok_or_else(|| {
            Error::invalid(
                "transport",
                "EOF awaiting login result; outcome preserved as unknown",
            )
        })?;
        let limits = Limits::default();
        let unpacked = unpack_frame_payload(&frame, &limits)
            .map_err(|_| Error::invalid("transport", "cannot unpack encrypted login response"))?;
        let node = Decoder::new(&unpacked, TokenDictionary::comparison_2_26_37_71(), &limits)
            .decode_node()
            .map_err(|_| {
                Error::invalid(
                    "transport",
                    "cannot decode login response with comparison dictionary",
                )
            })?;
        let status = match node.description.as_str() {
            "success" => "success",
            "failure" => "failure",
            "stream:error" => "stream_error",
            _ => "unrecognized",
        };
        let reason = node
            .get_attr("reason")
            .and_then(|v| v.as_text())
            .filter(|v| v.len() <= 5 && v.bytes().all(|b| b.is_ascii_digit()));
        // `<success>` carries this account's own LID (LX/0iS->A12 reads the
        // `lid` attribute and reports `reg-lid-chatd-lid-expected-but-null`
        // when it is missing). A LID-addressed group names us by it, so a
        // sender-key fan-out cannot be built without recording it here.
        let own_lid = node.get_attr("lid").and_then(|v| {
            v.as_jid()
                .cloned()
                .or_else(|| whatsapp_api::model::jid::Jid::parse(v.as_text()?).ok())
        });
        let result = serde_json::json!({"status":status,"reason":reason,"at":Timestamp::now(),"message_sent":false});
        // Preserve the raw login result. A `<failure>` can carry an appeal
        // token, and saving only the status code throws that away at exactly
        // the moment it matters; a refusal cannot be replayed to fetch it again.
        let login_wire =
            whatsapp_api::binary::Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .ok();
        store.write(|tx| {
            tx.put_json(Table::Checkpoints, LOGIN_KEY, 1, &result)?;
            if let Some(wire) = &login_wire {
                tx.put_json(
                    Table::Checkpoints,
                    format!("login-result:{}", Timestamp::now().0).as_bytes(),
                    1,
                    wire,
                )?;
            }
            if status == "success" {
                let mut account = AccountRecord::load_in_write(tx)?
                    .ok_or_else(|| Error::invalid("transport", "account disappeared"))?;
                account.last_authenticated_at = Some(Timestamp::now());
                account.save(tx)?;
                if let Some(lid) = &own_lid {
                    whatsapp_api::transport::group_send::record_own_lid(tx, lid)?;
                }
            }
            Ok(())
        })?;
        println!(
            "transport: own LID {} by the login result",
            if own_lid.is_some() {
                "recorded"
            } else {
                "not provided"
            }
        );
        println!(
            "transport: login status={status}, reason={}; message not sent",
            reason.unwrap_or("none")
        );
        if status != "success" {
            return Err(Error::invalid(
                "transport",
                "login not accepted; saved result must be inspected before retrying",
            ));
        }
        if media_route {
            use whatsapp_api::transport::messaging::{
                media_route_request, new_message_id, wait_for, write_node,
            };
            let id = new_message_id()?;
            let checkpoint = format!("media-route:{id}");
            store.write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    checkpoint.as_bytes(),
                    1,
                    &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now()}),
                )
            })?;
            write_node(&mut channel, &media_route_request(&id)).await?;
            let (reply, pending) = wait_for(&mut channel, "iq", &id).await?;
            let success = reply.get_attr("type").and_then(|v| v.as_text()) == Some("result")
                && reply.child("media_conn").is_some();
            let dict = TokenDictionary::comparison_2_26_37_71();
            let wire = whatsapp_api::binary::Encoder::new(dict).encode(&reply)?;
            store.write(|tx| {
                tx.put_json(Table::Checkpoints, checkpoint.as_bytes(), 1,
                    &serde_json::json!({"status": if success {"result"} else {"refused"}, "wire":wire, "at":Timestamp::now()}))?;
                for (index, node) in pending.iter().enumerate() {
                    let bytes = whatsapp_api::binary::Encoder::new(dict).encode(node)?;
                    tx.put_json(Table::Checkpoints, format!("inbound:{id}:{index:04}").as_bytes(), 1, &bytes)?;
                }
                Ok(())
            })?;
            if !success {
                return Err(Error::invalid(
                    "media_route",
                    "server did not return media routes; response saved encrypted",
                ));
            }
            let route = reply.child("media_conn").expect("checked");
            let hosts = route
                .child_nodes()
                .iter()
                .filter(|n| n.description == "host")
                .count();
            println!(
                "transport: media route result; hosts={hosts}, auth_present={}; saved encrypted; no image uploaded",
                route.get_attr("auth").is_some()
            );
        }
        if let Some(name) = push_name {
            use whatsapp_api::transport::{messaging::write_node, presence};
            write_node(&mut channel, &presence::push_name(name)?).await?;
            store.write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"push-name",
                    1,
                    &serde_json::json!({"status": "written", "name": name.trim(), "at": Timestamp::now()}),
                )
            })?;
            // Fire-and-forget: there is no reply, so "written" is all that can
            // honestly be claimed. Someone else's view of the name is the check.
            println!("transport: display name written; the server does not acknowledge it");
        }
        if let Some(typing) = typing {
            use whatsapp_api::transport::{
                messaging::{available_presence, write_node},
                presence::{ChatState, chatstate},
            };
            // `LX/B2G->A01` writes `composing` only while `LX/09Y->A06` holds;
            // announcing availability first is what every normal session has
            // done by this point (`LX/1Rd->A01`).
            write_node(&mut channel, &available_presence()).await?;
            write_node(&mut channel, &chatstate(&typing.chat, typing.state)?).await?;
            println!("transport: chat state {:?} written", typing.state);
            if typing.state != ChatState::Paused {
                // Hold, then stop: an indicator left running is not something
                // a real client does. This deliberately does not read while it
                // waits. Cancelling a read on this channel closes the
                // connection (`EncryptedReader::read`), so a timed read would
                // tear it down before `paused` could be written — which is what
                // the first live attempt did. Frames that arrive meanwhile stay
                // queued for whatever follows (`--message`) to read.
                tokio::time::sleep(typing.hold).await;
                write_node(&mut channel, &chatstate(&typing.chat, ChatState::Paused)?).await?;
                println!("transport: held {}s, then paused", typing.hold.as_secs());
            }
        }
        if receive {
            let capture = whatsapp_api::transport::messaging::new_message_id()?;
            let activation_key = format!("connection-active:{capture}");
            store.write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    activation_key.as_bytes(),
                    1,
                    &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now()}),
                )
            })?;
            whatsapp_api::transport::messaging::write_node(
                &mut channel,
                &whatsapp_api::transport::messaging::active_request(&capture),
            )
            .await?;
            let mut activated = false;
            let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            for index in 0..64 {
                let next = tokio::time::timeout_at(
                    deadline,
                    whatsapp_api::transport::messaging::read_node(&mut channel),
                )
                .await;
                let node = match next {
                    Err(_) => {
                        println!("transport: bounded receive window complete");
                        break;
                    }
                    Ok(Err(error)) => {
                        eprintln!("transport: receive ended: {error}");
                        break;
                    }
                    Ok(Ok(node)) => node,
                };
                let bytes =
                    whatsapp_api::binary::Encoder::new(TokenDictionary::comparison_2_26_37_71())
                        .encode(&node)?;
                store.write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        format!("inbound:{capture}:{index}").as_bytes(),
                        1,
                        &bytes,
                    )
                })?;
                // Persist every frame before correlating, including messages arriving
                // before the IQ result. Timeout must not discard these frames.
                if node.description == "iq"
                    && node.get_attr("id").and_then(|v| v.as_text()) == Some(capture.as_str())
                {
                    activated = node.get_attr("type").and_then(|v| v.as_text()) == Some("result");
                    store.write(|tx| tx.put_json(Table::Checkpoints, activation_key.as_bytes(), 1,
                        &serde_json::json!({"status":if activated {"result"} else {"refused"}, "at":Timestamp::now()})))?;
                    println!(
                        "transport: connection activation={}",
                        if activated { "accepted" } else { "refused" }
                    );
                    if !activated {
                        return Err(Error::invalid(
                            "transport",
                            "connection activation refused; response saved encrypted",
                        ));
                    }
                    if !subscribe.is_empty() {
                        use whatsapp_api::transport::{
                            messaging::{available_presence, write_node},
                            presence,
                        };
                        // Only an available client is sent other people's
                        // presence; the runtime announces it at this same point.
                        write_node(&mut channel, &available_presence()).await?;
                        let mut with_token = 0usize;
                        for peer in &subscribe {
                            let token = presence::saved_tctoken(&store, peer)?
                                .filter(|t| (1..=120).contains(&t.len()));
                            with_token += usize::from(token.is_some());
                            write_node(
                                &mut channel,
                                &presence::subscribe(peer, None, token.as_deref())?,
                            )
                            .await?;
                        }
                        println!(
                            "transport: subscribed to {} contact(s), {with_token} with a saved token",
                            subscribe.len()
                        );
                    }
                    continue;
                }
                let tag = match node.description.as_str() {
                    "message" => "message",
                    "receipt" => "receipt",
                    "notification" => "notification",
                    "iq" => "iq",
                    "ib" => "ib",
                    "stream:error" => "stream:error",
                    _ => "other",
                };
                let kind = node
                    .child("enc")
                    .and_then(|n| n.get_attr("type"))
                    .and_then(|v| v.as_text())
                    .filter(|v| matches!(*v, "msg" | "pkmsg" | "skmsg"))
                    .unwrap_or("none/other");
                let incoming = whatsapp_api::transport::inbox::process_with_policy(
                    &store,
                    &account,
                    &node,
                    format!("inbound:{capture}:{index}").as_bytes(),
                    config.transport.identity_changes,
                )
                .await?;
                if let Some(seen) = whatsapp_api::transport::presence::parse_presence(&node) {
                    use whatsapp_api::transport::presence::LastSeen;
                    println!(
                        "transport: presence {} (last seen {})",
                        match seen.available {
                            Some(true) => "available",
                            Some(false) => "unavailable",
                            None => "untyped",
                        },
                        match seen.last_seen {
                            LastSeen::Absent => "not given".to_owned(),
                            LastSeen::At(t) => format!(
                                "{}s ago",
                                u64::try_from(Timestamp::now().0 / 1000)
                                    .unwrap_or(0)
                                    .saturating_sub(t)
                            ),
                            LastSeen::Withheld(why) => why.to_owned(),
                        }
                    );
                }
                if let Some(typing) = whatsapp_api::transport::presence::parse_chatstate(&node) {
                    // Identities stay out of the log, as everywhere else here.
                    println!(
                        "transport: chat state {:?} in a {} chat",
                        typing.state,
                        if typing.chat.is_group() {
                            "group"
                        } else {
                            "direct"
                        }
                    );
                }
                if let Some(receipt) = whatsapp_api::transport::status_receipts::parse(&node)
                    && whatsapp_api::transport::status_receipts::record(&store, &receipt)?
                {
                    println!("transport: recorded a status receipt from one viewer");
                }
                // Settles a read receipt this window sent (`transport.mark_read`).
                whatsapp_api::transport::receipts::record_ack(&store, &node)?;
                let tracked = whatsapp_api::transport::receipts::record(&store, &node)?;
                if tracked > 0 {
                    println!("transport: updated {tracked} outgoing receipt records");
                }
                // Share the runtime's durable one-request policy; a repeated
                // capture or an uncertain write must never restart count=1.
                if incoming == whatsapp_api::transport::inbox::IncomingResult::Undecryptable {
                    match whatsapp_api::transport::receipts::queue_retry_for_incoming(
                        &store, &account, &node,
                    ) {
                        Ok(0) => println!("transport: retry already recorded; no repeat"),
                        Ok(_) => {
                            for receipt in
                                whatsapp_api::transport::receipts::reserve_pending(&store)?
                            {
                                whatsapp_api::transport::messaging::write_node(
                                    &mut channel,
                                    &receipt,
                                )
                                .await?;
                            }
                            println!(
                                "transport: retry queued; reserved receipt batch written; delivery remains unconfirmed"
                            );
                        }
                        Err(Error::InvalidInput { field, reason }) => {
                            eprintln!("transport: retry receipt not queued: {field}: {reason}");
                        }
                        Err(error) => return Err(error),
                    }
                }
                let confirmation = if tag == "receipt"
                    || matches!(
                        incoming,
                        whatsapp_api::transport::inbox::IncomingResult::Decrypted
                            | whatsapp_api::transport::inbox::IncomingResult::Duplicate
                    ) {
                    whatsapp_api::transport::messaging::received_confirmation_for_account(
                        &store, &account, &node,
                    )?
                } else {
                    None
                };
                let confirmed = confirmation.is_some();
                if let Some(confirmation) = confirmation {
                    let checkpoint = format!("receive-confirmation:{capture}:{index}");
                    store.write(|tx| {
                        tx.put_json(
                            Table::Checkpoints,
                            checkpoint.as_bytes(),
                            1,
                            &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now()}),
                        )
                    })?;
                    whatsapp_api::transport::messaging::write_node(&mut channel, &confirmation)
                        .await?;
                    store.write(|tx| {
                        tx.put_json(
                            Table::Checkpoints,
                            checkpoint.as_bytes(),
                            1,
                            &serde_json::json!({"status":"written", "at":Timestamp::now()}),
                        )
                    })?;
                }
                // Same opt-in as `listen`: mark readable content read once it
                // is confirmed delivered.
                let mut marked_read = false;
                if config.transport.mark_read
                    && matches!(
                        incoming,
                        whatsapp_api::transport::inbox::IncomingResult::Decrypted
                            | whatsapp_api::transport::inbox::IncomingResult::Duplicate
                    )
                {
                    match whatsapp_api::transport::receipts::queue_read_for_incoming(&store, &node)
                    {
                        Ok(0) => {}
                        Ok(_) => {
                            for receipt in
                                whatsapp_api::transport::receipts::reserve_pending(&store)?
                            {
                                whatsapp_api::transport::messaging::write_node(
                                    &mut channel,
                                    &receipt,
                                )
                                .await?;
                            }
                            marked_read = true;
                        }
                        Err(error) => eprintln!("transport: read receipt not queued: {error}"),
                    }
                }
                println!(
                    "transport: incoming {tag}, encryption={kind}; saved encrypted; delivery/receipt confirmation={confirmed}; marked_read={marked_read}; no opened receipt"
                );
                if tag == "stream:error" {
                    break;
                }
            }
            if !activated {
                return Err(Error::invalid(
                    "transport",
                    "connection activation unconfirmed; received frames preserved; no retry",
                ));
            }
        }
        if status == "success"
            && let Some(message) = &message
            && let Some(chat) = message
                .group_chat()?
                .or_else(|| message.status.is_some().then(status_broadcast))
        {
            // The group path holds its own post-send window, because answering
            // a retry needs the sender key and the skmsg that only it has.
            super::transport_group_send::send(
                &store,
                &account,
                &mut channel,
                message,
                &chat,
                image_payload.as_deref().map(|v| v.as_slice()),
            )
            .await?;
        } else if status == "success"
            && let Some(message) = &message
        {
            let recipient = message.recipient()?;
            let saved_keys: Option<Vec<u8>> = store.read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("message:{}:keys", message.request_id).as_bytes(),
                    1,
                )
            })?;
            let (reply, pending) = if let Some(bytes) = saved_keys {
                let decoded = unpack_frame_payload(&bytes, &limits)?;
                (
                    Decoder::new(&decoded, TokenDictionary::comparison_2_26_37_71(), &limits)
                        .decode_node()?,
                    Vec::new(),
                )
            } else {
                let query = whatsapp_api::transport::messaging::key_request(
                    &recipient,
                    &format!("{}-keys", message.request_id),
                );
                whatsapp_api::transport::messaging::write_node(&mut channel, &query)
                    .await
                    .map_err(map)?;
                let (reply, pending) = whatsapp_api::transport::messaging::wait_for(
                    &mut channel,
                    "iq",
                    &format!("{}-keys", message.request_id),
                )
                .await
                .map_err(map)?;
                (reply, pending)
            };
            let dict = TokenDictionary::comparison_2_26_37_71();
            store.write(|tx| {
                let bytes = whatsapp_api::binary::Encoder::new(dict)
                    .encode(&reply)
                    .map_err(|_| Error::invalid("transport", "cannot preserve key reply"))?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("message:{}:keys", message.request_id).as_bytes(),
                    1,
                    &bytes,
                )?;
                for (i, node) in pending.iter().enumerate() {
                    let bytes = whatsapp_api::binary::Encoder::new(dict)
                        .encode(node)
                        .map_err(|_| {
                            Error::invalid("transport", "cannot preserve pending stanza")
                        })?;
                    tx.put_json(
                        Table::Checkpoints,
                        format!("message:{}:pending:{i}", message.request_id).as_bytes(),
                        1,
                        &bytes,
                    )?;
                }
                Ok(())
            })?;
            let kind = reply
                .get_attr("type")
                .and_then(|v| v.as_text())
                .filter(|v| matches!(*v, "result" | "error"))
                .unwrap_or("unknown");
            let user = reply.child("list").and_then(|v| v.child("user"));
            println!(
                "transport: recipient key query type={kind}; user_present={}, identity_present={}, signed_key_present={}, one_time_key_present={}; response saved encrypted",
                user.is_some(),
                user.is_some_and(|v| v.child("identity").is_some()),
                user.is_some_and(|v| v.child("skey").is_some()),
                user.is_some_and(|v| v.child("key").is_some())
            );
            super::transport_send::send(
                &store,
                &account,
                &mut channel,
                message,
                &reply,
                image_payload.as_deref().map(|v| v.as_slice()),
            )
            .await?;
            after_send_window(&store, &mut channel, &message.request_id).await?;
        }
    } else {
        println!(
            "transport: certificate probe complete; account login not attempted, message not sent"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use whatsapp_api::{
        binary::Node,
        model::jid::Jid,
        store::{MasterKey, StaticKeySource},
    };

    #[test]
    fn cli_account_confirmation_distinguishes_own_and_remote_status() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("cli-status.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            create: true,
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
        })
        .unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550999@s.whatsapp.net").unwrap()),
            ..Default::default()
        };
        for (sender, expected_type) in [
            ("15555550999:2@s.whatsapp.net", Some("sender")),
            ("15555550100@s.whatsapp.net", None),
        ] {
            let incoming = Node::new("message")
                .attr("from", "status@broadcast")
                .attr("participant", sender)
                .attr("id", "CLI-STATUS")
                .attr("from_me", "true");
            let receipt = whatsapp_api::transport::messaging::received_confirmation_for_account(
                &store, &account, &incoming,
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                receipt.get_attr("type").and_then(|v| v.as_text()),
                expected_type
            );
            assert_eq!(
                receipt.get_attr("class").and_then(|v| v.as_text()),
                Some("status")
            );
        }
    }
}
