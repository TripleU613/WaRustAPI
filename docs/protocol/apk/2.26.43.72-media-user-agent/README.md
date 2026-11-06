# HTTPS media User-Agent interception (2.26.43.72)

Confirmed static evidence, not runtime or server acceptance. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 `libwhatsappmerged.so` SHA-256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`
(7,871,544 bytes). No target execution or live requests.

## Finding and scope

`0eA.<clinit>` assigns provider key 0x30075 to A04. `1Uw.A00` dispatches
(id >> 14) & 0x7f = 12 to RNH.A00; its id & 0x3fff = 117 branch
constructs RlW. RlW obtains 0lR.A02/A04 and installs WAUserAgentInterceptor,
then WAHttpPriorityContextInterceptor. `0eB.A00` obtains that holder and passes
it to TigonHttpClient; its request order names the same two interceptors.
The client constructor census includes other clients/delegating constructors:
this is not proof that all Tigon clients use this holder. `0eA.A00` selects
lazy clients A0r/A0q; full lazy initialization and effective runtime configuration
are not established here.

Named native registration at 0x4d5004 binds class slice 0xf575c/47 to the
single initHybrid tuple at 0x74d0a8, function 0x4d50b8. Its object stores the
two Java strings at +8/+0x20. Vtable 0x769d30 slot0x10 reaches makeInterceptor
0x4d5320; the named std::function wrapper 0x769d60 slot0x30 reaches invoke
0x4d5644. The complete invoke checks HTTPS via 0x6689e8, looks up User-Agent
via 0x664338, and adds it through 0x66386c only when absent and the selected
captured string is nonempty. Request category word +0x1e0 selects the second
string for values 3/5 and the first otherwise. Category meanings and complete
UA formatters are unknown; 0lR.A02's cache miss invokes A00 with 2.26.43.72,
while A04 invokes A01(null). Do not invent device or User-Agent values.

The direct interceptor constructor census returns zero, refuted as an absence
proof by RlW's optimized Object.<init> plus explicit native initHybrid call.
The initHybrid census finds RlW and SZl forwarding. The lambda-factory window
is copying/cloning context, not the invocation. Small helper windows can include
the next prologue; nearest exported-symbol headings are not function identity.
There is no observed registration success, effective interceptor activation,
server requirement, ban cause, or guarantee of avoiding bans.

## Reproduction

From repository root, use existing built xtask and LLVM 23.1.3. Each entry of
`dex-selectors.json` is [kind, exact class descriptor, method or null]:

```python
import json, subprocess
from pathlib import Path
base=Path('docs/protocol/apk/2.26.43.72-media-user-agent')
apk='../android-lab/apks/WhatsApp.1.apk'
for name,(kind,cls,method) in json.loads((base/'dex-selectors.json').read_text()).items():
    cmd=['target/debug/xtask','apk-callers' if kind=='callers' else 'apk-smali',apk,'--class',cls]
    if method: cmd += ['--method',method]
    if kind!='callers': cmd += ['--signatures','--exceptions']
    result=subprocess.check_output(cmd)
    assert result==(base/(name+'.txt')).read_bytes(), name
```

DEX method filters are substrings; retained output includes complete matching
methods. Printed instruction spans use byte offsets, branch deltas code units.
Reproduce native bindings and compare native-bindings.json:

```sh
python docs/protocol/apk/2.26.43.72-media-user-agent/inspect-native.py \
 ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

The script reuses the digest-pinned ELF section/relative-relocation parser from
`../2.26.43.72-media-body-provider/inspect-native-tuples.py`. For each range in
native-windows.json reproduce the corresponding .txt with:

```sh
llvm-objdump -d --no-show-raw-insn --start-address=START --stop-address=END \
 ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

Full successful REA holder Evidence and exact reproduction are in
`../../rea/2.26.43.72-media-user-agent/`. No bulk decompilation or native library
is retained here. checksums.json covers every file in this directory except itself.
