//! Projection into the normal durable event journal, never a second ratchet.
//! Ordinary supported media projects an account-bound sealed selector. Unsupported
//! payloads keep opaque placeholders; no raw media/view-once keys enter the journal.
use crate::{
    Result,
    binary::Node,
    model::{
        ids::{ChatId, DeviceAddress, MessageId},
        message::{
            Direction, Ephemerality, Message, MessageContent, MessageKey, UnsupportedMessage,
        },
        time::Timestamp,
    },
    store::{Table, WriteTx},
    sync::SyncEvent,
};
use prost::Message as _;
use sha2::{Digest, Sha256};

#[derive(prost::Message)]
struct Envelope {
    #[prost(string, optional, tag = "1")]
    conversation: Option<String>,
    #[prost(message, optional, tag = "6")]
    extended: Option<Extended>,
}
#[derive(prost::Message)]
struct Extended {
    #[prost(string, optional, tag = "1")]
    text: Option<String>,
    #[prost(bytes = "vec", optional, tag = "17")]
    context: Option<Vec<u8>>,
}
// Comparison LX/BWx.dynamicMethod: stanzaId=1, quotedMessage=3, expiration=25.
#[derive(prost::Message)]
struct Context {
    #[prost(string, optional, tag = "1")]
    stanza_id: Option<String>,
    #[prost(bytes = "vec", optional, tag = "3")]
    quoted: Option<Vec<u8>>,
    #[prost(uint32, optional, tag = "25")]
    expiration: Option<u32>,
}

fn event(key: &[u8], node: &Node, payload: &[u8]) -> SyncEvent {
    event_with_content(key, node, payload, None)
}

fn event_with_content(
    key: &[u8],
    node: &Node,
    payload: &[u8],
    content: Option<(MessageContent, Option<u32>)>,
) -> SyncEvent {
    let unsupported = || {
        SyncEvent::UnsupportedMessage(UnsupportedMessage {
            type_hint: format!("comparison-inbox:{}", hex::encode(Sha256::digest(key))),
            raw: bytes::Bytes::new(),
            truncated: true,
        })
    };
    let (content, expiration_seconds) = if let Some(content) = content {
        content
    } else {
        let Ok(fields) = crate::signal::direct::payload_fields(payload) else {
            return unsupported();
        };
        // Reaction (comparison 2.26.37.71 `Message.reactionMessage` 46): a change
        // to an existing message, never a new message event.
        if fields.contains(&46) {
            if fields.iter().any(|v| !matches!(v, 46 | 35)) {
                return unsupported();
            }
            let Some(reaction) = super::reactions::decode(payload) else {
                return unsupported();
            };
            let Some(sender) = super::inbox::session_sender(node) else {
                return unsupported();
            };
            let (Ok(chat), Ok(by)) = (ChatId::from_jid(&sender), DeviceAddress::from_jid(&sender))
            else {
                return unsupported();
            };
            let server_timestamp = node
                .get_attr("t")
                .and_then(|v| v.as_text())
                .and_then(|v| v.parse::<i64>().ok())
                .filter(|v| *v >= 0)
                .and_then(|v| v.checked_mul(1000))
                .map(Timestamp);
            // `fromMe` is the reacting sender's perspective: true means the target
            // was their own (incoming to us), false means the target was ours.
            let (direction, target_sender) = if reaction.target_from_sender {
                (Direction::Incoming, Some(by.clone()))
            } else {
                (Direction::Outgoing, None)
            };
            return SyncEvent::MessageUpdated(crate::model::message::MessageUpdate {
                key: MessageKey {
                    chat,
                    sender: target_sender,
                    direction,
                    id: reaction.target_id,
                },
                change: crate::model::message::MessageChange::Reaction {
                    by,
                    emoji: reaction.emoji,
                },
                server_timestamp,
            });
        }
        // Only known top-level text and metadata fields. Wrappers, protocol messages
        // and mixed media must never become an ordinary text event accidentally.
        if fields.iter().any(|v| !matches!(v, 1 | 6 | 35))
            || fields.iter().filter(|v| matches!(v, 1 | 6)).count() != 1
        {
            return unsupported();
        }
        let Ok(envelope) = Envelope::decode(payload) else {
            return unsupported();
        };
        let (body, expiration_seconds) = match (envelope.conversation, envelope.extended) {
            (Some(text), None) => (text, None),
            (
                None,
                Some(Extended {
                    text: Some(text),
                    context,
                }),
            ) => {
                let mut expiration = None;
                if let Some(context) = context {
                    let Ok(context) = Context::decode(context.as_slice()) else {
                        return unsupported();
                    };
                    if context.stanza_id.is_some() || context.quoted.is_some() {
                        return unsupported();
                    }
                    expiration = context.expiration.filter(|v| *v != 0);
                }
                (text, expiration)
            }
            _ => return unsupported(),
        };
        (
            MessageContent::Text { body, quoted: None },
            expiration_seconds,
        )
    };
    let Some(sender) = super::inbox::session_sender(node) else {
        return unsupported();
    };
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .and_then(|v| MessageId::new(v).ok())
    else {
        return unsupported();
    };
    let (Ok(chat), Ok(sender)) = (ChatId::from_jid(&sender), DeviceAddress::from_jid(&sender))
    else {
        return unsupported();
    };
    let server_timestamp = node
        .get_attr("t")
        .and_then(|v| v.as_text())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .and_then(|v| v.checked_mul(1000))
        .map(Timestamp);
    if let Some(duration) = expiration_seconds {
        // Never expose an already-expired historical message as fresh text.
        let Some(sent) = server_timestamp else {
            return unsupported();
        };
        if sent.0.saturating_add(i64::from(duration) * 1000) <= Timestamp::now().0 {
            return unsupported();
        }
    }
    SyncEvent::NewMessage(Message {
        key: MessageKey {
            chat,
            sender: Some(sender),
            direction: Direction::Incoming,
            id,
        },
        server_timestamp,
        observed_at: Timestamp::now(),
        content,
        ephemerality: Ephemerality {
            expiration_seconds,
            view_once: false,
        },
    })
}

/// Must be called in the ratchet/inbox write transaction, or on an already
/// authenticated historical record using its original saved route. Marker and
/// event commit together, so crashes cannot lose the event or emit it twice.
pub(super) fn publish(
    tx: &mut WriteTx<'_>,
    key: &[u8],
    node: &Node,
    payload: &[u8],
) -> Result<bool> {
    let projected = event(key, node, payload);
    #[cfg(feature = "experimental-registration-https")]
    let projected = if matches!(projected, SyncEvent::UnsupportedMessage(_)) {
        match super::media::event_content(tx, key, payload) {
            Ok(Some(content)) => event_with_content(key, node, payload, Some(content)),
            // Invalid/unsupported media stays an opaque event; it must not block
            // durable storage and acknowledgement of an authenticated message.
            Ok(None) | Err(crate::Error::InvalidInput { .. }) => projected,
            Err(error) => return Err(error),
        }
    } else {
        projected
    };
    if let Some(marker) = tx.get_json::<serde_json::Value>(Table::Inbox, key, 1)? {
        if !matches!(
            projected,
            SyncEvent::NewMessage(_) | SyncEvent::MessageUpdated(_)
        ) {
            return Ok(false);
        }
        let was_unsupported = if let Some(kind) = marker["kind"].as_str() {
            kind == "unsupported_message"
        } else {
            // Older projection markers did not retain kind. Only upgrade when
            // the actual journal proves this was an unsupported placeholder.
            let batch = tx.read_events(
                &crate::store::ConsumerId::new("projection-migration")?,
                crate::model::ids::EventCursor::START,
                4096,
            )?;
            batch.events.iter().any(|e| {
                serde_json::to_value(e.event_id).ok().as_ref() == Some(&marker["event_id"])
                    && matches!(e.event, SyncEvent::UnsupportedMessage(_))
            })
        };
        if !was_unsupported {
            return Ok(false);
        }
    }
    let envelope = tx.append_event(projected)?;
    tx.put_json(
        Table::Inbox,
        key,
        1,
        &serde_json::json!({"event_id":envelope.event_id,"kind":envelope.event.kind()}),
    )?;
    Ok(true)
}

/// Backfill only from a previously authenticated record's original capture.
/// Never trust a redelivery's replacement sender mapping or copy consumed media.
pub(super) fn restore(
    store: &crate::store::ProtocolStore,
    key: &[u8],
    saved: &serde_json::Value,
) -> Result<bool> {
    if saved["status"] != "decrypted" {
        return Ok(false);
    }
    let wire: Option<Vec<u8>> = if !saved["route"].is_null() {
        serde_json::from_value(saved["route"].clone()).ok()
    } else if let Ok(capture) = serde_json::from_value::<Vec<u8>>(saved["capture"].clone()) {
        store.read(|tx| tx.get_json(Table::Checkpoints, &capture, 1))?
    } else {
        None
    };
    let Some(wire) = wire else {
        return Ok(false);
    };
    let limits = crate::profile::Limits::default();
    let raw = crate::binary::unpack_frame_payload(&wire, &limits)?;
    let node = crate::binary::Decoder::new(
        &raw,
        crate::binary::TokenDictionary::comparison_2_26_37_70(),
        &limits,
    )
    .decode_node()?;
    let Some(sender) = super::inbox::session_sender(&node) else {
        return Ok(false);
    };
    let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(false);
    };
    if node.description != "message" || format!("received:{sender}:{id}").as_bytes() != key {
        return Err(crate::Error::invalid(
            "inbox",
            "saved capture does not match authenticated record",
        ));
    }
    let payload: zeroize::Zeroizing<Vec<u8>> = serde_json::from_value(saved["protobuf"].clone())
        .map_err(|_| crate::Error::invalid("inbox", "invalid saved authenticated payload"))?;
    store.write(|tx| {
        let current: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, key, 1)?;
        if current.as_ref() != Some(saved) {
            return Err(crate::Error::invalid(
                "inbox",
                "record changed during event backfill",
            ));
        }
        if let Some(from) = super::receipts::jid(&node, "from")
            && from.server == crate::model::jid::Server::Lid
        {
            let mapping = format!("authenticated-lid:{from}");
            let old: Option<crate::model::jid::Jid> =
                tx.get_json(Table::Checkpoints, mapping.as_bytes(), 1)?;
            if old.as_ref().is_some_and(|v| v != &sender) {
                return Err(crate::Error::invalid(
                    "inbox",
                    "authenticated LID mapping changed",
                ));
            }
            tx.put_json(Table::Checkpoints, mapping.as_bytes(), 1, &sender)?;
        }
        publish(tx, key, &node, &payload)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Error,
        model::ids::{AccountId, EventCursor},
        store::{ConsumerId, MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };
    fn node() -> Node {
        Node::new("message")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "fixture")
            .attr("t", "123")
    }
    #[test]
    fn text_projection_and_media_never_leak_raw_payloads() {
        let event = event(b"fixture", &node(), b"\x0a\x04test");
        let SyncEvent::NewMessage(message) = event else {
            panic!("text missing");
        };
        assert_eq!(
            message.content,
            MessageContent::Text {
                body: "test".into(),
                quoted: None
            }
        );
        assert_eq!(message.server_timestamp, Some(Timestamp(123000)));
        assert!(matches!(
            super::event(
                b"fixture",
                &node(),
                b"\x32\x0c\x0a\x04test\x8a\x01\x03\xc8\x01\x00"
            ),
            SyncEvent::NewMessage(_)
        ));
        assert!(matches!(
            super::event(
                b"fixture",
                &node(),
                b"\x32\x0c\x0a\x04test\x8a\x01\x03\xc8\x01\x01"
            ),
            SyncEvent::UnsupportedMessage(_)
        ));
        // Message.image (3), protocol (12), viewOnce wrapper (37), mixed text.
        for payload in [
            b"\x1a\x04keys".as_slice(),
            b"\x62\x02\x10\x04",
            b"\xaa\x02\x04keys",
            b"\x0a\x04test\x1a\x04keys",
        ] {
            let SyncEvent::UnsupportedMessage(value) = super::event(b"fixture", &node(), payload)
            else {
                panic!("unsupported became user text");
            };
            assert!(value.raw.is_empty());
            assert!(!value.type_hint.contains("keys"));
        }
    }
    #[test]
    fn reactions_project_as_message_updates_relative_to_the_reacting_sender() {
        use crate::model::{
            ids::{ChatId, DeviceAddress, MessageId},
            jid::Jid,
            message::{Direction, MessageChange, MessageKey, MessageRef},
        };
        let sender = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let target = |direction| MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(&sender).unwrap(),
                sender: None,
                direction,
                id: MessageId::new("71C7E1A34E0332A8821B812F2E028952").unwrap(),
            },
        };
        // The sender reacts to a message that was ours (their fromMe=false).
        let theirs_on_ours = super::super::reactions::payload(
            &sender,
            &sender,
            &target(Direction::Incoming),
            Some("👍"),
            5,
        )
        .unwrap();
        let SyncEvent::MessageUpdated(update) = event(b"fixture", &node(), &theirs_on_ours) else {
            panic!("reaction did not project");
        };
        assert_eq!(update.key.direction, Direction::Outgoing);
        assert_eq!(update.key.sender, None);
        assert_eq!(update.key.id.as_str(), "71C7E1A34E0332A8821B812F2E028952");
        assert_eq!(update.server_timestamp, Some(Timestamp(123000)));
        assert_eq!(
            update.change,
            MessageChange::Reaction {
                by: DeviceAddress::from_jid(&sender).unwrap(),
                emoji: Some("👍".into())
            }
        );
        // Removing a reaction on their own message (their fromMe=true).
        let removal = super::super::reactions::payload(
            &sender,
            &sender,
            &target(Direction::Outgoing),
            None,
            5,
        )
        .unwrap();
        let SyncEvent::MessageUpdated(update) = event(b"fixture", &node(), &removal) else {
            panic!("removal did not project");
        };
        assert_eq!(update.key.direction, Direction::Incoming);
        assert_eq!(
            update.key.sender,
            Some(DeviceAddress::from_jid(&sender).unwrap())
        );
        assert!(matches!(
            update.change,
            MessageChange::Reaction { emoji: None, .. }
        ));
        // A reaction mixed with text stays opaque.
        let mut mixed = theirs_on_ours.clone();
        mixed.extend_from_slice(b"\x0a\x04test");
        assert!(matches!(
            event(b"fixture", &node(), &mixed),
            SyncEvent::UnsupportedMessage(_)
        ));
    }
    #[test]
    fn event_and_marker_rollback_together_and_survive_restart_once() {
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("events.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let result: Result<()> = store.write(|tx| {
            publish(tx, b"fixture", &node(), b"\x0a\x04test")?;
            Err(Error::Cancelled)
        });
        assert!(result.is_err());
        let consumer = ConsumerId::new("test").unwrap();
        assert!(
            store
                .read_events(&consumer, EventCursor::START, 10)
                .unwrap()
                .events
                .is_empty()
        );
        assert!(
            store
                .write(|tx| publish(tx, b"fixture", &node(), b"\x0a\x04test"))
                .unwrap()
        );
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert!(
            !store
                .write(|tx| publish(tx, b"fixture", &node(), b"\x0a\x04test"))
                .unwrap()
        );
        assert_eq!(
            store
                .read_events(&consumer, EventCursor::START, 10)
                .unwrap()
                .events
                .len(),
            1
        );
        store
            .write(|tx| {
                publish(tx, b"upgrade", &node(), b"\x62\x02\x10\x04")?;
                let mut marker: serde_json::Value =
                    tx.get_json(Table::Inbox, b"upgrade", 1)?.unwrap();
                marker.as_object_mut().unwrap().remove("kind");
                tx.put_json(Table::Inbox, b"upgrade", 1, &marker)
            })
            .unwrap();
        assert!(
            store
                .write(|tx| publish(tx, b"upgrade", &node(), b"\x0a\x04test"))
                .unwrap()
        );
        assert!(
            !store
                .write(|tx| publish(tx, b"upgrade", &node(), b"\x0a\x04test"))
                .unwrap()
        );
    }
    #[test]
    fn historical_backfill_uses_original_route_and_never_consumed_payloads() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("history.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([8; 32]))),
            create: true,
        })
        .unwrap();
        let key = b"received:15555550100@s.whatsapp.net:fixture";
        let original = Node::new("message")
            .attr("from", "12345@lid")
            .attr("sender_pn", "15555550100@s.whatsapp.net")
            .attr("id", "fixture");
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70())
                .encode(&original)
                .unwrap();
        let saved = serde_json::json!({"status":"decrypted","route":route,"protobuf":b"\x0a\x04test".to_vec()});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key, 1, &saved))
            .unwrap();
        assert!(restore(&store, key, &saved).unwrap());
        assert!(!restore(&store, key, &saved).unwrap());
        assert!(
            restore(
                &store,
                b"received:15555550101@s.whatsapp.net:fixture",
                &saved
            )
            .is_err()
        );
        let mapping: crate::model::jid::Jid = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"authenticated-lid:12345@lid", 1))
            .unwrap()
            .unwrap();
        assert_eq!(mapping.to_string(), "15555550100@s.whatsapp.net");
        let consumed = serde_json::json!({"status":"consumed","route":route});
        assert!(!restore(&store, key, &consumed).unwrap());
        let consumer = ConsumerId::new("test").unwrap();
        assert_eq!(
            store
                .read_events(&consumer, EventCursor::START, 10)
                .unwrap()
                .events
                .len(),
            1
        );
    }
}
