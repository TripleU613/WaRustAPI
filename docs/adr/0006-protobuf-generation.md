# ADR 0006: Committed generated protobuf code via `protox` + `prost-build`

Status: accepted · 2026-09-17

## Decision

- Source schemas live in `proto/whatsmeow/` (copied from `tulir/whatsmeow`
  `proto/` @ 2375e175, MPL-2.0, `LICENSE` and `REVISION` committed
  alongside). Only the schemas needed for handshake/certificate work are
  vendored: `WACert`, `WAWebProtobufsWa6`, `WACommon`, `WAAdv`.
- `cargo run -p xtask -- proto-gen` compiles them with `protox` 0.9.1 and
  `prost-build` 0.14.4 (`skip_protoc_run`), writes
  `crates/whatsapp-api/src/proto/*.rs` plus a `mod.rs` header with source
  hashes and provenance. `proto-check` regenerates into a temp dir and fails
  CI on any diff. No `protoc`, no `build.rs`, no network.
- Generated types are `#[doc(hidden)]` and never appear in the public API.
- Evidence label: `ReferenceOnly`. `WebMessageInfo`/E2E schemas are not
  vendored; they must not become the default Android payload by accident.
