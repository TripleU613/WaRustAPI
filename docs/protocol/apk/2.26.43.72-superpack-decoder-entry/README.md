# Superpack decoder entry and narrow usability probe — APK 2.26.43.72

**Confirmed static**, checked native bytes/symbols and DEX TXT. No target code
was executed; Rizin disassembled a measured export, without a REA provider or
invented Evidence envelope. The JNI layout helper is a locally compiled host
program, not the APK decoder. All retained files are checksummed.

## Decoder identity and exact exported function

Selected member lib/arm64-v8a/libsuperpack.so is220920 bytes, SHA256
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`,
from pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The complete dynamic-symbol table and section inventory are retained. The
usability export is symbol address0xa4cc, size56. Section14 .text has matching
virtual/file base0x8f24 in THIS decoder, so the selected56-byte function's file
offset is also0xa4cc. That equality must not be generalized to the separate
[libs.so carrier](../2.26.43.72-metrics-native-carrier/README.md).

Reproduce by extracting the one member outside the checkout, checking its hash,
running `readelf --dyn-syms --wide FILE`, `readelf --sections --wide FILE`, then
`rizin -2 -q -e scr.color=0 -c 'pD 56 @ 0xa4cc; q' FILE`. Tool version and
function-byte digest are retained. No auto-analysis or native execution is
claimed. The measured body has14 AArch64 instructions ending at0xa500/ret.

## Usability check constructs constants, not an archive-decode exercise

The body stores0x53 as one byte and0x7747 as a little-endian halfword in a
three-byte stack region, forming47 77 53. It passes that region with length3,
start0 and the incoming byte-array argument to an indirect call through the
JNI table at offset0x680, then returns. There is no archive argument, compressed
input, loop or decoder call in this measured export.

The retained host helper uses offsetof(JNINativeInterface_,SetByteArrayRegion)
against the installed Temurin21 JNI headers; on64-bit host it reports0x680.
This supports the **inference** that the indirect operation sets the array
region under the standard JNI table ABI. It is not the APK's own build headers
or an observed Android runtime call. Reproduce by compiling jni-layout.c with
JDK/include and JDK/include/linux, then run the host helper.

The [APK loader](../2.26.43.72-superpack-unpack-boundary/loader-entry.txt) compares
that three-byte array against the same literals. Therefore this selected probe
is a narrow library/JNI linkage check, not evidence that a whole archive or any
inner native library successfully decompresses. The previously recorded12 XZ
outputs remain partial payloads and must not be upgraded to recovered ELF files
because this separate probe's constants match in source.

## Openbox Java wrapper identifies additional native boundaries

Complete WhatsAppOpenboxArchive DEX export is retained, reproduced with
`xtask apk-smali APK --class Lcom/whatsapp/superpack/WhatsAppOpenboxArchive;
--signatures --exceptions`. It holds an AssetFileDescriptor and a reader pointer.
Lazy initialization double-checks a zero pointer under its monitor, passing file
descriptor, start offset and length to openNative and storing the returned long.
getFileCount delegates through that reader; getFile(index) rejects negative or
out-of-count indices and obtains native pointer/size before constructingPFy.
The named native boundaries have no DEX implementation in this export.

This identifies a range-based native reader interface, not a decoded metadata
schema, an established mapping from a raw XZ stream to a file index, or recovered
member names. Native registration/function mapping, archive transforms andPFy's
read behavior still need tracing. No native wrapper was invoked.

## Remaining work

Resolve actual decoder registration and container transforms, reconstruct and
validate complete inner libraries, then follow metrics JNI to schema, storage,
flush/endpoint, responses and durable generation/replay ownership. Current
Rizin evidence closes only the measured entry above; it does not establish full
decoder reconstruction or metrics transport parity. No Rust behavior, sender,
telemetry, retry, HTTP route or live account/network operation changed.
