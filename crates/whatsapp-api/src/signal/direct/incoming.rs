//! Incoming prekey sessions. All mutations stay in memory until inbox commit.
use super::*;
use signal::{
    GenericSignedPreKey, PreKeyId, PreKeyRecord, PreKeySignalMessage, PreKeyStore, SignedPreKeyId,
    SignedPreKeyRecord, SignedPreKeyStore,
};

struct OneTime {
    record: Option<PreKeyRecord>,
}
#[async_trait]
impl PreKeyStore for OneTime {
    async fn get_pre_key(&self, id: PreKeyId) -> signal::error::Result<PreKeyRecord> {
        self.record
            .as_ref()
            .filter(|r| r.id().ok() == Some(id))
            .cloned()
            .ok_or(SignalProtocolError::InvalidPreKeyId)
    }
    async fn save_pre_key(&mut self, _: PreKeyId, _: &PreKeyRecord) -> signal::error::Result<()> {
        Err(SignalProtocolError::InvalidPreKeyId)
    }
    async fn remove_pre_key(&mut self, _: PreKeyId) -> signal::error::Result<()> {
        Err(SignalProtocolError::InvalidPreKeyId)
    }
}
struct Signed(SignedPreKeyRecord);
#[async_trait]
impl SignedPreKeyStore for Signed {
    async fn get_signed_pre_key(
        &self,
        id: SignedPreKeyId,
    ) -> signal::error::Result<SignedPreKeyRecord> {
        if self.0.id()? != id {
            return Err(SignalProtocolError::InvalidSignedPreKeyId);
        }
        Ok(self.0.clone())
    }
    async fn save_signed_pre_key(
        &mut self,
        _: SignedPreKeyId,
        _: &SignedPreKeyRecord,
    ) -> signal::error::Result<()> {
        Err(SignalProtocolError::InvalidSignedPreKeyId)
    }
}
/// Optional one-time prekey ID in a bounded incoming Signal envelope.
pub fn incoming_prekey_id(ciphertext: &[u8]) -> Result<Option<u32>> {
    if ciphertext.len() > 65536 {
        return Err(invalid("prekey envelope exceeds bound"));
    }
    Ok(PreKeySignalMessage::try_from(ciphertext)
        .map_err(crypto)?
        .pre_key_id()
        .map(u32::from))
}
/// Compare an incoming prekey's public identity to a saved pin without
/// decrypting, advancing a session or exposing either key. A match is not
/// authentication of the message; it is only an offline inspection result.
pub fn incoming_prekey_identity_matches(ciphertext: &[u8], pinned: &[u8]) -> Result<bool> {
    if ciphertext.len() > 65536 {
        return Err(invalid("prekey envelope exceeds bound"));
    }
    let message = PreKeySignalMessage::try_from(ciphertext).map_err(crypto)?;
    let pinned = IdentityKey::decode(pinned).map_err(crypto)?;
    Ok(message.identity_key() == &pinned)
}
/// SHA-256 diagnostics for the saved and observed public identities. These
/// are local key fingerprints, not WhatsApp safety numbers or authentication.
pub fn incoming_prekey_identity_fingerprints(
    ciphertext: &[u8],
    pinned: &[u8],
) -> Result<(String, String)> {
    use sha2::{Digest, Sha256};
    if ciphertext.len() > 65536 {
        return Err(invalid("prekey envelope exceeds bound"));
    }
    let message = PreKeySignalMessage::try_from(ciphertext).map_err(crypto)?;
    let pinned = IdentityKey::decode(pinned).map_err(crypto)?;
    Ok((
        hex::encode(Sha256::digest(pinned.serialize())),
        hex::encode(Sha256::digest(message.identity_key().serialize())),
    ))
}
/// Prepared incoming session and peer identity, plus the consumed prekey ID.
pub struct ReceivedPrekey {
    /// Authenticated plaintext and updated session; commit together.
    pub received: ReceivedDirect,
    /// First-use identity to pin; changed existing identities are refused.
    pub identity: Vec<u8>,
    /// One-time key to delete in the same transaction, when actually consumed.
    pub consumed_prekey: Option<u32>,
}
impl std::fmt::Debug for ReceivedPrekey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReceivedPrekey(<redacted>)")
    }
}
/// Establish/advance an incoming session with the actual registered signed key.
/// TOFU is allowed only for absent peer identity; changed pins are never replaced.
pub async fn receive_prekey(
    account: &AccountRecord,
    sender: &Jid,
    previous: (Option<&[u8]>, Option<&[u8]>),
    signed: &[u8],
    one_time: Option<&[u8]>,
    ciphertext: &[u8],
) -> Result<ReceivedPrekey> {
    if !account.is_registered() || ciphertext.len() > 65536 {
        return Err(invalid("invalid incoming prekey account or size"));
    }
    let key = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("missing local identity"))?;
    let public = PublicKey::from_djb_public_key_bytes(&key.0.public)
        .map_err(|_| invalid("invalid local public identity"))?;
    let private = PrivateKey::deserialize(&key.0.private)
        .map_err(|_| invalid("invalid local private identity"))?;
    let address = ProtocolAddress::new(&sender.user, DeviceId::new(sender.device.into()));
    let mut identities = Identities {
        local: IdentityKeyPair::new(IdentityKey::new(public), private),
        registration: account
            .signal_registration_id
            .ok_or_else(|| invalid("missing registration id"))?,
        peer: previous
            .1
            .map(IdentityKey::decode)
            .transpose()
            .map_err(crypto)?,
        address: address.clone(),
    };
    let mut sessions = Sessions {
        record: previous
            .0
            .map(SessionRecord::deserialize)
            .transpose()
            .map_err(crypto)?,
        address: address.clone(),
    };
    let mut one = OneTime {
        record: one_time
            .map(PreKeyRecord::deserialize)
            .transpose()
            .map_err(crypto)?,
    };
    let signed = Signed(SignedPreKeyRecord::deserialize(signed).map_err(crypto)?);
    let message = PreKeySignalMessage::try_from(ciphertext).map_err(crypto)?;
    let decrypted = signal::message_decrypt_prekey(
        &message,
        &address,
        &mut sessions,
        &mut identities,
        &mut one,
        &signed,
        &mut task_rng(),
        UsePQRatchet::No,
    )
    .await
    .map_err(crypto)?;
    let consumed_prekey = decrypted.consumed_prekey_id.map(u32::from);
    let mut plaintext = Zeroizing::new(decrypted.plaintext);
    let padding = plaintext.last().copied().unwrap_or(0) as usize;
    if !(1..=16).contains(&padding)
        || padding > plaintext.len()
        || !plaintext[plaintext.len() - padding..]
            .iter()
            .all(|b| *b as usize == padding)
    {
        return Err(invalid("invalid authenticated padding"));
    }
    let length = plaintext.len() - padding;
    plaintext.truncate(length);
    Ok(ReceivedPrekey {
        received: ReceivedDirect {
            plaintext,
            session: Zeroizing::new(
                sessions
                    .record
                    .ok_or_else(|| invalid("missing incoming session"))?
                    .serialize()
                    .map_err(crypto)?,
            ),
        },
        identity: identities
            .peer
            .ok_or_else(|| invalid("missing authenticated peer identity"))?
            .serialize()
            .to_vec(),
        consumed_prekey,
    })
}
