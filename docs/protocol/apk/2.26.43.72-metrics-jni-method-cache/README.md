# Metrics outbound JNI method-cache binding — APK 2.26.43.72

**Confirmed static / derived; observed bounded synthetic emulation.** Pinned APK
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Recovery provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No Android/JNI runtime, class loading, real native registration or network
operation was executed. Local Rizin exports and Unicorn results are not REA
Evidence envelopes. This closes the earlier **static outbound cache-binding**
gap; successful initialization in a live process remains unobserved.

## Exact class, method and slot

ELF dynamic symbol JNI_OnLoad is entry **0x651ca8**, size6704. Its selected
class/registration blocks lie within that symbol range. At0x6534bc..0x6534d0,
class lookup calls findClassAndTurnToGlobalRef with literal0x1122cd:
`com/whatsapp/wamsys/JniBridge`, then stores its returned class at **0xc44bc0**.
A separate guard protects this selected class initialization; the entire loader
and failure cleanup are not claimed complete.

At0x652824..0x652844 the initializer copies that class into **0xc44bd0**, then
registers49 native methods from table0xc078b0 using the same class name.
The29-entry static-method lookup loop uses parallel tables:

| Item | Address / calculation |
| --- | --- |
| names | 0xc076e0 + index*8 |
| signatures | 0xc077c8 + index*8 |
| method-ID destination | 0xc46310 + index*8 + 0x4e0 |

The loop passes the global class from0xc44bc0, selected name and signature to
getStaticMethodId. **Index0** resolves `jnidispatchIO` with exact signature
`(ILjava/lang/Object;)J`, storing its returned method ID into **0xc467f0**.
All29 names/signatures and slot calculations are retained in cache-boundaries.json.
This is an indexed store: a scan limited to direct stores at offset0x7f0 could
not establish it. The earlier negative scan was bounded, not proof no writer
existed.

Outbound bridge **0x6ea324** loads class0xc44bd0 and method ID0xc467f0, preserves
the supplied selector as w3 and object as x4, then calls callStaticLongMethod.
The class and method now statically bind to the
[Java credential producer](../2.26.43.72-metrics-credential-iq/README.md):
selector7 uses byte[] to build its privatestats/sign_credential request. The
native data bridge0x6ecd60 converts MCFData to byte[] and interprets the returned
long as nonzero/zero; no cryptographic or server result is implied by that bit.

## Partial lookup is not initialization success

The loop stores each result, including null, before testing it. A null result
stops iteration; previous slots are not cleared by the selected loop. After
iteration, byte **0xc4435c** is set from whether all29 lookups completed, then
its guard is released. A nonnull first method slot can coexist with this byte
being zero if a later lookup fails. Neither nonnull slot nor completed guard
release alone proves full initialization or successful JNI_OnLoad return.
Full consumers of this aggregate flag and whole-loader error policy remain
**unknown** here. Do not add a guessed Rust readiness check from this byte.

Five native-loop fixtures cover all-success and synthetic failures at indices
0,1,12,28. They verify lookup name/signature/class, each visited destination,
first-slot value, early stopping, copied outbound class, aggregate flag and
guard-release pointer. Registration, getStaticMethodId and guard release are
controlled adapters; they never call a JVM or register functions. Synthetic
method IDs are inert pointers in emulated memory, not usable credentials or
runtime handles. The class-initialization block itself is static evidence and
is not executed in these fixtures.

## Reproduce and limits

`python inspect-cache.py RECONSTRUCTED_LIBWHATSAPP.so` must reproduce
cache-boundaries.json. The script reuses the digest-pinned ELF resolver,
resolves the ELF symbol, relative string tables and mapped byte-window hashes.
Rizin exports use `pD SIZE @ VA` for windows.json; only selected portions of
JNI_OnLoad are retained, not its whole semantic scope.

With Unicorn2.1.4, `python check-cache.py RECONSTRUCTED_LIBWHATSAPP.so` must
reproduce cache-results.json. Execution begins at0x652824 with explicit
synthetic environment/class/slot registers and stops at0x651d04. Each case has
100000-instruction and one-second caps. Unexpected imports, unsupported loader
relocations or target interrupts fail closed. No real I/O adapter exists.
Checksums cover all other files in this bundle.

**Inferred:** this binding connects the selected native refresh request to the
already traced Java IQ producer without relying only on matching selector names.
**Observed live:** none. **Unknown:** runtime availability, whole-loader recovery,
credential blinding/verification, generation fencing and deployment necessity.
No Rust client runtime or mandatory telemetry added; no restriction guarantee.
