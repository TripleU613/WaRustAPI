# Tracked queue processing is not a universal commit-before-work gate

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Fourteen complete method exports from xtask apk-smali APK --class CLASS
--method METHOD --signatures --exceptions:2Ek.A00/A01/A02,1il.A00/A01/A03/A07,
1ix.<init>,2JI.run,1id.A08,2Jl.invoke,1iq.A01/A0H,1ic.A0B. DEX text, not REA Evidence
IDs. Only branches named below are interpreted; unrelated coroutine/Runnable
selectors, complete hold policy and disk refill subbranches are not audited.

## Queue consumer and scheduling

[1ix constructor](unordered-queue-constructor.txt) requests backend0x8c9 and
consumer0xd04, then passes them to1id as UnorderedQueue. This does not resolve
the effective injected objects. Previous [arrival hook](../2.26.43.72-count-queue-admission/queue-arrival-hook.txt)
1il.CHq callsA01 whenA07 is true; otherwise it routes a single stanza through
2Ek.A01. [A07](consumer-routing-policy.txt) returns configurationA0J for1in/
1iw and true for the other selected consumer kinds. Do not apply those two
ordered-consumer config branches to unordered processing automatically.

[1il.A01](consumer-scheduler.txt) checks shared pause state, optional1iz hold,
capacity and a compareAndSet(false,true) scheduling flag. It submits2JI(32)
to a ThreadPoolExecutor. These are scheduling checks, not disk-commit checks.
[2JI.run selector32](consumer-runner.txt) enters033c; for1iz it is named
UnorderedStanzaConsumer. It checks pause/hold/budget, calls1il.A03, updates its
cursor throughA00, and passes the returned nonempty list to2Ek.A02 at054a.
Selected cleanup resets its scheduling flag. Full pause/hold/rejection/crash
semantics remain open; no guarantee of eventual processing is inferred.

[1il.A03](consumer-selection-bridge.txt) selects1ix.A08 for1iz and calls
1ic.A0B on its selected list. [1ic.A0B](selection-stage-hook.txt) performs selected stage-hook bookkeeping;
its retained nonempty observer branch throws NullPointerException(onCcqStage).
It has no explicit storage call and does not prove a disk-commit barrier.
Observer configuration/failure policy remains unproven. A03 is a selection
bridge, not a socket writer.
[1il.A00](consumer-cursor-update.txt) advances A08 only for a larger nonnull
returned cursor. That cursor is not proof of ACK or successful disk commit.

## Memory selection does not require a row ID

[1id.A08](queue-selection.txt) waits for the queue load latch and scans memory
under the key-map monitor. Its first path calls1iq.A01 with2Jl(selector3),
takes a bounded prefix and calls1iq.A0H on selected entries before any later
conditional disk refill. When A0E is true it nulls the previous-stanza cursor
before constructing that predicate. Other disk/refill selection and cache-mode
branches are retained but not claimed fully understood here.

[1iq.A01](cache-filter.txt) filters A09 under the monitor using the supplied
predicate, without additional persistence checks. [2Jl selector3](cache-selection-predicate.txt),
04da..0518, checks ordering against an optional prior stanza and requires
A0H==2A3.A05. It does not require A0B (database row ID), A0D or A0I.
The prior [base constructor](../2.26.43.72-count-queue-admission/wrapper-base-constructor.txt)
initializes new wrappers toA0H=A05, A0I=false. Thus a newly admitted in-memory
entry can satisfy this state predicate before acquiring a database row ID.
[1iq.A0H](mark-selected-state.txt) changes selected entries toA06 and updates
the count of entries previously inA05. This is a memory state transition.

**Consequence:** a blanket disk-before-selection rule is not established and
would contradict this selected memory path. Actual processing-versus-disk
ordering depends on scheduling and branch inputs; no runtime race is claimed
observed, nor is disk persistence declared unnecessary for ACK durability.

## Routing selected work and failure distinction

[2Ek.A02](batch-router.txt) separates1k4 wrappers, groups those by stanza class,
and sends each group through [A00](class-group-router.txt). That method requires
a class handler, then invokes [A01](single-stanza-router.txt) for each entry.
A01 looks up1rA and callsA03. SQLiteFullException and
SQLiteDatabaseCorruptException from that call rethrow; LIL, other Exception
and StackOverflowError are handed to1ic.A0E. Missing class handlers throw
outside that selected catch. Full error-cleanup/recovery semantics remain open.
This route closes selection-to-handler admission, not count-specific bulk ACK
list construction or server-confirmed processing.

## Port boundary

The native design separates scheduling, in-memory processing state, raw stanza
persistence, pre-ACK persistence and server confirmation. The linked
[transactional writer](../2.26.43.72-preack-transactional-dispatch/README.md)
still establishes its conditional persistence-before-ACK-admission path.
These processing methods do not merge that boundary into a universal
persistence-before-work rule. Effective consumer injection, initial load,
full disk refill, hold policy, error cleanup and count-specific bulk ACK
provenance remain open. No Rust queue, ACK order, timer, retry or live account
operation changed from this static trace.
