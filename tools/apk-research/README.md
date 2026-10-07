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

`--timeout-seconds` is a parent-process wall limit (default 10, range 1–60).
`--max-instructions` charges whole ARM64 basic blocks across all guest threads
before execution (default 1000000, range 1–1000000000). Qiling's threaded runner
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
was insufficient even to reach that outcome. Native extraction is unfinished.

`--host-meminfo` explicitly maps real host `/proc/meminfo` for the local decoder.
It removes the missing-memory-metadata warning, but the tested reconstruction
still fails identically. It neither invents Android properties nor supplies
WhatsApp account/device prerequisites. Without the flag that path stays absent.

Regression checks (four passed locally, including genuine guest execution):

```sh
TELEWHATSAPP_LOADER_ROOTFS=lab/apk-2.26.37.71/native/qiling-rootfs \
  lab/native-analysis-tools-py312/bin/python -m unittest discover \
  -s tools/apk-research -p test_native_loader_check.py -v
```

The rootfs-dependent tests skip if that environment variable is absent. The
timeout test also needs Clang/LLD and builds its loop fixture in a temporary
directory. [Native evidence and remaining unknowns](../../docs/protocol/native-unpacking.md)
records APK addresses, JNI provenance, failures and actual check results.
