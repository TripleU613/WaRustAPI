# Hybrid bundles in the normal outbox

2026-10-12. Correlated primary-peer key replies containing a complete signed
ML-KEM-1024 bundle can now reach the normal outgoing coordinator. Fresh peers
use native version-4 bootstrap; held version-4 sessions continue without reset.
The normal scheduler now accepts both PN and LID primary recipients.

## Evidence and behavior

This integration reuses the retained [bundle verification](hybrid-bundle.md),
[native session](hybrid-session.md), [wire encoding](signal-wire.md) and
[held shared send](held-hybrid-send.md) evidence. No new native behavior is
inferred from this local integration. APK Bot.A03/0jc.A0L/CQc.A00 supply the
selected material, raw signatures, ordered derivation and pending KEM fields.

The connection actor remains responsible for authenticated Noise delivery.
Pending::prepare_with_policy requires the exact reserved query ID and exact
recipient before selecting either classic or hybrid material. A hybrid reply
verifies both signatures and the public encoding before any candidate exists.
The existing pin, stale-reply guard, own-identity exclusion and policy govern
replacement; no signature alone grants a changed identity permission.

The primary hybrid branch currently refuses own-account peers, companions and
replies carrying non_e2ee_attestation or identity_auth. These are explicit local
coverage limits, not claims that the APK refuses those replies. Their admission
and refresh paths remain separate work. The public classic-only parser continues
to refuse PQ material; the coordinator explicitly uses the hybrid parser.

A fresh candidate consumes the selected signed/optional ordinary/KEM material.
A held hybrid candidate uses the shared account/registration/pin/ratchet guards
and retains its own pending KEM fields, rather than adopting the newly fetched
KEM ciphertext. A same-key held classic record refuses migration: archival and
classic-to-hybrid refresh require a separate implementation. A permitted changed
identity derives fresh state and archives the original pin/session using the
existing identity-transition transaction.

Text encoding is shared with the existing direct API. Polls, replies, reactions,
chat-setting and media payloads use the same coordinator preparation selection;
existing content validation and envelopes are unchanged. No new command, request
ID, retry policy or server wait is introduced.

The existing transaction checks the original intent, pin, session and key-query
reservation, then commits the candidate, original client request ID, one wire ID,
one attempt and DeliveryUnknown before any socket write. Concurrent state changes
refuse; uncertain attempts cannot be replayed automatically.

## Tests and remaining work

Actual synthetic PN/LID bundle signatures and KEM decapsulation establish a Bob
state that decrypts the coordinator's fresh encrypted text. Pinned changed keys
leave the intent queued with no attempt/wire ID; APK-authenticated changed keys
produce a decryptable fresh candidate. Account rows remain unchanged. Reopening
retains the committed session and original attempt and refuses replay.

Own identity (including the legacy own-LID checkpoint), attestation/auth metadata,
a response for another device and valid held
classic migration refuse. Their complete store snapshots and journal heads stay
unchanged. The earlier held-session encrypted test now supplies a real signed PQ
reply and confirms the original pending KEM ciphertext survives continuation.
Malformed/missing PQ material continues to leave saved state unchanged.

This is offline coordinator validation; there was no live directory or socket
operation. Incoming KEM inventory, upload, lookup/consumption and bootstrap,
companion authority, attestation admission, same-key classic migration and live
Android/server interoperability remain incomplete. Ban immunity is not established.

Validation passed 978 workspace/all-feature tests, both crypto-only hybrid
fixtures, strict all-target Clippy, warnings-denied Rustdoc, 26 default CLI/API
symbol tests, experimental CLI build and formatting. No live test was performed.
