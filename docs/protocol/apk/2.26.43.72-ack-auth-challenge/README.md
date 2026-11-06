# ACK auth-challenge diversion — APK2.26.43.72

**Confirmed static**, checked DEX text, not REA Evidence envelopes or observed
runtime. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
the pinned artifact used in preceding ACK traces. Commands: `xtask apk-smali
APK --class CLASS --method METHOD --signatures --exceptions`.

## Special numeric459 precedes ordinary ACK dispatch

The retained [1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt)
reads error through0Tp.A05(error,0) at00a2..00ac, compares it with459 and
calls1lZ.A1F then returns at00ba..00c6. It does this before switching on
ACK class, so459 is not an ordinary message failure result.0Tp.A05 returns
its default when absent; A06 calls Integer.parseInt and translates parse
failure to LIL. Java decimal spelling, not raw string equality, controls459.

1lZ.A1F logs onAuthChallengeReceived and delegates1lW.BkU.1lY.BkU resolves
0ln and calls A0A, which schedules handler message7. The retained
[0m2.handleMessage](../2.26.43.72-distribution-ack-admission/handler-callback.txt)
case7 at01ea..021c uses Eoh.A08 to call0TV.A01, then puts a returned Eo7
into0TV.A04 at07e6..07ee. A01 removes the ID from A05, optionally schedules
EV7-related work under config0x3b15, and returns the removed record.

This selected route transfers an existing request between maps. It neither
invokes ordinary ACK success/failure callbacks nor establishes account-wide
closure, automatic retry, challenge completion or a ban. The exact producers
and recovery consumers of0TV.A04/A05 and optional cleanup remain untraced.
Ordinary message callbacks use the separate0Th map; this route does not
establish that every challenged message has an Eo7 record in0TV.

## Confirmed local mismatch and bounded fix

outgoing::acknowledge treated every error-present matching message ACK as
Failed, including459. outgoing_group's distribution waiter similarly treated
459 as DistributionRefused. That ordinary terminal settlement is not the
selected native dispatch. Both now exclude numeric459 from ordinary ACK
settlement using the existing Java-decimal parser plus i32 range checking.
Neither helper marks recipients accepted or releases the original reservation.

The direct regression fails on old code and passes after the guard. It tests
459, +459, zero-padded and full-width decimal spellings against a real in-flight
reservation, comparing the entire outbox row and checking it cannot be
reselected. Existing reopen/normal ACK acceptance checks then still pass.
The real-crypto group reaction test likewise rejects these spellings while
waiting for a distribution ACK; a later ordinary message ACK completes the
existing crypto/reopen flow. Existing non459 refusal behavior remains.

The actor already preserves raw received nodes before these helpers. A challenged
group waiter can later expire to its existing blocked/outcome_unknown policy;
no challenge-specific replay is added. This is not a full native challenge
controller or an account-wide stop policy. Other comparison/lab send loops
remain separate audit work. No new command/route or live account operation.

## Checksums

- [bridge.txt](bridge.txt): `e8ecbcf4053a7c08665288e5f300bfd449bcd1470bc59188aa7a7e36707980e5`
- [dispatch.txt](dispatch.txt): `bb60dbf1c60d424b7283aaa3987b0be0be508b926f9f5a51b693200a49bc4db4`
- [integer-parse.txt](integer-parse.txt): `ff0f1f7c71b9a6726dff0257b0cdf58a9bb0554480a8b439e943143007451445`
- [integer.txt](integer.txt): `71844beba60f9b9c35aa7c483d0f497598195ff5f6b14728e5f5117cdb72db35`
- [lookup.txt](lookup.txt): `298ec9ae0c5d136625ceec4319bf8600cd2dec77e999977f5391353488b337be`
- [manager.txt](manager.txt): `04ed8b379869e4e4ee8d40b1ebd13ff809ede5ebe5aba8d566a48a956a7ee648`

## Receipt-class coverage follow-up (2026-11-06)

The pre-class459 branch also excludes ordinary receipt ACK settlement. Rust's
receipt helper had remained error-presence based after the message/distribution
fix above. It now shares the same challenge classifier and leaves matching
single/batched reservations held. A failing-before/passing-after regression
covers direct/group and Java decimal spellings, reopen, journal preservation
and later normal success/refusal controls.
[Receipt interpretation and remaining gaps](../../receipt-ack-special.md#current-auth-challenge-diversion-precedes-receipt-settlement-2026-11-06).
No new static trace or challenge recovery capability is claimed.

## Integral admission and selected catch follow-up (2026-11-06)

The retained A05/A06 parser also excludes malformed/out-of-i32 errors from
ordinary ACK callbacks. Shared Rust classification/direct/distribution/receipt
settlement now preserves uncertainty for these values.
[Reader-call/catch offsets and regression scope](../../message-ack.md#integral-error-admission-before-ordinary-ack-outcomes-2026-11-06)
interpret the existing1s2.run export's selected LIL catch to1rg.A04, without
claiming complete error traffic or connection recovery. No new static export
or target execution.
