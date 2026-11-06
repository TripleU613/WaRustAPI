# Priority header producer downstream of the WA interceptor

**Confirmed static**, APK2.26.43.72 SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 libwhatsappmerged.so SHA-256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
No execution, network capture, server acceptance or account operation.

## Java and native construction handoff

App selection remains in the preceding
[selection bundle](../2.26.43.72-media-priority-selection/README.md).
The 12-object client constructor forwards the provider as argument9 to
TigonHttpClientServiceHolder; its constructor calls native initHybrid with the
same12 arguments. All checked matching constructors/declarations are retained.
The native tuple at0x74d260 has that exact prototype and points to0x4d6344.
A second tuple0x7434e8 includes NoiseChannelManagerHolder and is NOT the selected
Java declaration. Class-to-table registration for this new initHybrid tuple has
not been traced; prototype identity is a static candidate binding, not runtime
registration proof.

The complete JNI trampoline0x4d6344..0x4d64a0 marshals to0x4d71ec. AArch64
argument9 is at incoming stack+0x10 after JNI env/class and register arguments;
its x29+0x60 is forwarded as the second stack argument. Frame-save helpers are
retained. In construction0x4d71ec..0x4d80a4, that input becomes x25, then
local+0x2f8. If Java provider and unwrapped native pointer are nonnull,
0x4d74e0..0x4d7508 calls virtual slot0x10 with result at local+0x118. The
unwrapper0x4d81a8 resolves the named HttpPriorityCalculatorProvider class.
This slot matches the previously verified WA holder create-calculator0x65fd28.
Null provider/unwrapped pointer yields null calculator.

The raw calculator is placed under a shared control block at0x4d7934..0x4d7968;
control-block vtable0x76a320 RTTI names shared ownership of HttpPriorityCalculator.
The pair at local+0x220/+0x228 is passed to construction/composition routine
0x4f157c at0x4d79f4. It is loaded from the first stack argument, then moved into
local+0xc0 and captured by reference in stack-building closures. No exported
name is assigned to the whole routine: its closures carry buildFullTigonHTTPClientStack
RTTI, but that is not an exported function symbol for its enclosing body.

## Two distinct composition stages

Vtable0x76c430, slot0x30 ->0x4f6864 (RTTI buildFull...$_6) checks the captured
calculator pointer. Only when absent does it add a named
DefaultHttpPriorityContextInterceptor. A supplied WA calculator therefore skips
that fallback at this site. Existing WA context registration is retained in
the User-Agent/priority-context bundles. This stage is not the header producer.

Vtable0x76c970, slot0x30 ->0x4f4aa8 (RTTI buildFull...$_13) copies the supplied
calculator pair or creates DefaultHttpPriorityCalculator when null. It obtains
the next service through0x4adf44. When that service exists it constructs
TigonHttpPrioritizationService, vtable0x77a3a0, storing calculator at service+0x18
and next service at+8. If next service is absent, the stage returns empty service
information. The complete stage is retained, including its service-ID lazy
initialization. Graph dispatch/root/order activation is NOT proven here.

## Confirmed request-header construction

TigonHttpPrioritizationService slot0x10 ->0x661408. It copies the request's
context pair+0x170/+0x178 and calls calculator virtual slot0x10, passing typed
context and a property output container. Result is stored at frame-0x28. This
connects the previously traced WA numeric calculator output to a consumer.

The service clamps the signed first result byte to0..7, writes the24-byte result
to local request+0x158, and constructs header name `Priority` (0xcbdf5). Value:

- `u=` (0xf5308) plus decimal urgency;
- append `, i` (0xe0154) when result byte1 bit0 is set;
- append `, p` (0xdaf4e) when result byte2 bit0 is set;
- append `, r=` (0xf5351) plus decimal integer when the optional integer is
  engaged and positive.

The r input is not merely the calculator's optional member: at0x66157c it reads
property-container pair request+0x28/+0x30 with key3 via0x3ef220, then overwrites
the optional members before formatting. That helper requires a found value with
type tag1, returning its word and engagement bit. Full property-key semantics,
producer, and ordinary-upload presence remain unknown. An initial hypothesis
that r parsed a string/URL was refuted: the complete helper calls the property
lookup and examines a typed value. The numeric key is not a wire header name.

At0x6616a0..0x6616b0, it upserts the constructed key/value into the local request
header container+0x130 via0x663a9c, whose map helper inserts or updates the
returned existing mapped string. It then forwards the local request at0x661758
to next-service virtual slot0x10. This is confirmed header construction, not an
observation of transmitted HTTP/2 or HTTP/3 bytes. Backend stripping/override
and actual graph activation remain unobserved.

## Standard upload inputs and limits on a Rust correction

The prior full JAT.A04 export in `../../rea/2.26.43.72-media-upload-http/`
uses optional K1R.A02/A03 to set request category/purpose at byte0x0204..0x0228.
The selected MOu.A03 creates K1R with argument9=true and argument10=false;
its constructor stores these in A06 and A05. Thus K1R.A00 returns5 before
media-type checks; A02 maps5 to MEDIA_UPLOAD. A01 returns0 immediately when
A06=true; A03 maps0 to NONE. Enum constructors show values8 and0, respectively.
Complete REA A02 corroborates the checked category mapping.

The exported setRequestCategory setter writes native+0x1e0, the first word
copied into WAHttpPriorityContext. Generic Java serialization already retains
the category's explicit value; full native enum-conversion binding remains a
separate check. Given a context category8, the WA calculator returns(3,1,0)
regardless of enable flag or purpose, so this service formats prefix `u=3, i`.
An engaged positive key3 property can still append r. An existing non-WA context
or another upload factory is not covered by that inference. Other K1R branches
can yield IMAGE/VIDEO/API/AUDIO/DOCUMENT/OTHER/GRAPHQL; the17 physical metadata
constructor calls are retained and are not claimed to use the MOu inputs.

**Confirmed Rust source:** image_upload sets Accept-Encoding and optional
configured User-Agent; it has no explicit Priority header. This is a concrete
compatibility candidate. Do not add a universal fixed value yet: named native
registration, actual service graph selection and property3 production/presence
need closing for the selected upload path. No arbitrary configuration key,
header identity or r value was invented. This turn changes research/docs only.

## Negative searches and reproduction

The one-library dynamic symbol search found the HttpPriority setter plus
OS/folly priority imports; previous bundle retains that exact census. Priority
string inventory here is broader RTTI/literal discovery, not use/activation proof.
Java TigonHttpClientConfig constructor scan has OS/TCP/QUIC priority fields but
no explicit prioritization-service enable member in the selected methods; that
bounded search is not absence proof. A mistaken 0x76c320 table probe failed
RTTI lookup; correct control-block address is0x76a320, verified independently.
Default-calculator code is retained as explored context; its full policy is not
attributed to the selected WA calculator. Initial partial native windows were
replaced with complete relevant routines and cold exception tails. Nearest
exported-symbol headings are unrelated; use numeric addresses and RTTI bindings.

From repository root, reproduce every checked DEX export:

```python
import json,subprocess
from pathlib import Path
base=Path('docs/protocol/apk/2.26.43.72-media-priority-header')
apk='../android-lab/apks/WhatsApp.1.apk'
for name,(cls,method) in json.loads((base/'dex-selectors.json').read_text()).items():
    cmd=['target/debug/xtask','apk-smali',apk,'--class',cls,'--signatures','--exceptions']
    if method: cmd+=['--method',method]
    assert subprocess.check_output(cmd)==(base/(name+'.txt')).read_bytes(),name
```

Reproduce metadata-callers.txt with apk-callers APK --class 'LX/K1R;' --method
'<init>'. For each [start,end) in native-windows.json use existing LLVM23.1.3:

```sh
llvm-objdump -d --no-show-raw-insn --start-address=START --stop-address=END ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python docs/protocol/apk/2.26.43.72-media-priority-header/inspect-native.py ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

The verifier reproduces native-bindings.json, using the sibling media-body-provider
ELF parser. Priority string inventory reproduction (rodata has matching file/VA
addresses for these literals; verifier checks the cited VA strings separately):

```python
from pathlib import Path
d=Path('../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so').read_bytes()
pos=0; rows=[]
while True:
    pos=d.find(b'Priority',pos)
    if pos<0: break
    a=d.rfind(b'\0',0,pos)+1; b=d.find(b'\0',pos)
    rows.append(f'{a:#x} {d[a:b].decode("ascii",errors="replace")}\n');pos=b+1
assert ''.join(rows)==Path('docs/protocol/apk/2.26.43.72-media-priority-header/priority-strings.txt').read_text()
```

[Complete REA Evidence and exact CLI reproduction](../../rea/2.26.43.72-media-priority-header/README.md).
Checksums cover all files here except themselves; no bulk APK/library committed.
