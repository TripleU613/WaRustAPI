# Native metrics request tags and metadata — APK 2.26.43.72

**Confirmed static / derived; observed synthetic tag-branch selection.** Pinned
APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No live request, account, Android runtime or real telemetry value was used.
Local Rizin/Unicorn outputs are not REA Evidence envelopes. No Rust production
behavior or mandatory-telemetry requirement was added.

## Descriptor and selected sender values

General builder **0x72ca94** reads this native descriptor:

| Offset | Field / kind |
| --- | --- |
| +0 | optional product C string |
| +8 | optional surface C string |
| +0x10 | purpose unsigned enum |
| +0x18 | optional application_tags C string |
| +0x20 | request_category unsigned enum |
| +0x24 | is_ad raw enum |
| +0x28 | retry_attempt signed formatting input |

The [complete sender](../2.26.43.72-metrics-native-transport-responses/buffer-sender-assembly.txt)
at0x728470 saves manager+0xc8 as product source; it later converts that MCFString
and two empty literals to C strings, constructs the descriptor at sp+0x78, sets
purpose=**1**, category=**4**, leaves is_ad/retry zero, and calls the builder at
0x7287f4. The two empty strings supply surface/application_tags. Manager+0xc8's
upstream value remains unknown; no product identity was invented.

The builder maps purpose1 to **prefetch**, not fetch. These strings describe
native enum labels; their broader policy meaning is unproven. The selected
network-tag members on normal construction are optional nonempty product,
`purpose:"prefetch"`, `request_category:"analytics"`, `retry_attempt:"0"`.
Surface/application_tags are omitted by the empty-string pair guard; is_ad0
is omitted before calling that helper. Header value goes through MCFString
conversion; selected null conversion uses `{}` fallback. Allocation failure
and malformed product contents are not proved to produce that fallback.

## General enum and omission rules

Purpose table0x2e9638 and category table0x2e963c are independently resolved:

| Raw | purpose | request_category |
| --- | --- | --- |
| 0 | omit | omit |
| 1 | prefetch | image |
| 2 | fetch | video |
| 3 | refresh | api |
| 4 | empty/omit | analytics |
| 5 | empty/omit | upload |
| 6 | empty/omit | audio |
| 7 | empty/omit | document |
| above range, including signed-negative bit patterns | empty/omit | empty/omit |

is_ad0 skips; raw1 selects string **0**, raw2 string **1**; other values select
empty string and are omitted by the pair helper. These are string values in
this header, not Boolean JSON literals or a server restriction field.

Pair helper **0x72cd78** uses strlen checks: an empty key/value adds nothing.
It grows a local buffer and formats string values as `"%s":"%s",`, except
values beginning with `{` use raw-object format `"%s":%s,`. The selected code
does not perform JSON escaping. The builder replaces the first/last delimiters
with braces, wraps its selected inner object as **network_tags**, then optionally
adds **application_tags** outside that inner object. This is static formatter
behavior, not proof arbitrary input strings form valid JSON.

Retry formatter0x72cd24 uses `%d` with size5; full behavior for general large or
negative counters is not executed here. In particular, do not assume unlimited
integer text capacity. The selected sender supplies zero, which fits. Full
malloc/realloc failure handling, string safety and exact final bytes remain
unverified; the branch fixtures below intercept the pair/integer formatters.

## meta_data member provenance

The same sender0x72857c..0x728670 constructs the separate multipart JSON:

- `t`: raw integer returned by0x72c4fc. The clock's full runtime contract/units
  remain unverified, not a fabricated timestamp.
- `p`: signed byte manager+0x30 loaded into w27 and supplied to `%u`. The raw
  mode's semantic domain remains unknown; it is not a proved process ID.
- `rt`: literal **true** only when saved channel equals3; otherwise **false**.
  It is an unquoted JSON Boolean in the format string.
- `config_id`: present only when current credential data+0x28 is nonnull. It
  is produced by MCIHexStringCreateFromDataWithDelimiterChar with delimiter0,
  converted to C string and passed as the quoted field. Hex case and complete
  conversion/error behavior are not reconstructed here.

Exact formats are retained in tag-boundaries.json and the earlier
[request framing bundle](../2.26.43.72-metrics-native-request-builder/README.md).
Credential+0x28 is not the Host header source; Host instead comes from native
selector0x1a. Application-tags, network-tags and meta_data are separate layers.
Literal field presence does not establish a mandatory request requirement.

## Fixtures and reproduction

**24 bounded native selection cases** exercise the branch window
0x72caec..0x72cc90 using synthetic descriptors. They cover selected sender,
optional/empty strings, all documented enums, negative/out-of-range values and
is_ad mapping. The actual pair helper is intercepted to record key/value calls;
a controlled formatter writes zero for the exclusively zero retry inputs.
The results show nonempty pairs using the independently retained helper's
empty-string rule; **they do not execute complete JSON serialization**. The
application_tags stage lies beyond this window and has static evidence only.
No allocator or external I/O is invoked by the executed window.

Run `python inspect-tags.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
tag-boundaries.json. It reuses the digest-pinned ELF resolver, decodes switch
destinations and extracts mapped window hashes/literals. Rizin windows.json uses
`pD SIZE @ VA`. With Unicorn2.1.4 run
`python check-tag-selection.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
tag-selection-results.json. Each case has100000-instruction/one-second caps;
unexpected imports/interrupts or unsupported loader relocations fail closed.
Checksums cover all other files.

**Inferred:** these exact mappings are compatibility inputs if this native
metrics path is ported. **Observed live:** none. **Unknown:** upstream product
value, general formatting/escaping/allocation behavior, metadata clock/mode
semantics, actual Host/fallback selector plaintext, cryptography and deployment
necessity. No full metrics parity or account-restriction guarantee is claimed.

## Complete selected serialization fixtures (2026-11-05)

**Observed bounded synthetic emulation:**19 additional cases execute builder
0x72ca94 through its native return, including pair helper0x72cd78, retry
formatter0x72cd24, dynamic buffer growth and native variadic-list setup. This
extends the branch-only fixtures above; their narrower claims remain valid.
No formatter helper is intercepted in this additional harness.

Imported malloc/realloc/free/strlen are explicit bounded emulated-heap adapters.
Imported vsnprintf decodes the actual native AArch64 va_list register/stack
layout, permits only the three retained fixed formats, then delegates their
bounded formatting to host libc snprintf. It copies the result into emulated
memory with C size/NUL semantics. Android's libc implementation is not executed
or claimed identical beyond these tested supported operations. Unexpected
formats/imports, out-of-bounds allocator destinations, unterminated strings and
target interrupts fail. Real heap allocation failure is not modeled.

Exact native bytes match independent JSON serialization for supported safe
inputs, plus an explicitly invalid-JSON quote fixture. Covered cases include:

- Selected purpose1/category4 with retry boundaries0..9999 and signed extremes.
- Retry10000 → string `1000`; signed maximum → `2147`; signed minimum → `-214`.
  These are actual native caller bytes with the controlled snprintf adapter,
  not a speculative interpretation of the size5 argument.
- Optional surface and nested application_tags, application string, empty
  fields and a512-character product that forces buffer growth.
- A product beginning with `{` is inserted as a raw nested object. It is not
  quoted as an ordinary string by the selected pair helper.
- Synthetic product `synthetic"quote` produces unescaped embedded quote bytes;
  the complete result fails a separate JSON parser as expected. No valid input
  domain or production product value is inferred from this malformed fixture.

On the ordinary selected sender fixture with synthetic product, output is:

```json
{"network_tags":{"product":"synthetic-product","purpose":"prefetch","request_category":"analytics","retry_attempt":"0"}}
```

All19 cases return exactly one live synthetic allocation: the returned outer
string. Inner allocations/replaced buffers are accounted for by the adapters.
This is not a whole-runtime leak, allocator-failure or concurrency proof. The
malformed input demonstrates a formatting boundary; it does not justify
accepting malformed JSON or inventing telemetry in a Rust port.

With Unicorn2.1.4 and host libc available, run
`python check-tag-serialization.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must
match tag-serialization-results.json. Each case has100000-instruction/one-second
caps. Native varargs support windows were added to windows.json and the pinned
resolver. No real credentials, HTTP header transmission or Android process is
used. **Still unknown:** arbitrary-input/general allocator behavior, real
product values, full request acceptance, metadata semantics and credential
cryptography. The bounded final serialization gap for these fixtures is closed;
full protocol correctness and deployment requirements are not established.
