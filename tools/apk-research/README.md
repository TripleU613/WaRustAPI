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

`native_fingerprint_check.py --library PATH` requires Unicorn and pyelftools,
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
