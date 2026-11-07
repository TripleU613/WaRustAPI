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

**Historical Rust support gap/guarded offline:** cached route selection previously
ignored is_new and selected reply hosts regardless of prior-route semantics.
It now parses the flag with the Java-compatible integer helper/default and
explicitly refuses non-new replies because no frozen prior-host context exists.
At that stage this included the native first-route false branch. The subsequent
prospective wrapper supports that branch; legacy unbound standalone replies
remain refused. This is not a claim that native refuses such replies. Stored wire/checkpoints
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

### Native disconnect cancellation and demand triggers (2026-11-04)

**Confirmed static**, pinned2.26.43.72: 1iM.A02 clears the manager flag and
removes media Handler message0 during its disconnect handling. 0VM.A0T uses
a separate two-minute connection/auth demand threshold; 0VM.A0S routes401/403
to the existing gated request function. These do not authorize HTTP resend.
[Exact output, scope, hashes and remaining caller gaps](rea/2.26.43.72-media-retry-scheduling/README.md#disconnect-cancellation-and-additional-triggers-2026-11-04).

### Conditional HTTP auth refresh (2026-11-04)

**Confirmed static**, pinned2.26.43.72: all three direct0VM.A0S callers
are retained. Mtv.BkV gates auth-change retry eligibility on401/403, route
state, feature predicate, count<2 and changed auth; it accounts elapsed
wait against a per-context budget. Mtv.Bvd suppresses repeated refresh
reporting after auth replacement fails. MVv.A04 retains download response
codes while reporting them to the route manager.
[Full conditions, hashes and untraced flag producers/loops](rea/2.26.43.72-media-http-auth-refresh/README.md).
No generic HTTP retry is implemented or justified.

### Auth-refresh transfer-loop edge (2026-11-04)

**Confirmed static**, pinned2.26.43.72: LIY forwards auth failure to Mtv;
MS4.A03 invokes Nd8.BkV via the interface. A true result enters supplier
delay/sleep, reloads route and returns through cancellation/readiness checks
before another NYs.Cb6 invocation. Sleep interruption uses terminal handling.
Operation-specific replay/idempotency, AXk/CW7 policies and enable-argument
provenance remain open. [Exact branches and hashes](rea/2.26.43.72-media-http-auth-refresh/README.md#transfer-loop-continuation-2026-11-04).

### Pre-attempt auth readiness and supplier delays (2026-11-04)

**Confirmed static**, pinned2.26.43.72: CW7 shares the bounded auth wait
budget, refreshes almost-expired auth, and conditionally adopts a changed
token. Mtv's exponential AXk differs from LIY's inherited fixed AXk;
MvM.A00 selects the wrapper on one construction branch.
[Branches, exact bytecode and limits](rea/2.26.43.72-media-http-auth-refresh/README.md#readiness-and-distinct-delay-suppliers-2026-11-04).
No generic delay policy or automatic Rust retry is established.

### Transfer result flags and cancellation entry points (2026-11-04)

**Confirmed static**, pinned2.26.43.72: Ly8 constructor/factory flags and
MS4 ordering distinguish immediate recheck, supplier switch, auth recovery
and terminal handling. MS4.A02 supplies a false cancellation predicate;
MvO.ARB supplies a different predicate with MtZ upload callback.
[Matrix, HTTP leaf truth table, source hashes and limits](rea/2.26.43.72-media-transfer-results/README.md).
No generic retry rule is inferred from HTTP ranges.

The upload entry's two unnamed bytecode instructions are now decoded with
operand/boundary/truncation regressions; [regenerated evidence](rea/2.26.43.72-media-transfer-results/README.md#checked-upload-mnemonic-correction-2026-11-04)
closes that tooling limitation without claiming full transfer parity.

### Pinned upload HTTP construction boundary (2026-11-04)

**Confirmed static**, pinned2.26.43.72:15Q selects JAS/JAT transports;
K2k appends route auth before virtual execution. JAS uses identity, configured
UA, conditional analytics/companion/map headers and conditional range/raw/
multipart behavior. Annotation names are not automatically HTTP headers.
[Retained concrete implementations and unresolved input producers](rea/2.26.43.72-media-upload-http/README.md).
No mandatory-header mismatch for the supported Rust upload is proven yet.

### Route-bound raw body and stream offsets (2026-11-04)

**Confirmed static**, pinned2.26.43.72: K2k.A06 uses a raw first-stream
branch for nonnull route plus registered file; other inputs use multipart.
A08/JiH preserve offset/length; A01 skips positive offset, copies chunks
and handles optional end processing. This supports raw upload shape but
does not establish Rust parity for optional trailers or resume negotiation.
[Exact outputs and limits](rea/2.26.43.72-media-upload-http/README.md#raw-stream-versus-multipart-body-writer-2026-11-04).

### Optional EOF sealing boundary (2026-11-04)

**Confirmed static**, pinned2.26.43.72: raw single-file helper is count==1.
Optional Kif processing is gated by MvM.A06 and AL2 factory selection;
null Kif emits no EOF bytes in the copy stage. MtQ has specialised metadata
seal branches and distinct abort/error hooks.
[Exact processors, bounded implementation scan and unresolved operation binding](rea/2.26.43.72-media-upload-http/README.md#optional-eof-processor-and-single-file-predicate-2026-11-04).
No universal seal requirement or Rust body change is established.

### Voice-note seal scope needs construction binding (2026-11-04)

**Confirmed static**, pinned2.26.43.72: default LKZ uses Mv9 processing,
except newsletter-ptt; custom Nbs/Ncy can override. Seal byte order is
little-endian and charset UTF-8. Rust voice upload has no separate seal,
so this is a concrete potential omission pending finished-file constructor,
override and final hash-string provenance.
[Owner/encoding/hash evidence](rea/2.26.43.72-media-upload-http/README.md#seal-owner-excluded-kind-and-encoding-helpers-2026-11-04).
No guessed seal is added and full voice upload parity is not claimed.

### Finished voice constructor and seal hash formatting (2026-11-04)

**Confirmed static**, pinned2.26.43.72: plugin A09 has a branch setting
LKZ sent/finished true. Seal hash extraction maps first Mzq digest stream
to MAw.A00 and formats via Android Base64 flag2; it is not inferred from
the URL-safe path hash. Exact digest bytes and operation/override binding
remain open. [Retained branches and writer census](rea/2.26.43.72-media-upload-http/README.md#finished-voice-construction-and-digest-formatting-2026-11-04).

### Voice seal digest stream composition (2026-11-04)

**Confirmed static**, pinned2.26.43.72 standard non-chunk Mtd path:
outer SHA-256 covers encrypted stream including10-byte MAC; metadata
seal is appended afterward outside those filters. Mzq constructor field
order is retained explicitly to distinguish plaintext/transformed digests.
[Stream graph and branch limits](rea/2.26.43.72-media-upload-http/README.md#digest-stream-includes-cipher-mac-excludes-transport-seal-2026-11-04).
Higher-level path/processor binding remains open; no runtime body change.

### Standard finished voice seal binding (2026-11-04)

**Confirmed static**, pinned2.26.43.72: MFE creates standard MOu with
Lg0-selected Mvc for LKZ; Mvc implements Nbs without Ncy override, so
MOu selects the default voice processor. Finished non-newsletter branch
therefore appends a sent/hash seal. Rust currently lacks this equivalent;
exact serialization and historical reservation compatibility remain to fix.
[Factory/interface evidence and implementation boundary](rea/2.26.43.72-media-upload-http/README.md#standard-finished-voice-processor-binding-2026-11-04).

### Finished voice seal correction (2026-11-04, offline)

**Confirmed source mismatch fixed:** new one-shot primary voice upload body
now appends the pinned finished sent/hash seal. Cipher+MAC hash excludes
seal; other media bytes remain unchanged. Exact wire body/version/size is
retained in new private reservations. Existing acknowledged or uncertain
reservations are not rewritten/replayed.
[Encoding, native oversize behavior, tests and limits](rea/2.26.43.72-media-upload-http/README.md#finished-voice-seal-integration-2026-11-04-offline-validated).
No live upload; streaming/newsletter/unfinished processing remains outside
the existing one-shot operation.

### Sealed voice checkpoint reopen regression (2026-11-04)

**Confirmed offline test:** a sealed voice wire reservation retains body,
cipher hash, cipher length and framing version across HTTP200 header receipt
and encrypted-store reopen. It remains non-replayable without a complete
payload; repeated header replacement fails. A completed unversioned legacy
row still resolves its saved protobuf without changing the stored row or
backfilling a seal. No live request is made by this regression.
Test: image_upload::tests::sealed_voice_reservation_survives_reopen_without_replay_or_legacy_rewrite.
No new runtime behavior or native claim is introduced.

### Upload reply mode boundary (2026-11-04)

**Confirmed static**, pinned2.26.43.72: known MtZ callback MtR parses
reply into mode-specific LzN fields. direct_path UUID-v4 retention is
conditional on MKA.A01; unmatched originals use a separate field. It is
not a slash-path validation regex. Optional field reads alone do not prove
required fields or success. Rust's HTTP200+URL/slash-path acceptance remains
a local subset pending mode producers/downstream consumers.
[Exact callback/pattern and scope](rea/2.26.43.72-media-upload-response/README.md).

### Scheduled cross-reference versus ordinary path (2026-11-04)

**Confirmed static**, pinned2.26.43.72: descriptor flags map to metadata
presence and inverse MvM.A06. MOu.A07 adopts UUID handle through KmQ's
scheduled-cross-reference verifier, separately clearing ordinary path.
Whole-string regex matching is confirmed; ordinary path instead goes
through M0g normalization. [Mode/consumer evidence and remaining inputs](rea/2.26.43.72-media-upload-response/README.md#mode-producers-and-scheduled-cross-reference-consumer-2026-11-04).
No acceptance change or live request.

### Ordinary path signature/expiry parameter presence (2026-11-04)

**Confirmed source mismatch corrected offline:** new upload reply paths now
require oe and oh query keys as M0g.A00 does. Empty values count, encoded
key aliases/fragment-only parameters do not; accepted bytes are not rewritten.
This is presence, not signature authentication or expiry-time verification.
Original complete response is stored before refusal, and no replay is enabled.
[Native helper, platform semantics and regressions](rea/2.26.43.72-media-upload-response/README.md#ordinary-path-query-presence-correction-2026-11-04-offline).

## Media-refresh admission regression (2026-11-04)

**Confirmed existing behavior / newly tested offline:** invalid reply admission
preserves the old usable route and complete pending refresh across reopen.
A later valid result applies once; its identical duplicate is idempotent.
[Source boundary, pinned evidence and limits](network-correctness-audit.md#media-refresh-settlement-admission-review-2026-11-04).

### Malformed refresh replies preserve route and wait together (2026-11-05)

**Confirmed local / regression added:** the prospective media coordinator
already isolates unsupported result bodies while retaining its current route
and server wait. The new combined test installs a real local route, reserves
a refresh against it, then records a separate 507/60-second wait before the
reserved result arrives. Six invalid result cases cover missing/duplicate
media_conn, missing auth, malformed ttl, overflowing auth_ttl and overflowing
is_new. Each settles Unsupported without changing the installed CURRENT bytes
or throttle deadline; the raw reply remains retained.

The test unsupported_results_preserve_installed_route_and_server_wait_across_reopen
also verifies exact duplicates do not change any checkpoint or receipt counter,
a conflicting late valid reply is refused without mutation, reopening preserves
all rows, and a further prepare remains throttled without reserving anything.
The valid result control still replaces routing and clears the saved wait, so
this is not a blanket refusal to adopt fresh routing. No production behavior
was changed by this audit.

**Confirmed static evidence retained:** APK43.72 1xY.CK4 requires media_conn,
auth and long lifetimes; 0VM.A0A selects new/prior hosts; Bvj selects throttle
directives. See [media lifetime evidence](rea/2.26.43.72-media-route-lifetimes/README.md).
The Rust conn selector additionally rejects duplicate children as a local
admission policy. Stale-prior, outer result/error/ID admission and orphaned-owner
checks remain separate existing tests; this new test does not stand in for them.

**Bounded negative findings / limits:** has_context_rows refuses a 10,000-row
checkpoint scan rather than silently treating truncated history as absent.
Throttle recovery pages checkpoints but separately caps media records at 10,000.
Those local resource limits can refuse large histories; they were not removed
or claimed as native limits. Full upload/download HTTP interoperability, every
host choice, native lifecycle parity and live acceptance remain outside this
coordinator audit. No new refresh, retry, wait reset or live account operation.

## Download result reservation ownership (2026-11-06)

**Confirmed local gap / fixed offline:** ordinary media GET success wrote its
authenticated ciphertext checkpoint unconditionally after network awaits. Refusal
checked only outcome_unknown. A late result was not bound to the download attempt
that initiated it.

New downloads persist outcome_unknown with a fresh local reservation ID. Both
success and definite HTTP refusal compare the exact expected record inside their
settlement transaction before replacing it. Missing, changed or already-settled
state refuses without returning the newly decrypted media or overwriting the
current checkpoint. Existing cached completed media, old uncertain no-retry
records, preflight/hash checks and final availability checks retain their paths.
The local ID is not sent on the wire and does not authorize retry.

Regression late_download_result_cannot_replace_changed_or_settled_reservation_after_reopen
covers missing, differently owned unknown, consumed/refused/downloaded records,
unchanged checkpoints/journal and refusal after reopen. Exact success progresses
once; duplicate success or later refusal cannot replace it. Existing numeric HTTP
refusal tests still cover status retention and no-replay behavior.

**Bounded negative client audit:** image_upload.client, shared by ordinary media
GET and POST, already selects https_only, redirect Policy::none, retry::never,
no_proxy, referer(false), disabled transparent compression and explicit total/
connect timeouts. No mismatch was found in that local one-request configuration;
it is not evidence of native timeout equivalence or server policy. No network
capture/live operation was performed.

**Limits:** transaction checks do not lock HTTP and storage together. This change
does not add independent capture of every stale/partial GET body, alter view-once
consumption or solve all descriptor/session-generation races. No deletion,
identity rotation or automatic repeat request is introduced.

## Shared upload preflight precedes cache and reservation (2026-11-06)

**Confirmed local omission / fixed offline:** upload_source checked config and
registered-account/phone matching, but public prepare/prepare_outbox could reach
upload_prepared without those checks. That common function returned cached success
before config/account validation and could otherwise reach route selection,
encryption and saved upload-token randomness with invalid account context.

upload_prepared now validates config and requires a registered account with the
same phone before cache lookup or any route/randomness/reservation work. Existing
source-entry checks remain as an earlier boundary. Valid matching accounts can
still reuse cached upload payloads; no checkpoint is repaired or reset. This is
a local account/config integrity rule, not a recovered APK registration predicate.

Async offline regression upload_preflight_precedes_cached_success_and_preserves_state_on_refusal
runs both public entry points against a pre-existing cached success: absent,
unregistered, phone-mismatched and invalid-zero-timeout contexts refuse without
checkpoint/journal changes, while a valid registered matching account returns the
cache unchanged. The older group-cache binding fixture now installs a valid
account/config, preserving its recipient and explicit-group binding assertions.
No route or HTTP server exists in these fixtures; no network operation ran.

**Limits:** bounded local file reading can precede this common upload entry;
no-media input still returns None without creating an upload. This preflight does
not hold account state across later HTTP awaits or bind every account field to
upload settlement. The separate response reservation checks remain required.
No new retry, identity mutation or live operation.

## Refresh authority and lifetime audit (2026-11-06)

**Confirmed Rust/offline:** the existing prospective refresh path binds settlement
to its request generation, IQ identity, owner and frozen prior route. A competing
installed route makes the old reply stale instead of replacing newer auth. Exact
duplicate receipts do not extend the stored throttle. Upload route selection
checks both route TTL and auth TTL from the retained request time.

The first prospective response accepts `is_new=0` with no prior route and uses
incoming hosts; subsequent non-new responses retain prior hosts with incoming
metadata. The older unsupported-first-route wording above described the earlier
unbound cache path, not the current prospective implementation. The retained
[0VM.A0A evidence](rea/2.26.43.72-media-route-lifetimes/README.md#conditional-mergecurrent-route-follow-up)
already establishes the native first-route branch; no new APK trace is claimed.

Validation reran all 19 refresh/recovery tests and the pure first/new projection
regression. No runtime correction was found in this audit. **Unknown/unported:**
complete sender admission, native elapsed-clock/reboot parity, automatic native
refresh scheduling and complete host policy. The conservative local wall-clock
boundary remains; these checks do not prove server acceptance or prevent bans.
No live operations occurred.

## Media refresh reply window (2026-11-06)

**Confirmed pinned static / implemented offline:**0VM.A07 passes32000ms to
0Sf.A0R for media IQ124. The explicit CLI media-route operation previously used
the generic30s captured wait. It now calls refresh::write_and_wait_captured,
which uses public REPLY_TIMEOUT and messaging::write_and_wait_captured_until.
The absolute deadline starts before write and never restarts afterward. An
already expired supplied deadline refuses before any request record is written.
Existing generic callers keep30s. Awaited writes may delay observing expiry;
independent socket deadlines still apply. **A configured30s I/O cap still wins
over the32s callback window.** No configuration is silently lengthened.

[Exact registration source, shared timer chain and sender-admission negative](apk/2.26.43.72-media-reply-window/README.md).
Native expiry depends on Handler ownership/scheduling; Rust's absolute32s cut
is a local bounded policy, not a claim that Android universally rejects every
reply at that age. Native suspend/reboot/queued callback parity remains open.
No new sender rule was established, so no whitelist was invented.

The paused-clock certificate-verified in-memory fixture first reproduced a
31s reply failing at the generic correlated30s deadline with a60s I/O budget.
It now applies that reply and retains the route after reopen. At32s the local
callback deadline refuses settlement; a31s reply with30s I/O also refuses.
Both refusal cases preserve the original uncertain reservation byte-for-byte
through reopen, and preparing the same ID refuses rather than resending.
An expired-before-write control makes the peer observe EOF without a request
record. Existing prewrite-deadline controls cover no timer restart after write.
These are synthetic encrypted exchanges, not live WhatsApp account traffic.
No durable reset, throttle bypass, upload, automatic query or replay is added.

## Media header producer scope 2026-11-06

**Confirmed selected static:** pinned43.72 MvM.A02 merges provider defaults
before descriptor headers; descriptor entries override matching keys.845.A00
aggregates its injected providers exclusively under newsletter, so these defaults
are not established as ordinary image/PTT requirements. Standard voice-path
Mtt.A05 copies the existing descriptor map even when updating its kind; null-map
MetaAI branches do not prove standard uploads omit extra headers.
[Complete checked DEX, two complete REA records and reproduction](apk/2.26.43.72-media-header-producers/README.md).

**Unknown:** original standard descriptor's map producer, effective newsletter
providers, all constructor paths and JAT platform defaults. No demonstrated
missing universal header, Rust behavior change or live acceptance claim. Next:
trace the original MJe.A02 descriptor upstream without assuming it is empty.

## Original media extra headers traced 2026-11-06

**Confirmed selected static:** ordinary-message ACp.A01 passes a null header map
to M5T, then wraps it in MJe. The other selected MJe factories use MFB.A00 with
null maps; MFB.A01 also supplies null. M5T.A00 and Mtt.A05 copies preserve the
map. With the earlier newsletter-only defaults, this establishes no missing
extra header on the traced ordinary upload descriptor chain. Separate M5I.A00
builds backup authorization/metadata headers under kind backup; do not add them
to ordinary uploads. [Full origin censuses, methods, argument binding and REA](apk/2.26.43.72-media-header-origins/README.md).

**Unknown:** native default headers, effective transport and complete body-mode
binding, newsletter callbacks, backup subsystem and server mandatory checks.
No runtime behavior changed, new sender or live validation. Next trace is the
one-shot fixed-length/body-mode selector, which extra-map absence cannot prove.

## Raw body and fixed-length gates are separate 2026-11-06

**Confirmed selected static:** factory15Q snapshots00E.A14(0x666) intoA09,
then supplies JAS.A0A/JAT.A0G, both controlling their positive-length,
nonpositive-offset fixed-length setup. Route-bound single-stream raw-body
selection remains separate. A disabled fixed-length gate does not select
multipart: exact DEX branches bypass its Content-Type block. JAS uses chunked
streaming on that raw path; JAT carries offset/length onward without explicitly
adding Content-Length in this branch. Absence of an explicit header is not
proof of native-stack framing. [Complete constructors, branch table, REA and
reproduction](apk/2.26.43.72-media-body-mode/README.md).

Rust's known byte-body POST matches a supported raw shape. Effective0x666 and
native provider length/framing remain unknown, so complete transfer-mode parity
is unproven. No guessed multipart/range, Rust behavior change or live operation.
Next: downstream JAT provider and effective configuration/stream lengths.

## Native upload length handoff 2026-11-06

**Confirmed selected static:** JAT's IxI provider reports min(declared length,
INT_MAX), then separately invokes the body callback; selected gzip/worker flags
are false. KXB selector1 calls the existing K2k writer, including optional EOF
processing. JOT forwards writes and treats native return1 as cancellation, not a
partial-byte count. Xplat wrapper forwards declared length to native code.
Exact relocated JNI tuple maps reportBodyLengthNative to64fe3c in the verified
reconstructed ARM64 libwhatsappmerged; it sign-extends the integer and invokes
vtable slot0x30. Concrete target and wire framing remain unknown.
[Complete DEX/REA, tuple resolver, hashed assembly and reproduction](apk/2.26.43.72-media-body-provider/README.md).

REA native search was unavailable; full diagnostic retained separately from
successful Android Evidence. ELF/LLVM inspection continued without executing the
library. No Rust behavior change: declared length is not proof of body truncation,
server acceptance or authority to replay. Next resolve native slot0x30/length
handling and effective0x666; no live operation.

## Native length-handler candidates resolved 2026-11-06

**Confirmed selected static:** RTTI/relocations identify primary slot0x30 entries
for TigonLimitingBodyStream, mns::TigonMNSBodyStream and TigonHucBodyStream in the
verified reconstructed ARM64 library. Limiting entry forwards length unchanged
to its wrapped stream. MNS normal nonnull-owner path stores the supplied length
atowner+0x320 under mutex+0x290. This resolves conditional implementations of the
previous JNI boundary, not actual stream selection or final wire length.
[Resolver with executable-pointer checks, hashed assembly and exclusions](apk/2.26.43.72-media-native-length-handlers/README.md).

HUC downstream target, limiting-wrapper gate, dynamic object binding and MNS
stored-field consumers remain unknown. A class named Limiting does not itself
prove length truncation. No Rust behavior change, replay or live traffic. Next:
trace MNS+0x320 consumers or prove the actual stream implementation.

## Stored length reaches a count-and-total callback 2026-11-06

**Confirmed selected static:** candidate routine51aa80 incrementsowner+0x318,
loadsowner+0x320 under mutex+0x290, and captures the two values for a downstream
callback. Its closure passes count and declared length unchanged through the
callback object's slot0x30. This is a different interface from the stream's
length-report slot. No count/length comparison or HTTP header is constructed in
this selected routine. **Inferred:** upload-progress reporting, supported by the
pair/context and literals; exact callback target is not established.
[Complete selected method, helper/registration windows and search limits](apk/2.26.43.72-media-length-consumer/README.md).

The207 exact-offset textual matches include unrelated/stack objects and do not
constitute a field-alias census. Other consumers, effective configuration and
wire framing remain unproven. No Rust behavior change, truncation, replay or live
operation. This consumer does not establish length enforcement.

## Native queue bound and disable-header namespace 2026-11-06

**Confirmed selected static:** native5432b8 rejects cumulative queued bytes above
expectedBytes except expected-1, rolls back this increment and creates local
body-size-exceeds-content-length error. This is separate from the previously
traced progress total; expected-size producer and actual upload binding remain
unknown. A local failure does not prove earlier bytes stayed local or authorize
replay. [Complete branch, literals, callers and limitations](apk/2.26.43.72-media-queue-length-check/README.md).

The getDisableContentLengthHeader getter resolves toPandoTigonConfig, not the
inspected TigonHttpClientConfig. Its existence is not an ordinary-upload header
setting. No Rust body/header change or live operation. Next: queue expectedBytes
producer and its relationship to actual headers/provider length.

## Queue expected size is initialized from Content-Length 2026-11-06

**Confirmed selected static:** initializer53b9c4 reads request header key788e58
(relocated literal content-length), converts through642378, and stores expected
size with STP WZR,W26 at+0x70, whose second word is+0x74. It initializes queued
count+0x78 to0. Selected queue construction admits positive lengths or-1; absent
lookup becomes0 and skips this allocation. Full predicates/other branches remain
untraced. [Paired-store binding, exact key relocation, conversion and exclusions](apk/2.26.43.72-media-expected-length-source/README.md).

This closes the immediate queue expected-value source, separate from progress
total or inferred JNI length. Rust's known body includes the voice seal; preserve
its actual length rather than substituting hash-input length. No Rust behavior
change, live operation or replay. Native synthesis of absent explicit JAT length
and full ordinary-upload transport binding remain open.

## Java upload header submission boundary 2026-11-06

**Confirmed selected static:** JAT's request/provider pass through0eA.A01,
Kag selector2 andTigonHttpClientBase toTigonXplatService. Request serializer1Fb.A01
passes existing headers to1Fa.A05, which writes size/key/value strings; these
selected serializers have no body-provider input or added length entry. Exact
physical provider-length getter caller census finds onlyIxI's length report.
[Complete DEX chain, census and native candidate exclusions](apk/2.26.43.72-media-java-native-submit/README.md).

A native internal content-length=-1 write sits near sec-websocket-protocol setup;
no ordinary-upload binding or final wire-header claim established. Native
middleware/property effects and absent-explicit-header synthesis remain open.
No Rust behavior/header change, replay or live operation.

## Selected JNI upload provider adapter connected 2026-11-06

**Confirmed selected static:** JNI tuple identifies650ff0 as the selected provider
submit entry; matching initHybrid factory64f83c createsTigonXplatBodyProvider.
Class-to-table registration remains untraced, so its inherited-factory binding
is conditional.
Its slot0x10 adapter64fa04 createsTigonJavaBodyProvider, whose slot0x10 is the
previously traced64e908 stream start. Nonzero adapter+0x70 selects the limiting
wrapper; effective service input remains unknown. [Exact tuples, RTTI/primary
vtable checks, factories and full selected submit entry](apk/2.26.43.72-media-native-submit-provider/README.md).

Request buffer is decoded separately by651860; retained prefix reads method/URL
and header pairs. Remaining conversion/native middleware and automatic headers
are unproven. No Rust behavior change, acceptance, retry or live operation.

## Upload JNI class-to-table registration closed statically 2026-11-06

**Confirmed selected static:** complete registration routine64cf40 names
TigonXplatBodyProvider and binds its initHybrid table755fd0 to factory64f83c.
It also binds stream/token/service tables, including submit650ff0. Dispatcher
calls JNI slot0x6b8; independent installed JNI.h identifies RegisterNatives
index215. [Exact class slices, tuples, complete registration and ABI check](apk/2.26.43.72-media-jni-registration/README.md).

This closes the prior static registration uncertainty in the provider chain.
Actual loading/invocation/success remain unobserved. Effective wrapper gate,
remaining native request conversion and header synthesis remain open. No Rust
behavior change, registration execution, account mutation or live traffic.

## Native header transfer and timeout normalization 2026-11-06

**Confirmed selected static:** native request conversion moves the decoded
header container into request+0x130; exported addHeader independently confirms
that member. Four serialized millisecond fields pass through validateTimeout:
zero and values>=1000 remain; other signed values report a soft error and become
1000 on normal return. Native request/idle setters confirm two target offsets.
[Exact field table, complete helper and selected conversion windows](apk/2.26.43.72-media-request-conversion/README.md).

This is separate from Rust's configured total/connect bounds and callback
windows; no silent extension of shorter local bounds is introduced. Later native
properties/middleware, automatic headers and wire framing remain unproven.
No Rust behavior, reservation, account or live operation changed.

## New media upload User-Agent preflight (2026-11-06)

**Confirmed static APK behavior:** the app's selected Tigon factory installs
WAUserAgentInterceptor. Its native invocation fills an absent User-Agent for
HTTPS when the selected configured string is nonempty. Java inputs, encoded
provider dispatch, native registration, factory/vtables, complete invocation and
helper checks are retained in [the evidence bundle](apk/2.26.43.72-media-user-agent/README.md)
and [complete REA holder evidence](rea/2.26.43.72-media-user-agent/README.md).
This closes the previously open native automatic User-Agent branch for this
factory, not all middleware or every runtime client/configuration.

**Confirmed Rust gap/fixed offline:** client.user_agent defaulted to None;
new comparison media uploads could therefore proceed without the APK's header
fill. `image_upload::upload_prepared` now refuses missing client.user_agent
before route lookup, encryption/entropy checkpoint allocation, reservation or
network transmission. The shared HTTP builder already applies a configured
value; validation rejects empty/invalid values. This is a deliberate local
compatibility preflight, not a claim that WhatsApp requires the header. No UA
or device identity is synthesized, and the shared download client is unchanged.
The guard covers new image/voice/document/video uploads through this path.

Existing uploaded outcomes remain reusable; saved refusal and uncertain states
retain their existing errors and cannot be replayed. The regression exercises
all four states, encrypted-store reopen and byte-identical journal/checkpoint
rows, plus a configured-UA case that reaches the independent route check.
It failed before the guard and passed afterwards; 1,509 workspace tests and
strict all-target/all-feature Clippy passed. No live account operations.

**Unknown:** category meanings (native request values 3/5 choose second string),
full primary/secondary UA formatting, full lazy client selection and effective
runtime activation, server acceptance/requirements and account restriction
causes. Static compatibility work cannot establish zero bans. A zero physical
constructor census was not absence proof: optimized allocation uses Object.init
and explicit initHybrid, corroborated by checked DEX and complete REA output.

## Native HTTP priority context separated from headers (2026-11-06)

**Confirmed static:** the next interceptor installed by RlW attaches a shared
WAHttpPriorityContext when the request lacks one; an existing context is
preserved. Native registration, factory/vtable linkage and complete invocation
are retained, alongside the calculator's numeric branch mapping, in
[the evidence bundle](apk/2.26.43.72-media-priority-context/README.md).
[Complete constructor REA Evidence](rea/2.26.43.72-media-priority-context/README.md)
corroborates checked DEX. The calculator conditionally consumes this typed
context when its boolean flag is1; Java's no-argument holder constructor passes
false, which does not establish the selected app factory's effective value.

**Bounded negative:** this invocation attaches internal context rather than
directly inserting a request header. It does not establish a missing Priority
header in Rust. No speculative header, scheduling value, deadline, retry or
persistence change was made. **Unknown:** provider activation, flag selection,
context numeric enum meanings, downstream HTTP/2/3 priority serialization and
server policy. No target execution/live account operation, acceptance or ban
claim. This resolves the immediate interceptor's direct-write scope; downstream
middleware and scheduling remain open.

Next concrete static boundary: retained `0eB.A00` at DEX byte0x059c..0x05ac
obtains `0eA.A0z.getValue()` as HttpPriorityCalculatorProvider and passes it
into the TigonHttpClient constructor at0x05f6. Trace that lazy producer before
assuming which holder/boolean value is selected; the User-Agent bundle already
retains the complete factory and 0eA class initializer.

## Selected priority calculator flag source (2026-11-06)

**Confirmed static:** 0eA.A0z wraps1Wv(2) in00q; producer case2 constructs
WAHttpPriorityCalculatorHolder with0eA.A06.A14(38178 /0x9522), and0eB.A00
passes this provider to the client. This closes the prior selected-provider
boundary. The app path does not use the holder's default-false constructor.
The synchronized lazy caches the successful result and clears its initializer;
the gate is not re-read on every request through that wrapper. A14 forwards a
null boxed default through A0D to the generic configuration resolver; null is
not a proven false. **Unknown:** effective value, stored/default overrides,
initialization timing, downstream scheduling/HTTP priority serialization.

[Complete checked DEX, native setter window and reproduction](apk/2.26.43.72-media-priority-selection/README.md)
and [complete REA class search/producer Evidence](rea/2.26.43.72-media-priority-selection/README.md)
retain both positive and negative results. The exported native HttpPriority
setter writes a separate24-byte field+0x158, so neither that symbol nor the
Java priority setter proves metadata+0x1e0 is HttpPriority. A raw-name REA lookup
failed; the discovered alias X.C1Wv succeeded. No Rust behavior change, invented
header/gate value, replay or live account operation. Next unresolved boundary:
where the selected calculator result is consumed for actual request scheduling
and HTTP/2/3 serialization, including any conditional wire priority fields.

## Native Priority header producer and standard upload prefix (2026-11-06)

**Confirmed static:** downstream TigonHttpPrioritizationService, rather than the
WA context interceptor itself, calls the calculator and upserts a `Priority`
request header. It clamps urgency to0..7 and formats `u=N`, conditional `, i`
and `, p`, plus `, r=N` for an engaged positive integer. The r input is read
from typed property key3 and overwrites the calculator optional members before
formatting; it is not proven absent on uploads. Header construction is confirmed,
actual wire transmission/acceptance is not observed.

[Full checked Java/native routines, JNI candidate, RTTI bindings, literals and
reproduction](apk/2.26.43.72-media-priority-header/README.md) connect provider
creation, shared ownership and two stack-building closures to the service.
One closure suppresses DefaultHttpPriorityContextInterceptor when a calculator
is supplied; another creates the prioritization service when its next service
exists. Named JNI class/table registration and effective graph activation are
still untraced, so the generic chain is not claimed as observed app execution.

**Confirmed standard Java upload input:** MOu.A03 sets K1R.A06=true/A05=false;
K1R yields MEDIA_UPLOAD(value8) and NONE(value0), corroborated by
[complete REA category Evidence](rea/2.26.43.72-media-priority-header/README.md).
Given category8 in the WA native context, either flag38178 result returns(3,1,0):
header prefix `u=3, i`, with possible property-driven r suffix. Existing non-WA
contexts/other metadata factories remain separate. The exported native category
setter identifies+0x1e0 independently from the HttpPriority field+0x158; full
native serialized-enum conversion is not newly established here.

Rust uploads have no explicit Priority header: a concrete mismatch candidate,
not yet a justified unconditional correction. Close the named JNI registration,
service graph activation and key3 producer/presence before selecting a final
Rust policy. No Rust request/header/timing/retry/persistence change or live
account operation in this turn. The prior bounded negative concerned only the
interceptor; this downstream producer resolves that broader header question.

## Priority JNI registration and normal graph activation resolved (2026-11-06)

**Confirmed static:** named service-holder registration copies the10-entry
native table containing the exact initHybrid ->0x4d6344 and registers it against
the service-holder class. The provisional prototype-only binding is now closed.
The normal composition routine invokes all16 closures backwards, including
priority slot13. Its condition reads the current shared service pointer through
an atomic load, rather than a separate priority-enable flag. A non-null current
chain is wrapped by TigonHttpPrioritizationService; null/zero results preserve
the chain. Runtime loading/execution and base/backend policy remain unobserved.

[Complete registration, full walk, pointer load, bounded searches and repro](apk/2.26.43.72-media-priority-activation/README.md)
retain the result. Java1FJ keys used by JAT contain no numeric3, and1FK stores
its integer directly; this is not a full native-property absence proof. Native
getter call and nearby-immediate searches did not identify the key3 producer.
**Still unknown:** first-upload key3 engagement/value and full serialized native
category binding. No inferred retry-count label, fixed r value or unconditional
Rust Priority policy was added. No Rust runtime/header/deadline/retry/persistence
change or live account request. The earlier named-registration and independent
stage-selection uncertainties are resolved to the static scope above.

## Priority property origin: Java and cross-library boundary (2026-11-06)

**Confirmed bounded:** all24 physical1FK constructor sites across7 classes
include23 fixed keys (none3) and one dynamic property decoder1GV.A05. That
decoder reconstructs arbitrary typed properties and is used by request and
summary decoders; direct callers include callbacks/observers and HUC request
bridging. It cannot be dismissed as response-only, nor does it identify the
original writer of key3. Two native tag checks qualify the root integer consumed
by the Priority formatter; a nested group key3 alone is not equivalent.

[Complete checked methods, caller censuses, source hashes and repro](apk/2.26.43.72-media-priority-property-origin/README.md)
extend the native search to all83 pinned reconstructed ARM64 members by symbol
substring. Main-library25 root and37 Group direct setter sites, plus importing
ffsingleton/mobileconfig libraries, are retained with explicit scope. Their four
external root setters use44/20/20/46, not3; the main decoder has a dynamic-key
write. No inlining/indirect/dynamic-code absence proof is claimed. Property3's
original producer/first-upload value and native category conversion remain
unknown. No Rust request/header/retry/deadline/persistence change or live account
operation. These negative results prevent a false fixed-header correction;
previous static Priority header construction/activation proofs still stand.

## Standard upload category/purpose native binding closed (2026-11-06)

**Confirmed selected static:** Java serializes the explicit category.value and
purpose.value as consecutive little-endian32-bit words after the four timeouts.
Native raw reader consumes4 bytes twice; w21/w27 reach stores atsp+0x260/+0x264,
which are request+0x1e0/+0x1e4 relative to base sp+0x80. AArch64 opcode/register/
offset decoding independently checks all four load/store instructions. Reader
vtable/body and intervening outlined-register preservation are retained in
[the evidence bundle](apk/2.26.43.72-media-category-binding/README.md).

This closes the previously open native enum-transfer boundary: standard MOu/JAT
category8/purpose0 reach the pair copied into WA priority context without an
intermediate remapping on the selected normal path. Its calculator prefix is
`u=3, i` independently of flag38178. **Still unknown:** original property3
writer and first-upload value/engagement for the optional r suffix, plus live
runtime/server acceptance. The byte example is synthetic, not captured traffic.
No Rust request/header/retry/timing/persistence behavior or live account action
changed. Full converter error behavior and arbitrary malformed inputs are not
claimed verified by this bounded transfer proof.

## View-once download result ownership (2026-11-06)

**Confirmed local defect / fixed offline:** view-once GET persisted definite
HTTP refusal or authenticated download completion with unconditional checkpoint
writes after network awaits. Unlike the ordinary-download ownership guard,
these writes could recreate a removed checkpoint or replace a changed attempt,
consumed/refused/completed outcome, or outlive the source inbox row.

`image_download::download_view_once` now retains the exact selected inbox row
in zeroizing memory. Its new-download reservation transaction requires that
source row to remain byte-identical and no prior download state to exist. It
persists a fresh local reservation ID plus the existing unknown outcome/time.
The ID is never sent on the wire and does not authorize another request.

Both definite HTTP refusal and authenticated ciphertext completion now compare
the exact initiating reservation AND the original source row in the same write
transaction. A missing/changed row returns local InvalidInput(image_download)
without replacing current state. Successful completion returns only after the
checked commit. The extra fetched inbox bytes are zeroized after comparison.
Existing cached completed downloads, old uncertain/no-retry rows, config/account
preflight, crypto/length bounds and atomic consumption retain their paths.
No opening, opened receipt or plaintext export was added.

Regression `late_view_once_results_preserve_reservations_and_source_after_reopen`
first failed with the old unconditional write retained in the extracted
settlement helper (case missing), before adding the settlement checks. It now
covers missing/differently owned/legacy unknown checkpoints, consumed/refused/
downloaded outcomes, removed/changed inbox sources, and exact success/refusal
controls. It checks unchanged checkpoint bytes and journal on refusal, persistence
across reopen, one-time success/refusal, duplicate/late result refusal and
refusal to reserve again. `view_once_reservation_refuses_changed_source_without_mutation`
checks the pre-network source fence independently. All tests use synthetic
records and real encrypted local stores; no network server or account operation.

Validation:1,511 workspace tests pass (9 result suites), strict workspace/
all-target/all-feature Clippy and formatting pass. No new APK claim or native
parity inference is made: this is a Rust state-ownership correction, following
the existing ordinary-media invariant in this document.

**Limits:** this does not make HTTP and storage atomic. A source can change
after reservation while a GET is in flight; late settlement is refused. Exact
source bytes deliberately refuse even a metadata-only rewrite rather than guess
that it is equivalent. No generic retry, checkpoint reset, stale-body recovery,
identity rotation or broader account/session-generation race guarantee is added.
The native Priority optional-property uncertainty remains open independently.

## View-once consumption source identity (2026-11-06)

**Confirmed local defect / fixed offline:** consumption selected an image
descriptor before opening its write transaction, but inside that transaction
checked only that the current inbox status was decrypted. A replacement source
row could keep that status, causing the old descriptor/ciphertext to be used
while overwriting the replacement row and taking its route/capture metadata.
The earlier GET settlement guard did not cover this independent local handoff.

Selection now retains the original inbox bytes in zeroizing memory. The
consumption transaction compares that exact source before ciphertext inspection,
decryption or either consumed-checkpoint write, using the same source guard as
reservation/settlement. Missing or changed bytes return local
InvalidInput(image_download), preserving both records. Valid input still
atomically removes accessible body/protobuf records, returns the same opaque
ticket only after commit, and leaves opened_receipt=awaiting_viewer. This does
not assert that consumption displayed an image or sent an opened receipt.

Regression `consumption_refuses_changed_source_without_consuming_either_record_after_reopen`
failed on the replacement case while the old status-only transaction was
retained in the extracted private consume helper. After the guard it covers
replacement protobuf, metadata-only route rewrite, missing source, consumed
source, corrupted ciphertext and a valid control. All use synthetic encrypted
images and real encrypted stores. Refusals preserve checkpoint bytes and journal
across reopen; valid consumption preserves original route/capture, returns
expected plaintext/ticket, removes accessible body/protobuf and refuses reopening.
Existing public opening/GET ownership tests also pass. No user content was
opened, viewer launched or live account operation performed.

Validation:1,512 workspace tests pass (9 result suites); strict workspace/
all-target/all-feature Clippy and formatting pass.

**Limits:** exact bytes intentionally refuse metadata-only rewrites. The guard
binds the selected source, not a transaction-wide snapshot of every inbox
candidate or every account/session-generation race. Database-page secure erasure,
viewer success and server-open synchronization remain separate. No repeat GET,
consumption reset, retry or new opened receipt is introduced. This is a Rust
state-identity correction; no new APK policy or ban-cause claim is made.

## Opened-receipt collision and replay audit (2026-11-06)

Rust `transport::receipts::queue_opened` was reviewed at the local viewer-to-queue
boundary. It creates the pending played receipt and changes the consumed inbox
marker from awaiting_viewer to queued in one store transaction. Existing receipt
rows block replacement independently of their outcome. No production defect was
found in this transition; no production behavior changed.

Regression `opened_queue_refusals_preserve_checkpoint_bytes_and_journal_after_reopen`
uses synthetic routing and real encrypted stores. It checks missing source,
unconsumed source, queued/unknown/empty viewer markers, unavailable/malformed
routes, and collisions with pending, outcome_unknown, server_acknowledged and
refused receipt rows. Repeated refusal and refusal after reopen preserve every
checkpoint byte and the journal head. Existing
`opened_queue_survives_restart_and_never_replays_uncertain_receipts` supplies the
valid transition and one-time reservation/ACK controls.

Limits: this verifies local queue state, not actual viewer rendering, stanza
transmission, server acceptance, every PN/LID/group routing identity, or account
generation replacement. The ticket remains derived from the inbox key; no new
ticket generation policy is inferred. No viewer, user content, account or network
operation was used. No new APK or ban-cause claim is made.

Validation: 1,513 workspace tests (9 result suites), strict all-target/all-feature
Clippy, formatting, OpenAPI library coverage, CLI reference regeneration, APK
coverage freshness and repository hygiene pass.

## Viewer-completion privacy hold (2026-11-06)

Local viewer completion now queues automatic played provenance and can leave
that original pending intent held by known-disabled read-receipt privacy.
Consumption remains irreversible; neither a hold nor writer refusal permits
reopening. [Pinned policy, regression and limitations](read-receipt-policy-current.md#automatic-played-privacy-correction-2026-11-06).
Queued intent remains distinct from transmission/server acceptance; no live
image or account operation was performed for this correction.


## Wrapped lifetime conversion before local route eligibility (2026-11-06)

**Confirmed pinned static / implemented offline**, APK2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The complete retained [1t5 constructor](rea/2.26.43.72-media-route-lifetimes/model-checked.txt)
performs separate signed64 mul-long by1000 at004e and0058 for ttl/auth_ttl,
then adds the supplied request base. DEX multiplication wraps. Rust upload route
selection instead chose the smaller raw seconds value and saturated its product.
A valid signed-long lifetime that wraps to an expired duration could therefore
remain eligible under Rust's route/auth policy, causing use of stale credentials.
No actual server response or account restriction was observed.

Route selection now converts EACH lifetime with wrapping multiplication before
checking the existing local minimum-lifetime policy. Wrapped nonpositive duration
or elapsed duration refuses with the existing expired-route/explicit-refresh error.
The positive-seconds rule and request-time provenance checks remain unchanged;
negative wire lifetimes do not gain new authority merely because they could wrap
positive. Nor does every overflow refuse: a positive wrapped duration still uses
its resulting value under the same age guard. Raw reply/history and request time
are never changed, and this path schedules no automatic refresh or HTTP retry.

Independent executed host OpenJDK27 results from the existing
[Java oracle](MexJavaNumericOracle.java) confirm the selected arithmetic vectors:

| Signed seconds | Wrapped milliseconds | At request age >=2000ms |
| --- | --- | --- |
| 3600 | 3600000 | usable under the other lifetime |
| 9223372036854775807 | -1000 | expired |
| 9223372036854776 | -9223372036854775616 | expired |
| 2305843009213693952 | 0 | expired |
| 18446744073709552 | 384 | expired |
| 18446744073711352 | 1800384 | usable under the other lifetime |

These are host arithmetic reference results, not Android/APK execution. The DEX
operation itself establishes signed64 wrap; no runtime Unicode digit table is
needed for these ASCII cases. The new real-store regression applies every vector
independently to ttl and auth_ttl while keeping the other at3600, checks eligibility
before/after reopening, and verifies EVERY table and journal are unchanged.
The384ms case catches both saturation and taking the minimum before converting.
The positive wrapped vector prevents replacing this with a blanket overflow ban.
Existing missing/invalid/BMP lifetime, delayed response, auth-shorter-than-route,
legacy provenance and refresh-ownership cases remain in the workspace suite.

**Still unknown/unported:** native deadline ADD overflow, exact elapsed-clock/
reboot parity against the local wall-clock age model, complete TTL-versus-auth
consumers, native refresh scheduling and live host/auth acceptance. This correction
is specifically the seconds-to-ms conversion within the existing local guard;
it does not establish complete native deadline parity or make a ban guarantee.
Route selection runs before upload encryption/seed/reservation in the existing
new-upload path. No live HTTP/account operation or state migration was performed.


## Media deadline consumers separate refresh from auth validity (2026-11-06)

**Confirmed selected static / research only:** exact1t5 field census finds A04
route deadline read only by0VM.A0T's120s refresh test; auth A02 has distinct
cache/timer/wait/upload/supplier consumers. A06 schedules auth expiry minus60s;
A0C requires changed auth versus an optional old token plus strict future expiry
with a caller margin. A0B waits in <=1s chunks only with request flight, using an
uptime budget while auth uses elapsedRealtime. Selected upload paths pass120s
or0 margin with20s wait input, and may still accept future auth after requesting
refresh. Feature/online/offline and post-offline-skip transfer checks remain
separate. This refutes a blanket120s hard auth cutoff and does not establish
route TTL as every upload's hard expiry. Rust's current minimum is local policy.
[Complete field census, nine methods, exact gates and required next work](apk/2.26.43.72-media-deadline-consumers/README.md).
No timer, automatic refresh, runtime behavior or live account operation changed.


## Atomic upload route/account fence at reservation (2026-11-06)

**Confirmed Rust race / implemented local correctness fence.** The selected
[pinned upload/supplier evidence](apk/2.26.43.72-media-deadline-consumers/README.md)
retains feature-gated auth identity/expiry rechecks near transfer; it does NOT
establish an unconditional native120s cutoff or every native account check.
Independently, Rust selected its host/auth before encryption, then reserved and
posted using those captured values without revalidating route age/currentness.
Its initial registered-account/phone precondition could also change during that
preparation window. This correction fences those existing local preconditions;
it does not port native refresh timers, offline-skip or replacement-token policy.

New-upload reservation now reads current coordinator/legacy selection and checks
expiry, selected host/auth, registered account, phone and generation in the SAME
write transaction that commits the upload seed and outcome_unknown row. It will
not silently substitute another URL/token when material changed. Legitimate new
metadata retaining the same selected host/auth is not rejected merely for having
a different response timestamp; current validity and ownership still must pass.
Mutable authentication timestamps/credential observations are not frozen, and
full same-phone identity-replacement/account lifecycle parity remains open.

The seed is read/generated and used to build the URL inside the serialized
reservation. A new seed is persisted only alongside the successful reservation;
a failed builder, route/account check or duplicate cannot leave an orphan seed.
Concurrent successful uploads therefore reuse the committed seed instead of
constructing different token inputs outside transaction ownership. Large cipher-
body metadata is materialized before taking the writer; only seed-dependent URL
assembly, authority checks and final persistence are added within that boundary.
Account checks run after preparation and require the previously consumed phone/
generation to remain registered. No new identity/credential values are invented.

Coordinator active-record selection is factored over the same raw transactional
get/scan rules for read and write transactions, preserving owner/current pointer,
sealed projection, generation, action/receipt/request binding and legacy fallback
refusals. It does not nest a read transaction inside a writer or add a full-history
scan on the owned active-pointer path. JSON corruption retains storage-serialization
classification and redaction rather than becoming an ordinary input refusal.

Real-store tests select a route, then independently expire/change auth/change host,
refuse/remove it, introduce invalid coordinator ownership, remove/revoke/change
the account, or fail the URL builder. Each case runs with and without an existing
seed, returns no reservation and preserves EVERY table/journal, including reopen.
The success control commits seed+outcome_unknown together, preserves an existing
seed, and refuses duplicate reservation before calling the builder. It does not
pretend to be a completed upload or allow saved_payload reuse. A separate
coordinator test observes uncommitted owner/corrupt-current changes through the
new write reader and verifies full rollback and preserved error class/redaction.
Existing upload response snapshot/uncertain-replay tests remain in the suite.
No live HTTP/account operation or automatic refresh/retry was performed.

**Boundary:** this is reservation-time authority, not a guarantee that clocks,
account state or server auth cannot change after commit or while HTTP is in flight.
Late upload response account ownership and complete native transfer/refresh/clock
parity remain independent audit work. Existing uncertain POST reservations remain
held; this change never resets or replays them. Cached completed payload behavior
and the current local route-TTL/auth-TTL minimum policy remain unchanged.
