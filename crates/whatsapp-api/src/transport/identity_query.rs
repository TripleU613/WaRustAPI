//! APK 37.71 DNy.run case 40 / DF6.C8f case 0 identity-only directory read.
//! This stages correlated evidence only: no identity adoption, session mutation,
//! automatic ADV-failure retry, or companion verification is implemented here.
use super::actions;
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::{Jid, JidKind, Server},
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn invalid(message: &str) -> Error {
    Error::invalid("identity_query", message)
}
fn validate(peer: &Jid) -> Result<()> {
    // Jid permits server-only addresses for IQ destinations. This operation
    // needs a peer user (37.71 DNy.run identity/user); see peer-identity.md.
    if peer.user.is_empty() {
        return Err(invalid(
            "requires a nonempty peer user; server-only JID is not a peer",
        ));
    }
    if peer.device != 0
        || peer.agent != 0
        || !matches!(peer.server, Server::User | Server::Lid)
        || !matches!(peer.kind, JidKind::Plain | JidKind::AgentDevice)
    {
        return Err(invalid(
            "requires a primary PN or LID chat JID; companion identity verification is unfinished",
        ));
    }
    Ok(())
}
pub(super) fn same_primary_address(candidate: &Jid, expected: &Jid) -> bool {
    // JidKind is wire encoding, not APK Java subtype. The live 2026-10-08
    // contact reply established AD_JID-zero primary encoding; DF6.C8f's
    // identity query correlation is our local guard. Keep every address
    // component and namespace exact without making encoding an identity.
    !candidate.user.is_empty()
        && matches!(candidate.kind, JidKind::Plain | JidKind::AgentDevice)
        && matches!(expected.kind, JidKind::Plain | JidKind::AgentDevice)
        && candidate.user == expected.user
        && candidate.server == expected.server
        && candidate.agent == expected.agent
        && candidate.device == expected.device
}

fn marker(peer: &Jid) -> String {
    format!(
        "peer-identity-query:{}",
        hex::encode(Sha256::digest(peer.to_string().as_bytes()))
    )
}

/// Queue a single identity-only read. Every repeat returns the original ID,
/// including refused, invalid or uncertain outcomes; there is no reset/refresh.
pub fn queue(store: &ProtocolStore, peer: &Jid) -> Result<String> {
    validate(peer)?;
    store.write(|tx| {
        if let Some(id) = tx.get_json::<String>(Table::Checkpoints, marker(peer).as_bytes(), 1)? {
            return Ok(id);
        }
        let id = actions::queue_in(
            tx,
            "peer_identity_query",
            "peer identity-only inspection",
            |id| {
                // APK 37.71 DNy.run 1232..12da: identity/user, not key/user.
                Ok(Node::new("iq")
                    .attr("id", id)
                    .attr("type", "get")
                    .attr("xmlns", "encrypt")
                    .attr("to", Jid::parse("s.whatsapp.net")?)
                    .children(vec![
                        Node::new("identity")
                            .children(vec![Node::new("user").attr("jid", peer.clone())]),
                    ]))
            },
        )?;
        tx.put_json(Table::Checkpoints, marker(peer).as_bytes(), 1, &id)?;
        Ok(id)
    })
}

/// State of an identity-only read, separate from human verification or key trust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityQueryStatus {
    /// Queued locally, with no socket write reserved.
    Queued,
    /// Reserved once; no correlated reply, never automatically replayed.
    OutcomeUnknown,
    /// Correlated outer or per-user refusal.
    ServerRefused,
    /// Correlated response cannot establish the supported identity shape.
    InvalidReply,
    /// A supported primary public key was returned; not adopted or human verified.
    PrimaryIdentityReceived,
}
/// Fixed, content-free diagnosis of an unsupported saved reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityReplyFailure {
    /// Wrong IQ envelope, correlation or response type.
    Envelope,
    /// Missing, duplicate, extra or foreign user rows.
    UserSet,
    /// Missing, duplicate or non-single-byte type body.
    Type,
    /// Missing/duplicate identity, unsupported type or too-short public key.
    Key,
}
/// Redacted correlated evidence. A received identity never changes saved pins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityQueryInspection {
    /// Original durable action selector.
    pub action_id: String,
    /// Read lifecycle and supported reply classification.
    pub status: IdentityQueryStatus,
    /// Fixed local parser diagnosis; null for received identity/refusal.
    pub failure: Option<IdentityReplyFailure>,
    /// Parsed server refusal code; no raw text or inferred wait.
    pub server_error_code: Option<u16>,
    /// SHA-256 of type 5 plus the first 32 public bytes; not a safety number.
    pub fingerprint: Option<String>,
    /// Exact queried namespace only; null when no local pin exists.
    pub pin_matches: Option<bool>,
    /// Presence alone does not establish valid ADV signatures.
    pub device_identity_present: Option<bool>,
}
fn one<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    let mut found = node.child_nodes().iter().filter(|n| n.description == name);
    let value = found.next()?;
    found.next().is_none().then_some(value)
}
fn body(node: &Node) -> Option<&[u8]> {
    match &node.content {
        NodeContent::Bytes(bytes) => Some(bytes),
        NodeContent::Text(text) => Some(text.as_bytes()),
        _ => None,
    }
}
fn error_code(node: &Node) -> Option<u16> {
    node.get_attr("code")
        .and_then(|v| v.as_text())
        .and_then(|s| s.parse().ok())
}
fn public_identity(user: &Node) -> std::result::Result<[u8; 33], IdentityReplyFailure> {
    let kind = one(user, "type")
        .and_then(body)
        .ok_or(IdentityReplyFailure::Type)?;
    // APK 37.71 DF6.C8f 00de..00ee requires exactly one type byte.
    if kind.len() != 1 {
        return Err(IdentityReplyFailure::Type);
    }
    let raw = one(user, "identity")
        .and_then(body)
        .ok_or(IdentityReplyFailure::Key)?;
    // B7H.A02 0008..003c: >=33 encoded bytes, type 5, copy first 32.
    // Trailing bytes are ignored by the APK, not included in the diagnostic hash.
    if kind[0] != 5 || raw.len() < 32 {
        return Err(IdentityReplyFailure::Key);
    }
    let mut key = [0; 33];
    key[0] = 5;
    key[1..].copy_from_slice(&raw[..32]);
    Ok(key)
}

/// Inspect without queueing, adopting identity, advancing ratchets or aliasing PN/LID.
pub fn inspect(store: &ProtocolStore, peer: &Jid) -> Result<Option<IdentityQueryInspection>> {
    validate(peer)?;
    store.read(|tx| {
        let Some(id): Option<String> =
            tx.get_json(Table::Checkpoints, marker(peer).as_bytes(), 1)?
        else {
            return Ok(None);
        };
        let (action, reply) = actions::view_in(tx, &id)?
            .ok_or_else(|| invalid("identity marker has no saved action"))?;
        if action.kind != "peer_identity_query" {
            return Err(invalid("identity marker references another action kind"));
        }
        let mut view = IdentityQueryInspection {
            action_id: id.clone(),
            status: match action.status {
                actions::ActionStatus::Queued => IdentityQueryStatus::Queued,
                actions::ActionStatus::Sent => IdentityQueryStatus::OutcomeUnknown,
                _ => IdentityQueryStatus::InvalidReply,
            },
            failure: None,
            server_error_code: None,
            fingerprint: None,
            pin_matches: None,
            device_identity_present: None,
        };
        let Some(reply) = reply else {
            return Ok(Some(view));
        };
        if reply.description != "iq"
            || reply.get_attr("id").and_then(|v| v.as_text()) != Some(id.as_str())
        {
            view.failure = Some(IdentityReplyFailure::Envelope);
            return Ok(Some(view));
        }
        let kind = reply.get_attr("type").and_then(|v| v.as_text());
        if kind == Some("error") || (kind == Some("result") && one(&reply, "error").is_some()) {
            view.status = IdentityQueryStatus::ServerRefused;
            view.server_error_code = one(&reply, "error").and_then(error_code);
            return Ok(Some(view));
        }
        if kind != Some("result") || reply.child("error").is_some() {
            view.failure = Some(IdentityReplyFailure::Envelope);
            return Ok(Some(view));
        }
        let Some(list) = one(&reply, "list") else {
            view.failure = Some(IdentityReplyFailure::UserSet);
            return Ok(Some(view));
        };
        let users = list.child_nodes();
        // Correlation and ambiguity guards are local invariants, not APK requirements.
        if users.len() != 1
            || users[0].description != "user"
            || !users[0].get_attr("jid").is_some_and(|v| {
                v.as_jid()
                    .is_some_and(|jid| same_primary_address(jid, peer))
                    || v.as_text()
                        .and_then(|s| Jid::parse(s).ok())
                        .is_some_and(|jid| same_primary_address(&jid, peer))
            })
        {
            view.failure = Some(IdentityReplyFailure::UserSet);
            return Ok(Some(view));
        }
        let user = &users[0];
        if user.child("error").is_some() {
            view.status = IdentityQueryStatus::ServerRefused;
            view.server_error_code = one(user, "error").and_then(error_code);
            return Ok(Some(view));
        }
        let key = match public_identity(user) {
            Ok(key) => key,
            Err(reason) => {
                view.failure = Some(reason);
                return Ok(Some(view));
            }
        };
        let pin: Option<Vec<u8>> = tx.get_json(
            Table::PeerIdentities,
            format!("comparison-direct:{peer}").as_bytes(),
            1,
        )?;
        if let Some(pin) = pin {
            let encoded = match pin.len() {
                32 => [vec![5], pin].concat(),
                33 if pin[0] == 5 => pin,
                _ => return Err(invalid("saved primary identity has an invalid encoding")),
            };
            view.pin_matches = Some(encoded.as_slice() == key);
        }
        view.fingerprint = Some(hex::encode(Sha256::digest(key)));
        view.device_identity_present = Some(user.child("device-identity").is_some());
        view.status = IdentityQueryStatus::PrimaryIdentityReceived;
        Ok(Some(view))
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn server_only_addresses_refuse_before_query_state_and_preserve_reservations() {
        for server in [Server::User, Server::Lid] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("server-only.redb");
            let store = open(&path);
            let valid = peer();
            let id = queue(&store, &valid).unwrap();
            actions::reserve_pending(&store).unwrap();
            let selector = format!("comparison-direct:{valid}");
            store
                .write(|tx| {
                    tx.put_json(
                        Table::PeerIdentities,
                        selector.as_bytes(),
                        1,
                        &vec![5u8; 33],
                    )?;
                    tx.put_json(
                        Table::SignalSessions,
                        selector.as_bytes(),
                        1,
                        &vec![7u8; 19],
                    )
                })
                .unwrap();
            let before = store
                .read(|tx| {
                    [
                        Table::Checkpoints,
                        Table::PeerIdentities,
                        Table::SignalSessions,
                    ]
                    .into_iter()
                    .map(|table| tx.scan_after(table, 1, None, 1024))
                    .collect::<Result<Vec<_>>>()
                })
                .unwrap();
            let mut empty = valid.clone();
            empty.user.clear();
            empty.server = server;
            for kind in [JidKind::Plain, JidKind::AgentDevice] {
                empty.kind = kind;
                assert!(queue(&store, &empty).is_err());
                assert!(inspect(&store, &empty).is_err());
                assert!(!same_primary_address(&empty, &empty));
            }
            assert_eq!(
                store
                    .read(|tx| {
                        [
                            Table::Checkpoints,
                            Table::PeerIdentities,
                            Table::SignalSessions,
                        ]
                        .into_iter()
                        .map(|table| tx.scan_after(table, 1, None, 1024))
                        .collect::<Result<Vec<_>>>()
                    })
                    .unwrap(),
                before
            );
            drop(store);
            let store = open(&path);
            assert_eq!(queue(&store, &valid).unwrap(), id);
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
            assert_eq!(
                inspect(&store, &valid).unwrap().unwrap().status,
                IdentityQueryStatus::OutcomeUnknown
            );
        }
    }

    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("identity-fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn peer() -> Jid {
        Jid::parse("15555550101@s.whatsapp.net").unwrap()
    }
    fn user(peer: &Jid) -> Node {
        Node::new("user").attr("jid", peer.clone()).children(vec![
            Node::new("type").bytes(vec![5]),
            Node::new("identity").bytes(vec![11; 32]),
        ])
    }
    fn reply(id: &str, users: Vec<Node>) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .children(vec![Node::new("list").children(users)])
    }
    #[test]
    fn queue_is_durable_once_only_and_inspection_preserves_identity_session_and_devices() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("read.redb");
        let store = open(&path);
        let peer = peer();
        let id = queue(&store, &peer).unwrap();
        let request = actions::request(&store, &id).unwrap().unwrap();
        assert!(request.child("key").is_none());
        assert_eq!(
            request.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert_eq!(
            request
                .child("identity")
                .unwrap()
                .child("user")
                .unwrap()
                .get_attr("jid")
                .and_then(|v| v.as_jid()),
            Some(&peer)
        );
        assert_eq!(queue(&store, &peer).unwrap(), id);
        assert_eq!(actions::outcomes(&store).unwrap().len(), 1);
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().status,
            IdentityQueryStatus::Queued
        );
        actions::reserve_pending(&store).unwrap();
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().status,
            IdentityQueryStatus::OutcomeUnknown
        );
        assert_eq!(queue(&store, &peer).unwrap(), id);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        let selector = format!("comparison-direct:{peer}");
        let pin = [vec![5], vec![13; 32]].concat();
        store
            .write(|tx| {
                tx.put_json(Table::PeerIdentities, selector.as_bytes(), 1, &pin)?;
                tx.put_json(
                    Table::SignalSessions,
                    selector.as_bytes(),
                    1,
                    &vec![21u8; 17],
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    b"devices:fixture-preserve",
                    1,
                    &"prior-device-history",
                )
            })
            .unwrap();
        assert!(
            !actions::record_reply(
                &store,
                &reply("00000000000000000000000000000000", vec![user(&peer)])
            )
            .unwrap()
        );
        assert_eq!(
            inspect(&store, &peer).unwrap().unwrap().status,
            IdentityQueryStatus::OutcomeUnknown
        );
        assert!(actions::record_reply(&store, &reply(&id, vec![user(&peer)])).unwrap());
        let view = inspect(&store, &peer).unwrap().unwrap();
        assert_eq!(view.status, IdentityQueryStatus::PrimaryIdentityReceived);
        assert_eq!(view.pin_matches, Some(false));
        assert_eq!(view.device_identity_present, Some(false));
        let before = store
            .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 1024))
            .unwrap();
        assert_eq!(inspect(&store, &peer).unwrap().unwrap(), view);
        assert_eq!(
            store
                .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 1024))
                .unwrap(),
            before
        );
        store
            .read(|tx| {
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::PeerIdentities, selector.as_bytes(), 1)?
                        .unwrap(),
                    pin
                );
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::SignalSessions, selector.as_bytes(), 1)?
                        .unwrap(),
                    vec![21u8; 17]
                );
                assert_eq!(
                    tx.get_json::<String>(Table::Checkpoints, b"devices:fixture-preserve", 1)?
                        .unwrap(),
                    "prior-device-history"
                );
                Ok(())
            })
            .unwrap();
        let json = serde_json::to_string(&view).unwrap();
        assert!(!json.contains(&peer.to_string()));
        assert!(!json.contains("15555550101"));
        drop(store);
        let store = open(&path);
        assert_eq!(inspect(&store, &peer).unwrap().unwrap(), view);
        assert_eq!(queue(&store, &peer).unwrap(), id);
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
    }
    #[test]
    fn apk_key_type_length_and_trailing_byte_rules_are_preserved() {
        let peer = peer();
        let mut row = user(&peer);
        let key = public_identity(&row).unwrap();
        assert_eq!(key[0], 5);
        assert_eq!(&key[1..], &[11; 32]);
        row = row.children(vec![
            Node::new("type").text("\u{5}"),
            Node::new("identity").text("x".repeat(40)),
        ]);
        assert_eq!(&public_identity(&row).unwrap()[1..], &[b'x'; 32]);
        for kind in [vec![], vec![5, 5], vec![4]] {
            let r = Node::new("user").children(vec![
                Node::new("type").bytes(kind.clone()),
                Node::new("identity").bytes(vec![1; 32]),
            ]);
            assert_eq!(
                public_identity(&r).unwrap_err(),
                if kind.len() == 1 {
                    IdentityReplyFailure::Key
                } else {
                    IdentityReplyFailure::Type
                }
            );
        }
        for len in 0..32 {
            let r = Node::new("user").children(vec![
                Node::new("type").bytes(vec![5]),
                Node::new("identity").bytes(vec![1; len]),
            ]);
            assert_eq!(public_identity(&r).unwrap_err(), IdentityReplyFailure::Key);
        }
        assert!(
            public_identity(&Node::new("user").children(vec![
                Node::new("type").bytes(vec![5]),
                Node::new("type").bytes(vec![5]),
                Node::new("identity").bytes(vec![1; 32])
            ]))
            .is_err()
        );
    }
    #[test]
    fn malformed_ambiguous_foreign_and_refused_replies_never_look_like_received_identity() {
        let peer = peer();
        let mut cases = vec![
            (
                reply("unused", vec![]),
                IdentityQueryStatus::InvalidReply,
                Some(IdentityReplyFailure::UserSet),
            ),
            (
                reply("unused", vec![user(&peer), user(&peer)]),
                IdentityQueryStatus::InvalidReply,
                Some(IdentityReplyFailure::UserSet),
            ),
            (
                reply(
                    "unused",
                    vec![user(&Jid::parse("15555550102@s.whatsapp.net").unwrap())],
                ),
                IdentityQueryStatus::InvalidReply,
                Some(IdentityReplyFailure::UserSet),
            ),
            (
                reply(
                    "unused",
                    vec![Node::new("user").attr("jid", peer.clone()).children(vec![
                        Node::new("type").bytes(vec![5, 5]),
                        Node::new("identity").bytes(vec![11; 32]),
                    ])],
                ),
                IdentityQueryStatus::InvalidReply,
                Some(IdentityReplyFailure::Type),
            ),
            (
                reply(
                    "unused",
                    vec![Node::new("user").attr("jid", peer.clone()).children(
                        vec![Node::new("error").attr("code","403").attr("text","SECRET_BODY")],
                    )],
                ),
                IdentityQueryStatus::ServerRefused,
                None,
            ),
            (
                Node::new("iq").attr("type", "error").children(vec![
                    Node::new("error")
                        .attr("code", "429")
                        .attr("text", "SECRET_BODY"),
                ]),
                IdentityQueryStatus::ServerRefused,
                None,
            ),
        ];
        cases.push((
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("error").attr("code", "404")]),
            IdentityQueryStatus::ServerRefused,
            None,
        ));
        for (node, status, failure) in cases {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("case.redb"));
            let id = queue(&store, &peer).unwrap();
            actions::reserve_pending(&store).unwrap();
            actions::record_reply(&store, &node.attr("id", id.as_str())).unwrap();
            let view = inspect(&store, &peer).unwrap().unwrap();
            assert_eq!(view.status, status);
            assert_eq!(view.failure, failure);
            assert!(view.fingerprint.is_none());
            assert!(
                !serde_json::to_string(&view)
                    .unwrap()
                    .contains("SECRET_BODY")
            );
            assert_eq!(queue(&store, &peer).unwrap(), id);
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
        }
    }
    #[test]
    fn primary_wire_encoding_does_not_change_identity_reply_correlation() {
        for server in [Server::User, Server::Lid] {
            for expected_kind in [JidKind::Plain, JidKind::AgentDevice] {
                for returned_kind in [JidKind::Plain, JidKind::AgentDevice] {
                    let dir = tempfile::tempdir().unwrap();
                    let store = open(&dir.path().join("encoding.redb"));
                    let mut expected = Jid::new("111111111111111", server.clone()).unwrap();
                    expected.kind = expected_kind;
                    let id = queue(&store, &expected).unwrap();
                    actions::reserve_pending(&store).unwrap();
                    let mut returned = expected.clone();
                    returned.kind = returned_kind;
                    actions::record_reply(&store, &reply(&id, vec![user(&returned)])).unwrap();
                    let before = serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap();
                    let view = inspect(&store, &expected).unwrap().unwrap();
                    assert_eq!(view.status, IdentityQueryStatus::PrimaryIdentityReceived);
                    assert!(view.failure.is_none());
                    assert!(view.pin_matches.is_none(), "read never creates a pin");
                    let mut alternate = expected.clone();
                    alternate.kind = returned_kind;
                    assert_eq!(queue(&store, &alternate).unwrap(), id);
                    assert_eq!(inspect(&store, &alternate).unwrap(), Some(view));
                    assert_eq!(
                        serde_json::to_value(actions::outcomes(&store).unwrap()).unwrap(),
                        before
                    );
                    assert!(actions::reserve_pending(&store).unwrap().is_empty());
                }
            }
        }
    }

    #[test]
    fn encoding_correction_preserves_namespace_component_and_ambiguity_guards() {
        let expected = peer();
        let mut candidates = vec![];
        let mut foreign = expected.clone();
        foreign.user = "15555550102".into();
        candidates.push(foreign);
        let mut lid = expected.clone();
        lid.server = Server::Lid;
        candidates.push(lid);
        let mut hosted = expected.clone();
        hosted.server = Server::Hosted;
        candidates.push(hosted);
        candidates.push(expected.with_device(1));
        let mut agent = expected.clone();
        agent.agent = 1;
        candidates.push(agent);
        for kind in [JidKind::Interop, JidKind::Facebook] {
            let mut unsupported = expected.clone();
            unsupported.kind = kind;
            assert!(!same_primary_address(&unsupported, &expected));
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("unsupported.redb"));
            assert!(queue(&store, &unsupported).is_err());
            assert!(actions::outcomes(&store).unwrap().is_empty());
        }
        for candidate in candidates {
            assert!(!same_primary_address(&candidate, &expected));
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("foreign.redb"));
            let id = queue(&store, &expected).unwrap();
            actions::reserve_pending(&store).unwrap();
            actions::record_reply(&store, &reply(&id, vec![user(&candidate)])).unwrap();
            let view = inspect(&store, &expected).unwrap().unwrap();
            assert_eq!(view.status, IdentityQueryStatus::InvalidReply);
            assert_eq!(view.failure, Some(IdentityReplyFailure::UserSet));
            assert!(view.fingerprint.is_none());
        }
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("duplicate-encoding.redb"));
        let id = queue(&store, &expected).unwrap();
        actions::reserve_pending(&store).unwrap();
        let mut encoded = expected.clone();
        encoded.kind = JidKind::AgentDevice;
        actions::record_reply(&store, &reply(&id, vec![user(&expected), user(&encoded)])).unwrap();
        let view = inspect(&store, &expected).unwrap().unwrap();
        assert_eq!(view.failure, Some(IdentityReplyFailure::UserSet));
        assert!(view.fingerprint.is_none());
    }

    #[test]
    fn exact_lid_namespace_pin_comparison_does_not_adopt_or_guess_phone_aliases() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("lid.redb"));
        let lid = Jid::parse("111111111111111@lid").unwrap();
        let id = queue(&store, &lid).unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::PeerIdentities,
                    format!("comparison-direct:{lid}").as_bytes(),
                    1,
                    &vec![11u8; 32],
                )
            })
            .unwrap();
        actions::reserve_pending(&store).unwrap();
        actions::record_reply(&store, &reply(&id, vec![user(&lid)])).unwrap();
        assert_eq!(
            inspect(&store, &lid).unwrap().unwrap().pin_matches,
            Some(true)
        );
        assert!(inspect(&store, &peer()).unwrap().is_none());
        assert!(
            queue(&store, &lid.with_device(3))
                .unwrap_err()
                .to_string()
                .contains("companion identity verification is unfinished")
        );
    }
}
