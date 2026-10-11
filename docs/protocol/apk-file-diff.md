# Whole-APK file indexes and cached differences

2026-10-10. **Implemented/offline developer tooling.** `apk-file-index` records
SHA-256 and uncompressed byte length of every non-directory ZIP entry, plus the
whole APK SHA-256 and byte length. `apk-file-diff` compares two cached indexes
and reports added, removed and changed paths in sorted order. A changed row
contains old/new byte lengths and SHA-256; absent sides are null. The existing
[research-topic workflow](apk/README.md#version-scoped-recipes-and-fast-cached-differences)
continues to map reviewed methods to Rust files.

## Generate once, compare quickly

The complete [2.26.37.71 baseline](apk-files/2.26.37.71.json) is committed.
Its APK SHA-256 is
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
It contains 14,967 files / 212,245,592 uncompressed bytes and no file bodies.
It can be compared without retaining the old APK on the user's machine.

```sh
cargo build --locked -p xtask
mkdir -p lab/apk-file-index/next-release
target/debug/xtask apk-file-index /path/to/new-WhatsApp.apk \
  > lab/apk-file-index/next-release/index.json
target/debug/xtask apk-file-diff \
  docs/protocol/apk-files/2.26.37.71.json \
  lab/apk-file-index/next-release/index.json \
  > lab/apk-file-index/next-release/diff.json
jq '.files[] | {path, status, old, new}' lab/apk-file-index/next-release/diff.json
```

Use fresh output paths for each release. The commands emit JSON to stdout and
leave source APKs/indexes unchanged. Files are never extracted to disk or run.
Hashing reads every uncompressed byte, including media and native containers;
ZIP CRC or compressed length alone is not treated as content equality. Directory
entries, compression settings and ZIP metadata are not file content. APK
container changes remain visible separately through `apk_container_changed`,
even when entry hashes all match. This includes changes outside ZIP entries,
but does not decode or verify APK signing blocks or signature authenticity.

The [actual local validation](apk-file-index-validation.json) measured an
8.107389-second first index and 0.211419-second cached self-diff in the debug
build. Indexing is a one-time cost per APK. Cached comparison read 1,983,427-byte
indexes, reported all 14,967 unchanged files and matched the expected whole APK
hash. These are local measurements, not a guarantee on other hardware. The
committed copy also compares equal to the generated baseline. A different
release's actual content changes have not yet been validated here; changed,
added, removed and repacked archives are tested with independent synthetic ZIPs.

## From changed files to a Rust patch

Changed classes*.dex entries identify where method-level investigation is
needed; resources and native entries remain their own paths. ZIP file changes
alone do not establish which protocol behavior changed. For a mapped topic:

```sh
target/debug/xtask apk-research-remap /path/to/new-WhatsApp.apk \
  --recipe /path/to/old-reviewed-recipe.json \
  --output lab/apk-file-index/next-release/proposed-recipe.json
# Review the unique anchor matches and callers before using the proposal.
target/debug/xtask apk-research-bundle /path/to/new-WhatsApp.apk \
  --recipe lab/apk-file-index/next-release/proposed-recipe.json \
  --output lab/apk-file-index/next-release/topics
target/debug/xtask apk-research-diff /path/to/old-topic-bundle \
  lab/apk-file-index/next-release/topics
```

Remapping refuses a proposal when a topic is missing, ambiguous or unanchored.
Topic diffs retain Rust-file mappings; inspect their actual changed instructions
with apk-smali/REA before patching the corresponding code and regressions.
Unmapped classes, native superpack internals, runtime properties and server
behavior remain outside those topic mappings. The file index nevertheless
covers their outer archive entries. No method is declared understood just
because its DEX file was indexed; no Rust patch, APK profile update, key
rotation or server operation occurs automatically.

For a virtual method, the [exact superclass override census](self-read-eligibility.md)
also compares full DEX signatures and declared subtype bodies. Remap the root and
method first; scan each APK with `apk-virtual-overrides` and retain the results
beside the reviewed topic diff. Inherited behavior is not determined by inspecting
only a base declaration.

## Input validation and bounds

The reader uses streaming SHA-256 with a 64 KiB buffer and validates ZIP reads
through EOF, including their CRC check. Duplicate file names cannot be hidden by
ZIP name lookup: entries are visited by numeric index. Files need canonical UTF-8
relative names: no empty/dot/parent components, backslashes or control characters.
Malformed archives, corrupt entry data and unsupported compression fail before
JSON is printed. Encrypted ZIP entries are not decrypted. Local bounds are
512 MiB per APK/file, 2 GiB total expanded content, 100,000 entries, 1,024 bytes per
path and 32 MiB per JSON index; these are tooling limits, not WhatsApp limits.

Cached indexes require format 1, valid lowercase SHA-256 fields, bounded sizes,
valid paths and unique JSON file keys. Unknown fields and malformed metadata
refuse. Indexes are unsigned evidence exports; use unmodified generated files.
Validation cannot authenticate an edited index against an absent source APK.
Retain the whole APK hash and reviewed provenance with each new release.

## Validation

The first content-diff regression failed before implementation. Three tooling
tests now cover additions/removals/changes, unchanged file content after ZIP
reordering, duplicate ZIP names, unsafe paths, corrupt stored data, streaming
size refusal, bad cached metadata/path and duplicate JSON keys. Duplicate cached
keys failed before the custom map reader was added. The real APK export and
committed/generated baseline comparison passed without account or network use.

905 workspace/all-feature tests passed, including 76 tooling tests. Strict
all-target Clippy, warnings-denied Rustdoc, default CLI/API-symbol tests and the
CLI-reference check passed. The default build retains its existing unused
clock::observe_login warning. This is file-level coverage, not complete reverse
engineering, cross-release compatibility or live client validation.
