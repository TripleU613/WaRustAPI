//! Credential lifecycle and the key-source boundary (docs/design/README.md section 7 and 11).
//!
//! The account record holds our identities and *observed* credentials only.
//! Noise static keys and Signal identity keys are distinct typed values.
//! No field is invented because another API uses it.

use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::error::Result;
use crate::model::ids::AccountGeneration;
use crate::model::jid::Jid;
use crate::model::phone::PhoneNumber;
use crate::model::time::Timestamp;
use crate::store::{ReadTx, Table, WriteTx};

const RECORD_V1: u16 = 1;
const ACCOUNT_KEY: &[u8] = b"account";

/// A 32-byte private key with a 32-byte public key. Distinct newtypes wrap
/// it so Noise and Signal material cannot be swapped.
#[derive(Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct KeyMaterial {
    /// Public half.
    pub public: [u8; 32],
    /// Private half. Stored only inside encrypted envelopes.
    pub private: [u8; 32],
}

impl fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyMaterial(pub={})", hex::encode(self.public))
    }
}

/// Noise static identity (transport layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseStaticKey(pub KeyMaterial);

/// Signal identity key (E2EE layer). Not derived from the Noise key unless
/// evidence establishes a relationship (README main changes table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalIdentityKey(pub KeyMaterial);

/// One server-issued credential exactly as observed.
#[derive(Clone, Serialize, Deserialize)]
pub struct ObservedCredential {
    /// Protocol-level name of the credential (e.g. the response field name).
    pub name: String,
    /// Opaque value.
    pub value: Zeroizing<Vec<u8>>,
    /// When it was observed.
    pub observed_at: Timestamp,
    /// Registration attempt that produced it, if applicable.
    pub source_step: Option<String>,
}

impl fmt::Debug for ObservedCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ObservedCredential({}, <{} bytes>)",
            self.name,
            self.value.len()
        )
    }
}

/// Registration status summary kept with the account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RegistrationStatus {
    /// No completed registration.
    #[default]
    Unregistered,
    /// Registration committed for this generation.
    Registered,
    /// The server revoked the session.
    Revoked,
}

/// The singleton account record.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct AccountRecord {
    /// Generation this record belongs to.
    pub generation: AccountGeneration,
    /// Registered phone number, once known.
    pub phone: Option<PhoneNumber>,
    /// Our own JID, once assigned by the server.
    pub own_jid: Option<Jid>,
    /// Noise static key.
    pub noise_static: Option<NoiseStaticKey>,
    /// Signal identity key.
    pub signal_identity: Option<SignalIdentityKey>,
    /// Signal registration id, once generated.
    pub signal_registration_id: Option<u32>,
    /// Registration status.
    pub registration: RegistrationStatus,
    /// Observed credentials.
    pub credentials: Vec<ObservedCredential>,
    /// Last successful authenticated connection.
    pub last_authenticated_at: Option<Timestamp>,
}

impl fmt::Debug for AccountRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountRecord")
            .field("generation", &self.generation)
            .field("phone", &self.phone)
            .field("own_jid", &self.own_jid)
            .field("has_noise_static", &self.noise_static.is_some())
            .field("has_signal_identity", &self.signal_identity.is_some())
            .field("registration", &self.registration)
            .field("credentials", &self.credentials.len())
            .finish()
    }
}

impl AccountRecord {
    /// Load the account record, if any.
    pub fn load(tx: &ReadTx<'_>) -> Result<Option<Self>> {
        tx.get_json(Table::Account, ACCOUNT_KEY, RECORD_V1)
    }

    /// Load inside a write transaction.
    pub fn load_mut(tx: &WriteTx<'_>) -> Result<Option<Self>> {
        tx.get_json(Table::Account, ACCOUNT_KEY, RECORD_V1)
    }

    /// Persist the record.
    pub fn save(&self, tx: &mut WriteTx<'_>) -> Result<()> {
        tx.put_json(Table::Account, ACCOUNT_KEY, RECORD_V1, self)
    }

    /// Whether the account can authenticate without fresh registration.
    pub fn is_registered(&self) -> bool {
        self.registration == RegistrationStatus::Registered
    }
}

/// Export/restore policy. Only explicit, verified procedures are allowed;
/// none exist yet for this profile, so both operations are unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportPolicy {
    /// Never export secret material.
    #[default]
    Deny,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::model::ids::AccountId;
    use crate::store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource};

    #[test]
    fn account_record_roundtrip_and_redaction() {
        let dir = tempfile::tempdir().unwrap();
        let s = Arc::new(
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("p.redb"),
                account_id: AccountId::new("a").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([3u8; 32]))),
                create: true,
            })
            .unwrap(),
        );
        assert!(s.read(AccountRecord::load).unwrap().is_none());
        let rec = AccountRecord {
            generation: AccountGeneration(1),
            phone: Some(PhoneNumber::new(1, "5551234567").unwrap()),
            noise_static: Some(NoiseStaticKey(KeyMaterial {
                public: [1; 32],
                private: [2; 32],
            })),
            credentials: vec![ObservedCredential {
                name: "example".into(),
                value: Zeroizing::new(b"opaque".to_vec()),
                observed_at: Timestamp::now(),
                source_step: None,
            }],
            ..AccountRecord::default()
        };
        s.write(|tx| rec.save(tx)).unwrap();
        let back = s.read(AccountRecord::load).unwrap().unwrap();
        assert_eq!(back.noise_static.as_ref().unwrap().0.private, [2; 32]);
        assert!(!back.is_registered());
        let dbg = format!("{back:?}");
        assert!(!dbg.contains("5551234567"));
        assert!(!dbg.contains("opaque"));
        let raw = std::fs::read(s.path()).unwrap();
        assert!(!raw.windows(6).any(|w| w == b"opaque"));
    }
}
