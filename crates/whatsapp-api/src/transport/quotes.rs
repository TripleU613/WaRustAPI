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
    // Styling for a coloured full-screen text status. Field numbers and wire
    // types read from the `LX/BZF` descriptor in comparison 2.26.37.71, not
    // from the public schema: `textArgb` 7 and `backgroundArgb` 8 are
    // **FIXED32** (type char `\u{1006}`, `& 0xFF` = 6 = FIXED32), so they are
    // fixed 4-byte little-endian, *not* varint int32. Decoding them as int32
    // would misread the wire rather than merely confuse the sign of the alpha
    // channel.
    #[prost(fixed32, optional, tag = "7")]
    text_argb: Option<u32>,
    #[prost(fixed32, optional, tag = "8")]
    background_argb: Option<u32>,
    #[prost(int32, optional, tag = "9")]
    font: Option<i32>,
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

/// Font of a styled text status (`LX/C44`, **confirmed**: `forNumber`'s
/// packed switch covers exactly 0..=8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StatusFont {
    /// 0
    SansSerif,
    /// 1
    Serif,
    /// 2
    FacebookScriptRegular,
    /// 3
    BebasNeueRegular,
    /// 4
    SystemBold,
    /// 5
    MorningBreezeRegular,
    /// 6
    CalistogaRegular,
    /// 7
    Exo2ExtraBold,
    /// 8
    CourierPrimeBold,
}

impl StatusFont {
    /// Map a wire value, refusing anything outside the enumerated range rather
    /// than inventing a font for it.
    pub fn from_wire(value: i32) -> Option<Self> {
        Some(match value {
            0 => Self::SansSerif,
            1 => Self::Serif,
            2 => Self::FacebookScriptRegular,
            3 => Self::BebasNeueRegular,
            4 => Self::SystemBold,
            5 => Self::MorningBreezeRegular,
            6 => Self::CalistogaRegular,
            7 => Self::Exo2ExtraBold,
            8 => Self::CourierPrimeBold,
            _ => return None,
        })
    }
}

/// Presentation of a coloured full-screen text status. Absent on an ordinary
/// chat message, which carries none of these fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct TextStyle {
    /// `backgroundArgb`, ARGB with the alpha byte most significant.
    pub background_argb: Option<u32>,
    /// `textArgb`, ARGB with the alpha byte most significant.
    pub text_argb: Option<u32>,
    /// `font`, when it names a font this build enumerates.
    pub font: Option<StatusFont>,
}

impl TextStyle {
    /// Whether any styling was present.
    pub fn is_styled(&self) -> bool {
        self.background_argb.is_some() || self.text_argb.is_some() || self.font.is_some()
    }
}

/// A decoded text message together with its optional quote context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotedText {
    /// Message text.
    pub text: String,
    /// Colour and font of a styled text status.
    pub style: TextStyle,
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
        // A bare `conversation` carries no styling; only extendedTextMessage does.
        (Some(text), None) => Some(QuotedText {
            text,
            style: TextStyle::default(),
            expiration: None,
            stanza_id: None,
            participant: None,
            quoted: None,
        }),
        (
            None,
            Some(Extended {
                text: Some(text),
                text_argb,
                background_argb,
                font,
                context,
            }),
        ) => {
            let context = context.unwrap_or_default();
            Some(QuotedText {
                text,
                style: TextStyle {
                    background_argb,
                    text_argb,
                    // An unrecognized font value is dropped rather than guessed
                    // at: this build enumerates exactly 0..=8.
                    font: font.and_then(StatusFont::from_wire),
                },
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
            // A quoted reply is an ordinary chat message, not a styled status.
            text_argb: None,
            background_argb: None,
            font: None,
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

/// Add a quote to an encoded media `Message`: `contextInfo` (17) on its
/// `imageMessage` (3) or `audioMessage` (8), the comparison `BWp`/`BWZ` field
/// the receive path already reads (`transport::media`), with the same `BWx`
/// stanzaId/participant/quotedMessage as a text reply. Every other field is
/// copied byte for byte. Anything else refuses rather than guessing where the
/// quote belongs: the view-once wrapper (55, quote semantics untraced), not
/// exactly one media field, or a media message that already has a context.
pub fn media_reply_payload(
    payload: &[u8],
    stanza_id: &str,
    participant: &str,
    quoted: Vec<u8>,
) -> crate::Result<Vec<u8>> {
    use prost::encoding::{WireType, decode_varint, encode_key, encode_varint};
    let invalid = |reason| crate::Error::invalid("reply", reason);
    let context = Context {
        stanza_id: Some(stanza_id.to_owned()),
        participant: Some(participant.to_owned()),
        quoted: Some(quoted),
        expiration: None,
    }
    .encode_to_vec();
    let mut out = Vec::with_capacity(payload.len() + context.len() + 8);
    let mut media = 0;
    let mut buf = payload;
    while !buf.is_empty() {
        let start = buf;
        let (field, wire) = field_key(&mut buf).ok_or_else(|| invalid("invalid media payload"))?;
        if field == 55 {
            return Err(invalid("a view-once message cannot carry a quote here"));
        }
        if matches!(field, 3 | 8) && wire == WireType::LengthDelimited {
            let length = decode_varint(&mut buf).map_err(|_| invalid("invalid media payload"))?;
            let length = usize::try_from(length)
                .ok()
                .filter(|length| *length <= buf.len())
                .ok_or_else(|| invalid("invalid media payload"))?;
            let (inner, rest) = buf.split_at(length);
            buf = rest;
            if has_field(inner, 17).ok_or_else(|| invalid("invalid media payload"))? {
                return Err(invalid("media message already carries a context"));
            }
            let mut body = Vec::with_capacity(inner.len() + context.len() + 4);
            body.extend_from_slice(inner);
            encode_key(17, WireType::LengthDelimited, &mut body);
            encode_varint(context.len() as u64, &mut body);
            body.extend_from_slice(&context);
            encode_key(field, WireType::LengthDelimited, &mut out);
            encode_varint(body.len() as u64, &mut out);
            out.extend_from_slice(&body);
            media += 1;
        } else {
            prost::encoding::skip_field(
                wire,
                field,
                &mut buf,
                prost::encoding::DecodeContext::default(),
            )
            .map_err(|_| invalid("invalid media payload"))?;
            out.extend_from_slice(&start[..start.len() - buf.len()]);
        }
    }
    if media != 1 {
        return Err(invalid("expected exactly one image or audio message"));
    }
    Ok(out)
}

/// Whether `media_reply_payload` can place a quote in this payload. Checked
/// at selection, before any network reservation, so an unsupported shape
/// fails locally instead of after a key query.
pub fn media_quote_supported(payload: &[u8]) -> bool {
    media_reply_payload(payload, "", "", Vec::new()).is_ok()
}

fn field_key(buf: &mut &[u8]) -> Option<(u32, prost::encoding::WireType)> {
    let key = prost::encoding::decode_varint(buf).ok()?;
    let field = u32::try_from(key >> 3).ok().filter(|field| *field != 0)?;
    Some((field, prost::encoding::WireType::try_from(key & 7).ok()?))
}

/// Whether a well-formed encoded message carries `wanted`; `None` if malformed.
fn has_field(mut buf: &[u8], wanted: u32) -> Option<bool> {
    let mut found = false;
    while !buf.is_empty() {
        let (field, wire) = field_key(&mut buf)?;
        found |= field == wanted;
        prost::encoding::skip_field(
            wire,
            field,
            &mut buf,
            prost::encoding::DecodeContext::default(),
        )
        .ok()?;
    }
    Some(found)
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
    /// Payload assembled by hand from the `LX/BZF` descriptor, not by running
    /// the encoder: `extendedTextMessage{ text="Hi", textArgb=0xFFFFFFFF,
    /// backgroundArgb=0xFF25D366, font=3 }`. The two colours are written as
    /// FIXED32 (wire type 5, little-endian); reading them as varint int32 would
    /// fail to parse this at all, which is the point of pinning it.
    #[test]
    fn styled_status_text_decodes_colours_and_font() {
        let payload: &[u8] = &[
            0x32, 0x10, 0x0a, 0x02, 0x48, 0x69, 0x3d, 0xff, 0xff, 0xff, 0xff, 0x45, 0x66, 0xd3,
            0x25, 0xff, 0x48, 0x03,
        ];
        let decoded = decode(payload).expect("styled status must decode");
        assert_eq!(decoded.text, "Hi");
        assert_eq!(decoded.style.text_argb, Some(0xFFFF_FFFF));
        assert_eq!(decoded.style.background_argb, Some(0xFF25_D366));
        assert_eq!(decoded.style.font, Some(StatusFont::BebasNeueRegular));
        assert!(decoded.style.is_styled());
        // Alpha survives: a FIXED32 read keeps the top byte.
        assert_eq!(decoded.style.background_argb.unwrap() >> 24, 0xFF);
    }

    /// imageMessage{url="a"} and audioMessage{ptt=true}; the image fixture
    /// also has an unrelated outer field 35 that must survive byte for byte.
    const IMAGE: &[u8] = &[0x1a, 0x03, 0x0a, 0x01, 0x61, 0x9a, 0x02, 0x00];
    const AUDIO: &[u8] = &[0x42, 0x02, 0x30, 0x01];

    /// A media quote lands where the receive path (written from the APK
    /// descriptors) reads it, for image and audio. The parser exists only in
    /// the full comparison build.
    #[cfg(all(
        feature = "experimental-registration-https",
        feature = "experimental-signal-crypto",
        feature = "experimental-noise",
        feature = "experimental-transport"
    ))]
    #[test]
    fn media_reply_context_round_trips_through_the_receive_parser() {
        use super::super::media::quote_context;
        for payload in [IMAGE, AUDIO] {
            let out = media_reply_payload(payload, "ABC", "1@lid", conversation("parent")).unwrap();
            assert_eq!(
                quote_context(&out).unwrap(),
                Some(("ABC".to_owned(), Some("1@lid".to_owned())))
            );
        }
    }

    /// The original media fields stay a prefix of the rewritten body, other
    /// outer fields survive, and shapes with no single home for the quote refuse.
    #[test]
    fn media_reply_payload_keeps_fields_and_refuses_ambiguous_shapes() {
        for payload in [IMAGE, AUDIO] {
            let out = media_reply_payload(payload, "ABC", "1@lid", conversation("")).unwrap();
            let inner = payload[1] as usize;
            assert_eq!(out[0], payload[0]);
            assert_eq!(&out[2..2 + inner], &payload[2..2 + inner]);
            // contextInfo (17, length-delimited) follows the original fields.
            assert_eq!(&out[2 + inner..4 + inner], &[0x8a, 0x01]);
        }
        let out = media_reply_payload(IMAGE, "ABC", "1@lid", conversation("")).unwrap();
        assert!(out.ends_with(&[0x9a, 0x02, 0x00]));

        // Refusals: view-once wrapper, no or two media fields, existing
        // context, malformed bytes.
        let view_once: &[u8] = &[0xba, 0x03, 0x00];
        let two: &[u8] = &[0x1a, 0x00, 0x42, 0x00];
        let none: &[u8] = &[0x0a, 0x01, 0x61];
        let existing: &[u8] = &[0x1a, 0x03, 0x8a, 0x01, 0x00];
        let malformed: &[u8] = &[0x1a, 0x05, 0x0a];
        for (payload, reason) in [
            (view_once, "view-once"),
            (two, "exactly one"),
            (none, "exactly one"),
            (existing, "already carries"),
            (malformed, "invalid media payload"),
        ] {
            let error = media_reply_payload(payload, "ABC", "1@lid", conversation(""))
                .unwrap_err()
                .to_string();
            assert!(error.contains(reason), "{error}");
        }
    }

    /// An ordinary chat message carries none of the styling fields, and a plain
    /// `conversation` cannot carry them at all.
    #[test]
    fn unstyled_text_reports_no_style() {
        let plain = decode(b"\x0a\x05hello").expect("conversation decodes");
        assert_eq!(plain.text, "hello");
        assert!(!plain.style.is_styled());
        assert_eq!(plain.style, TextStyle::default());

        let reply = reply_payload("ok", "ABC", "1@s.whatsapp.net", conversation("hi"));
        let decoded = decode(&reply).expect("reply decodes");
        assert!(!decoded.style.is_styled());
    }

    /// The enum covers exactly 0..=8 in this build; anything else is dropped
    /// rather than mapped to a font we would be inventing.
    #[test]
    fn unknown_font_values_are_dropped() {
        for v in 0..=8 {
            assert!(StatusFont::from_wire(v).is_some(), "{v} should map");
        }
        for v in [-1, 9, 99] {
            assert!(StatusFont::from_wire(v).is_none(), "{v} must not map");
        }
    }

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
