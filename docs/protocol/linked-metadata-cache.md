# Linked-device metadata and cache records

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Five-topic recipe](apk/recipes/2.26.37.71-linked-metadata-cache.json),
[selected complete methods](apk/2.26.37.71-linked-metadata-cache/targets.json).
Offsets below are method-relative bytes. This narrows the
[physical writer inventory](linked-cache-writes.md); it changes no runtime or
private account state.

## Place and instrumentation metadata are not login transitions

**Confirmed:** 0jV.A05(Location,0jV,HxZ) constructs Android Geocoder with the
configured locale and requests one address from the supplied location. It selects
a nonempty locality. Its retained IllegalArgumentException/IOException handler
covers 0044..0096; empty locality returns without the SQL write or observer
scheduling. The nonempty path selects HxZ.A0A, expands its raw own aliases via
1IY.A06, and uses A04's placeholder selector. It UPDATEs devices.place_name
at 00f4..00fa without examining affected-row count. Under the monitor it looks
up a cached record via 1IY.A02, nulls only the view map A02 (0106..010a), and,
when a record exists, mutates HxZ.A06 to the locality (010e..0112). It closes
the store and schedules D3f discriminator 26 through 078.A00.

**Unknown:** supplied location/caller authority and complete observer effects.
This path does not establish that location is mandatory for registration,
pairing, ordinary messaging or identity verification. No location was generated
or transmitted. A field-store inventory misses in-place changes to mutable
records held by an immutable map.

**Confirmed:** 0jV.A0W(DeviceJid,String) similarly expands raw own aliases and
UPDATEs devices.instrumentation_device_id (0076..007c), ignoring the SQL result.
It looks up the existing raw-map record and nulls only A02 (0088..008c).
A nonnull record receives the supplied string in HxZ.A04 (0090..0094).
No observer invocation is visible in this method. It does not visibly generate
or validate that string. **Unknown:** complete producer/caller authority; the
field is not evidence of a cryptographic identity or registration ID.

## One cached record versus multiple selected SQL rows

**Confirmed:** 1IY.A02 reads raw map A01; absent map returns null without loading
SQL. It first tries the supplied DeviceJid, then parses each supplied raw string
through 0DK.A04 in array order, skipping null parses and returning the first
map hit. All misses return null. Both metadata writers therefore mutate only
one returned HxZ, although their SQL selector may include multiple saved own
aliases. This is not proof that every aliased cached record receives the new
metadata. Complete equality/parser semantics remain **unknown**.

**Confirmed:** 1IY.A04 builds `device_id IN (` plus one `?` per array entry,
joined by `, `, followed by `)`. Values are not interpolated into SQL. Empty
input produces `device_id IN ()`; wrapper handling and all callers' validation
remain **unknown**, not an inferred all-row fallback. Alias authority is limited
to the stored own PN/LID predicate described in [cache writes](linked-cache-writes.md).

The already retained [peer history staging](apk/2.26.37.71-history-queue/peer-history-staging.txt)
contains 1gz.A00's companion_meta_nonce SQL update at 2406..240c and view-only
cache reset at 2418..241c. Full in-memory nonce semantics remain **unknown**;
this observation is not a new nonce-authentication guarantee.

## Broader reset context, not an ordinary migration

**Confirmed / bounded:** GEu.A03 calls multiple account/store helpers. Its
selected 1IY segment calls 1IY.A03 then 0aC.A0D (0106..0114), nulls raw A01
and view A02 under the monitor (0116..011e), and later removes preference keys
with prefix `logged_out_companion_display_name/` before Editor.apply. Other
retained diagnostics include emoji/language/media-job database deletion paths.
The cache reset sits in a broader store-reset context; it is not evidence of
routine metadata, receive or identity-change handling.

**Unknown:** trigger/caller authority, complete cascade and the destructive
wrapper guarantees. No reset was run, and this trace does not authorize copying
reset behavior into the client. DLo.run, DM7.run and DNh.run physical cache
writes still need semantic tracing. Complete ingestion, rollback, trusted
linked-device lifecycle and read-self fanout remain unfinished.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-linked-metadata-cache.json \
  --output docs/protocol/apk/2.26.37.71-linked-metadata-cache --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-linked-metadata-cache \
  docs/protocol/apk/2.26.37.71-linked-metadata-cache
```
