//! Typed errors for the public API (docs/design/README.md section 14).
//!
//! The variants deliberately distinguish invalid input, unsupported
//! capability/profile, required registration action, rate limits with a
//! deadline, authentication failure, revoked session, identity change,
//! invalid certificate, malformed protocol, storage failure, wrong storage
//! key, exhausted resources and uncertain delivery. Server payloads are never
//! embedded; only sanitized codes are preserved.

use std::time::Duration;

use crate::capabilities::Capability;
use crate::model::ids::{ClientRequestId, DeviceAddress, LocalMessageId};
use crate::registration::RequiredAction;

/// Convenience alias used across the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Library error type.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A caller-supplied value failed validation.
    #[error("invalid input for `{field}`: {reason}")]
    InvalidInput {
        /// Name of the offending field or argument.
        field: &'static str,
        /// Human-readable, secret-free reason.
        reason: String,
    },

    /// The operation is not supported by this build or profile.
    #[error("unsupported capability {capability:?}: {reason}")]
    Unsupported {
        /// The capability that was requested.
        capability: Capability,
        /// Why it is unsupported (typically the missing evidence).
        reason: String,
    },

    /// The protocol profile lacks a fact that the operation needs.
    #[error("protocol profile `{profile}` is missing required facts: {missing:?}")]
    ProfileIncomplete {
        /// Profile identifier.
        profile: String,
        /// Ledger references of the missing facts.
        missing: Vec<&'static str>,
    },

    /// Registration cannot proceed without a caller action.
    #[error("registration requires a caller action: {0:?}")]
    RegistrationActionRequired(RequiredAction),

    /// The server or a local policy imposed a rate limit.
    #[error("rate limited; retry after {retry_after:?}")]
    RateLimited {
        /// Earliest time to retry, if known.
        retry_after: Option<Duration>,
        /// Sanitized server code, if any.
        code: Option<u32>,
    },

    /// Authentication with the server failed.
    #[error("authentication failed ({code:?})")]
    Authentication {
        /// Sanitized server code, if any.
        code: Option<u32>,
    },

    /// The account session was revoked; reconnecting will not help.
    #[error("session revoked: {0:?}")]
    Revoked(crate::sync::RevocationReason),

    /// A peer identity key changed; affected sends are suspended until the
    /// configured policy resolves the change.
    #[error("peer identity changed; affected sends suspended pending trust-policy resolution")]
    IdentityChanged(DeviceAddress),

    /// A transport certificate or trust binding failed verification.
    #[error("invalid certificate: {0}")]
    InvalidCertificate(String),

    /// The peer violated the protocol or a decoder bound.
    #[error("malformed protocol data: {0}")]
    MalformedProtocol(#[from] crate::binary::CodecError),

    /// Transport framing violated a bound or the stream ended unexpectedly.
    #[error("transport error: {0}")]
    Transport(#[from] crate::transport::TransportError),

    /// The protocol store failed.
    #[error("storage failure: {0}")]
    Storage(crate::store::StoreError),

    /// The supplied master key cannot decrypt the existing store.
    #[error("wrong storage key")]
    WrongStorageKey,

    /// A configured resource limit was reached.
    #[error("resource exhausted: {resource} (limit {limit})")]
    ResourceExhausted {
        /// Which limit.
        resource: &'static str,
        /// The configured limit.
        limit: u64,
    },

    /// The outcome of a transmitted operation could not be determined.
    #[error("delivery outcome uncertain for {0:?}")]
    DeliveryUncertain(LocalMessageId),

    /// A `ClientRequestId` was reused with different semantic content.
    #[error("idempotency conflict for client request {0:?}")]
    IdempotencyConflict(ClientRequestId),

    /// The operation was cancelled or the runtime is shutting down.
    #[error("operation cancelled")]
    Cancelled,

    /// A deadline elapsed.
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),

    /// The account runtime is not running.
    #[error("runtime not connected")]
    NotConnected,

    /// No completed registration exists in this store; connecting is
    /// impossible and is never repaired by automatic re-registration.
    #[error("account is not registered")]
    NotRegistered,

    /// I/O failure outside the store.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Shorthand for an invalid-input error.
    pub fn invalid(field: &'static str, reason: impl Into<String>) -> Self {
        Self::InvalidInput {
            field,
            reason: reason.into(),
        }
    }

    /// Shorthand for an unsupported-capability error.
    pub fn unsupported(capability: Capability, reason: impl Into<String>) -> Self {
        Self::Unsupported {
            capability,
            reason: reason.into(),
        }
    }

    /// Whether a reconnect could plausibly resolve this error (docs/design/README.md section 14).
    pub fn is_recoverable_by_reconnect(&self) -> bool {
        matches!(
            self,
            Self::Transport(_) | Self::Timeout(_) | Self::NotConnected | Self::Io(_)
        )
    }
}
