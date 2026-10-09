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
/// A group receipt names the sender as `participant`, exactly as the group
/// delivery receipt does; that the read form is the same builder with
/// `type="read"` is **inferred**, not traced.
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
    // A group is allowed as well: its receipt is the same builder
    // (`received_confirmation`, `LX/Cp2`) with the sender as `participant`,
    // which the live-verified group delivery receipt already uses.
    if !matches!(chat.server, Server::User | Server::Lid | Server::Group) || chat.device != 0 {
        return Err(Error::invalid(
            "receipt",
            "only primary direct chats and groups are supported",
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
        let key = format!("received:{chat}:{id}");
        let Some(saved) = store
            .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))?
        else {
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
    queue_read(
        store,
        &Receipt {
            chat: ChatId::from_jid(&chat)?,
            message_ids: vec![MessageId::new(id.to_owned())?],
            from: None,
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
/// Every receipt ever queued, in key order. No recipient is printed.
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
        || ["recipient", "edit"]
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
                // An ack naming a participant settles only a receipt that named
                // the same one: in a group, the id alone is ambiguous between
                // members.
                if sent.get_attr("id") != node.get_attr("id")
                    || sent.get_attr("type") != node.get_attr("type")
                    || jid(&sent, "to").is_none()
                    || jid(&sent, "to").map(logical_address)
                        != jid(node, "from").map(logical_address)
                    || (node.get_attr("participant").is_some()
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
    let ids = ids(node)?;
    let peer = from.to_non_ad();
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
        // Keep the comparison-lab's PN-only records separate from normal
        // outbox intents. Cmk.A02/A03 keeps message chat and actual actor
        // distinct; only an already authenticated association may join them.
        let mut cursor = None;
        loop {
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
        let outbox =
            crate::outbox::Outbox::new("receipt-observer", crate::profile::Limits::default());
        let fact_kind = match kind {
            "read" => crate::model::send::DeliveryFactKind::Read,
            "played" => crate::model::send::DeliveryFactKind::Played,
            _ => crate::model::send::DeliveryFactKind::Delivered,
        };
        // The authenticated actor remains in its received namespace even
        // when it is associated with an original intent in the other one.
        let device = crate::model::ids::DeviceAddress::from_jid(&from)?;
        for id in &ids {
            let wire_id = crate::model::ids::MessageId::new(id.clone())
                .map_err(|_| Error::invalid("receipt", "invalid message id"))?;
            let Some(mut entry) = outbox.by_wire_id_in_write(tx, &wire_id)? else {
                continue;
            };
            if !super::jid_map::authenticated_direct_chat(tx, &from, entry.intent.chat.jid())? {
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
        assert_eq!(jid(to_group, "participant"), Some(member.clone()));
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
        assert_eq!(jid(node, "to"), Some(group.clone()));
        assert_eq!(jid(node, "participant"), Some(sender.clone()));
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
                        code: None,
                    },
                )?;
                Ok(ticket.local_id)
            })
            .unwrap()
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
