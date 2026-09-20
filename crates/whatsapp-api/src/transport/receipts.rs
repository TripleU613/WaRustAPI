//! APK LX/ClU.A03 receipt facts; self receipts are not recipient delivery.
use crate::{
    Error, Result,
    binary::Node,
    model::jid::{Jid, Server},
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn decode(bytes: &[u8]) -> Result<Node> {
    let limits = crate::profile::Limits::default();
    let raw = crate::binary::unpack_frame_payload(bytes, &limits)?;
    Ok(crate::binary::Decoder::new(
        &raw,
        crate::binary::TokenDictionary::comparison_2_26_37_70(),
        &limits,
    )
    .decode_node()?)
}

/// Record successful display of the consumed image and durably queue `played`.
/// APK SendPlayedReceiptJobV2.A0G -> LX/0y0 (0x0c1e) -> LX/Cns.A09.
/// No action for merely downloaded media or historical unknown viewer outcomes.
pub fn queue_opened(store: &ProtocolStore, ticket: &str) -> Result<()> {
    store.write(|tx| {
        let mut cursor = None;
        let key = loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                return Err(Error::invalid(
                    "receipt",
                    "unknown consumed attachment ticket",
                ));
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            if let Some((key, _)) = rows.into_iter().find(|(key, _)| {
                key.starts_with(b"received:") && hex::encode(Sha256::digest(key)) == ticket
            }) {
                break key;
            }
        };
        let mut saved: serde_json::Value = tx
            .get_json(Table::Checkpoints, &key, 1)?
            .ok_or_else(|| Error::invalid("receipt", "inbox record disappeared"))?;
        if saved["status"] != "consumed" || saved["opened_receipt"] != "awaiting_viewer" {
            return Err(Error::invalid(
                "receipt",
                "image not awaiting viewer completion",
            ));
        }
        let incoming = decode(&saved_route(tx, &saved)?)?;
        let receipt = super::messaging::received_confirmation(&incoming)
            .filter(|n| n.description == "receipt")
            .ok_or_else(|| Error::invalid("receipt", "unsupported opened receipt routing"))?
            .attr("type", "played");
        let wire =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70())
                .encode(&receipt)?;
        let receipt_key = format!("receipt-outbox:{ticket}:played");
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, receipt_key.as_bytes(), 1)?
            .is_some()
        {
            return Err(Error::invalid("receipt", "opened receipt already queued"));
        }
        tx.put_json(
            Table::Checkpoints,
            receipt_key.as_bytes(),
            1,
            &serde_json::json!({"status":"pending","wire":wire}),
        )?;
        saved["opened_receipt"] = serde_json::json!("queued");
        tx.put_json(Table::Checkpoints, &key, 1, &saved)
    })
}

/// The original incoming wire node for a saved inbox row: the route copy when
/// present, otherwise the preserved raw capture.
fn saved_route(tx: &crate::store::WriteTx<'_>, saved: &serde_json::Value) -> Result<Vec<u8>> {
    if !saved["route"].is_null() {
        return serde_json::from_value(saved["route"].clone())
            .map_err(|_| Error::invalid("receipt", "invalid saved route"));
    }
    let capture: Vec<u8> = serde_json::from_value(saved["capture"].clone())
        .map_err(|_| Error::invalid("receipt", "missing saved route"))?;
    tx.get_json(Table::Checkpoints, &capture, 1)?
        .ok_or_else(|| Error::invalid("receipt", "capture unavailable"))
}

/// Durably queue explicit `read` receipts for authenticated incoming direct
/// messages, one receipt per message ID, all-or-nothing and idempotent.
///
/// Comparison `LX/Cns.A09` builds client receipts as `to`/`id`/`type` with
/// optional `participant`/`category` and no timestamp; `LX/CcH.<clinit>` lists
/// `read`, `read-self`, `played` and `played-self` as the client status-receipt
/// vocabulary, and `LX/ClU.A00` maps `read` to its own status code. Using that
/// builder with `type=read` is an inference from this vocabulary: the read
/// receipt job itself was not traced, so multi-ID `list` children are not
/// emitted. Nothing here sends automatically (`ReceiptPolicy::ExplicitOnly`).
pub fn queue_read(
    store: &ProtocolStore,
    receipt: &crate::model::receipt::Receipt,
) -> Result<usize> {
    use crate::model::receipt::ReceiptKind;
    if receipt.kind != ReceiptKind::Read {
        return Err(Error::invalid(
            "receipt",
            "only explicit read receipts can be queued",
        ));
    }
    if receipt.code.is_some() {
        return Err(Error::invalid(
            "receipt",
            "read receipts carry no error code",
        ));
    }
    let chat = receipt.chat.jid();
    if chat.server != Server::User || chat.device != 0 {
        return Err(Error::invalid(
            "receipt",
            "only primary direct chats are supported",
        ));
    }
    let mut ids: Vec<&str> = receipt.message_ids.iter().map(|id| id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() || ids.len() > 16 {
        return Err(Error::invalid(
            "receipt",
            "between one and sixteen message ids per receipt",
        ));
    }
    store.write(|tx| {
        let mut queued = 0;
        for id in &ids {
            let key = format!("received:{chat}:{id}");
            let Some(mut saved) =
                tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?
            else {
                return Err(Error::invalid(
                    "receipt",
                    "unknown incoming message for this chat",
                ));
            };
            if !matches!(saved["status"].as_str(), Some("decrypted" | "consumed")) {
                return Err(Error::invalid(
                    "receipt",
                    "incoming message is not authenticated locally",
                ));
            }
            let receipt_key = format!(
                "receipt-outbox:{}:read",
                hex::encode(Sha256::digest(key.as_bytes()))
            );
            if tx
                .get_json::<serde_json::Value>(Table::Checkpoints, receipt_key.as_bytes(), 1)?
                .is_some()
            {
                continue;
            }
            let incoming = decode(&saved_route(tx, &saved)?)?;
            let node = super::messaging::received_confirmation(&incoming)
                .filter(|n| n.description == "receipt")
                .ok_or_else(|| Error::invalid("receipt", "unsupported read receipt routing"))?
                .attr("type", "read");
            let wire = crate::binary::Encoder::new(
                crate::binary::TokenDictionary::comparison_2_26_37_70(),
            )
            .encode(&node)?;
            tx.put_json(
                Table::Checkpoints,
                receipt_key.as_bytes(),
                1,
                &serde_json::json!({"status":"pending","wire":wire}),
            )?;
            saved["read_receipt"] = serde_json::json!("queued");
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)?;
            queued += 1;
        }
        Ok(queued)
    })
}

/// One queued explicit receipt's outcome, redacted for operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptOutcome {
    /// `played` or `read`.
    pub kind: String,
    /// `pending`, `outcome_unknown`, `server_acknowledged` or `refused`.
    pub status: String,
    /// Wire message id the receipt refers to.
    pub message_id: String,
}
/// Every explicit receipt ever queued, in key order. No recipient is printed.
pub fn outcomes(store: &ProtocolStore) -> Result<Vec<ReceiptOutcome>> {
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                let Ok(name) = std::str::from_utf8(&key) else {
                    continue;
                };
                let Some(rest) = name.strip_prefix("receipt-outbox:") else {
                    continue;
                };
                let Some(row): Option<serde_json::Value> =
                    tx.get_json(Table::Checkpoints, &key, 1)?
                else {
                    continue;
                };
                let kind = rest.rsplit(':').next().unwrap_or("").to_owned();
                let message_id = row["wire"]
                    .as_array()
                    .and_then(|bytes| {
                        let wire: Vec<u8> = bytes
                            .iter()
                            .filter_map(|b| b.as_u64().map(|b| b as u8))
                            .collect();
                        decode(&wire).ok()
                    })
                    .and_then(|node| {
                        node.get_attr("id")
                            .and_then(|v| v.as_text())
                            .map(str::to_owned)
                    })
                    .unwrap_or_default();
                out.push(ReceiptOutcome {
                    kind,
                    status: row["status"].as_str().unwrap_or("").to_owned(),
                    message_id,
                });
            }
            if done {
                return Ok(out);
            }
        }
    })
}

/// Reserve at most sixteen pending explicit receipts before any socket write.
/// Unknown outcomes are excluded, including after a crash/reconnect.
pub fn reserve_pending(store: &ProtocolStore) -> Result<Vec<Node>> {
    store.write(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        while out.len() < 16 {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows
                .iter()
                .filter(|(key, _)| key.starts_with(b"receipt-outbox:"))
            {
                if out.len() == 16 {
                    break;
                }
                let Some(mut row): Option<serde_json::Value> =
                    tx.get_json(Table::Checkpoints, key, 1)?
                else {
                    continue;
                };
                if row["status"] != "pending" {
                    continue;
                }
                let wire: Vec<u8> = serde_json::from_value(row["wire"].clone())
                    .map_err(|_| Error::invalid("receipt", "invalid receipt intent"))?;
                out.push(decode(&wire)?);
                row["status"] = serde_json::json!("outcome_unknown");
                tx.put_json(Table::Checkpoints, key, 1, &row)?;
            }
        }
        Ok(out)
    })
}

/// Correlate an explicit receipt ACK by recipient, ID and type, not ID alone.
/// Only primary direct receipts are queued here. APK 1Qy.A02/CZH.equals also
/// bind participant, recipient and edit; those routes must not settle this queue.
pub fn record_ack(store: &ProtocolStore, node: &Node) -> Result<bool> {
    if node.description != "ack"
        || node.get_attr("class").and_then(|v| v.as_text()) != Some("receipt")
        || ["participant", "recipient", "edit"]
            .iter()
            .any(|name| node.get_attr(name).is_some())
    {
        return Ok(false);
    }
    store.write(|tx| {
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                return Ok(false);
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows
                .iter()
                .filter(|(key, _)| key.starts_with(b"receipt-outbox:"))
            {
                let Some(mut row): Option<serde_json::Value> =
                    tx.get_json(Table::Checkpoints, key, 1)?
                else {
                    continue;
                };
                if row["status"] != "outcome_unknown" {
                    continue;
                }
                let wire: Vec<u8> = serde_json::from_value(row["wire"].clone())
                    .map_err(|_| Error::invalid("receipt", "invalid intent"))?;
                let sent = decode(&wire)?;
                if sent.get_attr("id") != node.get_attr("id")
                    || sent.get_attr("type") != node.get_attr("type")
                    || jid(&sent, "to").is_none()
                    || jid(&sent, "to") != jid(node, "from")
                {
                    continue;
                }
                row["status"] = serde_json::json!(if node.get_attr("error").is_some() {
                    "refused"
                } else {
                    "server_acknowledged"
                });
                tx.put_json(Table::Checkpoints, key, 1, &row)?;
                return Ok(true);
            }
        }
    })
}

/// Independent facts: late delivery events never replace observed reads.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptFacts {
    /// Observed recipient delivery.
    pub delivered: bool,
    /// Observed recipient read.
    pub read: bool,
    /// Observed recipient playback/opening.
    pub played: bool,
}
impl ReceiptFacts {
    fn observe(&mut self, kind: &str) {
        match kind {
            "delivery" => self.delivered = true,
            "read" => self.read = true,
            "played" => self.played = true,
            _ => {}
        }
    }
}
pub(crate) fn jid(node: &Node, name: &str) -> Option<Jid> {
    let value = node.get_attr(name)?;
    value
        .as_jid()
        .cloned()
        .or_else(|| Jid::parse(value.as_text()?).ok())
}
fn ids(node: &Node) -> Result<Vec<String>> {
    let mut ids = std::collections::BTreeSet::new();
    let mut insert = |v: Option<&str>| -> Result<()> {
        let v = v
            .filter(|s| !s.is_empty() && s.len() <= 128)
            .ok_or_else(|| Error::invalid("receipt", "missing or invalid message id"))?;
        ids.insert(v.to_owned());
        if ids.len() > 257 {
            return Err(Error::invalid("receipt", "receipt batch exceeds bound"));
        }
        Ok(())
    };
    insert(node.get_attr("id").and_then(|v| v.as_text()))?;
    for list in node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "list")
    {
        if list.child_nodes().len() > 256 {
            return Err(Error::invalid("receipt", "receipt list exceeds bound"));
        }
        for item in list.child_nodes() {
            if item.description != "item" {
                return Err(Error::invalid("receipt", "unsupported receipt list child"));
            }
            insert(item.get_attr("id").and_then(|v| v.as_text()))?;
        }
    }
    Ok(ids.into_iter().collect())
}
/// Persist supported direct receipt facts, matching sends by peer AND wire ID.
/// Unknown/group/self receipts remain raw captures, not false delivery claims.
pub fn record(store: &ProtocolStore, node: &Node) -> Result<usize> {
    if node.description != "receipt"
        || node.get_attr("participant").is_some()
        || node.get_attr("recipient").is_some()
    {
        return Ok(0);
    }
    let kind = node
        .get_attr("type")
        .and_then(|v| v.as_text())
        .unwrap_or("delivery");
    if !matches!(kind, "delivery" | "read" | "played") {
        return Ok(0);
    }
    let Some(from) = jid(node, "from") else {
        return Ok(0);
    };
    if from.device != 0 {
        return Ok(0);
    }
    let peer = match from.server {
        Server::User => from.clone(),
        Server::Lid => {
            let Some(peer): Option<Jid> = store.read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("authenticated-lid:{from}").as_bytes(),
                    1,
                )
            })?
            else {
                return Ok(0);
            };
            peer
        }
        _ => return Ok(0),
    };
    let ids = ids(node)?;
    let records = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
    let outbox_records = store.read(|tx| tx.scan(Table::Outbox, 1, 10000))?;
    if outbox_records.len() == 10000 {
        return Err(Error::invalid("receipt", "outbox pagination required"));
    }
    if records.len() == 10000 {
        return Err(Error::invalid(
            "receipt",
            "checkpoint scan requires pagination",
        ));
    }
    let mut matched = 0;
    store.write(|tx| {
        for id in &ids {
            let key = format!(
                "receipt-facts:{}",
                hex::encode(Sha256::digest(format!("{peer}:{id}")))
            );
            let mut facts: ReceiptFacts = tx
                .get_json(Table::Checkpoints, key.as_bytes(), 1)?
                .unwrap_or_default();
            facts.observe(kind);
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &facts)?;
        }
        for (key, _) in &records {
            if !key.starts_with(b"message:") || !key.ends_with(b":send") {
                continue;
            }
            let Some(mut send): Option<serde_json::Value> =
                tx.get_json(Table::Checkpoints, key, 1)?
            else {
                continue;
            };
            if send["to"]
                .as_str()
                .map(|v| v.strip_prefix('+').unwrap_or(v))
                != Some(peer.user.as_str())
                || !send["wire_id"]
                    .as_str()
                    .is_some_and(|v| ids.iter().any(|id| id == v))
            {
                continue;
            }
            let mut facts: ReceiptFacts = if send["receipts"].is_null() {
                ReceiptFacts::default()
            } else {
                serde_json::from_value(send["receipts"].clone())
                    .map_err(|_| Error::invalid("receipt", "invalid saved facts"))?
            };
            facts.observe(kind);
            send["receipts"] = serde_json::to_value(facts)
                .map_err(|_| Error::invalid("receipt", "cannot encode facts"))?;
            tx.put_json(Table::Checkpoints, key, 1, &send)?;
            matched += 1;
        }
        let outbox =
            crate::outbox::Outbox::new("receipt-observer", crate::profile::Limits::default());
        let fact_kind = match kind {
            "read" => crate::model::send::DeliveryFactKind::Read,
            "played" => crate::model::send::DeliveryFactKind::Played,
            _ => crate::model::send::DeliveryFactKind::Delivered,
        };
        let device = crate::model::ids::DeviceAddress::from_jid(&peer)?;
        for (key, _) in &outbox_records {
            let Some(mut entry): Option<crate::outbox::OutboxEntry> =
                tx.get_json(Table::Outbox, key, 1)?
            else {
                continue;
            };
            if entry.intent.chat.jid() != &peer
                || !entry
                    .wire_id
                    .as_ref()
                    .is_some_and(|id| ids.iter().any(|v| v == id.as_str()))
            {
                continue;
            }
            if entry
                .facts
                .iter()
                .any(|fact| fact.kind == fact_kind && fact.device.as_ref() == Some(&device))
            {
                continue;
            }
            outbox.record_fact(
                tx,
                &mut entry,
                crate::model::send::DeliveryFact {
                    kind: fact_kind,
                    device: Some(device.clone()),
                    observed_at: crate::model::time::Timestamp::now(),
                    code: None,
                },
            )?;
            matched += 1;
        }
        Ok(())
    })?;
    Ok(matched)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(crate::store::OpenOptions {
            path: path.to_owned(),
            account_id: crate::model::ids::AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([7; 32]),
            )),
            create,
        })
        .unwrap()
    }
    #[test]
    fn opened_queue_survives_restart_and_never_replays_uncertain_receipts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("receipts.redb");
        let store = open(&path, true);
        let key = b"received:15555550100@s.whatsapp.net:fixture";
        let ticket = hex::encode(Sha256::digest(key));
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70())
                .encode(
                    &Node::new("message")
                        .attr("id", "fixture")
                        .attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap()),
                )
                .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key,
                    1,
                    &serde_json::json!({"status":"decrypted", "route":route}),
                )
            })
            .unwrap();
        assert!(queue_opened(&store, &ticket).is_err());
        store.write(|tx| tx.put_json(Table::Checkpoints, key, 1, &serde_json::json!({"status":"consumed", "opened_receipt":"awaiting_viewer", "route":route}))).unwrap();
        queue_opened(&store, &ticket).unwrap();
        assert!(queue_opened(&store, &ticket).is_err());
        drop(store);
        let store = open(&path, false);
        let pending = reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].get_attr("type").and_then(|v| v.as_text()),
            Some("played")
        );
        drop(store);
        let store = open(&path, false);
        assert!(reserve_pending(&store).unwrap().is_empty());
        let ack = Node::new("ack")
            .attr("id", "fixture")
            .attr("class", "receipt")
            .attr("type", "played");
        for field in ["participant", "recipient", "edit"] {
            assert!(
                !record_ack(
                    &store,
                    &ack.clone()
                        .attr("from", "15555550100@s.whatsapp.net")
                        .attr(field, "unexpected")
                )
                .unwrap()
            );
        }
        assert!(
            !record_ack(
                &store,
                &ack.clone().attr("from", "15555550101@s.whatsapp.net")
            )
            .unwrap()
        );
        assert!(
            !record_ack(
                &store,
                &Node::new("ack")
                    .attr("id", "fixture")
                    .attr("class", "receipt")
                    .attr("type", "read")
                    .attr("from", "15555550100@s.whatsapp.net")
            )
            .unwrap()
        );
        assert!(
            record_ack(
                &store,
                &ack.clone().attr("from", "15555550100@s.whatsapp.net")
            )
            .unwrap()
        );
        assert!(!record_ack(&store, &ack.attr("from", "15555550100@s.whatsapp.net")).unwrap());
        let saved: serde_json::Value = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("receipt-outbox:{ticket}:played").as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(saved["status"], "server_acknowledged");
    }
    #[test]
    fn explicit_read_receipts_queue_once_reserve_and_settle_by_type() {
        use crate::model::{
            ids::{ChatId, MessageId},
            receipt::{Receipt, ReceiptKind},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("read-receipts.redb");
        let store = open(&path, true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let encoder =
            || crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70());
        let route = |id: &str| {
            encoder()
                .encode(
                    &Node::new("message")
                        .attr("id", id)
                        .attr("from", peer.clone()),
                )
                .unwrap()
        };
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"received:15555550100@s.whatsapp.net:first",
                    1,
                    &serde_json::json!({"status":"decrypted", "route":route("first")}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    b"received:15555550100@s.whatsapp.net:second",
                    1,
                    &serde_json::json!({"status":"consumed", "route":route("second")}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    b"received:15555550100@s.whatsapp.net:captured",
                    1,
                    &serde_json::json!({"status":"captured", "route":route("captured")}),
                )
            })
            .unwrap();
        let receipt = |ids: &[&str], kind: ReceiptKind| Receipt {
            chat: ChatId::from_jid(&peer).unwrap(),
            message_ids: ids.iter().map(|id| MessageId::new(*id).unwrap()).collect(),
            from: None,
            kind,
            server_timestamp: None,
            code: None,
        };
        let rows = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    Ok(tx
                        .scan_after(Table::Checkpoints, 1, None, 1024)?
                        .into_iter()
                        .filter(|(key, _)| key.starts_with(b"receipt-outbox:"))
                        .count())
                })
                .unwrap()
        };

        // Refusals leave nothing queued: wrong kind, error code, group/device
        // chats, empty/oversized batches, unknown or unauthenticated messages.
        assert!(queue_read(&store, &receipt(&["first"], ReceiptKind::Delivered)).is_err());
        assert!(queue_read(&store, &receipt(&["first"], ReceiptKind::Played)).is_err());
        let mut coded = receipt(&["first"], ReceiptKind::Read);
        coded.code = Some(1);
        assert!(queue_read(&store, &coded).is_err());
        let mut other_chat = receipt(&["first"], ReceiptKind::Read);
        other_chat.chat =
            ChatId::from_jid(&Jid::parse("15555550101@s.whatsapp.net").unwrap()).unwrap();
        assert!(queue_read(&store, &other_chat).is_err());
        assert!(queue_read(&store, &receipt(&[], ReceiptKind::Read)).is_err());
        let many: Vec<String> = (0..17).map(|i| format!("id{i}")).collect();
        let many: Vec<&str> = many.iter().map(String::as_str).collect();
        assert!(queue_read(&store, &receipt(&many, ReceiptKind::Read)).is_err());
        assert!(queue_read(&store, &receipt(&["missing"], ReceiptKind::Read)).is_err());
        // Atomic: an unauthenticated member refuses the whole batch.
        assert!(queue_read(&store, &receipt(&["first", "captured"], ReceiptKind::Read)).is_err());
        assert_eq!(rows(&store), 0);

        // Queue once, then repeats and overlapping batches are idempotent.
        assert_eq!(
            queue_read(&store, &receipt(&["first", "first"], ReceiptKind::Read)).unwrap(),
            1
        );
        assert_eq!(
            queue_read(&store, &receipt(&["first"], ReceiptKind::Read)).unwrap(),
            0
        );
        assert_eq!(
            queue_read(&store, &receipt(&["second", "first"], ReceiptKind::Read)).unwrap(),
            1
        );
        assert_eq!(rows(&store), 2);
        let saved: serde_json::Value = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    b"received:15555550100@s.whatsapp.net:first",
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(
            saved["status"], "decrypted",
            "authenticated content is untouched"
        );
        assert_eq!(saved["read_receipt"], "queued");

        // Restart, then reserve: the wire is to/id/type only, no timestamp.
        drop(store);
        let store = open(&path, false);
        let pending = reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 2);
        for node in &pending {
            assert_eq!(node.description, "receipt");
            assert_eq!(jid(node, "to"), Some(peer.clone()));
            assert_eq!(
                node.get_attr("type").and_then(|v| v.as_text()),
                Some("read")
            );
            assert!(node.get_attr("t").is_none());
            assert!(node.get_attr("participant").is_none());
            assert!(
                !matches!(node.content, crate::binary::NodeContent::Children(_)),
                "no list children without evidence"
            );
        }
        let mut ids: Vec<&str> = pending
            .iter()
            .filter_map(|n| n.get_attr("id").and_then(|v| v.as_text()))
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, ["first", "second"]);
        drop(store);
        let store = open(&path, false);
        assert!(
            reserve_pending(&store).unwrap().is_empty(),
            "uncertain writes never replay"
        );
        assert_eq!(
            queue_read(&store, &receipt(&["first"], ReceiptKind::Read)).unwrap(),
            0
        );

        // Acknowledgements settle by peer, id and exact type.
        let ack = |id: &str, kind: &str, from: &str| {
            Node::new("ack")
                .attr("id", id)
                .attr("class", "receipt")
                .attr("type", kind)
                .attr("from", from)
        };
        assert!(
            !record_ack(
                &store,
                &ack("first", "played", "15555550100@s.whatsapp.net")
            )
            .unwrap()
        );
        assert!(!record_ack(&store, &ack("first", "read", "15555550101@s.whatsapp.net")).unwrap());
        assert!(record_ack(&store, &ack("first", "read", "15555550100@s.whatsapp.net")).unwrap());
        assert!(!record_ack(&store, &ack("first", "read", "15555550100@s.whatsapp.net")).unwrap());
        assert!(
            record_ack(
                &store,
                &ack("second", "read", "15555550100@s.whatsapp.net").attr("error", "479")
            )
            .unwrap()
        );
        let status = |id: &str| -> serde_json::Value {
            let key = format!(
                "receipt-outbox:{}:read",
                hex::encode(Sha256::digest(
                    format!("received:15555550100@s.whatsapp.net:{id}").as_bytes()
                ))
            );
            store
                .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap()["status"]
                .clone()
        };
        assert_eq!(status("first"), "server_acknowledged");
        assert_eq!(status("second"), "refused");
    }

    #[test]
    fn opened_receipts_page_history_and_keep_reservations_atomic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("receipt-pages.redb");
        let store = open(&path, true);
        let key = b"received:15555550100@s.whatsapp.net:fixture";
        let ticket = hex::encode(Sha256::digest(key));
        let encoder =
            || crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70());
        let route = encoder()
            .encode(
                &Node::new("message")
                    .attr("id", "fixture")
                    .attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            )
            .unwrap();
        store.write(|tx| {
            for i in 0..10_000 {
                tx.put(Table::Checkpoints, format!("a-history:{i:05}").as_bytes(), 1, b"unrelated")?;
            }
            tx.put_json(Table::Checkpoints, key, 1, &serde_json::json!({"status":"consumed", "opened_receipt":"awaiting_viewer", "route":route}))?;
            // Completed history must not starve later pending receipts.
            for i in 0..300 {
                tx.put_json(Table::Checkpoints, format!("receipt-outbox:000{i:03}").as_bytes(), 1, &serde_json::json!({"status":"server_acknowledged"}))?;
            }
            for i in 0..17 {
                let wire = encoder().encode(&Node::new("receipt").attr("id", format!("other-{i}")).attr("to", "15555550100@s.whatsapp.net").attr("type", "played"))?;
                tx.put_json(Table::Checkpoints, format!("receipt-outbox:zzz{i:03}").as_bytes(), 1, &serde_json::json!({"status":"pending", "wire":wire}))?;
            }
            Ok(())
        }).unwrap();
        queue_opened(&store, &ticket).unwrap();
        assert_eq!(reserve_pending(&store).unwrap().len(), 16);
        drop(store);
        let store = open(&path, false);
        assert_eq!(reserve_pending(&store).unwrap().len(), 2);
        assert!(reserve_pending(&store).unwrap().is_empty());
        let ack = Node::new("ack")
            .attr("class", "receipt")
            .attr("type", "played")
            .attr("id", "fixture")
            .attr("from", "15555550100@s.whatsapp.net");
        assert!(record_ack(&store, &ack).unwrap());
        assert!(!record_ack(&store, &ack).unwrap());
        // A malformed pending row after a valid one rolls the entire batch back.
        let first = b"receipt-outbox:zzz000";
        let last = b"receipt-outbox:zzzz-invalid";
        store
            .write(|tx| {
                let mut row: serde_json::Value =
                    tx.get_json(Table::Checkpoints, first, 1)?.unwrap();
                row["status"] = serde_json::json!("pending");
                tx.put_json(Table::Checkpoints, first, 1, &row)?;
                tx.put_json(
                    Table::Checkpoints,
                    last,
                    1,
                    &serde_json::json!({"status":"pending", "wire":"invalid"}),
                )
            })
            .unwrap();
        assert!(reserve_pending(&store).is_err());
        let row: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, first, 1))
            .unwrap()
            .unwrap();
        assert_eq!(row["status"], "pending");
    }
    #[test]
    fn receipt_matching_requires_peer_and_ignores_self_receipts() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("facts.redb"), true);
        let key = b"message:fixture:send";
        store.write(|tx|tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"to":"+15555550100","wire_id":"fixture","status":"server_acknowledged"}))).unwrap();
        let node = Node::new("receipt").attr("id", "fixture");
        assert_eq!(
            record(
                &store,
                &node.clone().attr("from", "15555550101@s.whatsapp.net")
            )
            .unwrap(),
            0
        );
        assert_eq!(
            record(
                &store,
                &node
                    .clone()
                    .attr("from", "15555550100@s.whatsapp.net")
                    .attr("type", "read-self")
            )
            .unwrap(),
            0
        );
        assert_eq!(
            record(
                &store,
                &node
                    .clone()
                    .attr("from", "15555550100@s.whatsapp.net")
                    .attr("type", "read")
            )
            .unwrap(),
            1
        );
        assert_eq!(
            record(&store, &node.attr("from", "15555550100@s.whatsapp.net")).unwrap(),
            1
        );
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key, 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["status"], "server_acknowledged");
        assert_eq!(
            saved["receipts"],
            serde_json::json!({"read":true,"delivered":true,"played":false})
        );
    }
    #[test]
    fn normal_outbox_receipts_are_peer_scoped_and_idempotent() {
        use crate::{
            model::{
                ids::{ChatId, ClientRequestId, MessageId},
                message::MessageContent,
                send::{DeliveryFactKind, SendRequest},
            },
            outbox::Outbox,
        };
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("outbox.redb"), true);
        let outbox = Outbox::new("test", crate::profile::Limits::default());
        let request = SendRequest {
            client_request_id: ClientRequestId::new("fixture").unwrap(),
            chat: ChatId::from_jid(&Jid::parse("15555550100@s.whatsapp.net").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "offline fixture".into(),
                quoted: None,
            },
        };
        let ticket = store
            .write(|tx| {
                let (ticket, _) = outbox.enqueue(tx, &request)?;
                let mut entry = outbox.get_mut(tx, ticket.local_id)?.unwrap();
                entry.wire_id = Some(MessageId::new("fixture")?);
                outbox.set_progress(
                    tx,
                    &mut entry,
                    crate::model::send::AttemptProgress::InFlight,
                )?;
                Ok(ticket)
            })
            .unwrap();
        let receipt = Node::new("receipt")
            .attr("id", "fixture")
            .attr("type", "read");
        assert_eq!(
            record(
                &store,
                &receipt.clone().attr("from", "15555550101@s.whatsapp.net")
            )
            .unwrap(),
            0
        );
        let receipt = receipt.attr("from", "15555550100@s.whatsapp.net");
        assert_eq!(record(&store, &receipt).unwrap(), 1);
        assert_eq!(record(&store, &receipt).unwrap(), 0);
        let status = store
            .read(|tx| outbox.status(tx, ticket.local_id))
            .unwrap()
            .unwrap();
        assert_eq!(status.facts.len(), 1);
        assert_eq!(status.facts[0].kind, DeliveryFactKind::Read);
        assert!(!status.server_accepted());
    }
    #[test]
    fn independent_facts_never_downgrade_or_invent_events() {
        let mut facts = ReceiptFacts::default();
        facts.observe("played");
        facts.observe("delivery");
        facts.observe("read");
        facts.observe("read-self");
        assert_eq!(
            facts,
            ReceiptFacts {
                delivered: true,
                read: true,
                played: true
            }
        );
        let mut read = ReceiptFacts::default();
        read.observe("read");
        assert!(!read.delivered && !read.played && read.read);
    }
    #[test]
    fn batch_ids_are_bounded_and_deduplicated() {
        let node = Node::new("receipt")
            .attr("id", "a")
            .children(vec![Node::new("list").children(vec![
                Node::new("item").attr("id", "b"),
                Node::new("item").attr("id", "a"),
            ])]);
        assert_eq!(ids(&node).unwrap(), ["a", "b"]);
        assert!(ids(&Node::new("receipt")).is_err());
        assert!(
            ids(&Node::new("receipt").attr("id", "a").children(vec![
                Node::new("list").children(vec![Node::new("item").attr("id", "b"); 257])
            ]))
            .is_err()
        );
    }
}
