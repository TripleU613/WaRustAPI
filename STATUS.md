# Status — 2026-09-17

**Gate position:** G0 complete (repository, dependency decisions, baseline).
G1 partially complete offline (codec, framing, Noise adapter, certificate
binding, schemas, reference manifest, **static APK observations**). **G2 not
attempted**: the Android registration payload encoding, transport port/layer
and login payload values are `Unknown` (see `docs/protocol/unknowns.md`). No
live server contact has been made and no OTP has been requested.

**Key static findings (2026-09-17):** the pinned client contains classic
`Noise_XX`/`Noise_IK` patterns **and nine ML-KEM512 hybrid Noise patterns**.
If the server requires a hybrid handshake for primary devices, the current
`snow`-based adapter cannot be used (ADR 0004 addendum). A separately pinned,
WhatsApp-signed 2.26.37.70 comparison statically confirms `WA 06 03` and the
registration URL/form-body construction, but does not replace evidence for
the active 2.26.36.75 profile.

## Verified in this repository (offline)

| Area | Evidence |
| --- | --- |
| Toolchain and dependency contract | `rust-toolchain.toml` 1.94.1; `cargo run -p xtask -- dependency-audit`: no denied crates in `whatsapp-api`, `whatsapp-api +experimental-noise`, `whatsapp-cli` (report: `docs/protocol/dependency-report.md`) |
| Reference APK pinned | `com.whatsapp` 2.26.36.75 / 263607533, sha256 `ce7468…922e9`, signing cert `39:87:D0:…:5D:44`; reproduced by `xtask reference-manifest --check` |
| APK static observations | `xtask apk-strings`: 449,216 dex strings in pinned 2.26.36.75; hosts, 12 Noise pattern names, registration vocabulary, ClientPayload field constants. `apk-xref`/`apk-smali`: structural method analysis, validated on separately hashed 2.26.37.70. |
| Token dictionary | `xtask apk-tokens --check`: ordered dictionary recovered from `classes2.dex` bytecode is identical to the committed fixture → `binary.tokens` is client-observed; version byte still ReferenceOnly |
| Store | encryption at rest, AAD row binding, wrapped data keys with wrong-key detection, master-key rotation, second owner rejected, generation fencing, atomic rollback, journal replay/ack/trim |
| Crash matrix (README 19.2) | `tests/crash_matrix.rs`: rows 1, 2, 3, 9, 10 (rotation) and the second-process ownership test pass; rows 4–8 and 11 need a transmission path / Signal backend / I/O fault injection |
| Codec | independent hand-written fixture, JID kinds, packed strings, bounds, duplicate attributes, zlib bomb; `tests/codec_props.rs` property round trips (found and fixed an odd-length hex padding overflow), decoder never panics on arbitrary bytes, framing under arbitrary fragmentation |
| Transport | framing fragmentation/cancellation tests; Noise XX handshake + records against a snow responder; cert chain binding incl. "valid-looking chain that does not bind to the remote key" |
| Registration | state table enforced; handles bound to attempt/generation/step; secrets never persisted; version-scoped 2.26.37.70 endpoint catalog and bounded URL-form request construction (no HTTPS driver yet) |
| Outbox | idempotent enqueue, conflict, limits, cancel rules, uncertain delivery representation |
| Client | durable enqueue → status → events → ack → restart over the same store |
| CLI / HTTP control plane | `keygen`, `store init/inspect`, `send`, `status`, `events --ack`, `connect`; loopback `serve` with health/profile/capability endpoints and structured gated-operation errors |

Test count: 99 (`cargo test --workspace --all-features`), clippy clean with
`-D warnings`, `cargo fmt --check` clean, and package/protobuf checks pass
locally.

## Blocked / not started

- Registration driver, transport connect, login: need Android-profile
  evidence (`docs/protocol/evidence-ledger.md` rows marked `Unknown`).
- Signal backend: licence gate (ADR 0005).
- TLS provider for HTTPS: gate (ADR 0003).
- Inbox, devices, groups, media, message operations, bridge adapter: gates
  G3–G6; modules intentionally not created yet.
- Data-key re-encryption, backup/restore, and I/O-fault injection for the
  remaining crash-matrix rows.
- `crates/tzibbur-api` and `crates/bridge` are not in this repository; the
  adapter contract could not be inspected.

## Suggested next steps (in gate order)

1. Static: obtain exact pinned 2.26.36.75 and reproduce the comparison
   build's dictionary-version / `WA` header and registration findings; then
   continue into handshake selection and the trust root
   (`docs/protocol/apk-analysis.md`).
2. Decide the Signal licence question (ADR 0005); it also gates real XEdDSA
   certificate verification, for which no independent vectors are available
   offline yet.
3. With a controlled test number and operator consent: capture one
   registration and one handshake to move `registration.*`, `transport.*`
   and `noise.*` rows to `Observed`; only then implement the registration
   driver and `connect`.
4. Resolve whether the server requires the ML-KEM512 hybrid handshake
   before investing further in the `snow` adapter.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
cargo package --locked -p whatsapp-api --allow-dirty
```
