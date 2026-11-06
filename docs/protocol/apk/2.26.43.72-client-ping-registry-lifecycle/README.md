# Pending ping registry reset and dispatch ownership

**Confirmed static**, WhatsApp 2.26.43.72 code 264307238, pinned APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, live account operation or REA Evidence envelope claim.
This follows [producer-family boundaries](../2.26.43.72-client-ping-producer-families/README.md)
and closes the previously unknown declared owner/use of 1xa.BVv.

## Dispatch policy is 1xZ, not 0lG

0ln.A0H looks up a callback in map A0H after cancelling its timeout-message
identity. It tests interface 1xZ and invokes BVv. 1xa.BVv returns true exactly
when its optional success Runnable is null. With BVv true AND effective
1hy.A01 enabled, A02 is called with true: it removes the callback under lock
and dispatches FcV selector 5 through 08Z.CbN. Otherwise concrete 1xa is handled
by A02(false), which removes the same callback and submits 1xb to serial A08.
Other callbacks can instead be posted to handler message 2 before A02 claims them.
Both selected workers invoke the same 0ln.A01 dispatcher.

Thus normal managed pings can use the gated alternate dispatch route; direct
A0B confirmation pings with nonnull Runnable return false and take 1xa's serial
route. BVv is not a retry, disposal or replay flag. Its earlier exact-negative
searches against 0lG/1xa did not establish non-use: 1xZ is the declared interface
and has one physical caller, 0ln.A0H. Effective 1hy.A01 selection and executor
thread/ordering implementation remain unported. Static enqueue is not completion.

## Registered, awaiting response and claimed are distinct states

Prior retained A08 registers queued callback ownership in map A0I. The complete
A0C method moves it under the shared lock to A0H as 1bW(callback,elapsedRealtime).
A02 atomically removes the A0H entry before scheduling the callback worker.
A successful claim consequently exists outside the registry map while waiting
for worker execution. The prior
[one-shot trace](../2.26.43.72-client-pong-schema/README.md) retains A02/A01/1xb.

0YB.A05 at 01fa..0268 calls router A0F(BNJ-derived Boolean,reason), then 0Sf.A0J,
then submits Fea(manager,27) for manager disconnect cleanup. Exact A0F census
also finds 0YU.handleMessage, whose selected branch passes reason -1. Full
connection-state prerequisites and all queue cleanup in 0Sf.A0J are not closed
by these excerpts. BNJ's meaning is not inferred solely from its name.

Router A0F posts handler message 3 with arg2=reason. Only when its Boolean is
false AND both callback maps are empty does it also reset A00 and remove timeout
messages 4/8 and clear their identity map before posting. This condition matters:
A0F's call is not itself a synchronous unconditional registry clear.

Handler case 3 -> 055a snapshots A0H entries and clears A0H under the shared
lock, then schedules 8bj selector 21 for every removed callback. That worker
invokes callback.BsS(id). For 1xa that is delivery-failure logging, not success
completion, queue retirement or manager-liveness success. The reason arg2 is
loaded for subsequent handling; no reason-dependent success is inferred here.
The queued A0I map is not cleared by this selected reset block. Complete queued
request disposal and write uncertainty still need the outbound-queue trace.

**Derived static limit:** clearing A0H can prevent later registry lookup/claim
of entries still in that map, but cannot by itself cancel a callback already
removed by A02 and captured by 1xb/FcV. Neither selected worker rechecks registry
membership before A01. A manager response worker likewise holds manager rather
than a channel epoch. This does not prove an exploitable or observed stale reply:
all producers, executor ordering, global teardown and replacement authority must
be traced before claiming such a native race or copying that lifecycle to Rust.

## Rust regression

`previous_link_ping_replies_cannot_settle_or_extend_restarted_ping` exercises
both stale successful result and stale IQ error. It writes a ping, cuts that
link, verifies recoverable termination, closes/reopens the store, and starts a
fresh drive. A new ping has a distinct ID. At 19 seconds into its 20-second local
window, the old link's reply is injected. An unrelated server ping/pong proves
receive processed it. The new ping still times out exactly 20 seconds after its
own admission. No stale reply clears or extends it. This uses paused time and
injected channels, not network observation or native timing validation.

A deliberate mutation removing the owned-ID match makes the regression fail;
the guard is restored and the normal test passes. This tests the existing
ownership guarantee without changing production behavior, native effective
durations, retry policy or durable reservations. The 20-second interval belongs
to the existing local test profile, not a newly inferred server requirement.

## Reproduce and remaining work

Use xtask apk-smali APK --class CLASS --method METHOD --signatures --exceptions
for 0ln.A0C/A0F/A0H, 0m2.handleMessage, 0YB.A05, 0YU.handleMessage, FcV.run,
8bj.run and full interface 1xZ. Extracted methods name the full-class export hash;
selected worker/dispatcher excerpts name full-method hashes and exact byte-PC
ranges. Run apk-callers with class 1xZ/method BVv and class 0ln/method A0F for
retained censuses. The prior full 1xa/manager exports establish callback behavior.
Checksums cover every other file; raw APK and bulk source stay outside Git.

Open: pre-send queued A0I disposal, executor/handler ordering across new channel
registration, already-claimed completion cancellation and effective dispatch flag.
Native durable pre-ACK replay/retirement remains unported. Existing Rust claims
stay local to drive; no live-account operation, automatic replay or durable-state
reset is introduced. This is not complete lifecycle parity or a ban guarantee.
