# Revoke receipt cache callbacks

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Five-topic recipe](apk/recipes/2.26.37.71-receipt-callbacks.json),
[full method signatures, flow and exception tables](apk/2.26.37.71-receipt-callbacks/targets.json).
This closes the two workers left untraced in
[revoke receipt-manager cleanup](receipt-manager.md).

## User callback removes one message-row cache entry

**Confirmed:** 8SJ.run's packed-switch maps discriminator 6 to byte address
05da. At 05da..05ea it casts A01 to 8G8, reads A00 as a long message row
and obtains 8G8.A00, an 0D3 cache. At 05ea..05fc it enters that cache's
monitor, boxes the row as Long, calls remove(Object), and exits the monitor.
Its catch-all branch 05fe..0604 releases the monitor before rethrowing.
The callback does not enumerate users, edit message content, change keys or
write SQL. SQL deletion is performed by the already traced 13v.A02 producer.

8G8.<init> 0006..0018 constructs a plain 0D3 with capacity 250 and stores
it in A00. **Unknown:** every later direct/reflected replacement of that field;
the constructor alone is not an exhaustive lifetime-write scan.
0D3.remove 0008..0034 takes its own lock, removes the supplied key from its
LinkedHashMap, and reduces measured size only when a value was present.
After releasing that inner lock, a present value triggers
entryRemoved(false,key,value,null) at 003a..0042. The retained base
0D3.entryRemoved body returns immediately. Thus constructor-created plain
0D3 instances have no additional eviction side effect in this base method;
this does not establish all subclass behavior or optional device-cache callbacks.
An absent entry is a normal no-op, not a reason to recreate a receipt row.

## Device callback uses the existing selected-row invalidation

**Confirmed:** DLn.run's packed-switch maps discriminator 6 to 01ac.
At 01ac..01be it casts A01 to 0Z5, reads the long A00 message row and calls
0Z5.A07(J); at 01be it returns. No device filter, timestamp test or cache-clear
loop occurs in this callback. The already retained
[0Z5.A07 / 0Z8.A0E evidence](receipt-cache.md#primary-and-companion-invalidation)
shows selected-key removal and optional callback draining. That evidence is
reused, not a newly discovered general clear operation. Optional 7cQ callback
producers and complete eviction ownership remain **unknown**.

The [manager producer](receipt-manager.md) supplies the same message row to
both SQL DELETE statements and these callback constructors. The previously
established [successful-close and rollback closure](peer-identity.md#cleanup-callback-commit-and-rollback-closure-2026-10-08)
controls their execution. No FIFO or exactly-once callback guarantee is added
by these worker bodies. Failed/rolled-back deletion does not authorize applying
successful-close cache effects or exclusion-set insertion in Rust.

## Implementation boundary

**Unported:** native receipt_user membership, mutable receipt_device cleanup,
revoke authorization/classification/projection, rollback-safe cache integration,
full own-device lifecycle, and identity-change fetch/resend recovery. Rust's
immutable original send-target/version snapshots must not be rewritten into
native mutable receipt sets. DeliveryFacts are not proof that receipt_user
rows exist. No message is retransmitted, original target is synthesized, or
account identity changed by this research.

Use existing apk-smali and apk-research-bundle with this hash-pinned recipe;
export another APK to a separately anchored generation before comparing files.
This adds static coverage, no CLI flag, HTTP route or live cleanup capability.
The full synthetic worker bodies preserve other switch branches for verification;
the claims above apply only to discriminator 6.

Callback validation: the five-topic / 10-file / 70,998-byte bundle reproduces
exactly; cached self-diff verifies five unchanged topics and four metadata
artifacts, not cross-release protocol equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting and diff hygiene passed. Repository
hygiene checked 1,436 local links and 36 bundles / 8,929,026 bytes. This final
chunk changes static research/docs only: it does not claim a new workspace
runtime or live cleanup test. The earlier own-LID runtime validation is separate.
No retained account was read or changed during this callback research.
