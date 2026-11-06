//! Local ML-KEM-1024 prekey inventory read by the normal inbox.
//!
//! Native evidence, pinned 2.26.37.71 (SHA-256 `1755111a…7d5c`):
//! REA `0kh.BTS` selects `record` from `kyber_prekeys` by `prekey_id` and
//! throws when the row is absent; `0kh.BWa` deletes only rows matching
//! `prekey_id` **and** `last_resort_key = 0` (docs/protocol/hybrid-incoming.md).
//! Successful last-resort use therefore retains the key.
//!
//! Rows are produced by [`upload_rows_in`] and migrated [`add_rows_in`]
//! only while the server's A/B
//! property 21198 (`PQ_KEYS_UPLOAD_CODE`) is on, as 2.26.37.71 `13c.A03` does
//! (docs/protocol/kem-prekey-inventory.md). Native generation and the initial
//! `next_kyber_prekey_id` are untraced; ids start at 1 locally.
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{
    Error, Result,
    store::{ReadTx, Table, WriteTx},
};

/// Prefix kept distinct from ordinary `comparison-prekey:` rows, whose prefix
/// scans (upload batches and counts) must never see KEM records.
pub(crate) const RECORD_PREFIX: &str = "comparison-kem-prekey:";
/// ML-KEM-1024 encapsulation key (FIPS 203: 384·k + 32, k = 4).
const PUBLIC_LEN: usize = 1568;
/// Expanded decapsulation key `dk_pke ‖ ek ‖ H(ek) ‖ z`; the APK stores this
/// 3168-byte form, not the 64-byte seed (docs/protocol/rust-kem.md).
const SECRET_LEN: usize = 3168;
/// Offset of the embedded `ek` inside the expanded key (`dk_pke` = 384·k).
const EMBEDDED_PUBLIC: usize = 1536;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("kem_prekey_inventory", reason)
}

pub(crate) fn record_key(id: u32) -> String {
    format!("{RECORD_PREFIX}{id}")
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConsumedKem {
    id: u32,
    public_sha256: [u8; 32],
}
fn consumed_key(id: u32) -> String {
    format!("kem-prekey-consumed:v1:{id}")
}

/// One account-local KEM prekey row. Encrypted at rest by the store; the
/// secret is zeroized on drop and never printed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredKemPrekey {
    /// Native uint32 `prekey_id`, equal to the row key.
    pub id: u32,
    /// Encapsulation key as published.
    pub public: Vec<u8>,
    /// Exact identity signature published for this row; absent on historical
    /// rows or newly generated inventory not yet reserved for upload.
    #[serde(default)]
    pub signature: Option<Vec<u8>>,
    /// Expanded decapsulation key.
    pub secret: Vec<u8>,
    /// Native `last_resort_key`; such keys survive successful use.
    pub last_resort: bool,
    /// Native `sent_to_server`: set once an upload batch carrying it was
    /// acknowledged. Rows written before the producer existed default false.
    #[serde(default)]
    pub sent: bool,
}
impl Drop for StoredKemPrekey {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}
impl std::fmt::Debug for StoredKemPrekey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StoredKemPrekey(<redacted>)")
    }
}
impl StoredKemPrekey {
    /// Local integrity only: row key, lengths and the public key embedded in
    /// the expanded secret. This is not a claim of an additional native check.
    fn validate(&self, id: u32) -> Result<()> {
        if self.id != id {
            return Err(invalid("row ID does not match its key"));
        }
        if self.signature.as_ref().is_some_and(|s| s.len() != 64) {
            return Err(invalid("unexpected KEM signature length"));
        }
        if self.public.len() != PUBLIC_LEN || self.secret.len() != SECRET_LEN {
            return Err(invalid("unexpected ML-KEM-1024 key length"));
        }
        if self.secret[EMBEDDED_PUBLIC..EMBEDDED_PUBLIC + PUBLIC_LEN] != self.public[..] {
            return Err(invalid("public key does not match expanded secret"));
        }
        Ok(())
    }
    /// Borrow as the hybrid receive input. The inbox, not the sender, decides
    /// last-resort retention.
    pub(crate) fn local(&self) -> crate::signal::hybrid::LocalKemPrekey<'_> {
        crate::signal::hybrid::LocalKemPrekey {
            id: self.id,
            secret: &self.secret,
            last_resort: self.last_resort,
        }
    }
}

/// The row an incoming envelope names, validated; `None` when absent. A row
/// that does not parse is reported like any other invalid row, never as a
/// store failure that would end the session.
pub(crate) fn load(tx: &ReadTx<'_>, id: u32) -> Result<Option<StoredKemPrekey>> {
    let Some(bytes) = tx.get(Table::Prekeys, record_key(id).as_bytes(), 1)? else {
        return Ok(None);
    };
    // Restoring a stale row must not override authenticated consumption.
    if tx
        .get(Table::Checkpoints, consumed_key(id).as_bytes(), 1)?
        .is_some()
    {
        return Err(invalid("live row conflicts with consumption marker"));
    }
    let row: StoredKemPrekey =
        serde_json::from_slice(&bytes).map_err(|_| invalid("row does not parse"))?;
    row.validate(id)?;
    Ok(Some(row))
}

/// Delete a consumed non-last-resort key inside the inbox commit, refusing
/// when the row changed since the candidate was authenticated. Mirrors the
/// ordinary one-time prekey guard; a last-resort row is never deleted here.
pub(crate) fn consume_in(tx: &mut WriteTx<'_>, id: u32, original: &StoredKemPrekey) -> Result<()> {
    let key = record_key(id);
    let current: Option<StoredKemPrekey> = tx.get_json(Table::Prekeys, key.as_bytes(), 1)?;
    if current.as_ref() != Some(original) || original.id != id || original.last_resort {
        return Err(Error::invalid(
            "inbox",
            "KEM prekey changed before commit; not consumed",
        ));
    }
    if tx
        .get(Table::Checkpoints, consumed_key(id).as_bytes(), 1)?
        .is_some()
    {
        return Err(invalid("live row conflicts with consumption marker"));
    }
    // The inbox commits this marker with authenticated consumption. It carries
    // no private material and distinguishes use-before-upload-ACK from loss.
    use sha2::{Digest, Sha256};
    tx.put_json(
        Table::Checkpoints,
        consumed_key(id).as_bytes(),
        1,
        &ConsumedKem {
            id,
            public_sha256: Sha256::digest(&original.public).into(),
        },
    )?;
    tx.delete(Table::Prekeys, key.as_bytes())?;
    Ok(())
}

/// Redacted counts for operators: (all rows, last-resort rows). Every row
/// under the prefix counts; one that does not parse is never last-resort.
pub(crate) fn counts(tx: &ReadTx<'_>) -> Result<(usize, usize)> {
    #[derive(Deserialize)]
    struct Flag {
        last_resort: bool,
    }
    let (mut total, mut last_resort) = (0, 0);
    let mut cursor: Option<Vec<u8>> = None;
    loop {
        let rows = tx.scan_after(Table::Prekeys, 1, cursor.as_deref(), 512)?;
        let done = rows.len() < 512;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows {
            if key.starts_with(RECORD_PREFIX.as_bytes())
                && let Some(bytes) = tx.get(Table::Prekeys, &key, 1)?
            {
                total += 1;
                last_resort += usize::from(
                    serde_json::from_slice::<Flag>(&bytes).is_ok_and(|flag| flag.last_resort),
                );
            }
        }
        if done {
            return Ok((total, last_resort));
        }
    }
}

/// Next KEM prekey id (`identities.next_kyber_prekey_id` in the APK; its
/// initial value is untraced, so starting at 1 is local, **inferred** policy).
const NEXT_ID: &[u8] = b"kem-next-prekey-id";

fn all_rows(tx: &WriteTx<'_>) -> Result<Vec<StoredKemPrekey>> {
    let mut out = Vec::new();
    let mut cursor: Option<Vec<u8>> = None;
    loop {
        let rows = tx.scan_after(Table::Prekeys, 1, cursor.as_deref(), 512)?;
        let done = rows.len() < 512;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, bytes) in rows {
            if key.starts_with(RECORD_PREFIX.as_bytes()) {
                // Batch selection must not turn corrupt owned rows into an
                // apparently missing inventory and generate around them.
                let row: StoredKemPrekey =
                    serde_json::from_slice(&bytes).map_err(|_| invalid("row does not parse"))?;
                if key != record_key(row.id).as_bytes() {
                    return Err(invalid("row ID does not match its key"));
                }
                if tx
                    .get(Table::Checkpoints, consumed_key(row.id).as_bytes(), 1)?
                    .is_some()
                {
                    return Err(invalid("live row conflicts with consumption marker"));
                }
                row.validate(row.id)?;
                out.push(row);
            }
        }
        if done {
            return Ok(out);
        }
    }
}

fn generate_in(tx: &mut WriteTx<'_>, last_resort: bool) -> Result<()> {
    let id: u32 = tx.get_json(Table::Checkpoints, NEXT_ID, 1)?.unwrap_or(1);
    if id > 0x00ff_ffff {
        return Err(invalid("KEM prekey id space exhausted"));
    }
    if tx
        .get_json::<ConsumedKem>(Table::Checkpoints, consumed_key(id).as_bytes(), 1)?
        .is_some()
        || tx
            .get(Table::Prekeys, record_key(id).as_bytes(), 1)?
            .is_some()
    {
        return Err(invalid(
            "KEM prekey id collision; refusing to overwrite or reuse",
        ));
    }
    let (public, secret) = crate::signal::kem::generate_key_pair()?;
    let row = StoredKemPrekey {
        id,
        public,
        secret: secret.as_bytes().to_vec(),
        last_resort,
        sent: false,
        signature: None,
    };
    row.validate(id)?;
    tx.put_json(Table::Prekeys, record_key(id).as_bytes(), 1, &row)?;
    tx.put_json(Table::Checkpoints, NEXT_ID, 1, &(id + 1))
}

/// The Kyber rows for one upload, generating what is missing.
///
/// **Confirmed static, 2.26.37.71:** `13c.A03` (`sendSetPreKey`) adds Kyber
/// keys only when `Can.A02()` (property 21198) is true and `0jc.A0c` finds an
/// *unsent* last-resort key; without one it logs "skipping kyber prekeys
/// upload". `D4s` generates a last-resort key when none exists. The one-time
/// batch is `0jc.A1C`: up to `max(1, property 21201)` unsent rows, default 10
/// (`07t.A0R 0196..01a2`). Generation itself is native and untraced; this uses
/// the same ML-KEM-1024 primitive the receive path verifies against.
/// Returns no rows once the last-resort key has been sent (nothing to upload).
pub(crate) fn upload_rows_in(tx: &mut WriteTx<'_>, batch: usize) -> Result<Vec<StoredKemPrekey>> {
    let rows = all_rows(tx)?;
    if !rows.iter().any(|r| r.last_resort) {
        generate_in(tx, true)?;
    } else if !rows.iter().any(|r| r.last_resort && !r.sent) {
        return Ok(Vec::new());
    }
    let unsent = rows.iter().filter(|r| !r.last_resort && !r.sent).count();
    for _ in unsent..batch.max(1) {
        generate_in(tx, false)?;
    }
    let rows = all_rows(tx)?;
    let mut one_time: Vec<_> = rows
        .iter()
        .filter(|r| !r.last_resort && !r.sent)
        .cloned()
        .collect();
    one_time.truncate(batch.max(1));
    one_time.extend(
        rows.into_iter()
            .filter(|r| r.last_resort && !r.sent)
            .take(1),
    );
    Ok(one_time)
}

/// Current-build migration predicate is an uploaded last-resort row
/// (43.72 0WJ.A0z). The caller separately applies gate21198.
pub(crate) fn migrated_in(tx: &WriteTx<'_>) -> Result<bool> {
    Ok(all_rows(tx)?.iter().any(|r| r.last_resort && r.sent))
}

/// ADD uses ordinary unsent KEM rows only. 43.72 0Yl.A06 fills the
/// configured inventory target; 0WJ.A1A selects an independent upload limit.
/// Local persisted ID convention is retained; native cursor migration is open.
pub(crate) fn add_rows_in(
    tx: &mut WriteTx<'_>,
    target: usize,
    batch: usize,
) -> Result<Vec<StoredKemPrekey>> {
    let count = all_rows(tx)?
        .iter()
        .filter(|r| !r.last_resort && !r.sent)
        .count();
    for _ in count..target {
        generate_in(tx, false)?;
    }
    let mut rows: Vec<_> = all_rows(tx)?
        .into_iter()
        .filter(|r| !r.last_resort && !r.sent)
        .collect();
    rows.truncate(batch.max(1));
    for row in &rows {
        row.validate(row.id)?;
    }
    Ok(rows)
}

/// Bind publication bytes in the existing upload reservation transaction.
/// Never replace an existing signature or infer one for an uploaded row.
pub(crate) fn retain_signature_in(
    tx: &mut WriteTx<'_>,
    original: &StoredKemPrekey,
    signature: &[u8],
) -> Result<()> {
    if signature.len() != 64 {
        return Err(invalid("unexpected KEM signature length"));
    }
    let key = record_key(original.id);
    let current: StoredKemPrekey = tx
        .get_json(Table::Prekeys, key.as_bytes(), 1)?
        .ok_or_else(|| invalid("KEM row disappeared before signature binding"))?;
    if &current != original {
        return Err(invalid("KEM row changed before signature binding"));
    }
    if let Some(existing) = &current.signature {
        if existing != signature {
            return Err(invalid("KEM published signature changed"));
        }
        return Ok(());
    }
    if current.sent {
        return Err(invalid("cannot infer historical uploaded KEM signature"));
    }
    let mut bound = current.clone();
    bound.signature = Some(signature.to_vec());
    tx.put_json(Table::Prekeys, key.as_bytes(), 1, &bound)
}

/// `sent_to_server = 1` for the rows of an acknowledged batch.
pub(crate) fn mark_sent_in(tx: &mut WriteTx<'_>, ids: &[u32]) -> Result<()> {
    for id in ids {
        let key = record_key(*id);
        // Ordinary keys may be authenticated/consumed before the upload reply.
        // Only the inbox's durable marker explains a missing row; never invent
        // consumption for historical missing material or recreate its secret.
        let Some(mut row) = tx.get_json::<StoredKemPrekey>(Table::Prekeys, key.as_bytes(), 1)?
        else {
            if tx
                .get_json::<ConsumedKem>(Table::Checkpoints, consumed_key(*id).as_bytes(), 1)?
                .is_some_and(|consumed| consumed.id == *id)
            {
                continue;
            }
            return Err(invalid(
                "reserved KEM prekey disappeared before acknowledgement",
            ));
        };
        if tx
            .get(Table::Checkpoints, consumed_key(*id).as_bytes(), 1)?
            .is_some()
        {
            return Err(invalid("live row conflicts with consumption marker"));
        }
        row.validate(*id)?;
        row.sent = true;
        tx.put_json(Table::Prekeys, key.as_bytes(), 1, &row)?;
    }
    Ok(())
}

/// Test-only writer of a chosen row.
#[cfg(test)]
pub(crate) fn install_for_test(
    tx: &mut WriteTx<'_>,
    id: u32,
    public: &[u8],
    secret: &[u8],
    last_resort: bool,
) -> Result<StoredKemPrekey> {
    let row = StoredKemPrekey {
        id,
        public: public.to_vec(),
        secret: secret.to_vec(),
        last_resort,
        sent: false,
        signature: None,
    };
    row.validate(id)?;
    tx.put_json(Table::Prekeys, record_key(id).as_bytes(), 1, &row)?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        signal::kem,
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };

    #[test]
    fn consumed_kem_row_restoration_cannot_restore_receive_or_publication_authority() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kem-consumed-restoration.redb");
        let reopen = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("kem-inventory").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [9; 32],
                ))),
                create: true,
            })
            .unwrap()
        };
        let store = reopen();
        let (public, secret) = kem::generate_key_pair().unwrap();
        let original = store
            .write(|tx| install_for_test(tx, 5, &public, secret.as_bytes(), false))
            .unwrap();
        store.write(|tx| consume_in(tx, 5, &original)).unwrap();
        assert!(store.read(|tx| load(tx, 5)).unwrap().is_none());
        // Normal use-before-upload-result remains settleable without restoring a key.
        store.write(|tx| mark_sent_in(tx, &[5])).unwrap();
        store
            .write(|tx| tx.put_json(Table::Prekeys, record_key(5).as_bytes(), 1, &original))
            .unwrap();
        let snapshot = store
            .read(|tx| {
                Ok((
                    tx.scan(Table::Prekeys, 1, 100)?,
                    tx.scan(Table::Checkpoints, 1, 100)?,
                ))
            })
            .unwrap();
        assert!(
            store.read(|tx| load(tx, 5)).is_err(),
            "restored row cannot erase consumption"
        );
        assert!(store.write(|tx| consume_in(tx, 5, &original)).is_err());
        assert!(store.write(|tx| migrated_in(tx)).is_err());
        assert!(store.write(|tx| upload_rows_in(tx, 10)).is_err());
        assert!(store.write(|tx| add_rows_in(tx, 20, 10)).is_err());
        assert!(store.write(|tx| mark_sent_in(tx, &[5])).is_err());
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan(Table::Prekeys, 1, 100)?,
                    tx.scan(Table::Checkpoints, 1, 100)?
                )))
                .unwrap(),
            snapshot
        );
        drop(store);
        let store = reopen();
        assert!(store.read(|tx| load(tx, 5)).is_err());
        assert!(store.write(|tx| mark_sent_in(tx, &[5])).is_err());
        assert_eq!(
            store
                .read(|tx| Ok((
                    tx.scan(Table::Prekeys, 1, 100)?,
                    tx.scan(Table::Checkpoints, 1, 100)?
                )))
                .unwrap(),
            snapshot
        );
    }

    #[test]
    fn batch_inventory_refuses_corrupt_owned_rows_before_generation_or_migration() {
        let (public, secret) = kem::generate_key_pair().unwrap();
        for case in ["json", "wrong-id", "alias", "utf8", "secret"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("kem-batch-integrity.redb");
            let reopen = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("kem-inventory").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [9; 32],
                    ))),
                    create: true,
                })
                .unwrap()
            };
            let store = reopen();
            store
                .write(|tx| install_for_test(tx, 5, &public, secret.as_bytes(), false))
                .unwrap();
            store
                .write(|tx| {
                    let mut row: StoredKemPrekey = tx
                        .get_json(Table::Prekeys, record_key(5).as_bytes(), 1)?
                        .unwrap();
                    tx.delete(Table::Prekeys, record_key(5).as_bytes())?;
                    let mut key = record_key(5).into_bytes();
                    match case {
                        "json" => return tx.put(Table::Prekeys, &key, 1, b"{"),
                        "wrong-id" => row.id = 6,
                        "alias" => key = format!("{RECORD_PREFIX}05").into_bytes(),
                        "utf8" => key.push(255),
                        "secret" => {
                            row.secret[EMBEDDED_PUBLIC] ^= 1;
                            row.last_resort = true;
                            row.sent = true;
                        }
                        _ => unreachable!(),
                    }
                    tx.put_json(Table::Prekeys, &key, 1, &row)
                })
                .unwrap();
            let snapshot = store
                .read(|tx| {
                    Ok((
                        tx.scan(Table::Prekeys, 1, 100)?,
                        tx.scan(Table::Checkpoints, 1, 100)?,
                    ))
                })
                .unwrap();
            assert!(
                store.write(|tx| migrated_in(tx)).is_err(),
                "{case}: no partial migration view"
            );
            assert!(
                store.write(|tx| upload_rows_in(tx, 10)).is_err(),
                "{case}: no regeneration around corrupt row"
            );
            assert!(store.write(|tx| add_rows_in(tx, 20, 10)).is_err(), "{case}");
            assert_eq!(
                store
                    .read(|tx| Ok((
                        tx.scan(Table::Prekeys, 1, 100)?,
                        tx.scan(Table::Checkpoints, 1, 100)?
                    )))
                    .unwrap(),
                snapshot
            );
            drop(store);
            let store = reopen();
            assert!(store.write(|tx| migrated_in(tx)).is_err());
            assert_eq!(
                store
                    .read(|tx| Ok((
                        tx.scan(Table::Prekeys, 1, 100)?,
                        tx.scan(Table::Checkpoints, 1, 100)?
                    )))
                    .unwrap(),
                snapshot
            );
        }
    }

    #[test]
    fn rows_validate_count_and_consume_only_unchanged_ordinary_keys() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("kem.redb"),
            account_id: AccountId::new("kem-inventory").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap();
        let (public, secret) = kem::generate_key_pair().unwrap();
        // The expanded-key layout this module relies on, from the primitive itself.
        assert_eq!(
            &secret.as_bytes()[EMBEDDED_PUBLIC..EMBEDDED_PUBLIC + PUBLIC_LEN],
            public.as_slice()
        );
        let (other_public, _) = kem::generate_key_pair().unwrap();
        store
            .write(|tx| {
                assert!(install_for_test(tx, 5, &other_public, secret.as_bytes(), false).is_err());
                install_for_test(tx, 5, &public, secret.as_bytes(), false)?;
                install_for_test(tx, 6, &public, secret.as_bytes(), true)?;
                tx.put_json(Table::Prekeys, b"comparison-prekey:5", 1, &vec![1u8])?;
                // A row whose ID disagrees with its key must not be used.
                let mut moved: StoredKemPrekey = tx
                    .get_json(Table::Prekeys, record_key(5).as_bytes(), 1)?
                    .unwrap();
                moved.id = 8;
                tx.put_json(Table::Prekeys, record_key(9).as_bytes(), 1, &moved)
            })
            .unwrap();
        assert_eq!(store.read(counts).unwrap(), (3, 1));
        {
            use super::super::prekeys::{KemRowState, kem_row_state};
            assert_eq!(kem_row_state(&store, 5).unwrap(), KemRowState::Held);
            assert_eq!(kem_row_state(&store, 6).unwrap(), KemRowState::LastResort);
            assert_eq!(kem_row_state(&store, 7).unwrap(), KemRowState::Missing);
            assert_eq!(kem_row_state(&store, 9).unwrap(), KemRowState::Invalid);
        }
        assert!(store.read(|tx| load(tx, 9)).is_err());
        assert!(store.read(|tx| load(tx, 7)).unwrap().is_none());
        let ordinary = store.read(|tx| load(tx, 5)).unwrap().unwrap();
        let last_resort = store.read(|tx| load(tx, 6)).unwrap().unwrap();
        assert_eq!(format!("{ordinary:?}"), "StoredKemPrekey(<redacted>)");
        assert!(store.write(|tx| consume_in(tx, 6, &last_resort)).is_err());
        assert!(store.write(|tx| consume_in(tx, 6, &ordinary)).is_err());
        let mut changed = ordinary.clone();
        changed.last_resort = true;
        assert!(store.write(|tx| consume_in(tx, 5, &changed)).is_err());
        let failed: Result<()> = store.write(|tx| {
            consume_in(tx, 5, &ordinary)?;
            Err(invalid("synthetic inbox commit cut"))
        });
        assert!(failed.is_err());
        assert!(store.read(|tx| load(tx, 5)).unwrap().is_some());
        assert!(
            store
                .read(|tx| tx.get_json::<ConsumedKem>(
                    Table::Checkpoints,
                    consumed_key(5).as_bytes(),
                    1
                ))
                .unwrap()
                .is_none()
        );
        store.write(|tx| consume_in(tx, 5, &ordinary)).unwrap();
        // Even a stale allocator checkpoint cannot reuse a consumed key ID.
        store
            .write(|tx| tx.put_json(Table::Checkpoints, NEXT_ID, 1, &5u32))
            .unwrap();
        assert!(store.write(|tx| generate_in(tx, false)).is_err());
        assert!(store.read(|tx| load(tx, 5)).unwrap().is_none());
        assert!(store.read(|tx| load(tx, 5)).unwrap().is_none());
        assert!(store.read(|tx| load(tx, 6)).unwrap().is_some());
        // The ordinary one-time row sharing the numeric ID is a different key.
        assert_eq!(
            store
                .read(|tx| tx.get_json::<Vec<u8>>(Table::Prekeys, b"comparison-prekey:5", 1))
                .unwrap(),
            Some(vec![1])
        );
        assert!(store.write(|tx| consume_in(tx, 5, &ordinary)).is_err());
    }
}
