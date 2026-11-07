# Device key-batch account ownership

## Reproduced cross-account settlement (2026-11-07)

**Implemented/offline:** the paginated directory-key cache used by group staging
froze requested devices and query IDs but not its saved account. A regression
reserved the first page under registration ID31, replaced the saved account with32
and showed a valid matching page still being accepted. The pre-fix expected-error
assertion failed. A frozen device plan and validated signatures alone did not
establish local account ownership for settlement or reuse.

New Batch records retain optional internal account_authority captured with the
first page's reservation. The shared transport fingerprint binds store/account
generation, phone, own JID, registration status/ID and identity bytes or explicit
absence. Only the digest is retained privately; it is not a new directory IQ
field, device identity, signature or MAC against rewriting local metadata.

reserve_next compares current authority before selecting another page or
reporting completion. settle first identifies an uncertain matching page, then
checks authority in the same write transaction before bundle validation or status/reply
mutation. Duplicate/unmatched/nonreply inputs retain their existing no-mutation
behavior. assembled checks authority in the same immutable snapshot as frozen
targets and answered-page validation before returning a combined private key
result. Read-only summary remains available under changed ownership so outcomes
can still be inspected.

Account replacement/removal or store-generation change refuses with the fixed
local `key-batch account changed` error. No refused page is converted to a server
error and no uncertain reservation becomes Ready again. Restoring identical
fixture authority demonstrates comparison, not a new reset/recovery operation.
Historical Batch records without the optional field retain unknown preparation
provenance and existing behavior; the missing value is not backfilled or called
proof of a matching account. New account-less offline batches bind absence.

## Regression and source scope

**Confirmed source/local:** device_key_batches.rs Batch, reserve_next, settle
and assembled. The test
key_batch_reservation_settlement_and_assembly_keep_the_original_account uses a
real two-page plan and valid signed fixture bundles. It checks changed-account
settlement/continuation refusal with unchanged persisted state, normal owner
completion, changed-account assembly refusal, diagnostic availability, refusal
across reopen and no new query after complete original-owner assembly.

This closes the new key-batch lifecycle's local account gap. It does not by
itself bind an earlier group Pending/Row to the first batch creation, protect all
read-to-crypto gaps, or establish account ownership in group sender-key commit.
Those higher-level transitions are now addressed in the subsequent
[group preparation account audit](group-preparation-account-ownership.md), within
its documented limits. Existing normal
[group writer checks](group-writer-account-ownership.md) and [group projection](group-projection-ownership.md)
remain separate safeguards. Non-group/offline consumers do not gain permission
to replay queries or override account/session state from this change.

No new APK export was required for this local defect; existing native directory
bundle validation and pinned APK2.26.43.72 evidence remain unchanged. No traffic,
account operation, identity rotation or server restriction/acceptance claim.
The wider network audit remains unfinished.

Verification: **1,605 workspace/all-feature tests passed**, plus strict
workspace/all-target/all-feature Clippy. Formatting, CLI/OpenAPI, APK coverage
freshness and staged repository hygiene are checked before commit.
