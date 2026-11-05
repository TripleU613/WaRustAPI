# One-shot action reply ownership across connections

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
This implementation follow-up reuses full checked-DEX exports, not new REA
Evidence envelopes or target/runtime observations:

- [IQ promotion and selected sent-map shutdown flush](../2.26.43.72-iq-promotion-shutdown/README.md):0ln.A0C moves queuedA0I callback to sentA0H before node writing; handler case3 snapshots and clearsA0H before submitting failure callbacks.
- [Response lookup/claim/parser ownership](../2.26.43.72-iq-response-ownership/README.md):A0H GET/timer cancellation is separate fromA02 REMOVE; only the successful remover schedules the callback parser. Duplicate response tasks do not each own it.
- [Ordinary disconnect eligibility and queued-map limits](../2.26.43.72-iq-shutdown-eligibility/README.md): selected0YB.A05 callsA0F with the original reason and channel-derived boolean. The posted handler's sent-map cleanup is not a universal queued-request flush or proof of every lifecycle branch.

## Rust gap and change

Normal ActionDrain reserves a queued action durably as Sent before local write.
Previously normal receive called actions::record_reply by ID against those rows
without requiring a live callback owner. A late result after a failed write/
reconnect could therefore settle an older held action and enter feature ingestion,
even though no callback had been established for it on the new connection.
Durable reservation and active request ownership were conflated.

**Inferred compatibility ownership / implemented offline:** private actions::LiveReplies
is fresh for each drive invocation. The normal drain registers the exact IQ ID
only after link.send succeeds locally. Drive processes no inbound stanza while
awaiting that write, so queued replies are considered after registration. Exact
iq result/error claims and removes ownership before invoking durable record_reply;
wrong kind/type/ID does not consume it. Duplicate replies and local parser/store
failure cannot obtain a second callback claim. A new drive or reopened process
never repopulates claims from persisted Sent rows or a reused numeric epoch.
No status resets, replay, identity/key rotation or new automatic request is added.

The public record_reply API retains its behavior for a caller with request-bound
callback or captured-reply reconciliation authority, now explicitly documented.
Offline settle_undecodable_replies remains separate and unchanged. Group membership
preparation has its own live Pending/deadline owner and still settles before the
generic action handler; this change does not replace those specialized owners.
Incoming captures remain preserved even when no action callback is claimed.

This does not equate the APK's pre-write promotion with successful wire/server
acceptance. Rust post-local-write registration is a local loop boundary, not a
new delivery guarantee. Individual action deadlines, every native callback marker,
upstream address admission, queuedA0I survival and all disconnect races are not
fully ported by this change. Refusals/uncertain sends remain held.

## Durable request identity is checked as well

Queue construction already required an IQ carrying its assigned ID, but both
reservation paths previously decoded stored wire without rechecking that invariant.
A damaged/misassigned row could return another request ID, allowing the live set
to register the wrong request. Private decode_owned now validates32-hex row ID,
IQ tag and exact saved-wire ID at bulk/single/feature-owned reservation and again
before settlement. Invalid stored data is a local integrity fault, not a content
refusal or server error; transactions leave rows and reservation state unchanged.
This is the existing Rust queue identity contract, not a claim about the APK's
private durable store representation.

## Offline regression

cargo test --locked -p whatsapp-api --all-features
live_reply_claims_do_not_resurrect_from_durable_rows verifies result/error, no
ownership from reservation alone, wrong type/kind/ID, duplicate, disk reopen and
local transactional fault. A failed settlement consumes the live claim while
leaving durable Sent uncertainty intact; explicit public reconciliation is not
silently retried.

paced_actions_keep_receive_and_stop_responsive_and_reserve_only_one exercises
the production loop: after first action's failed write and reconnect, an old
same-ID result is followed by an unrelated receipt as a receive barrier. Old action
stays Sent, next action sent on the new link can settle Ok, and reserved actions
are not retransmitted. Existing feature projection tests cover ordinary live
settlement. No APK execution, account traffic or server acceptance is claimed.

The production regression also queues a fast response before local write completion,
then releases the successful write and verifies normal settlement. The separate
saved_wire_identity_mismatch_never_reserves_or_settles test covers mismatched ID,
non-IQ tag and missing ID, all three reservation APIs and settlement, unchanged
stored rows and fixed diagnostics without either request ID.

Validation:1334 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.
