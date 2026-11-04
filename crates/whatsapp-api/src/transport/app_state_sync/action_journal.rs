//! Encrypted retention only: the caller must verify the complete collection
//! patch against authorized keys and its transactional baseline before calling.
//! No public input type, request digest or this journal authenticates a patch.
//! Mirrors storage::PendingAction's ordered representation and zeroizing Drop;
//! deliberately uses a distinct namespace from integrity-state/pending-apply rows.
use super::Collection;
use crate::{
    Error, Result,
    model::ids::AccountGeneration,
    signal::app_state_crypto::{DecryptedMutation, MutationOperation},
    store::{Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use zeroize::{Zeroize, Zeroizing};

/// Caller-supplied original request/reply binding, not proof of verification.
pub(crate) struct JournalContext<'a> {
    pub(crate) action_id: &'a str,
    pub(crate) collection: Collection,
    pub(crate) patch_version: u64,
    pub(crate) generation: AccountGeneration,
    pub(crate) request_digest: [u8; 32],
    pub(crate) reply_digest: [u8; 32],
}
impl fmt::Debug for JournalContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JournalContext(<redacted>)")
    }
}
impl Drop for JournalContext<'_> {
    fn drop(&mut self) {
        self.request_digest.zeroize();
        self.reply_digest.zeroize();
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainOutcome {
    Inserted,
    AlreadyPresent,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RetainedAction {
    operation: u8,
    index: Vec<u8>,
    value: Option<Vec<u8>>,
    schema_version: i32,
}
impl fmt::Debug for RetainedAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RetainedAction(<redacted>)")
    }
}
impl Drop for RetainedAction {
    fn drop(&mut self) {
        self.operation.zeroize();
        self.schema_version.zeroize();
        self.index.zeroize();
        if let Some(value) = &mut self.value {
            value.zeroize();
        }
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RetainedBatch {
    format_version: u8,
    action_id: String,
    collection: String,
    patch_version: u64,
    generation: u32,
    request_digest: [u8; 32],
    reply_digest: [u8; 32],
    actions: Vec<RetainedAction>,
}
impl fmt::Debug for RetainedBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RetainedBatch(<redacted>)")
    }
}
impl Drop for RetainedBatch {
    fn drop(&mut self) {
        self.patch_version.zeroize();
        self.generation.zeroize();
        self.action_id.zeroize();
        self.collection.zeroize();
        self.request_digest.zeroize();
        self.reply_digest.zeroize();
        // Each retained action separately zeroizes its owned contents.
    }
}
fn invalid(reason: &'static str) -> Error {
    Error::invalid("app_state_fetch_action_journal", reason)
}
fn key(ctx: &JournalContext<'_>) -> Vec<u8> {
    // Domain-separated, unambiguous tuple encoding. Generation is intentionally
    // absent: an old-generation row must fail AAD authentication, not look absent.
    let mut digest = Sha256::new();
    digest.update(b"app-state-fetch-action-journal:v1\0");
    digest.update((ctx.action_id.len() as u64).to_be_bytes());
    digest.update(ctx.action_id.as_bytes());
    digest.update((ctx.collection.name().len() as u64).to_be_bytes());
    digest.update(ctx.collection.name().as_bytes());
    digest.update(ctx.patch_version.to_be_bytes());
    let mut out = b"app-state-fetch-action-journal:v1:".to_vec();
    out.extend_from_slice(&digest.finalize());
    out
}

/// Retain already-authenticated ordered mutations inside the caller's write
/// transaction. Caller owes verification and current baseline/generation checks;
/// propagate errors so journal, held-state and cursor all roll back together.
/// This does not project actions, advance state/cursors, or authorize any send.
/// Bounds below are local resource policy, not recovered server requirements.
pub(crate) fn retain_authenticated_mutations_in(
    tx: &mut WriteTx<'_>,
    ctx: &JournalContext<'_>,
    mutations: &[DecryptedMutation],
) -> Result<RetainOutcome> {
    if ctx.action_id.is_empty() || ctx.action_id.len() > 128 || ctx.generation != tx.generation() {
        return Err(invalid("invalid journal context or account generation"));
    }
    if mutations.len() > 4096 {
        return Err(invalid("journal exceeds local mutation bound"));
    }
    let mut size = 0usize;
    for m in mutations {
        size = size
            .checked_add(m.index().len())
            .and_then(|n| n.checked_add(m.value().map_or(0, <[u8]>::len)))
            .ok_or_else(|| invalid("journal exceeds local byte bound"))?;
        if size > 16 * 1024 * 1024 {
            return Err(invalid("journal exceeds local byte bound"));
        }
    }
    let row = RetainedBatch {
        format_version: 1,
        action_id: ctx.action_id.to_owned(),
        collection: ctx.collection.name().to_owned(),
        patch_version: ctx.patch_version,
        generation: ctx.generation.0,
        request_digest: ctx.request_digest,
        reply_digest: ctx.reply_digest,
        actions: mutations
            .iter()
            .map(|m| RetainedAction {
                operation: match m.operation() {
                    MutationOperation::Set => 1,
                    MutationOperation::Remove => 2,
                },
                index: m.index().to_vec(),
                value: m.value().map(<[u8]>::to_vec),
                schema_version: m.version(),
            })
            .collect(),
    };
    let row_key = key(ctx);
    // Never treat an authentication, generation, schema or storage error as
    // absence. get_json is avoided to zeroize the decrypted JSON buffer.
    if let Some(bytes) = tx.get(Table::Checkpoints, &row_key, 1)? {
        let bytes = Zeroizing::new(bytes);
        let old: RetainedBatch = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("invalid retained mutation journal"))?;
        if old != row {
            return Err(invalid(
                "retained mutation journal conflicts with original context",
            ));
        }
        return Ok(RetainOutcome::AlreadyPresent);
    }
    let mut bytes = Zeroizing::new(Vec::new());
    serde_json::to_writer(&mut *bytes, &row)
        .map_err(|_| invalid("mutation journal serialization failed"))?;
    tx.put(Table::Checkpoints, &row_key, 1, &bytes)?;
    Ok(RetainOutcome::Inserted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        signal::app_state_crypto::MutationKeys,
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, StoreError},
    };
    use std::{path::Path, sync::Arc};
    fn open(path: &Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("fetch-journal-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([41; 32]))),
            create,
        })
        .unwrap()
    }
    fn context(generation: AccountGeneration) -> JournalContext<'static> {
        JournalContext {
            action_id: "original-private-fetch-id",
            collection: Collection::Regular,
            patch_version: 17,
            generation,
            request_digest: [7; 32],
            reply_digest: [8; 32],
        }
    }
    // Real encrypt/MAC/decrypt path; no public mutation constructor bypasses
    // value/index authentication in these retention fixtures.
    fn mutations() -> Vec<DecryptedMutation> {
        let keys = MutationKeys::derive(&[32; 32]).unwrap();
        let id = [3; 6];
        [
            (MutationOperation::Set, Some(&[8, 127][..]), 5),
            (MutationOperation::Remove, None, 9),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (op, value, version))| {
            let index = if i == 0 {
                br#"["journal-private-index"]"#.as_slice()
            } else {
                br#"["journal-private-remove"]"#.as_slice()
            };
            let encrypted = keys
                .encrypt_mutation(op, &id, index, value, version, [i as u8; 16])
                .unwrap();
            keys.decrypt_mutation(op, &id, &encrypted.body, &encrypted.index_mac)
                .unwrap()
        })
        .collect()
    }
    #[test]
    fn encrypted_reopen_preserves_order_and_exact_repeat_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("journal.redb");
        let store = open(&path, true);
        let ctx = context(store.generation());
        let ms = mutations();
        assert_eq!(
            store
                .write(|tx| retain_authenticated_mutations_in(tx, &ctx, &ms))
                .unwrap(),
            RetainOutcome::Inserted
        );
        let head = store.journal_head().unwrap();
        drop(store);
        let disk = std::fs::read(&path).unwrap();
        for secret in [
            b"journal-private-index".as_slice(),
            b"original-private-fetch-id".as_slice(),
        ] {
            assert!(!disk.windows(secret.len()).any(|w| w == secret));
        }
        let store = open(&path, false);
        assert_eq!(
            store
                .write(|tx| retain_authenticated_mutations_in(tx, &ctx, &ms))
                .unwrap(),
            RetainOutcome::AlreadyPresent
        );
        assert_eq!(store.journal_head().unwrap(), head);
        let bytes = Zeroizing::new(
            store
                .read(|tx| tx.get(Table::Checkpoints, &key(&ctx), 1))
                .unwrap()
                .unwrap(),
        );
        let saved: RetainedBatch = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(saved.actions.len(), 2);
        for (a, m) in saved.actions.iter().zip(&ms) {
            assert_eq!(a.index, m.index());
            assert_eq!(a.value.as_deref(), m.value());
            assert_eq!(a.schema_version, m.version());
        }
        assert_eq!(saved.actions[0].operation, 1);
        assert_eq!(saved.actions[1].operation, 2);
        for debug in [
            format!("{saved:?}"),
            format!("{:?}", saved.actions),
            format!("{ctx:?}"),
        ] {
            assert!(!debug.contains("private"));
        }
    }
    #[test]
    fn context_or_content_conflict_never_replaces_original() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("journal.redb"), true);
        let ctx = context(store.generation());
        let ms = mutations();
        store
            .write(|tx| retain_authenticated_mutations_in(tx, &ctx, &ms))
            .unwrap();
        let before = store
            .read(|tx| tx.get(Table::Checkpoints, &key(&ctx), 1))
            .unwrap();
        let mut changed = context(store.generation());
        changed.reply_digest[0] ^= 1;
        assert!(
            store
                .write(|tx| retain_authenticated_mutations_in(tx, &changed, &ms))
                .is_err()
        );
        changed = context(store.generation());
        changed.request_digest[0] ^= 1;
        assert!(
            store
                .write(|tx| retain_authenticated_mutations_in(tx, &changed, &ms))
                .is_err()
        );
        let err = store
            .write(|tx| retain_authenticated_mutations_in(tx, &ctx, &ms[..1]))
            .unwrap_err();
        assert!(!format!("{err:?}").contains("private"));
        assert_eq!(
            store
                .read(|tx| tx.get(Table::Checkpoints, &key(&ctx), 1))
                .unwrap(),
            before
        );
    }
    #[test]
    fn outer_failure_rolls_back_journal_held_state_and_cursor_then_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollback.redb");
        let store = open(&path, true);
        let ctx = context(store.generation());
        let ms = mutations();
        let result: Result<()> = store.write(|tx| {
            retain_authenticated_mutations_in(tx, &ctx, &ms)?;
            tx.put(Table::Checkpoints, b"test-held-state", 1, b"new")?;
            tx.put(Table::Checkpoints, b"test-fetch-cursor", 1, b"new")?;
            Err(invalid("later patch failed verification"))
        });
        assert!(result.is_err());
        drop(store);
        let store = open(&path, false);
        for k in [
            key(&ctx),
            b"test-held-state".to_vec(),
            b"test-fetch-cursor".to_vec(),
        ] {
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, &k, 1))
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(
            store
                .write(|tx| retain_authenticated_mutations_in(tx, &ctx, &ms))
                .unwrap(),
            RetainOutcome::Inserted
        );
    }
    #[test]
    fn generation_aad_failure_is_not_absence_or_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("generation.redb"), true);
        let ctx = context(store.generation());
        let ms = mutations();
        store
            .write(|tx| retain_authenticated_mutations_in(tx, &ctx, &ms))
            .unwrap();
        store.write(|tx| tx.bump_generation()).unwrap();
        let fresh = context(store.generation());
        let err = store
            .write(|tx| retain_authenticated_mutations_in(tx, &fresh, &ms))
            .unwrap_err();
        assert!(matches!(
            err,
            Error::Storage(StoreError::AuthenticationFailed)
        ));
        assert!(matches!(
            store.read(|tx| tx.get(Table::Checkpoints, &key(&fresh), 1)),
            Err(Error::Storage(StoreError::AuthenticationFailed))
        ));
    }
    #[test]
    fn malformed_json_is_static_failure_and_identity_is_unambiguous() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("malformed.redb"), true);
        let ctx = context(store.generation());
        let ms = mutations();
        store
            .write(|tx| tx.put(Table::Checkpoints, &key(&ctx), 1, b"private malformed JSON"))
            .unwrap();
        let err = store
            .write(|tx| retain_authenticated_mutations_in(tx, &ctx, &ms))
            .unwrap_err();
        assert!(!format!("{err:?}").contains("private"));
        let mut another = context(store.generation());
        another.collection = Collection::RegularLow;
        assert_eq!(
            store
                .write(|tx| retain_authenticated_mutations_in(tx, &another, &ms))
                .unwrap(),
            RetainOutcome::Inserted
        );
        another = context(store.generation());
        another.patch_version += 1;
        assert_eq!(
            store
                .write(|tx| retain_authenticated_mutations_in(tx, &another, &ms))
                .unwrap(),
            RetainOutcome::Inserted
        );
        let oversized_id = "x".repeat(129);
        for action_id in ["", oversized_id.as_str()] {
            let mut invalid_context = context(store.generation());
            invalid_context.action_id = action_id;
            assert!(
                store
                    .write(|tx| retain_authenticated_mutations_in(tx, &invalid_context, &ms))
                    .is_err()
            );
        }
    }
}
