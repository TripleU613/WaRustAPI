# Identity refresh preparation-account deferral

## Source-traced recovery gap (2026-11-07)

**Implemented/offline:** new action ownership prevents identity-refresh commit
under a different preparation account. However, stage previously used the
current saved account without checking the action's preparation owner. A change
before staging could therefore produce a candidate under the replacement account;
the final owned action view would refuse it as an error. recover propagates
application errors, so an old held row could interrupt later recovery rather
than retain the feature's established deferred behavior.

The regression changed saved Signal registration ID after valid reply settlement
and asserted a finished Deferred(action_account_changed) preparation. This failed
before the change. The source chain above identifies the later rejection point;
the pre-fix assertion did not separately measure a live connection termination.
No server restriction cause or runtime APK behavior is inferred.

## Same immutable snapshot, final fence retained

Actions now provides prepared_account_matches_in: a read-only tri-state check
of the stored preparation digest against current account/store authority.
Some(true/false) identifies matching/mismatching prospective metadata; None is
absent legacy metadata or missing row, never newly invented proof. No digest,
identity bytes or account identifiers are returned by this helper.

Identity-refresh stage captures this result in the same ReadTx as its request,
reply, peer pin/session and account. An unapplied action under ApkAuthenticated
policy with Some(false) returns the fixed deferred reason before session
candidate derivation. Pinned policy and already-applied shortcuts retain their
existing outcomes. Unknown legacy preparation retains its prior admission
checks. Historical inspection/preview remains diagnostic; this change does not
turn it into key-adoption authority.

Final commit still checks staged account, request/source, peer state and the
account-owned action view in its write transaction. A change after staging keeps
the prior account_changed_before_commit deferral. No ownership check is removed,
no stale action is reset and no query, resend or callback is resurrected.

## Validation and scope

**Confirmed source/local:** actions.rs prepared_account_matches_in and
identity_notifications/application.rs stage/apply/recover/commit. The new
changed_preparation_account_defers_refresh_recovery_without_mutation regression
requires the exact finished deferral, zero applications from recover and unchanged
all-table fixture snapshot plus journal head. Existing stage/commit drift,
identity policy, signed bundle and atomic invalidation tests remain active.

This fixes deferred handling around a demonstrated local admission gap; it adds
no identity-refresh wire field or new key adoption. APK2.26.43.72 remains pinned
for protocol research, with existing native/session evidence unchanged. The
local source issue needed no new disassembly. [Generic action account ownership](action-account-ownership.md)
and [retry refetch ownership](retry-refetch-account-ownership.md) retain the
separate write/callback boundaries. Other read-side feature consumers remain
individual audit work. No live account operation, traffic or server acceptance
claim.

Verification: **1,600 workspace/all-feature tests passed**, plus strict
workspace/all-target/all-feature Clippy, fmt, CLI reference, OpenAPI library
symbols and APK coverage freshness. Staged diff/repository hygiene are checked
before commit. These offline checks do not establish server acceptance.
