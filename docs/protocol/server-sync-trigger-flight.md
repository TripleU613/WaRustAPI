# Server-sync readiness and flight ownership

Static audit, 2026-11-04. **Confirmed** native findings use WhatsApp Android
2.26.43.72, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[New evidence and bounded caller inventories](rea/2.26.43.72-sync-trigger-flight/README.md)
reuse the preceding [lifecycle](server-sync-lifecycle.md),
[role/recovery](server-sync-role-recovery.md),
[version-aware request](app-state-versioned-fetch.md) and
[logical-index correction](app-state-logical-index.md) records. No runtime,
network or account operation occurred. This audit changes no scheduler code.

## The native flight owns a job and its assembled IQ

**Confirmed:** `0aS.A0Q` checks recovery eligibility, enters a synchronized
region, and prepares work only when `A02` is false. In the classic branch it
collects dirty collections and ready mutations in a map keyed by collection.
It queues `Dxc(manager,map)` and sets `A02` true when the final map is nonempty.
The KMP branch also derives the flag from whether it scheduled nonempty work.

`Dxc.run` discriminator 4 prepares one result for each map entry, including
empty mutation lists for collection fetches. `FJU.accept` discriminator 12
iterates those results through **one** `EfK` builder and sends **one IQ** with
multiple collection children. An empty list produces a fetch child; a
nonempty list produces a patch child. Thus the manager excludes parallel
sync jobs across collections, and one admitted job can carry both fetches and
writes. It is not one flag per collection child or one flight per mutation.

The explicit Rust queue currently emits separate IQs for its full fetch and
per-collection prepared patches. `queue_patch` protects the same collection's
pending baseline, but `actions::reserve_pending` can reserve up to 16 queued
actions, including several app-state IQs across different collections. A
one-IQ-per-batch limit alone would still permit overlap on the next reservation
call before a result arrives. This is a real scope difference from the traced
native manager, not evidence for locking every unrelated IQ.

## Complete direct flight field inventory

`apk-field-refs` finds eight direct DEX instructions for `0aS.A02:Z`:

| Method | Direct access |
| --- | --- |
| `A0Q` | One read and one write: admit/mark scheduled work. |
| `A06(manager,rescan)` | One write: set false, then optionally call `A0Q`. |
| `A0R(reason)` | Two reads: defer destructive sync-state work until safe. |
| `A0T(state)` | Two reads: deferred state handling. |
| `FdE.run` | One read: its worker observes the current flight. |

No constructor write occurs: the boolean starts at Java's false default. There
are only two direct writer methods in this scan. This is static DEX coverage,
not a claim about arbitrary external/reflection/native writes.

The five direct callers of the clearing method `A06` are:

- `SyncResponseHandler.A00`: KMP success processing, then rescan.
- `FeN.run`: classic result handling, then rescan.
- `SyncResponseHandler.A01`: retry scheduling, clear without immediate rescan.
- `SyncResponseHandler.A02`: failure, clear without immediate rescan.
- `SyncResponseHandler.A08`: fatal failure, clear and enter recovery handling.

Success releases the flight **after** handling its result, including native
stored/pending/version updates. Receiving an IQ with a matching random ID is
not enough to release an accepted-write baseline. `A0b` handles a deferred
`EZB` state object only when present, invokes its operation and clears that
object; this is distinct from claiming all persisted sends are live flights.

## Lost response and timeout lifetime

**Confirmed chain:** `FJU` sends the classic sync IQ with callback `FUc`,
discriminator 238 and timeout 32,000ms. `0Sf.A0R` tries the normal callback send;
if not immediately ready it registers pending state and queues the IQ instead.
The already-retained `0ln.A08` registers a delayed handler message for a
positive timeout at callback registration, including the queued-send path.

On timeout/failure `0m2` reaches `0ln.A03`, removes the pending callback and
schedules `8bj.run` to invoke `BsS`. `FUc.BsS` queues `Feb(this,4)`.
Its discriminator-4 body calls `FUc.A00` to detach the old request context,
then invokes `SyncResponseHandler.A01(delay,"delivery_failure",false,true)`.
The checked `A01` control flow clears the manager flight, handles a pending
recovery state if present, and schedules delayed retry when the delay helper
returns a value; otherwise it follows its failure path. The recovered Java has
an empty conditional, so the preceding checked DEX is the control-flow source.

`1C8.A01` increments a bounded retry counter, returns null when exhausted, and
otherwise computes capped exponential jitter before switching to a capped
delay. Constructor parameters selecting its cap/attempt count for this manager
remain untraced; do not invent those values. Server-provided backoff is another
existing input, not an instruction to mark an uncertain write successful.

**Critical distinction:** `A02` is an in-memory manager/job flag. Its lifetime
includes preparation before callback registration, so 32 seconds is not proven
to start when the flag becomes true. It is not a durable scan over every
historical request sent by an account. Process restart creates a new default
false manager, while native pending mutations/recovery preferences survive.
The Rust no-replay contract is stricter than blindly rebuilding an uncertain
prepared write; that contract must be preserved when implementing a flight.

## Other IQs and the namespace boundary

The exact namespace string has four current DEX refs: token dictionary,
`BVw` recovery deletion builder, `EfK` sync builder and a `Dzb` constructor.
`Dzb(String,String,int)` has a separate coex digest branch with
`coex mode=digest` and optional `coex_dirty_hash`; another discriminator branch
uses the push namespace. The generic constructor caller list includes overloads
and unrelated operations, so it does not establish digest admission/lifetime.

**Unknown:** full concurrency ownership of that independent coex digest
producer. It is not an `EfK` collection child. Likewise `BVw` recovery deletion
uses different state gates and a separate callback. A blanket lock based only
on `xmlns=w:sync:app:state` is therefore unproven. Ordinary startup reads in
other namespaces do not acquire this manager's `A02` through the traced sync
builder. The proposed scope is the supported collection-sync operation, not
generic IQ serialization or a replacement for the independent startup audit.

## Readiness mapping after the logical-index correction

**Confirmed source/data transition mapping:** a complete authenticated current
logical-index map now represents surviving SET rows for the supported port;
REMOVE erases its row, and orphan/unsupported SET rows still count. A positive
version, a key, or an empty collection checkpoint alone is not row presence.
Legacy nonempty maps without authenticated logical provenance remain a distinct
inspection/reconstruction state, not an inferred ready collection.

Native ready pending mutations are inserted with ready=0 and explicitly marked
ready=1 before encryption. Rust queues an already encrypted request with a
frozen baseline/prepared checkpoint. Therefore a Rust `Queued` action plus a
valid current `Bound` baseline, authenticated original wire and prepared result
can represent the port's **supported ready work**; `Queued` alone cannot.
This is an explicit porting correspondence, not identical native table schema.

| Rust action/pending condition | Scheduler meaning |
| --- | --- |
| `Queued` + validated bound current baseline | Prepared ready packet; not yet a socket reservation. |
| `Queued` without valid pending provenance | Orphan/malformed preparation; reservation must refuse. |
| `Sent` | Reserved before socket write; outcome remains uncertain until a correlated response. It must never be turned back into queued merely to release a flight. |
| `Ok`/`Error` with pending state | Response captured; feature settlement must validate/promote/catch up/release atomically before that pending baseline is complete. |
| No pending row after settlement | Completed/cancelled preparation; not pending readiness. |

The transport's plain-name held checkpoints and storage staging's hex-name
checkpoints are separate namespaces. Do not OR unrelated states for the same
collection or merge them silently to create eligibility. The authoritative
producer/consumer for each active flow must be selected explicitly.

Rust store records are authenticated to `WriteTx::generation`. A generation
bump fences old ciphertext; the store's regression expects authentication
failure for old rows, not an absent/clean replacement. Registration status,
last-authenticated time and missing new scheduler metadata do not prove native
recovery or bootstrap completion. Existing native role/recovery initialization
gaps remain in the preceding note.

## Proposed implementation boundary

Do not implement a global guard by counting all durable `Sent` actions: it
would turn unknown history into a permanent flight and unnecessarily block
independent work. The evidence instead supports a collection-sync **owner and
runtime epoch**, one active assembled IQ/job, and explicit phase/deadline state.
Persisted reservations still preserve the old request's uncertain outcome and
generation fencing; clearing an expired/live owner is not request replay.

Admission must be shared by automatic and explicitly selected reservations,
including multiple calls/batches. Success releases only after authenticated
result processing; timeout/delivery failure releases the live owner according
to a classified callback and delay policy, preserving pending writes for
reconciliation. Independent startup reads and untraced same-namespace operations
must not be swept into this guard. The exact connection/owner lifecycle and
uncertain-request reconciliation require coordination before code changes.

The automatic `server_sync` trigger additionally needs validated sender/type/
collection grammar, actual role/recovery/bootstrap gates, dirty-version
retention, response verification/pagination and completion rescan. Existing
version-aware fetch construction and MAC-map correction do not by themselves
complete that receive path. No generic ACK-only shortcut or invented eligibility
default was added here.
