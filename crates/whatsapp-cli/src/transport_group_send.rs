//! One durable experimental sender-key group send. No replay after uncertain
//! writes, and nothing is transmitted until every key it produced is committed.
//!
//! The order matters and is the same one the direct path uses: resolve the
//! membership, resolve devices, obtain the sessions, encrypt, **commit**, then
//! write. A sender key that is distributed but not persisted cannot decrypt our
//! own later messages, and one that is persisted but never sent leaves every
//! recipient holding a chain we will not use.
use whatsapp_api::{
    Error,
    binary::{Encoder, TokenDictionary},
    model::{jid::Jid, time::Timestamp},
    session::AccountRecord,
    signal::direct::prekey_bundle_for,
    store::{ProtocolStore, Table},
    transport::{
        devices::{self, SyncContext},
        group_send::{self, FanoutPlan},
        groups::{self, StanzaContent},
        handshake::EncryptedConnection,
        inbox,
        messaging::FrameRead,
        messaging::{key_request_many, new_message_id, write_and_wait_captured, write_node},
    },
};

type Channel = EncryptedConnection<tokio::net::TcpStream>;

fn invalid(reason: impl Into<String>) -> Error {
    Error::invalid("group_send", reason)
}

/// Keep every frame that arrived while we waited for a reply to something else.
///
/// The capture-aware waiter already saves nodes as they arrive. This keeps
/// the existing successful-path projection too; one may be a real incoming
/// message or a retry receipt. They go to the same content-hashed
/// capture keys the receive loop uses, so `lab-inbox` opens them later exactly
/// as it would a frame read by `--receive`.
fn reserve_distribution(
    tx: &mut whatsapp_api::store::WriteTx<'_>,
    parent_key: &[u8],
    parent: &serde_json::Value,
    child_key: &[u8],
    child: &serde_json::Value,
) -> Result<(), Error> {
    super::transport_send::check_reserved_intent(tx, parent_key, parent)?;
    if tx.get(Table::Checkpoints, child_key, 1)?.is_some() {
        return Err(invalid("distribution already reserved; no replay"));
    }
    tx.put_json(Table::Checkpoints, child_key, 1, child)
}

fn settle_distribution(
    tx: &mut whatsapp_api::store::WriteTx<'_>,
    parent_key: &[u8],
    parent: &serde_json::Value,
    child_key: &[u8],
    child: &serde_json::Value,
    result: &serde_json::Value,
) -> Result<(), Error> {
    super::transport_send::check_reserved_intent(tx, parent_key, parent)?;
    super::transport_send::settle_reserved_intent(tx, child_key, child, result)
}

fn settle_group_intent(
    tx: &mut whatsapp_api::store::WriteTx<'_>,
    key: &[u8],
    expected: &serde_json::Value,
    result: &serde_json::Value,
    wire_id: &str,
    accepted: bool,
) -> Result<(), Error> {
    super::transport_send::settle_reserved_intent(tx, key, expected, result)?;
    if accepted {
        // Matching durable send ownership must precede sent_sender_key promotion.
        whatsapp_api::transport::sender_key_distribution::confirm_ack_in(tx, wire_id)?;
    }
    Ok(())
}

fn keep(store: &ProtocolStore, frames: &[whatsapp_api::binary::Node]) -> Result<(), Error> {
    for frame in frames {
        inbox::preserve(store, frame)?;
    }
    if !frames.is_empty() {
        println!(
            "group-send: kept {} unrelated frame(s) that arrived meanwhile",
            frames.len()
        );
    }
    Ok(())
}

fn session_key(device: &Jid) -> String {
    // Same namespace as the direct path: a session with a device is one
    // session, whether it was established for a 1:1 chat or a fan-out.
    format!("comparison-direct:{device}")
}

/// Fetch the group's membership on this connection and save it as the snapshot
/// the rest of the client reads.
async fn fetch_membership(
    store: &ProtocolStore,
    channel: &mut Channel,
    chat: &Jid,
) -> Result<(), Error> {
    let id = new_message_id()?;
    let (reply, pending) =
        write_and_wait_captured(store, channel, &groups::info_request(chat, &id)?, "iq", &id)
            .await?;
    keep(store, &pending)?;
    if reply.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
        return Err(invalid("group info query refused; membership unknown"));
    }
    if !groups::record_info_reply(store, &reply)? {
        return Err(invalid("group info reply carried no group element"));
    }
    Ok(())
}

/// Ask `usync` for every recipient's device list and ingest the reply.
async fn fetch_devices(
    store: &ProtocolStore,
    channel: &mut Channel,
    own: &Jid,
    recipients: &[Jid],
) -> Result<(), Error> {
    let recipients: Vec<Jid> = recipients
        .iter()
        .map(Jid::to_non_ad)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for recipient in &recipients {
        devices::request_sync(store, Some(own), recipient, None, "group-send")?;
    }
    // MAX_USERS_PER_QUERY is our local bound. Iterate a fixed target snapshot
    // once; looping on all pending users would resend errors and drain other
    // contacts. Each reply must be complete before advancing to the next batch.
    for (index, batch) in recipients.chunks(devices::MAX_USERS_PER_QUERY).enumerate() {
        let (query, node) = devices::begin_query_for_users(store, SyncContext::Message, batch)?
            .ok_or_else(|| invalid("selected device batch unexpectedly empty"))?;
        let id = node
            .get_attr("id")
            .and_then(|v| v.as_text())
            .ok_or_else(|| invalid("device query lost its id"))?
            .to_owned();
        println!(
            "group-send: device batch={} users={} query_id={}",
            index + 1,
            batch.len(),
            id
        );
        let (reply, pending) = write_and_wait_captured(store, channel, &node, "iq", &id).await?;
        keep(store, &pending)?;
        if !query.matches(&reply) {
            return Err(invalid("device query reply did not match the reservation"));
        }
        let summary = query.ingest(store, &reply)?;
        println!(
            "group-send: device lists ingested={}, partial_errors={}, omitted_users={}, superseded_users={}, verification_errors={}, accepted={}",
            summary.ingested,
            summary.partial_errors,
            summary.omitted_users,
            summary.superseded_users,
            summary.verification_errors,
            summary.accepted
        );
        summary.require_complete()?;
    }
    Ok(())
}

/// Send one text message to a group through the sender-key fan-out.
pub(super) async fn send(
    store: &ProtocolStore,
    account: &AccountRecord,
    channel: &mut Channel,
    message: &whatsapp_api::transport::messaging::MessageRequest,
    chat: &Jid,
    media_payload: Option<&[u8]>,
) -> Result<(), Error> {
    let intent_key = format!("message:{}:send", message.request_id);
    store.read(|tx| {
        let previous: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, intent_key.as_bytes(), 1)?;
        if previous.is_some() {
            return Err(invalid("send intent already exists; no replay"));
        }
        Ok(())
    })?;
    let own = group_send::own_lid(store)?.ok_or_else(|| {
        invalid("no own LID recorded; log in once more so `<success>` can provide it")
    })?;

    // A status has no membership to fetch: its audience is named in the JSON.
    let status = message.status.clone();
    if let Some(status) = &status {
        let audience = status.recipients()?;
        fetch_devices(store, channel, &own, &audience).await?;
        let plan = group_send::plan_status(store, &own, &audience)?;
        println!(
            "status-post: recipients={}, devices={}, without_device_list={}",
            plan.recipients.len(),
            plan.devices.len(),
            plan.without_device_list.len()
        );
        if !plan.without_device_list.is_empty() {
            return Err(invalid(
                "some of the audience have no device list; they would silently not see this status",
            ));
        }
        // An uploaded image is already a complete `Message.imageMessage`, so a
        // picture status is the text payload swapped for that one.
        let (payload, content) = match media_payload {
            Some(bytes) => (bytes.to_vec(), StanzaContent::Media("image")),
            None => (
                group_send::status_text_payload(&status.text, &status.style()?)?,
                StanzaContent::Text,
            ),
        };
        return transmit(
            store,
            account,
            channel,
            message,
            chat,
            &plan,
            &payload,
            Stanza {
                content,
                // Observed on every incoming status: no `addressing_mode`, and
                // the server has never asked us for a status `phash`.
                addressing_mode: None,
                phash: None,
            },
        )
        .await;
    }

    fetch_membership(store, channel, chat).await?;
    // Validate routing before any device-query reservation. A missing list is
    // expected on first send; a routing refusal cannot be a fallback target set.
    let members = group_send::recipients_for_group(store, chat, &own)?;
    fetch_devices(store, channel, &own, &members).await?;
    let plan = group_send::plan(store, chat, &own)?;
    println!(
        "group-send: chat addressing_mode={}, phash={}, hash_devices={}, own_devices_unknown={}, recipients={}, devices={}, without_device_list={}",
        plan.addressing_mode,
        plan.participant_hash,
        plan.participant_hash_device_count,
        plan.own_devices_unknown,
        plan.recipients.len(),
        plan.devices.len(),
        plan.without_device_list.len()
    );
    if !plan.without_device_list.is_empty() {
        return Err(invalid(
            "some members have no device list; they would silently not receive this message",
        ));
    }

    // A reply or a reaction is the same fan-out carrying a different payload.
    let (payload, content) = match &message.quote {
        Some(quote) if quote.is_reaction() => (
            whatsapp_api::transport::reactions::group_payload(
                chat,
                &quote.sender()?,
                &quote.message_id,
                false,
                quote.emoji.as_deref().filter(|e| !e.is_empty()),
                Timestamp::now().0,
            )?,
            StanzaContent::Reaction,
        ),
        Some(quote) => (
            whatsapp_api::transport::quotes::reply_payload(
                &message.text,
                &quote.message_id,
                &quote.sender()?.to_string(),
                whatsapp_api::transport::quotes::conversation(
                    quote.quoted_text.as_deref().map_or("", |t| t.as_str()),
                ),
            ),
            StanzaContent::Text,
        ),
        None => (
            group_send::text_payload(&message.text)?,
            StanzaContent::Text,
        ),
    };
    transmit(
        store,
        account,
        channel,
        message,
        chat,
        &plan,
        &payload,
        Stanza {
            content,
            addressing_mode: Some(plan.addressing_mode),
            phash: Some(&plan.participant_hash),
        },
    )
    .await
}

/// The stanza-level choices one send makes. A retry resend reuses them as they
/// are: the recipient matches the resend to the message it is holding, so it
/// must not differ in type or attributes from the original.
#[derive(Clone, Copy)]
struct Stanza<'a> {
    content: StanzaContent<'a>,
    addressing_mode: Option<&'a str>,
    phash: Option<&'a str>,
}

/// Encrypt one payload with our sender key, address it to every planned device
/// and send it. Shared by a group message and a status post, which differ only
/// in how their audience was chosen and in which stanza attributes apply.
#[allow(clippy::too_many_arguments)]
async fn transmit(
    store: &ProtocolStore,
    account: &AccountRecord,
    channel: &mut Channel,
    message: &whatsapp_api::transport::messaging::MessageRequest,
    chat: &Jid,
    plan: &FanoutPlan,
    payload: &[u8],
    stanza: Stanza<'_>,
) -> Result<(), Error> {
    let intent_key = format!("message:{}:send", message.request_id);
    let snapshot =
        whatsapp_api::transport::group_fanout::Snapshot::load(store, plan, message.fresh_sessions)?;
    if message.fresh_sessions {
        println!(
            "send: establishing fresh sessions for all {} target device(s); pinned identities are kept and still enforced",
            plan.devices.len()
        );
    }
    let needed = snapshot.required_keys();
    let request_id = whatsapp_api::model::ids::ClientRequestId::new(message.request_id.clone())?;
    while let Some(query) =
        whatsapp_api::transport::device_key_batches::reserve_next(store, &request_id, &needed)?
    {
        let id = query
            .get_attr("id")
            .and_then(|v| v.as_text())
            .ok_or_else(|| invalid("reserved key query has no ID"))?;
        let (reply, pending) = write_and_wait_captured(store, channel, &query, "iq", id).await?;
        keep(store, &pending)?;
        if !whatsapp_api::transport::device_key_batches::settle(store, &request_id, &reply)? {
            return Err(invalid("key reply did not settle its saved batch"));
        }
        let progress = whatsapp_api::transport::device_key_batches::summary(store, &request_id)?
            .ok_or_else(|| invalid("missing saved key-query progress"))?;
        println!(
            "send: key queries answered={} refused={} invalid={} batches={}",
            progress.answered, progress.refused, progress.invalid_replies, progress.batches
        );
    }
    let keys = whatsapp_api::transport::device_key_batches::assembled(store, &request_id, &needed)?;

    let prepared = snapshot.prepare(account, keys.as_ref(), payload).await?;
    let fanout = prepared.targets();

    // Out-of-band distribution: deliver each device its sender key as an
    // ordinary direct message first, on the 1:1 path that is live-verified, and
    // leave `<participants>` off the group stanza entirely.
    // A group reaction with `<participants>` is acknowledged but never
    // delivered (observed 2026-11-01, groups::sender_key_stanza), so its
    // sender-key distributions always go out of band first.
    let out_of_band = message.skdm_out_of_band()? || stanza.content == StanzaContent::Reaction;
    if out_of_band {
        println!(
            "send: distributing the sender key out of band to {} device(s) before the group stanza",
            fanout.len()
        );
    }

    let wire_id = new_message_id()?;
    let node = groups::sender_key_stanza(
        chat,
        &wire_id,
        stanza.content,
        stanza.addressing_mode,
        stanza.phash,
        if out_of_band { &[] } else { fanout },
        prepared.ciphertext(),
    )?;
    let dict = TokenDictionary::comparison_2_26_37_71();
    let wire = Encoder::new(dict).encode(&node)?;
    let intent = serde_json::json!({
        "status": "outcome_unknown",
        "wire_id": wire_id,
        "chat": chat.to_string(),
        "devices": fanout.len(),
        // Preserve this send's actual frozen device set even when distributions
        // went out of band; mutable membership cannot attribute later receipts.
        "recipient_devices": fanout.iter().map(|target| &target.device).collect::<Vec<_>>(),
        "addressing_mode": plan.addressing_mode,
        "phash": plan.participant_hash,
        "wire": wire,
        "at": Timestamp::now(),
    });
    store.write(|tx| {
        let previous: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, intent_key.as_bytes(), 1)?;
        if previous.is_some() {
            return Err(invalid("send intent already reserved"));
        }
        prepared.reserve_in(
            tx,
            format!("message:{}:rotated-sender-key", message.request_id).as_bytes(),
        )?;
        prepared.await_ack_in(tx, &wire_id)?;
        tx.put_json(Table::Checkpoints, intent_key.as_bytes(), 1, &intent)
    })?;
    eprintln!(
        "send: sessions, own sender key and send intent committed; sending one fan-out to {} device(s)",
        fanout.len()
    );

    if out_of_band {
        for target in fanout {
            let id = new_message_id()?;
            let distribution = whatsapp_api::transport::messaging::direct_message(
                &target.device,
                &id,
                target.kind,
                target.ciphertext.clone(),
            )?;
            let child_key = format!("message:{}:skdm:{}", message.request_id, target.device);
            let child = serde_json::json!({
                "status":"outcome_unknown", "wire_id":id,
                "wire":Encoder::new(dict).encode(&distribution)?, "at":Timestamp::now(),
            });
            store.write(|tx| {
                reserve_distribution(
                    tx,
                    intent_key.as_bytes(),
                    &intent,
                    child_key.as_bytes(),
                    &child,
                )
            })?;
            let (ack, pending) =
                write_and_wait_captured(store, channel, &distribution, "ack", &id).await?;
            keep(store, &pending)?;
            inbox::preserve(store, &ack)?;
            let outcome =
                whatsapp_api::transport::messaging::message_ack_outcome_for(&ack, &target.device);
            let accepted =
                outcome == whatsapp_api::transport::messaging::MessageAckOutcome::Accepted;
            let mut child_result = child.clone();
            child_result["status"] = serde_json::json!(outcome.status());
            store.write(|tx| {
                settle_distribution(
                    tx,
                    intent_key.as_bytes(),
                    &intent,
                    child_key.as_bytes(),
                    &child,
                    &child_result,
                )
            })?;
            if matches!(
                outcome,
                whatsapp_api::transport::messaging::MessageAckOutcome::AuthChallenge
                    | whatsapp_api::transport::messaging::MessageAckOutcome::Unrelated
            ) {
                return Err(invalid(
                    "distribution ACK unresolved or auth-challenged; group stanza not sent; reservation remains uncertain; no automatic retry",
                ));
            }
            if !accepted {
                return Err(invalid(
                    "a sender-key distribution message was refused; the group stanza was not sent",
                ));
            }
        }
        println!("send: all distribution messages acknowledged");
    }

    let (ack, pending) = write_and_wait_captured(store, channel, &node, "ack", &wire_id).await?;
    let outcome = whatsapp_api::transport::messaging::message_ack_outcome_for(&ack, chat);
    let accepted = outcome == whatsapp_api::transport::messaging::MessageAckOutcome::Accepted;
    let code = ack
        .get_attr("error")
        .and_then(|v| v.as_text())
        .filter(|v| v.len() <= 5 && v.bytes().all(|b| b.is_ascii_digit()));
    let mut result = intent.clone();
    result["status"] = serde_json::json!(outcome.status());
    result["error"] = serde_json::json!(code);
    store.write(|tx| {
        settle_group_intent(
            tx,
            intent_key.as_bytes(),
            &intent,
            &result,
            &wire_id,
            accepted,
        )?;
        for (index, node) in pending.iter().chain(std::iter::once(&ack)).enumerate() {
            let bytes = Encoder::new(dict).encode(node)?;
            tx.put_json(
                Table::Checkpoints,
                format!("message:{}:send-response:{index}", message.request_id).as_bytes(),
                1,
                &bytes,
            )?;
        }
        Ok(())
    })?;
    if outcome == whatsapp_api::transport::messaging::MessageAckOutcome::AuthChallenge {
        return Err(Error::invalid(
            "message",
            "server auth challenge459; reservation remains uncertain; no automatic retry",
        ));
    }
    if outcome == whatsapp_api::transport::messaging::MessageAckOutcome::Unrelated {
        return Err(Error::invalid(
            "message",
            "unrelated ACK family; reservation remains uncertain; no automatic retry",
        ));
    }
    if accepted {
        println!("send: fan-out server_acknowledged; handset delivery/read not yet verified");
        answer_retries(
            store,
            account,
            channel,
            message,
            chat,
            plan,
            stanza,
            &wire_id,
            prepared.distribution_message(),
            prepared.ciphertext(),
            pending,
        )
        .await;
        Ok(())
    } else {
        Err(invalid(format!(
            "fan-out acknowledgement refused; code={}",
            code.unwrap_or("unknown")
        )))
    }
}

/// Hold the connection after an accepted fan-out and answer any retry receipt.
///
/// A device that cannot decrypt sends `receipt type="retry"`, and the sender is
/// expected to send that device the message again. Closing on the `ack` — which
/// is what every earlier attempt did — drops those receipts on the floor and
/// leaves the recipient holding a message forever.
///
/// The resend is deliberately **not** a new message. It reuses the original
/// message id, so the recipient matches it to what it is already holding, and it
/// carries the *same* `skmsg` bytes and the *same* distribution message. That
/// last part matters: the distribution advertises our sender-key chain at the
/// iteration the `skmsg` was encrypted at, so re-publishing from the advanced
/// record would hand the recipient a chain one step past the message it is
/// trying to read. What does change is the pairwise session: a retry is evidence
/// the old one did not work for them, so a fresh prekey bundle is fetched and
/// the distribution re-encrypted under a new session. This is
/// `forceSenderKeyDistribution` with `targetDeviceRawJids` set to the asking
/// device, which is why the APK insists the two go together.
#[allow(clippy::too_many_arguments)]
async fn answer_retries(
    store: &ProtocolStore,
    account: &AccountRecord,
    channel: &mut Channel,
    message: &whatsapp_api::transport::messaging::MessageRequest,
    chat: &Jid,
    plan: &FanoutPlan,
    stanza: Stanza<'_>,
    wire_id: &str,
    distribution: &[u8],
    skmsg: &[u8],
    early: Vec<whatsapp_api::binary::Node>,
) {
    use whatsapp_api::transport::receipts::parse_retry;
    const WINDOW: std::time::Duration = std::time::Duration::from_secs(45);
    // One answer per device per send. A retry loop is the peer's to drive, and
    // answering the same device repeatedly in one window would be us, not them.
    let mut answered: Vec<Jid> = Vec::new();
    // Frames read while waiting for something else — before the ack, or during
    // a key fetch for another device's retry — are handled before the next
    // read, so a second device asking at the same moment is not lost.
    let mut backlog: std::collections::VecDeque<_> = early.into();
    let deadline = tokio::time::Instant::now() + WINDOW;
    eprintln!(
        "group-send: holding the connection for {}s to answer retry receipts",
        WINDOW.as_secs()
    );
    for index in 0..64 {
        let node = match backlog.pop_front() {
            Some(node) => node,
            None => {
                let next = tokio::time::timeout_at(
                    deadline,
                    whatsapp_api::transport::messaging::read_frame(channel),
                )
                .await;
                match next {
                    Err(_) => break,
                    // A silent window runs the per-read I/O timeout out at the same
                    // moment as the window itself, and the inner deadline wins the race.
                    // That is the window completing, not the connection failing.
                    Ok(Err(whatsapp_api::transport::TransportError::Deadline(_)))
                        if tokio::time::Instant::now() + std::time::Duration::from_secs(2)
                            >= deadline =>
                    {
                        break;
                    }
                    Ok(Err(error)) => {
                        eprintln!("group-send: post-send window ended: {error}");
                        break;
                    }
                    Ok(Ok(FrameRead::Undecodable { frame, class })) => {
                        match inbox::preserve_undecodable(store, &frame, class, None) {
                            Ok(key) => {
                                if whatsapp_api::transport::messaging::terminal_frame_error(class)
                                    .is_some()
                                {
                                    eprintln!(
                                        "group-send: terminal frame ({class}); captured as {key}; window ended"
                                    );
                                    break;
                                }
                                eprintln!(
                                    "group-send: a frame did not decode ({class}); captured as {key}"
                                );
                                continue;
                            }
                            Err(error) => {
                                eprintln!(
                                    "group-send: could not preserve an undecodable frame ({error}); window ended"
                                );
                                break;
                            }
                        }
                    }
                    Ok(Ok(FrameRead::Node(node))) => node,
                }
            }
        };
        let dict = TokenDictionary::comparison_2_26_37_71();
        let saved = Encoder::new(dict)
            .encode(&node)
            .map_err(Error::from)
            .and_then(|bytes| {
                store.write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        format!("inbound:after-send-{}:{index}", message.request_id).as_bytes(),
                        1,
                        &bytes,
                    )
                })
            });
        if let Err(error) = saved {
            // A frame we cannot keep must not be acted on either.
            eprintln!("group-send: could not preserve a post-send frame ({error}); window ended");
            break;
        }
        // Refused receipt content is the frame's own: preserved, neither
        // correlated nor answered as a retry, and the window goes on. A
        // failure recording valid content concerns stored rows and ends it.
        let correlated = match whatsapp_api::transport::receipts::validate(&node) {
            Ok(()) => whatsapp_api::transport::receipts::record(store, &node),
            Err(Error::InvalidInput { field, reason }) => {
                eprintln!(
                    "group-send: receipt content refused ({field}: {reason}); frame preserved, not correlated or retried"
                );
                if let Some(nack) =
                    whatsapp_api::transport::messaging::received_parse_error_confirmation(&node)
                    && let Err(error) = write_node(channel, &nack).await
                {
                    eprintln!(
                        "group-send: error ACK write failed ({error}); window ended without replay"
                    );
                    break;
                }
                continue;
            }
            Err(error) => Err(error),
        };
        match correlated {
            Ok(0) => {}
            Ok(receipts) => println!("group-send: recorded {receipts} device receipt update(s)"),
            Err(error) => {
                eprintln!(
                    "group-send: receipt correlation failed ({error}); frame preserved, window ended"
                );
                break;
            }
        }
        let Some(retry) = parse_retry(&node) else {
            continue;
        };
        if let Some(reason) = retry_refusal(&node, &retry, chat, wire_id, &plan.devices, &answered)
        {
            println!("group-send: retry preserved, not answered ({reason})");
            continue;
        }
        println!(
            "group-send: answering retry count={} for one device with a fresh session and a forced distribution",
            retry.count
        );
        // Marked answered before trying: a failed answer is not retried inside
        // this window either, for the same reason a successful one is not.
        answered.push(retry.device.clone());
        // The original send is already acknowledged, so one device's failed
        // answer is that device's problem, not the send's: it is reported and
        // the window stays open for the others.
        match answer_one(
            store,
            account,
            channel,
            message,
            chat,
            stanza,
            wire_id,
            distribution,
            skmsg,
            &retry,
            &mut backlog,
        )
        .await
        {
            Ok(()) => {
                println!("group-send: retry stanza written; recipient delivery is unverified")
            }
            Err(error) => eprintln!("group-send: retry answer failed for one device: {error}"),
        }
    }
    println!(
        "group-send: post-send window complete; {} retry answer(s) attempted",
        answered.len()
    );
}

// Local supported-path admission before reserving a device or sending a key
// query. APK DNP.run excludes non-original recipients; count>4 is rejected by
// the native handler/job. Counts3/4 and receipt keys need unported base-key/key
// verification policy, as in the normal driver (docs/protocol/group-retry-admission.md).
fn retry_refusal(
    node: &whatsapp_api::binary::Node,
    retry: &whatsapp_api::transport::receipts::RetryRequest,
    chat: &Jid,
    wire_id: &str,
    devices: &[Jid],
    answered: &[Jid],
) -> Option<&'static str> {
    if retry.message_id != wire_id {
        Some("different_message")
    } else if chat.is_group() && &retry.chat != chat {
        Some("different_chat")
    } else if retry.count == 0 || retry.count > whatsapp_api::transport::retry_answer::MAX_COUNT {
        Some("count_out_of_range")
    } else if retry.count > 2 {
        Some("base_key_policy_unsupported")
    } else if node.child("keys").is_some() {
        Some("receipt_keys_unsupported")
    } else if answered.contains(&retry.device) {
        Some("already_answered")
    } else if !devices.contains(&retry.device) {
        Some("not_an_original_recipient")
    } else {
        None
    }
}

/// Answer one device's retry receipt: fresh bundle, fresh session, the same
/// distribution and the same `skmsg` under the same id and stanza attributes.
#[allow(clippy::too_many_arguments)]
async fn answer_one(
    store: &ProtocolStore,
    account: &AccountRecord,
    channel: &mut Channel,
    message: &whatsapp_api::transport::messaging::MessageRequest,
    chat: &Jid,
    stanza: Stanza<'_>,
    wire_id: &str,
    distribution: &[u8],
    skmsg: &[u8],
    retry: &whatsapp_api::transport::receipts::RetryRequest,
    backlog: &mut std::collections::VecDeque<whatsapp_api::binary::Node>,
) -> Result<(), Error> {
    let id = new_message_id()?;
    let (reply, pending) = write_and_wait_captured(
        store,
        channel,
        &key_request_many(std::slice::from_ref(&retry.device), &id)?,
        "iq",
        &id,
    )
    .await?;
    backlog.extend(pending);
    let peer_key = session_key(&retry.device);
    let expected = retry_peer_snapshot(store, &peer_key)?;
    let identity = expected.identity.as_deref();
    // A device row carrying pqkey takes the same verified hybrid derivation as
    // the shared fan-out (docs/protocol/hybrid-fanout.md); never drop the key.
    let prepared = if whatsapp_api::signal::hybrid::selects_pq_key(&reply, &retry.device)? {
        whatsapp_api::signal::hybrid::bundle_for(&reply, &retry.device)?.prepare_initial(
            account,
            identity,
            distribution,
        )?
    } else {
        let bundle = prekey_bundle_for(&reply, &retry.device)?;
        whatsapp_api::signal::direct::prepare_payload(
            account,
            &retry.device,
            Some(&bundle),
            None,
            identity,
            distribution,
        )
        .await?
    };
    let resend = groups::sender_key_stanza(
        chat,
        wire_id,
        stanza.content,
        stanza.addressing_mode,
        stanza.phash,
        &[groups::FanoutTarget {
            device: retry.device.clone(),
            kind: prepared.kind,
            ciphertext: prepared.ciphertext,
        }],
        skmsg,
    )?;
    let resend = retry_stanza_count(resend, retry.count)?;
    let checkpoint = format!("message:{}:retry:{}", message.request_id, retry.device);
    let reservation = RetryReservation {
        checkpoint: &checkpoint,
        query: &id,
        count: retry.count,
    };
    reserve_retry_write(
        store,
        &peer_key,
        &expected,
        &prepared.session,
        &prepared.identity,
        &reservation,
    )?;
    let written = write_node(channel, &resend).await;
    if let Err(error) = settle_retry_write(store, &reservation, written.is_ok()) {
        eprintln!(
            "group-send: retry checkpoint settlement failed (socket_write_ok={}); reserved state must not be replayed",
            written.is_ok()
        );
        return Err(error);
    }
    written.map_err(Error::from)
}

// APK43.72 FTy.AK7/AKj -> F3f.A04/A08/A09 carry the same job retry
// count to both skmsg and pairwise distribution enc children. Original sends
// still use the zero-count builder (docs/protocol/group-retry-count.md).
fn retry_stanza_count(
    mut stanza: whatsapp_api::binary::Node,
    count: u32,
) -> Result<whatsapp_api::binary::Node, Error> {
    use whatsapp_api::binary::{Attribute, NodeContent};
    if !(1..=2).contains(&count) {
        return Err(invalid("only retry counts1/2 are supported"));
    }
    if stanza.description != "message" {
        return Err(invalid("retry requires a message stanza"));
    }
    let NodeContent::Children(children) = &mut stanza.content else {
        return Err(invalid("retry stanza has no encrypted children"));
    };
    let set_count = |enc: &mut whatsapp_api::binary::Node, kinds: &[&str]| -> Result<(), Error> {
        if !enc
            .get_attr("type")
            .and_then(|value| value.as_text())
            .is_some_and(|kind| kinds.contains(&kind))
        {
            return Err(invalid("unsupported retry encryption child"));
        }
        enc.attributes.retain(|attribute| attribute.name != "count");
        let position = enc
            .attributes
            .iter()
            .position(|attribute| attribute.name == "type")
            .ok_or_else(|| invalid("retry encryption type missing"))?
            + 1;
        enc.attributes.insert(
            position,
            Attribute {
                name: "count".into(),
                value: count.to_string().into(),
            },
        );
        Ok(())
    };
    let mut root_enc = 0;
    for child in children {
        if child.description == "enc" {
            set_count(child, &["skmsg"])?;
            root_enc += 1;
        } else if child.description == "participants" {
            let NodeContent::Children(targets) = &mut child.content else {
                return Err(invalid("retry participants missing targets"));
            };
            for target in targets {
                if target.description != "to" {
                    return Err(invalid("unexpected retry participant child"));
                }
                let NodeContent::Children(children) = &mut target.content else {
                    return Err(invalid("retry target missing encrypted distribution"));
                };
                let mut found = false;
                for enc in children {
                    if enc.description == "enc" {
                        set_count(enc, &["msg", "pkmsg"])?;
                        found = true;
                    }
                }
                if !found {
                    return Err(invalid("retry target missing encrypted distribution"));
                }
            }
        }
    }
    if root_enc != 1 {
        return Err(invalid("retry requires one root skmsg"));
    }
    Ok(stanza)
}

struct RetryPeerSnapshot {
    identity: Option<Vec<u8>>,
    session: Option<Vec<u8>>,
}

struct RetryReservation<'a> {
    checkpoint: &'a str,
    query: &'a str,
    count: u32,
}

fn retry_peer_snapshot(store: &ProtocolStore, peer_key: &str) -> Result<RetryPeerSnapshot, Error> {
    store.read(|tx| {
        Ok(RetryPeerSnapshot {
            identity: tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
            session: tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
        })
    })
}

// Local concurrency/commit discipline, not a new native session-reset policy.
// Never overwrite a ratchet advanced during async preparation; reserve before
// writing and settle only after the write (docs/protocol/group-retry-admission.md).
fn reserve_retry_write(
    store: &ProtocolStore,
    peer_key: &str,
    expected: &RetryPeerSnapshot,
    session: &[u8],
    identity: &[u8],
    reservation: &RetryReservation<'_>,
) -> Result<(), Error> {
    store.write(|tx| {
        if tx
            .get_json::<serde_json::Value>(
                Table::Checkpoints,
                reservation.checkpoint.as_bytes(),
                1,
            )?
            .is_some()
        {
            return Err(invalid("retry checkpoint already exists; no replay"));
        }
        let current_identity: Option<Vec<u8>> =
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        let current_session: Option<Vec<u8>> =
            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
        if current_identity != expected.identity {
            return Err(invalid("peer identity changed before the retry commit"));
        }
        if current_session != expected.session {
            return Err(invalid("peer session changed before the retry commit"));
        }
        tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &session)?;
        tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &identity)?;
        tx.put_json(
            Table::Checkpoints,
            reservation.checkpoint.as_bytes(),
            1,
            &serde_json::json!({"status":"reserved", "query":reservation.query,
                "count":reservation.count, "reserved_at":Timestamp::now()}),
        )
    })
}

fn settle_retry_write(
    store: &ProtocolStore,
    reservation: &RetryReservation<'_>,
    written: bool,
) -> Result<(), Error> {
    store.write(|tx| {
        let mut state: serde_json::Value = tx
            .get_json(Table::Checkpoints, reservation.checkpoint.as_bytes(), 1)?
            .ok_or_else(|| invalid("retry reservation vanished before write settlement"))?;
        if state["status"].as_str() != Some("reserved")
            || state["query"].as_str() != Some(reservation.query)
            || state["count"].as_u64() != Some(u64::from(reservation.count))
        {
            return Err(invalid("retry reservation changed before write settlement"));
        }
        state["status"] = (if written { "written" } else { "uncertain" }).into();
        state["settled_at"] = serde_json::json!(Timestamp::now());
        tx.put_json(
            Table::Checkpoints,
            reservation.checkpoint.as_bytes(),
            1,
            &state,
        )
    })
}

#[cfg(test)]
mod retry_admission_tests {
    use super::*;
    use whatsapp_api::{binary::Node, transport::receipts::parse_retry};

    #[test]
    fn distribution_reserves_before_write_and_requires_parent_and_child_ownership() {
        use std::sync::Arc;
        use whatsapp_api::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        for change in ["parent", "child", "valid"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("distribution.redb");
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
            let parent = serde_json::json!({"status":"outcome_unknown","wire_id":"parent"});
            let child =
                serde_json::json!({"status":"outcome_unknown","wire_id":"child","wire":[1,2,3]});
            let mut result = child.clone();
            result["status"] = serde_json::json!("server_acknowledged");
            store
                .write(|tx| tx.put_json(Table::Checkpoints, b"parent", 1, &parent))
                .unwrap();
            store
                .write(|tx| reserve_distribution(tx, b"parent", &parent, b"child", &child))
                .unwrap();
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, b"child", 1))
                    .unwrap(),
                Some(child.clone())
            );
            assert!(
                store
                    .write(|tx| reserve_distribution(tx, b"parent", &parent, b"child", &child))
                    .is_err()
            );
            store.write(|tx|{
                if change!="valid" {tx.put_json(Table::Checkpoints,if change=="parent" {b"parent"} else {b"child"},1,
                    &serde_json::json!({"status":"outcome_unknown","wire_id":"replacement"}))?;}
                Ok(())
            }).unwrap();
            let settle = |store: &ProtocolStore| {
                store.write(|tx| {
                    settle_distribution(tx, b"parent", &parent, b"child", &child, &result)
                })
            };
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            if change == "valid" {
                settle(&store).unwrap();
                assert!(settle(&store).is_err());
            } else {
                assert!(settle(&store).is_err());
                assert_eq!(store.journal_head().unwrap(), journal);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    before
                );
                drop(store);
                let store = open(false);
                assert!(settle(&store).is_err());
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    before
                );
                if change == "parent" {
                    assert!(
                        store
                            .write(|tx| reserve_distribution(
                                tx,
                                b"parent",
                                &parent,
                                b"another-child",
                                &child
                            ))
                            .is_err()
                    );
                }
            }
        }
    }

    #[test]
    fn stale_group_ack_preserves_distribution_until_exact_intent_settlement() {
        use std::sync::Arc;
        use whatsapp_api::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
            transport::sender_key_distribution as distribution,
        };
        for case in ["missing", "changed", "refused", "acknowledged", "valid"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("group-owner.redb");
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
            let chat = Jid::parse("123456@g.us").unwrap();
            let own = Jid::parse("15555550101@s.whatsapp.net").unwrap();
            let peer = Jid::parse("15555550102:3@s.whatsapp.net").unwrap();
            let expected = serde_json::json!({"status":"outcome_unknown","wire_id":"group-wire","wire":[1,2,3]});
            let mut result = expected.clone();
            result["status"] = serde_json::json!("server_acknowledged");
            store
                .write(|tx| {
                    distribution::reserve_in(tx, &chat, &own, None, std::slice::from_ref(&peer))?;
                    distribution::pending_ack_in(
                        tx,
                        &chat,
                        &own,
                        "group-wire",
                        std::slice::from_ref(&peer),
                    )?;
                    let mut current = expected.clone();
                    match case {
                        "changed" => current["wire_id"] = serde_json::json!("another-wire"),
                        "refused" => current["status"] = serde_json::json!("server_refused"),
                        "acknowledged" => {
                            current["status"] = serde_json::json!("server_acknowledged")
                        }
                        _ => {}
                    }
                    if case != "missing" {
                        tx.put_json(Table::Checkpoints, b"send", 1, &current)?;
                    }
                    Ok(())
                })
                .unwrap();
            let settle = |store: &ProtocolStore| {
                store.write(|tx| {
                    settle_group_intent(tx, b"send", &expected, &result, "group-wire", true)
                })
            };
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            if case == "valid" {
                settle(&store).unwrap();
                assert!(
                    store
                        .read(|tx| distribution::load_in(tx, &chat, &own))
                        .unwrap()
                        .unwrap()
                        .delivered(&peer)
                );
                assert!(settle(&store).is_err());
            } else {
                assert!(settle(&store).is_err());
                assert!(
                    !store
                        .read(|tx| distribution::load_in(tx, &chat, &own))
                        .unwrap()
                        .unwrap()
                        .delivered(&peer)
                );
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
                assert!(
                    !store
                        .read(|tx| distribution::load_in(tx, &chat, &own))
                        .unwrap()
                        .unwrap()
                        .delivered(&peer)
                );
            }
        }
    }

    #[test]
    fn group_retry_count_reaches_each_enc_without_changing_other_stanza_bytes() {
        use whatsapp_api::binary::{Decoder, NodeContent};
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let targets = [
            groups::FanoutTarget {
                device: Jid::parse("222222222222222:4@lid").unwrap(),
                kind: "pkmsg",
                ciphertext: vec![4, 5],
            },
            groups::FanoutTarget {
                device: Jid::parse("15555550100@s.whatsapp.net").unwrap(),
                kind: "msg",
                ciphertext: vec![6, 7],
            },
        ];
        let original = groups::sender_key_stanza(
            &chat,
            "W1",
            StanzaContent::Media("image"),
            Some("lid"),
            Some("2:abc"),
            &targets,
            &[1, 2, 3],
        )
        .unwrap();
        let dict = TokenDictionary::comparison_2_26_37_71();
        let original_wire = Encoder::new(dict).encode(&original).unwrap();
        fn strip_counts(mut node: Node) -> Node {
            if node.description == "enc" {
                node.attributes
                    .retain(|attribute| attribute.name != "count");
            }
            if let NodeContent::Children(children) = &mut node.content {
                for child in children {
                    *child = strip_counts(child.clone());
                }
            }
            node
        }
        for count in [1, 2] {
            let once = retry_stanza_count(original.clone(), 1).unwrap();
            let counted = retry_stanza_count(once, count).unwrap();
            let wire = Encoder::new(dict).encode(&counted).unwrap();
            let decoded = Decoder::new(&wire[1..], dict, &whatsapp_api::profile::Limits::default())
                .decode_node()
                .unwrap();
            assert_eq!(decoded.attributes, original.attributes);
            assert_eq!(strip_counts(decoded.clone()), original);
            assert_eq!(
                Encoder::new(dict)
                    .encode(&strip_counts(decoded.clone()))
                    .unwrap(),
                original_wire
            );
            let encs = std::iter::once(decoded.child("enc").unwrap()).chain(
                decoded
                    .child("participants")
                    .unwrap()
                    .child_nodes()
                    .iter()
                    .map(|to| to.child("enc").unwrap()),
            );
            for enc in encs {
                assert_eq!(
                    enc.get_attr("count").and_then(|value| value.as_text()),
                    Some(count.to_string().as_str())
                );
                assert_eq!(
                    enc.attributes
                        .iter()
                        .filter(|attribute| attribute.name == "count")
                        .count(),
                    1
                );
                assert_eq!(enc.attributes[2].name, "count"); // Native A09 emits after v/type.
            }
        }
        for count in [0, 3, 4, 5, u32::MAX] {
            assert!(retry_stanza_count(original.clone(), count).is_err());
        }
        assert!(retry_stanza_count(Node::new("message"), 1).is_err());
        assert!(
            retry_stanza_count(
                Node::new("message").children(vec![Node::new("enc").attr("type", "msg")]),
                1
            )
            .is_err()
        );
        assert_eq!(Encoder::new(dict).encode(&original).unwrap(), original_wire);
        assert!(original.child("enc").unwrap().get_attr("count").is_none());
    }

    fn open_retry_store(path: &std::path::Path) -> ProtocolStore {
        use whatsapp_api::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        ProtocolStore::open(OpenOptions {
            path: path.into(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap()
    }

    fn retry_snapshot(store: &ProtocolStore) -> Vec<Vec<(Vec<u8>, Vec<u8>)>> {
        store
            .read(|tx| {
                [
                    Table::Checkpoints,
                    Table::SignalSessions,
                    Table::PeerIdentities,
                ]
                .into_iter()
                .map(|table| tx.scan(table, 1, 10000))
                .collect()
            })
            .unwrap()
    }

    #[test]
    fn group_retry_commit_preserves_newer_peer_state_and_existing_checkpoint() {
        for change in [
            "unchanged",
            "identity",
            "session",
            "session_appeared",
            "session_removed",
            "prior_checkpoint",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open_retry_store(&dir.path().join("retry.redb"));
            let peer = "comparison-direct:222222222222222:4@lid";
            store
                .write(|tx| {
                    tx.put_json(Table::PeerIdentities, peer.as_bytes(), 1, &vec![1u8])?;
                    if change != "session_appeared" {
                        tx.put_json(Table::SignalSessions, peer.as_bytes(), 1, &vec![2u8])?;
                    }
                    Ok(())
                })
                .unwrap();
            let expected = retry_peer_snapshot(&store, peer).unwrap();
            store
                .write(|tx| {
                    match change {
                        "identity" => {
                            tx.put_json(Table::PeerIdentities, peer.as_bytes(), 1, &vec![3u8])?
                        }
                        "session" | "session_appeared" => {
                            tx.put_json(Table::SignalSessions, peer.as_bytes(), 1, &vec![3u8])?
                        }
                        "session_removed" => {
                            tx.delete(Table::SignalSessions, peer.as_bytes())?;
                        }
                        "prior_checkpoint" => tx.put_json(
                            Table::Checkpoints,
                            b"retry",
                            1,
                            &serde_json::json!({"status":"written"}),
                        )?,
                        _ => {}
                    }
                    Ok(())
                })
                .unwrap();
            let before = retry_snapshot(&store);
            let reservation = RetryReservation {
                checkpoint: "retry",
                query: "Q1",
                count: 2,
            };
            let result = reserve_retry_write(&store, peer, &expected, &[9], &[1], &reservation);
            if change == "unchanged" {
                result.unwrap();
                let after = retry_peer_snapshot(&store, peer).unwrap();
                assert_eq!(after.session, Some(vec![9]));
                assert_eq!(after.identity, Some(vec![1]));
                let checkpoint: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, b"retry", 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(checkpoint["status"], "reserved");
            } else {
                assert!(result.is_err(), "{change}");
                assert_eq!(retry_snapshot(&store), before, "{change}");
            }
        }
    }

    #[test]
    fn group_retry_write_settlement_is_owned_durable_and_never_rewinds_the_ratchet() {
        for written in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("retry.redb");
            let store = open_retry_store(&path);
            let peer = "comparison-direct:222222222222222:4@lid";
            let expected = retry_peer_snapshot(&store, peer).unwrap();
            let reservation = RetryReservation {
                checkpoint: "retry",
                query: "Q1",
                count: 2,
            };
            reserve_retry_write(&store, peer, &expected, &[9], &[1], &reservation).unwrap();
            drop(store); // Simulated interruption after commit, before socket outcome.
            let store = open_retry_store(&path);
            let before = retry_snapshot(&store);
            assert!(
                reserve_retry_write(&store, peer, &expected, &[8], &[1], &reservation).is_err()
            );
            for (query, count) in [("Q2", 2), ("Q1", 1)] {
                let stale = RetryReservation {
                    checkpoint: "retry",
                    query,
                    count,
                };
                assert!(settle_retry_write(&store, &stale, written).is_err());
                assert_eq!(retry_snapshot(&store), before);
            }
            settle_retry_write(&store, &reservation, written).unwrap();
            let settled = retry_snapshot(&store);
            assert!(settle_retry_write(&store, &reservation, written).is_err());
            assert_eq!(retry_snapshot(&store), settled);
            drop(store);
            let store = open_retry_store(&path);
            let checkpoint: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, b"retry", 1))
                .unwrap()
                .unwrap();
            assert_eq!(
                checkpoint["status"],
                if written { "written" } else { "uncertain" }
            );
            let peer = retry_peer_snapshot(&store, peer).unwrap();
            assert_eq!(peer.session, Some(vec![9]));
            assert_eq!(peer.identity, Some(vec![1]));
        }
    }

    #[test]
    fn group_retry_admission_correlates_chat_count_keys_and_recipient_before_action() {
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let device = Jid::parse("222222222222222:4@lid").unwrap();
        let other = Jid::parse("333333333333333:4@lid").unwrap();
        let receipt = |from: &Jid, peer: &Jid, id: &str, count: u32, keys: bool| {
            let mut children = vec![
                Node::new("retry")
                    .attr("id", id)
                    .attr("t", "1")
                    .attr("count", count.to_string()),
                Node::new("registration").bytes(vec![0, 0, 0, 42]),
            ];
            if keys {
                children.push(Node::new("keys"));
            }
            Node::new("receipt")
                .attr("type", "retry")
                .attr("id", id)
                .attr("from", from.clone())
                .attr("participant", peer.clone())
                .children(children)
        };
        let devices = vec![device.clone()];
        let other_chat = Jid::parse("120363000000000002@g.us").unwrap();
        let cases = [
            (receipt(&chat, &device, "W1", 1, false), false, None),
            (receipt(&chat, &device, "W1", 2, false), false, None),
            (
                receipt(&chat, &device, "W2", 1, false),
                false,
                Some("different_message"),
            ),
            (
                receipt(&other_chat, &device, "W1", 1, false),
                false,
                Some("different_chat"),
            ),
            (
                receipt(&device, &device, "W1", 1, false),
                false,
                Some("different_chat"),
            ),
            (
                receipt(&chat, &other, "W1", 1, false),
                false,
                Some("not_an_original_recipient"),
            ),
            (
                receipt(&chat, &device, "W1", 0, false),
                false,
                Some("count_out_of_range"),
            ),
            (
                receipt(&chat, &device, "W1", 3, false),
                false,
                Some("base_key_policy_unsupported"),
            ),
            (
                receipt(&chat, &device, "W1", 4, false),
                false,
                Some("base_key_policy_unsupported"),
            ),
            (
                receipt(&chat, &device, "W1", 5, false),
                false,
                Some("count_out_of_range"),
            ),
            (
                receipt(&chat, &device, "W1", i32::MAX as u32, false),
                false,
                Some("count_out_of_range"),
            ),
            (
                receipt(&chat, &device, "W1", 1, true),
                false,
                Some("receipt_keys_unsupported"),
            ),
            (
                receipt(&chat, &device, "W1", 2, true),
                false,
                Some("receipt_keys_unsupported"),
            ),
            (
                receipt(&chat, &device, "W1", 1, false),
                true,
                Some("already_answered"),
            ),
        ];
        for (node, already_answered, expected) in cases {
            let retry = parse_retry(&node).unwrap();
            let answered = if already_answered {
                vec![device.clone()]
            } else {
                vec![]
            };
            assert_eq!(
                retry_refusal(&node, &retry, &chat, "W1", &devices, &answered),
                expected
            );
            assert_eq!(devices, vec![device.clone()]);
            assert_eq!(answered.len(), usize::from(already_answered));
        }
    }
}
