# Ordinary root receipt timestamp is lenient — APK 2.26.43.72

**Confirmed selected static / verified offline**, build 264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX method/catch-table exports through
existing xtask fallback; no native execution, account traffic or REA IDs.

## Do not apply aggregate item errors to ordinary root t

The [complete current F10.A03](../2.26.43.72-receipt-null-fact-actor/receipt-parser.txt)
at04d8..04ea reads root t with 6oe.A18 then 6ol.A06. This is not the
0Tp.A07/A08 -> A09 chain used by aggregate items.

[6oe.A18](optional-root-string.txt) calls A0M(t,null); [6ol.A06](root-time-millis.txt)
passes that string to 0HF.A01 with default0, then uses wrapping mul-long by1000.
[0HF.A01](lenient-long.txt) tests TextUtils.isEmpty: null/empty returns the supplied
default. Nonempty text uses Long.parseLong; its NumberFormatException catch
returns the default rather than constructing LIL. Thus malformed or overflowing
supported text returns0 too. Valid signed/BMP-decimal and i64-extreme integers
retain Java long parsing/multiplication behavior.

**Negative result:** the hypothesis that every malformed receipt t should get
error487 is refuted for this ordinary root path. It does not throw the native
parse exception on these numeric failures. This selected trace does not prove
all handlers, arbitrary binary representations or business/privacy time fields.

## Rust already has the appropriate boundary

The earlier [public fact-time policy](../../receipt-fact-time.md) was grounded in
2.26.37.71. The current build confirms the selected lenient root parser pattern.
This chunk changes no production admission, ACK, fact projection or wire policy.
Public facts intentionally retain None for absent/malformed/unrepresentable time,
so parser-default zero cannot masquerade as a supplied zero timestamp. A literal
valid zero remains Some(0). Native-style device-cache projection keeps its
existing default-zero/positive-time admission. Historical facts are not rewritten.
This is a documented projection distinction, not a claim that native default0 is
an authenticated observation of timestamp0.

`ordinary_root_time_fallback_never_becomes_aggregate_error_nack` now guards this
boundary after input codec roundtrip. Missing/empty/non-integral and positive/
negative overflow root times validate; no parse-error NACK is constructed; the
ordinary read ACK has no error. Recording matches the existing send, adds its
supported read fact with None time, and preserves intent/attempts/progress.
Literal0 and +Arabic22 controls preserve supplied0/22000ms. Reopening preserves
the fact; a duplicate adds no fact or journal entry. Existing aggregate tests
separately require487 for proven per-item failures.

**Unverified/separate:** full native storage/row admission, opaque binary/string
carrier equivalence, other handlers and time fields, clock authority/preference,
all request families and live acceptance. No new request, query, resend, identity
change or account operation. This does not establish human viewing or guarantee
that any account avoids restrictions.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/6oe;' --method A18 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6ol;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0HF;' --method A01 --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features ordinary_root_time_fallback
```

checksums.json covers every other file by byte length/SHA256. The complete
outer parser is linked instead of duplicated. No APK, private state or bulk tree.
