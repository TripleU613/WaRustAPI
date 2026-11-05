# Push-config alternate variant admission and Long syntax — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce: `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`. All selected methods are complete and checksummed. Only the
admission/null gates and numeric algorithm below are interpreted; this is not
full reconstruction of each platform's configuration semantics.

## Alternate variants contribute to the collection count

The [reply constructor](../2.26.43.72-push-config-reply-collection/README.md)
tries8TD,8TE,FVH,8TF,Hz6,8TG in that order. Each successful conversion becomes
a wrapper in the list whose converted-entry bound is0..2. The [Android converter](../2.26.43.72-push-config-android-grammar/README.md)
is only the first candidate. The other five return these concrete types:

| Converter / type | Selected null-return gates after config tag check |
| --- | --- |
|8TE /7OU | id String length1..1024; platform equal to ent |
|FVH /7OY | platform in apple/apple_dev/apple_inhouse/smbi/smbi_dev; preview enum; VOIP payload enum; default/groups/call Strings length1..200; lg/lc Strings length1..10; item-list helper result nonnull |
|8TF /7OV | appid in the retained43-member list; deviceid String length16..36; platform equal to fb |
|Hz6 /7OX | endpoint/auth/p256dh Strings length1..1024; platform equal to web |
|8TG /7OW | id String length1..255; platform equal to wns |

This table follows explicit branches on returned values, not merely attribute
names. Other helper invocations may set diagnostics or throw; absence from this
table does not establish an attribute has no effects. FVH's item helper uses
8RM selector8 and bounds0..Long.MAX_VALUE on converted entries; that selector's
full item grammar is not interpreted here. Its numerous additional attribute
parse results, including id, can be discarded without an explicit null-return
gate. The full export is retained so later work can trace those differences.

8TF's fbid result is discarded. Hz6's lg/lc results are discarded.8TG parses
version against three uwp strings and parses pkey/settings, but none of those
results is a null-return gate in this selected body. Thus there is no basis for
requiring WNS version solely because an enum list appears in its converter.
Expected literal platforms use8B6.A0J with the expected String argument and
compare mode false: the ordinary String branch still checks a nonnull expected
value for equality.6oj.A0v sets the path element and returns lower bound
-9007199254740991, not zero; it must not be named or treated as a zero helper.
Shared A0J is [retained here](../2.26.43.72-client-auth-token-reply/attribute-validation.txt).

Previously retained7OU/7OV/7OW/7OX/7OY visitors only check nonnull visitor and
return, without updating the eight Android projection holders. That does not
remove these parsed entries from the reply list's cardinality check. A future
parser that counts only Android records could therefore accept a reply that
this selected constructor rejects. This is a reconstruction requirement, not a
confirmed defect in an existing Rust push-config parser: none is enabled here.

## Long syntax is separate from per-field numeric bounds

8B6.A0J's ordinary Long conversion calls0Cd.A08. That method delegates toA09
with radix10. A09 returns null for an empty string; permits a leading ASCII
plus or minus only with following characters; processes Java UTF-16 chars via
Character.digit(char,10); and rejects a negative digit result. No trim, whitespace
skip, decimal-point or exponent branch exists in this selected algorithm.

The accumulator remains negative. Positive input uses limit-Long.MAX_VALUE;
negative input uses Long.MIN_VALUE. Before multiplication/subtraction it checks
the threshold and remaining range, returning null on overflow; success returns
the negative accumulator or its negation according to the sign. Leading zeros
are not independently rejected. A leading minus does not itself fail lexical
parsing; the caller's field lower bound can still reject a negative value.
Signed zero consequently reaches numeric0. A0J then applies its selected bounds
and nullable failure semantics before8TD constructs the Android value.

Character.digit delegates character acceptance to the Android Java runtime.
This trace does not supply a complete Unicode digit table or authorize replacing
it with ASCII-only Rust parsing while claiming identical syntax. Conversely,
static helper syntax is not evidence the server sends non-ASCII numeric fields.
No runtime or generated fixture equivalence test was performed in this chunk.

## Remaining work

Exact non-Android optional-field semantics, Apple item grammar, Unicode runtime
digit tables, full IQ header validation, duplicates, request provenance, complete
provider/reconciliation lifecycle and durable generation ownership remain open.
No Rust behavior, parser, sender, retry authority or live account operation is
changed. The complete network correctness objective remains unfinished.
