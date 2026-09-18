//! Property tests (docs/design/README.md 19.1): encode/decode round trips for arbitrary
//! nodes, framing under arbitrary fragmentation, and decoder robustness on
//! arbitrary bytes (never panics, never exceeds bounds).

use bytes::Bytes;
use proptest::prelude::*;
use whatsapp_api::binary::{
    AttributeValue, Decoder, Encoder, Node, NodeContent, TokenDictionary, unpack_frame_payload,
};
use whatsapp_api::model::jid::{Jid, Server};
use whatsapp_api::profile::Limits;
use whatsapp_api::transport::{FrameCodec, FrameReader, TransportError};

fn dict() -> &'static TokenDictionary {
    TokenDictionary::web_md_v3()
}

/// Strings the encoder may emit as tokens, packed digits/hex or raw bytes.
fn any_text() -> impl Strategy<Value = String> {
    prop_oneof![
        // known tokens
        (1u8..235).prop_map(|i| dict().single(i).unwrap_or("iq").to_owned()),
        (0u8..4, any::<u8>()).prop_map(|(p, i)| dict().double(p, i).unwrap_or("iq").to_owned()),
        // nibble-packable
        "[0-9.-]{1,40}",
        // hex-packable
        "[0-9A-F]{1,40}",
        // arbitrary printable
        "[a-zA-Z0-9 _:/@.-]{1,64}",
    ]
}

fn any_jid() -> impl Strategy<Value = Jid> {
    (
        "[0-9]{6,15}",
        prop_oneof![Just(Server::User), Just(Server::Lid), Just(Server::Group)],
        0u16..=255,
    )
        .prop_map(|(user, server, device)| match server {
            Server::Group => Jid::new(user, server).unwrap(),
            _ if device > 0 => Jid::new_ad(user, 0, device, server).unwrap(),
            _ => Jid::new(user, server).unwrap(),
        })
}

fn any_attr_value() -> impl Strategy<Value = AttributeValue> {
    prop_oneof![
        any_text().prop_map(AttributeValue::Text),
        any_jid().prop_map(AttributeValue::Jid),
    ]
}

fn leaf() -> impl Strategy<Value = Node> {
    (
        any_text().prop_filter("non-empty description", |d| !d.is_empty()),
        prop::collection::vec((any_text(), any_attr_value()), 0..6),
        prop_oneof![
            Just(NodeContent::Empty),
            prop::collection::vec(any::<u8>(), 0..300)
                .prop_map(|b| NodeContent::Bytes(Bytes::from(b))),
            any_jid().prop_map(NodeContent::Jid),
        ],
    )
        .prop_map(|(desc, attrs, content)| {
            let mut n = Node::new(desc);
            for (k, v) in attrs {
                if !k.is_empty() {
                    n = n.attr(k, v);
                }
            }
            n.content = content;
            n
        })
}

fn any_node() -> impl Strategy<Value = Node> {
    leaf().prop_recursive(3, 24, 6, |inner| {
        (
            any_text().prop_filter("non-empty description", |d| !d.is_empty()),
            prop::collection::vec((any_text(), any_attr_value()), 0..4),
            prop::collection::vec(inner, 0..6),
        )
            .prop_map(|(desc, attrs, children)| {
                let mut n = Node::new(desc);
                for (k, v) in attrs {
                    if !k.is_empty() {
                        n = n.attr(k, v);
                    }
                }
                n.children(children)
            })
    })
}

/// Attribute values that are empty text are dropped by the encoder by design;
/// normalise both sides before comparing.
fn normalise(n: &Node) -> Node {
    let mut out = Node::new(n.description.clone());
    for a in &n.attributes {
        if !matches!(&a.value, AttributeValue::Text(t) if t.is_empty()) {
            out = out.attr(a.name.clone(), a.value.clone());
        }
    }
    out.content = match &n.content {
        // An empty child list and no content share one wire form (LIST_EMPTY).
        NodeContent::Children(c) if c.is_empty() => NodeContent::Empty,
        NodeContent::Children(c) => NodeContent::Children(c.iter().map(normalise).collect()),
        NodeContent::Text(t) => NodeContent::Bytes(Bytes::from(t.clone().into_bytes())),
        other => other.clone(),
    };
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn node_roundtrip(node in any_node()) {
        let limits = Limits::default();
        let encoded = Encoder::new(dict()).encode(&node).unwrap();
        let payload = unpack_frame_payload(&encoded, &limits).unwrap();
        let decoded = Decoder::new(&payload, dict(), &limits).decode_node().unwrap();
        // Token/packed text content decodes as Text; raw text as Bytes. Both
        // are equivalent on the wire, so compare after normalising to bytes.
        let lhs = normalise(&node);
        let rhs = normalise(&decoded);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn decoder_never_panics_and_respects_bounds(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        let limits = Limits { max_string_len: 64, max_children: 8, max_node_depth: 4, max_attributes: 4, ..Limits::default() };
        let _ = Decoder::new(&bytes, dict(), &limits).decode_node();
        let _ = unpack_frame_payload(&bytes, &limits);
    }

    #[test]
    fn framing_survives_arbitrary_fragmentation(
        payloads in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..2000), 1..6),
        cuts in prop::collection::vec(1usize..64, 0..64),
    ) {
        let codec = FrameCodec::new(3, 1 << 16).unwrap();
        let mut stream = Vec::new();
        for p in &payloads {
            stream.extend_from_slice(&codec.encode(p, None).unwrap());
        }
        let mut reader = FrameReader::new(codec);
        let mut got = Vec::new();
        let mut pos = 0;
        let mut ci = 0;
        while pos < stream.len() {
            let step = cuts.get(ci).copied().unwrap_or(7).min(stream.len() - pos);
            ci += 1;
            reader.feed(&stream[pos..pos + step]).unwrap();
            pos += step;
            while let Some(f) = reader.next_frame().unwrap() {
                got.push(f.to_vec());
            }
        }
        prop_assert_eq!(got, payloads);
        prop_assert_eq!(reader.buffered(), 0);
    }

    #[test]
    fn framing_rejects_oversized_declared_lengths(len in 1025usize..(1 << 24)) {
        let codec = FrameCodec::new(3, 1024).unwrap();
        let mut reader = FrameReader::new(codec);
        let prefix = [(len >> 16) as u8, (len >> 8) as u8, len as u8];
        let r = reader.feed(&prefix);
        let too_large = matches!(r, Err(TransportError::FrameTooLarge { .. }));
        prop_assert!(too_large);
    }
}
