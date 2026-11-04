# Atomic app-state conflict catch-up

2026-11-04. **Confirmed local source defect:** conflict processing in
`transport/app_state_sync.rs::catch_up` loaded the held collection outside a
write transaction, verified each patch's held key in a separate write transaction,
and finally wrote the resulting state unconditionally. A concurrent accepted
patch could change the held baseline between those steps and be overwritten.
The function could not participate in the caller's atomic settlement transaction.
It also silently skipped a `patch` child with a non-byte payload.

**Confirmed static reference:** Android APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The [retained server-sync lifecycle](server-sync-lifecycle.md#response-verification-commit-and-rescan)
traces `SyncResponseHandler.A07` through `0mj.A00` and `0l8.A08` inside its
application transaction. The existing observed 409 shape (2026-09-24) remains
attributed in the Rust function documentation; this correction does not claim a
fresh live server response or complete native conflict/error policy.

## Correction

`catch_up_in(&mut WriteTx, reply)` reads the held baseline, resolves every held
key used by the supported verification path, verifies the complete ordered
patch chain, and writes the final integrity state in the caller's single durable
transaction. It does not open nested transactions. Thus pending-settlement changes
and catch-up can roll back together, and a baseline staged earlier in that same
transaction is the baseline actually checked. No result computed from an old
independent snapshot is written over newer verified state.

An actual patch without byte payload now refuses, as do unsupported protobufs,
unknown keys, bad MACs and nonsequential versions. Unknown non-`patch` children
remain ignored by this bounded path. A patch list requires a single explicit
collection error `code=409`; ambiguous error or patch-list siblings refuse.
These singleton/error-route checks are local defensive validation of the
supported conflict shape, not a claim that the native reader rejects every
other server form. The caller separately owns matching the authenticated reply
to the original request and expected collection.

This function still stores verified integrity state only; complete native mutation
action projection remains a separate, unfinished application responsibility.
No patch installs an incoming key. Patch authentication and record authentication
now resolve their separate key IDs from the already-held account keys within the
same transaction. **Confirmed current-build evidence:** `SyncResponseHandler.A07`
collects record IDs at DEX offsets `021e..0228` and `029c..02a6`, separately adds
the patch/snapshot ID at `0446..045a`, and resolves the combined set through
`0fI.A08` at `045e..0464`. See the retained
[response trace](rea/2.26.43.72-server-sync-lifecycle/server-sync-response.txt)
and envelope `ev_bdbbe822dbf9e326660455eea6fe36900bd682985f5b122054c44c507ecb7b73`.
Record IDs need not equal the enclosing patch ID. Any unheld key still refuses
the entire chain. No advertised dirty version,
unauthenticated action, privacy setting or automatic network request is introduced.

## Offline verification

Four focused tests exercise the real encrypted store and collection verifier:

- Independent Python/OpenSSL SET/REMOVE vectors (also used by the literal-wire
  collection tests) commit a two-patch chain to version two and empty LT state,
  survive reopen, and cannot replay over the advanced baseline.
- A bad second key, MAC, protobuf or non-byte payload rolls back the caller's
  earlier checkpoint write and every held-state change; reopen confirms the
  rollback. Unknown keys remain absent.
- A version-one baseline staged in the caller's still-open transaction is used
  by version-two catch-up. A non409 response containing patches cannot advance
  the saved state.
- An unchanged independent literal record, enclosed by snapshot/patch MACs made
  with a second synthetic key, refuses while that key is absent and commits only
  after it is held. This regression tests separate key resolution, not native
  mutation semantics across key rotation; logical-index tracking is separately
  audited.

These are local transactional/cryptographic regressions, not live acceptance or
proof that an account cannot be restricted. Root settlement/queue changes are
documented separately with their own invariants.

## Authenticated action retention (2026-11-04)

**Confirmed Rust defect and correction:** the supported 409 utility previously
dropped `VerifiedCollectionPatch::mutations` after advancing its integrity map.
It now writes those ordered decrypted actions into the encrypted action journal
inside the same transaction as the held checkpoint and pending-write release.
The production settlement caller derives request/reply SHA-256 fingerprints from
the exact immutable stored wire pair, not from re-encoding decoded nodes. The
journal binds action, collection, patch version and actual transaction generation.
All patch/record keys still come from the already-held key resolver in that
transaction. No response-provided key is installed.

This keeps the utility's existing **whole-chain atomic** contract. A malformed
or bad-MAC later patch rolls back all earlier journal writes, caller writes,
held-state changes and pending release. This is not native per-patch commit
parity, a successful-fetch applier, or contact/settings projection. The retained
actions remain available for a future projector instead of being irretrievably
discarded. Old already-applied conflicts are not retrospectively reconstructed.

Tests exercise real settled-action ingestion with exact stored digest checks,
pending release, reopen and no replay. Independent literal SET/REMOVE vectors
check both ordered operation batches survive reopen; late failure tests require
no partial journal. Existing journal conflict/AAD/rollback tests still apply.
Native reference remains the pinned 2.26.43.72 lifecycle above; this change adds
no APK evidence or live account operation.
