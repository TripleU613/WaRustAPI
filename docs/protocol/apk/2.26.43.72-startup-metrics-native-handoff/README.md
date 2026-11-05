# Primary metrics worker and native handoff — APK 2.26.43.72

**Confirmed static**, checked DEX TXT; not runtime observations, native-function
analysis or REA Evidence envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce with `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`. Constructor/readiness helper are complete. KTj.run is explicitly
excerpted with full-export hash, selected14 body/monitor handler, discriminator
dispatch and exception table. Unrelated selectors are omitted.

## Queue object and validation do not establish transmission

The [primary sink queue](../2.26.43.72-startup-metrics-sink/README.md) buildsKTj
with event, integer weight, discriminator14 and0NY. Its constructor stores those
inA02,A00,$t,A01, respectively. The run switch maps14 to0596.

The selected worker skips validation for1xN instances. Otherwise it validates
bufferChannel0/1 events with0FP.validate(). Nonempty validation errors log and
produce one1xN diagnostic event per error, sharing a generated random grouping
value. It feeds each back through0NY.CSW, so the earlier sampling/queue gates
still apply. It then continues with the original event: this is not a
blanket validate-failed/drop branch. Other channels skip that validation region.
Full diagnostic-event schema, validation rules and source data remain open;
no diagnostic or random field is manufactured in Rust.

0NY.A00 next awaits static CountDownLatchA0I without a timeout in this helper.
InterruptedException is logged, the thread interrupt flag is restored, and the
helper returns normally. The worker then continues; an interrupted await return
is not proof that the latch reached zero. Latch producers, startup ordering and
worker cancellation remain untraced. Waiting here is local synchronization, not
a per-event server response timer or successful session-readiness proof.

## Conditional enrichment and JNI boundary

Mode3 may add traceIdInt for allowlisted event codes. Modes8/10 can attempt
migration/dedup metadata on another branch. A selected1Ia subtype receives an
additional field. These branches and their exact helpers are retained but are
not a complete policy for every event or factory configuration.

The primary continuation obtains JniBridge, event code, psId, bufferChannel and
the worker's weight. Channels0..3 map to their selected integer channel values;
other values throw IllegalArgumentException before the JNI handoff. It calls
getFieldsMap, invokes the library loader, obtains getWajContext and calls
JniBridge.jvidispatchIIIIIOO with four Long arguments and two objects: event code,
weight, psId, channel, native context and field map. The returned Long is
discarded in this body. This traces the Java-to-native boundary only: neither
that opaque return type nor normal return proves serialization, enqueue,
network write or server acceptance.

After the native call, modes1/3 skip a later Java diagnostic-upload region.
Other selected paths consultJga's cached event-code configuration and random
selection before calling0B0.A0Z with a wam-event/name label; additional Falco
shadow/log paths are also present. Their full source authority and eventual
network effects remain outside this interpretation. No universal per-event
HTTP/IQ request or ordering requirement is inferred from a Java logging call.

## Remaining work

Native dispatch operation/implementation, library binding, serialization,
buffers, disk durability, batching/flush, endpoint selection, server response,
replay rules, source authority and generation ownership remain unresolved.
Full alternate sink and routing policy also remain open. The selected Java
exception table does not supply a general native/network retry or durable
reservation for this event. This evidence adds no Rust telemetry, sender,
retry, HTTP route or live account/network operation.
