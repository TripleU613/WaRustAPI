# Transactional stanza and pre-ACK dispatch

**Confirmed static**, WhatsApp APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Five complete method exports produced by `target/debug/xtask apk-smali
../android-lab/apks/WhatsApp.1.apk --class CLASS --method METHOD --signatures
--exceptions`: Flr.invokeSuspend, 1id.A0E, Euj.A00/A01/A02. These are DEX text,
not REA Evidence IDs. Only the selected paths below were audited; unrelated
Flr switch cases and all stanza-family serializer branches are outside scope.

## Flr selector7

[Coroutine](disk-write-coroutine.txt) switch7 enters0578. Positive supplied
A01 delay invokes0nD.A01; after resume it clears1id.A00 under A0D monitor,
then snapshots the pending queue through1iq.A01/FgD(3). A missing backend
1id.A0B or empty snapshot skips persistence. These are not upload timers.

With nonempty work, 0av.A0T selects two paths:

- True at061e falls through to0622: opens0aq and1hk, calls1id.A0E, wraps
  the returned selected stanzas in ElB(null, stanza, null, null), computes
  pre-ACK nodes with Euj.A01 and calls1iV.A0I using the outer0aq. Only then
  does it mark1hk successful, close1hk and close0aq (06c0..06d2).
  Nonempty selected wrappers reach Euj.A02(list,false,true) at0708.
- False branches to0750: calls1id.A0E, wraps its selected result and calls
  Euj.A02(list,false,false) at07ca, without the explicit outer pre-ACK
  insertion from the true branch.

Both paths set selected stanza A0I=true only after the sender call returns.
That is local state, not proof of a completed socket write or server acceptance.
The transactional cleanup handlers0734..0750 rethrow; they do not fall through
to Euj.A02 after a thrown outer close. Crash/ambiguous disk outcomes remain
unknown. No per-count branch/default configuration is inferred from A0T.

## What is selected and dispatched

[1id.A0E](stanza-persistence.txt) filters2Aa.A00 states first, skips rows with
existing A0B IDs, serializes/inserts using its own0aq/1hk, collects positive
insert IDs and closes that transaction before updating the cache. Its final
0846..08a8 result additionally excludes A0I=true and applies2DB.A00/A0J.
Thus its return is not the entire input snapshot. Full serializer correctness,
2Aa/2DB predicates, shared-provider transaction identity and reset on rollback
are not established here; do not equate returned selections with all receives.

[Euj.A00](ack-key-selection.txt) returns null for absent stanza node, otherwise
looks up its FuW generator by stanza class and calls B8b; a missing generator
throws IllegalArgumentException. [A01](preack-conversion.txt) uses metadata
marker -2, invokes the common F3e.A05 builder, and makes1vZ(node,null,stanza.A0C).
It skips null results and catches LIL per entry. [A02](ack-dispatch.txt) uses
stanza.A0C only when its first boolean is true, otherwise -2. Both Flr paths
pass false. It creates a message through1i8.A03 and calls0Sf.A09(message,true,
secondBoolean); that result is not a server confirmation. Per-entry LIL is
logged and processing continues. Other exception types are not claimed caught.

## Consequence and remaining work

This closes the previously untraced Flr insertion caller in the
[transaction bridge](../2.26.43.72-prekey-upload-replies/README.md#ack-transaction-completion-and-rollback-bridge-2026-11-04).
It establishes a conditional persistence-before-ACK-admission path and a
separate non-outer-transaction path. It does not establish count-handler bulk
list membership, effective A0T, registration of every FuW generator, upload
worker scheduling versus socket order, or native crash replay parity.
Rust still has durable inbound captures rather than this full stanza/pre-ACK
queue. No speculative ACK reorder, replay, timer or live account operation was
introduced. A future port must preserve selection, commit failure propagation,
per-entry refusal and post-commit send admission as separate boundaries.
