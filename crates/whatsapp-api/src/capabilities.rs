//! Capability matrix (README section 1).
//!
//! Every operation maps to a [`Capability`] with a [`SupportLevel`]. An
//! `Unsupported` operation returns [`crate::Error::Unsupported`]; it never
//! silently succeeds or drops fields.

use serde::{Deserialize, Serialize};

use crate::evidence::EvidenceLabel;

/// Operations the library can be asked to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Capability {
    /// Encrypted, transactional local protocol store.
    LocalStore,
    /// Durable at-least-once event journal with consumer cursors.
    EventJournal,
    /// Durable send intents with idempotency (`send_message` returns a ticket).
    DurableOutbox,
    /// Binary node encode/decode for the profile's token dictionary.
    BinaryCodec,
    /// Byte-stream framing.
    Framing,
    /// Primary phone-number registration.
    Registration,
    /// Authenticated connection to the server.
    Connect,
    /// Direct text send/receive.
    DirectText,
    /// Recipient device discovery and fanout.
    DeviceFanout,
    /// Receipts and retry handling.
    Receipts,
    /// Group listing, creation, metadata, participants and text.
    Groups,
    /// Media upload/download of pre-encoded content.
    Media,
    /// Replies, reactions, edits, revocations.
    MessageOperations,
    /// Profiles, contacts, presence, privacy settings.
    ProfileAndPresence,
    /// Voice and video calls.
    Calls,
    /// Backup import.
    BackupImport,
}

/// Support level of a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportLevel {
    /// Tested and covered by evidence.
    Supported,
    /// Implemented behind an experimental feature or with `ReferenceOnly` facts.
    Experimental,
    /// Not implemented; returns a typed error.
    Unsupported,
}

/// One row of the matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEntry {
    /// The capability.
    pub capability: Capability,
    /// Support level.
    pub level: SupportLevel,
    /// Best evidence label backing the level.
    pub evidence: EvidenceLabel,
    /// Short reason or pointer to the ledger.
    pub note: &'static str,
}

/// The capability matrix plus tested-profile information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityMatrix {
    /// Identifier of the protocol profile this matrix describes.
    pub profile_id: String,
    /// Library version.
    pub library_version: &'static str,
    /// Rows.
    pub entries: Vec<CapabilityEntry>,
}

impl CapabilityMatrix {
    /// The matrix for the current build against the given profile.
    pub fn current(profile_id: impl Into<String>) -> Self {
        use Capability as C;
        use EvidenceLabel as E;
        use SupportLevel as S;
        let row = |capability, level, evidence, note| CapabilityEntry {
            capability,
            level,
            evidence,
            note,
        };
        Self {
            profile_id: profile_id.into(),
            library_version: crate::VERSION,
            entries: vec![
                row(
                    C::LocalStore,
                    S::Supported,
                    E::Observed,
                    "redb + XChaCha20-Poly1305 envelopes; tested offline",
                ),
                row(
                    C::EventJournal,
                    S::Supported,
                    E::Observed,
                    "durable cursors, at-least-once replay; tested offline",
                ),
                row(
                    C::DurableOutbox,
                    S::Supported,
                    E::Observed,
                    "intent commit + idempotency; no transmission path yet",
                ),
                row(
                    C::BinaryCodec,
                    S::Experimental,
                    E::ReferenceOnly,
                    "token dictionary v3 from web multi-device references (ledger: binary.tokens)",
                ),
                row(
                    C::Framing,
                    S::Experimental,
                    E::ReferenceOnly,
                    "3-byte length prefix, 16 MiB bound (ledger: framing.length_prefix)",
                ),
                row(
                    C::Registration,
                    S::Unsupported,
                    E::Unknown,
                    "active Android profile lacks a confirmed flow; a newer comparison build only supports offline request construction",
                ),
                row(
                    C::Connect,
                    S::Unsupported,
                    E::Unknown,
                    "Android transport endpoint and login payload not observed (ledger: transport.*, login.*)",
                ),
                row(
                    C::DirectText,
                    S::Unsupported,
                    E::Unknown,
                    "requires G2 proof",
                ),
                row(
                    C::DeviceFanout,
                    S::Unsupported,
                    E::Unknown,
                    "requires G2/G4 evidence",
                ),
                row(
                    C::Receipts,
                    S::Unsupported,
                    E::Unknown,
                    "requires G3 evidence",
                ),
                row(
                    C::Groups,
                    S::Unsupported,
                    E::Unknown,
                    "requires G4 evidence",
                ),
                row(C::Media, S::Unsupported, E::Unknown, "requires G5 evidence"),
                row(
                    C::MessageOperations,
                    S::Unsupported,
                    E::Unknown,
                    "requires G5 evidence",
                ),
                row(
                    C::ProfileAndPresence,
                    S::Unsupported,
                    E::Unknown,
                    "only bootstrap pieces are in scope for v0.1",
                ),
                row(
                    C::Calls,
                    S::Unsupported,
                    E::Unknown,
                    "out of scope; not advertised",
                ),
                row(
                    C::BackupImport,
                    S::Unsupported,
                    E::Unknown,
                    "out of scope; not advertised",
                ),
            ],
        }
    }

    /// Look up the support level of a capability.
    pub fn level(&self, capability: Capability) -> SupportLevel {
        self.entries
            .iter()
            .find(|e| e.capability == capability)
            .map(|e| e.level)
            .unwrap_or(SupportLevel::Unsupported)
    }

    /// Return an `Unsupported` error if the capability is not `Supported` or
    /// `Experimental`.
    pub fn require(&self, capability: Capability) -> crate::Result<()> {
        match self.level(capability) {
            SupportLevel::Supported | SupportLevel::Experimental => Ok(()),
            SupportLevel::Unsupported => {
                let note = self
                    .entries
                    .iter()
                    .find(|e| e.capability == capability)
                    .map(|e| e.note)
                    .unwrap_or("not in capability matrix");
                Err(crate::Error::unsupported(capability, note))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_operations_error() {
        let m = CapabilityMatrix::current("test");
        assert!(m.require(Capability::LocalStore).is_ok());
        assert!(matches!(
            m.require(Capability::Calls),
            Err(crate::Error::Unsupported { .. })
        ));
    }

    #[test]
    fn nothing_protocol_facing_is_supported_yet() {
        let m = CapabilityMatrix::current("test");
        for c in [
            Capability::Registration,
            Capability::Connect,
            Capability::DirectText,
            Capability::Groups,
            Capability::Media,
        ] {
            assert_eq!(m.level(c), SupportLevel::Unsupported, "{c:?}");
        }
    }
}
