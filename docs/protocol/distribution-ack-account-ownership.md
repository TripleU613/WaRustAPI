# Sender-key distribution ACK account ownership

## Confirmed source and offline defect (2026-11-07)

PendingAck previously bound chat, own JID, chain and recipient devices but not
the account at reservation. A regression reserved a distribution under saved
registration ID31, changed it to32 and demonstrated that confirm_ack_in still
consumed the record and marked the device delivered. The pre-fix refusal assertion
failed. Chain equality alone does not establish the original account owner.

New PendingAck records retain an optional private account_authority digest from
the shared transport fingerprint, captured in pending_ack_in's write transaction.
confirm_ack_in checks that authority before deleting the pending record or
changing distribution state. Explicit mismatch returns the local error
`distribution ACK account changed`. Account-less fixture reservations bind absence.
No record remains a no-op, and rotated/replaced chains retain existing semantics.

The ordinary outgoing::acknowledge success branch calls confirm_ack_in inside
the same write transaction as outbox facts. Refusal therefore prevents that ACK
from creating a successful delivery fact or scheduling dependent token work.
The connection propagates this local storage/ownership error visibly; it does
not retry, clear the pending record or invent a server refusal. The integration
regression exercises the real outgoing group ACK route, unchanged outbox/journal
and sender-key state, then original-owner once-only completion.

## Scope and unknowns

**Unknown legacy provenance:** serde accepts old pending records without the
field and preserves their existing behavior. Missing ownership is not backfilled
or presented as confirmed. Direct sends without a distribution PendingAck and
ordinary failed message ACKs do not gain account binding from this change.
Normal outbox reservation/ACK binding is subsequently addressed in the
[ordinary message ACK audit](message-ack-account-ownership.md), with explicit
legacy and adapter limits.
Separate parent/child transactions and old distribution-state reuse likewise
require their own audit; this does not establish full network parity.

**Existing APK evidence:** sender_key_distribution.rs records the APK37.71
SendE2EMessageJob.A06 / 0bA after-ACK sent_sender_key behavior. No new APK export
was used for this Rust-only ownership correction; pinned APK2.26.43.72 evidence
is unchanged. This is not a newly discovered server requirement or proof of
server acceptance. No live traffic, identity reset, query replay or account
operation was performed. Zero bans remains unproven.

Regressions cover refusal before consumption, unchanged checkpoint/journal data,
refusal across reopen, and original-owner once-only completion. Verification: **1,609 workspace/all-feature tests passed**, plus strict
all-target/all-feature Clippy, formatting, CLI/OpenAPI checks, APK coverage
freshness and staged repository hygiene.
