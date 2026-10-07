//! Group-history notice metadata, APK 2.26.37.71 BZc/BV6/BX5 descriptors.
//! DA8.CF4 requires receivers, a positive window timestamp and message count.
//! This decoder neither authorizes a sender nor fetches/applies history.
use crate::{
    Error, Result,
    model::jid::{Jid, Server},
};
use prost::{
    Message,
    encoding::{DecodeContext, WireType, decode_key, skip_field},
};

// Local diagnostic limits, not APK/server requirements.
const MAX_BYTES: usize = 65_536;
const MAX_RECEIVERS: usize = 1024;

/// Validated supported metadata without receiver addresses or context contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Positive history message count (BX5 field 3, int64).
    pub message_count: i64,
    /// Positive raw window timestamp (field 2); units remain untraced.
    pub oldest_window_timestamp: i64,
    /// Optional raw bundle timestamp (field 5); no unit conversion is inferred.
    pub oldest_bundle_timestamp: Option<i64>,
    /// Number of supported primary PN/LID history receivers (field 1).
    pub receiver_count: usize,
    /// Number of supported primary PN/LID non-history receivers (field 4).
    pub non_receiver_count: usize,
    /// Whether opaque context information is present (BV6 field 1).
    pub context_present: bool,
    /// Whether opaque bot-share metadata is present (BV6 field 3).
    pub bot_metadata_present: bool,
}

fn invalid(reason: &'static str) -> Error {
    Error::invalid("history_notice", reason)
}

#[derive(prost::Message)]
struct Envelope {
    // BZc.dynamicMethod: messageHistoryNotice=102.
    #[prost(bytes = "vec", optional, tag = "102")]
    notice: Option<Vec<u8>>,
}
#[derive(prost::Message)]
struct WireMetadata {
    // BX5.dynamicMethod at 0066: repeated strings 1/4, int64 2/3/5.
    #[prost(string, repeated, tag = "1")]
    receivers: Vec<String>,
    #[prost(int64, optional, tag = "2")]
    oldest_window: Option<i64>,
    #[prost(int64, optional, tag = "3")]
    count: Option<i64>,
    #[prost(string, repeated, tag = "4")]
    non_receivers: Vec<String>,
    #[prost(int64, optional, tag = "5")]
    oldest_bundle: Option<i64>,
}

fn fields(mut bytes: &[u8]) -> Result<Vec<(u32, WireType)>> {
    if bytes.len() > MAX_BYTES {
        return Err(invalid("payload exceeds local inspection limit"));
    }
    let mut fields = Vec::new();
    while !bytes.is_empty() {
        let (tag, wire) = decode_key(&mut bytes).map_err(|_| invalid("invalid protobuf"))?;
        skip_field(wire, tag, &mut bytes, DecodeContext::default())
            .map_err(|_| invalid("invalid protobuf"))?;
        fields.push((tag, wire));
    }
    Ok(fields)
}

fn singleton_fields(fields: &[(u32, WireType)], allowed: &[u32]) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for &(tag, wire) in fields {
        if !allowed.contains(&tag) || wire != WireType::LengthDelimited {
            return Err(invalid("unsupported field or wire type"));
        }
        if !seen.insert(tag) {
            return Err(invalid("duplicate singleton field"));
        }
    }
    Ok(())
}

/// Inspect one Message envelope. Other message types return `None`; a present
/// unsupported/malformed notice returns a redacted error. Values are not
/// authenticated here. Group/app-data checks belong to the receive dispatcher.
pub fn decode(payload: &[u8]) -> Result<Option<Metadata>> {
    let tags = fields(payload)?;
    if !tags.iter().any(|(tag, _)| *tag == 102) {
        return Ok(None);
    }
    singleton_fields(&tags, &[35, 102])?;
    let envelope = Envelope::decode(payload).map_err(|_| invalid("invalid message envelope"))?;
    let notice = zeroize::Zeroizing::new(envelope.notice.ok_or_else(|| invalid("notice missing"))?);
    decode_body(&notice).map(Some)
}

fn decode_body(body: &[u8]) -> Result<Metadata> {
    let tags = fields(body)?;
    singleton_fields(&tags, &[1, 2, 3])?;
    // Check metadata's wire shape before prost can merge duplicate scalar or
    // message fields. Unknown data remains unsupported instead of being lost.
    #[derive(prost::Message)]
    struct RawNotice {
        // BV6.dynamicMethod at 0052: messageHistoryMetadata=2. Context/bot
        // bodies are deliberately skipped; only their presence is inspected.
        #[prost(bytes = "vec", optional, tag = "2")]
        metadata: Option<Vec<u8>>,
    }
    let raw = RawNotice::decode(body).map_err(|_| invalid("invalid notice"))?;
    let metadata_bytes =
        zeroize::Zeroizing::new(raw.metadata.ok_or_else(|| invalid("metadata missing"))?);
    let mut seen = std::collections::BTreeSet::new();
    let mut counts = [0usize; 2];
    for (tag, wire) in fields(&metadata_bytes)? {
        let expected = match tag {
            1 | 4 => WireType::LengthDelimited,
            2 | 3 | 5 => WireType::Varint,
            _ => return Err(invalid("unsupported metadata field")),
        };
        if wire != expected {
            return Err(invalid("invalid metadata wire type"));
        }
        if matches!(tag, 1 | 4) {
            let index = usize::from(tag == 4);
            counts[index] += 1;
            if counts[index] > MAX_RECEIVERS {
                return Err(invalid("receiver list exceeds local inspection limit"));
            }
        } else if !seen.insert(tag) {
            return Err(invalid("duplicate metadata scalar"));
        }
    }
    let metadata =
        WireMetadata::decode(metadata_bytes.as_slice()).map_err(|_| invalid("invalid metadata"))?;
    // DA8.CF4 00ac..00ec: these are required, not optional protobuf defaults.
    if metadata.receivers.is_empty() {
        return Err(invalid("history receivers missing"));
    }
    let oldest_window = metadata
        .oldest_window
        .filter(|v| *v > 0)
        .ok_or_else(|| invalid("positive oldest-window timestamp missing"))?;
    let count = metadata
        .count
        .filter(|v| *v > 0)
        .ok_or_else(|| invalid("positive message count missing"))?;
    // APK 8kZ.A1L -> 0D8.A01 parses UserJid or throws. Our supported
    // subset is primary numeric PN/LID; other kinds remain unsupported locally.
    for address in metadata.receivers.iter().chain(&metadata.non_receivers) {
        let jid = Jid::parse(address).map_err(|_| invalid("invalid receiver identity"))?;
        if !matches!(jid.server, Server::User | Server::Lid)
            || jid.device != 0
            || jid.agent != 0
            || jid.user.is_empty()
            || !jid.user.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(invalid("unsupported receiver identity"));
        }
    }
    Ok(Metadata {
        message_count: count,
        oldest_window_timestamp: oldest_window,
        oldest_bundle_timestamp: metadata.oldest_bundle,
        receiver_count: metadata.receivers.len(),
        non_receiver_count: metadata.non_receivers.len(),
        context_present: tags.iter().any(|(tag, _)| *tag == 1),
        bot_metadata_present: tags.iter().any(|(tag, _)| *tag == 3),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Literal protobuf, independently written from BZc/BV6/BX5 descriptors:
    // receiver 1@lid, positive window timestamp 10, positive count 2.
    const META: &[u8] = b"\x0a\x051@lid\x10\x0a\x18\x02";
    const PAYLOAD: &[u8] = b"\xb2\x06\x0d\x12\x0b\x0a\x051@lid\x10\x0a\x18\x02";

    fn body(meta: &[u8]) -> Vec<u8> {
        let mut out = vec![0x12];
        prost::encoding::encode_varint(meta.len() as u64, &mut out);
        out.extend_from_slice(meta);
        out
    }

    #[test]
    fn literal_metadata_decodes_without_receiver_or_context_values() {
        let metadata = decode(PAYLOAD).unwrap().unwrap();
        assert_eq!(metadata.message_count, 2);
        assert_eq!(metadata.oldest_window_timestamp, 10);
        assert_eq!(metadata.receiver_count, 1);
        assert_eq!(metadata.non_receiver_count, 0);
        assert_eq!(metadata.oldest_bundle_timestamp, None);
        assert!(!metadata.context_present);
        assert!(!format!("{metadata:?}").contains("1@lid"));
        assert_eq!(decode(b"\x0a\x02hi").unwrap(), None);
    }

    #[test]
    fn optional_metadata_retains_raw_timestamp_and_opaque_presence() {
        let mut meta = META.to_vec();
        meta.extend_from_slice(
            b"\x22\x102@s.whatsapp.net\x28\xff\xff\xff\xff\xff\xff\xff\xff\xff\x01",
        );
        let mut notice = body(&meta);
        notice.extend_from_slice(b"\x0a\x06secret\x1a\x03key");
        let metadata = decode_body(&notice).unwrap();
        assert_eq!(metadata.non_receiver_count, 1);
        assert_eq!(metadata.oldest_bundle_timestamp, Some(-1));
        assert!(metadata.context_present && metadata.bot_metadata_present);
        for value in ["secret", "key", "@", "2@s.whatsapp.net"] {
            assert!(!format!("{metadata:?}").contains(value));
        }
    }

    #[test]
    fn required_metadata_follows_apk_presence_and_positive_checks() {
        for meta in [
            &b"\x10\x0a\x18\x02"[..],            // no receivers
            &b"\x0a\x051@lid\x18\x02"[..],       // no window timestamp
            &b"\x0a\x051@lid\x10\0\x18\x02"[..], // zero timestamp
            &b"\x0a\x051@lid\x10\x0a"[..],       // no count
            &b"\x0a\x051@lid\x10\x0a\x18\0"[..], // zero count
            &b"\x0a\x051@lid\x10\x0a\x18\xff\xff\xff\xff\xff\xff\xff\xff\xff\x01"[..], // negative count
        ] {
            assert!(decode_body(&body(meta)).is_err());
        }
        assert!(decode_body(b"").is_err());
    }

    #[test]
    fn ambiguous_unknown_and_truncated_metadata_stays_unsupported() {
        for suffix in [&b"\x18\x03"[..], &b"\x30\x01"[..], &b"\x12\0"[..]] {
            let mut meta = META.to_vec();
            meta.extend_from_slice(suffix);
            assert!(decode_body(&body(&meta)).is_err());
        }
        let notice = body(META);
        let mut duplicate = notice.clone();
        duplicate.extend_from_slice(&notice);
        assert!(decode_body(&duplicate).is_err());
        let mut unknown = notice;
        unknown.extend_from_slice(b"\xfa\x07\0");
        assert!(decode_body(&unknown).is_err());
        for end in 1..PAYLOAD.len() {
            assert!(decode(&PAYLOAD[..end]).is_err());
        }
        let mut mixed = PAYLOAD.to_vec();
        mixed.extend_from_slice(b"\x0a\x02hi");
        assert!(decode(&mixed).is_err());
    }

    #[test]
    fn receiver_errors_and_local_bounds_are_redacted() {
        for receiver in ["private value", "1@g.us", "1:2@lid", "abc@lid"] {
            let mut meta = vec![0x0a, receiver.len() as u8];
            meta.extend_from_slice(receiver.as_bytes());
            meta.extend_from_slice(b"\x10\x0a\x18\x02");
            let error = decode_body(&body(&meta)).unwrap_err();
            assert!(!error.to_string().contains(receiver));
        }
        assert!(decode(&vec![0; MAX_BYTES + 1]).is_err());
        let mut too_many = b"\x10\x0a\x18\x02".to_vec();
        for _ in 0..=MAX_RECEIVERS {
            too_many.extend_from_slice(b"\x0a\x051@lid");
        }
        assert!(
            decode_body(&body(&too_many))
                .unwrap_err()
                .to_string()
                .contains("receiver list exceeds local inspection limit")
        );
    }
}
