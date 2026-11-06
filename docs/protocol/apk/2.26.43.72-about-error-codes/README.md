# About error-code preservation

Confirmed static, APK 2.26.43.72 code 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution, live request or REA Evidence envelope claim. This extends the
[About callback lifetime trace](../2.26.43.72-about-callback-lifetime/README.md).

1rA.A03's ordinary legacy 1ru error branch removes the callback, iterates direct
error children, skips missing code, calls Integer.parseInt and invokes A01(int)
for each parsed code. The previously retained JAj callback forwards those codes
to its UI callback. A malformed present code throws before subsequent children
are processed. The selected method's exception metadata does not catch that
NumberFormatException locally. Broader native exception/UI effects remain unknown.

Rust's generic compatibility error_code is u16 and picks one error, losing signed,
large and subsequent values for this producer. Typed About settlement now retains
optional AboutRefusal {codes: Vec<i32>, malformed_code: bool} in ActionOutcome's
about_refusal. It processes only direct error children, stops at the first invalid
present code and records no raw malformed text. The shared Java decimal parser plus
checked i32 conversion is used for supported text values. Nontext typed code
attributes are conservatively diagnosed malformed; full native attribute-type
conversion is not ported by this helper.

The field is scoped to saved about-set status/set IQ with one status child and
outer error. Outer results still ignore error extensions as proven previously.
Other request families and generic error_code/error_text remain unchanged. This
metadata is diagnostic, not a synthetic UI callback, retry instruction, cooldown
or new settings adoption. Legacy missing metadata remains unknown and is not
backfilled. The durable reply is retained by existing settlement. Status, reply,
sequence and metadata commit in one existing store transaction.

Existing actions plain/JSON exposes the new field. Tests cover signed limits,
Unicode digits, all ordered codes before overflow, ignored nested/unrelated
extensions, missing codes, duplicate settlement, store reopen and no reservation
replay. A second parser test checks result/request scope and matches all 13
executed JVM oracle cases. IntegerOracle.java is a synthetic JDK parser oracle,
not APK execution. Reproduce from repo root:

```sh
java docs/protocol/apk/2.26.43.72-about-error-codes/IntegerOracle.java > /tmp/about-integers.json
cmp /tmp/about-integers.json docs/protocol/apk/2.26.43.72-about-error-codes/integer-oracle.json
```

The exact Java runtime version is retained. Re-export 1rA.A03 with xtask apk-smali
APK --class 'LX/1rA;' --method A03 --signatures --exceptions; selected byte-PC
ranges and full-export hash are retained. Prior full legacy-IQ/JAj exports prove
callback binding and forwarding. Checksums cover every other file. Raw APK and
bulk exports stay local. Full UI failure behavior, server error prevalence and
other producers' integer/error grammars are not generalized. No ban guarantee.
