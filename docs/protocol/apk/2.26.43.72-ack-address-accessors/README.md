# Optional typed ACK addresses — APK 2.26.43.72

**Confirmed static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
rehashed 2026-11-06. Complete selected DEX methods with signatures/catch tables;
xtask fallback, not REA Evidence envelopes or runtime evidence. No account use.

[Existing complete 1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt)
requests Jid for from at000e..002c and participant at002c..003c, and UserJid for
recipient at003c..0054. These call 0Tp.A0A before required id and numeric error.

A0A reads cached 0Tl.A01 when present. Otherwise it calls A0M(name,null), then
0Dd.A02. A02 returns null for null/empty input and catches all throwables from
its selected A00 parse invocation, translating failure to null. The full A00
parser grammar and provider effects are not interpreted here.

If A0A obtains a Jid with isProtocolCompliant=false, it logs at003a..008a,
then continues. It does not throw LIL on that branch. Its optional DeviceJid
conversion is selected only for requested DeviceJid; these three ACK accesses
request Jid/UserJid instead. A0A Class.cast at00aa..00b8 catches
ClassCastException, logs and returns null at00b8..00f6. A failed recipient
UserJid cast therefore does not itself establish a fatal ACK admission error.
This is not proof every instruction/helper can never throw.

## Negative result and remaining gap

These accessors do not justify extending Rust's fatal ACK validator with a
blanket invalid-address rejection. Missing required id remains a different
accessor (A0L throws LIL). Rust receipts::jid likewise returns None on parse
failure, and outgoing::acknowledge refuses ordinary correlation without from.
That is a local supported subset, not complete native address equivalence.

The message branch at0294..02ac prefers nonnull recipient, otherwise normalizes
from through0Dn.A00; absent chat logs and goes to the generic ACK callback path.
Native recipient selection, 0Dn normalization, special user subtypes and
subsequent generic callbacks still need a separate authority audit before
changing settlement. Typed cached/nontext representations, duplicate attributes,
full JID grammar and native account cleanup remain open. No Rust behavior change,
new NACK, request, replay or connection stop follows from this negative finding.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0A --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method A02 --signatures --exceptions
```

checksums.json covers every other file. The caller and A0M required/optional
text accessors are already retained in linked bundles and not duplicated.
