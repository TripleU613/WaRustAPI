# ADR 0001: Pure-Rust build contract and workspace layout

Status: accepted · 2026-09-17

Amendment: the owner approved the narrowly scoped native `ring` TLS exception
in [ADR 0003](0003-tls-provider-gate.md). First-party Rust, no runtime
subprocesses, and the remaining native dependency restrictions still apply.

## Context

docs/design/README.md section 2 requires every first-party component to be Rust and forbids
bundled or linked C/C++ application, storage or cryptography code in the
production graph. No prior workspace existed: the repository contained only
the plan. `crates/tzibbur-api` and `crates/bridge`, referenced by the plan as
integration targets, are **not present** in this repository; their interfaces
could not be inspected and no compatibility requirement could be recorded.

## Decision

- Cargo workspace with `crates/whatsapp-api` (library), `crates/whatsapp-cli`
  (operator/proof tool) and `xtask` (automation), edition 2024, toolchain
  pinned to 1.94.1 in `rust-toolchain.toml`, MSRV 1.94.
- `unsafe_code = "forbid"` workspace-wide; `todo!`/`unimplemented!`/`dbg!` denied.
- The dependency graph is audited by `cargo run -p xtask -- dependency-audit`,
  which walks normal edges for Linux x86_64, flags `-sys` names, build
  scripts, `links` keys and a denied list (`ring`, `aws-lc-*`, `openssl*`,
  `libsqlite3-sys`, `rusqlite`, `zstd-sys`, `bzip2-sys`, `libz*-sys`). CI fails
  on a denied crate and on a stale committed report.
- Build scripts present in the graph were read and classified as cfg probes
  (`redb`, `snow`, `zmij`, `quote`, `libc`, `getrandom`, `serde*`, ...).
- Lab tooling (APK inspection) is also Rust (`xtask reference-manifest`).
  External tools (`keytool`, `unzip`) were used once to cross-check its
  output and are not required by any build or runtime path.

## Consequences

- The bridge adapter (docs/design/README.md section 17) cannot be written until the bridge
  crate is available; the library keeps `SyncEvent`/`SyncEngine` separate so
  an adapter can be added without touching protocol code.
- `cargo metadata` unifies dev-dependency features into the report (e.g.
  `tokio/test-util`); the report notes this approximation.
