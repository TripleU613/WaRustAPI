# Selected boolean extraction uses mask gates and bit 7

**Confirmed selected static evidence**, same pinned APK/native member as README.
No target execution or live account operation. This is the selected branch at
0xa2604, not a claim that all public boolean reads use this caller.

Outlined 0xcaf00 loads receiver from [x20], index from record word+0x3c and
receiver vptr. The branch calls that receiver virtual+0x40, retains its returned
word in w24, then invokes virtual+0x20 with the same receiver/index. For the
selected reader vptr 0xd61c0, slot+0x20 is the previously traced raw-byte
getBoolData (RELA 0xd61e0 -> 0xa351c); slot+0x40 resolves to 0xa3868.
The enclosing receiver construction and complete method identity remain open.

The selected control flow after these calls is:

1. If **(w24 & 0x11) == 0x11**, branch to 0xa2724 and set local byte
   stack+0x90 to 1. It does not extract/store a boolean on this branch.
2. Otherwise, if **(raw & 0x11) != 0**, branch to 0xa2760 without storing a
   boolean payload in this selected block.
3. Otherwise extract **(raw >> 7) & 1** at 0xa2638. After helper 0xc73a8,
   store that bit to local stack+0x78 and tag **1** to stack+0x70.

This establishes the value extraction and validity-mask distinction at this
boundary. Raw 0x00 yields value bit 0 and raw 0x80 yields value bit 1 when the
first gate permits extraction. Raw default **0x11** fails the second gate and
does not become boolean true merely because it is nonzero. Bits other than
0x11 and bit 7 are not all rejected by this selected block; do not impose an
unproven two-value-only encoding.

The subsequent 0xa2760 suffix additionally uses metadata w24: bit 6 can set
local stack+0x50, mask 6 gates other metadata work. Public meanings for those
flags, the full lifecycle of the local tagged value and all downstream result
handling are unresolved. Neither a no-store path nor local tag 1 proves the
final API return or server acceptance.

**Unknown:** typed caller binding across the full construction chain and the
translator-sentinel consumer. This trace resolves a conditional boolean decoder,
not the packed translation path. No Rust runtime correction is made without
binding the corresponding Rust path and complete caller semantics.

Retained bool-decode-body.txt, bool-decode-args.txt and bool-decode-slots.txt
provide bounded LLVM/RELA reproduction. Full REA record is rea-bool-decode.json.

The [metadata getter follow-up](bool-metadata.md) records the exact default
word 0xffffff11, field-8 source and cached count bound, separately from the raw
byte getter. Flag names and final public result remain unresolved.
