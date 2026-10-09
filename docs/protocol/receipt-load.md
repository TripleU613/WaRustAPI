# General device receipt loading and pending membership

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Seven-topic recipe](apk/recipes/2.26.37.71-receipt-load.json) and
[complete selected methods](apk/2.26.37.71-receipt-load/targets.json).
This closes the general loader boundary from [receipt-cache.md](receipt-cache.md).
It is static evidence, not a Rust mutable ledger implementation.

**Confirmed:** 0Z5.A02(message) looks up its row ID in the 0Z8 cache.
0Z6.A0E returns an existing nonnull cache result; on a miss, it creates a
singleton row-ID -> message-key map and calls static 0Z6.A00. It selects a
returned 7hf value or creates an empty one, then passes it to synchronized
0Z5.A03. A03 rechecks the cache, installs its supplied object only if still
absent and otherwise returns the existing one. It does not overwrite a
concurrently installed cache value. Complete invalidation/load concurrency
across database contexts remains **unknown**.

## General load retains pending rows

**Confirmed:** 0Z6.A00 returns an empty map for empty input; singleton row -1
returns a map containing an empty 7hf without querying SQL. Other inputs query
`receipt_device` by message_row_id IN chunks of 0x3cf (975), selecting
message_row_id, receipt_device_jid_row_id and receipt_device_timestamp.
The SQL has **no timestamp predicate**, no primary_device_version column and
no ORDER BY. A requested -1 row is added with empty 7hf at method completion.
This batching is a local SQL implementation detail, not a network rate limit.

For each returned row it resolves DeviceJid using 0VT.A0D, reads its timestamp
via Cursor.getLong and constructs 7QL with that value in A00. No explicit
isNull call or positive-value guard appears in this path. It inserts through
7hf.A00, whose body is an unconditional map.put, rather than 7hf.A01's
[earliest-positive incoming update](receipt-clock.md). Therefore existing
pending/null-timestamp rows are not excluded by the SQL or positive-time
admission filter. This establishes row membership separately from evidence of
a positive receipt timestamp. Exact Cursor null representation is not newly
established here; a pending row must not be labelled delivered just because
its device belongs to the loaded key set.

The loader constructs no nonnull 7QL.A01 primary version and does not read
that column. The independent 0Z6.A0M version query is documented in
[receipt-version.md](receipt-version.md). Do not infer a saved version from
the timestamp cache. If two resolved rows map to the same device key, put can
replace the first; effective winner is **unknown** because this SQL lacks
ORDER BY. A Rust port cannot manufacture precedence from the observed layout.

Unresolved device rows log and are skipped. A null rebuilt DeviceJid throws.
The retained catch table distinguishes SQLiteDatabaseCorruptException, which
logs and invokes 0Fy.A03 before returning the accumulated map, from other
exceptions and close handling. This is not a general ignore-storage-errors
policy; no equivalent suppression is introduced in Rust.

## Loader mapping differs from incoming updates

**Confirmed:** when the requested message key exists and 0DE.A0l(chat) is
true, the general loader maps the resolved device's user through 0Yo.A03;
otherwise it retains that user. It then rebuilds DeviceJid preserving the
actual device component. The [incoming update](receipt-clock.md) instead
uses 0Yo.A07 through 0Z5.A01. These are different native helper paths.

0Yo.A03 constructs 1wk with discriminator 3; its invoke dispatch selects the
default branch (cases 0, 1, 2 are explicit). Under the 0DE.A0e predicate,
non-0hd.A02 condition and A0H migration gate it resolves the original row
through 0VT.A07, asks 0VS.A0J for a counterpart row and resolves that through
0VT.A0C. A nonnull resolved user returns immediately; otherwise it calls
0Xw.A0I, whose result can be null. Inputs outside those conditions return
unchanged. Thus this loader mapping does not always share A07's explicit
original-user fallback. The downstream UserJid nonnull assertion matters.
Complete predicate/type/migration defaults and counterpart authority remain
**unknown**; this is not authorization to merge unverified Rust PN/LID aliases.

## Consequence for the resend port

**Confirmed relationship:** [SyncDeviceAndResendMessageJob](sync-resend.md)
subtracts the receipt cache's device key set before comparing primary versions.
The loader now establishes that this set can include pending rows, not only
devices with delivery observations. [Identity cleanup](identity-receipts.md)
can delete selected pending rows and invalidate caches. Immutable original send
targets and independent DeliveryFact values cannot alone reproduce that current
mutable membership. They remain valuable provenance and must remain unchanged.

**Unknown/unported:** full 15A message-state admission and receipt_user writes,
namespace/counterpart authority and collision policy, cache-mode defaults and
commit/eviction callbacks, and complete device-fetch/final-resend recovery.
Next port inputs are native receipt_user timestamp/status writers and 15A's
admission branches, not more guessed receipt promotion. No account operation,
query, send, receipt queue or live validation was performed in this chunk.

Reproduce with apk-research-bundle using the linked recipe, --output and
--check; export separate anchored generations for other releases and compare
with apk-research-diff. No CLI/API command or runtime capability changed.

Receipt-load validation: seven source topics / 12 files / 58,000 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified seven unchanged
topics and four metadata artifacts; this does not prove cross-release parity.
Repository hygiene checked 1,368 local links and 31 APK bundles / 8,257,368
bytes; diff hygiene passed. Research/docs only; no fresh Rust or live tests
are claimed beyond the preceding 66-test tooling chunk.
