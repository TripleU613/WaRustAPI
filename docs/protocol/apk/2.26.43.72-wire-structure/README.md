# Selected wire-structure failures — APK2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX export; already diagnosed unavailable Android provider
uses supported fallback. Not REA Evidence or runtime observations. No live account
or target execution.

## Four selected native refusals precede stanza dispatch

[Complete123.A00](../2.26.43.72-truncated-node/list-size-reader.txt) accepts list
markers0/248/249 and throws LIL for other markers at002e..005e. An unknown VALUE
tag is a different context; this does not classify all Rust UnknownTag cases.

[Complete123.A01](../2.26.43.72-group-missing-identification/current-jid-attribute-reader.txt)
reads the description token, handles token2 first by returningnull, then obtains
its string through A0A. At002a..0032 zero listcount or null description branches
to0278..0290, the LIL nextTree sees 0 list or null tag. This tests null, not
nonnull empty text. Root token2 must retain precedence even when countzero.
A nested zero-count list followed by token2 also returnsnull from nativeA01;
its parent/null-child behavior is unported. Rust retains the earlier local
InvalidNode refusal for that nested case instead of relabeling it corrupt.
Nested nonzero token2 semantics remain separately untraced/unchanged too.

[Previously retained String constructor and A09 check](../2.26.43.72-null-attribute/README.md)
prove that token0 ordinary attribute VALUE cannot be stored as a valid0Tl.
The newly retained [complete00l.A0F](null-throw.txt) constructs a
NullPointerException at00aa..00b4, delegates throwable processing A0J and throws
at00ba. A09 also has a throw-null fallback. There is no selected normal return
for the null argument. This is distinct from rawfc00 nonnull empty text and
from token0 used as an empty body.

## Initial exceptions leave this reader

[Complete1s2.run](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
calls123.A08 at0058..005e; its002a..0034 code-unit catch range covers this call.
LIL enters088a (byte1114), cleans the reader-local1ru map and rethrows to the
outer LIL handler08d9 (byte11b2), then onError/return. RuntimeException enters
087f (byte10fe), logs and rethrows at1112; the outer all-handler0963 (byte12c6)
logs ReaderThread/error0, invokes onError and rethrows. Neither path resumes
node reads or reaches the inner recoverable parsed-stanza handler. Cleanup is
path-specific: do not infer that null-attribute runtime failure clears the same
map as LIL, or that either deletes durable work. [Concrete listener follow-through
and global manager gaps](../2.26.43.72-compression-failure/README.md) remain distinct.

## Rust scope

[Canonical implementation and regression limits](../../frame-admission.md#selected-wire-structure-failures-are-terminal-2026-11-06).
InvalidWireStructure/classinvalid_wire_structure isolates exactly the selected
list/null checks from local unsupported shapes and other generic codec errors.
The full InvalidNode/UnknownTag families are not made terminal. Empty-description
local refusal and the nested zero-count/token2 refusal retain their prior class;
this is not a claim that they fully match native handling. No reconnect policy,
NACK, account-ban determination or replay authority is inferred.

## Reproduce

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/00l;' --method A0F --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1s2;' --method run --signatures --exceptions
```

PCs are bytes; catch coordinates are code units (multiply2). Previously retained
complete methods remain linked rather than duplicated. checksums.json covers
every other file here.
