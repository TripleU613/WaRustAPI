# Media HTTP outcome persistence

2026-11-04. **Confirmed Rust defect/fixed offline:** ordinary media GET returned
a definite non-200 response but retained only the pre-send `outcome_unknown`
reservation. `transport/media.rs::open` now commits `server_refused` plus numeric
`http_status` before returning its error. A later invocation reports that stored
status and does not issue another request. Transport interruption, length or
authentication failures retain the existing uncertain reservation. No response
body, URL or arbitrary header is printed or added to refusal records.

The private transition requires an existing `outcome_unknown` reservation; it
cannot create an unsolicited outcome or overwrite a downloaded/settled record.
Storage/authentication failures propagate. Tests use a real encrypted store and
cover reopen, missing reservation, redirect/refusal/rate-limit/server-error status
retention and protection of other outcomes. No live media request was made.

**Confirmed local behavior:** the shared client disables redirects, automatic
retries, proxies, referer and decompression; requires HTTPS and authenticated
TLS; bounded streaming precedes ciphertext/MAC/plaintext verification. These are
source observations, not claims of native header/timing parity. View-once GET
already persisted numeric refusals in its separate state machine; this change
only affects ordinary media.

**Unknown/current-build audit queue:** conditional media headers, host selection,
expiry, native retry scheduling, account-specific transport choices and complete
2.26.43.72 download call flow remain untraced. Existing 2.26.37.71 media schema and
crypto evidence does not establish current-build HTTP behavior. No new APK
evidence, server acceptance or restriction-cause claim is introduced.

## View-once preflight (2026-11-04)

**Confirmed Rust defects/fixed offline:** `download_view_once` previously lacked
the ordinary media path's config validation and registered-account/phone match
check. It now performs them before inbox selection, cache return or reservation.
A descriptor's encrypted hash was checked before GET, but an absent/malformed
plaintext hash could still consume a one-shot download and later fail decryption.
Both hashes must now be 32 bytes before any network reservation.

Real-store async tests prove missing or mismatched account/config refuses without
checkpoint changes; invalid timeouts refuse before selection. Descriptor tests
cover absent, short and long plaintext hashes. These are local account and crypto
prerequisite checks, not recovered native account predicates or evidence of a ban
trigger. Native HTTP behavior remains in the current-build audit queue above.
No live account operation, cache deletion or automatic replay is introduced.

## Upload response-header persistence follow-up

**Confirmed source defect/fixed offline:** `image_upload::upload_prepared`
previously obtained HTTP status before bounded response-body streaming but wrote
`http_status` only after the body finished. Interruption or a body exceeding the
local 64-KiB bound lost an already-observed status behind outcome_unknown.
It now persists `response_headers_received` with that numeric status before
body streaming. The transition requires the original outcome_unknown reservation
and preserves encrypted upload material and binding fields. Existing settled
outcomes cannot be overwritten. A body interruption or local size refusal leaves
this header observation durable, without implying upload success or permitting
replay. Complete bodies still move through response_received before validation;
only a fully validated result can become uploaded.

Real-store tests cover 200/302/403/429/503, missing reservation, reopen with no
completed body, binding preservation and replay refusal after header receipt.
They also protect completed/refused rows against overwrite. No interrupted upload
or other live network request was run.

## Explicit auth lifetime validation (2026-11-04)

**Confirmed Rust defect/fixed offline:** upload route selection parsed `auth_ttl`
with `and_then(...).unwrap_or(ttl)`, treating present invalid or overflowing
auth lifetimes as absent. It now falls back to route TTL only when the attribute
is absent. Present malformed/non-text auth TTL refuses. Zero, negative and expired
lifetimes continue to refuse through the existing age/minimum-lifetime guard.

A real-store encoded-route test covers absent and valid lifetimes, empty/malformed/
overflow values, zero/negative values and auth expiry shorter than route expiry.
Route inspection leaves every checkpoint unchanged and performs no refresh or
upload reservation. This was the then-existing local fallback, not a native-parity claim. The
subsequent pinned callback trace below **refutes** that absence fallback; the
current implementation now requires both lifetimes. Native expiry parsing/refresh policy
remains in the current-build audit queue. No live traffic occurred.

## Pinned native lifetime correction (2026-11-04)

**Confirmed static and corrected:** current `1xY.CK4 -> 0Tp.A07/A0L/A09`
requires both ttl and auth_ttl and parses each with Java Long.parseLong.
The previous missing-only fallback was a local assumption and is now removed.
Rust reuses the existing Java-compatible signed-decimal helper for both fields.
The encoded-route regression now refuses missing auth_ttl and admits supported
leading-plus/BMP digit spellings while preserving expiry/invalid-value guards.

`1t5.<init>` converts lifetimes to milliseconds relative to a supplied base;
`0VM.A08` conditionally restores a cached route only while its auth deadline
is future. [Six REA envelopes, checked DEX, hashes and exact limits](rea/2.26.43.72-media-route-lifetimes/README.md).
Base origin is closed by the request-clock follow-up below. Full refresh/deadline
consumers, effective restore gate and host selection remain unknown. No automatic query/retry or live traffic is introduced.

## Request-time route expiry (2026-11-04)

**Confirmed pinned static/corrected Rust:** `0VM.A07` records the monotonic request
handoff time in `1xY.A00`, which CK4 supplies to the lifetime model. Rust previously
replaced its initial reservation timestamp with receipt time, extending freshness
by response latency. The CLI now retains requested_at explicitly in both states;
route selection requires it and measures age from request time. Legacy routes
without request provenance refuse with explicit-refresh guidance. They are never
rebased to now or replayed. Receipt at remains diagnostic/selection metadata.

An encrypted-store delayed-receipt test proves a fresh receipt cannot rescue an
expired request, and missing/future/zero request times cannot supply eligibility.
[REA, checked request path and field census](rea/2.26.43.72-media-route-lifetimes/README.md#request-clock-origin-follow-up).
The local pre-write wall-clock base is conservative, not complete native elapsed
clock parity. Full throttle, last_id and host refresh behavior remain open.
No live operation occurred.

## Conditional prior-host retention (2026-11-04)

**Confirmed pinned static:** CK4 defaults is_new to integer 1 and treats other
integers as false. `0VM.A0A` installs the whole reply when no current route exists
or is_new is true. Otherwise it builds a route with the **old host list** and
new auth, IDs, lifetimes, base and other response metadata. `A0P` is a locked
getter with no expiry filtering; the request path obtains last_id from that
current object's A0B. This does not prove cache restore installs expired objects.

**Confirmed Rust support gap/guarded offline:** cached route selection previously
ignored is_new and selected reply hosts regardless of prior-route semantics.
It now parses the flag with the Java-compatible integer helper/default and
explicitly refuses non-new replies because no frozen prior-host context exists.
This is a local unsupported boundary, including the native first-route false
branch, not a claim that native refuses such replies. Stored wire/checkpoints
remain unchanged and no old token/host is silently substituted.

The required implementation is prospective prior-route capture, conditional
last_id request construction, and atomic merged-route projection with reply
generation/identity checks. Merely choosing an older saved result after a refusal
would pair stale auth and hosts incorrectly. Full refresh/expiry/throttle/runtime
coordination remains unfinished. Real-store flag tests cover default, 1, plus/BMP
spellings, other integers and malformed/overflow values with no writes or traffic.

## Pure host projection implemented (2026-11-04)

**Confirmed Rust/offline:** `media_routing::project_media_conn` implements the
traced 0VM.A0A host-selection operation: first route or new update selects
incoming hosts; non-new update with a prior object selects its exact host nodes.
Every resulting attribute remains from the incoming object, including auth,
TTL and ID. Host order and nested upload/download metadata remain intact.
Inputs are unchanged. Unknown incoming non-host children remain in the caller's
original response; the projection contains only interpreted host-list children.
Duplicate attributes and local structure/count bounds are explicit local policy.

`is_new_route` centralizes the native default/integer parsing. Cached upload
selection now uses these helpers for its supported new-route branch. Its existing
non-new refusal remains: a caller-supplied prior node is not proof of prospective
store origin. No durable prior capture, conditional last_id, merged-cache commit
or native refresh owner is claimed. This is the merge operation for that later
transaction wrapper, not a retrospective way to unlock old cached replies.

Three handwritten tests cover all first/new/non-new branches, exact old-host/new-
metadata separation, unchanged originals and invalid/ambiguous inputs. Existing
real-store route tests cover the public upload selection guard. Full host parser,
permissions, expiry and cryptography remain separate obligations. No live traffic.

## Prospective refresh integration (2026-11-04)

**Confirmed Rust/offline:** `media_routing::refresh` now wraps the pure projector
in prospective encrypted store context. Existing `lab-transport --media-route`
uses it after accepted login. Preparation freezes exact request bytes/digest,
actual generation, request time and a full copy of the coordinator's applied prior
route in one transaction. A known prior route ID becomes the conditional last_id
attribute traced in pinned 0VM.A07. Legacy route history never seeds this owner
retrospectively; the first prospective request has an explicitly absent prior.

A correlated result is validated and projected against that frozen copy. First
route false replies now install incoming hosts; non-new updates retain exact prior
host nodes with incoming auth/lifetimes/ID. Raw reply and derived effective reply
are stored separately. The applied current pointer and request settlement commit
atomically with CLI-saved unrelated inbound frames. If current state changed after
preparation, the reply is saved as stale and cannot overwrite it. Exact duplicates
are no-ops; conflicting replies refuse. Outer errors and unsupported results retain
raw evidence without replacing the applied route. Store/authentication faults
propagate; serialization diagnostics are static and preserve the storage error.

Upload selection follows the applied pointer, not response timestamps or random
request-ID order. It recomputes the projection from sealed prospective context
and verifies the stored derived result. A failed later refresh does not erase
the current object, matching the traced locked A0P getter/setter boundary; expiry
checks still apply before upload. Once this owner exists, missing current/orphaned
records cannot fall back to unrelated legacy history. Missing owner/source records
refuse new preparation rather than manufacturing prior authority.

Six real-store wrapper tests and one upload-selector test cover reopen, first
false reply, conditional last_id, exact old-host/new-auth merge, original-wire
retention, duplicate/conflicting settlement, stale CAS, transaction cuts, missing
source/owner and generation/digest failures. No live traffic occurred.

**Still incomplete:** native volatile flight/throttle/32-second timeout integration,
property149-dependent startup cache restore policy, full host/upload/download/bucket
selection, account logout teardown and upload select/reserve concurrency rechecks.
The durable current object is an explicit local continuity policy, not proof that
every native startup would restore it. Preparation alone neither sends nor replays
a request; the existing explicit CLI operation sends one. No automatic refresh,
key installation, identity reset, guessed historical request time or ban guarantee.

## Explicit media throttle (2026-11-04)

**Confirmed pinned static/implemented offline:** 1xY.Bvj consumes the first
valid nonzero error-code/backoff integer pair. Missing code defaults to zero;
malformed entries are skipped. A positive code507 backoff is seconds, capped
at 10800. Code503 clears this throttle and separately invokes native Fibonacci
retry logic. A supported successful routing result also clears the throttle.
[REA method, checked branches/constants and arithmetic limits](rea/2.26.43.72-media-route-lifetimes/README.md#explicit-media-refusal-throttle).

The prospective wrapper now stores a generation-authenticated deadline atomically
with the original refusal and its receipt timestamp. Preparation refuses while
now <= deadline and creates no new request record. Expired observations remain
inspectable without scheduling a query. Exact duplicate receipts do not extend
the wait. Existing corrupt/auth-failing state propagates instead of being cleared
as absence. Server-text strings are not printed; the CLI refusal reports only the
numeric deadline. `throttle_until` is a read-only library inspection.

Two tests cover cap/receipt anchoring, reopen, no-write preparation refusal,
duplicate receipt stability, exact expiry boundary, Java BMP integers, malformed
entry skipping, first-pair selection and 503/success resets without new requests.
Native clock service parity, volatile lifetime and the separate Fibonacci retry
owner remain unfinished. Durable persistence is a local continuity policy; no
other error code invents a delay and no automatic retry was introduced. No live
account operation occurred.

**Open migration boundary:** earlier wrapper versions could retain raw507
refusals without a dedicated throttle observation. This change does not replay or
reconcile those historical callbacks. A missing throttle checkpoint is therefore
not proof that every historical wait was processed. Durable receipt ordering and
bounded legacy backoff reconciliation need a separate audit before complete
startup throttle recovery can be claimed. Historical request/reply evidence is
preserved; do not infer native retry eligibility from deadline inspection alone.

## Durable refresh receipt order (2026-11-04)

**Confirmed Rust/offline:** every newly settled prospective refresh now receives
a positive local settlement_sequence in the same transaction as raw/effective
reply, current projection and throttle effects. This orders refused, unsupported,
stale and applied receipts together; wall-clock timestamps and request IDs are
not receipt order. Exact duplicates retain their original sequence. The local
counter cannot restart if sequenced records exist, accept zero, or overflow.
Preparation checks counter capacity/integrity before reserving another request.
A transaction cut rolls back the counter and every associated effect.

Missing sequence on older records remains explicitly unknown. Duplicate receipt
handling does not stamp them retroactively. `settlement_sequence(store,id)` is
a safe numeric-only inspection; the existing CLI reports receipt_order alongside
its applied-route counts or fixed local outcome. No auth, host, server-text or
raw wire is printed. The sequence is private local metadata, never a wire field.

Three real-store tests cover order across mixed outcomes with nonchronological
IDs, duplicate/reopen stability, missing/zero/exhausted counters, transaction
rollback and unchanged legacy records. This is the ordering prerequisite for
recovery; historical throttle reconciliation itself is still unimplemented. No
query, replay, clock guess or live account operation is introduced.

## Bounded historical wait recovery (2026-11-04)

**Confirmed Rust/offline:** `recover_throttle` validates prospective saved
receipts, request/reply correlation, generation and a complete contiguous local
sequence against the persisted counter before deriving any throttle effect. It
uses receipt order rather than request IDs or wall-clock order. A missing507
checkpoint can be restored from the original receipt timestamp plus its bounded
backoff; it is never restarted from now. Preparation calls this local recovery
before its separate reservation transaction, then rechecks the stored deadline.
The repair therefore survives a refusal to reserve a still-throttled query.

Recovery never erases or shortens an existing saved deadline whose origin is not
proved by a dedicated processing marker. Fresh qualified live reset transitions
still work as before. Unsequenced prospective507 receipts with an active original
bound are reported separately and prevent preparation; no reset order is guessed
and no historical sequence is stamped. Once those original bounds expire they
cease to block, without issuing requests or altering their evidence. Older unbound
route rows are counted and remain outside this recovery contract. A missing
checkpoint is not proof that such unbound history has been reconciled.

The safe RecoveryReport exposes sequenced/unsequenced/unbound counts, the active
unsequenced bound and preserved deadline. It contains no raw identifiers, hosts,
auth or server text. Incomplete/duplicate sequences, invalid source metadata,
authentication failures and the local 10000-route-history bound refuse before
partial repair. Four encrypted-store tests cover lost-wait/reopen restoration,
clock/ID ordering, preservation of ambiguous saved deadlines, unsequenced expiry
without restamping, and corrupt/incomplete history. No live traffic occurred.

**Still open:** unbound legacy refusal analysis, an authenticated last-processed
throttle marker that can prove recovery-time resets, full native volatile/Fibonacci
retry coordination and clock-service parity. This conservative repair is local
continuity, not a claim of complete native startup or automatic retry eligibility.

## Legacy refusal observations and offline command (2026-11-04)

**Confirmed Rust producer provenance:** `git show
65d41e7:crates/whatsapp-cli/src/transport.rs` retains the old producer: reserve
media-route:<id>, await an IQ with that ID, then store full original response and
receipt-time at under result/refused. Those records have no prospective request
context or receipt order. They cannot become prior-route authority.

**Implemented local recovery policy:** known legacy refused captures now require
a matching IQ/key ID and readable original wire. Only an outer error can supply a
positive507 observation, interpreted with pinned 2.26.43.72 code/backoff rules and
anchored to a positive original receipt timestamp. Its possible active bound is
reported as unbound_active_until and blocks new preparation through that bound.
It is not written as a proven ordered throttle or used to rebuild a request.
Missing/invalid timestamp, corrupt refusal wire, foreign ID or unknown status
refuses explicitly. Ordinary results/uncertain captures do not fabricate waits.
Legacy503 never clears another wait because relative order remains unknown.

The historical capture's APK/build/clock-service provenance remains unknown;
this is a guarded interpretation under the pinned reference, not cross-release
parser/expiry parity or proof that a reset never occurred. Expiry uses the
original bound and does not restart from now. Existing raw records are unchanged.
Two additional real-store tests cover the legacy bound, no context/owner creation,
no restamping, expiry, reset ambiguity, correlation/time/status faults and an IQ
result containing an unconsumed error child.

`whatsapp-cli media-route-recover --json` exposes the safe report offline using
an existing store. It may restore known local deadlines but performs no login,
query or retry and never creates a missing store. It reports only counts and
numeric deadlines. This closes the known legacy producer's refusal-observation
gap; complete native retry/clock/startup ownership and reset-proof processing
markers remain unfinished. No live account operation occurred.

## Native retry/timer owner trace (2026-11-04)

**Confirmed static/derived:** checked 0Vc/0Vf state shows seed1/cap987 and base
0,1,1,2,...,987 seconds. The first advancement after fresh/reset state requests zero delay without drawing
Random; positive bases use the retained signed-remainder jitter expression, with
derived millisecond bounds [750*s,1250*s-1]. The base cap is not a final-delay cap.
1VI selector2/what0 dispatches back to the gated A07 request path. With a current route, A06 replaces
message0 with an auth-deadline-minus60s timer; successful route installation/merge
resets the Fibonacci state before that timer setup.
[Full checked files, hashes, model and limitations](rea/2.26.43.72-media-retry-scheduling/README.md).

The xtask decoder now recognizes cb/add-double/2addr and be/div-long/2addr,
closing the previously noted arithmetic-symbolization gap. REA's configured Java
executable was unavailable for new method decompilation in this pass; the new
evidence is explicitly checked DEX, not a fabricated REA record. Session/DI and
disconnect cancellation, other timer producers and full automatic retry ownership
remain untraced. Runtime scheduling behavior is unchanged; no queries were sent.
