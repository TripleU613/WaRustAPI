# Client-auth-token failure outcomes — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-callers APK --class LX/0lG; --method BsS`.
Checksums cover every bundle file. FUT exports include both selector families;
only nonzero selector (manager constructs1) is interpreted as token recovery.
Selector0 handles client-config setting and must not be conflated with it.

## Delivery failure conditionally restores the nonce, without sending

FUT's constructor saves manager inA00 and original nonce String inA02.
Its nonzero BsS logs ClientAuthTokenManager/onDeliveryFailure/onError,
then invokes012.A00(null, originalNonce, manager.pendingReference).
The helper attempts AtomicReference.compareAndSet(expected, replacement),
retrying only while get() is still identical to expected. Here expected is
null: it restores the failed nonce if the slot is empty and leaves a newer
nonnull pending input intact. The boolean result is ignored by this callback.
This is a CAS contention loop, not a network retry loop.

No request send, timer installation, rotation invocation, token preference
write or held-request dispatch appears in this selected BsS branch. Restoring
the slot alone therefore does not imply an immediate resend. The previously
traced readiness observer/direct connection drain can later consume a slot;
their exact relative ordering and generation binding remain incomplete.

## IQ error does not perform the same restoration

Nonzero Bvj logs ClientAuthTokenManager/createCatResponseCallback/onError and
returns. It does not inspect the IQ's error code, restore the pending nonce,
rotate a key or dispatch held requests in this body. The selector0 branch's
409/client-config push-registration behavior does not apply to the token
manager. FUT.CmQ returns KNW.A00 for either selector; that constant is not
interpreted here as a retry or scheduling policy.

| Selected outcome | Established effect |
| --- | --- |
| Manager initially not ready | A01 stores/replaces pending input and returns before construction |
| Submit-time readiness fails | A06 calls BsS synchronously; pending restoration is conditional on empty |
| Registered IQ timer expires | Existing message4/A03/8bj21 path invokes BsS; same conditional restoration |
| Correlated IQ error | Bvj logs, with no restoration in the selected body |
| Result header invalid / token value invalid | Previously traced distinct failure paths; neither authorizes held-request replay |
| Valid token result | Previously traced persistence, rotation invocation and held-request selection; not proof completed rotation |

The timer-to-BsS chain is retained in the [auth-key outcome evidence](../../rea/2.26.43.72-authkey-outcomes/README.md),
especially pending/timeout removal and delivery-failure-checked.txt. Exact-class
BsS caller search has five sites, including0Sf.A06 and8bj.run; this is not an
interface/virtual dispatch census or proof of all failure sources.

## Submit-time gate is stronger than the manager's first check

Manager A01 first checks0AI.A0O (state2). Its A0Q wrapper passes false as the
last A06 argument. A06 rechecks0Sf.A0U, which requires both0AI.A06=true and
A0O=true. It does not inspect channel presence in this selected predicate.
If false, it invokes0ln.A0E(id), logs state, and calls callback.BsS(id) directly.
A0E removes that ID fromA0I under lock, with a conditional invariant assertion;
this alone is not a complete pending/timer cleanup specification.

If true, A06 registers0ln.A08 with the callback, ID, request type and supplied
duration, then constructs/dispatches its message throughA01/A05. For this
manager the previously retained duration is32000ms and request type314.
Registration/timer setup precedes dispatch; the two readiness checks do not
provide an atomic connection-generation lock through wire delivery. A0Q is
void and supplies no server acceptance signal to the manager.

## Implementation boundary

Selected failure effects are now distinguished; global backoff, shutdown
flushing, callback races, complete wire promotion and durable recovery remain
open. In particular, timeout does not establish rejection or safe replay.
No Rust sender, automatic retry, route or live account operation is added.
Existing uncertain holds remain unchanged. No ban-prevention claim follows.
