//! Transport: byte-stream framing, Noise adapter and certificate binding
//! (docs/design/README.md sections 8.3 and 8.4).
//!
//! Offline framing and inspection helpers compile by default. The Noise adapter lives behind the
//! `experimental-noise` feature because the handshake pattern, prologue and
//! trust root are `ReferenceOnly` for the Android primary profile.
//! Stateful messaging modules additionally require `experimental-signal-crypto`;
//! enabling TCP/Noise alone keeps socket, login payload and handshake support.

#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod abprops;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod actions;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod app_state_sync;
#[cfg(feature = "experimental-registration-https")]
pub mod ban_appeal;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod blocklist;
pub mod cert;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod chat_clear;
pub mod chat_settings;
#[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
mod classic;
pub mod clock;
pub mod comparison;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod connection;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod contacts;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod device_key_batches;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod devices;
pub mod framing;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod group_fanout;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod group_inbox;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod group_references;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod group_send;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod groups;
#[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
pub mod handshake;
pub mod history_notice;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod identity_conflicts;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod identity_notifications;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod identity_query;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod outgoing_group;
pub mod participant_hash;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod peer_keys;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod peer_verification;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod sender_key_distribution;

#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod companion_admission;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod group_receipts;
#[cfg(all(
    feature = "experimental-registration-https",
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod image_download;
#[cfg(all(
    feature = "experimental-registration-https",
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod image_upload;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod inbox;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
mod inbox_events;
pub mod jid_map;
#[cfg(feature = "experimental-transport")]
pub mod login;
#[cfg(all(
    feature = "experimental-registration-https",
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod media;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod messaging;
pub mod mex;
#[cfg(feature = "experimental-noise")]
pub mod noise;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod notifications;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod outgoing;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod outgoing_media;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
mod own_device;
pub mod polls;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod prekeys;
pub(crate) mod protobuf_enum;
pub mod quotes;
pub mod reactions;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod receipts;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod report;
pub mod selector;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod send_path;
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod send_versions;
#[cfg(feature = "experimental-transport")]
pub mod socket;
pub mod status_receipts;
#[cfg(all(
    feature = "experimental-registration-https",
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
mod voice_note;

use serde::{Deserialize, Serialize};

pub use framing::{FrameCodec, FrameReader};

/// Transport-level failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TransportError {
    /// Socket I/O failed; raw OS messages are not included in diagnostics.
    #[error("socket I/O failed: {0:?}")]
    Io(std::io::ErrorKind),
    /// A bounded operation exceeded its deadline.
    #[error("transport deadline exceeded during {0}")]
    Deadline(&'static str),
    /// An earlier failure or cancellation consumed the connection.
    #[error("transport connection is closed")]
    Closed,
    /// A declared frame length exceeded the configured bound.
    #[error("frame length {len} exceeds limit {limit}")]
    FrameTooLarge {
        /// Declared length.
        len: usize,
        /// Configured bound.
        limit: usize,
    },
    /// Buffered bytes exceeded the bound without completing a frame.
    #[error("receive buffer exceeded {limit} bytes")]
    BufferOverflow {
        /// Configured bound.
        limit: usize,
    },
    /// The peer closed the stream cleanly at a frame boundary. A reconnect
    /// with a fresh handshake may recover; nothing uncertain is replayed.
    #[error("peer closed the connection")]
    PeerClosed,
    /// The stream ended mid-frame.
    #[error("stream closed with {buffered} partial bytes buffered")]
    UnexpectedEof {
        /// Bytes that were buffered when the stream closed.
        buffered: usize,
    },
    /// The Noise record exceeded the backend's maximum record size.
    #[error("noise record of {len} bytes exceeds backend maximum {max}")]
    RecordTooLarge {
        /// Record length.
        len: usize,
        /// Backend maximum.
        max: usize,
    },
    /// A Noise operation failed. The connection must be closed.
    #[error("noise failure: {0}")]
    Noise(String),
    /// The handshake payload or certificate chain was invalid.
    #[error("handshake rejected: {0}")]
    Handshake(String),
    /// Invalid framing configuration.
    #[error("invalid framing configuration: {0}")]
    Config(&'static str),
    /// The connection epoch no longer matches (a stale completion).
    #[error("stale connection epoch {stale:?}; current is {current:?}")]
    StaleEpoch {
        /// The epoch of the completion.
        stale: ConnectionEpoch,
        /// The current epoch.
        current: ConnectionEpoch,
    },
}

/// Local epoch of one connection attempt. Every reconnect gets a new one so a
/// delayed completion from an old socket cannot complete a new request.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct ConnectionEpoch(pub u64);

impl ConnectionEpoch {
    /// The next epoch.
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod two_factor;

#[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
pub mod presence;

#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub mod about;
