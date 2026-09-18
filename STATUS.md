# Status — 2026-09-17

**Gate position:** G0 complete (repository, dependency decisions, baseline).
G1 partially complete offline (codec, framing, Noise adapter, certificate
binding, schemas, reference manifest, **static APK observations**). **G2 not
attempted**: the Android registration payload encoding, transport port/layer
and login payload values are `Unknown` (see `docs/protocol/unknowns.md`). No
live server contact has been made and no OTP has been requested.

**Owner-approved integration update:** registration HTTPS now selects
rustls/ring instead of the alpha RustCrypto provider. The optional
`experimental-signal-crypto` feature supplies an upstream-backed XEdDSA
certificate-signature verifier. This is not a complete Signal messaging
backend; trust-root evidence, key provisioning, sessions and live interop
remain unfinished. The historical offline inventory below predates this update.

The [persisted registration lab](docs/protocol/registration-lab.md) now connects
key generation/signing, encrypted persistence and comparison request assembly.
It is explicitly lab-profile-only; default registration and messaging remain blocked.
The opt-in CLI feature `experimental-signup` now exposes `lab-signup` for
single-action operator-authorized HTTPS attempts. Ordinary commands remain
blocked. Two prior operator-authorized attempts returned `ServerRefused` without
a recorded HTTP reason; neither established signup or OTP delivery. The editable
[request profile](crates/whatsapp-api/src/registration/request.rs) now exports to
private JSON, with offline preview and redacted HTTP diagnostics for future
explicit actions. [Outbound POST analysis](docs/protocol/http-post-analysis.md)
has started; it is not a complete traffic audit. Server acceptance is unverified.

One ownership-confirmed, one-shot legacy diagnostic on 2026-09-18 UTC reused
the saved identity and returned HTTP 200 / `fail` / `bad_param`. The actual
reason now survives restart. The offending field is unknown; no OTP delivery
or successful signup is confirmed. Known refusals and cooldowns remain ineligible
for that legacy-only action, and HTTP-library retries are explicitly disabled.

**Key static findings (2026-09-17):** the pinned client contains classic
`Noise_XX`/`Noise_IK` patterns **and nine ML-KEM512 hybrid Noise patterns**.
If the server requires a hybrid handshake for primary devices, the current
`snow`-based adapter cannot be used (ADR 0004 addendum). A separately pinned,
WhatsApp-signed 2.26.37.70 comparison statically confirms `WA 06 03` and the
registration URL/form-body construction and JSON response vocabulary. Its
compiled Noise selector defaults to classic mode, though remote configuration
can select ML-KEM and remains unobserved. Its built-in socket path uses raw
TCP for primary/fallback gateways on 443/5222, and TLS only for numbered-edge
port 443; the port-80 fallback is raw TCP with a bounded HTTP/1.1 chunked
`POST /chat` wrapper. These comparison findings do not replace evidence for
the active 2.26.36.75 profile.

## Verified in this repository (offline)

| Area | Evidence |
| --- | --- |
| Toolchain and dependency contract | `rust-toolchain.toml` 1.94.1; `cargo run -p xtask -- dependency-audit`: no denied crates in `whatsapp-api`, `whatsapp-api +experimental-noise`, `whatsapp-cli` (report: `docs/protocol/dependency-report.md`) |
| Reference APK pinned | `com.whatsapp` 2.26.36.75 / 263607533, sha256 `ce7468…922e9`, signing cert `39:87:D0:…:5D:44`; reproduced by `xtask reference-manifest --check` |
| APK static observations | `xtask apk-strings`: 449,216 dex strings in pinned 2.26.36.75; hosts, 12 Noise pattern names, registration vocabulary, ClientPayload field constants. `apk-xref`/`apk-callers`/`apk-smali`: structural method and caller analysis, validated on separately hashed 2.26.37.70. |
| Token dictionary | `xtask apk-tokens --check`: ordered dictionary recovered from `classes2.dex` bytecode is identical to the committed fixture → `binary.tokens` is client-observed; version byte still ReferenceOnly |
| Store | encryption at rest, AAD row binding, wrapped data keys with wrong-key detection, master-key rotation, second owner rejected, generation fencing, atomic rollback, journal replay/ack/trim |
| Crash matrix (docs/design/README.md 19.2) | `tests/crash_matrix.rs`: rows 1, 2, 3, 9, 10 (rotation) and the second-process ownership test pass; rows 4–8 and 11 need a transmission path / Signal backend / I/O fault injection |
| Codec | independent hand-written fixture, JID kinds, packed strings, bounds, duplicate attributes, zlib bomb; `tests/codec_props.rs` property round trips (found and fixed an odd-length hex padding overflow), decoder never panics on arbitrary bytes, framing under arbitrary fragmentation |
| Transport | framing fragmentation/cancellation tests; Noise XX handshake + records against a snow responder; complete version-scoped classic/ML-KEM selector map with tested classic compiled defaults; typed comparison-only `WA 06 03` header and built-in gateway/edge raw-TCP/TLS map; cert chain binding incl. "valid-looking chain that does not bind to the remote key"; opt-in upstream Signal signature verifier; feature-gated HTTPS with explicit rustls/ring provider, validation on, no redirects/retries/proxies, deadlines and bounded streaming responses |
| Registration | state table enforced; handles bound to attempt/generation/step; answers never persisted; prerequisite persistence is atomic with the pre-request state; elapsed cooldowns resume without regenerating keys; ambiguous outcomes have a non-repeating reconciliation seam; successful account credentials commit atomically with `Registered`; version-scoped 2.26.37.70 endpoint catalog, deterministic common-field, `/v2/code`, `/v2/register`, `/v2/security`, public key-bundle encoding, APK-backed `/v2/code` token derivation, and fresh-install UUID/token prerequisite generation; conservative comparison HTTP flow including the account-PIN branch is tested end-to-end with a mock transport (not installed for the active profile) |
| Outbox | idempotent enqueue, conflict, limits, cancel rules, uncertain delivery representation |
| Client | durable enqueue → status → events → ack → restart over the same store |
| CLI / HTTP control plane | `keygen`, `store init/inspect`, `send`, `status`, `events --ack`, `connect`; loopback `serve` with health/profile/capability and comparison transport/registration wire-profile endpoints plus structured gated-operation errors |

Test count: 161 (`cargo test --locked --workspace --all-features`). Formatting,
all-feature/all-target Clippy with `-D warnings`, strict rustdoc, package
verification, and protobuf regeneration checks pass on the continuation
machine (Rust 1.98.1). The repository and CI
remain pinned to Rust 1.94.1; that version was not rerun on this machine.
The newer-toolchain chunk-iterator warnings were resolved without changing
encoding behavior.

## Blocked / not started

- Registration driver, transport connect, login: need Android-profile
  evidence (`docs/protocol/evidence-ledger.md` rows marked `Unknown`).
- Signal backend: owner approved AGPL-component evaluation (ADR 0005);
  implementation and interoperability gates remain open.
- HTTPS now uses an explicit rustls/ring provider under the owner-approved
  native TLS exception (ADR 0003). Registration remains experimental behind
  `experimental-registration-https`; no live success is claimed.
- Inbox, devices, groups, media, message operations, bridge adapter: gates
  G3–G6; modules intentionally not created yet.
- Data-key re-encryption, backup/restore, and I/O-fault injection for the
  remaining crash-matrix rows.
- `crates/tzibbur-api` and `crates/bridge` are not in this repository; the
  adapter contract could not be inspected.

## Suggested next steps (in gate order)

The [compact APK research snapshot](docs/protocol/apk/README.md) now preserves
21 mapped protocol classes (149 methods), the comparison token dictionary,
and provenance/checksums in 564,315 bytes. The media-free export is bounded
to 2 MiB and can be regenerated or verified with `xtask apk-research-bundle`.
It is comparison-only structural disassembly, not a complete decompilation
or proof of live protocol behavior.

Continuation (2026-09-17): the supplied local APK was verified as the exact
2.26.37.70 comparison hash. Its frame writer uses a three-byte big-endian
length and rejects bodies at 16 MiB; server Noise record sizes remain open.
The Rust bytecode tool now displays `const/high16` values, with a regression
test covering this bound and signed bit patterns. No extra tools were needed.

1. Static: obtain exact pinned 2.26.36.75 and reproduce the comparison
   build's dictionary-version / `WA` header and registration findings; then
   continue into handshake selection and the trust root
   (`docs/protocol/apk-analysis.md`).
2. Integrate and validate the approved Signal candidate (ADR 0005); it also gates real XEdDSA
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
