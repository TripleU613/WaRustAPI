//! Send requests, tickets, attempt progress and delivery facts (README section 12).

use serde::{Deserialize, Serialize};

use super::ids::{ChatId, ClientRequestId, DeviceAddress, LocalMessageId, MessageId};
use super::message::MessageContent;
use super::time::Timestamp;

/// A request to send a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendRequest {
    /// Caller idempotency key. Required for retry-safe enqueue.
    pub client_request_id: ClientRequestId,
    /// Destination chat.
    pub chat: ChatId,
    /// Content to send.
    pub content: MessageContent,
}

/// Proof of durable acceptance of a send intent. Not proof of delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendTicket {
    /// Local message identity.
    pub local_id: LocalMessageId,
    /// The idempotency key that produced this ticket.
    pub client_request_id: ClientRequestId,
    /// The wire message ID, once a profile-valid one has been generated and
    /// persisted. `None` while the wire ID format is `Unknown` for the profile.
    pub wire_id: Option<MessageId>,
    /// When the intent was committed.
    pub accepted_at: Timestamp,
}

/// Progress of the local send attempt (distinct from delivery facts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttemptProgress {
    /// Intent committed, nothing prepared.
    Queued,
    /// E2EE payload prepared and committed, not yet written to a socket.
    Prepared,
    /// Written to a socket during the given connection epoch; no ACK yet.
    InFlight,
    /// A retry is scheduled after a classified recoverable failure.
    RetryScheduled,
    /// Cancelled before transmission.
    Cancelled,
    /// Terminal local failure.
    Failed,
}

/// Kinds of delivery fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryFactKind {
    /// The server accepted the stanza.
    ServerAccepted,
    /// A device reported delivery.
    Delivered,
    /// A device reported read.
    Read,
    /// A device reported played.
    Played,
    /// A terminal failure was reported.
    Failed,
    /// The outcome of a transmitted attempt was lost.
    DeliveryUnknown,
}

/// One delivery fact, attributed to a device where applicable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryFact {
    /// Fact kind.
    pub kind: DeliveryFactKind,
    /// The device the fact is about, when applicable.
    pub device: Option<DeviceAddress>,
    /// When the fact was observed locally.
    pub observed_at: Timestamp,
    /// Sanitized code, if any.
    pub code: Option<u32>,
}

/// Aggregate and per-device status of a send.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendStatus {
    /// The ticket this status belongs to.
    pub ticket: SendTicket,
    /// Local attempt progress.
    pub progress: AttemptProgress,
    /// Delivery facts, in observation order.
    pub facts: Vec<DeliveryFact>,
}

impl SendStatus {
    /// Whether the server accepted the stanza.
    pub fn server_accepted(&self) -> bool {
        self.facts
            .iter()
            .any(|f| f.kind == DeliveryFactKind::ServerAccepted)
    }

    /// Whether the outcome is currently unknowable from local state.
    pub fn is_uncertain(&self) -> bool {
        self.facts
            .iter()
            .any(|f| f.kind == DeliveryFactKind::DeliveryUnknown)
            && !self.server_accepted()
    }
}
