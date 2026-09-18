//! Crash-matrix scenarios from docs/design/README.md 19.2 that are provable today, using the
//! public API only. Rows that need a transmission path or a Signal backend
//! are listed at the bottom as not yet coverable.
//!
//! Failure injection works by returning an error from inside a store
//! transaction closure: `ProtocolStore::write` commits nothing unless the
//! closure returns `Ok`, which is the same guarantee a process crash gives.

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
        },
    }
}

/// Row 1: crash before intent commit → no ticket exists; the same client
/// request can enqueue normally afterwards.
#[test]
fn before_intent_commit() {
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

// Not yet coverable (need a transmission path or a Signal backend):
// row 4 (after crypto/attempt commit, before socket write), row 5 (during/after
// socket write, before ACK), row 6 (after ACK arrival, before its commit),
// row 7 (after inbound decrypt, before commit), row 8 (after inbound commit,
// before receipt), row 11 (disk-full/commit I/O failure stops advancement).
