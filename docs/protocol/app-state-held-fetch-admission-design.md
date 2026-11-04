# Prospective held-version fetch admission

2026-11-04. **Confirmed source/offline:** the durable admission subset below is
implemented in `held_fetch.rs`, with explicit CLI exposure and reservation-time
checks. Earlier **proposed** sections describe the design, not completed response
application. No new APK evidence or live account operation is claimed.
The [binder review](app-state-fetch-binding-review.md),
[application specification](app-state-inline-fetch-application-spec.md) and
[connection-flight design](app-state-connection-flight-design.md) remain the
separate authority, application and runtime-lifetime boundaries.

## A distinct prospective intent

**Proposed:** add one explicit `queue_held_fetch(store, collections)` producer,
with a private `queue_held_fetch_in(&mut WriteTx, collections)` transaction
helper. The caller selects existing known collections; no notification, timer,
startup event or retry implicitly creates this intent. A later manual CLI option
can expose it without altering existing saved requests.

Prefer a distinct durable action kind, for example `app-state-held-fetch`, so
new intents require their sealed admission record. Existing `app-state-fetch`
version-zero/snapshot intents remain identifiable historical/bootstrap requests,
unsupported by incremental binding; do not attach retrospective baselines to them.
Legacy behavior and any refusal-only backoff handling need an explicit shared
fetch-kind predicate, not a quiet reinterpretation of the old kind. Missing
admission metadata on the new kind is an integrity/implementation fault; absence
on the old kind is an expected unsupported legacy boundary.

No action kind alone proves original grammar. The new producer still builds and
validates the same supported patch-free request shape used by the pure binder.
Its version fields come from authenticated held checkpoints; return_snapshot is
false and therefore omitted by the existing version-aware builder. A positive
held version is not required merely to invent a safety restriction: an existing
valid complete empty checkpoint at version zero can be represented distinctly
from a missing checkpoint. Snapshot/bootstrap installation remains unsupported.

## Minimal private sealed record

**Proposed shape**, stored as normal generation-authenticated JSON in
`Table::Checkpoints` under a new action-ID prefix:

```text
HeldFetchRecordV1 {
  schema_version: 1,
  generation: AccountGeneration,
  action_id: String,
  request_digest: [u8; 32],
  baselines: Vec<SavedCollectionState>,
  phase: Queued | Admitted { reserved_at: Timestamp }
}
```

The prefix/field names are illustrative, not a new wire format. The record
contains no duplicated Node and no advertised dirty version. Baselines are
validated complete states normalized through `from_saved().to_saved()`, so
semantically irrelevant vector ordering cannot cause spurious refusal. Preserve
exact authenticated logical-index bytes; never normalize JSON indexes.

Its `request_digest` is SHA-256 of the actual `actions::Row.wire`, as stored by
the queue, not a re-encoding of a later decoded Node. The store encrypts the
whole record with transaction generation/table/key/version associated data.
These private records are authenticated local admission provenance; the existing
public `FetchBaselinePlan::new` remains a caller claim when used by itself.

`reserved_at` is a diagnostic socket-reservation timestamp, not proof of actual
write, server acceptance or a callback deadline. This record is not an account
flight lease. Old `Sent` records never acquire a new live epoch owner through
an account-wide scan.

## Queue transaction

**Confirmed existing mechanics:** `actions::queue_in` creates the action ID,
encodes a built Node and inserts `Row { status: Queued, wire, ... }` in its caller's
write transaction. The public `actions::queue` executes an arbitrary builder
outside the write lock, so it is not the correct factory for state-derived
atomic intent creation. No nested `store.write` belongs in the private factory.

**Proposed single transaction:**

1. Read `tx.generation()` and current server backoff. Refuse an active wait with
   its existing redacted diagnostic; do not remove or shorten it.
2. Validate a nonempty bounded unique caller collection list and load each
   **existing** transport plain-name held checkpoint. Preserve missing versus
   present empty state. Propagate stored-row authentication/decoding faults.
3. Restore each state through `CollectionState::from_saved`, require complete
   logical provenance and a signed-long-range version, and retain its normalized
   full saved baseline. Check relevant per-collection pending writes through the
   existing pending-state authority, not a scan of all historical action statuses.
4. Use `actions::queue_in` and `fetch_request_at_versions` with these exact
   collection/version pairs and snapshot policy false. Queueing neither reserves
   a socket write nor changes held collection state.
5. Read the newly inserted action's actual immutable encoded wire in this same
   transaction. Seal the queued admission record using that digest and the same
   generation/action ID/baselines. Failure rolls back both queue and record.

The minimal actions-side addition is a crate-private raw-wire accessor for a
write transaction, optionally returning action kind/status alongside the bytes.
A feature must not deserialize actions' private Row schema independently or
introduce a second encoder to obtain its digest. Another option is a narrowly
scoped queue helper returning its stored wire; neither requires public raw-wire
exposure, a new queue framework or running a callback under an unknown lock.

## Reservation transaction

**Confirmed existing mechanics:** `reserve_limit` implements both public bulk
reservation and actor `reserve_next`; `reserve_one_in` is a separate selected
path. Both mark Sent before the socket write. The held-fetch admission check
must be shared by both paths. Updating only the actor's immediate caller leaves
an independent selected/public reservation bypass.

**Proposed private surface:**

```text
check_held_fetch_reservation_in(tx, action_id, actual_row_wire)
  -> Eligible | Deferred(reason) | LocallyRefused(reason)
mark_held_fetch_admitted_in(tx, action_id, actual_row_wire, reserved_at)
```

These may be one guarded helper if the caller's transaction owns both record
and Row writes. The important boundary is that `Admitted` and `Row.status=Sent`
commit together; helper names are secondary. No other API should expose an
admitted record for a still-queued action or fabricate admission for an old send.

Within that same caller write transaction:

- Require the new action kind, queued action status, queued record phase,
  supported schema, exact record/action ID and transaction generation, and exact
  actual Row.wire digest. Revalidate supported immutable request grammar/domain.
- Re-read the requested held checkpoints, restore/validate them and compare full
  normalized version/hash/wire/logical state to the sealed baselines. Requested
  version must still equal baseline version. No current value replaces a stale
  captured one; no request is rebuilt or re-encoded.
- Recheck server backoff and relevant pending writes. These may have changed
  since queueing. Do not allow a fetch over a prepared/uncertain write baseline;
  do not infer pending readiness from a held key or positive version.
- Only after all checks, set record phase Admitted and Row Sent with the same
  reservation timestamp, in the same transaction. A failed commit leaves both
  queued. A crash after commit leaves an uncertain reserved request and its
  admitted provenance, never an automatic replay candidate.

A normal changed baseline must refuse **that intent's** send without inventing
a server `ActionStatus::Error`. Avoid ending receiving or blocking unrelated
queue work merely because this offline intent became stale. The eligibility
result can make bulk/actor scans skip that action and let inspection explain
why. Selected explicit reservation returns the precise refusal. If a durable
local-refusal marker is desired, extend the private record with a separate
static reason/phase; do not forge a server reply or reset the action for replay.
That extra marker is an observability choice, not required for a new framework.

Differentiate normal stale baseline/pending/backoff eligibility from corruption:
missing new-kind metadata, inconsistent queued/admitted phase, digest mismatch,
unknown schema, stored JSON/authentication faults and impossible generation
binding must remain integrity errors. Do not silently skip damaged store rows as
though they were ordinary unsupported network input. Aborted bulk transactions
must not leak previously reserved rows or partial admission phase changes.

## Generation, restart and application boundary

**Confirmed:** `tx.generation()` is the generation used by its reads/writes.
`bump_generation` takes effect in later transactions; reading an old row after
the bump produces authentication failure, not None. The factory must never use
`unwrap_or(empty)` after an error or claim the returned next generation applies
to the old transaction's already-sealed rows.

On restart, Queued+Queued can be rechecked prospectively against unchanged
baselines; Admitted+Sent remains reserved/uncertain and is not resent. Admitted
plus a captured settled result supplies an authenticated local admission record
for future processing. Phase/status inconsistency is not repaired by quietly
backfilling a baseline from current state. A new actor epoch can exist without
replaying old requests or scanning them into a permanent flight latch.

The later saved-result wrapper should load raw request, reply and admission
record under one transaction/view, check admitted phase/status/generation/digest,
and construct the pure binder's claim from that sealed record. That is where
store-origin provenance is established. Per-patch application still owes full
current-baseline comparison and atomic held+cursor commit; admission neither
verifies response MACs nor authorizes a newer server version acknowledgement.
No response downloads, state promotion, continuation query or notification ACK
belongs in this factory/reservation chunk.

## Focused implementation tests

- Queue a held-version request and reopen: raw frozen version fields/digest and
  normalized full baselines agree; action+record commit together, or both roll
  back on a store failure. Held state is unchanged.
- Change held version/hash/map after queueing: reservation refuses the original
  intent, preserves its exact wire and lets an unrelated control action remain
  reservable. Do not let a snapshot-policy refusal mask the version mismatch.
- Introduce a same-collection pending write or server wait after queueing:
  neither queued record nor action becomes admitted/Sent. Unrelated historical
  Sent/control actions are not an account-wide block.
- Test both `reserve_next`/bulk and `reserve_one_in` use the same guard. A failed
  transaction cannot leave admission phase changed while action remains queued.
- Successful reservation atomically marks exactly its action and record;
  second reservation/restart never resends it. Socket write failure preserves
  admitted uncertain history and all unreserved subsequent intents.
- Missing new-kind record, mismatched ID/digest/phase/schema, malformed saved
  map and generation-fenced ciphertext fail without guessed baseline or cleanup.
- Old version-zero/bootstrap actions receive no new record or retrospective
  rebasing, including when a nonzero checkpoint now exists.
- Redacted inspection can report known collection/version, phase and static
  refusal/backoff/provenance status without exposing raw wire, IDs, maps or keys.

No tests were run for this documentation-only design. Native scheduler role,
recovery initialization and connection-owned flight completion remain separate
work; they are not silently defaulted by creating a new local admission record.

## Implemented prospective factory and guard

The subsequent `held_fetch.rs` implements the distinct held-fetch intent and
private generation-sealed v1 record above. Its public `queue` builds explicit
held versions with snapshot policy false from existing validated checkpoints;
queue and record commit together. It reads actual `Row.wire` through the
crate-private actions accessor. Comparing a decoded request with the expected
producer Node is validation only: the stored bytes are never rewritten.

The shared action reservation paths invoke `reserve_in` under their existing
write transaction. Full normalized baseline disagreement permanently records
`LocallyRefused { BaselineChanged }` and returns false; generic scans commit that
local refusal and can reserve unrelated controls. Pending writes or server
backoff return false without changing phase. Integrity/schema/generation/digest
faults are errors. True marks the record admitted before the caller writes Sent;
both writes commit or roll back together. The diagnostic timestamp is not a
server acceptance, writer outcome or connection-owned callback lease.

Selected `reserve_one_in` converts false to Err, so its entire transaction,
including a new local-refusal diagnostic, rolls back. Only a successful bulk/
next scan persists that diagnostic in this path; inspection must not claim
otherwise. Existing production selected reservations are not held fetches.

The public `inspect` returns only schema/generation, static phase/local reason,
a request-fingerprint equality boolean and known collection/version claims.
It prints no raw ID/digest, map, key, request body or response material. Missing
old-kind records return None; missing new-kind metadata is an integrity error.
It does not claim that pending/backoff eligibility has just been recomputed.

Real encrypted-store tests cover reopening/unchanged wire/no replay, queue and
reservation transaction rollback, legacy/missing/corrupt provenance, complete
empty legacy normalization without version reset, same-version logical-map
change and permanent local refusal with unrelated traffic still reservable,
pending/server-wait rechecks, selected-refusal rollback versus bulk persistence,
new-kind metadata integrity faults and generation authentication fencing. Legacy
version-zero requests receive no retrospective record. These are local storage
and reservation checks, not successful network/application or flight completion.

Admission inspection also returns action status, numeric error code and receipt
sequence from the **same** read transaction as its phase; a CLI must not join a
second independently read action list and manufacture a phase/status race.
Queue/inspect/reservation boundaries redact serde serialization messages to a
static diagnostic while preserving `Error::Storage(StoreError::Serialization)`;
other integrity/authentication errors retain their types. A malformed stored
phase or field must not echo its plaintext value. Baseline-refusal warnings
print at most a validated opaque local action ID and explicitly say the private
refusal is durable only if the caller's transaction commits.

## Staged authenticated-action retention

**Confirmed Rust/offline:** `action_journal.rs` provides a crate-private encrypted
transaction-scoped retention primitive for ordered decrypted mutations. The
caller must first authenticate them and establish request/reply provenance.
Exact repeats are idempotent; conflicting identities and generation/authentication
faults refuse. Tests cover reopen, rollback and absence of plaintext at rest.
The existing 409 settlement utility now retains verified actions with this
primitive; see [conflict retention](app-state-catchup-atomicity.md#authenticated-action-retention-2026-11-04).
Successful held-fetch application and its cursor remain unimplemented. Successful per-patch verification and authoritative held-state/cursor
commits remain the next required work.

## Preserved-response provenance inspection

**Confirmed Rust/offline (2026-11-04):** `app-state-response-inspect` now binds
successful new-kind held-fetch replies against their sealed prospective record
and exact stored request in one read transaction. It requires admitted phase,
correct action kind, successful settlement and receipt sequence, validates the
sealed generation/domain/wire, and compares the selected reply with the preserved
settlement before invoking the pure binder. Structural binding refusal is exposed
as `admission_bound=false` alongside original safe structural diagnostics; corrupt
or absent mandatory admission is an error. Other action scopes report null.

This is frozen provenance, not a current-baseline comparison: a later legitimate
held-state update does not erase the historical binding. Tests deliberately change
held state after settlement and prove that a version acknowledgement cannot
promote it. Forged reply input, absent admission and unrequested response
collections are covered. Per-patch MAC verification and atomic state/cursor/journal
application remain unimplemented. No additional APK or live evidence is claimed.
