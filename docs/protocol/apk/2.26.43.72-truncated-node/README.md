# Truncated node reads — APK2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports; Android-provider fallback already diagnosed.
These are not REA Evidence or target runtime observations. No live accounts.

## Fixed-size buffers are read completely before interpretation

[123.A06](buffer-reader.txt) loops InputStream.read into the remaining buffer.
At0014..002e a read result-1 throws IOException with fixed text
ran out of bytes while reading into buffer. It has no recovery catch.
The [already retained123.A07](../2.26.43.72-packed-nibble/unpack.txt)
reads the packed count, allocates all declared pairs, then invokes A06 at001e
BEFORE digit count/iteration and nibble validation. Truncated packed input with
an invalid first nibble therefore fails its complete-buffer read first. The
native final odd padding still is not an actual digit, and pairs0+odd still
has the prior negative-digit-count refusal.

[123.A01](../2.26.43.72-group-missing-identification/current-jid-attribute-reader.txt)
uses A06 for binary content, e.g. BINARY8 length/read at0172..0184, and recursively
calls A01 for children. [A0A](../2.26.43.72-packed-nibble/string-reader.txt)
uses A06 for raw string bytes; a missing string token (-1) instead selects its
LIL at026c..0284. [A00](list-size-reader.txt) rejects an invalid initial list
marker, including read result-1, through its LIL at002e..005e. LIST8/LIST16
count-byte reads are not individually EOF-checked there: do not relabel every
native truncation as the same IOException. Negative count/array/lookup paths
and untraced alternate native formats are not fully emulated by this change.

## Complete packed-value failures also leave the reader

The same complete A07 selects bad nibble LIL at006a..0092 for actual nibble
values13..15 and negative-count LIL at010c..011c. These exceptions are raised
inside A08's initial parse, not from a successfully parsed stanza's handler.
Rust now makes invalid_packed_digit terminal and gives the negative-count case
its own public CodecError::InvalidPackedLength/classinvalid_packed_length,
instead of routing the entire shared invalid_node class as terminal. Valid
colon12, hexadecimal values and odd final padding keep their prior behavior.
Old saved invalid_node diagnostics are not relabeled or replayed.

## Initial parse failure is outside recoverable stanza handling

[123.A08](../2.26.43.72-stream-end/frame-reader.txt) bounds the node input to the
authenticated record or its inflated remainder. It cannot complete a truncated
node by taking bytes from the next encrypted record. [1s2.run](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
invokes A08 at0058..005e, covered by002a..0034 code units. Both IOException and
LIL first enter088a (byte1114), clean the reader-local1ru map, then rethrow.
Outer085c..08ba routes IOException to0912 (byte1224), LIL to08d9 (byte11b2).
These paths call onError and return; they do not reach the inner parsed-node
recoverable handler that may construct a487 response. No universal map cleanup
for every failure or durable reservation deletion is inferred.

The [concrete error listener and remaining manager gaps](../2.26.43.72-compression-failure/README.md)
were retained in the preceding compression investigation. Native error numbers,
global reconnect/backoff and full runtime parse-error parity remain distinct
from Rust's local PeerProtocol category. Existing future/callback uncertainty
cannot authorize replay.

## Rust consequence

[Canonical receive behavior and verification](../../frame-admission.md#truncated-nodes-consume-the-receive-epoch-2026-11-06).
Failing-before controls retain the packed error-priority mismatch, the usable
encrypted channel after truncation and the usable channel after a complete
invalid packed digit. Native selected buffer/token failures
justify terminating a truncated stanza; Rust applies that local refusal to its
whole unexpected_eof class without claiming identical native exception details
for every binary format. Pure offline decode still returns the codec diagnostic.

## Reproduce

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A07 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A0A --signatures --exceptions
```

PCs are bytes; catch coordinates are code units (multiply2). Linked complete
methods are not duplicated. checksums.json covers every other file here.
