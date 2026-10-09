# Per-user receipt timestamps and identity-cleanup membership

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-receipt-user.json) and
[retained signatures/branches](apk/2.26.37.71-receipt-user/targets.json).
This extends [receipt application](receipt-apply.md) to the user-row writer,
not a complete 15A admission or Rust mutable receipt port.

## Independent timestamp slots

**Confirmed:** 7lk.<init>(long,long,long) stores arguments into A00, A02,
A01 respectively. 7hg.A01(UserJid,status,time) rejects nonpositive time.
For the 75Y cache kind used by 8G8.A03, status 5 updates A00 (delivery),
13 updates A02 (read), 8 updates A01 (played). Each slot changes only when
its previous value is below 1 or greater than the incoming value; equal/later
positive values return false. A newly created 75a starts with three zero slots.
Updating one slot preserves the other two; read/played do not fabricate a
delivery timestamp. This is earliest-positive retention per user and status.

**Limited:** generic non-75Y caches use 75Z with the third constructor
argument -1. They are not interchangeable with the played-capable 75Y path.
For a positive unsupported status, the helper's branch at 00e8..0124 can put
an unchanged/default cache row and return true; this helper alone is not a
complete status admission guard. The actual SQL writer below rejects statuses
outside 5/8/13. Do not port a permissive helper as public receipt acceptance.

7lk.A00 derives an aggregate status: played when A01>0, otherwise read when
A02>0, otherwise delivery when A00>0, otherwise 4. This highest available
status does not fill absent timestamp slots. The meaning of fallback 4 and
complete downstream message-state projection remain **unknown** here.

## Writer and row existence

**Confirmed:** 15A.A02 at 0a06..0ac8 loads user caches through 8G8.A03 and
calls 7hg.A01 with CYV's status/time. It selects CYV.A03 original UserJid
when supplied, otherwise device.userJid, then maps via 8G8.A01. Only a true
helper result is added to the SQL write collection in this segment.

The write segment at 0ba4..0be0 chooses exactly one ContentValues column:

| Status | Column |
| --- | --- |
| 5 | `receipt_timestamp` |
| 8 | `played_timestamp` |
| 13 | `read_timestamp` |

Other statuses jump to 1222..1254 and throw IllegalArgumentException with
`Unexpected message status ... for user receipt`. This is narrower than
[1hg.A03's six-status factory predicate](receipt-factory.md); the complete
upstream transformations/selection in 15A remain **untraced**, so no claim
is made that self/inactive statuses reach this write unchanged.

15A resolves a non--1 user row via 0VT.A07. It updates `receipt_user` where
message_row_id matches and user row belongs to the selected row set. For chats
passing 0DE.A0l, that set is 8G8.A02's counterpart-aware set; otherwise it is
the single resolved user row. If the UPDATE affects zero rows, INSERT adds
message_row_id and receipt_user_jid_row_id alongside the one timestamp column.
An insert returning -1 logs a failure. Transaction/close/catch branches remain
in the retained method; no blanket retry or failure suppression is inferred.

**Confirmed relationship:** a positive read or played input admitted to this
segment can therefore create a user row whose delivery timestamp was never
written. [Identity cleanup](identity-receipts.md) intersects pending device
candidates with user-row existence, with no timestamp predicate on that query.
Its `getDeliveredMessageIds` diagnostic must not be substituted with a check
for positive receipt_timestamp or Rust's Delivered fact. That would exclude
read-only/played-only user rows the native query includes. Whether a particular
received event reaches this segment still depends on the complete 15A path.

## Loading and mapping

**Confirmed:** 8G8.A03 returns played-capable 75Y caches, using cached hits and
loading misses through private A00. A00 queries receipt_user in 975-row chunks,
selecting message/user row IDs plus all three timestamp columns. It resolves
the UserJid, obtains the requested message and maps through A01, reads the three
values via Cursor.getLong and constructs 75a. It puts the user value without
an earliest-time merge. SQL has no ORDER BY or timestamp predicate. Complete
null/canonical collision precedence is **unknown**; no value is invented.

On cache installation A03 rechecks under the cache monitor. A nonnull loaded
SQL result is put even if another cached value exists; when SQL result is null
it reuses the existing cache or creates empty 75Y. This differs from
[device cache install's existing-value preference](receipt-load.md), so a
shared invented cache race policy would not match both methods.

8G8.A01 maps through 0Yo.A03 only when message chat passes 0DE.A0l, and
returns the original UserJid when mapping returns null. That fallback differs
from the general device loader's nonnull assertion. Complete counterpart
migration defaults and association authority remain **unknown**; Rust's verified
namespace guards must remain intact. The [clock trace](receipt-clock.md) applies
before these millisecond slots; wall-clock observation time is a separate fact.

**Unknown/unported:** complete 15A selection/status transformation, native
receipt_user seeding for all send types, namespace/counterpart mode authority,
cache callbacks and commit/rollback ordering, mutable cleanup and full final
resend recovery. Current independent Rust DeliveryFact records and immutable
original target/version captures remain unchanged. No account operation, message
or receipt transmission was made; this chunk makes no live-validation claim.

Reproduce using apk-research-bundle with the linked recipe, --output and --check.
Use separately anchored generations for new releases before apk-research-diff.
No new CLI/API or runtime capability was introduced.

Receipt-user validation: eight source topics / 13 files / 128,135 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified eight unchanged
topics and four metadata artifacts, not cross-release equivalence. Hygiene
checked 1,381 local links and 32 APK bundles / 8,385,503 bytes; diff hygiene
passed. Research/docs only; no new runtime tests or live operation is claimed.

Follow-up: [receipt admission](receipt-admission.md) now traces positive-time
selection, equal/lower-status device-only handling and the manager pools leading
to these writers. Effective own-device/property/namespace authority and complete
post-write behavior remain unknown/unported.

Follow-up: [receipt-manager exclusions](receipt-manager.md) closes the native
A03 set producer and message deletion/copy context without changing the still
unported per-user admission or identity cleanup. Native revoke deletion is a
message-row-only operation, distinct from pending identity-change predicates.

Follow-up: [bulk user-receipt restoration](user-receipt-import.md) traces the
separate 8G8.A05 writer, its chat-transfer/history callers and descriptor fields.
Its all-positive-input SQL write differs from the one-column live path. Normal
send seeding and complete user-row admission remain unproven; no user rows may
be inferred merely from original targets or Delivered facts.

## Counterpart row set follow-up (2026-10-09)

**Confirmed static:** 8G8.A02 uses migration-gated 0Yo.A0B(input, false),
not the lambda default branch or its true-argument fallback. JID-row creation
is distinct from receipt_user membership; primary cleanup remains unported.
[Complete source, offsets and limits](receipt-counterparts.md).
