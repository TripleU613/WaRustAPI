# Group reply projection ownership

## Confirmed local gap (2026-11-07)

**Implemented/offline:** generic action settlement gained account binding, but
groups::ingest/apply_pending loaded an ActionOutcome, original decoded request
and saved reply before the final write transaction. ingest_reply_in trusted those
copies rather than reloading action ownership in the transaction that changes
the group snapshot, pending/ingested markers and journal. A regression settled
a group-info reply under registration ID31, changed the saved account to32 and
showed the old projection still committing. Its pre-fix expected refusal failed.

The shared ingest_reply_in now reloads actions::view_in_write after the existing
already-ingested shortcut and before any new projection. The shared view checks
new actions' prepared account authority. It then compares current ActionOutcome,
decoded request and saved reply to the supplied candidate, requires the same
supported group kind and settled Ok/Error status, and refuses changed/missing
inputs before membership or marker updates. This covers public bulk ingestion,
session-pending recovery and their common transactional implementation.

Enumeration now supplies candidates only. A matching ID or an outer success is
not enough to apply a different/newer saved result. The comparison uses decoded
Node equality, not a new binary-wire digest/MAC. No old request is transmitted,
rewritten or granted another callback. Existing once-only consumed rows retain
their shortcut; new account provenance is not manufactured for legacy rows with
no prepared-account field. Account mismatch propagates the shared fixed action
refusal, while changed group projection inputs use the existing group refusal
family. Pending-reply handling keeps its established storage-versus-content
failure distinction.

## Test evidence and related audit scope

**Confirmed source/local:** groups.rs ingest, apply_pending and ingest_reply_in;
actions.rs view_in_write. The regression
group_projection_rechecks_account_and_settlement_inside_commit exercises saved
account drift and a changed settlement sequence after candidate loading. Both
refuse with every checkpoint row and journal head unchanged and no new group
snapshot. Existing successful group-info/invite/leave/refresh, rollback, restart,
ambiguity and consumed-marker tests remain active. This closes a demonstrated
local ownership/concurrency defect; no new APK field or server ban cause is
inferred. Existing native group parser/completeness policy remains unchanged.

A bounded read-consumer review also found abprops::inspect_in feeding the
read-only PQ gate in prekeys/digest inspection, while actual publication uses
the write-side gate through the account-owned action view. That provisional
digest diagnostic is not a network/adoption authority. No new publication defect
was demonstrated by that path and its historical inspection is unchanged.
Other read consumers (peer-key/identity inspection, held fetch and independent
adapters) still need their own final-authority audit; a read result alone does
not prove correct downstream application.

[Generic action account binding and legacy boundary](action-account-ownership.md)
and [network audit](network-correctness-audit.md) retain the wider scope. APK
2.26.43.72 remains pinned for protocol research; this local source defect needed
no new disassembly. Direct offline group-information recording is a separate
explicit adapter, not a saved-action projection covered by this correction.
No live account operation, request, identity reset or server acceptance claim.

Verification: **1,599 workspace/all-feature tests passed**, along with strict
workspace/all-target/all-feature Clippy, fmt, CLI reference, OpenAPI library
symbols and APK coverage freshness. Staged diff/repository hygiene are checked
before commit. These offline checks do not prove server acceptance.
