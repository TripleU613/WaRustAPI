# Pinned upload HTTP construction boundary

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX text exports, not live execution or new REA Evidence envelopes.

## Construction chain

Previously retained MtZ.Cb6 invokes MvM.A02 for an input map, builds LzO,
then calls15Q.A01. MvM.A02 obtains a per-media-key map from845.A00 and
merges M5T.A0I (empty maps for absent inputs) via05p.A08. Merge collision
precedence and map producers are not traced here. A second845.A01 map
flows separately into LzO and K2k.A0A; do not conflate the two maps.

15Q.A01 selects JAT or JAS under runtime/configuration conditions involving
K1R and configured string filters/flags. No single active branch is proven
for the Rust comparison profile. K2k.A03 appends route auth as the pair
("auth", M3d.A01) to A0J before virtual A04 execution. It closes registered
input streams on success and exceptional return. K2k.A04 has no bytecode
body (tool labels abstract/native); concrete implementations below supply it.

## JAS HttpURLConnection branch

Confirmed from JAS.A04:

- connect timeout15000ms, read timeout60000ms, User-Agent fromK2k.A0I,
  Accept-Encoding=identity. Rust uses its configured UA/timeouts and identity;
  its total timeout is not identical to a native read timeout.
- X-FB-Request-Analytics-Tags is set only when K1R metadata exists, using
  K1R.A04(). upload_media_type/origin/paired_media_type in this block are
  marker annotations, not separately set HTTP headers.
- Companion_User_Secret is set only when JAS.A06 is nonnull. No evidence
  here establishes that a primary client must send a companion secret.
- Additional JAS.A08 map entries are set as request properties. A route flag
  controls Host override fromM3d.A07; socket/address selection is separate.
- If registered first stream has positive offset, it emits
  Content-Range="bytes <offset>-*/*". With offset0, a positive stream
  length and JAS.A0A can select fixed-length streaming. A different branch
  sets multipart/form-data with a generated boundary. Thus multipart and
  Content-Range are not universally required for every upload.
- Status is obtained before the response_headers_received marker and response
  header/body handling, consistent with Rust's recently repaired durable
  header-before-body observation ordering, not proof of native persistence.

## JAT boundary / remaining unknowns

JAT.A04 full output is retained: it constructs POST through the alternate
stack, sets identity and conditionally constructs range/body/length/type
fields. Its complete branch parity, compression, extra header source/override
order and response semantics remain untraced. The full method is evidence
for follow-up, not a completed header inventory.

Rust image_upload currently reserves once, POSTs encrypted bytes with identity,
uses configured UA and retains status before bounded body streaming. No native
mandatory-header mismatch is established by these leaves alone. Route/stream
input producers must be bound to the supported operation before changing body
framing or headers. No secret, metadata or guessed range is fabricated.
No live request or Rust runtime change was made.

Reproduce: apk-smali with --signatures --exceptions for15Q.A01, K2k.A03/A04,
JAS.A04, JAT.A04 and MvM.A02. The map/stream entry is in the previously
[retained upload producer](../2.26.43.72-media-transfer-results/upload-result-producer.txt).

- [transport-selector.txt](transport-selector.txt) — SHA-256 `20cf623506c3fae2deed0b1947a4615576cdfdb05348879da376a51b940bafeb`.
- [route-auth-entry.txt](route-auth-entry.txt) — SHA-256 `aff284445d9e432192808d2162b3e9f680dd400a23c0b62896cd28efa7f8ca0d`.
- [base-execute.txt](base-execute.txt) — SHA-256 `eb5cbd97de43971b8dd6e003c12f57437c1721d38c01b095ed65752f7974f92f`.
- [urlconnection-execute.txt](urlconnection-execute.txt) — SHA-256 `a2a64d41bc36f847219185cdc15c67433c24332c83558b321d8a094b683ffe37`.
- [tigon-execute.txt](tigon-execute.txt) — SHA-256 `207e8f9ee14715ec1b1abb142a76f16551d5919de1231bb7f4eab111f2964f86`.
- [upload-header-inputs.txt](upload-header-inputs.txt) — SHA-256 `01c56c94172e7d86ae8c7ae3bce07053ef61b8369e8834865c7c0b5c12594849`.
