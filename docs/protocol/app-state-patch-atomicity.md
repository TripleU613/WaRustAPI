# App-state patch queue and settlement atomicity

2026-11-04. **Confirmed source defects and offline corrections** in the Rust
comparison client; no account traffic or restriction causation is established.
Current APK2.26.43.72 transaction evidence is retained in
[server-sync lifecycle](server-sync-lifecycle.md): verified patch application
calls `0mj.A00` before transaction success. The outgoing/incoming version
separation is retained in [held-version construction](app-state-versioned-fetch.md).
This is not a claim of complete native scheduling parity.

## Confirmed defects

- `queue_patch` separately committed key generation/selection, queued wire and
  prepared state. A crash could leave a sendable action with no prepared state;
  concurrent publishers could claim the same collection baseline. A stored-key
  selection error was also treated as a reason to generate another key.
- `ingest` selected the first returned collection/version without checking its
  name against the original request. It wrote prepared state unconditionally,
  potentially replacing a later held state. Malformed/mismatched replies deleted
  preparation instead of preserving uncertainty.
- Conflict catch-up loaded state outside the commit transaction, verified keys
  in separate transactions and then unconditionally wrote the result. A later
  invalid patch could leave deletion/application boundaries inconsistent.
- Generic action settlement interpreted any matching IQ that was not `result`
  as a refusal, including `get`, missing type and future types. Current native
  `0ln.A01` dispatches only result/error; see [the retained outcome trace](authkey-outcomes.md).

## New queue and reservation boundaries

One write transaction checks for an unsettled patch of the same collection,
selects an eligible key (minting only on a successful no-key result), loads the
held state, builds the encrypted patch, and commits original wire plus a bound
baseline/prepared record. Errors roll back key generation too. The original
queue ID, encrypted patch and nonces are not rebuilt on socket reservation.
This per-collection local guard is not the entire native global batch scheduler.

Both action reservation paths revalidate preparation before marking the action
sent. They compare held state with the frozen baseline and verify the original
patch against held keys and its prepared result. Missing preparation, stale
baseline, corrupted post-state or invalid original wire refuses locally without
marking the action sent. Signed-Long version overflow is also refused locally.
No existing identity or key is replaced to resolve these refusals.

## Atomic accepted/refused settlement

The current action, preserved reply, original request, pending record, baseline,
keys and commit share one transaction. A success requires the original ID,
exactly one sync/collection with matching name, and the claimed prepared version.
The prepared transition is MAC-verified again. A later/different held state
cannot be overwritten; malformed or stale success keeps the pending record.
An already identical prepared state may clear its pending marker idempotently.
A definite outer IQ error requires a singleton error detail with an integral
code before releasing preparation without advancing held state. Malformed
refusals retain preparation rather than silently becoming a safe-to-repeat result.

An exact collection conflict uses [atomic catch-up](app-state-catchup-atomicity.md).
All patches and keys verify before the state and pending release commit together;
a malformed later patch rolls back the whole transaction. No conflict authorizes
unknown key adoption, reset, or automatic replay. The receive loop settles only
the current original action; local shape/stale refusals are logged and retained
while receiving continues. Storage faults still propagate visibly.

Unknown/missing IQ types now leave the original action sent/outcome-unknown.
They cannot authorize another identity read or a privacy-setter override merely
by carrying an existing ID. Valid later result/error responses can still settle.
The directory/identity tests now assert that uncertainty instead of a fabricated
refusal. Two generic-classifier fixtures use a neutral action kind: they are not
valid app-state writes and must not bypass the new prepared-state reservation.

## Existing stores and unresolved boundaries

The old prepared-only state format is still readable. For queued rows, the
original immutable wire is cryptographically reverified against current held
state and atomically bound before reservation. For sent rows, accepted promotion
reverifies the same transition from the held state. The private
`CollectionPatch::decode_prepared` supplies the frozen claimed version to the
local verification model because outgoing protobufs omit the read-only version;
it changes no request bytes and is never an incoming-network decoder.

After the [logical-index correction](app-state-logical-index.md), an older
preparation may lack index metadata even though its original wire and frozen
integrity fields are valid. Reservation and accepted settlement now derive the
post-state from that exact authenticated wire using the current authenticated
baseline. Canonical collection/version/hash/wire-MAC maps must match every frozen
field. Only absent legacy logical-index metadata is enrichable; supplied metadata
must match exactly too. A previously frozen baseline without metadata can use the
current baseline's authenticated indexes only when all its frozen integrity
fields agree. A nonempty current baseline without provenance still refuses.

Reservation stores the completed baseline/preparation in the same transaction as
the original action's first reservation. Accepted settlement stores the completed
authenticated result in its existing atomic transaction. Neither path changes
request bytes, identifiers, encryption randomness, keys or send status to retry
anything. Already-held matching post-state only releases pending preparation;
it does not overwrite metadata. Missing entire preparations remain unresolved.

If an old crash already lost preparation, queued orphan actions refuse before
reservation. Already-sent orphan outcomes remain preserved in the action log;
this change does not invent their lost state or replay them. Reconciliation of
such historic ambiguous outcomes remains open. A stale accepted baseline is
likewise retained for explicit resynchronization, not silently rebased. Generation
and native role/recovery/mutation-row eligibility, full pagination/refresh state,
and automatic notification scheduling remain separate required work.

## Verification

Eight settlement regressions cover concurrent publishers, queued/sent/reopen
single-flight behavior, invalid-state/key rollback, wrong collection/version and
ambiguous replies, a later real cryptographic state, legacy queued/sent rows with
unchanged original wire, and orphan/corrupt reservation refusal. Catch-up and
independent SET/REMOVE-vector regressions cover joined transactions, full rollback,
reopen, unknown keys, invalid MAC/protobuf/nonbytes and non409 refusal. Generic
IQ classification and legacy privacy-setter guards have dedicated negative tests.
No test used an external server.

The legacy compatibility matrix covers queued/already-sent actions, prepared-only
and bound formats, and empty/nonempty authenticated baselines. Frozen preparation
metadata is actually absent in these fixtures. Positive cases retain exact
original wire and commit the complete derived post-state. Negative cases preserve
all checkpoints when the current nonempty baseline lacks provenance or supplied
prepared logical indexes contradict the authenticated patch.

## Explicit server backoff

Current `FUc.Bvj` parses outer-error `backoff` as integral seconds; see its
[checked instructions](rea/2.26.43.72-server-sync-lifecycle/server-sync-request-handler.txt).
Explicit nonnegative waits persist in feature-specific refusal processing,
anchored to the action's original durable receipt time. Generic IQ settlement is
an earlier separate durable commit; recovery closes that application cut without
resending the request. Repeated inspection and
restart do not extend or erase that wait. New fetches and patches refuse with a deadline;
never-sent queued fetches/patches defer without being marked sent. Unrelated queued
control requests can still proceed, and expiration reserves the original patch
once without rebuilding its ID or bytes. Already-sent uncertain requests never
return to queued state. The longest retained explicit deadline is preserved.

The native -1 sentinel adds no explicit wait. Malformed, other negative or
local-time-overflowing values retain the saved outcome and any pending preparation rather than silently
removing a wait; these overflow/strictness bounds are local defensive policy.
No invented default wait, error-code retry recipe, automatic request replay or
complete native backoff scheduler is claimed. Nested conflict-specific retry
policy remains unclosed. Two additional regressions cover restart, fixed receipt
anchoring, expiry, unrelated traffic, invalid hints and pending-state rollback.

### Fetch-specific recovery correction

**Confirmed local defect, fixed:** fetch actions have no `PendingState`, so the
old `ingest_action` returned before reaching outer-error backoff handling. The
same current-build `FUc.Bvj` callback handles the assembled request's outer error
whether collection children were reads or writes. The fetch path now validates
its supported immutable request envelope, exact correlated outer error and
definite error detail before calling the same receipt-anchored wait writer.
Inner collection errors do not acquire this global outer-error policy.

Startup reprocesses saved fetch refusals before draining queued work. Malformed
waits remain inspectable and logged with fixed text rather than ending receiving;
storage faults still propagate. Explicit new fetch preparation and patch ingestion
also check saved fetch refusals strictly, so an unsupported historical refusal
is reported rather than interpreted as permission for a new explicit fetch.
Existing queued work has no newly inferred wait duration for an unparseable hint;
full native unknown/error classification remains required.

Successful fetch snapshots, inline patches, pagination and external blob handling
remain unported by this correction. It does not promote held versions, install
keys, consume a fetch result or schedule a retry. Three encrypted-store regressions
cover live/cut/reopen wait recovery, non-extension, deferred fetch/patch reservations,
unrelated control progress, malformed redaction/rollback and exclusion of unrelated
or inner-error paths. Original request bytes remain unchanged.

## Java decimal acknowledgement and conflict spelling (2026-11-04)

**Confirmed Rust mismatch/fixed offline:** accepted patch settlement used Rust
`parse::<u64>` while conflict catch-up compared the error code to literal `409`.
The pinned 2.26.43.72 F2T/0Tp readers use Java decimal parsing, as retained in
[fetch classification](app-state-fetch-completion.md#structural-classifier-api).
Both settlement checks now reuse the existing UTF-16/BMP Java signed-long helper.
The acknowledged value must still be nonnegative and exactly equal to the
validated prepared version, bounded by signed-long range. Conflict classification
still independently requires the native integer value 409. No alternate spelling
can supply a new hash or bypass original-request/key/MAC/baseline checks.

Handwritten cases cover leading plus, Arabic-Indic and fullwidth digits, leading
zeros, negative/mismatched values, whitespace, signed overflow and supplementary
mathematical digits. Successful settlement promotes only the authenticated frozen
prepared state and repeats do nothing; refused cases preserve all checkpoints.
Conflict spelling tests use independent literal authenticated patch vectors and
check retained actions. These are offline source tests, not observed server
spellings or a claim about Android Unicode-version differences. No live traffic
or new APK evidence was introduced.
