# Receipt factory and store dispatch

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [seven-topic recipe](apk/recipes/2.26.37.71-receipt-factory.json) retains
[full signatures, registers and exception tables](apk/2.26.37.71-receipt-factory/targets.json).
This corrects the writer boundary in [receipt application](receipt-apply.md).

## Status admission and downgrade

**Confirmed:** 1hg.A03 returns true exactly for statuses 5, 8, 13, 15, 17,
and 18. [The parser trace](direct-receipts.md) independently maps these to
ordinary delivery, played, read, inactive, read-self and played-self respectively.
This predicate includes self/inactive statuses; it is not just recipient delivery.

CNn.A00(DeviceJid,UserJid,178,int,long)->CYV performs no database write.
At 0020..005c, if 0zd.A07(message-key chat) is false, status is 8 or 13,
and the message-key A02 boolean is true, it logs that the setting is off and
replaces status with 5. Otherwise it preserves the input status. Device, original
user, message and timestamp are passed unchanged to CYV's constructor.
CYV stores these five inputs in A02, A03, A04, A00 and A01 respectively.
A settings downgrade is not evidence that a sender should emit a fabricated
read/played receipt or that all receipt facts should be merged in Rust.

**Confirmed / limited:** 0zd.A07 returns true for any of three JID predicates
(0DE.A0m/A0b/A0h), or when 08l.A1H and its private A00/A01 predicates are all
true. The full meanings of those predicates, setting defaults and current
account settings remain **unknown** here. The log alone does not establish
that one global boolean reproduces this policy.

## From value to storage

**Confirmed:** Cmn.A00 partitions CYV values by message instanceof 1iZ.
6an.A1Q adds to its first collection when its integer flag is nonzero, otherwise
to its second. The 1iZ collection goes directly to 0Z4.A06 at 03d4..03da;
other values go through 15A.A02 at 0498..049e. The latter is a large
message-state processor, not a trivial pass-through: its complete admission and
side effects remain **untraced**. Its retained body includes 0Z4.A06 calls at
07b4..07ba and 0a00..0a06, establishing the downstream store without claiming
every input reaches it unchanged.

0Z4.A06 partitions inputs in order: instanceof 1im, then instanceof 1iZ,
then ordinary values. Ordinary values invoke its A07:0Z6 instance's inherited
0Z5.A0A; 1im uses injected 0Z5 through A06; 1iZ uses A08:0ZE's inherited
0Z5.A0A. These class names are evidence, not guessed human message types.
[Previously retained 0Z5.A0A](receipt-version.md) filters positive timestamps,
selects the minimum within a batch per message/device pair and updates its
receipt timestamp column while preserving an existing nullable primary version.
Its timestamp write does not select a read/played column from CYV.A00.
Thus read/played values admitted by the factory can flow toward device receipt
storage, but the intermediate message-state policy must be traced before a
complete Rust implementation.

**Confirmed / limited:** 0Z4.A06 also has separate co-existence receipt writes
for qualifying Cju.A01 devices with an original user. Ordinary values can update
`receipt_coex` keyed by message_row_id/user_lid_row_id; the 1iZ branch has
`message_add_on_receipt_coex`. These are not proof of unrestricted PN/LID
aliasing. Cju's predicate, row resolution and complete co-existence authority
remain **unknown**. No account store, runtime receipts, resend or network
operation was changed by this research.

Reproduce with apk-research-bundle and the recipe's --check directory. Export
a separate anchored generation for a new release, then use apk-research-diff.
No new CLI/API capability was introduced; existing inspection commands remain
unchanged. Next inputs are 15A's admission/status transformations, parser clock
units/defaults, concrete store columns and identity-cleanup/cache policy.

Receipt-factory validation: seven source topics / 12 files / 137,766 bytes
reproduce exactly from APK 2.26.37.71. Cached self-diff reports seven unchanged
topics and four unchanged metadata artifacts; this is reproducibility, not
cross-release equivalence. Repository hygiene checked 1,340 local links and
28 APK bundles / 8,059,427 bytes; diff hygiene passed. Research/docs only; no
fresh Rust runtime test or live account operation is claimed for this chunk.
