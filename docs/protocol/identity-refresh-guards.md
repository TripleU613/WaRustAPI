# Original identity guards for notification refreshes

2026-10-09. **Implemented/offline:** every newly queued
`identity_refresh` saves immutable original identity provenance in the same
transaction as its query and notification event/device markers. This advances
[notification key reads](identity-notification-key-read.md) toward eventual
response adoption. It does not yet implement native admission, a persisted
session refresh or notification ACKs.

The [application follow-up](identity-refresh-application.md) now rechecks these
guards inside the transaction committing a supported changed-primary reply.
Missing historical snapshots still cannot authorize application.

## Evidence and the stale-response problem

**Confirmed static context:** APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The retained [0Z7.A00/A02 evidence](primary-device-version.md) reads a nullable
per-user primary-device version and increments it on primary identity changes.
It does not supply an exact companion transition counter. No native
notification-time snapshot equivalent is claimed by this change.

**Confirmed local:** current-pin equality alone cannot distinguish an unchanged
pin A from A->B->A while a directory request was outstanding. The existing
primary counter catches tracked primary changes, but not companion changes.
The response may be correctly signed by A while representing an older lifecycle.
A generic correlated result or valid signature is not permission to adopt it.
A directory query also does not freeze legitimate send/receive ratchet progress.

The local snapshot uses these fields, encrypted in Checkpoints:

| Field | Meaning |
| --- | --- |
| peer | Original exact PN/LID device, canonical wire kind |
| pin_fingerprint | SHA-256 of the original serialized saved identity |
| generation | Original local account generation |
| transition | Nullable last authenticated exact-device identity-change event ID |
| primary_version | Nullable current primary counter for this exact user namespace |

These are local concurrency/provenance guards, not native attestation state,
server epochs, safety numbers, signed-list authority or a verification decision.
Null is a captured absence at queue time, not zero historical changes. Required
nullable JSON members cannot silently default when missing. Snapshot data never
appears in inspection output or public validation artifacts.

## Authenticated transition provenance

`identity_conflicts::archive_identity` now records the new IdentityChanged
journal event ID as the exact-device transition marker. Both authenticated
incoming-message and signed-directory replacement paths already use this shared
archive hook. Incoming direct/group commits and outgoing direct/group bundle
commits were inspected at `inbox`, `outgoing` and `group_fanout`: replacement
hooks share their pin/session/reservation transaction. No new authority to
replace a pin was added. Notification tags and unauthenticated conflict
observations cannot advance this marker.

A real changed old pin produces a unique event and marker; initial acquisition
and identical keys return before that path. A->B->A therefore ends with a
different marker even when the pin equals its original bytes. PN/LID users and
companion indices stay distinct; wire kind alone does not split equivalent
device addresses. The marker and archive/event/follow-up writes roll back
atomically on failure. Malformed stored transition provenance refuses with
`invalid saved identity transition`; it is not overwritten to repair a failure.

**Unknown:** historical transitions before this instrumentation. Existing
archives are not scanned or projected into a guessed latest event. Manual,
unversioned private-store edits are outside the authenticated replacement
contract; this marker is not a general detector of every possible database edit.
Existing own-account protections and peer identity-change policies remain the
callers' admission gates. This marker itself is not an ownership policy.

## Inspection and preview behavior

```sh
whatsapp-cli --profile PROFILE actions --show ACTION_ID
whatsapp-cli --profile PROFILE actions --show ACTION_ID --preview-session
```

**Implemented/offline:** `Inspection.refresh_guard` exposes one fixed value:

| Value | Meaning |
| --- | --- |
| untracked | Original snapshot absent; historical provenance unknown |
| matches | Original pin/transition/primary version/generation currently match |
| pin_changed | Current pin changed or disappeared |
| transition_changed | Exact-device authenticated transition changed, including A->B->A |
| primary_version_changed | Same-namespace user's primary counter changed |
| generation_changed | Local account generation differs |

The ordered checks are snapshot binding/shape, generation, pin, transition, then
primary version. Missing snapshot returns untracked before comparison. A
malformed or differently bound snapshot refuses with
`invalid saved refresh snapshot`; no raw peer, malformed value or credential is
included. Snapshot and comparison use the same read transaction as the action,
reply and pin inspection. A matches result can become stale later; it is not an
atomic commit gate or native CPB admission.

Duplicate notification event IDs return their original action without capturing
again. Refused/invalid/uncertain earlier queries retain their previous no-retry
behavior. A distinct event after a valid signed read can capture a fresh current
baseline for a new read, even if the older baseline is stale. It cannot alter or
replay the old event. Failed snapshot capture rolls back the queued action and
all associated markers; it never leaves a partly prepared query.

The [session preview](session-refresh-preview.md) skips known stale guards with
`outcome=stale_refresh_guard`, retaining signature/pin diagnostics but deriving
nothing. Historical untracked reads may still exercise the original ephemeral
cryptographic preview; they remain untracked and cannot establish notification-
time provenance or commit authority. Preview always reports admission unevaluated
and persisted false. A missing pin on a newly captured read now diagnoses
pin_changed/stale_refresh_guard before candidate preparation.

## Verification and remaining work

Generated fixtures cover PN/LID primary and companion A->B->A transitions through
both authentication sources, a still-valid signed A reply, duplicate preservation,
new-event baseline capture and restart. Namespace/other-device changes leave the
original guard matching. Companion reads detect their user's primary change;
held ratchet updates alone do not invalidate identity provenance. Missing
historical snapshots stay absent after duplicate queueing. Missing required-null
members, malformed IDs, wrong bindings and extra fields refuse without leakage.
Generation mismatch and malformed primary counter capture are exercised. A
fake actor confirms the baseline is saved before the uncertain first socket
write; after reopening, the same snapshot and sent reservation remain, and
redelivery creates neither a new query nor an identity ACK.
Malformed transition provenance refuses archive writes; transaction rollback and
reopen preserve the committed marker. Synthetic stale-preview refusal leaves
pins and sessions equal. These tests establish local behavior, not fresh server
identity transitions or full native parity.

**Unknown / unported:** automatic native CPB/PQ/companion admission, authenticated
attestation-state storage/producers, alias/mapping updates, response adoption
with a final write-transaction recheck and deferred identity notification ACKs.
A future commit must recheck original identity provenance *and* the exact current
session staged for it. This change deliberately does not freeze the session at
query time or backfill old reads. No new live query is required to inspect
historical provenance; retained-account offline validation is recorded separately.

## Retained-account offline inspection

**Observed locally:** the current CLI inspected the previously saved
primary-LID key read from commit `41e66e3`. It retained verified_bundle,
valid signature and matching pin, while correctly reporting
`refresh_guard=untracked`. Before/after `store inspect` JSON compared equal.
No connection, new query, baseline backfill, pin/session write or notification
ACK occurred. [Redacted validation](identity-refresh-guard-validation.json)
contains no account/peer/action ID, fingerprint, transition selector or raw
wire data. Summary equality is not a byte-level account-row comparison; synthetic
store/restart tests establish the stronger local invariants. New guarded queries
and identity transitions have not been freshly validated live in this chunk.

The first inspector invocation failed at argument parsing (exit 2): default-
feature CLI integration tests had replaced target/debug/whatsapp-cli, so actions
was unavailable. Store inspection itself worked. Rebuilding with experimental
transport after those tests and checking actions --help fixed the local tool
selection; no state reset or network retry was involved. The corrected offline
inspection is the validation above. For reproducible manual checks, select the
feature explicitly with cargo run --locked -p whatsapp-cli --features
experimental-transport -- ..., or rebuild after default-feature test runs.

## Completed checks

858 workspace/all-feature tests passed (API 717, codec properties 4, crash
matrix 9, experimental CLI 54, API symbols 1, research tooling 73). Strict
workspace/all-target Clippy and warnings-denied Rustdoc passed. Default CLI
passed 25 tests plus one API-symbol integration test; experimental CLI reference
regeneration passed. Formatting and staged whitespace passed. Repository hygiene
verified 2,317 local links and 115 retained APK bundles / 23,735,849 bytes.
This chunk adds local guard behavior over previously retained APK context; no
new static export or fresh live identity transition is claimed.
