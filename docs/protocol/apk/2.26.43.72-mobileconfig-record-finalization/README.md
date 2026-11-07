# MobileConfig record finalization and conditional ordering

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static reads only; synthetic examples do not execute target code or use accounts.

## Finalization is guarded and has two ordering paths

The preceding [text record producer](../2.26.43.72-mobileconfig-text-records/README.md)
stores temporary records in keyed maps. Function 0xbfe18 proceeds only when loader
byte +0x70 equals 1 and byte +0xe1's low bit is clear. It branches on loader
child byte +0x98. Semantic names/producer invariants for these flags and complete
caller admission remain unresolved. The guard must not be replaced with a claim
that parsing or receipt of a callback guarantees finalized records.

When child byte +0x98 is not 1, it clears the record vector at loader+0x150 using
0x5f9e8, walks a group map via 0x6f44c, resolves each group's record range through
0x5e228, and appends its records using 0xbd714. That append moves the two strings
and scalar fields from the source record on its selected capacity path. The
source map records therefore cannot be assumed reusable unchanged afterward.
The full reallocation helper is retained; allocator/exception behavior is not
modeled. Iterator-defined group order is confirmed, but its comparator has not
been traced: no lexical, numeric or insertion-order claim is made for that map.

When child byte +0x98 equals 1, a nonempty pending-record map enables a new shared
vector at loader+0x150. It visits group ordinals from zero up to loader+0x17c,
looks up pending records and existing child ranges, and interleaves them by signed
record word +0x30. At 0xbff18–0xbff1c it compares the existing word with the pending
word and takes the pending record only when existing > pending. Otherwise it
appends the existing record and advances that source. Remaining records append
from whichever stream is left. The helper 0xbfbc0 copies both strings through
0x80c74 and copies scalar fields; this branch's ownership behavior differs from
the move-shaped append above.

**Equal words are not deduplicated by this comparison.** The existing record is
chosen first and the pending record remains for a later iteration. Upstream map
handling can constrain conflicts; equality in this merge is not evidence of
replacement, deletion or one global precedence rule. Neither source's sorting
invariants are proven here, so this comparison alone cannot establish that every
final vector is globally sorted.

## Shared handoff connects to the earlier record accessor

On the selected merge path's completion, it writes child count fields and copies
the new shared vector pair into child+0x50 through 0x75fc8, then marks loader
byte +0xe1 as 1. The local append path also reaches that completion marker.
An empty pending map can reach the marker while retaining existing child records.
The earlier [accessor](../2.26.43.72-mobileconfig-record-source/README.md) returns
child+0x50 when child byte +0x98 is 1, otherwise loader+0x150. This closes those
selected final-vector destinations without proving every runtime selects them.

The group-range helper 0xc66f0 additionally builds ranges over an existing
0x50-byte vector grouped by changes in word +0x4c, with another lookup structure.
Its source/order invariants and complete map behavior remain untraced. It is
separate from the later [schema query table](../2.26.43.72-mobileconfig-schema-grouping/README.md)
that groups by word +0x34. A shared record layout does not make every grouping key
or table interchangeable.

**Ordering consequence:** the earlier ascending index selector refers to the
resulting table, not necessarily original text-file order. Record finalization
can use group-map traversal or an existing/new merge before the query table is
built. Preserve each stage's order rather than sorting by a guessed universal ID.
Shared handoff and completion markers do not prove account ownership, race safety,
crash recovery or persisted configuration adoption.

## Reproduce and remaining limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-record-finalization/inspect-finalizer.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-finalizer.json
cmp /tmp/mc-finalizer.json docs/protocol/apk/2.26.43.72-mobileconfig-record-finalization/finalizer.json
```

The script reuses pinned ELF validation, checks selected BLs and the signed-GT
branch, and emits analyst-derived scalar merge examples. These cover interleaving,
equal words, signed ordering and an empty stream; they do not emulate maps,
record ownership, native exceptions or actual schema values.
LLVM retains the complete selected finalizer, append/copy/move/clear helpers,
range-builder region and bounded transition/factory/discovery windows. Bounded
reads of alternate text/file-source neighborhoods are preserved without claiming
their full grammar or semantics. Nearest JNI_OnLoad headings do not identify locals.

rea-finalizer-window.json retains complete REA Evidence
`ev_dc6208fc8abf1eb864ad35e32c7ad3710808055a5f30bf4339b6a021cccec838`:
`e scr.color=0; pd 62 @ 0xbfe18`, exit 0, untruncated output, unchanged member hash,
with an entrypoint warning. It is a bounded entry window; LLVM retains the wider
selected function. Register-relative annotations are not runtime memory.

**Remaining:** map comparators and actual input invariants, complete move/copy
exception paths, finalizer caller/flag producers, selected runtime file/decoder,
account ownership, synchronization and response persistence/cache activation.
No Rust runtime or saved account state changed.
