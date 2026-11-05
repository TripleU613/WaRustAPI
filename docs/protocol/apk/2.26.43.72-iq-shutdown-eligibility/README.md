# IQ shutdown eligibility and separate cancellation map — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-field-refs APK --class LX/0YB; --field A1B --type
LX/1A0; --writes-only`. Complete selected methods are retained and checksummed.
Only the identified shutdown/channel blocks are interpreted in large handlers.

## Ordinary disconnect passes a channel predicate and the original reason

0YB.A05 receives(handler, reason:int, error:boolean). At0022 it copies the
reason into v1; no intervening v1 write occurs before0228's0ln.A0F call.
The boolean passed toA0F is false ifA1B is null, otherwise the result of
A1B.BNJ. The third input, error, is not directly used as that flush boolean.
If the call returns normally, it next invokes0Sf.A0J. Thus neither boolean
argument nor the method name alone specifies a force-clear-all operation.
The [selected registry implementation](../2.26.43.72-iq-promotion-shutdown/README.md)
posts its sent-map flush and only resets all tracked timers in the already
empty/false-argument case; it does not clear the queued IQ map in that block.

## Selected login-back failure path is conditional

0YU.handleMessage's login-failed case receives2AF. Its selected login-back
branch checks1sJ.A00.get at05e6; when true it reports failure via1sJ.A03's optional
callback, transitions handler state and invokes0Th.A03(exception). It calls
0ln.A0F only whenA1B is nonnull andBNJ returns true. The actual arguments
at068c are booleantrue (v3 assigned1 at054e, not overwritten on this selected
path) and integer-1. The subsequent0Sf.A0J call is in the same gated block.
This is not a rule that every login error or every disconnect flushes exactly
the same state. Other branches of the complete handler remain uninterpreted.

## Concrete channel predicate is not a connection-state alias

0ZJ.A15 saves its0vH delegate, constructs1A3 and passes it to0YS.BqT.
The retained0YU.BqT implementation posts message0 carrying the channel;
0YU's message0 block stores it in0YB.A1B only whenA1H is true (otherwise
it logs not started and returns). The field-write inventory also finds the
0YV shutdown handler clearingA1B, along with message-client channel fields.
These bodies identify a producer and matching consumer implementation. Binding
the0YS instance across initialization paths, an exhaustive dynamic implementation
census and complete lifecycle ordering remain open.

1A3.BNJ first checks hasMessages(4). If true it returns true without reading
the delegate. Otherwise it obtains0ZJ.A0K's0vH and delegatesBNJ, normalizing
the boolean. Its handler's message4 forwards through0ZJ.A0t toA0r(Message).
That final handler body and0vH implementation predicates are not traced here.
The established formula is queued-message4 OR delegate.BNJ, not0AI.state==2,
socket-open, authenticated, fully disconnected, or permission to resend.

## Adjacent cancellation map is a separate mechanism

0Sf.A0J synchronizes on0Sf.A0E, iterates its entries as0Sp and invokes
AG3(null) on each, then clears that map. It does not directly reference
0ln.A0I/A0H,0TX's queue, the token manager's AtomicReference, or durable
Rust reservations. AG3 implementations can have further effects and are not
interpreted here. Therefore the adjacent call is not evidence that queued IQ
callbacks, tokens or every transport intent were cleared or safely cancelled.
A thrown callback can also leave the loop before clear; no universal successful
cleanup is inferred from this selected method's presence.

## Implementation boundary

This narrows shutdown argument/eligibility and cancellation-map ownership;
it does not establish full reconnect policy, delegate semantics, callback races,
queued-request fate after every shutdown, generation ownership or durable
recovery. Rust uncertainty/no-replay behavior remains unchanged. No sender,
automatic retry, route or live account operation is added.
