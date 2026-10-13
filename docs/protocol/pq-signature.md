# Directory PQ signature inspection

**Superseded 2026-10-13:** primary preview/application now derives a hybrid
session from a verified `pqkey` reply and `key_validation` reports the ML-KEM-1024
encoding check. See [hybrid identity refresh](hybrid-identity-refresh.md).

2026-10-12. Pinned APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

Identity refresh inspection now reports optional `pq_signature`, verifying the
selected directory PQ value's signature against the returned candidate identity.
It is signature-only diagnostics, not KEM format validation, human verification,
session derivation or adoption. [The consumable bundle boundary](pq-bundle-boundary.md)
still refuses PQ, and preview/application remain `pq_session_unported` even
when this signature verifies.

## Native byte domain and remaining primitive boundary

The [four-topic recipe](apk/recipes/2.26.37.71-pq-signature.json) reproduces the
[bundle](apk/2.26.37.71-pq-signature/targets.json) with full signatures, register
flow and catch tables. Complete REA Evidence retains the
[class](rea/2.26.37.71-pq-signature/public-class.json),
[constructor](rea/2.26.37.71-pq-signature/public-constructor.json),
[byte accessor](rea/2.26.37.71-pq-signature/public-bytes.json) and
[encapsulation boundary](rea/2.26.37.71-pq-signature/encapsulation.json).

`KEMPublicKey.<init>` assigns the supplied array to A00 without parsing or adding
a type prefix; A01 returns it unchanged. `CQc.A00` at `0068..00b4` verifies
that exact A00 byte array with the candidate identity through the same signature
adapter as the classic signed key. Failure throws an invalid Kyber-signature
exception. The inspector therefore uses original `pqkey/value` bytes directly;
it does not prepend EC type 5, an ID/version, or normalize/re-encode the value.
The existing Signal signature primitive performs the verification.

`KEMPublicKey.A00` obtains a 32-byte SHA1PRNG SecureRandom seed and calls JNI
`encapsulateNative` with the original array and seed. The returned shared secret
is appended after the EC agreements in `CQc.A00`. The hybrid HKDF label at
`028c..0296` is `WhisperText_X25519_SHA-256_CRYSTALS-KYBER-1024`; the classic
branch uses `WhisperText`. This identifies the Java suite label, not the JNI
algorithm revision, deterministic encoding, key lengths or full compatibility.
The subsequent [native JNI trace](native-kem.md) resolves dispatch to ML-KEM
and shows that Java seed contents are not consumed. Primitive equivalence and
the Rust hybrid backend remain unported.
These observations are static APK evidence, not execution of JNI or Android.

## API and CLI result

The existing `identity_notifications::inspect`, session-preview inspection and
`actions --show ACTION_ID` result include nullable **pq_signature**. Null means
absent selected metadata or an unavailable classic candidate. Otherwise it has:

- `status`: `malformed`, `payload_too_large`, `signature_verified` or `invalid_signature`.
- `value_bytes`: nullable original value length, never a recognized key type.
- `signature_valid`: null before crypto verification, otherwise its Boolean result.
- `key_validation`: `not_checked` (value absent or above the local 64 KiB bound),
  `valid_ml_kem_1024` or `invalid_ml_kem_1024` since 2026-10-13; formerly always `unported`.

Top-level `signature_valid` continues to refer only to the classic signed prekey.
PQ status does not prove key ID, KEM format, algorithm, encapsulation or admission.
Even an empty opaque value can have a valid signature; it is not thereby a usable
KEM key. Required byte-body and 64-byte signature checks, duplicate refusal and
the local 64 KiB inspection bound are explicit local shape/resource boundaries,
not newly inferred native admission policy. No bytes, peer address, key ID,
signature, notification ID or fingerprint are returned by this diagnostic.

## Verification and preservation

The regression failed with the new diagnostic absent. Generated Signal signatures
cover exact original bytes, altered values, mistakenly prefixed inputs, wrong
candidate identities, empty values, duplicate nodes/children, missing/nonbyte
bodies, wrong signature width and the local size bound. These opaque byte vectors
are not valid native Kyber fixtures. Inspection/preview/application leave all
saved tables and the journal unchanged; reopening reproduces the diagnostic.
Verified and failed PQ signatures both leave session creation deferred. Classic
signature reporting stays independent and private canaries never appear in JSON.

No new query, retry, send, inventory key, effective property, dependency or
cryptographic admission rule is introduced. Saved account/peer identities,
sessions, IDs, uncertain sends and waits are preserved. Full KEM/session support,
native primitive equivalence and fresh live PQ interoperability remain unfinished
or unverified. No live account operation ran for this change.

Validation passed 971 workspace/all-feature tests, strict all-target Clippy,
warnings-denied Rustdoc, the diagnostic/admission/reopen matrix with TCP/Noise/Signal
features, 26 default CLI/API-symbol tests and the experimental CLI build.
The four-topic APK bundle reproduced all nine files / 52,928 bytes exactly.
