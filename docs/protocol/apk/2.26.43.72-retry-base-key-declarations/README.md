# Retry base-key declarative schema — APK 2.26.43.72

**Confirmed selected static.** Build264307238, pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Twelve complete selected class/method exports with signatures/exception tables, a declared-override inventory and two bounded provider probes. These are repository xtask outputs, not REA Evidence. No APK execution, database operation, network request or server acceptance was observed.

## Remaining table references are declarations

The preceding [table reference inventory](../2.26.43.72-retry-base-key-schema/table-xref.txt) left1jB.ATY/ATc open. [ATY](index-declaration.txt) supplies `message_base_key`, index name and six-column unique-index DDL to0MZ.CWU. [ATc](table-declaration.txt) builds nine column descriptors and supplies them to0MY.CWk. Neither selected method converts phone/LID identities or message recipients; they cannot be cited as alias-migration implementations.

[0Mg initialization](column-types.txt) and its [constructor](column-type-constructor.txt) bind A07 to INTEGER, A0B to TEXT and A02 to BLOB SQL value strings. [0Me.A01](column-autoincrement-flag.txt) sets A05; [A02](column-primary-flag.txt) sets A08. The [descriptor constructor](column-descriptor.txt) renders those as AUTOINCREMENT and PRIMARY KEY; A06 adds NOT NULL and nonblank A01 adds DEFAULT text. [The blank predicate](default-blank-check.txt) accepts null/empty/whitespace/zero-width-space-only input as blank; literal0 is nonblank.

| Column | SQL type | Selected descriptor suffix |
| --- | --- | --- |
| _id | INTEGER | PRIMARY KEY AUTOINCREMENT |
| msg_key_remote_jid | TEXT | NOT NULL |
| msg_key_from_me | INTEGER | NOT NULL DEFAULT 0 |
| msg_key_id | TEXT | NOT NULL |
| recipient_id | INTEGER | none |
| recipient_type | INTEGER | NOT NULL DEFAULT 0 |
| device_id | INTEGER | NOT NULL DEFAULT 0 |
| last_alice_base_key | BLOB | NOT NULL |
| timestamp | INTEGER | none |

Spaces in SQL suffixes are emitted by the retained constructor/formatter. The table summarizes semantics, not a byte-exact SQL statement.

[0Me.A00](column-build-reset.txt) first constructs a0Mf snapshot, then clears its name, type, default, booleans and auxiliary fields before returning. On normal construction, later columns do not inherit an earlier NOT NULL, DEFAULT 0, primary-key or autoincrement flag. In particular recipient_id/timestamp have no explicit required/default suffix. This corroborates the preceding fresh-onCreate DDL; it does not establish a live database schema or runtime null admission. Failure/reset behavior before the constructor returns is not established.

## Selected sinks register descriptors, not SQL execution

The [declared override inventory](table-sink-overrides.txt) locates0MY's abstract CWk and0Mb's declared implementation, which copies the descriptor array to a list and delegates toCWj with an empty auxiliary list. This is interface-ancestry/declared-method evidence, not runtime resolution or proof of a unique provider.

[0Mb.CWj](table-registration.txt) stores a new0Mm in its A05 map under table name. [CWU](index-registration.txt) stores DDL by index name in A02, appends index name to the table's A03 list and stores index-to-table ownership in A07. The selected methods contain no SQLite execSQL. Duplicate table/index handling invokes00L.A0C(false,diagnostic) **after** Map.put; malformed-index handling invokes the same helper. Assertion policy, rollback and later map consumption are not traced. Do not claim validation-before-mutation or durable installation from declaration alone.

## Negative probe boundaries and remaining gaps

[Provider constructor](provider-constructor.txt) only calls Object.<init>. [Captured probes](provider-probes.json) found zero direct invoke callers of1jB.<init> and no matching1jB.<clinit> method body. These bounded results do not prove the provider unused: inlining, new-instance plus another constructor call, reflection or external registration were not resolved. Provider activation, later SQL application, full migration/alias policy, TTL and crash behavior remain unknown.

**Implementation consequence (inferred):** Rust must preserve nullable/default distinctions rather than inheriting builder flags or treating a declaration as installed storage. Base-key persistence and higher-count retry policy remain unported; this evidence changes no runtime, migration or retry limit.

Reproduce with `python3 docs/protocol/apk/2.26.43.72-retry-base-key-declarations/reproduce.py ../android-lab/apks/WhatsApp.1.apk` after building xtask. All outputs are compared byte for byte; probes preserve both stdout and stderr. `checksums.json` covers every retained file except itself.
