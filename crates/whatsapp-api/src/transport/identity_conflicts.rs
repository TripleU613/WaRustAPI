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
}
