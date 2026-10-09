//! Explicit opt-in normal outbox adapter for comparison primary text/uploaded media.
//! One key query at a time; no application retries or replay of uncertain sends.
use crate::{
    Error, Result,
    binary::{Encoder, Node, TokenDictionary},
    model::{
        ids::{LocalMessageId, MessageId},
        jid::Server,
        message::MessageContent,
        send::{AttemptProgress, DeliveryFact, DeliveryFactKind},
        time::Timestamp,
    },
    outbox::{AttemptRecord, Outbox, OutboxEntry},
    profile::Limits,
    session::AccountRecord,
    store::{ProtocolStore, Table},
    transport::ConnectionEpoch,
};
use zeroize::Zeroizing;

fn outbox() -> Outbox {
    Outbox::new("comparison-driver", Limits::default())
}
fn query_key(id: LocalMessageId) -> String {
    format!("outbox-key-query:{id}")
}

/// Redacted queued intent suspended by a fetched peer identity mismatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdentitySuspension {
    /// Existing local intent selector; never create another ID to retry.
    pub local_id: LocalMessageId,
    /// SHA-256 of the session address, not a phone number or safety number.
    pub peer_ticket: String,
    /// Static local reason; old records default to identity_changed.
    #[serde(default = "identity_change_reason")]
    pub reason: String,
}

fn identity_change_reason() -> String {
    "identity_changed".into()
}

/// Inspect identity-suspended queued sends offline, without replaying queries.
pub fn identity_suspended(store: &ProtocolStore) -> Result<Vec<IdentitySuspension>> {
    store.read(|tx| {
        let mut result = Vec::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if rows.is_empty() {
                break;
            }
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, value) in rows {
                if !key.starts_with(b"outbox-key-query:") {
                    continue;
                }
                let row: serde_json::Value = serde_json::from_slice(&value)
                    .map_err(|_| Error::invalid("outbox", "invalid saved query outcome"))?;
                if row["status"] != "identity_changed" {
                    continue;
                }
                let suspension: IdentitySuspension = serde_json::from_value(row)
                    .map_err(|_| Error::invalid("outbox", "invalid saved identity suspension"))?;
                if outbox()
                    .get(tx, suspension.local_id)?
                    .is_some_and(|entry| entry.progress == AttemptProgress::Queued)
                {
                    result.push(suspension);
                }
            }
        }
        Ok(result)
    })
}

/// Inspect one identity-suspended queued intent without exposing its saved reply.
pub fn identity_suspension(
    store: &ProtocolStore,
    id: LocalMessageId,
) -> Result<Option<IdentitySuspension>> {
    store.read(|tx| {
        let Some(entry) = outbox().get(tx, id)? else {
            return Ok(None);
        };
        if entry.progress != AttemptProgress::Queued {
            return Ok(None);
        }
        let row: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, query_key(id).as_bytes(), 1)?;
        let Some(row) = row.filter(|row| row["status"] == "identity_changed") else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_value(row).map_err(|_| {
            Error::invalid("outbox", "invalid saved identity suspension")
        })?))
    })
}

/// A pending query is connection-local; its reservation survives cancellation.
pub struct Pending {
    local_id: LocalMessageId,
    query_id: String,
    /// Bounded response deadline; expiry never causes a retry.
    pub deadline: tokio::time::Instant,
}
impl std::fmt::Debug for Pending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pending")
            .field("deadline", &self.deadline)
            .finish_non_exhaustive()
    }
}

/// Whether an already answered identity query can be used under this explicit
/// policy. Unknown/refused queries and invalid signatures are never replayed.
pub fn identity_query_resumable(
    row: &serde_json::Value,
    policy: crate::registration::request::IdentityChangePolicy,
) -> bool {
    policy == crate::registration::request::IdentityChangePolicy::ApkAuthenticated
        && row["status"] == "identity_changed"
        && matches!(row["reason"].as_str(), None | Some("identity_changed"))
        && row["reply"].is_array()
        && row["pinned_fingerprint"]
            .as_str()
            .is_some_and(|v| v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Resolve one explicitly selected suspended intent using its original saved
/// directory reply. No network query, new intent ID or replay of uncertain
/// sends occurs. The returned stanza is already durably reserved before write.
pub async fn resume_identity_selected(
    store: &ProtocolStore,
    account: &AccountRecord,
    id: LocalMessageId,
    epoch: ConnectionEpoch,
    policy: crate::registration::request::IdentityChangePolicy,
) -> Result<Option<Node>> {
    if policy == crate::registration::request::IdentityChangePolicy::Pinned {
        return Ok(None);
    }
    let row: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, query_key(id).as_bytes(), 1))?;
    let Some(row) = row.filter(|row| identity_query_resumable(row, policy)) else {
        return Ok(None);
    };
    let entry = store
        .read(|tx| outbox().get(tx, id))?
        .ok_or_else(|| Error::invalid("outbox", "missing suspended intent"))?;
    if entry.progress != AttemptProgress::Queued {
        return Ok(None);
    }
    if super::inbox::remote_identity_policy(store, account, entry.intent.chat.jid(), policy)?
        == crate::registration::request::IdentityChangePolicy::Pinned
    {
        return Ok(None);
    }
    let query_id = row["query_id"]
        .as_str()
        .ok_or_else(|| Error::invalid("outbox", "suspended query has no selector"))?
        .to_owned();
    let reply: Vec<u8> = serde_json::from_value(row["reply"].clone())
        .map_err(|_| Error::invalid("outbox", "invalid saved directory reply"))?;
    let limits = Limits::default();
    let raw = crate::binary::unpack_frame_payload(&reply, &limits)?;
    let reply =
        crate::binary::Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits)
            .decode_node()?;
    let pending = Pending {
        local_id: id,
        query_id,
        deadline: tokio::time::Instant::now(),
    };
    pending
        .prepare_with_policy(store, account, &reply, epoch, policy)
        .await
}

/// Reserve one eligible plain direct text intent before sending its key query.
pub fn begin(store: &ProtocolStore) -> Result<Option<(Pending, Node)>> {
    begin_selected(store, None)
}
fn unreserved_queued(tx: &crate::store::ReadTx<'_>, entry: &OutboxEntry) -> Result<bool> {
    Ok(entry.progress == AttemptProgress::Queued
        && tx
            .get_json::<serde_json::Value>(
                Table::Checkpoints,
                query_key(entry.local_id).as_bytes(),
                1,
            )?
            .is_none()
        && !super::outgoing_group::reserved_in(tx, entry.local_id)?)
}
/// Queued intents an outbox-enabled comparison session would attempt next, in
/// accept order across every idempotency namespace: `Queued` progress and no
/// durable key-query reservation. Read-only; `begin_selected` applies the same
/// filter before claiming, then fails unsupported content locally.
pub fn eligible(store: &ProtocolStore, limit: usize) -> Result<Vec<OutboxEntry>> {
    store.read(|tx| outbox().pending_where(tx, limit, |entry| unreserved_queued(tx, entry)))
}
/// Restrict transmission to one explicitly selected durable intent when set.
pub fn begin_selected(
    store: &ProtocolStore,
    ticket: Option<LocalMessageId>,
) -> Result<Option<(Pending, Node)>> {
    let entries = if let Some(id) = ticket {
        store.read(|tx| outbox().get(tx, id))?.into_iter().collect()
    } else {
        eligible(store, 256)?
    };
    for entry in entries {
        if entry.progress != AttemptProgress::Queued {
            continue;
        }
        let peer = entry.intent.chat.jid();
        // Groups require membership, selected device batches and sender keys;
        // the shared connection actor dispatches them to outgoing_group.
        if peer.server == Server::Group {
            continue;
        }
        let supported = peer.server == Server::User
            && peer.device == 0
            && !peer.user.is_empty()
            && match &entry.intent.content {
                MessageContent::Text { body, quoted, .. } => {
                    !body.is_empty()
                        && body.len() <= 4096
                        && quoted
                            .as_ref()
                            .is_none_or(|q| q.key.chat.jid().to_non_ad() == peer.to_non_ad())
                }
                MessageContent::Media {
                    reference,
                    caption: None,
                    file_name: None,
                } => {
                    super::outgoing_media::resolve(store, &entry.client_request_id, peer, reference)
                        .is_ok()
                }
                MessageContent::Reaction { target, emoji } => {
                    super::reactions::validate(peer, target, emoji.as_deref()).is_ok()
                }
                MessageContent::Poll {
                    name,
                    options,
                    selectable_count,
                } => super::polls::validate_poll(name, options, *selectable_count).is_ok(),
                MessageContent::PollVote { target, .. } => {
                    target.key.chat.jid().to_non_ad() == peer.to_non_ad()
                }
                MessageContent::DisappearingTimer { .. } | MessageContent::ChatPrivacy { .. } => {
                    true
                }
                MessageContent::KeepInChat { target, .. } => {
                    super::chat_settings::validate_keep(peer, target).is_ok()
                }
                _ => false,
            };
        let reserved = store
            .read(|tx| {
                tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    query_key(entry.local_id).as_bytes(),
                    1,
                )
            })?
            .is_some();
        if reserved {
            continue;
        }
        super::send_path::claim(
            store,
            &entry.client_request_id,
            super::send_path::SendPath::Outbox,
        )?;
        let query_id = super::messaging::new_message_id()?;
        let won = store.write(|tx| {
            let Some(mut current) = outbox().get_mut(tx, entry.local_id)? else {
                return Ok(false);
            };
            if current.progress != AttemptProgress::Queued
                || tx
                    .get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        query_key(entry.local_id).as_bytes(),
                        1,
                    )?
                    .is_some()
            {
                return Ok(false);
            }
            if !supported {
                outbox().set_progress(tx, &mut current, AttemptProgress::Failed)?;
                outbox().record_fact(
                    tx,
                    &mut current,
                    DeliveryFact {
                        kind: DeliveryFactKind::Failed,
                        device: None,
                        observed_at: Timestamp::now(),
                        code: None,
                    },
                )?;
                return Ok(false);
            }
            tx.put_json(
                Table::Checkpoints,
                query_key(entry.local_id).as_bytes(),
                1,
                &serde_json::json!({"status":"outcome_unknown","query_id":query_id}),
            )?;
            Ok(true)
        })?;
        if won {
            return Ok(Some((
                Pending {
                    local_id: entry.local_id,
                    query_id: query_id.clone(),
                    deadline: tokio::time::Instant::now() + std::time::Duration::from_secs(30),
                },
                super::messaging::key_request(peer, &query_id),
            )));
        }
    }
    Ok(None)
}

/// Our identities for poll-vote crypto: the saved login identity, else the
/// account's own JID.
fn login_identity(
    tx: &crate::store::WriteTx<'_>,
    account: &AccountRecord,
) -> Result<(
    Option<crate::model::jid::Jid>,
    Option<crate::model::jid::Jid>,
)> {
    let saved: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, b"login-identity", 1)?;
    let parse = |field: &str| {
        saved
            .as_ref()
            .and_then(|s| s[field].as_str())
            .and_then(|v| crate::model::jid::Jid::parse(v).ok())
            .map(|j| j.to_non_ad())
    };
    let pn = parse("pn").or_else(|| account.own_jid.as_ref().map(|j| j.to_non_ad()));
    Ok((pn, parse("lid")))
}

/// Encrypt this account's vote on `target`. The poll secret comes from the
/// received poll's plaintext (their poll) or from `poll-secret:<id>` (ours).
/// Identities follow the chat's addressing: the contact's LID when known,
/// our LID from the login reply when known, else phone-number JIDs.
fn vote_payload_for(
    store: &ProtocolStore,
    account: &AccountRecord,
    peer: &crate::model::jid::Jid,
    target: &crate::model::message::MessageRef,
    selected: &[String],
) -> Result<Vec<u8>> {
    use crate::model::message::Direction;
    if target.key.chat.jid().to_non_ad() != peer.to_non_ad() {
        return Err(Error::invalid("poll", "vote targets another chat"));
    }
    let poll_id = target.key.id.as_str().to_owned();
    // Validate actual saved provenance before identity lookup or vote entropy.
    // Caller-provided target.chat alone cannot authorize another poll's secret.
    let own_record = if target.key.direction == Direction::Outgoing {
        Some(store.write(|tx| {
            let own: serde_json::Value = tx
                .get_json(
                    Table::Checkpoints,
                    format!("poll-secret:{poll_id}").as_bytes(),
                    1,
                )?
                .ok_or_else(|| Error::invalid("poll", "unknown own poll"))?;
            let chat = super::polls::saved_chat(&own)?
                .ok_or_else(|| Error::invalid("poll", "saved poll has no chat binding"))?;
            if !super::inbox_events::authenticated_direct_chat(tx, peer, &chat)? {
                return Err(Error::invalid("poll", "saved poll belongs to another chat"));
            }
            Ok(own)
        })?)
    } else {
        None
    };
    let (own_pn, own_lid) = store.write(|tx| login_identity(tx, account))?;
    let peer_lid = super::inbox::lid_for(store, peer)?;
    let (secret, options, selectable, creator) = if target.key.direction == Direction::Incoming {
        let key = format!("received:{}:{poll_id}", peer.to_non_ad());
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?
            .ok_or_else(|| Error::invalid("poll", "unknown incoming poll for this chat"))?;
        let plain: Vec<u8> = serde_json::from_value(saved["protobuf"].clone())
            .map_err(|_| Error::invalid("poll", "poll record has no authenticated payload"))?;
        let poll = super::polls::decode_poll(&plain)
            .ok_or_else(|| Error::invalid("poll", "target message is not a poll"))?;
        let secret = poll
            .secret
            .ok_or_else(|| Error::invalid("poll", "poll carries no message secret"))?;
        (
            secret,
            poll.options,
            poll.selectable_count,
            peer_lid.clone().unwrap_or_else(|| peer.to_non_ad()),
        )
    } else {
        let own = own_record.ok_or_else(|| Error::invalid("poll", "unknown own poll"))?;
        let secret: Vec<u8> = serde_json::from_value(own["secret"].clone())
            .map_err(|_| Error::invalid("poll", "invalid saved poll secret"))?;
        let secret = <[u8; 32]>::try_from(secret)
            .map_err(|_| Error::invalid("poll", "invalid saved poll secret"))?;
        let options: Vec<String> =
            serde_json::from_value(own["options"].clone()).unwrap_or_default();
        let creator = own_lid
            .clone()
            .or_else(|| own_pn.clone())
            .ok_or_else(|| Error::invalid("poll", "own identity unknown"))?;
        (secret, options, 0, creator)
    };
    for name in selected {
        if !options.contains(name) {
            return Err(Error::invalid(
                "poll",
                "selected option is not part of the poll",
            ));
        }
    }
    if selectable != 0 && selected.len() > selectable as usize {
        return Err(Error::invalid("poll", "too many options selected"));
    }
    let voter = own_lid
        .or(own_pn)
        .ok_or_else(|| Error::invalid("poll", "own identity unknown"))?;
    let mut iv = [0u8; 12];
    getrandom::fill(&mut iv).map_err(|_| Error::invalid("poll", "vote entropy unavailable"))?;
    let enc = super::polls::encrypt_vote(&secret, &poll_id, &creator, &voter, selected, iv)?;
    let remote = peer_lid.unwrap_or_else(|| peer.to_non_ad());
    Ok(super::polls::vote_payload(
        &remote,
        target,
        enc,
        Timestamp::now().as_millis(),
    ))
}

/// Encode a quoted reply to `target` in the chat with `peer`. `participant` is
/// the quoted message's sender in the chat's addressing (the contact's LID when
/// known, our LID from the login reply for our own messages), matching what the
/// comparison handset sent when it quoted our message (participant = our LID).
/// `quotedMessage` is the referenced message when this store still holds it
/// (an authenticated incoming payload with `messageContextInfo` stripped, or
/// our own text intent), else an empty `conversation`.
fn reply_payload_for(
    store: &ProtocolStore,
    account: &AccountRecord,
    peer: &crate::model::jid::Jid,
    target: &crate::model::message::MessageRef,
    body: &str,
) -> Result<Vec<u8>> {
    use crate::model::message::Direction;
    if target.key.chat.jid().to_non_ad() != peer.to_non_ad() {
        return Err(Error::invalid("reply", "quoted message is in another chat"));
    }
    // The caller's key alone cannot attribute another saved entry's body.
    // The immutable SendRequest.chat attributes its body locally. APK 37.71
    // BoQ.A00 traces reaction keys, not native quoted-reply validation.
    let own_entry = if target.key.direction == Direction::Outgoing {
        let entry = store.read(|tx| outbox().by_wire_id(tx, &target.key.id))?;
        if entry
            .as_ref()
            .is_some_and(|entry| entry.intent.chat.jid().to_non_ad() != peer.to_non_ad())
        {
            return Err(Error::invalid(
                "reply",
                "saved quoted message belongs to another chat",
            ));
        }
        entry
    } else {
        None
    };
    let id = target.key.id.as_str();
    let (own_pn, own_lid) = store.write(|tx| login_identity(tx, account))?;
    let peer_lid = super::inbox::lid_for(store, peer)?;
    let (participant, quoted) = if target.key.direction == Direction::Incoming {
        let key = format!("received:{}:{id}", peer.to_non_ad());
        let saved: Option<serde_json::Value> =
            store.read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?;
        let quoted = saved
            .and_then(|s| serde_json::from_value::<Vec<u8>>(s["protobuf"].clone()).ok())
            .map(|plain| super::quotes::strip_message_context(&plain))
            .unwrap_or_else(|| super::quotes::conversation(""));
        (peer_lid.unwrap_or_else(|| peer.to_non_ad()), quoted)
    } else {
        let quoted = own_entry
            .and_then(|entry| match entry.intent.content {
                MessageContent::Text { body, .. } => Some(super::quotes::conversation(&body)),
                _ => None,
            })
            .unwrap_or_else(|| super::quotes::conversation(""));
        let own = own_lid
            .or(own_pn)
            .ok_or_else(|| Error::invalid("reply", "own identity unknown"))?;
        (own, quoted)
    };
    Ok(super::quotes::reply_payload(
        body,
        id,
        &participant.to_string(),
        quoted,
    ))
}

/// Encode a disappearing-timer, advanced-chat-privacy or keep-in-chat E2E
/// payload for `content`; `None` for any other content kind. `KeepInChat`
/// follows the same LID/phone-number rule as reactions for its target key.
fn chat_setting_payload_for(
    store: &ProtocolStore,
    peer: &crate::model::jid::Jid,
    content: &MessageContent,
) -> Result<Option<Vec<u8>>> {
    Ok(match content {
        MessageContent::DisappearingTimer { seconds } => {
            Some(super::chat_settings::disappearing_payload(*seconds))
        }
        MessageContent::ChatPrivacy { limited } => Some(
            super::chat_settings::limit_sharing_payload(*limited, Timestamp::now().as_millis()),
        ),
        MessageContent::KeepInChat { target, keep } => {
            super::chat_settings::validate_keep(peer, target)?;
            let remote = super::inbox::lid_for(store, peer)?.unwrap_or_else(|| peer.clone());
            Some(super::chat_settings::keep_payload(
                &remote,
                target,
                *keep,
                Timestamp::now().as_millis(),
            ))
        }
        _ => None,
    })
}

impl Pending {
    /// Only the exact query response can produce a prepared send.
    pub fn matches(&self, node: &Node) -> bool {
        node.description == "iq"
            && node.get_attr("id").and_then(|v| v.as_text()) == Some(self.query_id.as_str())
    }
    fn suspend_identity(
        &self,
        store: &ProtocolStore,
        peer: &crate::model::jid::Jid,
        previous: (Option<&[u8]>, Option<&[u8]>),
        node: &Node,
        reason: &str,
    ) -> Result<()> {
        use sha2::{Digest, Sha256};
        let (session, identity) = previous;
        let peer_key = format!("comparison-direct:{peer}");
        let reply = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(node)?;
        store.write(|tx| {
            let current = outbox().get_mut(tx, self.local_id)?
                .ok_or_else(|| Error::invalid("outbox", "missing intent"))?;
            let held_session: Option<Vec<u8>> = tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
            let held_identity: Option<Vec<u8>> = tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
            if current.progress != AttemptProgress::Queued || held_session.as_deref() != session || held_identity.as_deref() != identity {
                return Err(Error::invalid("outbox", "peer state changed during identity check"));
            }
            let reservation: serde_json::Value = tx.get_json(Table::Checkpoints, query_key(self.local_id).as_bytes(), 1)?
                .ok_or_else(|| Error::invalid("outbox", "missing key-query reservation"))?;
            if reservation["query_id"].as_str() != Some(self.query_id.as_str()) || !(reservation["status"] == "outcome_unknown" || identity_query_resumable(&reservation, crate::registration::request::IdentityChangePolicy::ApkAuthenticated)) {
                return Err(Error::invalid("outbox", "key-query reservation changed"));
            }
            // Keep the exact bundle reply encrypted at rest for later policy
            // resolution. Its identity/signature is not trusted by saving it;
            // no replacement, ratchet or send is reserved.
            tx.put_json(Table::Checkpoints, query_key(self.local_id).as_bytes(), 1,
                &serde_json::json!({"status":"identity_changed", "query_id":self.query_id,
                    "local_id":self.local_id, "peer_ticket":hex::encode(Sha256::digest(peer_key.as_bytes())),
                    "reply":reply, "reason":reason,
                    "observed_at":reservation.get("observed_at").cloned().unwrap_or_else(|| serde_json::json!(Timestamp::now())),
                    "pinned_fingerprint":identity.map(|pin| hex::encode(Sha256::digest(pin)))}))
        })
    }
    /// Prepare E2EE and atomically reserve ratchet + intent before any write.
    pub async fn prepare(
        self,
        store: &ProtocolStore,
        account: &AccountRecord,
        node: &Node,
        epoch: ConnectionEpoch,
    ) -> Result<Option<Node>> {
        self.prepare_with_policy(
            store,
            account,
            node,
            epoch,
            crate::registration::request::IdentityChangePolicy::Pinned,
        )
        .await
    }

    /// Explicit normal-outbox policy; only authenticated directory replies are
    /// passed here by the connection actor. The default wrapper stays pinned.
    pub async fn prepare_with_policy(
        self,
        store: &ProtocolStore,
        account: &AccountRecord,
        node: &Node,
        epoch: ConnectionEpoch,
        policy: crate::registration::request::IdentityChangePolicy,
    ) -> Result<Option<Node>> {
        if !self.matches(node) {
            return Err(Error::invalid("outbox", "mismatched key response"));
        }
        let entry = store
            .read(|tx| outbox().get(tx, self.local_id))?
            .ok_or_else(|| Error::invalid("outbox", "missing intent"))?;
        if entry.progress != AttemptProgress::Queued {
            return Ok(None);
        }
        if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
            store.write(|tx| {
                let mut current = outbox()
                    .get_mut(tx, self.local_id)?
                    .ok_or_else(|| Error::invalid("outbox", "missing intent"))?;
                if current.progress == AttemptProgress::Queued {
                    outbox().set_progress(tx, &mut current, AttemptProgress::Failed)?;
                }
                tx.put_json(
                    Table::Checkpoints,
                    query_key(self.local_id).as_bytes(),
                    1,
                    &serde_json::json!({"status":"refused","query_id":self.query_id}),
                )
            })?;
            return Ok(None);
        }
        let peer = entry.intent.chat.jid();
        let bundle = crate::signal::direct::prekey_bundle(node, peer)?;
        let peer_key = format!("comparison-direct:{peer}");
        let (session, identity, reservation): (
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            serde_json::Value,
        ) = store.read(|tx| {
            Ok((
                tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?,
                tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
                tx.get_json(Table::Checkpoints, query_key(self.local_id).as_bytes(), 1)?
                    .ok_or_else(|| Error::invalid("outbox", "missing key-query reservation"))?,
            ))
        })?;
        if reservation["status"] == "identity_changed" {
            use sha2::{Digest, Sha256};
            let current = identity
                .as_deref()
                .map(|pin| hex::encode(Sha256::digest(pin)));
            // Check against the same snapshot used for OCC: a newer incoming
            // transition cannot be rolled back by an older directory reply.
            if current.as_deref() != reservation["pinned_fingerprint"].as_str()
                && (identity.is_none()
                    || crate::signal::direct::validate_bundle_identity(
                        Some(&bundle),
                        identity.as_deref(),
                        peer,
                    )
                    .is_err())
            {
                return Err(Error::invalid(
                    "outbox",
                    "peer pin changed since saved directory reply; intent preserved, no send",
                ));
            }
        }
        let replacement = match crate::signal::direct::validate_bundle_identity(
            Some(&bundle),
            identity.as_deref(),
            peer,
        ) {
            Ok(()) => false,
            Err(Error::IdentityChanged(_)) => {
                let effective = super::inbox::remote_identity_policy(store, account, peer, policy)?;
                if effective == crate::registration::request::IdentityChangePolicy::Pinned {
                    self.suspend_identity(
                        store,
                        peer,
                        (session.as_deref(), identity.as_deref()),
                        node,
                        if policy
                            == crate::registration::request::IdentityChangePolicy::ApkAuthenticated
                        {
                            "own_identity_pinned"
                        } else {
                            "identity_changed"
                        },
                    )?;
                    return Ok(None);
                }
                if crate::signal::direct::validate_bundle_signature(&bundle).is_err() {
                    self.suspend_identity(
                        store,
                        peer,
                        (session.as_deref(), identity.as_deref()),
                        node,
                        "invalid_signed_bundle",
                    )?;
                    return Ok(None);
                }
                true
            }
            Err(error) => return Err(error),
        };
        let session = session.map(Zeroizing::new);
        // APK 2.26.37.71 0jc.A0L invalidates the old ratchet for a changed
        // identity, then CQc.A00 verifies the signed bundle and derives a new
        // session. Preserve the originals for OCC/history, stage new keys only.
        let encryption_session = if replacement {
            None
        } else {
            session.as_deref().map(|v| v.as_slice())
        };
        let encryption_identity = if replacement {
            None
        } else {
            identity.as_deref()
        };

        let media = match &entry.intent.content {
            MessageContent::Media {
                reference,
                caption: None,
                file_name: None,
            } => Some(super::outgoing_media::resolve(
                store,
                &entry.client_request_id,
                peer,
                reference,
            )?),
            MessageContent::Text { .. }
            | MessageContent::Reaction { .. }
            | MessageContent::Poll { .. }
            | MessageContent::PollVote { .. }
            | MessageContent::DisappearingTimer { .. }
            | MessageContent::ChatPrivacy { .. }
            | MessageContent::KeepInChat { .. } => None,
            _ => return Err(Error::invalid("outbox", "unsupported content")),
        };
        // Poll creation keeps its fresh message secret for decrypting votes.
        let mut poll_secret: Option<(Vec<u8>, Vec<String>, String)> = None;
        let poll_payload = match &entry.intent.content {
            MessageContent::Poll {
                name,
                options,
                selectable_count,
            } => {
                let mut secret = [0u8; 32];
                getrandom::fill(&mut secret)
                    .map_err(|_| Error::invalid("poll", "message secret entropy unavailable"))?;
                poll_secret = Some((secret.to_vec(), options.clone(), name.clone()));
                Some((
                    super::polls::creation_payload(name, options, *selectable_count, secret)?,
                    "creation",
                ))
            }
            MessageContent::PollVote { target, selected } => Some((
                vote_payload_for(store, account, peer, target, selected)?,
                "vote",
            )),
            _ => None,
        };
        let reply = match &entry.intent.content {
            MessageContent::Text {
                body,
                quoted: Some(target),
                // Outgoing styling is only meaningful on a status post, which
                // this path does not send.
                style: _,
            } => Some(reply_payload_for(store, account, peer, target, body)?),
            _ => None,
        };
        let reaction = match &entry.intent.content {
            MessageContent::Reaction { target, emoji } => {
                // Contacts who address us by LID keep their chat keyed by LID.
                let remote = super::inbox::lid_for(store, peer)?.unwrap_or_else(|| peer.clone());
                Some(super::reactions::payload(
                    peer,
                    &remote,
                    target,
                    emoji.as_deref(),
                    Timestamp::now().as_millis(),
                )?)
            }
            _ => None,
        };
        let chat_setting = chat_setting_payload_for(store, peer, &entry.intent.content)?;
        let prepared = if let Some((payload, _)) = &poll_payload {
            crate::signal::direct::prepare_payload(
                account,
                peer,
                Some(&bundle),
                encryption_session,
                encryption_identity,
                payload,
            )
            .await?
        } else if let Some(payload) = reply
            .as_deref()
            .or(reaction.as_deref())
            .or(chat_setting.as_deref())
        {
            crate::signal::direct::prepare_payload(
                account,
                peer,
                Some(&bundle),
                encryption_session,
                encryption_identity,
                payload,
            )
            .await?
        } else if let Some((payload, _)) = &media {
            crate::signal::direct::prepare_payload(
                account,
                peer,
                Some(&bundle),
                encryption_session,
                encryption_identity,
                payload,
            )
            .await?
        } else {
            let MessageContent::Text { body, .. } = &entry.intent.content else {
                unreachable!()
            };
            crate::signal::direct::prepare(
                account,
                peer,
                &bundle,
                encryption_session,
                encryption_identity,
                body,
            )
            .await?
        };
        let wire_id = super::messaging::new_message_id()?;
        let stanza = if let Some((_, kind)) = &media {
            super::messaging::direct_media(
                peer,
                &wire_id,
                prepared.kind,
                prepared.ciphertext,
                kind,
            )?
        } else if let Some((_, polltype)) = &poll_payload {
            super::messaging::poll_envelope(
                peer,
                &wire_id,
                prepared.kind,
                prepared.ciphertext,
                polltype,
            )?
        } else if reaction.is_some() {
            super::messaging::direct_typed(
                peer,
                &wire_id,
                prepared.kind,
                prepared.ciphertext,
                "reaction",
            )?
        } else {
            super::messaging::direct_message(peer, &wire_id, prepared.kind, prepared.ciphertext)?
        };
        let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&stanza)?;
        // Retain the authenticated directory reply as private evidence even
        // after successful reservation; it never authorizes replay of a send.
        let directory_reply =
            Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(node)?;
        store.write(|tx| {
            let mut current = outbox()
                .get_mut(tx, self.local_id)?
                .ok_or_else(|| Error::invalid("outbox", "missing intent"))?;
            let old_session: Option<Vec<u8>> =
                tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
            let old_identity: Option<Vec<u8>> =
                tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
            if current != entry
                || old_session.as_deref() != session.as_deref().map(|v| v.as_slice())
                || old_identity != identity
            {
                return Err(Error::invalid(
                    "outbox",
                    "intent or peer session changed before reservation",
                ));
            }
            let query: serde_json::Value = tx
                .get_json(Table::Checkpoints, query_key(self.local_id).as_bytes(), 1)?
                .ok_or_else(|| Error::invalid("outbox", "missing key-query reservation"))?;
            if query["query_id"].as_str() != Some(self.query_id.as_str())
                || !(query["status"] == "outcome_unknown"
                    || identity_query_resumable(&query, policy))
            {
                return Err(Error::invalid(
                    "outbox",
                    "key-query reservation changed before send",
                ));
            }
            if replacement {
                super::identity_conflicts::archive_bundle_replacement(
                    tx,
                    peer,
                    identity.as_deref(),
                    session.as_deref().map(|v| v.as_slice()),
                    &prepared.identity,
                )?;
            }
            super::send_versions::capture_in(tx, self.local_id, std::slice::from_ref(peer))?;
            if let Some((secret, options, name)) = &poll_secret {
                let (own_pn, own_lid) = login_identity(tx, account)?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("poll-secret:{wire_id}").as_bytes(),
                    1,
                    &serde_json::json!({
                        "secret": secret,
                        "options": options,
                        "name": name,
                        "chat": peer.to_non_ad().to_string(),
                        "creator_pn": own_pn.map(|j| j.to_string()),
                        "creator_lid": own_lid.map(|j| j.to_string()),
                    }),
                )?;
            }
            current.wire_id = Some(MessageId::new(wire_id)?);
            current.attempts.push(AttemptRecord {
                index: 0,
                prepared_at: Timestamp::now(),
                epoch: Some(epoch),
                payload: Some(wire.into()),
                re_encryption: false,
            });
            tx.put_json(
                Table::SignalSessions,
                peer_key.as_bytes(),
                1,
                &*prepared.session,
            )?;
            tx.put_json(
                Table::PeerIdentities,
                peer_key.as_bytes(),
                1,
                &prepared.identity,
            )?;
            outbox().set_progress(tx, &mut current, AttemptProgress::InFlight)?;
            // Reservation precedes socket write: even a crash just before write
            // remains uncertain, never falsely retried or called delivered.
            outbox().record_fact(
                tx,
                &mut current,
                DeliveryFact {
                    kind: DeliveryFactKind::DeliveryUnknown,
                    device: None,
                    observed_at: Timestamp::now(),
                    code: None,
                },
            )?;
            tx.put_json(
                Table::Checkpoints,
                query_key(self.local_id).as_bytes(),
                1,
                &serde_json::json!({"status":"answered","query_id":self.query_id,
                    "reply":directory_reply,
                    "observed_at":query.get("observed_at").cloned().unwrap_or_else(|| serde_json::json!(Timestamp::now()))}),
            )
        })?;
        Ok(Some(stanza))
    }
}

/// Correlate message ACKs to an existing in-flight intent by ID and recipient.
pub fn acknowledge(store: &ProtocolStore, node: &Node) -> Result<bool> {
    if node.description != "ack"
        || node.get_attr("class").and_then(|v| v.as_text()) != Some("message")
    {
        return Ok(false);
    }
    let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(false);
    };
    let Some(from) = super::receipts::jid(node, "from") else {
        return Ok(false);
    };
    // APK 2.26.37.71: 1Qy.A02 converts participant through 0DK.A00 only
    // on success; 1Dz.A0H stores it in event 19, and 17J.BFC (023e..024c)
    // ignores a nonnull DeviceJid. Plain PN/LID users convert to primary
    // devices too. Other native user subtypes remain outside this adapter.
    if node.get_attr("error").is_none()
        && super::receipts::jid(node, "participant").is_some_and(|jid| {
            matches!(jid.server, Server::User | Server::Lid) && !jid.user.is_empty()
        })
    {
        return Ok(false);
    }
    let rows = store.read(|tx| tx.scan(Table::Outbox, 1, 10000))?;
    if rows.len() == 10000 {
        return Err(Error::invalid("outbox", "pagination required"));
    }
    store.write(|tx| {
        for (key, _) in rows {
            let Some(mut entry): Option<OutboxEntry> = tx.get_json(Table::Outbox, &key, 1)? else {
                continue;
            };
            if entry.progress != AttemptProgress::InFlight
                || entry.intent.chat.jid() != &from
                || entry.wire_id.as_ref().map(|v| v.as_str()) != Some(id)
            {
                continue;
            }
            let kind = if node.get_attr("error").is_some() {
                DeliveryFactKind::Failed
            } else {
                DeliveryFactKind::ServerAccepted
            };
            if entry.facts.iter().any(|v| {
                matches!(
                    v.kind,
                    DeliveryFactKind::ServerAccepted | DeliveryFactKind::Failed
                )
            }) {
                return Ok(false);
            }
            if kind == DeliveryFactKind::Failed {
                outbox().set_progress(tx, &mut entry, AttemptProgress::Failed)?;
            }
            let code = node
                .get_attr("error")
                .and_then(|v| v.as_text())
                .and_then(|v| v.parse::<u32>().ok());
            outbox().record_fact(
                tx,
                &mut entry,
                DeliveryFact {
                    kind,
                    device: None,
                    observed_at: Timestamp::now(),
                    code,
                },
            )?;
            return Ok(true);
        }
        Ok(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId, ClientRequestId},
            jid::Jid,
            send::SendRequest,
        },
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn enqueue(store: &ProtocolStore) -> LocalMessageId {
        store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: ClientRequestId::new("test").unwrap(),
                            chat: ChatId::from_jid(
                                &Jid::parse("15555550100@s.whatsapp.net").unwrap(),
                            )
                            .unwrap(),
                            content: MessageContent::Text {
                                body: "offline fixture".into(),
                                quoted: None,
                                style: Default::default(),
                            },
                        },
                    )?
                    .0
                    .local_id)
            })
            .unwrap()
    }
    #[tokio::test]
    async fn signed_outgoing_replacement_is_explicit_atomic_and_never_replayed() {
        use crate::{
            model::ids::EventCursor,
            registration::request::IdentityChangePolicy,
            session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
            store::ConsumerId,
        };
        use wacore_libsignal::protocol::{
            DeviceId, GenericSignedPreKey, IdentityKeyPair, KeyPair, PreKeyBundle,
            SignedPreKeyRecord,
        };
        for scenario in 0..6 {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("outgoing-policy.redb");
            let store = open(&path);
            let id = enqueue(&store);
            let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
            let sender = Jid::parse("15555550199@s.whatsapp.net").unwrap();
            let key = format!("comparison-direct:{peer}");
            let mut rng = rand::rng();
            let local = IdentityKeyPair::generate(&mut rng);
            let old = IdentityKeyPair::generate(&mut rng);
            let new = IdentityKeyPair::generate(&mut rng);
            let signed = KeyPair::generate(&mut rng);
            let account_for = |identity: &IdentityKeyPair, registration| AccountRecord {
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(registration),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public: identity.public_key().public_key_bytes().try_into().unwrap(),
                    private: *identity.private_key().serialize(),
                })),
                ..Default::default()
            };
            let mut account = account_for(&local, 31);
            account.own_jid = Some(if scenario == 3 {
                peer.clone()
            } else {
                sender.clone()
            });
            let old_signature = old
                .private_key()
                .calculate_signature(&signed.public_key.serialize(), &mut rng)
                .unwrap();
            let old_bundle = PreKeyBundle::new(
                42,
                DeviceId::new(0),
                None,
                7.into(),
                signed.public_key,
                old_signature.to_vec(),
                *old.identity_key(),
            )
            .unwrap();
            let previous = crate::signal::direct::prepare(
                &account,
                &peer,
                &old_bundle,
                None,
                None,
                "prior fixture",
            )
            .await
            .unwrap();
            store
                .write(|tx| {
                    tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &previous.identity)?;
                    tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &*previous.session)?;
                    AccountRecord::save(&account, tx)?;
                    if scenario == 4 {
                        tx.put_json(
                            Table::Checkpoints,
                            b"login-identity",
                            1,
                            &serde_json::json!({"pn":peer.to_string()}),
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
            let saved_account = store
                .read(|tx| tx.scan_after(Table::Account, 1, None, 256))
                .unwrap();
            let signature = new
                .private_key()
                .calculate_signature(&signed.public_key.serialize(), &mut rng)
                .unwrap();
            let (pending, _) = begin_selected(&store, Some(id)).unwrap().unwrap();
            let reply = Node::new("iq")
                .attr("type", "result")
                .attr("id", pending.query_id.as_str())
                .children(vec![Node::new("list").children(vec![
                    Node::new("user").attr("jid", peer.clone()).children(vec![
                        Node::new("registration").bytes(vec![0, 0, 0, 42]),
                        Node::new("type").bytes(vec![5]),
                        Node::new("identity").bytes(new.public_key().public_key_bytes().to_vec()),
                        Node::new("skey").children(vec![
                            Node::new("id").bytes(vec![0, 0, 7]),
                            Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                            Node::new("signature").bytes(if scenario == 2 || scenario == 5 {
                                vec![0; 64]
                            } else {
                                signature.to_vec()
                            }),
                        ]),
                    ]),
                ])]);
            assert!(
                resume_identity_selected(
                    &store,
                    &account,
                    id,
                    ConnectionEpoch(1),
                    IdentityChangePolicy::ApkAuthenticated
                )
                .await
                .unwrap()
                .is_none(),
                "unanswered queries never resume"
            );
            let policy = if scenario == 1 || scenario == 5 {
                IdentityChangePolicy::Pinned
            } else {
                IdentityChangePolicy::ApkAuthenticated
            };
            let stanza = pending
                .prepare_with_policy(&store, &account, &reply, ConnectionEpoch(1), policy)
                .await
                .unwrap();
            let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
            assert_eq!(entry.local_id, id);
            assert_eq!(
                store
                    .read(|tx| tx.scan_after(Table::Account, 1, None, 256))
                    .unwrap(),
                saved_account
            );
            let events = store
                .read_events(&ConsumerId::new("fixture").unwrap(), EventCursor::START, 32)
                .unwrap();
            let changes = events
                .events
                .iter()
                .filter(|event| matches!(event.event, crate::sync::SyncEvent::IdentityChanged(_)))
                .count();
            let history = store
                .read(|tx| {
                    Ok(tx
                        .scan_after(Table::Checkpoints, 1, None, 256)?
                        .into_iter()
                        .filter(|(k, _)| k.starts_with(b"identity-history:"))
                        .collect::<Vec<_>>())
                })
                .unwrap();
            if scenario == 0 {
                let stanza = stanza.unwrap();
                let enc = stanza.child("enc").unwrap();
                assert_eq!(
                    enc.get_attr("type").and_then(|v| v.as_text()),
                    Some("pkmsg")
                );
                let crate::binary::NodeContent::Bytes(ciphertext) = &enc.content else {
                    panic!("ciphertext missing")
                };
                let signed_record = SignedPreKeyRecord::new(
                    7.into(),
                    wacore_libsignal::protocol::Timestamp::from_epoch_millis(0),
                    &signed,
                    &signature,
                );
                let decoded = crate::signal::direct::receive_prekey(
                    &account_for(&new, 42),
                    &sender,
                    (None, None),
                    &signed_record.serialize().unwrap(),
                    None,
                    ciphertext,
                )
                .await
                .unwrap();
                assert_eq!(
                    crate::signal::direct::payload_summary(&decoded.received.plaintext).unwrap(),
                    "protobuf fields=[1]"
                );
                assert_eq!(&*decoded.received.plaintext, b"\x0a\x0foffline fixture");
                assert_eq!(entry.progress, AttemptProgress::InFlight);
                assert_eq!(entry.attempts.len(), 1);
                assert!(entry.wire_id.is_some());
                assert!(
                    entry
                        .facts
                        .iter()
                        .any(|fact| fact.kind == DeliveryFactKind::DeliveryUnknown)
                );
                let versions =
                    super::super::send_versions::inspect(&store, entry.local_id).unwrap();
                assert!(versions.captured);
                assert_eq!(versions.users.len(), 1);
                assert_eq!(versions.users[0].at_send, Some(1));
                assert_eq!(versions.users[0].current, Some(1));
                assert_eq!(changes, 1);
                assert_eq!(history.len(), 1);
                let row: serde_json::Value = serde_json::from_slice(&history[0].1).unwrap();
                assert_eq!(row["authentication"], "signed_directory_bundle");
                assert_eq!(row["verification"], "unverified");
                assert_eq!(
                    serde_json::from_value::<Vec<u8>>(row["previous_identity"].clone()).unwrap(),
                    previous.identity
                );
                assert_eq!(
                    serde_json::from_value::<Vec<u8>>(row["previous_session"].clone()).unwrap(),
                    *previous.session
                );
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1))
                        .unwrap(),
                    Some(new.identity_key().serialize().to_vec())
                );
                assert!(identity_suspended(&store).unwrap().is_empty());
                assert!(identity_suspension(&store, id).unwrap().is_none());
                // Drop the unsent stanza to model a crash after durable reserve,
                // before socket write. It stays unknown rather than replaying.
            } else {
                assert!(stanza.is_none());
                assert_eq!(entry.progress, AttemptProgress::Queued);
                assert!(entry.attempts.is_empty());
                assert_eq!(changes, 0);
                assert!(history.is_empty());
                assert!(
                    !super::super::send_versions::inspect(&store, entry.local_id)
                        .unwrap()
                        .captured
                );
                let suspension = identity_suspended(&store).unwrap();
                assert_eq!(suspension.len(), 1);
                assert_eq!(
                    identity_suspension(&store, id).unwrap(),
                    Some(suspension[0].clone())
                );
                assert_eq!(
                    suspension[0].reason,
                    if scenario == 2 {
                        "invalid_signed_bundle"
                    } else if scenario == 3 || scenario == 4 {
                        "own_identity_pinned"
                    } else {
                        "identity_changed"
                    }
                );
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1))
                        .unwrap(),
                    Some(previous.identity.clone())
                );
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1))
                        .unwrap(),
                    Some((*previous.session).clone())
                );
            }
            if scenario == 5 {
                // Pinned policy records the mismatch before signature admission.
                // Enabling the policy must classify that original bad reply,
                // preserving first observation and never attempting a send.
                let before: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, query_key(id).as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert!(
                    resume_identity_selected(
                        &store,
                        &account,
                        id,
                        ConnectionEpoch(2),
                        IdentityChangePolicy::ApkAuthenticated
                    )
                    .await
                    .unwrap()
                    .is_none()
                );
                assert_eq!(
                    identity_suspension(&store, id).unwrap().unwrap().reason,
                    "invalid_signed_bundle"
                );
                let after: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, query_key(id).as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(before["observed_at"], after["observed_at"]);
                assert_eq!(before["reply"], after["reply"]);
            }
            if scenario == 1 {
                assert!(
                    resume_identity_selected(
                        &store,
                        &account,
                        id,
                        ConnectionEpoch(2),
                        IdentityChangePolicy::Pinned
                    )
                    .await
                    .unwrap()
                    .is_none()
                );
                let newer = IdentityKeyPair::generate(&mut rng);
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::PeerIdentities,
                            key.as_bytes(),
                            1,
                            &newer.identity_key().serialize().to_vec(),
                        )
                    })
                    .unwrap();
                let error = resume_identity_selected(
                    &store,
                    &account,
                    id,
                    ConnectionEpoch(2),
                    IdentityChangePolicy::ApkAuthenticated,
                )
                .await
                .err()
                .unwrap();
                assert!(
                    error
                        .to_string()
                        .contains("peer pin changed since saved directory reply")
                );
                assert_eq!(
                    store
                        .read(|tx| outbox().get(tx, id))
                        .unwrap()
                        .unwrap()
                        .progress,
                    AttemptProgress::Queued
                );
                // Restore fixture setup only; a real newer peer pin is preserved.
                store
                    .write(|tx| {
                        tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &previous.identity)
                    })
                    .unwrap();
                let resumed = resume_identity_selected(
                    &store,
                    &account,
                    id,
                    ConnectionEpoch(2),
                    IdentityChangePolicy::ApkAuthenticated,
                )
                .await
                .unwrap()
                .unwrap();
                assert_eq!(
                    resumed
                        .child("enc")
                        .unwrap()
                        .get_attr("type")
                        .and_then(|v| v.as_text()),
                    Some("pkmsg")
                );
                assert!(identity_suspended(&store).unwrap().is_empty());
                let row: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, query_key(id).as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    row["query_id"].as_str(),
                    reply.get_attr("id").and_then(|v| v.as_text())
                );
                assert_eq!(row["status"], "answered");
                assert_eq!(
                    serde_json::from_value::<Vec<u8>>(row["reply"].clone()).unwrap(),
                    Encoder::new(TokenDictionary::comparison_2_26_37_71())
                        .encode(&reply)
                        .unwrap()
                );
                assert_eq!(
                    store
                        .read(|tx| outbox().get(tx, id))
                        .unwrap()
                        .unwrap()
                        .progress,
                    AttemptProgress::InFlight
                );
            } else {
                assert!(
                    resume_identity_selected(
                        &store,
                        &account,
                        id,
                        ConnectionEpoch(2),
                        IdentityChangePolicy::ApkAuthenticated
                    )
                    .await
                    .unwrap()
                    .is_none(),
                    "invalid, own-device and already in-flight rows cannot resume"
                );
            }
            let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
            drop(store);
            let store = open(&path);
            assert!(begin_selected(&store, Some(id)).unwrap().is_none());
            assert!(eligible(&store, 10).unwrap().is_empty());
            assert_eq!(
                store.read(|tx| outbox().get(tx, id)).unwrap().unwrap(),
                entry
            );
        }
    }

    #[tokio::test]
    async fn changed_bundle_suspends_without_advancing_or_replaying_after_restart() {
        use wacore_libsignal::protocol::{IdentityKeyPair, KeyPair};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("changed.redb");
        let store = open(&path);
        let id = enqueue(&store);
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let key = format!("comparison-direct:{peer}");
        let mut rng = rand::rng();
        let old = IdentityKeyPair::generate(&mut rng);
        let new = IdentityKeyPair::generate(&mut rng);
        let signed = KeyPair::generate(&mut rng);
        let signature = new
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let pin = old.identity_key().serialize().to_vec();
        // A changed bundle must be stopped before deserializing/using even a
        // stale session. Valid ratchet reuse is covered in signal::direct.
        let session = vec![1, 2, 3];
        store
            .write(|tx| {
                tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &pin)?;
                tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &session)
            })
            .unwrap();
        let (pending, _) = begin_selected(&store, Some(id)).unwrap().unwrap();
        let response = Node::new("iq")
            .attr("type", "result")
            .attr("id", pending.query_id.as_str())
            .children(vec![Node::new("list").children(vec![
                Node::new("user").attr("jid", peer).children(vec![
                    Node::new("registration").bytes(vec![0, 0, 0, 9]),
                    Node::new("type").bytes(vec![5]),
                    Node::new("identity").bytes(new.public_key().public_key_bytes().to_vec()),
                    Node::new("skey").children(vec![
                        Node::new("id").bytes(vec![0, 0, 7]),
                        Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                        Node::new("signature").bytes(signature.to_vec()),
                    ]),
                ]),
            ])]);
        assert!(
            pending
                .prepare(
                    &store,
                    &AccountRecord::default(),
                    &response,
                    ConnectionEpoch(1)
                )
                .await
                .unwrap()
                .is_none()
        );
        let entry = store.read(|tx| outbox().get(tx, id)).unwrap().unwrap();
        assert_eq!(entry.progress, AttemptProgress::Queued);
        assert!(entry.attempts.is_empty());
        assert!(entry.wire_id.is_none());
        store
            .read(|tx| {
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::PeerIdentities, key.as_bytes(), 1)?,
                    Some(pin)
                );
                assert_eq!(
                    tx.get_json::<Vec<u8>>(Table::SignalSessions, key.as_bytes(), 1)?,
                    Some(session)
                );
                let row: serde_json::Value = tx
                    .get_json(Table::Checkpoints, query_key(id).as_bytes(), 1)?
                    .unwrap();
                assert_eq!(row["status"], "identity_changed");
                let reply: Vec<u8> = serde_json::from_value(row["reply"].clone()).unwrap();
                assert_eq!(
                    reply,
                    Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&response)?
                );
                Ok(())
            })
            .unwrap();
        let rows = identity_suspended(&store).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].local_id, id);
        assert_eq!(rows[0].peer_ticket.len(), 64);
        drop(store);
        let store = open(&path);
        assert_eq!(identity_suspended(&store).unwrap(), rows);
        assert!(begin_selected(&store, Some(id)).unwrap().is_none());
        assert!(eligible(&store, 10).unwrap().is_empty());
        store.write(|tx| outbox().cancel(tx, id)).unwrap();
        assert!(identity_suspended(&store).unwrap().is_empty());
    }
    #[test]
    fn eligible_spans_namespaces_and_excludes_reserved_or_cancelled_intents() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("eligible.redb"));
        let first = enqueue(&store);
        let request = |id: &str| SendRequest {
            client_request_id: ClientRequestId::new(id).unwrap(),
            chat: ChatId::from_jid(&Jid::parse("15555550100@s.whatsapp.net").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "offline fixture".into(),
                quoted: None,
                style: Default::default(),
            },
        };
        let other_namespace = Outbox::new("default", Limits::default());
        let second = store
            .write(|tx| {
                Ok(other_namespace
                    .enqueue(tx, &request("public-client"))?
                    .0
                    .local_id)
            })
            .unwrap();
        let cancelled = store
            .write(|tx| {
                let id = other_namespace
                    .enqueue(tx, &request("cancelled"))?
                    .0
                    .local_id;
                other_namespace.cancel(tx, id)?;
                Ok(id)
            })
            .unwrap();
        let ids =
            |entries: Vec<OutboxEntry>| entries.into_iter().map(|e| e.local_id).collect::<Vec<_>>();
        assert_eq!(ids(eligible(&store, 10).unwrap()), vec![first, second]);
        assert_eq!(ids(eligible(&store, 1).unwrap()), vec![first]);
        assert!(eligible(&store, 0).unwrap().is_empty());
        // Reserving the first key query removes it without touching the other.
        assert!(begin_selected(&store, Some(first)).unwrap().is_some());
        assert_eq!(ids(eligible(&store, 10).unwrap()), vec![second]);
        assert!(!ids(eligible(&store, 10).unwrap()).contains(&cancelled));
        assert_eq!(
            store
                .read(|tx| outbox().get(tx, second))
                .unwrap()
                .unwrap()
                .progress,
            AttemptProgress::Queued,
            "listing is read-only"
        );
    }
    #[test]
    fn reactions_are_eligible_for_their_own_chat_only() {
        use crate::model::message::{Direction, MessageKey, MessageRef};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("reaction-outbox.redb"));
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let other = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let reaction = |chat: &Jid, target_chat: &Jid, request: &str| SendRequest {
            client_request_id: ClientRequestId::new(request).unwrap(),
            chat: ChatId::from_jid(chat).unwrap(),
            content: MessageContent::Reaction {
                target: MessageRef {
                    key: MessageKey {
                        chat: ChatId::from_jid(target_chat).unwrap(),
                        sender: None,
                        direction: Direction::Outgoing,
                        id: MessageId::new("ABCDEF0123456789ABCDEF0123456789").unwrap(),
                    },
                },
                emoji: Some("👍".into()),
            },
        };
        // The outbox refuses a cross-chat target before anything is queued.
        assert!(
            store
                .write(|tx| outbox().enqueue(tx, &reaction(&peer, &other, "bad")))
                .is_err()
        );
        let id = store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(tx, &reaction(&peer, &peer, "good"))?
                    .0
                    .local_id)
            })
            .unwrap();
        let (pending, query) = begin_selected(&store, Some(id)).unwrap().unwrap();
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert!(pending.matches(&Node::new("iq").attr("id", pending.query_id.as_str())));
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, query_key(id).as_bytes(), 1))
                .unwrap()
                .is_some(),
            "key query reserved for the reaction"
        );
        let typed = super::super::messaging::direct_typed(&peer, "ID1", "msg", vec![1], "reaction")
            .unwrap();
        assert_eq!(
            typed.get_attr("type").and_then(|v| v.as_text()),
            Some("reaction")
        );
        let meta = typed
            .child_nodes()
            .iter()
            .find(|c| c.description == "meta")
            .unwrap();
        assert_eq!(
            meta.get_attr("content_type").and_then(|v| v.as_text()),
            Some("add_on")
        );
        let plain =
            super::super::messaging::direct_typed(&peer, "ID1", "msg", vec![1], "text").unwrap();
        assert!(plain.child_nodes().iter().all(|c| c.description != "meta"));
        assert!(
            super::super::messaging::direct_typed(&peer, "ID1", "msg", vec![1], "sticker").is_err()
        );
        // Polls and votes are eligible for their own chat; the envelope names the kind.
        let poll = SendRequest {
            client_request_id: ClientRequestId::new("poll").unwrap(),
            chat: ChatId::from_jid(&peer).unwrap(),
            content: MessageContent::Poll {
                name: "Q".into(),
                options: vec!["a".into(), "b".into()],
                selectable_count: 1,
            },
        };
        let poll_id = store
            .write(|tx| Ok(outbox().enqueue(tx, &poll)?.0.local_id))
            .unwrap();
        assert!(begin_selected(&store, Some(poll_id)).unwrap().is_some());
        let envelope =
            super::super::messaging::poll_envelope(&peer, "ID2", "msg", vec![1], "vote").unwrap();
        assert_eq!(
            envelope.get_attr("type").and_then(|v| v.as_text()),
            Some("poll")
        );
        let meta = envelope
            .child_nodes()
            .iter()
            .find(|c| c.description == "meta")
            .unwrap();
        assert_eq!(
            meta.get_attr("polltype").and_then(|v| v.as_text()),
            Some("vote")
        );
        assert!(
            super::super::messaging::poll_envelope(&peer, "ID2", "msg", vec![1], "quiz").is_err()
        );
        let bad_poll = SendRequest {
            client_request_id: ClientRequestId::new("bad-poll").unwrap(),
            chat: ChatId::from_jid(&peer).unwrap(),
            content: MessageContent::Poll {
                name: "Q".into(),
                options: vec!["a".into()],
                selectable_count: 0,
            },
        };
        assert!(store.write(|tx| outbox().enqueue(tx, &bad_poll)).is_err());
    }
    #[tokio::test]
    async fn oversized_text_fails_before_query_and_uploaded_media_uses_one_reservation() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("media-outbox.redb"));
        let id = enqueue(&store);
        store
            .write(|tx| {
                let mut entry = outbox().get_mut(tx, id)?.unwrap();
                entry.intent.content = MessageContent::Text {
                    body: "x".repeat(4097),
                    quoted: None,
                    style: Default::default(),
                };
                tx.put_json(Table::Outbox, &id.0, 1, &entry)
            })
            .unwrap();
        assert!(begin_selected(&store, Some(id)).unwrap().is_none());
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, query_key(id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .read(|tx| outbox().get(tx, id))
                .unwrap()
                .unwrap()
                .progress,
            AttemptProgress::Failed
        );
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let request_id = ClientRequestId::new("uploaded-media").unwrap();
        store.write(|tx| tx.put_json(Table::Checkpoints, b"message:uploaded-media:upload", 1,
            &serde_json::json!({"status":"uploaded","to":peer.user,"hash":"synthetic","protobuf":[26,0]}))).unwrap();
        let reference =
            super::super::outgoing_media::reference(&store, &request_id, &peer).unwrap();
        let media_id = store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: request_id,
                            chat: ChatId::from_jid(&peer)?,
                            content: MessageContent::Media {
                                reference,
                                caption: None,
                                file_name: None,
                            },
                        },
                    )?
                    .0
                    .local_id)
            })
            .unwrap();
        assert!(begin_selected(&store, Some(media_id)).unwrap().is_some());
        assert!(begin_selected(&store, Some(media_id)).unwrap().is_none());
    }

    #[tokio::test]
    async fn query_reservation_survives_restart_without_repeating_requests() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("outbox.redb");
        let store = open(&path);
        let local_id = enqueue(&store);
        let (pending, query) = begin(&store).unwrap().unwrap();
        assert_eq!(pending.local_id, local_id);
        assert!(pending.matches(
            &Node::new("iq").attr("id", query.get_attr("id").unwrap().as_text().unwrap())
        ));
        assert!(!pending.matches(&Node::new("iq").attr("id", "wrong")));
        assert!(begin(&store).unwrap().is_none());
        drop(store);
        let store = open(&path);
        assert!(begin(&store).unwrap().is_none());
        let entry = store
            .read(|tx| outbox().get(tx, local_id))
            .unwrap()
            .unwrap();
        assert!(entry.attempts.is_empty());
        assert!(entry.wire_id.is_none());
    }
    #[tokio::test]
    async fn selecting_one_ticket_never_reserves_other_queued_work() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("selected.redb"));
        let first = enqueue(&store);
        let second = store
            .write(|tx| {
                Ok(outbox()
                    .enqueue(
                        tx,
                        &SendRequest {
                            client_request_id: ClientRequestId::new("second")?,
                            chat: ChatId::from_jid(&Jid::parse("15555550100@s.whatsapp.net")?)?,
                            content: MessageContent::Text {
                                body: "second fixture".into(),
                                quoted: None,
                                style: Default::default(),
                            },
                        },
                    )?
                    .0
                    .local_id)
            })
            .unwrap();
        assert_eq!(
            begin_selected(&store, Some(second))
                .unwrap()
                .unwrap()
                .0
                .local_id,
            second
        );
        assert!(begin_selected(&store, Some(second)).unwrap().is_none());
        let first_reservation: Option<serde_json::Value> = store
            .read(|tx| tx.get_json(Table::Checkpoints, query_key(first).as_bytes(), 1))
            .unwrap();
        assert!(first_reservation.is_none());
        assert_eq!(
            store
                .read(|tx| outbox().get(tx, first))
                .unwrap()
                .unwrap()
                .progress,
            AttemptProgress::Queued
        );
    }

    #[tokio::test]
    async fn reserved_queries_do_not_starve_later_work_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reserved-history.redb");
        let store = open(&path);
        let initial = enqueue(&store);
        let template = store.read(|tx| outbox().get(tx, initial)).unwrap().unwrap();
        let last = LocalMessageId(299_u128.to_be_bytes());
        store
            .write(|tx| {
                tx.delete(Table::Outbox, &initial.0)?;
                for index in 0..300_u128 {
                    let mut entry = template.clone();
                    entry.local_id = LocalMessageId(index.to_be_bytes());
                    tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)?;
                    if entry.local_id != last {
                        tx.put_json(
                            Table::Checkpoints,
                            query_key(entry.local_id).as_bytes(),
                            1,
                            &serde_json::json!({"status":"outcome_unknown"}),
                        )?;
                    }
                }
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = open(&path);
        let (pending, _) = begin(&store).unwrap().unwrap();
        assert_eq!(pending.local_id, last);
        assert!(begin(&store).unwrap().is_none());
        let previous: serde_json::Value = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    query_key(LocalMessageId(0_u128.to_be_bytes())).as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(previous["status"], "outcome_unknown");
    }
    #[tokio::test]
    async fn acknowledgements_do_not_replay_or_cross_recipient_boundaries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("outbox.redb");
        let store = open(&path);
        let local_id = enqueue(&store);
        store
            .write(|tx| {
                let mut entry = outbox().get_mut(tx, local_id)?.unwrap();
                entry.wire_id = Some(MessageId::new("fixture")?);
                outbox().set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                outbox().record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::DeliveryUnknown,
                        device: None,
                        observed_at: Timestamp::now(),
                        code: None,
                    },
                )
            })
            .unwrap();
        // A process can die after the durable send reservation and before the
        // socket write or its ACK. Reopening must not make that intent eligible.
        drop(store);
        let store = open(&path);
        assert!(begin_selected(&store, Some(local_id)).unwrap().is_none());
        assert!(begin(&store).unwrap().is_none());
        assert!(
            store
                .read(|tx| outbox().status(tx, local_id))
                .unwrap()
                .unwrap()
                .is_uncertain()
        );
        let ack = Node::new("ack")
            .attr("class", "message")
            .attr("id", "fixture");
        assert!(
            !acknowledge(
                &store,
                &ack.clone().attr("from", "15555550101@s.whatsapp.net")
            )
            .unwrap()
        );
        let ack = ack.attr("from", "15555550100@s.whatsapp.net");
        // APK success participant conversion applies to primary users and
        // companion devices, for both literal-text and decoded JID attributes.
        for participant in [
            "15555550102@s.whatsapp.net",
            "15555550102:3@s.whatsapp.net",
            "123456789@lid",
            "123456789:3@lid",
        ] {
            for candidate in [
                ack.clone().attr("participant", participant),
                ack.clone()
                    .attr("participant", Jid::parse(participant).unwrap()),
            ] {
                assert!(!acknowledge(&store, &candidate).unwrap());
                let status = store
                    .read(|tx| outbox().status(tx, local_id))
                    .unwrap()
                    .unwrap();
                assert!(status.is_uncertain());
                assert!(!status.server_accepted());
                assert_eq!(status.facts.len(), 1);
                assert!(begin_selected(&store, Some(local_id)).unwrap().is_none());
            }
        }
        drop(store);
        let store = open(&path);
        assert!(begin(&store).unwrap().is_none());
        assert!(
            store
                .read(|tx| outbox().status(tx, local_id))
                .unwrap()
                .unwrap()
                .is_uncertain()
        );
        assert!(acknowledge(&store, &ack).unwrap());
        assert!(!acknowledge(&store, &ack).unwrap());
        drop(store);
        let store = open(&path);
        assert!(
            store
                .read(|tx| outbox().pending(tx, 10))
                .unwrap()
                .is_empty()
        );
        let status = store
            .read(|tx| outbox().status(tx, local_id))
            .unwrap()
            .unwrap();
        assert!(status.server_accepted());
        assert!(!status.is_uncertain());
        assert_eq!(status.facts.len(), 2);
    }
    #[test]
    fn participant_success_guard_does_not_discard_error_acks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("error-ack.redb");
        let store = open(&path);
        let local_id = enqueue(&store);
        store
            .write(|tx| {
                let mut entry = outbox().get_mut(tx, local_id)?.unwrap();
                entry.wire_id = Some(MessageId::new("fixture")?);
                outbox().set_progress(tx, &mut entry, AttemptProgress::InFlight)
            })
            .unwrap();
        let ack = Node::new("ack")
            .attr("class", "message")
            .attr("id", "fixture")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("participant", Jid::parse("123456789:3@lid").unwrap())
            .attr("error", "500");
        assert!(acknowledge(&store, &ack).unwrap());
        assert!(!acknowledge(&store, &ack).unwrap());
        drop(store);
        let store = open(&path);
        let status = store
            .read(|tx| outbox().status(tx, local_id))
            .unwrap()
            .unwrap();
        assert!(!status.server_accepted());
        assert_eq!(status.facts.len(), 1);
        assert_eq!(status.facts[0].kind, DeliveryFactKind::Failed);
        assert_eq!(status.facts[0].code, Some(500));
        assert!(begin_selected(&store, Some(local_id)).unwrap().is_none());
    }
}

#[cfg(test)]
mod quote_scope_tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId, ClientRequestId},
            jid::Jid,
            message::{Direction, MessageKey, MessageRef},
            send::SendRequest,
        },
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    #[test]
    fn outgoing_quote_body_requires_the_saved_intents_original_chat() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("quote.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
            create: true,
        })
        .unwrap();
        let original = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let foreign = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let request = SendRequest {
            client_request_id: ClientRequestId::new("fixture").unwrap(),
            chat: ChatId::from_jid(&original).unwrap(),
            content: MessageContent::Text {
                body: "private original body".into(),
                quoted: None,
                style: Default::default(),
            },
        };
        let wire = MessageId::new("ORIGINAL-WIRE").unwrap();
        let ticket = store.write(|tx| outbox().enqueue(tx, &request)).unwrap().0;
        store
            .write(|tx| {
                let mut entry = outbox().get_mut(tx, ticket.local_id)?.unwrap();
                entry.wire_id = Some(wire.clone());
                tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)
            })
            .unwrap();
        let account = AccountRecord {
            own_jid: Some(Jid::parse("15555550199@s.whatsapp.net").unwrap()),
            ..Default::default()
        };
        let target = |chat: &Jid, id: MessageId| MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(chat).unwrap(),
                sender: None,
                direction: Direction::Outgoing,
                id,
            },
        };
        let before = store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap();
        let error = reply_payload_for(
            &store,
            &account,
            &foreign,
            &target(&foreign, wire.clone()),
            "reply",
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("saved quoted message belongs to another chat")
        );
        assert!(!error.to_string().contains("private original body"));
        assert_eq!(
            store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap(),
            before
        );
        let payload = reply_payload_for(
            &store,
            &account,
            &original,
            &target(&original, wire),
            "reply",
        )
        .unwrap();
        assert!(
            payload
                .windows(b"private original body".len())
                .any(|w| w == b"private original body")
        );
        let unknown = reply_payload_for(
            &store,
            &account,
            &original,
            &target(&original, MessageId::new("UNKNOWN").unwrap()),
            "reply",
        )
        .unwrap();
        assert!(
            !unknown
                .windows(b"private original body".len())
                .any(|w| w == b"private original body")
        );
    }
}

#[cfg(test)]
mod poll_scope_tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId},
            jid::Jid,
            message::{Direction, MessageKey, MessageRef},
        },
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    #[test]
    fn own_vote_uses_saved_chat_and_refuses_missing_foreign_or_invalid_bindings() {
        for case in [
            "same",
            "mapped",
            "reverse",
            "foreign",
            "unattributed",
            "alias_only",
            "missing",
            "invalid",
            "invalid_map",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("vote.redb"),
                account_id: AccountId::new("fixture").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [22; 32],
                ))),
                create: true,
            })
            .unwrap();
            let pn = Jid::parse("15555550101@s.whatsapp.net").unwrap();
            let lid = Jid::parse("12345@lid").unwrap();
            let original = if case == "reverse" {
                lid.clone()
            } else {
                pn.clone()
            };
            let peer = match case {
                "same" | "reverse" => pn.clone(),
                "foreign" => Jid::parse("15555550102@s.whatsapp.net").unwrap(),
                _ => lid.clone(),
            };
            let own_jid = Jid::parse("15555550199@s.whatsapp.net").unwrap();
            let account = AccountRecord {
                own_jid: Some(own_jid.clone()),
                ..Default::default()
            };
            let mut record = serde_json::json!({"chat":original.to_string(),"secret":vec![9u8;32],"options":["private option"]});
            if case == "missing" {
                record.as_object_mut().unwrap().remove("chat");
            }
            if case == "invalid" {
                record["chat"] = serde_json::json!("private invalid chat");
            }
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, b"poll-secret:ORIGINAL", 1, &record)?;
                    if matches!(case, "mapped" | "reverse" | "invalid_map") {
                        let value = if case == "invalid_map" {
                            serde_json::json!({"private":"bad"})
                        } else {
                            serde_json::to_value(&pn).unwrap()
                        };
                        tx.put_json(
                            Table::Checkpoints,
                            b"authenticated-lid:12345@lid",
                            1,
                            &value,
                        )?;
                    }
                    if case == "alias_only" {
                        tx.put_json(
                            Table::Checkpoints,
                            b"lid:15555550101@s.whatsapp.net",
                            1,
                            &lid,
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
            let before = store.read(|tx| tx.scan(Table::Checkpoints, 1, 50)).unwrap();
            let target = MessageRef {
                key: MessageKey {
                    chat: ChatId::from_jid(&peer).unwrap(),
                    sender: None,
                    direction: Direction::Outgoing,
                    id: MessageId::new("ORIGINAL").unwrap(),
                },
            };
            let binding = super::super::polls::own_chat_binding(&store, &target.key.id, &peer);
            if matches!(case, "invalid" | "invalid_map") {
                assert!(binding.is_err());
            } else {
                let expected = if matches!(case, "same" | "mapped" | "reverse") {
                    "matched"
                } else if case == "missing" {
                    "missing_chat_binding"
                } else {
                    "unattributed"
                };
                assert_eq!(binding.unwrap(), expected, "{case}");
            }
            assert_eq!(
                super::super::polls::own_chat_binding(
                    &store,
                    &MessageId::new("UNKNOWN").unwrap(),
                    &peer
                )
                .unwrap(),
                "unknown_poll"
            );
            let result =
                vote_payload_for(&store, &account, &peer, &target, &["private option".into()]);
            if matches!(case, "same" | "mapped" | "reverse") {
                let vote = super::super::polls::decode_vote(&result.unwrap()).unwrap();
                assert!(vote.poll_from_sender);
                let hashes = super::super::polls::decrypt_vote(
                    &[9; 32], "ORIGINAL", &own_jid, &own_jid, &vote,
                )
                .unwrap();
                assert_eq!(
                    hashes,
                    vec![super::super::polls::option_hash("private option").to_vec()]
                );
            } else {
                let error = result.unwrap_err().to_string();
                let reason = match case {
                    "missing" => "saved poll has no chat binding",
                    "invalid" => "invalid saved poll chat binding",
                    "invalid_map" => "invalid authenticated LID mapping",
                    _ => "saved poll belongs to another chat",
                };
                assert!(error.contains(reason), "{case}");
                assert!(!error.contains("private"), "{case}");
            }
            assert_eq!(
                store.read(|tx| tx.scan(Table::Checkpoints, 1, 50)).unwrap(),
                before,
                "{case}"
            );
            assert!(
                store
                    .read(|tx| tx.scan(Table::Outbox, 1, 50))
                    .unwrap()
                    .is_empty()
            );
        }
    }
}
