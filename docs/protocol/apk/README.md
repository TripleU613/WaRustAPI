# Compact APK research snapshot

[Whole-APK file index and fast diff](../apk-file-diff.md) now complements these scoped
method exports. The committed 2.26.37.71 index covers 14,967 archive files;
new release indexes report changed paths before method remapping/review.

[Protocol research](../apk-analysis.md) · [Findings](../decompilation-log.md)

The `2.26.37.70/` bundle preserves selected protocol code as annotated Dalvik
instructions, plus source metadata, archive-size totals, the recovered token
dictionary, and checksums. It excludes images, audio, video, fonts, resource
payloads, executable DEX/native binaries, and full-app string dumps.

## Contents

| File | Purpose |
| --- | --- |
| `manifest.json` | Exact APK hash, Android version metadata, certificate fingerprint |
| `inventory.json` | Archive category sizes and DEX/manifest hashes; no media contents |
| `tokens.json` | Comparison dictionary recovered from bytecode, with its location |
| `targets.json` | Topic-to-class map and method counts |
| `*.txt` | Selected registration, Noise, certificate, socket, framing, and node-writer code |
| `checksums.json` | SHA-256 and byte length of every other generated file |

The exporter enforces **2 MiB total**, including checksums, before writing.
Files are plain text for search and review. No Git LFS is needed.
The committed snapshot has **26 files, 564,315 bytes, 21 classes, and 149
methods**. This is about 0.39% of the 146,380,260-byte source APK.
All selectors are scoped to APK SHA-256
`b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732`;
a different APK is rejected instead of applying potentially unrelated class names.

## Reproduce

From the repository root, with the exact comparison APK available locally:

```sh
cargo run --locked -p xtask -- apk-research-bundle /path/to/WhatsApp.apk \
  --output lab/rebuilt-protocol-bundle
```

The output directory must not already exist. To verify the committed snapshot
without modifying it:

```sh
cargo run --locked -p xtask -- apk-research-bundle /path/to/WhatsApp.apk \
  --output docs/protocol/apk/2.26.37.70 --check
```

The exporter reads DEX entries in memory. A full unpack is unnecessary for
reproduction. Local raw extraction belongs in ignored `lab/`; the original
APK remains outside Git. No account or live-server access is involved.

## Interpretation

These are APK-derived research artifacts, not first-party Rust source or
compilable Java/smali. Instruction offsets are relative to each method;
prototype strings are DEX shorties, not complete signatures. The structural
decoder leaves some operands unannotated and marks unknown mnemonics `??`.
It does not reconstruct whole-program control flow or original names. Recipe
options below can retain decoded exception tables and supported register operands.
Use the APK and existing `apk-smali`/`apk-xref` commands for deeper analysis.

This is a focused snapshot of currently mapped protocol classes, not a full
decompilation or a complete WhatsApp implementation. UI and unrelated code
are omitted. Constants compiled into an APK are client-static evidence;
they do not prove server acceptance or promote the active 2.26.36.75 profile.
The historical token extraction has four refuted slots; see
[token extraction evidence](../token-extraction.md). The comparison runtime now
uses the checked 37.71 fixture; the pinned Android profile remains unchanged. Existing curated findings remain authoritative for what
has actually been interpreted and verified.

## Version-scoped recipes and fast cached differences

`apk-research-bundle --recipe <json>` supports a different release only when the
recipe's exact SHA-256 matches. Each stable topic maps to an exact class, optional
method substring filter and relative affected Rust paths. Unknown fields and
unsafe names/paths refuse; absent/ambiguous classes refuse before writing. The
output directory and 2 MiB media-free constraints remain. Default 37.70 selectors
are unchanged.

The new [37.71 group recipe](recipes/2.26.37.71-groups.json) reproduces
[its scoped baseline](2.26.37.71-groups/manifest.json): 16 files/423,242 bytes.
This baseline includes eleven selected investigation targets, not all APK code.
The initial six-target export measured 16.768 seconds. Real cached
self-comparisons of that earlier baseline measured 0.0183..0.0215 seconds,
returning all six unchanged topics;
this is not a measured comparison of two distinct APK releases. Separate fixture
tests exercise meaningful changed/added/removed topics and provenance-only changes.

```sh
cargo run --locked -p xtask -- apk-research-bundle /path/to/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-groups.json \
  --output lab/group-baseline

target/debug/xtask apk-research-diff lab/group-baseline lab/next-group-baseline
```

The diff loads checksum-verified bounded exports without APK scanning and prints
JSON topic status, both evidence file paths/class names and affected Rust paths.
Only the per-file APK hash header is excluded from content comparison; renames,
offsets and unknown instructions remain visible. Re-anchor the next release's
recipe before exporting it. See [full porting direction](../apk-porting.md) for
coverage expansion and interpretation/patch requirements. There is no HTTP route,
account access, live request or automatic application of the changes.

Non-topic `artifacts` changes include manifest, inventory, target mappings and
the token dictionary; these remain visible separately from code topic changes.
The checksum index itself is input verification, not a semantic-change report.

Final validation: 588 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, CLI reference,
five OpenAPI checks, formatting, staged whitespace and hygiene (425 local links)
passed. Exact 37.71 snapshot regeneration with --check passed in 17.467s. Final
cached reports (six topics plus four metadata/dictionary artifacts) took
0.0200..0.0213s; all unchanged in the real self-comparison.

## Re-anchor a new release

`apk-research-remap <apk> --recipe <old-json> [--output <new-json>]` uses optional
exact-string `anchors` in each target. All anchors must share the target's method
or class scope. Missing, ambiguous, unanchored and split-scope results stay
explicit; no previous class name is copied as a guessed match. A new proposal is
written only when all topics are unique, under the new APK SHA-256; existing
files are refused. Source/caller review remains necessary before patching Rust.

The checked scanner fails DEX/structure errors rather than returning the older
exploratory xref tool's partial result. Physical method indices keep name/shorty
overloads distinct and whole-class selectors stay whole-class selectors. Candidate
sites and all anchor occurrences are preserved in JSON. This is not full DEX VM
verification or native-code remapping. See [porting workflow](../apk-porting.md#anchor-assisted-relocation).

Method targets can add `caller_anchors` to require a caller containing those
strings and a symbolic invocation with the candidate's full signature. The
current group classifier ambiguity is resolved this way; missing/noninvoking
evidence refuses instead of guessing. See [caller workflow](../apk-porting.md#caller-anchors-for-ambiguous-methods-2026-10-07).

## Reviewed 37.71 core baseline

[Core release review](../core-release-review.md) records the real 37.70→37.71
comparison, fourteen unique relocations and seven manually reviewed selections.
The [37.71 recipe](recipes/2.26.37.71-core.json) covers 22 topics and maps them to
Rust files. Its [baseline](2.26.37.71-core/targets.json) contains 27 files,
731,482 bytes and 150 methods. The [cached difference](37.70-to-37.71-core-diff.json)
took 0.0512 seconds: one added topic, nineteen changed texts, two unchanged.
Obfuscation and decoder annotation changes are included; this is not nineteen
confirmed behavior changes. The raw 37.70 APK is unavailable for re-extraction.

The typed-subject checkpoint had fifteen topics and 20 files/430,619 bytes. Checked token
regeneration previously left eleven code topics unchanged; subsequent typed
subject coverage adds four topics and signed annotations update two existing
texts. [Coverage diff](subject-types-coverage-diff.json) took 0.0227 seconds.
[Relocation evidence](remap-evidence/2.26.37.71-subject-types.json) has thirteen
unique topics and two unanchored helpers. The JLL parser constraint distinguishes
three string-matching methods; [wrong-signature evidence](remap-evidence/2.26.37.71-subject-wrong-shorty.json)
records an actual no_matching_shorty refusal without proposal creation.

The current twenty-topic group baseline is **25 files/448,127 bytes**. Five
recovery/MEX/guard topics were added with fifteen unchanged sources in a
[0.0204-second cached comparison](group-info-recovery-coverage-diff.json).
[Current remapping](remap-evidence/2.26.37.71-group-info-recovery.json) finds sixteen
unique and four unanchored topics. [Recovery interpretation](../groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07)
distinguishes recovered MEX behavior from the current classic Rust lookup.

The [37.71 MEX recipe](recipes/2.26.37.71-mex.json) now covers **38** request,
parser/callback, numeric, full group converter and participant/JID helper
methods linked to Rust's MEX module. Baseline: **43 files/162,671 bytes**,
including checksums and explicit catch tables. The opt-in recipe boolean
`include_exception_tables` defaults to false, preserving historical exports.
[Current coverage comparison](mex-participant-coverage-diff.json) against Git
78a7218 took **0.023364 seconds**: twenty added topics, eighteen changed only
by catch-table annotations; all their old instructions are identical after
stripping those annotations. This is a source/format expansion on the same
APK, not eighteen protocol changes. A fixture-based regression verifies that
a catch-only change is detectable. Inventory/manifest/tokens remain unchanged.
[Current remapping](remap-evidence/2.26.37.71-mex.json): fourteen unique,
seventeen unanchored, seven ambiguous; no full automatic proposal. Previous
[callback coverage checkpoint](mex-callback-coverage-diff.json) against cdd4056
retains its historical eighteen-topic interpretation. [Reproduction/boundaries](../mex.md#exception-aware-numeric-and-participant-release-coverage-2026-10-07).

Core source presentation refresh: naming bd mul-long/2addr updates one
noise-handshake annotation (+045e..0460), retaining opcode/offsets/operands.
[Exact cached comparison](core-mul-long-rendering-diff.json) against Git 34b5ab5
shows one changed/21 unchanged topics and four unchanged artifacts in 0.0328
seconds, same APK. Current core size: 27 files/731,494 bytes. This is an
annotation/checksum correction, not a new release or handshake behavior.

MEX coercion ownership refresh: four topics additionally identify the private
Rust coercion module. [Cached comparison](mex-coercion-owner-diff.json) against
Git 4fea159: 0.016635 seconds, all 38 method texts unchanged, targets metadata only.
Current snapshot remains 43 files, now 162,671 bytes; no new APK/behavior claim.
[Scalar port/reference](../mex.md#participant-scalar-coercion-and-float-rounding-2026-10-07).


Contact ADV caller expansion (37.71): core now includes eleven exact selected
signature/reconciliation/validation, sync/add-notification, baseline and identity
application methods. Current core: 33 topics/161 methods, 38 files/967529 bytes.
[Cached comparison](core-adv-contact-diff.json) against Git b058f61 took
0.028928 seconds: eleven added/twenty-two unchanged topics; inventory, manifest
and tokens unchanged, targets metadata expanded. Existing topics are not
rewritten as a new release. [Caller findings and runtime gaps](../cross-device-sync.md#contact-adv-verification-and-retention-2026-10-07).


The 2026-10-08 companion-removal property checkpoint expands the current core to
**164 topics / 349 methods / 169 files / 2,068,819 bytes**, leaving 28,333 bytes
under the 2 MiB bound. Nine property/default/cache/resolver/fallback topics were
added; all 155 previous topic texts and manifest/inventory/tokens were unchanged.
The [cached coverage comparison](companion-feature-gate-coverage-diff.json)
took **0.0942 seconds** after one actual APK export. This compares source coverage
of the same build, not a release behavior difference. For the earlier snapshot,
restore the core directory from commit `07fd5da`; regenerate the current snapshot
using [the recipe](recipes/2.26.37.71-core.json) and the hash-pinned APK.
[Property interpretation and unknowns](../peer-identity.md#companion-removal-property-and-effective-value-2026-10-08)
explain why false baked defaults do not establish the effective account setting.
No unanchored helper is automatically relocated or mapped to a new APK.


The [37.71 ABProps recipe](recipes/2.26.37.71-abprops.json) preserves eight
request/callback/type-conversion/login-hint/server-address topics, with exception
tables: **13 methods / 13 files / 97,174 bytes**. Its separate
[baseline](2.26.37.71-abprops/targets.json) avoids expanding the nearly full core
bundle. [Runtime read and remaining configuration gaps](../feature-configuration.md)
distinguish saved response evidence from an effective feature value. Generated
07t defaults remain local; compact Boolean registration windows are attributed
separately. No native MobileConfig mapping or automatic relocation is claimed.


The [37.71 group recipe](recipes/2.26.37.71-groups.json) now preserves
**39 topics / 43 methods / 44 files / 928,771 bytes**, with exception tables.
Five new topics cover ACK dispatch, network write, postwrite and frame encoding.
[Cached coverage comparison](group-ack-writer-coverage-diff.json) against
80f5636 took 0.039709 seconds; all 34 old instruction texts are unchanged after
removing exception annotations. [Writer trace and scope](../groups.md#ack-dispatch-and-uncompressed-write-boundary-2026-10-08)
explains the uncompressed boundary and why no runtime correction is justified.
This is scoped same-APK coverage, not full release remapping or protocol parity.


The [37.71 companion cleanup recipe](recipes/2.26.37.71-companion-cleanup.json)
preserves **57 topics / 76 selected method declarations / 62 files / 450,854 bytes**, including
session/identity SQL predicates, canonical-address branches, property definition,
transaction callback registration and cache-variant cleanup. Its
[baseline](2.26.37.71-companion-cleanup/targets.json) is separate from the nearly
full core bundle. [Interpretation and remaining gates](../peer-identity.md#companion-key-removal-scope-and-callback-boundaries-2026-10-08)
distinguish static deletion scope from permission to run cleanup. No account
keys/history are deleted; native/full mapping and effective feature gates remain open.


The cleanup recipe now enables optional root `include_register_flow` (boolean,
default false). Supported move/result/move-exception/array-length operands use
the interactive decoder fallback when the primary annotation is empty; nonempty
annotations keep precedence. Opted-in targets record the option. Historical
default snapshots remain stable. This does not decode every opcode or provide
lossless bytecode, whole-program dataflow or automatic Rust patches.

Six added transaction topics establish late successful-close delivery of the
selected session-cache callback and its rollback/discard behavior.
[Callback evidence and remaining limits](../peer-identity.md#cleanup-callback-commit-and-rollback-closure-2026-10-08)
carry the build/class/method attribution. The [cached coverage report](companion-callback-flow-coverage-diff.json)
against `257f186` took **0.011257 seconds**: six additions, sixteen annotation
changes and one unchanged topic. All seventeen old offset/opcode/mnemonic lists
and catch ranges are equal; manifest/inventory/tokens are unchanged. This is
expanded source/rendering coverage of the same APK, not a vendor release change.


The sender-key/deleted-observer checkpoint added eighteen scoped topics. At that
checkpoint, fifty-five declarations had code; DeS.BqD was the abstract declaration.
[Cached same-APK comparison](companion-sender-key-coverage-diff.json) against
`72afc9d` took **0.013700 seconds**: eighteen additions and all twenty-three old
topic texts unchanged, with manifest/inventory/tokens unchanged. All eighteen
new direct declaration/instruction/catch coverage lists match the export.
[Selection, local bounds and refuted first-identity label](../peer-identity.md#sender-key-group-selection-and-deleted-identity-observers-2026-10-08)
explain group-row selection, eight-group slices, three local passes and BqD
versus BqB/BqC. The older core identity-pin-first-event topic name is retained
for stable comparisons; its interpreted first-key meaning was refuted. No
teardown runtime or account/key mutation is enabled by this static bundle.


Canonical mapping coverage now adds sixteen topics: phone parsing, repository
single/bulk/counterpart paths, row queries, typed materialization and reverse-sort
map selection. [Cached coverage comparison](companion-mapping-coverage-diff.json)
against eafb8fd took **0.028460 seconds**: sixteen additions/forty-one unchanged
topic texts, with manifest/inventory/tokens unchanged. All new direct
method/instruction/catch coverage lists match. [Row-selection port and limits](../peer-identity.md#canonical-key-mapping-and-row-selection-in-rust-2026-10-08)
distinguish stored-row ranking from authenticated PN/LID mapping. The Rust
selector and redacted jid-map-row-inspect command are implemented offline;
mapping write authority, freshness, full reverse resolution and cleanup remain
unported. One of the 76 selected declarations remains abstract.

Signed 11n const/4 annotations preserve raw hexadecimal spelling and append
negative decimal meaning (-8 through -1). Older immutable generations can
therefore differ only in rendered annotation text after this tooling update;
that alone is not an APK protocol change. Receipt cache evidence and exact
regression scope are in [receipt-cache.md](../receipt-cache.md).
