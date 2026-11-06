# Presence reader — APK2.26.43.72

**Confirmed static**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
reverified2026-11-06. Complete selected DEX exports retain signatures and
exception tables. These are xtask fallback exports, not REA Evidence envelopes;
the previously diagnosed unavailable Android provider was not reinstalled. No
APK execution, account operation or observed network traffic.

## Reader binding and missing type

1rA.A02 at00c6..00d8 maps the 1rF handler to presence. Its complete constructor
stores the supplied1lZ and resolves09F through provider0xed. The inherited
1rA.A03 selected1rF branch02ac..03e4 reads the node/from and has separate device
and permanent-group routes before its user conversion0Dn.A00.

For the selected nonnull user route it reads optional type/name/presence.
Type=unavailable at03a4..03c2 invokes6qD.A00 then1lZ.A0y. At03c4 the null-type
branch goes to03d8, exactly the A0x call used by explicit type=available. Other
explicit strings skip this call. Rust metadata previously reported missing type
as untyped; it now returns available for supported primary PN/LID users.
Missing-type device/group metadata stays untyped; those native routes are
separate. Explicit unsupported/nontext values remain
unsupported locally. No other routing or durable state policy is ported here.

## Numeric and clock boundaries

6qD.A00 reads last. Missing last returns currentTimeMillis; deny/error/none
return zero. Other strings call6oe.A03, which invokes Long.parseLong and
multiplies by1000 with native signed-long wrapping. Neither exported method has
a catch table. Full upstream exception policy and downstream UI/storage effects
are not established by these bodies.

Rust LastSeen still preserves raw absence, nonnegative seconds and textual
sentinels. Its current u64 parser is **not** native signed-long grammar or
millisecond/default-clock parity: negative/Unicode decimal and unsigned overflow
behave differently. This metadata projection is not used to authorize an ACK,
request or account state change. Signed conversion/API shape and full projection
remain open; no guessed timestamp/clock is installed by this correction.

## Other selected routes and negative findings

The selected device branch has a09F.BRq gate before A1A dispatch. The group branch
reads count, sends positive counts through A13, and otherwise handles unavailable
through A12. Complete0Dn classifiers,09F provider binding, group adoption and
consumer authority are not reconstructed. The Rust metadata parser's acceptance
of a syntactic sender is not equivalent to those native routing gates.

Anchor searches for presence/last/deny produced66 references.1ro.<clinit> was a
string table, not the numeric reader.1rJ was an ib handler (A02's adjacent
branch), not presence. These false leads did not establish native behavior; the
retained tag dispatch identifies1rF and the selected inherited reader. Other
branches of the complete1rA.A03 export are retained for context, not claimed
fully interpreted. No absence or completeness claim follows from string hits.

## Reproduction

Run from the repository, with APK outside it:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-xref "$APK" --anchor last --anchor deny --anchor presence
target/debug/xtask apk-smali "$APK" --class 'LX/1rA;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rA;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rF;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6qD;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6oe;' --method A03 --signatures --exceptions
```

checksums.json covers every other file. Existing presence regression covers
missing/available/unavailable/empty/unknown/case-different/nontext type and the
existing sender/last-seen cases, including bare-server exclusion and the
separate device/group boundary. No outgoing request, ACK, retry or live
acceptance guarantee is added.
