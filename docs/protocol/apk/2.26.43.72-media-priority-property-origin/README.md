# Priority r-property origin: expanded bounded census

**Confirmed static**, APK2.26.43.72 SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, live account, traffic capture or remote configuration.
This expands the previous [priority activation search](../2.26.43.72-media-priority-activation/README.md);
it does not claim discovery of property3's first producer or absence on uploads.

## Lookup and mutation boundary

The priority header's typed getter calls0x668250, which first finds the root
entry through0x667fb0, then requires entry tag+0x40==1 before exposing payload+0x20.
The already retained0x3ef220 getter further requires payload tag+0x18==1 and
returns its first word plus engagement bit. Thus the formatting condition tests
a particular typed root property, not any matching integer nested in a group.
Complete root setter0x667d94..0x667fb0 and Group.setProperty alias0x667d8c are
retained. The alias branches to0x667ba8; a scan of only direct calls to that
helper would miss calls to the alias. Native lookup/setter algorithm coverage
is limited to the retained routines, not all possible mutation paths.

Direct main-library census finds25 B/BL sites to root setProperty,37 to its
Group alias,2 to underlying group helper0x667ba8 and3 to0x507c9c. The snippets
retain numeric keys or outlined helper calls. No literal3 root setter is
identified in those displayed assignments; do not turn that into a complete
value-flow or inlining absence proof. One root write at0x6540b8 uses a dynamic
key: helper0x72a664 forwards x23 as key and x20 as destination. The retained
0x654000..0x654120 prefix is explicitly partial and does not establish x23's
origin. The small-immediate candidate0x4fe320 calls a different helper and then
copies a byte; it is not evidence of a direct property3 setter. No semantic
retry-count label is assigned to r.

## All physical Java key allocations in the pinned DEX

The checked constructor census has24 physical sites across7 classes. The
previous bundle already retains1FJ's15 constants; all other matching methods
are retained here. Fixed sites in1FN,1gM,JY4,JY5,JZ6 use166,167,168,216,172,214,
261,263. Together these are23 constant sites; none constructs3. The remaining
site is1GV.A05, which decodes an integer from1GT and constructs1FK dynamically,
then sets a boolean, long, string, map or property group according to the
serialized type. Therefore expanding constant searches cannot exclude key3.
The constructor itself was previously checked to store its integer unchanged.
Reflection, optimized allocations and calls outside the physical census remain
outside this static constructor-search claim.

The property decoder has two physical callers:1GS.A01 (returns a TigonRequest)
and1GW.A00 (returns a Tigon summary). Complete bodies show request fields/header
map and arbitrary properties reconstructed from bytes, and summary properties
decoded after request/response members. These routines carry decoded keys;
they do not label property3 or establish its initial source.

Further censuses show5 physical summary-decoder calls in callbacks/observers,
and6 request-decoder calls including onStarted, observer initialization,
summary decoding, WAHucClient.startRequest and HucClient.startRequest. These
names and direct edges do not prove live runtime use. In particular, it would
be incorrect to call this a response-only decoder or treat its existence as a
new outbound property producer; HUC request bridging also reconstructs requests.
Full upstream bridge bodies and property3's first source remain untraced.

## Cross-library boundary

All83 reconstructed ARM64 members were checked byte-for-byte against the pinned
[reconstruction validation manifest](../2.26.43.72-superpack-reconstructed-members/README.md).
source-checks.json retains every member digest/size and the scoped symbol-string
scan. Only libwhatsappmerged.so, libffsingletonmerged.so and libmobileconfig-jni.so
contain either root-setter or Group-setter mangled-name substring. This search
does not cover stripped/inlined implementations, alternative symbol spellings,
other ABIs, Android system libraries or runtime-loaded code outside these83.

Dynamic symbols confirm the latter two import the setters. Their direct calls:

- libffsingletonmerged.so:3 root setters (keys44,20,20),4 Group setters
  (269,272,270,271),2 setPropertyGroup calls (root key65).
- libmobileconfig-jni.so:1 root setter (key46), no direct Group setter,
  1 setPropertyGroup call (root key48).

The root-setter windows are retained; Group/group-root contexts are in the
reproducible setter census. Some nearby3 arguments count records rather than
being the setter's key: the actual root key is65. No claim that these unrelated
requests belong to the selected media upload is made. Initial libar-bundle3.so
symbol inspection found no dynamic setter matches; the broader83-member scan
supersedes that single-library probe. All source digests are independently
anchored in the existing reconstruction manifest:

- main: d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d
- ffsingleton:8d758ab4b1b7d1a5819bcb4d7665e9c955ca1e276fabd9acce03aa810c14255c
- mobileconfig:5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c

**Remaining unknown:** original writer/value/engagement of typed root property3
on the selected first upload; full native category conversion binding. This
research eliminates misleading absence arguments, not the need for those
proofs. No Rust Priority/header/retry/timing/persistence policy changed. The
positive downstream header producer and normal stack activation remain valid.

## Exact reproduction

From repository root, existing xtask and LLVM23.1.3. For each entry in
dex-selectors.json, [class,method]:

```python
import json,subprocess
from pathlib import Path
base=Path('docs/protocol/apk/2.26.43.72-media-priority-property-origin')
for name,(cls,method) in json.loads((base/'dex-selectors.json').read_text()).items():
    out=subprocess.check_output(['target/debug/xtask','apk-smali','../android-lab/apks/WhatsApp.1.apk','--class',cls,'--method',method,'--signatures','--exceptions'])
    assert out==(base/(name+'.txt')).read_bytes(),name
```

Reproduce the four caller censuses using apk-callers APK --class CLASS --method
METHOD: java-key-callers (LX/1FK;,<init>), property-decoder-callers(LX/1GV;,A05),
request-decoder-callers(LX/1GS;,A01), summary-decoder-callers(LX/1GW;,A00).
For every [library,start,end) in native-windows.json:

```sh
llvm-objdump -d --no-show-raw-insn --start-address=START --stop-address=END ../android-lab/output/metrics-native/reconstructed-reproduction/LIBRARY
python docs/protocol/apk/2.26.43.72-media-priority-property-origin/inspect-sources.py ../android-lab/output/metrics-native/reconstructed-reproduction
```

The verifier reproduces source-checks.json. ELF address-to-file mapping uses
section bounds and checks every window. Full disassemblies stay outside Git:

```sh
llvm-objdump -d --no-show-raw-insn ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so > /tmp/wa-property-origin-main.txt
llvm-objdump -d --no-show-raw-insn ../android-lab/output/metrics-native/reconstructed-reproduction/libffsingletonmerged.so > /tmp/wa-property-origin-ff.txt
llvm-objdump -d --no-show-raw-insn ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/wa-property-origin-mc.txt
python docs/protocol/apk/2.26.43.72-media-priority-property-origin/scan-setters.py /tmp/wa-property-origin-main.txt /tmp/wa-property-origin-ff.txt /tmp/wa-property-origin-mc.txt
```

Output reproduces setter-census.json. No new REA Evidence envelope is fabricated:
existing REA category proof and checked DEX/LLVM supplement one another; native
engines remain unavailable. Prefixes and caller snippets are not complete method
proofs. Checksums cover every file except themselves; no APK/library/bulk tree.
