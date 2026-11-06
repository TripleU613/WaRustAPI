# Action settlement ordering audit

2026-11-04. **Confirmed local source defect; corrected for new sequenced replies.** This finding
does not require an APK behavior claim and has not been observed on a live account.

Before correction, `transport/actions.rs::settled` promised newest-first actions but obtained
`outcomes` in checkpoint-key order and reverses them. Action identifiers are
random hexadecimal values, not a chronology. It also reads replies in separate
transactions rather than returning one consistent snapshot of action and reply.

Consumers had a stronger replay problem than the inaccurate function comment:
`contacts::ingest` and `groups::ingest` iterate every unapplied reply and replace
saved projections. Replaying latest then older state can end on stale data.
`groups::ingest` additionally partitions by action kind. `blocklist::ingest`
partitions all fetch/block/unblock replies by kind and writes each projection
and its application marker in separate transactions. An already-known `Sent`
action must not be resent to repair any of these local projection problems.

The saved `Row` has wall-clock created/sent/settled timestamps but no durable
monotonic settlement sequence. Equal timestamps and clock rollback prevent those
fields from establishing an authoritative total receive order; old random IDs
cannot restore missing chronology. Sorting by timestamps alone would not close
this finding. Native per-feature generation/admission requirements are also not
established merely by generic IQ correlation.

Required correction: retain a monotonic receive/settlement order atomically with
new correlated replies, define legacy-order uncertainty explicitly, and make
each feature replay its supported actions across relevant kinds in the correct
order with state and once-only markers in one transaction. Establish whether
feature-specific generations or original-request guards reject superseded
results. Add multi-response restart/cut regressions whose request IDs sort in
the opposite order to receipt order. Existing markers and uncertain network
reservations remain intact; no remote requests, resets or synthetic timestamps
are authorized by this audit.

## Implemented local journal and replay policy

`record_reply` now allocates a checked monotonic sequence in the same encrypted
store transaction as its correlated terminal reply, action status and group
pending marker. Duplicate replies do not allocate another sequence. Initial
counter creation requires that no action already carries a sequence; a missing
counter after sequenced settlements refuses rather than restarting ordering.
Counter overflow and invalid zero refuse without changing the original action.
No old timestamp, ID or saved server value is converted into a synthetic sequence.

`settled_for_kinds` reads action/reply pairs in one snapshot and returns known
sequences oldest-first across all requested kinds. Duplicate or zero saved
sequences refuse. Contact replay uses this path; group replay passes its whole
kind set together. The legacy `settled` enumeration remains available for
inspectors and stronger feature-specific state verification, with its real
reverse-identifier ordering documented instead of calling it chronology.

Follow-up: the session-specific `groups::recover_pending` path previously still
enumerated pending marker keys in random-ID order, bypassing the generic
ingestor's ordering correction. It now selects sequenced settlements across
group kinds before checking their pending markers. Direct `apply_pending` also
retains an unsequenced legacy reply without projecting or marking it. The
restart regression uses descending IDs in ascending receive order to make the
old pending-key scan finish on the wrong snapshot; it also verifies unchanged
legacy checkpoints, once-only recovery and no new reservation.

This does not establish a per-group projection watermark or native generation
admission against earlier unsupported results that become interpretable after a
future decoder update. Nor does it order action replies against notifications
that lack this local sequence. Those source/generation boundaries remain open.

Old unsequenced action replies remain visible with nullable
`ActionOutcome.settlement_sequence`; `actions` text prints `unknown`. They are
excluded from automatic contact/group/blocklist projection replay without
changing their markers, reservations or replies. This is explicit historical
uncertainty, not proof that their result is invalid. Their reconciliation remains
required work. New sequence ordering also does not establish server snapshot
freshness, request-generation admission or compatibility of superseded fetches;
those per-feature policies remain unproven.

Blocklist projection, dhash and once-only marker now commit in one transaction.
The consumer re-reads its immutable original request and saved correlated reply
inside that transaction; the request supplies its set action/target. New redundant
target sidecars are no longer created; old rows remain preserved and ignored.
Supported singleton-list and primary-user shape validation precedes snapshot
replacement. Ambiguous list/hash/identity shapes refuse with static diagnostics
without partial replacement or a consumed marker. These are local supported-shape
checks, not claims of complete native parser equivalence.

Cross-kind block/unblock/fetch replay, both ID sort directions for successive
fetches, restart, a transaction cut after projection plus marker, malformed later
snapshot entries and unsequenced legacy rows have blocklist regressions. Refused
projections retain the original saved reply and absent application marker.

Focused journal tests cover IDs ordered opposite the old enumeration, reversed
wall-clock timestamps, duplicate responses, retained unsequenced rows and missing
or exhausted counters. A contact restart test replays two overlapping replies
and retains the last received projection rather than the last random ID. These
are local tests; no external account was contacted.

Validation: 1,177 workspace/all-feature tests, strict all-target/all-feature
Clippy, formatting, CLI-reference checking, coverage freshness and repository
hygiene passed. Legacy chronology and native fetch-generation admission remain
unresolved; this validation does not prove server acceptance or account safety.

## Queued action writer ownership and deadlines (2026-11-06)

**Confirmed Rust defect fixed offline:** the normal actor reserved an action as
Sent, then handed its Node to an unguarded writer queue. Replacing/removing or
settling the stored action during that wait did not prevent encoding. Known
response deadlines started before the queued write, but an already-expired
request could still be encoded; only its later reply would be refused.

The actor now reserves an internal owned action containing its original Node,
monotonic start, exact saved Sent-row bytes and optional existing producer timer.
The row snapshot is zeroized on drop and never logged. Immediately before
encoding, the writer reads the current row and requires exact bytes plus an
unexpired deadline after the read. Missing, changed, settled or expired intent
refuses the queued item and reports the local error; the connection ends. This
does not clear the Sent reservation, install a replacement or authorize replay.

The existing producer-specific duration selection is shared between writer guard
and live reply registry. The registry receives the reservation's original start,
so queue time, transaction completion and slow socket write completion do not
restart the timer. Untimed requests remain untimed; no universal IQ timeout is
introduced. Public reserve_pending still returns Nodes, and feature-owned
reserve_one_in keeps its existing contract. Only normal actor dispatch uses the
new owned writer handoff automatically.

**Confirmed static reference, APK2.26.43.72:**
[block-list callback registration and read/set separation](apk/2.26.43.72-blocklist-callback-timing/README.md)
retains1dH.A05's20000ms change request through0Sf.A0W and the separate zero-delay
fetch path. Existing timer selectors for initial AB-properties, account default
disappearing mode, two-factor, identity-only and token issue retain their source
comments/conditions unchanged. The queue ownership check is a local invariant,
not a claim that native uses Rust row bytes or durable Sent semantics. No new APK
trace, wire grammar, request family or callback duration is claimed.

Regression: `queued_action_write_retains_row_and_original_deadline` uses the
actual NodeLink queue and writer admission. It covers unchanged ownership;
missing row; independently changed status/wire/label/sent time;19999/20000/20001ms
for the20-second request; untimed fetch after60seconds; and admitted write whose
completion arrives after expiry. Refusals emit no next write, preserve checkpoint
bytes and journal head, and survive reopen. The slow-completion control verifies
that callback expiry uses the original base, not write completion.

**Limits:** no lock spans encoding/socket I/O. Other account, sidecar admission,
privacy/restriction/backoff changes after reservation are not re-evaluated by
this exact-row guard unless they change that row. Live reply ownership still
correlates by ID/deadline rather than retaining this row snapshot through
settlement; that is a separate remaining audit item. Low-level/feature-owned
writers retain their contracts. No reconnect, automatic retry, reset or live
account operation was added.

Validation:1,450 workspace/all-feature tests passed. Strict workspace/all-target/
all-feature Clippy, formatting, CLI reference regeneration, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.
