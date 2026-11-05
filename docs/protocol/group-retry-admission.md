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

- `answer_one` currently commits a checkpoint labelled `written` **before** its
  socket write. That label is not proof the retry stanza was written or accepted.
- Its retry crypto commit checks the identity pin but not a captured previous
  session, unlike the newly guarded normal direct retry path. Concurrent ratchet
  updates require a follow-up reservation audit.
- The group retry builder does not explicitly apply the received count to an
  outgoing encryption child. Exact current native group-child count/root policy
  needs a trace before changing fan-out bytes.

Full group/status retry eligibility, base-key/receipt-key semantics, successful
write/delivery and restart behavior remain unverified. No ban/acceptance claim
follows from this offline source correction.

Validation: `1314` workspace/all-feature tests, strict workspace/all-target/
all-feature Clippy, formatting and CLI-reference regeneration passed. APK
coverage and repository hygiene checked before push.
