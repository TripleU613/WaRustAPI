# Client-auth-token membership and scheduled delivery — APK 2.26.43.72

Confirmed static checked DEX and class_def metadata, not runtime observations
or REA envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `python3 inspect-interfaces.py APK`. The latter checks the APK
hash, reads DEX class_def/interface lists and prints the transitive declarations
for1jA. It does not infer types from method names. Every file is checksummed.
The two labelled excerpts retain full-signature context and exact offsets;
all other TXT exports preserve complete selected methods/classes.

## First readiness set includes the token-manager binding

The [state constructor and global resolver](../2.26.43.72-client-auth-token-observer/README.md)
obtain00D.A05(0x1d2c). 1Uw.A00 routes this ID toA01. A01 masks0x3fff and
its range chain selectsA0K because0x1c86 <0x1d2c <=0x1d9f. A0K's case7468
selects0612, loads07q.A4c and branches to095a, constructing07r with that
binding-ID array. The static array has17 entries; the first is0x83.
A01 case131 (0x83) invokes08o.A0S at04d4, the previously retained factory
that constructs1jA. This establishes token-manager binding membership in the
first set without requiring the second set's contents to be guessed.

00D.A05 replaces a fresh07r's default resolver with global00D.A05:05f;
00D's initializer creates05g, whose get(id) calls00D.A02(id). Thus member
resolution in this set uses the global cached resolver traced previously,
rather than assuming the default1Vo direct-factory resolver remains active.
0AJ.A00 combines both sets by carrying each binding ID and its source resolver
into parallel lists; A02(index) retrieves the corresponding resolver.
08x.next reads that index's ID and invokes its resolver, caching the resulting
object in the set. This is a static provider path, not proof of delivery during
an observed session or lifetime across cache-reset/account-generation events.

07s keeps the combined-set provider and a comparator. Its sorted branch
uses lazy1X1 selector28 to obtain the provider's Iterable and make a sorted
list; iterator merges this with dynamically registered observer entries.
The retained selected path therefore carries the binding into observer
iteration. Sorting priority and dynamic registration order are not inferred
from the binding array's order.

## Token manager is a scheduled-marker observer

Class metadata declares1jA implements0AK;0AK extends07v. There is no0Kn
or0LB in this transitive interface chain, and1jA's superclass is Object.
Therefore the previously retained07o.A00 partition puts this manager in the
07v list, not its ordinary inline list.

0AI.A0M passes a null dispatch hint. In07o.A00 the null hint takes the07v
branch into scheduling logic at017e. It callsA02 once without a partition
hint, or per configured priority when both lazy predicates are true. A02
can filter on marker/partition and submits1Wd selector3 through an098 executor
looked up/created in a ConcurrentHashMap by its integer key. It does not
invoke this manager's callback directly in the scheduling body.

1Wd's constructor preserves dispatcher, partition hint and notifier. Its
selector3 run calls07o.A01 with booleanfalse. That method obtains a fresh
observer iterator, filters07v entries (and partition when supplied), then
calls notifier.CbC. The previously retained2Gb selector2 invokesBzL, and
1jA.BzL drains the pending slot with a fresh readiness check. The task thus
retains notifier/dispatcher/partition rather than the initial observer list.
A stale notification does not itself bypass the token manager's gate.

This closes the previous membership and dispatch-category gaps for the
selected static path. Executor implementation, actual start/completion order,
partition predicate meanings, provider cache resets, generation ownership,
trusted push origin, consumer register control flow and request failure/backoff
remain open. Submission is not proof that work finished, nor a claim it must
start after the state monitor is released.

## Implementation boundary

No Rust sender, automatic replay, route or live account operation is added.
Uncertain holds remain. This evidence supports retaining a lifecycle recheck
at the eventual send boundary; it does not justify enabling automatic token
recovery or claiming ban prevention.
