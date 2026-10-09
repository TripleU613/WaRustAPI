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
private-JSON quote remains on the comparison path; normal typed quotes/reactions
are described below. Those unsupported JSON modes fail explicitly before store/upload access instead of silently losing
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
silently dropping a supplied JSON preview. Full media quotes, historical
own-namespace normalization and live group rendering remain unfinished.

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
