# Generic action account ownership

## Confirmed local defect and correction (2026-11-07)

**Implemented/offline:** a queued action retained wire and lifecycle state but
no account at preparation. A regression queued under saved registration ID31,
changed it to32 and showed reserve_next_owned accepting the old action. The
pre-fix expected-refusal assertion failed. Exact action row matching alone did
not establish account ownership after queue handoff or for a delayed reply.

New rows store optional internal prepared_account_authority. The fingerprint
uses transport/account_authority's store/account generation, phone, own JID,
registration state/ID and identity bytes or explicit missing-account state.
Raw identity data is only hash input; no new wire or public inspection field.
Observation timestamps are excluded. A digest is consistency metadata, not
proof against rewriting local row and digest together.

Public queue captures authority **before** calling its builder, then checks in
the write transaction before saving a row. A builder can still perform its
established independent store operations; if it changes the account, queue
refuses the action rather than assigning the new account to old preparation.
It does not undo account changes already committed by the external builder.
queue_in captures/checks around its builder within the feature transaction.

Both batch and feature-owned single-action reservations check prepared authority
before deferred-state preparation, held-fetch/patch reservations or Sent status.
A mismatch fails that transaction without reserving old work. The normal
connection's ActionWriteGuard also freezes current authority at reservation and
rechecks it at the existing final local write boundary, alongside row bytes,
restrictions and deadline. The returned-node/offline adapters do not acquire a
new remote-write lock or bypass feature-owned write guards.

## Reply settlement and transactional projection

record_reply_inner checks new rows' prepared authority before reply/sequence or
feature outcome mutation. A live owner additionally checks its frozen reserved
account even for a historical row without preparation metadata. LiveReplies still
consumes its callback before settlement: refusal leaves uncertainty and never
resurrects a callback. A captured-reply reconciliation call cannot bypass the
new prepared-row check merely by omitting a live guard.

view_in_write also checks prepared ownership before returning data to a feature's
atomic projection. Thus an account change after reply settlement does not make
its saved result authoritative for another account's transactional projection.
Read-only historical inspection remains possible. This is not an assertion that
every independently written downstream adapter uses view_in_write; those paths
still need individual auditing.

## Historical boundary and validation

Legacy missing prepared_account_authority is unknown and not backfilled. Its
existing reservation behavior remains; a new live reservation still binds its
current account at handoff. No old row is retroactively declared prepared under
that account. No action is automatically regenerated, retried, reset or moved to
another account. Restoration in fixtures tests equality only, not a new recovery
command. Historical/feature-owned offline adapters and projection consumers
remain separate audit obligations.

**Confirmed source/local:** actions.rs queue/queue_in/queue_prepared_in,
reserve_limit_owned/reserve_one_in, ActionWriteGuard, record_reply_inner and
view_in_write. Existing native exact IQ callback/disconnect and feature response
policies remain unchanged; see [action response and timing boundaries](network-correctness-audit.md).
This local ownership defect needed no new APK disassembly; APK2.26.43.72 remains
pinned for protocol comparisons. No server behavior/ban cause is inferred.

Tests: action_account_owner_survives_queue_handoff_and_live_reply checks changed
account refusal at reservation, write admission, live/captured reply settlement
and transactional view; all checkpoint rows remain unchanged and live callback
ownership is consumed. action_builder_cannot_adopt_a_replacement_account proves
account creation inside a public builder cannot bless a queued row.
legacy_action_preparation_stays_unknown_but_reserved_guard_binds_current_owner
proves optional-field compatibility, no backfill and reserved-guard refusal after
account removal. Existing independent device/refetch account regressions remain.
No live account operation, request or server acceptance claim.

Integration checks initially exposed two fixtures queuing before saving their
account (identity session preview and PQ upload-construction setup). They now
save the intended owner before queueing. The identity-refresh commit path also
checks its existing staged-account snapshot before the generic action view so
account drift keeps its established Deferred outcome, with no mutation. These
corrections preserve the new ownership checks rather than relaxing them.

Final verification: **1,598 workspace/all-feature tests passed** after the
integration corrections. Strict workspace/all-target/all-feature Clippy, fmt,
CLI reference, OpenAPI library symbols, APK coverage freshness, staged diff and
repository hygiene passed. No live/account traffic was used for these checks.
