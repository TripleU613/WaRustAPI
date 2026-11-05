# Group comparison retry admission — 2026-11-05

**Confirmed source correction / offline tested.** This concerns the explicit
CLI group/status comparison send window in `transport_group_send.rs`, not a new
normal-runtime group retry implementation. No live receipt, key request or resend
was used.

The window captured incoming frames and correlated receipts, then parsed retry
requests. It checked the original message ID, answered-device list and original
recipient devices, but did not check count bounds, carried keys or group-chat
identity before marking a device answered and calling `answer_one`. That helper
fetches keys before re-encrypting the distribution. Those missing guards could
therefore cause unnecessary directory queries and unsupported retry attempts.

The shared admission helper now runs **before** answered-device reservation or
`answer_one` and requires:

- The original wire message ID, and the same chat when the original is a group.
- Count1 or2; zero/above4 are out of range,3/4 explicitly need unported base-key
  policy and are a **local capability refusal**, not an APK maximum of2.
- No top-level receipt `keys` child, whose verification/session policy is unported.
- An original device that has not already been attempted in this window.

A refusal logs a stable local reason with no identifier/key contents. The frame
has already been captured; it is not marked answered and makes no key query or
resend through this branch. A later supported receipt may still qualify.
Status addressing is not subjected to a newly invented group-chat equality rule;
its complete retry routing remains open. The count/key limits apply to this
shared comparison window, including status sends.

The retained [native retry policy](retry-answers.md#apk-answer-policy-confirmed-static-2263771-2026-11-01)
excludes non-original recipients and limits native retry count to4. The current
[base-key policy](retry-enc-count.md#base-key-policy-is-traced-but-not-ported) and
[receipt-key policy](apk/2.26.43.72-retry-receipt-keys/README.md) are not fully
ported. Group-chat equality is our correlation guard for the original send, not
a claim about every Android receipt-address alias. These guards align explicit
capability boundaries with the normal direct driver's exclusions without
pretending full native group-session behavior is implemented.

A14-case deterministic regression exercises accepted counts1/2, different
message/group/direct-chat context, non-recipient, zero/3/4/5/signed-max counts,
carried keys and duplicate device. It invokes the same pure admission helper
used before network work and verifies the recipient/answered lists are unchanged.
It does not execute the receive loop, filesystem, socket or cryptography.

## Remaining comparison retry issues

The audit also identified separate source boundaries still requiring correction
or proof; this admission change does not close them:

- The prior checkpoint label and missing previous-session guard are corrected
  in the peer snapshot/write-settlement follow-up below. Historical `written`
  checkpoints are not retroactive proof of a successful socket write.
- The group retry builder does not explicitly apply the received count to an
  outgoing encryption child. Exact current native group-child count/root policy
  needs a trace before changing fan-out bytes.

Full group/status retry eligibility, base-key/receipt-key semantics, successful
write/delivery and restart behavior remain unverified. No ban/acceptance claim
follows from this offline source correction.

Validation: `1314` workspace/all-feature tests, strict workspace/all-target/
all-feature Clippy, formatting and CLI-reference regeneration passed. APK
coverage and repository hygiene checked before push.

## Peer snapshot and write settlement correction (2026-11-05)

**Confirmed local correction / implemented offline:** group/status comparison
retry preparation now captures identity and session together in one read snapshot.
Its write transaction requires both to remain unchanged and refuses an existing
retry checkpoint before saving the prepared session/pin. A concurrent ratchet
advance, session appearance/removal or pin change therefore preserves newer
state and creates no replacement reservation. This is local concurrency control,
not a newly inferred native reset/fetch policy.

The prepared peer state and checkpoint commit atomically **before** socket I/O.
The checkpoint is now **reserved**, with the key-query ID as reservation token,
received count and reservation timestamp. A successful `write_node` is followed
by owned settlement to **written**; an error settles to **uncertain**. Settlement
requires the same token/count and still-reserved status. Duplicate/stale settlement
cannot overwrite another state. Existing checkpoints, including legacy written
records, are not re-reserved or migrated into a resend.

A failed/ambiguous write does not roll back the advanced session and is not
retried. A crash after reservation leaves reserved state, also not replayable.
If socket I/O succeeds but settlement fails, the reservation remains held and an
explicit diagnostic reports the socket outcome without claiming a settled send.
The caller no longer claims the recipient can read the message: written proves
only the local write operation completed, not server ACK, delivery or decryption.

Two deterministic store regressions exercise the actual reservation/settlement
helpers: unchanged input; changed identity/session; absent→present and removed
session; existing legacy checkpoint; interruption/reopen after reservation;
stale token/count settlement; duplicate reservation/settlement; successful and
uncertain outcomes across reopen. Assertions preserve all checked tables on
refusal and preserve advanced peer state after either outcome. Socket outcomes
are supplied to the settlement helper; no real socket, process crash, concurrent
thread, crypto or injected storage fault is executed by these tests.

The group retry encryption-child count/root policy, status addressing and full
native base-key/key handling remain open. This fixes the two source issues above
without claiming full group/status retry parity or live acceptance.

Follow-up validation:1316 workspace/all-feature tests passed, along with strict
workspace/all-target/all-feature Clippy, formatting and CLI-reference regeneration.
Coverage/hygiene checked before push; no live socket outcome was measured.
