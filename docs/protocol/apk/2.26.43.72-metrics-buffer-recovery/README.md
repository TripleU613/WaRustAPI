# Metrics buffer file loader — APK 2.26.43.72

**Confirmed static / bounded native header admission.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
reconstructed ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction](../2.26.43.72-superpack-reconstructed-members/README.md).
No directory, real file, account, network or app was accessed/executed by the
fixtures. Local Rizin/Unicorn exports are not REA Evidence envelopes.

## Locate the loader separately from bulk deletion

The outgoing loader entry **0x726660..0x7266d0** unpacks a storage context and
calls **0x6fe1a8(store, raw1, callback)**. The complete selected loader
**0x6fe1a8..0x6fe628** requires its store queue context, creates a dictionary and
opens the directory at store+0x28. Raw0 selects filename prefix `wamt-a`;
nonzero selects **`wamt-o`**. It processes only entries whose dirent byte+0x12
is **8** and whose first **six** name bytes equal the selected prefix. No stat
fallback for unknown d_type, complete numeric filename parsing, filename/header
identity comparison or terminal-outcome marker lookup appears in this body.
This is a bounded statement about this loader, not all storage code.

A nearby routine **0x7266d0..0x7268f8** is **bulk deletion**, not recovery:
it closes handles through0x6fdfb0, clears store dictionary+0x40 and attempts to
remove regular-file entries matching either six-byte prefix. Its activation and
owner policy are not traced here. It must not be assumed to run on every
startup or after a terminal transport result. The selected bodies are retained
separately to prevent confusing a directory walk with recovery admission.

## File reading and rejection have distinct effects

The loader builds `%s/%s` from the root and full directory-entry name, creates a
file URL and opens through0x733ce0 with raw mode0 (read mode established in the
[metadata file trace](../2.26.43.72-metrics-native-metadata-file/README.md)). It
requests **store+0x30 +24** bytes via0x734174; the header-size helper0x72c4f0
returns24. Complete producer limits for store+0x30 remain open.

- Open failure logs and continues; this branch does not remove the file.
- Nonzero read result attempts remove, releases returned data and zeros it.
- Opened handles are closed through0x734448; its return value is not used for
  an acceptance branch in this body.
- Missing returned data is skipped. Rejected nonnull header/data attempts remove
  by the original filename, then releases the data. These remove results are
  not checked or retried here.
- If opendir fails, the loader reaches completion with the newly empty dictionary
  and a null error argument. This selected path does not distinguish that from
  an empty scan through its error argument; broader diagnostics remain separate.

No actual I/O or any of these side effects was executed in the fixtures below.

## Header and payload admission

The selected native checks are:

1. Returned data length must be at least **24**, including for legacy version1.
2. Bytes0..2 must be ASCII **WMT**. Version byte3 is initially bounded <=3;
   a later branch rejects0, leaving accepted versions **1,2,3**.
3. Offset4 carries an unvalidated8-byte timestamp value; offset12 a32-bit field;
   offset16 a32-bit **signed positive** declared payload length.
4. For version1 the header size is **20**, with checksum/state initialized by
   `adler32(0, null, 0)` (=1). Versions2/3 use header size **24** and restore
   the32-bit checksum/state at offset20.
5. Data length minus declared payload length must be at least that header size.
   Extra trailing bytes are allowed; only the declared payload is copied.

The selected path does **not** recompute/compare a checksum over recovered
payload bytes. It restores the32-bit checksum/state for versions2/3. The role of
that state in later serialization/integrity checks remains untraced; this is not
proof of a globally absent integrity check or permission to bypass one.

Accepted data yields a no-copy payload view and then a mutable copy. A buffer
object restores timestamp+0x20, the offset12 field at+0x28, zeroes+0x2c, stores
checksum/state+0x30 and version+0x34, and retains payload+0x18. The loader derives
a descriptor from that buffer and inserts descriptor→buffer into the result
dictionary. It does not create network requests or wait for transport outcomes.
The callback receives dictionary, raw mode and null error directly or through
a queued adapter according to the callback queue context. Dictionary insertion
is not proof the transport later admitted or sent its entries.

## Consequence for terminal and failed-delete files

The [completion/deletion chain](../2.26.43.72-metrics-buffer-deletion/README.md)
showed that a selected terminal regular outcome can remove an in-memory task
without scheduling that file's deletion; failed remove can also leave a file.
This loader's admission predicate has no previous terminal outcome input.
**Inferred conditionally:** a remaining matching regular file can pass these
local header gates if this loader is invoked and its bytes are otherwise valid.
Automatic restart replay is **not established**: full startup invocation,
callback consumers, bulk-delete activation and later admission remain open.
Do not translate this finding into a generic Rust retry after terminal refusal.

## Reproduction and scope

```sh
python inspect-recovery.py /path/to/libwhatsapp.so
python check-headers.py /path/to/libwhatsapp.so
```

Compare outputs with `recovery-boundaries.json` and `header-results.json`.
The inspector hashes selected complete bodies/helpers, resolves prefixes/path
format and lists direct BL callers of the loader in executable ELF sections.
It does not enumerate arbitrary indirect callers. Reproduce exports with
`rizin -2 -q -e scr.color=0 -c 'pD SIZE @ VA; q' IMAGE` and windows.json.

The checker requires Unicorn2.1.4 and executes **0x6fe34c** through acceptance
0x6fe498 or rejection0x6fe428. MCFData length/byte/pointer operations and the
initial adler32 call are controlled adapters. **18 cases** cover versions0..4,
short data, each bad magic byte, zero/negative/oversized payload, truncated
version2, trailing bytes, version1's distinct minimum/header size and two
arbitrary stored checksum values. Accepted cases assert restored checksum/state,
payload length and header size. The directory walk, reads, payload copying,
object creation, removal, callbacks and real checksum library are not executed.
The surrounding lifecycle/I/O claims above remain static branch conclusions.

**Observed live:** none. **Unknown:** full header producers, filesystem/read
semantics, checksum/state consumers, startup/terminal cleanup policy, downstream
callback/bookkeeping and actual replay/durability. No Rust/live behavior added.

## Consumer follow-up

The [bound callback consumer](../2.26.43.72-metrics-recovery-consumer/README.md)
now establishes the later version3 payload checksum comparison and selected
conditional task handoff. The absence of a checksum comparison in the file
reader is not a global absence. Startup flag producers, complete lifetimes and
actual replay remain open.
