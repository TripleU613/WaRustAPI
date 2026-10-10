//! First phase of APK 37.71 identity notification processing: a durable key read.
//! Notification bytes are a refresh trigger, never replacement-key authority.
//! Invariant primary classic replies can be applied under the explicit existing
//! APK-authenticated policy, including server-authorized same-key refresh. A notification ACK records handling of the control,
//! independently of whether its key read is completed or its key is adopted.
use super::{actions, messaging};
pub(crate) mod application;
mod attestation;
mod guard;
use crate::{
    Error, Result,
    binary::Node,
    model::jid::{JidKind, Server},
    session::AccountRecord,
    store::{ProtocolStore, Table},
};
pub use attestation::{AttestationAccountType, AttestationInspection, AttestationStatus};
pub use guard::RefreshGuard;
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

/// APK 37.71 14z.A09 schedules the identity worker without calling A07(true).
/// Base 0yO.A01 therefore ACKs after dispatch, not after bundle adoption.
/// The authenticated caller must durably capture/handle the control before write.
/// Unsupported routes are outside this builder; no key/message trust is implied.
pub fn notification_ack(node: &Node) -> Result<Option<Node>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("encrypt")
        || node
            .child_nodes()
            .first()
            .is_none_or(|v| v.description != "identity")
    {
        return Ok(None);
    }
    let Some(from) = super::receipts::jid(node, "from").filter(|peer| {
        !peer.user.is_empty()
            && peer.agent == 0
            && matches!(peer.server, Server::User | Server::Lid)
            && matches!(peer.kind, JidKind::Plain | JidKind::AgentDevice)
    }) else {
        return Ok(None);
    };
    let id = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|id| !id.is_empty() && id.len() <= 128)
        .ok_or_else(|| {
            Error::invalid(
                "identity_notification",
                "missing or excessive notification id",
            )
        })?;
    let participant = node
        .get_attr("participant")
        .map(|_| {
            super::receipts::jid(node, "participant").ok_or_else(|| {
                Error::invalid("identity_notification", "invalid notification participant")
            })
        })
        .transpose()?;
    // CmG.A02/A00 -> CSu.A00 preserves root from/id/type/participant.
    // Cp2.A05 uses notification type unchanged (only delivery receipts suppress
    // type). Its notification descriptor does not carry recipient/category/edit.
    let mut ack = Node::new("ack")
        .attr("id", id)
        .attr("to", from)
        .attr("class", "notification")
        .attr("type", "encrypt");
    if let Some(participant) = participant {
        ack = ack.attr("participant", participant);
    }
    Ok(Some(ack))
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
    /// Original notification-time identity provenance, never admission authority.
    pub refresh_guard: RefreshGuard,
    /// Selected-device attestation verification; null until a supported bundle.
    pub attestation: Option<AttestationInspection>,
    /// Selected user's server-provided child presence; null until a parsed bundle.
    /// This is not a local configuration switch or full native admission.
    pub force_override_e2ee_state: Option<bool>,
    /// Durable historical application fact, not current key agreement or trust.
    pub applied: bool,
}

/// Inspect an identity_refresh action, its original target and current pin in
/// one store snapshot. `actions --show ID` uses this without printing raw nodes.
pub fn inspect(store: &ProtocolStore, id: &str) -> Result<Option<Inspection>> {
    store.read(|tx| inspect_in(tx, id))
}

fn inspect_in(tx: &crate::store::ReadTx<'_>, id: &str) -> Result<Option<Inspection>> {
    use super::peer_keys::DirectoryStatus;

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
    let pin: Option<Vec<u8>> = tx.get_json(
        Table::PeerIdentities,
        format!("comparison-direct:{peer}").as_bytes(),
        1,
    )?;
    let refresh_guard = guard::inspect_in(tx, id, &peer, pin.as_deref())?;
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
        refresh_guard,
        attestation: None,
        force_override_e2ee_state: None,
        applied: application::applied_in(tx, id, &peer)?,
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
    } else if root_errors.is_empty() && lists.len() == 1 && users.len() == 1 && errors.len() == 1 {
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
    if users.len() == 1 {
        // APK 37.71 Bot.A03 0370..0384 / 1xi.A1U uses selected-user child presence.
        result.force_override_e2ee_state =
            Some(users[0].child("force_override_e2ee_state").is_some());
        result.attestation = Some(attestation::inspect(
            users[0],
            bundle
                .identity_key()
                .map_err(|_| Error::invalid("identity_notification", "invalid bundle identity"))?,
        ));
    }
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
}

/// Result of an offline cryptographic preview. No candidate bytes are exported.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SessionPreview {
    /// Existing correlated-reply and signature diagnostics.
    pub inspection: Inspection,
    /// Fixed reason for skipping, or `prepared` after successful derivation.
    pub outcome: &'static str,
    /// Whether a serialized session existed in the inspected store snapshot.
    pub held_session: bool,
    /// Number of old session states in the staged candidate; null if not derived.
    pub archived_states: Option<usize>,
    /// Native CPB admission is not evaluated by this cryptographic preview.
    pub admission_evaluated: bool,
    /// Always false: the candidate is discarded without changing saved state.
    pub persisted: bool,
}

/// Derive and discard a primary classic same-identity session from a saved read.
/// No network, message, pin change, reservation, notification ACK or store write.
/// Attestation/PQ/companion admission is deliberately outside this preview.
pub async fn preview_session(store: &ProtocolStore, id: &str) -> Result<SessionPreview> {
    use super::peer_keys::DirectoryStatus;
    let snapshot = store.read(|tx| {
        let inspection = inspect_in(tx, id)?.ok_or_else(|| {
            Error::invalid("identity_notification", "not an identity refresh action")
        })?;
        let request = actions::request_in(tx, id)?
            .ok_or_else(|| Error::invalid("identity_notification", "missing saved key request"))?;
        let peer = request
            .child("key")
            .and_then(|key| key.child_nodes().first())
            .and_then(|user| super::receipts::jid(user, "jid"))
            .ok_or_else(|| {
                Error::invalid("identity_notification", "invalid saved key request target")
            })?;
        let key = format!("comparison-direct:{peer}");
        let pin: Option<Vec<u8>> = tx.get_json(Table::PeerIdentities, key.as_bytes(), 1)?;
        let session: Option<Vec<u8>> = tx.get_json(Table::SignalSessions, key.as_bytes(), 1)?;
        let account = AccountRecord::load(tx)?;
        let reply = actions::view_in(tx, id)?.and_then(|(_, reply)| reply);
        Ok((
            inspection,
            peer,
            pin,
            session.map(zeroize::Zeroizing::new),
            account,
            reply,
        ))
    })?;
    let (inspection, peer, pin, session, account, reply) = snapshot;
    let mut result = SessionPreview {
        inspection,
        outcome: "reply_not_verified",
        held_session: session.is_some(),
        archived_states: None,
        admission_evaluated: false,
        persisted: false,
    };
    if result.inspection.status != DirectoryStatus::VerifiedBundle {
        return Ok(result);
    }
    if !matches!(
        result.inspection.refresh_guard,
        RefreshGuard::Matches | RefreshGuard::Untracked
    ) {
        result.outcome = "stale_refresh_guard";
        return Ok(result);
    }
    if peer.device != 0 {
        result.outcome = "companion_admission_unported";
        return Ok(result);
    }
    let reply =
        reply.ok_or_else(|| Error::invalid("identity_notification", "missing saved reply"))?;
    let bundle = crate::signal::direct::prekey_bundle_for(&reply, &peer)?;
    let user = reply
        .child("list")
        .and_then(|list| {
            list.child_nodes().iter().find(|user| {
                user.description == "user"
                    && super::receipts::jid(user, "jid")
                        .is_some_and(|jid| super::identity_query::same_primary_address(&jid, &peer))
            })
        })
        .ok_or_else(|| Error::invalid("identity_notification", "missing response user"))?;
    // Bot.A03 0248..02a6 carries non_e2ee_attestation into CPB; absence
    // selects E2EE there but does not prove our untracked stored CPB state.
    // Never use a classic-only preview to skip present native admission data.
    if user.child("non_e2ee_attestation").is_some() {
        result.outcome = "attestation_admission_unported";
        return Ok(result);
    }
    if user.child("pqkey").is_some() {
        result.outcome = "pq_session_unported";
        return Ok(result);
    }
    let Some(pin) = pin else {
        result.outcome = "missing_saved_identity";
        return Ok(result);
    };
    // The read snapshot is not an OCC commit guard: this operation never
    // commits a candidate or treats a matching pin as native admission.
    if bundle
        .identity_key()
        .map_err(|_| Error::invalid("identity_notification", "invalid bundle identity"))?
        .serialize()
        .as_ref()
        != pin.as_slice()
    {
        result.outcome = "identity_replacement_unported";
        return Ok(result);
    }
    let account =
        account.ok_or_else(|| Error::invalid("identity_notification", "missing saved account"))?;
    let candidate = crate::signal::direct::prepare_session_refresh(
        &account,
        &peer,
        &bundle,
        session.as_deref().map(Vec::as_slice),
        &pin,
    )
    .await?;
    let record = wacore_libsignal::protocol::SessionRecord::deserialize(&candidate)
        .map_err(|_| Error::invalid("identity_notification", "invalid staged session"))?;
    result.outcome = "prepared";
    result.archived_states = Some(record.previous_session_count());
    Ok(result)
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
        guard::capture_in(tx, &action, &peer, &pin)?;
        tx.put_json(Table::Checkpoints, event_marker.as_bytes(), 1, &action)?;
        tx.put_json(Table::Checkpoints, peer_marker.as_bytes(), 1, &action)?;
        Ok(Refresh::Queued(action))
    })
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::model::jid::Jid;
    use crate::{
        model::ids::AccountId,
        session::RegistrationStatus,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::{IdentityKeyPair, KeyPair};

    pub(super) fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("fixture").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
        })
        .unwrap()
    }
    pub(in crate::transport) fn account() -> AccountRecord {
        AccountRecord {
            registration: RegistrationStatus::Registered,
            own_jid: Some(Jid::parse("15555550999@s.whatsapp.net").unwrap()),
            ..Default::default()
        }
    }
    pub(in crate::transport) fn notification(peer: &str, id: &str) -> Node {
        Node::new("notification")
            .attr("type", "encrypt")
            .attr("from", peer)
            .attr("id", id)
            .children(vec![Node::new("identity")])
    }
    #[test]
    fn notification_ack_preserves_descriptor_and_rejects_unsupported_metadata() {
        // APK 37.71 CmG.A00/A02 and Cp2.A05, not key-read peer canonicalization.
        for route in [
            "15555550101@s.whatsapp.net",
            "15555550101:4@s.whatsapp.net",
            "10000000001@lid",
            "10000000001:4@lid",
        ] {
            let from = Jid::parse(route).unwrap();
            for typed in [false, true] {
                let mut input = notification(route, "fixture-ack");
                if typed {
                    input = input.attr("from", from.clone());
                }
                input = input
                    .attr("participant", "15555550202:2@s.whatsapp.net")
                    .attr("recipient", "15555550303@s.whatsapp.net")
                    .attr("category", "private")
                    .attr("edit", "1")
                    .attr("lid", "10000000002@lid")
                    .attr("display_name", "private");
                let ack = notification_ack(&input).unwrap().unwrap();
                let expected = Node::new("ack")
                    .attr("id", "fixture-ack")
                    .attr("to", from.clone())
                    .attr("class", "notification")
                    .attr("type", "encrypt")
                    .attr(
                        "participant",
                        Jid::parse("15555550202:2@s.whatsapp.net").unwrap(),
                    );
                assert_eq!(
                    crate::binary::Encoder::new(
                        crate::binary::TokenDictionary::comparison_2_26_37_71()
                    )
                    .encode(&ack)
                    .unwrap(),
                    crate::binary::Encoder::new(
                        crate::binary::TokenDictionary::comparison_2_26_37_71()
                    )
                    .encode(&expected)
                    .unwrap()
                );
                assert!(ack.child_nodes().is_empty());
            }
        }
        for route in ["s.whatsapp.net", "123@g.us", "123@broadcast", "bad-private"] {
            assert!(
                notification_ack(&notification(route, "fixture-ack"))
                    .unwrap()
                    .is_none()
            );
        }
        let input = notification("15555550101@s.whatsapp.net", "fixture-ack");
        for unrelated in [
            input.clone().attr("type", "other"),
            input
                .clone()
                .children(vec![Node::new("count"), Node::new("identity")]),
        ] {
            assert!(notification_ack(&unrelated).unwrap().is_none());
        }
        for bad in [
            input.clone().attr("id", ""),
            input.clone().attr("id", "x".repeat(129)),
            input.attr("participant", "private:invalid@s.whatsapp.net"),
        ] {
            let error = notification_ack(&bad).unwrap_err().to_string();
            assert!(!error.contains("private:invalid@s.whatsapp.net"));
        }
    }

    pub(in crate::transport) fn install_pin(store: &ProtocolStore, peer: &Jid) -> IdentityKeyPair {
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
    pub(in crate::transport) fn reply(
        peer: &Jid,
        id: &str,
        identity: &IdentityKeyPair,
        valid: bool,
    ) -> Node {
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

    #[tokio::test]
    async fn session_preview_is_read_only_and_refuses_unported_inputs() {
        use crate::session::{KeyMaterial, SignalIdentityKey};
        for case in [
            "prepared",
            "no_session",
            "companion",
            "attestation",
            "pq",
            "changed",
            "forged",
            "queued",
            "uncertain",
            "refused",
            "bad_session",
            "missing_pin",
            "session_identity_mismatch",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("preview.redb");
            let store = open(&path);
            let mut account = account();
            let local = IdentityKeyPair::generate(&mut rand::rng());
            account.signal_registration_id = Some(31);
            account.signal_identity = Some(SignalIdentityKey(KeyMaterial {
                public: local.public_key().public_key_bytes().try_into().unwrap(),
                private: *local.private_key().serialize(),
            }));
            let peer = Jid::parse(if case == "companion" {
                "222222222222222:4@lid"
            } else {
                "222222222222222@lid"
            })
            .unwrap();
            let identity = install_pin(&store, &peer);
            let Refresh::Queued(id) = queue(
                &store,
                &account,
                &notification(&peer.to_string(), "preview-event"),
            )
            .unwrap() else {
                panic!("expected read")
            };
            let mut response = reply(&peer, &id, &identity, case != "forged");
            if case == "changed" {
                response = reply(
                    &peer,
                    &id,
                    &IdentityKeyPair::generate(&mut rand::rng()),
                    true,
                );
            }
            let valid_reply = reply(&peer, &id, &identity, true);
            let held_bundle =
                crate::signal::direct::prekey_bundle_for(&valid_reply, &peer).unwrap();
            let mut held = crate::signal::direct::prepare_session_refresh(
                &account,
                &peer,
                &held_bundle,
                None,
                &identity.identity_key().serialize(),
            )
            .await
            .unwrap();
            if case == "session_identity_mismatch" {
                let foreign = IdentityKeyPair::generate(&mut rand::rng());
                let foreign_bundle = crate::signal::direct::prekey_bundle_for(
                    &reply(&peer, &id, &foreign, true),
                    &peer,
                )
                .unwrap();
                held = crate::signal::direct::prepare_session_refresh(
                    &account,
                    &peer,
                    &foreign_bundle,
                    None,
                    &foreign.identity_key().serialize(),
                )
                .await
                .unwrap();
            }
            store
                .write(|tx| {
                    account.save(tx)?;
                    let key = format!("comparison-direct:{peer}");
                    tx.put_json(
                        Table::SignalSessions,
                        key.as_bytes(),
                        1,
                        &if case == "bad_session" {
                            vec![1u8, 2, 3]
                        } else {
                            held.to_vec()
                        },
                    )?;
                    if case == "no_session" {
                        tx.delete(Table::SignalSessions, key.as_bytes())?;
                    }
                    if case == "missing_pin" {
                        tx.delete(Table::PeerIdentities, key.as_bytes())?;
                    }
                    Ok(())
                })
                .unwrap();
            if matches!(case, "attestation" | "pq") {
                let crate::binary::NodeContent::Children(lists) = &mut response.content else {
                    panic!("fixture lists")
                };
                let crate::binary::NodeContent::Children(users) = &mut lists[0].content else {
                    panic!("fixture users")
                };
                let crate::binary::NodeContent::Children(fields) = &mut users[0].content else {
                    panic!("fixture fields")
                };
                fields.push(Node::new(if case == "pq" {
                    "pqkey"
                } else {
                    "non_e2ee_attestation"
                }));
            }
            if case != "queued" {
                actions::reserve_pending(&store).unwrap();
                if case == "refused" {
                    response = Node::new("iq")
                        .attr("id", id.as_str())
                        .attr("type", "error")
                        .children(vec![Node::new("error").attr("code", "429")]);
                }
                if case != "uncertain" {
                    assert!(actions::record_reply(&store, &response).unwrap());
                }
            }
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        Table::ALL
                            .iter()
                            .map(|table| tx.scan_after(*table, 1, None, 512))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            let head = store.journal_head().unwrap();
            let result = preview_session(&store, &id).await;
            if matches!(case, "bad_session" | "session_identity_mismatch") {
                let error = result.unwrap_err().to_string();
                assert!(error.contains(if case == "bad_session" {
                    "Signal"
                } else {
                    "saved session identity does not match pin"
                }));
                assert!(!error.contains(&peer.user));
            } else {
                let result = result.unwrap();
                assert_eq!(
                    result.outcome,
                    match case {
                        "prepared" | "no_session" => "prepared",
                        "companion" => "companion_admission_unported",
                        "attestation" => "attestation_admission_unported",
                        "pq" => "pq_session_unported",
                        "changed" => "identity_replacement_unported",
                        "missing_pin" => "stale_refresh_guard",
                        _ => "reply_not_verified",
                    }
                );
                assert_eq!(
                    result.archived_states,
                    match case {
                        "prepared" => Some(1),
                        "no_session" => Some(0),
                        _ => None,
                    }
                );
                assert!(!result.persisted && !result.admission_evaluated);
                let json = serde_json::to_string(&result).unwrap();
                assert!(!json.contains(&peer.user) && !json.contains(&id));
            }
            assert_eq!(snapshot(&store), before);
            assert_eq!(store.journal_head().unwrap(), head);
            drop(store);
            let reopened = open(&path);
            assert_eq!(snapshot(&reopened), before);
            assert_eq!(reopened.journal_head().unwrap(), head);
        }
    }

    #[tokio::test]
    async fn verified_attestation_inspection_remains_separate_from_adoption() {
        for valid in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("attestation.redb"));
            let peer = Jid::parse("222222222222222@lid").unwrap();
            install_pin(&store, &peer);
            let fetched = IdentityKeyPair::generate(&mut rand::rng());
            let Refresh::Queued(id) = queue(
                &store,
                &account(),
                &notification(&peer.to_string(), "attestation"),
            )
            .unwrap() else {
                panic!("read")
            };
            actions::reserve_pending(&store).unwrap();
            let body = vec![8u8, 1, 16, 7];
            let signature = fetched
                .private_key()
                .calculate_signature(&[vec![6u8, 7], body.clone()].concat(), &mut rand::rng())
                .unwrap();
            let mut response = reply(&peer, &id, &fetched, true);
            let crate::binary::NodeContent::Children(lists) = &mut response.content else {
                panic!("list")
            };
            let crate::binary::NodeContent::Children(users) = &mut lists[0].content else {
                panic!("users")
            };
            let crate::binary::NodeContent::Children(fields) = &mut users[0].content else {
                panic!("fields")
            };
            fields.push(
                Node::new("non_e2ee_attestation")
                    .attr("account_type", "hybrid_e2ee")
                    .children(vec![
                        Node::new("value").bytes(body),
                        Node::new("signature").bytes(if valid {
                            signature.to_vec()
                        } else {
                            vec![0; 64]
                        }),
                    ]),
            );
            actions::record_reply(&store, &response).unwrap();
            let before = protected(&store);
            let head = store.journal_head().unwrap();
            let inspection = inspect(&store, &id).unwrap().unwrap();
            assert_eq!(inspection.pin_matches, Some(false));
            assert_eq!(inspection.signature_valid, Some(true));
            assert_eq!(
                inspection.attestation.as_ref().unwrap().status,
                if valid {
                    AttestationStatus::Verified
                } else {
                    AttestationStatus::InvalidSignature
                }
            );
            assert_eq!(
                inspection.attestation.as_ref().unwrap().signature_valid,
                Some(valid)
            );
            assert_eq!(
                preview_session(&store, &id).await.unwrap().outcome,
                "attestation_admission_unported"
            );
            assert_eq!(protected(&store), before);
            assert_eq!(store.journal_head().unwrap(), head);
            let json = serde_json::to_string(&inspection).unwrap();
            assert!(!json.contains(&peer.user) && !json.contains(&id));
        }
    }

    fn transition(store: &ProtocolStore, peer: &Jid, current: &[u8], incoming: bool) {
        store
            .write(|tx| {
                let key = format!("comparison-direct:{peer}");
                let previous: Vec<u8> = tx
                    .get_json(Table::PeerIdentities, key.as_bytes(), 1)?
                    .unwrap();
                if incoming {
                    super::super::identity_conflicts::archive_replacement(
                        tx,
                        peer,
                        Some(&previous),
                        None,
                        current,
                    )?;
                } else {
                    super::super::identity_conflicts::archive_bundle_replacement(
                        tx,
                        peer,
                        Some(&previous),
                        None,
                        current,
                    )?;
                }
                tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &current.to_vec())
            })
            .unwrap();
    }

    #[test]
    fn original_guard_survives_duplicate_and_primary_companion_aba() {
        for peer in [
            "15555550100@s.whatsapp.net",
            "222222222222222@lid",
            "15555550100:4@s.whatsapp.net",
            "222222222222222:4@lid",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("aba.redb");
            let store = open(&path);
            let peer = Jid::parse(peer).unwrap();
            let a = install_pin(&store, &peer);
            let b = IdentityKeyPair::generate(&mut rand::rng());
            let node = notification(&peer.to_string(), "original");
            let Refresh::Queued(id) = queue(&store, &account(), &node).unwrap() else {
                panic!("read")
            };
            let snapshot_key = format!("identity-refresh-snapshot:{id}");
            let baseline = store
                .read(|tx| tx.get(Table::Checkpoints, snapshot_key.as_bytes(), 1))
                .unwrap();
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().refresh_guard,
                RefreshGuard::Matches
            );
            transition(&store, &peer, &b.identity_key().serialize(), true);
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().refresh_guard,
                RefreshGuard::PinChanged
            );
            transition(&store, &peer, &a.identity_key().serialize(), false);
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().refresh_guard,
                RefreshGuard::TransitionChanged
            );
            assert_eq!(
                queue(&store, &account(), &node).unwrap(),
                Refresh::Queued(id.clone())
            );
            assert_eq!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, snapshot_key.as_bytes(), 1))
                    .unwrap(),
                baseline
            );
            actions::reserve_pending(&store).unwrap();
            assert!(actions::record_reply(&store, &reply(&peer, &id, &a, true)).unwrap());
            let inspection = inspect(&store, &id).unwrap().unwrap();
            assert_eq!(inspection.pin_matches, Some(true));
            assert_eq!(inspection.refresh_guard, RefreshGuard::TransitionChanged);
            let head = store.journal_head().unwrap();
            drop(store);
            let store = open(&path);
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().refresh_guard,
                RefreshGuard::TransitionChanged
            );
            assert_eq!(store.journal_head().unwrap(), head);
            // A distinct notification can read current state; it cannot repair
            // or replay the stale original event/action.
            let Refresh::Queued(next) = queue(
                &store,
                &account(),
                &notification(&peer.to_string(), "new-event"),
            )
            .unwrap() else {
                panic!("new read")
            };
            assert_ne!(next, id);
            assert_eq!(
                inspect(&store, &next).unwrap().unwrap().refresh_guard,
                RefreshGuard::Matches
            );
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().refresh_guard,
                RefreshGuard::TransitionChanged
            );
        }
    }

    #[test]
    fn guard_scopes_user_device_and_namespace_without_freezing_ratchets() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("scope.redb"));
        let peer = Jid::parse("222222222222222:4@lid").unwrap();
        install_pin(&store, &peer);
        let Refresh::Queued(id) = queue(
            &store,
            &account(),
            &notification(&peer.to_string(), "scope"),
        )
        .unwrap() else {
            panic!("read")
        };
        for other in ["15555550100@s.whatsapp.net", "222222222222222:5@lid"] {
            let other = Jid::parse(other).unwrap();
            install_pin(&store, &other);
            transition(
                &store,
                &other,
                &IdentityKeyPair::generate(&mut rand::rng())
                    .identity_key()
                    .serialize(),
                true,
            );
            assert_eq!(
                inspect(&store, &id).unwrap().unwrap().refresh_guard,
                RefreshGuard::Matches
            );
        }
        store
            .write(|tx| {
                tx.put_json(
                    Table::SignalSessions,
                    format!("comparison-direct:{peer}").as_bytes(),
                    1,
                    &vec![4u8, 5, 6],
                )
            })
            .unwrap();
        assert_eq!(
            inspect(&store, &id).unwrap().unwrap().refresh_guard,
            RefreshGuard::Matches
        );
        let primary = peer.to_non_ad();
        install_pin(&store, &primary);
        transition(
            &store,
            &primary,
            &IdentityKeyPair::generate(&mut rand::rng())
                .identity_key()
                .serialize(),
            false,
        );
        assert_eq!(
            inspect(&store, &id).unwrap().unwrap().refresh_guard,
            RefreshGuard::PrimaryVersionChanged
        );
    }

    #[tokio::test]
    async fn stale_preview_historical_missing_and_corrupt_snapshots_never_repair_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("snapshot.redb"));
        let peer = Jid::parse("222222222222222@lid").unwrap();
        let a = install_pin(&store, &peer);
        let node = notification(&peer.to_string(), "guard");
        let Refresh::Queued(id) = queue(&store, &account(), &node).unwrap() else {
            panic!("read")
        };
        actions::reserve_pending(&store).unwrap();
        actions::record_reply(&store, &reply(&peer, &id, &a, true)).unwrap();
        let key = format!("identity-refresh-snapshot:{id}");
        let baseline: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        transition(
            &store,
            &peer,
            &IdentityKeyPair::generate(&mut rand::rng())
                .identity_key()
                .serialize(),
            true,
        );
        let before = protected(&store);
        assert_eq!(
            preview_session(&store, &id).await.unwrap().outcome,
            "stale_refresh_guard"
        );
        assert_eq!(protected(&store), before);
        store
            .write(|tx| tx.delete(Table::Checkpoints, key.as_bytes()))
            .unwrap();
        assert_eq!(
            inspect(&store, &id).unwrap().unwrap().refresh_guard,
            RefreshGuard::Untracked
        );
        assert_eq!(
            queue(&store, &account(), &node).unwrap(),
            Refresh::Queued(id.clone())
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        for malformed in [
            "missing_primary",
            "missing_transition",
            "invalid_transition",
            "other_peer",
            "extra",
        ] {
            let mut value = baseline.clone();
            match malformed {
                "missing_primary" => {
                    value.as_object_mut().unwrap().remove("primary_version");
                }
                "missing_transition" => {
                    value.as_object_mut().unwrap().remove("transition");
                }
                "invalid_transition" => {
                    value["transition"] = serde_json::json!("private-credential-marker")
                }
                "other_peer" => {
                    value["peer"] =
                        serde_json::to_value(Jid::parse("111111111111111@lid").unwrap()).unwrap()
                }
                _ => value["unknown"] = serde_json::json!(true),
            }
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &value))
                .unwrap();
            let error = inspect(&store, &id).unwrap_err().to_string();
            assert!(error.contains("invalid saved refresh snapshot"));
            assert!(!error.contains("private-credential-marker") && !error.contains(&peer.user));
            assert_eq!(protected(&store), before);
        }
        let mut changed_generation = baseline;
        changed_generation["generation"] = serde_json::json!(999);
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &changed_generation))
            .unwrap();
        assert_eq!(
            inspect(&store, &id).unwrap().unwrap().refresh_guard,
            RefreshGuard::GenerationChanged
        );
        let transition_key = store
            .read(|tx| {
                Ok(tx
                    .scan_after(Table::Checkpoints, 1, None, 256)?
                    .into_iter()
                    .find(|(key, _)| key.starts_with(b"identity-transition:"))
                    .unwrap()
                    .0)
            })
            .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    &transition_key,
                    1,
                    &"private-invalid-epoch",
                )
            })
            .unwrap();
        let unchanged = protected(&store);
        let head = store.journal_head().unwrap();
        let failed: Result<()> = store.write(|tx| {
            super::super::identity_conflicts::archive_replacement(tx, &peer, Some(&[1]), None, &[2])
        });
        let error = failed.unwrap_err().to_string();
        assert!(error.contains("invalid saved identity transition"));
        assert!(!error.contains("private-invalid-epoch"));
        assert_eq!(protected(&store), unchanged);
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn failed_snapshot_capture_rolls_back_action_and_all_markers() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("rollback.redb"));
        let peer = Jid::parse("222222222222222@lid").unwrap();
        install_pin(&store, &peer);
        let key = format!("primary-device-version:{peer}");
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key.as_bytes(),
                    1,
                    &"private-invalid-counter",
                )
            })
            .unwrap();
        let before = protected(&store);
        let checkpoints = store
            .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 256))
            .unwrap();
        let head = store.journal_head().unwrap();
        let error = queue(
            &store,
            &account(),
            &notification(&peer.to_string(), "rollback"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("invalid saved primary device version"));
        assert!(!error.contains("private-invalid-counter"));
        assert!(actions::outcomes(&store).unwrap().is_empty());
        assert_eq!(protected(&store), before);
        assert_eq!(
            store
                .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 256))
                .unwrap(),
            checkpoints
        );
        assert_eq!(store.journal_head().unwrap(), head);
        store
            .write(|tx| tx.delete(Table::Checkpoints, key.as_bytes()))
            .unwrap();
        let Refresh::Queued(id) = queue(
            &store,
            &account(),
            &notification(&peer.to_string(), "rollback"),
        )
        .unwrap() else {
            panic!("corrected local counter")
        };
        assert_eq!(
            inspect(&store, &id).unwrap().unwrap().refresh_guard,
            RefreshGuard::Matches
        );
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
