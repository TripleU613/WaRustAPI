# HTTP priority context: static audit, 2.26.43.72

APK SHA-256: `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 libwhatsappmerged.so SHA-256:
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
Static only; no target execution or live account operations.

## Confirmed and bounded

The checked constructor initializes HybridData with native initHybrid(). Its
one physical constructor caller is RlW, retained in the preceding
[User-Agent bundle](../2.26.43.72-media-user-agent/README.md), which also holds
the selected app factory's interceptor order. Full constructor REA Evidence is
in [the companion directory](../../rea/2.26.43.72-media-priority-context/README.md).

Native registration 0x6602d8..0x6603ec binds calculator holder class slice
0xf5921/55 to table0x756f40 (initHybrid(Z), thunk0x65fc1c), and context
interceptor slice0xf5720/57 to table0x756fd0 (initHybrid(), thunk0x65fe28).
Each registration count is1. These thunks name their factory addresses directly:
0x65fc2c and0x65fe38. JNI RegisterNatives helper0x39fbe0 and ABI slot were
independently checked in the earlier JNI registration bundle; actual runtime
load/registration remains unobserved.

The interceptor factory installs vtable0x77a110, associated via RTTI with
JWAHttpPriorityContextInterceptor. Slot0x10 reaches makeInterceptor0x65ff28,
which names WAHttpPriorityContextInterceptor and constructs std::function
vtable0x77a140. Its slot0x30 invokes0x660000. That complete invocation reads
request shared-owner pair+0x170/+0x178 and preserves an existing non-null
context. When absent, it allocates a shared WAHttpPriorityContext: copies the
8 bytes at request+0x1e0 to context+8 (relative to the context subobject),
and a string derived through0x6642d8 from request+0x28/+0x30 to context+0x10.
The stored context subobject is allocation+0x18. Its vtable is0x77a1e0,
RTTI WAHttpPriorityContext. Pair replacement0x4d80e8 moves both pointers,
zeros the source pair and releases the old owner. Normal completion writes value1
to result byte0 through the outlined result helper (the invocation's w8=1);
this is local interceptor completion, never server acceptance.

The calculator holder has a Java default constructor forwarding false to its
boolean constructor; that constructor passes its value to native initHybrid.
The native factory normalizes the input boolean and stores it at provider+8;
provider vtable0x77a0b0 slot0x10 reaches create-calculator0x65fd28, which copies
that flag into calculator+8 and installs vtable0x77a0e0. Its slot0x10 reaches
0x65fd6c. The calculator invocation0x65fd6c dynamically casts
its supplied context to WAHttpPriorityContext and consults that flag. When no
matching context or flag!=1 it returns byte tuple(3,1,0), with optional members
at result+8/+0x10 disengaged. With a matching context/flag1 it reads the two
32-bit context words+8/+12. First word1,10,11 yields (4,0,0) if second==1,
otherwise(3,0,0). First word2 yields(4,0,0) if second==1, otherwise(3,1,0).
Other first words use the fallback. Numeric meanings are not established here.

**Bounded negative:** the complete interceptor invocation attaches context; it
does not directly call the header insertion/lookup path or write the previously
identified request header container+0x130. This is not evidence that downstream
scheduling never influences HTTP/2/HTTP/3 priority bytes or headers. Do not add
a speculative Priority header to Rust from this finding. Effective calculator
provider selection, enabled flag, enum meanings, final scheduling and protocol
serialization remain unknown. No Rust request, retry, deadline or state changed.

Initial broad low-offset 0x71f string-reference search hit unrelated helper
0x6fbee4 (points at0xe971f, not the class at0xf571f); that was rejected as
identity evidence. Nearby partial routines at0x65fb00 and after0x6603ec were
not attributed to this interceptor. Nearest exported-symbol headings in LLVM
output are unrelated; numeric ranges plus relocations/RTTI are authoritative.
String helper0x6642d8 is retained, but its complete semantic label is unresolved.

## Exact reproduction

From repository root, using existing xtask and LLVM23.1.3:

```sh
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class WAHttpPriorityContextInterceptor --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class WAHttpPriorityCalculatorHolder --signatures --exceptions
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/infra/tigon/WAHttpPriorityContextInterceptor;' --method '<init>'
python docs/protocol/apk/2.26.43.72-media-priority-context/inspect-native.py ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

The three DEX commands reproduce interceptor-class.txt, calculator-class.txt,
and interceptor-callers.txt. The Python verifier reproduces native-bindings.json;
it reuses the digest-pinned section/relative-relocation parser in the sibling
media-body-provider bundle. For every [start,end) in native-windows.json:

```sh
llvm-objdump -d --no-show-raw-insn --start-address=START --stop-address=END ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

Output reproduces the named .txt. Checksums cover every file except themselves.
No APK, native library or bulk decompilation is committed.
