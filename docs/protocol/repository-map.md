# Repository map (G0)

| Path | Responsibility | Status |
| --- | --- | --- |
| `README.md` | Project overview and quick start | — |
| `docs/design/` | Implementation plan with stable section numbers | — |
| `docs/README.md`, `CONTRIBUTING.md` | Documentation index and contributor workflow | — |
| `STATUS.md` | What works, what is blocked, gate status | — |
| `Cargo.toml`, `rust-toolchain.toml`, `Cargo.lock` | Workspace, pinned toolchain 1.98.1, lockfile | committed |
| `crates/whatsapp-api/src/lib.rs` | Public facade re-exports | done |
| `src/error.rs` | Typed errors | done |
| `src/evidence.rs` | Evidence labels | done |
| `src/model/` | ids, jid, phone, message, receipt, send, time | done |
| `src/capabilities.rs` | Capability matrix | done |
| `src/profile.rs` | Evidence-labelled protocol profile + reference manifest | done |
| `src/store/` | redb store, envelopes, keys, journal | done, including master-key rotation; backup/restore not implemented |
| `src/binary/` | Node codec, token dictionary | done, `Experimental` |
| `src/transport/` | framing, cert binding, `noise` (feature) | done, `Experimental` |
| `src/proto/` | Generated protobuf (do not edit) | generated |
| `src/registration/` | State machine, service, `UnsupportedFlow`, version-scoped registration wire request/response types and transport seam | comparison driver mock-tested; active-profile registration remains blocked |
| `src/session.rs` | Account record, key material types | done |
| `src/signal/` | Backend boundary, overlay, `NoBackend` | boundary only |
| `src/outbox.rs` | Durable send intents, tickets, facts | done; no transmission |
| `src/runtime/` | Account actor, epochs, correlation table | done; no socket |
| `src/sync.rs` | Events, `SyncEngine` | done |
| `src/client.rs` | `WhatsAppClient`, builder | done |
| `src/inbox.rs`, `src/reconcile.rs`, `src/devices.rs`, `src/groups.rs`, `src/media.rs` | Planned in docs/design/README.md 5 | **not created** (no real behaviour yet; docs/design/README.md 5 forbids empty modules) |
| `crates/whatsapp-api/fixtures/` | Reference manifest, token dictionary | committed with provenance |
| `crates/whatsapp-cli/` | Operator tool, proof harness, bounded loopback HTTP control plane | done for offline commands; network mutations remain gated |
| `xtask/` | proto-gen/check, dependency-audit, reference-manifest, apk-strings, apk-tokens, apk-xref, apk-smali | done |
| `proto/whatsmeow/` | Vendored schemas + LICENSE + REVISION | committed |
| `docs/adr/` | Decisions 0001–0006 | written |
| `docs/protocol/` | Evidence ledger, unknowns, component evaluation, APK analysis procedure, dependency report | written / generated |
| `crates/whatsapp-api/tests/` | `crash_matrix.rs` (docs/design/README.md 19.2 rows provable today), `codec_props.rs` (property tests) | done |
| `.github/workflows/ci.yml` | fmt, clippy, tests, audit, proto-check, package | written (not yet run on GitHub) |
