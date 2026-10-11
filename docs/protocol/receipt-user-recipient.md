# Receipt recipient class admission

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
ACK rendering and direct/group fact routing now share a typed recipient getter.
This follows [nullable optional addresses](receipt-optional-addresses.md).

## Native boundary and runtime correction

The retained [CmG.A03 wrapper](apk/2.26.37.71-direct-receipts/receipt-wrapper.txt)
requests **UserJid.class** for recipient at 0010..0024. Its
[route builder](apk/2.26.37.71-direct-receipts/receipt-route-builder.txt) instead
requests generic **Jid.class** for from and participant. The
[complete typed accessor](rea/2.26.37.71-receipt-user-recipient/typed-accessor.json)
uses a cached JID or nullable text factory, then Class.cast; ClassCastException
returns null. Its special DeviceJid conversion only applies when requesting
DeviceJid. The [independent UserJid cast](rea/2.26.37.71-receipt-user-recipient/user-cast.json)
also returns null on a failed instanceof test.

The [complete getType census](rea/2.26.37.71-receipt-user-recipient/jid-type-ancestry.txt)
retains 27 declarations and superclass chains across 101,453 physical DEX
classes, distinguishing UserJid from DeviceJid/GroupJid and other subclasses.
The [exact user factory](rea/2.26.37.71-receipt-user-recipient/user-factory-exact.txt)
supports the selected plain PN/LID/hosted/bot/interop namespaces and symbolic
aliases. [REA's full factory result](rea/2.26.37.71-receipt-user-recipient/user-factory.json)
has switch-reconstruction warnings: use exact instructions for its shared
hosted/PN branch rather than the misleading Java rendering.

The [retained generic factory](rea/2.26.37.71-receipt-list-shape/jid-parse.json)
constructs group/broadcast/newsletter/call/temp/interop-group and special gdpr
objects separately. Their relevant ancestry excludes UserJid. Device construction
produces DeviceJid subclasses, even for explicit text `:0`; that suffix cannot
erase the class distinction. The local model's text parser loses explicit zero
device syntax, so the recipient getter checks it before parsing.

Previously network_jid accepted recognized non-user/device recipients, copied
them into ACKs and treated them as qualifiers that hid matching ordinary facts.
The shared recipient getter now selects supported plain user namespaces,
rejects device encodings/text suffixes and excludes the gdpr special object.
Non-user recipients follow the nullable path: ACKs omit them and fact processing
retains its ordinary target checks. Generic participant admission stays separate.

Valid user recipients retain the qualified-route refusal. Original saved chat,
wire ID, authenticated mapping and addressed-device guards remain effective.
No intent, attempt, progress, uncertain outcome, receipt reservation or strict
explicit-receipt ACK correlation changes. Existing API/CLI receipt outcomes
and outbox status expose the corrected observations; there is no new endpoint,
flag, schema or request.

## Validation and update workflow

The new ACK regression failed on a group recipient before implementation.
Binary tests cover non-user/device omission, generic participant preservation,
valid user qualifiers and an explicit zero-device AD encoding. Direct PN/LID
tests cover delivery/read/played facts, original attempts/progress, duplicates
and reopen. Group tests use original device targets; the actor receives binary
frames and ACKs a receipt with the non-user recipient omitted.

```sh
target/debug/xtask apk-virtual-overrides ~/Downloads/WhatsApp.apk \
  --class 'Lcom/whatsapp/infra/core/jid/Jid;' --method getType --signature '()I'
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk \
  --class 'LX/0D2;' --method A02 --signatures --exceptions
```

The APK hash was rechecked. Review/remap symbols before exporting another
release; old obfuscated names alone are not cross-release authority. Full special
grammar, symbolic server-only aliases, native binary subtype construction,
self-status state processing and complete ACK dispatch remain unfinished.
No live request, account-key operation or server-wait change ran. Static class
agreement does not establish effective privacy settings or ban immunity.

Validation: **924 workspace/all-feature tests** passed, plus default CLI/API-symbol
checks. Strict workspace Clippy, warnings-denied Rustdoc, CLI-reference/OpenAPI
coverage and formatting passed. Three complete REA envelopes match the pinned
subject hash; 27 exact ancestry chains were checked. These checks are offline,
not evidence of fresh live receipt acceptance.
