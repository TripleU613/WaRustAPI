# ADR 0005: No Signal backend selected; boundary and overlay only

Status: accepted (gate open) · 2026-09-17

## Owner approval — 2026-09-17

The owner approved evaluating and integrating AGPL-bearing Signal dependencies
to advance standalone onboarding. This clears the permission gate, not the
interoperability or release gates. Preserve upstream notices and disclose the
dependency license rather than treating a crate's MIT headline as authoritative.
The first-party license declarations are unchanged; publication remains disabled.

Current candidate inspected: `oxidezap/whatsapp-rust` revision
`ae3cefd86065872a577bbd0ad5ee21b60c86c616`, `wacore-libsignal` 0.7.0.
Its `src/core/curve.rs` still carries Signal's AGPL-3.0-only header.
Only the crypto/protocol component is a candidate, not the upstream Web client
or QR-linking flow. Android registration and live interoperability still need
independent evidence.

Initial integration uses the exact published registry version 0.7.0, whose
packaged VCS revision is `f8165f282008935732e0b26d6f5cca5038ecd222` (not the
later inspected main revision above). `experimental-signal-crypto` provides
`SignalSignatureVerifier` for the existing certificate-chain boundary.
The source header was checked in the actual registry package. No upstream
code is copied or relicensed. The adapter test checks signatures and tampering;
it is not an independent Android vector. `NoBackend` remains the messaging
backend until session/prekey/ratchet integration and interoperability pass.

The same feature now also provides the persisted registration lab factory:
distinct Noise/Signal identities, signed prekeys using the upstream signing
convention, and encrypted transactional persistence. Android acceptance of
that convention is still unobserved; see [lab boundary](../protocol/registration-lab.md).

## Candidates examined

| Candidate | Finding |
| --- | --- |
| `oxidezap/whatsapp-rust` `wacore-libsignal` @ 4a0721b | Pure Rust (curve25519-dalek, x25519-dalek, aes, hkdf, hmac, sha2). **31 files carry `Copyright Signal Messenger, LLC` / `SPDX-License-Identifier: AGPL-3.0-only` headers** despite the crate's MIT headline. Depending on it makes the release AGPL-encumbered. Path/git dependency only (workspace crate); state format is its own. Web-profile fixtures only. |
| `signalapp/libsignal` `rust/protocol` | Upstream states use outside Signal is unsupported; AGPL-3.0; includes post-quantum components not known to be accepted by WhatsApp; no crates.io release. |
| Own implementation from primitives | Forbidden by docs/design/README.md 10.1 ("do not implement cryptographic primitives from scratch") for primitives; a protocol-level implementation over RustCrypto primitives would still need independent vectors and interop. |

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
the seven vector classes from docs/design/README.md 10.1 satisfied.
