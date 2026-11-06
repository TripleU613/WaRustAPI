# MEX request-owned envelope checks, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues the diagnosed unavailable Android provider. Complete selected methods retained. No APK execution/live traffic or REA Evidence envelope claim.

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/88J;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8B6;' --method A0J --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6of;' --method A1W --signatures --exceptions
```

From repository root. checksums.json binds every other file. Reuse [7Ou/7Ow constructors](../2.26.43.72-mex-result-cardinality/README.md) and [shared typed delegate](../2.26.43.72-prekey-lid-mex-iq-expiry/iq-delegate.txt). Only the identified String/Jid equality and selected envelope blocks are interpreted; numeric/binary helper branches remain outside this audit.

## Expected values are checks, not defaults

8B6.A0J receives expected value as parameter registerv24, copied to localv9; final Boolean isv26. In its String branch a true final Boolean uses null-safe equality at00c8..00d8. With false it requires a present value, applies supplied length bounds, then compares a non-null expected value at01a0..01ac. The Jid branch obtains a typed Jid through0Tp.A0A; true likewise performs null-safe equality at02b4..02c4. False requires a non-null parsed Jid and compares against a non-null expected Jid at02de..0302. Mismatch records a validation diagnostic and returns null. It does not substitute the expected value for a missing field.

## Common envelope and selected success constructor

88J.A00 takes reply, original request and validation accumulator. It requires reply root iq at0010..002a, reads the request id at002c..005e, then checks reply id against that expected String at005e..0082. It reads request to as a typed Jid at008a..00c0 and checks reply from against it at00c0..00ee;6of.A1W explicitly builds the from attribute path. These callers reject null validation results, so equality cannot turn both missing values into a valid admitted envelope. Root type is separately read into the returned BVX; this method does not alone establish a result/error whitelist.

7Ou(0Tp,0mK) performs an inline version of original-id and request-to/reply-from checks at0070..017c before its result payload/model selection and exact-one result cardinality. The request-error7Ow overload calls88J.A00, so a shared body parser does not mean request ownership is omitted for error models. Typed Jid equality uses the platform Jid object and decoder; complete normalization/subclass/alias semantics remain untraced. Do not replace this with guessed raw text comparison or assume an unrelated outer from identifies a trusted mapping.

## Rust boundary and port obligations

Rust mex::parse_json_response takes only a Node, selects result/update and parses JSON. It has no original-request argument and does not claim to authenticate the enclosing request or sender. Bounded direct-symbol search in crates at this revision finds its declaration and offline tests, not a live request callback using it; this is not a reflection/re-export/whole-program reachability proof. The earlier duplicate-result correction remains a payload shape check, not evidence of complete envelope admission.

Before using this parser to adopt data in a future live MEX flow: bind it to the actual sent request/connection callback, validate original ID and typed request-to/reply-from authority, preserve the supported model/error distinction and enforce expiry/restart/no-replay rules. Native live sender admission, Jid canonical equality, payload streaming updates, request-specific schema/configuration and authenticated mapping adoption remain incomplete. No new API, sender policy, live operation or Rust runtime behavior was introduced by this research.
