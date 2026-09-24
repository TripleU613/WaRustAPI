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
    signal::{
        direct::prekey_bundle_for,
        group::{encrypt_group_payload, prepare_own_sender_key, sender_key_db_key},
    },
    store::{ProtocolStore, Table},
    transport::{
        devices::{self, SyncContext},
        group_send::{self, FanoutPlan},
        groups,
        handshake::EncryptedConnection,
        messaging::{key_request_many, new_message_id, wait_for, write_node},
    },
};

type Channel = EncryptedConnection<tokio::net::TcpStream>;

fn invalid(reason: impl Into<String>) -> Error {
    Error::invalid("group_send", reason)
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
    let (reply, _) = wait_for(channel, "iq", &id).await?;
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
    for recipient in recipients {
        devices::request_sync(store, Some(own), recipient, None, "group-send")?;
    }
    let Some((query, node)) = devices::begin_query(store, SyncContext::Interactive)? else {
        return Ok(());
    };
    write_node(channel, &node).await?;
    let id = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .ok_or_else(|| invalid("device query lost its id"))?
        .to_owned();
    let (reply, _) = wait_for(channel, "iq", &id).await?;
    if !query.matches(&reply) {
        return Err(invalid("device query reply did not match the reservation"));
    }
    let summary = query.ingest(store, &reply)?;
    println!(
        "group-send: device lists ingested={}, partial_errors={}, accepted={}",
        summary.ingested, summary.partial_errors, summary.accepted
    );
    Ok(())
}

/// One recipient device with its saved session material.
struct Target {
    device: Jid,
    session: Option<zeroize::Zeroizing<Vec<u8>>>,
    identity: Option<Vec<u8>>,
}

fn load_targets(
    store: &ProtocolStore,
    plan: &FanoutPlan,
    fresh_sessions: bool,
) -> Result<Vec<Target>, Error> {
    store.read(|tx| {
        plan.devices
            .iter()
            .map(|device| {
                let key = session_key(device);
                let session = if fresh_sessions {
                    // Drop only the ratchet. The pinned peer identity below is
                    // still loaded and still enforced, so this cannot be used to
                    // accept a changed identity.
                    None
                } else {
                    tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?
                        .map(zeroize::Zeroizing::new)
                };
                Ok(Target {
                    device: device.clone(),
                    session,
                    identity: tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?,
                })
            })
            .collect()
    })
}

/// Send one text message to a group through the sender-key fan-out.
pub(super) async fn send(
    store: &ProtocolStore,
    account: &AccountRecord,
    channel: &mut Channel,
    message: &whatsapp_api::transport::messaging::MessageRequest,
    chat: &Jid,
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

    fetch_membership(store, channel, chat).await?;
    // Plan once to learn the membership, fetch the devices it names, then plan
    // again over what actually arrived.
    let members = group_send::plan(store, chat, &own)
        .map(|p| p.recipients)
        .or_else(|error| {
            // A first plan fails when no member has a device list yet; the
            // membership is still readable from the snapshot.
            groups::get(store, chat)?
                .map(|snapshot| {
                    snapshot
                        .members
                        .iter()
                        .filter_map(|m| Jid::parse(&m.jid).ok())
                        .map(|j| j.to_non_ad())
                        .filter(|j| *j != own)
                        .collect::<Vec<_>>()
                })
                .filter(|m: &Vec<Jid>| !m.is_empty())
                .ok_or(error)
        })?;
    fetch_devices(store, channel, &own, &members).await?;
    let plan = group_send::plan(store, chat, &own)?;
    println!(
        "group-send: chat addressing_mode={}, phash={}, recipients={}, devices={}, without_device_list={}",
        plan.addressing_mode,
        plan.participant_hash,
        plan.recipients.len(),
        plan.devices.len(),
        plan.without_device_list.len()
    );
    if !plan.without_device_list.is_empty() {
        return Err(invalid(
            "some members have no device list; they would silently not receive this message",
        ));
    }

    let mut targets = load_targets(store, &plan, message.fresh_sessions)?;
    if message.fresh_sessions {
        println!(
            "group-send: establishing fresh sessions for all {} target device(s); pinned identities are kept and still enforced",
            targets.len()
        );
    }
    let needed: Vec<Jid> = targets
        .iter()
        .filter(|t| t.session.is_none())
        .map(|t| t.device.clone())
        .collect();
    let keys = if needed.is_empty() {
        None
    } else {
        let id = format!("{}-keys", message.request_id);
        write_node(channel, &key_request_many(&needed, &id)?).await?;
        let (reply, _) = wait_for(channel, "iq", &id).await?;
        let dict = TokenDictionary::comparison_2_26_37_70();
        let wire = Encoder::new(dict).encode(&reply)?;
        store.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                format!("message:{}:keys", message.request_id).as_bytes(),
                1,
                &wire,
            )
        })?;
        println!(
            "group-send: key query for {} device(s) type={}",
            needed.len(),
            reply
                .get_attr("type")
                .and_then(|v| v.as_text())
                .unwrap_or("unknown")
        );
        Some(reply)
    };

    // Our own sender key for this chat. Loaded, advanced in memory, and written
    // once with everything else.
    let own_key_db = sender_key_db_key(chat, &own);
    let previous_key: Option<Vec<u8>> =
        store.read(|tx| tx.get_json(Table::SenderKeys, &own_key_db, 1))?;
    let minted = prepare_own_sender_key(chat, &own, previous_key.as_deref()).await?;
    // Not padded here: `prepare_payload` applies the E2E padding itself. Doing
    // it twice leaves junk bytes on the recipient's protobuf after their single
    // strip, which is what made this fail intermittently for weeks.
    let skdm_payload = minted.distribution_message.clone();

    let mut fanout = Vec::new();
    let mut updates = Vec::new();
    for target in &mut targets {
        let bundle = match (&target.session, &keys) {
            (Some(_), _) => None,
            (None, Some(reply)) => Some(prekey_bundle_for(reply, &target.device)?),
            (None, None) => return Err(invalid("a device has neither a session nor a bundle")),
        };
        let prepared = whatsapp_api::signal::direct::prepare_payload(
            account,
            &target.device,
            bundle.as_ref(),
            target.session.as_deref().map(|v| v.as_slice()),
            target.identity.as_deref(),
            &skdm_payload,
        )
        .await?;
        fanout.push(groups::FanoutTarget {
            device: target.device.clone(),
            kind: prepared.kind,
            ciphertext: prepared.ciphertext,
        });
        updates.push((
            session_key(&target.device),
            target.session.clone(),
            target.identity.clone(),
            prepared.session,
            prepared.identity,
        ));
    }

    // Out-of-band distribution: deliver each device its sender key as an
    // ordinary direct message first, on the 1:1 path that is live-verified, and
    // leave `<participants>` off the group stanza entirely.
    let out_of_band = message.skdm_out_of_band()?;
    if out_of_band {
        println!(
            "group-send: distributing the sender key out of band to {} device(s) before the group stanza",
            fanout.len()
        );
    }

    let plaintext = group_send::text_payload(&message.text)?;
    let (skmsg, advanced_key) =
        encrypt_group_payload(chat, &own, &minted.updated_key, &plaintext).await?;

    let wire_id = new_message_id()?;
    let node = groups::sender_key_stanza(
        chat,
        &wire_id,
        "text",
        Some(plan.addressing_mode),
        Some(&plan.participant_hash),
        if out_of_band { &[] } else { &fanout },
        &skmsg,
    )?;
    let dict = TokenDictionary::comparison_2_26_37_70();
    let wire = Encoder::new(dict).encode(&node)?;
    let intent = serde_json::json!({
        "status": "outcome_unknown",
        "wire_id": wire_id,
        "chat": chat.to_string(),
        "devices": fanout.len(),
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
        let current: Option<Vec<u8>> = tx.get_json(Table::SenderKeys, &own_key_db, 1)?;
        if current != previous_key {
            return Err(invalid("own sender key changed before commit"));
        }
        for (key, session, identity, next_session, next_identity) in &updates {
            let current: Option<Vec<u8>> = tx.get_json(Table::SignalSessions, key.as_bytes(), 1)?;
            let current_identity: Option<Vec<u8>> =
                tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?;
            // With a deliberate fresh session the stored ratchet is the one we
            // chose to replace, so only the identity is checked; otherwise both
            // must be exactly what we encrypted against.
            let session_unchanged = message.fresh_sessions
                || current.as_deref() == session.as_deref().map(|v| v.as_slice());
            if !session_unchanged || &current_identity != identity {
                return Err(invalid("a peer session changed before commit"));
            }
            tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &**next_session)?;
            tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, next_identity)?;
        }
        tx.put_json(Table::SenderKeys, &own_key_db, 1, &*advanced_key)?;
        tx.put_json(Table::Checkpoints, intent_key.as_bytes(), 1, &intent)
    })?;
    eprintln!(
        "group-send: sessions, own sender key and send intent committed; sending one fan-out to {} device(s)",
        fanout.len()
    );

    if out_of_band {
        for target in &fanout {
            let id = new_message_id()?;
            let distribution = whatsapp_api::transport::messaging::direct_message(
                &target.device,
                &id,
                target.kind,
                target.ciphertext.clone(),
            )?;
            write_node(channel, &distribution).await?;
            let (ack, _) = wait_for(channel, "ack", &id).await?;
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
        println!("group-send: all distribution messages acknowledged");
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
        println!("group-send: fan-out server_acknowledged; handset delivery/read not yet verified");
        answer_retries(
            store,
            account,
            channel,
            message,
            chat,
            &plan,
            &wire_id,
            &minted.distribution_message,
            &skmsg,
        )
        .await?;
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
    wire_id: &str,
    distribution: &[u8],
    skmsg: &[u8],
) -> Result<(), Error> {
    use whatsapp_api::transport::receipts::parse_retry;
    const WINDOW: std::time::Duration = std::time::Duration::from_secs(45);
    // One answer per device per send. A retry loop is the peer's to drive, and
    // answering the same device repeatedly in one window would be us, not them.
    let mut answered: Vec<Jid> = Vec::new();
    let deadline = tokio::time::Instant::now() + WINDOW;
    eprintln!(
        "group-send: holding the connection for {}s to answer retry receipts",
        WINDOW.as_secs()
    );
    for index in 0..64 {
        let next = tokio::time::timeout_at(
            deadline,
            whatsapp_api::transport::messaging::read_node(channel),
        )
        .await;
        let node = match next {
            Err(_) => break,
            Ok(Err(error)) => {
                eprintln!("group-send: post-send window ended: {error}");
                break;
            }
            Ok(Ok(node)) => node,
        };
        let dict = TokenDictionary::comparison_2_26_37_70();
        let bytes = Encoder::new(dict).encode(&node)?;
        store.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                format!("inbound:after-send-{}:{index}", message.request_id).as_bytes(),
                1,
                &bytes,
            )
        })?;
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

        let id = format!("{wire_id}-retry");
        write_node(
            channel,
            &key_request_many(std::slice::from_ref(&retry.device), &id)?,
        )
        .await?;
        let (reply, _) = wait_for(channel, "iq", &id).await?;
        let bundle = prekey_bundle_for(&reply, &retry.device)?;
        let peer_key = session_key(&retry.device);
        let identity: Option<Vec<u8>> =
            store.read(|tx| tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1))?;
        let prepared = whatsapp_api::signal::direct::prepare_payload(
            account,
            &retry.device,
            Some(&bundle),
            None,
            identity.as_deref(),
            distribution,
        )
        .await?;
        let resend = groups::sender_key_stanza(
            chat,
            wire_id,
            "text",
            Some(plan.addressing_mode),
            Some(&plan.participant_hash),
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
        answered.push(retry.device.clone());
        println!("group-send: retry answered; the recipient should now be able to read it");
    }
    println!(
        "group-send: post-send window complete; {} retry answer(s) sent",
        answered.len()
    );
    Ok(())
}
