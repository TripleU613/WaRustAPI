//! Verified hybrid bundle material; cryptographic validation is not directory authority.
use super::initialize_alice;
use crate::signal::{direct, kem};
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::{Jid, Server},
    session::AccountRecord,
};
use wacore_libsignal::protocol::{
    AliceSignalProtocolParameters, IdentityKey, IdentityKeyPair, KeyPair, PreKeyBundle, PrivateKey,
    PublicKey, UsePQRatchet,
};

fn invalid(reason: &'static str) -> Error {
    Error::invalid("hybrid_bundle", reason)
}
/// Error field for a well-formed KEM signature that does not verify.
pub(crate) const SIGNATURE_FIELD: &str = "hybrid_bundle_signature";
fn bytes(node: &Node) -> Result<&[u8]> {
    match &node.content {
        NodeContent::Bytes(bytes) => Ok(bytes),
        _ => Err(invalid("expected KEM bytes")),
    }
}

/// Exact-recipient signed EC/KEM material; not authenticated directory authority.
/// Debug is redacted and no signature/key material is serialized by this object.
pub struct HybridBundle {
    recipient: Jid,
    classic: PreKeyBundle,
    kem_id: u32,
    kem_public: Vec<u8>,
}
impl std::fmt::Debug for HybridBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HybridBundle(<redacted>)")
    }
}

/// Whether the exact recipient row of a directory reply carries a `pqkey`.
/// Selection only, with the bundle parsers' exact-address, duplicate-row and
/// per-user error guards; nothing is verified. Bot.A03 selects `pqkey` from the
/// user row, so callers choose [`bundle_for`] when this is true and must never
/// fall back to a classic bundle that drops the key.
pub fn selects_pq_key(reply: &Node, recipient: &Jid) -> Result<bool> {
    Ok(direct::prekey_user_for(reply, recipient)?
        .child("pqkey")
        .is_some())
}

/// Validate complete ML-KEM-1024 metadata and both signatures for an exact recipient.
/// The caller must separately establish reply correlation, freshness and key authority.
/// Unknown/unported admission flags are not decided by this cryptographic parser.
pub fn bundle_for(reply: &Node, recipient: &Jid) -> Result<HybridBundle> {
    if !matches!(recipient.server, Server::User | Server::Lid) || recipient.agent != 0 {
        return Err(invalid("unsupported bundle recipient"));
    }
    let user = direct::prekey_user_for(reply, recipient)?;
    let classic = direct::classic_bundle_from_user(user, recipient)?;
    direct::validate_bundle_signature(&classic)?;
    let pq = direct::child(user, "pqkey")?;
    let kem_id = direct::number(direct::child(pq, "id")?, 3)?;
    let public = bytes(direct::child(pq, "value")?)?;
    let signature = bytes(direct::child(pq, "signature")?)?;
    if signature.len() != 64 {
        return Err(invalid("invalid KEM signature width"));
    }
    if !classic
        .identity_key()
        .map_err(|_| invalid("invalid bundle identity"))?
        .public_key()
        .verify_signature(public, signature)
    {
        // Distinct field: callers classify a failed CQc.A00 0068..00b4 check
        // apart from shape errors without matching reason text.
        return Err(Error::invalid(SIGNATURE_FIELD, "invalid KEM signature"));
    }
    kem::public_key(public)?;
    Ok(HybridBundle {
        recipient: recipient.clone(),
        classic,
        kem_id,
        kem_public: public.to_vec(),
    })
}

impl HybridBundle {
    #[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
    pub(crate) fn classic(&self) -> &PreKeyBundle {
        &self.classic
    }
    #[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
    pub(crate) async fn prepare_payload(
        &self,
        account: &AccountRecord,
        previous_session: Option<&[u8]>,
        pinned: Option<&[u8]>,
        payload: &[u8],
    ) -> Result<direct::PreparedDirect> {
        if let Some(bytes) = previous_session {
            // A held ratchet, classic or version 4, continues exactly as the
            // classic outbox path continues one: the fetched bundle is checked
            // against the pin but derives nothing. The native PQ session-upgrade
            // job (pqUpgradeDevicesRequested / isPqSessionUpgrade,
            // bulk-prekey-processing.md) that could replace a held classic
            // session is unported, so the KEM key is not used here.
            direct::prepare_payload(
                account,
                &self.recipient,
                Some(&self.classic),
                Some(bytes),
                pinned,
                payload,
            )
            .await
        } else {
            self.prepare_initial(account, pinned, payload)
        }
    }

    /// Stage a fresh initial encrypted message from an already-authorized bundle.
    /// An optional held pin must match; changed keys require separate explicit policy.
    /// No held session is accepted/reset, no ID is allocated and no store/network is touched.
    pub fn prepare_initial(
        &self,
        account: &AccountRecord,
        pinned: Option<&[u8]>,
        payload: &[u8],
    ) -> Result<direct::PreparedDirect> {
        if payload.is_empty() || payload.len() > 32768 {
            return Err(invalid("invalid initial payload/account"));
        }
        self.stage(account, pinned)?.prepare(payload)
    }

    /// Identity-refresh candidate for this verified bundle; no message is encrypted.
    ///
    /// 2.26.37.71 `0jc.A0L` resets a changed-key session before `CQc.A00`
    /// derives (docs/protocol/identity-refresh-application.md): pass no held
    /// record and the candidate carries no old state. For an already admitted
    /// same-key refresh, `CQc.A00` `0124..014c` archives the held current state
    /// before deriving on both its classic and Kyber branches, so the held
    /// record (classic or hybrid) keeps that state as an archive here. Its
    /// remote identity must equal this bundle's and its local identity and
    /// registration the saved account's. Admission, OCC and commit belong to
    /// the caller; nothing is stored or sent.
    #[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
    pub(crate) fn prepare_session_refresh(
        &self,
        account: &AccountRecord,
        held: Option<&[u8]>,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>> {
        let identity = *self
            .classic
            .identity_key()
            .map_err(|_| invalid("invalid bundle identity"))?;
        let fresh = self.stage(account, Some(&identity.serialize()))?;
        let state = fresh
            .0
            .session_state()
            .ok_or_else(|| invalid("missing fresh hybrid state"))?
            .clone();
        let mut record = match held {
            None => wacore_libsignal::protocol::SessionRecord::new_fresh(),
            Some(bytes) => {
                if bytes.len() > 1024 * 1024 {
                    return Err(invalid("held session exceeds local bound"));
                }
                let record = wacore_libsignal::protocol::SessionRecord::deserialize(bytes)
                    .map_err(|_| invalid("invalid held session"))?;
                let current = record
                    .session_state()
                    .ok_or_else(|| invalid("held session has no current state"))?;
                if current
                    .remote_identity_key()
                    .map_err(|_| invalid("invalid held session identity"))?
                    .as_ref()
                    != Some(&identity)
                {
                    return Err(invalid("held session identity does not match bundle"));
                }
                if current
                    .local_identity_key()
                    .map_err(|_| invalid("invalid held local identity"))?
                    != state
                        .local_identity_key()
                        .map_err(|_| invalid("invalid fresh local identity"))?
                    || current.local_registration_id() != state.local_registration_id()
                {
                    return Err(invalid("held session does not match saved account"));
                }
                record
            }
        };
        record.promote_fresh_state(state);
        Ok(zeroize::Zeroizing::new(
            record
                .serialize()
                .map_err(|_| invalid("record serialization failed"))?,
        ))
    }

    /// Fresh version-4 Alice record; no message, store or network operation.
    fn stage(
        &self,
        account: &AccountRecord,
        pinned: Option<&[u8]>,
    ) -> Result<super::HybridSession> {
        if !account.is_registered() {
            return Err(invalid("invalid initial payload/account"));
        }
        direct::validate_bundle_identity(Some(&self.classic), pinned, &self.recipient)?;
        let identity = account
            .signal_identity
            .as_ref()
            .ok_or_else(|| invalid("missing local identity"))?;
        let public = PublicKey::from_djb_public_key_bytes(&identity.0.public)
            .map_err(|_| invalid("invalid local identity"))?;
        let private = PrivateKey::deserialize(&identity.0.private)
            .map_err(|_| invalid("invalid local identity"))?;
        if private
            .public_key()
            .map_err(|_| invalid("invalid local identity"))?
            != public
        {
            return Err(invalid("local identity key pair mismatch"));
        }
        let local = IdentityKeyPair::new(IdentityKey::new(public), private);
        let registration = account
            .signal_registration_id
            .ok_or_else(|| invalid("missing local registration"))?;
        let peer_identity = *self
            .classic
            .identity_key()
            .map_err(|_| invalid("invalid bundle identity"))?;
        let signed = self
            .classic
            .signed_pre_key_public()
            .map_err(|_| invalid("invalid signed key"))?;
        let mut parameters = AliceSignalProtocolParameters::new(
            local,
            KeyPair::generate(&mut rand::rng()),
            peer_identity,
            signed,
            signed,
            UsePQRatchet::No,
        );
        if let Some(one) = self
            .classic
            .pre_key_public()
            .map_err(|_| invalid("invalid ordinary prekey"))?
        {
            parameters.set_their_one_time_pre_key(one);
        }
        let (ciphertext, shared) = kem::encapsulate(&self.kem_public)?;
        let pending = direct::IncomingKem {
            prekey_id: self.kem_id,
            ciphertext: &ciphertext,
        };
        initialize_alice(
            &parameters,
            shared.as_bytes().try_into().expect("32-byte KEM secret"),
            &KeyPair::generate(&mut rand::rng()),
            (
                registration,
                self.classic
                    .registration_id()
                    .map_err(|_| invalid("invalid remote registration"))?,
            ),
            (
                self.classic
                    .pre_key_id()
                    .map_err(|_| invalid("invalid ordinary prekey"))?
                    .map(u32::from),
                u32::from(
                    self.classic
                        .signed_pre_key_id()
                        .map_err(|_| invalid("invalid signed id"))?,
                ),
                &pending,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
        signal::{direct, hybrid, kem},
    };
    use wacore_libsignal::protocol::{
        BobSignalProtocolParameters, IdentityKeyPair, KeyPair, PreKeySignalMessage, UsePQRatchet,
    };

    #[tokio::test]
    async fn exact_selected_pq_bundle_stages_a_fresh_authentic_exchange() {
        for raw_peer in ["15555550102@s.whatsapp.net", "67890@lid"] {
            let peer = Jid::parse(raw_peer).unwrap();
            let alice = IdentityKeyPair::generate(&mut rand::rng());
            let bob = IdentityKeyPair::generate(&mut rand::rng());
            let signed = KeyPair::generate(&mut rand::rng());
            let one = KeyPair::generate(&mut rand::rng());
            let (kem_public, kem_secret) = kem::generate_key_pair().unwrap();
            let signature = bob
                .private_key()
                .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
                .unwrap();
            let pq_signature = bob
                .private_key()
                .calculate_signature(&kem_public, &mut rand::rng())
                .unwrap();
            let user = Node::new("user").attr("jid", peer.clone()).children(vec![
                Node::new("registration").bytes(vec![0, 0, 0, 42]),
                Node::new("type").bytes(vec![5]),
                Node::new("identity").bytes(bob.public_key().public_key_bytes().to_vec()),
                Node::new("key").children(vec![
                    Node::new("id").bytes(vec![0, 0, 8]),
                    Node::new("value").bytes(one.public_key.public_key_bytes().to_vec()),
                ]),
                Node::new("skey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 7]),
                    Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                    Node::new("signature").bytes(signature.to_vec()),
                ]),
                Node::new("pqkey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 9]),
                    Node::new("value").bytes(kem_public.clone()),
                    Node::new("signature").bytes(pq_signature.to_vec()),
                ]),
            ]);
            let mut other_device = peer.clone();
            other_device.device = 2;
            let reply_for = |selected: Node| {
                Node::new("iq")
                    .attr("type", "result")
                    .children(vec![Node::new("list").children(vec![
                        Node::new("user")
                            .attr("jid", other_device.clone())
                            .children(vec![Node::new("error")]),
                        selected,
                    ])])
            };
            let reply = reply_for(user.clone());
            assert!(bundle_for(&reply, &other_device).is_err());
            // Selection shares the parsers' exact-row guards and verifies nothing.
            assert!(selects_pq_key(&reply, &peer).unwrap());
            assert!(selects_pq_key(&reply, &other_device).is_err());
            let classic_row = Node::new("user").attr("jid", peer.clone()).children(
                user.child_nodes()
                    .iter()
                    .filter(|n| n.description != "pqkey")
                    .cloned()
                    .collect(),
            );
            assert!(!selects_pq_key(&reply_for(classic_row), &peer).unwrap());
            let duplicate_users = Node::new("iq").attr("type", "result").children(vec![
                Node::new("list").children(vec![user.clone(), user.clone()]),
            ]);
            assert!(bundle_for(&duplicate_users, &peer).is_err());
            assert!(selects_pq_key(&duplicate_users, &peer).is_err());
            assert!(direct::prekey_bundle_for(&reply, &peer).is_err());
            let bundle = bundle_for(&reply, &peer).unwrap();
            assert_eq!(format!("{bundle:?}"), "HybridBundle(<redacted>)");
            let local = AccountRecord {
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(31),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public: alice.public_key().public_key_bytes().try_into().unwrap(),
                    private: *alice.private_key().serialize(),
                })),
                ..Default::default()
            };
            let prepared = bundle
                .prepare_initial(
                    &local,
                    Some(&bob.identity_key().serialize()),
                    b"\x0a\x05hello",
                )
                .unwrap();
            assert_eq!(prepared.ciphertext[0], 0x44);
            assert_eq!(prepared.identity, bob.identity_key().serialize());
            let outer = PreKeySignalMessage::try_from(prepared.ciphertext.as_slice()).unwrap();
            assert_eq!(outer.pre_key_id().map(u32::from), Some(8));
            let pq = direct::incoming_prekey_kem(&prepared.ciphertext)
                .unwrap()
                .unwrap();
            assert_eq!(pq.prekey_id, 9);
            let shared = kem::decapsulate(kem_secret.as_bytes(), pq.ciphertext).unwrap();
            let bp = BobSignalProtocolParameters::new(
                bob.clone(),
                signed.clone(),
                Some(one.clone()),
                signed.clone(),
                *alice.identity_key(),
                *outer.base_key(),
                UsePQRatchet::No,
            );
            let b = hybrid::initialize_bob(&bp, shared.as_bytes().try_into().unwrap(), (42, 31))
                .unwrap();
            let remote = AccountRecord {
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(42),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public: bob.public_key().public_key_bytes().try_into().unwrap(),
                    private: *bob.private_key().serialize(),
                })),
                ..Default::default()
            };
            let received = direct::receive(
                &remote,
                &Jid::parse("15555550101@s.whatsapp.net").unwrap(),
                &b.to_bytes().unwrap(),
                &alice.identity_key().serialize(),
                outer.message().serialized(),
            )
            .await
            .unwrap();
            assert_eq!(received.plaintext.as_slice(), b"\x0a\x05hello");
            #[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
            {
                use crate::{
                    model::{
                        AccountId, AttemptProgress, ChatId, ClientRequestId, DeliveryFactKind,
                        MessageContent, SendRequest,
                    },
                    outbox::Outbox,
                    profile::Limits,
                    store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
                    transport::{ConnectionEpoch, outgoing},
                };
                for scenario in 0..10 {
                    if scenario == 8 && peer.server != Server::Lid {
                        continue;
                    }
                    let target = peer.clone();
                    let dir = tempfile::tempdir().unwrap();
                    let path = dir.path().join("fresh-pq.redb");
                    let open = || {
                        ProtocolStore::open(OpenOptions {
                            path: path.clone(),
                            account_id: AccountId::new("fresh-pq").unwrap(),
                            key_source: std::sync::Arc::new(StaticKeySource::new(
                                MasterKey::from_bytes([94; 32]),
                            )),
                            create: true,
                        })
                        .unwrap()
                    };
                    let store = open();
                    let mut saved = local.clone();
                    saved.own_jid = Some(if scenario == 3 {
                        target.clone()
                    } else {
                        Jid::parse("15555550101@s.whatsapp.net").unwrap()
                    });
                    let outbox = Outbox::new("comparison-driver", Limits::default());
                    let request_id = ClientRequestId::new("fresh-pq-intent").unwrap();
                    let held_classic = if scenario == 5 {
                        Some(
                            direct::prepare_payload(
                                &saved,
                                &target,
                                Some(&bundle.classic),
                                None,
                                None,
                                b"\x0a\x05hello",
                            )
                            .await
                            .unwrap()
                            .session,
                        )
                    } else {
                        None
                    };

                    let id = store
                        .write(|tx| {
                            AccountRecord::save(&saved, tx)?;
                            if scenario == 8 {
                                tx.put_json(
                                    Table::Checkpoints,
                                    crate::transport::group_send::OWN_LID_KEY,
                                    1,
                                    &target,
                                )?;
                            }
                            let peer_key = format!("comparison-direct:{target}");
                            if (5..=8).contains(&scenario) {
                                let pin = if scenario == 5 {
                                    bob.identity_key()
                                } else {
                                    alice.identity_key()
                                };
                                tx.put_json(
                                    Table::PeerIdentities,
                                    peer_key.as_bytes(),
                                    1,
                                    &pin.serialize().to_vec(),
                                )?;
                                tx.put_json(
                                    Table::SignalSessions,
                                    peer_key.as_bytes(),
                                    1,
                                    &held_classic
                                        .as_deref()
                                        .map(|bytes| bytes.as_slice())
                                        .unwrap_or(&[1u8, 2, 3]),
                                )?;
                            }
                            Ok(outbox
                                .enqueue(
                                    tx,
                                    &SendRequest {
                                        client_request_id: request_id.clone(),
                                        chat: ChatId::from_jid(&target)?,
                                        content: MessageContent::Text {
                                            body: "hello".into(),
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
                    let (pending, query) = outgoing::begin_selected(&store, Some(id))
                        .unwrap()
                        .unwrap_or_else(|| panic!("no query scenario {scenario}"));
                    let mut fields = user.child_nodes().to_vec();
                    if scenario == 1 {
                        fields.push(Node::new("non_e2ee_attestation"));
                    }
                    if scenario == 2 {
                        fields.push(Node::new("identity_auth"));
                    }
                    if scenario == 9 {
                        // Bot.A03 shape only: signature bytes plus a certs node.
                        fields.push(Node::new("identity_auth").children(vec![
                            Node::new("signature").bytes(vec![1; 64]),
                            Node::new("certs"),
                        ]));
                    }
                    let mut returned_peer = target.clone();
                    if scenario == 4 {
                        returned_peer.device = 2;
                    }
                    let response = Node::new("iq")
                        .attr("type", "result")
                        .attr("id", query.get_attr("id").unwrap().as_text().unwrap())
                        .children(vec![Node::new("list").children(
                            vec![Node::new("user").attr("jid",returned_peer).children(fields)],
                        )]);
                    let snapshot = || {
                        store
                            .read(|tx| {
                                Table::ALL
                                    .iter()
                                    .map(|table| tx.scan(*table, 1, 200))
                                    .collect::<Result<Vec<_>>>()
                            })
                            .unwrap()
                    };
                    let before = snapshot();
                    let head = store.journal_head().unwrap();
                    let result = pending
                        .prepare_with_policy(
                            &store,
                            &saved,
                            &response,
                            ConnectionEpoch(1),
                            if scenario == 7 {
                                crate::registration::request::IdentityChangePolicy::ApkAuthenticated
                            } else {
                                crate::registration::request::IdentityChangePolicy::Pinned
                            },
                        )
                        .await;
                    if scenario == 5 {
                        // A held classic ratchet continues; the verified KEM
                        // key derives nothing and the record stays version 3.
                        let stanza = result.unwrap().unwrap();
                        let NodeContent::Bytes(ciphertext) = &stanza.child("enc").unwrap().content
                        else {
                            panic!("missing ciphertext")
                        };
                        assert_eq!(ciphertext[0], 0x33);
                        assert!(direct::incoming_prekey_kem(ciphertext).unwrap().is_none());
                        use wacore_libsignal::protocol::GenericSignedPreKey as _;
                        let signed_record = wacore_libsignal::protocol::SignedPreKeyRecord::new(
                            7.into(),
                            wacore_libsignal::protocol::Timestamp::from_epoch_millis(0),
                            &signed,
                            &signature,
                        )
                        .serialize()
                        .unwrap();
                        let one_record =
                            wacore_libsignal::protocol::PreKeyRecord::new(8.into(), &one)
                                .serialize()
                                .unwrap();
                        let received = direct::receive_prekey(
                            &remote,
                            saved.own_jid.as_ref().unwrap(),
                            (None, None),
                            &signed_record,
                            Some(&one_record),
                            ciphertext,
                        )
                        .await
                        .unwrap();
                        assert_eq!(received.received.plaintext.as_slice(), b"\x0a\x05hello");
                        let peer_key = format!("comparison-direct:{target}");
                        let session: Vec<u8> = store
                            .read(|tx| tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1))
                            .unwrap()
                            .unwrap();
                        assert!(hybrid::HybridSession::from_bytes(&session).is_err());
                        let entry = store.read(|tx| outbox.get(tx, id)).unwrap().unwrap();
                        assert_eq!(entry.progress, AttemptProgress::InFlight);
                        assert_eq!(entry.attempts.len(), 1);
                        continue;
                    }
                    if (1..=4).contains(&scenario) || scenario == 8 {
                        assert!(result.is_err(), "scenario {scenario}");
                        assert_eq!(snapshot(), before);
                        assert_eq!(store.journal_head().unwrap(), head);
                        continue;
                    }
                    if scenario == 6 {
                        assert!(result.unwrap().is_none());
                        let entry = store.read(|tx| outbox.get(tx, id)).unwrap().unwrap();
                        assert_eq!(entry.progress, AttemptProgress::Queued);
                        assert!(entry.attempts.is_empty());
                        assert!(entry.wire_id.is_none());
                        let peer_key = format!("comparison-direct:{target}");
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
                        continue;
                    }
                    let stanza = result
                        .unwrap()
                        .unwrap_or_else(|| panic!("no stanza scenario {scenario}"));
                    let NodeContent::Bytes(ciphertext) = &stanza.child("enc").unwrap().content
                    else {
                        panic!("missing ciphertext")
                    };
                    assert_eq!(ciphertext[0], 0x44);
                    let outer = PreKeySignalMessage::try_from(&ciphertext[..]).unwrap();
                    let pq = direct::incoming_prekey_kem(ciphertext).unwrap().unwrap();
                    let shared = kem::decapsulate(kem_secret.as_bytes(), pq.ciphertext).unwrap();
                    let bp = BobSignalProtocolParameters::new(
                        bob.clone(),
                        signed.clone(),
                        Some(one.clone()),
                        signed.clone(),
                        *alice.identity_key(),
                        *outer.base_key(),
                        UsePQRatchet::No,
                    );
                    let b = hybrid::initialize_bob(
                        &bp,
                        shared.as_bytes().try_into().unwrap(),
                        (42, 31),
                    )
                    .unwrap();
                    let received = direct::receive(
                        &remote,
                        saved.own_jid.as_ref().unwrap(),
                        &b.to_bytes().unwrap(),
                        &alice.identity_key().serialize(),
                        outer.message().serialized(),
                    )
                    .await
                    .unwrap();
                    assert_eq!(received.plaintext.as_slice(), b"\x0a\x05hello");
                    let entry = store.read(|tx| outbox.get(tx, id)).unwrap().unwrap();
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
                    let peer_key = format!("comparison-direct:{target}");
                    let session: Vec<u8> = store
                        .read(|tx| tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1))
                        .unwrap()
                        .unwrap();
                    hybrid::HybridSession::from_bytes(&session).unwrap();
                    if scenario == 7 {
                        let rows = store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 200))
                            .unwrap();
                        let histories = rows
                            .iter()
                            .filter(|(key, _)| key.starts_with(b"identity-history:"))
                            .collect::<Vec<_>>();
                        assert_eq!(histories.len(), 1);
                        let history: serde_json::Value =
                            serde_json::from_slice(&histories[0].1).unwrap();
                        assert_eq!(history["authentication"], "signed_directory_bundle");
                        assert_eq!(
                            serde_json::from_value::<Vec<u8>>(history["previous_identity"].clone())
                                .unwrap(),
                            alice.identity_key().serialize()
                        );
                        assert_eq!(
                            serde_json::from_value::<Vec<u8>>(history["previous_session"].clone())
                                .unwrap(),
                            vec![1u8, 2, 3]
                        );
                    }

                    assert_eq!(
                        store
                            .read(|tx| tx.get_json::<Vec<u8>>(
                                Table::PeerIdentities,
                                peer_key.as_bytes(),
                                1
                            ))
                            .unwrap(),
                        Some(bob.identity_key().serialize().to_vec())
                    );
                    drop(store);
                    let store = open();
                    assert_eq!(store.read(|tx| outbox.get(tx, id)).unwrap().unwrap(), entry);
                    assert!(
                        outgoing::begin_selected(&store, Some(id))
                            .unwrap()
                            .is_none()
                    );
                }
            }

            assert!(matches!(
                bundle.prepare_initial(
                    &local,
                    Some(&alice.identity_key().serialize()),
                    b"\x0a\x01x"
                ),
                Err(Error::IdentityChanged(_))
            ));
            assert!(
                bundle
                    .prepare_initial(&AccountRecord::default(), None, b"\x0a\x01x")
                    .is_err()
            );
            assert!(
                bundle
                    .prepare_initial(&local, None, &vec![0; 32769])
                    .is_err()
            );
            let mut bad_local = local.clone();
            bad_local.signal_identity.as_mut().unwrap().0.private[10] ^= 1;
            assert!(
                bundle
                    .prepare_initial(&bad_local, None, b"\x0a\x01x")
                    .is_err()
            );

            let replace = |name: &str, replacement: Option<Node>| {
                let mut fields = user
                    .child_nodes()
                    .iter()
                    .filter(|n| n.description != name)
                    .cloned()
                    .collect::<Vec<_>>();
                fields.extend(replacement);
                Node::new("user").attr("jid", peer.clone()).children(fields)
            };
            for field in ["id", "value", "signature"] {
                let children = user
                    .child("pqkey")
                    .unwrap()
                    .child_nodes()
                    .iter()
                    .filter(|n| n.description != field)
                    .cloned()
                    .collect();
                let selected = replace("pqkey", Some(Node::new("pqkey").children(children)));
                assert!(bundle_for(&reply_for(selected), &peer).is_err());
            }
            let without_one = bundle_for(&reply_for(replace("key", None)), &peer).unwrap();
            let no_one = without_one
                .prepare_initial(&local, None, b"\x0a\x05hello")
                .unwrap();
            let no_one_message =
                PreKeySignalMessage::try_from(no_one.ciphertext.as_slice()).unwrap();
            assert!(no_one_message.pre_key_id().is_none());
            let pq = direct::incoming_prekey_kem(&no_one.ciphertext)
                .unwrap()
                .unwrap();
            let shared = kem::decapsulate(kem_secret.as_bytes(), pq.ciphertext).unwrap();
            let params = BobSignalProtocolParameters::new(
                bob.clone(),
                signed.clone(),
                None,
                signed.clone(),
                *alice.identity_key(),
                *no_one_message.base_key(),
                UsePQRatchet::No,
            );
            let receiver =
                hybrid::initialize_bob(&params, shared.as_bytes().try_into().unwrap(), (42, 31))
                    .unwrap();
            assert_eq!(
                direct::receive(
                    &remote,
                    &Jid::parse("15555550101@s.whatsapp.net").unwrap(),
                    &receiver.to_bytes().unwrap(),
                    &alice.identity_key().serialize(),
                    no_one_message.message().serialized()
                )
                .await
                .unwrap()
                .plaintext
                .as_slice(),
                b"\x0a\x05hello"
            );
            for id in [vec![], vec![0, 9], vec![0, 0, 0, 9]] {
                let selected = replace(
                    "pqkey",
                    Some(Node::new("pqkey").children(vec![
                        Node::new("id").bytes(id),
                        Node::new("value").bytes(kem_public.clone()),
                        Node::new("signature").bytes(pq_signature.to_vec()),
                    ])),
                );
                assert!(bundle_for(&reply_for(selected), &peer).is_err());
            }
            let wrong_signature = alice
                .private_key()
                .calculate_signature(&kem_public, &mut rand::rng())
                .unwrap();
            let selected = replace(
                "pqkey",
                Some(Node::new("pqkey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 9]),
                    Node::new("value").bytes(kem_public.clone()),
                    Node::new("signature").bytes(wrong_signature.to_vec()),
                ])),
            );
            assert!(bundle_for(&reply_for(selected), &peer).is_err());
            let bad_classic = replace(
                "skey",
                Some(Node::new("skey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 7]),
                    Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                    Node::new("signature").bytes(vec![0; 64]),
                ])),
            );
            assert!(bundle_for(&reply_for(bad_classic), &peer).is_err());
            for bad_key in [vec![], vec![0; 800], vec![0xff; 1568]] {
                let signed_bad = bob
                    .private_key()
                    .calculate_signature(&bad_key, &mut rand::rng())
                    .unwrap();
                let selected = replace(
                    "pqkey",
                    Some(Node::new("pqkey").children(vec![
                        Node::new("id").bytes(vec![0, 0, 9]),
                        Node::new("value").bytes(bad_key),
                        Node::new("signature").bytes(signed_bad.to_vec()),
                    ])),
                );
                assert!(bundle_for(&reply_for(selected), &peer).is_err());
            }
            let mut duplicated_values = user.child("pqkey").unwrap().child_nodes().to_vec();
            duplicated_values.push(Node::new("value").bytes(kem_public.clone()));
            assert!(
                bundle_for(
                    &reply_for(replace(
                        "pqkey",
                        Some(Node::new("pqkey").children(duplicated_values))
                    )),
                    &peer
                )
                .is_err()
            );
            let nonbytes = replace(
                "pqkey",
                Some(Node::new("pqkey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 9]),
                    Node::new("value").text("PRIVATE-CANARY"),
                    Node::new("signature").bytes(pq_signature.to_vec()),
                ])),
            );
            let error = bundle_for(&reply_for(nonbytes), &peer)
                .unwrap_err()
                .to_string();
            assert!(!error.contains("PRIVATE"));
            let bad_sig = Node::new("pqkey").children(vec![
                Node::new("id").bytes(vec![0, 0, 9]),
                Node::new("value").bytes(kem_public.clone()),
                Node::new("signature").bytes(vec![0; 64]),
            ]);
            assert!(bundle_for(&reply_for(replace("pqkey", Some(bad_sig))), &peer).is_err());
            assert!(
                bundle_for(&reply, &Jid::parse("15555550198@s.whatsapp.net").unwrap()).is_err()
            );
            assert!(bundle_for(&reply_for(replace("pqkey", None)), &peer).is_err());
            let mut duplicate = user.child_nodes().to_vec();
            duplicate.push(user.child("pqkey").unwrap().clone());
            assert!(
                bundle_for(
                    &reply_for(
                        Node::new("user")
                            .attr("jid", peer.clone())
                            .children(duplicate)
                    ),
                    &peer
                )
                .is_err()
            );
        }
    }
}
