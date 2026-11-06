//! One-time prekey replenishment, traced from comparison APK **2.26.37.71**
//! (SHA-256 `1755111a…7d5c`; class names differ from the 2.26.37.70 profile).
//!
//! * Generation `LX/0ko.A06` `0008..0056`: with fewer than 812 (`0x32c`) unsent
//!   keys, generate `min(812 - unsent, 50)` new ones; ids come from
//!   `next_prekey_id` as `((next + i) % 0xFFFFFE) + 1` (`00c0..00da`).
//! * Batch `LX/0kf.A01`: `SELECT prekey_id, record FROM prekeys WHERE
//!   sent_to_server = 0 AND direct_distribution = 0 LIMIT 812`
//!   (`0x32c` passed by `LX/0jc.A1D`).
//! * Upload IQ `LX/0y0.A00` case 85, `12d0..1690`: `iq id/xmlns=encrypt/
//!   type=set/to=server` with children `identity` (raw 32 bytes),
//!   `registration` (4-byte big endian, `LX/1Lz.A03`), `type` (`[0x05]`),
//!   `list` of `key{id,value}` (3-byte big-endian id `LX/1Lz.A04`, raw public
//!   key with the type byte stripped, `LX/0km.A02` `0070..0082`), `skey{id,
//!   value,signature}`, then optional `pq_list` (of `key{id,value,signature}`)
//!   and `pq_last_resort_key{id,value,signature}` when Kyber keys exist
//!   (`153c..161c`). `LX/13c.A03` `00d6..00e4` logs "skipping kyber prekeys
//!   upload" without them, so the post-quantum children are optional; this
//!   client conditionally generates and sends them when enabled. The conditional `op`
//!   element (`13f0..1474`, runtime flag `0x5bf2` plus coexistence reason and
//!   token) is not sent either.
//! * Trigger `LX/14z.A09` `001a..008a`: `notification type=encrypt` with a
//!   first `count` child (optional signed value and sibling `pq_count`) queues
//!   `LX/14z.A02` `0154..01d4`, which generates when nothing is unsent and uploads
//!   legacy keys (`LX/13c.A0U`); a `pq_count`-only notification takes no action
//!   (`01a0`). Startup `LX/DNq.run` `011a..0160` uploads existing unsent keys
//!   or generates when none exist.
//! * Result `LX/Bor.A03`: `list`/`pq_list` children carry `count_low="true"`
//!   when the server still wants more, which schedules another upload
//!   (`LX/1Dz.A0c`, `needsMoreLegacyPrekeys`). Errors go through
//!   `LX/13c.A02` backoff; this client records a refusal and does not retry.
//!
//! Current-build 43.72 success followup (14o77/Fcj3/Fc8(7)/A0V): after
//! uploaded-LR migration with gate21198, reserve ADD for the independent demand
//! flags. Defaults: legacy inventory812/batch200, ordinary KEM inventory200/
//! batch10. Effective MobileConfig and native cursor migration remain unported;
//! existing local cursor conventions and uncertain requests are preserved.
//!
//! Local policy: a batch is reserved durably before its socket write; keys in
//! an uncertain batch are never re-sent. Startup generates first when nothing
//! is unsent and then uploads in the same session (the APK does these on
//! separate runs). Follow-up uploads after `count_low` are capped per session.
/// Read-only digest grammar and local material inspection; never resets keys.
pub mod digest;

use crate::{
    Error, Result,
    binary::{Encoder, Node, TokenDictionary},
    model::{jid::Jid, time::Timestamp},
    session::AccountRecord,
    store::{ProtocolStore, ReadTx, Table, WriteTx},
};

/// Read access shared by read and write transactions.
trait Lookup {
    fn scan_after(
        &self,
        table: Table,
        version: u16,
        after: Option<&[u8]>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        table: Table,
        key: &[u8],
        version: u16,
    ) -> Result<Option<T>>;
}
impl Lookup for ReadTx<'_> {
    fn scan_after(
        &self,
        table: Table,
        version: u16,
        after: Option<&[u8]>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        ReadTx::scan_after(self, table, version, after, limit)
    }
    fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        table: Table,
        key: &[u8],
        version: u16,
    ) -> Result<Option<T>> {
        ReadTx::get_json(self, table, key, version)
    }
}
impl Lookup for WriteTx<'_> {
    fn scan_after(
        &self,
        table: Table,
        version: u16,
        after: Option<&[u8]>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        WriteTx::scan_after(self, table, version, after, limit)
    }
    fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        table: Table,
        key: &[u8],
        version: u16,
    ) -> Result<Option<T>> {
        WriteTx::get_json(self, table, key, version)
    }
}
use serde::{Deserialize, Serialize};
use wacore_libsignal::{
    core::curve::KeyPair,
    protocol::{GenericSignedPreKey, PreKeyRecord, SignedPreKeyRecord},
};

/// Stop generating once this many keys are unsent (`0x32c`).
pub const UNSENT_TARGET: usize = 812;
/// Keys generated per call (`0x32`).
pub const GENERATE_PER_CALL: usize = 50;
/// Follow-up uploads accepted per session after `count_low`.
pub const MAX_ROUNDS_PER_SESSION: u32 = 20;
const ID_MODULUS: u32 = 0x00FF_FFFE;
const NEXT_ID_KEY: &[u8] = b"comparison-next-prekey-id";
const RECORD_PREFIX: &str = "comparison-prekey:";
const META_PREFIX: &str = "comparison-prekey-meta:";
const BATCH_PREFIX: &str = "prekey-upload:";

fn invalid(reason: &'static str) -> Error {
    Error::invalid("prekeys", reason)
}
fn record_key(id: u32) -> String {
    format!("{RECORD_PREFIX}{id}")
}
fn meta_key(id: u32) -> String {
    format!("{META_PREFIX}{id}")
}
fn batch_key(iq_id: &str) -> String {
    format!("{BATCH_PREFIX}{iq_id}")
}
fn request_key(iq_id: &str) -> String {
    // Separate prefix: batch summary scans must never decode wire data as Batch.
    format!("prekey-upload-request:v1:{iq_id}")
}

/// Upload state of one generated key. Missing metadata means unsent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    /// Generated, never included in a reserved batch (APK `sent_to_server = 0`).
    Unsent,
    /// Included in a batch whose outcome is unknown; never re-sent.
    OutcomeUnknown,
    /// Server acknowledged the batch (APK `sent_to_server = 1`).
    Uploaded,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Meta {
    status: KeyStatus,
    batch: Option<String>,
}
/// One reserved upload batch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Batch {
    /// Key ids in wire order.
    pub ids: Vec<u32>,
    /// `outcome_unknown`, `server_acknowledged` or `refused`.
    pub status: String,
    /// Canonical signed 32-bit server code for a refusal (invalid/missing -> 0).
    /// Historical batches may retain the earlier truncated raw string.
    pub code: Option<String>,
    /// Server still wants more legacy keys (`list/@count_low`).
    pub count_low: Option<bool>,
    /// Independent server PQ demand; None for historical/unsettled batches.
    #[serde(default)]
    pub pq_count_low: Option<bool>,
    /// Reservation time.
    pub reserved_at: Timestamp,
    /// Kyber prekey ids carried in the same IQ (`pq_list`/`pq_last_resort_key`).
    #[serde(default)]
    pub kem_ids: Vec<u32>,
}
/// Redacted local summary for operators.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// Keys never batched.
    pub unsent: usize,
    /// Keys in batches without a known outcome.
    pub uncertain: usize,
    /// Keys the server acknowledged.
    pub uploaded: usize,
    /// Reserved batches by status.
    pub batches: std::collections::BTreeMap<String, usize>,
    /// Local ML-KEM-1024 prekey rows the inbox can use for incoming hybrid
    /// bootstrap. Enabled SET uploads can generate these locally; current
    /// native ADD/replenishment strategy parity remains incomplete.
    pub kem_local: usize,
    /// Of those, native `last_resort_key` rows retained after successful use.
    pub kem_last_resort: usize,
    /// Historical acknowledged batches reporting PQ count_low; not current demand.
    pub pq_low_acknowledged_batches: usize,
    /// Saved refusal holds upload until the native retry policy is implemented.
    pub upload_held_after_refusal: bool,
}

fn scan_prefix<T: serde::de::DeserializeOwned>(
    tx: &impl Lookup,
    table: Table,
    prefix: &str,
) -> Result<Vec<(String, T)>> {
    let mut out = Vec::new();
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(table, 1, cursor.as_deref(), 512)?;
        let done = rows.len() < 512;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows {
            if let Ok(name) = std::str::from_utf8(&key)
                && name.starts_with(prefix)
                && let Some(value) = tx.get_json::<T>(table, &key, 1)?
            {
                out.push((name.to_owned(), value));
            }
        }
        if done {
            return Ok(out);
        }
    }
}
fn meta_map(tx: &impl Lookup) -> Result<std::collections::BTreeMap<u32, Meta>> {
    Ok(scan_prefix::<Meta>(tx, Table::Checkpoints, META_PREFIX)?
        .into_iter()
        .filter_map(|(key, meta)| key[META_PREFIX.len()..].parse().ok().map(|id| (id, meta)))
        .collect())
}
fn record_ids(tx: &impl Lookup) -> Result<Vec<u32>> {
    let mut ids: Vec<u32> = scan_prefix::<Vec<u8>>(tx, Table::Prekeys, RECORD_PREFIX)?
        .into_iter()
        .filter_map(|(key, _)| key[RECORD_PREFIX.len()..].parse().ok())
        .collect();
    ids.sort_unstable();
    Ok(ids)
}
fn unsent_in(tx: &impl Lookup) -> Result<Vec<u32>> {
    let metas = meta_map(tx)?;
    Ok(record_ids(tx)?
        .into_iter()
        .filter(|id| metas.get(id).is_none_or(|m| m.status == KeyStatus::Unsent))
        .collect())
}

/// Ids of generated keys not yet included in any reserved batch, ascending.
pub fn unsent_ids(store: &ProtocolStore) -> Result<Vec<u32>> {
    store.read(|tx| unsent_in(tx))
}

/// Generate new keys the way `LX/0ko.A06` does. Returns how many were added.
/// Never overwrites an existing record: an id collision after wrap-around fails.
pub fn generate(store: &ProtocolStore) -> Result<usize> {
    store.write(generate_in)
}

fn generate_in(tx: &mut WriteTx<'_>) -> Result<usize> {
    let unsent = unsent_in(&*tx)?.len();
    if unsent >= UNSENT_TARGET {
        return Ok(0);
    }
    let count = (UNSENT_TARGET - unsent).min(GENERATE_PER_CALL);
    let next: u32 = tx
        .get_json(Table::Checkpoints, NEXT_ID_KEY, 1)?
        .unwrap_or(0);
    let mut rng = rand::rng();
    for i in 0..count as u32 {
        let id = (next.wrapping_add(i) % ID_MODULUS) + 1;
        if tx
            .get(Table::Prekeys, record_key(id).as_bytes(), 1)?
            .is_some()
        {
            return Err(invalid("prekey id collision; refusing to overwrite"));
        }
        let pair = KeyPair::generate(&mut rng);
        let record = PreKeyRecord::new(id.into(), &pair)
            .serialize()
            .map_err(|_| invalid("cannot serialize generated prekey"))?;
        tx.put_json(Table::Prekeys, record_key(id).as_bytes(), 1, &record)?;
        tx.put_json(
            Table::Checkpoints,
            meta_key(id).as_bytes(),
            1,
            &Meta {
                status: KeyStatus::Unsent,
                batch: None,
            },
        )?;
    }
    tx.put_json(
        Table::Checkpoints,
        NEXT_ID_KEY,
        1,
        &(next.wrapping_add(count as u32) % ID_MODULUS),
    )?;
    Ok(count)
}

// 43.72 1nr.A04 emits only the low 24 bits. Local callers must not
// silently alias wider IDs, even though the native encoder itself truncates.
fn check_wire_id(id: u32) -> Result<()> {
    if id > 0x00ff_ffff {
        return Err(invalid("prekey id does not fit three bytes"));
    }
    Ok(())
}

fn legacy_record_in(tx: &impl Lookup, id: u32) -> Result<Option<PreKeyRecord>> {
    check_wire_id(id)?;
    let Some(raw) = tx.get_json::<Vec<u8>>(Table::Prekeys, record_key(id).as_bytes(), 1)? else {
        return Ok(None);
    };
    let record = PreKeyRecord::deserialize(&raw).map_err(|_| invalid("invalid prekey record"))?;
    if u32::from(
        record
            .id()
            .map_err(|_| invalid("invalid prekey record id"))?,
    ) != id
    {
        return Err(invalid("prekey record id does not match store key"));
    }
    Ok(Some(record))
}

fn be3(id: u32) -> Vec<u8> {
    vec![(id >> 16) as u8, (id >> 8) as u8, id as u8]
}
fn key_node(id: u32, public: &[u8], signature: Option<&[u8]>) -> Node {
    let mut children = vec![
        Node::new("id").bytes(be3(id)),
        Node::new("value").bytes(public.to_vec()),
    ];
    if let Some(signature) = signature {
        children.push(Node::new("signature").bytes(signature.to_vec()));
    }
    Node::new("key").children(children)
}

/// The post-quantum children of the upload IQ, appended after `skey`
/// (2.26.37.71 `LX/0y0.A00` case 85, `153c..161c`, confirmed static):
/// `pq_list` of `key{id,value,signature}` for one-time KEM keys (only when
/// there are any) and a bare `pq_last_resort_key{id,value,signature}` (only
/// when a last-resort key exists). The element *contents* follow the receive
/// side's confirmed rules, since peers parse and verify our `pqkey` with them
/// (`Bot.A03`, `KEMPublicKey`, `CQc.A00` `0068..00b4`): `value` is the raw
/// 1568-byte ML-KEM-1024 encapsulation key with no type prefix, `signature` is
/// the identity key's signature over exactly those bytes. The `id` tag is
/// confirmed (`v2` = "id", `00ec`). **Inferred:** the server relays the bytes
/// unchanged, as it does ordinary keys, and the `id` value is three bytes
/// big-endian like the ordinary and signed keys (the parser's local rule).
/// Docs: kem-prekey-inventory.md.
///
/// Called by [`begin_upload`] inside its reservation transaction; identity
/// validation also applies when no PQ rows are selected.
pub(crate) fn kem_upload_children(
    identity: &crate::session::SignalIdentityKey,
    rows: &[super::kem_prekeys::StoredKemPrekey],
) -> Result<Vec<Node>> {
    use wacore_libsignal::protocol::{PrivateKey, PublicKey};
    let private =
        PrivateKey::deserialize(&identity.0.private).map_err(|_| invalid("invalid identity"))?;
    let public = PublicKey::from_djb_public_key_bytes(&identity.0.public)
        .map_err(|_| invalid("invalid identity"))?;
    if private
        .public_key()
        .map_err(|_| invalid("invalid identity"))?
        != public
    {
        return Err(invalid("identity key pair mismatch"));
    }
    let entry = |row: &super::kem_prekeys::StoredKemPrekey| -> Result<Vec<Node>> {
        if row.id > 0x00ff_ffff {
            return Err(invalid("KEM prekey id does not fit three bytes"));
        }
        crate::signal::kem::public_key(&row.public)?;
        let signature = if let Some(signature) = &row.signature {
            if signature.len() != 64 || !public.verify_signature(&row.public, signature) {
                return Err(invalid("invalid saved KEM publication signature"));
            }
            signature.clone()
        } else {
            private
                .calculate_signature(&row.public, &mut rand::rng())
                .map_err(|_| invalid("cannot sign KEM prekey"))?
                .to_vec()
        };
        Ok(vec![
            Node::new("id").bytes(be3(row.id)),
            Node::new("value").bytes(row.public.clone()),
            Node::new("signature").bytes(signature.to_vec()),
        ])
    };
    let mut one_time = Vec::new();
    let mut last_resort = None;
    for row in rows {
        if row.last_resort {
            if last_resort.is_some() {
                return Err(invalid("more than one last-resort KEM prekey"));
            }
            last_resort = Some(Node::new("pq_last_resort_key").children(entry(row)?));
        } else {
            one_time.push(Node::new("key").children(entry(row)?));
        }
    }
    let mut children = Vec::new();
    if !one_time.is_empty() {
        children.push(Node::new("pq_list").children(one_time));
    }
    children.extend(last_resort);
    Ok(children)
}

/// A reserved upload awaiting the server's `iq` reply for this connection.
#[derive(Debug)]
pub struct PendingUpload {
    iq_id: String,
    ids: Vec<u32>,
    kem_ids: Vec<u32>,
    /// After this instant the outcome is treated as unknown for this session.
    pub deadline: tokio::time::Instant,
}
/// What the server said about a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UploadOutcome {
    /// `iq type=result`.
    pub accepted: bool,
    /// `list/@count_low="true"`: the server still wants more legacy keys.
    pub count_low: bool,
    /// Independent pq_list count-low flag; not a legacy upload trigger.
    pub pq_count_low: bool,
}

/// Reserve up to 812 unsent keys and build the upload IQ. `signed` is the
/// serialized registered signed prekey record. Returns `None` with nothing
/// unsent; a `Some` reservation is durable before the caller writes anything.
pub fn begin_upload(
    store: &ProtocolStore,
    account: &AccountRecord,
    signed: &[u8],
) -> Result<Option<(PendingUpload, Node)>> {
    begin_upload_selected(store, account, signed, None, false)
}

/// Follow-up demand from the legacy success callback (43.72 Fcj3/Fc8(7)
/// ->13z.A0V). Non-migrated accounts do not enter this ADD path.
pub(crate) fn begin_followup_upload(
    store: &ProtocolStore,
    account: &AccountRecord,
    signed: &[u8],
    legacy: bool,
    pq: bool,
) -> Result<Option<(PendingUpload, Node)>> {
    begin_upload_selected(store, account, signed, Some((legacy, pq)), false)
}

/// Count-notification path with the declared EMv0x70d7 defaultfalse:
/// migrated legacy demand uses ADD; non-migrated legacy demand uses SET.
/// PQ-only notifications remain inert until effective MobileConfig is ported.
pub(crate) fn begin_notification_upload(
    store: &ProtocolStore,
    account: &AccountRecord,
    signed: &[u8],
    legacy: bool,
) -> Result<Option<(PendingUpload, Node)>> {
    if !legacy {
        return Ok(None);
    }
    begin_upload_selected(store, account, signed, Some((true, false)), true)
}

fn begin_upload_selected(
    store: &ProtocolStore,
    account: &AccountRecord,
    signed: &[u8],
    demand: Option<(bool, bool)>,
    notification: bool,
) -> Result<Option<(PendingUpload, Node)>> {
    let identity = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("account has no Signal identity"))?;
    let registration = account
        .signal_registration_id
        .ok_or_else(|| invalid("account has no registration id"))?;
    let signed = SignedPreKeyRecord::deserialize(signed)
        .map_err(|_| invalid("invalid registered signed prekey"))?;
    let signed_id = u32::from(signed.id().map_err(|_| invalid("signed prekey id"))?);
    check_wire_id(signed_id)?;
    let signed_public = signed
        .public_key()
        .map_err(|_| invalid("signed prekey public key"))?;
    let signed_signature = signed
        .signature()
        .map_err(|_| invalid("signed prekey signature"))?;
    let identity_public =
        wacore_libsignal::protocol::PublicKey::from_djb_public_key_bytes(&identity.0.public)
            .map_err(|_| invalid("invalid publication identity"))?;
    if !identity_public.verify_signature(&signed_public.serialize(), &signed_signature) {
        return Err(invalid("signed prekey does not match publication identity"));
    }
    let iq_id = super::messaging::new_message_id()?;
    // 07t.A0R 0196..01a2: property 21201 defaults to 10; returned values are
    // not applied (MobileConfig unported), so the default is used.
    let pq_batch = 10;
    let reserved =
        store.write(|tx| {
            // The actor's snapshot can outlive a local account update. Bind
            // publication authority before selecting keys or freezing bytes.
            let current = AccountRecord::load_in_write(tx)?
                .ok_or_else(|| invalid("publication account unavailable"))?;
            let same_identity = current.signal_identity.as_ref().is_some_and(|key|
                key.0.public == identity.0.public && key.0.private == identity.0.private);
            if !account.is_registered() || !current.is_registered()
                || current.generation != tx.generation()
                || current.generation != account.generation
                || current.signal_registration_id != Some(registration)
                || current.own_jid != account.own_jid
                || current.phone != account.phone
                || !same_identity
            {
                return Err(invalid("publication account changed"));
            }
            let pq_enabled = super::abprops::pq_keys_upload_enabled_in_write(tx)?;
            // Refusal retry/backoff/regeneration policy is not ported. Returning
            // keys to Unsent is inventory state, not authority to reserve a new
            // IQ on the next notification or reconnect (43.72 13z.A02/14o.BLn).
            if refused_upload_in(&*tx)? {
                tracing::warn!("prekey upload held: saved refusal; retry policy unimplemented");
                return Ok(None);
            }
            let migrated = pq_enabled && super::kem_prekeys::migrated_in(tx)?;
            let add = demand.is_some() && (!notification || migrated);
            let (legacy, pq) = demand.unwrap_or((true, pq_enabled));
            if add && (!(legacy || pq) || !migrated) {
                return Ok(None);
            }
            // Kyber keys ride in the same IQ only while the server enables them
            // (property 21198; see kem_prekeys::upload_rows_in).
            if pq_enabled
                && scan_prefix::<Batch>(&*tx, Table::Checkpoints, BATCH_PREFIX)?
                    .iter()
                    .any(|(_, batch)| {
                        batch.status == "outcome_unknown" && !batch.kem_ids.is_empty()
                    })
            {
                // KEM rows only record sent=false until acknowledgement. Never
                // select those same rows under a new IQ after a reconnect.
                // Hold the complete upload rather than silently omitting PQ.
                tracing::warn!("prekey upload held: prior KEM batch outcome unknown");
                return Ok(None);
            }
            // Native ADD replenishes the inventory in <=50-key chunks.
            // Retain this store's historical local cursor convention.
            if add && legacy { while generate_in(tx)? != 0 {} }
            if notification && !add && unsent_in(&*tx)?.is_empty() { generate_in(tx)?; }
            // Declared 43.72 default: legacy ADD batch200 (EMZ0x74cb).
            // Effective MobileConfig values remain unported, as with PQ limits.
            let ids: Vec<u32> = if legacy { unsent_in(&*tx)?.into_iter().take(if add { 200 } else { UNSENT_TARGET }).collect() } else { Vec::new() };
            if ids.is_empty() && !add { return Ok(None); }
            let kem_rows = if add && pq {
                // 08Y defaults:21200 inventory200,21201 batch10. Distinct limits.
                super::kem_prekeys::add_rows_in(tx, 200, pq_batch)?
            } else if !add && pq_enabled {
                super::kem_prekeys::upload_rows_in(tx, pq_batch)?
            } else {
                Vec::new()
            };
            if ids.is_empty() && kem_rows.is_empty() { return Ok(None); }
            let mut keys = Vec::with_capacity(ids.len());
            for id in &ids {
                let record = legacy_record_in(&*tx, *id)?
                    .ok_or_else(|| invalid("unsent prekey record disappeared"))?;
                let public = record
                    .public_key()
                    .map_err(|_| invalid("prekey public key"))?;
                keys.push(key_node(*id, public.public_key_bytes(), None));
                tx.put_json(
                    Table::Checkpoints,
                    meta_key(*id).as_bytes(),
                    1,
                    &Meta {
                        status: KeyStatus::OutcomeUnknown,
                        batch: Some(iq_id.clone()),
                    },
                )?;
            }
            // Build/sign before the reservation commits. A local construction
            // failure must roll back legacy metadata and newly generated KEM rows.
            let kem_children = kem_upload_children(identity, &kem_rows)?;
            // EqE/Dfd retains signature bytes in the native row. Preserve our
            // exact publication signature with the row before request commit.
            for family in &kem_children {
                let entries = if family.description == "pq_list" { family.child_nodes() } else { std::slice::from_ref(family) };
                for entry in entries {
                    let bytes = |name: &str| -> Result<&[u8]> {
                        match entry.child(name).map(|n| &n.content) {
                            Some(crate::binary::NodeContent::Bytes(b)) => Ok(b.as_ref()),
                            _ => Err(invalid("constructed KEM publication field missing")),
                        }
                    };
                    let id = bytes("id")?;
                    let row = kem_rows.iter().find(|r| be3(r.id) == id)
                        .ok_or_else(|| invalid("constructed KEM publication ID changed"))?;
                    super::kem_prekeys::retain_signature_in(tx, row, bytes("signature")?)?;
                }
            }
            let node = if add {
                // 43.72 13z.A0B ADD: op, optional list, optional pq_list;
                // no identity/registration/type/skey/last-resort child.
                let mut children = vec![Node::new("op").attr("mode", "add")];
                if !keys.is_empty() { children.push(Node::new("list").children(keys)); }
                Node::new("iq").attr("id", iq_id.as_str()).attr("xmlns", "encrypt")
                    .attr("type", "set").attr("to", Jid::parse("s.whatsapp.net")?).children(children)
            } else {
                Node::new("iq")
                .attr("id", iq_id.as_str())
                .attr("xmlns", "encrypt")
                .attr("type", "set")
                .attr("to", Jid::parse("s.whatsapp.net")?)
                .children(vec![
                    Node::new("identity").bytes(identity.0.public.to_vec()),
                    Node::new("registration").bytes(registration.to_be_bytes().to_vec()),
                    Node::new("type").bytes(vec![0x05]),
                    Node::new("list").children(keys),
                    Node::new("skey").children(vec![
                        Node::new("id").bytes(be3(signed_id)),
                        Node::new("value").bytes(signed_public.public_key_bytes().to_vec()),
                        Node::new("signature").bytes(signed_signature.to_vec()),
                    ]),
                ])
            };
            let node = if kem_children.is_empty() {
                node
            } else {
                let mut children = node.child_nodes().to_vec();
                children.extend(kem_children);
                node.children(children)
            };
            // Local durable provenance: freeze the exact request before commit.
            // This is inspection evidence, never authority to replay an uncertain IQ.
            let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71()).encode(&node)?;
            tx.put_json(Table::Checkpoints, request_key(&iq_id).as_bytes(), 1,
            &serde_json::json!({"mode":if add { "add" } else { "set" }, "dictionary":"comparison_2_26_37_71", "wire":wire}))?;
            let kem_ids: Vec<u32> = kem_rows.iter().map(|r| r.id).collect();
            tx.put_json(
                Table::Checkpoints,
                batch_key(&iq_id).as_bytes(),
                1,
                &Batch {
                    ids: ids.clone(),
                    status: "outcome_unknown".into(),
                    code: None,
                    count_low: None,
                    pq_count_low: None,
                    reserved_at: Timestamp::now(),
                    kem_ids: kem_ids.clone(),
                },
            )?;
            Ok(Some((ids, kem_ids, node)))
        })?;
    let Some((ids, kem_ids, node)) = reserved else {
        return Ok(None);
    };
    Ok(Some((
        PendingUpload {
            iq_id,
            ids,
            kem_ids,
            deadline: tokio::time::Instant::now() + std::time::Duration::from_secs(30),
        },
        node,
    )))
}

impl PendingUpload {
    /// Only the exact `iq` result/error for this batch can settle it.
    pub fn matches(&self, node: &Node) -> bool {
        node.description == "iq"
            && node.get_attr("id").and_then(|v| v.as_text()) == Some(self.iq_id.as_str())
            // APK 43.72 0ln.A01 dispatches only result/error to IQ callbacks.
            // An incoming get/set with this ID is not a refusal or key release.
            && matches!(node.get_attr("type").and_then(|v| v.as_text()), Some("result" | "error"))
    }
    /// Apply the server reply durably: acknowledged keys become `Uploaded`;
    /// a refusal returns the keys to `Unsent` (APK keeps `sent_to_server = 0`)
    /// and records the sanitized code. Nothing is retried here.
    pub fn settle(&self, store: &ProtocolStore, node: &Node) -> Result<UploadOutcome> {
        if !self.matches(node) {
            return Err(invalid(
                "requires a matching IQ result/error for this upload",
            ));
        }
        let accepted = node.get_attr("type").and_then(|v| v.as_text()) == Some("result");
        // APK 43.72 1rA.A03 selects result -> 1ru.A02 or error ->
        // 1ud.A01. Fields from the other callback must not affect settlement.
        // APK 43.72 140/141.CK4 and 1ru.A02 select the first exact
        // list/pq_list independently; an absent flag is false.
        let low = |name: &str| {
            accepted
                && node
                    .child_nodes()
                    .iter()
                    .find(|c| c.description == name)
                    .and_then(|c| c.get_attr("count_low"))
                    .and_then(|v| v.as_text())
                    .is_some_and(|v| v.eq_ignore_ascii_case("true"))
        };
        let count_low = low("list");
        let pq_count_low = low("pq_list");
        let code = (!accepted).then(|| {
            // APK 43.72 1ud.A01 skips code-less errors, then 0HF.A00
            // uses Integer.parseInt with default zero. Do not skip a present
            // malformed code in favor of a later child's different outcome.
            node.child_nodes()
                .iter()
                .filter(|c| c.description == "error")
                .find_map(|c| c.get_attr("code").and_then(|v| v.as_text()))
                .and_then(super::mex::java_decimal_long)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or(0)
                .to_string()
        });
        store.write(|tx| {
            let key = batch_key(&self.iq_id);
            let mut batch: Batch = tx
                .get_json(Table::Checkpoints, key.as_bytes(), 1)?
                .ok_or_else(|| invalid("reserved batch disappeared"))?;
            if batch.status != "outcome_unknown" {
                return Err(invalid("batch already settled"));
            }
            // Local reservation ownership: the callback must still describe
            // this exact durable batch before any key or KEM state is changed.
            // Do not require private rows: authenticated inbox consumption may
            // legitimately remove a prekey before its upload ACK arrives.
            if batch.ids != self.ids {
                return Err(invalid("reserved batch key IDs changed"));
            }
            if batch.kem_ids != self.kem_ids {
                return Err(invalid("reserved batch KEM key IDs changed"));
            }
            for id in &batch.ids {
                let meta: Meta = tx
                    .get_json(Table::Checkpoints, meta_key(*id).as_bytes(), 1)?
                    .ok_or_else(|| invalid("reserved key metadata disappeared"))?;
                if meta.status != KeyStatus::OutcomeUnknown
                    || meta.batch.as_deref() != Some(self.iq_id.as_str())
                {
                    return Err(invalid("reserved key ownership changed"));
                }
            }
            batch.status = if accepted {
                "server_acknowledged".into()
            } else {
                "refused".into()
            };
            batch.code = code.clone();
            batch.count_low = Some(count_low);
            batch.pq_count_low = Some(pq_count_low);
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &batch)?;
            if accepted {
                super::kem_prekeys::mark_sent_in(tx, &batch.kem_ids)?;
            }
            for id in &self.ids {
                tx.put_json(
                    Table::Checkpoints,
                    meta_key(*id).as_bytes(),
                    1,
                    &Meta {
                        status: if accepted {
                            KeyStatus::Uploaded
                        } else {
                            KeyStatus::Unsent
                        },
                        batch: Some(self.iq_id.clone()),
                    },
                )?;
            }
            Ok(())
        })?;
        Ok(UploadOutcome {
            accepted,
            count_low,
            pq_count_low,
        })
    }
}

/// Parsed `notification type=encrypt` counts (APK 37.71 LX/14z.A09).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowCount {
    /// First child is `count`; independent of whether its value parsed.
    pub legacy_present: bool,
    /// Optional Java signed `count/@value` (missing/malformed becomes None).
    pub legacy: Option<i32>,
    /// Java signed `pq_count/@value`; a missing value defaults to zero.
    pub pq: Option<i32>,
}
fn count_integer(node: &Node) -> Option<i32> {
    node.get_attr("value")
        .and_then(|v| v.as_text())
        .and_then(super::mex::java_decimal_long)
        .and_then(|v| i32::try_from(v).ok())
}
fn pq_count_integer(node: &Node) -> Result<i32> {
    if node.get_attr("value").is_none() {
        return Ok(0);
    }
    count_integer(node).ok_or_else(|| invalid("pq_count value is not a signed 32-bit Java decimal"))
}
/// Recognize a prekey-count notification using its first child.
/// A malformed present PQ count refuses upload work; the actor separately ACKs
/// supported envelopes via the native base-handler error path. Legacy count uses
/// the APK's optional Integer.valueOf parser and can remain unknown.
pub fn low_count(node: &Node) -> Result<Option<LowCount>> {
    if node.description != "notification"
        || node.get_attr("type").and_then(|v| v.as_text()) != Some("encrypt")
        || node.get_attr("participant").is_some()
    {
        return Ok(None);
    }
    let Some(first) = node.child_nodes().first() else {
        return Ok(None);
    };
    // 14z.A09 000e..0038 dispatches by first child. A0H is nullable/optional;
    // PQ A05 defaults missing to zero but rejects malformed via A06. See
    // prekey-count-notifications.md; sibling count is not identity authority.
    let parsed = match first.description.as_str() {
        "count" => LowCount {
            legacy_present: true,
            legacy: count_integer(first),
            pq: node.child("pq_count").map(pq_count_integer).transpose()?,
        },
        "pq_count" => LowCount {
            legacy_present: false,
            legacy: None,
            pq: Some(pq_count_integer(first)?),
        },
        _ => return Ok(None),
    };
    Ok(Some(parsed))
}
/// Acknowledge a recognized count notification (`LX/0yO.A07(true)`; ack shape
/// `APK 37.71 LX/Cp2.A05 (0036..0074)` equivalent: id/to/class/type).
pub fn low_count_ack(node: &Node) -> Result<Node> {
    let id = node
        .get_attr("id")
        .and_then(|v| v.as_text())
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .ok_or_else(|| invalid("count notification without id"))?;
    // APK43.72 F0R.A02/A00 preserves an optional source in Eoh;
    // deferred89Y.A01/A00 reaches F3e.A05, which omits absent `to`.
    // Do not convert an explicit parse failure into an invented server peer.
    let to = node
        .get_attr("from")
        .map(|_| {
            super::receipts::jid(node, "from")
                .ok_or_else(|| invalid("count notification has invalid source"))
        })
        .transpose()?;
    let mut ack = Node::new("ack").attr("id", id);
    if let Some(to) = to {
        ack = ack.attr("to", super::receipts::ack_address(to));
    }
    Ok(ack.attr("class", "notification").attr("type", "encrypt"))
}

fn refused_upload_in(tx: &impl Lookup) -> Result<bool> {
    Ok(scan_prefix::<Batch>(tx, Table::Checkpoints, BATCH_PREFIX)?
        .iter()
        .any(|(_, batch)| batch.status == "refused"))
}

/// Whether unsent/initial inventory permits upload, with no saved refusal.
/// Native refusal retry policy is unported; reconnect is not retry authority.
pub fn wants_upload(store: &ProtocolStore) -> Result<bool> {
    store.read(|tx| {
        Ok(!refused_upload_in(tx)? && (!unsent_in(tx)?.is_empty() || record_ids(tx)?.is_empty()))
    })
}

/// Local row state for the KEM prekey an incoming hybrid `pkmsg` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KemRowState {
    /// No row: fresh hybrid bootstrap cannot run and the message stays preserved.
    Missing,
    /// Ordinary row, deleted with a successful bootstrap commit.
    Held,
    /// Native last-resort row, retained after successful use.
    LastResort,
    /// Row fails local integrity checks and is never used.
    Invalid,
}

/// Read-only classification of one local KEM row by ID. Returns no key bytes;
/// callers that print it must not print the ID either. Matching held sessions
/// need no row at all, so `Missing` alone does not predict refusal.
pub fn kem_row_state(store: &ProtocolStore, id: u32) -> Result<KemRowState> {
    match store.read(|tx| super::kem_prekeys::load(tx, id)) {
        Ok(None) => Ok(KemRowState::Missing),
        Ok(Some(row)) if row.last_resort => Ok(KemRowState::LastResort),
        Ok(Some(_)) => Ok(KemRowState::Held),
        Err(Error::InvalidInput {
            field: "kem_prekey_inventory",
            ..
        }) => Ok(KemRowState::Invalid),
        Err(error) => Err(error),
    }
}

/// Redacted counts for operators; no key material is returned or printed.
pub fn summary(store: &ProtocolStore) -> Result<Summary> {
    store.read(|tx| {
        let metas = meta_map(tx)?;
        let mut summary = Summary::default();
        for id in record_ids(tx)? {
            match metas.get(&id).map(|m| m.status) {
                None | Some(KeyStatus::Unsent) => summary.unsent += 1,
                Some(KeyStatus::OutcomeUnknown) => summary.uncertain += 1,
                Some(KeyStatus::Uploaded) => summary.uploaded += 1,
            }
        }
        for (_, batch) in scan_prefix::<Batch>(tx, Table::Checkpoints, BATCH_PREFIX)? {
            summary.upload_held_after_refusal |= batch.status == "refused";
            if batch.status == "server_acknowledged" && batch.pq_count_low == Some(true) {
                summary.pq_low_acknowledged_batches += 1;
            }
            *summary.batches.entry(batch.status).or_default() += 1;
        }
        (summary.kem_local, summary.kem_last_resort) = super::kem_prekeys::counts(tx)?;
        Ok(summary)
    })
}

/// Bytes content of a child, for tests.
#[cfg(test)]
pub(crate) fn child_bytes<'a>(node: &'a Node, name: &str) -> Option<&'a [u8]> {
    node.child_nodes()
        .iter()
        .find(|c| c.description == name)
        .and_then(|c| match &c.content {
            crate::binary::NodeContent::Bytes(b) => Some(b.as_ref()),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        session::{KeyMaterial, RegistrationStatus, SignalIdentityKey},
        store::{MasterKey, OpenOptions, StaticKeySource, WriteTx},
    };

    /// Our KEM upload entries, presented to a peer under `pqkey` exactly as
    /// uploaded (the relay assumption), pass the same receive-side parser and
    /// signature check we apply to peers' keys (`hybrid::bundle_for`). The
    /// element shape follows `LX/0y0.A00` case 85.
    #[test]
    fn kem_upload_entries_pass_the_receive_side_bundle_parser() {
        use crate::signal::{hybrid, kem};
        use wacore_libsignal::protocol::IdentityKeyPair;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("kem.redb"));
        let pair = IdentityKeyPair::generate(&mut rand::rng());
        let mut private = [0u8; 32];
        private.copy_from_slice(pair.private_key().serialize());
        let mut public = [0u8; 32];
        public.copy_from_slice(pair.public_key().public_key_bytes());
        let identity = SignalIdentityKey(KeyMaterial { public, private });
        let mut rows = Vec::new();
        for (id, last_resort) in [(9, false), (10, false), (0x00ff_fffe, true)] {
            let (kem_public, kem_secret) = kem::generate_key_pair().unwrap();
            rows.push(
                store
                    .write(|tx| {
                        super::super::kem_prekeys::install_for_test(
                            tx,
                            id,
                            &kem_public,
                            kem_secret.as_bytes(),
                            last_resort,
                        )
                    })
                    .unwrap(),
            );
        }
        let children = kem_upload_children(&identity, &rows).unwrap();
        let names: Vec<_> = children.iter().map(|n| n.description.as_str()).collect();
        assert_eq!(names, ["pq_list", "pq_last_resort_key"]);
        let entries: Vec<&Node> = children[0].child_nodes().iter().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.description == "key"));
        let last = &children[1];
        for entry in entries.iter().copied().chain([last]) {
            let fields: Vec<_> = entry
                .child_nodes()
                .iter()
                .map(|n| n.description.as_str())
                .collect();
            assert_eq!(fields, ["id", "value", "signature"]);
        }
        // A peer's directory reply carrying one of our entries as `pqkey`.
        let peer = Jid::parse("15555550199@s.whatsapp.net").unwrap();
        let signed = KeyPair::generate(&mut rand::rng());
        let signed_signature = pair
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
            .unwrap();
        for (entry, expected_id) in [(entries[0], 9), (last, 0x00ff_fffe)] {
            let user = Node::new("user").attr("jid", peer.clone()).children(vec![
                Node::new("registration").bytes(vec![0, 0, 0, 42]),
                Node::new("type").bytes(vec![5]),
                Node::new("identity").bytes(public.to_vec()),
                Node::new("skey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 7]),
                    Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                    Node::new("signature").bytes(signed_signature.to_vec()),
                ]),
                Node::new("pqkey").children(entry.child_nodes().to_vec()),
            ]);
            let reply = Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(vec![user])]);
            assert!(
                hybrid::bundle_for(&reply, &peer).is_ok(),
                "entry {expected_id} must verify as a peer's pqkey"
            );
        }
        // Refusals: an ID wider than three bytes, two last-resort keys.
        let mut wide = rows[0].clone();
        wide.id = 0x0100_0000;
        assert!(kem_upload_children(&identity, &[wide]).is_err());
        assert!(kem_upload_children(&identity, &[rows[2].clone(), rows[2].clone()]).is_err());
        assert!(kem_upload_children(&identity, &[]).unwrap().is_empty());
    }

    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn account_and_signed(store: &ProtocolStore) -> (AccountRecord, Vec<u8>) {
        let mut rng = rand::rng();
        let identity = KeyPair::generate(&mut rng);
        let signed = KeyPair::generate(&mut rng);
        let signature = identity
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let record = SignedPreKeyRecord::new(
            7.into(),
            wacore_libsignal::protocol::Timestamp::from_epoch_millis(0),
            &signed,
            &signature,
        );
        let mut public = [0u8; 32];
        public.copy_from_slice(identity.public_key.public_key_bytes());
        let account = AccountRecord {
            generation: store.generation(),
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(0x0102_0304),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public,
                private: *identity.private_key.serialize(),
            })),
            ..Default::default()
        };
        store.write(|tx| account.save(tx)).unwrap();
        (account, record.serialize().unwrap())
    }
    fn set_status(tx: &mut WriteTx<'_>, id: u32, status: KeyStatus) -> Result<()> {
        tx.put_json(
            Table::Checkpoints,
            meta_key(id).as_bytes(),
            1,
            &Meta {
                status,
                batch: None,
            },
        )
    }

    #[test]
    fn generation_follows_apk_counts_ids_and_wraps_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("gen.redb"));
        assert!(wants_upload(&store).unwrap(), "empty store generates first");
        assert_eq!(generate(&store).unwrap(), 50);
        assert_eq!(unsent_ids(&store).unwrap(), (1..=50).collect::<Vec<_>>());
        assert_eq!(generate(&store).unwrap(), 50);
        assert_eq!(unsent_ids(&store).unwrap().len(), 100);
        // Uploaded keys do not count as unsent; uncertain keys are excluded too.
        store
            .write(|tx| {
                for id in 1..=90 {
                    set_status(tx, id, KeyStatus::Uploaded)?;
                }
                set_status(tx, 91, KeyStatus::OutcomeUnknown)
            })
            .unwrap();
        assert_eq!(unsent_ids(&store).unwrap(), (92..=100).collect::<Vec<_>>());
        assert_eq!(generate(&store).unwrap(), 50);
        assert_eq!(unsent_ids(&store).unwrap().len(), 59);
        // Near the target only the remainder is generated, then nothing.
        store
            .write(|tx| {
                for id in 10_000..=(10_000 + 811 - 59 - 1) {
                    tx.put_json(Table::Prekeys, record_key(id).as_bytes(), 1, &vec![0u8])?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(unsent_ids(&store).unwrap().len(), 811);
        assert_eq!(generate(&store).unwrap(), 1);
        assert_eq!(generate(&store).unwrap(), 0);
        // Wrap-around: ids stay within 1..=0xFFFFFE and never overwrite.
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("wrap.redb"));
        store
            .write(|tx| tx.put_json(Table::Checkpoints, NEXT_ID_KEY, 1, &(ID_MODULUS - 2)))
            .unwrap();
        assert_eq!(generate(&store).unwrap(), 50);
        let ids = unsent_ids(&store).unwrap();
        assert!(ids.contains(&ID_MODULUS) && ids.contains(&1) && ids.contains(&48));
        assert!(!ids.contains(&0) && !ids.contains(&(ID_MODULUS + 1)));
        store
            .write(|tx| tx.put_json(Table::Checkpoints, NEXT_ID_KEY, 1, &(ID_MODULUS - 2)))
            .unwrap();
        assert!(generate(&store).is_err(), "collision refused");
        let record: Vec<u8> = store
            .read(|tx| tx.get_json(Table::Prekeys, record_key(1).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            PreKeyRecord::deserialize(&record).unwrap().id().unwrap(),
            1.into()
        );
    }

    /// 13c.A03: Kyber keys ride with the upload only while property 21198 is
    /// on, and only with an unsent last-resort key; once that is acknowledged
    /// the next upload carries none.
    #[test]
    fn publication_rejects_stale_account_authority_without_reserving_keys() {
        for change in 0..9 {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("stale-publication.redb");
            let store = open(&path);
            let (account, signed) = account_and_signed(&store);
            generate(&store).unwrap();
            let mut current = account.clone();
            match change {
                0 => current.generation.0 += 1,
                1 => current.registration = RegistrationStatus::Revoked,
                2 => current.signal_registration_id = Some(42),
                3 => current.signal_identity.as_mut().unwrap().0.public[0] ^= 1,
                4 => current.signal_identity.as_mut().unwrap().0.private[0] ^= 1,
                5 => current.own_jid = Some(Jid::parse("15555550123@s.whatsapp.net").unwrap()),
                6 => {
                    current.phone =
                        Some(crate::model::phone::PhoneNumber::new(1, "5555550123").unwrap())
                }
                7 => current.signal_identity = None,
                8 => {}
                _ => unreachable!(),
            }
            store
                .write(|tx| {
                    if change == 8 {
                        tx.delete(Table::Account, b"account").map(|_| ())
                    } else {
                        current.save(tx)
                    }
                })
                .unwrap();
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        [Table::Prekeys, Table::Checkpoints]
                            .into_iter()
                            .map(|table| tx.scan(table, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            assert!(
                begin_upload(&store, &account, &signed).is_err(),
                "case {change}"
            );
            assert_eq!(snapshot(&store), before);
            drop(store);
            let store = open(&path);
            assert_eq!(snapshot(&store), before);
        }
    }

    #[test]
    fn publication_ids_are_lossless_and_legacy_rows_match_their_store_keys() {
        for (stored, embedded, accepted) in [
            (1u32, 2u32, false),
            (0x0100_0001, 0x0100_0001, false),
            (0x00ff_ffff, 0x00ff_ffff, true),
            (0, 0, true),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("legacy-id.redb");
            let store = open(&path);
            let (account, signed) = account_and_signed(&store);
            let pair = KeyPair::generate(&mut rand::rng());
            let raw = PreKeyRecord::new(embedded.into(), &pair)
                .serialize()
                .unwrap();
            store
                .write(|tx| tx.put_json(Table::Prekeys, record_key(stored).as_bytes(), 1, &raw))
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            let upload = begin_upload(&store, &account, &signed);
            if accepted {
                let (_, node) = upload.unwrap().unwrap();
                assert_eq!(
                    child_bytes(&node.child("list").unwrap().child_nodes()[0], "id"),
                    Some(be3(stored).as_slice())
                );
            } else {
                assert!(upload.is_err());
                drop(store);
                let store = open(&path);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                        .unwrap(),
                    before
                );
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<Vec<u8>>(
                            Table::Prekeys,
                            record_key(stored).as_bytes(),
                            1
                        ))
                        .unwrap(),
                    Some(raw)
                );
            }
        }
        for id in [0, 0x00ff_ffff, 0x0100_0000, u32::MAX] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("signed-id.redb"));
            let (account, raw) = account_and_signed(&store);
            let record = SignedPreKeyRecord::deserialize(&raw).unwrap();
            let signed = SignedPreKeyRecord::new(
                id.into(),
                wacore_libsignal::protocol::Timestamp::from_epoch_millis(0),
                &record.key_pair().unwrap(),
                &record.signature().unwrap(),
            )
            .serialize()
            .unwrap();
            generate(&store).unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            let upload = begin_upload(&store, &account, &signed);
            if id <= 0x00ff_ffff {
                let (_, node) = upload.unwrap().unwrap();
                assert_eq!(
                    child_bytes(node.child("skey").unwrap(), "id"),
                    Some(be3(id).as_slice())
                );
            } else {
                assert!(upload.is_err());
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                        .unwrap(),
                    before
                );
                assert_eq!(summary(&store).unwrap().uncertain, 0);
            }
        }
    }

    #[test]
    fn legacy_publication_requires_a_signed_key_for_its_identity() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("signature.redb"));
        let (account, signed) = account_and_signed(&store);
        let other = open(&dir.path().join("other.redb"));
        let (_, foreign_signed) = account_and_signed(&other);
        generate(&store).unwrap();
        assert!(!super::super::abprops::pq_keys_upload_enabled(&store).unwrap());
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
            .unwrap();
        assert!(begin_upload(&store, &account, &foreign_signed).is_err());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap(),
            before
        );
        assert_eq!(summary(&store).unwrap().uncertain, 0);
        // Unrelated account metadata may change without revoking publication.
        let mut updated = account.clone();
        updated.last_authenticated_at = Some(Timestamp::now());
        store.write(|tx| updated.save(tx)).unwrap();
        assert!(begin_upload(&store, &account, &signed).unwrap().is_some());
    }

    #[test]
    fn kyber_keys_upload_only_behind_the_server_property() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("kyber.redb"));
        let (account, signed) = account_and_signed(&store);
        generate(&store).unwrap();
        let (pending, iq) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        assert!(iq.child("pq_list").is_none() && iq.child("pq_last_resort_key").is_none());
        let id = iq.get_attr("id").unwrap().as_text().unwrap().to_owned();
        pending
            .settle(
                &store,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result"),
            )
            .unwrap();
        // The server turns 21198 on in the A/B reply.
        let action = super::super::abprops::queue(&store).unwrap();
        super::super::actions::reserve_pending(&store).unwrap();
        let props = Node::new("props").attr("hash", "h").children(vec![
            Node::new("prop")
                .attr("config_code", "21198")
                .attr("config_value", "1"),
        ]);
        assert!(
            super::super::actions::record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", action.as_str())
                    .attr("type", "result")
                    .children(vec![props]),
            )
            .unwrap()
        );
        assert!(super::super::abprops::pq_keys_upload_enabled(&store).unwrap());
        generate(&store).unwrap();
        let (pending, iq) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        let names: Vec<&str> = iq
            .child_nodes()
            .iter()
            .map(|c| c.description.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "identity",
                "registration",
                "type",
                "list",
                "skey",
                "pq_list",
                "pq_last_resort_key"
            ]
        );
        assert_eq!(iq.child("pq_list").unwrap().child_nodes().len(), 10);
        let id = iq.get_attr("id").unwrap().as_text().unwrap().to_owned();
        pending
            .settle(
                &store,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result"),
            )
            .unwrap();
        assert_eq!(
            store.read(super::super::kem_prekeys::counts).unwrap(),
            (11, 1)
        );
        generate(&store).unwrap();
        let (_, iq) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        assert!(
            iq.child("pq_list").is_none(),
            "last-resort key already sent"
        );
    }

    #[test]
    fn non_reply_iqs_preserve_reserved_prekey_upload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("non-reply.redb");
        let store = open(&path);
        let (account, signed) = account_and_signed(&store);
        assert_eq!(generate(&store).unwrap(), 50);
        let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        let original: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, batch_key(&pending.iq_id).as_bytes(), 1))
            .unwrap()
            .unwrap();
        for kind in [
            None,
            Some("get"),
            Some("set"),
            Some("RESULT"),
            Some("unexpected"),
        ] {
            let node = Node::new("iq")
                .attr("id", pending.iq_id.as_str())
                .children(vec![Node::new("error").attr("code", "500")]);
            let node = if let Some(kind) = kind {
                node.attr("type", kind)
            } else {
                node
            };
            assert!(!pending.matches(&node));
            assert!(pending.settle(&store, &node).is_err());
            let held: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, batch_key(&pending.iq_id).as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(held, original);
            let state = summary(&store).unwrap();
            assert_eq!((state.unsent, state.uncertain, state.uploaded), (0, 50, 0));
            assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
        }
        drop(store);
        let store = open(&path);
        let state = summary(&store).unwrap();
        assert_eq!((state.unsent, state.uncertain, state.uploaded), (0, 50, 0));
        let result = Node::new("iq")
            .attr("id", pending.iq_id.as_str())
            .attr("type", "result");
        assert!(pending.settle(&store, &result).unwrap().accepted);
        assert!(pending.settle(&store, &result).is_err());
        assert_eq!(summary(&store).unwrap().uploaded, 50);
    }

    #[test]
    fn upload_construction_failure_rolls_back_keys_and_batch_across_reopen() {
        for pq in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("construction.redb");
            let store = open(&path);
            if pq {
                let id = super::super::abprops::queue(&store).unwrap();
                super::super::actions::reserve_pending(&store).unwrap();
                super::super::actions::record_reply(
                    &store,
                    &Node::new("iq")
                        .attr("id", id.as_str())
                        .attr("type", "result")
                        .children(vec![Node::new("props").attr("hash", "fixture").children(
                            vec![
                        Node::new("prop").attr("config_code", "21198").attr("config_value", "1")
                    ],
                        )]),
                )
                .unwrap();
            }
            generate(&store).unwrap();
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        [Table::Prekeys, Table::Checkpoints]
                            .into_iter()
                            .map(|table| tx.scan(table, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            let (mut account, signed) = account_and_signed(&store);
            account.signal_identity.as_mut().unwrap().0.public[0] ^= 1;
            assert!(begin_upload(&store, &account, &signed).is_err());
            assert_eq!(snapshot(&store), before);
            let state = summary(&store).unwrap();
            assert_eq!((state.unsent, state.uncertain, state.uploaded), (50, 0, 0));
            drop(store);
            let store = open(&path);
            assert_eq!(snapshot(&store), before);
            account.signal_identity.as_mut().unwrap().0.public[0] ^= 1;
            let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
            assert_eq!(summary(&store).unwrap().uncertain, 50);
            assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
            let result = Node::new("iq")
                .attr("id", pending.iq_id.as_str())
                .attr("type", "result");
            assert!(pending.settle(&store, &result).unwrap().accepted);
        }
    }

    #[test]
    fn uncertain_kem_batch_cannot_be_selected_again_with_new_legacy_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kem-uncertain.redb");
        let store = open(&path);
        let (account, signed) = account_and_signed(&store);
        let id = super::super::abprops::queue(&store).unwrap();
        super::super::actions::reserve_pending(&store).unwrap();
        super::super::actions::record_reply(
            &store,
            &Node::new("iq")
                .attr("id", id.as_str())
                .attr("type", "result")
                .children(vec![Node::new("props").attr("hash", "fixture").children(
                    vec![
                Node::new("prop").attr("config_code", "21198").attr("config_value", "1")
            ],
                )]),
        )
        .unwrap();
        generate(&store).unwrap();
        let (pending, first) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        assert!(first.child("pq_list").is_some());
        assert!(first.child("pq_last_resort_key").is_some());
        // A fresh connection may generate new legacy keys. The previous KEM
        // rows still have sent=false, but are held by the original Batch.
        generate(&store).unwrap();
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    [Table::Prekeys, Table::Checkpoints]
                        .into_iter()
                        .map(|table| tx.scan(table, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                })
                .unwrap()
        };
        // A matching IQ must not settle different KEM material if the saved
        // batch changes independently of the original callback reservation.
        let reserved_key = batch_key(&pending.iq_id);
        let original: Batch = store
            .read(|tx| tx.get_json(Table::Checkpoints, reserved_key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        let mut reordered = original.kem_ids.clone();
        reordered.reverse();
        for kem_ids in [Vec::new(), reordered, vec![0x00ff_fffe]] {
            let mut changed = original.clone();
            changed.kem_ids = kem_ids;
            store
                .write(|tx| tx.put_json(Table::Checkpoints, reserved_key.as_bytes(), 1, &changed))
                .unwrap();
            let damaged = snapshot(&store);
            for kind in ["result", "error"] {
                let reply = Node::new("iq")
                    .attr("id", pending.iq_id.as_str())
                    .attr("type", kind);
                assert!(pending.settle(&store, &reply).is_err());
                assert_eq!(snapshot(&store), damaged);
            }
        }
        // Restore only this synthetic fixture; production never repairs or
        // rewrites an uncertain batch to make a callback pass.
        store
            .write(|tx| tx.put_json(Table::Checkpoints, reserved_key.as_bytes(), 1, &original))
            .unwrap();
        let before = snapshot(&store);
        assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
        assert_eq!(snapshot(&store), before);
        drop(store);
        let store = open(&path);
        assert_eq!(snapshot(&store), before);
        assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
        assert_eq!(snapshot(&store), before);
        assert_eq!(
            (
                summary(&store).unwrap().unsent,
                summary(&store).unwrap().uncertain
            ),
            (50, 50)
        );
        // A local missing key is a store fault, not a successful settlement.
        // Remove the second KEM row so an earlier mark-sent write must roll back.
        let batch: Batch = store
            .read(|tx| tx.get_json(Table::Checkpoints, batch_key(&pending.iq_id).as_bytes(), 1))
            .unwrap()
            .unwrap();
        let missing_id = batch.kem_ids[1];
        let missing_key = super::super::kem_prekeys::record_key(missing_id);
        let row: super::super::kem_prekeys::StoredKemPrekey = store
            .read(|tx| tx.get_json(Table::Prekeys, missing_key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        store
            .write(|tx| tx.delete(Table::Prekeys, missing_key.as_bytes()))
            .unwrap();
        let damaged = snapshot(&store);
        let result = Node::new("iq")
            .attr("id", pending.iq_id.as_str())
            .attr("type", "result");
        assert!(pending.settle(&store, &result).is_err());
        assert_eq!(snapshot(&store), damaged);
        drop(store);
        let store = open(&path);
        assert_eq!(snapshot(&store), damaged);
        assert!(pending.settle(&store, &result).is_err());
        assert_eq!(snapshot(&store), damaged);
        // Restore only the deleted synthetic fixture, never generate replacement
        // material or reset a real pending upload as an automatic repair.
        store
            .write(|tx| tx.put_json(Table::Prekeys, missing_key.as_bytes(), 1, &row))
            .unwrap();
        // The real inbox helper may consume an ordinary key before upload ACK.
        // Its marker explains the absence without resurrecting private material.
        store
            .write(|tx| super::super::kem_prekeys::consume_in(tx, missing_id, &row))
            .unwrap();
        assert!(
            store
                .read(|tx| super::super::kem_prekeys::load(tx, missing_id))
                .unwrap()
                .is_none()
        );
        drop(store);
        let store = open(&path);
        // Only the original correlated result releases that uncertainty.
        pending
            .settle(
                &store,
                &Node::new("iq")
                    .attr("id", pending.iq_id.as_str())
                    .attr("type", "result"),
            )
            .unwrap();
        assert!(
            store
                .read(|tx| super::super::kem_prekeys::load(tx, missing_id))
                .unwrap()
                .is_none()
        );
        let (_, next) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        assert!(next.child("pq_list").is_none());
        assert!(next.child("pq_last_resort_key").is_none());
    }

    #[test]
    fn frozen_upload_request_survives_reopen_without_replay_or_legacy_backfill() {
        for historical in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("frozen.redb");
            let store = open(&path);
            let (account, signed) = account_and_signed(&store);
            generate(&store).unwrap();
            let (pending, node) = begin_upload(&store, &account, &signed).unwrap().unwrap();
            let key = request_key(&pending.iq_id);
            let frozen: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            let expected = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
            assert_eq!(frozen["mode"], "set");
            assert_eq!(frozen["dictionary"], "comparison_2_26_37_71");
            assert_eq!(
                serde_json::from_value::<Vec<u8>>(frozen["wire"].clone()).unwrap(),
                expected
            );
            if historical {
                // Model an older reserved batch which never stored wire bytes.
                store
                    .write(|tx| tx.delete(Table::Checkpoints, key.as_bytes()))
                    .unwrap();
            }
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                .unwrap();
            assert_eq!(
                summary(&store).unwrap().batches.get("outcome_unknown"),
                Some(&1)
            );
            drop(store);
            let store = open(&path);
            assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 10000))
                    .unwrap(),
                before
            );
            let saved: Option<serde_json::Value> = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap();
            assert_eq!(saved, (!historical).then_some(frozen.clone()));
            let reply = Node::new("iq")
                .attr("id", pending.iq_id.as_str())
                .attr("type", "result");
            assert!(pending.settle(&store, &reply).unwrap().accepted);
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        key.as_bytes(),
                        1
                    ))
                    .unwrap(),
                saved
            );
        }
    }

    #[test]
    fn upload_iq_matches_traced_layout_and_settles_by_reply() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("upload.redb");
        let store = open(&path);
        let (account, signed) = account_and_signed(&store);
        assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
        assert_eq!(generate(&store).unwrap(), 50);
        let (pending, iq) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        assert_eq!(iq.description, "iq");
        assert_eq!(
            iq.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        assert_eq!(iq.get_attr("type").and_then(|v| v.as_text()), Some("set"));
        assert_eq!(
            iq.get_attr("to")
                .and_then(|v| v.as_jid())
                .map(|j| j.to_string()),
            Some("s.whatsapp.net".into())
        );
        let names: Vec<&str> = iq
            .child_nodes()
            .iter()
            .map(|c| c.description.as_str())
            .collect();
        assert_eq!(names, ["identity", "registration", "type", "list", "skey"]);
        assert_eq!(child_bytes(&iq, "identity").unwrap().len(), 32);
        assert_eq!(child_bytes(&iq, "registration").unwrap(), &[1, 2, 3, 4]);
        assert_eq!(child_bytes(&iq, "type").unwrap(), &[5]);
        let list = &iq.child_nodes()[3];
        assert_eq!(list.child_nodes().len(), 50);
        let first = &list.child_nodes()[0];
        assert_eq!(first.description, "key");
        assert_eq!(child_bytes(first, "id").unwrap(), &[0, 0, 1]);
        assert_eq!(
            child_bytes(first, "value").unwrap().len(),
            32,
            "type byte stripped"
        );
        assert!(child_bytes(first, "signature").is_none());
        let skey = &iq.child_nodes()[4];
        assert_eq!(child_bytes(skey, "id").unwrap(), &[0, 0, 7]);
        assert_eq!(child_bytes(skey, "value").unwrap().len(), 32);
        assert_eq!(child_bytes(skey, "signature").unwrap().len(), 64);
        let wire = format!("{iq:?}");
        assert!(!wire.contains("pq_list") && !wire.contains("\"op\""));
        // Reserved before any write: nothing is unsent, a second begin is None.
        assert!(unsent_ids(&store).unwrap().is_empty());
        assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
        assert_eq!(summary(&store).unwrap().uncertain, 50);
        // Wrong reply id is refused; the result settles once, with count_low.
        let other = Node::new("iq").attr("id", "other").attr("type", "result");
        assert!(!pending.matches(&other));
        assert!(pending.settle(&store, &other).is_err());
        let iq_id = iq
            .get_attr("id")
            .and_then(|v| v.as_text())
            .unwrap()
            .to_owned();
        let result = Node::new("iq")
            .attr("id", iq_id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("list").attr("count_low", "true")]);
        assert_eq!(
            pending.settle(&store, &result).unwrap(),
            UploadOutcome {
                accepted: true,
                count_low: true,
                pq_count_low: false
            }
        );
        assert!(pending.settle(&store, &result).is_err(), "already settled");
        let s = summary(&store).unwrap();
        assert_eq!((s.unsent, s.uncertain, s.uploaded), (0, 0, 50));
        assert_eq!(s.batches.get("server_acknowledged"), Some(&1));
        assert!(!wants_upload(&store).unwrap());
        // A refusal returns keys to unsent and records the code; restart keeps it.
        assert_eq!(generate(&store).unwrap(), 50);
        let (pending, iq) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        let iq_id = iq
            .get_attr("id")
            .and_then(|v| v.as_text())
            .unwrap()
            .to_owned();
        let error = Node::new("iq")
            .attr("id", iq_id.as_str())
            .attr("type", "error")
            .children(vec![
                Node::new("error")
                    .attr("code", "500")
                    .attr("text", "internal-server-error"),
            ]);
        assert_eq!(
            pending.settle(&store, &error).unwrap(),
            UploadOutcome {
                accepted: false,
                count_low: false,
                pq_count_low: false
            }
        );
        drop(store);
        let store = open(&path);
        let s = summary(&store).unwrap();
        assert_eq!((s.unsent, s.uncertain, s.uploaded), (50, 0, 50));
        assert_eq!(s.batches.get("refused"), Some(&1));
        let batch: Batch = store
            .read(|tx| tx.get_json(Table::Checkpoints, batch_key(&iq_id).as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(batch.code.as_deref(), Some("500"));
        assert!(
            !wants_upload(&store).unwrap(),
            "refusal is not retry authority"
        );
        assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
    }

    #[test]
    fn upload_reply_fields_belong_to_the_selected_callback_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reply-fields.redb");
        let store = open(&path);
        let (account, signed) = account_and_signed(&store);
        drop(store);
        for (kind, accepted, expected_code, expected_low) in [
            ("result", true, None, true),
            ("error", false, Some("503"), false),
        ] {
            let store = open(&path);
            generate(&store).unwrap();
            let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
            let reply = Node::new("iq")
                .attr("id", pending.iq_id.as_str())
                .attr("type", kind)
                .children(vec![
                    Node::new("list").attr("count_low", "TrUe"),
                    Node::new("error").attr("code", "503"),
                ]);
            let outcome = pending.settle(&store, &reply).unwrap();
            assert_eq!(outcome.accepted, accepted);
            assert_eq!(outcome.count_low, expected_low);
            drop(store);
            let store = open(&path);
            let batch: Batch = store
                .read(|tx| tx.get_json(Table::Checkpoints, batch_key(&pending.iq_id).as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(batch.code.as_deref(), expected_code);
            assert_eq!(batch.count_low, Some(expected_low));
            assert_eq!(
                batch.status,
                if accepted {
                    "server_acknowledged"
                } else {
                    "refused"
                }
            );
        }
    }

    #[test]
    fn upload_refusal_selects_first_coded_error_and_preserves_java_integer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("coded-errors.redb");
        for (case, (raw, expected)) in [
            (None, "0"),
            (Some("+00503"), "503"),
            (Some("-2147483648"), "-2147483648"),
            (Some("2147483647"), "2147483647"),
            (Some("2147483648"), "0"),
            (Some(""), "0"),
            (Some(" 503"), "0"),
            (Some("oops"), "0"),
            (Some("５０３"), "503"),
        ]
        .into_iter()
        .enumerate()
        {
            let path = path.with_file_name(format!("coded-errors-{case}.redb"));
            let store = open(&path);
            let (account, signed) = account_and_signed(&store);
            generate(&store).unwrap();
            let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
            let mut children = vec![Node::new("error").attr("text", "no-code")];
            if let Some(raw) = raw {
                children.push(Node::new("error").attr("code", raw));
                children.push(Node::new("error").attr("code", "406"));
            }
            let reply = Node::new("iq")
                .attr("id", pending.iq_id.as_str())
                .attr("type", "error")
                .children(children);
            assert!(!pending.settle(&store, &reply).unwrap().accepted);
            drop(store);
            let store = open(&path);
            let batch: Batch = store
                .read(|tx| tx.get_json(Table::Checkpoints, batch_key(&pending.iq_id).as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(batch.code.as_deref(), Some(expected), "input {raw:?}");
            assert_eq!(batch.status, "refused");
        }
    }

    #[test]
    fn upload_settlement_rechecks_durable_key_ownership_without_partial_updates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ownership.redb");
        let store = open(&path);
        let (account, signed) = account_and_signed(&store);
        generate(&store).unwrap();
        let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        let key = batch_key(&pending.iq_id);
        let original: Batch = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        let last = *pending.ids.last().unwrap();
        for damage in 0..4 {
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &original)?;
                    for id in &pending.ids {
                        tx.put_json(
                            Table::Checkpoints,
                            meta_key(*id).as_bytes(),
                            1,
                            &Meta {
                                status: KeyStatus::OutcomeUnknown,
                                batch: Some(pending.iq_id.clone()),
                            },
                        )?;
                    }
                    match damage {
                        0 => {
                            let mut changed = original.clone();
                            changed.ids.reverse();
                            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &changed)?;
                        }
                        1 => {
                            tx.delete(Table::Checkpoints, meta_key(last).as_bytes())?;
                        }
                        2 | 3 => {
                            tx.put_json(
                                Table::Checkpoints,
                                meta_key(last).as_bytes(),
                                1,
                                &Meta {
                                    status: if damage == 2 {
                                        KeyStatus::Uploaded
                                    } else {
                                        KeyStatus::OutcomeUnknown
                                    },
                                    batch: Some(if damage == 2 {
                                        pending.iq_id.clone()
                                    } else {
                                        "another-upload".into()
                                    }),
                                },
                            )?;
                        }
                        _ => unreachable!(),
                    }
                    Ok(())
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 4096))
                .unwrap();
            for kind in ["result", "error"] {
                let reply = Node::new("iq")
                    .attr("id", pending.iq_id.as_str())
                    .attr("type", kind);
                assert!(pending.settle(&store, &reply).is_err());
                assert_eq!(
                    store
                        .read(|tx| tx.scan_after(Table::Checkpoints, 1, None, 4096))
                        .unwrap(),
                    before
                );
            }
        }
        drop(store);
        let store = open(&path);
        let reply = Node::new("iq")
            .attr("id", pending.iq_id.as_str())
            .attr("type", "result");
        assert!(
            pending.settle(&store, &reply).is_err(),
            "ownership mismatch survives reopen"
        );
    }

    #[test]
    fn pq_count_low_is_independent_first_child_and_persisted_without_backfill() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pq-low.redb");
        let store = open(&path);
        let (account, signed) = account_and_signed(&store);
        drop(store);
        for (kind, first, expected) in [
            ("result", "TrUe", true),
            ("result", "false", false),
            ("error", "true", false),
        ] {
            let store = open(&path);
            generate(&store).unwrap();
            let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
            let reply = Node::new("iq")
                .attr("id", pending.iq_id.as_str())
                .attr("type", kind)
                .children(vec![
                    Node::new("pq_list").attr("count_low", first),
                    Node::new("pq_list").attr("count_low", "true"),
                ]);
            let outcome = pending.settle(&store, &reply).unwrap();
            assert!(!outcome.count_low, "PQ demand must not imply legacy demand");
            assert_eq!(outcome.pq_count_low, expected);
            drop(store);
            let store = open(&path);
            let batch: Batch = store
                .read(|tx| tx.get_json(Table::Checkpoints, batch_key(&pending.iq_id).as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(batch.pq_count_low, Some(expected));
            assert_eq!(summary(&store).unwrap().pq_low_acknowledged_batches, 1);
            let mut historical = serde_json::to_value(&batch).unwrap();
            historical.as_object_mut().unwrap().remove("pq_count_low");
            let restored: Batch = serde_json::from_value(historical).unwrap();
            assert_eq!(
                restored.pq_count_low, None,
                "unknown historical flag stays unknown"
            );
        }
    }

    #[test]
    fn refused_upload_holds_new_reservations_after_notifications_and_reopen() {
        for kind in ["406", "503", "0"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("refusal-hold.redb");
            let store = open(&path);
            let (account, signed) = account_and_signed(&store);
            generate(&store).unwrap();
            let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
            let error = Node::new("iq")
                .attr("id", pending.iq_id.as_str())
                .attr("type", "error")
                .children(vec![Node::new("error").attr("code", kind)]);
            assert!(!pending.settle(&store, &error).unwrap().accepted);
            // Fresh unsent keys do not authorize retry of a refused upload.
            generate(&store).unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 4096))
                .unwrap();
            assert!(!wants_upload(&store).unwrap());
            assert!(summary(&store).unwrap().upload_held_after_refusal);
            for _ in 0..2 {
                assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
            }
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 4096))
                    .unwrap(),
                before
            );
            drop(store);
            let store = open(&path);
            assert!(!wants_upload(&store).unwrap());
            assert!(summary(&store).unwrap().upload_held_after_refusal);
            assert!(begin_upload(&store, &account, &signed).unwrap().is_none());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 4096))
                    .unwrap(),
                before
            );
        }
    }

    #[test]
    fn migrated_followups_reserve_add_for_exact_demand_and_freeze_wire() {
        for (legacy, pq) in [(true, false), (false, true), (true, true)] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("add.redb");
            let store = open(&path);
            let (account, signed) = account_and_signed(&store);
            assert!(
                begin_followup_upload(&store, &account, &signed, legacy, pq)
                    .unwrap()
                    .is_none()
            );
            let prop = super::super::abprops::queue(&store).unwrap();
            super::super::actions::reserve_pending(&store).unwrap();
            super::super::actions::record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", prop.as_str())
                    .attr("type", "result")
                    .children(vec![Node::new("props").attr("hash", "fixture").children(
                        vec![
                            Node::new("prop")
                                .attr("config_code", "21198")
                                .attr("config_value", "1"),
                        ],
                    )]),
            )
            .unwrap();
            // Gate alone does not prove uploaded-LR migration or generate ADD keys.
            assert!(
                begin_followup_upload(&store, &account, &signed, legacy, pq)
                    .unwrap()
                    .is_none()
            );
            generate(&store).unwrap();
            let (initial, set) = begin_upload(&store, &account, &signed).unwrap().unwrap();
            initial
                .settle(
                    &store,
                    &Node::new("iq")
                        .attr("id", initial.iq_id.as_str())
                        .attr("type", "result"),
                )
                .unwrap();
            let frozen_set: serde_json::Value = store
                .read(|tx| {
                    tx.get_json(
                        Table::Checkpoints,
                        request_key(&initial.iq_id).as_bytes(),
                        1,
                    )
                })
                .unwrap()
                .unwrap();
            assert!(set.child("pq_last_resort_key").is_some());
            let snapshot = |store: &ProtocolStore| {
                store
                    .read(|tx| {
                        [Table::Prekeys, Table::Checkpoints]
                            .into_iter()
                            .map(|t| tx.scan(t, 1, 10000))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot(&store);
            let mut wrong = account.clone();
            wrong.signal_identity.as_mut().unwrap().0.public[0] ^= 1;
            assert!(begin_followup_upload(&store, &wrong, &signed, legacy, pq).is_err());
            assert_eq!(
                snapshot(&store),
                before,
                "ADD construction failure must roll back inventory and reservation"
            );
            let (pending, add) = begin_followup_upload(&store, &account, &signed, legacy, pq)
                .unwrap()
                .unwrap();
            let expected: Vec<&str> = match (legacy, pq) {
                (true, true) => vec!["op", "list", "pq_list"],
                (true, false) => vec!["op", "list"],
                _ => vec!["op", "pq_list"],
            };
            assert_eq!(
                add.child_nodes()
                    .iter()
                    .map(|n| n.description.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                add.child("op")
                    .unwrap()
                    .get_attr("mode")
                    .and_then(|v| v.as_text()),
                Some("add")
            );
            assert_eq!(pending.ids.len(), if legacy { 200 } else { 0 });
            assert_eq!(pending.kem_ids.len(), if pq { 10 } else { 0 });
            if legacy {
                assert_eq!(summary(&store).unwrap().unsent, 612);
            }
            let checkpoint: serde_json::Value = store
                .read(|tx| {
                    tx.get_json(
                        Table::Checkpoints,
                        request_key(&pending.iq_id).as_bytes(),
                        1,
                    )
                })
                .unwrap()
                .unwrap();
            assert_eq!(checkpoint["mode"], "add");
            assert_eq!(
                checkpoint["wire"],
                serde_json::json!(
                    Encoder::new(TokenDictionary::comparison_2_26_37_71())
                        .encode(&add)
                        .unwrap()
                )
            );
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        request_key(&initial.iq_id).as_bytes(),
                        1
                    ))
                    .unwrap()
                    .unwrap(),
                frozen_set
            );
            if pq {
                assert!(
                    begin_followup_upload(&store, &account, &signed, legacy, pq)
                        .unwrap()
                        .is_none(),
                    "uncertain KEM reservation must hold"
                );
            }
            if pq {
                assert!(
                    begin_notification_upload(&store, &account, &signed, true)
                        .unwrap()
                        .is_none(),
                    "notification cannot bypass uncertain KEM reservation"
                );
            }
            drop(store);
            let store = open(&path);
            if pq {
                assert!(
                    begin_notification_upload(&store, &account, &signed, true)
                        .unwrap()
                        .is_none()
                );
            }
            if pq {
                assert!(
                    begin_followup_upload(&store, &account, &signed, legacy, pq)
                        .unwrap()
                        .is_none()
                );
            }
            pending
                .settle(
                    &store,
                    &Node::new("iq")
                        .attr("id", pending.iq_id.as_str())
                        .attr("type", "result"),
                )
                .unwrap();
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        request_key(&pending.iq_id).as_bytes(),
                        1
                    ))
                    .unwrap()
                    .unwrap(),
                checkpoint
            );
        }
    }

    #[test]
    fn offline_digest_matches_independent_sha1_vector_and_keeps_server_id_order() {
        use wacore_libsignal::protocol::{PrivateKey, Timestamp as SignalTimestamp};
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("digest-vector.redb"));
        let pair = |byte| {
            let private_key = PrivateKey::deserialize(&[byte; 32]).unwrap();
            KeyPair {
                public_key: private_key.public_key().unwrap(),
                private_key,
            }
        };
        let mut account = AccountRecord {
            signal_registration_id: Some(0x01020304),
            ..Default::default()
        };
        account.signal_identity = Some(SignalIdentityKey(KeyMaterial {
            public: [1; 32],
            private: [1; 32],
        }));
        // Structural hash fixture, not an authenticated signed-key fixture.
        let signed = SignedPreKeyRecord::new(
            7.into(),
            SignalTimestamp::from_epoch_millis(0),
            &pair(2),
            &[7; 64],
        )
        .serialize()
        .unwrap();
        store
            .write(|tx| {
                for (id, byte) in [(1u32, 3u8), (2, 4)] {
                    tx.put_json(
                        Table::Prekeys,
                        record_key(id).as_bytes(),
                        1,
                        &PreKeyRecord::new(id.into(), &pair(byte))
                            .serialize()
                            .unwrap(),
                    )?;
                }
                Ok(())
            })
            .unwrap();
        // Python hashlib + cryptography X25519 independently computed this
        // identity||signed public||signature||legacy public3||public4 vector.
        let vector: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/prekeys/2.26.43.72-digest-reference.json"
        ))
        .unwrap();
        let hash = hex::decode(vector["sha1"].as_str().unwrap()).unwrap();
        let reply = |order: &[u32]| {
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("digest").children(vec![
                    Node::new("registration").bytes(vec![1, 2, 3, 4]),
                    Node::new("type").bytes(vec![5]),
                    Node::new("skey").children(vec![Node::new("id").bytes(be3(7))]),
                    Node::new("list").children(
                        order
                            .iter()
                            .map(|id| Node::new("id").bytes(be3(*id)))
                            .collect(),
                    ),
                    Node::new("hash").bytes(hash.clone()),
                ])])
        };
        let check = |node: &Node| {
            store
                .read(|tx| digest::inspect_in(tx, &account, &signed, false, node))
                .unwrap()
        };
        let before = store.read(|tx| tx.scan(Table::Prekeys, 1, 100)).unwrap();
        assert_eq!(
            check(&reply(&[1, 2])).disposition,
            digest::Disposition::Matches
        );
        assert_eq!(
            check(&reply(&[2, 1])).disposition,
            digest::Disposition::HashMismatch
        );
        assert_eq!(
            check(&reply(&[1, 1])).disposition,
            digest::Disposition::HashMismatch
        );
        assert_eq!(
            check(&reply(&[1, 3])).disposition,
            digest::Disposition::MissingLegacyKey
        );
        let mut malformed = reply(&[1, 2]);
        let mut children = malformed.child("digest").unwrap().child_nodes().to_vec();
        children[4] = Node::new("hash").bytes(vec![0; 32]);
        malformed = malformed.children(vec![Node::new("digest").children(children)]);
        assert!(
            store
                .read(|tx| digest::inspect_in(tx, &account, &signed, false, &malformed))
                .is_err()
        );
        assert_eq!(
            store.read(|tx| tx.scan(Table::Prekeys, 1, 100)).unwrap(),
            before
        );
        let wide = SignedPreKeyRecord::new(
            0x0100_0007.into(),
            SignalTimestamp::from_epoch_millis(0),
            &pair(2),
            &[7; 64],
        )
        .serialize()
        .unwrap();
        assert!(
            store
                .read(|tx| digest::inspect_in(tx, &account, &wide, false, &reply(&[1, 2])))
                .is_err()
        );
        // Same public bytes reproduce the independent hash, but the embedded
        // row ID disagrees with the requested ID. That is invalid local state.
        store
            .write(|tx| {
                tx.put_json(
                    Table::Prekeys,
                    record_key(1).as_bytes(),
                    1,
                    &PreKeyRecord::new(2.into(), &pair(3)).serialize().unwrap(),
                )
            })
            .unwrap();
        let corrupt = store.read(|tx| tx.scan(Table::Prekeys, 1, 100)).unwrap();
        assert!(
            store
                .read(|tx| digest::inspect_in(tx, &account, &signed, false, &reply(&[1, 2])))
                .is_err()
        );
        assert_eq!(
            store.read(|tx| tx.scan(Table::Prekeys, 1, 100)).unwrap(),
            corrupt
        );
    }

    #[test]
    fn offline_pq_digest_uses_acknowledged_frozen_signature_without_resigning() {
        use sha2::Digest;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("digest-pq.redb"));
        let (account, signed) = account_and_signed(&store);
        let prop = super::super::abprops::queue(&store).unwrap();
        super::super::actions::reserve_pending(&store).unwrap();
        super::super::actions::record_reply(
            &store,
            &Node::new("iq")
                .attr("id", prop.as_str())
                .attr("type", "result")
                .children(vec![Node::new("props").attr("hash", "fixture").children(
                    vec![Node::new("prop").attr("config_code", "21198").attr("config_value", "1")],
                )]),
        )
        .unwrap();
        generate(&store).unwrap();
        let (pending, set) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        pending
            .settle(
                &store,
                &Node::new("iq")
                    .attr("id", pending.iq_id.as_str())
                    .attr("type", "result"),
            )
            .unwrap();
        let saved_rows = store
            .read(|tx| {
                pending
                    .kem_ids
                    .iter()
                    .map(|id| super::super::kem_prekeys::load(tx, *id).map(|row| row.unwrap()))
                    .collect::<Result<Vec<_>>>()
            })
            .unwrap();
        for row in &saved_rows {
            let entry = if row.last_resort {
                set.child("pq_last_resort_key").unwrap()
            } else {
                set.child("pq_list")
                    .unwrap()
                    .child_nodes()
                    .iter()
                    .find(|entry| child_bytes(entry, "id") == Some(be3(row.id).as_slice()))
                    .unwrap()
            };
            assert_eq!(row.signature.as_deref(), child_bytes(entry, "signature"));
        }
        // Build peer digest from the published wire fields, independently of
        // inspector's local-store lookup and frozen-history signature recovery.
        let skey = set.child("skey").unwrap();
        let lr = set.child("pq_last_resort_key").unwrap();
        let list = set.child("list").unwrap();
        let pq = set.child("pq_list").unwrap();
        let mut input = child_bytes(&set, "identity").unwrap().to_vec();
        input.extend_from_slice(child_bytes(skey, "value").unwrap());
        input.extend_from_slice(child_bytes(skey, "signature").unwrap());
        for key in list.child_nodes() {
            input.extend_from_slice(child_bytes(key, "value").unwrap());
        }
        input.extend_from_slice(child_bytes(lr, "value").unwrap());
        input.extend_from_slice(child_bytes(lr, "signature").unwrap());
        for key in pq.child_nodes() {
            input.extend_from_slice(child_bytes(key, "value").unwrap());
        }
        let reply =
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("digest").children(vec![
                set.child("registration").unwrap().clone(),
                Node::new("type").bytes(vec![5]),
                Node::new("skey").children(vec![skey.child("id").unwrap().clone()]),
                Node::new("list").children(
                    list.child_nodes()
                        .iter()
                        .map(|n| n.child("id").unwrap().clone())
                        .collect(),
                ),
                Node::new("hash").bytes(sha2::Sha256::digest(&input).to_vec()),
                Node::new("pq_list").children(
                    pq.child_nodes()
                        .iter()
                        .map(|n| n.child("id").unwrap().clone())
                        .collect(),
                ),
                Node::new("pq_last_resort_key").children(vec![lr.child("id").unwrap().clone()]),
            ])]);
        let snapshot = |store: &ProtocolStore| {
            store
                .read(|tx| {
                    [Table::Prekeys, Table::Checkpoints]
                        .into_iter()
                        .map(|t| tx.scan(t, 1, 10000))
                        .collect::<Result<Vec<_>>>()
                })
                .unwrap()
        };
        let before = snapshot(&store);
        let check = || {
            store
                .read(|tx| digest::inspect_in(tx, &account, &signed, true, &reply))
                .unwrap()
        };
        assert_eq!(check().disposition, digest::Disposition::Matches);
        assert_eq!(
            check().disposition,
            digest::Disposition::Matches,
            "randomized re-signing would fail this"
        );
        assert_eq!(snapshot(&store), before);
        store
            .write(|tx| tx.delete(Table::Checkpoints, request_key(&pending.iq_id).as_bytes()))
            .unwrap();
        assert_eq!(
            check().disposition,
            digest::Disposition::Matches,
            "new row retains exact published signature without frozen history"
        );
        let lr_id = pending
            .kem_ids
            .iter()
            .find(|id| {
                store
                    .read(|tx| super::super::kem_prekeys::load(tx, **id))
                    .unwrap()
                    .is_some_and(|r| r.last_resort)
            })
            .copied()
            .unwrap();
        store
            .write(|tx| {
                let key = format!("{}{}", super::super::kem_prekeys::RECORD_PREFIX, lr_id);
                let mut row: super::super::kem_prekeys::StoredKemPrekey =
                    tx.get_json(Table::Prekeys, key.as_bytes(), 1)?.unwrap();
                row.signature = None; // historical synthetic fixture; no production backfill/reset
                tx.put_json(Table::Prekeys, key.as_bytes(), 1, &row)
            })
            .unwrap();
        assert_eq!(
            check().disposition,
            digest::Disposition::UnavailableLastResortSignature
        );
    }

    #[test]
    fn prekey_counts_match_executed_java_integer_reference() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/prekeys/2.26.37.71-count-reference.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let text = case["input"].as_str().unwrap();
            let expected = case["valid"]
                .as_bool()
                .unwrap()
                .then(|| case["parsed"].as_str().unwrap().parse::<i32>().unwrap());
            let root = Node::new("notification").attr("type", "encrypt");
            let legacy = root
                .clone()
                .children(vec![Node::new("count").attr("value", text)]);
            assert_eq!(
                low_count(&legacy).unwrap(),
                Some(LowCount {
                    legacy_present: true,
                    legacy: expected,
                    pq: None
                }),
                "legacy {text:?}"
            );
            for children in [
                vec![
                    Node::new("pq_count").attr("value", text),
                    Node::new("count").attr("value", "7"),
                ],
                vec![
                    Node::new("count").attr("value", "7"),
                    Node::new("pq_count").attr("value", text),
                ],
            ] {
                let legacy_present = children[0].description == "count";
                let result = low_count(&root.clone().children(children));
                if let Some(value) = expected {
                    assert_eq!(
                        result.unwrap(),
                        Some(LowCount {
                            legacy_present,
                            legacy: legacy_present.then_some(7),
                            pq: Some(value)
                        }),
                        "PQ {text:?}"
                    );
                } else {
                    let diagnostic = result.unwrap_err().to_string();
                    assert!(
                        diagnostic.contains("pq_count value is not a signed 32-bit Java decimal")
                    );
                }
            }
        }
    }

    #[test]
    fn prekey_count_dispatch_and_missing_values_preserve_native_distinctions() {
        let root = Node::new("notification").attr("type", "encrypt");
        for first in ["identity", "other"] {
            assert_eq!(
                low_count(&root.clone().children(vec![
                    Node::new(first),
                    Node::new("count").attr("value", "7")
                ]))
                .unwrap(),
                None
            );
        }
        assert_eq!(
            low_count(&root.clone().children(vec![Node::new("count")])).unwrap(),
            Some(LowCount {
                legacy_present: true,
                legacy: None,
                pq: None
            })
        );
        assert_eq!(
            low_count(
                &root
                    .clone()
                    .children(vec![Node::new("count"), Node::new("pq_count")])
            )
            .unwrap(),
            Some(LowCount {
                legacy_present: true,
                legacy: None,
                pq: Some(0)
            })
        );
        assert_eq!(
            low_count(&root.clone().children(vec![
                Node::new("pq_count"),
                Node::new("count").attr("value", "7")
            ]))
            .unwrap(),
            Some(LowCount {
                legacy_present: false,
                legacy: None,
                pq: Some(0)
            })
        );
        let canary = "private-count-canary";
        let error = low_count(
            &root
                .clone()
                .children(vec![Node::new("pq_count").attr("value", canary)]),
        )
        .unwrap_err()
        .to_string();
        assert!(!error.contains(canary));
        let invalid = crate::binary::AttributeValue::Bytes(bytes::Bytes::from_static(b"\xff"));
        assert_eq!(
            low_count(
                &root
                    .clone()
                    .children(vec![Node::new("count").attr("value", invalid.clone())])
            )
            .unwrap()
            .unwrap()
            .legacy,
            None
        );
        assert!(
            low_count(&root.children(vec![Node::new("pq_count").attr("value", invalid)])).is_err()
        );
    }
    #[test]
    fn count_notifications_are_recognized_and_acknowledged() {
        let from = Jid::parse("s.whatsapp.net").unwrap();
        let node = Node::new("notification")
            .attr("type", "encrypt")
            .attr("id", "N1")
            .attr("from", from.clone())
            .children(vec![Node::new("count").attr("value", "9")]);
        assert_eq!(
            low_count(&node).unwrap(),
            Some(LowCount {
                legacy_present: true,
                legacy: Some(9),
                pq: None
            })
        );
        let ack = low_count_ack(&node).unwrap();
        assert_eq!(ack.description, "ack");
        assert_eq!(ack.get_attr("id").and_then(|v| v.as_text()), Some("N1"));
        assert_eq!(
            ack.get_attr("class").and_then(|v| v.as_text()),
            Some("notification")
        );
        assert_eq!(
            ack.get_attr("type").and_then(|v| v.as_text()),
            Some("encrypt")
        );
        let pq_only = Node::new("notification")
            .attr("type", "encrypt")
            .attr("id", "N2")
            .children(vec![Node::new("pq_count").attr("value", "3")]);
        assert_eq!(
            low_count(&pq_only).unwrap(),
            Some(LowCount {
                legacy_present: false,
                legacy: None,
                pq: Some(3)
            })
        );
        for other in [
            Node::new("notification")
                .attr("type", "encrypt")
                .attr("id", "N3"),
            Node::new("notification")
                .attr("type", "privacy_token")
                .children(vec![Node::new("count").attr("value", "9")]),
            Node::new("notification")
                .attr("type", "encrypt")
                .attr("participant", "x")
                .children(vec![Node::new("count").attr("value", "9")]),
            Node::new("iq").children(vec![Node::new("count").attr("value", "9")]),
        ] {
            assert_eq!(low_count(&other).unwrap(), None, "{other:?}");
        }
        assert!(low_count_ack(&Node::new("notification").attr("type", "encrypt")).is_err());
    }
}
