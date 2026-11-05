//! One durable experimental direct send. No replay after uncertain writes.
use whatsapp_api::{
    Error,
    binary::{Encoder, Node, TokenDictionary},
    session::AccountRecord,
    signal::direct::{prekey_bundle, prepare, prepare_payload},
    store::{ProtocolStore, Table},
    transport::{
        handshake::EncryptedConnection,
        messaging::{MessageRequest, direct_message, wait_for_captured as wait_for, write_node},
    },
};

pub(super) async fn send(
    store: &ProtocolStore,
    account: &AccountRecord,
    channel: &mut EncryptedConnection<tokio::net::TcpStream>,
    message: &MessageRequest,
    reply: &Node,
    image_payload: Option<&[u8]>,
) -> Result<(), Error> {
    let recipient = message.recipient()?;
    let bundle = prekey_bundle(reply, &recipient)?;
    let intent_key = format!("message:{}:send", message.request_id);
    let peer_key = format!("comparison-direct:{recipient}");
    let (session, identity): (Option<Vec<u8>>, Option<Vec<u8>>) = store.read(|tx| {
        let previous: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, intent_key.as_bytes(), 1)?;
        if previous.is_some() {
            return Err(Error::invalid(
                "message",
                "send intent already exists; no replay",
            ));
        }
        Ok((
            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
        ))
    })?;
    // A deliberately fresh session drops only the ratchet; the pinned peer
    // identity below is still loaded and still enforced.
    let session = session
        .filter(|_| !message.fresh_sessions)
        .map(zeroize::Zeroizing::new);
    if message.fresh_sessions {
        eprintln!(
            "transport: establishing a fresh session with this recipient; the pinned identity is kept and still enforced"
        );
    }
    let prepared = if let Some(payload) = image_payload {
        prepare_payload(
            account,
            &recipient,
            Some(&bundle),
            session.as_deref().map(|v| v.as_slice()),
            identity.as_deref(),
            payload,
        )
        .await?
    } else {
        prepare(
            account,
            &recipient,
            &bundle,
            session.as_deref().map(|v| v.as_slice()),
            identity.as_deref(),
            &message.text,
        )
        .await?
    };
    let wire_id = whatsapp_api::transport::messaging::new_message_id()?;
    let node = if image_payload.is_some() {
        whatsapp_api::transport::messaging::direct_media(
            &recipient,
            &wire_id,
            prepared.kind,
            prepared.ciphertext,
            if message.voice_note.is_some() {
                "ptt"
            } else if message.document.is_some() {
                "document"
            } else if message.video.is_some() {
                "video"
            } else {
                "image"
            },
        )?
    } else {
        direct_message(&recipient, &wire_id, prepared.kind, prepared.ciphertext)?
    };
    let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&node)?;
    let image_hash = whatsapp_api::transport::image_upload::fingerprint(message)?;
    let intent = serde_json::json!({"status":"outcome_unknown","wire_id":wire_id,"to":message.to.as_str(),"text":message.text.as_str(),"image_hash":image_hash,"wire":wire,"at":whatsapp_api::model::time::Timestamp::now()});
    store.write(|tx| {
        let previous: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, intent_key.as_bytes(), 1)?;
        if previous.is_some() {
            return Err(Error::invalid("message", "send intent already reserved"));
        }
        if image_payload.is_some() {
            let upload: Option<serde_json::Value> = tx.get_json(
                Table::Checkpoints,
                format!("message:{}:upload", message.request_id).as_bytes(),
                1,
            )?;
            if !upload.is_some_and(|v| {
                v["status"] == "uploaded"
                    && v["hash"] == image_hash
                    && v["to"] == message.to.as_str()
            }) {
                return Err(Error::invalid(
                    "image",
                    "source changed after upload; send not reserved",
                ));
            }
        }
        let current_session: Option<Vec<u8>> =
            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
        let current_identity: Option<Vec<u8>> =
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        let session_unchanged = message.fresh_sessions
            || current_session.as_deref() == session.as_deref().map(|v| v.as_slice());
        if !session_unchanged || current_identity != identity {
            return Err(Error::invalid(
                "message",
                "peer session changed before commit",
            ));
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
        tx.put_json(Table::Checkpoints, intent_key.as_bytes(), 1, &intent)
    })?;
    eprintln!(
        "transport: session and send intent committed; sending one encrypted {} message",
        if image_payload.is_some() {
            if message.voice_note.is_some() {
                "voice-note"
            } else {
                "image"
            }
        } else {
            "text"
        }
    );
    write_node(channel, &node).await?;
    let (ack, pending) = wait_for(store, channel, "ack", &wire_id).await?;
    let outcome = whatsapp_api::transport::messaging::message_ack_outcome(&ack);
    let accepted = outcome == whatsapp_api::transport::messaging::MessageAckOutcome::Accepted;
    let code = ack
        .get_attr("error")
        .and_then(|v| v.as_text())
        .filter(|v| v.len() <= 5 && v.bytes().all(|b| b.is_ascii_digit()));
    let mut result = intent;
    result["status"] = serde_json::json!(outcome.status());
    result["error"] = serde_json::json!(code);
    store.write(|tx| {
        tx.put_json(Table::Checkpoints, intent_key.as_bytes(), 1, &result)?;
        for (index, node) in pending.iter().chain(std::iter::once(&ack)).enumerate() {
            let bytes = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(node)?;
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
        println!("transport: message server_acknowledged; handset delivery/read not yet verified");
        Ok(())
    } else {
        Err(Error::invalid(
            "message",
            format!(
                "message acknowledgement refused; code={}",
                code.unwrap_or("unknown")
            ),
        ))
    }
}
