# Native priority calculator selection: APK2.26.43.72

Confirmed static evidence, not runtime. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 libwhatsappmerged.so SHA-256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
No target execution, device capture, live account or remote configuration fetch.

## Closed boundary

The prior [User-Agent bundle](../2.26.43.72-media-user-agent/README.md) retains
0eA.<clinit> and0eB.A00. At initializer byte0xdc..0xf4, A0z is assigned the
00o.A01 lazy wrapper around new1Wv(2). The producer constructor stores its
argument in $t. Its complete invoke loads $t at0x0000; switch case2 targets
0x1414. That case calls0eA.A06.A14(0x9522), constructs
WAHttpPriorityCalculatorHolder(boolean), and returns it. The key is38178.
Complete checked DEX and independent REA Java confirm the selected path.
Thus the default false holder constructor is not used by this factory path.

00o.A01 constructs00q with null lock and the initializer. 00q.<init> installs
an uninitialized sentinel and selects self as lock. getValue performs a sentinel
check, enters the monitor, checks again, invokes the initializer, stores the
result and clears the initializer only on success. Exceptions release the
monitor and propagate before those assignments. The effective gate is read at
successful lazy initialization, not afresh for every request through that
wrapper. This is a source observation, not proof of when the running app first
initializes it. The factory reads A0z.getValue at0x059c..0x05ac and supplies the
provider to TigonHttpClient at0x05f6.

00E.A14 calls A0D with00G.A03, the supplied key and a null boxed Boolean;
that null is not a default false. A0D passes the key through the generic A06
configuration resolver with a1XK callback, then unboxes its result. Full A06
resolution, stored overrides/default for38178 and effective runtime value are
not established. Do not pin true/false from this static caller.

This connects the provider selected by the app to the
[previous native factory/calculator trace](../2.26.43.72-media-priority-context/README.md):
the boolean is normalized in native provider construction, then copied into
created calculator state. Existing context preservation and conditional numeric
branch mappings remain as documented there. The selected path is now known;
the runtime boolean and downstream scheduling/serialization are still unknown.

## Bounded negative and naming correction

Dynamic-symbol search for priority/prioritization in this one ELF found the
exported TigonRequest::httpPriority(HttpPriority), plus imported OS/folly
priority symbols. This is not a full call-site or priority consumer inventory.
The complete exported setter0x663b10..0x663b2c copies24 bytes to
request+0x158..0x16f. That is distinct from context pair+0x170/+0x178 and the
request metadata+0x1e0/+0x1e4 read by the WA context interceptor. Do not relabel
those metadata words as HttpPriority just from the nearby symbol or shared name.
No connection from calculator result to this setter or wire serializer is
established. The checked TigonRequestBuilder export also has separate
httpPriority and httpPriorityContext members/setters; it does not establish
native offset bindings. The substring selector includes its Companion class.

REA initially rejected X.1Wv; search1Wv returned exactly X.C1Wv and a corrected
inspection succeeded with complete Java. This was an alias mismatch, not missing
APK code. The failed envelope and complete successful Evidence are retained.
No Rust/header/retry/deadline/persistence change follows from this evidence.

## Exact reproduction

Use existing xtask and LLVM23.1.3 from repository root:

```python
import json, subprocess
from pathlib import Path
base=Path('docs/protocol/apk/2.26.43.72-media-priority-selection')
for name,(cls,method) in json.loads((base/'dex-selectors.json').read_text()).items():
    cmd=['target/debug/xtask','apk-smali','../android-lab/apks/WhatsApp.1.apk','--class',cls,'--signatures','--exceptions']
    if method: cmd+=['--method',method]
    assert subprocess.check_output(cmd)==(base/(name+'.txt')).read_bytes(),name
```

Native output reproduction:

```sh
llvm-nm --dynamic --demangle ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so | rg -i 'priority|prioritization'
llvm-objdump -d --no-show-raw-insn --start-address=0x663b10 --stop-address=0x663b2c ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python docs/protocol/apk/2.26.43.72-media-priority-selection/inspect-native.py ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

Outputs are native-priority-symbols.txt, native-priority-setter.txt and
native-window.json. The verifier reuses the digest-pinned ELF parser in the
sibling media-body-provider bundle; complete REA reproduction and file digests
are in [its companion](../../rea/2.26.43.72-media-priority-selection/README.md).
Checksums cover every file here except themselves. No bulk APK/native library.
