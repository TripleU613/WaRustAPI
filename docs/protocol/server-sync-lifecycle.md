# Server-sync notification lifecycle

Static audit, 2026-11-04, WhatsApp Android 2.26.43.72; APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
**Confirmed** below means current shipped-code evidence, not live execution.
The [retained evidence](rea/2.26.43.72-server-sync-lifecycle/README.md)
contains complete REA envelopes, checked DEX bodies, limitations and digests.
This extends [the notification obligation audit](notification-sync-obligations.md).

## Dispatch and eligibility

**Confirmed:** `14l.A09` handles `server_sync` (integer 210), validates the
server sender/type through its parser, and forms collection-name/announced-version
entries. It invokes the sync manager only when `0aS.A0c()` is true **or**
`companion_syncd_critical_bootstrap_state == 1`. Eligible entries are each
passed to `0l8.A07(name,version)` before `0aS.A0P()`.

The gate is now traced precisely:

1. `0aS.A0d()` blocks ordinary sync when `0Yz.A09()` is true, meaning saved
   `syncd_dirty` is anything other than its default `-1`; it also blocks when
   `09F.BU7()` and `companion_syncd_dirty` are both true.
2. `0aS.A0c()` returns false when that block applies. Otherwise it requires
   an existing `syncd_mutations` row (`0g1.A0S`) or a pending mutation with
   `is_ready_to_sync = 1`. It does not simply test whether a socket is online.
3. The notification's critical-bootstrap-state alternative can admit the
   dirty-version writes even when `A0c` is false. `A0Q` independently checks
   `A0d` before dispatching work, so admission does not bypass recovery state.

**Confirmed ACK boundary:** `14l.A09` does not defer the base notification
ACK. Processing/scheduling returns before asynchronous sync results. When the
eligibility test is false it returns without dirty writes or starting sync;
the ordinary base ACK path still applies. That narrow no-work case requires
the actual gate state, not an assumption from absent Rust tables.

**Unknown:** exact mapping of all native bootstrap/recovery preferences and
the `BU7` role predicate into this Rust client's role and lifecycle. Creating
these preferences with invented defaults would not establish native eligibility.

## Advertised versions are separate metadata

**Confirmed, `0l8`:**

| Operation | State |
| --- | --- |
| `A01(name)` | Insert-or-ignore: held `version=0`, `lt_hash=NULL`, `dirty_version=-1`. Existing state is preserved. |
| `A07(name,v)` | Require `v>=0`, ensure the collection row, then transactionally assign only `dirty_version=v`. This is assignment, not `max(old,v)`. |
| `A03()` | Select names where `dirty_version=0 OR dirty_version>version`. |
| `A02(name)` / `A04()` | Read held versions; neither reads the announced dirty version. |
| `A08(hash,name,v)` | Update held `version` and `lt_hash`, separately from dirty metadata. |
| `A06(name)` | Change `dirty_version` from `0` to `-1` only. Positive announcements survive this operation. |
| `A09(name)` | Return `dirty_version!=0 AND held_version>=dirty_version`; a missing row uses zero/zero and returns false. |

A positive advertised version is satisfied when authenticated/applied state
catches up; it need not be deleted. A zero dirty version is a forced-fetch
sentinel, not evidence that the client has an authenticated version zero.
Never copy notification versions into `SavedCollectionState` or its LT hash.

## Single-flight scheduling and request version

**Confirmed:** `0aS.A0P` invokes `0gG.A00`, cancels an existing scheduled
sync runnable and clears that handle, then calls `A0Q` while synchronized.
`A0Q` checks `A0d`, invokes `0gG.A00`, and starts work only when its `A02`
in-flight flag is false.

The classic branch (feature 18234 disabled) selects dirty collection names,
loads eligible pending mutations, groups mutations by collection and inserts
empty mutation lists for dirty collections with no outgoing changes. It filters
collection eligibility for the companion role and excludes collections blocked
by missing keys (`0fI.A06.A00`). An empty final map starts no request.
Otherwise it queues `Dxc(0aS,Map)` and marks the sync in flight.

`Dxc` constructor binds this operation to `run` discriminator 4. Empty lists
produce an `EdY` request component without a patch; nonempty lists take the
outgoing encryption path. On successful request preparation `FJU.accept`
discriminator 12 reads each collection's **held** version using `0l8.A02`,
passes it to `EfK.A01`, and sends the assembled request with `FUc` as callback,
request discriminator 238 and timeout 32,000ms. Preparation failure goes to
`SyncResponseHandler.A09`; it is not an ACK-only success.

Feature 18234 enables a separate KMP outgoing processor branch. Both paths
are retained; this trace does not establish which remote feature values apply
to a particular account. `0gG.A00` internals and full `EfK` wire-builder policy
remain explicit follow-up points, not guessed no-ops.

## Response verification, commit and rescan

**Confirmed:** `FUc.CK4` parses the result into `F2T` and schedules
`FeN` discriminator 40. The checked DEX body traces its classic incoming
application path to `SyncResponseHandler.A04` and, after handling succeeds,
`0aS.A06(manager,true)`. The KMP result route is separately retained in
`SyncResponseHandler.A00` and reaches the same manager completion.

`SyncResponseHandler.A04` handles snapshots/patches and `hasMorePatch`.
Snapshots are constrained to companion bootstrap with held version zero.
Its `A07` is a complete **smali fallback**, not usable Java: it checks that an
incoming patch version is the held version plus one; snapshot handling has a
separate path. It reads key IDs and patch/snapshot MACs, rejects malformed
inputs, and contains feature-dependent mismatch handling. Those branches are
not permission to disable existing Rust cryptographic verification.

The successful application path invokes `0mj.A00`, which applies/stores
mutations and calls `0l8.A08` with the incoming version and resulting hash.
The checked `A07` call to `0mj.A00` at byte offsets `125a..1260` precedes
its transaction-success call at `1260..1266`. This is distinct from the
notification's dirty metadata transaction. The full mutation-handler policy
and its Android action side effects are not claimed ported.

When the collection has no more patches, `A04` calls `0l8.A06` to clear
only the forced-zero sentinel. Positive dirty versions continue to compare
against applied versions. `0aS.A06(true)` clears the in-flight flag and invokes
`A0Q` again: an announcement arriving during an active request can therefore
leave dirty work for the next rescan rather than requiring parallel requests.

**Confirmed failure boundaries:** `FUc.Bvj` requires an error child/code,
reads optional integral `backoff` in seconds and converts it to milliseconds,
then queues the error callback. `BsS` schedules a separate delivery-failure
worker; it does not commit advertised versions. `SyncResponseHandler.A09`
distinguishes fatal, failed, retriable, SQLite-full and unexpected failures.
The retained `A01` retry helper clears the in-flight flag before scheduling
delayed work when a delay exists; `A02` failure helper clears the flag without
an immediate rescan. Success clears the transient-failure timestamp and resets
the backoff helper. Exact error-code classifications/delivery worker internals
are still prerequisites for a faithful retry port. Decompiled `A01` contains
an empty conditional; its complete checked DEX is retained rather than treating
the recovered Java as an exact control-flow specification.

## Rust gap and next bounded implementation

**Confirmed source audit:** `transport/app_state_sync.rs` provides collection
verification/application, per-collection saved state, request builders and a
durable action queue. Its `fetch_request` currently asks all requested collections
at version zero with `return_snapshot=true`; `queue_fetch` requests all five.
It is not a native incremental notification scheduler and cannot be reused
unchanged for the held-version path above. There is no native-equivalent
notification dirty-version/gate/in-flight/completion lifecycle in the receive
dispatcher. No code or API behavior changed in this audit.

Next work is bounded by these concrete prerequisites:

1. Trace `EfK.A01`/`EdY` request construction and `0gG.A00`, and map the
   client role, initialized state, ready mutations and recovery/blocking state
   to actual Rust rows. Reuse current retained evidence; do not re-decompile
   `Dxc.run` or the full response handler merely to locate them.
2. Add a version-aware collection fetch primitive using held verified state;
   leave advertised dirty metadata in a separate checkpoint. Establish blocked
   missing-key behavior and a durable one-flight reservation/completion record.
3. Connect eligible notification processing and ordinary ACK only after that
   scheduling state is durably retained. Match the APK's ACK-before-async-result
   behavior while preserving recovery after an ACK write or process crash.
4. Wire verified apply, has-more pagination, rescan, refusal/backoff and uncertain
   request outcomes together. Test newer notifications during in-flight work,
   duplicate/stale/zero announcements, malformed bounds, key absence, failed MAC,
   crash around commit/ACK, and server retry delay. Never turn an unknown result
   into an unconditional request replay.

These are required network-state obligations, not grounds for adding fake
telemetry or broad automatic fetches. The affected operation remains session
listen (`POST /v1/session/listen`, CLI `whatsapp-cli listen`). No live acceptance
or account outcome is established.
