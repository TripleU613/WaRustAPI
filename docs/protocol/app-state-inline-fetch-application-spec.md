# Authenticated inline fetch application specification

2026-11-04. **Confirmed source** refers to main `136fc38`; **confirmed native**
refers to WhatsApp Android 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The pure claim-binding/header stage below is now implemented; the durable
admission and authenticated result processor remain proposed. No network request
or account state changed during implementation.

Evidence is already retained in [fetch completion](app-state-fetch-completion.md),
its [F2T envelopes and checked parser](rea/2.26.43.72-sync-fetch-completion/README.md),
and [lifecycle](rea/2.26.43.72-server-sync-lifecycle/README.md). The existing
[atomic 409 catch-up](app-state-catchup-atomicity.md) and
[logical-index correction](app-state-logical-index.md) are reused, not re-audited.

## Bounded supported operation

### Implemented pure binder

`binding::FetchBaselinePlan::new` pins the SHA-256 of supplied exact encoded
request bytes, a local generation, action ID and supplied saved baselines.
This object is deliberately a **claim**, not proof of authenticated store origin.
`bind_inline_fetch_plan` takes the current transaction-generation value as an
input, checks generation/fingerprint and decodes those same frozen bytes through
the shared action decoder. It does not re-encode a Node to manufacture identity.

The binder validates supported request grammar/attributes, explicit version equal
to complete internally validated claimed baseline, and exact request/admission
domains. It reclassifies the preserved original reply and compares the whole
public model, rejecting inconsistent supplied names/categories/bodies. Response
duplicates or names outside the request fail local binding; missing requested
responses remain unresolved. Pure-fetch acknowledgements and already-synced
categories never advance a baseline. Data candidates receive complete selected
collection header preflight before a bound plan is returned; snapshots, external
mutations and collection refusals remain distinct unsupported/deferred outcomes.

The output vector follows request order for membership, with `response_index`
and the original response retained. It is **not** an application-order directive:
the future worker must preserve native response-category/within-category order.
Headers alone never authorize application. The binder cannot detect a different
internally consistent current store map without an authoritative comparison;
the application transaction still owes full baseline/generation/pending-owner
rechecks. Durable admission capture, cursor writes, keys/MACs and state commits
are not implemented by this pure stage. No retrospective claims are created for
historical `Sent` actions. Debug/errors redact raw originals and private values.

Five handwritten binder regressions cover generation/changed bytes, version-zero
without a snapshot flag masking the guard, exact domain/provenance and valid
empty legacy state, response model tampering, response-order retention,
no-advance acknowledgements, whole-header preflight and external deferral.

**Proposed:** apply authenticated **inline incremental patches** from a saved,
correlated patch-free fetch to the transport's authoritative plain-name held
checkpoints. Preserve its original request and reply and make local processing
restartable. No snapshot installation, external retrieval, key acquisition,
mutation action projection, automatic continuation query or owner-flight release
is introduced by this operation alone.

The pure response classifier is structural only. A category named `Data` is
not permission to change state; `VersionAck` is not an authenticated new held
baseline. Known collection names are not proof of original request ownership.

The initial supported contract needs an exact requested collection set, explicit
nonnegative request versions and complete authenticated baseline provenance.
Absent-version requests, snapshot-bearing results and unsupported/external/exit
branches receive durable **unsupported** outcomes with original evidence retained.
Unknown response nodes stay preserved. These limits are local support boundaries,
not claims that the server or native parser universally refuses those shapes.

## Original request and baseline binding

**Confirmed local generation fence:** `WriteTx::generation()` exposes the actual
store `AccountGeneration`. `WriteTx::get` passes that generation through
`open_record` into `RecordAad`; the envelope binds it cryptographically.
`bump_generation` takes effect for later transactions, and the existing
`store::tests::generation_bump_fences_old_records` regression proves old rows
cannot open under the new generation. A prospective fetch plan can capture this
real local generation without inventing a native per-collection generation
field or changing any wire attribute. This does not prove that every logout or
account-switch path calls the fence; lifecycle integration still needs its own
evidence. No generation was bumped during this audit.

**Confirmed Rust:** `queue_fetch` freezes all five collections at version zero
with `return_snapshot=true`. The separate `fetch_request_at_versions` primitive
constructs explicit held versions but is not a durable scheduler. A queued
request is immutable. It is incorrect to reinterpret a version-zero request as
a request from current held version N, or re-encode it with N before application.

**Proposed admission:** atomically capture a fetch processing plan in the same
write transaction as its prospective socket reservation. It binds:

- Account generation and action ID/kind, plus a fingerprint of the original
  immutable encoded action request. Fingerprint the stored wire bytes directly;
  do not encode a decoded Node and call those identical original bytes.
- Validated IQ `type=set`, app-state namespace, server destination, correlated ID,
  one supported sync node and patch-free requested collections. Validate consumed
  singletons/attributes and producer collection uniqueness as explicitly local
  support policy. A misleading action kind cannot replace wire validation.
- For each requested collection, exact requested version and restored
  `SavedCollectionState`: collection identity, version, LT hash, wire-MAC map
  and complete authenticated logical-index provenance. Preserve both absent
  checkpoint and present empty checkpoint as distinguishable observations.
- Relevant pending-write ownership and any established recovery ownership at
  admission. Outstanding uncertain/prepared writes must not be bypassed by
  fetching over their baseline. Do not invent a new recovery/role predicate for
  authenticated incremental verification: the unclosed scheduler initialization
  gates stay separate, and an absent lifecycle row cannot authorize new sends.

For the supported incremental path, the request's version must equal its
captured authenticated held version. A missing or legacy unknown-provenance map
is diagnostic/reconstruction work, not an inferred clean baseline. A missing
checkpoint is not automatically authorized companion bootstrap; bootstrap role
and recovery initialization remain separate prerequisites.

Consequently existing version-zero fetches cannot automatically repair arbitrary
nonzero held checkpoints. Old `Sent`/settled requests without an admission plan
remain inspectable but do not get a retrospective baseline guessed from present
state. An explicit later authenticated reconciliation design may support them;
this spec does not discard them, retry them or call them corrupt.

Account generation is already part of encrypted store row authority. An explicit
plan generation strengthens diagnosability; old-generation decryption failures
remain failures, not absence. A connection epoch records provenance but does not
make a late captured matching response unusable by definition: admission-plan
identity and current baseline decide local reconciliation. A late reply must
never release a different live owner token.

## Preflight before the first commit

**Confirmed native:** `FUc.CK4` constructs the complete `F2T` response before
scheduling `FeN.run` case40. `F2T.A00` protobuf-decodes its selected collections'
patch lists and checks each patch's version presence, duplicate versions within
that list, exit-code branch and mixed external-reference/nonempty-inline data.
Thus a late malformed patch header can fail parsing before any incoming
application begins. This is not a claim that all collection cryptographic
verification is atomic: `A07` authenticates and commits subsequently.

**Proposed Rust stages, without writes:**

1. Load and validate the immutable action/request/reply and processing plan in
   one consistent store view. Require matching saved reply ID and a supported
   result action. Preserve its immutable stored-reply fingerprint. Generic action
   status `Ok`/`Error` is receipt classification, not completed processing.
2. Run the existing pure classifier, then bind every collection summary to the
   requested set. Unexpected names or duplicate response collection ownership
   are rejected before writes by local supported-response policy, without
   claiming F2T imposes global uniqueness. Missing requested collections become
   explicit `MissingResponse` outcomes; do not mark them already synced.
3. Decode **all selected supported inline patch bodies across the response**
   before the first held-state commit. Use `CollectionPatch::decode` for its
   existing size/field/record/singleton checks and typed key/MAC fields; it
   explicitly refuses external field3 and exit-code field7. Track duplicate
   incoming versions within each collection's full list and signed-long range.
   Preserve original body fingerprints and wire order. Never sort or normalize
   signed/MAC-bearing bytes.
4. Classify unsupported data/refusals before committing a neighboring supported
   collection. Distinguish unsupported shape from corrupt protobuf/header. A
   malformed response/header fails the preflight with **no new held writes**;
   supported but unimplemented branches may be retained as durable unsupported
   outcomes. The exact partial-support admission policy must be explicit to
   callers, not silently skip a field and announce complete success.

`CollectionPatch::decode` validates more than native F2T's header stage (for
example supported fixed widths/record structure). Using those existing stricter
checks uniformly before commits is local defensive policy. It does not recreate
all F2T fatal codes or establish cryptographic validity. There is no need to
loosen the existing decoder to accept external data in this chunk.

## Per-patch transaction and durable processing cursor

**Confirmed native:** incoming `A07` reads current version under its mutation
lock and DB transaction, checks current+1 for incremental data, resolves record
keys separately from the enclosing patch key, and commits mutations/hash/version
before transaction success. The native response can therefore leave earlier
patches or collections committed if a later cryptographic application fails.
The Rust 409 helper instead intentionally verifies its whole chain in its
caller's one transaction; keep that proven atomic settlement behavior unchanged.
Do not expose that private function to a non409 route merely to reuse its name.

**Proposed local checkpoint model** (names/schema not implemented):

| Information | Purpose |
| --- | --- |
| Immutable plan/action/request/reply identity | Bind a cursor to precisely one captured operation and generation. |
| Original per-collection baseline | Establish admission authority; never reconstruct from latest state. |
| Expected current state after last committed step | Full version/hash/wire/logical provenance comparison before the next step. |
| Next patch index plus committed patch fingerprints | Resume exact local processing after crash without applying a patch twice. |
| Per-collection disposition | Pending, applied-with-continuation, complete-inline, missing response, unsupported, key-deferred, verification failure or stale baseline. |
| Has-more/dirty hints | Schedule eligibility inputs only; not a held version or an implicit request. |

Store cursors under generation-authenticated transport checkpoints, separate
from hex-name storage staging state. Do not create an account-wide durable
`Sent` lock or equate this processing checkpoint with a volatile network flight.

For each supported patch, one `store.write` transaction must:

1. Revalidate action/reply/plan identity, generation and cursor and load the
   authoritative held checkpoint. Require exact agreement with the cursor's
   expected current state; version equality alone is insufficient. Concurrent
   changes, including authenticated same-version metadata changes, need an
   explicit compatibility rule; reuse frozen-state equivalence only where its
   cryptographic/provenance meaning is appropriate.
2. Require the decoded patch version to equal held+1 without overflow and remain
   within native signed-long range. Resolve the patch signing key and every
   record key from already-held keys inside this same transaction. Never install
   a key from response bytes or assume record keys equal the signing key.
3. Invoke the existing authenticated verifier against complete logical indexes.
   On failure, preserve held and cursor. A separate typed diagnostic may record
   key-deferred/stale/verification refusal; the failed transaction cannot partly
   advance the applied cursor. Do not catch store authentication faults as mere
   untrusted network refusal.
4. Persist verified held integrity state and advance that collection's exact
   cursor atomically. A crash before commit changes neither; a crash after commit
   resumes at the following patch. Record integrity application distinctly from
   still-unported application/action projection.

A later failure must not undo an earlier committed patch. Conversely an action
whose cursor is complete must not replay because its generic settled row remains
visible. Completion/progress markers need receipt-order recovery just as other
local projections do. A fresh runtime may continue local work; it never sends the
same frozen request again to repair a processing gap.

## Version acknowledgements and empty results

**Confirmed native:** classic `FeN` handles `EQq` version acknowledgements using
`EfB`'s original prepared mutation map and resulting hash, then deletes those
pending mutations and writes that prepared hash with the acknowledged version.
This is an outgoing accepted-write path, not evidence that a pure fetch may
promote a server-supplied version alone.

**Proposed pure-fetch policy:** a `VersionAck` is an explicit unsupported/unexpected
fetch acknowledgement. Never change held version/hash/map from it, even when
positive, equal to a dirty hint, or greater than held. An ack equal to held can
be retained as a no-advance observation; it must not imply all unsupported data
or missing requested collections were processed. Negative/sentinel handling
from the classifier does not make other signed values eligible app state.

`AlreadySynced` has no new integrity state and cannot manufacture a zero/empty
checkpoint. It may close that collection's request disposition only after
request/baseline binding and after unresolved pending-write ownership is
accounted for. In particular an absent checkpoint is not proven bootstrap
completion, and an old snapshot request is not reinterpreted as a current
incremental read.

## Continuation and release boundaries

**Confirmed native:** when no more patches remain, `A04` clears only the
forced-zero dirty sentinel through `0l8.A06`; positive dirty versions remain
compared with applied state. After response categories are handled, manager
release/rescan can discover additional dirty work. `has_more_patches=true`
is therefore not a requirement to hold one network flight forever.

**Proposed:** after the last supported inline patch, record `NeedsContinuation`
when has-more is true. With false, record `CompleteInline` for the supported
integrity-processing scope only. Retain advertised dirty version separately;
a newer notification during processing must survive for a subsequent eligibility
check. This chunk schedules **no continuation IQ**, clears no unimplemented
native recovery state and does not ACK notifications.

An eventual coordinator can release its specific owner after every affected
collection has a durable processed/deferred/failure disposition and pending
baseline obligations are accounted for. Release is not a universal success
claim; no missing-key, unsupported snapshot, incomplete projection or uncertain
write is erased. Independent control IQs remain independent.

## Required tests before application code is integrated

- Handwritten multi-collection response whose last patch has invalid protobuf,
  missing/duplicate version, external/exit-code branch or mixed data: preflight
  performs no held-state writes. Opaque unsupported data never silently passes.
- A successful first patch/collection followed by a bad MAC or missing held key:
  earlier committed progress survives reopen; failed step and its cursor do not
  advance. Retry local processing resumes exactly at that step without network.
- Same-version/different-hash or map baseline, generation change, unrelated IQ ID,
  unrequested/duplicate collection and mismatched request version refuse before
  promotion. Legacy missing plan remains diagnosable rather than guessed clean.
- Exact prepared-hash authority is required for writes; a pure-fetch version ack
  never changes held data, including a plausible positive version.
- Separate record/signing key IDs use already-held keys in the write transaction;
  no key activation or overwrite follows unauthenticated response material.
- Concurrent held update between preflight and application is caught by the
  transaction; crash points before/after held+cursor commit do not lose or replay
  progress. Has-more retains continuation without issuing any request.

No tests were run for this documentation-only specification. Native role/recovery,
external-download enforcement and full action projection remain the distinct
prerequisites already recorded in the linked audits.
