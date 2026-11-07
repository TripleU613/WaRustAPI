# Offline item timestamp versus status conversion — APK 2.26.43.72

**Confirmed selected static / fixed offline**, build 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected checked-DEX exports with catch tables, xtask fallback;
no runtime execution, account traffic or REA Evidence envelopes.

## Conversion is later than required time parsing

The retained [complete F10.A03](../2.26.43.72-receipt-null-fact-actor/receipt-parser.txt)
offline message_id branch first obtains a required DeviceJid at0098, reads the
item type with absent default delivery at00a4, reads required t at00b0, then
calls F10.A00(kind) at00ca. Reading the type string is not converting it.
The [required timestamp chain](../2.26.43.72-receipt-timestamp-errors/README.md)
throws LIL on missing or invalid supported long text before the status converter
is reached. A required-device failure precedes both time and conversion.

[Complete F10.A00](status-converter.txt) uses hashCode/sparse-switch plus exact
String.equals checks. Its selected mappings are delivery/sender ->5, played ->8,
read ->13, inactive ->15, read-self ->17 and played-self ->18. Unknown or empty
non-null text falls to IllegalArgumentException, not LIL. Nontext/local enum
carrier equivalence is not generalized from these string methods.

The [ordinary wrapper trace](../2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md)
binds LIL to fresh487. For non-LIL, the selected legacy 14h path rethrows;
queued1ic.A0E with a node invokes [1rg.A00](nonparse-error-handler.txt), which
calls the fresh context helper with500, then performs separate completion
bookkeeping. This does not prove that every incoming unknown type reaches a
particular wrapper, or that Rust should emit500 for every local refusal.

## Confirmed local ordering correction

Rust had checked its supported item-kind set before reading t. An unknown item
kind therefore hid a prior native timestamp failure and suppressed its evidenced
487 response. The kind conversion/refusal now occurs after required timestamp
parsing, matching the native order. It remains a generic refusal when time is
valid; no blanket487 or new500 behavior is added. Missing kind still defaults
to delivery. Opaque type values retain their existing unsupported-carrier policy.
Supported fact kinds, saved IDs, identity and retry/query authority are unchanged.

`offline_item_timestamp_error_precedes_unknown_status_conversion` failed before
correction on the wrong cause field. The codec-roundtripped table covers unknown
and empty kinds with absent, malformed and overflowing item times, verifies the
native_timestamp cause and fresh root read/error487 ACK, and keeps unknown kind
with valid time outside that classifier. A constructor-null actor control retains
native_required_device priority. Existing aggregate atomic-refusal tests and
normal actor capture/write-cut/no-replay checks run with this ordering change.

**Remaining:** native wrapper selection/non-LIL500 policy, exception lifecycle,
opaque type/binary construction, other schema/order cases, full native storage
and live acceptance. This is a specific precedence correction, not a universal
error-response mapping or a new receipt feature. No account operation.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/F10;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rg;' --method A00 --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features offline_item_timestamp_error_precedes
```

The complete outer parser and error paths are linked, not duplicated.
checksums.json covers every other file by byte length/SHA256; no APK/private state.
