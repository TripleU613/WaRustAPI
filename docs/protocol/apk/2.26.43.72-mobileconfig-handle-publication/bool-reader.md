# Selected raw boolean-data reader and bounded sentinel search

**Confirmed selected static evidence**, same pinned APK/native member as README.
No target execution, live account operation or Rust runtime change.

Reader vptr 0xd61c0 virtual+0x20, RELA **0xd61e0 -> 0xa351c**, identifies its
diagnostic method as **FBMobileConfigValueStore::getBoolData** (0x1af54).
Its input index is retained from w1. The selected raw-byte getter:

- defaults if root pointer+0x50 is null or root vector field is absent;
- defaults when unsigned index is **>= reader word+0x58**;
- otherwise reads the root vector's u32 count and, when index is below it,
  returns **the raw byte at vector+4+index**;
- when index is below cached word+0x58 but not below the actual vector count,
  optionally logs `DiskCorruptionError` (0x17596), then defaults.

Vector accessor 0x96c04 selects vtable entry 6 through 0x9ac18. This is an
optional relative-offset accessor, not a full body verifier. The default path
at 0xa3694 addresses **rodata 0x23797**, whose pinned byte is **0x11 (17)**,
then loads that byte into w0 at 0xa369c. The selected getter does not convert
its raw result to a language boolean. In particular, do not translate this
default to Rust `false`, or treat raw nonzero as `true`, without tracing the
consumer's encoding semantics. The public meaning of 0x11 is unresolved here.

There is no explicit comparison with 0xffff in this body. If a hypothetical
caller passed raw index 65535, the count comparisons would decide the branch;
that value alone is not a universal rejection rule. **Unknown:** whether any
caller extracts a translator's low-16 sentinel and routes it into this method.
This getter does not itself invoke the translator.

## Bounded negative search

The complete local LLVM text disassembly of libmobileconfig-jni.so was searched
for literal +0xa0/+0x98 accesses and direct calls to 0xa86cc. Selected reader
vtable entries and getter bodies were inspected; this did not bind an end-to-end
sentinel consumer. It does not rule out adjusted receivers, computed offsets,
inlining, other members or Java/JNI routing.

A separate streamed LLVM disassembly of reconstructed libwhatsappmerged.so
(SHA256 d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d)
found **332** literal `ldr xN, [xN, #0xa0]`-shaped accesses (receiver registers
may differ). Filtering for a nearby virtual+0x10 load and indirect call left
windows around 0x5d74c4 and 0x5e3138; neither was bound to this reader type.
This instruction-pattern search is not type-aware call coverage and must not
be described as absence of consumers. Bulk output stays local.

Retained bool-reader-body.txt, bool-reader-accessor.txt, bool-reader-args.txt,
bool-reader-slots.txt and bool-reader-data.json reproduce the positive trace.
REA evidence is retained in rea-bool-reader.json. Next required work remains
typed caller binding and raw-value decoding, not guessing boolean defaults.
