//! Who has received, viewed or reacted to a status we posted.
//!
//! **Observed** live 2026-09-24, after this client's first status post: the
//! server delivers `<receipt from=<viewer> id=<receipt id> [type] t=…>` with a
//! `<participants message_id=<status id>/>` child. The receipt's own `id` is the
//! receipt's, not the status's — the status is named by `participants/@message_id`,
//! which is what makes a status receipt different in shape from a chat one and
//! why it needs its own reader.
//!
//! A receipt with **no** `type` is a delivery; `read` is a view; `played` is a
//! watched video or listened voice status. That is the same convention the chat
//! receipts use, and the same one this client already refuses to *send* for
//! reads (`docs/protocol/messaging-transport.md`): recording that somebody
//! viewed our status is not the same as telling anybody we viewed theirs.
use crate::{
    Result,
    binary::Node,
    model::{jid::Jid, time::Timestamp},
    store::{ProtocolStore, Table, WriteTx},
};
use serde::{Deserialize, Serialize};

const PREFIX: &str = "status-receipt:";

/// What a viewer's device reported about one of our statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusReceiptKind {
    /// Their device has the status; nobody has necessarily looked at it.
    Delivered,
    /// Viewed.
    Read,
    /// Watched or listened to the end, for a video or voice status.
    Played,
}

impl StatusReceiptKind {
    fn from_attr(value: Option<&str>) -> Option<Self> {
        Some(match value {
            None => Self::Delivered,
            Some("read") | Some("read-self") => Self::Read,
            Some("played") => Self::Played,
            // `retry`, `sender`, `inactive` and anything new are not viewership
            // and must not be counted as one.
            Some(_) => return None,
        })
    }
}

/// One viewer's report about one status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusReceipt {
    /// The status we posted, by its wire id.
    pub status_id: String,
    /// Whose device reported.
    pub viewer: Jid,
    /// What they reported.
    pub kind: StatusReceiptKind,
    /// When we recorded it.
    pub at: Timestamp,
}

/// Parse a status receipt. Returns `None` for a chat receipt, a retry, or
/// anything whose shape we have not observed — a misread here would invent a
/// viewer.
///
/// The observed shape has `<participants message_id/>` with **no** entries.
/// With entries it is APK 37.71 Cmk.A03's offline aggregate for one of our chat
/// messages (`docs/protocol/offline-aggregated-receipts.md`), whose root `from`
/// is the chat, not a viewer; until 2026-10-13 this parser accepted that too and
/// would have filed a direct aggregate as a status view.
pub fn parse(node: &Node) -> Option<StatusReceipt> {
    if node.description != "receipt" {
        return None;
    }
    let participants = node.child("participants")?;
    if !matches!(participants.content, crate::binary::NodeContent::Empty) {
        return None;
    }
    let status_id = participants.get_attr("message_id")?.as_text()?.to_owned();
    if status_id.is_empty() || status_id.len() > 128 {
        return None;
    }
    let from = node.get_attr("from")?;
    let viewer = from
        .as_jid()
        .cloned()
        .or_else(|| Jid::parse(from.as_text()?).ok())?;
    if !viewer.is_user_like() {
        return None;
    }
    Some(StatusReceipt {
        status_id,
        viewer: viewer.to_non_ad(),
        kind: StatusReceiptKind::from_attr(node.get_attr("type").and_then(|v| v.as_text()))?,
        at: Timestamp::now(),
    })
}

fn key(status_id: &str, viewer: &Jid) -> String {
    format!("{PREFIX}{status_id}:{viewer}")
}

/// Record one status receipt, keeping the furthest state a viewer reached.
///
/// A delivery can arrive after a view when the server replays an offline queue,
/// and letting that overwrite the view would under-report. Returns whether
/// anything changed.
pub fn record(store: &ProtocolStore, receipt: &StatusReceipt) -> Result<bool> {
    let key = key(&receipt.status_id, &receipt.viewer);
    store.write(|tx| {
        let previous: Option<StatusReceipt> = tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
        let rank = |kind: StatusReceiptKind| match kind {
            StatusReceiptKind::Delivered => 0,
            StatusReceiptKind::Read => 1,
            StatusReceiptKind::Played => 2,
        };
        if previous
            .as_ref()
            .is_some_and(|p| rank(p.kind) >= rank(receipt.kind))
        {
            return Ok(false);
        }
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, receipt)?;
        Ok(true)
    })
}

/// Every recorded receipt, for one status or for all of them.
pub fn list(store: &ProtocolStore, status_id: Option<&str>) -> Result<Vec<StatusReceipt>> {
    let wanted = status_id.map(|id| format!("{PREFIX}{id}:"));
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 512)?;
            let done = rows.len() < 512;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, value) in rows {
                let Ok(name) = std::str::from_utf8(&key) else {
                    continue;
                };
                if !name.starts_with(PREFIX) {
                    continue;
                }
                if let Some(wanted) = &wanted
                    && !name.starts_with(wanted.as_str())
                {
                    continue;
                }
                if let Ok(receipt) = serde_json::from_slice::<StatusReceipt>(&value) {
                    out.push(receipt);
                }
            }
            if done {
                out.sort_by(|a, b| {
                    a.status_id
                        .cmp(&b.status_id)
                        .then_with(|| a.viewer.to_string().cmp(&b.viewer.to_string()))
                });
                return Ok(out);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };

    fn receipt(kind: Option<&str>) -> Node {
        let mut node = Node::new("receipt")
            .attr("from", Jid::parse("222222222222222@lid").unwrap())
            .attr("id", "SAGR0001");
        if let Some(kind) = kind {
            node = node.attr("type", kind);
        }
        node.children(vec![
            Node::new("participants").attr("message_id", "STATUS1"),
        ])
    }

    /// The status is named by `participants/@message_id`, not by the receipt's
    /// own id — reading the wrong one would file every viewer under the receipt.
    #[test]
    fn a_status_receipt_names_the_status_and_the_viewer() {
        let parsed = parse(&receipt(None)).unwrap();
        assert_eq!(parsed.status_id, "STATUS1");
        assert_eq!(parsed.viewer, Jid::parse("222222222222222@lid").unwrap());
        assert_eq!(parsed.kind, StatusReceiptKind::Delivered);
        assert_eq!(
            parse(&receipt(Some("read"))).unwrap().kind,
            StatusReceiptKind::Read
        );
        assert_eq!(
            parse(&receipt(Some("played"))).unwrap().kind,
            StatusReceiptKind::Played
        );
    }

    /// With participant entries it is a chat message's offline aggregate (APK
    /// 37.71 Cmk.A03), not a status view: the root `from` is the chat.
    #[test]
    fn an_aggregate_with_entries_is_not_a_status_view() {
        let aggregate = Node::new("receipt")
            .attr("from", Jid::parse("222222222222222@lid").unwrap())
            .attr("id", "AGG1")
            .children(vec![
                Node::new("participants")
                    .attr("message_id", "OUR-MESSAGE")
                    .children(vec![
                        Node::new("user")
                            .attr("jid", Jid::parse("222222222222222:2@lid").unwrap())
                            .attr("t", "1700000000"),
                    ]),
            ]);
        assert!(parse(&aggregate).is_none());
    }

    /// A retry is not viewership, and neither is a chat receipt with no
    /// `<participants>`; counting either would invent a viewer.
    #[test]
    fn non_viewership_receipts_are_refused() {
        assert!(parse(&receipt(Some("retry"))).is_none());
        assert!(parse(&receipt(Some("sender"))).is_none());
        assert!(
            parse(
                &Node::new("receipt")
                    .attr("from", Jid::parse("222222222222222@lid").unwrap())
                    .attr("id", "M1")
            )
            .is_none()
        );
        assert!(parse(&Node::new("message")).is_none());
    }

    /// A delivery replayed from an offline queue after a view must not demote
    /// the view.
    #[test]
    fn the_furthest_state_a_viewer_reached_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
            create: true,
        })
        .unwrap();
        assert!(record(&store, &parse(&receipt(Some("read"))).unwrap()).unwrap());
        assert!(
            !record(&store, &parse(&receipt(None)).unwrap()).unwrap(),
            "a replayed delivery must not overwrite a view"
        );
        let all = list(&store, None).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].kind, StatusReceiptKind::Read);
        assert!(record(&store, &parse(&receipt(Some("played"))).unwrap()).unwrap());
        assert_eq!(
            list(&store, Some("STATUS1")).unwrap()[0].kind,
            StatusReceiptKind::Played
        );
        assert!(list(&store, Some("OTHER")).unwrap().is_empty());
    }
}

const LIKE_PREFIX: &str = "status-like:";

/// Somebody's reaction to a status we posted — a "like".
///
/// A status reaction is an ordinary `Message.reactionMessage` (field 46) whose
/// key names `status@broadcast` as the `remoteJid`; that is what distinguishes
/// it from a reaction in a chat. In the APK it lands in `status_add_on` with
/// `type = 1`, which a trigger counts into `status_interactions.reaction_count`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusLike {
    /// The status reacted to, by wire id.
    pub status_id: String,
    /// Who reacted.
    pub by: Jid,
    /// The emoji; `None` is a reaction being *removed*, which is why an empty
    /// text is kept as a record rather than dropped.
    pub emoji: Option<String>,
    /// When we recorded it.
    pub at: Timestamp,
}

/// Recognize a status like in a decrypted payload. Returns `None` for a chat
/// reaction or anything else — a reaction in a chat is not a status like and
/// counting it as one would attribute it to the wrong thing entirely.
pub fn like_from_payload(payload: &[u8], node: &Node) -> Option<StatusLike> {
    let (remote_jid, _) = super::reactions::key_fields(payload)?;
    let remote = remote_jid?;
    // APK 37.71 BoQ.A00 selects 8oR only for the status singleton;
    // 8oR.<init>/getServer identify status@broadcast, not every broadcast.
    let remote = Jid::parse(&remote).ok()?;
    if remote.server != crate::model::jid::Server::Broadcast
        || remote.user != "status"
        || remote.agent != 0
        || remote.device != 0
    {
        return None;
    }
    let reaction = super::reactions::decode(payload)?;
    let by = node
        .get_attr("participant")
        .or_else(|| node.get_attr("from"))
        .and_then(|v| {
            v.as_jid()
                .cloned()
                .or_else(|| Jid::parse(v.as_text()?).ok())
        })?;
    if !by.is_user_like() {
        return None;
    }
    Some(StatusLike {
        status_id: reaction.target_id.as_str().to_owned(),
        by: by.to_non_ad(),
        emoji: reaction.emoji,
        at: Timestamp::now(),
    })
}

/// Record a like inside an existing transaction, so it commits with the message
/// that carried it and cannot survive a rolled-back stanza.
pub fn record_like_in(tx: &mut WriteTx<'_>, like: &StatusLike) -> Result<()> {
    let key = format!("{LIKE_PREFIX}{}:{}", like.status_id, like.by);
    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, like)
}

/// Every recorded like, for one status or all of them.
pub fn list_likes(store: &ProtocolStore, status_id: Option<&str>) -> Result<Vec<StatusLike>> {
    let wanted = status_id.map(|id| format!("{LIKE_PREFIX}{id}:"));
    store.read(|tx| {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 512)?;
            let done = rows.len() < 512;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, value) in rows {
                let Ok(name) = std::str::from_utf8(&key) else {
                    continue;
                };
                if !name.starts_with(LIKE_PREFIX) {
                    continue;
                }
                if let Some(wanted) = &wanted
                    && !name.starts_with(wanted.as_str())
                {
                    continue;
                }
                if let Ok(like) = serde_json::from_slice::<StatusLike>(&value) {
                    out.push(like);
                }
            }
            if done {
                out.sort_by(|a, b| {
                    a.status_id
                        .cmp(&b.status_id)
                        .then_with(|| a.by.to_string().cmp(&b.by.to_string()))
                });
                return Ok(out);
            }
        }
    })
}

#[cfg(test)]
mod like_tests {
    use super::*;

    fn payload(remote: &str, emoji: &str) -> Vec<u8> {
        use prost::Message as _;
        #[derive(prost::Message)]
        struct Key {
            #[prost(string, optional, tag = "1")]
            remote_jid: Option<String>,
            #[prost(string, optional, tag = "3")]
            id: Option<String>,
        }
        #[derive(prost::Message)]
        struct Reaction {
            #[prost(message, optional, tag = "1")]
            key: Option<Key>,
            #[prost(string, optional, tag = "2")]
            text: Option<String>,
        }
        #[derive(prost::Message)]
        struct Envelope {
            #[prost(message, optional, tag = "46")]
            reaction: Option<Reaction>,
        }
        Envelope {
            reaction: Some(Reaction {
                key: Some(Key {
                    remote_jid: Some(remote.into()),
                    id: Some("STATUS1".into()),
                }),
                text: Some(emoji.into()),
            }),
        }
        .encode_to_vec()
    }

    fn node() -> Node {
        Node::new("message")
            .attr("from", Jid::parse("status@broadcast").unwrap())
            .attr("participant", Jid::parse("222222222222222@lid").unwrap())
    }

    /// The key's `remoteJid` is what makes a reaction a status like. A chat
    /// reaction carries the chat there and must not be counted.
    #[test]
    fn only_a_reaction_keyed_to_status_broadcast_is_a_like() {
        let like = like_from_payload(&payload("status@broadcast", "👍"), &node()).unwrap();
        assert_eq!(like.status_id, "STATUS1");
        assert_eq!(like.by, Jid::parse("222222222222222@lid").unwrap());
        assert_eq!(like.emoji.as_deref(), Some("👍"));
        assert!(
            like_from_payload(&payload("15551234567@s.whatsapp.net", "👍"), &node()).is_none(),
            "a chat reaction is not a status like"
        );
        assert!(like_from_payload(b"not a protobuf at all", &node()).is_none());
        for remote in [
            "12345@broadcast",
            "location@broadcast",
            "status:3@broadcast",
        ] {
            assert!(like_from_payload(&payload(remote, "👍"), &node()).is_none());
        }
    }

    /// Removing a reaction sends an empty text; that is a record, not nothing,
    /// or the like would appear to persist forever.
    #[test]
    fn a_removed_reaction_is_recorded_with_no_emoji() {
        let like = like_from_payload(&payload("status@broadcast", ""), &node()).unwrap();
        assert!(like.emoji.is_none());
    }
}
