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

## Raw-stream versus multipart body writer (2026-11-04)

**Confirmed static**, same pinned APK. K2k.A08 registers JiH(type2) with
input stream/name/filename/offset/length. JiH constructor maps the first long
to A02 (offset) and second to A01 (length). The retained MtZ producer passes
LzO.A00/A01 in that order; LzO constructor preserves those long arguments.

K2k.A06 takes the **raw single-stream** branch only when supplied M3d is
nonnull and registered streams are nonempty. It checks the count with6oi.A1U
and an assertion describing single-file support, then copies first stream
via A01 and returns before multipart field emission. The predicate helper is closed in the follow-up below. Null route or empty stream list enters
the separate multipart writer: boundary/disposition, type-dependent part
headers, registered fields and late fields. Their literal header strings
are not proof they are emitted on the raw branch.

A01 uses JiH.A02 as positive skip amount, repeatedly calling InputStream.skip
until the remainder is nonpositive. It reads16KiB chunks, writes each, reports
progress starting at the offset and checks thread interruption and an atomic
abort flag between chunks. This is not an endorsement of copying its skip
loop without considering streams whose skip returns0. On EOF it consults
optional Kif processing and may emit additional bytes described by JdX.
The Kif implementations/producers and trailer semantics are still untraced.
No claim is made that every stream is simply copied byte-for-byte to EOF.

JAS.A04 selects this writer after connecting and obtaining its output stream.
Thus the APK demonstrably supports a route-bound raw upload; adding multipart
to every Rust request would be unjustified. Rust's one-shot encrypted-body
POST uses no resume offset. Full source binding for stream creation, optional
processing and server resume negotiation remains open; raw shape alone is
not a proof of complete upload parity. No runtime behavior was changed.

Negative lookup: JAS.A01 and JiH.A00 have no matching method bodies under
exact class/name filters; body copying resides in K2k.A01/A06 instead.
Reproduce apk-smali K2k.A08/A06/A01, JiH.<init>, LzO.<init> with
--signatures --exceptions. All selected outputs are retained below.

- [body-writer.txt](body-writer.txt) — SHA-256 `7f9276261e2c16bc0ba9b140e4d3557b68256f27bb5655658972d5770314e4dc`.
- [stream-copy.txt](stream-copy.txt) — SHA-256 `bc4de42fb3f1114746756b1d648f944e810c503eb1b9802823aafdc4f7b9b558`.
- [stream-input-descriptor.txt](stream-input-descriptor.txt) — SHA-256 `9d37f7e5bfa4fe04a54528641b24bf49033b992c166ae84b2ba23553762096ee`.
- [register-stream.txt](register-stream.txt) — SHA-256 `84ccc4e7d5c17ded08392220b602021663fc37c9083080e60ddb36647160e760`.
- [stream-fields.txt](stream-fields.txt) — SHA-256 `9c3ead53e2619ae27e7327bbdcf34516731793b3e5db99e68beb722f61b008c2`.

## Optional EOF processor and single-file predicate (2026-11-04)

**Confirmed static**, same pinned APK. Exact6oi.A1U returns input==1,
closing the raw writer's single-file assertion.

The retained MtZ.Cb6 obtains optional Kif from MOu.A0O.AL2 but explicitly
replaces it with null unless MvM.A06 is true. A06 returns true for LKY;
for LKZ it calls3q3.A1W on media kind versus0jS.A0e; otherwise false.
The helper comparison and0jS kind identity remain untraced, so do not name
every eligible media category from this predicate alone.

Whole-APK method-name scan B5p finds Kif's declaration and one concrete
MtQ method with selector branches. AL2 scan finds null-returning Mv8,
Mv9 constructing selector0 MtQ with NYu and MvM, Mve constructing selector1
with Function0 and MSm, plus Ncy declaration. This is a bounded DEX-name
scan, not proof of all runtime implementations or factory selection.

MtQ selector0 B5p calls A01 and returns JdX(bytes,true); selector1 checks
MSm.A05, producing a canceled seal with false completion or a sent seal
using the supplied hash callback with true completion. JdX constructor
stores bytes and completion directly. K2k.A01 writes returned bytes before
testing completion; false completion throws JOK. Null Kif instead creates
JdX(null,true), so this copy stage adds no EOF bytes in that branch.
Abort/error hooks B5o/B5q are separately retained; their bytes must not be
substituted for a normal-success seal.

MtQ.A00 builds JSON status, requires hash for its sent variant, encodes JSON
and concatenates RIFF + JSON bytes + four-byte(length XOR0x5614050b) + META,
rejecting total size>100. Byte order/charset helper identities are not closed
here. Selector0 A01 has its own voice-note metadata path, with status from
LKZ.A02 and hash from LKZ.A09, similarly framed. Its exception handler can
return null. These are specialised transport metadata, not a universal
requirement on every encrypted file.

Factory ownership, concrete media-kind helper, stream/hash producers, complete
seal bytes and configured operation binding remain open. Current one-shot
Rust body stays unchanged; no fabricated seal or automatic replay is added.
Reproduce whole-name apk-smali --class '' --method B5p/AL2, whole MtQ,
and exact MvM.A06/JdX.<init>/6oi.A1U with signatures/exceptions.

- [eof-implementations.txt](eof-implementations.txt) — SHA-256 `01725293e10e752fabe1b03e6e23cec43d6ce9d04be2280362f84ad713275d74`.
- [eof-processors.txt](eof-processors.txt) — SHA-256 `b2271730d3e35edd1f15b3226cf932682ea5648ef91c1b917cbb825848af24c4`.
- [eof-factories.txt](eof-factories.txt) — SHA-256 `2221b14ec05ad0d9fe316785d0b401fd9dbc6f21237aafc923e1010fc06d90ca`.
- [processor-eligibility.txt](processor-eligibility.txt) — SHA-256 `8d7db54afc43ffccfab715b84ee59776541a763fb10dafface33fbaeb727e5fa`.
- [processor-result.txt](processor-result.txt) — SHA-256 `2e2b7ae3d4d1cc7b9b45f9fa89a32b3da9990e2560452ed762b2310563fbdd55`.
- [single-stream-predicate.txt](single-stream-predicate.txt) — SHA-256 `f444cdca1f50630610f1d5c38d8c31c816990bf906e756eb325188b0ccbcd5a7`.

## Seal owner, excluded kind and encoding helpers (2026-11-04)

**Confirmed static**, same pinned APK. MOu.<init> prefers Nbs when it
also implements Ncy; otherwise LKZ gets Mv9 and other MvM subclasses get
Mv8 (null processor). Combined with the previously retained A06 gate, LKZ
processing is enabled except for identity-equal0jS.A0e.3q3.A1W is reference
inequality, and0jS.<clinit> constructs A0e with name newsletter-ptt.
Thus this exclusion is specifically newsletter-ptt, not every voice note.
The custom Nbs override remains significant; these defaults do not prove
which processor every higher-level upload chooses.

Klt.A0q allocates a ByteBuffer and sets LITTLE_ENDIAN.08Q.A05 is UTF-8.
This closes those previous encoding unknowns. Selector0 seal A01 uses Java
String.length for its XOR length field; selector1 helper A00 uses UTF-8 byte
length. Do not silently equate them for arbitrary non-ASCII JSON.

LKZ.A09 requires A03=true (recording finished), can reuse an NYu-keyed
cached hash, otherwise reads the stream returned by MWH.A03 to EOF and
gets Mzq.A00().MAw.A00. Its log identifies ciphertext SHA-256 computation,
but the MAw string encoding and stream/hash construction remain untraced.
The method closes the stream and caches the result. A logged stream-read
exception is not automatically a validated hash: remaining extraction still
follows the actual method and needs tracing.

**Confirmed source comparison / unresolved mismatch scope:** Rust's one-shot
voice-note upload currently POSTs encrypted.body with no separate metadata
seal. Native LKZ's default non-newsletter processing can add a seal. Whether
our finished-file upload corresponds to that path requires LKZ construction,
Nbs override and hash producer tracing. Do not claim universal parity or
append a guessed hash/seal. This is now a concrete potential omission in
the audit queue, not dismissed as universally optional and not yet a
fully bound construction fix. No runtime behavior or private state changed.

Reproduce exact apk-smali MOu.<init>,0jS.<clinit>,LKZ.A09,08Q.<clinit>,
3q3.A1W and Klt.A0q with signatures/exceptions.

- [seal-owner.txt](seal-owner.txt) — SHA-256 `45e99ee8e0ba0140a08637b2590cf63bf1ece5be275095f6c06b329705bb2444`.
- [media-kind-values.txt](media-kind-values.txt) — SHA-256 `2b7bb90d1caee46c3bfab201a98c101e6eeeaa2e994b31aeede4006cd036d784`.
- [voice-seal-hash.txt](voice-seal-hash.txt) — SHA-256 `f5f906c1875be4dd3cd3219df46a0f6481556b27075607af27456b90bda8e699`.
- [seal-charsets.txt](seal-charsets.txt) — SHA-256 `69ac55ea0e4e83cb125dffb15ef6c4483509af82776f34bd4de1783bc6e00c47`.
- [seal-kind-inequality.txt](seal-kind-inequality.txt) — SHA-256 `212612ba33a63619d71bdf7f0e9d206d155a6d31dca36cec5c76d178af0aa3f9`.
- [seal-byte-order.txt](seal-byte-order.txt) — SHA-256 `539a03bb41dfae5d85b6013c73a8b37ef2c6a2a3ec916ab3826e07fc0497a61f`.
