# Runnable linked-device metadata writers

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Three-topic recipe](apk/recipes/2.26.37.71-linked-runnable-writes.json),
[complete selected run methods](apk/2.26.37.71-linked-runnable-writes/targets.json).
The recipe includes exact SQL diagnostic string anchors for future remapping.
Offsets are method-relative bytes. These are selected branches of multiplexed
runnables, not full interpretations of every case.

This closes the immediate branch classification of the three remaining writers
in [the physical cache inventory](linked-cache-writes.md). It does **not** close
transitive producer/caller authority, in-place mutation, persistence failure
handling or trusted linked-device lifecycle. No account state or live request
changed. The shared alias expansion, one-record lookup and placeholder selector
are separately traced in [metadata caches](linked-metadata-cache.md).

## History access, case 23

**Confirmed:** DLo.run dispatches discriminator 23 to 00fa. It obtains B9V.A09
DeviceJid and the runnable's supplied Boolean A01, writes that value to
ContentValues.history_sync_access_type via 1xl.A1D, expands own aliases and
UPDATEs devices with setHistorySyncAccessType/UPDATE_DEVICES (0160..0166).
It ignores the returned affected-row count, looks up one cached HxZ under the
monitor, nulls view A02 (0172..0176), and writes HxZ.A00 from the supplied
Boolean (017a..017e). Raw map A01 remains present.

After store close, it selects own LID versus PN through property 24184 (0x5e78).
Absent selected own JID reaches a terminal path; companion mode logs that it
cannot send the complete on-demand access notification. Otherwise it calls
0q6.A05 with the selected own JID, DeviceJid and Boolean. **Unknown:** complete
Boolean/access semantics, producer authority, notification wire construction
and later branch effects. A saved access flag alone is not authorization for
history decryption, peer admission or a newly supplied identity.

## Last-active time, case 2

**Confirmed:** DM7.run case 2 starts at 025c and looks up HxZ through 0jV.A03.
Absent record skips the selected update. A present record receives the supplied
runnable long in HxZ.A01 **before** SQL (0284..0288). The method selects its
DeviceJid and computes a later preference-update flag: true when HxZ.A04()
returns true or B6a.ordinal() is 7, 15 or 21; false otherwise. Complete enum
meanings and A04 predicate semantics remain **unknown**.

All admitted records take the SQL last_active update through expanded aliases
and setLastActiveTime/UPDATE_DEVICES (0308..030e), regardless of that flag.
The result is ignored. Under the monitor, A02 looks up one raw-cache record,
view A02 is nulled (031a..031e), and a nonnull record receives HxZ.A01 again.
There is no comparison with the prior per-device last_active value in this
selected branch. After store close, only the flagged path compares against
last_active_companion_timestamp (default -1) and writes the supplied value
when greater (034a..0358). D3f discriminator 26 scheduling follows.

**Unknown:** long producer/unit/time authority, callback completion and failure
consistency. The earlier in-memory mutation means this trace cannot establish
SQL-before-cache atomicity or rollback restoration. The preference comparison
must not be generalized into a monotonic per-device SQL update.

## Nickname, case 20

**Confirmed:** DNh.run case 20 starts at 0aea, obtains a supplied DeviceJid and
String, resolves 1IW through 0jV, and UPDATEs devices.nickname through expanded
aliases with setDeviceNickname/UPDATE_DEVICES (0b68..0b6e). The result is ignored.
Under the monitor it selects one raw-map record, nulls only view A02
(0b7a..0b7e), and writes nonnull HxZ.A05 from the supplied String
(0b82..0b86). After store close, a nonnull 0jV.A03 lookup schedules D3f case 26;
the branch continues toward a supplied 1RT completion. **Unknown:** nickname
validation/producer, complete caller authority, completion and observer effects.
This is metadata, not a verified key transition.

## Bounds and reproduction

The twenty physical stores now have immediate classifications across the linked
set, cache-write, metadata-cache, history-queue and current traces. No full
whole-program trust closure is claimed. Full SQL wrapper behavior, all supplied
values/callers, cache record copying, observer delivery and read-self fanout remain
unfinished; no Rust runtime feature was added by these static exports.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-linked-runnable-writes.json \
  --output docs/protocol/apk/2.26.37.71-linked-runnable-writes --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-linked-runnable-writes \
  docs/protocol/apk/2.26.37.71-linked-runnable-writes
```

Follow-up: [metadata observers](linked-metadata-observers.md) identifies
case 26 as Bjk with no record payload, names the last-active platform flag,
and distinguishes 0jV.A03 view lookup from the raw-cache helper. Full downstream
authority remains unknown.
