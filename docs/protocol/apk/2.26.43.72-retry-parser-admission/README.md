# Retry parser field admission — APK 2.26.43.72

**Confirmed static / Rust correction tested offline.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The complete [F10.A04 parser export](../2.26.43.72-retry-receipt-keys/receipt-parser.txt)
checks registration bytes length4 at0012..002e, throws LIL at03b4..03c2 for
null/wrong-length content, defaults retry v to"1" at002e..0046 and returns null
for another version at004a..0050. Its count reader at0064..006e passes default1.

Complete checked exports here bind `0Tp.A05(String,int)`: absent attribute returns
the supplied default; a present attribute delegates to `0Tp.A06(String,String)`.
The latter invokes Java Integer.parseInt and converts NumberFormatException to
LIL. No APK/app or live request was executed. These local exports are not REA
Evidence envelopes.

[Implementation, tests and remaining fields](../../retry-parser-admission.md)
distinguish these selected native rules from full retry/ACK parity. Rust's model
represents nonnegative counts only; negative Java integers are not newly enabled.
Full Java Unicode numeric syntax and nested retry id/t routing are not claimed.

Reproduce one method per command with the pinned APK outside Git:

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0Tp;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0Tp;' --method A06 --signatures --exceptions
```

Stdout matches count-default-reader.txt and count-present-reader.txt. The linked
parser has its own bundle checksums and full signature. Checksums here cover this
README and the two complete exports. Full server acceptance/ban behavior remains
unknown; no mandatory request or telemetry is inferred.
