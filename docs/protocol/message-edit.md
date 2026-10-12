# Own-device ordinary text edits

Pinned APK: WhatsApp **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

Authenticated held own-device sessions now receive ordinary text edits as the
existing `MessageUpdated` / `MessageChange::Edited` event. They target a loaded
normal synchronized own text message or an attributed outgoing text intent.
The event retains its original chat namespace, outgoing direction and ID.
It does not replace stored parent bytes, journal history or send-status facts.

## Native evidence

The [seven-topic recipe](apk/recipes/2.26.37.71-message-edit.json) reproduces
[12 files, 159,857 bytes](apk/2.26.37.71-message-edit/targets.json), including full
method signatures, register flow and exception tables. Complete REA Evidence
records independently retain the [decoder](rea/2.26.37.71-message-edit/parser.json),
[parent mapper](rea/2.26.37.71-message-edit/parent-key.json),
[metadata constructor](rea/2.26.37.71-message-edit/metadata.json),
[ordering](rea/2.26.37.71-message-edit/ordering.json),
[descriptor](rea/2.26.37.71-message-edit/protocol-descriptor.json) and
[text kind](rea/2.26.37.71-message-edit/text-kind.json).
These are static APK observations, not an execution of the Android client.

`D9B.CF4` selects protocol `MESSAGE_EDIT=14`. `BZb.dynamicMethod` assigns
`editedMessage` field 14 and signed int64 `timestampMs` field 15. Replacement
content is decoded through the normal factory with edit mode 1; `C1he` text
and `C1iB` media are accepted without the poll/event feature gates.
`C1he`'s ordinary constructor sets native message kind zero.

`Ayo.A0S` reads only the embedded parent ID. Chat and ownership come from the
authenticated envelope; embedded `remoteJid`, `fromMe` and `participant` do not
select another parent. `D9B.CF4` inherits outer Message context only when inner
Message context is absent. A present empty inner context wins. Raw singular
submessage bodies concatenate in wire order to preserve protobuf merge behavior.
The original authenticated wrapper remains intact in its sealed checkpoint.

`D5e` stores stanza time and sender edit time separately. An absent wire sender
timestamp becomes native proto2 zero. `152.A03` rejects strictly older sender
edit time when current edited metadata exists; equal time is permitted.
`152.A00` at `02b8..02ee` rejects a revoked/terminal `1ii` parent. At
`033e..034e`, authenticated fromMe takes the branch to `045e`, bypassing the
peer window. Peers instead compare signed stanza-minus-parent time with the
effective integer property **2983** times 1000 at `042c..045a`. Its effective
runtime value remains unresolved; this implementation does not invent 15 minutes.
At `0b76..0bd6`, ordinary replacement and original kinds must agree.

## Receive behavior

The existing account/source/recipient/held-pin/session guards authenticate and
bind the wrapper before projection. Local outbox attribution takes priority;
otherwise the normal own-parent selector preserves its original PN/LID chat.
A directory alias alone cannot bind chats. Missing, ambiguous or cleared parent
content is not fabricated or fetched. Text replacements cannot edit poll/media
parents through this path.

Ordering and terminal checks use only previously committed `message_updated`
own controls with decrypted wrappers, valid saved own-device source and an
authenticated matching chat. A stale edit or edit after a received own revoke
commits its ratchet/capture and an opaque `UnsupportedMessage`, preserving the
original accepted event. Equal-time edits remain accepted. These history scans
are paged; a dedicated index is still needed for large histories.

The sender timestamp remains in the sealed original checkpoint, while the
existing event's `server_timestamp` carries stanza time. No public event or API
schema is added. Existing listen/event consumers can handle `Edited`. Atomic
publication, sender confirmation, reopen deduplication and own-message clearing
use existing paths. A forged edited redelivery of saved ordinary text still
refuses. Outbox facts, IDs, idempotency records, identities, pins and waits are
not rewritten; only the authenticated receive session advances.

## Verification and remaining work

The encrypted Signal regression failed at unsupported receive before the change.
It now covers accepted, strictly older, equal-time and newer edits, terminal
revoke/edit ordering, envelope ownership despite conflicting embedded keys,
stanza times outside the peer window, sender confirmation, reopen deduplication,
clearing, continued ratchet reception and unchanged uncertain sends/idempotency.
A separate encrypted-store matrix covers local text/poll intents, original PN
namespace via authenticated LID mapping, directory-only aliases, foreign chats
and unloaded parents without writing any table or journal. Literal decoder
checks cover merged replacement bodies, inner context presence, outer fallback,
default and negative sender timestamps, unknown/unsupported enum values,
mixed content, malformed lengths and unsupported media replacement.

Peer/group/status edits, media/caption and gated poll/event edits, future-proof
edited wrappers, orphan storage, expiry application and full native validation
remain unfinished. Repeated content fields still meet the existing normal text
projection gate; merged replacement fragments with one content field are covered.
Replacement text with an unresolved expiration declines projection under the
existing guard. Fresh live edit interoperability and a
second APK release comparison remain unverified. No live account operation ran
for this change; these results do not establish ban immunity.

Validation passed 966 workspace/all-feature tests, strict Clippy and
warnings-denied Rustdoc, the real Signal regression with TCP/Noise/Signal
features, 26 default CLI/API-symbol tests and the experimental CLI build.
The seven-topic APK bundle reproduced exactly. The reduced-feature build retains
three existing unused warnings in app-state test helpers and outgoing media;
strict all-feature checks pass without warnings.
