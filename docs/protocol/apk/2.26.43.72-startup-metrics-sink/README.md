# Startup metrics sink selection and admission — APK 2.26.43.72

**Confirmed static**, checked DEX TXT and digest-bound class metadata; not
runtime observations or REA Evidence envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce selected methods with `xtask apk-smali APK --class CLASS --method
METHOD --signatures --exceptions`; run `python3 inspect-direct-implementers.py
APK` for direct0NX declarations. Methods are complete except the explicitly
labelled provider-binding excerpt with complete-export hash. Unrelated routing
cases and individual runtime metric values are not interpreted.

## Provider route and configuration-selected concrete sink

The [earlier startup metrics body](../2.26.43.72-startup-session-hooks/README.md)
hands28f toA0O's0NX.CSW.1jG's constructor initializesA0O with05d.A00(0x3bf).
05d wraps00D.A00's05h handle; its get lazily calls00D.A02 with that ID and caches
the result. The previously [resolved global cache/router](../2.26.43.72-unified-session-eligibility/README.md)
therefore reaches1Uw.A01:959 is greater than841 and at most1122, selecting
1Uw.A04. Its exact959 case calls0D7.A00. Shared cache invalidation and runtime
configuration values remain unobserved.

0D7.A00 reads the integer configuration at0x4f99 and returns:

| Selected integer | Concrete return |
| --- | --- |
|4 or9 | KJv |
|2,5,6,7 or8 | KJu wrapping a primary0NY and a lazy alternate |
|Other values |0NY |

The digest-bound all-DEX inventory finds0NY,KJu,KJv directly implementing0NX;
all three declarations have final flag0x10. This inventory is not a census of
runtime dynamic loading or proof of a particular configuration value. Factory
selection must be established before treating one sink as universal behavior.

## Entry routing and primary sampling

0NY.CSW andKJv.CSW each check the metric nonnull and delegate to their CSJ
with null explicit sampling argument and false final flag. KJu.CSW computes
A01(metric), then separately checksA03(metric,result) for forwarding to primary
0NY andA02(metric,result) for forwarding to lazyKJv. These are independent
conditions in this body: it can attempt neither, either or both branches.
The three private predicates' complete semantics are not reconstructed here;
this is not a claim of uniform/random sampling across the two sinks.

0NY.CSJ obtains a nullable Integer from0FU.A00, passing event code, supplied
sampling descriptor or the event's own samplingRate, and the flag. Null takes
the dropped diagnostic path without invoking primaryA01. Nonnull passes its
integer weight toA01, then emits the sampled diagnostic throughA02. Full0FU
selection, alternateCSJ and diagnosticA02 side effects remain open. A diagnostic
String is not evidence that an event was persisted or delivered.

PrimaryA01 checks bufferChannel and event-code lists. For non3 channels it
constructsKTj selector14 carrying the event and weight and schedules098.execute
through0NR. Channel3 uses twoGtg code lists and can return without scheduling;
those list policies are not interpreted as server requirements. Full worker14
serialization, buffers, batching, transport and response/retry handling remain
untraced. Normal execute return is local scheduling, not delivery proof.

## Port boundary

This resolves the selected factory and closes primary entry sampling/queue
admission, while keeping alternate routing predicates and transport explicit.
The current trace supplies no basis for mandatory made-up startup telemetry,
synthetic metric values, a per-connect network write or a ban-prevention claim.
Full metric-source authority, sink predicates, queue durability, network schema,
flush timing, responses, generation handling and recovery remain incomplete.
No Rust behavior, telemetry event, sender, retry, HTTP route or live account/
network operation is added.
