# Client ping channel startup, counter reset and scheduler mode

**Confirmed static**, WhatsApp 2.26.43.72 code 264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution or account operation. Artifacts are xtask DEX exports, not REA
Evidence envelopes. Selected worker/dispatcher excerpts retain full-export
hashes and byte-PC intervals. The full manager is already retained in
[client-ping-errors/manager.txt](../2.26.43.72-client-ping-errors/manager.txt);
a fresh full-class export matches it byte for byte.

## Connected dispatch binds the exact worker

0YB.A01 at 0876..08b8 obtains the manager, takes its channel callback A1B,
sets manager.A04 from 0Z8.A03 and submits 2JJ(callback, manager, 2) to manager.A0H.
The retained constructor stores selector 2 in $t, manager in A00 and callback in
A01; run's switch maps 2 to 0036. This establishes the worker binding rather
than inferring it from a nearby string. 098.A02 is a SerialExecutor assertion,
not a timer cancellation or readiness mutation.

| Existing state | Connected-worker action |
| --- | --- |
| A05 true and incoming callback is the identical A01 object | Logs already connected and returns through 03c6..03ce; does not reset the counter, pending timestamp or schedule. |
| A05 true and callback differs | Calls 1A7.A02 to cancel/unregister the old manager's timers/receivers and clear old callback/connected flag, then initializes the new channel. This is manager cleanup, not proof of closing a socket. |
| A05 false | Initializes the accepted channel directly. |

Initialization at 0456..0472 stores the callback, resets counter A00 to 0,
outstanding timestamp A0J to 0 and timeout-notified A06 to false, then sets A05
true. The worker registers periodic/timeout receivers if needed in alarm mode,
and reaches A05 scheduling at 04ae. It schedules the initial interval rather
than directly calling A04 to send a ping. Complete exception/readiness behavior
of receiver registration is not established by this selected branch.

The exact field census finds two writes of counter A00: A04's increment after
A0B request submission, and this connected-worker reset. A05 selects key 15270
while the counter is zero, 15183 otherwise. Thus ordinary response rescheduling
uses the subsequent interval after the first normal submission. A repeated
same-channel connected callback cannot legitimately restart the initial interval.
This counter is per accepted channel; it is distinct from the configuration
owner's [first-access value cache](../2.26.43.72-client-ping-first-access/README.md).
No cache reset follows from resetting this counter.

## What determines A04, "can use short"

0Z8.A03 reads actual stored preferences, not an app-foreground flag:

1. If `c2dm_reg_id` is present (contains, not a nonempty-string test), continue.
2. Otherwise require a nonempty `fbns_token`; absent/empty returns false.
3. With either condition satisfied, return `logins_with_messages < 3`, reading
   that integer with default 0. There is no lower-bound clamp in this getter.

No live preference values or equivalent Rust authority are established here.
Wrong preference types do not acquire a guessed value from this trace.
Physical A04 census finds the single direct write in 0YB.A01 and three reads
(manager A05, A06 and worker logging); this does not exclude aliases/reflection.

Manager A06 selects delayed Runnable only when A04 AND effective Boolean
property 6493 are true. Otherwise A05 uses an alarm: type 3 with A04 true,
type 2 with A04 false, both with elapsedRealtime plus the selected interval.
The timeout alarm in A04 uses type 2. These are literal Android alarm type
arguments, not proof of an effective duration or permission to send fabricated
push state. Android alarm APIs and execution delays are not emulated here.

## Rust boundary and remaining work

Rust currently selects local durations from transport timeout and creates ping
state per drive invocation. This chunk changes no runtime timer, reconnect,
account, request or persistence behavior. Native initial/subsequent timing still
requires actual preference/configuration hydration and SDK selection; native
first-access owner lifetime remains open. The full on-demand producer graph,
callback ownership across asynchronous channel replacement, and receiver failure
paths remain to be audited. The static census is not proof against dynamic
mutation. No live acceptance or restriction-prevention guarantee follows.

## Reproduce

From the repository root use `target/debug/xtask apk-smali APK --class CLASS
--method METHOD --signatures --exceptions` for 2JJ.run/<init>, 0YB.A01, 0Z8.A03
and 098.A02, one method per invocation. Use full class 1A7 without --method and
compare with the prior manager artifact. Excerpt ranges and export hashes are in
the retained files. APK identity must match the hash above.

For each retained census run `target/debug/xtask apk-field-refs APK --class
'LX/1A7;' --field FIELD --type TYPE`: A00/I, A05/Z, A0J/J and A04/Z. Census output
contains every physical DEX reference returned for that exact field, not its
transitive aliases. Checksums cover every other file in this bundle. No raw APK,
bulk tree, private preferences or captured account traffic is retained.
