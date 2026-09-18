# Verification and CI

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 19. Verification and CI

The tests must prove invariants and interoperability, not just that our implementation agrees with itself.

### 19.1 Protocol and parser tests

- Independent node/token/JID fixtures, exact boundary lengths and malformed/truncated input.
- Noise transcripts and certificate verification, including a valid-looking certificate that does not bind to the remote key.
- Signal/prekey/sender-key vectors from section 10.
- Unknown token/profile behavior and bounded opaque payload preservation.
- Compression, protobuf and nested-node resource exhaustion cases.
- Deterministic replay of reconnect, expiry, duplicate, out-of-order and retry scenarios using a controllable clock.

### 19.2 Required crash matrix

| Inject failure here | Expected recovery |
| --- | --- |
| Before intent commit | No accepted send ticket exists; same client request can enqueue normally |
| After intent commit, before encryption | Intent remains queued with unchanged ratchet |
| After crypto calculation, before atomic commit | Derived output is discarded; nothing can have been transmitted |
| After crypto/attempt commit, before socket write | Prepared attempt and updated crypto state are both recoverable |
| During/after socket write, before ACK | Outcome remains uncertain; retries follow observed semantics with the same logical message |
| After ACK arrival, before its commit | Recovery does not invent a fresh message ID or assume failure |
| After inbound decrypt, before commit | Ratchet/message/prekey changes are all absent; replay can process again |
| After inbound commit, before receipt | Message is not re-emitted as new; pending receipt can resume |
| After consumer side effect, before cursor ACK | Same event may replay; consumer deduplication prevents duplicate local effects |
| During key rotation, schema migration or backup | Reopen produces a valid prior/new state or an explicit recoverable migration state |
| During disk-full/commit I/O failure | No further send/decrypt advancement occurs until durable state is resolved |

Include a second-process ownership test, store corruption/wrong-key tests, and bounded memory/disk behavior under a stalled consumer. Test partial writes and cancellation, not only clean disconnections.

### 19.3 Live interoperability evidence

Use a controlled account pair and record the APK/profile, library commit, test conditions and date. Verify the message contents on the official peer and on the Rust side. Cover direct conversations with recipient companions, group membership changes, media integrity and process restart.

Test cold registration only when that specific test is intentionally invoked; normal CI and reconnect tests reuse existing test state. Do not trigger new OTPs or account mutations from every pull request or scheduled job.

Choose and record the soak duration and message/device workload. Include at least one prolonged idle period, network outage, restart and prekey/device-list maintenance cycle before a production-readiness claim. Report what was actually exercised.

### 19.4 CI jobs

1. Formatting, Clippy and compilation for supported target/feature combinations.
2. Unit, independent-fixture, property, replay and crash-recovery tests, offline after dependencies are available.
3. Dependency/license/advisory and native-build audits for the release graph.
4. Rust-only protobuf regeneration followed by a clean generated-code diff.
5. Package verification and standalone example build outside the workspace.
6. A separately invoked live interoperability job using protected test state.

Start with Linux x86_64 as the proof target. Add Linux aarch64 and other targets only after their dependency and storage behavior is verified; advertise exactly the tested matrix. Pin the Rust toolchain and establish MSRV from actual dependency requirements.

Suggested baseline commands, once the crate and tasks exist:

```sh
cargo fmt --all -- --check
cargo clippy --locked -p whatsapp-api --all-targets -- -D warnings
cargo test --locked -p whatsapp-api --all-targets
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
cargo package --locked -p whatsapp-api
```

Run additional feature sets explicitly. Do not use `--all-features` as a substitute for understanding mutually exclusive backends. Do not require a C++-backed fuzzing engine in the strict Rust build pipeline; use the Rust property/replay harness for the required gate.
