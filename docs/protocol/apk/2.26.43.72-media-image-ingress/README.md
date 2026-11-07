# Current image protobuf ingress and hash projection — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports and exact descriptor decoding via xtask fallback;
no target execution, new REA Evidence IDs, account traffic or Rust runtime change.
Byte PCs below are method-relative; branch displacements use16-bit code units.
This connects the [message-model projection](../2.26.43.72-media-download-model/README.md)
to selected current-build protobuf fields, not the entire receive router.

## Current image schema and comparison with Rust

[DiL.dynamicMethod](image-descriptor.txt) and [decoded fields](image-fields.txt)
identify the current image schema from the descriptor/object-array pairing,
including [common helper](schema-common-helper.txt) and [image helper](schema-image-helper.txt).
The mapping is not inferred from old obfuscated class names or object-array order.

| Image field | Current wire tag | Rust ordinary Image |
| --- | --- | --- |
| url | 1 | url=1 |
| mimetype | 2 | mime=2 |
| fileSha256 | 4 | hash=4 |
| fileLength | 5 | length=5 |
| mediaKey | 8 | key=8 |
| fileEncSha256 | 9 | enc_hash=9 |
| viewOnce | 25 | once=25 |

Core fields match Rust's existing descriptor. This current-build confirmation
supersedes reliance on2.26.37.71 BZD naming for THESE fields only. The schema also
has scansSidecar=21 and scanLengths=22; firstScanSidecar=18/firstScanLength=19 are
separate fields, not aliases. Schema existence does not establish every producer
or operation requires them.

## Actual protobuf-to-message image projector

[Exact encrypted-hash field census](image-encrypted-hash-refs.txt) finds NINE DEX
read/write sites for DiL.fileEncSha256_. EwA.A03 is a direct reader and its
[complete export](image-projector.txt) accepts1g2 and DiL. This is stronger evidence
than following an arbitrary hash setter caller. Full receive wrapper/router
selection and callers' boolean/update-mode meanings remain untraced.

At036c..0394 the selected path reads fileSha256 bytes, requires length32 and calls
DBc.A1K; at0394..03b8 it reads fileEncSha256, likewise requires32 and calls DBd.A1R.
Presence checks depend on the method's existing update/context boolean atv20;
do not treat every absent-field case as the same fresh-message behavior. Wrong
length reaches a Dxp.A04(14) throw at0616/0626. That14 is a LOCAL parser error
argument, not an HTTP status or established ban/retry protocol code.

[DBc.A1K](plaintext-byte-setter.txt) Base64-encodes with flag2 then calls1eQ.Ch4;
[DBd.A1R](encrypted-byte-setter.txt) does the same then calls Ch2. The earlier
[retained setters](../2.26.43.72-media-download-model/README.md) mirror these values
into8tC.A0Z/A0Y and the model builder carries them into plaintext/transfer hash
fields. This closes the selected image hash projection chain. Base64 here is
local message representation, not evidence to Base64-encode Rust's protobuf hash
bytes or to send a new HTTP header. Hash algorithms/crypto were not re-derived
from the field names in this pass.

## Sidecar validation is separate from accepting the image hashes

At0160..020c EwA reads scanLengths and a scansSidecar presence bit. Its selected
validation requires count between2 and20 inclusive, integer division
`sidecar_bytes.length / 10 == count`, every scan length positive and the sum no
larger than Integer.MAX_VALUE. The division is FLOOR division: this leaf does NOT
check a zero remainder, so do not silently rewrite the condition to exact10*n.
On that validated path it invokes1fW.Cdn(bytes,indexes), later retains first10
sidecar bytes and first scan length in8tC. Invalid sidecar conditions branch past
this block to0282 and continue with thumbnail/hash processing; this leaf is not
a universal whole-image refusal on absent/invalid scan metadata.

The scans fields are therefore connected to message-side streaming state, but
concrete1fW/1fV provider storage and complete hydration/factory/start/end behavior
are not established. Rust currently ignores these fields on its explicit
whole-ciphertext path; no streaming feature, fabricated metadata, range or network
preparation is added from this evidence.

## Bounded candidates and negative findings retained

[Hash literal census](hash-schema-xrefs.txt) found17 fileEncSha256_ and15
fileSha256_ string sites, not17/15 request families. [Setter censuses](transfer-setter-callers.txt)
and [plaintext setters](plaintext-setter-callers.txt) find16/17 direct invocation
sites respectively. Shared setters are not sufficient to classify ingress.

- [6D9.AKF](excluded-generated-results.txt) copies selected5Uw generated-media
  result hashes into newly constructed messages/8tC; it is NOT established as
  the ordinary inbound protobuf decoder. Following its direct field writes was
  a bounded false lead, not proof of inbound hash mapping.
- [0xH.A02](excluded-cursor-projector.txt) is a Cursor-to-message projector.
- [DEf.A00](excluded-DEf-propagation.txt) propagates message media metadata;
  [Ed1.A00](excluded-Ed1-list.txt) operates on a list and calls setters;
  [EoO.A01](excluded-EoO-projector.txt) accepts AHF. None was established as the
  selected ordinary image protobuf reader.
- [F1B.A05](video-projector.txt) accepts DiM, NOT DiL. The [DiM schema](video-fields.txt)
  has video-shaped fields, including gifPlayback, streamingSidecar and processedVideos.
  It confirms a separate video mapping candidate, not an image alias.
- [Dh-prefix descriptor scan](candidate-Dh-fields.txt) is retained as bounded
  schema discovery. Similar fields occur in alternate/submessages. No broad
  inference of complete media/protobuf coverage follows from the shared names.
- Di4 exact schema decode refused unsupported oneof pairing; no field mapping was
  guessed or substituted using literal order. It is not needed for the DiL finding.

`retained-proto-fields` was also attempted on the raw apk-smali image export and
refused with `no protobuf descriptor decoded`. Source inspection shows that tool
expects the structural research-bundle header/op= format, not this raw export.
The successful `apk-proto-fields` output and pinned APK remain the schema oracle
here; no offline retained-schema regeneration success is claimed.

## Reproduction and limits

Run `apk-proto-fields <apk> --class 'LX/DiL;'` (and DiM); `apk-field-refs <apk>
--class 'LX/DiL;' --field fileEncSha256_ --type 'Lcom/google/protobuf/ByteString;'`;
`apk-callers <apk> --class 'LX/1eQ;' --method Ch2` (and Ch4); and
`apk-xref <apk> --anchor fileEncSha256_ --anchor fileSha256_`.
Export `apk-smali <apk> --class '<descriptor>' --method <name> --signatures
--exceptions` for DiL.dynamicMethod, EwA.A03, DBc.A1K, DBd.A1R,8t2.A1Y,DBg.A1J,
6D9.AKF,0xH.A02,DEf.A00,Ed1.A00,EoO.A01,F1B.A05. Dh-prefix schema output used
`apk-proto-fields --class 'LX/Dh'`. [checksums.json](checksums.json) pins all files.

**Still unknown:** complete envelope/receive dispatch, current audio/document/
sticker projection, update-mode inputs, concrete sidecar provider/loader, exact
native full-file request/response range semantics, account/output authority and
live interoperability. No mandatory-header, automatic retry,206/416 relaxation,
restriction-cause or zero-ban claim is introduced. Rust change is provenance
comments only; its established descriptor behavior remains intact.
