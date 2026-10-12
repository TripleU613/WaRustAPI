# Fresh signed hybrid bundles

2026-10-12. Experimental `signal::hybrid::bundle_for` selects one exact PN/LID
recipient, validates the classic signed key and the original ML-KEM-1024 value
signature, and returns a redacted `HybridBundle`. Its `prepare_initial` stages
native version-4 encrypted bytes and a canonical session candidate for an
already-authorized bundle. It does not write a store or contact the server.

## Evidence and implemented behavior

The retained APK 2.26.37.71 [bundle boundary](pq-bundle-boundary.md) traces
Bot.A03 selecting `pqkey` id/value/signature, 0jc.A0L constructing its public key,
and CQc.A00 verifying both EC and original KEM bytes under the candidate identity.
The [hybrid derivation](hybrid-kdf.md), [wire](signal-wire.md) and
[session](hybrid-session.md) evidence establish the downstream construction.
This chunk reuses those sources; it makes no new native-execution claim.

The parser reuses exact full-JID selection and classic shape checks. Missing or
duplicate selected users/fields refuse. KEM value/signature must be byte nodes;
the signature must be 64 bytes and the public key must be a valid 1568-byte
ML-KEM-1024 encoding. Its three-byte ID rule is the existing local bundle rule.
Signature verification uses the original value, not a normalized replacement.
The object does not expose material through Debug or serialization.

Preparation requires a registered account with a consistent local identity pair,
nonempty payload up to the local 32768-byte bound, and any supplied peer pin to
match. It generates fresh base/ratchet keys, encapsulates the selected KEM key,
retains ordinary/signed/KEM IDs and returns the existing PreparedDirect candidate.
An ordinary one-time EC prekey is optional. No held session is reset or accepted;
request IDs, wire IDs, retries, server waits and account state are untouched.
The caller must commit the candidate atomically before sending it, as for other
staged preparations.

## Validation and remaining integration

The fixture selects PN and LID peers beside another device's error row. Actual
EC signatures and KEM encapsulation/decapsulation establish matching Alice/Bob
states; Bob decrypts the prepared payload with and without an ordinary prekey.
Wrong peer/device, duplicate targets/fields, missing metadata, invalid signatures,
signed malformed KEM keys, incorrect ID widths, text values, changed pins,
unregistered accounts, oversized payloads and inconsistent local keys refuse.
A private text canary does not appear in errors.

Cryptographic validity under a candidate identity does not establish directory
reply correlation, freshness, companion authority, attestation policy or approval
of a changed identity. Normal outgoing selected-PQ admission and incoming KEM
bootstrap guards remain active. Coordinator adoption, local KEM inventory/upload,
incoming lookup/consumption and live Android/server interoperability remain open.
No new dependency, CLI command or network operation is introduced.

Validation: 978 workspace/all-feature tests, the crypto-only encrypted bundle
fixture, strict all-target Clippy, warnings-denied Rustdoc, 26 default CLI/API
symbol tests, experimental CLI build and formatting passed. No live operation
was exercised.
