# Token SDK defaults and value-source boundary

**Confirmed static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
These are repository xtask DEX exports and bounded derivations, not REA Evidence
envelopes or runtime observations. REA inspect_android_method was unavailable
because its configured Java21 executable was missing; the existing static reader
was used without changing configuration or executing APK code.

## Two generated default paths agree

09n.A01 composes an index from specifier low16 bits and bits54..59 shifted left16,
then selects one of four switches by index remainder4. The retained complete
method contains2357 switch entries. The four previously derived token specifiers
select direct const-wide/return-wide cases:

| Property | Default index | Switch PC | Literal PC | Value |
| --- | ---: | --- | --- | ---: |
| 865 | 131173 | 005a | 0356 | 604800 |
| 909 | 131182 | 004e | 0510 | 4 |
| 996 | 131197 | 005a | 0356 | 604800 |
| 997 | 131198 | 004e | 0510 | 4 |

Thus selected SDK defaults are available in DEX, independently of binary
params_map assets. derive-sdk-defaults.py verifies index/branch anchors, all four
switch sizes, unique keys and each selected immediate return. It consumes the
previous pinned specifier derivation and emits sdk-default-proof.json.

The earlier selected legacy08Y.A0Q writes did not prove final map values. Reusing
the full constructor/builder proof with --policy token-buckets now establishes
these same four final values in this.A02. The model visits5551 puts,4158 integer
writes and3585 distinct integer keys, and each selected token key occurs exactly
once. legacy-final-proof.json retains source hashes and selected PCs. Bulk builder
exports remain outside Git and reproduce from the pinned APK. The original
client-ping output and fault checks remain unchanged by the optional policy flag.

## SDK read path is not just a default lookup

The interface-aware declared override scan covers114178 APK classes and finds
Arl bodies in09u,0A2,QAg,QAh plus the bodyless interface declaration. This is
static declaration ancestry, not runtime implementation selection. The earlier
0A8.Arl search had no body:0A8 is the specifier
lookup, not this SDK value-reader implementation. No absence claim about SDK
reading follows from that negative result.

09u.Arl obtains a default from09n.A01 and calls Ark. Ark checks unit/type bits,
returns default on incompatible paths, and dispatches to A07/A08-selected contexts.
0A2.Arl likewise obtains the generated default and calls its Ark. That method
first checks enabled int overrides, then silent-push status/value, then selected
value-source metadata. SERVER or DEFAULT__SERVER_RETURNED_NULL paths reach
A06(JJZ)J; other selected sources return the supplied default. A00's silent-push
status has readiness, freshness, nonnull value and specifier-type checks.
The exact branch instructions and exception scopes are retained whole.

0A2.A06 and A07 have no DEX bodies in these exports. Their abstract/native status
is not distinguished by this exporter, so their implementation is an explicit
boundary rather than an invented result. QAg delegates to its09t; QAh has
additional lookup/remapping paths retained in the override export. Complete
runtime instance selection, native bodies, silent-push hydration, override
lifecycle and universe readiness remain unproven. These defaults cannot simply
be applied as effective account values from an AB reply or packaged asset.

## Reproduction and impact

From repository root:

```sh
python3 docs/protocol/apk/2.26.43.72-token-sdk-defaults/derive-sdk-defaults.py > /tmp/token-sdk-proof.json
cmp /tmp/token-sdk-proof.json docs/protocol/apk/2.26.43.72-token-sdk-defaults/sdk-default-proof.json
python3 docs/protocol/apk/2.26.43.72-client-ping-final-defaults/derive-defaults.py --apk ../android-lab/apks/WhatsApp.1.apk --policy token-buckets > /tmp/token-legacy-proof.json
cmp /tmp/token-legacy-proof.json docs/protocol/apk/2.26.43.72-token-sdk-defaults/legacy-final-proof.json
```

DEX exports use apk-smali with --signatures --exceptions and exact class/method:
09n.A01,09u.Ark,0A2.Ark/A00/A06/A07. The override export uses
apk-virtual-overrides --class Lcom/facebook/mobileconfig/factory/MobileConfigUnsafeContext;
--method Arl --signature (LX/0Ek;J)J --include-interfaces (quote shell descriptors).

This narrows the [configuration gap](../2.26.43.72-token-bucket-configuration/README.md)
and [MobileConfig boundary](../2.26.43.72-token-mobileconfig-binding/README.md).
It does not change Rust scheduling/admission, implement effective configuration,
prove server acceptance or authorize live requests. No live account operation.
