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

**Not supplied by this first actor port:** group quotes/reactions/polls/settings,
view-once JSON, explicit fresh sessions or out-of-band SKDM distribution. Those
JSON modes fail explicitly before store/upload access instead of silently losing
fields; the comparison path retains its existing capabilities. Group peer pin
replacement remains pinned even with the separate direct outgoing APK-compatible
replacement setting. A changed group peer reports `peer_identity_changed`;
compatible replacement and own-inventory synchronization remain unfinished.
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
