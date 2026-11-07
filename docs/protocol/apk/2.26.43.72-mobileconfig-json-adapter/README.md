# MobileConfig JSON adapter and metadata-consumer status

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Only static reads occur; no native parser or account operation runs.

## A selected loader branch parses JSON separately from records

The [record-source loader](../2.26.43.72-mobileconfig-record-source/README.md)
has multiple content paths. On the selected branch at 0xbb4e8–0xbb530 it reads a
stream into a temporary string via 0xc5588, invokes parser adapter 0xc950c, then
calls 0xc60f8 with an existing loader child and the parsed-root pointer. That
consumer scans an **existing** record vector: it is not established as the
original schema-record population routine. A separate helper 0xc541c also reads
file content and uses the same adapter, but sends its parsed root to 0xc52a4.
Do not conflate these consumers or declare one universal parameter-map decoder.

The broader loader also enters 0xbf604, which changes loader/child mode bytes and
calls additional header/content helpers. Its retained entry is an unfinished
boundary; no binding from the earlier packaged binary token-map header to this
JSON branch has been established. Filename presence alone does not resolve it.

## The YAJL adapter has a distinct parse-result boundary

0xc950c allocates a YAJL parser with callback table 0xd4260. It configures numeric
option 1 to value 1; the option's symbolic meaning and callback type mapping are
untraced here. It obtains input length through strlen, calls yajl_parse, and then
branches on yajl_complete_parse's returned word. This is a C-string length path,
not an arbitrary byte-span parser: embedded NUL bytes bound the supplied length.
No actual content or runtime validation is claimed.

On a zero completion status, it frees the parser and returns the retained root.
On a nonzero status, if a diagnostic buffer and size were supplied, it obtains a
YAJL error message and formats it into that buffer. It releases pending/partial
nodes and the parser, disposes of a retained root if present, and returns null.
The selected caller supplies a 50-byte diagnostic buffer. The complete callback
allocation/type rules remain unresolved; zero parser status alone does not prove
an application-valid root or a committed configuration.

## Consumer checks and a deliberately separate return value

0xc60f8 returns false for a null root or a root whose numeric tag is not 4. For a
tag-4 root it walks indexed children. The exact tag-to-JSON-kind callback mapping
has not been traced, so this report calls it an indexed collection rather than
asserting all malformed JSON shapes are safely rejected.

For each selected child it reads a C string and splits on colon into 16-byte
spans. Its initial size checks require at least two spans and an even count.
It converts the first span via 0x9e1ec (conversion failure semantics untraced),
then scans the existing 0x50-byte records by word +0x4c. The scan cursor starts at
zero once per function and advances forward; it does not restart for every input
item. Thus order matters for this selected matching path, independently of the
schema-index selection order traced earlier. Remaining per-item fields, mutation
and failure effects are not completely modeled.

**At the selected call site, its boolean result is not tested.** The call at
0xbb528 is immediately followed by `mov x0,x21` at 0xbb52c and parsed-root cleanup
at 0xbb530. The inspector checks that exact overwrite instruction. Later loader
processing follows; this observation must not be described as a checked
successful metadata application. It also does not establish that every caller
ignores this result, that failure always mutates state, or that another enclosing
validation cannot reject the loader later.

Within the consumer, malformed span counts and failed forward record matches
can end processing with false; a null/non-tag4 root also returns false. Empty
iteration takes its separate completion return. Complete per-item validation,
rollback/partial-application behavior and external loader status remain unknown.
The second consumer 0xc52a4 has different root/field handling and no identical
safety claim is made for it. Its bounded/full selected region is retained to avoid
mistaking shared parser usage for shared validation.

## Reproduction and evidence limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-json-adapter/inspect-json-adapter.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-json-adapter.json
cmp /tmp/mc-json-adapter.json docs/protocol/apk/2.26.43.72-mobileconfig-json-adapter/adapter.json
```

The inspector reuses pinned ELF validation, binds selected BL instructions and
checks the caller's status overwrite. LLVM windows retain the adapter, metadata
consumer, call site, stream helper and alternate consumers, plus a bounded entry
into the untraced binary-mode helper. Instruction-region retention is not a claim
of complete semantic modeling or exception coverage. Nearest exported JNI_OnLoad
headings do not identify local functions.

rea-json-window.json retains complete REA Evidence
`ev_bbcb764101027cd5508a9e0cc101eb728e0ee2942c2acf2725dcfcf525e44cd2`:
`e scr.color=0; pd 76 @ 0xc950c`, exit 0, untruncated output, unchanged member hash,
with an entrypoint warning. The window is bounded and LLVM retains the adapter
through its next function boundary. Register-relative annotations are not
observed runtime memory.

**Remaining:** callback/root type mapping, parser options and conversion rules,
full metadata mutation/validation, original schema-record population, selected
file identity and decoder routing, enclosing loader status, response persistence
and cache activation. No Rust runtime, sender or saved account state changed.

Follow-up: [native parameter reader](../2.26.43.72-mobileconfig-native-param-reader/README.md)
now traces the selected binary header/body helpers, with native validation
distinguished from the Java header. Their 8-byte body vector is not the
0x50-byte schema-record vector; original schema population remains unresolved.
