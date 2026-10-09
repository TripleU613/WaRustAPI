# Native receipt recipient-count producers

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Five-topic recipe](apk/recipes/2.26.37.71-receipt-recipient-count.json) and
[complete selected source](apk/2.26.37.71-receipt-recipient-count/targets.json).
This follows the [ten physical A06 write inventory](receipt-projection-count-evidence.txt)
and [receipt threshold calculation](receipt-projection.md). Offsets below are
method-relative bytes. Retained complete bodies are not complete semantic ports.

## Status-audience delta and bucketing

0zt.A0H is identified by StatusParticipantUserManager diagnostics. With no CJj
input it builds one through A00. Its prefetch-empty-audience/property branch
can return an empty set before updating anything. The Boolean mode at 0060
can select A0O and return instead of the later delta branch. Full CJj audience
construction and effective property/mode values remain unknown.

In the delta branch it loads the existing participant set, obtains CJj.A03 as
the new audience, and constructs removed/added sets. It resolves the local user
through 08Z.Apx; a null result logs and skips the normal update/count block.
After participant updates, 0252..026a computes audience.size minus one **only
when the audience contains that local user**. For a BpI wrapper, 0272..02a6
writes its underlying message A0U from participant serialization and A06 from
that count. A non-BpI wrapper does not take this A06 write. This count is of
users in a selected audience set, not encrypted device targets.

0zt.A0O is separately identified by updateParticipantsDeltaWithBucketing.
At 0252 it preserves the first input Set (parameter v26) as local v24.
After the selected SQL paths, 0714..0730 uses that set's size minus one if
it contains Apx's local user. BpI writes A0U and A06 at 0738..076c; the separate
73x branch applies its count through a 74M setter, not message.A06. That is
not evidence that every native status representation stores this field.
Complete bucketing, audience SQL/defaults and wrapper/subtype authority remain
unknown here. No status audience was generated or uploaded.

## Main insertion uses this count only under status predicates

0zC.A0E initially computes v23 from 0DE.A0i(chat), the independently traced
native type-five predicate. The direct A06 write at 07e8 sits behind its
0742..0752 context and that predicate; not every message insert runs it.
In the selected v17 branch it builds BpI(message), a CJj through BzC.A06/A00,
and resolves the local user with Apx. Nonnull local user gates this count:
CJj.A03.size minus one when CJj.A03 contains that user (07cc..07e8).
The participant string comes from a separately assembled set; it must not be
substituted for the CJj audience merely because both are nearby.
Null local user jumps past the field/string block. Full v20/v17 producers,
transaction ownership, deferred DOl selector 9 and all other insertion effects
remain unknown. The retained large method is evidence for this bounded branch,
not a complete message insertion implementation.

## Broadcast clone processing has a different count

36W.A02 identifies a missing mapping as missing_pn_lid_mapping_for_broadcast.
It rejects non-fromMe keys and 1bh/Bpo subtypes. Ordinary admitted status must
be 0 or 2. Its exceptional mode v14 is enabled only when A0y is true, status
is 4 and property **26867** is true; that mode can proceed from status 4.
A0y is not assigned a universal ownership meaning here. Effective property
and subtype authority remain unknown.

It requires a nonempty virtual message.A0D list. The retained base A0D merely
returns A0W; subtype overrides and list producer invariants are not established.
The loop starts count zero. In exceptional mode an own-user BO0 recipient is
skipped. It normalizes each other recipient through 0Yo.A0C(input,null);
a null mapping logs and skips without incrementing. A mapped recipient is
passed to clone helper A00, core store A06 and 36Y.A02, then increments count.
Those helpers' complete insertion/success semantics remain unknown: increment
is not independently verified durable insertion or delivery.

At 0172..017a **only exceptional mode** overwrites original message A06 with
this processed-entry count. All modes then call 15e.A05 and return the count.
There is no list deduplication in the selected loop; do not claim it counts
distinct recipients or raw devices. This does not initialize ordinary direct
text A06 or seed native receipt_user rows from a cloned intent.

## Restoration and outstanding writers

The existing [0VA cursor restoration source](apk/2.26.37.71-text-status-initialization/cursor-state-restoration.txt)
reads the literal `recipient_count` column and assigns its getInt result to
A06 at 0246..025e. It does not derive count from current receipt/device rows.
The scan's remaining Bp2.A0P, D7E.Byt, D7J.CHJ (two writes) and DMZ.run write
values/conditions remain untraced here; normal send/copy authority is not closed.

**Unknown/unported:** ordinary outgoing preparation/status, full native
receipt_user admission/settings and counterpart scope, remaining count writers,
cache/transaction callbacks and primary identity cleanup. Single-message receipt
projection does not use this multipart threshold, but its admission still must
be implemented faithfully. Rust device cardinality and raw DeliveryFacts are
not substituted for native counts or receipt_user membership. No runtime,
private account, identity, keys, request IDs, waits or live operation changed.
Existing APK CLI commands expose reproduction; no HTTP route/live capability
was added. Re-anchor separate recipes for future APKs before cached file diff;
static identity is not live or cross-release semantic parity.

Validation: five topics / ten files / 145,660 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies five unchanged topics and four
metadata artifacts, not another APK release. CLI reference regeneration,
OpenAPI/library-symbol checks, formatting and diff hygiene pass. This source/docs
chunk changes no Rust code; no fresh runtime suite or live check is claimed.

Follow-up: [count preprocessing and copies](receipt-count-preprocessing.md)
attributes the remaining five physical writes and participant-map helpers.
Immediate values are now attributed for all ten sites; full send initialization,
wire producer and preprocessing/mapping authority are still unproven.
