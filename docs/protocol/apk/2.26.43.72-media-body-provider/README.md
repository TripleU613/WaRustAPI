# Upload body provider and native length handoff — APK2.26.43.72

**Confirmed selected static**, build264307238, pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target/account execution or reconstructed library loading. This follows the
[body-mode gate trace](../2.26.43.72-media-body-mode/README.md).
[Complete fresh REA method](../../rea/2.26.43.72-media-body-provider/README.md)
corroborates checked DEX. Native inspection below is host ELF/LLVM analysis,
not a successful REA native decompilation.

## Length reporting and callback bytes are separate

Retained JAT.A04 passes first-stream declared length to IxI's long argument;
its two Boolean arguments are false,false. Constructor stores these asA06/A07,
so this selected path uses neither optional GZIPOutputStream nor worker dispatch.
IxI.getContentLength returns min(A02,2147483647), preserving negative sentinel
values rather than clamping to zero. A00 invokes that virtual getter, casts to
int and reports it through TigonBodyStream.reportBodyLength before writing.
This is NOT evidence of a 2GiB actual-media cap or body truncation.

A00 wraps the stream in JOT,1lT and BufferedOutputStream; selected A06=false
calls the supplied Function1 directly with that buffered stream. KXB selector1
(at015a..0188) calls K2k.A06(route,output,null,boundary), retaining the previously
traced raw/multipart decision and optional EOF processor. It closes/flushes the
buffered stream before writeEOM on the normal selected path. The callback has
no length parameter here; no comparison/truncation against reported length is
shown in this Java provider. Downstream native behavior is separate authority.

JOT.write(byte[],offset,length) forwards nonzero writes to transferBytes; return1
throws IOException("Stream was cancelled"). This return is not treated as a
partial-byte count, and no partial-byte resend loop is present here. A00 catches
selected stream exceptions, converts them to TigonError and calls reportError.
Its complete exception table remains in implementation.txt. This finding does
not authorize retry after any partial or uncertain write.

TigonXplatBodyStream implements the wrapper methods, forwarding to private native
reportBodyLengthNative(I)V, transferBytesArrayNative([BII)I and writeEOMNative()V.
The superclass-only override census fails to enumerate this interface implementer:
its declared coverage explicitly excludes interface dispatch. It is retained as
a negative tool-boundary example, not proof no implementation exists.

## Pinned native tuple and forwarding boundary

No literal libtigonjni.so entry exists among this APK's12 .so entries; the class
initializer's load name does not imply a standalone ZIP member. Reused existing
[reconstructed ARM64 members](../2.26.43.72-superpack-reconstructed-members/README.md).
Verified libwhatsappmerged.so size7871544 and SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`
against existing reconstruction metadata before analysis. No native artifact
is committed or loaded. inspect-native-tuples.py validates this identity, maps
ELF sections/RELA entries, extracts four exact JNI name/signature/function tuples,
and hashes the selected assembly windows. Independent readelf relocation text
agrees with its addresses. Tuple presence is not observed registration execution.

reportBodyLengthNative tuple at756090 names wrapper64fe3c. In selected assembly,
64fe5c retains w2 inw19;64fe78 sign-extends w19 tox1; helper72921c loads the
object vtable and slot0x30;64fe80 calls that indirect x8. Thus integer reporting
reaches a native virtual boundary. The concrete vtable target and wire framing
are NOT resolved. LLVM nearest-symbol headings such as QlogCollector/MCFSetGetCount
are display labels, not claims that these unnamed routines are those functions.
Transfer/EOM wrapper windows are retained for continuation without assigning
unresolved virtual targets or server semantics.

REA open_binary identified the pinned ELF. Its subsequent search_strings failed
provider_unavailable (Ghidra/Hopper missing, IDA unconfigured; current connection
also has the old Java path). Full error is retained as tool diagnostics, NOT
REA Evidence or successful native search. Existing llvm-objdump23.1.3 and readelf
were used instead. Optional Capstone was absent in both checked Python contexts;
it was not installed and is not required by the stdlib resolver.

**Unknown:** concrete native slot0x30 target, transfer length enforcement/header
synthesis, final wire framing, effective0x666, all processor-adjusted length inputs
and transport selection. Rust's actual known body length includes its finished
voice seal; no evidence here justifies truncating that body to the APK's reported
pre-processor length. No Rust behavior change or live parity claim.

## Reproduction

From repository root, with the already reconstructed pinned library:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
target/debug/xtask apk-smali "$APK" --class 'LX/IxI;' --signatures --exceptions > implementation.txt
target/debug/xtask apk-smali "$APK" --class 'Lcom/facebook/tigon/TigonXplatBodyProvider;' --signatures --exceptions > base.txt
target/debug/xtask apk-smali "$APK" --class 'LX/KXB;' --method invoke --signatures --exceptions > writer-callback.txt
target/debug/xtask apk-smali "$APK" --class 'LX/JOT;' --signatures --exceptions > output-adapter.txt
target/debug/xtask apk-smali "$APK" --class 'Lcom/facebook/tigon/TigonBodyStream;' --signatures --exceptions > stream-interface.txt
target/debug/xtask apk-virtual-overrides "$APK" --class 'Lcom/facebook/tigon/TigonBodyStream;' --method reportBodyLength --signature '(I)V' > stream-overrides.txt
target/debug/xtask apk-smali "$APK" --class 'Lcom/facebook/tigon/TigonXplatBodyStream;' --signatures --exceptions > xplat-stream.txt
python3 docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py "$LIB" > native-tuples.json
llvm-objdump -d --no-show-raw-insn --start-address=0x64fe3c --stop-address=0x64ff08 "$LIB" > native-length-disassembly.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72921c --stop-address=0x729228 "$LIB" > native-vcall-disassembly.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x64fbe0 --stop-address=0x64fe3c "$LIB" > native-transfer-eom-disassembly.txt
readelf -rW "$LIB" | rg '^00000000007560(48|50|58|60|68|70|78|80|88|90|98|a0) ' > native-tuple-relocations.txt
```

checksums.json covers every other file with bytes and SHA256. Native library
recovery reproduction and metadata verification remain in the linked bundle.
