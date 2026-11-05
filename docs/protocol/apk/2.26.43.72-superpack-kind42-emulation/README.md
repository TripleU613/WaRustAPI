# Type-42 decoder: bounded ARM64 emulation — APK 2.26.43.72

**Observed in a constrained emulator**, with static entry mapping. This bundle
changes the earlier static-only boundary: selected instructions from the pinned
APK decoder were executed by Unicorn 2.1.4, with host-defined memory helpers.
No Android process, JNI initialization, APK native library loading, device,
account, network or target filesystem operation was used. This is not a REA
Evidence envelope or a real-device observation.

Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 decoder SHA256:
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.
ARM64 carrier SHA256:
`1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5`.
The harness verifies all three hashes before extracting a selected payload and
checks its digest against the [framed block inventory](../2.26.43.72-superpack-transform-blocks/blocks.json).

## Entry and experiment

The [worker's type-42 branch](../2.26.43.72-superpack-transform-blocks/README.md)
calls 0x218bc with encoded pointer, output pointer, null third argument and
output-size limit. The retained complete selected entry body runs through
0x21a18 exclusive. It parses state via 0x276a8 and branches based on state +0x20:
1 calls 0x2a79c, 7/9 call 0x1fcf0, and other cases choose 0x1f4f0 or 0x20dd4
according to the state byte at offset zero. A state pointer at +0x10 can cause
temporary output allocation and a subsequent offset copy. The complete codec,
header grammar and all mode semantics have not been reconstructed in Rust.

The harness maps ELF PT_LOAD regions and applies RELATIVE relocations. Imported
symbols resolve to controlled stubs, not host dlopen. Only allocator, memory and
log operations are handled; other called imports stop emulation. Target
interrupts also stop. The selected helper gets a mapped input, separate output,
stack, heap and a return sentinel. Each run has a 45-second engine timeout and
10-billion-instruction cap; success requires returning to the sentinel and an
exact match with the framed declared output length before saving candidate bytes.
A bounded stop is not a decoder result.

Allocation uses a capped monotonic emulator heap; free is a no-op. Pointer reuse,
allocator failures and Android ABI/runtime effects are not faithfully modeled.
memset/memcpy are simple ARM64 byte loops in emulator memory, with source assembly
retained. Other memory helpers are Python callbacks. The helper code is embedded
as 52 bytes; its two entries are +0 and +24. No target I/O APIs are provided.
The operator-specified output file is written by the harness only after success;
use ignored research storage. This is a pinned research harness, not a general
safe decoder for arbitrary untrusted libraries or archives.

## Observed candidates and limits

Three selected type-42 blocks returned successfully:

- Ordinal 0: 17778932 bytes, SHA256
  `eb63ece20b7aba4b2c69e57edc83a44ba56ffbf163d017742385216409a91f43`.
- Ordinal 168: 4038 bytes, SHA256
  `6ea28e58af9ba0554d31b9fa1efc72777168412cdc7216bf9c894db4c291d1f5`.
- Ordinal 201: 58 bytes, SHA256
  `422e665faa1f57716c9d30b6867c976456cff73e6f21fed262c5bca8cc9ac2b7`.

Ordinal 201 produced identical length/digest using the original Python-only
memory callbacks and the faster emulated memory loops. Ordinal 0 was rerun from
the retained harness and reproduced byte-for-byte. This supports the selected
experiments, not general emulator equivalence or codec correctness.
Initial ordinal-0 attempts stopped before the sentinel under the lower
one-billion-instruction cap; bounded-stops.json retains those limits and PCs.
The first Python-callback attempt also had the same 45-second timeout, so its
specific limiting mechanism is not asserted. No output was saved from either.

None of these candidate buffers starts with ELF magic. Exact member stitching,
post-transform reconstruction, payload checksums and attribution to a library
remain unverified. Returning the declared size is insufficient to claim usable
native libraries. The other 70 type-42 blocks have not been emulated in this
bundle. Native metrics schema, storage and transport are still open.

## Reproduction

Install `unicorn==2.1.4` in an isolated research venv. Run
`python emulate-block.py PINNED.apk ORDINAL IGNORED_OUTPUT.bin` with ordinals
0, 168 or 201 and compare digests to decoded-candidates.json. The adjacent
framed inventory is resolved relative to the script. execution-results.txt
retains successful rerun stdout. Raw candidate bodies stay outside Git.
To verify memory-helper bytes, compile emulated-memory.s with
`clang --target=aarch64-linux-gnu -c emulated-memory.s -o MEMORY.o` and compare
its .text bytes with the embedded mem_write bytes. environment.txt records the
execution setup; range.json binds the selected decoder-entry assembly to bytes.
[Section map](../2.26.43.72-superpack-decoder-entry/elf-sections.txt) and
[Rizin version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) remain retained.

Next decode remaining framed payloads and trace the block-to-member assembly
and format transforms. No Rust client behavior, HTTP route, telemetry, sender,
retry, live account operation or network traffic changed.
