//! Signal backend boundary (docs/design/README.md section 10).
//!
//! No backend has passed the interoperability gate (ADR 0005). This module
//! defines the adapter boundary and the transactional overlay the account
//! actor uses so that ratchet mutations commit atomically with outbox/inbox
//! state. The only backend implementation is [`NoBackend`], which returns
//! typed `Unsupported` errors. Nothing here hides wire incompatibility.

use std::collections::BTreeMap;
use std::fmt;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::capabilities::Capability;
use crate::error::{Error, Result};
use crate::model::ids::{DeviceAddress, GroupId};
use crate::store::{Table, WriteTx};

#[cfg(feature = "experimental-signal-crypto")]
pub mod crypto;

const RECORD_V1: u16 = 1;

/// Which record a Signal store operation touches.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecordKey {
    /// Pairwise session for a device.
    Session(DeviceAddress),
    /// Trusted identity for a device.
    PeerIdentity(DeviceAddress),
    /// Prekey by kind and id; kinds cannot collide.
    PreKey {
        /// `signed` or `one_time`.
        kind: PreKeyKind,
        /// Identifier within the kind.
        id: u32,
    },
    /// Sender key for a group/sender/distribution triple.
    SenderKey {
        /// Group.
        group: GroupId,
        /// Sending device.
        sender: DeviceAddress,
        /// Distribution identifier.
        distribution_id: u32,
    },
}

/// Prekey kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PreKeyKind {
    /// Signed prekey.
    Signed,
    /// One-time prekey.
    OneTime,
}

impl RecordKey {
    fn table(&self) -> Table {
        match self {
            Self::Session(_) => Table::SignalSessions,
            Self::PeerIdentity(_) => Table::PeerIdentities,
            Self::PreKey { .. } => Table::Prekeys,
            Self::SenderKey { .. } => Table::SenderKeys,
        }
    }

    fn bytes(&self) -> Vec<u8> {
        match self {
            Self::Session(d) => format!("s:{}", d.to_jid()).into_bytes(),
            Self::PeerIdentity(d) => format!("i:{}", d.to_jid()).into_bytes(),
            Self::PreKey { kind, id } => format!("p:{kind:?}:{id}").into_bytes(),
            Self::SenderKey {
                group,
                sender,
                distribution_id,
            } => format!("k:{}:{}:{distribution_id}", group.jid(), sender.to_jid()).into_bytes(),
        }
    }
}

/// A loaded record with its revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Versioned {
    /// Opaque backend serialization (secret; encrypted at rest by the store).
    pub bytes: Bytes,
    /// Revision counter, incremented on every write.
    pub revision: u64,
}

/// In-memory transaction overlay over Signal records (docs/design/README.md 11.4).
///
/// The backend reads and writes the overlay only. The account actor commits
/// the mutations atomically with the message/event records, checking that no
/// record changed since it was loaded.
#[derive(Debug, Default)]
pub struct Overlay {
    loaded: BTreeMap<RecordKey, Option<Versioned>>,
    dirty: BTreeMap<RecordKey, Option<Bytes>>,
}

impl Overlay {
    /// Load a record snapshot into the overlay.
    pub fn load(&mut self, tx: &WriteTx<'_>, key: RecordKey) -> Result<Option<&Versioned>> {
        if !self.loaded.contains_key(&key) {
            let v: Option<Versioned> = tx.get_json(key.table(), &key.bytes(), RECORD_V1)?;
            self.loaded.insert(key.clone(), v);
        }
        Ok(self.loaded.get(&key).and_then(Option::as_ref))
    }

    /// Current view (dirty overrides loaded).
    pub fn get(&self, key: &RecordKey) -> Option<Bytes> {
        if let Some(d) = self.dirty.get(key) {
            return d.clone();
        }
        self.loaded
            .get(key)
            .and_then(|v| v.as_ref().map(|v| v.bytes.clone()))
    }

    /// Stage a write.
    pub fn put(&mut self, key: RecordKey, bytes: Bytes) {
        self.dirty.insert(key, Some(bytes));
    }

    /// Stage a delete (one-time prekey consumption).
    pub fn delete(&mut self, key: RecordKey) {
        self.dirty.insert(key, None);
    }

    /// Number of staged mutations.
    pub fn dirty_len(&self) -> usize {
        self.dirty.len()
    }

    /// Discard staged mutations (decrypt failure / backend error).
    pub fn discard(&mut self) {
        self.dirty.clear();
    }

    /// Commit staged mutations with expected-revision checks. On conflict
    /// nothing is written and the caller must reload and recompute.
    pub fn commit(self, tx: &mut WriteTx<'_>) -> Result<usize> {
        // Verify revisions first.
        for key in self.dirty.keys() {
            let expected = self.loaded.get(key).cloned().flatten().map(|v| v.revision);
            let current: Option<Versioned> = tx.get_json(key.table(), &key.bytes(), RECORD_V1)?;
            if current.as_ref().map(|v| v.revision) != expected {
                return Err(Error::Storage(crate::store::StoreError::Serialization(
                    format!("revision conflict on {key:?}; reload required"),
                )));
            }
        }
        let n = self.dirty.len();
        for (key, value) in self.dirty {
            match value {
                Some(bytes) => {
                    let revision = self
                        .loaded
                        .get(&key)
                        .cloned()
                        .flatten()
                        .map_or(1, |v| v.revision + 1);
                    tx.put_json(
                        key.table(),
                        &key.bytes(),
                        RECORD_V1,
                        &Versioned { bytes, revision },
                    )?;
                }
                None => {
                    tx.delete(key.table(), &key.bytes())?;
                }
            }
        }
        Ok(n)
    }
}

/// Kind of an outgoing E2EE message envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CiphertextKind {
    /// Session-establishing prekey message.
    PreKey,
    /// Ordinary ratchet message.
    Message,
    /// Sender-key group message.
    SenderKey,
}

/// An encrypted payload produced by a backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext {
    /// Envelope kind.
    pub kind: CiphertextKind,
    /// Bytes.
    pub body: Bytes,
}

/// Backend adapter. Every method operates on the overlay; no I/O.
pub trait SignalBackend: Send + Sync + fmt::Debug {
    /// Backend name and pinned revision for capability reports.
    fn identity(&self) -> &'static str;

    /// Encrypt to one device.
    fn encrypt(
        &self,
        overlay: &mut Overlay,
        to: &DeviceAddress,
        plaintext: &[u8],
    ) -> Result<Ciphertext>;

    /// Decrypt from one device.
    fn decrypt(
        &self,
        overlay: &mut Overlay,
        from: &DeviceAddress,
        ciphertext: &Ciphertext,
    ) -> Result<Bytes>;
}

/// Placeholder until a backend passes the section 10 gate.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoBackend;

impl SignalBackend for NoBackend {
    fn identity(&self) -> &'static str {
        "none (ADR 0005: no backend has passed the interoperability gate)"
    }

    fn encrypt(
        &self,
        _overlay: &mut Overlay,
        _to: &DeviceAddress,
        _plaintext: &[u8],
    ) -> Result<Ciphertext> {
        Err(Error::unsupported(Capability::DirectText, self.identity()))
    }

    fn decrypt(
        &self,
        _overlay: &mut Overlay,
        _from: &DeviceAddress,
        _ciphertext: &Ciphertext,
    ) -> Result<Bytes> {
        Err(Error::unsupported(Capability::DirectText, self.identity()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::model::ids::AccountId;
    use crate::model::jid::Jid;
    use crate::store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource};

    fn store() -> (tempfile::TempDir, Arc<ProtocolStore>) {
        let dir = tempfile::tempdir().unwrap();
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, Arc::new(s))
    }

    fn dev() -> DeviceAddress {
        DeviceAddress::from_jid(&Jid::parse("15550000000:2@s.whatsapp.net").unwrap()).unwrap()
    }

    #[test]
    fn overlay_commits_with_revision_checks() {
        let (_d, s) = store();
        let key = RecordKey::Session(dev());
        s.write(|tx| {
            let mut ov = Overlay::default();
            assert!(ov.load(tx, key.clone())?.is_none());
            ov.put(key.clone(), Bytes::from_static(b"v1"));
            assert_eq!(ov.get(&key), Some(Bytes::from_static(b"v1")));
            assert_eq!(ov.commit(tx)?, 1);
            Ok(())
        })
        .unwrap();
        // Stale overlay: loaded revision 1, but someone wrote revision 2.
        let mut stale = Overlay::default();
        s.write(|tx| {
            stale.load(tx, key.clone())?;
            Ok(())
        })
        .unwrap();
        s.write(|tx| {
            let mut ov = Overlay::default();
            ov.load(tx, key.clone())?;
            ov.put(key.clone(), Bytes::from_static(b"v2"));
            ov.commit(tx)?;
            Ok(())
        })
        .unwrap();
        stale.put(key.clone(), Bytes::from_static(b"derived-from-stale"));
        let r: Result<usize> = s.write(|tx| stale.commit(tx));
        assert!(r.is_err(), "stale snapshot must not commit");
        let v: Option<Versioned> = s
            .read(|tx| tx.get_json(Table::SignalSessions, &key.bytes(), RECORD_V1))
            .unwrap();
        assert_eq!(v.unwrap().bytes, Bytes::from_static(b"v2"));
        // Discard drops staged mutations.
        let mut ov = Overlay::default();
        ov.put(key.clone(), Bytes::from_static(b"x"));
        ov.discard();
        assert_eq!(ov.dirty_len(), 0);
    }

    #[test]
    fn prekey_kinds_do_not_collide_and_no_backend_is_unsupported() {
        let a = RecordKey::PreKey {
            kind: PreKeyKind::Signed,
            id: 1,
        };
        let b = RecordKey::PreKey {
            kind: PreKeyKind::OneTime,
            id: 1,
        };
        assert_ne!(a.bytes(), b.bytes());
        let mut ov = Overlay::default();
        let err = NoBackend.encrypt(&mut ov, &dev(), b"x").unwrap_err();
        assert!(matches!(
            err,
            Error::Unsupported {
                capability: Capability::DirectText,
                ..
            }
        ));
    }
}
