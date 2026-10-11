//! Node encoder.

use super::CodecError;
use super::node::{AttributeValue, Node, NodeContent};
use super::tokens::{TokenDictionary, tags};
use crate::model::jid::{Jid, JidKind, Server};

/// Encodes nodes into the binary form. Output starts with a zero flag byte
/// (uncompressed) so it can be used directly as a frame payload.
#[derive(Debug)]
pub struct Encoder<'a> {
    dict: &'a TokenDictionary,
    out: Vec<u8>,
}

impl<'a> Encoder<'a> {
    /// Create an encoder.
    pub fn new(dict: &'a TokenDictionary) -> Self {
        Self {
            dict,
            out: vec![0u8],
        }
    }

    /// Encode a node and return the payload (flag byte included).
    pub fn encode(mut self, node: &Node) -> Result<Vec<u8>, CodecError> {
        self.write_node(node)?;
        Ok(self.out)
    }

    fn push(&mut self, b: u8) {
        self.out.push(b);
    }

    fn list_start(&mut self, size: usize) -> Result<(), CodecError> {
        if size == 0 {
            self.push(tags::LIST_EMPTY);
        } else if size < 256 {
            self.push(tags::LIST_8);
            self.push(size as u8);
        } else if size <= u16::MAX as usize {
            self.push(tags::LIST_16);
            self.out.extend_from_slice(&(size as u16).to_be_bytes());
        } else {
            return Err(CodecError::Unencodable("list too long"));
        }
        Ok(())
    }

    fn write_len_prefixed(&mut self, data: &[u8]) -> Result<(), CodecError> {
        let len = data.len();
        if len < 256 {
            self.push(tags::BINARY_8);
            self.push(len as u8);
        } else if len < (1 << 20) {
            self.push(tags::BINARY_20);
            self.out.extend_from_slice(&[
                ((len >> 16) & 0x0F) as u8,
                ((len >> 8) & 0xFF) as u8,
                (len & 0xFF) as u8,
            ]);
        } else if len <= u32::MAX as usize {
            self.push(tags::BINARY_32);
            self.out.extend_from_slice(&(len as u32).to_be_bytes());
        } else {
            return Err(CodecError::Unencodable("binary too long"));
        }
        self.out.extend_from_slice(data);
        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<(), CodecError> {
        if let Some(i) = self.dict.index_of_single(s) {
            self.push(i);
        } else if let Some((d, i)) = self.dict.index_of_double(s) {
            self.push(tags::DICTIONARY_0 + d);
            self.push(i);
        } else if is_nibble(s) {
            self.write_packed(s, tags::NIBBLE_8);
        } else if is_hex(s) {
            self.write_packed(s, tags::HEX_8);
        } else {
            self.write_len_prefixed(s.as_bytes())?;
        }
        Ok(())
    }

    fn write_packed(&mut self, s: &str, tag: u8) {
        let b = s.as_bytes();
        self.push(tag);
        let mut header = b.len().div_ceil(2) as u8;
        if b.len() % 2 == 1 {
            header |= 0x80;
        }
        self.push(header);
        let pack = |c: u8| -> u8 {
            match tag {
                tags::NIBBLE_8 => match c {
                    b'-' => 10,
                    b'.' => 11,
                    0 => 15,
                    d => d - b'0',
                },
                _ => match c {
                    b'0'..=b'9' => c - b'0',
                    // Odd-length padding (mirrors the reference encoders).
                    0 => 15,
                    _ => c - b'A' + 10,
                },
            }
        };
        for pair in b.chunks(2) {
            let hi = pack(pair[0]);
            let lo = pack(*pair.get(1).unwrap_or(&0));
            self.push((hi << 4) | lo);
        }
    }

    fn write_jid(&mut self, jid: &Jid) -> Result<(), CodecError> {
        match jid.kind {
            JidKind::Facebook => {
                self.push(tags::FB_JID);
                self.write_string(&jid.user)?;
                self.out.extend_from_slice(&jid.device.to_be_bytes());
                self.write_string(jid.server.as_str())
            }
            JidKind::Interop => {
                self.push(tags::INTEROP_JID);
                self.write_string(&jid.user)?;
                self.out.extend_from_slice(&jid.device.to_be_bytes());
                self.out.extend_from_slice(&0u16.to_be_bytes());
                self.write_string(jid.server.as_str())
            }
            _ if jid.is_ad()
                && matches!(
                    jid.server,
                    Server::User | Server::Lid | Server::Hosted | Server::HostedLid
                ) =>
            {
                if jid.agent != 0 {
                    return Err(CodecError::Unencodable("AD JID has no agent field"));
                }
                if jid.device > u8::MAX as u16 {
                    return Err(CodecError::Unencodable("AD JID device exceeds u8"));
                }
                let domain = match jid.server {
                    Server::User => 0,
                    Server::Lid => 1,
                    Server::Hosted => 128,
                    Server::HostedLid => 129,
                    _ => unreachable!(),
                };
                self.push(tags::AD_JID);
                self.push(domain);
                self.push(jid.device as u8);
                self.write_string(&jid.user)
            }
            _ => {
                self.push(tags::JID_PAIR);
                if jid.user.is_empty() {
                    self.push(tags::LIST_EMPTY);
                } else {
                    self.write_string(&jid.user)?;
                }
                self.write_string(jid.server.as_str())
            }
        }
    }

    fn write_attr_value(&mut self, v: &AttributeValue) -> Result<(), CodecError> {
        match v {
            AttributeValue::Text(s) => self.write_string(s),
            AttributeValue::Jid(j) => self.write_jid(j),
            AttributeValue::Bytes(b) => self.write_len_prefixed(b),
        }
    }

    fn write_node(&mut self, node: &Node) -> Result<(), CodecError> {
        if node.description.is_empty() {
            return Err(CodecError::Unencodable("empty description"));
        }
        // APK 37.71 0hs.A0Q / 1Qe.A01 retains empty attribute strings;
        // 1Qe.A05 -> A06 serializes a zero-length value instead of omitting it.
        // Dropping them changed Hgb.A00's initial AB-property hash request.
        let attrs = &node.attributes;
        let has_content = !matches!(node.content, NodeContent::Empty);
        self.list_start(1 + 2 * attrs.len() + usize::from(has_content))?;
        self.write_string(&node.description)?;
        for a in attrs {
            self.write_string(&a.name)?;
            self.write_attr_value(&a.value)?;
        }
        match &node.content {
            NodeContent::Empty => {}
            NodeContent::Bytes(b) => self.write_len_prefixed(b)?,
            NodeContent::Text(t) => self.write_string(t)?,
            NodeContent::Jid(j) => self.write_jid(j)?,
            NodeContent::Children(c) => {
                self.list_start(c.len())?;
                for child in c {
                    self.write_node(child)?;
                }
            }
        }
        Ok(())
    }
}

fn is_nibble(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= tags::PACKED_MAX
        && s.bytes()
            .all(|b| b.is_ascii_digit() || b == b'-' || b == b'.')
}

fn is_hex(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= tags::PACKED_MAX
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;
    use crate::binary::decoder::{Decoder, unpack_frame_payload};
    use crate::profile::Limits;

    fn dict() -> &'static TokenDictionary {
        TokenDictionary::web_md_v3()
    }

    fn roundtrip(node: &Node) -> Node {
        let bytes = Encoder::new(dict()).encode(node).unwrap();
        let limits = Limits::default();
        let payload = unpack_frame_payload(&bytes, &limits).unwrap();
        Decoder::new(&payload, dict(), &limits)
            .decode_node()
            .unwrap()
    }

    #[test]
    fn independent_fixture_iq_ping() {
        // Independently derived from the reference encoding rules:
        // <iq id="1" type="get" xmlns="w:p"><ping/></iq>
        // LIST_8 size 8: iq, id,"1", type,get, xmlns,"w:p", content
        let bytes = [
            0x00, // flag byte
            0xF8, 0x08, // LIST_8, 8 items
            25,   // "iq"
            8,    // "id"
            0xFF, 0x81, 0x10, // NIBBLE_8 "1" (odd, 1 pair): 0x1 | 0x0
            4,    // "type"
            41,   // "get"
            22,   // "xmlns"
            87,   // "w:p"
            0xF8, 0x01, // LIST_8, 1 child
            0xF8, 0x01, // LIST_8 size 1
            86,   // "ping"
        ];
        let limits = Limits::default();
        let payload = unpack_frame_payload(&bytes, &limits).unwrap();
        let node = Decoder::new(&payload, dict(), &limits)
            .decode_node()
            .unwrap();
        assert_eq!(node.description, "iq");
        assert_eq!(node.get_attr("id").unwrap().as_text(), Some("1"));
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("get"));
        assert_eq!(node.get_attr("xmlns").unwrap().as_text(), Some("w:p"));
        assert_eq!(node.child_nodes().len(), 1);
        assert_eq!(node.child_nodes()[0].description, "ping");
        // "1" has two valid encodings: the packed nibble above and the
        // single-byte token 85. The encoder prefers the token; both decode to
        // the same node, and byte equality is not required where the profile
        // does not canonicalize (docs/design/README.md section 6.3).
        let expected = Node::new("iq")
            .attr("id", "1")
            .attr("type", "get")
            .attr("xmlns", "w:p")
            .children(vec![Node::new("ping")]);
        let encoded = Encoder::new(dict()).encode(&expected).unwrap();
        assert_eq!(
            encoded,
            [
                0x00, 0xF8, 0x08, 25, 8, 85, 4, 41, 22, 87, 0xF8, 0x01, 0xF8, 0x01, 86
            ]
        );
        assert_eq!(node, expected);
        let payload2 = unpack_frame_payload(&encoded, &limits).unwrap();
        let node2 = Decoder::new(&payload2, dict(), &limits)
            .decode_node()
            .unwrap();
        assert_eq!(node2, node);
    }

    #[test]
    fn apk_ad_jid_flags_select_namespace_without_creating_an_agent() {
        for (flags, server) in [
            (2, Server::User),
            (3, Server::Lid),
            (130, Server::Hosted),
            (131, Server::HostedLid),
        ] {
            let mut wire = b"\xf8\x03\xfc\x01n\xfc\x01j\xf7".to_vec();
            wire.extend_from_slice(&[flags, 0, 0xfc, 3]);
            wire.extend_from_slice(b"123");
            let node = Decoder::new(&wire, dict(), &Limits::default())
                .decode_node()
                .unwrap();
            let jid = node.get_attr("j").unwrap().as_jid().unwrap();
            assert_eq!(jid.server, server);
            assert_eq!(
                (jid.agent, jid.device, jid.kind),
                (0, 0, JidKind::AgentDevice)
            );
        }
    }

    #[test]
    fn apk_ad_jid_writer_refuses_an_agent_in_the_flag_byte() {
        let node = Node::new("n").attr("j", Jid::new_ad("123", 3, 2, Server::User).unwrap());
        assert!(Encoder::new(dict()).encode(&node).is_err());
    }

    #[test]
    fn roundtrips_jids_bytes_and_packed_strings() {
        let node = Node::new("message")
            .attr("to", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            .attr(
                "participant",
                Jid::parse("15551234567:3@s.whatsapp.net").unwrap(),
            )
            .attr("lid", Jid::parse("123456789012345:1@lid").unwrap())
            .attr("server", Jid::parse("g.us").unwrap())
            .attr("hex", "DEADBEEF")
            .attr("raw", "hello world")
            .attr("odd", "12345")
            .children(vec![
                Node::new("enc")
                    .attr("type", "pkmsg")
                    .bytes(Bytes::from_static(&[0xFF, 0x00, 0x80, 0x7F])),
                Node::new("presence").text("unavailable"),
            ]);
        let back = roundtrip(&node);
        assert_eq!(back, node);
        // Raw (non-token) text content is indistinguishable from bytes on the
        // wire and decodes as bytes; nothing is lossily converted.
        let raw = Node::new("text").text("some text");
        let back = roundtrip(&raw);
        assert_eq!(
            back.content,
            NodeContent::Bytes(Bytes::from_static(b"some text"))
        );
    }

    #[test]
    fn odd_length_packed_strings_roundtrip() {
        for v in ["A", "ABC", "DEADBEE", "1", "123", "1.2-3"] {
            let node = Node::new("iq").attr("v", v);
            assert_eq!(roundtrip(&node), node, "{v}");
        }
    }

    #[test]
    fn independent_empty_attribute_raw_encoding_fixture() {
        // Literal LIST_8(3), raw names, BINARY_8(0). This is a supported raw
        // encoding, not a claim about APK packed-string marker selection.
        let expected = b"\x00\xf8\x03\xfc\x0cfixture-node\xfc\x0cfixture-attr\xfc\x00";
        let node = Node::new("fixture-node").attr("fixture-attr", "");
        assert_eq!(Encoder::new(dict()).encode(&node).unwrap(), expected);
        assert_eq!(roundtrip(&node), node);
        let mut packed = expected.to_vec();
        let end = packed.len();
        packed[end - 2] = tags::NIBBLE_8;
        let limits = Limits::default();
        let payload = unpack_frame_payload(&packed, &limits).unwrap();
        assert_eq!(
            Decoder::new(&payload, dict(), &limits)
                .decode_node()
                .unwrap(),
            node
        );
    }

    #[test]
    fn empty_attributes_and_non_utf8_are_preserved() {
        let node = Node::new("x").attr("a", "").attr("b", "v");
        let back = roundtrip(&node);
        assert_eq!(back, node);
        let raw = Node::new("x").attr(
            "bin",
            AttributeValue::Bytes(Bytes::from_static(&[0xFF, 0xFE])),
        );
        let back = roundtrip(&raw);
        assert_eq!(
            back.get_attr("bin"),
            Some(&AttributeValue::Bytes(Bytes::from_static(&[0xFF, 0xFE])))
        );
    }

    #[test]
    fn long_binary_uses_20_and_32_bit_lengths() {
        let mid = Node::new("iq").bytes(vec![7u8; 300]);
        let big = Node::new("iq").bytes(vec![7u8; 1 << 20]);
        let limits = Limits {
            max_string_len: 2 << 20,
            ..Limits::default()
        };
        for n in [&mid, &big] {
            let bytes = Encoder::new(dict()).encode(n).unwrap();
            let payload = unpack_frame_payload(&bytes, &limits).unwrap();
            let back = Decoder::new(&payload, dict(), &limits)
                .decode_node()
                .unwrap();
            assert_eq!(&back, n);
        }
        // [flag][LIST_8][2]["iq" token][content tag]
        let bytes = Encoder::new(dict()).encode(&big).unwrap();
        assert_eq!(bytes[4], tags::BINARY_32);
        let bytes = Encoder::new(dict()).encode(&mid).unwrap();
        assert_eq!(bytes[4], tags::BINARY_20);
    }

    #[test]
    fn rejects_duplicate_attributes_truncation_and_trailing() {
        let limits = Limits::default();
        // duplicate "id"
        let dup = [0xF8, 0x05, 25, 8, 41, 8, 41];
        assert!(matches!(
            Decoder::new(&dup, dict(), &limits).decode_node(),
            Err(CodecError::DuplicateAttribute { .. })
        ));
        let trunc = [0xF8, 0x03, 25, 8];
        assert!(matches!(
            Decoder::new(&trunc, dict(), &limits).decode_node(),
            Err(CodecError::UnexpectedEof { .. })
        ));
        let trailing = [0xF8, 0x01, 86, 0x00];
        assert!(matches!(
            Decoder::new(&trailing, dict(), &limits).decode_node(),
            Err(CodecError::TrailingBytes(1))
        ));
        let unknown_tag = [0xF8, 0x01, 0xF0];
        assert!(matches!(
            Decoder::new(&unknown_tag, dict(), &limits).decode_node(),
            Err(CodecError::UnknownTag { tag: 0xF0, .. })
        ));
        // A dictionary with a hole: index 2 is undefined.
        let small = TokenDictionary::from_fixture_json(
            r#"{"provenance":{"evidence":"Unknown","source":"","source_revision":"","ledger_ref":""},
                "dictionary_version":0,"single_byte":["","iq"],"double_byte":[["a"]]}"#,
        )
        .unwrap();
        let unknown_token = [0xF8, 0x01, 0x02];
        assert!(matches!(
            Decoder::new(&unknown_token, &small, &limits).decode_node(),
            Err(CodecError::UnknownToken {
                dictionary: 0,
                index: 2,
                ..
            })
        ));
        let unknown_double = [0xF8, 0x01, 0xEC, 0x05];
        assert!(matches!(
            Decoder::new(&unknown_double, &small, &limits).decode_node(),
            Err(CodecError::UnknownToken {
                dictionary: 1,
                index: 5,
                ..
            })
        ));
    }

    #[test]
    fn enforces_length_depth_and_children_bounds() {
        let limits = Limits {
            max_string_len: 4,
            ..Limits::default()
        };
        let long = [0xF8, 0x01, 0xFC, 0x05, b'a', b'b', b'c', b'd', b'e'];
        assert!(matches!(
            Decoder::new(&long, dict(), &limits).decode_node(),
            Err(CodecError::LengthLimit { what: "binary", .. })
        ));
        // Declared 32-bit length far beyond input must fail on the bound, not allocate.
        let huge = [0xF8, 0x01, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(matches!(
            Decoder::new(&huge, dict(), &Limits::default()).decode_node(),
            Err(CodecError::LengthLimit { .. })
        ));
        let limits = Limits {
            max_node_depth: 2,
            ..Limits::default()
        };
        let deep = {
            let mut n = Node::new("a");
            for _ in 0..5 {
                n = Node::new("a").children(vec![n]);
            }
            n
        };
        let bytes = Encoder::new(dict()).encode(&deep).unwrap();
        assert!(matches!(
            Decoder::new(&bytes[1..], dict(), &limits).decode_node(),
            Err(CodecError::DepthLimit { limit: 2 })
        ));
        let limits = Limits {
            max_children: 1,
            ..Limits::default()
        };
        let wide = Node::new("a").children(vec![Node::new("b"), Node::new("c")]);
        let bytes = Encoder::new(dict()).encode(&wide).unwrap();
        assert!(matches!(
            Decoder::new(&bytes[1..], dict(), &limits).decode_node(),
            Err(CodecError::LengthLimit {
                what: "children",
                ..
            })
        ));
    }

    #[test]
    fn decompression_is_bounded() {
        let limits = Limits {
            max_decompressed_len: 1024,
            ..Limits::default()
        };
        let bomb_plain = vec![0u8; 1 << 16];
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&bomb_plain, 6);
        let mut payload = vec![0x02];
        payload.extend_from_slice(&compressed);
        assert!(matches!(
            unpack_frame_payload(&payload, &limits),
            Err(CodecError::Compression(_))
        ));
        // A small compressed node inflates and decodes.
        let node = Node::new("ping");
        let encoded = Encoder::new(dict()).encode(&node).unwrap();
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&encoded[1..], 6);
        let mut payload = vec![0x02];
        payload.extend_from_slice(&compressed);
        let inflated = unpack_frame_payload(&payload, &limits).unwrap();
        assert_eq!(
            Decoder::new(&inflated, dict(), &limits)
                .decode_node()
                .unwrap(),
            node
        );
        assert!(matches!(
            unpack_frame_payload(&[0x02, 0xFF, 0xFF], &limits),
            Err(CodecError::Compression(_))
        ));
        assert!(matches!(
            unpack_frame_payload(&[], &limits),
            Err(CodecError::UnexpectedEof { .. })
        ));
    }
}
