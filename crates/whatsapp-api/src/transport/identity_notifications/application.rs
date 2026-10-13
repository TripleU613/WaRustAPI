//! APK 37.71 bounded primary/classic and verified remote companion application.
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
    let same_identity = inspection.pin_matches == Some(true);
    let admitted_attestation = super::attestation::admitted(
        user,
        peer.device == 0,
        same_identity,
        inspection.attestation.as_ref(),
    );
    if user.child("non_e2ee_attestation").is_some() && !admitted_attestation {
        return Ok(deferred("attestation_admission_unported"));
    }
    // CQc.A00 0068..00b4 verifies the selected Kyber key's signature before
    // deriving with the hybrid label (pq-signature.md, hybrid-kdf.md); bundle_for
    // additionally validates the ML-KEM-1024 encoding. Companion and
    // attestation-bearing PQ replies remain local coverage gaps, not native rules.
    let hybrid = if user.child("pqkey").is_some() {
        if peer.device != 0 || user.child("non_e2ee_attestation").is_some() {
            return Ok(deferred("pq_session_unported"));
        }
        match crate::signal::hybrid::bundle_for(&reply, &peer) {
            Ok(bundle) => Some(bundle),
            Err(error) => {
                if let Error::InvalidInput { field, reason } = &error {
                    tracing::warn!(
                        field,
                        reason,
                        "identity refresh deferred: PQ bundle invalid"
                    );
                }
                return Ok(deferred("pq_bundle_invalid"));
            }
        }
    } else {
        None
    };
    // APK Bzb has NONE/LOG_ONLY only; 14J.A04 and DLx log auth after processing.
    // Preserve Bot.A03's required fields without inventing an admission gate.
    if !super::identity_auth_well_formed(user) {
        return Ok(deferred("malformed_identity_auth"));
    }
    // Bot.A03 0370..0384 sets Cai.A08 from child presence (1xi.A1U),
    // not its body or an attribute. CPB.A00 same-key absent-CWS E2EE:
    // equal old state -> Bjo; NOT_SET -> Bjn; HYBRID/NON with A08 ->
    // Bjn at 0252 -> 0306. Disabled main property -> Bjo. All admit;
    // without that server child, unknown old state/0x7497 cannot be guessed.
    if peer.device == 0
        && same_identity
        && !admitted_attestation
        && user.child("force_override_e2ee_state").is_none()
    {
        return Ok(deferred("same_identity_admission_unported"));
    }
    if inspection.pin_matches.is_none() {
        return Ok(deferred("missing_saved_identity"));
    }
    // CPB.A00: property 0x6f6f=false -> Bjo at 00cc; true + absent CWS
    // -> E2EE at 01e4, changed identity v12=1 at 00d2, then 0226 -> Bjn
    // at 0306, independent of old C0F/override/0x7497. Present CWS uses
    // the shared branch invariant above; no runtime flag/state is inferred.
    let bundle = match &hybrid {
        Some(hybrid) => hybrid.classic().clone(),
        None => crate::signal::direct::prekey_bundle_for(&reply, &peer)?,
    };
    let identity = bundle
        .identity_key()
        .map_err(|_| invalid("invalid bundle identity"))?
        .serialize()
        .to_vec();
    let pin = pin.ok_or_else(|| invalid("missing saved identity"))?;
    let companion = if peer.device != 0 {
        match store.read(|tx| {
            if same_identity {
                super::super::companion_admission::prepare_same_key_directory_in(
                    tx, &peer, user, &identity,
                )
            } else {
                super::super::companion_admission::prepare_changed_key_directory_in(
                    tx, &peer, user, &identity,
                )
            }
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
    if held.is_some_and(|held| !super::held_session_consistent(&account, &identity, held)) {
        tracing::warn!("identity refresh deferred: held session inconsistent; row kept");
        return Ok(deferred("held_session_inconsistent"));
    }
    let session = match &hybrid {
        Some(hybrid) => hybrid.prepare_session_refresh(&account, held.map(Vec::as_slice))?,
        None => {
            crate::signal::direct::prepare_session_refresh(
                &account,
                &peer,
                &bundle,
                held.map(Vec::as_slice),
                &identity,
            )
            .await?
        }
    };
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
        if let Some(authority) = &staged.companion {
            // Directory signatures and session derivation were checked before
            // this transaction; a newer ADV requests sync without moving its
            // verified floor or replacing an existing membership index. An
            // absent known device is added only after full signed ADV admission.
            authority.commit(tx)?;
        }
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
    fn attestation(response: &mut Node, key: &IdentityKeyPair, kind: &str, valid: bool) {
        let body = vec![8, if kind == "hybrid_e2ee" { 1 } else { 2 }, 16, 7];
        let mut signature = key
            .private_key()
            .calculate_signature(&[vec![6, 7], body.clone()].concat(), &mut rand::rng())
            .unwrap()
            .to_vec();
        if !valid {
            signature[0] ^= 1;
        }
        metadata(response, "non_e2ee_attestation");
        let NodeContent::Children(lists) = &mut response.content else {
            panic!()
        };
        let NodeContent::Children(users) = &mut lists[0].content else {
            panic!()
        };
        let NodeContent::Children(fields) = &mut users[0].content else {
            panic!()
        };
        *fields.last_mut().unwrap() = Node::new("non_e2ee_attestation")
            .attr("account_type", kind)
            .children(vec![
                Node::new("value").bytes(body),
                Node::new("signature").bytes(signature),
            ]);
    }

    #[tokio::test]
    async fn primary_attestation_admission_refreshes_guarded_sessions_once() {
        for peer in ["15555550111@s.whatsapp.net", "222222222222222@lid"] {
            for same in [false, true] {
                for kind in ["hybrid_e2ee", "non_e2ee"] {
                    for valid in [true, false] {
                        let f = fixture(peer).await;
                        let candidate = if same { &f.old } else { &f.new };
                        let mut response = reply(&f.peer, &f.id, candidate, true);
                        attestation(&mut response, candidate, kind, valid);
                        settle(&f, &response);
                        let before = snapshot(&f.store);
                        let head = f.store.journal_head().unwrap();
                        let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
                        assert_eq!(inspection.signature_valid, Some(true));
                        assert_eq!(inspection.attestation.unwrap().signature_valid, Some(valid));
                        let accepted = !same || valid;
                        let preview = super::super::preview_session(&f.store, &f.id)
                            .await
                            .unwrap();
                        assert_eq!(
                            preview.outcome,
                            if !accepted {
                                "attestation_admission_unported"
                            } else {
                                "prepared"
                            }
                        );
                        assert_eq!(
                            preview.archived_states,
                            accepted.then_some(usize::from(same))
                        );
                        assert!(!preview.admission_evaluated && !preview.persisted);
                        assert_eq!(snapshot(&f.store), before);
                        assert_eq!(f.store.journal_head().unwrap(), head);
                        assert_eq!(
                            apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                                .await
                                .unwrap(),
                            if accepted {
                                Application::Applied
                            } else {
                                Application::Deferred("attestation_admission_unported")
                            },
                            "{peer} same={same} {kind} valid={valid}"
                        );
                        if !accepted {
                            assert_eq!(snapshot(&f.store), before);
                            assert_eq!(f.store.journal_head().unwrap(), head);
                            continue;
                        }
                        let after = snapshot(&f.store);
                        for (index, table) in Table::ALL.iter().enumerate() {
                            if !matches!(
                                table,
                                Table::PeerIdentities | Table::SignalSessions | Table::Checkpoints
                            ) {
                                assert_eq!(after[index], before[index], "{table:?}");
                            }
                        }
                        let (pin, session) = f
                            .store
                            .read(|tx| {
                                Ok((
                                    tx.get_json::<Vec<u8>>(
                                        Table::PeerIdentities,
                                        format!("comparison-direct:{}", f.peer).as_bytes(),
                                        1,
                                    )?
                                    .unwrap(),
                                    tx.get_json::<Vec<u8>>(
                                        Table::SignalSessions,
                                        format!("comparison-direct:{}", f.peer).as_bytes(),
                                        1,
                                    )?
                                    .unwrap(),
                                ))
                            })
                            .unwrap();
                        assert_eq!(pin, candidate.identity_key().serialize().to_vec());
                        let record = SessionRecord::deserialize(&session).unwrap();
                        let old = SessionRecord::deserialize(&f.held).unwrap();
                        assert_eq!(
                            record.previous_session_count(),
                            if same {
                                old.previous_session_count() + 1
                            } else {
                                0
                            }
                        );
                        if same {
                            assert_eq!(
                                record
                                    .previous_session_states()
                                    .next()
                                    .unwrap()
                                    .unwrap()
                                    .alice_base_key(),
                                old.session_state().unwrap().alice_base_key()
                            );
                        }
                        assert_eq!(f.store.journal_head().unwrap() == head, same);
                        assert_eq!(
                            apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                                .await
                                .unwrap(),
                            Application::AlreadyApplied
                        );
                        assert_eq!(snapshot(&f.store), after);
                        let head = f.store.journal_head().unwrap();
                        let path = f.dir.path().join("application.redb");
                        drop(f.store);
                        let store = open(&path);
                        assert_eq!(
                            recover(&store, IdentityChangePolicy::ApkAuthenticated)
                                .await
                                .unwrap(),
                            0
                        );
                        assert_eq!(snapshot(&store), after);
                        assert_eq!(store.journal_head().unwrap(), head);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn changed_primary_attestation_fallback_requires_unambiguous_byte_fields() {
        for (node, accepted) in [
            (
                Node::new("non_e2ee_attestation").children(vec![
                    Node::new("value").bytes(vec![]),
                    Node::new("signature").bytes(vec![]),
                ]),
                true,
            ),
            (
                Node::new("non_e2ee_attestation")
                    .attr("account_type", "unknown")
                    .children(vec![
                        Node::new("value").bytes(vec![255]),
                        Node::new("signature").bytes(vec![0]),
                    ]),
                true,
            ),
            (
                Node::new("non_e2ee_attestation")
                    .attr("account_type", "hybrid_e2ee")
                    .children(vec![
                        Node::new("value").bytes(vec![8, 2]),
                        Node::new("signature").bytes(vec![0; 64]),
                    ]),
                true,
            ),
            (
                Node::new("non_e2ee_attestation")
                    .attr("account_type", "hybrid_e2ee")
                    .children(vec![
                        Node::new("value").bytes(vec![255]),
                        Node::new("signature").bytes(vec![0; 64]),
                    ]),
                true,
            ),
            (
                Node::new("non_e2ee_attestation")
                    .attr("account_type", "hybrid_e2ee")
                    .children(vec![
                        Node::new("value").bytes(vec![0; 65537]),
                        Node::new("signature").bytes(vec![0; 64]),
                    ]),
                true,
            ),
            (
                Node::new("non_e2ee_attestation")
                    .children(vec![Node::new("signature").bytes(vec![])]),
                false,
            ),
            (
                Node::new("non_e2ee_attestation").children(vec![Node::new("value").bytes(vec![])]),
                false,
            ),
            (
                Node::new("non_e2ee_attestation").children(vec![
                    Node::new("value").children(vec![]),
                    Node::new("signature").bytes(vec![]),
                ]),
                false,
            ),
            (
                Node::new("non_e2ee_attestation").children(vec![
                    Node::new("value").bytes(vec![]),
                    Node::new("signature").bytes(vec![]),
                    Node::new("value").bytes(vec![]),
                ]),
                false,
            ),
            (
                Node::new("non_e2ee_attestation").children(vec![
                    Node::new("value").bytes(vec![]),
                    Node::new("signature").bytes(vec![]),
                    Node::new("signature").bytes(vec![]),
                ]),
                false,
            ),
        ] {
            let f = fixture("222222222222222@lid").await;
            let mut response = f.response.clone();
            metadata(&mut response, "non_e2ee_attestation");
            let NodeContent::Children(lists) = &mut response.content else {
                panic!()
            };
            let NodeContent::Children(users) = &mut lists[0].content else {
                panic!()
            };
            let NodeContent::Children(fields) = &mut users[0].content else {
                panic!()
            };
            *fields.last_mut().unwrap() = node;
            settle(&f, &response);
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            assert_ne!(
                inspection.attestation.unwrap().status,
                super::super::AttestationStatus::Verified
            );
            let preview = super::super::preview_session(&f.store, &f.id)
                .await
                .unwrap();
            assert_eq!(
                preview.outcome,
                if accepted {
                    "prepared"
                } else {
                    "attestation_admission_unported"
                }
            );
            assert!(!preview.admission_evaluated && !preview.persisted);
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                if accepted {
                    Application::Applied
                } else {
                    Application::Deferred("attestation_admission_unported")
                }
            );
            if !accepted {
                assert_eq!(snapshot(&f.store), before);
                assert_eq!(f.store.journal_head().unwrap(), head);
            }
        }
    }

    #[tokio::test]
    async fn primary_attestation_keeps_classic_policy_and_metadata_guards() {
        for (case, reason) in [
            ("forged", "reply_not_verified"),
            ("pinned", "identity_policy_pinned"),
            ("untracked", "original_refresh_guard_not_matching"),
            ("pqkey", "pq_session_unported"),
            ("companion", "missing_device_identity"),
            ("duplicate", "attestation_admission_unported"),
            ("same_invalid_override", "attestation_admission_unported"),
        ] {
            let f = fixture(if case == "companion" {
                "222222222222222:4@lid"
            } else {
                "222222222222222@lid"
            })
            .await;
            let key = if case == "same_invalid_override" {
                &f.old
            } else {
                &f.new
            };
            let mut response = reply(&f.peer, &f.id, key, case != "forged");
            attestation(
                &mut response,
                key,
                "hybrid_e2ee",
                case != "same_invalid_override",
            );
            if case == "duplicate" {
                attestation(&mut response, key, "hybrid_e2ee", true);
            }
            if case == "pqkey" {
                metadata(&mut response, "pqkey");
            }
            if case == "same_invalid_override" {
                metadata(&mut response, "force_override_e2ee_state");
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
                Application::Deferred(reason)
            );
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }

    /// A directory reply with a known signed key and a signed ML-KEM-1024 key.
    fn pq_reply(
        peer: &Jid,
        id: &str,
        identity: &IdentityKeyPair,
        signed: &wacore_libsignal::protocol::KeyPair,
        kem_public: &[u8],
        kem_signature_valid: bool,
    ) -> (Node, Vec<u8>) {
        let sign = |bytes: &[u8]| {
            identity
                .private_key()
                .calculate_signature(bytes, &mut rand::rng())
                .unwrap()
                .to_vec()
        };
        let signed_signature = sign(&signed.public_key.serialize());
        let mut kem_signature = sign(kem_public);
        if !kem_signature_valid {
            kem_signature[0] ^= 1;
        }
        let response = Node::new("iq")
            .attr("type", "result")
            .attr("id", id)
            .children(vec![Node::new("list").children(vec![
                Node::new("user").attr("jid", peer.clone()).children(vec![
                    Node::new("registration").bytes(vec![0, 0, 0, 42]),
                    Node::new("identity").bytes(identity.public_key().public_key_bytes().to_vec()),
                    Node::new("skey").children(vec![
                        Node::new("id").bytes(vec![0, 0, 7]),
                        Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                        Node::new("signature").bytes(signed_signature.clone()),
                    ]),
                    Node::new("pqkey").children(vec![
                        Node::new("id").bytes(vec![0, 0, 9]),
                        Node::new("value").bytes(kem_public.to_vec()),
                        Node::new("signature").bytes(kem_signature),
                    ]),
                ]),
            ])]);
        (response, signed_signature)
    }

    #[tokio::test]
    async fn primary_pq_refresh_stages_hybrid_sessions_the_peer_can_open() {
        use crate::signal::{direct, hybrid, kem};
        use wacore_libsignal::protocol::{
            GenericSignedPreKey, KeyPair, SignedPreKeyRecord, Timestamp,
        };
        for peer in ["15555550111@s.whatsapp.net", "222222222222222@lid"] {
            for same in [false, true] {
                let f = fixture(peer).await;
                let candidate = if same { &f.old } else { &f.new };
                let signed = KeyPair::generate(&mut rand::rng());
                let (kem_public, kem_secret) = kem::generate_key_pair().unwrap();
                let (mut response, signed_signature) =
                    pq_reply(&f.peer, &f.id, candidate, &signed, &kem_public, true);
                if same {
                    // Existing same-key primary admission still requires it.
                    metadata(&mut response, "force_override_e2ee_state");
                }
                settle(&f, &response);
                let before = snapshot(&f.store);
                let head = f.store.journal_head().unwrap();
                let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
                let pq = inspection.pq_signature.as_ref().unwrap();
                assert_eq!(pq.signature_valid, Some(true));
                assert_eq!(pq.key_validation, "valid_ml_kem_1024");
                let preview = super::super::preview_session(&f.store, &f.id)
                    .await
                    .unwrap();
                assert_eq!(preview.outcome, "prepared", "{peer} same={same}");
                assert_eq!(preview.archived_states, Some(usize::from(same)));
                assert!(!preview.persisted);
                assert_eq!(snapshot(&f.store), before);
                assert_eq!(f.store.journal_head().unwrap(), head);
                assert_eq!(
                    apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                        .await
                        .unwrap(),
                    Application::Applied,
                    "{peer} same={same}"
                );
                let after = snapshot(&f.store);
                for (index, table) in Table::ALL.iter().enumerate() {
                    if !matches!(
                        table,
                        Table::PeerIdentities | Table::SignalSessions | Table::Checkpoints
                    ) {
                        assert_eq!(after[index], before[index], "{table:?}");
                    }
                }
                let peer_key = format!("comparison-direct:{}", f.peer);
                let (pin, session): (Option<Vec<u8>>, Option<Vec<u8>>) = f
                    .store
                    .read(|tx| {
                        Ok((
                            tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
                            tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
                        ))
                    })
                    .unwrap();
                assert_eq!(
                    pin.as_deref(),
                    Some(&candidate.identity_key().serialize()[..])
                );
                let session = session.unwrap();
                let record = SessionRecord::deserialize(&session).unwrap();
                assert_eq!(record.session_version().unwrap(), 4);
                // Changed keys never carry old state; same keys archive it.
                assert_eq!(record.previous_session_count(), usize::from(same));
                // The refreshed record opens a native 0x44 exchange the peer
                // can authenticate with its signed and KEM private keys.
                let first = hybrid::HybridSession::from_bytes(&session)
                    .unwrap()
                    .prepare(b"\x0a\x05hello")
                    .unwrap();
                assert_eq!((first.kind, first.ciphertext[0]), ("pkmsg", 0x44));
                assert_eq!(
                    direct::incoming_prekey_kem(&first.ciphertext)
                        .unwrap()
                        .unwrap()
                        .prekey_id,
                    9
                );
                let peer_account = AccountRecord {
                    registration: crate::session::RegistrationStatus::Registered,
                    signal_registration_id: Some(42),
                    signal_identity: Some(SignalIdentityKey(KeyMaterial {
                        public: candidate
                            .public_key()
                            .public_key_bytes()
                            .try_into()
                            .unwrap(),
                        private: *candidate.private_key().serialize(),
                    })),
                    ..Default::default()
                };
                let signed_record = SignedPreKeyRecord::new(
                    7.into(),
                    Timestamp::from_epoch_millis(0),
                    &signed,
                    &signed_signature,
                )
                .serialize()
                .unwrap();
                let opened = hybrid::receive_prekey(
                    &peer_account,
                    f.account.own_jid.as_ref().unwrap(),
                    (None, None, IdentityChangePolicy::Pinned),
                    Some(&signed_record),
                    None,
                    Some(hybrid::LocalKemPrekey {
                        id: 9,
                        secret: kem_secret.as_bytes(),
                        last_resort: false,
                    }),
                    &first.ciphertext,
                )
                .await
                .unwrap();
                assert_eq!(
                    opened.incoming.received.plaintext.as_slice(),
                    b"\x0a\x05hello"
                );
                assert_eq!(
                    apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                        .await
                        .unwrap(),
                    Application::AlreadyApplied
                );
                assert_eq!(snapshot(&f.store), after);
            }
        }
    }

    #[tokio::test]
    async fn primary_pq_refresh_keeps_invalid_and_unported_inputs_unchanged() {
        use crate::signal::kem;
        use wacore_libsignal::protocol::KeyPair;
        for (case, reason, preview_reason) in [
            (
                "bad_kem_signature",
                "pq_bundle_invalid",
                "pq_bundle_invalid",
            ),
            ("bad_kem_encoding", "pq_bundle_invalid", "pq_bundle_invalid"),
            ("attestation", "pq_session_unported", "pq_session_unported"),
            (
                "companion",
                "pq_session_unported",
                "missing_device_identity",
            ),
            (
                "same_without_override",
                "same_identity_admission_unported",
                "prepared",
            ),
            ("pinned", "identity_policy_pinned", "prepared"),
        ] {
            let f = fixture(if case == "companion" {
                "222222222222222:4@lid"
            } else {
                "222222222222222@lid"
            })
            .await;
            let candidate = if case == "same_without_override" {
                &f.old
            } else {
                &f.new
            };
            let (mut kem_public, _) = kem::generate_key_pair().unwrap();
            if case == "bad_kem_encoding" {
                // 12-bit coefficients of 0xfff exceed q = 3329.
                kem_public = vec![0xff; kem_public.len()];
            }
            let (mut response, _) = pq_reply(
                &f.peer,
                &f.id,
                candidate,
                &KeyPair::generate(&mut rand::rng()),
                &kem_public,
                case != "bad_kem_signature",
            );
            if case == "attestation" {
                attestation(&mut response, candidate, "hybrid_e2ee", true);
            }
            settle(&f, &response);
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            let pq = inspection.pq_signature.as_ref().unwrap();
            assert_eq!(pq.signature_valid, Some(case != "bad_kem_signature"));
            assert_eq!(
                pq.key_validation,
                if case == "bad_kem_encoding" {
                    "invalid_ml_kem_1024"
                } else {
                    "valid_ml_kem_1024"
                }
            );
            let preview = super::super::preview_session(&f.store, &f.id)
                .await
                .unwrap();
            assert_eq!(preview.outcome, preview_reason, "{case}");
            assert_eq!(snapshot(&f.store), before);
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
                Application::Deferred(reason),
                "{case}"
            );
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }

    #[tokio::test]
    async fn damaged_held_sessions_defer_without_stopping_recovery() {
        use crate::signal::{direct, kem};
        use wacore_libsignal::protocol::KeyPair;
        for pq in [false, true] {
            for case in ["garbage", "foreign_remote", "foreign_local"] {
                let f = fixture("222222222222222@lid").await;
                let damaged = if case == "garbage" {
                    vec![1u8, 2, 3]
                } else {
                    let other = IdentityKeyPair::generate(&mut rand::rng());
                    let mut account = f.account.clone();
                    let remote = if case == "foreign_remote" {
                        &other
                    } else {
                        account.signal_identity = Some(SignalIdentityKey(KeyMaterial {
                            public: other.public_key().public_key_bytes().try_into().unwrap(),
                            private: *other.private_key().serialize(),
                        }));
                        &f.old
                    };
                    let bundle = direct::prekey_bundle_for(
                        &reply(&f.peer, "damaged", remote, true),
                        &f.peer,
                    )
                    .unwrap();
                    direct::prepare_payload(
                        &account,
                        &f.peer,
                        Some(&bundle),
                        None,
                        Some(&remote.identity_key().serialize()),
                        b"damaged",
                    )
                    .await
                    .unwrap()
                    .session
                    .to_vec()
                };
                f.store
                    .write(|tx| {
                        tx.put_json(
                            Table::SignalSessions,
                            format!("comparison-direct:{}", f.peer).as_bytes(),
                            1,
                            &damaged,
                        )
                    })
                    .unwrap();
                let mut response = if pq {
                    let (kem_public, _) = kem::generate_key_pair().unwrap();
                    let signed = KeyPair::generate(&mut rand::rng());
                    pq_reply(&f.peer, &f.id, &f.old, &signed, &kem_public, true).0
                } else {
                    reply(&f.peer, &f.id, &f.old, true)
                };
                // Same-key primary admission needs the server override child.
                metadata(&mut response, "force_override_e2ee_state");
                settle(&f, &response);
                let before = snapshot(&f.store);
                let head = f.store.journal_head().unwrap();
                let preview = super::super::preview_session(&f.store, &f.id)
                    .await
                    .unwrap();
                assert_eq!(
                    preview.outcome, "held_session_inconsistent",
                    "{case} pq={pq}"
                );
                assert_eq!(
                    apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                        .await
                        .unwrap(),
                    Application::Deferred("held_session_inconsistent"),
                    "{case} pq={pq}"
                );
                // recover() runs at every connection start and must not fail.
                assert_eq!(
                    recover(&f.store, IdentityChangePolicy::ApkAuthenticated)
                        .await
                        .unwrap(),
                    0
                );
                assert_eq!(snapshot(&f.store), before);
                assert_eq!(f.store.journal_head().unwrap(), head);
            }
        }
    }

    fn install_companion_authority(f: &Fixture) -> IdentityKeyPair {
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
        key
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
    // Grz fields 1..4 and GsB 1=rawId/2=timestamp/3=keyIndex are APK 37.71
    // descriptors; this generated fixture binds the directory key directly,
    // without manufacturing a receive prekey envelope or native login state.
    fn signed_companion_adv(
        primary: &IdentityKeyPair,
        companion: &IdentityKeyPair,
        raw_id: u32,
        timestamp: u64,
        key_index: u32,
    ) -> Vec<u8> {
        use prost::Message as _;
        let details = crate::proto::wa_adv::AdvDeviceIdentity {
            raw_id: Some(raw_id),
            timestamp: Some(timestamp),
            key_index: Some(key_index),
            account_type: None,
            device_type: None,
        }
        .encode_to_vec();
        let remote = companion.public_key().public_key_bytes();
        let account = primary.public_key().public_key_bytes();
        let account_sig = primary
            .private_key()
            .calculate_signature(&[&[6, 0][..], &details, remote].concat(), &mut rand::rng())
            .unwrap();
        let device_sig = companion
            .private_key()
            .calculate_signature(
                &[&[6, 1][..], &details, remote, account].concat(),
                &mut rand::rng(),
            )
            .unwrap();
        crate::proto::wa_adv::AdvSignedDeviceIdentity {
            details: Some(details.into()),
            account_signature_key: None,
            account_signature: Some(account_sig.to_vec().into()),
            device_signature: Some(device_sig.to_vec().into()),
        }
        .encode_to_vec()
    }
    #[tokio::test]
    async fn companion_attestation_skips_primary_policy_without_skipping_adv() {
        for peer in ["15555550111:4@s.whatsapp.net", "222222222222222:4@lid"] {
            for same in [false, true] {
                for (kind, valid) in [("hybrid_e2ee", true), ("non_e2ee", false)] {
                    let f = fixture(peer).await;
                    let primary = install_companion_authority(&f);
                    let candidate = if same { &f.old } else { &f.new };
                    let mut response = reply(&f.peer, &f.id, candidate, true);
                    device_identity(
                        &mut response,
                        NodeContent::Bytes(
                            if same {
                                vec![]
                            } else {
                                signed_companion_adv(&primary, candidate, 8, 9, 7)
                            }
                            .into(),
                        ),
                    );
                    attestation(&mut response, candidate, kind, valid);
                    settle(&f, &response);
                    let before = snapshot(&f.store);
                    let head = f.store.journal_head().unwrap();
                    let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
                    assert_eq!(inspection.attestation.unwrap().signature_valid, Some(valid));
                    let preview = super::super::preview_session(&f.store, &f.id)
                        .await
                        .unwrap();
                    assert_eq!(preview.outcome, "prepared");
                    assert_eq!(preview.archived_states, Some(usize::from(same)));
                    assert!(!preview.persisted && !preview.admission_evaluated);
                    assert_eq!(snapshot(&f.store), before);
                    assert_eq!(f.store.journal_head().unwrap(), head);
                    assert_eq!(
                        apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                            .await
                            .unwrap(),
                        Application::Applied,
                        "{peer} same={same} {kind} valid={valid}"
                    );
                    let after = snapshot(&f.store);
                    for (index, table) in Table::ALL.iter().enumerate() {
                        if !matches!(
                            table,
                            Table::PeerIdentities | Table::SignalSessions | Table::Checkpoints
                        ) {
                            assert_eq!(after[index], before[index], "{table:?}");
                        }
                    }
                    let (pin, session, primary_pin) = f
                        .store
                        .read(|tx| {
                            Ok((
                                tx.get_json::<Vec<u8>>(
                                    Table::PeerIdentities,
                                    format!("comparison-direct:{}", f.peer).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                                tx.get_json::<Vec<u8>>(
                                    Table::SignalSessions,
                                    format!("comparison-direct:{}", f.peer).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                                tx.get_json::<Vec<u8>>(
                                    Table::PeerIdentities,
                                    format!("comparison-direct:{}", f.peer.to_non_ad()).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                            ))
                        })
                        .unwrap();
                    assert_eq!(pin, candidate.identity_key().serialize().to_vec());
                    assert_eq!(primary_pin, primary.identity_key().serialize().to_vec());
                    assert_eq!(
                        SessionRecord::deserialize(&session)
                            .unwrap()
                            .previous_session_count(),
                        if same {
                            SessionRecord::deserialize(&f.held)
                                .unwrap()
                                .previous_session_count()
                                + 1
                        } else {
                            0
                        }
                    );
                    let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
                    assert!(inspection.applied);
                    assert_eq!(inspection.attestation.unwrap().signature_valid, Some(valid));
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
        }
    }
    #[tokio::test]
    async fn returning_pinned_companion_commits_signed_addition_and_session_once() {
        for peer in ["15555550111:4@s.whatsapp.net", "222222222222222:4@lid"] {
            for changed in [false, true] {
                for registry in [0, 1, 2] {
                    let f = fixture(peer).await;
                    let primary = install_companion_authority(&f);
                    change_list(&f, |l| {
                        l.devices.retain(|d| d.jid.device == 0);
                        l.pending_removed_devices = if registry == 0 { None } else { Some(vec![]) };
                        if registry == 2 {
                            l.adv_added_devices = vec![f.peer.clone()];
                        }
                        l.notified_hash = Some("unchanged-notification-hash".to_owned());
                        l.hash_matches = Some(true);
                    });
                    let baseline = super::super::super::devices::device_list(&f.store, &f.peer)
                        .unwrap()
                        .unwrap();
                    let candidate = if changed { &f.new } else { &f.old };
                    let mut response = reply(&f.peer, &f.id, candidate, true);
                    device_identity(
                        &mut response,
                        NodeContent::Bytes(
                            signed_companion_adv(&primary, candidate, 8, 10, 6).into(),
                        ),
                    );
                    attestation(&mut response, candidate, "non_e2ee", false);
                    settle(&f, &response);
                    let inspected = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
                    assert_eq!(
                        if changed {
                            inspected.companion_changed_key_authority
                        } else {
                            inspected.companion_same_key_authority
                        },
                        Some("ready_addition_pending")
                    );
                    let before = snapshot(&f.store);
                    let head = f.store.journal_head().unwrap();
                    let preview = super::super::preview_session(&f.store, &f.id)
                        .await
                        .unwrap();
                    assert_eq!(preview.outcome, "prepared");
                    assert_eq!(preview.archived_states, Some(usize::from(!changed)));
                    assert!(!preview.persisted && !preview.admission_evaluated);
                    assert_eq!(snapshot(&f.store), before);
                    assert_eq!(f.store.journal_head().unwrap(), head);
                    assert_eq!(
                        recover(&f.store, IdentityChangePolicy::ApkAuthenticated)
                            .await
                            .unwrap(),
                        1
                    );
                    let mut expected = baseline;
                    expected
                        .devices
                        .push(super::super::super::devices::DeviceEntry {
                            jid: f.peer.clone(),
                            key_index: 6,
                        });
                    expected.devices.sort_by(|a, b| a.jid.cmp(&b.jid));
                    expected.adv_added_devices = vec![f.peer.clone()];
                    expected.expected_timestamp = Some(10);
                    expected.hash_matches = None;
                    assert_eq!(
                        super::super::super::devices::device_list(&f.store, &f.peer).unwrap(),
                        Some(expected)
                    );
                    assert_eq!(
                        super::super::super::devices::pending_users(&f.store).unwrap(),
                        vec![f.peer.to_non_ad()]
                    );
                    let (pin, session, primary_pin) = f
                        .store
                        .read(|tx| {
                            Ok((
                                tx.get_json::<Vec<u8>>(
                                    Table::PeerIdentities,
                                    format!("comparison-direct:{}", f.peer).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                                tx.get_json::<Vec<u8>>(
                                    Table::SignalSessions,
                                    format!("comparison-direct:{}", f.peer).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                                tx.get_json::<Vec<u8>>(
                                    Table::PeerIdentities,
                                    format!("comparison-direct:{}", f.peer.to_non_ad()).as_bytes(),
                                    1,
                                )?
                                .unwrap(),
                            ))
                        })
                        .unwrap();
                    assert_eq!(pin, candidate.identity_key().serialize().to_vec());
                    assert_eq!(primary_pin, primary.identity_key().serialize().to_vec());
                    let record = SessionRecord::deserialize(&session).unwrap();
                    let old = SessionRecord::deserialize(&f.held).unwrap();
                    assert_eq!(
                        record.previous_session_count(),
                        if changed {
                            0
                        } else {
                            old.previous_session_count() + 1
                        }
                    );
                    assert_ne!(
                        record.session_state().unwrap().alice_base_key(),
                        old.session_state().unwrap().alice_base_key()
                    );
                    if !changed {
                        assert_eq!(
                            record
                                .previous_session_states()
                                .next()
                                .unwrap()
                                .unwrap()
                                .alice_base_key(),
                            old.session_state().unwrap().alice_base_key()
                        );
                    }
                    assert_eq!(f.store.journal_head().unwrap() == head, !changed);
                    let after = snapshot(&f.store);
                    for (i, t) in Table::ALL.iter().enumerate() {
                        if !matches!(
                            t,
                            Table::PeerIdentities | Table::SignalSessions | Table::Checkpoints
                        ) {
                            assert_eq!(before[i], after[i], "{t:?}");
                        }
                    }
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
        }
    }
    #[tokio::test]
    async fn returning_companion_never_uses_equal_key_shortcut_without_membership() {
        for changed in [false, true] {
            for (case, expected) in [
                ("empty", "invalid_device_identity"),
                ("missing", "missing_device_identity"),
                ("old", "deleted_device"),
                ("equal", "deleted_device"),
                ("negative", "deleted_device"),
                ("signature", "invalid_device_identity"),
                ("primary", "invalid_device_identity"),
                ("raw_id", "raw_id_mismatch"),
                ("teardown", "pending_device_teardown"),
                ("index_collision", "conflicting_device_membership"),
                ("overflow", "conflicting_device_membership"),
                ("zero_index", "invalid_companion_index"),
                ("high_index", "invalid_companion_index"),
                ("non_e2ee_attestation", "attestation_admission_unported"),
                ("untracked", "original_refresh_guard_not_matching"),
                ("pinned", "identity_policy_pinned"),
            ] {
                let f = fixture("222222222222222:4@lid").await;
                let primary = install_companion_authority(&f);
                change_list(&f, |l| {
                    l.devices.retain(|d| d.jid.device == 0);
                    if case == "teardown" {
                        l.pending_removed_devices = Some(vec![f.peer.clone()]);
                    }
                    if case == "index_collision" {
                        l.devices.push(super::super::super::devices::DeviceEntry {
                            jid: f.peer.with_device(5),
                            key_index: 6,
                        });
                    }
                    if case == "overflow" {
                        l.devices.extend((100..1123).map(|n| {
                            super::super::super::devices::DeviceEntry {
                                jid: f.peer.with_device(n),
                                key_index: u32::from(n),
                            }
                        }));
                    }
                });
                let candidate = if changed { &f.new } else { &f.old };
                let other = IdentityKeyPair::generate(&mut rand::rng());
                let mut bytes = signed_companion_adv(
                    if case == "primary" { &other } else { &primary },
                    candidate,
                    if case == "raw_id" { 9 } else { 8 },
                    match case {
                        "old" => 8,
                        "equal" => 9,
                        "negative" => u64::MAX,
                        _ => 10,
                    },
                    match case {
                        "zero_index" => 0,
                        "high_index" => u32::MAX,
                        _ => 6,
                    },
                );
                if case == "signature" {
                    *bytes.last_mut().unwrap() ^= 1;
                }
                let mut response = reply(&f.peer, &f.id, candidate, true);
                if case != "missing" {
                    device_identity(
                        &mut response,
                        NodeContent::Bytes(if case == "empty" { vec![] } else { bytes }.into()),
                    );
                }
                if case == "non_e2ee_attestation" {
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
                if !matches!(case, "untracked" | "pinned") {
                    let preview = super::super::preview_session(&f.store, &f.id)
                        .await
                        .unwrap();
                    assert_eq!(preview.outcome, expected, "{case}/{changed}");
                    assert_eq!(preview.archived_states, None);
                    assert!(!preview.persisted && !preview.admission_evaluated);
                    assert_eq!(snapshot(&f.store), before);
                    assert_eq!(f.store.journal_head().unwrap(), head);
                }
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
                    Application::Deferred(expected),
                    "{case}/{changed}"
                );
                assert_eq!(snapshot(&f.store), before, "{case}/{changed}");
                assert_eq!(f.store.journal_head().unwrap(), head);
            }
        }
    }
    #[tokio::test]
    async fn returning_companion_addition_races_and_sync_failure_preserve_all_state() {
        for changed in [false, true] {
            for case in ["membership", "primary", "ratchet", "sync_failure"] {
                let f = fixture("222222222222222:4@lid").await;
                let primary = install_companion_authority(&f);
                change_list(&f, |l| l.devices.retain(|d| d.jid.device == 0));
                let candidate = if changed { &f.new } else { &f.old };
                let mut response = reply(&f.peer, &f.id, candidate, true);
                device_identity(
                    &mut response,
                    NodeContent::Bytes(signed_companion_adv(&primary, candidate, 8, 10, 6).into()),
                );
                settle(&f, &response);
                if case == "sync_failure" {
                    f.store
                        .write(|tx| {
                            tx.put_json(
                                Table::Checkpoints,
                                format!("device-sync-pending:{}", f.peer.to_non_ad()).as_bytes(),
                                1,
                                &serde_json::json!({"malformed":true}),
                            )
                        })
                        .unwrap();
                }
                let Preparation::Ready(staged) =
                    stage(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                        .await
                        .unwrap()
                else {
                    panic!("stage")
                };
                match case {
                    "membership" => change_list(&f, |l| {
                        l.devices.push(super::super::super::devices::DeviceEntry {
                            jid: f.peer.clone(),
                            key_index: 7,
                        })
                    }),
                    "primary" => {
                        install_pin(&f.store, &f.peer.to_non_ad());
                    }
                    "ratchet" => {
                        let progressed = crate::signal::direct::prepare_payload(
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
                                    &*progressed.session,
                                )
                            })
                            .unwrap();
                    }
                    _ => {}
                }
                let before = snapshot(&f.store);
                let head = f.store.journal_head().unwrap();
                if case == "sync_failure" {
                    assert!(commit(&f.store, &staged).is_err());
                } else {
                    assert_eq!(
                        commit(&f.store, &staged).unwrap(),
                        Application::Deferred(if case == "ratchet" {
                            "peer_state_changed_before_commit"
                        } else {
                            "companion_authority_changed_before_commit"
                        })
                    );
                }
                assert_eq!(snapshot(&f.store), before, "{case}/{changed}");
                assert_eq!(f.store.journal_head().unwrap(), head);
            }
        }
    }
    #[tokio::test]
    async fn changed_companion_adv_adopts_fresh_key_preserves_primary_and_recovers_once() {
        for peer in ["15555550111:4@s.whatsapp.net", "222222222222222:4@lid"] {
            // Existing-index lower/equal/newer timestamps; different-index
            // newer candidate preserves the old add-only membership index.
            for (timestamp, index) in [(8, 7), (9, 7), (10, 7), (10, 6), (u64::MAX, 7)] {
                let f = fixture(peer).await;
                let primary = install_companion_authority(&f);
                if timestamp == 10 {
                    change_list(&f, |l| l.expected_timestamp = Some(u64::MAX));
                }
                let original_list = super::super::super::devices::device_list(&f.store, &f.peer)
                    .unwrap()
                    .unwrap();
                let mut response = f.response.clone();
                device_identity(
                    &mut response,
                    NodeContent::Bytes(
                        signed_companion_adv(&primary, &f.new, 8, timestamp, index).into(),
                    ),
                );
                settle(&f, &response);
                let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
                assert_eq!(
                    inspection.companion_changed_key_authority,
                    Some(if timestamp == 10 {
                        "ready_refresh_pending"
                    } else {
                        "ready"
                    })
                );
                let before = snapshot(&f.store);
                let head = f.store.journal_head().unwrap();
                assert_eq!(
                    recover(&f.store, IdentityChangePolicy::ApkAuthenticated)
                        .await
                        .unwrap(),
                    1
                );
                let (pin, session, primary_pin) = f
                    .store
                    .read(|tx| {
                        Ok((
                            tx.get_json::<Vec<u8>>(
                                Table::PeerIdentities,
                                format!("comparison-direct:{}", f.peer).as_bytes(),
                                1,
                            )?
                            .unwrap(),
                            tx.get_json::<Vec<u8>>(
                                Table::SignalSessions,
                                format!("comparison-direct:{}", f.peer).as_bytes(),
                                1,
                            )?
                            .unwrap(),
                            tx.get_json::<Vec<u8>>(
                                Table::PeerIdentities,
                                format!("comparison-direct:{}", f.peer.to_non_ad()).as_bytes(),
                                1,
                            )?
                            .unwrap(),
                        ))
                    })
                    .unwrap();
                assert_eq!(pin, f.new.identity_key().serialize().to_vec());
                assert_eq!(primary_pin, primary.identity_key().serialize().to_vec());
                let record = SessionRecord::deserialize(&session).unwrap();
                assert_eq!(record.previous_session_count(), 0);
                assert_eq!(
                    record
                        .session_state()
                        .unwrap()
                        .remote_identity_key()
                        .unwrap(),
                    Some(*f.new.identity_key())
                );
                assert_ne!(session, f.held);
                assert_ne!(f.store.journal_head().unwrap(), head);
                let history: Vec<serde_json::Value> = f
                    .store
                    .read(|tx| {
                        Ok(tx
                            .scan_after(Table::Checkpoints, 1, None, 512)?
                            .into_iter()
                            .filter(|(k, _)| k.starts_with(b"identity-history:"))
                            .map(|(_, v)| serde_json::from_slice(&v).unwrap())
                            .collect())
                    })
                    .unwrap();
                assert_eq!(history.len(), 1);
                assert_eq!(history[0]["previous_session"], serde_json::json!(f.held));
                assert_eq!(
                    history[0]["previous_identity"],
                    serde_json::json!(f.old.identity_key().serialize().to_vec())
                );
                let list = super::super::super::devices::device_list(&f.store, &f.peer)
                    .unwrap()
                    .unwrap();
                let mut expected = original_list;
                if timestamp == 10 {
                    expected.expected_timestamp = Some(10);
                }
                assert_eq!(list, expected);
                let pending = super::super::super::devices::pending_users(&f.store).unwrap();
                assert_eq!(
                    pending,
                    if timestamp == 10 {
                        vec![f.peer.to_non_ad()]
                    } else {
                        vec![]
                    }
                );
                let after = snapshot(&f.store);
                for (i, t) in Table::ALL.iter().enumerate() {
                    if !matches!(
                        t,
                        Table::PeerIdentities | Table::SignalSessions | Table::Checkpoints
                    ) {
                        assert_eq!(before[i], after[i], "{t:?}");
                    }
                }
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
    }
    #[tokio::test]
    async fn changed_companion_adv_refusals_and_signature_policy_preserve_all_rows() {
        for (case, expected) in [
            ("bad_account", "invalid_device_identity"),
            ("bad_device", "invalid_device_identity"),
            ("wrong_candidate", "invalid_device_identity"),
            ("foreign_primary", "invalid_device_identity"),
            ("raw_id", "raw_id_mismatch"),
            ("zero_index", "invalid_companion_index"),
            ("high_index", "invalid_companion_index"),
            ("deleted_equal", "deleted_device"),
            ("deleted_old", "deleted_device"),
            ("negative_newer", "deleted_device"),
            ("missing_timestamp", "missing_list_timestamp"),
            ("non_e2ee_attestation", "attestation_admission_unported"),
            ("pqkey", "pq_session_unported"),
            ("identity_auth", "malformed_identity_auth"),
            ("pinned", "identity_policy_pinned"),
            ("forged", "reply_not_verified"),
            ("untracked", "original_refresh_guard_not_matching"),
        ] {
            let f = fixture("222222222222222:4@lid").await;
            let primary = install_companion_authority(&f);
            let other = IdentityKeyPair::generate(&mut rand::rng());
            let mut bytes = signed_companion_adv(
                if case == "foreign_primary" {
                    &other
                } else {
                    &primary
                },
                if case == "wrong_candidate" {
                    &other
                } else {
                    &f.new
                },
                if case == "raw_id" { 9 } else { 8 },
                match case {
                    "deleted_old" => 8,
                    "negative_newer" => u64::MAX,
                    _ => 9,
                },
                match case {
                    "zero_index" => 0,
                    "high_index" => u32::MAX,
                    "deleted_equal" | "deleted_old" | "negative_newer" => 6,
                    _ => 7,
                },
            );
            if matches!(case, "bad_account" | "bad_device") {
                use prost::Message as _;
                let mut adv =
                    crate::proto::wa_adv::AdvSignedDeviceIdentity::decode(bytes.as_slice())
                        .unwrap();
                let signature = if case == "bad_account" {
                    &mut adv.account_signature
                } else {
                    &mut adv.device_signature
                };
                let mut tampered = signature.as_ref().unwrap().to_vec();
                tampered[0] ^= 1;
                *signature = Some(tampered.into());
                bytes = adv.encode_to_vec();
            }
            let mut response = reply(&f.peer, &f.id, &f.new, case != "forged");
            device_identity(&mut response, NodeContent::Bytes(bytes.into()));
            if matches!(case, "non_e2ee_attestation" | "pqkey" | "identity_auth") {
                metadata(&mut response, case);
            }
            if case != "non_e2ee_attestation" {
                attestation(&mut response, &f.new, "non_e2ee", false);
            }
            if case == "missing_timestamp" {
                change_list(&f, |l| l.list_timestamp = None);
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
                Application::Deferred(expected),
                "{case}"
            );
            assert_eq!(snapshot(&f.store), before, "{case}");
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }
    #[tokio::test]
    async fn changed_companion_authority_race_and_failed_sync_roll_back_every_effect() {
        for case in ["primary", "list", "ratchet", "bad_pending"] {
            let f = fixture("222222222222222:4@lid").await;
            let primary = install_companion_authority(&f);
            let mut response = f.response.clone();
            device_identity(
                &mut response,
                NodeContent::Bytes(signed_companion_adv(&primary, &f.new, 8, 10, 7).into()),
            );
            settle(&f, &response);
            if case == "bad_pending" {
                f.store
                    .write(|tx| {
                        tx.put_json(
                            Table::Checkpoints,
                            format!("device-sync-pending:{}", f.peer.to_non_ad()).as_bytes(),
                            1,
                            &serde_json::json!({"malformed":true}),
                        )
                    })
                    .unwrap();
            }
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
                "list" => change_list(&f, |l| l.devices[1].key_index = 6),
                "ratchet" => {
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
                _ => {}
            }
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            if case == "bad_pending" {
                assert!(commit(&f.store, &staged).is_err());
            } else {
                assert_eq!(
                    commit(&f.store, &staged).unwrap(),
                    Application::Deferred(if case == "ratchet" {
                        "peer_state_changed_before_commit"
                    } else {
                        "companion_authority_changed_before_commit"
                    })
                );
            }
            assert_eq!(snapshot(&f.store), before, "{case}");
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
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
            ("not_member", "invalid_device_identity"),
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
                    "identity_auth" => "malformed_identity_auth",
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
    async fn malformed_identity_auth_cannot_enter_classic_application_or_preview() {
        for auth in [
            Node::new("identity_auth"),
            Node::new("identity_auth").children(vec![Node::new("signature").bytes(vec![])]),
            Node::new("identity_auth").children(vec![Node::new("certs")]),
            Node::new("identity_auth").children(vec![
                Node::new("signature").children(vec![Node::new("SECRET")]),
                Node::new("certs"),
            ]),
        ] {
            let f = fixture("222222222222222@lid").await;
            let mut response = reply(&f.peer, &f.id, &f.old, true);
            metadata(&mut response, "force_override_e2ee_state");
            metadata(&mut response, "identity_auth");
            let NodeContent::Children(lists) = &mut response.content else {
                panic!()
            };
            let NodeContent::Children(users) = &mut lists[0].content else {
                panic!()
            };
            let NodeContent::Children(fields) = &mut users[0].content else {
                panic!()
            };
            *fields.last_mut().unwrap() = auth;
            settle(&f, &response);
            let before = snapshot(&f.store);
            let head = f.store.journal_head().unwrap();
            assert_eq!(
                super::super::preview_session(&f.store, &f.id)
                    .await
                    .unwrap()
                    .outcome,
                "malformed_identity_auth"
            );
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                Application::Deferred("malformed_identity_auth")
            );
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }

    #[tokio::test]
    async fn log_only_identity_auth_never_relaxes_other_admission_guards() {
        for (case, expected) in [
            ("forged", "reply_not_verified"),
            ("pinned", "identity_policy_pinned"),
            ("same", "same_identity_admission_unported"),
            ("non_e2ee_attestation", "attestation_admission_unported"),
            ("pqkey", "pq_bundle_invalid"),
            ("untracked", "original_refresh_guard_not_matching"),
        ] {
            let f = fixture("222222222222222@lid").await;
            let mut response = reply(
                &f.peer,
                &f.id,
                if case == "same" { &f.old } else { &f.new },
                case != "forged",
            );
            if matches!(case, "non_e2ee_attestation" | "pqkey") {
                metadata(&mut response, case);
            }
            metadata(&mut response, "identity_auth");
            let NodeContent::Children(lists) = &mut response.content else {
                panic!()
            };
            let NodeContent::Children(users) = &mut lists[0].content else {
                panic!()
            };
            let NodeContent::Children(fields) = &mut users[0].content else {
                panic!()
            };
            *fields.last_mut().unwrap() =
                Node::new("identity_auth")
                    .attr("version", "1")
                    .children(vec![
                        Node::new("signature").bytes(vec![]),
                        Node::new("certs"),
                    ]);
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
                Application::Deferred(expected)
            );
            assert_eq!(snapshot(&f.store), before);
            assert_eq!(f.store.journal_head().unwrap(), head);
        }
    }

    #[tokio::test]
    async fn log_only_identity_auth_does_not_block_guarded_classic_refresh() {
        for (peer, same_key, version) in [
            ("15555550111@s.whatsapp.net", false, "1"),
            ("222222222222222@lid", false, "bad"),
            ("222222222222222@lid", true, "2"),
            ("222222222222222:4@lid", false, "1"),
        ] {
            let f = fixture(peer).await;
            let mut response = if same_key {
                reply(&f.peer, &f.id, &f.old, true)
            } else {
                f.response.clone()
            };
            if same_key {
                metadata(&mut response, "force_override_e2ee_state");
            }
            if f.peer.device != 0 {
                let primary = install_companion_authority(&f);
                device_identity(
                    &mut response,
                    NodeContent::Bytes(signed_companion_adv(&primary, &f.new, 8, 9, 7).into()),
                );
            }
            metadata(&mut response, "identity_auth");
            let NodeContent::Children(lists) = &mut response.content else {
                panic!()
            };
            let NodeContent::Children(users) = &mut lists[0].content else {
                panic!()
            };
            let NodeContent::Children(fields) = &mut users[0].content else {
                panic!()
            };
            *fields.last_mut().unwrap() = Node::new("identity_auth")
                .attr("version", version)
                .children(vec![
                    Node::new("signature").bytes(vec![]),
                    Node::new("certs"),
                ]);
            settle(&f, &response);
            let before = snapshot(&f.store);
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                Application::Applied
            );
            let after = snapshot(&f.store);
            for (index, table) in Table::ALL.iter().enumerate() {
                if !matches!(
                    table,
                    Table::PeerIdentities | Table::SignalSessions | Table::Checkpoints
                ) {
                    assert_eq!(after[index], before[index], "{table:?}");
                }
            }
            let inspection = super::super::inspect(&f.store, &f.id).unwrap().unwrap();
            assert!(inspection.applied);
            assert_eq!(inspection.identity_auth.unwrap().verification, "unported");
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
            let record = SessionRecord::deserialize(&session).unwrap();
            assert_eq!(
                record.previous_session_count(),
                if same_key {
                    SessionRecord::deserialize(&f.held)
                        .unwrap()
                        .previous_session_count()
                        + 1
                } else {
                    0
                }
            );
            let head = f.store.journal_head().unwrap();
            assert_eq!(
                apply(&f.store, &f.id, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                Application::AlreadyApplied
            );
            assert_eq!(snapshot(&f.store), after);
            let path = f.dir.path().join("application.redb");
            let id = f.id.clone();
            drop(f.store);
            let store = open(&path);
            assert_eq!(
                recover(&store, IdentityChangePolicy::ApkAuthenticated)
                    .await
                    .unwrap(),
                0
            );
            assert!(super::super::inspect(&store, &id).unwrap().unwrap().applied);
            assert_eq!(snapshot(&store), after);
            assert_eq!(store.journal_head().unwrap(), head);
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
