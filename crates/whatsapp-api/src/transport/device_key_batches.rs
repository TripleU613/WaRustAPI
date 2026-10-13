//! Durable bounded directory lookups for frozen group/status fanout devices.
//! APK 37.71 0y0.A00 1786..193a builds iq/get/encrypt/key/user@jid batches;
//! CQc.A00 verifies signed keys. Our 32-device width and no-retry policy are local.
use crate::{
    Error, Result,
    binary::{Decoder, Encoder, Node, TokenDictionary, unpack_frame_payload},
    model::{
        ids::ClientRequestId,
        jid::{Jid, JidKind, Server},
    },
    profile::Limits,
    store::{ProtocolStore, Table},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Local query width, not a discovered WhatsApp server limit.
pub const QUERY_WIDTH: usize = super::messaging::KEY_QUERY_MAX_DEVICES;
fn invalid(reason: &'static str) -> Error {
    Error::invalid("device_key_batch", reason)
}
fn key(id: &ClientRequestId) -> Vec<u8> {
    format!(
        "device-key-batch:{}",
        hex::encode(Sha256::digest(id.as_str().as_bytes()))
    )
    .into_bytes()
}
fn devices(input: &[Jid]) -> Result<Vec<Jid>> {
    if input.len() > Limits::default().max_children as usize {
        return Err(invalid("too many key-batch devices"));
    }
    let mut result = Vec::with_capacity(input.len());
    for device in input {
        if device.user.is_empty()
            || device.agent != 0
            || !matches!(device.server, Server::User | Server::Lid)
            || !matches!(device.kind, JidKind::Plain | JidKind::AgentDevice)
        {
            return Err(invalid(
                "key-batch target must be an ordinary PN/LID device",
            ));
        }
        let mut device = device.clone();
        device.kind = JidKind::Plain;
        result.push(device);
    }
    result.sort();
    if result.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("duplicate key-batch device"));
    }
    Ok(result)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Ready,
    OutcomeUnknown,
    Answered,
    Refused,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Reason {
    QueryRefused,
    DeviceRefused,
    InvalidReply,
    InvalidList,
    MissingDevice,
    DuplicateDevice,
    ForeignDevice,
    InvalidBundle,
    InvalidSignature,
    /// A verified PQ row whose metadata this client cannot admit yet.
    UnsupportedBundle,
}
#[derive(Serialize, Deserialize)]
struct Page {
    status: Status,
    query_id: Option<String>,
    reply: Option<Vec<u8>>,
    reason: Option<Reason>,
}
#[derive(Serialize, Deserialize)]
struct Batch {
    devices: Vec<Jid>,
    pages: Vec<Page>,
}
fn decode(wire: &[u8]) -> Result<Node> {
    let limits = Limits::default();
    let raw = unpack_frame_payload(wire, &limits)
        .map_err(|_| invalid("invalid saved key-batch reply"))?;
    Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits)
        .decode_node()
        .map_err(|_| invalid("invalid saved key-batch reply"))
}
impl Batch {
    fn validate(&self) -> Result<()> {
        if self.devices.is_empty()
            || devices(&self.devices)? != self.devices
            || self.pages.len() != self.devices.len().div_ceil(QUERY_WIDTH)
        {
            return Err(invalid("invalid saved key-batch plan"));
        }
        let mut query_ids = std::collections::HashSet::new();
        let mut unfinished = false;
        for page in &self.pages {
            if let Some(id) = &page.query_id {
                crate::model::ids::MessageId::new(id.clone())
                    .map_err(|_| invalid("invalid saved key-batch query id"))?;
                if !query_ids.insert(id) {
                    return Err(invalid("duplicate saved key-batch query id"));
                }
            }
            let consistent = match page.status {
                Status::Ready => {
                    page.query_id.is_none() && page.reply.is_none() && page.reason.is_none()
                }
                Status::OutcomeUnknown => {
                    page.query_id.is_some() && page.reply.is_none() && page.reason.is_none()
                }
                Status::Answered => {
                    page.query_id.is_some() && page.reply.is_some() && page.reason.is_none()
                }
                Status::Refused => {
                    page.query_id.is_some()
                        && page.reply.is_some()
                        && matches!(
                            page.reason,
                            Some(Reason::QueryRefused | Reason::DeviceRefused)
                        )
                }
                Status::Invalid => {
                    page.query_id.is_some()
                        && page.reply.is_some()
                        && page.reason.is_some()
                        && !matches!(
                            page.reason,
                            Some(Reason::QueryRefused | Reason::DeviceRefused)
                        )
                }
            };
            if !consistent || (unfinished && page.status != Status::Ready) {
                return Err(invalid("invalid saved key-batch outcome"));
            }
            if page.status != Status::Answered {
                unfinished = true;
            }
        }
        Ok(())
    }
    fn expected(&self, index: usize) -> &[Jid] {
        &self.devices[index * QUERY_WIDTH..((index + 1) * QUERY_WIDTH).min(self.devices.len())]
    }
    fn matches_devices(&self, input: &[Jid]) -> Result<()> {
        if self.devices != input {
            return Err(invalid(
                "saved key-batch targets changed; original outcomes preserved",
            ));
        }
        Ok(())
    }
}
fn load(tx: &crate::store::ReadTx<'_>, id: &ClientRequestId) -> Result<Option<Batch>> {
    let raw = tx.get(Table::Checkpoints, &key(id), 1)?;
    let batch: Option<Batch> = raw
        .map(|v| serde_json::from_slice(&v).map_err(|_| invalid("invalid saved key-batch state")))
        .transpose()?;
    if let Some(batch) = &batch {
        batch.validate()?;
    }
    Ok(batch)
}
fn legacy_in(tx: &crate::store::ReadTx<'_>, id: &ClientRequestId) -> Result<bool> {
    Ok(tx
        .get(
            Table::Checkpoints,
            format!("message:{id}:keys").as_bytes(),
            1,
        )?
        .is_some())
}
fn blocked(status: Status) -> Error {
    match status {
        Status::OutcomeUnknown => {
            invalid("key query outcome unknown; reservation preserved, no retry")
        }
        Status::Refused => invalid("saved key query refused; inspect group key-queries, no retry"),
        _ => invalid("saved key reply invalid or incomplete; inspect group key-queries, no retry"),
    }
}

/// Reserve one next query before socket write. Returns None only when no bundles
/// are required or all batches answered. Unknown/refused/invalid outcomes refuse;
/// a changed frozen target set never silently starts another query.
pub fn reserve_next(
    store: &ProtocolStore,
    id: &ClientRequestId,
    input: &[Jid],
) -> Result<Option<Node>> {
    let input = devices(input)?;
    store.write(|tx| {
        let raw=tx.get(Table::Checkpoints,&key(id),1)?;
        let mut batch:Batch=match raw {
            Some(raw)=>serde_json::from_slice(&raw).map_err(|_|invalid("invalid saved key-batch state"))?,
            None=>{
                if tx.get(Table::Checkpoints,format!("message:{id}:keys").as_bytes(),1)?.is_some() {
                    return Err(invalid("legacy key outcome lacks frozen batch provenance; original attempt preserved"));
                }
                if input.is_empty() { return Ok(None); }
                Batch { pages:(0..input.len().div_ceil(QUERY_WIDTH)).map(|_|Page { status:Status::Ready,query_id:None,reply:None,reason:None }).collect(), devices:input.clone() }
            },
        };
        batch.validate()?;batch.matches_devices(&input)?;
        let Some(index)=batch.pages.iter().position(|p|p.status!=Status::Answered) else { return Ok(None); };
        if batch.pages[index].status!=Status::Ready { return Err(blocked(batch.pages[index].status)); }
        let query_id=super::messaging::new_message_id()?;
        let node=super::messaging::key_request_many(batch.expected(index),&query_id)?;
        batch.pages[index].status=Status::OutcomeUnknown;batch.pages[index].query_id=Some(query_id);
        tx.put_json(Table::Checkpoints,&key(id),1,&batch)?;
        Ok(Some(node))
    })
}
fn classify(node: &Node, expected: &[Jid]) -> std::result::Result<(), Reason> {
    if node.get_attr("type").and_then(|v| v.as_text()) == Some("error")
        || node.child("error").is_some()
    {
        return Err(Reason::QueryRefused);
    }
    if node.get_attr("type").and_then(|v| v.as_text()) != Some("result") {
        return Err(Reason::InvalidReply);
    }
    let lists = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "list")
        .collect::<Vec<_>>();
    if lists.len() != 1 {
        return Err(Reason::InvalidList);
    }
    let mut seen = std::collections::BTreeSet::new();
    for user in lists[0].child_nodes() {
        if user.description != "user" {
            return Err(Reason::InvalidList);
        }
        let target = super::receipts::jid(user, "jid").ok_or(Reason::InvalidList)?;
        let target = devices(&[target])
            .map_err(|_| Reason::InvalidList)?
            .remove(0);
        if !expected.contains(&target) {
            return Err(Reason::ForeignDevice);
        }
        if !seen.insert(target.clone()) {
            return Err(Reason::DuplicateDevice);
        }
        if user.child("error").is_some() {
            return Err(Reason::DeviceRefused);
        }
    }
    if seen.len() != expected.len() {
        return Err(Reason::MissingDevice);
    }
    // Establish unambiguous complete device coverage before parsing a bundle;
    // otherwise the parser's duplicate guard hides the useful batch diagnosis.
    for target in expected {
        let user = crate::signal::direct::prekey_user_for(node, target)
            .map_err(|_| Reason::InvalidBundle)?;
        let bundle = crate::signal::direct::classic_bundle_from_user(user, target)
            .map_err(|_| Reason::InvalidBundle)?;
        crate::signal::direct::validate_bundle_signature(&bundle)
            .map_err(|_| Reason::InvalidSignature)?;
        // Bot.A03 reads a selected Kyber key into 0jc.A0L, and CQc.A00
        // 0068..00b4 verifies it before hybrid derivation (pq-bundle-boundary.md).
        // A verified, well-encoded key is usable; anything else blocks the batch.
        if user.child("pqkey").is_some() {
            match crate::signal::hybrid::bundle_for(node, target) {
                Ok(_) => {}
                Err(Error::InvalidInput { field, .. })
                    if field == crate::signal::hybrid::SIGNATURE_FIELD =>
                {
                    return Err(Reason::InvalidSignature);
                }
                Err(_) => return Err(Reason::InvalidBundle),
            }
            // Same local admission boundary as the direct outbox and identity
            // refresh: attestation-bearing or malformed identity_auth PQ rows.
            if !crate::signal::hybrid::admissible_pq_metadata(user) {
                return Err(Reason::UnsupportedBundle);
            }
        }
    }
    Ok(())
}
/// Commit the first authenticated matching query response, retaining its exact
/// wire privately. Returns false for unmatched/terminal IDs without mutation.
/// Call only after transport authentication; saving does not replace peer pins.
pub fn settle(store: &ProtocolStore, id: &ClientRequestId, node: &Node) -> Result<bool> {
    if node.description != "iq" {
        return Ok(false);
    }
    let Some(query_id) = node.get_attr("id").and_then(|v| v.as_text()) else {
        return Ok(false);
    };
    store.write(|tx| {
        let Some(raw) = tx.get(Table::Checkpoints, &key(id), 1)? else {
            return Ok(false);
        };
        let mut batch: Batch =
            serde_json::from_slice(&raw).map_err(|_| invalid("invalid saved key-batch state"))?;
        batch.validate()?;
        let Some(index) = batch.pages.iter().position(|p| {
            p.status == Status::OutcomeUnknown && p.query_id.as_deref() == Some(query_id)
        }) else {
            return Ok(false);
        };
        let result = classify(node, batch.expected(index));
        let page = &mut batch.pages[index];
        page.reason = result.err();
        page.status = match page.reason {
            None => Status::Answered,
            Some(Reason::QueryRefused | Reason::DeviceRefused) => Status::Refused,
            _ => Status::Invalid,
        };
        page.reply = Some(
            Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(node)
                .map_err(|_| invalid("could not preserve key-batch reply"))?,
        );
        tx.put_json(Table::Checkpoints, &key(id), 1, &batch)?;
        Ok(true)
    })
}
/// Assemble only complete validated pages into one private result for fanout
/// preparation. This is local composition, not another server response or query.
pub fn assembled(
    store: &ProtocolStore,
    id: &ClientRequestId,
    input: &[Jid],
) -> Result<Option<Node>> {
    let input = devices(input)?;
    store.read(|tx| {
        let Some(batch) = load(tx, id)? else {
            if legacy_in(tx, id)? {
                return Err(invalid(
                    "legacy key outcome lacks frozen batch provenance; original attempt preserved",
                ));
            }
            return if input.is_empty() {
                Ok(None)
            } else {
                Err(invalid("missing saved key-batch plan"))
            };
        };
        batch.matches_devices(&input)?;
        let mut users = Vec::with_capacity(batch.devices.len());
        for (index, page) in batch.pages.iter().enumerate() {
            if page.status != Status::Answered {
                return Err(blocked(page.status));
            }
            let reply = decode(
                page.reply
                    .as_deref()
                    .ok_or_else(|| invalid("missing saved key-batch reply"))?,
            )?;
            if reply.description != "iq"
                || reply.get_attr("id").and_then(|v| v.as_text()) != page.query_id.as_deref()
                || classify(&reply, batch.expected(index)).is_err()
            {
                return Err(invalid("invalid saved answered key-batch reply"));
            }
            users.extend_from_slice(
                reply
                    .child("list")
                    .ok_or_else(|| invalid("missing saved key-batch list"))?
                    .child_nodes(),
            );
        }
        Ok(Some(
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(users)]),
        ))
    })
}

/// Redacted directory-query progress, never message acceptance or delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    /// Frozen requested device count.
    pub devices: usize,
    /// Number of bounded queries in the frozen plan.
    pub batches: usize,
    /// Queries not yet reserved.
    pub unreserved: usize,
    /// Queries reserved before write without a saved response.
    pub outcome_unknown: usize,
    /// Queries with all expected valid signed bundles.
    pub answered: usize,
    /// Server/per-device refusals.
    pub refused: usize,
    /// Invalid, ambiguous, foreign or incomplete replies.
    pub invalid_replies: usize,
    /// Whether every page answered; not permission to send or trust changed pins.
    pub complete: bool,
    /// Fixed first failure classification; no server text or identifiers.
    pub blocking_reason: Option<String>,
}
/// Read-only existing batch inspection; None means no saved batch, not refusal.
pub fn summary(store: &ProtocolStore, id: &ClientRequestId) -> Result<Option<Summary>> {
    store.read(|tx| {
        let Some(batch) = load(tx, id)? else {
            if legacy_in(tx, id)? {
                return Err(invalid(
                    "legacy key outcome has no frozen plan; counts unknown, original preserved",
                ));
            }
            return Ok(None);
        };
        let count = |status| batch.pages.iter().filter(|p| p.status == status).count();
        let reason = batch.pages.iter().find_map(|p| p.reason).and_then(|r| {
            serde_json::to_value(r)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
        });
        Ok(Some(Summary {
            devices: batch.devices.len(),
            batches: batch.pages.len(),
            unreserved: count(Status::Ready),
            outcome_unknown: count(Status::OutcomeUnknown),
            answered: count(Status::Answered),
            refused: count(Status::Refused),
            invalid_replies: count(Status::Invalid),
            complete: count(Status::Answered) == batch.pages.len(),
            blocking_reason: reason,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use wacore_libsignal::protocol::{IdentityKeyPair, KeyPair};
    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.into(),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn id() -> ClientRequestId {
        ClientRequestId::new("original-send").unwrap()
    }
    fn target(n: usize) -> Jid {
        let server = if n.is_multiple_of(2) {
            "s.whatsapp.net"
        } else {
            "lid"
        };
        Jid::parse(&format!("1555555{:04}:{}@{server}", n, n % 4)).unwrap()
    }
    fn user_template() -> Node {
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        let signed = KeyPair::generate(&mut rand::rng());
        let signature = identity
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
            .unwrap();
        Node::new("user").children(vec![
            Node::new("registration").bytes(vec![0, 0, 0, 42]),
            Node::new("type").bytes(vec![5]),
            Node::new("identity").bytes(identity.public_key().public_key_bytes().to_vec()),
            Node::new("skey").children(vec![
                Node::new("id").bytes(vec![0, 0, 7]),
                Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                Node::new("signature").bytes(signature.to_vec()),
            ]),
        ])
    }
    /// A selected row whose identity also signs an ML-KEM-1024 key.
    fn pq_user_template(signature_valid: bool, encoding_valid: bool) -> Node {
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        let signed = KeyPair::generate(&mut rand::rng());
        let sign = |bytes: &[u8]| {
            identity
                .private_key()
                .calculate_signature(bytes, &mut rand::rng())
                .unwrap()
                .to_vec()
        };
        let (mut kem_public, _) = crate::signal::kem::generate_key_pair().unwrap();
        if !encoding_valid {
            // 12-bit coefficients of 0xfff exceed q = 3329.
            kem_public = vec![0xff; kem_public.len()];
        }
        let mut kem_signature = sign(&kem_public);
        if !signature_valid {
            kem_signature[0] ^= 1;
        }
        Node::new("user").children(vec![
            Node::new("registration").bytes(vec![0, 0, 0, 42]),
            Node::new("type").bytes(vec![5]),
            Node::new("identity").bytes(identity.public_key().public_key_bytes().to_vec()),
            Node::new("skey").children(vec![
                Node::new("id").bytes(vec![0, 0, 7]),
                Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                Node::new("signature").bytes(sign(&signed.public_key.serialize())),
            ]),
            Node::new("pqkey").children(vec![
                Node::new("id").bytes(vec![0, 0, 9]),
                Node::new("value").bytes(kem_public),
                Node::new("signature").bytes(kem_signature),
            ]),
        ])
    }
    fn reply(query: &Node, template: &Node) -> Node {
        Node::new("iq")
            .attr("id", query.get_attr("id").unwrap().as_text().unwrap())
            .attr("type", "result")
            .children(vec![
                Node::new("list").children(
                    query
                        .child("key")
                        .unwrap()
                        .child_nodes()
                        .iter()
                        .rev()
                        .map(|n| {
                            template
                                .clone()
                                .attr("jid", n.get_attr("jid").unwrap().clone())
                        })
                        .collect(),
                ),
            ])
    }
    fn state(store: &ProtocolStore) -> Vec<Vec<(Vec<u8>, Vec<u8>)>> {
        store
            .read(|tx| {
                [
                    Table::Checkpoints,
                    Table::PeerIdentities,
                    Table::SignalSessions,
                    Table::SenderKeys,
                    Table::Outbox,
                ]
                .into_iter()
                .map(|t| tx.scan(t, 1, 100))
                .collect()
            })
            .unwrap()
    }
    #[test]
    fn full_large_frozen_plan_batches_every_device_once_and_reuses_complete_replies_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large.redb");
        let store = open(&path);
        let id = id();
        let template = user_template();
        let targets = (0..130).map(target).collect::<Vec<_>>();
        let mut widths = vec![];
        let mut seen = std::collections::BTreeSet::new();
        while let Some(query) = reserve_next(&store, &id, &targets).unwrap() {
            let users = query.child("key").unwrap().child_nodes();
            widths.push(users.len());
            for user in users {
                assert!(
                    seen.insert(
                        devices(&[super::super::receipts::jid(user, "jid").unwrap()])
                            .unwrap()
                            .remove(0)
                    )
                );
            }
            let pending = state(&store);
            assert!(
                reserve_next(&store, &id, &targets)
                    .unwrap_err()
                    .to_string()
                    .contains("outcome unknown")
            );
            assert_eq!(pending, state(&store));
            assert!(
                !settle(
                    &store,
                    &id,
                    &reply(&query, &template).attr("id", "unrelated")
                )
                .unwrap()
            );
            assert_eq!(pending, state(&store));
            let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&reply(&query, &template))
                .unwrap();
            let incoming = decode(&wire).unwrap();
            assert!(settle(&store, &id, &incoming).unwrap());
            let settled = state(&store);
            assert!(!settle(&store, &id, &incoming).unwrap());
            assert_eq!(settled, state(&store));
        }
        assert_eq!(widths, vec![32, 32, 32, 32, 2]);
        assert_eq!(seen.len(), 130);
        let summary = summary(&store, &id).unwrap().unwrap();
        assert!(summary.complete);
        assert_eq!(summary.answered, 5);
        assert_eq!(summary.outcome_unknown, 0);
        let full = assembled(&store, &id, &targets).unwrap().unwrap();
        assert_eq!(full.child("list").unwrap().child_nodes().len(), 130);
        for target in &targets {
            crate::signal::direct::prekey_bundle_for(&full, target).unwrap();
        }
        let complete = state(&store);
        drop(store);
        let store = open(&path);
        let reordered = targets.iter().rev().cloned().collect::<Vec<_>>();
        assert!(reserve_next(&store, &id, &reordered).unwrap().is_none());
        assert_eq!(assembled(&store, &id, &reordered).unwrap(), Some(full));
        assert_eq!(complete, state(&store));
        let rendered = serde_json::to_string(&summary).unwrap();
        assert!(!rendered.contains("1555555") && !rendered.contains("signature"));
    }
    #[test]
    fn interrupted_query_and_changed_targets_remain_reserved_across_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("unknown.redb");
        let store = open(&path);
        let targets = vec![target(0), target(1)];
        let query = reserve_next(&store, &id(), &targets).unwrap().unwrap();
        let reserved = state(&store);
        drop(store);
        let store = open(&path);
        assert_eq!(summary(&store, &id()).unwrap().unwrap().outcome_unknown, 1);
        assert!(reserve_next(&store, &id(), &targets).is_err());
        assert!(reserve_next(&store, &id(), &[target(0)]).is_err());
        assert!(reserve_next(&store, &id(), &[]).is_err());
        assert_eq!(reserved, state(&store));
        assert!(settle(&store, &id(), &reply(&query, &user_template())).unwrap());
        assert!(reserve_next(&store, &id(), &targets).unwrap().is_none());
    }
    #[test]
    fn partial_ambiguous_foreign_refused_and_bad_signed_replies_block_later_batches_without_key_mutation()
     {
        for case in [
            "missing",
            "duplicate",
            "foreign",
            "user_error",
            "query_error",
            "pqkey",
            "type",
            "signature",
            "width",
            "lists",
            "namespace",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("failure.redb"));
            let targets = (0..33).map(target).collect::<Vec<_>>();
            let query = reserve_next(&store, &id(), &targets).unwrap().unwrap();
            let mut incoming = reply(&query, &user_template());
            let mut users = incoming.child("list").unwrap().child_nodes().to_vec();
            match case {
                "missing" => {
                    users.pop();
                }
                "duplicate" => users.push(users[0].clone()),
                "foreign" => {
                    users[0] = users[0].clone().attr("jid", target(999));
                }
                "namespace" => {
                    let original = super::super::receipts::jid(&users[0], "jid").unwrap();
                    let mut changed = original;
                    changed.server = if changed.server == Server::Lid {
                        Server::User
                    } else {
                        Server::Lid
                    };
                    users[0] = users[0].clone().attr("jid", changed);
                }
                "user_error" => {
                    let mut children = users[0].child_nodes().to_vec();
                    children.push(Node::new("error").attr("code", "480"));
                    users[0] = users[0].clone().children(children);
                }
                "type" => {
                    incoming = incoming.attr("type", Jid::parse("100001@lid").unwrap());
                }
                "query_error" => {
                    incoming = incoming.attr("type", "error");
                }
                "pqkey" => {
                    let mut children = users[0].child_nodes().to_vec();
                    children.push(Node::new("pqkey"));
                    users[0] = users[0].clone().children(children);
                }
                "signature" | "width" => {
                    let mut children = users[0].child_nodes().to_vec();
                    let skey = children
                        .iter_mut()
                        .find(|n| n.description == "skey")
                        .unwrap();
                    let mut keys = skey.child_nodes().to_vec();
                    *keys
                        .iter_mut()
                        .find(|n| n.description == "signature")
                        .unwrap() =
                        Node::new("signature").bytes(vec![0; if case == "width" { 3 } else { 64 }]);
                    *skey = skey.clone().children(keys);
                    users[0] = users[0].clone().children(children);
                }
                _ => {}
            }
            incoming = incoming.children(if case == "lists" {
                vec![Node::new("list").children(users), Node::new("list")]
            } else {
                vec![Node::new("list").children(users)]
            });
            let before_crypto = state(&store).into_iter().skip(1).collect::<Vec<_>>();
            let head = store.journal_head().unwrap();
            assert!(settle(&store, &id(), &incoming).unwrap());
            let progress = summary(&store, &id()).unwrap().unwrap();
            assert!(!progress.complete);
            assert_eq!(progress.unreserved, 1);
            let expected = match case {
                "missing" => "missing_device",
                "duplicate" => "duplicate_device",
                "foreign" | "namespace" => "foreign_device",
                "user_error" => "device_refused",
                "query_error" => "query_refused",
                "type" => "invalid_reply",
                "signature" => "invalid_signature",
                "width" | "pqkey" => "invalid_bundle",
                _ => "invalid_list",
            };
            assert_eq!(progress.blocking_reason.as_deref(), Some(expected));
            let saved = state(&store);
            let error = reserve_next(&store, &id(), &targets)
                .unwrap_err()
                .to_string();
            assert!(!error.contains("1555555") && !error.contains("480"));
            assert!(assembled(&store, &id(), &targets).is_err());
            assert!(!settle(&store, &id(), &reply(&query, &user_template())).unwrap());
            assert_eq!(saved, state(&store));
            assert_eq!(
                before_crypto,
                state(&store).into_iter().skip(1).collect::<Vec<_>>()
            );
            assert_eq!(head, store.journal_head().unwrap());
        }
    }
    #[test]
    fn verified_pq_rows_answer_and_invalid_pq_rows_block_without_key_mutation() {
        for (case, expected) in [
            ("valid", None),
            ("signature", Some("invalid_signature")),
            ("encoding", Some("invalid_bundle")),
            ("attestation", Some("unsupported_bundle")),
            ("identity_auth", Some("unsupported_bundle")),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("pq.redb"));
            let targets = vec![target(0), target(1)];
            let query = reserve_next(&store, &id(), &targets).unwrap().unwrap();
            let mut template = pq_user_template(case != "signature", case != "encoding");
            if matches!(case, "attestation" | "identity_auth") {
                // Shared local PQ admission boundary: attestation-bearing rows,
                // and identity_auth lacking Bot.A03's signature/certs shape.
                let mut children = template.child_nodes().to_vec();
                children.push(Node::new(if case == "attestation" {
                    "non_e2ee_attestation"
                } else {
                    "identity_auth"
                }));
                template = template.children(children);
            }
            let incoming = reply(&query, &template);
            let before_crypto = state(&store).into_iter().skip(1).collect::<Vec<_>>();
            let head = store.journal_head().unwrap();
            assert!(settle(&store, &id(), &incoming).unwrap());
            let progress = summary(&store, &id()).unwrap().unwrap();
            assert_eq!(progress.blocking_reason.as_deref(), expected, "{case}");
            assert_eq!(progress.complete, expected.is_none());
            if expected.is_none() {
                let full = assembled(&store, &id(), &targets).unwrap().unwrap();
                for target in &targets {
                    crate::signal::hybrid::bundle_for(&full, target).unwrap();
                    // The consumable classic parser still never drops the KEM key.
                    assert!(crate::signal::direct::prekey_bundle_for(&full, target).is_err());
                }
            } else {
                assert!(assembled(&store, &id(), &targets).is_err());
            }
            assert_eq!(
                before_crypto,
                state(&store).into_iter().skip(1).collect::<Vec<_>>()
            );
            assert_eq!(head, store.journal_head().unwrap());
        }
    }
    #[test]
    fn malformed_inputs_and_unattributed_legacy_outcomes_do_not_create_new_queries() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("guards.redb"));
        let before = state(&store);
        assert!(reserve_next(&store, &id(), &[]).unwrap().is_none());
        assert!(summary(&store, &id()).unwrap().is_none());
        assert!(assembled(&store, &id(), &[]).unwrap().is_none());
        let mut hosted = target(0);
        hosted.kind = JidKind::Interop;
        let mut agent = target(0);
        agent.agent = 1;
        agent.kind = JidKind::AgentDevice;
        for targets in [
            vec![target(0), target(0)],
            vec![Jid::parse("status@broadcast").unwrap()],
            vec![agent],
            vec![hosted],
            (0..4097).map(target).collect(),
        ] {
            assert!(reserve_next(&store, &id(), &targets).is_err());
            assert_eq!(before, state(&store));
        }
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"message:original-send:keys",
                    1,
                    &"private-old-result",
                )
            })
            .unwrap();
        let legacy = state(&store);
        let error = reserve_next(&store, &id(), &[target(0)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("legacy key outcome"));
        assert!(!error.contains("private-old-result"));
        assert_eq!(legacy, state(&store));
        assert!(
            summary(&store, &id())
                .unwrap_err()
                .to_string()
                .contains("counts unknown")
        );
        assert!(assembled(&store, &id(), &[]).is_err());
        assert_eq!(legacy, state(&store));
    }
    #[test]
    fn corrupted_saved_results_remain_intact_with_redacted_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("atomic.redb"));
        let targets = vec![target(0)];
        let query = reserve_next(&store, &id(), &targets).unwrap().unwrap();
        assert!(settle(&store, &id(), &reply(&query, &user_template())).unwrap());
        store
            .write(|tx| {
                let mut batch: Batch = tx.get_json(Table::Checkpoints, &key(&id()), 1)?.unwrap();
                batch.pages[0].reply = Some(vec![255]);
                tx.put_json(Table::Checkpoints, &key(&id()), 1, &batch)
            })
            .unwrap();
        let bad = state(&store);
        let error = assembled(&store, &id(), &targets).unwrap_err().to_string();
        assert!(error.contains("invalid saved"));
        assert_eq!(bad, state(&store));
        store
            .write(|tx| {
                let mut batch: Batch = tx.get_json(Table::Checkpoints, &key(&id()), 1)?.unwrap();
                batch.pages[0].query_id = Some("private query invalid".into());
                tx.put_json(Table::Checkpoints, &key(&id()), 1, &batch)
            })
            .unwrap();
        let bad = state(&store);
        let error = summary(&store, &id()).unwrap_err().to_string();
        assert!(!error.contains("private query invalid"));
        assert_eq!(bad, state(&store));
    }
}
