# Native metrics pending worker and persistence boundary — APK 2.26.43.72

**Confirmed static / derived control flow**, pinned reconstructed ARM64
libwhatsapp.so. No metrics/JNI execution, device, account or network operation.
Local Rizin exports are not REA Evidence envelopes. Library SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`,
12783016 bytes, is bound to APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
by the [complete native reconstruction checks](../2.26.43.72-superpack-reconstructed-members/README.md).

## Pending work is asynchronous and removes before processing

The [event manager handoff](../2.26.43.72-metrics-native-event-routing/README.md)
reaches 0x7239ec. It packages the manager into MCIParams, obtains the manager's
global execution queue and calls MCIExecutionQueueExecuteAsyncHelper with zero
delay and callback VA **0xbd8ed8**. This address is **code**, not a relocated
data-table slot: its BTI/B thunk branches to **0x723a40**. The retained inspector
decodes that branch instruction independently. The call carries exact diagnostic
name `_WAMRuntimeProcessPendingEventsAsync`.

Worker 0x723a40 unpacks parameters and requires the manager execution context;
a mismatch reaches abort. State zero logs `Cannot log before WAM runtime is
initialized.` and state five logs `Cannot log after WAM runtime is terminated.`
and exits this selected processing path. Other selected states proceed.
It checks array +0x50 count; under mutex +0xfc it obtains and retains index zero,
**removes index zero, then unlocks before processing the event**. Removal is a
local queue action, not a durable commit, network send or server acknowledgement.

The initial work counter is 10. A selected state-four/flag branch sets it to 100
and logs pending terminating-queue size. The body decrements it at the event
cleanup and may schedule another worker if work remains. Other internal paths
also update the counter, so these constants are not asserted to be a universal
batch-size or server rate limit. Full branch/sampling, buffer rotation and drop
policy in the retained worker are not yet semantically reconstructed.

## Channel-dependent time enrichment occurs after Java/JNI

At 0x723e7c the worker obtains a clock value through **0x72c4fc**. For channels
2/3 it computes signed integer division by 3600 and multiplication by 3600;
other selected channels retain the clock result. It supplies this value with
field identifier **0x2f (47)** to typed field helper 0x721a94. Thus this native
path adds time enrichment after the [earlier Java/JNI boundary](../2.26.43.72-startup-metrics-native-handoff/README.md).
It is not evidence to fabricate unrelated timestamps or telemetry in Rust.

The clock calls MCFAbsoluteTimeGetCurrent, loads an imported double named
**kMCFAbsoluteTimeIntervalSince1970**, adds a sign-dependent half and truncates
to a signed integer after range checks. The selected import relocation is
retained (AArch64 GLOB_DAT 1025). The external constant's numerical value is not
read from a live library here; epoch interpretation derives from its symbol.
Positive values in channels 2/3 are bucketed down to a 3600-unit boundary; negative
values use signed truncation, so this is not generalized as mathematical floor.
The exact serializer representation and source authority remain untraced.

## Selected snapshot write and flush boundaries

Near worker exit, a selected flag path logs `Persisting all buffers and metadata`
and calls wrapper **0x724fc8**. The wrapper creates a snapshot via 0x71fdf4 and
calls **0x6fdca0**. The latter has execution-context and raw-argument guards;
its selected write path requires the third raw argument zero. The wrapper also
has nonzero-argument branches whose complete successful reachability is not
resolved. These call edges must not be flattened into a proven successful write
for every worker state or shutdown. The assembly retains the guards explicitly.

The selected 0x6fdca0 body assembles a **0x69c-byte snapshot** from local counters,
flags and up to 100 array entries, wraps those bytes as MCFData and calls
**0x734210**. A nonzero low return word skips the following flush; zero calls
**0x734334**, whose return is not inspected in this selected caller. Snapshot
field semantics, file identity/open mode, buffer payload files and restart
reconstruction remain open. This is not a complete persisted-event format.

File helper 0x734210 checks bounds/state, obtains MCFData bytes and calls
fwrite with byte element size. It converts returned count, queries ferror/errno,
and returns a packed error value in x0 plus written count in w1. Its selected
caller checks the low error word, not w1. The complete chosen helper body is
retained; no successful I/O is observed. Flush helper 0x734334 calls **fflush**,
handles errno/ferror/clearerr and returns error information. It contains no
fsync/fdatasync or atomic rename. These selected helpers do not establish
crash durability, and the rest of the storage call graph is not declared free
of stronger persistence operations. No Rust durability invariant is weakened.

## Reproduction and next work

Run `python inspect-boundaries.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
boundaries.json. It reuses the adjacent digest-checked tuple resolver, maps every
VA through ELF sections, hashes windows, decodes the callback branch and resolves
the named clock import. Rizin reproduction uses `pD SIZE @ VA` from boundaries
and outlined-ranges.json; selected .text file offsets differ from VA by 0x4000.
The scheduler, worker, snapshot wrapper/body, file write, stdio flush and clock
are retained as complete selected bodies, alongside outlined register helpers.
Retention of the whole worker is not full semantic coverage of every branch.
[Section map](../2.26.43.72-metrics-native-event-routing/elf-sections.txt) and
[tool version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) apply.

Next resolve snapshot guard/caller behavior and file-open semantics, then trace
buffer serialization, ownership, flush triggers, actual transport and replies.
No Rust behavior, HTTP route, telemetry, sender, retry or live traffic changed.
