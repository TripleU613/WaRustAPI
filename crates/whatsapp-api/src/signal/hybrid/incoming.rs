//! Staged native hybrid receive; local key deletion belongs to the inbox transaction.
use super::{initialize_bob, invalid};
use crate::{
    Error, Result,
    model::jid::{Jid, Server},
    registration::request::IdentityChangePolicy,
    session::AccountRecord,
    signal::{direct, kem},
};
use wacore_libsignal::protocol::{
    BobSignalProtocolParameters, GenericSignedPreKey, IdentityKey, IdentityKeyPair, KeyPair,
    PreKeyRecord, PreKeySignalMessage, PrivateKey, PublicKey, SessionRecord, SignedPreKeyRecord,
    UsePQRatchet,
};

/// Borrowed local inventory material, not material supplied by the sender.
pub struct LocalKemPrekey<'a> {
    /// Exact inventory ID selected by the outer KEM field.
    pub id: u32,
    /// Expanded ML-KEM-1024 secret; keep encrypted at rest and out of logs.
    pub secret: &'a [u8],
    /// Native last-resort keys survive successful bootstrap.
    pub last_resort: bool,
}
impl std::fmt::Debug for LocalKemPrekey<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LocalKemPrekey(<redacted>)")
    }
}
/// Authenticated candidate and key consumption to commit in one transaction.
pub struct ReceivedHybridPrekey {
    /// Plaintext, advanced session, authenticated identity and ordinary-key use.
    pub incoming: direct::ReceivedPrekey,
    /// Delete this non-last-resort key only with the successful inbox commit.
    pub consumed_kem: Option<u32>,
}
impl std::fmt::Debug for ReceivedHybridPrekey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReceivedHybridPrekey(<redacted>)")
    }
}
fn pair_matches(pair: &KeyPair) -> Result<()> {
    if pair
        .private_key
        .public_key()
        .map_err(|_| invalid("invalid private key"))?
        != pair.public_key
    {
        return Err(invalid("local key pair mismatch"));
    }
    Ok(())
}

/// Authenticate an incoming version-4 KEM prekey without durable mutation.
/// Matching held/archived sessions need no local prekeys. New bootstrap requires
/// exact local signed/ordinary/KEM IDs; report consumption only after MAC/padding.
/// Own-account/companion authority and atomic archival are caller responsibilities.
pub async fn receive_prekey(
    account: &AccountRecord,
    sender: &Jid,
    previous: (Option<&[u8]>, Option<&[u8]>, IdentityChangePolicy),
    signed: Option<&[u8]>,
    one_time: Option<&[u8]>,
    local_kem: Option<LocalKemPrekey<'_>>,
    ciphertext: &[u8],
) -> Result<ReceivedHybridPrekey> {
    if !account.is_registered()
        || ciphertext.len() > 65536
        || previous.0.is_some_and(|bytes| bytes.len() > 1024 * 1024)
        || sender.agent != 0
        || !matches!(sender.server, Server::User | Server::Lid)
        || sender.user.is_empty()
    {
        return Err(invalid("invalid receive account, sender or size"));
    }
    let message = PreKeySignalMessage::try_from(ciphertext)
        .map_err(|_| invalid("invalid prekey envelope"))?;
    let incoming_kem =
        direct::incoming_prekey_kem(ciphertext)?.ok_or_else(|| invalid("missing KEM metadata"))?;
    if message.message_version() != 4 || message.message().message_version() != 4 {
        return Err(invalid("expected version 4"));
    }
    let key = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("missing local identity"))?;
    let public = PublicKey::from_djb_public_key_bytes(&key.0.public)
        .map_err(|_| invalid("invalid local identity"))?;
    let private =
        PrivateKey::deserialize(&key.0.private).map_err(|_| invalid("invalid local identity"))?;
    let local = IdentityKeyPair::new(IdentityKey::new(public), private);
    if local
        .private_key()
        .public_key()
        .map_err(|_| invalid("invalid local identity"))?
        != *local.public_key()
    {
        return Err(invalid("local identity pair mismatch"));
    }
    let registration = account
        .signal_registration_id
        .ok_or_else(|| invalid("missing registration"))?;
    let pin = previous
        .1
        .map(IdentityKey::decode)
        .transpose()
        .map_err(|_| invalid("invalid peer pin"))?;
    let changed = pin
        .as_ref()
        .is_some_and(|pin| pin != message.identity_key());
    if changed && previous.2 == IdentityChangePolicy::Pinned {
        return Err(Error::IdentityChanged(
            crate::model::ids::DeviceAddress::from_jid(sender)?,
        ));
    }
    let mut record = previous
        .0
        .map(SessionRecord::deserialize)
        .transpose()
        .map_err(|_| invalid("invalid held record"))?
        .unwrap_or_else(SessionRecord::new_fresh);
    if let Some(state) = record.session_state()
        && (state
            .local_identity_key()
            .map_err(|_| invalid("invalid held local identity"))?
            != *local.identity_key()
            || state.local_registration_id() != registration
            || pin.is_none()
            || state
                .remote_identity_key()
                .map_err(|_| invalid("invalid held remote identity"))?
                .as_ref()
                != pin.as_ref())
    {
        return Err(invalid("held account/pin mismatch"));
    }
    let matched = record
        .promote_matching_session(4, &message.base_key().serialize())
        .map_err(|_| invalid("invalid held session"))?;
    let (consumed_prekey, consumed_kem) = if matched {
        // shortcut: cross-identity archived promotion is unported; never roll back a current pin from a delayed envelope.
        if changed || previous.1.is_none() {
            return Err(invalid("matching hybrid session requires unchanged pin"));
        }
        (None, None)
    } else {
        let signed =
            SignedPreKeyRecord::deserialize(signed.ok_or_else(|| invalid("missing signed key"))?)
                .map_err(|_| invalid("invalid signed key"))?;
        if signed.id().map_err(|_| invalid("invalid signed id"))? != message.signed_pre_key_id() {
            return Err(invalid("signed key id mismatch"));
        }
        let signed_pair = signed
            .key_pair()
            .map_err(|_| invalid("invalid signed pair"))?;
        pair_matches(&signed_pair)?;
        if !local.public_key().verify_signature(
            &signed_pair.public_key.serialize(),
            &signed
                .signature()
                .map_err(|_| invalid("invalid signed signature"))?,
        ) {
            return Err(invalid("signed key signature invalid"));
        }
        let one = if let Some(id) = message.pre_key_id() {
            let one = PreKeyRecord::deserialize(
                one_time.ok_or_else(|| invalid("missing ordinary prekey"))?,
            )
            .map_err(|_| invalid("invalid ordinary prekey"))?;
            if one.id().map_err(|_| invalid("invalid ordinary id"))? != id {
                return Err(invalid("ordinary prekey id mismatch"));
            }
            let pair = one
                .key_pair()
                .map_err(|_| invalid("invalid ordinary pair"))?;
            pair_matches(&pair)?;
            Some(pair)
        } else {
            None
        };
        let kem = local_kem.ok_or_else(|| invalid("missing local KEM prekey"))?;
        if kem.id != incoming_kem.prekey_id {
            return Err(invalid("KEM prekey id mismatch"));
        }
        let shared = kem::decapsulate(kem.secret, incoming_kem.ciphertext)?;
        let parameters = BobSignalProtocolParameters::new(
            local.clone(),
            signed_pair.clone(),
            one,
            signed_pair,
            *message.identity_key(),
            *message.base_key(),
            UsePQRatchet::No,
        );
        let fresh = initialize_bob(
            &parameters,
            shared.as_bytes().try_into().expect("32-byte KEM secret"),
            (registration, message.registration_id()),
        )?;
        let state = fresh
            .0
            .session_state()
            .expect("fresh current session")
            .clone();
        if changed {
            record = SessionRecord::new_fresh();
        }
        record.promote_fresh_state(state);
        (
            message
                .pre_key_id()
                .map(u32::from)
                .filter(|id| *id != 0xffffff),
            (!kem.last_resort).then_some(kem.id),
        )
    };
    let state = record
        .session_state()
        .ok_or_else(|| invalid("missing candidate state"))?;
    if state
        .local_identity_key()
        .map_err(|_| invalid("invalid local state identity"))?
        != *local.identity_key()
        || state.local_registration_id() != registration
        || state
            .remote_identity_key()
            .map_err(|_| invalid("invalid remote state identity"))?
            .as_ref()
            != Some(message.identity_key())
    {
        return Err(invalid("candidate account/identity mismatch"));
    }
    let candidate = zeroize::Zeroizing::new(
        record
            .serialize()
            .map_err(|_| invalid("record serialization failed"))?,
    );
    let received = direct::receive(
        account,
        sender,
        &candidate,
        &message.identity_key().serialize(),
        message.message().serialized(),
    )
    .await?;
    Ok(ReceivedHybridPrekey {
        incoming: direct::ReceivedPrekey {
            received,
            identity: message.identity_key().serialize().to_vec(),
            consumed_prekey,
        },
        consumed_kem,
    })
}
