//! Answer a peer's `receipt type="retry"` for one of our direct sends.
//!
//! **Confirmed static, 2.26.37.71** (docs/protocol/retry-answers.md):
//! `RetryReceiptHandler` (`150.A04`) skips `count > 4` (`0270..027e`) and
//! originals older than 60 days (`0426..043c`, `0x134fd9000` ms); `DNP.run`
//! skips a device that is not an original recipient (`0118..0136`) or already
//! has the message (`01c6..01f4`), then re-injects the send job over the
//! original message: the same plaintext re-encrypted, the same message id.
//! `151.A00` keeps the existing session when the receipt's `<registration>`
//! equals the session's `remoteRegistrationId_`; a mismatch deletes the session
//! and waits for a fresh prekey bundle, and `count > 2` may also wait for one.
//!
//! **Implemented here:** direct chat, count 1 or 2, server-accepted original
//! within 60 days, using a matching session or a reserved fresh-bundle fetch.
//! **Confirmed static, 2.26.43.72:** the received count reaches the send job
//! unchanged and FTy.AK7 -> F3f.A04 -> F3f.A09 writes nonzero `enc count` as
//! decimal (docs/protocol/retry-enc-count.md). Both answer paths preserve it.
//! Not reproduced: the count-2 base-key record/current-key comparison
//! (`163.A04`, `0WJ.A0s/A15` in 43.72), receipts carrying `<keys>` (explicitly refused), and
//! group/broadcast retries. Counts above two remain refused.

use crate::{
    Error, Result,
    binary::{Encoder, Node, NodeContent, TokenDictionary},
    model::{
        ids::{DeviceAddress, MessageId},
        send::DeliveryFactKind,
        time::Timestamp,
    },
    outbox::{AttemptRecord, OutboxEntry},
    session::AccountRecord,
    store::{ProtocolStore, Table, WriteTx},
    transport::ConnectionEpoch,
};

/// `150.A04 0272`: counts above four are skipped.
pub const MAX_COUNT: u32 = 4;
/// `150.A04 0426`: originals older than 60 days are not resent.
pub const MAX_AGE_MS: i64 = 5_184_000_000;

fn original_too_old(prepared_at: Timestamp, now: Timestamp) -> bool {
    // Strict >60-day boundary without overflow on extreme saved timestamps.
    i128::from(now.as_millis()) - i128::from(prepared_at.as_millis()) > i128::from(MAX_AGE_MS)
}

/// The plaintext `Message` of a direct send, kept for a retry answer.
pub(crate) fn source_key(wire_id: &str) -> String {
    format!("retry-source:{wire_id}")
}

/// Retain the plaintext under the store's at-rest encryption at reservation.
pub(crate) fn keep_source_in(tx: &mut WriteTx<'_>, wire_id: &str, payload: &[u8]) -> Result<()> {
    let key = source_key(wire_id);
    if let Some(existing) = tx.get_json::<Vec<u8>>(Table::Checkpoints, key.as_bytes(), 1)? {
        if existing != payload {
            return Err(Error::invalid(
                "retry",
                "original retry source already retained with different bytes",
            ));
        }
        return Ok(());
    }
    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &payload)
}

fn unanswered(store: &ProtocolStore, key: &str, reason: &str) -> Result<Option<Node>> {
    tracing::warn!(reason, "retry receipt not answered");
    store.write(|tx| {
        if tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?.is_none() {
            tx.put_json(
                Table::Checkpoints,
                key.as_bytes(),
                1,
                &serde_json::json!({"status": "unanswered", "reason": reason, "observed_at": Timestamp::now()}),
            )?;
        }
        Ok(())
    })?;
    Ok(None)
}

/// Live callback ownership for one drive invocation. Durable refetch jobs are
/// reservations, not permission for a later connection to consume an old IQ.
/// APK43.72 0Th.A03 fails/clears pending IQ futures on disconnect.
#[derive(Default)]
pub(crate) struct ActiveRefetches(std::collections::HashMap<String, RefetchWriteGuard>);

impl ActiveRefetches {
    pub(crate) fn record_written(&mut self, answer: &ReservedAnswer) -> Result<()> {
        if answer.node.description != "iq" {
            return Ok(());
        }
        let id = answer
            .node
            .get_attr("id")
            .and_then(|v| v.as_text())
            .ok_or_else(|| Error::invalid("retry", "written refetch query lacks id"))?;
        let owner = answer
            .refetch
            .as_ref()
            .filter(|owner| owner.query == id)
            .ok_or_else(|| {
                Error::invalid(
                    "retry",
                    "written refetch lacks matching reservation snapshot",
                )
            })?;
        // Never replace an existing callback with a later same-ID job. The
        // native future-map owner is request-bound, not authority from any row
        // subsequently saved under that ID (retry-answers.md).
        match self.0.entry(id.to_owned()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(owner.clone());
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(Error::invalid(
                "retry",
                "refetch callback id already active",
            )),
        }
    }

    pub(crate) async fn complete(
        &mut self,
        store: &ProtocolStore,
        account: &AccountRecord,
        node: &Node,
        epoch: ConnectionEpoch,
    ) -> Result<Option<Option<Node>>> {
        if node.description != "iq"
            || !matches!(
                node.get_attr("type").and_then(|v| v.as_text()),
                Some("result" | "error")
            )
        {
            return Ok(None);
        }
        let Some(id) = node.get_attr("id").and_then(|v| v.as_text()) else {
            return Ok(None);
        };
        // Consume before processing: a callback failure cannot authorize a
        // second reply, nor can a persisted row resurrect a dead callback.
        let Some(owner) = self.0.remove(id) else {
            return Ok(None);
        };
        complete_refetch_inner(store, account, node, epoch, Some(&owner.job)).await
    }
}

/// A resend/key query whose state has already committed. A confirmation is
/// attached only after actual message/query work commits, never merely parsed input.
pub(crate) struct ReservedAnswer {
    pub node: Node,
    pub confirmation: Option<Node>,
    pub refetch: Option<RefetchWriteGuard>,
}

pub(crate) async fn answer_with_confirmation(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    epoch: ConnectionEpoch,
) -> Result<Option<ReservedAnswer>> {
    let mut refetch = None;
    let Some(answer) = answer_inner(store, account, node, epoch, &mut refetch).await? else {
        return Ok(None);
    };
    // APK43.72 ACK submission precedes the Fe5 callback which adds a send
    // job; that job's A0J starts a key fetch for unmet session/base-key
    // requirements. docs/protocol/retry-ack-dispatch.md
    let confirmation = if matches!(answer.description.as_str(), "message" | "iq") {
        let confirmation = super::messaging::handled_retry_confirmation(node);
        if confirmation.is_none() {
            tracing::warn!(
                field = "retry_context",
                "reserved retry lacks supported ACK context"
            );
        }
        confirmation
    } else {
        None
    };
    Ok(Some(ReservedAnswer {
        node: answer,
        confirmation,
        refetch,
    }))
}

/// Answer `node` if it is a retry receipt for our server-accepted direct send.
/// Returns the resend to write, already reserved; `None` when not answered.
pub async fn answer(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    epoch: ConnectionEpoch,
) -> Result<Option<Node>> {
    answer_inner(store, account, node, epoch, &mut None).await
}

async fn answer_inner(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    epoch: ConnectionEpoch,
    authority: &mut Option<RefetchWriteGuard>,
) -> Result<Option<Node>> {
    let Some(retry) = super::receipts::parse_retry(node) else {
        return Ok(None);
    };
    // APK43.72 completion submits the ACK before Fe5 dispatches retry work.
    // Preflight only: no success ACK is exposed until actual work commits.
    // A known-unwritable/local-unsupported context must not advance a ratchet
    // or reserve a refetch that the driver would send without its confirmation.
    if retry.chat.is_user_like()
        && super::messaging::handled_retry_confirmation(node).is_none_or(|confirmation| {
            Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&confirmation)
                .is_err()
        })
    {
        tracing::warn!(
            field = "retry_context",
            "retry ACK context unsupported; no retry work reserved"
        );
        return Ok(None);
    }
    let Ok(wire_id) = MessageId::new(retry.message_id.clone()) else {
        return Ok(None);
    };
    let outbox = super::outgoing::outbox();
    let Some(entry) = store.read(|tx| outbox.by_wire_id(tx, &wire_id))? else {
        return Ok(None); // Not one of ours (or a comparison send): nothing to say.
    };
    let key = format!(
        "retry-unanswered:{}:{}:{}",
        retry.message_id, retry.device, retry.count
    );
    let peer = entry.intent.chat.jid().clone();
    if retry.chat.is_group() || peer.is_group() {
        return unanswered(store, &key, "group_retry_unsupported");
    }
    if retry.count == 0 || retry.count > MAX_COUNT {
        return unanswered(store, &key, "count_out_of_range");
    }
    if retry.count > 2 {
        return unanswered(store, &key, "new_session_needed_unsupported");
    }
    // Our direct send addressed exactly `peer` (device 0); the receipt may
    // name that device by its LID.
    let lid = super::inbox::lid_for(store, &peer)?;
    // Native retry lookup includes the original remote JID, not only message
    // id/device. Compare using our existing saved PN/LID relation; do not learn
    // or rewrite identities from a receipt (docs/protocol/retry-original-chat.md).
    let retry_chat = retry.chat.to_non_ad();
    if !(retry_chat == peer.to_non_ad() || Some(&retry_chat) == lid.as_ref()) {
        return unanswered(store, &key, "not_the_original_chat");
    }
    let device = retry.device.to_non_ad();
    if retry.device.device != peer.device
        || !(device == peer.to_non_ad() || Some(&device) == lid.as_ref())
    {
        return unanswered(store, &key, "not_an_original_recipient");
    }
    if !entry
        .facts
        .iter()
        .any(|f| f.kind == DeliveryFactKind::ServerAccepted)
        || entry
            .facts
            .iter()
            .any(|f| f.kind == DeliveryFactKind::Failed)
    {
        return unanswered(store, &key, "original_not_server_accepted");
    }
    let addresses = [&peer, &retry.device]
        .into_iter()
        .filter_map(|j| DeviceAddress::from_jid(j).ok())
        .collect::<Vec<_>>();
    if entry.facts.iter().any(|f| {
        matches!(
            f.kind,
            DeliveryFactKind::Delivered | DeliveryFactKind::Read | DeliveryFactKind::Played
        ) && f.device.as_ref().is_none_or(|d| addresses.contains(d))
    }) {
        return unanswered(store, &key, "already_delivered");
    }
    let Some(first) = entry.attempts.first() else {
        return unanswered(store, &key, "original_attempt_missing");
    };
    if original_too_old(first.prepared_at, Timestamp::now()) {
        return unanswered(store, &key, "original_older_than_60_days");
    }
    let Some(original) = first.payload.as_deref() else {
        return unanswered(store, &key, "original_stanza_missing");
    };
    // APK43.72 F10.A04 and163.A02 route receipt-carried keys through a
    // separate session/verification policy. Until ported, do not silently
    // substitute our ordinary saved-session or directory-fetch answer.
    if node.child("keys").is_some() {
        return unanswered(store, &key, "receipt_keys_unsupported");
    }
    let peer_key = format!("comparison-direct:{peer}");
    let answer_key = format!(
        "retry-answer:{}:{}:{}",
        retry.message_id, retry.device, retry.count
    );
    let bytes = |table, key: &str| -> Result<Option<Vec<u8>>> {
        store.read(|tx| tx.get_json(table, key.as_bytes(), 1))
    };
    let source = bytes(Table::Checkpoints, &source_key(wire_id.as_str()))?;
    let session = bytes(Table::SignalSessions, &peer_key)?;
    let identity = bytes(Table::PeerIdentities, &peer_key)?;
    let answered: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, answer_key.as_bytes(), 1))?;
    if answered.is_some() {
        return Ok(None); // One answer per (message, device, count), ever.
    }
    let Some(source) = source else {
        // Sends reserved before this module existed kept only ciphertext.
        return unanswered(store, &key, "plaintext_not_retained");
    };
    let registration = retry
        .registration
        .as_deref()
        .filter(|b| b.len() <= 4)
        .map(|b| b.iter().fold(0u32, |n, byte| (n << 8) | u32::from(*byte)));
    let session = match session.as_deref() {
        Some(session)
            if registration.is_some()
                && registration
                    == Some(crate::signal::direct::session_remote_registration(session)?) =>
        {
            session
        }
        // 151.A00: on a registration change (or no session) the APK drops the
        // session and "will wait to send … until a new prekey has been
        // fetched". Ask the directory for the device's bundle; the reply
        // finishes the answer in `complete_refetch`.
        _ => {
            let query = super::messaging::new_message_id()?;
            let job = Refetch {
                wire_id: retry.message_id.clone(),
                answer_key: answer_key.clone(),
                unanswered_key: key.clone(),
                count: retry.count,
            };
            let reserved = reserve_refetch(
                store,
                &entry,
                account,
                RetryPeerSnapshot {
                    key: &peer_key,
                    identity: identity.as_deref(),
                    session: session.as_deref(),
                },
                &query,
                &job,
            )?;
            if reserved {
                *authority = Some(RefetchWriteGuard {
                    query: query.clone(),
                    job,
                });
                return Ok(Some(super::messaging::key_request(&peer, &query)));
            }
            return Ok(None);
        }
    };
    let prepared = match crate::signal::direct::prepare_payload(
        account,
        &peer,
        None,
        Some(session),
        identity.as_deref(),
        &source,
    )
    .await
    {
        Ok(prepared) => prepared,
        // A peer's receipt must not end receive; the pin is never adopted here.
        Err(Error::IdentityChanged(_)) => return unanswered(store, &key, "identity_changed"),
        Err(error) => return Err(error),
    };
    let resend = replace_enc(original, prepared.kind, prepared.ciphertext, retry.count)?;
    let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&resend)?;
    store.write(|tx| {
        ensure_retry_account(tx, account)?;
        ensure_retry_source(tx, wire_id.as_str(), &source)?;
        let mut current = outbox
            .by_wire_id_in_write(tx, &wire_id)?
            .ok_or_else(|| Error::invalid("retry", "original send vanished"))?;
        ensure_original_unchanged(&current, &entry, Timestamp::now())?;
        ensure_retry_peer_unchanged(tx, &peer_key, identity.as_deref(), Some(session))?;
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, answer_key.as_bytes(), 1)?
            .is_some()
        {
            return Err(Error::invalid("retry", "answer changed before reservation"));
        }
        // Reservation precedes the write: a crash leaves an uncertain answer,
        // never a second one.
        tx.put_json(
            Table::Checkpoints,
            answer_key.as_bytes(),
            1,
            &serde_json::json!({"status": "reserved", "count": retry.count, "reserved_at": Timestamp::now()}),
        )?;
        current.attempts.push(AttemptRecord {
            index: current.attempts.len() as u32,
            prepared_at: Timestamp::now(),
            epoch: Some(epoch),
            payload: Some(wire.into()),
            re_encryption: true,
        });
        tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &*prepared.session)?;
        tx.put_json(Table::Outbox, &current.local_id.0, 1, &current)
    })?;
    Ok(Some(resend))
}

fn refetch_key(query: &str) -> String {
    format!("retry-refetch:{query}")
}

/// A retry answer waiting for the device's fresh prekey bundle.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Refetch {
    wire_id: String,
    answer_key: String,
    unanswered_key: String,
    count: u32,
}

// Freeze at reservation, not by reloading the job after the ACK or queue wait.
// Local ownership boundary; native ACK-before-query evidence is retained in
// docs/protocol/retry-ack-dispatch.md. No new timer or replay authority.
#[derive(Clone)]
pub(crate) struct RefetchWriteGuard {
    query: String,
    job: Refetch,
}
impl RefetchWriteGuard {
    pub(crate) fn check_write_authority(&self, store: &ProtocolStore) -> Result<()> {
        store.write(|tx| ensure_refetch_owned(tx, &self.query, &self.job))
    }
}

// A delayed directory callback owns exactly the job it loaded and a fetching
// answer for that query/count. Recheck both in the transaction that consumes
// them; checking only query permits replacing a newer job or terminal marker.
fn ensure_refetch_owned(tx: &mut WriteTx<'_>, query: &str, job: &Refetch) -> Result<()> {
    let current: Option<Refetch> =
        tx.get_json(Table::Checkpoints, refetch_key(query).as_bytes(), 1)?;
    if current.as_ref() != Some(job) {
        return Err(Error::invalid(
            "retry",
            "pending refetch changed before use",
        ));
    }
    let state: Option<serde_json::Value> =
        tx.get_json(Table::Checkpoints, job.answer_key.as_bytes(), 1)?;
    if !matches!(job.count, 1 | 2)
        || state.as_ref().and_then(|v| v["status"].as_str()) != Some("fetching")
        || state.as_ref().and_then(|v| v["query"].as_str()) != Some(query)
        || state.as_ref().and_then(|v| v["count"].as_u64()) != Some(u64::from(job.count))
    {
        return Err(Error::invalid(
            "retry",
            "refetch answer ownership changed before use",
        ));
    }
    Ok(())
}

struct RetryPeerSnapshot<'a> {
    key: &'a str,
    identity: Option<&'a [u8]>,
    session: Option<&'a [u8]>,
}

// Reserving a key query must use the same eligibility snapshot as the retry
// decision. Another transaction can update delivery or peer state even before
// crypto begins (docs/protocol/retry-answers.md).
fn reserve_refetch(
    store: &ProtocolStore,
    prepared_from: &OutboxEntry,
    account: &AccountRecord,
    peer: RetryPeerSnapshot<'_>,
    query: &str,
    job: &Refetch,
) -> Result<bool> {
    let wire_id = MessageId::new(job.wire_id.clone())?;
    store.write(|tx| {
        ensure_retry_account(tx, account)?;
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, job.answer_key.as_bytes(), 1)?
            .is_some()
        {
            return Ok(false);
        }
        let current = super::outgoing::outbox()
            .by_wire_id_in_write(tx, &wire_id)?
            .ok_or_else(|| {
                Error::invalid("retry", "original send vanished before refetch query")
            })?;
        ensure_original_unchanged(&current, prepared_from, Timestamp::now())?;
        ensure_retry_peer_unchanged(tx, peer.key, peer.identity, peer.session)?;
        tx.put_json(
            Table::Checkpoints,
            job.answer_key.as_bytes(),
            1,
            &serde_json::json!({"status": "fetching", "query": query, "count": job.count}),
        )?;
        tx.put_json(Table::Checkpoints, refetch_key(query).as_bytes(), 1, job)?;
        Ok(true)
    })
}

/// Finish a retry answer from the directory reply to its key query.
/// The transport caller must own a live query on the current connection;
/// normal receive enforces this through ActiveRefetches before calling here.
/// `Ok(None)` when the reply is not one of ours, `Ok(Some(None))` when it was
/// ours but nothing is sent (the reason is recorded), else the resend.
///
/// A fresh session from the reply's bundle replaces the stale one, as the APK
/// does after a registration change (`151.A00`). The pinned identity still
/// holds: a bundle with a different identity is recorded `identity_changed`.
pub async fn complete_refetch(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    epoch: ConnectionEpoch,
) -> Result<Option<Option<Node>>> {
    complete_refetch_inner(store, account, node, epoch, None).await
}

async fn complete_refetch_inner(
    store: &ProtocolStore,
    account: &AccountRecord,
    node: &Node,
    epoch: ConnectionEpoch,
    expected: Option<&Refetch>,
) -> Result<Option<Option<Node>>> {
    // APK 43.72 0ln.A01 invokes IQ callbacks only for exact result/error.
    // A same-ID request must not consume the reserved refetch or retry history.
    if node.description != "iq"
        || !matches!(
            node.get_attr("type").and_then(|v| v.as_text()),
            Some("result" | "error")
        )
    {
        return Ok(None);
    }
    let Some(query) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(None);
    };
    let job: Option<Refetch> =
        store.read(|tx| tx.get_json(Table::Checkpoints, refetch_key(query).as_bytes(), 1))?;
    if expected.is_some_and(|expected| job.as_ref() != Some(expected)) {
        return Err(Error::invalid(
            "retry",
            "written refetch job changed before reply",
        ));
    }
    let Some(job) = job else {
        return Ok(None);
    };
    let refuse = |reason: &str| -> Result<Option<Option<Node>>> {
        store.write(|tx| {
            ensure_refetch_owned(tx, query, &job)?;
            // Retire the fetch only with both durable refusal records. A store
            // fault must not leave a fetching answer without its pending job.
            tx.delete(Table::Checkpoints, refetch_key(query).as_bytes())?;
            tx.put_json(
                Table::Checkpoints,
                job.answer_key.as_bytes(),
                1,
                &serde_json::json!({"status": "unanswered", "reason": reason}),
            )?;
            if tx.get_json::<serde_json::Value>(Table::Checkpoints, job.unanswered_key.as_bytes(), 1)?.is_none() {
                tx.put_json(Table::Checkpoints, job.unanswered_key.as_bytes(), 1,
                    &serde_json::json!({"status":"unanswered", "reason":reason, "observed_at":Timestamp::now()}))?;
            }
            Ok(())
        })?;
        tracing::warn!(reason, "retry receipt not answered");
        Ok(Some(None))
    };
    if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
        return refuse("refetch_refused");
    }
    let wire_id = MessageId::new(job.wire_id.clone())?;
    let outbox = super::outgoing::outbox();
    let Some(entry) = store.read(|tx| outbox.by_wire_id(tx, &wire_id))? else {
        return refuse("original_send_missing");
    };
    let peer = entry.intent.chat.jid().clone();
    // APK 37.71 DNP.run rechecks whether the recipient already has the
    // message before resending. Delivery may arrive while the query waits.
    let alias = super::inbox::lid_for(store, &peer)?;
    let addresses = std::iter::once(&peer)
        .chain(alias.as_ref())
        .filter_map(|jid| DeviceAddress::from_jid(jid).ok())
        .collect::<Vec<_>>();
    if entry.facts.iter().any(|fact| {
        matches!(
            fact.kind,
            DeliveryFactKind::Delivered | DeliveryFactKind::Read | DeliveryFactKind::Played
        ) && fact
            .device
            .as_ref()
            .is_none_or(|device| addresses.contains(device))
    }) {
        return refuse("already_delivered");
    }
    if !entry
        .facts
        .iter()
        .any(|fact| fact.kind == DeliveryFactKind::ServerAccepted)
        || entry
            .facts
            .iter()
            .any(|fact| fact.kind == DeliveryFactKind::Failed)
    {
        return refuse("original_not_server_accepted");
    }
    let Some(first) = entry.attempts.first() else {
        return refuse("original_attempt_missing");
    };
    // APK 37.71 150.A04 0416..043c: originals older than 60 days refuse.
    // Reapply eligibility after waiting for the bundle, before a new send.
    if original_too_old(first.prepared_at, Timestamp::now()) {
        return refuse("original_older_than_60_days");
    }
    let Some(original) = first.payload.clone() else {
        return refuse("original_stanza_missing");
    };
    let bundle = match crate::signal::direct::prekey_bundle_for(node, &peer) {
        Ok(bundle) => bundle,
        Err(Error::InvalidInput { .. }) => return refuse("refetch_bundle_unusable"),
        Err(error) => return Err(error),
    };
    let peer_key = format!("comparison-direct:{peer}");
    let (source, identity, previous_session) = store.read(|tx| {
        Ok((
            tx.get_json::<Vec<u8>>(
                Table::Checkpoints,
                source_key(wire_id.as_str()).as_bytes(),
                1,
            )?,
            tx.get_json::<Vec<u8>>(Table::PeerIdentities, peer_key.as_bytes(), 1)?,
            tx.get_json::<Vec<u8>>(Table::SignalSessions, peer_key.as_bytes(), 1)?,
        ))
    })?;
    let Some(source) = source else {
        return refuse("plaintext_not_retained");
    };
    let prepared = match crate::signal::direct::prepare_payload(
        account,
        &peer,
        Some(&bundle),
        None,
        identity.as_deref(),
        &source,
    )
    .await
    {
        Ok(prepared) => prepared,
        Err(Error::IdentityChanged(_)) => return refuse("identity_changed"),
        Err(Error::InvalidInput { .. }) => return refuse("refetch_bundle_unusable"),
        Err(error) => return Err(error),
    };
    let resend = replace_enc(&original, prepared.kind, prepared.ciphertext, job.count)?;
    let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&resend)?;
    store.write(|tx| {
        ensure_retry_account(tx, account)?;
        ensure_retry_source(tx, wire_id.as_str(), &source)?;
        ensure_refetch_owned(tx, query, &job)?;
        ensure_retry_peer_unchanged(tx, &peer_key, identity.as_deref(), previous_session.as_deref())?;
        // The consumed job, replacement session and reserved same-ID resend
        // commit together; preparation/storage failure leaves the fetch held.
        tx.delete(Table::Checkpoints, refetch_key(query).as_bytes())?;
        let mut current = outbox
            .by_wire_id_in_write(tx, &wire_id)?
            .ok_or_else(|| Error::invalid("retry", "original send vanished"))?;
        ensure_original_unchanged(&current, &entry, Timestamp::now())?;
        tx.put_json(
            Table::Checkpoints,
            job.answer_key.as_bytes(),
            1,
            &serde_json::json!({"status": "reserved", "count": job.count, "fresh_session": true, "reserved_at": Timestamp::now()}),
        )?;
        current.attempts.push(AttemptRecord {
            index: current.attempts.len() as u32,
            prepared_at: Timestamp::now(),
            epoch: Some(epoch),
            payload: Some(wire.into()),
            re_encryption: true,
        });
        tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &*prepared.session)?;
        tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &prepared.identity)?;
        tx.put_json(Table::Outbox, &current.local_id.0, 1, &current)
    })?;
    Ok(Some(Some(resend)))
}

// DNP.run checks delivery before resend; async preparation must not reserve
// against an older outbox snapshot. Apply the same transaction boundary to
// both saved-session and fresh-bundle answers (docs/protocol/retry-answers.md).
fn ensure_original_unchanged(
    current: &OutboxEntry,
    prepared_from: &OutboxEntry,
    now: Timestamp,
) -> Result<()> {
    if current != prepared_from {
        return Err(Error::invalid(
            "retry",
            "original send changed before retry reservation",
        ));
    }
    let first = prepared_from.attempts.first().ok_or_else(|| {
        Error::invalid("retry", "original attempt missing before retry reservation")
    })?;
    if original_too_old(first.prepared_at, now) {
        return Err(Error::invalid(
            "retry",
            "original expired before retry reservation",
        ));
    }
    Ok(())
}

// Both retry paths await crypto outside the transaction. The original
// plaintext is part of that preparation snapshot, just like peer ratchet state.
fn ensure_retry_source(tx: &WriteTx<'_>, wire_id: &str, expected: &[u8]) -> Result<()> {
    let current: Option<Vec<u8>> =
        tx.get_json(Table::Checkpoints, source_key(wire_id).as_bytes(), 1)?;
    if current.as_deref() != Some(expected) {
        return Err(Error::invalid(
            "retry",
            "original retry source changed before reservation",
        ));
    }
    Ok(())
}

// Retry crypto uses the drive's cached account. Compare the authority it
// actually consumes, not mutable authentication timestamps/observed credentials.
// Local commit guard, not a native storage-format or key-rotation requirement.
fn ensure_retry_account(tx: &WriteTx<'_>, prepared_from: &AccountRecord) -> Result<()> {
    let current = AccountRecord::load_in_write(tx)?
        .ok_or_else(|| Error::invalid("retry", "local account missing before retry reservation"))?;
    let same_identity = current
        .signal_identity
        .as_ref()
        .zip(prepared_from.signal_identity.as_ref())
        .is_some_and(|(current, prior)| {
            current.0.public == prior.0.public && current.0.private == prior.0.private
        });
    if !current.is_registered()
        || !prepared_from.is_registered()
        || current.generation != tx.generation()
        || current.generation != prepared_from.generation
        || current.phone != prepared_from.phone
        || current.own_jid != prepared_from.own_jid
        || current.signal_registration_id.is_none()
        || current.signal_registration_id != prepared_from.signal_registration_id
        || !same_identity
    {
        return Err(Error::invalid(
            "retry",
            "local account changed before retry reservation",
        ));
    }
    Ok(())
}

// Local optimistic concurrency guard, not a native identity-reset policy.
// Crypto preparation awaits outside the write transaction; a newer ratchet or
// pin must survive rather than being overwritten by this prepared replacement.
fn ensure_retry_peer_unchanged(
    tx: &WriteTx<'_>,
    peer_key: &str,
    identity: Option<&[u8]>,
    session: Option<&[u8]>,
) -> Result<()> {
    let held_identity: Option<Vec<u8>> =
        tx.get_json(Table::PeerIdentities, peer_key.as_bytes(), 1)?;
    let held_session: Option<Vec<u8>> =
        tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1)?;
    if held_identity.as_deref() != identity || held_session.as_deref() != session {
        return Err(Error::invalid(
            "retry",
            "peer state changed before retry reservation",
        ));
    }
    Ok(())
}

/// The original stanza with its `enc` re-encrypted and its protocol retry
/// count updated; message id and unrelated attributes/children stay unchanged.
fn replace_enc(original: &[u8], kind: &str, ciphertext: Vec<u8>, count: u32) -> Result<Node> {
    // Higher counts require the unported base-key comparison policy. Keep
    // even restored/refetched jobs inside the same supported answer boundary.
    if !(1..=2).contains(&count) {
        return Err(Error::invalid(
            "retry",
            "only retry counts 1 and 2 can be answered",
        ));
    }
    let limits = crate::profile::Limits::default();
    let raw = crate::binary::unpack_frame_payload(original, &limits)?;
    let mut stanza =
        crate::binary::Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits)
            .decode_node()?;
    let NodeContent::Children(children) = &mut stanza.content else {
        return Err(Error::invalid("retry", "original stanza has no children"));
    };
    let mut replaced = 0;
    for child in children.iter_mut().filter(|c| c.description == "enc") {
        // APK 43.72 FTy.AK7 passes SendE2EMessageJob.retryCount through
        // F3f.A04 to A09, which emits its nonzero decimal count unchanged.
        *child = child
            .clone()
            .attr("type", kind)
            .attr("count", count.to_string())
            .bytes(ciphertext.clone());
        replaced += 1;
    }
    if replaced != 1 {
        return Err(Error::invalid(
            "retry",
            "original stanza must hold exactly one enc",
        ));
    }
    Ok(stanza)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            ids::{AccountId, ChatId, ClientRequestId},
            jid::Jid,
            message::MessageContent,
            send::{AttemptProgress, DeliveryFact, SendRequest},
        },
        session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::{DeviceId, IdentityKeyPair, KeyPair, PreKeyBundle};

    fn receipt(peer: &Jid, count: u32, registration: u32) -> Node {
        // Shape observed live 2026-09-24 (receipts::RetryRequest).
        Node::new("receipt")
            .attr("from", peer.clone())
            .attr("type", "retry")
            .attr("id", "W1")
            .children(vec![
                Node::new("retry")
                    .attr("id", "W1")
                    .attr("t", "1")
                    .attr("count", count.to_string())
                    .attr("v", "1"),
                Node::new("registration").bytes(registration.to_be_bytes().to_vec()),
            ])
    }

    fn reason(store: &ProtocolStore, count: u32, peer: &Jid) -> Option<String> {
        let key = format!("retry-unanswered:W1:{peer}:{count}");
        store
            .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .map(|v| v["reason"].as_str().unwrap().to_owned())
    }

    #[test]
    fn retry_count_replaces_only_the_enc_count_and_preserves_original_stanza() {
        let original = Node::new("message")
            .attr("id", "W1")
            .attr("type", "text")
            .children(vec![
                Node::new("enc")
                    .attr("v", "2")
                    .attr("type", "msg")
                    .attr("mediatype", "image")
                    .attr("count", "9")
                    .bytes(vec![1, 2]),
                Node::new("tctoken").bytes(vec![3, 4]),
            ]);
        let encoded = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&original)
            .unwrap();
        for count in [1, 2] {
            let answer = replace_enc(&encoded, "pkmsg", vec![8, 9], count).unwrap();
            assert_eq!(answer.attributes, original.attributes);
            assert_eq!(answer.child("tctoken"), original.child("tctoken"));
            let enc = answer.child("enc").unwrap();
            assert_eq!(
                enc.get_attr("type").and_then(|v| v.as_text()),
                Some("pkmsg")
            );
            assert_eq!(enc.content, NodeContent::Bytes(vec![8, 9].into()));
            assert_eq!(
                enc.get_attr("count").and_then(|v| v.as_text()),
                Some(count.to_string().as_str())
            );
            assert_eq!(
                enc.attributes
                    .iter()
                    .filter(|attr| attr.name == "count")
                    .count(),
                1
            );
            assert_eq!(
                enc.get_attr("v"),
                original.child("enc").unwrap().get_attr("v")
            );
            assert_eq!(
                enc.get_attr("mediatype"),
                original.child("enc").unwrap().get_attr("mediatype")
            );
        }
        assert!(replace_enc(&encoded, "msg", vec![7], 0).is_err());
        assert!(replace_enc(&encoded, "msg", vec![7], 3).is_err());
        assert!(replace_enc(&encoded, "msg", vec![7], 4).is_err());
        assert!(replace_enc(&encoded, "msg", vec![7], MAX_COUNT + 1).is_err());
        assert_eq!(
            original
                .child("enc")
                .unwrap()
                .get_attr("count")
                .and_then(|v| v.as_text()),
            Some("9")
        );
    }

    #[test]
    fn retry_age_is_strict_and_does_not_overflow() {
        assert!(!original_too_old(Timestamp(0), Timestamp(MAX_AGE_MS)));
        assert!(original_too_old(Timestamp(0), Timestamp(MAX_AGE_MS + 1)));
        assert!(!original_too_old(Timestamp(1), Timestamp(0)));
        assert!(original_too_old(Timestamp(i64::MIN), Timestamp(i64::MAX)));
        assert!(!original_too_old(Timestamp(i64::MAX), Timestamp(i64::MIN)));
    }

    #[test]
    fn retry_source_retention_and_commit_preserve_original_bytes_across_reopen() {
        for change in ["same", "different", "missing", "malformed"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("retry-source.redb");
            let open = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("test").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [7; 32],
                    ))),
                    create: true,
                })
                .unwrap()
            };
            let store = open();
            let original = vec![1, 2, 3];
            store
                .write(|tx| keep_source_in(tx, "W1", &original))
                .unwrap();
            let retained = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            // Exact repetition is idempotent; conflicting producer writes
            // roll back earlier work in the caller's reservation transaction.
            store
                .write(|tx| keep_source_in(tx, "W1", &original))
                .unwrap();
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                retained
            );
            assert!(
                store
                    .write(|tx| {
                        tx.put_json(Table::Checkpoints, b"synthetic-side-effect", 1, &true)?;
                        keep_source_in(tx, "W1", &[4, 5, 6])
                    })
                    .is_err()
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                retained
            );
            // Model an independently changed/deleted/corrupt source after the
            // retry loaded the original but before its crypto commit.
            store
                .write(|tx| match change {
                    "different" => tx.put_json(
                        Table::Checkpoints,
                        source_key("W1").as_bytes(),
                        1,
                        &vec![4u8, 5, 6],
                    ),
                    "missing" => {
                        tx.delete(Table::Checkpoints, source_key("W1").as_bytes())?;
                        Ok(())
                    }
                    "malformed" => tx.put_json(
                        Table::Checkpoints,
                        source_key("W1").as_bytes(),
                        1,
                        &serde_json::json!({"invalid":"fixture"}),
                    ),
                    _ => Ok(()),
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let result = store.write(|tx| {
                tx.put_json(Table::Checkpoints, b"synthetic-retry-reservation", 1, &true)?;
                ensure_retry_source(tx, "W1", &original)
            });
            if change == "same" {
                result.unwrap();
            } else {
                assert!(result.is_err(), "{change}");
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    before
                );
            }
            if change == "malformed" {
                assert!(
                    store
                        .write(|tx| keep_source_in(tx, "W1", &original))
                        .is_err()
                );
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    before
                );
            }
            let after = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            drop(store);
            let reopened = open();
            assert_eq!(
                reopened
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                after
            );
        }
    }

    #[test]
    fn retry_account_authority_refuses_changed_keys_and_allows_observation_updates() {
        for change in [
            "same",
            "timestamp",
            "credential",
            "missing",
            "unregistered",
            "revoked",
            "generation",
            "store_generation",
            "phone",
            "own_jid",
            "registration_id",
            "missing_registration",
            "public",
            "private",
            "missing_identity",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("retry-account.redb");
            let open = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("test").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [7; 32],
                    ))),
                    create: true,
                })
                .unwrap()
            };
            let store = open();
            let mut expected = AccountRecord {
                generation: store.generation(),
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(31),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public: [1; 32],
                    private: [2; 32],
                })),
                ..Default::default()
            };
            let mut current = expected.clone();
            match change {
                "timestamp" => current.last_authenticated_at = Some(Timestamp(123)),
                "credential" => current
                    .credentials
                    .push(crate::session::ObservedCredential {
                        name: "offline-observation".into(),
                        value: zeroize::Zeroizing::new(vec![3]),
                        observed_at: Timestamp(123),
                        source_step: None,
                    }),
                "unregistered" => current.registration = RegistrationStatus::Unregistered,
                "revoked" => current.registration = RegistrationStatus::Revoked,
                "generation" => current.generation.0 += 1,
                "store_generation" => {
                    current.generation.0 += 1;
                    expected.generation = current.generation;
                }
                "phone" => {
                    current.phone =
                        Some(crate::model::phone::PhoneNumber::new(1, "5555550123").unwrap())
                }
                "own_jid" => {
                    current.own_jid = Some(Jid::parse("15555550123@s.whatsapp.net").unwrap())
                }
                "registration_id" => current.signal_registration_id = Some(32),
                "missing_registration" => current.signal_registration_id = None,
                "public" => current.signal_identity.as_mut().unwrap().0.public[0] ^= 1,
                "private" => current.signal_identity.as_mut().unwrap().0.private[0] ^= 1,
                "missing_identity" => current.signal_identity = None,
                _ => {}
            }
            if change != "missing" {
                store.write(|tx| current.save(tx)).unwrap();
            }
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        [
                            Table::Account,
                            Table::Checkpoints,
                            Table::Outbox,
                            Table::SignalSessions,
                            Table::PeerIdentities,
                        ]
                        .into_iter()
                        .map(|table| tx.scan(table, 1, 100))
                        .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            let result = store.write(|tx| {
                ensure_retry_account(tx, &expected)?;
                tx.put_json(Table::Checkpoints, b"synthetic-retry-reservation", 1, &true)
            });
            if matches!(change, "same" | "timestamp" | "credential") {
                result.unwrap();
            } else {
                assert!(result.is_err(), "{change}");
                assert_eq!(snapshot(&store), before, "{change}");
            }
            let after = snapshot(&store);
            drop(store);
            let reopened = open();
            assert_eq!(snapshot(&reopened), after, "{change}");
        }
    }

    #[test]
    fn refetch_query_reservation_rechecks_inputs_and_commits_once() {
        for change in [
            "unchanged",
            "delivery",
            "failure",
            "attempt",
            "identity",
            "session",
            "session_appeared",
            "expired",
            "missing",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("refetch-query.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [7; 32],
                ))),
                create: true,
            })
            .unwrap();
            let account = AccountRecord {
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(31),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public: [1; 32],
                    private: [2; 32],
                })),
                generation: store.generation(),
                ..Default::default()
            };
            store.write(|tx| account.save(tx)).unwrap();
            let outbox = super::super::outgoing::outbox();
            let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
            let peer_key = format!("comparison-direct:{peer}");
            let expected_session = (change != "session_appeared").then_some([2u8]);
            let prepared_from = store
                .write(|tx| {
                    let request = SendRequest {
                        client_request_id: ClientRequestId::new("refetch-query-race")?,
                        chat: ChatId::from_jid(&peer)?,
                        content: MessageContent::Text {
                            body: "fixture".into(),
                            quoted: None,
                            style: Default::default(),
                        },
                    };
                    let id = outbox.enqueue(tx, &request)?.0.local_id;
                    let mut entry = outbox.get_mut(tx, id)?.unwrap();
                    entry.wire_id = Some(MessageId::new("W1")?);
                    entry.attempts.push(AttemptRecord {
                        index: 0,
                        prepared_at: if change == "expired" {
                            Timestamp(Timestamp::now().as_millis() - MAX_AGE_MS - 1)
                        } else {
                            Timestamp::now()
                        },
                        epoch: Some(ConnectionEpoch(1)),
                        payload: Some(vec![1].into()),
                        re_encryption: false,
                    });
                    outbox.set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                    outbox.record_fact(
                        tx,
                        &mut entry,
                        DeliveryFact {
                            kind: DeliveryFactKind::ServerAccepted,
                            device: None,
                            observed_at: Timestamp::now(),
                            server_timestamp: None,
                            code: None,
                        },
                    )?;
                    tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &vec![1u8])?;
                    if let Some(session) = expected_session {
                        tx.put_json(
                            Table::SignalSessions,
                            peer_key.as_bytes(),
                            1,
                            &session.to_vec(),
                        )?;
                    }
                    Ok(entry)
                })
                .unwrap();
            // Commit competing receive/outbox work after capturing the inputs.
            store
                .write(|tx| {
                    let mut current = prepared_from.clone();
                    match change {
                        "delivery" | "failure" => current.facts.push(DeliveryFact {
                            kind: if change == "delivery" {
                                DeliveryFactKind::Delivered
                            } else {
                                DeliveryFactKind::Failed
                            },
                            device: None,
                            observed_at: Timestamp::now(),
                            server_timestamp: None,
                            code: None,
                        }),
                        "attempt" => current.attempts.push(AttemptRecord {
                            index: 1,
                            prepared_at: Timestamp::now(),
                            epoch: Some(ConnectionEpoch(2)),
                            payload: Some(vec![2].into()),
                            re_encryption: true,
                        }),
                        "identity" => {
                            tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &vec![9u8])?
                        }
                        "session" | "session_appeared" => {
                            tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &vec![9u8])?
                        }
                        "missing" => {
                            tx.delete(Table::Outbox, &current.local_id.0)?;
                            return Ok(());
                        }
                        _ => {}
                    }
                    tx.put_json(Table::Outbox, &current.local_id.0, 1, &current)
                })
                .unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        [
                            Table::Outbox,
                            Table::Checkpoints,
                            Table::PeerIdentities,
                            Table::SignalSessions,
                        ]
                        .into_iter()
                        .map(|table| tx.scan(table, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let job = Refetch {
                wire_id: "W1".into(),
                answer_key: "answer".into(),
                unanswered_key: "unanswered".into(),
                count: 2,
            };
            let before = snapshot();
            let result = reserve_refetch(
                &store,
                &prepared_from,
                &account,
                RetryPeerSnapshot {
                    key: &peer_key,
                    identity: Some(&[1]),
                    session: expected_session.as_ref().map(|s| s.as_slice()),
                },
                "Q1",
                &job,
            );
            if change == "unchanged" {
                assert!(result.unwrap());
                let reserved = snapshot();
                assert!(
                    !reserve_refetch(
                        &store,
                        &prepared_from,
                        &account,
                        RetryPeerSnapshot {
                            key: &peer_key,
                            identity: Some(&[1]),
                            session: expected_session.as_ref().map(|s| s.as_slice())
                        },
                        "Q2",
                        &job
                    )
                    .unwrap()
                );
                assert_eq!(snapshot(), reserved);
                assert!(
                    store
                        .read(|tx| tx.get_json::<Refetch>(
                            Table::Checkpoints,
                            refetch_key("Q1").as_bytes(),
                            1
                        ))
                        .unwrap()
                        .is_some()
                );
                assert!(
                    store
                        .read(|tx| tx.get_json::<Refetch>(
                            Table::Checkpoints,
                            refetch_key("Q2").as_bytes(),
                            1
                        ))
                        .unwrap()
                        .is_none()
                );
            } else {
                assert!(result.is_err(), "{change}");
                assert_eq!(snapshot(), before, "{change}");
            }
        }
    }

    #[test]
    fn retry_reservation_preserves_concurrent_outbox_changes_and_age_boundary() {
        for change in ["unchanged", "delivery", "failure", "attempt", "expired"] {
            let dir = tempfile::tempdir().unwrap();
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("retry-reservation.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [7; 32],
                ))),
                create: true,
            })
            .unwrap();
            let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
            let outbox = super::super::outgoing::outbox();
            let prepared_from = store
                .write(|tx| {
                    let request = SendRequest {
                        client_request_id: ClientRequestId::new("reservation-race")?,
                        chat: ChatId::from_jid(&peer)?,
                        content: MessageContent::Text {
                            body: "fixture".into(),
                            quoted: None,
                            style: Default::default(),
                        },
                    };
                    let id = outbox.enqueue(tx, &request)?.0.local_id;
                    let mut entry = outbox.get_mut(tx, id)?.unwrap();
                    entry.attempts.push(AttemptRecord {
                        index: 0,
                        prepared_at: Timestamp(0),
                        epoch: Some(ConnectionEpoch(1)),
                        payload: Some(vec![1].into()),
                        re_encryption: false,
                    });
                    entry.facts.push(DeliveryFact {
                        kind: DeliveryFactKind::ServerAccepted,
                        device: None,
                        observed_at: Timestamp(0),
                        server_timestamp: None,
                        code: None,
                    });
                    tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)?;
                    Ok(entry)
                })
                .unwrap();
            // Another receive/send commits after the retry captured its inputs.
            store
                .write(|tx| {
                    let mut current = prepared_from.clone();
                    match change {
                        "delivery" | "failure" => current.facts.push(DeliveryFact {
                            kind: if change == "delivery" {
                                DeliveryFactKind::Delivered
                            } else {
                                DeliveryFactKind::Failed
                            },
                            device: None,
                            observed_at: Timestamp(1),
                            server_timestamp: None,
                            code: None,
                        }),
                        "attempt" => current.attempts.push(AttemptRecord {
                            index: 1,
                            prepared_at: Timestamp(1),
                            epoch: Some(ConnectionEpoch(2)),
                            payload: Some(vec![2].into()),
                            re_encryption: true,
                        }),
                        _ => {}
                    }
                    tx.put_json(Table::Outbox, &current.local_id.0, 1, &current)
                })
                .unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        [Table::Outbox, Table::Checkpoints, Table::SignalSessions]
                            .into_iter()
                            .map(|table| tx.scan(table, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            let now = Timestamp(MAX_AGE_MS + i64::from(change == "expired"));
            let result = store.write(|tx| {
                let current: OutboxEntry = tx
                    .get_json(Table::Outbox, &prepared_from.local_id.0, 1)?
                    .unwrap();
                ensure_original_unchanged(&current, &prepared_from, now)?;
                tx.put_json(Table::Checkpoints, b"reserved-answer", 1, &true)?;
                tx.put_json(Table::SignalSessions, b"peer", 1, &vec![9u8])
            });
            if change == "unchanged" {
                assert!(result.is_ok()); // Exactly60 days remains eligible.
                assert_ne!(snapshot(), before);
            } else {
                assert!(result.is_err(), "{change}");
                assert_eq!(snapshot(), before, "{change}");
            }
        }
    }

    #[tokio::test]
    async fn queued_refetch_rechecks_owner_before_ack_and_query_encoding() {
        for change_at in ["none", "ack", "query"] {
            for change in ["missing_job", "replaced_job", "settled_answer"] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("queued-refetch.redb");
                let open = || {
                    ProtocolStore::open(OpenOptions {
                        path: path.clone(),
                        account_id: AccountId::new("test").unwrap(),
                        key_source: std::sync::Arc::new(StaticKeySource::new(
                            MasterKey::from_bytes([7; 32]),
                        )),
                        create: true,
                    })
                    .unwrap()
                };
                let store = std::sync::Arc::new(open());
                let job = Refetch {
                    wire_id: "W1".into(),
                    answer_key: "answer".into(),
                    unanswered_key: "unanswered".into(),
                    count: 2,
                };
                store
                    .write(|tx| {
                        tx.put_json(Table::Checkpoints, refetch_key("Q1").as_bytes(), 1, &job)?;
                        tx.put_json(
                            Table::Checkpoints,
                            b"answer",
                            1,
                            &serde_json::json!({"status":"fetching","query":"Q1","count":2}),
                        )
                    })
                    .unwrap();
                let authority = RefetchWriteGuard {
                    query: "Q1".into(),
                    job: job.clone(),
                };
                let (_incoming, incoming) = tokio::sync::mpsc::channel(1);
                let (outgoing, mut writes) = tokio::sync::mpsc::channel(2);
                let link = super::super::connection::NodeLink::new(incoming, outgoing);
                let worker_store = store.clone();
                let task = tokio::spawn(async move {
                    link.send_retry_answer(
                        &worker_store,
                        &ReservedAnswer {
                            node: Node::new("iq").attr("id", "Q1"),
                            confirmation: Some(Node::new("ack").attr("id", "R1")),
                            refetch: Some(authority),
                        },
                    )
                    .await
                });
                let mut expected = None;
                for stage in ["ack", "query"] {
                    let queued = writes.recv().await.expect("queued stage");
                    if change_at == stage {
                        store
                            .write(|tx| match change {
                                "missing_job" => {
                                    tx.delete(Table::Checkpoints, refetch_key("Q1").as_bytes())?;
                                    Ok(())
                                }
                                "replaced_job" => {
                                    let mut replacement = job.clone();
                                    replacement.wire_id = "W2".into();
                                    tx.put_json(
                                        Table::Checkpoints,
                                        refetch_key("Q1").as_bytes(),
                                        1,
                                        &replacement,
                                    )
                                }
                                _ => tx.put_json(
                                    Table::Checkpoints,
                                    b"answer",
                                    1,
                                    &serde_json::json!({"status":"reserved"}),
                                ),
                            })
                            .unwrap();
                        expected = Some(
                            store
                                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                                .unwrap(),
                        );
                    }
                    let prepared = queued.prepare();
                    if change_at == stage {
                        assert!(prepared.is_none(), "{stage}/{change}");
                        break;
                    }
                    let (node, written) = prepared.expect("unchanged owner admits encoding");
                    assert_eq!(node.description, if stage == "ack" { "ack" } else { "iq" });
                    written.send(Ok(())).unwrap();
                }
                assert_eq!(task.await.unwrap().is_ok(), change_at == "none");
                assert!(writes.recv().await.is_none());
                if let Some(expected) = expected {
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        expected
                    );
                    drop(store);
                    let reopened = open();
                    assert_eq!(
                        reopened
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        expected
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn refetch_settlement_preserves_replaced_job_and_non_fetching_answer() {
        for change in [
            "unchanged",
            "missing_job",
            "wire_id",
            "answer_key",
            "unanswered_key",
            "job_count",
            "missing_answer",
            "status",
            "query",
            "count",
            "string_count",
            "legacy",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("refetch-ownership.redb");
            let open = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("test").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [7; 32],
                    ))),
                    create: true,
                })
                .unwrap()
            };
            let store = open();
            let expected = Refetch {
                wire_id: "W1".into(),
                answer_key: "answer".into(),
                unanswered_key: "unanswered".into(),
                count: 2,
            };
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        refetch_key("Q1").as_bytes(),
                        1,
                        &expected,
                    )?;
                    tx.put_json(
                        Table::Checkpoints,
                        b"answer",
                        1,
                        &serde_json::json!({"status":"fetching", "query":"Q1", "count":2}),
                    )
                })
                .unwrap();
            let write_guard = RefetchWriteGuard {
                query: "Q1".into(),
                job: expected.clone(),
            };
            // Snapshot was loaded before another transaction changed ownership.
            store
                .write(|tx| {
                    let mut job = expected.clone();
                    match change {
                        "missing_job" => {
                            tx.delete(Table::Checkpoints, refetch_key("Q1").as_bytes())?;
                        }
                        "wire_id" | "answer_key" | "unanswered_key" | "job_count" => {
                            match change {
                                "wire_id" => job.wire_id = "W2".into(),
                                "answer_key" => job.answer_key = "new-answer".into(),
                                "unanswered_key" => job.unanswered_key = "new-unanswered".into(),
                                _ => job.count = 1,
                            }
                            tx.put_json(Table::Checkpoints, refetch_key("Q1").as_bytes(), 1, &job)?;
                        }
                        "missing_answer" => {
                            tx.delete(Table::Checkpoints, b"answer")?;
                        }
                        "status" | "query" | "count" | "string_count" | "legacy" => {
                            let mut state =
                                serde_json::json!({"status":"fetching", "query":"Q1", "count":2});
                            match change {
                                "status" => state["status"] = "reserved".into(),
                                "query" => state["query"] = "Q2".into(),
                                "count" => state["count"] = 1.into(),
                                "string_count" => state["count"] = "2".into(),
                                _ => state = serde_json::json!({"query":"Q1"}),
                            }
                            tx.put_json(Table::Checkpoints, b"answer", 1, &state)?;
                        }
                        _ => {}
                    }
                    Ok(())
                })
                .unwrap();
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        [
                            Table::Checkpoints,
                            Table::Outbox,
                            Table::SignalSessions,
                            Table::PeerIdentities,
                        ]
                        .into_iter()
                        .map(|t| tx.scan(t, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            assert_eq!(
                write_guard.check_write_authority(&store).is_ok(),
                change == "unchanged",
                "{change}"
            );
            assert_eq!(snapshot(&store), before);
            let result = store.write(|tx| {
                ensure_refetch_owned(tx, "Q1", &expected)?;
                tx.delete(Table::Checkpoints, refetch_key("Q1").as_bytes())?;
                tx.put_json(
                    Table::Checkpoints,
                    b"answer",
                    1,
                    &serde_json::json!({"status":"reserved"}),
                )
            });
            if change == "unchanged" {
                result.unwrap();
                let settled = snapshot(&store);
                assert!(write_guard.check_write_authority(&store).is_err());
                assert!(
                    store
                        .write(|tx| ensure_refetch_owned(tx, "Q1", &expected))
                        .is_err()
                );
                assert_eq!(snapshot(&store), settled);
            } else {
                assert!(result.is_err(), "{change}");
                assert_eq!(snapshot(&store), before, "{change}");
                // Real refusal entrypoint must not retire a job on a stale or
                // legacy answer marker, even with an otherwise matching query.
                if matches!(
                    change,
                    "missing_answer" | "status" | "query" | "count" | "string_count" | "legacy"
                ) {
                    let reply = Node::new("iq").attr("id", "Q1").attr("type", "error");
                    assert!(
                        complete_refetch(
                            &store,
                            &AccountRecord::default(),
                            &reply,
                            ConnectionEpoch(1)
                        )
                        .await
                        .is_err()
                    );
                    assert_eq!(snapshot(&store), before);
                }
                drop(store);
                let store = open();
                assert_eq!(snapshot(&store), before, "{change}");
                assert!(write_guard.check_write_authority(&store).is_err());
                assert_eq!(snapshot(&store), before, "{change}");
            }
        }
    }

    #[test]
    fn retry_guard_preserves_newer_peer_state_and_pending_job() {
        for changed_table in [Table::PeerIdentities, Table::SignalSessions] {
            let dir = tempfile::tempdir().unwrap();
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("peer-change.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [7; 32],
                ))),
                create: true,
            })
            .unwrap();
            let peer_key = "comparison-direct:15555550100@s.whatsapp.net";
            store
                .write(|tx| {
                    tx.put_json(Table::PeerIdentities, peer_key.as_bytes(), 1, &vec![1u8])?;
                    tx.put_json(Table::SignalSessions, peer_key.as_bytes(), 1, &vec![2u8])?;
                    tx.put_json(
                        Table::Checkpoints,
                        refetch_key("Q1").as_bytes(),
                        1,
                        &Refetch {
                            wire_id: "W1".into(),
                            answer_key: "answer".into(),
                            unanswered_key: "unanswered".into(),
                            count: 2,
                        },
                    )
                })
                .unwrap();
            store
                .write(|tx| {
                    ensure_retry_peer_unchanged(tx, peer_key, Some(&[1]), Some(&[2]))?;
                    Ok(())
                })
                .unwrap();
            // Simulate another authenticated receive advancing the ratchet or
            // replacing the pin while async crypto is being prepared.
            store
                .write(|tx| tx.put_json(changed_table, peer_key.as_bytes(), 1, &vec![9u8]))
                .unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        [
                            Table::Checkpoints,
                            Table::PeerIdentities,
                            Table::SignalSessions,
                        ]
                        .into_iter()
                        .map(|t| tx.scan(t, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            assert!(
                store
                    .write(|tx| {
                        ensure_retry_peer_unchanged(tx, peer_key, Some(&[1]), Some(&[2]))?;
                        tx.delete(Table::Checkpoints, refetch_key("Q1").as_bytes())
                    })
                    .is_err()
            );
            assert_eq!(snapshot(), before);
        }
    }

    #[tokio::test]
    async fn live_refetch_reply_cannot_consume_replacement_job() {
        for kind in ["result", "error"] {
            for change in [
                "same",
                "missing",
                "wire_id",
                "answer_key",
                "unanswered_key",
                "count",
            ] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("live-job-owner.redb");
                let open = || {
                    ProtocolStore::open(OpenOptions {
                        path: path.clone(),
                        account_id: AccountId::new("test").unwrap(),
                        key_source: std::sync::Arc::new(StaticKeySource::new(
                            MasterKey::from_bytes([7; 32]),
                        )),
                        create: true,
                    })
                    .unwrap()
                };
                let store = open();
                let original = Refetch {
                    wire_id: "W1".into(),
                    answer_key: "answer".into(),
                    unanswered_key: "unanswered".into(),
                    count: 2,
                };
                let answer = ReservedAnswer {
                    node: Node::new("iq").attr("id", "Q1"),
                    confirmation: None,
                    refetch: Some(RefetchWriteGuard {
                        query: "Q1".into(),
                        job: original.clone(),
                    }),
                };
                let mut active = ActiveRefetches::default();
                // Missing/mismatched snapshots cannot create authority.
                for invalid in [
                    ReservedAnswer {
                        node: Node::new("iq").attr("id", "Q1"),
                        confirmation: None,
                        refetch: None,
                    },
                    ReservedAnswer {
                        node: Node::new("iq"),
                        confirmation: None,
                        refetch: answer.refetch.clone(),
                    },
                    ReservedAnswer {
                        node: Node::new("iq").attr("id", "OTHER"),
                        confirmation: None,
                        refetch: answer.refetch.clone(),
                    },
                ] {
                    assert!(active.record_written(&invalid).is_err());
                    assert!(active.0.is_empty());
                }
                active.record_written(&answer).unwrap();
                let mut replacement = original.clone();
                replacement.wire_id = "W2".into();
                assert!(
                    active
                        .record_written(&ReservedAnswer {
                            node: Node::new("iq").attr("id", "Q1"),
                            confirmation: None,
                            refetch: Some(RefetchWriteGuard {
                                query: "Q1".into(),
                                job: replacement
                            }),
                        })
                        .is_err()
                );
                // Duplicate registration did not overwrite the first snapshot.
                assert_eq!(active.0["Q1"].job, original);
                let mut current = original.clone();
                match change {
                    "wire_id" => current.wire_id = "W2".into(),
                    "answer_key" => current.answer_key = "new-answer".into(),
                    "unanswered_key" => current.unanswered_key = "new-unanswered".into(),
                    "count" => current.count = 1,
                    _ => {}
                }
                store.write(|tx| {
                    if change!="missing" {
                        tx.put_json(Table::Checkpoints,refetch_key("Q1").as_bytes(),1,&current)?;
                    }
                    tx.put_json(Table::Checkpoints,current.answer_key.as_bytes(),1,
                        &serde_json::json!({"status":"fetching","query":"Q1","count":current.count}))?;
                    tx.put_json(Table::SignalSessions,b"peer",1,&vec![1u8])?;
                    tx.put_json(Table::PeerIdentities,b"peer",1,&vec![2u8])
                }).unwrap();
                let snapshot = |store: &ProtocolStore| {
                    store
                        .read(|tx| {
                            [
                                Table::Checkpoints,
                                Table::Outbox,
                                Table::SignalSessions,
                                Table::PeerIdentities,
                            ]
                            .into_iter()
                            .map(|table| tx.scan(table, 1, 100))
                            .collect::<Result<Vec<_>>>()
                        })
                        .unwrap()
                };
                let before = snapshot(&store);
                let reply = Node::new("iq").attr("id", "Q1").attr("type", kind);
                let result = active
                    .complete(
                        &store,
                        &AccountRecord::default(),
                        &reply,
                        ConnectionEpoch(1),
                    )
                    .await;
                assert!(active.0.is_empty());
                if change == "same" {
                    assert!(matches!(result.unwrap(), Some(None)));
                    assert_ne!(snapshot(&store), before);
                } else {
                    assert!(result.is_err(), "{kind}/{change}");
                    assert_eq!(snapshot(&store), before);
                }
                let after = snapshot(&store);
                assert!(
                    active
                        .complete(
                            &store,
                            &AccountRecord::default(),
                            &reply,
                            ConnectionEpoch(1)
                        )
                        .await
                        .unwrap()
                        .is_none()
                );
                assert_eq!(snapshot(&store), after);
                drop(store);
                let reopened = open();
                assert_eq!(snapshot(&reopened), after);
            }
        }
    }

    #[tokio::test]
    async fn active_refetch_callbacks_do_not_survive_disconnect_or_failed_completion() {
        for outcome in ["live", "disconnect", "local_failure"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("active-refetch.redb");
            let open = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("test").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [7; 32],
                    ))),
                    create: true,
                })
                .unwrap()
            };
            let store = open();
            let fetching = serde_json::json!({"status":"fetching","query":"Q1","count":2});
            let job = Refetch {
                wire_id: "W1".into(),
                answer_key: "answer".into(),
                unanswered_key: "unanswered".into(),
                count: 2,
            };
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, refetch_key("Q1").as_bytes(), 1, &job)?;
                    tx.put_json(Table::Checkpoints, b"answer", 1, &fetching)?;
                    tx.put_json(Table::SignalSessions, b"peer", 1, &vec![1u8])?;
                    tx.put_json(Table::PeerIdentities, b"peer", 1, &vec![2u8])
                })
                .unwrap();
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        [
                            Table::Checkpoints,
                            Table::Outbox,
                            Table::SignalSessions,
                            Table::PeerIdentities,
                        ]
                        .into_iter()
                        .map(|t| tx.scan(t, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            let account = AccountRecord::default();
            let reply = Node::new("iq").attr("id", "Q1").attr("type", "error");
            let mut active = ActiveRefetches::default();
            // A durable reservation alone is never a live callback.
            assert!(
                active
                    .complete(&store, &account, &reply, ConnectionEpoch(1))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(snapshot(&store), before);
            active
                .record_written(&ReservedAnswer {
                    node: Node::new("message").attr("id", "Q1"),
                    confirmation: None,
                    refetch: None,
                })
                .unwrap();
            assert!(active.0.is_empty());
            active
                .record_written(&ReservedAnswer {
                    node: Node::new("iq").attr("id", "Q1"),
                    confirmation: None,
                    refetch: Some(RefetchWriteGuard {
                        query: "Q1".into(),
                        job,
                    }),
                })
                .unwrap();
            for node in [
                Node::new("iq").attr("id", "Q1"),
                Node::new("iq").attr("id", "Q1").attr("type", "get"),
                Node::new("iq").attr("id", "Q1").attr("type", "set"),
                Node::new("message").attr("id", "Q1").attr("type", "result"),
                Node::new("iq").attr("id", "OTHER").attr("type", "error"),
            ] {
                assert!(
                    active
                        .complete(&store, &account, &node, ConnectionEpoch(1))
                        .await
                        .unwrap()
                        .is_none()
                );
                assert!(active.0.contains_key("Q1"));
                assert_eq!(snapshot(&store), before);
            }
            if outcome == "disconnect" {
                drop(active);
                drop(store);
                let store = open();
                let mut reconnected = ActiveRefetches::default();
                // Even an epoch counter reused after process restart cannot
                // revive this query: no write occurred on the new link.
                assert!(
                    reconnected
                        .complete(&store, &account, &reply, ConnectionEpoch(1))
                        .await
                        .unwrap()
                        .is_none()
                );
                assert_eq!(snapshot(&store), before);
            } else if outcome == "local_failure" {
                store
                    .write(|tx| {
                        tx.put_json(
                            Table::Checkpoints,
                            b"answer",
                            1,
                            &serde_json::json!({"status":"reserved","query":"Q1","count":2}),
                        )
                    })
                    .unwrap();
                let changed = snapshot(&store);
                assert!(
                    active
                        .complete(&store, &account, &reply, ConnectionEpoch(1))
                        .await
                        .is_err()
                );
                assert!(active.0.is_empty());
                assert_eq!(snapshot(&store), changed);
                // Repair only a synthetic fixture; an already-consumed live
                // callback still cannot be invoked by a second reply.
                store
                    .write(|tx| tx.put_json(Table::Checkpoints, b"answer", 1, &fetching))
                    .unwrap();
                assert!(
                    active
                        .complete(&store, &account, &reply, ConnectionEpoch(1))
                        .await
                        .unwrap()
                        .is_none()
                );
                assert_eq!(snapshot(&store), before);
            } else {
                assert!(matches!(
                    active
                        .complete(&store, &account, &reply, ConnectionEpoch(1))
                        .await
                        .unwrap(),
                    Some(None)
                ));
                assert!(active.0.is_empty());
                let settled = snapshot(&store);
                assert_ne!(settled, before);
                assert!(
                    active
                        .complete(&store, &account, &reply, ConnectionEpoch(1))
                        .await
                        .unwrap()
                        .is_none()
                );
                assert_eq!(snapshot(&store), settled);
            }
        }
    }

    #[tokio::test]
    async fn non_reply_iqs_preserve_refetch_and_retry_history_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("refetch.redb");
        let open = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [7; 32],
                ))),
                create: true,
            })
            .unwrap()
        };
        let store = open();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    refetch_key("Q1").as_bytes(),
                    1,
                    &Refetch {
                        wire_id: "W1".into(),
                        answer_key: "fixture-answer".into(),
                        unanswered_key: "fixture-unanswered".into(),
                        count: 2,
                    },
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    b"fixture-answer",
                    1,
                    &serde_json::json!({"status":"fetching", "query":"Q1", "count":2}),
                )
            })
            .unwrap();
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    [
                        Table::Checkpoints,
                        Table::Outbox,
                        Table::SignalSessions,
                        Table::PeerIdentities,
                    ]
                    .into_iter()
                    .map(|table| tx.scan(table, 1, 10000))
                    .collect::<Result<Vec<_>>>()
                })
                .unwrap()
        };
        let before = snapshot(&store);
        let account = AccountRecord::default();
        for kind in [
            None,
            Some("get"),
            Some("set"),
            Some("RESULT"),
            Some("unexpected"),
        ] {
            let mut node = Node::new("iq")
                .attr("id", "Q1")
                .children(vec![Node::new("error").attr("code", "503")]);
            if let Some(kind) = kind {
                node = node.attr("type", kind);
            }
            assert!(
                complete_refetch(&store, &account, &node, ConnectionEpoch(1))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(snapshot(&store), before);
        }
        drop(store);
        let store = open();
        assert_eq!(snapshot(&store), before);
        // A corrupt durable diagnostic row fails after deletion/first write
        // inside the transaction. The entire pending job and answer must survive.
        store
            .write(|tx| {
                tx.put(
                    Table::Checkpoints,
                    b"fixture-unanswered",
                    1,
                    b"invalid-json",
                )
            })
            .unwrap();
        let corrupt = snapshot(&store);
        let error = Node::new("iq").attr("id", "Q1").attr("type", "error");
        assert!(
            complete_refetch(&store, &account, &error, ConnectionEpoch(2))
                .await
                .is_err()
        );
        assert_eq!(snapshot(&store), corrupt);
        // Repair only this synthetic fixture to prove the original job can
        // still settle; production never repairs corrupt state automatically.
        store
            .write(|tx| tx.delete(Table::Checkpoints, b"fixture-unanswered"))
            .unwrap();
        let error = Node::new("iq").attr("id", "Q1").attr("type", "error");
        assert!(matches!(
            complete_refetch(&store, &account, &error, ConnectionEpoch(2))
                .await
                .unwrap(),
            Some(None)
        ));
        let settled = snapshot(&store);
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"fixture-answer", 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["reason"], "refetch_refused");
        assert!(
            complete_refetch(&store, &account, &error, ConnectionEpoch(2))
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(snapshot(&store), settled);
    }

    /// The APK's no-fetch branch: count 1/2 with a matching registration is
    /// answered once, under the saved session and the original message id;
    /// every refused case is recorded with its reason and sends nothing.
    #[tokio::test]
    async fn retry_original_chat_mismatch_never_reserves_or_reencrypts() {
        for explicit_recipient in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let store = ProtocolStore::open(OpenOptions {
                path: dir.path().join("chat.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [7; 32],
                ))),
                create: true,
            })
            .unwrap();
            let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
            let other = Jid::parse("15555550200@s.whatsapp.net").unwrap();
            let outbox = super::super::outgoing::outbox();
            store
                .write(|tx| {
                    let request = SendRequest {
                        client_request_id: ClientRequestId::new("chat-fixture")?,
                        chat: ChatId::from_jid(&peer)?,
                        content: MessageContent::Text {
                            body: "fixture".into(),
                            quoted: None,
                            style: Default::default(),
                        },
                    };
                    let id = outbox.enqueue(tx, &request)?.0.local_id;
                    let mut entry = outbox.get_mut(tx, id)?.unwrap();
                    entry.wire_id = Some(MessageId::new("W1")?);
                    entry.attempts.push(AttemptRecord {
                        index: 0,
                        prepared_at: Timestamp::now(),
                        epoch: Some(ConnectionEpoch(1)),
                        payload: Some(vec![1].into()),
                        re_encryption: false,
                    });
                    outbox.set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                    outbox.record_fact(
                        tx,
                        &mut entry,
                        DeliveryFact {
                            kind: DeliveryFactKind::ServerAccepted,
                            device: None,
                            observed_at: Timestamp::now(),
                            server_timestamp: None,
                            code: None,
                        },
                    )?;
                    keep_source_in(tx, "W1", &[1, 2, 3])?;
                    tx.put_json(
                        Table::SignalSessions,
                        format!("comparison-direct:{peer}").as_bytes(),
                        1,
                        &vec![2u8],
                    )?;
                    tx.put_json(
                        Table::PeerIdentities,
                        format!("comparison-direct:{peer}").as_bytes(),
                        1,
                        &vec![3u8],
                    )
                })
                .unwrap();
            let protected = || {
                store
                    .read(|tx| {
                        [Table::Outbox, Table::SignalSessions, Table::PeerIdentities]
                            .into_iter()
                            .map(|table| tx.scan(table, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = protected();
            let checkpoints = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            let node = if explicit_recipient {
                receipt(&peer, 1, 42).attr("recipient", other.clone())
            } else {
                receipt(&other, 1, 42).attr("participant", peer.clone())
            };
            assert!(
                answer(&store, &AccountRecord::default(), &node, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                reason(&store, 1, if explicit_recipient { &peer } else { &other }).as_deref(),
                Some("not_the_original_chat")
            );
            assert_eq!(protected(), before);
            let after = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            assert_eq!(after.len(), checkpoints.len() + 1);
            assert!(checkpoints.iter().all(|row| after.contains(row)));
        }
    }

    #[tokio::test]
    async fn answers_once_under_the_saved_session_and_records_every_refusal() {
        for pending_outcome in ["undelivered", "delivered", "aged"] {
            retry_workflow(pending_outcome).await;
        }
    }

    async fn retry_workflow(pending_outcome: &str) {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("retry.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap();
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let mut rng = rand::rng();
        let (local, remote, signed) = (
            IdentityKeyPair::generate(&mut rng),
            IdentityKeyPair::generate(&mut rng),
            KeyPair::generate(&mut rng),
        );
        let account = AccountRecord {
            generation: store.generation(),
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: local.public_key().public_key_bytes().try_into().unwrap(),
                private: *local.private_key().serialize(),
            })),
            ..Default::default()
        };
        store.write(|tx| account.save(tx)).unwrap();
        let signature = remote
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            None,
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *remote.identity_key(),
        )
        .unwrap();
        let first = crate::signal::direct::prepare(&account, &peer, &bundle, None, None, "hello")
            .await
            .unwrap();
        let original =
            super::super::messaging::direct_message(&peer, "W1", first.kind, first.ciphertext)
                .unwrap();
        let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&original)
            .unwrap();
        let outbox = super::super::outgoing::outbox();
        let key = format!("comparison-direct:{peer}");
        let source = crate::signal::direct::text_payload("hello").unwrap();
        store
            .write(|tx| {
                let request = SendRequest {
                    client_request_id: ClientRequestId::new("retry-test")?,
                    chat: ChatId::from_jid(&peer)?,
                    content: MessageContent::Text {
                        body: "hello".into(),
                        quoted: None,
                        style: Default::default(),
                    },
                };
                let id = outbox.enqueue(tx, &request)?.0.local_id;
                let mut entry = outbox.get_mut(tx, id)?.unwrap();
                entry.wire_id = Some(MessageId::new("W1")?);
                entry.attempts.push(AttemptRecord {
                    index: 0,
                    prepared_at: Timestamp::now(),
                    epoch: Some(ConnectionEpoch(1)),
                    payload: Some(wire.into()),
                    re_encryption: false,
                });
                outbox.set_progress(tx, &mut entry, AttemptProgress::InFlight)?;
                outbox.record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::ServerAccepted,
                        device: None,
                        observed_at: Timestamp::now(),
                        server_timestamp: None,
                        code: None,
                    },
                )?;
                keep_source_in(tx, "W1", &source)?;
                tx.put_json(Table::SignalSessions, key.as_bytes(), 1, &*first.session)?;
                tx.put_json(Table::PeerIdentities, key.as_bytes(), 1, &first.identity)
            })
            .unwrap();
        let retry_alias = Jid::parse("222222222222222@lid").unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("authenticated-lid:{retry_alias}").as_bytes(),
                    1,
                    &peer,
                )
            })
            .unwrap();
        let (store_ref, account_ref) = (&store, &account);
        let ask = |count: u32, registration: u32| {
            let mut node = receipt(&peer, count, registration);
            if count == 2 {
                node = node.attr(
                    "participant",
                    if registration == 43 {
                        "unknown.example"
                    } else {
                        "15555550100@unknown.example"
                    },
                );
            }
            if count == 1 {
                // A direct retry's optional participant must not redirect its
                // requesting device away from the authenticated root sender.
                node = node.attr("participant", "15555550101:3@s.whatsapp.net");
                // Native retry routing ignores a leading key-only participants
                // child; it must still reach the ordinary retry responder.
                let mut children = vec![Node::new("participants").attr("key", "IGNORED")];
                children.extend(node.child_nodes().iter().cloned());
                node = node.children(children);
            }
            async move {
                // The order listen uses: a retry passes validation, records
                // no delivery fact, and only then reaches the answer.
                super::super::receipts::validate(&node)?;
                assert_eq!(super::super::receipts::record(store_ref, &node)?, 0);
                let reserved =
                    answer_with_confirmation(store_ref, account_ref, &node, ConnectionEpoch(2))
                        .await?;
                Ok::<_, Error>(reserved.map(|answer| {
                    if answer.node.description == "iq" {
                        answer
                            .refetch
                            .as_ref()
                            .expect("query carries reservation snapshot")
                            .check_write_authority(store_ref)
                            .unwrap();
                        let query = answer
                            .node
                            .get_attr("id")
                            .and_then(|v| v.as_text())
                            .unwrap();
                        let job: Refetch = store_ref
                            .read(|tx| {
                                tx.get_json(Table::Checkpoints, refetch_key(query).as_bytes(), 1)
                            })
                            .unwrap()
                            .unwrap();
                        let state: serde_json::Value = store_ref
                            .read(|tx| {
                                tx.get_json(Table::Checkpoints, job.answer_key.as_bytes(), 1)
                            })
                            .unwrap()
                            .unwrap();
                        assert_eq!(state["status"], "fetching");
                        assert_eq!(state["query"], query);
                        assert_eq!(state["count"], job.count);
                        let ack = answer
                            .confirmation
                            .as_ref()
                            .expect("durable refetch gets a receipt ACK");
                        assert_eq!(
                            ack.get_attr("id").and_then(|v| v.as_text()),
                            node.get_attr("id").and_then(|v| v.as_text())
                        );
                        assert_eq!(
                            ack.get_attr("type").and_then(|v| v.as_text()),
                            Some("retry")
                        );
                        assert!(ack.get_attr("error").is_none());
                    }
                    answer.node
                }))
            }
        };

        // Malformed registration/version/count must not turn into a key
        // directory query or any retry bookkeeping (APK43.72 F10.A04).
        let all_retry_state = || {
            store
                .read(|tx| {
                    [
                        Table::Outbox,
                        Table::Checkpoints,
                        Table::SignalSessions,
                        Table::PeerIdentities,
                    ]
                    .into_iter()
                    .map(|table| tx.scan(table, 1, 10000))
                    .collect::<Result<Vec<_>>>()
                })
                .unwrap()
        };
        let before_invalid = all_retry_state();
        // A parseable retry is not authority to reserve work when its ACK
        // cannot be represented/written. Exercise both public entrypoints,
        // same-session resend and mismatched-registration refetch candidates.
        for (count, registration) in [(1, 42), (2, 43)] {
            let base = receipt(&peer, count, registration);
            let unencodable = base.clone().attr(
                "participant",
                Jid::parse("15555550101:300@s.whatsapp.net").unwrap(),
            );
            let candidate = super::super::messaging::handled_retry_confirmation(&unencodable)
                .expect("candidate exists before codec admission");
            assert!(
                Encoder::new(TokenDictionary::comparison_2_26_37_71())
                    .encode(&candidate)
                    .is_err()
            );
            for node in [
                base.clone().attr("participant", "Server@s.whatsapp.net"),
                base.clone().attr("participant", "123-456@temp"),
                // Typed objects are not classified through text factories.
                base.clone()
                    .attr("participant", Jid::parse("123@unknown.example").unwrap()),
                // A model object can construct a candidate but exceed the
                // selected writer's one-byte AD device field.
                unencodable,
                base.attr("edit", Jid::parse("12345@lid").unwrap()),
            ] {
                assert!(super::super::receipts::parse_retry(&node).is_some());
                assert!(
                    answer(&store, &account, &node, ConnectionEpoch(2))
                        .await
                        .unwrap()
                        .is_none()
                );
                assert!(
                    answer_with_confirmation(&store, &account, &node, ConnectionEpoch(2))
                        .await
                        .unwrap()
                        .is_none()
                );
                assert_eq!(all_retry_state(), before_invalid);
            }
        }
        for source in [
            "15555550100:100@s.whatsapp.net",
            "100001:128@lid",
            "01555550100@s.whatsapp.net",
        ] {
            let node = receipt(&peer, 1, 42)
                .attr("from", source)
                .attr("recipient", peer.clone())
                .attr("participant", peer.clone());
            assert!(
                answer_with_confirmation(&store, &account, &node, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(all_retry_state(), before_invalid);
        }
        // 1rU routing precedes ordinary retry parsing, even when a complete
        // resendable retry and matching saved session follow the selector.
        let ordinary = receipt(&peer, 1, 42);
        let mut aggregate_children = vec![Node::new("participants").attr("message_id", "W1")];
        aggregate_children.extend(ordinary.child_nodes().iter().cloned());
        for routed_elsewhere in [
            ordinary.clone().children(aggregate_children),
            ordinary.attr("category", "peer"),
        ] {
            assert!(
                answer_with_confirmation(&store, &account, &routed_elsewhere, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(all_retry_state(), before_invalid);
        }
        for registration in [
            None,
            Some(vec![]),
            Some(vec![1]),
            Some(vec![1, 2, 3]),
            Some(vec![1; 5]),
            Some(vec![1; 64]),
        ] {
            let mut children = vec![
                Node::new("retry")
                    .attr("id", "W1")
                    .attr("t", "1")
                    .attr("count", "1")
                    .attr("v", "1"),
            ];
            children.extend(registration.map(|bytes| Node::new("registration").bytes(bytes)));
            let node = receipt(&peer, 1, 42).children(children);
            assert!(
                answer(&store, &account, &node, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(all_retry_state(), before_invalid);
        }
        for retry in [
            Node::new("retry")
                .attr("id", "W1")
                .attr("t", "1")
                .attr("v", "2")
                .attr("count", "1"),
            Node::new("retry")
                .attr("id", "W1")
                .attr("t", "1")
                .attr("count", "2147483648"),
        ] {
            let node = receipt(&peer, 1, 42).children(vec![
                retry,
                Node::new("registration").bytes(42u32.to_be_bytes().to_vec()),
            ]);
            assert!(
                answer(&store, &account, &node, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(all_retry_state(), before_invalid);
        }
        // The nested ID identifies the failed message, independently of the
        // root receipt ID. Never resend W1 merely because the root names W1.
        for retry in [
            Node::new("retry").attr("id", "OTHER").attr("t", "1"),
            Node::new("retry").attr("t", "1"),
            Node::new("retry").attr("id", "W1"),
            Node::new("retry").attr("id", "W1").attr("t", "x"),
            Node::new("retry")
                .attr("id", "W1")
                .attr("t", "9223372036854775808"),
        ] {
            let node = receipt(&peer, 1, 42).children(vec![
                retry,
                Node::new("registration").bytes(42u32.to_be_bytes().to_vec()),
            ]);
            assert!(
                answer(&store, &account, &node, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(all_retry_state(), before_invalid);
        }
        // A receipt-carried bundle is a separate, unported policy. Neither
        // a matching nor changed registration may fall through to encryption
        // or a directory query, including empty/malformed keys children.
        let protected = || {
            store
                .read(|tx| {
                    [Table::Outbox, Table::SignalSessions, Table::PeerIdentities]
                        .into_iter()
                        .map(|table| tx.scan(table, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                })
                .unwrap()
        };
        let before_keys = protected();
        let checkpoints_before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
            .unwrap();
        let mut refused_snapshot = None;
        for registration in [42, 43] {
            for keys in [
                Node::new("keys"),
                Node::new("keys").bytes(vec![1]),
                Node::new("keys").children(vec![Node::new("identity").bytes(vec![2; 32])]),
            ] {
                let node = receipt(&peer, 1, registration);
                let mut children = node.child_nodes().to_vec();
                children.push(keys);
                let node = node.children(children);
                assert!(
                    answer(&store, &account, &node, ConnectionEpoch(2))
                        .await
                        .unwrap()
                        .is_none()
                );
                assert_eq!(
                    reason(&store, 1, &peer).as_deref(),
                    Some("receipt_keys_unsupported")
                );
                assert_eq!(protected(), before_keys);
                let checkpoints = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                    .unwrap();
                assert_eq!(checkpoints.len(), checkpoints_before.len() + 1);
                assert!(
                    checkpoints_before
                        .iter()
                        .all(|row| checkpoints.contains(row))
                );
                if let Some(previous) = &refused_snapshot {
                    assert_eq!(&checkpoints, previous);
                }
                refused_snapshot = Some(checkpoints);
            }
        }
        // A later supported receipt without keys still follows normal policy.
        // A cached drive account may remain valid for crypto after local
        // account replacement. The commit must still refuse, without ratchet
        // advancement, retry markers or another directory query.
        let mut changed_account = account.clone();
        changed_account.signal_registration_id = Some(32);
        store.write(|tx| changed_account.save(tx)).unwrap();
        let before_stale_account = all_retry_state();
        assert!(
            matches!(ask(1,42).await, Err(Error::InvalidInput { field:"retry", reason }) if reason == "local account changed before retry reservation")
        );
        assert_eq!(all_retry_state(), before_stale_account);
        store.write(|tx| account.save(tx)).unwrap(); // synthetic control only
        let default_count = receipt(&peer, 1, 42)
            .attr("id", "ROOT-RECEIPT")
            .attr("participant", "15555550101:3@s.whatsapp.net")
            .attr("recipient", retry_alias.clone())
            .children(vec![
                Node::new("retry").attr("id", "W1").attr("t", "1"),
                Node::new("registration").bytes(42u32.to_be_bytes().to_vec()),
            ]);
        super::super::receipts::validate(&default_count).unwrap();
        assert_eq!(
            super::super::receipts::record(&store, &default_count).unwrap(),
            0
        );
        let reserved =
            answer_with_confirmation(&store, &account, &default_count, ConnectionEpoch(2))
                .await
                .unwrap()
                .expect("default count answered as1");
        let confirmation = reserved
            .confirmation
            .expect("reserved message has a receipt ACK");
        assert_eq!(confirmation.description, "ack");
        assert_eq!(
            confirmation.get_attr("id").and_then(|v| v.as_text()),
            Some("ROOT-RECEIPT")
        );
        assert_eq!(
            confirmation.get_attr("type").and_then(|v| v.as_text()),
            Some("retry")
        );
        assert_eq!(
            confirmation.get_attr("recipient").and_then(|v| v.as_jid()),
            Some(&retry_alias)
        );
        assert!(confirmation.get_attr("error").is_none());
        let checkpoint: serde_json::Value = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("retry-answer:W1:{peer}:1").as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(checkpoint["status"], "reserved");
        assert!(
            answer_with_confirmation(&store, &account, &default_count, ConnectionEpoch(2))
                .await
                .unwrap()
                .is_none()
        );
        let resend = reserved.node;
        assert_eq!(resend.get_attr("id").and_then(|v| v.as_text()), Some("W1"));
        assert_eq!(
            resend.get_attr("type").and_then(|v| v.as_text()),
            Some("text")
        );
        let enc = resend.child("enc").unwrap();
        assert_eq!(
            enc.get_attr("type").and_then(|v| v.as_text()),
            Some("pkmsg")
        );
        assert_eq!(enc.get_attr("count").and_then(|v| v.as_text()), Some("1"));
        assert!(original.child("enc").unwrap().get_attr("count").is_none());
        assert_ne!(enc.content, original.child("enc").unwrap().content);
        let entry = store
            .read(|tx| outbox.by_wire_id(tx, &MessageId::new("W1")?))
            .unwrap()
            .unwrap();
        assert_eq!(entry.attempts.len(), 2);
        assert!(entry.attempts[1].re_encryption);

        // Once per (message, device, count), even across a restart of the loop.
        assert!(ask(1, 42).await.unwrap().is_none());
        assert!(ask(3, 42).await.unwrap().is_none());
        assert_eq!(
            reason(&store, 3, &peer).as_deref(),
            Some("new_session_needed_unsupported")
        );
        assert!(ask(5, 42).await.unwrap().is_none());

        // 151.A00: a changed registration asks the directory for a fresh
        // bundle, once; its reply completes the answer with a new session.
        store.write(|tx| changed_account.save(tx)).unwrap();
        let before_stale_query = all_retry_state();
        assert!(
            matches!(ask(2,43).await, Err(Error::InvalidInput { field:"retry", reason }) if reason == "local account changed before retry reservation")
        );
        assert_eq!(all_retry_state(), before_stale_query);
        store.write(|tx| account.save(tx)).unwrap(); // synthetic control only
        let query = ask(2, 43).await.unwrap().expect("key query");
        assert_eq!(
            query.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert!(ask(2, 43).await.unwrap().is_none(), "one fetch per count");
        let qid = query.get_attr("id").unwrap().as_text().unwrap().to_owned();
        // Non-replies leave the real reserved fetch available for the valid
        // bundle below, without changing original send/session history.
        let before_nonreply = store
            .read(|tx| {
                [
                    Table::Checkpoints,
                    Table::Outbox,
                    Table::SignalSessions,
                    Table::PeerIdentities,
                ]
                .into_iter()
                .map(|table| tx.scan(table, 1, 10000))
                .collect::<Result<Vec<_>>>()
            })
            .unwrap();
        for kind in [
            None,
            Some("get"),
            Some("set"),
            Some("RESULT"),
            Some("unexpected"),
        ] {
            let mut node = Node::new("iq").attr("id", qid.as_str());
            if let Some(kind) = kind {
                node = node.attr("type", kind);
            }
            assert!(
                complete_refetch(&store, &account, &node, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                store
                    .read(|tx| {
                        [
                            Table::Checkpoints,
                            Table::Outbox,
                            Table::SignalSessions,
                            Table::PeerIdentities,
                        ]
                        .into_iter()
                        .map(|table| tx.scan(table, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                    })
                    .unwrap(),
                before_nonreply
            );
        }
        let signed2 = KeyPair::generate(&mut rng);
        let signature2 = remote
            .private_key()
            .calculate_signature(&signed2.public_key.serialize(), &mut rng)
            .unwrap();
        let reply = Node::new("iq")
            .attr("type", "result")
            .attr("id", qid.as_str())
            .children(vec![Node::new("list").children(vec![
                Node::new("user").attr("jid", peer.clone()).children(vec![
                    Node::new("registration").bytes(43u32.to_be_bytes().to_vec()),
                    Node::new("type").bytes(vec![5]),
                    Node::new("identity").bytes(remote.public_key().public_key_bytes().to_vec()),
                    Node::new("skey").children(vec![
                        Node::new("id").bytes(vec![0, 0, 8]),
                        Node::new("value").bytes(signed2.public_key.public_key_bytes().to_vec()),
                        Node::new("signature").bytes(signature2.to_vec()),
                    ]),
                ]),
            ])]);
        let other = Node::new("iq")
            .attr("type", "result")
            .attr("id", "unrelated");
        assert!(
            complete_refetch(&store, &account, &other, ConnectionEpoch(2))
                .await
                .unwrap()
                .is_none()
        );
        store.write(|tx| changed_account.save(tx)).unwrap();
        let before_stale_completion = all_retry_state();
        assert!(
            matches!(complete_refetch(&store,&account,&reply,ConnectionEpoch(2)).await, Err(Error::InvalidInput {field:"retry",reason}) if reason == "local account changed before retry reservation")
        );
        assert_eq!(all_retry_state(), before_stale_completion);
        store.write(|tx| account.save(tx)).unwrap(); // synthetic control only
        if pending_outcome == "delivered" {
            store
                .write(|tx| {
                    let mut entry = outbox
                        .by_wire_id_in_write(tx, &MessageId::new("W1")?)?
                        .unwrap();
                    outbox.record_fact(
                        tx,
                        &mut entry,
                        DeliveryFact {
                            kind: DeliveryFactKind::Delivered,
                            device: Some(DeviceAddress::from_jid(&peer)?),
                            observed_at: Timestamp::now(),
                            server_timestamp: None,
                            code: None,
                        },
                    )
                })
                .unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        [Table::Outbox, Table::SignalSessions, Table::PeerIdentities]
                            .into_iter()
                            .map(|table| tx.scan(table, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            assert!(matches!(
                complete_refetch(&store, &account, &reply, ConnectionEpoch(2))
                    .await
                    .unwrap(),
                Some(None)
            ));
            assert_eq!(snapshot(), before);
            let outcome: serde_json::Value = store
                .read(|tx| {
                    tx.get_json(
                        Table::Checkpoints,
                        format!("retry-answer:W1:{peer}:2").as_bytes(),
                        1,
                    )
                })
                .unwrap()
                .unwrap();
            assert_eq!(outcome["reason"], "already_delivered");
            assert!(
                complete_refetch(&store, &account, &reply, ConnectionEpoch(2))
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(snapshot(), before);
            return;
        }
        if pending_outcome == "aged" {
            // Make the saved original stale after its query is reserved.
            // This is an offline fixture, not a production clock change.
            store
                .write(|tx| {
                    let mut entry = outbox
                        .by_wire_id_in_write(tx, &MessageId::new("W1")?)?
                        .unwrap();
                    entry.attempts[0].prepared_at =
                        Timestamp(Timestamp::now().as_millis() - MAX_AGE_MS - 1);
                    tx.put_json(Table::Outbox, &entry.local_id.0, 1, &entry)
                })
                .unwrap();
            let snapshot = || {
                store
                    .read(|tx| {
                        [Table::Outbox, Table::SignalSessions, Table::PeerIdentities]
                            .into_iter()
                            .map(|table| tx.scan(table, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            assert!(matches!(
                complete_refetch(&store, &account, &reply, ConnectionEpoch(2))
                    .await
                    .unwrap(),
                Some(None)
            ));
            assert_eq!(snapshot(), before);
            let outcome: serde_json::Value = store
                .read(|tx| {
                    tx.get_json(
                        Table::Checkpoints,
                        format!("retry-answer:W1:{peer}:2").as_bytes(),
                        1,
                    )
                })
                .unwrap()
                .unwrap();
            assert_eq!(outcome["reason"], "original_older_than_60_days");
            return;
        }
        let resend = complete_refetch(&store, &account, &reply, ConnectionEpoch(2))
            .await
            .unwrap()
            .unwrap()
            .expect("fresh-session resend");
        assert_eq!(resend.get_attr("id").and_then(|v| v.as_text()), Some("W1"));
        assert_eq!(
            resend
                .child("enc")
                .unwrap()
                .get_attr("type")
                .and_then(|v| v.as_text()),
            Some("pkmsg"),
            "a new session starts with a prekey message"
        );
        assert_eq!(
            resend
                .child("enc")
                .unwrap()
                .get_attr("count")
                .and_then(|v| v.as_text()),
            Some("2"),
            "refetched-session answers preserve the received count too"
        );
        let session: Vec<u8> = store
            .read(|tx| tx.get_json(Table::SignalSessions, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            crate::signal::direct::session_remote_registration(&session).unwrap(),
            43
        );
        assert!(
            complete_refetch(&store, &account, &reply, ConnectionEpoch(2))
                .await
                .unwrap()
                .is_none(),
            "settled once"
        );
        assert_eq!(
            reason(&store, 5, &peer).as_deref(),
            Some("count_out_of_range")
        );

        store
            .write(|tx| {
                let mut entry = outbox
                    .by_wire_id_in_write(tx, &MessageId::new("W1")?)?
                    .unwrap();
                outbox.record_fact(
                    tx,
                    &mut entry,
                    DeliveryFact {
                        kind: DeliveryFactKind::Delivered,
                        device: Some(DeviceAddress::from_jid(&peer)?),
                        observed_at: Timestamp::now(),
                        server_timestamp: None,
                        code: None,
                    },
                )
            })
            .unwrap();
        assert!(ask(2, 42).await.unwrap().is_none());
        assert_eq!(
            reason(&store, 2, &peer).as_deref(),
            Some("already_delivered")
        );
    }
}
