# Primary metrics readiness latch — APK 2.26.43.72

**Confirmed static**, checked DEX TXT; not runtime observations, native analysis
or REA Evidence envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce methods with `xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions`; field inventory uses `xtask apk-field-refs APK
--class LX/0NY; --field A0I --type Ljava/util/concurrent/CountDownLatch;`.
Selected methods are complete except explicitly labelled initialization-worker
excerpts with complete-export hash. Shared release dispatcher is retained in full;
only selector3 is interpreted. Initialization inputs are scoped below.

## Static latch and local status

0NY.<clinit> creates one static CountDownLatch(1). isInitialized returns exactly
whether its count is zero. The field inventory finds one write in this static
initializer and ten reads across the named methods/workers. This is not an
alias/native/reflection census and does not prove that all indirect latch uses
have been found. Reopening the count is not established by these selected bodies.

The earlier [event worker](../2.26.43.72-startup-metrics-native-handoff/README.md)
waits through0NY.A00. That helper catches interruption, reinterrupts and returns;
its normal return therefore remains distinct from count-zero status.

## Activation submits initialization and a separate release task

0NY.ALK checks instanceA03. If false, it submitsIBm selector47 to0NR.A00,
submitsIC0 selector16 on0NQ's0NR.A00 using its required runnable, then submits
I9Q selector3 to the originalA00 executor. Only after those execute calls does
it setA03=true. Missing required runnable throws; no local catch makes the whole
sequence atomic or restores partially accepted submissions.

I9Q's constructor stores the discriminator; run's switch maps3 to0052, where
it calls staticA0I.countDown and returns. The decrement is a separate scheduled
Java task, not an explicit native initialization-success callback.0NR's
constructor creates three distinct098 executor instances: A00,A01,A02. Primary
event queueing usesA01 while these initialization/release submissions useA00.
Exact executor scheduling, exception recovery and injected0NQ executor identity
remain separate proof obligations; source call order alone is not completion.

A03 is per-instance, while the latch and isInitialized status are static. Thus
one opened static latch does not, from these methods alone, prove every sink
instance completed its own native setup. No current runtime race/failure is
claimed; this is a constraint on the interpretation of the selected status.

## Initialization reaches another native operation

IBm selector47 obtains JniBridge and requires networkGetter and appContext. It
resolves/initializes a native NetworkSession through14D/Gy9, constructs selected
configuration values from08Y, and supplies a files-dir/wam path. The retained
map includes storage_size (configured value multiplied by1024), buffer_timeout,
throttling_percentage, throttling_exempt and conditional enable_ps_upload_v2,
among other values. Their full producers/defaults/units/server policy are not
established merely by their names.

After JsonSerialization.initialize and a library-loader helper, it invokes
JniBridge.jvidispatchIOOOO with selector0, path, NetworkSession, native context
and config map. The returned Long is discarded; no direct countDown appears in
this selected initialization body. Native dispatch meaning, disk initialization
and later response/error handling remain open. The separately queued release
task must not be interpreted as proof of a checked native success result.

CPo(integer) is retained as a dependent-operation example: it checks count and
executorA01 context. Zero count runsI9X selector6 directly; pending count with
the selected executor-context predicate true calls the wait helper first, while
false queues a wrapper. Its integer meaning and native continuation are not
reconstructed here. It is not a per-event server confirmation API.

## FIFO completion advances even after an initialization exception

**Confirmed static,2026-11-05:** the complete [098 executor](../2.26.43.72-prekey-upload-replies/prekey-ack-serial-executor.txt)
and [1VR wrapper](../2.26.43.72-prekey-upload-replies/prekey-ack-serial-worker.txt)
were re-exported and matched those retained sources byte for byte.098's
constructor creates ArrayDeque; execute offers a selector0 wrapper at its tail,
andA00 polls the head, marks it active and submits to08Z.0NR constructs its
three098 objects with the final Boolean false, selectingCbD in that submission.

1VR selector0 runs the task under a separate execution lock. On normal return
it increments the completed-task counter, clears its current-thread marker and
calls098.A00 to dispatch the next queued task. Its catch-all path performs the
same bookkeeping/next dispatch, then rethrows the original failure if that
advance returns. Therefore a task's exception does not by itself permanently
stop this serial queue. Failure in the next-dispatch call can still prevent
progress or replace the thrown outcome; underlying executor behavior is open.

0NY.ALK submits initialization47 before release3 to the same0NR.A00 instance.
Consequently, for accepted tasks whose wrappers execute/advance normally,
release cannot run ahead of that initialization task's completion, but completion
can be exceptional. Native setup may throw and the wrapper can still advance
to the release task. This is a static control-flow possibility, not an observed
native failure or proof the underlying executor always survives exceptions.

The intervening0NQ owner resolves0NR through the same global provider0x51 used
by the factory supplying0NY.A09. Under that shared cache binding, its scheduled
work also joinsA00 between those tasks. Provider replacement/invalidation and
concurrent activation are not tested here.0NQ constructs its own separate
CountDownLatch(1); that instance latch must not be confused with static0NY.A0I.
The middle task's full behavior remains untraced.

098.A03 can clear pending queue work and delayed-work bookkeeping, as shown in
the retained full executor. Its call sites and applicability to this metrics
startup remain open; no claim is made that it actually clears a release here.
A lost/rejected dispatch, queue clearing, thread/process termination or an
interrupted waiting event remains distinct from successful initialization.
No request replay, recovery latch reset or fabricated metrics is added.

## Remaining work

Underlying executor rejection/termination, activation callers,
network/session authority, complete configuration policy, native operations,
buffers, transport, server responses, durability and account-generation fencing
remain incomplete. No latch manipulation, synthetic configuration, telemetry,
Rust sender/retry, HTTP route or live account/network operation is added.
