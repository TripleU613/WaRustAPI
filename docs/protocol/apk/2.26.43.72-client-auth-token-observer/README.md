# Client-auth-token observer topology — APK 2.26.43.72

Confirmed static checked DEX TXT, not execution observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`, `apk-callers APK --class CLASS --method METHOD`, and
`apk-field-refs APK --class LX/1jA; --field A04 --type LX/0AI;`.
Checksums cover all files. The connection drain is an explicitly labelled
excerpt; other exports retain complete selected methods/classes.

## Notification invocation is not a universal synchronous callback guarantee

07o.A00 partitions its observer iteration into ordinary entries, 0Kn entries
and 07v entries, with 07v taking precedence when both checks succeed. Ordinary
entries go to list v2; their CbC calls are inline at0230. Other lists have
thread/policy-dependent inline or scheduled paths. In particular 0Kn entries
can be sent through0Gf.CbP with3eO selector23; 07v entries can reach A02
scheduling, potentially partitioned by a configured list. These paths must
not be collapsed into a single synchronous dispatch assumption.

The previously traced 0AI.A0M invokes this dispatcher while holding its own
monitor and after state2/open. That means an ordinary observer callback can
run before A0M releases its monitor. It does not establish which dispatch
category the token manager occupies, whether it is present, or ordering
relative to scheduled observers. Those membership/type facts remain open.

## Registration comes from a provider-backed observer set

0AI's constructor obtains sets using00D.A05(0x1d2c) and00T.A05(0x1e01),
combines them through0AJ.A00 and passes a00y provider to07o's constructor.
The latter creates07s(provider,comparator) as its observer container. This
shows why looking only for an explicit registration call is insufficient.
0AI.A0L itself schedules Fef selector3; its four exact callers are retained,
but they do not establish token-manager registration.

The sole exact constructor caller of1jA is08o.A0S, which simply constructs
and returns it. The selected constructor does not explicitly register itself.
Only constructor/A01 reference1jA.A04 in the exact field inventory.
00D.A05 delegates toA02, which resolves throughA03; A03 caches nonnull
objects by numeric ID in a static ConcurrentHashMap after a synchronized
second lookup. Resolution delegates to1Uw.A00, routing on bits14..20 of the
ID. IDs0x1d2c and0x101 select its shard0 (A01); the selected shard's later
set-binding branch is not retained or interpreted here. The second set's
00T resolver, lazy set population, manager membership and shared provider
identity across all call sites remain unproven. Cache behavior alone does
not prove those facts or account-generation lifetime.

## Connection-thread drain is a second trigger

0ZJ.A0z's selected block calls1jA.A00 at0b24 before the subsequent conditional
19i.A00 call at0b3e and fetching-client-config diagnostic. The boolean for
that conditional comes from1rj.A05; its meaning is not established here.
A00 itself rechecks readiness throughA01 after taking the pending slot.
Thus this call is not authority to skip the gate or proof a request was sent.
The excerpt does not establish entry predicates, relation to the handler
notification, successful rotation or complete startup ordering.

## Implementation boundary

No runtime sender, replay, route or live account traffic is added. The audit
now distinguishes provider membership, callback delivery and the direct drain.
Next traces: provider set bindings/iteration, token-manager dispatch category,
trusted push origin and consumer control flow, concurrency/generation binding,
and failure/backoff. Existing Rust uncertain holds remain unchanged.
