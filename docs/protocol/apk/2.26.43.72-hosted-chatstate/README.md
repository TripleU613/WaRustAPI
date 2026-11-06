# Hosted chatstate projects the underlying user

**Confirmed selected static / implemented offline**, APK2.26.43.72
build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed during this trace. Complete focused DEX exports; no new REA Evidence
records or live account operation.

## Hosted domain and user identity are distinct

Reuse [0De.A02](../2.26.43.72-message-ack-chat/user-factory.txt), re-exported
byte-for-byte, and the [generic textual parser](../2.26.43.72-message-ack-chat/jid-parser.txt).
The user factory shares the hosted and s.whatsapp.net construction path to
PhoneUserJid, and hosted.lid/lid to0Dx. The new PhoneUserJid.getServer returns
s.whatsapp.net; retained [0Dx](../2.26.43.72-message-ack-chat/lid-user.txt)
returns lid. Thus a plain hosted alias returns the non-hosted user class.
This is not a new canonicalization rule for all protocol surfaces.

For device-qualified input, the parser selects Dv4 for hosted and Dv6 for
hosted.lid after creating the underlying user. New complete Dv4 extends
DeviceJid, reports type24/server hosted and throws09P unless the original
constructor device integer is99. Reuse complete
[Dv6/0EM](../2.26.43.72-hosted-lid-device99/README.md): Dv6 type25/server
hosted.lid similarly requires99 and delegates through0EM to DeviceJid's
constructor. The new base constructor stores its supplied UserJid in the parent
userJid field; the numeric byte store is displayed as unknown opcode0x5d and
is not used to infer extra agent/byte coercion rules here.

New0EB.A01 builds a hosted(.lid) textual device form when device==99, using
its selected lid predicate, then parses it through A03. Other devices preserve
the supplied user's getServer branch. Effective other caller/factory policies
remain outside this mapping; no new device registration is inferred.

Reuse [incoming0Dn.A00/A0S](../2.26.43.72-message-ack-chat/README.md): listed
types24/25 return DeviceJid.userJid. The
[current chatstate reader](../2.26.43.72-chatstate-routing/README.md) applies
this to both from and participant before the UserJid author cast. Therefore
valid hosted(.lid) device99 maps to ordinary PN/LID chat metadata, even though
its original raw address and hash formatter retain the hosted domain.

## Rust correction and limits

The red regression showed hosted remained the selected metadata domain rather
than its PN user. The incoming-only helper now projects nonempty hosted aliases
to Server::User and hosted.lid aliases to Server::Lid, clearing device encoding
in the returned metadata. Explicit agent/device representations require device99;
other device values return no supported observation/author. Plain model aliases
remain supported. Native constructor/type bindings justify this projection;
they do not authorize authenticated alias adoption or editing saved identities.

Original Nodes/attributes remain unchanged. Hash/reconciliation and outbound
primary-target restrictions remain as before: their domain distinctions must
not be collapsed by this receive-only projection. The earlier hosted device
hash audit remains valid for those separate APIs. No global parser, JID display,
device collection, token lookup or authenticated mapping is changed.

This adapter consumes the existing Jid model. The global text parser's zero
qualifier normalization can erase distinctions before entry; complete native
text grammar/admission and all factory/string-error semantics remain unported.
Interop/bot/legacy/special singleton families and downstream null-chat consumer
policy remain open. This is metadata projection, not a native trust validator.

Tests cover both hosted domains as plain and explicit device99 JIDs, their
valid text forms, group authors, unchanged original text/typed attributes,
and unsupported explicit device0/2. Existing PN/LID/first-child/target tests
remain required. No stored state, ACK, request, replay or live operation.

## Reproduce

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-hosted-chatstate-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/Dv4;' --signatures --exceptions > "$OUT/hosted-phone.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0EB;' --method A01 --signatures --exceptions > "$OUT/device-factory.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/PhoneUserJid;' --method getServer --signatures --exceptions > "$OUT/phone-user-server.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/DeviceJid;' --method '<init>' --signatures --exceptions > "$OUT/device-init.txt"
```

Compare all four files byte-for-byte. Shared user/LID/parser/reader exports stay
under their existing identities. checksums.json covers every retained file
except itself. No APK, private state, media or bulk tree is committed.

Verification:1542 workspace tests across nine suites including doc tests, strict
workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and repository hygiene passed.
The APK SHA256 and shared user-factory export were rechecked; all four new
exports reproduce exactly. Focused regression:
`cargo test --locked -p whatsapp-api --all-features hosted_chatstate_projects_underlying_user_without_changing_wire_identity`.
