# Fresh-key retry ACK ordering — APK 2.26.43.72

**Confirmed static / bounded local correction.** Build code264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`, rechecked
from the local APK for this chunk. These are complete selected checked-DEX
exports with signatures/exception tables, not REA Evidence envelopes.
**Observed:** no APK runtime, account traffic or server acceptance. **Inferred:**
normal direct retry query ACK handoff is modeled by the selected classic
session-requirement path. **Unknown:** full upstream registration/BX2, every
job lifecycle/provider, request-manager batching/backoff selection, final key-IQ
serializer parameters, full alias/receipt-key/higher-count policy and ACK recovery.

## Session preparation is before ACK; dependent fetch scheduling is later

[160.A04](../2.26.43.72-retry-ack-dispatch/retry-handler.txt) invokesFG2.CbE at0654,
then constructsFe5, stores it in14h.A09 at06a6 and returns via the true path.
FG2.CbE calls163.A05, whose
[full body](../2.26.43.72-retry-receipt-keys/session-policy-entry.txt) processes
carried-key policy and invokes163.A04. This pre-ACK preparation must not be
mistaken for the deferred request callback.

163.A04 checks the held session. On a remote-registration mismatch it obtains
its current base key, invokes the selected0WJ.A0p/A0q cleanup branch and returns
**that old key** at0090, rather than universally returningnull. A missing session
reaches0136 withnull. Higher-count policy can likewise return a held base key;
count2 saves its current key via0WJ.A0s and returnsnull. Feature/session-kind
branches remain explicit; this does not establish universal cleanup semantics.

[14h.A00](../2.26.43.72-retry-ack-dispatch/base-dispatch.txt) submits0Sf.A0O before
removing/running its registered callback. Fe5 stores the returned byte array in
A0D, passes it throughrun's v47 into constructor argument v32 (old base key),
and constructs/enqueuesSendE2EMessageJob at048c..0516. The selected direct-device
constructor path at0268..0292 always attachesAxolotlSessionRequirement; when old
base key is nonnull,0296..02a0 also attachesAxolotlDifferentAliceBaseKeyRequirement.
Both checks occur on a job constructed after this ACK submission.

AxolotlSessionRequirement.BWq tests session presence for its selected scope and
classic/PQ kind. DifferentAliceBaseKeyRequirement.BWq requires an existing
session and compares its key with oldAliceBaseKey, returningnot-equal; it does
not itself fetch a key or declare delivery. Its readObject validates persisted
JID/non-null old key. These are readiness requirements, not wire attributes.

## Actual job-added fetch entry found

SendE2EMessageJob.A0J logs job added, skips duplicate jobs and iterates requirements.
Unmet single-device Session/DifferentAlice requirements save their device at0088
or00a6;02e4..0348 invokes14t.A07 with a one-element device array. Its retryCount
branch selects request-source integer12 (02d2..02e0), but that is **not claimed
as a final server attribute** without the remaining manager/serializer trace.
The method also has multi-device, sender-key and TEE branches not ported here.
A07 delegates toA06 with scopeE78.A02. A0J subsequently calls14t.A03: its retained
body conditionally exits backoff, resets0Vf and callsA04. This does not justify
unconditional Rust backoff resets, automatic retries, or bypassing a server wait.
The exact request-manager A06/A04-to-wire path remains open in this chunk.

The enqueue-to-hook binding is also retained:0fW.A01 constructsQlv selector31
with job/NhR. Qlv's constructor stores the selector/objects;run case31 at048e
invokesNhR.A04(job,owner,false). NhR.A04 conditionally serializes a persistent
job intoSQLite queue, initializes it throughNhf.A00 and invokes virtualJob.A0J
at00de before adding it to the runnable list. This closes the selected enqueue
hook; it does not prove every provider, persistence outcome or recovery route.
The separately inspectedNhR.A00 fallback only releases a held WakeLock, not the
job-added callback. Its full body is retained as a negative result.

This binds the selected missing-session fetch scheduling to the send job reached
through the post-ACK callback. It does not prove socket completion ordering
across threads or parity with every provider/job lifecycle. Rust's single query
and durable pending row are a bounded port, not a full native job queue.

## Bounded negative searches

The full19-method SendE2EMessageJob inventory was used to locateA0J; selected
constructor/A02/A0J bodies are retained, not the bulk class. A02 invoked at the
constructor's end validates arguments; it is not the prekey fetch entry. Fe5's
0wT.A02 candidate buildsDsi and logs MessageLogging/postRetryRejectEvent; it is
diagnostic, not a resend injection or request serializer. Its complete body is
retained to avoid repeating this misclassification. No broad absence claim
about other job methods or native query routes follows from these searches.

## Rust consequence and validation boundary

answer_with_confirmation now attaches the existing handled-retry ACK to a
returned reserved IQ as well as a message. answer() must first commit both the
pending Refetch row and matching fetching/query/count checkpoint. The shared
user-like/context guards and all original-chat/device/age/key/pin/session guards
remain. NodeLink writes that root receipt ACK before the key IQ, and stops on
ACK/query write failure. A callback must still pass the existing refetch
ownership/original/peer guards before any resend. No second receipt ACK is
synthesized from the later directory reply.

The synthetic crypto/store workflow now inspects the durable pending job and
fetching marker before observing the query's ACK. The same injected NodeLink
fault test covers message and IQ destinations: no second stanza before ACK
success and no repeat after either write error. Refusals/duplicates still return
no new pair. These tests do not execute Android/network/account behavior.
No key-query fields, fetch backoff, identity or replay policy changed.

Reproduce selected methods with `target/debug/xtask apk-smali <pinned-apk>
--class 'LX/CLASS;' --method METHOD --signatures --exceptions`, or the complete
named Java class shown in the export signature. Requirement exports use their
exact full class filters without a method filter. Only the method inventory is
extracted from the local full-class discovery export. Checksums cover every
other file in this bundle.
