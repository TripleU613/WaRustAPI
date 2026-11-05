# Channel queued-work predicate and deferred drain — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Command: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`. Complete selected methods are retained and checksummed.

## Two handler envelopes lead to the writer

1A3.CdY sets Message.what=4 and enqueues it on its Handler. The previously
retained [1A3 handler](../2.26.43.72-iq-shutdown-eligibility/channel-handler.txt)
routes4 through0ZJ.A0t toA0r. A0r passes the logical request type to
1iC.A01, copies the Message and submits it toA04:0vH.CdY. For logical type37,
it also invokes0uu.A02; that side effect is not generalized to token IQs.
1A3's constructor binds it to the connection thread's Looper.

The selected0vI implementation of0vH.CdY changes the copied Message.what
to2 and enqueues it on the writer Looper. Its handler case2 invokes0v0.A00,
the previously retained [writer dispatch](../2.26.43.72-iq-promotion-shutdown/writer.txt).
These values4/2 are local handler envelope codes, not IQ request types,
server status codes, timeouts, request counts or replay budgets. The logical
request type is recovered separately by1i7.A00 fromarg1/arg2.

## Concrete delegate binding at writer readiness

0v0.onLooperPrepared constructs0vI(writerLooper, thisWriter); its constructor
stores that writer inA00. It reports the delegate to0uv.CPl. The previously
traced writer initialization binds0uv to0uw. 0uw.CPl posts message0; its
handleMessage case0 invokes0ZJ.A16(delegate, thread), which callsA15.
The previously retainedA15 saves the delegate in0ZJ.A04 and publishes1A3.
0ZJ.A0K returns that sameA04 field for1A3.BNJ. This closes the selected
writer-delegate identity gap, not every initialization, replacement or teardown
path across account/connection generations.

## BNJ describes queued work, not delivery or active-write completion

0vI.BNJ returns true if its Handler has queued message2 OR its owning
writer'sA04 ArrayDeque is nonempty. Combined with the previous1A3.BNJ body,
the selected normal binding is:

`connectionHandler.hasMessages(4) || writerHandler.hasMessages(2) || !deferredDeque.isEmpty()`.

No socket-state, acknowledgement or currently-executing-write field appears
in either selected predicate. Handler.hasMessages tests queued work; a message
already executing need not remain queued, and the deferred item is removed
beforeA00 runs. Consequently a false result is not proof that no write is
in progress or that all requests were delivered. The reads across queues are
also not one atomic snapshot. This is an inference from the selected bodies,
not an observed concurrent interleaving or a complete thread-safety analysis.
The shutdown caller's boolean therefore must not be renamed serverAccepted,
connected, allRequestsSettled or safeToReplay.

## Connection drains the deque through the existing gates

0vI.handleMessage case0 stores the supplied1rj connection, logs the deferred
queue size, clears writerA01 and repeatedly removes the head Message and
calls0v0.A00 whileA01 remains false and the deque is nonempty. Case1 sets
A01=true. This closes the selected queue-drain body gap.

A00 still performs callback-presence and offline/drop checks and can fail,
so dequeue/drain invocation does not prove a second wire send. The token
submission wrapper sets dropIfOffline=true and positive-timeout callback
checking, as traced in the prior bundle. These local queue mechanisms do not
establish durable restart replay, preserved ratchets, fresh account binding,
server acceptance, a safe retry count or a policy applicable to every family.

## Implementation boundary

No Rust sender, retry, route or live account operation is added. The audit now
has the selected concrete queued-work predicate and drain path. Remaining work
includes complete shutdown/reconnect sequencing, callback races, delegate
replacement, trusted push origin, generation binding and durable recovery.
Existing Rust uncertain holds and consumed-connection behavior remain unchanged.
