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
byte representation, not by the later digest or identifier. The constructors were untraced at this checkpoint; the follow-up below resolves
full serialized data versus prefix-stripped comparator data. Full upstream
inventory completeness remains unknown. No
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
`0x71916c..0x719264` copies the supplied LID's full ASCII JID string into `+0x28`
and stable PN into `+0x30`, in the opposite layout to the V3 fields above.
The exact conversion/normalization behavior was **unknown** at this checkpoint.
The JNI callback follow-up below resolves it; this is not permission to guess
the current account's input inventory or selected scheme.

**Confirmed/static, display getter and result follow-up (37.71, same pinned
main-library hash):** the shared getters `0x6c1e0c..0x6c1e2c` and
`0x6c1e4c..0x6c1e6c` load `+0x18` and `+0x28` respectively. They perform no
class check and abort on a null object. These offsets alone do not identify an
object's type: the create routines also use them on input parameter objects.
For display, V4 result assembly reads the two completed individual objects
from the combined object's `+0x20`/`+0x28` at `0x7149f8..0x714a0c`, then calls
the display routine separately. Each object's `+0x18` is therefore K and its
`+0x28` is the copied full LID JID string. `0x714cb4..0x714cc0` converts that
string through selector 2 before hashing the parsed LID **user**. The native
adapter `0x6b73fc..0x6b7458` reads MCFASCIIStringGetCString and invokes JNI
table slot `+0x538` (NewStringUTF); it does not reinterpret the ASCII object
as a native JID. The [JNI specification](https://docs.oracle.com/en/java/javase/24/docs/specs/jni/functions.html#newstringutf)
assigns NewStringUTF index 167; `167 * 8 = 0x538` for ARM64 pointers.
QR tag 3 retains the full string, while decimal display uses
the parsed user. Hashing the full QR string directly would change the number.

The shared result constructor `0x715e98..0x715f34` copies QR bytes into
`+0x18`, local display into `+0x20`, and remote display into `+0x28`; it
preserves null displays and does not sort or join them. Java `0jc.A0U
0138..019a` requests model fields 7/8 through jvidispatchOIO selector 2,
requires both non-null before constructing KKq, and retains the QR model even
when the display model is null. KKq supplies the previously traced lexical
joining step. The native field-7/8 dispatch mapping remains **unknown**;
the constructor layout alone does not prove it. Exact JhM/JhN class dumps
contain only toString methods; they are comparison-result types, not display
assemblers. No account fingerprint inventory or verification bit was inferred.

Reproduce the native portions with `llvm-objdump -d` on the pinned main
library, using ranges `--start-address=0x6c1d84 --stop-address=0x6c1e6c`,
`0x71493c..0x714a60`, `0x6b73fc..0x6b745c`, and `0x715e98..0x715f34`.
The existing apk-smali utility reproduces `LX/0jc; --method A0U` and the
negative exact-class inspections `LX/JhM;` / `LX/JhN;`. Private getter/result
dumps are retained as `native/fingerprint-shared-getters.txt` and
`native/fingerprint-result-constructor.txt`; JhM/JhN outputs are retained too.

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

## Native-to-Java identifier callback and dispatch mapping

**Confirmed/static, same APK and reconstructed-library hashes as above:**
`0x6b7a04` (stable PN) and `0x6b7a94` (stable LID) both call the JNI bridge
`0x6c0cf4`, with selectors 1 and 2 respectively. The former converts a native
JID object's `+0x20` MCF string to Java (`0x6b777c..0x6b77cc`); the latter
converts its supplied MCF ASCII string (`0x6b73fc..0x6b7430`). The JNI call
loads class slot `0xd36ad0` and method slot `0xd36ae8` and invokes a static
object method (`0x6c0d20..0x6c0d30`).

Registration initializer `0x6bbd74..0x6bbea8` resolves the class through pointer
slot `0xd1c250` -> `0xd262b0` -> string `0x115430`,
`com/whatsapp/wamsys/JniBridge`. Its method-name and signature arrays begin at
`0xd06b28` and `0xd06c28`; relocation entries at `+0x10` give
`jnidispatchOO` (`0x164104`) and
`(ILjava/lang/Object;)Ljava/lang/Object;` (`0x169d58`). The initializer writes
that method ID to `0xd36ae8`. This distinguishes the Java callback
**jnidispatchOO** from the Java-to-native **jvidispatchOO** method; selector
numbers must not be interpreted against the wrong dispatch direction.

| Callback selector | Java 37.71 branch | Result |
| --- | --- | --- |
| 1 | `JniBridge.jnidispatchOO`, `000a..004a` | Parses a UserJid; type 0 returns `HNy.A00(jid.user)`, type 18 returns `jid.user`; other types return null |
| 2 | same method, `038e..03e8` | Parses a UserJid and requires type 18; returns `jid.user`; wrong type or caught invalid-JID failure returns null |

`0DE.A0e` checks JID type zero (`0004..0014`);
`PhoneUserJid.getType` returns zero (`0000..0004`). `0DE.A0a` checks type
`0x12` / 18 (`0004..0018`); the actual obfuscated `0ZY.getType` returns
18 (`0000..0006`). `0D8.A01` calls `0D1.A00`, requires the parsed object to
be a UserJid and otherwise throws `08j` (`0000..002a`). The parser splits at
the final `@` and handles domain/type branches. These callbacks operate on
parsed JID user fields, not an unvalidated string with an arbitrary suffix
removed. The native return conversion `0x6b72ac..0x6b73fc` accepts null,
checks every Java UTF-16 character is <=127 and constructs MCF ASCII data;
non-ASCII output returns null. JNI exceptions are asserted separately.

This **resolves** the normalization uncertainty recorded above: ordinary PN
stable identifiers use the already traced `HNy.A00` rule; the stable LID
callback returns the parsed type-18 user's bytes, with no HNy shortening in
that branch. It does not establish the current UI's selected input set or all
JID parser/device/hosted variants. Exact-class/method-filter searches for
`UserJid.getType`, `UserLidJid.getType` and `LidUserJid.getType` returned no
matching method output; that was not evidence that a LID type was missing.
`0ZY.getType` supplies the actual type-18 implementation in this build.

**Confirmed/static, key representation follow-up:** native key constructor
`0x712c00..0x712cf4` requires 33 serialized bytes, copies all 33 into `+0x28`,
and the final 32 into `+0x20` (`0x712c68..0x712cac`). The data-input constructor
`0x712cf4..0x712dc0` validates through `0xc9aa10`, copies the full input into
`+0x28` and copies input beginning at byte 1 into `+0x20` on success
(`0x712d60..0x712d94`). The raw-data adapter
`0x712dc0..0x712e58` prepends the literal byte at `0x30236d`, **0x05**, before
calling that constructor. Together with the comparator above, the normal
33-byte path sorts by the 32 key bytes and concatenates the 33-byte
prefix-plus-key representation. The validation helper's complete internals
remain untraced; a constructor call with arbitrary malformed input is not
claimed equivalent to a valid APK key.

**Confirmed/static, Java-to-native fingerprint path:** the 49-entry native
registration table begins at `0xd06d28`. Entry 16 (`0xd06ea8`) registers
`jvidispatchOOOO` at `0x6bdca0`; entry 7 (`0xd06dd0`) registers
`jvidispatchOO` at `0x6bcab0`. `0jc.A0U`'s selector 2 goes through
`0x6bdce0..0x6bdd1c` -> `0x6daeb4..0x6daeec` -> `0x714520` -> the two V4
individual create calls `0x714540` / `0x714554`. This connects the Java
V4-labelled wrapper to the reconstructed V4 anchor, rather than inferring
that connection from a string name alone.

Its result selector 4 indexes the byte jump table at `0x300e22`:
`0x6bcae8 + 4*table[4] = 0x6bcb20`. That branch sets the serialization argument
to zero before reaching `0x71493c` at `0x6bcc28`. The native V4 serializer
passes that argument to the combined fingerprint's version field and both
individual serializers (`0x714960..0x714998`). Hence this traced Java path
serializes combined version **0**, despite being called a V4 fingerprint.
Selector 5 instead targets `0x6bcc10`, sets the serialization argument to one
and calls the same V4 serializer. Its complete Java caller/selection conditions
and individual protobuf fields remain unfinished. Do not equate the algorithm
label V4 with QR version 4.

Reproduce using the existing offline CLI and native disassembler:

```sh
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk \
  --class 'Lcom/whatsapp/wamsys/JniBridge;' --method jnidispatchOO
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk \
  --class 'LX/0DE;' --method A0a
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk \
  --class 'LX/0ZY;' --method getType
llvm-objdump -d --start-address=0x6bbd74 --stop-address=0x6bbea8 \
  lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so
llvm-objdump -d --start-address=0x712c00 --stop-address=0x712e58 \
  lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so
```

Private `safety-number-jni-callback.smali`, `safety-number-pn-predicate.smali`,
`safety-number-lid-predicate.smali`, `safety-number-pn-type.smali`,
`safety-number-lid-obfuscated-type.smali`, `safety-number-user-parser.smali`,
`safety-number-jid-parser.smali` and `safety-number-jid-types.smali` retain
scoped DEX output. Negative type-filter outputs and native bridge/key
constructor dumps are also retained locally. No native JNI callback was run,
account state opened, verified bit set, or live WhatsApp request sent.

Validation of the callback follow-up: actual APK disassembly and ELF
relocations/jump-table bytes establish the mappings above. Eleven existing
native tooling checks passed with both fixtures configured; they cover the
hash slice and runner guards, not execution of the newly traced JNI callbacks
or QR serialization. CLI reference regeneration and repository hygiene passed.
This change records static evidence; it changes no Rust client runtime policy.

## Native QR schema, serialization and V4 comparison

**Confirmed/static, APK 2.26.37.71:** reconstructed `libwhatsappmerged.so`
SHA-256 `565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa`
exports the fingerprint protobuf-C initializer/pack/unpack functions at
`0x2d3db0..0x2d3e54`. Individual initializer template `0x740ac8` points to
message descriptor `0x740b60`; combined template `0x740bd8` and unpack code
point to descriptor `0x740c08`. Relative ELF relocations resolve names and
nested descriptors. Both message descriptor magic values are `0x28aaeef9`;
the hosted-state enum magic is `0x114315af`.

The ARM64 layout matches the upstream
[protobuf-C v1.5.2 descriptor declarations](https://github.com/protobuf-c/protobuf-c/blob/v1.5.2/protobuf-c/protobuf-c.h):
field metadata names each tag, type, presence selector and native structure
offset. This structural match does **not** establish the APK's protobuf-C
release version. The extracted compact [schema artifact](fingerprint-native-schema.json)
records every native field/descriptor address, offset and enum value. All
fields below are optional in the descriptor; runtime comparison imposes
additional requirements.

| Message | Tag | Name | Type | Native presence/value offsets |
| --- | ---: | --- | --- | --- |
| FingerprintData | 1 | public_key | bytes | `0x18 / 0x20` |
| FingerprintData | 2 | pn_identifier | bytes | `0x30 / 0x38` |
| FingerprintData | 3 | lid_identifier | bytes | `0x48 / 0x50` |
| FingerprintData | 4 | username_identifier | bytes | `0x60 / 0x68` |
| FingerprintData | 5 | hosted_state | enum | `0x78 / 0x7c` |
| FingerprintData | 6 | hashed_public_key | bytes | `0x80 / 0x88` |
| CombinedFingerprint | 1 | version | uint32 | `0x18 / 0x1c` |
| CombinedFingerprint | 2 | local_fingerprint | FingerprintData | pointer `0x20` |
| CombinedFingerprint | 3 | remote_fingerprint | FingerprintData | pointer `0x28` |

`whatsapp.HostedState` descriptor `0x740f58` has exactly two values, **E2EE=0**
and **HOSTED=1**, in value table `0x740f08`. Missing/unknown state is represented
by the params sentinel -1, not a third declared enum value. Individual native
message size is 152 bytes; combined size is 48 bytes. Bytes fields are explicit
length/pointer pairs, not NUL-terminated protobuf strings.

**Confirmed/static, serializers in libwhatsapp.so:** V3
`0x713cf8..0x713e00` and V4 `0x714f98..0x7150ac` select the key field from
the combined version argument. Zero copies params `+0x18` into **public_key**;
nonzero copies params `+0x20` into **hashed_public_key**. These are respectively
the concatenated serialized key inventory and its previously traced `H(K,K)`
64-byte digest. Version is validated as 0/1 on the receive/comparison wrapper.
Thus tag 6 is not the identifier-bound digest used for decimal display.

V3 writes params `+0x28` to pn_identifier and the retained LID object's `+0x20`
string to lid_identifier; V4 writes params `+0x28` to lid_identifier and
`+0x30` to pn_identifier. Neither serializer itself invokes the normalization
callback while writing these fields. In particular, do not replace LID QR
bytes with display-identifier bytes merely because the descriptor says
identifier. The field receives the params string as supplied; the display
path separately parses it through Java. Both serialize username from params
`+0x38`, and include hosted_state only for 0 or 1 (`0x713dc4..0x713dd8`,
`0x715070..0x715084`). V4 has an additional local-side condition: if its third
argument has bit 0 set and username_identifier is present, pn_identifier is
omitted (`0x715050..0x71506c`). Combined V4 passes third argument 1 for local
and 0 for remote (`0x714974..0x714998`). This omission is deliberate APK
behavior, not proof of missing PN inventory.

**Confirmed/static, V4 comparison wrapper `0x714a60..0x714c70`:** it unpacks
the combined payload, requires a present version <=1, both individual messages
and both lid_identifier presence bits. At `0x714b60..0x714bfc`, it matches the
known local LID against the scanned sides and swaps orientation when required;
a QR scan's local side must not always be treated as our local side. It invokes
`0x7150ac..0x715264` to convert each scanned individual. That converter requires
lid_identifier and the version-selected key field. Optional identifiers are
ASCII-decoded; invalid LID/PN/username conversion returns negative status.
Known hosted state must be 0/1; absent state becomes -1. The primitive and orientation-dependent remapping follow-up below resolves
ASCII acceptance and native statuses; error/status-to-UI mappings remain unknown.

**Confirmed/static, V4 comparison `0x7152fc..0x7155d8`:**

- A remote LID comparison through `0x715264` must match; otherwise -12.
- Version zero compares both sides' public_key data through `0x714358`;
  version one compares hashed_public_key through `0x7143b0`. Local mismatch
  returns -14 and remote mismatch -15.
- For each side, both the known and scanned individual must have at least one
  of PN or username. Missing both on the local pairing returns -39; missing
  both on the remote pairing returns -41. A matching LID/key alone is therefore
  insufficient on this path.
- When both PN values are present, `0x7155d8` uses MCFASCIIStringEqual;
  inequality returns -17 locally / -19 remotely. A missing PN is handled
  separately, consistent with username-bearing local QR omission above.
- Username helper `0x7155fc..0x715688` distinguishes absent/present pairs and
  calls MCFStringCompare with literal flag 1 when both exist. The follow-up below traces flag 1 to the ASCII `strncasecmp` path;
  ordinary Rust case-sensitive equality is not equivalent to that call. Different/missing username pairings return distinct statuses
  (-24/-26/-28 locally, -29/-31/-33 remotely).
- Known scanned hosted state is compared with known params state. A mismatch
  returns -34 locally or -36 remotely. If both scanned values are absent,
  comparison logs the legacy-state case and can return zero; unknown state is
  not a fabricated enum value.

The data equality helper `0x71f060..0x71f108` checks lengths, then calls
`0xc9bbfc`; the follow-up below traces and exercises its byte-comparison implementation.
These native return values are internal comparison results, not HTTP/server
reasons. The follow-up below resolves the native result remapper `0x7152b8`; the Java
action/verified-state commit remains unfinished. Do not mark a peer verified merely by implementing
these partial checks or treating an unknown negative code as success.

[`native_schema_check.py`](../../tools/apk-research/native_schema_check.py)
requires the exact merged-library hash and pyelftools, reads descriptors without
executing any native library, and prints the compact JSON artifact above.
It validates ARM64 ELF layout, file-backed reads, descriptor magic, known
optional types, bounded counts, sorted tags and structure offsets. Wrong/missing
builds refuse before producing any schema. No QR bytes, key bytes, phone/LID
values, account state or network requests are inputs to this command.

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_schema_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so
```

Private scoped dumps `fingerprint-v3-qr-serialize.txt`,
`fingerprint-v4-qr-serialize.txt`, `fingerprint-v4-qr-parse.txt`,
`fingerprint-v4-orientation.txt`, `fingerprint-data-equality.txt` and
`fingerprint-native-protobuf-init.txt` retain the source instructions under
the native APK lab root. The schema artifact contains only protocol metadata.
Fourteen Python tooling checks passed with all three fixture variables set,
including actual descriptor extraction and missing/wrong-build refusals. This
is static evidence and offline tooling validation, not a live scan/verification.

The committed schema regenerated unchanged from the pinned ELF in a separate
invocation. CLI reference regeneration/test and repository hygiene passed;
Rust runtime code did not change. These checks do not execute the QR comparator.


## Comparison primitives and orientation-dependent statuses

**Confirmed/static, APK 2.26.37.71**, reconstructed `libwhatsapp.so` SHA-256
`f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a` and
`libwhatsappmerged.so` SHA-256
`565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa`.
Addresses below are ELF virtual addresses, not file offsets. These findings
refine the preceding comparison trace; no account or verified bit changed.

The QR byte-to-identifier bridge `libwhatsapp.so 0x71d5dc..0x71d60c` uses
MCFInt32FromSizeT and MCFASCIIStringCreate. In the merged library, that creator
`0x5317b4..0x531818` invokes validator `0x531200..0x531264`. The validator
accepts zero length, but every nonempty byte must be **1..127**: it loads signed
bytes and rejects values <=0. Embedded NUL and bytes 0x80..0xff are rejected.
A negative signed length or null pointer with nonzero length aborts; invalid
content makes the creator return null. This supersedes the earlier incomplete
UTF-16 <=127 observation: the final creator still rejects embedded zero.
The primitive's empty-string acceptance does not prove that an empty LID/PN
is semantically valid in the complete application.

MCFASCIIStringEqual `merged 0x5316ac..0x5316f4` checks lengths at +0x18, then
calls memcmp over pointers at +0x20. Thus the PN comparator is byte equality,
not username comparison. Username helper `main 0x7155fc..0x715688` converts
MCFASCIIString to MCFString and calls MCFStringCompare with flag 1.
MCFASCIIStringCreateMCFString `merged 0x53172c..0x531764` supplies encoding
0x600 to MCFStringCreateWithBytes `0x538260`. Its constructor scans those
nonnegative bytes at `0x5383a8..0x5383c8`, enters the ASCII branch at
`0x538340`, sets w27=1 and stores that representation flag at +0x1a via
`0x538474`. MCFStringCompare `0x538a50..0x538ac0` masks flag bit 0 and compares
only the shorter length through helper `0x538ac0..0x538c34`, then compares
lengths if the prefixes match. In the two-ASCII branch `0x538b18..0x538b60`,
flag 1 selects GOT 0x759d90; its R_AARCH64_GLOB_DAT relocation names
**strncasecmp**. Flag zero selects GOT 0x75a1a8, **strncmp**. This proves the
actual called comparison, not locale-independent Unicode case folding.
**Unknown:** the Android libc implementation/locale dependency and non-ASCII
helper behavior; neither is substituted with invented normalization.

MCF-data equality `main 0x71f060..0x71f108` first compares lengths, accepts
matching empty data, and otherwise calls leaf `0xc9bbfc..0xc9bd20`. That leaf
accumulates XOR differences with OR across 32-byte NEON blocks, 8-byte blocks
and a byte tail. It returns the OR-reduced byte: zero iff all compared bytes
match. Branches in the traced routine depend on length/loop position, with
no early mismatch exit. This statement is local to that routine; it does
not prove constant-time behavior of the complete verification process.

**Observed/offline:** `native_fingerprint_check.py --check bytes` executed
that actual leaf, without modeled callees, on 862 built-in synthetic cases.
Lengths 0, 1, 7, 8, 9, 15, 16, 31, 32, 33, 63, 64, 65, 66 and 4096 exercise
loop boundaries and fingerprint input sizes. Every mismatch position through
66 bytes and selected positions of the long case were tested, with pointer
shifts 0/1. Buffers end at an unmapped page (one sentinel byte for shift 1).
All results equal OR(XOR(left[i], right[i])); all equal/mismatch cases of the
same length/shift execute the same instruction count. Each case must return
within ten seconds/one million instructions. This executes only a leaf on
synthetic data, not the whole comparator, a human scan or a live client.

**Confirmed/static:** V4 wrapper `main 0x714a60..0x714c70` uses the known
local LID to choose orientation. If it matches the scanned local side,
`0x714b94` calls the comparator and `0x714b98` remaps its result through
`0x7152b8`. If only the scanned remote side matches, `0x714bec..0x714bfc`
swaps the scanned arguments, then stores the comparator result directly at
`0x714c00`, without remapping. The first path takes priority if both match.
Neither match yields -13. Early unpack/version/field/conversion failures also
have their own return paths. Consequently, the remapper is **not** a generic
conversion to apply to every negative code.

The remapper `0x7152b8..0x7152fc` accepts only indices `code+41 < 42` whose
bit is set in mask 0x0201bd5ab5a5, then reads a signed 32-bit result from table
0x302404. Other inputs abort. Its admitted mappings are:

| Comparator code | Remapped code |
| --- | --- |
| -41 | -42 |
| -39 | -40 |
| -36 | -37 |
| -34 | -35 |
| -33 | -33 |
| -31 | -32 |
| -29 | -30 |
| -28 | -28 |
| -26 | -27 |
| -24 | -25 |
| -22 | -22 |
| -21 | -20 |
| -19 | -18 |
| -17 | -16 |
| -15 | -15 |
| -14 | -14 |
| -13 | -13 |
| -12 | -11 |
| -10 | -10 |
| -9 | -9 |
| 0 | 0 |

These are native results, not recognized server response reasons. Zero remains
zero in both orientations; successful native comparison still does not prove
a human verification action or justify committing a verification bit.

**Negative/static:** the guessed `LX/0jc;.A0S` coordinator method was checked
with the existing DEX disassembler. It locks the sender-key store and performs
group sender-key AES/CBC encryption, not fingerprint comparison. The private
`safety-number-compare-coordinator.smali` retains this refuted path; do not
repeat the guess or treat its unrelated success as verified peer state.
A complete disassembly of all 78 methods in `LX/0jc;` also contains no
`jvidispatchIOO` invocation; the comparison caller is outside this class or
behind another wrapper. The private `safety-number-coordinator-all.smali`
retains that search boundary. Absence here is not absence in the APK.
**Confirmed/static:** native registration entry 9 at `main 0xd06e00` names
`jvidispatchIOO`, signature `(ILjava/lang/Object;Ljava/lang/Object;)J`,
function `0x6bcfc8`. The selector-4 branch at `0x6bd084..0x6bd090` reaches
`0x6bd240`; it unwraps the first object through `0x6b6f30`, converts the
second through `0x6b7574`, and calls V4 comparison at `0x6bd268`. The result
is sign-extended to Java long at `0x6bd26c`, then both objects are released.
The scoped `fingerprint-comparison-jni-dispatch.txt` retains this dispatch.
**Unknown:** the Java caller of selector 4, active UI scheme/device-key
inventory and Java action that persists human verification remain untraced.

Private scoped native dumps retain `fingerprint-byte-comparator.txt`,
`fingerprint-ascii-validator.txt`, `fingerprint-ascii-operations.txt`,
`fingerprint-mcf-string-constructor.txt`, `fingerprint-string-comparison.txt`,
`fingerprint-status-remapper.txt` and decoded `fingerprint-status-table.txt`.
Use `llvm-objdump -d --start-address=VA --stop-address=END LIBRARY` with the
ranges above; GOT symbol names come from the merged ELF `.dynsym`/`.rela.dyn`.
The existing synthetic hash mode remains the default and models SHA/MCF;
only the new byte mode executes its entire primitive without modeled calls.

Validation: all 16 offline tooling tests passed with all three pinned fixture
variables set; CLI reference regeneration/test and repository hygiene passed.
No Rust runtime or saved account state changed.


## Java comparison selection and scan result action

**Confirmed/static, APK 2.26.37.71**, APK SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
DEX offsets below are method-relative byte offsets from `xtask apk-smali`.
These findings continue the native JNI mapping above, without changing account
state or implementing an unproven verification commit.

An exact `apk-callers` search for JniBridge.jvidispatchIOO returns eleven
invoke sites in eight unique methods of seven classes. `LX/KXj;.A01` is the
fingerprint selector-4 caller (classes10.dex invoke offset 0x4c0b3c).
Other sites are in `13o.A03` (three), `1BW.AFt`, `1BW.AFu`, `1CP.<init>`,
`1CY.run`, `I6R.A0S` and `Igr.run` (two). Sharing a dispatch name does not make
those other operations fingerprint comparisons; their complete behavior was
not traced. The private `safety-number-comparison-callers.txt` preserves the
complete search, not merely the matching fingerprint site.

`KXj.A01` selects its comparison as follows:

| Input/branch | Method-relative evidence | Comparison/result |
| --- | --- | --- |
| KBk native holder present and feature 0x1d2d true | 000c..002c, 003a..0058 | jvidispatchIOO selector 4, V4, JhN |
| Native holder present and 0x1d2d false | 007c..00c4 | feature 0x1da3 feeds 3c8.A07, OR with 1; jvidispatchIIOO selector 19, JhM |
| Native holder absent | 00c6..030c | Parse M76 and compare in Java, JhL |

On the V4 branch, the native result is converted long-to-int at 0044..0046,
then passed to `LX/1xm;.A1P` at 0046..004c. That helper's entire body
0000..000a returns true **iff its integer argument is zero**. Both that boolean
and the exact integer enter the result constructor `KA1.<init>` at 0052..0058.
The same zero test is used on the other native branch at 00b0..00c4.
The ctor stores the boolean through opcode 0x5c at 0006..000a (currently shown
as opaque by our disassembler) and the status through iput at 000a..000e;
the callers subsequently read A01 with iget-boolean and A00 with iget.
No nonzero native result is converted into success by the traced zero helper.
**Unknown:** complete native selector-19 mapping, the 3c8.A07 option conversion,
and runtime feature values. Do not substitute the V4 branch for all scans.

The legacy branch checks top-level presence bits and version equality before
identifier/content comparisons. Its two possible identifier orientations are
compared with ByteString equality, and content checks delegate to `0ko.A01`
and `0ko.A02`. The complete legacy error/content semantics remain untraced;
the existence of those calls is not a recovered algorithm for that comparator.
`KXj.A02` also filters key-set members through Cbb.A00 when feature 0x1da3 is
true (0004..004a); that predicate's meaning and complete device-key inventory
remain unknown.

An exact caller search for `KXj.A01` returns three sites:

| Caller | Invoke DEX offset | Result handling traced |
| --- | --- | --- |
| IdentityVerificationActivity.A0Z | classes10.dex 0x42999e | NFC payload comparison; A18 displays result |
| Kzw.C0k | classes10.dex 0x5ec35a | Scanner result; LSY.C0n(true) callback |
| LEL.invokeSuspend | classes10.dex 0x693070 | Async comparison; LDi then KaD.A00 |

The NFC method `IdentityVerificationActivity.A0Z` reads the first NDEF record's
payload at 0026..003a and identifier at 0044..0052, parses the record identifier
as US-ASCII UserJid, and invokes the comparison at 0112..0118 using the saved
KBk. A nonnull result with A01=true calls `A18(activity,true)` at 012c..0132.
This establishes the path being NFC, rather than mislabeling it camera QR.

`Kzw.C0k` invokes the same wrapper at 00d2..00d8. At 0108..0124, a nonnull
successful result calls the configured `LSY.C0n(true)`. The async route
`LEL.invokeSuspend` invokes it at 130c..1312 and schedules `LDi` at
1372..138a; `LDi.invokeSuspend` delegates to `KaD.A00` at 0126..013c.
`KaD.A00` reads the success bit at 0006..000e and reaches the same listener
at 007a..0080. Its A01 helper only logs selected legacy error statuses;
it is not a verification transaction.

The activity installs `Kwr(activity, selector)` as KaD's listener at
`IdentityVerificationActivity.onCreate 05a4..05b4`. The activity branch of
`Kwr.C0n`, 00a6..00d0, creates L54 selector 4 with the result boolean and
passes it to `KaD.A03`. `L54.run 0634..0648` calls `A18(activity,boolean)`.
`A18` itself (0000..0130) sets result-view visibility, icons/accessibility
text, animates, focuses the result and schedules A0q after 4000 ms. It has no
direct identity/verification-store write. `KaD.A03` changes scanner views and
attaches a JoG animation listener; the animation completion details remain
untraced. The other Kwr branch shows QrCodeValidationResultBottomSheet with
`is_valid`; it is a distinct activity path, not a peer metadata write.

**Negative/unknown:** the compared success paths traced here do not establish
any `mark_as_verified` or `mark_as_verified_action_seq` update. They show
comparison and result presentation. Delayed A0q/animation callbacks, any
separate human confirmation action, and the complete verification-state writer
call chain remain untraced. Do not infer that such a writer is absent from the
APK, or update TeleWhatsApp's verified bit from these UI paths alone. Existing
identity replacement records the new identity as unverified in its history;
the client has no editable human-verification metadata yet. See
[peer identity evidence](peer-identity.md).

Reproduce the searches and scoped disassembly with existing offline commands:

```sh
target/debug/xtask apk-callers ~/Downloads/WhatsApp.apk \
  --class 'Lcom/whatsapp/wamsys/JniBridge;' --method jvidispatchIOO
target/debug/xtask apk-callers ~/Downloads/WhatsApp.apk \
  --class 'LX/KXj;' --method A01
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk \
  --class 'LX/KXj;' --method A01
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk \
  --class 'Lcom/whatsapp/identity/ui/IdentityVerificationActivity;' --method A18
```

Private source dumps under the 37.71 lab root retain comparison caller lists,
`safety-number-compare-{KXj,13o,I6R,Igr}.smali`, result helpers 1xm/KA1,
scan callers 0/1/2 (activity/Kzw/LEL), KaD/LDi, the Kwr verified listener and
L54 scan-result runnable. The LSY caller search returned four sites; it locates
invocations, not implementations. Repeated exact caller searches reproduced
identical output. No Rust runtime, saved account state or live request changed.

Validation: repeated caller outputs matched byte for byte, CLI reference
regeneration/test passed, and repository hygiene checked 289 local links.
This change records static evidence and an existing research command; it adds
no runtime protocol behavior and claims no live comparison result.


The delayed result callbacks and guarded local verification writer are now
resolved separately in [verification metadata writers](peer-identity.md#verification-metadata-writers-and-rotation-guards-3771).
The writer checks the rendered identity, rechecks serialized identity under
lock, then updates the bit and action sequence; it is distinct from A18 scan
presentation. Complete UI triggers and scheme/device inventory remain unfinished.


The separate local [mark/unmark control](peer-identity.md#local-verification-control-and-captured-identity-3771)
now connects its saved verified bit and rendered identity to the guarded writer.
Marking uses explicit confirmation; unmarking schedules its own branch.
This does not make scan-result presentation a verification metadata commit.

## Explicit-input native display implementation

**Implemented/offline (37.71 evidence, not observed live):**
`whatsapp_api::fingerprint::native_display` now calculates a native 30-digit
side from an already normalized display identifier and raw 32-byte public
identity keys. `native_display_pair` computes both sides and the lexical
60-digit join. The public API is available without transport features.
`whatsapp-cli fingerprint-display --config <private-json>` exposes it without
opening an account store or starting a connection.

The private JSON has exactly `local` and `remote`; each side has
`display_identifier` and `raw_identity_keys_hex` (array of 64-character hex
strings). Unknown fields are rejected. These are **display** identifiers:
stable PN for native V3 or the parsed type-18 LID user for native V4. They are
not full LID QR strings. The caller supplies the chosen scheme's complete
inventory and normalized identifiers; current-account selection, authenticated
key inventory, device membership, PN normalization and QR parsing/comparison
are still unfinished. A number computed from incomplete or wrong inputs is
not evidence of account compatibility or human verification. No verification
metadata is written. This implements the shared native numeric primitive,
not the default-locale legacy Java rendering path.

Keys are sorted by unsigned raw bytes, prefixed with 0x05, concatenated with
duplicates preserved, then hashed from `00 00 || K || identifier` with K
appended in each of 5200 SHA-512 rounds. Six unsigned big-endian five-byte
chunks modulo 100000 are each formatted to five ASCII digits. Pair joining
uses KKq's lexical order, while result local/remote labels preserve orientation.
The result includes `algorithm=apk_2.26.37.71_native_identifier_bound` and
`input_authority=caller_supplied_selection_unverified`; these do not assert
which scheme the account uses. Input identifiers and keys are not printed.
Parse/validation errors omit offending values.

The CLI requires a private regular file no larger than 64 KiB and checks the
read bound again after opening. The library requires a nonempty ASCII identifier
without NUL, at most 256 bytes, and 1..128 keys per side. These are explicit
**local resource/input policies**, not claimed APK/server restrictions.
No profile request-file fallback is used for this separate diagnostic format.

Independent synthetic fixtures are preserved in
[the native display fixture](../../crates/whatsapp-api/fixtures/fingerprint/native-display-3771.json).
Each digest was checked using `native_fingerprint_check.check_flow` against
the pinned main-library hash, with seed `b'\0\0' + K + identifier` and K built
from the fixture's sorted keys. All three actual ARM64 slices returned after
72825 instructions. SHA and MCF calls were host-modeled as previously documented;
key sorting, seed assembly and decimal conversion are grounded in the static
traces, not claimed as whole-native display execution. The duplicate-key fixture
changes the output; side reversal preserves the joined number; hashing a full
LID-like string changes the output rather than being silently normalized.
Rust tests also cover empty/NUL/non-ASCII/oversized identifiers, inventory bounds,
private file permissions, unknown JSON fields, wrong-length/non-hex keys and
error/output redaction. No real account identifiers or keys appear in fixtures.

Reproduce a synthetic operator calculation:

```sh
python - <<'PY'
import json
from pathlib import Path
fixture = json.loads(Path('crates/whatsapp-api/fixtures/fingerprint/native-display-3771.json').read_text())
value = {side: {field: fixture['cases'][index][field]
               for field in ('display_identifier', 'raw_identity_keys_hex')}
         for index, side in enumerate(('local', 'remote'))}
path = Path('lab/fingerprint-display-synthetic.json')
path.parent.mkdir(exist_ok=True)
path.touch(mode=0o600, exist_ok=False)
path.write_text(json.dumps(value))
PY
cargo run --locked -p whatsapp-cli -- fingerprint-display --config lab/fingerprint-display-synthetic.json
```

This prints the fixture-derived number, not a current peer safety number. Reusing
the same file simply repeats the offline calculation; it does not transmit or
rotate account state. Complete scheme/inventory integration and a genuine
peer comparison remain required before implementing human verification.

Validation: 535 workspace/all-feature tests and 229 default-feature tests passed
with zero failures/ignored tests. Strict all-target/all-feature Clippy,
warning-free all-feature library rustdoc, generated CLI reference, formatting,
whitespace and repository hygiene (300 local links) passed. A default-feature
command-level run matched the fixture pair and did not create its supplied
unused account-store path. This was an offline synthetic calculation only;
no current account fingerprint, human verification, or live result is claimed.
