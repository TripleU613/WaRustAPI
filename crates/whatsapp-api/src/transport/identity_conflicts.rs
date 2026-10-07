//! Durable observations of differing public keys, never authority to replace pins.
//! APK 2.26.37.71 LX/0jc.A0K separates identity saving from verified state;
//! LX/0kc.A0I stores mark_as_verified/action_seq. Our trust transition is still
//! unimplemented, so an unauthenticated envelope must remain an observation.
use crate::{
    Error, Result,
    model::{jid::Jid, time::Timestamp},
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Redacted, first-seen observation. Fingerprints are SHA-256 of serialized
/// public keys, not WhatsApp safety numbers. No key or message is exposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityConflict {
    /// Stable local selector for this peer/pinned/observed combination.
    pub ticket: String,
    /// Local hash of the session address, without the address itself.
    pub peer_ticket: String,
    /// Public key fingerprint saved when this conflict was observed.
    pub pinned_fingerprint: String,
    /// Public key fingerprint carried by the unauthenticated prekey envelope.
    pub observed_fingerprint: String,
    /// First local observation time in milliseconds since the Unix epoch.
    pub observed_at: Timestamp,
}

pub(super) fn observe(
    store: &ProtocolStore,
    peer: &Jid,
    ciphertext: &[u8],
    pinned: Option<&[u8]>,
) -> Result<bool> {
    let Some(pinned) = pinned else {
        return Ok(false);
    };
    // Malformed envelopes remain the decryptor's responsibility. They cannot
    // create a conflict record, and storage failures must not be swallowed.
    let Ok((old, new)) =
        crate::signal::direct::incoming_prekey_identity_fingerprints(ciphertext, pinned)
    else {
        return Ok(false);
    };
    record(store, peer, pinned, old, new)
}

fn record(
    store: &ProtocolStore,
    peer: &Jid,
    pinned: &[u8],
    old: String,
    new: String,
) -> Result<bool> {
    if old == new {
        return Ok(false);
    }
    let peer_key = format!("comparison-direct:{peer}");
    let peer_ticket = hex::encode(Sha256::digest(peer_key.as_bytes()));
    let ticket = hex::encode(Sha256::digest(
        serde_json::to_vec(&(&peer_ticket, &old, &new))
            .map_err(|_| Error::invalid("identity_conflict", "invalid fingerprint tuple"))?,
    ));
    let key = format!("identity-conflict:{ticket}");
    store.write(|tx| {
        let current: Option<Vec<u8>> =
            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        if current.as_deref() != Some(pinned) {
            return Err(Error::invalid(
                "identity_conflict",
                "saved pin changed during observation",
            ));
        }
        if tx
            .get_json::<IdentityConflict>(Table::Checkpoints, key.as_bytes(), 1)?
            .is_some()
        {
            return Ok(false);
        }
        tx.put_json(
            Table::Checkpoints,
            key.as_bytes(),
            1,
            &IdentityConflict {
                ticket,
                peer_ticket,
                pinned_fingerprint: old,
                observed_fingerprint: new,
                observed_at: Timestamp::now(),
            },
        )?;
        Ok(true)
    })
}

/// Inspect durable conflict observations offline. No session, pin, journal,
/// receipt or network state is changed; records do not assert authentication.
pub fn list(store: &ProtocolStore) -> Result<Vec<IdentityConflict>> {
    store.read(|tx| {
        let mut result = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(k, _)| k.clone());
            for (key, value) in rows {
                if key.starts_with(b"identity-conflict:") {
                    result.push(serde_json::from_slice(&value).map_err(|_| {
                        Error::invalid("identity_conflict", "invalid saved observation")
                    })?);
                }
            }
        }
        Ok(result)
    })
}

/// Commit authenticated replacement history in the same transaction as inbox,
/// session and prekey changes. Call only after OCC checks and authentication.
pub(super) fn archive_replacement(
    tx: &mut crate::store::WriteTx<'_>,
    peer: &Jid,
    previous: Option<&[u8]>,
    previous_session: Option<&[u8]>,
    current: &[u8],
) -> Result<()> {
    let Some(previous) = previous.filter(|old| *old != current) else {
        return Ok(());
    };
    let observed_at = Timestamp::now();
    let event = tx.append_event(crate::sync::SyncEvent::IdentityChanged(
        crate::sync::IdentityChange {
            device: crate::model::ids::DeviceAddress::from_jid(peer)?,
            previous_fingerprint: Some(hex::encode(Sha256::digest(previous))),
            new_fingerprint: hex::encode(Sha256::digest(current)),
            observed_at,
        },
    ))?;
    // Unique journal event selector preserves repeated A->B->A transitions;
    // a deterministic old/new selector would overwrite distinct ratchets.
    let key = format!("identity-history:{}", hex::encode(event.event_id.0));
    tx.put_json(
        Table::Checkpoints,
        key.as_bytes(),
        1,
        &serde_json::json!({
            "peer":peer, "previous_identity":previous, "previous_session":previous_session,
            "current_fingerprint":hex::encode(Sha256::digest(current)), "at":observed_at,
            "verification":"unverified", "policy":"apk_authenticated",
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create,
        })
        .unwrap()
    }
    #[tokio::test]
    async fn incoming_prekey_conflict_is_recorded_before_decryption_can_fail() {
        use wacore_libsignal::protocol::{IdentityKeyPair, PreKeySignalMessage, SignalMessage};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("incoming.redb"), true);
        let mut rng = rand::rng();
        let saved = IdentityKeyPair::generate(&mut rng);
        let observed = IdentityKeyPair::generate(&mut rng);
        let receiver = IdentityKeyPair::generate(&mut rng);
        let peer = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let peer_key = format!("comparison-direct:{peer}");
        let pin = saved.identity_key().serialize().to_vec();
        store
            .write(|tx| tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &pin))
            .unwrap();
        let nested = SignalMessage::new(
            3,
            &[1; 32],
            *observed.public_key(),
            0,
            0,
            b"not authenticated plaintext",
            observed.identity_key(),
            receiver.identity_key(),
        )
        .unwrap();
        let wire = PreKeySignalMessage::new(
            3,
            17,
            None,
            7.into(),
            *observed.public_key(),
            *observed.identity_key(),
            nested,
        )
        .unwrap();
        let node = crate::binary::Node::new("message")
            .attr("from", peer)
            .attr("id", "conflict-test")
            .attr("type", "text")
            .children(vec![
                crate::binary::Node::new("enc")
                    .attr("type", "pkmsg")
                    .attr("v", "2")
                    .bytes(wire.serialized().to_vec()),
            ]);
        let capture = super::super::inbox::preserve(&store, &node).unwrap();
        // No registered signed prekey: decryption cannot run, but the observed
        // public identity difference must remain diagnosable and not trusted.
        let _ = super::super::inbox::process(
            &store,
            &crate::session::AccountRecord::default(),
            &node,
            &capture,
        )
        .await;
        let rows = list(&store).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].observed_fingerprint,
            hex::encode(Sha256::digest(observed.identity_key().serialize()))
        );
        let _ = super::super::inbox::process(
            &store,
            &crate::session::AccountRecord::default(),
            &node,
            &capture,
        )
        .await;
        assert_eq!(list(&store).unwrap(), rows);
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::PeerIdentities, peer_key.as_bytes(), 1))
                .unwrap(),
            Some(pin)
        );
        assert_eq!(store.read(|tx| tx.count(Table::SignalSessions)).unwrap(), 0);
    }
    #[test]
    fn observations_survive_restart_without_replacing_pins_or_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("conflict.redb");
        let store = open(&path, true);
        let peer = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let key = format!("comparison-direct:{peer}");
        let pin = vec![5u8; 33];
        store
            .write(|tx| {
                tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &pin)?;
                tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &vec![7u8; 20])
            })
            .unwrap();
        let old = hex::encode(Sha256::digest(&pin));
        let new = hex::encode(Sha256::digest([6u8; 33]));
        assert!(!record(&store, &peer, &pin, old.clone(), old.clone()).unwrap());
        assert!(record(&store, &peer, &pin, old.clone(), new.clone()).unwrap());
        let rows = list(&store).unwrap();
        let output = serde_json::to_string(&rows).unwrap();
        assert!(!output.contains(&peer.user));
        assert_eq!(rows.len(), 1);
        assert!(!observe(&store, &peer, b"malformed", Some(&pin)).unwrap());
        assert!(!observe(&store, &peer, b"malformed", None).unwrap());
        drop(store);
        let store = open(&path, false);
        assert!(!record(&store, &peer, &pin, old.clone(), new.clone()).unwrap());
        assert_eq!(list(&store).unwrap(), rows);
        assert!(record(&store, &peer, &[8u8; 33], old, new).is_err());
        store
            .read(|tx| {
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1)?,
                    Some(pin)
                );
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?,
                    Some(vec![7; 20])
                );
                Ok(())
            })
            .unwrap();
    }

    #[tokio::test]
    async fn authenticated_replacement_is_atomic_for_direct_and_group_messages() {
        use crate::{
            binary::Node,
            registration::request::{IdentityChangePolicy, LabRegistrationConfig},
            session::{AccountRecord, KeyMaterial, RegistrationStatus, SignalIdentityKey},
        };
        use wacore_libsignal::protocol::{
            DeviceId, GenericSignedPreKey, IdentityKey, IdentityKeyPair, PreKeyBundle,
            SessionRecord, SignedPreKeyRecord,
        };
        for group in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("replace.redb");
            let store = open(&path, true);
            let mut config: LabRegistrationConfig =
                serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
            config.registration_id = 42;
            config.signed_pre_key_id = 7;
            crate::registration::factory::install_test_material(&store, &config).unwrap();
            let receiver = store.read(AccountRecord::load).unwrap().unwrap();
            let signed = crate::registration::factory::registered_signed_prekey(&store).unwrap();
            let signed = SignedPreKeyRecord::deserialize(&signed).unwrap();
            let local = receiver.signal_identity.as_ref().unwrap();
            let mut serialized_identity = vec![5];
            serialized_identity.extend(local.0.public);
            let bundle = PreKeyBundle::new(
                42,
                DeviceId::new(0),
                None,
                7.into(),
                signed.public_key().unwrap(),
                signed.signature().unwrap().to_vec(),
                IdentityKey::decode(&serialized_identity).unwrap(),
            )
            .unwrap();
            let mut rng = rand::rng();
            let alice = IdentityKeyPair::generate(&mut rng);
            let old = IdentityKeyPair::generate(&mut rng)
                .identity_key()
                .serialize()
                .to_vec();
            let sender = Jid::parse("15557654321@s.whatsapp.net").unwrap();
            let peer_key = format!("comparison-direct:{sender}");
            let old_session = SessionRecord::new_fresh().serialize().unwrap();
            store
                .write(|tx| {
                    tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &old)?;
                    tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &old_session)
                })
                .unwrap();
            let sending = AccountRecord {
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(31),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public: alice.public_key().public_key_bytes().try_into().unwrap(),
                    private: *alice.private_key().serialize(),
                })),
                ..Default::default()
            };
            let prepared = crate::signal::direct::prepare(
                &sending,
                &receiver.own_jid.clone().unwrap(),
                &bundle,
                None,
                None,
                "identity test",
            )
            .await
            .unwrap();
            let node = |id: &str, ciphertext: Vec<u8>| {
                let mut n = Node::new("message").attr("id", id).attr("type", "text");
                n = if group {
                    n.attr("from", "120363000000000001@g.us")
                        .attr("participant", sender.clone())
                } else {
                    n.attr("from", sender.clone())
                };
                n.children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "pkmsg")
                        .bytes(ciphertext),
                ])
            };
            let n = node("identity-good", prepared.ciphertext.clone());
            let capture = super::super::inbox::preserve(&store, &n).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &n, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Undecryptable
            );
            let initial_head = store.journal_head().unwrap();
            let mut damaged = prepared.ciphertext.clone();
            let envelope = wacore_libsignal::protocol::PreKeySignalMessage::try_from(
                prepared.ciphertext.as_slice(),
            )
            .unwrap();
            let nested = envelope.message().serialized();
            let start = damaged
                .windows(nested.len())
                .position(|w| w == nested)
                .unwrap();
            damaged[start + nested.len() - 1] ^= 1;
            let bad = node("identity-bad", damaged);
            let bad_capture = super::super::inbox::preserve(&store, &bad).unwrap();
            assert_eq!(
                super::super::inbox::process_with_policy(
                    &store,
                    &receiver,
                    &bad,
                    &bad_capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                super::super::inbox::IncomingResult::Undecryptable
            );
            assert_eq!(store.journal_head().unwrap(), initial_head);
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<Vec<u8>>(
                        Table::PeerIdentities,
                        peer_key.as_bytes(),
                        1
                    ))
                    .unwrap(),
                Some(old.clone())
            );
            assert_eq!(
                super::super::inbox::process_with_policy(
                    &store,
                    &receiver,
                    &n,
                    &capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                super::super::inbox::IncomingResult::Decrypted
            );
            let head = store.journal_head().unwrap();
            assert!(head > initial_head);
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<Vec<u8>>(
                        Table::PeerIdentities,
                        peer_key.as_bytes(),
                        1
                    ))
                    .unwrap(),
                Some(alice.identity_key().serialize().to_vec())
            );
            let archives = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 256))
                .unwrap();
            let archives: Vec<_> = archives
                .into_iter()
                .filter(|(k, _)| k.starts_with(b"identity-history:"))
                .collect();
            assert_eq!(archives.len(), 1);
            let history: serde_json::Value = serde_json::from_slice(&archives[0].1).unwrap();
            assert_eq!(history["previous_identity"], serde_json::json!(old));
            assert_eq!(history["previous_session"], serde_json::json!(old_session));
            assert_eq!(history["verification"], "unverified");
            // Aborting a replacement transaction rolls back both its archive
            // and its journal event, including a repeated key transition.
            let before = store.journal_head().unwrap();
            let failed: Result<()> = store.write(|tx| {
                archive_replacement(
                    tx,
                    &sender,
                    Some(&alice.identity_key().serialize()),
                    Some(&old_session),
                    &old,
                )?;
                Err(Error::invalid("test", "simulated commit refusal"))
            });
            assert!(failed.is_err());
            assert_eq!(store.journal_head().unwrap(), before);
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 256))
                    .unwrap()
                    .iter()
                    .filter(|(k, _)| k.starts_with(b"identity-history:"))
                    .count(),
                1
            );
            // Duplicate after reopening must not rotate again or append history/events.
            drop(store);
            let store = open(&path, false);
            assert_eq!(
                super::super::inbox::process_with_policy(
                    &store,
                    &receiver,
                    &n,
                    &capture,
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap(),
                super::super::inbox::IncomingResult::Duplicate
            );
            assert_eq!(store.journal_head().unwrap(), head);
        }
    }
}
