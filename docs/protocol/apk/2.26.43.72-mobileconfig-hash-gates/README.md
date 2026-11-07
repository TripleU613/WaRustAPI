# MobileConfig hash gates and storage handoff

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static inspection only; no native execution, network request or account mutation.

## Hash-state rejection precedes later processing

The [consumer gate](../2.26.43.72-mobileconfig-consumer-gates/README.md)
continuation calls 0x8d670 at 0x83210. Outlined helper 0xcaf64 supplies caller
output as x0 and storage manager as x1; x2 points to decoded-object ownership,
x3 to the retained request-context pair. The callee reads decoded byte +0x60
and can copy the low 32 bits of decoded word +0x390 through optional x6.

The first predicates use C++ string lengths, not hash equality:

| Predicate | Selected result |
| --- | --- |
| Context string +0x10 nonempty AND decoded byte +0x2a8 bit 0 set AND decoded string +0x290 empty | Log `UnknownQueryHash`; internal result byte 4 |
| First predicate false AND decoded byte +0x288 exactly 1 AND decoded string +0x270 empty AND manager RequestData string +0x160 nonempty | Log `UnknownOneQueryHash`; internal result byte 5 |

Diagnostic fragments `hasQueryHash=`, `queryHash=`, `previousQueryHash=`,
`hasOneQueryHash=`, `oneQueryHash=` and `hasOneQueryHashSupported=` bind these
selected values to the hash-state diagnostics. These labels do not prove all
wire field producers or meanings. In particular the first context string's
producer and complete support semantics need tracing. The RequestData +0x160
string also appears in the previously retained query setup.

Optional output x4 receives the first predicate. Optional x5 receives the
decoded-one-hash-presence/empty-string predicate **without** the RequestData
support check; x5 can therefore be true while processing continues. Presence
checks differ: +0x2a8 tests bit 0, whereas +0x288 compares the full byte to 1.
No claim is made that noncanonical bytes are reachable in normal decoding.
The query-hash error takes precedence when both selected failures apply.

Both selected errors call initializer 0x60484 with result byte 4 or 5, second
byte zero and an empty string. Result 4 sets caller-output byte +0x50 to 1;
result 5 copies decoded byte +0x60 there. Both return through 0x8e3c4 before
the later metadata/storage path. These are internal codes, not HTTP statuses,
Rust enums or server ban reasons. Logging remains a side effect; no global
rollback or retry behavior is established.

## Later state construction and concrete virtual target

After the hash gates, 0x8e590 snapshots the manager's +0x360/+0x368 shared pair
while holding its +0x370 mutex. The caller retains decoded +0x460/+0x468 and
chooses its first pointer when nonnull, otherwise manager RequestData +0x98.
It transfers decoded ownership into a local processing context, copies request
state pairs and records manager flags. At 0x8db60 it calls 0x92fd4. The selected
entry of that large helper inspects context state and calls builder 0x90624;
its full output semantics remain unknown. The surrounding diagnostics include
`generateMetadata` and `response_`; names alone do not prove persistence.

The retained middle path contains string-table/vector construction, optional
fallback virtual getters, 0x907ac and finalizer 0x908c8. A null result from the
later 0x90624 call sets local byte +0xb0 to 1 and jumps to common continuation
0x8e194, rather than taking the builder-population loop. The complete consequence
of that local flag remains unknown. No metadata completeness or application
success is inferred from the helper's return.

At **0x8e2bc**, the caller invokes manager virtual slot **+0xb8** with locally
owned results and a decoded-data pointer. For the previously resolved
FBMobileConfigStorageManager vtable address point **0xd5580**, RELA addend at
**0xd5638** binds that call to **0x8674c**. This is conditional on that same
receiver construction chain; overrides and runtime replacements remain unknown.

The selected 0x8674c entry checks RequestData availability through 0x880f8,
transfers two input pointers, checks availability again, and distinguishes null
inputs (selected internal bytes 6/7) from later work. The first availability
failure produces byte 2. With both pointers present it initializes an internal
result to byte 8, locks manager mutex +0x2d0 and branches on byte +0x2a4.
The flag-1 branch begins at 0x86814; the other branch begins at 0x8706c.
The retained branches inspect filesystem paths and invoke additional helpers.
Helper 0xc2148 calls the imported C++ filesystem `__status` function; this
particular call establishes path inspection, not a write or durable commit.
At 0x87140 the alternate path invokes storage slot +0xb0; RELA at 0xd5630 binds
that slot to **0x860fc**, whose body remains untraced here.

The outer processing routine later performs optional virtual follow-up work,
copies decoded fields into manager state (+0x460, +0x462, +0x468), copies the
returned result through 0x7f970 and restores decoded byte +0x60 to output +0x50.
Those writes are visible in the selected continuation; their admission guards,
durability and cache meaning require the storage result and helper bodies.
They do not establish that the response was persisted or activated successfully.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-hash-gates/inspect-hash-gates.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-hash-gates.json
cmp /tmp/mc-hash-gates.json docs/protocol/apk/2.26.43.72-mobileconfig-hash-gates/hash-gates.json
```

The inspector validates pinned member/ELF identity, strings, selected direct
calls, one indirect instruction and both vtable RELA addends. Synthetic cases
are analyst-derived gate projections, not execution of the target. LLVM retains
the normal processing region and bounded helper/storage windows; exception
coverage and complete helper bodies are not claimed. Nearest JNI_OnLoad labels
are not identities for the local functions.

rea-hash-gates.json retains complete REA Evidence
`ev_2c63ead86866315f90cecdb74e7b8c574fd98e50e74452008ff82ec62b780a37`,
command `e scr.color=0; pd 52 @ 0x8d69c`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. This is a bounded static gate window;
provider version/external inputs are not bound and annotations are not runtime
memory observations.

**Remaining:** decoded hash-field producers and canonical presence bytes;
metadata-generation body; full storage branches and 0x860fc; actual writes,
commit/rollback ordering and durability; cache activation, retries and account
ownership. No Rust runtime or private saved state changed.
