# Post-availability startup session hooks — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`. Selected small methods are complete. Startup is an explicitly
labelled excerpt of the [full retained method](../../rea/2.26.43.72-authkey-outcomes/startup-trigger-checked.txt).
2JI.run is explicitly excerpted with full-export hash, discriminator dispatch,
selected39 body/continuations and exception table. Other selectors and most
individual metric field meanings are not interpreted here.

## Adjacent hooks are not automatically new protocol requests

After the conditional bare availability block,0ZJ.A0z invokes1s1.A00 with
1jv.A08().A08's integer session ID, then1jG.A05, then1lQ.A02. Selected1s1.A00
logs previous/new values and stores its boxed Integer inA00; it does not construct
a stanza or schedule another method.1lQ.A02 obtains its provider, reads
SystemClock.uptimeMillis and storesA02. Effects of resolving that provider are
not proven globally absent. Neither small body establishes a server request.

1jG.A05 is more involved: under its instance monitor, ifA0A is zero it submits
2JI selector39 through098.execute. It initializesA09 from uptime only if zero,
setsA0A using the previously traced08q.A00 clock, setsA0B from uptime, clears
nullableA0E, resets integerA00 and callsA04(this,false). A0A's check and later
clock assignment are local in-memory state, not a durable once-only work claim.
Executor acceptance is not completion or proof any metric was transmitted.

## Conditional persisted-metrics processing

2JI's constructor stores object/discriminator; run's switch maps39 to01b8.
The selected body first checksA15(0x2280). False returns. True enters its
monitor region and reads attempt_metrics_list_proto bytes. Null bytes exits
without parsing. Nonnull bytes cause the preference key to be removed with
apply BEFORE parseFrom into289. It then loops over attemptMetrics and calls
1jG.A00(metric,null,4). The protobuf's persisted uptime is logged, not used here
to establish a request deadline or replay-authority timestamp.

The selected exception table does not establish transactional restoration of
that preference after parse/handoff failure. Apply calls are not proof of
completed disk durability. This is native metrics recovery handling, not
permission to delete uncertain message/request state in Rust. The metrics
producer, persistence format validation and downstream sink policy remain open.

## Cleanup and metrics sink remain distinct from application state

A04 iteratesA0Q and skips records already marked isOfflineCompleteReceived.
For a remaining entry it callsA01(key,this), removes that iterator entry, and
(with the startup false argument) does not set the optional disconnected flag.
It callsA00(builder,null,2), then logs the metric's session/attempt/time fields.
A01 clearsA0D only if its key matches and conditionally invokes the offline_resume
helper. Complete side effects of that helper remain untraced.

A00 builds28f from runtime/record inputs and hands it to0NX.CSW(0FP), returning
the metric. Its complete body is retained, but individual field semantics and
input producers are not reconstructed here. The sink interface's concrete
sampling, queue, serialization, transport, batching, retry and server-response
behavior remain open. A sink invocation is not proof of a new HTTP/IQ write,
mandatory startup telemetry or successful delivery. It does not establish that
isOfflineCompleteReceived proves the inbox is fully applied or durably stored.

## Implementation boundary

This closes the selected three-hook ordering, two local bookkeeping bodies and
conditional metrics scheduling/handoff. Full startup eligibility, offline-complete
producer/consumer semantics, metrics sink/provider policy, generation handling
and durable recovery remain incomplete. No fabricated telemetry, new request,
Rust behavior, retry, HTTP route or live account/network operation is added.
