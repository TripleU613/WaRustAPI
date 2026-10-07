# Native archive reconstruction boundary

**Current result:** the ordinary unmodified 37.71 ARM64 decoder completes under
QEMU: 80 returned entries, 79 unique valid ELF libraries, independently reproduced
with matching hashes. The duplicate entry is `libsodium.so`. The Qiling partial
range failure and full-run timeouts below are historical tool-specific results,
not evidence that the archive cannot be reconstructed. Native fingerprint
algorithms and active UI scheme selection still require tracing.

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
The initial hello check only established an ELF-loader baseline. The follow-up
below loads and invokes the APK decoder locally. No WhatsApp app, account,
device or network session runs here.

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
It uses an empty guest environment and requires a recorded successful guest
exit rather than treating an emulator timeout as success. The first helper's
claimed execution bounds were refuted and corrected as recorded below.

**Observed limitations:** sigaltstack is unimplemented; `/system/etc/ld.config.txt`
is absent (linker defaults used); `/proc/self/fd/{3,4}` readlink warns and the
given library paths are used; the property client reports failed loading.
These warnings are retained. Full Android runtime compatibility, successful
superpack reconstruction, native algorithms and human safety-number verification
remain **unknown/unimplemented**. Searches of public extraction tools did not establish
a working portable decoder; device-based workflows were not used.

**Validation for this tooling/docs change:** the checked-in helper observed
`exit_group(0)`; missing-rootfs diagnostics exited 2. The bounded XZ reproducer
and all eight anchor offsets were rechecked. CLI reference regeneration,
OpenAPI JSON parsing, Python source compilation, staged diff whitespace checks
and repository hygiene passed (255 local links, 564315 research-checksum bytes).
No Rust runtime code changed; workspace results in STATUS.md are from the
preceding Rust change, not a newly repeated test run.

## Follow-up: ARM64 decoder execution

**Confirmed/static, same APK:** the original ZIP entries are
`lib/arm64-v8a/libs.so` (outer container) and `lib/arm64-v8a/libsuperpack.so`
(decoder); their hashes match the table above. ARM64 JNI relocations identify:

| Name | Function VA | Name/signature/function relocation slots |
| --- | --- | --- |
| `decompress_range_from_so` | `0x9a50` | `0x380e8`, `0x380f0`, `0x380f8` |
| `decompress_from_so` | `0x9ec8` | `0x38118`, `0x38120`, `0x38128` |

The range signature is
`(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;II)[Ljava/lang/String;`.
At `0x9ec8..0x9ed0`, the simpler entry passes both integer bounds as -1.
At `0x96d0..0x9760`, JNI registration finds the AssetDecompressor class
(literal `0x34dc`), checks exceptions, copies the eight-method table and
registers it. `JNI_OnLoad` is at `0x8f30`; successful registration proceeds
through initialization calls `0xb43c`, `0xb288`, `0x1b5d4`
(`0x9058..0x9060`). `0x9cdc..0x9cec` checks literal `spo` (`0x39a4`).
The reconstruction wrapper is `0xa964`; its index/end comparison
`0xaa0c..0xaa14` makes the upper bound exclusive. The third Java string is
passed through as the writer's path input (`0x9e84`, `0xa9a4`, `0xaa48`);
exact path-join internals remain **untraced**.

**Observed/local:** the public rootfs lacks libandroid/liblog, so the initial
load returned guest exit 2 and `dlopen failed: library "libandroid.so" not found`.
Own-source [asset imports](../../tools/apk-research/native-android-imports.c)
abort when called; [log imports](../../tools/apk-research/native-log-imports.c)
render diagnostics using Bionic vprintf. With those imports, the unmodified
APK decoder loads and resolves `JNI_OnLoad`. The
[probe](../../tools/apk-research/native-probe.c) then supplies a limited local
JNI adapter: known class selectors, native registration and ASCII strings/arrays
only. All unsupported callbacks abort. This is not a Java VM; unused adapter
objects live until the isolated worker exits. The standard table-slot layout
comes from Oracle's [JNI Functions](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/functions.html)
and [Invocation API](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/invocation.html).

The decoder returned `JNI_OnLoad=0x10006`, registering 8 AssetDecompressor,
6 WhatsAppObiInputStream and 5 WhatsAppOpenboxArchive methods (19 total), and
the guest exited zero. This proves local loading/initialization only. The
container is preloaded with flags 2 because the APK's flags 6 request
NOW|NOLOAD on ARM64; those values are documented in
[Bionic dlfcn.h](https://android.googlesource.com/platform/bionic/+/refs/heads/main/libc/include/dlfcn.h).

**Negative/fixed harness:** an initial range call used a C main argument named
`env` that shadowed the adapter pointer, causing an unmapped read at native
`0x9a84`. Renaming the argument/global adapter removed that error; this was
our harness bug, not a format rejection. Another initial load hit Qiling's
`munmap`/`del_mapinfo` IndexError. The helper now returns EINVAL for unaligned
or zero-length calls and zero for a valid wholly unmapped range, delegating
mapped ranges to the original handler. Linux explicitly allows unmapping an
unmapped range ([munmap manual](https://man7.org/linux/man-pages/man2/munmap.2.html)).
Partial mappings/shared-file handling remain the original Qiling behavior;
this adapter is not a complete syscall implementation.

**Correction/observed tooling:** Qiling 1.4.6 `os/linux/linux.py:run` delegates
to `thread_management.run`; `thread.py:_run` invokes `emu_start` in fixed
31337-instruction slices without the supplied run timeout/count. Therefore
the original `run(timeout=10000000, count=1000000)` did not establish bounds.
The helper now runs a child under a parent subprocess wall timeout and charges
ARM64 basic blocks conservatively before execution, across all guest threads.
Defaults are 10 seconds/1 million; allowed ranges are 1–60 seconds/1–1 billion.
Timeout, instruction exhaustion, missing exit and nonzero guest exit all fail.
Integration tests prove successful hello, actual guest instruction exhaustion,
missing-fixture refusal and timeout of an actual infinite ARM64 guest.

**Observed negative reconstruction:** range `[0,1)`, type `spo`, destination
`/decoded` reaches the archive work queue but returns native `Could not open
archive: 9/2`, then `AssetDecompressionException: could not extract file from
archive`, guest exit 7. The output directory remains empty. Diagnostics identify
stream 6 with expected/returned size 125448 and 163 incomplete work items.
Equal printed sizes do not establish valid reconstructed content. The APK
worker (`0x1ebe4`) checks output size at `0x1ef00..0x1ef08` and also conditionally
checks a hash (`0x1ef0c..0x1ef30`); the error branch/log is `0x1f044..0x1f094`.
The precise failing branch and cause are **unknown**; no checksum verification
was disabled or failure forced to success.

The 50-million instruction check stopped before the native result. With
1 billion/30-second bounds, the completed failed attempt charged 123367393
instructions. Explicitly mapping actual host `/proc/meminfo` removed the
missing-memory warning but yielded the same error/empty destination, charging
123365694 instructions. Missing meminfo alone is **refuted** as the explanation.
This metadata is only for the local decoder's resource estimate, not Android
device provenance or account protocol data. An initial adapter used the wrong
Qiling API owner (`ql.os.add_fs_mapper`); `ql.add_fs_mapper` is the working API.

Reproduce with the [helper setup/commands](../../tools/apk-research/README.md).
Private evidence includes `arm64-superpack-disassembly.txt`,
`arm64-decompress-entry.txt`, `arm64-reconstruct-wrapper.txt`,
`arm64-file-writer.txt` and the rootfs under the native lab directory.
No library was reconstructed and no modern fingerprint algorithm recovered.
The next useful step is tracing the native worker's actual failure branch,
not replacing identity pins or repeating account requests.

**Follow-up validation:** all three checked-in C sources were cross-compiled
against the pinned Bionic fixtures, and the resulting initialization probe
exited zero (985744 charged instructions). Four integration checks passed with
none skipped. Python source compilation, OpenAPI parsing, CLI-reference
regeneration, Rust formatting and diff whitespace checks passed. The two
completed reconstruction failures above are negative evidence, not passing
extraction tests. Repository hygiene passed with 261 local links and 564315
research-checksum bytes. Rust runtime sources were unchanged. Diagnostic logs
are preserved privately in native `execution-checks-2026-10-07/`; this includes
the original failed attempts rather than only their corrected successors.

## Follow-up: requested bytes versus declared stream size

**Observed/local, same unmodified ARM64 decoder:** a bounded read-only worker
trace reaches `0x1ef00`, `0x1ef40`, `0x1efa4`, then `0x1f044` for stream 6,
type 42. Produced bytes are 125448, but task `+0x20` is **zero**. Task
`+0x18` is 125448, and `+0x50` (conditional hash-check flag) is zero.
Thus the failure is the oversized-file branch against the requested count;
it is **not** a traced hash mismatch or failure to produce the declared size.
The previous general statement that equal printed sizes did not identify the
branch is superseded by this trace. The error and comparison are preserved;
no task field or native return value was patched.

**Confirmed/static:** producer `0x1e544..0x1e564` writes stream index/type at
`+8/+12`, declared length at `+0x18`, requested length at `+0x20`, and the
equality-derived hash-check flag at `+0x50`. At `0x1e3f8..0x1e424`, the
requested size initially equals declared size; context `+0xc8` can instead
select an indexed size from the stack table using context `+0xcc`. Worker
`0x1ef00..0x1ef08` compares produced to requested. The failure diagnostic
`0x1f044..0x1f078` prints declared size as "Expected" and requested count
separately, explaining why the two prominent printed lengths agreed.

**Inferred/unknown:** the partial range recipe likely activates the indexed
size selection. The flag's complete setter/caller chain, why a zero requested
count reaches this file handler, and whether this is valid for partial ranges
remain untraced. The `[0,1)` result cannot establish that ordinary full-archive
extraction fails. The harness now also captures `decompress_from_so` by its
registered signature and calls that ordinary entry without a partial range.

**Observed negative, not a native refusal:** the first full-archive check used
1 billion/60-second bounds and was terminated by its parent wall limit, with
no output. A subsequent 30-second worker trace likewise timed out after
progress beyond 100 million charged instructions. The optimized branch/futex
observer reached beyond 200 million before its 30-second limit, with no
recorded worker failure branch or file. These results establish insufficient
completed execution, not the same native `9/2` response as the range attempt.

`--trace-worker` requires the exact 37.71 ARM64 decoder hash before interpreting
any native offsets. It reports at most 64 exceptional branch records with
stage/offset, stream/type, declared/requested/produced byte counts and the
hash-check flag. Futex observation reports at most 64 state transitions using
local selector numbers rather than addresses; consecutive duplicates are
collapsed. It preserves the underlying syscall/scheduler behavior. Progress is
reported each 100 million conservatively charged instructions. Other builds
are refused instead of being interpreted with this layout.

The full-archive mode retains the empty-destination guard. Integration coverage
now also proves both extraction modes preserve partial output and tracing
refuses an unknown decoder before execution. Execution bounds remain enforced
by the same mechanisms; the configurable ceilings are now 600 seconds and
10 billion instructions, with unchanged 10-second/1-million defaults, because
full reconstruction exceeded the initial short research windows. These are
local tooling budgets, not server limits.

**Observed extended execution:** a 150-second full-entry run progressed past
1.2 billion charged instructions before its wall timeout, without a traced
failure or file. A subsequent full-entry attempt is bounded at 600 seconds/
10 billion instructions. Its outcome must be inspected from the live process
and output; the larger budget is not evidence of extraction success.

**Confirmed/static path construction:** `0x33628..0x33704` measures both path
components, checks a 4096-byte combined-length boundary and formats literal
`%s/%s` at `0x33f5`. Writer `0xa774` supplies the third input string as the
first component and the archive entry name as the second. This resolves the
previously untraced join: `/decoded` produces `/decoded/<entry-name>` rather
than concatenating the two without a separator.

## Ordinary full extraction succeeds under QEMU

**Observed/local:** installed QEMU **11.1.1** loads the own-source probe and
the exact unmodified 37.71 decoder against the same pinned Bionic fixtures.
Initial attempts returned `Exec format error` because the private probe/linker
copies lacked executable mode. Making those two copies executable resolved
it; relative interpreter prefixes also worked after that correction. No ELF
bytes, account identity or protocol input were changed. The public hello was
only tried before this correction; its QEMU success is not claimed.

A first isolated attempt failed because `/qemu` did not exist under a read-only
root mount. Preparing an empty mount point resolved it. The checked-in
[QEMU runner](../../tools/apk-research/native_qemu_check.py) handles these local
fixture prerequisites, validates the exact decoder/container hashes, and uses
Bubblewrap with network/PID namespaces, an empty environment, a read-only
fixture root and only `/decoded` writable for extraction. It runs the decoder
probe, not WhatsApp or any account. Process timeout is enforced by the parent;
Bubblewrap's die-with-parent behavior terminates the worker. QEMU has no
instruction counter in this helper. The configurable wall limit defaults to
30 seconds and allows 1–600 seconds. QEMU's
[official user-mode documentation](https://www.qemu.org/docs/master/user/main.html)
describes syscall translation and host-backed guest threading; it does not
establish Android application/runtime compatibility.

**Observed/local positive:** full `decompress_from_so`, type `spo`, destination
`/decoded`, returned 80 names and process exit zero within the 30-second bound.
The list contains `libsodium.so` twice, leaving 79 unique files totaling
**62,743,576 bytes**. All 79 parse as little-endian 64-bit `EM_AARCH64`, `ET_DYN`
ELFs; every program segment's file extent lies within its file. This is actual
ELF reconstruction, unlike the earlier bare XZ stream. The duplicate entries'
original byte equality is **unknown**; only the final unique file is retained
by the official writer. Do not silently report 80 distinct libraries.

The checked-in wrapper reproduced full extraction into the original Qiling
fixture root, exited zero, and all 79 hashes independently match the first
run. Repeating extraction into that populated directory refuses locally;
partial/existing files are not deleted or overwritten. Original APK, decoder,
container and failed-attempt logs remain preserved. The slower Qiling attempt
was terminated after QEMU success (its process handle completed with 143);
it had continued past 2.8 billion charged instructions without an output.
This was an intentional cancellation of a redundant local comparison, not a
native refusal or a completed Qiling extraction.

| Reconstructed target | Bytes | SHA-256 |
| --- | ---: | --- |
| `libwhatsapp.so` | 13790464 | `f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a` |
| `libwhatsappmerged.so` | 7726208 | `565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa` |

Private outputs are in native `qiling-range-trace-rootfs/decoded/` and
`qiling-rootfs/decoded/`; `extracted-libraries-manifest.json` records every
unique file's hash/size and target anchors. They are ignored raw libraries,
not committed research/media bundles. Only the own-source harness, evidence
and reproducible command surface are tracked.

## Fingerprint anchors in a reconstructed ELF

**Confirmed/static, reconstructed `libwhatsapp.so`, same build/hash above:**
the four earlier strings now have real ELF `.rodata` file offsets/VAs. They
have no matching dynamic export or direct `.rela.dyn` addend; absence there
does not mean the routines are missing. ARM64 ADRP/ADD references and compiler
unwind ranges establish the following containing code locations:

| Anchor | String file offset/VA | Example ADRP/ADD VAs | Containing unwind range |
| --- | --- | --- | --- |
| V3 create individual params | `0x11f558` | `0x71333c`, `0x713340` | `0x7132b4..0x713620` |
| V4 create individual params | `0x11f529` | `0x7147cc`, `0x7147d0` | `0x7145d0..0x71493c` |
| V3 compare individual fingerprints | `0xb50cc` | `0x714328`, `0x71432c` | `0x7140fc..0x714358` |
| V4 compare individual fingerprints | `0xb5094` | `0x7155a4`, `0x7155a8` | `0x7152fc..0x7155d8` |

These are the literal names recorded in the earlier anchor table, shortened
here for readability. Range ends are exclusive. The unwind ranges identify
compiler code extents containing those references, not a recovered C signature
or proof of a complete fingerprint algorithm. They must not be confused with
the first-XZ-stream offsets. Compact private disassemblies are
`fingerprint-v3-create.txt`, `fingerprint-v4-create.txt`,
`fingerprint-v3-compare.txt`, `fingerprint-v4-compare.txt`, plus
`fingerprint-anchor-xrefs.txt`. The next step is tracing their input validation,
producers and cryptographic calls, then the Java/wamsys dispatch mapping.

## Validation of the full-reconstruction tooling change

**Observed/local:** all eight Python runner regressions passed with the public
rootfs configured (no skipped cases). Separate actual QEMU initialization
registered 19 JNI methods and exited zero. A populated-destination check
returned exit 2 and preserved all 79 existing file hashes. The independently
reproduced full extraction described above is the positive reconstruction
check; the regression suite itself does not execute the APK decoder.

CLI reference regeneration/test, workspace formatting and repository hygiene
passed. No Rust runtime behavior changed in this tooling update. The failed,
cancelled and successful execution logs are retained privately under
`lab/apk-2.26.37.71/native/execution-checks-2026-10-07/`; native outputs and
the complete per-file manifest remain ignored local artifacts. No account
store was opened and no live WhatsApp request was sent.
