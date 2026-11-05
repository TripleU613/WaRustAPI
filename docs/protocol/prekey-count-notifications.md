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
