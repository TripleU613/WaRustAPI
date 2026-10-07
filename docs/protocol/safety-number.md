# Safety-number generation and verification boundaries

New evidence here is **confirmed/static** for APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
It does not establish the current account's selected scheme or human verification.
The existing `peer-identity` / `identity-conflicts` SHA-256 fingerprints remain
diagnostic values, **not WhatsApp safety numbers**.

## Java legacy path

The earlier description of both `0jc.A0U` and `A0V` as native wrappers is
**refuted**: `A0V` invokes Java `CjU`; `A0U` and `A0T` use wamsys dispatch.
`0jc.A0V` constructs a local key list from supplied companion keys and adds the
local primary identity (`000e..0052`). Its remote list comes from the supplied
remote keys (`005c..008e`); it does not separately add a remote primary here.
It invokes `CjU.A00` for each supplied identifier (`009e..00ae`). Missing or
incomplete device inventories cannot be replaced by one arbitrarily selected
PN/LID pin and still claimed to reproduce this input set.

**Key serialization and ordering:** `B7E.A00` (`0000..0028`) concatenates its
one-byte identity type and raw public key. `CjU.A02` serializes each identity,
sorts with `DO3.compare` (`003c..004c`) and concatenates the resulting arrays
(`0056..0088`). `DO3.compare` masks each byte with `0xff` before comparison
(`0018..0028`), so ordering is unsigned lexicographic, then length. No
key deduplication is present in this method. The supplied producer's inventory
completeness and deduplication rules remain untraced.

**Identifiers:** `JhJ.A0Z`'s Java branch (`0140..016c`) passes both strings
through `HNy.A00`. Its exact pattern is:

```text
^([17]|2[07]|3[0123469]|4[013456789]|5[12345678]|6[0123456]|8[1246]|9[0123458]|\d{3})\d*?(\d{4,6})$
```

`HNy.A00` (`0028..0044`) concatenates capture groups 1 and 2 on a match;
otherwise it returns the original string. `1xp.A0S` (`000a..0016`) confirms
concatenation. This retains the matched country-prefix branch and final four
to six digits; it is not ordinary E.164 validation and does not strip a leading
plus. Hash input uses UTF-8: `6an.A1b` reads `07l.A05`, initialized from `UTF-8`
in `07l.<clinit>` (`0000..0016`). Raw account phone digits must not silently be
substituted for this producer's normalized identifier.

**Hash and decimal groups:** `CjU.A00` (`0000..0064`) sets iterations to
`0x1450` = 5200, selects SHA-512, and constructs a seed with two zero bytes,
concatenated sorted serialized keys, then UTF-8 identifier. `6aj.A1V`
(`0000..000a`) establishes the last two seed array slots; `1Lz.A05` concatenates
them in array order. Every iteration hashes previous state followed by the same
serialized key bytes. In pseudocode, for one side:

```text
K = concatenate(unsigned_lexicographically_sorted(serialized_identity_keys))
state = 0x0000 || K || UTF8(identifier_from_the_caller)
repeat 5200 times:
    state = SHA512(state || K)
side = concatenate(format_decimal_5_digits(unsigned_big_endian_40_bits(state[i:i+5]) % 100000)
                   for i in [0, 5, 10, 15, 20, 25])
```

`CjU.A01` (`0000..0064`) masks signed Java bytes, shifts and joins them into
five-byte unsigned values and reduces modulo `0x186a0` = 100000. Its `%05d`
format at `0074..007e` delegates through `3c6.A11` to the **default-locale**
`String.format`, not an explicitly selected English locale. Numeric groups are
confirmed; Latin-digit rendering depends on locale. `Ayp.A19` and `1xj.A16`
confirm that all six groups are appended in their stated order. `KKq.A00`
(`0000..003a`) lexically orders the two side strings and concatenates them,
producing 60 formatted digits in a locale with five-digit decimal output.

DEX instructions previously rendered as `??` in this trace are documented by
[Android's instruction table](https://source.android.com/docs/core/runtime/dalvik-bytecode):
`bf` remainder-long/2addr, `c0` and-long/2addr, `c1` or-long/2addr and
`c3` shift-left-long/2addr. The developer disassembler now names these and
identity wide/boolean/byte field reads; it preserves their existing operands
and byte extents. Literal byte fixtures check both groups of instructions.

**Legacy QR content differs from a digest assumption:** `0jc.A0V` calls
`CjU.A02` for both sides (`00c2..00d2`), then builds `M76` version zero
(`00ec..00fc`). `BTe.content` receives the concatenated serialized keys, not
SHA-512 output; `identifier` receives UTF-8 identifier bytes. `M76.dynamicMethod`
and `BTe.dynamicMethod`, decoded independently with `apk-proto-fields`, give
`version=1`, `localFingerprint=2`, `remoteFingerprint=3`; nested `content=1`,
`identifier=2`. The assignments are at `0130..016c` / `018c..019c` locally and
`01c6..022a` remotely. QR comparison semantics are still untraced.

## Native schemes and UI selection

`JhJ.A0Z` invokes Java `A0V` at `0166..016c`, native V4-labelled `A0U` at
`028e..0294`, and native `A0T` at `0354..035a`. Selection uses JID predicates,
`26B.A00`, feature IDs `0x1d2c` and `0x1da3`, mappings and additional inputs;
their runtime values and complete producers are not established here. These
branches must not be selected from a guessed default or a desired result.

`0jc.A0U` converts both key sets (`0014..003c`) and passes extra identifier
inputs through `WESIndividualFingerprintParamsCreate` (`006e..0092`). It
uses dispatch `jvidispatchOOOO` with selector 2 at `00b2..00ba`, processes the
result through selector 4 at `00da..00e2`, reads protobuf bytes through model
field 6 (`010c..0122`) and two display strings through model fields 7/8
(`014c..017c`). These selectors are not the fingerprint algorithm. Feature
`0x1da3` gates supplied booleans versus sentinel integers of -1
(`004c..0062`, `0076..007e`). Their meanings are not inferred here.

**Native packaging finding:** the APK's `lib/arm64-v8a/libs.so` is an archive-container
ELF (13,615,136 bytes; SHA-256
`24dfc7c0b9c3a13d1a9bd875304035a9f5ca6aa6f8ff904aef6ec1cfe3418c6b`).
`readelf` reports `_superpack_archive_start=0xc870`, end `0xd07430` and size
`0xcfabc0`. The archive occupies `.data`, file offset `0x870`; `.text` is only
`0x50` bytes. Exported symbols and ordinary strings contain no fingerprint
implementation. That is an **outer packed-container limitation**, not evidence
that native fingerprint code is absent. `assets/native_deps.txt` names
`superpack`, `whatsappmerged` and `whatsapp`, but does not identify which member
owns these routines. At that checkpoint, member reconstruction and native algorithms remained
unknown. Subsequent reconstruction recovered 79 unique ELF members; the four
V3/V4 anchors are in `libwhatsapp.so`. See [native reconstruction](native-unpacking.md)
for pinned hashes, reproducible extraction and negative results.

No account verification bit, trust policy, session or identity was changed;
no server query or chat was sent. Human safety-number verification and
cross-device verification-state synchronization remain unfinished.

## Reproduction

Existing offline tooling is sufficient to reproduce the Java trace:

```sh
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/CjU;'
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0jc;' --method A0V
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/JhJ;' --method A0Z
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/HNy;' --method A00
cargo run -p xtask -- apk-proto-fields ~/Downloads/WhatsApp.apk --class 'LX/M76;'
cargo run -p xtask -- apk-proto-fields ~/Downloads/WhatsApp.apk --class 'LX/BTe;'
```

The other helper classes and method names are identified above. Private scoped
dumps are `lab/apk-2.26.37.71/safety-number-*.smali`; native extraction is
`lab/apk-2.26.37.71/native/arm64-libs.so`. They are not committed.

Validation: 26 xtask tests (including the two new opcode fixtures), 529
workspace/all-feature tests, 223 default-feature tests, strict all-target/all-feature
Clippy, warning-free rustdoc, regenerated CLI reference, formatting and repository
hygiene passed. Rebuilt `apk-smali` renders the actual `CjU.A01` mask/shift/or/
remainder instructions at their original byte extents, with no `??` remaining in
that method. These are static-tool checks, not a live safety-number comparison.

## Reconstructed native hash and display paths

**Confirmed/static, APK 2.26.37.71:** the reconstructed ARM64 `libwhatsapp.so`
has SHA-256 `f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a`.
Addresses below are ELF virtual addresses, not DEX offsets or bare XZ offsets.
The V3/V4 create anchor routines are `0x7132b4..0x713620` and
`0x7145d0..0x71493c` (end-exclusive). Both call the same hash helper at
`0x713938..0x7139e4` (`0x7134cc`, `0x71476c`).

The helper creates a SHA-512 context, appends its two data arguments, finishes
into a 64-byte output, then repeats output-plus-second-argument another 5199
times. `0x713994` loads `0x144f` (5199); `0x7139bc..0x7139c0` decrements
and loops. SHA wrappers `0x711828..0x71199c` resolve to the named imports
`mbedtls_sha512_init`, `starts` (second argument zero), `update` and `finish`.
The finish wrapper `0x711914..0x71194c` explicitly initializes/restarts the
context after finishing. It does not keep a cumulative hash state across rounds:

```text
H(seed, K):
    state = SHA512(seed || K)
    repeat 5199 times:
        state = SHA512(state || K)
    return state  # 64 bytes
```

**Confirmed/static, create inputs:** V3/V4 copy their supplied key array,
sort it through a function pointer at relocation slot `0xd1c2d0`, concatenate
each key object's `+0x28` MCF data and call `H(K, K)`. Sorting occurs at
`0x713408..0x71341c` / `0x7146a8..0x7146bc`; concatenation at
`0x713450..0x71348c` / `0x7146f0..0x71472c`; the copied seed is constructed
at `0x713498..0x7134c0` / `0x714738..0x714760`.
The relocation's addend is `0x712ef0`. That comparator reads each object's
`+0x20` MCF data, requires equal lengths (otherwise abort), calls `memcmp`
and returns -1/0/1 (`0x712ef0..0x712f84`). Sorting is therefore by that
byte representation, not by the later digest or identifier. Exact constructors
linking these two key representations remain untraced here; do not claim
arbitrary raw bytes reproduce a complete APK identity-key object. No
concatenation-time deduplication is present in these traced loops.

**Confirmed/static, validation:** V4 requires a non-null supplied LID object;
its refused branch resolves to `no addressable lid identifier provided`
(`0x714634`, `0x7147b0..0x7147ec`, string `0x12cc88`). V3 rejects when both
supplied PN and LID objects are null (`0x713318..0x71339c`, string `0x12cc65`).
Both require a nonempty key array and a hosted-state integer in {-1,0,1}; the
unsigned `(state + 1) >= 3` rejection is at `0x7133d8..0x7133e0` /
`0x714678..0x714680`. Strings resolve to `missing identity keys` at `0xb3fd4`
and `unknown hosted state: %d` at `0x14d93f`. PN objects also take a stable-PN
conversion branch whose failure logs `Stable PN JID creation failed` at
`0x128cd3`; its JNI-backed producer `0x6b7a04` is not implemented by our slice
checker. These conditions are APK input validation, not server requirements.

**Confirmed/static, separate display seed:** V3 display routine
`0x7139e4..0x713cf8` reads individual params data at `+0x18` and an ASCII
identifier at `+0x28`. Its seed is two zero bytes, that data, then the identifier
bytes; `0x713aa0` calls `H(seed, data)`. V3 params construction
`0x718ffc..0x7190dc` places concatenated keys at `+0x18`, their `H(K,K)` digest
at `+0x20`, stable PN identifier at `+0x28`, and retains supplied LID at
`+0x30`. This proves that the identifier-free create digest and identifier-bound
display digest differ. It does not establish all QR field assignments.

V4 display routine `0x714c70..0x714f98` similarly uses concatenated keys from
`+0x18`, but reads `+0x28` through the JNI-backed conversion at `0x6b7a94`
before constructing the identifier-bound seed. V4 params constructor
`0x71916c..0x719264` copies the supplied LID's ASCII identifier into `+0x28`
and stable PN into `+0x30`, in the opposite layout to the V3 fields above.
The exact conversion/normalization behavior is **unknown**. Do not substitute
raw PN/LID digits or reuse Java `HNy.A00` without tracing this producer.

Both display routines group the first 30 digest bytes into six five-byte
unsigned big-endian values, reduce each modulo 100000 and format with literal
`%.5lu` (string VA `0x9ae01`). V3 groups are at
`0x713ac4..0x713c94`; V4 at `0x714d58..0x714f38`. The ARM64 multiply-high
reciprocal uses `0x0a7c5ac471b47843` and subtracts quotient times 100000.
A local arithmetic check matched modulo for 10045 synthetic 40-bit values,
including endpoints, powers of two and seeded random cases. Native formatting
uses the MCF formatting import; Java default-locale behavior is not thereby
established for it.

**Observed/local, bounded control-flow check:**
[`native_fingerprint_check.py`](../../tools/apk-research/native_fingerprint_check.py)
checks the exact reconstructed library hash, maps only the hash routine's page
and emulates that routine under Unicorn. SHA/MCF entry calls are explicitly
host-modeled; any branch outside the routine or six supported entry points
refuses. Two synthetic cases exercise distinct seed/key data and equal seed/key
data, independently comparing the ARM64 iteration/input sequence to `H` above.
Each returns after 72825 instructions, 5200 finishes and 10400 updates, within
one million instructions and ten seconds. Only an actual return PC counts as
success. This validates native control flow with modeled primitives, not the
native SHA implementation, JNI/MCF runtime, complete create/display routines,
real account safety number, QR compatibility or human verification.

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_fingerprint_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so
```

Dependencies are Unicorn and pyelftools from the existing private research
venv. No account store or live connection is used; inputs are built-in synthetic
bytes. Private scoped disassemblies are `native/fingerprint-hash.txt`,
`fingerprint-sha-wrappers.txt`, `fingerprint-key-comparator.txt`,
`fingerprint-v3-display.txt`, `fingerprint-v4-display.txt`,
`fingerprint-v3-params.txt` and `fingerprint-v4-params.txt`, under the APK lab
root. The current scheme selector, key/identifier producers, full QR descriptor
and comparison semantics remain unfinished. These findings do not set a peer's
verified bit or authorize marking an identity verified.

Validation of this follow-up: eleven Python checks passed with both the public
rootfs and pinned library configured (no skipped cases), including the actual
synthetic ARM64 slice and wrong-build refusal. CLI reference regeneration,
workspace formatting and repository hygiene passed; Rust runtime code did not
change. No live safety-number comparison was performed.
