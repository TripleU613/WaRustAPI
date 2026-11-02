//! Answer a peer's `receipt type="retry"` for one of our direct sends.
//!
//! **Confirmed static, 2.26.37.71** (docs/protocol/retry-answers.md):
//! `RetryReceiptHandler` (`150.A04`) skips `count > 4` (`0270..027e`) and
//! originals older than 60 days (`0426..043c`, `0x134fd9000` ms); `DNP.run`
//! skips a device that is not an original recipient (`0118..0136`) or already
//! has the message (`01c6..01f4`), then re-injects the send job over the
//! original message: the same plaintext re-encrypted, the same message id.
//! `151.A00` keeps the existing session when the receipt's `<registration>`
//! equals the session's `remoteRegistrationId_`; a mismatch deletes the session
//! and waits for a fresh prekey bundle, and `count > 2` may also wait for one.
//!
//! **Implemented here:** only the no-fetch branch: direct chat, count 1 or 2,
//! matching registration, server-accepted original within 60 days. Every
//! other case records `retry-unanswered:` with its reason and sends nothing.
//! Not reproduced: the count-2 base-key record (`0jc.A0u`), receipts carrying
//! `<keys>` (`151.A01`), group/broadcast retries, and an `enc count` attribute
//! (Civ's stanza writer is untraced; `answer_one` omits it too).

use crate::{
    Error, Result,
    binary::{Encoder, Node, NodeContent, TokenDictionary},
    model::{
        ids::{DeviceAddress, MessageId},
        send::DeliveryFactKind,
        time::Timestamp,
    },
    outbox::AttemptRecord,
    session::AccountRecord,
    store::{ProtocolStore, Table, WriteTx},
    transport::ConnectionEpoch,
};

/// `150.A04 0272`: counts above four are skipped.
pub const MAX_COUNT: u32 = 4;
/// `150.A04 0426`: originals older than 60 days are not resent.
pub const MAX_AGE_MS: i64 = 5_184_000_000;

/// The plaintext `Message` of a direct send, kept for a retry answer.
pub(crate) fn source_key(wire_id: &str) -> String {
    format!("retry-source:{wire_id}")
}

/// Retain the plaintext under the store's at-rest encryption at reservation.
pub(crate) fn keep_source_in(tx: &mut WriteTx<'_>, wire_id: &str, payload: &[u8]) -> Result<()> {
    tx.put_json(
        Table::Checkpoints,
        source_key(wire_id).as_bytes(),
        1,
        &payload,
    )
}

fn unanswered(store: &ProtocolStore, key: &str, reason: &str) -> Result<Option<Node>> {
    tracing::warn!(reason, "retry receipt not answered");
    store.write(|tx| {
        if tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?.is_none() {
            tx.put_json(
                Table::Checkpoints,
                key.as_bytes(),
                1,
                &serde_json::json!({"status": "unanswered", "reason": reason, "observed_at": Timestamp::now()}),
            )?;
        }
        Ok(())
    })?;
    Ok(None)
}

/// Answer `node` if it is a retry receipt for our server-accepted direct send.
/// Returns the resend to write, already reserved; `None` when not answered.
pub async fn answer(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    epoch: ConnectionEpoch,
) -> Result<Option<Node>> {
    let Some(retry) = super::receipts::parse_retry(node) else {
        return Ok(None);
    };
    let Ok(wire_id) = MessageId::new(retry.message_id.clone()) else {
        return Ok(None);
    };
    let outbox = super::outgoing::outbox();
    let Some(entry) = store.read(|tx| outbox.by_wire_id(tx, &wire_id))? else {
        return Ok(None); // Not one of ours (or a comparison send): nothing to say.
    };
    let key = format!(
        "retry-unanswered:{}:{}:{}",
        retry.message_id, retry.device, retry.count
    );
    let peer = entry.intent.chat.jid().clone();
    if retry.chat.is_group() || peer.is_group() {
        return unanswered(store, &key, "group_retry_unsupported");
    }
    if retry.count == 0 || retry.count > MAX_COUNT {
        return unanswered(store, &key, "count_out_of_range");
    }
    if retry.count > 2 {
        return unanswered(store, &key, "new_session_needed_unsupported");
    }
    // Our direct send addressed exactly `peer` (device 0); the receipt may
    // name that device by its LID.
    let lid = super::inbox::lid_for(store, &peer)?;
    let device = retry.device.to_non_ad();
    if retry.device.device != peer.device
        || !(device == peer.to_non_ad() || Some(&device) == lid.as_ref())
    {
        return unanswered(store, &key, "not_an_original_recipient");
    }
    if !entry
        .facts
        .iter()
        .any(|f| f.kind == DeliveryFactKind::ServerAccepted)
        || entry
            .facts
            .iter()
            .any(|f| f.kind == DeliveryFactKind::Failed)
    {
        return unanswered(store, &key, "original_not_server_accepted");
    }
    let addresses = [&peer, &retry.device]
        .into_iter()
        .filter_map(|j| DeviceAddress::from_jid(j).ok())
        .collect::<Vec<_>>();
    if entry.facts.iter().any(|f| {
        matches!(
            f.kind,
            DeliveryFactKind::Delivered | DeliveryFactKind::Read | DeliveryFactKind::Played
        ) && f.device.as_ref().is_none_or(|d| addresses.contains(d))
    }) {
        return unanswered(store, &key, "already_delivered");
    }
    let Some(first) = entry.attempts.first() else {
        return unanswered(store, &key, "original_attempt_missing");
    };
    if Timestamp::now().as_millis() - first.prepared_at.as_millis() > MAX_AGE_MS {
        return unanswered(store, &key, "original_older_than_60_days");
    }
    let Some(original) = first.payload.as_deref() else {
        return unanswered(store, &key, "original_stanza_missing");
    };
    let peer_key = format!("comparison-direct:{peer}");
    let answer_key = format!(
        "retry-answer:{}:{}:{}",
        retry.message_id, retry.device, retry.count
    );
    let bytes = |table, key: &str| -> Result<Option<Vec<u8>>> {
        store.read(|tx| tx.get_json(table, key.as_bytes(), 1))
    };
    let source = bytes(Table::Checkpoints, &source_key(wire_id.as_str()))?;
    let session = bytes(Table::SignalSessions, &peer_key)?;
    let identity = bytes(Table::PeerIdentities, &peer_key)?;
    let answered: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, answer_key.as_bytes(), 1))?;
    if answered.is_some() {
        return Ok(None); // One answer per (message, device, count), ever.
    }
    let Some(source) = source else {
        // Sends reserved before this module existed kept only ciphertext.
        return unanswered(store, &key, "plaintext_not_retained");
    };
    let registration = retry
        .registration
        .as_deref()
        .filter(|b| b.len() <= 4)
        .map(|b| b.iter().fold(0u32, |n, byte| (n << 8) | u32::from(*byte)));
    let session = match session {
        Some(session)
            if registration.is_some()
                && registration
                    == Some(crate::signal::direct::session_remote_registration(
                        &session,
                    )?) =>
        {
            session
        }
        // 151.A00: on a registration change (or no session) the APK drops the
        // session and "will wait to send … until a new prekey has been
        // fetched". Ask the directory for the device's bundle; the reply
        // finishes the answer in `complete_refetch`.
        _ => {
            let query = super::messaging::new_message_id()?;
            let reserved = store.write(|tx| {
                if tx
                    .get_json::<serde_json::Value>(Table::Checkpoints, answer_key.as_bytes(), 1)?
                    .is_some()
                {
                    return Ok(false);
                }
                tx.put_json(
                    Table::Checkpoints,
                    answer_key.as_bytes(),
                    1,
                    &serde_json::json!({"status": "fetching", "query": query, "count": retry.count}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    refetch_key(&query).as_bytes(),
                    1,
                    &Refetch {
                        wire_id: retry.message_id.clone(),
                        answer_key: answer_key.clone(),
                        unanswered_key: key.clone(),
                        count: retry.count,
                    },
                )?;
                Ok(true)
            })?;
            return Ok(reserved.then(|| super::messaging::key_request(&peer, &query)));
        }
    };
    let prepared = match crate::signal::direct::prepare_payload(
        account,
        &peer,
        None,
        Some(&session),
        identity.as_deref(),
        &source,
    )
    .await
    {
        Ok(prepared) => prepared,
        // A peer's receipt must not end receive; the pin is never adopted here.
        Err(Error::IdentityChanged(_)) => return unanswered(store, &key, "identity_changed"),
        Err(error) => return Err(error),
    };
    let resend = replace_enc(original, prepared.kind, prepared.ciphertext)?;
    let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&resend)?;
    store.write(|tx| {
        let mut current = outbox
            .by_wire_id_in_write(tx, &wire_id)?
            .ok_or_else(|| Error::invalid("retry", "original send vanished"))?;
        let held: Option<Vec<u8>> = tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
        if held.as_deref() != Some(session.as_slice())
            || tx
                .get_json::<serde_json::Value>(Table::Checkpoints, answer_key.as_bytes(), 1)?
                .is_some()
        {
            return Err(Error::invalid("retry", "session or answer changed before reservation"));
        }
        // Reservation precedes the write: a crash leaves an uncertain answer,
        // never a second one.
        tx.put_json(
            Table::Checkpoints,
            answer_key.as_bytes(),
            1,
            &serde_json::json!({"status": "reserved", "count": retry.count, "reserved_at": Timestamp::now()}),
        )?;
        current.attempts.push(AttemptRecord {
            index: current.attempts.len() as u32,
            prepared_at: Timestamp::now(),
            epoch: Some(epoch),
            payload: Some(wire.into()),
            re_encryption: true,
        });
        tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &*prepared.session)?;
        tx.put_json(Table::Outbox, &current.local_id.0, 1, &current)
    })?;
    Ok(Some(resend))
}

fn refetch_key(query: &str) -> String {
    format!("retry-refetch:{query}")
}

/// A retry answer waiting for the device's fresh prekey bundle.
#[derive(serde::Serialize, serde::Deserialize)]
struct Refetch {
    wire_id: String,
    answer_key: String,
    unanswered_key: String,
    count: u32,
}

/// Finish a retry answer from the directory reply to its key query.
/// `Ok(None)` when the reply is not one of ours, `Ok(Some(None))` when it was
/// ours but nothing is sent (the reason is recorded), else the resend.
///
/// A fresh session from the reply's bundle replaces the stale one, as the APK
/// does after a registration change (`151.A00`). The pinned identity still
/// holds: a bundle with a different identity is recorded `identity_changed`.
pub async fn complete_refetch(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    epoch: ConnectionEpoch,
) -> Result<Option<Option<Node>>> {
    if node.description != "iq" {
        return Ok(None);
    }
    let Some(query) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(None);
    };
    let Some(job): Option<Refetch> =
        store.read(|tx| tx.get_json(Table::Checkpoints, refetch_key(query).as_bytes(), 1))?
    else {
        return Ok(None);
    };
    store.write(|tx| tx.delete(Table::Checkpoints, refetch_key(query).as_bytes()))?;
    let refuse = |reason: &str| -> Result<Option<Option<Node>>> {
        store.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                job.answer_key.as_bytes(),
                1,
                &serde_json::json!({"status": "unanswered", "reason": reason}),
            )
        })?;
        unanswered(store, &job.unanswered_key, reason).map(|_| Some(None))
    };
    if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
        return refuse("refetch_refused");
    }
    let wire_id = MessageId::new(job.wire_id.clone())?;
    let outbox = super::outgoing::outbox();
    let Some(entry) = store.read(|tx| outbox.by_wire_id(tx, &wire_id))? else {
        return refuse("original_send_missing");
    };
    let peer = entry.intent.chat.jid().clone();
    let Some(original) = entry.attempts.first().and_then(|a| a.payload.clone()) else {
        return refuse("original_stanza_missing");
    };
    let bundle = match crate::signal::direct::prekey_bundle_for(node, &peer) {
        Ok(bundle) => bundle,
        Err(Error::InvalidInput { .. }) => return refuse("refetch_bundle_unusable"),
        Err(error) => return Err(error),
    };
    let peer_key = format!("comparison-direct:{peer}");
    let (source, identity): (Option<Vec<u8>>, Option<Vec<u8>>) = store.read(|tx| {
        Ok((
            tx.get_json(
                Table::Checkpoints,
                source_key(wire_id.as_str()).as_bytes(),
                1,
            )?,
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
        ))
    })?;
    let Some(source) = source else {
        return refuse("plaintext_not_retained");
    };
    let prepared = match crate::signal::direct::prepare_payload(
        account,
        &peer,
        Some(&bundle),
        None,
        identity.as_deref(),
        &source,
    )
    .await
    {
        Ok(prepared) => prepared,
        Err(Error::IdentityChanged(_)) => return refuse("identity_changed"),
        Err(Error::InvalidInput { .. }) => return refuse("refetch_bundle_unusable"),
        Err(error) => return Err(error),
    };
    let resend = replace_enc(&original, prepared.kind, prepared.ciphertext)?;
    let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&resend)?;
    store.write(|tx| {
        let state: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, job.answer_key.as_bytes(), 1)?;
        if state.as_ref().and_then(|v| v["query"].as_str()) != Some(query) {
            return Err(Error::invalid("retry", "answer changed before reservation"));
        }
        let mut current = outbox
            .by_wire_id_in_write(tx, &wire_id)?
            .ok_or_else(|| Error::invalid("retry", "original send vanished"))?;
        tx.put_json(
            Table::Checkpoints,
            job.answer_key.as_bytes(),
            1,
            &serde_json::json!({"status": "reserved", "count": job.count, "fresh_session": true, "reserved_at": Timestamp::now()}),
        )?;
        current.attempts.push(AttemptRecord {
            index: current.attempts.len() as u32,
            prepared_at: Timestamp::now(),
            epoch: Some(epoch),
            payload: Some(wire.into()),
            re_encryption: true,
        });
        tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &*prepared.session)?;
        tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &prepared.identity)?;
        tx.put_json(Table::Outbox, &current.local_id.0, 1, &current)
    })?;
    Ok(Some(Some(resend)))
}

/// The original stanza with its `enc` re-encrypted; id and every other
/// attribute and child unchanged.
fn replace_enc(original: &[u8], kind: &str, ciphertext: Vec<u8>) -> Result<Node> {
    let limits = crate::profile::Limits::default();
    let raw = crate::binary::unpack_frame_payload(original, &limits)?;
    let mut stanza =
        crate::binary::Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits)
            .decode_node()?;
    let NodeContent::Children(children) = &mut stanza.content else {
        return Err(Error::invalid("retry", "original stanza has no children"));
    };
    let mut replaced = 0;
    for child in children.iter_mut().filter(|c| c.description == "enc") {
        *child = child.clone().attr("type", kind).bytes(ciphertext.clone());
        replaced += 1;
    }
    if replaced != 1 {
        return Err(Error::invalid(
            "retry",
            "original stanza must hold exactly one enc",
        ));
    }
    Ok(stanza)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId, ClientRequestId},
            jid::Jid,
            message::MessageContent,
            send::{AttemptProgress, DeliveryFact, SendRequest},
        },
        session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::{DeviceId, IdentityKeyPair, KeyPair, PreKeyBundle};

    fn receipt(peer: &Jid, count: u32, registration: u32) -> Node {
        // Shape observed live 2026-09-24 (receipts::RetryRequest).
        Node::new("receipt")
            .attr("from", peer.clone())
            .attr("type", "retry")
            .attr("id", "W1")
            .children(vec![
                Node::new("retry")
                    .attr("count", count.to_string())
                    .attr("v", "1"),
                Node::new("registration").bytes(registration.to_be_bytes().to_vec()),
            ])
    }

    fn reason(store: &ProtocolStore, count: u32, peer: &Jid) -> Option<String> {
        let key = format!("retry-unanswered:W1:{peer}:{count}");
        store
            .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .map(|v| v["reason"].as_str().unwrap().to_owned())
    }

    /// The APK's no-fetch branch: count 1/2 with a matching registration is
    /// answered once, under the saved session and the original message id;
    /// every refused case is recorded with its reason and sends nothing.
    #[tokio::test]
    async fn answers_once_under_the_saved_session_and_records_every_refusal() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("retry.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap();
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let mut rng = rand::rng();
        let (local, remote, signed) = (
            IdentityKeyPair::generate(&mut rng),
            IdentityKeyPair::generate(&mut rng),
            KeyPair::generate(&mut rng),
        );
        let account = AccountRecord {
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: local.public_key().public_key_bytes().try_into().unwrap(),
                private: *local.private_key().serialize(),
            })),
            ..Default::default()
        };
        let signature = remote
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            None,
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *remote.identity_key(),
        )
        .unwrap();
        let first = crate::signal::direct::prepare(&account, &peer, &bundle, None, None, "hello")
            .await
            .unwrap();
        let original =
            super::super::messaging::direct_message(&peer, "W1", first.kind, first.ciphertext)
                .unwrap();
        let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&original)
            .unwrap();
        let outbox = super::super::outgoing::outbox();
        let key = format!("comparison-direct:{peer}");
        let source = crate::signal::direct::text_payload("hello").unwrap();
        store
            .write(|tx| {
                let request = SendRequest {
                    client_request_id: ClientRequestId::new("retry-test")?,
                    chat: ChatId::from_jid(&peer)?,
                    content: MessageContent::Text {
                        body: "hello".into(),
                        quoted: None,
                        style: Default::default(),
                    },
                };
                let id = outbox.enqueue(tx, &request)?.0.local_id;
                let mut entry = outbox.get_mut(tx, id)?.unwrap();
                entry.wire_id = Some(MessageId::new("W1")?);
                entry.attempts.push(AttemptRecord {
                    index: 0,
                    prepared_at: Timestamp::now(),
                    epoch: Some(ConnectionEpoch(1)),
                    payload: Some(wire.into()),
                    re_encryption: false,
                });
                outbox.set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                outbox.record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::ServerAccepted,
                        device: None,
                        observed_at: Timestamp::now(),
                        server_timestamp: None,
                        code: None,
                    },
                )?;
                keep_source_in(tx, "W1", &source)?;
                tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &*first.session)?;
                tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &first.identity)
            })
            .unwrap();
        let (store_ref, account_ref) = (&store, &account);
        let ask = |count: u32, registration: u32| {
            let node = receipt(&peer, count, registration);
            async move {
                // The order listen uses: a retry passes validation, records
                // no delivery fact, and only then reaches the answer.
                super::super::receipts::validate(&node)?;
                assert_eq!(super::super::receipts::record(store_ref, &node)?, 0);
                answer(store_ref, account_ref, &node, ConnectionEpoch(2)).await
            }
        };

        let resend = ask(1, 42).await.unwrap().expect("answered");
        assert_eq!(resend.get_attr("id").and_then(|v| v.as_text()), Some("W1"));
        assert_eq!(
            resend.get_attr("type").and_then(|v| v.as_text()),
            Some("text")
        );
        let enc = resend.child("enc").unwrap();
        assert_eq!(
            enc.get_attr("type").and_then(|v| v.as_text()),
            Some("pkmsg")
        );
        assert_ne!(enc.content, original.child("enc").unwrap().content);
        let entry = store
            .read(|tx| outbox.by_wire_id(tx, &MessageId::new("W1")?))
            .unwrap()
            .unwrap();
        assert_eq!(entry.attempts.len(), 2);
        assert!(entry.attempts[1].re_encryption);

        // Once per (message, device, count), even across a restart of the loop.
        assert!(ask(1, 42).await.unwrap().is_none());
        assert!(ask(3, 42).await.unwrap().is_none());
        assert_eq!(
            reason(&store, 3, &peer).as_deref(),
            Some("new_session_needed_unsupported")
        );
        assert!(ask(5, 42).await.unwrap().is_none());

        // 151.A00: a changed registration asks the directory for a fresh
        // bundle, once; its reply completes the answer with a new session.
        let query = ask(2, 43).await.unwrap().expect("key query");
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert!(ask(2, 43).await.unwrap().is_none(), "one fetch per count");
        let qid = query.get_attr("id").unwrap().as_text().unwrap().to_owned();
        let signed2 = KeyPair::generate(&mut rng);
        let signature2 = remote
            .private_key()
            .calculate_signature(&signed2.public_key.serialize(), &mut rng)
            .unwrap();
        let reply = Node::new("iq")
            .attr("type", "result")
            .attr("id", qid.as_str())
            .children(vec![Node::new("list").children(vec![
                Node::new("user").attr("jid", peer.clone()).children(vec![
                    Node::new("registration").bytes(43u32.to_be_bytes().to_vec()),
                    Node::new("type").bytes(vec![5]),
                    Node::new("identity").bytes(remote.public_key().public_key_bytes().to_vec()),
                    Node::new("skey").children(vec![
                        Node::new("id").bytes(vec![0, 0, 8]),
                        Node::new("value").bytes(signed2.public_key.public_key_bytes().to_vec()),
                        Node::new("signature").bytes(signature2.to_vec()),
                    ]),
                ]),
            ])]);
        let other = Node::new("iq")
            .attr("type", "result")
            .attr("id", "unrelated");
        assert!(
            complete_refetch(&store, &account, &other, ConnectionEpoch(2))
                .await
                .unwrap()
                .is_none()
        );
        let resend = complete_refetch(&store, &account, &reply, ConnectionEpoch(2))
            .await
            .unwrap()
            .unwrap()
            .expect("fresh-session resend");
        assert_eq!(resend.get_attr("id").and_then(|v| v.as_text()), Some("W1"));
        assert_eq!(
            resend
                .child("enc")
                .unwrap()
                .get_attr("type")
                .and_then(|v| v.as_text()),
            Some("pkmsg"),
            "a new session starts with a prekey message"
        );
        let session: Vec<u8> = store
            .read(|tx| tx.get_json(Table::SignalSessions, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            crate::signal::direct::session_remote_registration(&session).unwrap(),
            43
        );
        assert!(
            complete_refetch(&store, &account, &reply, ConnectionEpoch(2))
                .await
                .unwrap()
                .is_none(),
            "settled once"
        );
        assert_eq!(
            reason(&store, 5, &peer).as_deref(),
            Some("count_out_of_range")
        );

        store
            .write(|tx| {
                let mut entry = outbox
                    .by_wire_id_in_write(tx, &MessageId::new("W1")?)?
                    .unwrap();
                outbox.record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::Delivered,
                        device: Some(DeviceAddress::from_jid(&peer)?),
                        observed_at: Timestamp::now(),
                        server_timestamp: None,
                        code: None,
                    },
                )
            })
            .unwrap();
        assert!(ask(2, 42).await.unwrap().is_none());
        assert_eq!(
            reason(&store, 2, &peer).as_deref(),
            Some("already_delivered")
        );
    }
}
