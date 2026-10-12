//! Authenticated own-device synchronization over an already held session.
use super::inbox::IncomingResult;
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::{Jid, Server},
    session::AccountRecord,
    store::{ProtocolStore, Table},
};
use prost::Message;
use zeroize::Zeroizing;

fn shape(account: &AccountRecord, node: &Node) -> Option<(Jid, Jid, String)> {
    // shortcut: alternate recipient mapping and special namespaces need their saved migration state.
    if [
        "participant",
        "peer_recipient_pn",
        "peer_recipient_lid",
        "peer_recipient_username",
        "recipient_latest_lid",
    ]
    .iter()
    .any(|name| node.get_attr(name).is_some())
        || !super::chat_settings::sender_revoke_edit(node)
        || node.get_attr("category").and_then(|v| v.as_text()) == Some("peer")
    {
        return None;
    }
    if !matches!(
        node.get_attr("type").and_then(|v| v.as_text()),
        Some("text" | "media" | "poll" | "reaction")
    ) {
        return None;
    }
    let own = account.own_jid.as_ref()?;
    if !account.is_registered() || own.agent != 0 || own.device != 0 {
        return None;
    }
    let source = super::receipts::jid(node, "from")?;
    if source.agent != 0 || source.device == 0 || source.user.is_empty() {
        return None;
    }
    let chat = super::receipts::recipient(node)?;
    if chat.agent != 0
        || chat.device != 0
        || chat.user.is_empty()
        || !matches!(chat.server, Server::User | Server::Lid)
    {
        return None;
    }
    let id = node.get_attr("id")?.as_text()?;
    if id.is_empty() || id.len() > 128 {
        return None;
    }
    let key = format!("received-own:{}:{id}", chat.to_non_ad());
    Some((source, chat.to_non_ad(), key))
}

pub(super) fn route(
    account: &AccountRecord,
    node: &Node,
    own_lid: Option<&Jid>,
) -> Option<(Jid, Jid, String)> {
    let (source, chat, key) = shape(account, node)?;
    let own = account.own_jid.as_ref()?;
    let authority = match source.server {
        Server::User => own,
        Server::Lid => own_lid?,
        _ => return None,
    };
    if source.to_non_ad() != authority.to_non_ad() {
        return None;
    }
    Some((source, chat, key))
}

pub(super) fn payload(payload: &[u8], chat: &Jid) -> Option<Zeroizing<Vec<u8>>> {
    #[derive(Message)]
    struct Envelope {
        #[prost(message, optional, tag = "31")]
        sent: Option<Sent>,
        #[prost(bytes = "vec", optional, tag = "35")]
        context: Option<Vec<u8>>,
    }
    #[derive(Message)]
    struct Sent {
        #[prost(string, optional, tag = "1")]
        destination: Option<String>,
        #[prost(bytes = "vec", optional, tag = "2")]
        message: Option<Vec<u8>>,
        #[prost(string, optional, tag = "3")]
        phash: Option<String>,
    }
    #[derive(Message)]
    struct Content {
        #[prost(string, optional, tag = "1")]
        conversation: Option<String>,
        #[prost(bytes = "vec", optional, tag = "6")]
        extended: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "3")]
        image: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "7")]
        document: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "8")]
        audio: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "9")]
        video: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "26")]
        sticker: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "66")]
        ptv: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "12")]
        protocol: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "46")]
        reaction: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "50")]
        vote: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "51")]
        keep: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "64")]
        poll: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "35")]
        context: Option<Vec<u8>>,
    }
    let fields = crate::signal::direct::payload_fields(payload).ok()?;
    if fields.iter().any(|f| !matches!(f, 31 | 35))
        || fields.iter().filter(|&&f| f == 31).count() != 1
    {
        return None;
    }
    let outer = Envelope::decode(payload).ok()?;
    if let Some(context) = &outer.context {
        crate::signal::direct::payload_fields(context).ok()?;
    }
    let sent = outer.sent?;
    let destination = sent.destination.map(Zeroizing::new)?;
    let _phash = sent.phash.map(Zeroizing::new);
    if destination
        .rsplit_once('@')
        .is_some_and(|(user, _)| user.contains(':'))
    {
        return None;
    }
    let parsed = Jid::parse(&destination).ok()?;
    if parsed.agent != 0 || parsed.device != 0 || parsed.to_non_ad() != *chat {
        return None;
    }
    let inner = Zeroizing::new(sent.message?);
    let fields = crate::signal::direct::payload_fields(&inner).ok()?;
    // shortcut: other control effects and view-once synchronization need their separate native lifecycle.
    if fields.iter().any(|f| {
        !matches!(
            f,
            1 | 3 | 6 | 7 | 8 | 9 | 12 | 26 | 35 | 46 | 50 | 51 | 64 | 66
        )
    }) || fields.iter().filter(|&&f| f != 35).count() != 1
    {
        return None;
    }
    let mut text = Content::decode(inner.as_slice()).ok()?;
    if text.poll.is_some() && !super::polls::ordinary_creation_fields(&inner) {
        return None;
    }
    if text.protocol.is_some()
        && super::chat_settings::decode_sender_revoke(&inner).is_none()
        && !matches!(
            super::chat_settings::decode_protocol(&inner),
            Some(
                super::chat_settings::Setting::DisappearingTimer { .. }
                    | super::chat_settings::Setting::EphemeralSyncResponse { .. }
            )
        )
    {
        return None;
    }
    if let Some(context) = &text.context {
        crate::signal::direct::payload_fields(context).ok()?;
    }
    if let Some(context) = outer.context {
        // CjY.A01 replaces inner Message context with outer context when present.
        text.context = Some(context);
        Some(Zeroizing::new(text.encode_to_vec()))
    } else {
        Some(inner)
    }
}

/// A unique authenticated normal own message, retaining its original chat selector.
pub(super) fn record_in(
    tx: &crate::store::WriteTx<'_>,
    peer: &Jid,
    message_id: &str,
) -> Result<Option<super::inbox::DirectRecord>> {
    let Some(account) = AccountRecord::load_in_write(tx)?.filter(|a| a.is_registered()) else {
        return Ok(None);
    };
    let Some(own_pn) = account
        .own_jid
        .filter(|j| j.server == Server::User && j.agent == 0 && j.device == 0)
    else {
        return Ok(None);
    };
    let own_lid = super::group_send::own_lid_in(tx)?;
    let mut selected = None;
    // shortcut: synchronized parents use a paged scan; index them when large histories need faster parent lookup.
    let mut cursor = Some(b"received-own:".to_vec());
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        let done = rows.len() < 256;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows {
            if !key.starts_with(b"received-own:") {
                return Ok(selected);
            }
            let Some((chat, id)) = std::str::from_utf8(&key)
                .ok()
                .and_then(|k| k.strip_prefix("received-own:"))
                .and_then(|k| k.rsplit_once(':'))
            else {
                continue;
            };
            if id != message_id {
                continue;
            }
            let Ok(chat) = Jid::parse(chat) else {
                continue;
            };
            if !super::jid_map::authenticated_direct_chat(tx, peer, &chat)? {
                continue;
            }
            let Some(saved) = tx.get_json::<serde_json::Value>(Table::Checkpoints, &key, 1)? else {
                continue;
            };
            if saved["status"] != "decrypted" {
                continue;
            }
            let Ok(source) = serde_json::from_value::<Jid>(saved["source"].clone()) else {
                continue;
            };
            if source.agent != 0
                || source.device == 0
                || (source.to_non_ad() != own_pn.to_non_ad()
                    && own_lid.as_ref() != Some(&source.to_non_ad()))
            {
                continue;
            }
            let Ok(wrapped) =
                serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()).map(Zeroizing::new)
            else {
                continue;
            };
            let Some(plain) = payload(&wrapped, &chat) else {
                continue;
            };
            if crate::signal::direct::payload_fields(&plain)
                .is_ok_and(|fields| fields.iter().any(|f| matches!(f, 12 | 46 | 50 | 51)))
            {
                continue;
            }
            if selected.is_some() {
                return Err(Error::invalid(
                    "own_device",
                    "ambiguous synchronized parent",
                ));
            }
            selected = Some(super::inbox::DirectRecord {
                chat: chat.clone(),
                key: format!("received-own:{chat}:{id}"),
                saved: Some(saved),
            });
        }
        if done {
            break;
        }
    }
    Ok(selected)
}

fn saved_edit_matches(node: &Node, chat: &Jid, saved: &serde_json::Value) -> bool {
    // An edited redelivery needs authenticated revoke content, not just a saved ID.
    node.get_attr("edit").is_none()
        || serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone())
            .ok()
            .map(Zeroizing::new)
            .and_then(|plain| payload(&plain, chat))
            .is_some_and(|plain| super::chat_settings::decode_sender_revoke(&plain).is_some())
}

pub(super) fn confirmation(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
) -> Result<Option<Node>> {
    let Some(id) = node.get_attr("id").and_then(|value| value.as_text()) else {
        return Ok(None);
    };
    let Some((source, _, _)) = shape(account, node) else {
        return Ok(None);
    };
    let lid = if source.server == Server::Lid {
        super::group_send::own_lid(store)?
    } else {
        None
    };
    let Some((source, chat, key)) = route(account, node, lid.as_ref()) else {
        return Ok(None);
    };
    let authenticated = store.read(|tx| {
        let saved = tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?;
        Ok(saved.is_some_and(|saved| {
            saved["status"] == "decrypted"
                && saved_edit_matches(node, &chat, &saved)
                && serde_json::from_value::<Jid>(saved["source"].clone())
                    .ok()
                    .as_ref()
                    == Some(&source)
        }))
    })?;
    // Bp2.A0P uses the normalized key held in A08; Cp2.A04 selects sender/recipient.
    Ok(authenticated.then(|| {
        Node::new("receipt")
            .attr("to", source)
            .attr("id", id)
            .attr("type", "sender")
            .attr("recipient", chat)
    }))
}

fn publish(
    tx: &mut crate::store::WriteTx<'_>,
    key: &[u8],
    event: crate::sync::SyncEvent,
) -> Result<()> {
    let local_id = if let crate::sync::SyncEvent::NewMessage(message) = &event {
        let outbox =
            crate::outbox::Outbox::new("comparison-driver", crate::profile::Limits::default());
        match outbox.by_wire_id_in_write(tx, &message.key.id)? {
            Some(entry)
                if matches!(
                    entry.intent.content,
                    crate::model::message::MessageContent::Text { .. }
                        | crate::model::message::MessageContent::Media { .. }
                        | crate::model::message::MessageContent::Poll { .. }
                ) && super::jid_map::authenticated_direct_chat(
                    tx,
                    message.key.chat.jid(),
                    entry.intent.chat.jid(),
                )? =>
            {
                Some(entry.local_id)
            }
            _ => None,
        }
    } else {
        None
    };
    if let Some(local_id) = local_id {
        // 0zC.A0A preserves ordinary existing rows; an own-device echo is not a send outcome.
        tx.put_json(
            Table::Inbox,
            key,
            1,
            &serde_json::json!({"kind":"outgoing_echo", "local_id":local_id}),
        )
    } else {
        let event = tx.append_event(event)?;
        tx.put_json(
            Table::Inbox,
            key,
            1,
            &serde_json::json!({"event_id":event.event_id,"kind":event.event.kind()}),
        )
    }
}

pub(super) async fn process(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    capture: &[u8],
) -> Result<IncomingResult> {
    let Some((source, _, _)) = shape(account, node) else {
        return Ok(IncomingResult::Unsupported);
    };
    let lid = if source.server == Server::Lid {
        super::group_send::own_lid(store)?
    } else {
        None
    };
    let Some((source, chat, key)) = route(account, node, lid.as_ref()) else {
        return Ok(IncomingResult::Unsupported);
    };
    let peer_key = format!("comparison-direct:{source}");
    let (session, identity, saved) = store.read(|tx| {
        Ok((
            tx.get_json::<Vec<u8>>(Table::SignalSessions, peer_key.as_bytes(), 1)?,
            tx.get_json::<Vec<u8>>(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
            tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?,
        ))
    })?;
    let (Some(session), Some(identity)) = (session, identity) else {
        return Ok(IncomingResult::Unsupported);
    };
    let session = Zeroizing::new(session);
    if let Some(saved) = saved {
        return Ok(
            if saved["status"] == "decrypted"
                && saved_edit_matches(node, &chat, &saved)
                && serde_json::from_value::<Jid>(saved["source"].clone())
                    .ok()
                    .as_ref()
                    == Some(&source)
            {
                IncomingResult::Duplicate
            } else {
                IncomingResult::Unsupported
            },
        );
    }
    let enc: Vec<_> = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "enc")
        .collect();
    if enc.len() != 1
        || enc[0].get_attr("type").and_then(|v| v.as_text()) != Some("msg")
        || enc[0].get_attr("v").and_then(|v| v.as_text()) != Some("2")
    {
        return Ok(IncomingResult::Unsupported);
    }
    let ciphertext = match &enc[0].content {
        NodeContent::Bytes(v) => v.as_ref(),
        NodeContent::Text(v) => v.as_bytes(),
        _ => return Ok(IncomingResult::Unsupported),
    };
    if ciphertext.is_empty() {
        return Ok(IncomingResult::Unsupported);
    }
    let received =
        match crate::signal::direct::receive(account, &source, &session, &identity, ciphertext)
            .await
        {
            Ok(received) => received,
            // shortcut: own-device retry routing is unported; do not emit an ordinary direct retry.
            Err(Error::InvalidInput {
                field: "direct_message",
                reason,
            }) => {
                tracing::warn!(reason, "own-device ciphertext preserved; recovery unported");
                return Ok(IncomingResult::Unsupported);
            }
            Err(error) => return Err(error),
        };
    let Some(payload) = payload(&received.plaintext, &chat) else {
        return Ok(IncomingResult::Unsupported);
    };
    let route_bytes =
        crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
            .encode(node)?;
    store.write(|tx| {
        let current = AccountRecord::load_in_write(tx)?.ok_or(Error::NotRegistered)?;
        let current_lid = if source.server == Server::Lid { super::group_send::own_lid_in(tx)? } else { None };
        if current.generation != account.generation
            || current.signal_registration_id != account.signal_registration_id
            || current.signal_identity.as_ref().map(|k| &k.0.public) != account.signal_identity.as_ref().map(|k| &k.0.public)
            || route(&current, node, current_lid.as_ref()) != Some((source.clone(), chat.clone(), key.clone()))
            || tx.get_json::<Vec<u8>>(Table::SignalSessions, peer_key.as_bytes(), 1)?.as_deref() != Some(session.as_slice())
            || tx.get_json::<Vec<u8>>(Table::PeerIdentities, peer_key.as_bytes(), 1)?.as_ref() != Some(&identity)
            || tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?.is_some()
        { return Err(Error::invalid("inbox", "own-device route or session changed before commit")); }
        let own = super::inbox::own_identities_in(tx)?;
        let Some(event) = super::inbox_events::own_device_message(tx, key.as_bytes(), &chat, node, &payload, &own)? else { return Ok(IncomingResult::Unsupported); };
        tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &*received.session)?;
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &serde_json::json!({
            "status":"decrypted", "source":source, "protobuf":&*received.plaintext,
            "capture":capture, "route":route_bytes, "received_at":crate::model::time::Timestamp::now()
        }))?;
        publish(tx, key.as_bytes(), event)?;
        Ok(IncomingResult::Decrypted)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outgoing_echo_matches_only_saved_wire_and_authenticated_chat_without_send_mutation() {
        use crate::model::{
            ids::{ChatId, ClientRequestId, MessageId},
            message::MessageContent,
            send::SendRequest,
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(crate::store::OpenOptions {
            path: dir.path().join("echo.redb"),
            account_id: crate::model::ids::AccountId::new("fixture").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([9; 32]),
            )),
        })
        .unwrap();
        let lid = Jid::parse("67890@lid").unwrap();
        let phone = Jid::parse("15555550888@s.whatsapp.net").unwrap();
        let foreign = Jid::parse("99999@lid").unwrap();
        let outbox = crate::outbox::Outbox::new("fixture", crate::profile::Limits::default());
        for (index, (actual, original, authenticated, wire_reserved, matched)) in [
            (&lid, &lid, false, true, true),
            (&phone, &phone, false, true, true),
            (&lid, &phone, false, true, false),
            (&lid, &phone, true, true, true),
            (&phone, &lid, true, true, true),
            (&foreign, &phone, true, true, false),
            (&lid, &lid, false, false, false),
        ]
        .into_iter()
        .enumerate()
        {
            let id = format!("ECHO-{index}");
            let request = SendRequest {
                client_request_id: ClientRequestId::new(id.clone()).unwrap(),
                chat: ChatId::from_jid(original).unwrap(),
                content: MessageContent::Text {
                    body: "original".into(),
                    quoted: None,
                    style: Default::default(),
                },
            };
            let (ticket, _) = store.write(|tx| outbox.enqueue(tx, &request)).unwrap();
            store
                .write(|tx| {
                    let map_key = format!("authenticated-lid:{lid}");
                    tx.delete(Table::Checkpoints, map_key.as_bytes())?;
                    if authenticated {
                        tx.put_json(Table::Checkpoints, map_key.as_bytes(), 1, &phone)?;
                    }
                    // A directory alias alone must not supply this association.
                    tx.put_json(
                        Table::Checkpoints,
                        format!("lid-for:{phone}").as_bytes(),
                        1,
                        &lid,
                    )?;
                    let mut entry = outbox.get_mut(tx, ticket.local_id)?.unwrap();
                    entry.wire_id = wire_reserved.then(|| MessageId::new(id.clone()).unwrap());
                    tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)
                })
                .unwrap();
            let before = store
                .read(|tx| {
                    Ok((
                        tx.scan(Table::Outbox, 1, 20)?,
                        tx.scan(Table::Idempotency, 1, 20)?,
                    ))
                })
                .unwrap();
            let head = store.journal_head().unwrap();
            let node = Node::new("message").attr("id", id.as_str());
            let key = format!("received-own:{actual}:{id}");
            store
                .write(|tx| {
                    let event = super::super::inbox_events::own_device_message(
                        tx,
                        key.as_bytes(),
                        actual,
                        &node,
                        b"\x0a\x04echo",
                        &[],
                    )?
                    .unwrap();
                    publish(tx, key.as_bytes(), event)
                })
                .unwrap();
            assert_eq!(
                store
                    .read(|tx| Ok((
                        tx.scan(Table::Outbox, 1, 20)?,
                        tx.scan(Table::Idempotency, 1, 20)?
                    )))
                    .unwrap(),
                before
            );
            assert_eq!(store.journal_head().unwrap() == head, matched);
            let marker: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Inbox, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(
                marker["kind"],
                if matched {
                    "outgoing_echo"
                } else {
                    "new_message"
                }
            );
            if matched {
                assert_eq!(marker["local_id"], serde_json::json!(ticket.local_id));
            }
        }
        let key = b"received-own:67890@lid:CORRUPT";
        store
            .write(|tx| tx.put(Table::Outbox, b"damaged", 1, b"invalid json"))
            .unwrap();
        let before_head = store.journal_head().unwrap();
        let error = store
            .write(|tx| {
                tx.put(Table::SignalSessions, b"rollback-fixture", 1, b"advanced")?;
                let event = super::super::inbox_events::own_device_message(
                    tx,
                    key,
                    &lid,
                    &Node::new("message").attr("id", "CORRUPT"),
                    b"\x0a\x04echo",
                    &[],
                )?
                .unwrap();
                publish(tx, key, event)
            })
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput {
                field: "outbox",
                ..
            }
        ));
        assert!(
            store
                .read(|tx| tx.get(Table::SignalSessions, b"rollback-fixture", 1))
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Inbox, key, 1))
                .unwrap()
                .is_none()
        );
        assert_eq!(store.journal_head().unwrap(), before_head);
    }

    #[test]
    fn confirmation_requires_committed_same_source_and_keeps_saved_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(crate::store::OpenOptions {
            path: dir.path().join("own-receipt.redb"),
            account_id: crate::model::ids::AccountId::new("fixture").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([9; 32]),
            )),
        })
        .unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            registration: crate::session::RegistrationStatus::Registered,
            ..Default::default()
        };
        let source = Jid::parse("15555550100:2@s.whatsapp.net").unwrap();
        let chat = Jid::parse("67890@lid").unwrap();
        let node = Node::new("message")
            .attr("from", source.clone())
            .attr("recipient", chat.clone())
            .attr("type", "text")
            .attr("id", "SYNC");
        assert!(confirmation(&store, &account, &node).unwrap().is_none());
        for (status, saved_source, accepted) in [
            ("pending", source.clone(), false),
            ("cleared", source.clone(), false),
            (
                "decrypted",
                Jid::parse("15555550100:3@s.whatsapp.net").unwrap(),
                false,
            ),
            ("decrypted", source.clone(), true),
        ] {
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        b"received-own:67890@lid:SYNC",
                        1,
                        &serde_json::json!({"status": status, "source": saved_source}),
                    )
                })
                .unwrap();
            let before = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10)).unwrap();
            let receipt = confirmation(&store, &account, &node).unwrap();
            assert_eq!(
                receipt,
                accepted.then(|| Node::new("receipt")
                    .attr("to", source.clone())
                    .attr("id", "SYNC")
                    .attr("type", "sender")
                    .attr("recipient", chat.clone()))
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Checkpoints, 1, 10)).unwrap(),
                before
            );
        }
        for changed in [
            node.clone().attr("from", "15555550999:2@s.whatsapp.net"),
            node.clone().attr("recipient", "67891@lid"),
            node.clone().attr("participant", source),
            node.clone().attr("category", "peer"),
            node.clone()
                .attr("peer_recipient_pn", "15555550888@s.whatsapp.net"),
        ] {
            assert!(confirmation(&store, &account, &changed).unwrap().is_none());
        }
        assert!(super::super::messaging::received_confirmation(&node).is_none());
    }

    #[test]
    fn own_route_requires_saved_user_companion_and_unambiguous_primary_recipient() {
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            registration: crate::session::RegistrationStatus::Registered,
            ..Default::default()
        };
        let lid = Jid::parse("12345@lid").unwrap();
        let node = Node::new("message")
            .attr("from", "12345:2@lid")
            .attr("type", "text")
            .attr("recipient", "67890@lid")
            .attr("id", "SYNC");
        assert!(route(&account, &node, Some(&lid)).is_some());
        assert!(route(&account, &node, None).is_none());
        assert!(
            route(
                &account,
                &node.clone().attr("from", "15555550100:2@s.whatsapp.net"),
                None
            )
            .is_some()
        );
        for source in [
            "99999:2@lid",
            "12345@lid",
            "12345_1:2@lid",
            "15555550101:2@s.whatsapp.net",
            "12345@g.us",
        ] {
            assert!(route(&account, &node.clone().attr("from", source), Some(&lid)).is_none());
        }
        for recipient in [
            "",
            "67890:2@lid",
            "67890:0@lid",
            "67890.1:0@lid",
            "67890@g.us",
            "status@broadcast",
        ] {
            assert!(
                route(
                    &account,
                    &node.clone().attr("recipient", recipient),
                    Some(&lid)
                )
                .is_none()
            );
        }
        for name in [
            "participant",
            "peer_recipient_pn",
            "peer_recipient_lid",
            "peer_recipient_username",
            "recipient_latest_lid",
        ] {
            assert!(route(&account, &node.clone().attr(name, ""), Some(&lid)).is_none());
        }
        for edit in ["", "7", "+007", "1", "2", "3"] {
            assert!(route(&account, &node.clone().attr("edit", edit), Some(&lid)).is_some());
        }
        for edit in ["8", "0", "4", " 7", "2147483655"] {
            assert!(route(&account, &node.clone().attr("edit", edit), Some(&lid)).is_none());
        }
        assert!(route(&account, &node.attr("category", "peer"), Some(&lid)).is_none());
    }
    #[test]
    fn synchronized_poll_keeps_ordinary_body_and_refuses_special_forms_and_mixed_content() {
        let chat = Jid::parse("67890@lid").unwrap();
        let wrap = |inner: &[u8]| {
            let mut sent = Vec::new();
            prost::encoding::string::encode(1, &chat.to_string(), &mut sent);
            prost::encoding::bytes::encode(2, &inner.to_vec(), &mut sent);
            let mut outer = Vec::new();
            prost::encoding::bytes::encode(31, &sent, &mut outer);
            outer
        };
        let ordinary =
            super::super::polls::creation_payload("Q", &["a".into(), "b".into()], 1, [8; 32])
                .unwrap();
        assert_eq!(
            payload(&wrap(&ordinary), &chat).unwrap().as_slice(),
            ordinary
        );
        for field in 6..=11 {
            let mut body = b"\x12\x01Q".to_vec();
            prost::encoding::uint32::encode(field, &0, &mut body);
            let mut inner = Vec::new();
            prost::encoding::bytes::encode(64, &body, &mut inner);
            assert!(
                payload(&wrap(&inner), &chat).is_none(),
                "special field {field}"
            );
        }
        assert!(payload(&wrap(b"\x92\x03\x00"), &chat).is_some()); // Vote validation follows unwrapping.
        assert!(payload(&wrap(&[ordinary.as_slice(), b"\x0a\x01X"].concat()), &chat).is_none());
    }

    #[test]
    fn wrapper_accepts_legacy_timer_protocols_without_admitting_unported_controls() {
        let chat = Jid::parse("67890@lid").unwrap();
        let wrap = |inner: &[u8]| {
            let mut sent = Vec::new();
            prost::encoding::string::encode(1, &chat.to_string(), &mut sent);
            prost::encoding::bytes::encode(2, &inner.to_vec(), &mut sent);
            let mut out = Vec::new();
            prost::encoding::bytes::encode(31, &sent, &mut out);
            out
        };
        for inner in [
            b"\x62\x04\x10\x03\x20\x3c".as_slice(),
            b"\x62\x04\x10\x04\x20\x00",
        ] {
            assert_eq!(payload(&wrap(inner), &chat).unwrap().as_slice(), inner);
        }
        for inner in [
            b"\x62\x02\x10\x00".to_vec(),
            b"\x62\x07\x10\x03\x20\x3c\xe8\x01\x01".to_vec(),
            b"\x62\x04\x10\x03\x20\x3c\x0a\x01X".to_vec(),
            super::super::chat_settings::limit_sharing_payload(true, 0),
        ] {
            assert!(payload(&wrap(&inner), &chat).is_none());
        }
    }

    #[test]
    fn wrapper_requires_exact_destination_text_and_retains_outer_context() {
        let chat = Jid::parse("67890@lid").unwrap();
        let wire = b"\xfa\x01\x14\x0a\x0967890@lid\x12\x07\x0a\x05hello";
        assert_eq!(payload(wire, &chat).unwrap().as_slice(), b"\x0a\x05hello");
        assert!(payload(wire, &Jid::parse("67891@lid").unwrap()).is_none());
        for invalid in [
            b"\xfa\x01\x00".as_slice(),
            b"\x0a\x05hello",
            b"\xfa\x01\x01\x12",
            b"\xfa\x01\x10\x0a\x0967890@lid\x12\x03\x62\x01\x00",
        ] {
            assert!(payload(invalid, &chat).is_none());
        }
        let mut with_context = wire.to_vec();
        with_context.extend(b"\x9a\x02\x02\x08\x02");
        assert_eq!(
            payload(&with_context, &chat).unwrap().as_slice(),
            b"\x0a\x05hello\x9a\x02\x02\x08\x02"
        );
        let old_context = b"\xfa\x01\x19\x0a\x0967890@lid\x12\x0c\x0a\x05hello\x9a\x02\x02\x08\x01";
        let mut replacement = old_context.to_vec();
        replacement.extend(b"\x9a\x02\x02\x08\x02");
        assert_eq!(
            payload(&replacement, &chat).unwrap().as_slice(),
            b"\x0a\x05hello\x9a\x02\x02\x08\x02"
        );
        let mut malformed_context = wire.to_vec();
        malformed_context.extend(b"\x9a\x02\x01\x08");
        assert!(payload(&malformed_context, &chat).is_none());
    }
}
