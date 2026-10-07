# Native archive reconstruction boundary

## Evidence and result (2026-10-07)

**Confirmed/static:** the comparison APK is WhatsApp **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The ARM64 outer archive ELF (`native/arm64-libs.so` in the private extraction)
is 13,615,136 bytes, SHA-256
`24dfc7c0b9c3a13d1a9bd875304035a9f5ca6aa6f8ff904aef6ec1cfe3418c6b`.
Its `.data` begins at file offset `0x870`, length `0xcfabc0`; exported
`_superpack_archive_start` / `_superpack_archive_end` span VAs `0xc870..0xd07430`.
The `.text` size is only `0x50`: this is a container, not evidence that the
APK has no native fingerprint implementation. See [safety-number.md](safety-number.md).

**Correction/negative:** the first scoped archive search missed the separate
decoder because it searched WhatsApp/wamsys names. The APK also has
`lib/<architecture>/libsuperpack.so` for ARM64, ARMv7, x86 and x86_64.
The extracted ARM64 decoder is 220,920 bytes, SHA-256
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`;
the x86_64 decoder is 234,664 bytes, SHA-256
`481c4ce71d1070cab27bc6437c3206bb85af4f91c381bf397afa2ed7a0478a97`.
Do not repeat the earlier narrower search as proof of decoder absence.

## First compressed stream

**Confirmed/static:** the archive slice starts with hex
`0001c05001afab058100000050000000c0abcf0000000000d500000000010000`.
It contains 140 XZ magic matches. Python's standard `lzma.LZMADecompressor`,
with 256 MiB memory and 128 MiB output bounds, decodes the first stream at
archive-relative offset **6,411,720** into **6,973,276** bytes. EOF is true;
5,892,400 trailing input bytes are unused. Output SHA-256:
`6abf8ccdfe25e6b9f564b5c294e83a339c05d48a86d0e12977195f59ed37b0d3`.

Reproduce against the preserved private outer ELF, without printing content:

```python
import hashlib, lzma
from pathlib import Path
outer = Path("lab/apk-2.26.37.71/native/arm64-libs.so").read_bytes()
archive = outer[0x870:0x870 + 0xcfabc0]
assert archive.find(bytes.fromhex("fd377a585a00")) == 6411720
decoder = lzma.LZMADecompressor(memlimit=256 * 1024 * 1024)
decoded = decoder.decompress(archive[6411720:], max_length=128 * 1024 * 1024)
assert decoder.eof and len(decoded) == 6973276
assert len(decoder.unused_data) == 5892400
assert hashlib.sha256(decoded).hexdigest() == (
    "6abf8ccdfe25e6b9f564b5c294e83a339c05d48a86d0e12977195f59ed37b0d3"
)
```

**Confirmed/static:** recovered anchors at these **decoded-stream byte offsets**:

| Offset | Literal |
| --- | --- |
| `0x363ed0` | `_WESFingerprintV4CompareWithTheirIndividualFingerprints` |
| `0x363f08` | `_WESFingerprintV3CompareWithTheirIndividualFingerprints` |
| `0x3ce365` | `_WESFingerprintV4CreateIndividualParamsIfValid` |
| `0x3ce394` | `_WESFingerprintV3CreateIndividualParamsIfValid` |
| `0x3585d5` | `WESProcessedFingerprint` |
| `0x36f9ea` | `WESIndividualFingerprintParams` |
| `0x3d4abc` | `WESIndividualFingerprintParamsValidated` |
| `0x3d4ae4` | `WESIndividualFingerprintV4ParamsValidated` |

**Negative:** this output is not a reconstructed ELF. Its prefix is
`_ZNK3art6Thread13DecodeJObjectEP`. Four ELF-magic matches at decimal offsets
103488, 1131947, 4547027 and 6923444 have invalid ELF class/data bytes.
They are false positives, not usable library headers. Offsets above are
neither virtual addresses nor native function locations. Ordinary DTLS
certificate fingerprint strings are unrelated to human safety numbers.

**Inferred:** further archive-specific reconstruction is required. Meta's
[Superpack engineering description](https://engineering.fb.com/2021/09/13/android/superpack/)
describes instruction-aware transforms and a modular compression pipeline;
it supports this general expectation, not an exact WhatsApp format recipe.

## Decoder entry points

**Confirmed/static, x86_64 decoder only:** `AssetDecompressor` native signatures
and ELF relocation/disassembly identify `decompress_from_so` at VA `0xc996`,
signature `(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)[Ljava/lang/String;`.
It delegates to `decompress_range_from_so` at `0xc502` with both range bounds
set to -1; the latter has two additional integer arguments. These are **not
ARM64 addresses**.

At `0xc595..0xc59d`, the range entry calls `dlopen` with flags 6; a fallback
uses `android_dlopen_ext` at `0xc640`. At `0xc651..0xc672` it resolves the
archive start/end exports (string VAs `0x3140`, `0x3e02`), calculates their
difference at `0xc75a`, and calls `0xd38e` at `0xc790`. At `0xc79b..0xc7ac`
it compares the second Java string to `spo` (string VA `0x38d8`); its refusal
literal is `so file must be spo` (`0x38c8`). It caps the sysconf-derived thread
count at four (`0xc86f..0xc894`) before calling `0xd3ac` (`0xc899..0xc8b3`).
`JNI_OnLoad` (`0xbb40`) registers native classes and calls initialization
functions `0xdd90`, `0xdc00`, `0x1d220` at `0xbba2..0xbbb1`.

**Unknown:** reconstruction internals, complete argument semantics, ARM64
entry addresses, JNI callbacks needed for reconstruction and modern fingerprint
algorithms. The first string is **inferred** to be the source library path
from its dlopen use; a third-string output-directory interpretation remains
unconfirmed. Imports include Android/log/dl/libc and Bionic `LIBC` versions.
**Inference, not a failed experiment:** substituting GNU libc is unsafe without
ABI analysis, including pthread layout. No such host shim was tried.

Private derivation files are under `lab/apk-2.26.37.71/native/`: original
decoders/container, `first-xz-decoded.bin`, `x86_64-superpack-disassembly.txt`
and `asset-decompressor.smali`. Raw libraries/trees stay outside Git.

## Bounded loader check

**Observed/local tooling:** Qiling 1.4.6 with CPython 3.12.14 runs the public
ARM64 Android hello executable to an observed `exit_group(0)` using
[native_loader_check.py](../../tools/apk-research/native_loader_check.py).
This only establishes an ELF-loader baseline; the WhatsApp decoder has not
been invoked. No WhatsApp app, account, device or network session runs here.

The four fixtures come from the official
[Qiling rootfs repository](https://github.com/qilingframework/rootfs/tree/df3fa4dfc0b9d4164f8678699d8923df847eb3d2/arm64_android),
revision `df3fa4dfc0b9d4164f8678699d8923df847eb3d2`:

| Relative path | Bytes | SHA-256 |
| --- | ---: | --- |
| `bin/arm64_android_hello` | 5816 | `7be49a4e982036b1ace09b337d36cd02a8333379bacbed886783b68cf424c8d7` |
| `system/bin/linker64` | 1262416 | `67cc9a74ef63401ecaea2a527cb2207187c7386676f680ad7590c9f8115a6227` |
| `system/lib64/libc.so` | 1035824 | `4e6d025f3356a3353418981e475561cfa133a0e439a20c9d9bede715ae167697` |
| `system/lib64/libdl.so` | 10688 | `e52d3d9b4267e37987f157d4996a3f5d1ca007dab0f217f959921e3d1c5cfe13` |

**Negative/setup:** Python 3.14 installation failed because a dependency's
Pillow 10.4 C extensions are incompatible; a private Python 3.12 environment
resolved it. Single-threaded loading failed at futex (`thread_management=None`).
Multithreaded loading then failed because Qiling's `newfstatat` did not resolve
`/proc/self/exe`. The helper supplies real executable metadata only for flags=0
on that exact path; all other calls retain the original syscall implementation.
It uses an empty guest environment and bounded execution, requiring a recorded
successful guest exit rather than treating an emulator timeout as success.

**Observed limitations:** sigaltstack is unimplemented; `/system/etc/ld.config.txt`
is absent (linker defaults used); `/proc/self/fd/{3,4}` readlink warns and the
given library paths are used; the property client reports failed loading.
These warnings are retained. Full Android runtime compatibility, superpack
reconstruction, native algorithms and human safety-number verification remain
**unknown/unimplemented**. Searches of public extraction tools did not establish
a working portable decoder; device-based workflows were not used.

**Validation for this tooling/docs change:** the checked-in helper observed
`exit_group(0)`; missing-rootfs diagnostics exited 2. The bounded XZ reproducer
and all eight anchor offsets were rechecked. CLI reference regeneration,
OpenAPI JSON parsing, Python source compilation, staged diff whitespace checks
and repository hygiene passed (255 local links, 564315 research-checksum bytes).
No Rust runtime code changed; workspace results in STATUS.md are from the
preceding Rust change, not a newly repeated test run.
