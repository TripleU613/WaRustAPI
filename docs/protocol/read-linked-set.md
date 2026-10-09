# Saved linked-device set used by read receipts

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Nine-topic recipe](apk/recipes/2.26.37.71-read-linked-set.json),
[complete selected methods](apk/2.26.37.71-read-linked-set/targets.json).
Offsets below are method-relative bytes. This extends the saved-set boundary
of [read-job admission](read-receipt-job.md), not a Rust linked-device importer
or permission to adopt arbitrary device identities.

## Eligibility is a saved companion set

**Confirmed:** 0bN.A02 resolves 1IW, then checks 08Z.BNm. False returns
ImmutableSet.of(); true returns the keySet of 1IW.A08. BNm's loaded-Me
presence check reuses [history mode evidence](history-mode.md); it is distinct
from positive registration-device companion mode. The read job's set is not
the remote contact/device directory or a receipt-user timestamp collection.

1IW.A08 iterates 1IY.A07's map and copies only entries whose HxZ.A03 is
false. A03 compares HxZ.A02 to zero and calls 1xi.A1Q, which is exactly
comparison>0. The loader/constructor below establishes A02 as logout_time.
Thus a positive logout time excludes an entry; zero or negative values pass
this filter. This is not proof that the remaining device's key is currently
verified, reachable or entitled to a full cross-device action.

## SQL loading and logout-field binding

**Confirmed:** 1IY.A01 is synchronized and reuses nonnull cached map A01.
A cache miss queries 1Kt.A00 via getDevices/QUERY_DEVICES. 1Kt.<clinit>
constructs SELECT plus the 1Ku.A00 column array, then FROM devices, with
no WHERE/ORDER BY appended in this method. Complete column-array/schema and
store wrapper semantics remain **unknown** here; no account read occurs in
this static trace.

The cursor obtains logout_time's index in v34 at 0074..0080. It reads that
long into v74 at 038e..039a; the HxZ constructor at 0410..0416 receives it
as the third long. HxZ.<init> assigns that long to A02 at 002e..0036.
The earlier long arguments are last_active and login_time; these fields are
not interchangeable with logout_time or receipt-device timestamps.

Each device_id string is parsed with 0DK.A04. A null parse skips the row;
otherwise the loader reads platform/type/history metadata, creates HxZ and
puts the parsed key into an ImmutableMap builder. It caches the built map
and sets A00 to cursor-row count minus map size. Duplicate-key behavior is
delegated to the builder; this trace does not invent a last-wins policy.
Parsing grammar, full metadata semantics and cache invalidation are
**unknown**. Close/catch branches are retained; no retry is inferred.

## Own namespace conversion and collision preference

**Confirmed:** 1IY.A07 obtains the raw loaded map, then the stored own PN via
08Z.Aq0. When PN is present, it constructs a LinkedHashMap in two passes:

1. Copy every entry except a key satisfying 0DE.A0Z whose user satisfies BO0.
2. For those excluded own entries, build DeviceJid(ownPN, original device).
   If construction returns null, retain the original key/value. If the PN key
   already exists, skip this converted entry. Otherwise put that PN key with
   HxZ.A01(newDeviceJid), which preserves all other constructor inputs.

0DE.A0Z's numeric kinds 18/19/25 reuse
[chat counterpart evidence](chat-counterparts.md); BO0's stored own PN/LID
equality reuses [own identity](own-identity.md). This is bounded own-identity
conversion, not a contact alias merge. The device number is preserved and an
already copied PN record wins over the converted own LID record; no general
collision policy for remote namespaces is established.

When own PN is absent, the loaded map remains unchanged. The resulting map
is cached in A02. HxZ.A01 passes the original logout long unchanged, so the
later 1IW active filter still sees it after conversion.

A07 separately re-reads the raw map under the monitor and compares it by
object identity with A09. A changed reference triggers a mixed-JID diagnostic
calculation including A00's skipped-row count and a property gate. Full gate
value, diagnostic effects and cache invalidation remain **unknown**. Returned
map membership is not a verified key inventory or successful receipt send.

## Remaining authority and Rust work

The read job uses this filtered set for its nonempty tests, including
read-self admission. This trace establishes storage and filtering, not the
set's complete ingestion/deletion authority, pairing verification or read-self
recipient fanout. Existing notification/ADV verification traces in
[cross-device synchronization](cross-device-sync.md) remain separate.
No saved set is synthesized from send targets, peer-directory rows or raw
server observations. Full linked-device receipt synchronization and primary
identity-change cleanup remain unfinished.
No Rust runtime, account state, keys, IDs, reservations or live requests changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-linked-set.json \
  --output docs/protocol/apk/2.26.37.71-read-linked-set --check
```

Follow-up: [linked-device cache writes](linked-cache-writes.md) inventories
physical raw/view writers and traces metadata insertion, logout versus deletion,
own-alias selectors and migration resets. Full caller authority, remaining
writer semantics and rollback/callback guarantees remain unknown.
