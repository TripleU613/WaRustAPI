//! APK 2.26.37.71 LX/Cmk.A03 receipt facts; self receipts are separate statuses.
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
        crate::binary::TokenDictionary::comparison_2_26_37_71(),
        &limits,
    )
    .decode_node()?)
}

// Encoding kind is not a routing identity. Preserve user/server/agent/device;
// never use to_non_ad here, which would merge different devices.
fn logical_address(mut jid: Jid) -> Jid {
    jid.kind = crate::model::jid::JidKind::Plain;
    jid
}

fn retry_selector(chat: &Jid, participant: &Option<Jid>, id: &str) -> Result<String> {
    // Keep the predecessor JSON tuple spelling for historical lookup.
    let route = serde_json::to_vec(&(chat, participant, id))
        .map_err(|_| Error::invalid("receipt", "invalid retry routing"))?;
    Ok(format!(
        "receipt-outbox:{}:retry",
        hex::encode(Sha256::digest(route))
    ))
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
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
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

/// Durably queue explicit `read` receipts for authenticated incoming direct or
/// group messages, one receipt per message ID, all-or-nothing and idempotent.
/// APK 2.26.37.71 SendReadReceiptJob.A0G -> 0zd.A04 -> CM8 ->
/// Cp2.A07/A00 confirms the shared receipt builder and optional participant.
/// Cp2.A0A builds native extra-ID lists at reservation for matching read routes;
/// each authenticated ID retains its original durable reservation.
/// See docs/protocol/read-receipt-job.md for the traced policy/route boundaries.
/// Full APK preference/contact policy and read-self selection remain unported;
/// this API is an explicit read action (`ReceiptPolicy::ExplicitOnly`).
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
    // A group is allowed as well: its receipt is the same builder
    // (`received_confirmation`, `LX/Cp2`) with the sender as `participant`,
    // which the live-verified group delivery receipt already uses.
    if !matches!(chat.server, Server::User | Server::Lid | Server::Group) || chat.device != 0 {
        return Err(Error::invalid(
            "receipt",
            "only primary direct chats and groups are supported",
        ));
    }
    if chat.server != Server::Group && receipt.from.is_some() {
        return Err(Error::invalid(
            "receipt",
            "participant is only a group read selector",
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
            let (key, mut saved) = if chat.server == Server::Group {
                let participant = receipt.from.as_ref().map(|d| d.to_jid());
                super::group_inbox::read_target(tx, chat, id, participant.as_ref())?
            } else {
                let key = format!("received:{chat}:{id}");
                let saved = tx
                    .get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?
                    .ok_or_else(|| {
                        Error::invalid("receipt", "unknown incoming message for this chat")
                    })?;
                (key, saved)
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
            let mut node = super::messaging::received_confirmation(&incoming)
                .filter(|n| n.description == "receipt")
                .ok_or_else(|| Error::invalid("receipt", "unsupported read receipt routing"))?
                .attr("type", "read");
            // Cp2.A00 appends read type before the optional participant.
            node.attributes.sort_by_key(|a| match a.name.as_str() {
                "to" => 0,
                "id" => 1,
                "type" => 2,
                "participant" => 3,
                _ => 4,
            });
            let wire = crate::binary::Encoder::new(
                crate::binary::TokenDictionary::comparison_2_26_37_71(),
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

/// `Message` fields that are something a person reads. A payload carrying any
/// of them earns a read receipt when `transport.mark_read` is on; one carrying
/// only a sender-key distribution (2), a protocol message (12), a reaction
/// (46) or anything else does not, because nobody "reads" those. Numbers are
/// the ones validated against the APK descriptor (`xtask apk-proto-fields`):
/// conversation 1, imageMessage 3, extendedTextMessage 6, documentMessage 7,
/// audioMessage 8, videoMessage 9, stickerMessage 26, ptvMessage 66.
const READABLE_FIELDS: [u32; 8] = [1, 3, 6, 7, 8, 9, 26, 66];

/// Queue a read receipt for one authenticated incoming message, including a
/// duplicate recovered offline, when its content is something a person reads. Returns how many
/// were queued (0 or 1).
///
/// This is the automatic form of [`queue_read`], for a session whose operator
/// asked for everything to be marked read (`transport.mark_read`). The official
/// client sends read receipts when a chat is on screen; a client without a
/// screen has to be told, which is why this is opt-in.
pub fn queue_read_for_incoming(store: &ProtocolStore, node: &Node) -> Result<usize> {
    use crate::model::{
        ids::{ChatId, MessageId},
        receipt::{Receipt, ReceiptKind},
    };
    if node.description != "message" {
        return Ok(0);
    }
    let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(0);
    };
    // Use the original stored selector, including a legacy PN row proven by
    // its authenticated capture; a redelivery alias cannot manufacture a read.
    let (chat, saved) = if let Some(chat) =
        jid(node, "from").filter(|chat| chat.server == Server::Group)
    {
        let selected = store
            .read(|tx| super::group_inbox::select(node, |k| tx.get(Table::Checkpoints, k, 1)))?;
        let Some(saved) = selected.saved else {
            return Ok(0);
        };
        (chat, saved)
    } else {
        let Some(record) = store.read(|tx| {
            super::inbox::direct_record_in(node, |key| tx.get_json(Table::Checkpoints, key, 1))
        })?
        else {
            return Ok(0);
        };
        let Some(saved) = record.saved else {
            return Ok(0);
        };
        (record.chat, saved)
    };
    if saved["status"] != "decrypted" {
        return Ok(0);
    }
    let Ok(protobuf) =
        serde_json::from_value::<zeroize::Zeroizing<Vec<u8>>>(saved["protobuf"].clone())
    else {
        return Ok(0);
    };
    let fields = crate::signal::direct::payload_fields(&protobuf).unwrap_or_default();
    if !fields.iter().any(|f| READABLE_FIELDS.contains(f)) {
        return Ok(0);
    }
    // DeviceAddress cannot preserve an agent component. Refuse this automatic
    // selector rather than accidentally marking agent 0's same-ID message.
    if chat.server == Server::Group
        && super::inbox::group_sender(node).is_some_and(|(_, p)| p.agent != 0)
    {
        return Ok(0);
    }
    queue_read(
        store,
        &Receipt {
            chat: ChatId::from_jid(&chat)?,
            message_ids: vec![MessageId::new(id.to_owned())?],
            from: if chat.server == Server::Group {
                super::inbox::group_sender(node)
                    .map(|(_, p)| crate::model::ids::DeviceAddress::from_jid(&p))
                    .transpose()?
            } else {
                None
            },
            kind: ReceiptKind::Read,
            server_timestamp: None,
            code: None,
        },
    )
}

/// Queue at most one resend request for a failed incoming message. Call only
/// after preserving the capture and classifying it as `Undecryptable`.
///
/// Shape: `messaging::retry_receipt`, observed handset receipts 2026-09-24.
/// One request per (chat, participant, message ID) is our conservative policy,
/// not an APK retry limit. The normal receipt reservation prevents replay after
/// an uncertain write; redelivery never resets the saved outcome or count.
pub fn queue_retry_for_incoming(
    store: &ProtocolStore,
    account: &crate::session::AccountRecord,
    node: &Node,
) -> Result<usize> {
    if node.description != "message" {
        return Err(Error::invalid(
            "receipt",
            "retry requires an incoming message",
        ));
    }
    let registration = account.signal_registration_id.ok_or_else(|| {
        Error::invalid("receipt", "retry requires the saved Signal registration id")
    })?;
    let chat = jid(node, "from")
        .ok_or_else(|| Error::invalid("receipt", "retry requires a sender JID"))?;
    let participant = jid(node, "participant");
    if !matches!(chat.server, Server::User | Server::Lid | Server::Group)
        || (chat.server == Server::Group && participant.is_none())
        || (node.get_attr("participant").is_some() && participant.is_none())
        || participant
            .as_ref()
            .is_some_and(|p| !matches!(p.server, Server::User | Server::Lid))
    {
        return Err(Error::invalid("receipt", "unsupported retry routing"));
    }
    let id = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .ok_or_else(|| Error::invalid("receipt", "retry requires a message id"))?;
    let key = retry_selector(
        &logical_address(chat.clone()),
        &participant.clone().map(logical_address),
        id,
    )?;
    // The old selector serialized JidKind. Probe every historical enum spelling
    // inside the same transaction before adding a canonical reservation. Keep
    // old unknown/refused/acknowledged rows and original wire untouched.
    use crate::model::jid::JidKind::{AgentDevice, Facebook, Interop, Plain};
    let mut historical = Vec::with_capacity(16);
    for chat_kind in [Plain, AgentDevice, Interop, Facebook] {
        let mut old_chat = chat.clone();
        old_chat.kind = chat_kind;
        for participant_kind in [Plain, AgentDevice, Interop, Facebook] {
            let mut old_participant = participant.clone();
            if let Some(participant) = &mut old_participant {
                participant.kind = participant_kind;
            }
            let candidate = retry_selector(&old_chat, &old_participant, id)?;
            if !historical.contains(&candidate) {
                historical.push(candidate);
            }
        }
    }
    let retry = super::messaging::retry_receipt(
        &chat,
        participant.as_ref(),
        id,
        1,
        registration,
        crate::model::time::Timestamp::now().0 / 1000,
    )?;
    let wire = crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
        .encode(&retry)?;
    store.write(|tx| {
        for candidate in &historical {
            if tx
                .get_json::<serde_json::Value>(Table::Checkpoints, candidate.as_bytes(), 1)?
                .is_some()
            {
                return Ok(0);
            }
        }
        tx.put_json(
            Table::Checkpoints,
            key.as_bytes(),
            1,
            &serde_json::json!({"status":"pending", "wire":wire}),
        )?;
        Ok(1)
    })
}

/// One queued receipt's outcome, redacted for operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptOutcome {
    /// `played`, `read` or `retry`.
    pub kind: String,
    /// `pending`, `outcome_unknown`, `server_acknowledged` or `refused`.
    pub status: String,
    /// Wire message id the receipt refers to.
    pub message_id: String,
}
fn batch_ack_id(row: &serde_json::Value) -> Result<Option<&str>> {
    row.get("batch_ack_id")
        .map(|value| {
            value
                .as_str()
                .filter(|id| !id.is_empty() && id.len() <= 128)
                .ok_or_else(|| Error::invalid("receipt", "invalid saved batch correlation"))
        })
        .transpose()
}

/// Every receipt ever queued, in key order. No recipient is printed.
/// Malformed saved wire/IDs return fixed redacted errors; inspection never
/// rewrites or repairs a reservation, including uncertain outcomes.
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
                let _ = batch_ack_id(&row)?;
                if row["wire"].is_null() {
                    return Err(Error::invalid("receipt", "missing saved receipt wire"));
                }
                // Do not drop non-byte values or truncate integers: inspection
                // must diagnose corrupt reservations without changing their outcome.
                let wire: Vec<u8> = serde_json::from_value(row["wire"].clone())
                    .map_err(|_| Error::invalid("receipt", "invalid saved receipt wire bytes"))?;
                // Decoder errors can contain input-derived detail; this offline
                // operator path reports a fixed diagnosis, never stored route data.
                let node = decode(&wire)
                    .map_err(|_| Error::invalid("receipt", "cannot decode saved receipt wire"))?;
                if node.description != "receipt" {
                    return Err(Error::invalid(
                        "receipt",
                        "saved receipt wire is not a receipt",
                    ));
                }
                let message_id = node
                    .get_attr("id")
                    .and_then(|value| value.as_text())
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| {
                        Error::invalid("receipt", "missing or invalid saved receipt message id")
                    })?
                    .to_owned();
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

/// Reserve at most sixteen pending per-message receipts before any socket write.
/// Matching read routes share the APK extra-ID list; unknown outcomes never replay.
pub fn reserve_pending(store: &ProtocolStore) -> Result<Vec<Node>> {
    store.write(|tx| {
        let mut pending = Vec::new();
        let mut cursor = None;
        while pending.len() < 16 {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows
                .iter()
                .filter(|(key, _)| key.starts_with(b"receipt-outbox:"))
            {
                if pending.len() == 16 {
                    break;
                }
                let Some(row): Option<serde_json::Value> =
                    tx.get_json(Table::Checkpoints, key, 1)?
                else {
                    continue;
                };
                if row["status"] != "pending" {
                    continue;
                }
                if row.get("batch_ack_id").is_some() {
                    return Err(Error::invalid(
                        "receipt",
                        "pending receipt has batch correlation",
                    ));
                }
                let wire: Vec<u8> = serde_json::from_value(row["wire"].clone())
                    .map_err(|_| Error::invalid("receipt", "invalid receipt intent"))?;
                pending.push((key.clone(), row, decode(&wire)?));
            }
        }
        let batchable = |node: &Node| {
            node.description == "receipt"
                && jid(node, "to").is_some_and(|j| {
                    j.agent == 0
                        && matches!(j.server, Server::User | Server::Lid | Server::Group)
                        && (j.server != Server::Group || j.device == 0)
                })
                && node.get_attr("type").and_then(|v| v.as_text()) == Some("read")
                && node
                    .get_attr("id")
                    .and_then(|v| v.as_text())
                    .is_some_and(|id| !id.is_empty() && id.len() <= 128)
                && matches!(node.content, crate::binary::NodeContent::Empty)
                && node
                    .attributes
                    .iter()
                    .all(|a| matches!(a.name.as_str(), "to" | "id" | "type" | "participant"))
        };
        let mut out = Vec::new();
        while !pending.is_empty() {
            let (key, row, mut node) = pending.remove(0);
            let mut reserved = vec![(key, row)];
            let mut extras = Vec::new();
            if batchable(&node) {
                let mut i = 0;
                while i < pending.len() {
                    let other = &pending[i].2;
                    if batchable(other)
                        && node
                            .attributes
                            .iter()
                            .filter(|a| a.name != "id")
                            .eq(other.attributes.iter().filter(|a| a.name != "id"))
                    {
                        let (key, row, other) = pending.remove(i);
                        extras.push(
                            Node::new("item").attr("id", other.get_attr("id").unwrap().clone()),
                        );
                        reserved.push((key, row));
                    } else {
                        i += 1;
                    }
                }
            }
            let batched = !extras.is_empty();
            if batched {
                // Cp2.A0A: first ID stays at root; only the remainder enters list.
                node.content =
                    crate::binary::NodeContent::Children(vec![Node::new("list").children(extras)]);
            }
            for (key, mut row) in reserved {
                row["status"] = serde_json::json!("outcome_unknown");
                if batched {
                    row["batch_ack_id"] =
                        serde_json::json!(node.get_attr("id").unwrap().as_text().unwrap());
                }
                // Preserve the original per-message wire for inspection/deduplication.
                tx.put_json(Table::Checkpoints, &key, 1, &row)?;
            }
            out.push(node);
        }
        Ok(out)
    })
}

/// Correlate an explicit receipt ACK by recipient, ID and type, not ID alone.
/// Direct and group receipts retain their exact participant presence. APK
/// 37.71 1Qy.A02 -> CSu.A00 -> Cb1.equals distinguishes nullable participants;
/// recipient/edit routes remain unsupported by this queue.
pub fn record_ack(store: &ProtocolStore, node: &Node) -> Result<bool> {
    if node.description != "ack"
        || node.get_attr("class").and_then(|v| v.as_text()) != Some("receipt")
        || ["recipient", "edit"]
            .iter()
            .any(|name| node.get_attr(name).is_some())
    {
        return Ok(false);
    }
    store.write(|tx| {
        let mut cursor = None;
        let mut matched = false;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                return Ok(matched);
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
                let mut sent = decode(&wire)?;
                if let Some(root) = batch_ack_id(&row)? {
                    sent = sent.attr("id", root);
                }
                // APK 37.71 Cb1.equals 0086..00a6 compares nullable participant
                // fields. An absent ACK participant cannot settle a group receipt
                // naming a member; ID/from/type alone leaves that member ambiguous.
                if sent.get_attr("id") != node.get_attr("id")
                    || sent.get_attr("type") != node.get_attr("type")
                    || jid(&sent, "to").is_none()
                    || jid(&sent, "to").map(logical_address)
                        != jid(node, "from").map(logical_address)
                    || (sent.get_attr("participant").is_some()
                        != node.get_attr("participant").is_some())
                    || (sent.get_attr("participant").is_some()
                        && (jid(&sent, "participant").is_none()
                            || jid(&sent, "participant").map(logical_address)
                                != jid(node, "participant").map(logical_address)))
                {
                    continue;
                }
                row["status"] = serde_json::json!(if node.get_attr("error").is_some() {
                    "refused"
                } else {
                    "server_acknowledged"
                });
                tx.put_json(Table::Checkpoints, key, 1, &row)?;
                matched = true;
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
    pub(super) fn observe(&mut self, kind: &str) {
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
// APK 124.A03 reconstructs zero-AD users; 1Qe.A00 writes their JID_PAIR form.
pub(super) fn ack_address(jid: Jid) -> Jid {
    if jid.kind == crate::model::jid::JidKind::AgentDevice
        && jid.agent == 0
        && jid.device == 0
        && matches!(jid.server, Server::User | Server::Lid)
    {
        jid.to_non_ad()
    } else {
        jid
    }
}

pub(super) fn network_jid(node: &Node, name: &str) -> Option<Jid> {
    if node
        .get_attr(name)?
        .as_text()
        .is_some_and(|text| text.starts_with('@'))
    {
        return None;
    }
    let value = jid(node, name)?;
    // shortcut: namespace/server-only checks, extend when special JID grammar is ported.
    if value.user.is_empty() {
        return (matches!(value.server, Server::User | Server::Group)
            || matches!(&value.server, Server::Other(server) if server == "call"))
        .then_some(value);
    }
    if matches!(value.server, Server::LegacyUser | Server::Messenger)
        || matches!(&value.server, Server::Other(server)
            if !matches!(server.as_str(), "call" | "bot" | "temp" | "interop.g.us" | "status_me" | "lid_me"))
    {
        return None;
    }
    Some(value)
}

pub(super) fn recipient(node: &Node) -> Option<Jid> {
    // Text :0 still constructs DeviceJid; local parsing loses that class distinction.
    if node.get_attr("recipient")?.as_text().is_some_and(|text| {
        text.rsplit_once('@')
            .is_some_and(|(user, _)| user.contains(':'))
    }) {
        return None;
    }
    let value = network_jid(node, "recipient")?;
    // shortcut: shipped UserJid namespaces/zero-device encoding, extend with special grammar.
    (matches!(value.kind, crate::model::jid::JidKind::Plain | crate::model::jid::JidKind::AgentDevice)
        && value.agent == 0
        && value.device == 0
        && !value.user.is_empty()
        && (matches!(value.server, Server::User | Server::Lid | Server::Hosted | Server::HostedLid | Server::Interop)
            || matches!(&value.server, Server::Other(server) if matches!(server.as_str(), "bot" | "status_me" | "lid_me")))
        && !(value.server == Server::User && value.user == "gdpr"))
    .then(|| value.to_non_ad())
}

pub(super) fn ids(node: &Node) -> Result<Vec<String>> {
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
    // Cmk.A03 05be..05e0 expands only one sole list child, not extensions.
    if let [list] = node.child_nodes()
        && list.description == "list"
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
pub(super) fn receipt_kind(node: &Node) -> Option<&str> {
    // APK 37.71 0hs.A0M supplies a default only when the attribute is absent.
    let kind = match node.get_attr("type") {
        None => "delivery",
        Some(value) => value.as_text()?,
    };
    matches!(kind, "delivery" | "read" | "played").then_some(kind)
}

/// Supplied root receipt time, separate from local observation/admission.
pub(super) fn server_timestamp(node: &Node) -> Option<crate::model::time::Timestamp> {
    // APK 37.71 Cmk.A03 04e0..04fe -> 0Gv.A01: Java signed decimal
    // seconds, then wrapping mul-long 1000. Preserve missing/invalid as
    // unknown in public facts; the native device-cache path defaults to zero.
    node.get_attr("t")
        .and_then(|v| v.as_text())
        .and_then(super::mex::java_decimal_long)
        .map(|seconds| crate::model::time::Timestamp(seconds.wrapping_mul(1000)))
}

/// Persist supported direct receipt facts, matching sends by peer AND wire ID.
/// Group participant facts use group_receipts; unsupported/self routes stay raw.
pub fn record(store: &ProtocolStore, node: &Node) -> Result<usize> {
    if node.description != "receipt" || recipient(node).is_some() {
        return Ok(0);
    }
    if let Some(participants) = node
        .child_nodes()
        .first()
        .filter(|n| n.description == "participants")
    {
        let receipts = if participants.get_attr("message_id").is_some() {
            offline_aggregate(node, participants)?
        } else {
            let local_ms = crate::model::time::Timestamp::now().as_millis();
            let seconds = store.read(|tx| Ok(super::clock::receipt_now(tx, local_ms)? / 1000))?;
            key_aggregate(node, participants, seconds)?
        };
        return store.write(|tx| {
            receipts
                .iter()
                .try_fold(0, |matched, receipt| Ok(matched + record_in(tx, receipt)?))
        });
    }
    store.write(|tx| record_in(tx, node))
}

fn offline_aggregate(node: &Node, participants: &Node) -> Result<Vec<Node>> {
    let invalid = |reason| Error::invalid("receipt.aggregate", reason);
    let id = participants
        .get_attr("message_id")
        .and_then(|v| v.as_text())
        .filter(|id| !id.is_empty() && id.len() <= 128)
        .ok_or_else(|| invalid("missing or invalid message_id"))?;
    let from = jid(node, "from").ok_or_else(|| invalid("missing or invalid from"))?;
    let crate::binary::NodeContent::Children(items) = &participants.content else {
        return Err(invalid("missing participant list"));
    };
    if items.len() > crate::profile::Limits::default().max_children as usize {
        return Err(invalid("participant list exceeds bound"));
    }
    let mut receipts = Vec::with_capacity(items.len());
    for item in items {
        let actor =
            jid(item, "jid").ok_or_else(|| invalid("missing or invalid participant jid"))?;
        let kind = match item.get_attr("type") {
            None => "delivery",
            Some(value) => value
                .as_text()
                .ok_or_else(|| invalid("invalid participant type"))?,
        };
        if !matches!(
            kind,
            "delivery" | "read" | "played" | "sender" | "inactive" | "read-self" | "played-self"
        ) {
            return Err(invalid("unknown participant type"));
        }
        // Cmk.A03 00aa..00be uses required 0hs.A07, not root's lenient time.
        let time = item
            .get_attr("t")
            .and_then(|v| v.as_text())
            .and_then(super::mex::java_decimal_long)
            .ok_or_else(|| invalid("missing or invalid participant t"))?;
        if matches!(kind, "delivery" | "read" | "played") {
            receipts.push(
                Node::new("receipt")
                    .attr("from", from.clone())
                    .attr("id", id)
                    .attr("participant", actor)
                    .attr("type", kind)
                    .attr("t", time.to_string()),
            );
        }
    }
    Ok(receipts)
}

fn key_aggregate(node: &Node, participants: &Node, default_seconds: i64) -> Result<Vec<Node>> {
    let invalid = |reason| Error::invalid("receipt.aggregate", reason);
    let id = participants
        .get_attr("key")
        .and_then(|v| v.as_text())
        .filter(|id| !id.is_empty() && id.len() <= 128)
        .ok_or_else(|| invalid("missing or invalid key"))?;
    let from = jid(node, "from").ok_or_else(|| invalid("missing or invalid from"))?;
    let kind = match node.get_attr("type") {
        None => "delivery",
        Some(value) => value
            .as_text()
            .ok_or_else(|| invalid("invalid root type"))?,
    };
    if !matches!(
        kind,
        "delivery" | "read" | "played" | "sender" | "inactive" | "read-self" | "played-self"
    ) {
        return Err(invalid("unknown root type"));
    }
    let crate::binary::NodeContent::Children(items) = &participants.content else {
        return Err(invalid("missing participant list"));
    };
    if items.len() > crate::profile::Limits::default().max_children as usize {
        return Err(invalid("participant list exceeds bound"));
    }
    let mut receipts = Vec::with_capacity(items.len());
    for item in items {
        // Cmk.A03 -> 0hs.A0A: absent/unparseable JIDs skip even malformed t.
        let Some(actor) = network_jid(item, "jid") else {
            continue;
        };
        let time = match item.get_attr("t") {
            None => default_seconds,
            Some(value) => value
                .as_text()
                .and_then(super::mex::java_decimal_long)
                .ok_or_else(|| invalid("invalid participant t"))?,
        };
        if matches!(kind, "delivery" | "read" | "played") {
            receipts.push(
                Node::new("receipt")
                    .attr("from", from.clone())
                    .attr("id", id)
                    .attr("participant", actor)
                    .attr("type", kind)
                    .attr("t", time.to_string()),
            );
        }
    }
    Ok(receipts)
}

fn record_in(tx: &mut crate::store::WriteTx<'_>, node: &Node) -> Result<usize> {
    if node.description == "receipt"
        && jid(node, "from").is_some_and(|from| from.server == Server::Group)
    {
        return super::group_receipts::record_in(tx, node);
    }

    if node.description != "receipt" || recipient(node).is_some() {
        return Ok(0);
    }
    let Some(kind) = receipt_kind(node) else {
        return Ok(0);
    };
    if !matches!(kind, "delivery" | "read" | "played") {
        return Ok(0);
    }
    let Some(from) = jid(node, "from") else {
        return Ok(0);
    };
    if from.user.is_empty()
        || from.agent != 0
        || from.device != 0
        || !matches!(from.server, Server::User | Server::Lid)
        || !matches!(
            from.kind,
            crate::model::jid::JidKind::Plain | crate::model::jid::JidKind::AgentDevice
        )
    {
        return Ok(0);
    }
    // APK 37.71 Cmk.A03 051a..05bc retains remote chat separately from
    // DeviceJid.Companion.A00(participant). Never treat a device as the chat.
    let participant = network_jid(node, "participant");
    if let Some(actor) = participant.as_ref()
        && (actor.user.is_empty()
            || actor.agent != 0
            || !matches!(actor.server, Server::User | Server::Lid)
            || !matches!(
                actor.kind,
                crate::model::jid::JidKind::Plain | crate::model::jid::JidKind::AgentDevice
            ))
    {
        return Ok(0);
    }
    let actor = participant.as_ref().unwrap_or(&from);
    let ids = ids(node)?;
    let peer = from.to_non_ad();
    let mut matched = 0;
    for id in ids.iter().filter(|_| participant.is_none()) {
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
    // Keep the comparison-lab's PN-only records separate from normal
    // outbox intents. Cmk.A02/A03 keeps message chat and actual actor
    // distinct; only an already authenticated association may join them.
    let mut cursor = None;
    while participant.is_none() {
        let page = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if page.is_empty() {
            break;
        }
        cursor = page.last().map(|(key, _)| key.clone());
        for (key, _) in page {
            if !key.starts_with(b"message:") || !key.ends_with(b":send") {
                continue;
            }
            let Some(mut send): Option<serde_json::Value> =
                tx.get_json(Table::Checkpoints, &key, 1)?
            else {
                continue;
            };
            if !send["wire_id"]
                .as_str()
                .is_some_and(|v| ids.iter().any(|id| id == v))
            {
                continue;
            }
            let Some(phone) = send["to"]
                .as_str()
                .map(|v| v.strip_prefix('+').unwrap_or(v))
            else {
                continue;
            };
            if phone.is_empty() || !phone.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let original = Jid::parse(&format!("{phone}@s.whatsapp.net"))?;
            if !super::jid_map::authenticated_direct_chat(tx, &from, &original)? {
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
            tx.put_json(Table::Checkpoints, &key, 1, &send)?;
            matched += 1;
        }
    }
    let outbox = crate::outbox::Outbox::new("receipt-observer", crate::profile::Limits::default());
    let fact_kind = match kind {
        "read" => crate::model::send::DeliveryFactKind::Read,
        "played" => crate::model::send::DeliveryFactKind::Played,
        _ => crate::model::send::DeliveryFactKind::Delivered,
    };
    // The authenticated actor remains in its received namespace even
    // when it is associated with an original intent in the other one.
    let device = crate::model::ids::DeviceAddress::from_jid(actor)?;
    for id in &ids {
        let wire_id = crate::model::ids::MessageId::new(id.clone())
            .map_err(|_| Error::invalid("receipt", "invalid message id"))?;
        let Some(mut entry) = outbox.by_wire_id_in_write(tx, &wire_id)? else {
            continue;
        };
        if !super::jid_map::authenticated_direct_chat(tx, &from, entry.intent.chat.jid())? {
            continue;
        }
        if participant.is_some() {
            if !super::jid_map::authenticated_direct_chat(tx, &actor.to_non_ad(), &from)? {
                continue;
            }
            let Some(targets) = super::send_versions::targets_in(tx, entry.local_id)? else {
                continue;
            };
            let mut targeted = false;
            for target in targets {
                if actor.device == target.device
                    && super::jid_map::authenticated_direct_chat(
                        tx,
                        &actor.to_non_ad(),
                        &target.to_non_ad(),
                    )?
                {
                    targeted = true;
                    break;
                }
            }
            if !targeted {
                continue;
            }
        }
        super::send_versions::observe_in(tx, entry.local_id, actor, node)?;
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
                server_timestamp: server_timestamp(node),
                code: None,
            },
        )?;
        matched += 1;
    }
    Ok(matched)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplied_receipt_times_follow_java_signed_wrapping_conversion() {
        for (input, expected) in [
            (None, None),
            (Some(""), None),
            (Some("bad"), None),
            (Some(" 10"), None),
            (Some("9223372036854775808"), None),
            (Some("+２０"), Some(20000)),
            (Some("١٠"), Some(10000)),
            (Some("-1"), Some(-1000)),
            (Some("0"), Some(0)),
            (Some("9223372036854775807"), Some(-1000)),
            (Some("-9223372036854775808"), Some(0)),
        ] {
            let node = match input {
                Some(value) => Node::new("receipt").attr("t", value),
                None => Node::new("receipt"),
            };
            assert_eq!(server_timestamp(&node).map(|t| t.as_millis()), expected);
        }
    }

    #[test]
    fn receipt_fact_times_survive_reopen_without_rewriting_historical_or_duplicate_facts() {
        for kind in ["delivery", "read", "played"] {
            for input in [None, Some("bad"), Some("+２０"), Some("-1"), Some("0")] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("receipt-time.redb");
                let store = open(&path, true);
                let peer = "15555550100@s.whatsapp.net";
                let local = saved_receipt_send(&store, "clock", peer, "clock-fixture");
                let outbox = crate::outbox::Outbox::new("clock", crate::profile::Limits::default());
                let before = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
                let mut receipt = Node::new("receipt")
                    .attr("from", peer)
                    .attr("id", "clock-fixture")
                    .attr("type", kind);
                if let Some(value) = input {
                    receipt = receipt.attr("t", value);
                }
                assert_eq!(record(&store, &receipt).unwrap(), 1);
                let after = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
                assert_eq!(after.attempts, before.attempts);
                assert_eq!(after.progress, before.progress);
                assert_eq!(after.facts[0], before.facts[0]);
                assert_eq!(after.facts.len(), 2);
                let expected = match input {
                    Some("+２０") => Some(20000),
                    Some("-1") => Some(-1000),
                    Some("0") => Some(0),
                    _ => None,
                };
                assert_eq!(
                    after.facts[1].server_timestamp.map(|t| t.as_millis()),
                    expected
                );
                drop(store);
                let store = open(&path, false);
                assert_eq!(record(&store, &receipt.attr("t", "10")).unwrap(), 0);
                assert_eq!(
                    store.read(|tx| outbox.get(tx, local)).unwrap().unwrap(),
                    after
                );
            }
        }
    }
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
    fn receipt_outcomes_reject_corrupt_wire_without_exposing_or_changing_saved_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("receipt-inspection.redb");
        let mut store = open(&path, true);
        let encode = |node: &Node| {
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(node)
                .unwrap()
        };
        let valid = encode(&Node::new("receipt").attr("id", "INSPECT-FIXTURE"));
        let cases = [
            (serde_json::Value::Null, "missing saved receipt wire"),
            (serde_json::json!([256]), "invalid saved receipt wire bytes"),
            (
                serde_json::json!(["PRIVATE-CORRUPTION-FIXTURE"]),
                "invalid saved receipt wire bytes",
            ),
            (serde_json::json!([255]), "cannot decode saved receipt wire"),
            (
                serde_json::json!(encode(&Node::new("receipt"))),
                "missing or invalid saved receipt message id",
            ),
            (
                serde_json::json!(encode(
                    &Node::new("message").attr("id", "PRIVATE-CORRUPTION-FIXTURE")
                )),
                "saved receipt wire is not a receipt",
            ),
        ];
        let key = b"receipt-outbox:inspection-fixture:read";
        for (wire, reason) in cases {
            let row = serde_json::json!({"status":"outcome_unknown", "wire":wire});
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key, 1, &row))
                .unwrap();
            let error = outcomes(&store).unwrap_err().to_string();
            assert!(error.contains(reason), "{error}");
            assert!(!error.contains("PRIVATE-CORRUPTION-FIXTURE"));
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key, 1))
                    .unwrap(),
                Some(row)
            );
            drop(store);
            store = open(&path, false);
            assert_eq!(outcomes(&store).unwrap_err().to_string(), error);
        }
        let row = serde_json::json!({"status":"outcome_unknown", "wire":valid});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key, 1, &row))
            .unwrap();
        drop(store);
        let store = open(&path, false);
        let result = outcomes(&store).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message_id, "INSPECT-FIXTURE");
        assert_eq!(result[0].status, "outcome_unknown");
        assert_eq!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key, 1))
                .unwrap(),
            Some(row)
        );
    }

    #[test]
    fn retry_queue_is_once_per_route_and_survives_restart_and_uncertainty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("retry.redb");
        let store = open(&path, true);
        let account = crate::session::AccountRecord {
            signal_registration_id: Some(0x0102_0304),
            ..Default::default()
        };
        let incoming = Node::new("message")
            .attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap())
            .attr("id", "FAILED");
        assert_eq!(
            queue_retry_for_incoming(&store, &account, &incoming).unwrap(),
            1
        );
        assert_eq!(
            queue_retry_for_incoming(&store, &account, &incoming).unwrap(),
            0
        );
        drop(store);
        let store = open(&path, false);
        let pending = reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 1);
        assert!(
            matches!(&pending[0].child("registration").unwrap().content, crate::binary::NodeContent::Bytes(b) if b.as_ref() == [1, 2, 3, 4])
        );
        assert_eq!(
            pending[0]
                .child("retry")
                .unwrap()
                .get_attr("count")
                .and_then(|v| v.as_text()),
            Some("1")
        );
        assert_eq!(outcomes(&store).unwrap()[0].kind, "retry");
        assert_eq!(outcomes(&store).unwrap()[0].status, "outcome_unknown");
        drop(store);
        let store = open(&path, false);
        // Changed ciphertext/metadata on redelivery cannot reset the request.
        assert_eq!(
            queue_retry_for_incoming(&store, &account, &incoming.clone().attr("t", "123")).unwrap(),
            0
        );
        assert!(reserve_pending(&store).unwrap().is_empty());
        let mut alternate = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        alternate.kind = crate::model::jid::JidKind::AgentDevice;
        assert_eq!(
            queue_retry_for_incoming(&store, &account, &incoming.clone().attr("from", alternate))
                .unwrap(),
            0,
            "wire encoding changes must not reserve a second retry"
        );
        let ack = Node::new("ack")
            .attr("class", "receipt")
            .attr("type", "retry")
            .attr("id", "FAILED")
            .attr("from", Jid::parse("15555550101@s.whatsapp.net").unwrap());
        assert!(!record_ack(&store, &ack).unwrap());
        let ack = ack.attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap());
        assert!(record_ack(&store, &ack).unwrap());
        assert_eq!(outcomes(&store).unwrap()[0].status, "server_acknowledged");
        assert_eq!(
            queue_retry_for_incoming(&store, &account, &incoming).unwrap(),
            0
        );
        // Same ID, different authenticated sender: a different request.
        assert_eq!(
            queue_retry_for_incoming(
                &store,
                &account,
                &incoming.attr("from", Jid::parse("15555550101@s.whatsapp.net").unwrap())
            )
            .unwrap(),
            1
        );
    }

    #[test]
    fn historical_retry_encoding_selectors_preserve_all_saved_outcomes() {
        use crate::model::jid::JidKind;
        // Independent Python hashlib fixture over a1d4c05's serde JSON tuple:
        // group AgentDevice/0 + LID AgentDevice/3 + LEGACY-GROUP. Synthetic IDs.
        const LEGACY: &str =
            "receipt-outbox:d6ec9bc1c5e86f1775f25cf89eb3344ab4f2c63a4318533e12d3a7995a15a5c8:retry";
        for status in [
            "pending",
            "outcome_unknown",
            "refused",
            "server_acknowledged",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("legacy.redb");
            let store = open(&path, true);
            let mut chat = Jid::parse("120363000000000001@g.us").unwrap();
            chat.kind = JidKind::AgentDevice;
            let member = Jid::parse("123456:3@lid").unwrap();
            assert_eq!(
                retry_selector(&chat, &Some(member.clone()), "LEGACY-GROUP").unwrap(),
                LEGACY
            );
            let retry = super::super::messaging::retry_receipt(
                &chat,
                Some(&member),
                "LEGACY-GROUP",
                1,
                31,
                1,
            )
            .unwrap();
            let wire = crate::binary::Encoder::new(
                crate::binary::TokenDictionary::comparison_2_26_37_71(),
            )
            .encode(&retry)
            .unwrap();
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        LEGACY.as_bytes(),
                        1,
                        &serde_json::json!({"status":status,"wire":wire}),
                    )
                })
                .unwrap();
            drop(store);
            let store = open(&path, false);
            let before = store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap();
            let account = crate::session::AccountRecord {
                signal_registration_id: Some(31),
                ..Default::default()
            };
            for chat_kind in [
                JidKind::Plain,
                JidKind::AgentDevice,
                JidKind::Interop,
                JidKind::Facebook,
            ] {
                for member_kind in [
                    JidKind::Plain,
                    JidKind::AgentDevice,
                    JidKind::Interop,
                    JidKind::Facebook,
                ] {
                    let mut c = chat.clone();
                    c.kind = chat_kind;
                    let mut m = member.clone();
                    m.kind = member_kind;
                    let node = Node::new("message")
                        .attr("from", c)
                        .attr("participant", m)
                        .attr("id", "LEGACY-GROUP")
                        .attr("t", "changed");
                    assert_eq!(
                        queue_retry_for_incoming(&store, &account, &node).unwrap(),
                        0
                    );
                }
            }
            assert_eq!(
                store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap(),
                before
            );
            let other = Node::new("message")
                .attr("from", chat)
                .attr("participant", member.with_device(4))
                .attr("id", "LEGACY-GROUP");
            assert_eq!(
                queue_retry_for_incoming(&store, &account, &other).unwrap(),
                1,
                "different device must remain separate"
            );
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        LEGACY.as_bytes(),
                        1
                    ))
                    .unwrap()
                    .unwrap()["status"],
                status
            );
        }
    }

    #[test]
    fn receipt_ack_ignores_encoding_but_preserves_agent_device_and_server() {
        use crate::model::jid::JidKind;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("ack-kind.redb"), true);
        let account = crate::session::AccountRecord {
            signal_registration_id: Some(31),
            ..Default::default()
        };
        let mut chat = Jid::parse("120363000000000001@g.us").unwrap();
        chat.kind = JidKind::AgentDevice;
        let member = Jid::parse("123456:3@lid").unwrap();
        let incoming = Node::new("message")
            .attr("from", chat.clone())
            .attr("participant", member.clone())
            .attr("id", "ACK-KIND");
        assert_eq!(
            queue_retry_for_incoming(&store, &account, &incoming).unwrap(),
            1
        );
        assert_eq!(reserve_pending(&store).unwrap().len(), 1);
        let ack = Node::new("ack")
            .attr("class", "receipt")
            .attr("type", "retry")
            .attr("from", chat.clone())
            .attr("participant", member.clone())
            .attr("id", "ACK-KIND");
        let mut missing = ack.clone();
        missing
            .attributes
            .retain(|attribute| attribute.name != "participant");
        assert!(!record_ack(&store, &missing).unwrap());
        assert_eq!(outcomes(&store).unwrap()[0].status, "outcome_unknown");
        assert!(reserve_pending(&store).unwrap().is_empty());
        let mut other_agent = member.clone();
        other_agent.agent = 1;
        let mut other_server = member.clone();
        other_server.server = Server::User;
        for wrong in [member.with_device(4), other_agent, other_server] {
            assert!(!record_ack(&store, &ack.clone().attr("participant", wrong)).unwrap());
        }
        chat.kind = JidKind::Facebook;
        let mut equivalent = member;
        equivalent.kind = JidKind::Plain;
        assert!(
            record_ack(
                &store,
                &ack.attr("from", chat).attr("participant", equivalent)
            )
            .unwrap()
        );
        assert_eq!(outcomes(&store).unwrap()[0].status, "server_acknowledged");
        assert_eq!(
            queue_retry_for_incoming(&store, &account, &incoming).unwrap(),
            0
        );
    }

    #[test]
    fn retry_requires_registration_and_supported_routing_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("invalid-retry.redb"), true);
        let account = crate::session::AccountRecord::default();
        let incoming = Node::new("message")
            .attr("id", "FAILED")
            .attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap());
        assert!(queue_retry_for_incoming(&store, &account, &incoming).is_err());
        let account = crate::session::AccountRecord {
            signal_registration_id: Some(7),
            ..Default::default()
        };
        for invalid in [
            Node::new("receipt"),
            Node::new("message").attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            incoming
                .clone()
                .attr("from", Jid::parse("status@broadcast").unwrap()),
            incoming
                .clone()
                .attr("from", Jid::parse("123-456@g.us").unwrap()),
            incoming.clone().attr("participant", "invalid"),
        ] {
            assert!(queue_retry_for_incoming(&store, &account, &invalid).is_err());
        }
        assert!(outcomes(&store).unwrap().is_empty());
        let group = incoming.attr("from", Jid::parse("123-456@g.us").unwrap());
        for participant in ["111111111111111@lid", "222222222222222@lid"] {
            assert_eq!(
                queue_retry_for_incoming(
                    &store,
                    &account,
                    &group
                        .clone()
                        .attr("participant", Jid::parse(participant).unwrap())
                )
                .unwrap(),
                1
            );
        }
        assert_eq!(reserve_pending(&store).unwrap().len(), 2);
    }

    #[test]
    fn opened_queue_survives_restart_and_never_replays_uncertain_receipts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("receipts.redb");
        let store = open(&path, true);
        let key = b"received:15555550100@s.whatsapp.net:fixture";
        let ticket = hex::encode(Sha256::digest(key));
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
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
    fn primary_lid_read_uses_device_wire_and_settles_plain_ack_after_reopen() {
        use crate::model::{
            ids::{ChatId, MessageId},
            jid::JidKind,
            receipt::{Receipt, ReceiptKind},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("primary-lid.redb");
        let store = open(&path, true);
        let peer = Jid::parse("100001@lid").unwrap();
        let incoming = Node::new("message")
            .attr("id", "LID-READ")
            .attr("from", peer.clone());
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&incoming)
                .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"received:100001@lid:LID-READ",
                    1,
                    &serde_json::json!({"status":"decrypted", "route":route}),
                )
            })
            .unwrap();
        let request = Receipt {
            chat: ChatId::from_jid(&peer).unwrap(),
            message_ids: vec![MessageId::new("LID-READ").unwrap()],
            from: None,
            kind: ReceiptKind::Read,
            server_timestamp: None,
            code: None,
        };
        assert_eq!(queue_read(&store, &request).unwrap(), 1);
        let pending = reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 1);
        let to = pending[0].get_attr("to").unwrap().as_jid().unwrap();
        assert_eq!((to.kind, to.agent, to.device), (JidKind::AgentDevice, 0, 0));
        drop(store);
        let store = open(&path, false);
        assert!(
            reserve_pending(&store).unwrap().is_empty(),
            "uncertain wire cannot replay"
        );
        let ack = Node::new("ack")
            .attr("class", "receipt")
            .attr("type", "read")
            .attr("id", "LID-READ")
            .attr("from", peer);
        assert!(record_ack(&store, &ack).unwrap());
        assert!(!record_ack(&store, &ack).unwrap());
        assert_eq!(outcomes(&store).unwrap()[0].status, "server_acknowledged");
        assert_eq!(queue_read(&store, &request).unwrap(), 0);
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"received:100001@lid:LID-READ", 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["route"], serde_json::json!(route));
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
            || crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71());
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

        // Restart, then reserve: one root ID and the native extra-ID list.
        drop(store);
        let store = open(&path, false);
        let pending = reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 1);
        let node = &pending[0];
        assert_eq!(node.description, "receipt");
        assert_eq!(jid(node, "to"), Some(peer.clone()));
        assert_eq!(
            node.get_attr("type").and_then(|v| v.as_text()),
            Some("read")
        );
        assert!(node.get_attr("t").is_none());
        assert!(node.get_attr("participant").is_none());
        let binary =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(node)
                .unwrap();
        assert_eq!(decode(&binary).unwrap(), *node);
        let root = node.get_attr("id").unwrap().as_text().unwrap().to_owned();
        let list = node.child("list").unwrap();
        assert_eq!(node.child_nodes().len(), 1);
        assert_eq!(list.child_nodes().len(), 1);
        assert_eq!(list.child_nodes()[0].description, "item");
        let extra = list.child_nodes()[0]
            .get_attr("id")
            .unwrap()
            .as_text()
            .unwrap()
            .to_owned();
        let mut ids = vec![root.as_str(), extra.as_str()];
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

        // Only the batch root ACK can settle all immutable per-ID reservations.
        let ack = |id: &str, kind: &str, from: &str| {
            Node::new("ack")
                .attr("id", id)
                .attr("class", "receipt")
                .attr("type", kind)
                .attr("from", from)
        };
        assert!(!record_ack(&store, &ack(&root, "played", "15555550100@s.whatsapp.net")).unwrap());
        assert!(!record_ack(&store, &ack(&root, "read", "15555550101@s.whatsapp.net")).unwrap());
        assert!(!record_ack(&store, &ack(&extra, "read", "15555550100@s.whatsapp.net")).unwrap());
        assert!(record_ack(&store, &ack(&root, "read", "15555550100@s.whatsapp.net")).unwrap());
        assert!(!record_ack(&store, &ack(&root, "read", "15555550100@s.whatsapp.net")).unwrap());
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
        assert_eq!(status("second"), "server_acknowledged");
    }

    fn pending_receipt(store: &ProtocolStore, key: &str, node: &Node) -> Vec<u8> {
        let wire =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(node)
                .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key.as_bytes(),
                    1,
                    &serde_json::json!({"status":"pending", "wire":wire}),
                )
            })
            .unwrap();
        wire
    }

    #[test]
    fn read_batches_keep_group_devices_and_other_receipt_types_separate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("read-batches.redb");
        let store = open(&path, true);
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let a = Jid::parse("222222222222222:4@lid").unwrap();
        let b = Jid::parse("222222222222222:5@lid").unwrap();
        let base = Node::new("receipt")
            .attr("to", group.clone())
            .attr("type", "read");
        let mut originals = Vec::new();
        for (i, node) in [
            base.clone().attr("id", "G1").attr("participant", a.clone()),
            base.clone().attr("id", "G2").attr("participant", a.clone()),
            base.clone().attr("id", "G3").attr("participant", b.clone()),
            base.clone().attr("id", "G4").attr("participant", b.clone()),
            Node::new("receipt")
                .attr("id", "D1")
                .attr("to", a.clone())
                .attr("type", "read"),
            Node::new("receipt")
                .attr("id", "D2")
                .attr("to", a.clone())
                .attr("type", "read"),
            base.clone()
                .attr("id", "PLAYED")
                .attr("participant", a.clone())
                .attr("type", "played"),
            base.attr("id", "RETRY")
                .attr("participant", a.clone())
                .attr("type", "retry")
                .children(vec![Node::new("retry")]),
        ]
        .iter()
        .enumerate()
        {
            let key = format!(
                "receipt-outbox:batch-{i}:{}",
                node.get_attr("type").unwrap().as_text().unwrap()
            );
            originals.push((key.clone(), pending_receipt(&store, &key, node)));
        }
        let head = store.journal_head().unwrap();
        let wire = reserve_pending(&store).unwrap();
        assert_eq!(wire.len(), 5);
        let batch = wire
            .iter()
            .find(|n| {
                n.get_attr("type").and_then(|v| v.as_text()) == Some("read")
                    && jid(n, "participant") == Some(a.clone())
            })
            .unwrap();
        assert_eq!(batch.get_attr("id").unwrap().as_text(), Some("G1"));
        assert_eq!(
            batch.child("list").unwrap().child_nodes()[0]
                .get_attr("id")
                .unwrap()
                .as_text(),
            Some("G2")
        );
        let mut ack = Node::new("ack")
            .attr("class", "receipt")
            .attr("id", "G1")
            .attr("type", "read")
            .attr("from", group);
        assert!(!record_ack(&store, &ack).unwrap());
        assert!(!record_ack(&store, &ack.clone().attr("participant", b)).unwrap());
        ack = ack.attr("participant", a);
        assert!(!record_ack(&store, &ack.clone().attr("id", "G2")).unwrap());
        assert!(record_ack(&store, &ack.clone().attr("error", "479")).unwrap());
        assert!(!record_ack(&store, &ack).unwrap());
        let before = outcomes(&store).unwrap();
        assert_eq!(before.len(), 8);
        assert_eq!(before.iter().filter(|o| o.status == "refused").count(), 2);
        assert_eq!(
            before
                .iter()
                .filter(|o| o.status == "outcome_unknown")
                .count(),
            6
        );
        for (key, original) in &originals {
            let row: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(row["wire"], serde_json::json!(original));
        }
        assert_eq!(store.journal_head().unwrap(), head);
        drop(store);
        let store = open(&path, false);
        assert!(reserve_pending(&store).unwrap().is_empty());
        assert_eq!(outcomes(&store).unwrap(), before);
    }

    #[test]
    fn read_batch_reservation_limit_counts_ids_and_singletons_keep_legacy_ack() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("limit.redb"), true);
        for i in 0..17 {
            pending_receipt(
                &store,
                &format!("receipt-outbox:limit-{i:02}:read"),
                &Node::new("receipt")
                    .attr("id", format!("ID{i:02}"))
                    .attr("to", Jid::parse("222222222222222@lid").unwrap())
                    .attr("type", "read"),
            );
        }
        let first = reserve_pending(&store).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].child("list").unwrap().child_nodes().len(), 15);
        assert_eq!(
            outcomes(&store)
                .unwrap()
                .iter()
                .filter(|o| o.status == "pending")
                .count(),
            1
        );
        let second = reserve_pending(&store).unwrap();
        assert_eq!(second.len(), 1);
        assert!(second[0].child("list").is_none());
        let ack = Node::new("ack")
            .attr("class", "receipt")
            .attr("type", "read")
            .attr("from", Jid::parse("222222222222222@lid").unwrap())
            .attr("id", second[0].get_attr("id").unwrap().clone());
        assert!(record_ack(&store, &ack).unwrap());
        assert_eq!(
            outcomes(&store)
                .unwrap()
                .iter()
                .filter(|o| o.status == "server_acknowledged")
                .count(),
            1
        );
        assert!(reserve_pending(&store).unwrap().is_empty());
    }

    #[test]
    fn corrupt_read_batch_state_rolls_back_reservation_and_ack_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("batch-corruption.redb"), true);
        let snapshot = || {
            store
                .read(|tx| {
                    Table::ALL
                        .iter()
                        .map(|t| tx.scan_after(*t, 1, None, 512))
                        .collect::<Result<Vec<_>>>()
                })
                .unwrap()
        };
        for i in 0..2 {
            pending_receipt(
                &store,
                &format!("receipt-outbox:corrupt-{i}:read"),
                &Node::new("receipt")
                    .attr("id", format!("ID{i}"))
                    .attr("to", Jid::parse("222222222222222@lid").unwrap())
                    .attr("type", "read"),
            );
        }
        let key = b"receipt-outbox:corrupt-1:read";
        store
            .write(|tx| {
                let mut row: serde_json::Value = tx.get_json(Table::Checkpoints, key, 1)?.unwrap();
                row["batch_ack_id"] = serde_json::json!("unexpected");
                tx.put_json(Table::Checkpoints, key, 1, &row)
            })
            .unwrap();
        let before = snapshot();
        let head = store.journal_head().unwrap();
        assert!(
            reserve_pending(&store)
                .unwrap_err()
                .to_string()
                .contains("pending receipt has batch correlation")
        );
        assert_eq!(snapshot(), before);
        assert_eq!(store.journal_head().unwrap(), head);
        store
            .write(|tx| {
                let mut row: serde_json::Value = tx.get_json(Table::Checkpoints, key, 1)?.unwrap();
                row.as_object_mut().unwrap().remove("batch_ack_id");
                tx.put_json(Table::Checkpoints, key, 1, &row)
            })
            .unwrap();
        let sent = reserve_pending(&store).unwrap();
        store
            .write(|tx| {
                let mut row: serde_json::Value = tx.get_json(Table::Checkpoints, key, 1)?.unwrap();
                row["batch_ack_id"] = serde_json::json!(999);
                tx.put_json(Table::Checkpoints, key, 1, &row)
            })
            .unwrap();
        let before = snapshot();
        assert!(
            outcomes(&store)
                .unwrap_err()
                .to_string()
                .contains("invalid saved batch correlation")
        );
        let ack = Node::new("ack")
            .attr("class", "receipt")
            .attr("type", "read")
            .attr("from", Jid::parse("222222222222222@lid").unwrap())
            .attr("id", sent[0].get_attr("id").unwrap().clone());
        assert!(
            record_ack(&store, &ack)
                .unwrap_err()
                .to_string()
                .contains("invalid saved batch correlation")
        );
        assert_eq!(snapshot(), before);
        assert_eq!(store.journal_head().unwrap(), head);
    }

    /// Automatic read receipts go only to content a person reads, in direct
    /// chats and groups, and only once.
    #[test]
    fn automatic_read_receipts_skip_what_nobody_reads() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("auto-read.redb"), true);
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let member = Jid::parse("15555550100@lid").unwrap();
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let encoder =
            || crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71());
        let group_msg = |id: &str| {
            Node::new("message")
                .attr("id", id)
                .attr("from", group.clone())
                .attr("participant", member.clone())
        };
        let direct_msg = |id: &str| {
            Node::new("message")
                .attr("id", id)
                .attr("from", peer.clone())
        };
        // Hand-assembled payloads: conversation (1), a sender-key distribution
        // alone (2), a protocol message (12) and a reaction (46, tag 0xf2 0x02).
        let text = vec![0x0a, 0x02, b'h', b'i'];
        let skdm = vec![0x12, 0x00];
        let protocol = vec![0x62, 0x00];
        let reaction = vec![0xf2, 0x02, 0x00];
        let history_notice = b"\xb2\x06\x0d\x12\x0b\x0a\x051@lid\x10\x0a\x18\x02".to_vec();
        let rows = [
            (format!("received:{group}:T"), group_msg("T"), text.clone()),
            (format!("received:{group}:K"), group_msg("K"), skdm),
            (format!("received:{group}:R"), group_msg("R"), reaction),
            (
                format!("received:{group}:H"),
                group_msg("H"),
                history_notice,
            ),
            (format!("received:{peer}:D"), direct_msg("D"), text),
            (format!("received:{peer}:P"), direct_msg("P"), protocol),
        ];
        store
            .write(|tx| {
                for (key, node, payload) in &rows {
                    let route = encoder().encode(node)?;
                    tx.put_json(
                        Table::Checkpoints,
                        key.as_bytes(),
                        1,
                        &serde_json::json!({"status":"decrypted", "route":route, "protobuf":payload}),
                    )?;
                }
                Ok(())
            })
            .unwrap();
        let queued = |node: &Node| queue_read_for_incoming(&store, node).unwrap();
        assert_eq!(queued(&group_msg("T")), 1);
        assert_eq!(queued(&group_msg("T")), 0, "once only");
        assert_eq!(queued(&group_msg("K")), 0, "a key distribution is not read");
        assert_eq!(queued(&group_msg("R")), 0, "a reaction is not read");
        assert_eq!(queued(&group_msg("H")), 0, "a history notice is not read");
        assert_eq!(queued(&direct_msg("D")), 1);
        assert_eq!(
            queued(&direct_msg("P")),
            0,
            "a protocol message is not read"
        );
        assert_eq!(queued(&direct_msg("missing")), 0);
        assert_eq!(queued(&Node::new("receipt").attr("id", "T")), 0);

        let pending = reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 2);
        let to_group = pending
            .iter()
            .find(|n| jid(n, "to") == Some(group.clone()))
            .unwrap();
        assert_eq!(
            jid(to_group, "participant"),
            Some(Jid::new_ad("15555550100", 0, 0, Server::Lid).unwrap())
        );
        assert!(pending.iter().any(|n| jid(n, "to") == Some(peer.clone())));
    }

    /// A group message's read receipt goes to the group and names the sender,
    /// like the group delivery receipt; its ack settles it only when it names
    /// the same member.
    #[test]
    fn group_read_receipts_name_the_sender_and_settle_on_a_matching_ack() {
        use crate::model::{
            ids::{ChatId, MessageId},
            receipt::{Receipt, ReceiptKind},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("group-read.redb"), true);
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let sender = Jid::parse("15555550100@lid").unwrap();
        let route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(
                    &Node::new("message")
                        .attr("id", "G1")
                        .attr("from", group.clone())
                        .attr("participant", sender.clone()),
                )
                .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("received:{group}:G1").as_bytes(),
                    1,
                    &serde_json::json!({"status":"decrypted", "route":route}),
                )
            })
            .unwrap();
        let receipt = Receipt {
            chat: ChatId::from_jid(&group).unwrap(),
            message_ids: vec![MessageId::new("G1").unwrap()],
            from: None,
            kind: ReceiptKind::Read,
            server_timestamp: None,
            code: None,
        };
        assert_eq!(queue_read(&store, &receipt).unwrap(), 1);
        let pending = reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 1);
        let node = &pending[0];
        assert_eq!(
            node.attributes
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>(),
            ["to", "id", "type", "participant"]
        );
        assert_eq!(jid(node, "to"), Some(group.clone()));
        assert_eq!(
            jid(node, "participant"),
            Some(Jid::new_ad("15555550100", 0, 0, Server::Lid).unwrap())
        );
        assert_eq!(
            node.get_attr("type").and_then(|v| v.as_text()),
            Some("read")
        );

        let ack = |participant: &Jid| {
            Node::new("ack")
                .attr("id", "G1")
                .attr("class", "receipt")
                .attr("type", "read")
                .attr("from", group.clone())
                .attr("participant", participant.clone())
        };
        let other = Jid::parse("15555550199@lid").unwrap();
        assert!(
            !record_ack(&store, &ack(&other)).unwrap(),
            "another member's ack"
        );
        let mut missing = ack(&sender);
        missing
            .attributes
            .retain(|attribute| attribute.name != "participant");
        assert!(!record_ack(&store, &missing).unwrap());
        assert_eq!(outcomes(&store).unwrap()[0].status, "outcome_unknown");
        drop(store);
        let store = open(&dir.path().join("group-read.redb"), false);
        assert_eq!(outcomes(&store).unwrap()[0].status, "outcome_unknown");
        assert!(reserve_pending(&store).unwrap().is_empty());
        assert!(!record_ack(&store, &missing.attr("error", "479")).unwrap());
        assert_eq!(outcomes(&store).unwrap()[0].status, "outcome_unknown");
        assert!(record_ack(&store, &ack(&sender)).unwrap());

        // A device-addressed or broadcast chat is still refused.
        let mut status = receipt.clone();
        status.chat = ChatId::from_jid(&Jid::parse("status@broadcast").unwrap()).unwrap();
        assert!(queue_read(&store, &status).is_err());
    }

    #[test]
    fn opened_receipts_page_history_and_keep_reservations_atomic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("receipt-pages.redb");
        let store = open(&path, true);
        let key = b"received:15555550100@s.whatsapp.net:fixture";
        let ticket = hex::encode(Sha256::digest(key));
        let encoder =
            || crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71());
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
                style: Default::default(),
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
                super::super::send_versions::capture_in(
                    tx,
                    ticket.local_id,
                    &[request.chat.jid().clone()],
                )?;
                Ok(ticket)
            })
            .unwrap();
        let receipt = Node::new("receipt")
            .attr("id", "fixture")
            .attr("type", "read")
            .attr("t", "+２０");
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
        assert_eq!(record(&store, &receipt.attr("t", "١٠")).unwrap(), 0);
        let versions = super::super::send_versions::inspect(&store, ticket.local_id).unwrap();
        assert_eq!(versions.device_receipts.unwrap()[0].timestamp, Some(10000));
        let status = store
            .read(|tx| outbox.status(tx, ticket.local_id))
            .unwrap()
            .unwrap();
        assert_eq!(status.facts.len(), 1);
        assert_eq!(status.facts[0].kind, DeliveryFactKind::Read);
        assert_eq!(
            status.facts[0].server_timestamp,
            Some(crate::model::time::Timestamp(20000))
        );
        assert!(!status.server_accepted());
    }
    fn saved_receipt_send(
        store: &ProtocolStore,
        namespace: &str,
        chat: &str,
        id: &str,
    ) -> crate::model::ids::LocalMessageId {
        use crate::model::{
            ids::{ChatId, ClientRequestId, MessageId},
            message::MessageContent,
            send::{AttemptProgress, DeliveryFact, DeliveryFactKind, SendRequest},
        };
        let outbox = crate::outbox::Outbox::new(namespace, crate::profile::Limits::default());
        store
            .write(|tx| {
                let (ticket, _) = outbox.enqueue(
                    tx,
                    &SendRequest {
                        client_request_id: ClientRequestId::new(id)?,
                        chat: ChatId::from_jid(&Jid::parse(chat)?)?,
                        content: MessageContent::Text {
                            body: "offline receipt fixture".into(),
                            quoted: None,
                            style: Default::default(),
                        },
                    },
                )?;
                let mut entry = outbox.get_mut(tx, ticket.local_id)?.unwrap();
                entry.wire_id = Some(MessageId::new(id)?);
                outbox.set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                outbox.record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::DeliveryUnknown,
                        device: None,
                        observed_at: crate::model::time::Timestamp(7),
                        server_timestamp: None,
                        code: None,
                    },
                )?;
                Ok(ticket.local_id)
            })
            .unwrap()
    }

    #[test]
    fn offline_aggregated_receipts_reject_malformed_batch_without_partial_facts() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("bad-aggregate.redb"), true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let id = saved_receipt_send(&store, "aggregate", &peer.to_string(), "fixture");
        store
            .write(|tx| {
                super::super::send_versions::capture_in(tx, id, std::slice::from_ref(&peer))
            })
            .unwrap();
        let good = Node::new("participant")
            .attr("jid", peer.clone())
            .attr("t", "20");
        let mut missing_time = good.clone();
        missing_time.attributes.retain(|a| a.name != "t");
        let mut missing_jid = good.clone();
        missing_jid.attributes.retain(|a| a.name != "jid");
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        for bad in [
            missing_time,
            missing_jid,
            good.clone().attr("t", ""),
            good.clone().attr("t", "SECRET-INVALID-TIME"),
            good.clone().attr("t", "9223372036854775808"),
            good.clone().attr("type", "UNKNOWN-SECRET-TYPE"),
            good.clone().attr("type", peer.clone()),
            good.clone()
                .attr("jid", "SECRET-BAD-JID:70000@s.whatsapp.net"),
        ] {
            let node = Node::new("receipt")
                .attr("from", peer.clone())
                .children(vec![
                    Node::new("participants")
                        .attr("message_id", "fixture")
                        .children(vec![good.clone(), bad]),
                ]);
            let error = record(&store, &node).unwrap_err().to_string();
            assert!(error.contains("receipt.aggregate"));
            assert!(!error.contains("SECRET") && !error.contains("15555550100"));
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
        for participants in [
            Node::new("participants").attr("message_id", "fixture"),
            Node::new("participants")
                .attr("message_id", "")
                .children(vec![good.clone()]),
            Node::new("participants")
                .attr("message_id", "fixture")
                .children(vec![good; 4097]),
        ] {
            assert!(
                record(
                    &store,
                    &Node::new("receipt")
                        .attr("from", peer.clone())
                        .children(vec![participants])
                )
                .is_err()
            );
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

    #[test]
    fn offline_aggregated_receipts_roll_back_late_authenticated_mapping_failure() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("late-aggregate.redb"), true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let id = saved_receipt_send(&store, "aggregate", &peer.to_string(), "fixture");
        store
            .write(|tx| {
                super::super::send_versions::capture_in(
                    tx,
                    id,
                    &[peer.clone(), peer.with_device(2)],
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    b"authenticated-lid:100001@lid",
                    1,
                    &serde_json::json!("SECRET-BAD-MAPPING"),
                )
            })
            .unwrap();
        let node = Node::new("receipt")
            .attr("from", peer.clone())
            .children(vec![
                Node::new("participants")
                    .attr("message_id", "fixture")
                    .children(vec![
                        Node::new("participant").attr("jid", peer).attr("t", "20"),
                        Node::new("participant")
                            .attr("jid", "100001:2@lid")
                            .attr("t", "30"),
                    ]),
            ]);
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        let error = record(&store, &node).unwrap_err().to_string();
        assert!(error.contains("invalid authenticated LID mapping"));
        assert!(!error.contains("SECRET"));
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
    fn key_aggregates_use_inner_key_root_kind_and_optional_target_times_once() {
        use crate::model::{send::DeliveryFactKind, time::Timestamp};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multi-target.redb");
        let store = open(&path, true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let id = saved_receipt_send(&store, "aggregate", &peer.to_string(), "inner-key");
        let envelope = saved_receipt_send(&store, "aggregate", &peer.to_string(), "envelope");
        store
            .write(|tx| {
                super::super::send_versions::capture_in(
                    tx,
                    id,
                    &[peer.clone(), peer.clone().with_device(2)],
                )?;
                super::super::send_versions::capture_in(tx, envelope, std::slice::from_ref(&peer))
            })
            .unwrap();
        super::super::clock::observe(
            &store,
            &Node::new("iq")
                .attr("type", "get")
                .attr("xmlns", "urn:xmpp:ping")
                .attr("t", "1700000000"),
        )
        .unwrap();
        let outbox = crate::outbox::Outbox::new("aggregate", crate::profile::Limits::default());
        let before = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
        let root_before = store
            .read(|tx| Ok(outbox.get(tx, envelope)?.unwrap()))
            .unwrap();
        let node = Node::new("receipt")
            .attr("from", peer)
            .attr("id", "envelope")
            .attr("type", "read")
            .attr("t", "999")
            .children(vec![
                Node::new("participants")
                    .attr("key", "inner-key")
                    .children(vec![
                        Node::new("participant")
                            .attr("jid", "15555550100:2@s.whatsapp.net")
                            .attr("type", "played")
                            .attr("t", "+٢٢"),
                        Node::new("participant").attr("jid", "15555550100@s.whatsapp.net"),
                        Node::new("participant").attr("t", "bad"),
                        Node::new("participant")
                            .attr("jid", "invalid")
                            .attr("t", "bad"),
                        Node::new("participant").attr("jid", "lid").attr("t", "bad"),
                        Node::new("participant")
                            .attr("jid", "broadcast")
                            .attr("t", "bad"),
                        Node::new("participant")
                            .attr("jid", "15555550100:3@s.whatsapp.net")
                            .attr("t", "10"),
                    ]),
            ]);
        let encoded =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
        let node = decode(&encoded).unwrap();
        assert_eq!(record(&store, &node).unwrap(), 2);
        let saved = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
        assert_eq!(saved.facts.len(), 3);
        assert_eq!(saved.facts[1].kind, DeliveryFactKind::Read);
        assert_eq!(saved.facts[1].server_timestamp, Some(Timestamp(22_000)));
        assert_eq!(saved.facts[1].device.as_ref().unwrap().device.0, 2);
        assert_eq!(saved.facts[2].kind, DeliveryFactKind::Read);
        assert!(
            (saved.facts[2].server_timestamp.unwrap().as_millis() - 1_700_000_000_000).abs() < 5000
        );
        assert_eq!(saved.attempts, before.attempts);
        assert_eq!(saved.progress, before.progress);
        assert_eq!(saved.intent, before.intent);
        assert_eq!(
            root_before,
            store
                .read(|tx| Ok(outbox.get(tx, envelope)?.unwrap()))
                .unwrap()
        );
        let head = store.journal_head().unwrap();
        drop(store);
        let store = open(&path, false);
        assert_eq!(record(&store, &node).unwrap(), 0);
        assert_eq!(
            saved,
            store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap()
        );
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn key_aggregates_reject_bad_fields_without_partial_facts() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("invalid-multi-target.redb"), true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let id = saved_receipt_send(&store, "aggregate", &peer.to_string(), "fixture");
        store
            .write(|tx| {
                super::super::send_versions::capture_in(tx, id, std::slice::from_ref(&peer))
            })
            .unwrap();
        let item = Node::new("participant")
            .attr("jid", peer.clone())
            .attr("t", "20");
        let good = Node::new("participants")
            .attr("key", "fixture")
            .children(vec![item.clone()]);
        let root = Node::new("receipt")
            .attr("from", peer)
            .attr("id", "fixture");
        let snapshot = || {
            store
                .read(|tx| {
                    Ok((
                        Table::ALL
                            .iter()
                            .map(|table| tx.scan(*table, 1, 100))
                            .collect::<Result<Vec<_>>>()?,
                        tx.journal_head()?,
                    ))
                })
                .unwrap()
        };
        let before = snapshot();
        for node in [
            root.clone()
                .children(vec![Node::new("participants").children(vec![item.clone()])]),
            root.clone().children(vec![
                Node::new("participants")
                    .attr("key", "")
                    .children(vec![item.clone()]),
            ]),
            root.clone()
                .children(vec![Node::new("participants").attr("key", "fixture")]),
            root.clone()
                .attr("type", "SECRET")
                .children(vec![good.clone()]),
            root.clone().children(vec![
                Node::new("participants")
                    .attr("key", "fixture")
                    .children(vec![item.clone(), item.clone().attr("t", "SECRET")]),
            ]),
            root.children(vec![
                Node::new("participants")
                    .attr("key", "fixture")
                    .children(vec![item.clone(), item.attr("t", "9223372036854775808")]),
            ]),
        ] {
            let error = record(&store, &node).unwrap_err().to_string();
            assert!(error.contains("receipt.aggregate"));
            assert!(!error.contains("SECRET"));
            assert_eq!(snapshot(), before);
        }
    }

    #[test]
    fn unsupported_aggregated_routes_never_use_root_id_as_a_receipt() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("key-aggregate.redb"), true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        saved_receipt_send(&store, "aggregate", &peer.to_string(), "fixture");
        let item = Node::new("participant")
            .attr("jid", peer.clone())
            .attr("t", "20");
        let base = Node::new("receipt")
            .attr("from", peer.clone())
            .attr("id", "fixture");
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        for node in [
            base.clone().attr("recipient", peer).children(vec![
                Node::new("participants")
                    .attr("message_id", "fixture")
                    .children(vec![item.clone()]),
            ]),
            base.clone().children(vec![
                Node::new("participants")
                    .attr("message_id", "fixture")
                    .children(vec![]),
            ]),
            base.children(vec![
                Node::new("participants")
                    .attr("message_id", "fixture")
                    .children(vec![item.attr("type", "read-self")]),
            ]),
        ] {
            assert_eq!(record(&store, &node).unwrap(), 0);
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

    #[test]
    fn offline_aggregated_receipts_use_inner_id_and_per_device_facts_after_reopen() {
        use crate::model::send::DeliveryFactKind;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aggregate.redb");
        let store = open(&path, true);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let id = saved_receipt_send(&store, "aggregate", &peer.to_string(), "fixture");
        store
            .write(|tx| {
                super::super::send_versions::capture_in(
                    tx,
                    id,
                    &[peer.clone(), peer.with_device(2)],
                )
            })
            .unwrap();
        let outbox = crate::outbox::Outbox::new("aggregate", crate::profile::Limits::default());
        let before = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
        // Cmk.A03's first-child message_id branch ignores root status/id/time.
        let item = |actor: &str, kind: &str, time: &str| {
            Node::new("participant")
                .attr("jid", actor)
                .attr("type", kind)
                .attr("t", time)
        };
        let node = Node::new("receipt")
            .attr("from", peer)
            .attr("id", "envelope-id")
            .attr("type", "read-self")
            .attr("t", "999")
            .children(vec![
                Node::new("participants")
                    .attr("message_id", "fixture")
                    .children(vec![
                        item("15555550100:2@s.whatsapp.net", "delivery", "20"),
                        item("15555550100:2@s.whatsapp.net", "delivery", "10"),
                        item("15555550100@s.whatsapp.net", "read", "+٢٢"),
                        item("15555550100:2@s.whatsapp.net", "played", "-1"),
                        item("15555550100:3@s.whatsapp.net", "delivery", "21"),
                        item("15555550101:2@s.whatsapp.net", "delivery", "21"),
                        item("15555550100:2@s.whatsapp.net", "read-self", "23"),
                    ]),
            ]);
        let encoded =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
        let node = decode(&encoded).unwrap();
        assert_eq!(record(&store, &node).unwrap(), 3);
        let saved = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
        assert_eq!(saved.facts.len(), 4);
        for (kind, device, time) in [
            (DeliveryFactKind::Delivered, 2, 20_000),
            (DeliveryFactKind::Read, 0, 22_000),
            (DeliveryFactKind::Played, 2, -1_000),
        ] {
            let fact = saved.facts.iter().find(|f| f.kind == kind).unwrap();
            assert_eq!(fact.device.as_ref().unwrap().device.0, device);
            assert_eq!(fact.server_timestamp.unwrap().as_millis(), time);
        }
        assert_eq!(saved.facts[0], before.facts[0]);
        assert_eq!(saved.attempts, before.attempts);
        assert_eq!(saved.progress, before.progress);
        let rows = super::super::send_versions::inspect(&store, id)
            .unwrap()
            .device_receipts
            .unwrap();
        assert_eq!(
            rows.iter()
                .map(|r| (r.device, r.timestamp))
                .collect::<Vec<_>>(),
            vec![(0, Some(22_000)), (2, Some(10_000))]
        );
        assert!(
            store
                .read(|tx| Ok(tx
                    .scan(Table::Checkpoints, 1, 100)?
                    .iter()
                    .all(|(key, _)| !key.starts_with(b"receipt-facts:"))))
                .unwrap()
        );
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

    #[test]
    fn direct_participant_receipts_require_original_targets_and_preserve_device_facts() {
        use crate::model::{ids::DeviceAddress, send::DeliveryFactKind};
        let pn = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        for (name, actor, snapshot, mapping, accepted) in [
            ("same", "15555550100:2@s.whatsapp.net", true, false, true),
            ("mapped", "100001:2@lid", true, true, true),
            ("unmapped", "100001:2@lid", true, false, false),
            (
                "wrong-device",
                "15555550100:3@s.whatsapp.net",
                true,
                false,
                false,
            ),
            (
                "wrong-user",
                "15555550101:2@s.whatsapp.net",
                true,
                false,
                false,
            ),
            (
                "no-snapshot",
                "15555550100:2@s.whatsapp.net",
                false,
                false,
                false,
            ),
            (
                "agent",
                "15555550100.1:2@s.whatsapp.net",
                true,
                false,
                false,
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("participant.redb");
            let store = open(&path, true);
            let id = saved_receipt_send(&store, name, &pn.to_string(), "fixture");
            store.write(|tx| {
                if snapshot { super::super::send_versions::capture_in(tx, id, &[pn.clone(), pn.with_device(2)])?; }
                if mapping { tx.put_json(Table::Checkpoints, b"authenticated-lid:100001@lid", 1, &pn)?; }
                tx.put_json(Table::Checkpoints, b"message:legacy:send", 1, &serde_json::json!({"to":"+15555550100","wire_id":"fixture","status":"outcome_unknown"}))
            }).unwrap();
            let outbox = crate::outbox::Outbox::new(name, crate::profile::Limits::default());
            let before = store
                .read(|tx| {
                    Ok((
                        tx.scan(Table::Checkpoints, 1, 100)?,
                        outbox.get(tx, id)?.unwrap(),
                        tx.journal_head()?,
                    ))
                })
                .unwrap();
            let base = Node::new("receipt")
                .attr("from", pn.clone())
                .attr("participant", actor)
                .attr("id", "fixture")
                .attr("t", "20");
            for (kind, expected_kind) in [
                ("delivery", DeliveryFactKind::Delivered),
                ("read", DeliveryFactKind::Read),
                ("played", DeliveryFactKind::Played),
            ] {
                let node = base.clone().attr("type", kind);
                assert_eq!(
                    record(&store, &node).unwrap(),
                    usize::from(accepted),
                    "{name}/{kind}"
                );
                if accepted {
                    let saved = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
                    let fact = saved
                        .facts
                        .iter()
                        .find(|f| f.kind == expected_kind)
                        .unwrap();
                    assert_eq!(
                        fact.device,
                        Some(DeviceAddress::from_jid(&Jid::parse(actor).unwrap()).unwrap())
                    );
                    assert_eq!(fact.server_timestamp.unwrap().as_millis(), 20_000);
                    assert_eq!(saved.progress, before.1.progress);
                    assert_eq!(saved.attempts, before.1.attempts);
                    assert_eq!(saved.intent, before.1.intent);
                    assert_eq!(saved.facts[0], before.1.facts[0]);
                    let head = store.read(|tx| tx.journal_head()).unwrap();
                    assert_eq!(record(&store, &node.attr("t", "10")).unwrap(), 0);
                    assert_eq!(head, store.read(|tx| tx.journal_head()).unwrap());
                }
            }
            let legacy: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, b"message:legacy:send", 1))
                .unwrap()
                .unwrap();
            assert!(legacy["receipts"].is_null());
            assert!(
                store
                    .read(|tx| Ok(tx
                        .scan(Table::Checkpoints, 1, 100)?
                        .iter()
                        .all(|(k, _)| !k.starts_with(b"receipt-facts:"))))
                    .unwrap()
            );
            if !accepted {
                assert_eq!(
                    before,
                    store
                        .read(|tx| Ok((
                            tx.scan(Table::Checkpoints, 1, 100)?,
                            outbox.get(tx, id)?.unwrap(),
                            tx.journal_head()?
                        )))
                        .unwrap()
                );
            }
            let saved = store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap();
            drop(store);
            let store = open(&path, false);
            assert_eq!(
                record(&store, &base.clone().attr("type", "read")).unwrap(),
                0
            );
            assert_eq!(
                saved,
                store.read(|tx| Ok(outbox.get(tx, id)?.unwrap())).unwrap()
            );
        }
    }

    #[test]
    fn direct_receipt_namespace_binding_preserves_actual_actor_and_uncertainty() {
        use crate::model::{
            ids::DeviceAddress,
            send::{AttemptProgress, DeliveryFactKind},
        };
        let pn = "15555550100@s.whatsapp.net";
        let lid = "100001@lid";
        for (name, original, actor, map, expected) in [
            ("pn", pn, pn, false, true),
            ("lid", lid, lid, false, true),
            ("mapped", pn, lid, true, true),
            ("reverse", lid, pn, true, true),
            ("zero-ad", pn, "15555550100:0@s.whatsapp.net", false, true),
            ("lid-zero-ad", lid, "100001:0@lid", false, true),
            ("mapped-zero-ad", pn, "100001:0@lid", true, true),
            ("unattributed", pn, lid, false, false),
            ("reverse-unattributed", lid, pn, false, false),
            ("foreign", pn, "15555550101@s.whatsapp.net", true, false),
            ("foreign-lid", lid, "100002@lid", true, false),
            ("companion", pn, "15555550100:1@s.whatsapp.net", true, false),
            ("agent", pn, "15555550100.1:0@s.whatsapp.net", true, false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("receipt.redb");
            let store = open(&path, true);
            let local_id = saved_receipt_send(&store, name, original, "fixture");
            let observer = crate::outbox::Outbox::new(
                "different-namespace",
                crate::profile::Limits::default(),
            );
            store
                .write(|tx| {
                    if map {
                        tx.put_json(
                            Table::Checkpoints,
                            b"authenticated-lid:100001@lid",
                            1,
                            &Jid::parse(pn)?,
                        )?;
                    }
                    // A directory alias alone must never grant receipt authority.
                    tx.put_json(Table::Checkpoints, b"lid:100001@lid", 1, &Jid::parse(pn)?)?;
                    tx.put_json(
                        Table::Checkpoints,
                        b"receipt-outbox:unchanged",
                        1,
                        &serde_json::json!({"status":"outcome_unknown", "wire":[1,2,3]}),
                    )
                })
                .unwrap();
            let before = store
                .read(|tx| observer.get(tx, local_id))
                .unwrap()
                .unwrap();
            let node = Node::new("receipt")
                .attr("from", actor)
                .attr("id", "fixture")
                .attr("type", "read");
            assert_eq!(
                record(&store, &node).unwrap(),
                usize::from(expected),
                "{name}"
            );
            assert_eq!(record(&store, &node).unwrap(), 0, "duplicate {name}");
            drop(store);
            let store = open(&path, false);
            let entry = store
                .read(|tx| observer.get(tx, local_id))
                .unwrap()
                .unwrap();
            assert_eq!(entry.intent, before.intent, "{name}");
            assert_eq!(entry.wire_id, before.wire_id, "{name}");
            assert_eq!(entry.progress, AttemptProgress::InFlight, "{name}");
            assert_eq!(entry.facts[0], before.facts[0], "{name}");
            assert_eq!(entry.facts.len(), if expected { 2 } else { 1 }, "{name}");
            if expected {
                assert_eq!(entry.facts[1].kind, DeliveryFactKind::Read);
                assert_eq!(
                    entry.facts[1].device,
                    Some(DeviceAddress::from_jid(&Jid::parse(actor).unwrap()).unwrap())
                );
            }
            let status = store
                .read(|tx| observer.status(tx, local_id))
                .unwrap()
                .unwrap();
            assert!(
                status.is_uncertain() && !status.server_accepted(),
                "a recipient receipt does not manufacture server ACK {name}"
            );
            let reservation: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, b"receipt-outbox:unchanged", 1))
                .unwrap()
                .unwrap();
            assert_eq!(
                reservation,
                serde_json::json!({"status":"outcome_unknown", "wire":[1,2,3]})
            );
        }
    }

    #[test]
    fn receipt_batch_refuses_ambiguous_wire_id_atomically_across_namespaces() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("ambiguous.redb"), true);
        saved_receipt_send(&store, "a", "15555550100@s.whatsapp.net", "a");
        saved_receipt_send(&store, "b", "15555550100@s.whatsapp.net", "duplicate");
        saved_receipt_send(&store, "c", "15555550101@s.whatsapp.net", "duplicate");
        store.write(|tx| tx.put_json(Table::Checkpoints, b"message:legacy:send", 1, &serde_json::json!({"to":"+15555550100","wire_id":"a","status":"outcome_unknown"}))).unwrap();
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        let node = Node::new("receipt")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "a")
            .children(vec![
                Node::new("list").children(vec![Node::new("item").attr("id", "duplicate")]),
            ]);
        let error = record(&store, &node).unwrap_err().to_string();
        assert!(error.contains("duplicate outbox wire id"));
        let after = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn receipt_mapping_corruption_is_redacted_and_rolls_back_legacy_facts() {
        let pn = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let invalid = [
            serde_json::json!("DO-NOT-ECHO"),
            serde_json::to_value(Jid::parse("15555550100:1@s.whatsapp.net").unwrap()).unwrap(),
            serde_json::to_value(Jid::parse("100002@lid").unwrap()).unwrap(),
        ];
        for value in invalid {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("map.redb"), true);
            saved_receipt_send(&store, "a", &pn.to_string(), "fixture");
            store.write(|tx| {
                tx.put_json(Table::Checkpoints, b"message:legacy:send", 1, &serde_json::json!({"to":"+15555550100","wire_id":"fixture","status":"outcome_unknown"}))?;
                tx.put_json(Table::Checkpoints, b"authenticated-lid:100001@lid", 1, &value)
            }).unwrap();
            let before = store
                .read(|tx| {
                    Ok((
                        tx.scan(Table::Checkpoints, 1, 100)?,
                        tx.scan(Table::Outbox, 1, 100)?,
                    ))
                })
                .unwrap();
            let node = Node::new("receipt")
                .attr("from", "100001@lid")
                .attr("id", "fixture");
            let error = record(&store, &node).unwrap_err().to_string();
            assert!(error.contains("invalid authenticated LID mapping"));
            assert!(
                !error.contains("DO-NOT-ECHO")
                    && !error.contains("15555550100")
                    && !error.contains("100002")
            );
            assert_eq!(
                before,
                store
                    .read(|tx| Ok((
                        tx.scan(Table::Checkpoints, 1, 100)?,
                        tx.scan(Table::Outbox, 1, 100)?
                    )))
                    .unwrap()
            );
        }
    }

    #[test]
    fn unparseable_optional_receipt_addresses_follow_native_null_semantics() {
        use crate::model::{ids::DeviceAddress, send::DeliveryFactKind};
        for peer in ["15555550100@s.whatsapp.net", "100001@lid"] {
            for name in ["participant", "recipient"] {
                for value in [
                    "unknown.example",
                    "15555550100@unknown.example",
                    "lid",
                    "@s.whatsapp.net",
                    "15555550100@c.us",
                ]
                .into_iter()
                .chain(
                    (name == "recipient")
                        .then_some([
                            "120363000000000001@g.us",
                            "status@broadcast",
                            "123@newsletter",
                            "s.whatsapp.net",
                            "gdpr@s.whatsapp.net",
                            "15555550100:2@s.whatsapp.net",
                            "100001:0@lid",
                        ])
                        .into_iter()
                        .flatten(),
                ) {
                    for (kind, fact_kind) in [
                        ("delivery", DeliveryFactKind::Delivered),
                        ("read", DeliveryFactKind::Read),
                        ("played", DeliveryFactKind::Played),
                    ] {
                        let dir = tempfile::tempdir().unwrap();
                        let path = dir.path().join("optional-address.redb");
                        let store = open(&path, true);
                        let local =
                            saved_receipt_send(&store, "optional", peer, "optional-fixture");
                        let outbox = crate::outbox::Outbox::new(
                            "optional",
                            crate::profile::Limits::default(),
                        );
                        let before = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
                        let node = Node::new("receipt")
                            .attr("from", peer)
                            .attr("id", "optional-fixture")
                            .attr("type", kind)
                            .attr("t", "20")
                            .attr(name, value);
                        let wire = crate::binary::Encoder::new(
                            crate::binary::TokenDictionary::comparison_2_26_37_71(),
                        )
                        .encode(&node)
                        .unwrap();
                        let node = decode(&wire).unwrap();
                        assert_eq!(
                            record(&store, &node).unwrap(),
                            1,
                            "{peer}/{name}/{value}/{kind}"
                        );
                        let after = store.read(|tx| outbox.get(tx, local)).unwrap().unwrap();
                        assert_eq!(after.intent, before.intent);
                        assert_eq!(after.attempts, before.attempts);
                        assert_eq!(after.progress, before.progress);
                        assert_eq!(after.facts[0], before.facts[0]);
                        assert_eq!(after.facts.len(), before.facts.len() + 1);
                        let fact = after.facts.last().unwrap();
                        assert_eq!(fact.kind, fact_kind);
                        assert_eq!(
                            fact.device,
                            Some(DeviceAddress::from_jid(&Jid::parse(peer).unwrap()).unwrap())
                        );
                        assert_eq!(fact.server_timestamp.unwrap().as_millis(), 20_000);
                        let head = store.journal_head().unwrap();
                        drop(store);
                        let store = open(&path, false);
                        assert_eq!(record(&store, &node).unwrap(), 0);
                        assert_eq!(store.journal_head().unwrap(), head);
                        assert_eq!(
                            store.read(|tx| outbox.get(tx, local)).unwrap().unwrap(),
                            after
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn unsupported_receipt_routes_and_self_statuses_leave_the_store_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("unsupported.redb"), true);
        saved_receipt_send(&store, "a", "15555550100@s.whatsapp.net", "fixture");
        let node = Node::new("receipt")
            .attr("id", "fixture")
            .attr("from", "15555550100@s.whatsapp.net");
        let mut unsupported = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        unsupported.kind = crate::model::jid::JidKind::Interop;
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        for receipt in [
            node.clone().attr("type", "read-self"),
            node.clone().attr("type", "played-self"),
            node.clone().attr("type", "sender"),
            node.clone().attr("type", "inactive"),
            node.clone().attr("type", "unrecognized"),
            node.clone().attr("from", unsupported),
            node.clone().attr("from", "100001@g.us"),
            node.clone()
                .attr("participant", "15555550100@s.whatsapp.net"),
            node.clone().attr(
                "recipient",
                Jid::new_ad("15555550100", 0, 0, Server::User).unwrap(),
            ),
            node.attr("recipient", "15555550100@s.whatsapp.net"),
        ] {
            assert_eq!(record(&store, &receipt).unwrap(), 0);
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

    #[test]
    fn present_nontext_receipt_type_never_defaults_to_delivery_after_wire_decode() {
        use crate::binary::{AttributeValue, Encoder, TokenDictionary};
        use crate::model::send::DeliveryFactKind;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("receipt-type.redb");
        let store = open(&path, true);
        let local_id = saved_receipt_send(&store, "a", "15555550100@s.whatsapp.net", "fixture");
        let key = b"message:legacy:send";
        store.write(|tx| tx.put_json(Table::Checkpoints, key, 1, &serde_json::json!({"to":"+15555550100","wire_id":"fixture","status":"outcome_unknown"}))).unwrap();
        let node = Node::new("receipt")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "fixture");
        let before = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Checkpoints, 1, 100)?,
                    tx.scan(Table::Outbox, 1, 100)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap();
        let dictionary = TokenDictionary::comparison_2_26_37_71();
        for value in [
            AttributeValue::Jid(Jid::parse("100001@lid").unwrap()),
            AttributeValue::Bytes(bytes::Bytes::from_static(&[0xff, 0xfe])),
            AttributeValue::Text(String::new()),
        ] {
            let wire = Encoder::new(dictionary)
                .encode(&node.clone().attr("type", value))
                .unwrap();
            let decoded = decode(&wire).unwrap();
            assert!(decoded.get_attr("type").is_some());
            assert_eq!(record(&store, &decoded).unwrap(), 0);
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
        let wire = Encoder::new(dictionary).encode(&node).unwrap();
        let decoded = decode(&wire).unwrap();
        assert!(decoded.get_attr("type").is_none());
        assert_eq!(record(&store, &decoded).unwrap(), 2);
        drop(store);
        let store = open(&path, false);
        let outbox = crate::outbox::Outbox::new("a", crate::profile::Limits::default());
        let entry = store.read(|tx| outbox.get(tx, local_id)).unwrap().unwrap();
        assert_eq!(entry.facts.len(), 2);
        assert_eq!(entry.facts[1].kind, DeliveryFactKind::Delivered);
        assert_eq!(entry.facts[0].kind, DeliveryFactKind::DeliveryUnknown);
        let legacy: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key, 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            legacy["receipts"],
            serde_json::json!({"delivered":true,"read":false,"played":false})
        );
        assert_eq!(legacy["status"], "outcome_unknown");
    }

    #[test]
    fn legacy_receipt_pages_history_without_treating_lid_digits_as_phone() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("pages.redb"), true);
        let key = b"message:legacy:send";
        store.write(|tx| {
            for i in 0..10_000 { tx.put(Table::Checkpoints, format!("a-history:{i:05}").as_bytes(), 1, b"opaque")?; }
            tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"to":"+15555550100","wire_id":"fixture","status":"outcome_unknown"}))
        }).unwrap();
        let node = Node::new("receipt")
            .attr("id", "fixture")
            .attr("from", "15555550100@lid");
        assert_eq!(record(&store, &node).unwrap(), 0);
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"authenticated-lid:15555550100@lid",
                    1,
                    &Jid::parse("15555550100@s.whatsapp.net")?,
                )
            })
            .unwrap();
        assert_eq!(record(&store, &node).unwrap(), 1);
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key, 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["status"], "outcome_unknown");
        assert_eq!(
            saved["receipts"],
            serde_json::json!({"delivered":true,"read":false,"played":false})
        );
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
    fn receipt_lists_only_expand_the_single_native_list_branch() {
        let list = || Node::new("list").children(vec![Node::new("item").attr("id", "extra")]);
        for children in [
            vec![list(), list()],
            vec![Node::new("biz"), list()],
            vec![list(), Node::new("biz")],
            vec![Node::new("unknown"), list()],
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("list-shape.redb");
            let store = open(&path, true);
            let peer = "15555550100@s.whatsapp.net";
            let root = saved_receipt_send(&store, "list-shape", peer, "root");
            let extra = saved_receipt_send(&store, "list-shape", peer, "extra");
            let outbox = crate::outbox::Outbox::new("inspect", crate::profile::Limits::default());
            let before = store.read(|tx| outbox.get(tx, extra)).unwrap().unwrap();
            let receipt = Node::new("receipt")
                .attr("id", "root")
                .attr("from", peer)
                .attr("type", "read")
                .attr("t", "20")
                .children(children);
            assert_eq!(record(&store, &receipt).unwrap(), 1);
            assert_eq!(
                store.read(|tx| outbox.get(tx, extra)).unwrap().unwrap(),
                before
            );
            assert_eq!(
                store
                    .read(|tx| outbox.get(tx, root))
                    .unwrap()
                    .unwrap()
                    .facts
                    .len(),
                2
            );
            let head = store.read(|tx| tx.journal_head()).unwrap();
            drop(store);
            let store = open(&path, false);
            assert_eq!(record(&store, &receipt).unwrap(), 0);
            assert_eq!(store.read(|tx| tx.journal_head()).unwrap(), head);
            assert_eq!(
                store.read(|tx| outbox.get(tx, extra)).unwrap().unwrap(),
                before
            );
            // The same saved extra ID is admitted when the actual single-list
            // branch selects it; ignored extensions cannot invent that fact.
            assert_eq!(record(&store, &receipt.children(vec![list()])).unwrap(), 1);
        }
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

/// A recipient device asking us to send a message again because it could not
/// decrypt it (`receipt type="retry"`).
///
/// **Observed** live 2026-09-24: `<receipt from=<chat> type="retry" id=<message
/// id> participant=<device> t=…>` with a `<retry count v id t/>` child and a
/// `<registration>` child holding the sender's current registration id. Two of
/// three addressed group devices returned nothing and only this one asked, which
/// is what makes a retry per-device rather than a verdict on the stanza.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryRequest {
    /// The chat the failed message was in — a group, broadcast or direct peer.
    pub chat: Jid,
    /// The device that could not decrypt and wants it again.
    pub device: Jid,
    /// The original message id, which the resend must reuse so the recipient
    /// matches it to the message it is holding.
    pub message_id: String,
    /// Which attempt this is; the first is 1.
    pub count: u32,
    /// The asking device's current registration id. A value different from the
    /// one in the bundle we encrypted against means their identity moved and a
    /// fresh prekey bundle is needed rather than the saved session.
    pub registration: Option<Vec<u8>>,
}

/// Parse a `receipt type="retry"`. Returns `None` for any other receipt.
pub fn parse_retry(node: &Node) -> Option<RetryRequest> {
    if node.description != "receipt" || node.get_attr("type")?.as_text()? != "retry" {
        return None;
    }
    let jid = |name: &str| -> Option<Jid> {
        let value = node.get_attr(name)?;
        value
            .as_jid()
            .cloned()
            .or_else(|| Jid::parse(value.as_text()?).ok())
    };
    let message_id = node.get_attr("id")?.as_text()?.to_owned();
    if message_id.is_empty() || message_id.len() > 128 {
        return None;
    }
    let retry = node.child("retry")?;
    let count = retry.get_attr("count")?.as_text()?.parse().ok()?;
    let registration = node.child("registration").and_then(|n| match &n.content {
        crate::binary::NodeContent::Bytes(b) if !b.is_empty() && b.len() <= 64 => Some(b.to_vec()),
        _ => None,
    });
    Some(RetryRequest {
        chat: jid("from")?,
        // A direct retry names no participant; the peer is the chat itself.
        device: jid("participant").or_else(|| jid("from"))?,
        message_id,
        count,
        registration,
    })
}

#[cfg(test)]
mod retry_tests {
    use super::*;

    fn receipt(with_participant: bool) -> Node {
        let mut node = Node::new("receipt")
            .attr("from", Jid::parse("120363000000000001@g.us").unwrap())
            .attr("type", "retry")
            .attr("id", "M1");
        if with_participant {
            node = node.attr("participant", Jid::parse("222222222222222@lid").unwrap());
        }
        node.children(vec![
            Node::new("retry").attr("count", "1").attr("v", "1"),
            Node::new("registration").bytes(vec![1, 2, 3, 4]),
        ])
    }

    /// The fields a resend needs: which device, which message, and the
    /// registration id that says whether the saved session is still theirs.
    #[test]
    fn a_group_retry_names_the_device_and_the_message() {
        let parsed = parse_retry(&receipt(true)).unwrap();
        assert_eq!(parsed.message_id, "M1");
        assert_eq!(parsed.count, 1);
        assert_eq!(parsed.registration.as_deref(), Some(&[1, 2, 3, 4][..]));
        assert_eq!(parsed.device, Jid::parse("222222222222222@lid").unwrap());
        assert_eq!(parsed.chat, Jid::parse("120363000000000001@g.us").unwrap());
    }

    /// A direct retry carries no participant, so the peer is the chat.
    #[test]
    fn a_direct_retry_falls_back_to_the_sender() {
        let node = Node::new("receipt")
            .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            .attr("type", "retry")
            .attr("id", "M2")
            .children(vec![Node::new("retry").attr("count", "2")]);
        let parsed = parse_retry(&node).unwrap();
        assert_eq!(parsed.device, parsed.chat);
        assert_eq!(parsed.count, 2);
        assert!(parsed.registration.is_none());
    }

    /// Anything that is not a retry must not be mistaken for one — a delivery
    /// receipt answered as a retry would resend a message nobody asked for.
    #[test]
    fn other_receipts_are_not_retries() {
        assert!(parse_retry(&Node::new("receipt").attr("id", "M1")).is_none());
        assert!(
            parse_retry(
                &Node::new("receipt")
                    .attr("type", "read")
                    .attr("id", "M1")
                    .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            )
            .is_none()
        );
        // A retry with no `<retry>` child has no count and is not actionable.
        let no_child = Node::new("receipt")
            .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            .attr("type", "retry")
            .attr("id", "M1");
        assert!(parse_retry(&no_child).is_none());
    }
}
