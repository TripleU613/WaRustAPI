# Message-to-download-model projection — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback; no target execution, new REA
Evidence IDs, live traffic or Rust runtime change. PCs below are method byte
offsets; branch displacements use16-bit code units. This follows the
[factory selection boundary](../2.26.43.72-media-download-factories/README.md).

## Model constructor and selected message builder

[Constructor census](constructor-callers.txt) finds FOUR direct MX3 constructor
sites, all in its own A01/A02/A03/A04 builders. The complete [constructor](constructor.txt)
and [A01 message builder](builder-A01.txt) establish the following register mapping.
The constructor receiver is v5; A01 invokes the same signature with receiver v22,
so corresponding input WORD registers differ by17. Longs consume TWO words;
these are register mappings, not guesses from argument ordinal counts.

| Field consumed by download | Constructor source register | A01 call register | Selected nonnull8tC source |
| --- | --- | --- | --- |
| A0K transfer hash | v18 | v35 | 8tC.A0Y ->v20 at01ac..01b4 ->v35 at0456 |
| A0L plaintext hash | v15 | v32 | 8tC.A0Z ->v19 at01b4..01bc ->v32 at044a |
| A09 object length | v40/v41 | v57/v58 | 8tC.A0I ->v11/v12 at0326 ->v57/v58 at0482 |
| A0t sidecar Callable | v27 | v44 | N3m(message,8tC.A13) at0338..0346 ->v44 at0472 |
| A03 transfer category | v36 | v53 | predicate result category3/2/1; see below |

A01 also has an interface-source alternative when the supplied8tC parameter is
null: Asm/Asq/AtH populate the corresponding string temporaries at0552..056e,
At7 supplies the length and AtA supplies the sidecar provider. Concrete semantics
of every interface implementation remain untraced. Do not claim the selected
nonnull8tC branch is the sole message representation.

The three other complete builders [A02](builder-A02.txt), [A03](builder-A03.txt),
[A04](builder-A04.txt) are retained to preserve the constructor census boundary;
their entire operation-specific input provenance is not classified here.

## Independent stored-field identity

Exact write-only DEX censuses find NINE sites each for [8tC.A0Y](transfer-hash-writers.txt)
and [8tC.A0Z](plaintext-hash-writers.txt); this is not a complete reflection/native
or runtime writer census. A selected writer, [27G.A01](message-hash-projection.txt),
is a database Cursor projector, NOT a protobuf decoder. At01c4..01d4 it reads
`enc_file_hash` into A0Y; at01d4..01e4 it reads `file_hash` into A0Z. It also reads
`file_length` into A0I immediately before these. This independently ties the
selected builder strings to stored encrypted/plaintext hash names.

[1eQ.Ch2](message-set-transfer-hash.txt) writes its input into1eQ.A06, calls A0w,
then mirrors it into8tC.A0Y if an8tC exists. [Ch4](message-set-plaintext-hash.txt)
similarly writes1eQ.A07 and conditionally8tC.A0Z. The selected setters do not
validate hash width, calculate a digest or prove a particular protobuf wire tag.
Other census writers, especially6D9.AKF, remain follow-up candidates. No wire
field number or digest algorithm is inferred merely from stored column names.

## Transfer category is not simply MIME or original message media type

A01 first invokes [ACf.A03](category3-predicate.txt). True assigns A03=3 at001e;
false reaches [ADF.A04](category2-predicate.txt), assigning2 when true and1 when
false at0596..05ae. ACf requires1g3, a nonnull AtA provider and provider.BNb()==true.
ADF has message subtype/provider/config/chat predicates; it is NOT a simple
`video/mp4` comparison. Its helper A01/config and every subtype/provider remain
untraced here.

The earlier factory mode selector labels categories3/2 photo/video for its
configuration categories. That does not mean every original image has A03=3 or
every original video A03=2. In particular, A03=1 can result from those predicates
failing even when the outer operation handles media. Thus MX3.A07's2/3 eligibility
cannot be equated to a raw protobuf media-kind test in Rust. No blanket native
sidecar requirement is inferred for every ordinary encrypted image.

## Sidecar acquisition includes preparation, not just field presence

[N3m](sidecar-callable.txt) retains message and1fV provider. Its call returns null
when provider is null. Otherwise it calls [AEH.A06](sidecar-preparation.txt), then
reads provider.B7k() bytes, AbZ() int array and At5() optional Long, and constructs
[LvR](sidecar-model.txt). LvR stores those directly as A01/A02/A00 without checking
lengths or contents in this constructor. Later factory/consumer checks remain
separate.

AEH.A06 dispatches1eQ through1fU.BbA using required message.A0F,9Kg through
ATe.BbB using message.A00, permits9L3 without those calls, and throws for other
media types. The invoked providers' I/O/side effects and full data loading are
not established by this selected method. It would be inaccurate to call it a
proven network fetch or to treat a raw protobuf sidecar byte string as the whole
native sidecar acquisition process. MX3.A05's already traced exception-to-null
boundary surrounds the Callable; N3m does not silently catch its own failures.

## Conclusion and remaining gap

**Confirmed:** selected stored enc_file_hash/file_hash/length reach the same
MX3 fields used by the normal validator; streaming category derives from native
provider/message predicates; sidecar is lazily prepared/projected separately.
This refutes a simplistic MIME-to-stream-path or absent-sidecar-to-normal-fallback
mapping. Existing Rust whole-file hashes and explicit no-replay handling are not
changed on a speculative interpretation of native streaming inputs.

**Unknown:** complete inbound protobuf-to-message mapping and current wire tags,
concrete provider.BNb/sidecar producer/loader behavior, the exact supported Rust
profile's native category and starting file/range state, native response/range
recovery and final output/account authority. No206/416 status relaxation, hash
fallback, automatic preparation/network fetch, header or retry is introduced.

Reproduce with `apk-callers <apk> --class 'LX/MX3;' --method '<init>'`;
`apk-smali <apk> --class '<descriptor>' --method <name> --signatures --exceptions`
for MX3.<init>/A01/A02/A03/A04, ACf.A03, ADF.A04, AEH.A06, LvR.<init>,27G.A01,
1eQ.Ch2/Ch4; export N3m without method filter. Exact write censuses use
`apk-field-refs <apk> --class 'LX/8tC;' --field A0Y --type 'Ljava/lang/String;'
--writes-only` (and A0Z). [checksums.json](checksums.json) pins all retained files.
