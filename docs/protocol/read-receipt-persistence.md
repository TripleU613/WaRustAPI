# Local read-receipt persistence and saved positions

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Ten-topic recipe](apk/recipes/2.26.37.71-read-receipt-persistence.json),
[complete selected methods/signatures/catches](apk/2.26.37.71-read-receipt-persistence/targets.json).
This follows [submission](read-receipt-submission.md) and the
[saved candidate selection](read-setting-observers.md). Offsets below are
method-relative bytes; exception-table ranges are DEX code units.
Whole generated workers retain unrelated cases without interpreting them.
No runtime/account/network behavior is changed.

## Message worker is an existing-row update

**Confirmed:** DNe's constructor saves its second object in A00, first in A01,
and selector in $t. Thus A0J's DNe(message,manager,2) reaches run case **2**
at **16bc** with manager A00/message A01. It obtains 0zc through manager.A05
and calls **A0B(message,-1,false)** (16e0..16e6). Its IOException catch at
code units **0b66..0b74 -> 0b74** covers byte 16cc..16e8; handler 16e8 logs
`ReadReceipts/updateMessageToRead Failed to update msg status back to 16`
and returns. This is a swallowed local persistence failure, not a recognized
server reason or permission to retransmit. Other exceptions and scheduling
failures are not universally caught by this selected branch.

0zc.A0B's false Boolean branches directly from 0000 to **025e**, bypassing
its broadcast scan/participant batching path. The -1 argument does not select a
new receipt status in this branch. It calls message.A0h(**1**) to set storage
type, opens 0Gc.A07, builds ContentValues through A03, and updates **message**
with `chat_row_id = ? AND from_me = ? AND key_id = ?` (0334..0342).
A05 constructs those arguments from required key chat -> 0VS.A0C row ID,
key.A02 as exact string `1`/`0`, and original key.A01 ID. No new identity,
message ID or direction is generated here. Full row-ID resolver authority
remains **unknown** in this chunk.

A03 includes these exact values before that update:

| Column | Source |
| --- | --- |
| origination_flags | message.A08() |
| origin | A05 |
| timestamp | A0F |
| status | B3b() |
| receipt_server_timestamp | A0E |
| message_type | A0h |
| text_data | A0g() through nullable helper |
| translated_text | A0V field through nullable helper |
| lookup_tables | Cjb.A00(message) |
| sort_id | A0k |
| message_add_on_flags | A04 |
| view_replies_thread_id | A0P |

The caller already assigned status 16 through A0H. This store rereads B3b();
it does not independently force constant 16. Full concurrent mutation,
virtual getters, nullable helper semantics and lookup mask remain **unknown**.
A0B additionally supplies broadcast from A0Y, recipient_count from A06,
participant_hash from A0U, received_timestamp from nonzero A0C or 08B.A00,
positive A0D as server_sts, and view_mode from A0M or zero. These are local
row values; received_timestamp fallback is not evidence of peer read time.

If A0j <= 0 it queries the same key for `_id` and assigns A0j when found.
No match leaves it unchanged. The selected update return count is discarded;
no insert or explicit transaction-success call appears in this selected path.
Cursor/database close and catch/rethrow paths are retained. Lower-level
transactions/triggers, disk durability and job/store atomicity remain **unknown**.

**Confirmed / bounded:** base 178.A0V() returns 0DE.A0b(key.chat), clarifying
A0J's exclusion at the base implementation. Overrides remain untraced, so this
is not a claim about every subtype. A0h(int) stores A0f after 00K.A0D validates
that existing storage type is zero or equals the supplied value. Its diagnostic
names storage type; this setter is distinct from A0H(status). Complete assertion
helper behavior is untraced. Case-insensitive method names must not be conflated.

## Chat/status sent positions advance separately

**Confirmed:** A0V creates the Object/Object/Object/I overload
3RC(status-map,manager,chat-message-map,**12**). Its constructor stores third
object in A00, second in A01, first in A02; two other same-shorty constructor
signatures are retained but are not this caller. Run case 12 begins at **0cb2**.
The first loop passes chat, selected message.A0j and A0k to **2x8.A01**.
A01 gets the existing chat via 0Fs.A00(...,false); missing chat logs and returns.
It advances only when incoming sort ID is **strictly greater** than cached A0W.
Then A0V receives the row ID and A0W the sort ID, and ContentValues stores
`last_read_receipt_sent_message_row_id` and
`last_read_receipt_sent_message_sort_id` through 0VS.A0Y. Equal/lower sort does
not replace the row. The return Boolean is discarded. Cached mutation precedes
this persistence call; transaction/cache rollback remains **unknown**.
SQLiteDatabaseCorruptException logs/notifies 0Fy.A03 and returns; selected
RuntimeException/Error catches log and rethrow. This is not network retry logic.

The second loop reads the status map's Long value. Missing 0YK.A0L status logs
and continues. It updates 7yi.A05 only for a greater value under a monitor,
clones with A08, then writes the clone's A05 to status column
`last_read_receipt_sent_message_table_id` where `jid_row_id=?` (0e1e..0e4e).
A zero update count logs missing saved status. Catch/close/monitor cleanup are
retained; complete clone, JID-row helper, triggers and storage authority remain
**unknown**. This table ID must not be unified with the chat row/sort pair.

**Inferred port consequence:** native “sent” positions and status 16 mark local
submission bookkeeping, because these workers are scheduled after queueing and
contain no server-ACK wait. They are not evidence of handset reading or confirmed
wire delivery. The candidate SQL uses these same saved positions; reproducing
its selection requires durable original row/sort identity, not synthetic current
timestamps. A Rust port still needs complete queue/store failure reconciliation,
effective read policy and worker authority. Receipt_user initialization and
primary peer identity cleanup remain unfinished. No automatic receipt replay,
HTTP endpoint or new live mutation is introduced.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-receipt-persistence.json \
  --output docs/protocol/apk/2.26.37.71-read-receipt-persistence --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-read-receipt-persistence \
  docs/protocol/apk/2.26.37.71-read-receipt-persistence
```

Verification: exact APK regeneration matched **15 files / 254,389 bytes**.
Cached self-diff checked ten unchanged topics and four metadata artifacts;
this establishes same-APK reproducibility, not cross-release semantic parity.
CLI-reference regeneration/coverage, OpenAPI library-symbol coverage, formatting,
whitespace and repository hygiene passed (1,896 local links; 83 checksum bundles,
17,906,555 bytes). No workspace runtime tests were repeated for this source-only
chunk, and no live persistence or handset-read result is claimed.

## Release relocation (2026-10-09)

**Confirmed local/static:** the separate
[relocation recipe](apk/recipes/2.26.37.71-read-receipt-persistence-remap.json)
adds checked locators without rewriting the original checksum generation.
[Compact self-remap evidence](read-receipt-persistence-remap-evidence.json)
preserves actual selected candidates, signature/caller filtering and full-report
digests. It uniquely finds the same six methods on this APK: message store,
message values, message worker, positions worker, chat position and storage type.
Worker strings locate the method, not a promise that a new release retains the
same discriminator number; new case routing still requires source review.

The message-values column intersection initially matched **three** scopes:
0S6.AR4 (VLL schema), 0VA.A00 (LLLL) and 0zc.A03 (VLL actual writer).
The checked VLL shorty removes the LLLL method; the physical caller anchored by
both UPDATE_MESSAGE_MAIN_SQL and FIND_MESSAGE_MAIN_SQL selects only 0zc.A03.
The caller scan is symbolic direct invocation evidence, not dynamic dispatch
or a claim that no other writer exists. Column-name matching alone would have
been ambiguous and could select schema code rather than the actual writer.

Four topics remain **unanchored/manual**: message-exclusion, message-key,
message-worker-ctor and positions-worker-ctor. They contain no selected exact
string locators in this recipe. Follow calls/fields from the newly relocated
methods and review complete signatures before supplying a new hash-pinned
export recipe. `all_unique=false` blocks automatic proposed recipe output;
retaining old class names does not count as a new-release relocation.
Only WhatsApp.apk was found in the scoped Downloads APK/XAPK/APKM file scan;
no second release was compared here, and no broader resource absence is claimed.

```sh
cargo run -p xtask -- apk-research-remap ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-receipt-persistence-remap.json
```

The checksum-verified cached comparison of the original generation with itself
completed in **0.014920 seconds**, with ten unchanged topics and four unchanged
metadata artifacts. That observed local timing excludes APK decoding, export
and remapping; it is not a cross-release speed/parity guarantee. After exporting
a separately reviewed new generation, use apk-research-diff on old/new directories
for file-level changes and their Rust owners. These recipe/report additions
introduce no runtime settings, row migration, receipt replay or HTTP capability.

Relocation verification: all six unique candidates match their original class
and method on 37.71; four remain unanchored. The explicit --output check returned
`recipe not written: unresolved anchor scope, method shorty or caller evidence`
and created no proposal file. CLI-reference regeneration, OpenAPI library-symbol
coverage, formatting, whitespace and hygiene passed (1,910 local links; unchanged
84 checksum bundles / 18,046,975 bytes). Original checksum artifacts were not
rewritten. No new runtime tests or live operation are claimed.
