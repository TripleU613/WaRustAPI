# Native KEM dispatch and randomness

2026-10-12. Static ARM64 evidence from APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This closes the JNI dispatch/size boundary, not cryptographic equivalence or
working Rust hybrid sessions. [Retained traces and hashes](native-kem-2.26.37.71/manifest.json)
come from the [previously reconstructed native archive](native-unpacking.md).

## Dispatch is ML-KEM, despite the Java suite label

`libwhatsapp.so` SHA-256
`f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a`
exports `Java_org_whispersystems_libsignal_kem_KEMPublicKey_encapsulateNative`
at `0x622f70`. Its complete 816-byte body is retained in
[jni-encapsulate.txt](native-kem-2.26.37.71/jni-encapsulate.txt).
It selects these branches using the original public array's byte length:

| Public bytes | Dispatch thunk | Imported function | Ciphertext bytes | Shared secret bytes |
| --- | --- | --- | --- | --- |
| 800 | `0x617080` | `OQS_KEM_ml_kem_512_encaps` | 768 | 32 |
| 1184 | `0x617088` | `OQS_KEM_ml_kem_768_encaps` | 1088 | 32 |
| 1568 | `0x617090` | `OQS_KEM_ml_kem_1024_encaps` | 1568 | 32 |

JNI GetArrayLength is the indirect slot at `0x558`; NewByteArray is `0x580`,
GetByteArrayElements `0x5c0`, ReleaseByteArrayElements `0x600` (ARM64 JNIEnv).
Length comparisons at `0x622fe4..0x622ff8` choose the thunk; output allocation
at `0x623070..0x623098` uses the table above. Other lengths take the exception
path at `0x6231c4`. Null public/seed arrays and a seed length other than 32
also take exception paths. This is a JNI length gate, not sufficient KEM
public-key validity or a directory policy decision.

The [dispatch trace](native-kem-2.26.37.71/jni-dispatch.txt) resolves each thunk
to a named ELF PLT import; dynamic relocation entries bind them to the matching
ML-KEM symbol. `libwhatsapp.so` lists `libwhatsappmerged.so` as DT_NEEDED.
The merged library SHA-256 is
`565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa`.
Its ML-KEM-1024 export at `0x2babc0` branches through `0x70a6b8` to
`0x70a9dc`; the [export](native-kem-2.26.37.71/ml-kem-exports.txt),
[dispatch](native-kem-2.26.37.71/ml-kem-dispatch.txt), and
[encapsulator](native-kem-2.26.37.71/ml-kem-encapsulate.txt) ranges retain this chain.
Disassembler labels attached to nearby unnamed code are enclosing ELF symbols,
not proof that these KEM routines are protobuf functions. Follow addresses.

The [Java HKDF label](pq-signature.md) remains
`WhisperText_X25519_SHA-256_CRYSTALS-KYBER-1024`. Do not replace that protocol
string because the native import is named ML-KEM, or choose legacy Kyber solely
from that string. Complete polynomial/encoding/validation equivalence with an
independently implemented ML-KEM primitive remains unverified.

## The Java seed is checked but not consumed

JNI preserves the second array in x22, checks non-null and length 32 at
`0x623054..0x62306c`, then overwrites x22 with the newly allocated ciphertext.
It never obtains the seed's array elements. The call at `0x623118` passes
ciphertext, shared-secret and original public-key pointers in x0/x1/x2.
There is no seed argument or copy from that Java array.

The merged encapsulator allocates a fresh 32-byte buffer and calls its native
randomness dispatcher at `0x70aa28 -> 0x715a7c`, then passes that buffer as x3
to the inner operation at `0x70aa3c -> 0x70a7d0`. The native randomness source,
full inner primitive, error handling and deterministic fixture oracle still
need tracing/validation. Java SHA1PRNG output therefore does **not** establish
deterministic encapsulation input for this build.

[Decapsulation JNI](native-kem-2.26.37.71/jni-decapsulate.txt) selects using
ciphertext sizes 768/1088/1568 and requires secret-key sizes 1632/2400/3168,
respectively; it allocates a 32-byte shared secret. This supports the same
three parameter sets. Valid-size malformed inputs and implicit rejection
behavior still require the underlying primitive, not inference from lengths.

## Reproduce and limits

From the repository root, using the retained decoded libraries and LLVM:

```sh
python tools/apk-research/native_kem_trace.py \
  lab/apk-2.26.37.71/native/qiling-rootfs/decoded \
  docs/protocol/native-kem-2.26.37.71 --check
```

The [checker](../../tools/apk-research/native_kem_trace.py) refuses mismatched
library hashes, checks dispatch/size/randomness instruction anchors, and compares
all seven retained files byte-for-byte. Generation omits `--check`. No native
library is loaded or executed. REA identified the native image; its deep native
provider is unavailable on this host, so these traces use installed llvm-objdump
and readelf. Java REA evidence remains linked separately.

This finding changes the porting choice from an unresolved Kyber-name assumption
to a traced ML-KEM dispatch boundary. No dependency, API behavior, key adoption,
session state or network operation changes. Full hybrid sessions, native fixture
comparison and live interoperability remain open.

Validation: the seven-file trace reproduced exactly; a deliberately wrong-build
library was refused before disassembly. Local links, artifact budgets and diff
whitespace checks passed. Runtime Rust code is unchanged; the prior 971-test
result does not verify this native primitive.

## Subsequent primitive evidence

The [offline native/NIST comparison](native-kem-vectors.md) now matches selected
ML-KEM-1024 key generation, deterministic encapsulation and decapsulation
vectors, including five modified ciphertext cases. This narrows the earlier
primitive-equivalence unknown; it does not close JNI, native RNG, full algorithm
coverage, other parameter sets or hybrid-session integration.
