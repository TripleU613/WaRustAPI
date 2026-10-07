//! Local manual verification metadata, grounded in APK 2.26.37.71 Cg3.A00,
//! 0jc.A0K and 0kc.A0I. This is a human claim, not automatic QR verification
//! or companion synchronization. Evidence: docs/protocol/peer-identity.md.
use crate::{
    Error, Result,
    model::jid::{Jid, JidKind, Server},
    session::AccountRecord,
    store::{ProtocolStore, ReadTx, Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
struct Metadata {
    identity: Vec<u8>,
    verified: bool,
    action_sequence: u64,
}

/// Current local human-verification claim, never a server or scan assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerVerification {
    /// True only for an explicit human mark bound to the current pin.
    pub verified: bool,
    /// Local action sequence; zero before the first mark/unmark.
    pub action_sequence: u64,
    /// Whether the stored claim's serialized identity equals the current pin.
    pub identity_matches_pin: bool,
}

fn key(peer: &Jid) -> String {
    format!("peer-verification:comparison-direct:{peer}")
}
fn invalid(reason: &'static str) -> Error {
    Error::invalid("peer_verification", reason)
}

pub(crate) fn inspect_in(
    tx: &ReadTx<'_>,
    peer: &Jid,
    pin: Option<&[u8]>,
) -> Result<Option<PeerVerification>> {
    let record: Option<Metadata> = tx.get_json(Table::Checkpoints, key(peer).as_bytes(), 1)?;
    record
        .map(|record| {
            validate_metadata(&record)?;
            let matches = pin == Some(record.identity.as_slice());
            Ok(PeerVerification {
                verified: record.verified && matches,
                action_sequence: record.action_sequence,
                identity_matches_pin: matches,
            })
        })
        .transpose()
}

fn validate_metadata(record: &Metadata) -> Result<()> {
    wacore_libsignal::protocol::IdentityKey::decode(&record.identity)
        .map_err(|_| invalid("invalid saved verification identity"))?;
    if record.action_sequence > i64::MAX as u64 {
        return Err(invalid("saved action sequence exceeds signed int64"));
    }
    Ok(())
}

/// Set a local explicit human mark/unmark for a primary remote LID.
///
/// The caller asserts a human comparison/decision. No comparison is performed
/// here. Expected SHA-256 pin and action sequence bind that decision to the
/// inspected state. Both are checked inside one transaction; missing pin,
/// stale decision, unknown local LID, own identity and sequence exhaustion refuse.
/// Sequence CAS and overflow refusal are local client policies. No network,
/// companion action, pin/session rewrite or verification event is generated.
pub fn set_local(
    store: &ProtocolStore,
    peer: &Jid,
    expected_fingerprint: &str,
    expected_action_sequence: u64,
    verified: bool,
) -> Result<PeerVerification> {
    if peer.server != Server::Lid
        || peer.device != 0
        || peer.agent != 0
        || peer.user.is_empty()
        || peer.kind != JidKind::Plain
    {
        return Err(invalid("requires a primary remote LID"));
    }
    if expected_fingerprint.len() != 64 {
        return Err(invalid("expected pin fingerprint must be SHA-256 hex"));
    }
    let expected = hex::decode(expected_fingerprint)
        .map_err(|_| invalid("expected pin fingerprint must be SHA-256 hex"))?;
    if expected.len() != 32 {
        return Err(invalid("expected pin fingerprint must be SHA-256 hex"));
    }
    store.write(|tx| {
        let account = AccountRecord::load_in_write(tx)?
            .ok_or_else(|| invalid("saved account identity is missing"))?;
        let local_key = account
            .signal_identity
            .as_ref()
            .ok_or_else(|| invalid("saved local Signal identity is missing"))?;
        let login: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, b"login-identity", 1)?;
        let own_lid = login
            .as_ref()
            .and_then(|record| record["lid"].as_str())
            .and_then(|text| Jid::parse(text).ok())
            .filter(|jid| jid.server == Server::Lid && jid.device == 0 && jid.agent == 0)
            .ok_or_else(|| invalid("saved local LID is unknown; cannot exclude own account"))?;
        if own_lid == *peer
            || account
                .own_jid
                .as_ref()
                .is_some_and(|own| own.to_non_ad() == *peer)
        {
            return Err(invalid("own account verification is refused"));
        }
        let pin_key = format!("comparison-direct:{peer}");
        let pin: Vec<u8> = tx
            .get_json(Table::PeerIdentities, pin_key.as_bytes(), 1)?
            .ok_or_else(|| invalid("saved peer identity pin is missing"))?;
        let identity = wacore_libsignal::protocol::IdentityKey::decode(&pin)
            .map_err(|_| invalid("invalid saved peer identity pin"))?;
        if identity.public_key().public_key_bytes() == local_key.0.public {
            return Err(invalid("local Signal identity cannot be marked as a peer"));
        }
        // Cg3.A00 0130..016a rechecks the captured serialized identity under
        // the address lock. Our operator surface uses its SHA-256 selector;
        // the transaction keeps the check and metadata update indivisible.
        if Sha256::digest(&pin).as_slice() != expected {
            return Err(invalid(
                "saved peer pin changed since inspection; decision refused",
            ));
        }
        let record: Option<Metadata> = tx.get_json(Table::Checkpoints, key(peer).as_bytes(), 1)?;
        if let Some(record) = &record {
            validate_metadata(record)?;
        }
        let sequence = record.as_ref().map_or(0, |record| record.action_sequence);
        if sequence != expected_action_sequence {
            return Err(invalid(
                "verification action sequence changed since inspection",
            ));
        }
        let next = sequence
            .checked_add(1)
            .filter(|next| *next <= i64::MAX as u64)
            .ok_or_else(|| invalid("verification action sequence exhausted"))?;
        // Cg3.A00 00fa..0116 advances the sequence for explicit mark/unmark.
        tx.put_json(
            Table::Checkpoints,
            key(peer).as_bytes(),
            1,
            &Metadata {
                identity: pin,
                verified,
                action_sequence: next,
            },
        )?;
        Ok(PeerVerification {
            verified,
            action_sequence: next,
            identity_matches_pin: true,
        })
    })
}

pub(super) fn invalidate_in(tx: &mut WriteTx<'_>, peer: &Jid, current: &[u8]) -> Result<()> {
    let Some(mut record): Option<Metadata> =
        tx.get_json(Table::Checkpoints, key(peer).as_bytes(), 1)?
    else {
        return Ok(());
    };
    validate_metadata(&record)?;
    // APK 37.71 0jc.A0K 012c..0150 preserves action_seq but clears verified
    // when serialized identity differs. Rotation back must not restore a mark.
    if record.identity != current {
        record.identity = current.to_vec();
        record.verified = false;
        validate_metadata(&record)?;
        tx.put_json(Table::Checkpoints, key(peer).as_bytes(), 1, &record)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        session::{KeyMaterial, SignalIdentityKey},
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::IdentityKeyPair;
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.into(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create,
        })
        .unwrap()
    }
    fn setup(store: &ProtocolStore, peer: &Jid) -> Vec<u8> {
        let local = IdentityKeyPair::generate(&mut rand::rng());
        let pin = IdentityKeyPair::generate(&mut rand::rng())
            .identity_key()
            .serialize()
            .to_vec();
        store
            .write(|tx| {
                AccountRecord {
                    signal_identity: Some(SignalIdentityKey(KeyMaterial {
                        public: local.public_key().public_key_bytes().try_into().unwrap(),
                        private: *local.private_key().serialize(),
                    })),
                    ..Default::default()
                }
                .save(tx)?;
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"lid":"111111111111111@lid"}),
                )?;
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &pin,
                )?;
                tx.put_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &vec![1u8, 2, 3],
                )
            })
            .unwrap();
        pin
    }
    fn fingerprint(pin: &[u8]) -> String {
        hex::encode(Sha256::digest(pin))
    }
    fn state(store: &ProtocolStore, peer: &Jid, pin: &[u8]) -> PeerVerification {
        store
            .read(|tx| inspect_in(tx, peer, Some(pin)))
            .unwrap()
            .unwrap()
    }
    #[test]
    fn decisions_bind_to_pin_and_sequence_and_persist_without_session_mutation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("decision.redb");
        let store = open(&path, true);
        let peer = Jid::parse("222222222222222@lid").unwrap();
        let pin = setup(&store, &peer);
        assert!(set_local(&store, &peer, &"00".repeat(32), 0, true).is_err());
        let marked = set_local(&store, &peer, &fingerprint(&pin), 0, true).unwrap();
        assert_eq!(
            marked,
            PeerVerification {
                verified: true,
                action_sequence: 1,
                identity_matches_pin: true
            }
        );
        assert!(set_local(&store, &peer, &fingerprint(&pin), 0, true).is_err());
        let key = format!("comparison-direct:{peer}");
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1))
                .unwrap(),
            Some(pin.clone())
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1))
                .unwrap(),
            Some(vec![1, 2, 3])
        );
        drop(store);
        let store = open(&path, false);
        assert_eq!(state(&store, &peer, &pin), marked);
        assert!(
            !set_local(&store, &peer, &fingerprint(&pin), 1, false)
                .unwrap()
                .verified
        );
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    super::key(&peer).as_bytes(),
                    1,
                    &Metadata {
                        identity: pin.clone(),
                        verified: false,
                        action_sequence: i64::MAX as u64,
                    },
                )
            })
            .unwrap();
        assert!(set_local(&store, &peer, &fingerprint(&pin), i64::MAX as u64, true).is_err());
        assert_eq!(state(&store, &peer, &pin).action_sequence, i64::MAX as u64);
    }
    #[test]
    fn both_authenticated_archive_paths_invalidate_atomically_and_rotation_back_stays_unverified() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("rotation.redb"), true);
        let peer = Jid::parse("222222222222222@lid").unwrap();
        let old = setup(&store, &peer);
        let new = IdentityKeyPair::generate(&mut rand::rng())
            .identity_key()
            .serialize()
            .to_vec();
        let fp = fingerprint(&old);
        set_local(&store, &peer, &fp, 0, true).unwrap();
        let failed: Result<()> = store.write(|tx| {
            super::super::identity_conflicts::archive_replacement(
                tx,
                &peer,
                Some(&old),
                None,
                &new,
            )?;
            tx.put_json(
                Table::PeerIdentities,
                format!("comparison-direct:{peer}").as_bytes(),
                1,
                &new,
            )?;
            Err(invalid("synthetic transaction rollback"))
        });
        assert!(failed.is_err());
        assert!(state(&store, &peer, &old).verified);
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_replacement(
                    tx,
                    &peer,
                    Some(&old),
                    None,
                    &new,
                )?;
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &new,
                )
            })
            .unwrap();
        assert_eq!(
            state(&store, &peer, &new),
            PeerVerification {
                verified: false,
                action_sequence: 1,
                identity_matches_pin: true
            }
        );
        store
            .write(|tx| {
                super::super::identity_conflicts::archive_bundle_replacement(
                    tx,
                    &peer,
                    Some(&new),
                    None,
                    &old,
                )?;
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &old,
                )
            })
            .unwrap();
        assert!(!state(&store, &peer, &old).verified);
        assert!(set_local(&store, &peer, &fp, 0, true).is_err());
        assert!(set_local(&store, &peer, &fp, 1, true).unwrap().verified);
    }
    #[test]
    fn missing_prerequisites_and_own_identity_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("guards.redb"), true);
        let peer = Jid::parse("222222222222222@lid").unwrap();
        assert!(set_local(&store, &peer, &"00".repeat(32), 0, true).is_err());
        let pin = setup(&store, &peer);
        for text in [
            "111111111111111@lid",
            "222222222222222:2@lid",
            "15555550100@s.whatsapp.net",
            "333333333333333@lid",
        ] {
            assert!(
                set_local(
                    &store,
                    &Jid::parse(text).unwrap(),
                    &fingerprint(&pin),
                    0,
                    true
                )
                .is_err()
            );
        }
        store
            .write(|tx| {
                let account = AccountRecord::load_in_write(tx)?.unwrap();
                let public = account.signal_identity.unwrap().0.public;
                let mut own = vec![5];
                own.extend_from_slice(&public);
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &own,
                )
            })
            .unwrap();
        assert!(
            set_local(&store, &peer, &fingerprint(&pin), 0, true)
                .unwrap_err()
                .to_string()
                .contains("local Signal")
        );
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({}),
                )
            })
            .unwrap();
        assert!(
            set_local(&store, &peer, &fingerprint(&pin), 0, true)
                .unwrap_err()
                .to_string()
                .contains("local LID is unknown")
        );
    }
}
