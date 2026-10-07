# APK research helpers

Small offline helpers used on the full Dalvik disassembly of the comparison APK
(`./target/release/xtask apk-smali ~/Downloads/WhatsApp.apk --class "" > dump.txt`,
about 570 MB for build 2.26.37.71). They never touch the network or the lab data.

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
hello fixture, with an empty guest environment, a ten-second execution bound
and a one-million-instruction limit. It requires an observed `exit_group(0)`.
It does not reconstruct Superpack or run WhatsApp. Missing fixtures/dependencies
are diagnosed before execution. Rootfs/tool environments stay in ignored `lab/`.

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
