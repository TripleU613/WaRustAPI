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
//! `fromMe` is relative to the sender of the reaction: a contact reacting to a
//! message they sent uses `fromMe=true`; reacting to ours uses `fromMe=false`.
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

/// Encode the E2E payload for reacting to `target` in `chat`.
pub fn payload(
    chat: &Jid,
    target: &MessageRef,
    emoji: Option<&str>,
    sender_timestamp_ms: i64,
) -> Result<Vec<u8>> {
    validate(chat, target, emoji)?;
    let envelope = Envelope {
        reaction: Some(ReactionMessage {
            key: Some(Key {
                remote_jid: Some(chat.to_non_ad().to_string()),
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

/// Decode a reaction payload whose top-level fields are already known to be
/// `46` (plus optional metadata `35`). Returns `None` for malformed content.
pub fn decode(payload: &[u8]) -> Option<Incoming> {
    let envelope = Envelope::decode(payload).ok()?;
    let reaction = envelope.reaction?;
    let key = reaction.key?;
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
        let ours = payload(&peer, &target(Direction::Outgoing), None, 1).unwrap();
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
