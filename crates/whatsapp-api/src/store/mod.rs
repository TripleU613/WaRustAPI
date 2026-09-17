//! Encrypted transactional protocol store on `redb` (README section 11).
//!
//! One database per account holds everything whose consistency affects future
//! communication. Every secret-bearing record is sealed in an authenticated
//! envelope before it reaches the database; `meta` is the only plaintext
//! table and contains no secrets. A single process owns a database at a time.
//!
//! Persistence is blocking; async callers use [`ProtocolStore::run_blocking`],
//! which moves the closure to a blocking thread. The account actor decides
//! operation order; no database guard is ever held across an `await`.

pub mod envelope;
pub mod keys;

use std::fmt;
use std::fs::{File, OpenOptions as FsOpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use redb::{
    Database, Durability, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use self::envelope::{RecordAad, Sealer};
use self::keys::{DerivedKeys, KeySource};
use crate::error::{Error, Result};
use crate::model::ids::{AccountGeneration, AccountId, EventCursor, EventId};
use crate::model::time::Timestamp;
use crate::sync::{EventBatch, EventEnvelope, SyncEvent};

pub use self::keys::{EnvKeySource, MasterKey, StaticKeySource};

/// Current on-disk schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Store-level failures.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// The database engine reported an error.
    #[error("database error: {0}")]
    Database(String),
    /// Filesystem error.
    #[error("store i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// The OS entropy source failed. Never substituted.
    #[error("entropy source failed: {0}")]
    Entropy(String),
    /// A cryptographic operation failed.
    #[error("crypto failure: {0}")]
    Crypto(&'static str),
    /// Envelope bytes are structurally invalid.
    #[error("malformed envelope: {0}")]
    MalformedEnvelope(&'static str),
    /// Envelope format version is not supported by this build.
    #[error("unsupported envelope version {0}")]
    UnsupportedEnvelopeVersion(u8),
    /// The envelope was sealed under a different key id.
    #[error("envelope key id mismatch (expected {expected}, found {found})")]
    KeyIdMismatch {
        /// Key id of the currently derived key.
        expected: u32,
        /// Key id found in the envelope.
        found: u32,
    },
    /// Authentication tag did not verify (wrong key, tampering or row substitution).
    #[error("envelope authentication failed")]
    AuthenticationFailed,
    /// The supplied master key does not open this database.
    #[error("wrong master key for this store")]
    WrongKey,
    /// Another process owns this database.
    #[error("store at {path} is owned by another process")]
    AlreadyOwned {
        /// Database path.
        path: PathBuf,
    },
    /// The database was written by an unsupported schema version.
    #[error("unsupported schema version {found} (this build supports {supported})")]
    SchemaVersion {
        /// Version found in `meta`.
        found: u32,
        /// Version this build supports.
        supported: u32,
    },
    /// A record or key exceeded the size bound.
    #[error("record too large")]
    RecordTooLarge,
    /// The requested cursor precedes retained events.
    #[error("cursor expired; earliest retained cursor is {floor:?}")]
    CursorExpired {
        /// First cursor still retained.
        floor: EventCursor,
    },
    /// The acknowledged cursor is beyond the journal head.
    #[error("cursor {cursor:?} is beyond journal head {head:?}")]
    CursorAhead {
        /// Requested cursor.
        cursor: EventCursor,
        /// Current head.
        head: EventCursor,
    },
    /// Record (de)serialization failed.
    #[error("record serialization failed: {0}")]
    Serialization(String),
    /// The database belongs to a different account.
    #[error("store belongs to a different account")]
    AccountMismatch,
    /// Internal lock poisoned.
    #[error("internal lock poisoned")]
    Poisoned,
}

impl From<StoreError> for Error {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::WrongKey => Error::WrongStorageKey,
            other => Error::Storage(other),
        }
    }
}

macro_rules! redb_err {
    ($($t:ty),*) => {$(
        impl From<$t> for StoreError {
            fn from(e: $t) -> Self { StoreError::Database(e.to_string()) }
        }
    )*};
}
redb_err!(
    redb::DatabaseError,
    redb::TransactionError,
    redb::TableError,
    redb::StorageError,
    redb::CommitError,
    redb::SetDurabilityError
);

/// Identifier of an event consumer.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConsumerId(String);

impl ConsumerId {
    /// Validate and wrap a consumer name.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.is_empty() || s.len() > 64 || !s.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(Error::invalid(
                "consumer_id",
                "1..=64 printable ASCII characters",
            ));
        }
        Ok(Self(s))
    }

    /// Borrow the name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ConsumerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConsumerId({})", self.0)
    }
}

/// Logical tables (README section 11.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Table {
    /// Our identities, Noise static material, registration state, credentials.
    Account,
    /// Durable registration attempt state.
    RegistrationAttempts,
    /// Prekeys keyed by `(kind, id)`.
    Prekeys,
    /// Peer identity material and trust decisions.
    PeerIdentities,
    /// Encrypted Signal session records.
    SignalSessions,
    /// Encrypted sender-key records.
    SenderKeys,
    /// Cached device lists.
    DeviceLists,
    /// Group metadata.
    Groups,
    /// Sender-key distribution state.
    SenderKeyDistribution,
    /// Inbound deduplication and processing state.
    Inbox,
    /// Durable send intents and attempts.
    Outbox,
    /// Client request id -> local message id.
    Idempotency,
    /// Observed catch-up cursors.
    Checkpoints,
    /// Media upload/download jobs.
    MediaJobs,
}

impl Table {
    const fn name(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::RegistrationAttempts => "registration_attempts",
            Self::Prekeys => "prekeys",
            Self::PeerIdentities => "peer_identities",
            Self::SignalSessions => "signal_sessions",
            Self::SenderKeys => "sender_keys",
            Self::DeviceLists => "device_lists",
            Self::Groups => "groups",
            Self::SenderKeyDistribution => "sender_key_distribution",
            Self::Inbox => "inbox",
            Self::Outbox => "outbox",
            Self::Idempotency => "idempotency",
            Self::Checkpoints => "checkpoints",
            Self::MediaJobs => "media_jobs",
        }
    }

    /// Tables whose lookup keys are sensitive identifiers (device addresses,
    /// contact identities, caller request ids) and are therefore blinded.
    const fn blinded_keys(self) -> bool {
        matches!(
            self,
            Self::PeerIdentities
                | Self::SignalSessions
                | Self::SenderKeys
                | Self::DeviceLists
                | Self::Groups
                | Self::SenderKeyDistribution
                | Self::Inbox
                | Self::Idempotency
                | Self::MediaJobs
        )
    }

    const fn def(self) -> TableDefinition<'static, &'static [u8], &'static [u8]> {
        TableDefinition::new(self.name())
    }

    /// Every table, for enumeration in tooling.
    pub const ALL: [Table; 14] = [
        Self::Account,
        Self::RegistrationAttempts,
        Self::Prekeys,
        Self::PeerIdentities,
        Self::SignalSessions,
        Self::SenderKeys,
        Self::DeviceLists,
        Self::Groups,
        Self::SenderKeyDistribution,
        Self::Inbox,
        Self::Outbox,
        Self::Idempotency,
        Self::Checkpoints,
        Self::MediaJobs,
    ];
}

const META: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
const EVENTS: TableDefinition<u64, &[u8]> = TableDefinition::new("events");
const CONSUMERS: TableDefinition<&str, &[u8]> = TableDefinition::new("consumers");

const META_SCHEMA: &str = "schema_version";
const META_SALT: &str = "salt";
const META_ACCOUNT: &str = "account_id";
const META_GENERATION: &str = "generation";
const META_KEY_CHECK: &str = "key_check";
const META_JOURNAL_FLOOR: &str = "journal_floor";
const KEY_CHECK_PLAINTEXT: &[u8] = b"whatsapp-api store key check v1";

/// Options for opening or creating a store.
#[derive(Debug)]
pub struct OpenOptions {
    /// Database file path.
    pub path: PathBuf,
    /// Account the database belongs to.
    pub account_id: AccountId,
    /// Master key source.
    pub key_source: Arc<dyn KeySource>,
    /// Create the database if it does not exist.
    pub create: bool,
}

#[derive(Serialize, Deserialize)]
struct EventRecord {
    event_id: EventId,
    generation: AccountGeneration,
    committed_at: Timestamp,
    kind: String,
    event: SyncEvent,
}

#[derive(Serialize, Deserialize, Default)]
struct ConsumerRecord {
    acked: EventCursor,
}

const RECORD_V1: u16 = 1;

/// Encrypted protocol store for one account.
pub struct ProtocolStore {
    db: Database,
    _owner_lock: File,
    path: PathBuf,
    account_id: AccountId,
    sealer: Sealer,
    index_key: [u8; 32],
    generation: RwLock<AccountGeneration>,
}

impl fmt::Debug for ProtocolStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProtocolStore")
            .field("path", &self.path)
            .field("account_id", &self.account_id)
            .finish_non_exhaustive()
    }
}

impl ProtocolStore {
    /// Open (or create) the store and take single-writer ownership.
    pub fn open(opts: OpenOptions) -> Result<Self> {
        let master = opts.key_source.master_key()?;
        let exists = opts.path.exists();
        if !exists && !opts.create {
            return Err(StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "store does not exist",
            ))
            .into());
        }
        if let Some(parent) = opts.path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(StoreError::Io)?;
        }
        let owner_lock = take_owner_lock(&opts.path)?;
        let db = Database::builder()
            .create(&opts.path)
            .map_err(StoreError::from)?;

        // Bootstrap or validate meta.
        let (salt, generation, key_check_exists) = {
            let tx = db.begin_write().map_err(StoreError::from)?;
            let (salt, generation, key_check) = {
                let mut meta = tx.open_table(META).map_err(StoreError::from)?;
                let schema = meta
                    .get(META_SCHEMA)
                    .map_err(StoreError::from)?
                    .map(|g| bytes_to_u32(g.value()));
                match schema {
                    None => {
                        let mut salt = [0u8; 32];
                        getrandom::fill(&mut salt)
                            .map_err(|e| StoreError::Entropy(e.to_string()))?;
                        meta.insert(META_SCHEMA, SCHEMA_VERSION.to_be_bytes().as_slice())
                            .map_err(StoreError::from)?;
                        meta.insert(META_SALT, salt.as_slice())
                            .map_err(StoreError::from)?;
                        meta.insert(META_ACCOUNT, opts.account_id.as_str().as_bytes())
                            .map_err(StoreError::from)?;
                        meta.insert(META_GENERATION, 1u32.to_be_bytes().as_slice())
                            .map_err(StoreError::from)?;
                        meta.insert(META_JOURNAL_FLOOR, 0u64.to_be_bytes().as_slice())
                            .map_err(StoreError::from)?;
                        (salt, AccountGeneration(1), None)
                    }
                    Some(v) if v == SCHEMA_VERSION => {
                        let salt_v = meta
                            .get(META_SALT)
                            .map_err(StoreError::from)?
                            .map(|g| g.value().to_vec())
                            .ok_or(StoreError::MalformedEnvelope("meta.salt missing"))?;
                        let mut salt = [0u8; 32];
                        if salt_v.len() != 32 {
                            return Err(StoreError::MalformedEnvelope("meta.salt length").into());
                        }
                        salt.copy_from_slice(&salt_v);
                        let acct = meta
                            .get(META_ACCOUNT)
                            .map_err(StoreError::from)?
                            .map(|g| g.value().to_vec())
                            .unwrap_or_default();
                        if acct != opts.account_id.as_str().as_bytes() {
                            return Err(StoreError::AccountMismatch.into());
                        }
                        let generation = meta
                            .get(META_GENERATION)
                            .map_err(StoreError::from)?
                            .map(|g| AccountGeneration(bytes_to_u32(g.value())))
                            .unwrap_or(AccountGeneration(1));
                        let key_check = meta
                            .get(META_KEY_CHECK)
                            .map_err(StoreError::from)?
                            .map(|g| g.value().to_vec());
                        (salt, generation, key_check)
                    }
                    Some(found) => {
                        return Err(StoreError::SchemaVersion {
                            found,
                            supported: SCHEMA_VERSION,
                        }
                        .into());
                    }
                }
            };
            // Ensure all tables exist so read transactions never fail on a
            // missing table.
            for t in Table::ALL {
                tx.open_table(t.def()).map_err(StoreError::from)?;
            }
            tx.open_table(EVENTS).map_err(StoreError::from)?;
            tx.open_table(CONSUMERS).map_err(StoreError::from)?;
            tx.commit().map_err(StoreError::from)?;
            (salt, generation, key_check)
        };

        let derived = DerivedKeys::derive(&master, &salt);
        let sealer = Sealer::new(&derived.data_key, derived.key_id);
        let key_check_aad = RecordAad {
            generation: 0,
            table: "meta",
            key: META_KEY_CHECK.as_bytes(),
            record_version: RECORD_V1,
        };
        match key_check_exists {
            Some(existing) => match sealer.open(&key_check_aad, &existing) {
                Ok(pt) if pt == KEY_CHECK_PLAINTEXT => {}
                Ok(_)
                | Err(StoreError::AuthenticationFailed)
                | Err(StoreError::KeyIdMismatch { .. }) => {
                    return Err(StoreError::WrongKey.into());
                }
                Err(e) => return Err(e.into()),
            },
            None => {
                let sealed = sealer.seal(&key_check_aad, KEY_CHECK_PLAINTEXT)?;
                let tx = db.begin_write().map_err(StoreError::from)?;
                {
                    let mut meta = tx.open_table(META).map_err(StoreError::from)?;
                    meta.insert(META_KEY_CHECK, sealed.as_slice())
                        .map_err(StoreError::from)?;
                }
                tx.commit().map_err(StoreError::from)?;
            }
        }

        Ok(Self {
            db,
            _owner_lock: owner_lock,
            path: opts.path,
            account_id: opts.account_id,
            sealer,
            index_key: derived.index_key,
            generation: RwLock::new(generation),
        })
    }

    /// Database path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Account this store belongs to.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Current account generation.
    pub fn generation(&self) -> AccountGeneration {
        *self.generation.read().unwrap_or_else(|p| p.into_inner())
    }

    /// Run a blocking store operation on a blocking thread.
    pub async fn run_blocking<T, F>(self: &Arc<Self>, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&ProtocolStore) -> Result<T> + Send + 'static,
    {
        let store = Arc::clone(self);
        tokio::task::spawn_blocking(move || f(&store))
            .await
            .map_err(|_| Error::Cancelled)?
    }

    /// Run a read transaction.
    pub fn read<T>(&self, f: impl FnOnce(&ReadTx<'_>) -> Result<T>) -> Result<T> {
        let tx = self.db.begin_read().map_err(StoreError::from)?;
        let rtx = ReadTx {
            store: self,
            tx: &tx,
            generation: self.generation(),
        };
        f(&rtx)
    }

    /// Run a write transaction with durable commit. If `f` fails, nothing is
    /// committed.
    pub fn write<T>(&self, f: impl FnOnce(&mut WriteTx<'_>) -> Result<T>) -> Result<T> {
        let mut tx = self.db.begin_write().map_err(StoreError::from)?;
        tx.set_durability(Durability::Immediate)
            .map_err(StoreError::from)?;
        let generation = self.generation();
        let mut wtx = WriteTx {
            store: self,
            tx: &tx,
            generation,
            new_generation: None,
        };
        let out = f(&mut wtx)?;
        let new_generation = wtx.new_generation;
        tx.commit().map_err(StoreError::from)?;
        if let Some(g) = new_generation {
            *self.generation.write().unwrap_or_else(|p| p.into_inner()) = g;
        }
        Ok(out)
    }

    fn blind(&self, table: Table, key: &[u8]) -> Vec<u8> {
        if !table.blinded_keys() {
            return key.to_vec();
        }
        let mut h = Sha256::new();
        h.update(b"whatsapp-api/index/v1");
        h.update(self.index_key);
        h.update((table.name().len() as u16).to_be_bytes());
        h.update(table.name().as_bytes());
        h.update((key.len() as u32).to_be_bytes());
        h.update(key);
        h.finalize().to_vec()
    }

    // --- journal conveniences used by SyncEngine -----------------------------

    /// Last contiguous acknowledged cursor of a consumer (`START` if unknown).
    pub fn consumer_cursor(&self, consumer: &ConsumerId) -> Result<EventCursor> {
        self.read(|tx| tx.consumer_cursor(consumer))
    }

    /// Read a bounded batch after `after`.
    pub fn read_events(
        &self,
        consumer: &ConsumerId,
        after: EventCursor,
        limit: u32,
    ) -> Result<EventBatch> {
        self.read(|tx| tx.read_events(consumer, after, limit))
    }

    /// Acknowledge contiguous progress.
    pub fn ack_events(&self, consumer: &ConsumerId, cursor: EventCursor) -> Result<()> {
        self.write(|tx| tx.ack_events(consumer, cursor))
    }

    /// Append one event in its own transaction.
    pub fn append_event(&self, event: SyncEvent) -> Result<EventEnvelope> {
        self.write(|tx| tx.append_event(event))
    }

    /// Current journal head (0 when empty).
    pub fn journal_head(&self) -> Result<EventCursor> {
        self.read(|tx| tx.journal_head())
    }
}

fn take_owner_lock(db_path: &Path) -> Result<File> {
    let mut lock_path = db_path.as_os_str().to_owned();
    lock_path.push(".lock");
    let file = FsOpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(StoreError::Io)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(StoreError::AlreadyOwned {
            path: db_path.to_path_buf(),
        }
        .into()),
        Err(TryLockError::Error(e)) => Err(StoreError::Io(e).into()),
    }
}

fn bytes_to_u32(b: &[u8]) -> u32 {
    let mut a = [0u8; 4];
    let n = b.len().min(4);
    a[4 - n..].copy_from_slice(&b[..n]);
    u32::from_be_bytes(a)
}

fn bytes_to_u64(b: &[u8]) -> u64 {
    let mut a = [0u8; 8];
    let n = b.len().min(8);
    a[8 - n..].copy_from_slice(&b[..n]);
    u64::from_be_bytes(a)
}

fn to_json<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|e| StoreError::Serialization(e.to_string()).into())
}

fn from_json<T: for<'de> Deserialize<'de>>(b: &[u8]) -> Result<T> {
    serde_json::from_slice(b).map_err(|e| StoreError::Serialization(e.to_string()).into())
}

/// A read transaction.
pub struct ReadTx<'a> {
    store: &'a ProtocolStore,
    tx: &'a redb::ReadTransaction,
    generation: AccountGeneration,
}

impl fmt::Debug for ReadTx<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadTx")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

/// A write transaction.
pub struct WriteTx<'a> {
    store: &'a ProtocolStore,
    tx: &'a redb::WriteTransaction,
    generation: AccountGeneration,
    new_generation: Option<AccountGeneration>,
}

impl fmt::Debug for WriteTx<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteTx")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

fn open_record(
    store: &ProtocolStore,
    generation: AccountGeneration,
    table: Table,
    blinded: &[u8],
    record_version: u16,
    value: &[u8],
) -> Result<Vec<u8>> {
    let aad = RecordAad {
        generation: generation.0,
        table: table.name(),
        key: blinded,
        record_version,
    };
    Ok(store.sealer.open(&aad, value)?)
}

fn read_events_in<T: ReadableTable<u64, &'static [u8]>>(
    store: &ProtocolStore,
    events: &T,
    floor: EventCursor,
    after: EventCursor,
    limit: u32,
) -> Result<EventBatch> {
    if after < floor {
        return Err(StoreError::CursorExpired { floor }.into());
    }
    let start = after.0.saturating_add(1);
    let mut out = Vec::new();
    let mut has_more = false;
    let range = events.range(start..).map_err(StoreError::from)?;
    for item in range {
        let (k, v) = item.map_err(StoreError::from)?;
        if out.len() as u32 >= limit {
            has_more = true;
            break;
        }
        let cursor = EventCursor(k.value());
        let aad_key = cursor.0.to_be_bytes();
        let aad = RecordAad {
            generation: 0,
            table: "events",
            key: &aad_key,
            record_version: RECORD_V1,
        };
        let pt = store.sealer.open(&aad, v.value())?;
        let rec: EventRecord = from_json(&pt)?;
        out.push(EventEnvelope {
            account_id: store.account_id.clone(),
            account_generation: rec.generation,
            event_id: rec.event_id,
            cursor,
            committed_at: rec.committed_at,
            event: rec.event,
        });
    }
    let ack_cursor = out.last().map(|e| e.cursor).unwrap_or(after);
    Ok(EventBatch {
        events: out,
        ack_cursor,
        has_more,
    })
}

fn consumer_cursor_in<T: ReadableTable<&'static str, &'static [u8]>>(
    store: &ProtocolStore,
    consumers: &T,
    consumer: &ConsumerId,
) -> Result<EventCursor> {
    let Some(v) = consumers.get(consumer.as_str()).map_err(StoreError::from)? else {
        return Ok(EventCursor::START);
    };
    let aad = RecordAad {
        generation: 0,
        table: "consumers",
        key: consumer.as_str().as_bytes(),
        record_version: RECORD_V1,
    };
    let pt = store.sealer.open(&aad, v.value())?;
    let rec: ConsumerRecord = from_json(&pt)?;
    Ok(rec.acked)
}

fn journal_floor_in<T: ReadableTable<&'static str, &'static [u8]>>(
    meta: &T,
) -> Result<EventCursor> {
    Ok(meta
        .get(META_JOURNAL_FLOOR)
        .map_err(StoreError::from)?
        .map(|g| EventCursor(bytes_to_u64(g.value())))
        .unwrap_or(EventCursor::START))
}

fn journal_head_in<T: ReadableTable<u64, &'static [u8]>>(events: &T) -> Result<EventCursor> {
    Ok(events
        .last()
        .map_err(StoreError::from)?
        .map(|(k, _)| EventCursor(k.value()))
        .unwrap_or(EventCursor::START))
}

impl ReadTx<'_> {
    /// Account generation the transaction reads under.
    pub fn generation(&self) -> AccountGeneration {
        self.generation
    }

    /// Get and decrypt a record.
    pub fn get(&self, table: Table, key: &[u8], record_version: u16) -> Result<Option<Vec<u8>>> {
        let t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        let blinded = self.store.blind(table, key);
        let Some(v) = t.get(blinded.as_slice()).map_err(StoreError::from)? else {
            return Ok(None);
        };
        open_record(
            self.store,
            self.generation,
            table,
            &blinded,
            record_version,
            v.value(),
        )
        .map(Some)
    }

    /// Get and deserialize a JSON record.
    pub fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        table: Table,
        key: &[u8],
        record_version: u16,
    ) -> Result<Option<T>> {
        match self.get(table, key, record_version)? {
            Some(b) => Ok(Some(from_json(&b)?)),
            None => Ok(None),
        }
    }

    /// Decrypt every record of a non-blinded table, in key order, up to `limit`.
    pub fn scan(
        &self,
        table: Table,
        record_version: u16,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        let mut out = Vec::new();
        for item in t.iter().map_err(StoreError::from)? {
            if out.len() >= limit {
                break;
            }
            let (k, v) = item.map_err(StoreError::from)?;
            let key = k.value().to_vec();
            let pt = open_record(
                self.store,
                self.generation,
                table,
                &key,
                record_version,
                v.value(),
            )?;
            out.push((key, pt));
        }
        Ok(out)
    }

    /// Number of records in a table.
    pub fn count(&self, table: Table) -> Result<u64> {
        let t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        Ok(t.len().map_err(StoreError::from)?)
    }

    /// Consumer's acknowledged cursor.
    pub fn consumer_cursor(&self, consumer: &ConsumerId) -> Result<EventCursor> {
        let c = self.tx.open_table(CONSUMERS).map_err(StoreError::from)?;
        consumer_cursor_in(self.store, &c, consumer)
    }

    /// Journal head.
    pub fn journal_head(&self) -> Result<EventCursor> {
        let e = self.tx.open_table(EVENTS).map_err(StoreError::from)?;
        journal_head_in(&e)
    }

    /// Bounded batch of events after `after`.
    pub fn read_events(
        &self,
        _consumer: &ConsumerId,
        after: EventCursor,
        limit: u32,
    ) -> Result<EventBatch> {
        let meta = self.tx.open_table(META).map_err(StoreError::from)?;
        let floor = journal_floor_in(&meta)?;
        let e = self.tx.open_table(EVENTS).map_err(StoreError::from)?;
        read_events_in(self.store, &e, floor, after, limit)
    }
}

impl WriteTx<'_> {
    /// Account generation the transaction writes under.
    pub fn generation(&self) -> AccountGeneration {
        self.generation
    }

    /// Get and decrypt a record.
    pub fn get(&self, table: Table, key: &[u8], record_version: u16) -> Result<Option<Vec<u8>>> {
        let t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        let blinded = self.store.blind(table, key);
        let Some(v) = t.get(blinded.as_slice()).map_err(StoreError::from)? else {
            return Ok(None);
        };
        open_record(
            self.store,
            self.generation,
            table,
            &blinded,
            record_version,
            v.value(),
        )
        .map(Some)
    }

    /// Get and deserialize a JSON record.
    pub fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        table: Table,
        key: &[u8],
        record_version: u16,
    ) -> Result<Option<T>> {
        match self.get(table, key, record_version)? {
            Some(b) => Ok(Some(from_json(&b)?)),
            None => Ok(None),
        }
    }

    /// Seal and store a record.
    pub fn put(
        &mut self,
        table: Table,
        key: &[u8],
        record_version: u16,
        plaintext: &[u8],
    ) -> Result<()> {
        if key.len() > 4096 {
            return Err(StoreError::RecordTooLarge.into());
        }
        let blinded = self.store.blind(table, key);
        let aad = RecordAad {
            generation: self.generation.0,
            table: table.name(),
            key: &blinded,
            record_version,
        };
        let sealed = self.store.sealer.seal(&aad, plaintext)?;
        let mut t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        t.insert(blinded.as_slice(), sealed.as_slice())
            .map_err(StoreError::from)?;
        Ok(())
    }

    /// Serialize as JSON, seal and store.
    pub fn put_json<T: Serialize>(
        &mut self,
        table: Table,
        key: &[u8],
        record_version: u16,
        value: &T,
    ) -> Result<()> {
        let b = to_json(value)?;
        self.put(table, key, record_version, &b)
    }

    /// Delete a record. Returns whether it existed.
    pub fn delete(&mut self, table: Table, key: &[u8]) -> Result<bool> {
        let blinded = self.store.blind(table, key);
        let mut t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        Ok(t.remove(blinded.as_slice())
            .map_err(StoreError::from)?
            .is_some())
    }

    /// Decrypt every record of a non-blinded table, in key order, up to `limit`.
    pub fn scan(
        &self,
        table: Table,
        record_version: u16,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        let mut out = Vec::new();
        for item in t.iter().map_err(StoreError::from)? {
            if out.len() >= limit {
                break;
            }
            let (k, v) = item.map_err(StoreError::from)?;
            let key = k.value().to_vec();
            let pt = open_record(
                self.store,
                self.generation,
                table,
                &key,
                record_version,
                v.value(),
            )?;
            out.push((key, pt));
        }
        Ok(out)
    }

    /// Number of records in a table.
    pub fn count(&self, table: Table) -> Result<u64> {
        let t = self.tx.open_table(table.def()).map_err(StoreError::from)?;
        Ok(t.len().map_err(StoreError::from)?)
    }

    /// Append a domain event to the journal. Committed atomically with the
    /// rest of this transaction.
    pub fn append_event(&mut self, event: SyncEvent) -> Result<EventEnvelope> {
        let mut events = self.tx.open_table(EVENTS).map_err(StoreError::from)?;
        let head = journal_head_in(&events)?;
        let cursor = head.next();
        let event_id = EventId::generate()?;
        let committed_at = Timestamp::now();
        let rec = EventRecord {
            event_id,
            generation: self.generation,
            committed_at,
            kind: event.kind().to_owned(),
            event,
        };
        let pt = to_json(&rec)?;
        let aad_key = cursor.0.to_be_bytes();
        let aad = RecordAad {
            generation: 0,
            table: "events",
            key: &aad_key,
            record_version: RECORD_V1,
        };
        let sealed = self.store.sealer.seal(&aad, &pt)?;
        events
            .insert(cursor.0, sealed.as_slice())
            .map_err(StoreError::from)?;
        Ok(EventEnvelope {
            account_id: self.store.account_id.clone(),
            account_generation: self.generation,
            event_id,
            cursor,
            committed_at,
            event: rec.event,
        })
    }

    /// Consumer's acknowledged cursor.
    pub fn consumer_cursor(&self, consumer: &ConsumerId) -> Result<EventCursor> {
        let c = self.tx.open_table(CONSUMERS).map_err(StoreError::from)?;
        consumer_cursor_in(self.store, &c, consumer)
    }

    /// Journal head.
    pub fn journal_head(&self) -> Result<EventCursor> {
        let e = self.tx.open_table(EVENTS).map_err(StoreError::from)?;
        journal_head_in(&e)
    }

    /// Bounded batch of events after `after`.
    pub fn read_events(
        &self,
        _consumer: &ConsumerId,
        after: EventCursor,
        limit: u32,
    ) -> Result<EventBatch> {
        let meta = self.tx.open_table(META).map_err(StoreError::from)?;
        let floor = journal_floor_in(&meta)?;
        let e = self.tx.open_table(EVENTS).map_err(StoreError::from)?;
        read_events_in(self.store, &e, floor, after, limit)
    }

    /// Acknowledge contiguous progress. Cursors never move backwards; a cursor
    /// beyond the head is rejected.
    pub fn ack_events(&mut self, consumer: &ConsumerId, cursor: EventCursor) -> Result<()> {
        let head = self.journal_head()?;
        if cursor > head {
            return Err(StoreError::CursorAhead { cursor, head }.into());
        }
        let current = self.consumer_cursor(consumer)?;
        if cursor <= current {
            return Ok(());
        }
        let rec = ConsumerRecord { acked: cursor };
        let pt = to_json(&rec)?;
        let aad = RecordAad {
            generation: 0,
            table: "consumers",
            key: consumer.as_str().as_bytes(),
            record_version: RECORD_V1,
        };
        let sealed = self.store.sealer.seal(&aad, &pt)?;
        let mut c = self.tx.open_table(CONSUMERS).map_err(StoreError::from)?;
        c.insert(consumer.as_str(), sealed.as_slice())
            .map_err(StoreError::from)?;
        Ok(())
    }

    /// Delete events at or below `up_to` **only** if every registered consumer
    /// has acknowledged past them. Later reads below the new floor return
    /// `CursorExpired`. Returns the number of events removed.
    pub fn trim_events(&mut self, up_to: EventCursor) -> Result<u64> {
        let consumers = self.tx.open_table(CONSUMERS).map_err(StoreError::from)?;
        let mut min_ack: Option<EventCursor> = None;
        for item in consumers.iter().map_err(StoreError::from)? {
            let (k, _) = item.map_err(StoreError::from)?;
            let id = ConsumerId::new(k.value())?;
            let acked = consumer_cursor_in(self.store, &consumers, &id)?;
            min_ack = Some(min_ack.map_or(acked, |m| m.min(acked)));
        }
        drop(consumers);
        let limit = match min_ack {
            Some(m) => up_to.min(m),
            None => up_to,
        };
        let mut events = self.tx.open_table(EVENTS).map_err(StoreError::from)?;
        let removed = events
            .extract_if(|k, _| k <= limit.0)
            .map_err(StoreError::from)?
            .count() as u64;
        drop(events);
        let mut meta = self.tx.open_table(META).map_err(StoreError::from)?;
        meta.insert(META_JOURNAL_FLOOR, limit.0.to_be_bytes().as_slice())
            .map_err(StoreError::from)?;
        Ok(removed)
    }

    /// Advance the account generation (restore/re-registration fencing).
    /// Takes effect for records written by later transactions.
    pub fn bump_generation(&mut self) -> Result<AccountGeneration> {
        let next = self.generation.next();
        let mut meta = self.tx.open_table(META).map_err(StoreError::from)?;
        meta.insert(META_GENERATION, next.0.to_be_bytes().as_slice())
            .map_err(StoreError::from)?;
        self.new_generation = Some(next);
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::ChatId;
    use crate::model::jid::Jid;
    use crate::model::message::MessageKey;
    use crate::model::{Direction, MessageId};

    fn temp_store(key: MasterKey) -> (tempfile::TempDir, Arc<ProtocolStore>) {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("protocol.redb"),
            account_id: AccountId::new("acct-1").unwrap(),
            key_source: Arc::new(StaticKeySource::new(key)),
            create: true,
        })
        .unwrap();
        (dir, Arc::new(store))
    }

    fn sample_event(n: u64) -> SyncEvent {
        SyncEvent::MessageDeleted(MessageKey {
            chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net").unwrap()).unwrap(),
            sender: None,
            direction: Direction::Incoming,
            id: MessageId::new(format!("MSG{n}")).unwrap(),
        })
    }

    #[test]
    fn records_roundtrip_and_are_encrypted() {
        let (_d, s) = temp_store(MasterKey::from_bytes([1u8; 32]));
        s.write(|tx| {
            tx.put(Table::Account, b"account", 1, b"top secret")?;
            tx.put(Table::SignalSessions, b"dev-1", 1, b"session bytes")?;
            Ok(())
        })
        .unwrap();
        let v = s
            .read(|tx| tx.get(Table::Account, b"account", 1))
            .unwrap()
            .unwrap();
        assert_eq!(v, b"top secret");
        let v2 = s
            .read(|tx| tx.get(Table::SignalSessions, b"dev-1", 1))
            .unwrap()
            .unwrap();
        assert_eq!(v2, b"session bytes");
        // Raw database bytes must not contain the plaintext or the blinded
        // table's original key.
        let raw = std::fs::read(s.path()).unwrap();
        assert!(!raw.windows(10).any(|w| w == b"top secret"));
        assert!(!raw.windows(5).any(|w| w == b"dev-1"));
        assert!(
            raw.windows(7).any(|w| w == b"account"),
            "non-blinded key visible"
        );
    }

    #[test]
    fn wrong_key_is_typed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.redb");
        let acct = AccountId::new("a").unwrap();
        {
            let s = ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: acct.clone(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([1u8; 32]))),
                create: true,
            })
            .unwrap();
            s.write(|tx| tx.put(Table::Account, b"x", 1, b"y")).unwrap();
        }
        let err = ProtocolStore::open(OpenOptions {
            path: path.clone(),
            account_id: acct.clone(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([2u8; 32]))),
            create: false,
        })
        .err()
        .unwrap();
        assert!(matches!(err, Error::WrongStorageKey), "{err:?}");
        // Right key still works after the failed attempt (no key was regenerated).
        let s = ProtocolStore::open(OpenOptions {
            path,
            account_id: acct,
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([1u8; 32]))),
            create: false,
        })
        .unwrap();
        assert_eq!(
            s.read(|tx| tx.get(Table::Account, b"x", 1))
                .unwrap()
                .unwrap(),
            b"y"
        );
    }

    #[test]
    fn second_owner_is_rejected() {
        let (_d, s) = temp_store(MasterKey::from_bytes([1u8; 32]));
        let err = ProtocolStore::open(OpenOptions {
            path: s.path().to_path_buf(),
            account_id: s.account_id().clone(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([1u8; 32]))),
            create: false,
        })
        .err()
        .unwrap();
        assert!(
            matches!(err, Error::Storage(StoreError::AlreadyOwned { .. })),
            "{err:?}"
        );
    }

    #[test]
    fn account_mismatch_and_missing_store() {
        let (_d, s) = temp_store(MasterKey::from_bytes([1u8; 32]));
        let path = s.path().to_path_buf();
        drop(s);
        let err = ProtocolStore::open(OpenOptions {
            path,
            account_id: AccountId::new("other").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([1u8; 32]))),
            create: false,
        })
        .err()
        .unwrap();
        assert!(matches!(err, Error::Storage(StoreError::AccountMismatch)));
        let err = ProtocolStore::open(OpenOptions {
            path: PathBuf::from("/nonexistent/dir/x.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([1u8; 32]))),
            create: false,
        })
        .err()
        .unwrap();
        assert!(matches!(err, Error::Storage(StoreError::Io(_))));
    }

    #[test]
    fn journal_is_ordered_replayable_and_acknowledged() {
        let (_d, s) = temp_store(MasterKey::from_bytes([3u8; 32]));
        let consumer = ConsumerId::new("bridge").unwrap();
        for i in 1..=5 {
            let env = s.append_event(sample_event(i)).unwrap();
            assert_eq!(env.cursor, EventCursor(i));
        }
        let b = s.read_events(&consumer, EventCursor::START, 2).unwrap();
        assert_eq!(b.events.len(), 2);
        assert!(b.has_more);
        assert_eq!(b.ack_cursor, EventCursor(2));
        // Replay without ack returns the same events.
        let b2 = s.read_events(&consumer, EventCursor::START, 2).unwrap();
        assert_eq!(b.events, b2.events);
        s.ack_events(&consumer, EventCursor(2)).unwrap();
        assert_eq!(s.consumer_cursor(&consumer).unwrap(), EventCursor(2));
        // Ack never regresses, never exceeds head.
        s.ack_events(&consumer, EventCursor(1)).unwrap();
        assert_eq!(s.consumer_cursor(&consumer).unwrap(), EventCursor(2));
        assert!(matches!(
            s.ack_events(&consumer, EventCursor(99)),
            Err(Error::Storage(StoreError::CursorAhead { .. }))
        ));
        let rest = s.read_events(&consumer, EventCursor(2), 10).unwrap();
        assert_eq!(rest.events.len(), 3);
        assert!(!rest.has_more);
        assert_eq!(rest.ack_cursor, EventCursor(5));
        let ids: std::collections::HashSet<_> = rest.events.iter().map(|e| e.event_id).collect();
        assert_eq!(ids.len(), 3, "event ids are unique");
    }

    #[test]
    fn trim_respects_slowest_consumer_and_expires_cursors() {
        let (_d, s) = temp_store(MasterKey::from_bytes([4u8; 32]));
        let fast = ConsumerId::new("fast").unwrap();
        let slow = ConsumerId::new("slow").unwrap();
        for i in 1..=6 {
            s.append_event(sample_event(i)).unwrap();
        }
        s.ack_events(&fast, EventCursor(6)).unwrap();
        s.ack_events(&slow, EventCursor(2)).unwrap();
        let removed = s.write(|tx| tx.trim_events(EventCursor(5))).unwrap();
        assert_eq!(removed, 2, "only events <= slowest ack are removed");
        let err = s.read_events(&slow, EventCursor::START, 10).err().unwrap();
        assert!(matches!(
            err,
            Error::Storage(StoreError::CursorExpired {
                floor: EventCursor(2)
            })
        ));
        let ok = s.read_events(&slow, EventCursor(2), 10).unwrap();
        assert_eq!(ok.events.len(), 4);
    }

    #[test]
    fn failed_transaction_commits_nothing() {
        let (_d, s) = temp_store(MasterKey::from_bytes([5u8; 32]));
        let r: Result<()> = s.write(|tx| {
            tx.put(Table::Outbox, b"m1", 1, b"intent")?;
            tx.append_event(sample_event(1))?;
            Err(Error::Cancelled)
        });
        assert!(r.is_err());
        assert_eq!(s.read(|tx| tx.get(Table::Outbox, b"m1", 1)).unwrap(), None);
        assert_eq!(s.journal_head().unwrap(), EventCursor::START);
    }

    #[test]
    fn generation_bump_fences_old_records() {
        let (_d, s) = temp_store(MasterKey::from_bytes([6u8; 32]));
        s.write(|tx| tx.put(Table::Account, b"k", 1, b"gen1"))
            .unwrap();
        let g = s.write(|tx| tx.bump_generation()).unwrap();
        assert_eq!(g, AccountGeneration(2));
        assert_eq!(s.generation(), AccountGeneration(2));
        // The old record was authenticated under generation 1 and must not
        // open under generation 2.
        let err = s.read(|tx| tx.get(Table::Account, b"k", 1)).err().unwrap();
        assert!(matches!(
            err,
            Error::Storage(StoreError::AuthenticationFailed)
        ));
    }

    #[tokio::test]
    async fn run_blocking_works() {
        let (_d, s) = temp_store(MasterKey::from_bytes([7u8; 32]));
        let head = s.run_blocking(|st| st.journal_head()).await.unwrap();
        assert_eq!(head, EventCursor::START);
    }
}
