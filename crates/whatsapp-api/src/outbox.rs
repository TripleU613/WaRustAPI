//! Durable outbox (docs/design/README.md section 12).
//!
//! `enqueue` commits an immutable intent and returns a [`SendTicket`]. It
//! claims nothing about server acceptance or delivery. Attempt progress and
//! delivery facts are recorded as separate, append-only facts. Enqueue itself
//! never transmits. The default profile stays offline; the explicit comparison
//! runtime can reserve and transmit supported intents from this same outbox.

use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::model::ids::{ClientRequestId, LocalMessageId, MessageId};
use crate::model::message::MessageContent;
use crate::model::send::{AttemptProgress, DeliveryFact, SendRequest, SendStatus, SendTicket};
use crate::model::time::Timestamp;
use crate::profile::Limits;
use crate::store::{ProtocolStore, ReadTx, Table, WriteTx};
use crate::sync::{SendStatusUpdate, SyncEvent};
use crate::transport::ConnectionEpoch;

const RECORD_V1: u16 = 1;

/// One prepared/transmitted attempt under a logical message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptRecord {
    /// Attempt index (0-based) under the logical message.
    pub index: u32,
    /// When the attempt was prepared.
    pub prepared_at: Timestamp,
    /// Connection epoch used for transmission, if it was written to a socket.
    pub epoch: Option<ConnectionEpoch>,
    /// Prepared E2EE payload (encrypted at rest by the store). Never Noise
    /// ciphertext: retransmission needs fresh transport encryption.
    pub payload: Option<Bytes>,
    /// Whether this attempt was a protocol-directed re-encryption.
    pub re_encryption: bool,
}

/// Durable outbox entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxEntry {
    /// Local identity.
    pub local_id: LocalMessageId,
    /// Caller idempotency key.
    pub client_request_id: ClientRequestId,
    /// Immutable intent.
    pub intent: SendRequest,
    /// SHA-256 over the semantic content of the intent.
    pub semantic_hash: [u8; 32],
    /// Wire ID once generated in the profile's format.
    pub wire_id: Option<MessageId>,
    /// Local attempt progress.
    pub progress: AttemptProgress,
    /// Attempts, in order.
    pub attempts: Vec<AttemptRecord>,
    /// Delivery facts, in observation order.
    pub facts: Vec<DeliveryFact>,
    /// Accept time.
    pub accepted_at: Timestamp,
    /// Last update time.
    pub updated_at: Timestamp,
}

impl OutboxEntry {
    /// The ticket for this entry.
    pub fn ticket(&self) -> SendTicket {
        SendTicket {
            local_id: self.local_id,
            client_request_id: self.client_request_id.clone(),
            wire_id: self.wire_id.clone(),
            accepted_at: self.accepted_at,
        }
    }

    /// The status view.
    pub fn status(&self) -> SendStatus {
        SendStatus {
            ticket: self.ticket(),
            progress: self.progress,
            facts: self.facts.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdempotencyRecord {
    local_id: LocalMessageId,
    semantic_hash: [u8; 32],
}

fn semantic_hash(req: &SendRequest) -> Result<[u8; 32]> {
    #[derive(Serialize)]
    struct Semantic<'a> {
        chat: &'a crate::model::ids::ChatId,
        content: &'a MessageContent,
    }
    let bytes = serde_json::to_vec(&Semantic {
        chat: &req.chat,
        content: &req.content,
    })
    .map_err(|e| Error::invalid("send_request", e.to_string()))?;
    Ok(Sha256::digest(&bytes).into())
}

fn idempotency_key(namespace: &str, id: &ClientRequestId) -> Vec<u8> {
    let mut k = Vec::with_capacity(2 + namespace.len() + id.as_str().len());
    k.extend_from_slice(&(namespace.len() as u16).to_be_bytes());
    k.extend_from_slice(namespace.as_bytes());
    k.extend_from_slice(id.as_str().as_bytes());
    k
}

fn validate(req: &SendRequest, limits: &Limits) -> Result<()> {
    match &req.content {
        MessageContent::Text { body, .. } => {
            if body.is_empty() {
                return Err(Error::invalid("content.body", "must not be empty"));
            }
            if body.len() > limits.max_text_len as usize {
                return Err(Error::ResourceExhausted {
                    resource: "text length",
                    limit: u64::from(limits.max_text_len),
                });
            }
            Ok(())
        }
        MessageContent::Reaction { target, emoji } => {
            crate::transport::reactions::validate(req.chat.jid(), target, emoji.as_deref())
        }
        MessageContent::Unsupported(_) => Err(Error::invalid(
            "content",
            "cannot send an Unsupported message form",
        )),
        _ => Ok(()),
    }
}

/// Outbox operations. All methods run inside a caller-provided transaction so
/// they commit atomically with related state.
#[derive(Debug, Clone)]
pub struct Outbox {
    namespace: String,
    limits: Limits,
}

impl Outbox {
    /// Create an outbox for a caller namespace (idempotency scope).
    pub fn new(namespace: impl Into<String>, limits: Limits) -> Self {
        Self {
            namespace: namespace.into(),
            limits,
        }
    }

    /// Durably accept an intent. Returns the ticket and whether it was newly
    /// created (`false` for an identical repeat).
    pub fn enqueue(&self, tx: &mut WriteTx<'_>, req: &SendRequest) -> Result<(SendTicket, bool)> {
        validate(req, &self.limits)?;
        let hash = semantic_hash(req)?;
        let ikey = idempotency_key(&self.namespace, &req.client_request_id);
        if let Some(existing) =
            tx.get_json::<IdempotencyRecord>(Table::Idempotency, &ikey, RECORD_V1)?
        {
            if existing.semantic_hash != hash {
                return Err(Error::IdempotencyConflict(req.client_request_id.clone()));
            }
            let entry: OutboxEntry = tx
                .get_json(Table::Outbox, &existing.local_id.0, RECORD_V1)?
                .ok_or_else(|| {
                    Error::Storage(crate::store::StoreError::Serialization(
                        "idempotency record without outbox entry".into(),
                    ))
                })?;
            return Ok((entry.ticket(), false));
        }
        let count = tx.count(Table::Outbox)?;
        if count >= u64::from(self.limits.max_outbox_entries) {
            return Err(Error::ResourceExhausted {
                resource: "outbox entries",
                limit: u64::from(self.limits.max_outbox_entries),
            });
        }
        let now = Timestamp::now();
        let entry = OutboxEntry {
            local_id: LocalMessageId::generate()?,
            client_request_id: req.client_request_id.clone(),
            intent: req.clone(),
            semantic_hash: hash,
            wire_id: None,
            progress: AttemptProgress::Queued,
            attempts: Vec::new(),
            facts: Vec::new(),
            accepted_at: now,
            updated_at: now,
        };
        tx.put_json(Table::Outbox, &entry.local_id.0, RECORD_V1, &entry)?;
        tx.put_json(
            Table::Idempotency,
            &ikey,
            RECORD_V1,
            &IdempotencyRecord {
                local_id: entry.local_id,
                semantic_hash: hash,
            },
        )?;
        tx.append_event(SyncEvent::SendStatusChanged(SendStatusUpdate {
            local_id: entry.local_id,
            progress: Some(AttemptProgress::Queued),
            fact: None,
        }))?;
        Ok((entry.ticket(), true))
    }

    /// Load an entry.
    pub fn get(&self, tx: &ReadTx<'_>, local_id: LocalMessageId) -> Result<Option<OutboxEntry>> {
        tx.get_json(Table::Outbox, &local_id.0, RECORD_V1)
    }

    /// Look up an immutable intent by its namespace-scoped request ID before
    /// performing preparation with external side effects. This is read-only;
    /// callers must still use `enqueue` for the final atomic conflict check.
    pub fn get_by_request_id(
        &self,
        tx: &ReadTx<'_>,
        id: &ClientRequestId,
    ) -> Result<Option<OutboxEntry>> {
        let Some(record) = tx.get_json::<IdempotencyRecord>(
            Table::Idempotency,
            &idempotency_key(&self.namespace, id),
            RECORD_V1,
        )?
        else {
            return Ok(None);
        };
        let entry = self.get(tx, record.local_id)?.ok_or_else(|| {
            Error::Storage(crate::store::StoreError::Serialization(
                "idempotency record without outbox entry".into(),
            ))
        })?;
        if entry.client_request_id != *id || entry.semantic_hash != record.semantic_hash {
            return Err(Error::Storage(crate::store::StoreError::Serialization(
                "inconsistent idempotency record".into(),
            )));
        }
        Ok(Some(entry))
    }

    /// Load an entry inside a write transaction.
    pub fn get_mut(
        &self,
        tx: &WriteTx<'_>,
        local_id: LocalMessageId,
    ) -> Result<Option<OutboxEntry>> {
        tx.get_json(Table::Outbox, &local_id.0, RECORD_V1)
    }

    /// Status view of a ticket.
    pub fn status(&self, tx: &ReadTx<'_>, local_id: LocalMessageId) -> Result<Option<SendStatus>> {
        Ok(self.get(tx, local_id)?.map(|e| e.status()))
    }

    /// All entries that still need work, in accept order (bounded).
    pub fn pending(&self, tx: &ReadTx<'_>, limit: usize) -> Result<Vec<OutboxEntry>> {
        self.pending_where(tx, limit, |_| Ok(true))
    }

    /// Filter before truncating, so durable reservations cannot hide eligible
    /// work. Scan one consistent snapshot in pages, retaining only `limit`
    /// candidates plus one page rather than the entire message history.
    pub(crate) fn pending_where(
        &self,
        tx: &ReadTx<'_>,
        limit: usize,
        mut eligible: impl FnMut(&OutboxEntry) -> Result<bool>,
    ) -> Result<Vec<OutboxEntry>> {
        let mut out = Vec::new();
        if limit == 0 {
            return Ok(out);
        }
        let mut cursor = None;
        loop {
            let page = tx.scan_after(Table::Outbox, RECORD_V1, cursor.as_deref(), 256)?;
            let done = page.len() < 256;
            cursor = page.last().map(|(key, _)| key.clone());
            for (_, bytes) in page {
                let entry: OutboxEntry = serde_json::from_slice(&bytes).map_err(|e| {
                    Error::Storage(crate::store::StoreError::Serialization(e.to_string()))
                })?;
                if matches!(
                    entry.progress,
                    AttemptProgress::Queued
                        | AttemptProgress::Prepared
                        | AttemptProgress::RetryScheduled
                ) && eligible(&entry)?
                {
                    out.push(entry);
                }
            }
            out.sort_by_key(|e| (e.accepted_at, e.local_id.0));
            out.truncate(limit);
            if done {
                break;
            }
        }
        Ok(out)
    }

    /// Cancel a queued intent. Returns `false` if the entry is unknown.
    /// Cancellation after transmission is rejected: that requires revocation.
    pub fn cancel(&self, tx: &mut WriteTx<'_>, local_id: LocalMessageId) -> Result<bool> {
        let Some(mut entry) = self.get_mut(tx, local_id)? else {
            return Ok(false);
        };
        match entry.progress {
            AttemptProgress::Queued
            | AttemptProgress::Prepared
            | AttemptProgress::RetryScheduled => {}
            AttemptProgress::Cancelled => return Ok(true),
            AttemptProgress::InFlight | AttemptProgress::Failed => {
                return Err(Error::invalid(
                    "local_id",
                    "cannot cancel after transmission; use revocation",
                ));
            }
        }
        self.set_progress(tx, &mut entry, AttemptProgress::Cancelled)?;
        Ok(true)
    }

    /// Record a progress change and emit the event.
    pub fn set_progress(
        &self,
        tx: &mut WriteTx<'_>,
        entry: &mut OutboxEntry,
        progress: AttemptProgress,
    ) -> Result<()> {
        entry.progress = progress;
        entry.updated_at = Timestamp::now();
        tx.put_json(Table::Outbox, &entry.local_id.0, RECORD_V1, entry)?;
        tx.append_event(SyncEvent::SendStatusChanged(SendStatusUpdate {
            local_id: entry.local_id,
            progress: Some(progress),
            fact: None,
        }))?;
        Ok(())
    }

    /// Record a delivery fact and emit the event.
    pub fn record_fact(
        &self,
        tx: &mut WriteTx<'_>,
        entry: &mut OutboxEntry,
        fact: DeliveryFact,
    ) -> Result<()> {
        entry.facts.push(fact.clone());
        entry.updated_at = Timestamp::now();
        tx.put_json(Table::Outbox, &entry.local_id.0, RECORD_V1, entry)?;
        tx.append_event(SyncEvent::SendStatusChanged(SendStatusUpdate {
            local_id: entry.local_id,
            progress: None,
            fact: Some(fact),
        }))?;
        Ok(())
    }

    /// Convenience: enqueue through the store in one transaction.
    pub async fn enqueue_async(
        &self,
        store: &Arc<ProtocolStore>,
        req: SendRequest,
    ) -> Result<(SendTicket, bool)> {
        let this = self.clone();
        store
            .run_blocking(move |s| s.write(|tx| this.enqueue(tx, &req)))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AccountId, ChatId};
    use crate::model::jid::Jid;
    use crate::model::send::DeliveryFactKind;
    use crate::store::{MasterKey, OpenOptions, StaticKeySource};

    fn store() -> (tempfile::TempDir, Arc<ProtocolStore>) {
        let dir = tempfile::tempdir().unwrap();
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([2u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, Arc::new(s))
    }

    fn req(id: &str, body: &str) -> SendRequest {
        SendRequest {
            client_request_id: ClientRequestId::new(id).unwrap(),
            chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: body.into(),
                quoted: None,
            },
        }
    }

    #[test]
    fn request_lookup_is_scoped_read_only_and_detects_missing_entries() {
        let (_d, s) = store();
        let ob = Outbox::new("test", Limits::default());
        let other = Outbox::new("other", Limits::default());
        let request = req("lookup", "original");
        assert!(
            s.read(|tx| ob.get_by_request_id(tx, &request.client_request_id))
                .unwrap()
                .is_none()
        );
        let (ticket, _) = s.write(|tx| ob.enqueue(tx, &request)).unwrap();
        let head = s.journal_head().unwrap();
        assert_eq!(
            s.read(|tx| ob.get_by_request_id(tx, &request.client_request_id))
                .unwrap()
                .unwrap()
                .intent,
            request
        );
        assert!(
            s.read(|tx| other.get_by_request_id(tx, &request.client_request_id))
                .unwrap()
                .is_none()
        );
        assert_eq!(s.journal_head().unwrap(), head);
        s.write(|tx| tx.delete(Table::Outbox, &ticket.local_id.0))
            .unwrap();
        assert!(
            s.read(|tx| ob.get_by_request_id(tx, &request.client_request_id))
                .is_err()
        );
    }

    #[test]
    fn enqueue_is_idempotent_and_detects_conflicts() {
        let (_d, s) = store();
        let ob = Outbox::new("test", Limits::default());
        let (t1, new1) = s.write(|tx| ob.enqueue(tx, &req("r1", "hi"))).unwrap();
        assert!(new1);
        assert_eq!(
            t1.wire_id, None,
            "wire id format is Unknown for this profile"
        );
        let (t2, new2) = s.write(|tx| ob.enqueue(tx, &req("r1", "hi"))).unwrap();
        assert!(!new2);
        assert_eq!(t1, t2);
        let err = s
            .write(|tx| ob.enqueue(tx, &req("r1", "different")))
            .unwrap_err();
        assert!(matches!(err, Error::IdempotencyConflict(_)));
        assert_eq!(s.read(|tx| tx.count(Table::Outbox)).unwrap(), 1);
        // One event for the single accepted intent.
        assert_eq!(s.journal_head().unwrap().0, 1);
        let st = s.read(|tx| ob.status(tx, t1.local_id)).unwrap().unwrap();
        assert_eq!(st.progress, AttemptProgress::Queued);
        assert!(!st.server_accepted());
    }

    #[test]
    fn validation_and_limits() {
        let (_d, s) = store();
        let ob = Outbox::new(
            "test",
            Limits {
                max_outbox_entries: 2,
                max_text_len: 5,
                ..Limits::default()
            },
        );
        assert!(matches!(
            s.write(|tx| ob.enqueue(tx, &req("e", ""))),
            Err(Error::InvalidInput { .. })
        ));
        assert!(matches!(
            s.write(|tx| ob.enqueue(tx, &req("l", "toolong"))),
            Err(Error::ResourceExhausted {
                resource: "text length",
                ..
            })
        ));
        s.write(|tx| ob.enqueue(tx, &req("a", "1"))).unwrap();
        s.write(|tx| ob.enqueue(tx, &req("b", "2"))).unwrap();
        assert!(matches!(
            s.write(|tx| ob.enqueue(tx, &req("c", "3"))),
            Err(Error::ResourceExhausted {
                resource: "outbox entries",
                ..
            })
        ));
        // A failed enqueue leaves no partial idempotency record.
        assert_eq!(s.read(|tx| tx.count(Table::Idempotency)).unwrap(), 2);
    }

    #[test]
    fn cancel_rules_and_pending_listing() {
        let (_d, s) = store();
        let ob = Outbox::new("test", Limits::default());
        let (t1, _) = s.write(|tx| ob.enqueue(tx, &req("r1", "a"))).unwrap();
        let (t2, _) = s.write(|tx| ob.enqueue(tx, &req("r2", "b"))).unwrap();
        let pending = s.read(|tx| ob.pending(tx, 10)).unwrap();
        assert_eq!(pending.len(), 2);
        assert!(s.write(|tx| ob.cancel(tx, t1.local_id)).unwrap());
        assert_eq!(s.read(|tx| ob.pending(tx, 10)).unwrap().len(), 1);
        // Simulate transmission of t2, then cancellation must be refused.
        s.write(|tx| {
            let mut e = ob.get_mut(tx, t2.local_id)?.unwrap();
            ob.set_progress(tx, &mut e, AttemptProgress::InFlight)?;
            ob.record_fact(
                tx,
                &mut e,
                DeliveryFact {
                    kind: DeliveryFactKind::DeliveryUnknown,
                    device: None,
                    observed_at: Timestamp::now(),
                    code: None,
                },
            )
        })
        .unwrap();
        assert!(s.write(|tx| ob.cancel(tx, t2.local_id)).is_err());
        let st = s.read(|tx| ob.status(tx, t2.local_id)).unwrap().unwrap();
        assert!(st.is_uncertain());
        assert!(
            !s.write(|tx| ob.cancel(tx, LocalMessageId([0; 16])))
                .unwrap()
        );
    }

    #[test]
    fn pending_reaches_work_beyond_completed_history_and_filters_before_limit() {
        let (_d, s) = store();
        let ob = Outbox::new("test", Limits::default());
        let (ticket, _) = s
            .write(|tx| ob.enqueue(tx, &req("template", "fixture")))
            .unwrap();
        let template = s.read(|tx| ob.get(tx, ticket.local_id)).unwrap().unwrap();
        let last = LocalMessageId(299_u128.to_be_bytes());
        s.write(|tx| {
            tx.delete(Table::Outbox, &ticket.local_id.0)?;
            for index in 0..300_u128 {
                let mut entry = template.clone();
                entry.local_id = LocalMessageId(index.to_be_bytes());
                entry.progress = if entry.local_id == last {
                    AttemptProgress::Queued
                } else {
                    AttemptProgress::Cancelled
                };
                tx.put_json(Table::Outbox, &entry.local_id.0, RECORD_V1, &entry)?;
            }
            Ok(())
        })
        .unwrap();
        let pending = s.read(|tx| ob.pending(tx, 1)).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].local_id, last);
        assert!(s.read(|tx| ob.pending(tx, 0)).unwrap().is_empty());
        s.write(|tx| {
            for index in 0..299_u128 {
                let id = LocalMessageId(index.to_be_bytes());
                let mut entry = ob.get_mut(tx, id)?.unwrap();
                ob.set_progress(tx, &mut entry, AttemptProgress::Queued)?;
            }
            Ok(())
        })
        .unwrap();
        let eligible = s
            .read(|tx| ob.pending_where(tx, 1, |entry| Ok(entry.local_id == last)))
            .unwrap();
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].local_id, last);
        // Same-time entries use stable physical identity order across pages.
        let pending = s.read(|tx| ob.pending(tx, 2)).unwrap();
        assert_eq!(
            pending
                .iter()
                .map(|entry| entry.local_id.0)
                .collect::<Vec<_>>(),
            vec![0_u128.to_be_bytes(), 1_u128.to_be_bytes()]
        );
    }
}
