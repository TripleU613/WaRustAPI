# Native regular metrics drain status and outcome ownership — APK 2.26.43.72

**Confirmed static / derived; observed bounded synthetic status execution.**
Pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No real callback, error allocation, network, filesystem or Android process was
used. Rizin/Unicorn outputs are not REA Evidence envelopes. No Rust behavior
changed. This regular manager has its own field layout; do not merge it with
the private HTTP manager's dictionaries/provider fields at the same offsets.

## The nonzero-outcome helper is a drain-status notifier

Previously unresolved helper **0x7273a8** does not implement task backoff or
persistent retry. It acts only when manager byte **+0x50 equals1**; other raw
values return unchanged. It clears that flag **before** notifying the upper
callback table at manager **+0x40**, function slot **+0x30**, context **+0x38**.

| Raw helper argument w1 low bit | Selected status callback arguments |
| --- | --- |
| 0 | context, w1=0, w2=0, error=null |
| 1 | context, w1=0, w2=0, created local MCFError |

The error path calls MCFErrorCreate with domain object **0xc30518**, local code
**3**, null user-info, then releases the error after callback. It still creates
and releases that error when no callback table exists. Neither code3 nor these
raw callback arguments are HTTP status, account restriction code or the separate
regular task outcome enum. Full error-domain identity and callback consumer
semantics are untraced here.

Repeated helper invocation while the flag stays cleared does not notify again.
This is not generation fencing: a later start sets the same flag back to1,
and the selected helper receives no generation ID. Complete obsolete-completion
ownership must still be established before making an exactly-once claim.

No task-set remove, file write/delete, timer or send appears in this helper.
The native error argument alone does not prove the buffer was discarded or
that a retransmission was scheduled.

## Start selects one task and marks the drain active

**0x727294** snapshots regular manager task set+0x48. With at least one entry it
retains the first snapshot value, sets+0x50=1 and, when callback table exists,
invokes its+0x30 status function with context,w1=1,w2=0,error=null. Set snapshot
order is not a proved FIFO/priority guarantee. It then packages three objects
and submits an asynchronous job via **0xbd8bb8 →0x6fedf0** on the selected queue.
The subsequent [regular-IQ trace](../2.26.43.72-metrics-regular-iq/README.md)
now binds this job to Java selector6 and w:stats request/result handling.

If no selected task exists, it calls the same0x7273a8 helper with raw argument0.
An active drain therefore reports its no-error end through this path; merely
calling start on an already inactive empty set does not produce a status event.
The start body itself is static evidence; this fixture executes only the end
helper. No queue submission or actual task is run.

## Task outcome handling remains separate

The [retained full callback0x7275d0](../2.26.43.72-metrics-native-transport-responses/regular-task-callback-assembly.txt)
uses the exact labels success0/retryable1/non-retryable2/failure3. Its selected
branches combine drain notification with independent task ownership:

| Outcome | Selected task action |
| --- | --- |
| 0 success | optional table+0x18; remove from set+0x48; invoke drain-start0x727294 |
| 1 retryable | end drain with error; keep set entry; optional table+0x20 |
| 2 non-retryable | end drain with error; remove from set; optional table+0x28 |
| 3 failure | end drain with error; keep set entry; optional table+0x20 |

Thus the one-shot status callback is not the per-task success/failure callback.
No direct drain-start invocation appears in selected outcomes1/3; higher-layer
callbacks/readiness/timing can matter and remain open. A retained entry is not
authorization for immediate retry. A removed entry is not proof of durable file
deletion or successful delivery. The existing callback's missing-membership
handling is diagnostic rather than a proved stale-generation fence.

## Fourteen bounded helper cases

Fixtures cover active normal/error, even/odd raw arguments, absent callback
table, noncanonical/inactive flag values0/2/3, and sequential repeated helper
calls. Actual native helper/outlined instructions execute; controlled MCFError
and status-function adapters record arguments. Each callback observes flag0,
error allocation/release matches the branch, and repeated completion does not
re-notify while the flag stays cleared. No whole MCF runtime/reference-count or
allocation-failure behavior is exercised.

Run `python inspect-drain.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
drain-boundaries.json. It reuses the pinned ELF resolver and independently
decodes the async thunk and mapped byte-window digests. Rizin windows.json uses
`pD SIZE @ VA`. With Unicorn2.1.4 run
`python check-drain-end.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
drain-end-results.json. Every invocation has100000-instruction/one-second caps;
unexpected imports/interrupts or unsupported loader relocations fail closed.
Checksums cover every other file.

**Inferred:** drain status, per-task outcome, retry scheduling and durable buffer
ownership must remain separate porting concerns. **Observed live:** none.
**Unknown:** upper callback binding, full regular job/send schema, backoff,
generation fencing and persistent deletion/recovery. No server restriction
policy, automatic retry or ban-prevention guarantee is inferred.

## Later recovery entry

The [regular status and timer trace](../2.26.43.72-metrics-regular-recovery/README.md)
binds a separate interval30 readiness/idle recheck and provider-identity gate.
This does not turn the drain-end notifier into backoff or prove durable deletion.
