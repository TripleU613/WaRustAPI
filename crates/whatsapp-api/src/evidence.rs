//! Evidence labels attached to protocol constants and profile decisions.
//!
//! The plan (README section 6.1) requires every important wire constant to
//! point at evidence. These labels are carried in the protocol profile and in
//! the capability matrix so that a consumer can distinguish tested behavior
//! from reference-derived hypotheses.

use serde::{Deserialize, Serialize};

/// Provenance class of a protocol fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EvidenceLabel {
    /// Observed against the pinned Android reference client or a live server,
    /// with a dated record in `docs/protocol/`.
    Observed,
    /// Derived from observed facts by a documented argument (for example a
    /// length computed from an observed encoding rule).
    Derived,
    /// Taken from a third-party implementation of a *different* profile
    /// (typically the WhatsApp Web multi-device protocol). Valid as a starting
    /// hypothesis only.
    ReferenceOnly,
    /// No evidence. Code paths depending on an `Unknown` fact must fail with a
    /// typed error rather than guess.
    Unknown,
}

impl EvidenceLabel {
    /// Whether the fact may be used by a production code path.
    ///
    /// Only `Observed` and `Derived` facts qualify. `ReferenceOnly` facts are
    /// allowed behind explicitly experimental features.
    pub const fn is_production_grade(self) -> bool {
        matches!(self, Self::Observed | Self::Derived)
    }
}

/// A value together with its evidence label and a pointer to the record that
/// justifies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidenced<T> {
    /// The value itself.
    pub value: T,
    /// Its provenance class.
    pub label: EvidenceLabel,
    /// A short reference into `docs/protocol/evidence-ledger.md` (for example
    /// `"framing.length_prefix"`).
    pub ledger_ref: String,
}

impl<T> Evidenced<T> {
    /// Construct an evidenced value.
    pub fn new(value: T, label: EvidenceLabel, ledger_ref: impl Into<String>) -> Self {
        Self {
            value,
            label,
            ledger_ref: ledger_ref.into(),
        }
    }

    /// Construct a `ReferenceOnly` value.
    pub fn reference_only(value: T, ledger_ref: impl Into<String>) -> Self {
        Self::new(value, EvidenceLabel::ReferenceOnly, ledger_ref)
    }
}
