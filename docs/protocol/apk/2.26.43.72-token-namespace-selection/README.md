# Trusted-contact target, storage and counterpart namespaces

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
DEX exports only; no target execution, account operation or REA Evidence ID.
This follows [direct timestamp state](../2.26.43.72-token-success-state/README.md).
The current names re-anchor the earlier37.71 token feature/counterpart trace.

## Request/pending namespace is not the store key

87x.A00 reads Boolean14303 (0x37df). When true it delegates to0ht.A0L.
Otherwise, for a JID accepted by0Dn.A0Y (types18/19/25), it attempts the
phone counterpart and returns it when nonnull; other inputs remain unchanged.
The full selected helper preserves the exact conversion calls. This chooses
the user later passed into pending admission and job construction by87x.A04.

0ht.A0L independently checks0Dn.A0d: type0. For that PhoneUserJid it calls
0Ws.A0E and returns a nonnull account-user result; otherwise it returns the
original user. It does not read14303. Thus a PN wire/job target can use an
account/LID-backed persistence key. The two namespaces must not be conflated.
The ordinary branch can be summarized, conditional on proven mapping results:

| Input |14303 true pending/job user |14303 false pending/job user |0ht storage normalization |
| --- | --- | --- | --- |
| PN | mapped account user, else PN | PN | mapped account user, else PN |
| Account user | original user | mapped phone user, else original | original user |

This table is not authority to manufacture mappings, choose an effective Boolean
from its packaged default, or rewrite a request that was already reserved.
The [effective configuration boundary](../../token-feature-inputs.md) remains open.

## Counterpart side effects are separately gated

0ht.A02(original,normalized) reads Boolean15491 (0x3c83). True returns null.
False still returns null when original differs from normalized; only equality
calls0Ws.A0K(original). A0K rejects its A0X category, routes A0Y account users
toA0J phone lookup, and its A0c phone category toA0E account lookup. Complete
predicate category meanings, reverse lookup authority and exceptional dispatch
are not collapsed into an arbitrary bidirectional alias map here.

The retained [A0V](../2.26.43.72-privacy-token-callback/success-store.txt) performs
its canonical timestamp transaction before this counterpart selection/deletion.
When a mapped PN differs from its normalized account user, A02 yields no
counterpart; one must not automatically delete the input PN row merely because
A0L returned another namespace. For an unchanged account user, a nonnull selected
phone counterpart may instead be deleted by that later separate handle.
This narrows a real persistence obligation, not an integrated Rust alias port.

## Mapping authority and bounded negative result

0Ws.A0E first checks message-store readiness. A not-ready store returns null
with a fixed native diagnostic. The own-user branch returns09F.Auj. Otherwise
it obtains a phone row ID, checks its cached0t7 selection, loads0Wy.A01 on a
cache miss, takes0t7.A00 and materializes an account user viaA00. A nonpositive
phone row ID is rejected; a null or negative selected account row is rejected.
The selected row and subsequent materialization/validation are distinct steps.
The whole method is retained without claiming those transitive helpers closed.

Rust jid_map::select_account_row models a retained older APK row selector, not
a runtime authority to adopt PN/LID relationships. authenticated-lid/lid-for
checkpoints have separate association/alias roles; they are not evidence of
this native readiness, own-user, ranked-row or effective configuration state.
This audit therefore does not merge or delete token rows from those aliases.
Complete native lookup selection/materialization and counterpart lifecycle remain
unported, as do full configured expiry and pending/job semantics.

## Confirmed existing bot exclusion

The earlier callback bundle retains0Dn.A0O: getType26 or27 returns true.
Current0xC.getType is26 and getServer is bot; its constructor says Invalid
BotJid and validates a numeric nonzero-leading user. Dv2 extends DeviceJid,
captures a0xC and device index, returns type27 and server bot, with its native
primary-device restriction. These are bot identities, not unnamed hosted/LID
classes. 87x.A04 applies A0O after namespace selection and before state access.
Neither namespace branch converts these bot inputs through the selected type0
or A0Y mapping paths, so this exclusion is independent of14303.

Rust already schedules only nonempty PN/LID targets; @bot is outside that
allowlist. The new regression checks bot user and primary-device forms produce
no action or checkpoint mutation, typed issue recognition excludes them, and
PN/LID positive controls still schedule. This confirms an existing guard; it
adds no new sending restriction, server claim or bot-traffic experiment.
Hosted/other producer parity is not established by this bot-specific result.

## Reproduction

Existing xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions exports0ht.A0L/A02,87x.A00,0Ws.A0E/A0K,0Dn.A0d/A0Y and full
0xC/Dv2 classes. Existing apk-virtual-overrides on Jid.getType ()I locates
the26/27 declarations through superclass ancestry. That inventory says nothing
about interface dispatch. The previous callback bundle retains A0O and A0V;
the reservation-refresh bundle retains full87x.A04. Checksums cover every other
file. No binary, account mapping or bulk decompiled tree is committed.
