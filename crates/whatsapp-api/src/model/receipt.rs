//! Receipts and acknowledgements (docs/design/README.md section 13).
//!
//! Transport ACK, delivery receipt, read receipt and played receipt are
//! separate kinds. None of them are emitted automatically as "read".

use serde::{Deserialize, Serialize};

use super::ids::{ChatId, DeviceAddress, MessageId};
use super::time::Timestamp;

/// Receipt class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReceiptKind {
    /// The server acknowledged receiving our stanza (transport-level).
    ServerAck,
    /// A recipient device received the message.
    Delivered,
    /// A recipient read the message.
    Read,
    /// A recipient played an audio/video message.
    Played,
    /// The recipient requested a retry / re-encryption.
    Retry,
    /// The server or recipient reported a terminal error.
    Error,
}

/// A receipt fact about one or more messages in a chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    /// Chat the messages belong to.
    pub chat: ChatId,
    /// Message identifiers covered by the receipt.
    pub message_ids: Vec<MessageId>,
    /// Device the receipt came from, when known. For an explicit outgoing group
    /// read request, selects the original participant; an absent selector is
    /// accepted only when the group/message ID identifies one saved record.
    pub from: Option<DeviceAddress>,
    /// Receipt class.
    pub kind: ReceiptKind,
    /// Server timestamp, when provided.
    pub server_timestamp: Option<Timestamp>,
    /// Sanitized error code for `Error` receipts.
    pub code: Option<u32>,
}
