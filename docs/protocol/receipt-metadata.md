# Receipt-route flag and mention/bot metadata provenance

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Fourteen-topic recipe](apk/recipes/2.26.37.71-receipt-metadata.json),
[complete methods, signatures and catches](apk/2.26.37.71-receipt-metadata/targets.json),
[physical field writers and constructor/attachment callers](receipt-metadata-evidence.txt).
Offsets are method-relative bytes unless the inventory labels them DEX offsets.
This extends [receipt routing](receipt-route.md), not a complete Rust user-row port.

## Four explicit flag writers

**Confirmed:** the all-DEX writes-only scan finds four physical methods writing
178.A0z. No constructor write is in this exact scan; **inferred:** absent an
explicit initialization, the new instance starts with the platform false default.
This does not prove the flag on a fully processed, restored or quoted message.

- **1gy.A05:** follows 178.A09's nested message, adopting the second message's
  nested value through A0M when needed. For a nonnull nested value it looks up
  its key through 0V9.Aon. When found, it copies 1ib.A01's list through A04;
  it then sets the **nested** message's A0z=true at 005e..0064, even if lookup
  returned null. The parent message is not that assignment's receiver. Complete
  quoted/copy context, A01 selection and lookup authority remain untraced.
- **1iN.CHJ:** only when A0n bit 1 is set, calls 16S.A01 to load mention metadata,
  then writes 1ht.A03(message)'s result at 0034..0040. Its nonnull processor
  argument has a separate callback/error branch retained but untraced here.
- **CoJ.A05:** incoming MessageContextInfoDeserializer context processing calls
  1ib.A04 with its built list at 2166..216c, then stores 1ht.A03's result at
  216c..2178. Complete mention bitmap, limits and other context branches remain
  untraced; the whole method is retained, not claimed fully reverse-engineered.
- **DKL.CFe:** reads the source message's A0z and copies it to the destination
  at 0006..000e. It does not recompute metadata classification.

**Confirmed quoted-message scope:** 178.A09 checks virtual A0n and returns
A0H only when allowed; the base A0n returns true. A0M's diagnostics explicitly
identify a quoted message and check source A0n, target A0m and target A0e()==2
(StorageType.QUOTED). Its valid path clears the target's own quote through
A0M(null) before assigning A0H; null input skips those checks and clears A0H.
Complete virtual overrides and diagnostic assertion behavior remain untraced.
This proves the quoted receiver scope of 1gy's flag write, rather than treating
that assignment as universal initialization of the parent outgoing message.

**Inferred consequence from these assignments and 1ht.A03's retained branches:**
classification can depend on prior flag state. An existing true flag bypasses
metadata-user selection on recomputation, while chat exclusion, message A02 and
BotGroupMetadata presence still apply. A fresh computation from only the current
mention list would lose this history. No live observation of that lifecycle is
claimed; do not translate it into a generic receipt acceptance flag.

## Mention list attachment and database hydration

**Confirmed:** 1ic.<init> stores the supplied List reference directly; it does
not copy or validate list elements. The scan finds only that physical A00:List
field writer and one constructor invocation, 1ib.A04. This is not proof of
immutable elements or absence of in-place collection mutation.

A04 attaches a new 1ic for a nonnull/nonempty list via 1i7.A03, and ORs A0n
with bit **1**. For null/empty input it clears the descriptor through A03(null),
**without clearing bit 1 or A0z** in the shown method. Thus an absent descriptor
cannot be substituted for a cleared lookup marker or a reset route flag.
The physical attachment inventory has 15 calls, including SQL load, outgoing
media preparation, copy, deserialization and other retained-but-untraced routes.
Those are calls, not fifteen verified workflows.

16S.A01 queries **message_mentions**, selected by message_row_id, for
jid_row_id, display_name and mention_type. This is not receipt_user. Null/zero
mention_type takes the JID path: resolve through 0VT.A09 then 0D7.A00, skip an
unresolved value, otherwise construct DLg(jid,display_name). Type 1 has a
322.A01 setting/account gate and can yield D32.A00. Unknown types log and skip.
Full gate defaults and JID conversion authority remain unknown. The query has
no ORDER BY; do not invent a canonical winner for the first qualifying user.
After closing resources it passes the collected list to A04 at 0144..014a.
The [route selector](receipt-route.md) filters DLg only, so the D32 marker is
not interchangeable with a qualifying user mention.

## The excluding descriptor is BotGroupMetadata

**Confirmed:** 1ie's toString names `BotGroupMetadata(participantsMetadata=...)`.
Its constructor requires a nonnull Set and stores the same reference; equality
and hashCode use that Set. The exploratory A00:int field search had no matches;
that field-type hypothesis is refuted, not evidence that A00 is absent. The
correct A00:Set writes-only scan finds the constructor.

1id.A01 sets or clears lookup bit **0x100000000** depending on descriptor
presence, then attaches the supplied descriptor through 1i7.A03. 178.A0I ORs
its argument into A0n; A0K ANDs A0n with the complemented mask. This explicit
clear differs from the mention-list helper's null path above.

CoJ.A05's gated BZI.botGroupMetadata path reads BOd.participantsMetadata,
converts each BPA.botFbid string to CTT, collects a Set, constructs 1ie at
09f4..09fe and attaches through 1id.A01. Earlier bot feature/context gates and
protobuf field numbers remain **unknown** here. 1ht.A03 rejects descriptor
presence, irrespective of Set cardinality: an empty Set is not absence.
The constructor scan also finds 9L7.C8k, Cm6.A08 and DGm.A00; their full sources
and producer/serialization authority remain untraced. The diagnostic name
establishes the native descriptor label, not complete service/account semantics.

## Rust and remaining authority

No normal-send receipt_user row is established by a flag, mention-table record,
lookup bit, bot descriptor, original target or Delivered fact. Actual user-row
admission still requires the [native writer](receipt-user.md), status/ownership
selection, read settings, manager filters and counterpart resolution. These
findings prevent constructing fictitious membership for primary identity cleanup.

**Unknown/unported:** complete field/message subtype semantics, descriptor
producers and serialization, effective settings, user-row seeding/admission,
primary cleanup and final cross-device recovery. Rust runtime and saved account
state are unchanged. No login, chat, receipt, identity reset or wait alteration
ran. Existing APK research CLI commands reproduce the work; research is CLI-only
with x-live=false and no new HTTP route. Reviewed new-release selectors remain
necessary for unanchored topics; static differences alone do not prove parity.

Validation: fourteen topics / 19 files / 208,314 bytes reproduce exactly from
APK 37.71; cached self-diff reports unchanged artifacts. API-symbol checks, CLI
reference regeneration, formatting and diff hygiene pass. The preceding
812-test result covers unchanged Rust; this chunk is static research/docs only
and makes no new messaging-runtime or live-validation claim.
