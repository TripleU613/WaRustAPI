# Ordinary peer sender-revoke receive window

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
A supported direct peer sender revoke with an authenticated loaded parent and
explicit timestamps now declines its update when the native signed difference
is greater than three days. Its ciphertext/plaintext checkpoint and advanced
receive ratchet still commit; an opaque UnsupportedMessage records the ignored
operation. Parent body/media, receipt state, outbox facts, IDs and waits are not
deleted or rewritten. This is one definite receive rejection, not full native
revocation acceptance or an outgoing delete deadline.

## Native control flow and timestamp provenance

The [eight-topic recipe](apk/recipes/2.26.37.71-sender-revoke-window.json) reproduces
[13 files / 115,209 bytes](apk/2.26.37.71-sender-revoke-window/targets.json), with
full signatures, register flow and exception tables: 153 store, 1gd kind,
Bpt/178 constructors, 0DE namespace predicate, 1iG payment addon, Bua receive
and CoT envelope constructor. Complete independent REA envelopes retain their
Evidence IDs and limits:
[kind](rea/2.26.37.71-sender-revoke-window/revoke-eligibility-kind.json),
[sender constructor](rea/2.26.37.71-sender-revoke-window/revoke-eligibility-sender-constructor.json),
[namespace predicate](rea/2.26.37.71-sender-revoke-window/revoke-eligibility-newsletter.json),
[payment addon lookup](rea/2.26.37.71-sender-revoke-window/revoke-eligibility-payment-classifier.json).
These are static observations, not execution of the APK.

1gd.A1A recognizes message kinds 15 and 64. Bua.CF4 constructs the sender
revoke as Bpt with CoT.A0A and A05. Bpt's direct constructor sets kind 15 and
edit class 7; 178's constructor stores its time argument in A0F at 0096..009a.
The copy constructor instead retains the original message's A0F and stores
revoke time separately in 1ii.A00. Those two timestamp producers must not be
conflated. Existing [wire timestamp evidence](message-timestamps.md) establishes
the root t -> signed millisecond input, including wrapping multiplication and
the native clock fallback which remains separate from historical Rust projection.

153.A00 0640..064a reads incoming A0F minus original A0F using sub-long.
064e..065e compares that signed result with 0x0f731400 = 259,200,000 ms;
strict greater-than sets the old flag. Exact equality is not old. It does not
subtract local receipt time, current wall clock or a profile timezone.

The ordinary received-parent branch at 0680 reaches the payment/old/terminal
checks at 06ae..0722; newsletter type 21 can bypass those drops. 0DE.A0b is
that type-21 predicate, outside the supported User/PN/LID direct route.
06e6..0704 selects revoke-drop-old. A normal own parent bypasses these checks;
admin kind 64 adds a separate own-admin/ownership path. Current held own-device
sender revokes use their existing outgoing-parent path and do not get the peer
window imposed on them. This does not authorize caller-chosen ownership.

Other predicates remain incomplete. 1iG.A00 reads a C1iI addon rather than merely
checking a payment message type; its full producer/lifecycle is not ported.
Terminal classification, orphan processing, author/namespace reconciliation,
admin/group/status admission and receipt/cache/physical deletion still need their
own evidence and state integration. This three-day receiver grace comparison
must not be advertised as permission to queue or send a revoke.

## Projection and recovery

The existing authenticated direct parent selector now also returns the explicit
timestamp from its original route or captured route. Current updates use the
shared signed Java t parser. Missing/malformed time or an unloaded parent stays
unknown; local received_at and a guessed clock do not fill it. A non-rejection
is still only the existing received-action projection, not full native approval.

Known old updates stay opaque, including before media fallback. Their private
inbox marker stores the fixed rejection sender_revoke_too_old. Exact redelivery,
reopen and captured replay cannot upgrade it after later parent metadata changes.
A fresh independently authenticated update ID is evaluated separately. Already
published MessageUpdated markers skip renewed parent/time evaluation, retaining
the original historical event and namespace even if parent metadata later changes.
No journal history is erased and no saved record is automatically migrated.
The incoming-prefix role stays incoming; later own-LID membership does not grant
an old peer action the own-device bypass.

Existing listen/event APIs expose the opaque result or supported update; no new
route, option, event variant or dependency is introduced. All key/account/session
and source/recipient checks remain effective. No read, retry, history fetch or
key adoption is added. Outgoing revoke and physical deletion remain unimplemented.

## Verification

The regression failed before implementation with a peer revoke one second past
the receiver bound still appearing as Revoked. A 36-case encrypted-store matrix
covers decrypted/consumed parents, canonical/attributed legacy layouts, exact and
exceeded boundaries, BMP-decimal input, reverse order, missing/malformed times,
wrapping timestamps and an unrelated local received_at. Parent rows stay equal;
rejected markers remain final after metadata correction, a fresh ID is evaluated,
and existing published history stays unchanged after loading older timestamps.

The [desktop Java arithmetic oracle](SenderRevokeWindowOracle.java) independently
executes the DEX subtraction/comparison for
[18 signed-long vectors](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-sender-revoke-window-oracle.json),
including the one-millisecond boundary and both overflow directions. These are
language-arithmetic observations, not APK execution or full native admission.

The real Signal test authenticates/receives a peer revoke at the boundary,
records a later too-old revoke as opaque, deduplicates both after reopening,
and continues the ratchet. A trusted own-device revoke with explicit times
outside the peer window still follows its separate source/ownership path.
Changed account-LID fixture state does not reopen the old peer rejection.
Original uncertain outbox/idempotency rows remain equal; clearing still uses
existing paths. No live account operation or new network request ran here.

Validation passed 964 workspace/all-feature tests, strict Clippy,
warnings-denied Rustdoc, the Signal regression with only TCP/Noise/Signal
features enabled, 26 default CLI/API-symbol tests and the experimental CLI build.
The eight-topic APK bundle reproduced exactly. Full native revocation policy,
physical effects, missing-time fallback and fresh live interoperability remain
unfinished or unverified.
