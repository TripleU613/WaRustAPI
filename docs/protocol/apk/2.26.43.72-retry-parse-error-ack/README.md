# Retry parsing-error ACK context — APK 2.26.43.72

**Confirmed static / bounded Rust correction.** APK code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete checked DEX exports retain signatures/exception tables. These local
exports are not REA Evidence envelopes. **Observed:** no runtime execution or
account traffic. **Inferred:** selected non-BX2 handler error policy is the
compatibility target for Rust's ordinary decoded receipts. **Unknown:** full
upstream handler registration, BX2, Unicode/non-text equivalence, malformed
carried-key branches, complete ACK recovery and server acceptance.

## Required-field exceptions and native read order

[Complete F10.A04 parser](../2.26.43.72-retry-receipt-keys/receipt-parser.txt)
reads required retry at000a, required registration at0016 and its binary bytes
at001e. Null/non-four-byte data branches to03b4, constructsLIL("invalid
registration node") and throws. The retained0Tp.A0G invokes optionalA0F and
throwsLIL when no matching child exists. It does not return an empty default.

Only after that doesF10 check v (default1), returningnull for non-1. On its
v1 path, after typed recipient selection, it reads count via0Tp.A05 (default1),
required t viaA07 and required id viaA0L. Existing retained
[count reader](../2.26.43.72-retry-parser-admission/count-present-reader.txt)
wrapsInteger.parseInt NumberFormatException asLIL; the
[long reader](../2.26.43.72-retry-message-correlation/long-value-reader.txt)
does the same forLong.parseLong. The
[required string reader](../2.26.43.72-retry-message-correlation/required-string-reader.txt)
throwsLIL only for null/missing, not empty. Negative/high counts and empty ids
must not be promoted from local admission limits into native parsing errors.

## Different context from an unhandled version

[14h.A00 and1rg.A04](../2.26.43.72-retry-ack-dispatch/README.md) routeLIL on the
non-BX2 path to1rg.A03(node,487); the allowlist explicitly contains receipt.
[1rg.A03](../2.26.43.72-retry-original-chat/parse-error-context-reader.txt)
allocates a freshEfa and sets class=root tag, required root id, error487,
loggable ID=-2, optional from, type and participant. It does **not** copy recipient
or edit. Efa's retained constructor initializes only its extra-attribute map;
reference fields therefore stay null. Efa.A01 stores a string-valued error
attribute in that map. 0Sf.A0S explicitly bypasses its receipt flag test for
typeretry, and the selected
[ACK writer/conversion trace](../2.26.43.72-retry-version-error-ack/README.md)
binds this context toF3e.A05, which serializes the nonnull fields/extra attrs.

By contrast, non-1 version returnsnull, causing2Cx.A00 to copy the original
context and add487, retaining recipient/edit. Registration errors precede version:
a bad registration plus v2 uses the fresh parse-error context, not the copied
unsupported-version context. Both error numbers are487;2Cx's490 is diagnostic.

## Implementation and test scope

Rust now classifies confirmed parser errors separately from unsupported versions.
It preserves the observed required-child/registration/version/count/t/id order.
Supported root id/from/participant are mirrored; parse-error ACKs omit recipient
and edit. Optional unsupported root grammars are still refused locally before
classification. Version default1 and count default1 remain intact.

Numeric error classification is deliberately limited to ASCII syntax/range
failures: Rust's i32/i64 parsers cover signs and boundaries there. Non-ASCII and
non-text values are not declared corrupt. Nested id is classified malformed only
when absent. Carried-key processing, native address/alias grammar and local
count/id restrictions remain separate incomplete policies. No session mutation,
key fetch, replay or success ACK is introduced. Existing callers retain their
capture/validation/store sequencing before invoking confirmation.

The table regression verifies first-error field and exact fresh ACK context for
missing children, registration absent/null/children/short/long, count invalid/
overflow, t absent/invalid/overflow and absent nested id. It includes bad
registration+v2 (parse error wins), present recipient/edit (both omitted), invalid
later fields (version refusal still wins), signed boundaries and local negative/
high-count/empty-id/carried-key/Unicode refusals. Error ACKs round-trip through
the existing comparison token codec; this is not native execution/live acceptance.
Older no-success-ACK tests now supply complete valid retry fields so that missing
required fields no longer masquerade as an unsupported capability case.

Reproduce new exports with `target/debug/xtask apk-smali <pinned-apk> --class
'LX/0Tp;' --method A0G --signatures --exceptions`, and the same command for
`LX/Efa;` methodsA01 and<init>. Linked prior bundles retain all other methods.
`checksums.json` records bytes/SHA256 for every other file here.
