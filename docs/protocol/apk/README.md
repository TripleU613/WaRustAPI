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
