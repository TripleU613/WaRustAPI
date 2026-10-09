//! Normal group quote/reaction attribution over APK 37.71 BZa/BWI/BVK/BZF.
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
        MessageContent::Reaction { target, .. } => Ok(target),
        _ => Err(invalid("expected an unstyled quoted text or reaction")),
    }
}
struct Parent {
    sender: Option<Jid>,
    text: Option<String>,
}
fn parent(store: &ProtocolStore, chat: &Jid, target: &MessageRef) -> Result<Parent> {
    if chat.server != Server::Group || target.key.chat.jid().to_non_ad() != chat.to_non_ad() {
        return Err(invalid("target belongs to another chat or is not a group"));
    }
    if target.key.direction == Direction::Outgoing {
        let entry = store.read(|tx| {
            Outbox::new("comparison-driver", Limits::default()).by_wire_id(tx, &target.key.id)
        })?;
        if entry
            .as_ref()
            .is_some_and(|e| e.intent.chat.jid().to_non_ad() != chat.to_non_ad())
        {
            return Err(invalid("saved outgoing target belongs to another chat"));
        }
        return Ok(Parent {
            sender: None,
            text: entry.and_then(|e| match e.intent.content {
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
    let key = format!("received:{}:{}", chat.to_non_ad(), target.key.id);
    let saved: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?;
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
    } else if let Ok(capture) = serde_json::from_value::<Vec<u8>>(saved["capture"].clone()) {
        store.read(|tx| tx.get_json(Table::Checkpoints, &capture, 1))?
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
    let text = if saved["status"] == "decrypted" {
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
    let _ = parent(store, chat, reference)?;
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
    let reference = target(content)?;
    let resolved = parent(store, chat, reference)?;
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
    match content {
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
        _ => unreachable!("validated content"),
    }
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
            matches!(validate(&store,&chat,&content),Err(Error::InvalidInput{reason,..}) if reason=="saved target agent component is unsupported by MessageRef")
        );
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap()
        );
    }
}
