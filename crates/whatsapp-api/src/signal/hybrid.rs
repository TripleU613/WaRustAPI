//! Staged hybrid sessions; no directory admission, account-store mutation or network I/O.
use super::{
    direct::{IncomingKem, PreparedDirect},
    kem, wire,
};
use crate::{Error, Result};
mod bundle;
mod incoming;
#[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
pub(crate) use bundle::SIGNATURE_FIELD;
pub use bundle::{HybridBundle, bundle_for};
pub use incoming::{LocalKemPrekey, ReceivedHybridPrekey, receive_prekey};
use wacore_libsignal::{
    crypto::aes_256_cbc_encrypt_into,
    protocol::{
        AliceSignalProtocolParameters, BobSignalProtocolParameters, ChainKey, KeyPair, PrivateKey,
        PublicKey, RootKey, SessionRecord, SessionState, UsePQRatchet,
    },
};
use zeroize::Zeroizing;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("hybrid_session", reason)
}
fn agreement(private: &PrivateKey, public: &PublicKey) -> Result<Zeroizing<[u8; 32]>> {
    private
        .calculate_agreement(public)
        .map(Zeroizing::new)
        .map_err(|_| invalid("invalid agreement"))
}
fn keys(
    a: &[u8; 32],
    b: &[u8; 32],
    c: &[u8; 32],
    one: Option<&[u8; 32]>,
    kem: &[u8; 32],
) -> (RootKey, ChainKey) {
    let (root, chain) = kem::initial_hybrid_keys(a, b, c, one, kem);
    (
        RootKey::new(root.as_bytes().try_into().expect("32-byte root")),
        ChainKey::new(chain.as_bytes().try_into().expect("32-byte chain"), 0),
    )
}

/// A staged canonical version-4 record, with secret-bearing Debug redacted.
/// This does not establish peer trust or authorize replacing a held session.
pub struct HybridSession(SessionRecord);
impl std::fmt::Debug for HybridSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HybridSession(<redacted>)")
    }
}
impl HybridSession {
    /// Reopen a trusted canonical record. This is not an APK database importer.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 1024 * 1024 {
            return Err(invalid("record exceeds bound"));
        }
        let record = SessionRecord::deserialize(bytes).map_err(|_| invalid("invalid record"))?;
        Self::from_record(record)
    }
    pub(crate) fn from_record(record: SessionRecord) -> Result<Self> {
        let state = record
            .session_state()
            .ok_or_else(|| invalid("missing current session"))?;
        if state
            .session_version()
            .map_err(|_| invalid("invalid version"))?
            != 4
        {
            return Err(invalid("expected version 4"));
        }
        Ok(Self(record))
    }
    /// Export secret-bearing canonical bytes for encrypted, atomic storage only.
    pub fn to_bytes(&self) -> Result<Zeroizing<Vec<u8>>> {
        self.0
            .serialize()
            .map(Zeroizing::new)
            .map_err(|_| invalid("record serialization failed"))
    }
    /// Prepare padded protobuf ciphertext and an updated candidate record.
    /// Commit the returned record atomically before sending or preparing another
    /// payload from this session. The source record remains unchanged, even on error.
    /// Trust/identity policy belongs to the account coordinator, not this helper.
    pub fn prepare(&self, payload: &[u8]) -> Result<PreparedDirect> {
        if payload.len() > 65536 {
            return Err(invalid("payload exceeds bound"));
        }
        let mut candidate = self.0.clone();
        let pending_components = candidate
            .clone()
            .into_components()
            .map_err(|_| invalid("record projection failed"))?
            .current_session
            .and_then(|state| state.pending_pre_key);
        let state = candidate
            .session_state_mut()
            .ok_or_else(|| invalid("missing current session"))?;
        if state
            .session_version()
            .map_err(|_| invalid("invalid version"))?
            != 4
        {
            return Err(invalid("expected version 4"));
        }
        let local = state
            .local_identity_key()
            .map_err(|_| invalid("missing local identity"))?;
        let remote = state
            .remote_identity_key()
            .map_err(|_| invalid("invalid remote identity"))?
            .ok_or_else(|| invalid("missing remote identity"))?;
        let chain = state
            .get_sender_chain_key()
            .map_err(|_| invalid("missing sender chain"))?;
        let (message_keys, next) = chain
            .step_with_message_keys()
            .map_err(|_| invalid("sender counter exhausted"))?;
        let message_keys = message_keys.generate_keys();
        let mut plain = Zeroizing::new(payload.to_vec());
        let mut entropy = [0u8; 1];
        getrandom::fill(&mut entropy).map_err(|_| invalid("padding entropy unavailable"))?;
        let padding = (entropy[0] & 15) + 1;
        let padded_length = plain.len() + usize::from(padding);
        plain.resize(padded_length, padding);
        let mut ciphertext = Vec::new();
        aes_256_cbc_encrypt_into(
            &plain,
            message_keys.cipher_key(),
            message_keys.iv(),
            &mut ciphertext,
        )
        .map_err(|_| invalid("encryption failed"))?;
        let nested = wire::encode_signal(
            4,
            message_keys.mac_key(),
            (&local, &remote),
            &state
                .sender_ratchet_key()
                .map_err(|_| invalid("missing sender ratchet"))?,
            (chain.index(), state.previous_counter()),
            &ciphertext,
        )?;
        let (ciphertext, kind) = if let Some(items) =
            state
                .unacknowledged_pre_key_message_items()
                .map_err(|_| invalid("invalid pending prekey"))?
        {
            let pending = pending_components
                .as_ref()
                .ok_or_else(|| invalid("missing pending prekey"))?;
            let kem_id = pending
                .kyber_pre_key_id
                .ok_or_else(|| invalid("missing pending KEM id"))?;
            let kem_ciphertext = pending
                .kyber_ciphertext
                .as_deref()
                .filter(|c| c.len() == 1568)
                .ok_or_else(|| invalid("invalid pending KEM ciphertext"))?;
            (
                wire::encode_prekey(
                    4,
                    state.local_registration_id(),
                    (
                        items.pre_key_id().map(u32::from),
                        u32::from(items.signed_pre_key_id()),
                    ),
                    (items.base_key(), &local),
                    &nested,
                    Some((kem_id, kem_ciphertext)),
                )?,
                "pkmsg",
            )
        } else {
            (nested, "msg")
        };
        state
            .set_sender_chain_key(&next)
            .map_err(|_| invalid("sender chain advance failed"))?;
        candidate.reserve_sender_chain_counters(chain.index());
        Ok(PreparedDirect {
            ciphertext,
            kind,
            session: Zeroizing::new(
                candidate
                    .serialize()
                    .map_err(|_| invalid("record serialization failed"))?,
            ),
            identity: remote.serialize().to_vec(),
        })
    }
}

/// Initialize an Alice candidate from already-authorized EC keys and a KEM secret.
/// Registration tuple is local/remote; pending tuple is ordinary ID/signed ID/KEM.
/// The KEM ciphertext must be the matching ML-KEM-1024 encapsulation output.
pub fn initialize_alice(
    parameters: &AliceSignalProtocolParameters,
    kem_shared: &[u8; 32],
    sender_ratchet: &KeyPair,
    registrations: (u32, u32),
    pending: (Option<u32>, u32, &IncomingKem<'_>),
) -> Result<HybridSession> {
    if !matches!(parameters.use_pq_ratchet(), UsePQRatchet::No)
        || parameters.their_ratchet_key() != parameters.their_signed_pre_key()
        || pending.2.ciphertext.len() != 1568
    {
        return Err(invalid("unsupported hybrid parameters"));
    }
    let identity = parameters.our_identity_key_pair();
    let base = parameters.our_base_key_pair();
    let a = agreement(identity.private_key(), parameters.their_signed_pre_key())?;
    let b = agreement(
        &base.private_key,
        parameters.their_identity_key().public_key(),
    )?;
    let c = agreement(&base.private_key, parameters.their_signed_pre_key())?;
    let one = parameters
        .their_one_time_pre_key()
        .map(|key| agreement(&base.private_key, key))
        .transpose()?;
    let (root, chain) = keys(&a, &b, &c, one.as_deref(), kem_shared);
    let (sender_root, sender_chain) = root
        .create_chain(parameters.their_ratchet_key(), &sender_ratchet.private_key)
        .map_err(|_| invalid("ratchet agreement failed"))?;
    let mut state = SessionState::new(
        4,
        identity.identity_key(),
        parameters.their_identity_key(),
        &sender_root,
        &base.public_key,
    )
    .with_receiver_chain(parameters.their_ratchet_key(), &chain)
    .with_sender_chain(sender_ratchet, &sender_chain);
    state.set_local_registration_id(registrations.0);
    state.set_remote_registration_id(registrations.1);
    state.set_unacknowledged_pre_key_message(
        pending.0.map(Into::into),
        pending.1.into(),
        &base.public_key,
    );
    let mut components = SessionRecord::new(state)
        .into_components()
        .map_err(|_| invalid("record projection failed"))?;
    let value = components
        .current_session
        .as_mut()
        .expect("fresh current session")
        .pending_pre_key
        .as_mut()
        .expect("pending prekey was set");
    value.kyber_pre_key_id = Some(pending.2.prekey_id);
    value.kyber_ciphertext = Some(pending.2.ciphertext.to_vec());
    SessionRecord::from_components(components)
        .map(HybridSession)
        .map_err(|_| invalid("record construction failed"))
}

/// Initialize a Bob candidate from already-authorized EC keys and a decapsulated KEM secret.
/// No local prekey is deleted or marked consumed; commit/deletion belongs to the coordinator.
pub fn initialize_bob(
    parameters: &BobSignalProtocolParameters,
    kem_shared: &[u8; 32],
    registrations: (u32, u32),
) -> Result<HybridSession> {
    if !matches!(parameters.use_pq_ratchet(), UsePQRatchet::No)
        || parameters.our_ratchet_key_pair().public_key
            != parameters.our_signed_pre_key_pair().public_key
    {
        return Err(invalid("unsupported hybrid parameters"));
    }
    let identity = parameters.our_identity_key_pair();
    let signed = parameters.our_signed_pre_key_pair();
    let a = agreement(
        &signed.private_key,
        parameters.their_identity_key().public_key(),
    )?;
    let b = agreement(identity.private_key(), parameters.their_base_key())?;
    let c = agreement(&signed.private_key, parameters.their_base_key())?;
    let one = parameters
        .our_one_time_pre_key_pair()
        .map(|key| agreement(&key.private_key, parameters.their_base_key()))
        .transpose()?;
    let (root, chain) = keys(&a, &b, &c, one.as_deref(), kem_shared);
    let mut state = SessionState::new(
        4,
        identity.identity_key(),
        parameters.their_identity_key(),
        &root,
        parameters.their_base_key(),
    )
    .with_sender_chain(parameters.our_ratchet_key_pair(), &chain);
    state.set_local_registration_id(registrations.0);
    state.set_remote_registration_id(registrations.1);
    Ok(HybridSession(SessionRecord::new(state)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::jid::Jid,
        session::{AccountRecord, KeyMaterial, RegistrationStatus, SignalIdentityKey},
        signal::{direct, kem},
    };
    use wacore_libsignal::protocol::{IdentityKeyPair, KeyPair, PreKeySignalMessage, UsePQRatchet};
    fn account(identity: &IdentityKeyPair, registration: u32) -> AccountRecord {
        AccountRecord {
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(registration),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: identity.public_key().public_key_bytes().try_into().unwrap(),
                private: *identity.private_key().serialize(),
            })),
            ..Default::default()
        }
    }
    #[tokio::test]
    async fn hybrid_bootstrap_reply_reopen_and_pending_keys_are_staged() {
        for one_time in [false, true] {
            let alice = IdentityKeyPair::generate(&mut rand::rng());
            let bob = IdentityKeyPair::generate(&mut rand::rng());
            let base = KeyPair::generate(&mut rand::rng());
            let signed = KeyPair::generate(&mut rand::rng());
            let one = KeyPair::generate(&mut rand::rng());
            let ratchet = KeyPair::generate(&mut rand::rng());
            let (kem_public, kem_secret) = kem::generate_key_pair().unwrap();
            let (kem_ciphertext, shared) = kem::encapsulate(&kem_public).unwrap();
            let bob_shared = kem::decapsulate(kem_secret.as_bytes(), &kem_ciphertext).unwrap();
            let mut ap = AliceSignalProtocolParameters::new(
                alice.clone(),
                base.clone(),
                *bob.identity_key(),
                signed.public_key,
                signed.public_key,
                UsePQRatchet::No,
            );
            if one_time {
                ap.set_their_one_time_pre_key(one.public_key);
            }
            let bp = BobSignalProtocolParameters::new(
                bob.clone(),
                signed.clone(),
                one_time.then_some(one),
                signed,
                *alice.identity_key(),
                base.public_key,
                UsePQRatchet::No,
            );
            let pending = IncomingKem {
                prekey_id: 9,
                ciphertext: &kem_ciphertext,
            };
            let a = initialize_alice(
                &ap,
                shared.as_bytes().try_into().unwrap(),
                &ratchet,
                (31, 42),
                (one_time.then_some(8), 7, &pending),
            )
            .unwrap();
            let b =
                initialize_bob(&bp, bob_shared.as_bytes().try_into().unwrap(), (42, 31)).unwrap();
            assert_eq!(format!("{a:?}"), "HybridSession(<redacted>)");
            let before = a.to_bytes().unwrap();
            let first = a.prepare(b"\x0a\x05hello").unwrap();
            assert_eq!(a.to_bytes().unwrap().as_slice(), before.as_slice());
            assert_eq!(first.kind, "pkmsg");
            assert_eq!(first.ciphertext[0], 0x44);
            let outer = PreKeySignalMessage::try_from(first.ciphertext.as_slice()).unwrap();
            assert_eq!(outer.message().serialized()[0], 0x44);
            let metadata = direct::incoming_prekey_kem(&first.ciphertext)
                .unwrap()
                .unwrap();
            assert_eq!(metadata.prekey_id, 9);
            assert_eq!(metadata.ciphertext, kem_ciphertext);
            let bob_account = account(&bob, 42);
            let alice_account = account(&alice, 31);
            let alice_jid = Jid::parse("15555550101@s.whatsapp.net").unwrap();
            let bob_jid = Jid::parse("15555550102@s.whatsapp.net").unwrap();
            let bob_before = b.to_bytes().unwrap();
            let mut wrong_ciphertext = kem_ciphertext.clone();
            wrong_ciphertext[0] ^= 1;
            let wrong_shared = kem::decapsulate(kem_secret.as_bytes(), &wrong_ciphertext).unwrap();
            let wrong_bob =
                initialize_bob(&bp, wrong_shared.as_bytes().try_into().unwrap(), (42, 31)).unwrap();
            let wrong_before = wrong_bob.to_bytes().unwrap();
            assert!(
                direct::receive(
                    &bob_account,
                    &alice_jid,
                    &wrong_before,
                    &alice.identity_key().serialize(),
                    outer.message().serialized()
                )
                .await
                .is_err()
            );
            assert_eq!(
                wrong_bob.to_bytes().unwrap().as_slice(),
                wrong_before.as_slice()
            );
            let mut forged = outer.message().serialized().to_vec();
            *forged.last_mut().unwrap() ^= 1;
            assert!(
                direct::receive(
                    &bob_account,
                    &alice_jid,
                    &bob_before,
                    &alice.identity_key().serialize(),
                    &forged
                )
                .await
                .is_err()
            );
            assert_eq!(b.to_bytes().unwrap().as_slice(), bob_before.as_slice());

            {
                use crate::registration::request::IdentityChangePolicy;
                use wacore_libsignal::protocol::{
                    GenericSignedPreKey, PreKeyRecord, SignedPreKeyRecord, Timestamp,
                };
                let sp = bp.our_signed_pre_key_pair();
                let signature = bob
                    .private_key()
                    .calculate_signature(&sp.public_key.serialize(), &mut rand::rng())
                    .unwrap();
                let signed = SignedPreKeyRecord::new(
                    7.into(),
                    Timestamp::from_epoch_millis(0),
                    sp,
                    &signature,
                )
                .serialize()
                .unwrap();
                let ordinary = bp
                    .our_one_time_pre_key_pair()
                    .map(|key| PreKeyRecord::new(8.into(), key).serialize().unwrap());
                let local_kem = |id, last_resort| LocalKemPrekey {
                    id,
                    secret: kem_secret.as_bytes(),
                    last_resort,
                };
                let incoming = receive_prekey(
                    &bob_account,
                    &alice_jid,
                    (None, None, IdentityChangePolicy::Pinned),
                    Some(&signed),
                    ordinary.as_deref(),
                    Some(local_kem(9, false)),
                    &first.ciphertext,
                )
                .await
                .unwrap();
                assert_eq!(
                    incoming.incoming.received.plaintext.as_slice(),
                    b"\x0a\x05hello"
                );
                assert_eq!(incoming.incoming.identity, alice.identity_key().serialize());
                assert_eq!(incoming.incoming.consumed_prekey, one_time.then_some(8));
                assert_eq!(incoming.consumed_kem, Some(9));
                assert_eq!(format!("{incoming:?}"), "ReceivedHybridPrekey(<redacted>)");
                assert_eq!(
                    format!("{:?}", local_kem(9, false)),
                    "LocalKemPrekey(<redacted>)"
                );
                let last_resort = receive_prekey(
                    &bob_account,
                    &alice_jid,
                    (None, None, IdentityChangePolicy::Pinned),
                    Some(&signed),
                    ordinary.as_deref(),
                    Some(local_kem(9, true)),
                    &first.ciphertext,
                )
                .await
                .unwrap();
                assert_eq!(last_resort.consumed_kem, None);
                if let Some(one) = bp.our_one_time_pre_key_pair() {
                    let reusable = PreKeyRecord::new(0xffffff.into(), one).serialize().unwrap();
                    let envelope = wire::encode_prekey(
                        4,
                        31,
                        (Some(0xffffff), 7),
                        (outer.base_key(), outer.identity_key()),
                        outer.message().serialized(),
                        Some((9, &kem_ciphertext)),
                    )
                    .unwrap();
                    let reusable = receive_prekey(
                        &bob_account,
                        &alice_jid,
                        (None, None, IdentityChangePolicy::Pinned),
                        Some(&signed),
                        Some(&reusable),
                        Some(local_kem(9, false)),
                        &envelope,
                    )
                    .await
                    .unwrap();
                    assert_eq!(reusable.incoming.consumed_prekey, None);
                    assert_eq!(reusable.consumed_kem, Some(9));
                }
                for scenario in 0..7 {
                    let mut ciphertext = first.ciphertext.clone();
                    if scenario == 0 {
                        *ciphertext.last_mut().unwrap() ^= 1;
                    }
                    if scenario == 6 {
                        ciphertext[0] = 0x33;
                    }
                    let invalid_kem = if scenario == 2 {
                        None
                    } else {
                        Some(local_kem(if scenario == 1 { 10 } else { 9 }, false))
                    };
                    let wrong_pin = bob.identity_key().serialize();
                    let pin = (scenario == 3).then_some(wrong_pin.as_ref());
                    let result = receive_prekey(
                        &bob_account,
                        &alice_jid,
                        (None, pin, IdentityChangePolicy::Pinned),
                        if scenario == 4 { None } else { Some(&signed) },
                        if scenario == 5 {
                            None
                        } else {
                            ordinary.as_deref()
                        },
                        invalid_kem,
                        &ciphertext,
                    )
                    .await;
                    if scenario == 5 && !one_time {
                        assert!(result.is_ok());
                    } else {
                        assert!(result.is_err(), "scenario {scenario}");
                    }
                }
                let replaced = receive_prekey(
                    &bob_account,
                    &alice_jid,
                    (
                        None,
                        Some(&bob.identity_key().serialize()),
                        IdentityChangePolicy::ApkAuthenticated,
                    ),
                    Some(&signed),
                    ordinary.as_deref(),
                    Some(local_kem(9, false)),
                    &first.ciphertext,
                )
                .await
                .unwrap();
                assert_eq!(replaced.incoming.identity, alice.identity_key().serialize());
                let next = HybridSession::from_bytes(&first.session)
                    .unwrap()
                    .prepare(b"\x0a\x05again")
                    .unwrap();
                let held = &incoming.incoming.received.session;
                let before = held.to_vec();
                let continued = receive_prekey(
                    &bob_account,
                    &alice_jid,
                    (
                        Some(held),
                        Some(&alice.identity_key().serialize()),
                        IdentityChangePolicy::Pinned,
                    ),
                    None,
                    None,
                    None,
                    &next.ciphertext,
                )
                .await
                .unwrap();
                assert_eq!(
                    continued.incoming.received.plaintext.as_slice(),
                    b"\x0a\x05again"
                );
                assert_eq!(continued.incoming.consumed_prekey, None);
                assert_eq!(continued.consumed_kem, None);
                assert_eq!(held.as_slice(), before);
                assert!(
                    receive_prekey(
                        &bob_account,
                        &alice_jid,
                        (
                            Some(&continued.incoming.received.session),
                            Some(&alice.identity_key().serialize()),
                            IdentityChangePolicy::Pinned
                        ),
                        None,
                        None,
                        None,
                        &next.ciphertext
                    )
                    .await
                    .is_err()
                );
                let mut other = SessionRecord::deserialize(held).unwrap();
                let mut components = b.0.clone().into_components().unwrap();
                components.current_session.as_mut().unwrap().alice_base_key = Some(
                    KeyPair::generate(&mut rand::rng())
                        .public_key
                        .serialize()
                        .to_vec(),
                );
                let different = SessionRecord::from_components(components).unwrap();
                let state = different.session_state().unwrap().clone();
                other.promote_fresh_state(state);
                let archived = other.serialize().unwrap();
                let recovered = receive_prekey(
                    &bob_account,
                    &alice_jid,
                    (
                        Some(&archived),
                        Some(&alice.identity_key().serialize()),
                        IdentityChangePolicy::Pinned,
                    ),
                    None,
                    None,
                    None,
                    &next.ciphertext,
                )
                .await
                .unwrap();
                assert_eq!(
                    recovered.incoming.received.plaintext.as_slice(),
                    b"\x0a\x05again"
                );
                assert_eq!(recovered.consumed_kem, None);
                let mut wrong_account = bob_account.clone();
                wrong_account.signal_registration_id = Some(99);
                assert!(
                    receive_prekey(
                        &wrong_account,
                        &alice_jid,
                        (
                            Some(held),
                            Some(&alice.identity_key().serialize()),
                            IdentityChangePolicy::Pinned
                        ),
                        None,
                        None,
                        None,
                        &next.ciphertext
                    )
                    .await
                    .is_err()
                );
            }
            let received = direct::receive(
                &bob_account,
                &alice_jid,
                &bob_before,
                &alice.identity_key().serialize(),
                outer.message().serialized(),
            )
            .await
            .unwrap();
            assert_eq!(received.plaintext.as_slice(), b"\x0a\x05hello");
            let b = HybridSession::from_bytes(&received.session).unwrap();
            let alice_signed = KeyPair::generate(&mut rand::rng());
            let signature = alice
                .private_key()
                .calculate_signature(&alice_signed.public_key.serialize(), &mut rand::rng())
                .unwrap();
            let alice_bundle = wacore_libsignal::protocol::PreKeyBundle::new(
                31,
                wacore_libsignal::protocol::DeviceId::new(0),
                None,
                11.into(),
                alice_signed.public_key,
                signature.to_vec(),
                *alice.identity_key(),
            )
            .unwrap();
            let reply = direct::prepare(
                &bob_account,
                &alice_jid,
                &alice_bundle,
                Some(&received.session),
                Some(&alice.identity_key().serialize()),
                "world",
            )
            .await
            .unwrap();
            assert_eq!(
                b.to_bytes().unwrap().as_slice(),
                received.session.as_slice()
            );
            assert!(
                direct::prepare_payload(
                    &bob_account,
                    &alice_jid,
                    None,
                    Some(&received.session),
                    None,
                    b"\x0a\x01x"
                )
                .await
                .is_err()
            );
            for invalid_account in [account(&alice, 42), account(&bob, 43)] {
                assert!(
                    direct::prepare_payload(
                        &invalid_account,
                        &alice_jid,
                        None,
                        Some(&received.session),
                        Some(&alice.identity_key().serialize()),
                        b"\x0a\x01x"
                    )
                    .await
                    .is_err()
                );
            }
            let mut mismatched_private = account(&bob, 42);
            mismatched_private
                .signal_identity
                .as_mut()
                .unwrap()
                .0
                .private[10] ^= 1;
            assert!(
                direct::prepare_payload(
                    &mismatched_private,
                    &alice_jid,
                    None,
                    Some(&received.session),
                    Some(&alice.identity_key().serialize()),
                    b"\x0a\x01x"
                )
                .await
                .is_err()
            );
            assert!(matches!(
                direct::prepare_payload(
                    &bob_account,
                    &alice_jid,
                    None,
                    Some(&received.session),
                    Some(&bob.identity_key().serialize()),
                    b"\x0a\x01x"
                )
                .await,
                Err(Error::IdentityChanged(_))
            ));
            let forged_bundle = wacore_libsignal::protocol::PreKeyBundle::new(
                31,
                wacore_libsignal::protocol::DeviceId::new(0),
                None,
                11.into(),
                alice_signed.public_key,
                vec![0; 64],
                *alice.identity_key(),
            )
            .unwrap();
            assert!(
                direct::prepare(
                    &bob_account,
                    &alice_jid,
                    &forged_bundle,
                    Some(&received.session),
                    Some(&alice.identity_key().serialize()),
                    "never"
                )
                .await
                .is_err()
            );
            let mut broken = b.0.clone().into_components().unwrap();
            broken
                .current_session
                .as_mut()
                .unwrap()
                .sender_chain
                .as_mut()
                .unwrap()
                .sender_ratchet_key_private = Some(vec![0; 32]);
            let broken = SessionRecord::from_components(broken)
                .unwrap()
                .serialize()
                .unwrap();
            assert!(
                direct::prepare_payload(
                    &bob_account,
                    &alice_jid,
                    None,
                    Some(&broken),
                    Some(&alice.identity_key().serialize()),
                    b"\x0a\x01x"
                )
                .await
                .is_err()
            );
            assert_eq!(
                b.to_bytes().unwrap().as_slice(),
                received.session.as_slice()
            );
            assert_eq!(reply.kind, "msg");
            assert_eq!(reply.ciphertext[0], 0x44);
            let acknowledged = direct::receive(
                &alice_account,
                &bob_jid,
                &first.session,
                &bob.identity_key().serialize(),
                &reply.ciphertext,
            )
            .await
            .unwrap();
            assert_eq!(acknowledged.plaintext.as_slice(), b"\x0a\x05world");
            let a = HybridSession::from_bytes(&acknowledged.session).unwrap();
            let next = direct::prepare_payload(
                &alice_account,
                &bob_jid,
                None,
                Some(&acknowledged.session),
                Some(&bob.identity_key().serialize()),
                b"\x0a\x05again",
            )
            .await
            .unwrap();
            assert_eq!(next.kind, "msg");
            assert!(
                direct::incoming_prekey_kem(&first.ciphertext)
                    .unwrap()
                    .is_some()
            );
            let final_reply = direct::receive(
                &bob_account,
                &alice_jid,
                &reply.session,
                &alice.identity_key().serialize(),
                &next.ciphertext,
            )
            .await
            .unwrap();
            assert_eq!(final_reply.plaintext.as_slice(), b"\x0a\x05again");
            let pending = HybridSession::from_bytes(&first.session)
                .unwrap()
                .prepare(b"\x0a\x05again")
                .unwrap();
            assert_eq!(pending.kind, "pkmsg");
            assert_eq!(
                direct::incoming_prekey_kem(&pending.ciphertext)
                    .unwrap()
                    .unwrap()
                    .ciphertext,
                kem_ciphertext
            );
            let parsed = PreKeySignalMessage::try_from(pending.ciphertext.as_slice()).unwrap();
            assert!(parsed.message().counter() > outer.message().counter());
            let a_before = a.to_bytes().unwrap();
            assert!(a.prepare(&vec![0; 65536]).is_err());
            assert_eq!(a.to_bytes().unwrap().as_slice(), a_before.as_slice());
            #[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
            {
                use crate::{
                    binary::{Node, NodeContent},
                    model::{
                        ids::{AccountId, ChatId, ClientRequestId},
                        message::MessageContent,
                        send::{AttemptProgress, DeliveryFactKind, SendRequest},
                    },
                    outbox::Outbox,
                    profile::Limits,
                    store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
                    transport::{ConnectionEpoch, outgoing},
                };
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("held-hybrid.redb");
                let open = || {
                    ProtocolStore::open(OpenOptions {
                        path: path.clone(),
                        account_id: AccountId::new("held-hybrid").unwrap(),
                        key_source: std::sync::Arc::new(StaticKeySource::new(
                            MasterKey::from_bytes([93; 32]),
                        )),
                        create: true,
                    })
                    .unwrap()
                };
                let store = open();
                let mut saved_account = account(&alice, 31);
                saved_account.own_jid = Some(alice_jid.clone());
                let peer_key = format!("comparison-direct:{bob_jid}");
                let outbox = Outbox::new("comparison-driver", Limits::default());
                let request_id = ClientRequestId::new("hybrid-held-intent").unwrap();
                let local_id = store
                    .write(|tx| {
                        AccountRecord::save(&saved_account, tx)?;
                        tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &*before)?;
                        tx.put_json(
                            Table::PeerIdentities,
                            peer_key.as_bytes(),
                            1,
                            &bob.identity_key().serialize().to_vec(),
                        )?;
                        Ok(outbox
                            .enqueue(
                                tx,
                                &SendRequest {
                                    client_request_id: request_id.clone(),
                                    chat: ChatId::from_jid(&bob_jid)?,
                                    content: MessageContent::Text {
                                        body: "coordinator".into(),
                                        quoted: None,
                                        style: Default::default(),
                                    },
                                },
                            )?
                            .0
                            .local_id)
                    })
                    .unwrap();
                let account_rows = store.read(|tx| tx.scan(Table::Account, 1, 200)).unwrap();
                let (pending, query) = outgoing::begin_selected(&store, Some(local_id))
                    .unwrap()
                    .unwrap();
                let signature = bob
                    .private_key()
                    .calculate_signature(
                        &bp.our_signed_pre_key_pair().public_key.serialize(),
                        &mut rand::rng(),
                    )
                    .unwrap();
                let reply = Node::new("iq")
                    .attr("type", "result")
                    .attr("id", query.get_attr("id").unwrap().as_text().unwrap())
                    .children(vec![Node::new("list").children(vec![
                            Node::new("user")
                                .attr("jid", bob_jid.clone())
                                .children(vec![
                                    Node::new("registration").bytes(vec![0, 0, 0, 42]),
                                    Node::new("type").bytes(vec![5]),
                                    Node::new("identity")
                                        .bytes(bob.public_key().public_key_bytes().to_vec()),
                                    Node::new("skey").children(vec![
                                        Node::new("id").bytes(vec![0, 0, 7]),
                                        Node::new("value").bytes(
                                            bp.our_signed_pre_key_pair()
                                                .public_key
                                                .public_key_bytes()
                                                .to_vec(),
                                        ),
                                        Node::new("signature").bytes(signature.to_vec()),
                                    ]),
                                    // On the user row, as Bot.A03 selects it;
                                    // a pqkey nested in skey never reached the
                                    // hybrid branch this test exists to cover.
                                    Node::new("pqkey").children(vec![
                                        Node::new("id").bytes(vec![0, 0, 9]),
                                        Node::new("value").bytes(kem_public.clone()),
                                        Node::new("signature").bytes(
                                            bob.private_key()
                                                .calculate_signature(
                                                    &kem_public,
                                                    &mut rand::rng(),
                                                )
                                                .unwrap()
                                                .to_vec(),
                                        ),
                                    ]),
                                ]),
                        ])]);
                let stanza = pending
                    .prepare(&store, &saved_account, &reply, ConnectionEpoch(1))
                    .await
                    .unwrap()
                    .unwrap();
                let enc = stanza.child("enc").unwrap();
                let NodeContent::Bytes(ciphertext) = &enc.content else {
                    panic!("missing ciphertext")
                };
                assert_eq!(ciphertext[0], 0x44);
                let parsed = PreKeySignalMessage::try_from(&ciphertext[..]).unwrap();
                let received = direct::receive(
                    &bob_account,
                    &alice_jid,
                    &bob_before,
                    &alice.identity_key().serialize(),
                    parsed.message().serialized(),
                )
                .await
                .unwrap();
                assert_eq!(received.plaintext.as_slice(), b"\x0a\x0bcoordinator");
                assert_eq!(
                    direct::incoming_prekey_kem(ciphertext)
                        .unwrap()
                        .unwrap()
                        .ciphertext,
                    kem_ciphertext
                );
                let entry = store.read(|tx| outbox.get(tx, local_id)).unwrap().unwrap();
                assert_eq!(entry.client_request_id, request_id);
                assert_eq!(entry.progress, AttemptProgress::InFlight);
                assert_eq!(entry.attempts.len(), 1);
                assert_eq!(
                    entry.facts.last().unwrap().kind,
                    DeliveryFactKind::DeliveryUnknown
                );
                assert_eq!(
                    store.read(|tx| tx.scan(Table::Account, 1, 200)).unwrap(),
                    account_rows
                );
                let saved = store
                    .read(|tx| {
                        tx.get_json::<Vec<u8>>(Table::SignalSessions, peer_key.as_bytes(), 1)
                    })
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    HybridSession::from_bytes(&saved)
                        .unwrap()
                        .prepare(b"\x0a\x01x")
                        .unwrap()
                        .ciphertext[0],
                    0x44
                );
                drop(store);
                let store = open();
                assert!(
                    outgoing::begin_selected(&store, Some(local_id))
                        .unwrap()
                        .is_none()
                );
                assert_eq!(
                    store
                        .read(|tx| outbox.get(tx, local_id))
                        .unwrap()
                        .unwrap()
                        .wire_id,
                    entry.wire_id
                );
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<Vec<u8>>(
                            Table::SignalSessions,
                            peer_key.as_bytes(),
                            1
                        ))
                        .unwrap()
                        .unwrap(),
                    saved
                );
            }
            let partial = IncomingKem {
                prekey_id: 9,
                ciphertext: &kem_ciphertext[..1535],
            };
            assert!(
                initialize_alice(
                    &ap,
                    shared.as_bytes().try_into().unwrap(),
                    &ratchet,
                    (31, 42),
                    (None, 7, &partial)
                )
                .is_err()
            );
        }
    }
}
