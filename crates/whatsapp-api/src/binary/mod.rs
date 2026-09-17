//! Binary node codec (README section 8.4).
//!
//! WhatsApp stanzas are trees of nodes encoded with a token dictionary. This
//! module provides the in-memory [`Node`] shape, a bounded decoder and an
//! encoder. The token dictionary and every tag constant are `ReferenceOnly`
//! for the Android primary profile (ledger `binary.*`); the codec is therefore
//! marked `Experimental` in the capability matrix.
//!
//! Decoder guarantees:
//!
//! * every length is bounded before allocation,
//! * nesting depth, attribute count, child count and string length are
//!   bounded by [`crate::profile::Limits`],
//! * zlib inflation is bounded,
//! * malformed bytes are never converted through lossy UTF-8,
//! * duplicate attributes are rejected,
//! * an unknown structural tag is a `CodecError`, not a guess.

mod decoder;
mod encoder;
pub mod node;
pub mod tokens;

pub use decoder::{Decoder, unpack_frame_payload};
pub use encoder::Encoder;
pub use node::{Attribute, AttributeValue, Node, NodeContent};
pub use tokens::{TokenDictionary, tags};

/// Decoding/encoding failures. Positions are byte offsets into the input.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CodecError {
    /// Input ended before a complete structure was read.
    #[error("unexpected end of input at offset {offset} (needed {needed} more bytes)")]
    UnexpectedEof {
        /// Offset where the read started.
        offset: usize,
        /// Bytes that were required.
        needed: usize,
    },
    /// A tag byte that is not part of the structural grammar.
    #[error("unknown tag {tag} at offset {offset}")]
    UnknownTag {
        /// The tag byte.
        tag: u8,
        /// Its offset.
        offset: usize,
    },
    /// A dictionary index has no token.
    #[error("unknown token (dictionary {dictionary}, index {index}) at offset {offset}")]
    UnknownToken {
        /// Dictionary page (0 = single byte).
        dictionary: u8,
        /// Index within the page.
        index: u8,
        /// Offset.
        offset: usize,
    },
    /// A packed nibble/hex digit was outside its alphabet.
    #[error("invalid packed digit {value} at offset {offset}")]
    InvalidPackedDigit {
        /// The 4-bit value.
        value: u8,
        /// Offset.
        offset: usize,
    },
    /// A string or bytes value exceeded the configured bound.
    #[error("{what} length {len} exceeds limit {limit}")]
    LengthLimit {
        /// What was being read.
        what: &'static str,
        /// Declared length.
        len: usize,
        /// Configured limit.
        limit: usize,
    },
    /// Nesting depth exceeded the configured bound.
    #[error("node depth exceeds limit {limit}")]
    DepthLimit {
        /// Configured limit.
        limit: usize,
    },
    /// Structural rule violation.
    #[error("invalid node structure at offset {offset}: {reason}")]
    InvalidNode {
        /// Offset.
        offset: usize,
        /// Reason.
        reason: &'static str,
    },
    /// An attribute name appeared twice on one node.
    #[error("duplicate attribute `{name}`")]
    DuplicateAttribute {
        /// Attribute name.
        name: String,
    },
    /// Attribute keys must be strings.
    #[error("non-string attribute key at offset {offset}")]
    NonStringKey {
        /// Offset.
        offset: usize,
    },
    /// A JID component was invalid.
    #[error("invalid JID at offset {offset}: {reason}")]
    InvalidJid {
        /// Offset.
        offset: usize,
        /// Reason.
        reason: String,
    },
    /// zlib inflation failed or exceeded the bound.
    #[error("compressed payload rejected: {0}")]
    Compression(&'static str),
    /// Trailing bytes after a complete node.
    #[error("{0} trailing bytes after node")]
    TrailingBytes(usize),
    /// Encoder input exceeded a representable size.
    #[error("cannot encode: {0}")]
    Unencodable(&'static str),
}
