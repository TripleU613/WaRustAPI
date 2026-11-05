# Native metrics connection-status scheduler timer — APK 2.26.43.72

**Confirmed static / derived; observed bounded synthetic timer execution.**
Pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Recovery provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No real queue, timer delay, mutex concurrency, provider, account or network
operation was executed. Rizin/Unicorn outputs are not REA Evidence envelopes.
No production Rust behavior or retry authority was added.

## Correct the provisional completion label

Body **0x72ab10..0x72acd8**, previously saved with provisional filename
load-completion-assembly.txt, is **connection-status handling**. Exact logs say
`statuschangedcallback`, `connection_status_changed`, `connection_ready`, and
`connection_unavailable`; helper0xbe4674 preserves incoming w1 as the raw status.
The [paired credential-file bundle](../2.26.43.72-metrics-native-credential-file/README.md)
now names that retained file connection-status-assembly.txt. Earlier provisional
label was not proof of a file-load completion contract.

After unwrapping a live weak manager, the body invalidates an existing manager
+0xa8 timer through0x73629c, then branches on the raw status:

| Status | Selected behavior |
| --- | --- |
| 0 | log ready; call task scheduler0x727d40; create/retain/store/start a timer |
| 1 | log unavailable; call0x728050 with raw error argument1 |
| other | abort |

The status0 timer uses callback thunk **0xbd8fc8 →0x72acd8**, manager weak
context+0xa0, its execution queue and interval **30.0**. Its diagnostic explicitly
calls this interval seconds. Old timer is released when replaced. Invalidating
it is not itself clearing manager+0xa8 or proving no queued callback can run.
The external registration/producers of this status callback and complete
disconnect cleanup are not closed by these selected bodies.

## Shared timer arms once and repeats until canceled

Constructor **0x735fdc** requires a positive interval, stores it at+0x70,
callback/context at+0x58/+0x60, execution queue+0x68 and a weak timer reference
+0x50. It sets repeat byte **+0x80=1**. Initialization/allocation policy is static
only; fixtures below supply synthetic timer objects directly.

Arm **0x7360fc** uses the timer mutex. If canceled bit0 at+0x20 OR already-armed
bit0 at+0x21 is set, it does not enqueue. Otherwise it sets+0x21=1, unlocks and
queues **0xbd9168 →0x7361cc** after the stored interval with its weak timer
reference. Duplicate arm calls are not independent new repeating timers.

Event wrapper **0x7361cc** resolves the weak timer. If no live timer exists,
it returns. Otherwise it reads canceled bit0 under the mutex and, when clear,
invokes the stored callback/context. It then checks repeat byte equals1, reads
cancellation again under the mutex, and requeues the same event after the
stored interval only when still enabled. Values0/2 in the repeat byte do not
repeat on this selected branch. No real elapsed-time behavior is measured.

Cancel **0x73629c** locks, sets byte+0x20=1 and unlocks. This selected body does
not call an API that removes an already queued event. The wrapper's cancel
check suppresses later callback/requeue when it sees the flag. Mutex release
precedes callback invocation and scheduling: cancellation can race after a read,
so these static checks are not a linearizable guarantee that no callback starts
after cancellation returns. Such concurrency is not executed in these fixtures.

## Credential task callback gates work, not repetition

Callback **0x72acd8** separately resolves its manager weak reference. For a live
manager it requires nonnull provider+0x48 and readiness predicate0x6fef18 low
bit1 before calling task scheduler0x727d40. Missing provider/not-ready logs and
returns without scheduling task work. It does not directly create credentials
or unconditionally send an HTTP request.

The generic wrapper still repeats after a no-work callback when its own timer
is live, uncanceled and repeat=1. Thus **provider not ready does not stop this
repeating timer by itself**. Manager and timer weak lifetimes are distinct;
no whole-program stale-generation or manager-retention guarantee follows.
The task scheduler's busy flag and endpoint rebuilding remain separate gates;
see [endpoint ownership](../2.26.43.72-metrics-native-endpoints/README.md).

## Fifteen bounded cases and reproduction

Eight event cases run the shared native wrapper, actual thunk and actual
credential callback: ready/not-ready/no-provider, vanished manager/timer,
canceled, and repeat0/2. Five arm cases check canceled/armed bits, including a
noncanonical raw canceled value2 (bit0 clear). Two cancel cases verify setting
cancellation without queue submission. Noncanonical fixtures describe bit tests,
not a recommendation to persist those values.

Type/weak references, mutex calls, diagnostics, readiness and task scheduler
are explicit controlled adapters. Queue submission is recorded and never run;
interval30 is asserted at that boundary. No real task or network side effect
is provided. The hypothetical live-timer/dead-manager fixture is a controlled
lifetime case, not evidence it occurs in the actual manager's ownership graph.

Run `python inspect-timer.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
timer-boundaries.json. It reuses the pinned ELF resolver, mapped hashes,
labels and independent branch decoding. Rizin windows.json uses `pD SIZE @ VA`.
With Unicorn2.1.4, `python check-timer.py RECONSTRUCTED_LIBWHATSAPP.so` must
reproduce timer-results.json. Each case has100000-instruction/one-second caps;
unexpected imports/interrupts and unsupported loader relocations fail closed.
Checksums cover every other file.

## Rust audit boundary

A bounded review of current activation/prekey/device/outgoing IQ matchers and
retained [IQ reader admission](../2.26.43.72-iq-reader-admission/README.md) found
no evidence supporting a new **universal** reply-from rule. Family-specific
[from/destination binding](../2.26.43.72-client-auth-token-reply/README.md) must
not be generalized to every IQ. Existing type/deadline guards were preserved;
this review does not prove all Rust reply-origin handling correct.

**Inferred:** task retry opportunity, provider readiness, timer cancellation and
credential-file recovery are different state machines. **Observed live:** none.
**Unknown:** complete callback registration/status producers, generation fencing,
concurrent cancellation and queue delay guarantees, account policy, cryptography
and mandatory deployment scope. No Rust retry, identity reset or restriction
avoidance guarantee is established.
