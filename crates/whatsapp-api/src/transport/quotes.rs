//! Quoted replies: `extendedTextMessage` with a `contextInfo` that names the
//! message being replied to (docs/protocol/messaging-transport.md, "Quoted replies").
//!
//! Field numbers are from the comparison build 2.26.37.71 protobuf-lite
//! descriptors: `Message.extendedTextMessage` = 6 (`LX/BVL`), `ExtendedTextMessage.text` = 1
//! and `.contextInfo` = 17, `ContextInfo.stanzaId` = 1, `.participant` = 2,
//! `.quotedMessage` = 3 (`LX/BVK`), `.expiration` = 25 (`LX/BWx`).

use prost::Message;

#[derive(Clone, PartialEq, prost::Message)]
struct Envelope {
    #[prost(string, optional, tag = "1")]
    conversation: Option<String>,
    #[prost(message, optional, tag = "6")]
    extended: Option<Extended>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct Extended {
    #[prost(string, optional, tag = "1")]
    text: Option<String>,
    #[prost(message, optional, tag = "17")]
    context: Option<Context>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct Context {
    #[prost(string, optional, tag = "1")]
    stanza_id: Option<String>,
    #[prost(string, optional, tag = "2")]
    participant: Option<String>,
    #[prost(bytes = "vec", optional, tag = "3")]
    quoted: Option<Vec<u8>>,
    #[prost(uint32, optional, tag = "25")]
    expiration: Option<u32>,
}

/// A decoded text message together with its optional quote context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotedText {
    /// Message text.
    pub text: String,
    /// Disappearing-message expiration, when non-zero.
    pub expiration: Option<u32>,
    /// `contextInfo.stanzaId`: the wire id of the message replied to.
    pub stanza_id: Option<String>,
    /// `contextInfo.participant`: the JID of the quoted message's sender.
    pub participant: Option<String>,
    /// `contextInfo.quotedMessage`: the quoted `Message`, encoded.
    pub quoted: Option<Vec<u8>>,
}

impl QuotedText {
    /// Whether any quote field is present.
    pub fn is_reply(&self) -> bool {
        self.stanza_id.is_some() || self.quoted.is_some()
    }
}

/// Decode a `conversation` or `extendedTextMessage` payload. Other fields are
/// ignored; callers gate on the top-level field set first.
pub fn decode(payload: &[u8]) -> Option<QuotedText> {
    let envelope = Envelope::decode(payload).ok()?;
    match (envelope.conversation, envelope.extended) {
        (Some(text), None) => Some(QuotedText {
            text,
            expiration: None,
            stanza_id: None,
            participant: None,
            quoted: None,
        }),
        (
            None,
            Some(Extended {
                text: Some(text),
                context,
            }),
        ) => {
            let context = context.unwrap_or_default();
            Some(QuotedText {
                text,
                expiration: context.expiration.filter(|v| *v != 0),
                stanza_id: context.stanza_id,
                participant: context.participant,
                quoted: context.quoted,
            })
        }
        _ => None,
    }
}

/// Encode a quoted reply: `extendedTextMessage{text, contextInfo{stanzaId,
/// participant, quotedMessage}}`. `quoted` is the referenced `Message`, already
/// encoded (a `conversation` text when the original is not at hand).
pub fn reply_payload(text: &str, stanza_id: &str, participant: &str, quoted: Vec<u8>) -> Vec<u8> {
    Envelope {
        conversation: None,
        extended: Some(Extended {
            text: Some(text.to_owned()),
            context: Some(Context {
                stanza_id: Some(stanza_id.to_owned()),
                participant: Some(participant.to_owned()),
                quoted: Some(quoted),
                expiration: None,
            }),
        }),
    }
    .encode_to_vec()
}

/// Encode a plain `Message{conversation}` for use as a `quotedMessage`.
pub fn conversation(text: &str) -> Vec<u8> {
    Envelope {
        conversation: Some(text.to_owned()),
        extended: None,
    }
    .encode_to_vec()
}

/// Strip `messageContextInfo` (35) from a message before quoting it, so a
/// quoted poll never re-sends its message secret.
pub fn strip_message_context(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len());
    let mut buf = payload;
    while !buf.is_empty() {
        let start = buf;
        let Ok(key) = prost::encoding::decode_varint(&mut buf) else {
            return payload.to_vec();
        };
        let field = (key >> 3) as u32;
        let Ok(wire_type) = prost::encoding::WireType::try_from(key & 7) else {
            return payload.to_vec();
        };
        if prost::encoding::skip_field(
            wire_type,
            field,
            &mut buf,
            prost::encoding::DecodeContext::default(),
        )
        .is_err()
        {
            return payload.to_vec();
        }
        if field != 35 {
            out.extend_from_slice(&start[..start.len() - buf.len()]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_reply() {
        let payload = reply_payload("yes", "ABC", "1@lid", conversation("hi"));
        assert_eq!(payload[0], 0x32, "extendedTextMessage is field 6");
        let decoded = decode(&payload).unwrap();
        assert_eq!(decoded.text, "yes");
        assert_eq!(decoded.stanza_id.as_deref(), Some("ABC"));
        assert_eq!(decoded.participant.as_deref(), Some("1@lid"));
        assert_eq!(
            decoded.quoted.as_deref(),
            Some(conversation("hi").as_slice())
        );
        assert!(decoded.is_reply());
        assert!(!decode(&conversation("plain")).unwrap().is_reply());
    }

    #[test]
    fn strips_message_context_only() {
        let mut poll = vec![0x9a, 0x02, 0x02, 0x1a, 0x00]; // field 35 {field 3 empty}
        poll.extend_from_slice(&conversation("x"));
        assert_eq!(strip_message_context(&poll), conversation("x"));
        assert_eq!(strip_message_context(&conversation("x")), conversation("x"));
    }
}
