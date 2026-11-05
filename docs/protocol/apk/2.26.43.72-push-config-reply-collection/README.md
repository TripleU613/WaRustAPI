# Push-config reply collection — APK 2.26.43.72

**Confirmed static**, checked DEX TXT; not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected methods are retained and checksummed. Reproduce with
`xtask apk-smali APK --class CLASS --method METHOD --signatures --exceptions`.
The reply constructor is the exact7Ow(0Tp,1rt) overload selected from five
exports, also retained in the earlier lower-level IQ bundle.

## The bound applies after conversion

In7Ow(0Tp,1rt), after the selected header parser succeeds,016e calls
0Tp.A0N("config") and iterates the returned children. For each child the
constructor checks the config tag, then tries a six-member mixin group through
8B6.A0K:8TD,8TE,FVH,8TF,Hz6,8TG, in that order. Its diagnostic group name is
AndroidClientResponse|EnterpriseClient|AppleClient|FBClient|WebClient|WNSClient.
Those diagnostic names do not themselves prove each converter's full grammar.

At01fc a null conversion loops without adding a wrapper. A nonnull conversion
is cast to2Ke, wrapped in7Nk and appended to the parsed list. At0216 and0250,
6oe.A04 returns that list's size as a Long. The constructor compares that size
against minimum0 and maximum2, throws LIL on an out-of-range count, and only
then stores the list and original response node at0272..027a. These are actual
constructor throw paths, distinct from the corrective sender's diagnostic-only
attribute validators. Zero successfully converted entries pass this collection
bound, assuming the preceding header succeeds and no called helper throws.

Consequently this is not a maximum of two raw config children. The selected
collection code has no explicit duplicate-platform check; that bounded absence
does not establish all duplicate records pass the individual converters.

## First nonnull variant and null failure

8B6.A0K invokes A0H for each candidate and returns its first nonnull value.
If every candidate returns null, it collects available diagnostic strings,
sets a required-mixin-group diagnostic and returns null. It limits diagnostic
text by error count/length; those thresholds are not collection or wire limits.
A0H with the empty path supplied here directly invokes8oe.ABJ on the child.
Neither selected helper has an exception table: exceptions from a converter
are not caught here. Null failure may therefore skip a child, while a thrown
failure need not. Do not collapse these outcomes into unconditional tolerance
of every unknown or malformed config child.

## Port boundary

This closes the collection-count and variant-choice control flow only. Individual
Android/other variant fields, ABJ exception behavior, header helper details,
duplicate semantics, resource limits and complete durable reconciliation remain
open. The internal Bundle projection keys remain insufficient to define the
wire parser. No Rust sender/parser, replay authority, HTTP route or live account
operation is added. No claim about ban prevention follows from this static trace.
