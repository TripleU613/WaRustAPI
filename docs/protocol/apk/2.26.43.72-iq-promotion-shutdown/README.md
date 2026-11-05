# IQ callback promotion and shutdown — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-callers APK --class CLASS --method METHOD`.
The registry files select complete individual methods from the full0ln export.
Other files retain complete selected methods. All files are checksummed.
Only the IQ portions of the large writer/handler are interpreted here.

## Promotion precedes node writing, not follows it

0ln.A08 previously registered callbacks inA0I with a message4 timer before
writer dispatch. A0C(id), underA0F's monitor, checksA0I, removes its callback,
and inserts1bW(callback, elapsedRealtime) intoA0H. WhenA0I lacks the ID, A0C
returns without inserting a replacement callback. No wire write appears inA0C.
Its sole exact caller is0uw.C0e. The writer invokes0uv.C0e at00b0, before
its selected node-writing branch. 0ZJ.onLooperPrepared explicitly constructs
0uw and passes it into0v0's constructor, which stores it asA02:0uv; this
establishes the concrete callback binding for that initialization path.

For the token request,0Sf.A01 creates a Message with arg1=233, arg2=314,
iqId present, dropIfOffline=true and checkCallback=true (positive32000ms).
1i7.A00 maps envelope233 to arg2, so the writer's initial logical type is314;
the later dispatch switches directly on arg1 and selects233's0344 branch.
That branch calls1rj.A03 at035c; A03 then invokes1r2.Cyu(node,1).
The promotion callback at00b0 therefore occurs before even this node-write
entry point. A0H membership or its timestamp is not proof of completed socket
write, server receipt or server acceptance. Cyu's full byte/socket behavior
is not reconstructed by these selected bodies.

## Presence check and promotion are separate operations

When checkCallback=true,0v0.A00 first calls0uv.BNI(id). 0uw.BNI delegates
to0ln.A0I, which checksA0I.containsKey under that map's monitor. A missing
callback sets the writer's drop predicate; it reachesC6j rather than writing.
The writer also consults itsA01 flag and dropIfOffline; that is a separate
local decision, not a server rejection.

There is no single critical section spanning the BNI check, C0e promotion
and node-writing call in the selected chain. A timeout can remove a queued
callback between presence check and promotion; A0C's missing-ID return does
not return a status to stop this writer path. This is a static possible
interleaving, not an observed race. It prevents inferring definite non-delivery
solely from callback removal or expiration. Once writing begins, partial or
uncertain delivery remains possible regardless of registry bookkeeping.

C6j's concrete implementation calls0ln.A0D(id). A0D removes the tracked
message4 timer when present and posts message5. Handler case5 invokesA03
with false. A03 removes a callback fromA0I first orA0H second and schedules
8bj selector21/BsS, or logs a missing callback. For token BsS the previously
traced effect is conditional pending-nonce restoration, not immediate send.

The writer's IOException handler sets itsA01 flag, puts a Message copy at
the front of its target queue and callsCGH. This is not proof of successful
retransmission: reprocessing still encounters the gates described above.
No general retry allowance or complete reconnect policy follows here.

## Shutdown processing distinguishes the two maps

0ln.A0F(Z,I) checks both maps underA0F's monitor. Only when its boolean
argument is false AND both maps are empty does the selected block resetA00,
remove messages4/8 and clear the timer tracking map. It then posts message3
with the supplied integer asarg2. The boolean's broader policy meaning is
not inferred from its position or the method name.

Handler case3 snapshotsA0H entries and clearsA0H under the same registry
monitor, then releases it before submitting8bj21/BsS for each snapshot entry.
The selected IQ block does not clearA0I. The remainder also handles message
ACK state, which is not equated with these IQ maps. A0F has two exact callers
in0YB.A05 and0YU.handleMessage; their full shutdown eligibility and arguments
remain untraced here. Thus this is not a claim that shutdown clears every
queued callback or cancels every pending timer.

## Implementation boundary

This narrows the earlier wire-promotion/shutdown gap: callback promotion is
pre-write, local drops become delivery-failure callbacks, and the selected
shutdown flush targetsA0H. Native concurrent ordering, late-result dispatch,
complete socket/reconnect behavior, account/generation ownership and durable
recovery remain open. Existing Rust uncertainty/no-replay behavior remains;
no runtime sender, automatic retry, route or live account operation is added.
