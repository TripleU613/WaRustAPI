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

### Client-auth-token readiness producer (2026-11-04)

**Confirmed static, APK43.72:** 0AI.A0M writes state2, opens the condition
variable and invokes the selector2 observer notifier. Both exact callers are
in the connected handler, on distinct branches; state2 does not prove every
connected-path side effect completed. The notifier invokes BzL; the token
manager implementation drains one replaceable in-memory pending slot and
rechecks readiness. Registration, scheduling, instance identity and concurrent
lifecycle ordering remain untraced.
[Full selected bodies and limits](apk/2.26.43.72-client-auth-token-readiness/README.md).
No runtime sender, durable recovery claim, new replay authority or live traffic.

### Client-auth-token observer topology (2026-11-04)

**Confirmed static, APK43.72:** readiness dispatcher separates inline ordinary
observers from marker-dependent inline/scheduled paths. The state constructor
uses provider-backed combined observer sets; explicit registration search alone
cannot establish membership. A separate connection-thread pending-slot drain
precedes a conditional rotation invocation in its selected block. Membership,
dispatch category, provider identity and cross-thread ordering remain open.
[Selected bodies, excerpt, searches and limits](apk/2.26.43.72-client-auth-token-observer/README.md).
No Rust behavior change, sender, new retry authority or live traffic.

### Client-auth-token observer membership and delivery (2026-11-04)

**Confirmed static, APK43.72:** readiness set binding0x1d2c includes0x83,
which resolves to the token-manager factory through the global cached resolver.
The combined set preserves per-member resolvers into observer iteration. DEX
interface declarations place1jA in07v, so the null-hint readiness notification
enters scheduled dispatch. The task re-enumerates observers before invoking
the notifier; token-manager delivery still rechecks readiness.
[Bindings, declarations, dispatch and limits](apk/2.26.43.72-client-auth-token-membership/README.md).
Membership/category gaps from the prior trace are closed for this selected
path. Executor completion/order, cache resets, generation binding, trusted
push origin and failure/backoff remain open. No runtime or live change.

### Client-auth-token nonce register preservation (2026-11-04)

**Confirmed static, APK43.72:** a digest-bound conservative CFG checker
finds only the original v69-to-v31 copy reaches the manager argument move.
The later v31 reuses cannot reach it without another definition; the sole
call chain is guarded by the original nonce's nonnull check. The previously
retained method was regenerated from the pinned APK and matched byte-for-byte.
[Reproducible checker, results and limits](apk/2.26.43.72-client-auth-token-nonce-flow/README.md).
This closes the register-preservation gap, not trusted push origin, freshness,
complete eligibility, failure/backoff or durable recovery. No runtime change,
sender, new retry authority or live traffic.

### Client-auth-token delivery failure versus IQ error (2026-11-04)

**Confirmed static, APK43.72:** FUT's token selector restores the original
nonce on BsS only via compareAndSet(null,nonce), preserving newer pending
input. Bvj only logs in that selector; client-config selector0 behavior does
not apply. Neither branch immediately sends or dispatches held requests.
Submit-time A0U requires ready flag A06 plus state2, beyond the manager's first
state2 check; failure calls BsS directly, while success registers before dispatch.
[Callbacks, helper, submit chain and outcome limits](apk/2.26.43.72-client-auth-token-failure/README.md).
Timer expiry shares BsS restoration, not proof of rejection or safe replay.
Global backoff, shutdown/callback races and durable recovery remain open.
No Rust sender, runtime change, new retry authority or live traffic.

### IQ pre-write promotion and shutdown boundaries (2026-11-04)

**Confirmed static, APK43.72:** writer C0e promotes callbacks fromA0I toA0H
before its node-write entry point. The concrete0uw binding is traced through
writer initialization. A0H membership is not proof of wire delivery. Callback
presence check and promotion are separate, leaving a possible timeout
interleaving; removal alone cannot establish definite non-delivery. Local
drops post a failure path; shutdown case3 snapshots/clearsA0H and schedules
BsS, without clearingA0I in its selected IQ block.
[Complete selected bodies, ordering and limits](apk/2.26.43.72-iq-promotion-shutdown/README.md).
Full socket/reconnect behavior, shutdown eligibility, concurrent and durable
ownership remain open. No Rust behavior change, new replay authority or live traffic.

### IQ shutdown eligibility and cancellation-map ownership (2026-11-04)

**Confirmed static, APK43.72:** ordinary disconnect passes the original reason
and channel.BNJ (false when absent) into the registry flush. The selected
login-back failure branch instead gates the call on BNJ and passes(true,-1).
Concrete1A3.BNJ is queued-handler-message4 OR delegate.BNJ, not a connection
state alias. Adjacent0Sf.A0J invokes cancellation callbacks and clears a
separate map; it does not directly clear IQ maps or the pending token slot.
[Complete selected bodies, channel path and limits](apk/2.26.43.72-iq-shutdown-eligibility/README.md).
Delegate semantics, cancellation callback effects, full reconnect ordering and
durable ownership remain open. No Rust behavior change or live traffic.

### Concrete channel queued-work predicate and deferred drain (2026-11-04)

**Confirmed static, APK43.72:** selected writer readiness binds0vI into
0ZJ.A04. The channel predicate combines connection-handler message4, writer
message2 and a nonempty deferred deque. Those are queued-work checks, not
wire-delivery or executing-write completion proofs. Connection case0 clears
the writer flag and drains head Messages throughA00's existing gates; case1
sets the flag. Dequeue/drain does not prove retransmission or safe replay.
[Concrete binding, submit chain, drain and limits](apk/2.26.43.72-channel-queue-predicate/README.md).
Full reconnect ordering, races, replacement/generation ownership and durable
recovery remain open. No Rust behavior change or live account operation.

### IQ response ownership precedes token parsing (2026-11-04)

**Confirmed static, APK43.72:** response admission cancels the tracked timer
and looks upA0H; the ordinary token callback is later claimed by removal in
A02 before executor parsing. Failure/shutdown can win between those stages;
lookup or timer removal is not successful token installation. The selected
LV5 catch handles LIL's recoverable flag without reinsertion, BsS restoration
or held-request dispatch. Recoverable does not authorize network replay.
[Dispatch chain, declarations, exception routing and limits](apk/2.26.43.72-iq-response-ownership/README.md).
Full upstream admission, hooks, executor ordering and durable/generation
ownership remain open. No Rust behavior change or live account operation.

### IQ reader admission and unmatched fallback (2026-11-04)

**Confirmed static, APK43.72:** the selected1rD IQ processor reaches the
ordinary response bridge only for result/error with nonnull ID. get/set have
separate branches. A lower-level1ru map can process a result first; an
ordinary IQ-registry miss posts a fallback which can remove a separate
message callback and invokeBsT(null). A registry miss is not proof of global
nonprocessing. Declared iq tag mapping and processor-map registration are
traced; full injected/alternate bindings and callback effects remain open.
[Selected bodies, caller inventories and limits](apk/2.26.43.72-iq-reader-admission/README.md).
No Rust behavior change, new replay authority or live account operation.

### Shared lower-level IQ ownership and push-config producer (2026-11-04)

**Confirmed static, APK43.72:** the selected setup passes one new map to
reader1rD.A00 and writer1rj.A03. Push-config GET registers1rv before writing;
its result builds a configuration object and forwards it separately from the
ordinary IQ registry. The generic token node path has no explicit low-level
record insertion in the selected writer bodies. Alias escapes, ID collisions
and full Cyu behavior remain open; this is not a global absence claim.
[Shared-map setup, producer, result bridge and limits](apk/2.26.43.72-lower-level-iq-ownership/README.md).
Full configuration grammar/application, other families and durable/generation
ownership remain untraced. No Rust behavior change or live account operation.
