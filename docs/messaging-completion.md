# Messaging completion checklist

Requested scope: implement the following ten improvements. A checked item needs
implementation and relevant tests; live verification is recorded separately.
Comparison APK evidence is 2.26.37.70, not the pinned 2.26.36.75 profile.

- [x] Opened-receipt queue after successful view-once viewing; live ACK pending.
- [x] Continuous incoming-message processing with bounded keepalives.
- [x] Immediate authenticated decryption and durable-before-ACK delivery confirmation.
- [x] Outgoing delivery/read/played tracking, including normal outbox events.
- [x] Ordinary image download/display without plaintext files.
- [x] Incoming voice-note retrieval/playback; live incoming sample pending.
- [x] Incoming prekey messages from new senders; live new-sender sample pending.
- [ ] Notification handling and cross-device synchronization ([evidence and gaps](protocol/cross-device-sync.md));
  `encrypt` and `devices` notifications are now handled and contact device lists
  are verified and stored; own-account sync, app-state snapshots and fan-out remain.
- [x] Crash/reconnect coverage for receiving, consumption and uncertain sends,
  including socket cut points on the real session loop; disk-full remains uncovered.
- [x] Integration with the normal account runtime/outbox (one live combined
  session sent a queued text with delivery and read receipts; handset display
  unconfirmed).

## Verification and operator handoff

2026-10-06 receive-path audit: `listen` lacked the retry request that lab
receive sent for undecryptable messages. Both now share the durable once-only
retry queue, inspectable with `receipts`; restart and write-cut tests cover it.
See [runtime parity correction](protocol/messaging-transport.md#runtime-parity-correction-2026-10-06).
Fresh live receive/auto-read validation remains pending. The saved private
`lab/` state was recovered from the former checkout in the desktop Trash;
profile `l` opens offline, registered, with no eligible sends or queued actions.
The original and all identities/send IDs were preserved. A 90-second receive-only
session then logged in and activated successfully; a direct pkmsg failed, its
once-only retry was server-acknowledged, and redelivery still failed. No new
read receipt was queued. This verifies the retry path, not fresh-message auto-read.
Offline pin inspection shows that both PN and LID aliases mismatch the incoming
prekey identities; the cause is unknown and verified identity-change handling
was absent at that live checkpoint. The pins remain intact. Direct/group prekey processing now preserves
first-seen conflict fingerprints across restart/redelivery, inspectable through
`identity-conflicts` (offline). These records establish an observed difference,
not a legitimate replacement. A subsequent explicit `apk_authenticated` policy
adds authenticated remote replacement, archived prior state and atomic journal
events. Isolated replay recovered one direct message from the original captures,
with the original account unchanged. Correcting group pairwise selection to
prefer the held LID session recovered the remaining group capture; the saved profile now selects the APK-compatible policy in a private corrected
configuration. Canonical offline replay recovered both pending captures; previous
peer keys/sessions and the original configuration were preserved. See [peer identity evidence](protocol/peer-identity.md).

A subsequent 45-second receive-only session delivery-confirmed both recovered
messages as duplicates. No fresh message arrived; the duplicate auto-read gap
was corrected and covered by a real-loop write-cut regression. A bounded
explicit saved-message read test obtained matching server ACKs for both recovered
text reads (eight reads plus the earlier retry acknowledged in total). Handset
blue ticks remain unconfirmed; do not call this fresh-message auto-read validation.

Work remains on broad notification/cross-device synchronization, socket-level
cut-point crash coverage and full outbox integration. Preserve the existing encrypted account, identities, send IDs
and uncertain outcomes. No unsolicited chats, identity replacement, registration
retries, or reopening of consumed media are part of this work.

Any scenario that needs a new incoming test message, another linked device, or
handset confirmation belongs here; it must not be described as live verified
from offline tests alone. Existing received captures can be processed locally.

### Implemented and verified locally

- History-notice diagnostics (2026-10-07): current APK 2.26.37.71
  `BZc/BV6.dynamicMethod` identifies Message field 102 and its nested schema.
  `lab-inbox --inspect` labels it without applying history or printing values.
  The same inspection path now redacts poll question/option strings. Static
  evidence, literal-fixture tests and saved-account offline inspection of the
  49-byte notice (nested fields `[1, 2]`) establish diagnostic behavior; history
  rendering, runtime feature selection and synchronization remain unfinished.
  `DA8.CF4` group/app-data checks and positive metadata requirements are now
  traced; the descriptor reader now supports mixed repeated/scalar fields.
  Supported metadata now has a pure decoder and redacted diagnostics, with
  local duplicate/unknown-field and receiver bounds. Saved-account inspection
  validated one receiver and context presence, without reporting their values.
  Ordinary authenticated group notices with meta/appdata=group_history now
  project to the distinct group_history_notice control event. Historical group
  and broadcast backfill now checks their original chat route; rollback/restart
  tests cover once-only projection. Private-copy and canonical backfill each
  upgraded exactly one group_history_notice; repetition upgraded zero and
  changed no other event kind. No history transfer is established.
- Polls: creation (`pollCreationMessageV3`, field 64, with `messageContextInfo`
  secret) and votes (`pollUpdateMessage`, field 50, AES-256-GCM over an
  HKDF-derived key; the APK performs the derivation natively, so the recipe is
  the documented one with the APK's inputs) are decoded, projected and sendable
  through the normal outbox with `poll`/`vote` CLI commands. Our own poll secrets
  are persisted so incoming votes decrypt; undecryptable votes stay opaque. The
  login reply's `pn`/`lid` are saved for message keys and vote identities. Live:
  the contact's poll decoded, our vote was server-accepted and our own poll was
  delivered; incoming votes on both polls decrypted live, confirming the recipe.
- Groups (2026-09-20, `docs/protocol/groups.md`): management requests
  (create, info, subject, description, add/remove/promote/demote, leave,
  invite link) on the action queue with snapshot ingestion and group events;
  not live-verified — the first create was followed by a stream error and the
  account was then refused at login (`STATUS.md`).
- Group message **receive** (2026-09-22, `docs/protocol/groups.md`): sender-key
  decryption is implemented. An SKDM (`Message` field 2) arriving in a member's
  direct `pkmsg`/`msg` — whether standalone or as a second `enc` child of the
  group stanza itself — installs the sender key; the stanza's `skmsg` child then
  decrypts with it, strips the APK plaintext padding and projects as a
  `NewMessage` keyed by `ChatId::Group` with the `participant` as sender. The
  member's direct session, pinned identity, any consumed one-time prekey, the
  sender key and the inbox row commit in one transaction. Undecryptable stanzas
  degrade to Unsupported without failing the session, and a decrypted one is
  confirmed with `<receipt to=<group> id=<id> participant=<sender>/>` (attribute
  set and device-addressed participant form both confirmed from `LX/Cp2->A00`/`A06`). **Live-verified 2026-09-22**: a
  handset-created group with the lab number as a member delivered two stanzas
  and both decrypted, installing the sender key and projecting as group
  `NewMessage` events. The live stanza exposed two defects an offline round-trip
  test could not: the distribution is at field 2 (not 18), and one stanza really
  does carry `msg` + `skmsg` together. Both messages were recovered from their
  preserved captures with `lab-inbox --replay-captures`, since the server does
  not redeliver a queued stanza. Group message **sending** (per-device SKDM
  fan-out) is not implemented and is blocked on unknown evidence: a participant
  hash is required on every group send and its algorithm is untraced.
- Chat actions (2026-09-20, `docs/protocol/chat-actions.md`): a durable
  one-shot IQ action queue carries block/unblock/block-list fetch, spam report,
  the default disappearing-timer IQ and `usync` contact checks; E2E protocol
  messages carry the per-chat disappearing timer, keep-in-chat and limit
  sharing. Live: block, unblock (after dropping `pn_jid` from unblock items),
  contact check and all three setting messages were server-accepted; the
  settings were delivered to the handset. Clear chat is local-only and contact
  sync to companions waits on app-state patch writing.
- Quoted replies: incoming text with `contextInfo.stanzaId` projects as
  `Text { quoted: Some(..) }`, attributing the quoted message to us when
  `participant` is one of our login identities (the handset sent our LID) and to
  the sender otherwise. Outgoing `Text { quoted }` encodes
  `extendedTextMessage` with `stanzaId`, `participant` and the referenced
  `quotedMessage` (secret-stripped) and goes out in the plain text envelope;
  `reply` enqueues it. Live: the handset's reply to our image was projected with
  the quote and our quoted reply back was delivered and read (2026-09-20).
- Reactions: incoming `reactionMessage` payloads (field 46) now project into
  `MessageUpdated` events with the reacting device and emoji on the target key;
  `reproject` upgrades earlier opaque placeholders offline. Outgoing
  `MessageContent::Reaction` is validated, encrypted like other payloads and sent
  as `message type=reaction` with `meta content_type=add_on`; `react` enqueues it.
  Live: a contact's 👍 on our test text was decrypted and projected; our first 👍
  reply (sent without the meta child) was server-accepted but not displayed, a ❤️
  with the child but a phone-number `remoteJid` was also not shown; keying the
  reaction by the contact's LID made the handset display it (confirmed 2026-09-20).
- Contact device-list synchronization from 2.26.37.71 evidence: `devices`
  notifications are acknowledged and record the contact; sessions send the
  `usync devices version=2` query for pending contacts at start and after a
  notification; replies are verified against the contact's pinned primary
  identity with the existing ADV verifier and `11T.A08` reconciliation, then
  stored per contact atomically with hash agreement, hosted-device skips and
  unverified reasons. Unpinned contacts keep only their primary. `device_list`/
  `request_device_sync` on the client and `whatsapp-cli devices` are offline.
  Tests cover the query layout, notification parsing, verified/unverified/
  errored users, raw-id and timestamp refusals, restart, and a session flow with
  a cut on the query write followed by an interactive re-query. Device lists are
  not yet used for fan-out; own-account device sync is untraced beyond capture.
- Legacy one-time prekey replenishment is implemented from comparison 2.26.37.71
  evidence (the restored APK is that build; its class names differ from the
  2.26.37.70 references and are kept separate): generation of up to 50 keys while
  fewer than 812 are unsent, the `iq/encrypt/set` upload with identity,
  registration, type, key list and signed prekey, acknowledgement of the
  `encrypt` low-count notification, upload of unsent keys at session start and
  bounded `count_low` follow-ups. Batches are reserved before the write, uncertain
  batches are never re-sent, refusals record the code without retry, and
  post-quantum children are omitted (optional in the APK). `whatsapp-cli prekeys`
  shows redacted counts. Tests cover counts, id wrap-around without overwrite, the
  exact IQ layout and byte encodings, settlement by result/error/count_low,
  notification recognition, and a session-level flow with a socket cut mid-upload.
  Until now this client had never uploaded one-time prekeys; the first live
  upload is pending.
- `mark-read` exposes the explicit read-receipt queue from the CLI. A server clock
  sample is now saved from authenticated incoming `message`/`receipt` `t` values
  (never regressing, never invented), and `resolve_chat_mute` converts saved mute
  deadlines locally with the traced `08B.A06` rule and a caller-supplied
  everyone-mention policy, returning `None` without a mute or a sample. The
  sample source is a documented inference; the offset producer in the APK is
  still untraced. Tests cover implausible/regressing samples, saturation, and
  client-level resolution needing both inputs. All 319 workspace/all-feature
  tests, strict Clippy, rustdoc, formatting and hygiene passed.
- The connection worker is now split into the real encrypted socket adapter and a
  node-level session loop (`drive`) that takes an injected link, so tests cut the
  link at exact points without a server or Noise peer. Six cut-point tests cover:
  write failure and peer close before activation (recoverable, never ready) and
  activation refusal (terminal); `stream:error` captured encrypted and terminal;
  an explicit receipt reserved before its write, the write cut, no replay next
  epoch, and a late server ACK settling it; a delivery-ack write cut followed by
  server redelivery confirming again with no new fact rows; a key-query write cut
  leaving the reservation byte-identical and never reselected; and an unanswered
  keepalive ending the epoch with a typed recoverable timeout. Live wire behavior
  is unchanged. All 317 workspace/all-feature tests, strict Clippy, rustdoc with
  warnings denied, formatting and hygiene passed; the six tests passed five runs.
- The CLI now has one normal-runtime session: `listen --send-outbox` receives,
  acknowledges and transmits every eligible queued outbox intent from any
  idempotency namespace (including `send`), printing each candidate before it
  connects. `outbox-pending` lists the same candidates offline and `cancel
  --local-id` withdraws one before transmission; neither touches the network and
  message bodies are never printed. Eligibility is the exact read-only filter the
  outbox adapter applies before claiming, shared through one function and tested
  across namespaces, reservations and cancellation. No live combined session has
  run yet. All 311 workspace/all-feature tests, strict Clippy, formatting and
  hygiene passed.
- `send_receipt` is no longer a stub: with explicit comparison configuration it
  durably queues `read` receipts for authenticated incoming direct messages (one
  to sixteen IDs, all-or-nothing, idempotent per message) through the existing
  receipt outbox, so the next active connection sends each once and settles it by
  peer/ID/type-matched server acknowledgement. Unknown, unauthenticated, group or
  device chats, wrong kinds and error codes are refused before any write; other
  profiles still report the unsupported capability. The stanza follows the traced
  `LX/Cns.A09` builder; `type=read` is an inference from the APK's receipt
  vocabulary, documented as such, with no `list` children. Tests cover refusals,
  atomic batches, idempotence, restart without replay and ACK settlement by type.
  No live read receipt has been sent. All 309 workspace/all-feature tests, strict
  Clippy, formatting, the transport-only build and hygiene passed.
- The comparison account actor now reconnects after classified recoverable
  connection loss with capped jittered exponential backoff, a fresh epoch and a
  full handshake per attempt, bounded by `transport.reconnect` in the private
  JSON (default five attempts, 2 s base, 60 s cap; `max_attempts: 0` disables).
  Peer close and unanswered keepalives are now typed as recoverable; refusals,
  `stream:error`, revocation, missing registration, worker panics, uncertain
  saved logins and explicit disconnects never retry. The attempt budget resets
  only on server activation or an explicit `connect`; `nudge` retries at once
  within the budget; `disconnect`/shutdown cancel the wait and its waiters.
  Injected-worker tests cover fresh epochs, bounded exhaustion with the last
  error preserved, attempt reset after readiness, waiters spanning backoff,
  nudge/connect/disconnect control, disabled policy, and the outbox key-query
  reservation surviving every epoch and a store reopen byte-for-byte. `listen`
  continues through `Backoff` and reports the attempt number. No live reconnect
  was exercised; socket cut points and ACK-commit I/O failure remain unsimulated.
  All 307 workspace/all-feature tests, strict Clippy, formatting and hygiene
  passed; the actor tests passed ten consecutive runs.
- Mute timer resolution now follows the APK's local/server-clock conversion,
  with distinct unmuted/indefinite/deadline states and explicit everyone-mention
  policy. Tests cover skew, expiry, missing fields, independent mention settings
  and overflow. It neither changes stored sync data nor guesses a runtime clock
  estimate; clock observation/policy provisioning and suppression remain unwired.
  All 302 workspace/all-feature tests, strict Clippy, formatting and repository
  hygiene passed.
- The sync action applier now supports APK-derived schema-2 mute/unmute alongside
  pin/unpin, retaining finite/indefinite mute timers and optional mention metadata
  in encrypted chat state. Literal-wire and mixed-batch tests cover field presence,
  rollback in both directions, restart and idempotence. This does not yet suppress
  notifications: runtime clock/policy integration and live sync remain incomplete.
  All 301 workspace/all-feature tests, strict Clippy, formatting and repository
  hygiene checks passed after this addition.
- Incoming real-Signal crash coverage now exits a child process after durable
  capture and after authenticated session/inbox/event commit, before any receipt
  write. Reopening after capture preserves the original session and no event;
  reopening after commit preserves decrypted content and exactly one normal
  event. Redelivery returns duplicate without advancing the ratchet and still
  constructs the correct direct delivery receipt. This uses a generated peer
  and encrypted temporary store, not the live account. Socket write/ACK loss,
  server redelivery and power loss are not simulated by this test.
  The real-crypto parent test passed ten consecutive runs. The full workspace
  passed 300 tests (including the no-op child harness), strict Clippy, formatting,
  whitespace and repository hygiene checks.
- Opened-receipt queueing, reservation and ACK processing now use bounded pages
  inside the write transaction, removing the shared-checkpoint 10,000-row cutoff.
  A large-history fixture covers completed-history starvation, the 16-receipt
  reservation cap, restart/no replay, ACK deduplication and late-error rollback.
  APK-derived ACK routing checks also refuse participant/recipient/edit fields
  that cannot match this direct-only queue. No live opened receipt was sent.
  Verification: all 299 workspace/all-feature tests, strict all-target/all-feature
  Clippy, formatting, diff whitespace and repository hygiene passed.
- Key selection now pages through checkpoint history in the same transaction;
  unrelated history no longer causes the old 10,000-row refusal. A 10,600-row
  fixture covers cross-page ordering, uncommitted imports, restart and no expired
  candidate fallback; store tests cover physical cursors and rollback. All 298
  workspace/all-feature tests, strict Clippy, formatting and hygiene checks pass.
  Static follow-up also identifies the APK's outgoing share-acknowledgement gate:
  queueing a generated key for companions is not immediate activation. That
  lifecycle and live synchronization remain unfinished.
- App-state active-key selection now follows APK candidate ordering (signed epoch
  descending, device ID ascending), then age/fingerprint checks. It requires an
  explicit authorized-device context and age policy; missing eligibility never
  falls back to older keys. Restart/no-write/tie/high-bit/expiry tests cover the
  primitive. Live activation is still unwired pending trusted pairing state and
  policy resolution; no account keys were generated or activated.
- Primary linking research now distinguishes assigned-device response, pending
  login, matching device-list confirmation, saved pairing identity persistence,
  and primary critical-data bootstrap. Exact comparison APK call paths/offsets
  are in the [sync evidence](protocol/cross-device-sync.md#primary-pairing-completion-and-identity-provenance).
  This resolves the identity's provenance: it comes from saved pairing material,
  not an incoming notification/key share. Linking and bootstrap remain unfinished;
  no live pairing or key activation was performed during this static trace.
- Verified collection staging now retains encrypted current action records as
  well as integrity state and the pending application queue. Snapshot replacement
  and SET/REMOVE updates are transactional. An authenticated identical snapshot
  can rebuild a missing legacy record cache without replaying applied actions;
  tampering, rollback and restart tests cover that path. APK-complete pin/snapshot
  reconciliation and live sync authorization remain unfinished.
- ADV key-index verification now follows the APK's signed-list schema and
  `06 02` signature prefix, using an independently trusted account key. Tests
  reject changed keys, signature tampering, wrong/stale timestamps and malformed
  signed lists. Companion membership reconciliation and key activation remain
  separate; this does not yet enable live cross-device synchronization.
- Device-map reconciliation now follows APK `11T.A08`: filters unsupported
  incoming indexes, preserves newer local allocations and valid same-device
  indexes, and returns a candidate without changing inputs. Tests cover those
  branches, account substitution and index collisions. Authenticated ingestion,
  identity/device-hash validation, persistence and key activation remain incomplete.
- The APK identity-key hash now has an unsigned-byte-sort SHA-256 implementation
  and independent vector. Missing pinned identities fail; the runtime truncation
  length and notification hash binding remain unverified, so it does not activate
  keys or establish a new trusted device.
- First sync action handler: APK-derived `pin_v1` SET projects pin/unpin state
  into encrypted chat metadata. Projection, pending-batch consumption and applied
  cursor commit atomically; ordering, late rollback and restart/idempotence tests
  pass. Other actions and snapshot reconciliation remain pending; this is not yet
  wired to live sync or the normal client's chat UI.
- Collection sync now has transactional staging: verified integrity state and
  encrypted pending actions commit together against the saved baseline. Tests
  cover restart, rollback, stale patches and repeated snapshots. This does not
  apply actions or complete device authorization/live synchronization.
- Opened receipts require successful viewer completion, not download. The queue
  reserves before write, survives store reopen, never replays uncertain writes,
  and matches ACK recipient/ID/type. The earlier consumed image cannot be reopened
  and is not retroactively marked as server-opened.
- Split Noise reader/writer share serialized cipher state. Tests cover concurrent
  reads/writes, read cancellation and EOF poisoning the other half.
- New-sender prekey decryption uses the registered identity/signed prekey, consumes
  only the returned one-time-prekey ID in the inbox transaction, and refuses
  changed pinned identities. Generated independent-peer roundtrip tests pass.
- Image/audio key domains remain separate; hash, MAC, size and plaintext hash
  are verified. Ordinary media rejects view-once flags/wrappers. Voice notes are
  Ogg/Opus validated and played from a pipe. No plaintext media files are created.
- Receipt events are peer-scoped, independently accumulated and deduplicated in
  normal outbox status. Self receipts do not establish recipient delivery.
- Trusted-contact notifications use APK-derived timestamp handling; older or
  duplicate tokens cannot overwrite newer ones. Fully supported notifications
  are acknowledged after persistence. Unknown notifications remain captured.
- Cross-device work now includes an APK-derived key-share decoder, fingerprint
  validator and pinned-session-only own-device peer receive gate. These are
  tested foundations, not full sync. Key import is now atomic with the pinned
  own-device inbox/ratchet and leaves existing IDs untouched. Normal-contact shares
  never import. Authenticated device-state establishment, key activation, peer
  acknowledgements and patch application remain unfinished.
- APK-derived mutation-key derivation and index/value/snapshot/patch integrity
  checks now have independent synthetic vectors and input-tampering tests.
  They do not yet activate stored keys or apply synchronized state. Peer receipt
  subtype tracing also shows that `peer_msg` is not a blanket peer-message ACK.
- Mutation decryption now verifies the value MAC before CBC decryption and the
  exact index MAC before returning a redacted, zeroizing object. APK-derived
  envelope checks and independent encrypted SET/REMOVE fixtures pass; action
  handlers and ordered patch application remain unfinished.
- Rolling state-hash arithmetic now follows comparison `C6C.A00` with independent
  expansion/add/subtract/wraparound vectors. It calculates an immutable candidate;
  collection/index tracking and snapshot verification must precede state commit.
- The collection candidate verifier now composes patch/mutation authentication,
  per-index old-MAC tracking, replacement/removal, rolling hash and snapshot MAC.
  Independent sequence fixtures pass; failures leave source state unchanged.
  Verification remains side-effect-free; transactional staging is available above.
  Authorized key resolution, action application and live sync transport remain
  incomplete. Inline patch protobuf decoding now feeds this verifier directly,
  with explicit refusal of unsupported external-mutation/exit-code branches.
- Snapshot decoding/bootstrap verification now uses APK-derived protobuf fields,
  authenticates every record and final snapshot MAC, and rejects duplicate indexes,
  rollback and conflicting same-version state. Literal-wire fixtures pass. Actual
  snapshot retrieval/freshness and action application remain pending; transactional
  durable staging is covered above.
- Public client actor opt-in: `comparison_transport(config)`. Store opening stays
  offline; connect/disconnect/shutdown are actor-owned. Tests cover typed
  unregistered errors, successive epochs and releasing the store at shutdown.
- Incoming session/inbox commits now append normal public events atomically.
  Plain and extended text retain disappearing-message expiry metadata; already
  expired history is not exposed as new text. Quotes and unsupported media stay
  explicit unsupported events with an opaque selector, never raw attachment keys.
  Consumers must honor the expiry metadata; automatic journal expiry is not yet
  implemented. The old authenticated captures can be projected without another
  decryption. Historical LID mappings come only from original authenticated
  captures, never untrusted replacement redelivery attributes.
- A generated-peer real-crypto test forces a late inbox transaction failure and
  verifies that the ratchet rolls back, then commits and reopens the store to
  verify redelivery produces neither another ratchet advance nor another event.
- Subprocess crash tests exit without store destructors before/after a transaction
  commit. Reopening verifies intent, idempotency, journal and reservation state
  commit together; these are not power-loss or disk-full tests. A send-reservation
  reopen test preserves uncertainty until the matching ACK and never retransmits.
- Pending outbox lookup now scans a consistent snapshot in bounded pages and
  filters reservations before limiting results. Tests cover completed history,
  299 reserved requests ahead of eligible work, restart and deterministic ordering.
  This fixes queue starvation without resetting or retrying reserved requests.
- `comparison_outbox(true)` separately enables queued primary direct text and
  successfully uploaded PNG/Ogg-Opus references. It is **off by default**, including
  `listen`. Groups, quotes, captions and other media remain unsupported. Query/send
  reservations precede writes; unknown outcomes are not retried. Media references
  are sealed to the account generation/request/recipient and reused byte-for-byte
  across restart. Tests reject missing/uncertain uploads, changed bindings and
  tampered envelopes. Oversized text now fails before reserving a key query.
- `comparison_outbox_ticket(id)` restricts sending to one durable intent;
  `outbox-send --config … --message … --allow-live-request` exposes it through
  private JSON. Other queued messages remain untouched. Existing lab-send IDs
  are rejected to prevent cross-path duplication. CLI tests cover required paths
  and refusal before file/store access without explicit live opt-in.
  Media JSON now reuses bounded uploads then goes through that selected outbox;
  public upload/download convenience APIs remain incomplete. Existing saved
  lab-send IDs cannot cross over to this path.
- Shared durable path ownership now also prevents the reverse crossover:
  lab commands cannot resend an ID already owned by the normal outbox. Historical
  outbox entries are checked across namespaces in bounded pages. Same-path claims
  preserve each implementation's existing uncertainty/retry guards; ownership
  never resets an outcome. Tests cover historical records and store restart.

Recent offline corrections: normal-outbox JSON now checks the saved intent before
media preparation, so reusing a text ID for an image/voice note fails before file
access or upload. Restart regression tests also cover changed text/recipient and
read-only namespace-scoped lookup. Notification device-list hashing is now traced
separately from identity-key hashing, with explicit APK v2/v3 calculations and
independent vectors; authenticated ingestion/persistence is still pending.
Verification for these corrections: workspace/all-feature tests (283), strict
all-target/all-feature Clippy, formatting and repository hygiene passed. Rebuilt
CLI replay of the saved normal-outbox image JSON returned its saved server ACK
and delivery fact locally, without upload or connection.

Public ordinary-media integration is now implemented behind comparison opt-in:
`received_media()` lists account-bound references offline; `download_media()`
authenticates/decrypts through the existing bounded downloader and explicitly
exports to a new mode-0600 file. It never overwrites files/symlinks or exposes
view-once media. Tests cover cross-account and metadata tampering, view-once
exclusion, private/no-clobber export, and a cached public-client download with the
connection remaining stopped. This is an offline fixture result, not a new live
download. Public ordinary PNG/Ogg-Opus upload now uses the same one-shot uploader
with separate content/kind checkpoints and stable account-bound references. A
fixture-backed public-client test covers reuse/restart, normal-outbox acceptance,
one key-query reservation and uncertainty refusal without any network connection.
A fresh public-API network upload remains a separate unverified live check.
The upload integration passed 288 workspace/all-feature tests, strict workspace
Clippy and the transport-feature build without the public-client feature. The
rebuilt CLI returned the existing image ACK/delivery facts locally after uploader
refactoring; no upload or chat was repeated. Public upload-reference tests also
reject another account and altered saved protobufs.

Actor fault coverage now uses an injected connection worker with the real actor,
store and outbox reservation logic. Explicit disconnect and worker panic preserve
the query reservation byte-for-byte across a new connection epoch and store
restart; neither automatically reconnects nor selects the reserved send again.
A separate cancellation test proves late readiness from an aborted epoch cannot
complete the next connection, and shutdown releases the store. These are actor
fault tests, not simulated WhatsApp server acceptance or power-loss tests.
All three actor tests also passed ten consecutive runs; the workspace/all-feature
suite now passes 290 tests, with strict Clippy, formatting and hygiene checks.

Subsequent authenticated-record retention/recovery changes passed all 292
workspace/all-feature tests, strict all-target/all-feature Clippy, formatting,
diff whitespace and repository hygiene checks. These are offline storage/protocol
results; they do not establish live device linking or snapshot projection.

View-once crash coverage now exits a subprocess without store destructors after
real consumption, successful-viewer receipt queueing, and receipt reservation.
The parent reopens the encrypted synthetic store and verifies inaccessible media
body/protobuf, no reopening, no automatic receipt for an unknown viewer outcome,
and no replay of a reserved receipt. A queued/unreserved receipt remains eligible
exactly once; a matching synthetic ACK settles the reservation across another
restart. These are actual process-exit tests, not live viewer/socket tests or
power-loss/disk-full simulations.
The crash matrix passed ten consecutive runs; after adding explicit saved-status
assertions, the full workspace/all-feature suite passed 293 tests. Strict Clippy,
formatting, whitespace and repository hygiene also passed. The default-feature
crash matrix still passes its eight applicable tests.

Normal incoming media event integration now projects unwrapped ordinary images
and PTT audio as `NewMessage`/`Media`, with an account-bound sealed selector accepted
by the existing downloader. Image captions are retained (comparison `BWp` field 3).
`BWp`/`BWZ` context field 17 is not discarded: messages carrying context remain
unsupported until quote/expiry projection is implemented. Mixed payloads and
view-once flags/wrappers never enter this event path. A synthetic test covers
atomic rollback, restart/deduplication, caption/kind metadata and reference
resolution; it does not prove a new live media delivery or download.
Verification passed 294 workspace/all-feature tests, strict workspace Clippy,
the transport/Signal build without HTTPS/public-client features, formatting and
repository hygiene. The rebuilt CLI processed the existing account captures
offline: zero newly decrypted payloads and no requests; the redacted journal
still had one `new_message`, not a newly supported media event. Existing media
captures therefore do not establish live verification of this projection branch.

Media context follow-up: empty/non-quoted context and BWx expiration field 25 now
project into normal media events with the original timer. Quoted media (a media
message carrying a quote) remains unsupported; quoted text replies project. Expired media, or timed media without a usable authenticated route
timestamp, is excluded from listing/reference resolution/retrieval. Availability
is rechecked after decrypt/download and before public file export; existing sealed
references do not bypass expiry. Tests cover fresh/expired/missing timestamps and
revalidation of an existing reference. This is access enforcement, not secure
erasure of historical database pages, existing exports, or immutable journal data.
All 294 workspace/all-feature tests, strict Clippy, formatting and hygiene passed.
Rebuilt CLI offline backfill upgraded two historical payloads to normal events:
the journal grew from 30 to 32 entries (three total `new_message` events). A second
pass stayed at 32 with zero new decryptions and no network requests. This verifies
historical projection/idempotence on the saved account, not new incoming delivery.

### Live checks completed

- 2026-09-19 (agent-run, saved account, comparison 2.26.37.71 evidence): one
  120-second `listen --send-outbox` session uploaded the first batch of 50
  one-time prekeys (server acknowledged) and transmitted one fresh queued text to
  the existing test contact, which received server acceptance, a recipient
  delivery receipt and a read receipt within the session. A second session sent
  the first `usync devices version=2` query for that contact; the reply listed a
  single primary device and no signed key-index list, stored as primary-only. A
  third session transmitted the first explicit `read` receipt for the newest
  incoming message and received its matching server acknowledgement
  (`whatsapp-cli receipts`). No identities, IDs or saved outcomes were changed;
  handset-side display of the read state is not confirmed by these runs.

- After the worker-test seam refactor, the rebuilt public-client listener
  authenticated and activated, ran for a bounded 35 seconds, then shut down
  cleanly. Outbox transmission stayed disabled. This validates normal live
  startup/shutdown, not a live injected socket failure or new-message delivery.

- One new PNG image to the existing test contact received a matching server ACK
  through the normal outbox. An expired route initially stopped before upload;
  one explicit route refresh allowed the same saved request to proceed. Repeating
  its JSON returned the saved ACK locally, without another upload or connection.
  A separate 35-second listener saved `delivered=true`, `read=false`, `played=false`.
  Handset display is not yet confirmed.
- One new two-second synthetic Ogg/Opus beep received a matching server ACK through
  the normal outbox voice-note path. Repeating its saved JSON returned locally
  without uploading or connecting. A 35-second receive-only session then saved
  `delivered=true`, `read=false`, `played=false`. This is not proof of audible
  playback. Both media listener sessions shut down cleanly.
- Persistent connection authenticated, activated and ran for 75 seconds, then
  shut down cleanly. Repeated successfully through the public client actor.
- One ordinary incoming image downloaded, authenticated and displayed via mpv;
  player exited successfully. Encrypted cache can be reused.
- One new direct text to the existing test contact received a matching server
  acknowledgement through the public normal-outbox adapter. This establishes
  server acceptance, not handset receipt or read. Earlier image/voice sends are
  still lab-path results, not proof of normal-outbox media support.
- Repeating that exact private JSON returned the saved acknowledgement without
  network access. A separate 35-second receive-only public-client session ended
  cleanly; saved status then showed `delivered=true`, `read=false`, `played=false`.
  Recipient delivery is protocol-reported; human handset confirmation is pending.
- Saved-account crossover regression: the normal-outbox test returned its saved
  ACK/delivery status locally, while passing that same JSON to `lab-transport`
  failed at the shared ownership guard before connecting. Neither check sent a
  message or changed its request ID. The path-ownership checkpoint was adopted.
- Saved-account offline projection exposed the earlier text reply through normal
  public events (`new_message`), without new decryption or network requests.
  `events --summary` verifies kinds without logging contents/identifiers. Earlier
  unsupported placeholders remain immutable journal history, not extra replies.

### Operator-assisted checks to leave aside

1. Fresh incoming view-once picture: verify one-use display and the new receipt's
   matching server ACK/handset opened indicator. Do not reuse the consumed image.
2. Incoming ordinary voice note: verify retrieval, decoding and audible playback.
3. Message from a second controlled sender: verify new-session/prekey receiving.
4. Linked-device/cross-device scenarios need actual authorized device state and
   keys, plus further protocol implementation; no keys or sync results invented.
5. Handset confirmation for the normal-outbox integration test; its server
   acknowledgement is already saved. Do not change its ID or resend it.
6. Handset image display and voice-note playback for the normal-outbox media tests;
   both already have matching server ACKs. Keep their IDs and do not resend.

### Outgoing peer identity boundary (2026-10-07)

- [x] Refuse a changed fetched identity even with a held session; preserve queued
  intent, query reply/reservation and old ratchet without ending receive.
- [x] Inspect suspended intent IDs and hashed peer selectors with `outbox-suspended`.
- [ ] Resolve outgoing PN/LID identity transitions under an explicit evidenced
  policy, then validate with an authorized selected send. No live send attempted.

Evidence and remaining unknowns: [peer identity](protocol/peer-identity.md#outgoing-held-session-identity-guard-2026-10-07).

- [x] Inspect PN/LID pins, current-session identity agreement and authenticated
  transition history with `peer-identity --chat <primary-PN-JID>`. The recovered
  direct peer has distinct pins, both internally consistent; history links the
  held PN pin to the current LID pin. Directory refresh remains unverified.
  Default-feature CLI builds now include the corrected transport command guards.

### Signed outgoing identity replacement (2026-10-07)

- [x] Independent `transport.outgoing_identity_changes` opt-in, valid signed
  directory bundle check, atomic old-state history/identity event/send reserve,
  and own-account identity protection.
- [x] Resume one explicitly selected identity-suspended original JSON from its
  saved reply without another key query or resetting an uncertain send.
- [ ] Fetch/inspect the recovered peer's current directory bundle and validate
  one authorized selected send. Canonical outgoing policy remains pinned.

### Live peer directory validation (2026-10-07)

- [x] `peer-keys` queues/inspects a once-only directory action; atomic duplicate
  protection, verified-only explicit refresh, refusal/signature/restart coverage.
- [x] Current live directory signed bundle matches recovered LID and differs
  from held PN; no chat send or pin adoption, and saved summaries stayed unchanged.
- [ ] One new selected outgoing identity-recovery draft awaits authorization;
  all prior acknowledged sends retain their IDs and are not repeated.

## Ordinary sticker receive extension (2026-10-07)

APK 2.26.37.71's StickerMessage schema and image-key domain now support
ordinary URL-bearing WebP stickers through existing media inspection/retrieval
and sealed event references. Offline tests cover literal APK wire tags,
authentication failures, cached retrieval, quote/view-once exclusion and
reference projection. Live incoming/display, Lottie, directPath-only retrieval
and sending remain unverified or unsupported. See [sticker evidence](protocol/stickers.md).

Validation: 525 workspace/all-feature tests, 221 default-feature tests, strict
all-target/all-feature Clippy, warning-free rustdoc, generated CLI reference,
formatting and repository hygiene passed. No live request was made for this
sticker extension.

## Ordinary video and video-note receive extension (2026-10-07)

APK 2.26.37.71's shared VideoMessage schema and distinct video-key domain now
support ordinary URL-bearing MP4 receive media through existing CLI inspection,
retrieval, event projection and sealed references. Offline independent crypto
fixtures test both kinds, caption tags, once-only journal events and cached
retrieval. No live video sample or playback is established; sending, HD/streaming,
circular rendering and view-once video remain unsupported.
[Video evidence and limitations](protocol/video-media.md).

Validation for the video extension: 527 workspace/all-feature tests, 221
default-feature tests, strict all-target/all-feature Clippy, warning-free rustdoc,
generated CLI reference, formatting and repository hygiene passed. The current
release CLI was rebuilt before the bounded receive check.

The rebuilt saved account then completed one 60-second receive window with chat
sends disabled: authenticated/active, clean shutdown, unchanged 20-event journal,
unchanged nine server-acknowledged receipts, no eligible outbox and no queued
actions. An existing uncertain action remained preserved. No fresh incoming
message/video or automatic-read result was established; private evidence and
backup locations are recorded in the video protocol document.

## Safety-number research boundary (2026-10-07)

The Java legacy safety-number path, identifier normalization, unsigned identity
key ordering and version-zero QR fields are now traced from 37.71. Newer native
paths and runtime scheme selection remain untraced; no human verification bit
or account protocol record was changed. The APK native ELF contains an outer
packed archive, so missing fingerprint strings there do not prove absence.
Seven disassembler mnemonic gaps exposed by these traces are fixed and tested.
[Evidence and next native boundary](protocol/safety-number.md).

Validation: 529 workspace/all-feature tests, 223 default-feature tests, strict
Clippy, rustdoc, CLI reference, formatting and hygiene passed. The rebuilt
disassembler successfully rendered the actual APK decimal-group method with
its previous opaque arithmetic instructions named. No live request occurred.

Native follow-up: separate Superpack decoders and first-stream V3/V4 fingerprint
symbols are located. The stream is not a reconstructed ELF. The ARM64 decoder
now loads and registers 19 JNI methods locally, exiting zero. Bounded first-entry
reconstruction fails with native `9/2`; no file is produced, even with real host
memory metadata. Native algorithm recovery and human verification remain
unfinished. The helper's ineffective original threaded limits were corrected;
eight regression checks cover actual guest execution, both resource guards,
partial-output preservation and refusing unknown trace layouts. Worker tracing
shows a zero requested count triggering the partial-range oversized-file
branch; normal full extraction has exceeded short wall limits without a
native refusal/output. Ordinary full extraction now succeeds under QEMU:
80 returned entries, 79 unique valid ARM64 ELF files, independently matching
all hashes on a wrapper reproduction. V3/V4 fingerprint anchors and containing
unwind ranges are located in `libwhatsapp.so`; algorithms, dispatch mapping
and human verification remain unfinished. Full-entry/QEMU checks and
hash-pinned read-only branch/futex diagnostics are exposed.
Setup failures and remaining loader warnings are recorded in
[native reconstruction evidence](protocol/native-unpacking.md).


Native safety-number follow-up traces the shared V3/V4 5200-round SHA-512
helper, sorted key concatenation and separate identifier-bound display seed.
A pinned ARM64 slice checker validates iteration/input ordering on synthetic
data with explicitly modeled SHA/MCF operations. Full input producers,
JNI normalization, QR semantics and current scheme/human verification remain
unfinished; no account verification state or live connection changed.
[Evidence and reproducible check](protocol/safety-number.md).


The native identifier callback is now traced back into Java: PN shortening
uses HNy, LID output uses the parsed JID user, and the return conversion
requires ASCII. Native key constructors establish 32-byte sorting versus
33-byte serialized hashing inputs. Java V4 creation and result dispatch now
connect to the recovered routines; the traced result path emits QR version 0.
Current scheme selection, complete device inventory, individual QR fields,
comparison semantics and human verification remain unfinished.
[Callback and dispatch evidence](protocol/safety-number.md#native-to-java-identifier-callback-and-dispatch-mapping).


Native QR descriptors and serialization are now recorded with a reproducible
metadata extractor. Version zero carries serialized keys; version one their
identifier-free digest. V4 comparison resolves peer orientation and checks
keys, LID and additional PN/username/hosted-state conditions; LID/key equality
alone cannot establish verification. Fourteen offline tooling checks pass.
Full comparator primitive/error semantics, UI selection/inventory and the
verified-state action remain unfinished; no account verification was performed.
[QR evidence and schema](protocol/safety-number.md#native-qr-schema-serialization-and-v4-comparison).


Fingerprint comparison primitives now establish NUL/non-ASCII rejection, PN
byte equality, username strncasecmp and orientation-dependent native status
remapping in APK 37.71. The actual native byte leaf passed 862 synthetic
comparisons without modeled callees; all 16 offline tooling tests passed.
V4 comparison is mapped to jvidispatchIOO selector 4. Its Java caller, active
scheme/device inventory and human verification-state commit remain untraced.
No account verification or live request occurred.
[Evidence and reproducible check](protocol/safety-number.md#comparison-primitives-and-orientation-dependent-statuses).


APK 37.71 Java comparison selection now connects KXj.A01 to native V4
selector 4 behind holder/feature gates and an exact zero-result success test.
All three comparison callers are located. Camera/NFC success reaches result
presentation; the traced path does not establish a persisted human-verification
bit. Delayed callbacks, the verification writer and complete scheme/device
inventory remain unfinished. No account state or live request changed.
[Java caller evidence](protocol/safety-number.md#java-comparison-selection-and-scan-result-action).


APK 37.71 verification metadata writers are now traced: the local sender
rejects stale rendered identities, rechecks the stored key under lock, advances
the action sequence and writes existing metadata before companion fan-out.
Inbound peer actions are companion-only with target/key/sequence guards;
history verification has separate presence/key requirements. BWE/BT5 schemas
and ProtocolMessage field 32 are confirmed. Delayed scan callbacks only present
results. Full human-trigger wiring, active fingerprint scheme/device inventory
and client implementation remain unfinished; no account state changed.
[Writer and schema evidence](protocol/peer-identity.md#verification-metadata-writers-and-rotation-guards-3771).


The APK local verification control now traces from saved metadata/rendered
identity to explicit mark confirmation and separate unmark execution. The
writer receives that captured key, allowing its rotation guards to refuse stale
confirmation. Feature/predicate gates and complete fingerprint inventory remain
unverified; client verification metadata remains unimplemented. Two missing
DEX field-write mnemonics are fixed with literal APK instruction regression
coverage. No live account operation occurred.
[Local control evidence](protocol/peer-identity.md#local-verification-control-and-captured-identity-3771).

Verification gate follow-up: the hosted/contact-state exclusions and stored
SQL sources are now traced in [peer identity](protocol/peer-identity.md#verification-contact-exclusions-3771).
The other contact-state value's semantic name and current account gate inputs
remain unknown. Native display/result tracing distinguishes full LID QR bytes
from parsed-user decimal inputs and locates the Java KKq joining step; no
current-account safety number or verification commit is implemented.

- [x] Expose the APK 37.71 native identifier-bound display primitive through
  the library and `fingerprint-display --config` with independent synthetic
  hash-flow fixtures, private inputs and explicit unverified selection labels.
- [ ] Integrate evidenced current scheme/device-key inventory and peer comparison
  before adding human verification commits. No real peer number was calculated.

- [x] Inspect retained PN/LID device-key coverage through `peer-identity`,
  including missing-pin selectors, signed-list status and notification mismatch.
  Complete retained coverage is not proof of current APK fingerprint inventory.
- [ ] Resolve active scheme/contact-state/username inputs and full own-device
  inventory before integrating current-peer fingerprint comparison.

- [x] Export native V4 combined fingerprint protobuf version 0/1 from explicit
  private inputs, with full LID/optional fields and create-only private output.
- [ ] Render/compare real peer QR data with evidenced complete inventories and
  active scheme before committing human verification. No current-peer QR export
  or scan was performed.

2026-10-07 local verification: guarded manual mark/unmark and nullable inspection
are implemented and tested offline; both authenticated replacement archives
clear marks without advancing sequence. Automated safety-number/QR comparison,
complete APK inventory selection and companion verification sync remain open.
No real account verification action was taken. See
[local metadata](protocol/peer-identity.md#local-manual-verification-metadata-2026-10-07).

2026-10-07 native comparison validation: actual V4 core/helper/remapper execution
now reproduces 115 synthetic cases, including all 81 hosted-state combinations,
with explicitly modeled dependencies. Both scanned hosted states absent skips
known-state checks on the legacy path. This narrows implementation requirements;
production parsing/orientation/Android username semantics remain open, and no
live scan or human verification is claimed. See
[comparison execution](protocol/safety-number.md#executed-native-v4-comparison-branches-2026-10-07).

2026-10-07 native username comparison: the same 115 core statuses reproduce with
actual APK MCFStringCompare and the public Bionic strncasecmp fixture; 268 native
ASCII/prefix cases narrow the runtime comparison requirements. Object
construction/other primitives remain modeled; whole parsing, orientation,
current inventory and production scanning remain open. See
[actual string scope](protocol/safety-number.md#actual-native-ascii-username-comparison-2026-10-07).

2026-10-07 comparison implementation: `fingerprint-compare` consumes private V4
binary protobuf with explicit private known inputs. Traced conversion,
orientation and result remapping are implemented; native core/string fixtures
are covered. Complete protobuf-C parsing edges and current inventory selection
remain unverified; no live scan or automatic human-verification write occurs.
See [comparison semantics](protocol/safety-number.md#explicit-input-native-v4-binary-comparison-2026-10-07).

2026-10-07 parser correction: actual native unpacking refuted eight prost
acceptance assumptions. The comparison decoder now reproduces the 50-case
native corpus's acceptance and recognized values; groups reject even beside
otherwise valid matching data. Other edges/current inventory/live scanning
remain unverified, with no account changes. See
[native decoder evidence](protocol/safety-number.md#actual-protobuf-c-parsing-and-bounded-decoder-correction-2026-10-07).

2026-10-07 own fingerprint inventory: the own-directory source, strict positive
logout-time exclusion, alias precedence and add/remove cache invalidation are
now traced in APK 37.71. BNm is Me-presence rather than signed-list freshness.
Active own membership/authentication, scheme/contact inputs and live comparison
remain open; no peer mark or account operation occurred. See
[own-directory selection](protocol/safety-number.md#own-directory-source-logout-filtering-and-alias-precedence-2026-10-07).

2026-10-07 fingerprint contact predicate: exact field-reference searching now
locates direct DEX field producers/consumers. APK 37.71 0Da.A04 is stored as
`external_user_state`; guest/Cameo and pending-invitation branches are traced.
This narrows scheme-input requirements without manufacturing contact state.
Active inputs/authenticated inventory and live peer comparison remain open.
See [contact-state mapping](protocol/safety-number.md#fingerprint-contact-predicate-is-external-user-state-2026-10-07).

2026-10-07 fingerprint input selection: distinct captured/fresh gates now
separate V3/V4 selection from PN inclusion. Local companion-key inclusion,
username sources/normalization and nullable PN-sharing state are traced; the
smali signed-jump display is corrected and tested. Active values and authenticated
inventory still need integration, and native V3 binary comparison is unfinished.
See [input branches](protocol/safety-number.md#scheme-username-and-pn-sharing-input-branches-2026-10-07).

2026-10-07 V3 scan follow-up: traced selector-19 JNI entry, options 1/3,
PN-or-LID orientation, converter/core errors and remapping. V4 rules cannot
substitute for this path. Native corpus execution, JID/equality semantics,
authenticated inputs and live peer comparison remain open.
See [V3 branches](protocol/safety-number.md#native-v3-scan-dispatch-and-comparison-branches-2026-10-07).

2026-10-07 V3 core validation: 264 synthetic native cases validate key versions,
PN/username presence, options 1/3, hosted-state combinations and status
precedence/remapping. Generic MCFEqual and JID construction remain modeled;
full binary comparison, current authenticated inputs and human verification
are still unfinished. See [executed corpus](protocol/safety-number.md#executed-v3-core-and-remapper-corpus-2026-10-07).

2026-10-07 V3 equality/LID trace: class registrations resolve ASCII byte equality
and constructed-full-string JID equality. Device-zero flattening precedes the
primary-JID and LID-kind guards; exact number/suffix bounds are recorded.
Existing synthetic tokens do not prove converter acceptance. Native parser
execution, runtime integration and authenticated inputs remain open.
See [equality and conversion](protocol/safety-number.md#v3-equality-callbacks-and-primary-lid-conversion-2026-10-07).
