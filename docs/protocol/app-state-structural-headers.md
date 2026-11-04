# App-state structural header inspection

Confirmed from APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`:
F2T.A00 parses snapshot reference before selected patches, parses the entire
patch list before semantic checks, then checks exit, required version, mixed
external/inline and duplicate collection version in that order. Native fatal
reason metadata: snapshot protobuf 69; patch protobuf 18; exit100 36; exit101
37; other or missing nested exit code 38; missing version 8; mixed source14;
duplicate version9. See app-state-fetch-completion.md and its checked trace.

Confirmed nested descriptor shapes (all retained under
`rea/2.26.43.72-patch-header-audit/`): Dh0 tag1 DYK version; repeated tag2 Dbb
mutation; tag3 Dg5 reference; tag4/5 bytes; tag6 DYH; tag7 Dba exit; tag8
uint32; tag9 **bytes**, not a nested debug message. DYK tag1 uint64. Dba tag1
uint64, tag2 string. Dg5 tags1/5/6 bytes, tags2/3 string, tag4 uint64. Dbb tag1
enum, tag2 DdZ record. DdZ tags1/2/3 DYI/DYJ/DYH respectively. Each of these
three wrappers has tag1 bytes. Descriptor scalar kind IDs are checked against
the shipped protobuf FieldType initializer: uint64=3, string=8, message=9,
bytes=10, uint32=11, enum=12; message-list=27. The current descriptor strings
use presence flags independently of these low-byte kinds.

Reproduction from repository root:

```sh
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dg5;' --method dynamicMethod --signatures --exceptions
```

Repeat for DYK, Dba, Dbb, DdZ, DYH, DYI and DYJ. The retained FieldType and
WireFormat$FieldType initializers use the same command with their complete
`Lcom/google/protobuf/...;` class descriptor and `--method '<clinit>'`.
These are checked disassembly text, not manufactured REA envelopes.

Implemented `transport::app_state_sync::headers::inspect_collection_headers`
is pure and borrows every original body. It recursively checks recognized
nested protobuf syntax, preserves unknown fields in the originals and performs
whole-collection parse-before-semantics. It never downloads, looks up keys,
verifies MACs, writes state, emits requests or authorizes promotion. An external
reference without inline mutations is explicitly deferred; it is never silently
converted into an empty inline patch. Debug output excludes payloads, versions,
exit text, reference paths, keys and unknown body content. Existing cryptographic
CollectionPatch decoding remains unchanged and strictly rejects unsupported
exit/external paths.

Local restrictions are explicitly separate: aggregate16MiB,4096 patches,
262144 wire fields; duplicate recognized singleton fields; known-field wire
mismatch; group wire encoding; over-wide varints; invalid UTF-8 strings.
These are **local policy**, not alleged native fatal reasons. Unknowns: Java
protobuf's complete handling of noncanonical values, duplicate merged messages,
unknown groups and UTF-8 replacement has not been established. Therefore this
is a bounded structural subset, not full Java protobuf parser equivalence.
Definite truncation/invalid field-number syntax is reported with the applicable
native parse-fatal metadata; unsupported encoding is never relabelled as that
native error. State application, snapshot download/authentication, version
continuity and requested collection admission remain separate obligations.

Focused tests cover recursive malformed records/references, snapshot-before-
patch and all-patches-before-exit failure priority, native fatal reasons,
external deferral, local-vs-native separation, duplicate versions, borrowed
unknown-field originals and redacted diagnostics.

Retained new descriptor SHA-256 checksums:

```text
5257df0d3d84df5445ef7a3d710056683a2ded5a55e52deb7745410df39c635d  reference-descriptor-checked.txt
c60cedca7d844b2661b8936bac36b1f223e728923032a3ad79626db6cef913e8  version-descriptor-checked.txt
5685a3846102319c0a9f939a4a8b0167867530f11a3b81156e8da1a0af48c600  exit-descriptor-checked.txt
44287decc512da116f60dafb5dcbb0d9305b5350699bb509b089fd615ec70780  record-descriptor-checked.txt
2edc1e794e674098eb32ae501a2a37574c1aa59fbb2233410f9f0e40fd3cc5e0  mutation-descriptor-checked.txt
b4469186cf828b12485bc10a59a6fc0c09cc5b3dacc21657dba732c3703e8602  key-id-descriptor-checked.txt
5b1904aeff005a9d4dcd6e2011ce40dd0d7a0146f32776852c74b8c65b1c8f99  index-descriptor-checked.txt
b8bc9c6d06bfa4a436d573d07adac4f682d7fd3ac97959cc62fa375bd7bedfe1  value-descriptor-checked.txt
f751703fe01a2bc7b8eaaf3a04c0267365dace4fdb0488bcd2449985a47e7b08  protobuf-fieldtype-checked.txt
9427f435b1d33edb7578a09afc1966d09100c6355cab7912c281237c15671309  protobuf-wiretype-checked.txt
```
