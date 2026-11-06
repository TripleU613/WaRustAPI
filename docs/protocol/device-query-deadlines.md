# Device-query deadline admission

## Confirmed local gaps, fixed offline (2026-11-06)

At6396c0c9, the actor filtered replies by DeviceQuery.deadline, but public
DeviceQuery.ingest and DeviceSyncCursor.settle did not. A direct result at the
exact30-second deadline could adopt a device list, clear pending users and mark
the query acknowledged. An error could also mark it refused after expiry.
Queued standalone/group usync nodes used unguarded NodeLink.send, so a request
could be encoded after its live reply claim had already expired.

These are Rust callback/queue lifetime defects. The existing30-second monotonic
window is unchanged; this is not a new APK timeout claim, server ban cause or
live observation. Native device parsing and verification evidence remains in
[messaging transport](messaging-transport.md).

## Admission and local expiry classification

DeviceQuery.ingest_internal now checks Instant::now() >= deadline after exact
IQ ownership matching, and again inside the write transaction before committing
its query record. If verification/user application crossed the deadline, all
staged device-list and pending-user changes roll back with that transaction.
No accepted/refused status, list or pin is inferred from an expired reply.

The dedicated local InvalidInput context device_query_deadline distinguishes
expiry from storage, verification and ownership errors. DeviceSyncCursor.settle
defers every requested user for the connection on expiry and returns the error;
new pending revisions do not make those users immediately eligible again. This
matches the existing actor timeout policy. Pending work is not deleted, and a
later connection's read-query eligibility policy is unchanged.

Standalone actor dispatch consumes only this classified expiry as a captured
late reply and continues receive. Group preparation blocks with OutcomeUnknown
when its device query expires; cancelled preparation remains IntentCancelled.
Neither branch advances to key queries or encryption from an expired device
result. Other failures continue to surface through their existing paths.

## Queue writer coverage

The internal writer envelope now has a typed WriteGuard: existing prekey
ownership/material guards or a device-query deadline. All four standalone
cursor dispatch sites (maintenance, activation, continuation and notification)
carry the original inner query deadline. All three group step handoffs use a
helper that attaches the inner device deadline when the pending step is Devices;
other group steps keep their existing writer path. No deadline is renewed at
handoff, and the group wrapper's separate deadline does not replace the inner
query's claim.

Shared writer preparation rejects an expired device envelope before encoding
or socket write, reports its actual local error and ends that writer epoch.
Query records and pending users remain untouched. Prekey admission retains its
existing checks through the same typed-envelope refactor. No new retry, reissue,
account reset or historical callback reconstruction is added.

## Regression evidence

- expired_device_claims_cannot_ingest_or_retry_new_revisions: result/error at
  29,999/30,000/30,001ms, direct/cursor expiry refusal, exact Checkpoints/pins/
  sessions preservation, reopen, and deferral despite a later notification.
  Old direct ingestion failed at exactly30,000ms by accepting the result.
- queued_device_query_refuses_encoding_at_its_original_deadline: held actual
  outbound envelopes at the same boundaries; before expiry one encoding,
  at/after expiry zero encodings, unchanged query/pending rows and no extra write.
- expired_group_device_reply_blocks_preparation_without_adopting_lists: direct
  group handler at expiry returns a blocked step, retains the uncertain query,
  adopts no list, defers fresh revisions and does not reselect the original send.

All fixtures are synthetic, use paused monotonic time and perform no network
operation. Existing multi-page/background device and group crypto flows remain
part of workspace regression validation, not a claim of native/live coverage.

## Remaining boundaries

Deadline admission is not atomic with later encoding, socket I/O or transaction
commit. This guard does not bind all query/account/record fields, resolve ADV
identity replacement, implement own-account device synchronization, or prove
native timeout/retry policy. Direct callers using their own I/O do not acquire
the actor writer guard merely by constructing a Node. Persisted requested_at
and server timestamps are not used to revive or age historical callbacks.

Validation: direct ingestion reproduced expired result adoption at30,000ms
before the fix. All 1,408 workspace/all-feature tests, strict Clippy, CLI reference,
OpenAPI library, formatting and APK coverage freshness checks passed.
