# Experimental Rust ML-KEM-1024 primitive

2026-10-12. `signal::kem` under `experimental-signal-crypto` now supplies
ML-KEM-1024 key generation, encapsulation and decapsulation. It uses pinned
[RustCrypto ml-kem 0.3.2](https://docs.rs/ml-kem/0.3.2/ml_kem/), not custom
cryptographic arithmetic. Cargo.lock records checksum
`5e15f3e5b957493873e396a66914e83e616b6afe335cdef7efe5c6e1216aba66`.
MIT OR Apache-2.0 and zeroize are recorded in the third-party notices.
Upstream explicitly reports no independent audit; vector agreement does not
substitute for one. The feature stays experimental.

## Public primitive API

- `generate_key_pair()` returns a 1568-byte public key and a `KemSecret`
  containing the native expanded 3168-byte secret key. OS entropy seeds it.
- `encapsulate(public)` validates the ML-KEM-1024 public-key encoding and uses
  the existing crypto RNG, returning a 1568-byte ciphertext and 32-byte secret.
- `decapsulate(secret, ciphertext)` validates expanded secret-key encoding and
  ciphertext length, returning a 32-byte secret. Modified ciphertexts use
  implicit rejection; successful decapsulation is not message authentication.
- `KemSecret::as_bytes()` explicitly borrows secret material for crypto/storage.
  It is not serializable or cloneable; Debug is redacted and owned storage is
  zeroized on drop. Callers must protect any copies they make.

No deterministic seed/encapsulation option is exposed by the application API.
The dependency's deterministic entry is used only by the fixture tests. The
upstream expanded-key API is deprecated in favor of seeds; this adapter uses
it deliberately because the [APK JNI boundary](native-kem.md) uses expanded
keys. This does not specify a new saved-account format or populate a KEM key
inventory. Primitive validation is a local Rust boundary; full equivalence of
JNI rejection and arbitrary malformed-key handling remains unverified.

## Evidence and regression

The Rust test consumes the same 16 pinned public NIST vectors as the
[offline native execution](native-kem-vectors.md). It compares both serialized
key outputs, deterministic ciphertext/shared secret outputs, and all ten
valid/modified-ciphertext decapsulation secrets byte-for-byte. Expected outputs
come from the NIST corpus, not Rust round trips. A separate randomized adapter
round trip checks the production generation/encapsulation path, wrong widths,
invalid public coefficients, invalid expanded secret hash, redacted Debug and
secret-free errors.

The signature inspector, selected PQ bundle refusal, identity refresh deferral
and classic backend remain separate. This module cannot adopt an identity,
allocate a request/prekey ID, write a store, query a peer or send a message.
Full PQ support still needs hybrid agreement order/HKDF, outgoing/incoming
prekey fields, session record encoding, local key inventory and atomic
identity/session integration. No live account operation was performed.

Validation passed 973 workspace/all-feature tests, the final two-test primitive
regression, strict workspace/all-target Clippy, warnings-denied Rustdoc, the
generated dependency audit, 26 default CLI/API-symbol tests, the experimental
CLI build, formatting and local link/artifact hygiene. No fresh live PQ
interoperability was tested.

The subsequent [initial hybrid KDF helper](hybrid-kdf.md) implements the APK's
ordered premaster composition and root/chain split. Full session admission,
wire/state integration and incoming processing remain unfinished.
