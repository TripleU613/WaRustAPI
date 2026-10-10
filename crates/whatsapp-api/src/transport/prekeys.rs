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
//!   value,signature}`, then optional `pq_list`/`pq_last_resort_key` when
//!   Kyber keys exist. `LX/13c.A03` `00d6..00e4` logs "skipping kyber prekeys
//!   upload" without them, so the post-quantum children are optional; this
//!   client has no Kyber material and never sends them. The conditional `op`
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
//! Local policy: a batch is reserved durably before its socket write; keys in
//! an uncertain batch are never re-sent. Startup generates first when nothing
//! is unsent and then uploads in the same session (the APK does these on
//! separate runs). Follow-up uploads after `count_low` are capped per session.
use crate::{
    Error, Result,
    binary::Node,
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
    /// Sanitized server error code for a refusal.
    pub code: Option<String>,
    /// Server still wants more legacy keys (`list/@count_low`).
    pub count_low: Option<bool>,
    /// Reservation time.
    pub reserved_at: Timestamp,
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
    store.write(|tx| {
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
    })
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

/// A reserved upload awaiting the server's `iq` reply for this connection.
#[derive(Debug)]
pub struct PendingUpload {
    iq_id: String,
    ids: Vec<u32>,
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
}

/// Reserve up to 812 unsent keys and build the upload IQ. `signed` is the
/// serialized registered signed prekey record. Returns `None` with nothing
/// unsent; a `Some` reservation is durable before the caller writes anything.
pub fn begin_upload(
    store: &ProtocolStore,
    account: &AccountRecord,
    signed: &[u8],
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
    let signed_public = signed
        .public_key()
        .map_err(|_| invalid("signed prekey public key"))?;
    let signed_signature = signed
        .signature()
        .map_err(|_| invalid("signed prekey signature"))?;
    let iq_id = super::messaging::new_message_id()?;
    let reserved = store.write(|tx| {
        let ids: Vec<u32> = unsent_in(&*tx)?.into_iter().take(UNSENT_TARGET).collect();
        if ids.is_empty() {
            return Ok(None);
        }
        let mut keys = Vec::with_capacity(ids.len());
        for id in &ids {
            let record: Vec<u8> = tx
                .get_json(Table::Prekeys, record_key(*id).as_bytes(), 1)?
                .ok_or_else(|| invalid("unsent prekey record disappeared"))?;
            let record =
                PreKeyRecord::deserialize(&record).map_err(|_| invalid("invalid prekey record"))?;
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
        tx.put_json(
            Table::Checkpoints,
            batch_key(&iq_id).as_bytes(),
            1,
            &Batch {
                ids: ids.clone(),
                status: "outcome_unknown".into(),
                code: None,
                count_low: None,
                reserved_at: Timestamp::now(),
            },
        )?;
        Ok(Some((ids, keys)))
    })?;
    let Some((ids, keys)) = reserved else {
        return Ok(None);
    };
    let node = Node::new("iq")
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
        ]);
    Ok(Some((
        PendingUpload {
            iq_id,
            ids,
            deadline: tokio::time::Instant::now() + std::time::Duration::from_secs(30),
        },
        node,
    )))
}

impl PendingUpload {
    /// Only the exact `iq` reply for this batch settles it.
    pub fn matches(&self, node: &Node) -> bool {
        node.description == "iq"
            && node.get_attr("id").and_then(|v| v.as_text()) == Some(self.iq_id.as_str())
    }
    /// Apply the server reply durably: acknowledged keys become `Uploaded`;
    /// a refusal returns the keys to `Unsent` (APK keeps `sent_to_server = 0`)
    /// and records the sanitized code. Nothing is retried here.
    pub fn settle(&self, store: &ProtocolStore, node: &Node) -> Result<UploadOutcome> {
        if !self.matches(node) {
            return Err(invalid("reply does not belong to this upload"));
        }
        let accepted = node.get_attr("type").and_then(|v| v.as_text()) == Some("result");
        let count_low = node
            .child_nodes()
            .iter()
            .find(|c| c.description == "list")
            .and_then(|c| c.get_attr("count_low"))
            .and_then(|v| v.as_text())
            .is_some_and(|v| v.eq_ignore_ascii_case("true"));
        let code = node
            .child_nodes()
            .iter()
            .find(|c| c.description == "error")
            .and_then(|c| c.get_attr("code"))
            .and_then(|v| v.as_text())
            .map(|v| v.chars().take(8).collect::<String>());
        store.write(|tx| {
            let key = batch_key(&self.iq_id);
            let mut batch: Batch = tx
                .get_json(Table::Checkpoints, key.as_bytes(), 1)?
                .ok_or_else(|| invalid("reserved batch disappeared"))?;
            if batch.status != "outcome_unknown" {
                return Err(invalid("batch already settled"));
            }
            batch.status = if accepted {
                "server_acknowledged".into()
            } else {
                "refused".into()
            };
            batch.code = code.clone();
            batch.count_low = Some(count_low);
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &batch)?;
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
/// A malformed present PQ count refuses before ACK/upload; legacy count uses
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
    let to = super::receipts::jid(node, "from").unwrap_or(Jid::parse("s.whatsapp.net")?);
    Ok(Node::new("ack")
        .attr("to", to)
        .attr("id", id)
        .attr("class", "notification")
        .attr("type", "encrypt"))
}

/// Whether a session should upload now: something is unsent, or nothing has
/// ever been generated (then `generate` first, mirroring `LX/DNq.run`).
pub fn wants_upload(store: &ProtocolStore) -> Result<bool> {
    store.read(|tx| Ok(!unsent_in(tx)?.is_empty() || record_ids(tx)?.is_empty()))
}

/// Redacted counts for operators; no key material is read.
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
            *summary.batches.entry(batch.status).or_default() += 1;
        }
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

    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn account_and_signed() -> (AccountRecord, Vec<u8>) {
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
        (
            AccountRecord {
                registration: RegistrationStatus::Registered,
                signal_registration_id: Some(0x0102_0304),
                signal_identity: Some(SignalIdentityKey(KeyMaterial {
                    public,
                    private: *identity.private_key.serialize(),
                })),
                ..Default::default()
            },
            record.serialize().unwrap(),
        )
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

    #[test]
    fn upload_iq_matches_traced_layout_and_settles_by_reply() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("upload.redb");
        let store = open(&path);
        let (account, signed) = account_and_signed();
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
                count_low: true
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
                count_low: false
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
        assert!(wants_upload(&store).unwrap());
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
