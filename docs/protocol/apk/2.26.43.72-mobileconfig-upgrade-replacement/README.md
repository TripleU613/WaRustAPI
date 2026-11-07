# MobileConfig upgrade file replacement and local rename retries

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static inspection only; no file replacement, sleep, network or account execution.

## Upgrade writes are conditional, not one universal transaction

This follows the [upgrade dispatch](../2.26.43.72-mobileconfig-upgrade-dispatch/README.md)
method 0x85164 after two generated objects have been obtained. Their complete
construction and state-installation semantics remain untraced. Selected helper
arguments and rename diagnostics distinguish table and metadata replacement.

Manager byte **+0x29b equal to 1** selects direct final-path writes at
0x854b4/0x854cc: metadata first, table second, using the previously resolved
storage writer slot+0xb0. A false first writer skips the second. A false writer
reaches output reason byte **7** and false return. This selected direct branch
has no rename stage. It does not prove rollback of a successful metadata write
if the table write then fails.

Otherwise the method derives a temporary table path by appending **`.tmp`**
to its selected source string, and a sibling metadata temporary through
0xc36c4. The final table path is a separate supplied argument; its producer and
actual runtime contents remain unknown. It checks for existing temporary paths
and logs an overwrite diagnostic when applicable, then writes the temporary
metadata and table in that order at 0x85938/0x85950. Both writes must return true
before rename selection. Existing-path status is not a refusal by itself.

## Default and ordered rename paths differ

With manager byte **+0x298 not equal to 1**, the code calls **0xc2820** twice:
metadata temporary to metadata final at **0x85b1c**, then table temporary to
table final at **0x85b2c**. It attempts the second call **even when the first
returns false**, and requires both low-bit results true before success
continuation. These are single helper calls; no retry loop surrounds them here.

With manager byte **+0x298 equal to 1**, helper **0x8bb80** uses the low bit
of manager byte **+0x299** to select order and calls retry helper **0x8bc94**:

| Order flag | Calls and selected result |
| --- | --- |
| Set | Table first. First false -> 0, no metadata attempt. Table true -> metadata next; metadata false -> 2, true -> 3 |
| Clear | Metadata first. First false -> 0, no table attempt. Metadata true -> table next; table false -> 1, true -> 3 |

The caller admits only result **3** to the selected success continuation.
Results 0/1/2 enter rename-failure handling. These are local pair outcomes,
not HTTP statuses, server responses or active-configuration enums. The pair
helper does not reverse a successful first rename when the second fails.
Outer failure cleanup is separate and does not establish atomic pair rollback.

## Retry is bounded local filesystem behavior

Helper **0xc2820** constructs two filesystem paths, invokes imported
`filesystem::__rename`, and returns true when its error code is zero.
Retry helper 0x8bc94 repeatedly calls it on the same source/destination pair.
The normal initialized counter starts at −3, advances after every attempt and
exits at zero; there are at most **three rename attempts**. On failure it
sleeps **4, 8, then 16 milliseconds**, respectively. The last failure also
sleeps before returning false. A true result exits without that attempt's sleep.
The selected normal three-attempt path never reaches the large-duration
saturation edge also present in the generic duration arithmetic.

These sleeps and retries are **file rename policy**, not authorization or
evidence for retrying a network request, ACK or message. No network operation
occurs in these selected helpers. The host did not execute their sleep calls.

## Failure reporting and success continuation

Rename failure logs `SchemaUpgradeFailedToRenameFile`, `Unable to rename
flatbuffer`, and a table/meta-specific diagnostic, then joins the upgrade
failure continuation with output reason byte **7** and false return. The
previously traced outer method handles a false upgrade with cleanup and
`UpdateFailed`; complete recovery/restoration of prior contents remains unknown.

After both selected replacements succeed, 0x85b50 calls **0x8acd0**. That
helper builds candidate path collections, checks their status, attempts removal
of existing candidates and accumulates a false result on removal failures.
Its candidate-generation rules remain untraced. The caller logs
`upgrade_schema_delete_failed` when this helper returns false, but still
continues. Thus cleanup failure does not turn this selected upgrade outcome
into false.

The finishing path constructs a result object, calls **0x8ada0** and records
timing/status, then sets reason byte **6** and returns true through common
cleanup. Those follow-up bodies and their effect on manager state remain a
separate boundary. The post-success virtual+0xd0 target 0x873ec, retained at
entry here, enumerates files and performs further conditional work; it is not
proof of in-memory installation from its call alone.

Two successful renames establish this selected code path's local result, not a
two-file atomic transaction, synchronized parent directory, crash durability,
complete active-state admission or server acceptance. The retained helpers
contain no explicit fsync call; imported filesystem behavior and other callers
remain outside that bounded statement.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-upgrade-replacement/inspect-replacement.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-replacement.json
cmp /tmp/mc-replacement.json docs/protocol/apk/2.26.43.72-mobileconfig-upgrade-replacement/replacement.json
```

The inspector pins member/ELF identity, strings and selected BL targets. Pair
and retry examples are analyst-derived outcome projections, not executed
filesystem, timing or crash-recovery tests. LLVM retains selected upgrade
branches, the complete pair/retry/rename helpers, cleanup and argument helpers.
Nearest JNI_OnLoad headings are not local identities. No complete exception,
flag-producer or OS error model is claimed.

rea-rename.json retains complete REA Evidence
`ev_660d785be8a9f1b49845bb940986e8904b2db0057307c3c7680fa57ff5125571`,
command `e scr.color=0; pd 42 @ 0x8bce4`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. Its bounded window begins after retry
initialization and extends into the next helper; LLVM retains the initialization.
Provider version/external inputs are not bound; annotations are not runtime reads.

**Remaining:** flag producers, exact paths and generated body formats, candidate
cleanup rules, follow-up and active-state installation, complete restart recovery,
OS/filesystem durability and retry/account ownership. No Rust runtime or saved
account state changed.
