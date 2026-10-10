//! APK 37.71 bounded primary/classic and known same-key companion application.
//! Stored attestation state/property assignments remain unknown: this branch
//! admits under both property values, without manufacturing either assignment.
use super::{RefreshGuard, actions, guard, inspect_in};
use crate::{
    Error, Result,
    binary::Node,
    model::jid::{Jid, Server},
    registration::request::IdentityChangePolicy,
    session::AccountRecord,
    store::{ProtocolStore, ReadTx, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Application {
    Unrelated,
    Applied,
    AlreadyApplied,
    Deferred(&'static str),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Applied {
    peer: Jid,
    generation: u32,
    identity_fingerprint: [u8; 32],
}
fn key(id: &str) -> String {
    format!("identity-refresh-applied:{id}")
}
fn invalid(reason: &'static str) -> Error {
    Error::invalid("identity_notification", reason)
}
pub(super) fn applied_in(tx: &ReadTx<'_>, id: &str, peer: &Jid) -> Result<bool> {
    applied(tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?, peer)
}
fn applied(bytes: Option<Vec<u8>>, peer: &Jid) -> Result<bool> {
    let Some(bytes) = bytes else { return Ok(false) };
    let marker: Applied =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid saved refresh application"))?;
    if marker.peer != *peer {
        return Err(invalid("invalid saved refresh application"));
    }
    // Historical application is a fact, not a promise this key is still current.
    Ok(true)
}
fn account_fingerprint(account: &AccountRecord) -> Result<[u8; 32]> {
    let bytes = Zeroizing::new(
        serde_json::to_vec(account).map_err(|_| invalid("invalid saved refresh account"))?,
    );
    Ok(Sha256::digest(&*bytes).into())
}
struct Staged {
    id: String,
    peer: Jid,
    reply: Node,
    pin: Vec<u8>,
    previous_session: Option<Zeroizing<Vec<u8>>>,
    identity: Vec<u8>,
    session: Zeroizing<Vec<u8>>,
    account_fingerprint: [u8; 32],
    companion: Option<Box<super::super::companion_admission::AuthoritySnapshot>>,
}
enum Preparation {
    Ready(Box<Staged>),
    Finished(Application),
}
fn deferred(reason: &'static str) -> Preparation {
    Preparation::Finished(Application::Deferred(reason))
}
async fn stage(
    store: &ProtocolStore,
    id: &str,
    policy: IdentityChangePolicy,
) -> Result<Preparation> {
    use super::super::peer_keys::DirectoryStatus;
    let snapshot = store.read(|tx| {
        let Some(inspection) = inspect_in(tx, id)? else {
            return Ok(None);
        };
        let (action, reply) =
            actions::view_in(tx, id)?.ok_or_else(|| invalid("missing saved action"))?;
        let request =
            actions::request_in(tx, id)?.ok_or_else(|| invalid("missing saved key request"))?;
        let peer = request
            .child("key")
            .and_then(|key| key.child_nodes().first())
            .and_then(|user| super::super::receipts::jid(user, "jid"))
            .ok_or_else(|| invalid("invalid saved key request target"))?;
        let peer_key = format!("comparison-direct:{peer}");
        let pin: Option<Vec<u8>> = tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        let session: Option<Vec<u8>> =
            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
        Ok(Some((
            inspection,
            action.status,
            peer,
            pin,
            session.map(Zeroizing::new),
            AccountRecord::load(tx)?,
            reply,
        )))
    })?;
    let Some((inspection, status, peer, pin, previous_session, account, reply)) = snapshot else {
        return Ok(Preparation::Finished(Application::Unrelated));
    };
    if inspection.applied {
        return Ok(Preparation::Finished(Application::AlreadyApplied));
    }
    if policy != IdentityChangePolicy::ApkAuthenticated {
        return Ok(deferred("identity_policy_pinned"));
    }
    if status != actions::ActionStatus::Ok || inspection.status != DirectoryStatus::VerifiedBundle {
        return Ok(deferred("reply_not_verified"));
    }
    if inspection.refresh_guard != RefreshGuard::Matches {
        return Ok(deferred("original_refresh_guard_not_matching"));
    }
    let Some(account) = account.filter(AccountRecord::is_registered) else {
        return Ok(deferred("missing_registered_account"));
    };
    if super::super::inbox::remote_identity_policy(store, &account, &peer, policy)?
        != IdentityChangePolicy::ApkAuthenticated
        || super::super::group_send::own_lid(store)?.as_ref() == Some(&peer.to_non_ad())
    {
        return Ok(deferred("own_identity"));
    }
    let reply = reply.ok_or_else(|| invalid("missing saved reply"))?;
    let user = reply
        .child("list")
        .and_then(|list| {
            list.child_nodes().iter().find(|user| {
                user.description == "user"
                    && super::super::receipts::jid(user, "jid").is_some_and(|jid| {
                        super::super::identity_query::same_primary_address(&jid, &peer)
                    })
            })
        })
        .ok_or_else(|| invalid("missing response user"))?;
    for (name, reason) in [
        ("non_e2ee_attestation", "attestation_admission_unported"),
        ("pqkey", "pq_session_unported"),
        ("identity_auth", "identity_auth_unported"),
    ] {
        if user.child(name).is_some() {
            return Ok(deferred(reason));
        }
    }
    let same_identity = inspection.pin_matches == Some(true);
    // Bot.A03 0370..0384 sets Cai.A08 from child presence (1xi.A1U),
    // not its body or an attribute. CPB.A00 same-key absent-CWS E2EE:
    // equal old state -> Bjo; NOT_SET -> Bjn; HYBRID/NON with A08 ->
    // Bjn at 0252 -> 0306. Disabled main property -> Bjo. All admit;
    // without that server child, unknown old state/0x7497 cannot be guessed.
    if peer.device != 0 && !same_identity {
        return Ok(deferred("companion_admission_unported"));
    }
    if peer.device == 0 && same_identity && user.child("force_override_e2ee_state").is_none() {
        return Ok(deferred("same_identity_admission_unported"));
    }
    if inspection.pin_matches.is_none() {
        return Ok(deferred("missing_saved_identity"));
    }
    // CPB.A00: property 0x6f6f=false -> Bjo at 00cc; true + absent CWS
    // -> E2EE at 01e4, changed identity v12=1 at 00d2, then 0226 -> Bjn
    // at 0306, independent of old C0F/override/0x7497. Bot.A03 present
    // native metadata is refused above; never infer a runtime flag/state.
    let bundle = crate::signal::direct::prekey_bundle_for(&reply, &peer)?;
    let identity = bundle
        .identity_key()
        .map_err(|_| invalid("invalid bundle identity"))?
        .serialize()
        .to_vec();
    let pin = pin.ok_or_else(|| invalid("missing saved identity"))?;
    let companion = if peer.device != 0 {
        match store.read(|tx| {
            super::super::companion_admission::prepare_same_key_directory_in(
                tx, &peer, user, &identity,
            )
        })? {
            super::super::companion_admission::Admission::Ready(authority) => Some(authority),
            super::super::companion_admission::Admission::Refused(reason) => {
                return Ok(deferred(reason));
            }
        }
    } else {
        None
    };

    // 0jc.A0L 0096/00a2 resets a changed-key session via A0E/A0s -> A0D,
    // which deletes the selected sessions row. Derive fresh, retaining old
    // bytes in the encrypted local transition archive, not the new ratchet.
    // 0jc.A0L matching-key branch skips A0E reset, but CQc.A00 still
    // archives the held current ratchet and derives a fresh one. Keep the
    // old receiving state only for the same identity, never across key change.
    let held = same_identity
        .then_some(previous_session.as_deref())
        .flatten();
    let session = crate::signal::direct::prepare_session_refresh(
        &account,
        &peer,
        &bundle,
        held.map(Vec::as_slice),
        &identity,
    )
    .await?;
    Ok(Preparation::Ready(Box::new(Staged {
        id: id.to_owned(),
        peer,
        reply,
        pin,
        previous_session,
        identity,
        session,
        account_fingerprint: account_fingerprint(&account)?,
        companion,
    })))
}
fn commit(store: &ProtocolStore, staged: &Staged) -> Result<Application> {
    store.write(|tx| {
        if applied(
            tx.get(Table::Checkpoints, key(&staged.id).as_bytes(), 1)?,
            &staged.peer,
        )? {
            return Ok(Application::AlreadyApplied);
        }
        if actions::completed_reply_in(tx, &staged.id)?.as_ref() != Some(&staged.reply) {
            return Ok(Application::Deferred("reply_changed_before_commit"));
        }
        let peer_key = format!("comparison-direct:{}", staged.peer);
        let pin: Option<Vec<u8>> = tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
        if guard::inspect_write_in(tx, &staged.id, &staged.peer, pin.as_deref())?
            != RefreshGuard::Matches
        {
            return Ok(Application::Deferred("original_refresh_guard_not_matching"));
        }
        let session: Option<Zeroizing<Vec<u8>>> = tx
            .get_json::<Vec<u8>>(Table::SignalSessions, peer_key.as_bytes(), 1)?
            .map(Zeroizing::new);
        if pin.as_deref() != Some(staged.pin.as_slice()) || session != staged.previous_session {
            return Ok(Application::Deferred("peer_state_changed_before_commit"));
        }
        let Some(account) = AccountRecord::load_in_write(tx)? else {
            return Ok(Application::Deferred("account_changed_before_commit"));
        };
        if !account.is_registered() || account_fingerprint(&account)? != staged.account_fingerprint
        {
            return Ok(Application::Deferred("account_changed_before_commit"));
        }
        let own = account.own_jid.as_ref().map(Jid::to_non_ad);
        let own_lid = super::super::group_send::own_lid_in(tx)?;
        let login: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, b"login-identity", 1)?;
        let own_pn = login
            .as_ref()
            .and_then(|v| v.get("pn"))
            .filter(|v| !v.is_null())
            .map(|v| -> Result<Jid> {
                let jid = v
                    .as_str()
                    .and_then(|s| Jid::parse(s).ok())
                    .filter(|j| j.server == Server::User && !j.user.is_empty() && j.agent == 0)
                    .ok_or_else(|| invalid("invalid saved own-pn authority"))?;
                Ok(jid.to_non_ad())
            })
            .transpose()?;
        let peer = staged.peer.to_non_ad();
        if [own, own_lid, own_pn]
            .into_iter()
            .flatten()
            .any(|own| own == peer)
        {
            return Ok(Application::Deferred("own_identity"));
        }
        if let Some(authority) = &staged.companion
            && !authority.matches(tx)?
        {
            return Ok(Application::Deferred(
                "companion_authority_changed_before_commit",
            ));
        }
        super::super::identity_conflicts::archive_bundle_replacement(
            tx,
            &staged.peer,
            Some(&staged.pin),
            staged.previous_session.as_deref().map(Vec::as_slice),
            &staged.identity,
        )?;
        tx.put_json(
            Table::PeerIdentities,
            peer_key.as_bytes(),
            1,
            &staged.identity,
        )?;
        tx.put_json(
            Table::SignalSessions,
            peer_key.as_bytes(),
            1,
            &*staged.session,
        )?;
        tx.put_json(
            Table::Checkpoints,
            key(&staged.id).as_bytes(),
            1,
            &Applied {
                peer: staged.peer.clone(),
                generation: tx.generation().0,
                identity_fingerprint: Sha256::digest(&staged.identity).into(),
            },
        )?;
        Ok(Application::Applied)
    })
}

/// No network or chat creation. Caller supplies the existing identity-change policy.
pub(crate) async fn apply(
    store: &ProtocolStore,
    id: &str,
    policy: IdentityChangePolicy,
) -> Result<Application> {
    match stage(store, id, policy).await? {
        Preparation::Ready(staged) => commit(store, &staged),
        Preparation::Finished(result) => Ok(result),
    }
}
/// Repair a cut after reply settlement without replaying IQs or historical reads.
pub(crate) async fn recover(store: &ProtocolStore, policy: IdentityChangePolicy) -> Result<usize> {
    let mut applied = 0;
    for action in actions::outcomes(store)? {
        if action.kind == "identity_refresh"
            && action.status == actions::ActionStatus::Ok
            && apply(store, &action.id, policy).await? == Application::Applied
        {
            applied += 1;
        }
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::super::{
        Refresh, queue,
        tests::{account, install_pin, notification, open, reply},
    };
    use super::*;
    use crate::{
        binary::NodeContent,
        session::{KeyMaterial, SignalIdentityKey},
    };
    use wacore_libsignal::protocol::{IdentityKeyPair, SessionRecord};

    struct Fixture {
        dir: tempfile::TempDir,
        store: ProtocolStore,
        account: AccountRecord,
        peer: Jid,
        id: String,
        old: IdentityKeyPair,
        new: IdentityKeyPair,
        response: Node,
        held: Vec<u8>,
    }
    async fn fixture(peer: &str) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("application.redb"));
        let peer = Jid::parse(peer).unwrap();
        let mut account = account();
        let local = IdentityKeyPair::generate(&mut rand::rng());
        account.signal_registration_id = Some(31);
        account.signal_identity = Some(SignalIdentityKey(KeyMaterial {
            public: local.public_key().public_key_bytes().try_into().unwrap(),
            private: *local.private_key().serialize(),
        }));
        let old = install_pin(&store, &peer);
        let old_reply = reply(&peer, "old", &old, true);
        let old_bundle = crate::signal::direct::prekey_bundle_for(&old_reply, &peer).unwrap();
        let prepared = crate::signal::direct::prepare_payload(
            &account,
            &peer,
            Some(&old_bundle),
            None,
            Some(&old.identity_key().serialize()),
            b"fixture",
        )
        .await
        .unwrap();
        let held = prepared.session.to_vec();
        store
            .write(|tx| {
                account.save(tx)?;
                tx.put_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &held,
                )
            })
            .unwrap();
        let Refresh::Queued(id) =
            queue(&store, &account, &notification(&peer.to_string(), "change")).unwrap()
        else {
            panic!("expected original guarded read")
        };
        let new = IdentityKeyPair::generate(&mut rand::rng());
        let response = reply(&peer, &id, &new, true);
        Fixture {
            dir,
            store,
            account,
            peer,
            id,
            old,
            new,
            response,
            held,
        }
    }
    fn settle(f: &Fixture, response: &Node) {
        assert_eq!(actions::reserve_pending(&f.store).unwrap().len(), 1);
        assert!(actions::record_reply(&f.store, response).unwrap());
    }
    type Rows = Vec<Vec<(Vec<u8>, Vec<u8>)>>;
    fn snapshot(store: &ProtocolStore) -> Rows {
        store
            .read(|tx| {
                Table::ALL
                    .iter()
                    .map(|table| tx.scan_after(*table, 1, None, 512))
                    .collect()
            })
            .unwrap()
    }
    fn metadata(node: &mut Node, name: &str) {
        let NodeContent::Children(lists) = &mut node.content else {
            panic!()
        };
        let NodeContent::Children(users) = &mut lists[0].content else {
            panic!()
        };
        let NodeContent::Children(fields) = &mut users[0].content else {
            panic!()
        };
        fields.push(Node::new(name));
    }
    fn install_companion_authority(f: &Fixture) {
        use super::super::super::devices::{DeviceEntry, DeviceList, list_key};
        let primary = f.peer.to_non_ad();
        let key = install_pin(&f.store, &primary);
        let list = DeviceList {
            owner: primary.clone(),
            devices: vec![
                DeviceEntry {
                    jid: primary,
                    key_index: 0,
                },
                DeviceEntry {
                    jid: f.peer.clone(),
                    key_index: 7,
                },
            ],
            raw_id: Some(8),
            list_timestamp: Some(9),
            expected_timestamp: None,
            pending_removed_devices: None,
            adv_added_devices: vec![],
            companions_verified: true,
            verified_primary_key_fingerprint: Some(hex::encode(Sha256::digest(
                key.public_key().public_key_bytes(),
            ))),
            unverified_reason: None,
            notified_hash: None,
            hash_matches: None,
            hosted_skipped: 0,
            fetched_at: crate::model::time::Timestamp::now(),
        };
        f.store.write(|tx| {
            tx.put_json(Table::Checkpoints, list_key(&list.owner).as_bytes(), 1, &list)?;
            tx.put_json(Table::Checkpoints, b"login-identity", 1,
                &serde_json::json!({"pn":"15555550999@s.whatsapp.net", "lid":"999999999999999@lid"}))
        }).unwrap();
    }
    fn device_identity(response: &mut Node, content: NodeContent) {
        metadata(response, "device-identity");
        let NodeContent::Children(lists) = &mut response.content else {
            panic!()
        };
        let NodeContent::Children(users) = &mut lists[0].content else {
            panic!()
        };
        let NodeContent::Children(fields) = &mut users[0].content else {
            panic!()
        };
        fields.last_mut().unwrap().content = content;
    }
    fn change_list(
        f: &Fixture,
        change: impl FnOnce(&mut super::super::super::devices::DeviceList),
    ) {
        use super::super::super::devices::{DeviceList, list_key};
        f.store
            .write(|tx| {
                let key = list_key(&f.peer.to_non_ad());
                let mut list: DeviceList =
                    tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                change(&mut list);
                tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &list)
            })
            .unwrap();
    }
    #[tokio::test]
    async fn known_same_key_companion_refresh_archives_ratchet_and_recovers_once() {
        for (peer, bytes) in [
            ("15555550111:4@s.whatsapp.net", vec![]),
            ("222222222222222:4@lid", vec![0xff]),
        ] {
            let f = fixture(peer).await;
            install_companion_authority(&f);
            let mut response = reply(&f.peer, &f.id, &f.old, true);
            // Native fast return skips ADV parsing, even for non-protobuf bytes.
            device_identity(&mut response, NodeContent::Bytes(bytes.into()));
            settle(&f, &response);
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            assert_eq!(inspection.companion_same_key_authority, Some("ready"));
            assert_eq!(inspection.force_override_e2ee_state, Some(false));
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            assert_eq!(
                recover(&f.store, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                1
            );
            let after = snapshot(&f.store);
            for (i, table) in Table::ALL.iter().enumerate() {
                if !matches!(table, Table::SignalSessions | Table::Checkpoints) {
                    assert_eq!(before[i], after[i], "{table:?}");
                }
            }
            // Exactly the application marker is added; no list/ADV refresh.
            let checkpoints = Table::ALL
                .iter()
                .position(|t| *t == Table::Checkpoints)
                .unwrap();
            let mut expected = before[checkpoints].clone();
            let marker = after[checkpoints]
                .iter()
                .find(|(k, _)| k == key(&f.id).as_bytes())
                .unwrap();
            expected.push(marker.clone());
            expected.sort_by(|a, b| a.0.cmp(&b.0));
            assert_eq!(expected, after[checkpoints]);
            assert_eq!(f.store.journal_head().unwrap(), head);
            let saved: Vec<u8> = f
                .store
                .read(|tx| {
                    tx.get_json(
                        Table::SignalSessions,
                        format!("comparison-direct:{}", f.peer).as_bytes(),
                        1,
                    )
                })
                .unwrap()
                .unwrap();
            let old = SessionRecord::deserialize(&f.held).unwrap();
            let new = SessionRecord::deserialize(&saved).unwrap();
            assert_eq!(
                new.previous_session_count(),
                old.previous_session_count() + 1
            );
            assert_eq!(
                new.previous_session_states()
                    .next()
                    .unwrap()
                    .unwrap()
                    .alice_base_key(),
                old.session_state().unwrap().alice_base_key()
            );
            assert_ne!(
                new.session_state().unwrap().alice_base_key(),
                old.session_state().unwrap().alice_base_key()
            );
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                Application::AlreadyApplied
            );
            assert_eq!(snapshot(&f.store), after);
            let path = f.dir.path().join("application.redb");
            drop(f.store);
            let reopened = open(&path);
            assert_eq!(
                recover(&reopened, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                0
            );
            assert_eq!(snapshot(&reopened), after);
            assert!(actions::reserve_pending(&reopened).unwrap().is_empty());
        }
    }
    #[tokio::test]
    async fn companion_same_key_refusals_preserve_all_rows_and_explain_authority() {
        for (case, expected) in [
            ("missing_adv", "missing_device_identity"),
            ("nonnull_shape", "invalid_device_identity"),
            ("duplicate_adv", "ambiguous_input"),
            ("oversize", "invalid_device_identity"),
            ("missing_list", "missing_device_list"),
            ("stale", "stale_device_list"),
            ("foreign", "stale_device_list"),
            ("not_member", "missing_device_membership"),
            ("duplicate_member", "ambiguous_device_membership"),
            ("teardown", "pending_device_teardown"),
            ("changed", "same_identity_required"),
            ("own", "own_device"),
            ("unknown_own", "missing_own_identity"),
            ("missing_primary", "missing_primary_pin"),
        ] {
            let f = fixture("222222222222222:4@lid").await;
            install_companion_authority(&f);
            let mut response = reply(
                &f.peer,
                &f.id,
                if case == "changed" { &f.new } else { &f.old },
                true,
            );
            if case != "missing_adv" {
                device_identity(
                    &mut response,
                    if case == "nonnull_shape" {
                        NodeContent::Empty
                    } else {
                        NodeContent::Bytes(
                            if case == "oversize" {
                                vec![0; 16385]
                            } else {
                                vec![]
                            }
                            .into(),
                        )
                    },
                );
            }
            if case == "duplicate_adv" {
                metadata(&mut response, "device-identity");
            }
            match case {
                "stale" => change_list(&f, |l| l.verified_primary_key_fingerprint = None),
                "foreign" => {
                    change_list(&f, |l| l.owner = Jid::parse("888888888888888@lid").unwrap())
                }
                "not_member" => change_list(&f, |l| l.devices.retain(|d| d.jid != f.peer)),
                "duplicate_member" => change_list(&f, |l| l.devices.push(l.devices[1].clone())),
                "teardown" => change_list(&f, |l| {
                    l.pending_removed_devices = Some(vec![f.peer.clone()])
                }),
                "missing_list" | "missing_primary" | "unknown_own" => {
                    f.store
                        .write(|tx| {
                            tx.delete(
                                if case == "missing_primary" {
                                    Table::PeerIdentities
                                } else {
                                    Table::Checkpoints
                                },
                                if case == "missing_list" {
                                    super::super::super::devices::list_key(&f.peer.to_non_ad())
                                        .into_bytes()
                                } else if case == "missing_primary" {
                                    format!("comparison-direct:{}", f.peer.to_non_ad()).into_bytes()
                                } else {
                                    b"login-identity".to_vec()
                                }
                                .as_slice(),
                            )
                        })
                        .unwrap();
                }
                "own" => {
                    f.store
                        .write(|tx| {
                            tx.put_json(
                                Table::Checkpoints,
                                b"login-identity",
                                1,
                                &serde_json::json!({"lid": f.peer.to_non_ad().to_string()}),
                            )
                        })
                        .unwrap();
                }
                _ => {}
            }
            settle(&f, &response);
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            assert_eq!(
                inspection.companion_same_key_authority,
                Some(expected),
                "{case}"
            );
            assert!(
                matches!(
                    apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                        .await
                        .unwrap(),
                    Application::Deferred(_)
                ),
                "{case}"
            );
            assert_eq!(snapshot(&f.store), before, "{case}");
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }
    #[tokio::test]
    async fn ready_companion_authority_does_not_override_signature_policy_metadata_or_guard() {
        for case in [
            "forged",
            "pinned",
            "non_e2ee_attestation",
            "pqkey",
            "identity_auth",
            "untracked",
        ] {
            let f = fixture("222222222222222:4@lid").await;
            install_companion_authority(&f);
            let mut response = reply(&f.peer, &f.id, &f.old, case != "forged");
            device_identity(&mut response, NodeContent::Bytes(vec![].into()));
            if matches!(case, "non_e2ee_attestation" | "pqkey" | "identity_auth") {
                metadata(&mut response, case);
            }
            settle(&f, &response);
            if case == "untracked" {
                f.store
                    .write(|tx| {
                        tx.delete(
                            Table::Checkpoints,
                            format!("identity-refresh-snapshot:{}", f.id).as_bytes(),
                        )
                    })
                    .unwrap();
            }
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            assert_eq!(
                inspection.companion_same_key_authority,
                if case == "forged" {
                    None
                } else {
                    Some("ready")
                }
            );
            assert_eq!(
                apply(
                    &f.store,
                    &f.id,
                    if case == "pinned" {
                        IdentityChangePolicy::Pinned
                    } else {
                        IdentityChangePolicy::ApkAuthenticated
                    }
                )
                .await
                .unwrap(),
                Application::Deferred(match case {
                    "forged" => "reply_not_verified",
                    "pinned" => "identity_policy_pinned",
                    "non_e2ee_attestation" => "attestation_admission_unported",
                    "pqkey" => "pq_session_unported",
                    "identity_auth" => "identity_auth_unported",
                    _ => "original_refresh_guard_not_matching",
                })
            );
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }
    #[tokio::test]
    async fn companion_authority_races_cannot_overwrite_session() {
        for case in ["primary", "list", "login", "ratchet"] {
            let f = fixture("222222222222222:4@lid").await;
            install_companion_authority(&f);
            let mut response = reply(&f.peer, &f.id, &f.old, true);
            device_identity(&mut response, NodeContent::Bytes(vec![].into()));
            settle(&f, &response);
            let Preparation::Ready(staged) =
                stage(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap()
            else {
                panic!("stage")
            };
            match case {
                "primary" => {
                    install_pin(&f.store, &f.peer.to_non_ad());
                }
                "list" => change_list(&f, |l| l.expected_timestamp = Some(10)),
                "login" => {
                    f.store.write(|tx| tx.put_json(Table::Checkpoints, b"login-identity", 1,
                    &serde_json::json!({"lid":"999999999999999@lid","pn":"15555550888@s.whatsapp.net"}))).unwrap();
                }
                _ => {
                    let progress = crate::signal::direct::prepare_payload(
                        &f.account,
                        &f.peer,
                        None,
                        Some(&f.held),
                        Some(&f.old.identity_key().serialize()),
                        b"progress",
                    )
                    .await
                    .unwrap();
                    f.store
                        .write(|tx| {
                            tx.put_json(
                                Table::SignalSessions,
                                format!("comparison-direct:{}", f.peer).as_bytes(),
                                1,
                                &*progress.session,
                            )
                        })
                        .unwrap();
                }
            }
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            assert_eq!(
                commit(&f.store, &staged).unwrap(),
                Application::Deferred(if case == "ratchet" {
                    "peer_state_changed_before_commit"
                } else {
                    "companion_authority_changed_before_commit"
                })
            );
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }
    #[tokio::test]
    async fn server_same_identity_override_retains_old_ratchet_without_transition_once() {
        for peer in ["15555550111@s.whatsapp.net", "222222222222222@lid"] {
            let f = fixture(peer).await;
            let mut response = reply(&f.peer, &f.id, &f.old, true);
            metadata(&mut response, "force_override_e2ee_state");
            settle(&f, &response);
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                Application::Applied
            );
            let after = snapshot(&f.store);
            for (index, table) in Table::ALL.iter().enumerate() {
                if !matches!(table, Table::SignalSessions | Table::Checkpoints) {
                    assert_eq!(after[index], before[index], "{table:?}");
                }
            }
            assert_eq!(f.store.journal_head().unwrap(), head);
            let session: Vec<u8> = f
                .store
                .read(|tx| {
                    tx.get_json(
                        Table::SignalSessions,
                        format!("comparison-direct:{}", f.peer).as_bytes(),
                        1,
                    )
                })
                .unwrap()
                .unwrap();
            let old = SessionRecord::deserialize(&f.held).unwrap();
            let new = SessionRecord::deserialize(&session).unwrap();
            assert_eq!(
                new.previous_session_count(),
                old.previous_session_count() + 1
            );
            let original = old.session_state().unwrap();
            let archived = new.previous_session_states().next().unwrap().unwrap();
            assert_eq!(archived.alice_base_key(), original.alice_base_key());
            assert_eq!(
                archived.get_sender_chain_key().unwrap().index(),
                original.get_sender_chain_key().unwrap().index()
            );
            assert_ne!(
                new.session_state().unwrap().alice_base_key(),
                original.alice_base_key()
            );
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            assert_eq!(inspection.force_override_e2ee_state, Some(true));
            assert!(inspection.applied);
            assert_eq!(inspection.pin_matches, Some(true));
            assert_eq!(inspection.refresh_guard, RefreshGuard::Matches);
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                Application::AlreadyApplied
            );
            assert_eq!(snapshot(&f.store), after);
            let id = f.id.clone();
            let path = f.dir.path().join("application.redb");
            drop(f.store);
            let reopened = open(&path);
            assert_eq!(
                recover(&reopened, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                0
            );
            assert!(
                super::super::inspect(&reopened, &id)
                    .unwrap()
                    .unwrap()
                    .applied
            );
            assert_eq!(snapshot(&reopened), after);
        }
    }

    #[tokio::test]
    async fn same_identity_override_is_not_root_attribute_or_metadata_bypass() {
        for case in [
            "root",
            "attribute",
            "attestation",
            "pq",
            "identity_auth",
            "forged",
            "pinned",
            "companion",
        ] {
            let f = fixture(if case == "companion" {
                "222222222222222:4@lid"
            } else {
                "222222222222222@lid"
            })
            .await;
            let mut response = reply(&f.peer, &f.id, &f.old, case != "forged");
            if case == "root" {
                let NodeContent::Children(children) = &mut response.content else {
                    panic!()
                };
                children.push(Node::new("force_override_e2ee_state"));
            } else if case == "attribute" {
                let NodeContent::Children(lists) = &mut response.content else {
                    panic!()
                };
                let NodeContent::Children(users) = &mut lists[0].content else {
                    panic!()
                };
                users[0] = users[0].clone().attr("force_override_e2ee_state", "true");
            } else {
                metadata(&mut response, "force_override_e2ee_state");
            }
            if matches!(case, "attestation" | "pq" | "identity_auth") {
                metadata(
                    &mut response,
                    if case == "attestation" {
                        "non_e2ee_attestation"
                    } else if case == "pq" {
                        "pqkey"
                    } else {
                        "identity_auth"
                    },
                );
            }
            settle(&f, &response);
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            assert!(
                matches!(
                    apply(
                        &f.store,
                        &f.id,
                        if case == "pinned" {
                            IdentityChangePolicy::Pinned
                        } else {
                            IdentityChangePolicy::ApkAuthenticated
                        }
                    )
                    .await
                    .unwrap(),
                    Application::Deferred(_)
                ),
                "{case}"
            );
            assert_eq!(snapshot(&f.store), before, "{case}");
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }

    #[tokio::test]
    async fn same_identity_staging_cannot_overwrite_intervening_ratchet_progress() {
        let f = fixture("222222222222222@lid").await;
        let mut response = reply(&f.peer, &f.id, &f.old, true);
        metadata(&mut response, "force_override_e2ee_state");
        settle(&f, &response);
        let Preparation::Ready(staged) =
            stage(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                .await
                .unwrap()
        else {
            panic!("expected staged refresh")
        };
        let progress = crate::signal::direct::prepare_payload(
            &f.account,
            &f.peer,
            None,
            Some(&f.held),
            Some(&f.old.identity_key().serialize()),
            b"fixture progress",
        )
        .await
        .unwrap();
        f.store
            .write(|tx| {
                tx.put_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{}", f.peer).as_bytes(),
                    1,
                    &*progress.session,
                )
            })
            .unwrap();
        let before = snapshot(&f.store);
        let head = f.store.journal_head().unwrap();
        assert_eq!(
            commit(&f.store, &staged).unwrap(),
            Application::Deferred("peer_state_changed_before_commit")
        );
        assert_eq!(snapshot(&f.store), before);
        assert_eq!(f.store.journal_head().unwrap(), head);
        assert!(
            !super::super::inspect(&f.store, &f.id)
                .unwrap()
                .unwrap()
                .applied
        );
        // A new stage uses the latest saved ratchet rather than freezing it at notification time.
        assert_eq!(
            apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                .await
                .unwrap(),
            Application::Applied
        );
        let session: Vec<u8> = f
            .store
            .read(|tx| {
                tx.get_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{}", f.peer).as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        let refreshed = SessionRecord::deserialize(&session).unwrap();
        let archived = refreshed.previous_session_states().next().unwrap().unwrap();
        let advanced = SessionRecord::deserialize(&progress.session).unwrap();
        assert_eq!(
            archived.get_sender_chain_key().unwrap().index(),
            advanced
                .session_state()
                .unwrap()
                .get_sender_chain_key()
                .unwrap()
                .index()
        );
    }

    #[tokio::test]
    async fn changed_primary_adopts_fresh_session_once_and_recovers_settled_cut() {
        for peer in ["15555550111@s.whatsapp.net", "222222222222222@lid"] {
            let f = fixture(peer).await;
            settle(&f, &f.response);
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            // Recovery exercises the durable cut after action reply settlement.
            assert_eq!(
                recover(&f.store, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                1
            );
            let (pin, session) = f
                .store
                .read(|tx| {
                    let key = format!("comparison-direct:{}", f.peer);
                    Ok((
                        tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1)?
                            .unwrap(),
                        tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?
                            .unwrap(),
                    ))
                })
                .unwrap();
            assert_eq!(pin, f.new.identity_key().serialize().to_vec());
            let record = SessionRecord::deserialize(&session).unwrap();
            assert_eq!(
                record
                    .session_state()
                    .unwrap()
                    .remote_identity_key()
                    .unwrap(),
                Some(*f.new.identity_key())
            );
            assert_eq!(record.previous_session_count(), 0); // old-key ratchet is not reused
            assert_ne!(session, f.held);
            assert_ne!(f.store.journal_head().unwrap(), head);
            let history: Vec<serde_json::Value> = f
                .store
                .read(|tx| {
                    Ok(tx
                        .scan_after(Table::Checkpoints, 1, None, 512)?
                        .into_iter()
                        .filter(|(key, _)| key.starts_with(b"identity-history:"))
                        .map(|(_, bytes)| serde_json::from_slice(&bytes).unwrap())
                        .collect())
                })
                .unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(
                history[0]["previous_identity"],
                serde_json::json!(f.old.identity_key().serialize().to_vec())
            );
            assert_eq!(history[0]["previous_session"], serde_json::json!(f.held));
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            assert!(inspection.applied);
            assert_eq!(inspection.pin_matches, Some(true));
            assert_eq!(inspection.refresh_guard, RefreshGuard::PinChanged);
            let after = snapshot(&f.store);
            for (i, table) in Table::ALL.iter().enumerate() {
                if !matches!(
                    table,
                    Table::PeerIdentities | Table::SignalSessions | Table::Checkpoints
                ) {
                    assert_eq!(before[i], after[i], "unrelated table changed: {table:?}");
                }
            }
            let progressed = crate::signal::direct::prepare_payload(
                &f.account,
                &f.peer,
                None,
                Some(&session),
                Some(&pin),
                b"fixture progress",
            )
            .await
            .unwrap();
            f.store
                .write(|tx| {
                    tx.put_json(
                        Table::SignalSessions,
                        format!("comparison-direct:{}", f.peer).as_bytes(),
                        1,
                        &*progressed.session,
                    )
                })
                .unwrap();
            let saved = snapshot(&f.store);
            let saved_head = f.store.journal_head().unwrap();
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                Application::AlreadyApplied
            );
            assert_eq!(
                recover(&f.store, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                0
            );
            assert_eq!(snapshot(&f.store), saved);
            assert_eq!(f.store.journal_head().unwrap(), saved_head);
            let path = f.dir.path().join("application.redb");
            drop(f.store);
            let reopened = open(&path);
            assert_eq!(
                recover(&reopened, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                0
            );
            assert!(
                super::super::inspect(&reopened, &f.id)
                    .unwrap()
                    .unwrap()
                    .applied
            );
            assert_eq!(snapshot(&reopened), saved);
            assert!(actions::reserve_pending(&reopened).unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn unsupported_refused_uncertain_own_and_historical_reads_preserve_every_table() {
        for case in [
            "pinned",
            "same",
            "forged",
            "queued",
            "uncertain",
            "refused",
            "companion",
            "attestation",
            "pq",
            "identity_auth",
            "untracked",
            "own_pn",
            "own_lid",
        ] {
            let f = fixture(if case == "companion" {
                "222222222222222:4@lid"
            } else if case == "own_pn" {
                "15555550111@s.whatsapp.net"
            } else {
                "222222222222222@lid"
            })
            .await;
            let mut response = if case == "same" {
                reply(&f.peer, &f.id, &f.old, true)
            } else if case == "forged" {
                reply(&f.peer, &f.id, &f.new, false)
            } else {
                f.response.clone()
            };
            if let Some(name) = match case {
                "attestation" => Some("non_e2ee_attestation"),
                "pq" => Some("pqkey"),
                "identity_auth" => Some("identity_auth"),
                _ => None,
            } {
                metadata(&mut response, name);
            }
            if case == "refused" {
                response = Node::new("iq")
                    .attr("id", f.id.as_str())
                    .attr("type", "error")
                    .children(vec![Node::new("error").attr("code", "429")]);
            }
            if case != "queued" {
                assert_eq!(actions::reserve_pending(&f.store).unwrap().len(), 1);
                if case != "uncertain" {
                    assert!(actions::record_reply(&f.store, &response).unwrap());
                }
            }
            if case == "untracked" {
                f.store
                    .write(|tx| {
                        tx.delete(
                            Table::Checkpoints,
                            format!("identity-refresh-snapshot:{}", f.id).as_bytes(),
                        )
                    })
                    .unwrap();
            }
            if matches!(case, "own_pn" | "own_lid") {
                f.store.write(|tx| tx.put_json(Table::Checkpoints, b"login-identity", 1,
                    &serde_json::json!({if case == "own_pn" {"pn"} else {"lid"}: f.peer.to_string()}))).unwrap();
            }
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            let result = apply(
                &f.store,
                &f.id,
                if case == "pinned" {
                    IdentityChangePolicy::Pinned
                } else {
                    IdentityChangePolicy::ApkAuthenticated
                },
            )
            .await
            .unwrap();
            assert!(
                matches!(result, Application::Deferred(_)),
                "{case}: {result:?}"
            );
            assert!(
                !super::super::inspect(&f.store, &f.id)
                    .unwrap()
                    .unwrap()
                    .applied
            );
            assert_eq!(snapshot(&f.store), before, "{case}");
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }

    #[tokio::test]
    async fn staged_reply_cannot_overwrite_ratchet_account_or_identity_transitions() {
        for case in ["ratchet", "account", "reply", "transition", "own_alias"] {
            let f = fixture("222222222222222@lid").await;
            settle(&f, &f.response);
            let Preparation::Ready(staged) =
                stage(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap()
            else {
                panic!("stage")
            };
            let progressed = crate::signal::direct::prepare_payload(
                &f.account,
                &f.peer,
                None,
                Some(&f.held),
                Some(&staged.pin),
                b"concurrent fixture progress",
            )
            .await
            .unwrap();
            f.store
                .write(|tx| {
                    let peer_key = format!("comparison-direct:{}", f.peer);
                    match case {
                        "ratchet" => tx.put_json(
                            Table::SignalSessions,
                            peer_key.as_bytes(),
                            1,
                            &*progressed.session,
                        ),
                        "account" => {
                            let mut a = AccountRecord::load_in_write(tx)?.unwrap();
                            a.signal_registration_id = Some(32);
                            a.save(tx)
                        }
                        "reply" => {
                            let key = format!("action-outbox:{}", f.id);
                            let mut row: serde_json::Value =
                                tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                            row["status"] = serde_json::json!("error");
                            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)
                        }
                        "own_alias" => tx.put_json(
                            Table::Checkpoints,
                            b"login-identity",
                            1,
                            &serde_json::json!({"lid":f.peer.to_string()}),
                        ),
                        "transition" => {
                            super::super::super::identity_conflicts::archive_bundle_replacement(
                                tx,
                                &f.peer,
                                Some(&staged.pin),
                                Some(&f.held),
                                &staged.identity,
                            )?;
                            super::super::super::identity_conflicts::archive_bundle_replacement(
                                tx,
                                &f.peer,
                                Some(&staged.identity),
                                None,
                                &staged.pin,
                            )
                        }
                        _ => unreachable!(),
                    }
                })
                .unwrap();
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            assert!(
                matches!(commit(&f.store, &staged).unwrap(), Application::Deferred(_)),
                "{case}"
            );
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }

    #[tokio::test]
    async fn failing_invalidation_rolls_back_version_pin_session_history_and_marker() {
        let f = fixture("222222222222222@lid").await;
        settle(&f, &f.response);
        f.store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"sender-key-distribution:broken",
                    1,
                    &serde_json::json!({}),
                )
            })
            .unwrap();
        let before = snapshot(&f.store);
        let head = f.store.journal_head().unwrap();
        let error = apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("invalid saved sender-key distribution"));
        assert_eq!(snapshot(&f.store), before);
        assert_eq!(f.store.journal_head().unwrap(), head);
        assert!(
            !super::super::inspect(&f.store, &f.id)
                .unwrap()
                .unwrap()
                .applied
        );
    }
}
