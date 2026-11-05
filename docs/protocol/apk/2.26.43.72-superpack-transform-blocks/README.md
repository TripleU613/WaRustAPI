# Superpack spo descriptor and block framing — APK 2.26.43.72

**Confirmed static / derived block framing**, not decoded libraries, runtime
observation or REA Evidence envelopes. No target native code was loaded or run.
The digest-bound inspector reuses the [metadata inspector](../2.26.43.72-superpack-metadata/inspect-metadata.py)
for APK/member identity checks and the metadata-derived payload cursor.

Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 carrier SHA256:
`1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5`.
ARM64 decoder SHA256:
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.

## spo dispatch is not the standalone XZ descriptor

At 0x1dbec the resolver compares storage names and, for exact literal `spo`,
loads slot VA 0x383c0. ELF RELA R_AARCH64_RELATIVE (1027, symbol index zero)
resolves that slot to descriptor VA 0x383e0. File mappings and relocated callback
addresses are retained in blocks.json. Descriptor +0x55 contains `spo`.
The generic initializer 0x1dc9c, with nonnull payload, invokes descriptor +8
with payload pointer and length. For this descriptor that is 0x1e0d8: it allocates
state, stores the memory region and initializes a cursor. Other mapped callbacks
include block reader 0x1e130 (+0x10), settings 0x1eb60 (+0x28), and worker entry
0x1ebe4 is supplied to pthread_create by the block reader. Callback purposes
not otherwise traced remain addresses, not invented API names.

The standalone `xz` resolver branch uses a different descriptor at VA 0x38500.
The outer `spo` container may itself dispatch particular encoded blocks to XZ;
these are separate layers. No complete archive follows from scanning XZ magic.

## Exact payload framing

The [archive parser](../2.26.43.72-superpack-metadata/README.md) reaches payload
archive offset 0xc73 after 83 member metadata records. It supplies the archive
word at offset 0x18 to setting 1. Settings callback 0x1eb60 stores that value at
state +0xc0; the block-reader loop compares its block counter to that field.
The pinned value is **213**.

In the memory-backed path, 0x1e308 checks and copies a **42-byte header**, advances
the cursor, and reads a u32 count at header +38. It bounds and consumes count*4
auxiliary bytes. It then reads encoded byte count at header +20, bounds the
encoded payload against the memory region and advances by that count.
Header +12 supplies the output-size value; +28 supplies the kind byte, and
+0 supplies a block identifier used in output state selection. The exact
identifier-to-member mapping is still untraced; block IDs are not assumed to be
member ordinals or contiguous IDs. Auxiliary-table meaning and content hashes'
algorithm are also still unknown.

The inspector follows exactly this framed cursor progression and bounds-checks
each header, auxiliary region and payload. All **213** records fit, and the final
cursor is **0xdb6641**, exactly the previously measured archive length. Therefore
these records account for the entire remaining byte range, with no trailing
bytes under this pinned layout. This does not prove encoded-body integrity,
successful transform execution or reconstruction of the declared output.

Of these records **140 have kind 1**, and every such payload starts with XZ magic
`fd377a585a00`. The other **73 have kind 42**. Their payload prefixes and SHA256
are retained; their complete decoding remains open. The worker's kind-1 branch
at 0x1edf4 initializes helper 0x2bb08 and loops through 0x2b3d8, whereas kind 42
at 0x1ed80 calls **0x218bc**. This is a concrete second decoder target, not a
claim that it implements standard XZ. The old bounded XZ scan's 140 candidates
now have structure-backed block boundaries; those earlier partial decoded
outputs still must not be presented as complete ELF libraries.

The block reader also has worker queues, configuration-dependent limits and
reference-input branches. Complete scheduling, every failure path, checksums,
partial-range decoding, output stitching and post-transform ELF reconstruction
are not established by this framing inventory. Full selected reader/worker
windows are retained to continue that trace without losing branch context.

## Reproduction and next proof

Run `python3 inspect-blocks.py /path/to/WhatsApp.1.apk` and compare stdout with
blocks.json. It imports the adjacent metadata inspector by a path relative to
its own file; invocation accepts the same single APK path. It verifies pinned
hashes before interpreting data. It performs no decompression or target code
execution. Per-block raw header bytes, offsets, auxiliary digest, encoded-body
length/digest/prefix and declared output size are retained.

Assembly exports use `rizin -2 -q -e scr.color=0 -c 'pD SIZE @ ADDRESS; q' FILE`
with ranges.json. Selected text VA equals file offset in the [retained section
map](../2.26.43.72-superpack-decoder-entry/elf-sections.txt); descriptor VAs use
actual ELF section mappings and relocations. [Rizin version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt)
and [full relocation records](../2.26.43.72-superpack-jni-registration/elf-relocations.txt)
are already retained. Bundle checksums cover all local evidence.

Next trace kind-42 helper 0x218bc and decoded block/member assembly, then recover
native metrics schema, persistence and network behavior. No Rust behavior,
HTTP route, telemetry, sender, retry or live account/network operation changed.
