# Response-relative client ping scheduling and packaged timing entries

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static DEX only, no target execution, external socket or live account operation.
No REA Evidence envelope is claimed. This continues the
[client ping callback/manager trace](../2.26.43.72-client-ping-errors/README.md)
and [success schema/one-shot claim correction](../2.26.43.72-client-pong-schema/README.md).

## A response starts the next interval

The retained1xa.Bvj and validatedCK4 both scheduleFea(manager,24). Its selected
response branch skips if timeout-notified, otherwise zeros outstandingA0J and
calls1A7.A05. A05 cancels old periodic and timeout work, then schedules a new
periodic Runnable with098.A05(runnable,duration) or an alarm for
elapsedRealtime+duration. Thus the next ping period starts at this response
processing; it is not merely the next point on a fixed periodic timer grid.
This conclusion does not require choosing unknown effective duration values.

The normal Rust driver previously shared one interval tick between maintenance
and client pings. A response arriving19seconds into its20-second local response
window could be followed by another ping1second later at the fixed tick. That
is a confirmed scheduling mismatch, independent of whether20seconds is the
correct effective account interval. Changing the shared tick's next deadline
would also postpone unrelated queued receipts, actions and device work.

The driver now tracks separate maintenance and next-ping deadlines. It wakes
at the earliest due work, services due maintenance first, and sends a ping only
when its own deadline is due and none is outstanding. A matched validresult or
error restarts only the next-ping deadline from response processing. Malformed
results consume their claim without clearing/rescheduling the outstanding
ping; non-replies, old duplicates and unowned/late replies do not restart it.
Maintenance keeps its existing locally selected interval. Slow maintenance may
delay ping submission, and its response window still starts at its own send
admission. No automatic retry, durable callback replay or clock adoption added.

## Packaged values: selected assignments, not effective settings

08Y.A0I's integer-builder alias is established at001a..001e from its parameter.
Its direct literal entries at0796..07f2 are:

| AB property | Native use | Selected packaged value |
| --- | --- | --- |
|15270(0x3ba6) |1A7.A05 before first ping |15000ms(15s) |
|15183(0x3b4f) |1A7.A05 after count becomes nonzero |240000ms(240s) |
|15103(0x3aff) |1A7.A07 manager timeout |32000ms(32s) |

A07 clamps its effective timeout to8000..32000ms. The distinctA0B IQ submission
uses32000ms callback timeout. A0I is called with constructor integerbuilderv13
at43b8..43be; buildKeepingLast atcc16..cc28 binds that builder to08Y.A02.
Constructor's selected Boolean entry6493(0x195d) isfalse: v1=0 at015c,
Boolean.valueOf at016c, valuev2 reaches the0252..0258put. The selected branch
1A7.A06 also requires its manager mode before choosing delayed Runnables;
false does not mean pings are disabled, because the manager can schedule alarms.

These are proven selected assignments. A whole-map final-last-writer proof,
complete effective account preferences/cache/readiness and SDK selection are
not established by the excerpts. No packaged value is silently applied as an
effective account setting. Existing00E.A02/A01/A06 and adapter/DI evidence is
retained in the [configuration audit](../2.26.43.72-token-bucket-configuration/README.md).
It separates registered fallback entries, persisted values and guarded SDK
reads; raw AB reply rows are not sufficient to choose final values.

The checked0El lookup-map derivation finds no literal specifier entries for
6493/15103/15183/15270 in its2065 writes to a41080-long array. Do not mistake
a zero lookup/missing specifier for a zero duration, or equate absence from a
raw AB result with this literal-map fact. Full effective resolver handling and
possible shared-array alias mutation remain separate limits. This negative
result contrasts with the four nonzero token specifiers in the prior bundle.

## Regression and remaining port

The paused-time actor regression runs bothresult anderror. It writes a client
ping, waits19seconds, supplies the response and confirms receive with a separate
server ping/pong. An action queued then must drain at the old maintenance tick
1second later, without an early client ping. The next client ping occurs exactly
20seconds after the response, with a new ID. On stop the unrelated sent action
remains held and cannot be reserved again. The same regression failed against
the prior driver because the next write was an early ping instead of the queued
action; it passes with the scheduling split. These are injected channels and
paused time, not live timing or server acceptance observations.

The20-second value is the test profile's existing local interval. Runtime first
and subsequent durations and ping timeout still derive from the existing local
I/O-timeout policy; the15s/240s/32s packaged entries and alarm/Runnable modes are
not yet ported as effective policy. Full initial connected-state eligibility,
configuration hydration, native cadence/timeout and all on-demand paths remain
open. No completeClientPingManager or ban-prevention claim follows.

## Reproduction and verification

Use existingxtask apk-smali APK --class 'LX/08Y;' --method A0I/<init>
--signatures --exceptions, one method per invocation. Excerpts retain exact
byte-PC ranges and full-export hashes. The prior manager bundle retains1A7,
1xa andFea with complete callback binding. Bounded literal-key searches
for client_ping_timeout/ping_interval located1lQ and Ht8/INv references. Their
relationship to cadence was not followed beyond that census here; names alone
do not establish client timing or replace the numeric callers. No consumer was
ported from a shared string name.

`python3 derive-specifiers.py` reads the previous bundle's full0El.A0L export,
checks all2065 literal array writes and reproducesspecifiers.json. It is a pinned
literal derivation, not a runtime interpreter. Checksums cover every other file;
no raw APK, private traffic or bulk decompiled tree is committed.
