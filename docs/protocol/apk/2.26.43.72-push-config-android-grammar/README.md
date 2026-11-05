# Android push-config response grammar — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not observed traffic or REA Evidence
envelopes. APK SHA256 reverified2026-11-05:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce: `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`. Complete selected methods are retained and checksummed.8RM.ABJ
is retained in full but only selector9 is interpreted here; its constructor
stores the supplied discriminator and its switch explicitly maps9 to00cc.
Shared8B6.A0J is retained in the [token reply bundle](../2.26.43.72-client-auth-token-reply/attribute-validation.txt).
All limits below describe this selected parser, not general server constraints.

## Wire fields and value mapping

8TD.ABJ is the first candidate in the previously traced [config variant choice](../2.26.43.72-push-config-reply-collection/README.md).
It checks tag config, parses optional item children, checks the config tag again,
and reads the following attributes through8B6.A0L/A0J.6of.A1U yields the id
path;6oj.A0p yields1,3q4.A0d yields0,6oi.A0s yields9007199254740991.
7NQ's exact constructor connects each argument to the projected fields.

| Wire input | Selected parser behavior | Retained value |
| --- | --- | --- |
| platform | One of adm, fbns, gcm; null result returns null variant |7NQ.A04 |
| id | String length1..1024; null result returns null variant |7NQ.A02, provider token |
| voip_payload_type | One of strings0,1,2; null is not a variant-return gate |7NQ.A05 |
| app_mute | Long0..9007199254740991; null is not a variant-return gate |7NQ.A00 |
| pkey | String length1..64; null is not a variant-return gate |7NQ.A03 |
| num_acc | Long0..5; null is not a variant-return gate |7NQ.A01 |
| appid | Enum473039703209605,614665046147756,994766073959253; result discarded | Not stored in7NQ |
| settings | Long0..9007199254740991; result discarded | Not stored in7NQ |

String length uses Java String.length (UTF-16 code units), not UTF-8 byte count.
A0J's ordinary String/Long branches return null on missing, conversion or bound
failure and can set diagnostics. A0L returns null when its parsed String is not
in the supplied list. In8TD, only platform and id nulls abort this variant after
the tag checks. The other results can be null or discarded while construction
continues. Thus a diagnostic is not itself a blanket variant-rejection flag.
The id length restriction is a real null-return gate here, unlike the separately
traced diagnostic-only corrective writer validators. No trim, default, clamp,
platform-specific appid pairing or account-count minimum1 is inferred.
Exact lexical behavior of Long conversion0Cd.A08 remains outside this bundle.

The constructed7NQ and nullable7NL item are wrapped in7OZ. The previously
traced visitor projects token by recognized platform, app_mute only in the gcm
branch, and nullable hash/count/key/VOIP values through the shared continuation.
[Visitor and consumer evidence](../2.26.43.72-push-config-projection/README.md).
These are wire names now connected to that projection, rather than names guessed
from internal Bundle keys. This does not establish raw token provenance/freshness.

## Item/hash parsing counts successful conversions

8TD constructs8RM selector9 and invokesA0G with path item. A0G callsA0M with
minimum0, maximum1, returns the first converted item when the resulting list is
nonnull/nonempty, otherwise null. A0M iterates all named item children, skips
null conversions, and checks the converted list's size; out-of-range returns
null with diagnostics. It does not throw directly in its selected bound path.
Neither helper has a catch table, so a converter exception may still propagate.

8RM selector9 checks tag item and reads hash as String length1..256 throughA0J;
null yields no item. Success constructs7NL with discriminator12, retaining hash
inA01 and the source node. Thus zero valid items, all-null item conversions or
more than one valid item can yield null7NL, and8TD does not reject the overall
Android variant merely because that optional item result is null. This is not
proof every malformed item is tolerated: called-helper exceptions remain distinct.
No hash algorithm, encoding, muted-chat ordering or hash-generation policy is
established by this attribute parser alone.

## Implementation boundary

This closes selected Android wire names, value mapping, null gates, enum choices,
and item/hash parsing. Other five variants, complete IQ header validation,
Long lexical grammar, duplicate semantics, full config lifecycle, source authority,
generation binding and durable reconciliation remain incomplete. Rejection by
8TD can still allow a later candidate to accept the same node; the whole reply
must not be declared invalid solely from this converter's null result. No Rust
parser/sender, replay authority, HTTP route or live account operation is added.
