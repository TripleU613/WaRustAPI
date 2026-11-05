# Client-auth-token nonce register flow — APK 2.26.43.72

Confirmed static, method-specific conservative control-flow analysis of checked
DEX TXT. Not runtime observations, general DEX verification or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The source method was regenerated from that APK with `xtask apk-smali APK
--class LX/F2Y; --method A02 --signatures --exceptions` and compared byte-for-byte
with the [retained full consumer](../../rea/2.26.43.72-authkey-outcomes/push-consumer-checked.txt).
Its SHA256 is `df65a93664e6d6c9be99020e891993030df015fd0a2b230dc772c00ffed4d5ba`.

Reproduce from the repository root:
`python3 docs/protocol/apk/2.26.43.72-client-auth-token-nonce-flow/check-nonce-flow.py docs/protocol/rea/2.26.43.72-authkey-outcomes/push-consumer-checked.txt`.
The checker rejects any other source digest. checksums.json covers this bundle.

## Original nonce is the sole reaching definition

The [trigger trace](../2.26.43.72-client-auth-token-trigger/README.md) mapped
GcmListenerService.A04's map pn value to F2Y.A02 parameter v69, labelled
pushNonce by the consumer diagnostic. It left an explicit gap because v31,
which initially copies v69 at0134, is reused at1d28 and287e.

The retained checker builds normal edges for every instruction, both sides
of conditionals, all printed switch targets, and exception edges from every
protected instruction to every listed handler. Branch deltas and exception
addresses are code units; printed instruction addresses/switch targets are
bytes. Returns/throws do not fall through. Every edge target must exist.
It intentionally overapproximates exception reachability rather than deciding
which protected instructions can throw or which handler will match.

A worklist propagates v31's most recent definition from method entry over
this graph. There are2585 parsed instruction/payload rows and no invalid
normal targets. The three definitions are0134,1d28 and287e. Only0134 reaches
the2702 move of v31 into the token-manager argument v1. Neither later reuse
can reach2702 without another definition. Entry's uninitialized value does
not reach it. Predecessor-register v30 writes are explicitly checked object
moves, excluding an unnoticed wide write that overlaps v31. The source v69
and predecessor v68 have no writes in this method.

The complete graph's sole predecessors of26fe,2702 and2706 are respectively
26fa,26fe and2702. At26fa the if-eqz on v69 skips to270c; only its nonnull
fallthrough reaches the manager load/move/invocation. Together these facts
close the consumer register-preservation gap: whenever this invocation is
reached, it passes the original nonnull v69 nonce, not either reused value.
This does not prove that every nonnull incoming nonce reaches the invocation;
earlier branches can prevent it. No server acceptance is inferred.

## Limits and implementation boundary

This is deliberately bound to one reviewed export and its three nonthrowing
move definitions. It is not a reusable register-width model or a replacement
for a full Dalvik verifier. Extra handler edges can admit impossible paths;
the unique definition still holds on that larger graph. The final per-definition
reachability lines are graph diagnostics, not claims those branches execute.

GCM field parsing and register preservation do not establish trusted delivery,
origin verification, replay freshness or complete push eligibility. Other
push producers, account/generation binding, executor completion ordering,
request failure/backoff and durable recovery remain open. No Rust sender,
automatic replay, route or live account operation is added. Uncertain holds
remain unchanged. This evidence does not establish ban prevention.
