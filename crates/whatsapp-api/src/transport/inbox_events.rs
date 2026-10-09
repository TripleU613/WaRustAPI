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

// APK 37.71 CnN.A02 root t -> CaF.A09 -> CgC.A01 00c2..00e0,
// 01c4..01d0 -> Cih.A0J: Java signed decimal and DEX wrapping ms.
// Preserve only explicit valid evidence here: native current-clock fallback
// must not make historical expiring content fresh. See message-timestamps.md.
pub(super) fn stanza_timestamp(node: &Node) -> Option<Timestamp> {
    node.get_attr("t")
        .and_then(|value| value.as_text())
        .and_then(super::mex::java_decimal_long)
        .map(|seconds| Timestamp(seconds.wrapping_mul(1000)))
}

#[cfg(test)]
fn event(key: &[u8], node: &Node, payload: &[u8]) -> SyncEvent {
    project(key, node, payload, None, None, None, &[])
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
    /// Original saved poll chat, including a retained legacy PN selector.
    pub(super) target_chat: Option<crate::model::jid::Jid>,
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
    if super::inbox::session_sender(node).is_none() {
        return Ok(None);
    }
    let poll_id = vote.poll_id.as_str().to_owned();
    if vote.poll_from_sender {
        // The sender's own poll: its authenticated plaintext holds the secret.
        let target = node.clone().attr("id", poll_id.as_str());
        let Some(record) =
            super::inbox::direct_record_in(&target, |key| tx.get_json(Table::Checkpoints, key, 1))?
        else {
            return Ok(None);
        };
        let Some(saved) = record.saved else {
            return Ok(None);
        };
        let sender = record.chat;
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
            target_chat: Some(sender),
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
    // APK 37.71 Ban.A01 -> Clc.A02 -> BoQ.A00 attributes the parent to
    // the envelope chat. Our saved chat binds the secret locally; directory
    // aliases and raw metadata alone cannot select another chat's poll.
    let Some(original) = super::polls::saved_chat(&own)? else {
        return Ok(None); // Legacy missing binding is unknown, not invented.
    };
    let Some(from) = super::receipts::jid(node, "from") else {
        return Ok(None);
    };
    if !authenticated_direct_chat(tx, &from, &original)? {
        return Ok(None);
    }
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
        target_chat: Some(original),
        secret,
        options,
        creators,
    }))
}
fn poll_vote_event(
    key: &[u8],
    node: &Node,
    payload: &[u8],
    context: &PollContext,
) -> Option<SyncEvent> {
    let vote = super::polls::decode_vote(payload)?;
    let sender = super::inbox::sender_for_record(key, node)?;
    let (chat, by) = (
        ChatId::from_jid(context.target_chat.as_ref().unwrap_or(&sender)).ok()?,
        DeviceAddress::from_jid(&sender).ok()?,
    );
    // The voter is the sender, addressed by phone number or LID; try each
    // creator/voter pairing until the authenticated decryption succeeds.
    let mut voters = vec![sender.to_non_ad()];
    if let Some(legacy) = context.target_chat.as_ref() {
        voters.push(legacy.to_non_ad());
    }
    if let Some(phone) = super::receipts::jid(node, "sender_pn").filter(|phone| {
        phone.server == crate::model::jid::Server::User && phone.agent == 0 && phone.device == 0
    }) {
        voters.push(phone.to_non_ad());
    }
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
    let server_timestamp = stanza_timestamp(node);
    let (direction, target_sender) = if vote.poll_from_sender {
        (
            Direction::Incoming,
            Some(DeviceAddress::from_jid(context.target_chat.as_ref().unwrap_or(&sender)).ok()?),
        )
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
    project(key, node, payload, content, None, None, own)
}

/// Chat and authenticated sender for a projection. A direct message's `from` is
/// both the chat and the sender; a group or broadcast (status) message's `from`
/// is the chat and its `participant` is the sender.
fn chat_and_sender(key: &[u8], node: &Node) -> Option<(ChatId, DeviceAddress)> {
    let (chat_jid, sender_jid) = match super::inbox::sender_key_chat(node) {
        Some(pair) => pair,
        None => {
            let chat = super::inbox::sender_for_record(key, node)?;
            let sender = super::receipts::jid(node, "from")
                .filter(|peer| peer.device != 0 && peer.agent == 0)
                .unwrap_or_else(|| chat.clone());
            (chat, sender)
        }
    };
    Some((
        ChatId::from_jid(&chat_jid).ok()?,
        DeviceAddress::from_jid(&sender_jid).ok()?,
    ))
}

/// `own` lists this account's identities (phone-number and LID) so a quoted
/// reply's `contextInfo.participant` can be attributed to us or to the sender.
fn project(
    key: &[u8],
    node: &Node,
    payload: &[u8],
    content: Option<(MessageContent, Option<u32>)>,
    poll: Option<&PollContext>,
    reference_target: Option<&MessageKey>,
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
        if fields.contains(&102) {
            // APK 2.26.37.71 DA8.CF4: an ordinary group-history notice needs
            // group context and meta/appdata=group_history, plus valid BX5
            // metadata. This event never substitutes notice text/history data.
            let Ok(Some(metadata)) = super::history_notice::decode_for_stanza(node, payload) else {
                return unsupported();
            };
            let Some((chat @ ChatId::Group(_), sender)) = chat_and_sender(key, node) else {
                return unsupported();
            };
            let Some(id) = node
                .get_attr("id")
                .and_then(|v| v.as_text())
                .and_then(|v| MessageId::new(v).ok())
            else {
                return unsupported();
            };
            return SyncEvent::GroupHistoryNotice {
                key: MessageKey {
                    chat,
                    sender: Some(sender),
                    direction: Direction::Incoming,
                    id,
                },
                metadata,
                server_timestamp: stanza_timestamp(node),
                observed_at: Timestamp::now(),
            };
        }
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
            // DeviceAddress cannot retain an agent component. Do not fold an
            // unsupported reactor into another user's/agent's reaction fact.
            if super::inbox::group_sender(node).is_some_and(|(_, p)| p.agent != 0) {
                return unsupported();
            }
            let Some((chat, by)) = chat_and_sender(key, node) else {
                return unsupported();
            };
            let server_timestamp = stanza_timestamp(node);
            // In a direct chat, fromMe distinguishes their message from ours.
            // Group authors are resolved separately using BZa.participant_.
            let (direction, target_sender) = if reaction.target_from_sender {
                (Direction::Incoming, Some(by.clone()))
            } else {
                (Direction::Outgoing, None)
            };
            let target = if chat.jid().server == crate::model::jid::Server::Group {
                let Ok(target) = super::reactions::group_target_for_sender(
                    chat.jid(),
                    &by.to_jid(),
                    payload,
                    own,
                ) else {
                    return unsupported();
                };
                target
            } else {
                MessageKey {
                    chat,
                    sender: target_sender,
                    direction,
                    id: reaction.target_id,
                }
            };
            return SyncEvent::MessageUpdated(crate::model::message::MessageUpdate {
                key: reference_target.cloned().unwrap_or(target),
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
                .and_then(|context| poll_vote_event(key, node, payload, context))
                .unwrap_or_else(unsupported);
        }
        // Keep in chat (comparison `Message.keepInChatMessage` field 51;
        // docs/protocol/chat-actions.md §4): a change to an existing message,
        // never a new message event.
        if fields.contains(&51) {
            if super::inbox::group_sender(node).is_some_and(|(_, actor)| actor.agent != 0) {
                return unsupported();
            }
            if fields.iter().any(|v| !matches!(v, 51 | 35)) {
                return unsupported();
            }
            let Some(keep) = super::chat_settings::decode_keep(payload) else {
                return unsupported();
            };
            let source = if super::inbox::group_sender(node).is_some() {
                chat_and_sender(key, node)
            } else {
                super::inbox::sender_for_record(key, node).and_then(|sender| {
                    Some((
                        ChatId::from_jid(&sender).ok()?,
                        DeviceAddress::from_jid(&sender).ok()?,
                    ))
                })
            };
            let Some((chat, by)) = source else {
                return unsupported();
            };
            let server_timestamp = stanza_timestamp(node);
            // `fromMe` is the keeping sender's perspective, same as reactions:
            // true means the target was their own (incoming to us), false
            // means the target was ours.
            let (direction, target_sender) = if keep.target_from_sender {
                (Direction::Incoming, Some(by.clone()))
            } else {
                (Direction::Outgoing, None)
            };
            let target = if chat.jid().server == crate::model::jid::Server::Group {
                let Ok(target) =
                    super::chat_settings::group_keep_target(chat.jid(), &by.to_jid(), payload, own)
                else {
                    return unsupported();
                };
                target
            } else {
                MessageKey {
                    chat,
                    sender: target_sender,
                    direction,
                    id: keep.target_id,
                }
            };
            return SyncEvent::MessageUpdated(crate::model::message::MessageUpdate {
                key: reference_target.cloned().unwrap_or(target),
                change: crate::model::message::MessageChange::KeepInChat {
                    by,
                    kept: keep.keep,
                    sender_timestamp: keep.sender_timestamp,
                    update_id: node
                        .get_attr("id")
                        .and_then(|v| v.as_text())
                        .and_then(|id| MessageId::new(id).ok()),
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
                let Some((chat, by)) = chat_and_sender(key, node) else {
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
                    // `contextInfo.participant` names the quoted message's author,
                    // who in a group is often a third member rather than the
                    // sender of this reply. Prefer it; fall back to the stanza
                    // sender when it is absent or unparseable (a direct chat's
                    // participant is the other party, so the two agree there).
                    let author = participant
                        .as_ref()
                        .and_then(|p| DeviceAddress::from_jid(p).ok())
                        .unwrap_or(by);
                    (Direction::Incoming, Some(author))
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
                // Colour and font of a styled status; all-None for ordinary text.
                style: text.style,
            },
            text.expiration,
        )
    };
    let Some((chat, sender)) = chat_and_sender(key, node) else {
        return unsupported();
    };
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .and_then(|v| MessageId::new(v).ok())
    else {
        return unsupported();
    };
    let server_timestamp = stanza_timestamp(node);
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

/// Retain direct target keys using original authenticated incoming routes or
/// attributed outgoing intents. This is local layout compatibility, not an APK migration.
fn direct_reaction_target(
    tx: &WriteTx<'_>,
    node: &Node,
    payload: &[u8],
) -> Result<Option<MessageKey>> {
    let Some(reaction) = super::reactions::decode(payload) else {
        return Ok(None);
    };
    let Some(from) = super::receipts::jid(node, "from").filter(|from| {
        from.device == 0
            && from.agent == 0
            && matches!(
                from.server,
                crate::model::jid::Server::User | crate::model::jid::Server::Lid
            )
    }) else {
        return Ok(None);
    };
    // Status reactions have a separate parent namespace (37.71 BoQ.A00).
    if super::reactions::key_fields(payload)
        .and_then(|(remote, _)| remote)
        .and_then(|remote| crate::model::jid::Jid::parse(&remote).ok())
        .is_some_and(|remote| {
            remote.server == crate::model::jid::Server::Broadcast && remote.user == "status"
        })
    {
        return Ok(None);
    }
    if !reaction.target_from_sender {
        return outgoing_reaction_target(tx, &from, reaction.target_id);
    }
    let target = node.clone().attr("id", reaction.target_id.as_str());
    let Some(record) =
        super::inbox::direct_record_in(&target, |key| tx.get_json(Table::Checkpoints, key, 1))?
    else {
        return Ok(None);
    };
    if !record
        .saved
        .as_ref()
        .is_some_and(|saved| matches!(saved["status"].as_str(), Some("decrypted" | "consumed")))
    {
        return Ok(None);
    }
    Ok(Some(MessageKey {
        chat: ChatId::from_jid(&record.chat)?,
        sender: Some(DeviceAddress::from_jid(&record.chat)?),
        direction: Direction::Incoming,
        id: reaction.target_id,
    }))
}

/// Group reaction/keep parent attribution retains the original device from a matching saved
/// route. This binds our existing event-key layout; no raw PN/LID alias is
/// adopted and no parent body is copied or view-once content opened.
fn group_update_target(
    tx: &WriteTx<'_>,
    node: &Node,
    payload: &[u8],
    own: &[crate::model::jid::Jid],
) -> Result<Option<MessageKey>> {
    let Some((chat, actor)) = super::inbox::group_sender(node) else {
        return Ok(None);
    };
    let fields = crate::signal::direct::payload_fields(payload).unwrap_or_default();
    if actor.agent != 0 {
        return Ok(None);
    }
    // APK 37.71 Bal.A01 and D9L.A01 both use BoQ for group references.
    // Mixed top-level updates stay opaque rather than picking one by order.
    let target = if fields.contains(&46)
        && fields.iter().all(|f| matches!(f, 46 | 35))
        && super::reactions::decode(payload).is_some()
    {
        super::reactions::group_target_for_sender(&chat, &actor, payload, own)
    } else if fields.contains(&51)
        && fields.iter().all(|f| matches!(f, 51 | 35))
        && super::chat_settings::decode_keep(payload).is_some()
    {
        super::chat_settings::group_keep_target(&chat, &actor, payload, own)
    } else {
        return Ok(None);
    };
    let Ok(mut target) = target else {
        return Ok(None);
    };
    if target.direction == Direction::Outgoing {
        let outbox =
            crate::outbox::Outbox::new("comparison-driver", crate::profile::Limits::default());
        if let Some(entry) = outbox.by_wire_id_in_write(tx, &target.id)? {
            if entry.intent.chat != target.chat {
                return Err(crate::Error::invalid(
                    "group_reaction",
                    "saved outgoing target chat differs",
                ));
            }
            target.chat = entry.intent.chat;
        }
        return Ok(Some(target));
    }
    let participant = target.sender.as_ref().unwrap().to_jid();
    let parent = Node::new("message")
        .attr("from", chat)
        .attr("participant", participant)
        .attr("id", target.id.as_str());
    let selected = super::group_inbox::select(&parent, |k| tx.get(Table::Checkpoints, k, 1))?;
    if let Some(saved) = selected
        .saved
        .as_ref()
        .filter(|s| matches!(s["status"].as_str(), Some("decrypted" | "consumed")))
    {
        let original =
            super::group_inbox::original_participant(saved, |k| tx.get(Table::Checkpoints, k, 1))?;
        target.sender = Some(DeviceAddress::from_jid(&original)?);
    }
    Ok(Some(target))
}

/// Bind a reaction to an original outgoing intent without changing its facts.
/// A directory/contact alias alone is not an authenticated chat association.
fn outgoing_reaction_target(
    tx: &WriteTx<'_>,
    from: &crate::model::jid::Jid,
    id: MessageId,
) -> Result<Option<MessageKey>> {
    let outbox = crate::outbox::Outbox::new("comparison-driver", crate::profile::Limits::default());
    let Some(entry) = outbox.by_wire_id_in_write(tx, &id)? else {
        return Ok(None);
    };
    let original = entry.intent.chat.jid();
    if !authenticated_direct_chat(tx, from, original)? {
        return Ok(None);
    }
    Ok(Some(MessageKey {
        chat: entry.intent.chat,
        sender: None,
        direction: Direction::Outgoing,
        id,
    }))
}

pub(super) use super::jid_map::authenticated_direct_chat;

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
    let reference_target = if super::inbox::group_sender(node).is_some() {
        // An already-published update remains historical evidence. A later
        // uncertain parent must not make its authenticated redelivery fail.
        let projected = tx
            .get_json::<serde_json::Value>(Table::Inbox, key, 1)?
            .is_some_and(|marker| marker["kind"] == "message_updated");
        if projected {
            None
        } else {
            group_update_target(tx, node, payload, &own)?
        }
    } else {
        direct_reaction_target(tx, node, payload)?
    };
    // A reaction whose key names `status@broadcast` is a like on a status,
    // which belongs with the viewer records rather than only in the event
    // journal. Recording it writes nothing outward.
    if let Some(like) = super::status_receipts::like_from_payload(payload, node) {
        super::status_receipts::record_like_in(tx, &like)?;
    }
    let projected = project(
        key,
        node,
        payload,
        None,
        poll.as_ref(),
        reference_target.as_ref(),
        &own,
    );
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
            SyncEvent::NewMessage(_)
                | SyncEvent::MessageUpdated(_)
                | SyncEvent::GroupHistoryNotice { .. }
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
        crate::binary::TokenDictionary::comparison_2_26_37_71(),
        &limits,
    )
    .decode_node()?;
    // Group/broadcast rows are keyed by their chat, not the participant's
    // direct-session selector. The direct-only check silently skipped their
    // authenticated historical captures, including field-102 history notices.
    let Some(sender) = super::inbox::sender_key_chat(&node)
        .map(|(chat, _)| chat)
        .or_else(|| super::inbox::sender_for_record(key, &node))
    else {
        return Ok(false);
    };
    let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(false);
    };
    let matches = if sender.server == crate::model::jid::Server::Group {
        super::group_inbox::matches(key, &node)
    } else {
        format!("received:{sender}:{id}").as_bytes() == key
    };
    if node.description != "message" || !matches {
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
            && let Some(phone) = super::inbox::authenticated_phone_alias(&node, &sender)
        {
            let mapping = format!("authenticated-lid:{}", from.to_non_ad());
            let old: Option<crate::model::jid::Jid> =
                tx.get_json(Table::Checkpoints, mapping.as_bytes(), 1)?;
            if old.as_ref().is_some_and(|v| v != &phone) {
                return Err(crate::Error::invalid(
                    "inbox",
                    "authenticated LID mapping changed",
                ));
            }
            tx.put_json(Table::Checkpoints, mapping.as_bytes(), 1, &phone)?;
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
    fn group_node() -> Node {
        Node::new("message")
            .attr("from", "120363000000000001@g.us")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "fixture")
            .attr("t", "123")
    }
    #[test]
    fn explicit_message_timestamps_match_java_oracle_across_event_kinds() {
        use crate::model::{ids::ChatId, jid::Jid, message::MessageRef};
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-numeric-reference.json"
        ))
        .unwrap();
        let sender = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let target = MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(&sender).unwrap(),
                sender: None,
                direction: Direction::Incoming,
                id: MessageId::new("original").unwrap(),
            },
        };
        let reaction =
            super::super::reactions::payload(&sender, &sender, &target, Some("👍"), 5).unwrap();
        let keep = super::super::chat_settings::keep_payload(&sender, &target, true, 5);
        let timer = super::super::chat_settings::disappearing_payload(60);
        for case in oracle["cases"].as_array().unwrap() {
            let text = case["input"].as_str().unwrap();
            let expected = case["valid"]
                .as_bool()
                .unwrap()
                .then(|| Timestamp(case["scaled"].as_str().unwrap().parse().unwrap()));
            let direct = node().attr("t", text);
            for payload in [
                b"\x0a\x04test".as_slice(),
                timer.as_slice(),
                reaction.as_slice(),
                keep.as_slice(),
            ] {
                let received = event(b"fixture", &direct, payload);
                let actual = match received {
                    SyncEvent::NewMessage(message) => message.server_timestamp,
                    SyncEvent::MessageUpdated(update) => update.server_timestamp,
                    _ => panic!("supported timestamp fixture became opaque"),
                };
                assert_eq!(actual, expected, "{text:?}");
            }
            let SyncEvent::NewMessage(message) =
                event(b"fixture", &group_node().attr("t", text), b"\x0a\x04test")
            else {
                panic!("group text became opaque");
            };
            assert_eq!(message.server_timestamp, expected, "{text:?}");
            let history = group_node()
                .attr("t", text)
                .children(vec![Node::new("meta").attr("appdata", "group_history")]);
            let SyncEvent::GroupHistoryNotice {
                server_timestamp, ..
            } = event(
                b"fixture",
                &history,
                b"\xb2\x06\x0d\x12\x0b\x0a\x051@lid\x10\x0a\x18\x02",
            )
            else {
                panic!("history notice became opaque");
            };
            assert_eq!(server_timestamp, expected, "{text:?}");
        }
        let missing = Node::new("message")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "fixture");
        assert_eq!(stanza_timestamp(&missing), None);
    }

    #[test]
    fn historical_expiring_text_keeps_unknown_or_expired_time_opaque() {
        // APK ContextInfo.expiration=25, ExtendedTextMessage.contextInfo=17;
        // one-second expiration, matching the existing literal fixture below.
        let payload = b"\x32\x0c\x0a\x04test\x8a\x01\x03\xc8\x01\x01";
        for time in [
            None,
            Some(""),
            Some("invalid"),
            Some("-1"),
            Some("9223372036854775807"),
            Some("-9223372036854775808"),
            Some("١٢٣"),
        ] {
            let mut incoming = Node::new("message")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("id", "fixture");
            if let Some(time) = time {
                incoming = incoming.attr("t", time);
            }
            assert!(matches!(
                event(b"fixture", &incoming, payload),
                SyncEvent::UnsupportedMessage(_)
            ));
        }
        let future = (Timestamp::now().0 / 1000 + 60).to_string();
        let future_bmp: String = future
            .chars()
            .map(|c| char::from_u32('٠' as u32 + c.to_digit(10).unwrap()).unwrap())
            .collect();
        assert!(matches!(
            event(b"fixture", &node().attr("t", future_bmp), payload),
            SyncEvent::NewMessage(_)
        ));
    }

    #[test]
    fn explicit_message_time_survives_atomic_journal_reopen_without_reprojection() {
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("message-times.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let expected = [123000, -42000, -1000];
        for (index, time) in ["١٢٣", "-42", "9223372036854775807"]
            .into_iter()
            .enumerate()
        {
            let wire_id = format!("fixture{index}");
            let incoming = node().attr("id", wire_id.clone()).attr("t", time);
            let key = format!("received:15555550100@s.whatsapp.net:{wire_id}");
            let route = crate::binary::Encoder::new(
                crate::binary::TokenDictionary::comparison_2_26_37_71(),
            )
            .encode(&incoming)
            .unwrap();
            let saved = serde_json::json!({"status":"decrypted", "route":route, "protobuf":b"\x0a\x04test".to_vec()});
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)?;
                    publish(tx, key.as_bytes(), &incoming, b"\x0a\x04test")
                })
                .unwrap();
            assert!(!restore(&store, key.as_bytes(), &saved).unwrap());
        }
        let head = store.journal_head().unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        let events = store
            .read_events(
                &ConsumerId::new("message-time-check").unwrap(),
                EventCursor::START,
                10,
            )
            .unwrap()
            .events;
        assert_eq!(events.len(), 3);
        for (saved, expected) in events.iter().zip(expected) {
            let SyncEvent::NewMessage(message) = &saved.event else {
                panic!("text event missing");
            };
            assert_eq!(message.server_timestamp, Some(Timestamp(expected)));
        }
        assert_eq!(store.journal_head().unwrap(), head);
        for index in 0..3 {
            let key = format!("received:15555550100@s.whatsapp.net:fixture{index}");
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert!(!restore(&store, key.as_bytes(), &saved).unwrap());
        }
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn group_reactions_bind_original_authors_devices_and_preserve_parent_records() {
        use crate::binary::{Encoder, TokenDictionary};
        use crate::model::{
            jid::Jid,
            send::{AttemptProgress, SendRequest},
        };
        use crate::store::Table;
        use prost::Message as _;
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("group-reactions.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let actor = Jid::parse("15555550100:3@s.whatsapp.net").unwrap();
        let authors = [
            Jid::parse("15555550101:3@s.whatsapp.net").unwrap(),
            Jid::parse("15555550101:3@lid").unwrap(),
        ];
        let own = Jid::parse("15555550199@s.whatsapp.net").unwrap();
        let own_lid = Jid::parse("222222222222222@lid").unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"pn":own.to_string(),"lid":own_lid.to_string()}),
                )
            })
            .unwrap();
        // Trusted stored-route fixtures test attribution only. They do not
        // authenticate a peer; real sender-key inbox crypto has separate tests.
        for (n, author) in authors.iter().enumerate() {
            let parent = group_node()
                .attr("participant", author.clone())
                .attr("id", "ORIGINAL");
            let route = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&parent)
                .unwrap();
            let key = if n == 0 {
                format!("received:{chat}:ORIGINAL")
            } else {
                super::super::group_inbox::key(&chat, author, "ORIGINAL").unwrap()
            };
            let saved =
                serde_json::json!({"status":"consumed", "route":route, "wire_id":"ORIGINAL"});
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved))
                .unwrap();
        }
        let outbox =
            crate::outbox::Outbox::new("original-own-group", crate::profile::Limits::default());
        store
            .write(|tx| {
                let ticket = outbox
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: crate::model::ids::ClientRequestId::new(
                                "original-own",
                            )?,
                            chat: ChatId::from_jid(&chat)?,
                            content: MessageContent::Text {
                                body: "saved own text".into(),
                                quoted: None,
                                style: Default::default(),
                            },
                        },
                    )?
                    .0;
                let mut entry = outbox.get_mut(tx, ticket.local_id)?.unwrap();
                entry.wire_id = Some(MessageId::new("OWN")?);
                outbox.set_progress(tx, &mut entry, AttemptProgress::InFlight)
            })
            .unwrap();
        let original_outbox = store.read(|tx| tx.scan(Table::Outbox, 1, 100)).unwrap();
        let parents = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let mut keys = Vec::new();
        let own_actor = Jid::parse("222222222222222:3@lid").unwrap();
        for (n, author) in authors
            .iter()
            .chain([&own, &own_lid, &own, &own_lid])
            .cycle()
            .take(24)
            .enumerate()
        {
            let case = n % 6;
            let reactor = if n % 12 < 6 { &actor } else { &own_actor };
            let id = if case < 2 { "ORIGINAL" } else { "OWN" };
            let mut payload = super::super::reactions::group_payload(
                &chat,
                &author.to_non_ad(),
                id,
                case >= 4,
                if n % 2 == 0 { Some("👍") } else { None },
                42,
            )
            .unwrap();
            if case == 0 {
                let mut envelope =
                    super::super::reactions::Envelope::decode(payload.as_slice()).unwrap();
                envelope
                    .reaction
                    .as_mut()
                    .unwrap()
                    .key
                    .as_mut()
                    .unwrap()
                    .remote_jid = Some("120363000000000002@g.us".into());
                payload = envelope.encode_to_vec();
            }
            if n >= 12 {
                let reaction =
                    super::super::reactions::Envelope::decode(payload.as_slice()).unwrap();
                payload = super::super::chat_settings::Envelope {
                    protocol: None,
                    keep_in_chat: Some(super::super::chat_settings::KeepInChatMessage {
                        key: reaction.reaction.unwrap().key,
                        keep_type: Some(if n % 2 == 0 { 1 } else { 2 }),
                        timestamp_ms: Some(42),
                    }),
                }
                .encode_to_vec();
            }
            let node = group_node()
                .attr("participant", reactor.clone())
                .attr("id", format!("REACTION-{n}"));
            let selector =
                super::super::group_inbox::key(&chat, reactor, &format!("REACTION-{n}")).unwrap();
            assert!(
                store
                    .write(|tx| publish(tx, selector.as_bytes(), &node, &payload))
                    .unwrap()
            );
            keys.push((selector, node, payload));
        }
        let batch = store
            .read_events(
                &ConsumerId::new("reaction-check").unwrap(),
                EventCursor::START,
                100,
            )
            .unwrap();
        let events: Vec<_> = batch
            .events
            .iter()
            .filter(|e| matches!(e.event, SyncEvent::MessageUpdated(_)))
            .collect();
        assert_eq!(events.len(), 24);
        for (n, event) in events.iter().enumerate() {
            let case = n % 6;
            let incoming = if n % 12 < 6 {
                !(2..4).contains(&case)
            } else {
                case < 4
            };
            let SyncEvent::MessageUpdated(update) = &event.event else {
                panic!("not a reaction");
            };
            assert_eq!(update.key.chat, ChatId::from_jid(&chat).unwrap());
            assert_eq!(
                update.key.id.as_str(),
                if case < 2 { "ORIGINAL" } else { "OWN" }
            );
            assert_eq!(
                update.key.direction,
                if incoming {
                    Direction::Incoming
                } else {
                    Direction::Outgoing
                }
            );
            assert_eq!(
                update.key.sender,
                if incoming && case < 2 {
                    Some(DeviceAddress::from_jid(&authors[case]).unwrap())
                } else if incoming {
                    Some(
                        DeviceAddress::from_jid(if case % 2 == 0 { &own } else { &own_lid })
                            .unwrap(),
                    )
                } else {
                    None
                }
            );
            let by = if n < 12 {
                let crate::model::message::MessageChange::Reaction { by, emoji } = &update.change
                else {
                    panic!("wrong reaction change");
                };
                assert_eq!(emoji.as_deref(), if n % 2 == 0 { Some("👍") } else { None });
                by
            } else {
                let crate::model::message::MessageChange::KeepInChat {
                    by,
                    kept,
                    sender_timestamp,
                    update_id,
                } = &update.change
                else {
                    panic!("wrong keep change");
                };
                assert_eq!(*kept, n % 2 == 0);
                assert_eq!(*sender_timestamp, Some(Timestamp(42)));
                assert_eq!(
                    update_id.as_ref().unwrap().as_str(),
                    format!("REACTION-{n}")
                );
                by
            };
            assert_eq!(
                *by,
                DeviceAddress::from_jid(if n % 12 < 6 { &actor } else { &own_actor }).unwrap()
            );
        }
        for (key, raw) in &parents {
            assert_eq!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key, 1))
                    .unwrap()
                    .as_ref(),
                Some(raw)
            );
        }
        assert_eq!(
            store.read(|tx| tx.scan(Table::Outbox, 1, 100)).unwrap(),
            original_outbox
        );
        let head = store.journal_head().unwrap();
        drop(store);
        let mut reopen = options();
        reopen.create = false;
        let store = ProtocolStore::open(reopen).unwrap();
        for (key, node, payload) in &keys {
            assert!(
                !store
                    .write(|tx| publish(tx, key.as_bytes(), node, payload))
                    .unwrap()
            );
        }
        assert_eq!(store.journal_head().unwrap(), head);
        let retained = store
            .read_events(
                &ConsumerId::new("reaction-reopen-check").unwrap(),
                EventCursor::START,
                100,
            )
            .unwrap();
        assert_eq!(retained.events, batch.events);
        // Two stored claims for the same original member are uncertainty, not
        // permission to select either parent or repeat an old event marker.
        let legacy: serde_json::Value = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("received:{chat}:ORIGINAL").as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    super::super::group_inbox::key(&chat, &authors[0], "ORIGINAL")?.as_bytes(),
                    1,
                    &legacy,
                )
            })
            .unwrap();
        let (key, node, payload) = &keys[0];
        assert!(
            !store
                .write(|tx| publish(tx, key.as_bytes(), node, payload))
                .unwrap()
        );
        assert_eq!(store.journal_head().unwrap(), head);
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let payload = super::super::reactions::group_payload(
            &chat,
            &authors[0],
            "ORIGINAL",
            false,
            Some("👍"),
            43,
        )
        .unwrap();
        assert!(matches!(store.write(|tx| {
            tx.put_json(Table::Checkpoints,b"rollback-marker",1,&true)?;
            publish(tx,b"new-ambiguous-reaction",&group_node(),&payload)
        }),Err(Error::InvalidInput{reason,..}) if reason=="multiple records for the same group participant and id"));
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn malformed_group_reaction_keys_remain_opaque_without_an_update() {
        use crate::model::jid::Jid;
        use prost::Message as _;
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let author = Jid::parse("15555550101@lid").unwrap();
        let own = [Jid::parse("15555550199@s.whatsapp.net").unwrap()];
        let base =
            super::super::reactions::group_payload(&chat, &author, "TARGET", false, Some("👍"), 42)
                .unwrap();
        for (participant, remote) in [
            (None, chat.to_string()),
            (Some("".to_owned()), chat.to_string()),
            (Some("status@broadcast".to_owned()), chat.to_string()),
            (Some(author.to_string()), "status@broadcast".to_owned()),
        ] {
            let mut envelope = super::super::reactions::Envelope::decode(base.as_slice()).unwrap();
            let key = envelope.reaction.as_mut().unwrap().key.as_mut().unwrap();
            key.participant = participant;
            key.remote_jid = Some(remote);
            assert!(matches!(
                project(
                    b"malformed",
                    &group_node(),
                    &envelope.encode_to_vec(),
                    None,
                    None,
                    None,
                    &own
                ),
                SyncEvent::UnsupportedMessage(_)
            ));
        }
        assert!(matches!(
            project(b"unknown-own", &group_node(), &base, None, None, None, &[]),
            SyncEvent::UnsupportedMessage(_)
        ));
        let agent = Jid::new_ad("15555550100", 2, 3, crate::model::jid::Server::User).unwrap();
        assert!(matches!(
            project(
                b"agent-reactor",
                &group_node().attr("participant", agent),
                &base,
                None,
                None,
                None,
                &own
            ),
            SyncEvent::UnsupportedMessage(_)
        ));
    }

    #[test]
    fn history_notice_is_a_control_event_not_a_reply_or_send_form() {
        let payload = b"\xb2\x06\x0d\x12\x0b\x0a\x051@lid\x10\x0a\x18\x02";
        let node = group_node().children(vec![Node::new("meta").attr("appdata", "group_history")]);
        let projected = event(b"notice", &node, payload);
        assert_eq!(projected.kind(), "group_history_notice");
        let SyncEvent::GroupHistoryNotice {
            key,
            metadata,
            server_timestamp,
            ..
        } = &projected
        else {
            panic!("notice did not project");
        };
        assert!(matches!(key.chat, ChatId::Group(_)));
        assert_eq!(key.id.as_str(), "fixture");
        assert_eq!(key.direction, Direction::Incoming);
        assert_eq!(metadata.message_count, 2);
        assert_eq!(*server_timestamp, Some(Timestamp(123000)));
        let encoded = serde_json::to_value(&projected).unwrap();
        assert!(!encoded.to_string().contains("1@lid"));
        assert!(serde_json::from_value::<MessageContent>(encoded).is_err());
        assert!(matches!(
            event(b"notice", &group_node(), payload),
            SyncEvent::UnsupportedMessage(_)
        ));
        assert!(matches!(
            event(b"notice", &node.clone().attr("category", "peer"), payload),
            SyncEvent::UnsupportedMessage(_)
        ));
    }

    #[test]
    fn group_notice_backfill_is_atomic_once_only_and_preserves_protocol_rows() {
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("notice.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let payload = b"\xb2\x06\x0d\x12\x0b\x0a\x051@lid\x10\x0a\x18\x02".to_vec();
        let node = group_node().children(vec![Node::new("meta").attr("appdata", "group_history")]);
        let key = b"received:120363000000000001@g.us:fixture";
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
        let saved = serde_json::json!({"status":"decrypted", "route":route, "protobuf":payload});
        store
            .write(|tx| {
                tx.put_json(Table::Checkpoints, key, 1, &saved)?;
                tx.put_json(Table::PeerIdentities, b"baseline", 1, &vec![1u8, 2, 3])?;
                tx.put_json(Table::SignalSessions, b"baseline", 1, &vec![4u8, 5, 6])?;
                // Simulate the old unimplemented content projection while retaining
                // the original authenticated route/body for its subsequent upgrade.
                let old = tx.append_event(SyncEvent::UnsupportedMessage(UnsupportedMessage {
                    type_hint: "fixture".into(),
                    raw: bytes::Bytes::new(),
                    truncated: true,
                }))?;
                tx.put_json(
                    Table::Inbox,
                    key,
                    1,
                    &serde_json::json!({"event_id":old.event_id, "kind":"unsupported_message"}),
                )
            })
            .unwrap();
        let cancelled: Result<()> = store.write(|tx| {
            assert!(publish(tx, key, &node, &payload)?);
            Err(Error::Cancelled)
        });
        assert!(cancelled.is_err());
        let kind = |store: &ProtocolStore| {
            store
                .read(|tx| tx.get_json::<serde_json::Value>(Table::Inbox, key, 1))
                .unwrap()
                .unwrap()["kind"]
                .clone()
        };
        assert_eq!(kind(&store), "unsupported_message");
        assert!(restore(&store, key, &saved).unwrap());
        assert_eq!(kind(&store), "group_history_notice");
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert!(!restore(&store, key, &saved).unwrap());
        let events = store
            .read_events(&ConsumerId::new("notice").unwrap(), EventCursor::START, 10)
            .unwrap()
            .events;
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].event.kind(), "group_history_notice");
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::PeerIdentities, b"baseline", 1))
                .unwrap(),
            Some(vec![1, 2, 3])
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::SignalSessions, b"baseline", 1))
                .unwrap(),
            Some(vec![4, 5, 6])
        );
        assert!(restore(&store, b"received:other@g.us:fixture", &saved).is_err());
        let mut consumed = saved.clone();
        consumed["status"] = "consumed".into();
        assert!(!restore(&store, key, &consumed).unwrap());
    }

    #[test]
    fn authenticated_broadcast_backfill_uses_original_chat_route() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("broadcast.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap();
        let node = group_node().attr("from", "status@broadcast");
        let key = b"received:status@broadcast:fixture";
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
        let saved = serde_json::json!({"status":"decrypted", "route":route, "protobuf":b"\x0a\x04test".to_vec()});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key, 1, &saved))
            .unwrap();
        assert!(restore(&store, key, &saved).unwrap());
        assert!(!restore(&store, key, &saved).unwrap());
    }
    #[test]
    fn group_text_projects_with_group_chat_and_participant_sender() {
        use crate::model::{ids::ChatId, jid::Jid};
        let SyncEvent::NewMessage(message) = event(b"fixture", &group_node(), b"\x0a\x04test")
        else {
            panic!("group text did not project as a new message");
        };
        assert_eq!(
            message.content,
            MessageContent::Text {
                body: "test".into(),
                quoted: None,
                style: Default::default(),
            }
        );
        // The chat is the group, not the sender.
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        assert_eq!(message.key.chat, ChatId::from_jid(&group).unwrap());
        assert!(matches!(message.key.chat, ChatId::Group(_)));
        // The sender is the participant, not the group.
        let sender = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        assert_eq!(
            message.key.sender,
            Some(DeviceAddress::from_jid(&sender).unwrap())
        );
    }
    #[test]
    fn status_update_projects_into_the_broadcast_chat_with_the_poster_as_sender() {
        use crate::model::{ids::ChatId, jid::Jid};
        let status = Node::new("message")
            .attr("from", "status@broadcast")
            .attr("participant", "15555550100@s.whatsapp.net")
            .attr("id", "fixture")
            .attr("t", "123");
        let SyncEvent::NewMessage(message) = event(b"fixture", &status, b"\x0a\x04test") else {
            panic!("status did not project as a new message");
        };
        let broadcast = Jid::parse("status@broadcast").unwrap();
        assert_eq!(message.key.chat, ChatId::from_jid(&broadcast).unwrap());
        let ChatId::Broadcast(ref b) = message.key.chat else {
            panic!("status must project into the broadcast chat");
        };
        assert!(b.is_status());
        // The poster is the participant, carried in `sender` as for a group.
        let poster = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        assert_eq!(
            message.key.sender,
            Some(DeviceAddress::from_jid(&poster).unwrap())
        );
    }
    #[test]
    fn group_stanza_without_participant_or_with_non_group_from_stays_opaque() {
        // `from` is a group but no `participant`: not a well-formed group stanza.
        let no_participant = Node::new("message")
            .attr("from", "120363000000000001@g.us")
            .attr("id", "fixture")
            .attr("t", "123");
        assert!(matches!(
            event(b"fixture", &no_participant, b"\x0a\x04test"),
            SyncEvent::UnsupportedMessage(_)
        ));
        // `participant` present but `from` is not a group: direct routing refuses
        // it (session_sender returns None when participant is set).
        let not_a_group = Node::new("message")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("participant", "15555550101@s.whatsapp.net")
            .attr("id", "fixture")
            .attr("t", "123");
        assert!(matches!(
            event(b"fixture", &not_a_group, b"\x0a\x04test"),
            SyncEvent::UnsupportedMessage(_)
        ));
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
                quoted: None,
                style: Default::default(),
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
                sender_timestamp: Some(Timestamp(5)),
                update_id: Some(MessageId::new("fixture").unwrap()),
            }
        );
        // Literal BV3 timestamp omission/zero are distinct received facts.
        // The action ID never replaces the original target's ID.
        for (hex, expected) in [
            ("9a03090a051a036162631001", None),
            ("9a030b0a051a0361626310011800", Some(Timestamp(0))),
        ] {
            let payload = hex::decode(hex).unwrap();
            let SyncEvent::MessageUpdated(update) = event(b"fixture", &node(), &payload) else {
                panic!("literal keep did not project");
            };
            assert_eq!(update.key.id.as_str(), "abc");
            assert_eq!(update.server_timestamp, Some(Timestamp(123000)));
            assert!(matches!(update.change, MessageChange::KeepInChat {
                sender_timestamp, update_id: Some(ref id), ..
            } if sender_timestamp == expected && id.as_str() == "fixture"));
        }
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
    fn aliasless_lid_vote_finds_legacy_poll_without_changing_its_message_key() {
        use crate::model::{
            jid::Jid,
            message::{MessageChange, MessageRef},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("legacy-poll.redb"),
            account_id: AccountId::new("legacy-poll").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([21; 32]))),
            create: true,
        })
        .unwrap();
        let phone = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let lid = Jid::parse("123456789@lid").unwrap();
        let options = vec!["Yes".into(), "No".into()];
        let creation =
            super::super::polls::creation_payload("Lunch?", &options, 0, [5; 32]).unwrap();
        let original = Node::new("message")
            .attr("from", lid.clone())
            .attr("sender_pn", phone.clone())
            .attr("id", "LEGACYPOLL");
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&original)
                .unwrap();
        let original_key = format!("received:{phone}:LEGACYPOLL");
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    original_key.as_bytes(),
                    1,
                    &serde_json::json!({"status":"decrypted","protobuf":creation,"route":route}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("authenticated-lid:{lid}").as_bytes(),
                    1,
                    &phone,
                )?;
                publish(tx, original_key.as_bytes(), &original, &creation)?;
                Ok(())
            })
            .unwrap();
        let encrypted = super::super::polls::encrypt_vote(
            &[5; 32],
            "LEGACYPOLL",
            &phone,
            &lid,
            &options[..1],
            [8; 12],
        )
        .unwrap();
        let target = MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(&phone).unwrap(),
                sender: None,
                direction: Direction::Outgoing,
                id: MessageId::new("LEGACYPOLL").unwrap(),
            },
        };
        let payload = super::super::polls::vote_payload(&phone, &target, encrypted, 7);
        let vote = Node::new("message")
            .attr("from", lid.clone())
            .attr("id", "LIDVOTE");
        store
            .write(|tx| {
                publish(
                    tx,
                    format!("received:{lid}:LIDVOTE").as_bytes(),
                    &vote,
                    &payload,
                )
            })
            .unwrap();
        let events = store
            .read_events(
                &ConsumerId::new("legacy-poll-check").unwrap(),
                EventCursor::START,
                10,
            )
            .unwrap()
            .events;
        let SyncEvent::MessageUpdated(update) = &events.last().unwrap().event else {
            panic!("legacy poll vote not projected")
        };
        assert_eq!(update.key.chat, ChatId::from_jid(&phone).unwrap());
        assert_eq!(
            update.key.sender,
            Some(DeviceAddress::from_jid(&phone).unwrap())
        );
        assert_eq!(update.key.id.as_str(), "LEGACYPOLL");
        assert_eq!(
            update.change,
            MessageChange::PollVote {
                by: DeviceAddress::from_jid(&lid).unwrap(),
                selected: vec!["Yes".into()],
                unresolved: vec![]
            }
        );
        assert_eq!(events.len(), 2);
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
                        "chat": voter.to_string(),
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
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
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
    fn group_quoted_reply_attributes_the_quote_to_its_author_not_the_replier() {
        use crate::model::{
            ids::{ChatId, DeviceAddress},
            jid::Jid,
            message::Direction,
        };
        // In a group, member A replies quoting member B. The quote must be keyed
        // to B (contextInfo.participant), not to A (the stanza sender).
        let own = Jid::parse("77700000001@lid").unwrap();
        let author = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let payload = super::super::quotes::reply_payload(
            "agreed",
            "ABCDEF",
            "15555550102@s.whatsapp.net",
            super::super::quotes::conversation("original"),
        );
        let SyncEvent::NewMessage(message) = project(
            b"fixture",
            &group_node(),
            &payload,
            None,
            None,
            None,
            std::slice::from_ref(&own),
        ) else {
            panic!("expected a group message")
        };
        let MessageContent::Text { body, quoted, .. } = message.content else {
            panic!("expected text")
        };
        assert_eq!(body, "agreed");
        // The reply itself is from the stanza participant.
        assert_eq!(
            message.key.sender,
            Some(
                DeviceAddress::from_jid(&Jid::parse("15555550100@s.whatsapp.net").unwrap())
                    .unwrap()
            )
        );
        let quoted = quoted.expect("quoted reference");
        assert_eq!(quoted.key.direction, Direction::Incoming);
        // The quoted message belongs to its author, a third member.
        assert_eq!(
            quoted.key.sender,
            Some(DeviceAddress::from_jid(&author).unwrap())
        );
        // Both live in the group chat.
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        assert_eq!(quoted.key.chat, ChatId::from_jid(&group).unwrap());
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
            None,
            std::slice::from_ref(&own),
        ) else {
            panic!("expected message")
        };
        let MessageContent::Text { body, quoted, .. } = message.content else {
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

#[cfg(test)]
mod direct_reaction_target_tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, EventCursor},
            jid::Jid,
            message::{MessageChange, MessageRef},
        },
        store::{ConsumerId, MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };

    #[test]
    fn direct_reaction_keeps_original_target_and_actual_reactor_without_rewriting_history() {
        for case in [
            "legacy",
            "consumed",
            "canonical",
            "absent",
            "foreign",
            "ambiguous",
            "missing_route",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("reaction.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [17; 32],
                    ))),
                    create,
                })
                .unwrap()
            };
            let store = open(true);
            let lid = Jid::parse("12345@lid").unwrap();
            let phone = Jid::parse("15555550101@s.whatsapp.net").unwrap();
            let target_id = MessageId::new("ORIGINAL").unwrap();
            let legacy_key = format!("received:{phone}:{target_id}");
            let canonical_key = format!("received:{lid}:{target_id}");
            let original = Node::new("message")
                .attr("id", target_id.as_str())
                .attr(
                    "from",
                    if case == "foreign" {
                        Jid::parse("54321@lid").unwrap()
                    } else {
                        lid.clone()
                    },
                )
                .attr("sender_pn", phone.clone());
            let route = crate::binary::Encoder::new(
                crate::binary::TokenDictionary::comparison_2_26_37_71(),
            )
            .encode(&original)
            .unwrap();
            // Trusted historical-record fixture, not an unauthenticated capture
            // promoted by the receive path. No ratchet is advanced by publish.
            store.write(|tx| {
                tx.put_json(Table::Checkpoints, format!("authenticated-lid:{lid}").as_bytes(), 1, &phone)?;
                if matches!(case, "legacy"|"consumed"|"foreign"|"ambiguous"|"missing_route") {
                    let mut saved = serde_json::json!({"status":"decrypted", "route":route, "protobuf":b"\x0a\x04test".to_vec()});
                    if case=="consumed" { saved["status"]="consumed".into();saved.as_object_mut().unwrap().remove("protobuf"); }
                    if case=="missing_route" { saved.as_object_mut().unwrap().remove("route"); }
                    tx.put_json(Table::Checkpoints, legacy_key.as_bytes(), 1, &saved)?;
                }
                if matches!(case,"canonical"|"ambiguous") {
                    tx.put_json(Table::Checkpoints, canonical_key.as_bytes(), 1, &serde_json::json!({"status":"decrypted"}))?;
                }
                Ok(())
            }).unwrap();
            let before = store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap();
            let node = Node::new("message")
                .attr("from", lid.clone())
                .attr("id", "REACTION");
            let key = format!("received:{lid}:REACTION");
            // fromMe=true is the remote sender's own incoming target. The
            // payload builder's local Outgoing direction produces that bit.
            let target = MessageRef {
                key: MessageKey {
                    chat: ChatId::from_jid(&lid).unwrap(),
                    sender: None,
                    direction: Direction::Outgoing,
                    id: target_id.clone(),
                },
            };
            let payload =
                super::super::reactions::payload(&lid, &lid, &target, Some("👍"), 7).unwrap();
            let result = store.write(|tx| publish(tx, key.as_bytes(), &node, &payload));
            if matches!(case, "ambiguous" | "missing_route") {
                let reason = result.unwrap_err().to_string();
                assert!(reason.contains(if case == "ambiguous" {
                    "multiple authenticated direct inbox records"
                } else {
                    "legacy direct inbox route unavailable"
                }));
                assert_eq!(
                    store.read(|tx| tx.journal_head()).unwrap(),
                    EventCursor::START
                );
                assert!(
                    store
                        .read(|tx| tx.get(Table::Inbox, key.as_bytes(), 1))
                        .unwrap()
                        .is_none()
                );
                continue;
            }
            assert!(result.unwrap());
            assert_eq!(
                store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap(),
                before
            );
            drop(store);
            let store = open(false);
            assert!(
                !store
                    .write(|tx| publish(tx, key.as_bytes(), &node, &payload))
                    .unwrap()
            );
            let events = store
                .read_events(&ConsumerId::new("fixture").unwrap(), EventCursor::START, 10)
                .unwrap()
                .events;
            assert_eq!(events.len(), 1);
            let SyncEvent::MessageUpdated(update) = &events[0].event else {
                panic!("expected reaction")
            };
            let expected = if matches!(case, "legacy" | "consumed") {
                &phone
            } else {
                &lid
            };
            assert_eq!(update.key.chat, ChatId::from_jid(expected).unwrap());
            assert_eq!(
                update.key.sender,
                Some(DeviceAddress::from_jid(expected).unwrap())
            );
            assert_eq!(update.key.direction, Direction::Incoming);
            assert_eq!(update.key.id, target_id);
            assert_eq!(
                update.change,
                MessageChange::Reaction {
                    by: DeviceAddress::from_jid(&lid).unwrap(),
                    emoji: Some("👍".into())
                }
            );
            assert_eq!(
                store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap(),
                before
            );
            // Without a saved outgoing intent, outgoing targets do not acquire
            // this selector. Status, group and companion actors remain separate.
            let ours = MessageRef {
                key: MessageKey {
                    direction: Direction::Incoming,
                    ..target.key.clone()
                },
            };
            let ours = super::super::reactions::payload(&lid, &lid, &ours, None, 7).unwrap();
            store
                .write(|tx| {
                    assert!(direct_reaction_target(tx, &node, &ours)?.is_none());
                    assert!(
                        direct_reaction_target(
                            tx,
                            &node.clone().attr("from", "12345:3@lid"),
                            &payload
                        )?
                        .is_none()
                    );
                    assert!(
                        direct_reaction_target(
                            tx,
                            &node
                                .clone()
                                .attr("from", "120363000000000001@g.us")
                                .attr("participant", lid.clone()),
                            &payload
                        )?
                        .is_none()
                    );
                    let status = super::super::reactions::payload(
                        &lid,
                        &Jid::parse("status@broadcast")?,
                        &target,
                        None,
                        7,
                    )?;
                    assert!(direct_reaction_target(tx, &node, &status)?.is_none());
                    Ok(())
                })
                .unwrap();
        }
    }
}

#[cfg(test)]
mod outgoing_reaction_tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ClientRequestId},
            jid::Jid,
            message::{MessageChange, MessageRef},
            send::{AttemptProgress, DeliveryFact, DeliveryFactKind, SendRequest},
        },
        outbox::Outbox,
        profile::Limits,
        store::{ConsumerId, MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };
    #[test]
    fn outgoing_reactions_use_original_intents_and_only_authenticated_namespace_pairs() {
        for case in [
            "mapped",
            "reverse",
            "same_pn",
            "same_lid",
            "unattributed",
            "alias_only",
            "foreign_pn",
            "foreign_lid",
            "invalid_map",
            "duplicate",
            "absent",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("outgoing.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [19; 32],
                    ))),
                    create,
                })
                .unwrap()
            };
            let store = open(true);
            let ob = Outbox::new("saved-namespace", Limits::default());
            let phone = Jid::parse("15555550101@s.whatsapp.net").unwrap();
            let lid = Jid::parse("12345@lid").unwrap();
            let actor = if matches!(case, "reverse" | "same_pn") {
                phone.clone()
            } else {
                lid.clone()
            };
            let original = match case {
                "reverse" | "same_lid" => lid.clone(),
                "foreign_lid" => Jid::parse("54321@lid").unwrap(),
                "foreign_pn" => Jid::parse("15555550102@s.whatsapp.net").unwrap(),
                _ => phone.clone(),
            };
            let wire = MessageId::new("ORIGINAL-OWN").unwrap();
            let request = SendRequest {
                client_request_id: ClientRequestId::new("original-request").unwrap(),
                chat: ChatId::from_jid(&original).unwrap(),
                content: MessageContent::Text {
                    body: "private original body".into(),
                    quoted: None,
                    style: Default::default(),
                },
            };
            let ticket = store.write(|tx| ob.enqueue(tx, &request)).unwrap().0;
            store
                .write(|tx| {
                    let mut entry = ob.get_mut(tx, ticket.local_id)?.unwrap();
                    entry.wire_id = Some(wire.clone());
                    entry.progress = AttemptProgress::InFlight;
                    entry.facts.push(DeliveryFact {
                        kind: DeliveryFactKind::DeliveryUnknown,
                        device: None,
                        observed_at: Timestamp(7),
                        server_timestamp: None,
                        code: None,
                    });
                    tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)?;
                    if case == "duplicate" {
                        entry.local_id = crate::model::ids::LocalMessageId([42; 16]);
                        tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)?;
                    }
                    if matches!(case, "mapped" | "reverse" | "foreign_pn" | "foreign_lid") {
                        tx.put_json(
                            Table::Checkpoints,
                            format!("authenticated-lid:{lid}").as_bytes(),
                            1,
                            &phone,
                        )?;
                    }
                    if case == "invalid_map" {
                        tx.put_json(
                            Table::Checkpoints,
                            format!("authenticated-lid:{lid}").as_bytes(),
                            1,
                            &serde_json::json!({"private":"never echo"}),
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
            let before = store.read(|tx| tx.scan(Table::Outbox, 1, 10)).unwrap();
            let before_head = store.read(|tx| tx.journal_head()).unwrap();
            let target_id = if case == "absent" {
                MessageId::new("UNKNOWN").unwrap()
            } else {
                wire.clone()
            };
            // Local fixture: the remote sender's Incoming perspective yields fromMe=false.
            let target = MessageRef {
                key: MessageKey {
                    chat: ChatId::from_jid(&actor).unwrap(),
                    sender: None,
                    direction: Direction::Incoming,
                    id: target_id.clone(),
                },
            };
            let payload =
                super::super::reactions::payload(&actor, &actor, &target, Some("👍"), 7).unwrap();
            let mut node = Node::new("message")
                .attr("id", "NEW-REACTION")
                .attr("from", actor.clone());
            if case == "alias_only" {
                node = node.attr("sender_pn", phone.clone());
            }
            let key = format!("received:{actor}:NEW-REACTION");
            let result = store.write(|tx| publish(tx, key.as_bytes(), &node, &payload));
            if matches!(case, "invalid_map" | "duplicate") {
                let error = result.unwrap_err().to_string();
                assert!(error.contains(if case == "duplicate" {
                    "duplicate outbox wire id"
                } else {
                    "invalid authenticated LID mapping"
                }));
                assert!(!error.contains("never echo"));
                assert_eq!(store.read(|tx| tx.journal_head()).unwrap(), before_head);
                assert!(
                    store
                        .read(|tx| tx.get(Table::Inbox, key.as_bytes(), 1))
                        .unwrap()
                        .is_none()
                );
            } else {
                assert!(result.unwrap());
                drop(store);
                let store = open(false);
                assert!(
                    !store
                        .write(|tx| publish(tx, key.as_bytes(), &node, &payload))
                        .unwrap()
                );
                let events = store
                    .read_events(&ConsumerId::new("fixture").unwrap(), before_head, 10)
                    .unwrap()
                    .events;
                assert_eq!(events.len(), 1);
                let SyncEvent::MessageUpdated(update) = &events[0].event else {
                    panic!("expected reaction")
                };
                let expected = if matches!(case, "mapped" | "reverse" | "same_pn" | "same_lid") {
                    &original
                } else {
                    &actor
                };
                assert_eq!(update.key.chat, ChatId::from_jid(expected).unwrap());
                assert_eq!(update.key.direction, Direction::Outgoing);
                assert_eq!(update.key.sender, None);
                assert_eq!(update.key.id, target_id);
                assert_eq!(
                    update.change,
                    MessageChange::Reaction {
                        by: DeviceAddress::from_jid(&actor).unwrap(),
                        emoji: Some("👍".into())
                    }
                );
                assert_eq!(
                    store.read(|tx| tx.scan(Table::Outbox, 1, 10)).unwrap(),
                    before
                );
                continue;
            }
            assert_eq!(
                store.read(|tx| tx.scan(Table::Outbox, 1, 10)).unwrap(),
                before
            );
        }
    }
}

#[cfg(test)]
mod own_poll_chat_tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, EventCursor},
            jid::{Jid, JidKind},
            message::{MessageChange, MessageRef},
        },
        store::{ConsumerId, MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };
    #[test]
    fn own_poll_votes_keep_original_chat_actual_voter_and_durable_markers() {
        for case in [
            "same_pn",
            "same_lid",
            "mapped",
            "reverse",
            "zero_ad",
            "unattributed",
            "alias_only",
            "foreign",
            "missing_chat",
            "invalid_chat",
            "invalid_map",
            "companion",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("poll.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [21; 32],
                    ))),
                    create,
                })
                .unwrap()
            };
            let store = open(true);
            let pn = Jid::parse("15555550101@s.whatsapp.net").unwrap();
            let lid = Jid::parse("12345@lid").unwrap();
            let creator = Jid::parse("15555550199@s.whatsapp.net").unwrap();
            let original = if matches!(case, "same_lid" | "reverse") {
                lid.clone()
            } else {
                pn.clone()
            };
            let mut voter = match case {
                "same_pn" | "reverse" => pn.clone(),
                "foreign" => Jid::parse("15555550102@s.whatsapp.net").unwrap(),
                _ => lid.clone(),
            };
            if case == "zero_ad" {
                voter.kind = JidKind::AgentDevice;
            }
            if case == "companion" {
                voter.device = 3;
                voter.kind = JidKind::AgentDevice;
            }
            let mut own = serde_json::json!({"chat":original.to_string(),"secret":vec![9u8;32],"options":["Yes","No"],"creator_pn":creator.to_string()});
            if case == "missing_chat" {
                own.as_object_mut().unwrap().remove("chat");
            }
            if case == "invalid_chat" {
                own["chat"] = serde_json::json!("private invalid chat");
            }
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, b"poll-secret:ORIGINAL", 1, &own)?;
                    if matches!(case, "mapped" | "reverse" | "zero_ad" | "invalid_map") {
                        let value = if case == "invalid_map" {
                            serde_json::json!({"private":"invalid"})
                        } else {
                            serde_json::to_value(&pn).unwrap()
                        };
                        tx.put_json(
                            Table::Checkpoints,
                            b"authenticated-lid:12345@lid",
                            1,
                            &value,
                        )?;
                    }
                    if case == "alias_only" {
                        tx.put_json(
                            Table::Checkpoints,
                            b"lid:15555550101@s.whatsapp.net",
                            1,
                            &lid,
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
            let before = store.read(|tx| tx.scan(Table::Checkpoints, 1, 50)).unwrap();
            let enc = super::super::polls::encrypt_vote(
                &[9; 32],
                "ORIGINAL",
                &creator,
                &voter,
                &["No".into()],
                [3; 12],
            )
            .unwrap();
            let target = MessageRef {
                key: MessageKey {
                    chat: ChatId::from_jid(&original).unwrap(),
                    sender: None,
                    direction: Direction::Incoming,
                    id: MessageId::new("ORIGINAL").unwrap(),
                },
            };
            let payload = super::super::polls::vote_payload(&creator, &target, enc, 7);
            let node = Node::new("message")
                .attr("from", voter.clone())
                .attr("id", "VOTE")
                .attr("t", "123");
            let key = format!("received:{}:VOTE", voter.to_non_ad());
            let result = store.write(|tx| publish(tx, key.as_bytes(), &node, &payload));
            let consumer = ConsumerId::new("fixture").unwrap();
            if matches!(case, "invalid_chat" | "invalid_map") {
                assert!(result.is_err(), "{case}");
                let error = result.unwrap_err().to_string();
                assert!(!error.contains("private"), "{case}");
                assert!(
                    store
                        .read_events(&consumer, EventCursor::START, 10)
                        .unwrap()
                        .events
                        .is_empty()
                );
            } else {
                assert!(result.unwrap(), "{case}");
                let events = store
                    .read_events(&consumer, EventCursor::START, 10)
                    .unwrap()
                    .events;
                let event = &events[0].event;
                if matches!(
                    case,
                    "same_pn" | "same_lid" | "mapped" | "reverse" | "zero_ad"
                ) {
                    let SyncEvent::MessageUpdated(update) = event else {
                        panic!("{case}: missing vote")
                    };
                    assert_eq!(update.key.chat.jid(), &original, "{case}");
                    assert_eq!(update.key.direction, Direction::Outgoing);
                    assert_eq!(update.key.id.as_str(), "ORIGINAL");
                    assert_eq!(update.key.sender, None);
                    assert_eq!(
                        update.change,
                        MessageChange::PollVote {
                            by: DeviceAddress::from_jid(&voter).unwrap(),
                            selected: vec!["No".into()],
                            unresolved: vec![]
                        }
                    );
                } else {
                    assert!(!matches!(event, SyncEvent::MessageUpdated(_)), "{case}");
                }
                drop(store);
                let reopened = open(false);
                assert!(
                    !reopened
                        .write(|tx| publish(tx, key.as_bytes(), &node, &payload))
                        .unwrap(),
                    "{case}"
                );
                assert_eq!(
                    reopened
                        .read_events(&consumer, EventCursor::START, 10)
                        .unwrap()
                        .events
                        .len(),
                    1
                );
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 50))
                        .unwrap(),
                    before,
                    "{case}"
                );
                continue;
            }
            assert_eq!(
                store.read(|tx| tx.scan(Table::Checkpoints, 1, 50)).unwrap(),
                before,
                "{case}"
            );
        }
    }
}
