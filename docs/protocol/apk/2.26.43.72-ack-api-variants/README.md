# ACK API variants and live-location distribution boundary — APK2.26.43.72

**Confirmed static**, checked DEX text exports, not observed runtime or REA
Evidence envelopes. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
the pinned artifact from the preceding ACK traces. Commands: `xtask apk-smali
APK --class CLASS --method METHOD --signatures --exceptions` and
`xtask apk-callers APK --class CLASS --method METHOD`. Offsets are method-relative bytes.

## Registration variant controls callback result shape

All selected0Sf.A0B/A0C/A0D normalize Eoh through A02 before registration.
A0B accepts an Android Message and uses0Th.A01, then dispatches through A05.
A0C accepts a node/key/int, checks0AI.A06 and A0O(), registers through0Th.A01,
wraps node/key in ERp and dispatches Message.what=486. A0D has the analogous
readiness branch and dispatch but uses0Th.A02. In the failed-readiness branch,
C/D return an FfK completed with IOException; they do not register or dispatch
in that selected branch. The full readiness predicates remain untraced.

0Th.A01 snapshots the same pre-ACK marker and elapsed time as A02 but constructs
2Gc with selector0 (0096..00a4), while A02 uses selector1. The previously
retained [2Gc.BsT](../2.26.43.72-sender-key-ack-confirmation/wrapper.txt) shows
selector0 calls its wrapped callback with null; selector1 passes through the
received object. Both then record timing and retire a held marker. BsR only
forwards the exception: this selected failure method performs no marker retirement.

**Consequence:** FfK.get returning null in an A01-backed request is not proof
that no ACK arrived, nor proof of a server error. The API intentionally discards
the reply object on that success path. Conversely,0Sf.A0D and the retained
SendE2EMessageJob inline A02 registration preserve the received node for their
callers' separate error inspection. Do not share one reply-content expectation
across all ACK-backed request families. A matched success callback is not itself
a universal proof of an error-free wire node.

## Live-location key job is a notification distribution

SendLiveLocationKeyJob.A0L builds an Eoh key with class=notification,
type=location, ID from0Sf.A0H and from=Dv3.A00 (02c0..02e4). The destination
singleton's constructor calls Jid.<init> with literal location; no complete JID
serialization claim follows from that literal alone. The same destination
object is used for the outbound to attribute. The outer node is notification
with id/to/type attributes, and its participants child contains to/jid entries
with F3f.A03-produced encrypted children (02e6..040c).

It sends through0Sf.A0C with discriminator123, waits on FfK.get and discards
the return value (0414..0426), then marks location-specific recipient state
(0440 onward). This is a selected matched-callback/control-flow observation,
not evidence all error outcomes are successful or all alternative entry paths
have the same handling. Native encryption task bodies and full retry policy
remain untraced.

**Refutation / qualification:** the existence of this out-of-band location job
does not establish that ordinary group/reaction SKDMs should be sent as plain
one-to-one message stanzas, or that their ACK keys use the destination device
as from. This job uses a notification family and location-specific state. The
includeSenderKeysInMessage flag likewise does not, by itself, identify the
producer used when the flag is false. Historical local direct-distribution
experiments and their handset uncertainty remain separate evidence.

## Bounded caller census and remaining work

Exact physical A0B callers:7 (receipt, peer-message and status jobs); A0C:9
call sites (including2 in SendNewsletterMessageJob); A0D:5. The A06 sender-job
inventory has2 methods, FhB.invoke and SendE2EMessageJob.A0L. These inventories
are retained for follow-up, not a runtime/interface/reflective census.

**Unknown:** normal reaction's distribution producer and callback address tuple,
full primary/companion/PN/LID parser mapping, error459 routing, durable individual
distribution confirmation and native recovery/reset policy. Rust's class/ID
guard is unchanged. No guessed sender guard or notification replacement is
authorized by this trace. No production behavior or live network/account operation.

## Checksums

- [api-b.txt](api-b.txt): `a30c11a369d8e45a318fcc5015d3ad4a01888d8342b223d07c9537da2e996cc9`
- [api-c-callers.txt](api-c-callers.txt): `be4de3f66e4fc1103f920905d58b16a2361b2020daac07870e829fdee765d5f5`
- [api-c.txt](api-c.txt): `41e166d8ab1baa284b1ed720c31a5dbee27050d817124908c7492f4094b89baa`
- [api-callers.txt](api-callers.txt): `23959fc17c605e5956026d26782c9a7f27fe9e73615e4e3b445f02b7466b509f`
- [api-d-callers.txt](api-d-callers.txt): `cb98c6bfca7956adc644134301ef0af5df6b7ac425d0918d940ac3e8fe9c1225`
- [api-d.txt](api-d.txt): `9e678768e2b4a7ec819cdfddb3a7620c7732a15329bbc5bf4a8a5c7c62622aa8`
- [location-destination-init.txt](location-destination-init.txt): `506b8f139d021f7da80c31127a87743c5967033a423f3a0c37b50b37dddac411`
- [location-destination.txt](location-destination.txt): `0b74ef0ea7e9adf9754084a34f4218e6b4baa3c271770da169fba6a9220d6ce2`
- [location-job.txt](location-job.txt): `3977d47d53e9d080276e4bdaed34433312a9206efe7eebdc7989b9c3e792f3f1`
- [registration-a01.txt](registration-a01.txt): `77e00a8bdd26d93c4d6f8c29561149da59f6c962becb07dc8f74e3fb3c60ed1d`
- [send-job-callers.txt](send-job-callers.txt): `5618b25630de881e00d1016a052f7ebb5c1d40967716261ad488a69b6b210c6c`
- [wrapper-failure.txt](wrapper-failure.txt): `e399bf9184ae6c9788ffb5405bd578bb747806a8244858cc955595391c8c2c1b`
