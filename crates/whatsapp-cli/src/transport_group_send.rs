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
        messaging::{key_request_many, new_message_id, wait_for, write_node},
    },
};

type Channel = EncryptedConnection<tokio::net::TcpStream>;

fn invalid(reason: impl Into<String>) -> Error {
    Error::invalid("group_send", reason)
}

/// Keep every frame that arrived while we waited for a reply to something else.
///
/// `wait_for` returns them rather than dropping them, and one of them may be a
/// real incoming message or a retry receipt. They go to the same content-hashed
/// capture keys the receive loop uses, so `lab-inbox` opens them later exactly
/// as it would a frame read by `--receive`.
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
    write_node(channel, &groups::info_request(chat, &id)?).await?;
    let (reply, pending) = wait_for(channel, "iq", &id).await?;
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
        write_node(channel, &node).await?;
        let (reply, pending) = wait_for(channel, "iq", &id).await?;
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
        write_node(channel, &query).await?;
        let (reply, pending) = wait_for(channel, "iq", id).await?;
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
    let out_of_band = message.skdm_out_of_band()?;
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
            write_node(channel, &distribution).await?;
            let (ack, pending) = wait_for(channel, "ack", &id).await?;
            keep(store, &pending)?;
            let accepted = ack.get_attr("error").is_none();
            store.write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("message:{}:skdm:{}", message.request_id, target.device).as_bytes(),
                    1,
                    &serde_json::json!({
                        "status": if accepted { "server_acknowledged" } else { "server_refused" },
                        "wire_id": id,
                        "at": Timestamp::now(),
                    }),
                )
            })?;
            if !accepted {
                return Err(invalid(
                    "a sender-key distribution message was refused; the group stanza was not sent",
                ));
            }
        }
        println!("send: all distribution messages acknowledged");
    }

    write_node(channel, &node).await?;
    let (ack, pending) = wait_for(channel, "ack", &wire_id).await?;
    let accepted = ack.get_attr("class").and_then(|v| v.as_text()) == Some("message")
        && ack.get_attr("error").is_none();
    let code = ack
        .get_attr("error")
        .and_then(|v| v.as_text())
        .filter(|v| v.len() <= 5 && v.bytes().all(|b| b.is_ascii_digit()));
    let mut result = intent;
    result["status"] = serde_json::json!(if accepted {
        "server_acknowledged"
    } else {
        "server_refused"
    });
    result["error"] = serde_json::json!(code);
    store.write(|tx| {
        tx.put_json(Table::Checkpoints, intent_key.as_bytes(), 1, &result)?;
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
                    whatsapp_api::transport::messaging::read_node(channel),
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
                    Ok(Ok(node)) => node,
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
        // Refused receipt content is the frame's own: preserved, not
        // correlated, and the window still watches for retries. A failure
        // recording valid content concerns stored rows and ends the window.
        let correlated = match whatsapp_api::transport::receipts::validate(&node) {
            Ok(()) => whatsapp_api::transport::receipts::record(store, &node),
            Err(Error::InvalidInput { field, reason }) => {
                eprintln!(
                    "group-send: receipt content refused ({field}: {reason}); frame preserved, not correlated"
                );
                Ok(0)
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
        if retry.message_id != wire_id {
            println!("group-send: retry for another message; preserved, not answered");
            continue;
        }
        if answered.contains(&retry.device) {
            println!("group-send: this device was already answered in this window");
            continue;
        }
        if !plan.devices.contains(&retry.device) {
            println!("group-send: retry from a device this send did not address; not answered");
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
                println!("group-send: retry answered; the recipient should now be able to read it")
            }
            Err(error) => eprintln!("group-send: retry answer failed for one device: {error}"),
        }
    }
    println!(
        "group-send: post-send window complete; {} retry answer(s) attempted",
        answered.len()
    );
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
    write_node(
        channel,
        &key_request_many(std::slice::from_ref(&retry.device), &id)?,
    )
    .await?;
    let (reply, pending) = wait_for(channel, "iq", &id).await?;
    backlog.extend(pending);
    let peer_key = session_key(&retry.device);
    let identity: Option<Vec<u8>> =
        store.read(|tx| tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1))?;
    // A device row carrying pqkey takes the same verified hybrid derivation as
    // the shared fan-out (docs/protocol/hybrid-fanout.md); never drop the key.
    let prepared = if whatsapp_api::signal::hybrid::selects_pq_key(&reply, &retry.device)? {
        whatsapp_api::signal::hybrid::bundle_for(&reply, &retry.device)?.prepare_initial(
            account,
            identity.as_deref(),
            distribution,
        )?
    } else {
        let bundle = prekey_bundle_for(&reply, &retry.device)?;
        whatsapp_api::signal::direct::prepare_payload(
            account,
            &retry.device,
            Some(&bundle),
            None,
            identity.as_deref(),
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
    // The new session commits before the stanza goes out, exactly as the
    // original send does: a session we used but did not persist cannot
    // decrypt what the peer sends back under it.
    store.write(|tx| {
        let current_identity: Option<Vec<u8>> =
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        if current_identity != identity {
            return Err(invalid("peer identity changed before the retry commit"));
        }
        tx.put_json(
            Table::SignalSessions,
            peer_key.as_bytes(),
            1,
            &*prepared.session,
        )?;
        tx.put_json(
            Table::PeerIdentities,
            peer_key.as_bytes(),
            1,
            &prepared.identity,
        )?;
        tx.put_json(
            Table::Checkpoints,
            format!("message:{}:retry:{}", message.request_id, retry.device).as_bytes(),
            1,
            &serde_json::json!({
                "status": "written",
                "count": retry.count,
                "at": Timestamp::now(),
            }),
        )
    })?;
    write_node(channel, &resend).await?;
    Ok(())
}
