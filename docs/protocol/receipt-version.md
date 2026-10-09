# Receipt versions and device-sync resend filtering

2026-10-09. **Confirmed static**, APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [14-topic recipe](apk/recipes/2.26.37.71-receipt-version.json) preserves
[methods, operands, full signatures and catches](apk/2.26.37.71-receipt-version/targets.json).
The [caller/string/negative inventory](receipt-version-inventory.txt) keeps the
ordinary DEX searches. This explains a consumer of the
[ported primary-change counter](primary-device-version.md), without enabling
Rust automatic resends or changing saved send reservations.

## Versions captured for a message

**Confirmed:** 0Z5.A09 delegates to A0D with false. A0D rejects an empty device
set or message row ID -1, obtains the cache/current users' versions through
0Z7.A01 at 0066..006c, and captures that map in DPo selector 1 before entering
its write context. DPo's Object/Object/Object/int constructor assigns arguments
2/1/3 to A00/A01/A02; its nonzero invoke branch therefore holds store/message/map.
It binds message row ID, actual device row ID, then the device user's nullable
version at consecutive indices. HoP.A05 calls SQLiteProgram.bindNull; A06 calls
bindLong. Missing version is SQL null, not a synthesized zero or one.
The [0Z6.A0L INSERT OR IGNORE layout](apk/2.26.37.71-identity-receipts/message-receipt-store-columns.txt)
names these three columns. Complete 7SG
batch execution, cache/commit callback lifecycle and counterpart effects remain
unknown; the bound version dataflow itself is established.

**Confirmed:** 0Z7.A01 first copies nonnull cached versions and collects cache
misses, then queries primary_device_version by user row ID in chunks of 975.
Only found rows are inserted into the returned map/cache. It does not create
missing version rows, increment a counter or replace a missing value with zero.
The cached/current snapshot is read before A0D's later receipt transaction;
no single atomic read/insert snapshot is inferred for the APK.

**Confirmed:** 0Z6.A0M queries device row ID and primary_device_version for the
message. It keeps only resolvable primary devices and nonnull version columns.
For a UserJid chat, it passes that device user through 0Yo.A03 before map insertion;
otherwise it uses the exact device user. 0ZE.A0M has the same primary/null filter
for add-on receipt rows and the same conditional mapping. Effective mapping
and multiple-row winner authority remain unknown. SQL null rows disappear from
these maps; missing map entry does not prove missing receipt rows.

## Updates preserve an existing row's version

**Confirmed:** 0Z5.A0A first filters invalid/raw message state and nonpositive
receipt timestamps, retaining the smaller timestamp for each message/device pair.
The cache operation 7hf.A01 controls which changes reach SQL; its complete
semantics are not newly traced. The UPDATE at 031c writes receipt timestamp
against the message/device-row selection. Only when the update matched zero rows
does 034e..0366 read 0Z7.A00 and add the current nullable version to the INSERT.
It does not refresh a version column on every timestamp update.

70u.A0H is the analogous status receipt writer. Its comparison at 003a..0056
is a receipt timestamp comparison against 7QL.A00, not a version comparison.
The existing-row UPDATE carries timestamp only; zero matches reaches the INSERT
and current user version at 0118..012e. Thus copying current version into every
new receipt fact would not reproduce the native row lifecycle.

**Negative:** querying 7QL.A01:Ljava/lang/Long; found no ordinary field references.
The relevant version is read through the separate SQL/map path, not established
as a second cached-row field. HoP.A04 executes a statement, not bindNull; 8Vw
selector 15 updates the receipt cache, not SQL parameters. Those initial trace
branches must not be reused as version binding evidence.

## The version comparison in SyncDeviceAndResendMessageJob

**Confirmed:** A0G checks login, expiry and other requirements before syncing
and resolving resend candidates. At 05ec..0618 it loads current versions through
0Z7.A01 and the message's prior receipt versions through the selected 0Z5.A0M.
This snapshot is separate from an incoming retry receipt or a generic outbox
reservation. Earlier device/message exclusions are retained in the full method.

At 0646, a candidate failing Cbb.A00 jumps directly to 06ca. Cbb.A00 checks
raw classes Blg/Bli; log context names the true branch hosted-device handling.
The property 0x6ebd test lies in that true branch, not around all ordinary
version comparisons. Its active value and complete hosted policies remain unknown.

06ca..076a looks up both maps by the candidate's user. A nonnull prior version
proceeds to nullable equality through 08p.A00. When prior is missing, missing
current or current exactly 1 selects the special default-transition branch.
Enabled property 0x7dba can admit that branch; when disabled it falls back to
equality. 08p.A00 returns true for two nulls and otherwise invokes Object.equals.
A mismatch fails ordinary devices; hosted-type exceptions have additional
conditions and diagnostic branches outside this ordinary-device table.

Assuming all earlier guards pass and Cbb.A00 is false:

| Prior message version | Current version | 0x7dba disabled | 0x7dba enabled |
| --- | --- | --- | --- |
| absent | absent | keep candidate | keep candidate |
| absent | 1 | drop candidate | keep candidate |
| absent | 0, 2 or another value | drop candidate | drop candidate |
| present | absent | drop candidate | drop candidate |
| equal present values | equal | keep candidate | keep candidate |
| unequal present values | different | drop candidate | drop candidate |

This is equality, not a greater-than test. Signed-long wrap does not authorize
bypassing a mismatch. The log `primary_version_v0_to_v1_transition` does not
mean the database stored numeric zero: the branch actually tests an absent
prior value. After filtering, A0G seeds receipt rows and queues a distinct send
with its existing message through the native sender; full resend requirement,
uncertain outcome, job recovery and wire dispatch parity remain unported.

## Rust port boundary

**Confirmed local:** only the current per-user primary-change counter has been
ported. Native per-message/device send-time receipt versions, pending-row state,
version map canonicalization, hosted feature gates and sync/resend jobs are not
implemented. Existing delivery/read/played facts and receipt-outbox reservations
retain their behavior. Neither this trace nor a changed counter authorizes
clearing an acknowledged or uncertain request, rotating its ID, or re-encrypting
and sending it again.

**Required next state input:** capture the actual per-user version at initial
send preparation for direct and group devices, preserving namespace provenance
and atomicity with the existing reservation. Historical sends lacking a snapshot
remain unknown; do not relabel them as known-null native versions. A snapshot
can distinguish a known missing counter from absence of instrumentation. The
native decision also needs effective feature inputs and complete candidate
selection before it can become a live resend policy.

Reproduce the linked recipe with apk-research-bundle into a new directory, or
--check against the committed bundle. apk-research-diff compares these stable
source topics with mapped Rust files without rereading the APK. Self-comparison
verifies plumbing only. No real account operation or live resend test was made.

Receipt-version research validation: 14 topics / 19 files / 135,279 bytes
reproduce exactly from the comparison APK. Cached self-diff reports all topics
and four metadata artifacts unchanged in 0.022983 seconds, not cross-release
equivalence. All 65 xtask tests, CLI-reference regeneration, API/library-symbol
check, formatting and diff hygiene passed. Repository hygiene checked 1,304
local links and 25 APK bundles / 7,536,554 bytes. Runtime changes are comments
only; no account operation, live resend or receipt reservation change occurred.
