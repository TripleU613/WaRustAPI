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
| F1 receipt content refused by `receipts::record` (aggregate participant type outside the seven known, bad `t`, unknown key-aggregate root, non-`item` list child, more than 256 items, bad id; same id checks on the group path) | `?` ended the session without reconnecting; the receipt was never acknowledged, so it was redelivered and ended every later session | Logged; capture kept; no ACK; receive continues | `refused_receipt_and_notification_content_does_not_end_receive` |
| F2 `privacy_token` notification refused by `notifications::handle` (missing `t`, token count, non-bytes or oversized token, bad `sender_lid`) | Same | Same | Same test |
| F3 malformed `pq_count` | Same; documented as "refuses", not as ending receive | Same | `prekey_count_dispatch_bounds_uploads_and_skips_malformed_counts_without_ending_receive` |
| F4 own-companion `category=peer` message whose app-state key share does not decode | `KeyShare::from_peer_message(..)?` ended the session; nothing committed | `Unsupported`, nothing committed, no confirmation (the group path already did this) | `primary_lid_prekey_without_alias_commits_once_after_authentication` (extended) |
| F5 group view-once marked `consumed` after viewing, then redelivered | `Unsupported`: never confirmed, redelivered at every login | `Duplicate` and confirmed, as on the direct path | `consumed_group_record_is_a_duplicate_on_redelivery` |
| F7 a devices notification naming our own LID | Treated as a contact's: a device sync queued for our own LID | Excluded like our own PN (devices.rs already excluded it elsewhere) | none added; one-line comparison |
| F8 `messaging-transport.md` said `Unsupported` content "should be acknowledged" | Doc only; the code never confirmed it | Doc corrected | n/a |

Each new test was checked against the old behavior: with the fix reverted, it
fails. A refused receipt or notification is still refused again when it is
redelivered at the next login; the fix stops it ending the session, not the
redelivery. Storage errors and post-decryption commit failures on the direct
path still end the session, by design: they mean a rollback the caller must see.

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
- Notifications are durable before their ACK: identity, group dirty, privacy
  token, contact device sync. The prekey count ACK precedes its upload, as
  documented.
- Dropping `drive` at any await leaves either nothing or a complete commit.
- A server `iq get urn:xmpp:ping` gets no reply; deliberately not ported
  ([token-clock-producers](token-clock-producers.md)).

## Validation

837 all-feature, 241 default and 336 `experimental-signal-crypto`-only library
tests, strict all-target Clippy for default, all features and each single
experimental feature plus `comparison-client`, warnings-denied Rustdoc, CLI
reference check, formatting and repository hygiene passed. Test-only items that
broke single-feature `--all-targets` builds (outside the CI matrix, which runs
default and all features) are now gated on the features their tests need.
Arbitrary feature pairs such as noise+transport without Signal are still not
lint-clean and are not a supported configuration. No live request was sent.
