# Key-based device receipt lookup and missing-message store choice

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Seven-topic recipe](apk/recipes/2.26.37.71-receipt-key-lookup.json),
[complete selected signatures, branches and catches](apk/2.26.37.71-receipt-key-lookup/targets.json).
This follows [property-18225 existing target lookup](blank-receipt-deferred.md)
and [ordinary cache loading](receipt-load.md). Offsets below are method-relative
bytes. Original ordinary/add-on A0O bodies were already retained in the
[sync-resend source](apk/2.26.37.71-sync-resend/original-receipt-store.txt) and
[identity-receipt add-on store](apk/2.26.37.71-identity-receipts/addon-receipt-store-columns.txt).
This trace reuses that evidence rather than treating those implementations as
absent; the base 0Z5.A0O declaration alone was not full override coverage.

## Store selection reads the original ordinary message

**Confirmed:** 0Z4.A01(manager,key) obtains 0V9 and calls Aon(key).
A returned **1im** selects the injected 75W store (001c..003e).
Any other **nonnull** message selects ordinary **0Z6** (003e..004c).
A **null** lookup selects **0ZE** add-on store (0042..0048).
This null branch does not return an empty ordinary cache or prove the original
message was deleted. Full 75W semantics and dynamic subtype authority remain
**unknown**; no new Rust subtype inference is introduced.

0V9.Aon delegates to 0VA.Aon. A null key or null chat returns null immediately.
Otherwise 0VA first checks A01's cache path. A nonnull hit returns directly;
full cache loading/invalidation authority remains **unknown** here.
On a miss it opens 0Gc.A06, resolves chat row through 0VS.A0C and queries
**message** with `chat_row_id=? AND from_me=? AND key_id=?` (0048..00a2).
The selected columns include status, timestamps, row/sort IDs, type and flags.
It uses Cursor.moveToLast, then A05(cursor,chat,false,true) when a row exists.
Missing SQL rows return null; there is no ORDER BY in this query, no insert,
no identity adoption and no reconstruction of a missing send in this body.
The cursor decoder is separately retained by [message loading](message-loader.md);
complete cache and row uniqueness/failure behavior remain **unknown**.
Cursor/store close and rethrow paths are retained. This ordinary lookup does
not itself load an add-on message before the selector chooses 0ZE.

## Concrete ordinary and add-on methods differ

**Confirmed:** 0Z6.A0O performs its own 0V9.Aon lookup again. A nonnull result
calls inherited 0Z5.A05(message), the copied key set of 7hf loaded through A0E.
A missing result returns a new empty LinkedHashSet. It may therefore see a
changed lookup result after the earlier store choice; no atomic double-lookup
snapshot is established. The ordinary loader can include pending rows,
[not only delivered devices](receipt-load.md#general-load-retains-pending-rows).

0ZE.A0O instead directly calls its private A00(originalKey), then copies that
7hf map's keys into a new HashSet. It does not require a successfully reconstructed
ordinary message. A00 supplies chat row, direction string 1/0 and original key
ID to **1Yu.A01** SQL, under diagnostic GET_MESSAGE_ADD_ON_DEVICE_RECEIPTS_SQL.
The static SQL builder (0078..00ba) constructs:

```sql
SELECT receipt_device_jid_row_id, receipt_device_timestamp
FROM message_add_on
JOIN message_add_on_receipt_device
  ON message_add_on._id = message_add_on_receipt_device.message_add_on_row_id
WHERE chat_row_id = ? AND from_me = ? AND key_id = ?
```

This SQL has no timestamp predicate, primary-version column or ORDER BY.
For each result A00 resolves DeviceJid through 0VT.A0D, maps its user through
0Yo.A03 only for chat passing 0DE.A0l, preserves the actual device component,
reads Cursor.getLong(timestamp), creates 7QL.A00 and calls 7hf.A00.
That helper's unconditional map.put is [already traced](receipt-load.md),
not the earliest-positive incoming-time update. Null/unresolved device rows
log and skip. Full Cursor null representation, mapping defaults/collision
precedence and rebuilt-device failure behavior remain **unknown**. Its selected
catch table has close/rethrow handling, not the ordinary general loader's
corrupt-database accumulated-map fallback.

**Confirmed consequence:** missing ordinary message lookup can still lead to
add-on device membership through this keyed join. Ordinary/add-on key methods
must not be replaced with one universally empty-on-miss implementation.
Neither path establishes receipt_user row existence, peer read confirmation or
primary identity-cleanup eligibility. Full property-18225 state, status subtype
selection, manager admission and primary cleanup remain unknown/unported.
No historical send, key, identity or receipt reservation is recreated or reset.

## Offline account check and reproduction

**Observed local failure:** the current `receipts` CLI was rebuilt and invoked
with saved profile `l`, offline. It stopped with a missing master-key environment
input (`WHATSAPP_API_MASTER_KEY_L`) before opening the account store. The actual
receipt journal was therefore **not inspected**, and no healthy/corrupt-account
or account-count claim follows. This is a missing local input, not a WhatsApp
refusal. No key value was printed or copied, no account mutation or live request
occurred, and no replacement account/key was created. The preceding synthetic
[inspection correction](receipt-ack-correlation.md#saved-receipt-inspection-diagnostics-2026-10-09)
remains separately verified; it does not substitute for a saved-account check.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-key-lookup.json \
  --output docs/protocol/apk/2.26.37.71-receipt-key-lookup --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-receipt-key-lookup \
  docs/protocol/apk/2.26.37.71-receipt-key-lookup
```

Existing offline research commands reproduce this bounded source scope; a new
release needs independently reviewed selectors. No HTTP capability or Rust
runtime behavior changed in this trace.

Verification: exact regeneration matched **12 files / 57,353 bytes**;
cached self-diff checked seven unchanged topics and four metadata artifacts.
CLI-reference regeneration, OpenAPI library-symbol coverage, formatting,
whitespace and repository hygiene passed (1,927 local links; 85 checksum bundles,
18,104,328 bytes). These establish static reproducibility, not cross-release
or live parity. No fresh workspace runtime tests were repeated for this
source-only chunk; the prior inspection fix's 823-test validation is separate.
