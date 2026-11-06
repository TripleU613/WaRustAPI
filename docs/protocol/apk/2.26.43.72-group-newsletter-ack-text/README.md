# Group/newsletter ACK text admission — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX class/method exports through xtask fallback, not REA
Evidence envelopes or execution. No account operations.

[0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt)0138..0142 constructs
1bc directly with the whole user for g.us; newsletter0176..017c calls
[0De.A03](../2.26.43.72-call-sender-text/newsletter-cache.txt).
Neither branch routes through PN/LID agent/device suffix parsing.

1bc first invokes0nW.A02: no hyphen,1..20 characters, first ASCII1..9,
remaining ASCII0..9. Otherwise it finds the first hyphen and requires the
prefix to pass [0Dl.A02 PN grammar](../2.26.43.72-pn-ack-text/phone-grammar.txt)
and suffix exactly10 ASCII digits with first1..9. This is narrower than the
existing generic Rust group shape (which is not globally rewritten).0nW.A02's
hyphen-string call is preserved in full; any hyphen is incompatible with its
subsequent ASCII digit loop regardless of that preliminary helper's details.

**Corrected interpretation:** newsletter validation's exception message says
"valid number starting with non-zero", but the actual0De.A03 body does not
check digits or numeric length. It calls Java trim, rejects empty, exact0 and
indexOf("0")==0 on the trimmed string, then constructs1cO with the ORIGINAL
untrimmed user. Java trim removes characters <=U+0020, not Unicode whitespace
such as NBSP. Letters, minus sign, fullwidth digits, dots and colons are not
rejected by this selected predicate. The class constructor stores the original
user and adds no validation. Its server/type are newsletter/21.1bc is group/1.
Do not infer numeric grammar from an error message or constructor name.

The [optional accessor](../2.26.43.72-ack-address-accessors/optional-parser.txt)
turns construction failures into null. For call ACK, null sender precedes later
id/type checks and follows recoverable generic exception routing. Nonnull
types1/21 fail [0Dn.A0R](../2.26.43.72-call-ack-admission/sender-types.txt),
independently of whether Rust's generic Jid parser accepts the text. Numeric459
still diverts before call dispatch. Group/newsletter recipient values fail the
UserJid cast and cannot override from. Valid selected from keeps the whole user;
it does not normalize newsletter dot/colon to device/agent components.

Rust now uses these selected text predicates across call admission and message
ACK selection. For selected from it constructs a plain Jid with original user,
within existing model limits; colon/control/overlong user values that the model
cannot represent retain uncertainty. No bypass of the model validator or global
JID rewrite. Binary typed-JID equivalence, cache/reflection completeness, generic
receipt routes and full native reader/recovery behavior remain unproven.

## Reproduce

Run from checkout root, pinned APK outside Git, compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1bc;' --signatures --exceptions > group-class.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0nW;' --method A02 --signatures --exceptions > modern-group-predicate.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1cO;' --signatures --exceptions > newsletter-class.txt
```

The complete newsletter factory was already retained and is linked above.
Static admission is not evidence of server acceptance or absence of restrictions.
