//! APK 37.71 11T.A0D matched-baseline companion admission.
//! Known-device newer ADV updates pending refresh state after authentication,
//! never the verified-list timestamp (0bN.A05, 0bp.A00/A05). See peer-identity.md.
use super::devices::{DeviceList, authority_in, device_list_in, list_key};
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::{Jid, Server},
    session::AccountRecord,
    signal::adv::VerifiedDeviceIdentity,
    store::{ProtocolStore, Table, WriteTx},
};

fn same_account(a: &AccountRecord, b: &AccountRecord) -> bool {
    a.generation == b.generation
        && a.registration == b.registration
        && a.signal_registration_id == b.signal_registration_id
        && a.own_jid == b.own_jid
        && a.signal_identity.as_ref().map(|k| k.0.public)
            == b.signal_identity.as_ref().map(|k| k.0.public)
}

fn same_user(a: &Jid, b: &Jid) -> bool {
    a.user == b.user && a.server == b.server
}

pub(crate) enum Admission {
    Ready(Box<AuthoritySnapshot>),
    Refused(&'static str),
}

/// Inputs authorizing this candidate, rechecked with the ratchet commit.
/// No Debug implementation: this is internal authority, not diagnostic output.
pub(crate) struct AuthoritySnapshot {
    list: DeviceList,
    primary_pin: Vec<u8>,
    login_identity: Option<serde_json::Value>,
    account: AccountRecord,
    refresh_timestamp: Option<u64>,
    device_addition: Option<super::devices::DeviceEntry>,
}

impl AuthoritySnapshot {
    pub(crate) fn commit(&self, tx: &mut WriteTx<'_>) -> Result<()> {
        self.check(tx)?;
        let mut list = self.list.clone();
        if let Some(entry) = &self.device_addition {
            // 11T.A0D 06f2..0994 adds absent addresses only. Both ADV
            // signatures and nested Signal MAC were checked before this commit.
            list.devices.push(entry.clone());
            list.devices.sort_by(|a, b| a.jid.cmp(&b.jid));
            list.adv_added_devices.push(entry.jid.clone());
            list.adv_added_devices.sort();
            list.hash_matches = None;
            tx.put_json(
                Table::Checkpoints,
                list_key(&list.owner).as_bytes(),
                1,
                &list,
            )?;
        }
        if let Some(timestamp) = self.refresh_timestamp {
            super::devices::request_adv_refresh_in(tx, &list, timestamp)?;
        }
        Ok(())
    }
    pub(crate) fn check(&self, tx: &WriteTx<'_>) -> Result<()> {
        let list: Option<DeviceList> =
            tx.get_json(Table::Checkpoints, list_key(&self.list.owner).as_bytes(), 1)?;
        let pin: Option<Vec<u8>> = tx.get_json(
            Table::PeerIdentities,
            format!("comparison-direct:{}", self.list.owner).as_bytes(),
            1,
        )?;
        let login = tx.get_json::<serde_json::Value>(Table::Checkpoints, b"login-identity", 1)?;
        let account = AccountRecord::load_in_write(tx)?;
        if list.as_ref() != Some(&self.list)
            || pin.as_ref() != Some(&self.primary_pin)
            || login != self.login_identity
            || !account
                .as_ref()
                .is_some_and(|a| same_account(a, &self.account))
        {
            return Err(Error::invalid(
                "inbox",
                "companion authority changed; stanza not committed",
            ));
        }
        Ok(())
    }
}

/// Read-only admission diagnostic for an actual companion prekey candidate.
/// Returns fixed status names, never keys or signed metadata. `ready` means
/// account-baseline admission is supported; `ready_refresh_pending` also
/// requires a pending refresh after authentication. Nested Signal MAC verification
/// and atomic state commit are still required. No query or state mutation.
pub fn inspect(
    store: &ProtocolStore,
    account: &AccountRecord,
    peer: &Jid,
    node: &Node,
    ciphertext: &[u8],
) -> Result<&'static str> {
    Ok(match prepare(store, account, peer, node, ciphertext)? {
        Admission::Ready(authority) if authority.refresh_timestamp.is_some() => {
            "ready_refresh_pending"
        }
        Admission::Ready(_) => "ready",
        Admission::Refused(status) => status,
    })
}

pub(crate) fn prepare(
    store: &ProtocolStore,
    account: &AccountRecord,
    peer: &Jid,
    node: &Node,
    ciphertext: &[u8],
) -> Result<Admission> {
    use Admission::{Ready, Refused};
    if peer.agent != 0 || peer.device == 0 || !matches!(peer.server, Server::User | Server::Lid) {
        return Ok(Refused("unsupported_sender"));
    }
    // DEs.CF2 -> Cih.A0V -> Bp2.A0L is the ordinary device-identity body
    // passed by CoU.A04 to 11T.A0D. Unique byte-body selection is local policy.
    let adv: Vec<_> = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "device-identity")
        .collect();
    let pairwise = node
        .child_nodes()
        .iter()
        .filter(|n| {
            n.description == "enc"
                && matches!(
                    n.get_attr("type").and_then(|v| v.as_text()),
                    Some("msg" | "pkmsg")
                )
        })
        .count();
    if adv.len() > 1 || pairwise != 1 {
        return Ok(Refused("ambiguous_input"));
    }
    let Some(adv) = adv.first() else {
        return Ok(Refused("missing_device_identity"));
    };
    let NodeContent::Bytes(bytes) = &adv.content else {
        return Ok(Refused("invalid_device_identity"));
    };
    store.read(|tx| {
        let Some(saved_account) = AccountRecord::load(tx)? else {
            return Ok(Refused("missing_account"));
        };
        if !same_account(&saved_account, account) {
            return Ok(Refused("account_changed"));
        }
        let login: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, b"login-identity", 1)?;
        let own_aliases: Vec<Jid> = ["pn", "lid"]
            .into_iter()
            .filter_map(|field| {
                login.as_ref()?[field]
                    .as_str()
                    .and_then(|s| Jid::parse(s).ok())
            })
            .chain(saved_account.own_jid.iter().cloned())
            .collect();
        if own_aliases.iter().any(|own| same_user(own, peer)) {
            return Ok(Refused("own_device"));
        }
        // Pinned policy permits first-seen keys. It alone cannot protect own
        // companions. Unknown own namespace is deferred by our local guard.
        if !own_aliases.iter().any(|own| own.server == peer.server) {
            return Ok(Refused("missing_own_identity"));
        }
        let primary = peer.to_non_ad();
        let Some(pin): Option<Vec<u8>> = tx.get_json(
            Table::PeerIdentities,
            format!("comparison-direct:{primary}").as_bytes(),
            1,
        )?
        else {
            return Ok(Refused("missing_primary_pin"));
        };
        let raw: Option<&[u8; 32]> = (pin.len() == 33 && pin[0] == 5)
            .then(|| pin[1..].try_into().ok())
            .flatten();
        let Some(raw) = raw else {
            return Ok(Refused("invalid_primary_pin"));
        };
        let Ok(verified) = VerifiedDeviceIdentity::verify_prekey(bytes, raw, ciphertext) else {
            return Ok(Refused("invalid_device_identity"));
        };
        let Some(list) = device_list_in(tx, &primary)? else {
            return Ok(Refused("missing_device_list"));
        };
        if !same_user(&list.owner, &primary) || !authority_in(tx, &list)?.companions_current {
            return Ok(Refused("stale_device_list"));
        }
        if !verified.is_e2ee() {
            return Ok(Refused("unsupported_encryption_type"));
        }
        if list.raw_id != Some(verified.raw_id()) {
            return Ok(Refused("raw_id_mismatch"));
        }
        let Some(timestamp) = list.list_timestamp else {
            return Ok(Refused("missing_list_timestamp"));
        };
        // 11T.A0D 04c8..05fa checks the ACTUAL device index, not an alias.
        // A known index may continue with pending refresh (05fa..0628).
        // Remote addition is add-only; own/canonical reconciliation is separate.
        let entries: Vec<_> = list
            .devices
            .iter()
            .filter(|d| {
                same_user(&d.jid, peer) && d.jid.agent == peer.agent && d.jid.device == peer.device
            })
            .collect();
        if entries.len() > 1 {
            return Ok(Refused("ambiguous_device_membership"));
        }
        // 11T.A0D 0694 converts Java int to long; 08f8..0994 requires
        // a positive companion index. High unsigned values are negative there.
        if verified.key_index() == 0 || verified.key_index() > i32::MAX as u32 {
            return Ok(Refused("invalid_companion_index"));
        }
        let matched = entries
            .first()
            .is_some_and(|entry| entry.key_index == verified.key_index());
        if !matched && verified.timestamp() <= timestamp {
            return Ok(Refused("deleted_device"));
        }
        let device_addition = if entries.is_empty() {
            // CoU.A04 gates absent offline senders on the dead-device registry.
            // That lifecycle is unported; this is our conservative deferral.
            if node.attributes.iter().any(|a| a.name == "offline") {
                return Ok(Refused("offline_device_update_deferred"));
            }
            // Our retained-map uniqueness/bounds guards; not server errors.
            if list.devices.len() >= 1024
                || list
                    .devices
                    .iter()
                    .any(|d| d.key_index == verified.key_index())
            {
                return Ok(Refused("conflicting_device_membership"));
            }
            Some(super::devices::DeviceEntry {
                jid: peer.clone(),
                key_index: verified.key_index(),
            })
        } else {
            None
        };
        Ok(Ready(Box::new(AuthoritySnapshot {
            list,
            device_addition,
            primary_pin: pin,
            login_identity: login,
            account: saved_account,
            refresh_timestamp: (verified.timestamp() > timestamp).then_some(verified.timestamp()),
        })))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{ids::AccountId, time::Timestamp},
        registration::request::{IdentityChangePolicy, LabRegistrationConfig},
        session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
        signal::{direct, group},
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use sha2::{Digest, Sha256};
    use wacore_libsignal::protocol::{
        DeviceId, GenericSignedPreKey, IdentityKey, IdentityKeyPair, KeyPair, PreKeyBundle,
        PreKeyRecord, SessionRecord, SignedPreKeyRecord,
    };

    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("companion-test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([27; 32]))),
            create,
        })
        .unwrap()
    }

    async fn fixture(
        path: &std::path::Path,
        raw_peer: &str,
    ) -> (ProtocolStore, AccountRecord, Jid, Node) {
        let (store, account, peer, node, _, _) = fixture_payload(path, raw_peer, None).await;
        (store, account, peer, node)
    }
    type FixtureOutput = (
        ProtocolStore,
        AccountRecord,
        Jid,
        Node,
        AccountRecord,
        direct::PreparedDirect,
    );
    async fn fixture_payload(
        path: &std::path::Path,
        raw_peer: &str,
        payload: Option<&[u8]>,
    ) -> FixtureOutput {
        let store = open(path, true);
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.registration_id = 42;
        config.signed_pre_key_id = 7;
        crate::registration::factory::install_test_material(&store, &config).unwrap();
        let receiver = store.read(AccountRecord::load).unwrap().unwrap();
        let signed = crate::registration::factory::registered_signed_prekey(&store).unwrap();
        let signed = SignedPreKeyRecord::deserialize(&signed).unwrap();
        let one = KeyPair::generate(&mut rand::rng());
        let mut local_identity = vec![5];
        local_identity.extend(receiver.signal_identity.as_ref().unwrap().0.public);
        let bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            Some((11.into(), one.public_key)),
            7.into(),
            signed.public_key().unwrap(),
            signed.signature().unwrap().to_vec(),
            IdentityKey::decode(&local_identity).unwrap(),
        )
        .unwrap();
        let peer = Jid::parse(raw_peer).unwrap();
        let primary = IdentityKeyPair::generate(&mut rand::rng());
        let companion = IdentityKeyPair::generate(&mut rand::rng());
        let sender = AccountRecord {
            own_jid: Some(peer.to_non_ad()),
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: companion
                    .public_key()
                    .public_key_bytes()
                    .try_into()
                    .unwrap(),
                private: *companion.private_key().serialize(),
            })),
            ..Default::default()
        };
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let key = group::prepare_own_sender_key(&chat, &peer, None)
            .await
            .unwrap();
        let encrypted = direct::prepare_payload(
            &sender,
            receiver.own_jid.as_ref().unwrap(),
            Some(&bundle),
            None,
            None,
            payload.unwrap_or(&key.distribution_message),
        )
        .await
        .unwrap();
        assert_eq!(encrypted.kind, "pkmsg");
        // Literal GsB/Grz fields from 37.71, independently assembled.
        let details = [8, 7, 16, 9, 24, 2];
        let account_key = primary.public_key().public_key_bytes();
        let device_key = companion.public_key().public_key_bytes();
        let account_sig = primary
            .private_key()
            .calculate_signature(
                &[&[6, 0][..], &details, device_key].concat(),
                &mut rand::rng(),
            )
            .unwrap();
        let device_sig = companion
            .private_key()
            .calculate_signature(
                &[&[6, 1][..], &details, device_key, account_key].concat(),
                &mut rand::rng(),
            )
            .unwrap();
        let mut adv = vec![10, 6];
        adv.extend(details);
        adv.extend([18, 32]);
        adv.extend(account_key);
        adv.extend([26, 64]);
        adv.extend(account_sig);
        adv.extend([34, 64]);
        adv.extend(device_sig);
        let list = DeviceList {
            owner: peer.to_non_ad(),
            devices: vec![
                super::super::devices::DeviceEntry {
                    jid: peer.to_non_ad(),
                    key_index: 0,
                },
                super::super::devices::DeviceEntry {
                    jid: peer.clone(),
                    key_index: 2,
                },
            ],
            raw_id: Some(7),
            list_timestamp: Some(9),
            expected_timestamp: None,
            companions_verified: true,
            adv_added_devices: Vec::new(),
            verified_primary_key_fingerprint: Some(hex::encode(Sha256::digest(account_key))),
            unverified_reason: None,
            notified_hash: None,
            hash_matches: None,
            hosted_skipped: 0,
            fetched_at: Timestamp::now(),
        };
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{}", list.owner).as_bytes(),
                    1,
                    &primary.identity_key().serialize().to_vec(),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    list_key(&list.owner).as_bytes(),
                    1,
                    &list,
                )?;
                tx.put_json(
                    Table::Prekeys,
                    b"comparison-prekey:11",
                    1,
                    &PreKeyRecord::new(11.into(), &one).serialize().unwrap(),
                )?;
                tx.put_json(Table::Checkpoints, b"login-identity", 1, &serde_json::json!({
                "pn": receiver.own_jid.as_ref().unwrap().to_string(), "lid": "99990000100@lid",
            }))
            })
            .unwrap();
        let node = Node::new("message")
            .attr("from", chat)
            .attr("participant", peer.clone())
            .attr("id", "COMPANION-KEY")
            .children(vec![
                Node::new("enc")
                    .attr("type", "pkmsg")
                    .attr("v", "2")
                    .bytes(encrypted.ciphertext.to_vec()),
                Node::new("device-identity").bytes(adv),
            ]);
        (store, receiver, peer, node, sender, encrypted)
    }

    fn ciphertext(node: &Node) -> &[u8] {
        let NodeContent::Bytes(bytes) = &node.child_nodes()[0].content else {
            panic!()
        };
        bytes
    }
    type CryptoRows = Vec<Vec<(Vec<u8>, Vec<u8>)>>;
    fn crypto_rows(store: &ProtocolStore) -> CryptoRows {
        store
            .read(|tx| {
                [
                    Table::Prekeys,
                    Table::SignalSessions,
                    Table::PeerIdentities,
                    Table::SenderKeys,
                ]
                .into_iter()
                .map(|table| tx.scan(table, 1, 100))
                .collect()
            })
            .unwrap()
    }

    #[tokio::test]
    async fn newer_known_adv_refresh_commits_only_with_authenticated_message() {
        for (direct, corrupt, membership, raw) in [false, true].into_iter().flat_map(|direct| {
            [false, true].into_iter().flat_map(move |corrupt| {
                (0..3).flat_map(move |membership| {
                    ["15555550101:3@s.whatsapp.net", "99990000101:3@lid"]
                        .into_iter()
                        .map(move |raw| (direct, corrupt, membership, raw))
                })
            })
        }) {
            let dir = tempfile::tempdir().unwrap();
            let (store, receiver, peer, mut node) =
                fixture(&dir.path().join("refresh.redb"), raw).await;
            if direct {
                node = Node::new("message")
                    .attr("from", peer.clone())
                    .attr("id", "DIRECT-REFRESH")
                    .children(node.child_nodes().to_vec());
            }
            let mut baseline = super::super::devices::device_list(&store, &peer)
                .unwrap()
                .unwrap();
            baseline.list_timestamp = Some(8);
            if membership == 1 {
                baseline.devices.retain(|d| d.jid.device == 0);
            } else if membership == 2 {
                baseline.devices[1].key_index = 3;
            }
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        list_key(&baseline.owner).as_bytes(),
                        1,
                        &baseline,
                    )
                })
                .unwrap();
            if corrupt {
                let NodeContent::Children(children) = &mut node.content else {
                    panic!()
                };
                let NodeContent::Bytes(bytes) = &mut children[0].content else {
                    panic!()
                };
                let mut tampered = bytes.to_vec();
                *tampered.last_mut().unwrap() ^= 1;
                *bytes = tampered.into();
            }
            let rows = crypto_rows(&store);
            assert_eq!(
                inspect(&store, &receiver, &peer, &node, ciphertext(&node)).unwrap(),
                "ready_refresh_pending"
            );
            assert_eq!(
                super::super::devices::device_list(&store, &peer).unwrap(),
                Some(baseline.clone())
            );
            assert!(
                super::super::devices::pending_users(&store)
                    .unwrap()
                    .is_empty()
            );
            // Inject an abort after the refresh write: authority, queue and its
            // revision must share the enclosing receive transaction's rollback.
            let Admission::Ready(authority) =
                prepare(&store, &receiver, &peer, &node, ciphertext(&node)).unwrap()
            else {
                panic!()
            };
            assert!(
                store
                    .write::<()>(|tx| {
                        authority.commit(tx)?;
                        Err(Error::invalid("test", "abort after refresh"))
                    })
                    .is_err()
            );
            assert_eq!(
                super::super::devices::device_list(&store, &peer).unwrap(),
                Some(baseline.clone())
            );
            assert!(
                super::super::devices::pending_users(&store)
                    .unwrap()
                    .is_empty()
            );
            let capture = super::super::inbox::preserve(&store, &node).unwrap();
            let result = super::super::inbox::process(&store, &receiver, &node, &capture)
                .await
                .unwrap();
            if corrupt {
                assert_eq!(result, super::super::inbox::IncomingResult::Undecryptable);
                assert_eq!(crypto_rows(&store), rows);
                assert_eq!(
                    super::super::devices::device_list(&store, &peer).unwrap(),
                    Some(baseline)
                );
                assert!(
                    super::super::devices::pending_users(&store)
                        .unwrap()
                        .is_empty()
                );
            } else {
                assert_eq!(result, super::super::inbox::IncomingResult::Decrypted);
                let mut expected = baseline;
                expected.expected_timestamp = Some(9);
                if membership == 1 {
                    expected.devices.push(super::super::devices::DeviceEntry {
                        jid: peer.clone(),
                        key_index: 2,
                    });
                    expected.devices.sort_by(|a, b| a.jid.cmp(&b.jid));
                    expected.adv_added_devices = vec![peer.clone()];
                    expected.hash_matches = None;
                }
                assert_eq!(
                    super::super::devices::device_list(&store, &peer).unwrap(),
                    Some(expected.clone())
                );
                assert_eq!(
                    super::super::devices::pending_users(&store).unwrap(),
                    vec![peer.to_non_ad()]
                );
                drop(store);
                let store = open(&dir.path().join("refresh.redb"), false);
                assert_eq!(
                    super::super::devices::device_list(&store, &peer).unwrap(),
                    Some(expected)
                );
                assert_eq!(
                    super::super::inbox::process(&store, &receiver, &node, &capture)
                        .await
                        .unwrap(),
                    super::super::inbox::IncomingResult::Duplicate
                );
            }
        }
    }

    #[tokio::test]
    async fn known_companion_prekey_bootstraps_exact_session_and_control_once() {
        for raw in ["15555550101:3@s.whatsapp.net", "99990000101:3@lid"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("receive.redb");
            let (store, receiver, peer, node) = fixture(&path, raw).await;
            if peer.server == Server::Lid {
                let mut list = super::super::devices::device_list(&store, &peer)
                    .unwrap()
                    .unwrap();
                list.list_timestamp = Some(10); // An older ADV with the same index still qualifies.
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::Checkpoints,
                            list_key(&list.owner).as_bytes(),
                            1,
                            &list,
                        )
                    })
                    .unwrap();
            }
            assert_eq!(
                inspect(&store, &receiver, &peer, &node, ciphertext(&node)).unwrap(),
                "ready"
            );
            let list = super::super::devices::device_list(&store, &peer).unwrap();
            let capture = super::super::inbox::preserve(&store, &node).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &node, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Decrypted
            );
            assert_eq!(
                super::super::receipts::queue_read_for_incoming(&store, &node).unwrap(),
                0
            );
            store
                .read(|tx| {
                    assert!(
                        tx.get_json::<Vec<u8>>(
                            Table::SignalSessions,
                            format!("comparison-direct:{peer}").as_bytes(),
                            1
                        )?
                        .is_some()
                    );
                    assert!(
                        tx.get_json::<Vec<u8>>(
                            Table::SignalSessions,
                            format!("comparison-direct:{}", peer.to_non_ad()).as_bytes(),
                            1
                        )?
                        .is_none()
                    );
                    assert!(
                        tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:11", 1)?
                            .is_none()
                    );
                    assert_eq!(tx.scan(Table::SenderKeys, 1, 100)?.len(), 1);
                    let marker: serde_json::Value = tx
                        .get_json(
                            Table::Checkpoints,
                            b"received:120363000000000001@g.us:COMPANION-KEY",
                            1,
                        )?
                        .unwrap();
                    assert_eq!(marker["status"], "sender_key_distributed");
                    Ok(())
                })
                .unwrap();
            assert_eq!(
                super::super::devices::device_list(&store, &peer).unwrap(),
                list
            );
            let rows = crypto_rows(&store);
            drop(store);
            let store = open(&path, false);
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &node, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Duplicate
            );
            assert_eq!(crypto_rows(&store), rows);
        }
    }

    #[tokio::test]
    async fn signatures_do_not_replace_nested_mac_and_pinned_policy() {
        for (changed_pin, direct_route) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            let dir = tempfile::tempdir().unwrap();
            let (store, receiver, peer, mut node) = fixture(
                &dir.path().join("refuse.redb"),
                "15555550101:3@s.whatsapp.net",
            )
            .await;
            if direct_route {
                node = Node::new("message")
                    .attr("from", peer.clone())
                    .attr("id", "DIRECT-REFUSAL")
                    .children(node.child_nodes().to_vec());
            }
            if changed_pin {
                let old = IdentityKeyPair::generate(&mut rand::rng())
                    .identity_key()
                    .serialize()
                    .to_vec();
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::PeerIdentities,
                            format!("comparison-direct:{peer}").as_bytes(),
                            1,
                            &old,
                        )?;
                        tx.put_json(
                            Table::SignalSessions,
                            format!("comparison-direct:{peer}").as_bytes(),
                            1,
                            &SessionRecord::new_fresh().serialize().unwrap(),
                        )
                    })
                    .unwrap();
            } else {
                let NodeContent::Children(children) = &mut node.content else {
                    panic!()
                };
                let NodeContent::Bytes(bytes) = &mut children[0].content else {
                    panic!()
                };
                let mut tampered = bytes.to_vec();
                *tampered.last_mut().unwrap() ^= 1;
                *bytes = tampered.into();
            }
            assert_eq!(
                inspect(&store, &receiver, &peer, &node, ciphertext(&node)).unwrap(),
                "ready"
            );
            let before = crypto_rows(&store);
            let capture = super::super::inbox::preserve(&store, &node).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &node, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Undecryptable
            );
            assert_eq!(crypto_rows(&store), before);
            if changed_pin {
                assert_eq!(
                    super::super::inbox::process_with_policy(
                        &store,
                        &receiver,
                        &node,
                        &capture,
                        IdentityChangePolicy::ApkAuthenticated
                    )
                    .await
                    .unwrap(),
                    super::super::inbox::IncomingResult::Decrypted
                );
                assert_eq!(
                    super::super::identity_conflicts::list(&store)
                        .unwrap()
                        .len(),
                    1
                );
            }
        }
    }

    #[tokio::test]
    async fn companion_authority_changes_roll_back_commit() {
        for changed in 0..4 {
            let dir = tempfile::tempdir().unwrap();
            let (store, receiver, peer, node) =
                fixture(&dir.path().join("cas.redb"), "99990000101:3@lid").await;
            let Admission::Ready(authority) =
                prepare(&store, &receiver, &peer, &node, ciphertext(&node)).unwrap()
            else {
                panic!()
            };
            store
                .write(|tx| match changed {
                    0 => {
                        let mut list = authority.list.clone();
                        list.devices.clear();
                        tx.put_json(
                            Table::Checkpoints,
                            list_key(&list.owner).as_bytes(),
                            1,
                            &list,
                        )
                    }
                    1 => tx
                        .delete(
                            Table::PeerIdentities,
                            format!("comparison-direct:{}", peer.to_non_ad()).as_bytes(),
                        )
                        .map(|_| ()),
                    2 => tx.put_json(
                        Table::Checkpoints,
                        b"login-identity",
                        1,
                        &serde_json::json!({"lid": peer.to_non_ad().to_string()}),
                    ),
                    _ => {
                        let mut account = receiver.clone();
                        account.registration = RegistrationStatus::Revoked;
                        account.save(tx)
                    }
                })
                .unwrap();
            let before = crypto_rows(&store);
            assert!(
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::SignalSessions,
                            b"companion-cas-sentinel",
                            1,
                            &vec![1u8],
                        )?;
                        authority.check(tx)
                    })
                    .is_err()
            );
            assert_eq!(crypto_rows(&store), before);
        }
    }

    #[tokio::test]
    async fn companion_refusals_preserve_crypto_and_report_missing_authority() {
        for (case, expected) in [
            (0, "deleted_device"),
            (1, "deleted_device"),
            (2, "raw_id_mismatch"),
            (3, "offline_device_update_deferred"),
            (4, "stale_device_list"),
            (5, "own_device"),
            (6, "missing_own_identity"),
            (7, "missing_device_identity"),
            (8, "invalid_device_identity"),
            (9, "conflicting_device_membership"),
            (10, "ambiguous_device_membership"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let (store, receiver, peer, mut node) =
                fixture(&dir.path().join("negative.redb"), "99990000101:3@lid").await;
            let mut list = super::super::devices::device_list(&store, &peer)
                .unwrap()
                .unwrap();
            match case {
                0 => list.devices.retain(|d| d.jid.device == 0),
                1 => list.devices[1].key_index = 3,
                2 => list.raw_id = Some(8),
                3 => {
                    list.list_timestamp = Some(8);
                    list.devices.retain(|d| d.jid.device == 0);
                    node = node.attr("offline", "0");
                }
                9 => {
                    list.list_timestamp = Some(8);
                    list.devices.retain(|d| d.jid.device == 0);
                    list.devices.push(super::super::devices::DeviceEntry {
                        jid: peer.with_device(4),
                        key_index: 2,
                    });
                }
                10 => list.devices.push(list.devices[1].clone()),
                4 => list.verified_primary_key_fingerprint = None,
                5 => store
                    .write(|tx| {
                        tx.put_json(
                            Table::Checkpoints,
                            b"login-identity",
                            1,
                            &serde_json::json!({"lid": peer.to_non_ad().to_string()}),
                        )
                    })
                    .unwrap(),
                6 => {
                    store
                        .write(|tx| tx.delete(Table::Checkpoints, b"login-identity"))
                        .unwrap();
                }
                7 => {
                    let NodeContent::Children(children) = &mut node.content else {
                        panic!()
                    };
                    children.pop();
                }
                _ => {
                    let NodeContent::Children(children) = &mut node.content else {
                        panic!()
                    };
                    let NodeContent::Bytes(bytes) = &mut children[1].content else {
                        panic!()
                    };
                    let mut changed = bytes.to_vec();
                    *changed.last_mut().unwrap() ^= 1;
                    *bytes = changed.into();
                }
            }
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        list_key(&list.owner).as_bytes(),
                        1,
                        &list,
                    )
                })
                .unwrap();
            assert_eq!(
                inspect(&store, &receiver, &peer, &node, ciphertext(&node)).unwrap(),
                expected
            );
            let before = crypto_rows(&store);
            let capture = super::super::inbox::preserve(&store, &node).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &node, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Unsupported
            );
            assert_eq!(crypto_rows(&store), before);
            assert_eq!(
                super::super::devices::device_list(&store, &peer).unwrap(),
                Some(list)
            );
        }
    }

    #[tokio::test]
    async fn refused_companion_prekey_does_not_block_held_sender_key_payload() {
        let dir = tempfile::tempdir().unwrap();
        let (store, receiver, peer, mut node) = fixture(
            &dir.path().join("fallback.redb"),
            "15555550101:3@s.whatsapp.net",
        )
        .await;
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let sender_key = group::prepare_own_sender_key(&chat, &peer, None)
            .await
            .unwrap();
        let (_, distribution) =
            group::extract_skdm_from_payload(&sender_key.distribution_message).unwrap();
        let applied = group::process_skdm(
            &chat,
            &peer,
            group::sender_key_db_key(&chat, &peer),
            None,
            &distribution,
        )
        .await
        .unwrap();
        let (encrypted, _) =
            group::encrypt_group_payload(&chat, &peer, &sender_key.updated_key, b"\x0a\x04test")
                .await
                .unwrap();
        store
            .write(|tx| {
                group::commit_skdm(tx, &applied)?;
                tx.delete(Table::Checkpoints, list_key(&peer.to_non_ad()).as_bytes())?;
                Ok(())
            })
            .unwrap();
        let NodeContent::Children(children) = &mut node.content else {
            panic!()
        };
        children.push(
            Node::new("enc")
                .attr("type", "skmsg")
                .attr("v", "2")
                .bytes(encrypted.to_vec()),
        );
        let before = crypto_rows(&store);
        let capture = super::super::inbox::preserve(&store, &node).unwrap();
        assert_eq!(
            super::super::inbox::process(&store, &receiver, &node, &capture)
                .await
                .unwrap(),
            super::super::inbox::IncomingResult::Decrypted
        );
        let after = crypto_rows(&store);
        assert_eq!(&before[..3], &after[..3]); // No prekey/session/pin adoption.
        assert_ne!(before[3], after[3]); // Only the authenticated sender chain advanced.
    }

    #[tokio::test]
    async fn direct_companion_prekey_and_followup_keep_chat_and_device_separate() {
        for raw in ["15555550101:3@s.whatsapp.net", "99990000101:3@lid"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("direct.redb");
            let (store, receiver, peer, group_node, sending, encrypted) =
                fixture_payload(&path, raw, Some(b"\x0a\x04test")).await;
            let mut node = Node::new("message")
                .attr("from", raw)
                .attr("id", "DIRECT-COMPANION")
                .children(group_node.child_nodes().to_vec());
            if peer.server == Server::Lid {
                node = node.attr("sender_pn", "15555550101@s.whatsapp.net");
            }
            let chat = super::super::inbox::session_sender(&node).unwrap();
            assert_eq!(chat.device, 0);
            let capture = super::super::inbox::preserve(&store, &node).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &node, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Decrypted
            );
            assert_eq!(
                super::super::receipts::queue_read_for_incoming(&store, &node).unwrap(),
                1
            );
            let peer_key = format!("comparison-direct:{peer}");
            let (session, identity) = store
                .read(|tx| {
                    Ok((
                        tx.get_json::<Vec<u8>>(Table::SignalSessions, peer_key.as_bytes(), 1)?
                            .unwrap(),
                        tx.get_json::<Vec<u8>>(Table::PeerIdentities, peer_key.as_bytes(), 1)?
                            .unwrap(),
                    ))
                })
                .unwrap();
            let reply = direct::prepare_payload(
                &receiver,
                &peer,
                None,
                Some(&session),
                Some(&identity),
                b"\x0a\x04ack!",
            )
            .await
            .unwrap();
            let acknowledged = direct::receive(
                &sending,
                receiver.own_jid.as_ref().unwrap(),
                &encrypted.session,
                &encrypted.identity,
                &reply.ciphertext,
            )
            .await
            .unwrap();
            let followup = direct::prepare_payload(
                &sending,
                receiver.own_jid.as_ref().unwrap(),
                None,
                Some(&acknowledged.session),
                Some(&encrypted.identity),
                b"\x0a\x04next",
            )
            .await
            .unwrap();
            assert_eq!(followup.kind, "msg");
            let second = Node::new("message")
                .attr("from", peer.clone())
                .attr("id", "DIRECT-NEXT")
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "msg")
                        .bytes(followup.ciphertext.to_vec()),
                ]);
            let capture2 = super::super::inbox::preserve(&store, &second).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &second, &capture2)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Decrypted
            );
            store
                .read(|tx| {
                    assert!(
                        tx.get_json::<Vec<u8>>(
                            Table::SignalSessions,
                            format!("comparison-direct:{}", peer.to_non_ad()).as_bytes(),
                            1
                        )?
                        .is_none()
                    );
                    assert!(
                        tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:11", 1)?
                            .is_none()
                    );
                    if peer.server == Server::Lid {
                        assert_eq!(
                            tx.get_json::<Jid>(
                                Table::Checkpoints,
                                format!("authenticated-lid:{}", peer.to_non_ad()).as_bytes(),
                                1
                            )?,
                            Some(Jid::parse("15555550101@s.whatsapp.net").unwrap())
                        );
                        assert!(
                            tx.get_json::<Jid>(
                                Table::Checkpoints,
                                format!("authenticated-lid:{peer}").as_bytes(),
                                1
                            )?
                            .is_none()
                        );
                    }
                    Ok(())
                })
                .unwrap();
            let consumer = crate::store::ConsumerId::new("direct-test").unwrap();
            let events = store
                .read_events(&consumer, crate::model::ids::EventCursor::START, 20)
                .unwrap()
                .events;
            let messages: Vec<_> = events
                .iter()
                .filter_map(|event| match &event.event {
                    crate::sync::SyncEvent::NewMessage(message) => Some(message),
                    _ => None,
                })
                .collect();
            assert_eq!(messages.len(), 2);
            assert_eq!(
                messages[0].key.sender.as_ref().unwrap(),
                &crate::model::ids::DeviceAddress::from_jid(&peer).unwrap()
            );
            assert_eq!(messages[0].key.chat.jid(), &chat);
            let confirmation = super::super::messaging::received_confirmation(&node).unwrap();
            assert_eq!(
                super::super::receipts::jid(&confirmation, "to"),
                Some(peer.clone())
            );
            let mut no_alias = node.clone();
            no_alias
                .attributes
                .retain(|attribute| attribute.name != "sender_pn");
            let alias_capture = super::super::inbox::preserve(&store, &no_alias).unwrap();
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &no_alias, &alias_capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Duplicate
            );
            let rows = crypto_rows(&store);
            drop(store);
            let store = open(&path, false);
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &node, &capture)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Duplicate
            );
            assert_eq!(
                super::super::inbox::process(&store, &receiver, &second, &capture2)
                    .await
                    .unwrap(),
                super::super::inbox::IncomingResult::Duplicate
            );
            assert_eq!(crypto_rows(&store), rows);
        }
    }

    #[tokio::test]
    async fn direct_companion_distribution_waits_for_authority_and_keeps_sender_device() {
        let dir = tempfile::tempdir().unwrap();
        let (store, receiver, peer, original) =
            fixture(&dir.path().join("direct-key.redb"), "99990000101:3@lid").await;
        let node = Node::new("message")
            .attr("from", peer.clone())
            .attr("id", "DIRECT-KEY")
            .children(original.child_nodes().to_vec());
        let list = super::super::devices::device_list(&store, &peer)
            .unwrap()
            .unwrap();
        store
            .write(|tx| tx.delete(Table::Checkpoints, list_key(&list.owner).as_bytes()))
            .unwrap();
        let before = crypto_rows(&store);
        let capture = super::super::inbox::preserve(&store, &node).unwrap();
        assert_eq!(
            super::super::inbox::process(&store, &receiver, &node, &capture)
                .await
                .unwrap(),
            super::super::inbox::IncomingResult::Unsupported
        );
        assert_eq!(crypto_rows(&store), before);
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    list_key(&list.owner).as_bytes(),
                    1,
                    &list,
                )
            })
            .unwrap();
        assert_eq!(
            super::super::inbox::process(&store, &receiver, &node, &capture)
                .await
                .unwrap(),
            super::super::inbox::IncomingResult::Decrypted
        );
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        store
            .read(|tx| {
                assert!(group::load_sender_key_bytes(tx, &chat, &peer)?.1.is_some());
                assert!(
                    group::load_sender_key_bytes(tx, &chat, &peer.to_non_ad())?
                        .1
                        .is_none()
                );
                Ok(())
            })
            .unwrap();
        assert_eq!(
            super::super::receipts::queue_read_for_incoming(&store, &node).unwrap(),
            0
        );
    }
}
