# PN ACK text constructors — APK 2.26.43.72

Confirmed selected static, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX method exports via xtask fallback,
not REA Evidence envelopes or runtime observations. No live account operations.

PhoneUserJid.<init> invokes0Dl.A02 and throws09P on false. The helper requires
Java length5..20, rejects leading0, rejects prefix10 unless prefix1008, and
compares every UTF16 character to ASCII0..9. Because every accepted character
is ASCII, Rust byte length and Java UTF16 length agree for accepted inputs.
This is protocol user grammar; no registration/E.164 validation change follows.
PhoneUserJid.getServer returns s.whatsapp.net. The retained
[factory](../2.26.43.72-message-ack-chat/user-factory.txt) routes both hosted and
s.whatsapp.net to that constructor for ordinary users.

The factory separately handles0 (DCF/type7) and Server (Dv7/type8); the
[parser](../2.26.43.72-message-ack-chat/jid-parser.txt) handles plain
gdpr@s.whatsapp.net asJ9O/type9. Types come from the retained
[type census](../2.26.43.72-message-ack-chat/jid-types.txt). These are not
PhoneUserJid constructor failures. Their conversion and ACK selection are
unported; the shared text classifier leaves them unclassified and preserves
existing adapter handling. No universal null classification is claimed.

For explicit devices the same parser constructs0EE for PN, Dv4 for hosted.
0EE invokes DeviceJid's constructor and stores original user/int. Dv4 invokes
that constructor and additionally requires original integer99. The shared
[base constructor](../2.26.43.72-lid-device-ack-text/device-constructor.txt)
narrows int to signed byte before0..99 admission. Last-dot/last-colon parsing,
dot-suffix omission and Java Integer.parseInt apply as documented in the
[LID device bundle](../2.26.43.72-lid-device-ack-text/README.md).
Explicit device0 still constructs DeviceJid, so the optional UserJid recipient
accessor returns null; selected from unwraps to canonical base user instead.

These exports establish constructor admission and canonical base server, not
full device serialization, binary JID equivalence, cache/reflection completeness,
special-user routing, server acceptance or absence of restrictions.

## Reproduce

Run from the checkout root with the pinned APK outside Git:

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/infra/core/jid/PhoneUserJid;' --method '<init>' --signatures --exceptions > phone-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/infra/core/jid/PhoneUserJid;' --method getServer --signatures --exceptions > phone-server.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Dl;' --method A02 --signatures --exceptions > phone-grammar.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0EE;' --method '<init>' --signatures --exceptions > phone-device.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dv4;' --method '<init>' --signatures --exceptions > hosted-device.txt
```

Compare output byte-for-byte with these exports. Existing parser, factory,
base constructor and type census are linked instead of duplicated.
