# Fetch app state at held versions

**Confirmed static:** WhatsApp Android 2.26.43.72, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete current REA records and checked DEX exports are in
[the evidence directory](rea/2.26.43.72-app-state-versioned-fetch/README.md).
No live request or APK execution occurred.

## Current producer and wire construction

The [server-sync lifecycle trace](server-sync-lifecycle.md) establishes that
`Dxc.run` discriminator 4 prepares an `EdY` with an empty mutation list for a
dirty collection without outgoing changes. **Confirmed, `EdY.A00`:** when
neither inline mutations nor external mutations are present, it returns an
`EXV` with a null patch. Its constructor stores collection name/list/hash;
it does not invent a patch, version or key material for a read.

`FJU.accept` discriminator 12 obtains each collection's **held** version from
`0l8.A02`, then passes it to `EfK.A01`. The held version differs from a
`server_sync` notification's `dirty_version` (see the preceding lifecycle note).
The boolean comes from `09F.BU7`, not from the advertised version. The new
Rust primitive accepts that snapshot request policy explicitly; callers must
establish it from their actual role/lifecycle.

**Confirmed producer uniqueness:** classic `0aS.A0Q` builds a map keyed by
collection. `Dxc.run` discriminator 4 iterates its entries and prepares one
result/future per key; both the empty `EdY` branch and outgoing-mutation branch
carry that key as the result's collection name. `FJU.accept` discriminator 12
emits one `EfK.A01` call per prepared result. `EfK` itself appends children:
its collection-name set is result bookkeeping, not wire deduplication. The Rust
slice boundary rejects duplicates (matching or conflicting versions) to preserve
the traced producer's one-entry-per-collection contract. This is not a claim
that all native callers or the low-level `EfK` helper reject duplicates.

**Confirmed, `EfK.A01`:**

| Held version | Snapshot requested | Collection attributes |
| --- | --- | --- |
| absent | false | No `version`, no `return_snapshot` |
| absent | true | No `version`, `return_snapshot="true"` |
| zero | false | `version="0"`, no `return_snapshot` |
| zero | true | `version="0"`, `return_snapshot="true"` |
| positive | either | Unchanged decimal `version`, no `return_snapshot` |

Each child carries its collection name and `order=0` only for
`critical_unblock_low`; all other supported collections use `order=1`.
`patch` is added only for a non-null patch argument. A read carries no patch,
including when its held version is positive. `B1y.A1I` invokes
`0Tl(String,long)`, whose body delegates to `String.valueOf(long)`; there is
no floating-point conversion or version increment.

**Confirmed, `EfK.<init>` checked DEX and `EfK.A00`:** the root is
`iq`, addressed to the server, with `type=set`, `xmlns=w:sync:app:state`,
the supplied request ID, and a `sync data_namespace=3` child. `B22.A0K`
and `DBi.A0h` establish the recipient/type/namespace helper chain.
The optional bot/coexistence branch is separate and remains unsupported by
this primitive. Java constructor recovery hides field initializers; the retained
DEX confirms the `sync` builder allocation and initialized maps/set.

## Rust behavior

`transport::app_state_sync::fetch_request_at_versions(id, collections,
return_snapshot)` now constructs that patch-free request. Each entry is
`(Collection, Option<u64>)`. Example:

```rust,ignore
fetch_request_at_versions(
    "F1",
    &[
        (Collection::Regular, Some(7)),
        (Collection::CriticalUnblockLow, None),
    ],
    true,
)?;
```

The first collection asks for updates after held version 7. The second omits
`version` and asks for a snapshot. This function takes no store, sends nothing,
reserves no IQ, applies no state and ACKs no notification.

Validation preserves the existing request-ID contract (nonempty, at most
128 bytes) and rejects an empty collection list. Typed collection names reuse
the established five-name enum. Held versions above `i64::MAX` are rejected
because the traced producer/attribute constructor uses Java signed `Long`;
the unsigned Rust type excludes negative versions. This is a representable
range check, not an inferred server limit. There is no arbitrary lower cap,
IEEE-754 restriction or sorting. Duplicate collection names are rejected with
a diagnostic naming the collection rather than constructing ambiguous entries.

The existing `fetch_request` delegates with explicit `Some(0)` and snapshot
requested for every collection, preserving its prior wire behavior.
It now also rejects duplicate collection names through the shared validation.
`queue_fetch` still uses that existing full-fetch policy. No automatic
notification scheduling or receive-loop behavior changed. This primitive is
a library construction capability, not a new CLI command or HTTP endpoint.

## Sync hook and remaining obligations

**Confirmed, `0gG.A00`:** resolve `00X.A01()` then call
`09H.A03(provider,1251)` and discard the result. The body contains no dirty
version write, scheduled task cancellation, IQ creation, ACK, timer, or flight
flag mutation. **Unknown:** transitive dependency initialization side effects.
There is no evidence to reproduce this hook as fabricated telemetry or a
network request in Rust.

The [remaining state lifecycle](server-sync-lifecycle.md) still requires actual
role/recovery/bootstrap mapping; distinct durable dirty metadata; blocked-key
and pending-mutation eligibility; one-flight scheduling with a held-version
read; verified response commit and has-more continuation; completion rescan;
and explicit failure/backoff/uncertain-outcome handling. `EfK`/`EdY` wire
construction is now traced and the version-aware read primitive is available.
The `0gG` body is traced to the bounded provider lookup above. Neither closes
the remaining lifecycle obligations or permits an ACK-only shortcut.

## Validation

The focused request-builder suite passes all 12 tests, including four new
regressions: absent/zero/positive versions crossed with both snapshot policies;
decimal precision above 2^53 and at `i64::MAX` through the existing wire codec;
invalid IDs/empty targets/signed-range overflow; and duplicate collections with
matching, differing or absent versions, also through the legacy wrapper.
Existing full-fetch and
patch fixtures still pass. Static tests establish construction only, not live
server acceptance or scheduler completeness.
