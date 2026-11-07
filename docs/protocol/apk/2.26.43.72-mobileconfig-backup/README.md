# MobileConfig backup preparation and continuation

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No target execution, copy experiment, network request or private-state mutation.

## Flag-1 branch prepares a backup before the two writes

In storage method 0x8674c, manager byte +0x2a4 equal to 1 enters 0x86814
under manager+0x2d0 mutex. This branch is **backup preparation**, not a proved
alternative atomic write implementation. Helper 0x8b810 derives a path from
the same base helper, appends `_backup` and a trailing slash. The selected
normal backup path checks existing state, handles a `DONE` path, attempts
cleanup as applicable, and calls 0xc28ac. The latter uses filesystem status
and, when needed, imported `__create_directories`; it returns according to
its error code. Exact runtime paths and prerequisite-helper semantics remain
unknown. Helper 0xc228c compares the filesystem-status byte to numeric 2;
the report does not assign that value a kind from an unverified enum mapping.

The branch obtains a string from manager virtual slot +0xe0, derives a sibling
using the previously retained `.mctable` to `.meta` helper, and uses 0x8b8bc to
select a third source. That selector examines candidate path/status and string
comparisons, with diagnostics on a mismatch. Its complete candidate producers,
file roles and fallback semantics remain untraced. Each source is paired with
a destination made from the backup directory and basename helper 0xc2da0.
It calls **0xc1f4c** three times, in order, at **0x86edc, 0x86f0c, 0x86f3c**.
These calls copy existing selected files; they do not write the new response
bodies through the framing writer.

Only after those helpers return true does it create the backup `DONE` path
through 0x79dfc at 0x86f80. The supplied string is **empty**, from literal
0x25876; `DONE` is a filename component, not text written into the file.
The adapter passes pointer/length to 0xc1a9c, which opens the path with `wb`,
skips fwrite for zero length, closes, and compares write count to requested
length. The close return is not checked. Thus marker-helper true alone does
not prove durable backup contents.

## Backup codes do not stop the later write attempt

Selected local backup outcomes are distinct from the outer application result:

| Local code | Selected source |
| --- | --- |
| 0 | Marker helper true |
| 1 | Selected missing state/path/lookup prerequisites |
| 2 | Empty backup base |
| 3 | Selected cleanup failure |
| 4 | Directory helper false |
| 5 / 6 / 7 | First / second / third copy false |
| 8 | Marker helper false |

At 0x86fec, codes below 2 branch directly to **0x8706c**. Codes 2 and above
log `CacheBackupError` / `Failed to backup cache, result: ` and then also
continue to **0x8706c**. Therefore a backup-preparation failure is not a
hard refusal of the new write in these selected normal branches. The earlier
[two-file writer evidence](../2.26.43.72-mobileconfig-file-writer/README.md)
applies after this preparation as well as when manager byte +0x2a4 is not 1.
Exceptions and prerequisite availability/null-input returns are separate.

## Copy helper success is weaker than verified copying

Function 0xc1f4c opens source with `rb` and destination with `wb`. It loops
until source `feof` is nonzero, calling fread with item size 1/count 4096,
then fwrite with item size 1/count equal to fread's returned count. In the
retained whole function, **fwrite return values are not checked**, there is
no explicit ferror check, and both fclose returns are ignored before returning
true. This is not a rename primitive, an atomic copy, or a byte-verified backup.
Read-error termination and short-write handling are not demonstrated.

Failure to open either file returns false, optionally logging `FileCopyError`
with `copyFile`. On the destination-open failure path, the selected normal
continuation has no source fclose call before returning false; this is a
bounded static lifetime finding, not an observed descriptor leak experiment.
The three backup calls supply a null logger, so optional helper logging does
not establish that every copy failure is logged inside the helper; the caller's
backup-code diagnostic remains separate.

The cleanup helper 0xc2420 invokes a directory-listing helper and then removes
constructed child paths through 0xc2310, stopping further removal attempts after
a false removal result. Full listing/filter semantics and restoration consumers
remain unknown. No complete rollback, prior-file restoration, restart recovery,
atomic replacement or crash-durability claim follows from this branch.
These limitations are evidence to guide a reliable Rust design, not instructions
to reproduce unchecked I/O behavior.

## Reproduction and retained evidence

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-backup/inspect-backup.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-backup.json
cmp /tmp/mc-backup.json docs/protocol/apk/2.26.43.72-mobileconfig-backup/backup.json
```

The inspector pins member/ELF identity, literals and selected branch targets.
Outcome labels are analyst transcriptions, not executed backup or recovery tests.
LLVM retains backup control flow, full copy/marker helpers and bounded path,
selection, status, directory and cleanup windows. Nearest JNI_OnLoad headings
are not local function identities. Exceptions and all called helpers are not
fully modeled.

rea-copy.json retains complete REA Evidence
`ev_302eaec470724c9f708dcd21495c2687a006c69eef362ffe50fb8caf0775d4a5`,
command `e scr.color=0; pd 36 @ 0xc1fd4`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. Its window covers the loop and part of
open-error handling; LLVM retains the full helper. Provider version and external
inputs are not bound, and annotations are not runtime memory observations.

**Remaining:** backup source and directory producers, complete cleanup/filtering,
restore/startup consumer, new payload formats, activation/result callbacks,
retry/account ownership and crash durability. No Rust runtime or saved account
state changed.
