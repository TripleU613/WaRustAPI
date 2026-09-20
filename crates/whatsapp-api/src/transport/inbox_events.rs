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
use sha2::{Digest, Sha256};

#[cfg(test)]
fn event(key: &[u8], node: &Node, payload: &[u8]) -> SyncEvent {
    project(key, node, payload, None, None, &[])
}

fn poll_content(poll: super::polls::Poll) -> Option<MessageContent> {
    if poll.options.len() > super::polls::MAX_OPTIONS
        || poll
            .options
            .iter()
            .chain(std::iter::once(&poll.name))
            .any(|t| t.len() > super::polls::MAX_TEXT_BYTES || t.chars().any(char::is_control))
    {
        return None;
    }
    Some(MessageContent::Poll {
        name: poll.name,
        options: poll.options,
        selectable_count: poll.selectable_count,
    })
}

/// What the projection needs to decrypt a vote on one poll.
#[derive(Clone, Debug)]
pub(super) struct PollContext {
    pub(super) poll_id: String,
    pub(super) secret: [u8; 32],
    pub(super) options: Vec<String>,
    /// Candidate creator identities (phone-number and LID forms).
    pub(super) creators: Vec<crate::model::jid::Jid>,
}
/// Poll secret and options for the poll a vote refers to, from this store:
/// a poll we created (`poll-secret:<id>`) or one received from the sender.
pub(super) fn poll_context(
    tx: &WriteTx<'_>,
    node: &Node,
    payload: &[u8],
) -> Result<Option<PollContext>> {
    let Some(vote) = super::polls::decode_vote(payload) else {
        return Ok(None);
    };
    let Some(sender) = super::inbox::session_sender(node) else {
        return Ok(None);
    };
    let poll_id = vote.poll_id.as_str().to_owned();
    if vote.poll_from_sender {
        // The sender's own poll: its authenticated plaintext holds the secret.
        let key = format!("received:{sender}:{poll_id}");
        let Some(saved) =
            tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?
        else {
            return Ok(None);
        };
        let Ok(plain) = serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()) else {
            return Ok(None);
        };
        let Some(poll) = super::polls::decode_poll(&plain) else {
            return Ok(None);
        };
        let Some(secret) = poll.secret else {
            return Ok(None);
        };
        let mut creators = vec![sender.to_non_ad()];
        if let Some(lid) = super::receipts::jid(node, "from")
            .filter(|j| j.server == crate::model::jid::Server::Lid)
        {
            creators.push(lid.to_non_ad());
        }
        if let Some(lid) = super::inbox::lid_for_in(tx, &sender)?
            && !creators.contains(&lid)
        {
            creators.push(lid);
        }
        return Ok(Some(PollContext {
            poll_id,
            secret,
            options: poll.options,
            creators,
        }));
    }
    // Our poll: the secret was persisted when the creation was reserved.
    let Some(own): Option<serde_json::Value> = tx.get_json(
        Table::Checkpoints,
        format!("poll-secret:{poll_id}").as_bytes(),
        1,
    )?
    else {
        return Ok(None);
    };
    let Ok(secret) = serde_json::from_value::<Vec<u8>>(own["secret"].clone()) else {
        return Ok(None);
    };
    let Ok(secret) = <[u8; 32]>::try_from(secret) else {
        return Ok(None);
    };
    let options: Vec<String> = serde_json::from_value(own["options"].clone()).unwrap_or_default();
    let mut creators = Vec::new();
    for field in ["creator_pn", "creator_lid"] {
        if let Some(j) = own[field].as_str()
            && let Ok(j) = crate::model::jid::Jid::parse(j)
        {
            creators.push(j.to_non_ad());
        }
    }
    Ok(Some(PollContext {
        poll_id,
        secret,
        options,
        creators,
    }))
}
fn poll_vote_event(node: &Node, payload: &[u8], context: &PollContext) -> Option<SyncEvent> {
    let vote = super::polls::decode_vote(payload)?;
    let sender = super::inbox::session_sender(node)?;
    let (chat, by) = (
        ChatId::from_jid(&sender).ok()?,
        DeviceAddress::from_jid(&sender).ok()?,
    );
    // The voter is the sender, addressed by phone number or LID; try each
    // creator/voter pairing until the authenticated decryption succeeds.
    let mut voters = vec![sender.to_non_ad()];
    if let Some(lid) =
        super::receipts::jid(node, "from").filter(|j| j.server == crate::model::jid::Server::Lid)
    {
        voters.push(lid.to_non_ad());
    }
    let hashes = context.creators.iter().find_map(|creator| {
        voters.iter().find_map(|voter| {
            super::polls::decrypt_vote(&context.secret, &context.poll_id, creator, voter, &vote)
                .ok()
        })
    })?;
    let (selected, unresolved) = super::polls::resolve_options(&hashes, &context.options);
    let server_timestamp = node
        .get_attr("t")
        .and_then(|v| v.as_text())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .and_then(|v| v.checked_mul(1000))
        .map(Timestamp);
    let (direction, target_sender) = if vote.poll_from_sender {
        (Direction::Incoming, Some(by.clone()))
    } else {
        (Direction::Outgoing, None)
    };
    Some(SyncEvent::MessageUpdated(
        crate::model::message::MessageUpdate {
            key: MessageKey {
                chat,
                sender: target_sender,
                direction,
                id: vote.poll_id,
            },
            change: crate::model::message::MessageChange::PollVote {
                by,
                selected,
                unresolved,
            },
            server_timestamp,
        },
    ))
}

fn event_with_content(
    key: &[u8],
    node: &Node,
    payload: &[u8],
    content: Option<(MessageContent, Option<u32>)>,
    own: &[crate::model::jid::Jid],
) -> SyncEvent {
    project(key, node, payload, content, None, own)
}

/// `own` lists this account's identities (phone-number and LID) so a quoted
/// reply's `contextInfo.participant` can be attributed to us or to the sender.
fn project(
    key: &[u8],
    node: &Node,
    payload: &[u8],
    content: Option<(MessageContent, Option<u32>)>,
    poll: Option<&PollContext>,
    own: &[crate::model::jid::Jid],
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
        // Chat-setting protocol message (comparison 2.26.37.71
        // `Message.protocolMessage` field 12; docs/protocol/chat-actions.md §3,
        // §5): a disappearing-timer change, its peer echo, or advanced chat
        // privacy. Never a reply/media wrapper, so this must be the only field
        // besides the optional `messageContextInfo`.
        if fields.contains(&12) {
            if fields.iter().any(|v| !matches!(v, 12 | 35)) {
                return unsupported();
            }
            let Some(setting) = super::chat_settings::decode_protocol(payload) else {
                return unsupported();
            };
            // `EPHEMERAL_SETTING` and `EPHEMERAL_SYNC_RESPONSE` both carry the
            // `ephemeralExpiration` field per the `LX/BZb` descriptor, so both
            // project as the same disappearing-timer content (chat-actions.md
            // §3 "Implementation").
            let content = match setting {
                super::chat_settings::Setting::DisappearingTimer { seconds }
                | super::chat_settings::Setting::EphemeralSyncResponse { seconds } => {
                    MessageContent::DisappearingTimer { seconds }
                }
                super::chat_settings::Setting::LimitSharing { enabled }
                | super::chat_settings::Setting::Acp2 { enabled } => {
                    MessageContent::ChatPrivacy { limited: enabled }
                }
            };
            return event_with_content(key, node, payload, Some((content, None)), own);
        }
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
        // Poll creation (2.26.37.71 `pollCreationMessageV3` 64 with `messageContextInfo` 35).
        if fields.contains(&64) {
            if fields.iter().any(|v| !matches!(v, 64 | 35)) {
                return unsupported();
            }
            let Some(poll) = super::polls::decode_poll(payload) else {
                return unsupported();
            };
            let Some(content) = poll_content(poll) else {
                return unsupported();
            };
            return event_with_content(key, node, payload, Some((content, None)), own);
        }
        // Poll vote (`pollUpdateMessage` 50): a change to the poll, decrypted with
        // the poll's message secret when this store holds it.
        if fields.contains(&50) {
            return poll
                .and_then(|context| poll_vote_event(node, payload, context))
                .unwrap_or_else(unsupported);
        }
        // Keep in chat (comparison `Message.keepInChatMessage` field 51;
        // docs/protocol/chat-actions.md §4): a change to an existing message,
        // never a new message event.
        if fields.contains(&51) {
            if fields.iter().any(|v| !matches!(v, 51 | 35)) {
                return unsupported();
            }
            let Some(keep) = super::chat_settings::decode_keep(payload) else {
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
            // `fromMe` is the keeping sender's perspective, same as reactions:
            // true means the target was their own (incoming to us), false
            // means the target was ours.
            let (direction, target_sender) = if keep.target_from_sender {
                (Direction::Incoming, Some(by.clone()))
            } else {
                (Direction::Outgoing, None)
            };
            return SyncEvent::MessageUpdated(crate::model::message::MessageUpdate {
                key: MessageKey {
                    chat,
                    sender: target_sender,
                    direction,
                    id: keep.target_id,
                },
                change: crate::model::message::MessageChange::KeepInChat {
                    by,
                    kept: keep.keep,
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
        let Some(text) = super::quotes::decode(payload) else {
            return unsupported();
        };
        // A quoted reply (comparison 2.26.37.71 `contextInfo` stanzaId=1,
        // participant=2, quotedMessage=3, observed live). `participant` names the
        // quoted message's sender: one of our identities means it was ours,
        // anything else in a direct chat means it was the sender's own.
        let quoted = match (&text.stanza_id, &text.quoted) {
            (None, None) => None,
            (Some(stanza_id), _) => {
                let Ok(id) = MessageId::new(stanza_id.clone()) else {
                    return unsupported();
                };
                let Some(sender) = super::inbox::session_sender(node) else {
                    return unsupported();
                };
                let (Ok(chat), Ok(by)) =
                    (ChatId::from_jid(&sender), DeviceAddress::from_jid(&sender))
                else {
                    return unsupported();
                };
                let participant = text
                    .participant
                    .as_deref()
                    .and_then(|p| crate::model::jid::Jid::parse(p).ok())
                    .map(|p| p.to_non_ad());
                let ours = participant
                    .as_ref()
                    .is_some_and(|p| own.iter().any(|o| o.to_non_ad() == *p));
                let (direction, sender) = if ours {
                    (Direction::Outgoing, None)
                } else {
                    (Direction::Incoming, Some(by))
                };
                Some(crate::model::message::MessageRef {
                    key: MessageKey {
                        chat,
                        sender,
                        direction,
                        id,
                    },
                })
            }
            (None, Some(_)) => return unsupported(),
        };
        (
            MessageContent::Text {
                body: text.text,
                quoted,
            },
            text.expiration,
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
    let poll = poll_context(tx, node, payload)?;
    let own = super::inbox::own_identities_in(tx)?;
    let projected = project(key, node, payload, None, poll.as_ref(), &own);
    #[cfg(feature = "experimental-registration-https")]
    let projected = if matches!(projected, SyncEvent::UnsupportedMessage(_)) {
        match super::media::event_content(tx, key, payload) {
            Ok(Some(content)) => event_with_content(key, node, payload, Some(content), &own),
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
        // Message.image (3), viewOnce wrapper (37), mixed text.
        for payload in [
            b"\x1a\x04keys".as_slice(),
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
        // protocolMessage{type=4 (EPHEMERAL_SYNC_RESPONSE)} carries the same
        // `ephemeralExpiration` field as type=3, so it now projects as a
        // disappearing-timer update instead of staying opaque (chat-actions.md
        // §3 "Implementation").
        let SyncEvent::NewMessage(message) = super::event(b"fixture", &node(), b"\x62\x02\x10\x04")
        else {
            panic!("ephemeral sync response did not project");
        };
        assert_eq!(
            message.content,
            MessageContent::DisappearingTimer { seconds: 0 }
        );
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
    fn disappearing_timer_and_chat_privacy_protocol_messages_project_as_new_messages() {
        let disappearing = super::super::chat_settings::disappearing_payload(604_800);
        let SyncEvent::NewMessage(message) = event(b"fixture", &node(), &disappearing) else {
            panic!("disappearing timer did not project");
        };
        assert_eq!(
            message.content,
            MessageContent::DisappearingTimer { seconds: 604_800 }
        );
        assert_eq!(message.server_timestamp, Some(Timestamp(123000)));
        let limit_on = super::super::chat_settings::limit_sharing_payload(true, 1_700_000_000_000);
        let SyncEvent::NewMessage(message) = event(b"fixture", &node(), &limit_on) else {
            panic!("limit sharing did not project");
        };
        assert_eq!(
            message.content,
            MessageContent::ChatPrivacy { limited: true }
        );
        // A chat-setting protocol message mixed with text stays opaque.
        let mut mixed = disappearing.clone();
        mixed.extend_from_slice(b"\x0a\x04test");
        assert!(matches!(
            event(b"fixture", &node(), &mixed),
            SyncEvent::UnsupportedMessage(_)
        ));
    }
    #[test]
    fn keep_in_chat_projects_as_message_updates_relative_to_the_acting_sender() {
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
        // The sender keeps a message that was ours (their fromMe=false).
        let theirs_on_ours = super::super::chat_settings::keep_payload(
            &sender,
            &target(Direction::Incoming),
            true,
            5,
        );
        let SyncEvent::MessageUpdated(update) = event(b"fixture", &node(), &theirs_on_ours) else {
            panic!("keep did not project");
        };
        assert_eq!(update.key.direction, Direction::Outgoing);
        assert_eq!(update.key.sender, None);
        assert_eq!(update.key.id.as_str(), "71C7E1A34E0332A8821B812F2E028952");
        assert_eq!(update.server_timestamp, Some(Timestamp(123000)));
        assert_eq!(
            update.change,
            MessageChange::KeepInChat {
                by: DeviceAddress::from_jid(&sender).unwrap(),
                kept: true,
            }
        );
        // Undoing a keep on their own message (their fromMe=true).
        let undo = super::super::chat_settings::keep_payload(
            &sender,
            &target(Direction::Outgoing),
            false,
            5,
        );
        let SyncEvent::MessageUpdated(update) = event(b"fixture", &node(), &undo) else {
            panic!("undo did not project");
        };
        assert_eq!(update.key.direction, Direction::Incoming);
        assert_eq!(
            update.key.sender,
            Some(DeviceAddress::from_jid(&sender).unwrap())
        );
        assert!(matches!(
            update.change,
            MessageChange::KeepInChat { kept: false, .. }
        ));
        // A keep-in-chat message mixed with text stays opaque.
        let mut mixed = theirs_on_ours.clone();
        mixed.extend_from_slice(b"\x0a\x04test");
        assert!(matches!(
            event(b"fixture", &node(), &mixed),
            SyncEvent::UnsupportedMessage(_)
        ));
    }
    #[test]
    fn polls_project_as_new_messages_and_votes_decrypt_with_the_saved_secret() {
        use crate::model::{jid::Jid, message::MessageChange};
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("polls.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap();
        // Their poll creation projects as a new message with poll content.
        let options = vec!["Yes".to_owned(), "No".to_owned()];
        let creation =
            super::super::polls::creation_payload("Lunch?", &options, 0, [5; 32]).unwrap();
        let SyncEvent::NewMessage(message) = event(b"fixture", &node(), &creation) else {
            panic!("poll did not project");
        };
        assert_eq!(
            message.content,
            MessageContent::Poll {
                name: "Lunch?".into(),
                options: options.clone(),
                selectable_count: 0
            }
        );
        // A vote on our own poll decrypts with the persisted secret and creator ids.
        let our_pn = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let voter = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let secret_bytes = vec![9u8; 32];
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"poll-secret:OURPOLL",
                    1,
                    &serde_json::json!({
                        "secret": secret_bytes,
                        "options": options,
                        "name": "Lunch?",
                        "creator_pn": our_pn.to_string(),
                    }),
                )
            })
            .unwrap();
        let enc = super::super::polls::encrypt_vote(
            &[9; 32],
            "OURPOLL",
            &our_pn,
            &voter,
            &options[1..],
            [1; 12],
        )
        .unwrap();
        let target = crate::model::message::MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(&voter).unwrap(),
                sender: None,
                direction: Direction::Incoming,
                id: MessageId::new("OURPOLL").unwrap(),
            },
        };
        // From the voter's perspective our poll is not theirs: fromMe=false.
        let payload = super::super::polls::vote_payload(&our_pn, &target, enc, 7);
        let vote_node = Node::new("message")
            .attr("from", voter.clone())
            .attr("id", "VOTE1")
            .attr("t", "123");
        assert!(
            store
                .write(|tx| publish(
                    tx,
                    b"received:15555550101@s.whatsapp.net:VOTE1",
                    &vote_node,
                    &payload
                ))
                .unwrap()
        );
        let consumer = ConsumerId::new("polls").unwrap();
        let events = store
            .read_events(&consumer, EventCursor::START, 10)
            .unwrap()
            .events;
        let SyncEvent::MessageUpdated(update) = &events.last().unwrap().event else {
            panic!(
                "vote did not project: {:?}",
                events.last().map(|e| e.event.kind())
            );
        };
        assert_eq!(update.key.id.as_str(), "OURPOLL");
        assert_eq!(update.key.direction, Direction::Outgoing);
        assert_eq!(
            update.change,
            MessageChange::PollVote {
                by: DeviceAddress::from_jid(&voter).unwrap(),
                selected: vec!["No".into()],
                unresolved: vec![]
            }
        );
        // A vote we cannot decrypt stays opaque rather than guessing.
        let enc = super::super::polls::encrypt_vote(
            &[8; 32],
            "OURPOLL",
            &our_pn,
            &voter,
            &options[..1],
            [2; 12],
        )
        .unwrap();
        let payload = super::super::polls::vote_payload(&our_pn, &target, enc, 8);
        let node2 = Node::new("message")
            .attr("from", voter.clone())
            .attr("id", "VOTE2");
        store
            .write(|tx| {
                publish(
                    tx,
                    b"received:15555550101@s.whatsapp.net:VOTE2",
                    &node2,
                    &payload,
                )
            })
            .unwrap();
        let events = store
            .read_events(&consumer, EventCursor::START, 10)
            .unwrap()
            .events;
        assert!(matches!(
            events.last().unwrap().event,
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
                // Message.image (3) raw bytes: an opaque placeholder, used only
                // to exercise the marker-upgrade path below.
                publish(tx, b"upgrade", &node(), b"\x1a\x04keys")?;
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

    #[test]
    fn quoted_reply_projects_text_with_the_quoted_reference() {
        use crate::model::{
            ids::{ChatId, DeviceAddress},
            message::Direction,
        };
        let own = crate::model::jid::Jid::parse("77700000001@lid").unwrap();
        let sender = crate::model::jid::Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let reply = |participant: &str| {
            super::super::quotes::reply_payload(
                "ok",
                "ABCDEF",
                participant,
                super::super::quotes::conversation("hi"),
            )
        };
        // Their reply to our message: participant is our LID.
        let SyncEvent::NewMessage(message) = project(
            b"fixture",
            &node(),
            &reply("77700000001@lid"),
            None,
            None,
            std::slice::from_ref(&own),
        ) else {
            panic!("expected message")
        };
        let MessageContent::Text { body, quoted } = message.content else {
            panic!("expected text")
        };
        assert_eq!(body, "ok");
        let quoted = quoted.expect("quoted reference");
        assert_eq!(quoted.key.direction, Direction::Outgoing);
        assert_eq!(quoted.key.sender, None);
        assert_eq!(quoted.key.id.as_str(), "ABCDEF");
        assert_eq!(quoted.key.chat, ChatId::from_jid(&sender).unwrap());
        // Their reply to their own message: participant is the sender.
        let SyncEvent::NewMessage(message) = project(
            b"fixture",
            &node(),
            &reply("15555550100@s.whatsapp.net"),
            None,
            None,
            &[own],
        ) else {
            panic!("expected message")
        };
        let MessageContent::Text { quoted, .. } = message.content else {
            panic!("expected text")
        };
        let quoted = quoted.expect("quoted reference");
        assert_eq!(quoted.key.direction, Direction::Incoming);
        assert_eq!(
            quoted.key.sender,
            Some(DeviceAddress::from_jid(&sender).unwrap())
        );
        // Plain text keeps no quote.
        let SyncEvent::NewMessage(message) = event(
            b"fixture",
            &node(),
            &super::super::quotes::conversation("plain"),
        ) else {
            panic!("expected message")
        };
        assert!(matches!(
            message.content,
            MessageContent::Text { quoted: None, .. }
        ));
    }
}
