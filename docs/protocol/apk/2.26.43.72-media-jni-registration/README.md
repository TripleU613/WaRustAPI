# Upload JNI class-to-table registration — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Verified reconstructed ARM64 libwhatsappmerged.so SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
Host ELF/LLVM evidence; no library/target/account execution or new REA native
decompilation. This closes the immediate STATIC class-to-table uncertainty in
[the previous provider-chain trace](../2.26.43.72-media-native-submit-provider/README.md).

## Factory registration is explicit

Complete selected routine64cf40..64d11c constructs exact class-name slices and
calls findClassLocal, then dispatches each class/table/count through39fbe0:

| Class suffix undercom/facebook/tigon/ | Table | Count | Selected entry |
|---|---|---|---|
| TigonXplatBodyProvider | 0x755fd0 | 1 | initHybrid ->0x64f83c |
| TigonXplatBodyStream | 0x756048 | 5 | reportBodyLengthNative ->0x64fe3c |
| TigonXplatRequestToken | 0x7560f0 | 5 | cancellation/priority/pause/resume bindings only |
| TigonXplatService | 0x756198 | 4 | sendRequestBodyBufferIntegerBuffer ->0x650ff0 |

Full method names/signatures/function entries are inbindings.json. Token method
registration is not a trace of cancellation semantics or retry permission.

At64cf58..64cf60 helper728a90 reads the three-pointer tuple from755fd0 intoq0/x9;
at64cf7c/64cf80 those fields are copied to the stack table. Class name bytes
f5dd3..f5dfc are passed with exact length41 to string construction and
findClassLocal. Helper72e7fc supplies that copied tuple and count1 before
64cfac dispatch. This is direct static registration of the previously identified
factory, not just a same-name initHybrid tuple elsewhere in the library.

The remaining stream/token/service tables are copied bymemcpy with lengths
120/120/96 (5/5/4 JNI triples) and paired with their explicit class names and
counts. The full registration routine includes its cleanup/error exits. A
near-factory window previously inspected contains unrelated adjacent destructors;
it did not supply registration evidence and is retained as an exclusion.

## RegisterNatives dispatcher independently identified

39fbe0's helper310600 retains inputclass-reference pointer, table pointer and
count. It obtains currentJNIEnv, loads the class reference, supplies methods
and count asx2/w3 and calls JNI function-table slot0x6b8. Nonzero return is
converted to throwCppExceptionIf. This records the error route but is not an
observed successful result.

Independent local Temurin21.0.13 JNI.h identifies RegisterNatives as table
index215, including four reserved slots; ARM64 eight-byte pointer indexing gives
215*8=0x6b8. inspect-jni-slot.py parses the actual header struct and verifies its
SHA256 and index rather than guessing from a nearest ELF symbol. The header
is external primary ABI evidence, not APK/runtime evidence. No full JDK source
or binary is committed. This supports the selected dispatch interpretation;
Android JNI initialization/ABI execution was not exercised.

## What is now closed and still unknown

Static class-to-table binding now supports the inherited Java IxI/Xplat factory
chain: Xplat factory creates its native peer, whose slot0x10 adapter creates
TigonJavaBodyProvider, whose stream-start slot is64e908. Prior reports correctly
left this registration untraced; this new bundle supplies that missing evidence.
Native dispatch, actual loaded class identity and runtime setup success remain
unobserved. No registration was replayed, invoked or reset.

Effective service/adapter wrapper gate, remaining native request conversion,
header synthesis and wire framing remain unknown. A registration table says
which native code a method uses, not which operation branch ran or what the
server accepted. No Rust behavior/request/header/timer, saved account state,
retry or replay changed. Next useful scope remains native request conversion
and its actual header middleware, with this provider association now grounded.

## Reproduction

From repository root, using existing recovered library and installed JDK:

```sh
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python3 docs/protocol/apk/2.26.43.72-media-jni-registration/resolve-registration.py "$LIB" > bindings.json
python3 docs/protocol/apk/2.26.43.72-media-jni-registration/inspect-jni-slot.py /home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS/include/jni.h > jni-slot.json
llvm-objdump -d --no-show-raw-insn --start-address=0x64cf40 --stop-address=0x64d11c "$LIB" > body-provider.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x39fbe0 --stop-address=0x39fc30 "$LIB" > dispatch-helper.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x310600 --stop-address=0x310610 "$LIB" > dispatch-arguments.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x728a90 --stop-address=0x728aa0 "$LIB" > table-helper.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72e7fc --stop-address=0x72e808 "$LIB" > arguments.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72baf4 --stop-address=0x72bb00 "$LIB" > array-arguments.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64f790 --stop-address=0x64f83c "$LIB" > near-factory.txt
```

Python stdlib, existing digest-pinned ELF resolver and LLVM23.1.3. Numeric
addresses identify entries; LLVM nearest exported-symbol labels do not.
bindings.json pins class slices/tuples/window hashes; jni-slot.json identifies
its independent source digest. checksums.json covers every other file with
bytes/SHA256. Runtime invocation and registration success are explicitly unclaimed.
