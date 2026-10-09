//! Narrow comparison notification handling; unsupported sync stays captured.
//! APK 37.71 GeneratePrivacyTokenJob.A0G / 0q4.A0J use separate outbound
//! issue/send timestamps. Incoming bytes here never supply refresh authority;
//! see docs/protocol/identity-token-refresh.md for the still-unported job.
//! APK CnI.A03/CoO.A02 restore incoming bytes and outbound sender timestamps
//! separately during imports (token-import.md); these notifications do not
//! implement that history/transfer authority or create outbound issue state.
//! LX/11P.A09 reads tokens/token, trusted_contact, sender_lid and timestamp.
//! LX/0q2.A0P replaces an incoming token only when its timestamp is newer.
use super::receipts::jid;

/// Process a fully supported notification before returning its acknowledgement.
/// APK 37.71 LX/0yO.A01 dispatches before A00 acknowledges; LX/Cp2.A05
/// (0036..0074) serializes id/to/class/type. Unknown token kinds stay without ACK.
pub fn handle(store: &ProtocolStore, node: &Node) -> Result<Option<Node>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("privacy_token")
        || node.get_attr("participant").is_some()
        || node.get_attr("recipient").is_some()
    {
        return Ok(None);
    }
    let Some(from) = jid(node, "from")
        .filter(|v| matches!(v.server, Server::User | Server::Lid) && v.device == 0)
    else {
        return Ok(None);
    };
    let Some(id) = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.len() <= 128)
    else {
        return Ok(None);
    };
    record(store, node)?;
    if node.child_nodes().len() != 1
        || node.child_nodes()[0].description != "tokens"
        || node.child_nodes()[0].child_nodes().is_empty()
        || node.child_nodes()[0].child_nodes().iter().any(|v| {
            v.description != "token"
                || v.get_attr("type").and_then(|v| v.as_text()) != Some("trusted_contact")
        })
    {
        return Ok(None);
    }
    Ok(Some(
        Node::new("ack")
            .attr("to", from)
            .attr("id", id)
            .attr("class", "notification")
            .attr("type", "privacy_token"),
    ))
}
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::Server,
    store::{ProtocolStore, Table},
};

/// Apply supported trusted-contact tokens atomically. This does not fabricate
/// Signal identity mappings, acknowledge unknown notifications, or apply app-state.
pub fn record(store: &ProtocolStore, node: &Node) -> Result<usize> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("privacy_token")
    {
        return Ok(0);
    }
    if node.get_attr("participant").is_some() || node.get_attr("recipient").is_some() {
        return Ok(0);
    }
    let Some(from) = jid(node, "from")
        .filter(|v| matches!(v.server, Server::User | Server::Lid) && v.device == 0)
    else {
        return Ok(0);
    };
    let target = if node.get_attr("sender_lid").is_some() {
        jid(node, "sender_lid")
            .filter(|v| v.server == Server::Lid && v.device == 0)
            .ok_or_else(|| Error::invalid("notification", "invalid sender LID"))?
    } else {
        from
    };
    let tokens: Vec<_> = node
        .child_nodes()
        .iter()
        .filter(|v| v.description == "tokens")
        .collect();
    if tokens.len() != 1 || tokens[0].child_nodes().len() > 256 {
        return Err(Error::invalid(
            "notification",
            "missing or excessive token list",
        ));
    }
    let timestamp = |node: &Node| {
        node.get_attr("t")
            .and_then(|v| v.as_text())
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v <= i64::MAX as u64)
    };
    let parent_time = timestamp(node)
        .ok_or_else(|| Error::invalid("notification", "invalid notification timestamp"))?;
    let mut updates = Vec::new();
    for token in tokens[0].child_nodes() {
        if token.description != "token"
            || token.get_attr("type").and_then(|v| v.as_text()) != Some("trusted_contact")
        {
            continue;
        }
        let NodeContent::Bytes(bytes) = &token.content else {
            return Err(Error::invalid("notification", "token data missing"));
        };
        if bytes.is_empty() || bytes.len() > 4096 {
            return Err(Error::invalid(
                "notification",
                "token size outside local bound",
            ));
        }
        let time = if token.get_attr("t").is_some() {
            timestamp(token)
                .ok_or_else(|| Error::invalid("notification", "invalid token timestamp"))?
        } else {
            parent_time
        };
        updates.push((time, bytes));
    }
    let key = format!("incoming-trusted-contact:{target}");
    store.write(|tx| {
        let mut count = 0;
        for (time, bytes) in updates {
            let saved: Option<serde_json::Value> =
                tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
            if let Some(saved) = saved {
                let old = saved["timestamp"].as_u64().ok_or_else(|| {
                    Error::invalid("notification", "invalid saved token timestamp")
                })?;
                if old >= time {
                    continue;
                }
            }
            tx.put_json(
                Table::Checkpoints,
                key.as_bytes(),
                1,
                &serde_json::json!({"timestamp":time,"token":bytes}),
            )?;
            count += 1;
        }
        Ok(count)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_and_older_notifications_do_not_replace_newer_tokens() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("n.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        })
        .unwrap();
        let notification = |t: &str, data: u8| {
            Node::new("notification")
                .attr("type", "privacy_token")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("sender_lid", "12345@lid")
                .attr("t", t)
                .children(vec![Node::new("tokens").children(
                    vec![Node::new("token").attr("type","trusted_contact").bytes(vec![data])],
                )])
        };
        assert_eq!(record(&store, &notification("20", 2)).unwrap(), 1);
        assert_eq!(record(&store, &notification("10", 1)).unwrap(), 0);
        assert_eq!(record(&store, &notification("20", 3)).unwrap(), 0);
        assert_eq!(record(&store, &notification("30", 4)).unwrap(), 1);
        assert!(record(&store, &notification("bad", 5)).is_err());
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"incoming-trusted-contact:12345@lid", 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved, serde_json::json!({"timestamp":30,"token":[4]}));
        let mapping: Option<serde_json::Value> = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"authenticated-lid:12345@lid", 1))
            .unwrap();
        assert!(mapping.is_none());
        let ack = handle(&store, &notification("30", 4).attr("id", "fixture"))
            .unwrap()
            .unwrap();
        assert_eq!(ack.description, "ack");
        assert_eq!(
            ack.get_attr("class").and_then(|v| v.as_text()),
            Some("notification")
        );
        assert!(
            handle(&store, &Node::new("notification").attr("type", "devices"))
                .unwrap()
                .is_none()
        );
    }
}
