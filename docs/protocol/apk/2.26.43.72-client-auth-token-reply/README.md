# Client-auth-token reply validation — APK2.26.43.72

**Confirmed static**, checked DEX TXT, not REA Evidence envelopes or runtime
observations. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
reverified2026-11-04. Commands: `xtask apk-smali APK --class CLASS --method
METHOD --signatures --exceptions`. Cgu/Cgr exports contain their constructor
and ABJ dispatcher; only Cgu selector12 and Cgr selector5 are interpreted here.
BVZ is filtered to exact constructor `(LX/0Tp;[BI)V` from15 overloads.

## Request-correlated result header precedes token parsing

FUT first calls6oe.A1M ->0Tp.A01(iq), which explicitly throws LIL for a
wrong/null root tag. This is separate from the outbound logging-only assert.

The retained [FUT.CK4](../2.26.43.72-challenged-request-ownership/token-success.txt)
nonzero-selector branch constructs Cgu(originalRequestNode,EqZ,12). Cgu's
constructor stores that first argument in A01. ABJ selector12 (0138..0264)
requires the reply tag iq via8B6.A0N. It reads the original request's id and
to fields, then requires reply id equal to that ID, parsed reply from equal
to the request destination JID, and reply type=result. The path factories
6of.A1W/A1Y explicitly mean from/type. Missing/mismatched required fields
return null through the schema helpers rather than authorizing token storage.

8B6.A0J's String/Jid expected-value branches use09W.A00/Object.equals; this
is parsed-JID equality for from, not arbitrary PN/LID alias inference or proof
of every native JID parser mapping. The request writer's destination is the
server JID as retained in the [writer trace](../2.26.43.72-client-auth-token-writer/README.md).
No echoed namespace requirement appears in these selected converters/wrapper;
that bounded absence is not a statement about every upstream router or schema.

B1w.A17 passes an empty traversal path to8B6.A0H, which calls the chosen
converter on the same root node. The first header conversion must return
nonnull before FUT attempts its separate token-value conversion. Parsing the
cat value alone is therefore insufficient to reproduce this callback's checks.

## cat is bounded opaque byte content

Cgr.ABJ selector5 (0302..0370) requires an iq root, then uses path
cat/#elementValue with byte[] type and bounds1..1024.6oj.A0p supplies the
lower1; the upper is literal0x400.8B6.A0I delegates to A0J; its byte-array
branch reads the selected child node's A01 byte payload. Missing bytes or
lengths outside the bounds return null with a diagnostic. With minimum1, the
empty-value fallback in A0I does not authorize a missing/empty token.

Unlike the outbound writer's diagnostic String validator, these reader paths
actually return null on invalid input. This is not a Base64-decoding parser:
it retains opaque wire bytes. BVZ(selector0)'s constructor places the byte[]
argument in A00, which FUT later Base64-encodes with flag3 into
pref_client_auth_token. No token value is exported or generated here.

6of.A0i calls0Tp.A0F, which returns the first child with the requested name.
The selected parser does not count or reject duplicate cat children. That is
a bounded parser fact, not a recommendation to emit ambiguous replies or a
claim the service sends them. Unknown extra-child handling outside this path
and full transport/parser canonicalization remain separate.

## Invalid-header and invalid-value outcomes differ

FUT throws the schema error when its header conversion returns null. After a
valid header, a null token-value conversion instead reaches the return branch
without storing a new token, invoking rotation or selecting held requests.
B1w.A17/8B6.A0H do not manufacture a fallback token. The selected callback
does not clear an older stored token in that branch; that is not proof it
remains usable or that replay is authorized.

With a nonnull token wrapper, the previously retained order remains: save the
Base64 preference, call19i.A00, then select/dispatch held requests. Calling
rotation is not proof of completed key replacement. Complete refusal/backoff
callbacks, conditional input authenticity/eligibility, per-account/generation
binding and restart reconciliation still need tracing before a Rust sender or
recovery controller is implemented.

## Port boundary

Reply grammar/converters are now connected for this selected callback; native
JID parsing and all request-entry/recovery conditions are not complete. A future
port needs the original reserved request for ID/destination binding, opaque
byte length checks and distinct parse-failure outcomes, not merely an IQ ID
and an arbitrary cat child. Do not apply this family's from/type contract to
every unrelated IQ parser without its own trace.

This evidence-only chunk adds no request, token, automatic challenge retry,
account reset or live account/network operation. Rust uncertainty holds remain.

## Checksums

- [attribute-validation.txt](attribute-validation.txt): `42989269b621c6c4fcc3c4aec860fd5cf05bcd3d8832e069c0e2bc2d708acd86`
- [child-path.txt](child-path.txt): `00d7411ef9fe930830b5b7e3230375cda7867a818426a9c646e42daac37a70e9`
- [child-selection.txt](child-selection.txt): `d0d1534f2ccbf7046482d845157b888f83e86e8cb2bc779074725b95f3bfecb6`
- [converter-wrapper.txt](converter-wrapper.txt): `a75bc8b0cb71bcdb3b35487892bb5d12488b3b221af72772ebe95182484bc7d6`
- [equality.txt](equality.txt): `8be8f552e79d63c23f38a38309a950088bb57d78f9fa186e558b5a62024b48bf`
- [from-path.txt](from-path.txt): `cee93223d3065581e322515c00aeafd24b1c307388d273e5f59f1b0326276cc9`
- [min-length.txt](min-length.txt): `2328f15eab9797dcc4296afac4c2268336e55acb6bae97f8b70a956f4f901448`
- [primary-converter-all.txt](primary-converter-all.txt): `d3c5d2bcecbaae34286fd8653eaa3eefc3b5e94a3ba57b95bbc46bec6c7ee3c8`
- [schema-dispatch.txt](schema-dispatch.txt): `298ba4c69ea2aad57d689812fbb8eef82a5575b62778d776e3a0864bfa39dee9`
- [tag-validation.txt](tag-validation.txt): `a849bb008b49bb291e8275afa105f4dffe7fbd9f92479199a450cdf65ebf20e3`
- [type-path.txt](type-path.txt): `8446751f6e9f827c7edd7cb82506f27d827e80826e8f5b1329cce60e53fd0d35`
- [value-binding.txt](value-binding.txt): `c632b9a5deb6f7c4104b9baad607841c9ff8b0f4871b89f6b5ddbbc9e8288c80`
- [value-converter-all.txt](value-converter-all.txt): `1a44982c4032b2a099b6395a67819be1bfa88ec5edada4f4ad517fa1e46baccd`
- [value-validation.txt](value-validation.txt): `c4adce48153d30ff2f05eafaeb5b70e8e427f626180d7c876b0f05958b17f336`
- [entry-validation.txt](entry-validation.txt): `5232ae9a8eb8a33ba69cfc8d3ef7dd09f0929d319d34ab34739f8e0ed35a85c6`
- [root-required.txt](root-required.txt): `3ea77a197282fd89d6c05b2961e5a488a8c710e42bac618b1c9adebc9ce521e5`
