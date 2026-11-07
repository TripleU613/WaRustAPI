# Token lightweight-manager parameter header and schema gate

**Confirmed static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static xtask exports and ZIP-only selected extraction, not REA Evidence envelopes
or live observations. Follows [handle/cache lifecycle](../2.26.43.72-token-config-refresh/README.md).

## Manager installation depends on schema compatibility

The previously retained0A8.BPZ constructs0EV, attempts its latest buffer and reads
configuration-table metadata. In its parameter-map path it reads a map header,
compares schema hashes usingA05, invokesPcn, rechecks the parsed schema, then
installsPCc into0EV.A00 only after obtaining the parsed translation array.
Selected successful flow produces result12 and later refreshes session state;
the already retained result-dependent ready/cache logic remains additional.
This closes the selected schema gate, not every manager/error/result branch.

A05 requires a nonnull/nonempty first string and nonempty second string, strips
each at the first colon when present, then requires both prefixes nonempty and
case-sensitive equal. It does not trim, compare suffixes, interpret numbers or
merge arbitrary schema versions. The second argument is dereferenced before its
later null check; callers must supply it nonnull. Raw AB properties and packaged
map existence alone do not establish this compatibility with the loaded table.

## The binary v4 header is now decoded

PfF.A01 reads the first2 bytes, then selects v2 or v4. The v4 path requires one
28-byte read, decodesPfF.A00, then reads exactly its returned hash length. The
helper selects LITTLE_ENDIAN and consumes this layout (offsets include v4):

| Offset | Type | Meaning established here |
| ---: | --- | --- |
| 0 | 2 bytes | ASCII v4 selector |
| 2 | signed i16 | total header length |
| 4 | signed i16 | magic32767 |
| 6 | unsigned u16 | PHU.A00 |
| 8 | unsigned u16 | PHU.A02 |
| 10 through25 | bytes | consumed/skipped fields; semantic names unproven |
| 26 | unsigned u16 | PHU.A01 |
| 28 | signed i16 | nonnegative schema-hash byte length |
| 30 | that many bytes | decoded schema-hash string |

The helper requires total length==30+hash length; invalid magic/length returns-1.
PHU initializes all numeric fields to0 and hash to empty; header failure is not
proof of a valid empty schema. Short reads, v2 parsing and callback semantics
remain distinct from this pinned selected extraction.

All four packaged headers have total length62 and32-byte schema hashes. Raw
PHU fields differ by universe:

| Universe | PHU.A00 | PHU.A02 | PHU.A01 | Body bytes after header |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 635 | 645 | 645 | 5160 |
| 2 | 22398 | 22458 | 22458 | 179664 |
| 4 | 2 | 72 | 72 | 576 |
| 6 | 38 | 38 | 38 | 304 |

asset-headers.json retains full member identities and schema hashes. These raw
field labels are intentionally not renamed counts/defaults without tracing their
consumers. The body remains undecoded; neither headers nor schema hashes are token
bucket values or proof of effective account configuration. These packaged hashes
are artifact metadata, not copied account data.

Pcn.A01 selects its v4 parser using the file prefix (or bounded filename fallback
after a failed/empty version read), optionally locates params_names and determines
selected universe/sessionless flags beforeA02. The full dispatcher is retained.
Its body parser, translation row semantics and named-map behavior are not ported
from an assumed text format. The .txt suffix does not imply text rows.

## Reproduce and next boundary

```sh
python3 docs/protocol/apk/2.26.43.72-token-param-header/derive-asset-headers.py --apk ../android-lab/apks/WhatsApp.1.apk > /tmp/token-param-headers.json
cmp /tmp/token-param-headers.json docs/protocol/apk/2.26.43.72-token-param-header/asset-headers.json
```

The extractor pins the whole APK and rechecks member SHA256 against the previous
[asset inventory](../2.26.43.72-token-mobileconfig-binding/parameter-assets.json).
It reads selected ZIP members only and commits no raw binary asset. Exports use
apk-smali --signatures --exceptions forPfF.A01/A00,PHU.<init>,0A8.A05,Pcn.A01.
Re-use full0A8.BPZ from the earlier binding bundle for call-site conditions.

Remaining: translation body semantics, actual loaded-table schema and hydration,
complete manager/flag selection and00E effective-cache lifecycle. No Rust timing,
admission or wire request changed; no private state or live account operation.
Full network parity and restriction outcomes remain unproven.
