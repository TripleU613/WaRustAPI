# Superpack archive metadata — APK 2.26.43.72

**Confirmed static / derived metadata interpretation**, pinned ARM64 carrier and
decoder. No native APK code executed, decoded libraries recovered, or live traffic
sent. Local Rizin output is not a REA Evidence envelope.

APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Carrier `lib/arm64-v8a/libs.so` SHA256
`1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5`.
Decoder `lib/arm64-v8a/libsuperpack.so` SHA256
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.

## Structure-backed inventory

The [from-so handoff](../2.26.43.72-superpack-from-so/README.md) and
[member-output core](../2.26.43.72-superpack-member-output/README.md) lead to
0x1c7ac. Its full selected body, through 0x1cc8c exclusive, is retained here.
Helper 0x1d558 compares the first eight bytes against little-endian numeric
constant 0x05abaf0150c00100 and returns 3 on mismatch. Helper 0x1d57c checks
the word at archive offset 8 using a bitfield-normalized comparison with 0x81;
the pinned raw word is exactly 0x81 and passes. These checks are not an integrity
or authenticity proof. The initial reads' input-length safety is not established.

At 0x1c8dc the parser advances by 28 bytes and reduces the supplied length by
28. It passes that remaining region to 0x2d560. This helper requires at least
28 bytes, copies a 28-byte fixed structure, and reads an unaligned u32 at
structure offset 2. For nonzero count it allocates count*4 bytes, verifies that
28+count*4 fits the input and copies that array. It reports consumed bytes as
28+count*4. The pinned count is zero, so the next cursor is archive offset 56.
No semantic names are assigned to this auxiliary structure's other fields.

The member count comes from archive offset 12: **83**. Starting at that cursor,
0x1c988..0x1cacc parses each record as:

- A NUL-terminated name, using strnlen with bound 0x1000 before testing the terminator.
- A NUL-terminated type, using strnlen with bound 8 before testing the terminator.
- Two unaligned 64-bit fields immediately after the type terminator.

The record cursor advances by name length+1, raw type length+1 and 16 bytes.
An empty type reuses the previous type when one exists (0x1ca20..0x1ca88).
The first record specifies `so`; all remaining pinned raw types are empty and
inherit `so`. The first 64-bit field is stored at member offset 0x18 and the
[writer's length accessor](../2.26.43.72-superpack-member-output/README.md) reads
that offset; it is recorded as declared_size. The second is retained as
opaque_u64: its meaning and verification are still unknown.

The resulting complete metadata inventory includes `libwhatsapp.so` at ordinal
63 with declared size 12783016 and `libwhatsappmerged.so` at ordinal 64 with
declared size 7871544. These are structure-derived records, improving on prior
raw string candidates. They are **not** recovered ELF images, validated decoded
sizes, or attribution of any JNI implementation to a particular library.
The metadata ends at archive offset 0xc73 (carrier file offset 0x14e3).
That is where this path passes the remaining payload to transform initialization;
it is not a proven XZ boundary or independently decodable member boundary.

## Transform boundary and limits

The path resolves literal `spo` through 0x1dbec and calls 0x1dc9c with remaining
payload and raw setting 0x40000. It can apply configuration through 0x1e094 and
then selects helpers 0x2e368 or 0x2e638 based on transform state. Full meanings,
checks, decompression and subsequent member reconstruction remain untraced.
The end argument also limits retained metadata entries: positive end caps the
retained count, while all records are still scanned. This supplements the
outer core's ordinal selection; no claim is made about decoded ordering across
all transform modes. The no-range path uses end -1 and retains all 83 records.

inspect-metadata.py intentionally inspects only the pinned carrier. It verifies
APK and both library hashes before parsing; bounds, ASCII and fixed pinned
header assertions constrain this research reproduction. It is not a generic
untrusted archive reader or faithful implementation of all native malformed-input
behavior. It does not write member files or execute target code.

## Reproduction

Run `python3 inspect-metadata.py /path/to/WhatsApp.1.apk`; stdout must match
metadata.json exactly. Member archive/carrier offsets, raw/effective types, sizes
and opaque fields are retained for all 83 entries. The inspector reads the APK
ZIP members directly; no extracted carrier is required.

For assembly use `rizin -2 -q -e scr.color=0 -c 'pD SIZE @ ADDRESS; q' FILE`
with ranges.json values and verified decoder. Selected .text VA equals file
offset in the [section map](../2.26.43.72-superpack-decoder-entry/elf-sections.txt).
[Tool version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) is retained.
checksums.json covers the bundle. Next establish the `spo` transform and member
reconstruction, then inspect native metrics schema, storage and network behavior.
No Rust runtime, route, sender, telemetry or live account operation changed.
