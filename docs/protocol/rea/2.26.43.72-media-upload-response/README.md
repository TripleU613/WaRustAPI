# Upload JSON response modes

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX outputs, not runtime observation or new REA Evidence envelopes.

MtZ.Cb6 registers MtR(selector2), whose CDR default branch creates LzN
and parses nonnull response JSON. url,handle,enc_handle use optString.
mcs_request_id has separate null/pattern handling. direct_path goes through
KmR.A00(value,MKA.A01): nonnull input, true mode and Lkh regex match
return the original; otherwise null. Matched value goes to LzN.A00 with
meta_hmac; unmatched value is retained separately as LzN.A02.
Thumbnail direct path has analogous mode selection into A09/A0A. Other
fields include fbid,ts,thumbnail_sha256,metadata_url,dash_manifest_url, and
EIF-derived list. Complete downstream use/requirements are not established
just by optString reads. JSON exception sets MKA.A03 rather than silently
creating a successful upload.

Lkh literal pattern is lowercase UUID-v4 shape:
`[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}`.
06K.A07 matching semantics are not independently traced here. This is a
mode-specific opaque handle pattern, **not** a slash-path validator.

The retained MtZ.Cb6 status flow distinguishes400, negative transport codes
and0..399 before mode/handle checks and later result classification. It is
not equivalent to a universal HTTP200 parser accepting any optString value.
[Full previously retained producer](../2.26.43.72-media-transfer-results/upload-result-producer.txt).

Rust locations() currently requires HTTPS approved URL without credentials/
explicit nondefault port and a slash-prefixed direct_path under local bounds.
It is invoked only after durable complete reply storage and HTTP200. This
is a supported local comparison subset, not complete native response parity.
No UUID requirement is added to slash paths and no success is fabricated from
optional metadata. Mode producer MKA.A01/A02, LzN consumers, full MIME/media
reply requirements and alternate handle modes remain open. No runtime
acceptance rule, stored result or live account operation changed.

Bounded discovery: exact direct_path anchor finds99 const-string sites; this
pass selects MtR.CDR via the known MtZ callback, not every99 consumer.
Reproduce apk-xref direct_path; apk-smali MtR.CDR,KmR.A00,Lkh.<clinit>
with signatures/exceptions. Outputs below are complete selections.

- [direct-path-xref.txt](direct-path-xref.txt) — SHA-256 `5aabcb5d5ff4cf9d56b36802c11475740a8d93941b374c1c0a7699cf0110adc5`.
- [response-callback.txt](response-callback.txt) — SHA-256 `4dcdc74000ec5e07629e6af6e6b6578fc937f6c9148957b9425b7fa62cf74cb7`.
- [direct-path-mode.txt](direct-path-mode.txt) — SHA-256 `81c83f1c5d9dc144740daa35669c297d1d47fcfc5d01c535aebd9bd27a051f17`.
- [opaque-handle-pattern.txt](opaque-handle-pattern.txt) — SHA-256 `94ca81e108f5cb76a52b8069cd33fdf43d1d067a74338717bb52e3e0822be44b`.

## Mode producers and scheduled cross-reference consumer (2026-11-04)

**Confirmed static**, same pinned APK. Exact write scans find one writer
each for MKA.A01/A02, both in retained MtZ.Cb6. LzO constructor preserves
the three trailing boolean parameters as A0D/A0E/A0C; MtZ forwards A0C
to MKA.A01, A0E to MKA.A02. At descriptor construction, A0C is true iff
LIX.A04 is nonnull, and A0E is the inverse of MvM.A06. LIX.A04's higher
producer remains untraced; naming this input by field identity avoids
assuming its full semantic type. The earlier M0p.A00 contributes another
boolean, not these two mode flags; full helper retained to preserve that
negative distinction.

MOu.A07 adoption reads LzN.A00 and, when Lyh.A04 permits and nonempty,
passes it to KmQ.A00. KmQ's log explicitly identifies verification of
"scheduled cross_reference". It validates using Lkh's UUID-v4 pattern
and returns original or null, with diagnostic logging on malformed/empty
input.06K.A07 invokes Pattern.matcher(...).matches(), confirming whole
string matching, not substring search. The adopted value goes through
MDL.A0A and clears the alternate path via MDL.A0B(null). Otherwise
LzN.A02 passes through M0g.A00(value,false) into MDL.A0B. Thumbnail
values take analogous branches. No names are inferred for remaining
MDL storage fields without their producers/consumers.

Direct LzN.A02 reference scan finds only callback write, MtZ validation
and MOu adoption. This bounds exact direct accesses, not reflection or
transitive object consumers. The scheduled cross-reference is therefore
not a slash-path validator and should not be accepted as an ordinary
Rust media direct_path. M0g normalization, LIX.A04 producer and complete
post-adoption status handling remain open. Rust acceptance remains unchanged.

Reproduce apk-field-refs MKA.A01/A02:Z --writes-only, LzN.A02:String;
apk-smali MOu.A07,KmQ.A00,06K.A07,M0p.A00 with signatures/exceptions.
Existing MtZ/LzO exports carry the constructor/flag dataflow above.

- [ordinary-path-consumers.txt](ordinary-path-consumers.txt) — SHA-256 `f7a1504e957c89ac988b9d7d657e7adb1def3c112b44d893afd4eb80b286e054`.
- [result-adoption.txt](result-adoption.txt) — SHA-256 `bb61ae0b269829016ec3ec32f400376f65c69b5c0eee53a8831760626adbe4ef`.
- [cross-reference-verifier.txt](cross-reference-verifier.txt) — SHA-256 `88445b59c5331695728c769532ecca515efce30079bec350636e522b8bb8a786`.
- [regex-match-semantics.txt](regex-match-semantics.txt) — SHA-256 `e4eb216de6fa46a789777e86ef23d63dfd213a7c3b5fcd0981619930f8011f0d`.
- [mode-helper.txt](mode-helper.txt) — SHA-256 `072506169f0fa7d0be8c5f8664936dffe942d418cd5d1d5ece9ba7851bf72023`.
- [opaque-mode-writers.txt](opaque-mode-writers.txt) — SHA-256 `ca1a1df2da4c5b94072237341e4e7239ba2d172882c29a7d179dcb1179962af0`.
- [required-path-mode-writers.txt](required-path-mode-writers.txt) — SHA-256 `6cab8100fe1282c1f43875a507dd2c9db70193ff22258ed87bd9ac08d5c0125a`.

## Ordinary path query-presence correction (2026-11-04, offline)

**Confirmed static/source mismatch fixed:** M0g.A00 returns null for empty
path and for Uri query lacking either oe or oh; both nonnull query values
return the exact original string. Empty values count as present. The boolean
argument changes empty-path diagnostics, not this query requirement. MDL.A0B
then stores the supplied string directly under its monitor. This leaf does
not check signature cryptography or parse expiry time.

Rust locations() now applies the same presence requirement to its existing
slash-path subset after HTTPS/host/path guards. It compares exact encoded
query keys (case-sensitive), excludes fragment text, and retains the entire
original accepted path without rewriting query order/values. Android platform
[Uri.getQueryParameter](https://android.googlesource.com/platform/frameworks/base/+/b10b48f62d3cac684424e4181d4e8ec61f227e95/core/java/android/net/Uri.java)
confirms raw encoded-key matching and empty-value presence. Percent-encoded
key aliases do not count as oe/oh; ampersand, not semicolon, separates pairs.

The complete HTTP response is already stored encrypted as response_received
before locations() runs. Missing required query fields therefore refuse
payload construction without erasing evidence or authorizing another upload.
No expiration value is invented and no retry is enabled. Previously uploaded
legacy payloads remain reusable; this changes only new response acceptance.

A regression covers ordinary/reordered/empty parameters, fragment-only fake
parameters, missing keys, encoded aliases, case mismatches, value-embedded
keys and semicolon separation. Existing untrusted URL/path tests remain.
No live upload or server acceptance was tested; other native modes remain open.
Reproduce apk-smali M0g.A00 and MDL.A0B with signatures/exceptions.

- [ordinary-path-verifier.txt](ordinary-path-verifier.txt) — SHA-256 `1ea7c004396e5e05e590be545de0591f1a6af2855c43dce7a97debc3100170a9`.
- [ordinary-path-setter.txt](ordinary-path-setter.txt) — SHA-256 `7ad02ee18ee37a7f3c0939d066a5863ccdc67664364e5c6fd408cf63e8b83f5d`.
