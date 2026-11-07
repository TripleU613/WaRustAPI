# Ordinary message ACK account ownership

## Confirmed local defect (2026-11-07)

A real direct-send regression prepared ciphertext from a valid signed fixture
bundle through begin_selected and Pending.prepare. After saved registration ID31
was replaced by32, both a successful matching message ACK and a matching error500
ACK could settle the original outbox entry. Separate pre-fix runs confirmed each
refusal assertion failed. Existing ID/chat/type/error grammar checks established
correlation, but did not establish the reservation's account ownership.

## Implemented local ownership

Normal direct and group final ciphertext reservation now calls
outgoing::reserve_ack_owner_in in the same transaction that writes sessions,
wire ID, attempt and InFlight state. The private checkpoint is indexed by local
message ID and retains the exact wire ID plus shared account/store authority
digest. Existing owner records cannot be overwritten by another reservation.
An account-less offline reservation binds explicit absence.

acknowledge checks the original owner inside its write transaction after unique
wire lookup, chat/progress and already-terminal checks, before either success or
failure mutation. Changed account or wire authority returns the fixed local
`message ACK reservation account changed` error. This precedes Failed progress,
sender-key delivered-device adoption, token scheduling and the delivery fact.
Refusal leaves the durable reservation uncertain and does not send anything.
Duplicate terminal ACKs retain their existing no-op behavior. The original
checkpoint remains historical provenance; no new retransmission path is added.

**Unknown legacy provenance:** older/manual InFlight entries without this private
checkpoint preserve existing ACK behavior. Absence is not backfilled from the
replying account or called ownership proof. Directly constructed/offline sends
outside normal outbox reservation are not automatically bound. Transport identity,
account replacement procedures, receipt facts and all other adapters still require
their own audit. This change does not establish full protocol parity.

## Evidence and verification

**Confirmed source/offline:** outgoing.rs final direct transaction and acknowledge;
outgoing_group.rs final group reservation. The real encrypted direct regression
covers success and failure, unchanged checkpoints/outbox/journal on refusal,
reopen refusal, no fresh query, and original-owner once-only completion. The
group ACK integration fixture now reserves the same owner and checks rejection
of both successful and failed ACKs before normal original-owner completion.

No new APK export was needed for this Rust ownership defect. Existing
[message ACK grammar](message-ack.md), pinned APK2.26.43.72 evidence and
[distribution-specific ACK ownership](distribution-ack-account-ownership.md)
remain separate evidence. No new wire field, invented server rule, identity reset,
replay or live account operation. Server acceptance and zero bans are unproven.

Verification: **1,610 workspace/all-feature tests passed**, plus the expanded
group ACK regression, strict all-target/all-feature Clippy, formatting, CLI/OpenAPI
checks, APK coverage freshness and staged repository hygiene.
