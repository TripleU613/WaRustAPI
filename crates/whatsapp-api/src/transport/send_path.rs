//! Shared send-path ownership. An ID cannot migrate between lab and outbox to
//! accidentally repeat an acknowledged or uncertain transmission.
use crate::{
    Error, Result,
    model::ids::ClientRequestId,
    outbox::OutboxEntry,
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};

/// The implementation that owns a durable send request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendPath {
    /// Historical explicit comparison lab send.
    Lab,
    /// Public-client outbox send.
    Outbox,
}
fn conflict() -> Error {
    Error::invalid(
        "send_path",
        "request ID belongs to another send path; preserve its saved outcome, no replay",
    )
}

/// Reserve the implementation before any upload/key query/socket write. Repeating
/// the same path is allowed but does not override that path's own retry guards.
/// Historical lab intents and all outbox namespaces are checked before adoption.
pub fn claim(store: &ProtocolStore, id: &ClientRequestId, path: SendPath) -> Result<()> {
    let key = format!("message-path:{}", id.as_str());
    let existing: Option<SendPath> =
        store.read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?;
    if let Some(existing) = existing {
        return if existing == path {
            Ok(())
        } else {
            Err(conflict())
        };
    }
    store.read(|tx| {
        let lab = tx
            .get(
                Table::Checkpoints,
                format!("message:{}:send", id.as_str()).as_bytes(),
                1,
            )?
            .is_some();
        if lab && path != SendPath::Lab {
            return Err(conflict());
        }
        let mut after = None;
        loop {
            let page = tx.scan_after(Table::Outbox, 1, after.as_deref(), 256)?;
            if page.is_empty() {
                break;
            }
            after = page.last().map(|(key, _)| key.clone());
            for (_, bytes) in page {
                let entry: OutboxEntry = serde_json::from_slice(&bytes)
                    .map_err(|_| Error::invalid("send_path", "invalid historical outbox record"))?;
                if entry.client_request_id == *id && (path != SendPath::Outbox || lab) {
                    return Err(conflict());
                }
            }
        }
        Ok(())
    })?;
    store.write(|tx| {
        if let Some(existing) = tx.get_json::<SendPath>(Table::Checkpoints, key.as_bytes(), 1)? {
            if existing != path {
                return Err(conflict());
            }
            return Ok(());
        }
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &path)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId},
            jid::Jid,
            message::MessageContent,
            send::SendRequest,
        },
        outbox::Outbox,
        profile::Limits,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    #[test]
    fn claims_survive_restart_and_historical_outbox_namespaces_block_lab() {
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("paths.redb"),
            account_id: AccountId::new("test").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
        };
        let store = ProtocolStore::open(options()).unwrap();
        let id = ClientRequestId::new("same-request").unwrap();
        claim(&store, &id, SendPath::Outbox).unwrap();
        assert!(claim(&store, &id, SendPath::Lab).is_err());
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        claim(&store, &id, SendPath::Outbox).unwrap();
        assert!(claim(&store, &id, SendPath::Lab).is_err());
        let old = ClientRequestId::new("historical").unwrap();
        store
            .write(|tx| {
                Outbox::new("custom-namespace", Limits::default()).enqueue(
                    tx,
                    &SendRequest {
                        client_request_id: old.clone(),
                        chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net")?)?,
                        content: MessageContent::Text {
                            body: "synthetic".into(),
                            quoted: None,
                            style: Default::default(),
                        },
                    },
                )?;
                Ok(())
            })
            .unwrap();
        assert!(claim(&store, &old, SendPath::Lab).is_err());
        claim(&store, &old, SendPath::Outbox).unwrap();
        let lab = ClientRequestId::new("historical-lab").unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"message:historical-lab:send",
                    1,
                    &serde_json::json!({"status":"outcome_unknown"}),
                )
            })
            .unwrap();
        assert!(claim(&store, &lab, SendPath::Outbox).is_err());
        claim(&store, &lab, SendPath::Lab).unwrap();
    }
}
