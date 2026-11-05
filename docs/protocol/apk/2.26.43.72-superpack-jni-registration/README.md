# Superpack native registration mapping — APK 2.26.43.72

**Confirmed static / derived mapping**, ELF relocations, bounded Rizin assembly
and DEX signature comparison; not target execution or REA Evidence envelopes.
APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Decoder member lib/arm64-v8a/libsuperpack.so SHA256:
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.
Script refuses different digests. Native binaries stay outside the checkout.

## Relocated tables, not nearby name guesses

`python3 inspect-registration.py APK` parses ELF64 section mappings and RELA
entries, accepts selected R_AARCH64_RELATIVE records with zero symbol index,
and resolves the name/signature/function pointers for two measured tables.
The output records table-slot virtual AND file addresses, full strings and
function virtual/file addresses. Addresses are image coordinates, not observed
ASLR process addresses. For example tableVA0x380d0 maps to file0x340d0.

The complete readelf relocation inventory is retained. Reproduce with
`readelf --relocs --wide FILE`. Prior [decoder section/symbol evidence](../2.26.43.72-superpack-decoder-entry/README.md)
supplies the exact same member identity. The script is purpose-built for this
pinned artifact and two traced tables, not a general ELF/JNI recovery tool.

The first table has8 methods for AssetDecompressor. Important entry points:

| Exact selected method | Native image address |
| --- | --- |
|decompress_range |0x9764 |
|decompress_range_from_so |0x9a50 |
|decompress |0x9ebc |
|decompress_from_so |0x9ec8 |
|decompress_legacy |0x9ed4 |
|decompress_with_ref |0xa1dc |

Its remaining entries are sync_file_to_disk and get_architecture. The second
table has5 methods for WhatsAppOpenboxArchive: openNative=0x956c, count=0x95ac,
file pointer=0x95b4, file size=0x95c0 and close=0x95cc. Full signatures and
coordinates are in registration-tables.json. All13 exact class/name/signature
combinations were compared against the retained DEX API exports and matched;
that check does not establish what those native functions do internally.

## Registration call sites connect table shape to Java classes

The exportedJNI_OnLoad entry at0x8f30 branches to0x8fa0. The retained bounded
windows show the JavaVM table call at offset0x30 with version0x10006, followed
by helper0x96d0, another helper0x9184, and helper0x94bc with an Openbox class
String. Failure branches return-1; selected successful registration paths
continue to additional native initialization calls, whose full effects remain
untraced. Windows are explicit address/byte ranges, not automatically recovered
whole-function dossiers or code-coverage claims.

At0x96d0, the class String points to AssetDecompressor. It copies0xc0 bytes
from0x380d0 to the stack, passes that table and count8 to the JNI-table call
at offset0x6b8, and checks the returned status. At0x94bc, the supplied class
String resolves to WhatsAppOpenboxArchive,0x78 bytes from0x38018 are copied,
and count5 is supplied to the same table offset. Pointer stride24 matches
both copy sizes. The additional0x9184 class registration is outside this mapping.

The retained host JNI layout helper, compiled against Temurin21 headers on64-bit,
reports JNINativeMethod size24 and fields0/8/16; JNI RegisterNatives=0x6b8,
FindClass=0x30, ExceptionCheck=0x720 and JavaVM GetEnv=0x30. These comparisons
support the **derived** registration interpretation; they are host-header ABI
checks, not observed Android calls or this APK's original compilation headers.
No target shared library or JNI method was loaded or invoked.

Reproduce bounded assembly with Rizin color disabled: pD4@0x8f30,
pD212@0x8fa0, pD148@0x96d0 and pD176@0x94bc (using spaces in actual commands).
The [previous decoder bundle](../2.26.43.72-superpack-decoder-entry/rizin-version.txt)
retains the tool version. Compile jni-layout.c with JDK/include and its platform
include directory, then run only that host helper.

## Remaining work

This closes selected Java-to-native entry mappings, not archive unpacking.
Next trace0x9ec8/0x9a50 and the Openbox reader into metadata, decompression and
transforms, recover complete inner libraries, then resolve metrics dispatch to
serialization, transport, server responses and durable replay/generation rules.
Other registrations, full error/cleanup and build/ABI variants remain open.
No Rust behavior, telemetry, sender, retry, HTTP route or live account/network
operation is added.
