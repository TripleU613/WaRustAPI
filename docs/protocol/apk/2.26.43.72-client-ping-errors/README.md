# Client keepalive IQ error is a ping response, not a connection refusal

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Existing xtask static DEX exports; no REA Evidence envelope, APK execution,
external socket or live account operation. This extends the
[startup/keepalive audit](../../network-startup-audit-2026-11-04.md).
It is separate from the [server-origin ping response](../../server-pong.md).

## Reader callback to shared manager response

1A7.A0B buildsDzb(String,int), whose selected constructor explicitly sets its
kind16 and creates a w:p IQ request. It binds1xa(manager,builder,Runnable) and
submits through0Sf.A0Q with family22 and32000ms IQ callback timeout. The
manager has a separate configurable timeout; neither is the periodic cadence.

1xa.Bvj logs onError and schedulesFea(manager,24) on1A7.A0H. It does not parse
an error code, wait, text, timestamp or account-restriction state in this method.
1xa.CK4 validates its result schema, then schedules the exact sameFea(manager,24)
before its optional success-only Runnable. The selected Fea(Object,int)
constructor copies its discriminator directly. run's switch maps24 to0ee4.
At0f00..0f10 it ignores a response after the manager's timeout-notified flag;
otherwise it zeros outstanding-ping elapsedA0J and calls1A7.A05.

A05 cancels previous periodic/timeout scheduling then schedules the next
periodic ping. Thus the selected error callback counts as a response for ping
liveness; it is not the manager's timeout/disconnect callback. The latter is
A03, with connected/connection-ready/once-only checks and1A0.CdJ notification.
The separate1xa.BsS delivery-failure method only logs in its selected body.
Full writer-error delivery and success-schema exception lifecycle remain open.

The existing correlated-IQ router's exact result/error admission remains the
basis for calling the selected callback. This trace does not weaken login,
authentication, revocation, notification restrictions or unrelated IQ errors.
It does not authorize any operation that a different request was refused.

## Confirmed Rust mismatch and correction

The normal driver previously returned fixed InvalidInput(connection,
keepalive refused) for an owned, unexpired ping's iq/error. That stopped receive
and classified it as nonrecoverable. A new paused-time injected-link regression
failed against that code because the link had closed before the next ping.

Now either exact result or error consumes only its matching live ping. An error
emits a fixed diagnostic and receive continues. It is not reported as application
success or converted into a retry/reset/account mutation. Its timestamp does
not update confirmed time. No error text or code is copied into that diagnostic.
Existing capture retains the authenticated response according to prior policy.

The regression verifies a successor ping can be written on the same connection,
a duplicate old error cannot consume that successor, a server-origin ping still
receives its native pong, clock provenance is unchanged, and an unanswered
successor still times out as recoverable. The existing non-reply IQ regression
continues to reject missing/get/set/uppercase/unknown types and now follows
native liveness handling for both exact result and error. No real network or
account state was exercised. The new test is additional to existing ownership,
late-reply, slow-write and recoverable-timeout coverage.

## Cadence, timeout and schema remain separate gaps

The retained1A7.A07 clamps integer property15103(0x3aff) to8000..32000ms for
manager timeout. A05 uses property15270(0x3ba6) before the first ping and
15183(0x3b4f) subsequently. Its alarm versus delayed-Runnable path depends on
A06, which reads manager mode plus Boolean property6493(0x195d). Effective
values, defaults and all connected-state producers remain untraced here.
No native cadence is inferred from an attribute name or generic I/O timeout.

Rust's periodic interval and timeout still use its documented local policy
based on transport.timeout_ms. It does not yet port next-ping scheduling relative
to each response or these effective property values. Success-result schema is
also stricter in1xa.CK4 than Rust's current matched result check: selected typed
from/id/t/type validation is retained, not claimed ported. Full8B6 helper/default
semantics, callback exception lifecycle and source authorization remain open.
No successful-result validation, timeout duration or schedule changed here.

## Reproduction and verification

Use existingxtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions for1A7(fullclass),1xa(fullclass),Fea.run/constructors and the exact
Dzb(String,int) constructor. The last export is selected from the full overload
export; no unrelated constructor tree is committed. xrefs retain exact anchors
used to locate the manager and callback. Worker excerpts name full-export hash
and selected byte-PC ranges; omitted code is not inferred absent.

Focused tests: keepalive_error_clears_only_its_live_ping_without_ending_receive,
non_reply_iqs_leave_keepalive_pending_until_real_reply_or_timeout and
unanswered_keepalive_times_out_as_recoverable_and_answered_keepalive_continues.
The first failed before correction (link open assertion) and all three pass
afterward. Full shared-Rust validation is recorded inSTATUS.md. Checksums cover
every other file; no private traffic, keys or bulk decompiled tree is included.
