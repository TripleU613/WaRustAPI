//! Shared staged sender-key fanout used by the comparison group/status sender.
//! APK 37.71 SendE2EMessageJob.A06 carries pairwise distributions and skmsg;
//! D4x.A00 marks exposed own keys for rotation after primary identity changes.
//! Preparation is memory-only; reserve_in must share the caller's send transaction.
use super::{group_send::FanoutPlan, groups::FanoutTarget, sender_key_distribution};
use crate::{
    Error, Result,
    binary::Node,
    model::jid::{Jid, Server},
    registration::request::IdentityChangePolicy,
    session::AccountRecord,
    signal::{direct, group},
    store::{ProtocolStore, Table, WriteTx},
};
use zeroize::Zeroizing;

pub(super) const COMPANION_AUTHORITY_CHANGED: &str =
    "primary replacement requires refreshed companion authority";

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
            let mut keys = vec![
                (Table::Checkpoints, "login-identity".to_owned()),
                (
                    Table::Checkpoints,
                    String::from_utf8(super::group_send::OWN_LID_KEY.to_vec())
                        .expect("static ASCII"),
                ),
            ];
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
    /// Stage memory-only fanout under the default unchanged-pin policy.
    /// The reply must already be authenticated and correlated by the caller.
    pub async fn prepare(
        self,
        account: &AccountRecord,
        reply: Option<&Node>,
        payload: &[u8],
    ) -> Result<Prepared> {
        self.prepare_policy(None, account, reply, payload, IdentityChangePolicy::Pinned)
            .await
    }
    /// Stage a signed remote bundle replacement under the explicit outgoing
    /// policy. Authentication/correlation remain the actor's responsibility;
    /// no query, pin, ratchet or intent is changed until reserve_in commits.
    pub async fn prepare_with_policy(
        self,
        store: &ProtocolStore,
        account: &AccountRecord,
        reply: Option<&Node>,
        payload: &[u8],
        policy: IdentityChangePolicy,
    ) -> Result<Prepared> {
        self.prepare_policy(Some(store), account, reply, payload, policy)
            .await
    }
    async fn prepare_policy(
        self,
        store: Option<&ProtocolStore>,
        account: &AccountRecord,
        reply: Option<&Node>,
        payload: &[u8],
        policy: IdentityChangePolicy,
    ) -> Result<Prepared> {
        // APK 37.71 0jc.A0L invalidates a changed remote ratchet before
        // CQc.A00 verifies/derives the new signed bundle session. Keep originals
        // for OCC/history; policy never clears a persisted pin to make it pass.
        let mut inputs = Vec::with_capacity(self.targets.len());
        let mut hybrids = Vec::with_capacity(self.targets.len());
        let mut replacements = Vec::with_capacity(self.targets.len());
        for target in &self.targets {
            let (bundle, hybrid) = if self.fresh_sessions || target.session.is_none() {
                let reply =
                    reply.ok_or_else(|| invalid("a device has neither a session nor a bundle"))?;
                // Batch callbacks (Bot.A03) carry a selected Kyber key into the
                // device-agnostic CQc.A00, which verifies it and derives a
                // hybrid session (pq-bundle-boundary.md, hybrid-bundle.md).
                if crate::signal::hybrid::selects_pq_key(reply, &target.device)? {
                    let hybrid = crate::signal::hybrid::bundle_for(reply, &target.device)?;
                    (Some(hybrid.classic().clone()), Some(hybrid))
                } else {
                    (
                        Some(direct::prekey_bundle_for(reply, &target.device)?),
                        None,
                    )
                }
            } else {
                (None, None)
            };
            let replacement = match direct::validate_bundle_identity(
                bundle.as_ref(),
                target.identity.as_deref(),
                &target.device,
            ) {
                Ok(()) => false,
                Err(Error::IdentityChanged(device)) => {
                    let saved_own_lid: Option<Jid> = match store {
                        Some(store) => store.read(|tx| {
                            tx.get_json(Table::Checkpoints, super::group_send::OWN_LID_KEY, 1)
                        })?,
                        None => None,
                    };
                    if policy == IdentityChangePolicy::Pinned
                        || saved_own_lid.as_ref().map(Jid::to_non_ad).as_ref()
                            == Some(&target.device.to_non_ad())
                        || target.device.to_non_ad() == self.own.to_non_ad()
                        || super::inbox::remote_identity_policy(
                            store.expect("explicit policy store"),
                            account,
                            &target.device,
                            policy,
                        )? == IdentityChangePolicy::Pinned
                    {
                        return Err(Error::IdentityChanged(device));
                    }
                    direct::validate_bundle_signature(bundle.as_ref().expect("changed bundle"))?;
                    // ADV companion authority was established under the old
                    // primary pin. 37.71 0jc.A0L / 11T.A08 require reconciliation;
                    // a self-signed primary bundle does not reauthorize that list.
                    if target.device.device == 0
                        && target.device.agent == 0
                        && self.targets.iter().any(|other| {
                            other.device.to_non_ad() == target.device.to_non_ad()
                                && (other.device.device != 0 || other.device.agent != 0)
                        })
                    {
                        return Err(invalid(COMPANION_AUTHORITY_CHANGED));
                    }
                    true
                }
                Err(error) => return Err(error),
            };
            replacements.push(replacement);
            inputs.push(bundle);
            hybrids.push(hybrid);
        }
        let changed_primaries: Vec<_> = self
            .targets
            .iter()
            .zip(&replacements)
            .filter(|(_, changed)| **changed)
            .map(|(t, _)| t.device.clone())
            .collect();
        let rotate = self.distribution.as_ref().is_some_and(|s| {
            s.rotation_pending()
                || s.after_primary_changes(&changed_primaries)
                    .rotation_pending()
        });
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
        for (((target, bundle), replacement), hybrid) in self
            .targets
            .iter()
            .zip(&inputs)
            .zip(&replacements)
            .zip(&hybrids)
        {
            let session = if self.fresh_sessions || *replacement {
                None
            } else {
                target.session.as_deref().map(|v| v.as_slice())
            };
            let identity = if *replacement {
                None
            } else {
                target.identity.as_deref()
            };
            // Bare protobuf input: both preparers apply APK padding once. A
            // fetched hybrid bundle exists only when no held ratchet is used.
            let prepared = match hybrid {
                Some(hybrid) if session.is_none() => {
                    hybrid.prepare_initial(account, identity, &minted.distribution_message)?
                }
                _ => {
                    direct::prepare_payload(
                        account,
                        &target.device,
                        bundle.as_ref(),
                        session,
                        identity,
                        &minted.distribution_message,
                    )
                    .await?
                }
            };
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
            replacements,
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
    replacements: Vec<bool>,
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
        sender_key_distribution::validate_in(
            tx,
            &self.original.chat,
            &self.original.own,
            self.original.distribution.as_ref(),
        )?;
        let changed: Vec<_> = self
            .original
            .targets
            .iter()
            .zip(&self.replacements)
            .filter(|(_, replaced)| **replaced)
            .map(|(target, _)| target.device.clone())
            .collect();
        // All original routing/crypto checks have passed. Archives, verification
        // invalidation, device refresh, other-group rotation and this new chain
        // commit in the caller's original send transaction, or all roll back.
        for ((target, (_, current)), replaced) in self
            .original
            .targets
            .iter()
            .zip(&self.updates)
            .zip(&self.replacements)
        {
            if *replaced {
                super::identity_conflicts::archive_bundle_replacement(
                    tx,
                    &target.device,
                    target.identity.as_deref(),
                    target.session.as_deref().map(|v| v.as_slice()),
                    current,
                )?;
            }
        }
        let expected_distribution = self
            .original
            .distribution
            .as_ref()
            .map(|state| state.after_primary_changes(&changed));
        sender_key_distribution::reserve_in(
            tx,
            &self.original.chat,
            &self.original.own,
            expected_distribution.as_ref(),
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
    #[tokio::test]
    async fn signed_group_replacement_rotates_exposed_chain_and_keeps_history_atomic() {
        for lid in [false, true] {
            for mode in [
                "accept",
                "pinned",
                "own_alias",
                "own_lid",
                "bad_signature",
                "stale_pin",
                "stale_alias",
                "stale_distribution",
                "rollback",
            ] {
                if mode == "own_lid" && !lid {
                    continue;
                }
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("replacement.redb");
                let store = open(&path);
                let mut old = fixture();
                let mut new = fixture();
                if lid {
                    for f in [&mut old, &mut new] {
                        f.plan.own = jid("111111111111111@lid");
                        f.plan.addressing_mode = "lid";
                        f.plan.devices =
                            vec![jid("222222222222221@lid"), jid("222222222222222:3@lid")];
                        f.plan.recipients = f.plan.devices.iter().map(Jid::to_non_ad).collect();
                        f.plan.participant_hash = super::super::groups::participant_hash_v2(
                            &std::iter::once(f.plan.own.clone())
                                .chain(f.plan.devices.clone())
                                .collect::<Vec<_>>(),
                        );
                        let users = f
                            .reply
                            .child("list")
                            .unwrap()
                            .child_nodes()
                            .iter()
                            .cloned()
                            .zip(&f.plan.devices)
                            .map(|(u, j)| u.attr("jid", j.clone()))
                            .collect();
                        f.reply = Node::new("iq")
                            .attr("type", "result")
                            .children(vec![Node::new("list").children(users)]);
                        for ((account, _), device) in f.bobs.iter_mut().zip(&f.plan.devices) {
                            account.own_jid = Some(device.clone());
                        }
                    }
                }
                // Only the primary changes. The other user's companion keeps
                // its original authenticated inventory and identity.
                new.reply = Node::new("iq").attr("type", "result").children(vec![
                    Node::new("list").children(vec![
                        new.reply.child("list").unwrap().child_nodes()[0].clone(),
                        old.reply.child("list").unwrap().child_nodes()[1].clone(),
                    ]),
                ]);
                save_routing(&store, &old);
                let prior_key = group::prepare_own_sender_key(&old.plan.chat, &old.plan.own, None)
                    .await
                    .unwrap();
                let other_chat = jid("120363000000000002@g.us");
                let mut prior = vec![];
                for device in &old.plan.devices {
                    let bundle = direct::prekey_bundle_for(&old.reply, device).unwrap();
                    let prepared = direct::prepare_payload(
                        &new.alice,
                        device,
                        Some(&bundle),
                        None,
                        None,
                        b"old session fixture",
                    )
                    .await
                    .unwrap();
                    prior.push((prepared.session.to_vec(), prepared.identity));
                }
                store
                    .write(|tx| {
                        new.alice.save(tx)?;
                        if lid {
                            super::super::group_send::record_own_lid(tx, &old.plan.own)?;
                        }
                        if mode == "own_lid" {
                            super::super::group_send::record_own_lid(tx, &old.plan.devices[0])?;
                        }
                        if mode == "own_alias" {
                            tx.put_json(
                                Table::Checkpoints,
                                b"login-identity",
                                1,
                                &serde_json::json!({"pn":old.plan.devices[0].to_string()}),
                            )?;
                        }
                        for (device, (session, pin)) in old.plan.devices.iter().zip(&prior) {
                            tx.put_json(
                                Table::SignalSessions,
                                session_key(device).as_bytes(),
                                1,
                                session,
                            )?;
                            tx.put_json(
                                Table::PeerIdentities,
                                session_key(device).as_bytes(),
                                1,
                                pin,
                            )?;
                        }
                        tx.put_json(
                            Table::SenderKeys,
                            &group::sender_key_db_key(&old.plan.chat, &old.plan.own),
                            1,
                            &*prior_key.updated_key,
                        )?;
                        sender_key_distribution::reserve_in(
                            tx,
                            &old.plan.chat,
                            &old.plan.own,
                            None,
                            &old.plan.devices,
                        )?;
                        sender_key_distribution::reserve_in(
                            tx,
                            &other_chat,
                            &old.plan.own,
                            None,
                            &old.plan.devices,
                        )
                    })
                    .unwrap();
                if mode == "bad_signature" {
                    let mut user = new.reply.child("list").unwrap().child_nodes()[0].clone();
                    let mut fields = user.child_nodes().to_vec();
                    let skey = fields.iter_mut().find(|n| n.description == "skey").unwrap();
                    let mut keys = skey.child_nodes().to_vec();
                    *keys
                        .iter_mut()
                        .find(|n| n.description == "signature")
                        .unwrap() = Node::new("signature").bytes(vec![0; 64]);
                    *skey = Node::new("skey").children(keys);
                    user = user.children(fields);
                    new.reply = Node::new("iq").attr("type", "result").children(vec![
                        Node::new("list").children(vec![
                            user,
                            old.reply.child("list").unwrap().child_nodes()[1].clone(),
                        ]),
                    ]);
                }
                let before = state(&store);
                let head = store.journal_head().unwrap();
                let prepared = Snapshot::load(&store, &old.plan, true)
                    .unwrap()
                    .prepare_with_policy(
                        &store,
                        &new.alice,
                        Some(&new.reply),
                        b"new independent group body",
                        if mode == "pinned" {
                            IdentityChangePolicy::Pinned
                        } else {
                            IdentityChangePolicy::ApkAuthenticated
                        },
                    )
                    .await;
                if ["pinned", "own_alias", "own_lid", "bad_signature"].contains(&mode) {
                    assert!(prepared.is_err(), "{mode}");
                    assert_eq!(before, state(&store));
                    assert_eq!(head, store.journal_head().unwrap());
                    continue;
                }
                let prepared = prepared.unwrap();
                assert!(prepared.rotate);
                let received = direct::receive_prekey(
                    &new.bobs[0].0,
                    &old.plan.own,
                    (None, None),
                    &new.bobs[0].1,
                    None,
                    &prepared.targets()[0].ciphertext,
                )
                .await
                .unwrap();
                let (_, skdm) =
                    group::extract_skdm_from_payload(&received.received.plaintext).unwrap();
                let installed = group::process_skdm(
                    &old.plan.chat,
                    &old.plan.own,
                    group::sender_key_db_key(&old.plan.chat, &old.plan.own),
                    None,
                    &skdm,
                )
                .await
                .unwrap();
                let recovered = group::receive_group(
                    &old.plan.chat,
                    &old.plan.own,
                    Some(&installed.updated_key),
                    prepared.ciphertext(),
                )
                .await
                .unwrap()
                .unwrap();
                assert_eq!(&*recovered.plaintext, b"new independent group body");
                assert!(
                    group::receive_group(
                        &old.plan.chat,
                        &old.plan.own,
                        Some(&prior_key.updated_key),
                        prepared.ciphertext()
                    )
                    .await
                    .is_err()
                );
                assert_eq!(before, state(&store));
                if mode == "stale_pin" {
                    let pin = direct::prekey_bundle_for(&new.reply, &old.plan.devices[0])
                        .unwrap()
                        .identity_key()
                        .unwrap()
                        .serialize()
                        .to_vec();
                    store
                        .write(|tx| {
                            tx.put_json(
                                Table::PeerIdentities,
                                session_key(&old.plan.devices[0]).as_bytes(),
                                1,
                                &pin,
                            )
                        })
                        .unwrap();
                }
                if mode == "stale_alias" {
                    store
                        .write(|tx| {
                            tx.put_json(
                                Table::Checkpoints,
                                b"login-identity",
                                1,
                                &serde_json::json!({"pn":old.plan.devices[0].to_string()}),
                            )
                        })
                        .unwrap();
                }
                if mode == "stale_distribution" {
                    store
                        .write(|tx| {
                            sender_key_distribution::after_primary_change_in(
                                tx,
                                &old.plan.devices[0],
                            )
                        })
                        .unwrap();
                }
                let precommit = state(&store);
                let result = store.write(|tx| {
                    prepared.reserve_in(tx, b"original-rotated-group-key")?;
                    tx.put_json(Table::Checkpoints, b"original-send-intent", 1, &"reserved")?;
                    if mode == "rollback" {
                        return Err(invalid("injected caller transaction cut"));
                    }
                    Ok(())
                });
                if ["stale_pin", "stale_alias", "stale_distribution", "rollback"].contains(&mode) {
                    assert!(result.is_err());
                    assert_eq!(precommit, state(&store));
                    assert_eq!(head, store.journal_head().unwrap());
                    continue;
                }
                result.unwrap();
                let histories: Vec<serde_json::Value> = store
                    .read(|tx| {
                        Ok(tx
                            .scan(Table::Checkpoints, 1, 1000)?
                            .into_iter()
                            .filter(|(k, _)| k.starts_with(b"identity-history:"))
                            .map(|(_, v)| serde_json::from_slice(&v).unwrap())
                            .collect())
                    })
                    .unwrap();
                assert_eq!(histories.len(), 1);
                assert_eq!(histories[0]["authentication"], "signed_directory_bundle");
                assert_eq!(histories[0]["verification"], "unverified");
                assert_eq!(
                    histories[0]["previous_identity"],
                    serde_json::json!(prior[0].1)
                );
                assert_eq!(
                    histories[0]["previous_session"],
                    serde_json::json!(prior[0].0)
                );
                assert!(
                    !store
                        .read(|tx| sender_key_distribution::load_in(
                            tx,
                            &old.plan.chat,
                            &old.plan.own
                        ))
                        .unwrap()
                        .unwrap()
                        .rotation_pending()
                );
                assert!(
                    store
                        .read(|tx| sender_key_distribution::load_in(tx, &other_chat, &old.plan.own))
                        .unwrap()
                        .unwrap()
                        .rotation_pending()
                );
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<Vec<u8>>(
                            Table::Checkpoints,
                            b"original-rotated-group-key",
                            1
                        ))
                        .unwrap(),
                    Some(prior_key.updated_key.to_vec())
                );
                assert!(
                    super::super::devices::pending_users(&store)
                        .unwrap()
                        .contains(&old.plan.devices[0])
                );
                let head = store.journal_head().unwrap();
                drop(store);
                let reopened = open(&path);
                assert_eq!(head, reopened.journal_head().unwrap());
                assert!(
                    reopened
                        .read(|tx| tx.get_json::<String>(
                            Table::Checkpoints,
                            b"original-send-intent",
                            1
                        ))
                        .unwrap()
                        .is_some()
                );
            }
        }
    }
    #[tokio::test]
    async fn primary_replacement_cannot_reauthorize_old_companion_inventory() {
        use super::super::devices::{DeviceEntry, DeviceList};
        use sha2::Digest;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("old-authority.redb"));
        let mut old = fixture();
        let new = fixture();
        let peer = old.plan.devices[0].clone();
        old.plan.devices[1] = peer.with_device(3);
        old.plan.recipients = vec![peer.clone()];
        let users = old
            .reply
            .child("list")
            .unwrap()
            .child_nodes()
            .iter()
            .cloned()
            .zip(&old.plan.devices)
            .map(|(n, j)| n.attr("jid", j.clone()))
            .collect();
        old.reply = Node::new("iq")
            .attr("type", "result")
            .children(vec![Node::new("list").children(users)]);
        save_routing(&store, &old);
        let pin = direct::prekey_bundle_for(&old.reply, &peer)
            .unwrap()
            .identity_key()
            .unwrap()
            .serialize()
            .to_vec();
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    session_key(&peer).as_bytes(),
                    1,
                    &pin,
                )?;
                let mut list: DeviceList = tx
                    .get_json(
                        Table::Checkpoints,
                        super::super::devices::list_key(&peer).as_bytes(),
                        1,
                    )?
                    .unwrap();
                list.devices = old
                    .plan
                    .devices
                    .iter()
                    .map(|j| DeviceEntry {
                        jid: j.clone(),
                        key_index: 0,
                    })
                    .collect();
                list.verified_primary_key_fingerprint =
                    Some(hex::encode(sha2::Sha256::digest(&pin[1..])));
                tx.put_json(
                    Table::Checkpoints,
                    super::super::devices::list_key(&peer).as_bytes(),
                    1,
                    &list,
                )
            })
            .unwrap();
        let plan = super::super::group_send::plan(&store, &old.plan.chat, &old.plan.own).unwrap();
        assert_eq!(plan.devices, old.plan.devices);
        let before = state(&store);
        let head = store.journal_head().unwrap();
        let result = Snapshot::load(&store, &plan, true)
            .unwrap()
            .prepare_with_policy(
                &store,
                &old.alice,
                Some(&new.reply),
                b"must remain unsent",
                IdentityChangePolicy::ApkAuthenticated,
            )
            .await;
        assert!(
            matches!(result,Err(Error::InvalidInput{reason,..}) if reason==COMPANION_AUTHORITY_CHANGED)
        );
        assert_eq!(before, state(&store));
        assert_eq!(head, store.journal_head().unwrap());
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
    async fn hybrid_primary_and_companion_distributions_open_natively_and_continue_once() {
        use crate::signal::{hybrid, kem};
        use wacore_libsignal::protocol::{IdentityKey, PrivateKey};
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("hybrid-fanout.redb");
        let store = open(&path);
        let mut f = fixture();
        // Pins come from the classic projection of the same rows.
        save_routing(&store, &f);
        let mut secrets = Vec::new();
        let mut users = f.reply.child("list").unwrap().child_nodes().to_vec();
        for (index, (user, (bob, _))) in users.iter_mut().zip(&f.bobs).enumerate() {
            let key = &bob.signal_identity.as_ref().unwrap().0;
            let identity = IdentityKeyPair::new(
                IdentityKey::new(
                    wacore_libsignal::protocol::PublicKey::from_djb_public_key_bytes(&key.public)
                        .unwrap(),
                ),
                PrivateKey::deserialize(&key.private).unwrap(),
            );
            let (kem_public, kem_secret) = kem::generate_key_pair().unwrap();
            let signature = identity
                .private_key()
                .calculate_signature(&kem_public, &mut rand::rng())
                .unwrap();
            let id = 9 + index as u32;
            let mut children = user.child_nodes().to_vec();
            children.push(Node::new("pqkey").children(vec![
                Node::new("id").bytes(id.to_be_bytes()[1..].to_vec()),
                Node::new("value").bytes(kem_public),
                Node::new("signature").bytes(signature.to_vec()),
            ]));
            *user = user.clone().children(children);
            secrets.push((id, kem_secret));
        }
        f.reply = Node::new("iq")
            .attr("type", "result")
            .children(vec![Node::new("list").children(users)]);
        let original = state(&store);
        let payload = super::super::group_send::text_payload("hybrid group fixture").unwrap();
        // A tampered KEM signature refuses the whole memory-only preparation.
        let mut forged = f.reply.clone();
        let crate::binary::NodeContent::Children(lists) = &mut forged.content else {
            panic!("list")
        };
        let crate::binary::NodeContent::Children(rows) = &mut lists[0].content else {
            panic!("rows")
        };
        let crate::binary::NodeContent::Children(fields) = &mut rows[1].content else {
            panic!("fields")
        };
        let crate::binary::NodeContent::Children(pq) = &mut fields.last_mut().unwrap().content
        else {
            panic!("pqkey")
        };
        let crate::binary::NodeContent::Bytes(signature) = &pq[2].content else {
            panic!("signature")
        };
        let mut tampered = signature.to_vec();
        tampered[0] ^= 1;
        pq[2] = Node::new("signature").bytes(tampered);
        assert!(
            Snapshot::load(&store, &f.plan, false)
                .unwrap()
                .prepare(&f.alice, Some(&forged), &payload)
                .await
                .is_err()
        );
        let prepared = Snapshot::load(&store, &f.plan, false)
            .unwrap()
            .prepare(&f.alice, Some(&f.reply), &payload)
            .await
            .unwrap();
        assert_eq!(original, state(&store));
        let mut sessions = Vec::new();
        for ((target, (bob, signed)), (id, secret)) in
            prepared.targets().iter().zip(&f.bobs).zip(&secrets)
        {
            assert_eq!((target.kind, target.ciphertext[0]), ("pkmsg", 0x44));
            let opened = hybrid::receive_prekey(
                bob,
                &f.plan.own,
                (None, None, IdentityChangePolicy::Pinned),
                Some(signed),
                None,
                Some(hybrid::LocalKemPrekey {
                    id: *id,
                    secret: secret.as_bytes(),
                    last_resort: false,
                }),
                &target.ciphertext,
            )
            .await
            .unwrap();
            assert_eq!(opened.consumed_kem, Some(*id));
            let (chat, skdm) =
                group::extract_skdm_from_payload(&opened.incoming.received.plaintext).unwrap();
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
            sessions.push((
                opened.incoming.received.session.to_vec(),
                opened.incoming.identity,
            ));
        }
        store
            .write(|tx| prepared.reserve_in(tx, b"archive:hybrid"))
            .unwrap();
        let committed = state(&store);
        drop(store);
        let store = open(&path);
        assert_eq!(committed, state(&store));
        // Held version-4 records continue without another directory read;
        // the still-pending KEM fields keep the next send a 0x44 pkmsg.
        let cached = Snapshot::load(&store, &f.plan, false).unwrap();
        assert!(cached.required_keys().is_empty());
        let next = cached.prepare(&f.alice, None, &payload).await.unwrap();
        for ((target, (bob, _)), (session, pin)) in
            next.targets().iter().zip(&f.bobs).zip(&sessions)
        {
            assert_eq!((target.kind, target.ciphertext[0]), ("pkmsg", 0x44));
            let again = hybrid::receive_prekey(
                bob,
                &f.plan.own,
                (Some(session), Some(pin), IdentityChangePolicy::Pinned),
                None,
                None,
                None,
                &target.ciphertext,
            )
            .await
            .unwrap();
            assert_eq!(again.consumed_kem, None);
        }
        assert_eq!(committed, state(&store));

        // Fresh mode with a signed primary replacement under the explicit
        // policy: the changed device derives a fresh hybrid session, the
        // unchanged companion re-derives against its pinned identity.
        let replacement = IdentityKeyPair::generate(&mut rand::rng());
        let signed_new = KeyPair::generate(&mut rand::rng());
        let (kem_public, kem_secret) = kem::generate_key_pair().unwrap();
        let sign = |bytes: &[u8]| {
            replacement
                .private_key()
                .calculate_signature(bytes, &mut rand::rng())
                .unwrap()
                .to_vec()
        };
        let signed_signature = sign(&signed_new.public_key.serialize());
        let changed = Node::new("user")
            .attr("jid", f.plan.devices[0].clone())
            .children(vec![
                Node::new("registration").bytes(vec![0, 0, 0, 42]),
                Node::new("type").bytes(vec![5]),
                Node::new("identity").bytes(replacement.public_key().public_key_bytes().to_vec()),
                Node::new("skey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 7]),
                    Node::new("value").bytes(signed_new.public_key.public_key_bytes().to_vec()),
                    Node::new("signature").bytes(signed_signature.clone()),
                ]),
                Node::new("pqkey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 11]),
                    Node::new("value").bytes(kem_public.clone()),
                    Node::new("signature").bytes(sign(&kem_public)),
                ]),
            ]);
        let reply =
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(vec![
                    changed,
                    f.reply.child("list").unwrap().child_nodes()[1].clone(),
                ])]);
        let fresh = Snapshot::load(&store, &f.plan, true).unwrap();
        assert_eq!(fresh.required_keys(), f.plan.devices);
        assert!(matches!(
            Snapshot::load(&store, &f.plan, true)
                .unwrap()
                .prepare(&f.alice, Some(&reply), &payload)
                .await,
            Err(Error::IdentityChanged(_))
        ));
        let replaced = fresh
            .prepare_with_policy(
                &store,
                &f.alice,
                Some(&reply),
                &payload,
                IdentityChangePolicy::ApkAuthenticated,
            )
            .await
            .unwrap();
        let new_bob = account(&replacement, &f.plan.devices[0], 42);
        let new_signed = SignedPreKeyRecord::new(
            7.into(),
            wacore_libsignal::protocol::Timestamp::from_epoch_millis(0),
            &signed_new,
            &signed_signature,
        )
        .serialize()
        .unwrap();
        let target = &replaced.targets()[0];
        assert_eq!((target.kind, target.ciphertext[0]), ("pkmsg", 0x44));
        let opened = hybrid::receive_prekey(
            &new_bob,
            &f.plan.own,
            (None, None, IdentityChangePolicy::Pinned),
            Some(&new_signed),
            None,
            Some(hybrid::LocalKemPrekey {
                id: 11,
                secret: kem_secret.as_bytes(),
                last_resort: false,
            }),
            &target.ciphertext,
        )
        .await
        .unwrap();
        assert!(group::extract_skdm_from_payload(&opened.incoming.received.plaintext).is_some());
        assert_eq!(committed, state(&store));
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
