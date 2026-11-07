//! Bounded node decoder.

use bytes::Bytes;

use super::CodecError;
use super::node::{Attribute, AttributeValue, Node, NodeContent};
use super::tokens::{TokenDictionary, tags};
use crate::model::jid::{Jid, JidKind, Server};
use crate::profile::Limits;

/// Frame-payload flag bit indicating zlib compression of the remainder
/// APK 2.26.43.72 `123.A08` masks `0x02` before selecting inflation.
pub const FLAG_COMPRESSED: u8 = 0x02;

/// Strip the leading flag byte and inflate if the compression flag is set.
/// Inflation output is bounded by `limits.max_decompressed_len`.
pub fn unpack_frame_payload(payload: &[u8], limits: &Limits) -> Result<Bytes, CodecError> {
    let Some((&flags, rest)) = payload.split_first() else {
        return Err(CodecError::UnexpectedEof {
            offset: 0,
            needed: 1,
        });
    };
    // APK 43.72 123.A08 rejects header-only records and mask0x01 fragmentation;
    // a fragment must never reach ordinary node/ACK/notification processing.
    // Evidence: docs/protocol/apk/2.26.43.72-stream-end/frame-reader.txt.
    if rest.is_empty() {
        return Err(CodecError::HeaderOnlyFrame);
    }
    if flags & 0x01 != 0 {
        return Err(CodecError::UnsupportedFragment);
    }
    if flags & FLAG_COMPRESSED == 0 {
        return Ok(Bytes::copy_from_slice(rest));
    }
    let max = limits.max_decompressed_len as usize;
    let out =
        miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(rest, max).map_err(|e| match e
            .status
        {
            miniz_oxide::inflate::TINFLStatus::HasMoreOutput => {
                CodecError::Compression("decompressed output exceeds limit")
            }
            _ => CodecError::Compression("invalid zlib stream"),
        })?;
    Ok(Bytes::from(out))
}

/// A bounded decoder over one unpacked payload.
#[derive(Debug)]
pub struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
    dict: &'a TokenDictionary,
    limits: &'a Limits,
}

enum Value {
    Absent,
    Text(String),
    Bytes(Vec<u8>),
    Jid(Jid),
}

impl<'a> Decoder<'a> {
    /// Create a decoder.
    pub fn new(data: &'a [u8], dict: &'a TokenDictionary, limits: &'a Limits) -> Self {
        Self {
            data,
            pos: 0,
            dict,
            limits,
        }
    }

    /// Decode exactly one node, rejecting trailing bytes.
    pub fn decode_node(mut self) -> Result<Node, CodecError> {
        // APK 43.72 123.A01 0018..0022 returns null for root token2,
        // before ordinary count validation. 1s2.run then exits the reader.
        // Evidence: docs/protocol/apk/2.26.43.72-stream-end/README.md.
        if matches!(self.data, [0, 2, ..] | [248, _, 2, ..] | [249, _, _, 2, ..]) {
            return Err(CodecError::StreamEnd);
        }
        let node = self.read_node(0)?;
        let remaining = self.data.len() - self.pos;
        if remaining != 0 {
            return Err(CodecError::TrailingBytes(remaining));
        }
        Ok(node)
    }

    fn need(&self, n: usize) -> Result<(), CodecError> {
        if self.pos + n > self.data.len() {
            return Err(CodecError::UnexpectedEof {
                offset: self.pos,
                needed: n - (self.data.len() - self.pos),
            });
        }
        Ok(())
    }

    fn byte(&mut self) -> Result<u8, CodecError> {
        self.need(1)?;
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn u16be(&mut self) -> Result<u16, CodecError> {
        self.need(2)?;
        let v = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    fn u20be(&mut self) -> Result<usize, CodecError> {
        self.need(3)?;
        let d = &self.data[self.pos..self.pos + 3];
        self.pos += 3;
        Ok(((d[0] as usize & 0x0F) << 16) | ((d[1] as usize) << 8) | d[2] as usize)
    }

    fn u32be(&mut self) -> Result<usize, CodecError> {
        self.need(4)?;
        let d = &self.data[self.pos..self.pos + 4];
        self.pos += 4;
        Ok(u32::from_be_bytes([d[0], d[1], d[2], d[3]]) as usize)
    }

    fn raw(&mut self, len: usize, what: &'static str) -> Result<&'a [u8], CodecError> {
        let limit = self.limits.max_string_len as usize;
        if len > limit {
            return Err(CodecError::LengthLimit { what, len, limit });
        }
        self.need(len)?;
        let s = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(s)
    }

    fn list_size(&mut self, tag: u8) -> Result<usize, CodecError> {
        match tag {
            tags::LIST_EMPTY => Ok(0),
            tags::LIST_8 => Ok(self.byte()? as usize),
            tags::LIST_16 => Ok(self.u16be()? as usize),
            // APK 43.72 123.A00 throws LIL for a non-list marker.
            // Keep unknown value tags separate: native formats may be unported.
            _ => Err(CodecError::InvalidWireStructure {
                offset: self.pos - 1,
                reason: "invalid list marker",
            }),
        }
    }

    fn packed(&mut self, tag: u8) -> Result<String, CodecError> {
        let start = self.byte()?;
        let pairs = (start & 0x7F) as usize;
        let odd = start & 0x80 != 0;
        // APK 43.72 123.A07 counts 2*pairs-odd actual digits. The
        // final low nibble is padding, not a digit to validate then discard.
        // Evidence: docs/protocol/apk/2.26.43.72-packed-nibble/README.md.
        if odd && pairs == 0 {
            return Err(CodecError::InvalidPackedLength);
        }
        // APK 43.72 123.A07 calls A06 to read the entire declared packed
        // buffer before interpreting digits. A bad prefix must not mask a
        // missing suffix as a recoverable digit error.
        // Evidence: docs/protocol/apk/2.26.43.72-truncated-node/README.md.
        self.need(pairs)?;
        let mut out = String::with_capacity(pairs * 2);
        for index in 0..pairs {
            let offset = self.pos;
            let b = self.byte()?;
            out.push(unpack_digit(tag, b >> 4, offset)?);
            if !odd || index + 1 < pairs {
                out.push(unpack_digit(tag, b & 0x0F, offset)?);
            }
        }
        Ok(out)
    }

    /// The leading node's description, without validating the rest. Used to
    /// recognise a terminal stanza (`stream:error`, `failure`) whose body does
    /// not decode. `None` when even the description cannot be read.
    pub fn peek_description(mut self) -> Option<String> {
        let tag = self.byte().ok()?;
        if self.list_size(tag).ok()? == 0 {
            return None;
        }
        self.read_string_value().ok().flatten()
    }

    fn read_string_value(&mut self) -> Result<Option<String>, CodecError> {
        let offset = self.pos;
        // A string position never holds a JID; a JID found here was always
        // refused below. Refuse its tag before reading it: JID parts are read
        // through this function, so nested JID tags otherwise recursed without
        // bound and a small compressed frame overflowed the stack (an abort,
        // not an error). Found by the 2026-10-13 network-input panic audit.
        if matches!(
            self.data.get(self.pos),
            Some(&(tags::JID_PAIR | tags::AD_JID | tags::FB_JID | tags::INTEROP_JID))
        ) {
            return Err(CodecError::InvalidNode {
                offset,
                reason: "expected string",
            });
        }
        match self.read_value(true)? {
            Value::Absent => Ok(None),
            Value::Text(s) => Ok(Some(s)),
            Value::Bytes(_) | Value::Jid(_) => Err(CodecError::InvalidNode {
                offset,
                reason: "expected string",
            }),
        }
    }

    fn read_server(&mut self) -> Result<Server, CodecError> {
        let offset = self.pos;
        let s = self
            .read_string_value()?
            .ok_or_else(|| CodecError::InvalidJid {
                offset,
                reason: "missing server".into(),
            })?;
        Server::parse(&s).map_err(|e| CodecError::InvalidJid {
            offset,
            reason: e.to_string(),
        })
    }

    fn read_jid_pair(&mut self) -> Result<Jid, CodecError> {
        let offset = self.pos;
        let user = self.read_string_value()?.unwrap_or_default();
        let server_offset = self.pos;
        let raw_server = self
            .read_string_value()?
            .ok_or_else(|| CodecError::InvalidJid {
                offset: server_offset,
                reason: "missing server".into(),
            })?;
        // APK43.72 123.A04 joins an empty user to the bare name;
        // 0Dd.A00 returns0EG/2LK for these exact symbolic UserJids.
        // Keep qualified/cache-dependent aliases and general server grammar separate.
        let server = if user.is_empty() && matches!(raw_server.as_str(), "status_me" | "lid_me") {
            Server::Other(raw_server)
        } else {
            Server::parse(&raw_server).map_err(|e| CodecError::InvalidJid {
                offset: server_offset,
                reason: e.to_string(),
            })?
        };
        Jid::new(user, server).map_err(|e| CodecError::InvalidJid {
            offset,
            reason: e.to_string(),
        })
    }

    fn read_ad_jid(&mut self) -> Result<Jid, CodecError> {
        let offset = self.pos;
        let flags = self.byte()?;
        let device = self.byte()? as u16;
        let user = self
            .read_string_value()?
            .ok_or_else(|| CodecError::InvalidJid {
                offset,
                reason: "missing user".into(),
            })?;
        // APK 37.71 124.A02 uses only the LID and hosted flag bits.
        let server = match flags & 0x81 {
            0 => Server::User,
            1 => Server::Lid,
            128 => Server::Hosted,
            129 => Server::HostedLid,
            _ => unreachable!(),
        };
        Jid::new_ad(user, 0, device, server).map_err(|e| CodecError::InvalidJid {
            offset,
            reason: e.to_string(),
        })
    }

    fn read_fb_jid(&mut self) -> Result<Jid, CodecError> {
        let offset = self.pos;
        let user = self.read_string_value()?.unwrap_or_default();
        let device = self.u16be()?;
        let server = self.read_server()?;
        if server != Server::Messenger {
            return Err(CodecError::InvalidJid {
                offset,
                reason: "FB JID with non-msgr server".into(),
            });
        }
        let mut j = Jid::new(user, server).map_err(|e| CodecError::InvalidJid {
            offset,
            reason: e.to_string(),
        })?;
        j.device = device;
        j.kind = JidKind::Facebook;
        Ok(j)
    }

    fn read_interop_jid(&mut self) -> Result<Jid, CodecError> {
        let offset = self.pos;
        let user = self.read_string_value()?.unwrap_or_default();
        let device = self.u16be()?;
        let integrator = self.u16be()?;
        // APK 43.72 123.A03 preserves the integrator in the address, and
        // 2p3 stores it separately. This legacy tag245 model cannot retain it;
        // accepting it would silently re-encode a different identity (zero).
        // Evidence: docs/protocol/apk/2.26.43.72-interop-identity/README.md.
        if integrator != 0 {
            return Err(CodecError::InvalidJid {
                offset,
                reason: "nonzero interop integrator is not representable".into(),
            });
        }
        let server = self.read_server()?;
        if server != Server::Interop {
            return Err(CodecError::InvalidJid {
                offset,
                reason: "interop JID with non-interop server".into(),
            });
        }
        let mut j = Jid::new(user, server).map_err(|e| CodecError::InvalidJid {
            offset,
            reason: e.to_string(),
        })?;
        j.device = device;
        j.kind = JidKind::Interop;
        Ok(j)
    }

    /// Read one value. `as_string` controls whether raw bytes are returned as
    /// text (attributes/description) or bytes (content). Text is only produced
    /// for valid UTF-8; otherwise raw bytes are returned unchanged.
    fn read_value(&mut self, as_string: bool) -> Result<Value, CodecError> {
        let offset = self.pos;
        let tag = self.byte()?;
        match tag {
            tags::LIST_EMPTY => Ok(Value::Absent),
            tags::LIST_8 | tags::LIST_16 => Err(CodecError::InvalidNode {
                offset,
                reason: "list in value position",
            }),
            tags::BINARY_8 => {
                let len = self.byte()? as usize;
                let raw = self.raw(len, "binary")?;
                Ok(bytes_or_text(raw, as_string))
            }
            tags::BINARY_20 => {
                let len = self.u20be()?;
                let raw = self.raw(len, "binary")?;
                Ok(bytes_or_text(raw, as_string))
            }
            tags::BINARY_32 => {
                let len = self.u32be()?;
                let raw = self.raw(len, "binary")?;
                Ok(bytes_or_text(raw, as_string))
            }
            tags::DICTIONARY_0..=tags::DICTIONARY_3 => {
                let page = tag - tags::DICTIONARY_0;
                let index = self.byte()?;
                self.dict
                    .double(page, index)
                    .map(|s| Value::Text(s.to_owned()))
                    .ok_or(CodecError::UnknownToken {
                        dictionary: page + 1,
                        index,
                        offset,
                    })
            }
            tags::JID_PAIR => Ok(Value::Jid(self.read_jid_pair()?)),
            tags::AD_JID => Ok(Value::Jid(self.read_ad_jid()?)),
            tags::FB_JID => Ok(Value::Jid(self.read_fb_jid()?)),
            tags::INTEROP_JID => Ok(Value::Jid(self.read_interop_jid()?)),
            tags::NIBBLE_8 | tags::HEX_8 => Ok(Value::Text(self.packed(tag)?)),
            t if (1..tags::DICTIONARY_0).contains(&t) => self
                .dict
                .single(t)
                .map(|s| Value::Text(s.to_owned()))
                .ok_or(CodecError::UnknownToken {
                    dictionary: 0,
                    index: t,
                    offset,
                }),
            other => Err(CodecError::UnknownTag { tag: other, offset }),
        }
    }

    fn read_content(&mut self, depth: usize) -> Result<NodeContent, CodecError> {
        let offset = self.pos;
        let tag = self.byte()?;
        match tag {
            tags::LIST_EMPTY => Ok(NodeContent::Empty),
            tags::LIST_8 | tags::LIST_16 => {
                let size = self.list_size(tag)?;
                let limit = self.limits.max_children as usize;
                if size > limit {
                    return Err(CodecError::LengthLimit {
                        what: "children",
                        len: size,
                        limit,
                    });
                }
                let mut children = Vec::with_capacity(size.min(64));
                for _ in 0..size {
                    children.push(self.read_node(depth + 1)?);
                }
                Ok(NodeContent::Children(children))
            }
            _ => {
                self.pos = offset;
                match self.read_value(false)? {
                    Value::Absent => Ok(NodeContent::Empty),
                    Value::Text(s) => Ok(NodeContent::Text(s)),
                    Value::Bytes(b) => Ok(NodeContent::Bytes(Bytes::from(b))),
                    Value::Jid(j) => Ok(NodeContent::Jid(j)),
                }
            }
        }
    }

    fn read_node(&mut self, depth: usize) -> Result<Node, CodecError> {
        let limit = self.limits.max_node_depth as usize;
        if depth > limit {
            return Err(CodecError::DepthLimit { limit });
        }
        let offset = self.pos;
        let size_tag = self.byte()?;
        let list_size = self.list_size(size_tag)?;
        if list_size == 0 {
            // Native A01 returns null for token2 before checking list size.
            // Root control is handled above decode_node; nested null-child
            // semantics remain unported, so retain their local refusal class.
            if depth > 0 && self.data.get(self.pos) == Some(&2) {
                return Err(CodecError::InvalidNode {
                    offset,
                    reason: "empty node list",
                });
            }
            return Err(CodecError::InvalidWireStructure {
                offset,
                reason: "empty node list",
            });
        }
        // APK 43.72 123.A01 rejects a null tag; its check does not equate
        // nonnull empty text with null. Preserve the separate local refusal.
        let description = self
            .read_string_value()?
            .ok_or(CodecError::InvalidWireStructure {
                offset,
                reason: "null node description",
            })?;
        if description.is_empty() {
            return Err(CodecError::InvalidNode {
                offset,
                reason: "missing description",
            });
        }
        let attr_count = (list_size - 1) / 2;
        let attr_limit = self.limits.max_attributes as usize;
        if attr_count > attr_limit {
            return Err(CodecError::LengthLimit {
                what: "attributes",
                len: attr_count,
                limit: attr_limit,
            });
        }
        let mut attributes: Vec<Attribute> = Vec::with_capacity(attr_count);
        for _ in 0..attr_count {
            let key_offset = self.pos;
            let name = match self.read_value(true)? {
                Value::Text(s) => s,
                _ => return Err(CodecError::NonStringKey { offset: key_offset }),
            };
            if attributes.iter().any(|a| a.name == name) {
                return Err(CodecError::DuplicateAttribute { name });
            }
            let value = match self.read_value(true)? {
                // APK 43.72 123.A01 passes this value to 0Tl's checked
                // String constructor. A null marker is not explicit empty text.
                // Evidence: docs/protocol/apk/2.26.43.72-null-attribute/README.md.
                Value::Absent => {
                    return Err(CodecError::InvalidWireStructure {
                        offset: self.pos - 1,
                        reason: "null attribute value",
                    });
                }
                Value::Text(s) => AttributeValue::Text(s),
                Value::Bytes(b) => AttributeValue::Bytes(Bytes::from(b)),
                Value::Jid(j) => AttributeValue::Jid(j),
            };
            attributes.push(Attribute { name, value });
        }
        let content = if list_size % 2 == 1 {
            NodeContent::Empty
        } else {
            self.read_content(depth)?
        };
        Ok(Node {
            description,
            attributes,
            content,
        })
    }
}

fn bytes_or_text(raw: &[u8], as_string: bool) -> Value {
    if as_string {
        match std::str::from_utf8(raw) {
            Ok(s) => Value::Text(s.to_owned()),
            Err(_) => Value::Bytes(raw.to_vec()),
        }
    } else {
        Value::Bytes(raw.to_vec())
    }
}

fn unpack_digit(tag: u8, v: u8, offset: usize) -> Result<char, CodecError> {
    match tag {
        tags::NIBBLE_8 => match v {
            0..=9 => Ok((b'0' + v) as char),
            10 => Ok('-'),
            11 => Ok('.'),
            // APK 43.72 123.A07: 12 is colon; 13..15 are bad data.
            12 => Ok(':'),
            _ => Err(CodecError::InvalidPackedDigit { value: v, offset }),
        },
        tags::HEX_8 => match v {
            0..=9 => Ok((b'0' + v) as char),
            10..=15 => Ok((b'A' + v - 10) as char),
            _ => Err(CodecError::InvalidPackedDigit { value: v, offset }),
        },
        _ => Err(CodecError::UnknownTag { tag, offset }),
    }
}

#[cfg(test)]
mod self_user_pair_tests {
    use super::*;
    #[test]
    fn bare_self_user_pairs_decode_without_opening_other_symbolic_domains() {
        // APK43.72 123.A04 empty-user join ->0Dd.A00 ->0EG/2LK.
        for (name, wire) in [
            (
                "status_me",
                &b"\xf8\x03\xfc\x01n\xfc\x04from\xfa\x00\xfc\x09status_me"[..],
            ),
            (
                "lid_me",
                &b"\xf8\x03\xfc\x01n\xfc\x04from\xfa\x00\xfc\x06lid_me"[..],
            ),
        ] {
            let dict = TokenDictionary::comparison_2_26_37_71();
            let node = Decoder::new(wire, dict, &Limits::default())
                .decode_node()
                .unwrap();
            let expected = Jid::new("", Server::Other(name.into())).unwrap();
            assert_eq!(
                node.get_attr("from").and_then(|value| value.as_jid()),
                Some(&expected)
            );
            let encoded = crate::binary::Encoder::new(dict).encode(&node).unwrap();
            assert_eq!(
                Decoder::new(&encoded[1..], dict, &Limits::default())
                    .decode_node()
                    .unwrap(),
                node
            );
            // Qualified self aliases depend on native cache; do not guess them.
            let qualified = Node::new("n").attr(
                "from",
                Jid::new("12345", Server::Other(name.into())).unwrap(),
            );
            let wire = crate::binary::Encoder::new(dict)
                .encode(&qualified)
                .unwrap();
            assert!(
                Decoder::new(&wire[1..], dict, &Limits::default())
                    .decode_node()
                    .is_err()
            );
            assert!(
                Server::parse(name).is_err(),
                "general server grammar stays unchanged"
            );
        }
        let node = Node::new("n").attr(
            "from",
            Jid::new("", Server::Other("unknown_self".into())).unwrap(),
        );
        let dict = TokenDictionary::comparison_2_26_37_71();
        let wire = crate::binary::Encoder::new(dict).encode(&node).unwrap();
        assert!(
            Decoder::new(&wire[1..], dict, &Limits::default())
                .decode_node()
                .is_err()
        );
    }
}
