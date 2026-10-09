# Bulk user-receipt import and row-existence boundaries

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Nine-topic recipe](apk/recipes/2.26.37.71-user-receipt-import.json),
[complete methods/signatures/flow/catches](apk/2.26.37.71-user-receipt-import/targets.json),
[string/xref/caller/descriptor inventory and scoped negatives](user-receipt-import-evidence.txt).
This closes an additional receipt_user writer, not complete normal-send seeding
or the still-unported primary cleanup runtime.

## Bulk writer differs from the live one-column writer

**Confirmed:** 8G8.A05(UserJid,178,long,long,long) receives delivery, read and
played milliseconds in that order. At 0008..0024 it loads the message-row cache
through A04 and selects the mapped user through A01. Each positive input is
passed to 7hg.A01 with status 5, 13 or 8 respectively. Nonpositive delivery
selects changed=false at 0148..014e before returning to the read check; it does
not prevent a positive read/played slot from qualifying. If none of these cache
calls changes a slot, 0072..0076 jumps to 01d6 and returns without SQL.

When any slot changes, 0076..009c resolves/asserts a non--1 user row and
009c..00f0 constructs ContentValues with message/user row IDs plus **every
positive input timestamp**. The other positive inputs are not gated by their
own helper-change result, nor read back from the earliest-positive cache.
00f0..0146 chooses the counterpart-aware row set only when chat passes 0DE.A0l,
otherwise the single resolved user row. 0190..0196 calls 0IY.A02 to UPDATE;
zero matches causes INSERT at 019c..01a6. INSERT -1 logs, while the retained
catch paths close/rethrow without any retry loop.

0IY.A02 000c..0016 passes the same ContentValues argument directly to
SQLiteDatabase.update and returns its result. It contains no timestamp min merge.
Its preliminary A01 receives diagnostic/selection arguments, not ContentValues.
The trace establishes the values supplied to SQLite, not all database triggers
or full cache reload behavior. The UPDATE also receives both row-ID members
from that ContentValues, not only timestamps. Interaction with counterpart
rows and the unique index is not fully established; do not replace that scope
with an invented alias merge or suppress constraint failures.

**Confirmed structural consequence / inferred example:** with a cached delivery
of 10 and no read timestamp, inputs delivery=20/read=15 cause the delivery helper
to report no improvement but the read helper to report a change. The UPDATE
then receives delivery=20/read=15, while the cache helper retained delivery=10.
This follows those branches; it has not been observed on a live imported message.
Do not silently substitute a universal per-column minimum merge for this bulk
writer. The [live 15A writer](receipt-user.md#writer-and-row-existence) instead
selects one timestamp column for its admitted status; it remains a distinct path.

## Two actual callers restore serialized receipts

The exact all-DEX invocation scan finds two callers of 8G8.A05:
CnI.A05 at classes7.dex 0x6697be and CoO.A02 at 0x689890. This is physical
call-site coverage, not proof against reflection or alternate generic writers.

**Confirmed:** CnI.A05 loads BU4.message_ as BZY, inserts/converts the message
through its broader importer, then at 0164..0174 gates receipt processing on
message key A02. At 017e..01e8 it iterates BZY.userReceipt_, parses each
6tK.userJid_ with 8kU.A0d, skips a null user, multiplies each timestamp by
1,000 with DEX mul-long/2addr and calls 8G8.A05. The log anchor identifies
ChatTransferMessagesImporter. The key boolean's complete import authority and
all broader message conversion/cancellation/commit gates are **untraced** here.
Its dedicated SQLiteConstraintException branches log and continue; they do not
establish that every error can be swallowed.

**Confirmed:** CoO.A02 similarly checks message key A02 at 0bf0..0bfc,
iterates userReceipt_ at 0c1c..0c40, parses userJid_ with UserJid.Companion.A02,
multiplies delivery/read/played by 1,000 and calls 8G8.A05 at 0c84..0c8a.
The log identifies HistorySyncChunkProcessor. Its full message/history admission
is not ported by reading this subpath. Different user-parser entry points are
preserved; null/throw behavior must not be assumed identical.

**Confirmed descriptor numbers:** BZY.dynamicMethod's newMessageInfo maps
userReceipt to field **40**. 6tK.dynamicMethod maps userJid=1,
receiptTimestamp=2, readTimestamp=3, playedTimestamp=4, pendingDeviceJid=5,
deliveredDeviceJid=6. Numbers were decoded from descriptors, not array positions
or a public schema. Neither caller's traced user loop supplies fields 5/6 to
A05. Pending/delivered device import and other branches remain separate unknowns.
Multiplication here establishes serialized seconds to supplied milliseconds,
including signed-long wrapping; it is not permission to synthesize missing times.

## Schema, scoped negatives and primary cleanup

**Confirmed:** 0Tn.AR1 registers a unique index on
(message_row_id,receipt_user_jid_row_id). AR4 builds six columns: _id,
message_row_id, receipt_user_jid_row_id, receipt_timestamp, read_timestamp,
played_timestamp. AR6 registers schema maintenance through 0Kx.A00/CKN.
These methods do not insert an ordinary-send receipt row. Complete column-builder
flag/default/nullability behavior and maintenance consumers are **unknown**;
numeric builder flags alone are not proof of a timestamp default.

The string scan found 13 strings containing receipt_user; exact literal
receipt_user has 14 instruction references. ChI's two occurrences are backup
metadata collection/map entries; BkU.A0D updates a backup migration-finished
boolean; DOs.invoke's selected occurrence belongs to a schema/column descriptor
collection. Their offsets and reasoning are preserved in the inventory.
Those paths are not normal send-time INSERT evidence. The remaining generic
schema consumers and four named untraced reference sites prevent concluding
that no global seed/import/other writer exists.

**Unknown/unported:** full normal-send user-row seeding, effective 15A admission,
PN/LID counterpart migration, all import trust/transaction gates and cache/SQL
reconciliation. Therefore primary identity cleanup must continue to require
actual receipt_user existence, never an invented row from an immutable target,
a Delivered fact or original pending-device membership. Read-only/played-only
rows can exist in the already traced one-column writer. This bulk import writer
adds another route to existence, not proof that ordinary sends create all user
rows in advance. [Primary selection](identity-receipts.md#primary-receipt-selection)
and [ported exact companion cleanup](companion-receipt-cleanup.md) remain distinct.

This is static research only. No Rust receipt_user writer, history importer,
primary cleanup or resend is enabled. No account, key, ID or server wait was read
or changed; no network request ran. Existing apk-smali, apk-proto-fields,
apk-xref, apk-callers and apk-research-bundle expose reproduction; the research
API entry is CLI-only with x-live=false. Export another anchored generation
before a cross-release cached diff; changed files are not semantic parity proof.

## Release remapping boundary

The [self-remap evidence](user-receipt-remap-evidence.json) verifies that seven
anchor intersections select the original class/method uniquely on this APK.
The general ChatTransferMessagesImporter prefix alone initially matched A03,
A04 and A05; adding the actual xpm-failed-receipt-import literal isolates A05.
Both protobuf descriptor topics use their actual field-name string intersections.
These are verified recipe anchors, not guessed cross-release class names.

Two topics deliberately remain manual: schema maintenance's receipt_user token
matches fourteen methods, and the direct SQLite update wrapper has no exact
string anchor. all_unique=false therefore prevents an automatically emitted
complete proposed recipe. Do not select an arbitrary candidate or retain an old
class on a new APK merely to get a complete report. Re-anchor those edges from
the anchored writer/schema before export. The cached topic diff is fast after
both generations exist; this self-test is not proof that a new release remaps
or behaves identically.

Validation: nine source topics / 14 files / 204,931 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies nine unchanged topics and four metadata
artifacts, not another release's behavior. Self-remap selects the correct original
class/method for seven topics; one is ambiguous and one unanchored, so no complete
proposed recipe is emitted. The committed compact report keeps actual candidate
selections and the full report digest; unrelated per-anchor hit lists stay local.
CLI-reference regeneration, API/library-symbol validation, formatting and diff
hygiene passed. Repository hygiene checked 1,463 local links and 37 bundles /
9,133,957 bytes. No Rust runtime changed, no fresh workspace/runtime test is
claimed, and no account or live operation ran. The prior cfda410 functional
chunk's 810-test validation remains separate.
