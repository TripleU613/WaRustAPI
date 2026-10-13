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

## Validation

837 all-feature, 241 default and 336 `experimental-signal-crypto`-only library
tests, strict all-target Clippy for default, all features and each single
experimental feature plus `comparison-client`, warnings-denied Rustdoc, CLI
reference check, formatting and repository hygiene passed. Test-only items that
broke single-feature `--all-targets` builds (outside the CI matrix, which runs
default and all features) are now gated on the features their tests need.
Arbitrary feature pairs such as noise+transport without Signal are still not
lint-clean and are not a supported configuration. No live request was sent.
