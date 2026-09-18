# Build and dependency policy

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 2. Pure Rust means a verifiable build contract

### 2.1 Required contract

All new first-party library code, CLI code, replay tools, fixture generators, and project-specific automation are Rust. TOML, JSON, protobuf schemas, Markdown, and CI YAML are data/configuration, not additional application languages.

The shipped library and CLI must not depend on a Node, Python, Go, Java, browser, emulator, or official WhatsApp process. They must not wrap Baileys, whatsmeow, WhatsApp Web, or a hosted service as their implementation.

For this plan, interpret the dependency requirement strictly: no bundled or linked C/C++ application, storage, or cryptography implementation in the new production graph. Ordinary operating-system interfaces used through Rust's standard library and OS bindings are allowed. This is not a demand to rewrite the kernel, TLS certificate store, or Rust toolchain.

Existing external tools may be used to inspect a reference APK in the research lab. They must not become installation, build, or runtime requirements for this product. Do not add first-party foreign-language scripts to work around missing Rust functionality.

### 2.2 Storage and code generation

`rusqlite` explicitly uses SQLite's C API; its bundled feature compiles SQLite through `cc`. It is a Rust API, but does not meet the dependency contract above. Use `redb` as the initial transactional backend, behind our own store interface. `redb` describes itself as a pure-Rust ACID embedded database. [rusqlite build documentation](https://docs.rs/crate/rusqlite/0.37.0), [redb documentation](https://docs.rs/redb/latest/redb/)

Commit generated protobuf Rust code. Regenerate it with a Rust `xtask` using `protox` and `prost-build::compile_fds`, keeping their descriptor types on compatible versions. Normal consumers must not need `protoc`, a C++ compiler, network downloads, or code generation during `cargo build`. This removes the proposed default `build.rs`. [protox documentation](https://docs.rs/protox/latest/protox/)

Audit upstream build scripts too: checking in our generated code does not remove a transitive dependency's `protoc` requirement.

### 2.3 TLS is an explicit dependency gate

rustls separates TLS from its cryptographic provider. A Rust HTTP client using rustls does not, by itself, prove that all its cryptography is Rust. Pin and inspect the provider actually selected by the chosen rustls/HTTP versions. Do not copy feature names between rustls versions. [rustls provider documentation](https://github.com/rustls/rustls#cryptography-providers)

The reviewed `rustls-rustcrypto` documentation describes a pure-Rust provider but currently warns against production use. It is a research candidate, **not an automatically approved production dependency**. Establish compatibility, security maturity, supported algorithms, and maintenance before relying on it. [rustls-rustcrypto documentation](https://docs.rs/rustls-rustcrypto/latest/rustls_rustcrypto/)

Graviola is a separate candidate if the project later chooses to allow Rust plus included assembly: its own documentation says it includes assembly and has architecture/CPU restrictions. “No C compiler required” is not the same as an entirely Rust cryptography implementation. Under the strict contract in this plan, do not silently select it. [Graviola documentation](https://github.com/ctz/graviola)

**Decision rule:** select an adequately reviewed pure-Rust provider that works with the chosen HTTP stack. If none qualifies, mark production HTTPS support blocked and document the exact deficiency. Offline work may continue. Do not weaken certificate validation, implement a new TLS stack, or quietly relax the language requirement to clear this gate. An experimental build must remain explicitly experimental.

### 2.4 Prove the dependency graph

For each supported release target and feature set:

1. Inspect `cargo metadata` and `cargo tree -e features` for normal, build, and development dependencies relevant to that job.
2. Audit build scripts and native linking. A crate name ending in `-sys` is a review signal, not a complete detection algorithm.
3. Reject unapproved native crypto/storage, subprocess runtimes, and code-generation downloads.
4. Include the eventual WhatsApp-enabled bridge executable in the audit before calling that executable pure Rust.
5. Keep an auditable dependency report with exact versions/revisions and enabled features.

Do not migrate unrelated `tzibbur-api` internals merely to make this new crate. Report any existing workspace dependency that prevents the intended final executable from meeting the contract.

## 3. Reuse protocol work without importing the wrong product

There is an existing Rust project, `oxidezap/whatsapp-rust`, whose documented structure includes a binary codec, Signal implementation, app-state logic, and protobuf definitions. Its documented authentication is QR/pair-code linking for the WhatsApp Web API. Those facts make it a candidate source of reusable components, not proof of Android primary-device support. [whatsapp-rust repository](https://github.com/oxidezap/whatsapp-rust)

Before writing a new binary codec or Signal state machine, evaluate that project's relevant Rust modules at a pinned revision. For each proposed reuse, record:

- Component boundaries and whether it can run without the parent client's linked-device login.
- Android-profile fixture compatibility, rather than just web-client compatibility.
- State-store integration, error handling, resource limits, and test coverage.
- Complete source provenance and file-level licensing, including inherited code.
- Rust-only build compliance and maintenance cost.
- Whether a registry release can depend on it, or whether a maintained vendored component is necessary.

Prefer a small, reviewed reusable component over a full-client fork. Keep any fork's patch set explicit. Do not make production builds fetch a floating branch.

The official Signal repository states that use outside Signal is unsupported and its APIs can change without notice. Its Rust protocol package exists in the repository, but that does not establish a supported crates.io dependency or WhatsApp compatibility. The current manifest also includes post-quantum components. Pinning “latest libsignal” is therefore not a compatibility strategy. [Signal repository](https://github.com/signalapp/libsignal), [protocol package manifest](https://github.com/signalapp/libsignal/blob/main/rust/protocol/Cargo.toml)

Select a Signal backend only after the interoperability gate in section 10. Preserve all applicable license notices; a repository's headline license does not resolve every imported file's provenance.

## 18. Dependency policy

Do not paste the original dependency versions into a new manifest. Select compatible, maintained releases when implementation begins, record them, and commit the workspace lockfile. Pin source dependencies by immutable revision. For a publishable package, every required dependency must have a valid registry/distribution strategy; a local path or Git-only reference is not a completed publication plan.

| Purpose | Preferred component or decision |
| --- | --- |
| Async runtime and networking | `tokio`, with actual required features including `io-util` where used |
| Cancellation and stream utilities | Targeted `tokio-util` / `futures-util` features only where used |
| Shared byte buffers | `bytes` |
| HTTP | `reqwest` with defaults disabled, or an injected Rust HTTP implementation; explicit reviewed TLS provider |
| Noise | `snow` Rust resolver if profile-compatible; otherwise the documented compatibility decision |
| Protobuf runtime | `prost`; avoid leaking generated types into the public API |
| Protobuf regeneration | `protox` + compatible `prost-build`, in Rust `xtask` only |
| Signal | Selected and pinned only after section 10's gate |
| Transactional store | `redb` |
| Storage AEAD | RustCrypto `chacha20poly1305` with XChaCha20 support |
| Wire-required hashes/KDF/MAC/ciphers | Specific RustCrypto components justified by evidence |
| Secret lifetime | `zeroize`, plus redacted wrapper types; `secrecy` if it simplifies the boundary |
| Randomness | OS-backed CSPRNG via compatible `rand_core`/`getrandom` integration; reconcile backend versions |
| Serialization | `serde` and a versioned representation appropriate to each record; `serde_json` for human-facing evidence/configuration |
| Typed errors/logging | `thiserror`, `tracing` with secret-safe fields |
| Object-safe async boundaries | `async-trait` only where dynamic dispatch actually requires it |
| Local IDs | `uuid` only if useful locally; wire IDs follow the profile |
| Property/adversarial tests | `proptest` and a Rust replay/mutation harness |

Optional dependencies need a concrete feature and tested graph. Do not enable native HTTP compression backends, native TLS defaults, browser transports or unneeded algorithms transitively. Multiple uses of the same primitive should converge on reviewed compatible implementations where practical, but distinct wire requirements take precedence over cosmetic dependency deduplication.
