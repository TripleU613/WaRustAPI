# App-state fetch result completion

Static audit, 2026-11-04. **Confirmed native** findings use WhatsApp Android
2.26.43.72, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Complete new envelopes and checked bytecode](rea/2.26.43.72-sync-fetch-completion/README.md)
extend [the lifecycle](server-sync-lifecycle.md) and
[connection flight design](app-state-connection-flight-design.md). No APK
execution, network request or account operation occurred. The structural-only
classifier described below is now implemented; authenticated fetch completion
remains unported and no remotely controlled feature mode is selected.

## Response grammar precedes completion

**Confirmed:** `FUc.CK4` constructs `F2T` before dispatching `FeN.run` case40.
`F2T.<init>(0Tp)` requires a `sync` child and partitions each collection:

| Collection shape | Native parsed category |
| --- | --- |
| `type=error` | `Ebo` error with mandatory error child, integer code (default `-1` when absent) and optional text. A 409 with nonempty patches parses its accompanying data; otherwise its `EUl` has an empty patch list. |
| Nonerror with `version` attribute | `EQq` version acknowledgement. Missing/invalid parsed value `-1` becomes a server-transient parse error. It is not incoming patch data. |
| No version, no snapshot and no nonempty patches child | Name in already-synced list. |
| Remaining snapshot/patch collection | `EUl`: name, optional external snapshot reference, ordered patches and `has_more_patches`. Only literal `true` sets the last boolean. |

`F2T.A01` requires a name. `A02` refuses an empty snapshot/patch body. `A00`
parses a snapshot body as `Dg5` (ExternalBlobReference), **not** a `Dem`
(SyncdSnapshot). Patch bodies are `Dh0` (SyncdPatch). It rejects a patch with
an exit-code field (specific fatal reasons for exit100/101/other), missing
version, duplicate patch versions within that collection, or external mutation
reference plus nonempty inline mutations. Snapshot protobuf parse failure maps
to fatal69, patch protobuf failure18; structural patch failures use their own
reason codes. Checked `F2T` DEX corroborates the recovered loop whose Java
rendering otherwise looks as though it always throws after version handling.

**Boundary:** this parser does not itself bind all collection names to the
original request, reject duplicate collection names globally, or prove that
all requested names occur exactly once. Those are port admission/response
correlation obligations, not claims about this parser. Preserve input order
and exact original request bytes; do not rebuild a frozen outgoing request.

## External data authentication has two layers

**Confirmed chain:** `SyncResponseHandler.A04` calls `0lN.A02` for snapshot
references and `0lN.A01` for external mutations. Both construct `Dxd`, run its
download job, then parse the authenticated output file as respectively `Dem`
or `DXE` (SyncdMutations). The latter's mutation list replaces the external
reference for subsequent record/MAC verification; it is not silently omitted.

`Dxd.<init>` requires presence of media key, direct path, plaintext SHA-256 and
encrypted SHA-256 in `Dg5` (bits1,2,16,32). `Dxd.A05` builds the MMS download
request with `md-app-state`, `mms`, encrypted hash and direct path, and carries
media key/plaintext hash to the encrypted transfer. Presence checks here do
not establish exact byte-length or URL policy validation elsewhere.

The exact crypto route is `Dxd -> LIp -> N2E.A04 -> MuB -> MuI.CQc`:

- `0jS.<clinit>` binds `A0R`/`md-app-state` to bytes of
  `WhatsApp App State Keys`; its constructor stores these in `A03`.
- `Mta.AMt` calls `1oO.A00(mediaKey, domain, 80)`. `1oP.A00` extracts with
  HMAC-SHA256 and 32 zero salt bytes; `1oO.A01` expands using previous block,
  domain and a one-byte counter. This establishes HKDF-SHA256, output80.
- `MTj.A01` splits IV `[0,16)`, AES key `[16,48)`, MAC key `[48,80)`.
  `LP2.<init>` initializes AES/CBC/PKCS5Padding decryption and HMAC-SHA256
  already updated with the IV. `LP2.write(byte[],off,len)` MACs ciphertext,
  separates the final10 bytes, compares the tag with the first10 HMAC bytes,
  and only on agreement finalizes padding and marks completion.
- `MuI` wraps encrypted input and decrypted output in distinct SHA-256
  digest streams. `MuB.A00`/`MXA.A00` check expected hashes and successful
  authenticated-stream completion. Plaintext produced incrementally is not
  authoritative before those checks. `LP2.close` itself merely closes output;
  it is not the authentication decision.

The `Dxd` path supplies the encrypted hash to URL construction; the inspected
`N2E.A04` constructs its `LzD` with no additional expected encrypted hash in
`A06`, while retaining the plaintext hash and authenticated decryption. This
means both digest streams exist, but **do not claim the Dxd path directly
compares both hashes in MuB.A00**: its encrypted-digest comparison is guarded
by a nonnull `LzD.A06`. Full encrypted-download URL/hash enforcement in
`JAd/N41` remains untraced. An eventual Rust implementation should retain the
expected encrypted hash and verify it explicitly; that is a local defensive
policy unless the remaining native path is closed.

Do not substitute image/audio/document key domains. Current Rust image media
helpers use those separate domains; no app-state external downloader exists.
The `CollectionPatch` decoder explicitly refuses external refs (field3) and
exit-code handling (field7). A raw local snapshot reconstruction operation is
not this download pipeline and cannot authorize primary snapshot installation.

## Download and processing errors

**Confirmed:** `0lN.A00` accepts only successful `MXt`. Failed download status5
becomes expired snapshot/external patch (`Dn6`); the callback maps these to
fatal58/4. A transient-status predicate becomes `Dn9`, with server-side true
only for status16 on that branch; remaining non5 failures also become
server-side `Dn9`. The exact membership of `MXt.A04` remains untraced here.
Invalid downloaded protobuf becomes `Dn8`, mapped fatal70 for snapshot or26
for mutations. Local file I/O becomes `Dn4`, then `Due`.

`SyncResponseHandler.A09` dispatches:

| Exception | Action |
| --- | --- |
| `Duh` | Fatal `A08(errorCode,collection)`, entering the already-traced role/recovery path. |
| `Due` | `A02` failure; reason `mms_local_io`, `mms_upload` or `syncd_failed` according to its cause. |
| `Dug` | `A01(backoffHelper.A01(), server_error/network_error, isServerTransient, !isServerTransient)`. |
| SQLiteFullException | `A01(delay,sqlite_full,false,true)`. |
| Other exception | `A01(delay,unexpected_error,true,false)`. |

Checked `A01`, not its warning-laden recovered Java, proves: server-transient
failures track the first wall-clock failure; more than604800000ms routes fatal.
For nonserver failures, its second boolean gates retry eligibility. A nonnull
computed delay releases the live flag without immediate rescan, handles deferred
recovery if present, otherwise schedules the manager after delay. Ineligible or
exhausted retries use `A02`, which resets the helper and releases without rescan.
These are native transitions, not permission to requeue an uncertain Rust write.
Constructor-selected retry limits remain untraced; no new retry policy is ported.

`FeN.run` case40 additionally classifies collection400/404 as fatal62/63;
409 applies accompanying verified patches and no-patch409 becomes server
transient. Other codes follow its retry/error path. A collection refusal is not
necessarily an outer IQ refusal, and no per-collection backoff attribute was
established by this audit.

## Commit gates and release scope

**Confirmed:** the classic result worker detaches `FUc`'s job context (`EfB`)
and uses its original prepared mutation map and resulting hashes. The version
ack path resolves the original collection's pending mutations; inside a DB
transaction it removes the acknowledged pending rows (`0g1.A0P`) and stores the
returned version with the job's prepared hash (`0l8.A08`). It does not infer an
accepted hash from an unsolicited version advertisement.

For incoming data, `A07` holds the mutation manager lock and opens a transaction,
reads the current stored collection version **inside** it, and requires an
incremental patch to equal current+1. Snapshot application instead requires
companion mode and held version zero at `A04`. It resolves both record key IDs
and patch/snapshot signing IDs. A missing key can produce a deferred result;
feature-dependent mismatch behavior must not be copied as relaxed Rust MAC
verification. Authenticated mutation/hash/version commit through `0mj.A00`
occurs before transaction success. Transactions are per applied snapshot/patch,
not one all-or-nothing transaction for the complete multi-collection IQ.

When no more patches remain, `A04` clears the forced-zero dirty sentinel through
`0l8.A06`; positive dirty versions remain compared with applied versions.
A missing-key/deferred result is not proof that held state advanced, even when
outer processing returns. The worker reaches `0aS.A06(true)` only after its
response categories have been processed, then resets transient-failure/backoff
state. The manager rescan can find more dirty work; `has_more_patches=true`
therefore does not require holding the same network flight indefinitely.

**Not established:** a native integer per-collection generation token matching
Rust account generation. The confirmed authority is job context plus current
transactional version/key/role state and retained dirty metadata. Rust account
generation/epoch tokens and baseline compare are a proposed port correspondence,
not a recovered native field.

## Partial port plan and remaining obligations

**Proposed sequence:**

1. Add a pure bounded response classifier preserving all four categories and
   original collection ordering, with explicit unresolved external/exit-code
   outcomes. Bind processing to the durable original IQ ID/requested collection
   set and account generation. No network operation or request rewriting.
2. Add durable per-collection processing outcomes/cursors, storing original
   reply receipt order and exact held baseline/version/hash/logical provenance.
   On restart, continue local processing only. Successful earlier per-collection
   commits must be recognized before processing the next collection or page;
   do not replay mutations or roll them back because a later collection failed.
3. Implement bounded inline-patch result application using already-held keys
   and the existing verifier, guarded by original fetch intent, same collection
   authority and current version. Empty/already-synced results may settle a
   request without inventing held version/hash. Preserve unresolved work and
   dirty hints; no generic success marker clears an uncertain pending write.
4. Separately port the external reference/downloader and bootstrap role gates,
   authenticated download fixtures, snapshot installation, missing-key recovery,
   collection errors and continuation scheduling. The descriptor/crypto trace
   above is sufficient to design local authenticated-blob tests; it does not
   finish host selection, transfer retry policy or role initialization.
5. Release an owner token only after durable classification/commit or explicit
   deferred/failure outcome for every affected collection. Recheck newer dirty
   hints and account/epoch authority before scheduling subsequent work. Local
   completed/deferred state must be distinguishable from generic `Ok`/`Error`.

**Confirmed Rust gap at audit start:** fetch successes have no pending baseline,
so `ingest_action` does not install their data. The root correction in `dbc7087`
adds receipt-anchored outer-error fetch backoff and captured-refusal recovery;
that bounded refusal handling is separate from successful fetch data processing,
inner collection errors, downloads, pagination, retries and flight ownership.
No implementation should claim those gaps closed by observing a saved IQ reply.

Feature18234 selects a different outgoing processor and another feature gates
classic/KMP incoming handling; account values are unknown. This plan targets
explicitly supported classic semantics without asserting those features are
universally disabled. Existing frozen IQs remain byte-identical.

The initial research phase made no source changes. The structural implementation
and its tests are described below; authenticated completion remains unported.
Next useful tests: independently formed external descriptor/blob with wrong
ciphertext/hash/tag/padding, missing/duplicate versions and mixed data sources;
multiple collections where first commits and second fails; repeated local
settlement after crash; missing key without advancement; new dirty hint during
processing; stale baseline/epoch/generation; and late replies after release.


## Structural classifier API

**Implemented structural-only stage:** a separate
`transport/app_state_sync/response.rs` with `classify_response(&Node) ->
Result<Response<'_>>`. It returns borrowed original IQ/sync nodes and an ordered
vector of collection summaries. Each summary retains its original node, exact
name, optional known `Collection`, and one typed category:

- `Error { code: i32, text: Option<&str>, conflict_data: Option<CollectionData> }`.
- `VersionAck { version: i64 }`.
- `AlreadySynced`.
- `Data(CollectionData { snapshot_ref: Option<&[u8]>, patches: Vec<&[u8]>, has_more_patches: bool })`.

Original nodes preserve unknown attributes/children and unknown collection
names without claiming support; raw snapshot/patch bodies remain opaque. No
Debug/inspection surface should print their contents. Protobuf/header validity,
cryptographic authentication, requested-name binding and state eligibility are
separate stages. This initial classifier must not claim to reproduce all of
`F2T.A00` until those protobuf branch checks are implemented.

Exact precedence and corner cases to test independently with handwritten Nodes:

1. Error wins over a version attribute or snapshot. Missing error child fails;
   missing code yields `-1` (`0Tp.A05` is now retained); malformed code fails.
2. Nonerror present version wins over payload children. Parse a signed Java-long
   counterpart and reject sentinel `-1`; other negative values may be preserved
   in the pure category for a later state validator to refuse. No unsigned cast.
3. No version, no snapshot and absent patches or an entirely empty patches child
   gives `AlreadySynced`. Counting all patches-node children here is distinct
   from selecting only children named `patch` for opaque patch bodies.
4. A patches child with only unknown children is nonempty for category selection,
   but yields an empty selected-patch list. Retain those children; do not silently
   reinterpret this case as the native empty-response category.
5. Error409 parses its accompanying data only when the patches node has at least
   one child of any name. A snapshot without such patches does not enter that
   branch; it remains in the original error node. Other errors do not parse data.
6. Only literal `true` enables has-more. Preserve other spellings/opaque fields
   without normalizing them into true. A present snapshot/patch needs nonempty
   bytes if it enters data parsing, per `F2T.A02`.

Local bounds and ambiguous singleton refusal (IQ/sync, patches, snapshot or
attributes) should be explicit local parser policy. Do not attribute global
collection uniqueness or requested-collection completeness to `F2T`; the
inspected constructor establishes neither. Repeated collection names can remain
ordered opaque summaries for later request-bound admission to reject.

## Implemented structural stage and inspector

`response::classify_response` now implements the borrowed model above. It keeps
unknown names/children and repeated collections in wire order; snapshots remain
opaque external references. It reuses the existing UTF-16-aware Java decimal
helper, including BMP digit behavior. Debug output exposes only fixed category
tags, counts and known/unknown flags. Nonempty Bytes and Text bodies are borrowed
as their exact bytes; no protobuf decoding or body normalization occurs.

The local policy bounds are 4,096 collection/selected-patch items, 16 MiB of
consumed body bytes, 64 attributes per consumed node and 4,096 bytes per consumed
text attribute. Ambiguous consumed singleton nodes/attributes refuse. These are
local guards rather than newly attributed native protocol restrictions.

Offline `app-state-response-inspect ACTION_ID` opens an existing store and checks
that the saved fetch/patch request and reply carry the selected action ID and
app-state namespace. It classifies the preserved result in a single read snapshot.
JSON/text reports include static known names (null for unknown), category,
version acknowledgement/error code, error-text presence, reference presence,
patch count and continuation flag. Raw names, error text, payloads and reference
keys are never printed. `integrity_verified` is always false. It does not check
request/collection admission, protobuf semantic rules, keys/MACs, freshness or
application eligibility; it performs no download, state write or network action.

Ten handwritten classifier regressions cover the listed precedence, sentinel,
opaque-data, numeric, ambiguity/resource and Debug boundaries. A real encrypted
store inspector regression checks JSON/Debug redaction, category order and
unchanged checkpoints/journal. This is a prerequisite to authenticated processing,
not evidence that a server result was applied or that an account is safe.
