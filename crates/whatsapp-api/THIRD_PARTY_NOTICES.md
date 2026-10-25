# Third-party notices

The workspace declares MIT OR Apache-2.0 for first-party code. This does not
relicense third-party inputs or dependencies.

| Input | Provenance and license |
| --- | --- |
| Optional `wacore-libsignal` 0.7.0 | Exact registry version/checksum in Cargo.lock; published source revision `f8165f282008935732e0b26d6f5cca5038ecd222`. Contains Signal Messenger AGPL-3.0-only source headers despite MIT Cargo metadata. Owner approved evaluation/integration; upstream licenses still apply. |
| Optional `ml-kem` 0.3.2 | RustCrypto, MIT OR Apache-2.0; exact checksum in Cargo.lock; zeroize enabled. Upstream reports no independent audit. Experimental primitive only, selected APK/NIST vectors verified. |
| Optional rustls `ring` provider | `ring` 0.17.14, Cargo.lock; native crypto under its upstream licenses, explicitly approved for HTTPS in ADR 0003 |
| Vendored protobuf schemas | `proto/whatsmeow/`, from tulir/whatsmeow; [revision](https://github.com/TripleU613/WaRustAPI/blob/main/proto/whatsmeow/REVISION) and [MPL-2.0 license](https://github.com/TripleU613/WaRustAPI/blob/main/proto/whatsmeow/LICENSE) |
| Generated protobuf Rust | `crates/whatsapp-api/src/proto/`, generated from those schemas; retain upstream provenance |
| Binary token fixture | `crates/whatsapp-api/fixtures/tokens/web-md-v3.json`; source revision, MIT designation, and hash are embedded in the fixture |
| Reference and comparison observations | [Reference manifest](https://github.com/TripleU613/WaRustAPI/blob/main/crates/whatsapp-api/fixtures/reference-manifest.json) and [research log](https://github.com/TripleU613/WaRustAPI/blob/main/docs/protocol/decompilation-log.md); APKs are not distributed |

See the [component review](https://github.com/TripleU613/WaRustAPI/blob/main/docs/protocol/reusable-components.md),
[dependency report](https://github.com/TripleU613/WaRustAPI/blob/main/docs/protocol/dependency-report.md), and
[Signal licensing gate](https://github.com/TripleU613/WaRustAPI/blob/main/docs/adr/0005-signal-backend-gate.md).
Cargo dependencies retain their individual licenses. Publication remains
disabled pending the project's compatibility, licensing, and release gates.
