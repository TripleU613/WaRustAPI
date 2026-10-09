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
constructor/helper performs SQL or sends a message. **Unknown:** consumers of
these fields, actual generic import/export behavior and the semantic role of
its nullable third string. A descriptor is not proof of pending receipt rows.

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
