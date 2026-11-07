# MobileConfig schema table grouping and index indirection

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static reads only; synthetic examples do not execute target code or use accounts.

## Table getter has two distinct initialization routes

The preceding [selection trace](../2.26.43.72-mobileconfig-schema-selection/README.md)
obtains its table through 0xaa594. The predicate 0x5fb64 selects between two paths
using fields of configuration+0x10. On its true branch, a state lookup through
0xba1dc and call_once initialization (thunk 0xc9e90 to 0xba1d4/0xba0d4) precedes
returning a pointer from configuration+0x98. That factory and its cache identity
are not fully traced; this report does not equate them to the deferred path.

On the false branch, 0xaa3dc reads deferred byte configuration+0x258 -> +0x41.
When nonzero it invokes 0xaa438, which uses the earlier byte-CAS 1-to-2 refresh
pattern. The owner of that refresh obtains a shared source pair through 0xa99c0.
If its object pointer is nonnull, 0x760f4 supplies a 32-bit count and 0xc1534 builds
the table. Otherwise 0xaa544 allocates an empty 128-byte table object. The selected
publication transfers the built pointer into configuration+0x258's +0x10 field
through 0x9c80c, then releases temporary state and calls the refresh-release helper.
Complete synchronization and account-generation ownership remain unproven.

0xa99c0 itself has alternative lookup and deferred in-memory branches. Therefore
this trace identifies an **existing record vector** as 0xc1534's input, not a
JSON/parser entry point or proof of a file source. The selected slot-count getter
and source producer remain incomplete. Never substitute a guessed count or schema.

## Grouping uses adjacency and a separate word

0xc1534 allocates the empty table first. A zero count or an empty source vector
returns that empty object without resizing range/index vectors. Otherwise it
initializes count entries in a 0x18-byte range table and a 32-bit index map.
The range resize helper initializes added entries with null pointers and low flag
bit 1. The index-map resize helper initializes added words to zero.

It walks 0x50-byte source records in their existing order. Whenever record word
+0x34 changes, it finalizes the preceding contiguous run into the next sequential
table entry; it finalizes the last run after the loop. Each actual run stores
begin/end pointers into the original source vector, and its low flag bit becomes
zero. Remaining preallocated entries retain the empty/flagged representation.
It does not sort or copy the records into new groups here.

**The grouping word +0x34 is distinct from +0x4c**, which the previous fragment
and checksum trace uses for numeric wire tokens, filtering and arithmetic. For
each finalized run the builder stores sequential table index into the 32-bit map
at index group_word34. It also calls label-map insertion 0xc16fc using the first
record and the range entry, and 0xb9e90 with first-record word +0x4c and group_word34.
Those map insertion/duplicate policies are not completely modeled here.

The direct 32-bit map write is explicit: a nonadjacent repeated group_word34 can
produce separate runs and overwrite that direct map entry with the later table
index. Whether real record producers permit such input is unknown. The source
must satisfy grouping and bounds invariants; out-of-range table or group-ID-map
indices reach BRK at 0xc16e0. Initial -1 grouping sentinel behavior and malformed
memory/ranges are outside the synthetic model's valid domain.

A zero default in the direct map is not enough to classify an ID as missing:
index zero is also a valid first table entry. Do not infer a missing-ID sentinel
or choose an empty fallback from that word alone.

## Consequence for query ordering

The selected index producer visits table indices in ascending order, but those
table positions represent source **run encounter order**, not a sort by group ID
or wire token. Thus the ordered fragment/checksum pipeline must preserve the
source table's order. The synthetic sparse example has grouping IDs 2 then 0,
but table positions 0 then 1 and distinct wire words 101 then 202. Reordering by
either ID would alter the earlier derived checksum and array order.

These table entries retain pointers to source storage. The shared source is held
during construction; long-term retention and replacement synchronization must be
traced separately before reproducing this with borrowed pointers or cached views.
No raw-pointer design is recommended for the Rust implementation from this alone.

## Reproduction and evidence limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-schema-grouping/derive-grouping.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-grouping.json
cmp /tmp/mc-grouping.json docs/protocol/apk/2.26.43.72-mobileconfig-schema-grouping/grouping.json
```

The script reuses pinned ELF/member validation and selected BL checks, then emits
an analyst-derived synthetic projection of contiguous ranges and the direct index
map. It covers sparse IDs, nonadjacent repeats, empty records and zero count.
It models valid nonnegative in-bounds group IDs only, omits the other map helpers,
and does not execute the APK or extract actual schemas. Bounds refusals are
Python model errors, not an emulation of native BRK or an approved recovery API.

LLVM windows retain complete selected grouping logic, range/index resize helpers,
getter/refresh paths and bounded source/count/alternative-factory/map-insertion
entries. Those bounded downstream reads are unfinished boundaries. Outlined
helpers retain register setup; nearest exported JNI_OnLoad headings are not local
function identities. The earlier checksum combiner remains in the preceding
query-setup evidence; no new checksum identity or runtime configuration is claimed.

rea-grouping-window.json retains complete REA Evidence
`ev_cf32bb7dd173fed07f599f95053989f1ee9e543b2714edd015f51695e4f5cf78`:
`e scr.color=0; pd 65 @ 0xc1534`, exit 0, untruncated output, unchanged artifact
hash, with an entrypoint warning. It is a bounded window inside grouping; LLVM
retains the remaining selected function. Register-relative Rizin annotations are
not observed runtime memory.

**Remaining:** original record-vector population, count/field producers, alternate
factory, map collision rules, source retention, refresh concurrency, request-data
construction and response parsing/file commit/cache activation. No Rust runtime
or live account behavior changes are justified by this static grouping alone.
