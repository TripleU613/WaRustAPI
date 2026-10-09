# Deferred blank-device receipt expansion

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-blank-receipt-deferred.json),
[complete signatures, operands and catches](apk/2.26.37.71-blank-receipt-deferred/targets.json).
This closes the DNt discriminator-6 boundary in
[blank receipt entry](message-status.md#normal-blank-receipt-entry-is-a-device-path).
Offsets are method-relative bytes; unrelated worker cases remain uninterpreted.

## Caller, saved arguments and target difference

**Confirmed:** the retained DGG.Bqh uses this branch only for 1im with a
nonempty pre-populated receipt cache, after its target and message guards.
Ayk.A1X(key), key.A02 and message.A0y must all be true before scheduling
DNt(targets,processor,message,**6**) through 1f0.A02 code **74** (00b2..00f8).
Those predicates are not replaced by a Rust original-target snapshot.
Full Ayk helper and field authority remain unknown; see the prior source.

DNt's Object/Object/Object/I constructor stores its second argument in A00,
third in A01, first in A02 and selector in $t. The separate 14C/1B3/Cb1 overload
has a different field order and is not this caller. Run case 6 begins at **1586**,
restoring DGG/message/target collection accordingly. It obtains 0zt and calls
6d6.A01(message), then uses returned 8Bw.A03 mode and A0H Set in
**0zt.A0U(set,mode)** (15ae..15c8). It copies that result through 0C6.A1O,
removes the originally supplied target collection, then removes
**0Z4.A02(message)**'s existing targets. Finally it calls
**0Z4.A08(message,remainingTargets)** (1606..160c) and discards the Boolean.
The worker does not apply DGG.Bqh's immediate false-result status-20 fallback.
No retry or server wait is established; transitive scheduling/failure handling
remains **unknown**.

6d6.A01 obtains context through A02; when absent, it constructs 8Bw and attaches
through A03, then rereads. A still-null reread throws IllegalStateException.
Full attachment/defaults/field producer semantics remain **unknown**.
8Bw.A03 returns **3** when A0N is true, otherwise integer A00. This is a setting
mode getter, not a protocol receipt status or primary identity version.

0zt.A0U returns 0Vl.A00 immediately for mode 3. Other modes obtain separate
0YK.A0E/A0F lists and A0R -> 7pF.A01, construct CR9, choose BzC.A03 and call
private A0G with the saved set and mode. It expands the returned users through
A0T and logs `StatusParticipantUserManager/generateTargetDevicesFromPeerStatusSetting`.
Full privacy modes, list authority, A0G and A0T device expansion remain
**unknown**. The diagnostic anchors this as status-target work; it does not
justify using arbitrary directory members or treating every outgoing direct
message as this branch.

## Existing device membership, not user receipt membership

**Confirmed:** 0Z4.A02 uses Boolean property **18225** (0x4731). True selects
0Z4.A00(message)'s store and **0Z5.A05(message)**; false requires the original
message key, selects 0Z4.A01(key)'s store and invokes **A0O(key)**.
Effective property and complete subtype store choice remain **unknown**.
A05 loads 7hf through A0E and returns a new HashSet of its A00 cache's keys,
without testing a timestamp or version in this body. The cache load is separately
retained in [receipt loading](receipt-load.md). A0O is an abstract declaration
with no body in this base class; concrete overrides remain **untraced here**.
Do not assume the two property paths have identical loading/failure behavior.

**Confirmed negative / scoped:** this selected DNt branch has no receipt_user
INSERT/UPDATE, no per-user timestamp write, and no primary identity cleanup.
It calls the already traced 0Z4.A08 device/coexistence blank writer after device
set subtraction. Thus it does not close normal-send user-row seeding. This is
not proof that no other writer exists anywhere in the APK. No receipt_user row
may be inferred from original devices, cached device keys or an immutable target.
The [primary cleanup intersection](identity-receipts.md#primary-receipt-selection)
still requires actual user-row existence, independently of a delivery timestamp.

**Unknown/unported:** full ordinary user-row initialization/admission,
status privacy/target authority, A0O overrides, property values, asynchronous
cache/store reconciliation and primary receipt cleanup. No Rust runtime, account,
key, identity, original request ID, wait or reservation changed. No live request
or cleanup ran. This source closes a candidate investigation, without inventing
user membership or claiming normal-send initialization complete.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-blank-receipt-deferred.json \
  --output docs/protocol/apk/2.26.37.71-blank-receipt-deferred --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-blank-receipt-deferred \
  docs/protocol/apk/2.26.37.71-blank-receipt-deferred
```

Verification: exact regeneration matched **13 files / 140,420 bytes**.
Cached self-diff checked eight unchanged topics and four metadata artifacts;
this is same-APK reproducibility, not cross-release semantic parity.
CLI-reference regeneration/coverage, OpenAPI library-symbol coverage, formatting,
whitespace and repository hygiene passed (1,905 local links; 84 checksum bundles,
18,046,975 bytes). No fresh workspace runtime tests or live operation are
claimed for this source-only chunk.
