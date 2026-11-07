# MobileConfig file writer and alternate two-file ordering

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, filesystem experiment, account mutation or network request.

## Actual writer, separate from metadata construction

Following the [storage handoff](../2.26.43.72-mobileconfig-hash-gates/README.md),
selected StorageManager vtable+0xb0 resolves through RELA 0xd5630 to **0x860fc**.
Its trace-section string is `FBMobileConfigStorageManager.writeToDiskWrapper`.
It first checks RequestData availability, then input body pointer +0x40 and
unsigned 32-bit body size +0x30. Null pointer or zero size takes a false-return
diagnostic path before opening a file. It does not derive size from a string.

The normal writer opens the supplied path with helper 0x98e40, supplied mode 4.
That wrapper ORs 0x10 into the mode and calls 0x990c0, which passes the selected
C-string path and numeric mode **0x14** to imported basic_filebuf::open. Opening
failure is handled separately. Runtime path contents and filesystem conditions
are not known from the static call.

On the selected successful writes, the file layout is:

| Offset | Bytes |
| --- | --- |
| 0 | `3c f1` magic |
| 2 | byte `01`, diagnostic names this file format version |
| 3 | byte `08`, diagnostic names this body offset |
| 4 | little-endian u32 of body size + 10 |
| 8 | exact supplied body pointer/size span |
| 8 + body size | `3c f1` magic footer |

The size calculation is a 32-bit addition; this selected writer has no separate
overflow rejection around that addition. This is a static arithmetic fact, not
permission to generate oversized inputs or a claim about upstream size bounds.
The framing is a local storage format, not a network message format.

After each of the six writes, the selected stream-state helper tests **mask 5**
and takes a diagnostic failure path when set. It also checks the stream's
vtable-relative offset before accessing state; the defensive range-failure path
traps. Errors distinguish header, version, body offset, size/body and footer
writes. At 0x86638 it explicitly closes through 0x9901c; that helper invokes
basic_filebuf::close and ORs numeric mask 4 through ios_base::clear when close
returns null. After close, the writer tests **bit 0**, not the earlier mask 5,
and returns true only on that selected clear-bit branch. False paths log
`WriteToDiskError`; some attempt close before cleanup. Exception cleanup is
retained but not given a complete operational model.

The routine has no explicit fsync/rename call in its retained body. Imported
stream and filesystem internals remain separate; successful stream close is not
proof of crash durability, atomic replacement, or server acceptance.

## Alternate storage path performs two ordered writes

This is the manager-byte+0x2a4 **not equal to 1** branch at 0x8706c, entered
under the previously acquired manager+0x2d0 mutex. It derives a first path via
0x88048 using a base helper, manager word +0x18 converted to decimal, and the
literal `.mctable`. Helper 0xc36c4 derives a sibling by replacing a matched
8-byte `.mctable` span with the 5-byte `.meta` literal; its no-match branch
returns an empty string. Exact base contents, match-selection semantics and
the roles of the two input bodies remain unproven.

1. At 0x87140 it calls the writer with the first owned input and first path.
   A false return skips the second write and proceeds with the initialized
   internal result (byte 8 in the earlier retained setup).
2. After a true return, it derives the sibling path and calls the same writer
   with the second owned input at 0x871ec.
3. A true second return sets local result first byte to **1** at 0x871f8.
   A false second return calls **0xc2310 on the first path** at 0x87208.
   That helper invokes imported filesystem `__remove` at 0xc234c. Its return is
   not tested here before continuation. Thus this is an attempted first-file
   cleanup, not a guaranteed deletion or complete rollback. It does not prove
   cleanup of a partially written second file or restoration of prior contents.

The local result is transferred into caller output by 0x80244 at 0x87224.
Later code has additional conditional work, an unconditional system_clock::now
call followed by helper 0x8aad0, further result checks/helpers, sets output byte
+1 to 6, invokes another virtual callback and unlocks the mutex. Their semantic
effects remain untraced. In particular, byte 1 alone does not prove completion
of all later activation work. The flag-1 storage branch and its path transitions
remain separate from this alternate path.

## Reproduction and evidence limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-file-writer/inspect-writer.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-writer.json
cmp /tmp/mc-writer.json docs/protocol/apk/2.26.43.72-mobileconfig-file-writer/writer.json
```

The inspector reuses pinned member/ELF validation, checks strings, magic bytes,
selected BLs and writer RELA binding. Synthetic frames are analyst projections,
not executions of the writer, parser acceptance or actual stored records. LLVM
retains the writer, alternate path, stream helpers, removal/sibling helpers and
selected follow-up entries. Nearest JNI_OnLoad headings are not local identities.

rea-writer.json retains complete REA Evidence
`ev_67e039238de48781f31aac3487c1465d12d888630015b2caf54464a35f9c024f`,
command `e scr.color=0; pd 30 @ 0x865d4`, exit 0, untruncated output, unchanged
member hash, with an entrypoint warning. The window ends during post-close state
setup; LLVM retains the subsequent branch. Provider version and external inputs
are not bound, and annotations are not runtime reads.

**Remaining:** input-body formats and ownership producers; exact runtime paths;
flag-1 branch; complete result meanings and follow-up callbacks; imported stream
effects, crash recovery, atomic replacement, durability, cache activation and
retry/account ownership. No Rust runtime or saved account state changed.
