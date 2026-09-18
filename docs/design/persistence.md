# Persistence and durable messaging

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 11. Persistence: one atomic protocol state, optional separate history

### 11.1 Database boundaries

Use one protocol database per account, initially `protocol.redb`. It stores everything whose consistency affects future communication: credentials, crypto state, minimal pending messages, retry state, durable events and checkpoints.

Optional searchable history belongs in a separate `history.redb` or a caller-supplied projection. The library's correctness must not depend on that projection being current.

This deliberately changes the original key-only database rule. A minimal inbox/outbox journal belongs with the ratchets because the send/decrypt operation must commit atomically. Completed historical conversation storage can remain separate. A key-only backup cannot safely claim to restore pending protocol work.

### 11.2 Logical schema

These are logical tables, not SQL DDL. Prefix all records with account generation where applicable. Version both the database schema and individual record formats.

| Table | Key | Value |
| --- | --- | --- |
| `meta` | Metadata name | Schema version, profile ID, account generation, journal counters |
| `account` | Singleton | Our identities, Noise static material, registration state and observed credentials |
| `registration_attempts` | Attempt ID | Durable state, challenge metadata, generation, deadlines and outcome |
| `prekeys` | `(kind, prekey_id)` | Secret record, publication/consumption state and version |
| `peer_identities` | Full device address | Identity material, trust decision and change history |
| `signal_sessions` | Full device address | Encrypted serialized session and revision |
| `sender_keys` | `(group, sender_device, distribution_id)` | Encrypted sender-key record and revision |
| `device_lists` | Contact identity | Device set, evidence/version and refresh state |
| `groups` | Group ID | Metadata, membership revision and pending mutations |
| `sender_key_distribution` | `(group, key_generation, recipient_device)` | Pending/observed distribution state |
| `inbox` | Protocol deduplication key | Pending/processed marker, bounded payload, receipt obligations |
| `outbox` | Local message ID | Request identity, durable intent, prepared attempts and delivery facts |
| `idempotency` | `(caller_namespace, client_request_id)` | Local message ID and semantic request hash |
| `events` | Monotonic cursor | Stable event ID, kind, revision and encrypted payload |
| `consumers` | Consumer ID | Last contiguous acknowledged cursor and retention policy |
| `checkpoints` | Observed stream/sync namespace | Catch-up cursor/version if the protocol actually provides one |
| `media_jobs` | Job ID | Encrypted references, staging state, digests and resume metadata |

The same protocol-store transaction must support atomic updates to any necessary combination of these tables. A store abstraction exposing only independent `put_session` and `put_message` calls is insufficient.

### 11.3 At-rest encryption

Use RustCrypto `XChaCha20Poly1305` for new at-rest envelopes, with an independently generated 24-byte random nonce for each write. Its larger nonce space avoids a fragile application-managed persistent counter across crashes and backups. This is a storage format choice; it does not alter WhatsApp's wire cryptography. Fail on entropy-source failure; never substitute a timestamp, row ID or deterministic test RNG in production. [RustCrypto algorithm documentation](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/)

Envelope fields: format version, algorithm identifier, key ID, nonce, ciphertext and authentication tag. Authenticate the account generation, table/record identity and format version as associated data using an unambiguous length-delimited encoding. Substituting a valid ciphertext into another row must fail.

Keep the master key outside the database. Accept an injected `KeySource`; let the CLI obtain it from an operator-selected secret source, with an environment variable as an explicitly supported option. Validate the key encoding/length. Never generate a replacement key when an existing store cannot decrypt.

Use a wrapped per-database data key and a separately domain-separated index key where needed. Sensitive lookup identifiers can use keyed indexes; their reversible mappings remain encrypted. Encrypt message journal bodies, media keys, auth state and device fingerprint data too. Table sizes, record counts and some structural metadata will remain visible: this is record encryption, not whole-file confidentiality.

Wrong key, corrupted tag, unsupported envelope version and missing key ID are typed failures. Implement key rotation as a restartable migration with old-key retention until all referenced records/backups are addressed. Use redacted diagnostics and zeroizing secret containers; do not claim memory erasure from every allocator copy.

The existing `AesGcmCipher` pattern may inform the `KeySource` and envelope boundary after inspection. Do not share a nontransactional `FileSessionStore` as a second live source of ratchet state. If importing an existing encrypted format is needed, make it an explicit migration, not a plaintext fallback.

### 11.4 Async/transaction boundary

`redb` persistence is blocking; run database operations on a dedicated worker or bounded blocking executor. The account actor owns the operation order.

For an async Signal backend, load an owned, versioned snapshot of required records and use an in-memory transaction overlay. The adapter reads/writes that overlay. Commit the resulting mutations and message/event records atomically with expected-revision checks. Never retain database guards across asynchronous crypto callbacks or network I/O. On conflict, discard the derived output and reload; never transmit ciphertext derived from an uncommitted stale snapshot.

Use durable commit settings for cryptographic advancement. If a commit reports an ambiguous I/O failure, stop further protocol mutation until reopening/recovery determines what committed. Do not keep running with a potentially divergent in-memory state.

### 11.5 Backups and rollback

Back up a consistent protocol snapshot, including pending inbox/outbox and crypto state. Exclude optional historical projections if desired. Do not copy live database files outside the backend's supported snapshot/backup procedure.

Restoring an old valid encrypted snapshot is not the same as recovering the latest ratchet state. AEAD detects tampering, not rollback. Fence off the old running owner, preserve the restored generation, and use only a verified recovery procedure. If safe recovery is unavailable, return a recovery-required condition; do not force old state into service or automatically re-register.

Never resume old Noise transport counters after restore. Preserve idempotency mappings and message IDs where recovery supports retry. State exactly what a backup can and cannot recover.

## 12. Outbox: commit encryption state before transmission

### 12.1 API guarantee

`send_message` durably accepts an intent and returns a `SendTicket`. It does not claim server acceptance, recipient delivery, or reading.

Require a caller-supplied `ClientRequestId` for callers that need retry-safe enqueue. An identical repeated request returns the same ticket; reuse of the ID with different semantic content returns `IdempotencyConflict`. Generate the WhatsApp wire ID using the observed format and persist it before transmission. UUIDs may be local IDs; they are not automatically valid wire IDs.

### 12.2 Send sequence

1. Validate capability, limits and the request identity; commit the immutable intent to the outbox.
2. Resolve recipients/devices and required trust/group state. Network discovery happens outside a database transaction.
3. Load the relevant versioned crypto snapshot and prepare the E2EE payloads in the transaction overlay.
4. Atomically commit updated crypto state, the full prepared E2EE attempt, destination snapshot, wire identity and status/event changes.
5. Only after that durable commit, encode the envelope and send it through the current Noise connection.
6. Persist validated ACKs, receipts, protocol retry requests and terminal errors as new facts.

For multi-device sends, the committed attempt must contain the full intended fanout and all associated state mutations, or explicitly model staged sub-attempts. A half-prepared send cannot disappear from recovery.

Persist **E2EE payloads and necessary stanza data**, not old Noise ciphertext. Retransmission on a new connection requires new transport encryption.

Distinguish two cases:

- A network retry of the same committed attempt reuses its persisted E2EE payload and message identity when the observed protocol allows it.
- A protocol-directed re-encryption/session-repair attempt is a new durable attempt under the same logical message, with its own validated crypto mutations. Never re-encrypt blindly on every timeout.

An existing prepared group attempt must be reconsidered when membership changes. Do not blindly resend distribution material to a removed member. Follow the verified protocol's retry/update rules or return an explicit unresolved outcome.

### 12.3 Honest outcomes

Separate attempt progress (`Queued`, `Prepared`, `InFlight`, `RetryScheduled`) from delivery facts (`ServerAccepted`, recipient/device receipts, `Failed`, `DeliveryUnknown`). Receipt ordering may differ from ACK ordering. Maintain per-recipient/device facts before deriving a user-facing aggregate.

Exactly-once delivery over an unreliable external network is not promised. After a lost response, local state may not prove whether the server accepted an operation. Preserve the ambiguity rather than generating a second logical message with a new ID.

Cancellation before transmission can stop a queued intent. Cancellation after transmission cannot unsend it; that requires a separately supported revocation operation. Dropping a Rust future after enqueue must not erase the durable send.

## 13. Inbox, receipts and durable events

### 13.1 Inbound sequence

1. Validate transport authentication, structural bounds, envelope routing and addressing.
2. Determine the protocol deduplication identity. A processed duplicate follows the appropriate ACK/receipt behavior without ratcheting or re-emitting a new logical message. Distinguish a legitimate protocol retry/re-encryption from conflicting content under the same identity; do not silently overwrite an existing message or trust an ID collision as proof of equivalence.
3. Persist a minimal recoverable inbound envelope if the protocol requires an early transport acknowledgement.
4. Decrypt using a versioned crypto snapshot/overlay; validate the message content and any authenticated control operation.
5. Atomically commit crypto mutations, prekey consumption, processed identity, normalized event and required receipt work.
6. Fulfill application-level delivery receipts after durable processing, according to observed semantics.

Not all acknowledgements mean “message decrypted and delivered.” Identify transport ACK, delivery receipt, read receipt and played receipt separately. If an ACK must precede successful decryption, first durably store enough material to resume processing. Do not globally delay time-sensitive protocol ACKs based on an invented rule.

If decryption fails or a backend partially mutates its overlay before returning an error, discard those changes unless the backend contract explicitly requires a specific recovery mutation. Persist a bounded retry/quarantine state, not a successful user message.

Do not acknowledge a message as read merely because it was stored, emitted to a callback, or forwarded to a bridge. Read/played receipts are explicit application policy.

### 13.2 Journal and replay

Each committed domain event has a stable `EventId`, monotonically ordered per-account cursor, account generation and entity revision where relevant. Consumers request bounded batches after a cursor and acknowledge contiguous progress.

Deliver at least once. Consumers deduplicate by event identity and make their own local effects transactional. Waking a subscriber may use an in-memory channel; the database remains the source of truth if a notification is dropped.

A slow consumer must never cause silent loss. Apply configured disk/queue limits, pause work where the protocol permits, and expose backpressure. Retention may expire an abandoned consumer only under an explicit policy; later reads return `CursorExpired` with recovery information, not a false empty batch.

Project history in its own transaction, then acknowledge the source cursor. A crash between these operations causes replay, which the projection must handle idempotently. Do not attempt an implicit distributed transaction between protocol and history stores.

### 13.3 Sync is not necessarily a server history API

`SyncEngine` consumes the protocol journal, manages local projections and performs only catch-up operations actually supported by the primary profile. It does not run a second connection reader.

Do not import linked-device history/app-state assumptions wholesale. Primary startup, offline queues, group metadata synchronization and companion history transfer can have different responsibilities.

Define `sync_once` as a bounded catch-up/drain with a cursor and progress report. It cannot promise “all history downloaded” without an observed API that provides it. Return a `HistoryGap` or unsupported-capability result where recovery is unavailable.
