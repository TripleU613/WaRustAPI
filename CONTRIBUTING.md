# Contributing

Read [current status](STATUS.md), the [design index](docs/design/README.md),
and the relevant [architecture decision](docs/README.md#architecture-decisions)
before changing a protocol boundary.

## Setup

Use rustup and the checked-in Rust 1.94.1 toolchain. Linux x86_64 is the proof
target. Run `rustc --version` to confirm the selected compiler; system Cargo
installations may ignore `rust-toolchain.toml`. Build with
`cargo build --locked --workspace`. No APK, account, Java, or protoc is needed
for normal builds and tests.

## Checks

Run these from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
cargo test --locked -p whatsapp-api --features experimental-noise
cargo test --locked -p whatsapp-api --features experimental-registration-https
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
cargo package --locked -p whatsapp-api --allow-dirty
git diff --check
```

The current optional features are compatible. Normal CI tests default and
combined features for library changes; weekly/manual extended checks also test
each feature independently. CLI and tooling edits use scoped tests, and docs
edits skip Cargo builds. See [CI routing and checks](docs/ci.md).
If new backends are mutually exclusive, update the matrix.
Dependency audit checks the project's build contract, not vulnerability
advisories. Review its generated report before committing it.

## Changes and review

Keep changes focused. Explain the problem, resulting behavior, validation,
and unresolved limitations in your pull request. Add meaningful regression
coverage for behavioral changes. Update the relevant docs and status when
capabilities change. Do not claim mocked or offline tests prove live service
interoperability.

Any change to a CLI command, flag, HTTP route, library capability or observed
server response code updates `docs/api/openapi.json` in the same change (see
[docs/api/README.md](docs/api/README.md)). `cargo test -p whatsapp-cli` fails
when a command is missing from the document.

First-party automation belongs in the Rust `xtask` crate. Avoid unrelated
dependency upgrades, new runtimes, or empty modules for planned features.
Use `cargo run --locked -p xtask -- proto-gen` to regenerate protobuf code;
do not hand-edit `crates/whatsapp-api/src/proto/`. Preserve provenance and
third-party license notices.

The root README is an overview. Design requirements live in `docs/design/`,
research findings in `docs/protocol/`, and decisions in `docs/adr/`.
Original design section numbers remain stable for source references.

## Research and sensitive data

Use [the APK procedure](docs/protocol/apk-analysis.md) and record exact hashes,
versions, dates, and evidence labels. Keep APKs and raw research output in
ignored `lab/` or outside the checkout. Fixtures must be sanitized.

Live registration requires a controlled test number and explicit operator
consent. Normal CI must not contact WhatsApp, request OTPs, or mutate accounts.
Do not include keys, PINs, OTPs, account databases, private messages, or
unsanitized captures in issues or pull requests. Follow [SECURITY.md](SECURITY.md)
for sensitive reports.
