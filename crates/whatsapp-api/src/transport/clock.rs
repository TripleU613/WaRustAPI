//! Confirmed APK clock inputs plus historical inferred stanza estimates.
//!
//! APK 37.71 08B.A04/A07, 0iS.A12/A0y and 1Qt.A02 establish
//! login/server-ping clock inputs, elapsed baseline and persisted wall offset.
//! See docs/protocol/token-clock-producers.md. NTP-preferred A00 parity and
//! token refresh remain unported. Historical message/receipt samples remain
//! explicitly inferred fallback only; absent samples never invent an offset.
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

/// Record authenticated incoming time: get/urn:xmpp:ping updates confirmed
/// clock state; message/receipt time updates only the separate inferred sample.
/// Older inferred samples are ignored; positive confirmed corrections are not.
/// Callers must supply authenticated stanzas, not arbitrary captured traffic.
pub fn observe(store: &ProtocolStore, node: &Node) -> Result<bool> {
    // 1Qt.A02 selects namespace/type, not a required ping child.
    if node.description == "iq"
        && node.get_attr("type").and_then(|v| v.as_text()) == Some("get")
        && node.get_attr("xmlns").and_then(|v| v.as_text()) == Some("urn:xmpp:ping")
    {
        let local_ms = Timestamp::now().as_millis();
        let (epoch, elapsed) = context();
        return store.write(|tx| {
            update_confirmed(
                tx,
                node,
                ConfirmedSource::ServerPing,
                local_ms,
                epoch,
                elapsed,
            )
        });
    }
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

const CONFIRMED_KEY: &[u8] = b"server-clock-confirmed-v1";

/// APK-derived input, separate from historical inferred stanza samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmedSource {
    /// APK 37.71 0iS.A12/A0y: authenticated success@t.
    LoginSuccess,
    /// APK 37.71 1Qt.A02: get IQ in urn:xmpp:ping.
    ServerPing,
}
#[derive(Clone, Serialize, Deserialize)]
struct ConfirmedClock {
    source: ConfirmedSource,
    client_server_diff_ms: i64,
    elapsed_base_ms: Option<i64>,
    process_epoch: Option<[u8; 16]>,
}

fn context() -> (Option<[u8; 16]>, Option<i64>) {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        // Android elapsedRealtime includes suspend. CLOCK_BOOTTIME has that
        // property; Rust Instant/CLOCK_MONOTONIC does not. The process nonce
        // deliberately invalidates the elapsed baseline after process restart,
        // matching 08B's in-memory A04 lifecycle without rotating account identity.
        static EPOCH: std::sync::OnceLock<Option<[u8; 16]>> = std::sync::OnceLock::new();
        let epoch = *EPOCH.get_or_init(|| {
            let mut value = [0; 16];
            getrandom::fill(&mut value).ok().map(|()| value)
        });
        let now = rustix::time::clock_gettime(rustix::time::ClockId::Boottime);
        let elapsed = now
            .tv_sec
            .checked_mul(1000)
            .and_then(|v| v.checked_add(now.tv_nsec / 1_000_000));
        (epoch, elapsed)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        (None, None)
    }
}
fn update_confirmed(
    tx: &mut crate::store::WriteTx<'_>,
    node: &Node,
    source: ConfirmedSource,
    local_ms: i64,
    epoch: Option<[u8; 16]>,
    elapsed: Option<i64>,
) -> Result<bool> {
    let Some(seconds) = node
        .get_attr("t")
        .and_then(|v| v.as_text())
        .and_then(super::mex::java_decimal_long)
    else {
        return Ok(false);
    };
    // 0iS.A0p/A0y multiply before 08B.A07's positive check, using Java longs.
    let server_ms = seconds.wrapping_mul(1000);
    if server_ms <= 0 {
        return Ok(false);
    }
    // 0iS.A12 records device seconds; A0y widens that sample back to ms.
    let device_ms = if source == ConfirmedSource::LoginSuccess {
        (local_ms / 1000).wrapping_mul(1000)
    } else {
        local_ms
    };
    let value = ConfirmedClock {
        source,
        client_server_diff_ms: device_ms.wrapping_sub(server_ms),
        elapsed_base_ms: elapsed
            .filter(|_| epoch.is_some())
            .map(|v| server_ms.wrapping_sub(v)),
        process_epoch: epoch,
    };
    // A07 accepts every positive update, including backwards corrections.
    tx.put_json(Table::Checkpoints, CONFIRMED_KEY, 1, &value)?;
    Ok(true)
}
/// Record authenticated login time in the same transaction as login success.
pub(crate) fn observe_login(tx: &mut crate::store::WriteTx<'_>, node: &Node) -> Result<bool> {
    if node.description != "success" {
        return Ok(false);
    }
    let local_ms = Timestamp::now().as_millis();
    let (epoch, elapsed) = context();
    update_confirmed(
        tx,
        node,
        ConfirmedSource::LoginSuccess,
        local_ms,
        epoch,
        elapsed,
    )
}
fn resolve(
    value: &ConfirmedClock,
    local_ms: i64,
    epoch: Option<[u8; 16]>,
    elapsed: Option<i64>,
) -> (i64, &'static str) {
    if value.process_epoch.is_some()
        && value.process_epoch == epoch
        && let (Some(base), Some(now)) = (value.elapsed_base_ms.filter(|v| *v != 0), elapsed)
    {
        return (base.wrapping_add(now), "elapsed_realtime");
    }
    (
        local_ms.wrapping_sub(value.client_server_diff_ms),
        "saved_offset",
    )
}
/// Receipt parser clock: confirmed elapsed/offset, otherwise local wall time.
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
pub(super) fn receipt_now(tx: &ReadTx<'_>, local_ms: i64) -> Result<i64> {
    // shortcut: native NTP preference is unavailable, upgrade when NTP acquisition is ported.
    let value = tx.get_json::<ConfirmedClock>(Table::Checkpoints, CONFIRMED_KEY, 1)?;
    let (epoch, elapsed) = context();
    Ok(value.map_or(local_ms, |value| {
        resolve(&value, local_ms, epoch, elapsed).0
    }))
}

/// Resolve server time from confirmed input, falling back to historical inference.
pub fn server_now(tx: &ReadTx<'_>, local_ms: i64) -> Result<Option<i64>> {
    if let Some(value) = tx.get_json::<ConfirmedClock>(Table::Checkpoints, CONFIRMED_KEY, 1)? {
        let (epoch, elapsed) = context();
        return Ok(Some(resolve(&value, local_ms, epoch, elapsed).0));
    }
    Ok(sample(tx)?.map(|v| v.server_now(local_ms)))
}
/// Content-free clock provenance; contains no raw stanza, account or epoch ID.
#[derive(Debug, Serialize)]
pub struct ClockInspection {
    /// Saved confirmed source, or null for no confirmed sample.
    pub confirmed_source: Option<ConfirmedSource>,
    /// elapsed_realtime, saved_offset, inferred_stanza or unavailable.
    pub resolution: String,
    /// Whether a historical inferred sample exists, without exposing timestamps.
    pub inferred_sample_present: bool,
}
/// Inspect saved clock provenance offline without adopting captures or sampling a server.
pub fn inspect(store: &ProtocolStore) -> Result<ClockInspection> {
    store.read(|tx| {
        let value = tx.get_json::<ConfirmedClock>(Table::Checkpoints, CONFIRMED_KEY, 1)?;
        let inferred = sample(tx)?.is_some();
        let (epoch, elapsed) = context();
        Ok(ClockInspection {
            confirmed_source: value.as_ref().map(|v| v.source),
            resolution: value
                .as_ref()
                .map(|v| resolve(v, Timestamp::now().as_millis(), epoch, elapsed).1)
                .unwrap_or(if inferred {
                    "inferred_stanza"
                } else {
                    "unavailable"
                })
                .into(),
            inferred_sample_present: inferred,
        })
    })
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
    fn receipt_clock_ignores_inferred_stanzas_and_uses_confirmed_offset() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir);
        let local_ms = 30_123;
        assert_eq!(
            store.read(|tx| receipt_now(tx, local_ms)).unwrap(),
            local_ms
        );
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    KEY,
                    1,
                    &ClockSample {
                        server_ms: 9_000_000,
                        local_ms,
                    },
                )
            })
            .unwrap();
        assert_eq!(
            store.read(|tx| receipt_now(tx, local_ms)).unwrap(),
            local_ms
        );
        store
            .write(|tx| {
                update_confirmed(
                    tx,
                    &Node::new("success").attr("t", "20"),
                    ConfirmedSource::LoginSuccess,
                    local_ms,
                    None,
                    None,
                )
            })
            .unwrap();
        let head = store.journal_head().unwrap();
        assert_eq!(store.read(|tx| receipt_now(tx, 31_234)).unwrap(), 21_234);
        assert_eq!(store.journal_head().unwrap(), head);
        drop(store);
        let store = open(&dir);
        assert_eq!(store.read(|tx| receipt_now(tx, 31_234)).unwrap(), 21_234);
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn confirmed_inputs_preserve_sources_and_ignore_non_clock_iqs() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir);
        assert_eq!(inspect(&store).unwrap().resolution, "unavailable");
        let ping = || {
            Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping")
        };
        for node in [
            ping(),
            ping().attr("t", "bad"),
            ping().attr("t", "-1"),
            ping().attr("t", "9223372036854775807"),
            ping().attr("type", "result").attr("t", "1700000000"),
            ping().attr("xmlns", "w:p").attr("t", "1700000000"),
        ] {
            assert!(!observe(&store, &node).unwrap());
        }
        assert!(inspect(&store).unwrap().confirmed_source.is_none());
        assert!(observe(&store, &ping().attr("t", "1700000000")).unwrap());
        assert_eq!(
            inspect(&store).unwrap().confirmed_source,
            Some(ConfirmedSource::ServerPing)
        );
        // Stanza inference can remain saved, but must not override confirmed time.
        assert!(observe(&store, &Node::new("message").attr("t", "1900000000")).unwrap());
        let resolved = store
            .read(|tx| server_now(tx, Timestamp::now().as_millis()))
            .unwrap()
            .unwrap();
        assert!((resolved - 1_700_000_000_000).abs() < 5000);
        assert!(
            store
                .write(|tx| observe_login(tx, &Node::new("success").attr("t", "1600000000")))
                .unwrap()
        );
        assert_eq!(
            inspect(&store).unwrap().confirmed_source,
            Some(ConfirmedSource::LoginSuccess)
        );
        // A07 accepts backwards correction; the earlier newer inference stays separate.
        let resolved = store
            .read(|tx| server_now(tx, Timestamp::now().as_millis()))
            .unwrap()
            .unwrap();
        assert!((resolved - 1_600_000_000_000).abs() < 5000);
        assert!(inspect(&store).unwrap().inferred_sample_present);
        let report = serde_json::to_string(&inspect(&store).unwrap()).unwrap();
        for secret in [
            "process_epoch",
            "elapsed_base_ms",
            "1600000000",
            "1900000000",
        ] {
            assert!(!report.contains(secret));
        }
        // Login and clock commit together; failed transaction retains old source.
        let result: Result<()> = store.write(|tx| {
            update_confirmed(
                tx,
                &ping().attr("t", "1700000000"),
                ConfirmedSource::ServerPing,
                20_123,
                Some([1; 16]),
                Some(500),
            )?;
            Err(crate::Error::invalid("fixture", "rollback"))
        });
        assert!(result.is_err());
        assert_eq!(
            inspect(&store).unwrap().confirmed_source,
            Some(ConfirmedSource::LoginSuccess)
        );
        drop(store);
        let reopened = open(&dir);
        assert_eq!(
            inspect(&reopened).unwrap().confirmed_source,
            Some(ConfirmedSource::LoginSuccess)
        );
    }

    #[test]
    fn apk_clock_resolution_handles_wall_jumps_restart_and_whole_second_login_sample() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir);
        let node = Node::new("success").attr("t", "10");
        store
            .write(|tx| {
                update_confirmed(
                    tx,
                    &node,
                    ConfirmedSource::LoginSuccess,
                    20_123,
                    Some([1; 16]),
                    Some(500),
                )
            })
            .unwrap();
        let value: ConfirmedClock = store
            .read(|tx| tx.get_json(Table::Checkpoints, CONFIRMED_KEY, 1))
            .unwrap()
            .unwrap();
        assert_eq!(value.client_server_diff_ms, 10_000);
        assert_eq!(value.elapsed_base_ms, Some(9500));
        assert_eq!(
            resolve(&value, 999_999, Some([1; 16]), Some(600)),
            (10_100, "elapsed_realtime")
        );
        // New process/same host uptime does not reuse old in-memory baseline.
        assert_eq!(
            resolve(&value, 21_123, Some([2; 16]), Some(600)),
            (11_123, "saved_offset")
        );
        assert_eq!(
            resolve(&value, 21_123, None, None),
            (11_123, "saved_offset")
        );
        store
            .write(|tx| {
                update_confirmed(tx, &node, ConfirmedSource::ServerPing, 20_123, None, None)
            })
            .unwrap();
        let ping: ConfirmedClock = store
            .read(|tx| tx.get_json(Table::Checkpoints, CONFIRMED_KEY, 1))
            .unwrap()
            .unwrap();
        assert_eq!(ping.client_server_diff_ms, 10_123);
        assert_eq!(resolve(&ping, 21_123, None, None), (11_000, "saved_offset"));
        let zero = ConfirmedClock {
            elapsed_base_ms: Some(0),
            ..value
        };
        assert_eq!(
            resolve(&zero, 21_123, Some([1; 16]), Some(600)),
            (11_123, "saved_offset")
        );
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
