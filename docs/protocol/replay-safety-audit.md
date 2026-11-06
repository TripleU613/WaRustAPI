# Transmission replay-safety audit (2026-10-13)

A read-only code audit of every path that writes to the socket, checked against
the project's rules: never replay a reserved or uncertain send, at most one
retry receipt per message, bounded reconnects, refusals never retried. Nothing
was run against the server. Line numbers refer to the tree at commit `4fa6583`.
This is a code trace (**traced**), not live observation.

## Fixed in this change

| Finding | Fix | Evidence |
|---|---|---|
| A redelivery of the same undecryptable message under the sender's other PN/LID address missed the once-only retry row and could queue a second retry (`receipts.rs` probed only encoding kinds) | Probe the alias from `sender_pn`/`sender_lid`/`participant_pn`/`participant_lid` and the authenticated LID-to-PN mapping | [messaging-transport](messaging-transport.md#retry-once-only-across-pn-and-lid-2026-10-13) |
| The reconnect budget reset on every activation, so activate-then-drop could loop fresh logins forever | Reset only after the activated connection stayed up for `max_delay_ms` | [messaging-transport](messaging-transport.md#reconnect-budget-stability-2026-10-13) |

## Open observations (no replay of a reserved stanza)

- **One request ID in two outbox namespaces** (`send_path.rs` claim; media
  upload checkpoint `message:{request_id}:upload` in `image_upload.rs`). Two
  library namespaces sharing a `ClientRequestId` could each transmit with
  different wire IDs. Reachable only through `ClientBuilder::namespace`; the
  CLI always uses `default`. Whether to refuse cross-namespace reuse is a
  library API decision; not changed here.
- **Actions labelled `sent` before they are written.** `actions::reserve_pending`
  marks up to 16 actions `Sent` in one transaction; they are then written one
  at a time with `action_delay_ms` pauses. A cut part-way leaves unwritten ones
  labelled `sent`. Nothing is replayed (any non-pending row blocks reselection);
  only the label overstates. Receipts use `outcome_unknown` for the same case.
  The stored value is kept for compatibility; its API documentation now says
  "reserved, written or not" instead of "written" (2026-10-13).
- **Key-query status after a later local failure.** If direct preparation fails
  after the key-query reply arrived and was captured, the error ends the session
  and the query row stays `outcome_unknown`. Nothing is retransmitted; the label
  understates.

## Checked and sound

- Direct send: wire ID, attempt, in-flight and delivery-unknown commit in one
  transaction that rechecks the intent and session before the stanza is
  returned; the stanza is built from the verified snapshot.
- Direct key query: written `outcome_unknown` before the write; any query row
  excludes its intent from selection; a missed deadline ends the session without
  reconnecting.
- Group send: batches written `OutcomeUnknown` before the write; unknown,
  refused or invalid outcomes block; the final reservation rechecks membership,
  sessions and sender key; an interrupted preparation is never resumed.
- Reconnect: pending send state lives in one connection; nothing carries into
  the next. Login is written `outcome_unknown` first and a non-`success` saved
  login refuses before any network I/O.
- Panics become `Cancelled` (not retried). Only transport, timeout,
  not-connected and I/O errors reconnect; `failure`, `stream:error`, refused
  activation and refused login end the session for good.
- Message and receipt ACK matching: random 128-bit wire IDs, recipient and
  participant presence checked, only the first terminal ACK counts.
- Read/played receipts: once only, keyed by the stored inbox record, rechecked
  in the write transaction.
- Prekey uploads: keys written `OutcomeUnknown` before upload, never reselected.
- A peer's retry receipt never triggers an automatic resend in the normal
  runtime (`parse_retry` is test-only).
- `outbox-send`: returns a saved acceptance without sending; refuses failed,
  uncertain, group-reserved or non-resumable queries; reserves media upload
  before its POST.

Sent again by design (outside the rules): delivery confirmations for server
redeliveries, notification/identity ACKs, the per-connection presence and
active IQ, and `usync` device lookups re-queried after a cut.

## Receive-side audit (2026-10-13)

A second read-only trace, of the receive loop (`connection.rs` `drive`), the
direct, group and own-device inbox paths, receipts and notifications, at commit
`04445f5`. The rule checked: everything a stanza changes commits before its ACK
or delivery confirmation, nothing is consumed twice, and no single stanza can
stop receiving. Code trace (**traced**); none of it was seen live. Findings are
numbered as in the audit report.

### Fixed

| Finding | Before | Now | Test |
|---|---|---|---|
| F1 receipt content refused by `receipts::record` (aggregate participant type outside the seven known, bad `t`, unknown key-aggregate root, non-`item` list child, more than 256 items, bad id; same id checks on the group path) | `?` ended the session without reconnecting; the receipt was never acknowledged, so it was redelivered and ended every later session | `receipts::validate` checks the wire content before the store; a refusal there is logged, the capture kept, no ACK, receive continues | `refused_receipt_and_notification_content_does_not_end_receive` |
| F2 `privacy_token` notification refused by `notifications::handle` (missing `t`, token count, non-bytes or oversized token, bad `sender_lid`) | Same | Same, through `notifications::validate` | Same test |
| F3 malformed `pq_count` | Same; documented as "refuses", not as ending receive | Same | `prekey_count_dispatch_bounds_uploads_and_skips_malformed_counts_without_ending_receive` |
| F4 own-companion `category=peer` message whose app-state key share does not decode | `KeyShare::from_peer_message(..)?` ended the session; nothing committed | `Unsupported`, nothing committed, no confirmation (the group path already did this) | `primary_lid_prekey_without_alias_commits_once_after_authentication` (extended) |
| F5 group view-once marked `consumed` after viewing, then redelivered | `Unsupported`: never confirmed, redelivered at every login | `Duplicate` and confirmed, as on the direct path | `consumed_group_record_is_a_duplicate_on_redelivery` |
| F7 a devices notification naming our own LID | Treated as a contact's: a device sync queued for our own LID | Excluded like our own PN (devices.rs already excluded it elsewhere); the own-LID record is read only for a LID owner | `own_lid_devices_notification_queues_no_contact_sync` |
| F8 `messaging-transport.md` said `Unsupported` content "should be acknowledged" | Doc only; the code never confirmed it | Doc corrected | n/a |

Each new test was checked against the old behavior: with the fix reverted, it
fails. A refused receipt or notification is still refused again when it is
redelivered at the next login; the fix stops it ending the session, not the
redelivery. Faults in our own stored rows, storage errors and post-decryption
commit failures on the direct path still end the session, by design: they mean
a rollback or a local integrity problem the operator must see.

### Review corrections (same day)

An independent review of the first version of these fixes found three
problems, now corrected. All are traced in code and tested offline; none was
seen live.

- **Integrity faults were hidden as content refusals.** The first version
  caught every `InvalidInput` from `receipts::record` and
  `notifications::handle`, including ones raised *inside* the write transaction
  about our stored rows (a corrupt outbox entry, two entries sharing a wire ID,
  damaged saved facts or token timestamps). Every later receipt would then have
  been left unacknowledged while `listen` looked healthy. Now the content checks
  are pure functions (`receipts::validate`, `notifications::validate`) that the
  recording paths themselves call first (`direct_target`,
  `group_receipts::target`), so validation and recording cannot disagree, and
  the loop catches only that stage. Test:
  `corrupt_stored_rows_still_end_receive_on_a_valid_receipt`.
- **Status receipts had been ending receive since 2026-10-10.** The status
  receipt observed live on 2026-09-24 is `<participants message_id=<status
  id>/>` with **no** entries ([status](status.md#views-status-receipts-observed-2026-09-24)).
  The offline-aggregate branch added on 2026-10-10 follows APK 37.71
  `Cmk.A03`, which throws "empty participant list" on that shape, so from then
  on every status receipt ended the session; with the first version of this
  fix it was left unacknowledged instead, while its viewer row was still
  recorded. Now an entry-less `participants message_id` that `status_receipts`
  recognizes (a user viewer, delivery/read/played) is recorded as a status view
  and acknowledged, as it was before 2026-10-10, and adds no chat fact. The
  native status dispatch, which must run before `Cmk.A03`, is **not traced**.
  Status recording now also requires the receipt to be accepted. Test:
  `observed_status_receipt_is_recorded_and_acknowledged`.
- **A direct offline aggregate could be filed as a status view.**
  `status_receipts::parse` accepted any `participants message_id` child; with
  entries it is a chat message's aggregate whose root `from` is the chat. It
  now requires the entry-less shape. Test:
  `an_aggregate_with_entries_is_not_a_status_view`.

### Live-shape regression sweep (2026-10-13)

Prompted by the status-receipt regression above, a read-only sweep compared
every stanza shape the docs record as **observed** live with what the current
code does to it. Git history in this clone starts at `5c45218` (2026-10-11), so
earlier changes are dated from the docs only. **No other live shape is now
refused, misclassified or able to end receive**, and none that was
acknowledged before is now left unacknowledged; the never-acknowledged kinds
listed below are unchanged, by policy.

Fixed from the sweep:

- `lab-transport --login --receive` and the lab group-send window still called
  `receipts::record` directly, so refused receipt content ended those windows,
  and the receive window recorded a status view before the receipt was
  accepted. Both now use `receipts::validate` first, like `listen`.
- A devices or prekey-count notification without `id` (or a devices one
  without `from`) failed while building its ACK, which ended `listen`. The ACK
  is now built first; if it cannot be, the notification is refused (logged, no
  sync or upload, no ACK). No such shape was seen live. Test: extended
  `refused_receipt_and_notification_content_does_not_end_receive`.
- `status_receipts::parse` now also treats an empty child list as entry-less.
  The decoder turns the `LIST_EMPTY` token into no body but a zero-length
  `LIST_8`/`LIST_16` into an empty child list, so that form was reachable and
  would have been taken as an empty aggregate (acknowledged, view not recorded).

Recorded, not changed:

- The 2026-10-11 ACK and confirmation encoding changes are unverified live
  ([unknowns](unknowns.md#ack-and-confirmation-encodings-since-2026-10-11-live-check-pending)).
- Stanzas seen live that are never acknowledged come back at every login: `psa`
  notifications (about four per session), `mex`, non-dirty `w:gp2` and retry
  receipts. They are captured and harmless today, by the "acknowledge only what
  is processed" policy; any future handler for them must refuse content without
  `?`, or it repeats the status-receipt failure.
- Messages carrying an `edit` attribute are never delivery-confirmed
  ([sender-revoke](sender-revoke.md)); no edit has been seen live.

Checked and still as documented: direct and group delivery/read receipts (with
or without `list`, LID participants), retry receipts, `ack` of our receipts and
messages, direct `msg`/`pkmsg` from PN or LID with `sender_pn`, redelivered
duplicates, reactions, polls and votes, quoted replies, the
`msg`+`reporting`+`skmsg` group stanza, device-addressed group payloads,
undecryptable messages (one retry), `groups_dirty`, identity notifications,
`psa`/`mex`/`ib`, iq results, and `success`/`failure`/`stream:error`.

### Network-input panic audit (2026-10-13)

A read-only audit looked for panics, aborts and hangs reachable from bytes or
stanzas received from the network. Its measurements (nesting depths at which
the stack overflowed, peak memory, the fuzz count) are **observed** in scratch
runs outside the repository, not reproducible from it; the regression tests
reproduce the overflow itself. It used Clippy with `arithmetic_side_effects`,
`indexing_slicing`, `unwrap_used`, `expect_used`, `panic` and `unreachable`
(about 700 library warnings, triaged by hand), manual recursion and loop scans,
and a scratch fuzzer that ran about 3.6 million mutated frames through decode
and re-encode. The reason this matters: a worker panic becomes `Cancelled`,
which never reconnects, and an unacknowledged stanza is redelivered at every
login, so one bad stanza could stop receive for good.

Fixed:

- **Unbounded recursion in the binary decoder** on nested JID tags: a stack
  overflow aborted the process, from the reader task, before capture. Fixed
  and measured ([AD JID decoding](ad-jid-decoding.md#nested-jid-tags-are-refused-2026-10-13)).
  Only the certificate-verified server can send such a frame.
- **A frame the decoder refuses ended receive, uncaptured.** A decrypted frame
  that did not unpack or decode (an unknown token from a newer dictionary, a
  limit, a duplicate attribute, trailing bytes) became `MalformedProtocol`,
  which is not recoverable. The frame had not been captured, because capture
  follows decoding, and it was not acknowledged, so a redelivered one ended
  every session and left nothing to inspect. Now the reader passes the
  decrypted frame through; `drive` stores it once, encrypted, under
  `inbound-undecodable:sha256:<digest>` with a fixed error class (`store
  inspect` counts that kind; `whatsapp-cli undecodable` and the
  `GET /v1/inbox/undecodable` store route list them redacted and say whether
  the current decoder now reads each one), logs the class and continues. Nothing is
  acknowledged, since there is no ID. A frame that decrypted cannot have
  desynchronised the stream: records are length-delimited and authenticated.
  Continuing is **local policy**; where the APK catches a decode error is not
  traced. Test: `undecodable_frame_is_captured_and_receive_continues`.
- **Lab windows** (`lab-transport` receive and post-send windows, the
  group-send window) ended on a decode error too. They now read with
  `messaging::read_frame`, capture such a frame the same way (no epoch) and
  continue. Login, handshake and the bounded reply waits (`messaging::wait_for`,
  used for send ACKs and iq/key replies) keep `read_node`: an undecodable frame
  there still ends that flow uncaptured, which is replay-safe because the send
  or query stays reserved or uncertain.
- **Review correction (same day): a terminal stanza must still stop.** The
  first version continued after any undecodable frame, so an undecodable
  `stream:error` or `failure` (say, a newer dictionary token in its child) was
  followed by the server's close, which is a recoverable `PeerClosed`, and the
  actor logged in again up to `transport.reconnect` times, retrying what the
  server had ended. The reader now peeks the description of an undecodable
  frame (`Decoder::peek_description`); a `stream:error` or `failure` is captured
  and ends the session without reconnecting, like a decodable one. If even the
  description cannot be read, a server close that follows before any decoded
  stanza is also treated as server-ended. Test:
  `undecodable_terminal_stanzas_end_the_session_without_reconnect`.
- **Group-send window and refused retries.** A retry receipt whose content
  `receipts::validate` refused could still reach `parse_retry` and be answered
  with a fresh session; it is now preserved and neither correlated nor answered.

Recorded, not changed:

- If the reader task itself ever panicked (none found), `drive` would see the
  channel close and end with "reader ended", which is not recoverable.
- **Memory amplification (open).** A 4.9 KB compressed frame can decode to
  340 × 4,096 minimal nodes, within every limit; decode plus re-encode of one
  such frame peaked at about 200 MB in the audit's scratch run. The reader task
  decodes ahead of `drive` into a 64-slot channel while `drive` makes a durable
  write per stanza, so a burst of about 65 such frames (some 320 KB on the wire)
  could hold an estimated 11 GB of decoded nodes and abort on memory
  exhaustion. Only the server can send them. A total-node bound per frame, or a
  bound on decoded data waiting in the channel, would close it; neither is
  APK-derived, so none is set yet.
- `wacore-libsignal` 0.7.0: the parse entry points check lengths before
  indexing. The ratchet and record internals were not exhaustively audited.
- `serde_json` index assignment on stored rows (`row["status"] = …`) panics
  if a row is not an object. Every such row is written by this client, so only
  store corruption reaches it, not the network.
- No unbounded hang: the full Checkpoints scans per direct receipt and per
  count notification are bounded by store size, not input.

Checked and guarded, in brief: framing and Noise bounds, AES-GCM length checks,
nonce exhaustion, padding removal, KEM ciphertext length, hand-written protobuf
scanners (shift and depth caps), every `[0]` index (behind a length check),
`unwrap`/`expect` call sites on the receive path (each behind an invariant),
time arithmetic (wrapping or saturating), allocation sizes (bounded by the
child, frame and inflate limits), and `forbid(unsafe_code)` crate-wide.

### Open (recorded, not changed)

- **F6, speculative.** A redelivery under the sender's other PN/LID address
  (first `from=<PN>`, later `from=<LID> sender_pn=<PN>`) misses the stored
  record, because `direct_record_in` accepts a PN row only when its original
  route was LID-addressed. It would then fail decryption against a ratchet
  already past it, be classed `Undecryptable` and get one retry receipt. If the
  sender answers with the same id, a second record and event could result. The
  group key includes the participant's server, so the same gap applies there.
  Whether the server ever redelivers under the other address is **unknown**
  ([messaging-transport](messaging-transport.md#retry-once-only-across-pn-and-lid-2026-10-13)).
- **F9.** Group commit failures after decryption become `Unsupported`, which is
  replay-safe (everything rolls back, nothing is ACKed). But a member can send a
  group reaction whose key id is a wire ID we used in another chat
  (`group_reaction` "saved outgoing target chat differs"); that stanza then stays
  unconfirmed and is redelivered indefinitely. Changing it needs a decision on
  confirming refused reactions, which the APK trace does not yet support.
- **F10, availability only.** When a group stanza's pairwise child fails to
  decrypt, the `skmsg` is not tried even with a held sender key; when the `skmsg`
  fails, a correctly decrypted SKDM in the same stanza is discarded. Nothing
  commits in either case; delivery waits for the sender's resend.
- **F11.** Own-device `recipient` messages deliberately send no retry receipt,
  while own-companion `category=peer` decryption failures send an ordinary one,
  without a `category` attribute. Which the APK does for each is not traced.
- **F7 remainder.** A devices notification for our own account (PN or LID) is
  still acknowledged without processing ("captured and acknowledged only"),
  which departs from acknowledging only what is processed. Own-account device
  sync is not implemented.

### Checked and sound

- Raw capture commits before any processing.
- Direct, group and own-device `recipient` paths each commit record, session,
  identity pin, prekey or KEM consumption, LID mapping, SKDM, event and marker
  in one transaction, before any confirmation; prekey/KEM rows, session,
  identity and the inbox row are rechecked inside it. A cut after commit makes
  the redelivery `Duplicate`, which confirms again; nothing is consumed twice.
- `Undecryptable`: capture kept, no confirmation, at most one retry receipt,
  reserved `outcome_unknown` before the write and probed across PN/LID aliases.
- Receipt ACKs follow the recorded facts; unsupported receipt types stay
  unacknowledged. `record_ack` and `status_receipts::record` fail only on
  storage or our own stored rows, never on peer content, so they still propagate.
  `status_receipts::record` runs only for an accepted receipt.
- Notifications are durable before their ACK: identity, group dirty, privacy
  token, contact device sync. The prekey count ACK precedes its upload, as
  documented.
- Dropping `drive` at any await leaves either nothing or a complete commit.
- A server `iq get urn:xmpp:ping` gets no reply; deliberately not ported
  ([token-clock-producers](token-clock-producers.md)).
- Since 2026-10-13 the pure validators the receive loop runs on server stanzas
  are property-tested (`crates/whatsapp-api/tests/receive_props.rs`, 1,024
  cases per run): `receipts::validate`, `receipts::parse_retry`,
  `notifications::validate`, `status_receipts::parse`, `prekeys::low_count` and
  `prekeys::low_count_ack` return for any `receipt`/`notification` stanza built
  from the tags, attribute names and children they read, with arbitrary values,
  JIDs, bytes and nesting, and never panic. This pins the panic audit's
  conclusion for those functions; it does not cover the stateful `record`
  paths, which need a store.

## Disk full and failed commits (2026-10-13)

Design requirement ([verification](../design/verification.md#192-required-crash-matrix)):
during a disk-full or commit I/O failure, no further send or decrypt
advancement occurs until durable state is resolved. Earlier crash coverage
injected aborts inside transactions and cut sockets, but never a failing
commit, so the completion tracker listed disk full as uncovered.

**Method.** A test-only redb storage backend (`store::full_disk`) wraps the
real file backend and fails writes and file growth with `StorageFull` while a
flag is set, or fails `fsync` alone. This is simulated storage on redb 4.3,
not a full filesystem or a power cut.

**Observed (offline, redb 4.3):**

- A commit that cannot write returns the I/O error, and none of it is visible
  then or after a reopen.
- Reads on the same handle keep working.
- redb refuses every later write on that handle with `PreviousIo` ("close and
  re-open"), even after space returns. No later transaction can commit until
  the store is reopened, so no reservation, ratchet or inbox row can advance.
- Reopening while the disk is still full fails with the I/O error and changes
  nothing. Reopening after space returns recovers exactly the state committed
  before the failure, and writes work again.
- **A failed `fsync` leaves the commit unknown.** When the pages were written
  but `fsync` failed, `write` returned the error, yet the transaction was
  visible after a reopen. A store write error therefore means "maybe
  committed", never "not committed". The `ProtocolStore::write` documentation
  now says so. Every reservation in this client is already treated that way:
  a reserved send without a settled outcome stays uncertain and is never
  replayed, so a reservation that did commit despite the error blocks rather
  than resends.
- On the real session loop (`connection.rs`, injected link), a receipt
  arriving while the disk is full is not acknowledged and leaves no record,
  and the session ends with the storage error. After space returns and the
  store is reopened, the server's redelivery is stored and acknowledged.
- On the same loop, the outbox key-query reservation for a direct send:
  - **Disk full at that commit:** the reservation does not persist, no query
    is written, and after reopen the query is sent once.
  - **Only `fsync` fails:** the reservation persists. After reopen the query
    is never sent: it is reserved and unsettled, exactly as after a socket cut.

  The fault is armed on the page that carries the reservation key, so it hits
  that commit and no earlier one.

**Test correction (same day).** The first version of the receive test set
the fault immediately after activation. The session's post-activation
receipt and action reservations, which commit even when nothing is pending,
failed first, so the receipt was never processed. The test passed without
exercising what it named. It now waits for the session's readiness signal,
after which the worker idles until input arrives, and it asserts that the
receipt left no record.

Tests: `store::tests::a_full_disk_commits_nothing_and_a_reopen_recovers_committed_state`,
`store::tests::a_failed_fsync_leaves_the_commit_unknown`,
`store::tests::a_failed_rotation_commit_reports_an_unknown_outcome`,
`transport::connection::tests::a_full_disk_ends_receive_without_an_ack_and_redelivery_confirms_after_reopen`,
`transport::connection::tests::key_query_reservation_on_a_failing_disk_never_sends_unreserved_or_twice`.

**Callers (static read, same day).** Every non-test store write in both
crates was checked for code that catches a write error and then acts as if
nothing committed. None sends, resends, re-reserves or compensates on that
assumption:

- Registration, outbox, group fan-out and lab sends reserve with `?` before any
  network I/O. Enqueue is idempotent per request ID.
- Catches of `InvalidInput` only see refusals raised inside the transaction,
  before the commit; a commit failure is always `Error::Storage`.
- Reconnect never treats a storage error as recoverable.

Four sites assumed "not committed" only in their wording or in a local flag.
In each, the next write on the poisoned handle ends the work before any
network I/O:

- the automatic read-receipt queue (`connection.rs` and CLI `transport.rs`);
- the dropped `noise_appeal_token` copy, recoverable from the saved frame;
- the registration restore message;
- the lab group-send retry answer, which logs and moves on, then stops at its
  next write.

**Fixed in this change:**

- The read-receipt logs now say "may not be queued".
- The registration restore error now says the attempt stays in its state
  "unless the failed write still reached the disk", and points to
  `lab-signup status`.
- `store rotate-key`: a failed rotation commit returns
  `StoreError::RotationOutcomeUnknown`, whose message says the store may now
  open only with the new key and to keep both keys. Before this, a generic
  database error could lead someone to discard the new key while it was the
  one in force. In `a_failed_rotation_commit_reports_an_unknown_outcome`,
  exactly one key opens the store afterwards: the old key when nothing was
  written, the new key when only `fsync` failed.

**Not established:** behavior on a real full filesystem, and partial-page
writes during a power cut. Of the outgoing paths, only the direct outbox
key-query reservation has a fault-injected test. Group fan-out, media upload
and registration under a failing commit were checked by reading the code.

## Validation

837 all-feature, 241 default and 336 `experimental-signal-crypto`-only library
tests, strict all-target Clippy for default, all features and each single
experimental feature plus `comparison-client`, warnings-denied Rustdoc, CLI
reference check, formatting and repository hygiene passed. Test-only items that
broke single-feature `--all-targets` builds (outside the CI matrix, which runs
default and all features) are now gated on the features their tests need.
Arbitrary feature pairs such as noise+transport without Signal are still not
lint-clean and are not a supported configuration. No live request was sent.

### Partial encrypted comparison write regression (2026-11-04)

**Observed offline local fixture:** the verified classic-Noise handshake test
now exercises `messaging::write_and_wait_captured` with a real encoded IQ,
encrypted record and framed AsyncWrite over a local Tokio duplex stream. The
peer reads the three-byte record header and64 ciphertext bytes, then drops
the stream. The advertised record exceeds the8192-byte duplex capacity, so
the whole record cannot be buffered before disconnect. The helper returns
BrokenPipe; a second attempt on that consumed channel returns Closed. A
synthetic outcome_unknown checkpoint is unchanged after encrypted-store reopen.

Test: `transport::handshake::tests::partial_encrypted_comparison_write_keeps_durable_uncertainty_and_consumes_channel`.
The fixture synchronizes after handshake-finish consumption to avoid hiding
record bytes inside the peer's frame-reader buffer. It uses a valid local
certificate chain, not disabled certificate verification.

This adds a partial-wire cut to the existing callback-level failure and raw
framing/cancellation checks. It supports the [native pre-write promotion
boundary](apk/2.26.43.72-iq-promotion-shutdown/README.md): registry admission
and an I/O error cannot establish definite non-delivery. The checkpoint is
a test fixture, not a production outbox reservation or complete reconnect
policy; this is local duplex I/O, not a TCP/live-server observation. No runtime
behavior, sender, retry authority or live account operation changes.

## Old counters no longer request retries — 2026-11-05

**Confirmed static:** [ED4 old-counter producers and -1001 mapping](apk/2.26.43.72-old-counter-no-retry/README.md)
close one typed cause: selected native retry eligibility excludes this status.
**Inferred compatibility mapping / offline implementation:** libsignal's
DuplicatedMessage now survives as Error::SignalOldMessageCounter. Direct and
pairwise SKDM inbox handling preserves capture and continues as Unsupported,
without retry or delivery ACK. Durable authenticated same-ID duplicates keep
the existing Duplicate result. No counter/address/backend data enters the error.

Other retry error codes, native old-counter confirmation policy, sender-key
primitive classification and full route/key/count policy remain open. Existing
group crypto error formatting is also still an uncorrected redaction gap; no
primitive equivalence is inferred from direct cipher evidence. No live request.

## Group counter and producer-specific errors — 2026-11-05

**Confirmed static:** [sender-key ED4/-1001 and key-error producer paths](apk/2.26.43.72-group-counter-error-paths/README.md).
Sender-key old counters use the same excluded retry status as direct messages.
**Inferred mapping, implemented offline:** group DuplicatedMessage now retains
SignalOldMessageCounter, preserving capture without retry, delivery ACK or state
commit. Group provider diagnostics now use fixed variant-only messages. Durable
authenticated duplicates retain their prior confirmation behavior.

EDR identifies missing/invalid local prekeys in F22.A04, but group Eyj.A01 wraps
EDR as EDY before result classification. Session builder signature failures
also use a different exception from the exact “Invalid signature!” refinement.
Global exception/error-name -> wire-code mappings would lose these distinctions.
No other retry wire-code policy changed. Full native ACK policy and remaining
producer/primitive/key/count/routing parity remain open; no live request.

## Direct coordinator attribution and cause aggregation — 2026-11-05

**Correction / confirmed static:**0WJ.A0P is fast-ratchet group decrypt, not
ordinary direct decrypt. Ordinary direct0WJ.A0N -> F22.A05/A01/A02 independently
confirms ED4 -> -1001, supporting the already implemented no-retry behavior.
[Selected methods, missing-session chain and signature producer](apk/2.26.43.72-direct-retry-cause-paths/README.md).

F22.A05 missing session throws EDS -> A0N -1008 -> wire1. F22.A01 aggregates
EDY failures into EDY(List), whose outer No valid sessions. message refines to
wire8; raw BadMac/future-counter mappings cannot ignore this wrapper. Sender-key
Fj2.A00 exact Invalid signature! directly throws EDY, supporting selected group
wire5; bundle key signatures use a separate producer. No new wire code/runtime
behavior added. Remaining cause propagation, provider equivalence and full
retry/ACK policy are open. No live account operation.

## Missing-session retry error preserved — 2026-11-05

**Confirmed static:** [ordinary direct parser/session order and -1008 -> wire1](apk/2.26.43.72-direct-retry-cause-paths/README.md),
and [group absent sender-key EDS -> -1008](apk/2.26.43.72-group-counter-error-paths/README.md).
A0N constructs Fj3 beforeF22.A05 tests session presence. Eyj.A01 checks empty
sender-key record before parsing its message. These orderings differ.

**Implemented / offline tested:** IncomingResult::MissingSession preserves the
specific absence through normal receive. Direct requires no saved session and
successful SignalMessage syntax admission; a present session with missing
identity remains generic Undecryptable. No sender-key record yields MissingSession
on the group path. Normal receive queues retry with error=1 using the existing
count1/original timestamp and durable selector/reservation. Malformed direct
input does not inherit this cause. Syntax admission does not authenticate MAC
or plaintext without a session.

The existing public low-level queue_retry_for_incoming and retry_receipt helpers
retain their unclassified behavior; callers without cause evidence do not gain
a guessed default. Private construction accepts the confirmed code from normal
receive. Existing reservation lookup keeps old held/uncertain/acknowledged wire
unchanged; introducing a cause never resets retry count or creates a second
attempt. Offline CLI replay recognizes the new outcome and sends nothing.

Production-loop regression uses a structurally valid synthetic Signal message
for the direct absent-session case and absent group key for skmsg; asserts
error1/original timestamp, pending reservation before failed write, and no repeat
or delivery ACK after reconnect/redelivery. This does not prove server acceptance.
Generic failures still lack full typed eligibility/error classification, and
native key/count/routing/ACK parity remains unfinished. No live account traffic.

## Version gates and no-retry limits — 2026-11-05

**Confirmed static:** [selected native version gates and backend comparison](apk/2.26.43.72-message-version-error-paths/README.md).
ED6 is used for legacy versions by four selected message constructors; it does
not establish identity-refusal semantics. Direct Fj3 admits version nibble2..4,
while the current backend admits3..4. Full version2 crypto support is not proven.
Prekey gate3..4 and sender-key gates exactly3 have different high-version exception
paths. Ordinary direct/group ED6 -> -1007 is excluded, while EDY -> -1005 is eligible.
Prekey outer construction has a separate catch region; later ED6 handlers cannot
be applied indiscriminately to it. These differences rule out a blanket backend
unsupported-version -> no-retry mapping. Identity policy, full range/primitive
parity and producer-specific retry causes remain open. Research only; no new
wire code, runtime behavior or live request.

## Legacy sender-key message retry exclusion — 2026-11-05

**Confirmed static:** [Fj2 legacy version gate -> ED6 and selected -1007 exclusion](apk/2.26.43.72-message-version-error-paths/README.md),
with [0Yl.A00 group coordinator catch table](apk/2.26.43.72-group-counter-error-paths/README.md).
**Inferred provider mapping / implemented offline:** only receive_group's skmsg
decryption maps backend LegacyCiphertextVersion to the redacted typed error
SignalLegacySenderKeyMessage. Group inbox preserves capture and continues as
Unsupported, with no automatic retry or delivery ACK and no ratchet/event commit.
This neither authenticates the input nor creates a durable duplicate marker.

The classification stays scoped to sender-key message decrypt. SKDM ingestion,
ordinary/prekey direct parsing, newer unknown versions and short ciphertext
retain their existing policy; their full native result semantics are not inferred
from the legacy skmsg branch. Missing sender-key record is still classified
before message parsing and uses the existing MissingSession/error1 path.

Offline inbox regression mutates real synthetic sender-key ciphertext to version
nibbles0/1/2 under fresh IDs and verifies Unsupported with unchanged journal and
sender keys. Version4 and a64-byte version2 payload remain Undecryptable, showing
that newer-version and length admission are not swept into this exclusion.
Authenticated durable duplicates, including consumed content, remain Duplicate.
Other producer-specific retry code/key/count/routing/ACK parity remains unfinished.
No live account operation or server acceptance claimed.

## Sender-key signature retry error preserved — 2026-11-05

**Confirmed static / inferred provider equivalence:** [message signature verifier,
status refinement, wire5 and backend source identity](apk/2.26.43.72-sender-key-signature-retry/README.md).
Only skmsg decryption maps SignatureValidationFailed to the fixed redacted
Error::SignalInvalidSenderKeySignature. Inbox returns InvalidSenderKeySignature;
normal receive queues retry/error5. The private RetryCause enum now distinguishes
missing session (wire1) from sender-key message signature (wire5), with no guessed
generic fallback. Other signature producers are unchanged.

Production-loop regression tampers a real synthetic group ciphertext signature,
checks error5, original timestamp and reservation before failed write; reconnect
redelivery emits neither another retry nor delivery ACK. Sender-key rows remain
unchanged. Existing direct/group missing-session cases retain error1. Offline CLI
replay sends nothing. Count/key progression, other typed causes, full routing/ACK
policy and server acceptance remain unverified. No live account operation.

## Future-counter typed mapping prerequisites — 2026-11-05

**Confirmed static:** [sender-key signature-before-counter order, strict2000
boundary and backend limits](apk/2.26.43.72-sender-key-future-counter/README.md).
Native exact future-counter message refines to wire6. Backend also uses2000,
but returns generic InvalidMessage shared with CBC failures. Its selected-chain
accessor is private, front state may be wrong, and into_components advances the
front chain to its lease ceiling rather than passively exposing current position.
No provider-string matching or partial front-state mapping was added.

Native signed-int versus backend unsigned-counter comparison also differs at
high-bit inputs; this arithmetic inference prevents claiming universal cause
parity from a matching normal-range constant. Full typed cause/state access,
integer-domain compatibility, precedence and remaining retry/ACK policy are open.
Research only; no runtime behavior or live request changed.

## Sender-key future-counter retry error preserved — 2026-11-05

**Confirmed source / inferred provider equivalence / implemented offline:**
[effective matching-state metadata view and typed future-counter retry](apk/2.26.43.72-sender-key-future-retry/README.md).
Loaded-record serialization provides a passive view after reservation recovery,
without component-export advancement or front-state guessing. After backend
InvalidMessage(SenderKey), the adapter classifies only a matching-chain jump
strictly above2000 with both counters in native nonnegative signed-int range.
It uses typed fields, not provider strings, and zeroizes temporary record bytes.

The fixed typed error/outcome reaches normal retry/error6; signature precedence,
missing-session error1, signature error5, original timestamp and once-only durable
reservation remain intact. Actual backend tests cover2000/2001, retained older
chain, reservation ceiling, bad signature and high-bit nonclassification. The
production loop checks error6 and no replay/state change after uncertain write.
Offline CLI replay sends nothing. High-bit/native arithmetic equivalence, other
primitives and full retry key/count/routing/ACK parity remain open. No live traffic.

## Empty sender-key records and missing chains — 2026-11-05

**Confirmed static + platform contract / inferred equivalence:** [native empty
record versus wrapped missing-chain paths](apk/2.26.43.72-sender-key-chain-retry/README.md).
Empty record is checked before message parsing and selects error1. Unknown chain
inside a nonempty record constructs EDR then EDY(Throwable); Android's documented
class-prefixed cause-message behavior prevents the bare No keys prefix refinement,
retaining -1005 -> wire4. Full classes, platform references and a local OpenJDK
corroboration fixture are retained; no APK runtime observation is claimed.

receive_group now preserves empty-record precedence. Nonempty-record typed
NoSenderKeyState becomes SignalMissingSenderKeyChain / MissingSenderKeyChain and
normal retry/error4. Diagnostics are fixed/redacted. Production-loop fixtures
check error1 for malformed input with an empty record, error4 for a parsed unknown
chain, and unchanged keys/no repeated retry or delivery ACK after uncertain write.
Existing signature/future/missing-session cases retain5/6/1. Offline CLI replay
sends nothing. Other causes, count/key/routing/ACK parity and live acceptance
remain incomplete. No live account operation.

## Classic incoming local-prekey retry error and lazy lookup — 2026-11-05

**Confirmed static / inferred provider equivalence / implemented offline:**
[matching-session reuse, local EDR -> wire3, adapter ordering and tests](apk/2.26.43.72-incoming-local-prekey-retry/README.md).
Classic receive now decodes local prekey records only on backend key-store lookup,
so corrupt unused records do not block a matching-session follow-up. Actual
selected local key failures retain SignalIncomingLocalPrekey / IncomingLocalPrekey
through normal retry/error3, including classic pairwise group processing.

Real-crypto tests cover required missing/corrupt records and successful held-session
reuse with corrupt unused records. Normal-loop cases assert error3 and unchanged
prekeys/no repeated retry or delivery ACK after uncertain write. Three claimed
MAC fixtures were corrected to modify the nested MAC, preserving outer selectors.
Factory signed-material gates, storage failures, hybrid/KEM and later key-pair
integrity classification remain separate open work. Other typed causes/count/key/
routing/ACK parity remain incomplete; no live traffic.

## Newer unknown sender-key versions retain retry error4 — 2026-11-05

**Confirmed static / inferred provider equivalence / implemented offline:**
[version gate, cause refinement and backend identity](apk/2.26.43.72-sender-key-unknown-version/README.md).
Only skmsg version>3 typed rejection becomes SignalUnknownSenderKeyVersion /
UnknownSenderKeyVersion and normal retry/error4. Native Unknown version is not
EtY's Unknown type refinement. Lower legacy versions keep no-retry handling;
empty-record and short-input precedence remain intact. Other formats are unchanged.

Inbox tests cover high nibbles4/15, legacy0/1/2 and short2/4, with unchanged keys/
journal. The production-loop fixture checks error4 before signature verification,
original timestamp, durable uncertain reservation and no second retry/delivery ACK
on reconnect/redelivery. Offline CLI replay sends nothing. Native short-input,
other formats and full key/count/routing/ACK parity remain unfinished. No live traffic.

## Application-padding count and retry admission corrected — 2026-11-05

**Confirmed static / implemented offline:** [EHq unsigned count rule, retry
exclusion and separate CBC failure exploration](apk/2.26.43.72-application-padding-retry/README.md).
Receive now shares the native count-only rule0<count<plaintext length. It no longer
requires count<=16 or repeated padding bytes, and it rejects entire-buffer padding
rather than returning empty plaintext. Existing crypto authentication/block padding
and sender-side padding are unchanged.

SignalInvalidApplicationPadding covers empty/invalid decrypted application framing;
direct/group inbox continues as Unsupported without automatic retry, delivery ACK
or Rust atomic inbox/ratchet commit. Native Signal-store call order before EHq
trimming is retained as a separate durability/transaction tracing gap. Tests cover
unsigned counts through255, nonrepeating tails, actual authenticated ciphertext
and production-loop no-retry/no-confirmation/no-state-event behavior. CBC/provider
errors, full native commit/ACK policy and other causes remain open; no live traffic.

## One-shot action callbacks stay with their connection — 2026-11-05

**Confirmed static / inferred ownership / implemented offline:**
[selected sent-map flush, callback claim and Rust lifetime guard](apk/2.26.43.72-action-callback-lifetime/README.md).
Normal receive now requires private LiveReplies ownership from a successful local
action write on that drive. Exact result/error consumes the claim before durable
settlement; duplicates or a local failure cannot claim twice. Reconnect/reopen
never reconstructs callbacks from held Sent rows or numeric epochs.

Regression covers reserved-only rows, wrong kind/type/ID, result/error, duplicate,
local transactional failure and disk reopen. Production-loop failed-write then
reconnect leaves the old action Sent despite a late result, while the new action
settles normally. Public low-level/captured-reply reconciliation and separately
owned group membership waits remain unchanged. No replay/status reset, new route
or live account operation. Complete action deadlines, upstream/native lifecycle
branches and queued-callback policy remain open.

The same chunk rechecks the queue's saved-wire/row ID identity before bulk, single
and feature-owned reservation and before settlement. Malformed/mismatched stored
wire fails atomically instead of registering a callback under another request's
ID. Corruption regressions verify no row/status changes and no identifier echo.
A fast reply queued before local write completion still settles after success.

Validation:1334 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.

## AB-properties callback timer ordering — 2026-11-05

**Confirmed static / inferred race boundary:** current APK HBE.A00 passes
32000ms to0Sf.A0W/A0A; ready admission registers0ln.A08 before dispatch.
The positive-duration timer starts with queued ownership and is not restarted
byA0C promotion. Expiry handler4/A03 removes callback ownership; HyT selector2
BsS returns without scheduling a retry. Response lookup cancels the tracked
timer before its later response claim, without an independent age comparison.
An overdue timer is not proof that its callback has already expired.
[Complete producer/admission/expiry exports and limits](apk/2.26.43.72-abprops-callback-timer/README.md).
Rust's connection-scoped action owner still lacks per-action expiry. Porting
requires producer-specific timing and competing claims; a universal32s cutoff
or fresh post-write budget is unsupported. No runtime behavior or live traffic
changes in this research chunk.

## Initial AB-properties callback expiry ported — 2026-11-05

**Implemented offline / confirmed static timer:** the normal connection now
expires the initial abt/get/props protocol1 empty-hash callback32s from the
pre-write boundary, including local-write delay. A dedicated select branch
releases callback authority without changing the durable Sent row, replaying,
applying configuration or stopping receive. Response claims cancel the timer;
an overdue timer not yet dispatched remains a competing event, not a separate
reply-age predicate. Other action producers receive no invented default timeout.
[Producer, not-ready placeholder removal, implementation and regressions](apk/2.26.43.72-abprops-callback-timer/README.md).
Paused-clock unit and normal-driver tests cover expiry, reply cancellation,
late replies, write delay, unrelated IQs and continued receive. Rust serializes
expiry after awaited local writes; full Android concurrent scheduling, other
producer durations and configuration scheduling remain open. No live traffic.

## Block-list read/change callback timing corrected — 2026-11-05

**Confirmed static / implemented offline:** current APK1dH.A05 block/unblock
uses20000ms through0Sf.A0W/A0A;1dH.A0M fetch passes0 throughA0Q/A06,
which supplies no positive-duration replacement. The normal Rust loop now
expires block/unblock live callbacks20s from the pre-write boundary, retaining
Sent uncertainty without list/hash mutation, rollback or retry. Fetch receives
no invented20/32s timeout. Reply cancellation and expiry remain competing claims.
[Complete producer, failure callback, tests and limits](apk/2.26.43.72-blocklist-callback-timing/README.md).
Paused-clock tests cover19/20s, delayed write, result/error cancellation and
fetch reply at60s. Normal-driver delayed-write/late-reply tests now cover both
changes and initial AB-properties, continued receive and no resend/projection.
The native change failure callback reaches2KT.CE5(false); its concrete downstream
side effects remain open. Full native scheduler parity and other producers'
deadlines remain unfinished. No live account operation.

## Account-default disappearing timer callback expiry — 2026-11-05

**Confirmed static / implemented offline:** current APK ChangeDMSettingActivity.A0c
set and0oy.A00 get both pass20000ms to0Sf.A0W. Normal live action ownership
now assigns20s to these account-default shapes from the pre-write boundary.
Expiry preserves durable Sent uncertainty without replay, applying the requested
value or local rollback. Result/error cancels that timer. Per-chat E2E timer
messages and usync disappearing_mode children do not inherit it.
[Producer, native success/failure distinction, tests and limits](apk/2.26.43.72-default-timer-callback/README.md).
Paused-clock tests cover19/20s, delayed write, result/error and a separate
AB-properties callback surviving until32s. The normal-driver expiry test also
covers default-timer write delay, late reply, receive ACK and no resend.
Native timeout publishes the saved duration through observers; successful
callback has a distinct preference-write path. Complete Rust default-setting
adoption, alternate-mode gates, native error/observer behavior and exact
concurrent scheduling remain open. No live account operation.

## About reply semantics and UI/network lifetime distinguished — 2026-11-05

**Confirmed static / implemented offline:** current APK FxR.A04 sends writer29,
which storesJAj in a legacy map shared by1rj/1rh/1rD. SetAboutInfo.A5f's32s
UI wait does not itself remove the callback in the selected handler. No32s
network timer is invented for About. Selected reader teardown invokes failure
callbacks then clears this map; full lifecycle/concurrency policy remains open.
Native outer result removes the callback andJAj.A02 ignores the payload, so a
synthetic nested error extension must not turn typed About success into refusal.
Rust now preserves this producer-specific meaning; outer error/code and other
producers' nested refusals remain intact. [Full alias/dispatch evidence and regression limits](apk/2.26.43.72-about-callback-lifetime/README.md).
Paused-clock regression covers empty/nonempty About at60s, independent32s
AB-properties expiry, outer result/error and duplicate claims. Normal-driver
regression verifiesOk/no refusal code, continued receipt ACK and no resend.
Saved About adoption, UI observers and full native lifecycle remain unfinished;
no live account operation or observed server extension response is claimed.

## Profile-photo callback construction and upload result — 2026-11-05

**Confirmed static / implemented offline:** current Hyd.A00's namespace helper
also returns the to=s.whatsapp.net attribute, which Hyd appends; the existing
Rust address is confirmed rather than inferred. 6p1 lookup builds that address
explicitly. Both selected producers pass duration0 through distinctA0W/A0R
wrappers, so no generic20/32s callback timer is assigned. Upload success requires
the first child picture and ignores later error extensions; Rust now preserves
this typed success shape instead of reporting a synthetic nested refusal.
Outer errors and other action refusal parsers remain intact.
[Producer/helpers, separate lookup parser, tests and limits](apk/2.26.43.72-profile-photo-callbacks/README.md).
Tests cover exact4/5 attribute counts, server address, same-connection claims at60s,
outer result/error, duplicate/mislabeled requests and normal receive continuation.
Lookup result adoption, full malformed-result admission, cache/removal/account-center,
target normalization and image transforms remain unfinished. No live traffic or
observed server extension response is claimed.

## Two-step verification callback and diagnostic privacy — 2026-11-05

**Confirmed static / implemented offline:** current1HL.A07 passes32000ms to
0Sf.A0W; FUS.CK4 schedules original supplied values without reading response
payload. Normal two-factor-set callbacks now expire32s from pre-write start,
retaining Sent uncertainty without retry or credential adoption. Typed outer
result ignores nested error extensions; outer error/code remains separate.
[Producer, timeout/success paths, tests and limits](apk/2.26.43.72-two-factor-callback/README.md).
TwoFactorChange's derived Debug exposed PIN/email; custom Debug now preserves
field presence and redacts values without changing inputs or request bytes.
Paused-clock tests cover31/32s, delayed write, PIN/disable/email-only changes,
result/error/duplicate and unchanged durable rows after expiry. Normal-driver
late-result/receive/no-resend test includes this producer. Native credential
adoption, encryption, email-status job and error/observer behavior remain open;
no live account operation or server extension response is claimed.

## Trusted-contact token callback and execution-time gap — 2026-11-05

**Confirmed static / implemented offline:** current infra/privacytoken
GeneratePrivacyTokenJob.A0L request299 uses32000ms through0Sf.A0W. This is
separate from client-auth request314. Normal trusted-contact callbacks now
expire32s from pre-write start, retaining Sent uncertainty and scheduled state
without retry/window reset. Typed result follows8R8 selector3's captured-data
success rather than generic nested-refusal parsing; outer errors stay separate.
[Full job/callback evidence, tests and important remaining gap](apk/2.26.43.72-privacy-token-callback/README.md).
PN/LID tests cover31/32s, delayed write, result/error/duplicate, checkpoint
preservation, unchanged same-window scheduling and same-namespace exclusions.
Normal-driver late-reply/receive/no-resend test includes the actual scheduler.
**Remaining confirmed gap:** the APK reloads scheduled state and checks the
execution-time threshold before constructing the request; Rust freezes t at
queue time. Queued supersession/coalescing, effective expiry/clock, success-store
and native job retry policy need a separate port. Sent rows must not be rewritten
or replayed as a workaround. No live account operation.

## Trusted-contact queued timestamp reload and age floor — 2026-11-05

**Confirmed static / implemented offline:**87x.A04 advances scheduled state before
pending-set admission; the job reloads the stored timestamp at execution. Queued
Rust token actions now coalesce by user/schedule owner and refresh their stored
wire from current state in the same transaction that marks Sent. Reserved bytes,
IDs and send facts are never rewritten or replayed. Legacy duplicate ownership
is recovered without resurrecting a current Sent/Ok/Error action.
[Native age-floor proof, ownership, clock and regressions](apk/2.26.43.72-token-reservation-refresh/README.md).
0hu's max-per-branch then min construction proves a mandatory182-day lower bound.
Reservation defers missing state, below-floor time and superseded rows, recording
optional ActionOutcome.deferred_reason for actions plain/JSON; unrelated drain
and receive continue. Scheduling/reservation use confirmed A04 clock semantics,
without inferred stanza time. Boundary, reopen, corruption rollback, legacy and
normal-driver tests cover these paths. Stricter configured bucket admission,
full native job/pending/retry/counterpart and success-store policy remain open.
No live account operation; the age floor is necessary, not full eligibility.

## Trusted-contact successful issue state — 2026-11-05

Confirmed current 0ht.A0V/8R8.CK4 selected timestamp persistence now accompanies
Rust token action settlement atomically. Captured request time confirms a matching
schedule/nulls real metadata, or advances nonnull real time while protecting a
newer schedule; missing state inserts captured completion, dominating state stays
unchanged. A0W carries nonnegative decoded issue time: known SQL NULL means prior
sent time, raw zero means unissued. Missing/negative projection initializes zero.
Legacy unknown completion history stays distinct.
Existing actions plain/JSON reports optional token_issue_completion without users
or times. No historical Ok replay/backfill; legacy missing metadata is unknown.
[Native source, independent SQL vectors, reorder/reopen/rollback tests and limits](apk/2.26.43.72-token-success-state/README.md).
Configured bucket policy, canonical counterparts and full job/cache lifecycle remain
unfinished. No live account operation or automatic replay is introduced.

## Contact-check preparation and reservation atomicity —2026-11-05

Confirmed local fault: separate action/correlation commits could leave a sendable
orphan on disk. queue_check now prepares both atomically. Real full-disk and
uncertain-fsync tests verify neither/both consistency after reopen. Existing
Queued requests lacking correlation defer with contact_check_correlation_missing;
invalid/mismatched correlation refuses locally, and unrelated eligible traffic
drain continues. No automatic correlation repair, Sent reset or replay.
[Fault evidence, native caller-dependent usync timers and tests](apk/2.26.43.72-contact-queue-atomicity/README.md).
Current0pD.A07 receives a caller timeout: selected HxV lightweight call is32s,
while another0p7 call is128s. Rust contact-check's exact factory/deadline remains
unported; no universal usync timer is inferred. No live traffic was sent.

## Contact protocol metadata/backoff obligation —2026-11-05

Confirmed current0pD.CK4/A01 retains contact protocol success/integrity/error,
refresh and backoff separately from per-user in/out/invalid. Error backoff uses
seconds*1000 capped at3600000ms, with a missing-attribute default before the cap.
Selected observers persist positive deadlines using08q.A00;503 also writes
contact-executor global_backoff_time, while429 has an additional guarded scope.
Regular and lightweight query observers write different per-scope preferences.
[Bindings, full selected methods, strict deadline gate and remaining port requirements](apk/2.26.43.72-contact-protocol-backoff/README.md).
Rust still lacks full metadata/wait adoption. Outer IQ success alone is not contact
protocol success. No new wait, retry or account-state mutation is implemented here.

## Single-contact projection bound to original request —2026-11-05

A reproduced local defect let a successful contact-check reply overwrite status
for any returned phone. Projection now reloads/validates saved request provenance
and applies only a uniquely matching requested number. Unrelated/duplicate/
missing-origin/malformed cases preserve contact data and record a content-free
ActionOutcome.contact_projection through existing actions inspection. Status,
projection diagnostic and once-only marker commit atomically; settled wire/reply/
sequence and historical consumed markers stay unchanged. No query is replayed.
[Red regression, disk-fault/reopen tests, inspection and native-policy limits](contact-projection-correlation.md).
Current native protocol integrity/backoff and exact factory/deadline remain open.

## Usync outer-refusal numeric metadata —2026-11-05

Confirmed current0pD.Bvj uses absent code0/backoff-1, unlike protocol-result
A01's absent-backoff default before its cap. Present values use signed JVM
parse and wrapping seconds*1000 capped at3600000ms. Durable usync get-action
outer errors now save optional ActionOutcome.usync_refusal with presence flags
and fixed numeric parse failures, exposed through existing actions plain/JSON.
Metadata survives projection/reopen; duplicate replies do not rewrite it.
[Full callback, independent JVM vectors, tests and scope limits](apk/2.26.43.72-usync-outer-refusal/README.md).
This does not enforce a wait, select an observer/deadline or replay historical
callbacks. Factory/read/reset/clock adoption remains unfinished; no live traffic.


## Contact backoff gates and reset callers — 2026-11-05

**Confirmed static**, APK2.26.43.72: the selected contact executor filters
protocols individually; its FULL contact/sidelist wait-key selection depends on
property37559. Exact0pQ.A02 direct callers are registration contact-sync start
and IBi selector16 scheduled by a contact `sync` notification. This event-bound
reset does not clear every wait key and is not generic retry permission.
[Evidence, delay arithmetic, branch scope and remaining port requirements](apk/2.26.43.72-contact-backoff-gates/README.md).
Rust eligibility/reset behavior is unchanged; query reachability, clock authority,
authenticated notification/reset adoption and effective configuration remain open.
No live account operations or server acceptance claims.


## Contact protocol refusal inspection — 2026-11-05

**Confirmed static and locally tested**, APK2.26.43.72: an outer usync IQ result
can carry a separate contact-protocol refusal. Actions now durably expose
`contact_protocol_refusal` with native backoff-first parsing, absentcode-1 and
absentbackoff3600000ms. Outer-IQ refusal defaults remain separate. Numeric
parse failures are fixed/redacted diagnostics, not guessed waits. Existing
plain/JSON actions inspection retains the field across projection and reopen;
duplicates cannot settle again. Legacy absent fields remain unknown.
[Source readers, JVM oracle, tests and unported adoption/wait policy](apk/2.26.43.72-contact-protocol-refusal/README.md).
Outer action status/per-user projection remain separate; no automatic retry,
wait enforcement, live operations or complete native integrity-policy claim.


## Pinned contact backoff clock and NTP scheduling — 2026-11-05

**Confirmed static**, APK2.26.43.72: contact backoffs use NTP-preferred08q.A00,
whereas token scheduling uses08q.A04. Constructor NTP cache admission is less
than one day; selected in-process getters do not expire it. Newly traced
property170 selects WorkManager12h/6h-flex versus12h elapsed alarm scheduling;
both setup paths also invoke acquisition immediately. A separate six-hour
process-static recent-success guard may skip it. Property2225 conditionally
controls the dual one-day adoption rejection; neither effective value is known.
[Source exports, field/caller census, search limits and port obligations](apk/2.26.43.72-contact-backoff-clock/README.md).
No Rust clock/eligibility, NTP acquisition, retry/reset or live account changes.


## Client keepalive error response correction — 2026-11-05

**Confirmed static / locally corrected**, APK2.26.43.72:1xa.Bvj routes a ping
IQ error through the sameFea(24) manager response as validated success. Rust
previously ended receive as nonrecoverable keepalive refused. A matched live
ping error now clears only that ping and receive continues with a fixed
redacted diagnostic. Duplicate old errors do not consume successor pings;
non-replies/late replies and unowned IQs retain existing admission. No clock,
account, request retry or restriction-reset authority is added.
[Callback binding, red/green regression and open cadence/schema gaps](apk/2.26.43.72-client-ping-errors/README.md).
Full native keepalive scheduling/configuration and successful-result schema
remain unported; no live account operation or causal ban-prevention claim.


## Client pong success schema and consumed callback — 2026-11-05

**Confirmed static / corrected locally**, APK2.26.43.72:1xa.CK4 requires parsed
from, original-request ID equality, bounded signed-long t and exactresult.
Malformed responses throw recoverableLIL before the manager response, after
one-shot IQ callback removal. Rust now separates callback claim from ping timer:
malformedresult consumes the claim, leaves the original timeout active and
continues receive; a later same-ID validresult/error cannot revive it. Fixed
field diagnostics contain no supplied values. Error-response behavior remains
separate. Existing Rust JID text bounds are a local supported-path limit.
[Reader/exception/ownership proof, regressions and remaining gaps](apk/2.26.43.72-client-pong-schema/README.md).
No clock adoption, replay, new request, cadence/timeout change or live traffic.


## Client ping interval restarts on response — 2026-11-05

**Confirmed static / locally corrected**, APK2.26.43.72:1A7.A05 schedules the
next periodic ping from its accepted response. Rust's fixed shared tick could
send again1second after a response delayed19seconds. Separate maintenance and
next-ping deadlines now restart only the ping interval; queued work still drains
at its original local maintenance deadline. Invalid/duplicate/unowned replies
do not restart it. Red/green actor regression covers bothresult anderror.
[Evidence, packaged15s/240s/32s entries and resolver limits](apk/2.26.43.72-client-ping-scheduling/README.md).
Effective native durations/configuration/mode remain unported; existing local
values, wire requests, uncertainty guards and no-live-operation boundary remain.


## Client ping first-access configuration precedence — 2026-11-05

**Confirmed static**, APK2.26.43.72: ping getters use00G.A02 with
firstAccessValue=true, selectingA07 rather than the token getters' ordinary
cache. Selected readiness resetA0t replacesA0d, notA07. The SDK comparison
branch clones exposure options while preserving first-access semantics; a
successful legacy read can populate the winner before SDKputIfAbsent. Direct
SDK selection remains distinct. No effective value is inferred from raw AB
rows or missing specifiers, and cache owner/replacement scope remains open.
[Complete options/getters, field census and conditional precedence proof](apk/2.26.43.72-client-ping-first-access/README.md).
Research only; native effective timing/configuration remains unported. No live,
request, timer, retry/reset, account or clock behavior changed.

### 2026-11-05: final compiled ping defaults

**Confirmed static / derived static, APK 2.26.43.72:** whole integer-builder
argument-flow analysis of 08Y.<init>, A0B, A0I and A0Q finds 4158 integer puts,
3585 distinct keys, and exactly one write each for 15103=32000, 15183=240000
and 15270=15000. The final snapshot binds to this.A02; no later target write
replaces those entries. This closes the selected-assignment-only caveat for
these three keys. **Unknown/unported:** effective account values and configuration
owner lifetime. No runtime timer, retry, replay or live-account behavior changes.
[Reproducible bounded proof and limits](apk/2.26.43.72-client-ping-final-defaults/README.md).

### 2026-11-05: ping channel lifecycle and scheduler-mode authority

**Confirmed static, APK 2.26.43.72:** 0YB.A01 -> 2JJ selector 2 binds startup.
Same connected callback identity returns without resetting; new accepted channel
resets the ping counter/outstanding timestamp/timeout flag and schedules the
initial interval. Counter reset does not reset the configuration first-access
cache. Scheduler mode derives from push-registration preference presence and
logins_with_messages < 3, then effective property 6493 for Runnable selection.
**Unknown/unported:** actual preference/config values, complete owner lifetime,
on-demand/channel-replacement callback interactions and receiver failure paths.
[Source, exact field census and transition table](apk/2.26.43.72-client-ping-channel-lifecycle/README.md).
No runtime policy or live-account behavior changed.

### 2026-11-05: normal pings and ACK-confirmation callbacks differ

**Confirmed static, APK 2.26.43.72:** A08 uses normal A04 pending guards; direct
A0B callers also include offline-resume and coroutine pre-ACK confirmation.
Bound Fef14 and Feq9 continuations request A0J(marker) only after 1xa's validated
success, not IQ error or delivery failure. Enqueued manager-liveness work and
synchronous success completion have no join establishing a fixed execution order.
A0A additionally serves arbitrary calling/signaling callback families; shared
request shape does not establish shared response policy. **Still open:** registry
cleanup across channel replacement, coroutine cancellation and complete durable
replay/retirement parity. No Rust timing, retry, replay or account behavior changed.
[Producer census, constructor bindings and limitations](apk/2.26.43.72-client-ping-producer-families/README.md).

### 2026-11-05: ping registry reset and already-claimed callbacks

**Confirmed static, APK 2.26.43.72:** 1xZ.BVv selects gated alternate dispatch;
nonnull success continuations use 1xa's serial path. Reset A0F posts handler 3,
which snapshots/clears awaiting-response map A0H and schedules delivery failure.
Queued A0I and already-claimed worker ownership are distinct; no synchronous
all-callback cancellation follows. **Verified locally:** stale result/error from
a prior closed link cannot settle or extend the restarted link's ping after store
reopen. **Open:** complete queued disposal, native executor/registration ordering
and claimed-worker cancellation. No production timing/retry policy changed.
[Evidence, regression and limits](apk/2.26.43.72-client-ping-registry-lifecycle/README.md).

### 2026-11-05: disconnect cancels a scoped retriable-IQ job map

**Confirmed static, APK 2.26.43.72:** 0Sf.A0E holds delayed T56/TKl retry jobs,
not queued IQ callback ownership. Disconnect requests job cancellation; selected
coroutine cancellation forwards original delivery failure. Scheduling requires
remaining budget and callback CmQ permission; errors additionally require allowed
500..599 code excluding the retained eight-code set. Direct application caller
is a feature-gated status-privacy job, not every IQ. **Unported/unknown:** caller
callback eligibility, full job-queue/cancellation ordering and durable recovery.
[Complete callback/budget bodies, caller census and boundaries](apk/2.26.43.72-iq-retry-job-cancellation/README.md).
No runtime retry, replay, timer, account or persistence policy changed.

### 2026-11-05: status-privacy callback distinguishes root and nested errors

**Confirmed static, APK 2.26.43.72:** original 8RJ17 permits bounded wrapper retry,
but T56 reads root IQ code/backoff while the original reads nested error code.
Nested 500 alone reaches original code/future handling and the job's exception;
404 logs no settings without success adoption. Successful parser requires privacy
child/type before conditional audience persistence. **Open:** effective flags,
store transactions, scheduler limits and durable recovery. Rust has no matching
status-audience fetch/adoption path; no runtime retry or settings change added.
[Exact callback binding, branch table and limits](apk/2.26.43.72-status-privacy-fetch-callback/README.md).

### 2026-11-05: preserve About refusal callback codes

**Confirmed static / implemented offline, APK 2.26.43.72:** the ordinary About
error path processes signed Java Integer codes for each direct error child in
order, stopping on malformed present input. Existing generic u16 metadata lost
signed/large/subsequent values. Optional ActionOutcome.about_refusal now retains
the codes and malformed boundary atomically with settlement, exposed by actions
plain/JSON. Legacy absence remains unknown. Duplicate/reopen/no-replay and JVM
oracle tests cover the correction. No retry, cooldown or saved About adoption.
[Exact source, oracle and limits](apk/2.26.43.72-about-error-codes/README.md).

### 2026-11-05: current prekey error backoff has scoped state

**Confirmed static, APK 2.26.43.72:** non-PQ error selector 76 moves affected
existing in-flight entries to pending only for 500..599. A01 mode and A00 posted
flag control scheduling; the 10/610 generator starts at zero delay. Delayed drain
clears posted flag but retains mode; explicit reset differs. Complete normal
admission does not establish a blanket stop of all key queries. **Unknown:**
network-to-listener attribution, external reset authority and durable job/write
recovery. No Rust retry, admission or reservation policy changed.
[Exact anchors, source transitions and delay derivation](apk/2.26.43.72-prekey-error-backoff/README.md).

### 2026-11-05: current key-fetch error attribution and signed codes

**Confirmed static / implemented offline, APK 2.26.43.72:** writer binds Dxh to
captured devices; outer errors map to that set and per-user errors are grouped
only while iterating original requested devices. A1S/A0e constructs event 76
matching the backoff listener. Signed Java integer code grammar is retained.
DirectoryInspection.server_error_code widens from Option<u16> to Option<i32>,
preserving valid negative/large/Unicode codes after existing unique-context guards.
JVM-case regressions cover root/user attribution, redaction, reopen and no refresh
or resend. Other inspectors/generic action fields remain unchanged. Full native
registration, duplicate/missing-device and durable retry ownership remain open.
[Source, API consequence and limits](apk/2.26.43.72-prekey-wire-error-attribution/README.md).

### 2026-11-05: non-reply IQ cannot consume a pending key page

**Implemented offline, confirmed APK43.72 outer dispatch:** key-batch settle now
returns false without store mutation for IQs lacking result/error type, even with
matching ID and error child. Previously reusable settlement could terminally
classify such input and lose the later actual reply. Normal group-driver already
filtered these types. Red/green regression covers state equality, blocked reserve,
reopen, later valid settlement, duplicate refusal and complete local assembly.
No new callback authority, key adoption, timeout or automatic retry is added.
[Evidence, scope and unresolved native lifetime](device-key-batches.md#key-batch-reply-admission-correction-2026-11-05).

## CLI direct-send ACK retains intent authority (2026-11-06)

**Confirmed local gap / fixed offline:** transport_send::send froze its intent
before the network write, but constructed its result from that cached value and
unconditionally replaced the checkpoint after the awaited ACK. A changed, missing
or already-settled durable intent was not checked at settlement.

The settlement transaction now requires expected status=outcome_unknown and exact
current intent equality before changing status or writing its additional response
records. Existing shared ACK matching, outcome classification, deadline and
capture-before-return remain unchanged. Pending frames and matching ACK are
already retained by write_and_wait_captured, so stale settlement refusal does not
discard network evidence. No session/identity is regenerated or reinstalled; no
uncertain send is authorized for replay. This is a local persistence invariant,
not a newly inferred Android ACK policy.

Regressions cover missing intent, changed wire ID/reservation, already refused
or acknowledged intent, unchanged captured evidence/checkpoints/journal and
refusal after reopening. A valid exact reservation settles once, a duplicate
cannot rewrite it, and saved Signal session bytes remain unchanged. All inputs
are synthetic; no network or account operation ran.

**Limits:** this closes CLI direct-send result replacement only. It does not add
a session-wide credential lock, prove handset delivery, or port all group/media
send paths. Existing auth-challenge459 and unrelated-family outcomes remain
uncertain, without automatic retry.
