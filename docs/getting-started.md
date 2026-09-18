# Getting started

[Documentation index](README.md)

The supported proof environment is Linux x86_64. Use rustup so
[rust-toolchain.toml](../rust-toolchain.toml) selects Rust 1.94.1; a distribution
Cargo binary may ignore that file. Verify with `rustc --version`.
The lockfile is committed. Initial builds need access to the crate registry.

## Build and inspect

```sh
cargo build --locked --workspace
cargo run --locked -p whatsapp-cli -- --help
cargo run --locked -p whatsapp-cli -- profile
cargo run --locked -p whatsapp-cli -- capabilities
```

The profile identifies missing protocol evidence. Live registration, login,
and message transmission remain gated; these commands do not request an OTP.

## Local encrypted store

Set `WHATSAPP_API_MASTER_KEY` to a 32-byte key encoded as 64 hex characters,
obtained from your secret manager. `whatsapp-cli keygen` can generate a fresh
key; its stdout is secret. Keep it outside the database and retain it across
restarts. Do not generate a new key for an existing store.

```sh
cargo run --locked -p whatsapp-cli -- --store protocol.redb store init
cargo run --locked -p whatsapp-cli -- --store protocol.redb store inspect
cargo run --locked -p whatsapp-cli -- --store protocol.redb events
```

The default account is `default`; select another with `--account`.
Use one process per account database. Store files and environment files are
ignored by Git. Backup/restore is not implemented.

## Local HTTP inspection

```sh
cargo run --locked -p whatsapp-cli -- serve --bind 127.0.0.1:8787
```

From another terminal:

```sh
curl http://127.0.0.1:8787/healthz
curl http://127.0.0.1:8787/v1/profile
curl http://127.0.0.1:8787/v1/capabilities
curl http://127.0.0.1:8787/v1/transport/wire-profile
curl http://127.0.0.1:8787/v1/registration/wire-profile
```

The HTTP interface has no authentication. Keep the default loopback binding.
The wire-profile routes describe the comparison APK, not a working connection.
Network mutation endpoints return structured errors while their gates remain
unresolved.

## Next steps

See [contributor checks](../CONTRIBUTING.md), the
[library usage sketch](../crates/whatsapp-api/README.md), and
[APK research](protocol/apk-analysis.md).
