# Message ACK chat selection — APK 2.26.43.72

**Confirmed static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
rehashed 2026-11-06. Complete selected DEX methods and declared getType override
census (superclass ancestry only, not runtime dispatch). xtask fallback, not
REA Evidence envelopes. No live account operations.

## Selected chat differs from original remote-device metadata

[Complete existing 1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt)
reads UserJid recipient before id/error/class routing. In the message branch,
0294..02ac prefers nonnull recipient, otherwise passes from through0Dn.A00.
The selected nonnull chat passes private1rH.A00 at02ba..02c2 and is used to
construct1dW(chat,id,true) at02ee..0300. Both success and error branches reuse
that key; success separately converts original participant and from to
DeviceJid at044a..0466 before1lZ.A10 at046a. A10 passes key plus separate
remote-device metadata to A0H and submits through1lW.CPm. Full downstream
callback ownership and generic A1D effects are not established by this bundle.

0Dn.A00 unwraps listed device types to DeviceJid.userJid; otherwise0Di.A00
returns only0DY instances. UserJid extends0DY. Private1rH.A00 only applies its
conditional1aa.A02 /0x9.A00 conversion when0Dn.A0O recognizes types26/27.
The declared override census ties those to0xC/Dv2, not plain PN/LID. The user
factory maps plain s.whatsapp.net to PhoneUserJid (type0), and lid through the
shared hosted.lid constructor branch to0Dx (type18). These types bypass that
special conversion. Device types17/19 include PN/LID forms and are in A0S.

0Dx constructor retains LID length/digit checks. Factory and parser retain
additional families and special singleton branches, which are not globally
ported here.0x9.A00 includes a special constant-to-constant mapping and an
A07 conversion, but its full feature policy is not implemented or generalized
to PN/LID. Missing/unparseable text and failed UserJid casts are described in
[optional typed accessors](../2.26.43.72-ack-address-accessors/README.md).

## Rust correction and limits

Ordinary outgoing ACK settlement previously correlated by exact from alone.
It could ignore a matching supported recipient, or settle a matching from even
when the selected recipient named another chat. It now prefers plain numeric
PN/LID recipients (1..15 ASCII digits as this adapter's supported subset) and
normalizes PN/LID device from to its user when no user recipient applies.
Known group/broadcast/newsletter recipients and device recipients cannot pass
the selected UserJid cast and therefore use from. Other parsed recipient
families/forms remain uncertain rather than selecting from speculatively.
This is not a claim that the APK imposes the adapter's numeric-PN length bound.
The existing exact saved-chat equality, unique wire-id lookup, InFlight state,
challenge/malformed gates and once-only fact rules remain required.

All address forms/aliases, special bot policy, interop/hosted user normalization,
generic callbacks, distribution-wait and comparison-CLI correlation remain
separate audits. No PN/LID identity merge, new request, replay or live claim.
The full status-user class was inspected while distinguishing renamed classes;
it is not the LID constructor. Requested historical LidUserJid/LidJid class
filters yielded no method bodies; factory plus actual class census establishes
the selected LID mapping instead.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/1rH;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0O --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0S --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Di;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method A10 --signatures --exceptions
target/debug/xtask apk-virtual-overrides "$APK" --class 'Lcom/whatsapp/infra/core/jid/Jid;' --method getType --signature '()I'
target/debug/xtask apk-smali "$APK" --class 'LX/0x9;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0De;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dx;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/UserJid;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0EG;' --signatures --exceptions
```

checksums.json covers every other file in this bundle. No account-restriction
outcome or actual server acceptance is proven by static construction.

## Plain LID constructor follow-up

[Constructor grammar/canonical-server correction](../../message-ack.md#plain-lid-text-uses-constructor-grammar-and-canonical-server-2026-11-06)
now applies this retained0Dx evidence to plain ACK text. Invalid plain LID
recipient becomes null/falls back; invalid from cannot establish chat. Valid
hosted.lid text uses the same lid-valued constructor. The earlier numeric-only
adapter limit is superseded for these confirmed cases; explicit device/agent,
opaque typed values and other-family normalization remain unported.
