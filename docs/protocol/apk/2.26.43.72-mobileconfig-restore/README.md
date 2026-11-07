# MobileConfig backup restoration and load continuation

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static inspection only; no restore execution, filesystem experiment or account I/O.

## Selected storage method contains a gated restore subpath

RELA 0xd5598 binds the previously resolved StorageManager vtable+0x18 to
**0x834c8**. This report traces its restore subpath and later load continuation;
it does **not** establish every caller or that the method necessarily runs at
startup. A search of the local complete disassembly found no direct BL to that
entry, which does not exclude indirect callers.

The method acquires manager+0x2d0 mutex. At 0x836cc, manager byte+0x120 bit 0
clear and byte+0x12d equal to 2 select the restore attempt. It first calls
**0x8cee8** with diagnostic `restore_from_backup`. That helper can perform
additional external/state work and calls cleanup helper **0xc2420** on the
current base directory. Its return is the cleanup result's low bit. Therefore
cleanup is attempted **before** checking the backup base and `DONE` marker in
this selected path. The exact current-directory contents, cleanup listing and
external side effects remain unproven; do not treat this as a safe public
restore operation for arbitrary paths.

After successful pre-cleanup, it obtains the `_backup/` base through the
[backup helper](../2.26.43.72-mobileconfig-backup/README.md), requires it to be
nonempty, and tests the joined `DONE` path with filesystem-status helper
0xc2148. This is the previously retained existence-like predicate (status byte
neither 0 nor 255), not validation of marker contents, regular-file kind or
backup completeness. It then ensures the destination base through 0xc28ac.

## Directory callback and copy latch

The method allocates a callback with address point **0xd5a90**, capturing a
shared local success byte (initially 1), backup base, destination base and
manager. RELA **0xd5ac0** binds callback slot+0x30 to **0x8d550**. Its type
string at 0x22b89 is
`ZN12mobileconfig28FBMobileConfigStorageManager14copyFromBackupEvE3$_0`.
That identifies the callback's copyFromBackup context; it is not a standalone
proof of startup scheduling.

At 0x837b8, helper 0xc2530 creates a filesystem directory_iterator over the
backup base. For each item it gets `path::__filename`, creates a string and
invokes the callback through slot+0x30. It increments the iterator even after
the callback's local success byte has become false. Ordering is supplied by
the filesystem iterator; no stable sort is shown.

The callback's string helper 0x70a28 performs length and byte equality (via
0x5df14, 0x5d9f8/memcmp and result conversion). Names exactly **`.`**, **`..`**
or **`DONE`** return without changing the latch. Other names are copied only
while the latch's low bit is set:

- source = captured backup base + filename;
- destination = captured current base + filename;
- call the previously retained **0xc1f4c** copy helper;
- replace the latch with that helper's low-bit result.

When the latch is already false, subsequent non-skipped callbacks leave it
false and do not attempt another copy. Earlier copied files are not rolled back
in this callback. It has no extension allowlist or per-file manifest check in
the retained body. Directory entry kinds and actual backup contents are unknown.
The copy helper's unchecked write/close results remain relevant: a true latch
does not prove restored bytes are complete or durable.

If iteration completes normally without a qualifying entry, the initialized
latch can remain true. A marker-only directory therefore does not itself prove
a complete restored set. This is a control-flow projection, not an executed
empty-directory test; iterator exceptions and imported filesystem behavior are
separate.

## Local recovery status is followed by loading

| Local recovery code | Selected source |
| --- | --- |
| 0 | Iteration returned with copy latch true |
| 1 | Backup base empty |
| 2 | `DONE` status predicate false |
| 3 | Destination-directory helper false |
| 4 | Iteration returned with copy latch false |
| 5 | Pre-restore cleanup helper false |

These codes are reported with `instacrash_recovery_from_backup` and
`recover_status`. They are not HTTP statuses, final application results or
proof of activation. Every selected normal recovery-code path joins the later
continuation, which clears a diagnostic string and chooses subsequent loading
based on manager flags. The method can call virtual slot+0x90 or +0x98, or
select numeric byte 8 through a flag path, and stores the selected result at
manager+0x318. For the selected vtable, those slots bind to 0x84a24/0x85060;
their bodies, validation and loaded-state admission remain untraced here.
Further callbacks and mutex release follow. Recovery-code 0 is consequently
not equivalent to complete loading or server acceptance.

## Reproduction and coverage

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-restore/inspect-restore.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-restore.json
cmp /tmp/mc-restore.json docs/protocol/apk/2.26.43.72-mobileconfig-restore/restore.json
```

The inspector validates pinned member/ELF identity, strings, vtable addends and
selected direct calls. Its supplied-order callback examples are analyst
projections, not native tests. LLVM retains method, callback, iteration,
pre-cleanup and comparison windows. Nearest JNI_OnLoad headings are not local
identities; all imported filesystem behavior and exception paths are not modeled.

rea-restore.json retains complete REA Evidence
`ev_14a387d4dfa8b78d96198c58556315ed9e6be875bef423f12fed2e7f860db7a0`,
command `e scr.color=0; pd 40 @ 0x8d574`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. It is a bounded callback window;
provider version and external inputs are not bound, and annotations are not
runtime memory observations.

**Remaining:** startup/other caller binding, flag producers, pre-cleanup external
effects, directory/imported-I/O exceptions, verified restored bytes and complete
load/activation admission, timing/retry/account ownership. No Rust runtime or
saved account state changed.
