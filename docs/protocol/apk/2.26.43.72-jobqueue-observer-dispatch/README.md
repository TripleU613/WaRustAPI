# Jobqueue connection observer dispatch category

**Confirmed static DEX metadata plus retained dispatch bodies**, pinned
WhatsApp2.26.43.72/code264307238, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `c86f5337`. No Android execution, runtime change or live account operation.

[Interface metadata](interfaces.txt) proves RNL implements0AK and RF1;0AK extends
07v. Its superclass is Object and no0Kn/0LB appears in the transitive declarations.
The [hash-checking metadata reader](inspect-interfaces.py) reuses the prior parser
with its root changed from1jA toRNL and its description updated. This is declaration evidence, not an
inference from similarly named callback methods.

The [existing dispatcher trace](../2.26.43.72-client-auth-token-membership/README.md#token-manager-is-a-scheduled-marker-observer)
binds07o.A00's07v partition, null-hint scheduling through07o.A02/1Wd selector3,
and fresh observer iteration in07o.A01 before notifier.CbC. The newly proven
RNL marker chain places this connection observer in that same selected scheduled
partition. The [state-ready producer](../2.26.43.72-client-auth-token-readiness/README.md)
uses a null hint;2Gb selector2 calls0AK.BzL.

[RNL.BzL](../2.26.43.72-jobqueue-wake/observer.txt) checks its saved manager and
callsNhR.A05 only when nonnull. [Compiled membership/association](../2.26.43.72-jobqueue-observer-membership/README.md)
now binds that saved manager throughCgm for the selected construction chain.
The [wake trace](../2.26.43.72-jobqueue-wake/README.md) separately shows A05
submittingQnY selector33; that worker opens the selector's condition, which
causes requirement/group rechecking before job handoff. Neither observer dispatch
nor wake submission is itself a socket write, request completion or server ACK.

This composes two scheduled stages: observer notification and queue wake task.
Executor rejection/teardown, partition predicates, global provider identity,
registration-versus-Cgm races, actual start/completion order and restart policy
remain separate. In particular, constructor-time observer registration precedes
manager association; a callback with a null manager returns without a wake.
No universal delivery/lost-wake guarantee is inferred, and no Rust replay is added.

Reproduce from the checkout:

```
python3 docs/protocol/apk/2.26.43.72-jobqueue-observer-dispatch/inspect-interfaces.py ../android-lab/apks/WhatsApp.1.apk
```

The parser verifies the pinned APK hash. Existing complete dispatch methods are
linked rather than duplicated; checksums.json pins this compact metadata bundle.
