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
