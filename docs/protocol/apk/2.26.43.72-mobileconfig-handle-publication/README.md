# MobileConfig shared-handle publication and status

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, state mutation experiment or account/network operation.

## Publication target and initial gate

The selected StorageManager vtable+**0xe8** at **0xd5668** resolves to
**0x87940**, whose diagnostic identifies
`FBMobileConfigStorageManager::validateAndCreateMmapHandle`. Earlier storage
and upgrade paths invoke this slot with an internal result object. It is a
concrete shared-handle replacement boundary; this report does not equate that
with complete application-level activation or network readiness.

The method first invokes virtual+0xc8 with manager word+0x18. Selected target
**0x87388** derives a path through 0x88048 and uses filesystem status helper
0xc2148. False appends reason byte **5** to the supplied result and returns
before the shared-handle replacement block. This is an existence-like gate,
not validation of bytes. A true path gate proceeds to reader construction if
RequestData availability holds; unavailable data appends **6** and selects a
null candidate, then still reaches the later publication suffix.

## Hash handling selects a candidate; it is not one universal refusal

The available path constructs a reader through the previously traced 0x88144,
obtains its table-derived hash string through 0xa3e58 and compares it to the
RequestData string from 0x5faec. Raw equality retains that reader candidate.
When unequal:

- manager byte+0x120 **not equal to 1** constructs a replacement reader with a
  null root through 0xa3d78, logs `SchemaMismatchError` and appends reason **7**;
- when +0x120 **equals 1**, it strips the **candidate hash only** to the prefix
  before its first colon via 0xa8fa4, then compares that prefix to the **raw**
  RequestData string. Equality logs `shared_extension_schema_match` and retains
  the original reader;
- if that comparison fails and manager byte+0x12a is 1, the selected rebuild
  path calls 0x97d08, 0x8ede0, 0x8f3fc, 0x91198 and wrapper 0x88344 to obtain
  another candidate. Their complete body/schema semantics remain untraced;
- otherwise it retains the original reader candidate despite the unequal hash
  comparisons on this selected path.

This asymmetric prefix comparison differs from the earlier schema helper,
which projected both strings. Do not silently substitute one policy for the
other or infer a wire-format rule from either. Candidate selection and semantic
acceptance remain distinct.

## Probe failure does not bypass shared-pair replacement

At **0x87c04**, a null candidate appends reason **4**; a nonnull candidate
whose virtual+0x10 returns false appends **8**. Both paths then join the same
publication block as a true probe. For the selected reader vtables **0xd61c0**
and **0xd5880**, this slot binds to **0x886ec**, which merely tests whether
**object+0x50 is nonnull** through helper 0xcabbc. It does not rerun the body
verifier, check file durability or prove valid configuration contents.

The method increments manager word+**0x18** before locking manager mutex
**+0x34**. It then calls **0x75fc8 at 0x87c5c** to replace the shared pair at
manager+**0x20** with the candidate pair. The replacement helper retains the
new control block, stores the pair and releases the previous control block.
It accepts a null candidate pair. Thus failure reasons and even null candidates
do not by themselves preserve the old shared handle in this selected suffix.
Other enclosing locks/ordering and exceptions remain outside this claim.

While still holding the mutex, it stores status byte **+0x30**:

| Supplied result byte+1 | Stored handle/probe | Selected status |
| --- | --- | --- |
| 5 | Nonnull, probe true | 2 |
| 5 | Null or probe false | 3 |
| Other | Nonnull, probe true | 6 |
| Other | Null or probe false | 7 |

It unlocks through 0x5e4a8 with flag 1. The numeric statuses are internal;
their full public meanings remain unproven. In particular status 6 reflects a
selected nonnull-root probe, not evidence that all verification/activation gates
passed. The script's examples model only this suffix, not upstream reader
construction or an executed response.

The matching virtual+0x40 getter **0x83c7c** locks the same +0x34 mutex, copies
the +0x20 shared pair with reference retention and status+0x30 into an output
snapshot, then unlocks. This confirms the pair and status are a shared snapshot
surface; it does not establish all downstream consumers' admission rules.

## Follow-up and adjacent state are separate

After unlock, manager+0x440 nonnull, candidate nonnull and probe true can trigger
helper **0xc4338** with manager word+0x408 and string+0x428. The [follow-up trace](followup.md) now identifies keyed event/version
bookkeeping and its virtual dispatch boundary; concrete receiver effects remain
unknown. A call is not proof that another owner activated it.

The separately inspected **0x79e24** setter works on a +0x360 shared pair under
a +0x370 mutex, also read by 0x8e590. Its two bounded caller windows include
construction/wrapper state. It is **not** the +0x20/+0x30 publication above;
do not conflate these shared pairs merely because both use ownership helpers.

Observer helper **0x89e8c** snapshots its manager+0x320 collection under a
+0x338 mutex, unlocks, invokes selected observer virtual+0x18, then submits
captured observer work through the supplied executor slot+0x10. The captured
invoke 0x8a918 calls observer virtual+0x10. Observer identities, consumer effects
and runtime collection lifecycle remain unknown. These retained adjacent reads
do not establish response commit ordering or app-level activation.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-handle-publication/inspect-publication.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-publication.json
cmp /tmp/mc-publication.json docs/protocol/apk/2.26.43.72-mobileconfig-handle-publication/publication.json
```

The inspector pins member/ELF identity, diagnostics, selected vtable targets and
BLs. Suffix cases are analyst projections, not native publication or activation
tests. LLVM retains the method, getter, ownership/probe helpers and bounded
candidate/context/observer windows. Nearest JNI_OnLoad headings are not local
identities; all constructors, observers, imports and exceptions are not modeled.

rea-publication.json retains complete REA Evidence
`ev_66a961dffbe201a68756f9910180435d2b2d7dff9ff81ec7815b391a04faf98e`,
command `e scr.color=0; pd 38 @ 0x87c04`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. It is a bounded replacement/status window;
provider version/external inputs are not bound and annotations are not runtime
memory observations.

**Remaining:** complete candidate/rebuild schema, status meanings and flag
producers, downstream owner/observer admission, runtime timing/exception behavior,
durable active configuration and retry/account ownership. No Rust runtime or
saved account state changed.

The [selected consumer predicate](consumer.md) now traces StorageManager slot
+0x58: RequestData availability, snapshot, nonnull handle and root probe. It
does not read the snapshot status; other consumers remain unresolved.

The [reload consumer](reload.md) now distinguishes hash equality, schema-bit
selection, translation attempt and explicit republication. Snapshot status is
not consumed there; directory selection and final acceptance remain unknown.

The [disk selector](selection.md) chooses the greatest qualifying signed numeric
filename prefix, requiring an existence-like metadata counterpart check. Equal
indexes retain iterator-first choice; body validity is not checked there.

The [translation suffix](translation.md) independently replaces the shared
reader and stores status 4/5 after translator readiness. Its return retains
readiness rather than the later root probe; nested builder semantics remain open.

The [vector mapping trace](mapping.md) confirms sentinel-filled u16 mappings,
bounds-log continuation into vector installation, and packed lookup preserving
upper bits. Nonempty-vector readiness is not complete mapping coverage.

The [raw boolean-data reader](bool-reader.md) now records count/default paths
and pinned raw default byte 0x11. Typed sentinel caller and value decoding remain
unresolved; raw nonzero must not be assumed to mean a Rust boolean.

The [selected boolean decoder](bool-decode.md) now establishes conditional
mask-0x11 gating and bit-7 extraction. Full receiver construction and final API
return semantics remain untraced.

The [metadata getter follow-up](bool-metadata.md) records the exact default
word 0xffffff11, field-8 source and cached count bound, separately from the raw
byte getter. Flag names and final public result remain unresolved.
