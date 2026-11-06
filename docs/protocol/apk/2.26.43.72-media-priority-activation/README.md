# Priority service registration/activation and bounded property search

**Confirmed static**, APK2.26.43.72 SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 libwhatsappmerged.so SHA-256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
No target execution, runtime account activity or configuration fetch.

## Named JNI binding now closed

The previous [header producer bundle](../2.26.43.72-media-priority-header/README.md)
identified an initHybrid tuple by prototype only. Complete registration routine
0x4d6234..0x4d62fc copies240 bytes (10 triples) from0x74d260 into a local table,
constructs the67-byte class slice0xf5d26:
`com/crossapp/tigonhttp/TigonHttpClient$TigonHttpClientServiceHolder`, finds that
class and registers10 methods through0x39fbe0. The first triple is the exact
12-object initHybrid prototype ->0x4d6344. All10 relocated triples are retained
in native-bindings.json. This closes static class-to-table identity, not observed
library loading or runtime RegisterNatives success. The shared JNI helper's
RegisterNatives ABI slot was independently checked in earlier registration work.

## Normal stack construction invokes the priority stage

The full client-stack-composition routine in the previous bundle builds16
std::function slots, each0x30 bytes, starting at local+0x700. The priority closure
is slot13 at+0x970 (vtable0x76c970 ->invoke0x4f4aa8). Slots14 and15 use
vtable0x76c9d0 ->0x4f39a8 and0x76ca30 ->0x4f385c, respectively.
At0x4f282c..0x4f2834, x1 is the array; helper0x72fa88 sets x0 to the stack
object and w2=16, then calls0x65f998.

Complete walk0x65f998..0x65fbf4 computes end=begin+count*0x30 and iterates
backwards: each normal-path step loads the preceding function pointer at
end-0x10, invokes virtual slot0x30 through helper0x72aecc, processes ServiceInfo,
subtracts0x30 and repeats. There is no configuration skip predicate before the
priority closure. A missing function throws; exceptions/allocation errors can
stop construction. This is static normal-path activation, not runtime execution.

For a result with nonzero pointer or service ID, the walk atomically replaces
the stack's current shared service pair at stack+8/+0x10; zero-ID/null results
preserve the current service. It also records nonzero service IDs. The priority
closure's previously open condition reads that current pair through0x4adf44:
it obtains the shared-pointer mutex, copies both words, increments ownership,
and unlocks. This is an atomic shared-pointer load, NOT a weak_ptr lock or
an enable flag. The priority closure constructs its service when the current
service pointer exists; absence yields empty ServiceInfo. Slot15 supplies a base
service through0x4d8768; slot14 can add another service before slot13 executes.
The base result helper and preceding stage are retained as explicitly partial
prefixes, not complete base/backend policy traces. The root service's effective
presence is not asserted from those prefixes.

Thus the graph always invokes the priority stage during normal construction,
and installs it when the previously built chain is non-null. The old uncertainty
about a separate unknown priority-stage selection flag is resolved. This does
not prove every app client uses this constructor, backend network acceptance,
or that the actual process registered/ran these methods.

## Optional r property remains unknown

Complete1FJ.<clinit> declares numeric keys5,7,12,20,21,30,32,33,42,45,52,53,58,59,65.
1FK.<init> directly stores the integer, without a remapping. The JAT setters in
the previous full execute export use members of this class; none is numeric3.
This excludes those explicit Java constants as a direct source of key3. It is
NOT an absence proof for the full native property container or middleware;
namespace/value serialization, inherited properties and native writers remain
separate. Do not infer r=0 or omit it solely from this census.

A reproducible bounded native search retains nine direct BL call sites to the
typed integer getter0x3ef220. Only the priority service's directly preceding
immediate selects3; other visible immediate keys are27,36,33,34,4, with one
caller accepting its key argument dynamically. A register assignment can occur
outside the displayed window, so these snippets are discovery, not dataflow proof.

A nearby-immediate heuristic for w1=3 followed by the first BL within8 instructions
finds105 sites across58 targets. A narrower w2=3 scan within0x4f0000..0x530000
finds four contexts, none proven a property3 producer. No direct setPropertyGroup
call with w1=3 in the preceding12 instruction lines was found. Outlined helpers,
dynamic keys, value construction, indirect calls and other properties make this
an incomplete search. The setPropertyGroup prefix was inspected only to anchor
that operation's shape, not to identify r or a missing write. No retry-count
meaning is assigned from the isolated retry_count literal or nearby retry code.

**Still open before a Rust Priority policy:** identify the typed key3 producer
and its first-upload value/presence, and close native serialized category binding
for the standard MOu/JAT path. Previous evidence proves the calculator prefix
conditionally; it does not justify inventing r, dropping inherited context or
adding a header to all other transport paths. No Rust/header/retry/deadline/
persistence behavior was changed.

## Exact reproduction

From repository root, existing xtask and LLVM23.1.3:

```sh
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1FJ;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1FK;' --method '<init>' --signatures --exceptions
python docs/protocol/apk/2.26.43.72-media-priority-activation/inspect-native.py ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
llvm-objdump -d --no-show-raw-insn ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so > /tmp/wa-priority-activation-all.txt
python docs/protocol/apk/2.26.43.72-media-priority-activation/scan-disassembly.py /tmp/wa-priority-activation-all.txt
```

Outputs correspond to java-property-keys.txt, java-key-constructor.txt,
native-bindings.json and bounded-search.json. The native verifier reuses the
sibling media-body-provider digest-pinned ELF section/relocation parser. For each
[start,end) in native-windows.json reproduce the named .txt with:

```sh
llvm-objdump -d --no-show-raw-insn --start-address=START --stop-address=END ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

Numeric addresses and checked relocation/literal data identify routines; nearest
exported headings are unrelated. Prefix files are explicitly incomplete windows.
Previous REA category Evidence remains in the header bundle; no new native REA
record is fabricated where Ghidra/Hopper remain unavailable. Native findings
use pinned ELF/LLVM. Checksums cover every file here except themselves. No bulk
APK, full disassembly or native library is committed.
