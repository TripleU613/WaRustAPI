# App-state logical indexes and sync eligibility

Current status: **corrected and tested offline; server-sync scheduling pending**. Static audit and an
offline synthetic Rust reproduction, 2026-11-04. Native evidence is WhatsApp
Android 2.26.43.72, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained evidence and pre-fix reproduction](rea/2.26.43.72-sync-mutation-eligibility/README.md).
No APK execution, live account operation, server request or key rotation occurred.

## Confirmed mismatch

Native stored mutations are indexed by the decrypted mutation-index string,
not by the key-dependent wire index MAC. `0g1.A05` inserts or replaces
`syncd_mutations.mutation_index` and its value MAC. `F1N` constructors retain
that string; `F0k.A02` parses its JSON array of nonempty string components.
Supported mutation objects expose that index through `DCL.A08`; unsupported
records retain the index too.

For incoming verification, the current `SyncResponseHandler.A07` checked DEX
gathers affected logical indexes before calling `0g1.A0I` at byte offsets
`0cde..0ce4`. That query selects previous value MACs by collection and
`mutation_index`, then the handler subtracts those MACs from the LT hash before
adding the new SET MACs. Outgoing `Dxc.run` likewise reads previous MACs by
logical index while preparing a patch. Current `0mj.A00` applies unsupported
SET/REMOVE records and updates held version/hash in the same application flow.

**Confirmed pre-fix Rust source defect:** `CollectionState.values`,
`verify_patch`, `build_patch` and snapshot construction used the wire
index MAC as the stored identity. The index MAC is HMAC under a key derived
from the mutation key. Therefore identical authenticated index bytes under a
different mutation key can have a different wire index MAC.

The retained standalone pre-fix probe demonstrates the consequence:

```text
same logical index, distinct mutation keys: wire index MAC differs=true
first SET rows=1, replacement SET rows=2
cross-key REMOVE result=invalid input for `app_state_collection`: REMOVE has no saved index in this collection
```

Native storage replaces the old row for that same index, so the second SET
must subtract its old value MAC. Keeping both MACs changes the LT hash. A
REMOVE must subtract the stored SET MAC selected by logical index, not require
that the new key reproduce the old wire index MAC. This is a baseline integrity
defect; it is not evidence that a server accepted a malformed patch.

This bounded finding uses identical authenticated index bytes. Full
action-specific canonicalization/normalization of equivalent JSON spellings is
not established here. Never normalize the bytes before verifying their wire
index MAC; existing crypto checks cover the exact original bytes.

## Record-level keys are independent

Current `A07` builds `Ewt` from each record's key ID at `021e..0228`, adds
`EVt.A01` to its key-ID set at `029c..02a6`, separately adds the patch/snapshot
key ID at `0446..045a`, and resolves the combined set through `0fI.A08`
at `045e..0464`. Missing map entries enter retry handling. Record IDs are not
required to equal the patch signing key ID. The already-retained complete
smali envelope is
`ev_bdbbe822dbf9e326660455eea6fe36900bd682985f5b122054c44c507ecb7b73`
in the preceding server-sync lifecycle bundle. This does not remove existing
account/device/fingerprint authorization requirements on held keys.

## Stored and pending row transitions

**Confirmed current schema/writes:** `1tc.ATc` defines stored mutation rows
with index, value, schema version, collection, key device/epoch, value MAC,
dependency flag, action name and optional chat JID. It has no `is_active`
column in this schema. `are_dependencies_missing` is a separate preserved flag.

| Operation | Native transition |
| --- | --- |
| Stored SET | `0g1.A06` requires a key ID/value MAC and calls `A05`: INSERT OR REPLACE by logical index. |
| Stored REMOVE | `A06` calls `A07`: physically DELETE matching logical indexes. No REMOVE tombstone remains in this stored table. |
| Supported/orphan storage | `F3j.A0U` sets dependencies-missing false; `A0V` sets it true; both call `0g1.A0O`, which applies SET/REMOVE. `A0X` also reaches `A0O`. |
| Unsupported SET/REMOVE | `0mj.A00` retains unsupported SET rows with MACs; unsupported REMOVE deletes their logical indexes. Supported UI application is not required for stored-row existence. |
| New pending mutation | `0g1.A0J` requires an enabled action handler (`F3j.A0Y`), inserts/replaces a pending row with `is_ready_to_sync=0`, and returns its row ID. |
| Readiness commit | `0g1.A0R(ids)` transactionally sets `is_ready_to_sync=1` for those row IDs. `A0N` wraps insert+mark-ready; `0aS.A0a` marks selected IDs then forces sync. |
| Pending cancellation | `0g1.A0L`/`A0Q` delete pending IDs; they do not implicitly commit a stored mutation. |
| Accepted outgoing work | `0g1.A0P` applies stored SET/REMOVE and deletes the accepted pending IDs in a transaction. Current `FeN` result worker wraps this together with `0l8.A08` held-version/hash update at `122e..1248`. |

`0aS.A0J` itself refuses insertion when `A0c` is false. Pending readiness
therefore cannot be inferred from merely having an app-state key or a feature
command. Handler `F3j.A0T` marks one known pending row ready. Coexistence job
call sites and the KMP callback are retained but not claimed fully ported.
The pending table schema's default-ready value is 1; the explicit insertion
method above overrides it to 0. Looking only at schema defaults gives the
wrong application transition.

There is no stored-table active filter in `0g1.A0S`: any surviving row counts,
including orphan/unsupported SETs. A positive held version can have no stored
rows after its last REMOVE. An empty collection/key row does not count as a
stored mutation. This refines the [role/recovery mapping](server-sync-role-recovery.md).

## Implemented correction and migration

`CollectionState` retains the wire-indexMAC/valueMAC integrity map and now
holds a separate exact authenticated-index-to-wireMAC mapping. Incoming
verification, outgoing construction and snapshots populate it only from
authenticated/decrypted or locally constructed index bytes. SET removes the old
wire/value entry selected by logical index before adding the new entry; REMOVE
subtracts that stored SET MAC even when the removing record uses a different
key. Duplicate logical indexes across record keys are refused in both patches
and snapshots. Exact wire MAC inputs remain unchanged; no JSON normalization
or provenance guess is performed.

`SavedCollectionState.logical_indexes` is an optional serialized mapping.
New saves carry a complete bijection; older JSON without the field remains
decodable. `from_saved` validates collection identity, MAC widths, unique wire
indexes, unique valid logical indexes, one-to-one references and cardinality.
It recomputes the LT hash from all saved value MACs and requires exact equality.
The saved state's Debug output omits indexes/MACs, which can contain identifiers.
`has_logical_indexes()` exposes a redacted completeness flag for inspection.

A validated nonempty legacy integrity-only row remains inspectable, with its
version/hash/map preserved and completeness false. `build_patch` and
`verify_patch` refuse it with a matching-authenticated-snapshot diagnostic.
There is no silent reset or partial mapping. Empty legacy maps with consistent
zero hash retain their existing version and have no missing index provenance.

`reconstruct_logical_indexes(snapshot,resolver)` is an in-memory, strict
same-version operation. It authenticates every record and snapshot, requires
the saved hash and complete wireMAC/valueMAC map to agree, and returns a
candidate with authenticated indexes. Repeating the operation authenticates and
returns the same complete state. Older/newer snapshot versions are refused by
this migration API. `CollectionSnapshot::key_ids()` exposes the snapshot and
record IDs for preparation of the existing-held-key resolver; those IDs are
not authority to fetch, activate or trust a key.

The storage staging API and transport publisher use distinct checkpoint keys:
`storage.rs` uses `app-state-collection:<hex collection name>`, while transport
`app_state_sync.rs` uses `app-state-collection:<plain collection name>`.
Both saved formats now use the same validator and retain logical provenance.
`storage::stage_snapshot` can persist a verified same-version metadata upgrade
without adding a pending apply batch or replaying actions; its enclosing
transaction governs rollback. Missing authenticated record caches can be
rebuilt from the same verified snapshot without advancing the applied cursor.
This staging path alone does not update a transport publisher checkpoint.

The transport reconstruction API and `app-state-inspect` CLI now integrate
the in-memory helper with the transport checkpoint separately. Use optional
`--reconstruct-snapshot PATH --collection NAME` to supply an already acquired
local snapshot; this performs no fetch. Reconstruction requires an existing
held checkpoint, no pending preparation, the exact held version/hash/wire map,
and only keys already held by the account. Repeat reconstruction authenticates
but makes no changes. The transaction preserves the existing generation,
held integrity state and pending-send reservation boundaries.
Snapshot freshness, sender role and existing key authority remain caller
obligations. This correction does not authorize automatic primary snapshot
requests, unconditional resynchronization, or replay of an uncertain request.

## Verification

All 28 focused collection tests pass. Nine new regressions cover cross-key
incoming and outgoing SET/REMOVE with persisted state; refusal of a validly
signed snapshot MAC that incorrectly retains both old/new rows; duplicate
logical records under different keys; saved-map corruption/bijection failures;
legacy inspection with blocked incremental work; matching snapshot
reconstruction and bad MAC/key/version/wire-map rejection; and atomic,
restart-safe, idempotent storage migration without action replay.
Existing independent encrypted SET/REMOVE and snapshot fixtures still pass.
The pre-fix reproduction remains a historical defect artifact, not current
behavior. Live server acceptance and a complete notification scheduler remain
unverified.

## Eligibility model still pending

After corrected state is committed atomically, a complete authenticated
logical-index/valueMAC map can represent stored-row presence for the supported
port. Legacy maps and partially applied rows must remain distinguishable.
Rust pending baseline/prepared checkpoints and action statuses need an explicit
readiness/flight mapping: `Sent` is uncertain until a correlated result and must
not be reclassified as ready-to-resend. Native readiness precedes encryption;
Rust queues an already prepared IQ, so those records are not interchangeable
just because they both describe pending work.

The next scheduler work still binds generation/role/recovery gates, native
mutation presence and readiness, advertised dirty versions, held-version
fetches, single-flight reservation, verified commit, pagination and completion
rescan. No generic ACK-only handler or eligibility default was added here.
