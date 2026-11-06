# Selected native submission reaches the Java-body adapter — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Verified reconstructed ARM64 libwhatsappmerged.so SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
No library/target/account execution. Host ELF/LLVM evidence, not a new REA native
decompilation. Follows [selected Java submission](../2.26.43.72-media-java-native-submit/README.md).

## Exact tuples and conditional provider chain

Relocated JNI tuple7561e0 maps sendRequestBodyBufferIntegerBuffer with the exact
Java provider overload signature to650ff0. Tuple755fd0 maps initHybrid()HybridData
to64f83c. The existing Java IxI ->TigonXplatBodyProvider constructor invokes
an inherited native initHybrid with that signature. Static class-to-table
registration is not traced here; treat this as its matching factory candidate. Complete selected factory64f83c..64f9c8
allocates16 bytes and installs native TigonXplatBodyProvider address point7788d0
at64f86c..64f874. Static RTTI and executable relocation checks bind that table's
slot0x10 to64fa04. Tuple presence is not observed JNI registration execution.

The selected submit entry receives body provider inJNI x5, retains it inx23
at651024, and checks it for null. Its service Java receiver is retained by
helper72a1cc and resolved through its hybrid accessor. On the nonnull-provider
branch, the provider peer's slot0x10 is called at651070..651080 with a service
value loaded fromnative service+0x18. Exact class/peer-accessor guard behavior
has not been universally reconstructed, so this is a static call-chain binding,
not proof that every possible Java subclass supplies the same native peer.
For the retained inherited factory, that slot resolves to64fa04.

Complete adapter64fa04..64faa4 allocates120 bytes and installs address point778780
at64fa44..64fa50. That table's RTTI is TigonJavaBodyProvider, slot0x10=64e908.
It retains two values at+0x68/+0x70. The latter comes from its x1 argument;
its service producer/effective meaning remains untraced. Neighbor slot0x18
returns provider name Java; slot0x20 has a shared-pointer-copy-shaped routine.
Their exact virtual API names are not inferred from neighboring offsets.
The copy window was inspected but is not called a size/length getter.

Conditional on the matching peer/factory binding, this connects the Java upload
provider to the previously inspected
TigonJavaBodyProvider stream-start routine64e908, rather than choosing it by
string proximity alone. That routine's selected nonzero+0x70 path constructs
TigonLimitingBodyStream; zero bypasses that wrapper. Its previously traced
length-report slot forwards unchanged. The effective+0x70 input is still unknown,
so no unconditional wrapper-selection or framing claim follows.

## Request bytes and downstream native conversion

Selected JNI entry retains request byte-array used length inw19 (originalw4)
and property used length inw24 (originalw7). It adapts the provider separately,
then calls651860 with the serialized request buffer and its used length at
651140..651148. Header lengths/counts in that INTERNAL serialized buffer must
not be confused with HTTP body Content-Length.

The retained converter PREFIX651860..651b64 constructs a buffer reader, reads
two strings through6536b8 (matching Java method/URL serialization order), reads
a32-bit count, and processes that many key/value string pairs into a native
map-like structure. Negative count is clamped to0 before the loop. The loop's
duplicate-key handling and entire reader/remaining request conversion are not
fully interpreted here. This prefix does not establish whole-stack automatic
headers, accepted malformed input, or the on-wire request. Its many fields and
metadata after the retained prefix remain open.

The JNI entry retains further callback/executor/properties adaptation and cleanup
paths in full. Target-executed exceptions, callback completion, native middleware
and request replay policy are not established by static cleanup instructions.
No runtime retry/resend authority or acceptance claim is derived.

**Unknown:** effective service+0x18 / adapter+0x70 value; complete peer guard and
registration lifetime; remaining converter/middleware; header synthesis when JAT
omits explicit length; full wire framing and actual transport. No Rust behavior
changed. Known sealed wire_body and uncertain reservations remain unchanged.

## Reproduction

From repository root, using the existing recovered pinned library:

```sh
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python3 docs/protocol/apk/2.26.43.72-media-native-submit-provider/resolve-submit.py "$LIB" > resolution.json
llvm-objdump -d --no-show-raw-insn --start-address=0x650ff0 --stop-address=0x651344 "$LIB" > entry.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x651860 --stop-address=0x651b64 "$LIB" > converter.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64f83c --stop-address=0x64f9c8 "$LIB" > provider-factory.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64fa04 --stop-address=0x64faa4 "$LIB" > provider-adapter.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64f064 --stop-address=0x64f090 "$LIB" > provider-metadata.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x3ed468 --stop-address=0x3ed49c "$LIB" > provider-copy.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64e908 --stop-address=0x64ec00 "$LIB" > stream-start.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72a1cc --stop-address=0x72a1d8 "$LIB" > jni-arguments.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72e194 --stop-address=0x72e1a0 "$LIB" > buffer-arguments.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x732ba8 --stop-address=0x732bb4 "$LIB" > adapter-result.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72f908 --stop-address=0x72f914 "$LIB" > adapter-arguments.txt
```

LLVM23.1.3, Python stdlib and prior digest-pinned ELF resolver. Numeric entries,
not nearest exported-symbol headings, identify routines. resolution.json verifies
exact JNI signatures, primary-table RTTI/zero top offset/executable entries and
selected byte hashes. checksums.json covers every other file with bytes/SHA256.
Partial windows/prefixes are explicitly labelled; full binaries stay local.
