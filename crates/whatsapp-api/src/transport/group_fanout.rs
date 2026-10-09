//! Shared staged sender-key fanout used by the comparison group/status sender.
//! APK 37.71 SendE2EMessageJob.A06 carries pairwise distributions and skmsg;
//! D4x.A00 marks exposed own keys for rotation after primary identity changes.
//! Preparation is memory-only; reserve_in must share the caller's send transaction.
use super::{group_send::FanoutPlan, groups::FanoutTarget, sender_key_distribution};
use crate::{
    Error, Result,
    binary::Node,
    model::jid::{Jid, Server},
    session::AccountRecord,
    signal::{direct, group},
    store::{ProtocolStore, Table, WriteTx},
};
use zeroize::Zeroizing;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("group_send", reason)
}
fn session_key(device: &Jid) -> String {
    format!("comparison-direct:{device}")
}
struct Target {
    device: Jid,
    // Always retain the actual held ratchet, even when fresh encryption is selected.
    session: Option<Zeroizing<Vec<u8>>>,
    identity: Option<Vec<u8>>,
}
struct RoutingRow {
    table: Table,
    key: Vec<u8>,
    value: Option<Zeroizing<Vec<u8>>>,
}

/// Private original state for optimistic concurrency. Does not adopt keys,
/// change a ratchet, queue a query or reserve a send. Fresh mode affects only
/// encryption input; it never discards the original concurrency snapshot.
pub struct Snapshot {
    chat: Jid,
    own: Jid,
    targets: Vec<Target>,
    own_key: Option<Zeroizing<Vec<u8>>>,
    distribution: Option<sender_key_distribution::DistributionState>,
    fresh_sessions: bool,
    routing: Vec<RoutingRow>,
}
impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupFanoutSnapshot")
            .field("target_count", &self.targets.len())
            .field("fresh_sessions", &self.fresh_sessions)
            .finish_non_exhaustive()
    }
}
impl Snapshot {
    /// Revalidate the supplied plan and capture routing and crypto state in one
    /// read transaction. Missing or stale plans fail before crypto or queries.
    pub fn load(store: &ProtocolStore, plan: &FanoutPlan, fresh_sessions: bool) -> Result<Self> {
        if !plan.without_device_list.is_empty() {
            return Err(invalid("some members have no device list"));
        }
        store.read(|tx| {
            let current = match plan.chat.server {
                Server::Group => super::group_send::plan_in(tx, &plan.chat, &plan.own)?,
                Server::Broadcast if plan.chat.user == "status" => {
                    super::group_send::plan_status_in(tx, &plan.own, &plan.recipients)?
                }
                _ => return Err(invalid("unsupported fanout chat")),
            };
            if current != *plan {
                return Err(invalid("fanout plan changed before preparation"));
            }
            // APK 37.71 SendE2EMessageJob.A06 (1300) carries the participant
            // hash; 1jL.A0N/A0O derive it from stored devices. Freezing the rows
            // authorizing our target set is a local concurrency contract.
            let mut keys = Vec::new();
            if plan.chat.server == Server::Group {
                keys.push((
                    Table::Checkpoints,
                    super::groups::key(&plan.chat.to_string()),
                ));
            }
            for recipient in &plan.recipients {
                keys.push((Table::Checkpoints, super::devices::list_key(recipient)));
                // Companion authority depends on the primary pin even if the
                // saved device list does not address that primary itself.
                keys.push((Table::PeerIdentities, session_key(recipient)));
            }
            let routing = keys
                .into_iter()
                .map(|(table, key)| {
                    Ok(RoutingRow {
                        table,
                        value: tx.get(table, key.as_bytes(), 1)?.map(Zeroizing::new),
                        key: key.into_bytes(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let targets = plan
                .devices
                .iter()
                .map(|device| {
                    let key = session_key(device);
                    Ok(Target {
                        device: device.clone(),
                        session: tx
                            .get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?
                            .map(Zeroizing::new),
                        identity: tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Self {
                chat: plan.chat.clone(),
                own: plan.own.clone(),
                targets,
                own_key: tx
                    .get_json::<Vec<u8>>(
                        Table::SenderKeys,
                        &group::sender_key_db_key(&plan.chat, &plan.own),
                        1,
                    )?
                    .map(Zeroizing::new),
                distribution: sender_key_distribution::load_in(tx, &plan.chat, &plan.own)?,
                fresh_sessions,
                routing,
            })
        })
    }
    /// Actual devices needing directory bundles. This list creates no query.
    pub fn required_keys(&self) -> Vec<Jid> {
        self.targets
            .iter()
            .filter(|t| self.fresh_sessions || t.session.is_none())
            .map(|t| t.device.clone())
            .collect()
    }
    /// Stage pairwise distributions and the encrypted group body entirely in
    /// memory. Pins stay enforced in fresh mode; no policy adopts a changed key.
    /// The reply must already be authenticated and correlated by the caller.
    pub async fn prepare(
        self,
        account: &AccountRecord,
        reply: Option<&Node>,
        payload: &[u8],
    ) -> Result<Prepared> {
        let rotate = self
            .distribution
            .as_ref()
            .is_some_and(|s| s.rotation_pending());
        let minted = group::prepare_own_sender_key(
            &self.chat,
            &self.own,
            if rotate {
                None
            } else {
                self.own_key.as_deref().map(|v| v.as_slice())
            },
        )
        .await?;
        let mut fanout = Vec::with_capacity(self.targets.len());
        let mut updates = Vec::with_capacity(self.targets.len());
        for target in &self.targets {
            let session = if self.fresh_sessions {
                None
            } else {
                target.session.as_deref().map(|v| v.as_slice())
            };
            let bundle = if session.is_none() {
                Some(direct::prekey_bundle_for(
                    reply.ok_or_else(|| invalid("a device has neither a session nor a bundle"))?,
                    &target.device,
                )?)
            } else {
                None
            };
            // Bare protobuf input: prepare_payload applies APK padding exactly once.
            let prepared = direct::prepare_payload(
                account,
                &target.device,
                bundle.as_ref(),
                session,
                target.identity.as_deref(),
                &minted.distribution_message,
            )
            .await?;
            fanout.push(FanoutTarget {
                device: target.device.clone(),
                kind: prepared.kind,
                ciphertext: prepared.ciphertext,
            });
            updates.push((prepared.session, prepared.identity));
        }
        let (ciphertext, own_key) =
            group::encrypt_group_payload(&self.chat, &self.own, &minted.updated_key, payload)
                .await?;
        Ok(Prepared {
            original: self,
            fanout,
            updates,
            ciphertext,
            distribution_message: minted.distribution_message,
            own_key,
            rotate,
        })
    }
}

/// Staged ciphertext and next crypto state. Dropping it makes no persistent change.
/// It must be reserved atomically with the original send intent before any write.
pub struct Prepared {
    original: Snapshot,
    fanout: Vec<FanoutTarget>,
    updates: Vec<(Zeroizing<Vec<u8>>, Vec<u8>)>,
    ciphertext: Zeroizing<Vec<u8>>,
    distribution_message: Zeroizing<Vec<u8>>,
    own_key: Zeroizing<Vec<u8>>,
    rotate: bool,
}
impl std::fmt::Debug for Prepared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedGroupFanout")
            .field("target_count", &self.fanout.len())
            .field("rotates_own_key", &self.rotate)
            .finish_non_exhaustive()
    }
}
impl Prepared {
    /// Pairwise distribution ciphertext for the frozen target devices.
    pub fn targets(&self) -> &[FanoutTarget] {
        &self.fanout
    }
    /// Sender-key encrypted message body; contains no plaintext.
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }
    /// Secret-bearing bare sender-key distribution for the caller's bounded
    /// original-send retry window. Keep private; never print or log these bytes.
    pub fn distribution_message(&self) -> &[u8] {
        &self.distribution_message
    }
    /// Validate every captured ratchet/pin/own-key/exposure snapshot and reserve
    /// all next state in the caller's send transaction. Fresh mode has the same
    /// concurrency protection as existing-session mode. No network occurs.
    /// The archive key must be unique to this original send when rotation occurs.
    pub fn reserve_in(&self, tx: &mut WriteTx<'_>, rotated_archive_key: &[u8]) -> Result<()> {
        for row in &self.original.routing {
            let held = tx.get(row.table, &row.key, 1)?;
            if held.as_deref() != row.value.as_deref().map(|v| v.as_slice()) {
                return Err(invalid(
                    "group membership or device authority changed before commit",
                ));
            }
        }
        let own_key_db = group::sender_key_db_key(&self.original.chat, &self.original.own);
        let held: Option<Vec<u8>> = tx.get_json(Table::SenderKeys, &own_key_db, 1)?;
        if held.as_deref() != self.original.own_key.as_deref().map(|v| v.as_slice()) {
            return Err(invalid("own sender key changed before commit"));
        }
        // Check every peer before staging writes, even though WriteTx also rolls
        // back a later error. Never let explicit fresh mode overwrite a receive.
        for target in &self.original.targets {
            let key = session_key(&target.device);
            let session: Option<Vec<u8>> = tx.get_json(Table::SignalSessions, key.as_bytes(), 1)?;
            let identity: Option<Vec<u8>> =
                tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?;
            if session.as_deref() != target.session.as_deref().map(|v| v.as_slice())
                || identity != target.identity
            {
                return Err(invalid("a peer session changed before commit"));
            }
        }
        sender_key_distribution::reserve_in(
            tx,
            &self.original.chat,
            &self.original.own,
            self.original.distribution.as_ref(),
            &self
                .fanout
                .iter()
                .map(|t| t.device.clone())
                .collect::<Vec<_>>(),
        )?;
        for (target, (session, identity)) in self.original.targets.iter().zip(&self.updates) {
            let key = session_key(&target.device);
            tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &**session)?;
            tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, identity)?;
        }
        if self.rotate {
            // Caller retains the same per-send archive selector; no old own
            // generation or uncertain reservation is discarded to retry.
            tx.put_json(
                Table::Checkpoints,
                rotated_archive_key,
                1,
                &self.original.own_key.as_deref().map(|v| v.as_slice()),
            )?;
        }
        tx.put_json(Table::SenderKeys, &own_key_db, 1, &*self.own_key)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::{
        model::{ids::AccountId, time::Timestamp},
        session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::{
        GenericSignedPreKey, IdentityKeyPair, KeyPair, SignedPreKeyRecord,
    };
    fn jid(s: &str) -> Jid {
        Jid::parse(s).unwrap()
    }
    pub(in crate::transport) fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.into(),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([8; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn account(identity: &IdentityKeyPair, own: &Jid, registration: u32) -> AccountRecord {
        AccountRecord {
            registration: RegistrationStatus::Registered,
            own_jid: Some(own.clone()),
            signal_registration_id: Some(registration),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: identity.public_key().public_key_bytes().try_into().unwrap(),
                private: *identity.private_key().serialize(),
            })),
            ..Default::default()
        }
    }
    pub(in crate::transport) struct Fixture {
        pub(in crate::transport) alice: AccountRecord,
        pub(in crate::transport) bobs: Vec<(AccountRecord, Vec<u8>)>,
        pub(in crate::transport) plan: FanoutPlan,
        pub(in crate::transport) reply: Node,
    }
    pub(in crate::transport) fn fixture() -> Fixture {
        let own = jid("15555550100@s.whatsapp.net");
        let alice = account(&IdentityKeyPair::generate(&mut rand::rng()), &own, 31);
        let devices = vec![
            jid("15555550101@s.whatsapp.net"),
            jid("15555550102:3@s.whatsapp.net"),
        ];
        let mut bobs = Vec::new();
        let mut users = Vec::new();
        for device in &devices {
            let identity = IdentityKeyPair::generate(&mut rand::rng());
            let signed = KeyPair::generate(&mut rand::rng());
            let signature = identity
                .private_key()
                .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
                .unwrap();
            let signed_bytes = SignedPreKeyRecord::new(
                7.into(),
                wacore_libsignal::protocol::Timestamp::from_epoch_millis(0),
                &signed,
                &signature,
            )
            .serialize()
            .unwrap();
            users.push(Node::new("user").attr("jid", device.clone()).children(vec![
                Node::new("registration").bytes(vec![0, 0, 0, 42]),
                Node::new("type").bytes(vec![5]),
                Node::new("identity").bytes(identity.public_key().public_key_bytes().to_vec()),
                Node::new("skey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 7]),
                    Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                    Node::new("signature").bytes(signature.to_vec()),
                ]),
            ]));
            bobs.push((account(&identity, device, 42), signed_bytes));
        }
        Fixture {
            alice,
            bobs,
            plan: FanoutPlan {
                chat: jid("120363000000000001@g.us"),
                addressing_mode: "pn",
                own,
                recipients: devices.iter().map(Jid::to_non_ad).collect(),
                devices,
                without_device_list: vec![],
                own_devices_unknown: true,
                participant_hash: super::super::groups::participant_hash_v2(&[
                    jid("15555550100@s.whatsapp.net"),
                    jid("15555550101@s.whatsapp.net"),
                    jid("15555550102:3@s.whatsapp.net"),
                ]),
                participant_hash_device_count: 3,
            },
            reply: Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(users)]),
        }
    }
    fn save_routing(store: &ProtocolStore, f: &Fixture) {
        use super::super::devices::{DeviceEntry, DeviceList};
        use sha2::Digest as _;
        let members = std::iter::once(&f.plan.own)
            .chain(f.plan.recipients.iter())
            .map(|j| Node::new("participant").attr("jid", j.clone()))
            .collect();
        let reply = Node::new("iq").attr("type", "result").children(vec![
            Node::new("group")
                .attr("id", f.plan.chat.clone())
                .children(members),
        ]);
        assert!(super::super::groups::record_info_reply(store, &reply).unwrap());
        store
            .write(|tx| {
                for device in &f.plan.devices {
                    let owner = device.to_non_ad();
                    let bundle = direct::prekey_bundle_for(&f.reply, device)?;
                    let pin = bundle.identity_key().unwrap().serialize().to_vec();
                    tx.put_json(
                        Table::PeerIdentities,
                        session_key(&owner).as_bytes(),
                        1,
                        &pin,
                    )?;
                    let list = DeviceList {
                        owner: owner.clone(),
                        devices: vec![DeviceEntry {
                            jid: device.clone(),
                            key_index: 0,
                        }],
                        raw_id: None,
                        list_timestamp: None,
                        expected_timestamp: None,
                        pending_removed_devices: None,
                        adv_added_devices: vec![],
                        companions_verified: true,
                        verified_primary_key_fingerprint: Some(hex::encode(sha2::Sha256::digest(
                            &pin[1..],
                        ))),
                        unverified_reason: None,
                        notified_hash: None,
                        hash_matches: None,
                        hosted_skipped: 0,
                        fetched_at: Timestamp::now(),
                    };
                    tx.put_json(
                        Table::Checkpoints,
                        super::super::devices::list_key(&owner).as_bytes(),
                        1,
                        &list,
                    )?;
                }
                Ok(())
            })
            .unwrap();
    }
    fn state(store: &ProtocolStore) -> Vec<Vec<(Vec<u8>, Vec<u8>)>> {
        store
            .read(|tx| {
                [
                    Table::SignalSessions,
                    Table::PeerIdentities,
                    Table::SenderKeys,
                    Table::Checkpoints,
                ]
                .into_iter()
                .map(|t| tx.scan(t, 1, 100))
                .collect()
            })
            .unwrap()
    }
    async fn seed(store: &ProtocolStore, f: &Fixture) {
        for device in &f.plan.devices {
            let bundle = direct::prekey_bundle_for(&f.reply, device).unwrap();
            let p = direct::prepare(&f.alice, device, &bundle, None, None, "prior fixture")
                .await
                .unwrap();
            store
                .write(|tx| {
                    tx.put_json(
                        Table::SignalSessions,
                        session_key(device).as_bytes(),
                        1,
                        &*p.session,
                    )?;
                    tx.put_json(
                        Table::PeerIdentities,
                        session_key(device).as_bytes(),
                        1,
                        &p.identity,
                    )
                })
                .unwrap();
        }
    }
    #[tokio::test]
    async fn staged_primary_and_companion_distributions_decrypt_without_double_padding_and_commit_once()
     {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("roundtrip.redb");
        let store = open(&path);
        let f = fixture();
        save_routing(&store, &f);
        let original = state(&store);
        let snapshot = Snapshot::load(&store, &f.plan, false).unwrap();
        assert_eq!(snapshot.required_keys(), f.plan.devices);
        let payload = super::super::group_send::text_payload("offline group fixture").unwrap();
        let prepared = snapshot
            .prepare(&f.alice, Some(&f.reply), &payload)
            .await
            .unwrap();
        assert_eq!(original, state(&store));
        for (target, (bob, signed)) in prepared.targets().iter().zip(&f.bobs) {
            assert_eq!(target.kind, "pkmsg");
            let decrypted = direct::receive_prekey(
                bob,
                &f.plan.own,
                (None, None),
                signed,
                None,
                &target.ciphertext,
            )
            .await
            .unwrap();
            let (chat, skdm) =
                group::extract_skdm_from_payload(&decrypted.received.plaintext).unwrap();
            assert_eq!(chat, f.plan.chat.to_string());
            let sk = group::process_skdm(&f.plan.chat, &f.plan.own, vec![], None, &skdm)
                .await
                .unwrap();
            let body = group::receive_group(
                &f.plan.chat,
                &f.plan.own,
                Some(&sk.updated_key),
                prepared.ciphertext(),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(&*body.plaintext, &payload);
        }
        store
            .write(|tx| {
                prepared.reserve_in(tx, b"archive:fixture")?;
                tx.put_json(
                    Table::Checkpoints,
                    b"message:fixture:send",
                    1,
                    &serde_json::json!({"status":"outcome_unknown","at":Timestamp::now()}),
                )
            })
            .unwrap();
        let committed = state(&store);
        assert_ne!(committed, original);
        assert!(
            store
                .write(|tx| prepared.reserve_in(tx, b"archive:fixture"))
                .is_err()
        );
        assert_eq!(committed, state(&store));
        drop(store);
        let store = open(&path);
        assert_eq!(committed, state(&store));
        let cached = Snapshot::load(&store, &f.plan, false).unwrap();
        assert!(cached.required_keys().is_empty());
        let next = cached.prepare(&f.alice, None, &payload).await.unwrap();
        assert_eq!(next.targets().len(), 2);
        assert_eq!(committed, state(&store));
        let debug = format!("{next:?}");
        assert!(
            !debug.contains("1555555")
                && !debug.contains("120363")
                && !debug.contains("offline group")
        );
    }
    #[tokio::test]
    async fn fresh_and_cached_staging_refuse_every_concurrent_crypto_change_and_preserve_late_writes()
     {
        for fresh in [false, true] {
            for changed in ["ratchet", "pin", "own", "exposure"] {
                let d = tempfile::tempdir().unwrap();
                let store = open(&d.path().join("race.redb"));
                let f = fixture();
                save_routing(&store, &f);
                seed(&store, &f).await;
                let snapshot = Snapshot::load(&store, &f.plan, fresh).unwrap();
                assert_eq!(snapshot.required_keys().len(), if fresh { 2 } else { 0 });
                let prepared = snapshot
                    .prepare(
                        &f.alice,
                        Some(&f.reply),
                        &super::super::group_send::text_payload("staged").unwrap(),
                    )
                    .await
                    .unwrap();
                let last = f.plan.devices.last().unwrap();
                if changed == "ratchet" {
                    let (session, identity): (Vec<u8>, Vec<u8>) = store
                        .read(|tx| {
                            Ok((
                                tx.get_json(
                                    Table::SignalSessions,
                                    session_key(last).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                                tx.get_json(
                                    Table::PeerIdentities,
                                    session_key(last).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                            ))
                        })
                        .unwrap();
                    let p = direct::prepare_payload(
                        &f.alice,
                        last,
                        None,
                        Some(&session),
                        Some(&identity),
                        &[10, 1, b'x'],
                    )
                    .await
                    .unwrap();
                    store
                        .write(|tx| {
                            tx.put_json(
                                Table::SignalSessions,
                                session_key(last).as_bytes(),
                                1,
                                &*p.session,
                            )
                        })
                        .unwrap();
                } else {
                    store
                        .write(|tx| match changed {
                            "pin" => tx.put_json(
                                Table::PeerIdentities,
                                session_key(last).as_bytes(),
                                1,
                                &vec![9u8; 32],
                            ),
                            "own" => tx.put_json(
                                Table::SenderKeys,
                                &group::sender_key_db_key(&f.plan.chat, &f.plan.own),
                                1,
                                &vec![9u8; 32],
                            ),
                            _ => sender_key_distribution::reserve_in(
                                tx,
                                &f.plan.chat,
                                &f.plan.own,
                                None,
                                std::slice::from_ref(last),
                            ),
                        })
                        .unwrap();
                }
                let newer = state(&store);
                let head = store.journal_head().unwrap();
                let error = store
                    .write(|tx| {
                        tx.put_json(Table::Checkpoints, b"caller-intent", 1, &true)?;
                        prepared.reserve_in(tx, b"archive:race")
                    })
                    .unwrap_err()
                    .to_string();
                assert!(error.contains("changed before commit"));
                assert!(!error.contains("1555555") && !error.contains("120363"));
                assert_eq!(newer, state(&store));
                assert_eq!(head, store.journal_head().unwrap());
            }
        }
    }
    #[tokio::test]
    async fn identity_rotation_archives_the_exact_previous_generation_and_retains_uncertain_intents()
     {
        let d = tempfile::tempdir().unwrap();
        let store = open(&d.path().join("rotation.redb"));
        let f = fixture();
        save_routing(&store, &f);
        let first = Snapshot::load(&store, &f.plan, false)
            .unwrap()
            .prepare(&f.alice, Some(&f.reply), &[10, 1, b'a'])
            .await
            .unwrap();
        store
            .write(|tx| {
                first.reserve_in(tx, b"archive:first")?;
                tx.put_json(
                    Table::Checkpoints,
                    b"old-uncertain-send",
                    1,
                    &"outcome_unknown",
                )
            })
            .unwrap();
        let previous: Vec<u8> = store
            .read(|tx| {
                tx.get_json(
                    Table::SenderKeys,
                    &group::sender_key_db_key(&f.plan.chat, &f.plan.own),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        store
            .write(|tx| sender_key_distribution::after_primary_change_in(tx, &f.plan.devices[0]))
            .unwrap();
        let snapshot = Snapshot::load(&store, &f.plan, false).unwrap();
        assert!(snapshot.distribution.as_ref().unwrap().rotation_pending());
        let prepared = snapshot
            .prepare(&f.alice, None, &[10, 1, b'b'])
            .await
            .unwrap();
        assert!(prepared.rotate);
        store
            .write(|tx| prepared.reserve_in(tx, b"archive:second"))
            .unwrap();
        let archived: Vec<u8> = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"archive:second", 1))
            .unwrap()
            .unwrap();
        assert_eq!(archived, previous);
        assert!(
            !store
                .read(|tx| sender_key_distribution::load_in(tx, &f.plan.chat, &f.plan.own))
                .unwrap()
                .unwrap()
                .rotation_pending()
        );
        assert_eq!(
            store
                .read(|tx| tx.get_json::<String>(Table::Checkpoints, b"old-uncertain-send", 1))
                .unwrap()
                .unwrap(),
            "outcome_unknown"
        );
    }
    #[tokio::test]
    async fn routing_changes_roll_back_the_entire_send_in_fresh_and_cached_modes() {
        for fresh in [false, true] {
            for changed in [
                "membership",
                "devices",
                "authority",
                "missing_group",
                "missing_list",
                "primary_pin",
            ] {
                let d = tempfile::tempdir().unwrap();
                let store = open(&d.path().join("routing-race.redb"));
                let f = fixture();
                save_routing(&store, &f);
                seed(&store, &f).await;
                let prepared = Snapshot::load(&store, &f.plan, fresh)
                    .unwrap()
                    .prepare(&f.alice, Some(&f.reply), &[10, 1, b'a'])
                    .await
                    .unwrap();
                let member = f.plan.recipients.last().unwrap();
                let group_key = super::super::groups::key(&f.plan.chat.to_string());
                let list_key = super::super::devices::list_key(member);
                store
                    .write(|tx| match changed {
                        "membership" => {
                            let mut group: super::super::groups::GroupSnapshot = tx
                                .get_json(Table::Checkpoints, group_key.as_bytes(), 1)?
                                .unwrap();
                            group.members.retain(|m| m.jid != member.to_string());
                            tx.put_json(Table::Checkpoints, group_key.as_bytes(), 1, &group)
                        }
                        "missing_group" => tx
                            .delete(Table::Checkpoints, group_key.as_bytes())
                            .map(|_| ()),
                        "missing_list" => tx
                            .delete(Table::Checkpoints, list_key.as_bytes())
                            .map(|_| ()),
                        "primary_pin" => tx.put_json(
                            Table::PeerIdentities,
                            session_key(member).as_bytes(),
                            1,
                            &vec![9u8; 33],
                        ),
                        _ => {
                            let mut list: super::super::devices::DeviceList = tx
                                .get_json(Table::Checkpoints, list_key.as_bytes(), 1)?
                                .unwrap();
                            if changed == "devices" {
                                list.devices.clear();
                            } else {
                                list.verified_primary_key_fingerprint = None;
                            }
                            tx.put_json(Table::Checkpoints, list_key.as_bytes(), 1, &list)
                        }
                    })
                    .unwrap();
                let newer = state(&store);
                let head = store.journal_head().unwrap();
                let error = store
                    .write(|tx| {
                        tx.put_json(
                            Table::Checkpoints,
                            b"caller-original-intent",
                            1,
                            &"outcome_unknown",
                        )?;
                        prepared.reserve_in(tx, b"archive:routing-race")
                    })
                    .unwrap_err()
                    .to_string();
                assert!(error.contains("membership or device authority changed"));
                assert!(!error.contains(&member.user) && !error.contains(&f.plan.chat.user));
                assert_eq!(state(&store), newer);
                assert_eq!(store.journal_head().unwrap(), head);
                drop(store);
                assert_eq!(state(&open(&d.path().join("routing-race.redb"))), newer);
            }
        }
    }

    #[tokio::test]
    async fn stale_or_invented_plans_fail_before_crypto_and_unrelated_updates_do_not_block() {
        let d = tempfile::tempdir().unwrap();
        let store = open(&d.path().join("plan-validation.redb"));
        let f = fixture();
        let empty = state(&store);
        assert!(Snapshot::load(&store, &f.plan, false).is_err());
        assert_eq!(state(&store), empty);
        save_routing(&store, &f);
        for field in ["hash", "devices", "recipients", "mode", "own_inventory"] {
            let mut forged = f.plan.clone();
            match field {
                "hash" => forged.participant_hash = "2:wrong".into(),
                "devices" => {
                    forged.devices.pop();
                }
                "recipients" => {
                    forged.recipients.pop();
                }
                "mode" => forged.addressing_mode = "lid",
                _ => forged.own_devices_unknown = false,
            }
            let before = state(&store);
            assert!(Snapshot::load(&store, &forged, false).is_err());
            assert_eq!(state(&store), before);
        }
        let mut status =
            super::super::group_send::plan_status(&store, &f.plan.own, &f.plan.recipients).unwrap();
        let prepared = Snapshot::load(&store, &status, false)
            .unwrap()
            .prepare(&f.alice, Some(&f.reply), &[10, 1, b's'])
            .await
            .unwrap();
        store
            .write(|tx| tx.put_json(Table::Checkpoints, b"unrelated-contact", 1, &true))
            .unwrap();
        store
            .write(|tx| prepared.reserve_in(tx, b"archive:status"))
            .unwrap();
        status.devices.pop();
        assert!(Snapshot::load(&store, &status, false).is_err());
        // Status has an explicit audience, so an unrelated group update is not
        // authority for it. Its audience lists and primary pins still are.
        let status =
            super::super::group_send::plan_status(&store, &f.plan.own, &f.plan.recipients).unwrap();
        let prepared = Snapshot::load(&store, &status, false)
            .unwrap()
            .prepare(&f.alice, None, &[10, 1, b't'])
            .await
            .unwrap();
        store
            .write(|tx| {
                tx.delete(
                    Table::Checkpoints,
                    super::super::devices::list_key(&f.plan.recipients[0]).as_bytes(),
                )
            })
            .unwrap();
        let before = state(&store);
        assert!(
            store
                .write(|tx| prepared.reserve_in(tx, b"archive:status-next"))
                .is_err()
        );
        assert_eq!(before, state(&store));
    }

    #[tokio::test]
    async fn fresh_mode_requires_real_bundles_and_never_adopts_a_changed_pin() {
        let d = tempfile::tempdir().unwrap();
        let store = open(&d.path().join("pin.redb"));
        let f = fixture();
        save_routing(&store, &f);
        seed(&store, &f).await;
        let before = state(&store);
        assert!(
            Snapshot::load(&store, &f.plan, true)
                .unwrap()
                .prepare(&f.alice, None, &[10, 1, b'x'])
                .await
                .is_err()
        );
        assert_eq!(before, state(&store));
        let other = fixture();
        assert!(
            Snapshot::load(&store, &f.plan, true)
                .unwrap()
                .prepare(&f.alice, Some(&other.reply), &[10, 1, b'x'])
                .await
                .is_err()
        );
        assert_eq!(before, state(&store));
        let mut incomplete = f.plan.clone();
        incomplete
            .without_device_list
            .push(f.plan.devices[0].clone());
        assert!(Snapshot::load(&store, &incomplete, false).is_err());
        assert_eq!(before, state(&store));
    }
}
