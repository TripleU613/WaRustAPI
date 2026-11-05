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

## 2026-11-04 — Last-resort producer and timestamp

**Confirmed static (43.72):**A0e reuses newest unsentLR or generates at
rawKEM cursor and advancespositive cursor. Storage writes upload_timestamp
while sent=false, so timestamp is not migration/ACK proof. Current Rust
SET helper differs; entry-driven ADD/rotation port remains incomplete.
[Exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#last-resort-producer-and-pre-upload-timestamp-2026-11-04). No production change or live traffic.

## Refusal hold actor verification (2026-11-04)

**Verified synthetic actor, not live.** New connection regression
`refused_prekey_upload_keeps_count_acks_without_retry_after_reopen` drives
the actual public-client actor through a fake NodeLink. Activation emits one
reserved50-key SET; a matching503 refusal settles it. Two subsequent zero
legacy-count notifications each produce only their matching ACK, and the
actor stops cleanly. The encrypted synthetic store is reopened without
reinstalling account/key material. A second activation emits no upload;
another count notification receives its ACK without a replacement request.

The test checks that only one refused batch remains, no uncertain/uploaded
keys are invented,50 keys remainunsent, and all private prekey row bytes are
unchanged through both actor epochs. Bounded no-write windows cover immediate
output, not all future elapsed time; durable begin_upload admission is
separately tested. The hold must not suppress ACKs or end normal receiving.

This broadens prior helper-level refusal verification to actor wiring. It
does not port native retry scheduling, prove server interoperability or
claim an actual account was tested. No production behavior changed here.

## 2026-11-04 — Legacy success reaches migrated ADD admission

**Confirmed static (43.72):**14o77 ->Fcj3 marks both families, clears
backoff, then either demandflag ->Fc8(selector7) ->A0V. Thus existing
migration/admission/ADD evidence applies to legacy SET success too.
Rust unconditional legacy SET followup remains a confirmed gap.
[Exports and port requirements](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-success-to-migrated-add-admission-2026-11-04). No production change or live traffic.

## Migrated success followup ADD (2026-11-04)

**Implemented offline from confirmed43.72 entry/writer evidence.** Legacy
success14o77 ->Fcj3 marks keys sent, thenFc8(7) ->A0V uses both demand
flags. A0V requires gate21198 and an uploadedLR, then selects ADD. See
prekey-upload-replies README legacy-success-to-migrated-add-admission and
ADD writer/selection sections.

Rust's success actor now passes legacy/PQ flags independently to a dedicated
reservation path; non-migrated/gate-off demand does not trigger another SET.
Migrated ADD children are exactly op(mode=add), optional legacylist, optional
ordinarypq_list. No identity/registration/type/skey/LR child is emitted.
Initial startup and notification paths retain their existing SET behavior;
this is an entry-specific port, not a claim that all routing is complete.

Requested legacy inventory fills toward812 in <=50-key chunks inside the
reservation transaction, then selects up to200. Requested ordinary KEM
inventory fills toward200 and selects up to10, excluding LR. These are the
retained declared defaults, **not observed/effective MobileConfig values**.
Existing local legacy/KEM cursor conventions, collision/consumed-ID refusal
and stored key bytes are preserved; no reinterpretation or reseeding.
Native cursor migration, configuration application and direct-distribution
metadata parity remain open. ADD still uses existing registered-material
readiness validation, which is stricter than the native ADD wire requirements.

Generation, identity/signing checks, reservations, batch IDs and exact encoded
wire/mode=add commit together. Existing mode=set checkpoints are untouched.
OutcomeUnknown KEM batches and saved refusals still hold new requests.
Result/error settlement uses the same bound key lists and transactional
ownership checks; no uncertain replay or refusal reset is introduced.
The local per-session round cap remains.

Regression covers all three demand combinations, gate-only/non-migrated
refusal, legacy812/200 and KEM200/10 selection, exact omitted children,
construction-failure rollback, encoded wire/mode freeze, reopen, original SET
checkpoint preservation and uncertain KEM hold. A synthetic actor confirms
PQ-only result demand emits op+pq_list ADD and settles it. Prior actor test
now verifies non-migrated success does not unconditionally send another SET;
subsequent independent count notifications keep their existing path.

**Remaining:** notification/startup current routing, effective config values,
full retry/backoff/reset strategy and native cursor/storage migration. No live
account operation or complete native replenishment parity is claimed.

## Count notification SET/ADD routing (2026-11-04)

**Implemented offline from43.72 evidence, declared flagdefault branch.**
Retained15v.A02 distinguishes migrated ADD/non-migrated SET. With EMv.A00
(0x70d7) false, migrated legacy presence invokesA0V(true,false); PQ-only
notification skips upload. The descriptor declaresfalse; effective MobileConfig
application remains unported, so this is **not** a live/effective flag assertion.

The actor now keeps explicit Startup/Success/Count triggers. Count routes
through begin_notification_upload: within reservation transaction, gate21198
and uploadedLR select migrated legacy-only ADD; otherwise legacy SET.
Legacy child presence triggers even if optional value is unparseable, as
before. PQ-only notifications remain ACK-only under declaredfalse. A PQ
upload-result demand flag independently enters ADD through Success.

Non-migrated empty legacy inventory generation now occurs inside its SET
reservation transaction. Migrated ADD uses existing family-specific inventory
and batch defaults. Uncertain KEM/refusal holds are checked before any such
generation/reservation. Cursor conventions and historical frozen requests
remain unchanged. No timestamp-based migration inference is introduced.

Synthetic actor regression now exercises initialPQ SET ->PQ-only success
ADD, then PQ-only count ACK without upload, then malformed optional legacy
count ACK +legacy-only ADD200, settling allthree batches. Expanded library
regression verifies a count notification cannot bypass an uncertain KEM ADD
before/after reopen. Existing non-migrated count and receive tests remain.

**Remaining:** effective PQ-notification configtrue branch, current startup
strategy, native retry/backoff/reset and cursor/storage migration. Local ACK
wire ordering still follows durable capture before upload work; complete native
worker/ACK durability/order is not proven. No live traffic/full parity claim.

## 2026-11-04 — Startup new-store/digest distinction

**Confirmed static (43.72):**new-store flag drivesFea6, which sends
existingunsent keys or onlygenerates;non-new-store digestflag drives
A0M/readiness/elapsed>60000ms thenarg1=88. A0Q mark-unsent entry is not
proven genericstartup. Rust startup parity remains incomplete.
[Exports and port requirements](apk/2.26.43.72-prekey-upload-replies/README.md#new-store-startup-and-digest-entry-distinction-2026-11-04). No production change or live traffic.

## 2026-11-04 — Digest request/reply wire grammar

**Confirmed static (43.72):**arg1=88 GETencrypt/digest with291selector1;
result fixed-sizefields/id lists and hash20/32 dependingonLR presence.
Success/error events82/83/84 distinguished; numeric parser differs from
upload290. [Exports and verification/port limits](apk/2.26.43.72-prekey-upload-replies/README.md#digest-request-and-wire-callback-grammar-2026-11-04).
No production change or live traffic.

## 2026-11-04 — Digest local verifier/resynchronization

**Confirmed static (43.72):**orderedlocalkey-material digest with
SHA1/SHA256 chosenbyPQgate+LRpresence;IDs arenot hashed. Registration,
key availability/ID andhashmismatches have separateA0Q reasons.
Attempttime andpersisted digestflag clears distinguished.
[Exports, preimage and recovery limits](apk/2.26.43.72-prekey-upload-replies/README.md#digest-local-material-verification-and-resynchronization-2026-11-04). No production change or live traffic.

## 2026-11-04 — Digest error/resync metadata scope

**Confirmed static (43.72):**404 worker callsA0Q4 thenclearsdigestflag;
503 onlyclearsattempttime. Resync updates formerlysent legacy/KEM metadata
and clears key-type uploadhistory, preserving recordfields inselectedSQL.
Uncertain Rustreservations remain distinct; correlation/port stillopen.
[Exports and statechange boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#digest-not-foundserver-error-and-metadata-resync-scope-2026-11-04). No production change or live traffic.

## Offline digest inspection (2026-11-04)

**Implemented/verified offline from43.72 wire/verifier evidence.**
`whatsapp-cli prekeys --digest-capture <saved-inbound-capture-key>` decodes
a single existing encrypted capture and prints redactedJSON disposition,
legacy/PQ counts,LRpresence andalgorithm. Library entry is
`transport::prekeys::digest::inspect(store,node)`. It requiresIQresult/digest
grammar and never sends, queues, acknowledges, clearsflags, resetsmetadata
or applies recovery. Existing GET /v1/prekeys remains summary-only; this
CLI flag does not create an HTTP inspection route.

Parser keeps firstmatchingrequiredchildren, direct3-byteID lists andoptional
PQ null-versus-empty distinction; LRpresence determineswirehash length.
Local comparison uses registration,type,signedID, requestedlegacy order,
PQgate/LR algorithm condition andavailablekeymaterial. Sorting/deduplication
is not introduced. Missing/material mismatches are fixed redacted dispositions;
invalid localrecords/grammar return fixed diagnostics ratherthan rawpayload.
MultipleuploadedLR rows returnAmbiguousLastResort because native insertion
order metadata is not available locally; no highest-ID guess is made.

**Historical signature gap (closed for new reservations below):** older StoredKemPrekey rows do not persist uploadedLR signatures.
Re-signing is randomized and would givefalsehashmismatch. Inspector recovers
only from an acknowledged frozenmode=set request naming the sameLRID and
publicbytes, validates requestID/type/namespace andidentitysignature, and
requires unambiguous signature bytes. Missinghistoricalwire orconflicting
signatures returnsUnavailableLastResortSignature. No backfill or rowmutation.

Account,keyrows and registeredsignedprekey validation shareoneReadTx via
extractedfactory helper. The saved PQ gate now shares that same ReadTx; results remain
provisional offline evidence, **not request correlation, snapshot authority
for recovery, or proof of currentserver state**. Historicalreply comparison
usescurrentlocalmaterial and cannot establish a past match or authorize
changingOutcomeUnknown reservations.

Regressions: independentPython cryptography/X25519 +hashlibSHA1 public-material
vector (oracles/prekey_digest_reference.py anddigest-referencefixture),
reordered/repeated/missingIDs, invalidhashsize, read-onlytablecomparison;
PQdigest builtfrompublishedSET verifies repeatedinspections do notresign;
new rows retain exact publication signatures without frozen history; removing both synthetic signature and history yieldsUnavailableSignature.
The structuralSHA1 fixture is not an authenticatedkey fixture; publicinspect
validates the registeredsignedkey through existingfactory logic.

**Remaining:** actualdigest request/admission, startupflag lifecycle, correlated
responseorigin/generation, nativeinsertionordering and authorizedresync/retry.
No liveaccountoperation or complete startup/recovery parity claimed.

### Exact KEM publication signatures (2026-11-04)

Implemented offline: KEM upload reservation now atomically persists the exact
identity signature sent for each selected row. Later construction reuses and
verifies saved signatures; digest inspection prefers the saved last-resort
signature, with acknowledged frozen SET recovery for historical unsigned rows.
Missing historical signatures remain unavailable; inspection never re-signs or
writes. Invalid saved signatures fail locally. Reservation rollback and consumed
row handling remain unchanged.

Confirmed static source remains pinned APK 2.26.43.72 EqE/Dfd signature storage
and Fdf digest preimage (see the prekey-upload-replies evidence bundle). This is
partial storage parity: Rust signs at reservation, whereas native generation
stores the signature earlier. No live verification, recovery, retry-policy or
ban-prevention guarantee is implied.

Validation: workspace/all-feature tests passed with two test threads, strict
Clippy, CLI reference, fmt, coverage freshness and hygiene passed. The initial
default-concurrency run hit an unexpected-write assertion in the migrated actor
test; the same test passed alone and in the complete bounded run. Heartbeat
timing under load is suspected, not proven; production timing was not changed.

### PQ gate and key transaction consistency (2026-11-04)

**Implemented, offline:** prekey SET/ADD reservation reads the saved property
21198 through the same WriteTx that selects/generates key rows and freezes the
publication request. Digest inspection reads the saved gate, account, registered
signed key and local prekeys through one ReadTx. A prior separate store read
could combine a previously read gate with later key state. This is a local
transaction consistency fix, not new APK timing evidence or a MobileConfig port.

Both paths reuse the existing correlated AB-property parser and its absent/
invalid false default. The reservation path also validates decoding of the
saved original query through the existing action write-view helper. A missing
referenced action or malformed stored action remains a local error, never an
inferred enabled gate. Static gate provenance remains APK 2.26.37.71 D1G.Bat /
07t.A0L and 2.26.43.72 13z migration predicates in the retained prekey bundles.

Regression holds an immutable snapshot across a committed config/key change,
checks uncommitted gate deletion in a write transaction and rollback, and
reopens the resulting synthetic store. Request/reply origin, current actor
account binding, effective MobileConfig, startup recovery and safe resync remain
separate unfinished work. No live operation or automatic recovery was added.

Validation: all 1,296 workspace/all-feature tests passed with two test
threads; strict Clippy, CLI reference, fmt, coverage freshness, JSON validation
and repository hygiene passed. All tests used synthetic/offline stores.

### Prekey publication account authority (2026-11-04)

**Confirmed local defect, fixed offline:** begin_upload_selected accepted an
AccountRecord supplied before reservation without rereading the durable account.
It also serialized the supplied signed-prekey signature without verification on
the PQ-disabled path. A stale caller could therefore freeze a SET under obsolete
registration/identity material, or submit a signed key from another identity.

SET and ADD admission now require registered caller and stored accounts, equal
transaction/stored/caller generation, registration ID, own JID, phone and Signal
public/private identity. These checks run in the existing reservation WriteTx
before key selection, generation, signature persistence or request freezing.
Missing account and changed authority have fixed local errors. Signed-prekey
signature is verified over the type-prefixed serialized public key under the
publication identity even when PQ is off. Authentication timestamps/credentials
are not publication identity and do not invalidate an otherwise matching caller.

This is a local consistency invariant. APK 2.26.43.72 13z.A0B/A0S request layout
and stored key sourcing remain documented in the prekey-upload-replies bundle;
these Rust error messages and checks are not claimed as native callback policy.
Regression fixtures now persist their synthetic account before reserving uploads.
Nine stale/missing/revoked account cases leave Prekeys and Checkpoints unchanged
across reopen; a foreign-identity signed key fails with PQ off, while an unrelated
authentication timestamp update permits the valid request.

**Remaining:** same-identity signed-key rotation ownership, account changes after
reservation/before write, correlated digest recovery and native retry/reset
policy are not closed by this guard. No live operations or automatic resets.

Validation: all 1,298 workspace/all-feature tests passed with two test threads;
strict Clippy, CLI reference, fmt, JSON, coverage freshness and hygiene passed.
Earlier fixture failures exposed unsaved/generation-zero test accounts; those
fixtures now persist the matching store generation. No production guard was
weakened to accommodate them.

### Prekey wire ID integrity (2026-11-04)

**Confirmed local construction defects, fixed offline:** signed-prekey IDs wider
than three bytes were truncated by be3. Legacy publication trusted the numeric
store-key suffix without checking the deserialized PreKeyRecord ID, so a row
could publish its public key under a different ID. Offline digest comparison
likewise accepted those inconsistent rows and could match an aliased signed ID.

Signed IDs are now checked before publication or digest comparison. Legacy rows
share a checked loader that requires a lossless three-byte requested ID and an
embedded record ID equal to its store key. Publication failures roll back the
reservation transaction; read-only digest failures leave records untouched.
KEM publication already checked the three-byte width and row ID binding.

**Confirmed static boundary:** pinned APK 2.26.43.72 1nr.A04 writes low24 bits
(shift16/8/0), retained under ADD selection and exact child layout in the
prekey-upload-replies bundle. Its truncation is native encoder behavior; rejecting
wide local values is our consistency guard, not evidence that native rejects
those values. Native legacy row parsing can skip caught IOExceptions; Rust's
stricter invalid-row refusal remains explicitly different. No cursor, sorting,
generation, retry, reset or acknowledgement policy changes are inferred.

Regressions cover mismatched legacy store/embedded IDs, wide legacy IDs, signed
IDs 0x1000000/u32::MAX, and exact 0/0xffffff field boundaries. An independent
SHA1 digest fixture is reused with unchanged public bytes but a mismatched row
ID, and with an aliased wide signed ID: both now fail instead of falsely matching.
These field-boundary tests do not establish native generation of ID zero.

Validation: all 1,299 workspace/all-feature tests passed with two test threads,
including 35 targeted prekey tests. Strict Clippy, CLI reference, fmt, JSON
validation, coverage freshness and repository hygiene passed. Offline fixtures
only; no server or account operation was performed.

### Count ACK metadata and queue boundaries (2026-11-04)

**Confirmed static, APK 2.26.43.72:** count dispatch requests deferred metadata;
89Y.A01 consumes a one-use7p2 token viaA00. The downstream0v6.A00 branches to
bulk1ic.A0G or0Sf.A0O readiness/pre-ACK queue admission. The latter creates
message76 and can convert to queued pre-ACK message43; it is not a synchronous
socket-completion callback. [Exact methods, offsets, limitations and 18 checked
exports](apk/2.26.43.72-prekey-upload-replies/README.md#count-notification-ack-token-and-queue-stages-2026-11-04).

This closes the meaning of the worker's ACK call, not native wire-order parity.
The same metadata object dispatches at most once, but durable/redelivery-wide
ACK deduplication is not established. A six-second handler timer and pre-ACK
threshold200 were observed statically; neither is a new Rust timer/default.
Concrete live branch selection, queue durability,43 pre-ACK conversion/flush and
failure/reconnect drain behavior remain unknown. Rust ACK-before-upload-decision
ordering remains unchanged pending those boundaries; no speculative socket
reorder, pre-ACK outcome_unknown reservation, retry or duplicate suppression.
No production source change or live account operation in this evidence chunk.

Validation: pinned APK digest and all18 new export checksums verified;
OpenAPI JSON, coverage freshness, staged whitespace and repository hygiene
passed. This documentation/evidence-only chunk did not rerun Rust tests.

### Durable pre-ACK storage and confirmation watermark (2026-11-04)

**Confirmed static, APK2.26.43.72:** the ACK conversion helper accepts message76
and uses the same generic ACK builder as its direct writer. Message43 writes the
already-built node. Native1iV.A0F serializes queued nodes into SQLite preacks.ptn;
1iY loads by ascending row ID and deletes cumulatively through a confirmed row
watermark. This closes the prior unknown about existence of durable pre-ACK
storage, but not whether every live count notification selects that branch.

The ping manager snapshots a queue marker and its successful IQ callback requests
retirement; socket write alone is not proven retirement authority. Replay-deferred
and passive-state holds, a reconnect-epoch check, capacity-driven pending drops,
periodic flush scheduling and persisted/pending selection are retained. [Exact
methods, boundaries and 24 new checked exports](apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-conversion-sqlite-persistence-and-retirement-2026-11-04).

**Still open:** effective configuration, complete initial load and replay-send
ownership, bulkCCQ selection, ping correlation/error timing, partial/ambiguous
commit handling, and all retirement entry points. Rust has durable raw inbound
captures, not this native pre-ACK queue. No ACK replay/deduplication, new ping,
automatic deletion or socket-order change was implemented from partial evidence.
No live account operation; this is a documentation/evidence chunk.

Validation: pinned APK SHA256 and all24 new export hashes verified; OpenAPI
JSON, coverage freshness, staged whitespace and repository hygiene passed.
Documentation/evidence-only change; Rust tests were not rerun.

### Independent keepalive response deadline (2026-11-04)

**Fixed, offline:** heartbeat maintenance writes could delay an outbound ping,
then the next heartbeat expired it after only the remaining fraction of the
interval. A paused-clock regression reproduces a19-second maintenance delay
followed by a ping that previously timed out1second later. Each ping now owns
an admission-time deadline and a separate timeout selection; silence still
produces the existing recoverable Timeout after the configured full window.
Heartbeats cannot send a second ping while one is pending. Existing response
correlation, duration policy, request shape and reconnect classification remain.

**Confirmed static, APK43.72:** the pre-ACK confirmation ping is an empty
IQ/get/w:p to s.whatsapp.net. 0Sf.A06 registers its ID callback in0ln.A08 before
send admission, with an independent Handler delay (32000ms for this native
purpose). [Request, timer methods and regression limits](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
The native pre-ACK delay does not replace Rust's local keepalive setting. This
does not port durable ACK replay/retirement, or guarantee timely observation
while the actor is blocked in a write. No live operation was performed.

Validation: original paused-clock regression reproduced premature link closure;
corrected regression and existing keepalive tests passed. All 1,300 workspace/
all-feature tests passed with two test threads; strict Clippy, CLI reference,
fmt, JSON validation, coverage freshness, ten new export hashes and hygiene
passed. Tests used offline injected links; no live operations.

### Pre-ACK reload and replay-generation ownership (2026-11-04)

**Confirmed static,43.72:** initial load has unbounded and oldest-window modes;
window mode tracks total persisted count separately and can skip up to eight
empty decoded pages. Reload preserves SQLite IDs with loggableID=-1. The store
compares dictionary version3 and deletes incompatible nonzero-version pre-ACK
rows; this is not identity/key reset authority. Declared descriptor values are
retained, while effective configuration remains unknown.

Replay generation initialization clears the sent watermark and increments an
in-memory connection counter. Mark-sent, clear-deferred and delayed retirement
check that generation; a watermark is not an account-generation or stanza ID.
Capacity SQL prunes newest excess rows in batches500, preserving the oldest
prefix. An independent synthetic SQLite oracle verifies strict-after paging,
cumulative retirement and that pruning direction. [Exact boundaries,11 new
exports and oracle](apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-reload-capacity-and-connection-generation-ownership-2026-11-04).

A supplied-transaction insertion path also exists, but its callback commit/
rollback semantics and full replay-batch ownership remain unfinished. No Rust
send/replay/deletion, socket order, timer or account behavior changed here.

Validation: pinned APK SHA256, all11 new export hashes and synthetic SQLite
oracle fixture checked; OpenAPI JSON, coverage freshness, staged whitespace
and repository hygiene passed. No Rust source changed or Rust tests rerun.

### ACK transaction success and rollback bridge (2026-11-04)

**Confirmed static,43.72:** ACK insertion's0aq.A04 registers MZH selector1.
Its Runnable executes in BjU after successful outer transaction completion,
not directly in SQLite.onCommit. Separate selector2/BjQ handles rollback.
1hk.close ends SQLite first and only selects success release when no captured
end error, its success flag is set, and no rollback marker is present. Inner
transaction closure does not release the outer callback map.

Bulk1ic.A0G ->8b6(selector1)->1ic.A0A can insert pre-ACKs with the same0aq
transaction used by its queue helpers, then register success/rollback Runnables.
After successful completion, TB5(11) promotes persisted entries directly or
schedules further projection according to load/window state. This closes the
normal callback boundary, not crash/disk-error semantics or complete provenance
of every count notification's ACK list. [Methods,19 new exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#ack-transaction-completion-and-rollback-bridge-2026-11-04).

**Remaining:** complete count-specific bulk/config selection, Flr insertion
coroutine, TB8 projection, exact window comparator, ambiguous persistence and
full replay ownership. No Rust network/recovery behavior or live account state
changed. Future port must distinguish durable insertion from post-commit memory
promotion and confirmation-based retirement.

Validation: pinned APK SHA256 and all19 new export hashes verified; OpenAPI
JSON, coverage freshness, existing synthetic SQL oracle, staged whitespace
and hygiene passed. Evidence/docs-only; Rust tests were not rerun.

### ACK projection serial queue and row-order helpers (2026-11-04)

**Confirmed static,43.72:** TB8 selector8 directly calls A07; it does not add
another latch wait. The queue owns a098 serial executor: its load/projection/
periodic jobs use a FIFO wrapper that attempts next-task dispatch on normal or
exceptional completion. This closes same-instance queued-job ordering, not all
cross-thread epoch mutation, dispatch failures or database snapshot isolation.

Fey14 orders persisted row IDs naturally, substituting Long.MAX_VALUE for null;
no loggable-ID tie-break is used. Out-of-order window merge concatenates, sorts
and takes a prefix without deduplication. Refill keeps a null/backend failure
separate from a completed empty read, and adjusts only memory/count state on
successful short reads. [Methods,17 new exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#ack-projection-serialization-merge-order-and-refill-2026-11-04).

These are additional recovery-port inputs, not implemented Rust ACK replay or
proof every callback schedules after loading. Complete bulk ACK-list provenance,
Flr insertion, replay ownership and ambiguous persistence failures remain open.
No production Rust source or live account/network behavior changed.

Validation: pinned APK SHA256, all17 export hashes, OpenAPI JSON, coverage
freshness, existing synthetic SQL oracle, staged whitespace and hygiene passed.
Evidence/docs-only chunk; Rust tests were not rerun.

### Actor IQ waits expire independently (2026-11-04)

**Confirmed local defects, fixed offline:** prekey upload and device-query waits
had30-second monotonic deadlines, but only a20-second heartbeat checked them.
A reply at31seconds could settle either request before the40-second tick. In
paused-clock regressions, the old code promoted50 prekeys to uploaded and
converted a reserved device query to server-refused from such late replies.

The actor now selects each wait's own deadline. Expired uploads drop only their
session-local waiter and keep durable outcome_unknown/key reservations. Expired
device queries are deferred for this session while pending users and the durable
query row remain unchanged. This timeout does not authorize a resend, key reuse,
repeating those deferred users, cooldown bypass or clearing uncertain history. Existing admission
triggers and the selected30-second local durations are retained.

Prekey, device-query and keepalive reply admission also checks that its waiter
has not expired, so a simultaneously ready incoming branch cannot settle an
expired waiter merely by winning select. Original inbound capture still commits
before reply handling. Session monotonic deadlines are not reconstructed from
wall-clock/server timestamps after restart; offline library settlement/inspection
and other request families remain separate authority boundaries.

**Confirmed static timing architecture, APK43.72:** 0Sf.A06 registers a pending
ID callback through0ln.A08 before send dispatch; positive delays use an
ID-specific Handler message. [Retained methods](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
That supports independent request timers, not proof that30seconds is this APK's
native timeout for both families or that Rust now ports its entire callback map.
The earlier native pre-ACK confirmation ping's32-second delay remains distinct.

Regressions: late_prekey_reply_cannot_settle_between_deadline_and_heartbeat and
late_device_reply_cannot_settle_between_deadline_and_heartbeat. They preserve
late captures, check uncertain/query/pending-user state, service an inbound ping
and reopen without reinstalling account material. The former verifies50 keys
remain uncertain and none uploaded; the latter verifies the query remains
identical instead of storing a late503 refusal.

**Limits:** awaited writes may still delay observing expiry. Device admission
and unrelated maintenance remain on their existing cadence. Outgoing/group
waits and other request families are not converted by this chunk; no full
network/timing or ban-prevention guarantee is claimed. No live operations.

Validation: both new regressions reproduced late settlement before the fix
and passed afterward. All 1,302 workspace/all-feature tests passed with two
test threads; strict Clippy, CLI reference, fmt, JSON and coverage freshness
passed. Staged whitespace and repository hygiene passed. Capture/reopen checks used
synthetic stores and injected links only.
