# WaRustAPI

> **Status: experimental and incomplete. Not production-ready.** Interfaces,
> storage formats and behaviour change without notice.

Experimental Rust workspace for a standalone WhatsApp **primary-device** client.
The goal is phone-number registration and messaging without a paired phone or
another running WhatsApp client.

**First live message acknowledged by WhatsApp.** The experimental comparison
flow completed registration, authenticated login and one encrypted text send.
The recipient confirmed receipt; one text reply and one protocol sync payload were decrypted. General messaging remains incomplete. See
[STATUS.md](STATUS.md) and [transport usage](docs/protocol/messaging-transport.md).

## Get started

Linux x86_64 is the initial proof target. Install Rust through
[rustup](https://rustup.rs/); this repository pins Rust 1.98.1 in
[rust-toolchain.toml](rust-toolchain.toml).

```sh
cargo build --locked --workspace
cargo run --locked -p whatsapp-cli -- --help
cargo run --locked -p whatsapp-cli -- profile
cargo run --locked -p whatsapp-cli -- capabilities
```

These inspection commands need no account or APK. For local storage and the
HTTP control plane, see [Getting started](docs/getting-started.md).
Enqueueing a message currently stores an intent; it does not send it.

Onboarding a phone number is two commands (`accounts new`, then `lab-signup begin`); see [docs/accounts.md](docs/accounts.md).
Every command is listed in [docs/cli.md](docs/cli.md) and described in [docs/api/openapi.json](docs/api/openapi.json).

## Workspace

| Package | Purpose |
| --- | --- |
| [whatsapp-api](crates/whatsapp-api/README.md) | Async library, encrypted store, codecs, protocol boundaries |
| whatsapp-cli | Operator CLI and local HTTP inspection |
| xtask | Rust-only schema, dependency, and APK analysis tools |

Optional `experimental-noise`, `experimental-registration-https`,
`experimental-signal-crypto`, and `experimental-transport` remain lab features.
HTTPS uses owner-approved native `ring` crypto; the Signal adapter includes
AGPL-marked dependencies. See [third-party notices](THIRD_PARTY_NOTICES.md).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

Read [CONTRIBUTING.md](CONTRIBUTING.md) for setup, feature checks, generated code,
and the full validation workflow. First-party code and automation are Rust.

## Documentation

- [API reference (OpenAPI)](docs/api/README.md) — every CLI command and HTTP route, machine-readable
- [CLI reference](docs/cli.md) — generated command tree
- [Configuration reference](docs/configuration.md) — request.json fields and profiles
- [Documentation index](docs/README.md)
- [Running more than one account](docs/accounts.md) — profiles, onboarding a second number, the two-account pair check
- [Current status](STATUS.md)
- [Design plan and numbered section index](docs/design/README.md)
- [Protocol evidence](docs/protocol/evidence-ledger.md) and [APK research](docs/protocol/apk-analysis.md)
- [Architecture decisions](docs/README.md#architecture-decisions)
- [Security reporting](SECURITY.md)

## License

First-party code is licensed under [MIT](LICENSE-MIT) OR
[Apache-2.0](LICENSE-APACHE), matching the Cargo manifests.
[Third-party notices](THIRD_PARTY_NOTICES.md) describe separately licensed inputs.
This project is not affiliated with WhatsApp or Meta.
