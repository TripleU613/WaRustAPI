# Trusted-contact token schedule account ownership

## Confirmed local defect (2026-11-07)

The trusted-contact send cache was indexed by peer alone. Generic action ownership
already prevented an old queued action from being reserved under a replacement
account, but schedule_after_ack_in returned early for a recent cached timestamp
before inspecting the action. An offline regression saved a schedule under
registration ID31, changed the saved account to32, and demonstrated silent
suppression as if the old cache were current. The pre-fix refusal assertion failed.

New private SendState rows retain optional shared account/store authority.
Scheduling checks existing authority before the recent-timestamp early return,
legacy owner lookup, action reuse or any new schedule. prepare_in checks before
expiry/defer decisions or owner repair. record_success_in checks an existing
row before timestamp mutation; a newly inserted completion row captures current
validated account authority. These checks share their caller's write transaction.
Generic action reservation, writer and reply account checks remain additional
safeguards, not replaced by the cache check.

A mismatch returns the local `token schedule account changed` error. Neither old
state nor queued actions are deleted, re-owned or replayed. A new schedule or
inserted completion binds its current owner; historical rows without metadata
remain unknown and preserve their existing behavior. An explicit new schedule
from a legacy row is not retroactive proof of historical ownership. Account-less
offline fixtures bind absence. Digests are private local provenance, not a new IQ
field or protection against malicious local store rewriting.

## Verification and scope

The regression checks unchanged checkpoint rows and journal after refusal,
refusal across reopen, normal recent-state suppression for the original fixture
owner, then reservation and successful once-only completion of the original
action. The original-account restoration is a test fixture operation, not a new
reset/recovery API. No live account operation or network request was performed.

**Existing static evidence:** [token-issue producer research](token-issue.md)
records native per-user scheduled timestamp handling and fallback policy in
APK37.71 and43.72. No new APK extraction was needed for this Rust cache-ownership
defect. Configured bucket properties, producer exclusions and complete clock
parity remain unported; this fix does not fill them or establish server acceptance.
The wider inbound/outbound protocol audit and zero-ban aspiration remain unproven.

Verification: **1,611 workspace/all-feature tests passed**, plus strict
all-target/all-feature Clippy, formatting, CLI/OpenAPI checks, APK coverage
freshness and staged repository hygiene.
