//! Local ML-KEM-1024 prekey inventory read by the normal inbox.
//!
//! Native evidence, pinned 2.26.37.71 (SHA-256 `1755111a…7d5c`):
//! REA `0kh.BTS` selects `record` from `kyber_prekeys` by `prekey_id` and
//! throws when the row is absent; `0kh.BWa` deletes only rows matching
//! `prekey_id` **and** `last_resort_key = 0` (docs/protocol/hybrid-incoming.md).
//! Successful last-resort use therefore retains the key.
//!
//! Rows are produced by [`upload_rows_in`] only while the server's A/B
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

/// One account-local KEM prekey row. Encrypted at rest by the store; the
/// secret is zeroized on drop and never printed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredKemPrekey {
    /// Native uint32 `prekey_id`, equal to the row key.
    pub id: u32,
    /// Encapsulation key as published.
    pub public: Vec<u8>,
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
            if key.starts_with(RECORD_PREFIX.as_bytes())
                && let Ok(row) = serde_json::from_slice::<StoredKemPrekey>(&bytes)
            {
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
    let (public, secret) = crate::signal::kem::generate_key_pair()?;
    let row = StoredKemPrekey {
        id,
        public,
        secret: secret.as_bytes().to_vec(),
        last_resort,
        sent: false,
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

/// `sent_to_server = 1` for the rows of an acknowledged batch.
pub(crate) fn mark_sent_in(tx: &mut WriteTx<'_>, ids: &[u32]) -> Result<()> {
    for id in ids {
        let key = record_key(*id);
        if let Some(mut row) = tx.get_json::<StoredKemPrekey>(Table::Prekeys, key.as_bytes(), 1)? {
            row.sent = true;
            tx.put_json(Table::Prekeys, key.as_bytes(), 1, &row)?;
        }
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
        store.write(|tx| consume_in(tx, 5, &ordinary)).unwrap();
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
