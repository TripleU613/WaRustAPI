# Inline fetch binder: independent invariant review

2026-11-04, main `1f60b8b`. **Confirmed Rust** means current source inspection;
**proposed** means a pure API/test contract, not implemented authentication or
application. No APK re-export, network operation or source change occurred.
This review complements the [inline application specification](app-state-inline-fetch-application-spec.md)
and [header audit](app-state-patch-header-audit.md); it does not duplicate their
protobuf or cryptographic work.

## Generation and baseline authority

**Confirmed:** `ReadTx::generation` and `WriteTx::generation` are the generation
used in record AEAD associated data. `ProtocolStore::generation` is a separate
current-store view. A binder integrated with a transaction should use that
transaction's generation, not an earlier/later independent store read.

`WriteTx::bump_generation` updates metadata and records `new_generation`; the
current transaction retains its original generation. The new generation takes
effect for later transactions. Do not label a plan with the returned next value
while its request/baselines were read/written under the old transaction value.
Store regression `generation_bump_fences_old_records` establishes that old rows
fail authentication after a bump; they do not become absent or a clean baseline.

**Confirmed:** `CollectionState::from_saved` already validates collection name,
hash width/recomputed LT hash, wire-map dedup, logical-map cardinality/bijection
and exact logical-index shape. Only after that should the binder inspect
`has_logical_indexes`. Nonempty legacy maps with absent logical metadata remain
inspectable but unauthorized for incremental verification. An empty legacy map
normalizes to complete empty provenance while preserving its version/hash.
Missing checkpoint and valid empty checkpoint are different input states.

**Proposed:** a supplied saved baseline is an admission claim, not independently
proven store authentication. Name it `AdmittedBaseline`, record its generation
and immutable request identity, and document that the store wrapper is responsible
for loading it under the same transaction as reservation. A public pure function
cannot establish that a caller obtained bytes from an authenticated store.

## Small typed API boundary

**Proposed sketch, names are illustrative:**

```text
AdmittedBaseline { collection: Collection, saved: &SavedCollectionState }
Admission { generation: AccountGeneration, request_identity, baselines: &[AdmittedBaseline] }
bind_fetch(request: &Node, response: &Response, admission: &Admission,
           current_generation: AccountGeneration) -> Result<BoundFetch>

BoundFetch { generation, original_request, original_response, collections }
BoundCollection { collection, requested_version, validated_baseline, disposition }
Disposition = InlineCandidate(opaque_data)
            | NoAdvanceVersionAck(signed_observation)
            | NoAdvanceAlreadySynced
            | CollectionRefusal(original_error)
            | MissingResponse
            | UnsupportedData(original_data)
```

No `Verified`, `Applied` or `CompleteSync` label belongs in this output. The
separate header stage validates all selected bodies before any applier gets a
step; the crypto stage validates the state transition. A version acknowledgement
or empty category can remain a typed no-advance observation, with no synthetic
new hash/map. Manual Debug must redact originals, arbitrary names/error text,
keys and body/reference material just as the current classifier does.

`request_identity` must bind the immutable saved action request; a durable
fingerprint should use stored encoded bytes, not a fresh Node encoding. If the
pure API only accepts Nodes, it must not claim byte-identical fingerprint
validation that its input cannot perform. The future store wrapper owns that
additional check and persistence; a pure structural binder can still check ID,
namespace, destination, action-kind authority and exact collection/version intent.

## Binding invariants

**Proposed supported contract:**

- Verify the saved request is the intended patch-free app-state IQ, exact
  correlated ID and supported server destination. `FETCH_ACTION_KIND` alone
  cannot establish request grammar. Preserve unknown fields; unsupported request
  branches must not silently become ordinary fetch intent.
- Derive the requested known collection domain from the immutable request.
  Reject duplicate/unknown requested names as local supported-producer policy;
  do not match a prefix, case-fold, normalize or accept an alias. Admitted
  baselines must cover exactly that domain once, with matching saved collection
  identities. A response name does not enlarge the requested authority.
- Require explicit nonnegative signed-long-range request versions and exact
  equality to restored admitted held versions. A version-zero frozen request
  paired with a held-version-N baseline is a mismatch, regardless of whether
  response data looks sequential from N. Never rebase or rewrite its request.
- Missing/extra/duplicate admitted baselines and incomplete logical provenance
  fail binding. Normalization through `from_saved().to_saved()` may remove map
  ordering differences; it cannot fabricate logical provenance or replace a
  missing checkpoint. No held key or dirty version substitutes for a baseline.
- Preserve response wire order and originals, but bind each response collection
  to a unique requested owner. Unexpected/duplicate response ownership should
  fail the supported binder before application; that is local ambiguity policy,
  not a native F2T duplicate-collection rule. Missing responses get explicit
  unresolved dispositions, not inferred `AlreadySynced` results.
- Keep version acknowledgements and already-synced outcomes out of the
  inline-advance candidate list. A plausible version is not a prepared outgoing
  hash. Existing native accepted-write acknowledgements use EfB's original
  prepared mutations/hash, which pure fetches lack.
- An `InlineCandidate` still requires the independent full selected-data/header
  preflight for the complete response. Do not validate/commit one collection
  while a later selected patch has not even been decoded. External/snapshot/exit
  support and native fatal classification are owned by the separate header stage.
- Generation mismatch is a binding failure; an old epoch alone is not a reason
  to erase an uncertain result. Later application must recheck the actual held
  baseline inside its write transaction, because pure binding cannot prevent
  concurrent updates after it returns.

## Handwritten negative fixtures

These target the real binder, not a mirror of its implementation:

1. Request `version=0`, admitted regular version7, and structurally valid reply
   for version8: binding refuses and original request bytes stay unchanged.
2. Request regular, admission regular_low, response regular; or saved state whose
   internal collection disagrees with its baseline entry: neither domain binds.
3. Unrequested response name, requested/admitted duplicate name, unknown name,
   missing admitted baseline and extra baseline: no inline candidate escapes.
   Missing response remains explicitly unresolved without an invented state.
4. Same positive version but LT hash inconsistent with its map; valid nonempty legacy state
   with missing logical metadata; invalid duplicate wire MAC or wrong cardinality:
   restore/binding refuses. A different internally consistent map at the same
   version is not detectable as stale by this pure API without an authoritative
   comparison input: the later transaction must compare its full held state
   against the admitted baseline. Separately a valid empty legacy baseline preserves
   its positive version and complete empty provenance, rather than being reset.
5. Admission generation1 versus current transaction generation2 refuses. A real
   encrypted-store test reads the old row after `bump_generation` and verifies
   authentication error is propagated, never converted to missing/empty state.
6. Wrong ID/type/namespace/destination, patch-bearing request, omitted request
   version, negative/overflowed version and duplicate consumed singleton fields:
   no supported fetch authority is produced. Non-ASCII BMP digits accepted by
   the shared Java-decimal helper stay accepted where native numbers are parsed.
7. Pure-fetch version acknowledgements at held, held+1 and much higher versions,
   and an empty/already-synced category: typed no-advance output only, no generated
   hash or new baseline. Negative signed acknowledgements remain ineligible.
8. Ordered multi-collection inline candidates with a malformed final protobuf:
   binding may succeed structurally, but the separate all-data preflight must
   refuse before any application API is invoked. Test that boundary composition
   once both stages exist, without duplicating the header worker's fixtures.
9. Borrowed originals and body bytes are pointer-identical; all diagnostic Debug
   output omits private names, error text, reference keys, bodies and raw IDs.

No tests were run for this documentation-only review. No new role/recovery
predicate is proposed for the pure binder; the native scheduler admission and
bootstrap initialization gaps remain separate from structural authority checks.

## Review of the subsequent pure binder implementation

**Confirmed source during this change:** `binding.rs::FetchBaselinePlan` owns
its baseline vector and pins a SHA-256 digest of caller-supplied raw request
bytes, action ID and account generation. Its constructor explicitly describes
this as a claim, not proof of store origin. Binding uses the shared action
wire decoder and validates the supported patch-free request envelope, exact
requested/admitted/response domain, complete restored logical provenance and
explicit requested version equality. The original version-zero intent is never
rebased. Snapshot, refusal, missing, version-ack and already-synced dispositions
supply no state-advancement authority.

The first inspected draft trusted the public `Response` model's mutable summary
fields. That could associate an original empty result with a fabricated data
category/body. The refreshed implementation reclassifies `response.original`
and compares the entire model using derived `PartialEq/Eq` on Response,
CollectionReply, Category and CollectionData before binding dispositions. This
closes ordinary inconsistent-summary construction without claiming MAC/store
origin authentication; manual Debug remains redacted. An immutable bound borrow
also prevents later ordinary mutation of that same model while it is in use.

Focused additional cases for the implementation tests:

- Mutate the classified collection name, known-collection discriminator,
  category, patch body, snapshot reference, has-more, collection-original node
  or collection order independently. Each inconsistent model must refuse.
  Semantically equal borrowed copies need not have identical addresses.
- Change one encoded request byte after claim construction, and independently
  change current generation: both refuse before binding. Check the raw length
  bound before hashing at the binder boundary, not just in claim construction.
- For the version-zero mismatch fixture use an explicit version-zero request
  **without** return_snapshot; otherwise snapshot-request rejection can mask
  a broken version/baseline guard.
- Confirm a reverse-order response remains in `BoundFetch.response` in response
  order, even though `BoundFetch.collections` is documented in request order.
  A future per-patch applier must not silently iterate request order as native
  response order: use the preserved response traversal or an explicit response
  position. This is not a current overwrite bug because this stage does no I/O.

No source edits or tests were performed by this independent review. The root
implementation and tests own final validation; the separate header module owns
protobuf failure precedence. The binder still cannot prevent a concurrent held
state update after it returns, nor make a public constructor's baseline claim
authenticated store provenance.
