# Third-party notices

The workspace declares MIT OR Apache-2.0 for first-party code. This does not
relicense third-party inputs or dependencies.

| Input | Provenance and license |
| --- | --- |
| Vendored protobuf schemas | `proto/whatsmeow/`, from tulir/whatsmeow; [revision](proto/whatsmeow/REVISION) and [MPL-2.0 license](proto/whatsmeow/LICENSE) |
| Generated protobuf Rust | `crates/whatsapp-api/src/proto/`, generated from those schemas; retain upstream provenance |
| Binary token fixture | `crates/whatsapp-api/fixtures/tokens/web-md-v3.json`; source revision, MIT designation, and hash are embedded in the fixture |
| Reference and comparison observations | [Reference manifest](crates/whatsapp-api/fixtures/reference-manifest.json) and [research log](docs/protocol/decompilation-log.md); APKs are not distributed |
| Selected APK bytecode annotations and comparison tokens | [Research snapshot](docs/protocol/apk/README.md); derived from the exact WhatsApp APK identified in its manifest, not first-party Rust code; the workspace license does not relicense these inputs |

See the [component review](docs/protocol/reusable-components.md),
[dependency report](docs/protocol/dependency-report.md), and
[Signal licensing gate](docs/adr/0005-signal-backend-gate.md).
Cargo dependencies retain their individual licenses. Publication remains
disabled pending the project's compatibility, licensing, and release gates.
