# Metadata word supplying the boolean decoder gate

**Confirmed selected static evidence**, same pinned APK/native member as README.
No target execution, live account operation or Rust runtime change.

Reader vptr 0xd61c0 slot+0x40, RELA **0xd6200 -> 0xa3868**, supplies the word
retained in w24 by the selected boolean-decoder branch. This getter is distinct
from slot+0x20 getBoolData; its semantic public name remains unproven.

The normal 0xa3868 body reads root+0x50. Null selects the default. Otherwise
outlined 0xcb66c preserves the reader and selects its root as receiver for
0x96c10. That accessor uses vtable field **8** via the previously traced
optional relative-offset helper 0x9ac18. Absent field also selects default.
The getter compares input index unsigned against cached **reader word+0x5c**;
index >= cached count returns default. Otherwise it addresses **vector+4+
4*index** and returns the 32-bit word through outlined 0xcc250.

The default pointer comes from **0xcbf20 -> rodata 0x23798**. Pinned bytes are
**11 ff ff ff**, little-endian **0xffffff11** (4294967057 unsigned, -239 signed).
Keep the bit pattern; neither signed interpretation nor its nonzero value is
a boolean result. This differs from the byte getter's separate default at
0x23797, **0x11**. Both have mask-0x11 equal to 0x11, so the selected decoder
metadata-default gate takes its no-boolean-extraction branch as documented.

Unlike getBoolData, this selected metadata getter does **not** load/compare
the actual vector's u32 count a second time and does not log DiskCorruptionError
in its own body. It relies on the cached bound and earlier construction state.
The retained count initialization window 0xa2d48..0xa2d98 shows root accessor
0x96c10 count, or zero if absent, stored at reader+0x5c. The full lifetime and
immutability contract are still unknown; no new exploit or live mutation is
tested or inferred from differing local checks.

**Unknown:** public names for metadata flags, full receiver/caller construction,
the final tagged result's lifecycle and packed-translation sentinel consumers.
The decoder's bit-7 value extraction remains conditional on metadata/raw gates;
this trace adds its exact default and source field, not a complete config port.

Retained metadata-body.txt, metadata-helpers.txt, metadata-accessor.txt,
metadata-count-init.txt, metadata-slots.txt and metadata-data.json reproduce
the positive trace. Full REA record is rea-bool-metadata.json.
