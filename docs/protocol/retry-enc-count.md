# Retry-answer encryption count — 2026-11-04

**Confirmed static:** WhatsApp Android **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The [retained evidence and reproduction ledger](rea/2.26.43.72-retry-enc-count/README.md)
uses freshly resolved current-build classes, not names copied from 2.26.37.71.
No account operation, APK execution or live retry answer occurred.

## Received count reaches the outgoing `enc` unchanged

The complete chain is:

1. `F10.A04(0Tp,Eoh)` reads the receipt's `<retry count>` with
   `A05("count", 1)`; the `EYC` constructor stores that integer in `A00`.
2. Checked current DEX instructions for `160.A04(0Tp,Eoh)` load `EYC.A00`
   at `0084..0088`, preserve it in `v62`, then pass it unchanged into the
   `Fe5` constructor at `0690..06a2`. Its integer field `A00` stores that input.
3. `Fe5.run` loads that field into `v4` at `0018..001c`; no instruction then
   assigns a new value to `v4`. It passes `v4` as the second integer in the
   `SendE2EMessageJob` constructor at `04e0..050c`. That constructor explicitly
   stores its second integer argument in `retryCount`. There is no increment.
4. `FTy.AK7(Ezj)` reads `SendE2EMessageJob.retryCount` and passes it to
   `F3f.A04`. That helper constructs the actual `enc` node and passes the
   count to `F3f.A09`, whose rule is `if (i2 != 0)` then
   `count=String.valueOf(i2)`; zero omits the attribute.

This is an outgoing encryption-child attribute, not a new root-message ID or
an independent local attempt counter. The existing original-send path omits it.

**Implemented/offline:** `transport::retry_answer` now supplies the same received
count when replacing the original direct stanza's encrypted payload, for both
saved-session and fresh-bundle answers. Message ID, unrelated children/attributes,
identity pinning, delivery exclusions, once-per-device/count reservations and
ratchet/outbox transaction order remain unchanged. The replacement helper refuses
anything outside **1 or 2**, including restored-job inputs. Native parsing's upper
bound remains four; support for higher counts is not inferred from that bound.
Regression checks cover values 1/2, original zero-count omission, stale-count
replacement, ciphertext/type and unrelated-field preservation, the reserved
saved-session path, fresh-session path and duplicate/refusal behavior.

## Base-key policy is traced but not ported

**Confirmed static, current build:** `163.A04(EoY,1dW,int,int)` records the
current session's Alice base key on count 2. For counts above 2, it waits for a
fresh prekey if the saved/current base keys match, or for a type-3 address under
its effective property gate. A registration mismatch follows the separate
session-replacement path.

`0WJ.A0s` records `last_alice_base_key` in `message_base_key` for a tuple of
message remote JID, from-me flag, message ID, recipient ID/type and device ID,
alongside a seconds timestamp. It saves the original and address-mapped recipient
forms under locks/transaction. `0WJ.A15` reads the mapped tuple and compares the
saved bytes with the current session's Alice base key; an absent record returns
false. This is not a global retry counter or a raw ciphertext equality check.

**Not implemented:** the Rust session accessor and durable per-message/device
base-key record/alias semantics, their lifecycle, and the effective type-3 gate.
The next bounded port is a validated Alice-base-key accessor plus an atomic
count-2 record keyed by immutable original message/device identity. Admission for
counts 3/4 must wait for that comparison and gate behavior. Existing conservative
refusal of those counts remains. Group/broadcast retries and receipt-carried keys
also remain outside this correction.

## Retry-root normalization remains unresolved

**Confirmed source presence, not complete policy:** current `FTr.ATZ` contains
retry-count branches which remove root `t`, `participant`, `recipient`,
`recipient_pn` and `peer_recipient_username` attributes and adjust addressing.
The complete [REA export](rea/2.26.43.72-retry-enc-count/root-normalization-unresolved.json)
(`ev_16359dda1f71bb61452a0a703d0885d097b0cef825f93cff260cf8e32ea46ef4`)
contains substantial JADX duplicate-block warnings. Its `eiB`/address-classifier
conditions and actual branch gates have not been traced in checked DEX here;
source repetition is not proof of universal attribute removal. No root-normalization
port was made. The Rust count correction preserves the existing root attributes.
A separate follow-up must establish applicable branches and routing producers
before changing that behavior.

## Provider limitations

REA returned no body for the actual DEX-backed `160.A04` and `Fe5.run`; that does
not establish native/abstract implementations. Their complete instruction streams
were obtained through the repository's existing checked `apk-smali` workflow.
REA's much larger send-job `A06` reply was partial after more than 1 MiB of JADX
warning output. Its full partial envelope stays outside Git; no conclusion here
uses a truncated body. The direct producer and writer helpers above are complete.

## Checked root-gate follow-up (2026-11-05)

The [complete current DEX export](apk/2.26.43.72-retry-root-gates/README.md)
confirms map removals are behind multiple gates, including Enu.A0B and further
unresolved state. This does not establish universal direct-retry normalization.
No root attribute changes were made. The separate
[saved-session reservation correction](retry-answers.md#saved-session-retry-reservation-guard-2026-11-05)
rechecks local outbox/peer snapshots and age without expanding supported counts.

Receipt-key support remains unported; the normal direct driver now
[explicitly refuses keys-bearing receipts](retry-answers.md#receipt-carried-keys-capability-boundary-2026-11-05)
without silently substituting saved-session encryption or a directory query.
