# Third-party notices

The workspace declares MIT OR Apache-2.0 for first-party code. This does not
relicense third-party inputs or dependencies.

| Input | Provenance and license |
| --- | --- |
| Vendored protobuf schemas | `proto/whatsmeow/`, from tulir/whatsmeow; [revision](https://github.com/TripleU613/TeleWhatsApp/blob/main/proto/whatsmeow/REVISION) and [MPL-2.0 license](https://github.com/TripleU613/TeleWhatsApp/blob/main/proto/whatsmeow/LICENSE) |
| Generated protobuf Rust | `crates/whatsapp-api/src/proto/`, generated from those schemas; retain upstream provenance |
| Binary token fixture | `crates/whatsapp-api/fixtures/tokens/web-md-v3.json`; source revision, MIT designation, and hash are embedded in the fixture |
| Reference and comparison observations | [Reference manifest](https://github.com/TripleU613/TeleWhatsApp/blob/main/crates/whatsapp-api/fixtures/reference-manifest.json) and [research log](https://github.com/TripleU613/TeleWhatsApp/blob/main/docs/protocol/decompilation-log.md); APKs are not distributed |

See the [component review](https://github.com/TripleU613/TeleWhatsApp/blob/main/docs/protocol/reusable-components.md),
[dependency report](https://github.com/TripleU613/TeleWhatsApp/blob/main/docs/protocol/dependency-report.md), and
[Signal licensing gate](https://github.com/TripleU613/TeleWhatsApp/blob/main/docs/adr/0005-signal-backend-gate.md).
Cargo dependencies retain their individual licenses. Publication remains
disabled pending the project's compatibility, licensing, and release gates.
