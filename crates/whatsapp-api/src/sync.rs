//! Durable domain events and the `SyncEngine` (docs/design/README.md sections 13 and 14).
//!
//! `SyncEngine` consumes the protocol journal. It does not run a connection
//! reader and does not promise history that the profile has not been observed
//! to provide.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::ids::{
    AccountGeneration, AccountId, ContactId, DeviceAddress, EventCursor, EventId, GroupId,
    LocalMessageId,
};
use crate::model::message::{Message, MessageKey, MessageUpdate, UnsupportedMessage};
use crate::model::receipt::Receipt;
use crate::model::send::{AttemptProgress, DeliveryFact};
use crate::model::time::Timestamp;
use crate::store::{ConsumerId, ProtocolStore};

/// Why a session became unusable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RevocationReason {
    /// The server reported the device/account as logged out.
    ServerLoggedOut {
        /// Sanitized server code.
        code: Option<u32>,
    },
    /// The operator revoked the session locally.
    LocalLogout,
    /// The account was re-registered elsewhere.
    ReplacedByNewRegistration,
    /// The stored generation is older than a restored/fenced one.
    Fenced,
}

/// Group metadata snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Group identity.
    pub id: GroupId,
    /// Subject (name), if known.
    pub subject: Option<String>,
    /// Participants (user-level identities).
    pub participants: Vec<ContactId>,
    /// Server-provided membership revision, if any.
    pub revision: Option<u64>,
}

/// A group change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GroupUpdate {
    /// Subject changed.
    Subject {
        /// Group.
        id: GroupId,
        /// New subject.
        subject: Option<String>,
    },
    /// Participants added.
    ParticipantsAdded {
        /// Group.
        id: GroupId,
        /// Added participants.
        participants: Vec<ContactId>,
    },
    /// Participants removed.
    ParticipantsRemoved {
        /// Group.
        id: GroupId,
        /// Removed participants.
        participants: Vec<ContactId>,
    },
}

/// A peer identity replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityChange {
    /// The device whose identity changed.
    pub device: DeviceAddress,
    /// SHA-256 fingerprint (hex) of the previously trusted identity key.
    pub previous_fingerprint: Option<String>,
    /// SHA-256 fingerprint (hex) of the newly observed identity key.
    pub new_fingerprint: String,
    /// When it was observed.
    pub observed_at: Timestamp,
}

/// A gap in available history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryGap {
    /// The chat affected, if scoped.
    pub chat: Option<crate::model::ids::ChatId>,
    /// Why the gap exists.
    pub reason: String,
    /// Whether the profile offers any recovery.
    pub recoverable: bool,
}

/// Send status change for a ticket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendStatusUpdate {
    /// Local message identity.
    pub local_id: LocalMessageId,
    /// New attempt progress, if it changed.
    pub progress: Option<AttemptProgress>,
    /// A newly observed delivery fact, if any.
    pub fact: Option<DeliveryFact>,
}

/// Domain events (docs/design/README.md section 14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SyncEvent {
    /// A new message was committed.
    NewMessage(Message),
    /// A message changed.
    MessageUpdated(MessageUpdate),
    /// A message was deleted locally.
    MessageDeleted(MessageKey),
    /// A receipt was committed.
    ReceiptUpdated(Receipt),
    /// A send ticket's status changed.
    SendStatusChanged(SendStatusUpdate),
    /// A group was created.
    GroupCreated(Group),
    /// A group changed.
    GroupUpdated(GroupUpdate),
    /// A peer identity changed.
    IdentityChanged(IdentityChange),
    /// History is missing and may not be recoverable.
    HistoryGap(HistoryGap),
    /// An inbound message could not be interpreted.
    UnsupportedMessage(UnsupportedMessage),
    /// The session was revoked.
    SessionRevoked(RevocationReason),
    /// A chat was cleared locally (`transport::chat_clear::clear_chat`). The
    /// `clearChatAction` app-state mutation that would sync this to companion
    /// devices (docs/protocol/chat-actions.md section 6) is not implemented;
    /// this event only reflects the local-only clear.
    ChatCleared {
        /// The cleared chat.
        chat: crate::model::ids::ChatId,
        /// When the clear was committed.
        cleared_at: Timestamp,
    },
}

impl SyncEvent {
    /// Stable kind name for diagnostics and journal indexing.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NewMessage(_) => "new_message",
            Self::MessageUpdated(_) => "message_updated",
            Self::MessageDeleted(_) => "message_deleted",
            Self::ReceiptUpdated(_) => "receipt_updated",
            Self::SendStatusChanged(_) => "send_status_changed",
            Self::GroupCreated(_) => "group_created",
            Self::GroupUpdated(_) => "group_updated",
            Self::IdentityChanged(_) => "identity_changed",
            Self::HistoryGap(_) => "history_gap",
            Self::UnsupportedMessage(_) => "unsupported_message",
            Self::SessionRevoked(_) => "session_revoked",
            Self::ChatCleared { .. } => "chat_cleared",
        }
    }
}

/// A committed event with its journal position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Account the event belongs to.
    pub account_id: AccountId,
    /// Account generation at commit time.
    pub account_generation: AccountGeneration,
    /// Stable event identity (for consumer deduplication).
    pub event_id: EventId,
    /// Journal position.
    pub cursor: EventCursor,
    /// Commit time.
    pub committed_at: Timestamp,
    /// The event.
    pub event: SyncEvent,
}

/// A bounded batch of events after a cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventBatch {
    /// Events in cursor order.
    pub events: Vec<EventEnvelope>,
    /// Cursor to acknowledge once every event in this batch is durably
    /// processed. Equal to the last event's cursor, or the requested cursor
    /// when the batch is empty.
    pub ack_cursor: EventCursor,
    /// Whether more events were available beyond `limit`.
    pub has_more: bool,
}

/// Result of one bounded catch-up pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncProgress {
    /// Events drained from the journal in this pass.
    pub drained: usize,
    /// Cursor after the pass.
    pub cursor: EventCursor,
    /// Server catch-up performed. Always `false` until the profile has an
    /// observed catch-up API; then callers see a `HistoryGap` event instead.
    pub server_catch_up: bool,
}

/// Consumer-side engine over the durable journal.
#[derive(Debug, Clone)]
pub struct SyncEngine {
    store: Arc<ProtocolStore>,
    consumer: ConsumerId,
}

impl SyncEngine {
    /// Create an engine for a named consumer.
    pub fn new(store: Arc<ProtocolStore>, consumer: ConsumerId) -> Self {
        Self { store, consumer }
    }

    /// The consumer identity.
    pub fn consumer(&self) -> &ConsumerId {
        &self.consumer
    }

    /// Last contiguous acknowledged cursor.
    pub async fn acknowledged_cursor(&self) -> Result<EventCursor> {
        let store = self.store.clone();
        let consumer = self.consumer.clone();
        store
            .run_blocking(move |s| s.consumer_cursor(&consumer))
            .await
    }

    /// Fetch a bounded batch after the given cursor (at-least-once).
    pub async fn next_batch(&self, after: EventCursor, limit: u32) -> Result<EventBatch> {
        if limit == 0 {
            return Err(Error::invalid("limit", "must be positive"));
        }
        let store = self.store.clone();
        let consumer = self.consumer.clone();
        store
            .run_blocking(move |s| s.read_events(&consumer, after, limit))
            .await
    }

    /// Acknowledge contiguous progress. Acknowledging a cursor beyond the
    /// journal head is rejected.
    pub async fn ack(&self, cursor: EventCursor) -> Result<()> {
        let store = self.store.clone();
        let consumer = self.consumer.clone();
        store
            .run_blocking(move |s| s.ack_events(&consumer, cursor))
            .await
    }

    /// Bounded drain: deliver up to `limit` events to `handler`, acknowledging
    /// after the handler returns `Ok`. A handler error stops the pass without
    /// acknowledging, so the same events replay next time.
    pub async fn sync_once<F>(&self, limit: u32, mut handler: F) -> Result<SyncProgress>
    where
        F: FnMut(&EventEnvelope) -> Result<()>,
    {
        let start = self.acknowledged_cursor().await?;
        let batch = self.next_batch(start, limit).await?;
        let mut last = start;
        for ev in &batch.events {
            handler(ev)?;
            last = ev.cursor;
        }
        if last != start {
            self.ack(last).await?;
        }
        Ok(SyncProgress {
            drained: batch.events.len(),
            cursor: last,
            server_catch_up: false,
        })
    }
}
