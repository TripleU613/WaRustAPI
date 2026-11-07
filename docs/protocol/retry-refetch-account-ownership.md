# Retry refetch account ownership

## Reproduced handoff gap (2026-11-07)

**Implemented/offline:** RefetchWriteGuard froze the durable job and fetching
answer, but not the saved account at reservation. A regression held those rows
constant, changed Signal registration ID31 to32, and demonstrated that the old
guard still passed. The pre-fix expected-refusal assertion failed. This could
admit a stale retry ACK or directory query after queue handoff.

Successful reserve_refetch already checks the supplied registered account in
its write transaction. The production guard now fingerprints that checked
snapshot using its generation; it does not reload a replacement account after
reservation and bless it as the original. The shared transport/account_authority
helper binds store/account generation, phone, own JID, registration state/ID and
identity public/private bytes or explicit absence. Observation-only timestamps
and credentials outside that authority remain excluded. Only a digest is held,
with no extra stanza, durable record or inspection field. DeviceQuery uses the
same internal helper; its prior regression remains in the full suite.

## Existing write cuts and live callbacks

RefetchWriteGuard.check_write_authority now reloads current saved authority
before its existing job/answer ownership checks. NodeLink's existing queued ACK
and query preparation both use that guard. A change before ACK encoding blocks
both writes; a change after ACK acknowledgement but before query preparation
blocks the query without manufacturing permission to replay the ACK.

ActiveRefetches now forwards its complete frozen guard into completion rather
than only the job. Both the refusal transaction and successful replacement-
session/resend transaction recheck account authority before durable mutation.
Existing current-account, original/source, peer identity/session and once-only
job checks remain independent. A failed live callback is still removed before
processing: failure cannot recreate a replayable callback. Refusal records,
fetching answer and job remain unchanged when account authority fails. No key
query or uncertain resend is replayed, and no historical job is granted a new
live callback by this change.

The public offline complete_refetch adapter still has no frozen live callback;
its existing caller-account and durable-job rules are unchanged. This chunk
binds actual live callback/handoff ownership, not a new durable account-provenance
migration for old refetch rows. A guard check also does not hold a lock over a
remote network write; the established local encoding/write cut remains the
boundary. Complete native receipt-carried-key adoption is still unsupported.

## Evidence, tests and limits

**Confirmed source/local:** retry_answer.rs reserve_refetch, RefetchWriteGuard,
ActiveRefetches.complete and complete_refetch_inner; connection.rs queued
send_retry_answer guards. Native ACK-before-query and disconnect callback
ownership evidence remains in [retry ACK dispatch](retry-ack-dispatch.md) and
[retry answers](retry-answers.md). This is a local ownership correction, not a
new APK wire field or server restriction claim; no new disassembly was needed.

refetch_handoff_rejects_changed_account_before_ack_or_query demonstrates the
pre-fix acceptance, post-fix refusal, unchanged checkpoints and stale live outer-
error callback refusal with consumed live authority. The queued_refetch test
adds account creation after an account-less synthetic guard at both real ACK
and query preparation cuts; it proves no subsequent writes and unchanged rows
across disk reopen. Existing replaced-job/settled-answer, disconnect/no-replay,
current-account cryptographic ownership and device-query tests remain active.
Account-less tests are synthetic adapters, not live registration authorization.

Prekey publication review confirmed its existing saved-account checks at
reservation, final write and adoption; no new defect was demonstrated there.
This does not establish every prekey consumer or all network ownership paths.
No live traffic/account operation or server acceptance claim.

Verification: **1,595 workspace/all-feature tests passed** after refreshing the
APK coverage report for the shared helper. Strict workspace/all-target/all-feature
Clippy, fmt, CLI reference, OpenAPI library symbols, coverage freshness, staged
diff and repository hygiene passed. The earlier stale-report failure was resolved;
no runtime test failed in that run. These offline results do not prove server
acceptance or complete native retry parity.
