# Special PN device ACK identities — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete0EE/Dv4 classes exported with xtask fallback, not REA Evidence envelopes
or execution. Existing parser/factory/constructor evidence is linked below.
No live account operation.

[0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt) at0196..021a
separates explicit device syntax using last dot/colon, rejects trailing
separators and gets its base UserJid through
[0De.A04](../2.26.43.72-call-sender-text/user-cache.txt).
That factory falls through to
[0De.A02](../2.26.43.72-message-ack-chat/user-factory.txt), whose PN/hosted
special cases return [DCF0 or Dv7Server](../2.26.43.72-special-pn-ack/README.md).
These are UserJid instances, not failed PhoneUserJid numeric constructors.
A04 checks existing cache hits with0Dn.A0k, otherwise rebuilds throughA02.
No universal cache/reflection completeness is inferred.

0Dd constructs0EE for PN and Dv4 for hosted; both constructors here accept
UserJid rather than specifically PhoneUserJid. Both invoke the same
[DeviceJid constructor](../2.26.43.72-lid-device-ack-text/device-constructor.txt),
which excludes status0EG but not DCF/Dv7, then checks narrowed signed byte0..99.
Dv4 additionally requires original integer99.0EE returns type17/serverPN;
Dv4 type24/serverhosted. Both retain the original integer. Dv4.getRawString
constructs a hosted device string; no Rust wire serialization port follows.

Consequently0:0 andServer:0 at PN are DeviceJid, despite the different plain
base types.0:256 andServer:-256 also pass byte narrowing;100/255 fail. Hosted
99 passes and355 fails. Dot-only suffix defaults device0 (PN passes, hosted
fails); dot suffix is ignored rather than parsed as numeric agent. Numeric
suffix uses Java signed-i32 grammar, including supported BMP decimal digits.

[0Dn.A0R](../2.26.43.72-call-ack-admission/sender-types.txt) admits types17/24,
unlike plain base types7/8. Thus these valid devices do not independently
reject call sender admission. A null parse still reaches the earlier recoverable
null-sender route; missing/empty call fields reject only after nonnull sender.
[Participant conversion](../2.26.43.72-ack-participant-success/device-conversion.txt)
returns DeviceJid unchanged, including a Server device whose plain base would
not convert. [Recipient accessor](../2.26.43.72-ack-address-accessors/typed-address.txt)
fails UserJid cast for explicit device0 too; selected from instead
[unwraps device.userJid](../2.26.43.72-message-ack-chat/unwrap-device.txt).
The base canonical server is PN, including hosted0/Server devices.

## Reproduce

Run from checkout root, APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0EE;' --signatures --exceptions > pn-device.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dv4;' --signatures --exceptions > hosted-device.txt
```

Binary JID equivalence, full cache behavior, service-chat application handling,
other native families and server acceptance remain unproven. This establishes
selected text construction/classification, not full APK parity or no-ban proof.
