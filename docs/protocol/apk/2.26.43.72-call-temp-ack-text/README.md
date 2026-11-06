# Call and temporary ACK JID text — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX class/method exports through xtask fallback, not REA
Evidence envelopes or execution. No account operations.

[0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt)011a..0124 directly
constructsDv5 for call;0158..0162 constructs7Kr for temp. Neither branch uses
PN/LID agent/device parsing. Dv5 requires18..32 ASCII hexadecimal characters,
accepting0..9,a..f,A..F. Its server/type are call/10.7Kr checks0Cf.A0z(user,"-",false)
and otherwise throws09P; the selected helper passes String '-' toA0O at offset0,
which with String input and ignoreCasefalse invokes String.indexOf. Thus any
hyphen anywhere suffices, including bare '-' and leading/trailing hyphens.
No numeric/length restriction is present in the selected7Kr constructor.
Its server/type are temp/2. Both retain original user strings.

[Optional parser](../2.26.43.72-ack-address-accessors/optional-parser.txt)
converts constructor failures to null, preserving known-null call sender
ordering before later call fields. [0Dn.A0R](../2.26.43.72-call-ack-admission/sender-types.txt)
admits type10 but excludes2: valid call JID alone does not reject a call ACK,
while valid temp sender does.459 diversion precedes this classification.
This does not implement call payload parsing or claim all call ACKs are valid.

Dv5 extends Jid directly, not0DY. The message-key normalization
[0Dn.A00](../2.26.43.72-message-ack-chat/unwrap-device.txt) delegates nondevices
to0Di.A00, retained here: instance-of0DY or null. Therefore even a validDv5
cannot establish message chat from. It also fails a UserJid recipient cast,
falling back to from.7Kr extends GroupJid; retained
[ancestry](../2.26.43.72-message-ack-chat/jid-types.txt) binds it through0nT to0DY.
It can establish selected chat from within Rust model limits but cannot override
from as UserJid recipient. Full user is preserved; dot/colon is not device syntax.

## Reproduce

From checkout root with pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dv5;' --signatures --exceptions > call-class.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/7Kr;' --signatures --exceptions > temp-class.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Cf;' --method A0z --signatures --exceptions > hyphen-contains.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Cf;' --method A0O --signatures --exceptions > string-index.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Di;' --method A00 --signatures --exceptions > chat-cast.txt
```

Native values outside existing Rust Jid limits retain uncertainty; binary typed
JIDs, cache/reflection completeness, bare aliases, generic receipt routes and
full recovery/payload parsing remain open. No live or zero-ban claim.
