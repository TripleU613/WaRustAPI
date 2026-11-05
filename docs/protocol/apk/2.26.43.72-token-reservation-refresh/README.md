# Trusted-contact token reservation: current timestamp, ownership and age floor

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX disassembly, not runtime observations or REA Evidence envelopes.
[Previous full job/callback exports](../2.26.43.72-privacy-token-callback/README.md)
retain GeneratePrivacyTokenJob.A0L and success/error lifecycle boundaries.

## Native current-state and pending-job order

87x.A04's ordinary scheduling branch writes0ht.A0W with current08q.A04 time
before callingA05. A05 checks/inserts its user in the pending Set under a
monitor and returns false for an existing member. Only successful admission
constructs a new GeneratePrivacyTokenJob. Thus advancing scheduled state does
not require another pending job for that user. Full bucket/batch/counterpart
policy remains separate; only the selected order and pending test are used here.

The job'sA0L loads0ht.A0O at execution and uses current7p3.A00, not a queued
snapshot of its timestamp. It refuses missing/older state before construction.
The earlier Rust wire snapshot at queue time could therefore become stale;
multiple expired-window ACKs could also create duplicate unwritten actions.

## A provable necessary age bound, not the full configured threshold

0hu.A00 reads08q.A04()/1000 and computes floor=now_seconds-15724800.
For either nonpositive bucket input it returns that floor. For positive inputs
it returns max((now/bucket_length-(valid_buckets-1))*bucket_length,floor),
including its logged strange-configuration branch. Hence its result is always
at least floor. A02 uses properties865/909; A01 takes min ofA00 for996/997
andA02. Both operands are at least floor, so their min is also at least floor.
The job rejects strict older-than and accepts equality at its effective limit.
A timestamp below the182-day floor is therefore invalid for every configuration;
passing that necessary bound does not prove effective configured admission.

08q.A04 is baseline+elapsedRealtime when its baseline is nonzero, otherwise
currentTimeMillis-A02. It does not select NTP-preferredA00 or inferred stanza
time. Rust now uses its already confirmed clock/offset resolver in the same
write transaction, or local wall time when no confirmed record exists, both
when scheduling after ACK and when checking the age floor. No inferred message
clock is substituted. Full Android clock acquisition parity remains separate.

## Rust implementation

**Implemented offline:** SendState gains an optional current action_id, defaulting
to absent on legacy records. This is durable schedule ownership, never a live
callback. A later eligible ACK advances stored time and reuses a matching Queued
owner; an already reserved/settled owner is never rewritten or reset. A distinct
later scheduling window may create a new intent, preserving old send facts.

Before single/bulk/feature-owned reservation marks Sent, typed token requests
validate original shape and row identity, reload SendState and prepare the node
from its saved timestamp. Only an unwritten Queued row may receive the refreshed
wire, atomically with reservation. The ID/user remain the original ones; now is
used for age checking, never manufactured as a replacement token timestamp.
Once Sent, bytes remain frozen across later schedule changes and reopen.

Missing state, below-floor time and nonowner duplicates remain Queued with an
optional local deferred_reason: privacy_token_state_missing,
privacy_token_expired or privacy_token_superseded. These are not server errors,
proof of delivery or a retry permit. Normal receive continues and other eligible
actions can drain. The reason is the last reservation deferral, cleared when a
request is reserved; actions plain/JSON expose it without user/timestamp/body.
No terminal outcome, deletion or automatic replay is invented for held rows.

Legacy records recover an owner with a complete paginated producer scan: a
matching current timestamp is preferred, and a reserved/successful/refused action
for that schedule prevents an old Queued duplicate from resurrecting it. A new
scheduling window may reuse a proven unwritten legacy row. New indexed records
use direct lookup; historical scans are only for legacy ownership recovery.
A missing/foreign/malformed owner fails the transaction with fixed diagnostics,
rather than repairing correlation or silently replacing IDs. All held records
and original historical wires remain inspectable.

## Verification and explicit remaining work

Offline tests cover stale hold then later-ACK coalescing, refreshed reservation
after disk reopen, current timestamp/ID persistence, single and bulk paths,
immutable Sent bytes and no replay. Independent legacy JSON fixtures cover
queued duplicates and current Sent/Ok/Error recovery. Tests also cover missing
state with unrelated drain, foreign owner rollback/no identifier echo, exact
floor equality versus one second older, and confirmed-offset versus inferred
clock inputs. A normal-driver test writes no expired token while an unrelated
request and subsequent receipt ACK still work. No live account operation.

Effective996/997/865/909 values/precedence, stricter configured expiry, full
native pending/job retry/cancellation, counterpart mappings and success-store
semantics remain unfinished. This necessary floor and ownership port must not
be reported as complete native token policy or eligibility for every target.
Rust does not reconstruct callback authority from action_id or durable Sent.

## Reproduction

`target/debug/xtask apk-smali APK --class CLASS [--method METHOD]
--signatures --exceptions`:0hu full class(expiry-policy),87x.A04(schedule),
87x.A05(pending),08q.A04(clock). Full selected methods retain unrelated branches;
only the paths described above are interpreted. checksums.json covers every
other file. No APK, real account data or bulk tree is committed.
