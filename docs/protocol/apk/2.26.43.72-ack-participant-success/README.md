# Participant-qualified ordinary ACK success — APK 2.26.43.72

**Confirmed static selected methods**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete DEX methods/signatures/catches, xtask fallback;
not REA Evidence envelopes, APK execution or live account evidence.

[Existing complete ACK parser](../2.26.43.72-distribution-ack-admission/native.txt)
converts Eoh.A01 participant through0EB.A00 at044a..045a only on the ordinary
success branch, then passes that result as the first DeviceJid to1lZ.A10.
[Retained A10](../2.26.43.72-message-ack-chat/success-dispatch.txt) forwards it to
A0H. The new complete A0H sets Message what0/arg1=19/arg2=0 and writes that
first argument as participant using A0g. A0g uses0Dn.A09 and putString; it does
not invent a participant. Full serialization/helper exception semantics are
not reinterpreted here.

0EB.A00 returns a DeviceJid unchanged, or recognized UserJid.getPrimaryDevice;
other inputs return null.0Dn.A0k admits types0,7,11,18,22,26. The linked
[declared type census](../2.26.43.72-message-ack-chat/jid-types.txt) and user
factory bind plain PN/LID to types0/18. Plain users therefore convert too;
this is not solely a colon/device-suffix rule.

Exact log-string census locates one reference at196.BLn, classes.dex offset
0x8310c2. Its event19 branch parses participant at00e0..00fc as DeviceJid;
at0ad6..0aea a nonnull participant without the bot-feedback exception logs and
returns true before the ordinary success worker. Producer/consumer event19 and
field shape agree; the entire external dispatcher invocation chain has not
been retraced here. The earlier pinned37.71 full route already supports the
ordinary guard; this43.72 trace refreshes its selected producer/consumer sites.

## Explicit bot-feedback exception

At0ab2..0ad4 the participant user must pass0x9.A03 and Ep1.A00(chat,message)
for v15 to bypass the guard. Ep1.A00 requires a DwD message instance and checks
chat predicates.0x9.A03 recognizes0xC or A06; A06 can recognize PhoneUserJid
through A0A(user). The latter predicate and full DwD creation/policy are not
ported here. Do not infer that every possible PN participant can never be bot
feedback. Rust's ordinary supported-send adapter conservatively retains its
existing PN/LID guard and does not implement this exception.

## Rust scope

The existing ordinary outbox guard is now shared by expected-chat classification,
which also drives distribution waits and CLI comparison results. Supported
nonempty PN/LID participants on error-absent success cannot settle an ordinary
send or release a distribution. Error-present refusal/challenge classification
is separate and remains available. Arbitrary non-user/malformed participant
presence is not by itself evidence of a nonnull converted device.

Full special/bot/hosted/interop behavior, generic callbacks and native account
policy remain open. No new fatal ACK rule, error reply, recovery or replay.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-xref "$APK" --anchor 'MessageReceivedByServerHandler/onMessageReceivedByServer message received by server ignored due to participant is present'
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method A0H --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/196;' --method BLn --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0EB;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0k --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method A0g --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Ep1;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0x9;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0x9;' --method A06 --signatures --exceptions
```

checksums.json covers every other file in this bundle. Complete existing caller
and type census are linked instead of duplicated. No restriction guarantee.
