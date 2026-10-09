//! Normal group quote/reaction/keep attribution over APK 37.71 protobuf keys.
//! Caller keys do not authorize copying a different saved parent's plaintext.
use super::{groups::StanzaContent, quotes, reactions};
use crate::{
    Error, Result,
    binary::{Decoder, TokenDictionary},
    model::{
        jid::{Jid, Server},
        message::{Direction, MessageContent, MessageRef},
        time::Timestamp,
    },
    outbox::Outbox,
    profile::Limits,
    store::{ProtocolStore, Table},
};
fn invalid(reason: &'static str) -> Error {
    Error::invalid("group_reference", reason)
}
fn target(content: &MessageContent) -> Result<&MessageRef> {
    match content {
        MessageContent::Text {
            quoted: Some(target),
            style,
            ..
        } if !style.is_styled() => Ok(target),
        MessageContent::Reaction { target, .. } | MessageContent::KeepInChat { target, .. } => {
            Ok(target)
        }
        _ => Err(invalid(
            "expected an unstyled quoted text, reaction or keep",
        )),
    }
}
#[derive(Clone, PartialEq, Eq)]
struct Parent {
    sender: Option<Jid>,
    text: Option<String>,
}
fn parent(
    store: &ProtocolStore,
    chat: &Jid,
    target: &MessageRef,
    include_text: bool,
) -> Result<Parent> {
    store.read(|tx| {
        parent_in(
            chat,
            target,
            include_text,
            || Outbox::new("comparison-driver", Limits::default()).by_wire_id(tx, &target.key.id),
            |k| tx.get(Table::Checkpoints, k, 1),
            |k| tx.get_json(Table::Checkpoints, k, 1),
        )
    })
}
fn parent_in(
    chat: &Jid,
    target: &MessageRef,
    include_text: bool,
    outgoing: impl FnOnce() -> Result<Option<crate::outbox::OutboxEntry>>,
    mut get: impl FnMut(&[u8]) -> Result<Option<Vec<u8>>>,
    capture: impl FnOnce(&[u8]) -> Result<Option<Vec<u8>>>,
) -> Result<Parent> {
    if chat.server != Server::Group || target.key.chat.jid().to_non_ad() != chat.to_non_ad() {
        return Err(invalid("target belongs to another chat or is not a group"));
    }
    if target.key.direction == Direction::Outgoing {
        let entry = outgoing()?;
        if entry
            .as_ref()
            .is_some_and(|e| e.intent.chat.jid().to_non_ad() != chat.to_non_ad())
        {
            return Err(invalid("saved outgoing target belongs to another chat"));
        }
        return Ok(Parent {
            sender: None,
            text: entry
                .filter(|_| include_text)
                .and_then(|e| match e.intent.content {
                    MessageContent::Text { body, .. } => Some(body),
                    _ => None,
                }),
        });
    }
    let supplied = target
        .key
        .sender
        .as_ref()
        .ok_or_else(|| invalid("incoming group target requires participant"))?
        .to_jid();
    if !supplied.is_user_like() {
        return Err(invalid("participant must be a user identity"));
    }
    let node = crate::binary::Node::new("message")
        .attr("from", chat.clone())
        .attr("participant", supplied.clone())
        .attr("id", target.key.id.as_str());
    let selection = super::group_inbox::select(&node, &mut get)?;
    if selection.saved.is_none() && selection.legacy_collision {
        return Err(invalid("saved incoming target attribution differs"));
    }
    let saved = selection.saved;
    let Some(saved) = saved else {
        return Ok(Parent {
            sender: Some(supplied.to_non_ad()),
            text: None,
        });
    };
    // Existing legacy group rows are chat/id keyed. Decode their preserved
    // authenticated route before copying any body; do not silently attribute
    // a same-id record from another participant to the supplied key.
    let wire: Option<Vec<u8>> = if !saved["route"].is_null() {
        serde_json::from_value(saved["route"].clone()).ok()
    } else if let Ok(key) = serde_json::from_value::<Vec<u8>>(saved["capture"].clone()) {
        capture(&key)?
    } else {
        None
    };
    let wire = wire.ok_or_else(|| invalid("saved incoming target lacks authenticated route"))?;
    let limits = Limits::default();
    let raw = crate::binary::unpack_frame_payload(&wire, &limits)?;
    let node =
        Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits).decode_node()?;
    let (saved_chat, sender) = super::inbox::group_sender(&node)
        .ok_or_else(|| invalid("saved incoming target has no group sender"))?;
    if sender.agent != 0 {
        return Err(invalid(
            "saved target agent component is unsupported by MessageRef",
        ));
    }
    if saved_chat.to_non_ad() != chat.to_non_ad()
        || node.get_attr("id").and_then(|v| v.as_text()) != Some(target.key.id.as_str())
        || sender.to_non_ad() != supplied.to_non_ad()
    {
        return Err(invalid("saved incoming target attribution differs"));
    }
    let text = if include_text && saved["status"] == "decrypted" {
        serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone())
            .ok()
            .and_then(|p| quotes::decode(&p))
            .map(|q| q.text)
    } else {
        None
    };
    Ok(Parent {
        sender: Some(sender.to_non_ad()),
        text,
    })
}
/// Validate group target/chat attribution without writes, queries or crypto.
/// Unknown targets retain an explicit participant but no invented preview.
pub fn validate(store: &ProtocolStore, chat: &Jid, content: &MessageContent) -> Result<()> {
    let reference = target(content)?;
    if let MessageContent::Reaction { emoji, .. } = content {
        reactions::validate(chat, reference, emoji.as_deref())?;
    }
    let _ = parent(store, chat, reference, false)?;
    Ok(())
}
/// Encode the APK-derived bare Message for one frozen group sender identity.
/// Text previews use retained authenticated text only; media/control bodies are
/// never copied. Encoding creates no send reservation or persistent mutation.
pub fn payload(
    store: &ProtocolStore,
    chat: &Jid,
    own: &Jid,
    content: &MessageContent,
) -> Result<(Vec<u8>, StanzaContent<'static>)> {
    let (body, kind, _) = guarded_payload(store, chat, own, content, Timestamp::now())?;
    Ok((body, kind))
}

// Parent resolution is read in one snapshot, then checked again in the crypto /
// outbox reservation transaction. An await must not authorize a stale quote or
// a legacy same-id record that acquired a different authenticated author.
pub(crate) struct ReferenceGuard {
    chat: Jid,
    target: MessageRef,
    include_text: bool,
    parent: Parent,
}
impl ReferenceGuard {
    pub(crate) fn validate_in(&self, tx: &crate::store::WriteTx<'_>) -> Result<()> {
        let current = match parent_in(
            &self.chat,
            &self.target,
            self.include_text,
            || {
                Outbox::new("comparison-driver", Limits::default())
                    .by_wire_id_in_write(tx, &self.target.key.id)
            },
            |k| tx.get(Table::Checkpoints, k, 1),
            |k| tx.get_json(Table::Checkpoints, k, 1),
        ) {
            Ok(parent) => parent,
            Err(Error::InvalidInput { .. }) => return Err(invalid("reference_changed")),
            Err(error) => return Err(error),
        };
        if current != self.parent {
            return Err(invalid("reference_changed"));
        }
        Ok(())
    }
}
pub(crate) fn guarded_payload(
    store: &ProtocolStore,
    chat: &Jid,
    own: &Jid,
    content: &MessageContent,
    accepted_at: Timestamp,
) -> Result<(Vec<u8>, StanzaContent<'static>, ReferenceGuard)> {
    let reference = target(content)?;
    let resolved = parent(
        store,
        chat,
        reference,
        matches!(content, MessageContent::Text { .. }),
    )?;
    let guard = ReferenceGuard {
        chat: chat.clone(),
        target: reference.clone(),
        include_text: matches!(content, MessageContent::Text { .. }),
        parent: resolved.clone(),
    };
    let participant = match reference.key.direction {
        Direction::Incoming => resolved.sender.expect("validated incoming participant"),
        Direction::Outgoing => {
            if reference
                .key
                .sender
                .as_ref()
                .is_some_and(|d| d.to_jid().to_non_ad() != own.to_non_ad())
            {
                return Err(invalid(
                    "outgoing target participant differs from frozen own identity",
                ));
            }
            own.to_non_ad()
        }
    };
    if !participant.is_user_like() {
        return Err(invalid("participant must be a user identity"));
    }
    let encoded: Result<(Vec<u8>, StanzaContent<'static>)> = match content {
        MessageContent::Text { body, .. } => {
            let _ = super::group_send::text_payload(body)?;
            // 37.71 BVK contextInfo: stanzaId=1, participant=2, quotedMessage=3.
            Ok((
                quotes::reply_payload(
                    body,
                    reference.key.id.as_str(),
                    &participant.to_string(),
                    quotes::conversation(resolved.text.as_deref().unwrap_or("")),
                ),
                StanzaContent::Text,
            ))
        }
        MessageContent::Reaction { emoji, .. } => Ok((
            reactions::group_payload(
                chat,
                &participant,
                reference.key.id.as_str(),
                reference.key.direction == Direction::Outgoing,
                emoji.as_deref(),
                Timestamp::now().as_millis(),
            )?,
            StanzaContent::Reaction,
        )),
        MessageContent::KeepInChat { keep, .. } => Ok((
            super::chat_settings::group_keep_payload(
                chat,
                &participant,
                reference,
                *keep,
                accepted_at.as_millis(),
            )?,
            // APK 37.71 BoR.A02: bare field 51 falls through to type=text.
            StanzaContent::Text,
        )),
        _ => unreachable!("validated content"),
    };
    let (body, kind) = encoded?;
    Ok((body, kind, guard))
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::{
        binary::{Encoder, Node},
        model::{
            ids::{ChatId, DeviceAddress, MessageId},
            message::MessageKey,
        },
    };
    fn jid(s: &str) -> Jid {
        Jid::parse(s).unwrap()
    }
    pub(in crate::transport) fn reference(chat: &Jid, sender: &Jid, ours: bool) -> MessageRef {
        MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(chat).unwrap(),
                sender: if ours {
                    None
                } else {
                    Some(DeviceAddress::from_jid(sender).unwrap())
                },
                direction: if ours {
                    Direction::Outgoing
                } else {
                    Direction::Incoming
                },
                id: MessageId::new("ORIGINAL-TARGET").unwrap(),
            },
        }
    }
    pub(in crate::transport) fn save_parent(
        store: &ProtocolStore,
        chat: &Jid,
        sender: &Jid,
        payload: &[u8],
    ) -> Result<()> {
        let node = Node::new("message")
            .attr("from", chat.clone())
            .attr("participant", sender.clone())
            .attr("id", "ORIGINAL-TARGET")
            .attr("type", "text");
        let route = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&node)?;
        store.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                format!("received:{chat}:ORIGINAL-TARGET").as_bytes(),
                1,
                &serde_json::json!({"status":"decrypted","route":route,"protobuf":payload}),
            )
        })
    }
    #[test]
    fn keep_validation_and_encoding_preserve_consumed_parents_and_refuse_foreign_authors() {
        use prost::Message as _;
        let dir = tempfile::tempdir().unwrap();
        let store = super::super::group_fanout::tests::open(&dir.path().join("keep.redb"));
        let chat = jid("120363000000000001@g.us");
        let own = jid("111111111111111@lid");
        let author = jid("222222222222222:3@lid");
        save_parent(&store, &chat, &author, &[0x1a, 0]).unwrap();
        let saved_key = format!("received:{chat}:ORIGINAL-TARGET");
        store
            .write(|tx| {
                let mut saved: serde_json::Value = tx
                    .get_json(Table::Checkpoints, saved_key.as_bytes(), 1)?
                    .unwrap();
                saved["status"] = "consumed".into();
                saved.as_object_mut().unwrap().remove("protobuf");
                tx.put_json(Table::Checkpoints, saved_key.as_bytes(), 1, &saved)
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap();
        let head = store.journal_head().unwrap();
        for kept in [true, false] {
            let content = MessageContent::KeepInChat {
                target: reference(&chat, &author, false),
                keep: kept,
            };
            validate(&store, &chat, &content).unwrap();
            let (bytes, kind) = payload(&store, &chat, &own, &content).unwrap();
            assert!(matches!(kind, StanzaContent::Text));
            let envelope = super::super::chat_settings::Envelope::decode(bytes.as_slice()).unwrap();
            assert_eq!(
                envelope.keep_in_chat.unwrap().key.unwrap().participant,
                Some(author.to_non_ad().to_string())
            );
            let forged = MessageContent::KeepInChat {
                target: reference(&chat, &jid("333333333333333@lid"), false),
                keep: kept,
            };
            assert!(validate(&store, &chat, &forged).is_err());
            assert!(payload(&store, &chat, &own, &forged).is_err());
        }
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap()
        );
        assert_eq!(head, store.journal_head().unwrap());
    }
    #[test]
    fn reference_guard_rolls_back_reservations_after_parent_changes_during_crypto() {
        let dir = tempfile::tempdir().unwrap();
        let store =
            super::super::group_fanout::tests::open(&dir.path().join("reference-race.redb"));
        let chat = jid("120363000000000001@g.us");
        let own = jid("111111111111111@lid");
        let author = jid("222222222222222:3@lid");
        for case in [
            "quote_body",
            "quote_consumed",
            "reaction_author",
            "keep_author",
            "keep_malformed",
        ] {
            save_parent(
                &store,
                &chat,
                &author,
                &quotes::conversation("original body"),
            )
            .unwrap();
            let target = reference(&chat, &author, false);
            let content = if case.starts_with("quote") {
                MessageContent::Text {
                    body: "reply".into(),
                    quoted: Some(target),
                    style: Default::default(),
                }
            } else if case.starts_with("reaction") {
                MessageContent::Reaction {
                    target,
                    emoji: None,
                }
            } else {
                MessageContent::KeepInChat {
                    target,
                    keep: false,
                }
            };
            let (_, _, guard) =
                guarded_payload(&store, &chat, &own, &content, Timestamp(1)).unwrap();
            store.write(|tx| guard.validate_in(tx)).unwrap();
            if case == "quote_consumed" {
                store
                    .write(|tx| {
                        let key = format!("received:{chat}:ORIGINAL-TARGET");
                        let mut saved: serde_json::Value =
                            tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                        saved["status"] = "consumed".into();
                        saved.as_object_mut().unwrap().remove("protobuf");
                        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)
                    })
                    .unwrap();
            } else if case == "keep_malformed" {
                store
                    .write(|tx| {
                        tx.put(
                            Table::Checkpoints,
                            format!("received:{chat}:ORIGINAL-TARGET").as_bytes(),
                            1,
                            b"not-json",
                        )
                    })
                    .unwrap();
            } else {
                let sender = if case.ends_with("author") {
                    jid("333333333333333@lid")
                } else {
                    author.clone()
                };
                save_parent(
                    &store,
                    &chat,
                    &sender,
                    &quotes::conversation("replacement body"),
                )
                .unwrap();
            }
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap();
            let result = store.write(|tx| {
                tx.put(
                    Table::Checkpoints,
                    b"simulated-send-reservation",
                    1,
                    b"reserved",
                )?;
                guard.validate_in(tx)
            });
            assert!(
                matches!(result, Err(Error::InvalidInput { field: "group_reference", reason, .. }) if reason=="reference_changed")
            );
            assert_eq!(
                rows,
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                    .unwrap()
            );
        }
    }
    #[test]
    fn references_do_not_copy_a_foreign_sender_body_or_media_and_inspection_is_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = super::super::group_fanout::tests::open(&dir.path().join("references.redb"));
        let chat = jid("120363000000000001@g.us");
        let own = jid("111111111111111@lid");
        let sender = jid("222222222222222:3@lid");
        let r = reference(&chat, &sender, false);
        let content = MessageContent::Text {
            body: "reply".into(),
            quoted: Some(r.clone()),
            style: Default::default(),
        };
        save_parent(
            &store,
            &chat,
            &sender,
            &quotes::conversation("authenticated text"),
        )
        .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap();
        let head = store.journal_head().unwrap();
        let (encoded, _) = payload(&store, &chat, &own, &content).unwrap();
        let decoded = quotes::decode(&encoded).unwrap();
        assert_eq!(decoded.participant.as_deref(), Some("222222222222222@lid"));
        assert_eq!(
            quotes::decode(&decoded.quoted.unwrap()).unwrap().text,
            "authenticated text"
        );
        let mut forged = r.clone();
        forged.key.sender = Some(DeviceAddress::from_jid(&jid("333333333333333@lid")).unwrap());
        assert!(
            validate(
                &store,
                &chat,
                &MessageContent::Reaction {
                    target: forged,
                    emoji: Some("👍".into())
                }
            )
            .is_err()
        );
        let foreign = jid("120363000000000002@g.us");
        assert!(payload(&store, &foreign, &own, &content).is_err());
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap()
        );
        assert_eq!(head, store.journal_head().unwrap());
        let outgoing = reference(&chat, &own, true);
        let box_ = Outbox::new("original-parent", Limits::default());
        store
            .write(|tx| {
                let ticket = box_
                    .enqueue(
                        tx,
                        &crate::model::send::SendRequest {
                            client_request_id: crate::model::ids::ClientRequestId::new(
                                "foreign-parent",
                            )?,
                            chat: ChatId::from_jid(&foreign)?,
                            content: MessageContent::Text {
                                body: "foreign saved body".into(),
                                quoted: None,
                                style: Default::default(),
                            },
                        },
                    )?
                    .0;
                let mut entry = box_.get_mut(tx, ticket.local_id)?.unwrap();
                entry.wire_id = Some(outgoing.key.id.clone());
                box_.set_progress(
                    tx,
                    &mut entry,
                    crate::model::send::AttemptProgress::InFlight,
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap();
        assert!(
            payload(
                &store,
                &chat,
                &own,
                &MessageContent::Text {
                    body: "forged quote".into(),
                    quoted: Some(outgoing),
                    style: Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap()
        );
        // A Message{imageMessage} fixture is not a text quote. Its opaque
        // descriptor is never copied into quotedMessage, opened or consumed.
        save_parent(&store, &chat, &sender, &[0x1a, 0]).unwrap();
        let (encoded, _) = payload(&store, &chat, &own, &content).unwrap();
        assert_eq!(
            quotes::decode(&quotes::decode(&encoded).unwrap().quoted.unwrap())
                .unwrap()
                .text,
            ""
        );
        let mut missing = r.clone();
        missing.key.id = MessageId::new("UNRETAINED").unwrap();
        assert!(
            validate(
                &store,
                &chat,
                &MessageContent::Reaction {
                    target: missing,
                    emoji: None
                }
            )
            .is_ok()
        );
        let agent = Jid::new_ad(sender.user.clone(), 2, sender.device, Server::User).unwrap();
        save_parent(&store, &chat, &agent, &quotes::conversation("agent target")).unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap();
        assert!(
            matches!(validate(&store,&chat,&content),Err(Error::InvalidInput{reason,..}) if reason=="saved incoming target attribution differs")
        );
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap()
        );
    }
}
