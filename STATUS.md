# Status — 2026-09-17

**Gate position:** G0 complete (repository, dependency decisions, baseline).
G1 partially complete offline (codec, framing, Noise adapter, certificate
binding, schemas, reference manifest, **static APK observations**). **G2 not
attempted**: the Android registration payload encoding, transport port/layer
and login payload values are `Unknown` (see `docs/protocol/unknowns.md`). No
live server contact has been made and no OTP has been requested.

**Key static finding (2026-09-17):** the pinned client contains classic
`Noise_XX`/`Noise_IK` patterns **and nine ML-KEM512 hybrid Noise patterns**.
If the server requires a hybrid handshake for primary devices, the current
`snow`-based adapter cannot be used (ADR 0004 addendum).

## Verified in this repository (offline)

| Area | Evidence |
| --- | --- |
| Toolchain and dependency contract | `rust-toolchain.toml` 1.94.1; `cargo run -p xtask -- dependency-audit`: no denied crates in `whatsapp-api`, `whatsapp-api +experimental-noise`, `whatsapp-cli` (report: `docs/protocol/dependency-report.md`) |
| Reference APK pinned | `com.whatsapp` 2.26.36.75 / 263607533, sha256 `ce7468…922e9`, signing cert `39:87:D0:…:5D:44`; reproduced by `xtask reference-manifest --check` |
| APK static observations | `xtask apk-strings`: 449,216 dex strings; hosts (`g.whatsapp.net`, `e1`–`e16`, `v.whatsapp.net`, `mmg.whatsapp.net`), 12 Noise pattern names, registration parameter vocabulary, ClientPayload field constants |
| Token dictionary | `xtask apk-tokens --check`: ordered dictionary recovered from `classes2.dex` bytecode is identical to the committed fixture → `binary.tokens` is client-observed; version byte still ReferenceOnly |
| Store | encryption at rest, AAD row binding, wrapped data keys with wrong-key detection, master-key rotation, second owner rejected, generation fencing, atomic rollback, journal replay/ack/trim |
| Crash matrix (README 19.2) | `tests/crash_matrix.rs`: rows 1, 2, 3, 9, 10 (rotation) and the second-process ownership test pass; rows 4–8 and 11 need a transmission path / Signal backend / I/O fault injection |
| Codec | independent hand-written fixture, JID kinds, packed strings, bounds, duplicate attributes, zlib bomb; `tests/codec_props.rs` property round trips (found and fixed an odd-length hex padding overflow), decoder never panics on arbitrary bytes, framing under arbitrary fragmentation |
| Transport | framing fragmentation/cancellation tests; Noise XX handshake + records against a snow responder; cert chain binding incl. "valid-looking chain that does not bind to the remote key" |
| Registration | state table enforced; handles bound to attempt/generation/step; secrets never persisted (byte scan of the database) |
| Outbox | idempotent enqueue, conflict, limits, cancel rules, uncertain delivery representation |
| Client | durable enqueue → status → events → ack → restart over the same store |
| CLI | `keygen`, `store init/inspect`, `send`, `status`, `events --ack`, `connect` (typed exit 4), `registration begin` (blocked with missing ledger refs), wrong key (exit 5) |

Test count: 77 (`cargo test --workspace --all-features`), clippy clean with
`-D warnings`, `cargo fmt --check` clean, CI green on GitHub.

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

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
cargo package --locked -p whatsapp-api --allow-dirty
```
