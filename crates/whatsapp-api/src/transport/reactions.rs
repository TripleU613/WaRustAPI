//! Reaction payloads, traced from comparison APK **2.26.37.71** protobuf-lite
//! descriptors: `LX/BWI.dynamicMethod` (ReactionMessage: key=1 message,
//! text=2 string, groupingKey=3 string, senderTimestampMs=4 int64) and
//! `LX/BZa.dynamicMethod` (MessageKey: remoteJid=1 string, fromMe=2 bool,
//! id=3 string, participant=4 string). The stanza `type` attribute value
//! `reaction` comes from the message-kind table in `LX/0zO.A01` (`2bec`) and
//! was observed on a live incoming reaction (`message type=reaction` with `enc`
//! and `meta` children). The top-level `Message.reactionMessage` field number
//! (46) was confirmed against that live authenticated payload.
//!
//! In direct chats, `fromMe` is relative to the reacting contact: a reaction to
//! their message uses true, to ours false. Groups also require the explicit
//! original participant because a different member may be the author.
//! An empty `text` removes a previous reaction.
use crate::{
    Error, Result,
    model::{
        ids::MessageId,
        jid::Jid,
        message::{Direction, MessageRef},
    },
};
use prost::Message as _;

pub(super) fn key_failure(key: &Key) -> Option<&'static str> {
    // APK 37.71 C7G.A00 requires present/nonempty id and remoteJid, then
    // 1xf.A0k -> 0D7.A02 requires a nullable ChatJid parse to succeed.
    // Our supported JID categories/bounds remain local, not full native parity.
    let Some(id) = key.id.as_deref() else {
        return Some("missing_id");
    };
    if MessageId::new(id).is_err() {
        return Some("invalid_id");
    }
    let Some(remote) = key.remote_jid.as_deref() else {
        return Some("missing_remote_jid");
    };
    let Ok(remote) = Jid::parse(remote) else {
        return Some("invalid_remote_jid");
    };
    if remote.user.is_empty() || matches!(remote.server, crate::model::jid::Server::Other(_)) {
        return Some("unsupported_remote_jid");
    }
    None
}

/// Longest accepted reaction text in bytes (local bound; emoji sequences).
pub const MAX_TEXT_BYTES: usize = 64;

#[derive(prost::Message)]
pub(crate) struct Envelope {
    #[prost(message, optional, tag = "46")]
    pub(crate) reaction: Option<ReactionMessage>,
}
#[derive(prost::Message)]
pub(crate) struct ReactionMessage {
    #[prost(message, optional, tag = "1")]
    pub(crate) key: Option<Key>,
    #[prost(string, optional, tag = "2")]
    pub(crate) text: Option<String>,
    #[prost(string, optional, tag = "3")]
    pub(crate) grouping_key: Option<String>,
    #[prost(int64, optional, tag = "4")]
    pub(crate) sender_timestamp_ms: Option<i64>,
}
#[derive(prost::Message)]
pub(crate) struct Key {
    #[prost(string, optional, tag = "1")]
    pub(crate) remote_jid: Option<String>,
    #[prost(bool, optional, tag = "2")]
    pub(crate) from_me: Option<bool>,
    #[prost(string, optional, tag = "3")]
    pub(crate) id: Option<String>,
    #[prost(string, optional, tag = "4")]
    pub(crate) participant: Option<String>,
}

fn invalid(reason: &'static str) -> Error {
    Error::invalid("reaction", reason)
}

/// Validate a reaction intent for a primary direct chat.
pub fn validate(chat: &Jid, target: &MessageRef, emoji: Option<&str>) -> Result<()> {
    if target.key.chat.jid().to_non_ad() != chat.to_non_ad() {
        return Err(invalid("target message belongs to another chat"));
    }
    if let Some(text) = emoji
        && (text.is_empty() || text.len() > MAX_TEXT_BYTES || text.chars().any(char::is_control))
    {
        return Err(invalid(
            "reaction text must be 1..=64 bytes without control characters",
        ));
    }
    Ok(())
}

/// Encode the E2E payload for reacting to `target` in `chat`. `remote` is the
/// identity to place in `key.remoteJid`: the contact's LID when their messages
/// arrive LID-addressed (observed: the official client's reaction key used a
/// `@lid` remoteJid), otherwise the phone-number chat JID.
pub fn payload(
    chat: &Jid,
    remote: &Jid,
    target: &MessageRef,
    emoji: Option<&str>,
    sender_timestamp_ms: i64,
) -> Result<Vec<u8>> {
    validate(chat, target, emoji)?;
    let envelope = Envelope {
        reaction: Some(ReactionMessage {
            key: Some(Key {
                remote_jid: Some(remote.to_non_ad().to_string()),
                from_me: Some(target.key.direction == Direction::Outgoing),
                id: Some(target.key.id.as_str().to_owned()),
                participant: None,
            }),
            text: Some(emoji.unwrap_or("").to_owned()),
            grouping_key: None,
            sender_timestamp_ms: Some(sender_timestamp_ms),
        }),
    };
    Ok(envelope.encode_to_vec())
}

/// Encode a reaction to a message in a **group or status**, where the key must
/// name the original sender.
///
/// A direct reaction's key identifies the message by chat and id alone, because
/// there is only one other party. In a group the same id is ambiguous without
/// `participant`, which is exactly the field the receive path reads to attribute
/// a reaction — and the one the direct builder deliberately leaves unset.
///
/// `from_me` is the *reacting* side's view of the target: true when we are
/// reacting to our own message.
pub fn group_payload(
    chat: &Jid,
    participant: &Jid,
    target_id: &str,
    from_me: bool,
    emoji: Option<&str>,
    sender_timestamp_ms: i64,
) -> Result<Vec<u8>> {
    if !matches!(
        chat.server,
        crate::model::jid::Server::Group | crate::model::jid::Server::Broadcast
    ) {
        return Err(invalid("a group reaction targets a group or broadcast"));
    }
    if target_id.is_empty() || target_id.len() > 128 {
        return Err(invalid("invalid target message id"));
    }
    if let Some(text) = emoji
        && (text.is_empty() || text.len() > MAX_TEXT_BYTES || text.chars().any(char::is_control))
    {
        return Err(invalid(
            "reaction text must be 1..=64 bytes without control characters",
        ));
    }
    Ok(Envelope {
        reaction: Some(ReactionMessage {
            key: Some(Key {
                remote_jid: Some(chat.to_non_ad().to_string()),
                from_me: Some(from_me),
                id: Some(target_id.to_owned()),
                participant: Some(participant.to_non_ad().to_string()),
            }),
            text: Some(emoji.unwrap_or("").to_owned()),
            grouping_key: None,
            sender_timestamp_ms: Some(sender_timestamp_ms),
        }),
    }
    .encode_to_vec())
}

/// A decoded incoming reaction, before mapping into the event model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incoming {
    /// Wire id of the reacted-to message.
    pub target_id: MessageId,
    /// Whether the reacted-to message was sent by the reacting party.
    pub target_from_sender: bool,
    /// Reaction text; `None` removes a reaction.
    pub emoji: Option<String>,
}

/// Diagnostic: every field number present in a reaction payload, at the
/// `Message`, `ReactionMessage` and `MessageKey` levels, plus the participant's
/// server and whether it carries a device suffix. No identities or text.
pub fn shape(payload: &[u8]) -> Option<String> {
    /// A field number and, for length-delimited fields, its body.
    type Field<'a> = (u32, Option<&'a [u8]>);
    fn fields(bytes: &[u8]) -> Option<Vec<Field<'_>>> {
        let mut out = Vec::new();
        let mut buf = bytes;
        while !buf.is_empty() {
            let tag = prost::encoding::decode_varint(&mut buf).ok()?;
            let number = (tag >> 3) as u32;
            match tag & 7 {
                0 => {
                    prost::encoding::decode_varint(&mut buf).ok()?;
                    out.push((number, None));
                }
                1 => {
                    buf = buf.get(8..)?;
                    out.push((number, None));
                }
                2 => {
                    let len = prost::encoding::decode_varint(&mut buf).ok()? as usize;
                    let (body, rest) = (buf.get(..len)?, buf.get(len..)?);
                    out.push((number, Some(body)));
                    buf = rest;
                }
                5 => {
                    buf = buf.get(4..)?;
                    out.push((number, None));
                }
                _ => return None,
            }
        }
        Some(out)
    }
    let top = fields(payload)?;
    let reaction = top.iter().find(|(n, _)| *n == 46)?.1?;
    let inner = fields(reaction)?;
    let key = inner.iter().find(|(n, _)| *n == 1).and_then(|(_, b)| *b);
    let key_fields = key.and_then(fields).unwrap_or_default();
    let participant = key_fields
        .iter()
        .find(|(n, _)| *n == 4)
        .and_then(|(_, b)| std::str::from_utf8((*b)?).ok())
        .map(|p| {
            let server = p.rsplit_once('@').map_or("", |(_, s)| s);
            format!("{server}{}", if p.contains(':') { "+device" } else { "" })
        });
    let nums = |v: &[Field<'_>]| v.iter().map(|(n, _)| *n).collect::<Vec<_>>();
    Some(format!(
        "message={:?} reaction={:?} key={:?} participant={:?} key_validation={}",
        nums(&top),
        nums(&inner),
        nums(&key_fields),
        participant,
        Envelope::decode(payload)
            .ok()
            .and_then(|envelope| envelope.reaction)
            .map_or("malformed_reaction", |reaction| reaction
                .key
                .as_ref()
                .map_or("missing_key", |key| key_failure(key)
                    .unwrap_or("valid")))
    ))
}

/// Diagnostic: the key's `remoteJid` string and whether `participant` is set.
pub fn key_fields(payload: &[u8]) -> Option<(Option<String>, bool)> {
    let key = Envelope::decode(payload).ok()?.reaction?.key?;
    Some((key.remote_jid, key.participant.is_some()))
}

/// Resolve a group reaction's explicit original author. APK 37.71
/// Bal.A00 reads BZa.participant_ (003a..0046); fromMe alone cannot identify
/// a third member. Own classification here is a local model projection, not
/// a port of the native own/hosted canonicalizer.
pub fn group_target(
    chat: &Jid,
    payload: &[u8],
    own: &[Jid],
) -> Result<crate::model::message::MessageKey> {
    use crate::model::{
        ids::{ChatId, DeviceAddress},
        jid::Server,
        message::MessageKey,
    };
    if chat.server != Server::Group {
        return Err(invalid("not_group_chat"));
    }
    let reaction = Envelope::decode(payload)
        .map_err(|_| invalid("malformed_reaction"))?
        .reaction
        .ok_or_else(|| invalid("missing_reaction"))?;
    let key = reaction.key.ok_or_else(|| invalid("missing_key"))?;
    if let Some(reason) = key_failure(&key) {
        return Err(invalid(reason));
    }
    let remote = Jid::parse(key.remote_jid.as_deref().unwrap())?;
    if remote.to_non_ad() != chat.to_non_ad() {
        return Err(invalid("group_chat_mismatch"));
    }
    let participant = key
        .participant
        .as_deref()
        .ok_or_else(|| invalid("missing_participant"))?;
    let participant = Jid::parse(participant).map_err(|_| invalid("invalid_participant"))?;
    if participant.user.is_empty() {
        return Err(invalid("invalid_participant"));
    }
    if !matches!(participant.server, Server::User | Server::Lid) {
        return Err(invalid("unsupported_participant"));
    }
    if participant.agent != 0 {
        return Err(invalid("unsupported_participant_agent"));
    }
    let valid_own = |o: &Jid| {
        o.agent == 0 && !o.user.is_empty() && matches!(o.server, Server::User | Server::Lid)
    };
    if !own.iter().any(valid_own) {
        return Err(invalid("own_identity_missing"));
    }
    let ours = own
        .iter()
        .filter(|o| valid_own(o))
        .any(|o| o.to_non_ad() == participant.to_non_ad());
    Ok(MessageKey {
        chat: ChatId::from_jid(chat)?,
        sender: if ours {
            None
        } else {
            Some(DeviceAddress::from_jid(&participant.to_non_ad())?)
        },
        direction: if ours {
            Direction::Outgoing
        } else {
            Direction::Incoming
        },
        id: MessageId::new(key.id.unwrap())?,
    })
}

/// Decode a reaction payload whose top-level fields are already known to be
/// `46` (plus optional metadata `35`). Returns `None` for malformed content.
pub fn decode(payload: &[u8]) -> Option<Incoming> {
    let envelope = Envelope::decode(payload).ok()?;
    let reaction = envelope.reaction?;
    let key = reaction.key?;
    if key_failure(&key).is_some() {
        return None;
    }
    let target_id = MessageId::new(key.id?).ok()?;
    let text = reaction.text.unwrap_or_default();
    if text.len() > MAX_TEXT_BYTES || text.chars().any(char::is_control) {
        return None;
    }
    Some(Incoming {
        target_id,
        target_from_sender: key.from_me.unwrap_or(false),
        emoji: (!text.is_empty()).then_some(text),
    })
}

#[cfg(test)]
mod group_tests {
    use super::*;

    #[test]
    fn group_target_reads_literal_participant_fields_and_refuses_unknown_scope() {
        // Independent descriptor-grounded bytes: BZa 1/2/3/4 inside BWI.key=1,
        // Message.reactionMessage=46. This does not call our payload builder.
        fn fixture(remote: &str, participant: Option<&str>, from_me: Option<bool>) -> Vec<u8> {
            let mut key = vec![0x0a, remote.len() as u8];
            key.extend_from_slice(remote.as_bytes());
            if let Some(v) = from_me {
                key.extend_from_slice(&[0x10, u8::from(v)]);
            }
            key.extend_from_slice(b"\x1a\x06TARGET");
            if let Some(p) = participant {
                key.extend_from_slice(&[0x22, p.len() as u8]);
                key.extend_from_slice(p.as_bytes());
            }
            let mut reaction = vec![0x0a, key.len() as u8];
            reaction.extend(key);
            reaction.extend_from_slice(b"\x12\x00");
            let mut message = vec![0xf2, 0x02, reaction.len() as u8];
            message.extend(reaction);
            message
        }
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let own = [
            Jid::parse("15555550199@s.whatsapp.net").unwrap(),
            Jid::parse("222222222222222@lid").unwrap(),
        ];
        let author = Jid::parse("15555550101@lid").unwrap();
        for flag in [None, Some(false), Some(true)] {
            let bytes = fixture(&chat.to_string(), Some(&author.to_string()), flag);
            let key = group_target(&chat, &bytes, &own).unwrap();
            assert_eq!(key.direction, Direction::Incoming);
            assert_eq!(key.sender.unwrap().to_jid(), author);
            assert_eq!(key.id.as_str(), "TARGET");
            assert!(decode(&bytes).unwrap().emoji.is_none());
        }
        for author in &own {
            let key = group_target(
                &chat,
                &fixture(&chat.to_string(), Some(&author.to_string()), Some(false)),
                &own,
            )
            .unwrap();
            assert_eq!(key.direction, Direction::Outgoing);
            assert!(key.sender.is_none());
        }
        for (p, reason) in [
            (None, "missing_participant"),
            (Some(""), "invalid_participant"),
            (Some("not-a-jid"), "invalid_participant"),
            (Some("s.whatsapp.net"), "invalid_participant"),
            (Some("status@broadcast"), "unsupported_participant"),
            (Some("123@g.us"), "unsupported_participant"),
        ] {
            assert!(
                matches!(group_target(&chat,&fixture(&chat.to_string(),p,Some(false)),&own),Err(Error::InvalidInput {reason:r,..}) if r==reason)
            );
        }
        let agent = Jid::new_ad("15555550101", 2, 3, crate::model::jid::Server::User).unwrap();
        assert!(
            matches!(group_target(&chat,&fixture(&chat.to_string(),Some(&agent.to_string()),None),&own),Err(Error::InvalidInput {reason,..}) if reason=="unsupported_participant_agent")
        );
        assert!(
            matches!(group_target(&chat,&fixture("120363000000000002@g.us",Some(&author.to_string()),None),&own),Err(Error::InvalidInput {reason,..}) if reason=="group_chat_mismatch")
        );
        assert!(
            matches!(group_target(&chat,&fixture(&chat.to_string(),Some(&author.to_string()),None),&[]),Err(Error::InvalidInput {reason,..}) if reason=="own_identity_missing")
        );
    }

    /// The participant is what makes a group reaction attributable: the same
    /// message id in a group means nothing without knowing whose message it was,
    /// and the direct builder deliberately leaves that field unset.
    #[test]
    fn a_group_reaction_names_the_original_sender() {
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let participant = Jid::parse("222222222222222@lid").unwrap();
        let bytes =
            group_payload(&chat, &participant, "TARGET1", false, Some("\u{1f44d}"), 42).unwrap();
        let (remote, has_participant) = key_fields(&bytes).unwrap();
        assert_eq!(remote.as_deref(), Some("120363000000000001@g.us"));
        assert!(has_participant, "a group reaction must carry participant");
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded.target_id.as_str(), "TARGET1");
        assert_eq!(decoded.emoji.as_deref(), Some("\u{1f44d}"));
        assert!(!decoded.target_from_sender);

        // Removing a reaction is an empty text, which decodes to no emoji.
        let cleared = group_payload(&chat, &participant, "TARGET1", false, None, 42).unwrap();
        assert!(decode(&cleared).unwrap().emoji.is_none());
    }

    /// A direct chat is not a group: the key shape differs and using the group
    /// builder there would put a participant where none belongs.
    #[test]
    fn group_payload_refuses_a_direct_chat_and_bad_input() {
        let direct = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let participant = Jid::parse("222222222222222@lid").unwrap();
        assert!(group_payload(&direct, &participant, "T", false, None, 1).is_err());
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        assert!(group_payload(&chat, &participant, "", false, None, 1).is_err());
        assert!(group_payload(&chat, &participant, "T", false, Some(""), 1).is_err());
        assert!(
            group_payload(&chat, &participant, "T", false, Some("a\u{7}b"), 1).is_err(),
            "control characters are refused"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        ids::{ChatId, DeviceAddress},
        message::MessageKey,
    };
    fn target(direction: Direction) -> MessageRef {
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(&peer).unwrap(),
                sender: (direction == Direction::Incoming)
                    .then(|| DeviceAddress::from_jid(&peer).unwrap()),
                direction,
                id: MessageId::new("ABCDEF0123456789ABCDEF0123456789").unwrap(),
            },
        }
    }
    #[test]
    fn payload_roundtrips_with_apk_field_numbers_and_sender_relative_from_me() {
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let bytes = payload(
            &peer,
            &peer,
            &target(Direction::Incoming),
            Some("👍"),
            1_700_000_000_000,
        )
        .unwrap();
        // Field 46, wire type 2: key (46 << 3) | 2 = 370 = varint F2 02.
        assert_eq!(&bytes[..2], &[0xF2, 0x02]);
        let envelope = Envelope::decode(bytes.as_slice()).unwrap();
        let reaction = envelope.reaction.unwrap();
        let key = reaction.key.as_ref().unwrap();
        assert_eq!(
            key.remote_jid.as_deref(),
            Some("15555550101@s.whatsapp.net")
        );
        assert_eq!(key.from_me, Some(false), "reacting to their message");
        assert_eq!(key.id.as_deref(), Some("ABCDEF0123456789ABCDEF0123456789"));
        assert_eq!(reaction.text.as_deref(), Some("👍"));
        assert_eq!(reaction.sender_timestamp_ms, Some(1_700_000_000_000));
        let lid = Jid::parse("123456789012345@lid").unwrap();
        let ours = payload(&peer, &lid, &target(Direction::Outgoing), None, 1).unwrap();
        assert_eq!(
            Envelope::decode(ours.as_slice())
                .unwrap()
                .reaction
                .unwrap()
                .key
                .unwrap()
                .remote_jid
                .as_deref(),
            Some("123456789012345@lid")
        );
        let reaction = Envelope::decode(ours.as_slice()).unwrap().reaction.unwrap();
        assert_eq!(
            reaction.key.unwrap().from_me,
            Some(true),
            "reacting to our own message"
        );
        assert_eq!(
            reaction.text.as_deref(),
            Some(""),
            "removal is an empty text"
        );
        // Decoding mirrors the sender-relative flag.
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded.emoji.as_deref(), Some("👍"));
        assert!(!decoded.target_from_sender);
        let removed = decode(&ours).unwrap();
        assert_eq!(removed.emoji, None);
        assert!(removed.target_from_sender);
        assert!(decode(&[0x08, 0x01]).is_none());
    }
    #[test]
    fn validation_rejects_other_chats_and_bad_text() {
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let other = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        assert!(validate(&other, &target(Direction::Incoming), Some("x")).is_err());
        assert!(validate(&peer, &target(Direction::Incoming), Some("")).is_err());
        assert!(validate(&peer, &target(Direction::Incoming), Some(&"x".repeat(65))).is_err());
        assert!(validate(&peer, &target(Direction::Incoming), Some("a\nb")).is_err());
        assert!(validate(&peer, &target(Direction::Incoming), Some("❤️")).is_ok());
        assert!(validate(&peer, &target(Direction::Incoming), None).is_ok());
    }
}

#[cfg(test)]
mod key_validation_tests {
    use super::*;
    fn literal(remote: Option<&str>, id: Option<&str>) -> Vec<u8> {
        // Independent field layout from BWI/BZa.dynamicMethod in APK 37.71.
        let mut key = Vec::new();
        for (tag, value) in [(0x0a, remote), (0x1a, id)] {
            if let Some(value) = value {
                key.push(tag);
                prost::encoding::encode_varint(value.len() as u64, &mut key);
                key.extend_from_slice(value.as_bytes());
            }
        }
        let mut reaction = vec![0x0a];
        prost::encoding::encode_varint(key.len() as u64, &mut reaction);
        reaction.extend(key);
        let mut payload = vec![0xf2, 0x02];
        prost::encoding::encode_varint(reaction.len() as u64, &mut payload);
        payload.extend(reaction);
        payload
    }
    #[test]
    fn apk_reaction_key_requires_id_and_supported_parseable_remote_without_echoing_it() {
        for (remote, id, failure) in [
            (None, Some("TARGET"), "missing_remote_jid"),
            (Some(""), Some("TARGET"), "invalid_remote_jid"),
            (
                Some("private invalid address"),
                Some("TARGET"),
                "invalid_remote_jid",
            ),
            (
                Some("private@unknown.example"),
                Some("TARGET"),
                "unsupported_remote_jid",
            ),
            (
                Some("s.whatsapp.net"),
                Some("TARGET"),
                "unsupported_remote_jid",
            ),
            (Some("12345@lid"), None, "missing_id"),
            (Some("12345@lid"), Some(""), "invalid_id"),
        ] {
            let payload = literal(remote, id);
            assert!(decode(&payload).is_none());
            let diagnostic = shape(&payload).unwrap();
            assert!(diagnostic.ends_with(&format!("key_validation={failure}")));
            assert!(!diagnostic.contains("private"));
            assert!(!diagnostic.contains("TARGET"));
        }
        for remote in [
            "12345@lid",
            "15555550101@s.whatsapp.net",
            "12345:3@lid",
            "120363000000000001@g.us",
            "status@broadcast",
        ] {
            let payload = literal(Some(remote), Some("TARGET"));
            let decoded = decode(&payload).unwrap();
            assert_eq!(decoded.target_id.as_str(), "TARGET");
            assert!(!decoded.target_from_sender); // APK boolean getter defaults false.
            assert_eq!(decoded.emoji, None); // A missing/empty text is removal.
            assert!(shape(&payload).unwrap().ends_with("key_validation=valid"));
        }
    }
}
