# Tracked unordered queue consumer binding

**Confirmed static**, WhatsApp 2.26.43.72, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete xtask DEX method exports; not REA Evidence IDs, JADX output or runtime
observation. Only the provider IDs/branches below are interpreted. The known
REA Android provider limitation was not retried. No app/account was run.

## Reproduce exports

For each row, from the repository root run
`target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'CLASS' --method 'METHOD' --signatures --exceptions > OUTPUT`,
substituting the exact class, method and bundle-local output name below.

| CLASS | METHOD | OUTPUT |
| --- | --- | --- |
| LX/1Uw; | A00 | provider-dispatch.txt |
| LX/1Uw; | A01 | provider-range.txt |
| LX/1Uw; | A0T | provider-bindings.txt |
| LX/0Zy; | A0J | consumer-factory.txt |
| LX/0Zy; | A0I | hold-factory.txt |
| LX/1k7; | <init> | hold-constructor.txt |
| LX/1k7; | A00 | ordered-head-snapshot.txt |
| LX/00E; | A15 | config-reader.txt |
| LX/00E; | A0D | config-resolution.txt |

The complete provider methods retain many unrelated branches. Their presence
is reproducibility evidence, not an assertion that those bindings were audited.
A preliminary1Uv.A00 export was not used: the actual dispatcher below routes
through1Uw.A01. No inference relies on a guessed adjacent class name.

## 0xd04 constructs the unordered consumer

The earlier [1ix constructor](../2.26.43.72-count-queue-processing/unordered-queue-constructor.txt)
requests05d.A00(0xd04) and passes it as the queue consumer. The prior
[provider resolution/cache trace](../2.26.43.72-count-database-provider/README.md)
resolves the ID through1Uw.A00, caching nonnull results by binding ID within
that provider-cache lifetime.

[1Uw.A00](provider-dispatch.txt) chooses its dispatcher using
`(id >> 14) & 0x7f`. For0xd04 this is0, selecting002e ->1Uw.A01.
[A01](provider-range.txt) masks the ID with0x3fff; at0d74..0d98 the ID is
greater than0xc10 and at most0xd29, selecting1Uw.A0T. The
[A0T packed switch](provider-bindings.txt) maps3332 (0xd04) to0786,
which invokes0Zy.A0J. [A0J](consumer-factory.txt) creates exactly a new1iz
and invokes its no-argument constructor. Thus the selected queue binding has
an actual construction path to1iz, not just a compatible consumer interface.

The prior [1iz constructor and CHt physical declaration census](../2.26.43.72-count-completion-rescheduling/README.md)
show1iz extends1il without a physical CHt override. For this construction path,
completion therefore uses base1il.CHt/A06 with its1iz config branch0x6f06,
not the separate1in/1iw ordered overrides. This closes the earlier selected
consumer construction unknown; cache resets, alternate injection and runtime
object replacement were not observed or exhaustively ruled out.

## Hold owner is distinct from configuration

The same1iz constructor requests binding0xd09 for its A00 hold owner.
That ID passes the same dispatcher/range chain; A0T maps3337 to07ae,
which calls [0Zy.A0I](hold-factory.txt) and constructs1k7. Its
[constructor](hold-constructor.txt),0054..0064, initializes A0A with
AtomicBoolean(false). The earlier [scheduler](../2.26.43.72-count-queue-processing/consumer-scheduler.txt)
reads that atomic and returns while true, in addition to checking shared pause,
configuration, capacity and its scheduling CAS. Initial false is not proof of
its value at a later scheduling attempt. Hold mutation/release is still open.

[1k7.A00](ordered-head-snapshot.txt), inspected while following that owner,
constructs1k9 from the1if and1iu queues' A0A keys through1ic accessors. It is
not the scheduler's A0A atomic read and does not establish a hold-release rule.
This retains the negative result rather than equating equal obfuscated names.

## Boolean configuration reader does not establish the value

BaseA06 calls00E.A15(00G.A02,0x6f06). [A15](config-reader.txt) delegates to
[A0D](config-resolution.txt) with a null Boolean argument, not literal false.
A0D constructs1XK(selector2) and delegates to00E.A06 using its two config
owners, the supplied key and final true, then casts/unboxes the returned Boolean.
Neither declared0x6f06 default nor effective value, callback behavior, cache
refresh or failure policy is established here. Do not interpret the null
argument as disabled-by-default or enable Rust scheduling from this trace.

## Consequences and remaining work

For the selected stable-provider construction, count queue completion can reach
the unordered base consumer, whose existing configuration/hold/capacity/CAS
checks still control work scheduling. This does not prove a job executes,
ACK-before/after ordering, disk commit, server confirmation, or native retry/
recovery parity. Hold mutation, effective config/defaults, scheduler rejection
and crash/rollback semantics remain open. No Rust network behavior, replay
permission, account reset or live account operation changed.
