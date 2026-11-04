# Connection-owned collection-sync flight design

Static source audit, 2026-11-04, against main `92c3a96`. **Confirmed Rust**
means the checked-in implementation, not a live observation. **Proposed** means
an integration contract requiring implementation and tests. No network request,
APK execution, source behavior change or new APK export occurred in this audit.

The **confirmed native** basis is [the retained flight trace](server-sync-trigger-flight.md):
WhatsApp Android 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
`0aS.A0Q/A06`, `Dxc.run`, `FJU.accept`, `FUc`, `SyncResponseHandler` and
`FeN.run`. The native manager owns a job which produces one potentially
multi-collection IQ; its flag is not a durable scan of historical sends. Its
32,000ms callback timeout begins at callback registration, after preparation.
Do not reinterpret that as a 32-second preparation timeout.

## Actual reservation and timer paths

**Confirmed Rust:** `transport/connection.rs::drive` has one `ConnectionEpoch`
and single `tokio::select!` loop. The two generic action drains are activation
success and the heartbeat arm. The one-second outbox arm handles message/key
preparations, not generic queued action draining. Heartbeat cadence is
`clamp(timeout_ms / 2, 1000, 20000)` milliseconds; it is not an app-state
callback deadline.

Both generic drains call `actions::reserve_pending`, which marks up to 16 rows
`Sent` in one transaction before the loop writes them. The loop awaits
`action_delay_ms` between requests inside its selected branch. While that sleep
is pending, the actor does not dispatch inbound replies or another timer arm.
`NodeLink::send` also awaits writer completion. Thus adding a deadline checked
only on heartbeat, or limiting each batch to one app-state request, would not
establish a timely flight across repeated calls. This is a source-level
scheduling observation; no actual timeout or queue delay was measured.

| Existing surface | Integration requirement |
| --- | --- |
| Activation and heartbeat `reserve_pending` drains | Both use the same actor-owned coordinator, never independent local guards. |
| Public `actions::reserve_pending` | Currently accepts fetch and patch kinds without an epoch/flight owner. A new actor-only path leaving this public bypass unchanged cannot enforce the contract. Preserve generic control-action use, but route collection sync through an explicit owner-aware reservation surface, with a documented compatibility decision for existing callers. |
| Crate `actions::reserve_one_in` | Also admits app-state kinds today. Its current production caller is `outgoing_group` reserving a `group-info` request; this independent query must stay available. Future selected app-state admission must use the same owner-aware path, not this unrestricted helper. |
| `actions::queue`, `queue_in`, `app_state_sync::queue_fetch/queue_patch` | Queueing is offline preparation, not flight acquisition. Do not acquire a runtime lease merely because a frozen packet exists. |
| CLI bounded `lab-transport` loop | Its receipt drains are separate; this audit found no generic action reservation there. Do not claim adding the coordinator to `drive` automatically covers every possible public API consumer. |

The production-call inventory excludes test modules. Current direct generic
reservation call sites are those two `drive` branches; the selected helper's
production caller is the group-info path above. This inventory does not
constrain downstream users of the public library.

## Proposed bounded actor integration

The following is a **design**, not implemented support for automatic server
sync. Preserve existing immutable queued wire bytes; do not merge/re-encode
separately prepared requests to imitate the native multi-collection builder.
One queued collection-sync IQ is one supported job in this port.

1. A coordinator belongs to the active account actor and owns at most one
   collection-sync job. Its token includes account generation, connection
   epoch, opaque action ID, affected collections and phase. Historical `Sent`
   rows do not acquire this token. Concurrent actor/public reservation ownership
   must be decided explicitly before claiming process-wide exclusion.
2. Select one eligible app-state action in the reservation transaction, using
   current server backoff and prepared-baseline validation. Acquire the live
   token with that admission. Reserve unrelated IQs independently. Prevent a
   second app-state admission across activation, heartbeat and selected calls.
3. Model preparing/reserved, callback-registered/writing, awaiting-result and
   processing phases separately. Register the result correlation and monotonic
   `Instant + 32s` deadline before handing the IQ to the writer, so a fast
   response has an owner. This local send boundary is a proposed counterpart of
   native callback registration; it is not proof of identical queue internals.
4. Pace writes through a separate `sleep_until` branch/state, rather than
   sleeping through an entire batch. Keep receive, stop and flight-deadline
   dispatch responsive. A pending writer completion needs explicit cancellation
   semantics: a timed-out future must not leave an untracked queued write that
   later runs while a replacement job is admitted. Ending/poisoning that epoch
   is safer than assuming cancellation proves non-delivery.
5. Correlate replies by exact live ID and epoch/generation. Capture the reply,
   process all affected collections, and commit held/pending/recovery changes
   before normal release/rescan. Independent controls, receipts, heartbeat,
   device reads and group-info requests never acquire this collection token.
6. On delivery failure/deadline, release only the live runtime ownership through
   a classified outcome. Preserve original `Sent`, frozen wire and uncertain
   pending baseline; never requeue/replay. A late reply can still be captured
   and reconciled with its original durable action, but cannot release a newer
   job's token. Disconnect drops the epoch owner, not durable history.

A monotonic deadline is runtime state, not a persisted wall-clock timestamp.
A persisted diagnostic may record an epoch/outcome, but cannot resurrect an old
`Instant` or imply an uncertain write was rejected. Existing wall-clock
server-backoff semantics remain separate.

## Result processing is not yet a complete scheduler

**Confirmed Rust:** `actions::record_reply` captures a correlated `Sent` result
or error and assigns receipt order in its own transaction. `drive` then calls
`app_state_sync::ingest_action`. For prepared patch actions, that method
validates original request, held baseline, key authority and response before
atomic promotion/pending deletion; the bounded 409 catch-up path applies verified
patches. An `InvalidInput` refusal is logged and the pending baseline survives.
A reply captured as `Ok`/`Error` is therefore not equivalent to completed feature
processing. Startup currently repairs contacts/identity projection, not all
app-state captured-but-unprocessed settlements.

`queue_fetch` requests all collections at version zero. It has no app-state
pending baseline; `ingest_action` immediately returns zero if no pending row is
found. Consequently its matching IQ can be recorded as a settled action without
installing snapshots, advancing collections or retaining fetch-specific backoff.
Neither zero returned by ingestion nor generic action status proves successful
fetch completion. The implemented version-aware request builder is not a
version-aware durable fetch scheduler.

Before complete trigger/refetch integration, the following obligations remain:

- **Per-collection dirty generation:** retain advertised dirty version separately
  from authenticated held version; coalesce repeated notifications without
  fabricating progress; compare the admitted generation again at completion and
  rescan if dirtiness changed during the flight.
- **Authoritative eligibility:** use authenticated logical-map presence plus
  validated ready work for the supported correspondence, without interpreting
  missing legacy provenance, missing rows, a held key or a positive version as
  native readiness. Transport plain-name checkpoints and storage hex-name
  staging checkpoints remain separate authorities.
- **Role/bootstrap/recovery ownership:** generation-bound initialization,
  authenticated primary/companion role, critical bootstrap state and recovery
  markers still need the native transitions documented in
  [role/recovery](server-sync-role-recovery.md). No default-clean migration or
  automatic delete-all-data operation is justified.
- **Fetch response engine:** validate requested collection membership/uniqueness,
  versions and held keys, authenticate snapshot/patch data, handle pagination
  and external blob/key prerequisites, atomically update the correct held
  namespace, and classify errors/backoff/fatal recovery before release. The
  bounded offline same-version reconstruction helper cannot substitute for it.
- **Captured settlement recovery:** replay local processing (never network
  sending) in durable receipt order after a crash. Distinguish completed,
  malformed/quarantined, retryable-read and uncertain-write outcomes. A malformed
  captured response must not silently release as success or create a permanent
  account-wide latch. Its affected collections/pending writes remain fenced for
  explicit reconciliation while unrelated work remains available.
- **Timeout retry policy:** native retry cap/constructor parameters and full
  recovery/error transitions are not all traced. Clearing the volatile owner
  is not authority to retry a frozen write or invent an exponential schedule.

## Next bounded implementation and verification

**Proposed:** first separate owner-aware collection admission from generic
control reservations, add the actor token/result/deadline mechanism and retain
clear diagnostics for unsupported fetch/result lifecycle. Coordinate API
compatibility and ownership before code edits. This can correct concurrent
supported prepared writes; it must not advertise complete automatic
`server_sync` processing while the obligations above remain open.

Meaningful tests should inject a fake link and paused monotonic time: activation
plus repeated heartbeat cannot overlap collection jobs; independent IQs still
leave; pacing does not starve inbound replies; callback deadline starts after
admission/preparation at the specified boundary; captured-but-unprocessed replies
and malformed responses never promote state; timeout/write failure preserves
original bytes and pending state; late old-epoch replies cannot clear a new
owner; restart does not replay historical `Sent`; and generation changes fence
old actions. Existing prepared baseline/backoff/mixed-key tests remain necessary.
No new tests were run for this documentation-only audit.

## Separate bounded correction: generic action pacing

**Proposed, independent of scheduler completion:** replace each blocking batch
with actor-owned drain state shared by activation and heartbeat: remaining
budget (at most the existing 16), next-send monotonic deadline, and readiness.
A dedicated `select!` branch reserves at most one action only when it is due,
then writes exactly that reserved request. Start/refresh the drain only at the
existing activation/heartbeat trigger points; a permanently enabled one-action
branch would silently change the existing work cadence and batch limit. Do not
stack budgets when a heartbeat occurs during an active paced drain. Newly
queued actions need not be reserved until their send opportunity.

After a successful write, consume one budget slot and schedule the next due
instant using `action_delay_ms`. If the queue has no eligible work, end that
drain without busy polling. `delay_ms=0` still permits the receive/select loop
between individual writes. Define whether a new heartbeat may reset pacing
while the preceding drain is active: preserving one shared due instant avoids
accidental near-simultaneous writes and simplifies verification. Document any
intentional cross-batch minimum because the current first request in each
separate batch is immediate.

This removes the batch sleep and reduces speculative `Sent` reservations. It
cannot eliminate `NodeLink::send`'s own socket-completion await, nor prove
app-state result completion or native single-flight behavior. Cancellation of
a send, independent writer ownership, control-IQ priority and eventual
collection-coordinator admission remain separate concerns. The one-second
message-outbox branch should not become the owner of this generic queue merely
because it already has a timer.
## Implemented bounded action pacing correction

The generic drain correction is now implemented independently of the proposed
app-state job coordinator. `actions::reserve_next` shares the exact private
`reserve_limit` scan, backoff and prepared-state validation with the existing
public bulk API. The connection uses only the one-action helper. Actions beyond
the selected write remain `Queued`; an uncertain write never resets its `Sent`
reservation or causes it to be selected in a replacement connection.

Activation and heartbeat still trigger at most sixteen successful generic
action writes per drain. An active drain ignores additional triggers without
stacking budgets or resetting its monotonic deadline. An empty eligible scan
ends the drain. Zero delay returns through `select!` between writes, rather than
spinning on an empty queue. Pacing is measured after each successful action
write, and its wait is a selected timer rather than a sleep inside a batch arm.

Heartbeat IQs and independent startup key/device work can now precede queued
generic actions. This is an intentional local scheduling change: authenticated
readiness is not an action-queue flush, and a manual action delay must not hold
heartbeat processing behind an entire batch. Native mandatory precedence among
those independent requests remains unproven. No registration/authentication,
rate-limit bypass, new request intent or automatic resend is introduced.

`NodeLink::send` still awaits socket-write completion; pacing responsiveness
does not establish cancellation or receive progress while that write itself is
blocked. Other bounded receipt/startup write loops and full app-state
flight/result/timeout/generation ownership remain separate required work.

Offline verification includes a fake-link seventeen-action queue: exactly one
row is reserved before its write outcome; a correlated reply settles during a
configured five-minute pacing gap; stop finishes within one second; a failed
write leaves later rows queued and a replacement connection skips the uncertain
ID. A paused-clock zero-delay case sends sixteen, retains the seventeenth until
the next heartbeat, and observes heartbeat before that later drain. A drain-state
regression checks that repeated triggers neither stack budget nor reset due time.
All 1,181 workspace/all-feature tests and strict Clippy passed. No live account
traffic was used, and the broader coordinator proposal above remains unported.
