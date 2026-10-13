//! Apply supported authenticated pending actions, never arbitrary wire payloads.
use super::*;
use crate::{
    model::ids::ChatId,
    model::jid::Jid,
    store::{ProtocolStore, ReadTx},
};
use prost::Message;
use sha2::{Digest, Sha256};
mod mute;
// Only the comparison-client resolution test uses it.
#[cfg(all(test, feature = "comparison-client"))]
pub(crate) use mute::save_for_test as save_mute_for_test;
pub use mute::{ChatMute, MuteDeadline, ResolvedChatMute, chat_mute};

enum Projection {
    Pin(String, ChatPin),
    Mute(String, ChatMute),
}

fn decode_action(action: &PendingAction) -> Result<Projection> {
    let index: Vec<String> =
        serde_json::from_slice(&action.index).map_err(|_| invalid("invalid action index"))?;
    match index.first().map(String::as_str) {
        Some("pin_v1") => decode_pin(action).map(|(chat, pin)| Projection::Pin(chat, pin)),
        Some("mute") => mute::decode(action).map(|(chat, mute)| Projection::Mute(chat, mute)),
        _ => Err(invalid("unsupported action index; batch remains pending")),
    }
}

#[derive(Message)]
struct ActionValue {
    #[prost(int64, optional, tag = "1")]
    timestamp: Option<i64>,
    #[prost(message, optional, tag = "5")]
    pin: Option<Pin>,
}
#[derive(Message)]
struct Pin {
    #[prost(bool, optional, tag = "1")]
    pinned: Option<bool>,
}

/// Local projection of an authenticated pin action, not a receipt or remote write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatPin {
    /// Whether the chat is pinned.
    pub pinned: bool,
    /// Original action timestamp; units are preserved without conversion.
    pub timestamp: i64,
}

fn pin_key(chat: &str) -> Vec<u8> {
    format!(
        "app-state-chat-pin:{}",
        hex::encode(Sha256::digest(chat.as_bytes()))
    )
    .into_bytes()
}
fn applied_key(collection: &str) -> Vec<u8> {
    format!("app-state-applied:{}", hex::encode(collection)).into_bytes()
}

/// Read locally projected pin state without exposing action payloads or keys.
pub fn chat_pin(tx: &ReadTx<'_>, chat: &Jid) -> Result<Option<ChatPin>> {
    ChatId::from_jid(chat)?;
    tx.get_json(Table::Checkpoints, &pin_key(&chat.to_string()), 1)
}

// Require only the supported fields, rather than letting prost discard another
// action in the same envelope and then incorrectly reporting full application.
fn only_fields(mut bytes: &[u8], allowed: &[u32]) -> Result<()> {
    use prost::encoding::{DecodeContext, decode_key, skip_field};
    let mut seen = BTreeSet::new();
    while !bytes.is_empty() {
        let (tag, wire) = decode_key(&mut bytes).map_err(|_| invalid("invalid action protobuf"))?;
        if !allowed.contains(&tag) || !seen.insert(tag) {
            return Err(invalid(
                "unsupported or ambiguous action field; batch remains pending",
            ));
        }
        skip_field(wire, tag, &mut bytes, DecodeContext::default())
            .map_err(|_| invalid("invalid action protobuf"))?;
    }
    Ok(())
}

fn decode_pin(action: &PendingAction) -> Result<(String, ChatPin)> {
    if action.operation != 1 || action.schema_version != 5 {
        return Err(invalid(
            "unsupported action operation or schema; batch remains pending",
        ));
    }
    let index: Vec<String> =
        serde_json::from_slice(&action.index).map_err(|_| invalid("invalid action index"))?;
    if index.len() != 2 || index[0] != "pin_v1" {
        return Err(invalid("unsupported action index; batch remains pending"));
    }
    let jid = Jid::parse(&index[1]).map_err(|_| invalid("invalid action chat"))?;
    ChatId::from_jid(&jid).map_err(|_| invalid("unsupported action chat"))?;
    let value = action
        .value
        .as_deref()
        .ok_or_else(|| invalid("missing pin action"))?;
    only_fields(value, &[1, 5])?;
    let decoded = ActionValue::decode(value).map_err(|_| invalid("invalid pin action"))?;
    // Check the raw nested bytes too, including unknown and repeated fields.
    #[derive(Message)]
    struct Raw {
        #[prost(bytes = "vec", optional, tag = "5")]
        pin: Option<Vec<u8>>,
    }
    let raw = Raw::decode(value).map_err(|_| invalid("invalid pin action"))?;
    only_fields(
        raw.pin
            .as_deref()
            .ok_or_else(|| invalid("missing pin action"))?,
        &[1],
    )?;
    Ok((
        jid.to_string(),
        ChatPin {
            pinned: decoded
                .pin
                .and_then(|p| p.pinned)
                .ok_or_else(|| invalid("missing pinned boolean"))?,
            timestamp: decoded
                .timestamp
                .ok_or_else(|| invalid("missing action timestamp"))?,
        },
    ))
}

/// Apply one previously authenticated pending batch atomically, preserving wire
/// order and a separate applied-version cursor. Unsupported batches fail without
/// changing projections, consuming actions, or advancing the cursor. Callers must
/// have authorized keys before staging; this does not establish device trust.
/// Snapshot application is deferred until reconciliation of absent actions exists.
/// Returns false only for the already-applied current version.
pub fn apply_pending(store: &ProtocolStore, collection: &str, version: u64) -> Result<bool> {
    store.write(|tx| apply_in(tx, collection, version))
}

fn apply_in(tx: &mut WriteTx<'_>, collection: &str, version: u64) -> Result<bool> {
    let state = load(tx, collection)?;
    let applied: u64 = tx
        .get_json(Table::Checkpoints, &applied_key(collection), 1)?
        .unwrap_or(0);
    if version == applied {
        return Ok(false);
    }
    if version < applied || version > state.version {
        return Err(invalid("invalid pending action version"));
    }
    let key = batch_key(collection, version);
    let bytes = Zeroizing::new(
        tx.get(Table::Checkpoints, &key, 1)?
            .ok_or_else(|| invalid("missing pending action batch"))?,
    );
    let batch: PendingBatch =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid pending action batch"))?;
    if batch.snapshot || batch.previous_version != applied {
        return Err(invalid(
            "snapshot reconciliation or earlier pending batch required",
        ));
    }
    let projections = batch
        .actions
        .iter()
        .map(decode_action)
        .collect::<Result<Vec<_>>>()?;
    for projection in projections {
        let (chat, pin) = match projection {
            Projection::Pin(chat, pin) => (chat, pin),
            Projection::Mute(chat, mute) => {
                mute::apply(tx, &chat, mute)?;
                continue;
            }
        };
        let key = pin_key(&chat);
        if let Some(old) = tx.get_json::<ChatPin>(Table::Checkpoints, &key, 1)? {
            // Conservative local conflict refusal until APK conflict resolution
            // across collections/local changes is integrated.
            if pin.timestamp < old.timestamp || (pin.timestamp == old.timestamp && pin != old) {
                return Err(invalid("pin timestamp conflict; batch remains pending"));
            }
        }
        tx.put_json(Table::Checkpoints, &key, 1, &pin)?;
    }
    tx.put_json(Table::Checkpoints, &applied_key(collection), 1, &version)?;
    tx.delete(Table::Checkpoints, &key)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };

    fn pin(pinned: bool) -> PendingAction {
        PendingAction {
            operation: 1,
            index: br#"["pin_v1","15550000000@s.whatsapp.net"]"#.to_vec(),
            value: Some(vec![8, 9, 0x2a, 2, 8, u8::from(pinned)]),
            schema_version: 5,
        }
    }
    fn mute(muted: bool, timestamp: u8) -> PendingAction {
        PendingAction {
            operation: 1,
            index: br#"["mute","15550000000@s.whatsapp.net"]"#.to_vec(),
            value: Some(vec![8, timestamp, 0x22, 4, 8, u8::from(muted), 16, 120]),
            schema_version: 2,
        }
    }
    #[test]
    fn pin_wire_presence_and_unsupported_fields() {
        assert!(decode_pin(&pin(true)).unwrap().1.pinned);
        assert!(!decode_pin(&pin(false)).unwrap().1.pinned);
        for bytes in [
            vec![8, 9, 0x2a, 0],
            vec![0x2a, 2, 8, 1],
            vec![8, 9, 0x2a, 4, 8, 1, 16, 0],
            vec![8, 9, 0x2a, 2, 8, 1, 0x12, 0],
        ] {
            let mut action = pin(true);
            action.value = Some(bytes);
            assert!(decode_pin(&action).is_err());
        }
        let mut action = pin(true);
        action.operation = 2;
        assert!(decode_pin(&action).is_err());
    }
    #[tokio::test]
    async fn pending_application_is_atomic_ordered_and_restart_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("actions.redb"),
            account_id: AccountId::new("test").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
        };
        let store = ProtocolStore::open(options()).unwrap();
        // Synthetic already-authenticated staging boundary; crypto verification
        // is separately tested with the independent encrypted fixtures.
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    &state_key("regular"),
                    1,
                    &SavedState {
                        collection: "regular".into(),
                        version: 2,
                        hash: vec![0; 128],
                        values: vec![],
                    },
                )?;
                for version in 1..=2 {
                    tx.put_json(
                        Table::Checkpoints,
                        &batch_key("regular", version),
                        1,
                        &PendingBatch {
                            previous_version: version - 1,
                            snapshot: false,
                            actions: vec![pin(true), mute(true, 9)],
                        },
                    )?;
                }
                Ok(())
            })
            .unwrap();
        assert!(apply_pending(&store, "regular", 2).is_err());
        let rollback: Result<()> = store.write(|tx| {
            apply_in(tx, "regular", 1)?;
            Err(Error::Cancelled)
        });
        assert!(rollback.is_err());
        let jid = Jid::parse("15550000000@s.whatsapp.net").unwrap();
        assert!(store.read(|tx| chat_pin(tx, &jid)).unwrap().is_none());
        assert!(store.read(|tx| chat_mute(tx, &jid)).unwrap().is_none());
        assert!(apply_pending(&store, "regular", 1).unwrap());
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert!(!apply_pending(&store, "regular", 1).unwrap());
        assert!(store.read(|tx| chat_pin(tx, &jid)).unwrap().unwrap().pinned);
        let saved_mute = store.read(|tx| chat_mute(tx, &jid)).unwrap().unwrap();
        assert!(saved_mute.muted);
        assert_eq!(saved_mute.end_timestamp, 120);
        store
            .write(|tx| {
                let mut unsupported = pin(false);
                unsupported.schema_version = 99;
                tx.put_json(
                    Table::Checkpoints,
                    &batch_key("regular", 2),
                    1,
                    &PendingBatch {
                        previous_version: 1,
                        snapshot: false,
                        actions: vec![pin(false), unsupported],
                    },
                )
            })
            .unwrap();
        assert!(apply_pending(&store, "regular", 2).is_err());
        assert!(store.read(|tx| chat_pin(tx, &jid)).unwrap().unwrap().pinned);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, &batch_key("regular", 2), 1))
                .unwrap()
                .is_some()
        );
        // A later conflict must roll back an earlier projection write too.
        store
            .write(|tx| {
                let mut newer = pin(false);
                newer.value = Some(vec![8, 10, 0x2a, 2, 8, 0]);
                tx.put_json(
                    Table::Checkpoints,
                    &batch_key("regular", 2),
                    1,
                    &PendingBatch {
                        previous_version: 1,
                        snapshot: false,
                        actions: vec![mute(false, 10), newer, pin(true)],
                    },
                )
            })
            .unwrap();
        assert!(apply_pending(&store, "regular", 2).is_err());
        assert!(store.read(|tx| chat_pin(tx, &jid)).unwrap().unwrap().pinned);
        assert_eq!(
            store.read(|tx| chat_mute(tx, &jid)).unwrap(),
            Some(saved_mute)
        );
        // A mute conflict must also roll back an earlier pin write.
        store
            .write(|tx| {
                let mut newer = pin(false);
                newer.value = Some(vec![8, 10, 0x2a, 2, 8, 0]);
                tx.put_json(
                    Table::Checkpoints,
                    &batch_key("regular", 2),
                    1,
                    &PendingBatch {
                        previous_version: 1,
                        snapshot: false,
                        actions: vec![newer, mute(false, 8)],
                    },
                )
            })
            .unwrap();
        assert!(apply_pending(&store, "regular", 2).is_err());
        assert!(store.read(|tx| chat_pin(tx, &jid)).unwrap().unwrap().pinned);
        assert_eq!(
            store.read(|tx| chat_mute(tx, &jid)).unwrap(),
            Some(saved_mute)
        );
        store
            .write(|tx| {
                let mut newer = pin(false);
                newer.value = Some(vec![8, 10, 0x2a, 2, 8, 0]);
                tx.put_json(
                    Table::Checkpoints,
                    &batch_key("regular", 2),
                    1,
                    &PendingBatch {
                        previous_version: 1,
                        snapshot: false,
                        actions: vec![newer, mute(false, 10)],
                    },
                )
            })
            .unwrap();
        assert!(apply_pending(&store, "regular", 2).unwrap());
        assert!(!store.read(|tx| chat_pin(tx, &jid)).unwrap().unwrap().pinned);
        assert!(!apply_pending(&store, "regular", 2).unwrap());
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        let unmuted = store.read(|tx| chat_mute(tx, &jid)).unwrap().unwrap();
        assert!(!unmuted.muted);
        assert_eq!(unmuted.end_timestamp, 0);
        assert!(!apply_pending(&store, "regular", 2).unwrap());
        #[cfg(feature = "comparison-client")]
        {
            // Public read sees only the successfully applied state, remains
            // offline, and does not consume a subsequently staged batch.
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        &batch_key("regular", 3),
                        1,
                        &PendingBatch {
                            previous_version: 2,
                            snapshot: false,
                            actions: vec![pin(true)],
                        },
                    )
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            drop(store);
            let mut config: crate::registration::request::LabRegistrationConfig =
                serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
            config.token = Zeroizing::new("offline-fixture".into());
            let client = crate::WhatsAppClient::builder()
                .store_path(options().path)
                .account_id(AccountId::new("test").unwrap())
                .key_source(options().key_source)
                .create_store(false)
                .comparison_transport(config)
                .build()
                .await
                .unwrap();
            let settings = client
                .synced_chat_settings(&ChatId::from_jid(&jid).unwrap())
                .await
                .unwrap();
            assert!(!settings.pin.unwrap().pinned);
            assert_eq!(settings.mute, Some(unmuted));
            let absent = client
                .synced_chat_settings(
                    &ChatId::from_jid(&Jid::parse("15550000001@s.whatsapp.net").unwrap()).unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(absent.pin, None);
            assert_eq!(absent.mute, None);
            assert_eq!(
                client
                    .store()
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
            assert!(matches!(
                client.connection_state().borrow().state,
                crate::runtime::ConnectionState::Stopped
            ));
            client.shutdown().await;
        }
    }
}
