# APK research helpers

Small offline helpers used on the full Dalvik disassembly of the comparison APK
(`./target/release/xtask apk-smali ~/Downloads/WhatsApp.apk --class "" > dump.txt`,
about 570 MB for build 2.26.37.71). The DEX helpers read their supplied dump;
the native helpers below use explicitly supplied local fixtures. Neither sends
network requests.

- `dumpm.py DUMP ANCHOR [ANCHOR...] [--max-methods N] [--max-lines N] [--full]`
  prints every method whose body contains a `const-string` equal to an anchor,
  in a compact view (strings, calls, constants). Use it to find the builder of a
  stanza from an attribute or namespace string; remember that names in the
  binary token dictionary (`crates/whatsapp-api/fixtures/tokens/web-md-v3.json`)
  may still appear as string constants in builders, but not always.
- `desc.py DUMP [CLASS...] [--anchor FIELD_NAME...]` parses protobuf-lite
  `dynamicMethod` descriptors (RawMessageInfo strings) into `(field name, number,
  type)` triples. Field names are consumed in declaration order; for very large
  messages the name alignment can drift after fields whose names are not string
  constants, so cross-check numbers against live captures or the public schema.

Evidence produced with these tools is written up in `docs/protocol/*.md` with
the class names of the build it came from.

## Public native-loader baseline

`native_loader_check.py --rootfs PATH` runs only the public Qiling ARM64 Android
hello fixture by default, with an empty guest environment. It requires an
observed `exit_group(0)`. Missing fixtures/dependencies
are diagnosed before execution. Rootfs/tool environments stay in ignored `lab/`.

`--timeout-seconds` is a parent-process wall limit (default 10, range 1–600).
`--max-instructions` charges whole ARM64 basic blocks across all guest threads
before execution (default 1000000, range 1–10000000000). Qiling's threaded runner
ignores its ordinary `run(timeout, count)` arguments; those alone did **not**
enforce the bounds claimed by the first version of this helper. Both guards now
have integration checks, including a real infinite ARM64 guest. A limit or
missing/nonzero guest exit fails the check.

Setup used locally (Python 3.14 failed in Pillow; Python 3.12 works):

```sh
uv venv --python 3.12 lab/native-analysis-tools-py312
uv pip install --python lab/native-analysis-tools-py312/bin/python qiling==1.4.6
```

Download these four public paths into
`lab/apk-2.26.37.71/native/qiling-rootfs/`, preserving the directory layout:
`bin/arm64_android_hello`, `system/bin/linker64`, `system/lib64/libc.so`,
`system/lib64/libdl.so`. Use the source prefix
`https://raw.githubusercontent.com/qilingframework/rootfs/df3fa4dfc0b9d4164f8678699d8923df847eb3d2/arm64_android/`.
Verify their hashes against [the provenance table](../../docs/protocol/native-unpacking.md)
before running:

```sh
lab/native-analysis-tools-py312/bin/python tools/apk-research/native_loader_check.py \
  --rootfs lab/apk-2.26.37.71/native/qiling-rootfs
```

The narrow `/proc/self/exe` stat adapter uses actual executable metadata, only
with flags=0; it does not invent system/device properties. Unimplemented
sigaltstack, linker-config/proc-fd warnings and failed property-client loading
remain visible. A passing hello check does not establish decoder compatibility.

## ARM64 decoder probe

`--check decoder-init` loads the exact 37.71 decoder and calls `JNI_OnLoad`
through the small local JNI adapter in `native-probe.c`. It requires version
`0x10006`, 19 registered methods and exit zero. This is an artifact harness,
not a Java VM or an account client. Unsupported JNI/asset calls abort; the
string subset is ASCII fixture paths. `native-log-imports.c` renders diagnostics
with Bionic vprintf, without an Android logging service.

Build the adapters and probe with Clang/LLD using the pinned Bionic fixtures:

```bash
native_probe_root=lab/apk-2.26.37.71/native/qiling-rootfs
native_probe_flags=(--target=aarch64-linux-android21 -fuse-ld=lld -nostdlib
  -Wl,-z,max-page-size=4096 -L"$native_probe_root/system/lib64")
clang "${native_probe_flags[@]}" -fPIC -shared -Wl,-soname,libandroid.so \
  tools/apk-research/native-android-imports.c -l:libc.so \
  -o "$native_probe_root/system/lib64/libandroid.so"
clang "${native_probe_flags[@]}" -fPIC -shared -Wl,-soname,liblog.so \
  tools/apk-research/native-log-imports.c -l:libc.so \
  -o "$native_probe_root/system/lib64/liblog.so"
clang "${native_probe_flags[@]}" -fPIE -pie -Wl,-e,_start \
  -Wl,--dynamic-linker=/system/bin/linker64 tools/apk-research/native-probe.c \
  -l:libc.so -l:libdl.so -o "$native_probe_root/bin/decoder_probe"
cp lab/apk-2.26.37.71/native/arm64-v8a-superpack.so \
  "$native_probe_root/system/lib64/libsuperpack.so"
lab/native-analysis-tools-py312/bin/python tools/apk-research/native_loader_check.py \
  --rootfs "$native_probe_root" --check decoder-init --max-instructions 2000000
```

`--check decoder-first-entry` additionally requires the APK's
`lib/arm64-v8a/libs.so` copied to rootfs `system/lib64/wa_archive.so` and an
empty rootfs `decoded/` directory. It preloads the container, then invokes the
registered `decompress_range_from_so` with `spo`, `/decoded`, range `[0,1)`.
This may write local reconstruction output; it refuses a nonempty destination
and does not delete partial results. Selecting one output still requires
internal shared archive work. Tested bounds: `--max-instructions 1000000000
--timeout-seconds 30`. This attempt **fails**, with native `9/2` and an
`AssetDecompressionException`; no file was reconstructed. A 50-million budget
was insufficient even to reach that outcome. This Qiling range mode does not
establish the ordinary full extraction outcome; QEMU succeeds as described below.

`--check decoder-full-archive` invokes the registered ordinary
`decompress_from_so` entry without partial-range bounds, using the same required
container and empty destination. The range failure compares returned length to
a zero requested-byte count, whereas its prominent diagnostic prints the full
declared size. It does not establish the ordinary full-archive outcome. Initial
full checks exceeded 30/60-second limits without a native refusal or output.

`--trace-worker` requires the pinned 37.71 ARM64 decoder hash and an extraction
check. It emits exceptional branch stage/offset, stream/type, declared/requested/
produced counts and the hash-check flag, capped at 64 records. At most 64 futex
state transitions use local selector numbers instead of addresses; consecutive
duplicates are collapsed. Charged instruction progress prints each 100 million.
It observes without altering native comparisons, returns or syscall behavior.
Unknown decoder builds are refused. The source/layout evidence is in
[the worker follow-up](../../docs/protocol/native-unpacking.md).

`--host-meminfo` explicitly maps real host `/proc/meminfo` for the local decoder.
It removes the missing-memory-metadata warning, but the tested reconstruction
still fails identically. It neither invents Android properties nor supplies
WhatsApp account/device prerequisites. Without the flag that path stays absent.

Regression checks (six passed locally, including genuine guest execution,
partial-output preservation and refusal to trace an unknown decoder):

```sh
TELEWHATSAPP_LOADER_ROOTFS=lab/apk-2.26.37.71/native/qiling-rootfs \
  lab/native-analysis-tools-py312/bin/python -m unittest discover \
  -s tools/apk-research -p test_native_loader_check.py -v
```

The rootfs-dependent tests skip if that environment variable is absent. The
timeout test also needs Clang/LLD and builds its loop fixture in a temporary
directory. [Native evidence and remaining unknowns](../../docs/protocol/native-unpacking.md)
records APK addresses, JNI provenance, failures and actual check results.

## Ordinary full extraction with QEMU

`native_qemu_check.py --rootfs PATH [--check decoder-init|decoder-full-archive]
[--timeout-seconds N]` uses installed `bwrap` and `qemu-aarch64-static`.
The wall limit defaults to 30 seconds and accepts 1–600; there is no instruction
counter in this runner. It validates the exact 37.71 decoder/container hashes,
uses an empty environment, isolates network/PIDs, mounts fixtures read-only
and permits extraction file writes only under `/decoded`. It prepares mount
points and makes only the local probe/linker copies executable (QEMU refused
their original non-executable mode). Build the same C probe/imports above first.

```sh
python tools/apk-research/native_qemu_check.py \
  --rootfs lab/apk-2.26.37.71/native/qiling-rootfs --check decoder-init
python tools/apk-research/native_qemu_check.py \
  --rootfs lab/apk-2.26.37.71/native/qiling-rootfs \
  --check decoder-full-archive --timeout-seconds 30
```

Full extraction requires an empty `decoded/` destination and the original
outer ELF at `system/lib64/wa_archive.so`. A repeat into populated storage
refuses; no existing/partial output is deleted. Locally the normal full entry
returned 80 names and process exit zero, yielding 79 unique ARM64 ELF files
(one duplicate `libsodium.so` name). An independent wrapper run reproduced all
79 hashes. Raw libraries remain ignored. This executes the decoder alone;
it does not run WhatsApp, authenticate an account or recover a safety-number
algorithm by itself.

Both runners' regression checks (eight passed locally):

```sh
PYTHONDONTWRITEBYTECODE=1 \
  TELEWHATSAPP_LOADER_ROOTFS=lab/apk-2.26.37.71/native/qiling-rootfs \
  lab/native-analysis-tools-py312/bin/python -m unittest discover \
  -s tools/apk-research -p 'test_native_*check.py' -v
```

QEMU preflight checks require no emulator execution; real QEMU initialization
and full extraction were validated separately against the pinned artifacts.

## Fingerprint hash control flow

`native_fingerprint_check.py --library PATH [--check hash|bytes]` requires Unicorn and pyelftools,
and the exact reconstructed 37.71 `libwhatsapp.so` hash. It maps only the shared
hash routine page, runs built-in synthetic cases, and models SHA/MCF calls on
the host. Native ARM64 ordering/counts must agree with 5200 rounds of SHA-512;
each case has a ten-second/one-million-instruction limit and must actually
return. Unexpected branches refuse. This does not recover the JNI producers,
execute native crypto or establish an account safety number.
[Addresses, validation and unresolved inputs](../../docs/protocol/safety-number.md).

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_fingerprint_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so
PYTHONDONTWRITEBYTECODE=1 \
  TELEWHATSAPP_LOADER_ROOTFS=lab/apk-2.26.37.71/native/qiling-rootfs \
  TELEWHATSAPP_FINGERPRINT_LIBRARY=lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so \
  lab/native-analysis-tools-py312/bin/python -m unittest discover \
  -s tools/apk-research -p 'test_native_*check.py' -v
```

The latter includes eleven checks. Pinned-library execution skips without
`TELEWHATSAPP_FINGERPRINT_LIBRARY`; fingerprint preflight checks need no ELF
or emulator dependency. Existing rootfs requirements/skips still apply.


The [native callback follow-up](../../docs/protocol/safety-number.md#native-to-java-identifier-callback-and-dispatch-mapping)
connects the static ELF dispatch table to Java PN/LID normalization and the
V4 create/result routines. Existing `xtask apk-smali` and `llvm-objdump`
commands there reproduce it. The synthetic checker models SHA/MCF entry calls;
it does not execute these JNI conversions, QR serializers or key constructors.


## Native fingerprint QR schema

`native_schema_check.py --library PATH` requires pyelftools and the exact
reconstructed 37.71 `libwhatsappmerged.so` hash. It reads the two message and
hosted-state enum descriptors without native execution or account inputs.
Output is protocol metadata, including tags, types, offsets and source addresses;
[committed output](../../docs/protocol/fingerprint-native-schema.json) is reproducible.
[Serialization/comparison findings](../../docs/protocol/safety-number.md#native-qr-schema-serialization-and-v4-comparison)
distinguish optional schema fields from required comparison inputs.

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_schema_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so
PYTHONDONTWRITEBYTECODE=1 \
  TELEWHATSAPP_LOADER_ROOTFS=lab/apk-2.26.37.71/native/qiling-rootfs \
  TELEWHATSAPP_FINGERPRINT_LIBRARY=lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so \
  TELEWHATSAPP_SCHEMA_LIBRARY=lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so \
  lab/native-analysis-tools-py312/bin/python -m unittest discover \
  -s tools/apk-research -p 'test_native_*check.py' -v
```

All sixteen cases passed locally with these fixtures. Schema integration skips
without `TELEWHATSAPP_SCHEMA_LIBRARY`; its missing/wrong-build preflight cases
need no library or ELF dependency. Neither schema extraction nor these tests
executes whole QR comparison or sets a peer's verified bit.


`--check bytes` executes the actual 37.71 ARM64 equal-length byte comparator,
without modeled runtime/crypto calls. The 862 built-in synthetic comparisons
cover zero length, 8/32-byte loop transitions, 33-byte keys, 64-byte digests,
66-byte concatenations and 4096 bytes, with shifted pointers and mismatch
positions. Input ends meet an unmapped page; shifted cases leave one sentinel
byte. Each invocation has the same ten-second/one-million-instruction bounds
and must return. Equal-length instruction counts agree for these cases; this
is not an end-to-end constant-time proof or a full QR comparison.

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_fingerprint_check.py --check bytes \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so
```

[Primitive and orientation-dependent error evidence](../../docs/protocol/safety-number.md#comparison-primitives-and-orientation-dependent-statuses)
records the remaining comparison-to-verification boundary.

## Native V4 comparison branch checker

`native_v4_comparison_check.py --library PATH` requires the pinned 37.71
reconstructed `libwhatsapp.so`, Unicorn and pyelftools. It executes actual core,
PN/username helper and remapper instructions on 115 built-in synthetic cases;
getters, key/LID equality and MCF string primitives are modeled. It accepts no
account data, keys or scanned payloads. Output is deterministic JSON matching
[committed observations](../../docs/protocol/fingerprint-native-comparison-cases.json).
Wrong/missing builds, unexpected branches and budget exhaustion refuse before
printing partial results. Bounds are ten seconds/one million instructions per
core/remap call. Both explicit native abort cases are intercepted. No native
protobuf parsing, orientation wrapper or Android libc string behavior is tested.

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_v4_comparison_check.py --library \
  lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so
```

`test_native_v4_comparison_check.py` covers missing/wrong-build refusals and
compares all observations against the committed artifact. Actual emulation skips
unless `TELEWHATSAPP_FINGERPRINT_LIBRARY` is set. See the
[execution scope and findings](../../docs/protocol/safety-number.md#executed-native-v4-comparison-branches-2026-10-07).

Optional paired flags on that checker execute actual APK merged MCFStringCompare
and public Bionic strncasecmp over synthetic MCFString objects, replacing host
case-folding. Both additional libraries must match their pinned hashes. The
public libc fixture is not APK-bundled or evidence of every Android version.
The 115 core statuses agree; 268 additional ASCII-byte/prefix/empty/256-byte
cases reproduce case-insensitive ASCII equality with exact lengths. Other
primitives/object construction remain modeled. No account state changes.

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_v4_comparison_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so \
  --string-library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so \
  --libc lab/apk-2.26.37.71/native/qiling-rootfs/system/lib64/libc.so
```

Set `TELEWHATSAPP_BIONIC_LIBRARY` along with the main/merged fixture variables
for the additional actual-string regression test. Pairing/wrong-build preflight
checks require no fixtures. See
[scope and compact observations](../../docs/protocol/safety-number.md#actual-native-ascii-username-comparison-2026-10-07).

## Native fingerprint protobuf-C parser

`native_fingerprint_parse_check.py --library PATH` executes actual APK 37.71
combined unpack/generated init/internal parse/merge/free instructions on 50
built-in synthetic buffers. Exact merged-library hash is required. Custom
allocator/free and memcpy/memset are modeled; no native library initialization,
JNI, account or socket executes. It accepts no arbitrary/account payload.
Instruction/return, 64-KiB input, 1-MiB heap and allocation-release guards refuse
unexpected branches or exhaustion before partial JSON output. The
[committed corpus](../../docs/protocol/fingerprint-native-parser-cases.json)
contains synthetic bytes and all recognized decoded fields/presence, including
noncanonical cases that disproved prost equivalence. See
[native parsing rules and correction](../../docs/protocol/safety-number.md#actual-protobuf-c-parsing-and-bounded-decoder-correction-2026-10-07).

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_fingerprint_parse_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so
```

`test_native_fingerprint_parse_check.py` includes actual corpus reproduction
with `TELEWHATSAPP_SCHEMA_LIBRARY`, otherwise explicitly skipped, and always
runs missing/wrong-build empty-output refusal checks. All fixture variables
configured brings the native tooling suite to 25 tests.

### Own-device fingerprint directory (37.71)

Existing `xtask apk-smali` now has attributed own-directory excerpts in
[the evidence snapshot](../../docs/protocol/fingerprint-own-directory-evidence.txt)
and [selection reasoning](../../docs/protocol/safety-number.md#own-directory-source-logout-filtering-and-alias-precedence-2026-10-07).
Use exact class filters `LX/1IW;` (A08/A0F), `LX/1IY;` (A01/A07/A08),
`LX/HxZ;` (constructor/A01/A03), `LX/1xi;` (A1Q), `LX/1Kt;` (constructor)
and `LX/0bN;` (A09/A0A). A class-empty `--method BNm` dump identifies all
three mode-predicate definitions without guessing the implementation class.
These static excerpts establish source/logout/alias/cache rules, not current
account membership, signed-list freshness or a live safety-number calculation.
Raw dumps remain ignored under `lab/apk-2.26.37.71/`.

### Exact DEX field producers and consumers

`cargo run --locked -p xtask -- apk-field-refs <apk> --class 'LX/Contact;' \
--field state --type I [--writes-only]` finds exact standard instance/static
field access sites. The optional switch excludes reads. Output includes DEX
file, method/shorty, read/write opcode and absolute DEX byte offset; total count
is on stderr. Payloads are skipped; malformed traversed structures and no DEX
files refuse before stdout. Reflection/native/quickened offsets are outside
scope. Follow each site with `apk-smali`, rather than treating a write as proof
of its input's origin or server meaning.

The [contact-state follow-up](../../docs/protocol/safety-number.md#fingerprint-contact-predicate-is-external-user-state-2026-10-07)
and [scoped evidence](../../docs/protocol/fingerprint-contact-state-evidence.txt)
use this command to identify all 17 direct APK 37.71 writes to 0Da.A04 and trace
the fingerprint predicate to `external_user_state`. Raw APKs/dumps stay local.

### Scheme and optional fingerprint inputs

[Selection evidence](../../docs/protocol/fingerprint-input-selection-evidence.txt)
and [branch reasoning](../../docs/protocol/safety-number.md#scheme-username-and-pn-sharing-input-branches-2026-10-07)
connect JhJ.A0Z to V3/V4, own companion inclusion, username storage/prefixing
and nullable PN-sharing state. Reproduce with `apk-smali --class 'LX/JhJ;' \
--method A0Z`, class-empty ATB/Aq3 definition searches and `apk-xref --anchor \
self_user_name`. Relative code-unit jumps now print signed magnitudes, including
goto/32. The former plus-prefixed two's-complement output obscured backwards
control flow; APK/minimum/truncation tests cover the correction. No active
account feature values, verified membership or current-peer scan are inferred.

The [V3 scan trace](../../docs/protocol/safety-number.md#native-v3-scan-dispatch-and-comparison-branches-2026-10-07)
and [scoped native evidence](../../docs/protocol/fingerprint-v3-comparison-evidence.txt)
resolve selector 19 and options 1/3 using existing apk-smali and llvm-objdump.
JNI records require `.rela.dyn` addends; raw pointer slots are zero. The V3
converter/orientation/core differ from V4. No V3 execution checker or binary
comparison capability is implemented by this static follow-up.

### Execute the native V3 comparison core

`native_v3_comparison_check.py --library PATH` requires the pinned 37.71
reconstructed main library and runs 264 built-in synthetic cases through the
actual core, identity/optional-value/PN-presence helpers and remapper. It
reports deterministic codes and instruction counts; see
[observations](../../docs/protocol/fingerprint-native-v3-comparison-cases.json)
and [scope/unknowns](../../docs/protocol/safety-number.md#executed-v3-core-and-remapper-corpus-2026-10-07).

```sh
<python-env> tools/apk-research/native_v3_comparison_check.py --library <libwhatsapp.so>
TELEWHATSAPP_FINGERPRINT_LIBRARY=<libwhatsapp.so> <python-env> -m unittest discover -s tools/apk-research -p 'test_native_v3_comparison_check.py'
```

Generic MCFEqual and getters/key equality are modeled. Native JID construction,
protobuf decode, orientation wrapper, JNI and human action do not run. No
arbitrary/account input is accepted. Missing/wrong builds, unmodeled branches
and execution-budget exhaustion refuse before JSON output. Per core/remap call:
ten seconds/one million instructions; maximum 227 instructions observed.
The shared V4 runner retains its existing default/output and explicitly refuses
using V4's string mode for V3. This is offline research, not runtime V3 support.

The [V3 equality/LID follow-up](../../docs/protocol/safety-number.md#v3-equality-callbacks-and-primary-lid-conversion-2026-10-07)
and [scoped native ranges](../../docs/protocol/fingerprint-v3-equality-lid-evidence.txt)
resolve class equality callbacks and primary-LID conversion statically.
The corpus's synthetic identity tokens remain control-flow inputs, not valid
converter fixtures. Case-sensitive byte equality differs from V4's username
helper. Device-zero flattening occurs before the final kind gate. Actual
class runtime/parser execution and full binary V3 support remain unfinished.

### Execute V3 primary-LID decisions

`native_v3_lid_check.py --library PATH` requires the pinned 37.71 main library.
It accepts only 38 built-in synthetic inputs and executes native parser routing,
LID validation, constructors, device-zero flattening and final kind selection.
[Observations](../../docs/protocol/fingerprint-native-v3-lid-cases.json) and
[scope/limits](../../docs/protocol/safety-number.md#executed-v3-primary-lid-parser-decisions-2026-10-07)
retain the evidence and modeled dependencies.

```sh
<python-env> tools/apk-research/native_v3_lid_check.py --library <libwhatsapp.so>
TELEWHATSAPP_FINGERPRINT_LIBRARY=<libwhatsapp.so> <python-env> -m unittest discover -s tools/apk-research -p 'test_native_v3_lid_check.py'
```

MCF split/getters/copies/equality/formatting, allocation/class registration,
retain/release/errors, constants and validated decimal conversion are modeled.
This is not full MCF or protobuf-converter execution. Per entry: ten seconds /
one million instructions; return PC required; 1-MiB heap and 256-byte ASCII
string bounds. Largest observed case is 1188 instructions. Unknown branches,
bounds and build failures refuse before JSON. No runtime/account support added.
