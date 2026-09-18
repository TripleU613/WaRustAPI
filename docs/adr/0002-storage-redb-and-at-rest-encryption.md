# ADR 0002: `redb` protocol store with XChaCha20-Poly1305 record envelopes

Status: accepted · 2026-09-17

## Decision

- One `protocol.redb` per account (redb 4.3.0, pure Rust, ACID). All logical
  tables from docs/design/README.md 11.2 exist as `&[u8] -> &[u8]` tables plus `meta`,
  `events` (`u64 -> &[u8]`) and `consumers`.
- Every record except `meta` is sealed with XChaCha20-Poly1305
  (`chacha20poly1305` 0.11) under a **random per-database data key**. The data
  key and a separate random index key are stored in `meta.wrapped_keys`,
  sealed under a key-encryption key (KEK) derived by HKDF-SHA256 from the
  injected master key and a random 32-byte salt in `meta`. Nonces are 24
  random bytes per write from the OS CSPRNG; entropy failure is an error,
  never substituted. The envelope `key_id` identifies the data key and is
  stable across master-key rotations.
- Associated data binds `(account generation, table, record key, record
  format version)` with a length-delimited encoding. Tests prove that moving
  a ciphertext to another row, table, generation or version fails.
- Lookup keys of tables whose keys are sensitive identifiers (sessions, peer
  identities, device lists, groups, inbox, idempotency, media jobs) are
  blinded with a keyed SHA-256 under a separately derived index key.
- Unwrapping `meta.wrapped_keys` is the key check: a wrong master key fails
  authentication and returns `Error::WrongStorageKey`; a replacement key is
  never generated.
- `rotate_master_key` re-wraps the data keys under a new KEK and salt in one
  durable transaction (O(1), restartable, records untouched); afterwards the
  old master key no longer opens the store. CLI: `store rotate-key`.
- Single-writer ownership uses `std::fs::File::try_lock` on `<db>.lock`
  (no extra dependency); a second opener gets `StoreError::AlreadyOwned`.
- Commits use `Durability::Immediate`. Async callers run store work through
  `spawn_blocking`; no guard is held across an `await`.
- The event journal is append-only with monotonic cursors; consumers
  acknowledge contiguous progress; `trim_events` never removes events below
  the slowest consumer and later reads below the floor return
  `CursorExpired`.

## Not done yet

Data-key rotation (re-encrypting every record), backup/restore procedure,
schema migrations beyond version 1, and the `history.redb` projection.
