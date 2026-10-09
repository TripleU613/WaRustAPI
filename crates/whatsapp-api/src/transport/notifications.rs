//! Narrow comparison notification handling; unsupported sync stays captured.
//! APK 37.71 GeneratePrivacyTokenJob.A0G / 0q4.A0J use separate outbound
//! issue/send timestamps. Incoming bytes here never supply refresh authority;
//! see docs/protocol/identity-token-refresh.md for the still-unported job.
//! APK CnI.A03/CoO.A02 restore incoming bytes and outbound sender timestamps
//! separately during imports (token-import.md); these notifications do not
//! implement that history/transfer authority or create outbound issue state.
//! LX/11P.A09 reads tokens/token, trusted_contact, sender_lid and timestamp.
//! 37.71 11P.A09 008e..00bc resolves from/sender_lid as UserJid before
//! storing bytes. Our generic Jid admits server-only values, so this consumer
//! also requires a concrete user; see token-notification-peer.md.
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
    let Some(from) = jid(node, "from").filter(|v| {
        !v.user.is_empty() && matches!(v.server, Server::User | Server::Lid) && v.device == 0
    }) else {
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
    let Some(from) = jid(node, "from").filter(|v| {
        !v.user.is_empty() && matches!(v.server, Server::User | Server::Lid) && v.device == 0
    }) else {
        return Ok(0);
    };
    let target = if node.get_attr("sender_lid").is_some() {
        jid(node, "sender_lid")
            .filter(|v| !v.user.is_empty() && v.server == Server::Lid && v.device == 0)
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
            // APK 37.71 11P.A09 -> 0hs.A09/A08 uses Long.parseLong;
            // retain the local nonnegative storage bound after Java parsing.
            .and_then(super::mex::java_decimal_long)
            .and_then(|v| u64::try_from(v).ok())
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
    fn java_token_times_preserve_fallback_override_and_atomic_refusal() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("time.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [4; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let token = |time: Option<&str>, data: u8| {
            let node = Node::new("token")
                .attr("type", "trusted_contact")
                .bytes(vec![data]);
            if let Some(time) = time {
                node.attr("t", time)
            } else {
                node
            }
        };
        let notification = |time: &str, children| {
            Node::new("notification")
                .attr("type", "privacy_token")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("id", "fixture")
                .attr("t", time)
                .children(vec![Node::new("tokens").children(children)])
        };
        // APK 11P.A09 -> 0hs.A09/A08: Java BMP decimal digits and ASCII sign.
        // Independent Java Long.parseLong cases are also pinned in mex tests.
        let first = notification("+２０", vec![token(None, 1)]);
        assert!(handle(&store, &first).unwrap().is_some());
        let next = notification("٢٠", vec![token(Some("+٣٠"), 2)]);
        assert!(handle(&store, &next).unwrap().is_some());
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
        };
        let before = snapshot(&store);
        assert_eq!(record(&store, &first).unwrap(), 0);
        for invalid in ["", " 40", "40 ", "-1", "9223372036854775808", "𝟜𝟘"] {
            // A malformed later token must not commit the earlier newer token.
            let bad = notification("40", vec![token(Some("40"), 3), token(Some(invalid), 4)]);
            let error = handle(&store, &bad).unwrap_err().to_string();
            assert!(error.contains("invalid token timestamp"));
            assert!(!error.contains(invalid) || invalid.is_empty());
            assert_eq!(snapshot(&store), before);
        }
        let zero = notification("-0", vec![token(None, 5)]);
        assert_eq!(record(&store, &zero).unwrap(), 0);
        assert_eq!(snapshot(&store), before);
        let saved: serde_json::Value = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    b"incoming-trusted-contact:15555550100@s.whatsapp.net",
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(saved, serde_json::json!({"timestamp":30,"token":[2]}));
        drop(store);
        assert_eq!(snapshot(&open(false)), before);
    }
    #[test]
    fn empty_notification_peers_never_store_tokens_or_acknowledge() {
        use crate::{
            model::{
                ids::AccountId,
                jid::{Jid, JidKind},
            },
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("peers.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [4; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let notification = || {
            Node::new("notification")
                .attr("type", "privacy_token")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("id", "fixture")
                .attr("t", "20")
                .children(vec![Node::new("tokens").children(
                    vec![Node::new("token").attr("type", "trusted_contact").bytes(vec![1])],
                )])
        };
        assert!(handle(&store, &notification()).unwrap().is_some());
        store
            .write(|tx| {
                tx.put_json(Table::PeerIdentities, b"sentinel", 1, &true)?;
                tx.put_json(Table::SignalSessions, b"sentinel", 1, &true)
            })
            .unwrap();
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    Ok([
                        tx.scan(Table::Checkpoints, 1, 100)?,
                        tx.scan(Table::PeerIdentities, 1, 100)?,
                        tx.scan(Table::SignalSessions, 1, 100)?,
                    ])
                })
                .unwrap()
        };
        let before = snapshot(&store);
        for server in [Server::User, Server::Lid] {
            let text = notification().attr("from", server.as_str());
            assert_eq!(record(&store, &text).unwrap(), 0);
            assert!(handle(&store, &text).unwrap().is_none());
            for kind in [JidKind::Plain, JidKind::AgentDevice] {
                let peer = Jid {
                    user: String::new(),
                    server: server.clone(),
                    agent: 0,
                    device: 0,
                    kind,
                };
                let node = notification().attr("from", peer);
                assert_eq!(record(&store, &node).unwrap(), 0);
                assert!(handle(&store, &node).unwrap().is_none());
            }
        }
        for value in [
            Jid::parse("lid").unwrap(),
            Jid {
                user: String::new(),
                server: Server::Lid,
                agent: 0,
                device: 0,
                kind: JidKind::AgentDevice,
            },
        ] {
            let node = notification().attr("sender_lid", value);
            let error = record(&store, &node).unwrap_err().to_string();
            assert!(error.contains("invalid sender LID"));
            assert!(handle(&store, &node).is_err());
        }
        assert_eq!(snapshot(&store), before);
        drop(store);
        assert_eq!(snapshot(&open(false)), before);
    }

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
