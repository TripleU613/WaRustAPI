# Native ML-KEM-1024 vector comparison

2026-10-12. The pinned APK's native ML-KEM-1024 primitive agrees byte-for-byte
with **16 selected NIST FIPS203 cases**: three key generations, three deterministic
encapsulations and ten decapsulations. This is a primitive-level comparison,
not WhatsApp session interoperability, FIPS validation or proof for all inputs.

## Independent source and observed outputs

Inputs and expected outputs come from the public
[NIST ACVP-Server corpus](https://github.com/usnistgov/ACVP-Server/tree/975de31eb83d87039ec88934fdc47d8c312b892d/gen-val/json-files),
pinned at commit `975de31eb83d87039ec88934fdc47d8c312b892d`, using the FIPS203
revision (not FIPS203-tr1). [Retained vectors](native-kem-nist-vectors.json)
include source URLs and complete-source SHA-256 values. Three cases are selected
from each ML-KEM-1024 keyGen/encapsulation group; all ten are selected from its
decapsulation group. The library does not generate the expected answers.

[Recorded results](native-kem-nist-results.json) contain native return codes,
output hashes and modeled callback counts. Full expected byte strings remain in
the public vector fixture. Both public/secret key outputs (1568/3168 bytes),
ciphertext/shared secret outputs (1568/32 bytes), and decapsulated secrets
(32 bytes) compare exactly. KeyGen uses NIST d followed by z as the 64-byte
seed; deterministic encapsulation uses the NIST 32-byte m.

NIST's pinned internalProjection classifies the decapsulation inputs as five
valid and five modified ciphertexts. The [retained provenance](native-kem-nist-provenance.json)
records those reasons, test IDs and source hash. Modified ciphertext cases
96/98/101/104/105 return success and the exact expected fallback shared secrets;
this corroborates implicit rejection on these vectors. It does not establish
validity of arbitrary expanded secret keys, all public-key validation paths,
constant-time behavior or errors at JNI/directory boundaries.

## Actual code versus modeled runtime

The [checker](../../tools/apk-research/native_kem_check.py) uses the existing
Unicorn/pyelftools research environment. It hash-checks reconstructed
`libwhatsappmerged.so` against
`565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa`,
loads its ELF segments and relative relocations into isolated emulator memory,
and runs these pinned native entries:

| Operation | Entry | Scope |
| --- | --- | --- |
| KeyGen | `0x70a6d0` | deterministic ML-KEM-1024 keypair inner function |
| Encapsulation | `0x70a7d0` | deterministic inner function called by traced random wrapper |
| Decapsulation | `0x2babe8` | exported ML-KEM-1024 decapsulation thunk |

Polynomial arithmetic and SHA3/SHAKE execute from the library. No Python hash
function supplies crypto outputs. Modeled dependencies are memset, memcpy,
checked memcpy, bounded aligned allocation, allocation lifetime/free and
single-thread pthread_once. The latter executes the actual native initializer
once for each guest control pointer. Each case starts in a fresh guest.

Only pinned crypto/helper code ranges and the six explicit libc callback
addresses are admitted; unexpected branches fail. Each case has a 10-million
instruction / 20-second emulator bound, bounded memory copies and a bounded
fixture heap. Imports are not dynamically loaded or resolved to host execution.
No account, network, Android VM, native randomness, Java seed input or
hybrid Signal state is involved. See the [JNI trace](native-kem.md) for the
separate dispatch and unused-Java-seed finding.

## Reproduce and validation

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_kem_check.py \
  lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so \
  docs/protocol/native-kem-nist-vectors.json --output /tmp/native-kem-results.json
cmp docs/protocol/native-kem-nist-results.json /tmp/native-kem-results.json
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  -m unittest discover -s tools/apk-research -p test_native_kem_check.py
```

The three small harness tests check wrong-build refusal before execution,
expected-output selection independent of JSON field order, output mismatch and
nonzero native return rejection. They model results and do not replace the
16-case native comparison. The complete matrix independently reproduced its
recorded results. Local link/budget and diff-whitespace checks passed.

This closes selected ML-KEM-1024 primitive vectors and guides an independently
maintained Rust primitive choice. No application dependency, admission rule,
saved identity/session, request ID, wait or network behavior changes. Rust KEM
integration, hybrid derivation/records, incoming ciphertext handling and fresh
live interoperability remain unfinished.
