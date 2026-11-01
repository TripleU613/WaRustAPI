# ARM64 decoder probe (Rust)

`decoder_probe.rs` is the guest-side probe that `xtask native-qemu-check` and
`tools/apk-research/native_loader_check.py` run inside the public Bionic rootfs.
`android_imports.rs` (libandroid.so) and `log_imports.rs` (liblog.so) supply the
two Android libraries that rootfs lacks. All three are `no_std` Rust ports of
the former C sources. They are not part of the Cargo workspace: they target
`aarch64-linux-android` and link against the rootfs Bionic, never the host.

Requirements: a Rust toolchain with defining C-variadic functions available
(`log_imports.rs` forwards `__android_log_print` to Bionic `vprintf`; built and
verified with 1.99.1) and the `aarch64-linux-android` target. `rust-lld` ships
with rustup, so no NDK or C compiler is needed. A toolchain kept apart from your
default one works:

```sh
export RUSTUP_HOME=lab/native-rust/rustup CARGO_HOME=lab/native-rust/cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
  --no-modify-path --default-toolchain 1.99.1 --profile minimal \
  --target aarch64-linux-android
```

Build into the rootfs (same layout the runners expect):

```sh
ROOT=lab/apk-2.26.37.71/native/qiling-rootfs
RUSTC="$CARGO_HOME/bin/rustc +1.99.1"
COMMON=(--edition 2024 --target aarch64-linux-android -C panic=abort -C opt-level=s
  -C linker=rust-lld -C linker-flavor=ld.lld -C link-arg=-L$ROOT/system/lib64
  -C link-arg=-z -C link-arg=max-page-size=4096 -C link-arg=-l:libc.so)
cd tools/apk-research/native-probe
$RUSTC "${COMMON[@]}" --crate-type cdylib -C link-arg=-soname=libandroid.so \
  android_imports.rs -o $ROOT/system/lib64/libandroid.so
$RUSTC "${COMMON[@]}" --crate-type cdylib -C link-arg=-soname=liblog.so \
  log_imports.rs -o $ROOT/system/lib64/liblog.so
$RUSTC "${COMMON[@]}" --crate-type bin -C relocation-model=pie -C link-arg=-pie \
  -C link-arg=-e -C link-arg=_start -C link-arg=--dynamic-linker=/system/bin/linker64 \
  -C link-arg=-l:libdl.so decoder_probe.rs -o $ROOT/bin/decoder_probe
```

(Paths are relative to the repository root and `cd` changes it; use absolute
paths for `ROOT` and the lab variables in practice.)

**Verified 2026-11-01** against the C-built originals: `xtask native-qemu-check
--check decoder-init` printed byte-identical output, and
`--check decoder-full-archive` into an empty `decoded/` produced the same 79
libraries with every SHA-256 equal. Evidence:
[native-unpacking](../../../docs/protocol/native-unpacking.md#rust-probe-and-runner-2026-11-01).
