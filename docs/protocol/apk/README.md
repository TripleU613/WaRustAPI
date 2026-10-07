# Compact APK research snapshot

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
It does not reconstruct control flow, exception tables, or original names.
Use the APK and existing `apk-smali`/`apk-xref` commands for deeper analysis.

This is a focused snapshot of currently mapped protocol classes, not a full
decompilation or a complete WhatsApp implementation. UI and unrelated code
are omitted. Constants compiled into an APK are client-static evidence;
they do not prove server acceptance or promote the active 2.26.36.75 profile.
The token dictionary differs from the active profile and is not installed
into the library. Existing curated findings remain authoritative for what
has actually been interpreted and verified.

## Version-scoped recipes and fast cached differences

`apk-research-bundle --recipe <json>` supports a different release only when the
recipe's exact SHA-256 matches. Each stable topic maps to an exact class, optional
method substring filter and relative affected Rust paths. Unknown fields and
unsafe names/paths refuse; absent/ambiguous classes refuse before writing. The
output directory and 2 MiB media-free constraints remain. Default 37.70 selectors
are unchanged.

The new [37.71 group recipe](recipes/2.26.37.71-groups.json) reproduces
[its scoped baseline](2.26.37.71-groups/manifest.json): 13 files/397,778 bytes.
This baseline includes eight selected investigation targets, not all APK code.
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
