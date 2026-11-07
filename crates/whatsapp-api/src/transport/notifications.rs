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
/// work finishes. This historical path is not a universal current-build ACK
/// rule:43.72 has additional deferred feature/campaign/revoke work. See
/// docs/protocol/psa-ack-lifecycle.md; this helper retains its legacy policy.
/// `17o.A09` (0000..0104) only inspects children when `from` is
/// `0@s.whatsapp.net`: a `surfaces` (QP surface) or `psa_wakeup` (push PSA)
/// child whose feature is off is ACKed by `17o.A03` with the extra attribute
/// `error="487"`. The legacy Rust helper uses that unsupported-feature
/// response for these payloads; current native effective flags remain unknown.
/// The legacy Rust helper uses plain ACK for other children/senders; the
/// complete current native branch matrix and work ownership remain unported.
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

/// Acknowledge a captured `picture` notification without applying photo state.
/// APK 2.26.43.72 EL2 maps picture to189;19k extends0v6, has no ACK
/// override and does not call its A07 deferral helper. Normal return reaches
/// the base ACK. See the group-missing-identification evidence bundle.
/// Missing-identification context and photo updates remain unported.
pub fn picture_ack(node: &Node) -> Result<Option<Node>> {
    base_ack(node, "picture")
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
    // APK 43.72 0v7.A09 routes only the first update and lowercases op_name
    // with Locale.US. This supported operation name is ASCII.
    for child in node
        .child_nodes()
        .iter()
        .filter(|child| child.description == "update")
        .take(1)
    {
        if !child
            .get_attr("op_name")
            .and_then(|v| v.as_text())
            .is_some_and(|op| op.eq_ignore_ascii_case("NotificationUserReachoutTimelockUpdate"))
        {
            continue;
        }
        let body = match &child.content {
            NodeContent::Bytes(b) => b.to_vec(),
            NodeContent::Text(t) => t.as_bytes().to_vec(),
            _ => {
                return Err(Error::invalid(
                    "reachout_timelock",
                    "recognized update missing JSON body; prior restriction preserved",
                ));
            }
        };
        // A recognized but unusable update must not become an ignored MEX
        // operation that receives the generic ACK. Native 8AF.A09 likewise
        // distinguishes typed conversion failure from successful dispatch;
        // no claim of identical native error-ACK policy is made here.
        let json: serde_json::Value = serde_json::from_slice(&body).map_err(|_| {
            Error::invalid(
                "reachout_timelock",
                "invalid update JSON; prior restriction preserved",
            )
        })?;
        let lock = &json["data"]["xwa2_notify_account_reachout_timelock"];
        let active = lock["is_active"].as_bool().ok_or_else(|| {
            Error::invalid(
                "reachout_timelock",
                "missing or invalid active flag; prior restriction preserved",
            )
        })?;
        // APK 43.72 7vL.A00 clears inactive state without parsing duration.
        // Only active updates need an expiry; never fabricate zero for them.
        let ends = if !active {
            0
        } else {
            lock["time_enforcement_ends"]
                .as_str()
                // APK 43.72 6oe.A03 calls Java Long.parseLong.
                .and_then(super::mex::java_decimal_long)
                .or_else(|| lock["time_enforcement_ends"].as_i64())
                .filter(|value| *value >= 0)
                .ok_or_else(|| {
                    Error::invalid(
                        "reachout_timelock",
                        "missing or invalid expiry; existing restriction preserved",
                    )
                })?
        };
        // Local integrity boundary: an envelope we cannot acknowledge must
        // not alter account restriction state before mex_ack rejects it.
        base_ack(node, "mex")?;
        let lock = ReachoutTimelock { active, ends };
        store.write(|tx| tx.put_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1, &lock))?;
        return Ok(Some(lock));
    }
    Ok(None)
}

/// The recorded active lock, including one whose displayed deadline elapsed.
/// APK 43.72 6qK.onFinish -> 7uq.A00 fetches state; expiry alone does not clear it.
pub(super) fn recorded_active_reachout_timelock(
    store: &ProtocolStore,
) -> Result<Option<ReachoutTimelock>> {
    store.read(recorded_active_reachout_timelock_read_in)
}

/// The same recorded restriction in an existing read snapshot (writer admission).
pub(super) fn recorded_active_reachout_timelock_read_in(
    tx: &crate::store::ReadTx<'_>,
) -> Result<Option<ReachoutTimelock>> {
    let lock: Option<ReachoutTimelock> = tx.get_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1)?;
    Ok(lock.filter(|l| l.active))
}

/// Read restriction admission state in the same transaction as a reservation.
pub(super) fn recorded_active_reachout_timelock_in(
    tx: &crate::store::WriteTx<'_>,
) -> Result<Option<ReachoutTimelock>> {
    let lock: Option<ReachoutTimelock> = tx.get_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1)?;
    Ok(lock.filter(|l| l.active))
}

/// An active record with an unelapsed displayed deadline, for inspection only.
/// A `None` here does not authorize an operation: authoritative active state
/// can persist until a subsequent update, even after the displayed deadline.
pub fn active_reachout_timelock(
    store: &ProtocolStore,
    now_seconds: i64,
) -> Result<Option<ReachoutTimelock>> {
    Ok(recorded_active_reachout_timelock(store)?.filter(|l| l.ends > now_seconds))
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
    let from =
        ack_route(node, "from").ok_or_else(|| invalid("missing or invalid notification from"))?;
    let participant = node
        .get_attr("participant")
        .map(|_| {
            ack_route(node, "participant")
                .ok_or_else(|| invalid("invalid notification participant"))
        })
        .transpose()?;
    let mut ack = Node::new("ack")
        .attr("id", id)
        .attr("to", super::receipts::ack_wire_address(node, "from", from))
        .attr("class", "notification")
        .attr("type", kind);
    if let Some(participant) = participant {
        ack = ack.attr(
            "participant",
            super::receipts::ack_wire_address(node, "participant", participant),
        );
    }
    Ok(Some(ack))
}

// APK43.72 F0R.A00 -> 0Tp.A0A -> 0Dd.A00 rejects text whose last @ is
// at index zero. Keep that boundary in supported notification ACK builders.
// Native nullable-address policy remains separate; see notification-ack-audit.md.
pub(super) fn ack_route(node: &Node, name: &str) -> Option<crate::model::jid::Jid> {
    // Native types2/8/9/11 are not protocol-compliant at the final writer.
    // Parsed metadata alone must not grant an unwritable ACK route.
    if node
        .get_attr(name)
        .is_some_and(super::messaging::ack_attribute_is_known_unwritable)
    {
        return None;
    }
    if node
        .get_attr(name)?
        .as_text()
        .is_some_and(|text| text.rfind('@') == Some(0))
    {
        None
    } else {
        // Retain existing parsed metadata where available; the validated native
        // nonpositive profile supplies formerly unparseable signed/Unicode forms.
        jid(node, name)
            .or_else(|| {
                node.get_attr(name)
                    .and_then(super::receipts::explicit_nonpositive_ack_address)
                    .map(|wire| wire.to_non_ad())
            })
            // 43.72 0De.A02 constructs both lid/hosted.lid users through
            // 0Dx. A generic Rust Jid can carry values that constructor rejects.
            // This does not change device grammar or ACK wire normalization.
            .filter(|peer| {
                if peer.user.is_empty() || !matches!(peer.server, Server::Lid | Server::HostedLid) {
                    return true;
                }
                // Native parsing strips ignored dot qualifiers before user
                // construction; Rust's generic retained spelling may differ.
                match node
                    .get_attr(name)
                    .and_then(|value| value.as_text())
                    .and_then(super::messaging::ack_user_text)
                {
                    Some(
                        super::messaging::AckUserText::User(_)
                        | super::messaging::AckUserText::Device(_),
                    ) => true,
                    Some(_) => false,
                    None => super::messaging::native_plain_lid_user_valid(&peer.user),
                }
            })
    }
}

// 10G.A09 requests UserJid. 0Dd.A00 tests qualifier presence, not numeric
// value: explicit .0/:0 still selects DeviceJid, which is not a UserJid.
// Binary zero-device values are distinct (123.A02/A04 reconstruct a user).
fn token_peer(node: &Node, name: &str) -> Option<crate::model::jid::Jid> {
    if node.get_attr(name)?.as_text().is_some_and(|text| {
        text.rsplit_once('@')
            .is_some_and(|(user, _)| user.contains(['.', ':']))
    }) {
        return None;
    }
    jid(node, name).filter(|peer| {
        !peer.user.is_empty()
            && matches!(peer.server, Server::User | Server::Lid)
            // 43.72 10G.A09 requests UserJid; a LID user is constructed
            // through 0Dx (1..15 ASCII digits, no leading zero). Generic
            // Jid parsing alone does not establish that native class.
            && (peer.server != Server::Lid
                || super::messaging::native_plain_lid_user_valid(&peer.user))
            && peer.agent == 0
            && peer.device == 0
    })
}

/// Apply supported tokens before acknowledging this known notification type.
/// APK 43.72 10G.A09 skips token kinds other than trusted_contact; its
/// 0v6.A01 base still ACKs after dispatch. Unsupported child kinds do not
/// suppress that ACK (docs/protocol/notification-ack-audit.md). Unknown
/// notification types and locally refused wire/store state stay separate.
pub fn handle(store: &ProtocolStore, node: &Node) -> Result<Option<Node>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("privacy_token")
        || node.get_attr("participant").is_some()
        || node.get_attr("recipient").is_some()
    {
        return Ok(None);
    }
    let Some(from) = token_peer(node, "from") else {
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
    // APK43.72 10G.A09 requests UserJid for from/sender_lid. Rust's
    // generic parser can also carry a legacy agent qualifier; that is not
    // primary token authority (the LID domain bit is Server::Lid, not agent).
    let Some(from) = token_peer(node, "from") else {
        return Ok(None);
    };
    // APK43.72 10G.A09 uses0Tp.A0G/A0F: first named container,
    // not a uniqueness scan and not a fallback to a later valid container.
    let tokens = node
        .child("tokens")
        .ok_or_else(|| Error::invalid("notification", "missing or excessive token list"))?;
    if tokens.child_nodes().len() > 256 {
        return Err(Error::invalid(
            "notification",
            "missing or excessive token list",
        ));
    }
    let mut supported = tokens
        .child_nodes()
        .iter()
        .filter(|token| {
            token.description == "token"
                && token.get_attr("type").and_then(|v| v.as_text()) == Some("trusted_contact")
        })
        .peekable();
    // APK43.72 10G.A09 consumes sender_lid/time only for trusted_contact.
    // No selected data means no token target or store access is needed.
    if supported.peek().is_none() {
        return Ok(None);
    }
    let target = if node.get_attr("sender_lid").is_some() {
        token_peer(node, "sender_lid")
            .filter(|peer| peer.server == Server::Lid)
            .ok_or_else(|| Error::invalid("notification", "invalid sender LID"))?
    } else {
        from
    };
    let timestamp = |node: &Node| {
        node.get_attr("t")
            .and_then(|v| v.as_text())
            // APK 37.71 11P.A09 -> 0hs.A09/A08 uses Long.parseLong;
            // retain the local nonnegative storage bound after Java parsing.
            .and_then(super::mex::java_decimal_long)
            .and_then(|v| u64::try_from(v).ok())
    };
    let mut updates = Vec::new();
    for token in supported {
        let NodeContent::Bytes(bytes) = &token.content else {
            return Err(Error::invalid("notification", "token data missing"));
        };
        if bytes.is_empty() || bytes.len() > 4096 {
            return Err(Error::invalid(
                "notification",
                "token size outside local bound",
            ));
        }
        // APK43.72 10G.A09 reads parent time only inside the supported
        // trusted_contact branch. Empty/ignored lists need no timestamp.
        let parent_time = timestamp(node)
            .ok_or_else(|| Error::invalid("notification", "invalid notification timestamp"))?;
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
    fn notification_ack_refuses_native_noncompliant_classes_but_keeps_devices() {
        use crate::binary::{AttributeValue, Decoder, Encoder, TokenDictionary};
        for kind in ["psa", "business", "picture", "mex", "disappearing_mode"] {
            let base = Node::new("notification")
                .attr("type", kind)
                .attr("id", "COMPLIANCE")
                .attr("from", "s.whatsapp.net");
            for address in [
                "Server@s.whatsapp.net",
                "Server@hosted",
                "gdpr@s.whatsapp.net",
                "a-b@temp",
            ] {
                for value in [
                    AttributeValue::from(address),
                    AttributeValue::from(crate::model::jid::Jid::parse(address).unwrap()),
                ] {
                    for field in ["from", "participant"] {
                        let input = base.clone().attr(field, value.clone());
                        let dict = TokenDictionary::comparison_2_26_37_71();
                        let wire = Encoder::new(dict).encode(&input).unwrap();
                        let input =
                            Decoder::new(&wire[1..], dict, &crate::profile::Limits::default())
                                .decode_node()
                                .unwrap();
                        assert!(base_ack(&input, kind).is_err(), "{kind}/{field}/{address}");
                    }
                }
            }
            // Native Device17/24 remains compliant even with a ServiceUser base.
            for address in [
                "0@s.whatsapp.net",
                "Server:0@s.whatsapp.net",
                "Server:99@hosted",
            ] {
                assert!(
                    base_ack(&base.clone().attr("from", address), kind)
                        .unwrap()
                        .is_some(),
                    "{address}"
                );
            }
        }
    }

    #[test]
    fn notification_ack_routes_refuse_invalid_constructed_lid_users() {
        use crate::{binary::AttributeValue, model::jid::Jid};
        for kind in ["psa", "business", "picture", "mex", "disappearing_mode"] {
            for field in ["from", "participant"] {
                for server in ["lid", "hosted.lid"] {
                    for user in ["letters-private", "0", "0123", "1234567890123456", "１２３"] {
                        let text = format!("{user}@{server}");
                        for value in [
                            AttributeValue::from(text.as_str()),
                            AttributeValue::from(Jid::parse(&text).unwrap()),
                        ] {
                            let node = Node::new("notification")
                                .attr("type", kind)
                                .attr("id", "GRAMMAR")
                                .attr("from", "s.whatsapp.net")
                                .attr(field, value);
                            let error = base_ack(&node, kind)
                                .expect_err("invalid constructed LID became ACK route");
                            assert!(!error.to_string().contains("letters-private"));
                        }
                    }
                }
                for user in ["1", "999999999999999"] {
                    let text = format!("{user}@lid");
                    let node = Node::new("notification")
                        .attr("type", kind)
                        .attr("id", "VALID")
                        .attr("from", "s.whatsapp.net")
                        .attr(field, text);
                    assert!(base_ack(&node, kind).unwrap().is_some());
                }
            }
        }
    }

    #[test]
    fn base_ack_refuses_empty_user_text_without_rejecting_server_only_routes() {
        // APK43.72 0Dd.A00 rejects an @ at index zero, whereas a bare server
        // is a distinct accepted input. Never repair malformed text into a route.
        for kind in ["psa", "business", "picture", "mex", "disappearing_mode"] {
            let incoming = Node::new("notification")
                .attr("type", kind)
                .attr("id", "EMPTY-USER")
                .attr("from", "s.whatsapp.net");
            assert!(base_ack(&incoming, kind).unwrap().is_some());
            // Native type11 is not protocol-compliant; decode support must not
            // grant it notification ACK routing authority.
            for name in ["status_me", "lid_me"] {
                let peer = crate::model::jid::Jid::new("", Server::Other(name.into())).unwrap();
                assert!(base_ack(&incoming.clone().attr("from", peer.clone()), kind).is_err());
                assert!(base_ack(&incoming.clone().attr("participant", peer), kind).is_err());
            }
            for invalid in ["@s.whatsapp.net", "@lid", "@g.us"] {
                assert!(base_ack(&incoming.clone().attr("from", invalid), kind).is_err());
                assert!(base_ack(&incoming.clone().attr("participant", invalid), kind).is_err());
            }
        }
    }

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
    fn active_bad_expiry_preserves_lock_but_explicit_inactive_clears_without_expiry() {
        let dir = tempfile::tempdir().unwrap();
        let store = super::super::group_fanout::tests::open(&dir.path().join("lock.redb"));
        let original = ReachoutTimelock {
            active: true,
            ends: 1000,
        };
        store
            .write(|tx| tx.put_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1, &original))
            .unwrap();
        for ends in [
            serde_json::Value::Null,
            serde_json::json!("bad"),
            serde_json::json!(-1),
            serde_json::json!(1.5),
            serde_json::json!("9223372036854775808"),
        ] {
            for active in [true, false] {
                let body = serde_json::json!({"data":{"xwa2_notify_account_reachout_timelock":{
                    "is_active":active,"time_enforcement_ends":ends}}});
                let node = Node::new("notification")
                    .attr("type", "mex")
                    .attr("id", "fixture-reachout")
                    .attr("from", "s.whatsapp.net")
                    .children(vec![
                        Node::new("update")
                            .attr("op_name", "NotificationUserReachoutTimelockUpdate")
                            .bytes(serde_json::to_vec(&body).unwrap()),
                    ]);
                if active {
                    assert!(matches!(
                        record_reachout_timelock(&store, &node),
                        Err(Error::InvalidInput { .. })
                    ));
                    assert_eq!(
                        active_reachout_timelock(&store, 999).unwrap(),
                        Some(original)
                    );
                } else {
                    assert_eq!(
                        record_reachout_timelock(&store, &node).unwrap(),
                        Some(ReachoutTimelock {
                            active: false,
                            ends: 0
                        })
                    );
                    assert!(active_reachout_timelock(&store, 999).unwrap().is_none());
                    store
                        .write(|tx| {
                            tx.put_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1, &original)
                        })
                        .unwrap();
                }
            }
        }
    }

    #[test]
    fn reachout_dispatch_uses_first_update_and_ascii_case_matching() {
        let dir = tempfile::tempdir().unwrap();
        let store = super::super::group_fanout::tests::open(&dir.path().join("dispatch.redb"));
        let update = |tag| {
            Node::new(tag).attr("op_name", "notificationuserreachouttimelockupdate")
            .bytes(br#"{"data":{"xwa2_notify_account_reachout_timelock":{"is_active":true,"time_enforcement_ends":"1000"}}}"#.to_vec())
        };
        let wrap = |children| {
            Node::new("notification")
                .attr("type", "mex")
                .attr("id", "fixture-reachout")
                .attr("from", "s.whatsapp.net")
                .children(children)
        };
        assert!(
            record_reachout_timelock(&store, &wrap(vec![update("other")]))
                .unwrap()
                .is_none()
        );
        assert!(
            record_reachout_timelock(
                &store,
                &wrap(vec![
                    Node::new("update").attr("op_name", "unhandled"),
                    update("update")
                ])
            )
            .unwrap()
            .is_none()
        );
        assert!(active_reachout_timelock(&store, 999).unwrap().is_none());
        assert_eq!(
            record_reachout_timelock(&store, &wrap(vec![update("other"), update("update")]))
                .unwrap(),
            Some(ReachoutTimelock {
                active: true,
                ends: 1000
            })
        );
    }

    #[test]
    fn invalid_reachout_envelope_cannot_clear_existing_restriction() {
        let dir = tempfile::tempdir().unwrap();
        let store = super::super::group_fanout::tests::open(&dir.path().join("envelope.redb"));
        let prior = ReachoutTimelock {
            active: true,
            ends: 1000,
        };
        store
            .write(|tx| tx.put_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1, &prior))
            .unwrap();
        let base = Node::new("notification").attr("type", "mex").children(vec![
            Node::new("update")
                .attr("op_name", "NotificationUserReachoutTimelockUpdate")
                .bytes(
                    br#"{"data":{"xwa2_notify_account_reachout_timelock":{"is_active":false}}}"#
                        .to_vec(),
                ),
        ]);
        for node in [
            base.clone(),
            base.clone().attr("id", "id"),
            base.clone().attr("from", "s.whatsapp.net"),
            base.clone().attr("id", "").attr("from", "s.whatsapp.net"),
            base.clone()
                .attr("id", "id")
                .attr("from", "1:invalid@s.whatsapp.net"),
            base.clone()
                .attr("id", "id")
                .attr("from", "s.whatsapp.net")
                .attr("participant", "1:invalid@s.whatsapp.net"),
        ] {
            assert!(record_reachout_timelock(&store, &node).is_err());
            assert_eq!(active_reachout_timelock(&store, 999).unwrap(), Some(prior));
        }
        let valid = base.attr("id", "id").attr("from", "s.whatsapp.net");
        assert_eq!(
            record_reachout_timelock(&store, &valid).unwrap(),
            Some(ReachoutTimelock {
                active: false,
                ends: 0
            })
        );
        assert!(mex_ack(&valid).unwrap().is_some());
    }

    #[test]
    fn recognized_malformed_reachout_body_refuses_without_replacing_lock() {
        let dir = tempfile::tempdir().unwrap();
        let store = super::super::group_fanout::tests::open(&dir.path().join("body.redb"));
        let prior = ReachoutTimelock {
            active: true,
            ends: 1000,
        };
        store
            .write(|tx| tx.put_json(Table::Checkpoints, REACHOUT_TIMELOCK, 1, &prior))
            .unwrap();
        let op = || Node::new("update").attr("op_name", "NotificationUserReachoutTimelockUpdate");
        let wrap = |child| {
            Node::new("notification")
                .attr("type", "mex")
                .attr("id", "body-test")
                .attr("from", "s.whatsapp.net")
                .children(vec![child])
        };
        for child in [
            op(),
            op().bytes(b"{".to_vec()),
            op().bytes(b"null".to_vec()),
            op().bytes(b"{}".to_vec()),
            op().bytes(
                br#"{"data":{"xwa2_notify_account_reachout_timelock":{"is_active":"false"}}}"#
                    .to_vec(),
            ),
        ] {
            let err = record_reachout_timelock(&store, &wrap(child)).unwrap_err();
            assert!(matches!(err, Error::InvalidInput { .. }));
            assert_eq!(active_reachout_timelock(&store, 999).unwrap(), Some(prior));
        }
        // Unknown operations remain separate from malformed recognized ones.
        let unknown = Node::new("update")
            .attr("op_name", "unimplemented")
            .bytes(b"{".to_vec());
        assert!(
            record_reachout_timelock(&store, &wrap(unknown))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn reachout_expiry_uses_java_decimal_string_rules() {
        let dir = tempfile::tempdir().unwrap();
        let store = super::super::group_fanout::tests::open(&dir.path().join("decimal.redb"));
        let update = |text: &str| {
            Node::new("notification").attr("type", "mex").attr("id", "decimal").attr("from", "s.whatsapp.net").children(vec![
            Node::new("update").attr("op_name", "NotificationUserReachoutTimelockUpdate").bytes(serde_json::to_vec(&serde_json::json!({"data":{"xwa2_notify_account_reachout_timelock":{"is_active":true,"time_enforcement_ends":text}}})).unwrap())])
        };
        for text in ["+1000", "١٠٠٠", "１０００", "1٠０0"] {
            assert_eq!(
                record_reachout_timelock(&store, &update(text)).unwrap(),
                Some(ReachoutTimelock {
                    active: true,
                    ends: 1000
                })
            );
        }
        for text in [
            " 1000",
            "1000 ",
            "0x3e8",
            "1_000",
            "𝟙𝟘𝟘𝟘",
            "9223372036854775808",
            "-1",
        ] {
            assert!(record_reachout_timelock(&store, &update(text)).is_err());
            assert_eq!(
                active_reachout_timelock(&store, 999).unwrap(),
                Some(ReachoutTimelock {
                    active: true,
                    ends: 1000
                })
            );
        }
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
        let node = Node::new("notification")
            .attr("type", "mex")
            .attr("id", "fixture-reachout")
            .attr("from", "s.whatsapp.net")
            .children(vec![
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
            let qualified = Jid {
                user: "15555550100".into(),
                server: server.clone(),
                agent: 1,
                device: 0,
                kind: JidKind::AgentDevice,
            };
            for node in [
                notification().attr("from", qualified.clone()),
                notification().attr("from", qualified.to_string()),
            ] {
                assert_eq!(record(&store, &node).unwrap(), 0);
                assert!(handle(&store, &node).unwrap().is_none());
            }
            // Text qualifiers select native DeviceJid even when their value is0.
            // They cannot supply UserJid authority for incoming tokens.
            for local in ["15555550100:0", "15555550100.0", "15555550100.0:0"] {
                let node = notification()
                    .attr("t", "30")
                    .attr("from", format!("{local}@{}", server.as_str()));
                assert_eq!(record(&store, &node).unwrap(), 0);
                assert!(handle(&store, &node).unwrap().is_none());
            }
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
                user: "15555550100".into(),
                server: Server::Lid,
                agent: 1,
                device: 0,
                kind: JidKind::AgentDevice,
            },
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
        for local in ["10000000001:0", "10000000001.0", "10000000001.0:0"] {
            let node = notification().attr("sender_lid", format!("{local}@lid"));
            assert!(record(&store, &node).is_err());
            assert!(handle(&store, &node).is_err());
        }
        assert_eq!(snapshot(&store), before);
        drop(store);
        assert_eq!(snapshot(&open(false)), before);
    }

    #[test]
    fn privacy_token_lid_constructor_grammar_precedes_storage_and_ack() {
        use crate::{
            binary::AttributeValue,
            model::{ids::AccountId, jid::Jid},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        for user in [
            "letters-private",
            "0",
            "01234",
            "1234567890123456",
            "１２３",
            "1",
            "999999999999999",
        ] {
            let valid = matches!(user, "1" | "999999999999999");
            for field in ["from", "sender_lid"] {
                for typed in [false, true] {
                    let dir = tempfile::tempdir().unwrap();
                    let open = |create| {
                        ProtocolStore::open(OpenOptions {
                            path: dir.path().join("lid-grammar.redb"),
                            account_id: AccountId::new("test").unwrap(),
                            create,
                            key_source: std::sync::Arc::new(StaticKeySource::new(
                                MasterKey::from_bytes([4; 32]),
                            )),
                        })
                        .unwrap()
                    };
                    let store = open(true);
                    store
                        .write(|tx| {
                            tx.put_json(Table::PeerIdentities, b"sentinel", 1, &true)?;
                            tx.put_json(Table::SignalSessions, b"sentinel", 1, &true)
                        })
                        .unwrap();
                    let text = format!("{user}@lid");
                    let address = if typed {
                        AttributeValue::from(Jid::parse(&text).unwrap())
                    } else {
                        AttributeValue::from(text.as_str())
                    };
                    let node = Node::new("notification")
                        .attr("type", "privacy_token")
                        .attr("from", "15555550100@s.whatsapp.net")
                        .attr("id", "fixture")
                        .attr("t", "20")
                        .attr(field, address)
                        .children(vec![Node::new("tokens").children(
                            vec![Node::new("token").attr("type", "trusted_contact").bytes(vec![1])],
                        )]);
                    super::super::inbox::preserve(&store, &node).unwrap();
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
                    let head = store.journal_head().unwrap();
                    if valid {
                        assert_eq!(record(&store, &node).unwrap(), 1);
                        assert!(handle(&store, &node).unwrap().is_some());
                    } else if field == "from" {
                        assert_eq!(
                            record(&store, &node).unwrap(),
                            0,
                            "invalid LID became token owner"
                        );
                        assert!(handle(&store, &node).unwrap().is_none());
                    } else {
                        assert!(validate(&node).is_err());
                        let error = handle(&store, &node).unwrap_err();
                        assert!(!error.to_string().contains("letters-private"));
                    }
                    if !valid {
                        assert_eq!(snapshot(&store), before);
                        assert_eq!(store.journal_head().unwrap(), head);
                    }
                    let saved = snapshot(&store);
                    drop(store);
                    assert_eq!(snapshot(&open(false)), saved);
                }
            }
        }
    }

    #[test]
    fn mixed_privacy_tokens_commit_supported_entries_and_still_ack_known_type() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("mixed.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [9; 32],
                ))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        let notification = |tokens| {
            Node::new("notification")
                .attr("type", "privacy_token")
                .attr("id", "MIXED")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("t", "20")
                .children(vec![Node::new("tokens").children(tokens)])
        };
        let supported = || {
            Node::new("token")
                .attr("type", "trusted_contact")
                .bytes(vec![4, 5])
        };
        let node = notification(vec![
            supported(),
            Node::new("token").attr("type", "future_kind"),
        ]);
        validate(&node).unwrap();
        let ack = handle(&store, &node)
            .unwrap()
            .expect("known notification gets ACK after processing");
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("MIXED"));
        assert_eq!(
            ack.get_attr("type").and_then(|v| v.as_text()),
            Some("privacy_token")
        );
        assert_eq!(
            ack.get_attr("class").and_then(|v| v.as_text()),
            Some("notification")
        );
        let saved = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"incoming-trusted-contact:15555550100@s.whatsapp.net",
                        1,
                    )
                })
                .unwrap()
        };
        let expected = Some(serde_json::json!({"timestamp":20,"token":[4,5]}));
        assert_eq!(saved(&store), expected);
        for tokens in [vec![Node::new("token").attr("type", "future_kind")], vec![]] {
            for unused_time in [None, Some("bad"), Some("-1"), Some("9223372036854775808")] {
                let mut skipped = notification(tokens.clone());
                skipped.attributes.retain(|attribute| attribute.name != "t");
                if let Some(time) = unused_time {
                    skipped = skipped.attr("t", time);
                }
                for unused_alias in [
                    Some("letters-private@lid"),
                    Some("01234@lid"),
                    Some("1234567890123456@lid"),
                    None,
                    Some("1:invalid@lid"),
                    Some("15555550101.1:0@lid"),
                    Some("10000000001:0@lid"),
                    Some("10000000001.0@lid"),
                ] {
                    let mut ignored = skipped.clone();
                    if let Some(alias) = unused_alias {
                        ignored = ignored.attr("sender_lid", alias);
                    }
                    validate(&ignored).unwrap();
                    assert!(handle(&store, &ignored).unwrap().is_some());
                    assert_eq!(
                        saved(&store),
                        expected,
                        "ignored kinds must not consume unused metadata or create state"
                    );
                }
            }
        }
        let mut extra = node.clone();
        let NodeContent::Children(children) = &mut extra.content else {
            unreachable!()
        };
        children.push(Node::new("future_extension"));
        assert!(handle(&store, &extra).unwrap().is_some());
        for later in [
            Node::new("tokens").children(vec![supported().attr("t", "30").bytes(vec![9])]),
            Node::new("tokens").children(vec![Node::new("token").attr("type", "trusted_contact")]),
        ] {
            let mut children = node.child_nodes().to_vec();
            children.insert(0, Node::new("future_extension"));
            children.push(later);
            let extended = node.clone().children(children);
            validate(&extended).unwrap();
            assert!(handle(&store, &extended).unwrap().is_some());
            assert_eq!(
                saved(&store),
                expected,
                "later token containers cannot replace selected state"
            );
        }
        // A malformed supported entry still refuses before any partial commit.
        let malformed = notification(vec![
            supported(),
            Node::new("token").attr("type", "trusted_contact"),
        ]);
        let mut children = malformed.child_nodes().to_vec();
        children.push(Node::new("tokens").children(vec![supported().attr("t", "30")]));
        assert!(
            handle(&store, &malformed.clone().children(children)).is_err(),
            "a later valid container cannot rescue a malformed first one"
        );
        assert!(validate(&malformed).is_err());
        assert!(handle(&store, &malformed).is_err());
        assert_eq!(saved(&store), expected);
        assert!(
            handle(&store, &node.clone().attr("type", "unknown_notification"))
                .unwrap()
                .is_none()
        );
        assert!(
            handle(&store, &node.clone().attr("id", ""))
                .unwrap()
                .is_none()
        );
        assert!(
            handle(&store, &node.clone().attr("from", "s.whatsapp.net"))
                .unwrap()
                .is_none()
        );
        drop(store);
        assert_eq!(
            saved(&open(false)),
            expected,
            "supported update precedes ACK and survives reopening"
        );
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
