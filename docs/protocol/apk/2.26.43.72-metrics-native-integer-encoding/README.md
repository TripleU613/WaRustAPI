# Native metrics integer and field encoding — APK 2.26.43.72

**Confirmed static plus observed bounded synthetic emulation.** Pinned recovered
ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`,
from APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Full reconstruction checks](../2.26.43.72-superpack-reconstructed-members/README.md)
bind the image. Only selected encoding helpers executed in Unicorn on synthetic
data adapters. No Android app/JNI, credentials, actual telemetry, I/O or network
operation. These are not REA Evidence envelopes.

## Field encoder reached from the native pending worker

The [retained worker](../2.26.43.72-metrics-native-drain-persistence/README.md)
creates a field builder with 0x72be34, constructs a record for the event code and
weight through 0x722264, then calls **0x72bf48**. It also invokes the same encoder
for an array of other records. The selected worker call-site window and complete
field-encoder body are retained. Full event ordering, all array producers,
batching, checksums and resulting transport payload remain unresolved.

The field encoder reserves a leading byte, encodes record key +0x18 via
**0x72c184**, requires returned key-width tag 3 or 4, and then selects its value
path from the value object's raw kind at +0x18. The exact five-byte switch maps
raw kinds 0..4 to no-value, integer, floating, string and boolean branches.
These names derive from the called accessors/operations, not source enum names.
Float/string conversion, invalid-value behavior and end-to-end field construction
are not exhaustively established by the integer fixtures below.

Before patching the reserved byte via 0x72c280, the selected body computes the
low byte of `record_flags | (value_tag << 4)`, additionally ORing 8 when the
key-width tag is 4. Record flags come from record +0x28. Key-width rejection and
that expression are static observations, not a complete network-frame grammar.
The integer helper supports wider keys internally, while this caller rejects
key-width tags outside 3/4. No new Rust key range or telemetry sender is added.

## Exact integer bytes and tags

General helper **0x72c184** chooses the smallest signed width that preserves its
64-bit input and appends little-endian two's-complement bytes:

| Signed input range | Bytes appended | Returned tag |
| --- | --- | --- |
| -128..127 | 1 | 3 |
| -32768..32767, outside prior range | 2 | 4 |
| -2147483648..2147483647, outside prior ranges | 4 | 5 |
| Remaining signed 64-bit values | 8 | 6 |

Scalar helper **0x72c244** returns tag **1 for zero** and **2 for one**, appending
no value bytes for either. Other values branch to the general helper above.
This distinction matters: the general key/length helper still emits a byte for
0/1, while scalar values can use their header tag alone. Boolean encoding in
the field body selects its low bit and uses that scalar helper. The string
branch uses the general width helper for representation length, then adds five
to its returned width tag; complete string/encoding limits need separate proof.
No fixture here claims a complete event or request was serialized.

## Synthetic native execution checks

check-integers.py uses Unicorn 2.1.4 to map pinned ELF PT_LOAD bytes and supported
relocations. It supplies a writer and a bounded synthetic MCFData object. Only
selected length, append and mutable-byte-pointer operations are modeled; all
other called imports/interrupts stop the run. A synthetic thread-local canary
supports the native stack checks. Real MCF allocation, ownership, failures,
threading and the whole metrics runtime are not emulated.

**42 cases pass**: 21 signed values run through both helpers, including signed
64-bit endpoints, signed 32/16/8-bit boundaries and adjacent values, 0/1, 255,
65535 and 65536. Native tags and exact bytes match an independent Python
signed-width/two's-complement encoder. Each case has a 100000-instruction /
one-second engine bound and must return to a sentinel. Inputs, tags, output
hex and imports are retained in integer-results.json. The enclosing field/header
encoder is static evidence in this bundle, not an executed full-field fixture.

## Reproduction and remaining work

Run `python inspect-encoding.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
encoding-boundaries.json. Under the Unicorn venv run
`python check-integers.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
integer-results.json. Both reuse the adjacent digest-checked image resolver.
Rizin exports reproduce with `pD SIZE @ VA` from the byte windows; actual section
mappings bind hashes, rather than assuming text VA equals file offset.
The complete selected field/general/scalar/header-patch helpers and selected
worker call site are retained. [Section map](../2.26.43.72-metrics-native-event-routing/elf-sections.txt)
and [tool version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) apply.

Next verify complete field framing and float/string cases, then follow encoded
buffers into ownership, flush, transport and server-response handling. Full
metrics parity remains unproven; no Rust runtime, telemetry, route or live change.
