# Initial hybrid Signal key derivation

2026-10-12. Pinned APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
`signal::kem::initial_hybrid_keys` now derives the initial hybrid root and chain
keys under `experimental-signal-crypto`. It accepts already-computed agreements;
it does not admit a directory bundle, select keys, allocate IDs or create a
session. Full identity/session and prekey wire integration remain unfinished.

## Native composition and exact KDF

The [six-topic recipe](apk/recipes/2.26.37.71-hybrid-kdf.json) retains full DEX
signatures, register flow and exception tables in a
[reproducible eleven-file bundle](apk/2.26.37.71-hybrid-kdf/targets.json).
Complete REA evidence preserves the
[key split](rea/2.26.37.71-hybrid-kdf/split-keys.json),
[extract](rea/2.26.37.71-hybrid-kdf/extract.json),
[HMAC wrapper](rea/2.26.37.71-hybrid-kdf/hmac-wrapper.json),
[expand](rea/2.26.37.71-hybrid-kdf/expand.json) and
[counter subclass](rea/2.26.37.71-hybrid-kdf/counter-variant.json).
The constructor has no decompiled body in REA; DEX confirms its Object
constructor call. The counter choice is established by Cew.A01's instanceof
branch, not by inventing a constructor implementation.

`CQc.A00` writes this input in order:

| Offset in method | Input component | Alice side |
| --- | --- | --- |
| `01d6..01ec` | 32 discontinuity bytes `0xff` | fixed prefix |
| `01ec..020a` | first EC agreement | local identity private / remote signed prekey public |
| `020a..0228` | second EC agreement | local base private / remote identity public |
| `0228..0236` | third EC agreement | local base private / remote signed prekey public |
| `0236..0258` | optional fourth EC agreement | local base private / remote one-time public |
| `0258..027a` | KEM shared secret | encapsulation output appended last |

The Rust boundary requires each already-computed input to be 32 bytes, covering
ML-KEM-1024 as separately verified in the [native primitive vectors](native-kem-vectors.md).
Its premaster input is 160 bytes without a one-time key or 192 bytes with one.
It does not infer that arbitrary 32-byte strings are authenticated DH/KEM values.
All agreements and the KEM result must come from the appropriate verified keys.

`CQc.A00` constructs DZC and selects ASCII info
`WhisperText_X25519_SHA-256_CRYSTALS-KYBER-1024`. CBN.A00 requests **64 bytes**
from Cew.A02 and splits them 32/32: first half into COt root, second into Ckg
chain with initial index zero. Cew.A02 extracts with HMAC-SHA256 under **32
zero salt bytes**. Cew.A01 initializes its expansion counter at **1** for DZC
and computes each block from previous block + info + the one-byte counter.
This is standard HKDF-SHA256 for this variant; the other counter-zero subclass
is a separate legacy path and is not implemented by this helper.

The existing HKDF dependency performs the Rust derivation. Owned premaster, temporary
output and returned secret storage are zeroized; returned values use the same
redacted KemSecret as the [KEM primitive API](rust-kem.md). Public ordering is
`identity_signed, base_identity, base_signed, optional base_one_time, kem_shared`;
return order is `(root, chain)`.

## Independent check and API scope

The [independent HMAC reproducer](../../xtask/src/vectors.rs) (built on the
`hmac`/`sha2` primitives, not the library's HKDF; formerly a Python script)
creates [16 synthetic fixtures](hybrid-kdf-vectors.json): eight distinct input
patterns, each with and without the optional agreement. Zero/ff and distinct
per-position byte patterns exercise the prefix, ordering, optional component,
KEM tail, label, block counters and half split. Rust compares both complete
32-byte outputs. These are independent HMAC computations on synthetic
already-computed agreements, not APK execution, actual DH results, live KEM
samples or an end-to-end session interoperability check.

```sh
cargo run --locked -p xtask -- research-vectors hybrid-kdf --check
cargo test -p whatsapp-api --all-features \
  signal::kem::tests::hybrid_initial_keys_match_independent_hmac_vectors
```

The regression first failed because the helper was absent. The existing
ML-KEM primitive fixture and randomized round-trip regressions remain separate.
No CLI key-material command is added. The low-level Rust API is documented;
selected PQ bundles still refuse, and identity preview/application still defer.

## Remaining backend integration

The locked wacore-libsignal 0.7.0 classic ratchet helper uses `WhisperText` and
expands 96 bytes, exposing an extra initial PQR key that its Alice initializer
ignores. Its first 64 bytes are the classic root/chain halves, but its label
cannot be used for the hybrid branch. This adapter requests the evidenced 64
bytes under the exact APK hybrid label; it does not infer a new PQR ratchet.

The APK sets session version 4 when its KEM optional is present (CQc.A00
`016c..01aa`) and writes a separate pendingKyberPreKey ID/ciphertext record at
`02c6..036a`, followed by the ordinary pending prekey record. Backend components
can carry optional Kyber fields, but its pending-prekey setter assigns None,
its prekey message constructor lacks KEM inputs, and its classic initializer
derives without the KEM tail. Legacy conversion also refuses nonempty Kyber
fields. These remaining paths must be integrated before normal PQ messaging.
Do not substitute this KDF helper for completed session support or lift the
current consumable bundle guard merely because the outputs derive.

No dependency or at-rest format changed in this chunk. Saved identities, keys,
requests, uncertain sends, waits and sessions are untouched; no live operation
ran. Native admission/signature authority, agreement calculation, session chain
construction, incoming prekey processing and fresh interoperability remain open.

Validation passed 974 workspace/all-feature tests, including all three KEM/KDF
regressions, strict all-target Clippy, warnings-denied Rustdoc, 26 default
CLI/API-symbol tests, the experimental CLI build, formatting and diff checks.
The six-topic APK bundle reproduced all eleven files / 57,422 bytes exactly;
the independent HMAC fixture reproduced all sixteen cases. Local link/artifact
hygiene passed. No live hybrid session was exercised.
