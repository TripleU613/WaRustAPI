//! Normal-outbox group preparation over the existing APK-derived builders.
//! Each query is durably reserved before write; interrupted work is never
//! automatically reselected. The final crypto and original intent share a commit.
use super::{actions, device_key_batches, devices, group_fanout, group_send, groups};
use crate::{
    Error, Result,
    binary::{Encoder, Node, TokenDictionary},
    model::{
        ids::{ClientRequestId, LocalMessageId, MessageId},
        jid::{Jid, Server},
        message::MessageContent,
        send::{AttemptProgress, DeliveryFact, DeliveryFactKind},
        time::Timestamp,
    },
    outbox::{AttemptRecord, Outbox, OutboxEntry},
    profile::Limits,
    session::AccountRecord,
    store::{ProtocolStore, ReadTx, Table},
    transport::ConnectionEpoch,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const PREFIX: &str = "outbox-group:";
fn key(id: LocalMessageId) -> String {
    format!("{PREFIX}{id}")
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{ChatId, ClientRequestId},
            send::SendRequest,
        },
        signal::{direct, group},
    };
    pub(in crate::transport) fn enqueue(
        store: &ProtocolStore,
        chat: &Jid,
        id: &str,
    ) -> LocalMessageId {
        let req = SendRequest {
            client_request_id: ClientRequestId::new(id).unwrap(),
            chat: ChatId::from_jid(chat).unwrap(),
            content: MessageContent::Text {
                body: "normal group offline fixture".into(),
                quoted: None,
                style: Default::default(),
            },
        };
        store
            .write(|tx| {
                Ok(Outbox::new("fixture-namespace", Limits::default())
                    .enqueue(tx, &req)?
                    .0
                    .local_id)
            })
            .unwrap()
    }
    pub(in crate::transport) fn membership(query: &Node, own: &Jid, users: &[Jid]) -> Node {
        Node::new("iq")
            .attr("type", "result")
            .attr("id", query.get_attr("id").unwrap().clone())
            .children(vec![
                Node::new("group")
                    .attr("id", super::super::receipts::jid(query, "to").unwrap().user)
                    .children(
                        std::iter::once(own)
                            .chain(users)
                            .map(|j| Node::new("participant").attr("jid", j.clone()))
                            .collect(),
                    ),
            ])
    }
    pub(in crate::transport) fn device_reply(query: &Node) -> Node {
        let users = query
            .child("usync")
            .unwrap()
            .child("list")
            .unwrap()
            .child_nodes()
            .iter()
            .map(|u| {
                Node::new("user")
                    .attr("jid", u.get_attr("jid").unwrap().clone())
                    .children(vec![Node::new("devices").children(vec![
                        Node::new("device-list").children(vec![
                            Node::new("device").attr("id", "0").attr("key-index", "0"),
                        ]),
                    ])])
            })
            .collect();
        Node::new("iq")
            .attr("type", "result")
            .attr("id", query.get_attr("id").unwrap().clone())
            .children(vec![
                Node::new("usync").children(vec![Node::new("list").children(users)]),
            ])
    }
    pub(in crate::transport) fn key_reply(
        query: &Node,
        fixture: &group_fanout::tests::Fixture,
    ) -> Node {
        let template = &fixture.reply.child("list").unwrap().child_nodes()[0];
        let users = query
            .child("key")
            .unwrap()
            .child_nodes()
            .iter()
            .map(|u| {
                template
                    .clone()
                    .attr("jid", u.get_attr("jid").unwrap().clone())
            })
            .collect();
        Node::new("iq")
            .attr("type", "result")
            .attr("id", query.get_attr("id").unwrap().clone())
            .children(vec![Node::new("list").children(users)])
    }
    fn rows(store: &ProtocolStore) -> Vec<Vec<(Vec<u8>, Vec<u8>)>> {
        store
            .read(|tx| {
                [
                    Table::Outbox,
                    Table::SignalSessions,
                    Table::PeerIdentities,
                    Table::SenderKeys,
                    Table::Checkpoints,
                ]
                .into_iter()
                .map(|t| tx.scan(t, 1, 10000))
                .collect()
            })
            .unwrap()
    }
    #[tokio::test]
    async fn normal_group_references_decrypt_with_exact_participants_and_never_replay() {
        use super::super::group_references::tests::{reference, save_parent};
        for lid in [false, true] {
            for kind in [
                "reply",
                "reaction",
                "remove",
                "own_reply",
                "own_reaction",
                "keep",
                "unkeep",
                "own_keep",
                "own_unkeep",
                "changed_target",
                "changed_keep",
            ] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("group-reference-actor.redb");
                let store = group_fanout::tests::open(&path);
                let f = group_fanout::tests::fixture();
                let own = if lid {
                    Jid::parse("111111111111111@lid").unwrap()
                } else {
                    f.plan.own.clone()
                };
                let peer = if lid {
                    Jid::parse("222222222222222@lid").unwrap()
                } else {
                    f.plan.devices[0].clone()
                };
                if lid {
                    store
                        .write(|tx| group_send::record_own_lid(tx, &own))
                        .unwrap();
                }
                let ours = kind.starts_with("own_");
                let target = reference(&f.plan.chat, &peer, ours);
                save_parent(
                    &store,
                    &f.plan.chat,
                    &peer,
                    &super::super::quotes::conversation("retained group text"),
                )
                .unwrap();
                let reaction = kind.contains("reaction") || kind == "remove";
                let keep = kind.contains("keep");
                let content = if reaction {
                    MessageContent::Reaction {
                        target,
                        emoji: if kind == "remove" {
                            None
                        } else {
                            Some("👍".into())
                        },
                    }
                } else if keep {
                    MessageContent::KeepInChat {
                        target,
                        keep: !kind.contains("unkeep"),
                    }
                } else {
                    MessageContent::Text {
                        body: "quoted group reply".into(),
                        quoted: Some(target),
                        style: Default::default(),
                    }
                };
                let id = store
                    .write(|tx| {
                        Ok(outbox()
                            .enqueue(
                                tx,
                                &SendRequest {
                                    client_request_id: ClientRequestId::new(
                                        "original-group-reference",
                                    )?,
                                    chat: ChatId::from_jid(&f.plan.chat)?,
                                    content,
                                },
                            )?
                            .0
                            .local_id)
                    })
                    .unwrap();
                let original = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
                let (mut pending, mut query) = begin_selected(&store, id).unwrap().unwrap();
                let mut cursor = devices::DeviceSyncCursor::default();
                let mut sent = None;
                let mut distribution = None;
                for _ in 0..4 {
                    let reply = match query.get_attr("xmlns").and_then(|v| v.as_text()) {
                        // An out-of-band distribution: the server ACKs it.
                        None if query.description == "message" => {
                            let id = query.get_attr("id").unwrap().as_text().unwrap().to_owned();
                            distribution = Some(query.clone());
                            Node::new("ack").attr("class", "message").attr("id", id)
                        }
                        Some("w:g2") => membership(&query, &own, std::slice::from_ref(&peer)),
                        Some("usync") => device_reply(&query),
                        Some("encrypt") => {
                            if kind.starts_with("changed_") {
                                save_parent(
                                    &store,
                                    &f.plan.chat,
                                    &Jid::parse("333333333333333@lid").unwrap(),
                                    &super::super::quotes::conversation("different sender"),
                                )
                                .unwrap();
                            }
                            key_reply(&query, &f)
                        }
                        _ => panic!("unexpected query"),
                    };
                    let step = pending
                        .settle(&store, &f.alice, &reply, ConnectionEpoch(29), &mut cursor)
                        .await
                        .unwrap();
                    if let Some(next) = step.pending {
                        pending = next;
                        query = step.node.unwrap();
                    } else {
                        sent = step.node;
                        break;
                    }
                }
                let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
                if kind.starts_with("changed_") {
                    assert!(sent.is_none());
                    assert_eq!(entry, original);
                    assert_eq!(
                        preparation(&store, id)
                            .unwrap()
                            .unwrap()
                            .blocking_reason
                            .as_deref(),
                        Some("reference_changed")
                    );
                } else {
                    let node = sent.unwrap();
                    let skmsg = node.child("enc").unwrap();
                    assert_eq!(skmsg.get_attr("decrypt-fail").is_some(), reaction);
                    assert_eq!(node.child("meta").is_some(), reaction);
                    // A group reaction carrying distributions is never
                    // delivered (observed 2026-11-01): the fresh device gets
                    // its key first as a direct message, the reaction none.
                    assert_eq!(node.child("participants").is_none(), reaction);
                    assert_eq!(distribution.is_some(), reaction);
                    let target = match &distribution {
                        Some(direct) => direct.child("enc").unwrap(),
                        None => node.child("participants").unwrap().child_nodes()[0]
                            .child("enc")
                            .unwrap(),
                    };
                    let bytes = |n: &Node| match &n.content {
                        crate::binary::NodeContent::Bytes(b) => b.to_vec(),
                        _ => panic!("ciphertext"),
                    };
                    let mut bob = f.bobs[0].0.clone();
                    bob.own_jid = Some(peer.clone());
                    let decoded = direct::receive_prekey(
                        &bob,
                        &own,
                        (None, None),
                        &f.bobs[0].1,
                        None,
                        &bytes(target),
                    )
                    .await
                    .unwrap();
                    let (_, skdm) =
                        group::extract_skdm_from_payload(&decoded.received.plaintext).unwrap();
                    let installed = group::process_skdm(&f.plan.chat, &own, vec![], None, &skdm)
                        .await
                        .unwrap();
                    let body = group::receive_group(
                        &f.plan.chat,
                        &own,
                        Some(&installed.updated_key),
                        &bytes(skmsg),
                    )
                    .await
                    .unwrap()
                    .unwrap();
                    let expected = if ours {
                        own.to_non_ad()
                    } else {
                        peer.to_non_ad()
                    };
                    if reaction {
                        let decoded = super::super::reactions::decode(&body.plaintext).unwrap();
                        assert_eq!(decoded.target_from_sender, ours);
                        assert_eq!(decoded.emoji.is_none(), kind == "remove");
                        assert_eq!(
                            super::super::reactions::key_fields(&body.plaintext).unwrap(),
                            (Some(f.plan.chat.to_string()), true)
                        );
                        assert!(
                            body.plaintext
                                .windows(expected.to_string().len())
                                .any(|w| w == expected.to_string().as_bytes())
                        );
                    } else if keep {
                        use prost::Message as _;
                        let envelope = super::super::chat_settings::Envelope::decode(
                            body.plaintext.as_slice(),
                        )
                        .unwrap();
                        assert!(envelope.protocol.is_none());
                        let keep = envelope.keep_in_chat.unwrap();
                        let key = keep.key.unwrap();
                        assert_eq!(key.remote_jid, Some(f.plan.chat.to_string()));
                        assert_eq!(key.participant, Some(expected.to_string()));
                        assert_eq!(key.from_me, Some(ours));
                        assert_eq!(key.id.as_deref(), Some("ORIGINAL-TARGET"));
                        assert_eq!(
                            keep.keep_type,
                            Some(if kind.contains("unkeep") { 2 } else { 1 })
                        );
                        assert_eq!(keep.timestamp_ms, Some(original.accepted_at.as_millis()));
                        assert_eq!(
                            node.get_attr("type").and_then(|a| a.as_text()),
                            Some("text")
                        );
                    } else {
                        let decoded = super::super::quotes::decode(&body.plaintext).unwrap();
                        assert_eq!(decoded.participant, Some(expected.to_string()));
                        assert_eq!(decoded.stanza_id.as_deref(), Some("ORIGINAL-TARGET"));
                        assert_eq!(
                            super::super::quotes::decode(&decoded.quoted.unwrap())
                                .unwrap()
                                .text,
                            if ours { "" } else { "retained group text" }
                        );
                    }
                    assert_eq!(entry.attempts.len(), 1);
                    assert!(entry.status().is_uncertain());
                }
                let before = rows(&store);
                assert!(begin_selected(&store, id).unwrap().is_none());
                assert_eq!(before, rows(&store));
                drop(store);
                let reopened = group_fanout::tests::open(&path);
                assert_eq!(before, rows(&reopened));
            }
        }
    }
    #[tokio::test]
    async fn normal_actor_uses_only_explicit_signed_remote_replacement_policy() {
        for mode in ["accept", "pinned", "own_alias", "bad_signature"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("actor-identity.redb");
            let store = group_fanout::tests::open(&path);
            let old = group_fanout::tests::fixture();
            let mut new = group_fanout::tests::fixture();
            let peer = new.plan.devices[0].clone();
            let pin = direct::prekey_bundle_for(&old.reply, &peer)
                .unwrap()
                .identity_key()
                .unwrap()
                .serialize()
                .to_vec();
            store
                .write(|tx| {
                    new.alice.save(tx)?;
                    tx.put_json(
                        Table::PeerIdentities,
                        format!("comparison-direct:{peer}").as_bytes(),
                        1,
                        &pin,
                    )?;
                    if mode == "own_alias" {
                        tx.put_json(
                            Table::Checkpoints,
                            b"login-identity",
                            1,
                            &serde_json::json!({"lid":peer.to_string()}),
                        )?;
                    }
                    Ok(())
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
                new.reply = Node::new("iq")
                    .attr("type", "result")
                    .children(vec![Node::new("list").children(vec![user])]);
            }
            let id = enqueue(&store, &new.plan.chat, "same-original-group-intent");
            let original = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
            let (mut pending, mut query) = begin_selected(&store, id).unwrap().unwrap();
            let mut cursor = devices::DeviceSyncCursor::default();
            let policy = if mode == "pinned" {
                crate::registration::request::IdentityChangePolicy::Pinned
            } else {
                crate::registration::request::IdentityChangePolicy::ApkAuthenticated
            };
            let mut sent = None;
            for _ in 0..3 {
                let reply = match query.get_attr("xmlns").and_then(|v| v.as_text()) {
                    Some("w:g2") => membership(&query, &new.plan.own, std::slice::from_ref(&peer)),
                    Some("usync") => device_reply(&query),
                    Some("encrypt") => key_reply(&query, &new),
                    _ => panic!("unexpected stage"),
                };
                let step = pending
                    .settle_with_policy(
                        &store,
                        &new.alice,
                        &reply,
                        ConnectionEpoch(23),
                        &mut cursor,
                        policy,
                    )
                    .await
                    .unwrap();
                if let Some(next) = step.pending {
                    pending = next;
                    query = step.node.unwrap();
                } else {
                    sent = step.node;
                    break;
                }
            }
            let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
            let history: Vec<serde_json::Value> = store
                .read(|tx| {
                    Ok(tx
                        .scan(Table::Checkpoints, 1, 1000)?
                        .into_iter()
                        .filter(|(k, _)| k.starts_with(b"identity-history:"))
                        .map(|(_, v)| serde_json::from_slice(&v).unwrap())
                        .collect())
                })
                .unwrap();
            if mode == "accept" {
                let node = sent.unwrap();
                assert_eq!(node.description, "message");
                assert_eq!(entry.progress, AttemptProgress::InFlight);
                assert_eq!(entry.attempts.len(), 1);
                assert_eq!(entry.attempts[0].epoch, Some(ConnectionEpoch(23)));
                assert_eq!(history.len(), 1);
                assert_eq!(history[0]["previous_identity"], serde_json::json!(pin));
                assert_eq!(history[0]["authentication"], "signed_directory_bundle");
                let target = node.child("participants").unwrap().child_nodes()[0]
                    .child("enc")
                    .unwrap();
                let ciphertext = match &target.content {
                    crate::binary::NodeContent::Bytes(b) => b.as_ref(),
                    _ => panic!("ciphertext"),
                };
                let decrypted = direct::receive_prekey(
                    &new.bobs[0].0,
                    &new.plan.own,
                    (None, None),
                    &new.bobs[0].1,
                    None,
                    ciphertext,
                )
                .await
                .unwrap();
                assert!(group::extract_skdm_from_payload(&decrypted.received.plaintext).is_some());
            } else {
                assert!(sent.is_none());
                assert_eq!(entry, original);
                assert!(history.is_empty());
                let summary = preparation(&store, id).unwrap().unwrap();
                assert_eq!(summary.phase, "blocked");
                assert_eq!(
                    summary.blocking_reason.as_deref(),
                    Some(if mode == "bad_signature" {
                        "keys_invalid"
                    } else {
                        "peer_identity_changed"
                    })
                );
            }
            let account = store.read(AccountRecord::load).unwrap().unwrap();
            let actual = account.signal_identity.unwrap();
            let expected = new.alice.signal_identity.as_ref().unwrap();
            assert_eq!(actual.0.public, expected.0.public);
            assert_eq!(actual.0.private, expected.0.private);
            let before = rows(&store);
            assert!(begin_selected(&store, id).unwrap().is_none());
            assert_eq!(before, rows(&store));
            drop(store);
            let reopened = group_fanout::tests::open(&path);
            assert_eq!(before, rows(&reopened));
        }
    }
    #[tokio::test]
    async fn cold_pn_and_lid_groups_prepare_real_crypto_through_all_pages_and_reopen_without_replay()
     {
        for (lid, size) in [(false, 130), (true, 2)] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("group-flow.redb");
            let store = group_fanout::tests::open(&path);
            let f = group_fanout::tests::fixture();
            let own = if lid {
                Jid::parse("111111111111111@lid").unwrap()
            } else {
                f.plan.own.clone()
            };
            if lid {
                store
                    .write(|tx| group_send::record_own_lid(tx, &own))
                    .unwrap();
            }
            let users: Vec<_> = (0..size)
                .map(|i| {
                    Jid::parse(&format!(
                        "{}@{}",
                        222222222220000u64 + i,
                        if lid { "lid" } else { "s.whatsapp.net" }
                    ))
                    .unwrap()
                })
                .collect();
            let id = enqueue(&store, &f.plan.chat, "normal-group-flow");
            let (mut pending, mut query) = begin_selected(&store, id).unwrap().unwrap();
            let mut device_widths = vec![];
            let mut key_widths = vec![];
            let sent = loop {
                let reply = if query.get_attr("xmlns").and_then(|v| v.as_text()) == Some("w:g2") {
                    membership(&query, &own, &users)
                } else if query.get_attr("xmlns").and_then(|v| v.as_text()) == Some("usync") {
                    assert_eq!(
                        query
                            .child("usync")
                            .unwrap()
                            .get_attr("context")
                            .and_then(|v| v.as_text()),
                        Some("message")
                    );
                    device_widths.push(
                        query
                            .child("usync")
                            .unwrap()
                            .child("list")
                            .unwrap()
                            .child_nodes()
                            .len(),
                    );
                    device_reply(&query)
                } else {
                    key_widths.push(query.child("key").unwrap().child_nodes().len());
                    key_reply(&query, &f)
                };
                let step = pending
                    .settle(
                        &store,
                        &f.alice,
                        &reply,
                        ConnectionEpoch(17),
                        &mut devices::DeviceSyncCursor::default(),
                    )
                    .await
                    .unwrap();
                match step.pending {
                    Some(next) => {
                        pending = next;
                        query = step.node.unwrap();
                    }
                    None => break step.node.expect("final reserved send"),
                }
            };
            assert_eq!(device_widths, if lid { vec![2] } else { vec![64, 64, 2] });
            assert_eq!(
                key_widths,
                if lid {
                    vec![2]
                } else {
                    vec![32, 32, 32, 32, 2]
                }
            );
            assert_eq!(
                sent.get_attr("addressing_mode").and_then(|v| v.as_text()),
                Some(if lid { "lid" } else { "pn" })
            );
            let versions = super::super::send_versions::inspect(&store, id).unwrap();
            assert!(versions.captured);
            assert_eq!(
                versions.users.iter().map(|u| u.devices).sum::<usize>(),
                usize::try_from(size).unwrap()
            );
            assert!(
                versions
                    .users
                    .iter()
                    .all(|u| u.at_send.is_none() && u.current.is_none() && u.matches)
            );
            let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
            assert_eq!(entry.progress, AttemptProgress::InFlight);
            assert_eq!(entry.attempts.len(), 1);
            assert_eq!(entry.attempts[0].epoch, Some(ConnectionEpoch(17)));
            assert_eq!(entry.facts.len(), 1);
            assert_eq!(entry.facts[0].kind, DeliveryFactKind::DeliveryUnknown);
            let first = &sent.child("participants").unwrap().child_nodes()[0];
            let enc = first.child("enc").unwrap();
            let cipher = match &enc.content {
                crate::binary::NodeContent::Bytes(b) => b,
                _ => panic!("ciphertext"),
            };
            let (bob, signed) = &f.bobs[0];
            let received = direct::receive_prekey(bob, &own, (None, None), signed, None, cipher)
                .await
                .unwrap();
            let (_, skdm) = group::extract_skdm_from_payload(&received.received.plaintext).unwrap();
            let key = group::process_skdm(&f.plan.chat, &own, vec![], None, &skdm)
                .await
                .unwrap();
            let skmsg = sent
                .child_nodes()
                .iter()
                .find(|n| {
                    n.description == "enc"
                        && n.get_attr("type").and_then(|v| v.as_text()) == Some("skmsg")
                })
                .unwrap();
            let cipher = match &skmsg.content {
                crate::binary::NodeContent::Bytes(b) => b,
                _ => panic!("group ciphertext"),
            };
            let text = group::receive_group(&f.plan.chat, &own, Some(&key.updated_key), cipher)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                &*text.plaintext,
                &group_send::text_payload("normal group offline fixture").unwrap()
            );
            let saved = rows(&store);
            drop(store);
            let store = group_fanout::tests::open(&path);
            assert!(begin_selected(&store, id).unwrap().is_none());
            assert_eq!(saved, rows(&store));
            assert_eq!(
                super::super::send_versions::inspect(&store, id).unwrap(),
                versions
            );
            let inspection = preparations(&store).unwrap();
            assert_eq!(inspection[0].phase, "reserved");
            assert!(
                !serde_json::to_string(&inspection)
                    .unwrap()
                    .contains(&own.user)
            );
        }
    }
    #[tokio::test]
    async fn uploaded_group_media_is_chat_bound_and_late_crypto_changes_roll_back_reservation() {
        for media in ["image", "ptt", "race", "quoted"] {
            let d = tempfile::tempdir().unwrap();
            let store = group_fanout::tests::open(&d.path().join("media-group.redb"));
            let f = group_fanout::tests::fixture();
            // A quoted image replies to a member's message this store never
            // received: the participant is kept, the preview stays empty.
            let quoted = (media == "quoted").then(|| crate::model::message::MessageRef {
                key: crate::model::message::MessageKey {
                    chat: ChatId::from_jid(&f.plan.chat).unwrap(),
                    sender: Some(
                        crate::model::ids::DeviceAddress::from_jid(&f.plan.recipients[0]).unwrap(),
                    ),
                    direction: crate::model::message::Direction::Incoming,
                    id: MessageId::new("GROUP-PARENT").unwrap(),
                },
            });
            let request_id = ClientRequestId::new("group-media-original").unwrap();
            let content = if media == "race" {
                MessageContent::Text {
                    body: "staged race".into(),
                    quoted: None,
                    style: Default::default(),
                }
            } else {
                store.write(|tx|tx.put_json(Table::Checkpoints,b"message:group-media-original:upload",1,&serde_json::json!({
                    "status":"uploaded","to":f.plan.chat.to_string(),"hash":if media=="ptt" {"ptt:fixture"} else {"fixture"},
                    "protobuf":if media=="ptt" {vec![0x62u8,0]} else {vec![0x1au8,0]},
                }))).unwrap();
                let reference =
                    super::super::outgoing_media::reference(&store, &request_id, &f.plan.chat)
                        .unwrap();
                assert!(
                    super::super::outgoing_media::reference(
                        &store,
                        &request_id,
                        &Jid::parse("120363000000000002@g.us").unwrap()
                    )
                    .is_err()
                );
                MessageContent::Media {
                    reference,
                    caption: None,
                    file_name: None,
                    quoted: quoted.clone(),
                }
            };
            let request = SendRequest {
                client_request_id: request_id,
                chat: ChatId::from_jid(&f.plan.chat).unwrap(),
                content,
            };
            let id = store
                .write(|tx| {
                    Ok(Outbox::new("media-fixture", Limits::default())
                        .enqueue(tx, &request)?
                        .0
                        .local_id)
                })
                .unwrap();
            if media == "quoted" {
                let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
                let (body, kind, guard) = payload(&store, &entry, Some(&f.plan.own)).unwrap();
                assert!(matches!(kind, groups::StanzaContent::Media("image")));
                assert!(guard.is_some());
                assert_eq!(
                    super::super::media::quote_context(&body).unwrap(),
                    Some((
                        "GROUP-PARENT".to_owned(),
                        Some(f.plan.recipients[0].to_non_ad().to_string())
                    ))
                );
            }
            let (p, q) = begin_selected(&store, id).unwrap().unwrap();
            let mut cursor = devices::DeviceSyncCursor::default();
            let step = p
                .settle(
                    &store,
                    &f.alice,
                    &membership(&q, &f.plan.own, &f.plan.recipients),
                    ConnectionEpoch(1),
                    &mut cursor,
                )
                .await
                .unwrap();
            let step = step
                .pending
                .unwrap()
                .settle(
                    &store,
                    &f.alice,
                    &device_reply(&step.node.unwrap()),
                    ConnectionEpoch(1),
                    &mut cursor,
                )
                .await
                .unwrap();
            if media == "race" {
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::SenderKeys,
                            &group::sender_key_db_key(&f.plan.chat, &f.plan.own),
                            1,
                            &vec![9u8; 32],
                        )
                    })
                    .unwrap();
            }
            let reply = key_reply(step.node.as_ref().unwrap(), &f);
            let before = store
                .read(|tx| {
                    [
                        Table::SignalSessions,
                        Table::PeerIdentities,
                        Table::SenderKeys,
                        Table::Outbox,
                    ]
                    .into_iter()
                    .map(|t| tx.scan(t, 1, 100))
                    .collect::<Result<Vec<_>>>()
                })
                .unwrap();
            let next = step
                .pending
                .unwrap()
                .settle(&store, &f.alice, &reply, ConnectionEpoch(1), &mut cursor)
                .await
                .unwrap();
            if media == "race" {
                assert!(next.node.is_none());
                assert_eq!(
                    before,
                    store
                        .read(|tx| [
                            Table::SignalSessions,
                            Table::PeerIdentities,
                            Table::SenderKeys,
                            Table::Outbox
                        ]
                        .into_iter()
                        .map(|t| tx.scan(t, 1, 100))
                        .collect::<Result<Vec<_>>>())
                        .unwrap()
                );
                assert_eq!(
                    preparation(&store, id)
                        .unwrap()
                        .unwrap()
                        .blocking_reason
                        .as_deref(),
                    Some("preparation_changed")
                );
            } else {
                let sent = next.node.unwrap();
                assert_eq!(
                    sent.get_attr("type").and_then(|v| v.as_text()),
                    Some("media")
                );
                assert_eq!(
                    sent.child("enc")
                        .unwrap()
                        .get_attr("mediatype")
                        .and_then(|v| v.as_text()),
                    Some(if media == "quoted" { "image" } else { media })
                );
                assert_eq!(
                    store
                        .read(|tx| outbox().get(tx, id))
                        .unwrap()
                        .unwrap()
                        .attempts
                        .len(),
                    1
                );
            }
        }
    }

    #[tokio::test]
    async fn independent_namespaces_do_not_share_another_intents_frozen_directory_plan() {
        let d = tempfile::tempdir().unwrap();
        let store = group_fanout::tests::open(&d.path().join("namespaced-group.redb"));
        let f = group_fanout::tests::fixture();
        let mut ids = vec![];
        for (namespace, chat, user_count) in [
            ("actor-a", "120363000000000001@g.us", 2),
            ("actor-b", "120363000000000002@g.us", 1),
        ] {
            let chat = Jid::parse(chat).unwrap();
            let users: Vec<_> = (0..user_count)
                .map(|n| {
                    Jid::parse(&format!(
                        "{}@s.whatsapp.net",
                        if namespace == "actor-a" {
                            222222222220000u64 + n
                        } else {
                            333333333330000u64 + n
                        }
                    ))
                    .unwrap()
                })
                .collect();
            let request = SendRequest {
                client_request_id: ClientRequestId::new("shared-caller-id").unwrap(),
                chat: ChatId::from_jid(&chat).unwrap(),
                content: MessageContent::Text {
                    body: "namespace-specific intent".into(),
                    quoted: None,
                    style: Default::default(),
                },
            };
            let id = store
                .write(|tx| {
                    Ok(Outbox::new(namespace, Limits::default())
                        .enqueue(tx, &request)?
                        .0
                        .local_id)
                })
                .unwrap();
            let (mut p, mut q) = begin_selected(&store, id).unwrap().unwrap();
            let mut cursor = devices::DeviceSyncCursor::default();
            loop {
                let reply = match q.get_attr("xmlns").and_then(|v| v.as_text()) {
                    Some("w:g2") => membership(&q, &f.plan.own, &users),
                    Some("usync") => device_reply(&q),
                    Some("encrypt") => key_reply(&q, &f),
                    _ => panic!("query"),
                };
                let step = p
                    .settle(&store, &f.alice, &reply, ConnectionEpoch(3), &mut cursor)
                    .await
                    .unwrap();
                if let Some(next) = step.pending {
                    p = next;
                    q = step.node.unwrap();
                } else {
                    assert_eq!(step.node.unwrap().description, "message");
                    break;
                }
            }
            let inspect = preparation(&store, id).unwrap().unwrap();
            assert_eq!(inspect.phase, "reserved");
            assert_eq!(inspect.key_batches.unwrap().devices, user_count as usize);
            ids.push(id);
        }
        assert_ne!(ids[0], ids[1]);
        for id in ids {
            let before = rows(&store);
            assert!(begin_selected(&store, id).unwrap().is_none());
            assert_eq!(before, rows(&store));
        }
    }

    #[tokio::test]
    async fn refused_partial_and_interrupted_queries_preserve_original_intents_and_do_not_reselect()
    {
        for cut in [
            "membership",
            "wrong_group",
            "truncated",
            "missing_group",
            "devices",
            "keys",
            "partial",
            "changed_membership",
            "unknown",
        ] {
            let d = tempfile::tempdir().unwrap();
            let store = group_fanout::tests::open(&d.path().join("refusal.redb"));
            let f = group_fanout::tests::fixture();
            let users = f.plan.recipients.clone();
            let id = enqueue(&store, &f.plan.chat, "original-group-refusal");
            let (p, q) = begin_selected(&store, id).unwrap().unwrap();
            let p = if matches!(cut, "wrong_group" | "truncated" | "missing_group") {
                let mut reply = membership(&q, &f.plan.own, &users);
                if cut == "missing_group" {
                    reply.content = crate::binary::NodeContent::Empty;
                } else if let crate::binary::NodeContent::Children(children) = &mut reply.content {
                    children[0] = children[0].clone().attr(
                        if cut == "truncated" {
                            "truncated"
                        } else {
                            "id"
                        },
                        if cut == "truncated" {
                            "true"
                        } else {
                            "120363000000000002@g.us"
                        },
                    );
                }
                p.settle(
                    &store,
                    &f.alice,
                    &reply,
                    ConnectionEpoch(1),
                    &mut devices::DeviceSyncCursor::default(),
                )
                .await
                .unwrap();
                None
            } else if cut == "membership" {
                p.settle(
                    &store,
                    &f.alice,
                    &Node::new("iq")
                        .attr("id", q.get_attr("id").unwrap().clone())
                        .attr("type", "error"),
                    ConnectionEpoch(1),
                    &mut devices::DeviceSyncCursor::default(),
                )
                .await
                .unwrap();
                None
            } else if cut == "unknown" {
                p.expire(&store, &mut devices::DeviceSyncCursor::default())
                    .unwrap();
                None
            } else {
                let step = p
                    .settle(
                        &store,
                        &f.alice,
                        &membership(&q, &f.plan.own, &users),
                        ConnectionEpoch(1),
                        &mut devices::DeviceSyncCursor::default(),
                    )
                    .await
                    .unwrap();
                Some((step.pending.unwrap(), step.node.unwrap()))
            };
            if let Some((p, q)) = p {
                let reply = if cut == "devices" {
                    Node::new("iq")
                        .attr("type", "error")
                        .attr("id", q.get_attr("id").unwrap().clone())
                } else if cut == "partial" {
                    Node::new("iq")
                        .attr("type", "result")
                        .attr("id", q.get_attr("id").unwrap().clone())
                        .children(vec![Node::new("usync").children(vec![Node::new("list")])])
                } else {
                    device_reply(&q)
                };
                if cut == "changed_membership" {
                    store
                        .write(|tx| {
                            tx.delete(
                                Table::Checkpoints,
                                groups::key(&f.plan.chat.to_string()).as_bytes(),
                            )
                        })
                        .unwrap();
                }
                let step = p
                    .settle(
                        &store,
                        &f.alice,
                        &reply,
                        ConnectionEpoch(1),
                        &mut devices::DeviceSyncCursor::default(),
                    )
                    .await
                    .unwrap();
                if let Some(p) = step.pending {
                    let q = step.node.unwrap();
                    let reply = Node::new("iq")
                        .attr("type", "error")
                        .attr("id", q.get_attr("id").unwrap().clone());
                    p.settle(
                        &store,
                        &f.alice,
                        &reply,
                        ConnectionEpoch(1),
                        &mut devices::DeviceSyncCursor::default(),
                    )
                    .await
                    .unwrap();
                }
            }
            let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
            assert_eq!(entry.progress, AttemptProgress::Queued);
            assert!(entry.wire_id.is_none() && entry.attempts.is_empty() && entry.facts.is_empty());
            let before = rows(&store);
            assert!(begin_selected(&store, id).unwrap().is_none());
            assert!(
                super::super::outgoing::begin_selected(&store, Some(id))
                    .unwrap()
                    .is_none()
            );
            assert_eq!(before, rows(&store));
            assert_eq!(preparation(&store, id).unwrap().unwrap().phase, "blocked");
        }
    }
}
fn outbox() -> Outbox {
    Outbox::new("comparison-driver", Limits::default())
}
fn invalid(reason: &'static str) -> Error {
    Error::invalid("outbox_group", reason)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Membership,
    Devices,
    Keys,
    Reserved,
    Blocked,
}

/// Fixed diagnostics; never include server text, addresses, keys or content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Block {
    UnsupportedContent,
    MembershipRefused,
    InvalidMembership,
    OwnIdentityMissing,
    RoutingRefused,
    MembershipChanged,
    DevicesIncomplete,
    KeysRefused,
    KeysInvalid,
    CryptoPreparationRefused,
    PeerIdentityChanged,
    OutcomeUnknown,
    PreparationChanged,
    ForeignKeyHistory,
    CompanionAuthorityChanged,
    ReferenceChanged,
    /// This path rotates the sender key and carries distributions in the
    /// stanza; the server ACKs such a group reaction but delivers it to nobody
    /// (observed live 2026-11-01, `groups::sender_key_stanza`). Refused before
    /// any query until out-of-band distribution exists here.
    ReactionDistributionUnsupported,
    /// A reaction's out-of-band distribution message was refused by the
    /// server; the reserved group stanza was not sent and stays uncertain.
    DistributionRefused,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Row {
    local_id: LocalMessageId,
    phase: Phase,
    reason: Option<Block>,
    info_query: String,
    device_queries: Vec<String>,
    key_queries: u32,
    recipient_count: usize,
    device_count: usize,
}

/// Read-only progress, independent of delivery facts. Reservation is not an ACK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreparationSummary {
    /// Original opaque local intent selector.
    pub local_id: LocalMessageId,
    /// membership/devices/keys/reserved/blocked; interrupted stages are not replayed.
    pub phase: String,
    /// Fixed local reason, null when no explicit refusal/timeout was recorded.
    pub blocking_reason: Option<String>,
    /// Frozen recipient count once routing is resolved, else null.
    pub recipients: Option<usize>,
    /// Frozen addressed-device count once device resolution is complete, else null.
    pub devices: Option<usize>,
    /// Number of durably reserved selected device queries.
    pub device_queries: usize,
    /// Number of durably reserved directory-key queries.
    pub key_queries: u32,
    /// Always false: inspection does not authorize a new query or send attempt.
    pub replay_allowed: bool,
    /// Durable directory batch counts/reasons; null when no batch was required or reserved.
    pub key_batches: Option<device_key_batches::Summary>,
}
fn bundle_selector(id: LocalMessageId) -> Result<ClientRequestId> {
    ClientRequestId::new(format!("normal-group:{id}"))
}
fn summary(store: &ProtocolStore, row: &Row) -> Result<PreparationSummary> {
    let phase = serde_json::to_value(row.phase)
        .expect("fixed enum")
        .as_str()
        .unwrap()
        .to_owned();
    let reason = row.reason.map(|r| {
        serde_json::to_value(r)
            .expect("fixed enum")
            .as_str()
            .unwrap()
            .to_owned()
    });
    Ok(PreparationSummary {
        local_id: row.local_id,
        phase,
        blocking_reason: reason,
        recipients: (row.recipient_count > 0).then_some(row.recipient_count),
        devices: (row.device_count > 0).then_some(row.device_count),
        device_queries: row.device_queries.len(),
        key_queries: row.key_queries,
        replay_allowed: false,
        key_batches: device_key_batches::summary(store, &bundle_selector(row.local_id)?)?,
    })
}
/// Inspect all saved preparations across idempotency namespaces, without I/O to WhatsApp.
pub fn preparations(store: &ProtocolStore) -> Result<Vec<PreparationSummary>> {
    let rows = store.read(|tx| {
        let mut result = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(k, _)| k.clone());
            for (k, value) in rows {
                if !k.starts_with(PREFIX.as_bytes()) {
                    continue;
                }
                let row: Row = serde_json::from_slice(&value)
                    .map_err(|_| invalid("invalid saved group preparation"))?;
                if k != key(row.local_id).as_bytes() {
                    return Err(invalid("group preparation selector mismatch"));
                }
                result.push(row);
            }
        }
        Ok(result)
    })?;
    rows.iter().map(|row| summary(store, row)).collect()
}
/// Inspect one original intent while the client owns the account store.
pub fn preparation(
    store: &ProtocolStore,
    id: LocalMessageId,
) -> Result<Option<PreparationSummary>> {
    let row = store.read(|tx| {
        let row: Option<Row> = tx
            .get(Table::Checkpoints, key(id).as_bytes(), 1)?
            .map(|value| {
                serde_json::from_slice(&value)
                    .map_err(|_| invalid("invalid saved group preparation"))
            })
            .transpose()?;
        if row.as_ref().is_some_and(|r| r.local_id != id) {
            return Err(invalid("group preparation selector mismatch"));
        }
        Ok(row)
    })?;
    row.as_ref().map(|row| summary(store, row)).transpose()
}
pub(crate) fn reserved_in(tx: &ReadTx<'_>, id: LocalMessageId) -> Result<bool> {
    Ok(tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?.is_some())
}

enum Waiting {
    Membership(String),
    Devices(devices::DeviceQuery),
    Keys(String),
    /// A reserved reaction's distributions go out first as direct messages,
    /// each ACKed before the next; the group stanza follows the last.
    Distribution {
        id: String,
        rest: Vec<Node>,
        group: Box<Node>,
    },
}
/// One connection's original preparation. No resume/reset/replay API is provided.
pub(crate) struct Pending {
    entry: OutboxEntry,
    row: Row,
    waiting: Waiting,
    own: Option<Jid>,
    recipients: Vec<Jid>,
    membership: Option<[u8; 32]>,
    completed_users: usize,
    snapshot: Option<group_fanout::Snapshot>,
    plan: Option<group_send::FanoutPlan>,
    needed: Vec<Jid>,
    pub deadline: tokio::time::Instant,
}
impl std::fmt::Debug for Pending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingGroupOutbox")
            .field("phase", &self.row.phase)
            .field("recipients", &self.recipients.len())
            .field("deadline", &self.deadline)
            .finish_non_exhaustive()
    }
}
pub(crate) struct Step {
    pub pending: Option<Pending>,
    pub node: Option<Node>,
}
impl Step {
    fn waiting(pending: Pending, node: Node) -> Self {
        Self {
            pending: Some(pending),
            node: Some(node),
        }
    }
    fn done() -> Self {
        Self {
            pending: None,
            node: None,
        }
    }
}
type GroupPayload = (
    Zeroizing<Vec<u8>>,
    groups::StanzaContent<'static>,
    Option<super::group_references::ReferenceGuard>,
);
fn payload(store: &ProtocolStore, entry: &OutboxEntry, own: Option<&Jid>) -> Result<GroupPayload> {
    match &entry.intent.content {
        MessageContent::Text {
            body,
            quoted: None,
            style,
        } if !style.is_styled() => Ok((
            Zeroizing::new(group_send::text_payload(body)?),
            groups::StanzaContent::Text,
            None,
        )),
        MessageContent::Text {
            quoted: Some(_), ..
        }
        | MessageContent::Reaction { .. }
        | MessageContent::KeepInChat { .. } => {
            super::group_references::validate(
                store,
                entry.intent.chat.jid(),
                &entry.intent.content,
            )?;
            let Some(own) = own else {
                return Ok((
                    Zeroizing::new(Vec::new()),
                    groups::StanzaContent::Text,
                    None,
                ));
            };
            let (body, kind, guard) = super::group_references::guarded_payload(
                store,
                entry.intent.chat.jid(),
                own,
                &entry.intent.content,
                // APK 37.71 11x.A01 records keep time before Cgz.A02 queues
                // the send; preserve original intent time across preparation.
                entry.accepted_at,
            )?;
            Ok((Zeroizing::new(body), kind, Some(guard)))
        }
        MessageContent::Media {
            reference,
            caption: None,
            file_name: None,
            quoted,
        } => {
            let (body, media) = super::outgoing_media::resolve(
                store,
                &entry.client_request_id,
                entry.intent.chat.jid(),
                reference,
            )?;
            if quoted.is_none() {
                return Ok((body, groups::StanzaContent::Media(media), None));
            }
            if !super::quotes::media_quote_supported(&body) {
                return Err(invalid("quoted media payload shape is unsupported"));
            }
            // Quoted media follows the quoted-text reference path: the same
            // parent resolution, participant and reservation-time guard.
            super::group_references::validate(
                store,
                entry.intent.chat.jid(),
                &entry.intent.content,
            )?;
            // Support check only (no own identity yet): like quoted text, an
            // empty body, never the media without its quote.
            let Some(own) = own else {
                return Ok((
                    Zeroizing::new(Vec::new()),
                    groups::StanzaContent::Media(media),
                    None,
                ));
            };
            let (body, guard) = super::group_references::guarded_media_payload(
                store,
                entry.intent.chat.jid(),
                own,
                &entry.intent.content,
                &body,
            )?;
            Ok((
                Zeroizing::new(body),
                groups::StanzaContent::Media(media),
                Some(guard),
            ))
        }
        _ => Err(invalid(
            "group content is not implemented by the normal adapter",
        )),
    }
}
/// Reserve fresh group membership once for one eligible original intent.
pub(crate) fn begin_selected(
    store: &ProtocolStore,
    ticket: LocalMessageId,
) -> Result<Option<(Pending, Node)>> {
    let Some(entry) = store.read(|tx| outbox().get(tx, ticket))? else {
        return Ok(None);
    };
    if entry.progress != AttemptProgress::Queued
        || entry.intent.chat.jid().server != Server::Group
        || entry.wire_id.is_some()
        || !entry.attempts.is_empty()
        || store.read(|tx| reserved_in(tx, ticket))?
    {
        return Ok(None);
    }
    super::send_path::claim(
        store,
        &entry.client_request_id,
        super::send_path::SendPath::Outbox,
    )?;
    let foreign_keys = store
        .read(|tx| {
            tx.get(
                Table::Checkpoints,
                format!("message:{}:keys", entry.client_request_id).as_bytes(),
                1,
            )
        })?
        .is_some()
        || device_key_batches::summary(store, &entry.client_request_id)?.is_some();
    let supported = match payload(store, &entry, None) {
        Ok(_) => true,
        Err(Error::InvalidInput { .. }) => false,
        Err(error) => return Err(error),
    };
    let result = store.write(|tx| {
        if outbox().get_mut(tx, ticket)?.as_ref() != Some(&entry)
            || tx
                .get(Table::Checkpoints, key(ticket).as_bytes(), 1)?
                .is_some()
        {
            return Ok(None);
        }
        let mut row = Row {
            local_id: ticket,
            phase: Phase::Membership,
            reason: None,
            info_query: String::new(),
            device_queries: vec![],
            key_queries: 0,
            recipient_count: 0,
            device_count: 0,
        };
        if !supported || foreign_keys {
            row.phase = Phase::Blocked;
            row.reason = Some(if foreign_keys {
                Block::ForeignKeyHistory
            } else {
                Block::UnsupportedContent
            });
            tx.put_json(Table::Checkpoints, key(ticket).as_bytes(), 1, &row)?;
            return Ok(None);
        }
        row.info_query = actions::queue_in(tx, "group-info", "group info", |id| {
            groups::info_request(entry.intent.chat.jid(), id)
        })?;
        let node = actions::reserve_one_in(tx, &row.info_query)?;
        tx.put_json(Table::Checkpoints, key(ticket).as_bytes(), 1, &row)?;
        Ok(Some((row, node)))
    })?;
    Ok(result.map(|(row, node)| {
        (
            Pending {
                waiting: Waiting::Membership(row.info_query.clone()),
                entry,
                row,
                own: None,
                recipients: vec![],
                membership: None,
                completed_users: 0,
                snapshot: None,
                plan: None,
                needed: vec![],
                deadline: deadline(),
            },
            node,
        )
    }))
}
fn deadline() -> tokio::time::Instant {
    tokio::time::Instant::now() + std::time::Duration::from_secs(30)
}
fn membership_digest(store: &ProtocolStore, chat: &Jid) -> Result<Option<[u8; 32]>> {
    store.read(|tx| {
        Ok(tx
            .get(
                Table::Checkpoints,
                groups::key(&chat.to_string()).as_bytes(),
                1,
            )?
            .map(|raw| Sha256::digest(raw).into()))
    })
}
impl Pending {
    pub fn matches(&self, node: &Node) -> bool {
        match &self.waiting {
            Waiting::Membership(id) | Waiting::Keys(id) => {
                node.description == "iq"
                    && node.get_attr("id").and_then(|v| v.as_text()) == Some(id)
            }
            Waiting::Devices(query) => query.matches(node),
            Waiting::Distribution { id, .. } => {
                node.description == "ack"
                    && node.get_attr("id").and_then(|v| v.as_text()) == Some(id)
            }
        }
    }
    pub fn devices_pending(&self) -> bool {
        matches!(self.waiting, Waiting::Devices(_))
    }
    pub fn observe_device_query(&self, cursor: &mut devices::DeviceSyncCursor) {
        if let Waiting::Devices(query) = &self.waiting {
            cursor.observe_query(query);
        }
    }
    fn save(&self, store: &ProtocolStore, old: &Row) -> Result<()> {
        store.write(|tx| {
            let held: Option<Row> =
                tx.get_json(Table::Checkpoints, key(self.entry.local_id).as_bytes(), 1)?;
            if held.as_ref() != Some(old) {
                return Err(invalid("group preparation changed"));
            }
            tx.put_json(
                Table::Checkpoints,
                key(self.entry.local_id).as_bytes(),
                1,
                &self.row,
            )
        })
    }
    fn block(mut self, store: &ProtocolStore, reason: Block) -> Result<Step> {
        let old = self.row.clone();
        self.row.phase = Phase::Blocked;
        self.row.reason = Some(reason);
        self.save(store, &old)?;
        Ok(Step::done())
    }
    pub fn expire(
        self,
        store: &ProtocolStore,
        cursor: &mut devices::DeviceSyncCursor,
    ) -> Result<()> {
        if let Waiting::Devices(query) = &self.waiting {
            cursor.defer(query);
        }
        self.block(store, Block::OutcomeUnknown).map(|_| ())
    }
    /// Caller supplies only authenticated matching nodes from the real session loop.
    #[cfg(test)]
    pub async fn settle(
        self,
        store: &ProtocolStore,
        account: &AccountRecord,
        node: &Node,
        epoch: ConnectionEpoch,
        cursor: &mut devices::DeviceSyncCursor,
    ) -> Result<Step> {
        self.settle_with_policy(
            store,
            account,
            node,
            epoch,
            cursor,
            crate::registration::request::IdentityChangePolicy::Pinned,
        )
        .await
    }
    pub async fn settle_with_policy(
        mut self,
        store: &ProtocolStore,
        account: &AccountRecord,
        node: &Node,
        epoch: ConnectionEpoch,
        cursor: &mut devices::DeviceSyncCursor,
        policy: crate::registration::request::IdentityChangePolicy,
    ) -> Result<Step> {
        if !self.matches(node) {
            return Err(invalid("mismatched group preparation reply"));
        }
        match &self.waiting {
            Waiting::Membership(id) => {
                if !actions::record_reply(store, node)? {
                    return Err(invalid("membership action cannot settle"));
                }
                if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
                    return self.block(store, Block::MembershipRefused);
                }
                if node.child("error").is_some()
                    || !store.read(|tx| {
                        Ok(actions::view_in(tx, id)?.is_some_and(|(outcome, _)| {
                            outcome.status == actions::ActionStatus::Ok
                        }))
                    })?
                {
                    return self.block(store, Block::MembershipRefused);
                }
                let candidates: Vec<_> = node
                    .child_nodes()
                    .iter()
                    .filter(|n| n.description == "group")
                    .collect();
                if candidates.len() != 1
                    || groups::parse_group(candidates[0])
                        .and_then(|group| Jid::parse(&group.jid).ok())
                        .as_ref()
                        != Some(self.entry.intent.chat.jid())
                    || candidates[0]
                        .get_attr("truncated")
                        .is_some_and(|v| v.as_text() != Some("false"))
                {
                    return self.block(store, Block::InvalidMembership);
                }
                if !groups::apply_pending(store, id)? {
                    return self.block(store, Block::InvalidMembership);
                }
                let chat = self.entry.intent.chat.jid();
                let resolved = store.read(|tx| {
                    let group = groups::get_in(tx, chat)?
                        .ok_or_else(|| invalid("membership projection missing"))?;
                    let lid = group
                        .members
                        .first()
                        .and_then(|m| Jid::parse(&m.jid).ok())
                        .is_some_and(|j| j.server == Server::Lid);
                    let own = if lid {
                        tx.get_json(Table::Checkpoints, group_send::OWN_LID_KEY, 1)?
                    } else {
                        account.own_jid.clone()
                    };
                    let Some(own) = own else {
                        return Ok(None);
                    };
                    let users = group_send::recipients_for_group_in(tx, chat, &own)?;
                    if users.len() > 4096 {
                        return Err(invalid("group exceeds local 4096-user bound"));
                    }
                    let raw = tx
                        .get(
                            Table::Checkpoints,
                            groups::key(&chat.to_string()).as_bytes(),
                            1,
                        )?
                        .ok_or_else(|| invalid("membership projection missing"))?;
                    Ok(Some((own, users, Sha256::digest(raw).into())))
                });
                match resolved {
                    Ok(Some((own, users, digest))) => {
                        self.own = Some(own);
                        self.recipients = users;
                        self.membership = Some(digest);
                    }
                    Ok(None) => return self.block(store, Block::OwnIdentityMissing),
                    Err(Error::InvalidInput { .. }) => {
                        return self.block(store, Block::RoutingRefused);
                    }
                    Err(error) => return Err(error),
                }
                self.reserve_devices(store)
            }
            Waiting::Devices(query) => {
                let result = cursor.settle(query, store, node)?;
                if result.require_complete().is_err() {
                    return self.block(store, Block::DevicesIncomplete);
                }
                self.completed_users += self.recipients[self.completed_users..]
                    .len()
                    .min(devices::MAX_USERS_PER_QUERY);
                if self.completed_users < self.recipients.len() {
                    return self.reserve_devices(store);
                }
                if membership_digest(store, self.entry.intent.chat.jid())? != self.membership {
                    return self.block(store, Block::MembershipChanged);
                }
                let plan = match group_send::plan(
                    store,
                    self.entry.intent.chat.jid(),
                    self.own.as_ref().expect("resolved own"),
                ) {
                    Ok(plan)
                        if plan.without_device_list.is_empty()
                            && plan.recipients == self.recipients =>
                    {
                        plan
                    }
                    _ => return self.block(store, Block::RoutingRefused),
                };
                let snapshot = group_fanout::Snapshot::load(store, &plan, false)?;
                self.needed = snapshot.required_keys();
                self.snapshot = Some(snapshot);
                self.plan = Some(plan);
                self.next_keys(store, account, epoch, policy).await
            }
            Waiting::Keys(_) => {
                if !device_key_batches::settle(store, &bundle_selector(self.entry.local_id)?, node)?
                {
                    return Err(invalid("directory batch cannot settle"));
                }
                self.next_keys(store, account, epoch, policy).await
            }
            Waiting::Distribution { .. } => {
                if node.get_attr("error").is_some() {
                    return self.block(store, Block::DistributionRefused);
                }
                let Waiting::Distribution { rest, group, .. } =
                    std::mem::replace(&mut self.waiting, Waiting::Keys(String::new()))
                else {
                    unreachable!()
                };
                self.next_distribution(rest, *group)
            }
        }
    }
    fn reserve_devices(mut self, store: &ProtocolStore) -> Result<Step> {
        if membership_digest(store, self.entry.intent.chat.jid())? != self.membership {
            return self.block(store, Block::MembershipChanged);
        }
        let users: Vec<_> = self.recipients[self.completed_users..]
            .iter()
            .take(devices::MAX_USERS_PER_QUERY)
            .cloned()
            .collect();
        let old = self.row.clone();
        let (query, node) = store.write(|tx| {
            if outbox().get_mut(tx, self.entry.local_id)?.as_ref() != Some(&self.entry) {
                return Err(invalid("intent changed before device query"));
            }
            let held: Option<Row> =
                tx.get_json(Table::Checkpoints, key(self.entry.local_id).as_bytes(), 1)?;
            if held.as_ref() != Some(&old) {
                return Err(invalid("group preparation changed"));
            }
            // APK 37.71 SyncDevicesJob message preset select message context; these
            // local pages contain only the frozen group recipients, once each.
            for user in &users {
                devices::request_sync_in(tx, user, None, "group-outbox")?;
            }
            let (query, node) =
                devices::reserve_query_in(tx, devices::SyncContext::Message, users)?
                    .ok_or_else(|| invalid("selected device batch empty"))?;
            self.row.phase = Phase::Devices;
            self.row.recipient_count = self.recipients.len();
            self.row.device_queries.push(
                node.get_attr("id")
                    .and_then(|v| v.as_text())
                    .ok_or_else(|| invalid("device query selector missing"))?
                    .to_owned(),
            );
            tx.put_json(
                Table::Checkpoints,
                key(self.entry.local_id).as_bytes(),
                1,
                &self.row,
            )?;
            Ok((query, node))
        })?;
        self.waiting = Waiting::Devices(query);
        self.deadline = deadline();
        Ok(Step::waiting(self, node))
    }
    async fn next_keys(
        mut self,
        store: &ProtocolStore,
        account: &AccountRecord,
        epoch: ConnectionEpoch,
        policy: crate::registration::request::IdentityChangePolicy,
    ) -> Result<Step> {
        if membership_digest(store, self.entry.intent.chat.jid())? != self.membership {
            return self.block(store, Block::MembershipChanged);
        }
        if let Some(keys) =
            device_key_batches::summary(store, &bundle_selector(self.entry.local_id)?)?
        {
            if keys.outcome_unknown > 0 {
                return self.block(store, Block::OutcomeUnknown);
            }
            if keys.refused > 0 {
                return self.block(store, Block::KeysRefused);
            }
            if keys.invalid_replies > 0 {
                return self.block(store, Block::KeysInvalid);
            }
        }
        match device_key_batches::reserve_next(
            store,
            &bundle_selector(self.entry.local_id)?,
            &self.needed,
        ) {
            Ok(Some(query)) => {
                let id = query
                    .get_attr("id")
                    .and_then(|v| v.as_text())
                    .ok_or_else(|| invalid("directory query selector missing"))?
                    .to_owned();
                let old = self.row.clone();
                self.row.phase = Phase::Keys;
                self.row.device_count = self.plan.as_ref().expect("resolved plan").devices.len();
                self.row.key_queries += 1;
                self.save(store, &old)?;
                self.waiting = Waiting::Keys(id);
                self.deadline = deadline();
                Ok(Step::waiting(self, query))
            }
            Err(Error::InvalidInput { .. }) => self.block(store, Block::KeysInvalid),
            Err(error) => Err(error),
            Ok(None) => {
                let keys = device_key_batches::assembled(
                    store,
                    &bundle_selector(self.entry.local_id)?,
                    &self.needed,
                )?;
                let (body, content, reference_guard) =
                    match payload(store, &self.entry, self.own.as_ref()) {
                        Ok(value) => value,
                        Err(Error::InvalidInput { .. }) => {
                            return self.block(store, Block::ReferenceChanged);
                        }
                        Err(error) => return Err(error),
                    };
                let prepared = match self
                    .snapshot
                    .take()
                    .expect("staged snapshot")
                    .prepare_with_policy(store, account, keys.as_ref(), &body, policy)
                    .await
                {
                    Ok(value) => value,
                    Err(Error::IdentityChanged(_)) => {
                        return self.block(store, Block::PeerIdentityChanged);
                    }
                    Err(Error::InvalidInput { ref reason, .. })
                        if reason == group_fanout::COMPANION_AUTHORITY_CHANGED =>
                    {
                        return self.block(store, Block::CompanionAuthorityChanged);
                    }
                    Err(Error::InvalidInput { .. }) => {
                        return self.block(store, Block::CryptoPreparationRefused);
                    }
                    Err(error) => return Err(error),
                };
                // A group reaction carrying distributions is ACKed but never
                // delivered; sending each distribution first as a direct
                // message and the reaction without `<participants>` was
                // delivered and decrypted (observed live 2026-11-01, groups.md).
                let out_of_band =
                    content == groups::StanzaContent::Reaction && !prepared.targets().is_empty();
                let plan = self.plan.as_ref().expect("resolved plan");
                let wire_id = super::messaging::new_message_id()?;
                // APK 37.71 SendE2EMessageJob.A06/0y0.A00: per-device SKDM
                // participants, addressing_mode and the existing candidate V2 hash.
                let node = groups::sender_key_stanza(
                    &plan.chat,
                    &wire_id,
                    content,
                    Some(plan.addressing_mode),
                    Some(&plan.participant_hash),
                    if out_of_band { &[] } else { prepared.targets() },
                    prepared.ciphertext(),
                )?;
                let distributions = if out_of_band {
                    prepared
                        .targets()
                        .iter()
                        .map(|target| -> Result<Node> {
                            Ok(super::messaging::direct_message(
                                &target.device,
                                &super::messaging::new_message_id()?,
                                target.kind,
                                target.ciphertext.clone(),
                            )?)
                        })
                        .collect::<Result<Vec<_>>>()?
                } else {
                    Vec::new()
                };
                let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&node)?;
                let archive = format!("outbox-group-rotated:{}", self.entry.local_id);
                let mut reserved = self.row.clone();
                reserved.phase = Phase::Reserved;
                reserved.device_count = plan.devices.len();
                let result = store.write(|tx| {
                    let mut current = outbox()
                        .get_mut(tx, self.entry.local_id)?
                        .ok_or_else(|| invalid("original group intent missing"))?;
                    let held: Option<Row> =
                        tx.get_json(Table::Checkpoints, key(self.entry.local_id).as_bytes(), 1)?;
                    if current != self.entry || held.as_ref() != Some(&self.row) {
                        return Err(invalid("original group preparation changed"));
                    }
                    if let Some(guard) = &reference_guard {
                        guard.validate_in(tx)?;
                    }
                    prepared.reserve_in(tx, archive.as_bytes())?;
                    prepared.await_ack_in(tx, &wire_id)?;
                    super::send_versions::capture_in(tx, self.entry.local_id, &plan.devices)?;
                    current.wire_id = Some(MessageId::new(wire_id)?);
                    current.attempts.push(AttemptRecord {
                        index: 0,
                        prepared_at: Timestamp::now(),
                        epoch: Some(epoch),
                        payload: Some(wire.into()),
                        re_encryption: false,
                    });
                    outbox().set_progress(tx, &mut current, AttemptProgress::InFlight)?;
                    outbox().record_fact(
                        tx,
                        &mut current,
                        DeliveryFact {
                            kind: DeliveryFactKind::DeliveryUnknown,
                            device: None,
                            observed_at: Timestamp::now(),
                            server_timestamp: None,
                            code: None,
                        },
                    )?;
                    tx.put_json(
                        Table::Checkpoints,
                        key(self.entry.local_id).as_bytes(),
                        1,
                        &reserved,
                    )
                });
                if let Err(error) = result {
                    // Storage failures are not a local protocol refusal. Preserve
                    // the transaction error rather than claiming a saved block.
                    if let Error::InvalidInput { field, .. } = error {
                        let block = if field == "group_reference" {
                            Block::ReferenceChanged
                        } else {
                            Block::PreparationChanged
                        };
                        return self.block(store, block);
                    }
                    return Err(error);
                }
                self.row = reserved;
                self.next_distribution(distributions, node)
            }
        }
    }

    /// Send the next out-of-band distribution and wait for its ACK, or, when
    /// none remain, the reserved group stanza itself.
    fn next_distribution(mut self, mut rest: Vec<Node>, group: Node) -> Result<Step> {
        if rest.is_empty() {
            return Ok(Step {
                pending: None,
                node: Some(group),
            });
        }
        let next = rest.remove(0);
        let id = next
            .get_attr("id")
            .and_then(|v| v.as_text())
            .ok_or_else(|| invalid("distribution message id missing"))?
            .to_owned();
        self.waiting = Waiting::Distribution {
            id,
            rest,
            group: Box::new(group),
        };
        self.deadline = deadline();
        Ok(Step::waiting(self, next))
    }
}
