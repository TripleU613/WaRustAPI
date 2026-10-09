# Remaining literal user-receipt reference roles

2026-10-09. **Confirmed static**, WhatsApp APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Ten-topic recipe](apk/recipes/2.26.37.71-user-receipt-reference-roles.json) and
[complete source, signatures, register operands and catches](apk/2.26.37.71-user-receipt-reference-roles/targets.json).
This follows the [four previously untraced literal references](user-receipt-import-evidence.txt).
All offsets below are method-relative bytes. The exact-string inventory still
has fourteen occurrences; closing these four roles does not close every generic
consumer or prove global absence of ordinary-send user-row seeding.

## Backup-change triggers

**Confirmed:** 0LS.AR6(LX/0Kg;)V loads `_id` at 000c..0010 and passes
`receipt_user` with that column to A04 at 0134..013e. A04 registers three SQL
strings through 0Kg.CKO, with `backup_changes` as its first argument:
INSERT and UPDATE use NEW._id, DELETE uses OLD._id. Each goes through A01.
A01 constructs CREATE TRIGGER IF NOT EXISTS / AFTER operation ON table;
its body deletes existing INSERT/UPDATE entries from **backup_changes** for the
same table_name/table_row_id, then inserts the supplied operation into that
same backup_changes table. The affected receipt table appears as the observed
table and as a string value in backup metadata, not as the INSERT destination.
Thus the INSERT constant in A04 is not evidence of receipt_user row seeding.

A02, also used by AR6 for parent-message bookkeeping, similarly constructs a
trigger whose DELETE/INSERT destinations are backup_changes and whose recorded
table_name is `message`. Its complete SQL is retained separately to avoid
mistaking another INSERT literal for a receipt writer. **Unknown:** concrete
0Kg.CKO schema application, installation timing, trigger naming helper A00 and
all generic backup consumers. These source strings do not establish runtime
installation or normal-send row existence.

## Migration error classification

**Confirmed:** 0Uw.A01(LX/B4Q;Ljava/util/List;)V identifies itself with
DatabaseMigrationManager/handleMigrationPhase and DatabaseMigration/doMigration/error
strings. At 0046..004a it obtains the current name from 0Uz.A05 (String field).
The receipt_user literal at 09e2..09e6 compares that name, after the same
receipt_device comparison. At 09f2..0a06 either matching name selects a test of
0Uz.A05()J against signed long **20**. If below 20, A00 receives the migration
name, `stuck` and the preserved Throwable; the other branch continues through
separate error handling. There is no receipt INSERT construction at this
literal site. **Unknown:** the unit/meaning of A05()J, complete migration
implementation and effects of its generic helpers. Do not call 20 a server wait,
retry budget or account restriction from this trace.

## Bulk clear is deletion

**Confirmed:** 0zC.A0Y(ZZ)V logs CoreMessageStore/deleteAllMessages, obtains a
transaction through 0ZN.A00 and at 01de..01ec calls
0IY.A04(`receipt_user`, null, `clearAllMessagesInDB/DELETE_RECEIPT_USER`, null).
Neighboring calls do the same for receipt_device and receipt_orphaned. The
wrapper A04 at 0010..0016 passes table, selection and arguments directly to
SQLiteDatabase.delete and returns the row count. Consequently this site is a
whole-table delete with null selection, not a user-row seed or the selected
identity-change cleanup. The full enclosing catches and boolean branches are
retained, but full bulk-clear transaction/callback parity is **unported**.
No bulk clear, database reset or account operation was executed.

## Receipt table descriptors

**Confirmed:** BcK.<init>()V creates seven CKy cells. The first is
(`receipt_user`, `_id`, null) through CnT.A01; the second is
(`receipt_device`, `_id`, null) through A02. The other five cover add-on device,
orphaned, played-self, coexistence and add-on coexistence tables, with their
actual column strings preserved in the source. Category `receipt` and the cell
list are supplied to CnT.<init>(String,List).

CnT.A01/A02 only construct CKy and store it at array index 0/1. CnT's two-argument
constructor delegates with singleton `message`; its three-argument constructor
only assigns category/list fields A00/A02/A01. CKy.<init> assigns the first three
inputs to fields A03/A02/A01 and initializes empty lists A04/A05. No traced
constructor/helper performs SQL or sends a message. The [follow-up consumer trace](#descriptor-consumers-follow-up) below identifies
backup readers and labels the nullable third string. Complete import authority
and ordinary-send membership remain unknown. A descriptor is not proof of
pending receipt rows.

## Consequence for the Rust port

**Confirmed scoped negative:** all four previously untraced exact literal sites
now have the roles above. None supplies normal-send receipt_user insertion
evidence. **Unknown:** indirect/generic table consumers, ordinary-send seeding,
full live-writer admission, PN/LID counterpart authority and import trust.
The [primary cleanup selector](identity-receipts.md#primary-receipt-selection)
still needs actual user-row existence; immutable targets or delivery facts cannot
stand in for that membership. The already traced live and bulk-import timestamp
writers remain distinct paths, not grounds to invent missing rows.

This chunk is static evidence only. Existing apk-smali, apk-xref,
apk-field-refs and apk-research-bundle reproduce and extend it; no CLI command,
HTTP route, Rust runtime behavior or live capability is added. Raw APK/private
research remain local. String-free helper topics need reviewed relocation on a
new release; the recipe is pinned to this APK and cached diffs are file changes,
not semantic equivalence.

Validation: ten source topics / 15 files / 138,158 bytes reproduce exactly from
the pinned 37.71 APK; cached self-diff verifies ten unchanged topics and four
metadata artifacts. CLI-reference regeneration, API/library-symbol validation,
formatting and whitespace checks passed. No fresh workspace/runtime test or
live operation is claimed; the prior functional validation remains separate.
Repository hygiene checked 1,471 local links and 38 bundles / 9,272,115 bytes.

## Descriptor consumers follow-up

2026-10-09, same pinned APK. [Exact field-reference scans and attributed source
excerpts](user-receipt-descriptor-consumer-evidence.txt) include full method
signatures, excerpt boundaries, enclosing catch tables and SHA-256 of the
reproducible complete local dumps. Excerpts are deliberately scoped; they are
not a complete backup importer or a new checksum-bundle generation.

**Confirmed:** an all-DEX exact CnT.A02:List field scan finds three reads:
Cmf.A03 at 0x653510 / 0x653ed0 and CpC.A05 at 0x6b9788, plus the constructor
write. CKy.A03:String has three reads: CRU.A01 at 0x5abdd2, Cmf.A03 at
0x653576 and CpC.A05 at 0x6b97f4, plus its constructor write. These scans
cover explicit instructions, not reflection or helper reachability.

**Confirmed:** CRU.A01(List):JSONObject serializes CKy.A03 as table_name,
A02 as table_id_field, optional A01 as lookup_field, A04 as deleted_id_files,
A05 as modified_entity_files and A00 as records_count_in_db. It only emits a
selected CKy entry after a nonempty file-list branch; the full body is retained.
The prior unknown third-string label is now established as lookup_field in this
serializer, not a message routing alias.

**Confirmed:** Cmf.A03(CancellationSignal,BiR,DbE,File,long,boolean):void
logs MessageStoreIncrementalBackup/exporting entities at 034e..0358, iterates
the category/table lists at 0388..03ac, reads CKy.A03 at 03ee..03f2 and builds
_deleted_%d.json / _modified_%d.json names. At 066c..06e4 it builds a SELECT
rowid,* FROM dynamic table WHERE rowid IN expression and queries through
0IY.A0A with INCREMENTAL_BACKUP_QUERY_ROWS. At 0d48..0d62 the other category
list read passes to CRU.A01 and places the resulting array into entities JSON.
These consumers export existing backup entities; they do not establish ordinary
send-time pending-row insertion. All export trust/cancellation/storage semantics
and transitive helpers remain **unknown/unported**.

**Confirmed:** CpC.A05(CpC,DbH,List,int):CRd reads the category table list
at 1fc4..1fe8 and table string at 2030..2034 in restoration. Its unknown-table
log is retained at 2066..2070. A separate branch at 2cc8..2d6e converts a
Map into ContentValues using CoR.A04, throws on an unsupported value type and
calls 0IY.A05 with a dynamic table string and diagnostic
BackupChangesStore/INCREMENTAL_BACKUP_INSERT_ROW. **Unknown:** full continuity
from the selected receipt descriptor through restored files/map/table values,
all validation and transaction gates, and whether specific receipt rows pass
this path in practice. Do not claim a verified receipt_user restore or normal
send seed from these excerpts. The generic INSERT is precisely why a literal
receipt_user-only scan cannot prove absence of other writers.

No Rust runtime, HTTP route or new CLI command is added. No backup was restored,
no database row changed and no account/network operation ran. Full normal-send
seeding and effective receipt admission remain the next port inputs.

Consumer follow-up validation: CLI-reference regeneration, OpenAPI/library-symbol
checks, formatting and whitespace checks passed. Hygiene checked 1,475 local
links and the unchanged 38 checksum bundles / 9,272,115 bytes. The new excerpts
are separately attributed, not counted as checksum-bundle coverage. No fresh
workspace/runtime or live test is claimed.
