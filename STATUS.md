# Status — 2026-10-07

Blank-receipt failure continuation (2026-10-09, **confirmed static**):
four source topics identify Cnn/0zO as message-send event logging and separate
builder result 3 from native message status 20. No direct status setter or
receipt_user access is established; helper effects and primary cleanup remain
unknown/unported. No runtime, account or live change.
[Evidence and limits](docs/protocol/message-status.md#blank-receipt-failure-logging-continuation-2026-10-09).

Offline account diagnostics (2026-10-09, **observed/local**): the original
private key enabled saved-profile receipt inspection: nine ACKed reservations
(eight read, one retry), identical logical results across reopen; no live request.
Account pending-work counts now exclude retained cancelled/failed/in-flight
history and share the pending-list progress predicate. No resend/state reset.
[Count evidence](docs/protocol/account-status-counts.md). [Receipt check](docs/protocol/receipt-ack-correlation.md#original-saved-account-inspection-restored-2026-10-09).

Key-based receipt lookup (2026-10-09, **confirmed static**): seven topics
close ordinary/add-on overrides and show missing ordinary messages select the
add-on keyed receipt join. Pending membership is distinct from delivered/user
rows; primary cleanup remains unfinished. Offline saved-account receipts check
stopped before store open because its master-key environment input was absent.
No account mutation, runtime change or live request.
[Evidence and limits](docs/protocol/receipt-key-lookup.md).

Receipt inspection correction (2026-10-09, **confirmed local**): offline
receipts inspection now diagnoses malformed saved wire/IDs with fixed redacted
errors instead of silently dropping/truncating bytes or returning an empty ID.
Reservations and outcomes remain unchanged across failure/reopen; primary
cleanup and native user-row admission remain unfinished. No live operation.
[Evidence and limits](docs/protocol/receipt-ack-correlation.md#saved-receipt-inspection-diagnostics-2026-10-09).

Read persistence release relocation (2026-10-09, **confirmed static/local**):
a separate checked recipe locates six methods; signature plus physical caller
resolves a three-way field-writer ambiguity. Four string-free topics remain
manual and prevent automatic proposal. Cached ten-topic self-diff took
0.014920 seconds; no second release/runtime/live parity is claimed.
[Evidence and limits](docs/protocol/read-receipt-persistence.md#release-relocation-2026-10-09).

Deferred blank receipt expansion (2026-10-09, **confirmed static**): eight
source topics close DNt selector 6 as status-target device-set subtraction
and blank-device insertion, not normal receipt_user seeding. Existing-target
property and abstract override boundaries remain unknown; primary cleanup
still requires actual user-row membership. No runtime/live change.
[Evidence and limits](docs/protocol/blank-receipt-deferred.md).

Read-receipt persistence (2026-10-09, **confirmed static**): ten source topics
connect local status marking to existing message-row updates and separate
chat row/sort and status-table sent positions. IOException may be logged and
swallowed after the in-memory mark; no ACK wait is established. Complete
queue/store reconciliation remains unknown/unported; no runtime/live change.
[Evidence and limits](docs/protocol/read-receipt-persistence.md).

Read-receipt submission (2026-10-09, **confirmed static**): thirteen source
topics separate grouped queueing from later per-message policy, 256-ID batches,
forced read-self and local status-16 marking. Maximum store-time is preserved;
worker/migration authority and queue atomicity remain unknown/unported.
No runtime or live change.
[Evidence and limits](docs/protocol/read-receipt-submission.md).

Read-setting observer binding (2026-10-09, **confirmed static**): sixteen
source topics connect generated observer membership to DK9 and distinguish
category/value Bsa from empty list callback C5W. Its read-work scan uses saved
read/receipt positions before played-receipt work; selection is not transmission.
Full sender/other-observer authority remains unknown; no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-observers.md).

Read-setting pending lifecycle (2026-10-09, **confirmed static**): eight
source topics bind the shared 20-second main-Handler runnable to clearing all
pending settings and notifying observers, without a preference write or resend
in that branch. UI pending display differs from committed settings. Effective
state/observer authority remain unknown; no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-pending.md).

Read-setting fetch and observers (2026-10-09, **confirmed static**): thirteen
source topics distinguish fetch mode 23463 from set mode 23639, retain SMAX
get/callback parsing, and identify played-receipt queueing in a settings observer.
Boolean local-set refusal and local submission versus server acceptance are
explicit. Effective settings/bootstrap/observer authority remain unknown;
no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-bootstrap.md).

Read-setting wire producer (2026-10-09, **confirmed static**): a read-self
receipt ACK with a present readreceipts attribute emits arg1 event 227 and
still reaches generic ACK forwarding. Four source topics connect enabled
dispatch to the retained 14c preference consumer. Effective state and fallback
registration remain unknown; no runtime/live change.
[Evidence and limits](docs/protocol/read-setting-sync.md#read-self-ack-wire-producer-continuation).

Read-setting synchronization (2026-10-09, **confirmed static**): ten source
topics distinguish local-off-preserving discriminator 227 from validated
category-map adoption and pending-value mismatch guards. A0Q dispatches observers,
not a direct query. Effective account setting, bootstrap and complete lifecycle
remain unknown/unported; no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-sync.md).

Device-receipt timestamp correction (2026-10-09, **confirmed local / APK-backed**):
Java BMP receipt times now reach earliest-positive mutable device rows rather
than remaining falsely pending. A failing-before test checks received-row
preservation through companion cleanup, rollback, immutable provenance and reopen;
normal-outbox receipt attribution is also exercised. Primary cleanup remains
unported; no real account/live change.
[Evidence and limits](docs/protocol/device-receipt-rows.md#java-receipt-time-and-companion-cleanup-correction-2026-10-09).

Ordinary text status source (2026-10-09, **confirmed static / bounded inference**):
user-input text construction uses 1Ze/1he and the NTP-preferred clock; its
plain construction boundary differs from media's explicit status 1. Seventeen
source topics retain callbacks and exclude unrelated setter workflows.
Full preparation/user-row admission and identity receipt cleanup remain unported.
[Evidence and limits](docs/protocol/message-status.md#ordinary-text-construction-and-excluded-setter-candidates-2026-10-09).

Trusted-contact timestamp parser (2026-10-09, **confirmed local / static-backed**):
parent/token times now use Java-compatible decimal-long parsing within the
existing nonnegative storage range. A failing-before regression covers BMP
digits, override/fallback, malformed-token rollback and reopening. No real
account/live change; complete native token admission remains unfinished.
[Evidence and limits](docs/protocol/token-notification-peer.md#timestamp-parser-correction-2026-10-09).

NTP clock preference (2026-10-09, **confirmed static**): nine source topics
establish separate NTP/WhatsApp baselines, offset sign, acquisition correlation,
cache/recent-sync checks and feature-controlled rejection. NTP fetching,
effective property 2225 and upstream scheduling remain unported/unknown.
No runtime or real-account/live change.
[Evidence and limits](docs/protocol/token-clock-producers.md#ntp-preference-and-acquisition-2026-10-09-continuation).

Confirmed clock runtime (2026-10-09, **confirmed local / APK-backed**):
authenticated login and server-ping time now save source/offset separately from
inferred stanza samples. Linux/Android use suspend-aware elapsed time during the
same process; restart uses the saved offset. Client mute resolution prefers this
confirmed source. Offline `clock-inspect` reports redacted provenance. NTP clock,
token scheduling and live parity remain unfinished; no real-account/live change.
[Evidence and limits](docs/protocol/token-clock-producers.md#confirmed-runtime-clock-inputs-2026-10-09).

Server-ping clock input (2026-10-09, **confirmed static/local audit**):
`iq type=get xmlns=urn:xmpp:ping` supplies envelope t through signed-long/fallback
parsing to Android arg1 selector 4. Result IQs do not use this clock branch.
Six source topics close wire provenance; downstream reply writing and Rust
clock parity remain unported. No runtime or real-account/live change.
[Evidence and limits](docs/protocol/token-clock-producers.md#server-ping-wire-producer-2026-10-09-continuation).

Token clock producers (2026-10-09, **confirmed static/local audit**):
APK `08B.A07` takes login `success@t` through `1Rd` or selector-4 Bundle time,
sets an elapsed baseline and persists client/server difference. Seven source
topics close those inputs; selector-4 wire provenance and full Rust clock parity
remain unknown/unported. Current stanza clock sampling is still an inference;
no real account or live change.
[Evidence and limits](docs/protocol/token-clock-producers.md).

Trusted-contact peer guard (2026-10-09, **confirmed local / static-backed**):
privacy-token receive now rejects empty PN/LID users before storage or ACK;
supplied empty sender_lid keeps a fixed diagnostic. Generic server-only JIDs
previously passed our namespace-only guard. A failing-before regression checks
text/Plain/AD inputs, unchanged tokens/pins/sessions and reopening; valid token
handling remains covered. No real account/live change.
[Evidence and limits](docs/protocol/token-notification-peer.md).

Token mapping consumers (2026-10-09, **confirmed static/local**):
selected table readers feed a separate specifier cache and native stable-ID list.
`00D.A0C` compares/logs values; its caller separately chooses native or fallback
return. Readiness and initial preference inputs are traced, effective account
settings remain unknown. Renderer now decodes APK shift opcodes b8/c5 with a
failing-before regression; seven source topics preserve the boundary.
[Evidence and limits](docs/protocol/token-feature-inputs.md#mapping-consumers-and-shadow-comparison-2026-10-09).

Token MobileConfig mapping (2026-10-09, **confirmed static**): provider
233 resolves to `09N`; its generated array has nonzero initial specifiers for
15491 (`0x811d83000016e2`) and 14303 (`0x811af9000014f5`). Eight source topics
preserve the binding and indexed lookup. Effective native values/readiness and
array consumers remain unknown; no runtime/live change.
[Evidence and limits](docs/protocol/token-feature-inputs.md#property-to-mobileconfig-mapping-2026-10-09).

Token Boolean provider bindings (2026-10-09, **confirmed static**):
`00D` binds `1Y4(4)` and `1Y5(0)` to its Boolean providers. Both resolve IDs
through `09M.B2y`; zero mapping returns false, nonzero calls MobileConfig
`AVn`. Six source topics preserve bindings/selectors/branches; actual mappings,
native values and effective precedence remain unknown. No runtime/live change.
[Evidence and limits](docs/protocol/token-feature-inputs.md#boolean-provider-bindings-2026-10-09-continuation).

Token feature inputs (2026-10-09, **confirmed static/local**): APK 37.71
registers properties 15491/14303 as false Booleans; its getter can use caches,
preferences or MobileConfig, so effective values remain unknown. Offline
`abprops-query --show` now reports selected returned token values and invalid
last-value flags, without default fallback or runtime activation. Twelve source
topics and regression/reopen tests preserve the evidence. No real account or
live change. [Evidence and limits](docs/protocol/token-feature-inputs.md).

Token canonicalization gates (2026-10-09, **confirmed static**): APK
37.71 normalizes type-zero phone users independently of property 15491, which
suppresses only counterpart lookup/cleanup. Separate property 14303 selects
pending-job namespace. The six-topic snapshot records incoming-byte versus
outbound timestamp getters and four physical counterpart-helper callers. Effective
flags, mapping authority and refresh runtime remain unknown/unported. No account
state or live request changed.
[Evidence and limits](docs/protocol/token-canonicalization.md).

Identity-only peer prerequisite (2026-10-09, **confirmed local**): public
query/inspection now refuses an empty peer user before reading or saving action
state; shared primary-address matching also refuses server-only values. The
regression failed before the fix and covers PN/LID, Plain/AD encodings, unchanged
pins/sessions/checkpoints and reopening without replay of an uncertain query.
This is our input guard, not a WhatsApp refusal; no real account or live change.
[Evidence and limits](docs/protocol/peer-identity.md#empty-peer-user-refusal-2026-10-09).

Receipt ACK state boundary (2026-10-09, **confirmed static/local**): APK
37.71 DDH preserves ACK status 10/12 and zero time; selected ordinary items
can reach 15A.A07 with a null callback through Cmn. Permanent-failure status
21 skips that update. Deferred jobs preserve key direction/status/time and use
a separate processing budget. The six-topic snapshot closes model/deferred-job
inputs, not complete persistence or primary user-row authority. Rust ACKs still
settle receipt reservations separately from peer facts; no runtime or live change.
[Evidence and limits](docs/protocol/receipt-ack-state.md).

Receipt ACK handler bindings (2026-10-09, **confirmed static/local**): APK
37.71 N12 providers resolve read selector 86 to 14c and state selector 262
to 17J. Read processing checks chat type 5; state processing enqueues Cmn
with arguments 0/5000/0. The offline switch renderer now retains every declared
case and rejects malformed tails, closing its former 64-case visibility gap.
The eighteen-topic snapshot preserves concrete worker boundaries; full persistence
and live parity remain unknown. No account state or live request changed.
[Evidence and limits](docs/protocol/receipt-ack-handlers.md).

Specialized receipt ACK producers (2026-10-09, **confirmed static**): APK
37.71 read ACKs produce selector 86 bundles; played/server-error produce selector
262 models with arguments 10/12 and zero time. The fourteen-topic snapshot
traces arg1 dispatch to N12 A0d/A0Z or registered/fallback handlers. Concrete
application and persistence remain unknown; these are not peer timestamps or
receipt_user membership. No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/receipt-ack-special.md).

Receipt ACK dispatch (2026-10-09, **confirmed static**): APK 37.71 generic
ACKs remove full Cb1 callback keys under a map lock, then separately handle ID
queues. Read-job registration normalizes only device-zero address forms; duplicate
registration replaces callbacks after logging. The ten-topic snapshot retains
null versus original-node callback payloads. Full queue ownership, timeout/error
lifecycle and receive effects remain unknown. No runtime, account state or live
request changed.
[Evidence and limits](docs/protocol/receipt-ack-dispatch.md).

Receipt ACK participant guard (2026-10-09, **confirmed static/local**): APK
37.71 Cb1 compares nullable participants. Rust now rejects a participant-less
ACK for a group read/retry reservation; incomplete error ACKs also leave the
saved outcome unknown. Regression coverage checks restart/no replay and complete
matching ACKs. The former CZH attribution is corrected and retained as a negative.
No private state or live request changed; full primary cleanup remains unfinished.
[Evidence and limits](docs/protocol/receipt-ack-correlation.md).

Linked metadata observers (2026-10-09, **confirmed static**): APK 37.71
case 26 invokes a no-payload Bjk callback. A physical scan finds thirteen empty
concrete bodies and two state-update consumers; transitive effects/registration
remain unknown. The five-topic snapshot also names the browser/desktop flag
and distinguishes view lookup from raw-cache lookup. No runtime, account state
or live request changed; this is not identity-verification or receive-replay proof.
[Evidence and limits](docs/protocol/linked-metadata-observers.md).

Linked-device runnable writes (2026-10-09, **confirmed static**): APK 37.71
DLo case 23, DM7 case 2 and DNh case 20 write history access, last-active time
and nickname. A three-topic anchored snapshot completes immediate classification
of the twenty physical cache stores; transitive authority remains unfinished.
Last-active mutates a record before SQL, and only its separate companion
preference update has a greater-than check. No runtime, account state or live
request changed.
[Evidence and limits](docs/protocol/linked-runnable-writes.md).

Linked-device metadata caches (2026-10-09, **confirmed static**): APK 37.71
place/instrumentation writers update SQL, null only the view map and mutate
one existing raw-map record. The five-topic snapshot distinguishes that behavior
from GEu's broader store-reset context. Location requirements, metadata authority,
remaining runnable writers and complete linked-device trust remain unknown.
No runtime, private account state or live request changed.
[Evidence and limits](docs/protocol/linked-metadata-cache.md).

Linked-device cache writes (2026-10-09, **confirmed static**): APK 37.71
has twenty physical raw/view cache writes across fourteen methods. The six-topic
snapshot traces metadata writes, logout marking versus row deletion, own-alias
selectors and bounded PN/LID migration resets. Full remaining writer/caller
authority and commit/rollback semantics remain unknown; no cache reset implies
verified identity or removal permission. No runtime, account state or live
request changed; linked-device receipt handling remains unfinished.
[Evidence and limits](docs/protocol/linked-cache-writes.md).

Read-job saved linked set (2026-10-09, **confirmed static**): APK 37.71
requires loaded Me and derives its set from saved devices, excluding positive
logout times. The nine-topic snapshot traces SQL loading and bounded own
LID-to-PN device conversion with existing-PN collision preference. Set membership
is not key verification; full ingestion/deletion authority, cache invalidation
and read-self fanout remain unknown. No runtime, account state or live request
changed; primary cleanup remains unfinished.
[Evidence and limits](docs/protocol/read-linked-set.md).

Outgoing read-job trace (2026-10-09, **confirmed static**): APK 37.71
SendReadReceiptJob uses Cp2's shared receipt builder and extra-ID list construction.
Its outgoing read/read-self selection differs from the incoming factory policy;
job admission also checks IDs, status age and a saved set. The eleven-topic
snapshot supersedes wire-format inference. Rust still sends one explicit ID
per receipt; complete policy, linked-device scope and scheduling remain unfinished.
No runtime behavior, account state or live request changed.
[Evidence and limits](docs/protocol/read-receipt-job.md).

Receipt read-policy classifiers (2026-10-09, **confirmed static**): APK 37.71
uses a configured JID-user list, chat codes -1/-3 and contact-addon presence.
The twelve-topic snapshot retains dependent-account age-state checks and a
predicate that can schedule a refresh before returning true. Effective state,
classifier writers and refresh completion remain unknown; full user-row
admission and primary cleanup remain unfinished. No runtime, account state
or live request changed.
[Evidence and limits](docs/protocol/receipt-read-classifiers.md).

Receipt factory read policy (2026-10-09, **confirmed static**): APK 37.71
uses read_receipts_enabled default true plus chat, contact and account predicates.
The eleven-topic snapshot retains literal smb:/ent: prefix checks, premium-message
existence queries, dependent-account preference and the entry thread guard.
Effective preferences/classifiers and settings synchronization remain unknown;
full user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](docs/protocol/receipt-read-policy.md).

Receipt state-only suppression (2026-10-09, **confirmed static**): APK 37.71
C8L selects incoming keys and mapped state arguments 13/16/17. Its state-only
caller skips A07 when the chat last-read message row is at least message A0j;
the missing-chat-model getter returns 1. A two-topic snapshot and eleven-write
inventory retain the immediate boundary. Full chat read-state lifecycle,
self/user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](docs/protocol/receipt-state-suppression.md).

Receipt state-only admission (2026-10-09, **confirmed static**): APK 37.71
keeps message-key direction separate from receipt status: read-self forces false,
while played-self retains the recipient-presence branch. The ten-topic snapshot
traces A07 advancement, played eligibility and core persistence. A07 true does
not prove every downstream database update succeeded; the base A0V helper tests
chat kind, not a view-once flag. Full self/user-row admission and primary cleanup
remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/receipt-state-only.md).

History mode provenance (2026-10-09, **confirmed static**): APK 37.71
BMv reads positive registration_device_id from preferences; BNm tests loaded
Me presence. These are separate from own-user equality, registration state and
linked-device authentication. An eight-topic snapshot retains mode/getter/setter
and the bounded completion-handler source. Full pairing/crypto and receipt_user
admission remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/history-mode.md).

History queue admission (2026-10-09, **confirmed static**): APK 37.71
stages history chunks in the peer handler after own-user and companion-mode
checks. SQL insertion-constraint recovery updates the original message ID;
queue selection and null/empty binding differences are retained. The four-topic
snapshot preserves full staging/store methods. Upstream crypto, linked-device
trust, media authentication and receipt_user creation remain unfinished.
No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/history-queue.md).

Mapping import entry points (2026-10-09, **confirmed static**): APK 37.71's
remaining conversation-mapping callers are chat-transfer import and the initial
history-bootstrap branch. The immediate history inputs are bounded file reads
or inflated inline chunks; nonce presence/storage is not demonstrated nonce
verification. A four-topic snapshot preserves the callers and full worker.
Upstream chunk/transfer authentication and receipt_user admission remain unknown.
No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/mapping-admission.md).

Chat counterpart provenance (2026-10-09, **confirmed static**): APK 37.71
uses a HashBiMap populated from distinct positive original chat-JID rows and
from parsed conversation account-LID/phone pairs. Reverse lookup, chat deletion,
cache clearing and a mapping-sync consumer are retained in a twelve-topic snapshot.
The backup restore caller is distinct from direct wire admission; upstream trust,
full sync receiver and receipt_user membership remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](docs/protocol/chat-counterparts.md).

Receipt counterpart selection (2026-10-09, **confirmed static**): APK 37.71
8G8.A02 expands selected JID rows only through the migration gate and the
0Yo.A0B(input, false) mapping path; its comparison uses object identity.
JID-row materialization does not establish receipt_user membership. The eleven-topic
snapshot retains the mapping/gate and full untraced chat-cache loader. Mapping
producers, full user-row admission and primary cleanup remain unported.
No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/receipt-counterparts.md).

Receipt metadata provenance (2026-10-09, **confirmed static / inferred lifecycle**):
APK 37.71 has four physical route-flag writers and hydrates 1ic mentions from
message_mentions, separately from receipt_user. Clearing its list preserves
lookup bit 1 and the route flag; BotGroupMetadata attachment explicitly sets or
clears a separate bit. The fourteen-topic snapshot preserves these sources and
remaining constructor inventories. Complete user-row admission and primary
cleanup remain unported. No Rust runtime or account state changed.
[Evidence and limits](docs/protocol/receipt-metadata.md).

Receipt metadata route (2026-10-09, **confirmed static**): APK 37.71 special
classification requires chat exclusion, flag/first qualifying metadata user,
message A02 and descriptor absence. It does not count recipients; A0z bypasses
metadata selection only. The fourteen-topic snapshot retains the helper chain.
Disassembler opcode-c2 naming is corrected with a literal-byte regression.
Metadata producers, full user-row admission and primary cleanup remain unported.
No messaging runtime, saved identity or reservation changed.
[Evidence and limits](docs/protocol/receipt-route.md).

Ordinary cursor status (2026-10-09, **confirmed static**): APK 37.71 0VA.A00
reads status into the virtual message setter and loads server receipt time
without enum conversion or minimum selection. Two physical cursor entry points,
the type factory and one status-1 media preparation source are retained in a
five-topic snapshot. Full outer SQL/cache and user-row creation remain unknown;
no Rust runtime, reservation or account state changed.
[Evidence and limits](docs/protocol/message-loader.md).

FStatus conversion (2026-10-09, **confirmed static**): APK 37.71 7z7.A00
maps named 1cE enum ordinals with media/subtype conditions; 7zY.A01 keeps
native 178 integers separate. The ten-topic snapshot retains the transition
matrix and all six direct enum writes. This narrows the earlier restoration
trace: ordinary SQL status loading and universal outgoing initial status remain
unknown. No Rust runtime or account state changed.
[Evidence and limits](docs/protocol/status-enum.md).

Success ACK admission (2026-10-09, **confirmed static / tested offline**): APK 37.71
wire parser → device conversion → event 19 → handler proves that a nonnull
converted participant suppresses success acceptance. Rust now guards supported
PN/LID participants, including primary forms, while preserving error ACKs and
original uncertain reservations across restart. The twenty-topic snapshot also
retains native accepted-message SQL, status branches, failure helpers and scoped
poll-path negatives. Native extra user subtypes, complete status/user-row parity
and a qualifying live ACK remain unverified. No sends or account resets ran.
[Evidence and limits](docs/protocol/message-ack.md).

Device send preparation (2026-10-07): selected recipient queries now
cover all local 64-user batches once, without draining unrelated pending users.
Refused, partial and omitted-user replies block before message reservation even
with cached lists. Saved omitted counts remain null for historical/unknown
records; device-query inspects them offline without identifiers or resets.
130-target and cached-refusal/duplicate-omission regressions pass. A 20-second
receive-only live check saved one acknowledged device query with one ingested
user and zero errors/omissions; no chat sends or queued work remained. The list
is primary-only/no signed index list. Large-group batches and own device sync
remain unverified.
[Evidence and limits](docs/protocol/messaging-transport.md#scoped-device-batches-and-completeness-2026-10-07).

Group hash inventory (2026-10-07, offline): saved recipient companions now
contribute to the candidate V2 hash, alongside the known own primary. Previously
they could be addressed but omitted from phash. New group fanout-inspect shows
redacted input counts and own inventory unknown; explicit --hash reveals the
candidate. Complete own-account device sync, active feature selection and live
companion-group sending remain unverified. The 34-method group snapshot compares
in 0.045388 seconds. [Evidence and limits](docs/protocol/groups.md#saved-device-hash-planning-2026-10-07).

Participant hashing/cache trace (2026-10-07, offline): APK 37.71 confirms
UTF-16 ordering, V2 literal agent zero and feature-gated V3 no-agent hashing.
Pure Rust hashing and offline inspection match 684 executed host-Java reference
modes; default xtask feature-gating is fixed. Existing member updates preserve
devices; traced cache/alias writes do not prove a database transaction.
Automatic sends remain V2; the subsequent correction includes saved recipient
devices. Own inventory and active feature selection remain unfinished.
The expanded 86-method snapshot compares in 0.052231 seconds. No account state
or live operation changed. [Evidence and limits](docs/protocol/groups.md#participant-cache-and-versioned-hashes-2026-10-07).

What is true now. The dated experiment log that led here, including every
superseded conclusion and dead end, is in
[`docs/status-history.md`](docs/status-history.md); per-area evidence is in
`docs/protocol/`. Labels: **live** = seen working against the real server,
**acked** = the server accepted it but a handset result is not confirmed,
**offline** = tested locally only.

Receive recovery (2026-10-07): the saved account recovered its blocked direct
and group texts, then delivery-confirmed both on live redelivery. The two
recovered-message read receipts received matching server ACKs. Current receipt
state: eight acknowledged reads and one acknowledged retry. Handset blue ticks
and fresh-message automatic-read validation remain unconfirmed.

The causes in our implementation were blanket refusal of changed peer pins
and group pairwise selection of a PN ratchet despite a held LID session. An
explicit APK-compatible incoming policy now commits an authenticated peer
replacement atomically with encrypted prior-key/session history and an
`IdentityChanged` event. The library default remains `pinned`; profile `l`
selects `apk_authenticated` in its preserved private corrected configuration.
No local account identity, send ID or reservation was reset. Automatic reads
also cover eligible authenticated duplicates, with once-only write-cut coverage.
Human safety-number verification remains incomplete. Native V4 explicit-input
display, binary export/comparison and guarded local manual marks are implemented
and tested offline. Current account/peer scheme and complete inventory selection,
protobuf-C edge equivalence and companion verification synchronization remain
unverified.
[Safety-number evidence](docs/protocol/safety-number.md). Outgoing key-bundle
mismatches now suspend the original queued intent before ratchet use;
`outbox-suspended` inspects them offline. Independent
`transport.outgoing_identity_changes=apk_authenticated` now stages a signed
directory-bundle replacement and commits prior-state history, the identity
event and original send reservation atomically. Explicitly selected suspended
intents can use their saved reply without another key query; invalid signatures,
own-account changes, stale-pin rollback and uncertain sends remain blocked.
The canonical outgoing policy remains pinned; live replacement is unverified. `peer-identity` now inspects both namespaces offline;
the recovered direct peer has different PN/LID pins, each matching its own
held session. Authenticated history connects the PN pin to the updated LID pin.
A 25-second directory-only live check obtained a valid signed current bundle
matching the recovered LID pin and differing from the PN pin. Peer pin/session
summaries stayed unchanged. One private test draft awaits authorization; no
new chat was sent.

Private state was recovered from the former checkout in the desktop Trash,
with originals preserved. The selected profile has no eligible chat sends or
queued actions. The successful 45-second receive window and 20-second read
window sent no queued chats. Evidence: [peer identity](docs/protocol/peer-identity.md),
[group session selection](docs/protocol/groups.md#group-pairwise-pnlid-session-correction-2026-10-07),
[read recovery](docs/protocol/messaging-transport.md#authenticated-duplicate-auto-read-correction-2026-10-07).
The earlier interval with no incoming messages is still unexplained.

## Accounts

Registration uses the recipe in
[registration-lab](docs/protocol/registration-lab.md#working-recipe)
(`enc_empty_h`, 2.26.37.70 User-Agent and token material).
`lab-signup resend-code` asks for a new code on an existing pending attempt.

## Messaging

| Capability | State | Where |
| --- | --- | --- |
| Direct text, PN or LID addressed | live | [messaging-transport](docs/protocol/messaging-transport.md) |
| Direct image, voice note, view-once image | live (receipt confirmed; playback not) | same |
| Replies, reactions, polls (direct) | live | same, [chat-actions](docs/protocol/chat-actions.md) |
| Receive + decrypt direct and group messages | live | [groups](docs/protocol/groups.md) |
| Read receipts, group (`mark-read --chat …@g.us`) | acked 2026-09-25; handset "Read by" unconfirmed | [messaging-transport](docs/protocol/messaging-transport.md) |
| Automatic read receipts (`transport.mark_read`, `listen`) | offline | same |
| Group send (sender-key fan-out) | live — operator read it | [groups](docs/protocol/groups.md) |
| Group replies | live — shown on the handset (the operator reacted to one) | same |
| Group reactions | **not rendering**: payload, stanza and target match the official client's, but the handset shows nothing and the server's ack carries no `count`; cause unknown | same |
| Retry receipts: sending ours when we cannot decrypt | runtime retry server-acknowledged 2026-10-06; redelivery still undecryptable | [messaging-transport](docs/protocol/messaging-transport.md) |
| Retry receipts: answering theirs after a fan-out | offline; one live retry was received before the answer existed | [groups](docs/protocol/groups.md) |
| Status: receive | live | [status](docs/protocol/status.md) |
| Status: post styled text | live — operator saw it | same |
| Status: post image | acked as `type=text`; now `type=media`, **not yet sent live** | same |
| Status: viewers (delivered/read/played) and likes received | live (`status-viewers`) | same |
| Status: sending a like, HD dual upload, audio | not implemented | same |
| App-state writes (contact action) incl. 409 catch-up | live | [chat-actions](docs/protocol/chat-actions.md) |
| Block/unblock, contact check, disappearing, keep-in-chat | live | same |
| Two-step verification set/change/disable | live (set) | [chat-actions](docs/protocol/chat-actions.md) |
| Display name (`presence name=…`) | written live, handset unconfirmed | [presence](docs/protocol/presence.md) |
| Display name app-state record (`pushNameSetting`, `critical_block`) | live — accepted at version 1 | same |
| About text (`iq xmlns=status`, `whatsapp-cli about`) | server-accepted 2026-09-28, as reported in the session export; handset unconfirmed | [presence](docs/protocol/presence.md) |
| Typing / recording indicators (`chatstate`) | typing handset-confirmed in the session export; recording and incoming indicators unconfirmed | same |
| Presence subscribe, incoming availability / last seen | live — server answered with the peer's presence | same |
| Prekey upload and replenishment | live | [messaging-completion](docs/messaging-completion.md) |

A single undecryptable incoming message no longer ends a session; it degrades
to `Undecryptable` and we send a retry receipt. Direct messages from a LID sender now decrypt with the LID
session our group sends established; that was why one of the operator's
messages stayed unreadable for four days (fixed 2026-09-28, verified by offline
replay). A group payload addressed only to this device (such as a
`messageHistoryBundle`) is now recorded and acknowledged instead of being
redelivered at every login.

## Not done

- **Stickers and video notes.** Field numbers are decoded from the APK
  (`stickerMessage` 26, `ptvMessage` 66, `VideoMessage` inner fields);
  StickerMessage's 22-field schema and image-key domain are now traced in
  2.26.37.71. Ordinary URL-bearing WebP sticker listing, sealed references,
  authenticated retrieval and local visual playback are implemented offline.
  Lottie, directPath-only retrieval and sending remain unsupported; no incoming
  sticker/PTV stanza or sticker display has been captured. Ordinary URL-bearing
  MP4 video/video-note listing, caption/reference events and video-domain
  authenticated retrieval are now implemented offline; live playback, circular
  rendering, HD variants and sending remain unverified or unsupported. One rebuilt
  60-second receive window authenticated/activated and stopped cleanly, with the
  20-event journal and nine acknowledged receipts unchanged; no fresh sample.
  [Sticker evidence](docs/protocol/stickers.md), [video evidence](docs/protocol/video-media.md),
  [apk-analysis](docs/protocol/apk-analysis.md).
- Calls; profile picture.
- Privacy settings IQ (namespace unknown).
- Channels and QR/companion pairing — parked by the owner.
- Companion-device fan-out and own-account device sync.
- History-notice handling. APK 2.26.37.71 identifies the formerly unexplained
  49-byte `[102]` envelope as `messageHistoryNotice`; offline inspection validates
  supported metadata and reports redacted counts without applying history. Saved-account offline inspection
  confirmed the 49-byte notice metadata: one receiver, no non-receivers, context
  present, no optional bundle timestamp or bot metadata. The APK group-notice handler
  and metadata requirements are traced. Supported normal group/meta context now
  projects a distinct control event; historical group/broadcast backfill is fixed.
  Canonical offline backfill upgraded this one notice; repetition upgraded zero.
  APK feature selection, rendering, timestamp units and history processing remain
  unverified. [Cross-device evidence](docs/protocol/cross-device-sync.md).

## Offline verification

Native research now locates separate Superpack decoders and V3/V4 fingerprint
symbols in one decoded stream. The ARM64 decoder loads and initializes all
19 JNI methods locally, exiting zero. A bounded first-entry reconstruction
returns native `9/2` / `could not extract file from archive`, with no output;
real host memory metadata did not fix it. Worker tracing now identifies an
oversized-file comparison against zero requested bytes, despite the matching
125448 declared/produced sizes. This partial-range result does not establish
ordinary full extraction failure; bounded full-entry checks have reached wall
limits without output or native refusal. Hash-pinned branch/futex tracing is
available. The unmodified ordinary full decoder now succeeds under QEMU,
returning 80 entries (one duplicate) and 79 valid ARM64 ELFs; a second run
matches all 79 hashes. V3/V4 fingerprint code anchors and containing unwind
ranges are located in reconstructed `libwhatsapp.so`; the algorithms and active
scheme remain unfinished. The helper's previously ineffective threaded-runner
limits are corrected, with eight regression checks including actual guest
execution, resource guards and preflight/output preservation. These tooling
checks make no live account request.
[Evidence, setup failures and remaining warnings](docs/protocol/native-unpacking.md).

529 all-feature tests (`cargo test --locked --workspace --all-features`) and
223 default-feature tests (`cargo test --locked --workspace`) pass on
2026-10-07; Clippy with `-D warnings` is clean. Rust 1.98.1 locally and in CI. Store, codec, crash
matrix, registration state machine and transport coverage are described in
[docs/design](docs/design/README.md) and the per-area protocol docs.

Before any live check, rebuild the release binary: `cargo test` does not
refresh `target/release/whatsapp-cli`, and a stale one once produced a wrong
"cause eliminated" conclusion.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo build --release -p whatsapp-cli --features experimental-transport
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
```


Native safety-number follow-up traces the shared V3/V4 5200-round SHA-512
helper, sorted key concatenation and separate identifier-bound display seed.
A pinned ARM64 slice checker validates iteration/input ordering on synthetic
data with explicitly modeled SHA/MCF operations. Full input producers,
JNI normalization, QR semantics and current scheme/human verification remain
unfinished; no account verification state or live connection changed.
[Evidence and reproducible check](docs/protocol/safety-number.md).


The native identifier callback is now traced back into Java: PN shortening
uses HNy, LID output uses the parsed JID user, and the return conversion
requires ASCII. Native key constructors establish 32-byte sorting versus
33-byte serialized hashing inputs. Java V4 creation and result dispatch now
connect to the recovered routines; the traced result path emits QR version 0.
Current scheme selection, complete device inventory, individual QR fields,
comparison semantics and human verification remain unfinished.
[Callback and dispatch evidence](docs/protocol/safety-number.md#native-to-java-identifier-callback-and-dispatch-mapping).


Native QR descriptors and serialization are now recorded with a reproducible
metadata extractor. Version zero carries serialized keys; version one their
identifier-free digest. V4 comparison resolves peer orientation and checks
keys, LID and additional PN/username/hosted-state conditions; LID/key equality
alone cannot establish verification. Fourteen offline tooling checks pass.
Full comparator primitive/error semantics, UI selection/inventory and the
verified-state action remain unfinished; no account verification was performed.
[QR evidence and schema](docs/protocol/safety-number.md#native-qr-schema-serialization-and-v4-comparison).


Fingerprint comparison primitives now establish NUL/non-ASCII rejection, PN
byte equality, username strncasecmp and orientation-dependent native status
remapping in APK 37.71. The actual native byte leaf passed 862 synthetic
comparisons without modeled callees; all 16 offline tooling tests passed.
V4 comparison is mapped to jvidispatchIOO selector 4. Its Java caller, active
scheme/device inventory and human verification-state commit remain untraced.
No account verification or live request occurred.
[Evidence and reproducible check](docs/protocol/safety-number.md#comparison-primitives-and-orientation-dependent-statuses).


APK 37.71 Java comparison selection now connects KXj.A01 to native V4
selector 4 behind holder/feature gates and an exact zero-result success test.
All three comparison callers are located. Camera/NFC success reaches result
presentation; the traced path does not establish a persisted human-verification
bit. Delayed callbacks, the verification writer and complete scheme/device
inventory remain unfinished. No account state or live request changed.
[Java caller evidence](docs/protocol/safety-number.md#java-comparison-selection-and-scan-result-action).


APK 37.71 verification metadata writers are now traced: the local sender
rejects stale rendered identities, rechecks the stored key under lock, advances
the action sequence and writes existing metadata before companion fan-out.
Inbound peer actions are companion-only with target/key/sequence guards;
history verification has separate presence/key requirements. BWE/BT5 schemas
and ProtocolMessage field 32 are confirmed. Delayed scan callbacks only present
results. Full human-trigger wiring, active fingerprint scheme/device inventory
and client implementation remain unfinished; no account state changed.
[Writer and schema evidence](docs/protocol/peer-identity.md#verification-metadata-writers-and-rotation-guards-3771).


The APK local verification control now traces from saved metadata/rendered
identity to explicit mark confirmation and separate unmark execution. The
writer receives that captured key, allowing its rotation guards to refuse stale
confirmation. Feature/predicate gates and complete fingerprint inventory remain
unverified; client verification metadata remains unimplemented. Two missing
DEX field-write mnemonics are fixed with literal APK instruction regression
coverage. No live account operation occurred.
[Local control evidence](docs/protocol/peer-identity.md#local-verification-control-and-captured-identity-3771).

APK 37.71 manual-verification contact exclusions now resolve to a hosted-set
predicate and a saved contact-state value ==2 (semantic name still unknown).
Hosted device-account type 1 and identity attestation HYBRID_E2EE type 2 come
from separate SQL sources; neither is the native QR HostedState enum. Native
display getter tracing also confirms that QR retains the full LID JID while
its decimal seed uses the parsed LID user. Current account inventory, active
scheme and human-verification implementation remain incomplete. No account
state or live request changed. See the peer-identity contact-exclusion and
safety-number display follow-ups.

Native fingerprint display is now implemented as an offline library primitive
and `fingerprint-display --config` CLI command using explicit private normalized
identifiers and raw public key inventories. Three independent synthetic ARM64
hash-flow fixtures match, including duplicate-key preservation; result labels
state that scheme/input selection is unverified. The calculation does not open
account storage or mark a peer verified. Active scheme, complete device-key
inventory, QR comparison and human verification remain unfinished.

Fingerprint calculation validation passed 535 workspace/all-feature tests,
229 default-feature tests, strict Clippy, warning-free rustdoc, CLI reference,
formatting and repository hygiene. A command-level synthetic calculation
matched the fixture pair without creating the supplied unused store path.
No network request or current-account verification was performed.

APK peer fingerprint inventory now traces to a selected device set and a
nonempty identity map with equal cardinality. Feature 0x1da3 excludes hosted
99-device classes; the local key helper has separate primary/companion controls.
`peer-identity` now reports retained E2EE device-key coverage without claiming
complete APK inventory or fingerprint readiness. Current scheme/contact-state
values, source inventory freshness and full own-device synchronization remain
unknown. No account keys or verification metadata were changed.

The rebuilt peer inspector passed on an isolated saved-account copy: PN has
no saved device list; its mapped LID has one retained device/one pin and no
missing pins, but companions_verified=false (primary-only fallback).
Original encrypted database bytes were unchanged. These are saved-state
observations, not a live directory refresh or proof of complete inventory.
Validation passed 536 workspace/all-feature tests and strict Clippy.

Native V4 combined fingerprint protobuf export now supports QR versions 0/1
through the library and paired fingerprint-display --qr-version/--qr-output
flags. Independent synthetic wire fixtures preserve full LID, hosted-state
presence and local PN omission; version 1 uses identifier-free H(K,K).
Exports are private/create-only and do not open account storage. This is binary
comparison data, not a QR image or a verified peer comparison. Active scheme,
complete inventory, scanning/comparison and human-verification commits remain
unfinished; no current-account QR or live operation was performed.

Native QR export validation passed 540 workspace/all-feature and 233 default
feature tests, strict Clippy, warning-free rustdoc and CLI reference. The rebuilt
CLI matched independent 200/229-byte synthetic payloads for versions 0/1,
created mode-600 files and refused repeats without changing them or creating
an account store. A zero-qualified JID test exposed and fixed the new export's
canonical-input guard; the generic JID parser and account state were unchanged.

Local manual verification metadata (offline, 2026-10-07): `peer-verification`
records an explicit human mark/unmark bound to an inspected pin and sequence.
`peer-identity` reports the saved bit/sequence/pin agreement. Authenticated
incoming and signed-directory replacement clear the bit atomically, preserving
sequence; rotation back does not restore it. Tests cover restart, stale/own
refusals, unchanged session/pin and archive rollback. No actual account was
marked. Automated QR comparison and companion synchronization remain unfinished.
[Evidence and operator semantics](docs/protocol/peer-identity.md#local-manual-verification-metadata-2026-10-07).
Validation for this change: 545 workspace/all-feature tests passed; strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, formatting,
regenerated CLI reference and repository hygiene passed (304 local links).
The first suite run caught the new OpenAPI `x-cli` including flags instead of
the command name; corrected to the repository's command-name contract before
rerunning successfully.

Native V4 comparison follow-up (offline, 2026-10-07): 115 synthetic cases execute
actual APK 37.71 comparison/helper/remapper branches with explicit host-modeled
getters and string/key/LID primitives. All 81 hosted-state combinations confirm
that both scanned states absent takes the legacy success branch regardless of
known states. Missing-versus-unequal username results and version-selected key
checks are reproduced. Converter refusal codes are statically attributed.
Whole parsing/orientation/Android string behavior and production scanning remain
unimplemented; no account was marked or changed.
[Executed branches and boundaries](docs/protocol/safety-number.md#executed-native-v4-comparison-branches-2026-10-07).
Validation: all 19 pinned native tooling tests and 36 CLI/API tests passed;
CLI reference regeneration and repository hygiene passed (310 local links).
The final attributed observation artifact also regenerated identically and
passed its three focused preflight/execution checks. Rust runtime code did not
change in this research increment.

Native username primitive (offline, 2026-10-07): the V4 checker now optionally
executes actual APK 37.71 merged MCFStringCompare and the pinned public Bionic
strncasecmp fixture. All 115 core statuses agree; 268 additional accepted-ASCII,
every-position mismatch, prefix, empty and 256-byte cases confirm ASCII
case-insensitive equality plus exact length for these fixtures. Public Bionic
is not APK-bundled or proof of every Android build; object construction and
non-string dependencies remain modeled. QR parsing/orientation/current inventory
and production scanning remain unfinished. No account state changed.
[Native string evidence](docs/protocol/safety-number.md#actual-native-ascii-username-comparison-2026-10-07).
Validation for the native-string increment: all 22 native tooling tests passed
with all four fixtures configured, alongside 36 CLI/API tests, regenerated CLI
reference, formatting and repository hygiene (315 local links). Rust runtime
code did not change; the account database was not opened.

Explicit binary comparison (offline, 2026-10-07): `fingerprint-compare` now
compares private native V4 protobuf against explicit known JSON, with traced
conversion order, both scan orientations and native status remapping. Tests
match all 113 executed non-abort core cases and 268 native string cases;
independent wire fixtures cover both versions/orientations. Result reports
caller-input and bounded native-parser scope; complete protobuf-C edge equivalence,
current inventory and live scanning remain unverified. No account or human
verification metadata is changed by comparison.
[Comparison scope](docs/protocol/safety-number.md#explicit-input-native-v4-binary-comparison-2026-10-07).
Validation: 551 workspace/all-feature tests and 239 default-feature tests passed,
with strict all-target/all-feature Clippy, warning-free Rustdoc, formatting,
CLI reference and repository hygiene (317 local links). Command-level synthetic
version-0/1 fixtures matched without creating an unused account-store path;
malformed payload/private-permission checks passed. Both-scanned-LIDs-matching
priority also passed its focused regression. The initial documentation check
compiled before the new OpenAPI operation was added; completing the contract
and regenerating/rerunning resolved the coverage refusal.

Native parser correction (offline, 2026-10-07): actual APK 37.71 protobuf-C
unpack/init/merge/free now reproduces a 50-case synthetic corpus. Eight prost
acceptance differences were observed; a valid fingerprint plus an unknown group
could previously produce a false match. `fingerprint-compare` now uses a bounded
schema decoder matching native tag/wire/length/scalar/merge rules, validated for
acceptance and all recognized field values. Its parser-authority output reports
corpus validation with other edges still unverified. Unknown opaque fields are
skipped, not round-tripped; current inventory/live scanning remain unverified.
No account store, pin/session or verification metadata was changed.
[Native parser evidence](docs/protocol/safety-number.md#actual-protobuf-c-parsing-and-bounded-decoder-correction-2026-10-07).

Validation (offline): 552 workspace/all-feature tests, 240 default-feature tests
and all 25 fully configured native tooling checks passed, alongside strict
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference. A rebuilt CLI matched both independent version-0/1 fixtures and
returned -8 for the otherwise-valid-plus-group and malformed fixtures; the
unused account-store path remained absent. The corpus test also checks explicit
nested-holder presence, with its final focused rerun passing.
Repository hygiene passed with 323 local links; staged diff whitespace checks passed.

Own fingerprint inventory follow-up (static, 2026-10-07): APK 37.71 own-device
selection is now traced to the cached `devices` directory, strict positive
logout-time exclusion, self LID-to-PN projection with existing-PN precedence,
and later fingerprint-facing address remapping. BNm checks loaded Me presence,
not signed-list freshness. Add/remove paths invalidate both directory caches.
Current membership, authentication and active fingerprint inputs remain unknown;
no account state was opened or changed.
[Own-directory evidence](docs/protocol/safety-number.md#own-directory-source-logout-filtering-and-alias-precedence-2026-10-07).

Validation: the APK hash was rechecked, and an independent exact A08 method
extraction matched the scoped snapshot byte-for-byte. CLI reference regeneration
and its test passed; repository hygiene passed with 329 local links. The initial
snapshot's trailing blank line was removed; final staged whitespace checks passed.
This change adds static research evidence only; runtime code and API/CLI
capabilities are unchanged, and no live account validation is claimed.

Fingerprint contact-state follow-up (static, 2026-10-07): the reusable
`xtask apk-field-refs` command found 17 direct writes to APK 37.71 0Da.A04.
Read/write tracing identifies `external_user_state`, with guest/Cameo,
pending-invitation and normal-user transitions, rather than Signal attestation
or QR HostedState. Complete producer authority and active scheme remain open.
[Contact input evidence](docs/protocol/safety-number.md#fingerprint-contact-predicate-is-external-user-state-2026-10-07).

Validation: 555 workspace/all-feature tests and 243 default-feature tests
passed, including three new APK/DEX regression checks. Strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference passed. Final tool-only tests/Clippy also cover filename sorting.
Command-level no-DEX and malformed-DEX archives refused with empty stdout;
wrong field type produced zero sites. The shared caller search reproduced all
134 previous BNm call-site records; the first byte-for-byte check differed
because ZIP filename iteration order varies, so record sets were compared.
The new field-search command sorts DEX names to make its output reproducible.
No live account or identity-change test was performed in this change.
Two sorted field-search runs matched byte-for-byte. Repository hygiene passed
with 334 local links; final staged whitespace checks passed.

Fingerprint selection follow-up (static, 2026-10-07): APK 37.71's captured/contact
gate selects native V3/V4; the later fresh read selects PN inclusion. Own-key
companion inclusion, empty-to-null strings, stored own username, remote `@`
prefixing and nullable `is_pn_shared` storage are now traced. The earlier
username-availability hypothesis for that Boolean was refuted. Smali branch
annotations now show signed jumps, with regression coverage. Current feature
values, authenticated input producers and native V3 comparison remain open;
no current account fingerprint or verification action occurred.
[Selection evidence](docs/protocol/safety-number.md#scheme-username-and-pn-sharing-input-branches-2026-10-07).

Validation (offline): 556 workspace/all-feature tests and 244 default-feature
tests passed. Strict workspace/all-target/all-feature Clippy, warning-free
Rustdoc, formatting and CLI reference regeneration/test passed. Fresh APK
method output confirms both backward jumps and matches the scoped selection
excerpt. No account state was opened or changed and no live request was sent.
Repository hygiene passed with 339 local links; staged whitespace checks passed.

Native V3 comparison follow-up (static, 2026-10-07): selector 19 now resolves
through the relocated JNI table to 0x71373c, with Java options 1/3 and distinct
PN-or-LID orientation, conversion errors, PN/username presence checks, hosted
omission and result remapping. Generation and scan selection use separate
feature flags. Native execution, MCFEqual/JID semantics and binary comparison
implementation remain open; no account state or live request changed.
[V3 evidence](docs/protocol/safety-number.md#native-v3-scan-dispatch-and-comparison-branches-2026-10-07).

Validation (static/offline): APK and reconstructed-library hashes matched the
pinned values; the option extraction returned one method. Independent fresh
native extraction matched all 424 converter/helper/core instructions in the
scoped snapshot. CLI reference regeneration/test passed; repository hygiene
passed with 344 local links and staged whitespace checks passed. This changes
research/docs only; native execution and live account validation are not claimed.

V3 native execution follow-up (offline, 2026-10-07): 264 synthetic cases now
execute the actual V3 core/optional-value/PN-or-LID helpers and remapper,
validating the traced presence asymmetry, option bits, hosted omission and
error ordering. Ten V3/shared-V4 tests passed, preserving existing V4
observations. Generic object equality and constructors remain modeled;
JID/equality semantics, decoder/orientation integration and runtime V3 binary
comparison remain open. No account or network state changed.
[V3 corpus](docs/protocol/safety-number.md#executed-v3-core-and-remapper-corpus-2026-10-07).

Final validation: CLI reference regeneration/test and OpenAPI library-symbol
check passed. Repository hygiene checked 349 local links; staged whitespace
checks passed. All ten V3/shared-V4 tests passed again after restricting hooks
to the selected scheme. Rust runtime code was unchanged; no live validation
is claimed for this offline research-tool change.

V3 equality/LID follow-up (static, 2026-10-07): registered ASCII equality is
length-sensitive byte equality, including case-sensitive usernames. JID
equality uses the constructed full string. The scanned LID path parses and
flattens device-zero objects before requiring primary category and LID kind;
nonzero companions fail that guard. Exact validator bounds are preserved.
Actual parser/class-runtime execution and full binary V3 comparison remain
open; no account or live action changed.
[Equality/LID evidence](docs/protocol/safety-number.md#v3-equality-callbacks-and-primary-lid-conversion-2026-10-07).

Validation (static/offline): APK, main and merged hashes matched the pinned
build. Fresh extraction matched all 18 ASCII comparator instructions. CLI
reference regeneration/test passed, repository hygiene checked 354 local
links and staged whitespace checks passed. This change adds attributed research
only; it does not execute the native parser or change runtime/account behavior.

V3 primary-LID execution follow-up (offline, 2026-10-07): 38 synthetic inputs
now execute native parser routing, validators, constructors, device-zero
flattening and kind selection. They confirm explicit zero suffix normalization,
nonzero companion rejection and exact base-number bounds within modeled MCF
primitive boundaries. Four utility tests passed. Native MCF/class/ownership
behavior and full binary V3 comparison remain unfinished; accounts unchanged.
[Executed LID decisions](docs/protocol/safety-number.md#executed-v3-primary-lid-parser-decisions-2026-10-07).

Final validation: CLI reference regeneration/test and OpenAPI library-symbol
check passed. Repository hygiene checked 359 local links and staged whitespace
checks passed. This adds an offline research utility and observations; Rust
runtime behavior and saved accounts are unchanged.

V3 client comparison follow-up (2026-10-07): explicit `compare_v3` and
`fingerprint-compare --scheme native-v3` now compare bounded binary payloads
using the APK's V3 converter/core/orientation/remapping. Optional `--v3-hosted`
selects APK options 3; otherwise 1. V3 permits PN-only known identities and
uses case-sensitive usernames. Native MCF splitting is statically confirmed.
The port matches 263 non-abort core and 38 LID branch observations; current
input selection, native MCF/JNI/ownership edges and real peer scan remain open.
No account or live action changed; V3 QR export remains unimplemented.
[Runtime comparison](docs/protocol/safety-number.md#explicit-native-v3-binary-comparison-in-the-client-2026-10-07).

Validation: workspace all-feature tests (561), default workspace tests (249),
strict all-target/all-feature Clippy, warning-free Rustdoc, native V3 utility
tests (4), CLI reference regeneration and formatting passed. Synthetic CLI
comparison confirmed options 1/3 and V3 case-mismatch -25 versus default V4 0,
with redacted metadata and no account-store creation.

Repository hygiene passed: 364 local links, tracked-file budgets, package
licenses and research checksums; staged whitespace checks passed.

V3 export follow-up (2026-10-07): explicit `encode_v3` and
`fingerprint-display --qr-scheme native-v3 --qr-version 0|1 --qr-output` now
produce V3 comparison bytes, preserving both PN fields with usernames. Static
combined/individual serializer instructions confirm field/order differences.
Independent Python wire/hash fixtures cover both versions, optional addresses,
zero suffix preservation and both comparison orientations. CLI V3 display/export
requires stable PN display input; library export also accepts LID-only sides.
Current feature/inventory selection and real-peer interoperability remain open.
No account or network action changed.
[V3 export evidence](docs/protocol/safety-number.md#explicit-native-v3-binary-export-2026-10-07).

Validation (offline): 564 workspace/all-feature tests and 252 default-feature
workspace tests passed. Strict workspace/all-target/all-feature Clippy,
warning-free Rustdoc, generated CLI reference, formatting and repository hygiene
(367 local links) passed. The initial workspace run preceded doc regeneration
and failed only its stale-reference test; after regeneration the full rerun
passed. A built-CLI synthetic PN-only export in versions 0/1 matched the exact
independent fixture, created mode-0600 files, compared with V3 status zero and
refused overwrite without changing bytes. No account store was created.

Ordinary document receive extension (2026-10-07): APK 37.71 BZB schema and
distinct document-key domain now support URL-bearing documents, sealed media
references and durable receive events. New media-save explicitly exports to a
create-only private operator path; filenames remain metadata. Document playback
refuses and names that command. Independent crypto/wire/store fixtures cover
authentication, restart, deduplication, labels and overwrite/symlink refusal.
Live incoming documents, directPath-only retrieval, large files and sending
remain unfinished. [Evidence and limits](docs/protocol/document-media.md).

Validation: 567 workspace/all-feature tests and 252 default-feature workspace
tests passed, plus strict all-target/all-feature Clippy, warning-free Rustdoc,
generated CLI reference, formatting and repository hygiene (370 local links).
The first Rustdoc run rejected an unescaped <clinit> in the new API comment;
after formatting that name as code, its rerun passed. No runtime change was
needed. Focused document/schema/crypto/restart/export tests passed.

A bounded 45-second receive-only check authenticated/activated and stopped
cleanly. Events/receipts/actions/outbox summaries were unchanged (20/9/12/0),
with the prior uncertain action preserved. No fresh encrypted message/document
arrived; live document retrieval is still unverified. Private backup/log paths
and notification observations are recorded in the document protocol page.

Group-notification follow-up (2026-10-07): capture inspection now preserves the
APK's exact w:gp2 label instead of hiding it because of its digit. Saved
create/groups_dirty shapes are visible with values redacted. Static 0yv.A09
shows groups_dirty queues FetchTruncatedGroupsJob; request/application and ACK
policy remain unfinished. No notification or membership was applied and no
network request was made. [Evidence and next boundary](docs/protocol/groups.md#saved-group-notifications-and-inspection-correction-2026-10-07).

Validation: 568 workspace/all-feature tests, 24 default-feature CLI tests,
42 final all-feature CLI tests, strict CLI all-target/all-feature Clippy,
CLI reference regeneration/test, formatting and repository hygiene (374 local
links) passed. Rebuilt-CLI saved-account inspection exposed the previously
hidden group type without network I/O, replay or notification application.


Group refresh serializer follow-up (2026-10-07, **offline/static**): APK
37.71 context A05 is truncatable dirty recovery. The concrete `AaH` →
`Aib` case 25 → `9QE`/`EJy` IQ constructor uses `w:g2`, `type=get`,
`query[context]` and per-group JID children; the feature-gated alternative
uses MEX `QueryBatchGetGroups`, `groups` and `request_context`.
Inherited `0yO.A01` dispatch explains the zero concrete-handler caller result
and reaches a queued/direct ACK boundary. Complete ACK serialization,
response/application guards and current feature values remain untraced;
notification implementation remains unchecked. The disassembler now names the
literal double multiply/divide used in the memory budget. See
[request/dispatch evidence](docs/protocol/groups.md#dirty-refresh-request-and-inherited-dispatch-trace-2026-10-07).
32 xtask tests, strict all-target xtask Clippy and an actual APK re-dump passed.
No account state was opened or live request sent for this research.

Final checks for this change: generated CLI reference test, all five OpenAPI
coverage/HTTP/library-symbol checks, formatting, staged whitespace and repository
hygiene (379 local links) passed. These are local checks, not live group refresh
validation.


Batch response/ACK follow-up (2026-10-07, **offline/static**): the 37.71
writer is Cp2.A05 reached from selector 76 through 0y2.A02. The response
parser distinguishes full/truncated/error groups and checks request-correlated
attributes. ID guards check string length; request child-count diagnostics log
without throwing. A truncated result is not a complete group snapshot. Full
membership commit, dirty-state clearing and inbound ACK-route production remain
untraced; notification handling remains incomplete. Empty apk-smali results now
name the active class/method filters without falsely claiming class absence.
32 xtask tests, strict Clippy and the actual APK diagnostic smoke passed.
[Evidence and remaining boundary](docs/protocol/groups.md#batch-response-and-current-build-ack-writer-2026-10-07).
No account state was opened or live request sent for this trace.

Final local checks for this trace: generated CLI reference, five OpenAPI
coverage/HTTP/library-symbol tests, formatting, staged whitespace and repository
hygiene (383 local links) passed. No live group notification application was
validated.


Truncated group snapshot correction (2026-10-07, **offline**): direct and
queued info reply ingestion now refuse truncated group data before replacing
members or emitting removal events. The original snapshot/events and saved
reply survive restart; no ingestion marker or automatic retry is created.
Invalid non-boolean truncation flags also refuse as local policy. APK A9s/8lQ
separate full and truncated processing; the underlying full membership commit
and dirty-state database semantics remain untraced. Notification handling is
still incomplete. [Correction and evidence](docs/protocol/groups.md#truncated-snapshot-protection-and-completion-boundary-2026-10-07).
571 workspace/all-feature and 24 default CLI tests, strict Clippy and Rustdoc
passed. No account was opened or live request sent.

Final checks: generated CLI reference, all five OpenAPI coverage/HTTP/library
checks, formatting, staged whitespace and repository hygiene (387 local links)
passed. The added non-text-flag regression also passed independently.


Atomic queued group ingestion (2026-10-07, **offline**): the once-only marker
now commits with the snapshot/events, invite update or leave deletion. An abort
and encrypted-store reopen test proves rollback across all three paths, then
one successful application and no repeat after another reopen. Truncated replies
still preserve state without an ingestion marker. Two APK completion helpers
include gated state handling and member-suggestion sync, not proven generic
dirty-state deletion. Full group notification parsing/application remains open.
[Atomic ingestion and helper evidence](docs/protocol/groups.md#atomic-queued-group-ingestion-2026-10-07).
No account state was opened or live request sent.

Validation: 572 workspace/all-feature tests, 24 default-feature CLI tests,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated
CLI reference, five OpenAPI checks, formatting, staged whitespace and repository
hygiene (390 local links) passed. Group notification handling remains incomplete.


Typed group participants (2026-10-07, **offline**): the snapshot parser now
keeps supported user-level JIDs and applies APK linked-map duplicate behavior:
one entry per JID, last role wins, first position retained. Refused participant
entries remain excluded. The saved snapshot and GroupCreated journal regression
covers typed/text duplicates and missing/invalid/non-user/device entries.
APK alias/name/history/country metadata producers are traced but not applied;
role-change events and full group notification handling remain incomplete.
[Participant evidence](docs/protocol/groups.md#typed-participant-map-and-duplicate-roles-2026-10-07).
No account was opened or live request sent.

Validation: 573 workspace/all-feature tests, 24 default-feature CLI tests,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated
CLI reference, five OpenAPI checks, formatting, staged whitespace and repository
hygiene (393 local links) passed. No live participant-map validation occurred.


Notification input trace (2026-10-07, **offline**): base handler BFC produces
notification ACK metadata through CmG.A02/A00 using incoming from/id/participant/type.
Receipt-only recipient/category/edit extraction is a different wrapper. Normal-edge
analysis confirms dirty refresh round zero and zero per-group integer; its meaning
and exception paths remain unknown. Alternate executor and upstream group dispatch
remain untraced; runtime group notification completion is still open.
[Producer and round evidence](docs/protocol/groups.md#notification-metadata-producer-and-initial-refresh-round-2026-10-07).
No account store was opened or live request sent.

Validation: all five OpenAPI checks, generated CLI reference, whitespace and
repository hygiene passed. No runtime behavior changed or live validation occurred.


Queued group replies now apply during receiving (2026-10-07, **offline tested**),
instead of waiting for `group list`. Settlement stores a durable projection marker;
startup recovers only these newly marked replies once. Truncated/invalid group
responses remain saved and diagnosable without blocking unrelated receiving;
storage failures still stop the session. No request is replayed or historical
account state backfilled. [Session projection](docs/protocol/groups.md#apply-queued-replies-during-receiving-2026-10-07).
One bounded 30-second live group-info check settled ok with a matching saved group
reply and four-member snapshot updated during receiving. Group list afterward
applied zero replies. No chat outbox sends occurred; the previously unresolved
two-factor action remained sent. Existing account identity, keys and config were
preserved. Unsolicited group notifications/batch refresh and live recovery remain
unverified.

Validation: 575 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, CLI reference
regeneration and formatting passed. Bounded live snapshot projection passed;
unsolicited group notification completion remains open.

Final OpenAPI checks (five), staged whitespace and repository hygiene
(399 local links) also passed.


Dirty batch partition/round trace (2026-10-07, **offline**): corrected missing
23x binary operands in xtask and independently checked raw job bytes. The APK
partitions by accumulated participant estimate, carries returned truncated sizes
with fallback to the previous estimate, and computes a bounded round limit when
none is provided. The all-zero dirty input derives bound 2, subject to outcomes.
The alternate notification executor also reaches application-before-ACK.
[Evidence and remaining boundaries](docs/protocol/groups.md#batch-partition-and-bounded-follow-up-rounds-2026-10-07).
Automatic dirty refresh/error/rate-limit application remains unfinished; no
account state was opened or live request sent for this trace.

Validation: 576 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated CLI
reference, five OpenAPI checks, formatting and repository hygiene passed.
Client wire behavior is unchanged by this disassembler/research change.


Classic batch refresh (2026-10-07): `group refresh --chat <group-jid>` (repeat
--chat) queues one classic per_group_dirty_recovery IQ. Complete groups/group
responses project only when all requested identities occur once, atomically with
events and the projection marker. Incomplete/truncated/error-bearing batches
preserve all snapshots and saved responses. [Batch queue and limitation](docs/protocol/groups.md#classic-batch-queue-and-complete-response-projection-2026-10-07).
Mixed-union application, follow-up rounds and automatic dirty notification
scheduling/ACK remain unfinished.

One two-group classic batch was live verified in a bounded 30-second receive
session: ok, two matching saved results and both snapshots updated during the
window (one/four members). Group list afterward applied zero actions. No chat
intent was sent; the unresolved two-factor action remained sent. Identity/keys
and existing request IDs were preserved. Mixed/truncated continuation remains
unimplemented and unverified. 577 workspace/all-feature and 24 default CLI tests,
strict Clippy, warning-free Rustdoc, CLI reference and five OpenAPI checks passed.

Final staged whitespace and repository hygiene (407 local links) passed.


Mixed batch projection (2026-10-07, **offline tested**): full snapshots now commit
beside APK-recognized truncated(size 0..19999), forbidden(403) and not-found(404)
results. Partial items preserve existing membership, and a durable outcome commits
with full updates/events and projection markers. `group refresh-status --id`
inspects only counts without queueing. [Union evidence and remaining work](docs/protocol/groups.md#mixed-batch-projection-and-outcome-inspection-2026-10-07).
No live query was repeated or saved account opened. Bounded follow-up and dirty
notification handling remain unfinished; mixed/report behavior is not live verified.

Validation: 579 workspace/all-feature and 24 default CLI tests, strict Clippy,
warning-free Rustdoc, regenerated CLI reference, all five OpenAPI checks,
formatting and repository hygiene passed. Mixed/report live validation remains
unverified.

Opt-in bounded group refresh continuation (2026-10-07, **offline tested**):
`group refresh --follow-up` uses the APK truncatable context and rounds 0..2.
Recognized results atomically project/report and queue one distinct next query;
uncertain, refused and unsupported results never continue. Original input order
and unsent partitions persist; local budget 10000 is conservative, not a claim
about APK runtime memory/features. Reports expose round, next action, pending
count and exhaustion. Default manual refresh remains single-query.
[Evidence and boundaries](docs/protocol/groups.md#opt-in-bounded-refresh-continuation-2026-10-07).
No live query repeated. Live continuation, unsolicited dirty scheduling/ACK and
APK error/wait helper handling remain unverified or unfinished.

Validation: 583 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, regenerated CLI
reference and formatting passed. Transaction/reopen and old-report inspection
checks are offline; live truncated continuation remains unverified.

Final five OpenAPI checks, whitespace and repository hygiene (413 local links)
also passed. Private account state and conversation exports remain untracked.

Automatic supported dirty-group notifications (2026-10-07, **offline tested**):
`listen` now schedules the bounded truncatable refresh atomically with a durable
source/ID deduplication marker before acknowledging w:gp2/groups_dirty. Duplicate
supported delivery repeats only the ACK; uncertain queries retain their IDs and
reservations. Unsupported/empty/conflicting inputs stay captured without ACK and
have redacted diagnostics. `actions` and `group refresh-status` inspect progress.
[Evidence and local restrictions](docs/protocol/groups.md#automatic-supported-dirty-notifications-2026-10-07).
No account opened or live request sent. Live dirty input/ACK, other notification
types and APK error/wait handling remain unverified or unfinished.

Live supported dirty notification projection subsequently passed in one bounded
45-second authenticated receive session: zero queued actions/chat intents at
preflight; one notification-derived refresh settled ok and committed one full
group at round 0, no pending or follow-up work. Session stopped cleanly; no
incoming user message or chat send. The unresolved two-factor action stayed sent.
ACK writing completed without session error, not independent server ACK proof.
Live truncated/deduplication/failure recovery remains unverified.

Validation: 586 workspace/all-feature and 24 default CLI tests, strict Clippy,
warning-free Rustdoc, regenerated CLI reference, five OpenAPI checks, formatting
and whitespace passed. The corrected malformed-JID fixture and full rerun are
recorded in protocol docs.

Final OpenAPI rerun, whitespace and repository hygiene (417 local links) passed.
Private receive evidence and conversation exports remain excluded from Git.

Owner steering (2026-10-07): comprehensive APK-derived Rust behavior and quick
release-to-release file comparison now drive the work queue. Normal onboarding,
messaging, identity/state preservation and honest live boundaries remain required.
[Porting and release workflow](docs/protocol/apk-porting.md).

Version-aware research export now accepts an exact-hash JSON recipe with stable
topic/class/optional method filter and affected Rust paths. The committed 37.71
six-topic group baseline has 11 files/391,666 bytes; initial extraction took
16.768 seconds. `apk-research-diff` checksum-verifies cached topic files and links
structural changes to evidence/Rust paths. Three real cached self-comparisons
returned six unchanged topics in 0.0183..0.0215 seconds. Added/removed/changed,
provenance-only changes and tampering are covered by distinct synthetic fixtures.
This is scoped evidence comparison, not complete APK coverage or automatic
semantic remapping/translation. Only the current APK was found in Downloads/lab;
the committed 37.70 snapshot remains available, but its group mapping is absent.

Subject freshness trace confirms a separate notification-version cache;
ordinary snapshot s_t is not proven equivalent. The parent propagation helper is
not the main subject writer. Application, alias effects and full cache lifecycle
remain unfinished. No saved account opened or live request sent for this change.

Final validation: 588 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, CLI reference,
five OpenAPI checks, formatting, staged whitespace and hygiene (425 local links)
passed. Exact 37.71 snapshot regeneration with --check passed in 17.467s. Final
cached reports (six topics plus four metadata/dictionary artifacts) took
0.0200..0.0213s; all unchanged in the real self-comparison.

Anchor-assisted APK release maintenance (2026-10-07): `apk-research-remap`
finds exact-string co-occurrences while preserving method versus whole-class
scope. Ambiguous, missing, split-scope and unanchored topics cannot produce a
recipe; existing proposals cannot be overwritten. Confirmed static scan of
2.26.37.71 uniquely recovered all six reviewed group targets in 5.225 seconds.
A real split-scope negative probe refused output; adding anchors changed only
metadata, with no code-topic differences. The baseline is now 392,185 bytes.
Broader APK/native coverage and a distinct next-release comparison remain
unfinished. No account state was opened or live request sent for this work.

Validation: 590 workspace/all-feature tests and 24 default CLI tests passed,
as did strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting and exact baseline
regeneration with --check.
Repository hygiene passed: tracked-file budgets, 427 local links, package
licenses and reference research checksums; staged whitespace passed.

Confirmed static APK 37.71 subject-cache follow-up: the post-commit Q4N selector
16 calls 118.A01 to remove the group from the cached version map. 118.A02
insertion preserves supplied versions and initializes participant_version=0;
the subject writer's update-miss branch supplies zeros for the other versions.
Literal DEX timestamp multiplication/division bytes are independently verified,
and the decoder now names all standard 90..af typed binary operations. The
versioned group recipe gains an insertion topic. This establishes research
boundaries, not a subject receive handler; full lifecycle/type/alias/system-message
parity remains open. No saved account opened or live request sent.

Validation: 591 workspace/all-feature tests and 24 default CLI tests passed,
plus strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting and exact snapshot
regeneration (--check, 12 files/395,753 bytes). All seven targets resolved uniquely.
Repository hygiene passed (429 local links, tracked-file budgets, licenses
and reference checksums), as did staged whitespace.

APK 37.71 complete-group classification is now persisted in Rust: parent,
default subgroup, general chat, linked subgroup, ordinary, with APK child
presence precedence and separate linked-parent identity. group list --json
exposes it; historical rows/local stubs remain null until a complete reply.
The actual snapshot API schema is corrected. Offline precedence, invalid-parent,
truncated and encrypted-store historical/reopen checks passed. Alias research
traces through 17H/0Xw to transactional contact mapping, not Signal pin replacement;
remaining storage/cache/callback bodies and subject handling are unfinished.

Observed bounded live classification check: one new info query for an already
known group settled ok and projected ordinary/null-parent with four members.
The unqueried historical row stayed unknown; zero queued actions/chat intents
remained after the clean 30-second session. The unresolved two-factor action
remained sent. Other group types and subject handling remain live unverified.
The eight-topic recipe has seven unique remaps and one two-candidate ambiguity;
proposal writing refuses rather than guessing. Validation: 593 workspace/all-
feature and 24 default CLI tests, strict workspace/all-target/all-feature Clippy,
warning-free Rustdoc, CLI reference, five OpenAPI checks, formatting and exact
snapshot regeneration (13 files/397,778 bytes) passed.
Repository hygiene passed (430 local links, tracked-file budgets, licenses
and reference checksums); staged whitespace and final OpenAPI rerun passed.

APK release maintenance now supports bounded method caller_anchors with full
cross-DEX signatures, preserving prefilter candidates/call sites and refusing
missing/split/noninvoking caller evidence. In APK 37.71, group_response caller
metadata resolves the classifier's ADs.A01/AZK.AAK ambiguity without old-name
fallback: all eight topics now unique (10.012s) and a new proposal was written.
Eight code topics remained unchanged in cached comparison; only target metadata
changed (baseline 397,832 bytes). Indirect/native dispatch and a distinct next
APK remain unverified. No saved account opened or live request sent this turn.

Validation: 595 workspace/all-feature and 24 default CLI tests, final strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, CLI reference,
five OpenAPI checks, formatting and exact snapshot regeneration passed. All
three actual caller-negative probes exited 1 without creating proposals.
Repository hygiene passed (432 local links, tracked-file budgets, licenses
and reference checksums); final staged whitespace checks passed.

APK 37.71 storage trace now separates group contact display-name writes from
row-ID JID mappings and closes alias cache commit/rollback cleanup. Full DEX
constructor signatures disambiguate four identical shorties; downstream listener
semantics, subject message parity and complete nested-transaction behavior remain
unknown. No Signal key replacement is established by this path. The group recipe
now has eleven unique remaps and a 16-file/423,242-byte baseline; three topics
were added and eight previous code topics stayed unchanged. Same-APK comparison
is coverage expansion, not a verified upstream release change. No saved account
opened, chat queued, or live request sent during this static work.
Validation: 40 xtask tests, five OpenAPI checks, regenerated CLI reference,
exact 16-file snapshot reproduction, all eleven selector comparisons and
repository hygiene passed. The unchanged runtime Rust code was not retested
across the whole workspace for this source-evidence change.
Final staged hygiene passed (435 local links, budgets, licenses and reference
checksums); staged whitespace and the five OpenAPI checks passed again.

APK-to-Rust release work now includes a reviewed 22-topic core 37.71 baseline
(27 files/731,482 bytes/150 methods) and a genuine cached 37.70→37.71 difference
in 0.0512 seconds. Fourteen old-topic relocations were unique; three ambiguous
and four unanchored selections required manual review. Automatic proposal
writing refused without creating an output. Text changes include obfuscation
and annotations and do not establish nineteen behavior changes.

Confirmed static APK 37.71 `1Rl.<clinit>` indexed stores exposed four corrupted
historical comparison token slots. The raw-byte heuristic is replaced with
bounded typed initializer evaluation. The runtime now uses a Derived 37.71
fixture, independently matching all 1,260 held web-v3 entries. Historical 37.70
artifacts remain immutable; the old raw APK is unavailable. Literal wire-byte
regressions cover all four corrected slots. Saved keys, reservations and event
markers are preserved; old projections are not silently replayed. See
[core review](docs/protocol/core-release-review.md) and
[token extraction](docs/protocol/token-extraction.md).

Validation: 598 workspace/all-feature tests, 24 default CLI tests, 42 final xtask
checks, strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting and exact core/group
snapshot reproduction passed. Final review also added a regression refusing a
const operand that resembles a final return opcode. A bounded 30-second live
receive session authenticated and exited 0; one automatic group refresh settled
ok, all sixteen existing outcomes stayed unchanged, and no queued actions or
eligible chat intents remained. No chats were sent. Individual corrected token
slots remain live unverified; fourteen unsupported incoming classifications are
recorded in the token extraction document. The two-factor sent action remains
unresolved. Full APK/client parity remains unfinished.

APK-to-Rust coverage now traces required subject timestamps to signed Long
parsing, cached group_type to its database column, and separate ordinary/parent
system-message construction. Fifteen group topics map to Rust ownership paths;
full subject application/message storage remains unfinished. A reusable checked
method_shorty recipe constraint resolves the three diagnostic-matching timestamp
parsers without obfuscated-name fallback, retaining prefilter evidence. Thirteen
topics remap uniquely; two string-free helpers still require manual relocation.
Wrong/malformed constraints refuse; whole-recipe proposal writing remains blocked
by those explicit unknowns. Signed 21s constants now show raw hex and negative
meaning, with independently checked APK insert-sentinel bytes.

The group baseline is 20 files/430,619 bytes; a cached comparison takes 0.0227
seconds (four topics added, two annotation-only changes, nine unchanged). The
core baseline remains byte-identical. No saved account opened or live request
sent during this static work. See [typed subject trace](docs/protocol/groups.md#subject-typed-branches-and-signed-timestamp-contract-2026-10-07).

Validation: 600 workspace/all-feature tests, 44 final xtask tests, 24 default
CLI tests, strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting, exact 20-file group
snapshot reproduction and byte-identical core comparison passed. Actual wrong
signature, malformed parameter and missing-method probes refused as documented;
no proposal was created for either unresolved full mapping or wrong signature.
Repository hygiene passed (467 local links, budgets, licenses and reference
checksums), with clean staged whitespace. Runtime/client behavior is unchanged
by this research-tool and source-coverage checkpoint.

Further APK 37.71 subject recovery tracing establishes a material runtime gap:
0Xm.A0F dispatches normal group info through MEX QueryGroupInfo, while Rust's
working group-info command uses classic IQ. Exact ten-reason context mapping,
private-set override, mode-5/6 in-flight guard and the separate 60,000-ms local
monotonic rate guard are preserved, including its confirmed map write. Error
429 is traced to an A0G time update and an untraced runnable, not a proven retry
wait. Envelope/document ID, response conversion and complete callbacks remain
unknown; no MEX endpoint or subject handler is claimed implemented.

Group source coverage is now twenty topics, 25 files/448,127 bytes. Sixteen
remaps are unique; four string-free helpers remain unresolved automatically.
Cached same-APK comparison took 0.0204 seconds (five added, fifteen unchanged).
No saved account opened or live request sent. See [recovery trace](docs/protocol/groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07).

Validation: 44 xtask tests, five OpenAPI checks, regenerated CLI reference,
exact 25-file/448,127-byte snapshot reproduction, current twenty-topic remap,
repository hygiene (477 local links, tracked-file budgets, licenses/reference
checksums) and clean staged whitespace passed. Shared runtime Rust was unchanged
and was not retested across the workspace for this source-only follow-up.

APK 37.71 MEX request construction now has an offline Rust port. Selected
QueryGroupInfo persisted IDs export as a small version/hash-scoped file; exact
server ID appears in trace, query attribute and nested JSON. The release CLI
checks the real APK export; an offline preview invokes the actual Rust builder.
This advances APK-to-Rust behavior and file-based release patching. It does not
queue a MEX action or overwrite group state; complete JSON/Argo/Pando response
handling and durable dispatch remain unfinished. Namespace initialization,
assertion failure semantics and timeout units remain explicit unknowns. See
[MEX source/port](docs/protocol/mex.md). No saved account opened or live request sent.

Validation: 604 workspace/all-feature tests and 24 default CLI tests passed;
final duplicate-ZIP correction passed all 47 xtask tests. Strict workspace
Clippy and warning-free Rustdoc passed, with the corrected xtask rechecked.
CLI reference was regenerated; all five API checks passed. Real APK export
including provenance matched the fixture after the correction; the Rust node
preview and five negative CLI probes passed (refusals emitted no partial stdout).
Repository hygiene checked 487 local links, tracked budgets, licenses and
reference checksums; formatting and staged whitespace passed. No account
opened, live request sent or existing saved state changed.

MEX JSON response provider 0x1763 is now traced through the actual DEX switch
entry to O4r selector 0. Nullable object data and array errors parse separately;
valid counterparts survive malformed fields. Rust reproduces these response
parts with explicit invalid states and redacted inspection, without inventing
Java exception strings or treating an IQ result as group success. The subject
getter is now confirmed as subject/8yq. Full callback/error schema, Argo/Pando,
group conversion and durable MEX dispatch remain unfinished. Sixteen synthetic
field-type combinations cover both result/update. See [parser trace](docs/protocol/mex.md#json-response-field-port-2026-10-07).
No saved account opened or live request sent in this work.

Validation: 607 workspace/all-feature tests, 24 default CLI tests, all 47 xtask
tests, regenerated CLI reference and five OpenAPI checks passed. Strict
workspace/all-target/all-feature Clippy, warning-free workspace Rustdoc and
formatting passed.
Five positive and three refusal CLI probes preserved counts/types without
content or partial failure output. The twelve-topic MEX baseline reproduced
exactly (17 files/58,761 bytes); cached comparison took 0.0064 seconds.
Remap has six unique/four unanchored/two ambiguous mappings, with no complete
proposal claimed. Repository hygiene and staged whitespace passed. No account
opened, outgoing intent queued or live request sent.

MEX group callback gate is now ported: supported partial data with nonempty
error objects blocks the success callback; nonobject error entries and absent
model are explicit separate states. Nonempty error-free data can invoke the
callback, but its feature gate and complete group conversion/application are
unfinished. --group-info exposes the redacted disposition offline. Direct DEX
superclass inspection closed the inherited 9Np/BaseMexCallback C2c path;
IWF error fields and partial ADv getters remain source-only where not ported.
See [callback trace](docs/protocol/mex.md#group-callback-gate-and-inherited-dispatch-2026-10-07).
No account opened, intent queued or live request sent.

Callback-gate validation: workspace/all-feature tests **609 passed**, default
CLI **24 passed**, CLI reference regeneration, OpenAPI library-symbol check,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc and
formatting passed. Actual CLI probes covered **20** callback cases across
result/update plus **10** default-mode checks with redacted output. The existing
37.71 MEX bundle reproduced exactly (**17 files, 58,761 bytes**). No live
account request or group-state application occurred in this chunk.

MEX callback source coverage is now versioned with the request/parser files:
18 topics, 23 files/76,908 bytes, exact APK reproduction passed. Cached
comparison against Git cdd4056 took 0.0069 seconds: six added/twelve unchanged
topics on the same APK. Remap: eight unique/seven unanchored/three ambiguous;
is_retryable/shorty-Z still matches three methods and needs review. Full group
conversion, Argo/Pando and durable live application remain unfinished.
All 48 xtask tests passed; no Rust behavior changed or account request sent.
See [versioned MEX files](docs/protocol/mex.md#versioned-parserrequest-files).

Coverage validation also passed exact checksum membership/18 single-method
selections, CLI reference regeneration, OpenAPI library-symbol validation,
repository hygiene (518 local links) and staged whitespace.

MEX converter static tracing now preserves full 0Xo.A03's four hash branches,
required group/participants/properties versus nullable subject, separate
1000-scaled group/subject times and unscaled announcement version, plus
feature-gated LID/country creator metadata. Wire type names, A4m slot mapping,
Java coercion/catch behavior and full membership conversion remain unfinished.
The disassembler now names the independently verified mul-long/2addr opcode;
this adds no runtime state application or live request. See [converter trace](docs/protocol/mex.md#group-converter-branches-and-time-arithmetic-2026-10-07).

Converter-trace validation: **610** workspace/all-feature tests (including
**49** xtask tests), strict workspace/all-target/all-feature Clippy, formatting,
CLI reference regeneration, OpenAPI library-symbol check, exact MEX snapshot
reproduction (23 files/76,908 bytes), hygiene (523 links) and staged whitespace
passed. No saved account opened, queued intent or live validation performed.

MEX result constructor mapping now confirms group type A03, distinct long
slots and participant/creator maps, preserving unknown flag/getter meanings.
The long parser's NumberFormatException catch returns the supplied zero;
reflection fallback catches four named exceptions, not arbitrary Throwable.
apk-smali --exceptions exposes checked code-unit ranges/types/targets, with
independent APK code_item regression. Complete numeric coercion, participant
conversion and durable group application remain unfinished. No live request.
See [constructor/catch trace](docs/protocol/mex.md#result-constructor-and-explicit-exception-inspection-2026-10-07).

Constructor/catch validation: **615** workspace/all-feature tests, including
**54** xtask tests, strict workspace/all-target/all-feature Clippy, warning-free
Rustdoc, formatting, CLI reference regeneration and OpenAPI library-symbol
validation passed. Actual CLI probes verified the typed numeric catch, four
reflection catches, explicit zero-range result, unchanged default output and
named-file/no-partial-output refusal for malformed later DEX. All three 37.71
bundles reproduce: MEX 23 files/76,908 bytes; groups 25/448,127; core 27/731,494.
Core's one prior op=bd unknown annotation was refreshed with identical opcode,
offsets and operands (one changed/21 unchanged topics, four unchanged artifacts,
same APK; cached presentation diff 0.0328 seconds). Hygiene (533 local links)
and staged whitespace passed. No live request or saved account mutation.

MEX numeric group conversion is now in Rust: distinct signed creation/subject
numbers scaled by 1000 and unscaled announcement version, zero fallback and
wrapping arithmetic. Independent host Java reference covers 479 cases and
all UTF-16 code units (370 decimal digits/37 BMP blocks); Android runtime/SDK
and full JSON lexical parity remain unverified. --group-numbers with
--group-info explicitly prints private numbers as decimal strings; default
summary/Debug remain redacted. Full membership/type/feature/state application
remains unfinished. No live request or saved account mutation. See [numeric
projection](docs/protocol/mex.md#numeric-group-projection-into-rust-2026-10-07).

Numeric-port validation: **618** workspace/all-feature tests, **24** default
CLI tests, strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
formatting, CLI reference regeneration and OpenAPI library-symbol validation
passed. Actual CLI probes: **70** numeric cases across result/update, **70**
unchanged redacted modes, flag dependency and six content-free/no-partial-output
refusals. Oracle runtime was checked directly as Temurin **21.0.12.1+1 LTS**;
479 corpus cases and all 65,536 UTF-16 units matched that independent reference.
Hygiene (540 local links) and staged whitespace passed. No saved account,
identity, action state or live server request was changed.

MEX participant roles now convert in Rust from nullable server strings using
APK 37.71 enum/fallback and exact member role mapping; missing/unknown versus
ordinary member stay distinct. Offline `mex-role-inspect` invokes that helper
without echoing input. Independent host Java reference contains 91 cases;
full Android Unicode-version parity remains unverified. The same trace
confirms phash retain semantics and separate BotJid/UserJid participant paths;
full member conversion and durable application remain unfinished. See
[role/participant evidence](docs/protocol/mex.md#participant-role-conversion-and-member-map-boundaries-2026-10-07).
No saved account, identity, queued action or live request changed.
Role-port validation: 619 workspace/all-feature tests and 24 default CLI tests,
strict Clippy, warning-free Rustdoc, formatting, CLI reference and API symbol
checks passed. Actual CLI matched 90 representable oracle inputs plus omission;
the embedded-NUL case is library-tested because OS argv cannot carry NUL.
Hygiene checked 548 local links and staged whitespace passed.

APK release comparison now includes 38 MEX methods / 43 files / 162,435 bytes,
with opt-in typed/catch-all exception tables. Default-format exports remain
unchanged. Actual cached diff took 0.023364 seconds; twenty new topics and
eighteen catch-annotation additions, with all prior instructions identical.
A catch-only mutation test now verifies handler changes are visible. Actual
remapping: 14 unique, 17 unanchored, 7 ambiguous; full automatic relocation
still requires review. Source coverage does not complete member/state ports.
[Coverage and catch evidence](docs/protocol/mex.md#exception-aware-numeric-and-participant-release-coverage-2026-10-07).
No account opened, state changed or live request sent.
Exception-aware snapshot validation: 620 workspace/all-feature tests (55
xtask), 24 default CLI tests, strict Clippy, warning-free Rustdoc, formatting,
CLI reference/API symbols, hygiene (557 local links), staged whitespace passed.
Both actual exports reproduced: new 43-file catch-aware snapshot and prior
23-file default-format snapshot, with complete checksums unchanged for the latter.

MEX object-edge roles now project in Rust and via explicit --edge-roles on
mex-response-inspect (--group-info required): ordered recognized enum strings,
nonobject skips and nullable/wrong-type list fallback. Eighteen synthetic cases
cover both payload children. This is a field projection before phash/member
validation, not a member map/state update. Participant constructor fields and
history/join boxing are now traced; literal class tables confirm BotJid is a
UserJid through abstract 0ZX, whose zero class-data pointer explains the empty
method lookup. Remaining full conversion/type/alias/boolean/application gaps
are [documented](docs/protocol/mex.md#participant-model-fields-and-object-edge-projection-2026-10-07).
No saved account, identity, action state or live request changed.
Edge projection validation: 621 workspace/all-feature tests, 24 default CLI
tests, strict Clippy, warning-free Rustdoc, formatting, CLI reference/API symbol
checks and hygiene (564 local links) passed. Actual CLI checked 36 edge modes,
36 combined numeric modes, 36 unchanged redacted defaults and the flag dependency;
blocked projections never emitted partial stdout or private sentinel content.

MEX phash/history/join scalar coercions now project in Rust and explicit
--participant-scalars (--group-info required), with redacted Debug/defaults.
408 independent pinned-AOSP/host-Java cases cover number wrapping, string/float
saturation, decimal/hex rounding and nullable values. A local serde JSON
precision bug falsely mapped 0.99999999999999994 to1/phash=true; float_roundtrip
fixes it with a regression. Full Android/JSONTokener and member-map/state
application remain unverified/unfinished. Snapshot owners now include the
coercion module, 43 files/162671 bytes; all 38 topic texts unchanged.
[Evidence and limits](docs/protocol/mex.md#participant-scalar-coercion-and-float-rounding-2026-10-07).
No saved account, keys, identity, action state or live request changed.
Scalar-port validation: 623 workspace/all-feature tests, 24 default CLI tests,
strict Clippy, warning-free Rustdoc, formatting, CLI reference/API symbols and
hygiene (578 local links) passed. All 408 oracle cases reproduced using the
committed annotation stubs; actual CLI checked 816 scalar modes, 816 combined
modes, 816 unchanged redacted defaults, 16 blocked/no-partial-output modes and
flag dependency. Actual 43-file/162671-byte APK snapshot re-export matched.

Phone/account inventory audit (2026-10-07, offline): 19 profile files reference
13 distinct country/national-number pairs across 16 existing configurations.

Participant address trace (2026-10-07, static): APK 37.71 strict member/alias
checks reject device wrappers (DeviceJid extends Jid, outside UserJid); plain
bot/LID/interop constructors qualify. Phone/LID digit and length rules, reserved
phone prefix, interop last-hyphen/signed parseInt behavior and plain hosted-domain
normalization are recorded with scoped instructions and thirteen literal class
definitions. Full singleton parsing/member assembly/durable MEX application remain
unfinished; no API/CLI change, saved account access or live request.
[Evidence and negative results](docs/protocol/mex.md#participant-address-validation-and-strict-type-boundary-2026-10-07).
Validation: APK hash and all thirteen class-definition literal bytes rechecked;
phone validation, full interop class and DeviceJid constructor re-exported exactly.
Repository hygiene passed (583 local links); staged whitespace passed. This
source-only chunk changes no runtime code, so no new runtime test result is claimed.


Strict MEX participant primary-address parser now implemented (2026-10-07):
constructor/type guards, plain hosted normalization, signed/Unicode interop
integrators and confirmed Server/0/Me singleton handling, separate from sending
eligibility. New offline inspector outputs subtype only and redacts failures.
283 independent Java references and nineteen domain class literals preserved;
full member/alias/durable MEX application and Android/cache parity remain open.
Release bundle expands to 50 methods / 55 files / 191422 bytes; 38 old texts
unchanged, cached diff0.051814s. No saved account access or live request.
[Evidence and limitations](docs/protocol/mex.md#strict-member-primary-address-parser-in-rust-2026-10-07).

Validation: 625 workspace/all-feature tests, 24 default CLI tests, strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference/API-symbol checks passed. After the negative-prefix CLI correction,
all 56 xtask tests and strict xtask Clippy passed. Actual CLI:282 integrator cases,
seven subtype successes, eight content-free/no-stdout refusals; NUL library-only.
All nineteen domain class literal records were reread against the APK; final
snapshot checksums and repository hygiene passed (591 local links). Staged
whitespace passed. No complete member conversion or live validation is claimed.


Nullable ordered MEX member map implemented (2026-10-07): phash null remains a
separate variant, invalid nullable bots skip, strict primary/alias failures abort,
canonical duplicate keys replace values in first insertion order. Role/join/history/
display slots mapped, with explicit refusal for unported Java floating/structured
string coercions. Private --participant-members inspection composes with prior
projections; failed partial maps, Debug and default output remain redacted.
41 synthetic APK-branch fixtures and eight independently executed Java hashes
cover both payload paths. Actual typename names, complete group conversion and
live/durable application remain unknown/unverified. No saved account/live action.
[Sources and current boundary](docs/protocol/mex.md#nullable-ordered-participant-map-port-2026-10-07).

Validation: 628 workspace/all-feature tests, 24 default CLI tests, strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting, regenerated CLI
reference and API-symbol checks passed. Actual CLI:82 member modes,82 combined,
82 unchanged default summaries,56 no-partial-output refusals and flag dependency.

Release snapshot:59 methods/64 files/204600bytes; all 50 prior instruction texts
unchanged, checksums verified; cached comparison0.019005s. Builder numeric type
predicates are documented separately from strict alias instanceof checks; broader
32q callers remain untraced. No device-alias acceptance was introduced.

After diagnostic improvements, the full workspace tests/Clippy/Rustdoc reruns
passed. Refusals name the offending attribute without its value. Scoped model
constructors were confirmed as wrappers only; broader string formatting remains
unported. Repository hygiene passed (599 local links); staged whitespace passed.


MEX metadata strings (2026-10-07, offline): formerly unsupported floating,
object and array display metadata now convert against pinned AOSP/host-Java
references. Private ordered parsing preserves duplicate position, insertion order,
escaping and integer negative-zero classification; original public serde data,
requests and persisted serializers are unchanged. Independent exact-rational
Java 21 formatter adds no dependency or copied JDK source. 2242 IEEE and 424
raw JSON references, including 848 member conversion paths, passed. Android
runtime-version/lexical equivalence and full/live durable group conversion remain
unverified. Evidence and discovered reference-harness failure are preserved in
[the string trace](docs/protocol/mex.md#participant-metadata-string-conversion-2026-10-07).

Validation: 632 workspace/all-feature tests, 24 default CLI tests, strict Clippy,
warning-free Rustdoc, formatting and regenerated CLI reference passed. Actual CLI
checked 82 member modes, 82 combined modes, 82 default redacted modes and 40
blocked/no-partial-output cases, plus 848 raw structured member checks and 848
unchanged default summaries. APK owner-only re-export has 59 unchanged method
texts / 64 files / 205234 bytes; cached diff took 0.038028 seconds. No account
access, registration, live transmission or new number acquisition occurred.
Final repository hygiene passed: tracked-file budgets, 611 local links, package
licenses and 564315 bytes of APK research checksums; staged whitespace was clean.
The formatter corpus also passed after the strict-Clippy parity expression fix.


MEX member-storage consumers (2026-10-07, offline): APK 37.71 A4m.A0O has one
behavioral reader, 0yw.A0X; A0U skips member reconciliation for null but calls it
for nonnull empty maps. Consumer timestamp scaling/previous-clock fallback and
nullable-history integer preservation are now implemented as a pure Rust helper
and GroupParticipant method, with explicit private-JSON CLI inspection. 432 host
Java transcribed-branch reference cases and 432 actual CLI cases passed; nine
invalid/oversized input refusals stayed content-free with empty stdout. Full
membership/alias/device/own-account application, storage atomicity and live MEX
settlement remain unfinished. [Evidence](docs/protocol/mex.md#downstream-member-storage-and-null-map-2026-10-07).

Validation: 634 workspace/all-feature tests, 24 default CLI tests, regenerated CLI
reference, warning-free Rustdoc, formatting and final strict Clippy passed.
The initial Clippy run found redundant test-only qualifications; they were
removed and strict Clippy rerun successfully. No live request/account access or
identity/key change. Versioned bundle adds seven methods (66 methods / 71 files /
285828 bytes), preserving all 59 prior texts; cached diff 0.035567s. Baseline 8295e18.

Final hygiene corrected an existing coverage gap: its checksum section formerly
checked only 37.70, while MEX had separate diff verification. It now checks all
four discovered snapshot bundles (2029764 bytes), with 621 local links, tracked
budgets and package licenses passing. Its standalone corruption/size/extra-file
test passed and is wired into CI. Staged whitespace and CI-tool formatting passed.
[Scope correction](docs/protocol/apk-porting.md#checksum-coverage-correction-2026-10-07).
Final diagnostic correction reran all 634 workspace/all-feature tests and strict
Clippy successfully; all 432 actual CLI cases and nine named refusals passed.
Actionlint accepted the changed workflow. No new live validation is claimed.


Background device-sync correction (2026-10-07): durable local revisions keep
newer pending notifications when an older reply settles; notification pending
work commits before ACK writing. The receive actor continues eligible 64-user
batches while deferring refused/errored/omitted/unanswered users for this
connection. No wire/identity changes or automatic refusal replay. Offline
injected-link checks cover 130 users, newer-notification race, ACK socket-cut
reopen and timeout/late reply; live coverage of these branches remains unknown.
Own-account sync and notification-ID deduplication remain unfinished.
[Evidence and limits](docs/protocol/messaging-transport.md#background-device-revisions-and-continuation-2026-10-07).

Live follow-up 21:57:48 UTC: one existing-peer device read in a bounded
20-second receive-only session saved server_acknowledged, one requested/ingested,
zero errors/omissions/superseded, code null. Device pending returned to zero;
outbox/actions/nine ACKed receipts unchanged, no chat sends. Primary only,
companions still unverified. Live races/large batches/socket-cut remain unknown.
Validation: 651 workspace/all-feature tests, default CLI24, strict Clippy,
warning-free Rustdoc, CLI reference and formatting passed. Six actual isolated
CLI probes plus live saved-query inspection passed; hygiene645 local links,
2405965 bytes/four unchanged APK bundles.


Contact ADV caller trace (2026-10-07, APK 37.71): eleven exact selected
methods now preserve supplied-key verification, stale/invalid caller branches,
notification addition and primary identity/session application. Saved identity
is not always the APK's selected key. Invalid/stale signed candidates do not
apply through the success path; raw-ID mismatch has separate reset branches.
Our current fallback actually drops companions and keeps only raw-ID/time floor;
prior preservation comments were wrong and are corrected. Runtime retention,
verified device authority after identity change, feature 0x3285 selection and
full supplied-key integration remain unfinished; no account/live request changed.
[Evidence and unknowns](docs/protocol/cross-device-sync.md#contact-adv-verification-and-retention-2026-10-07).
Core snapshot: 33 topics/161 methods, 38 files/967529 bytes. Cached comparison
against b058f61 took 0.028928 seconds: eleven added, all twenty-two old topics
unchanged; inventory/manifest/tokens unchanged, targets metadata expanded.

Validation: exact APK re-export/check verified all 38 files/967529 bytes;
checksum-verified cached diff preserved all 22 older topic texts. CLI reference
regeneration, four CLI OpenAPI coverage checks, API symbol check, formatting and
staged whitespace passed. Hygiene verified 659 local links and 2642000 bytes
across all four independent APK bundles. This source/comment-only chunk makes
no runtime behavior change; no new workspace-test or live-interop claim.


Device candidate retention implemented (2026-10-07): stale/invalid supplied
signed candidates and unsigned non-hosted companions now preserve cached lists,
pins/sessions and pending work. Fixed local verification_failures counts remain
separate from server errors; device-query exposes counts/null history and send
preparation blocks rejected reads. The receive cursor defers rejected users while
other users advance. Raw-ID replacement/supplied-key application remain unsupported;
no resets occur. Regression covers cache/pin/session preservation, duplicate rows,
encrypted reopen, fresh valid settlement and mixed missing-pin/other-user success.
[Evidence and limits](docs/protocol/cross-device-sync.md#rejected-device-candidate-retention-2026-10-07).

Conflicting duplicate-row correction: a valid first row could previously clear
pending before a later refused row. Selection now preflights all requested-user
rows before mutation; exact duplicates collapse, differing rows refuse with fixed
local conflicting_user_rows, preserving cached/pending state in both orders and
across reopen. This is local ambiguity policy; APK duplicate dispatch is unknown.
The fixed local reason set above additionally includes conflicting_user_rows.


Observed live 2026-10-07 22:13:51 UTC: zero outbox/queued-action/pending-receipt/
pending-device preflight, one existing-peer read and a 20-second authenticated
receive-only session completed cleanly. Saved CLI query: server_acknowledged,
requested_users=1, ingested=1, partial_errors=0, omitted_users=0,
superseded_users=0, verification_errors=0, verification_failures={}, code=null.
Pending users returned to zero; outbox, 17 actions (error1/ok15/sent1) and all
nine ACKed receipts unchanged. No chat sends or resets. This proves the supported
unsigned-primary live path and new diagnostics schema, not live rejected-list or
supplied-key behavior; the primary-only list still has no signed index evidence.

Validation: 654 workspace/all-feature tests and 24 default CLI tests passed;
strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage and formatting passed. Fifteen actual isolated CLI probes covered all
nine local reasons, zero success, historical/unsettled null, absent, unchanged
repeat and invalid-ID redaction. Conflicting duplicates preserve cache/pending
in either order and after encrypted reopen. Hygiene663 local links and 2642000
bytes across four unchanged APK checksum bundles passed. No live malformed,
signature-refusal or raw-ID replacement claim is made.


Signed-device authority binding and authenticated refresh (2026-10-07): successful
signed lists privately bind their actual verifier key; historical rows remain
unknown. devices/peer-identity expose current authority flags without the binding
fingerprint. Group/status plans refuse unbound/changed cached companions while
preserving history. Already-authenticated remote primary changes queue refresh
inside their pin/session/archive/event transaction; own PN/LID and companions
are excluded. Receive heartbeat picks up eligible post-startup work without a
notification/reconnect and keeps existing refusal/unknown deferral. Offline
rollback/reopen/rebinding and queued-during-receive checks cover the local path;
live primary replacement/own inventory/supplied ADV-key adoption remain unknown.
[Evidence and limits](docs/protocol/cross-device-sync.md#signed-device-authority-and-authenticated-refresh-2026-10-07).


Observed live 2026-10-07 22:33:56 UTC: a zero-send/action/receipt/device-pending preflight,
one existing-peer read and a 20-second authenticated receive-only session
completed cleanly. Saved query: server_acknowledged, requested/ingested=1,
partial_errors/omitted/superseded/verification_errors=0, verification_failures={},
code=null. Pending devices returned to zero; outbox, all 17 actions
(error1/ok15/sent1) and nine ACKed receipts unchanged, no chat sends. Authority:
binding_known=false, primary_pin_present=true, primary_pin_matches=null,
companions_current=false. This accurately reports the unsigned primary-only
reply with no signed index list; it does not prove live companion binding,
identity replacement, own inventory or a supplied-key branch.

Validation: 658 workspace/all-feature tests and 24 default CLI tests passed.
Strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage, formatting and staged whitespace passed. Thirteen actual isolated
CLI JSON/human/request probes covered bound/current, changed, missing pin,
historical unknown, unsigned primary and absent list; no private verifier
fingerprint was exported and --json --request parsed as one object. Fixture
metadata probes test inspection, not cryptographic signing; genuine signed
baseline/replacement tests cover authority separately. The own-primary hook
additionally passed explicit PN/LID/companion exclusions. Hygiene668 local
links and 2642000 bytes across four unchanged APK checksum bundles passed.

ADV identity follow-up trace (2026-10-07, **confirmed static**, APK 37.71):
DNy case 40 sends an encrypt/identity read after verifier failure; DF6/DNk
separate primary identity saves from companion ADV validation. Feature 0x3285
enabled skips immediate application; its raw-ID callback does not establish
ordinary deferred success. Raw-ID cleanup removes remote companions while
explicitly preserving the primary. Remaining key-type/companion verifier,
continuation and cleanup internals are unknown; current full peer-keys queries
are not identity-only recovery. No account or live request changed.
[Source and limits](docs/protocol/cross-device-sync.md#adv-identity-follow-up-and-raw-id-cleanup-2026-10-07).

Primary identity-only read staging (2026-10-07): peer-identity-query queues the
APK 37.71 encrypt/identity read once; --show classifies saved primary PN/LID
public-key evidence with exact namespace pin match and redacted failures.
B7H type 5 / first 32 key bytes and companion dual-signature checks are traced.
Four offline regressions preserve state and uncertain/refused IDs. Automatic
ADV-failure scheduling, key adoption, companion policy and downstream success
event/session recovery remain unfinished; presence is not verification.
[Evidence and usage](docs/protocol/cross-device-sync.md#primary-identity-only-query-2026-10-07).

Identity-only read live validation (2026-10-07): one bounded 20-second session
settled the existing-peer query ok / primary_identity_received, null failure/code,
PN pin match false and no device-identity node. Returned public identity matched
an already saved verified full bundle in an offline cross-check. No pin/session,
outbox/receipt/prior-action change or chat send; repeat reuses the original ID.
This validates the read, not adoption or automatic ADV-failure recovery.
662 workspace/all-feature tests, 24 default CLI tests, strict Clippy/Rustdoc and
CLI/API coverage passed; core cached diff is 0.053777 seconds.

Sender-key device separation (2026-10-07, offline): APK Cnr retains recipient
and device fields; Rust previously flattened companions into the primary's
record. Storage/libsignal names now preserve the actual sender device and keep
primary history in place. Real-crypto PN/LID three-device store/reopen coverage
passes; companions never borrow a historical primary record. Primary save
callbacks additionally trace remote companion cleanup and sender-key rotation;
full actor/category/alias variants and identity-triggered rotation remain
unfinished. Identity-only queries still do not adopt keys.
[Evidence and limits](docs/protocol/cross-device-sync.md#primary-save-callbacks-and-device-sender-key-storage-2026-10-07).

Validation of this correction: 663 workspace/all-feature tests, 24 default CLI
tests, strict workspace/all-target Clippy, warning-free Rustdoc and regenerated
CLI/API coverage passed. The 13 added source topics expanded the core to 60
topics / 197 methods, 65 files / 1,528,287 bytes. Cached comparison took 0.053107
seconds: all 47 prior topics unchanged; manifest/inventory/tokens unchanged;
checksums verified. A rebuilt-client 20-second receive-only session ended
cleanly with peer pins, device inspection, actions, outbox and receipts unchanged,
no added query or chat sends. Private evidence index:
`lab/releases/sender-key-live/active.json`. Live companion decryption and
identity-triggered rotation remain unverified; no primary key was adopted.


Sender-key cleanup trace (2026-10-07, **confirmed static** APK 2.26.37.71):
0jc.A19 deletes one translated chat/account-type/device/bucket tuple;
0kX.A00 and 0kT.A02 preserve chat, bucket and sender device during translation.
This does not establish blanket remote incoming-key deletion. Account-provider
selection is partly traced; feature/alias authority and distribution-history
parity remain unknown/unimplemented. The comparison CLI distributes an existing
SKDM to every target but has no sent-sender-key history; redistributing is not
identity-triggered rotation. No live request or identity adoption occurred.
See [evidence and next implementation](docs/protocol/cross-device-sync.md#scoped-sender-key-removal-and-recipient-translation-2026-10-07).


Own sender-key exposure/rotation (2026-10-07): the comparison sender reserves
exact recipient-device exposure with key/session/send state. An authenticated
remote primary change marks indexed own keys pending in the same archive/event
transaction; the next new authorized group/status intent rotates with OCC and
private old-key archival. Incoming chains and uncertain intents are retained.
`peer-identity` exposes redacted pending_sender_key_rotations counts; zero does
not establish historical coverage. APK 37.71 D4x.A00/0jc.A19 motivate this local
subset; full bucket/alias/companion and normal actor group parity remain
unimplemented, and live rotation is unverified.
See [evidence and limits](docs/protocol/cross-device-sync.md#durable-own-sender-key-exposure-and-rotation-2026-10-07).

Validation of own-key exposure handling: 666 final workspace/all-feature tests,
24 default CLI tests, strict Clippy, warning-free Rustdoc, CLI/API checks passed.
A 20-second receive-only rebuilt-client check left peer/device/action/outbox/receipt
inspection unchanged. Live rotation remains unverified. The initial workspace
wrong-key reopen test failed AlreadyOwned; focused and full reruns passed without
relaxing locking or key validation.


Directory bundle selection (2026-10-07): shared parser now compares user/server/
agent/device without conflating wire-encoding kinds, refuses duplicate requested
user rows and optional one-time keys, and refuses contradictory outer errors.
These local ambiguity guards precede identity/session preparation; other batch
devices remain valid. Existing peer-keys inspection reports invalid_reply and
blocks refresh while preserving its selector and pins. APK 37.71 B7H.A06/CQc.A00
provide address/signature context; exact duplicate policy is not attributed to
the APK. [Evidence and tests](docs/protocol/peer-identity.md#exact-directory-bundle-selection-2026-10-07).

Validation of exact directory selection: 667 workspace/all-feature tests, 24
default CLI tests, strict Clippy, warning-free Rustdoc and CLI reference passed.
Rebuilt-client offline inspection retained verified_bundle/signature_valid=true
for the existing saved server response without changing protocol inspection.
No new live query or network session; malformed live responses remain unobserved.


Missing group sender-key correction (2026-10-07): supported nonempty skmsg with
no sender key is now Undecryptable, reaching the existing once-only retry queue;
it is no longer silently Unsupported. Empty/structured skmsg with a stored key
cannot be reported Decrypted or delivery-confirmed. Byte/Text ciphertext views
are consistent across direct/group receive. Real-crypto late-key recovery and
direct/group loop write-cut fixtures preserve captures, ratchets and retry
reservations. Handset retry shape is observed; APK 37.71 Cp2.A05 is ACK evidence,
not a confirmed missing-key retry count policy. Fresh live group recovery remains
unverified. [Evidence and limits](docs/protocol/messaging-transport.md#missing-group-sender-key-recovery-2026-10-07).

Validation of missing-group-key handling: 668 final workspace/all-feature tests,
24 default CLI tests, strict Clippy, warning-free Rustdoc and CLI reference passed.
A 20-second receive-only rebuilt-client session left protocol inspection unchanged;
no fresh missing-key group recovery was observed and no chat intent was added.


SKDM-only group control deduplication (2026-10-07): real Signal regression
reproduced Undecryptable after a previously accepted control was redelivered
across reopen. Newly authenticated controls now commit sender_key_distributed
markers with key/session state and return Duplicate without another decrypt.
They carry no user protobuf/event/read receipt; lab-inbox --inspect labels them
redacted. APK Message field 2 is existing schema evidence; this private marker
is a local atomicity fix, and APK internal duplicate policy is unknown. No
historical marker synthesis or ratchet rewind; fresh live control redelivery
remains unverified. [Evidence and regression](docs/protocol/messaging-transport.md#durable-sender-key-only-group-controls-2026-10-07).

Validation of SKDM-only control markers: 669 workspace/all-feature tests, 24
default CLI tests, strict Clippy, warning-free Rustdoc and CLI/API checks passed.
A 20-second receive-only rebuilt-client check left protocol inspection unchanged;
no fresh control/redelivery was observed, so that live branch remains unverified.


Retry/ACK encoding identity correction (2026-10-07): a reproduced second-retry
reservation caused only by JidKind differences is fixed. Canonical logical
selectors preserve user/server/agent/device and probe all historical kind keys
in the same transaction, retaining pending/unknown/refused/acknowledged records
and original wire. ACKs ignore encoding kind but keep exact logical routes.
Independent historical-hash, reopen/state-preservation, foreign-route ACK and
actual receive-loop cut/reconnect tests cover the correction. APK 37.71 Cp2.A05
is wire ACK context; APK duplicate/retry-limit policy remains unknown. Historical
duplicate rows are not consolidated; live encoding changes remain unverified.
[Evidence](docs/protocol/messaging-transport.md#retry-reservation-identity-across-jid-encodings-2026-10-07).

Retry encoding validation: 671 workspace/all-feature tests, 24 default CLI
tests, strict Clippy, warning-free Rustdoc and regenerated CLI/API checks passed.
No live retry was manufactured or settled request replayed; actual server
encoding-altered redelivery/ACK remains unverified.

Primary-change encoding correction (2026-10-07, confirmed offline): reproduced
missed own group-key rotation for a remote AgentDevice/0 primary and accidental
own-PN device refresh. Encoding is normalized only after primary agent/device
checks; PN/LID namespaces and companion boundaries remain exact. Redacted rotation
inspection now ignores kind without folding devices. PN incoming and LID signed
bundle archive/reopen, own exclusion, rollback and stale-reservation tests cover
the local correction. APK 37.71 D4x.A00/B7H.A06 remain source context; live alternate
primary encoding and subsequent group rotation remain unverified.
[Evidence](docs/protocol/cross-device-sync.md#encoding-independent-primary-change-follow-ups-2026-10-07).

Primary-change encoding validation: 673 workspace/all-feature tests, 24 default
CLI tests, strict Clippy, warning-free Rustdoc and CLI/API checks passed. Rebuilt
CLI read-only saved-account inspection left all inspected protocol state unchanged;
no login/query/send or manufactured identity change was used to test the branch.

Own sender account selection (2026-10-07, confirmed static APK 37.71): 08i
initializes Apw from self_lid and Apz from saved Me.jabber_id; both retain the
persisted registration_device_id (default 0). D4x.A00 selects LID versus PN using
0bA.A0k -> 1jL.A0Y (group row.A00 == 2), then wraps the own device via B7H.A04.
Me.lid consistency diagnostics are not an established immediate self_lid fallback;
1XY is a call callback, not run. Group-row mode production, async reconciliation,
complete bucket/actor semantics and Rust own-address parity remain unfinished.
No account state, keys, identity policy or live traffic changed.
[Sources and boundaries](docs/protocol/cross-device-sync.md#own-pn-lid-sender-selection-and-persisted-device-index-2026-10-07).

Own-account trace validation: 58 xtask tests, CLI reference regeneration and five
API checks passed. The core now holds 81 topics / 219 methods, 86 files /
1,586,459 bytes under its 2 MiB limit. The checksum-verified cached diff took
0.080554 seconds: 13 added topics, all 68 prior topics unchanged; manifest,
inventory and tokens unchanged. This measures cached scoped source comparison,
not full-app decompilation or automatic porting. No live request was needed.

Group-mode producer/storage trace (2026-10-07, confirmed static APK 37.71): five
1jL.A00 writes and its constructor input are retained. Cache-miss mode comes
from B7i.A00 chat predicates, not membership suffixes alone; contact mode is saved
through 0Y9.A0s -> 0gt.A0V -> 9R1.A0J in wa_group_admin_settings. Reload 0lO.A01
normalizes case-insensitive pn to pn, other returned values to lid. B7i lookup
and predicate have different absent-contact defaults; broadcasts have additional
diagnostic behavior. Full mode provenance, category predicates and Rust mode/
own-PN fallback integration remain incomplete. No live/account state changed.
[Evidence and negative results](docs/protocol/cross-device-sync.md#group-mode-producers-storage-and-cache-classification-2026-10-07).

Group-mode trace validation: 58 xtask tests, CLI reference regeneration and five
API checks passed. The core contains 100 topics / 238 methods, 105 files /
1,731,431 bytes under the 2 MiB budget. The checksum-verified cached diff took
0.075441 seconds: 19 added topics, all 81 prior topics unchanged; manifest,
inventory and tokens unchanged. No live/account request or runtime migration
was used for this source trace.

Group routing before device sync (2026-10-07, confirmed offline): the comparison
sender's catch-all plan fallback returned device-query targets even after route
refusals. The extracted old logic failed a mixed-membership regression. Shared
recipients_for_group now validates namespace/own membership before sync, allows
initial missing lists and excludes/deduplicates own/remote targets. Final plan
still validates device authority before reservation. APK mode/category state
integration and automatic own-PN selection remain incomplete; no new live send.
[Routing correction](docs/protocol/groups.md#group-recipient-validation-before-device-sync-2026-10-07).

Group-routing validation: 675 workspace/all-feature tests, 24 default CLI tests,
strict Clippy, warning-free Rustdoc and regenerated CLI/API checks passed. Rebuilt
CLI read-only inspection left outbox, actions, receipts, devices and pins unchanged.
The private evidence index is lab/releases/group-routing-inspection/active.json;
no login, device query, group send or identity replacement was manufactured.

Concrete group-mode kinds (2026-10-07, confirmed APK 37.71): 0DE predicates
resolve broadcast-list type 3, interop.g.us type 28, status@broadcast type 5,
and cache-group types 1=g.us/2=temp. A0n additionally includes type 28.
Temporary groups are not established as old PN g.us groups. Existing 0Fs cache
and ADs reply classifications are linked; numeric group-type agreement alone
does not establish complete callback/history provenance. Runtime mode/contact
synchronization and unsupported namespaces remain unported; no account request.
[Kinds and remaining gaps](docs/protocol/cross-device-sync.md#concrete-jid-kinds-in-group-mode-classification-2026-10-07).

Concrete-kind trace validation: 58 xtask tests, CLI reference regeneration and
five API checks passed. Core snapshot: 109 topics / 275 methods, 114 files /
1,755,155 bytes (2 MiB bound); supplementary A0n source is separate. The
checksum-verified cached diff took 0.197003 seconds: nine added topics, all
100 prior topics unchanged; manifest, inventory and tokens unchanged. These are
static/offline findings; no live behavior or automatic mode adoption is claimed.

Group-type persistence continuation (2026-10-07, **confirmed static APK 37.71**):
existing-chat updates write group_type through transactional database helpers;
missing chats are left absent. Selected subgroup/zero-update callers and secondary
type-4 model writes are retained without inventing complete-reply enum values.
A0Y's outer A0n guard is narrowed again by A0A/A0m, excluding interop type 28
from the type-3 branch. Runtime contact/type synchronization remains incomplete;
no account/network request. [Evidence and unknowns](docs/protocol/cross-device-sync.md#persisted-group-type-updates-feeding-sender-mode-2026-10-07).

Group-type trace validation: 58 xtask tests, regenerated CLI reference and five
API checks passed. The core checksum bundle now has 118 topics / 289 methods,
123 files / 1,829,892 bytes (2 MiB bound). The verified cached diff took
0.168698 seconds: nine added topics, all 109 previous topics unchanged;
manifest, inventory and tokens unchanged. Selected runnable excerpts remain
outside the checksum bundle. No runtime behavior or live validation is claimed.

Addressing-mode producer trace (2026-10-07, **confirmed static APK 37.71**):
complete binary group replies do not directly read the mode attribute. Generated
validators use exact pn/lid membership; incoming group-message parsing has a
separate case-insensitive PN check. Five contact-mode setter callers now have
selected paths retained, including gated info-error and PN-conversion branches;
triggers, full callback provenance and runtime integration remain unfinished.
No live request or account change. [Evidence and boundaries](docs/protocol/groups.md#addressing-mode-producers-and-separate-parser-rules-2026-10-07).

Addressing-mode trace validation: 58 xtask tests, regenerated CLI reference and
five API checks passed. Core checksum bundle: 129 topics / 304 methods,
134 files / 1,900,193 bytes (2 MiB bound). Verified cached comparison took
0.151473 seconds: eleven new topics, all 118 prior topics unchanged; manifest,
inventory and tokens unchanged. Scan/caller appendix remains outside the bundle.
No runtime migration, automatic error recovery or live validation is claimed.

Direct LID representation correction (2026-10-07, **confirmed offline**):
textual and typed sender JIDs now select the same held LID session. A real
Signal regression failed Undecryptable before the fix; both representations
now decrypt and cross-representation redelivery remains Duplicate after reopen,
with no PN session or identity rewrite. APK DEw.CF0 parsing is linked, while
full direct-session producer provenance remains incomplete.
[Evidence and boundaries](docs/protocol/messaging-transport.md#direct-lid-sessions-across-attribute-representations-2026-10-07).

Direct-LID correction validation: 676 workspace/all-feature tests and 24 default
CLI tests passed, along with strict all-target/all-feature Clippy, warning-free
Rustdoc, CLI reference/API checks and the experimental CLI build. The first
local preflight ran before that build was terminal and encountered an older
CLI without outbox-pending; it stopped before opening an account connection.
A subsequent preflight uses the completed build.

**Observed live follow-up:** the completed experimental CLI passed queue preflight
and authenticated for one 20-second receive-only window, then stopped cleanly.
Outbox, actions, receipts, retained devices and peer pins matched their preflight
inspections afterward. No fresh encrypted user-message stanza was logged, so
live text/typed-LID decryption is not established by this window. Private evidence
index: lab/releases/direct-lid-live/active.json. No queued chat transmission,
new identity query, reservation reset or manufactured recipient message.

Group pairwise companion correction (2026-10-07, **confirmed offline**):
held and pinned companion msg sessions retain their actual participant device;
primary/alias fallback, unpinned and prekey-bootstrap paths are excluded from
this local subset. Real Signal PN/LID primary/device-3 control fixtures now
install sender keys and retain duplicate markers across reopen. B7H forward/
reverse device construction is linked; CPQ.A00 is refuted as a 37.71 session
selector. Full companion bootstrap/identity policy and live device coverage
remain unfinished. [Evidence and scope](docs/protocol/groups.md#group-pairwise-session-device-correction-2026-10-07).

Companion pairwise refusal diagnostics name unsupported prekey or nonzero-agent
routing. Missing held/pinned state reports only session_held and identity_pinned
Booleans; no user/device address, key bytes, ciphertext or message content.

Group companion correction validation: 677 workspace/all-feature tests, 24
default CLI tests, strict all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference/API checks and the experimental CLI build passed.
The selector regression returned a primary before the fix, and the encrypted
companion control fixture returned Unsupported before the correction.

Rebuilt-CLI offline inspection left outbox, actions, receipts, device state and
peer pins unchanged. Private evidence index:
lab/releases/group-device-inspection/active.json. No login/query/group send or
new companion identity was manufactured. Fresh live companion decryption and
full companion bootstrap remain unverified.

Incoming companion prekey caller trace (static, 2026-10-07): APK 37.71
CoU.A04 supplies a companion candidate to 11T.A0D before CQn.A00 dispatches
kind-1 ciphertext to 0jc.A0O. The latter's B77.A04 branch is STATELESS,
using fresh DS0 in-memory sessions and DRz's no-op identity save; ordinary
0ko.A04 uses durable adapters. This refutes conflating stateless mode with
companion addressing. Typed catches now associate obfuscated exceptions with
local decrypt results without inventing server refusal/ban interpretations.
DOL.call is a voice-call ADV gate, not messaging admission evidence. Full
sender/ADV/address producers, feature/dead-device conditions, transaction
coverage and companion bootstrap/application policy remain unported/unknown.
The group selector still accepts only held exact pinned companion msg sessions;
companion pkmsg remains our implementation limitation. No account/live changes.
Evidence: docs/protocol/peer-identity.md#outer-prekey-dispatch-and-companion-admission-2026-10-07.
Final APK regeneration verified 136 topics / 318 methods / 141 files, 1975976
bytes; seven additions and all 129 prior topics unchanged in a 0.072842-second
cached comparison. The 58 xtask tests, CLI reference, five API checks and formatting
passed. Four APK bundles total 3650447 bytes; appendix outside the core bundle.
Repository hygiene checked 850 local links, licenses and all four APK checksum
bundles. No saved state, request identity or live queue changed.

Companion ADV signature verification (2026-10-07): Rust now checks both account
and device signatures over exact details and ciphertext identity, using pinned
primary authority and APK 37.71 device-type-selected E2EE/HOSTED prefixes.
lab-inbox --inspect exposes bounded read-only companion_adv diagnostics, without
key/ADV/ciphertext metadata, query or state adoption. Four real-signature fixture
tests and one PN/LID direct/group read-only/refusal test passed. The saved-account
inspection produced no qualifying companion diagnostic; outbox/actions/receipts/
devices/pins were unchanged. Full workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI tests, CLI reference and final build passed.
APK export: 145 topics / 327 methods / 150 files / 2001638 bytes; nine additions,
all 136 prior topics unchanged in a 0.075721-second cached comparison. Four
bundles total 3676109 bytes; producer appendix outside core. Ordinary Bp2 ADV
producer and admission/freshness/raw-ID/teardown/atomic bootstrap remain open;
companion pkmsg is still unsupported by the runtime selector. No live/account
identity change. See docs/protocol/peer-identity.md#companion-adv-cryptographic-binding-and-inspection-2026-10-07.
Repository hygiene passed 861 local links, licenses and all four APK checksum
bundles. Private original state and unrelated exports remain outside Git.


Ordinary companion ADV producer and pending timestamp (2026-10-07,
**confirmed static APK 37.71**): DEs.CF2 -> Cih.A0V -> Bp2.A0L closes the
ordinary producer gap. Matched-primary/raw-ID/type admission rejects old/equal
candidates when the actual device index is absent/different. SQL read/write
and LbQ constructor mappings prove 0bN.A05 advances expected_timestamp while
preserving the verified list timestamp; treating incoming ADV as a verified
list update was refuted. Atomic companion pkmsg bootstrap, expected-refresh
application, new-device/alias mutation and own/raw-ID/type reconciliation remain
unfinished. No account/network change. [Evidence and unknowns](docs/protocol/peer-identity.md#ordinary-companion-adv-producer-and-baseline-timestamps-2026-10-07).
Export: 155 topics / 337 methods / 160 files / 2,036,151 bytes; cached comparison
0.071644 seconds, ten added topics and 145 prior topics unchanged. Thirteen
full-register sources re-exported exactly. xtask 58 tests, CLI reference,
OpenAPI library-symbol check, formatting and whitespace passed.
Repository hygiene passed 865 local links, package licenses and all four
checksum bundles (3,710,622 bytes total).


Known companion prekey bootstrap (2026-10-07, **implemented/offline**):
group/status version-2 pkmsg now admits an E2EE companion from a current signed
device list with matching primary binding, raw ID, exact device/key index and
non-newer timestamp. Both ADV signatures and nested Signal MAC are required.
Exact companion session/pin, one-time prekey and SKDM/control/message commit
atomically with primary/list/account/own-alias rechecks. Configured pinned versus
apk_authenticated policy remains effective; authenticated changes archive the
exact prior companion state. Own and unknown-own namespace candidates defer.
`lab-inbox --inspect` reports fixed matched-baseline admission diagnostics without
state mutation or values. Five real-crypto tests cover bootstrap, one-time
consumption, reopen dedupe, MAC/policy failures, nine deferred inputs, four
authority changes/rollback and held-skmsg fallback. Direct companion prekeys,
newer-ADV pending refresh/new-device addition and raw-ID/type/own reconciliation
remain unfinished; fresh live companion behavior is unverified.
[Evidence](docs/protocol/peer-identity.md#known-companion-prekey-bootstrap-2026-10-07).
Workspace/all-feature tests passed (687); strict Clippy, warning-free Rustdoc,
formatting and whitespace passed. Further CLI/API/build and saved-state inspection
results are recorded below when complete.


**Observed locally:** final experimental CLI build and saved-account read-only
inspection completed. Captures yielded no qualifying companion ADV/admission
sample, so no live bootstrap is claimed. Before/after outbox, actions, receipts,
device and pin summaries were identical; no login/query/chat was sent. Private
inspection records remain in ignored lab storage. Default CLI tests (24),
regenerated CLI reference, OpenAPI library-symbol check and final build passed.
Repository hygiene passed 869 local links, package licenses and all four
checksum bundles (3,710,622 bytes). No fresh companion sample was found.


Direct companion receive and receipts (2026-10-08, **implemented/offline**):
known-companion direct pkmsg reuses the APK-derived matched-baseline ADV/atomic
checks; held msg selects the actual device ratchet. Events and direct SKDM retain
that device, while chat/dedupe uses its primary PN/LID namespace. Optional PN alias
disappearance dedupes; authenticated aliases stay under the primary LID. Delivery/
read routing supports non-hosted companion destinations and primary LID read chats.
Static Cp2.A00/CmC.A06 supports the traced DeviceJid destination branch; Cp2.A06
was refuted as ordinary producer (it is Bp1/status). Upstream receipt model
producers remain untraced. Tests cover PN/LID bootstrap, ordinary-msg continuation,
one-time consumption, events/receipts, alias/reopen dedupe, direct MAC/policy
failures and delayed-authority actual-device SKDM installation. An own peer
control regression found by the full suite was corrected; receive and backfill
now share alias rules. Workspace/all-feature tests passed (689). Newer-ADV
refresh/new-device application, wider reconciliation, direct-control projection
parity and fresh live companion samples remain unfinished.
[Evidence and unknowns](docs/protocol/peer-identity.md#direct-companion-receive-and-receipts-2026-10-08).


**Validation completion:** all 689 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI tests (24), regenerated CLI reference, API
library-symbol check and final experimental CLI build passed. Four scoped APK
receipt source outputs reproduced exactly. Saved-account offline inspection
found no qualifying companion sample; before/after outbox/actions/receipts/device/
pin summaries were identical. No login, query or chat send occurred. Fresh live
companion interoperability remains unverified; private inspection records stay
in ignored lab storage.
Repository hygiene passed 873 local links, package licenses and all four
checksum bundles (3,710,622 bytes total).


Known companion newer ADV (2026-10-08, static/offline): matching-index direct/group pkmsg now authenticates and atomically records a pending expectation/device sync without advancing verified timestamp. Lower signed responses retain pending work; reaching responses settle it. Inspection reports ready_refresh_pending without mutation. New-device addition, own-account reconciliation and live newer-companion validation remain unfinished. Evidence: [peer identity refresh](docs/protocol/peer-identity.md#known-companion-newer-adv-refresh-2026-10-08).

Validation for newer-ADV refresh: 691 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI tests (24 including API symbols), regenerated CLI reference and experimental API-symbol check passed. Offline saved-capture inspection found no qualifying ADV/admission sample; before/after outbox, actions, receipts, devices and pins were identical. No login, query or chat send occurred. Private inspection artifact: `lab/releases/companion-refresh-inspection/active.json`.


Companion new-device membership trace (2026-10-08, **confirmed static 37.71**): `11T.A0D` removes already present addresses before add-only application; a newer different index does not itself overwrite existing membership. `0bN.A0G` filters ownership; `1IW.A02/A09` selects gated canonical users; `AzR.A00` rebinds device numbers; `B7j.A03` writes user_device rows separately from the verified LbQ timestamp. Added-member participant/observer calls and caller teardown Boolean are traced only to their boundaries. Runtime gate values, full side effects and Rust new-device application remain incomplete. Full raw methods and reasoning: [new-device application](docs/protocol/peer-identity.md#companion-new-device-membership-application-2026-10-08). No live request or saved account mutation occurred.

New-device trace validation: all nine selected APK methods exported successfully with exception tables; 58 xtask tests, regenerated CLI reference and experimental API-symbol check passed. Runtime Rust behavior and saved account state were not changed.


Companion teardown producer (2026-10-08, **confirmed static 37.71**): ordinary CoU.A04 combines 0bq.A04, the local dead_devices registry and a nonnull parsed offline Integer for its final admission Boolean. Offline metadata is separate from retry count/dhash. 1Uf.A06 persists removal markers; A02 rechecks current membership before cleanup, dropping returned-device markers without key teardown. Removal callbacks, transient A09 set and local worker bounds are distinct. Rust registry/reap and new-device application remain unported; no account mutation occurred. Disassembler b2 multiplication now has its proper label and independent literal regression. [Evidence and limits](docs/protocol/peer-identity.md#companion-teardown-producer-and-registry-2026-10-08).

Teardown trace validation: 692 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI/API-symbol checks (24), regenerated CLI reference and final experimental CLI build passed. Actual APK re-export renders 0CK.A07 multiplication at 0082 correctly. Four committed checksum bundles have no op=b2 and remain unchanged. No login, live request, key cleanup or account-state mutation was performed.


Remote newer-ADV application (2026-10-08, APK 37.71 static/offline): absent non-offline companions now enter authenticated direct/group receive and are added atomically with provenance/refresh, preserving verified timestamp. Existing different indices are retained until sync. devices exposes adv_added_devices and human expected_ts/count. Signed reconciliation retains newer allocations and clears provenance only on confirmation/removal. Absent offline candidates remain locally deferred pending teardown lifecycle; alias/participant/own reconciliation and fresh live validation remain incomplete. [Implementation and limits](docs/protocol/peer-identity.md#remote-newer-adv-device-application-in-rust-2026-10-08).

Remote device application validation: 692 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI/API checks (24), regenerated CLI reference and experimental API-symbol/build checks passed. Real PN/LID direct/group fixtures cover known/absent/different-index candidates, MAC refusal, atomic rollback and duplicates after reopen; legacy provenance defaults and signed confirmation are checked. Offline saved-capture inspection found no qualifying companion sample; outbox/actions/receipts/devices/pins matched before and after. No login, query or chat transmission occurred. Private artifact: `lab/releases/device-add-inspection/active.json`.


Pending removal registry (2026-10-08, **static/offline**): signed device reconciliation records disappeared companions and clears authenticated returned addresses in nullable pending_removed_devices. Legacy null stays unknown until signed reconciliation; unsigned responses do not initialize/clear it. Offline absent candidates proceed only with known no-marker state; pending_device_teardown refuses a marked address. Complete-list CAS includes removal state. Inspection exposes unknown/known-empty/count. No cleanup, identity/key deletion, reset, alias fan-out or worker parity is implemented. [Evidence and limits](docs/protocol/peer-identity.md#namespace-local-pending-removal-registry-2026-10-08).

Removal registry validation: 693 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI/API checks (24), regenerated CLI reference, experimental API-symbol check and final CLI build passed. Offline saved-capture inspection found no qualifying companion sample; before/after outbox, actions, receipts, devices and pins matched. No login, device query, message transmission or key cleanup occurred. Private artifact: `lab/releases/removal-registry-inspection/active.json`.


Nullable offline metadata (2026-10-08, **static/offline**): companion admission
now matches APK 37.71 nullable signed-Integer semantics instead of attribute
presence. First-match malformed/overflow values do not invoke removal guards;
zero/negative valid values do. Existing authority/MAC/atomic checks remain.
Independent host-Java corpus and encrypted PN/LID direct/group fixtures cover
the correction. Android Unicode/gates, cleanup and live companion validation
remain unknown. See [evidence and limits](docs/protocol/peer-identity.md#nullable-offline-companion-metadata-2026-10-08).

Nullable-offline validation: 694 workspace/all-feature tests, strict all-target
Clippy, warning-free Rustdoc, 24 default CLI/API checks, regenerated CLI
reference, experimental API-symbol check, final CLI build and formatting passed.
The repeated checks used system Rust 1.99.0 with isolated LLVM 22 after the host
package change; pinned 1.98.1 proof was not established. Repository hygiene
checked 896 local links. Offline saved-capture inspection found no qualifying
companion sample; outbox, actions, receipts, devices and pins matched before/after.
No login, device query, chat send or cleanup occurred. Private inspection artifact:
`lab/releases/offline-parser-inspection/active.json`.


Companion removal gate (2026-10-08, **confirmed static 37.71**): 0bq.A04
is ABPropBoolean property 35661, with false production/debug baked defaults.
Ordinary receive and worker predicates resolve it through 00D.A13; cached,
MobileConfig and preference branches can supersede the supplied default.
The effective account value remains unknown. Rust's current removal guard is
conservative local policy, not an unconditional WhatsApp requirement. No runtime
state, identity, keys, markers or waits changed. Nine bounded source topics are
added to the cached core release baseline; automatic relocation is unestablished.
[Evidence and limits](docs/protocol/peer-identity.md#companion-removal-property-and-effective-value-2026-10-08).

Feature-gate trace validation: thirteen complete selected method/class exports
(with exception tables), one field-reference scan and the current APK SHA-256
were checked. Actual core export: 164 topics / 349 methods / 169 files /
2,068,819 bytes, within the 2 MiB bound. Checksum-verified cached comparison:
nine additions, 155 unchanged, 0.0942 seconds. All 59 xtask tests, regenerated
CLI reference, experimental API-symbol check and formatting passed. Runtime
Rust behavior is unchanged; no account access, login, query, send or cleanup
occurred. Pinned-toolchain proof is not claimed; checks used system Rust 1.99.0.
Repository hygiene passed: 902 local links and 3,743,290 bytes across four
checksum bundles. Reproduction commands quote DEX descriptors/constructor names
for shell use. No runtime code or private state changed.


AB-property receive read (2026-10-08, static/offline): new abprops-query queues
one initial APK 37.71 abt/props read; --show inspects saved correlated values
without applying feature configuration. Boolean 35661 is registered in the
concrete APK table and parsed as signed Integer != 0. Full/delta/hashless and
last-duplicate semantics are distinguished. Empty text attributes now survive
encoding, preserving the initial empty hash. Effective MobileConfig precedence,
configuration application/scheduling remain unfinished; live read outcome is unknown.
No identity/key rotation, marker deletion or reservation replay is added.
[Evidence, diagnostics and limits](docs/protocol/feature-configuration.md).

AB-property validation (2026-10-08, **observed live**): the existing account
logged in and became active, then received a terminal stream:error (no outer
code; ack child) before the bounded window finished. One read is reserved/sent
with no correlated response: outcome_unknown, no returned/effective property
value. Its original ID is retained and repeats do not transmit. Pre/post
selected peer pins, devices, receipts and outbox matched; prior actions were
preserved. No chat send or identity rotation occurred. The terminal cause and
configuration application remain unresolved. All 699 workspace/all-feature
tests, strict Clippy, Rustdoc, default CLI checks, experimental API-symbol and
CLI-reference checks, experimental build and formatting passed on system Rust
1.99.0. See [feature configuration](docs/protocol/feature-configuration.md) for evidence and inspection limits.

Repository hygiene passed: 912 local links, package licenses, tracked-file
budgets and 3,840,464 bytes across five checksum-verified APK bundles.

Terminal ACK diagnostics (2026-10-08, **observed offline / attribution unknown**):
lab-inbox --inspect now prints fixed ACK class/type and ID/to presence flags,
redacting all echoed identifiers/unknown values and diagnosing duplicate ACKs.
Saved captures are content-addressed without timestamps/epochs, and inspection
covers all history. The stored notification-ACK stream:error cannot therefore
be attributed to the latest receive window from this inspection alone; prior
wording was too strong. No causal query/ban/retry interpretation is established.
No login, query replay, key change or capture mutation occurred in this chunk.
See [terminal evidence and scope](docs/protocol/terminal-errors.md).

Rebuilt offline inspection identifies the saved ACK type as w:gp2; no network
session was opened. Its capture origin and relation to recent closure remain
unknown. All 47 experimental CLI checks, strict CLI/all-target Clippy, regenerated
reference, experimental build and formatting passed on system Rust 1.99.0.

Terminal provenance (2026-10-08 local, **implemented/offline**): comparison login
and receive now atomically preserve terminal captures with first/last local
observation milliseconds, actor-local epoch, observation count and latest
terminal marker. Offline inspection exposes these fixed metadata fields; old
captures remain unknown and offline replay cannot backfill them. Epochs can
repeat and wall clocks can reverse; latest follows commit order. Atomic rollback,
reopen, repeated observation and legacy non-backfill are tested. No request IDs
or identities are changed. [Evidence and limits](docs/protocol/terminal-errors.md).

Terminal-provenance validation: all 702 workspace/all-feature checks, strict
Clippy, warning-free Rustdoc, default CLI checks, regenerated reference,
experimental build and formatting passed on system Rust 1.99.0. The session-loop
terminal fixture additionally confirms metadata persistence. One live receive-only
32-second window authenticated, stayed active and stopped cleanly (exit 0).
No terminal stanza was observed; the older terminal capture remains unknown.
Actions/devices/pins/receipts/outbox matched preflight, with zero new actions.
The original AB-property read remains outcome_unknown with its original ID;
no query was queued or replayed. Fresh incoming-message decryption and long-term
reliability are not established by this window.

ACK writer closure (2026-10-08, **confirmed static APK 37.71**): Android arg1
selectors 76/129 converge through Cp2.A05 and 1Rd.A04; Ck7 flag 1 serializes
the supplied node with raw leading zero. No destination/type rewrite appears
at the traced boundary; no APK-backed runtime correction was found. Historical
terminal ACK origin remains unknown. The expanded group bundle preserves 39
topics / 43 methods / 44 files / 928771 bytes with exception tables. Cached
same-APK comparison against 80f5636 took 0.039709 seconds: five additions,
34 annotation-changed files; all old instruction texts remain equal. Native
framing, metadata side effects and complete notification application remain
unfinished. No account access, live request, query replay or runtime change.
[Evidence and comparison](docs/protocol/groups.md#ack-dispatch-and-uncompressed-write-boundary-2026-10-08).

ACK evidence verification: all 59 xtask tests, actual bounded export, cached
checksum diff and old instruction equivalence, regenerated CLI reference,
experimental API-symbol check and formatting passed on system Rust 1.99.0.
No runtime Rust change, saved-account access or live request occurred.

Repository hygiene passed: 928 local links, tracked-file budgets, package
licenses and 3,996,167 bytes across five checksum-verified APK bundles.

Companion removal scope (2026-10-08, **confirmed static APK 37.71**): A0s/A0r
are session and identity removal, with device-specific recipient/type/device
SQL predicates. Feature-gated address mapping can select namespace counterparts;
CDO property 31194 has prod false/debug true defaults but unknown effective
state. The session callback also invalidates cache variants for that device.
Sender-key SQL is sender/type/device plus a selected group list. Transaction
callback delivery, full mapping/observer effects, group selection and effective
gates remain incomplete; no destructive Rust cleanup is enabled. A separate
17-topic / 17-method / 22-file / 181102-byte checksum bundle retains the complete
selected chain with exception tables. All 59 xtask tests passed. No saved account
was opened, no live request sent, and no keys or history were deleted.
[Scope and port boundaries](docs/protocol/peer-identity.md#companion-key-removal-scope-and-callback-boundaries-2026-10-08).

Cleanup evidence checks: 17 direct dumps match bundle instruction-offset/
mnemonic coverage and ordered catch ranges; direct operand details are retained
separately. Regenerated CLI reference, experimental API-symbol check and formatting
also passed on system Rust 1.99.0. Runtime Rust code is unchanged.

Repository hygiene passed: 935 local links, tracked-file budgets, package
licenses and 4,177,269 bytes across six checksum-verified APK bundles.


Cleanup callback closure and register-aware exports (2026-10-08): **confirmed
static APK 2.26.37.71** Cpf discriminator 1 uses Bb1 -> 8kU.A1P after successful
1gB.close/0IY.endTransaction through 0Gy.A02; rollback/discard uses Bax and does
not execute that selected callback. Immediate SQLite onCommit is not its run
point. Callback ordering, complete nested/concurrent behavior, effective gates
and full address/group cleanup remain unimplemented; destructive cleanup stays
disabled. **Implemented offline** recipe include_register_flow (boolean, default
false) retains supported register operands missing from legacy compact exports.
A literal CDO move-result destination regression verifies a register-only change
is visible with the option; default rendering and primary annotations are stable.
The cleanup bundle now has 23 topics/38 methods/28 files/214535 bytes. Cached
same-APK diff against 257f186 took 0.011257 seconds: six additions, sixteen
annotation-changed and one unchanged topic. All old offset/opcode/mnemonic and
catch ranges match; all six new direct dumps match those coverage lists.
No account was opened, live request sent, identity changed or keys deleted.
[Callback evidence and exporter limits](docs/protocol/peer-identity.md#cleanup-callback-commit-and-rollback-closure-2026-10-08).

Validation on system Rust 1.99.0: all 60 xtask tests, strict xtask/all-target
Clippy, xtask build, regenerated CLI reference, experimental OpenAPI library-symbol
check and formatting passed. Repository hygiene verified 941 local links, package
licenses, tracked-file budgets and 4210702 bytes across six checksum bundles.
Runtime Rust code is unchanged; workspace/runtime tests were not repeated for
this exporter-only Rust change.
Exact hash-pinned APK re-export with --check verified all 28 files/214535 bytes.


Sender-key selection and deletion observer closure (2026-10-08, **confirmed
static APK 2.26.37.71**): 0kX.A02 selects DISTINCT stored group IDs for the
translated sender/type/device; 1Uf.A02 slices eight groups per lock/delete block
and rechecks for at most three local passes. Its inner list helper receives
972/972 but operates on those at-most-eight groups; these are not server limits.
Argument concatenation puts the tuple collection before the group IDs. D3f
selector 41/0x29 is BqD identity-deleted, including notifyDeviceIdentityDeleted
and shared D4x effects; Fco deletion is a no-op for identity waits. The earlier
first-identity interpretation of 0k4.A0L is refuted and corrected: 0jc.A0K's
nonnull-key/no-old-value branch uses selector 42/BqB, while null-key/old-value
uses A0L/BqD. Complete effective gates, canonical mapping, cache/executor
semantics, concurrent scheduling and teardown runtime remain unimplemented.
Current cleanup bundle: 41 topics/56 selected declarations (one abstract),
46 files/395788 bytes. Same-APK cached diff against 72afc9d took 0.013700 seconds:
eighteen additions/twenty-three unchanged topics, all old topic texts equal.
All new direct declaration/instruction/catch coverage lists match. No account
was opened, live request sent, identity changed or keys/history deleted.
[Evidence and remaining port requirements](docs/protocol/peer-identity.md#sender-key-group-selection-and-deleted-identity-observers-2026-10-08).

Validation: actual hash-pinned bounded export, checksum-verified cached diff,
unchanged old topic bytes and all new direct declaration/instruction/catch
coverage checks passed. Regenerated CLI reference, experimental OpenAPI
library-symbol check, formatting and staged whitespace passed on system Rust
1.99.0. Hygiene verified 948 local links, licenses, tracked-file budgets and
4391955 bytes across six checksum bundles. No Rust runtime/tool source changed;
workspace tests/Clippy were not repeated for this source-only research chunk.


Canonical mapping row selector (2026-10-08): **confirmed static APK 2.26.37.71**
0Xx queries rank by descending sort/LID IDs, then 0eU's reverse TreeMap overwrites
equal sorts; the winner is greatest signed sort/smallest tied account row.
**Implemented/offline** default-library jid_map::select_account_row ports that
selection, preserving invalid winning rows before downstream checks. Independent
SQLite fixtures cover 36 synthetic cases, including all 24 permutations of a
tie/cross-phone set, signed extremes and invalid winners. New redacted
jid-map-row-inspect --input reports counts/presence/nonnegative/positive checks;
private values are omitted, known missing fields are diagnosed and malformed/
unknown/duplicate/type/budget errors refuse without partial stdout. Mapping
write authority, effective gates, freshness, complete reverse/bulk materialization,
worker scheduling and cleanup runtime remain unported. The extended cleanup
bundle has 57 topics/76 declarations (one abstract)/62 files/450854 bytes. Cached
same-APK comparison against eafb8fd took 0.028460 seconds: sixteen additions,
forty-one unchanged topic texts. Direct method/instruction/catch coverage matches.
No account was opened, live request sent, identity changed or keys/history deleted.
[Source, port and remaining mapping gaps](docs/protocol/peer-identity.md#canonical-key-mapping-and-row-selection-in-rust-2026-10-08).

Validation on system Rust 1.99.0: all 708 workspace/all-feature tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, 24 default CLI
checks, regenerated CLI reference, experimental OpenAPI library-symbol check,
xtask build and formatting passed. The independent SQLite fixture --check passed.
The compiled inspector selected an invalid synthetic winner without falling back;
an unknown-field invocation exited nonzero with no partial stdout/private-key
echo. Repository hygiene verified 957 local links, package licenses, tracked-file
budgets and 4447021 bytes across six checksum bundles. No account or live-server
access was involved; fresh runtime mapping/cleanup validation remains pending.


Contact LID ingestion (2026-10-08, **confirmed static APK 37.71**, offline-tested):
corrected per-user `lid@jid` to `lid@val`, gated by successful `usync/result/lid`
metadata and strict plain-LID validation. Wrong subtypes/zero device suffixes and
failed/missing protocol metadata cannot supply a LID. Existing CLI status exposes
the result. Historical ingested rows, identities and reservations remain intact.
Fresh live replies, refresh/backoff and full contact-type state parity remain
unverified. See [contact response evidence](docs/protocol/chat-actions.md#contact-lid-response-correction-2026-10-08).


Contact type/status inspection (2026-10-08, **confirmed static APK 37.71**):
newly ingested existence follows exact `in`, rather than supplied JID presence;
`out`/`invalid` are preserved in nullable `contact_type`. Missing/unsupported type
has a fixed local diagnostic. Historical rows keep their original boolean and
unknown type. `contacts status` inspects offline without queuing another request.
Fresh live types and full address-book state handling remain unverified.
[Evidence](docs/protocol/chat-actions.md#contact-type-and-offline-status-2026-10-08).


Contact receive projection/live check (2026-10-08): a 20-second receive-only
session authenticated cleanly and settled one original existing-peer contact
query as `in`. Contact replies now project automatically during receive and
recover at startup after a settlement/projection crash gap. Live evidence exposed
a local rejection of zero-component AD_JID LIDs; the parser now distinguishes
wire encoding from logical account type. A guarded offline `contacts repair-lid`
recovered the LID from that same consumed reply, retaining prior status/history;
repeat was unchanged. The 19 earlier actions, 9 acknowledged receipts and empty
outbox stayed unchanged. No query replay or chat send. Fresh post-fix replies,
native wire factory and full contact sync remain unverified.
[Evidence and negative results](docs/protocol/chat-actions.md#live-contact-projection-and-primary-wire-lid-2026-10-08).


Primary identity-read correlation (2026-10-08, offline): identity-only replies
and per-user directory refusals now compare actual address components across
Plain/AD_JID-zero encodings. Foreign namespaces/users, agents and companions stay
distinct; duplicate identity-only rows and unsupported kinds remain refused.
Successful signed-bundle matching was already component-based. Existing saved PN
inspection was already received; LID identity-only evidence was absent, so this
is fixture-demonstrated compatibility, not a new live identity milestone.
[Evidence and audit boundaries](docs/protocol/peer-identity.md#primary-wire-address-correlation-2026-10-08).

Directory refusal attribution (2026-10-08, offline): `peer-keys --show` now
reports fixed local invalid-reply diagnoses and binds numeric refusals to one
unambiguous selected-user/root context. Foreign first-error codes and duplicate
rows cannot masquerade as selected-peer refusals. Original action history and
retry reservations remain intact. APK 37.71 DF6 identity-only error handling
supports contextual attribution; duplicate guards are local policy and fresh
live refusal validation remains unverified.
[Evidence and limits](docs/protocol/peer-identity.md#directory-refusal-attribution-2026-10-08).

Identity-success wait consumer (2026-10-09, **confirmed static APK 37.71**):
the sole DEX Fco constructor caller is a feature-gated extensions/Flows path.
Its 30-second wait wakes callbacks after success or Exception timeout; those
callbacks reverify held keys. This does not establish incoming replay/session
repair. Eight new reproducible identity-wait topics preserve the caller, observer,
timed worker and verification chain; all other event consumers and complete
identity-only application remain unfinished. No account or network changed.
[Evidence and scope](docs/protocol/cross-device-sync.md#identity-wait-caller-and-extensions-verification-2026-10-09).

Identity-success consumers (2026-10-09, **confirmed static APK 37.71**): the
full DEX scan preserves 22 BdO/BdR declarations/bodies. Only Fco and AL2 have
nonempty BdO; the second waiter is a 30-second business-profile UI task.
No scanned callback directly decrypts or retries messages; transitive helpers
remain unknown. Primary LID receive's sender_pn prerequisite is a local guard,
not a proven APK requirement, and remains the next receive-routing trace.
No account or network changed. [Evidence and limits](docs/protocol/cross-device-sync.md#identity-success-consumer-inventory-2026-10-09).

Primary LID routing trace (2026-10-09, **confirmed static APK 37.71**): missing
sender_pn/username logs join nullable metadata construction rather than throwing.
The parsed username attribute is username; DFc's 13 field bindings are recovered.
Ordinary LID's type 18 selects the message's A05 route for sender-device lookup,
without consulting alias metadata. Rust's unconditional primary alias guard is
a local limitation; historical PN-marker stability and the raw stanza producer
remain next boundaries before a receive change. Six unrelated/special builder
write paths are recorded to avoid repeating them. No account or network changed.
[Evidence and limits](docs/protocol/peer-identity.md#primary-lid-route-and-nullable-aliases-2026-10-09).

Primary LID receive (2026-10-09, implemented/offline): new primary messages use
actual LID without requiring a phone alias. Original authenticated capture proofs
retain historical PN rows for duplicate/event/read lookup; no row rename, ratchet
replay or alias adoption from redelivery. Real-crypto msg/pkmsg, reopen/MAC failure,
legacy capture ambiguity and poll-target fixtures cover the correction. Fresh
live alias-less interoperability and full APK storage parity remain unverified.
[Evidence and scope](docs/protocol/peer-identity.md#primary-lid-receive-and-legacy-record-stability-2026-10-09).

- Primary LID legacy deduplication also retains consumed view-once markers using
  original route proof; alias-less redelivery cannot recreate plaintext or an event.
- Bounded receive smoke check authenticated and stopped cleanly after 45 seconds
  with reconnect/chat sends disabled; 15 decrypted records remained 15, with no
  fresh DM or alias-less live proof. Original action/receipt/outbox/event summaries
  were unchanged. The later consumed-marker fix has offline validation only.

Reaction receive correction (2026-10-09, implemented/offline): APK 37.71 C7G
requires target ID plus parseable remoteJid; malformed keys now remain opaque
with redacted key_validation diagnostics. Primary direct reactions retain proven
historical PN target keys while the reactor keeps its actual LID; consumed
markers and original records are preserved. Status likes now require the actual
status@broadcast singleton. Own/outgoing/group/companion reconciliation and fresh
live reaction validation remain incomplete. A 21-topic/26-method checksum bundle
preserves the trace and cached diff (0.028679 seconds).
[Evidence, negative findings and limits](docs/protocol/reactions.md).
Validation: 727 workspace/all-feature tests, strict Clippy, warning-free Rustdoc,
24 default CLI checks and CLI/API reference checks passed. Retained-account
offline reprojection upgraded zero events; action/receipt/outbox/event and
peer-key summaries stayed unchanged, with two valid saved reaction keys. Capture
inspection now uses one timestamp snapshot and handles overflowing ages; a
subprocess regression proves malformed-key diagnostics remain visible. No new
chat/live request or identity/reset/retry authority was added.

Outgoing reaction attribution (2026-10-09, implemented/offline): a unique normal-
outbox wire reservation plus exact address or existing authenticated PN/LID pair
retains the original sent-message key. Reactor identity remains actual; attempts,
facts, progress and uncertainty are unchanged. Lookup now checks all pages and
uncommitted rows for duplicate wire IDs. Outgoing quote bodies require the actual
saved intent's chat, rather than just the caller's reference. Unknown targets,
legacy lab-only sends, full native alias/companion/group reconciliation and fresh
live reactions remain unfinished.
[Sources, negative findings and fixtures](docs/protocol/reactions.md#outgoing-intent-attribution-and-quote-scope-2026-10-09).
Validation: 730 workspace/all-feature tests, strict Clippy, warning-free Rustdoc,
24 default CLI checks and regenerated CLI/API reference checks passed. Offline
reprojection upgraded zero events and kept retained summaries unchanged. A
45-second receive-only live smoke check authenticated and stopped cleanly, with
15 decrypted records remaining 15; no fresh reaction/DM established new target
interoperability. Original uncertain actions, outbox and receipts were preserved.

### Poll vote structural guard (2026-10-09)

APK 2.26.37.71 `Ban.A01` calls the same `C7G.A00` parent-key validator as
reactions; `Clc.A02` independently confirms the shared `BoQ.A00` mapper on
the poll path. The new seven-topic/seven-method checksummed source bundle
contains 12 files/69,090 bytes and reproduces with the exact APK/recipe.
Poll decoding now checks supported ID/remoteJid, timestamp presence and
ciphertext/IV presence with a twelve-byte IV. `lab-inbox --inspect` explains
refused saved updates with fixed redacted validation labels. Zero/negative
present timestamps, absent fromMe/metadata and present empty ciphertext are
not incorrectly rejected structurally; AEAD validation remains separate.
Native JID parity, feature selection, expiry/edit/quiz behavior and outgoing
poll chat binding remain unfinished. No new poll/vote was transmitted.
[Source, implementation and remaining chat issue](docs/protocol/poll-votes.md).

Validation: 732 workspace/all-feature tests, strict Clippy/Rustdoc, CLI/API
checks and hygiene passed. Recovered-profile offline projection upgraded zero
events; peer/action/receipt/outbox/event summaries and normalized capture
diagnostics were unchanged. That profile has no saved poll updates, so no
fresh or retained-poll interoperability is claimed.

### Own-poll chat binding (2026-10-09)

Own poll vote projection now binds the creation's existing saved chat to the
actual primary voter through exact PN/LID components or an existing authenticated
association. AEAD projection retains the original chat and actual actor;
historical markers remain unchanged. Missing legacy binding stays unknown,
unattributed/companion references stay opaque, and corrupt saved state refuses
locally. Vote preparation checks the actual saved chat before identity/entropy,
with fixed missing/foreign/invalid diagnoses. `lab-inbox --inspect` exposes a
read-only stored-selector binding label without printing identities or secrets.
The earlier structural guard remains effective. Native parent-store/feature/
expiry/edit/quiz/companion/group parity and fresh live poll attribution remain
incomplete. [Evidence and boundaries](docs/protocol/poll-votes.md#saved-chat-attribution-2026-10-09).

Validation: 734 workspace/all-feature tests, strict Clippy/Rustdoc, 24 default
CLI tests, CLI/API reference checks, formatting and hygiene passed. Offline
projection upgraded zero events. A 45-second receive-only live window was
active/authenticated and stopped cleanly; peer keys, action outcomes, receipts,
empty outbox and events stayed unchanged. Fifteen decrypted payloads remained
fifteen; no fresh DM/poll/vote proved new live attribution. Two uncertain sent
actions were preserved without replay; reconnect and chat sending were disabled.

### Identity-change receipt/version/token scope (2026-10-09)

Static APK 2.26.37.71 D4x receipt selection now has attributable helper coverage:
primary message-ID intersection, scoped null-timestamp companion/device/add-on
rows, coexistence/status paths, persistent primary_device_version increments
and conditional refresh of an existing unexpired privacy token. The reviewed
bundle preserves 23 topics/53 declarations, 28 files/323,094 bytes, with exact
register operands and catch metadata. Runtime mapping/migration, cache callback
closure and refresh-job lifecycle remain incomplete. Existing Rust incoming
token storage, authenticated identity archival and sender-key exposure policy
are unchanged; native receipt caches are not receipt-outbox ACK reservations.
No private state or server request was changed. [Full source and next port
boundaries](docs/protocol/identity-receipts.md).

Validation: exact APK/recipe re-export, CLI/API checks and hygiene passed. Cached
23-topic self-comparison took 0.026378 seconds; no cross-release equivalence or
new runtime/live behavior is claimed. Incoming PN/LID delivery-fact attribution
remains a separately identified receive gap to trace and correct next.

Direct receipt attribution (2026-10-09, **confirmed static / local correction**):
APK 37.71 `CmG`/`Cmk`/`DDH` keep message chat and receipt actor separate;
stale `ClU` attribution is corrected. Primary normal-outbox receipts now bind
the unique saved wire intent to the actual actor through exact PN/LID or an
existing authenticated association in either direction, within one transaction.
Actual actor, original IDs, progress, uncertainty and receipt-send reservations
are preserved. Legacy PN records page history; ambiguous/corrupt matches roll
back. Native canonicalizer gates, identity cleanup and fresh live PN/LID
receipt attribution remain unverified. [Evidence and boundaries](docs/protocol/direct-receipts.md).

Receipt correction validation: 739 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and repository hygiene
pass. The 12-topic APK bundle reproduces exactly; cached self-diff is 0.017892
seconds. A 45-second receive-only session authenticated and stopped cleanly;
no fresh user message or qualifying receipt arrived. Saved peer keys, actions,
receipt ACKs, outbox and events remained unchanged. No chat was sent, no
uncertain action replayed. Fresh live namespace attribution remains pending.

Receipt canonicalizer (2026-10-09, **confirmed static / local correction**):
APK 37.71 `0Yo.A07` selects PN/LID by global migration state and exempts
PNH_CTWA chats; `0Xw.A0I` has explicit subtype filters. Bot conversion uses
a separate gated static dictionary. Effective state and complete application
remain unknown/unported. `0hs.A0C/A0M` defaults only for absent attributes;
Rust now refuses present nontext/empty/unknown receipt types without false
delivery mutation, covered through actual wire encode/decode and store reopen.
No account identity, map or send state is reset. [Evidence and limits](docs/protocol/receipt-canonicalizer.md).

Canonicalizer validation: 740 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, CLI/API/reference checks and hygiene pass. The 24-method
source bundle reproduces exactly and cached self-diff takes 0.007559 seconds.
Offline projection upgraded zero events; saved keys, actions, receipt ACKs,
outbox, events and retained inspection stayed identical. No new live request
or saved-account mutation; malformed types and native runtime gates remain
unverified live.

Group participant receipts (2026-10-09, **confirmed static / local implementation**):
APK 37.71 `Cmk`/`DDH`/`ReceiptProcessingJob`/`Cmn` retain separate message key,
group chat and participant device through deferred receipt processing. Rust now
tracks ordinary group delivery/read/played against the original addressed-device
set, retaining actual PN/LID actors and companion indices. Frozen targets are a
local guard; historical out-of-band targets remain unknown. Comparison sends
preserve new targets before writing; `group receipts --request-id` inspects only
redacted counts offline. Normal group send preparation, native orphaned receipt
application/aggregation and fresh live participant attribution remain unfinished.
[Evidence and boundaries](docs/protocol/group-receipts.md).

Group receipt validation: 749 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and formatting pass.
The 21-method / 15-file / 124,482-byte source bundle reproduces exactly; cached
self-comparison takes 0.019472 seconds. Offline inspection found seven saved
group reservations across twelve request files, with zero tracked participant
receipt devices. Projection upgraded zero events; saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized inspection were unchanged.
No historical receipt capture was replayed or new group send prepared.

**Observed live regression:** a 45-second receive-only session authenticated,
activated and stopped cleanly with chat sending and reconnects disabled.
Peer keys, action outcomes, receipt ACKs, empty outbox, events and retained
noncapture inspection stayed unchanged. Transport capture rows increased from
442 to 446; these are not four user messages. All seven saved group summaries
stayed unchanged with zero tracked participant receipt devices. No fresh group
receipt established live attribution, and no uncertain send was replayed.

Shared group fanout (2026-10-09, **confirmed local correction**): the comparison
CLI now uses library Snapshot/Prepared staging and atomic crypto reservation.
Fresh mode retains the original ratchet for concurrency checks instead of
allowing a late same-identity advance to be overwritten. Pins remain enforced;
identity-triggered own-key rotation archives the original generation and leaves
uncertain sends intact. Existing fanout-inspect adds nullable missing-bundle and
cached-session counts offline. Native query batching and normal group outbox
integration remain unfinished; no new group/status send is claimed.
[Evidence and port boundaries](docs/protocol/group-fanout.md).

Shared fanout validation: 753 workspace/all-feature tests passed, plus the final
CLI inspection regression, strict Clippy, warning-free Rustdoc, default CLI,
API symbol/reference, formatting and hygiene checks (1,117 local links; 15 APK
bundles). Offline projection upgraded zero events. Saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized retained inspection were
unchanged. No server connection, group/status send, request replay or saved
account crypto mutation occurred in this chunk. Shared staging/rotation is
verified offline; live group/status validation and normal actor wiring remain.

Directory-key batches (2026-10-09, **confirmed static / local implementation**):
APK 37.71 `0y0.A00` confirms batched encrypt/key/user construction and optional
identity-reason/PQ/context branches; effective native producer flags and limits
remain unknown. Group/status lookup now covers all frozen bundle-required
devices in local 32-device pages, durably reserved before write, with complete
signed responses reused only for the same plan. Unknown/refused/invalid/legacy
unattributed outcomes block without resets. New group key-queries inspection
prints redacted progress offline. Normal group actor integration, optional native
fields, own inventory and fresh live large-group coverage remain unfinished.
[Evidence and boundaries](docs/protocol/device-key-batches.md).

Directory-batch validation: 758 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and formatting pass.
Offline projection upgraded zero events; saved peer keys, action outcomes,
receipt ACKs, outbox, events and normalized retained inspection stayed unchanged.
Twelve existing request files were inspected offline: no new batch records,
eleven with neither batch nor historical key outcome, one with a preserved
legacy outcome and unknown frozen counts. No server request or message was sent.

The existing groups APK recipe now anchors get-pre-key-batch and maps its writer
to messaging.rs and device_key_batches.rs for future release diffs. Exact
re-export changed only targets/checksums metadata, not native method bodies;
all 44 files / 928,943 bytes / 43 selected methods reproduce. Cached self-diff
is 0.047877 seconds, not a claim of cross-release equivalence. Large-group live
lookup, native optional-field gates and normal group actor wiring remain pending.

Group routing snapshot (2026-10-09, **confirmed local correction**): planning
now reads membership/lists/primary authority together; shared fanout revalidates
the supplied plan and refuses changed routing rows inside the original send
transaction. Fresh/cached crypto and caller reservation roll back together,
with newer rows preserved. The native TEE node-token equality is not evidence
of a membership transaction. Normal group outbox actor wiring, own inventory
and fresh live group fanout remain unfinished.
[Evidence and limits](docs/protocol/group-fanout.md#routing-snapshot-correction-2026-10-09).

Routing-guard validation: 760 workspace/all-feature tests passed, alongside
strict Clippy, warning-free Rustdoc, default CLI, API symbols/CLI reference,
formatting and repository hygiene (1,129 local links; 15 APK bundles). Offline
projection upgraded zero events. Saved peer keys, action outcomes, receipt ACKs,
outbox, events and normalized retained inspection stayed unchanged. No server
request, message transmission or saved-account crypto mutation occurred.

Normal group outbox (2026-10-09, **confirmed local implementation**): the actor
now resolves fresh membership, selected message-context device pages and all
missing signed key pages, then reserves shared fanout crypto and the original
normal intent together before write. Existing private JSON group/text/image/voice
uses outbox-send; group uploads bind the chat. Redacted outbox-group-preparations
shows original phase, fixed blockers, counts and scoped batch outcomes offline.
Cold PN/LID real-crypto tests, 130-user batches and injected four-stage write cuts
cover preparation and once-only reservations; simulated late ACK/participant
delivery settles without another attempt. Normal group live acceptance, broader
message kinds, answered-preparation resume, group peer replacement and full own
inventory remain unfinished.
[Evidence, invocation and boundaries](docs/protocol/normal-group-outbox.md).

Validation for this chunk: 767 workspace/all-feature tests and strict Clippy,
Rustdoc, default CLI/API/reference/format checks passed. The upgraded saved
account authenticated and stopped cleanly after a 45-second receive-only window,
with chat sends and reconnect disabled. No live outgoing group was tested.

Signed group peer replacement (2026-10-09, **confirmed local implementation**):
normal group fanout now honors the explicit outgoing APK-authenticated policy for
verified signed remote bundles, archives old pins/ratchets and emits identity
events in the original send transaction, and rotates an exposed current chain
while leaving other exposed groups pending. Own identities remain pinned;
changed-primary fanout with old companion authority reports
companion_authority_changed. Interrupted/blocked preparations are not reset or
replayed. Cached-session refresh, companion reconciliation, preparation resume
and live group identity transition remain unfinished.

Signed group policy validation: 770 workspace/all-feature tests and strict
Clippy, Rustdoc, default CLI/API/reference/format and hygiene checks passed.
A 45-second receive-only check authenticated and stopped cleanly; peer keys,
actions, receipts, outbox and events stayed unchanged. Five transport captures
were added. No live outgoing group identity transition was tested.

Typed group quotes/reactions (2026-10-09, **confirmed local implementation**):
reply/react now accept an original --participant for incoming group targets and
--ours derives frozen own identity. Normal fanout carries the existing APK
quote/reaction encodings, checks retained parent attribution, and blocks late
reference changes without a wire attempt. Existing default IDs/queued records
are preserved. Private JSON quote, media quotes, incoming group/id collisions
and fresh live group rendering remain unfinished.

Typed group reference validation: 773 workspace/all-feature tests and strict
Clippy, Rustdoc, default CLI/API/reference/format and hygiene checks passed.
A 45-second receive-only check authenticated and stopped cleanly with sends and
reconnect disabled; peer keys, actions, receipts, outbox and events stayed
unchanged. Five transport captures were added. Group quote/reaction handset
rendering and the participant-aware incoming storage port remain unverified.

Participant-scoped group inbox (2026-10-09, **confirmed local implementation**):
new records distinguish original participant namespace/agent and wire ID, retain
original device routes, and preserve legacy records/event markers. Quotes use the
selected participant's body. `mark-read --participant` selects a group member;
ambiguous unqualified group/ID reads refuse atomically. Automatic reads remain
opt-in and refuse unsupported agent selection. Native SQLite-key parity and live
same-ID group collision behavior remain unknown. See [group inbox](docs/protocol/group-inbox.md).

Group inbox validation: 775 workspace/all-feature tests passed, together with
strict Clippy, Rustdoc, default CLI, library-symbol, regenerated CLI reference
and formatting checks. Repository hygiene passed (15 APK research bundles,
5,815,284 bytes). A 45-second receive-only check authenticated and stopped
cleanly with sends/reconnect disabled; peer keys, actions, receipts, outbox and
events were unchanged. Original capture shapes were preserved; captures grew
from 461 to 466. No fresh group send, explicit read request or live same-ID
collision was performed. The prior incoming storage gap is now corrected
locally; native storage parity and fresh group handset rendering remain unknown.

Incoming group reaction authors (2026-10-09, **confirmed local correction**):
BZa participant now selects the original author; a third member's message no
longer becomes an outgoing target merely because fromMe is false. Matching
legacy/new parent routes retain actual devices; consumed bodies stay consumed.
Known own PN/LID selects outgoing targets; unsupported agent/malformed/foreign
group keys remain opaque. Inspection adds fixed target_validation labels. Parent
ambiguities refuse atomically; old event markers are preserved. Fresh live group
reactions and native own/hosted normalization remain unknown. See [reactions](docs/protocol/reactions.md#incoming-group-original-author-2026-10-09).

Group reaction validation: 778 workspace/all-feature tests and strict Clippy,
Rustdoc, default CLI, API/library symbols, CLI-reference regeneration and format
checks passed. Hygiene checked the unchanged 15 APK evidence checksum bundles
(5,815,284 bytes). Two retained group keys report target_validation=valid; no
historical events were rewritten. A 45-second receive-only session authenticated
and stopped cleanly with sends/reconnect disabled. Peer keys, actions, receipts,
outbox and events stayed unchanged; original capture shapes were retained and
captures grew from 466 to 471. Fresh live group reaction targeting is unverified.
BoQ.A00's BO0 predicate, effective caller gates and contradictory fromMe/author
normalization remain unported; exact known own PN/LID matching is a local
projection contract. One temporary-store ownership test refusal passed isolated
and on full rerun without changing ownership guards; its cause remains unknown.

Native group reaction predicate/branch (2026-10-09, **confirmed static/local**):
the new 18-topic APK 37.71 bundle closes BO0's own PN/LID comparison and
user/server/type equality. Ordinary groups consult it only for false/default
fromMe; true/present-author stays incoming, correcting our prior unconditional
own classification. Nonzero device authors no longer fold into UserJid equality.
Nullable-author fallback, actor producer authority, zero-device text and
authenticated own-envelope context remain unported. No identity/alias adoption
or historical-marker rewrite is introduced. See [group mapper](docs/protocol/group-reaction-mapper.md).

Ordinary remote-key correction (**confirmed APK 37.71 BoQ.A00**, `00fe..0106`):
plain PN/LID/group remote keys may differ from the envelope group. The update
and parent selection now remain in that authenticated envelope group. The
previous exact-match refusal was our local limitation; status/device/unported
remote kinds stay separate. Native nullable-author and own-envelope source
context remain unported; no fresh live group-targeting result is claimed.

**Observed receive validation (2026-10-09):** two bounded 45-second windows
authenticated and stopped cleanly; the second ran the final ordinary-remote
correction. Both disabled chat sends and automatic reconnect. Original captured
records were retained; capture rows grew from 471 to 481. Peer keys, actions,
receipts, outbox and durable events were unchanged. Two retained group reaction
keys still inspected as target_validation=valid. These are retained-key and
receive-runtime checks, not fresh live group reaction/own-envelope proof.

Native mapper verification: 778 workspace/all-feature tests, strict Clippy,
Rustdoc, default CLI, API/library-symbol checks, CLI-reference regeneration and
formatting passed. Final API edits passed five OpenAPI tests and rebuilt the CLI.
The 18-topic APK export/check reproduces 23 files / 86,053 bytes; hygiene validates
16 checksum bundles / 5,901,337 bytes. Cached verified self-diff took 0.008 seconds;
new-release extraction and re-anchoring speed remains unmeasured.

Group envelope direction (2026-10-09, **confirmed static/local**): the APK 37.71
CnN/CgC/Cih/Bp2/Cna producer chain now establishes the ordinary network group's
selected sender and own-envelope flag. Sender-aware reactions preserve payload
direction for a known own sender; remote true/absent-or-empty-author references
fall back to the authenticated sender user. Inspection verifies the saved original
route against its selector. Existing projection markers, consumed bodies, parent
routes, outbox records and trust admission remain preserved. This supersedes the
ordinary own-envelope/source-producer gap above; full own-device synchronization,
hosted/status references, special chat normalization and fresh live own-device
reaction behavior remain unverified/unported. See [envelope direction](docs/protocol/group-envelope-own.md).

**Confirmed verification:** 779 workspace/all-feature tests passed (652 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), along with strict
all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI tests,
API/library checks, CLI-reference regeneration and formatting. The final embedded
API/build check also passed. The 24-topic export and revalidation agree on
29 files / 127,370 bytes. Hygiene validates 17 APK checksum bundles totaling
6,028,707 bytes.

**Observed receive check (2026-10-09):** one 45-second window authenticated and
stopped cleanly with chat sends and automatic reconnect disabled. Original
captures were retained; capture rows grew from 481 to 487. Peer keys, actions,
receipts, outbox and durable events stayed unchanged. Both retained group keys
still inspected as target_validation=valid using their original sender routes.
No historical markers were repaired or removed. This checks the receive runtime
and retained references; it does not establish fresh live own-device/group
reaction handling or human handset rendering.

Group keep/unkeep receive attribution (2026-10-09, **confirmed static/local**):
APK 37.71 D9L.A01 maps BV3.key through BoQ, just like ordinary group reactions.
Reception now resolves the original author independently of the keeper, uses
authenticated own-envelope context and retains matching parent devices. Incoming
updates preserve consumed bodies, outbox facts and existing event markers.
`lab-inbox --inspect` exposes fixed group keep target_validation labels using
original saved routes. Direct keep projection retains its existing behavior.
Normal group keep sending, native downstream authorization/expiry enforcement,
and fresh live group keep/handset rendering remain unimplemented or unverified.
See [group keep](docs/protocol/group-keep.md).

**Confirmed verification:** 780 workspace/all-feature tests passed (653 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), together with
strict all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI,
API/library-symbol checks, CLI-reference regeneration and formatting. Export
and checksum revalidation reproduce six topics / 11 files / 84,246 bytes.
Repository hygiene validates 18 APK bundles totaling 6,112,953 bytes.

**Observed receive validation (2026-10-09):** one bounded 45-second window
authenticated and stopped cleanly with chat sends and automatic reconnect
disabled. Original captures were retained; capture rows grew from 487 to 492.
Peer keys, actions, receipts, outbox and durable events stayed unchanged. Two
retained group reaction targets remain valid; no retained group keep payload
was available to validate live attribution. No fresh group keep/unkeep, group
send or human handset rendering is claimed. Native keep timestamp ordering,
authorization, expiry enforcement and outgoing group keep remain open gaps.

Outgoing keep key research (2026-10-09, **confirmed static**): APK 37.71
Bai.AD7 serializes the original parent key through CSZ.A01. The new 22-topic
bundle traces explicit id/fromMe/participant setters, gated parent/edit/partial-ID
lookup and retained author-device selection. A supplied author may be omitted
when the gated lookup misses; remote namespace mapping and effective gate values
remain unknown. No Rust send behavior changed. Normal group keep sending,
native permission/expiry checks and fresh live keep/unkeep remain unfinished.
[Evidence and implementation boundaries](docs/protocol/group-keep-send-key.md).

**Confirmed research validation:** exact re-export/check reproduces 22 topics /
27 files / 75,556 bytes; checksum-verified self-diff is unchanged. All 63 xtask,
50 experimental CLI and one API-library symbol tests passed, including CLI
reference regeneration. Hygiene validates 19 APK bundles / 6,188,509 bytes.
No account connection, group draft, message send or private state change was
performed. This verifies the research export, not outgoing keep implementation.

Normal group keep/unkeep (2026-10-09, **confirmed static/local implementation**):
APK 37.71 original-key, namespace, creation-clock and stanza traces now ground
a separate group codec and typed keep --participant/--ours/--undo/--dry-run.
Normal fanout dispatch preserves the original accepted timestamp and original
intent reservations. Shared quote/reaction/keep parent selection now reads one
snapshot and revalidates inside the crypto/outbox reservation transaction;
changed references roll back with reference_changed. Consumed bodies stay
consumed and keeps/reactions do not parse parent text previews. Native
permission/expiry enforcement, adjusted-clock synchronization, complete parent
normalization, full own-device sync and fresh group ACK/rendering remain open.
[Evidence and limits](docs/protocol/group-keep-outbox.md).

**Confirmed validation:** 784 workspace/all-feature tests (656 API, 4 property,
9 crash, 51 CLI, one API-library symbol, 63 xtask), strict Clippy, warning-free
Rustdoc, default CLI/API checks, regenerated CLI reference, formatting and build
passed. APK re-export/check reproduces 33 topics / 38 files / 259,783 bytes;
cached checksum-verified self-diff is unchanged in 0.017053 seconds, not a
new-release extraction benchmark. Hygiene checks 1,237 local links and 20 APK
bundles / 6,448,292 bytes. Offline keep dry-runs enqueue nothing; missing group
participant refuses. Saved account facts and 492 retained captures were unchanged
before live validation.

**Observed receive validation:** a single completed 45-second window
authenticated and stopped cleanly with sends and automatic reconnect disabled.
Peer keys, actions, receipts, outbox, events and retained original capture shapes
stayed unchanged; captures grew from 492 to 497. Two retained group reaction
references remain valid; no group keep payload was available. No new group draft,
keep/unkeep transmission, group ACK or handset rendering was tested. An earlier
local ownership refusal resulted from overlapping offline inspection and receive
orchestration; its logs were preserved and inspection completion verified before
this window. No server retry, account reset or ownership bypass was added.

2026-10-09 — **Confirmed local/static:** incoming keep events now preserve
optional payload milliseconds and their own envelope ID, independently of the
original target and stanza time. Literal legacy-journal records remain readable
and retain their shape; reopening preserves group update metadata without
republication. APK 37.71 native ordering prefers newer envelope time, then newer
payload time, then the smaller action ID. Permission/expiry enforcement and
fresh live keep reception remain unverified. [Evidence and limits](docs/protocol/keep-timestamps.md).

Keep metadata verification: 786 workspace/all-feature tests, strict Clippy,
Rustdoc, default CLI, API-symbol checks, CLI reference and formatting passed.
One 45-second receive-only window authenticated and stopped cleanly with sends
and reconnect disabled. Captures grew 497→502 with originals retained; keys,
actions, receipts, outbox and event summaries were unchanged. No group keep
payload was available, so fresh keep semantics remain unverified.

2026-10-09 — **Confirmed static:** identity-change token refresh now has traced
outbound issue/send storage, bucket expiry formula, request destination/time,
callbacks and explicit completion/cancellation/retry branches. Incoming token
bytes are not its outbound state. Native 4xx IQ refusal suppresses job retry;
base terminal dispatch, active properties and canonical counterpart gates remain
unknown. Rust refresh sending/storage is still unported; account state was
untouched. [Evidence and port boundary](docs/protocol/identity-token-refresh.md).

Token-refresh research validation: 26 topics / 31 files / 104,087 bytes reproduce
from the comparison APK; checksum/self-diff, 63 xtask tests, API symbols, CLI
reference, formatting and repository hygiene passed. Source changes are a
comment only. No live refresh or identity replacement was attempted.

2026-10-09 — **Confirmed static/local:** traced the normal server-acceptance
producer of outbound token scheduled timestamps, its raw message exclusions,
namespace transfer and separate call producers. Scheduled timestamp state is
not proof of token server acceptance. New apk-smali --signatures and recipe
include_method_signatures expose complete DEX descriptors, distinguishing two
DNf constructors with identical shorties while preserving legacy defaults.
Token scheduling/refresh runtime remains unported; account state was untouched.
[Evidence, invocation and boundaries](docs/protocol/token-issue.md).

Token-issue/full-signature validation: 788 workspace/all-feature tests, strict
Clippy, Rustdoc, default CLI/API/reference/format and build checks passed. New
14-topic evidence and the prior default-mode bundle reproduce; self-diff took
0.022072 seconds. No account operation or live token/identity request was made.

2026-10-09 — **Confirmed local:** manual peer verification now refuses a saved
own LID with no user before writing metadata. A failing-before regression and
reopen checks cover mark/unmark refusals, unchanged pins/sessions/events and
valid remote decisions. The initial zero-device bypass hypothesis was refuted:
valid textual zero forms already normalize; no parser change was needed.
This is our local prerequisite, not a server refusal. Native malformed-login
parity and fresh live identity transitions remain unknown. No real account
verification decision or network request was made.
[Evidence and reproduction](docs/protocol/own-verification.md).

Own-prerequisite verification: 790 workspace/all-feature tests (660 API, four
property, nine crash, 51 CLI, one API symbol and 65 xtask), strict Clippy,
warning-free Rustdoc, default CLI/API checks, regenerated CLI reference,
formatting and build passed. Hygiene checked 1,283 local links and 23 APK
bundles / 6,988,623 bytes. Validation used synthetic stores only.

2026-10-09 — **Confirmed static:** the two remaining outbound token timestamp
writers restore imported chat-transfer/companion-history sender state, separately
from received token bytes. The companion key-parse failure can still reach token
restoration; no valid-key admission shortcut is inferred. The hidden descriptor
id-array helper explains the field decoder's honest 59/58 refusal. Manual tag and
presence mapping is preserved with the helper evidence. History source authority,
active gates and Rust import/refresh remain unported. Account state was untouched.
[Evidence and limits](docs/protocol/token-import.md).

Token-import research validation: seven topics / 12 files / 412,652 bytes
reproduce from the APK. Cached self-diff reports seven topics and four metadata
artifacts unchanged in 0.021659 seconds; this is not a cross-release benchmark.
All 65 xtask tests, CLI-reference regeneration, API/library-symbol check,
formatting and diff hygiene passed. Repository hygiene checked 1,287 local links
and 24 APK bundles / 7,401,275 bytes. Runtime source changes are comments only;
no account operation or live token/history request was made.

2026-10-09 — **Confirmed local/static:** authenticated primary replacements
now persist the APK-style per-user device version atomically with their archive
and follow-ups, exposed in PeerPinState through existing offline inspection.
Missing old state stays null; first tracked change starts at 1. PN/LID namespaces
stay separate, own/companion exclusions remain, and receipt reservations are
preserved. Native receipt version use and fresh live increments remain unverified.
[Evidence and limits](docs/protocol/primary-device-version.md).

Primary-version verification: 792 workspace/all-feature tests (662 API, four
property, nine crash, 51 CLI, one API symbol and 65 xtask), strict Clippy,
warning-free Rustdoc, default CLI/API checks, regenerated CLI reference,
formatting and build passed. Hygiene checked 1,296 local links and 24 APK
bundles / 7,401,275 bytes. Real encrypted direct/group transition fixtures
exercise the receive integration; isolated tests cover both archive sources,
rollback, reopening, namespace separation, signed-long overflow and legacy JSON.

**Observed compatibility:** one completed 45-second receive-only window
authenticated and stopped cleanly, with chat sends and automatic reconnect
disabled. Saved peer facts (apart from the new absent-version null field),
actions, receipts, outbox and event summaries were unchanged. Captures grew
502→508 with all original shapes retained; two retained group reaction targets
remain valid. No fresh identity transition occurred, so live version increments
remain unverified. The original 23-topic / 28-file / 323,094-byte APK receipt
bundle reproduced byte-for-byte.

**Observed inspection clarification:** direct `peer-identity` checks with the
previous and rebuilt binaries match after omitting only the new null counter.
Two inspected primary namespaces have absent historical versions. `peer-keys
--show` exposes directory match diagnostics, not PeerPinState; its original
compatibility comparison remains valid but cannot prove counter output. The API
and invocation description were corrected to name `peer-identity` only.

2026-10-09 — **Confirmed static:** native pending device-receipt rows capture
nullable send-time primary versions; existing-row timestamp updates retain them.
The ordinary sync/resend path compares prior/current values with nullable equality,
plus a gated absent-to-1 exception. The raw hosted branch has additional gates;
0x6ebd does not surround all ordinary comparisons. Historical Rust sends lack
this snapshot and remain unknown. Rust resend/snapshot runtime remains unported;
no saved reservation or account was changed. [Evidence and next input](docs/protocol/receipt-version.md).

Receipt-version research validation: 14 topics / 19 files / 135,279 bytes
reproduce exactly from the comparison APK. Cached self-diff reports all topics
and four metadata artifacts unchanged in 0.022983 seconds, not cross-release
equivalence. All 65 xtask tests, CLI-reference regeneration, API/library-symbol
check, formatting and diff hygiene passed. Repository hygiene checked 1,304
local links and 25 APK bundles / 7,536,554 bytes. Runtime changes are comments
only; no account operation, live resend or receipt reservation change occurred.

2026-10-09 — **Confirmed local/static:** normal direct/group original-send
reservations now capture nullable per-user primary versions after authenticated
replacement hooks, atomically with their original state. All group plan devices
are included. Offline outbox-versions exposes hashes/counts and prior/current
values; absent historical snapshots remain unknown. Missing JSON version members
and malformed state refuse without private echoes. Native resend/receipt-cache
runtime remains unported; no new live send was authorized or performed.
[Evidence and limits](docs/protocol/send-version-snapshot.md).

Send-version validation: all 795 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, 26 default CLI checks, API/library-symbol check,
CLI-reference regeneration, formatting and experimental CLI build passed.
Hygiene checked 1,313 local links and 25 APK bundles / 7,536,554 bytes.
Offline retained-profile CLI checks left store inspection and queued intents
unchanged; invalid IDs and unknown intents refused without input echoes.
There were no retained group preparations to inspect; historical missing-state
coverage is from fixtures. New send capture has no live validation in this chunk.

2026-10-09 — **Confirmed static/local audit:** native sync/resend now has a
22-topic trace of its distinct encrypt/key_fetch request, existing remote
registration encoding, reply/worker completion, original receipt-device
subtraction and post-version target mutation. An empty/errored-device result
can complete true; callback completion is not per-device usability. Expiry
is strictly past, not equal-time rejection. Rust lacks key_fetch and original
per-device receipt membership; user counts cannot reproduce removeAll.
Current nullable user snapshots are preserved; no live worker/resend, account
operation or retry-guard relaxation was added. [Evidence and next port inputs](docs/protocol/sync-resend.md).

Sync/resend research validation: all 22 source topics / 27 files / 272,318
bytes reproduced from the exact comparison APK. Cached self-diff verified
22 unchanged topics and four unchanged metadata artifacts in 0.031055 seconds;
this is plumbing verification, not cross-release equivalence. All 65 xtask
tests, CLI-reference regeneration, API/library-symbol check and formatting
passed. Repository hygiene checked 1,324 local links and 26 APK bundles /
7,808,872 bytes. Runtime changes are comments only; no account operation,
key-fetch query or live resend was performed.

2026-10-09 — **Confirmed local/static:** original send snapshots now preserve
exact per-user device components; outbox-versions exposes only component/hash
rows. Historical count-only rows remain unknown and untouched. Group receipts
use the full original plan, validate envelope distributions as a subset and
refuse contradictions atomically. Current fanout sends SKDMs to every target;
omitted-distribution coverage is fixture/native compatibility, not a reproduced
current live failure. Native mutable receipt lifecycle/resend remains unfinished.
[Evidence and limits](docs/protocol/send-version-snapshot.md).

Exact-device validation: all 798 workspace/all-feature tests passed (668 API,
4 property, 9 crash, 51 CLI, 1 API-symbol and 65 xtask), alongside strict
Clippy, warning-denying Rustdoc, 26 default CLI checks, CLI-reference/API checks,
formatting and experimental CLI build. Hygiene checked 1,329 local links and
26 APK bundles / 7,808,872 bytes. Native-shaped subset/contradiction fixtures
cover planned companions, unplanned devices, immutable send state, reopening
and legacy wire-only fallback. Old count-only snapshots remain readable;
malformed device IDs refuse with redacted diagnostics.
Offline retained-profile CLI checks left store inspection and queued intents
unchanged. No retained group preparations existed to inspect; no new live
message or native pending receipt cache is claimed from these fixtures.

2026-10-09 — **Confirmed static:** receipt application now has an eight-topic
trace through 14e/Cmn and the DDH device/time/status/key getters. Cmn.A00
selects CNn.A00 for a status predicate and separately 15A.A07 for other
statuses. CNn/CYV row policy and complete clocks/mapping remain untraced;
no mutable receipt ledger or runtime status promotion is claimed.
[Writer boundary and next inputs](docs/protocol/receipt-apply.md).

Receipt-application validation: eight source topics / 13 files / 112,789 bytes
reproduced exactly from the pinned comparison APK. Cached self-diff verified
eight unchanged topics and four unchanged metadata artifacts. Repository
hygiene checked 1,336 local links and 27 APK bundles / 7,921,661 bytes; diff
hygiene passed. This follow-up changes research/docs only; the preceding
cdca7ad runtime chunk passed all 798 workspace tests and strict lint/doc/CLI
checks. No new runtime implementation or live receipt application is claimed.

2026-10-09 — **Confirmed static:** CNn is a receipt value factory, not a row
writer. Its settings branch downgrades played/read (8/13) to delivery (5);
1hg.A03 admits six statuses. The downstream 0Z4 dispatch separates message
classes and reaches 0Z5 timestamp storage. Complete 15A admission, clock and
namespace/cleanup policy remain unknown; no runtime or live operation changed.
[Evidence and reproducible recipe](docs/protocol/receipt-factory.md).

Receipt-factory validation: seven source topics / 12 files / 137,766 bytes
reproduce exactly from APK 2.26.37.71. Cached self-diff reports seven unchanged
topics and four unchanged metadata artifacts; this is reproducibility, not
cross-release equivalence. Repository hygiene checked 1,340 local links and
28 APK bundles / 8,059,427 bytes; diff hygiene passed. Research/docs only; no
fresh Rust runtime test or live account operation is claimed for this chunk.

2026-10-09 — **Confirmed static:** receipt timestamp branches now distinguish
direct zero defaults, strict aggregated participant parsing and adjusted-clock
defaults for successful-message participant receipts. Wire seconds become
milliseconds. The cache retains earliest positive time without rewriting the
nullable primary version; ordinary concrete device-receipt columns and conditional
device-preserving namespace mapping are traced. Clock calibration, complete
message admission and mutable identity cleanup remain unknown/unported.
[Evidence and comparison recipe](docs/protocol/receipt-clock.md); no runtime/account operation changed.

Receipt-clock validation: 18 source topics / 23 files / 82,330 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 18 unchanged
topics and four unchanged metadata artifacts; this does not establish parity
between different releases. Repository hygiene checked 1,350 local links and
29 APK bundles / 8,141,757 bytes. Diff hygiene passed. Changes are research/docs
only; no new runtime tests, live receipt or account changes are claimed.

2026-10-09 — **Confirmed local/static:** signed const/4 annotations now show
negative decimal values; 0Z8's trimToSize input is -1 (clear), not 255.
Primary selected-row invalidation and companion broad cache clear are traced,
including optional map/callback handling. Incoming SQL fallback selects timestamps
without primary versions; general load/eviction and complete mutable receipt
runtime remain unported. [Evidence and recipe](docs/protocol/receipt-cache.md).

Receipt-cache validation: all 66 xtask tests, strict xtask Clippy, rebuilt
research CLI, CLI-reference regeneration, OpenAPI/library-symbol check, formatting
and diff hygiene passed. Eight source topics / 13 files / 57,611 bytes reproduce
exactly with the corrected renderer; cached self-diff verifies eight unchanged
topics and four metadata artifacts. Hygiene checked 1,359 local links and
30 APK bundles / 8,199,368 bytes. No transport code changed; previous 798-test
runtime validation is not claimed as a fresh run. No live account operation ran.

2026-10-09 — **Confirmed static:** the general receipt loader retains device
row membership without a positive timestamp filter; it does not load primary
versions. Cache install rechecks rather than overwrites, and general row mapping
uses 0Yo.A03 rather than the incoming update's A07. Pending row membership,
delivery facts and immutable original targets remain distinct. Mutable runtime
receipt_user/status admission and alias authority are still unported.
[Evidence and next inputs](docs/protocol/receipt-load.md); no account or live operation changed.

Receipt-load validation: seven source topics / 12 files / 58,000 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified seven unchanged
topics and four metadata artifacts; this does not prove cross-release parity.
Repository hygiene checked 1,368 local links and 31 APK bundles / 8,257,368
bytes; diff hygiene passed. Research/docs only; no fresh Rust or live tests
are claimed beyond the preceding 66-test tooling chunk.

2026-10-09 — **Confirmed static:** per-user receipt timestamps retain earliest
positive delivery/read/played independently. User rows can be created by read
or played without a delivery timestamp; identity cleanup's existence query
is not equivalent to a Delivered fact check. Writer rejects other statuses,
while complete upstream 15A admission and namespace authority remain unknown.
[Evidence and reproducible recipe](docs/protocol/receipt-user.md); mutable Rust receipt runtime remains
unported, with no account or live operation in this chunk.

Receipt-user validation: eight source topics / 13 files / 128,135 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified eight unchanged
topics and four metadata artifacts, not cross-release equivalence. Hygiene
checked 1,381 local links and 32 APK bundles / 8,385,503 bytes; diff hygiene
passed. Research/docs only; no new runtime tests or live operation is claimed.

2026-10-09 — **Confirmed static:** 15A admission rejects nonpositive times
and splits message/user advancement from device timestamp handling. Native
nonadvancing comparison includes equal ranks; its 22-entry ordering was recovered
from the exact APK payload. Lower/equal events can still reach device storage.
State-only status translation, manager exclusions and conditional message
classification are traced with explicit remaining authority/lifecycle unknowns.
[Evidence and next port inputs](docs/protocol/receipt-admission.md); no runtime/live changes in this chunk.

Receipt-admission validation: 13 source topics / 18 files / 121,518 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 13 unchanged
topics and four metadata artifacts; raw 96-byte/22-entry status-order evidence
independently matches classes2.dex at its recorded offset. Repository hygiene
checked 1,393 local links and 33 APK bundles / 8,507,021 bytes; diff hygiene
passed. Research/docs only; no fresh runtime tests or live account operation
is claimed.

2026-10-09 — Original-device receipt rows (offline): new original reservations
now atomically seed pending rows with exact frozen targets and nullable versions.
Authenticated attributed supported root receipts retain earliest positive server
milliseconds, including earlier duplicate facts, without rewriting original sends.
`outbox-versions` exposes redacted rows; historical missing rows stay null.
Native cleanup, PN/LID row canonicalization, per-user admission and resend remain
unfinished. No fresh send/live capture is claimed.
[Evidence and limits](docs/protocol/device-receipt-rows.md).

Receipt-row validation: 802 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, formatting, default CLI, API symbols, CLI reference,
experimental CLI build and hygiene passed. Retained-state inspection was
unchanged, with zero network commands and no retained group preparation sample.
A repeated parallel run exposed the previously recorded immediate-drop store
ownership refusal; owner teardown now explicitly releases its own lock after
closing the database. A shared-descriptor regression still rejects active second
owners and permits reopening after drop. No external store unlock/retry was added.
[Mechanism and attribution limits](docs/design/verification.md#temporary-store-ownership-refusal-observed-2026-10-09).

2026-10-09 — **Confirmed static:** receipt-manager A03 is populated from the
revoke receipt-deletion path after successful transaction close. The native
user/device DELETEs select the original supplied message row with no timestamp
predicate; manager exclusions occur after the earlier device-only pool. The
message A0l deletion/copy contexts are traced without inventing its persistence
or other writers. Signed negative 22b/22s arithmetic now renders explicitly,
including the native transaction -1 nesting adjustment. Rust revoke, per-user
admission and identity cleanup remain unfinished. No account/live operation.
[Evidence and reproducible recipe](docs/protocol/receipt-manager.md).

Receipt-manager validation: all 67 xtask tests, strict xtask Clippy, rebuilt
CLI, CLI reference/API symbols, formatting and hygiene passed. Eight topics /
13 files / 301,312 bytes reproduce exactly; cached self-diff verifies eight
unchanged topics plus four metadata artifacts. Raw signed decrement bytes match
the recorded APK offset. Hygiene: 1,416 links, 34 bundles / 8,808,333 bytes.
Research/tooling only; no new shared-runtime or live account test is claimed.

2026-10-09 — Stored own-LID guard correction: both remote primary-change hooks
now recognize authenticated login identity before a group-preparation checkpoint
exists. One shared read-only lookup refuses malformed/conflicting saved authority
without repair or echo. New `group own-lid` prints only lid_known. APK BO0/BO1
proxy/direct bodies and stored PN/LID producers are retained; BO0 equality is
own stored user namespaces, not a remote directory alias. Native own-primary
versions, full own-device state and mutable identity cleanup remain unfinished.
[Evidence and limits](docs/protocol/own-identity.md).

Own-LID validation: all 807 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, default CLI, reference/API-symbol and formatting checks
passed. Exact bundle reproduction and cached self-diff passed. One 45-second
receive-only session authenticated and stopped cleanly, with sends/reconnects
disabled; peer/actions/receipts/outbox/events remained unchanged. Eight additional
inbound checkpoints remain unclassified; no live own-primary transition is
claimed. See the linked own-identity evidence for counts and limits.

2026-10-09 — **Confirmed static:** revoke receipt callbacks are now traced:
8SJ discriminator 6 removes the selected message-row user cache entry under a
monitor; DLn discriminator 6 delegates selected-row device-cache invalidation.
The user cache constructor uses plain 0D3; base entryRemoved is empty. Existing
transaction-close/rollback and optional device-cache callback evidence remains
separate. Mutable cleanup, revoke projection and resend are unported.
[Worker evidence and boundaries](docs/protocol/receipt-callbacks.md).

Callback validation: the five-topic / 10-file / 70,998-byte bundle reproduces
exactly; cached self-diff verifies five unchanged topics and four metadata
artifacts, not cross-release protocol equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting and diff hygiene passed. Repository
hygiene checked 1,436 local links and 36 bundles / 8,929,026 bytes. This final
chunk changes static research/docs only: it does not claim a new workspace
runtime or live cleanup test. The earlier own-LID runtime validation is separate.
No retained account was read or changed during this callback research.

2026-10-09 — Companion receipt lifecycle implementation (offline verified):
authenticated changed companion keys now clear only pending exact-device mutable
membership for ordinary direct/group original intents. A later positive supported
receipt reinserts with current nullable user version; original at_send/targets
and held send/receipt reservations are preserved. Primary/counterpart/status/
coexistence cleanup and final resend remain unfinished.
[APK evidence, diagnostics and boundary](docs/protocol/companion-receipt-cleanup.md).

Companion lifecycle validation: all 810 workspace/all-feature tests, strict
Clippy, warning-denying Rustdoc, default CLI, CLI reference/API symbols,
formatting and experimental CLI build passed. Hygiene checked 1,447 links and
36 bundles / 8,929,026 bytes. One receive-only session authenticated, then ended
early with unknown cause and no retry. Saved peer/actions/receipts/outbox/events
were unchanged; six new inbound checkpoints remain controls/unknowns rather
than user replies. No qualifying live companion transition occurred. listen now
preserves terminal state before shutdown and prints fixed redacted cause labels;
this diagnostic fix is locally tested, not yet live observed. Primary cleanup,
namespace migration and resend remain unfinished.

2026-10-09 — **Confirmed static:** bulk user receipt restoration uses a distinct
8G8.A05 writer from chat transfer/history sync. Any cache-slot improvement enables
SQL with every positive supplied timestamp; this is not the one-column live
writer or a universal SQL minimum policy. Descriptor field 40 and nested fields
1..6, actual caller branches and unique user-row index are retained. Exact table
xref negatives do not prove absence of generic normal-send seeding. Primary
cleanup still needs actual user-row existence/admission, not synthesized facts.
[Evidence and remaining port inputs](docs/protocol/user-receipt-import.md).

Validation: nine source topics / 14 files / 204,931 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies nine unchanged topics and four metadata
artifacts, not another release's behavior. Self-remap selects the correct original
class/method for seven topics; one is ambiguous and one unanchored, so no complete
proposed recipe is emitted. The committed compact report keeps actual candidate
selections and the full report digest; unrelated per-anchor hit lists stay local.
CLI-reference regeneration, API/library-symbol validation, formatting and diff
hygiene passed. Repository hygiene checked 1,463 local links and 37 bundles /
9,133,957 bytes. No Rust runtime changed, no fresh workspace/runtime test is
claimed, and no account or live operation ran. The prior cfda410 functional
chunk's 810-test validation remains separate.

2026-10-09 — **Confirmed static:** the four remaining exact receipt_user
reference roles are classified: backup_changes trigger construction, migration
error classification, whole-table deletion and receipt table metadata. The
trigger INSERT destinations are backup_changes, not receipt_user. Generic
consumers/normal-send user-row seeding remain unknown; primary cleanup still
requires actual user-row existence. No runtime or account behavior changed.
[Complete methods and reasoning](docs/protocol/user-receipt-reference-roles.md).

Validation: ten source topics / 15 files / 138,158 bytes reproduce exactly from
the pinned 37.71 APK; cached self-diff verifies ten unchanged topics and four
metadata artifacts. CLI-reference regeneration, API/library-symbol validation,
formatting and whitespace checks passed. No fresh workspace/runtime test or
live operation is claimed; the prior functional validation remains separate.
Repository hygiene checked 1,471 local links and 38 bundles / 9,272,115 bytes.

2026-10-09 — **Confirmed static:** explicit receipt table-descriptor readers
are incremental backup export/restore and JSON metadata serialization. Descriptor
fields now have actual serializer labels, including optional lookup_field.
Restore contains a dynamic table INSERT; its specific receipt input continuity
and complete validation remain untraced. Normal-send user-row seeding and
primary cleanup are still unfinished. Exact scans, scoped source, signatures,
catch tables and complete-dump digests are retained outside the checksum bundle.
[Consumer evidence and limits](docs/protocol/user-receipt-reference-roles.md#descriptor-consumers-follow-up).
No Rust/account/network behavior changed.

Consumer follow-up validation: CLI-reference regeneration, OpenAPI/library-symbol
checks, formatting and whitespace checks passed. Hygiene checked 1,475 local
links and the unchanged 38 checksum bundles / 9,272,115 bytes. The new excerpts
are separately attributed, not counted as checksum-bundle coverage. No fresh
workspace/runtime or live test is claimed.

2026-10-09 — **Confirmed static:** receipt ownership now has an exact caller
boundary: true message-key from-me inputs short circuit to ordinary admission
before the A0y helper/foreign-input property. False-key helper logic is
(type 64 + stored-own Bps.A00 + clear flag); later pool gates still apply.
Property 35516 has true/true baked defaults, not a known effective account value.
Ten explicit flag writers are retained with scoped source and full-dump digests;
constructors/receive/copy contexts do not prove universal origin semantics.
[Source, branch arithmetic and remaining prerequisites](docs/protocol/receipt-ownership.md).
No Rust user-row projection, primary cleanup, account or network change is claimed.

Ownership validation: eight topics / 13 files / 59,820 bytes reproduce exactly
from APK 37.71. Cached self-diff verifies eight unchanged topics and four metadata
artifacts, not cross-release equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting, whitespace and hygiene passed.
Hygiene checked 1,486 local links and 39 bundles / 9,331,935 bytes. Flag-writer
excerpts are separately attributed outside that bundle. No fresh workspace
runtime test or live account operation is claimed.

2026-10-09 — **Confirmed static:** native message status is A08, with direct
getter/reset and a downgrade-logging setter that still assigns. Two subtype
setters always select 6. Five direct writer methods and 111 physical superclass
setter calls are retained. The blank receipt message entry connects to device
pending insertion; failure writes local state 20, not a recognized server reason.
Conditional coexistence rows are separate from receipt_user membership.
[Exact methods, provenance and remaining initialization inputs](docs/protocol/message-status.md).
Ordinary outgoing initialization, full admission and primary cleanup remain
unfinished. No Rust runtime, account or network behavior changed.

Message-status validation: eleven topics / 16 files / 79,539 bytes reproduce
exactly from APK 37.71. Cached self-diff verifies eleven unchanged topics and
four metadata artifacts, not cross-release semantic parity. CLI-reference
regeneration, OpenAPI/library-symbol checks, formatting, whitespace and hygiene
passed. Hygiene checked 1,499 local links and 40 bundles / 9,411,474 bytes.
Caller/writer inventories and the no-matching ordinary override body result
remain separately attributed. No fresh workspace/runtime or live test is claimed.
