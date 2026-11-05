# Reconstructed ARM64 native members — APK 2.26.43.72

**Observed in bounded ARM64 emulation, independently checked against pinned
metadata and ELF tooling.** All **83** packed ARM64 members were reconstructed,
totaling **65766648 bytes**. Every output matches its metadata-declared size and
the metadata u64 when computed as **XXH64 with seed zero**. Every file is ELF64
little-endian AArch64 ET_DYN and passes readelf header/section/program/dynamic
inspection without stderr. This is recovery for static analysis, not proof that
the libraries load or run on Android, or that the client protocol is correct.
No reconstructed library was loaded or invoked; only selected decoder/rebuilder
instructions ran in Unicorn. No account, network or device operation occurred.

APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Carrier SHA256 `1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5`.
Decoder SHA256 `fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.
These are local experiment results, not REA Evidence envelopes.

## Shared streams and reconstruction

The [previous stage](../2.26.43.72-superpack-all-blocks/README.md) produced 213
intermediate buffers. The harness verifies each against that retained SHA256
manifest and creates stream-vector state using native helpers 0x2ded0 and
0x2e33c. It supplies each decoded buffer to the stream indexed by its framed
block ID, with buffer/length/cursor fields derived from the retained stream
assembly. All pinned block IDs are unique. No assumption that block IDs equal
member ordinals is used.

It initializes the native driver's lookup state via 0xb488 and supplies the
28-byte auxiliary archive structure (zero auxiliary-array count) as configuration.
In the metadata's member order, helper **0x1adb8** receives configuration, shared
stream vector, output pointer, declared member size and a consumed-length pointer.
Successful calls advance shared stream cursors. The harness reproduces the outer
0x1af20 loop, advancing output by the reported consumed length until the member
size is reached. Every call must return zero to a sentinel within a 45-second /
10-billion-instruction bound; consumed length must be positive and fit remaining
output. No body is saved for a failed or incomplete member.

The harness does not emulate the entire APK loader or JNI. It explicitly supplies
already decoded streams and executes only the reconstruction path. Its allocator
uses capped emulator memory with a reusable free pool; realloc allocates and
copies, and does not reclaim the old block immediately. Memory operations are
controlled callbacks or retained ARM64 loops. qsort is an in-emulator heapsort
calling the target comparator. Unsupported imports and target interrupts stop
the run. No host target I/O services are supplied.

## Independent checks and exact boundaries

validation.json retains lengths, SHA256, XXH64 and layout checks for all 83 files.
verify-members.py re-extracts metadata from the hash-verified APK and independently
checks each output with the installed host libxxhash (version 0.8.4) and readelf.
The interpretation of the former opaque u64 as XXH64(seed=0) is supported by
**83/83 exact matches**; the native hash implementation itself has not been
fully identified. This is an empirical metadata/content check, not a claim of
APK-signature verification of separately reconstructed files.

The first member, libhiddenapis2.so, is 21984 bytes with SHA256
`9c92952268b54b4927d725bf1ad6f47c61d87202cc886b60f1bcc5fc06b7fab8`.
The first 37 outputs are byte-identical between initial insertion-sort and final
heapsort helper runs. The initial run stopped inside the insertion helper while
reconstructing ordinal 37; its bounds/PC are retained, and no incomplete member
was saved. The heapsort helper passes 18 independent cases comparing sorted keys
and record preservation against Python (including duplicates, zero/one item,
ascending/descending/random input, up to 1000 records). Its source and results
are retained. No general C-library equivalence is asserted.

Both libwhatsapp.so and libwhatsappmerged.so now exist as structurally checked,
metadata-matching files for analysis. Selected dynamic-symbol outputs retain
JNI_OnLoad addresses. Neither selected dynamic table exports a name containing
jvidispatch; this does not prove absence of those methods or native registration.
Full JNI mapping, native metrics schema/storage/transport and server-response
behavior still need investigation. None of this is a Rust behavior fix or a ban
prevention guarantee.

## Reproduction

With Python containing unicorn==2.1.4, from repository root run
`python reconstruct-members.py PINNED.apk 0 IGNORED_OUTPUT_DIRECTORY`.
The legacy zero argument selects a verified bootstrap block; it is not a member
selector. The script reads the adjacent evidence manifests and ignored
`../android-lab/output/metrics-native/framed-decoded/` buffers from the prior
stage. Keep all reconstructed binaries outside Git. execution-results.txt retains
the successful full run; reproduction-results.txt retains a fresh full run of
the retained script into a separate directory. Every resulting file was compared
byte-for-byte, then the independent verifier was rerun.

Run `python verify-members.py PINNED.apk RECONSTRUCTED_DIRECTORY` for the 83
metadata/hash/layout checks, and `python check-sort-helper.py` under the Unicorn
venv for the sort fixtures. Compile emulated-qsort.s with clang's AArch64 target
to check that .text matches the bytes embedded in the harness. The memory-loop
source is in the [earlier emulator bundle](../2.26.43.72-superpack-kind42-emulation/emulated-memory.s).
Assembly ranges and checksums bind the selected parser/stream helpers to the
pinned decoder; prior Rizin/ELF mapping evidence remains applicable.

Next inspect the recovered native JNI registration and metrics implementation,
then compare actual serialization, persistence and transport behavior with Rust.
No client runtime, HTTP route, sender, telemetry, retry or live traffic changed.
