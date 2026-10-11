# Own-device echo correlation to a normal outgoing intent

2026-10-11. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Implemented/offline:** authenticated supported own-device text/media whose
wire ID and chat match an existing normal outgoing intent is retained and
confirmed without publishing a second NewMessage or altering that intent.
This extends [text](own-device-text.md), [media](own-device-media.md), and
[sender confirmations](own-device-confirmation.md).

## Native decision path

The [release recipe](apk/recipes/2.26.37.71-own-device-echo.json) retains
[DEX/register/exception evidence](apk/2.26.37.71-own-device-echo/targets.json)
and maps it to receive, outbox and chat-association code. Offsets below are
method-relative **bytes**, not the code-unit labels in REA's smali rendering.

**Confirmed static:** CmS.A00 builds and postprocesses an ordinary message,
then invokes 1gz.A00. That dispatcher consults CPF.A00 and its DcU processor
set before its ordinary CoreMessageStore path. CPF returns the first result
other than DEJ, converts a handler exception to DEM, and returns DEJ when none
handles the message. The [complete class-reference census](rea/2.26.37.71-own-device-echo/processor-class-references.json)
contains CPF, six DD* classes and O5m. References do not establish registry
membership/order or runtime selection; specialized processor effects are unported.

0zC.A08 calls A09 with reason **22** when message A0y is true, otherwise -1.
A09 has substantial preprocessing/postprocessing beyond the common A0A call;
those effects are retained, not all reconstructed. Within A0A:

- The fromMe/A0y branches establish the incoming/own-device path separately
  from a local outgoing insertion. The direct route reaches A0E at `0448`.
- `34T.A03` is tested at `04a2..04aa`. Its constructor and
  [toString labels](rea/2.26.37.71-own-device-echo/result-labels.json) identify
  that field as **isDuplicate**, independently of wasSuccess/A06.
- The duplicate path fetches the existing message by its 1gc key at
  `04c2..04ca`. Expiration preprocessing is a separate guarded branch.
- Existing type 31, type 11, type 98 and selected futureproof type-12 values
  have specific replacement/recovery paths. An ordinary existing type-zero
  row follows `053c -> 0820 -> 0866 -> 0900`, skipping those replacements.
- The resulting ordinary duplicate path uses DGq, delivery handling through
  0zH.A0P, and a handler notification. It does not directly rewrite the
  existing ordinary body in that selected path. Full asynchronous effects,
  status semantics and optional ephemeral replacement remain unknown/unported.

The [complete A0A smali envelope](rea/2.26.37.71-own-device-echo/duplicate-selection.json)
avoids treating its failed Java reconstruction as authoritative. A0E's retained
SQLiteConstraintException path logs `addmsg duplicate` at `0cd0`, selects
C0l.A04 at `0d10`, and returns the separately labeled write result. SQL failure
must not be confused with a successful new insert. The existing
[main-row key trace](text-main-update.md) identifies chat/fromMe/ID addressing.
These observations do not establish that a peer-device echo is a server ACK.

## Rust transaction and API behavior

The existing held-session authentication, own PN/LID authority, wrapper
destination/content validation and commit-time session/pin/account checks still
apply. After projecting supported content, the receive transaction performs the
existing paginated, unique outbox wire-ID lookup. It correlates only ordinary
Text/Media/Poll intents whose original primary chat matches the actual recipient,
using exact address identity or an existing authenticated PN/LID association.
Contact/directory aliases and raw metadata do not provide that association.
This is the existing local association guard, not full native migration policy.

A match records `Inbox.kind=outgoing_echo` and the original local_id. The
original received-own checkpoint still holds the authenticated wrapper/route and
source, and the advanced Signal session commits with that marker. No NewMessage
or SendStatusChanged event is appended for that echo. The caller's existing
intent and ticket remain authoritative. New own-device messages, unrelated
chats and unreserved wire IDs retain their normal outgoing NewMessage event.
Malformed or ambiguous saved outbox state refuses and rolls back the receive
transaction; it is never repaired by guessing.

The echo does not modify the original body, attempts, progress, delivery facts,
idempotency key, wire ID, server waits or retry reservations. In particular,
InFlight/DeliveryUnknown stays uncertain until a separately supported ACK or
receipt establishes a fact. Receipt construction still uses the committed
same-source checkpoint and emits the ordinary `sender` confirmation. Existing
media selectors and clearing apply to retained echo content without automatic
download. No new command, configuration option or HTTP route is added.

## Tests and remaining work

A real Signal fixture seeds a normal in-flight text intent with DeliveryUnknown,
authenticates wrapped own-device text under its saved wire ID, verifies no new
journal event, checks byte-equivalent outbox/idempotency records and the local
correlation marker, confirms delivery, reopens and deduplicates, clears all own
rows, and continues the ratchet. The synchronized body deliberately differs
from the intent: correlation preserves the original rather than overwriting it.

Independent encrypted-store cases cover exact PN/LID, both authenticated
namespace directions, untrusted directory aliases, unrelated chats and absent
wire reservations. Corrupt outbox JSON rolls back a staged session write,
projection marker and journal. Existing outbox lookup tests cover pagination
and ambiguous wire IDs.

Cross-own-source reconciliation, legacy lab sends, placeholder/futureproof
replacement, native ephemeral replacement and asynchronous status effects,
alternate-recipient migration, new own-device admission and full native retry/
read-self policy remain unported. No fresh live echo was observed, and this
does not prove complete APK parity or ban immunity.

Local validation passed 945 workspace/all-feature tests, strict workspace Clippy
and warnings-denied Rustdoc, the actual Signal regression with only TCP/Noise/
Signal enabled, 26 default CLI tests and the experimental CLI build. The pinned
APK recipe reproduced all 14 files (545,978 bytes). No live account was changed.
