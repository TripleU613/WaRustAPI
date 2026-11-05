# Superpack loader and bounded static decompression — APK 2.26.43.72

**Confirmed static**, checked DEX TXT and bounded Python lzma output; not target
execution, a native function trace, full archive reconstruction or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected loader/API exports and probe script/output are retained.
No APK, carrier, decompressed payload or bulk native tree is committed.

## Selected loader entry and decoder boundary

The exact superpack literal inventory finds WhatsAppLibLoader.BbD. Its complete
body includes an ABI loop constructing libsuperpack.so paths from a matching
split_config ABI APK or the base sourceDir, and a separate fallback library-name
load path. It invokes AssetDecompressor.testDecompressorLibraryUsable with a
three-byte array and compares against an expected three-byte payload. Selected
UnsatisfiedLinkError handlers log/fallback or return false; exact bodies and
exception ranges are retained. ABI selection, all upstream loader gates and
recovery helpers are not fully reconstructed here.

AssetDecompressor exposes decompress, decompress_from_so, decompress_legacy,
range and reference variants, architecture and sync operations, plus the probe.
Their selected exports contain no DEX implementation; the Java API does not
supply a pure-Java archive decoder. The word superpack alone is not enough to
substitute an unrelated similarly named serialization format. Meta's public
[SoLoader repository](https://github.com/facebook/SoLoader) also identifies
Superpack compression as a loader use case; that external context is not proof
of this APK's inner archive format or compression algorithm.

Reproduce the DEX bodies with `xtask apk-smali APK --class CLASS [--method METHOD]
--signatures --exceptions`; the literal inventory uses apk-xref --anchor superpack.
The API export contains12 methods and the exact loader method isBbD()Z.

## First twelve XZ candidates are not recovered ELF libraries

Run `python3 probe-xz.py APK`. It verifies both APK and selected ARM64 carrier
SHA256, takes the previously [measured file range](../2.26.43.72-metrics-native-carrier/README.md)
0x870..0xdb6eb1 (size0xdb6641), and searches for exact XZ magic. Offsets in
xz-probe.json are relative to that range, not ELF virtual addresses or guessed
member indices. There are140 magic matches; this bounded pass probes only the
first12, each with128MiB decoder memory and64MiB maximum output.

All12 reach XZ end-of-stream within those bounds. The output records exact
consumed compressed bytes, output size, digest and whether output starts with
ELF magic. None starts with ELF. This establishes those selected streams decode;
it does NOT identify their archive member names, restore an ELF header/sections,
reconstruct JNI symbols or prove outputs are invalid/unusable. Additional
transforms, reference data, metadata parsing and unprobed streams remain open.
No native payload was loaded, invoked or instrumented.

The probe was independently rerun from the pinned APK through the retained
script and matched the initial12 metadata results exactly. It produces metadata
only, so a future pass cannot mistake its result JSON for usable inner binaries.
No arbitrary filename match is promoted to a recovered member identity.

## Next proof obligations

Trace the actual archive metadata/transform and reference handling, recover
complete inner libraries with verified boundaries/digests, resolve JNI
registration, then analyze the dispatcher to metrics schema/buffer/transport,
responses and durable recovery. The currently unavailable native REA providers
and this partial decode are documented constraints, not a claim that further
static work is impossible. No fabricated telemetry, Rust sender/retry, HTTP
route or live account/network operation is added.
