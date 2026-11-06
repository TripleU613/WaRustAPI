# Native upload length-handler candidates — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 libwhatsappmerged.so size7871544, SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`,
reused from the [verified reconstructed members](../2.26.43.72-superpack-reconstructed-members/README.md).
No library was loaded/executed; no account/network operation. This follows the
[JNI length-forwarding boundary](../2.26.43.72-media-body-provider/README.md).
Host ELF/LLVM evidence, not new REA decompilation or runtime dispatch proof.

## Candidate resolution and exclusions

resolve-vtables.py reuses the prior digest-pinned ELF resolver. It locates exact
RTTI names, follows relative relocations from name to typeinfo, then candidate
primary vtables referencing that typeinfo. Requires zero offset-to-top and seven
relocated pointers into executable sections before accepting each candidate.
Those checks reject data/inheritance typeinfo references that a raw pointer
search would otherwise mislabel as vtables. It asserts the three selected
name/slot bindings and hashes assembly windows. This is not a complete census
of all subclasses, secondary tables, thunks or possible dynamic objects.

| RTTI class | Primary address point | Slot0x30 entry |
|---|---|---|
| TigonLimitingBodyStream | 0x779370 | 0x4b1e3c |
| mns::TigonMNSBodyStream | 0x76ebc0 | 0x517bc8 |
| TigonHucBodyStream | 0x778550 | 0x64a364 |

The preceding JNI wrapper sign-extends the Java-reported length and calls
slot0x30. These tables therefore identify conditional implementations for that
operation, NOT which one receives a particular JAT upload.

## Limiting wrapper forwards length unchanged

At4b1e3c..4b1e50 the method loads its wrapped pointer fromthis+0x18,
loads that object's vtable slot0x30 and tail branches without changing x1.
No comparison, truncation or rewriting of the supplied length appears in this
complete tiny forwarding entry. Neighbor slots0x20/0x28 likewise forward to
the same wrapped pointer. The name Limiting is not evidence this method limits
body length; its buffer/flow-control behavior elsewhere remains separate.

The partial construction window64e908..64ea50 shows a conditional path testing
[x19+0x70], allocating a shared control/object block and installing address
point779370 atx21+0x18 (64e984..64e994). This is evidence of a construction site,
not a complete beginStream method or binding of its wrapped pointer to a given
transport. The surrounding routine, gate producer and dynamic object remain
unresolved. Do not promote this snippet into proof that every upload uses it.

## MNS candidate records the supplied value

Entry517bc8 calls outlined helper727afc, which retains x1 inx19. The next
frame helper72a0dc does not change x19. After resolving an owner, entry checks
x21 nonnull, locks mutex atx21+0x290, stores x19 atx21+0x320 (517c08), then
unlocks. Thus on this selected normal path the value is recorded under a mutex;
there is no compare against actual written bytes in this method. A missing owner
skips that store. Full owner acquisition, field consumers, wire Content-Length
and actual transport admission are not established here. A stored size is not
server acceptance and says nothing about whether a later consumer enforces it.

HUC candidate64a364 uses a guarded cached target and downstream call; its entire
selected entry window is retained, but target initialization/dispatch semantics
are not resolved here. No HUC framing conclusion is made.

**Negative/search limits:** selected dynamic symbols expose only two matching
provider symbols; absence of named stream methods is not absence of code. Raw
RTTI-pointer matches included non-vtable data and were filtered as above. LLVM
nearest-symbol headings name unrelated preceding exports; use numeric entries,
not those headings, for attribution. Current REA native provider unavailability
is already retained in the preceding bundle; it was not repeatedly invoked.

**Unknown:** concrete object behind JNI dispatch; remaining stream subclasses;
limiting-wrapper gate/owner; MNS+0x320 consumers; HUC cached target; effective
fixed-length configuration and final wire framing. No Rust behavior change is
justified by these conditional candidates. Existing uncertain uploads remain
unmodified and unreplayed. Next trace MNS stored-length consumers or bind the
actual stream object before selecting a transport-specific construction change.

## Reproduction

From repository root, using the existing reconstructed pinned library:

```sh
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python3 docs/protocol/apk/2.26.43.72-media-native-length-handlers/resolve-vtables.py "$LIB" > vtables.json
readelf -Ws "$LIB" | c++filt | rg 'Tigon.*(BodyStream|BodyProvider)' > native-symbols.txt
strings -t x "$LIB" | rg 'Tigon.*BodyStream|reportBodyLength|body length|bodyLength|content length|ContentLength' > native-strings.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x4b1e14 --stop-address=0x4b1e50 "$LIB" > limiting.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x517bc8 --stop-address=0x517c48 "$LIB" > mns.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64a364 --stop-address=0x64a448 "$LIB" > huc.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x727afc --stop-address=0x727b10 "$LIB" > mns-arguments.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72a0dc --stop-address=0x72a0e8 "$LIB" > mns-frame-helper.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64e908 --stop-address=0x64ea50 "$LIB" > wrapper-construction.txt
```

Uses existing LLVM23.1.3, readelf and Python stdlib. Script assumes repository
root for the shared resolver path; no new dependency. checksums.json covers
every other file with bytes and SHA256. Full disassembly remains private/local.
