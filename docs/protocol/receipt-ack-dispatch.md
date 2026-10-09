# Generic receipt ACK dispatch and callback registration

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Ten-topic recipe](apk/recipes/2.26.37.71-receipt-ack-dispatch.json),
[complete selected methods](apk/2.26.37.71-receipt-ack-dispatch/targets.json),
[physical callback/caller inventory](receipt-ack-dispatch-inventory.txt).
Offsets are method-relative bytes; exception tables use code units.
This follows [nullable ACK correlation](receipt-ack-correlation.md) and reuses
[SendReadReceiptJob](apk/2.26.37.71-read-receipt-job/read-job.txt).
No runtime, private account state or live request changed.

## Full-key callback map, separate ID queues

**Confirmed:** the all-DEX BaE substring scan finds exactly two declarations,
abstract 1Dw and concrete 1Dy. 1Dy.BaE resolves 0y8 and invokes A08 with the
original node/key. A08 constructs Cal(node,Cb1), obtains handler message **6**
and sends it to target. Cal.<init> stores node A00 and key A01 unchanged.
This establishes the immediate Java forwarding path, not all native dispatch or
observer registration closure.

0yG.handleMessage case 6 starts at 0010. It takes the Cal.A01 key and, under
0hn.A04's monitor, removes that exact key at 0050..0056. A nonnull returned
1Yj receives BjN(original node) outside the monitor (0062..006c); absent key
skips this invocation. 0hn.<init> creates A04 as HashMap. Together with retained
Cb1.equals, this is full-key callback correlation, including nullable participant.
Cb1.hashCode includes participant A01 at 0038..0046 as well as other route and
metadata fields. Complete helper hash/JID equality semantics remain unknown.

The handler then separately reads key.A08 ID, removes a ConcurrentMap entry,
cancels handler message 8 when an entry exists, and uses 0ha.A02/other ID maps
(0092..0118). This runs even when the earlier full-key lookup missed. **Unknown:**
full ID-queue ownership, request classes and those downstream helpers. Thus the
full-key map is not proof every Android ACK consumer checks the same route,
and the ID queue is not grounds to weaken Rust's explicit-receipt guard.

## Read-job registration and device-zero key normalization

**Confirmed:** retained SendReadReceiptJob builds CSu with class receipt, type,
original ID, remote chat, participant and recipient, calls A00 and then
0h8.A0B (0138..013e). A0B normalizes the key through A02, creates AzY,
registers it through 0hn.A01 (0024..002e), then calls A05 to dispatch the
Android message. Registration precedes that dispatch call, not necessarily
socket-write completion. Full A05 transport failure behavior remains unknown.

0h8.A02 conditionally converts a device-zero remote DeviceJid to its userJid
and similarly a device-zero convertible participant to its userJid. A nonzero
device is not stripped; an absent participant is not created or removed by
this helper. It clones via Cb1.A00/CSu only when needed, otherwise returns the
original key. Complete clone/helper guarantees and native subtype predicates
remain unknown. This is bounded representation normalization, not PN/LID merging.

The second wrapper 0h8.A0C also normalizes and registers A01 when 09Y.A06 and
A0O admit it. It builds CHK and Android message 486 (0x1e6), with
messageClient:dropIfOffline false. The other branch completes AzY via BjK with
IOException and MessageClient not ready, user not registered likely. That is
an APK diagnostic, not proof of a particular server refusal or account state.

Physical scans find two callers each: A01 from 0h8.A0B/A0C, and A02 from
SendE2EMessageJob.A06/SendE2eStatusJob.A0G. The latter full bodies are not newly
traced here; this is call-site evidence, not complete send-job behavior.

## Duplicate callback replacement and result payloads

**Confirmed:** 0hn.A01/A02 both synchronize on A04, check containsKey, and log
added duplicate ackable stanza plus duplicate_ackable_stanza on a match. They
still put a new D42 under the supplied key at 00a4..00aa. They differ in D42
discriminator 0 versus 1. Replacing a callback is not evidence of a transmission,
identity rotation or permission to replay an uncertain request.

D42.BjN discriminator 0 invokes the held callback with **null** (006e..007e).
Nonzero invokes it with the supplied payload (0008..0016). Both then perform
elapsedRealtime latency/metrics calls and optional 1fQ cleanup. This callback
method does not inspect an error code itself. It must not be read as handset
receipt, decryption success, verified identity or all-error acceptance policy.
Complete AzY completion, upstream ACK special cases, wrapper error callbacks,
timeout selection and receive effects remain unknown/unported.

Rust keeps durable explicit receipt reservation/correlation and refusal handling;
no callback replacement, timeout retry or ID-only settlement was ported. All
prior IDs, wires, keys and waits remain intact. This evidence narrows generic
ACK registration/dispatch, not full receipt_user admission, primary cleanup or
verified identity recovery. Existing offline research commands reproduce it;
no HTTP route, CLI command/flag or live capability was added.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-ack-dispatch.json \
  --output docs/protocol/apk/2.26.37.71-receipt-ack-dispatch --check
```

Validation: ten topics / fourteen selected methods / fifteen files / 86,052
bytes reproduced exactly from the pinned APK. Cached self-diff verified ten
unchanged topics and four indexed artifacts. Regenerated CLI reference, API
library-symbol checks, formatting and diff hygiene passed. Repository hygiene
passed across 1,717 local links and 62 checksum bundles. No new runtime test
or live result is claimed; the preceding ACK fix passed 812 workspace tests.

Follow-up: [specialized ACK producers and dispatch](receipt-ack-special.md)
traces read/played/server-error state messages separately from generic key
completion. Concrete N12 A0d/A0Z and fallback applications remain unknown.
