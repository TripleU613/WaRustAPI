//! First phase of APK 37.71 identity notification processing: a durable key read.
//! Notification bytes are a refresh trigger, never replacement-key authority.
//! Native response adoption and deferred notification ACK are still unported.
use super::{actions, messaging};
use crate::{
    Error, Result,
    binary::Node,
    model::jid::{JidKind, Server},
    session::AccountRecord,
    store::{ProtocolStore, Table},
};
use sha2::{Digest, Sha256};
use wacore_libsignal::protocol::IdentityKey;

/// Redacted result of considering an authenticated notification for a key read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refresh {
    /// Not a first-child identity notification; another handler may consume it.
    Unrelated,
    /// The route is outside the supported PN/LID device subset.
    UnsupportedPeer,
    /// No existing peer identity, matching the native worker's fetch gate.
    MissingIdentity,
    /// Local primary/companion account identity must remain protected.
    OwnIdentity,
    /// A refused, invalid or uncertain earlier read blocks another query.
    BlockedPrior,
    /// Existing or newly queued once-only action; no pin/session has changed.
    Queued(String),
}

fn selector(prefix: &str, bytes: &[u8]) -> String {
    format!("{prefix}:{}", hex::encode(Sha256::digest(bytes)))
}

/// Offline diagnosis of a correlated read; contains no peer address or key.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Inspection {
    /// Generic action outcome versus successful signature verification.
    pub status: super::peer_keys::DirectoryStatus,
    /// Supported namespace, never a user identifier.
    pub peer_namespace: &'static str,
    /// Exact queried device, separate from the notification's participant.
    pub device: u16,
    /// Recognized numeric refusal code; raw server text is not exposed.
    pub server_error_code: Option<u16>,
    /// Null until a complete supported signed device bundle is available.
    pub signature_valid: Option<bool>,
    /// Null without a complete bundle/pin; comparison is not adoption or trust.
    pub pin_matches: Option<bool>,
}

/// Inspect an identity_refresh action, its original target and current pin in
/// one store snapshot. `actions --show ID` uses this without printing raw nodes.
pub fn inspect(store: &ProtocolStore, id: &str) -> Result<Option<Inspection>> {
    use super::peer_keys::DirectoryStatus;
    store.read(|tx| {
        let Some((action, reply)) = actions::view_in(tx, id)? else {
            return Ok(None);
        };
        if action.kind != "identity_refresh" {
            return Ok(None);
        }
        let request = actions::request_in(tx, id)?
            .ok_or_else(|| Error::invalid("identity_notification", "missing saved key request"))?;
        let users = request
            .child("key")
            .map(Node::child_nodes)
            .unwrap_or_default();
        let peer = (users.len() == 1)
            .then(|| super::receipts::jid(&users[0], "jid"))
            .flatten()
            .filter(|v| {
                !v.user.is_empty() && v.agent == 0 && matches!(v.server, Server::User | Server::Lid)
            })
            .ok_or_else(|| {
                Error::invalid("identity_notification", "invalid saved key request target")
            })?;
        let mut result = Inspection {
            status: match action.status {
                actions::ActionStatus::Queued => DirectoryStatus::Queued,
                actions::ActionStatus::Sent => DirectoryStatus::OutcomeUnknown,
                _ => DirectoryStatus::InvalidReply,
            },
            peer_namespace: if peer.server == Server::User {
                "pn"
            } else {
                "lid"
            },
            device: peer.device,
            server_error_code: None,
            signature_valid: None,
            pin_matches: None,
        };
        let Some(reply) = reply else {
            return Ok(Some(result));
        };
        if reply.description != "iq"
            || reply.get_attr("id").and_then(|v| v.as_text()) != Some(id)
            || !matches!(
                reply.get_attr("type").and_then(|v| v.as_text()),
                Some("result" | "error")
            )
        {
            return Ok(Some(result));
        }
        // Refusals belong to this root or the exact requested device. Never
        // report an unrelated user's error as this device's server response.
        let root_errors: Vec<_> = reply
            .child_nodes()
            .iter()
            .filter(|v| v.description == "error")
            .collect();
        let lists: Vec<_> = reply
            .child_nodes()
            .iter()
            .filter(|v| v.description == "list")
            .collect();
        let users: Vec<_> = lists
            .iter()
            .flat_map(|v| v.child_nodes())
            .filter(|v| {
                v.description == "user"
                    && super::receipts::jid(v, "jid")
                        .is_some_and(|v| super::identity_query::same_primary_address(&v, &peer))
            })
            .collect();
        let errors: Vec<_> = users
            .iter()
            .flat_map(|v| v.child_nodes())
            .filter(|v| v.description == "error")
            .collect();
        let refusal = if root_errors.len() == 1 {
            Some(root_errors[0])
        } else if root_errors.is_empty()
            && lists.len() == 1
            && users.len() == 1
            && errors.len() == 1
        {
            Some(errors[0])
        } else {
            None
        };
        if let Some(error) = refusal {
            result.status = DirectoryStatus::ServerRefused;
            result.server_error_code = error
                .get_attr("code")
                .and_then(|v| v.as_text())
                .and_then(|v| v.parse().ok());
            return Ok(Some(result));
        }
        let Ok(bundle) = crate::signal::direct::prekey_bundle_for(&reply, &peer) else {
            return Ok(Some(result));
        };
        let valid = crate::signal::direct::validate_bundle_signature(&bundle).is_ok();
        result.signature_valid = Some(valid);
        let pin: Option<Vec<u8>> = tx.get_json(
            Table::PeerIdentities,
            format!("comparison-direct:{peer}").as_bytes(),
            1,
        )?;
        result.pin_matches = pin
            .map(|pin| {
                bundle
                    .identity_key()
                    .map(|key| key.serialize().as_ref() == pin.as_slice())
            })
            .transpose()
            .map_err(|_| Error::invalid("identity_notification", "invalid bundle identity"))?;
        result.status = if valid {
            DirectoryStatus::VerifiedBundle
        } else {
            DirectoryStatus::InvalidSignature
        };
        Ok(Some(result))
    })
}

/// Queue the APK-derived `key/user reason=identity` read from an authenticated
/// receive path. Existing actions/CLI inspections expose its `identity_refresh`
/// kind. Calling this does not reserve a write, acknowledge or adopt any key.
pub fn queue(store: &ProtocolStore, account: &AccountRecord, node: &Node) -> Result<Refresh> {
    // 14z.A09 000e..0038, 00d0..00e0 selects child zero, not any identity sibling.
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("encrypt")
        || node
            .child_nodes()
            .first()
            .is_none_or(|v| v.description != "identity")
    {
        return Ok(Refresh::Unrelated);
    }
    // 0yO.BFC -> CmG.A02(null address) -> CmG.A00 0000..0014 uses root from;
    // participant goes into a separate descriptor field. 0DK.A00 preserves
    // DeviceJid or projects UserJid to primary. Other native types are unported.
    let Some(peer) = super::receipts::jid(node, "from").filter(|peer| {
        !peer.user.is_empty()
            && peer.agent == 0
            && matches!(peer.server, Server::User | Server::Lid)
            && matches!(peer.kind, JidKind::Plain | JidKind::AgentDevice)
    }) else {
        return Ok(Refresh::UnsupportedPeer);
    };
    let peer = peer.to_non_ad().with_device(peer.device);
    let id = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .ok_or_else(|| {
            Error::invalid(
                "identity_notification",
                "missing or excessive notification id",
            )
        })?;
    if !account.is_registered() {
        return Err(Error::NotRegistered);
    }
    if super::inbox::remote_identity_policy(
        store,
        account,
        &peer,
        crate::registration::request::IdentityChangePolicy::ApkAuthenticated,
    )? == crate::registration::request::IdentityChangePolicy::Pinned
    {
        return Ok(Refresh::OwnIdentity);
    }
    let pin_key = format!("comparison-direct:{peer}");
    let peer_marker = selector("identity-refresh-peer", peer.to_string().as_bytes());
    let event_marker = selector(
        "identity-refresh-event",
        &serde_json::to_vec(&(peer.clone(), id)).map_err(|_| {
            Error::invalid("identity_notification", "invalid notification selector")
        })?,
    );
    store.write(|tx| {
        if let Some(action) =
            tx.get_json::<String>(Table::Checkpoints, event_marker.as_bytes(), 1)?
        {
            return Ok(Refresh::Queued(action));
        }
        // DLp.run 0246..0252 skips fetch with no stored identity. Validate the
        // local pin instead of treating arbitrary saved bytes as that identity.
        let Some(pin) = tx.get_json::<Vec<u8>>(Table::PeerIdentities, pin_key.as_bytes(), 1)?
        else {
            return Ok(Refresh::MissingIdentity);
        };
        IdentityKey::decode(&pin)
            .map_err(|_| Error::invalid("identity_notification", "invalid saved peer identity"))?;
        if let Some(previous) =
            tx.get_json::<String>(Table::Checkpoints, peer_marker.as_bytes(), 1)?
        {
            // Local no-retry guard, not the full native 14L in-flight/job policy.
            // A genuinely new notification may follow a successful signed read;
            // a refusal/invalid/uncertain read is never reset by another event ID.
            let Some(reply) = actions::completed_reply_in(tx, &previous)? else {
                return Ok(Refresh::BlockedPrior);
            };
            let verified = crate::signal::direct::prekey_bundle_for(&reply, &peer)
                .and_then(|bundle| crate::signal::direct::validate_bundle_signature(&bundle));
            if verified.is_err() {
                return Ok(Refresh::BlockedPrior);
            }
        }
        let action = actions::queue_in(
            tx,
            "identity_refresh",
            "identity notification key read",
            |id| {
                messaging::key_request_many_with_identity_changes(
                    std::slice::from_ref(&peer),
                    std::slice::from_ref(&peer),
                    id,
                )
            },
        )?;
        tx.put_json(Table::Checkpoints, event_marker.as_bytes(), 1, &action)?;
        tx.put_json(Table::Checkpoints, peer_marker.as_bytes(), 1, &action)?;
        Ok(Refresh::Queued(action))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::jid::Jid;
    use crate::{
        model::ids::AccountId,
        session::RegistrationStatus,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::{IdentityKeyPair, KeyPair};

    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("fixture").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
        })
        .unwrap()
    }
    fn account() -> AccountRecord {
        AccountRecord {
            registration: RegistrationStatus::Registered,
            own_jid: Some(Jid::parse("15555550999@s.whatsapp.net").unwrap()),
            ..Default::default()
        }
    }
    fn notification(peer: &str, id: &str) -> Node {
        Node::new("notification")
            .attr("type", "encrypt")
            .attr("from", peer)
            .attr("id", id)
            .children(vec![Node::new("identity")])
    }
    fn install_pin(store: &ProtocolStore, peer: &Jid) -> IdentityKeyPair {
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &identity.identity_key().serialize().to_vec(),
                )?;
                tx.put_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &vec![1u8, 2, 3],
                )
            })
            .unwrap();
        identity
    }
    fn reply(peer: &Jid, id: &str, identity: &IdentityKeyPair, valid: bool) -> Node {
        let signed = KeyPair::generate(&mut rand::rng());
        let signature = identity
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
            .unwrap();
        Node::new("iq")
            .attr("type", "result")
            .attr("id", id)
            .children(vec![Node::new("list").children(vec![
                Node::new("user").attr("jid", peer.clone()).children(vec![
                    Node::new("registration").bytes(vec![0, 0, 0, 42]),
                    Node::new("identity").bytes(identity.public_key().public_key_bytes().to_vec()),
                    Node::new("skey").children(vec![
                        Node::new("id").bytes(vec![0, 0, 7]),
                        Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                        Node::new("signature").bytes(if valid {
                            signature.to_vec()
                        } else {
                            vec![0; 64]
                        }),
                    ]),
                ]),
            ])])
    }
    type Protected = (Vec<(Vec<u8>, Vec<u8>)>, Vec<(Vec<u8>, Vec<u8>)>);
    fn protected(store: &ProtocolStore) -> Protected {
        store
            .read(|tx| {
                Ok((
                    tx.scan_after(Table::PeerIdentities, 1, None, 128)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 128)?,
                ))
            })
            .unwrap()
    }

    #[test]
    fn source_routes_select_root_device_and_preserve_state_after_reopen() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/prekeys/2.26.37.71-notification-routes.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("routes.redb");
            let store = open(&path);
            let peer = Jid::parse(case["expected"].as_str().unwrap()).unwrap();
            install_pin(&store, &peer);
            let before = protected(&store);
            let head = store.journal_head().unwrap();
            let mut node = notification(case["from"].as_str().unwrap(), "fixture-route");
            for attr in ["participant", "recipient"] {
                if let Some(v) = case[attr].as_str() {
                    node = node.attr(attr, v);
                }
            }
            let Refresh::Queued(id) = queue(&store, &account(), &node).unwrap() else {
                panic!("expected key read")
            };
            assert_eq!(
                queue(&store, &account(), &node).unwrap(),
                Refresh::Queued(id.clone())
            );
            let request = actions::request(&store, &id).unwrap().unwrap();
            let user = &request.child("key").unwrap().child_nodes()[0];
            assert_eq!(super::super::receipts::jid(user, "jid"), Some(peer.clone()));
            assert_eq!(
                user.get_attr("reason").and_then(|v| v.as_text()),
                Some("identity")
            );
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().status,
                super::super::peer_keys::DirectoryStatus::Queued
            );
            assert_eq!(actions::reserve_pending(&store).unwrap().len(), 1);
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().status,
                super::super::peer_keys::DirectoryStatus::OutcomeUnknown
            );
            assert_eq!(
                queue(
                    &store,
                    &account(),
                    &notification(case["from"].as_str().unwrap(), "new-event")
                )
                .unwrap(),
                Refresh::BlockedPrior
            );
            assert_eq!(protected(&store), before);
            assert_eq!(store.journal_head().unwrap(), head);
            drop(store);
            let store = open(&path);
            assert_eq!(
                queue(&store, &account(), &node).unwrap(),
                Refresh::Queued(id)
            );
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
            assert_eq!(protected(&store), before);
            assert_eq!(store.journal_head().unwrap(), head);
        }
    }

    #[test]
    fn unsupported_missing_own_and_malformed_inputs_never_queue() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("guards.redb"));
        for peer in [
            "s.whatsapp.net",
            "15555550101@g.us",
            "15555550101@hosted",
            "15555550101.1:4@s.whatsapp.net",
        ] {
            assert_eq!(
                queue(&store, &account(), &notification(peer, "fixture")).unwrap(),
                Refresh::UnsupportedPeer
            );
        }
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        assert_eq!(
            queue(
                &store,
                &account(),
                &notification(&peer.to_string(), "fixture")
            )
            .unwrap(),
            Refresh::MissingIdentity
        );
        for first in ["count", "pq_count", "other"] {
            let node = notification(&peer.to_string(), "fixture")
                .children(vec![Node::new(first), Node::new("identity")]);
            assert_eq!(
                queue(&store, &account(), &node).unwrap(),
                Refresh::Unrelated
            );
        }
        let own = account().own_jid.unwrap().with_device(4);
        install_pin(&store, &own);
        assert_eq!(
            queue(&store, &account(), &notification(&own.to_string(), "own")).unwrap(),
            Refresh::OwnIdentity
        );
        let lid = Jid::parse("111111111111111:4@lid").unwrap();
        install_pin(&store, &lid);
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"lid":"111111111111111@lid"}),
                )
            })
            .unwrap();
        assert_eq!(
            queue(
                &store,
                &account(),
                &notification(&lid.to_string(), "own-lid")
            )
            .unwrap(),
            Refresh::OwnIdentity
        );
        for id in [String::new(), "x".repeat(129)] {
            let error =
                queue(&store, &account(), &notification(&peer.to_string(), &id)).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("missing or excessive notification id")
            );
        }
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &vec![0u8],
                )
            })
            .unwrap();
        let error = queue(
            &store,
            &account(),
            &notification(&peer.to_string(), "bad-pin"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("invalid saved peer identity"));
        assert!(actions::outcomes(&store).unwrap().is_empty());
    }

    #[test]
    fn only_new_events_after_valid_signed_results_can_queue_again() {
        for outcome in ["valid", "forged", "refused", "empty", "unknown_type"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("settle.redb"));
            let peer = Jid::parse("15555550101:4@s.whatsapp.net").unwrap();
            install_pin(&store, &peer);
            let identity = IdentityKeyPair::generate(&mut rand::rng());
            let before = protected(&store);
            let original = notification(&peer.to_string(), "first-event");
            let Refresh::Queued(id) = queue(&store, &account(), &original).unwrap() else {
                panic!("expected key read")
            };
            actions::reserve_pending(&store).unwrap();
            let response = match outcome {
                "valid" | "forged" => reply(&peer, &id, &identity, outcome == "valid"),
                "refused" => Node::new("iq")
                    .attr("type", "error")
                    .attr("id", id.as_str())
                    .children(vec![Node::new("error").attr("code", "429")]),
                "unknown_type" => Node::new("iq")
                    .attr("type", "get")
                    .attr("id", id.as_str())
                    .children(vec![Node::new("error").attr("code", "429")]),
                _ => Node::new("iq")
                    .attr("type", "result")
                    .attr("id", id.as_str()),
            };
            assert!(actions::record_reply(&store, &response).unwrap());
            use super::super::peer_keys::DirectoryStatus;
            let inspection = inspect(&store, &id).unwrap().unwrap();
            assert_eq!(inspection.device, 4);
            assert_eq!(
                inspection.status,
                match outcome {
                    "valid" => DirectoryStatus::VerifiedBundle,
                    "forged" => DirectoryStatus::InvalidSignature,
                    "refused" => DirectoryStatus::ServerRefused,
                    _ => DirectoryStatus::InvalidReply,
                }
            );
            assert_eq!(
                inspection.signature_valid,
                match outcome {
                    "valid" => Some(true),
                    "forged" => Some(false),
                    _ => None,
                }
            );
            assert_eq!(
                inspection.pin_matches,
                match outcome {
                    "valid" | "forged" => Some(false),
                    _ => None,
                }
            );
            assert_eq!(
                inspection.server_error_code,
                (outcome == "refused").then_some(429)
            );
            let rendered = serde_json::to_string(&inspection).unwrap();
            assert!(!rendered.contains(&peer.user));
            assert_eq!(protected(&store), before);

            assert_eq!(
                queue(&store, &account(), &original).unwrap(),
                Refresh::Queued(id.clone())
            );
            let next = queue(
                &store,
                &account(),
                &notification(&peer.to_string(), "second-event"),
            )
            .unwrap();
            if outcome == "valid" {
                let Refresh::Queued(new_id) = next else {
                    panic!("valid read permits a new event")
                };
                assert_ne!(new_id, id);
                assert_eq!(actions::reserve_pending(&store).unwrap().len(), 1);
            } else {
                assert_eq!(next, Refresh::BlockedPrior);
                assert!(actions::reserve_pending(&store).unwrap().is_empty());
            }
            assert_eq!(protected(&store), before);
        }
    }
}
