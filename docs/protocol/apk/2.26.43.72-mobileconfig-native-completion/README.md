# MobileConfig native completion entry and stored callable

**Confirmed selected static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Arm64 reconstructedlibmobileconfig-jni.so is852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
This exactly matches the prior [validated superpack member](../2.26.43.72-superpack-reconstructed-members/validation.json),
including its independent reconstruction checks. No native library or APK runs.

## Relocations close the Java/native entry boundary

The Java handler's onCompletion(Z,String) declaration from the prior
[fetch route](../2.26.43.72-mobileconfig-fetch-route/README.md) matches this
three-pointer method table through R_AARCH64_RELATIVE addends:

| Table slot VA | Relocated value |
| --- | --- |
| 0xd1f20 | 0x184a5: onCompletion |
| 0xd1f28 | 0x1e248: (ZLjava/lang/String;)V |
| 0xd1f30 | 0x59574: native entry |

The raw file words arezero: interpreting unrelocated bytes as pointers would miss
this binding. inspect-binding.py parses ELF program/section/RELA records, checks
prior member identity and this exact table. The registration window at6ef58
loads it and constructs the class name from52 bytes at1b7ed. The stored C string
continues with a semicolon; the explicit52-byte length excludes that semicolon.
The class span iscom/facebook/mobileconfig/MobileConfigFetcherHandler. A count1
registration call follows;4f0f8 dispatches through the JNI environment table at
0x6b8 and calls throwCppExceptionIf for a nonzero return. No claim about other
registrations or classloader replacements follows from this selected window.

## Completion is a stored-callable handoff

In59574's bounded assembly window, the wrapper saves the Java boolean/string,
sets up JNI environment/holder access, converts JString usingtoStdString, tests
the boolean low byte and cset normalizes it into w1. It loads the holder's stored
pointer at offset0x30 into x0 and calls5971c with the temporary string address in
x2. Temporary string/JNI cleanup follows. This proves argument handoff, not JSON
parsing, persisted data or a successful configuration transaction.

5971c saves the boolean byte, checks the stored pointer, loads its vtable slot
0x30 and invokes it indirectly at59748. The stack-guard helpercaa70 is retained
so its BL is not mistaken for another semantic callback. If no callable exists,
59778 allocates and throwsbad_function_call, as confirmed by selected imported
RTTI/vtable/destructor relocations. Missing callback is not silent success.
The concrete stored callable, its construction and captured owner remain open;
the boolean can still represent Java's failure or success from the prior trace.
Neither branch is treated as policy activation simply because this bridge returns.

## REA evidence and bounded negative result

rea-negative-xref.json retains the complete REA/Rizin observation for
`aaa; axtj @0x184a5`: it returned an empty array with an entrypoint warning, exit0,
no output truncation and unchanged artifact hash. This is only that provider's
xref result; it is not proof that the string is unused. The independently parsed
relocation table resolves it. rea-thunk-window.json retains the complete REA
`pd80 @0x59574` command observation.80 instructions is an explicitly bounded
window, not complete function/exception/callgraph coverage; exit0 does not remove
its provider limitations. LLVM windows provide a second disassembly presentation.
No provider pseudocode or recovered runtime pointer identity is claimed.

## Reproduction and next step

```sh
python3 docs/protocol/apk/2.26.43.72-mobileconfig-native-completion/inspect-binding.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-native-binding.json
cmp /tmp/mc-native-binding.json docs/protocol/apk/2.26.43.72-mobileconfig-native-completion/binding.json
```

Assembly windows use llvm-objdump -d --no-show-raw-insn with ranges59574..5971c,
5971c..597b0,6eec0..6f030,4f0f8..4f148,caa70..caa80. Their headings may show
nearest exported symbolJNI_OnLoad even for unrelated local functions; addresses,
not those fallback labels, identify this evidence. readelf-rW reproduces selected
relocations. Reconstruct the exact member using the prior validated workflow if
absent; never substitute another build's library.

Next: native stored-callable construction, captured account/config owner,
response parsing, retry policy, file commit and cache notifications. No Rust
request or persistent account state changed. No live operation, private account
read, native execution or zero-ban/server-acceptance claim.
