# Special PN ACK identities — APK 2.26.43.72

**Confirmed selected static**, build264307238, pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected class/method exports using xtask fallback; not REA Evidence
envelopes or runtime observations. No account operations.

[Factory0De.A02](../2.26.43.72-message-ack-chat/user-factory.txt) returns
DCF for0 and Dv7 forServer on s.whatsapp.net/hosted. The complete classes
here establish UserJid inheritance, canonical s.whatsapp.net server, literal
user and types7/8. [Parser0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt)
008a..009e separately returnsJ9O for plain gdpr@s.whatsapp.net. Its class
extends0DY rather than UserJid; user gdpr/server s.whatsapp.net/type9.

The [call sender predicate](../2.26.43.72-call-ack-admission/sender-types.txt)
admits0,10,17,20,18,19,26,27,24,25, excluding7/8/9. All three plain special
identities are nonnull, so valid call id/type reaches EW4's rejection/wrapped
LIL. Global error459 diversion still precedes this. They are not invalid
PhoneUserJid text that should take the recoverable null-sender branch.

[Participant conversion](../2.26.43.72-ack-participant-success/device-conversion.txt)
first preserves DeviceJid, otherwise checks
[0Dn.A0k](../2.26.43.72-ack-participant-success/user-types.txt), admitting7
but excluding8/9. Thus plain Server and gdpr produce null participant device.
DCF invokes UserJid.getPrimaryDevice ->0EB.A01(user,0), which formats user,
agent,device and native server through0EB.A03. The shared DeviceJid constructor
rejects the status singleton0EG rather than DCF; explicit special-user device
classification/serialization is not added by this change. Existing null-on-
constructor-failure behavior remains separate from these plain identities.

[Typed accessor](../2.26.43.72-ack-address-accessors/typed-address.txt) requests
UserJid for recipient. J9O fails that cast and therefore cannot override from.
DCF and Dv7 are UserJid recipients; a different selected identity must not fall
back to an otherwise matching from. Server's full outgoing application handling
is unported; Rust retains uncertainty for that selected special chat. Hosted
plain0/Server use the canonical PN server. Binary/cache-wide equivalence and
all other special/native types remain unproven.

## Reproduce

From checkout root, pinned APK outside Git:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/DCF;' --signatures --exceptions > zero-user.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dv7;' --signatures --exceptions > server-user.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/J9O;' --signatures --exceptions > gdpr-jid.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/infra/core/jid/UserJid;' --method getPrimaryDevice --signatures --exceptions > primary-device.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0EB;' --method A01 --signatures --exceptions > device-factory.txt
```

Compare each output byte-for-byte; previously retained parser/predicates/accessors
are linked above. Static behavior is not server acceptance or zero-ban proof.
