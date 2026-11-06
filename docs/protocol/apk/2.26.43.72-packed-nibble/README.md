# Packed nibble alphabet and actual digit count — APK 2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence or runtime observations.
No live account operations.

## Native writer and reader agree on colon

The retained [1r2.A06 writer](../2.26.43.72-node-serializer/bytes.txt) tries
NIBBLE_8 (ff) for packed-eligible strings shorter than128bytes. It maps ASCII
0..9 to0..9, '-' to10, '.' to11 and ':' to12. Fallback tries uppercase hex
(fb) then raw bytes. Odd lengths put f in the unused low nibble. The retained
[A05 context](../2.26.43.72-node-serializer/string.txt) first looks up tokens;
its last call forwards its packing boolean intoA06. Native node attribute
values invokeA05 with packing enabled; other context-sensitive choices,
including text-to-JID conversion and tag/name packing, are separate authority.
No claim that every string position must use packed bytes follows.

123.A07 is reached by two exact physical callers: A01 node reader andA0A
string reader. The former's [complete existing body](../2.26.43.72-group-missing-identification/current-jid-attribute-reader.txt)
selectsA07 forfb/ff content; A0A selects it for those string-value tags and
constructs a String from returned bytes. The constructor-only1qz export is
record context, not the packed parser; the connection's previously retained
0ZJ initialization constructs123 around it. No parser provider was guessed
from a class name.

AtA07 0000..002e the header's low7bits give byte-pair count, highbit gives odd
count, and actual digits are2*pairs-odd. Negative count throwsLIL. Only those
digits are interpreted. For ff, 0..9 become digits,10/11 become '-'/'.',12
becomes ':', and13..15 throwbad-nibbleLIL. For fb all0..15 become uppercase
hex digits. The last low nibble of an odd string is never interpreted: any
padding nibble is ignored, even13/14/15. Do not require f padding on input
solely because the writer emits it. Empty even packed value is admitted;
zero pairs with oddbit is rejected.

## Confirmed Rust mismatches fixed

Rust's encoder did not select colon for nibble packing, decoder rejected12,
and decoder treated15 as a NUL data character. It also decoded/validated the
padding then popped it, rejecting invalid padding digits and accepting a
negative actual count as empty. Rust now selects colon in its existing
packing contexts, maps12 to ':', rejects13..15 in actual nibble data, skips
odd trailing padding before digit validation and rejects oddzero-pair input.
Hex15 remains F. Existing resource limits and tag/JID handling stay unchanged.

Literal fixtures use the established synthetic web dictionary: iq25/id8.
`00 f8 03 19 08 ff 01 c1` encodesid=:1; prior encoder emittedrawfc02 3a31,
and prior reader rejected the packed fixture. Both tests failed before fixes.
Reader fixtures independently cover colon, odd f padding, ignored d padding,
empty even value, negative digit count, invalid actual13/14/15 and hexCF.
These are offline wire fixtures, not native execution, server rejection or
account-restriction observations. Decoder behavior is shared across transport
node parsing; it does not bypass later response/notification admission.

## Reproduce

From checkout root, pinned APK outside Git; use a separate output directory
and compare complete bytes with retained filenames:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A07 --signatures --exceptions > unpack.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A0A --signatures --exceptions > string-reader.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A07 > unpack-callers.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1qz;' --signatures --exceptions > record-context-constructor.txt
```

PCs are bytes; branch deltas are code units. Caller census is physical direct
references, not runtime coverage. Remaining work: complete context-specific
string/token/JID emission parity, native parser fault routing, disconnect
ownership and other outbound/inbound families. No full codec/ban guarantee.
