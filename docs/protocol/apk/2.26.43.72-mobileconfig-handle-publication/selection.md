# Disk candidate selection uses numeric filename prefixes

**Confirmed selected static evidence**, same pinned APK/native member as README.
No execution of the target, filesystem experiment or live account operation.

The 0xc310c selector initializes an output string empty and its selected signed
32-bit index to **-1**, then visits the directory via 0xc2530. Its callable
object address point is 0xd6690; virtual+0x30 relocation **0xd66c0 -> 0xc326c**
binds the filename callback. Captures are base path (+8), index pointer (+0x10)
and output-string pointer (+0x18).

The callback extracts the filename's pointer/length through 0x5b8d8 and checks
suffix **`.mctable`**, eight bytes, through 0xc3444 -> 0x9e2c8. That helper
compares bytes backwards at the end; it is a suffix test, not substring search.
It finds the **first dot** from position zero and copies the prefix before it.
Empty prefix is skipped. It constructs **base path + prefix + `.meta`** and
requires existence-like helper 0xc2148 to return true. No regular-file/content
check is established by this gate.

For a qualifying prefix it calls 0x81a24 with pointer, strlen, default **-1**
and radix **10** (outlined 0xcc420 and 0xcb840). The wrapper around 0x81b18
requires full consumption and error code zero; invalid/range/error paths return
the supplied default. Its signed conversion handles a leading minus separately.
The full parser's digit-loop/overflow implementation is not re-derived here.
Do not infer that every successful prefix has a canonical decimal spelling.

The callback updates only when parsed signed index is **strictly greater** than
the current selected index (`cmp w0,w9; b.le` at 0xc3360/64). It first stores
that index and then assigns **base path + the full original filename** to the
output string. Thus the metadata counterpart is derived from the prefix, while
the selected table path retains the complete name. An intermediate dot is not
rejected by this callback itself; for example the structural tests on
`12.extra.mctable` use prefix `12` and metadata path `base + 12.meta`.
This is a static branch consequence, not a filesystem replay.

Starting at -1 excludes negative parsed indexes and the error default from
selection. Equal indexes do not replace the first qualifying entry encountered.
The retained 0xc2530 loop uses filesystem directory_iterator, extracts filenames,
dispatches each callback, then increments; no sorting or timestamp access occurs
in that selected loop. Therefore this selected policy means greatest qualifying
numeric index, with equal-index outcome dependent on iterator order. It does
not mean greatest modification time, newest valid body or complete pair.

If nothing updates, the output remains empty with index -1; reload exits on the
empty string before snapshot/hash handling. No table parser/body verifier runs
inside this selector. Validation occurs elsewhere in the traced reader paths.

**Unknown:** enumeration exceptions/completeness, concurrent directory mutation,
all filename-parser edge cases and downstream translation/activation. No Rust
runtime correction or server-mandatory field follows from this evidence alone.

Reproduce using retained selection-callback.txt, selection-suffix.txt,
selection-integer.txt, selection-directory.txt, selection-helpers.txt,
selection-slots.txt and selection-data.json. REA evidence is recorded in
rea-selection.json and covers the metadata gate and maximum-index update.
