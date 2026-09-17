//! Timestamps. Server timestamps and local observation times are kept apart
//! by the types that carry them; this module only provides the scalar.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Milliseconds since the Unix epoch.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct Timestamp(pub i64);

impl Timestamp {
    /// Current wall-clock time. Callers that need determinism inject a clock
    /// instead of calling this.
    pub fn now() -> Self {
        let d = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        Self(i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
    }

    /// Construct from whole seconds (the protocol commonly uses seconds).
    pub const fn from_unix_seconds(secs: i64) -> Self {
        Self(secs.saturating_mul(1000))
    }

    /// Milliseconds since the epoch.
    pub const fn as_millis(self) -> i64 {
        self.0
    }

    /// Whole seconds since the epoch.
    pub const fn as_unix_seconds(self) -> i64 {
        self.0.div_euclid(1000)
    }

    /// Saturating addition of a duration.
    pub fn saturating_add(self, d: Duration) -> Self {
        Self(self.0.saturating_add(i64::try_from(d.as_millis()).unwrap_or(i64::MAX)))
    }
}
