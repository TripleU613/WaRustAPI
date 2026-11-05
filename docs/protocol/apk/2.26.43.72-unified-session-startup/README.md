# Unified-session startup node and ID lifecycle — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce: `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`, `xtask apk-callers APK --class LX/0Sb; --method A04`, and
`xtask apk-field-refs APK --class LX/0Sb; --field A06 --type Ljava/lang/String;`.
Complete selected methods/inventories are retained and checksummed. Only the
0Sb/nonzero-discriminator branch of shared1WM.Bnp is interpreted here.

## The node after config GET is ib/unified_session

The earlier [startup excerpt](../2.26.43.72-push-config-trigger-error/startup-order-excerpt.txt)
calls0Sb.A02 after1rj.A02 (push-config GET), then sends a nonnull node through
1rj.A03.0Sb.A02 obtainsA03's cached/generated String. Null returns no node;
nonnull constructs this shape using the previously traced append-child helper:

```text
ib
  unified_session [id=generated String]
```

There is no IQ wrapper, request ID, namespace, type or destination attribute in
this selected construction. The id belongs to unified_session, not an IQ callback.
Its String validator is called with0..64 and optional=false; the earlier
[0Tk.A06 body](../2.26.43.72-client-auth-token-writer/README.md) has diagnostic-only
nonnull length assertions.64 is therefore not a demonstrated hard server limit.
Full startup eligibility and the server meaning/obligation remain unproven.

## Cached identity and generation

0Sb.A03 returnsA06 immediately when nonnull. When null, already-initializedA07
plus false09F.BUx returns null. Otherwise it enters theA05 lock, rechecksA06,
and callsA00 if still null. A00 registers1WM selector1 once through09F.CWa,
setsA07, and stores09F.Auo's current PhoneUserJid inA00. FalseBUx clearsA06;
trueBUx generates a String and increments the longA01 counter. The precise
meaning/source ofBUx remains open; it is not labelled generic online readiness.

The generator is String.valueOf((08q.A00(clock)+259200000)%604800000).
Constants are milliseconds (three and seven days), but this is not a traced
seven-day rotation timer: explicit refresh can recompute at other times. Java
signed Long arithmetic/remainder and the supplied clock remain the actual
operations; no random UUID, phone-derived token or stable account identifier
is inferred.08q.A00 uses elapsedRealtime plus nonzeroA03 orA04 offset, otherwise
currentTimeMillis minusA02. Offset producers/server synchronization remain open.

A03 captures the generated value/counter for a follow-up send only whenA07 was
already true before this initialization. It later compares that captured counter
under the lock before callingA01 with the candidate or null. First initialization
does not take that follow-up-send candidate. It returns the currentA06 for the
caller's node construction. This is a scoped race check, not an atomic fence
covering every later dispatch or account switch. If called byA02, an initialized
cache-miss path can invokeA01 and still return a node for startup to submit;
do not deduplicate those calls by assuming a single global producer.

A04 explicitly regenerates under the lock and submits the resulting String after
unlocking. Its direct caller inventory includes onActivityStarted,0Mz.A01,
DirectReplyService.onHandleIntent andF4k.handleMessage. Caller-specific gates
and timing remain untraced.1WM selector1 observes09F.Auo, compares against the
stored JID under the same lock, regenerates on difference, and callsA01 after
unlocking; unchanged identity passes null (A01 returns without submission).
The eight A06 field references show writes only inA00 and reads in these selected
methods/observer. This is a field inventory, not full alias/runtime coverage.
No durable restore of this String is established by the selected cache methods.

## Refresh submission is separate from startup's direct writer call

0Sb.A01 returns for null; otherwise builds the same ib/unified_session shape
and invokes0Sf.A0X(node,447), discarding its boolean. A0X checks0AI.A06, creates
Message.what=0,arg1=6,arg2=447,obj=node, sets dropIfOffline=false and callsA05
with both flags false. A normal helper return yields true; initial falseA06
yields false. It does not establish server acceptance or eventual wire delivery.

A05 handles channel presence/readiness separately: a missing channel can throw,
or a selected state12/not-ready race path can log/drop. With a channel it runs
its selected normalization/dispatch path and invokesCdY. FullA00 normalization,
writer family447 handling, queue ownership and cancellation remain open. No
per-request IQ result/timer registration appears inA01/A0X themselves; effects
of deeper helpers must not be inferred from that bounded absence.

## Port boundary

This identifies another actual startup stanza and its cached/refresh producers.
Full upstream gates, clock-offset authority, refresh cadence, family447 writer,
server responses, acknowledgement requirements, generation fencing and durable
recovery remain incomplete. It is not proof this stanza must always be sent or
that omitting it causes restrictions. No Rust sender, retry, route or live
account/network operation is added.
