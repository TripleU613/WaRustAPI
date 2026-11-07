# Vector translation mapping and sentinel lookup

**Confirmed selected static evidence**, same pinned APK/native member as README.
No native execution or live account operation. Scope: the non-mode-3 vector
branch of 0x8bfe0 and its concrete lookup; upstream parser semantics remain open.

After the projected-hash comparison passes, 0x8c488 allocates 96 bytes and
initializes four 24-byte vector descriptors. The selected reader count getters
0x96b54, 0x96b80, 0x96bd8 and 0x96bac supply signed lengths, sign-extended to
the vector-fill helper **0xa8574**. Each vector element is a 16-bit value;
the initial fill value passed is **0xffff**. Helper growth includes size/error
paths; this does not authorize blindly allocating attacker-provided sizes.

The mapping loop at 0x8c76c..0x8c800 traverses two record ranges in 0x50-byte
steps. Each key is formed literally as **u32(+0x30) OR (u32(+0x4c) << 12)**.
On unequal keys, unsigned ordering advances one side; on equality, it reads
the first record's signed byte+0x40, subtracts one for the type-vector index,
and uses the second record's u16+0x3c as the slot index. It writes the first
record's word+0x3c, truncated to u16, into that mapping slot. Both iterators
then advance. The record-range provenance, sorting invariant, duplicate-key
policy and public field names are not established here.

Before writing, unsigned bounds comparisons require the type index below the
outer vector count and slot below the selected inner vector length. Failure
branches to **0x8cb1c**, logs `TranslationTableTypeSlotOutofBoundsError`
(0x17479), method `FBMobileConfigStorageMigrationUtils::buildTranslationTable`
(0x1a049), and formatted type/slot values. Critically, after logging and string
cleanup, **0x8cbdc branches to 0x8c820**, the vector-swap cleanup path. It does
not clear all mappings or return false at that point. Already filled vectors
and sentinel entries can survive into the installed translator. This is a
selected static control-flow fact, not proof that valid upstream data reaches
that branch.

The selected translator vptr is 0xd6320. Lookup slot+0x10, RELA
**0xd6330 -> 0xa86cc**, receives a packed 64-bit value. It extracts **bits
48..53** for a type, subtracts one, and bounds-checks against outer vector
count. It extracts **bits 0..15** as the slot and checks the selected inner
vector length. For in-range lookup it replaces only the low 16 bits with the
stored u16 mapping and preserves the upper 48 bits. On either bounds failure
it returns **input OR 0xffff**, likewise preserving the upper 48 bits. An
in-range slot still containing the fill sentinel naturally produces the same
low-16 sentinel. No special sentinel rejection exists in this lookup body.

Readiness slot+0x20 (0xa8724) checks only outer-vector nonemptiness. Four outer
descriptors exist on this selected construction path even if their lengths are
zero. Readiness therefore must not be interpreted as complete slot coverage,
absence of sentinels or proof that the mapping loop reached its natural end.

**Unknown:** actual packed-field meaning, all lookup callers, how consumers
treat low-16 sentinel values, nested parser validity and final activation.
No Rust runtime mapping is added from this incomplete evidence. Retained
mapping-build.txt, mapping-bounds.txt, mapping-lookup.txt, mapping-fill.txt,
mapping-slots.txt and mapping-data.json reproduce the bounded static trace.
Full REA evidence in rea-mapping.json covers the lookup implementation.
