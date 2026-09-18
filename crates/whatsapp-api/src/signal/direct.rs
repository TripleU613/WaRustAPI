//! Single-recipient primary-device encryption over the upstream Signal backend.
//! Mutations stay in memory until caller atomically commits them with send intent.
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::Jid,
    session::AccountRecord,
};
use async_trait::async_trait;
use prost::Message;
use wacore_libsignal::protocol as signal;
use wacore_libsignal::protocol::{
    CiphertextMessageType, DeviceId, Direction, IdentityChange, IdentityKey, IdentityKeyPair,
    IdentityKeyStore, PreKeyBundle, PrivateKey, ProtocolAddress, PublicKey, SessionRecord,
    SessionStore, SignalProtocolError, UsePQRatchet, message_encrypt, process_prekey_bundle,
};
use zeroize::Zeroizing;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("direct_message", reason)
}
fn crypto(_: SignalProtocolError) -> Error {
    invalid("Signal key/session operation failed")
}
fn data(node: &Node) -> Result<&[u8]> {
    match &node.content {
        NodeContent::Bytes(v) => Ok(v),
        NodeContent::Text(v) => Ok(v.as_bytes()),
        _ => Err(invalid("expected key bytes")),
    }
}
fn child<'a>(node: &'a Node, name: &str) -> Result<&'a Node> {
    let mut matches = node.child_nodes().iter().filter(|v| v.description == name);
    let value = matches
        .next()
        .ok_or_else(|| invalid("missing key bundle field"))?;
    if matches.next().is_some() {
        return Err(invalid("duplicate key bundle field"));
    }
    Ok(value)
}
fn number(node: &Node, size: usize) -> Result<u32> {
    let bytes = data(node)?;
    if bytes.len() != size {
        return Err(invalid("invalid key identifier width"));
    }
    Ok(bytes
        .iter()
        .fold(0, |value, byte| (value << 8) | *byte as u32))
}
fn public(node: &Node) -> Result<PublicKey> {
    let bytes = data(node)?;
    if bytes.len() != 32 {
        return Err(invalid("invalid public key width"));
    }
    PublicKey::from_djb_public_key_bytes(bytes).map_err(|_| invalid("invalid public key"))
}

/// Validate identity/registration/signed-key fields for the exact requested JID.
pub fn prekey_bundle(reply: &Node, recipient: &Jid) -> Result<PreKeyBundle> {
    if reply.description != "iq"
        || reply.get_attr("type").and_then(|v| v.as_text()) != Some("result")
    {
        return Err(invalid("key query was not successful"));
    }
    let list = child(reply, "list")?;
    let user = child(list, "user")?;
    let jid = user
        .get_attr("jid")
        .ok_or_else(|| invalid("missing bundle JID"))?;
    let matches = jid
        .as_jid()
        .is_some_and(|v| v.to_non_ad() == recipient.to_non_ad() && v.device == recipient.device)
        || jid.as_text().is_some_and(|v| v == recipient.to_string());
    if !matches {
        return Err(invalid("key bundle is for another recipient"));
    }
    if user.child("error").is_some() {
        return Err(invalid("server refused recipient key lookup"));
    }
    if data(child(user, "type")?)? != [5] {
        return Err(invalid("unsupported identity key type"));
    }
    let identity = IdentityKey::new(public(child(user, "identity")?)?);
    let signed = child(user, "skey")?;
    let signature = data(child(signed, "signature")?)?.to_vec();
    if signature.len() != 64 {
        return Err(invalid("invalid signed-key signature width"));
    }
    let prekey = user
        .child("key")
        .map(|key| -> Result<_> {
            Ok((
                number(child(key, "id")?, 3)?.into(),
                public(child(key, "value")?)?,
            ))
        })
        .transpose()?;
    PreKeyBundle::new(
        number(child(user, "registration")?, 4)?,
        DeviceId::new(recipient.device.into()),
        prekey,
        number(child(signed, "id")?, 3)?.into(),
        public(child(signed, "value")?)?,
        signature,
        identity,
    )
    .map_err(crypto)
}

struct Identities {
    local: IdentityKeyPair,
    registration: u32,
    peer: Option<IdentityKey>,
    address: ProtocolAddress,
}
#[async_trait]
impl IdentityKeyStore for Identities {
    async fn get_identity_key_pair(&self) -> signal::error::Result<IdentityKeyPair> {
        Ok(self.local.clone())
    }
    async fn get_local_registration_id(&self) -> signal::error::Result<u32> {
        Ok(self.registration)
    }
    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> signal::error::Result<IdentityChange> {
        if address != &self.address || self.peer.as_ref().is_some_and(|v| v != identity) {
            return Err(SignalProtocolError::UntrustedIdentity(address.clone()));
        }
        self.peer = Some(*identity);
        Ok(IdentityChange::NewOrUnchanged)
    }
    async fn is_trusted_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        _: Direction,
    ) -> signal::error::Result<bool> {
        Ok(address == &self.address && self.peer.as_ref().is_none_or(|v| v == identity))
    }
    async fn get_identity(
        &self,
        address: &ProtocolAddress,
    ) -> signal::error::Result<Option<IdentityKey>> {
        if address != &self.address {
            return Err(SignalProtocolError::UntrustedIdentity(address.clone()));
        }
        Ok(self.peer)
    }
}
struct Sessions {
    record: Option<SessionRecord>,
    address: ProtocolAddress,
}
#[async_trait]
impl SessionStore for Sessions {
    async fn load_session(
        &self,
        address: &ProtocolAddress,
    ) -> signal::error::Result<Option<SessionRecord>> {
        if address != &self.address {
            return Err(SignalProtocolError::SessionNotFound(address.clone()));
        }
        Ok(self.record.clone())
    }
    async fn has_session(&self, address: &ProtocolAddress) -> signal::error::Result<bool> {
        Ok(address == &self.address && self.record.is_some())
    }
    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: SessionRecord,
    ) -> signal::error::Result<()> {
        if address != &self.address {
            return Err(SignalProtocolError::SessionNotFound(address.clone()));
        }
        self.record = Some(record);
        Ok(())
    }
}

/// Prepared encryption plus updated durable session and peer identity.
pub struct PreparedDirect {
    /// Opaque encrypted Signal message, not plaintext.
    pub ciphertext: Vec<u8>,
    /// `pkmsg` for first use, `msg` for an established session.
    pub kind: &'static str,
    /// Secret session serialization, must commit before transmission.
    pub session: Zeroizing<Vec<u8>>,
    /// Public peer identity, pin on first use and refuse changes.
    pub identity: Vec<u8>,
}
impl std::fmt::Debug for PreparedDirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedDirect")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

/// Prepare one text message without mutating durable state or doing network I/O.
/// Only a new session accepts a bundle; an existing session is never reset.
pub async fn prepare(
    account: &AccountRecord,
    recipient: &Jid,
    bundle: &PreKeyBundle,
    previous_session: Option<&[u8]>,
    previous_identity: Option<&[u8]>,
    text: &str,
) -> Result<PreparedDirect> {
    if text.is_empty() || text.len() > 4096 || !account.is_registered() {
        return Err(invalid("invalid message/account"));
    }
    let key = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("missing saved Signal identity"))?;
    let public = PublicKey::from_djb_public_key_bytes(&key.0.public)
        .map_err(|_| invalid("invalid local public key"))?;
    let private = PrivateKey::deserialize(&key.0.private)
        .map_err(|_| invalid("invalid local private key"))?;
    let local = IdentityKeyPair::new(IdentityKey::new(public), private);
    let address = ProtocolAddress::new(&recipient.user, DeviceId::new(recipient.device.into()));
    let mut identities = Identities {
        local,
        registration: account
            .signal_registration_id
            .ok_or_else(|| invalid("missing local registration id"))?,
        peer: previous_identity
            .map(IdentityKey::decode)
            .transpose()
            .map_err(crypto)?,
        address: address.clone(),
    };
    let mut sessions = Sessions {
        record: previous_session
            .map(SessionRecord::deserialize)
            .transpose()
            .map_err(crypto)?,
        address: address.clone(),
    };
    if sessions.record.is_none() {
        process_prekey_bundle(
            &address,
            &mut sessions,
            &mut identities,
            bundle,
            &mut rand::rng(),
            UsePQRatchet::No,
        )
        .await
        .map_err(crypto)?;
    }
    #[derive(prost::Message)]
    struct TextMessage {
        #[prost(string, tag = "1")]
        conversation: String,
    }
    let mut plain = Zeroizing::new(
        TextMessage {
            conversation: text.into(),
        }
        .encode_to_vec(),
    );
    // LX/Cb1.A00: random count 1..16, repeated count byte appended.
    let mut random = [0; 1];
    getrandom::fill(&mut random).map_err(|_| invalid("padding entropy unavailable"))?;
    let padding = (random[0] & 15) + 1;
    let padded_len = plain.len() + padding as usize;
    plain.resize(padded_len, padding);
    let encrypted = message_encrypt(&plain, &address, &mut sessions, &mut identities)
        .await
        .map_err(crypto)?;
    let kind = match encrypted.message_type() {
        CiphertextMessageType::PreKey => "pkmsg",
        CiphertextMessageType::Whisper => "msg",
        _ => return Err(invalid("unsupported ciphertext type")),
    };
    Ok(PreparedDirect {
        ciphertext: encrypted.serialize().to_vec(),
        kind,
        session: Zeroizing::new(
            sessions
                .record
                .ok_or_else(|| invalid("missing updated session"))?
                .serialize()
                .map_err(crypto)?,
        ),
        identity: identities
            .peer
            .ok_or_else(|| invalid("missing pinned peer identity"))?
            .serialize()
            .to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{KeyMaterial, RegistrationStatus, SignalIdentityKey};
    use signal::{
        GenericSignedPreKey, KeyPair, PreKeyId, PreKeyRecord, PreKeySignalMessage, PreKeyStore,
        SignedPreKeyId, SignedPreKeyRecord, SignedPreKeyStore, message_decrypt_prekey,
    };

    struct Keys {
        one: PreKeyRecord,
        signed: SignedPreKeyRecord,
    }
    #[async_trait]
    impl PreKeyStore for Keys {
        async fn get_pre_key(&self, id: PreKeyId) -> signal::error::Result<PreKeyRecord> {
            assert_eq!(u32::from(id), 8);
            Ok(self.one.clone())
        }
        async fn save_pre_key(
            &mut self,
            _: PreKeyId,
            record: &PreKeyRecord,
        ) -> signal::error::Result<()> {
            self.one = record.clone();
            Ok(())
        }
        async fn remove_pre_key(&mut self, _: PreKeyId) -> signal::error::Result<()> {
            Ok(())
        }
    }
    #[async_trait]
    impl SignedPreKeyStore for Keys {
        async fn get_signed_pre_key(
            &self,
            id: SignedPreKeyId,
        ) -> signal::error::Result<SignedPreKeyRecord> {
            assert_eq!(u32::from(id), 7);
            Ok(self.signed.clone())
        }
        async fn save_signed_pre_key(
            &mut self,
            _: SignedPreKeyId,
            record: &SignedPreKeyRecord,
        ) -> signal::error::Result<()> {
            self.signed = record.clone();
            Ok(())
        }
    }

    #[tokio::test]
    async fn generated_bundle_encrypts_decrypts_and_refuses_changed_identity() {
        let mut rng = rand::rng();
        let alice = IdentityKeyPair::generate(&mut rng);
        let bob = IdentityKeyPair::generate(&mut rng);
        let signed = KeyPair::generate(&mut rng);
        let one = KeyPair::generate(&mut rng);
        let signature = bob
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            Some((8.into(), one.public_key)),
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *bob.identity_key(),
        )
        .unwrap();
        let account = AccountRecord {
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: alice.public_key().public_key_bytes().try_into().unwrap(),
                private: *alice.private_key().serialize(),
            })),
            ..Default::default()
        };
        let recipient = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let encrypted = prepare(&account, &recipient, &bundle, None, None, "test")
            .await
            .unwrap();
        assert_eq!(encrypted.kind, "pkmsg");
        let bad_bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            Some((8.into(), one.public_key)),
            7.into(),
            signed.public_key,
            vec![0; 64],
            *bob.identity_key(),
        )
        .unwrap();
        assert!(
            prepare(&account, &recipient, &bad_bundle, None, None, "test")
                .await
                .is_err()
        );
        let address = ProtocolAddress::new("offline-alice", DeviceId::new(0));
        let mut identities = Identities {
            local: bob,
            registration: 42,
            peer: None,
            address: address.clone(),
        };
        let mut sessions = Sessions {
            record: None,
            address: address.clone(),
        };
        let signed_record = SignedPreKeyRecord::new(
            7.into(),
            signal::Timestamp::from_epoch_millis(0),
            &signed,
            &signature,
        );
        let mut keys = Keys {
            one: PreKeyRecord::new(8.into(), &one),
            signed: signed_record.clone(),
        };
        let signed_keys = Keys {
            one: keys.one.clone(),
            signed: signed_record,
        };
        let message = PreKeySignalMessage::try_from(encrypted.ciphertext.as_slice()).unwrap();
        let decrypted = message_decrypt_prekey(
            &message,
            &address,
            &mut sessions,
            &mut identities,
            &mut keys,
            &signed_keys,
            &mut rng,
            UsePQRatchet::No,
        )
        .await
        .unwrap();
        assert_eq!(&decrypted.plaintext[..6], b"\x0a\x04test");
        let padding = *decrypted.plaintext.last().unwrap() as usize;
        assert!((1..=16).contains(&padding));
        assert_eq!(decrypted.plaintext.len(), 6 + padding);
        assert!(
            decrypted.plaintext[6..]
                .iter()
                .all(|v| *v as usize == padding)
        );
        let wrong = IdentityKeyPair::generate(&mut rng)
            .identity_key()
            .serialize();
        assert!(
            prepare(&account, &recipient, &bundle, None, Some(&wrong), "test")
                .await
                .is_err()
        );
        let resumed = prepare(
            &account,
            &recipient,
            &bundle,
            Some(&encrypted.session),
            Some(&encrypted.identity),
            "second",
        )
        .await
        .unwrap();
        assert_ne!(encrypted.ciphertext, resumed.ciphertext);
        assert_ne!(*encrypted.session, *resumed.session);
    }
}
