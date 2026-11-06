# Ping producers have different pending and completion authority

**Confirmed static**, WhatsApp APK 2.26.43.72 code 264307238, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, live account operation or claimed REA Evidence envelope.
Static exports use xtask; selected worker excerpts retain source hashes and exact
byte-PC intervals, not unrelated bulk switch branches.

## Three producer families

| Entry | Confirmed selected behavior | Authority boundary |
| --- | --- | --- |
| 1A7.A08 | Posts Fea(manager,26); run 26 at 0ec4 invokes A04. | Same connected/callback and outstanding checks as normal periodic ping; no evidence for parallel normal managed pings. |
| 1A7.A0B(Runnable) | Builds ping and installs 1xa(manager,request,Runnable), callback timeout 32000, family 22. | Direct callers can submit without A04's manager outstanding guard/timestamp/counter increment. Optional Runnable is success completion, not error completion. |
| 1A7.A0A(callback,timeout) | Builds ping with caller-provided callback and timeout, family 22. | No assumption of 1xa schema, manager liveness handling or completion policy for arbitrary callbacks. |

The complete manager and callback are already retained in
[client-ping-errors](../2.26.43.72-client-ping-errors/README.md).
The A08 census returns three physical callers: 0YB.A00, 1A5.A03 and FeK.run.
The selected 0YB branch logs reconnectIfNecessary/not_disconnected/check_connectivity
and is gated by its incoming Boolean at 011c. FeK's selected availability branch
calls channel.Ccu, updates ancillary state and requests A08 at 0148. 1A5.A03's
full export retains its short-mode, call/background state, configuration, delayed
logout and final A04/A08 branch; these are conditional producers, not a new
unconditional ping-on-every-message policy. Transitive producer origins and all
ancillary methods are not established here.

A0B census has exactly three physical callers: normal manager A04, 2JI.run and
FmU.invokeSuspend. This is a static direct-invoke census, not proof about reflection,
dynamic dispatch aliases or concurrent requests. The following traces bind the two
non-normal callers to distinct durable-ACK confirmation work.

## Offline-resume confirmation

2JI selector 44 -> 0164 reads 1iV.A0C's marker. Null returns; nonnull constructs
Fef(marker,1us,14) and directly calls A0B at 01b0. Its constructor stores 1us in
A00, marker in A01 and selector 14. Fef.run 14 -> 1584 reads exactly those fields
and calls 1iV.A0J(marker). This closes the selected Fef retirement entry point
left untraced by the previous
[pre-ACK watermark audit](../2.26.43.72-prekey-upload-replies/README.md#confirmation-and-cumulative-retirement).
The queue's existing hold/epoch/watermark/deletion rules still apply downstream;
this is a retirement request, not proof that every call immediately deletes rows.

## Coroutine confirmation

FmU selector 12 -> 08d4 enters a suspension branch; at 08fa it takes 1t2 and its
captured marker-like object, creates continuation KmF, then Feq(marker,1t2,KmF,9)
and directly submits A0B. The exact generic constructor puts manager in A02,
marker in A01 and continuation in A00. Feq.run 9 -> 1d9a calls A0J on the captured
1vZ, then resumes the continuation with Unit. Thus normal return of this selected
completion follows the retirement request. It does not prove durable deletion,
exception recovery, cancellation handling or how every coroutine was constructed.
There is no 1t2.A02 method body in this APK; no producer is invented from that name.

## Error versus success, and asynchronous ordering

1xa.Bvj only enqueues Fea(manager,24) for manager liveness. BsS only logs delivery
failure. CK4 validates the pong schema before enqueueing the same manager worker
and then synchronously invokes its optional Runnable at 019c..01aa. Neither IQ
error nor delivery failure executes that optional Runnable. Consequently a
confirmation IQ error must not retire a queued marker merely because it clears
normal ping liveness. Malformed success cannot execute it either.

Enqueueing the manager worker is not completion of that worker. The optional
retirement callback runs in CK4's current execution context after execute returns;
there is no join proving manager scheduling has completed first. Conversely, the
worker could already have run by then. No fixed ordering between the two is
established by this source. The Fea24 branch holds only manager, tests A06,
clears A0J and reschedules; it has no request ID, channel identity or epoch check
in that selected branch. This is not evidence that a stale reply is accepted:
callback registry cleanup/ownership across channel replacement remains unresolved.
Rust's link-local owned callback claims remain intact; no speculative weakening.

1xa.BVv returns true only when its optional Runnable is null. Exact caller searches
against declared 0lG.BVv and concrete 1xa.BVv return zero. The former interface's
export has no BVv declaration. These bounded negatives do not establish that BVv
is unused: another interface owner/dispatch/reflection remains untraced. No retry
or disposal policy is inferred from that Boolean.

A0A's direct callers are 1t2.A00, OutgoingSignalingHandler.sendOutgoingCallPing
and FV9.BLH. Prior retained 1t2/8RG evidence covers the first; the other callback
families remain open and must not inherit 1xa behavior simply because the wire
request builder is shared.

## Port and investigation boundary

No Rust behavior changes here. Rust does not yet implement native durable pre-ACK
queue replay/retirement; its raw inbound capture and ACK write do not establish
confirmation-watermark parity. A future port must keep normal outstanding state,
direct success continuations, arbitrary callback families, channel/epoch ownership,
write uncertainty and cumulative durable retirement separate. Do not add automatic
replay/deletion, treat IQ error as confirmation, or copy manager-only stale-response
behavior without closing registry lifecycle. Remaining work includes coroutine
entry/cancellation, callback cleanup across replacement, arbitrary A0A callbacks,
complete replay admission/load and ambiguous-commit recovery. No server acceptance
or ban-prevention guarantee follows.

## Reproduce

Use `target/debug/xtask apk-callers APK --class 'LX/1A7;' --method METHOD`
for A08/A0B/A0A. Use apk-smali with --class CLASS --method METHOD --signatures
--exceptions for 2JI.run, FmU.invokeSuspend, Fef.run/<init>, Feq.run/<init>,
Fea.run, FeK.run, 0YB.A00 and 1A5.A03. Excerpts retain byte-PC ranges and complete
source export SHA256. Exact negative searches: apk-callers with class 0lG and
1xa, method BVv. Prior full callback/manager exports provide the branch contracts.
Checksums cover every other bundle file; APK and bulk exports stay outside Git.
