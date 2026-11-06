# Count completion cache removal and conditional scheduling

**Confirmed static**, WhatsApp 2.26.43.72, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete xtask DEX exports with signatures/exceptions, not REA Evidence IDs.
The known REA Android provider limitation was not retried; these exports do not
claim JADX execution or Android runtime observation. Only branches described
below are interpreted, not unrelated FgR selectors or every scheduling policy.

## Reproduction

From the repository root, for each table row run:

`target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'CLASS' --method 'METHOD' --signatures --exceptions > OUTPUT`

Replace CLASS/METHOD/OUTPUT with the exact entries below (OUTPUT is in this
bundle directory). The empty CLASS in the census row intentionally searches
physical declarations APK-wide; it is not an interface/inheritance census.
APK identity was rechecked with `sha256sum ../android-lab/apks/WhatsApp.1.apk`.

| CLASS | METHOD | OUTPUT |
| --- | --- | --- |
| LX/1il; | CHt | consumer-completion.txt |
| empty string | CHt | completion-declarations.txt |
| LX/1il; | A06 | resume-gate.txt |
| LX/FgR; | invoke | removal-predicate.txt |
| LX/FgR; | <init> | predicate-constructor.txt |
| LX/1iq; | A03 | count-delta.txt |
| LX/07W; | A0S | removeif-bridge.txt |
| LX/07W; | A0T | removeif-list.txt |
| LX/07W; | A0R | removeif-iterator.txt |
| LX/1iz; | <init> | unordered-consumer-constructor.txt |

## Key predicate removes matching entries regardless of queued state

Earlier [1id.A0J](../2.26.43.72-count-completion-retirement/memory-retirement.txt)
0162..01ae constructs a completion-key set, removes map/set entries and passes
FgR(counter,cache,keySet,8) to 07W.A0S for its memory list under the monitor.
The [constructor](predicate-constructor.txt) assigns third argument to A00,
second to A01 and first to A02. Thus [selector8](removal-predicate.txt),
0664..06b2, checks membership of stanza.A05 (2DG key) in that set and returns
that Boolean. For a matching stanza whose A0H is not 2A3.A05 (QUEUED), it also
increments Km2.element. Queued matches are still removed; they simply do not
increment this counter. No row ID, stanza bytes or generation is compared by
this predicate. Do not infer unsafe replacement from the predicate alone:
upstream key uniqueness, locking and descriptor lifetime remain separate work.

[07W.A0S](removeif-bridge.txt) calls A0T with the Boolean true. In the
[RandomAccess branch](removeif-list.txt), predicate-true entries are excluded
while survivors are compacted and the tail is removed. The other branch calls
[A0R](removeif-iterator.txt), whose iterator removes entries exactly when the
predicate Boolean equals that true parameter. This confirms remove-if rather
than keep-if semantics for both selected list paths.

A0J negates Km2.element and calls [1iq.A03](count-delta.txt). A03 invokes the
optional A0G Function1 only for a nonzero delta. This is a callback carrying the
negative number of removed non-QUEUED entries; the callback's concrete counter
storage is not traced here. It is not SQL deletion or an ACK-write count.

## Completion notification can request more work, conditionally

A0J0360..03d8, when its mapped stanza list is nonempty, counts remaining cached
entries that fail 2Aa.A00 (the earlier enum trace identifies completed states as
ordinal greater than PROCESSING), releases the monitor and calls its 1im.CHt
with that count. [Physical declarations](completion-declarations.txt) find
1im abstract plus implementations in 1il, 1in and 1iw. This bounded declaration
search does not resolve the actual DI consumer object for a given queue.

The [1il implementation](consumer-completion.txt) ignores its integer argument,
calls A06 and invokes A01 only when that returns true. [A06](resume-gate.txt)
uses configuration 0x6f06 for an instance of 1iz; other selected kinds use the
1kB.A00 descriptor through 00E.A18. Effective values/defaults and refresh
lifetime are not established. [1iz constructor](unordered-consumer-constructor.txt)
extends 1il; no physical 1iz.CHt is present in the declaration census. These facts
support inherited base behavior for that class, not concrete queue DI identity.

The census also retains 1in/1iw overrides: configuration 0av.A0K suppresses their
super.CHt call when true, while later 1jQ.A05 and 1k7.A03 calls still occur on
normal completion. Those downstream observer/coordination effects are untraced;
do not apply the ordered override policy to the unordered class automatically.

The prior [scheduler](../2.26.43.72-count-queue-processing/consumer-scheduler.txt)
still checks pause/hold/capacity and a scheduling CAS before submitting work.
Therefore completion is a possible scheduling stimulus, not proof that another
job runs, that a socket ACK precedes it, or that disk state is confirmed.

## Port consequences and limits

Memory removal by key, non-QUEUED count adjustment, completion notification,
conditional scheduling, SQL row retirement and server confirmation remain
separate authorities. This closes selected FgR predicate and CHt base-consumer
unknowns from the earlier retirement bundle. It does not close effective DI,
configuration, full pause/rejection policy, counter callback storage, key lifetime,
rollback/crash recovery or native pre-ACK recovery. No Rust network behavior,
key generation, replay authority or live account operation changed.
