# Contact protocol error metadata and durable backoff scope

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No live account operation, target execution or invented REA Evidence ID.
This continues the [contact queue/timer audit](../2.26.43.72-contact-queue-atomicity/README.md).
The findings identify a runtime port obligation, not a completed cooldown port.

## Protocol metadata and contact type are separate

0pD.CK4 loads usync/result/contact when the original H6l selected contact
protocol, passes it through A01 and retains HLG. HLG's constructor establishes
A05=success Boolean, A01=error Integer, A03=backoff Long, with separate fields
for integrity/recovery and refresh metadata. A01's contact-specific integrity
branches include pass, pending, timelock and a snapshot_recovery path guarded
by property22410 (0x578a). Those strings alone must not be converted into a
made-up global restriction or account-absence result.

A01's error-child branch reads backoff with absent-value default7200 seconds,
multiplies by1000 using Java long arithmetic, and applies Math.min(...,3600000).
This is a one-hour cap, not a minimum. It retains the error code (default-1)
and returns A05=false. A no-error branch returns A05=true and optionally parses
refresh. The retained0Tp.A08 only uses its default for absent attributes; present
values delegate to A09. Malformed-present parsing and all overflow/exception
semantics must not be replaced by an invented zero/default policy.

CK4 reports unsuccessful contact protocol metadata through0p9.BLg with captured
context and SID, code and parsed backoff, then continues its wider result
processing. Per-user in/out/invalid is parsed separately. Thus a protocol-level
failure must not simply be renamed to per-user out, nor must all users/LID
results automatically be discarded without tracing the selected consumer.
Existing [lightweight query](../2.26.43.72-contact-queue-atomicity/lightweight-query.txt)
also handles an empty result with contact code429 as a rate-limit outcome.
The full partial-result/adoption policy is still open.

## Observer binding determines the persisted scope

| Observer | Positive backoff writes | Additional conditional write |
| --- | --- | --- |
|0pA (regular ContactSyncHelper) |contact_sync_backoff |code503 also global_backoff_time; predicate0vC.A00 plus code429 also delta_sync_backoff |
|Hzo (ContactQuerySync) |sidelist_sync_backoff |code503 also global_backoff_time |
|Hzq (contact sync manager) |contact_sync_backoff |code503 also global_backoff_time; predicate0vC.A00 plus code429 also delta_sync_backoff |
|Hzp (ContactMexDeltaSyncHandler) |selected BLg logs only |no persistence in this selected method |

These are native protocol codes, not HTTP response statuses or retry permits.
Nonpositive backoff skips the selected persistence branches. Positive durations
are added to08q.A00 to form absolute millisecond deadlines. Constructors bind
0p7 to0pA. HxV's A0H lazy helper uses IDA selector44, which creates0pD withHzo;
this is the same A0H used by its lightweight query. IDA.A01 and its constructor
retain the supplied discriminator directly; the complete small helpers are
retained alongside the selected invoke branch. 0pB.A00 passes its observer
argument directly to0pD. Selected constructor/selector excerpts preserve those
links without committing entire unrelated lazy-provider switch bodies.

This scope difference matters: a contact-shaped usync envelope alone does not
identify which persisted timer applies. The Rust contact-check factory has not
yet been bound to that complete native policy. This audit does not choose a
scope from protocol vocabulary, blanket-block all IQ traffic, or assume every
MEX delta handler writes the same wait.

## Deadline read and clock domain

The retained0wB.run executor excerpt reads global_backoff_time with default-1,
gets now via08q.A00 and bypasses subsequent dispatch when deadline>now. Equality
does not take that backoff branch. The log explicitly identifies the contact
sync request executor; global here does not prove a process-wide network ban.
Complete caller/factory reachability and all per-scope read gates remain open.

Current08q.A00 prefers nonzero A03+elapsedRealtime, then A04+elapsedRealtime,
then currentTimeMillis-A02. It differs from the A04 domain used for token
scheduling, and is not an inferred message/stanza clock. Native NTP acquisition
and full A00 parity remain unported. A future Rust wait must retain its source,
units and clock assumptions rather than silently substituting an unrelated clock.

0pQ.A02 explicitly resets the listed synchronization backoffs to-1, including
global. The selected reset method alone does not establish when the application
is allowed to clear them; it is not permission to reset waits during retry.

## Current Rust gap and next work

contacts::ingest projects per-user data and outer action errors; it does not
model the complete contact protocol integrity/backoff result or observer-scoped
absolute waits. actions::record_reply's generic nested-error inspection does
not reach this third-level protocol metadata. A successful outer IQ therefore
is not proof that contact protocol succeeded. This is a confirmed local coverage
gap, not proof of a server restriction cause or a completed runtime correction.

Next port requirements: bind the contact-check native factory/observer policy;
trace each applicable read gate and reset authority; model protocol metadata
without conflating it with per-user contact type; persist waits atomically with
accepted reply ownership; recheck eligibility before reservation; preserve
uncertain writes, pending rows and unrelated receive/ACK work. No auto retry,
wait deletion, fabricated integrity data or historical callback replay is justified.

## Reproduction and verification

Existing xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions:0pD.CK4/A01/<init>,HLG.<init>,0pA/Hzo/Hzp/Hzq.BLg,
0pQ full class,08q.A00,0Tp.A08,0pB.A00,0p7/HxV constructors,IDA.invoke and
0wB.run. Excerpts are explicitly marked with full-export SHA256. The previous
bundle retains HxV.A0B and caller-dependent dispatch evidence. Native APK input
hash, checksum/link hygiene and documentation contracts are checked; no runtime
eligibility or server acceptance is asserted. Checksums cover all other files.
