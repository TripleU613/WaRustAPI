//! Server clock sample for converting server-domain sync timers locally.
//!
//! Comparison `08B.A00` resolves "server now" from elapsed-realtime plus an
//! available server offset with saved-offset/local fallbacks (traced in
//! docs/protocol/cross-device-sync.md). The offset **source** was not traced;
//! this module's choice to sample the `t` attribute of authenticated incoming
//! `message`/`receipt` stanzas is a documented local inference. Samples are
//! never invented: without one, resolution reports no estimate.
use crate::{
    Result,
    binary::Node,
    model::time::Timestamp,
    store::{ProtocolStore, ReadTx, Table},
};
use serde::{Deserialize, Serialize};

const KEY: &[u8] = b"server-clock-sample";

/// One contemporaneous (server, local) millisecond pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockSample {
    /// Server time from the stanza `t` attribute, in milliseconds.
    pub server_ms: i64,
    /// Local wall clock when the stanza was received, in milliseconds.
    pub local_ms: i64,
}
impl ClockSample {
    /// Estimated server time for a later local instant, saturating.
    pub fn server_now(&self, local_now_ms: i64) -> i64 {
        self.server_ms
            .saturating_add(local_now_ms.saturating_sub(self.local_ms))
    }
}

/// Record the newest sample from an authenticated incoming stanza. Only
/// `message` and `receipt` stanzas with a plausible whole-second `t` count;
/// samples older than the saved one (server domain) are ignored.
pub fn observe(store: &ProtocolStore, node: &Node) -> Result<bool> {
    if !matches!(node.description.as_str(), "message" | "receipt") {
        return Ok(false);
    }
    let Some(seconds) = node
        .get_attr("t")
        .and_then(|v| v.as_text())
        .and_then(|v| v.parse::<i64>().ok())
        // Reject pre-2010 or millisecond-looking values; the wire uses seconds.
        .filter(|v| (1_262_304_000..=100_000_000_000).contains(v))
    else {
        return Ok(false);
    };
    let sample = ClockSample {
        server_ms: seconds.saturating_mul(1000),
        local_ms: Timestamp::now().as_millis(),
    };
    store.write(|tx| {
        let previous: Option<ClockSample> = tx.get_json(Table::Checkpoints, KEY, 1)?;
        if previous.is_some_and(|p| p.server_ms > sample.server_ms) {
            return Ok(false);
        }
        tx.put_json(Table::Checkpoints, KEY, 1, &sample)?;
        Ok(true)
    })
}

/// The saved sample, if any authenticated stanza has provided one.
pub fn sample(tx: &ReadTx<'_>) -> Result<Option<ClockSample>> {
    tx.get_json(Table::Checkpoints, KEY, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(dir: &tempfile::TempDir) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: dir.path().join("clock.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap()
    }
    #[test]
    fn samples_come_only_from_plausible_message_or_receipt_times_and_never_regress() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir);
        assert!(store.read(sample).unwrap().is_none());
        for node in [
            Node::new("iq").attr("t", "1700000000"),
            Node::new("message"),
            Node::new("message").attr("t", "abc"),
            Node::new("message").attr("t", "1700000000123"),
            Node::new("message").attr("t", "12"),
            Node::new("message").attr("t", "-5"),
        ] {
            assert!(!observe(&store, &node).unwrap(), "{node:?}");
        }
        assert!(store.read(sample).unwrap().is_none());
        assert!(observe(&store, &Node::new("message").attr("t", "1700000100")).unwrap());
        let first = store.read(sample).unwrap().unwrap();
        assert_eq!(first.server_ms, 1_700_000_100_000);
        assert!((first.local_ms - Timestamp::now().as_millis()).abs() < 5_000);
        // Older server time never replaces a newer sample; newer receipts do.
        assert!(!observe(&store, &Node::new("receipt").attr("t", "1700000050")).unwrap());
        assert_eq!(store.read(sample).unwrap().unwrap(), first);
        assert!(observe(&store, &Node::new("receipt").attr("t", "1700000200")).unwrap());
        assert_eq!(
            store.read(sample).unwrap().unwrap().server_ms,
            1_700_000_200_000
        );
        let s = ClockSample {
            server_ms: 10_000,
            local_ms: 5_000,
        };
        assert_eq!(s.server_now(6_500), 11_500);
        assert_eq!(s.server_now(i64::MAX), i64::MAX);
    }
}
