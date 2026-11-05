//! Incoming prekey sessions. All mutations stay in memory until inbox commit.
use super::*;
use signal::{
    GenericSignedPreKey, PreKeyId, PreKeyRecord, PreKeySignalMessage, PreKeyStore, SignedPreKeyId,
    SignedPreKeyRecord, SignedPreKeyStore,
};

// F22.A04 first reuses a matching current/previous session. Load/decode
// local key material only when the backend actually requests that key.
struct OneTime<'a> {
    record: Option<&'a [u8]>,
}
#[async_trait]
impl PreKeyStore for OneTime<'_> {
    async fn get_pre_key(&self, id: PreKeyId) -> signal::error::Result<PreKeyRecord> {
        let bytes = self.record.ok_or(SignalProtocolError::InvalidPreKeyId)?;
        let record =
            PreKeyRecord::deserialize(bytes).map_err(|_| SignalProtocolError::InvalidPreKeyId)?;
        if record.id().ok() != Some(id) {
            return Err(SignalProtocolError::InvalidPreKeyId);
        }
        Ok(record)
    }
    async fn save_pre_key(&mut self, _: PreKeyId, _: &PreKeyRecord) -> signal::error::Result<()> {
        Err(SignalProtocolError::InvalidPreKeyId)
    }
    async fn remove_pre_key(&mut self, _: PreKeyId) -> signal::error::Result<()> {
        Err(SignalProtocolError::InvalidPreKeyId)
    }
}
struct Signed<'a>(&'a [u8]);
#[async_trait]
impl SignedPreKeyStore for Signed<'_> {
    async fn get_signed_pre_key(
        &self,
        id: SignedPreKeyId,
    ) -> signal::error::Result<SignedPreKeyRecord> {
        let record = SignedPreKeyRecord::deserialize(self.0)
            .map_err(|_| SignalProtocolError::InvalidSignedPreKeyId)?;
        if record.id().ok() != Some(id) {
            return Err(SignalProtocolError::InvalidSignedPreKeyId);
        }
        Ok(record)
    }
    async fn save_signed_pre_key(
        &mut self,
        _: SignedPreKeyId,
        _: &SignedPreKeyRecord,
    ) -> signal::error::Result<()> {
        Err(SignalProtocolError::InvalidSignedPreKeyId)
    }
}
/// Borrowed KEM metadata from an incoming prekey envelope, not authentication.
pub struct IncomingKem<'a> {
    /// Native uint32 prekey ID, retained even when zero.
    pub prekey_id: u32,
    /// Original ciphertext bytes; empty presence is not a valid KEM ciphertext.
    pub ciphertext: &'a [u8],
}
impl std::fmt::Debug for IncomingKem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IncomingKem(<redacted>)")
    }
}
fn parse_prekey(ciphertext: &[u8]) -> Result<(PreKeySignalMessage, Option<IncomingKem<'_>>)> {
    if ciphertext.len() > 65536 {
        return Err(invalid("prekey envelope exceeds bound"));
    }
    let message = PreKeySignalMessage::try_from(ciphertext).map_err(crypto)?;
    let kem = crate::transport::protobuf_enum::prekey_kem(&ciphertext[1..])
        .map_err(|_| invalid("incomplete or malformed KEM prekey metadata"))?
        .map(|(prekey_id, ciphertext)| IncomingKem {
            prekey_id,
            ciphertext,
        });
    Ok((message, kem))
}
/// Inspect paired KEM fields without decrypting or modifying saved state.
/// This validates envelope structure, not signature, trust or KEM byte encoding.
pub fn incoming_prekey_kem(ciphertext: &[u8]) -> Result<Option<IncomingKem<'_>>> {
    Ok(parse_prekey(ciphertext)?.1)
}
/// Optional one-time prekey ID in a bounded incoming Signal envelope.
pub fn incoming_prekey_id(ciphertext: &[u8]) -> Result<Option<u32>> {
    Ok(parse_prekey(ciphertext)?.0.pre_key_id().map(u32::from))
}
/// Compare an incoming prekey's public identity to a saved pin without
/// decrypting, advancing a session or exposing either key. A match is not
/// authentication of the message; it is only an offline inspection result.
pub fn incoming_prekey_identity_matches(ciphertext: &[u8], pinned: &[u8]) -> Result<bool> {
    let (message, _) = parse_prekey(ciphertext)?;
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
    let (message, _) = parse_prekey(ciphertext)?;
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
    /// Authenticated identity to pin under the selected policy. Replacement
    /// requires an atomic archive and event with the session/inbox commit.
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
    receive_prekey_with_policy(
        account,
        sender,
        (
            previous.0,
            previous.1,
            crate::registration::request::IdentityChangePolicy::Pinned,
        ),
        signed,
        one_time,
        ciphertext,
    )
    .await
}

/// Prepare an authenticated incoming message under an explicit remote identity
/// policy. Changes remain in memory; the caller must compare saved state and
/// atomically archive the previous pin/session when committing a replacement.
pub async fn receive_prekey_with_policy(
    account: &AccountRecord,
    sender: &Jid,
    previous: (
        Option<&[u8]>,
        Option<&[u8]>,
        crate::registration::request::IdentityChangePolicy,
    ),
    signed: &[u8],
    one_time: Option<&[u8]>,
    ciphertext: &[u8],
) -> Result<ReceivedPrekey> {
    if !account.is_registered() || ciphertext.len() > 65536 {
        return Err(invalid("invalid incoming prekey account or size"));
    }
    let (message, kem) = parse_prekey(ciphertext)?;
    if kem.is_some() {
        return Err(invalid("post-quantum session processing is unsupported"));
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
    let saved_identity = previous
        .1
        .map(IdentityKey::decode)
        .transpose()
        .map_err(crypto)?;
    // APK 2.26.37.71 0km.BRS trusts the supplied peer identity and COt saves
    // it through the coordinator. Limit that behavior to this bounded candidate:
    // no durable key/session mutation happens before nested MAC and padding pass.
    let candidate_identity = match previous.2 {
        crate::registration::request::IdentityChangePolicy::Pinned => saved_identity,
        crate::registration::request::IdentityChangePolicy::ApkAuthenticated => {
            Some(*message.identity_key())
        }
    };
    let mut identities = Identities {
        local: IdentityKeyPair::new(IdentityKey::new(public), private),
        registration: account
            .signal_registration_id
            .ok_or_else(|| invalid("missing registration id"))?,
        peer: candidate_identity,
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
    let mut one = OneTime { record: one_time };
    let signed = Signed(signed);
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
    .map_err(|error| match error {
        // APK43.72 F22.A04 missing/invalid local ordinary or signed key ->
        // EDR; 0WJ.A0O -1003 -> internal3 -> wire3. Classic receive only.
        SignalProtocolError::InvalidPreKeyId | SignalProtocolError::InvalidSignedPreKeyId => {
            Error::SignalIncomingLocalPrekey
        }
        other => crypto(other),
    })?;
    let consumed_prekey = decrypted.consumed_prekey_id.map(u32::from);
    let plaintext = crate::signal::padding::unpad(decrypted.plaintext)?;
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
