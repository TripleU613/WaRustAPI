# APK-derived Rust client and release maintenance

Owner steering, 2026-10-07: prioritize comprehensive APK reverse engineering into
Rust and a quick, file-based comparison/patch workflow for new releases. The
normal client remains the result; repeated isolated live probes are not the main
work queue. Existing saved identities, keys, requests and server waits remain
preserved. APK-derived compatibility does not establish immunity from account
restrictions, and does not authorize avoiding verification, challenges or limits.

## Release workflow

1. Keep the source APK local. Pin its SHA-256 and Android version in the generated
   manifest; never apply another release's obfuscated selectors blindly.
2. Re-anchor client behavior by strings, descriptors and callers. Put the reviewed
   topic/class/method mapping in a release recipe, with the Rust files it affects.
3. Export reproducible media-free files under stable topic filenames. Keep older
   files intact. Compare cached exports quickly without rescanning the APK.
4. Read changed instructions and their callers/conditions. Separate class-renaming
   and code-layout churn from behavior changes; the diff does not make this
   interpretation automatically.
5. Patch the linked Rust modules, evidence-backed fixtures and API/CLI contract in
   the same change. Preserve unsupported/unknown branches explicitly.
6. Verify offline equivalence, persistence and refusal handling first; perform
   bounded authorized live validation when it adds evidence. Do not repeat
   acknowledged/uncertain sends or manufacture telemetry to obtain a response.

The first versioned recipe is
[2.26.37.71-groups.json](apk/recipes/2.26.37.71-groups.json), covering group
notification/ACK, refresh job and subject freshness helpers. This establishes
release-aware extraction and comparison; it does **not** cover the whole APK.
The existing [37.70 snapshot](apk/README.md) remains intact.

```sh
cargo run --locked -p xtask -- apk-research-bundle /path/to/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-groups.json \
  --output lab/releases/2.26.37.71-groups

# After re-anchoring/exporting the next release with the same topic names:
target/debug/xtask apk-research-diff \
  lab/releases/2.26.37.71-groups lab/releases/next-groups
```

The JSON comparison gives added/removed/changed/unchanged topics, both evidence
file paths, both class descriptors, and affected Rust paths. Non-topic metadata
and dictionary changes appear in the separate artifacts list. Use `git diff --no-index`
on a selected old/new file to read its actual instruction changes. The tool
ignores only the provenance hash line; it does not erase offsets or names to
make incompatible code look identical. Cached comparisons validate file hashes.
Fresh extraction still scans APK code and takes longer than cached comparison.
The first real 37.71 group export took 16.768 seconds; three cached self-comparisons
took 0.0183..0.0215 seconds. Cross-release behavior differences were not proven
by that self-comparison; synthetic fixtures verify changed/added/removed reporting.

## Work queue and completion standard

Expand release recipes and interpreted Rust mappings across onboarding/request
construction, authentication/Noise/certificates, binary dictionaries and framing,
protobuf schemas, Signal identity/session/device behavior, direct/group messaging,
media, receipts, notifications, app-state/history/cross-device synchronization,
and relevant native algorithms. Existing evidence and implementations are reused;
new findings require explicit confirmed/inferred/observed/unknown labels.

An area is complete only when outbound construction, defaults/conditions,
response handling, state transitions and applicable native dependencies are
traced and reproduced in Rust with meaningful fixtures. A disassembly file,
green unit test or server ACK alone does not establish full equivalence.

The group recipe now has eleven topics: the new contact subject writer and
ordered JID-map producer/storage helpers extend the eight-topic baseline.
[Storage and transaction interpretation](groups.md#subject-contact-storage-and-alias-transaction-closure-2026-10-07)
records display-name persistence, cache invalidation, successful insert filtering,
rollback cleanup and full-signature constructor resolution. All eleven topics
remap uniquely on 37.71. Cached comparison shows three added topics and eight
unchanged code topics; the 16-file export is 423,242 bytes. Both APK hashes are
identical, so this establishes source coverage rather than release equivalence.
Subject notifications and downstream alias listener behavior remain unfinished
in Rust; the ownership paths identify where further implementation belongs.

Still required for broad release maintenance: comprehensive coverage accounting,
broader anchor coverage and caller-assisted disambiguation, normalized
method-level comparison where sound, protobuf/token/config extraction recipes,
and native-code change attribution. UI/media payload extraction is unnecessary
for protocol reproduction. Bulk analysis trees and native binaries stay local;
compact attributable evidence and interpreted code stay in Git.

## Anchor-assisted relocation

Recipe targets now accept `anchors`: exact string constants chosen from the
interpreted source. Use specific diagnostic/protocol constants, preferably more
than one independent string. Common strings such as `type` are weak locators.
Local bounds are eight nonempty anchors per target, each at most 4096 bytes.

```sh
target/debug/xtask apk-research-remap /path/to/new-WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-groups.json \
  --output lab/releases/proposed-next-groups.json
```

The scanner runs all anchors in one DEX pass. It uses checked string/class-data/
instruction extents and fails reported DEX/structural errors rather than using
the exploratory `apk-xref` warning-and-continue path. This does not constitute
full VM verification or recovery of unknown/native behavior.

Method targets intersect physical `(DEX, class, method-index)` identities;
whole-class targets intersect `(DEX, class)`. Repeated loads in a method do not
create ambiguity. Same name/shorty with different indices and same index in
another DEX remain distinct. Whole-class targets retain their whole-class filter,
instead of silently narrowing to the method that loaded an anchor.

JSON statuses are `unique`, `missing`, `ambiguous`, `unanchored`, and
`no_common_scope` (all strings exist but share no required scope). Both complete
candidate locations and per-anchor occurrence lists are retained. Index/offset
values are APK-local locators, never stable cross-release identifiers. Original
class/method/Rust ownership remains in the report for comparison. No old-name
fallback is made when anchors disappear or move to separate helpers.

Without `--output`, the command only inspects. With `--output`, every topic must
be unique or the command refuses to write, after printing its diagnostics.
Existing output files are never overwritten. A generated proposal still needs
source/caller/condition review before Rust behavior is changed; matching strings
alone cannot establish equivalence. Method-substring filters become the observed
method name in proposals; broad substring scopes require explicit review.

The committed six-topic group recipe includes scoped anchors from its existing
bytecode evidence. [Current-APK relocation evidence](apk/remap-evidence/2.26.37.71-groups.json)
records every candidate and occurrence. This verifies relocation against the
current baseline; a distinct new-release APK comparison remains unverified.

Confirmed static validation (2026-10-07): APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
yielded unique candidates for all six topics in 5.225 seconds, matching the
reviewed class/method selectors exactly. The linked report carries the DEX,
method indices and instruction offsets needed to repeat each match.
A negative probe combining `GET_GROUP_NOTIFICATION_VERSION_SQL` and
`UPDATE_GROUP_NOTIFICATION_SUBJECT_TIMESTAMP` found both strings but no common
method scope: output was refused, exit 1, with no proposal file created.
An existing proposal was refused without changing its bytes. Adding anchors
changed only the target metadata in the cached comparison: zero code-topic
changes. The regenerated baseline contains 11 files/392,185 bytes.
These are local static observations, not a new-release or live protocol test.

The cache-lifecycle follow-up adds a seventh insertion topic to the current
recipe; all seven anchors resolve uniquely in the same 37.71 APK. The updated
baseline is 12 files/395,753 bytes. Newly named standard arithmetic opcodes
change disassembly rendering, not the APK hash or demonstrated runtime behavior.
See [trace](groups.md#subject-cache-invalidation-and-timestamp-decoding-2026-10-07).

The complete-group classification follow-up adds an eighth reviewed topic
(13 files/397,778 bytes). Its four anchors resolve to **two** methods in 37.71:
ADs.A01 and AZK.AAK. The exact reviewed baseline selector remains ADs.A01, but
automatic proposal writing correctly refuses the ambiguous topic. Seven other
topics remain unique. This is evidence of an actual duplicated locator scope,
not a reason to choose the old obfuscated name in a new release. Caller-assisted
disambiguation remains needed.


## Caller anchors for ambiguous methods (2026-10-07)

`caller_anchors` is an optional exact-string array with the same eight-string /
4096-byte bounds as target anchors. It requires target anchors and method scope;
whole-class caller filtering is deliberately unsupported. All caller anchors
must occur in one physical caller method, and that method must symbolically
invoke the candidate. Multiple qualifying callers are allowed; multiple surviving
candidates remain ambiguous. No old class/name fallback is used.

The scanner maps declaring class, method name and **full** parameter/return
signature across DEX pools. Shorties cannot distinguish different object overloads;
[DEX method/prototype/type-list tables](https://source.android.com/docs/core/runtime/dex-format)
provide the actual descriptors. Code streams use the checked reference walker,
including payload exclusion. Reported read/structure/metadata failures propagate;
this is not complete DEX VM verification, reflective/native reachability or
runtime virtual dispatch resolution.

Extra unresolved statuses: `missing_caller_anchor`, `no_common_caller_scope`,
`no_matching_caller`. Configured topics retain `initial_candidates`,
`caller_anchor_sites`, `qualifying_callers`, `caller_matches` (target, full
signature and all symbolic caller sites); `candidates` is the filtered set.
These are source locators, not proof of unchanged behavior or automatic patches.

**Confirmed static:** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The group-type recipe's target strings originally matched ADs.A01 and AZK.AAK.
Adding actual caller anchor `group_response` resolves ADs.A01 through its caller
A0C and AWc.C8f; A0C loads the string at method-relative 0236 and invokes the
classifier at 0248. [Evidence](group-type-caller-evidence.txt) records all eight
symbolic classifier invokes and five string-anchor scopes. The strict report
retains zero direct invokes for the competing schema reader; it does not claim
that reader is unused or cannot be reached indirectly.

The real scan resolved all eight topics and wrote a new proposal in 10.012s.
All selected class/method pairs match the reviewed current recipe. The updated
baseline is 13 files/397,832 bytes; cached before/after comparison found all
8 code topics unchanged and only targets metadata changed. A distinct new
release remains unverified. This supersedes the earlier current-APK inability
to generate an all-unique proposal, not the need for semantic review.

Three actual current-APK negative fixtures used a missing caller string,
existing strings split across caller scopes, and a valid unrelated writer
anchor. They respectively reported missing_caller_anchor, no_common_caller_scope
and no_matching_caller; every requested proposal exited 1 with no output file.
Synthetic DEX fixtures separately cover same-shorty object overloads, cross-DEX
matching, payload invoke bytes, invalid parameter indices/truncated lists and
physical caller/callee index separation. An initial JSON-bound fixture syntax
error was fixed before the full test rerun. Strict Clippy requested array chunks
for two-byte type entries; the same checked even-sized extent is retained.

The [core release review](core-release-review.md) expands this workflow to 22
registration, Noise, framing, socket and dictionary topics. A genuine cached
37.70→37.71 comparison completes in 0.0512 seconds and points to affected Rust
files. The [token extraction correction](token-extraction.md) demonstrates the
porting loop: trace actual indexed APK writes, preserve refuted historical
results, install a derived Rust fixture, and test literal wire bytes. Broad APK
coverage and complete client behavior remain unfinished.

## Checked shorty constraints

Recipe method targets may specify `method_shorty`, an exact DEX shorty (return
character followed by parameter characters). References/arrays collapse to L;
V is legal only in the return position. Local bound is 256 characters. A shorty
requires a method filter and cannot replace full-signature caller evidence.
Export and remap both apply it; absent matches diagnose `no_matching_shorty`.
The remap report preserves `before_shorty_candidates` and the chosen constraint.
Unknown fields, malformed shorties and a constraint without a method refuse.

The 37.71 subject timestamp parser's diagnostics occur in three methods. The
reviewed JLL return/parameter shape selects 0hs.A09 instead of integer parser
0hs.A06 or reference-valued 8lB.A0L. An intentionally wrong VLL constraint
must refuse proposal creation; it must not fall back to names or another return
type. Two unanchored subject helper topics remain unresolved automatically.
[Typed subject evidence](groups.md#subject-typed-branches-and-signed-timestamp-contract-2026-10-07)
records the resulting source coverage and unfinished Rust notification behavior.

Current typed-subject coverage is fifteen topics (20 files/430,619 bytes).
The [cached coverage diff](apk/subject-types-coverage-diff.json) took 0.0227
seconds: four added topics, two signed-annotation changes, nine unchanged topics.
The [reviewed remap](apk/remap-evidence/2.26.37.71-subject-types.json) has thirteen
unique and two unanchored topics; full proposal creation still refuses.
[Wrong-shorty evidence](apk/remap-evidence/2.26.37.71-subject-wrong-shorty.json)
preserves all three diagnostic matches and their rejection by VLL. Current core
27-file export remains byte-identical. This is same-APK coverage expansion and
rendering correction, not a new release or completed subject receive behavior.

The [recovery follow-up](groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07)
adds five topics for twenty total group topics (25 files/448,127 bytes).
[Cached coverage comparison](apk/group-info-recovery-coverage-diff.json) took
0.0204 seconds, with fifteen previous code topics unchanged. Sixteen remaps are
unique and four helpers are unanchored. MEX QueryGroupInfo is now a concrete
missing Rust path; the recovered local monotonic guard, context enums and
callback contract identify the work needed before replacing classic lookup.
The MEX document/envelope, complete result conversion and recovery callbacks
must still be traced. Static discovery does not authorize testing invented
query identifiers or replaying uncertain requests.

## Persisted operation IDs and request ports

[The MEX trace and port](mex.md) adds selected APK asset extraction alongside
method evidence. `apk-mex-queries` exports version/hash-scoped operation records
with exact server IDs, preserving client IDs separately. Compare these small
JSON files directly for the next release; avoid scanning disassembly to patch
a data-only ID change. `mex-group-preview` invokes the actual Rust group request
builder using the selected file, making its ordered node and nested JSON
inspectable without an account. Both tools are offline. The 37.71 QueryGroupInfo
fixture is reproduced from the real APK; complete response conversion and
durable live dispatch remain the next port, not a finished group implementation.

MEX response-parts parsing and group callback gating now have offline inspection.
The [eighteen-topic MEX recipe](apk/recipes/2.26.37.71-mex.json) exports 23 files/
76,908 bytes including checksums, linked to the actual Rust module. The
[callback coverage comparison](apk/mex-callback-coverage-diff.json) adds six
methods with twelve unchanged sources in 0.0069 seconds; both sides use the
same APK, so this is coverage expansion rather than a release change. Remap
has eight unique/seven unanchored/three ambiguous mappings. Ambiguous
is_retryable/shorty-Z matches include I1H.A06 and 916.BGC/BQ2; unresolved
mappings require review before next-release patching. [Callback interpretation](mex.md#group-callback-gate-and-inherited-dispatch-2026-10-07)
distinguishes a permitted callback from complete group validity and state
application. A parser inspection or self-diff is not full APK/client parity.
