//! Read-only peer directory inspection over the existing once-only IQ queue.
//! Uses the APK-derived encrypt/key query in messaging::key_request, never
//! deriving a session, changing a pin or reserving/transmitting a chat.
//! Query baseline: APK 2.26.37.70 LX/0y0.A00 1786..193a; identity-change
//! reason: 2.26.37.71 same writer 1792..17da (bulk-prekey-refresh.md). Signed-key
//! check: APK 2.26.37.71 LX/CQc.A00 0030..0064 (builds kept separate).
use super::{actions, identity_conflicts, messaging};
use crate::{
    Error, Result,
    model::jid::{Jid, JidKind, Server},
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn validate(peer: &Jid) -> Result<()> {
    // Jid also represents server destinations. A key/user query needs an
    // actual peer, matching identity_query's guard; see peer-identity.md.
    if peer.user.is_empty() {
        return Err(Error::invalid(
            "peer_keys",
            "requires a nonempty peer user; server-only JID is not a peer",
        ));
    }
    if peer.server != Server::User
        || peer.device != 0
        || peer.agent != 0
        || !matches!(peer.kind, JidKind::Plain | JidKind::AgentDevice)
    {
        return Err(Error::invalid(
            "peer_keys",
            "requires a primary phone-number chat JID",
        ));
    }
    Ok(())
}
fn key(peer: &Jid) -> String {
    format!(
        "peer-directory-query:{}",
        hex::encode(Sha256::digest(
            format!("comparison-direct:{peer}").as_bytes()
        ))
    )
}

/// Redacted offline request preview; no store, action or account is opened.
#[derive(Debug, Serialize)]
pub struct IdentityChangePreview {
    /// Fixed IQ namespace.
    pub namespace: &'static str,
    /// Fixed IQ type.
    pub iq_type: &'static str,
    /// Fixed selected-user reason; no peer address is printed.
    pub user_reason: &'static str,
    /// Encoded comparison request size with a fixed offline selector.
    pub encoded_bytes: usize,
}

/// Inspect the missing native identity-change request shape without queueing.
/// APK 37.71 0y0.A00 17b6..17c8; bulk-prekey-refresh.md records authority gaps.
pub fn identity_change_preview(peer: &Jid) -> Result<IdentityChangePreview> {
    validate(peer)?;
    let node = messaging::key_request_many_with_identity_changes(
        std::slice::from_ref(peer),
        std::slice::from_ref(peer),
        "offline-identity-change",
    )?;
    let wire = crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
        .encode(&node)?;
    Ok(IdentityChangePreview {
        namespace: "encrypt",
        iq_type: "get",
        user_reason: "identity",
        encoded_bytes: wire.len(),
    })
}

// Local durable ownership, not an additional server wire requirement. Keep
// the original single-peer producer separate from batched/identity-only reads.
fn owns_request(request: &crate::binary::Node, id: &str, peer: &Jid) -> bool {
    if request.description != "iq"
        || request.get_attr("id").and_then(|value| value.as_text()) != Some(id)
        || request.get_attr("type").and_then(|value| value.as_text()) != Some("get")
        || request.get_attr("xmlns").and_then(|value| value.as_text()) != Some("encrypt")
        || !super::receipts::jid(request, "to").is_some_and(|jid| {
            jid.user.is_empty() && jid.server == Server::User && jid.agent == 0 && jid.device == 0
        })
    {
        return false;
    }
    let [key] = request.child_nodes() else {
        return false;
    };
    if key.description != "key" {
        return false;
    }
    let [user] = key.child_nodes() else {
        return false;
    };
    user.description == "user"
        && super::receipts::jid(user, "jid")
            .is_some_and(|jid| super::identity_query::same_primary_address(&jid, peer))
}

/// Queue one peer directory query. Repeats return its existing ID locally.
/// An explicit refresh requires a previously successful, valid signed bundle;
/// refused, invalid or uncertain queries cannot be reset or automatically retried.
pub fn queue(store: &ProtocolStore, peer: &Jid, refresh: bool) -> Result<String> {
    validate(peer)?;
    store.write(|tx| {
        let existing: Option<String> = tx.get_json(Table::Checkpoints, key(peer).as_bytes(), 1)?;
        if refresh && existing.is_none() {
            return Err(Error::invalid(
                "peer_keys",
                "no prior query to refresh; queue once without --refresh",
            ));
        }
        if let Some(id) = existing {
            if !refresh {
                return Ok(id);
            }
            let (action, reply, request) = actions::view_in_write(tx, &id)?.ok_or_else(|| {
                Error::invalid(
                    "peer_keys",
                    "directory marker has no saved action; no refresh",
                )
            })?;
            if action.kind != "peer_keys" || !owns_request(&request, &id, peer) {
                return Err(Error::invalid(
                    "peer_keys",
                    "prior action does not own this directory request; no refresh",
                ));
            }
            if action.status != actions::ActionStatus::Ok {
                return Err(Error::invalid(
                    "peer_keys",
                    "prior directory query is refused, incomplete or uncertain; no refresh",
                ));
            }
            let reply = reply.ok_or_else(|| {
                Error::invalid("peer_keys", "prior directory reply is missing; no refresh")
            })?;
            if reply.description != "iq"
                || reply.get_attr("id").and_then(|value| value.as_text()) != Some(id.as_str())
            {
                return Err(Error::invalid(
                    "peer_keys",
                    "prior directory reply does not match its request; no refresh",
                ));
            }
            let bundle = crate::signal::direct::prekey_bundle(&reply, peer).map_err(|_| {
                Error::invalid("peer_keys", "prior reply has no usable bundle; no refresh")
            })?;
            crate::signal::direct::validate_bundle_signature(&bundle).map_err(|_| {
                Error::invalid("peer_keys", "prior bundle signature is invalid; no refresh")
            })?;
        }
        let id = actions::queue_in(tx, "peer_keys", "peer directory inspection", |id| {
            Ok(messaging::key_request(peer, id))
        })?;
        tx.put_json(Table::Checkpoints, key(peer).as_bytes(), 1, &id)?;
        Ok(id)
    })
}

/// Directory result, distinct from handset or human identity verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectoryStatus {
    /// Queued locally; no socket write reserved yet.
    Queued,
    /// Reserved before socket write; no matching reply, never automatically retried.
    OutcomeUnknown,
    /// The response reports an outer or per-user server refusal.
    ServerRefused,
    /// Missing or malformed bundle; no key or pin is adopted.
    InvalidReply,
    /// Parsed bundle whose signed device key fails its identity signature check.
    InvalidSignature,
    /// Authenticated saved directory response has a valid signed device key.
    VerifiedBundle,
}

/// Fixed local diagnosis of a saved directory reply; not a server reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectoryReplyFailure {
    /// Wrong IQ envelope, correlation or unsupported response type.
    Envelope,
    /// Missing/duplicate refusal list or absent/duplicate matching primary user.
    UserSet,
    /// Missing or duplicate error child in the selected refusal context.
    ErrorNode,
    /// The selected successful result has no supported complete key bundle.
    Bundle,
}

/// Comparison with a locally authenticated primary LID mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedPinMatch {
    /// Hash of the saved session address, without a peer JID.
    pub peer_ticket: String,
    /// Null when no pin is held for that alias.
    pub pin_matches: Option<bool>,
}

/// Redacted inspection of the latest directory action and current saved pins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryInspection {
    /// Original one-shot action selector; never a phone number.
    pub action_id: String,
    /// Recognized local/server result.
    pub status: DirectoryStatus,
    /// Fixed local invalid-reply diagnosis; null for accepted/refused/unstaged evidence.
    #[serde(default)]
    pub failure: Option<DirectoryReplyFailure>,
    /// Signed Java integer from a unique root/selected-peer refusal. No raw text.
    pub server_error_code: Option<i32>,
    /// SHA-256 of serialized directory public identity, not a safety number.
    pub fingerprint: Option<String>,
    /// Null when no complete bundle can be checked.
    pub signature_valid: Option<bool>,
    /// Null when the directory identity or saved PN pin is absent.
    pub phone_pin_matches: Option<bool>,
    /// Every locally mapped primary LID, without choosing one silently.
    pub linked_pin_matches: Vec<LinkedPinMatch>,
}

// Local unique-context guard shared by inspection and typed action attribution.
// APK43.72 Dxh.A02 groups errors by originally requested devices (037a..0416).
pub(super) fn selected_user<'a>(
    reply: &'a crate::binary::Node,
    peer: &Jid,
) -> std::result::Result<&'a crate::binary::Node, DirectoryReplyFailure> {
    let lists: Vec<_> = reply
        .child_nodes()
        .iter()
        .filter(|node| node.description == "list")
        .collect();
    if lists.len() != 1 {
        return Err(DirectoryReplyFailure::UserSet);
    }
    let users: Vec<_> = lists[0]
        .child_nodes()
        .iter()
        .filter(|user| user.description == "user")
        .filter(|user| {
            user.get_attr("jid").is_some_and(|value| {
                value
                    .as_jid()
                    .is_some_and(|jid| super::identity_query::same_primary_address(jid, peer))
                    || value
                        .as_text()
                        .and_then(|text| Jid::parse(text).ok())
                        .is_some_and(|jid| super::identity_query::same_primary_address(&jid, peer))
            })
        })
        .collect();
    if users.len() != 1 {
        return Err(DirectoryReplyFailure::UserSet);
    }
    Ok(users[0])
}

/// Inspect saved directory evidence and pins in one read transaction. No key,
/// session, journal, action, receipt or send record is changed.
pub fn inspect(store: &ProtocolStore, peer: &Jid) -> Result<Option<DirectoryInspection>> {
    validate(peer)?;
    store.read(|tx| {
        let Some(id): Option<String> = tx.get_json(Table::Checkpoints, key(peer).as_bytes(), 1)?
        else {
            return Ok(None);
        };
        let (action, reply) = actions::view_in(tx, &id)?
            .ok_or_else(|| Error::invalid("peer_keys", "directory marker has no saved action"))?;
        if action.kind != "peer_keys" {
            return Err(Error::invalid(
                "peer_keys",
                "directory marker references another action kind",
            ));
        }
        let request = actions::request_in(tx, &id)?
            .ok_or_else(|| Error::invalid("peer_keys", "directory request is missing"))?;
        if !owns_request(&request, &id, peer) {
            return Err(Error::invalid(
                "peer_keys",
                "directory request does not match selected peer",
            ));
        }
        let mut result = DirectoryInspection {
            action_id: id,
            status: match action.status {
                actions::ActionStatus::Queued => DirectoryStatus::Queued,
                actions::ActionStatus::Sent => DirectoryStatus::OutcomeUnknown,
                actions::ActionStatus::Ok => DirectoryStatus::InvalidReply,
                actions::ActionStatus::Error => DirectoryStatus::InvalidReply,
            },
            failure: None,
            server_error_code: None,
            fingerprint: None,
            signature_valid: None,
            phone_pin_matches: None,
            linked_pin_matches: Vec::new(),
        };
        let Some(reply) = reply else {
            return Ok(Some(result));
        };
        // Correlation and duplicate rejection are local inspection guards.
        // APK43.72 Dxh.A02 reads signed code in each user context;
        // a generic action's first inner error need not belong to this peer.
        if reply.description != "iq"
            || reply.get_attr("id").and_then(|v| v.as_text()) != Some(result.action_id.as_str())
            || !matches!(
                reply.get_attr("type").and_then(|v| v.as_text()),
                Some("result" | "error")
            )
        {
            result.failure = Some(DirectoryReplyFailure::Envelope);
            return Ok(Some(result));
        }
        // Parse original evidence independently of the generic action status:
        // older records classified root/foreign errors as whole-query failure.
        {
            let root_errors: Vec<_> = reply
                .child_nodes()
                .iter()
                .filter(|node| node.description == "error")
                .collect();
            let code = |node: &crate::binary::Node| {
                node.get_attr("code")
                    .and_then(|value| value.as_text())
                    .and_then(super::mex::java_decimal_long)
                    .and_then(|code| i32::try_from(code).ok())
            };
            if reply.get_attr("type").and_then(|v| v.as_text()) == Some("error") {
                if root_errors.len() > 1 {
                    result.failure = Some(DirectoryReplyFailure::ErrorNode);
                    return Ok(Some(result));
                }
                result.status = DirectoryStatus::ServerRefused;
                result.server_error_code = root_errors.first().and_then(|error| code(error));
                return Ok(Some(result));
            }
            let user = match selected_user(&reply, peer) {
                Ok(user) => user,
                Err(reason) => {
                    result.failure = Some(reason);
                    return Ok(Some(result));
                }
            };
            let errors: Vec<_> = user
                .child_nodes()
                .iter()
                .filter(|node| node.description == "error")
                .collect();
            if errors.len() > 1 {
                result.failure = Some(DirectoryReplyFailure::ErrorNode);
                return Ok(Some(result));
            }
            if let Some(error) = errors.first() {
                result.status = DirectoryStatus::ServerRefused;
                result.server_error_code = code(error);
                return Ok(Some(result));
            }
        }
        let Ok(bundle) = crate::signal::direct::prekey_bundle(&reply, peer) else {
            result.failure = Some(DirectoryReplyFailure::Bundle);
            return Ok(Some(result));
        };
        let identity = bundle
            .identity_key()
            .map_err(|_| Error::invalid("peer_keys", "directory bundle identity unavailable"))?;
        let fingerprint = hex::encode(Sha256::digest(identity.serialize()));
        let valid = crate::signal::direct::validate_bundle_signature(&bundle).is_ok();
        let saved = identity_conflicts::inspect_peer_in(tx, peer)?;
        result.phone_pin_matches = saved
            .phone
            .pinned_fingerprint
            .as_ref()
            .map(|pin| pin == &fingerprint);
        result.linked_pin_matches = saved
            .linked
            .into_iter()
            .map(|linked| LinkedPinMatch {
                peer_ticket: linked.state.peer_ticket,
                pin_matches: linked
                    .state
                    .pinned_fingerprint
                    .map(|pin| pin == fingerprint),
            })
            .collect();
        result.fingerprint = Some(fingerprint);
        result.signature_valid = Some(valid);
        result.status = if valid {
            DirectoryStatus::VerifiedBundle
        } else {
            DirectoryStatus::InvalidSignature
        };
        Ok(Some(result))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binary::Node,
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::{IdentityKeyPair, KeyPair};
    #[test]
    fn identity_change_preview_is_redacted_and_has_no_store_dependency() {
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let preview = identity_change_preview(&peer).unwrap();
        assert_eq!(preview.user_reason, "identity");
        assert!(preview.encoded_bytes > 0);
        let json = serde_json::to_string(&preview).unwrap();
        assert!(!json.contains(&peer.user));
        assert!(!json.contains("offline-identity-change"));
        assert!(identity_change_preview(&Jid::parse("s.whatsapp.net").unwrap()).is_err());
        assert!(identity_change_preview(&peer.with_device(3)).is_err());
    }
    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn signed_reply(peer: &Jid, id: &str, identity: &IdentityKeyPair, valid: bool) -> Node {
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
                    Node::new("type").bytes(vec![5]),
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
    #[test]
    fn empty_peer_directory_inputs_refuse_without_changing_saved_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty-peer.redb");
        let store = open(&path);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let id = queue(&store, &peer, false).unwrap();
        assert_eq!(actions::reserve_pending(&store).unwrap().len(), 1);
        let pin = IdentityKeyPair::generate(&mut rand::rng())
            .identity_key()
            .serialize()
            .to_vec();
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &pin,
                )?;
                tx.put_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &vec![1u8, 2, 3],
                )
            })
            .unwrap();
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    Ok((
                        tx.scan_after(Table::Checkpoints, 1, None, 256)?,
                        tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                        tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                        tx.scan_after(Table::Outbox, 1, None, 256)?,
                    ))
                })
                .unwrap()
        };
        let before = snapshot(&store);
        let head = store.journal_head().unwrap();
        for kind in [JidKind::Plain, JidKind::AgentDevice] {
            let mut empty = Jid::parse("s.whatsapp.net").unwrap();
            empty.kind = kind;
            for refresh in [false, true] {
                let error = queue(&store, &empty, refresh).unwrap_err().to_string();
                assert!(error.contains("nonempty peer user"));
            }
            assert!(
                inspect(&store, &empty)
                    .unwrap_err()
                    .to_string()
                    .contains("nonempty peer user")
            );
            assert!(
                identity_conflicts::inspect_peer(&store, &empty)
                    .unwrap_err()
                    .to_string()
                    .contains("nonempty peer user")
            );
        }
        assert_eq!(snapshot(&store), before);
        assert_eq!(store.journal_head().unwrap(), head);
        assert_eq!(queue(&store, &peer, false).unwrap(), id);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        drop(store);
        let store = open(&path);
        assert_eq!(snapshot(&store), before);
        assert_eq!(store.journal_head().unwrap(), head);
        assert_eq!(queue(&store, &peer, false).unwrap(), id);
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().status,
            DirectoryStatus::OutcomeUnknown
        );
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
    }
    #[test]
    fn directory_queries_are_atomic_once_only_redacted_and_refresh_only_verified_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("directory.redb");
        let store = std::sync::Arc::new(open(&path));
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let lid = Jid::parse("222222222222222@lid").unwrap();
        let old = IdentityKeyPair::generate(&mut rand::rng());
        let new = IdentityKeyPair::generate(&mut rand::rng());
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &old.identity_key().serialize().to_vec(),
                )?;
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{lid}").as_bytes(),
                    1,
                    &new.identity_key().serialize().to_vec(),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("authenticated-lid:{lid}").as_bytes(),
                    1,
                    &peer,
                )
            })
            .unwrap();
        assert!(inspect(&store, &peer).unwrap().is_none());
        assert!(queue(&store, &peer, true).is_err());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(4));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let store = store.clone();
                let peer = peer.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    queue(&store, &peer, false).unwrap()
                })
            })
            .collect();
        let ids: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert!(ids.iter().all(|id| id == &ids[0]));
        let id = ids[0].clone();
        assert_eq!(actions::outcomes(&store).unwrap().len(), 1);
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().status,
            DirectoryStatus::Queued
        );
        assert!(queue(&store, &peer, true).is_err());
        let pending = actions::reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0], messaging::key_request(&peer, &id));
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().status,
            DirectoryStatus::OutcomeUnknown
        );
        assert!(queue(&store, &peer, true).is_err());
        drop(store);
        let store = open(&path);
        assert_eq!(queue(&store, &peer, false).unwrap(), id);
        assert!(
            actions::reserve_pending(&store).unwrap().is_empty(),
            "write cut never repeats query"
        );
        let reply = signed_reply(&peer, &id, &new, true);
        assert!(actions::record_reply(&store, &reply).unwrap());
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                    tx.scan_after(Table::Outbox, 1, None, 256)?,
                    tx.scan_after(Table::Checkpoints, 1, None, 256)?,
                ))
            })
            .unwrap();
        let result = inspect(&store, &peer).unwrap().unwrap();
        assert_eq!(result.status, DirectoryStatus::VerifiedBundle);
        assert_eq!(result.signature_valid, Some(true));
        assert_eq!(result.phone_pin_matches, Some(false));
        assert_eq!(result.linked_pin_matches.len(), 1);
        assert_eq!(result.linked_pin_matches[0].pin_matches, Some(true));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains(&peer.user));
        assert!(!json.contains(&lid.user));
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                    tx.scan_after(Table::Outbox, 1, None, 256)?,
                    tx.scan_after(Table::Checkpoints, 1, None, 256)?
                )))
                .unwrap(),
            before
        );
        let refreshed = queue(&store, &peer, true).unwrap();
        assert_ne!(refreshed, id);
        assert_eq!(queue(&store, &peer, false).unwrap(), refreshed);
        assert_eq!(actions::outcomes(&store).unwrap().len(), 2);
        assert_eq!(actions::reply(&store, &id).unwrap(), Some(reply));
        assert_eq!(actions::reserve_pending(&store).unwrap().len(), 1);
        assert!(
            actions::record_reply(&store, &signed_reply(&peer, &refreshed, &new, false)).unwrap()
        );
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().status,
            DirectoryStatus::InvalidSignature
        );
        assert!(queue(&store, &peer, true).is_err());
        assert_eq!(queue(&store, &peer, false).unwrap(), refreshed);
    }

    #[test]
    fn refresh_requires_original_action_request_and_reply_ownership() {
        for case in [
            "kind",
            "request_id",
            "request_peer",
            "request_type",
            "namespace",
            "destination",
            "request_width",
            "reply_id",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("ownership.redb");
            let store = open(&path);
            let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
            let foreign = Jid::parse("15555550102@s.whatsapp.net").unwrap();
            let identity = IdentityKeyPair::generate(&mut rand::rng());
            let id = queue(&store, &peer, false).unwrap();
            actions::reserve_pending(&store).unwrap();
            let reply = signed_reply(&peer, &id, &identity, true);
            actions::record_reply(&store, &reply).unwrap();
            let mut request = actions::request(&store, &id).unwrap().unwrap();
            let mut reply = reply;
            match case {
                "request_id" => request = request.attr("id", "00000000000000000000000000000000"),
                "request_peer" => request = messaging::key_request(&foreign, &id),
                "request_type" => request = request.attr("type", "set"),
                "namespace" => request = request.attr("xmlns", "foreign"),
                "destination" => request = request.attr("to", foreign.clone()),
                "request_width" => {
                    request = messaging::key_request_many(&[peer.clone(), foreign], &id).unwrap()
                }
                "reply_id" => reply = reply.attr("id", "00000000000000000000000000000000"),
                _ => {}
            }
            let encode = |node: &Node| {
                crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                    .encode(node)
                    .unwrap()
            };
            store
                .write(|tx| {
                    let key = format!("action-outbox:{id}");
                    let mut row: serde_json::Value =
                        tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                    row["wire"] = serde_json::to_value(encode(&request)).unwrap();
                    row["reply"] = serde_json::to_value(encode(&reply)).unwrap();
                    if case == "kind" {
                        row["kind"] = serde_json::json!("foreign");
                    }
                    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)
                })
                .unwrap();
            drop(store);
            let store = open(&path);
            let snapshot = || {
                store
                    .read(|tx| {
                        Ok((
                            tx.scan_after(Table::Checkpoints, 1, None, 256)?,
                            tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                            tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                            tx.scan_after(Table::Outbox, 1, None, 256)?,
                        ))
                    })
                    .unwrap()
            };
            let before = snapshot();
            let head = store.journal_head().unwrap();
            assert!(queue(&store, &peer, true).is_err(), "{case}");
            assert!(
                !inspect(&store, &peer).is_ok_and(
                    |view| view.is_some_and(|v| v.status == DirectoryStatus::VerifiedBundle)
                ),
                "{case}"
            );
            assert_eq!(before, snapshot(), "{case}");
            assert_eq!(head, store.journal_head().unwrap());
            assert_eq!(queue(&store, &peer, false).unwrap(), id);
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
        }
    }

    #[test]
    fn directory_action_errors_belong_to_original_selected_peer() {
        for first in [false, true] {
            for refused in [false, true] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("selected.redb");
                let store = open(&path);
                let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
                let foreign = Jid::parse("15555550102@s.whatsapp.net").unwrap();
                let identity = IdentityKeyPair::generate(&mut rand::rng());
                let id = queue(&store, &peer, false).unwrap();
                actions::reserve_pending(&store).unwrap();
                let reply = signed_reply(&peer, &id, &identity, true);
                let selected = if refused {
                    Node::new("user")
                        .attr("jid", peer.clone())
                        .children(vec![Node::new("error").attr("code", "480")])
                } else {
                    reply.child("list").unwrap().child("user").unwrap().clone()
                };
                let other = Node::new("user").attr("jid", foreign).children(vec![
                    Node::new("error")
                        .attr("code", "503")
                        .attr("text", "FOREIGN_PRIVATE"),
                ]);
                let users = if first {
                    vec![other, selected]
                } else {
                    vec![selected, other]
                };
                let incoming = reply.children(vec![Node::new("list").children(users)]);
                actions::record_reply(&store, &incoming).unwrap();
                drop(store);
                let store = open(&path);
                let outcome = actions::outcomes(&store).unwrap().remove(0);
                assert_eq!(
                    outcome.status,
                    if refused {
                        actions::ActionStatus::Error
                    } else {
                        actions::ActionStatus::Ok
                    }
                );
                assert_eq!(outcome.error_code, if refused { Some(480) } else { None });
                assert!(
                    !serde_json::to_string(&outcome)
                        .unwrap()
                        .contains("FOREIGN_PRIVATE")
                );
                assert_eq!(
                    inspect(&store, &peer).unwrap().unwrap().status,
                    if refused {
                        DirectoryStatus::ServerRefused
                    } else {
                        DirectoryStatus::VerifiedBundle
                    }
                );
                assert!(actions::reserve_pending(&store).unwrap().is_empty());
                assert_eq!(actions::reply(&store, &id).unwrap(), Some(incoming));
                assert_eq!(queue(&store, &peer, true).is_ok(), !refused);
            }
        }
    }

    #[test]
    fn duplicate_result_lists_cannot_authorize_directory_refresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("duplicate-list.redb");
        let store = open(&path);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        let id = queue(&store, &peer, false).unwrap();
        actions::reserve_pending(&store).unwrap();
        let reply = signed_reply(&peer, &id, &identity, true);
        let list = reply.child("list").unwrap().clone();
        let incoming = reply.children(vec![list.clone(), list]);
        actions::record_reply(&store, &incoming).unwrap();
        drop(store);
        let store = open(&path);
        let before = serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap();
        let head = store.journal_head().unwrap();
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().failure,
            Some(DirectoryReplyFailure::UserSet)
        );
        assert!(queue(&store, &peer, true).is_err());
        assert_eq!(
            serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap(),
            before
        );
        assert_eq!(store.journal_head().unwrap(), head);
        assert_eq!(queue(&store, &peer, false).unwrap(), id);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
    }

    #[test]
    fn mixed_root_errors_preserve_selected_bundle_and_read_only_inspection() {
        for historical in [false, true] {
            for first in [false, true] {
                for valid in [false, true] {
                    let dir = tempfile::tempdir().unwrap();
                    let path = dir.path().join("mixed-root.redb");
                    let store = open(&path);
                    let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
                    let identity = IdentityKeyPair::generate(&mut rand::rng());
                    let id = queue(&store, &peer, false).unwrap();
                    actions::reserve_pending(&store).unwrap();
                    let reply = signed_reply(&peer, &id, &identity, valid);
                    let list = reply.child("list").unwrap().clone();
                    let error = Node::new("error").attr("code", "503");
                    let reply = reply.children(if first {
                        vec![error, list]
                    } else {
                        vec![list, error]
                    });
                    assert!(actions::record_reply(&store, &reply).unwrap());
                    assert_eq!(
                        actions::outcomes(&store).unwrap()[0].status,
                        actions::ActionStatus::Ok
                    );
                    if historical {
                        // Reproduce old persisted classification without changing saved reply bytes.
                        store
                            .write(|tx| {
                                let key = format!("action-outbox:{id}");
                                let mut row: serde_json::Value =
                                    tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                                row["status"] = serde_json::json!("error");
                                row["error_code"] = serde_json::json!(503);
                                tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &row)
                            })
                            .unwrap();
                    }
                    drop(store);
                    let store = open(&path);
                    let snapshot = || {
                        store
                            .read(|tx| {
                                Ok((
                                    tx.scan_after(Table::Checkpoints, 1, None, 256)?,
                                    tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                                    tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                                ))
                            })
                            .unwrap()
                    };
                    let before = snapshot();
                    let head = store.journal_head().unwrap();
                    let view = inspect(&store, &peer).unwrap().unwrap();
                    assert_eq!(
                        view.status,
                        if valid {
                            DirectoryStatus::VerifiedBundle
                        } else {
                            DirectoryStatus::InvalidSignature
                        }
                    );
                    assert_eq!(view.signature_valid, Some(valid));
                    assert_eq!(view.server_error_code, None);
                    assert_eq!(view.failure, None);
                    assert_eq!(before, snapshot());
                    assert_eq!(head, store.journal_head().unwrap());
                    assert!(!serde_json::to_string(&view).unwrap().contains(&peer.user));
                    assert!(actions::reserve_pending(&store).unwrap().is_empty());
                    assert_eq!(actions::reply(&store, &id).unwrap(), Some(reply));
                    assert_eq!(queue(&store, &peer, true).is_ok(), valid && !historical);
                }
            }
        }
    }

    #[test]
    fn ambiguous_directory_bundle_stays_invalid_across_reopen_without_refresh_or_pin_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ambiguous.redb");
        let store = open(&path);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let old = IdentityKeyPair::generate(&mut rand::rng());
        let new = IdentityKeyPair::generate(&mut rand::rng());
        let pin_key = format!("comparison-direct:{peer}");
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    pin_key.as_bytes(),
                    1,
                    &old.identity_key().serialize().to_vec(),
                )
            })
            .unwrap();
        let id = queue(&store, &peer, false).unwrap();
        assert_eq!(actions::reserve_pending(&store).unwrap().len(), 1);
        let mut reply = signed_reply(&peer, &id, &new, true);
        let user = reply.child("list").unwrap().child("user").unwrap().clone();
        reply.content = crate::binary::NodeContent::Children(vec![
            Node::new("list").children(vec![user.clone(), user]),
        ]);
        assert!(actions::record_reply(&store, &reply).unwrap());
        drop(store);
        let store = open(&path);
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                    tx.scan_after(Table::Checkpoints, 1, None, 256)?,
                ))
            })
            .unwrap();
        let result = inspect(&store, &peer).unwrap().unwrap();
        assert_eq!(result.status, DirectoryStatus::InvalidReply);
        assert_eq!(result.signature_valid, None);
        assert_eq!(result.fingerprint, None);
        assert!(queue(&store, &peer, true).is_err());
        assert_eq!(queue(&store, &peer, false).unwrap(), id);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan_after(Table::PeerIdentities, 1, None, 256)?,
                    tx.scan_after(Table::SignalSessions, 1, None, 256)?,
                    tx.scan_after(Table::Checkpoints, 1, None, 256)?
                )))
                .unwrap(),
            before
        );
        assert!(!serde_json::to_string(&result).unwrap().contains(&peer.user));
    }

    #[test]
    fn existing_action_builders_run_outside_the_store_write_lock() {
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(open(&dir.path().join("builder.redb")));
        let worker_store = store.clone();
        let (sent, received) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let id = actions::queue(&worker_store, "fixture", "builder reentrancy", |id| {
                worker_store
                    .write(|tx| tx.put_json(Table::Checkpoints, b"builder-marker", 1, &true))?;
                Ok(Node::new("iq").attr("id", id).attr("type", "get"))
            });
            sent.send(id).unwrap();
        });
        let id = received
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("legacy action builder must not hold the store write lock")
            .unwrap();
        worker.join().unwrap();
        assert_eq!(actions::outcomes(&store).unwrap()[0].id, id);
        assert_eq!(
            store
                .read(|tx| tx.get_json::<bool>(Table::Checkpoints, b"builder-marker", 1))
                .unwrap(),
            Some(true)
        );
    }

    #[test]
    fn directory_refusals_and_unknown_replies_never_authorize_a_refresh() {
        for kind in 0..3 {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("refusal.redb"));
            let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
            let id = queue(&store, &peer, false).unwrap();
            actions::reserve_pending(&store).unwrap();
            let reply = match kind {
                0 => Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .children(vec![Node::new("list").children(vec![
                        Node::new("user").attr("jid", peer.clone()).children(vec![
                                Node::new("error")
                                    .attr("code", "480")
                                    .attr("text", "private-peer-marker"),
                            ]),
                    ])]),
                1 => Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "error")
                    .children(vec![Node::new("error").attr("code", "503")]),
                _ => Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "unknown"),
            };
            actions::record_reply(&store, &reply).unwrap();
            let result = inspect(&store, &peer).unwrap().unwrap();
            assert_eq!(
                result.status,
                if kind < 2 {
                    DirectoryStatus::ServerRefused
                } else {
                    DirectoryStatus::OutcomeUnknown
                }
            );
            assert_eq!(
                result.server_error_code,
                match kind {
                    0 => Some(480),
                    1 => Some(503),
                    _ => None,
                }
            );
            assert_eq!(result.signature_valid, None);
            assert_eq!(result.fingerprint, None);
            assert!(
                !serde_json::to_string(&result)
                    .unwrap()
                    .contains("private-peer-marker")
            );
            assert!(queue(&store, &peer, true).is_err());
            assert_eq!(queue(&store, &peer, false).unwrap(), id);
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
        }
    }
    #[test]
    fn per_user_refusal_correlation_ignores_encoding_but_preserves_actual_address() {
        let expected = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let mut encoded = expected.clone();
        encoded.kind = JidKind::AgentDevice;
        let mut lid = encoded.clone();
        lid.server = Server::Lid;
        let mut agent = encoded.clone();
        agent.agent = 5;
        let mut foreign = encoded.clone();
        foreign.user = "15555550102".into();
        for (returned, wanted) in [
            (expected.clone(), DirectoryStatus::ServerRefused),
            (encoded.clone(), DirectoryStatus::ServerRefused),
            (encoded.with_device(3), DirectoryStatus::InvalidReply),
            (lid, DirectoryStatus::InvalidReply),
            (agent, DirectoryStatus::InvalidReply),
            (foreign, DirectoryStatus::InvalidReply),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("refusal-encoding.redb"));
            let id = queue(&store, &expected, false).unwrap();
            actions::reserve_pending(&store).unwrap();
            let returned = if returned.agent == 0 {
                crate::binary::node::AttributeValue::Jid(returned)
            } else {
                crate::binary::node::AttributeValue::Text(returned.to_string())
            };
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("list").children(vec![Node::new("user")
                    .attr("jid", returned).children(vec![Node::new("error")
                        .attr("code", "503").attr("text", "SECRET_BODY")])])]);
            actions::record_reply(&store, &reply).unwrap();
            let before = serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap();
            let view = inspect(&store, &expected).unwrap().unwrap();
            assert_eq!(view.status, wanted);
            assert!(view.fingerprint.is_none());
            assert!(view.signature_valid.is_none());
            assert!(
                !serde_json::to_string(&view)
                    .unwrap()
                    .contains("SECRET_BODY")
            );
            assert_eq!(
                serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap(),
                before
            );
            assert_eq!(queue(&store, &expected, false).unwrap(), id);
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
        }
    }
    #[test]
    fn directory_refusal_codes_preserve_java_integer_without_refresh_or_pin_changes() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/apk/2.26.43.72-about-error-codes/integer-oracle.json"
        ))
        .unwrap();
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let foreign = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        for root in [false, true] {
            for case in oracle.as_array().unwrap() {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("directory-code.redb");
                let store = open(&path);
                let id = queue(&store, &peer, false).unwrap();
                actions::reserve_pending(&store).unwrap();
                let code = case["text"].as_str().unwrap();
                let error = Node::new("error")
                    .attr("code", code)
                    .attr("text", "PRIVATE_REASON");
                let children = if root {
                    vec![error]
                } else {
                    let users = vec![
                        Node::new("user")
                            .attr("jid", foreign.clone())
                            .children(vec![Node::new("error").attr("code", "503")]),
                        Node::new("user")
                            .attr("jid", peer.clone())
                            .children(vec![error]),
                    ];
                    vec![Node::new("list").children(users)]
                };
                let reply = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", if root { "error" } else { "result" })
                    .children(children);
                assert!(actions::record_reply(&store, &reply).unwrap());
                let view = inspect(&store, &peer).unwrap().unwrap();
                assert_eq!(view.status, DirectoryStatus::ServerRefused);
                assert_eq!(
                    view.server_error_code,
                    case["integer"].as_i64().map(|v| v as i32),
                    "{code}"
                );
                assert_eq!(view.failure, None);
                assert_eq!(view.fingerprint, None);
                assert_eq!(view.phone_pin_matches, None);
                assert!(
                    !serde_json::to_string(&view)
                        .unwrap()
                        .contains("PRIVATE_REASON")
                );
                assert!(queue(&store, &peer, true).is_err());
                drop(store);
                let store = open(&path);
                assert_eq!(inspect(&store, &peer).unwrap().unwrap(), view);
                assert!(actions::reserve_pending(&store).unwrap().is_empty());
            }
        }
    }

    #[test]
    fn refusal_codes_are_bound_to_one_unambiguous_selected_context() {
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let foreign = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let error = |code: &str| {
            Node::new("error")
                .attr("code", code)
                .attr("text", "SECRET_BODY")
        };
        let refusal = |who: &Jid, code: &str| {
            Node::new("user")
                .attr("jid", who.clone())
                .children(vec![error(code)])
        };
        let selected = refusal(&peer, "480");
        let other = refusal(&foreign, "503");
        let list = |users| Node::new("list").children(users);
        let mut encoded = peer.clone();
        encoded.kind = JidKind::AgentDevice;
        let cases = vec![
            (
                vec![error("503"), list(vec![selected.clone()]), error("500")],
                None,
                Some(480),
            ),
            (
                vec![list(vec![other.clone(), selected.clone()])],
                None,
                Some(480),
            ),
            (
                vec![list(vec![selected.clone(), other.clone()])],
                None,
                Some(480),
            ),
            (
                vec![list(vec![other.clone()])],
                Some(DirectoryReplyFailure::UserSet),
                None,
            ),
            (
                vec![list(vec![selected.clone(), selected.clone()])],
                Some(DirectoryReplyFailure::UserSet),
                None,
            ),
            (
                vec![list(vec![selected.clone(), refusal(&encoded, "503")])],
                Some(DirectoryReplyFailure::UserSet),
                None,
            ),
            (
                vec![list(vec![selected.clone()]), list(vec![selected.clone()])],
                Some(DirectoryReplyFailure::UserSet),
                None,
            ),
            (
                vec![list(vec![
                    Node::new("user")
                        .attr("jid", peer.clone())
                        .children(vec![error("480"), error("503")]),
                ])],
                Some(DirectoryReplyFailure::ErrorNode),
                None,
            ),
            (
                vec![list(vec![
                    other,
                    Node::new("user").attr("jid", peer.clone()),
                ])],
                Some(DirectoryReplyFailure::Bundle),
                None,
            ),
            (
                vec![error("480"), error("503")],
                Some(DirectoryReplyFailure::UserSet),
                None,
            ),
        ];
        for (children, failure, code) in cases {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("ambiguous-refusal.redb"));
            let id = queue(&store, &peer, false).unwrap();
            actions::reserve_pending(&store).unwrap();
            let reply = Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(children);
            actions::record_reply(&store, &reply).unwrap();
            let before = serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap();
            let view = inspect(&store, &peer).unwrap().unwrap();
            assert_eq!(
                view.status,
                if failure.is_none() {
                    DirectoryStatus::ServerRefused
                } else {
                    DirectoryStatus::InvalidReply
                }
            );
            assert_eq!(view.failure, failure);
            assert_eq!(view.server_error_code, code);
            assert!(view.fingerprint.is_none());
            assert!(view.signature_valid.is_none());
            assert!(
                !serde_json::to_string(&view)
                    .unwrap()
                    .contains("SECRET_BODY")
            );
            assert_eq!(
                serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap(),
                before
            );
            assert_eq!(queue(&store, &peer, false).unwrap(), id);
            assert!(queue(&store, &peer, true).is_err());
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
        }
    }

    #[test]
    fn legacy_directory_inspection_defaults_failure_to_unknown() {
        let view: DirectoryInspection = serde_json::from_value(serde_json::json!({
            "action_id":"existing", "status":"server_refused", "server_error_code":503,
            "fingerprint":null, "signature_valid":null, "phone_pin_matches":null,
            "linked_pin_matches":[]
        }))
        .unwrap();
        assert!(view.failure.is_none());
        assert_eq!(view.server_error_code, Some(503));
    }
}
