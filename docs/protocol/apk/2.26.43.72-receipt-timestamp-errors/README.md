# Aggregate timestamp errors and native NACK classification — APK 2.26.43.72

**Confirmed selected static / fixed offline**, build 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX exports with catch tables, existing
xtask fallback; no native execution, traffic or REA Evidence envelopes.

## Required versus defaulted timestamps

The retained [complete F10.A03 parser](../2.26.43.72-receipt-null-fact-actor/receipt-parser.txt)
uses A07(t) at00b0 in the offline message_id item branch and A08(t,default) at0376
in the key item branch. [A07](required-long.txt) gets a required string via
[A0L](required-string.txt), then invokes [A09](parse-long.txt). Missing/null
string makes A0L throw LIL. Empty strings are not missing at A0L; they reach
Long.parseLong and fail there. A09 catches NumberFormatException and throws LIL.

[A08](default-long.txt) returns the supplied default only when its A0M string
result is null. A present supported text value goes through the SAME A09 parser;
malformed/overflowing text does not take the default. The existing Java UTF16/BMP
signed-decimal adapter remains unchanged: negative values, plus signs, BMP digits
and both i64 extrema are not malformed merely because they are unusual times.
Opaque caller/codec attribute representations are not newly equated with native
null text or NumberFormatException.

The [ordinary handler/error trace](../2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md)
binds LIL from F10.A03 to 1rg.A04/A03 and fresh root error487 output on the two
selected wrapper paths. This supports these specific timestamp causes, not every
receipt refusal or the complete native storage pipeline.

## Countercheck: root delivery does NOT gate key aggregates

An initial partial reading of F10.A03 0284..028c suggested that non-delivery
roots skip the key branch. Following the complete branch targets REFUTED that:
028c + 2*0x8f =03aa; at03aa v4 becomes null, and03ac - 2*0x6e =02d0. This
rejoins before the participants/key block at02dc..030a. The delivery comparison
selects optional biz/privacy metadata extraction, not key-aggregate parsing.
The missing-biz branch at029c also goes to03aa. Printed PCs are bytes; DEX branch
operands are code units. Existing read/played key routing and the root-kind fact
regression are retained. No root-versus-inner ID routing change is published.

## Local implementation and regressions

The parser now uses distinct diagnostic cause
`receipt.aggregate.native_timestamp` for missing required offline item time and
invalid supported signed-long item text in either aggregate branch. Missing key
item time retains its existing sampled clock default. Opaque time attributes
remain generic unsupported failures. A shared native-error-field predicate keeps
normal receive and the public validation-gated NACK builder synchronized with
the existing required-device cause. CLI receive paths reuse that builder.

The supported-recipient no-fact shortcut now preflights selected participants
aggregates through expand, including the key form. It grants no additional fact
projection, but cannot silently bypass a confirmed item parse error solely
because a recipient is present. Non-aggregate recipient behavior is unchanged.

`aggregate_timestamp_native_errors_get_487_without_reclassifying_defaults` first
failed because invalid numeric text had no error response. Its codec-roundtrip
table covers empty, fractional, spaced/sign-only and both-sided overflow text;
valid signed/BMP/i64-extreme controls; required missing time; absent key-time
default; and opaque timestamp refusal without a native NACK. The extended normal
receive actor checks missing/empty/overflow time in both selected aggregate forms,
a real UserJid recipient, fresh root response, capture-before-write and unchanged
outbox/identity/session snapshots. Existing actor redelivery/write-cut/reopen/no-
replay checks and aggregate atomic-fact tests remain in the same suite.

**Remaining:** root ordinary timestamp parsing and other error causes; full
native null-string/opaque binary equivalence; native clock acquisition/preference;
queue/storage/feature behavior; all request families and live acceptance.
No global Jid parser, identity change, resend/query authority or account operation.
No zero-restriction guarantee is inferred from static fidelity or local tests.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A07 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A08 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A09 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0L --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features aggregate_timestamp_native_errors
cargo test --locked -p whatsapp-api --all-features offline_required_actor_parse_error_gets
```

checksums.json covers every other file by bytes/SHA256. Existing complete outer
parser and error-handler evidence is linked, not duplicated. No APK/private state.
