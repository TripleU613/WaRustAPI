# Ordinary text submission into the message store

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Nine-topic recipe](apk/recipes/2.26.37.71-text-store-submission.json) and
[complete new methods](apk/2.26.37.71-text-store-submission/targets.json)
continue [text construction](message-status.md#ordinary-user-input-text-chain).
Offsets are method-relative bytes unless the physical caller inventory says DEX.
This is source attribution, not a runtime send-state or user-row implementation.

## Actual construction caller and selected submission routes

The [exact B1l.A00 caller inventory](text-store-caller-evidence.txt) finds three
physical invokes: B1l.A01, DMk.run and 3XZ.invoke. This inventory does not prove
all three workflows are identical or that every text enters B1l.A01.
B1l.A01 invokes A00 at 0062..0068 and keeps the returned 1he as v2. Its later
metadata, preview and specialized helper calls remain partially uninterpreted;
retaining the full body does not establish their complete state effects.

Selected routes in A01 include 0zC.A0O(message,1) at 0586..0592,
Cmw.A04 at 0594..05b6 under 0DE.A0b(chat), and 0zC.A0K at 05ba..05c4.
The Cmw route and complete earlier branch conditions are unknown here. These
are alternatives, not three calls made for every message. No direct 178.A0H
appears in B1l.A01. Its return is not a proven database commit or transmission.

## Default reason, queue selector and deferred target

0zC.A0K chooses integer **22** when message.A0y is true, otherwise **-1**,
and passes that integer with the message into A0O. It neither sets native
message status nor inserts a receipt row. A0y producer semantics remain
separate from ordinary authored-text construction.

0zC.A0O first emits 0zN.A06 bookkeeping with message-ID hash, constant 4,
constant -1 and message type. It then constructs 3Pn(message, reason, **4**,
store) and submits it through 1f0.A01 with local scheduling code **26**.
The constants in this scheduling/telemetry call are not native receipt status
assignments or server rate limits. Complete 0zN and scheduler guarantees remain
unknown; no outgoing stanza is constructed in this selected method.

3Pn's constructor stores the third argument as discriminator $t, the second
as A00, the fourth/store as A01 and first/message as A02. Its switch maps
selector **4** to **0486**, which casts those fields and calls
0zC.A04(store,message,reason) at 049a..04a0. Other selectors are retained
but are not interpreted as part of text submission.

A04 calls 0zK.A07 metrics only for fromMe AND !A0y, then invokes
0zC.A09(message,reason) regardless of that metrics guard. The A09 integer
argument remains the original reason; no setter in A04 assigns it as status.
A04 checks result A02 for a selected chat update and result A03 to log
isWrittenToDb=false with the result error/key. Nonempty listener maps can
queue DNs selector 49 using scheduling code 25. Full 2jQ construction,
listener effects and transaction guarantees remain unknown.

**Confirmed negative:** A0K/A0O, 3Pn selector 4 and A04 contain no direct
message status setter and no receipt_user write. This closes their handoff;
it does not prove all A09 helpers preserve the initial zero or that the
queued worker succeeded. The previously retained
[A09 preprocessing](apk/2.26.37.71-text-status-initialization/settings-preprocessing.txt)
contains a conditional status 13 and other helper work, already attributed in
[message status](message-status.md#separate-workflows-not-universal-initial-state-evidence).
Full ordinary add/send transition and SQL/observer-to-network authority remain
unfinished. Do not invent status 2 at this queue boundary merely because the
call is asynchronous or because Rust uses Prepared/InFlight reservations.

## Setter candidates excluded from new text initialization

The earlier superclass setter inventory was followed into three scoped methods,
now retained so their contexts need not be rediscovered:

| Method | Confirmed context / limit |
| --- | --- |
| 11k.A09 | Parses payment_reminder child/attributes and uses PaymentReminderNotificationHandler diagnostics; not a new user-input text factory. Full payment semantics are outside this trace. |
| 1Qk.A02 | SendE2eStatusAckHandler lookup compares existing status with a supplied new status, conditionally sets positive server time, then calls core update with reason 24. An ACK application is not initial construction. |
| 1Tk.A02 | Unsent recovery with status/newsletter/media/location/album/call/system guards and app/unsent/autoretry diagnostics. Its selected setter/persistence branch is not a universal fresh-text state. Full retry eligibility/scheduling remains unknown and is not ported. |

Their existence grants no new retry/reset authority. Existing Rust reservations,
ambiguous outcomes, peer keys and server waits remain unchanged. No account,
new message ID, live send or receive window was created.

## Remaining prerequisites

**Unknown/unported:** all ordinary preparation helpers and accepted worker
results, complete send-state progression, read/settings and namespace admission,
actual native user-row membership and primary identity cleanup. Existing raw
DeliveryFacts and immutable device targets stay separate from receipt_user.
Existing APK CLI tools reproduce this evidence; no HTTP route or runtime/live
capability was added. Future releases need separately anchored snapshots before
cached diff; file equality is not live or semantic protocol equivalence.

Validation: nine topics / fourteen files / 116,232 bytes reproduce exactly
from the pinned APK. Cached self-diff verifies nine unchanged topics and four
metadata artifacts. CLI reference regeneration, OpenAPI/library-symbol checks,
formatting and diff hygiene pass. This source/docs-only chunk changes no Rust
runtime code and claims no fresh workspace/runtime suite or live validation.

Follow-up: [store insertion](text-store-insertion.md) traces A0A/A0E into
the type registry and hook order, with concrete inserter effects still unknown.
