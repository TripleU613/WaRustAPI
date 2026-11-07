# Receipt recipient class admission

2026-10-11 correction: [the actual Java AD decoder](ad-jid-decoding.md) omits a
zero device suffix before constructing the JID. Binary AD-zero user recipients
are now admitted; literal text `:0` and nonzero devices retain the null path.
Wire encoding kind alone is not the native class. Earlier 924-test validation
below is historical; the follow-up records the corrected checks.

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
rejects nonzero devices/unsupported encodings and explicit text device suffixes,
and excludes the gdpr special object. Zero-component binary AD user JIDs are allowed.
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


## Current text-class admission, including dot qualifiers (2026-11-06)

**Confirmed local defect / fixed offline:** the recipient getter rejected colon
syntax but admitted `user.0@lid` and PN equivalents after generic parsing erased
the qualifier. It also admitted dotted nonnumeric suffixes and constructor-invalid
plain users. That copied a non-UserJid recipient into receipt ACKs and suppressed
otherwise attributable ordinary facts. The ACK regression failed with an echoed
recipient; the outbox-fact regression failed with0 updates instead of1.

The getter now reuses messaging::ack_user_text for raw PN/LID/hosted/hosted.lid
class admission. User/ServiceUser results continue through existing namespace
and supported-route checks; Device, NonUser and Null results take the optional
null path. Other families retain their prior supported subset and colon guard;
dots are not generalized to untraced namespaces. No new parser is duplicated,
no alias/address normalization is added, and typed binary-zero handling remains.

**Reused confirmed static evidence**, pinned APK43.72 SHA256887a6f94:

- [F0R.A03](apk/2.26.43.72-retry-ack-dispatch/receipt-context-factory.txt)
  requests UserJid.class for recipient at0010..0024, sets the Efa field then
  freezes it. This is the current-build receipt descriptor, separate from the
  incoming ACK reader and the older CmG fact-dispatch trace.
- [0Tp.A0A](apk/2.26.43.72-group-missing-identification/ack-typed-jid-read.txt)
  uses nullable parsing and Class.cast; failed conversion returns null.
- [0Dd.A00](apk/2.26.43.72-message-ack-chat/jid-parser.txt)
  tests last dot/colon presence at0196..01c4 before numeric value. Both .0 and
  an ignored dot suffix still select DeviceJid construction, not UserJid.
  [Declared ancestry](apk/2.26.43.72-group-missing-identification/current-device-overrides.txt)
  separates0EE/0EM from UserJid.
- [PN constructor/grammar](apk/2.26.43.72-pn-ack-text/README.md) and
  [LID constructor](apk/2.26.43.72-message-ack-chat/lid-user.txt) reject invalid
  plain users; [hosted factory/constructors](apk/2.26.43.72-hosted-chatstate/README.md)
  share the base user grammar and add original device99 constraints.
  Special PN UserJid singletons remain admitted by the shared classifier.

These are reused retained static exports, not new REA Evidence IDs or observed
traffic. Full current receipt-fact dispatch is not re-traced by this correction.

**Regressions:** actual binary encode/decode ACK cases omit dot-qualified
recipient while retaining the separate generic participant field. Additional
recipient-only cases cover invalid plain users and hosted .0 forms. The direct
fact matrix covers PN/LID original chats, delivery/read/played, dot0/ignored-dot,
invalid user and hosted qualifiers. It retains the original actor, intent,
attempts, progress and uncertainty, appends only the matching independent fact,
and proves duplicates across reopen add no fact or journal event. Existing valid
plain-user and binary zero-device controls remain.

**Checked unchanged:** record_ack does not call this getter and retains strict
raw recipient/edit exclusion, account-bound reservation and immutable callback
membership checks. No pending receipt is settled through this new nullable path.
Retry context admission does use the getter; unsupported recipient context stays
captured rather than being assigned a guessed parse-error ACK. Retry-answer
recipient syntax already refuses dot/colon. Generic participants, global JID
parsing, saved identities, aliases, read policy and replay state are unchanged.

**Unknown:** every special namespace/class, cached typed values with lost text
provenance, complete native error/ACK ordering and live acceptance. This does not
claim complete receipt parity. No new request, retry or account operation.

Verification: both ACK and fact regressions failed before correction and pass
with it. All1544 workspace/all-feature tests, strict all-target/all-feature
Clippy, formatting, OpenAPI library symbols, regenerated CLI reference, APK
coverage and repository hygiene pass. Hygiene checked8031 local links and451
APK checksum bundles. These are offline results, not live acceptance evidence.


## Hosted user output and binary recipient precedence (2026-11-06)

**Confirmed static / implemented offline:** a plain hosted recipient is not a
native UserJid retaining that server string. The shared
[0De user factory](apk/2.26.43.72-message-ack-chat/user-factory.txt) returns
PhoneUserJid for hosted/s.whatsapp.net and0Dx for hosted.lid/lid.
[PhoneUserJid.getServer](apk/2.26.43.72-hosted-chatstate/phone-user-server.txt)
returns s.whatsapp.net; [0Dx.getServer](apk/2.26.43.72-message-ack-chat/lid-user.txt)
returns lid. [F0R.A03](apk/2.26.43.72-retry-ack-dispatch/receipt-context-factory.txt)
selects the UserJid and the [F3e renderer](apk/2.26.43.72-prekey-upload-replies/prekey-ack-wire-builder.txt)
passes it through [DBe/0Tl](apk/2.26.43.72-group-missing-identification/ack-attribute-constructors.txt),
which retains the object and getRawString rather than the original alias text.

**Binary binding:** [123.A01](apk/2.26.43.72-group-missing-identification/current-jid-attribute-reader.txt)
branches on JID_PAIR0xfa at008a to00aa; reads user/server, joins with A04
and calls0Dd.A00 at00dc. The same export reads AD via A02 at00f6 then
calls the factory at0102. [A02](apk/2.26.43.72-group-missing-identification/current-ad-string-reader.txt)
and [A04](apk/2.26.43.72-group-missing-identification/current-ad-string-join.txt)
omit device0 before parsing. Thus ordinary zero-device binary PN/LID yields
UserJid; it is distinct from explicit text :0/.0 DeviceJid syntax.

Rust's recipient getter now projects supported zero-component hosted/hosted.lid
user attributes to PN/LID after existing class/namespace guards. The ACK
regression previously checked only field existence; strengthened exact-value
checks failed with hosted instead of s.whatsapp.net. It now checks exact fields
after real text and typed JID_PAIR encode/decode, retaining ordinary PN/LID,
bot/interop and binary-zero controls. Device99/explicit-zero class guards remain.
This supersedes only this getter's prior unprojected hosted output; no global
JID decoder, stored identity or alias-trust normalization is introduced.

**Message-ACK consequence / confirmed local defect:** outgoing message ACK
correlation already projected raw hosted text but not binary hosted pairs, and
it treated binary AD0 recipients as devices. Current
[1rH.A03](apk/2.26.43.72-distribution-ack-admission/native.txt) instead prefers
the parsed UserJid recipient before from at0294..02ac. A failed regression showed
an unrelated binary-zero recipient being ignored and matching from incorrectly
settling the message. A separate failed run showed matching hosted-pair recipient
failing to settle. The known zero-component binary PN/LID/hosted recipient branch
now uses the same user projection, preserving recipient precedence. Other typed
families, nonzero devices and from grammar retain their previous subset.

A new outbox regression covers PN/LID, primary pairs, hosted pairs and binary
AD0, matching versus mismatching recipient with opposite from, and success versus
error500. Mismatches preserve the original entry and journal head; matches append
one original server-acceptance/failure fact. Duplicates add nothing; after reopen
the same entry persists and no original intent is eligible to resend. No account
operation, new query, identity adoption or callback replay was performed.

**Boundaries:** receipt fact routing still treats a recognized user recipient as
a qualified route; this projection does not add a new fact target or merge PN/LID.
Strict outbound-receipt callback settlement is unchanged. Full current fact
processing, special typed families, long typed PN recipients, other from forms,
malformed cached objects and live ACK acceptance remain open. Static class and
wire evidence is reused; no new REA Evidence ID is invented.

Verification: the exact hosted-field regression failed before projection; the
new outbox test separately failed on missed hosted-pair settlement and false
binary-zero fallback settlement. All1545 workspace/all-feature tests pass.
Strict all-target/all-feature Clippy and the focused outbox regression pass
after test-only unused path-qualification cleanup. Formatting, OpenAPI library
symbols, regenerated CLI reference, APK coverage and repository hygiene pass.
Hygiene checked8044 local links and451 APK checksum bundles. Offline verification
does not establish live server ACK acceptance or full native parity.


## Long typed PN follow-up (2026-11-06)

The previously listed long typed PN recipient gap is now closed for the
[existing native-validated longer-PN correlation branch](message-ack.md#long-phone-user-recipient-admission-is-independent-of-carrier-2026-11-06).
A native-valid20-digit user no longer requires a text carrier to correlate:
primary/hosted JID_PAIR and binary AD0 use the projected user grammar too.
21-digit, leading-zero and prohibited10-prefix cases remain unsettled. LID's
bound and special/short-typed/decode-layer limitations remain separate.
No receipt getter, global alias/store normalization or live operation changed.
