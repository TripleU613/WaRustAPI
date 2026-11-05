# Unified-session generic writer path — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The complete0v0.A00 export was re-created using `xtask apk-smali APK --class
LX/0v0; --method A00 --signatures --exceptions` and matched byte for byte the
[already retained writer](../2.26.43.72-iq-promotion-shutdown/writer.txt).
The request-kind helper is complete. Normalization/wrapper-detection entry
excerpts are explicitly labelled with complete-export hashes; unrelated paths
in those methods are not interpreted. Printed instruction offsets are bytes;
branch deltas and exception table offsets are code units.

## Envelope dispatch and normalization

The [unified-session refresh sender](../2.26.43.72-unified-session-startup/README.md)
uses0Sf.A0X(node,447): Message.arg1=6, arg2=447, obj=ib node, and only the
selected dropIfOffline=false Bundle flag.1i7.A00 returnsarg2 for envelope6
(and several other wrapper kinds); otherwise it returnsarg1.447 is the logical
request kind used for policy/logging, not a required447 case in the final
arg1 writer switch. Envelope6 and233 both target0344 in0v0.A00.

0Sf.A05 with both flags false calls its privateA00 normalization helper.
That helper first invokes1i8.A07. For arg1=6, none ofA07's explicit branch
values or71..73 switch cases matches, so its entry returns null. PrivateA00
then returns null immediately. A05 consequently copies the original Message
and dispatches it throughCdY, rather than replacing this node with a loggable
pre-ACK wrapper. It does not enter the normalization helper's later ACK/receipt
stages for this selected envelope. This is a branch-specific result, not proof
that all generic nodes bypass every logging or validation layer.

## Writer gate, defer queue and node call

For the selected unchanged Message, no iqId or checkCallback key was supplied
byA0X. The writer reads null iqId and false checkCallback, taking its local
callback-missing flag=false branch. The type helper returns447, which differs
from special envelope43 and is absent from the72-kind offline-drop allowlist.
When the writer'sA01 flag is true, dropIfOffline=false and the callback flag
allow it to copy this Message into its ArrayDeque and return. WhenA01 is false
it continues. The separately [traced writer lifecycle](../../authkey-outcomes.md)
connects this flag/deque to disconnection and later drain; no durable queue is
established here.

The null iqId skips0uv.C0e promotion. At00f0 the switch uses raw Message.arg1,
so6 reaches0344. That branch obtains the logical kind only for logging, casts
obj to0Tp and invokes1rj.A03(node). The [completeA03](../2.26.43.72-iq-promotion-shutdown/node-write.txt)
only delegates to1r2.Cyu(node,1). There is no explicit lower-map insertion in
that method/selected branch. Alias/interceptor effects and fullCyu behavior
remain separate unknowns, not globally disproven by this local absence.

Thus selected refresh submission is not an IQ registry callback/timer request,
and the node's nested unified_session/id is not promoted as a callback ID.
The earlier startup caller can invoke1rj.A03 directly, separately from this
refresh envelope/queue path. Their eligibility/order must not be collapsed.

## Failure does not prove absence of wire effects

The node call at035c lies within the writer exception region covering printed
bytes00be..03bc (code units005f..01de). IOException routes to2578, which logs,
sets writerA01=true, copies the Message to its target Handler's front queue,
and invokes0uv.CGH. Subsequent writer admission/defer gates still apply; this
is not proof another write completed or a safe durable replay protocol.
The selectedE5W/E5X catches log oversized-buffer/stanza failures and return.
Their types are not generalized to every serialization or runtime failure.

No callback ID, per-node result waiter, successful server response or durable
transmission reservation is established for this ib node in these methods.
That is not authority to clear uncertainty or automatically resend a future
Rust operation after an ambiguous write. Complete native reconnect ownership,
server ACK policy, stale cached-ID handling and generation fencing remain open.
No Rust behavior, retry policy, HTTP route or live account/network operation changed.
