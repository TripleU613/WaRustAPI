# IQ reader admission and unmatched response path — APK 2.26.43.72

Confirmed static checked DEX and class_def metadata, not runtime observations
or REA envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-callers APK --class CLASS --method METHOD`;
`python3 inspect-interfaces.py APK`. The script checks the APK digest and
labels external declarations uninspected. Complete selected methods are
retained except the explicitly labelled fallback excerpt. All are checksummed.
Only identified IQ/callback blocks of large dispatchers are interpreted here.

## Selected IQ processor admits result/error with a nonnull ID

1rA.A03 first selects its1rD subclass branch. It obtains the node, reads type,
id and xmlns. Exact type=result requires a nonnull ID before reaching1lZ.A1E.
Exact type=error takes its own processing branch, then reaches the same
forwarder only when ID is nonnull (00e0..00e6). The error branch can perform
map operations and parse error children before that later test; this is not
permission to accept a malformed/missing-ID error. Parsing can also throw.

Missing type throwsLIL at0680. Unknown types throwLIL at0658. get and set
enter distinct request-handling branches, not the selected response forwarder.
The callback call at0072 has incoming paths from the result branch and the
nonnull-ID error branch. This supports Rust's existing requirement that a
correlated IQ reply be result/error, not merely any IQ carrying the ID.
No from/namespace equality check appears before this selected result handoff;
that bounded observation is not a global absence claim. The token callback's
own request-bound from/header validation still applies later.

## A lower-level callback map acts before the ordinary IQ registry

The1rD branch removes the ID from its ownA00 map as part of result/error
processing. For result, a nonnull1ru record receivesA02(node) before1lZ.A1E.
For error, it invokes family-dependent handling:290 has a specialized path;
other1ru records can receive parsed error codes throughA01. Their complete
side effects and ownership are not reconstructed here. This map is passed
through1r7/1rC into1rD, separate from0ln's queued/sent callback maps.
A result missing from0ln.A0H therefore need not have been ignored by every
reader-level mechanism. No safe replay or settlement rule follows from that
ordinary-registry miss alone.

## Declared root dispatch and its remaining binding limit

1rA.A02 advertises the sole tag iq for1rD. 0ZJ.A19 registers a processor for
each advertised tag and throws on duplicate registration. A0S constructs the
map by iterating its injected factory set;1rC.AL1 constructs1rD from1r7's
request bridge and lower-level callback map. 1rh stores its supplied processor
map. In1s2.run, v10 is the node description; the fallback dispatch looks up
that key in1rh.A01 and calls the chosen processor with a1uU wrapper.
The alternate2Ek dispatcher uses its own supplied map keyed by1kz.

These bodies establish declared tag mapping and selected dispatch behavior.
Membership of1rC in every injected factory set, all alternate-map bindings,
pre-dispatch validation/transforms, duplicate attribute treatment, complete
stanza authentication and all runtime configurations remain open. A03's
standalone subclass branch is not itself a universal root-tag validator.

## Ordinary registry miss has a fallback

1lZ.A1E logs and invokes1lW.C0g(node,id). DEX metadata declares1lY implements
1lW/1lX. Its C0g calls0ln.A0H; a true result returns immediately, while false
posts1lY handler message4 carrying the node and iqId. That handler extracts
iqId and invokes0YS.C0f(id). The retained0YU.C0f implementation posts message8;
0YU's message8 block removes that ID from0Th.A05 and, if a1ue callback exists,
invokesBsT(null). It does not construct/send an IQ reply in the selected block.
Those1ue effects and complete0YS instance binding remain untraced; this is
not proof every unmatched IQ is acknowledged or every fallback is a no-op.

Exact-class/interface caller inventories are retained as bounded physical
sites, not a dynamic census. The callback-router call site is1lY.C0g; the
positive interface call site is1lZ.A1E, whose direct caller is1rA.A03.

## Implementation boundary

This narrows upstream type/ID admission and exposes separate lower-level and
fallback ownership. Complete root/provider admission, lower-level/fallback
callback effects, concurrent/generation binding and durable recovery remain
open. No Rust sender, retry, route or live account operation is added.
Existing IQ grammar, uncertainty and no-replay behavior remain unchanged.
