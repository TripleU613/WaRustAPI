# Registered before-insertion hooks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-before-hooks.json),
[retained methods](apk/2.26.37.71-text-before-hooks/targets.json),
[ordered source index](text-before-hook-index.json).
This completes the immediate before registry from
[text insertion](text-type-inserter.md), alongside the
[54-entry after registry](text-final-after-hooks.md). The three selected
additional delegates below have bounded interpretation; other helpers are not
fully traced. No Rust runtime, account, network or request-ID change.

## Factory continuity

The retained **07A.A3C** initializer supplies these 13 IDs. Root selection is
**(ID >> 14) & 0x7f**, selected switch keys are **ID & 0x3fff**; root zero
uses 1UE.A01 partitions, roots 2/4/5 select 1xr/6as/8kb.A00. The index links
existing complete provider methods rather than copying their large bodies.
Offsets in this table are method-relative bytes in the selected switch.

| Index / ID | Selected dispatch / key / offset | Factory / hook |
| --- | --- | --- |
| 0 / 0x9f2 | 1UE.A0Q / 2546 / 0138 | 0VW.A0C -> 1vW |
| 1 / 0x6f7 | 1UE.A0N / 1783 / 011a | 0HT.A0x -> 1vV |
| 2 / 0x16e3 | 1UE.A0D / 5859 / 08c2 | 0wQ.A0L -> AVE |
| 3 / 0x16e5 | 1UE.A0D / 5861 / 08d6 | 0wQ.A0H -> AVC |
| 4 / 0x1413a | 8kb.A00 / 314 / 0f02 | direct new D7H |
| 5 / 0x77a | 1UE.A0N / 1914 / 05de | 0HZ.A0f -> D7I |
| 6 / 0x1410f | 8kb.A00 / 271 / 0cfe | direct new AVD |
| 7 / 0x10224 | 6as.A00 / 548 / 1e38 | direct new 8C8 |
| 8 / 0x3c5 | 1UE.A04 / 965 / 0234 | 0Fv.A1Q -> D7J (previous trace) |
| 9 / 0x3c6 | 1UE.A04 / 966 / 023e | 0Fv.A1R -> D7K |
| 10 / 0x14179 | 8kb.A00 / 377 / 1214 | direct new AVB |
| 11 / 0x858f | 1xr.A00 / 1423 / 5efa | direct new 3KE |
| 12 / 0x101f8 | 6as.A00 / 504 / 02be | direct new 8C7 |

**Confirmed bounded negative:** none of the 13 retained CHJ bodies directly
calls **178.A0H**, assigns **178.A08:I**, or contains literal **receipt_user**.
This excludes only direct instructions in those methods. It does not exclude
transitive writes, nested insertion, triggers, provider initialization, callbacks
or subclass implementations. Native status reads and helper arguments are not
native status assignments. Full before/after mapping is not a complete client port.

## Selected behavior

**1vW** calls 1eZ.A09 with 0YA/0Fs and the message; a true result ORs
origination flag **0x200** through A0J. Full qualification is unknown.
**1vV** obtains an 00s provider value then discards it; provider initialization
may have effects. Its apparent lack of direct mutations is not proof of a no-op.
Both contain the retained nonnull callback refusal `onProcessorExecuted`.
The insertion caller previously passes null; other caller contracts remain separate.

**AVE** first requires 8o9.A03 and a nonnull chat, and clears existing AUu.A03.
It excludes Bmy, requires stored 0Zn low bit, compares message A0F with a
property-0x6c9b long, and admits the chat raw string into a static Set. It
constructs a **new Bmy**, invokes nested **1hh.A01** with **AiX selector 11**,
then passes that new object to 0Ys.A0O(reason -1). The selected Exception
handler removes the Set entry and calls A6R.A00 with 6/true. That numeric 6
is a helper argument, not a direct A0H setter. The helper construction,
callback, Set lifecycle and resulting system-message status remain unknown.

**AVC** selects outgoing, eligible chat messages excluding A0y and Bmy,
requires positive stored chat A0Y plus 0Zn, then attaches **AUu**. Its fields
come from low-bit/bit-1 booleans, **(0Zn.A00 & 0x3c) >> 2**, and chat A0Y.
Full chat policy and enum meaning remain unknown; AUu metadata is not native
receipt-user membership. This is the producer side of the AUu path previously
seen in the AVI after-hook, not proof of complete sharing behavior.

**D7H** invokes 6al.A1M(null,message,D5z) before qualifying a 1jl/CrW
AppointmentInfo with a nonempty identifier and property **0x8dd5**. It builds
D5z with an Integer selected by comparison against C0i.A07, attaches it,
and chooses **A0I/A0K** for lookup mask **0x4000000000** based on intValue.
The retained **178.A0K** delegate confirms **A0n &= ~mask**; A0I's OR is
previously traced. This resolves the selected lookup-mask mutation, not an
appointment enum, origination mask or native status. Complete 6al/enum producers
remain unknown. The D7n after-hook is the later appointment-table writer.

**D7I** handles qualified Fdp payment metadata, looks up another transaction
by message key, compares through Fdp.A0R and chooses a transaction string
from stored/current Fdp fields or **UNSET**, then invokes 1iG.A02. Its
synchronized flag read and null transaction diagnostic are retained. Full
transaction-string setter/storage/permission behavior remains unknown; no
payment or protocol capability was added.

**AVD** handles 1iz subtype value 1 with a positive configured offset and
message A0j equal **-1**. It queries
`SELECT _id FROM available_message_view ORDER BY _id DESC LIMIT 1`, using
**1** when the cursor is empty. If that selected value equals 1, A0j becomes
the configured offset; otherwise it becomes **selected ID + offset + 1**.
This is row-ID assignment, not receipt insertion or native status. The equality
branch also applies to an actual selected row ID 1. Cursor/store close failures
and suppressed exceptions are retained. Offset origin, collision policy and
view definition remain unknown; no universal positive-ID guarantee is inferred.

**8C8** selects outgoing supported message text (1he,72p,1iB or17A fields/list),
requiring a nonempty string satisfying **5TG.A01** and property **0x5e92**,
then calls **B31.A00(message,true)**. That retained delegate ORs origination
mask **0x800000000000** via A0J; its false branch clears via A0L. String
qualification and full feature meaning remain unknown. This is not encryption,
attestation, receipt admission or a native status change.

**D7J** is the previously attributed count preprocessor; its CHJ and registration
remain linked from the index. Its counter is not itself a receipt-user row.
**D7K** checks hosted/coexistence chat eligibility, current device-target sets,
feature/property paths and hosted-device predicates, then can OR origination
mask **0x8000000**. Its diagnostics say no hosted/CoExV2 device was found.
Full hosted classification, targets and effective feature policy remain unknown;
this is not permission to fabricate a hosted device or merge namespaces.

**AVB** gates on A0C and reads B3b: when A0C is zero only native statuses
**0/1/2** proceed; nonzero A0C skips that filter. An A0S message with nonnull
A16 returns. Property **0x229c** and 1if.A02 gate assignment of boolean A0a
and **CmM.A01() bytes to A16**. Full byte generation/semantics are unknown;
these bytes are not assumed to be identity keys or attestation.

**3KE** has UserJid/outgoing/subtype/device/feature/contact predicates; selected
contact metadata A0z leads to **B35.A05(message,0)**. The retained delegate
sets **D6C.A03 = 0** and clears origination mask **0x100** through A0L.
Its positive-argument branch stores that argument and ORs the same mask.
This field is not 178.A08. Full contact, D6C construction and predicate meanings
remain unknown. **8C7** calls 7z0.A04 and, when true, ORs lookup flag
**0x40000000** through A0I. Full qualification remains unknown.

## Remaining admission boundary

**Unknown/unported:** transitive hook effects, actual receipt_user seeding,
normal-send native status progression, effective receipt policy and complete
PN/LID counterpart authority. Primary identity cleanup still requires actual
native user-row existence; DeliveryFacts, original target counts and these
metadata/flag writes are not substitutes. Mapping all **13 before + 54 after**
entries identifies the immediate registry but does not close this dependency.
The next source work must follow actual receipt-user insertion/admission rather
than assume a missing hook writes it. Existing APK export/diff/smali commands
reproduce this research; no new CLI flag, HTTP route or live operation exists.

## Validation

Before-hook validation: exact 21-topic export (26 files/83,350 bytes), cached
self-diff, 13 ordered dispatch/factory/body assertions, CLI reference,
API/library-symbol checks, formatting and whitespace passed. No fresh runtime
suite or live account test is claimed.

Hygiene checked 2,106 local links and 104 APK checksum bundles.
