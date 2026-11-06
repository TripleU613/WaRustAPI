# Count queue database sessions share normal transaction primitives

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Eight complete method exports:0aq.<init>/A00/A02,0ar.<init>/A02/close,
0Xp.BIC/Ax0, produced by xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions. 0ar class export contains its three methods.
These are DEX text, not REA Evidence IDs. Only the selected acquisition and
transaction branches below are interpreted; health instrumentation is retained
but not exhaustively traced.

## Session acquisition preserves helper identity

The [previous provider trace](../2.26.43.72-count-database-provider/README.md)
establishes conditional same0Xp identity in a stable provider-cache lifetime.
Its0Xp.A07 uses a read lock and always passes true for writable acquisition.
It selects base0aq or alternate0ar using0H0.A03.

[0aq constructor](base-session-constructor.txt) stores the supplied read lock,
gets helper.Ax0, locks when present and calls helper.BIC for true, B2w for false.
It stores the returned0KU and listener. A failure during database acquisition
logs, callsclose and rethrows. The tracked call chain under audit uses true;
read-onlyB2w behavior is not inferred.

[0Xp.Ax0](shared-listener.txt) returns itsA06 listener directly.
[0Xp.BIC](writable-handle-cache.txt) synchronizes onA0B and returns cachedA00
when its SQLiteDatabase remains open. Otherwise it obtainsA0F, configures/stores
the replacement0KU and callsA0A initialization before returning. Within the
same helper/open-handle interval, callers therefore share0KU and its
SQLiteDatabase; this is not unconditional identity across close/replacement.

[0ar constructor](alternate-session.txt) first invokes the same base0aq
constructor with the supplied helper, lock and writable flag. It then retains
clock/health dependencies and may run health/session-duration instrumentation.
It does not select another database in the selected constructor path. Health
callbacks and their exceptional behavior are not declared equivalent to base.

## Both transaction factories use1hk

[0aq.A00](transaction-entry.txt) passes stored0HC and0KU to virtualA02 with a
null supplied SQLiteTransactionListener. [BaseA02](base-transaction-factory.txt)
creates1hk with null0H0 health state. Override0ar.A02 creates the same1hk with
its retained0H0. Both pass the supplied0HC/0KU unchanged.

The previously retained [1hk constructor](../2.26.43.72-prekey-upload-replies/prekey-ack-txn-begin.txt)
reads0KU.A02 SQLiteDatabase, checks inTransaction, selects
beginTransactionWithListener for the outer path and the nested begin helper
otherwise. The earlier completion bridge retains normal successful outer
callback release versus rollback. These bodies apply to the wrapper factories
above; a different transaction class was not introduced by0ar.

The [queue transaction](../2.26.43.72-prekey-upload-replies/prekey-ack-txn-bulk-transaction.txt)
invokes retirement synchronously, and [1ih.A04](../2.26.43.72-count-completion-retirement/disk-retirement.txt)
acquires its session through that same memoized provider. Under the stated
stable provider/open-handle conditions, the nested chain reaches the same
0KU/SQLiteDatabase and0HC. This closes the prior normal wrapper identity gap,
not proof of every backend/configuration or failure outcome.

## Close and exceptional boundaries

0ar.close may perform health/duration reporting, then invokes0aq.close at00e8.
That is not a separate database-retirement operation. Complete lock-release
behavior when health reporting throws, base close implementation,0H0 transaction
instrumentation and database replacement/reset remain open here.

No static wrapper trace proves power-loss durability, physical fsync success,
all SQLite engine behavior or recovery after ambiguous commit. A faithful Rust
port still needs atomic stanza retirement/pre-ACK persistence and independent
post-commit memory promotion, with explicit uncertain-outcome behavior. No
Rust queue, timer, database mode, retry or live account operation changed.
