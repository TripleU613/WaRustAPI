# Receipt-driven message status projection

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Four-topic recipe](apk/recipes/2.26.37.71-receipt-projection.json) and
[complete new methods](apk/2.26.37.71-receipt-projection/targets.json).
The parent [15A.A02](apk/2.26.37.71-receipt-admission/receipt-admission.txt)
and [13v.A00](apk/2.26.37.71-receipt-admission/user-cache-loader.txt) are reused
without rewriting historical snapshots. All offsets are method-relative bytes.
This closes the post-user-write projection boundary, not complete native
receipt admission or a Rust message-state implementation.

## Per-user aggregate and threshold counters

After the user-write transaction's A00/close and store close at 0d52..0d64,
15A.A02 reaches the single and multipart projection loops. The listener-list
branch at 0d84..0dc6 contains an explicit NullPointerException construction;
its effective list contents and complete callback authority remain unknown.
Do not assume projection always runs after a user-row write.

The single loop at 0dca..0e4e calls private A01 with CYV's message and receipt
status, then posts Handler message what=2, arg1=-1, arg2=0, object=message.
This is local asynchronous notification, not a wire receipt or server ACK.

The multipart loop loads caches through 13v.A00 -> 8G8.A03. At 0ed0..0ee4
it requires a cache keyed by message A0j; missing cache throws rather than
inventing empty membership. It loads **message A06** as the threshold and
iterates every cached 7lk.A00 highest status. The existing
[user-status projection](apk/2.26.37.71-receipt-user/user-status-projection.txt)
returns 8 for positive played time, else 13 for positive read, else 5 for
positive delivery, else 4. Absent lower timestamp slots stay absent.

Branches 0f2c..0f52 count:

| User's aggregate status | Played count | Read count | Delivery count |
| --- | --- | --- | --- |
| 8 | +1 | +1 | +1 |
| 13 | unchanged | +1 | +1 |
| 5 | unchanged | unchanged | +1 |
| other (including 4) | unchanged | unchanged | unchanged |

At 0f54..0f70 it selects 8 if played count >= A06, otherwise 13 if read
count >= A06, otherwise 5 if delivery count >= A06, otherwise 4.
This is a threshold calculation, not the maximum status in any one user row.
For example, threshold two with one played user and one read user computes
13, even though the played user need not have a separate read timestamp.
That example follows the static arithmetic, not live execution.

**Confirmed limit:** this selected block does not validate a positive A06;
zero or negative thresholds satisfy the first comparison even with zero counts.
The log at 10e8..10f8 labels A06 `recipientCount`, but its producer/invariant
must still be traced. The [exact write inventory](receipt-projection-count-evidence.txt)
finds ten physical writes in nine methods, including cursor restoration,
0zt.A0H/A0O and copy/worker paths. Counts cannot be replaced with Rust's current
number of device targets or inferred from only the user rows that have arrived.
Those producer values remain **unknown** in this chunk.

## A computed downgrade is logged, then applied

At 0fbe, exact equality branches to 11e2 and posts Handler what=10 without
A01. Unequal states call 1hg.A00; a positive result takes the downgrade
diagnostic block. That block rejoins **11b2**, which invokes A01 anyway.
A nonpositive comparison also jumps directly there. Both mutation branches
post Handler what=2 and add the event to the changed collection.

**Confirmed consequence:** this multipart projection is not A07's monotonic
state-only admission. It can log a computed downgrade and still invoke the
setter/database update. Imposing A07's refusal here would alter this path.
The array and unknown-status comparator behavior are retained in
[receipt admission](receipt-admission.md#complete-status-array-rendering-2026-10-09).
No raw Rust fact is projected into native status from this finding.

## Status-only existing-row update and view-once hook

15A.A01 first invokes virtual 178.A0H with the requested status (0008..000e),
then opens 0zc's store and updates only `message.status`, with where clause
chat_row_id/from_me/key_id and label `UPDATE_MESSAGE_STATUS_MAIN_SQL`.
0zc.A05 derives ordered arguments: chat row from 0VS.A0C after nonnull check,
fromMe as `1` or `0`, then original message ID. It does not generate another ID.

Exactly one affected row logs success; zero or multiple rows log failure and
still proceed to close, elapsed-uptime telemetry and 8o5.A03. There is no insert,
retry or rollback of the already changed in-memory status established here.
The selected A01 method has no explicit transaction success call; enclosing
store transaction guarantees remain unknown. Exceptions close/rethrow with
suppression rather than proving a successful database write.

8o5.A03 does work only for an **Atm** message with B3b in {8,9,10}. It calls
Atm.CXK(1), then persists Atm.BAA() via 8o5.A00 using original message A0j.
A00 updates `message_view_once_media.state` where message_row_id matches;
it ignores affected-row count and catches/logs Exception as updateInsert failed.
**Negative:** despite that diagnostic, the retained body has no INSERT.
Atm implementations, state integer meanings and complete media consumption
semantics remain unknown. This hook is not a sent opened receipt or a guarantee
of durable cross-device view-once consumption.

## Implementation boundary and reproduction

Primary identity cleanup still needs actual receipt_user membership, complete
ordinary outgoing preparation/status, effective admission/settings and namespace
scope. Projection itself is after the user writer; it does not prove seeding
on send or authorize creating rows from immutable targets/DeliveryFacts.
No Rust messaging runtime, private account, keys, request IDs, waits or network
operation changed. Existing APK CLI tools expose reproduction; no HTTP route
or live capability was introduced. Use the linked recipe with --check and
separate anchored snapshots for other APK releases. Cached self-diff is not
cross-release semantic proof.

Validation: four topics / nine files / 39,768 bytes reproduce exactly from
the pinned APK. Cached self-diff checks four unchanged topics and four metadata
artifacts; it does not validate another APK. CLI reference regeneration,
OpenAPI/library-symbol checks, formatting and diff hygiene passed. This is a
source/docs-only chunk: the preceding 826-test runtime/tool run is not counted
as fresh validation of a runtime receipt projection.

Follow-up: [recipient-count producers](receipt-recipient-count.md) closes
status-audience and exceptional broadcast count writes plus cursor restoration.
Remaining send/copy producers and full user-row admission remain unknown;
no Rust device-list cardinality is substituted for those counts.
