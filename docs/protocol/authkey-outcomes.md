# Auth-key outcomes and conditional client-auth-token traffic

2026-11-04. **Confirmed static**, Android APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Complete evidence and checked instructions](rea/2.26.43.72-authkey-outcomes/README.md).
This continues the [key lifecycle audit](authkey-lifecycle.md); no live request,
account mutation, token fabrication or key rotation occurred.

## Queue acceptance is not server acceptance

`19i.A00` submits its previously traced key IQ through `0Sf.A0W`, type331,
32,000ms. A0W delegates to `A0A` with two false flags. A0A checks message-client
readiness and channel presence; when unavailable it returns false before adding
the new callback/request. Otherwise `0ln.A08` registers the callback, schedules
Handler message4 when the supplied duration is positive, and queues the request.
The boolean return is local queue acceptance, not an IQ result. The inspected
`19i.A00` does not branch on that return.

The timer starts with pending registration, before a wire response exists.
`0m2.handleMessage` case4 calls `0ln.A03`, which removes the callback from the
queued or sent registry and schedules `8bj` discriminator21. The checked runnable
calls `0lG.BsS`, **not** `Bvj`. For rotation callback `2IS`, BsS only logs.
Thus this timeout path does not itself invoke the recorded failure timestamp /
attempt-decrement helper. Do not turn a 32-second missing result into a definite
server rejection, successful rotation, or a newly authorized key request.
`0ln.A06` schedules message8 for a different acknowledgement path; it is not the
rotation IQ's message4 timer.

## Result/error dispatch

`0ln.A0H` locates the matching sent callback, removes the pending timer and
queues response dispatch (some callback categories use other executor paths).
`0ln.A02` removes the sent callback under lock, then invokes `A01` through its
executor. A01 dispatches exact type=result to CK4 and type=error to Bvj.
An unexpected type does not become success. Complete Java warnings in A0H are
retained; no uninitialized boolean in its rendered fast-path selection is used
as a specification.

The already retained `2IS` bodies then distinguish:

| Outcome | Native callback effect established here |
| --- | --- |
| Local not-ready return | No new callback/request queued by A0A; rotation manager does not consume the return |
| Timer or delivery failure | BsS logs; no success, local key replacement or failure-counter helper in that body |
| Correlated IQ error | Bvj invokes failure bookkeeping; its log mentions500 but the branch is not limited to error500 |
| Correlated IQ result | CK4 attempts local replacement; failure takes failure bookkeeping, success installs key and records success |

The complete socket-write-to-sent-registry promotion and shutdown flushing are
not reconstructed by these selected bodies. No exactly-once wire delivery or
post-restart reconciliation guarantee follows from a callback map. Rust's
existing one-shot action queue preserves uncertain sends; it is not yet a port
of this rotation-specific lifecycle.

## Startup trigger is conditional on a real server-issued token

Current `19i.A00` has three reported static using-nodes: incoming rotation
notification `19h.A09`, `0ZJ.A0z` startup, and `FUT.CK4` token callback. The totals
are static using-nodes, not a claim of every native/dynamic caller.

Checked startup A0z at0b08..0b44 calls `1rj.A05`, saves its boolean, invokes
`1jA.A00`, then calls the rotation manager only when that saved boolean is true.
`1rj.A05` reads `pref_client_auth_token`; when absent/empty it returns false.
Otherwise it decodes the stored token, emits `<ib><cat>{bytes}</cat></ib>` and
returns true. It does not generate a placeholder. The exact native bound on
CAT bytes and all transitive failure conditions are still unclosed here.

`1jA.A00` atomically takes a pending string and calls A01 only when one exists.
A01 defers the string when its connection gate is false; otherwise it constructs
`Dzb(id,string,28)` and submits through A0Q with type314 and32,000ms. Exact
constructor28 builder grammar is now traced in the writer follow-up below;
its conditional input provenance remains a prerequisite.
The successful `FUT.CK4` branch validates/converts the reply, saves a Base64
`pref_client_auth_token`, calls the rotation manager, then invokes0Sf.A08.
The selected reply converters are now traced in the reply-validation follow-up
below. The post-token held-request selection/dispatch
helper is now connected in the challenged-request ownership follow-up below.

**Bounded negative/correction:** the other A01 caller is `F2Y.A02`, identified
by checked diagnostics as **IncomingPushHandler**, not a registration-response
consumer. This refutes a tentative registration-token inference from the class
name/reference alone. Its actual push input authenticity, parameter provenance
and eligibility branch remain unclosed. There is no basis here to add guessed
registration JSON fields, fabricate CAT bytes, or send this traffic on every
new account. A source search found no Rust client-auth-token maintenance flow;
that is a conditional protocol gap, not proof it was required for a particular
past account or caused a restriction.

## Remaining implementation obligations

Constructor28 and the selected reply converters are now traced; close
push/connection gating, native JID parsing, wire-to-sent
promotion, candidate lifetime and lost-result/restart reconciliation. Then
connect durable request/candidate/token state and accepted key installation,
preserving native attempt/cooldown distinctions and the usable old key.
Do not replace the account key on queue acceptance or timeout. Native callback
bookkeeping alone does not establish which old keys the server will accept.

Separately reviewed Rust `queue_read`/automatic `mark_read` paths remain explicit
operator read policy; effective APK privacy/read-self selection is documented as
unported. No read-receipt policy was changed on an assumption about that missing
state. The [privacy continuation](privacy-mex-config.md) owns those prerequisites.

### Challenged-request selection and re-dispatch ownership (2026-11-04)

**Confirmed static, APK43.72:**0TV.A03 selects eligible held requests without
removing those source entries; over-budget entries are removed.0Sf.A08/A0L
re-dispatch through A04 with the original callback retained when absent from
the new invocation. A token-result callback and the selected handleConnected
block reach selection. The token callback calls key rotation before dispatch,
but does not prove rotation completion. Client-side selection counters are not
a server retry allowance or authority to replay Rust reservations.
[Exact methods, map inventories, count conditions and remaining gaps](apk/2.26.43.72-challenged-request-ownership/README.md).
The post-token helper is now connected; token construction/converters, request
family/callback ownership and durable restart recovery remain incomplete.
No production change, new retry authority or live operation.

### Client-auth-token writer grammar and validation (2026-11-04)

**Confirmed static, APK43.72:** exact Dzb(String,String,int) selector28 builds
a server IQ GET in urn:xmpp:whatsapp:push with a pn child containing UTF-8
manager input. That label is not proof of phone-number input; the branch does
not create an auth_token element. The ID helper checks String.length, not a
53-bit numeric value. Nonnull length violations log via00L.A0C and return true
in the selected validator;300 is not a proven server or hard constructor limit.
[Selected overload, helper chain, checksums and unknowns](apk/2.26.43.72-client-auth-token-writer/README.md).
Writer construction is now connected; input eligibility/authenticity, reply
converters and durable challenge recovery remain unported. No production
change, request sender, new retry authority or live traffic.

### Client-auth-token reply binding and byte grammar (2026-11-04)

**Confirmed static, APK43.72:** FUT's selected result pipeline first requires
iq, matching original request ID, parsed from equal to original destination
and type=result. It then reads opaque cat bytes with strict1..1024 bounds.
The value converter alone does not perform request correlation. Native child
lookup picks the first cat, without duplicate counting in this selected path.
Invalid header throws schema error; null token conversion after a valid header
returns without token persistence or held-request dispatch.
[Converters, helper chain and remaining eligibility/recovery limits](apk/2.26.43.72-client-auth-token-reply/README.md).
This closes the selected reply converter gap, not native JID parser, input
authenticity, failure/backoff or durable recovery parity. No production change
or live operation; Rust uncertain holds remain.

### Client-auth-token trigger boundary (2026-11-04)

**Confirmed static, APK43.72:** GCM map key pn maps to the push-consumer
argument labelled pushNonce. The manager call is guarded by its nonnull check;
full v31 control-flow preservation and trusted delivery provenance remain open.
0AI.A0O is exactly state==2; traced transitions write3 (with server-disconnected
diagnostic) and4. The state2 producer and complete eligibility remain unknown.
[Checked exports, caller inventory and limits](apk/2.26.43.72-client-auth-token-trigger/README.md).
No sender, retry authority, production behavior change or live traffic.
