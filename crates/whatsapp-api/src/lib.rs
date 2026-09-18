//! `whatsapp-api`: a standalone async Rust library intended to register and
//! operate as a WhatsApp **primary device**.
//!
//! # Status
//!
//! This crate is at gate **G0/G1** of the implementation plan in the repository
//! `docs/design/README.md`. Everything that works today is offline and local:
//!
//! * a typed domain model and public API contract (`client`, `model`, `sync`),
//! * an encrypted transactional protocol store on `redb` with a durable event
//!   journal (`store`),
//! * a bounded binary-node codec driven by a token dictionary whose evidence
//!   label is tracked (`binary`),
//! * byte-stream framing with bounded length prefixes (`transport`),
//! * a durable registration state machine with typed steps (`registration`),
//! * a durable outbox that returns send tickets, not delivery promises (`outbox`).
//!
//! Nothing here has exchanged a message with a WhatsApp server. Every
//! protocol-facing operation that lacks `Observed` evidence for the Android
//! primary profile returns a typed [`Error::Unsupported`] or
//! [`Error::ProfileIncomplete`] instead of pretending to succeed. See
//! [`capabilities::CapabilityMatrix`] and `docs/protocol/evidence-ledger.md`.
//!
//! # Layering
//!
//! ```text
//! client (facade) -> runtime (account actor) -> outbox/inbox -> store
//!                                      \-> transport (framing, noise) -> binary (nodes)
//! sync (durable events) ---------------------------------------> store
//! registration (state machine) --------------------------------> store
//! ```
//!
//! The library must not depend on any bridge or application crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod binary;
pub mod capabilities;
pub mod client;
pub mod error;
pub mod evidence;
pub mod model;
pub mod outbox;
pub mod profile;
#[doc(hidden)]
pub mod proto;
pub mod registration;
pub mod runtime;
pub mod session;
pub mod signal;
pub mod store;
pub mod sync;
pub mod transport;

pub use client::{ClientBuilder, WhatsAppClient};
pub use error::{Error, Result};
pub use profile::ProtocolProfile;
pub use sync::{EventEnvelope, SyncEngine, SyncEvent};

/// Crate version, exposed for capability reports and diagnostics.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
