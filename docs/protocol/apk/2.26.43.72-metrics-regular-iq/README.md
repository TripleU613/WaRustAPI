# Regular metrics IQ producer and response bridge — APK 2.26.43.72

**Confirmed static / derived**, pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No Java/native job, actual queue, telemetry payload, Android or network operation
was executed. CLI/Rizin exports are not REA Evidence envelopes. No Rust behavior
or mandatory telemetry requirement was added.

## Native regular job reaches a different Java selector

[Regular drain start](../2.26.43.72-metrics-native-drain-outcomes/README.md)
submits thunk0xbd8bb8 → **0x6fedf0**. The job unpacks its typed objects,
gets task payload through0x71e250/object+0x18, then calls byte-array bridge
**0x6ecd60** with selector **6**. That bridge uses the
[confirmed JNI method cache](../2.26.43.72-metrics-jni-method-cache/README.md)
for `JniBridge.jnidispatchIO(ILjava/lang/Object;)J` at0xc467f0.
Selector7 in the same Java method is credential signing, not this request.
The [complete exact method](../2.26.43.72-metrics-credential-iq/java-request-dispatch.txt)
was already retained; java-selector6-excerpt.txt selects this body/return/catch
context without copying every unrelated selector.

## Eligibility and IQ construction

Selector6 treats input as byte[], obtains36e through0WS.A0C, and iterates
36e.A02's35J records. Each reads a call-state predicate through FvZ/1Cp.A01.
False predicate or config3c93=true proceeds to the next record. Otherwise it
requires nonnull CallInfo, isGroupCall=true and config1d4=true to continue;
failing those branches returns long0. These are raw selected predicates, not
proof of a universal ban on metrics during every call or a fully reconstructed
call-state policy.

After that loop,0AI.A0N=true together with config25b6=true also returns0.
Config/record producers and effective runtime values remain unknown. On the
selected sending branch it generates an ID through0Sf.A0G and constructs:

| Node | Attributes/content |
| --- | --- |
| iq | xmlns=w:stats; to=0mL.A00; type=set; conditional generated id |
| add | t from currentTimeMillis()/1000, subject to diagnostic helper result; supplied binary buffer |

0mL's server destination was traced as s.whatsapp.net in
[prior IQ evidence](../2.26.43.72-prekey-upload-replies/README.md).
Attribute insertion order is xmlns,to,type,optional id. No actual timestamp,
ID or telemetry bytes were invented or sent.

## Important correction: schema diagnostics do not necessarily reject

Binary helper0Tk.A04 is called with bounds **4..126976**. The caller stores the
same byte[] afterward without testing a Boolean return. On an out-of-range
length, A04 calls **00L.A0C(false,message)**. The exact retained A0C allocates
AssertionError, logs it through Log.e(Throwable), and **returns normally**;
there is no throw instruction. These are diagnostic bounds at this writer
stage, not proof it prevents dispatching an out-of-range payload.

This also corrects the earlier credential-selector7 claim: its A04(bytes,32,32)
declares diagnostic32-byte bounds, not a hard Java rejection guard. The
[credential IQ notes](../2.26.43.72-metrics-credential-iq/README.md) and summary
claims have been corrected. Native producer size/response validation can impose
separate constraints; no global claim that arbitrary credentials are valid
follows. The retained length helper file is now binary-length-diagnostic.txt.

The add timestamp's0Tk.A05 receives nonnull Long and bounds **1577865600..
4102473600**, optional=false. It returns true for nonnull values even after
logging an out-of-range diagnostic through A0C. Thus on this selected normal
return path the nonnull computed timestamp still reaches the t attribute; do
not invent omission or refusal outside the stated range. Null-value handling
and logging failures are separate branches. ID0Tk.A06 is also a diagnostic
schema helper; its prior [writer analysis](../2.26.43.72-client-auth-token-writer/README.md)
explains why nonnull out-of-range strings are not a hard rejection proof.
A future Rust policy should distinguish declared schema from native enforcement.

## Callback-backed wait and its32-second timer

The producer creates **FfK**, installs **T54 selector0**, and calls
**0Sf.A06(callback,node,id,logicalType58,delay32000,trailingFlag=true)**.
The retained A06 checks A0U. On ready it registers callback0ln.A08 **before**
constructing/dispatching the outgoing message. A08's Handler message4 timer
makes32000 a32-second callback delay, not a recurring heartbeat phase.
On not-ready it removes the queued callback via A0E and invokes **BsS(id)**
synchronously. This differs from credential selector7's A0W/A0A false-return path.

FfK contains a CountDownLatch(1). Its no-argument **get()** calls await() without
its own timeout; then returns stored data if A02=true, otherwise throws
ExecutionException. The producer uses this no-argument get, not the timed
get overload also present in the class. Handler/executor delivery can delay
completion beyond the scheduled callback delay. No hard end-to-end32-second
latency bound or interruption recovery guarantee is established.

T54's exact class_def implements only0lG, with no1xZ/1xa marker, so it uses the
ordinary [IQ response/timeout ownership chain](../2.26.43.72-iq-response-ownership/README.md):

- CK4(result) and Bvj(error), selector0, both call FfK.BsT(node), marking data
  available and counting down. No response-child schema is parsed in those
  selector0 branches; the class's other selector handles unrelated UI data.
- BsS(id), selector0, creates IOException with disconnected-wait text and calls
  FfK.BsR(exception), setting failure state and counting down. This is reached
  by selected timeout/local failure routing, not an HTTP status classification.

After get returns, producer reads the node's type and returns long1 only for
exact **result**, else long0. ExecutionException/InterruptedException are logged
and return0 through the shared failure exit; other caught Exception goes through
0Wg and returns0. This Java result aggregates local inhibition, error replies
and wait failures. A zero return is not necessarily a definite server refusal
or proof no bytes were sent.

## Native outcome mapping and task retention

Bridge0x6ecd60 normalizes Java long !=0 to a Boolean. The native job maps true
into raw regular outcome **0**, false into **3**, then dispatches the task and
outcome through expected callback **0xbd8a68 →0x7275d0** directly or via queued
**0xbd8b98 →0x6fea64**. The queued body unpacks the stored numeric result and
checks the same exact callback identity; unexpected identity traps.

The earlier [regular outcome trace](../2.26.43.72-metrics-native-drain-outcomes/README.md)
then removes success0 tasks and advances the drain; failure3 ends the drain with
error and retains the task entry. This selected producer does not emit regular
outcomes1/2 through this Boolean mapping. No complete retry timing, callback
consumer or durable file-deletion policy is established. Callback identity checks
are not full stale-generation fencing.

A bounded source search of crates/whatsapp-api/src and crates/whatsapp-cli/src
for w:stats/FieldStatsXmpp/wamsys found no dedicated operational sender. Token
fixtures are not an implementation. No speculative telemetry sender or blanket
retry policy was added from this evidence.

## Reproduction and limits

Run `python inspect-regular.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
regular-boundaries.json (pinned mapped windows, JNI table and callback branch
resolution). Rizin windows.json uses `pD SIZE @ VA`. Java exports use xtask
apk-smali with exact class/method and --signatures --exceptions; files carry
exact method prototypes. The selector excerpt refers back to the complete
method and is explicitly partial. `python inspect-interfaces.py APK` reproduces
callback-interfaces.txt with the pinned APK digest. Checksums cover all other
files. These are static contracts, not execution tests or server observations.

**Inferred:** ready/admitted, callback response, drain outcome and durable buffer
ownership are separate milestones. **Observed live:** none. **Unknown:** effective
eligibility configs, full buffer framing, upper callback/durable ownership,
regular retry scheduling and generation fencing. No full transport parity,
server restriction rule or ban-prevention guarantee follows.
