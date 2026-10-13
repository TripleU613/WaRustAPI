//! APK 37.71 Cmk.A03/DDH keep group chat, message IDs and participant device
//! separate. Receipt-send ACK reservations and native receipt cleanup are distinct.
use super::receipts::{ReceiptFacts, ids, jid, receipt_kind};
use crate::{
    Error, Result,
    binary::Node,
    model::{
        ids::{ClientRequestId, DeviceAddress, MessageId},
        jid::{Jid, JidKind, Server},
        send::{DeliveryFact, DeliveryFactKind},
        time::Timestamp,
    },
    outbox::{Outbox, OutboxEntry},
    profile::Limits,
    store::{ProtocolStore, Table, WriteTx},
};
use serde::{Deserialize, Serialize};

fn invalid(reason: &'static str) -> Error {
    Error::invalid("group_receipt", reason)
}
fn plain(jid: &Jid) -> Jid {
    let mut jid = jid.clone();
    jid.kind = JidKind::Plain;
    jid
}
fn group(jid: &Jid) -> bool {
    !jid.user.is_empty()
        && jid.server == Server::Group
        && jid.agent == 0
        && jid.device == 0
        && matches!(jid.kind, JidKind::Plain | JidKind::AgentDevice)
}
fn actor(jid: &Jid) -> bool {
    !jid.user.is_empty()
        && matches!(jid.server, Server::User | Server::Lid)
        && jid.agent == 0
        && matches!(jid.kind, JidKind::Plain | JidKind::AgentDevice)
}
fn decode_wire(bytes: &[u8]) -> Result<Node> {
    let limits = Limits::default();
    let raw = crate::binary::unpack_frame_payload(bytes, &limits)
        .map_err(|_| invalid("invalid saved group receipt wire"))?;
    crate::binary::Decoder::new(
        &raw,
        crate::binary::TokenDictionary::comparison_2_26_37_71(),
        &limits,
    )
    .decode_node()
    .map_err(|_| invalid("invalid saved group receipt wire"))
}
fn check_wire(node: &Node, chat: &Jid, id: &str) -> Result<()> {
    if node.description != "message"
        || node.get_attr("id").and_then(|v| v.as_text()) != Some(id)
        || !jid(node, "to").is_some_and(|to| group(&to) && plain(&to) == plain(chat))
    {
        return Err(invalid("saved group receipt wire does not match intent"));
    }
    Ok(())
}
fn validate_targets(targets: Vec<Jid>) -> Result<Vec<Jid>> {
    if targets.len() > Limits::default().max_children as usize || targets.iter().any(|j| !actor(j))
    {
        return Err(invalid("invalid saved group receipt targets"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for target in &targets {
        if !seen.insert(plain(target)) {
            return Err(invalid("duplicate saved group receipt target"));
        }
    }
    Ok(targets)
}
fn wire_targets(node: &Node) -> Result<Option<Vec<Jid>>> {
    let mut lists = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "participants");
    let Some(list) = lists.next() else {
        return Ok(None);
    };
    if lists.next().is_some() {
        return Err(invalid("duplicate saved group receipt target list"));
    }
    let targets = list
        .child_nodes()
        .iter()
        .map(|n| {
            if n.description != "to" {
                return Err(invalid("invalid saved group receipt target"));
            }
            jid(n, "jid").ok_or_else(|| invalid("invalid saved group receipt target"))
        })
        .collect::<Result<Vec<_>>>()?;
    if targets.is_empty() {
        return Ok(None);
    }
    validate_targets(targets).map(Some)
}
fn saved_targets(saved: &serde_json::Value, chat: &Jid, id: &str) -> Result<Option<Vec<Jid>>> {
    let Some(wire) = saved.get("wire") else {
        return Ok(None);
    };
    let wire: Vec<u8> = serde_json::from_value(wire.clone())
        .map_err(|_| invalid("invalid saved group receipt wire"))?;
    let node = decode_wire(&wire)?;
    check_wire(&node, chat, id)?;
    let embedded = wire_targets(&node)?;
    if let Some(value) = saved.get("recipient_devices") {
        let targets: Vec<Jid> = serde_json::from_value(value.clone())
            .map_err(|_| invalid("invalid saved group receipt targets"))?;
        let targets = validate_targets(targets)?;
        if embedded.as_ref().is_some_and(|list| {
            list.iter()
                .map(plain)
                .collect::<std::collections::BTreeSet<_>>()
                != targets.iter().map(plain).collect()
        }) {
            return Err(invalid("saved group receipt target lists disagree"));
        }
        return Ok(Some(targets));
    }
    Ok(embedded)
}
fn entry_targets(
    tx: &WriteTx<'_>,
    entry: &OutboxEntry,
    chat: &Jid,
    id: &str,
) -> Result<Option<Vec<Jid>>> {
    // The original prepared envelope is durable; mutable membership is not
    // evidence of which devices this particular send addressed.
    let Some(wire) = entry
        .attempts
        .first()
        .and_then(|attempt| attempt.payload.as_ref())
    else {
        return Ok(None);
    };
    let node = decode_wire(wire)?;
    check_wire(&node, chat, id)?;
    let embedded = wire_targets(&node)?;
    // The participants list distributes sender keys, not necessarily the full
    // group target set. APK 37.71 0Z5.A0D captures every actual receipt device;
    // our original reservation records all plan.devices in send_versions.
    if let Some(targets) = super::send_versions::targets_in(tx, entry.local_id)? {
        if embedded.as_ref().is_some_and(|list| {
            list.iter()
                .any(|target| !targets.iter().any(|held| plain(held) == plain(target)))
        }) {
            return Err(invalid(
                "saved sender-key targets exceed original device plan",
            ));
        }
        return Ok(Some(targets));
    }
    // Legacy sends have no full target instrumentation. Keep only their known
    // original wire recipients; mutable current membership cannot fill the gap.
    Ok(embedded)
}
fn targeted(tx: &WriteTx<'_>, from: &Jid, targets: &[Jid]) -> Result<bool> {
    for target in targets {
        if from.device == target.device
            && super::jid_map::authenticated_direct_chat(
                tx,
                &from.to_non_ad(),
                &target.to_non_ad(),
            )?
        {
            return Ok(true);
        }
    }
    Ok(false)
}
#[derive(Clone, Serialize, Deserialize)]
struct DeviceReceipt {
    device: DeviceAddress,
    facts: ReceiptFacts,
}
fn saved_facts(saved: &serde_json::Value) -> Result<Vec<DeviceReceipt>> {
    match saved.get("device_receipts") {
        None => Ok(Vec::new()),
        Some(value) => {
            let rows: Vec<DeviceReceipt> = serde_json::from_value(value.clone())
                .map_err(|_| invalid("invalid saved group receipt facts"))?;
            if rows.len() > Limits::default().max_children as usize
                || rows.iter().any(|r| !actor(&r.device.to_jid()))
            {
                return Err(invalid("invalid saved group receipt facts"));
            }
            let mut seen = std::collections::HashSet::new();
            if rows.iter().any(|r| !seen.insert(r.device.clone())) {
                return Err(invalid("duplicate saved group receipt facts"));
            }
            Ok(rows)
        }
    }
}

/// Record supported group participant facts against original addressed devices.
/// Call only on an authenticated transport receipt. No send or ACK is performed.
pub fn record(store: &ProtocolStore, node: &Node) -> Result<usize> {
    if !jid(node, "from").is_some_and(|from| group(&from)) {
        return Ok(0);
    }
    super::receipts::record(store, node)
}

/// A group receipt's wire content, checked without the store.
pub(super) struct Target<'a> {
    kind: &'a str,
    chat: Jid,
    from: Jid,
    ids: Vec<String>,
    wire_ids: Vec<MessageId>,
    device: DeviceAddress,
}

/// `None` for a receipt this path skips; `Err` for content it refuses.
/// Shared by [`record_in`] and `receipts::validate`, so content that passes
/// validation cannot be refused inside the write transaction.
pub(super) fn target(node: &Node) -> Result<Option<Target<'_>>> {
    if node.description != "receipt" || super::receipts::recipient(node).is_some() {
        return Ok(None);
    }
    let Some(kind) = receipt_kind(node) else {
        return Ok(None);
    };
    let (Some(chat), Some(from)) = (jid(node, "from"), jid(node, "participant")) else {
        return Ok(None);
    };
    if !group(&chat) || !actor(&from) {
        return Ok(None);
    }
    let ids = ids(node)?;
    let device = DeviceAddress::from_jid(&from)?;
    let wire_ids = ids
        .iter()
        .map(|id| {
            MessageId::new(id.clone()).map_err(|_| invalid("invalid group receipt message id"))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(Target {
        kind,
        chat,
        from,
        ids,
        wire_ids,
        device,
    }))
}

pub(super) fn record_in(tx: &mut WriteTx<'_>, node: &Node) -> Result<usize> {
    let Some(Target {
        kind,
        chat,
        from,
        ids,
        wire_ids,
        device,
    }) = target(node)?
    else {
        return Ok(0);
    };
    let outbox = Outbox::new("group-receipt-observer", Limits::default());
    let mut matched = 0;
    let fact_kind = match kind {
        "read" => DeliveryFactKind::Read,
        "played" => DeliveryFactKind::Played,
        _ => DeliveryFactKind::Delivered,
    };
    for (id, wire_id) in ids.iter().zip(&wire_ids) {
        if let Some(mut entry) = outbox.by_wire_id_in_write(tx, wire_id)? {
            if plain(entry.intent.chat.jid()) != plain(&chat) {
                continue;
            }
            let Some(targets) = entry_targets(tx, &entry, &chat, id)? else {
                continue;
            };
            if !targeted(tx, &from, &targets)? {
                continue;
            }
            super::send_versions::observe_in(tx, entry.local_id, &from, node)?;
            if !entry
                .facts
                .iter()
                .any(|f| f.kind == fact_kind && f.device.as_ref() == Some(&device))
            {
                outbox.record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: fact_kind,
                        device: Some(device.clone()),
                        observed_at: Timestamp::now(),
                        server_timestamp: super::receipts::server_timestamp(node),
                        code: None,
                    },
                )?;
                matched += 1;
            }
        }
    }
    let mut seen_legacy_ids = std::collections::HashSet::new();
    let mut cursor = None;
    loop {
        let page = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if page.is_empty() {
            break;
        }
        cursor = page.last().map(|(k, _)| k.clone());
        for (key, _) in page {
            if !key.starts_with(b"message:") || !key.ends_with(b":send") {
                continue;
            }
            let Some(mut saved): Option<serde_json::Value> =
                tx.get_json(Table::Checkpoints, &key, 1)?
            else {
                continue;
            };
            let Some(id) = saved["wire_id"]
                .as_str()
                .filter(|id| ids.iter().any(|v| v == id))
            else {
                continue;
            };
            let Some(raw_chat) = saved["chat"].as_str() else {
                continue;
            };
            let saved_chat =
                Jid::parse(raw_chat).map_err(|_| invalid("invalid saved group receipt chat"))?;
            if !group(&saved_chat) || plain(&saved_chat) != plain(&chat) {
                continue;
            }
            if !seen_legacy_ids.insert(id.to_owned()) {
                return Err(invalid("ambiguous saved group receipt wire id"));
            }
            let Some(targets) = saved_targets(&saved, &chat, id)? else {
                continue;
            };
            if !targeted(tx, &from, &targets)? {
                continue;
            }
            let mut facts = saved_facts(&saved)?;
            let index = match facts.iter().position(|r| r.device == device) {
                Some(index) => index,
                None => {
                    if facts.len() >= Limits::default().max_children as usize {
                        return Err(invalid("saved group receipt fact limit reached"));
                    }
                    facts.push(DeviceReceipt {
                        device: device.clone(),
                        facts: ReceiptFacts::default(),
                    });
                    facts.len() - 1
                }
            };
            let previous = facts[index].facts.clone();
            facts[index].facts.observe(kind);
            if previous == facts[index].facts {
                continue;
            }
            saved["device_receipts"] = serde_json::to_value(facts)
                .map_err(|_| invalid("cannot encode group receipt facts"))?;
            tx.put_json(Table::Checkpoints, &key, 1, &saved)?;
            matched += 1;
        }
    }
    Ok(matched)
}

/// Redacted comparison-send receipt counts; each fact concerns one device.
/// Counts never establish delivery to every member or server acceptance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupReceiptSummary {
    /// Preserved recognized send outcome, or `unknown` for a legacy value.
    pub send_status: String,
    /// Original target count, or unknown when historical wire lacks targets.
    pub addressed_devices: Option<usize>,
    /// Distinct actual devices with at least one observed receipt fact.
    pub receipt_devices: usize,
    /// Devices with a delivery observation (reads do not invent delivery).
    pub delivered_devices: usize,
    /// Devices with a read observation.
    pub read_devices: usize,
    /// Devices with a played observation.
    pub played_devices: usize,
}
/// Inspect one existing comparison group-send reservation without queuing work.
/// None means no matching group reservation, not a server refusal.
pub fn summary(
    store: &ProtocolStore,
    request_id: &ClientRequestId,
) -> Result<Option<GroupReceiptSummary>> {
    store.read(|tx| {
        let Some(saved): Option<serde_json::Value> = tx.get_json(
            Table::Checkpoints,
            format!("message:{request_id}:send").as_bytes(),
            1,
        )?
        else {
            return Ok(None);
        };
        let Some(chat) = saved["chat"].as_str() else {
            return Ok(None);
        };
        let chat = Jid::parse(chat).map_err(|_| invalid("invalid saved group receipt chat"))?;
        if !group(&chat) {
            return Ok(None);
        }
        let id = saved["wire_id"]
            .as_str()
            .ok_or_else(|| invalid("missing saved group receipt wire id"))?;
        let targets = saved_targets(&saved, &chat, id)?;
        let facts = saved_facts(&saved)?;
        let status = saved["status"]
            .as_str()
            .filter(|s| {
                matches!(
                    *s,
                    "outcome_unknown" | "server_acknowledged" | "server_refused"
                )
            })
            .unwrap_or("unknown");
        Ok(Some(GroupReceiptSummary {
            send_status: status.into(),
            addressed_devices: targets.map(|v| v.len()),
            receipt_devices: facts.len(),
            delivered_devices: facts.iter().filter(|r| r.facts.delivered).count(),
            read_devices: facts.iter().filter(|r| r.facts.read).count(),
            played_devices: facts.iter().filter(|r| r.facts.played).count(),
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binary::{Encoder, TokenDictionary},
        model::ids::{AccountId, ChatId, LocalMessageId},
        model::{
            message::MessageContent,
            send::{AttemptProgress, SendRequest},
        },
        outbox::AttemptRecord,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    const CHAT: &str = "120363000000000001@g.us";
    const PN: &str = "15555550100@s.whatsapp.net";
    const LID: &str = "100001@lid";
    fn open(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.into(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create,
        })
        .unwrap()
    }
    fn wire(id: &str, targets: &[Jid]) -> Vec<u8> {
        let mut node = Node::new("message")
            .attr("to", Jid::parse(CHAT).unwrap())
            .attr("id", id)
            .attr("type", "text");
        let mut children = vec![
            Node::new("enc")
                .attr("v", "2")
                .attr("type", "skmsg")
                .bytes(vec![1, 2, 3]),
        ];
        if !targets.is_empty() {
            children.push(
                Node::new("participants").children(
                    targets
                        .iter()
                        .map(|t| {
                            Node::new("to").attr("jid", t.clone()).children(vec![
                                Node::new("enc").attr("type", "msg").bytes(vec![4, 5, 6]),
                            ])
                        })
                        .collect(),
                ),
            );
        }
        node = node.children(children);
        Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&node)
            .unwrap()
    }
    fn legacy(
        store: &ProtocolStore,
        request: &str,
        id: &str,
        targets: &[Jid],
        out_of_band: bool,
        new_field: bool,
    ) {
        let wire = wire(id, if out_of_band { &[] } else { targets });
        let mut row = serde_json::json!({"chat":CHAT,"wire_id":id,"wire":wire,"devices":targets.len(),"status":"outcome_unknown"});
        if new_field {
            row["recipient_devices"] = serde_json::to_value(targets).unwrap();
        }
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("message:{request}:send").as_bytes(),
                    1,
                    &row,
                )
            })
            .unwrap();
    }
    fn queued(store: &ProtocolStore, id: &str, targets: &[Jid]) -> LocalMessageId {
        let outbox = Outbox::new("group-fixture", Limits::default());
        store
            .write(|tx| {
                let (ticket, _) = outbox.enqueue(
                    tx,
                    &SendRequest {
                        client_request_id: ClientRequestId::new(id)?,
                        chat: ChatId::from_jid(&Jid::parse(CHAT)?)?,
                        content: MessageContent::Text {
                            body: "private group fixture".into(),
                            quoted: None,
                            style: Default::default(),
                        },
                    },
                )?;
                let mut entry = outbox.get_mut(tx, ticket.local_id)?.unwrap();
                entry.wire_id = Some(MessageId::new(id)?);
                entry.attempts.push(AttemptRecord {
                    index: 0,
                    prepared_at: Timestamp(7),
                    epoch: None,
                    payload: Some(wire(id, targets).into()),
                    re_encryption: false,
                });
                outbox.set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                outbox.record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::DeliveryUnknown,
                        device: None,
                        observed_at: Timestamp(8),
                        server_timestamp: None,
                        code: None,
                    },
                )?;
                Ok(ticket.local_id)
            })
            .unwrap()
    }
    fn receipt(id: &str, participant: &str, kind: &str) -> Node {
        Node::new("receipt")
            .attr("from", CHAT)
            .attr("participant", participant)
            .attr("id", id)
            .attr("type", kind)
    }
    fn summary_for(store: &ProtocolStore, id: &str) -> GroupReceiptSummary {
        summary(store, &ClientRequestId::new(id).unwrap())
            .unwrap()
            .unwrap()
    }

    #[test]
    fn offline_aggregated_group_receipts_use_frozen_targets_through_public_handler() {
        for form in ["message_id", "key"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("group-aggregate.redb");
            let store = open(&path, true);
            let targets = [
                Jid::parse(PN).unwrap(),
                Jid::parse(PN).unwrap().with_device(2),
            ];
            let id = queued(&store, "fixture", &targets);
            store
                .write(|tx| super::super::send_versions::capture_in(tx, id, &targets))
                .unwrap();
            let outbox = Outbox::new("group-fixture", Limits::default());
            let before = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
            let node = Node::new("receipt")
                .attr("from", CHAT)
                .attr("id", "transport-id")
                .children(vec![
                    Node::new("participants")
                        .attr(form, "fixture")
                        .children(vec![
                            Node::new("participant").attr("jid", PN).attr("t", "20"),
                            Node::new("participant")
                                .attr("jid", "15555550100:2@s.whatsapp.net")
                                .attr("type", "read")
                                .attr("t", "30"),
                            Node::new("participant")
                                .attr("jid", "15555550100:3@s.whatsapp.net")
                                .attr("t", "40"),
                        ]),
                ]);
            assert_eq!(record(&store, &node).unwrap(), 2);
            let saved = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
            assert_eq!(saved.facts.len(), 3);
            assert_eq!(saved.facts[1].kind, DeliveryFactKind::Delivered);
            assert_eq!(saved.facts[1].device.as_ref().unwrap().device.0, 0);
            assert_eq!(saved.facts[1].server_timestamp, Some(Timestamp(20_000)));
            assert_eq!(
                saved.facts[2].kind,
                if form == "key" {
                    DeliveryFactKind::Delivered
                } else {
                    DeliveryFactKind::Read
                }
            );
            assert_eq!(saved.facts[2].device.as_ref().unwrap().device.0, 2);
            assert_eq!(saved.facts[2].server_timestamp, Some(Timestamp(30_000)));
            assert_eq!(saved.attempts, before.attempts);
            assert_eq!(saved.progress, before.progress);
            assert_eq!(saved.intent, before.intent);
            let head = store.read(|tx| tx.journal_head()).unwrap();
            drop(store);
            let store = open(&path, false);
            assert_eq!(record(&store, &node).unwrap(), 0);
            assert_eq!(
                saved,
                store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap()
            );
            assert_eq!(head, store.read(|tx| tx.journal_head()).unwrap());
        }
    }

    #[test]
    fn group_receipts_full_plan_includes_devices_without_sender_key_children() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("full-plan.redb");
        let store = open(&path, true);
        let primary = Jid::parse(PN).unwrap();
        let companion = primary.with_device(3);
        let lid = Jid::parse(LID).unwrap();
        let targets = [primary.clone(), companion.clone(), lid.clone()];
        let local = queued(&store, "full-plan-wire", std::slice::from_ref(&primary));
        store
            .write(|tx| super::super::send_versions::capture_in(tx, local, &targets))
            .unwrap();
        let outbox = Outbox::new("observer", Limits::default());
        let before = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
        for target in [&companion, &lid] {
            let node = receipt("full-plan-wire", &target.to_string(), "delivery").attr("t", "20");
            assert_eq!(record(&store, &node).unwrap(), 1);
            assert_eq!(record(&store, &node).unwrap(), 0);
            assert_eq!(record(&store, &node.attr("t", "10")).unwrap(), 0);
        }
        assert_eq!(
            record(
                &store,
                &receipt(
                    "full-plan-wire",
                    &primary.with_device(4).to_string(),
                    "read"
                )
                .attr("t", "1")
            )
            .unwrap(),
            0
        );
        let after = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
        assert_eq!(after.attempts, before.attempts);
        assert_eq!(after.progress, before.progress);
        assert_eq!(after.wire_id, before.wire_id);
        assert_eq!(after.facts.len(), before.facts.len() + 2);
        assert!(
            after
                .facts
                .iter()
                .skip(before.facts.len())
                .all(|fact| fact.server_timestamp == Some(Timestamp(20000)))
        );
        let versions = super::super::send_versions::inspect(&store, local).unwrap();
        let rows = versions.device_receipts.as_ref().unwrap();
        assert_eq!(
            rows.iter().filter(|r| r.timestamp == Some(10000)).count(),
            2
        );
        assert_eq!(rows.iter().filter(|r| r.timestamp.is_none()).count(), 1);
        assert_eq!(
            versions
                .users
                .iter()
                .map(|u| u.device_targets.as_ref().unwrap().len())
                .sum::<usize>(),
            3
        );
        drop(store);
        let store = open(&path, false);
        assert_eq!(
            record(
                &store,
                &receipt("full-plan-wire", &companion.to_string(), "delivery")
            )
            .unwrap(),
            0
        );
        assert_eq!(
            super::super::send_versions::inspect(&store, local).unwrap(),
            versions
        );
    }

    #[test]
    fn group_receipts_refuse_conflicting_full_plan_and_preserve_legacy_unknown_targets() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("conflict-plan.redb"), true);
        let primary = Jid::parse(PN).unwrap();
        let companion = primary.with_device(3);
        let local = queued(&store, "conflicting-wire", std::slice::from_ref(&primary));
        store
            .write(|tx| {
                super::super::send_versions::capture_in(tx, local, std::slice::from_ref(&companion))
            })
            .unwrap();
        let before = store.read(|tx| tx.scan(Table::Outbox, 1, 100)).unwrap();
        let error = record(&store, &receipt("conflicting-wire", PN, "delivery"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("sender-key targets exceed original device plan"));
        assert!(!error.contains(PN));
        assert_eq!(
            store.read(|tx| tx.scan(Table::Outbox, 1, 100)).unwrap(),
            before
        );
        queued(&store, "legacy-wire", std::slice::from_ref(&primary));
        assert_eq!(
            record(
                &store,
                &receipt("legacy-wire", &companion.to_string(), "delivery")
            )
            .unwrap(),
            0
        );
        assert_eq!(
            record(&store, &receipt("legacy-wire", PN, "delivery")).unwrap(),
            1
        );
    }

    #[test]
    fn group_receipts_bind_frozen_targets_keep_actual_device_and_survive_reopen() {
        for (name, target, participant, map, expected) in [
            ("phone", PN, PN, false, true),
            ("lid", LID, LID, false, true),
            ("mapped", PN, LID, true, true),
            ("reverse", LID, PN, true, true),
            ("zero-ad", PN, "15555550100:0@s.whatsapp.net", false, true),
            (
                "companion",
                "15555550100:3@s.whatsapp.net",
                "15555550100:3@s.whatsapp.net",
                false,
                true,
            ),
            (
                "mapped-companion",
                "15555550100:3@s.whatsapp.net",
                "100001:3@lid",
                true,
                true,
            ),
            (
                "other-device",
                "15555550100:3@s.whatsapp.net",
                "15555550100:4@s.whatsapp.net",
                true,
                false,
            ),
            ("foreign", PN, "15555550101@s.whatsapp.net", true, false),
            ("unattributed", PN, LID, false, false),
            ("reverse-unattributed", LID, PN, false, false),
            ("agent", PN, "15555550100.1:0@s.whatsapp.net", true, false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("group.redb");
            let store = open(&path, true);
            let targets = [Jid::parse(target).unwrap()];
            legacy(&store, name, "lab-wire", &targets, false, false);
            let local = queued(&store, "normal-wire", &targets);
            store
                .write(|tx| {
                    if map {
                        tx.put_json(
                            Table::Checkpoints,
                            b"authenticated-lid:100001@lid",
                            1,
                            &Jid::parse(PN)?,
                        )?;
                    }
                    tx.put_json(Table::Checkpoints, b"lid:100001@lid", 1, &Jid::parse(PN)?)?;
                    tx.put_json(
                        Table::Checkpoints,
                        b"receipt-outbox:held",
                        1,
                        &serde_json::json!({"status":"outcome_unknown","wire":[9,8,7]}),
                    )
                })
                .unwrap();
            let observer = Outbox::new("another-namespace", Limits::default());
            let before = store.read(|tx| observer.get(tx, local)).unwrap().unwrap();
            let a = receipt("lab-wire", participant, "read");
            let b = receipt("normal-wire", participant, "read");
            // Exercise the actual wire decoder and shared receive dispatcher.
            let encoded = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&a)
                .unwrap();
            assert_eq!(
                super::super::receipts::record(&store, &decode_wire(&encoded).unwrap()).unwrap(),
                usize::from(expected),
                "{name}"
            );
            assert_eq!(
                super::super::receipts::record(&store, &b).unwrap(),
                usize::from(expected),
                "{name}"
            );
            assert_eq!(record(&store, &a).unwrap(), 0);
            assert_eq!(record(&store, &b).unwrap(), 0);
            drop(store);
            let store = open(&path, false);
            let summary = summary_for(&store, name);
            assert_eq!(summary.addressed_devices, Some(1));
            assert_eq!(summary.read_devices, usize::from(expected));
            assert_eq!(summary.delivered_devices, 0);
            assert_eq!(summary.played_devices, 0);
            assert_eq!(summary.send_status, "outcome_unknown");
            let after = store.read(|tx| observer.get(tx, local)).unwrap().unwrap();
            assert_eq!(before.intent, after.intent);
            assert_eq!(before.wire_id, after.wire_id);
            assert_eq!(before.attempts, after.attempts);
            assert_eq!(before.progress, after.progress);
            assert_eq!(before.facts[0], after.facts[0]);
            assert_eq!(after.facts.len(), if expected { 2 } else { 1 });
            if expected {
                assert_eq!(
                    after.facts[1].device,
                    Some(DeviceAddress::from_jid(&Jid::parse(participant).unwrap()).unwrap())
                );
            }
            assert!(
                store
                    .read(|tx| observer.status(tx, local))
                    .unwrap()
                    .unwrap()
                    .is_uncertain()
            );
            let held: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, b"receipt-outbox:held", 1))
                .unwrap()
                .unwrap();
            assert_eq!(
                held,
                serde_json::json!({"status":"outcome_unknown","wire":[9,8,7]})
            );
        }
    }
    #[test]
    fn group_facts_are_independent_per_device_and_never_mean_every_member() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("facts.redb"), true);
        let targets = [
            Jid::parse(PN).unwrap(),
            Jid::parse("15555550101:2@s.whatsapp.net").unwrap(),
        ];
        legacy(&store, "fixture", "wire", &targets, false, true);
        assert_eq!(record(&store, &receipt("wire", PN, "read")).unwrap(), 1);
        assert_eq!(record(&store, &receipt("wire", PN, "delivery")).unwrap(), 1);
        assert_eq!(
            record(
                &store,
                &receipt("wire", "15555550101:2@s.whatsapp.net", "played")
            )
            .unwrap(),
            1
        );
        let s = summary_for(&store, "fixture");
        assert_eq!(s.addressed_devices, Some(2));
        assert_eq!(s.receipt_devices, 2);
        assert_eq!(s.delivered_devices, 1);
        assert_eq!(s.read_devices, 1);
        assert_eq!(s.played_devices, 1);
        assert_eq!(s.send_status, "outcome_unknown");
    }
    #[test]
    fn out_of_band_target_snapshot_enables_new_sends_and_old_unknowns_stay_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("out-of-band.redb"), true);
        let targets = [Jid::parse(PN).unwrap()];
        legacy(&store, "old", "old-wire", &targets, true, false);
        legacy(&store, "new", "new-wire", &targets, true, true);
        assert_eq!(
            record(&store, &receipt("old-wire", PN, "delivery")).unwrap(),
            0
        );
        assert_eq!(
            record(&store, &receipt("new-wire", PN, "delivery")).unwrap(),
            1
        );
        assert_eq!(summary_for(&store, "old").addressed_devices, None);
        assert_eq!(summary_for(&store, "new").addressed_devices, Some(1));
        // An explicit empty frozen set is known zero, unlike a missing old set.
        legacy(&store, "empty", "empty-wire", &[], true, true);
        assert_eq!(summary_for(&store, "empty").addressed_devices, Some(0));
        assert_eq!(
            record(&store, &receipt("empty-wire", PN, "read")).unwrap(),
            0
        );
    }
    #[test]
    fn invalid_optional_recipient_does_not_hide_an_original_group_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("optional-group-recipient.redb");
        let store = open(&path, true);
        let target = Jid::parse(PN).unwrap().with_device(2);
        let local = queued(&store, "optional-group", std::slice::from_ref(&target));
        let outbox = Outbox::new("group-fixture", Limits::default());
        let before = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
        for (kind, fact_kind, recipient) in [
            (
                "delivery",
                DeliveryFactKind::Delivered,
                "15555550100@unknown.example",
            ),
            ("read", DeliveryFactKind::Read, "120363000000000001@g.us"),
            (
                "played",
                DeliveryFactKind::Played,
                "15555550100:2@s.whatsapp.net",
            ),
        ] {
            let node =
                receipt("optional-group", &target.to_string(), kind).attr("recipient", recipient);
            assert_eq!(record(&store, &node).unwrap(), 1);
            let head = store.journal_head().unwrap();
            assert_eq!(record(&store, &node).unwrap(), 0);
            assert_eq!(store.journal_head().unwrap(), head);
            let saved = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
            assert_eq!(saved.attempts, before.attempts);
            assert_eq!(saved.progress, before.progress);
            let fact = saved
                .facts
                .iter()
                .find(|fact| fact.kind == fact_kind)
                .unwrap();
            assert_eq!(fact.device, Some(DeviceAddress::from_jid(&target).unwrap()));
        }
        let saved = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
        let head = store.journal_head().unwrap();
        for participant in ["unknown.example", "15555550100:3@s.whatsapp.net"] {
            assert_eq!(
                record(
                    &store,
                    &receipt("optional-group", participant, "read")
                        .attr("recipient", "unknown.example")
                )
                .unwrap(),
                0
            );
            assert_eq!(store.journal_head().unwrap(), head);
            assert_eq!(
                store.read(|tx| outbox.get(tx, local)).unwrap().unwrap(),
                saved
            );
        }
        drop(store);
        let store = open(&path, false);
        assert_eq!(
            store.read(|tx| outbox.get(tx, local)).unwrap().unwrap(),
            saved
        );
        assert_eq!(
            record(
                &store,
                &receipt("optional-group", &target.to_string(), "read")
                    .attr("recipient", "unknown.example")
            )
            .unwrap(),
            0
        );
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn group_metadata_and_malformed_type_refusals_are_side_effect_free() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("unknown.redb"), true);
        legacy(
            &store,
            "fixture",
            "wire",
            &[Jid::parse(PN).unwrap()],
            false,
            true,
        );
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        let mut nonstandard = Jid::parse(PN).unwrap();
        nonstandard.kind = JidKind::Interop;
        for node in [
            Node::new("receipt").attr("from", CHAT).attr("id", "wire"),
            receipt("wire", PN, "read-self"),
            receipt("wire", PN, "played-self"),
            receipt("wire", PN, "read").attr("recipient", PN),
            receipt("wire", PN, "read").attr("from", "120363000000000002@g.us"),
            receipt("wire", PN, "read").attr("from", "120363000000000001:1@g.us"),
            receipt("wire", PN, "read").attr("participant", "status@broadcast"),
            receipt("wire", PN, "read").attr("participant", nonstandard),
            receipt("wire", PN, "read").attr("type", Jid::parse(LID).unwrap()),
            receipt("wire", PN, "read").attr(
                "type",
                crate::binary::AttributeValue::Bytes(bytes::Bytes::from_static(&[0xff])),
            ),
        ] {
            assert_eq!(super::super::receipts::record(&store, &node).unwrap(), 0);
            assert_eq!(
                before,
                store
                    .read(|tx| Ok((
                        tx.scan(Table::Checkpoints, 1, 100)?,
                        tx.scan(Table::Outbox, 1, 100)?,
                        tx.journal_head()?
                    )))
                    .unwrap()
            );
        }
        assert!(
            summary(&store, &ClientRequestId::new("missing").unwrap())
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn corrupt_late_group_record_rolls_back_prior_outbox_facts_and_events() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("rollback.redb"), true);
        let targets = [Jid::parse(PN).unwrap()];
        let local = queued(&store, "wire", &targets);
        store
            .write(|tx| super::super::send_versions::capture_in(tx, local, &targets))
            .unwrap();
        legacy(&store, "fixture", "wire", &targets, false, true);
        store
            .write(|tx| {
                let mut row: serde_json::Value = tx
                    .get_json(Table::Checkpoints, b"message:fixture:send", 1)?
                    .unwrap();
                row["device_receipts"] = serde_json::json!("DO-NOT-ECHO");
                tx.put_json(Table::Checkpoints, b"message:fixture:send", 1, &row)
            })
            .unwrap();
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        let error = record(&store, &receipt("wire", PN, "read").attr("t", "10"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("invalid saved group receipt facts"));
        assert!(!error.contains("DO-NOT-ECHO"));
        assert_eq!(
            before,
            store
                .read(|tx| Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?
                )))
                .unwrap()
        );
    }
    #[test]
    fn group_wire_and_new_snapshot_must_agree_without_redaction_leaks() {
        for (field, value) in [
            ("wire", serde_json::json!("DO-NOT-ECHO")),
            ("recipient_devices", serde_json::json!("DO-NOT-ECHO")),
            (
                "recipient_devices",
                serde_json::json!([Jid::parse("15555550101@s.whatsapp.net").unwrap()]),
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("corrupt.redb"), true);
            legacy(
                &store,
                "fixture",
                "wire",
                &[Jid::parse(PN).unwrap()],
                false,
                true,
            );
            store
                .write(|tx| {
                    let mut row: serde_json::Value = tx
                        .get_json(Table::Checkpoints, b"message:fixture:send", 1)?
                        .unwrap();
                    row[field] = value.clone();
                    tx.put_json(Table::Checkpoints, b"message:fixture:send", 1, &row)
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let error = record(&store, &receipt("wire", PN, "delivery"))
                .unwrap_err()
                .to_string();
            assert!(!error.contains("DO-NOT-ECHO") && !error.contains("15555550101"));
            assert_eq!(
                before,
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap()
            );
        }
    }
    #[test]
    fn group_receipt_checkpoint_pages_and_ambiguous_legacy_ids_are_atomic() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("pages.redb"), true);
        let targets = [Jid::parse(PN).unwrap()];
        store
            .write(|tx| {
                for i in 0..10_000 {
                    tx.put(
                        Table::Checkpoints,
                        format!("a-history:{i:05}").as_bytes(),
                        1,
                        b"opaque",
                    )?;
                }
                Ok(())
            })
            .unwrap();
        legacy(&store, "first", "wire", &targets, false, false);
        assert_eq!(record(&store, &receipt("wire", PN, "read")).unwrap(), 1);
        legacy(&store, "duplicate", "wire", &targets, false, false);
        let before = summary_for(&store, "first");
        let error = record(&store, &receipt("wire", PN, "played"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("ambiguous saved group receipt wire id"));
        assert_eq!(before, summary_for(&store, "first"));
        assert_eq!(summary_for(&store, "duplicate").played_devices, 0);
    }
    #[test]
    fn grouped_ids_deduplicate_and_ambiguous_outbox_wire_rolls_back_the_batch() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("batch.redb"), true);
        let targets = [Jid::parse(PN).unwrap()];
        legacy(&store, "first", "a-wire", &targets, false, false);
        legacy(&store, "second", "b-wire", &targets, false, false);
        let batch = receipt("a-wire", PN, "read").children(vec![Node::new("list").children(vec![
            Node::new("item").attr("id", "b-wire"),
            Node::new("item").attr("id", "a-wire"),
        ])]);
        assert_eq!(record(&store, &batch).unwrap(), 2);
        assert_eq!(record(&store, &batch).unwrap(), 0);
        assert_eq!(summary_for(&store, "first").read_devices, 1);
        assert_eq!(summary_for(&store, "second").read_devices, 1);
        queued(&store, "a-wire", &targets);
        let duplicate = queued(&store, "z-wire", &targets);
        store
            .write(|tx| {
                let outbox = Outbox::new("other-namespace", Limits::default());
                let mut entry = outbox.get_mut(tx, duplicate)?.unwrap();
                entry.wire_id = Some(MessageId::new("b-wire")?);
                tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)
            })
            .unwrap();
        let second = queued(&store, "b-wire", &targets);
        assert_ne!(second, duplicate);
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        let error = record(&store, &batch.attr("type", "played"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("duplicate outbox wire id"));
        assert_eq!(
            before,
            store
                .read(|tx| Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?
                )))
                .unwrap()
        );
    }
}
