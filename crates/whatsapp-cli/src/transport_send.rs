//! One durable experimental direct send. No replay after uncertain writes.
use whatsapp_api::{
    Error,
    binary::{Encoder, Node, TokenDictionary},
    session::AccountRecord,
    signal::direct::{prekey_bundle, prepare, prepare_payload},
    store::{ProtocolStore, Table},
    transport::{
        handshake::EncryptedConnection,
        messaging::{MessageRequest, direct_message, write_and_wait_captured},
    },
};

// The injected exchange is the actual CLI captured writer, or an offline
// fault boundary in tests. The adapter never opens its own connection.
pub(super) async fn fetch_keys<R>(
    store: &ProtocolStore,
    request_id: &whatsapp_api::model::ids::ClientRequestId,
    recipient: &whatsapp_api::model::jid::Jid,
    mut exchange: R,
) -> Result<(Node, Vec<Node>), Error>
where
    R: AsyncFnMut(&Node, &str) -> Result<(Node, Vec<Node>), Error>,
{
    // Reuse the group path's frozen-target, once-only key-query state machine.
    // The legacy cache alone cannot prove which request/recipient owned it.
    use whatsapp_api::transport::device_key_batches;
    let targets = std::slice::from_ref(recipient);
    let mut pending = Vec::new();
    while let Some(query) = device_key_batches::reserve_next(store, request_id, targets)? {
        let query_id = query
            .get_attr("id")
            .and_then(|value| value.as_text())
            .ok_or_else(|| Error::invalid("message", "reserved key query has no ID"))?;
        let (reply, observed) = exchange(&query, query_id).await?;
        pending.extend(observed);
        if !device_key_batches::settle(store, request_id, &reply)? {
            return Err(Error::invalid(
                "message",
                "key reply does not own the reserved query",
            ));
        }
    }
    let reply = device_key_batches::assembled(store, request_id, targets)?
        .ok_or_else(|| Error::invalid("message", "direct query has no retained bundle"))?;
    Ok((reply, pending))
}

// A captured ACK does not own a send whose durable intent was replaced.
pub(super) fn check_reserved_intent(
    tx: &whatsapp_api::store::WriteTx<'_>,
    key: &[u8],
    expected: &serde_json::Value,
) -> Result<(), Error> {
    let current: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, key, 1)?;
    if expected["status"] != "outcome_unknown" || current.as_ref() != Some(expected) {
        return Err(Error::invalid(
            "message",
            "send intent changed before acknowledgement settlement",
        ));
    }
    Ok(())
}

pub(super) fn settle_reserved_intent(
    tx: &mut whatsapp_api::store::WriteTx<'_>,
    key: &[u8],
    expected: &serde_json::Value,
    result: &serde_json::Value,
) -> Result<(), Error> {
    check_reserved_intent(tx, key, expected)?;
    tx.put_json(Table::Checkpoints, key, 1, result)
}

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
    let (ack, pending) = write_and_wait_captured(store, channel, &node, "ack", &wire_id).await?;
    let outcome = whatsapp_api::transport::messaging::message_ack_outcome_for(&ack, &recipient);
    let accepted = outcome == whatsapp_api::transport::messaging::MessageAckOutcome::Accepted;
    let code = ack
        .get_attr("error")
        .and_then(|v| v.as_text())
        .filter(|v| v.len() <= 5 && v.bytes().all(|b| b.is_ascii_digit()));
    let mut result = intent.clone();
    result["status"] = serde_json::json!(outcome.status());
    result["error"] = serde_json::json!(code);
    store.write(|tx| {
        settle_reserved_intent(tx, intent_key.as_bytes(), &intent, &result)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use whatsapp_api::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };

    #[tokio::test]
    async fn direct_key_query_is_reserved_before_exchange_and_never_replayed_after_cut() {
        use whatsapp_api::{
            model::{ids::ClientRequestId, jid::Jid},
            transport::device_key_batches,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("direct-key-cut.redb");
        let open = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("fixture").unwrap(),
                create: true,
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            })
            .unwrap()
        };
        let store = open();
        let id = ClientRequestId::new("direct-cut").unwrap();
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let captured = std::cell::RefCell::new(Vec::new());
        let calls = std::cell::Cell::new(0);
        let result = fetch_keys(&store, &id, &peer, async |_, _| {
            calls.set(calls.get() + 1);
            assert_eq!(
                device_key_batches::summary(&store, &id)?
                    .expect("query reservation must precede writer")
                    .outcome_unknown,
                1
            );
            *captured.borrow_mut() = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
            Err(Error::Transport(
                whatsapp_api::transport::TransportError::Closed,
            ))
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.get(), 1);
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap(),
            *captured.borrow()
        );
        drop(store);
        let reopened = open();
        let changed = Jid::parse("15555550200@s.whatsapp.net").unwrap();
        for target in [&peer, &changed] {
            assert!(
                fetch_keys(&reopened, &id, target, async |_, _| {
                    panic!("uncertain query cannot be written again")
                })
                .await
                .is_err()
            );
        }
        assert_eq!(
            reopened
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap(),
            *captured.borrow()
        );
    }

    #[tokio::test]
    async fn direct_key_query_refuses_unbound_legacy_cache_without_overwrite() {
        use whatsapp_api::model::{ids::ClientRequestId, jid::Jid};
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("legacy.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            create: true,
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
        })
        .unwrap();
        let id = ClientRequestId::new("legacy-direct").unwrap();
        let bytes = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(
                &Node::new("iq")
                    .attr("type", "result")
                    .children(vec![Node::new("list")]),
            )
            .unwrap();
        store
            .write(|tx| tx.put_json(Table::Checkpoints, b"message:legacy-direct:keys", 1, &bytes))
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
            .unwrap();
        let journal = store.journal_head().unwrap();
        assert!(
            fetch_keys(
                &store,
                &id,
                &Jid::parse("15555550100@s.whatsapp.net").unwrap(),
                async |_, _| { panic!("legacy cache must not authorize another query") }
            )
            .await
            .is_err()
        );
        assert_eq!(store.journal_head().unwrap(), journal);
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap(),
            before
        );
    }

    #[tokio::test]
    async fn direct_key_query_refusal_or_foreign_reply_never_authorizes_another_write() {
        use whatsapp_api::{
            model::{ids::ClientRequestId, jid::Jid},
            transport::device_key_batches,
        };
        for foreign in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("direct-query-reply.redb");
            let open = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create: true,
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
                })
                .unwrap()
            };
            let store = open();
            let id = ClientRequestId::new("direct-refusal").unwrap();
            let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
            let calls = std::cell::Cell::new(0);
            assert!(
                fetch_keys(&store, &id, &peer, async |_, query_id| {
                    calls.set(calls.get() + 1);
                    Ok((
                        Node::new("iq")
                            .attr("id", if foreign { "unrelated" } else { query_id })
                            .attr("type", if foreign { "result" } else { "error" }),
                        Vec::new(),
                    ))
                })
                .await
                .is_err()
            );
            assert_eq!(calls.get(), 1);
            let summary = device_key_batches::summary(&store, &id).unwrap().unwrap();
            assert_eq!(summary.refused, usize::from(!foreign));
            assert_eq!(summary.outcome_unknown, usize::from(foreign));
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            drop(store);
            let reopened = open();
            assert!(
                fetch_keys(&reopened, &id, &peer, async |_, _| {
                    panic!("no retry after refusal or unowned reply")
                })
                .await
                .is_err()
            );
            assert_eq!(
                reopened
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                    .unwrap(),
                before
            );
        }
    }

    #[test]
    fn stale_direct_ack_cannot_overwrite_current_intent_after_reopen() {
        let expected = serde_json::json!({"status":"outcome_unknown", "wire_id":"original", "to":"fixture", "wire":[1,2,3]});
        let accepted = serde_json::json!({"status":"server_acknowledged", "wire_id":"original"});
        for current in [
            None,
            Some(serde_json::json!({"status":"outcome_unknown", "wire_id":"replacement"})),
            Some(serde_json::json!({"status":"server_refused", "wire_id":"original"})),
            Some(accepted.clone()),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("direct-ack.redb");
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
                    if let Some(current) = &current {
                        tx.put_json(Table::Checkpoints, b"send", 1, current)?;
                    }
                    // The production wait captures before invoking this settlement.
                    tx.put_json(
                        Table::Checkpoints,
                        b"captured-ack",
                        1,
                        &serde_json::json!({"class":"message","id":"original"}),
                    )
                })
                .unwrap();
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            let settle = |store: &ProtocolStore| {
                store.write(|tx| settle_reserved_intent(tx, b"send", &expected, &accepted))
            };
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
        }
    }

    #[test]
    fn exact_direct_send_reservation_settles_once_without_rewriting_crypto() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("direct-valid.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            create: true,
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
        })
        .unwrap();
        let expected =
            serde_json::json!({"status":"outcome_unknown","wire_id":"original","wire":[1,2,3]});
        let mut result = expected.clone();
        result["status"] = serde_json::json!("server_acknowledged");
        store
            .write(|tx| {
                tx.put_json(Table::Checkpoints, b"send", 1, &expected)?;
                tx.put(Table::SignalSessions, b"fixture", 1, b"unchanged-session")
            })
            .unwrap();
        store
            .write(|tx| settle_reserved_intent(tx, b"send", &expected, &result))
            .unwrap();
        let journal = store.journal_head().unwrap();
        assert!(
            store
                .write(|tx| settle_reserved_intent(tx, b"send", &expected, &result))
                .is_err()
        );
        assert_eq!(store.journal_head().unwrap(), journal);
        assert_eq!(
            store
                .read(|tx| tx.get(Table::SignalSessions, b"fixture", 1))
                .unwrap()
                .unwrap(),
            b"unchanged-session"
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, b"send", 1))
                .unwrap(),
            Some(result)
        );
    }
}
