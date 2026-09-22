//! Crash-matrix scenarios from docs/design/README.md 19.2 that are provable today, using the
//! public API only. Rows that need a transmission path or a Signal backend
//! are listed at the bottom as not yet coverable.
//!
//! Failure injection works by returning an error from inside a store
//! transaction closure, plus subprocess exits without running store destructors.
//! With comparison-client, real consumption/receipt APIs are also exercised
//! across abrupt process exit using encrypted synthetic media, without networking.
//! These check rollback/recovery, not disk failure or power-loss durability.

use std::sync::Arc;

use whatsapp_api::Error;
use whatsapp_api::model::ids::{AccountId, ChatId, ClientRequestId, EventCursor};
use whatsapp_api::model::jid::Jid;
use whatsapp_api::model::message::MessageContent;
use whatsapp_api::model::send::{AttemptProgress, SendRequest};
use whatsapp_api::outbox::Outbox;
use whatsapp_api::profile::Limits;
use whatsapp_api::store::{
    ConsumerId, MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table,
};
use whatsapp_api::sync::SyncEvent;

// Fork/exec can briefly inherit other tests' lock descriptors until CLOEXEC.
// Isolate this binary's subprocess launches from its immediate-reopen checks.
static CRASH_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn open(dir: &tempfile::TempDir) -> Arc<ProtocolStore> {
    Arc::new(
        ProtocolStore::open(OpenOptions {
            path: dir.path().join("protocol.redb"),
            account_id: AccountId::new("crash").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
            create: true,
        })
        .unwrap(),
    )
}

fn req(id: &str) -> SendRequest {
    SendRequest {
        client_request_id: ClientRequestId::new(id).unwrap(),
        chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net").unwrap()).unwrap(),
        content: MessageContent::Text {
            body: "hello".into(),
            quoted: None,
            style: Default::default(),
        },
    }
}

/// Child harness: the normal test run is a no-op. The parent supplies only a
/// fresh temporary test directory, never an operator account database.
#[test]
fn subprocess_crash_fixture() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    let Some(path) = std::env::var_os("TELEWHATSAPP_CRASH_FIXTURE_PATH") else {
        return;
    };
    let mode = std::env::var("TELEWHATSAPP_CRASH_FIXTURE_MODE").unwrap();
    let store = ProtocolStore::open(OpenOptions {
        path: path.into(),
        account_id: AccountId::new("crash").unwrap(),
        key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
        create: true,
    })
    .unwrap();
    #[cfg(feature = "comparison-client")]
    if mode.starts_with("view-once-") {
        view_once_crash::child(&store, &mode);
    }
    let outbox = Outbox::new("test", Limits::default());
    store
        .write(|tx| {
            outbox.enqueue(tx, &req("subprocess-intent"))?;
            tx.put_json(
                Table::Checkpoints,
                b"crash-reservation",
                1,
                &serde_json::json!({"status":"outcome_unknown"}),
            )?;
            if mode == "before-commit" {
                std::process::exit(74);
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(mode, "after-commit");
    std::process::exit(75);
}

#[cfg(feature = "comparison-client")]
mod view_once_crash {
    use super::*;
    use prost::Message;
    use sha2::{Digest, Sha256};
    use whatsapp_api::{
        binary::{Encoder, Node, TokenDictionary},
        transport::{image_download, receipts},
    };

    const INBOX: &[u8] = b"received:crash:view-once";

    // Minimal APK ImageMessage fields: hash=4, length=5, key=8,
    // encrypted hash=9 and view-once=25, inside Message field 3.
    #[derive(Message)]
    struct ImageFixture {
        #[prost(bytes = "vec", tag = "4")]
        hash: Vec<u8>,
        #[prost(uint64, tag = "5")]
        length: u64,
        #[prost(bytes = "vec", tag = "8")]
        key: Vec<u8>,
        #[prost(bytes = "vec", tag = "9")]
        encrypted_hash: Vec<u8>,
        #[prost(bool, tag = "25")]
        once: bool,
    }

    #[derive(Message)]
    struct EnvelopeFixture {
        #[prost(message, optional, tag = "3")]
        image: Option<ImageFixture>,
    }

    fn download_key() -> String {
        format!("view-once-download:{}", hex::encode(Sha256::digest(INBOX)))
    }

    fn seed(store: &ProtocolStore) {
        let encrypted = whatsapp_api::signal::image::encrypt(b"crash fixture image").unwrap();
        let protobuf = EnvelopeFixture {
            image: Some(ImageFixture {
                hash: encrypted.plaintext_hash.to_vec(),
                length: encrypted.plaintext_len,
                key: encrypted.media_key.to_vec(),
                encrypted_hash: encrypted.encrypted_hash.to_vec(),
                once: true,
            }),
        }
        .encode_to_vec();
        let route = Encoder::new(TokenDictionary::comparison_2_26_37_70())
            .encode(
                &Node::new("message")
                    .attr("id", "crash-picture")
                    .attr("from", Jid::parse("15550000000@s.whatsapp.net").unwrap()),
            )
            .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    INBOX,
                    1,
                    &serde_json::json!({"status":"decrypted", "protobuf":protobuf, "route":route}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    download_key().as_bytes(),
                    1,
                    &serde_json::json!({"status":"downloaded_unopened", "body":encrypted.body,
                    "encrypted_bytes":encrypted.body.len()}),
                )
            })
            .unwrap();
    }

    pub(super) fn child(store: &ProtocolStore, mode: &str) -> ! {
        let (ticket, plaintext) = image_download::consume_view_once_with_ticket(store).unwrap();
        assert_eq!(&*plaintext, b"crash fixture image");
        if mode == "view-once-consumed" {
            // Exit before viewer success: no opened receipt may be invented.
            std::process::exit(76);
        }
        // The fixture explicitly models a successful viewer before queueing.
        receipts::queue_opened(store, &ticket).unwrap();
        if mode == "view-once-queued" {
            std::process::exit(77);
        }
        assert_eq!(mode, "view-once-reserved");
        assert_eq!(receipts::reserve_pending(store).unwrap().len(), 1);
        // A socket outcome would now be ambiguous. Never replay after restart.
        std::process::exit(78);
    }

    #[test]
    fn abrupt_exit_cannot_reopen_or_replay_uncertain_opened_receipt() {
        let _guard = CRASH_TEST_LOCK.lock().unwrap();
        for (mode, code, expected) in [
            ("view-once-consumed", 76, "awaiting_viewer"),
            ("view-once-queued", 77, "queued"),
            ("view-once-reserved", 78, "queued"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir);
            seed(&store);
            drop(store);
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "subprocess_crash_fixture", "--nocapture"])
                .env(
                    "TELEWHATSAPP_CRASH_FIXTURE_PATH",
                    dir.path().join("protocol.redb"),
                )
                .env("TELEWHATSAPP_CRASH_FIXTURE_MODE", mode)
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(code),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let store = open(&dir);
            assert!(image_download::consume_view_once_with_ticket(&store).is_err());
            for key in [INBOX, download_key().as_bytes()] {
                let saved: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, key, 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(saved["status"], "consumed");
                assert!(saved.get("body").is_none());
                assert!(saved.get("protobuf").is_none());
            }
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, INBOX, 1))
                .unwrap()
                .unwrap();
            assert_eq!(saved["opened_receipt"], expected);
            let receipt_key = format!(
                "receipt-outbox:{}:played",
                hex::encode(Sha256::digest(INBOX))
            );
            let receipt: Option<serde_json::Value> = store
                .read(|tx| tx.get_json(Table::Checkpoints, receipt_key.as_bytes(), 1))
                .unwrap();
            match mode {
                "view-once-consumed" => assert!(receipt.is_none()),
                "view-once-queued" => assert_eq!(receipt.unwrap()["status"], "pending"),
                _ => assert_eq!(receipt.unwrap()["status"], "outcome_unknown"),
            }
            let pending = receipts::reserve_pending(&store).unwrap();
            assert_eq!(pending.len(), usize::from(mode == "view-once-queued"));
            assert!(receipts::reserve_pending(&store).unwrap().is_empty());
            if mode != "view-once-consumed" {
                let ack = Node::new("ack")
                    .attr("class", "receipt")
                    .attr("type", "played")
                    .attr("id", "crash-picture")
                    .attr("from", "15550000000@s.whatsapp.net");
                assert!(receipts::record_ack(&store, &ack).unwrap());
                drop(store);
                let store = open(&dir);
                let settled: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, receipt_key.as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(settled["status"], "server_acknowledged");
                assert!(receipts::reserve_pending(&store).unwrap().is_empty());
                assert!(image_download::consume_view_once_with_ticket(&store).is_err());
            }
        }
    }
}

#[test]
fn abrupt_process_exit_preserves_atomic_intent_and_reservation() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    for (mode, code, committed) in [("before-commit", 74, false), ("after-commit", 75, true)] {
        let dir = tempfile::tempdir().unwrap();
        // Initialize the store so this test targets record commit, not creation.
        drop(open(&dir));
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "subprocess_crash_fixture", "--nocapture"])
            .env(
                "TELEWHATSAPP_CRASH_FIXTURE_PATH",
                dir.path().join("protocol.redb"),
            )
            .env("TELEWHATSAPP_CRASH_FIXTURE_MODE", mode)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(code),
            "subprocess failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let store = open(&dir);
        let outbox = Outbox::new("test", Limits::default());
        let pending = store.read(|tx| outbox.pending(tx, 10)).unwrap();
        assert_eq!(pending.len(), usize::from(committed));
        let reservation: Option<serde_json::Value> = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"crash-reservation", 1))
            .unwrap();
        assert_eq!(reservation.is_some(), committed);
        if committed {
            assert_eq!(reservation.unwrap()["status"], "outcome_unknown");
            let (ticket, new) = store
                .write(|tx| outbox.enqueue(tx, &req("subprocess-intent")))
                .unwrap();
            assert!(!new);
            assert_eq!(ticket.local_id, pending[0].local_id);
        } else {
            assert_eq!(store.journal_head().unwrap(), EventCursor::START);
            assert_eq!(store.read(|tx| tx.count(Table::Idempotency)).unwrap(), 0);
        }
    }
}

/// Row 1: crash before intent commit → no ticket exists; the same client
/// request can enqueue normally afterwards.
#[test]
fn before_intent_commit() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let s = open(&dir);
    let ob = Outbox::new("t", Limits::default());
    let r: Result<(), Error> = s.write(|tx| {
        ob.enqueue(tx, &req("r1"))?;
        Err(Error::Cancelled) // crash after staging, before commit
    });
    assert!(r.is_err());
    assert_eq!(s.read(|tx| tx.count(Table::Outbox)).unwrap(), 0);
    assert_eq!(s.read(|tx| tx.count(Table::Idempotency)).unwrap(), 0);
    assert_eq!(s.journal_head().unwrap(), EventCursor::START);
    let (ticket, new) = s.write(|tx| ob.enqueue(tx, &req("r1"))).unwrap();
    assert!(new);
    assert_eq!(ticket.client_request_id.as_str(), "r1");
}

/// Row 2: crash after intent commit, before encryption → intent remains
/// queued and is visible after reopen; no crypto state was touched.
#[test]
fn after_intent_commit_before_encryption() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let ticket = {
        let s = open(&dir);
        let ob = Outbox::new("t", Limits::default());
        let (t, _) = s.write(|tx| ob.enqueue(tx, &req("r1"))).unwrap();
        t
        // process "crashes" here: store dropped without further work
    };
    let s = open(&dir);
    let ob = Outbox::new("t", Limits::default());
    let st = s
        .read(|tx| ob.status(tx, ticket.local_id))
        .unwrap()
        .unwrap();
    assert_eq!(st.progress, AttemptProgress::Queued);
    assert_eq!(s.read(|tx| tx.count(Table::SignalSessions)).unwrap(), 0);
    let pending = s.read(|tx| ob.pending(tx, 10)).unwrap();
    assert_eq!(pending.len(), 1);
}

/// Row 3: crash after crypto calculation, before atomic commit → derived
/// output is discarded (overlay never committed) and nothing exists to
/// transmit.
#[test]
fn after_crypto_before_commit_discards_overlay() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    use bytes::Bytes;
    use whatsapp_api::model::ids::DeviceAddress;
    use whatsapp_api::signal::{Overlay, RecordKey};

    let dir = tempfile::tempdir().unwrap();
    let s = open(&dir);
    let dev =
        DeviceAddress::from_jid(&Jid::parse("15550000000:1@s.whatsapp.net").unwrap()).unwrap();
    let r: Result<(), Error> = s.write(|tx| {
        let mut ov = Overlay::default();
        ov.load(tx, RecordKey::Session(dev.clone()))?;
        ov.put(
            RecordKey::Session(dev.clone()),
            Bytes::from_static(b"advanced ratchet"),
        );
        ov.commit(tx)?; // staged in the transaction
        Err(Error::Cancelled) // crash before the transaction commits
    });
    assert!(r.is_err());
    assert_eq!(s.read(|tx| tx.count(Table::SignalSessions)).unwrap(), 0);
}

/// Row 9: consumer side effect happened, crash before cursor ACK → the same
/// event replays with the same `EventId`, so a deduplicating consumer
/// produces no duplicate effect.
#[test]
fn consumer_side_effect_before_ack_replays_same_event_id() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let s = open(&dir);
    let consumer = ConsumerId::new("bridge").unwrap();
    let ob = Outbox::new("t", Limits::default());
    s.write(|tx| ob.enqueue(tx, &req("r1"))).unwrap();
    let first = s.read_events(&consumer, EventCursor::START, 10).unwrap();
    assert_eq!(first.events.len(), 1);
    // side effect performed here; crash before ack
    drop(s);
    let s = open(&dir);
    let again = s.read_events(&consumer, EventCursor::START, 10).unwrap();
    assert_eq!(again.events[0].event_id, first.events[0].event_id);
    assert!(matches!(
        again.events[0].event,
        SyncEvent::SendStatusChanged(_)
    ));
    s.ack_events(&consumer, again.ack_cursor).unwrap();
    assert!(
        s.read_events(&consumer, again.ack_cursor, 10)
            .unwrap()
            .events
            .is_empty()
    );
}

/// Row 10 (partial): crash during key rotation → either the old or the new
/// key opens the store, never neither. Rotation is a single durable
/// transaction, so we check both halves of the invariant.
#[test]
fn key_rotation_is_atomic() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("protocol.redb");
    let acct = AccountId::new("crash").unwrap();
    let old = MasterKey::from_bytes([9u8; 32]);
    let new = MasterKey::from_bytes([10u8; 32]);
    {
        let s = open(&dir);
        s.write(|tx| tx.put(Table::Account, b"k", 1, b"v")).unwrap();
        s.rotate_master_key(&StaticKeySource::new(new.clone()))
            .unwrap();
    }
    let opens = |key: MasterKey| {
        ProtocolStore::open(OpenOptions {
            path: path.clone(),
            account_id: acct.clone(),
            key_source: Arc::new(StaticKeySource::new(key)),
            create: false,
        })
    };
    let with_new = opens(new).map(|s| s.read(|tx| tx.get(Table::Account, b"k", 1)).unwrap());
    assert_eq!(with_new.unwrap().unwrap(), b"v");
    assert!(matches!(opens(old), Err(Error::WrongStorageKey)));
}

/// Second-process ownership: a second opener on the same database is
/// rejected while the first owner lives, and admitted after it exits.
#[test]
fn second_owner_rejected_then_admitted() {
    let _guard = CRASH_TEST_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let s = open(&dir);
    let attempt = ProtocolStore::open(OpenOptions {
        path: dir.path().join("protocol.redb"),
        account_id: AccountId::new("crash").unwrap(),
        key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
        create: false,
    });
    assert!(matches!(
        attempt,
        Err(Error::Storage(
            whatsapp_api::store::StoreError::AlreadyOwned { .. }
        ))
    ));
    drop(s);
    assert!(open(&dir).journal_head().is_ok());
}

// Comparison-feature unit tests additionally cover durable query/send
// reservations, uncertain sends across reopen, real inbound ratchet rollback,
// duplicate inbound processing after reopen, and one-use receipt reservations.
// The integration tests above add abrupt process exits for view-once consumption
// and receipt queue/reservation boundaries. They do not simulate socket writes.
// ACK-commit I/O failure,
// disk-full behavior and exhaustive socket cut points remain uncovered.
