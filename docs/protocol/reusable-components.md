# Reusable Rust component evaluation (README section 3)

Pinned revision: `oxidezap/whatsapp-rust` @ `4a0721b9aa944822807b95926e2770da4e670b2f`
(headline MIT; Cargo package `whatsapp-rust` 0.7.0, "Rust client for WhatsApp Web").

| Component | Boundary | Runs without linked-device login? | Android fixtures? | Build compliance | Licence / provenance | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| `wacore-binary` (codec, ~11k LOC) | Separate crate; deps `bytes`, `compact_str`, `smallvec`, `zlib-rs`, `yoke`, build.rs generates token maps from `tokens.json` | Yes (pure codec) | No (web tokens, proptest roundtrips) | Pure Rust; `zlib-rs` is Rust | MIT | **Data reused**: `tokens.json` vendored as a `ReferenceOnly` fixture. Code not reused: our codec is ~600 LOC, bounded by `Limits`, no build.rs; smaller review surface. |
| `wacore-noise` | Depends on `wacore-libsignal` (for XEdDSA cert verification) and `waproto` | Yes | No | Pure Rust | MIT, but transitively AGPL via libsignal | Not reused (dependency on AGPL files). Structure mirrored in `transport/cert.rs` with an injected verifier. |
| `wacore-libsignal` (~28k LOC) | Separate crate | Yes | No | Pure Rust (curve25519-dalek 5, x25519-dalek 3, RustCrypto) | **31 files: `Copyright Signal Messenger, LLC`, `SPDX-License-Identifier: AGPL-3.0-only`** | Blocked pending licence decision (ADR 0005). |
| `waproto` | `buffa`-generated code from a committed descriptor | Yes | — | Pure Rust | MIT; schemas from WhatsApp/whatsmeow | Not reused; we generate with `prost` from whatsmeow schemas (ADR 0006). |
| `whatsapp-rust` client | Full linked-device client (QR/pair code), sqlite storage (C) by default | **No** | No | Default features pull `libsqlite3-sys` (bundled C) and `ring` via rustls | MIT | Not a candidate for the primary-device product. |

`tulir/whatsmeow` @ `2375e1751bbd7243b7d4bbb3f410f1acf34f3382` (Go, MPL-2.0) was
used as a second, independent reference for constants and for the protobuf
schemas (vendored with its LICENSE). No Go code is used.

Registry publication: neither Rust project publishes the component crates in
a form we could depend on from crates.io with a pinned, reviewed revision;
any reuse would be a vendored, patch-tracked copy.
