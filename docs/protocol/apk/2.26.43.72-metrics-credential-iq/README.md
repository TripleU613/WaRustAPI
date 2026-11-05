# Metrics credential IQ producer and callback — APK 2.26.43.72

**Confirmed static / derived**, pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
**Observed live:** none. No Java/native callback, credential generation or network
operation was executed. CLI smali and local Rizin exports are not REA Evidence
envelopes. [Previous lifecycle trace](../2.26.43.72-metrics-native-credential-lifecycle/README.md).

## Java request producer

Exact method `JniBridge.jnidispatchIO(ILjava/lang/Object;)J`, selector **7** at
byte offsets0014..01ee, treats its object as byte[]. It obtains 08p through
0WS.A0C →36e.A01 →05a.A00. The retained full method includes other selectors;
those must not be merged with selector7. It constructs:

| Node | Attributes / content |
| --- | --- |
| iq | xmlns=privatestats; type=get; to=0mL.A00; generated id conditionally added after 0Tk.A06 |
| sign_credential | version=1; child of iq |
| blinded_credential | supplied binary bytes; child of sign_credential |

At0130 it calls 0Tk.A04(bytes,32,32), declaring a32-byte diagnostic range.
**Correction:**00L.A0C logs AssertionError and returns; it does not throw.
The selected Java writer therefore does not hard-reject a different length
through this helper before storing the bytes. This is binary content, not a
text-encoding rule. Native producer/response constraints remain separate; see
[the diagnostic helper trace](../2.26.43.72-metrics-regular-iq/README.md). 0Sf.A0G supplies the ID. Conditional ID
validation is retained, not replaced with an assumption that every generated
string is accepted. 0mL's server destination was previously traced as
s.whatsapp.net in [confirmation-IQ evidence](../2.26.43.72-prekey-upload-replies/README.md).
Attribute insertion order is xmlns,type,to,then optional id; no wire-byte order
or complete ID algorithm is inferred.

The producer installs Hyg callback with elapsedRealtime start timestamp, then
calls **0Sf.A0W(callback,node,id,0xef,32000)**. A true admission returns long1;
false returns long0. Neither proves the server signed or accepted anything.
The method's outer Exception handler logs through 0Wg and returns0. No automatic
retry or invented blinded input is authorized by this method.

## Callback registration is before dispatch

0Sf.A0W forwards to static A0A with its two trailing Boolean flags false.
A0A requires A07.A06 true and A0J nonnull. On the admitted branch, it calls
0ln.A08(callback,id,logicalType,delay,false), then constructs/dispatches outgoing
message via A01/A05 and returns true. A08 stores the callback under a monitor;
positive delay schedules Handler message what4 keyed by ID with
sendMessageDelayed. Here32000 is a **32-second callback delay**, not a recurring
heartbeat phase or a proved end-to-end latency bound.

On not-ready admission, A0A calls A0E(id), logs and returns false. **This selected
branch does not invoke callback.BsS.** It differs from the separately retained
A06 wrapper. Do not import A06's callback behavior into this producer. Timeout
handler dispatch/removal, cancellation and generation fencing remain open.

## Java success/error callback and native result mapping

Hyg.CK4 requires child sign_credential (0Tp.A0G throws LIL if absent), parses its
required attribute **t** as a signed Java long through A07→A0L→A09, and discards
the result. Missing t or nonintegral t throws; it is not optional merely because
its parsed value is unused here. It requires children signed_credential and
acs_public_key, reads their byte arrays, and optionally reads config_id bytes.
No explicit byte-length check occurs in this Java callback; the native
completion's checks remain separate. It logs diagnostic outcome1, ensures the
library is loaded, gets wajContext, and calls:

`jvidispatchIOOOO(4, context, signed_bytes, public_key_bytes, config_bytes_or_null)`.

Both Hyg.BsS(String) and Bvj(node,String) log diagnostic outcome3 and invoke the
same selector4 with **all three data arguments null**. The method names alone
are not proof of their dispatcher triggers. Their diagnostic outcomes1/3 are
not the native credential result enum.

The previously pinned JNI tuple binds jvidispatchIOOOO to native0x6eee64.
Selector4 is independently resolved to **0x6eef0c**. It unwraps context, converts
three Java arrays to MCFData (null stays null), loads provider+0x38 callback
context, and passes raw status **0** when the first two converted data objects
are nonnull. If either is null, it passes null data and raw status **1** via
0xbdf888. It then releases/zeros that provider callback context. Missing provider
aborts; missing callback context returns without dispatch.

Thus **both selected Java failure methods reach raw native result1**, not raw3
just because the Java diagnostic value is3. Likewise, selector4's native status0
means nonnull inputs reached completion; it does not prove successful signature
validation. The prior completion then checks 32-byte data lengths and performs
its cryptographic path before installing a credential.

Native dispatcher0x6feba4 either packages five params and queues thunk0xbd8ba8
→0x6fec84, or invokes exact callback thunk0xbd8a70 →0x729cd0 with its stored
context. Both branches check the expected callback identity; an unexpected
identity traps. Full execution-context behavior and stale-response protection
are unproven, rather than assumed from pointer checks.

## Native outbound bridge and remaining binding boundary

The previous refresh worker calls **0x6ecd60** with selector7 and MCFData. This
bridge converts data to a Java byte array (0x6e9f1c), calls static-long helper
0x6ea324 with method-ID cache **0xc467f0**, then tests long result !=0.
If false, the refresh worker invokes completion with null data/raw1 and clears
its provider context (retained worker0x729c24..0x729c38). Java's selected
not-ready path therefore has an explicit native-side false-return handling path.

**Confirmed static binding (subsequent trace):** selector7 and byte-array/long
signature identify the Java producer above through its initializer and cache. The static cache initialization is now confirmed by the
[JNI_OnLoad method-cache trace](../2.26.43.72-metrics-jni-method-cache/README.md):
index0 resolves that exact name/signature into0xc467f0. Live initialization
remains unobserved. The bounded ARM64 scan for ADRP page0xc46000 plus
nearby same-register 64-bit load/store offset0x7f0 found only this load; it does
not cover indirect stores, address materialization or the initializer generally.
That limited scan did not cover the indexed store subsequently found in
JNI_OnLoad; the static binding is now closed by that separate evidence.

The APK anchor privatestats also finds separate 1hS.A00 ACSSender and1ro static
initialization. The retained other-acs-request.txt is a different producer;
its request/signature behavior is not substituted for this selector7 flow.
A bounded search of crates/whatsapp-api/src and crates/whatsapp-cli/src for
privatestats/sign_credential/blinded_credential found no implementation. Token
fixtures containing these strings are not an operational Rust credential client.
No production Rust mismatch is claimed or behavior added from this trace.

## Reproduction and next boundaries

Export via `target/debug/xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions`. Each file carries exact class/signature. The request
file selects the exact jnidispatchIO signature from the prefix-matched CLI dump;
Hyg retains all six methods. Native windows are listed in windows.json.
`python inspect-native.py RECONSTRUCTED_LIBWHATSAPP.so` must reproduce
native-boundaries.json (digest-bound mapped windows, selector/thunk destinations).
Checksums cover all other files. These exports establish static source behavior,
not synthetic execution tests or live interoperability.

**Unknown:** live Java method-cache availability, blinded-credential/verification
algorithms, full error/timeout dispatcher, retry waits, callback generation and
durability, and whether this metrics subsystem is required for a deployment.
No mandatory telemetry, server restriction policy or ban guarantee is inferred.
The method-cache static binding and selected timeout/error dispatch are now
retained in subsequent sections/bundles. Next useful work is provider recovery,
followed by evidence-backed Rust behavior only where a real surface exists.

## Credential callback timeout, ownership and schema failure (2026-11-05)

**Confirmed static / derived:** fresh DEX class_def inspection establishes Hyg
implements only0lG, extends Object, and has neither1xZ nor1xa dispatch marker.
Run `python inspect-interfaces.py APK`; stdout must match callback-interfaces.txt.
This closes the callback-specific marker question, using the same pinned digest.
The following links reuse already retained complete registry/handler methods;
fresh exports were compared rather than duplicating their bodies here.

For an incoming correlated result/error, [0ln.A0H](../2.26.43.72-iq-response-ownership/registry-A0H.txt)
removes the tracked timer entry and message4 under A0F's monitor, then GETS the
sent-map A0H record. With Hyg's ordinary marker set it posts handler message2;
[0m2.handleMessage case2](../2.26.43.72-iq-promotion-shutdown/handler.txt)
calls [0ln.A02](../2.26.43.72-iq-response-ownership/registry-A02.txt).
That method REMOVES the sent-map record under the monitor, then queues
[1xb.run](../2.26.43.72-iq-response-ownership/response-runnable.txt).
[0ln.A01](../2.26.43.72-iq-response-ownership/registry-A01.txt) dispatches exact
type=result to Hyg.CK4 and type=error to Hyg.Bvj. Unknown types are diagnostic
failures, not successful credential replies. Upstream root/address/correlation
admission is a separate boundary; this mapping does not establish those checks.

Timeout message4 uses its obj ID and arg1 flag to call
[0ln.A03](../2.26.43.72-iq-promotion-shutdown/registry-A03.txt), then removes the
ID from its timer-tracking map. A03 removes queued A0I first; only when absent
does it remove sent A0H. It releases the monitor before scheduling8bj selector21
on executor098. The retained timeout-failure-runnable excerpt proves selector21
calls callback.BsS(id), so the credential timeout reaches Hyg.BsS and the
previously established null-data/native-raw1 path. This does not translate the
timeout into native raw3 or a server refusal. Message5 invokes A03(false) too;
[local drop A0D](../2.26.43.72-iq-promotion-shutdown/registry-A0D.txt) cancels the
tracked timer and posts that failure message. Exact timeout execution can be
later than its scheduled delay because handler/executor work is asynchronous.

These are competing ownership claims, not proof of completed I/O. If A03
removes first, A02 finds no sent callback and does not queue the response parser.
If A02 removes first, a later A03 has no unchanged map entry to claim. A response
GET in A0H, timer cancellation, and subsequent REMOVE in A02 are separate stages;
ID reuse/replacement and callback generation remain unproven. Selected A0H/A02
do not use their elapsedRealtime value as a strict deadline-admission check.
Do not weaken Rust's explicit monotonic deadlines based on this static trace.

For Hyg's missing required response child/t, or nonintegral t, retained parser
helpers throw LIL(String). [The constructor](../2.26.43.72-iq-response-ownership/schema-error-constructors.txt)
sets isRecoverable=true. Registry A01 catches LV5, reports it through0B0.A0M,
records schema-error diagnostics, and follows recoverability routing. This
selected catch does **not** reinsert a callback, invoke BsS/Bvj, or synthesize a
native completion. Native credential pending byte+0x88 is cleared in the native
completion, not by these Java helpers.

**Inferred risk boundary:** a malformed credential reply can consume registry
ownership and cancel its timer without reaching the native completion that
clears renewal state. No whole-program stuck-state claim follows: other native
lifecycle transitions or the concrete reporting hook may intervene. Those
recovery effects remain unknown. A Rust port should preserve explicit pending
ownership and report parse failure; it must not copy an assumed automatic retry
or treat callback consumption as a valid credential. No automatic repair,
credential replacement or telemetry was added from this observation.
