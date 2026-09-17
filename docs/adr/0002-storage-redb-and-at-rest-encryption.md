# ADR 0002: `redb` protocol store with XChaCha20-Poly1305 record envelopes

Status: accepted · 2026-09-17

## Decision

- One `protocol.redb` per account (redb 4.3.0, pure Rust, ACID). All logical
  tables from README 11.2 exist as `&[u8] -> &[u8]` tables plus `meta`,
  `events` (`u64 -> &[u8]`) and `consumers`.
- Every record except `meta` is sealed with XChaCha20-Poly1305
  (`chacha20poly1305` 0.11) under a per-database data key derived by
  HKDF-SHA256 from the injected master key and a random 32-byte salt stored in
  `meta`. Nonces are 24 random bytes per write from the OS CSPRNG; entropy
  failure is an error, never substituted.
- Associated data binds `(account generation, table, record key, record
  format version)` with a length-delimited encoding. Tests prove that moving
  a ciphertext to another row, table, generation or version fails.
- Lookup keys of tables whose keys are sensitive identifiers (sessions, peer
  identities, device lists, groups, inbox, idempotency, media jobs) are
  blinded with a keyed SHA-256 under a separately derived index key.
- A key-check record detects a wrong master key and returns
  `Error::WrongStorageKey`; a replacement key is never generated.
- Single-writer ownership uses `std::fs::File::try_lock` on `<db>.lock`
  (no extra dependency); a second opener gets `StoreError::AlreadyOwned`.
- Commits use `Durability::Immediate`. Async callers run store work through
  `spawn_blocking`; no guard is held across an `await`.
- The event journal is append-only with monotonic cursors; consumers
  acknowledge contiguous progress; `trim_events` never removes events below
  the slowest consumer and later reads below the floor return
  `CursorExpired`.

## Not done yet

Key rotation as a restartable migration, backup/restore procedure, schema
migrations beyond version 1, and the `history.redb` projection.
