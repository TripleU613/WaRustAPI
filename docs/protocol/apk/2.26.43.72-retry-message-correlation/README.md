# Retry message key and required timestamp — APK 2.26.43.72

**Confirmed static / Rust correction tested offline.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK, network request, account or live retry was executed. Local checked DEX
exports are not REA Evidence envelopes.

The complete [F10.A04 parser](../2.26.43.72-retry-receipt-keys/receipt-parser.txt)
loads the nested retry node at0002..0012. After registration/version/count:

- 0070..007c calls **retry.A07("t")**, obtaining a long;007c..0084 multiplies
  it by1000 before forwarding the parsed value into EYC.
- 0084..0090 calls **retry.A0L("id")**. This is the nested child, not the root
  receipt. It passes the string through DBb.A0g to1dW's constructor at00c0..00c6.
- The constructor stores that string in **1dW.A01**, remote JID in A00 and
  from-me flag in A02. It is the failed original message key.

Complete0Tp.A0L requires a nonnull string and throws LIL for a missing attribute.
A07 calls A0L then A09; A09 uses Java Long.parseLong and translates
NumberFormatException to LIL. Thus missing/nonintegral/overflow timestamp does
not produce a parsed native retry here. The native signed multiplication is not
reimplemented as a new Rust clock/age policy: this correction only validates the
required timestamp and continues using the persisted original attempt for age.

The Rust shared parser now returns nested retry.id as RetryRequest.message_id,
requires nested t parseable as signed64-bit and keeps the prior local nonempty/
128-byte identifier bounds. It does not require equality with the separately
validated root receipt ID or use root ID as a substitute. Root receipt ACK and
correlation semantics are not changed by this correction.

[Implementation and regressions](../../retry-parser-admission.md#nested-message-key-and-timestamp-correction-2026-11-05)
cover different root/nested IDs, malformed/missing nested fields, signed time
boundaries and preservation of retry state on refusal. Full Java Unicode numeric
syntax, timestamp consumers and all receipt/routing policy remain unverified.
No new public model field or live compatibility claim is introduced.

Reproduce with the pinned APK outside Git and compare complete stdout:

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0Tp;' --method A0L --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0Tp;' --method A07 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0Tp;' --method A09 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/DBb;' --method A0g --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/1dW;' --method '<init>' --signatures --exceptions
```

The constructor export intentionally includes both overloads; its exact targeted
signature `(LX/0DY;Ljava/lang/String;Z)V` is identified above. Other exports each
select one method. Checksums cover this README and complete exports.
