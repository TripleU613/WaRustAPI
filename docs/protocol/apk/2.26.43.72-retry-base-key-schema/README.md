# Retry base-key schema, replacement and deletion — APK 2.26.43.72

**Confirmed selected static.** Build264307238, pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Four complete method exports, three bounded source-hash/byte-length-checked excerpts and three string/reference inventories. The string inventory preserves exact stdout, including trailing whitespace in APK strings, inside a JSON object with its original byte count/hash. Method exports include exception tables; excerpts explicitly omit other instructions and exception tables. These are repository xtask outputs, not REA Evidence. No Android execution, account request, live database or server acceptance was observed.

## Replacement and fresh schema

The prior [message base-key writer](../2.26.43.72-retry-message-base-key/save-message-base-key.txt) calls 0KU.A08. Its [complete body](replace-helper.txt) invokes SQLiteDatabase.replaceOrThrow with the supplied table and ContentValues and a null null-column-hack argument, then returns the row ID. The selected corruption catch calls 0KU.A00 and rethrows. This binds the replacement API; full transaction/crash policy and every error consumer remain untraced.

[28o.onCreate at0080..0094](fresh-schema-excerpt.txt) executes table creation and the unique index `message_base_key_idx` on `(msg_key_remote_jid, msg_key_from_me, msg_key_id, recipient_id, recipient_type, device_id)`. The table has an autoincrement `_id`; message remote/id and base-key blob are NOT NULL; from-me, recipient type and device default0; `recipient_id INTEGER` and `timestamp INTEGER` are nullable. Do not claim unconditional one-row guarantees for every malformed/legacy/null state from this selected DDL.

The writer supplies recipient ID with ContentValues.put(String,String), despite the INTEGER column declaration. A blanket Rust i64 cast is not established. The selected upgrade's explicit legacy PN cast is a separate migration operation, not the normal writer's producer.

[Schema strings](schema-strings.json) include three-, five- and six-column index forms. [Exact references](schema-xref.txt) distinguish the six-column fresh creation from upgrade history. The [bounded upgrade excerpt](upgrade-schema-excerpt.txt) includes the new-version40 comparison, early three-column creation, recipient/device additions, selected non-PN deletion/PN cast and later recipient-type addition/six-column index. It omits other control flow: full old-version thresholds, provider invocation and migration outcome are not proved. This is not evidence that any user's database is current or should be migrated/reset.

## Explicit deletion paths, not an established TTL

[0WJ.A0r](delete-message-coordinator.txt) builds original plus normalized address members, uses the selected 0Wk set-coordination helpers and local database transaction scope, and invokes EbJ.A00 for each address. Normal completion calls1hk.A00, closes handles and calls0Wk.A03. Full coordination/commit helper semantics remain separate.

[EbJ.A00](delete-message-store.txt) matches message remote/id/from-me and recipient ID/type/device, using `!=0` for from-me=true and `=0` for false. It delegates to [0KU.A04](delete-helper.txt), which invokes SQLiteDatabase.delete with the supplied predicate/arguments and returns the affected-row count. No timestamp predicate appears in this selected single-message delete.

[Fc1.run selector17](bulk-delete-excerpt.txt) takes a supplied address-to-set-of-message-keys map. It remaps each address, puts the supplied set under original/mapped addresses in a new map, coordinates those addresses, then calls EbJ.A00 for each address/key pair inside its selected transaction scope. Repeated map keys use Map.put; input collision/coalescing constraints and scheduler/trigger are untraced. No timestamp cutoff is established by this selected case. Other switch cases/exception tables are omitted; this is not the entire runnable.

The exact `message_base_key` [reference inventory](table-xref.txt) also finds1jB.ATY/ATc. A [subsequent declaration trace](../2.26.43.72-retry-base-key-declarations/README.md) resolves them as schema declarations with resetting column-builder state; provider activation and actual application remain untraced. Consequently no global cleanup absence, periodic TTL, all alias-migration semantics or complete lifecycle is claimed. No Rust runtime/store migration, count limit or network behavior changed.

Reproduce with `python3 docs/protocol/apk/2.26.43.72-retry-base-key-schema/reproduce.py ../android-lab/apks/WhatsApp.1.apk` after building xtask. The script checks the APK, complete-method identities and every retained export. `checksums.json` covers all files except itself.
