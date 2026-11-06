# Null attribute marker is not empty text — APK 2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence/runtime observations.
No live account operations.

## Reader-to-constructor boundary

The [complete123.A01 node reader](../2.26.43.72-group-missing-identification/current-jid-attribute-reader.txt)
at008e..00a0 reads an ordinary attribute value withA0A and invokes
0Tl(String name,String value). [CompleteA0A](../2.26.43.72-packed-nibble/string-reader.txt)
at0090..0098 returnsnull for token0 (LIST_EMPTY). Its rawbyte string branch
constructs aString, including nonnull empty text for rawzero-length bytes.
These wire representations are distinct.

Retained0Tl's String constructor invokes00l.A09(value,1). That helper tests
nonnull; ifnull it constructs a diagnostic, callsA0F and has an explicit
throw-null fallback. The delegated four-field constructor checks the same
String value nonnull again. Thus the selected native node cannot construct a
normal attribute from token0 through this path. Empty-string input passes
these checks; no minimum-length condition occurs in those constructors.
The typed-JID constructor also requires a nonnull JID, but its full cast/
protocol-compliance branches are not newly ported by this change.

Retained0Tp.A0M looks up the attribute and returns its stored text, or the
supplied default only when the attribute itself is missing. It does not make
token0 into empty text. This rules out the earlier normalization hypothesis;
exact higher-level native exception/corruption routing remains separate.

## Confirmed Rust mismatch corrected

Decoder.read_node previously mappedValue::Absent in an attribute value to
AttributeValue::Text(empty), making a null marker appear present with empty
text. That could reach required-id/optional-field admission as if the peer had
explicitly encoded an empty value. The decoder now returnsInvalidNode with
reason `null attribute value` at the value tag offset. It does not omit the
attribute, synthesize empty text or pass the node to ordinary ACK settlement.
Existing decode-error/capture paths remain in force; no new retry or NACK is
introduced and no native reconnect/error-code parity is claimed.

Explicit rawempty textfc00 still decodes to Text(empty). LIST_EMPTY content
outside an attribute value retains the existing Empty body behavior. This is
a context-specific correction, not a global ban on token0 or empty strings.
Attribute names already require a string inRust; other native tag/name/duplicate
rules and resource limits remain separate audit boundaries.

## Regression and limits

Literal synthetic web-dictionary fixture `f8 03 19 08 00` (iq/id/null) must
refuse; before correction it decoded successfully and the new test failed.
Control `f8 03 19 08 fc 00` keeps present emptyid, and `f8 02 19 00` keeps an
empty body. These are uncompressed node payloads, without transport flag byte,
used directly byDecoder. No server rejection, native runtime or ban outcome
is inferred. Decoder applies this boundary across attribute names/families;
it does not implement each ACK/notification handler's semantic rules.

Existing saved nodes/intents are not migrated or rewritten. Previously decoded
null markers cannot be reconstructed from stored empty text without original
capture evidence; no speculative migration or live-account action is performed.
Complete native exception routing, node-constructor constraints and remaining
outbound/inbound families remain open.

## Reproduce

From checkout root with pinned APK outside Git; compare complete outputs:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Tl;' --method '<init>' --signatures --exceptions > constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Tp;' --method A0M --signatures --exceptions > accessor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/00l;' --method A09 --signatures --exceptions > nonnull-check.txt
```

Linked complete A01/A0A exports retain their own recipes/checksums. PCs are
bytes; deltas code units. No full parser/lifecycle/server acceptance claim.
