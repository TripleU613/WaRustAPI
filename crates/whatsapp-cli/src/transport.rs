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
        messaging::FrameRead,
        noise::StaticKeypair,
        socket::{SocketLimits, connect_comparison},
    },
};

/// Load the account and validate any lab registration copy in one snapshot.
fn load_transport_account(
    store: &ProtocolStore,
) -> Result<
    (
        AccountRecord,
        whatsapp_api::transport::connection::LoginSnapshot,
    ),
    Error,
> {
    store.read(|tx| {
        let account = AccountRecord::load(tx)?
            .ok_or_else(|| Error::invalid("transport", "missing account"))?;
        let snapshot =
            whatsapp_api::transport::connection::LoginSnapshot::capture_in(tx, &account)?;
        Ok((account, snapshot))
    })
}

fn apply_cli_login_success(
    tx: &mut whatsapp_api::store::WriteTx<'_>,
    reply: &whatsapp_api::binary::Node,
) -> Result<(), Error> {
    let pn = whatsapp_api::transport::connection::update_login_identity(tx, reply)?;
    let mut account = AccountRecord::load_in_write(tx)?
        .ok_or_else(|| Error::invalid("transport", "account disappeared"))?;
    account.last_authenticated_at = Some(Timestamp::now());
    if account.own_jid.is_none() {
        account.own_jid = pn;
    }
    account.save(tx)?;
    if reply.get_attr("lid").is_some() {
        // The shared handler has already validated this supplied field and
        // committed its canonical primary form inside this transaction.
        let identity: serde_json::Value = tx
            .get_json(
                whatsapp_api::store::Table::Checkpoints,
                b"login-identity",
                1,
            )?
            .ok_or_else(|| Error::invalid("transport", "missing validated login identity"))?;
        let lid = identity["lid"]
            .as_str()
            .and_then(|v| whatsapp_api::model::jid::Jid::parse(v).ok())
            .ok_or_else(|| Error::invalid("transport", "missing validated login LID"))?;
        whatsapp_api::transport::group_send::record_own_lid(tx, &lid)?;
    }
    Ok(())
}

fn reserve_cli_login(
    store: &ProtocolStore,
    snapshot: &whatsapp_api::transport::connection::LoginSnapshot,
    payload_bytes: usize,
) -> Result<String, Error> {
    use whatsapp_api::store::Table;
    let reservation = whatsapp_api::transport::messaging::new_message_id()?;
    store.write(|tx| {
        snapshot.validate_in_write(tx)?;
        let previous: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, b"comparison-primary-login-v1", 1)?;
        if previous.is_some_and(|v| v["status"] != "success") {
            return Err(Error::invalid("transport", "login already reserved or refused"));
        }
        tx.put_json(Table::Checkpoints, b"comparison-primary-login-v1", 1,
            &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now(), "payload_bytes":payload_bytes, "reservation":reservation}))
    })?;
    Ok(reservation)
}

fn validate_cli_login_owner(
    tx: &whatsapp_api::store::WriteTx<'_>,
    snapshot: &whatsapp_api::transport::connection::LoginSnapshot,
    reservation: &str,
) -> Result<(), Error> {
    snapshot.validate_in_write(tx)?;
    let current: Option<serde_json::Value> = tx.get_json(
        whatsapp_api::store::Table::Checkpoints,
        b"comparison-primary-login-v1",
        1,
    )?;
    if !current.is_some_and(|v| {
        v["status"] == "outcome_unknown" && v["reservation"].as_str() == Some(reservation)
    }) {
        return Err(Error::invalid(
            "transport",
            "login reply no longer owns reservation",
        ));
    }
    Ok(())
}

fn record_cli_login_reply(
    store: &ProtocolStore,
    node: &whatsapp_api::binary::Node,
    login_snapshot: &whatsapp_api::transport::connection::LoginSnapshot,
    reservation: &str,
) -> Result<(&'static str, Option<String>), Error> {
    let reason = whatsapp_api::transport::connection::login_reply_reason(node)?
        .map(|value| value.to_string());
    let status = if reason.is_some() {
        "failure"
    } else {
        "success"
    };
    // `<success>` carries this account's own LID (LX/0iS->A12 reads the
    // `lid` attribute and reports `reg-lid-chatd-lid-expected-but-null`
    // when it is missing). A LID-addressed group names us by it, so a
    // sender-key fan-out cannot be built without recording it here.
    let result = serde_json::json!({"status":status,"reason":reason,"at":Timestamp::now(),"message_sent":false});
    // Preserve the raw login result. A `<failure>` can carry an appeal
    // token, and saving only the status code throws that away at exactly
    // the moment it matters; a refusal cannot be replayed to fetch it again.
    let login_wire = whatsapp_api::binary::Encoder::new(
        whatsapp_api::binary::tokens::TokenDictionary::comparison_2_26_37_71(),
    )
    .encode(node)
    .ok();
    store.write(|tx| {
        validate_cli_login_owner(tx, login_snapshot, reservation)?;
        tx.put_json(
            whatsapp_api::store::Table::Checkpoints,
            b"comparison-primary-login-v1",
            1,
            &result,
        )?;
        if let Some(wire) = &login_wire {
            tx.put_json(
                whatsapp_api::store::Table::Checkpoints,
                format!("login-result:{}", Timestamp::now().0).as_bytes(),
                1,
                wire,
            )?;
        }
        if status == "success" {
            apply_cli_login_success(tx, node)?;
        }
        Ok(())
    })?;
    Ok((status, reason))
}

/// Pure preflight: run before uploads, refusal-history mutation or sockets.
fn saved_noise_identity(account: &AccountRecord) -> Result<StaticKeypair, Error> {
    let key = account
        .noise_static
        .as_ref()
        .ok_or_else(|| Error::invalid("transport", "missing saved Noise key"))?;
    let local = StaticKeypair::from_parts(key.0.private, key.0.public);
    local.validate_curve25519()?;
    Ok(local)
}

/// Preserve exactly the refusal inspected by the explicit recheck path.
/// A stale read must never clear a newer uncertain login reservation.
fn archive_refusal_for_recheck(
    store: &ProtocolStore,
    expected: &serde_json::Value,
) -> Result<String, Error> {
    use whatsapp_api::store::Table;
    const LOGIN_KEY: &[u8] = b"comparison-primary-login-v1";
    if expected["status"] != "failure" {
        return Err(Error::invalid(
            "transport",
            "recheck requires a saved failure",
        ));
    }
    let history = format!(
        "login-refusal-history:{}",
        expected["at"].as_i64().unwrap_or(0)
    );
    store.write(|tx| {
        let current: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, LOGIN_KEY, 1)?;
        if current.as_ref() != Some(expected) {
            return Err(Error::invalid(
                "transport",
                "login changed before refusal recheck",
            ));
        }
        let archived: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, history.as_bytes(), 1)?;
        if let Some(archived) = archived {
            if &archived != expected {
                return Err(Error::invalid(
                    "transport",
                    "refusal history key already belongs to another result",
                ));
            }
        } else {
            tx.put_json(Table::Checkpoints, history.as_bytes(), 1, expected)?;
        }
        tx.delete(Table::Checkpoints, LOGIN_KEY)?;
        Ok(())
    })?;
    Ok(history)
}

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
            whatsapp_api::transport::messaging::read_frame(channel),
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
            Ok(Ok(FrameRead::Undecodable { frame, class })) => {
                let key = whatsapp_api::transport::inbox::preserve_undecodable(
                    store, &frame, class, None,
                )?;
                if whatsapp_api::transport::messaging::terminal_frame_error(class).is_some() {
                    eprintln!(
                        "transport: terminal frame ({class}); captured as {key}; window ended"
                    );
                    break;
                }
                eprintln!("transport: a frame did not decode ({class}); captured as {key}");
                continue;
            }
            Ok(Ok(FrameRead::Node(node))) => node,
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
    if let Some(message) = &message {
        message.require_comparison_support()?;
    }
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let (account, login_snapshot) = load_transport_account(&store)?;
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
    let local = saved_noise_identity(&account)?;
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
            let history =
                archive_refusal_for_recheck(&store, previous.as_ref().expect("checked failure"))?;
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
        let reservation = reserve_cli_login(&store, &login_snapshot, payload.len())?;
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
        // Durable capture precedes ownership checks: a stale reply is evidence,
        // not authority to settle a newer login or change its account.
        whatsapp_api::transport::inbox::preserve_in_epoch(&store, &node, ConnectionEpoch(1))?;
        let supplied_lid = node.get_attr("lid").is_some();
        let (status, reason) =
            record_cli_login_reply(&store, &node, &login_snapshot, &reservation)?;
        println!(
            "transport: own LID {} by the login result",
            if status != "success" {
                "not updated"
            } else if supplied_lid {
                "recorded"
            } else {
                "not provided"
            }
        );
        println!(
            "transport: login status={status}, reason={}; message not sent",
            reason.as_deref().unwrap_or("none")
        );
        if status != "success" {
            return Err(Error::invalid(
                "transport",
                "login not accepted; saved result must be inspected before retrying",
            ));
        }
        if media_route {
            use whatsapp_api::transport::{
                media_routing::refresh::write_and_wait_captured, messaging::new_message_id,
            };
            let id = new_message_id()?;
            let request = whatsapp_api::transport::media_routing::refresh::prepare(&store, &id)?;
            let (reply, pending) =
                write_and_wait_captured(&store, &mut channel, &request, &id).await?;
            let dict = TokenDictionary::comparison_2_26_37_71();
            let settlement = store.write(|tx| {
                let settlement =
                    whatsapp_api::transport::media_routing::refresh::settle_in(tx, &id, &reply)?;
                for (index, node) in pending.iter().enumerate() {
                    let bytes = whatsapp_api::binary::Encoder::new(dict).encode(node)?;
                    tx.put_json(
                        Table::Checkpoints,
                        format!("inbound:{id}:{index:04}").as_bytes(),
                        1,
                        &bytes,
                    )?;
                }
                Ok(settlement)
            })?;
            let receipt_order =
                whatsapp_api::transport::media_routing::refresh::settlement_sequence(&store, &id)?
                    .ok_or_else(|| {
                        Error::invalid("media_route", "settled route has no receipt order")
                    })?;
            if settlement != whatsapp_api::transport::media_routing::refresh::Settlement::Applied {
                return Err(Error::invalid(
                    "media_route",
                    format!(
                        "media route reply {settlement:?}; receipt_order={receipt_order}; response saved encrypted"
                    ),
                ));
            }
            let (hosts, auth_present) =
                whatsapp_api::transport::media_routing::refresh::current_summary(&store)?
                    .ok_or_else(|| Error::invalid("media_route", "applied route missing"))?;
            println!(
                "transport: media route result; hosts={hosts}, auth_present={auth_present}, receipt_order={receipt_order}; saved encrypted; no image uploaded"
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
            let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            whatsapp_api::transport::messaging::write_node(
                &mut channel,
                &whatsapp_api::transport::messaging::active_request(&capture),
            )
            .await?;
            let mut activated = false;
            for index in 0..64 {
                let next = tokio::time::timeout_at(
                    deadline,
                    whatsapp_api::transport::messaging::read_frame(&mut channel),
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
                    Ok(Ok(FrameRead::Undecodable { frame, class })) => {
                        let key = whatsapp_api::transport::inbox::preserve_undecodable(
                            &store, &frame, class, None,
                        )?;
                        if whatsapp_api::transport::messaging::terminal_frame_error(class).is_some()
                        {
                            eprintln!(
                                "transport: terminal frame ({class}); captured as {key}; receive ended"
                            );
                            break;
                        }
                        eprintln!(
                            "transport: a frame did not decode ({class}); captured as {key}; receive continues"
                        );
                        continue;
                    }
                    Ok(Ok(FrameRead::Node(node))) => node,
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
                whatsapp_api::transport::messaging::validate_received_ack(&node)?;
                // Persist every frame before correlating, including messages arriving
                // before the IQ result. Timeout must not discard these frames.
                if tokio::time::Instant::now() < deadline
                    && whatsapp_api::transport::messaging::iq_reply_matches(&node, &capture)
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
                // Same gate as `listen` (receive audit F1): a receipt whose
                // wire content is refused stays captured; proven native parse
                // failures may get a fresh error ACK. Stored faults still end it.
                let mut receipt_error_confirmation = None;
                let receipt_accepted = match whatsapp_api::transport::receipts::validate(&node) {
                    Ok(()) => true,
                    Err(Error::InvalidInput { field, reason }) => {
                        receipt_error_confirmation =
                            whatsapp_api::transport::messaging::received_parse_error_confirmation(
                                &node,
                            );
                        eprintln!(
                            "transport: receipt content refused ({field}: {reason}); captured; error response available={}",
                            receipt_error_confirmation.is_some()
                        );
                        false
                    }
                    Err(error) => return Err(error),
                };
                // Settles a read receipt this window sent (`transport.mark_read`).
                whatsapp_api::transport::receipts::record_ack(&store, &node)?;
                if receipt_accepted {
                    let tracked = whatsapp_api::transport::receipts::record(&store, &node)?;
                    if tracked > 0 {
                        println!("transport: updated {tracked} outgoing receipt records");
                    }
                    if let Some(receipt) = whatsapp_api::transport::status_receipts::parse(&node)
                        && whatsapp_api::transport::status_receipts::record(&store, &receipt)?
                    {
                        println!("transport: recorded a status receipt from one viewer");
                    }
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
                                whatsapp_api::transport::receipts::reserve_pending_owned(&store)?
                            {
                                receipt.check_write_authority(&store)?;
                                whatsapp_api::transport::messaging::write_node(
                                    &mut channel,
                                    receipt.node(),
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
                let confirmation = if let Some(nack) = receipt_error_confirmation {
                    Some(nack)
                } else if (tag == "receipt" && receipt_accepted)
                    || matches!(
                        incoming,
                        whatsapp_api::transport::inbox::IncomingResult::Decrypted
                            | whatsapp_api::transport::inbox::IncomingResult::Duplicate
                    )
                {
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
                                whatsapp_api::transport::receipts::reserve_pending_owned(&store)?
                            {
                                receipt.check_write_authority(&store)?;
                                whatsapp_api::transport::messaging::write_node(
                                    &mut channel,
                                    receipt.node(),
                                )
                                .await?;
                            }
                            marked_read = true;
                        }
                        Err(error) => {
                            eprintln!("transport: read receipt may not be queued: {error}")
                        }
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
            let request_id =
                whatsapp_api::model::ids::ClientRequestId::new(message.request_id.clone())?;
            let (reply, pending) = super::transport_send::fetch_keys(
                &store,
                &request_id,
                &recipient,
                async |query, query_id| {
                    whatsapp_api::transport::messaging::write_and_wait_captured(
                        &store,
                        &mut channel,
                        query,
                        "iq",
                        query_id,
                    )
                    .await
                },
            )
            .await?;
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
                "transport: recipient key query type={kind}; user_present={}, identity_present={}, signed_key_present={}, one_time_key_present={}; bundle outcome retained encrypted",
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
    fn cli_login_reply_admission_preserves_owned_uncertainty_after_capture_and_reopen() {
        use whatsapp_api::store::Table;
        for reply in [
            Node::new("failure"),
            Node::new("failure").attr("reason", "private-invalid-reason"),
            Node::new("failure").attr("reason", "2147483648"),
            Node::new("iq").attr("reason", "403"),
            Node::new("stream:error").attr("reason", "403"),
            Node::new("notification").attr("reason", "403"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("cli-reply-admission.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create,
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
                })
                .unwrap()
            };
            let store = open(true);
            store.write(|tx| AccountRecord::default().save(tx)).unwrap();
            let (_, snapshot) = load_transport_account(&store).unwrap();
            let owner = reserve_cli_login(&store, &snapshot, 7).unwrap();
            whatsapp_api::transport::inbox::preserve_in_epoch(&store, &reply, ConnectionEpoch(1))
                .unwrap();
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let account = store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap();
            let head = store.journal_head().unwrap();
            let error = record_cli_login_reply(&store, &reply, &snapshot, &owner)
                .expect_err("invalid reply settled CLI login");
            assert!(!error.to_string().contains("private-invalid-reason"));
            assert!(!error.is_recoverable_by_reconnect());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                rows
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
                account
            );
            assert_eq!(store.journal_head().unwrap(), head);
            drop(store);
            let store = open(false);
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                rows
            );
            assert!(reserve_cli_login(&store, &snapshot, 7).is_err());
            assert_eq!(store.journal_head().unwrap(), head);
        }
    }

    #[test]
    fn cli_login_valid_reply_uses_shared_grammar_and_cannot_settle_twice() {
        use whatsapp_api::store::Table;
        for (reply, status, reason) in [
            (
                Node::new("failure").attr("reason", "+403"),
                "failure",
                Some("403"),
            ),
            (
                Node::new("failure").attr("reason", "000000403"),
                "failure",
                Some("403"),
            ),
            (
                Node::new("failure").attr("reason", "４０３"),
                "failure",
                Some("403"),
            ),
            (
                Node::new("failure").attr("reason", "2147483647"),
                "failure",
                Some("2147483647"),
            ),
            (
                Node::new("failure").attr("reason", "-2147483648"),
                "failure",
                Some("-2147483648"),
            ),
            (
                Node::new("success")
                    .attr("lid", "100@lid")
                    .attr("reason", "ignored-private"),
                "success",
                None,
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("cli-valid-reply.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create,
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
                })
                .unwrap()
            };
            let store = open(true);
            store.write(|tx| AccountRecord::default().save(tx)).unwrap();
            let (_, snapshot) = load_transport_account(&store).unwrap();
            let owner = reserve_cli_login(&store, &snapshot, 7).unwrap();
            whatsapp_api::transport::inbox::preserve_in_epoch(&store, &reply, ConnectionEpoch(1))
                .unwrap();
            let result = record_cli_login_reply(&store, &reply, &snapshot, &owner).unwrap();
            assert_eq!(result, (status, reason.map(str::to_owned)));
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, b"comparison-primary-login-v1", 1))
                .unwrap()
                .unwrap();
            assert_eq!(saved["status"], status);
            assert_eq!(saved["reason"], serde_json::json!(reason));
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let account = store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap();
            let head = store.journal_head().unwrap();
            drop(store);
            let store = open(false);
            assert!(record_cli_login_reply(&store, &reply, &snapshot, &owner).is_err());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                rows
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
                account
            );
            assert_eq!(store.journal_head().unwrap(), head);
        }
    }

    #[test]
    fn cli_login_identity_preserves_partial_fields_and_refuses_malformed_success_atomically() {
        use whatsapp_api::store::Table;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cli-identity.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("fixture").unwrap(),
                create,
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            })
            .unwrap()
        };
        let store = open(true);
        store.write(|tx| AccountRecord::default().save(tx)).unwrap();
        store
            .write(|tx| apply_cli_login_success(tx, &Node::new("success").attr("lid", "100:2@lid")))
            .unwrap();
        store
            .write(|tx| {
                apply_cli_login_success(
                    tx,
                    &Node::new("success").attr("pn", "15555550100@s.whatsapp.net"),
                )
            })
            .unwrap();
        let (pn, lid) = whatsapp_api::transport::inbox::own_identities(&store).unwrap();
        assert_eq!(pn.unwrap().to_string(), "15555550100@s.whatsapp.net");
        assert_eq!(lid.unwrap().to_string(), "100@lid");
        assert_eq!(
            whatsapp_api::transport::group_send::own_lid(&store)
                .unwrap()
                .unwrap()
                .to_string(),
            "100@lid"
        );
        for reply in [
            Node::new("success").attr("lid", "private-malformed"),
            Node::new("success").attr("lid", "123@s.whatsapp.net"),
            Node::new("success").attr("pn", ""),
            Node::new("failure").attr("lid", "200@lid"),
        ] {
            let account = store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap();
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            let error = store
                .write(|tx| apply_cli_login_success(tx, &reply))
                .unwrap_err()
                .to_string();
            assert!(!error.contains("private-malformed"));
            assert_eq!(store.journal_head().unwrap(), journal);
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
                account
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                rows
            );
        }
        drop(store);
        let store = open(false);
        assert_eq!(
            whatsapp_api::transport::group_send::own_lid(&store)
                .unwrap()
                .unwrap()
                .to_string(),
            "100@lid"
        );
    }

    #[test]
    fn cli_login_reservation_and_reply_refuse_stale_credentials_or_owner_after_reopen() {
        use whatsapp_api::store::Table;
        for case in ["account", "material", "owner", "settled", "valid"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("cli-owner.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create,
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
                })
                .unwrap()
            };
            let store = open(true);
            store.write(|tx| AccountRecord::default().save(tx)).unwrap();
            let (_, snapshot) = load_transport_account(&store).unwrap();
            let owner = reserve_cli_login(&store, &snapshot, 7).unwrap();
            assert!(reserve_cli_login(&store, &snapshot, 7).is_err());
            store.write(|tx| {
                match case {
                    "account" => {
                        let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                        account.last_authenticated_at = Some(Timestamp(123));
                        account.save(tx)?;
                    }
                    "material" => tx.put(Table::Account, b"registration-lab-material", 1, b"{")?,
                    "owner" | "settled" => tx.put_json(Table::Checkpoints, b"comparison-primary-login-v1", 1,
                        &serde_json::json!({"status":if case == "settled" {"failure"} else {"outcome_unknown"}, "reservation":"different"}))?,
                    _ => {}
                }
                Ok(())
            }).unwrap();
            // Preserve reply evidence before attempting the guarded state write.
            whatsapp_api::transport::inbox::preserve(&store, &Node::new("success")).unwrap();
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            let settle = |store: &ProtocolStore| {
                store.write(|tx| {
                    validate_cli_login_owner(tx, &snapshot, &owner)?;
                    tx.put_json(
                        Table::Checkpoints,
                        b"comparison-primary-login-v1",
                        1,
                        &serde_json::json!({"status":"success"}),
                    )
                })
            };
            if case == "valid" {
                settle(&store).unwrap();
                assert!(settle(&store).is_err());
            } else {
                assert!(settle(&store).is_err());
                assert_eq!(store.journal_head().unwrap(), journal);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                drop(store);
                let store = open(false);
                assert!(settle(&store).is_err());
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                if matches!(case, "account" | "material") {
                    assert!(reserve_cli_login(&store, &snapshot, 7).is_err());
                }
            }
        }
    }

    #[test]
    fn cli_account_load_refuses_present_invalid_material_without_mutation_after_reopen() {
        use whatsapp_api::store::Table;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("material-preflight.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("fixture").unwrap(),
                create,
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            })
            .unwrap()
        };
        let store = open(true);
        store.write(|tx| AccountRecord::default().save(tx)).unwrap();
        // Imported accounts need not have the optional lab registration record.
        assert!(load_transport_account(&store).is_ok());
        store
            .write(|tx| tx.put(Table::Account, b"registration-lab-material", 1, b"{"))
            .unwrap();
        let rows = store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap();
        let journal = store.journal_head().unwrap();
        assert!(load_transport_account(&store).is_err());
        assert_eq!(store.journal_head().unwrap(), journal);
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
            rows
        );
        assert!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 20))
                .unwrap()
                .is_empty()
        );
        drop(store);
        let store = open(false);
        assert!(load_transport_account(&store).is_err());
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 20)).unwrap(),
            rows
        );
    }

    #[test]
    fn cli_noise_preflight_refuses_unrelated_halves_without_repair_or_key_echo() {
        use whatsapp_api::{
            session::{KeyMaterial, NoiseStaticKey},
            transport::noise::NoiseConfig,
        };
        let noise = NoiseConfig {
            pattern: "Noise_XX_25519_AESGCM_SHA256".into(),
            prologue: vec![],
        };
        let pair = StaticKeypair::generate(&noise).unwrap();
        let other = StaticKeypair::generate(&noise).unwrap();
        let mut account = AccountRecord {
            noise_static: Some(NoiseStaticKey(KeyMaterial {
                private: *pair.private(),
                public: *pair.public(),
            })),
            ..Default::default()
        };
        assert!(saved_noise_identity(&account).is_ok());
        let mut changed = *pair.public();
        changed[10] ^= 1;
        for (private, public) in [
            (*pair.private(), changed),
            (*other.private(), *pair.public()),
            (*pair.private(), [0; 32]),
        ] {
            account.noise_static = Some(NoiseStaticKey(KeyMaterial { private, public }));
            let before = serde_json::to_vec(&account).unwrap();
            let error = saved_noise_identity(&account).unwrap_err().to_string();
            assert!(error.contains("saved Noise keypair is inconsistent"));
            assert!(!error.contains(&hex::encode(private)));
            assert!(!error.contains(&hex::encode(public)));
            assert_eq!(serde_json::to_vec(&account).unwrap(), before);
        }
        account.noise_static = None;
        assert!(saved_noise_identity(&account).is_err());
    }

    #[test]
    fn refusal_recheck_preserves_newer_login_and_conflicting_history_after_reopen() {
        use whatsapp_api::store::Table;
        let expected = serde_json::json!({"status":"failure", "at":7, "reason":"403"});
        for replacement in [
            None,
            Some(serde_json::json!({"status":"outcome_unknown", "reservation":"new-owner"})),
            Some(serde_json::json!({"status":"success", "at":8})),
            Some(serde_json::json!({"status":"failure", "at":8, "reason":"405"})),
            Some(expected.clone()),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("recheck.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create,
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
                })
                .unwrap()
            };
            let store = open(true);
            store
                .write(|tx| {
                    if let Some(value) = &replacement {
                        tx.put_json(Table::Checkpoints, b"comparison-primary-login-v1", 1, value)?;
                    }
                    // Keep stale-row cases independent of the archive collision.
                    if replacement.as_ref() == Some(&expected) {
                        tx.put_json(
                            Table::Checkpoints,
                            b"login-refusal-history:7",
                            1,
                            &serde_json::json!({"status":"failure", "at":7, "reason":"405"}),
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            assert!(archive_refusal_for_recheck(&store, &expected).is_err());
            assert_eq!(store.journal_head().unwrap(), journal);
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
            drop(store);
            let store = open(false);
            assert!(archive_refusal_for_recheck(&store, &expected).is_err());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
        }
    }

    #[test]
    fn refusal_recheck_archives_only_exact_failure_and_preserves_identical_history() {
        use whatsapp_api::store::Table;
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("recheck-valid.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            create: true,
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
        })
        .unwrap();
        let failure = serde_json::json!({"status":"failure", "at":7, "reason":"403"});
        for existing in [false, true] {
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        b"comparison-primary-login-v1",
                        1,
                        &failure,
                    )?;
                    if !existing {
                        tx.delete(Table::Checkpoints, b"login-refusal-history:7")?;
                    }
                    Ok(())
                })
                .unwrap();
            assert_eq!(
                archive_refusal_for_recheck(&store, &failure).unwrap(),
                "login-refusal-history:7"
            );
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, b"comparison-primary-login-v1", 1))
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"login-refusal-history:7",
                        1
                    ))
                    .unwrap(),
                Some(failure.clone())
            );
            assert!(archive_refusal_for_recheck(&store, &failure).is_err());
        }
        assert!(
            archive_refusal_for_recheck(&store, &serde_json::json!({"status":"outcome_unknown"}))
                .is_err()
        );
    }

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
