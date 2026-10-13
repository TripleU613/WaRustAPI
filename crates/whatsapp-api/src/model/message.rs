//! Messages as a tagged domain enum with an explicit `Unsupported` form
//! (docs/design/README.md section 9).

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

/// Media kind for already-encoded content (docs/design/README.md section 16).
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
        /// Colour and font of a styled full-screen text status. Default (all
        /// `None`) for an ordinary chat message, which carries no styling.
        #[serde(default)]
        style: crate::transport::quotes::TextStyle,
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
        /// Reply target named by the media's contextInfo, if any. Omitted when
        /// absent, so existing records and intent hashes are unchanged.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        quoted: Option<MessageRef>,
    },
    /// A poll (comparison `pollCreationMessageV3`).
    Poll {
        /// Poll question.
        name: String,
        /// Option names in wire order.
        options: Vec<String>,
        /// How many options a voter may select; `0` means no limit.
        selectable_count: u32,
    },
    /// This account's vote on a poll; `selected` holds option names.
    PollVote {
        /// The poll being voted on.
        target: MessageRef,
        /// Selected option names (empty retracts the vote).
        selected: Vec<String>,
    },
    /// A change to this chat's disappearing-message timer (comparison
    /// `protocolMessage` type `EPHEMERAL_SETTING`; docs/protocol/chat-actions.md §3).
    DisappearingTimer {
        /// New timer in seconds; `0` means off.
        seconds: u32,
    },
    /// A change to this chat's advanced privacy ("limit sharing") setting
    /// (comparison `protocolMessage` type `LIMIT_SHARING`/`ACP2_SETTING`;
    /// docs/protocol/chat-actions.md §5).
    ChatPrivacy {
        /// Whether sharing/forwarding out of the chat is limited.
        limited: bool,
    },
    /// Keep (or un-keep) a message in the chat despite its disappearing timer
    /// (comparison `keepInChatMessage`; docs/protocol/chat-actions.md §4).
    KeepInChat {
        /// The kept/un-kept message.
        target: MessageRef,
        /// `true` keeps the message for everyone; `false` undoes a keep.
        keep: bool,
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
    /// Content replaced by an edit. Supported authenticated own-device updates
    /// retain the parent's key and stanza time; captions keep the original media reference.
    Edited(Box<MessageContent>),
    /// Reaction added, changed or removed by a device.
    Reaction {
        /// Reacting device.
        by: DeviceAddress,
        /// Emoji, or `None` for removal.
        emoji: Option<String>,
    },
    /// A vote on a poll, decrypted with the poll's message secret.
    PollVote {
        /// Voting device.
        by: DeviceAddress,
        /// Selected option names resolved from the poll; unknown hashes are
        /// reported in `unresolved` as lowercase hex.
        selected: Vec<String>,
        /// Selected option hashes that matched no known option.
        unresolved: Vec<String>,
    },
    /// Received sender revocation from the acting device.
    /// Journaling this action does not apply native timing, receipt cleanup or physical deletion.
    Revoked {
        /// Revoking device.
        by: DeviceAddress,
    },
    /// Received keep/unkeep action from a device (chat-actions.md §4).
    /// Recording the action does not enforce native keep permission/expiry.
    KeepInChat {
        /// Acting device.
        by: DeviceAddress,
        /// `true` if the message was kept for everyone; `false` if the keep
        /// was undone.
        kept: bool,
        /// Keeping sender's payload timestamp in milliseconds (APK 37.71
        /// BV3 field 3 -> D9L.A01 -> BnF.A02). This is distinct from the
        /// update's stanza-derived server timestamp and local observation time.
        /// Absent for omitted wire fields and historical journal records.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sender_timestamp: Option<Timestamp>,
        /// Wire ID of the keep/unkeep envelope, distinct from the target ID.
        /// APK 37.71 0Y5.A00 uses BnF's own key ID for equal-time ordering.
        /// Unknown for historical records or unsupported/missing envelope IDs.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        update_id: Option<MessageId>,
    },
    /// The referenced original has not arrived; a tombstone is pending.
    PendingReference,
}

#[cfg(test)]
mod keep_serialization_tests {
    use super::*;

    #[test]
    fn legacy_keep_event_json_stays_readable_without_inventing_ordering_metadata() {
        // Literal pre-timestamp EventEnvelope, with the old KeepInChat shape.
        // Synthetic actor/target/ID only; no private historical record is copied.
        let legacy = r#"{
            "account_id":"fixture","account_generation":1,
            "event_id":"00000000000000000000000000000001",
            "cursor":13,"committed_at":777,
            "event":{"MessageUpdated":{
                "key":{"chat":{"Direct":{"user":"222","agent":0,"device":0,"server":"Lid","kind":"Plain"}},
                    "sender":null,"direction":"Outgoing","id":"ORIGINAL"},
                "change":{"KeepInChat":{"by":{"contact":{"user":"222","agent":0,"device":0,"server":"Lid","kind":"Plain"},"device":3},"kept":true}},
                "server_timestamp":123000
            }}
        }"#;
        let mut event: crate::sync::EventEnvelope = serde_json::from_str(legacy).unwrap();
        let crate::sync::SyncEvent::MessageUpdated(update) = &mut event.event else {
            panic!("update");
        };
        let MessageChange::KeepInChat {
            sender_timestamp,
            update_id,
            ..
        } = &mut update.change
        else {
            panic!("keep");
        };
        assert_eq!(*sender_timestamp, None);
        assert_eq!(*update_id, None);
        assert_eq!(update.server_timestamp, Some(Timestamp(123000)));
        assert_eq!(event.committed_at, Timestamp(777));
        assert_eq!(
            serde_json::to_value(&event).unwrap(),
            serde_json::from_str::<serde_json::Value>(legacy).unwrap()
        );
        let crate::sync::SyncEvent::MessageUpdated(update) = &mut event.event else {
            unreachable!();
        };
        let MessageChange::KeepInChat {
            sender_timestamp,
            update_id,
            ..
        } = &mut update.change
        else {
            unreachable!();
        };
        *sender_timestamp = Some(Timestamp(0));
        *update_id = Some(MessageId::new("KEEP-ACTION").unwrap());
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(
            json["event"]["MessageUpdated"]["change"]["KeepInChat"]["sender_timestamp"],
            0
        );
        assert_eq!(
            json["event"]["MessageUpdated"]["change"]["KeepInChat"]["update_id"],
            "KEEP-ACTION"
        );
        assert_eq!(json["event"]["MessageUpdated"]["key"]["id"], "ORIGINAL");
        assert_eq!(
            serde_json::from_value::<crate::sync::EventEnvelope>(json).unwrap(),
            event
        );
    }
}
