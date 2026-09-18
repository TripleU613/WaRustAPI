//! Domain model: identifiers, addresses, messages, receipts and send types.
//!
//! Identifiers are newtypes (docs/design/README.md section 9). A contact identifier is not a
//! phone number, and a cryptographic session belongs to a [`ids::DeviceAddress`],
//! never to a human-readable contact.

pub mod ids;
pub mod jid;
pub mod message;
pub mod phone;
pub mod receipt;
pub mod send;
pub mod time;

pub use ids::{
    AccountGeneration, AccountId, ChatId, ClientRequestId, ContactId, DeviceAddress, DeviceIndex,
    EventCursor, EventId, GroupId, LocalMessageId, MessageId,
};
pub use jid::{Jid, JidKind, Server};
pub use message::{
    Direction, Message, MessageContent, MessageKey, MessageRef, MessageUpdate, UnsupportedMessage,
};
pub use phone::PhoneNumber;
pub use receipt::{Receipt, ReceiptKind};
pub use send::{
    AttemptProgress, DeliveryFact, DeliveryFactKind, SendRequest, SendStatus, SendTicket,
};
pub use time::Timestamp;
