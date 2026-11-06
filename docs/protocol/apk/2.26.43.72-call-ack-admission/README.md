# Call-class ACK admission — APK 2.26.43.72

**Confirmed selected static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Existing complete1rH.A03 was re-exported and compared
byte-for-byte. New complete constructor/predicate exports use xtask fallback,
not REA Evidence envelopes or runtime observations. No account use.

[Complete1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt) first
requires id, parses error and diverts numeric459 before class routing. Its
class=call branch begins04aa. With a parsed nonnull from, it converts children,
then requires id nonnull/nonempty at0518..052c and type nonnull/nonempty at
052c..053c. These branches target05f8 (id) and0608 (type) and throw LIL.
DEX branch deltas are code units; displayed offsets are bytes. The checks use
String.length, not trimming or a recognized type whitelist.

EW4.<init> stores id/from/type/children/error and checks0Dn.A0R(from). False
throws IllegalArgumentException at0028..0036. The caller's catch range code
units028c..02aa (bytes0518..0554) translates IllegalArgumentException through
0618..062a into LIL. A0R admits types0,10,17,20,18,19,26,27,24,25.
[Existing declared type census](../2.26.43.72-message-ack-chat/jid-types.txt)
binds normal group1bc to1 and newsletter1cO to21; [parser/factory evidence](../2.26.43.72-message-ack-chat/jid-parser.txt)
selects these known sender classes. Those two sender families cannot pass the
shown call-ACK constructor. No full mapping of all Rust server variants is
claimed. The constructor performs no type-string whitelist check.

[Selected corruption handler and reader exit](../2.26.43.72-stanza-corruption-routing/README.md)
rethrows LIL for root ack and exits the reader with onError2; it does not send
an ACK-to-ACK487. Numeric459 bypasses these class-specific checks, although
missing id still fails the earlier required accessor. Empty id is permitted
by that earlier accessor and is rejected here only for ordinary call ACKs.

## Boundaries

Missing/null sender branches at04c2 to062a and throws IllegalStateException,
a different exception route not reinterpreted as this selected LIL path.
Children conversion precedes the id/type checks; its full payload/exception
semantics remain unported. Nontext id/type A0M cache coercion, all other sender
families, call-state updates and concrete connection-manager recovery remain
open. Rust only adds confirmed empty/missing text-field and known group/
newsletter checks to its shared captured-receive admission. Its non-reconnectable
local error remains conservative policy, not native recovery emulation.
This is no call feature, request authorization, retry or account-policy claim.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/1rH;' --method A03 --signatures --exceptions > /tmp/call-ack-parser.txt
cmp /tmp/call-ack-parser.txt docs/protocol/apk/2.26.43.72-distribution-ack-admission/native.txt
target/debug/xtask apk-smali "$APK" --class 'LX/EW4;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0R --signatures --exceptions
```

checksums.json covers every other file. Existing full parser/census/catch exports
are linked instead of duplicated. Static evidence cannot establish zero bans.

## Known-null sender ordering follow-up

[New selected Exception handler and continuation trace](../2.26.43.72-call-null-sender/README.md)
resolves the absent/empty-text sender path: it precedes call-field checks and
returns to reading through generic1rg.A00. Rust now skips only those later
class checks for known-null sender inputs; global required-id/numeric checks
remain earlier. Other parsing/provider/recovery gaps remain open.
