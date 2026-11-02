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

/// Acknowledge a `psa` (WhatsApp announcement) notification. None for any
/// other stanza; an error only for a psa notification that cannot be ACKed.
///
/// **Confirmed, 2.26.37.71:** `Fbs` registers type `psa`; its handler
/// `LX/17o` (`PSANotificationHandler`) extends the base notification handler
/// `0yO`, whose `A01` dispatches to `A09` and then ACKs through `A00` unless
/// the handler deferred (`A07(true)`), in which case `AAR.A00` ACKs when the
/// work finishes. Every psa notification is therefore ACKed exactly once.
/// `17o.A09` (0000..0104) only inspects children when `from` is
/// `0@s.whatsapp.net`: a `surfaces` (QP surface) or `psa_wakeup` (push PSA)
/// child whose feature is off is ACKed by `17o.A03` with the extra attribute
/// `error="487"`. This client implements neither, which is that branch.
/// Any other child or sender falls through to the plain base ACK.
/// `Cp2.A05` writes id/to/class/type/participant (see
/// identity-notification-ack.md). **Observed 2026-11-01:** without this ACK the
/// server ended the session with `stream:error` echoing
/// `ack class=notification type=psa`.
pub fn psa_ack(node: &Node) -> Result<Option<Node>> {
    let Some(ack) = base_ack(node, "psa")? else {
        return Ok(None);
    };
    let from_server = jid(node, "from").is_some_and(|from| {
        from.user == "0" && from.server == Server::User && from.agent == 0 && from.device == 0
    });
    let not_enabled =
        from_server && (node.child("surfaces").is_some() || node.child("psa_wakeup").is_some());
    Ok(Some(if not_enabled {
        ack.attr("error", "487")
    } else {
        ack
    }))
}

/// Acknowledge a `disappearing_mode` notification (a contact's default
/// disappearing timer). **Inferred handler, confirmed ACK:** 2.26.37.71 `11O`
/// reads its `disappearing_mode` child (`duration`, `t`) and extends `0yO`,
/// which ACKs every notification it dispatches. The timer is not applied here;
/// the durable capture keeps it.
pub fn disappearing_mode_ack(node: &Node) -> Result<Option<Node>> {
    base_ack(node, "disappearing_mode")
}

/// Acknowledge a `business` notification. **Confirmed, 2.26.37.71:**
/// `BusinessNotificationHandler` is `17Y`, which extends `0yO` and neither
/// defers nor adds attributes, so it gets the plain base ACK. Its content
/// (directory, campaign status, NUX) is not applied; the capture keeps it.
pub fn business_ack(node: &Node) -> Result<Option<Node>> {
    base_ack(node, "business")
}

/// Acknowledge a `mex` notification. **Confirmed, 2.26.37.71:** the MEX
/// notification router is `0yP` (logs `mexnotificationrouter/onhandlernotfound`
/// and `…/onoperationnotfound`); it extends `0yO`, neither defers nor adds
/// attributes, and returns to the base ACK even when no handler or operation
/// matches. Its operations (group push name, username, reminders, …) are not
/// applied here; the capture keeps them. Most frequent type in our captures.
pub fn mex_ack(node: &Node) -> Result<Option<Node>> {
    base_ack(node, "mex")
}

const REACHOUT_TIMELOCK: &[u8] = b"reachout-timelock";

/// The account's reach-out timelock, as the server last pushed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReachoutTimelock {
    /// `is_active`.
    pub active: bool,
    /// `time_enforcement_ends`, Unix seconds.
    pub ends: i64,
}

/// Record a `mex` `NotificationUserReachoutTimelockUpdate`.
///
/// **Observed 2026-11-01** on `q`: `{"data":{"xwa2_notify_account_reachout_timelock":
/// {"enforcement_type":"DEFAULT","is_active":true,"time_enforcement_ends":"<unix s>"}}}`,
/// pushed when WhatsApp locked the new account's outreach for six hours; group
/// creation was refused 463 `account_reachout_restricted` meanwhile.
/// **Confirmed static, 2.26.37.71:** the op name and the `ReachoutTimelock*`
/// handler/banner/query classes exist (`FetchReachoutTimelockQuery`); the
/// APK's exact enforcement in the UI is not traced. Returns the recorded lock.
pub fn record_reachout_timelock(
    store: &ProtocolStore,
    node: &Node,
) -> Result<Option<ReachoutTimelock>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("mex")
    {
        return Ok(None);
    }
    for child in node.child_nodes() {
        if child.get_attr("op_name").and_then(|v| v.as_text())
            != Some("NotificationUserReachoutTimelockUpdate")
        {
            continue;
        }
        let body = match &child.content {
            NodeContent::Bytes(b) => b.to_vec(),
            NodeContent::Text(t) => t.as_bytes().to_vec(),
            _ => continue,
        };
        let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) else {
            continue;
        };
        let lock = &json["data"]["xwa2_notify_account_reachout_timelock"];
        let Some(active) = lock["is_active"].as_bool() else {
            continue;
        };
        let ends = lock["time_enforcement_ends"]
            .as_str()
            .and_then(|v| v.parse().ok())
            .or_else(|| lock["time_enforcement_ends"].as_i64())
            .unwrap_or(0);
        let lock = ReachoutTimelock { active, ends };
        store.write(|tx| tx.put_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1, &lock))?;
        return Ok(Some(lock));
    }
    Ok(None)
}

/// The recorded lock if it is still in force at `now_seconds`.
pub fn active_reachout_timelock(
    store: &ProtocolStore,
    now_seconds: i64,
) -> Result<Option<ReachoutTimelock>> {
    let lock: Option<ReachoutTimelock> =
        store.read(|tx| tx.get_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1))?;
    Ok(lock.filter(|l| l.active && l.ends > now_seconds))
}

/// The plain notification ACK the base handler sends (2.26.37.71 `0yO.A00`
/// via `Cp2.A05`): id/to/class/type, plus participant when present. None
/// for any other stanza or type; an error for a matching notification that
/// cannot be ACKed (missing/oversized id, invalid from/participant).
pub(crate) fn base_ack(node: &Node, kind: &str) -> Result<Option<Node>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some(kind)
    {
        return Ok(None);
    }
    let invalid = |reason: &str| Error::invalid("notification", format!("{kind}: {reason}"));
    let id = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|id| !id.is_empty() && id.len() <= 128)
        .ok_or_else(|| invalid("missing or excessive notification id"))?;
    let from = jid(node, "from").ok_or_else(|| invalid("missing or invalid notification from"))?;
    let participant = node
        .get_attr("participant")
        .map(|_| {
            jid(node, "participant").ok_or_else(|| invalid("invalid notification participant"))
        })
        .transpose()?;
    let mut ack = Node::new("ack")
        .attr("id", id)
        .attr("to", super::receipts::ack_address(from))
        .attr("class", "notification")
        .attr("type", kind);
    if let Some(participant) = participant {
        ack = ack.attr("participant", super::receipts::ack_address(participant));
    }
    Ok(Some(ack))
}

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
            .attr("to", super::receipts::ack_address(from))
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
    let Some((key, updates)) = parse(node)? else {
        return Ok(0);
    };
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

/// The wire-content checks [`record`] makes, without the store: missing or
/// invalid `t`, token list shape, token bytes or size, `sender_lid`. A refusal
/// here is the stanza's own (receive audit F2); once this passes, anything
/// [`handle`] raises concerns the stored token row or storage.
pub fn validate(node: &Node) -> Result<()> {
    parse(node).map(|_| ())
}

/// Checkpoint key and `(timestamp, token)` updates.
type TokenUpdates<'a> = (String, Vec<(u64, &'a bytes::Bytes)>);

/// The updates to apply, or `None` for a notification this path does not record.
fn parse(node: &Node) -> Result<Option<TokenUpdates<'_>>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("privacy_token")
    {
        return Ok(None);
    }
    if node.get_attr("participant").is_some() || node.get_attr("recipient").is_some() {
        return Ok(None);
    }
    let Some(from) = jid(node, "from").filter(|v| {
        !v.user.is_empty() && matches!(v.server, Server::User | Server::Lid) && v.device == 0
    }) else {
        return Ok(None);
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
    Ok(Some((
        format!("incoming-trusted-contact:{target}"),
        updates,
    )))
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
    fn psa_notifications_are_acked_with_487_only_for_unimplemented_features() {
        let text = |n: &Node, k: &str| n.get_attr(k).and_then(|v| v.as_text()).map(str::to_owned);
        let psa = |from: &str, child: &str| {
            Node::new("notification")
                .attr("from", from)
                .attr("type", "psa")
                .attr("id", "1234")
                .attr("t", "1793557256")
                .children(vec![Node::new(child)])
        };
        for (child, error) in [
            ("surfaces", Some("487")),
            ("psa_wakeup", Some("487")),
            ("campaign", None),
            ("other", None),
        ] {
            let ack = psa_ack(&psa("0@s.whatsapp.net", child)).unwrap().unwrap();
            assert_eq!(ack.description, "ack");
            assert_eq!(text(&ack, "id").as_deref(), Some("1234"));
            assert_eq!(text(&ack, "class").as_deref(), Some("notification"));
            assert_eq!(text(&ack, "type").as_deref(), Some("psa"));
            assert!(ack.get_attr("to").is_some());
            assert_eq!(text(&ack, "error").as_deref(), error, "{child}");
        }
        // 17o.A09 inspects children only for the server sender.
        let broadcast = psa("status@broadcast", "surfaces").attr("participant", "0@s.whatsapp.net");
        let ack = psa_ack(&broadcast).unwrap().unwrap();
        assert!(ack.get_attr("error").is_none());
        assert!(ack.get_attr("participant").is_some());
        // Other stanzas are not ours; a psa without id cannot be ACKed.
        assert!(
            psa_ack(&Node::new("notification").attr("type", "devices"))
                .unwrap()
                .is_none()
        );
        assert!(
            psa_ack(
                &Node::new("notification")
                    .attr("type", "psa")
                    .attr("from", "0@s.whatsapp.net")
            )
            .is_err()
        );
    }

    #[test]
    fn mex_notifications_get_the_plain_base_ack() {
        let node = Node::new("notification")
            .attr("from", "s.whatsapp.net")
            .attr("type", "mex")
            .attr("id", "99")
            .children(vec![
                Node::new("update").attr("op_name", "NotificationReminder"),
            ]);
        let ack = mex_ack(&node).unwrap().unwrap();
        assert_eq!(ack.get_attr("type").and_then(|v| v.as_text()), Some("mex"));
        assert!(ack.get_attr("error").is_none());
    }

    #[test]
    fn reachout_timelock_is_recorded_and_expires() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(crate::store::OpenOptions {
            path: dir.path().join("r.redb"),
            account_id: crate::model::ids::AccountId::new("t").unwrap(),
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([2; 32]),
            )),
            create: true,
        })
        .unwrap();
        let body = br#"{"data":{"xwa2_notify_account_reachout_timelock":{"enforcement_type":"DEFAULT","is_active":true,"time_enforcement_ends":"1000"}}}"#;
        let node = Node::new("notification").attr("type", "mex").children(vec![
            Node::new("update")
                .attr("op_name", "NotificationUserReachoutTimelockUpdate")
                .bytes(body.to_vec()),
        ]);
        let lock = record_reachout_timelock(&store, &node).unwrap().unwrap();
        assert_eq!(
            lock,
            ReachoutTimelock {
                active: true,
                ends: 1000
            }
        );
        assert!(active_reachout_timelock(&store, 999).unwrap().is_some());
        assert!(active_reachout_timelock(&store, 1000).unwrap().is_none());
    }

    #[test]
    fn business_notifications_get_the_plain_base_ack() {
        let node = Node::new("notification")
            .attr("from", "15551234567@s.whatsapp.net")
            .attr("type", "business")
            .attr("id", "88");
        let ack = business_ack(&node).unwrap().unwrap();
        assert_eq!(
            ack.get_attr("type").and_then(|v| v.as_text()),
            Some("business")
        );
        assert!(
            business_ack(&Node::new("notification").attr("type", "psa"))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn disappearing_mode_notifications_get_the_plain_base_ack() {
        let node = Node::new("notification")
            .attr("from", "15551234567@s.whatsapp.net")
            .attr("type", "disappearing_mode")
            .attr("id", "77")
            .children(vec![
                Node::new("disappearing_mode").attr("duration", "86400"),
            ]);
        let ack = disappearing_mode_ack(&node).unwrap().unwrap();
        let text = |k: &str| ack.get_attr(k).and_then(|v| v.as_text()).map(str::to_owned);
        assert_eq!(text("type").as_deref(), Some("disappearing_mode"));
        assert_eq!(text("class").as_deref(), Some("notification"));
        assert_eq!(text("id").as_deref(), Some("77"));
        assert!(ack.get_attr("to").is_some() && ack.get_attr("error").is_none());
        assert!(
            disappearing_mode_ack(&Node::new("notification").attr("type", "psa"))
                .unwrap()
                .is_none()
        );
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
