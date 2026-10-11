# Reaction keys and historical target stability

2026-10-11 follow-up: [native reaction text limit](reaction-text.md) supersedes
the old local byte/control limits below. Bal.A01 own envelopes also reach the
common BoQ call after setting the actor null; the earlier non-own-only wording
was incomplete. Own-device reaction projection and full native lifecycle remain
unported; historical validation below does not cover this follow-up.

## APK evidence, 2026-10-09

**Confirmed static:** comparison APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [hash-pinned recipe](apk/recipes/2.26.37.71-reaction-key.json) retains
21 topics / 26 methods with register operands and exception tables in the
[bounded bundle](apk/2.26.37.71-reaction-key/targets.json). The exact
[BWI.key_ field inventory](reaction-key-field-evidence.txt) found 16 references
across BNx, Bal, DGD and DHR; this is DEX inventory, not Java virtual-dispatch
closure or live coverage.

- [BWI.dynamicMethod](apk/2.26.37.71-reaction-key/reaction-schema.txt) retains
  key=1, text=2, groupingKey=3, senderTimestampMs=4;
  [BZa.dynamicMethod](apk/2.26.37.71-reaction-key/key-schema.txt) retains
  remoteJid=1, fromMe=2, id=3, participant=4.
- [C7G.A00](apk/2.26.37.71-reaction-key/key-validator.txt), 0004..004e,
  requires the ID presence bit 4, nonempty ID, remoteJid presence bit 1,
  nonempty remoteJid and a nonnull ChatJid parse. It does **not** require a
  fromMe presence bit. [1xf.A0k](apk/2.26.37.71-reaction-key/chat-jid-parse-adapter.txt)
  delegates to [0D7.A02](apk/2.26.37.71-reaction-key/chat-jid-nullable-parser.txt),
  whose nullable parse converts exceptions into null. Its
  [A01](apk/2.26.37.71-reaction-key/chat-jid-required-parser.txt) requires
  the parsed JID to be the 0Cy subtype. [0D1.A00](apk/2.26.37.71-reaction-key/jid-parser.txt)
  dispatches recognized namespaces and throws on unknown ones; its native
  factory/cache/subtype and special server-only behavior are not fully ported.
- [Bal.A01](apk/2.26.37.71-reaction-key/reaction-incoming-builder.txt)
  validates an ordinary reaction through C7G at 01b2..01ca, constructs its
  model with A00 at 01fa..0200 and, for the non-own envelope with a chat,
  resolves its target through BoQ.A00 at 0206..0256. Other encrypted-reaction
  and edit branches remain outside this implementation.
  [Bal.A00](apk/2.26.37.71-reaction-key/reaction-model-key.txt), 0012..0050,
  first constructs a key from remoteJid/fromMe/id and participant.
  [Ayo.A0S](apk/2.26.37.71-reaction-key/message-key-builder.txt) reads the ID
  and builds 1gc with the supplied chat and boolean.
- [BoQ.A00](apk/2.26.37.71-reaction-key/ordinary-target-mapper.txt) treats
  a parsed 8oR specially at 0016..006a. Otherwise 00fe..0106 selects the
  envelope's 1gc.A00 chat; the 00f6..00fe branch keeps the supplied fromMe
  flag. Thus **inference from the traced ordinary branch:** substituting an
  arbitrary payload remoteJid as our ordinary update chat is not justified.
  Complete participant/own/hosted branches and Android parent storage are
  still **unknown/unported**.
- [8oR](apk/2.26.37.71-reaction-key/status-special-remote.txt) constructs
  user `status`, returns server `broadcast`, and has type 5. This is the
  status singleton, not every broadcast address. 0D1.A00 separately constructs
  location and ordinary broadcast variants at 00a4..00f4.

**Negative findings:** DGD.CF7 is explicitly FStatusReactionProtobuf, not the
ordinary reaction receiver. Its [builder](apk/2.26.37.71-reaction-key/status-reaction-builder.txt)
uses CYF's [status target helper](apk/2.26.37.71-reaction-key/status-target-mapper.txt).
The status serializer and DHR key writer are retained as separate field-inventory
paths; their full caller/runtime conditions were not established. Bal.AD7 is
an outgoing serializer and reaches CSZ.A01; complete outgoing normalization,
feature gates and encrypted reactions were not established by this trace.
No identity-only adoption, identity-change cleanup or full APK chat merge is
proved by any of these methods.

## Implemented local receive correction

The former reaction decoder required an ID but ignored remoteJid entirely.
It now requires both fields and a supported JID parse. Existing local MessageId
(128-byte), JID and reaction text bounds remain local. Unknown namespaces,
empty-user/server-only values and native special classes remain unsupported;
this is **not complete 0Cy/native parser parity**. Malformed reactions remain
opaque rather than becoming MessageUpdated events. The existing
`lab-inbox --inspect` reaction shape adds fixed `key_validation` labels:
valid, missing_key, missing_id, invalid_id, missing_remote_jid,
invalid_remote_jid, unsupported_remote_jid, malformed_reaction. No key,
remote address or target ID is echoed by this diagnostic.

For a primary direct reaction on the reacting sender's own incoming message,
projection resolves the target in the authenticated envelope's chat context.
The [existing bounded direct-record proof](peer-identity.md#primary-lid-receive-and-legacy-record-stability-2026-10-09)
can retain the original PN target chat/author while the reaction's `by` remains
its actual LID. A consumed target retains its key without restoring plaintext.
Absent or foreign legacy targets stay in the actual envelope namespace.
Ambiguous qualifying rows or missing original legacy route evidence refuse
inside the same write transaction; neither marker nor event can commit alone.

This is a **local historical-record compatibility rule**, not evidence of an
Android database migration. It neither renames rows, guesses PN/LID identity,
copies sessions/pins, deletes projection markers, nor creates retry authority.
Existing MessageUpdated markers are not rewritten: previously projected
misattributed updates remain historical evidence. Own/outgoing targets, group,
companion and status target reconciliation remain separate unfinished work.
The status-like recognizer now requires zero-component `status@broadcast`;
location/other broadcast targets and companion-suffixed status targets are not
recorded as status likes.

## Evidence and reproduction

An independent literal protobuf-key fixture covers absent/empty ID and remote,
invalid syntax, unsupported/server-only categories, both primary namespaces,
group/status keys, default fromMe and empty-text removal, with redacted labels.
A trusted historical-store projection fixture covers legacy/consumed/canonical/
absent/foreign/ambiguous/missing-route targets, actual reactor attribution,
restart/deduplication, unchanged original records and atomic refusal. This is
local projection evidence, **not a new cryptographic receive or live reaction**.
The existing receive authentication/ratchet tests remain required.

```sh
cargo test --locked -p whatsapp-api --all-features reaction
cargo run --locked -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-reaction-key.json \
  --output docs/protocol/apk/2.26.37.71-reaction-key --check
cargo run --locked -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-reaction-key docs/protocol/apk/2.26.37.71-reaction-key
```

The exact APK re-export/check passed for 26 files / 127,657 bytes. A checksum-
verified cached same-bundle diff took **0.028679 seconds**, with 21 unchanged
source topics and four unchanged metadata artifacts; [measurement](apk/reaction-key-cache-check.json).
That is selected-topic cached comparison, not a different-release or whole-APK
equivalence claim. Fresh malformed-key, reaction-on-legacy-target and status-like
live validation remain **unverified**; no new reaction is sent merely to test this.

Inspection now emits the shape diagnostic even when reaction decoding refuses
the key; the former placement inside the successful-decode branch hid exactly
those failures. A subprocess CLI regression reads an isolated malformed stored
reaction and asserts the missing_remote_jid label without the target ID. It
also inspects an extreme captured timestamp without a panic. Capture ages use
one local timestamp per inspection, with checked subtraction. Per-capture now()
previously made repeated shapes differ during a long scan; this was a diagnostic
artifact, not evidence that four captures had been removed.

Final shared-code validation passed: **727 workspace/all-feature tests**, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, 24 default CLI
checks, CLI reference generation, OpenAPI library-symbol validation and formatting.
The CLI subprocess test exercises the actual inspection command's refusal path,
not merely the reaction decoder helper.

**Observed offline on retained state:** reprojection upgraded zero events without
network requests. Peer-key/action/receipt/outbox/event inspection stayed unchanged;
two saved reaction shapes reported `key_validation=valid`. Noncapture inspection
was unchanged apart from the new validation suffixes. Capture shape sets matched
after normalizing elapsed age; the former scan's per-capture time produced four
extra duplicate diagnostic lines, not proof of deleted records. This comparison
does not claim raw encrypted-database byte equality or fresh live behavior.
No saved identity/request ID was reset, no uncertain action was replayed and no
new chat or live request was sent in this chunk.

## Outgoing intent attribution and quote scope (2026-10-09)

**Confirmed local audit:** `outbox::Outbox::by_wire_id` formerly returned the
first matching row and never examined later pages for ambiguity. The own-message
branch of `transport::outgoing::reply_payload_for` trusted the caller's supplied
chat key, but did not compare that chat against the actual saved outgoing intent
before copying its text into the quote. A foreign saved wire ID could therefore
supply another chat's body. No such real-account disclosure is established;
this is a code/fixture finding, not a live incident.

**APK evidence remains the preceding hash-pinned 2.26.37.71 trace:** Bal.A01
0206..0256 and BoQ.A00 00fe..0106 keep ordinary targets in chat context, with
Ayo.A0S constructing their actual message key. This supports contextual
attribution, **not** a traced Android outbox migration or native alias authority.
BoQ is a reaction-key helper, not the quote decoder. The quote-body guard derives
from the local immutable SendRequest.chat association; this does not establish
native quote-parent validation. No new APK trace/export was necessary for this
local storage correction; the 21-topic reaction bundle remains unchanged.

**Implemented locally:** shared paginated lookup now examines all normal-outbox
namespaces, selects only a unique wire reservation and returns fixed
`duplicate outbox wire id` / `invalid saved outbox entry` errors. The new
`by_wire_id_in_write` performs the same read-only lookup within the existing
ratchet/event transaction and sees its uncommitted rows. These are local
consistency diagnoses, not WhatsApp reasons. A reserved wire ID is not proof of
socket write, server acceptance, handset delivery or a successful registration.

A primary direct reaction to our message (`fromMe=false` from the remote
sender's perspective) can retain its original immutable outgoing chat key if
the current sender is the same exact primary address, or if the PN/LID pair has
one existing encrypted `authenticated-lid:<LID>` relation to that exact PN.
Both PN-to-LID and LID-to-PN perspectives use this **same explicit relation**;
there is no reverse graph scan or materialized reverse map. Plain/AD-zero
component correlation retains the existing primary-address rules. Different
PNs or LIDs cannot bridge through a shared alias. Contact/directory metadata and
raw sender_pn alone do not authorize this selection. Malformed saved relations
refuse with the existing fixed `invalid authenticated LID mapping` error.

The MessageUpdated target uses the original chat, Outgoing direction and no
remote target author, while `by` remains the actual reactor. This attribution
never mutates OutboxEntry, facts, attempts, progress, identities or pins, and
never resends a reservation. An uncertain send remains uncertain; a reaction
adds no server-accepted/delivered/read fact. Missing, foreign or unattributed
normal-outbox targets keep the existing actual-envelope namespace fallback.
Companion/group/status parents, own-device synchronization and lab-only sends
without a normal OutboxEntry remain separate unfinished work. Existing event
markers are preserved rather than rewriting previously projected updates.

Outgoing quoted text now validates the **actual saved intent's chat** before
using its body. `saved quoted message belongs to another chat` refuses before
login-identity checkpoint writes. Unknown IDs retain the existing empty-quote
fallback; other media quote support and cross-namespace quote normalization are
not established. No caller-provided reference can transfer a foreign body by
merely claiming the requested chat. This is local request attribution policy,
not a new APK/server-mandated field, alias adoption or refusal reset.

**Offline fixture evidence:** a 300-entry encrypted-store test chooses matches
in the first and last physical pages, proves a transaction sees its uncommitted
duplicate, refuses without echoing the ID, rolls back that insertion, leaves
progress/facts unchanged and redacts malformed-entry diagnostics. An eleven-case
projection matrix covers both alias directions, same PN/LID, absent/foreign
pairs, raw-alias-only deferral, malformed relations and duplicate wire IDs. Store
reopen preserves one reaction event and unchanged original outbox rows, including
DeliveryUnknown. A quoted-text regression refuses a forged foreign reference
without checkpoint writes or private body in the error, permits the original
chat's body, and retains the unknown-ID fallback. These are local projection/
reservation fixtures; they do not establish new live E2E reaction interoperability.

Validation of this extension: **730 workspace/all-feature tests**, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, 24 default CLI
checks, regenerated CLI reference, OpenAPI library-symbol validation and
formatting passed. Retained-account offline reprojection upgraded zero events;
peer-key/action/receipt/outbox/event inspections remained unchanged, as did
normalized capture shapes and both valid saved reaction diagnostics. The saved
state comparison does not claim raw encrypted-database byte equality.

**Observed live smoke check:** the final behavior build authenticated and listened
for 45 seconds with reconnect and chat sends disabled, then stopped cleanly
(exit 0). Peer-key/action/receipt/outbox/event summaries stayed unchanged;
decrypted payload inspection remained 15 records. Capture diagnostics grew from
427 to 432 shapes; those are diagnostic shapes, not five user messages. No fresh
reaction or new decrypted DM was established. This verifies connection/normal
receive plumbing, not the new target attribution against a live incoming reaction.
The original two uncertain actions were retained and never replayed. No outbox
send, identity rotation or server-wait reset occurred.

## Incoming group original author (2026-10-09)

**Confirmed local defect, corrected:** ordinary incoming group reaction
projection previously dropped `BZa.participant_`. Its direct-chat fallback
interpreted `fromMe=false` as a reaction to our message. A different group member
can be the original author, so that fallback produced the wrong MessageKey.

**Confirmed static**, comparison 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
[Bal.A00](apk/2.26.37.71-reaction-key/reaction-model-key.txt), `002e..0046`,
reads `BWI.key_.participant_` and parses its UserJid before constructing the
participant-bearing reaction model (`0046..0054`). BZa.dynamicMethod gives
participant field 4, separate from remoteJid/fromMe/id. This is evidence for
preserving the explicit author; it does **not** prove every native own/hosted
canonicalizer branch or a mandatory participant gate in C7G.

**Confirmed local / inferred compatibility:** group reactions now use the
explicit participant. A third member stays an incoming author independently of
`fromMe`; the reacting member remains the separate `Reaction.by`. Exact known
own PN/LID from the authenticated `login-identity` checkpoint classifies an
outgoing target. No directory/contact alias is adopted to infer ownership. An
absent own checkpoint leaves the key unprojected rather than guessing direction.
The public `reactions::group_target` is the offline key/own-scope helper.

The target group must match the envelope group. Participant must parse as PN/LID
with a nonempty user and without an agent component. The local Jid parser accepts
server-only strings, so parse success alone does not establish an author. Missing/malformed/foreign-group keys remain opaque;
these are conservative local support guards, not claims that native C7G enforces
them. An agent-qualified reactor also stays opaque because DeviceAddress cannot
retain that component. Full hosted/agent behavior remains **unknown/unported**.

Matching [participant-scoped parent records](group-inbox.md) supply the original
actual device so the update matches this client's existing durable MessageKey.
Legacy parents are selected through their original route, not by first scan
match. Consumed parents qualify from route/status only; plaintext is neither
restored nor inspected. A target absent locally retains its validated explicit
author. Exact-group saved outgoing intents retain their original chat and IDs.
Foreign outgoing claims, duplicate parent claims or missing original saved
routes refuse with fixed local errors in the enclosing transaction. Original
incoming captures remain available for diagnosis; no refusal resets occur.
Reactions never add acceptance/delivery facts or modify the parent's body,
identity, uncertainty, intent or progress. Existing projection markers remain
once-only; old misattributed events are historical evidence, not rewritten.
An existing message_updated marker skips group parent re-resolution, so a later
parent conflict does not make its already-authenticated redelivery fail. Legacy
markers without a stored kind retain the existing migration behavior; full native
projection-marker migration remains unknown.

`lab-inbox --inspect` adds a fixed `group reaction target_validation` label for
payload/own-scope checks. It does not claim to resolve stored parent ambiguity.
Receive/replay errors identify those storage problems separately. Labels carry
no addresses, text or plaintext body. Inspection advances no ratchet or marker.

**Confirmed offline:** descriptor-grounded literal bytes test missing/default/
true/false fromMe independently of the payload builder, third-member PN/LID
attribution, known own PN/LID and fixed invalid-key diagnoses. Projection
fixtures cover same-ID PN/LID parent namespaces, original companion devices,
legacy/new selectors, consumed bodies, add/removal, immutable parent/outbox
records, duplicate-claim rollback and reopen/no second event. Trusted stored
route fixtures prove local attribution, not live peer authentication.
**Unknown:** fresh live group reaction attribution, native own/hosted mapping,
and full group quote/keep-in-chat/poll target normalization.

**Confirmed saved inspection:** the current account contains two older group
reaction keys that pass the new target payload/own-scope validation. Comparing
before/after inspection found exactly two added `target_validation=valid` lines,
no removed lines and no other inspection changes. Peer keys, actions, receipts,
outbox and events stayed unchanged. This does not establish fresh live target
projection or repair historical reaction events; their markers remain intact.

**Confirmed static follow-up / inference boundary:**
[BoQ.A00](apk/2.26.37.71-reaction-key/ordinary-target-mapper.txt), `0006..0016`,
also parses the key participant before choosing its target. The fromMe-false
path at `00dc..00e8` calls `LX/08Z;->BO0(participant)`; its true branch reaches
`00ba` with the native key boolean set and no separate author. Interpreting BO0
as an own-user predicate is **inferred** from this control flow and its account
interface callers, not from an exported concrete implementation. The effective
v11/v12 gates, alternate actor/participant branches and full native PN/LID/hosted
semantics have **not** been established here. In particular, literal true/false
fixtures prove this helper's explicit-author contract, not APK equivalence for
contradictory fromMe/participant combinations. Re-anchor the concrete BO0
implementation and caller operand roles before extending native normalization.

### Native predicate and ordinary branch correction (2026-10-09)

The [group mapper trace](group-reaction-mapper.md) supersedes the BO0 inference
boundary above: 08a forwards to 08i.BO0, which compares own PN and LID using
native Jid user/server/type equality. It also proves that a normal group's
fromMe-true/present-author branch skips BO0 and produces an incoming author,
even for a contradictory own author. Rust now follows that branch; the prior
unconditional own-author classification was a local implementation limitation.
Own context is required only on the false/default branch. A device subtype is
not a native UserJid and is no longer normalized into own equality. Missing
native-author fallback, zero-device textual factory behavior, actor producers,
own-envelope context and hosted/status mapping remain unported. Historical
projection markers and saved state are preserved.

**Native ordinary-chat correction:** the same trace refutes the earlier local
requirement that key.remoteJid equal the envelope group. Supported plain
PN/LID/group keys now keep the envelope group, as BoQ does; no other chat or
parent body is adopted. Status and unported subtypes retain a separate fixed
refusal. The fixture changes a key's remote group and still binds the original
consumed parent, author and group.

Group envelope direction (2026-10-09, **confirmed static/local**): the APK 37.71
CnN/CgC/Cih/Bp2/Cna producer chain now establishes the ordinary network group's
selected sender and own-envelope flag. Sender-aware reactions preserve payload
direction for a known own sender; remote true/absent-or-empty-author references
fall back to the authenticated sender user. Inspection verifies the saved original
route against its selector. Existing projection markers, consumed bodies, parent
routes, outbox records and trust admission remain preserved. This supersedes the
ordinary own-envelope/source-producer gap above; full own-device synchronization,
hosted/status references, special chat normalization and fresh live own-device
reaction behavior remain unverified/unported. See [envelope direction](group-envelope-own.md).
