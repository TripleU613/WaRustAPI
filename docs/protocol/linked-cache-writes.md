# Saved linked-device writes and cache invalidation

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Six-topic recipe](apk/recipes/2.26.37.71-linked-cache-writes.json),
[complete selected methods](apk/2.26.37.71-linked-cache-writes/targets.json),
[physical cache-writer inventory](linked-cache-write-inventory.txt).
Offsets are method-relative bytes except the inventory's absolute DEX offsets.
This extends [read linked-set storage](read-linked-set.md). No local account
records were read or changed by these static tools.

## Complete physical writer inventory, limited semantic closure

**Confirmed:** exact writes-only field scans found seven stores to
1IY.A01 (raw map) and thirteen to A02 (own-PN view), twenty instructions across
fourteen distinct method declarations. The two already retained loaders write
nonnull maps; selected mutation paths below null both caches. Other writers
are 0jV.A05/A0W, 1gz.A00, DLo.run, DM7.run, DNh.run and GEu.A03.
Their transitive producer/authority and exact reset conditions remain
**unknown** here. Scanned field stores are not a whole-program dataflow proof;
mutable collections or callbacks can require separate tracing.

## Metadata insertion

**Confirmed:** 1IW.A0F(HxZ) constructs ContentValues for devices, including
last_active<-A01, login_time<-A08, logout_time<-A02, adv_key_index<-A07,
full_sync_required<-A0D and metadata/history columns. It invokes 0IY.A08
with diagnostic addDevice/REPLACE_DEVICES (0450..0464), calls the inner
transaction A00 and nulls both map fields under the monitor (0464..0474).
It closes the inner transaction/store and calls 1IW.A05 with old/addition sets
before closing its outer transaction/store; later A0A/A04 callbacks follow.
Complete wrapper conflict/return semantics, callback scheduling, rollback and
all supplied metadata authority remain **unknown**. The SQL diagnostic alone
is not proof of durable success or a trusted Signal identity.

The existing [authorized pairing chain](cross-device-sync.md#primary-pairing-completion-and-identity-provenance)
separately traces saved CYl material through DMU/DMk and identity persistence.
This cache-write evidence does not authorize importing an uncorrelated server
JID or overwriting a pinned key. A0F also has namespace/migration setup;
those branches and helper guarantees are retained but not fully interpreted here.

## Logout marking differs from row deletion

**Confirmed:** 1IW.A0E(set,boolean) asserts the set does not contain
08Z.Apz, with never remove my primary device. Empty input takes the terminal
path. Nonempty input with no saved own PN logs and returns without mutation.
The Boolean selects two storage paths:

- True builds logout_time from 08B.A00 and UPDATEs devices for selected
  device_id strings (00ba..0144). It calls transaction A00 and nulls both
  caches before transaction/store close. Rows remain present; a positive
  written logout value makes the read-linked-set filter exclude them.
- False calls 1IY.A08 to DELETE selected devices rows; it also computes an
  auxiliary set through private 1IW.A00. Complete auxiliary effects are unknown.

A0E continues through 1IW.A05, outer transaction close, A03/A0A and A04
notifications, including separate own-LID set conversion. Complete key teardown,
observer effects, caller selection of the Boolean and rollback/callback delivery
remain **unknown**. This is saved own-device removal, distinct from remote
primary receipt_device cleanup and its receipt_user existence predicate.

1IY.A08 constructs device_id IN (...) placeholders from A06's string array,
DELETEs devices, calls transaction A00 and nulls A01/A02 under the monitor
before close. Affected-row count is not used to infer success in this method.
Empty-set handling/SQL-wrapper behavior and full caller authority are untraced;
no broad deletion is inferred or executed.

## Selected raw IDs preserve own aliases, not arbitrary counterparts

**Confirmed:** 1IY.A06 first adds each supplied DeviceJid's raw string to a
LinkedHashSet. Only when its user satisfies stored own-user BO0 does it also
construct stored own LID and own PN DeviceJids with that same device number,
adding nonnull raw strings. It returns the deduplicated array. Missing saved
aliases are skipped; no alias is invented. The helper may therefore select
both stored own namespaces even when supplied input uses just one.

This is the own identity predicate already traced in [own identity](own-identity.md),
not general contact counterpart authority. Full DeviceJid factory constraints
and every removal caller remain separate boundaries.

## Namespace-migration cache writes

**Confirmed / bounded:** 0ja.A0E requires saved own PN; without it, it logs
that rows are left as LID and returns false. The retained body scans devices,
deletes selected duplicate rows and invokes 1IY.A00 with UPDATE_TO_PN before
calling transaction A00 and nulling both caches at 020c..021c.

0ja.A0F requires a Boolean descriptor, property 24184 (0x5e78) and saved own
LID in its entry branches. The retained body scans devices and has duplicate
row deletion/UPDATE_TO_LID diagnostics; it nulls both caches after transaction
A00 at 0364..0374. Complete row selection, duplicate precedence, update helper,
effective gates and migration state machine are **unknown**. These SQL
migrations are distinct from the read-only map conversion in 1IY.A07.
Cache reset alone does not justify running either migration in Rust.

Full ingestion/deletion authority, remaining writer branches, commit/rollback
semantics and linked-device read-self fanout remain unfinished. No Rust runtime,
account identity, key, ID, reservation or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-linked-cache-writes.json \
  --output docs/protocol/apk/2.26.37.71-linked-cache-writes --check
cargo run -p xtask -- apk-field-refs ~/Downloads/WhatsApp.apk \
  --class 'LX/1IY;' --field A01 \
  --type 'Lcom/google/common/collect/ImmutableMap;' --writes-only
```

Follow-up: [metadata cache records](linked-metadata-cache.md) traces 0jV.A05/A0W
and GEu.A03, and reuses the retained 1gz nonce writer. DLo/DM7/DNh runnable
branches and complete caller/transaction authority remain unknown.

Follow-up: [runnable metadata writes](linked-runnable-writes.md) classifies
DLo case 23, DM7 case 2 and DNh case 20. Earlier remaining-branch unknowns
are narrowed; complete caller/persistence/trust authority remains unfinished.
