# Metrics JNI declarations and packaged native carrier — APK 2.26.43.72

**Confirmed static**, checked DEX declarations and GNU readelf output. REA
provider status is an observed environment check, not target execution, a native
function trace or a REA Evidence envelope. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK/native binary is committed. Every retained file is checksummed; readelf text has trailing whitespace removed.

## Exact native declarations

The selected metrics event call resolves the Java native declaration
jvidispatchIIIIIOO(JJJJ,Object,Object):long. Setup calls native
jvidispatchIOOOO(int,Object,Object,Object,Object):long. Both have no DEX code. A digest-bound method-metadata scan confirms flags0x109
(public/static/native), abstract=false and code_off=0 for both exact names.
Reproduce with `python3 inspect-native-flags.py APK`; output is retained.
The event method export is exact; the setup declaration was selected by its full
signature from seven substring matches. The instruction call site and native
method spelling are not proof of a native export address or operation semantics.

Reproduce with `xtask apk-smali APK --class
Lcom/whatsapp/wamsys/JniBridge; --method METHOD --signatures --exceptions`,
then select the exact full signature. The previously retained event/setup
workers remain the source for argument flow and discarded return values.

## ARM64 carrier identity and coordinates

The APK member inventory has libs.so, libsuperpack.so and libunwindstack_binary.so
for four ABI directories. It does not expose a separately named wamsys/WhatsApp
library member in this inventory. The selected lib/arm64-v8a/libs.so is14383776
bytes, SHA256
`1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5`.
Carrier identity and all native member names/sizes are retained in JSON.

Readelf identifies this selected file as ELF64 little-endian AArch64 ET_DYN.
Its complete dynamic-symbol table has six entries: null, two undefined libc
functions, and three superpack archive markers. No named JNI dispatcher appears
in that table. This does not prove absence of code inside the packaged archive,
absence of static symbols or absence of dynamically registered native methods.

_superpack_archive_start/end are virtual addresses0xc870 and0xdc2eb1; the
absolute size symbol is0xdb6641. Section19 .data instead starts at file offset
0x870 with size0xdb6641. These virtual/file coordinates differ and must not be
interchanged when extracting the range. This is an archive carrier boundary,
not a located metrics function. Compression/container format, inner members,
architecture mapping and JNI registration are not decoded in this chunk.

Reproduce by reading that one member from Python zipfile outside the checkout,
checking its SHA256, then running `readelf -h FILE`, `readelf --sections --wide
FILE` and `readelf --dyn-syms --wide FILE`. Reading never loads or executes it.

## Current REA environment boundary

The live binary_session check returned no open target. Its selected availability
projection is retained with REA6.5.0 identity: Ghidra/Hopper executables are
missing under their configured paths; IDA is not configured. The advertised
per-provider operation catalogue is not evidence that a provider is currently
usable. This environment check is dated2026-11-05 and may change later.
No unavailable provider was repeatedly invoked and no MCP configuration was
changed. Existing local readelf supplied the bounded carrier evidence instead.

## Next native proof obligations

Unpack the verified superpack carrier, identify/hash the actual inner library,
resolve JNI registration and exact dispatcher operation, then trace event/setup
handling to serialization, durable buffers, flush/endpoint, responses and retry
ownership. Searching wrapper names in this carrier's dynamic symbols alone
cannot close those obligations. No Rust sender, telemetry, replay authority,
route, provider invocation or live account/network operation is added.
