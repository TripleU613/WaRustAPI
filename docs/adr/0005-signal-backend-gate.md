# ADR 0005: No Signal backend selected; boundary and overlay only

Status: accepted (gate open) · 2026-09-17

## Candidates examined

| Candidate | Finding |
| --- | --- |
| `oxidezap/whatsapp-rust` `wacore-libsignal` @ 4a0721b | Pure Rust (curve25519-dalek, x25519-dalek, aes, hkdf, hmac, sha2). **31 files carry `Copyright Signal Messenger, LLC` / `SPDX-License-Identifier: AGPL-3.0-only` headers** despite the crate's MIT headline. Depending on it makes the release AGPL-encumbered. Path/git dependency only (workspace crate); state format is its own. Web-profile fixtures only. |
| `signalapp/libsignal` `rust/protocol` | Upstream states use outside Signal is unsupported; AGPL-3.0; includes post-quantum components not known to be accepted by WhatsApp; no crates.io release. |
| Own implementation from primitives | Forbidden by README 10.1 ("do not implement cryptographic primitives from scratch") for primitives; a protocol-level implementation over RustCrypto primitives would still need independent vectors and interop. |

None has passed the section 10 gate (independent vectors 1–7).

## Decision

- `src/signal/` defines the adapter trait (`SignalBackend`), record keys
  with non-colliding `(kind, id)` prekey identifiers, sender-key triples, and
  a transactional `Overlay` with expected-revision commit (tested: a stale
  snapshot cannot commit).
- The only backend is `NoBackend`, which returns
  `Error::Unsupported(DirectText)`. No wire incompatibility is hidden.

## Exit criteria

Either a licence decision by the project owner permitting AGPL components
(then evaluate `wacore-libsignal` against Android-profile vectors), or a
clean-room protocol implementation over reviewed RustCrypto primitives with
the seven vector classes from README 10.1 satisfied.
