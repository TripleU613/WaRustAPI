# Normal group outbox — 2026-10-09

**Confirmed local implementation / live normal-group send unknown:** the
comparison public-client actor now prepares ordinary group text and already
uploaded ordinary image/voice-note intents from the durable normal outbox.
Previously `outgoing::begin_selected` classified any group as unsupported and
marked the queued intent Failed. The direct adapter now leaves groups to the
separate `outgoing_group` state machine. It never resets those historical Failed
entries or creates another logical message to repeat them.

## Evidence and port boundary

**Confirmed static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:

- **Confirmed vocabulary / inferred request shape:**
  `GetGroupInfoProtocolHelper.A00` names `interactive` and `group_response`;
  the `query request=interactive` shape retains the inference label in the
  existing [group-info table](groups.md). The existing `groups::info_request`
  is reused; this actor port does not establish a new complete native writer trace.
- `SyncDevicesJob.A0G -> 0xX` selects `MESSAGE_MULTI_PROTOCOL_QUERY`;
  `0xY` supplies context `message`. The retained writer `0y0.A00` emits
  `usync/query/devices version=2` and the selected user list. See
  [device evidence](cross-device-sync.md) and the source comments in devices.rs.
- `0y0.A00` at `1786..193a` constructs encrypt/key/user directory batches.
  [Durable batch evidence](device-key-batches.md) separates optional native
  fields/feature gates from the supported local query shape.
- `SendE2EMessageJob.A06` carries per-device pairwise sender-key distributions,
  `skmsg`, addressing mode (`12d0..1322`) and the supplied participant hash
  (`1300..1304`); the retained
  [method](apk/2.26.37.71-groups/group-send-job-hash-check.txt) and
  [writer](apk/2.26.37.71-groups/group-ack-writer-dispatch.txt) are mapped to
  outgoing_group.rs in the [recipe](apk/recipes/2.26.37.71-groups.json).
- The existing `1jL.A0N/A0O -> AzR.A03` device-hash derivation and bare field-2
  SKDM encoding are reused. [Planning](groups.md),
  [shared crypto](group-fanout.md) and [participant receipts](group-receipts.md)
  retain their evidence and limitations.

**Local policy, not a traced native scheduler/transaction contract:** each
preparation query has a 30-second connection-local deadline; device pages hold
at most 64 users, directory pages at most 32 devices and this adapter accepts at
most 4,096 recipient users. These are local bounds, not WhatsApp limits. Native
feature negotiation, full own-device inventory, participant-user hashes and
scheduler/job parity remain **unknown/unported**. Automatic hashing remains the
existing candidate V2; offline vectors do not prove every live inventory.

## Original intent through the actor

With an explicit comparison outbox opt-in, the actor selects the first eligible
intent in accept order across namespaces, or only its explicit local ticket.
`listen` without `--send-outbox` does not start this group preparation. Incoming
captures, ordinary receipts, notifications and heartbeats continue through the
same authenticated connection loop while a group query is awaited.

1. Validate supported content without network access. Claim the existing send
   path; a comparison/lab send ID cannot be reused to duplicate transmission.
   Existing foreign/legacy user-request key history blocks rather than being
   silently reused or overwritten.
2. Queue **and reserve** the original membership action together with
   `outbox-group:<local_id>` inside one write transaction, before socket write.
   A matched server result must contain exactly one complete group element
   naming the requested group. Short numeric group IDs are normalized through
   the existing parser. Missing, wrong-group, duplicate or truncated membership
   does not authorize a cached snapshot fallback. The shared queued-info
   projection also checks the request target, so recovery cannot later apply a
   wrong-group reply as that requested group's membership.
3. Resolve own PN or authenticated saved own LID according to supported saved
   membership. Missing own identity, mixed namespaces or absence from membership
   blocks explicitly. Freeze recipients and the group-row digest together in
   one read transaction; do not guess an own LID from another member.
4. Reserve each selected device page once, with message context, original local
   marker revisions and preparation metadata in one transaction. Each reply
   must be complete: refusals, per-user errors, omitted users, rejected signed
   candidates or superseded revisions block. Cached lists cannot turn that
   failure into permission to send. Pending membership changes stop later pages.
5. Rebuild the complete matching plan and load the shared routing/crypto
   snapshot. Request only devices without a held session, using the durable
   directory controller. Its internal selector derives from the original local
   intent ID; different idempotency namespaces cannot share another intent's
   frozen key plan. The actual private-JSON request ID is unchanged. All signed
   directory replies must pass coverage and signature checks before crypto.
6. Stage pairwise SKDMs and sender-key body encryption. Commit the original
   outbox entry, exact wire payload/ID, first AttemptRecord, next pairwise and
   own key state, exposure/rotation archive and DeliveryUnknown fact atomically
   **before** returning a stanza for socket write. Current intent and every
   captured routing/crypto row must still match. An OCC rejection rolls back
   staged crypto and intent changes, preserving the newer state.

The group crypto future previously held ThreadRng across a libsignal await,
which prevented integration with the spawned actor. It now seeds a cryptographic
StdRng synchronously from the OS-seeded thread RNG before awaiting. Existing real
Signal decrypt tests and the compiled/spawned actor cover this local integration
correction; no weaker randomness or new protocol value was introduced.

**Remaining implementation limitation:** any saved preparation marker excludes
automatic reselection, including interruptions between query settlement and
final reservation. No explicit answered-preparation resume is implemented yet.
This is our tooling's conservative behavior, not a server requirement. Each
underlying action/device/directory reply remains encrypted for inspection.
Directory page reservation and the parent stage update are separate commits;
a crash between them retains an unknown reserved page and never reissues it.
Refusals/timeouts save a fixed block reason while leaving the original queued
intent and facts intact; receiving can continue. Storage errors propagate instead
of being claimed as a saved protocol refusal. Existing background device sync
may re-read pending users in a later epoch; it cannot resume the original group
send. Group device reservations/settlement share the session cursor, so failures
are not immediately retried as background work in that same connection.

## Invoking and inspecting

Use the existing private JSON format and a stable original request ID:

```json
{"group":"GROUP_JID","request_id":"ORIGINAL_REQUEST_ID","text":"TEXT"}
```

Placeholders above are intentionally not real addresses/IDs. `outbox-send
--config PRIVATE_CONFIG --message PRIVATE_JSON --allow-live-request` selects
only the resulting normal intent. `image`/`voice_note` uses the existing upload
path; successful private uploads now bind `to` to the full group JID. Stable
sealed media references refuse another chat. Legacy empty-recipient group upload
rows are not silently migrated/rebound. The comparison prepare entry point
retains its original empty-to binding and can still reuse those saved successes;
only normal outbox prepare_outbox uses the explicit group binding. A cached-upload
regression proves both paths preserve original rows and refuse another chat
without a media route or network operation. Public account-bound ordinary upload
references retain their existing recipient-independent semantics.

`outbox-group-preparations` (or `--json`) inspects local selector, last phase,
fixed block reason, recipient/device counts, reserved query counts and the
scoped durable key-batch summary. It prints no chat, member, content, query ID,
key or raw response. Key-batch counts are read separately from phase metadata
and are diagnostic, not a consistent send-permission snapshot. `replay_allowed`
is always false. The single-ticket library counterpart is
`WhatsAppClient::group_preparation`; no HTTP route is introduced.

Group receipt output reports delivered/read/played **device counts**, preserving
actual participant addresses internally. One participant's delivery is not whole
group delivery. Server acceptance and recipient facts remain independent of the
original DeliveryUnknown reservation. Repeating an accepted private JSON returns
locally before reconnecting. A blocked/uncertain old JSON never changes its ID to
resend. The selected CLI's overall ACK wait remains 60 seconds; the library
receive duration can be configured independently through existing commands.

**Remaining first-port gaps:** group polls/settings,
view-once JSON, explicit fresh sessions or out-of-band SKDM distribution. Group
private-JSON replies (no `emoji`/`quoted_text`) now use the normal outbox
(2026-10-13, [quoted media sending](quoted-media.md#sending-2026-10-13));
JSON reactions and caller previews remain on the comparison path; normal typed
quotes/reactions are described below. Those unsupported JSON modes fail explicitly before store/upload access instead of silently losing
fields; the comparison path retains its existing capabilities. Group peer pin
replacement uses the explicit outgoing signed-bundle policy described below.
The default remains pinned and reports `peer_identity_changed`. Companion
reconciliation and own-inventory synchronization remain unfinished.
The library's ordinary-media staging test does not establish group view-once
semantics or handset rendering.

## Verification

**Confirmed offline:** a cold 130-recipient PN group traverses device pages
64/64/2 and directory pages 32/32/32/32/2, stores one exact original attempt and
reopens without replay. A LID group uses the authenticated saved own LID.
Recipient libsignal decrypt independently recovers SKDM and the exact bare text
body. Distinct namespaces using the same caller ID complete separate frozen key
plans for separate group intents, without reusing or replaying another plan. Media fixtures verify image/ptt envelope selection and chat-bound references;
they are synthetic uploaded protobufs, not handset display/playback. A late own
key change leaves newer crypto/outbox rows intact after atomic rejection.

**Confirmed injected real-loop tests:** cuts before write completion on membership,
devices, keys and final group ciphertext preserve original reservations. Restart
never sends the original group/query again; an independent background usync read
is separately identified. A late simulated server ACK and participant delivery
receipt settle the exact original wire without another attempt, while preserving
DeliveryUnknown. These are fake authenticated-node/socket-boundary tests, not
Noise/server/handset evidence. Missing/wrong/truncated membership, device/key
refusals, partial device coverage and explicit unknown deadlines preserve queued
intents without wire IDs, attempts or invented failure/delivery facts.

Fresh live normal-group server acceptance, companion-group coverage, participant
read/played receipts and handset rendering remain **unknown**. Receive-only
validation cannot establish an outgoing group message. Do not replay historical
comparison or normal-outbox message JSON to manufacture that result.

**Confirmed repository checks (2026-10-09):** 767 workspace/all-feature tests,
strict all-target/all-feature Clippy, warning-free Rustdoc, default CLI tests,
OpenAPI library-symbol validation, regenerated CLI reference and formatting
passed. The saved account's offline inspection retained peer keys, actions,
receipts, outbox, event summary and normalized inbox inspection; projection of
original authenticated captures added zero events. The new preparation inspector
returned zero saved group preparations. None of these offline checks sent a
request or changed an account identity.

**Confirmed reproducible evidence:** the 2.26.37.71 group bundle check reproduced
44 files, 929,065 bytes and 43 selected methods. This chunk changes Rust source
links in bundle metadata; it adds no native method body. A cached same-bundle
comparison took 0.033819 seconds with no differences. This timing is a local
self-comparison, not proof of cross-release equivalence or patch completeness.

**Observed live (2026-10-09):** the upgraded CLI authenticated with the existing
saved account, remained active for a bounded 45-second receive-only window and
stopped cleanly (exit 0). Chat sends and automatic reconnect were disabled. This
checks retained login/receive compatibility only; no normal group was sent and
no outgoing group delivery, receipt or handset behavior was established.

**Observed retained state after that window:** peer-key inspection, saved actions,
receipts, pending outbox, event summary and noncapture inbox inspection remained
unchanged. Capture rows increased from 446 to 451. Those five additional
transport captures are not counted as decrypted user replies or delivery facts;
no event was projected to manufacture either result.

Repository hygiene passed: tracked-file budgets, 1,142 local links, package
licenses and 5,815,284 bytes across 15 APK research checksum bundles.

## Signed group peer replacement (2026-10-09)

**Confirmed static (APK 2.26.37.71):** re-reading `LX/0jc.A0L` confirms the
changed-identity/held-session path before `LX/CQc.A00` at `0310..0324`.
`CQc.A00` checks the signed device key at `0030..0064` before session derivation
and identity saving. These are the existing shared peer-processing primitives;
this re-read did not establish a separate group-specific replacement exception.
`LX/D4x.A00` selects groups with `sent_sender_key = 1` (`0b1c`) after a primary
change. [Peer evidence](peer-identity.md#explicit-signed-outgoing-bundle-policy-2026-10-07)
and [exposure source](apk/2.26.37.71-core/identity-change-cleanup.txt) remain the
source anchors.

**Confirmed local implementation:** the normal group actor now passes
`transport.outgoing_identity_changes` to shared fanout preparation. `pinned`
remains the default. `apk_authenticated` permits a changed remote bundle only
after signed-key verification on an authenticated, original-query-correlated
directory response. Preparation preserves every old pin/ratchet and stages a
fresh pairwise session in memory. Account PN, saved login PN/LID, own-LID
checkpoint and the frozen own fanout identity stay pinned. Login and own-LID
rows join the final routing concurrency guard, so a later own-alias change
cannot authorize a stale replacement. `Snapshot::prepare` keeps its pinned
contract; `Snapshot::prepare_with_policy` exposes the explicit policy.

The final original-send transaction archives each replaced peer with
`authentication=signed_directory_bundle`, clears local verification through the
existing archive path, emits `IdentityChanged`, reserves device-refresh work and
marks other exposed groups for rotation. If this group's old generation was
exposed to a changed primary, preparation already minted an independent chain;
reservation archives the old generation and settles this group's exposure for
that new chain. The original exposure is checked before archive callbacks mark
it pending, then the predicted post-callback snapshot is checked again; a later
real rotation cannot be hidden by our own callback. It does not leave the just-minted generation spuriously pending
for another rotation. OCC checks precede these writes; caller failure rolls
archives, events, keys, refresh work and the original intent back together.

**Inferred guard / confirmed local behavior:** a changed primary cannot
reauthorize companions from a saved ADV list validated under its previous pin.
A fanout containing that primary's companions refuses with
`companion_authority_changed`, preserving the original intent and reply.
[ADV evidence](peer-identity.md) (`0jc.A0L`, `11T.A08`) establishes the dependence
on primary authority; automatic fresh reconciliation and continuation of this
original preparation remain unfinished. This guard is our implementation
boundary, not a claimed server refusal. An unrelated recipient's still-valid
companion can remain in the fanout.

**Confirmed offline:** independently generated PN/LID old/new Signal identities
prove new-recipient SKDM/group-body decryption, old-chain rejection, exact
old-pin/session archives, exposed-current-chain rotation and other-group pending
rotation. Default policy, saved-own aliases, invalid signatures, newer pins and
failed caller commits preserve the original state. Real actor tests exercise
the explicit policy with a pinned peer whose session is absent, maintain the
original local/caller IDs and one reserved attempt, and refuse replay after
reopening. Old-authority companion fanout is separately refused without mutation.
These tests do not establish a live group identity transition or handset result.

**Unknown / unfinished:** cached group sessions are reused without an unsolicited
refresh query. This change handles bundles actually needed by the existing
workflow; it does not invent evidence of a changed cached peer. Group preparation
resume, companion reconciliation after primary replacement, complete own-device
inventory and fresh live outgoing group validation remain outstanding. Previously
blocked group preparations retain their markers; changing policy never resets
those records or replays their queries.

**Confirmed validation for signed group policy:** 770 workspace/all-feature tests,
strict all-target/all-feature Clippy, warning-free Rustdoc, default CLI tests,
OpenAPI library-symbol validation, regenerated CLI reference and formatting
passed. Hygiene checked tracked budgets, 1,146 local links, package licenses and
5,815,284 bytes across 15 unchanged APK research checksum bundles. Offline saved
account comparison retained peer keys, actions, receipts, outbox, events and
normalized inbox inspection. The own-LID fixture's initial PN-address mistake
was rejected by the existing parser and corrected to a genuine LID fixture; no
validation guard was relaxed.

**Observed live for this chunk (2026-10-09):** the upgraded saved account
authenticated, ran a 45-second receive-only window and stopped cleanly (exit 0),
with chat sends and automatic reconnect disabled. Saved peer-key inspection,
actions, receipts, outbox, event summary and noncapture inbox inspection stayed
unchanged. Capture rows increased from 451 to 456; these transport captures are
not counted as user replies or delivery facts. Offline inspection found zero
saved normal-group preparations. No group was sent, no account key was rotated,
and no live group identity transition or handset behavior was established.

## Typed group quotes and reactions (2026-10-09)

**Confirmed static:** APK **2.26.37.71**, same hash as above,
`BWI.dynamicMethod`/`BZa.dynamicMethod` encode reaction key fields
remoteJid=1/fromMe=2/id=3/participant=4; BVK/BZF descriptors encode quoted
context stanzaId=1/participant=2/quotedMessage=3 and extended text context=17.
The existing [reaction bundle](apk/2.26.37.71-reaction-key/targets.json),
[reaction trace](reactions.md) and `quotes.rs` remain the attributed source.
The existing group stanza builder supplies reaction `decrypt-fail=hide` and
`meta content_type=add_on`, rather than treating a reaction as plain text.
No new native scheduling or outgoing normalization closure is established.

**Confirmed local implementation:** ordinary typed `Text{quoted}` and `Reaction`
now traverse the normal group's membership/device/key preparation and shared
fanout transaction. CLI `reply` and `react` accept a group chat and require
`--participant <original-PN-or-LID-JID>` for an incoming target. `--ours` derives
the participant from the actor's frozen own group identity; it conflicts with
`--participant`. Direct command behavior and all existing default request-ID
recipes stay unchanged. Reusing a request ID for a different chat/participant
remains an idempotency conflict; this change does not rename old queued intents.
The commands enqueue offline; explicit `listen --send-outbox` performs sends.
Inspect `outbox-pending` first because that flag selects all eligible intents.

Examples (fictional group/sender, operator supplies actual originals locally):

```sh
whatsapp-cli --profile <profile> reply --chat 120363000000000001@g.us \
  --id ORIGINAL-TARGET --participant 222222222222222@lid \
  --text 'reply' --request-id original-reply-intent
whatsapp-cli --profile <profile> react --chat 120363000000000001@g.us \
  --id ORIGINAL-TARGET --participant 222222222222222@lid \
  --emoji '👍' --request-id original-reaction-intent
```

`group_references::validate` and `payload` expose read-only attribution and
encoding. The target must name the exact group namespace. A known outgoing
wire ID must belong to the actual saved intent's group; ambiguous IDs refuse.
For retained incoming targets, the preserved authenticated route must match
chat, ID and original participant before copying a text preview. Unknown targets
retain the explicitly supplied participant and an empty preview. Only decoded
text becomes a bare quoted conversation; media, controls and message secrets
are not copied, opened or consumed. The current MessageRef/DeviceAddress model
cannot retain an agent component: CLI agent participants and retained routes
carrying one refuse explicitly, rather than silently folding them into a primary.
Full hosted/agent reference support remains unknown/unported. Group participant is its original user
identity, not an unrelated contact's PN/LID mapping. A changed local target
detected after query preparation reports `reference_changed` and leaves the
original queued intent without a wire attempt; the answered queries are kept.

**Historical negative, corrected locally 2026-10-09:** legacy retained group
rows omit participant from their selector. New records now bind participant;
quote/reaction lookup preserves historical attribution and recognizes each new
participant separately. See [group inbox](group-inbox.md) for atomic legacy
handling, read selection and native-storage unknowns.
Private MessageRequest `quote`/caller preview remains the comparison JSON path;
this chunk exposes typed normal quotes/reactions through `reply`/`react` without
silently dropping a supplied JSON preview. (2026-10-13: quoted image/voice note
replies use this guarded path, see [quoted media sending](quoted-media.md#sending-2026-10-13).)
Historical own-namespace normalization and live group rendering remain unfinished.

**Confirmed offline:** real PN/LID actor preparation decrypts SKDM and the group
body independently for incoming/own quoted text and reaction add/removal, checks
exact participant and target ID, reaction metadata, a single original attempt,
reopen/no replay and target-attribution changes before final preparation. Local
inspection tests reject foreign chat/participant bodies and do not quote a media
descriptor. These fixtures are not server or handset observations.

**Confirmed local fixture correction:** a synthetic LID/agent fixture did not
exercise the retained-route agent refusal: the existing binary encoder uses AD
domain byte 1 for LID, and the decoder restores that as LID with agent 0. The
regression instead uses a PN-domain agent byte 2, which retains its agent. This
is a codec/fixture distinction, not new proof of native hosted-agent semantics.

**Confirmed checks for typed references:** 773 workspace/all-feature tests,
strict all-target/all-feature Clippy, warning-free Rustdoc, default CLI tests,
OpenAPI library-symbol checks, regenerated CLI reference and formatting passed.
Hygiene checked tracked budgets, 1,148 local links, package licenses and unchanged
5,815,284 bytes across 15 APK checksum bundles. Offline upgraded-account
comparison retained peer keys, actions, receipts, outbox, events and normalized
inbox inspection; it sent no request or changed any saved identity.

**Observed live for typed-reference chunk (2026-10-09):** the upgraded account
authenticated, ran a 45-second receive-only window and stopped cleanly (exit 0).
Chat sends and automatic reconnect were disabled. Saved peer keys, actions,
receipts, outbox, events and noncapture inspection stayed unchanged; capture
rows increased from 456 to 461. These are transport captures, not new user-reply
or delivery facts. Offline inspection found zero normal-group preparations.
No live group quote/reaction, registration request, identity rotation or handset
rendering was performed or established.

## Group keep/unkeep and reference commit guard

The actor now accepts supported KeepInChat references, with original group
participant, sender-relative fromMe and the original accepted timestamp.
Quote/reaction/keep parent resolution now uses one read snapshot and revalidates
inside the crypto/outbox reservation transaction; changed references roll back
with reference_changed. No keep/reaction parent preview is copied or opened.
See [native evidence, invocation and limitations](group-keep-outbox.md).
Native keep permissions, expiry enforcement and fresh group acceptance remain
unverified; this is supported codec/dispatch, not complete native keep policy.

### Preparation expiry hardening (2026-11-04)

**Confirmed local / offline:** the actor now owns independent direct/group
preparation timers and refuses expired replies before advancing dependent sends.
Existing durations, durable uncertainty and no-retry policy are preserved.
The late group-device regression fails before the fix and passes afterward,
including durable reopen. Native distribution ACK ownership remains untraced.
[Evidence, regression and limits](network-correctness-audit.md#outgoing-preparation-owns-its-deadline-2026-11-04).

### Distribution ACK class admission (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 dispatches ordinary
message ACKs by exact class; Efa/Eoh field propagation and Eoh.equals confirm
class is part of callback identity. Rust's distribution waiter previously
accepted any ACK family with the same ID. It now requires class=message for
both success and error settlement. Missing/receipt/notification/call/case-mismatch
regressions fail before the fix and pass afterward, with valid reaction
distributions still completing the existing crypto and reopen/no-replay test.
[Retained methods, error459 qualification and unknowns](apk/2.26.43.72-distribution-ack-admission/README.md).
Native distribution callback registration, address normalization and complete
durable confirmation remain open; no raw sender-equality rule or retry was added.
No live operations or complete ban-prevention claim.

### Sender-key ACK marking, one-time reset and callback ownership (2026-11-04)

**Confirmed static, APK43.72:** SendE2EMessageJob.A06 registers FfK before
dispatch and waits for its matched callback; normal successful ACK processing
checks key existence and marks selected devices. The one-time-key branch instead
reaches0i9.A0X ->0j9.A01, which resets sent_sender_key=0, not1. Blanket
interpretations of every post-ACK helper as marking reusable key possession
are refuted. This is not proof that reactions use the one-time branch.
0Th.A02 also connects2Gc selector1's pre-ACK marker origin: a matched callback
invokes the wrapped result before scheduling marker retirement, so those effects
are not one atomic transaction.
[Methods, offsets, local persistence gap and next port boundary](apk/2.26.43.72-sender-key-ack-confirmation/README.md).
Native out-of-band distribution address/registration ownership remains open.
No production change, new replay authority or live operation.

### ACK callback normalization and inbound bridge (2026-11-04)

**Confirmed static, APK43.72:** Ewq/Eng preserve the job's Eoh callback key.
0Sf.A02 canonicalizes supported device-zero from/participant fields to user
JIDs and retains nonzero companions. This helper does not establish PN/LID
alias equivalence. The generic1lZ.A1D ->1lY.Big ->0ln.A09 bridge is now
connected; the selected inbound bridge does not call that outbound normalizer.
A0T is exact normalized-key failure removal, not success handling.
[Field propagation, type predicates, callers and port limits](apk/2.26.43.72-ack-key-normalization/README.md).
Distribution producer inputs and parser/type mapping still need tracing before
changing expected-address admission. No production or live behavior changed.

### ACK API result variants and distribution-shape qualification (2026-11-04)

**Confirmed static, APK43.72:**0Sf.A0B/A0C register through0Th.A01/2Gc0,
which discards the returned node; A0D and ordinary SendE2EMessageJob use
A02/2Gc1 and retain it. Null get() results therefore do not universally mean
a missing ACK. Failure forwarding does not retire the held pre-ACK marker.
SendLiveLocationKeyJob builds notification/type=location with participants,
not ordinary direct messages. Neither that job nor includeSenderKeysInMessage
proves the native ordinary reaction distribution shape or callback address tuple.
[Methods, caller inventories and remaining gaps](apk/2.26.43.72-ack-api-variants/README.md).
No production behavior, speculative address restriction or live operation changed.

### Auth-challenge ACKs preserve uncertainty (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 diverts numeric459
to onAuthChallengeReceived before ordinary ACK class processing. The traced
manager transfers an existing request between maps; it is not an ordinary
terminal message failure or proof of a ban. Rust direct/group ACK settlement
now excludes459, preserving original reservations and uncertainty. Java decimal
spellings are covered; no challenge bypass, retry or account-wide policy added.
[Methods, before/after regression and unported boundaries](apk/2.26.43.72-ack-auth-challenge/README.md).
No live operations. Full challenge-controller parity remains incomplete.

### Comparison sends share message ACK classification (2026-11-04)

**Confirmed local defects / fixed offline:** CLI comparison direct and final
group sends stored every nonaccepted ACK as server_refused, including numeric459.
The out-of-band group distribution loop accepted any ACK family without an
error and did not preserve the matched ACK itself with its other captures.

All three now use messaging::message_ack_outcome: ordinary message-class ACKs
classify as accepted/refused; numeric459 is AuthChallenge before class dispatch;
other ACK families are Unrelated. Challenge/unrelated map to outcome_unknown
and return an explicit no-retry error after saving the response. Distribution
ACKs are preserved through the encrypted inbound capture path, and unresolved
ones stop the loop before any later distribution/group write. They do not
confirm sender-key possession. Original intents remain nonreplayable.

**Confirmed static support, APK43.72:**1rH.A03's class dispatch and special459
diversion, Integer.parseInt grammar and callback-map distinction are retained
in [auth-challenge evidence](apk/2.26.43.72-ack-auth-challenge/README.md) and
[ACK admission evidence](apk/2.26.43.72-distribution-ack-admission/README.md).
The shared helper classifies a node only; ID/address binding remains the caller's
duty. Existing wait_for matches tag/ID. Full native expected-address tuples,
participant/type/edit rules and challenge recovery remain incomplete.

The bounded CLI exits with uncertainty rather than waiting for a later matching
family; this is local no-replay policy, not a port of the native callback
scheduler. Existing ordinary refusal statuses remain unchanged. Historical
saved server_refused rows are not rewritten. This chunk adds no retry, reset,
new request shape or live operation.

Validation: protocol table covers missing/wrong classes, numeric459 decimal
variants including full-width digits, ordinary500, accepted ACKs and non-ACK
nodes. Existing durable actor reservation regressions still exercise the same
shared challenge parser. Full socket-level CLI sequence coverage is not claimed.

### Comparison IQ waiter reply admission (2026-11-04)

**Confirmed local defect / fixed offline:** messaging::wait_for matched only
node tag and ID. A same-ID IQ get/set, absent type, empty type or differently
cased Result could return as a request response, ending the wait and reaching
a caller's success/refusal or parsing path. The shared comparison waiter now
requires exact result/error for IQs. Other stanza families keep tag/ID matching;
message ACK classification remains separate.

**Confirmed static, APK43.72**, SHA-256887a6f94…0c2e:
0ln.A01 reads type and invokes CK4 only for result and Bvj only for error;
unexpected types reach unexpected_recv_iq_type diagnostics/assertion.
[Retained REA method](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json),
Evidence ev_445bcabed23b1091f4b30e63e2bd073f980299ad5b82df8787e84b425d8bf9c2.
This supports dispatch grammar, not identical native pending-map lifetime or
assertion/error policy. Rust retains unmatched IQs in its existing bounded
pending buffer and continues awaiting a valid reply. No new resend.

The existing30-second whole-operation timeout and32-node count bound remain
local policy. Regression comparison_iq_wait_only_admits_native_result_error_replies
fails with the old extracted predicate and passes after the guard; it covers
request/absent/empty/case-mismatched types, both response types, wrong IDs/tags
and unchanged ACK tag/ID admission. This is a predicate regression, not a new
encrypted-socket integration proof or complete native callback-map parity.

**Remaining capture gap:** wait_for returns buffered unmatched nodes only on
success. Timeout, terminal stanza, read error and count exhaustion currently
discard that local buffer; the helper has no store parameter. That loss is not
fixed by this admission guard and must be addressed separately without turning
a failed wait into permission to resend reserved requests. Generic ACK sender
normalization and challenge-controller integration also remain incomplete.
No live traffic or account operation.

### Comparison wait captures survive failed exits (2026-11-04)

**Confirmed local defect / fixed for repository CLI callers:** wait_for kept
unmatched decoded nodes only in its pending Vec, returned only on success.
Timeout, transport error, terminal stanza and count exhaustion discarded that
buffer before callers could persist it. This impaired diagnosis of reserved
requests; it was not evidence those requests had never reached the server.

New messaging::wait_for_captured uses the same bounded read/match loop with an
observation callback that commits each decoded node through inbox::preserve
before matching, checking a terminal tag or advancing the count. It captures
the matched reply too. Capture failure propagates and stops the operation; it
does not authorize settlement or another write. Every current repository CLI
comparison IQ/ACK wait now supplies its existing encrypted store, including
media routes, direct/group key queries, distributions and group retry refetch.
Older successful-path captures remain for compatibility and content-hash writes
remain idempotent; these are diagnostics, not ACKs or processing confirmations.

The existing public wait_for signature is retained for callers without a store;
it still has no durable capture guarantee. Both wrappers share the same IQ
result/error admission,30-second whole-operation deadline and32-node bound.
An explicit post-observation monotonic check prevents capture latency from
allowing reply settlement after the deadline. Blocking storage can delay
observing expiry; no hard real-time responsiveness guarantee is claimed.

**Evidence scope:** this is local diagnostic/persistence hardening, not a claim
that the APK uses our inbox table or content-hash keys. The pinned43.72
[IQ callback grammar](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json)
remains the admission reference; native callback-map/challenge parity and
per-request durable recovery are separate work.

Regression decoded_wait_captures_survive_each_exit_and_reopen_without_settlement
uses the actual shared wait loop and encrypted ProtocolStore. Synthetic readers
exercise valid response, timeout, read error, terminal stanza, count exhaustion
and injected capture failure. After reopen, every committed decoded-node
capture exists and a saved uncertain intent remains unchanged.
Capture failure prevents reading the queued matching response. The test is
not an encrypted-socket/Noise interoperability proof.

**Limits:** undecodable or partially read frames do not produce a Node and are
not captured by this helper. Nodes not yet read at timeout cannot be preserved.
No existing historical loss is reconstructed, uncertain send is replayed,
identity is changed, or server wait is overridden. Raw-node capture alone does
not perform protocol ACK/business processing for unrelated notifications.
No live account or network operations were used.

### Comparison request deadlines start before write (2026-11-04)

**Confirmed local timing gap / fixed offline:** comparison IQ/message callers
performed write_node and only then started wait_for_captured's30-second window.
A slow request write could therefore extend request ownership by a fresh30s.
Activation was already fixed separately; the remaining9 repository comparison
write/wait pairs now use messaging::write_and_wait_captured.

The helper creates the existing30-second monotonic deadline before exactly one
write and passes that same deadline into the capture-aware wait. An expired
deadline immediately returns uncertainty without reading a queued late response.
The existing post-capture expiry guard also remains. Request envelopes, IDs,
query batching, encryption and saved send reservations are unchanged. No send
is reserved or retried by the helper itself; the caller retains that duty.
Membership/read-only and retry-refetch request durability is not newly claimed.

Covered pairs: group membership, device batches, directory batches, direct
distributions, final group send, group retry key refetch, direct message send,
media-route query and direct-message key query. Other unpaired writes and
receive loops retain their existing policies. Read-only wait_for and
wait_for_captured APIs still start a window when called; request senders should
use the new combined helper to avoid splitting deadline ownership.

**Confirmed static architectural support, APK43.72:**0Sf.A0A calls0ln.A08
registration before A05 dispatch; A08 schedules its positive-duration callback
timer by ID. [Retained send order](rea/2.26.43.72-authkey-outcomes/send-iq-internal.json)
and [registration](rea/2.26.43.72-authkey-outcomes/pending-registration.json).
The separate [ACK APIs](apk/2.26.43.72-ack-api-variants/README.md) also register
with0Th before dispatch; their full timer policy remains distinct.
This supports registration-time ownership, not a claim that30s is the native
duration for every covered request or that all native scheduling is ported.

Regression expired_request_deadline_is_not_restarted_when_reply_wait_begins
advances the paused clock past a preexisting deadline before entering the actual
shared wait loop, then verifies Deadline and zero reader calls despite a queued
matching result. This models slow-write latency; it is not a new encrypted
socket integration test. Existing decoded/malformed capture and uncertainty
reopen regressions exercise the same deadline-aware core.

**Limits:** a pending write is still governed by its existing transport I/O
policy and can delay observation of the response deadline; it is not silently
cancelled by this helper. A failed write returns its existing error. Unread
late records are not captured, and expiry never proves the server did not
receive the write. Original uncertain intents remain nonreplayable. No native
request-address/challenge-controller parity, new route or live operation.

### Ordinary message ACK forwarding (2026-11-05)

**Confirmed static, APK43.72:** selected ordinary message success and refusal
event paths both jump back to generic A1D with the original node/Eoh key.
This closes those forwarding edges into the previously traced concrete callback
bridge. Error459 and malformed/early paths remain separately qualified. The
destination rewrite helper inspected here is FBID-migration-specific; Efa's
null type default does not prove the out-of-band producer leaves type unset.
[Exact offsets, retained exports and remaining producer gap](apk/2.26.43.72-message-ack-forwarding/README.md).
No runtime guard, replay policy or live account operation changed.

### Sender-key requirement/session boundary (2026-11-05)

**Confirmed static, APK43.72:** the sender-key requirement inherits conditional
recipient selection and pairwise session readiness. A selected hash mismatch
retains the original candidate set instead of applying later sent-key filtering.
The latched A07/A08 booleans come from session/PQ policy, not distribution ACKs.
The exact native ordinary direct-SKDM producer is still unlocated; the status
requester and include flag do not establish it.
[Methods, offsets, search scope and unknowns](apk/2.26.43.72-sender-key-session-requirement/README.md).
No Rust runtime, replay or live account behavior changed.

### Membership guard across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** after reserving a reaction's
pairwise distributions and group ciphertext, a saved membership change while
waiting for a distribution ACK did not stop the next distribution or final
group stanza. Query/key preparation had a frozen-membership digest guard, but
the ACK-driven continuation omitted it.

Every continuation now compares the saved group snapshot digest before returning
another stanza, including the initial distribution return and final group return.
A mismatch persists the existing `membership_changed` preparation block and
returns no dependent ciphertext. Already exposed keys, pairwise sessions,
original encrypted attempt, uncertainty and reservations stay held; no replay,
rollback, fresh IDs, automatic membership query or rotation is authorized.

The regression `membership_change_during_distribution_cannot_release_more_ciphertext`
prepares two real pairwise distributions, applies the real remove-notification
snapshot handler while either the first or second ACK is pending, then supplies
the matching successful ACK. Old code failed by releasing stale ciphertext.
The corrected path stops before the remaining distribution or final group send,
preserves all crypto/outbox state and all non-preparation checkpoints, and stays
nonselectable after reopen. Existing unchanged-membership crypto/decrypt tests
continue to cover successful release.

**APK corroboration, not scheduler parity:** APK43.72's session requirement
A01 01de..0206 distinguishes a changed participant hash from equal-hash
sent-key filtering, as retained in
[the requirement bundle](apk/2.26.43.72-sender-key-session-requirement/README.md).
This fix extends the existing local frozen-snapshot policy; native per-ACK
revalidation or identical blocking semantics have not been proven. The check
is a saved-state observation before returning a node, not an atomic transaction
with a remote socket write; the entire snapshot digest also conservatively
includes non-membership group fields under the existing policy. Native direct
SKDM producer/address ownership remains unresolved. No live account operation.

### Authority guard across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** device-list, own-login identity
and peer pin rows could change after crypto reservation while a distribution
ACK was pending. The reservation transaction validated the original inputs,
but later continuations used held ciphertext without revisiting that authority.

The reservation transaction now also captures its resulting authority rows:
the original routing keys (group snapshot, own login/LID, recipient device
lists and primary pins) plus the encrypted targets' exact device pins. Taking
this snapshot after reservation avoids rejecting the same transaction's own
accepted signed pin replacement. Before returning another distribution or the
final group stanza, the connection-local guard compares those saved bytes in
a read transaction. A change records fixed `routing_changed` and emits no
further ciphertext. The existing membership guard retains its distinct reason.
Ratchets are excluded: receive-side session progression is not alone proof
that already reserved ciphertext has lost identity authority.

The red/green regression
`authority_change_during_distribution_cannot_release_more_ciphertext` covers
changed device-list authority metadata, a different valid peer identity pin,
and own-login authority, each while the first or second of two distributions
awaits its ACK. Old code released stale ciphertext. The corrected path retains
all post-change crypto/outbox rows and non-preparation checkpoints, blocks
reselection, and preserves exactly that state after reopen. The prior real
remove-notification regression shares setup and remains separate. Existing
unchanged-authority tests still cover successful decrypt and continuation.

**Evidence / limits:** this extends the local routing OCC policy already grounded
in APK37.71 group device hash and primary identity invalidation evidence in
[group fanout](group-fanout.md). It does not claim a native per-ACK scheduler
contract. Exact row comparison conservatively includes metadata-only changes.
It is not atomic with socket write, does not freeze every store row or pending
work marker, and does not establish native direct-SKDM ACK ownership. The guard
is connection-local; interruption continues to preserve existing non-replayable
reservations. No automatic refresh, key reset, resend or live operation.

### Sender-key validity across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** the post-reservation routing
guard did not include sender-key chain authority. Calling the existing
primary-change invalidation helper could mark the exposed key rotation_pending
without changing any routing/pin bytes, and a later distribution ACK still
released old-chain ciphertext. The regression reproduced this release.

The original reservation transaction now captures the distribution state's
own identity and chain ID. Before returning dependent ciphertext the same read
transaction that checks routing also requires the state to exist, retain those
values and have rotation_pending=false. Invalidation, replacement or removal
blocks with the existing routing_changed diagnostic. It neither rolls back
exposure/ratchets nor deletes the original ciphertext or pending ACK history.
This is connection-local authority checking; it does not authorize restarting
or automatically rotating/resending the held intent.

`sender_key_invalidation_during_distribution_cannot_release_old_ciphertext`
uses the real invalidation helper while either the first or second distribution
ACK is outstanding. Its second case also stages and reserves a real independent
fanout/new chain before delivering the old ACK. Both cases stop further output,
preserve post-change crypto and non-preparation checkpoints, and stay blocked
after reopen. `same_chain_ack_metadata_does_not_block_distribution_continuation`
records and confirms a separate same-chain pending ACK: changing delivered-device
bookkeeping alone still allows the next distribution/final group stanza.
The guard deliberately compares chain authority rather than the whole mutable
exposure record or sender-key ratchet bytes.

**Evidence / limits:** APK37.71 D4x.A00 primary-change exposure invalidation and
the existing generation-aware confirm_ack_in contract are retained in
[group fanout](group-fanout.md) and sender_key_distribution.rs. This extends the
local frozen-state policy; native per-ACK revalidation/scheduler equivalence is
not proven. The guard does not make validation atomic with socket write or
resolve the native direct-SKDM producer/address tuple. Corrupt store data remains
an error. No live account operation or recovery reset.

### Device-refresh revisions across preparation and ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** a parsed contact device-change
notification queues synchronization before its new list exists. The old saved
list stays byte-identical, so checking that list/pins alone allowed a dependent
distribution or final group stanza after a newer notification. The regression
failed on this stale release before the revision guard.

Fanout snapshots now include each recipient's existing durable device-sync
revision. This monotonic local work counter is already raised by request_sync_in;
it is not a server device hash, ADV timestamp or delivery version. The original
reservation transaction validates it alongside routing, then captures its
post-reservation value for the connection-local continuation guard. A later
affected-recipient refresh blocks with routing_changed; a revision raised during
crypto staging rejects that reservation through the existing OCC error path.
No counter reset, pending-work deletion or automatic replay is introduced.

`device_refresh_during_distribution_blocks_only_affected_recipients` parses a
real-shaped devices/add notification, invokes the same durable request_sync sink
as the actor, and verifies the saved list has not changed. While either of two
distributions waits for its ACK, a recipient's refresh stops dependent output;
an unrelated contact's refresh still permits continuation. Both retain all
post-change rows except the explicit preparation block, and remain nonselectable
after reopening. The shared fresh/cached reservation race test also includes a
refresh raised after staging and proves rollback/preservation.

**Evidence / limits:** APK37.71 11Q.A09 ->101.A02 records pending device-sync
work before SyncDevicesJob obtains replacement lists, as retained in devices.rs
and [cross-device synchronization](cross-device-sync.md). Extending the local
monotonic revision guard is a concurrency policy, not claimed native per-ACK
scheduler parity. Any new local refresh reason advances this counter, so the
guard conservatively stops even without proof the remote inventory changed.
It does not reject every pre-existing pending marker, implement own-account
companion synchronization, or make state validation atomic with socket write.
Already exposed ciphertext, keys and uncertainty stay reserved; no live request.

### Cancellation while a preparation query is pending (2026-11-05)

**Confirmed local defect / corrected offline:** cancellation is allowed before
the original outbox intent becomes InFlight. If it happened while a group
membership query was pending, the reply entered normal continuation and failed
the next device-query reservation's unchanged-intent check, ending the actor.
A cancellation before a device reply could also progress to a directory query
before the later crypto transaction noticed the changed intent.

For an already-cancelled intent, a matching current preparation reply now goes
through its existing owned handler (membership action, device cursor or key batch)
and then persists the fixed intent_cancelled preparation block. No dependent
query, key adoption, ciphertext reservation or send is returned. Original
cancellation/events, completed query history and any pre-existing uncertainty
remain held; cancellation does not erase a query already transmitted. Handler
and store errors still propagate instead of being labelled successful cleanup.

The actor regression
`cancelled_group_preparation_settles_owned_reply_without_followup_or_disconnect`
cancels during each of membership, device and key waits, delivers the matching
result and services an inbound ping as a receive barrier. Old code failed with
the link closed at the membership stage. Corrected code returns the pong, emits
no follow-up query/ciphertext, retains a zero-attempt cancelled intent, records
the owned reply as settled and adopts no Signal sessions/pins/sender keys.
The preparation remains blocked and nonselectable after reopen.

**Boundary:** Outbox::cancel already rejects InFlight/Failed intents. Thus
ordinary cancellation during the later reserved distribution ACK waits was a
negative finding, not a newly supported cancellation capability. This change
observes cancellation before reply continuation; it does not make every external
concurrent mutation atomic with socket write, port native job cancellation,
authorize revocation, delete uncertain history or replay cancelled work. APK43.72
result/error callback admission remains the existing prerequisite. No live
account operation or new APK-runtime observation.

### Unique group-info projection across recovery (2026-11-05)

**Confirmed local defect / corrected offline:** normal group preparation
required one group child, but shared queued projection/recovery selected the
first group child from the same reply. A rejected ambiguous result could later
replace the saved snapshot when recover_pending processed its retained action.
The offline regression reproduced recovery reporting one applied projection
for such an ambiguous reply; the direct record_info_reply entry also accepted it.

A shared unique-group selector now rejects multiple outer group elements before
projection. Queued group-info additionally requires a group to be present.
Those fixed group-field wire refusals remain unapplied through the existing
transaction/catch boundary: saved action reply, pending marker, prior snapshot
and journal are preserved. Repeated recovery does not choose the first group
or transmit another query. A valid single correlated group still projects once.
The direct recorder preserves its no-group=false contract and rejects multiple
groups before any write. Other group action families are not given this new
outer cardinality rule.

`ambiguous_group_info_cannot_replace_membership_on_recovery` settles and reopens
queued replies containing duplicate expected groups, expected plus foreign
group, no group, or a valid single group. Invalid recovery preserves all
checkpoints/journal, snapshot and retained reply; repeated recovery applies
nothing and reserve_pending remains empty. The single-group control updates
the snapshot once. Duplicate direct projection is also refused.

**Evidence / limits:** this aligns shared projection with the existing local
normal-group exact-one correlation guard. APK37.71 ADs parses selected group
nodes; that alone does not prove a native outer-IQ cardinality rule. Existing
participant null-skip/duplicate-map semantics are deliberately retained: an
initial review found these are supported by ADs.A05 evidence in parse_group,
so no speculative stricter member parser was added. Native exception/recovery
parity, every group action and a guarantee against all stale snapshots remain
unproven. No query replay, live account operation or cache fallback.
