# Receipt ACK type and namespace admission

2026-10-11 follow-up: [recipient class admission](receipt-user-recipient.md)
now additionally requires the supported UserJid class domain for recipient.
Recognized non-user/device recipients omit that ACK field; generic participant
admission stays separate.

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Incoming supported receipt ACKs now reuse the strict receipt-kind getter and the
namespace check shared with key-based aggregate targets. No new message type,
read policy, request, reservation, endpoint or account migration is introduced.

## Evidence and corrected behavior

**Confirmed static:** Cmk.A03 starts with `0hs.A0M("type", "delivery")`
at `000e..0014` in the [complete instruction trace](rea/2.26.37.71-receipt-list-shape/parser-exact.txt).
The complete [REA A0M getter](rea/2.26.37.71-receipt-ack-namespace/string-attribute.json)
returns its supplied default only when the attribute object is absent. A present
object uses its A03 value; it is not absence just because the local Rust value
has no text representation. The existing receipt-fact getter already made this
distinction. The ACK builder previously collapsed non-text type into missing
type and emitted a delivery ACK. It now shares the fact getter: absent type
selects delivery, recognized delivery/read/played are supported, and a present
non-text or unsupported type stays captured without an ACK.

**Confirmed static:** [optional JID access](rea/2.26.37.71-receipt-list-shape/optional-jid.json)
uses the [nullable factory](rea/2.26.37.71-receipt-list-shape/nullable-jid-factory.json).
Its exception wrapper returns null on failed parsing. The [complete factory
smali fallback](rea/2.26.37.71-receipt-list-shape/jid-parse.json) retains namespace
switches and the unknown-server throw; its Java decompilation failed and is not
used as an executable reconstruction. The [server-only parser](rea/2.26.37.71-receipt-list-shape/server-jid-parse.json)
recognizes exactly `s.whatsapp.net`, `g.us` and `call`, returning null otherwise.
In particular, bare `lid` and `broadcast` are not valid server-only JIDs.
The same complete factory rejects an `@` at index zero: text such as
`@s.whatsapp.net` now refuses instead of becoming a bare server address.

Rust's general Jid model intentionally preserves other domains and historical
`c.us`/`msgr` variants. That model alone is not the APK's network namespace
admission. The existing aggregate-target namespace mask is now shared with the
ACK builder and extended with the confirmed server-only distinction. Unknown
namespaces no longer become ACK destinations or optional participant/recipient
attributes. Missing/unparseable optional addresses still omit those attributes;
refusing every malformed optional address would contradict the nullable accessor.
Known PN/LID, group, status/broadcast, hosted and special namespace behavior is
retained; actual device and encoding values are not canonicalized or merged.

The [generic ACK renderer](apk/2.26.37.71-identity-notification-ack/ack-wire.txt)
conditionally emits to/type/participant/recipient. Its ability to render a null
from descriptor is separate: this client's supported receipt ACK path continues
to require a recognized original from. Complete nullable-root dispatch and
specialized runtime JID subtype/class predicates are not claimed as ported.

## Aggregate targets and recovery

Key-based participants entries use the same helper before time parsing. Bare
`lid`/`broadcast` targets now skip just like absent or unparseable optional JIDs,
so malformed time on such skipped entries cannot reject the supported entries.
Known namespace admission is not original-send authority: existing exact target,
chat, saved-attempt and immutable outcome guards still decide attribution.

The generic Jid parser remains unchanged for local storage and API models.
Unsupported root/type receipts remain durably captured. The actor sends no ACK
for those receipts and continues receiving; a subsequent ordinary receipt can
still be acknowledged. This does not queue a retry, manufacture a delivery fact,
adopt a key or change explicit/automatic read behavior. Existing receipt ACK
correlation, batch root selectors, uncertain outcomes and server waits are intact.

## Checks and limits

The new ACK regression failed before implementation; the empty-user cases also
failed before adding their shared boundary check. It covers absent versus
non-text type, unknown and legacy namespaces, invalid server-only forms, optional
field omission and recognized server-only/status/hosted/companion routes. The
actor test passes actual binary frames through the receive interface, verifies
raw capture and unchanged event-journal head, observes no unsupported ACK, then
acknowledges the next valid receipt without restarting. The existing binary
aggregate regression now includes bare lid/broadcast targets with malformed
time and still projects only the two originally authorized device facts.

Validation: all 913 workspace/all-feature tests passed, along with strict Clippy,
warnings-denied Rustdoc, default CLI/API-symbol and CLI-reference checks, CLI
build, formatting and repository evidence/link hygiene.

The shared helper checks namespaces and server-only forms, not every native
special JID grammar or subtype conversion. Complete native ACK dispatch,
read-self/played-self processing, interop/hosted client behavior and fresh live
receipt acceptance remain separate unfinished work. No new network request ran.
