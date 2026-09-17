//! Messages as a tagged domain enum with an explicit `Unsupported` form
//! (README section 9).

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use super::ids::{ChatId, DeviceAddress, MessageId};
use super::time::Timestamp;

/// Direction of a message relative to our account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Sent by this account.
    Outgoing,
    /// Received from another party.
    Incoming,
}

/// Composite logical message key. Message IDs are not assumed globally unique.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageKey {
    /// The chat the message belongs to.
    pub chat: ChatId,
    /// The sending device, when known. `None` for our own outgoing messages
    /// before the sending device is fixed.
    pub sender: Option<DeviceAddress>,
    /// Direction relative to this account.
    pub direction: Direction,
    /// Wire identifier.
    pub id: MessageId,
}

/// A reference to another message (reply target, reaction target, edit target).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageRef {
    /// Key of the referenced message.
    pub key: MessageKey,
}

/// Media kind for already-encoded content (README section 16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaKind {
    /// Still image.
    Image,
    /// Video.
    Video,
    /// Audio / voice note.
    Audio,
    /// Document.
    Document,
    /// Sticker.
    Sticker,
}

/// An opaque, encrypted-at-rest media reference. The plaintext structure is
/// profile-specific and only materialized inside the media module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaReference {
    /// Encrypted reference envelope (see `store::envelope`).
    pub sealed: Bytes,
    /// Declared media kind.
    pub kind: MediaKind,
    /// Declared MIME type (label only).
    pub mime_type: String,
    /// Declared plaintext length, if known.
    pub length: Option<u64>,
}

/// Domain message content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MessageContent {
    /// Plain text (may quote another message).
    Text {
        /// Body text.
        body: String,
        /// Reply target, if any.
        quoted: Option<MessageRef>,
    },
    /// Reaction to another message. `emoji == None` removes a reaction.
    Reaction {
        /// The reacted-to message.
        target: MessageRef,
        /// Emoji, or `None` for removal.
        emoji: Option<String>,
    },
    /// Edit of a previous message by the same sender.
    Edit {
        /// The edited message.
        target: MessageRef,
        /// Replacement content.
        replacement: Box<MessageContent>,
    },
    /// Revocation ("delete for everyone") of a previous message.
    Revoke {
        /// The revoked message.
        target: MessageRef,
    },
    /// Media with an already-encoded payload.
    Media {
        /// Media reference (encrypted at rest).
        reference: MediaReference,
        /// Optional caption.
        caption: Option<String>,
        /// Remote filename, treated strictly as a label.
        file_name: Option<String>,
    },
    /// Content the library cannot interpret for this profile.
    Unsupported(UnsupportedMessage),
}

/// A message whose type or encoding is not understood.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedMessage {
    /// A short hint about the type (for example a protobuf field number).
    pub type_hint: String,
    /// Bounded raw payload retained for later decoding. May be truncated;
    /// `truncated` records that.
    pub raw: Bytes,
    /// Whether `raw` was truncated to the configured bound.
    pub truncated: bool,
}

/// Disappearing-message and view-once indicators, preserved from the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Ephemerality {
    /// Expiry timer in seconds, when the chat has disappearing messages.
    pub expiration_seconds: Option<u32>,
    /// Whether the content is view-once.
    pub view_once: bool,
}

/// A committed message in the domain model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Composite logical key.
    pub key: MessageKey,
    /// Server timestamp, if provided by the protocol.
    pub server_timestamp: Option<Timestamp>,
    /// Local observation time (always present).
    pub observed_at: Timestamp,
    /// Content.
    pub content: MessageContent,
    /// Ephemerality indicators.
    pub ephemerality: Ephemerality,
}

/// A mutation applied to an existing message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageUpdate {
    /// The message that changed.
    pub key: MessageKey,
    /// The new content or state.
    pub change: MessageChange,
    /// Server timestamp of the change, if any.
    pub server_timestamp: Option<Timestamp>,
}

/// Kinds of message change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MessageChange {
    /// Content replaced by an edit.
    Edited(Box<MessageContent>),
    /// Reaction added, changed or removed by a device.
    Reaction {
        /// Reacting device.
        by: DeviceAddress,
        /// Emoji, or `None` for removal.
        emoji: Option<String>,
    },
    /// Revoked by the acting device.
    Revoked {
        /// Revoking device.
        by: DeviceAddress,
    },
    /// The referenced original has not arrived; a tombstone is pending.
    PendingReference,
}
