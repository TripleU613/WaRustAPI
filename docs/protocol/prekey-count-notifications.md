# Prekey count notifications

APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Confirmed static** findings below are from retained annotated DEX, not a live
execution of the APK. They do not transfer obfuscated names to other releases.

## Dispatch and numeric semantics

`14z.A09` selects child zero (`000e..0016`), then its tag. A first `count`
uses `0hs.A0H("value")` (`0038..0040`) and optionally selects a sibling
`pq_count` with `0hs.A05("value", 0)` (`0040..005c`). A first `pq_count`
uses that same defaulted strict parser (`009c..00a4`), independently of later
legacy children. Other first children, including `identity`, take other paths.
The presence of a later count alone does not authorize replenishment.

`0hs.A0H` returns null for an absent attribute or caught
`NumberFormatException`; present valid input goes through `Integer.valueOf`.
The existing [A05/A06 evidence](apk/2.26.37.71-message-recipient-count/attribute-int-optional.txt)
shows absence defaults to zero, whereas malformed present input throws. These
are signed 32-bit Java decimal counts, including supported BMP decimal digits
and signs, rather than unsigned ASCII counts. Malformed PQ parsing occurs
before acknowledgement metadata is constructed or work is queued.

The legacy branch queues `DMA` selector **3**. Constructor fields preserve
legacy and PQ separately; `DMA.run`'s decoded switch targets `0558`, and
`057a..0580` forwards an explicit **true** boolean to `14z.A02`, even when
the optional legacy integer is null. This flag represents legacy dispatch,
not numeric validity. The PQ-only branch queues `DLx` selector **32**. Its
[existing full run evidence](apk/2.26.37.71-status-observers/post-runnable-run.txt)
has explicit switch cases 0 through 31; selector 32 reaches the default block
`000e..0036`, forwarding null legacy and **false** at `002e..0034`.

`14z.A02` tests this flag at `0126` and `01ac` in the legacy non-migrated
path: true permits generation/upload (`0154..0190`, `01ce..01d4`); false takes
the PQ-only no-action branch. Full PQ migration strategy is **unknown/unported**.
The worker invokes acknowledgement at `023a..0244`, after its processing.
Rust's existing acknowledgement-before-upload ordering is unchanged; this
change does **not** establish native scheduling/ordering parity.

## Rust behavior and validation

**Implemented/offline tested:** `prekeys::low_count` returns
`Result<Option<LowCount>>`; `legacy_present` records first-child dispatch,
and `legacy`/`pq` are optional signed `i32`. Absent/malformed legacy values
remain optional; missing PQ values default to zero; malformed present PQ
refuses with a fixed diagnostic. Existing outer namespace/type/participant
guards are local restrictions; complete native outer dispatch is **unknown**.
Non-text legacy is optional and non-text PQ refuses, a local malformed-input
policy rather than a demonstrated native wire producer.

The actor preserves the encrypted raw inbound capture before parsing. It uses
the dispatch flag for legacy replenishment, rather than optional parse success.
Malformed PQ refuses before ACK/upload, without retrying. Existing upload
reservations, round bounds and identity/session guards remain in force.
Since 2026-10-13 that refusal is logged and receive continues; the capture stays
without ACK, so the server redelivers it at the next login (receive audit F3,
[replay-safety-audit](replay-safety-audit.md#receive-side-audit-2026-10-13)).
Before, it ended the session for good, and every later session on the same
redelivered notification. The native throw is in `14z.A09` dispatch, before
work is queued (above); which frame catches it, and whether the connection
survives, is **not traced**. Ending receive was our policy, not APK evidence;
continuing is also a local policy choice.
[Identity-first notification handling](identity-notifications.md) remains
**unported**; preventing
its mistaken count classification does not implement identity notification
validation, pin adoption, invalidation or acknowledgement.

The committed [public reference](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-count-reference.json)
contains 487 independently executed `Integer.valueOf` cases, generated with
[MexJavaNumericOracle.java](MexJavaNumericOracle.java), mode `integers`, on
Temurin Java 21.0.12.1+1. Each is tested in three count layouts (1,461 parser
projections). This is a host Java oracle, **not APK runtime observation**.
A fake-link actor test covers identity-first exclusion, missing-value PQ-only
ACK without upload, invalid optional legacy replenishment, strict PQ refusal,
and durable malformed capture/state preservation across reopen. Error text
is checked with a unique private-value canary, not arbitrary substring tests.

No real account was opened and no fresh live count variant was requested.
The preceding bounded live session's `encrypt` control classification does
not establish its first-child tag or prove this fix occurred live.

## Reproduce and compare

The [recipe](apk/recipes/2.26.37.71-prekey-count-notifications.json) retains five
topics / ten files / 73,903 bytes, including signatures, register flow and
exception tables. Bulk APK/decompiled media remains outside Git. Existing
A05/A06 and DLx evidence is reused instead of duplicated.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-prekey-count-notifications.json \
  --output docs/protocol/apk/2.26.37.71-prekey-count-notifications --check
cargo test -p whatsapp-api --all-features prekey_count
cargo test -p whatsapp-api --all-features prekeys_upload_at_start
```

Use the recipe with the pinned APK for export/check and `apk-research-diff` for
file/topic changes between retained release bundles. Cached self-diff proves
reproducibility only; it is not a comparison with a newly released APK.
Operators already have `listen` for bounded receiving and `prekeys` for offline
redacted upload/reservation inspection. No new HTTP route or CLI command is
introduced by this library correction.

## Upload reply-type correction (2026-11-04)

**Offline source correction, pinned 2.26.43.72 shared IQ boundary:**
prekeys::PendingUpload now matches only same-ID iq/result or iq/error.
Other types no longer become a refusal that returns reserved keys to
Unsent. The full uncertain batch and key reservation survive those non-reply inputs
and reopen until a valid matching reply; no automatic retry is added.
[Source finding, native dispatcher and regression](network-correctness-audit.md#prekey-upload-reply-type-admission-2026-11-04).
Older build-specific count/upload layout evidence above is not silently
re-attributed to the current APK. Sender and full current response matrix
remain open; no live behavior is verified.

## Current-build upload reply bodies (2026-11-04)

**Confirmed static:** 43.72 140.CK4 / 141.CK4 support optional first-list
count_low case-insensitive true handling; they also forward a separate PQ flag.
[Retained callbacks, first-child helper, error parser and limits](apk/2.26.43.72-prekey-upload-replies/README.md).
No runtime change or full upload scheduling parity is claimed.

## Current-build successful storage and failure-policy boundary

**Confirmed static, 43.72:** Fd9 marks legacy/Kyber uploaded before completion;
EcZ.A00 has gated 406/5xx scheduling branches. This supersedes the earlier
untraced downstream boundary without proving full retry or regeneration policy.
[Exports and pending helper/runnable trace](apk/2.26.43.72-prekey-upload-replies/README.md#store-effects-and-completion-policy-follow-up-2026-11-04).

## Current-build retry delay inputs

**Confirmed static, 43.72:** 13z.A02 bounds ordinary retry by a backoff counter
and limited mode by a decrementing allowance; 406/503 limited mode uses a
configured delay. [Configuration descriptor, helpers and unresolved producers](apk/2.26.43.72-prekey-upload-replies/README.md#retry-limits-and-delay-inputs-2026-11-04).
No Rust retry/scheduling behavior changed.

## Limited-mode producer boundary

**Confirmed static, 43.72:** allowance comes from config0x3741 in conditional
FDh.CbF scope, not unconditional startup. 14o.BLn resets/consumes that mode.
[Reachability and remaining listener/optional-provider gaps](apk/2.26.43.72-prekey-upload-replies/README.md#limited-mode-producer-boundary-2026-11-04).
No Rust scheduler change.

## Current-build success continuation construction gap

**Confirmed static, 43.72 / unported:** EcZ/Fcd continuation uses ADD upload,
configured legacy batch limit and distinct 141 callback ownership. Rust still
uses older SET/812 follow-up construction. [Evidence and remaining porting inputs](apk/2.26.43.72-prekey-upload-replies/README.md#successful-continuation-selects-add-2026-11-04).

## Current-build ADD selection and fields

**Confirmed static, 43.72:** ADD uses unsent/non-direct SQL LIMIT and op plus
optional legacy/PQ lists, without repeating SET account/signature setup children.
[Exact layouts and remaining generation/config/persistence work](apk/2.26.43.72-prekey-upload-replies/README.md#add-selection-and-exact-child-layout-2026-11-04).
No runtime switch yet.

## Current-build generation gap

**Confirmed static, 43.72:** legacy ensure fills the 812 deficit in chunks up
to50 rather than returning after one chunk. Native cursor expressions differ
from the local checkpoint convention. [Control flow and migration boundary](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-generation-loop-and-cursor-convention-2026-11-04).
Older 37.71 text above is not evidence of current-build generation parity.

## Current-build cursor ownership

**Confirmed static, 43.72:** self identity tuple owns native cursor; missing row
throws, creation seeds1..0xfffffe separately for legacy/PQ. This is not a reset
policy. [Reader/writer and creation evidence](apk/2.26.43.72-prekey-upload-replies/README.md#native-cursor-ownership-and-creation-seed-2026-11-04).
Existing local zero-based cursor remains unchanged pending versioned compatibility.

## Current-build ADD ordinary PQ

**Confirmed static, 43.72:** ADD selects non-last-resort ordinary PQ keys;
generation fills an injected target with loader gating,<=50 chunks and separate
cursor. [Evidence and remaining target/signature/admission trace](apk/2.26.43.72-prekey-upload-replies/README.md#add-ordinary-pq-selection-and-generation-dispatch-2026-11-04).
No runtime switch.

## Current-build PQ signing input

**Confirmed static, 43.72:** EqE signs the public byte array later uploaded;
0Yj.A00 does not strip its first byte. Enum A02 resolves to ML_KEM_1024.
[Java byte provenance, Rust comparison and native-body limits](apk/2.26.43.72-prekey-upload-replies/README.md#pq-signing-and-wire-byte-provenance-2026-11-04).
No prefix/algorithm patch is inferred from enum constructor codes.

## Current-build PQ inventory versus batch

**Confirmed static, 43.72:** generation target property21200 defaults200;
upload batch property21201 defaults10. The provider captures the former in
0Yj; runtime account values were not read. [Source/default proof and Rust gap](apk/2.26.43.72-prekey-upload-replies/README.md#pq-target-provider-and-declared-defaults-2026-11-04).

## Construction rollback correction (2026-11-04)

**Confirmed local correction:** identity validation/KEM child signing now happens
inside upload reservation, so construction failure cannot commit uncertain keys
or newly generated KEM state. [PQ-off/on reopen regression and scope](network-correctness-audit.md#prekey-construction-and-reservation-atomicity-2026-11-04).
Current-build ADD port remains unfinished.

## Current-build manager entry admission

**Confirmed static, 43.72:** low-count13z.A0V requires migrated state before
ADD; migration predicate requires21198 and sent last-resort PQ row. A0R is
separate PQ-only SET scope. [Flags, shared guard and upstream gaps](apk/2.26.43.72-prekey-upload-replies/README.md#setadd-entry-admission-and-migrated-state-2026-11-04).
No Rust account migration or scheduling switch.

## Frozen request provenance (2026-11-04)

**Implemented offline:** newly reserved upload node bytes/mode/dictionary are
retained internally under store encryption before returning the node. Older
batches receive no backfill and uncertain requests remain unreplayed.
[Reopen/settlement regression and exact scope](network-correctness-audit.md#frozen-prekey-request-provenance-2026-11-04).

## Uncertain KEM selection correction (2026-11-04)

**Confirmed local correction:** PQ-enabled uploads remain held while a prior
KEM-bearing Batch is outcome_unknown; sent=false no longer permits selecting
its keys under a fresh legacy upload after reconnect. [Reopen regression and scope](network-correctness-audit.md#uncertain-kem-batch-replay-admission-2026-11-04).

## KEM settlement integrity (2026-11-04)

**Confirmed local correction:** acknowledged batch settlement requires each
reserved KEM row to exist and validate; missing material rolls back complete
settlement. [Partial-write/reopen regression and native-policy distinction](network-correctness-audit.md#kem-acknowledgement-inventory-integrity-2026-11-04).
No key regeneration or uncertain-state reset.

## Consumption before upload ACK (2026-11-04)

**Local correction:** ordinary KEM consumption now leaves a nonsecret atomic
marker, allowing later upload settlement without resurrecting its key. Unexplained
loss still refuses; stale allocators cannot reuse consumed IDs.
[Ordering/rollback/reopen regressions and prior-guard qualification](network-correctness-audit.md#consumed-kem-key-before-upload-ack-2026-11-04).

## Current-build routing and ACK decision order

**Confirmed static, 43.72:** 15v.A02 routes migrated ADD versus non-migrated
legacy SET, passing count presence rather than threshold. Worker ACK follows
upload decision; asynchronous wire order is not established.
[Notification/daily sources and remaining porting scope](apk/2.26.43.72-prekey-upload-replies/README.md#notification-and-daily-pq-entry-routing-2026-11-04).

## Current-build legacy SET handoff

**Confirmed static,43.72:** A0S/A03/Fdv snapshots selected keys then hands
EXb through message85, distinct from modern callback-based SET/ADD.
[Conditions, reason argument and remaining final-writer scope](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-set-snapshot-and-message85-handoff-2026-11-04).
No runtime port from this trace alone.

## Current-build legacy writer/result path

**Confirmed static,43.72:** arg1=85 fullSET writer emits the broad existing
fields with conditional op, while290/1ru forwards result count_low via success
event arg1=77. [Writer and separate reply-map scope](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writer-shape-and-success-event-path-2026-11-04).
The prior what85 prose was corrected to what0,arg1=85.

## Current-build legacy error event

**Confirmed static,43.72:** 1rA removes matching legacy entry for exact result/
error;290 error uses coded-error extractor then emits arg1=78 with code/text.
[Special route versus generic callback and map/listener limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-reply-removal-and-error-event-path-2026-11-04).
No retry/reset port yet.

## Legacy reply-map reference binding

**Confirmed static,43.72:** normal transport setup shares one map through writer
and reader factory context. [Alias proof and remaining factory/lifecycle scope](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writerreader-map-alias-2026-11-04).
This is in-memory binding, not durable replay/account-state policy.

## 2026-11-04 — Legacy IQ registration boundary

**Confirmed static (43.72):** inherited 1rA.A02 maps 1rD to `iq`;
0ZJ.A19 rejects duplicate tag registration. Exact 1rC constructor search
finds no direct invoke callers, which does not establish factory absence.
Configured factory membership and actual dispatch remain unknown.
[Exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-iq-handler-registration-boundary-2026-11-04). No production change or live traffic.

## 2026-11-04 — Concrete legacy listener handoff

**Confirmed static (43.72):** normal setup binds1lZ.A01 to1lY;
CPm uses arg1 and conditionally copies events to0ln/0m2 or queues on1lY.
A0Q-enabled77/78 use0ln; subsequent14o manager binding remains unknown.
[Exports, branch boundaries and interface-search limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-concrete-listener-handoff-2026-11-04).
No production change or live traffic.

## 2026-11-04 — Legacy enabled-route manager binding

**Confirmed static, conditional (43.72):**0ln.A00 routes77/78 through
SQ0.A0s provider6735, whose application-scope factory constructs14o.
This closes the A0Q-enabled route to retained14o.BLn upload branches.
Alternate configuration/factory membership remain unproven.
[Exports and conditions](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-enabled-route-manager-binding-2026-11-04). No production change or live traffic.

## 2026-11-04 — Legacy fallback event route

**Confirmed static, conditional (43.72):** selected cold-start fallback
0ZJ.A0q ->0YU.CPn/what5 returns without handling77/78. This does not
prove these events take fallback in normal configuration. Enabled route
remains bound to14o; lifecycle guards remain unverified.
[Exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-fallback-event-route-2026-11-04). No production change or live traffic.

## 2026-11-04 — Configured legacy IQ factory membership

**Confirmed static (43.72):** A1H set7466 includes binding126, which
allocates1rC through08o.A0Q with Object constructor inlining. Membership
is now proven; earlier zero direct constructor callers did not cover it.
A0Q comes from property0x4c7a at construction; runtime value unknown.
[Exports and remaining lifecycle scope](apk/2.26.43.72-prekey-upload-replies/README.md#configured-legacy-iq-factory-membership-2026-11-04). No production change or live traffic.

## 2026-11-04 — Disabled-flag registry route

**Confirmed static after initialization (43.72):** set7469 includes14o
(binding6735), Amf declares77/78, and0ZJ.onLooperPrepared calls0ln.A05
to register them. Disabled flag does not itself imply callback loss.
Fallback disposition remains true if reached; early-event/lifecycle unknowns
remain. [Exports and conditions](apk/2.26.43.72-prekey-upload-replies/README.md#disabled-flag-registry-route-2026-11-04). No production change or live traffic.

## 2026-11-04 — Upload reply fields separated

**Implemented offline,43.72 callback evidence:** successful prekey replies
ignore error metadata; refusals ignore count_low flags. Synthetic mixed-body
reply regression verifies outcomes and persisted batches after reopen.
[Evidence, test and remaining gaps](apk/2.26.43.72-prekey-upload-replies/README.md#rust-reply-field-separation-2026-11-04). No live traffic.

## 2026-11-04 — First coded upload error and numeric conversion

**Confirmed static / implemented offline (43.72):**1ud.A01/0HF.A00
select first present code, parse signed Java integer, default0. Rust now
persists canonical decimal without eight-character truncation; historical
batches retained. Reopen regression covers malformed-first and limits.
[Evidence and remaining scope](apk/2.26.43.72-prekey-upload-replies/README.md#rust-coded-error-selection-and-numeric-conversion-2026-11-04). No automatic retry or live traffic.

## Upload settlement reservation ownership (2026-11-04)

**Implemented offline, local persistence invariant.** PendingUpload::settle
previously required unresolved status but used its in-memory legacy IDs without
comparing the persisted batch or current per-key reservation metadata. A changed
batch/owner could therefore be overwritten by an otherwise correlated reply.
No native/server requirement or observed live occurrence is asserted.

Settlement now verifies exact ordered batch IDs and, for each ID, existing
OutcomeUnknown metadata owned by the same IQ. The checks run in the same write
transaction before batch status, KEM sent flags or legacy metadata can change.
A mismatch returns a specific diagnostic and leaves the transaction unchanged.
Private key row presence is deliberately not required: authenticated inbox
consumption can precede an ACK. No consumed material is regenerated/resurrected.

Regression `upload_settlement_rechecks_durable_key_ownership_without_partial_updates`
changes ordered batch IDs, removes the last key's metadata, changes its status
with owner retained, or changes owner with status retained. Both result/error
settlements refuse and all checkpoint bytes remain unchanged. The ownership
refusal survives reopen. Existing normal settlement and consumed-KEM-before-ACK
regressions remain required. Historical data is not repaired automatically;
no retries, live traffic or complete native lifecycle parity are claimed.

## KEM upload reservation binding (2026-11-04)

**Implemented offline, local persistence invariant.** The prior ownership
check bound legacy IDs/metadata, but PendingUpload did not retain KEM IDs.
Settlement trusted the saved Batch.kem_ids, so an independently changed list
could mark different KEM rows sent or omit original rows while acknowledging
the legacy batch. No live occurrence/native vulnerability is asserted.

The same reservation transaction now returns its ordered KEM IDs alongside
legacy IDs and frozen request. PendingUpload retains these IDs privately.
Settlement compares the entire durable KEM list before any writes on either
result or refusal. Missing/reordered/substituted IDs refuse atomically.
Empty legacy-only lists compare too; no saved schema or wire change is needed.
This remains callback correlation, not uncertain-request replay authority.

The existing uncertain-KEM/reopen/consumption regression now replaces the
saved list with empty, reversed and unrelated-ID variants. Both reply kinds
refuse with all Prekeys/Checkpoints bytes unchanged. Restoring the synthetic
fixture allows the original missing-row/consumption/late-ACK checks to run,
including no resurrection and the next upload's normal behavior. Production
never repairs an uncertain batch to pass this guard. No new retry or live traffic.

## Independent PQ count-low retention (2026-11-04)

**Confirmed static / implemented offline.** Pinned43.72 140/141.CK4 and
1ru.A02 read first exact list/pq_list independently, case-insensitive true,
absent=false (checksummed prekey-upload-replies bundle). Rust now retains
pq_count_low separately in UploadOutcome and durable Batch. Result parses
the flag; error storesfalse. Historical batch field defaultsNone, not a
backfilled observation. Original upload key ownership checks still apply.

`prekeys` text/JSON and existing read-only GET /v1/prekeys summary expose
pq_low_acknowledged_batches: a historical count of acknowledged batches
with saved PQ flagtrue. It is not current demand or permission to upload.
No account/key/request IDs or contents are exposed. Previously misleading
CLI/schema no-KEM-producer wording is corrected.

Regression uses PQ-only demand with mixed case, duplicate children (first
wins), an error carryingtrue and reopen; legacy flag staysfalse and the
historical counter is not incremented by refusals/false. Missing historical
field deserializesNone. Existing actor still schedules followup from legacy
count_low only: PQ-only automatic replenishment requires the incomplete
current SET/ADD routing strategy. No such port or live behavior is claimed.

## Durable refusal upload hold (2026-11-04)

**Confirmed source mismatch / implemented offline local policy.** A refused
upload returned legacy metadata toUnsent; wants_upload and begin_upload
then permitted a new IQ on reconnect or another count notification. Earlier
"does not retry" statements covered only the immediate error callback and
were too broad. Reconnect was implicitly retrying without the native retry
policy. Pinned43.72 13z.A02/14o.BLn evidence describes conditional backoff,
406 regeneration and other code/gate-specific handling; that complete policy
is still unported, so no correct delay or retry is invented here.

New reservations now check for any persisted refused Batch in their write
transaction and returnNone with a fixed diagnostic. wants_upload also returns
false for startup. Keys remain intact/unsent; this status is inventory rather
than retry authority. Historical refusals also hold. The existing prekey
CLI/JSON/HTTP summary exposes upload_held_after_refusal and text explains it.
There is no reset/delete/resume operation or inferred cooldown.

This is an explicit local hold while retry strategy is incomplete, **not
native behavior parity or completion**. Porting the native strategy must
replace this hold with evidence-backed admission; do not clear refusal
records merely to regain uploads.

Regression covers406/503/default0 refusals, additional freshly generated
legacy material, repeated begin attempts and reopen. No new batch/checkpoint
write occurs. Existing acknowledged-upload and malformed-reply tests remain.
No live traffic or account state was modified.

## 2026-11-04 — Legacy refusal workers/reset boundary

**Confirmed static (43.72):**legacy406 Fea9 callsA05/A0m(reason9)
thenA0S(reason7), distinct from modernFea13/A06/A0R. A0m records identity
reset reason and enters cleanup whose full effects remain unknown.
Legacy5xx Fea10/11 ->Fea8 rechecks backoff/admission beforeA03.
[Exports and incomplete port scope](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-refusal-retry-workers-and-reset-boundary-2026-11-04). No production change or live traffic.

## 2026-11-04 — SQLite reset base/helper selection

**Confirmed static (43.72):**base helper attempts database deletion and
checks sidecar absence;Eoe clears three collections. Helper flag0x646a
selects0Xv(axolotl.db) or28o path. Full virtual/native cleanup and rebuild
semantics remain unproven. [Exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#sqlite-reset-base-implementation-and-helper-selection-2026-11-04).
No production change, account reset or live traffic.
