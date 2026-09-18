# Operations and publication

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 20. Operational behavior and publication

Expose structured, secret-safe observations: connection state/reason, queue depth, oldest pending age, crypto/store error class, reconnect attempts, retry deadlines, protocol profile and journal lag. Avoid message bodies, phone numbers, private keys, OTPs, full authenticated stanzas and media URLs in normal logs or panic reports.

Provide configurable limits for frame sizes, decoder work, message size, media bytes, outbox count/bytes, inbox retention, journal retention, skipped-message state, pending requests and task concurrency. Defaults are local product limits, clearly distinguished from observed wire limits.

Ship a Rust CLI with commands for registration steps, connection/status, a direct-text proof, event consumption, profile inspection and consistent backup/restore. Prefer typed Rust protocol/replay output over shelling out to helper runtimes. Protect secret inputs from command-line history.

The CLI also exposes a bounded local HTTP control plane for operational
inspection. It listens on loopback by default and requires an explicit opt-in
for non-loopback addresses because HTTP authentication is not implemented.
Start it with:

```sh
cargo run --locked -p whatsapp-cli -- serve --bind 127.0.0.1:8787
curl http://127.0.0.1:8787/healthz
curl http://127.0.0.1:8787/v1/profile
curl http://127.0.0.1:8787/v1/capabilities
curl http://127.0.0.1:8787/v1/transport/wire-profile
curl http://127.0.0.1:8787/v1/registration/wire-profile
```

`POST /v1/connect` and `POST /v1/registration` return structured `501`
responses while their evidence-gated protocol implementations are incomplete.
They must not report success without opening the corresponding authenticated
path.

Before release:

- Verify the intended package name is available and choose a concrete licensing/distribution strategy for all included code.
- Make `cargo package` and a clean standalone example succeed without repository-only files or tools.
- Document public API guarantees, durable enqueue semantics, retry ambiguity, primary registration requirements and actual capabilities.
- Include the protocol compatibility matrix, known blockers, source notices and tested target/toolchain information.
- Keep experimental backend/profile support visibly separate from production-ready capability claims.

Publishing is a later release action. Writing this plan does not publish a crate or authorize account mutations.
