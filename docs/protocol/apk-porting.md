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
