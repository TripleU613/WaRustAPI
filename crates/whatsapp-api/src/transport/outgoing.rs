//! Explicit opt-in normal outbox adapter for comparison primary text/uploaded media.
//! One key query at a time; no application retries or replay of uncertain sends.
use crate::{
    Error, Result,
    binary::{Encoder, Node, TokenDictionary},
    model::{
        ids::{LocalMessageId, MessageId},
        jid::Server,
        message::MessageContent,
        send::{AttemptProgress, DeliveryFact, DeliveryFactKind},
        time::Timestamp,
    },
    outbox::{AttemptRecord, Outbox, OutboxEntry},
    profile::Limits,
    session::AccountRecord,
    store::{ProtocolStore, Table},
    transport::ConnectionEpoch,
};
use zeroize::Zeroizing;

fn outbox() -> Outbox {
    Outbox::new("comparison-driver", Limits::default())
}
fn query_key(id: LocalMessageId) -> String {
    format!("outbox-key-query:{id}")
}

/// A pending query is connection-local; its reservation survives cancellation.
pub struct Pending {
    local_id: LocalMessageId,
    query_id: String,
    /// Bounded response deadline; expiry never causes a retry.
    pub deadline: tokio::time::Instant,
}
impl std::fmt::Debug for Pending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pending")
            .field("deadline", &self.deadline)
            .finish_non_exhaustive()
    }
}

/// Reserve one eligible plain direct text intent before sending its key query.
pub fn begin(store: &ProtocolStore) -> Result<Option<(Pending, Node)>> {
    begin_selected(store, None)
}
fn unreserved_queued(tx: &crate::store::ReadTx<'_>, entry: &OutboxEntry) -> Result<bool> {
    Ok(entry.progress == AttemptProgress::Queued
        && tx
            .get_json::<serde_json::Value>(
                Table::Checkpoints,
                query_key(entry.local_id).as_bytes(),
                1,
            )?
            .is_none())
}
/// Queued intents an outbox-enabled comparison session would attempt next, in
/// accept order across every idempotency namespace: `Queued` progress and no
/// durable key-query reservation. Read-only; `begin_selected` applies the same
/// filter before claiming, then fails unsupported content locally.
pub fn eligible(store: &ProtocolStore, limit: usize) -> Result<Vec<OutboxEntry>> {
    store.read(|tx| outbox().pending_where(tx, limit, |entry| unreserved_queued(tx, entry)))
}
/// Restrict transmission to one explicitly selected durable intent when set.
pub fn begin_selected(
    store: &ProtocolStore,
    ticket: Option<LocalMessageId>,
) -> Result<Option<(Pending, Node)>> {
    let entries = if let Some(id) = ticket {
        store.read(|tx| outbox().get(tx, id))?.into_iter().collect()
    } else {
        eligible(store, 256)?
    };
    for entry in entries {
        if entry.progress != AttemptProgress::Queued {
            continue;
        }
        let peer = entry.intent.chat.jid();
        let supported = peer.server == Server::User
            && peer.device == 0
            && !peer.user.is_empty()
            && match &entry.intent.content {
                MessageContent::Text { body, quoted: None } => {
                    !body.is_empty() && body.len() <= 4096
                }
                MessageContent::Media {
                    reference,
                    caption: None,
                    file_name: None,
                } => {
                    super::outgoing_media::resolve(store, &entry.client_request_id, peer, reference)
                        .is_ok()
                }
                MessageContent::Reaction { target, emoji } => {
                    super::reactions::validate(peer, target, emoji.as_deref()).is_ok()
                }
                _ => false,
            };
        let reserved = store
            .read(|tx| {
                tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    query_key(entry.local_id).as_bytes(),
                    1,
                )
            })?
            .is_some();
        if reserved {
            continue;
        }
        super::send_path::claim(
            store,
            &entry.client_request_id,
            super::send_path::SendPath::Outbox,
        )?;
        let query_id = super::messaging::new_message_id()?;
        let won = store.write(|tx| {
            let Some(mut current) = outbox().get_mut(tx, entry.local_id)? else {
                return Ok(false);
            };
            if current.progress != AttemptProgress::Queued
                || tx
                    .get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        query_key(entry.local_id).as_bytes(),
                        1,
                    )?
                    .is_some()
            {
                return Ok(false);
            }
            if !supported {
                outbox().set_progress(tx, &mut current, AttemptProgress::Failed)?;
                outbox().record_fact(
                    tx,
                    &mut current,
                    DeliveryFact {
                        kind: DeliveryFactKind::Failed,
                        device: None,
                        observed_at: Timestamp::now(),
                        code: None,
                    },
                )?;
                return Ok(false);
            }
            tx.put_json(
                Table::Checkpoints,
                query_key(entry.local_id).as_bytes(),
                1,
                &serde_json::json!({"status":"outcome_unknown","query_id":query_id}),
            )?;
            Ok(true)
        })?;
        if won {
            return Ok(Some((
                Pending {
                    local_id: entry.local_id,
                    query_id: query_id.clone(),
                    deadline: tokio::time::Instant::now() + std::time::Duration::from_secs(30),
                },
                super::messaging::key_request(peer, &query_id),
            )));
        }
    }
    Ok(None)
}

impl Pending {
    /// Only the exact query response can produce a prepared send.
    pub fn matches(&self, node: &Node) -> bool {
        node.description == "iq"
            && node.get_attr("id").and_then(|v| v.as_text()) == Some(self.query_id.as_str())
    }
    /// Prepare E2EE and atomically reserve ratchet + intent before any write.
    pub async fn prepare(
        self,
        store: &ProtocolStore,
        account: &AccountRecord,
        node: &Node,
        epoch: ConnectionEpoch,
    ) -> Result<Option<Node>> {
        if !self.matches(node) {
            return Err(Error::invalid("outbox", "mismatched key response"));
        }
        let entry = store
            .read(|tx| outbox().get(tx, self.local_id))?
            .ok_or_else(|| Error::invalid("outbox", "missing intent"))?;
        if entry.progress != AttemptProgress::Queued {
            return Ok(None);
        }
        if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
            store.write(|tx| {
                let mut current = outbox()
                    .get_mut(tx, self.local_id)?
                    .ok_or_else(|| Error::invalid("outbox", "missing intent"))?;
                if current.progress == AttemptProgress::Queued {
                    outbox().set_progress(tx, &mut current, AttemptProgress::Failed)?;
                }
                tx.put_json(
                    Table::Checkpoints,
                    query_key(self.local_id).as_bytes(),
                    1,
                    &serde_json::json!({"status":"refused","query_id":self.query_id}),
                )
            })?;
            return Ok(None);
        }
        let peer = entry.intent.chat.jid();
        let bundle = crate::signal::direct::prekey_bundle(node, peer)?;
        let peer_key = format!("comparison-direct:{peer}");
        let (session, identity): (Option<Vec<u8>>, Option<Vec<u8>>) = store.read(|tx| {
            Ok((
                tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
                tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
            ))
        })?;
        let session = session.map(Zeroizing::new);
        let media = match &entry.intent.content {
            MessageContent::Media {
                reference,
                caption: None,
                file_name: None,
            } => Some(super::outgoing_media::resolve(
                store,
                &entry.client_request_id,
                peer,
                reference,
            )?),
            MessageContent::Text { quoted: None, .. } | MessageContent::Reaction { .. } => None,
            _ => return Err(Error::invalid("outbox", "unsupported content")),
        };
        let reaction = match &entry.intent.content {
            MessageContent::Reaction { target, emoji } => Some(super::reactions::payload(
                peer,
                target,
                emoji.as_deref(),
                Timestamp::now().as_millis(),
            )?),
            _ => None,
        };
        let prepared = if let Some(payload) = &reaction {
            crate::signal::direct::prepare_payload(
                account,
                peer,
                &bundle,
                session.as_deref().map(|v| v.as_slice()),
                identity.as_deref(),
                payload,
            )
            .await?
        } else if let Some((payload, _)) = &media {
            crate::signal::direct::prepare_payload(
                account,
                peer,
                &bundle,
                session.as_deref().map(|v| v.as_slice()),
                identity.as_deref(),
                payload,
            )
            .await?
        } else {
            let MessageContent::Text { body, .. } = &entry.intent.content else {
                unreachable!()
            };
            crate::signal::direct::prepare(
                account,
                peer,
                &bundle,
                session.as_deref().map(|v| v.as_slice()),
                identity.as_deref(),
                body,
            )
            .await?
        };
        let wire_id = super::messaging::new_message_id()?;
        let stanza = if let Some((_, kind)) = &media {
            super::messaging::direct_media(
                peer,
                &wire_id,
                prepared.kind,
                prepared.ciphertext,
                kind,
            )?
        } else if reaction.is_some() {
            super::messaging::direct_typed(
                peer,
                &wire_id,
                prepared.kind,
                prepared.ciphertext,
                "reaction",
            )?
        } else {
            super::messaging::direct_message(peer, &wire_id, prepared.kind, prepared.ciphertext)?
        };
        let wire = Encoder::new(TokenDictionary::comparison_2_26_37_70()).encode(&stanza)?;
        store.write(|tx| {
            let mut current = outbox()
                .get_mut(tx, self.local_id)?
                .ok_or_else(|| Error::invalid("outbox", "missing intent"))?;
            let old_session: Option<Vec<u8>> =
                tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
            let old_identity: Option<Vec<u8>> =
                tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
            if current != entry
                || old_session.as_deref() != session.as_deref().map(|v| v.as_slice())
                || old_identity != identity
            {
                return Err(Error::invalid(
                    "outbox",
                    "intent or peer session changed before reservation",
                ));
            }
            current.wire_id = Some(MessageId::new(wire_id)?);
            current.attempts.push(AttemptRecord {
                index: 0,
                prepared_at: Timestamp::now(),
                epoch: Some(epoch),
                payload: Some(wire.into()),
                re_encryption: false,
            });
            tx.put_json(
                Table::SignalSessions,
                peer_key.as_bytes(),
                1,
                &*prepared.session,
            )?;
            tx.put_json(
                Table::PeerIdentities,
                peer_key.as_bytes(),
                1,
                &prepared.identity,
            )?;
            outbox().set_progress(tx, &mut current, AttemptProgress::InFlight)?;
            // Reservation precedes socket write: even a crash just before write
            // remains uncertain, never falsely retried or called delivered.
            outbox().record_fact(
                tx,
                &mut current,
                DeliveryFact {
                    kind: DeliveryFactKind::DeliveryUnknown,
                    device: None,
                    observed_at: Timestamp::now(),
                    code: None,
                },
            )?;
            tx.put_json(
                Table::Checkpoints,
                query_key(self.local_id).as_bytes(),
                1,
                &serde_json::json!({"status":"answered","query_id":self.query_id}),
            )
        })?;
        Ok(Some(stanza))
    }
}

/// Correlate message ACKs to an existing in-flight intent by ID and recipient.
pub fn acknowledge(store: &ProtocolStore, node: &Node) -> Result<bool> {
    if node.description != "ack"
        || node.get_attr("class").and_then(|v| v.as_text()) != Some("message")
    {
        return Ok(false);
    }
    let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(false);
    };
    let Some(from) = super::receipts::jid(node, "from") else {
        return Ok(false);
    };
    let rows = store.read(|tx| tx.scan(Table::Outbox, 1, 10000))?;
    if rows.len() == 10000 {
        return Err(Error::invalid("outbox", "pagination required"));
    }
    store.write(|tx| {
        for (key, _) in rows {
            let Some(mut entry): Option<OutboxEntry> = tx.get_json(Table::Outbox, &key, 1)? else {
                continue;
            };
            if entry.progress != AttemptProgress::InFlight
                || entry.intent.chat.jid() != &from
                || entry.wire_id.as_ref().map(|v| v.as_str()) != Some(id)
            {
                continue;
            }
            let kind = if node.get_attr("error").is_some() {
                DeliveryFactKind::Failed
            } else {
                DeliveryFactKind::ServerAccepted
            };
            if entry.facts.iter().any(|v| {
                matches!(
                    v.kind,
                    DeliveryFactKind::ServerAccepted | DeliveryFactKind::Failed
                )
            }) {
                return Ok(false);
            }
            if kind == DeliveryFactKind::Failed {
                outbox().set_progress(tx, &mut entry, AttemptProgress::Failed)?;
            }
            let code = node
                .get_attr("error")
                .and_then(|v| v.as_text())
                .and_then(|v| v.parse::<u32>().ok());
            outbox().record_fact(
                tx,
                &mut entry,
                DeliveryFact {
                    kind,
                    device: None,
                    observed_at: Timestamp::now(),
                    code,
                },
            )?;
            return Ok(true);
        }
        Ok(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId, ClientRequestId},
            jid::Jid,
            send::SendRequest,
        },
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn enqueue(store: &ProtocolStore) -> LocalMessageId {
        store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: ClientRequestId::new("test").unwrap(),
                            chat: ChatId::from_jid(
                                &Jid::parse("15555550100@s.whatsapp.net").unwrap(),
                            )
                            .unwrap(),
                            content: MessageContent::Text {
                                body: "offline fixture".into(),
                                quoted: None,
                            },
                        },
                    )?
                    .0
                    .local_id)
            })
            .unwrap()
    }
    #[test]
    fn eligible_spans_namespaces_and_excludes_reserved_or_cancelled_intents() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("eligible.redb"));
        let first = enqueue(&store);
        let request = |id: &str| SendRequest {
            client_request_id: ClientRequestId::new(id).unwrap(),
            chat: ChatId::from_jid(&Jid::parse("15555550100@s.whatsapp.net").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "offline fixture".into(),
                quoted: None,
            },
        };
        let other_namespace = Outbox::new("default", Limits::default());
        let second = store
            .write(|tx| {
                Ok(other_namespace
                    .enqueue(tx, &request("public-client"))?
                    .0
                    .local_id)
            })
            .unwrap();
        let cancelled = store
            .write(|tx| {
                let id = other_namespace
                    .enqueue(tx, &request("cancelled"))?
                    .0
                    .local_id;
                other_namespace.cancel(tx, id)?;
                Ok(id)
            })
            .unwrap();
        let ids =
            |entries: Vec<OutboxEntry>| entries.into_iter().map(|e| e.local_id).collect::<Vec<_>>();
        assert_eq!(ids(eligible(&store, 10).unwrap()), vec![first, second]);
        assert_eq!(ids(eligible(&store, 1).unwrap()), vec![first]);
        assert!(eligible(&store, 0).unwrap().is_empty());
        // Reserving the first key query removes it without touching the other.
        assert!(begin_selected(&store, Some(first)).unwrap().is_some());
        assert_eq!(ids(eligible(&store, 10).unwrap()), vec![second]);
        assert!(!ids(eligible(&store, 10).unwrap()).contains(&cancelled));
        assert_eq!(
            store
                .read(|tx| outbox().get(tx, second))
                .unwrap()
                .unwrap()
                .progress,
            AttemptProgress::Queued,
            "listing is read-only"
        );
    }
    #[test]
    fn reactions_are_eligible_for_their_own_chat_only() {
        use crate::model::message::{Direction, MessageKey, MessageRef};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("reaction-outbox.redb"));
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let other = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let reaction = |chat: &Jid, target_chat: &Jid, request: &str| SendRequest {
            client_request_id: ClientRequestId::new(request).unwrap(),
            chat: ChatId::from_jid(chat).unwrap(),
            content: MessageContent::Reaction {
                target: MessageRef {
                    key: MessageKey {
                        chat: ChatId::from_jid(target_chat).unwrap(),
                        sender: None,
                        direction: Direction::Outgoing,
                        id: MessageId::new("ABCDEF0123456789ABCDEF0123456789").unwrap(),
                    },
                },
                emoji: Some("👍".into()),
            },
        };
        // The outbox refuses a cross-chat target before anything is queued.
        assert!(
            store
                .write(|tx| outbox().enqueue(tx, &reaction(&peer, &other, "bad")))
                .is_err()
        );
        let id = store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(tx, &reaction(&peer, &peer, "good"))?
                    .0
                    .local_id)
            })
            .unwrap();
        let (pending, query) = begin_selected(&store, Some(id)).unwrap().unwrap();
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert!(pending.matches(&Node::new("iq").attr("id", pending.query_id.as_str())));
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, query_key(id).as_bytes(), 1))
                .unwrap()
                .is_some(),
            "key query reserved for the reaction"
        );
        let typed = super::super::messaging::direct_typed(&peer, "ID1", "msg", vec![1], "reaction")
            .unwrap();
        assert_eq!(
            typed.get_attr("type").and_then(|v| v.as_text()),
            Some("reaction")
        );
        assert!(
            super::super::messaging::direct_typed(&peer, "ID1", "msg", vec![1], "sticker").is_err()
        );
    }
    #[tokio::test]
    async fn oversized_text_fails_before_query_and_uploaded_media_uses_one_reservation() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("media-outbox.redb"));
        let id = enqueue(&store);
        store
            .write(|tx| {
                let mut entry = outbox().get_mut(tx, id)?.unwrap();
                entry.intent.content = MessageContent::Text {
                    body: "x".repeat(4097),
                    quoted: None,
                };
                tx.put_json(Table::Outbox, &id.0, 1, &entry)
            })
            .unwrap();
        assert!(begin_selected(&store, Some(id)).unwrap().is_none());
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, query_key(id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .read(|tx| outbox().get(tx, id))
                .unwrap()
                .unwrap()
                .progress,
            AttemptProgress::Failed
        );
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let request_id = ClientRequestId::new("uploaded-media").unwrap();
        store.write(|tx| tx.put_json(Table::Checkpoints, b"message:uploaded-media:upload", 1,
            &serde_json::json!({"status":"uploaded","to":peer.user,"hash":"synthetic","protobuf":[26,0]}))).unwrap();
        let reference =
            super::super::outgoing_media::reference(&store, &request_id, &peer).unwrap();
        let media_id = store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: request_id,
                            chat: ChatId::from_jid(&peer)?,
                            content: MessageContent::Media {
                                reference,
                                caption: None,
                                file_name: None,
                            },
                        },
                    )?
                    .0
                    .local_id)
            })
            .unwrap();
        assert!(begin_selected(&store, Some(media_id)).unwrap().is_some());
        assert!(begin_selected(&store, Some(media_id)).unwrap().is_none());
    }

    #[tokio::test]
    async fn query_reservation_survives_restart_without_repeating_requests() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("outbox.redb");
        let store = open(&path);
        let local_id = enqueue(&store);
        let (pending, query) = begin(&store).unwrap().unwrap();
        assert_eq!(pending.local_id, local_id);
        assert!(pending.matches(
            &Node::new("iq").attr("id", query.get_attr("id").unwrap().as_text().unwrap())
        ));
        assert!(!pending.matches(&Node::new("iq").attr("id", "wrong")));
        assert!(begin(&store).unwrap().is_none());
        drop(store);
        let store = open(&path);
        assert!(begin(&store).unwrap().is_none());
        let entry = store
            .read(|tx| outbox().get(tx, local_id))
            .unwrap()
            .unwrap();
        assert!(entry.attempts.is_empty());
        assert!(entry.wire_id.is_none());
    }
    #[tokio::test]
    async fn selecting_one_ticket_never_reserves_other_queued_work() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("selected.redb"));
        let first = enqueue(&store);
        let second = store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: ClientRequestId::new("second")?,
                            chat: ChatId::from_jid(&Jid::parse("15555550100@s.whatsapp.net")?)?,
                            content: MessageContent::Text {
                                body: "second fixture".into(),
                                quoted: None,
                            },
                        },
                    )?
                    .0
                    .local_id)
            })
            .unwrap();
        assert_eq!(
            begin_selected(&store, Some(second))
                .unwrap()
                .unwrap()
                .0
                .local_id,
            second
        );
        assert!(begin_selected(&store, Some(second)).unwrap().is_none());
        let first_reservation: Option<serde_json::Value> = store
            .read(|tx| tx.get_json(Table::Checkpoints, query_key(first).as_bytes(), 1))
            .unwrap();
        assert!(first_reservation.is_none());
        assert_eq!(
            store
                .read(|tx| outbox().get(tx, first))
                .unwrap()
                .unwrap()
                .progress,
            AttemptProgress::Queued
        );
    }

    #[tokio::test]
    async fn reserved_queries_do_not_starve_later_work_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reserved-history.redb");
        let store = open(&path);
        let initial = enqueue(&store);
        let template = store.read(|tx| outbox().get(tx, initial)).unwrap().unwrap();
        let last = LocalMessageId(299_u128.to_be_bytes());
        store
            .write(|tx| {
                tx.delete(Table::Outbox, &initial.0)?;
                for index in 0..300_u128 {
                    let mut entry = template.clone();
                    entry.local_id = LocalMessageId(index.to_be_bytes());
                    tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)?;
                    if entry.local_id != last {
                        tx.put_json(
                            Table::Checkpoints,
                            query_key(entry.local_id).as_bytes(),
                            1,
                            &serde_json::json!({"status":"outcome_unknown"}),
                        )?;
                    }
                }
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = open(&path);
        let (pending, _) = begin(&store).unwrap().unwrap();
        assert_eq!(pending.local_id, last);
        assert!(begin(&store).unwrap().is_none());
        let previous: serde_json::Value = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    query_key(LocalMessageId(0_u128.to_be_bytes())).as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(previous["status"], "outcome_unknown");
    }
    #[tokio::test]
    async fn acknowledgements_do_not_replay_or_cross_recipient_boundaries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("outbox.redb");
        let store = open(&path);
        let local_id = enqueue(&store);
        store
            .write(|tx| {
                let mut entry = outbox().get_mut(tx, local_id)?.unwrap();
                entry.wire_id = Some(MessageId::new("fixture")?);
                outbox().set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                outbox().record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::DeliveryUnknown,
                        device: None,
                        observed_at: Timestamp::now(),
                        code: None,
                    },
                )
            })
            .unwrap();
        // A process can die after the durable send reservation and before the
        // socket write or its ACK. Reopening must not make that intent eligible.
        drop(store);
        let store = open(&path);
        assert!(begin_selected(&store, Some(local_id)).unwrap().is_none());
        assert!(begin(&store).unwrap().is_none());
        assert!(
            store
                .read(|tx| outbox().status(tx, local_id))
                .unwrap()
                .unwrap()
                .is_uncertain()
        );
        let ack = Node::new("ack")
            .attr("class", "message")
            .attr("id", "fixture");
        assert!(
            !acknowledge(
                &store,
                &ack.clone().attr("from", "15555550101@s.whatsapp.net")
            )
            .unwrap()
        );
        let ack = ack.attr("from", "15555550100@s.whatsapp.net");
        assert!(acknowledge(&store, &ack).unwrap());
        assert!(!acknowledge(&store, &ack).unwrap());
        drop(store);
        let store = open(&path);
        assert!(
            store
                .read(|tx| outbox().pending(tx, 10))
                .unwrap()
                .is_empty()
        );
        let status = store
            .read(|tx| outbox().status(tx, local_id))
            .unwrap()
            .unwrap();
        assert!(status.server_accepted());
        assert!(!status.is_uncertain());
        assert_eq!(status.facts.len(), 2);
    }
}
