# Privacy observer binding and separate receipt callbacks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[16-topic recipe](apk/recipes/2.26.37.71-read-setting-observers.json),
[signatures, complete selected operands and catches](apk/2.26.37.71-read-setting-observers/targets.json),
[independent member-array bytes and physical references](read-setting-observer-members-evidence.txt).
This extends [fetch/observer](read-setting-bootstrap.md) and
[pending-clear](read-setting-pending.md) evidence. It closes callback identity
and generated membership, not complete delivery/queue atomicity or a Rust port.
Offsets below are method-relative bytes. Unrelated factory/lambda branches remain
uninterpreted; their retention does not establish every component's semantics.

## Category/value events differ from list notifications

**Confirmed:** IUr.COL selectors 4 and 5 both take its two saved strings,
cast the recipient to J31 and invoke **Bsa(category,value)** (000a..002a).
0sv.A0R's admitted matching-pending branch emits selector 4. 0sv.A0O's local
integer setter emits selector 5, but explicitly refuses `readreceipts`.
These are the retained [adoption/setter branches](read-setting-bootstrap.md),
not a claim that all Bsa producers have been inventoried.

IUu.COL selector 17 instead casts the saved value to List and invokes
**J31.C5W(list)** (033c..035c). 0sv.A0Q constructs that selector, including
when the delayed runnable clears pending state and passes an empty list.
**DK9.C5W is exactly return-void.** This callback does not execute DK9.Bsa's
read/played work. Conflating the two would create outbound receipt consequences
absent in this particular callback. Other observers/transitive effects remain
**unknown**, so the negative is not universal “no network activity.”

DK9.Bsa's selected gate remains exact category `readreceipts` and value different
from exact `none`. Its first shared helper, 1xo.A1Y, checks both inputs through
00h.A0A and returns **true** (0000..0012). Thus successful helper return supplies
true to 0zH.A0X; it is not a privacy/account predicate or comparison of values.
The current source therefore does not re-read 08l.A1H at this DK9 entry gate.
That does not justify calling Bsa with arbitrary unvalidated values.

## Generated membership reaches the concrete observer

**Confirmed:** 0sv's prior constructor requests 00S.A05 binding ID **7517**
(0x1d5d), wraps the returned set in a provider and passes it to 078.
A05 obtains 06G and delegates to 00S.A04. A04 resolves a component and invokes
1UE.A00 with the original ID. The previously retained
[family router](apk/2.26.37.71-receipt-ack-handlers/factory-router.txt) selects
family zero for 7517, then 1UE.A01. A01 selects A0K for the interval above
0x1c5a through 0x1d71 (0e84..0ea8). A0K case 7517 selects **07A.A6P**
at 080e..0814 and constructs 07B(component,array) at 088a..0896.

07A.<clinit> creates its three-element A6P at 8618..863a:

`[6726 (0x1a46), 98904 (0x18258), 4197 (0x1065)]`.

The raw fixture independently retains initial register constants and the 34-byte
array construction/store, at classes.dex 0x38b214..0x38b236. Physical field
references identify its sole direct field write at 0x38b232. This is not proof
that no array element can later change through an alias.

07B keeps these binding IDs and a component-backed 07C provider. Its iterator
08K.next selects the binding ID, invokes the provider and caches the resolved
object. 07C.get delegates to 1UE.A00. ID **98904** selects family **6**, then
Ayr.A00 masks with 0x3fff to **600**; case 600 constructs **DK9** at 25f0..25fc.
The physical DK9 constructor scan independently finds that one callsite.
The other two member implementations remain **unknown** in this trace.
These are dependency binding IDs, not protocol field numbers or account IDs.

078's constructor installs 07D(provider,comparator); 07D.iterator combines
provider-supplied members with its registered-map keys, using its separate
sorted/cached branch when a comparator is present. 078.A00 partitions observers
by 07G/0L6 marker predicates, dispatches/schedules their COL invocations using
its supplied mode and main-thread checks, and directly invokes COL for ordinary
observers at 0214..0236. Full registration mutation, component lifetimes,
threading/configuration and transitive scheduling effects remain **unknown**.
A generated DK9 member is stronger evidence than a name-only callback scan;
it is still not a live observation that this account executed it.

## Read candidates are saved read state, not fabricated reads

**Confirmed:** DK9.Bsa calls 0zH.A0X(**true**) before the retained played-self
query/queue/cleanup sequence. A0X obtains 2x8 and first queries
available_message_view joined to chat_view. Its selected SQL requires:

- from_me=0 and status **IS NOT 16** and **IS NOT 10**;
- last_read_message_sort_id >= message.sort_id **or** status=17;
- last_read_receipt_sent_message_sort_id < message.sort_id and
  last_read_receipt_sent_message_row_id > 0;
- message_type NOT IN ('10','15'); hidden IS NULL or hidden=0;
- sort_id descending, **LIMIT 4096**.

This preserves SQL predicates, including IS NOT and the OR branch; it is not
an invented universal integer-state model. Cursor restoration through 0V9.A03
must return a message, and message.A0F must be strictly greater than
**1,415,214,000,000** before adding it (00d4..00fc). No new read action or timestamp
is manufactured here. A0F's full producer authority is untraced in this change;
the numeric cutoff is not a server wait or account-verification requirement.

A second status-list query joins message to status by sender_jid_row_id. It
requires the selected chat_row_id, from_me=0, last_read_message_table_id >=
message._id, last_read_receipt_sent_message_table_id < message._id and >0,
and message_type **IS NOT 15**, ordered by message._id descending with
LIMIT 4096. Its separate cursor restoration and result handling remain in the
source. The two different row/sort comparisons must not be unified casually.
SQLite full/corrupt and IllegalStateException paths log/notify local helpers;
full recovery behavior is **unknown** and not automatic network retry authority.

A0X combines the selected collections and calls **A0V(collection,true)**
(036a..037a). A0V's grouped/individual admission and local marking are now followed in
[the submission trace](read-receipt-submission.md); worker/atomicity limits remain
**unknown**:
selection is not proof of actual receipt transmission. DK9 then queries saved
played_self_receipt, groups records through A00, queues
SendPlayedReceiptJobV2(record,false), and deletes selected saved played rows,
as described in the [previous trace](read-setting-bootstrap.md#read-setting-observers-can-schedule-receipt-work).
A00 groups paired row/message IDs under Ci4(destination,participant,false),
then delegates to CcN.A00. Complete grouped-object creation, queue/store atomicity
and played-job execution remain **unknown**. No replay is authorized by this trace.

**Consequence for the port:** receipt scheduling depends on which callback runs,
not only a changed global Boolean. Pending-list notifications do not invoke the
category/value method in DK9. A matching native read-setting completion has an
actual read-work/played-work consumer, while complete other consumers and final
sender authority remain unfinished. Effective preference, fetch/set properties,
bootstrap, receipt_user admission and primary identity cleanup remain unported.
No runtime, private state, keys, request IDs, server waits or live request changed.
No HTTP settings route or new live mutation capability is introduced.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-setting-observers.json \
  --output docs/protocol/apk/2.26.37.71-read-setting-observers --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-read-setting-observers \
  docs/protocol/apk/2.26.37.71-read-setting-observers
```

Verification: exact regeneration matched 21 files / 801,785 bytes; cached
self-diff reports 16 unchanged topics and four unchanged metadata artifacts.
The independent 42-byte member-array/register fixture matches its exact DEX
offsets. CLI-reference regeneration/coverage, OpenAPI library-symbol coverage,
formatting, whitespace and repository hygiene passed (1,878 local links;
81 checksum bundles). No Rust runtime behavior changed; workspace runtime tests
were not repeated, and no live parity is claimed from these static checks.
