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
automatic anchor-assisted remapping with ambiguity diagnostics, normalized
method-level comparison where sound, protobuf/token/config extraction recipes,
and native-code change attribution. UI/media payload extraction is unnecessary
for protocol reproduction. Bulk analysis trees and native binaries stay local;
compact attributable evidence and interpreted code stay in Git.
