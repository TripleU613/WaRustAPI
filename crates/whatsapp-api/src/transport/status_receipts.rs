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
    store::{ProtocolStore, Table},
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
pub fn parse(node: &Node) -> Option<StatusReceipt> {
    if node.description != "receipt" {
        return None;
    }
    let participants = node.child("participants")?;
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
