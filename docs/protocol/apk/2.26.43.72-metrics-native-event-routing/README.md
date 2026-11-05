# Native metrics JNI event routing — APK 2.26.43.72

**Confirmed static / derived register flow**, pinned reconstructed ARM64
libwhatsapp.so. No JNI call or metrics code was executed. Local Rizin/readelf
exports are not REA Evidence envelopes. The [reconstruction evidence](../2.26.43.72-superpack-reconstructed-members/README.md)
binds this 12783016-byte image to the pinned APK through metadata size/hash
checks and full reproduction. Library SHA256:
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.

## Exact JNI tuples and code mapping

resolve-jni.py verifies image identity, parses ELF sections and RELA relative
relocations (type 1027, symbol index zero), then resolves these exact tuples:

| Method | Native entry VA | Entry file offset |
| --- | --- | --- |
| jvidispatchIIIIIOO | 0x6f71ac | 0x6f31ac |
| jvidispatchIOOOO | 0x6eee64 | 0x6eae64 |

Full signatures and tuple-slot mappings are retained in jni-tuples.json. They
match the [native DEX declarations](../2.26.43.72-metrics-native-carrier/README.md)
for com.whatsapp.wamsys.JniBridge. This is a derived relocated tuple mapping;
registration execution and every enclosing table/class registration call remain
unobserved. Unlike selected decoder windows, this library's text VA differs from
file offset by 0x4000. All byte-window hashes use actual section mappings.

## Event construction preserves a channel distinction

The [Java worker](../2.26.43.72-startup-metrics-native-handoff/README.md) supplies
four longs: event code, weight, psId, channel; followed by native context and
field map. Outlined helper bodies are retained so register moves are not guessed
across calls. The event entry unwraps/retains the native context through
0x6e99e8 and converts the field-map object through 0x6e9f88 (full Java-to-MCF
conversion semantics still untraced).

It uses the **low 32 bits** of all four longs. Event code and weight are
sign-extended from those 32-bit values for constructor 0x721b54. The constructor
aborts on a negative event-code argument, initializes native arrays and stores
code, weight and channel; psId slot +0x40 starts at -1. Channels 0/1 retain that
default; channels 2/3 additionally store low-32-bit psId into +0x40. Other channel
values hit abort in this selected entry. This is native behavior under the Java
worker's existing bounded channel mapping, not new permission to invent IDs,
weights, channels or telemetry. No Rust value range is changed from this trace.

After field conversion, it reads native-context slot +0x38. If nonnull, it calls
manager helper **0x7259cc** with the event; otherwise that manager handoff is
skipped. It releases event, converted map and context. The normal return is
**literal zero**. No per-event network acknowledgement is returned by this
entry; zero cannot establish serialization, disk persistence or server acceptance.
Null-pointer/error behavior of the called conversion routines remains outside
this selected interpretation.

## Typed field visitor and local manager branches

Helper **0x6f8164** traverses keys/values of an MCF dictionary. It converts each
numeric key through 0x6f8314 and selects string, number or boolean handling using
MCF type APIs. Numbers branch again on MCFNumberIsFloatType, with separate float
and integral conversion calls. MCFNull values take the next-item path without
adding a field. Unsupported types call a diagnostic function pointer associated
with source string `whatsapp/xplat/android/src/WhatsAppMetrics/WAMEventDecoder.c`,
then continue the loop. Full conversion failure/exception policy, field ordering
and wire encoding are not established by this visitor. Its typed helper calls
are not wire-format type tags.

Manager 0x7259cc reads raw state +0xc8:

- States 0..3 lock its mutex, append the event to local array +0x50, unlock, and
  reach helper 0x7239ec. Event code 0xa0e has an additional diagnostic call.
- States 4/5 call 0x71ffc8 and tail-call 0x724fc8 with selected manager fields.
- Other values reach abort.

State names, flush semantics, batch construction, replay, storage durability,
network ownership and response interpretation remain unknown. A local append
and scheduling call must not be counted as delivery or a requirement to send
one network request per Java event.

## Setup selector boundary

The exact six-byte selector table maps selector **0** to 0x6eeec0. That branch
converts/unpacks its objects, calls helper **0x6f8b90** and returns its low bit via
common cleanup. Other selectors are retained for control-flow context but not
assigned metrics-only meanings; this dispatcher is shared with unrelated work.
The selected 460-byte setup-entry excerpt checks context slot +0x38 and returns
zero immediately when it is already populated, otherwise begins dictionary
configuration. Full 0x6f8b90 initialization is much larger and remains untraced.
The [Java setup/latch trace](../2.26.43.72-metrics-readiness-latch/README.md) already
shows that the JNI return is discarded; neither a zero return nor latch opening
is a standalone native-manager health or wire-delivery check.

## Reproduction and remaining work

Run `python resolve-jni.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
jni-tuples.json. Rizin exports use `pD SIZE @ VA`, with ranges.json supplying
image/file mappings and raw byte digests. elf-sections.txt is retained locally;
[tool version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) remains
applicable. The event entry, constructor, visitor, manager helper and setup
selector body are retained as complete selected bodies. Setup initialization is
explicitly only an excerpt; called helper semantics are not promoted to full
coverage. Checksums cover every retained file.

Next trace manager initialization, 0x7239ec/0x724fc8 buffering/flush and eventual
serialization, storage, transport and server responses; then fix any confirmed
Rust mismatch with evidence-backed tests. No Rust behavior, HTTP route, sender,
telemetry, retry or live account/network operation changed.
