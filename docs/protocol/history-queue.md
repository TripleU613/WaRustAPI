# History-sync companion queue staging and persistence

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Four-topic recipe](apk/recipes/2.26.37.71-history-queue.json),
[complete methods and catches](apk/2.26.37.71-history-queue/targets.json),
[physical binder/record-constructor callers](history-queue-evidence.txt).
Offsets are method-relative bytes. This extends
[history mapping import inputs](mapping-admission.md), without implementing sync.

## Incoming peer and mode gates

**Confirmed:** 1gz.A00 starts with registered IncomingMessageManager observers;
non-Continue responses can stop processing before the peer branch. At
015a..0172 it compares Bp2.A0D against the string `peer`. The peer path obtains
the message's A0p DeviceJid, takes its userJid and checks 08Z.BO0(user)
(017c..01be) before subtype handling. This is a native own-user predicate,
not a test that any PN and LID attributes are interchangeable. Its complete
implementation, active account settings and upstream message authentication
are **unknown** here.

The Bng subtype branch at 09da identifies itself as
`PeerMessageHandler/handleHistorySyncNoticationMessage`. It then calls
08Z.BMv at 0a00..0a0c. False logs
`ReceiveHistorySyncManager/ Received history sync as primary device`, calls
0zH.A0K(message, 0), and returns through the retained handler path before
queue staging. The companion branch continues. BNm/database readiness,
message-access-status type 8 and other processing branches are retained but
not fully traced. This is a companion history receive path; normal primary
message handling must not invent queued history solely to create aliases.

## Record construction and stable-ID persistence

**Confirmed:** the admitted history branch constructs 1tI from Bng fields at
0b9e..0c08: message key ID, media key/hash/encrypted hash/direct path,
inline payload, encoded handle, sync type, chunk order, file size and start
time. The retained constructor maps its arguments directly to record fields;
no new key generation or ID allocation appears in that constructor.
The subtype/protobuf mapper and exact field numbers remain **unknown** here.

1gz.A00 opens the history-store writer and compiles a twelve-placeholder INSERT
into history_sync_companion at 0cc4..0cd4, binds through 1h9.A01 and executes
HoP.A01. Its SQLiteConstraintException recovery uses 1sl.A00's UPDATE,
rebinds the same record, adds its original message ID as parameter 13 and
executes HoP.A00 (0cf8..0d2a). An update count other than one annotates failure
and rethrows the original exception; it is not a generic refusal reset.
The retained catch table identifies the actual protected regions. Database
schema constraints, transaction wrappers, concurrent update policy and full
message-store transaction authority remain **unknown**.

1sl's initializer builds UPDATE history_sync_companion SET for these twelve
columns, with WHERE message_id = ?. The binder's parameters are:

| Position | SQL column | 1tI field | Null binding |
| --- | --- | --- | --- |
| 1 | message_id | A09 | supplied string |
| 2 | sync_type | A01 | integer promoted to long |
| 3 | chunk_order | A00 | integer promoted to long |
| 4 | media_key | A0B | empty byte array |
| 5 | media_hash | A08 | empty string |
| 6 | media_enc_hash | A07 | empty string |
| 7 | file_size | A02 | long |
| 8 | direct_path | A04 | empty string |
| 9 | local_path | A06 | SQL null |
| 10 | start_time | A03 | long |
| 11 | inline_payload | A0A | SQL null |
| 12 | enc_handle | A05 | SQL null |

**Confirmed:** 1h9.A01 is a statement binder, not itself an INSERT. Its exact
caller scan finds both calls in 1gz.A00. Empty strings/bytes and SQL null have
different meanings here; reproducing the record must preserve that distinction.
Full media-key/hash decoding at restoration is retained in 1h9.A00 but not
claimed as end-to-end media authentication.

## Worker selection and deletion

**Confirmed:** 1h9.A02 first queries history_sync_companion for sync types
0, 1, 4, 5, 6 with chunk_order=0, ordered by sync_type ascending. It returns
the first row through A00. If none exists, it reads 08s.A01's JSON state and
queries the exact saved sync_type/chunk_order; missing state returns null,
and a JSONException logs `Could not read history sync state`. These query
predicates select work, not peer trust or proof that all earlier chunks arrived.
The state producer, sync enum meanings, completion/ordering invariants and
empty-cursor behavior beyond this method remain unfinished.

1h9.A03 deletes a queue row by its original message_id through
`DELETE FROM history_sync_companion WHERE message_id=?`. No queue row,
progress marker, identity, key or outbox reservation was deleted or changed
in this work. Worker completion/delete callers remain untraced.

## Port boundary and reproduction

**Inferred:** the mapping importer sits downstream of a dedicated companion
history queue, with own-user/mode admission and original-ID persistence.
This is stronger evidence than treating decoded BZA bytes as authenticated,
but it still does not prove complete linked-device authority, envelope crypto,
history-media authentication or actual receipt_user row creation. Those remain
necessary before implementing native counterpart/primary cleanup parity.
No Rust runtime or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-history-queue.json \
  --output docs/protocol/apk/2.26.37.71-history-queue --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-history-queue \
  docs/protocol/apk/2.26.37.71-history-queue
```

The checks verify reproducible static evidence, not live sync parity.
