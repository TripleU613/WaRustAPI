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
