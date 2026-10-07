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
true (0004..004a); its hosted-class meaning and inventory producers are
resolved in the peer-device follow-up below; current runtime inputs remain unknown.

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

## Peer device inventory and scheme inputs (37.71)

**Confirmed/static, same pinned 37.71 APK:** JhJ.A0Z 000c..006a obtains
0bN.A0B(peer), applies KXj.A02, then looks up identities through CSC.A00.
It returns null at 035e..0362 if the resulting map is empty or its size differs
from the selected device set. CSC.A00 0000..0052 iterates DeviceJids, converts
each through B7H.A04, calls 0jc.A0W, and omits missing identities from its map.
Thus a held primary ratchet alone does not establish peer fingerprint inventory
coverage. The map values' B7D.A00 keys are collected at 00a8..00ec;
1xf.A1E 0000..000c constructs an ArrayList, not a deduplicating key set.

**Confirmed/static, hosted filter:** KXj.A02 returns its original set when
feature 0x1da3 is false (0004..0016). When true, 0016..005e retains only
non-null members for which Cbb.A00 is false; a null member throws rather than
being silently skipped. Cbb.A00 0000..001e recognizes Blg and Bli. Their
constructors enforce device ID **99** (0016..0038 / 0016..0030); their raw
servers are respectively hosted and hosted.lid, with getType values 24 and 25.
This confirms exclusion of hosted devices behind the feature. It does not
justify dropping device 99 on an ordinary PN/LID address by numeric ID alone.
The current feature value and upstream hosted-device attestation remain unknown.

0bN.A0B's non-self branch copies 1IW.A09(peer).keySet into a HashSet and
adds peer.getPrimaryDevice (0084..00b8). The two own-account equality branches
instead use A0A/A09 and own primary getters (0028..0084); their underlying
contents are untraced. 0bN.A02 returns an empty own-device set when 08Z.BNm
is false, otherwise 1IW.A08().keySet (0028..0054). BNm implementation, source
map freshness, signature validation and complete own-device synchronization
are unknown at this checkpoint; the map is not automatically a complete live
account directory.

JhJ.A00 receives separate include-primary and include-companions booleans.
The primary branch reads 0kc.A0B().JAk.A01, strips its first byte, converts
through B7H.A03 and appends B7D.A00 (0008..0048). The companion branch applies
KXj.A02 to 0bN.A02, then CSC.A00 and appends each found key (0048..009e).
1xf.A1D constructs an ArrayList. Unlike the earlier peer-map gate, this helper
has no matching own-device cardinality refusal. Do not infer a universal
all-device gate or key deduplication from the peer branch.

**Confirmed/static, scheme predicate:** 26B.A00 0000..0042 requires JID type
18 (0DE.A0a), loads its contact through 0YA.A09, and returns 0np.A06's result
when that contact is present. 0np.A06 0008..0026 tests contact 0DV.A0D.A04
for membership in **{1,2,3}**. This is distinct from the previously traced
0lr.A01 ==2 verification-control exclusion. The semantic enum and producer of
0Da.A04 remain **unknown**; equal numeric values do not identify it as the
stored Signal attestation enum or QR HostedState.

JhJ.A0Z uses fresh feature 0x1d2c checks and this predicate at 0104..012a
and 01ee..0214, the captured A04 boolean at 0174..018c, PN/LID mappings at
01aa..01b6 and 02e8..0306, and username/other inputs before A0U/A0T. KXj.A03
captures feature 0x1d2c when constructing JhJ (0012..002c). KXj.A04 returns
true for that feature or a non-null contact passing 0np.A06 (0000..002e).
These traces resolve input producers and a static predicate; they do not
establish this account's active scheme or the full username/boolean branch
semantics. An explicit caller-supplied display calculation remains labelled
unverified selection. No human verification was committed.

Reproduce via apk-smali with exact class filters LX/JhJ; (A0Z/A00), LX/KXj;
(A02/A03/A04), LX/CSC; (A00), LX/0bN; (A0B/A02), LX/Cbb; (A00), LX/Blg;,
LX/Bli;, LX/26B; (A00), LX/0np; (A06), and LX/1xf; (A1D/A1E).
Private fingerprint-*.smali artifacts under lab/apk-2.26.37.71 preserve these
scoped outputs. Raw APK and account identities are not committed.

## Explicit native V4 QR payload export

**Implemented/offline from confirmed 37.71 schema/serializer traces:**
`fingerprint::qr::encode_v4` now serializes the native combined fingerprint
protobuf. Version 0 includes sorted 0x05-prefixed public-key inventory in each
individual tag 1; version 1 includes the 64-byte **identifier-free H(K,K)**
digest in tag 6. Combined tag 1 is present even for version zero; local and
remote remain in combined tags 2/3 without display-number lexical reordering.
Native descriptors/serializer addresses above carry the evidence. This is not
Java legacy BTe/M76 nested-field semantics.

Each individual preserves full LID JID in tag 3, optional caller-supplied PN
in tag 2, username in tag 4, and hosted E2EE=0/HOSTED=1 in tag 5. Missing state
is omitted, never encoded as -1. E2EE zero retains explicit presence. Local
PN is omitted when a username is **present**, including an empty username;
remote PN is retained. Optional empty bytes preserve presence, unlike null.
The Rust wire model represents the two valid hosted enum members as int32
varints behind the typed HostedState enum; it cannot emit unknown members.
The lower-level encoder validates bounded ASCII/keys and version, preserves
provided full strings, and does not claim to implement the APK JID parser,
active scheme selection, comparison or verified-state persistence.

The existing private display JSON accepts optional `lid_jid`, `pn_identifier`,
`username_identifier` and `hosted_state` (`e2ee`/`hosted`/null) per side. Display
alone ignores these QR-specific inputs. Add the paired flags:

```sh
whatsapp-cli fingerprint-display --config <private-json> \
  --qr-version 0 --qr-output <new-private-payload.pb>
```

This exports **binary comparison protobuf**, not a rendered QR image. Version
1 selects the alternative native QR key-hash representation, not scheme V1.
The CLI requires a plain primary @lid JID on both sides whose parsed user is
exactly the supplied display_identifier. That is a documented ordinary-JID
subset using our parser, not a claim to cover all APK grammar/device/hosted
variants. Each side also requires PN or username presence, because the traced
native comparator rejects both absent even when LID/key agree. No missing
identifier or hosted status is invented. Caller-selected complete inventories
and actual scheme/optional inputs remain unverified.

The command parses the private input once for both display and export. It
validates/encodes before creating the output with create_new and mode 0600,
then writes and syncs it. Existing files refuse without alteration. A write
failure preserves the private partial artifact and does not claim success.
Stdout contains only the existing calculation plus qr_export metadata (format,
version, byte count, private_file_created); no payload, input key/JID or output
path is printed. The exported file itself contains private contact identifiers
and public key/hash material. No account store opens, socket, send, pin change,
QR comparison or human verification occurs.

Independent [native QR fixtures](../../crates/whatsapp-api/fixtures/fingerprint/native-qr-3771.json)
use literal tags from the pinned merged descriptor and a separate Python
varint/length encoder, not Rust/prost-derived expected bytes. For each fixture
side K is the sorted 0x05-prefixed concatenation; `check_flow(library,K,K)`
checked its digest against the pinned native hash control flow (72825
instructions each, with host-modeled SHA/MCF). Version 0's individual tags are
1/3/4/5 locally and 1/2/3/4/5 remotely; version 1 replaces tag 1 with tag 6 at
the end. Both outer versions explicitly encode tag 1 then local/remote blobs.
These synthetic fixtures establish wire construction and digest ordering;
whole-native serialization, QR-image rendering, real-peer scanning and active
inventory compatibility remain **unknown/unverified**.

Regression tests check literal wire bytes for both versions, identifier-free
hashes, full LID versus display-user bytes, local PN suppression, remote PN
retention, hosted zero/one/absence, empty-versus-absent optionals, unknown version,
invalid inputs, redacted Debug/errors, CLI flag pairing, private file permissions,
refusing overwrite and ordinary LID/display consistency. Existing native
numeric fixtures still match after sharing key-inventory serialization.

The qualified-zero negative test exposed a CLI validation gap: our generic
Jid::parse maps a zero agent/device to Plain and to_string removes the qualifier.
The QR CLI now additionally requires parsed.to_string() == original lid_jid.
The generic parser is unchanged; this is the export's declared canonical-plain
subset guard, not an inferred WhatsApp server requirement. The regression covers
explicit :0/:2 and wrong hosted/PN domains before creating any export.

Validation passed 540 workspace/all-feature tests, 233 default-feature tests,
strict all-target/all-feature Clippy, warning-free all-feature library rustdoc,
generated CLI reference and formatting. A rebuilt default-feature command-level
check exported both synthetic fixtures exactly: version 0 was 200 bytes and
version 1 229 bytes, both mode 0600. Repeating each export refused, emitted no
success output, and preserved its bytes. The supplied unused account-store path
was never created. Private input/output/stdout/stderr evidence is retained in
lab/fingerprint-qr-synthetic-check. No current-account QR, native full serializer
execution, actual QR image/scan, live request or human verification is claimed.
Final staged repository hygiene (302 local links) and diff whitespace checks
also passed; only reviewed source/docs and synthetic fixtures are committed.

## Executed native V4 comparison branches (2026-10-07)

**Observed/offline, with explicitly modeled dependencies:**
[native_v4_comparison_check.py](../../tools/apk-research/native_v4_comparison_check.py)
executes actual APK **2.26.37.71** reconstructed `libwhatsapp.so` instructions
at `0x7152fc..0x715688` (core plus PN/username helpers) and
`0x7152b8..0x7152fc` (status remapper), reading its table at `0x302404`.
The runner requires SHA-256
`f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a`,
from APK SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This builds on the attributed static comparator trace above. It accepts a
library path only and supplies built-in synthetic parameters, never account
keys/identifiers or scanned user payloads. No account store/network opens.

The [115-case artifact](fingerprint-native-comparison-cases.json) records
actual core code, remapped code, instruction count and modeled-string-call
count, or an explicit invariant-abort observation. Its expectations are
literal instruction return branches and the separately traced table, not a
Rust comparator's expected values. All ordinary runs returned to the designated
return PC, using at most **245 instructions** including remapping. Each core
and remapper invocation has a ten-second/one-million-instruction bound;
unexpected branches, budget exhaustion and wrong/missing libraries refuse
without partial JSON output. Expected aborts are intercepted as Python
exceptions, not allowed to abort a host process.

**Modeled boundary:** LID/key getter-equality helpers, PN/username/hosted getters,
MCF string allocation/release, PN byte equality and username string comparison
are host models. Logging is disabled. The actual PN helper converts the modeled
equality boolean into its mismatch result; the actual username helper executes
its absent/present branches and passes literal flag 1 to the modeled string
comparison. Host `bytes.lower()` is used on these synthetic ASCII examples,
**not** asserted to reproduce Android libc/locale behavior. Whole protobuf
unpacking/conversion, native object construction, orientation wrapper, JNI,
Android string primitives and current APK inventory are **not executed**.

The cases establish these branch results with matching key/LID models:

| Pairing issue | Local code | Remote code |
| --- | --- | --- |
| Either known or scanned side lacks both PN and username | -39 | -41 |
| Both PNs present but unequal | -17 | -19 |
| Both usernames present but modeled comparison unequal | -24 | -29 |
| Known username present, scanned absent | -26 | -31 |
| Known username absent, scanned present | -28 | -33 |
| Known/scanned hosted state unequal when checked | -34 | -36 |

A missing PN alone succeeds when the usernames match; both absent usernames
succeed when PNs match. Versions 0/1 select only their public/hash key field;
a mismatch in the other field is ignored on the core path. Known remote LID
mismatch is -12. The core does **not** repeat the known-local LID orientation
check; the previously traced wrapper supplies that guard. Core zero in the
synthetic local-LID mismatch case is therefore **not** a valid full scan result.
The runner invokes remapping separately; it does not prove which orientation
would select that remapper. Actual wrapper tracing remains the authority for
when to swap/remap, and success alone never records human verification.

All **81** combinations of known and scanned hosted states {-1,0,1} were
executed. When **both scanned states are absent (-1)**, the legacy branch
returns zero regardless of the two known states. When at least one scanned
state is present, it checks local agreement first (-34), then remote (-36).
This resolves the earlier "can return zero" boundary; do not reject the legacy
both-absent case solely because known states are present. Two direct-core cases
with scanned state 2 reach native abort. They are invariant tests, **not** proof
that a real QR causes an abort: the preceding converter rejects invalid known
enum values before calling this core.

**Confirmed/static converter details, same build:** `0x7150ac..0x715264`
returns -43 for absent LID (`0x715134`), -1 for failed
LID ASCII creation (`0x71514c`), -44 for failed PN creation (`0x71518c`), -23
for failed username creation (`0x715128`), -38 for a present hosted enum outside
0/1 (`0x715174..0x715188`), and -4 for an absent version-selected key
(`0x7151a0..0x715220`). Hosted absence becomes -1. These are internal native
codes, not server reasons. No key length/content validation is established by
the converter trace alone. Complete protobuf-C acceptance (including duplicate
fields/unknown enums), Android username behavior, UI status meanings and a
production scan comparator remain **unknown/unimplemented**. The private
`fingerprint-v4-qr-parse.txt` and comparison dumps retain the source ranges.

Reproduce:

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_v4_comparison_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so
```

The artifact regenerated identically in a separate invocation. Three focused
checks passed, including actual pinned execution and missing/wrong-build
refusals with empty stdout. This is offline branch verification, not a live
scan, automatic verified-state write or companion synchronization.

## Actual native ASCII username comparison (2026-10-07)

**Confirmed/static and observed/offline, explicitly scoped fixtures:** optional
paired `--string-library` and `--libc` flags on the existing V4 checker replace
its host case-folding comparison with actual APK **2.26.37.71** merged
`MCFStringCompare` (`0x538a50..0x538c34`) and the public Bionic rootfs fixture's
`strncasecmp` (`0x64e88..0x64ec8`). This remains a synthetic execution slice,
not an Android client or a scan. The pinned inputs are:

| Artifact | SHA-256 | Authority |
| --- | --- | --- |
| Reconstructed libwhatsapp.so | `f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a` | APK 37.71 comparison/helper instructions |
| Reconstructed libwhatsappmerged.so | `565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa` | APK 37.71 MCFStringCompare instructions |
| Public rootfs system/lib64/libc.so | `4e6d025f3356a3353418981e475561cfa133a0e439a20c9d9bede715ae167697` | Comparison Android/Bionic fixture; **not APK-bundled libc** |

The [public rootfs provenance](native-unpacking.md#bounded-loader-check)
and existing hashes are recorded in native-unpacking; the public fixture is
not evidence of the server, this account's platform or every Android build.
The APK SHA-256 remains `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

**Confirmed/static in the public Bionic fixture:** strncasecmp has no calls or
locale/TLS reads. It uses an unsigned byte lookup in a fixed table at
`0xab94a`, compares folded bytes, stops on difference/NUL/count exhaustion and
returns their difference. `strncasecmp_l` at `0x260d4` tail-calls strncasecmp
without inspecting a locale argument. The separate `tolower` implementation
at `0x5e2c8` uses a table pointer; it is **not called** by this strncasecmp
slice. These statements are fixture-local; they do not resolve arbitrary
Android libc versions or Unicode comparison, which are not accepted native
ASCII fingerprint inputs in the traced creator.

The runner supplies synthetic MCFString objects in the traced ASCII/pointer
representation: +0x19 storage flag 0, +0x1a ASCII flag 1, +0x24 character count,
+0x28 pointer to bytes. Native object construction/retention remains modeled.
Synthetic TLS supplies the stack guard. The actual merged flag-1 branch resolves
GOT slot `0x759d90` to the pinned Bionic entry, then executes its instructions;
the flag-zero strncmp slot is null so an unexpected branch refuses. No merged
or Bionic comparison instruction is replaced by host lowercasing in this mode.
Other key/LID/getter/PN-equality/logging primitives remain modeled. No library
initialization, JNI, protobuf unpacking or scan-orientation wrapper executes.

**Observed/offline:** all original **115** core cases retain identical native
and remapped status codes with actual string comparison. Those cases execute
**195** native string comparisons and at most **493** instructions per
core/remap run. A further **268** cases exercise both local and remote username
pairings: all accepted ASCII bytes 1..127 with swapped letter case, a distinct
mismatch at each of 127 positions, both prefix-length directions, empty/empty,
empty/nonempty in both directions, and the local 256-byte input bound. All
match ASCII case-insensitive byte equality plus exact length. This comparison
is not Unicode normalization. Maximum instruction count was **2986**; every
ordinary run returned within the existing ten-second/one-million-instruction
limits. Wrong/missing libraries or unpaired flags refuse before emulation and
before any JSON output. A
[compact attributed observation](fingerprint-native-string-check.json) records
hashes/counts/bounds; the checker reproduces full core results plus the ASCII
check summary.

Reproduce:

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_v4_comparison_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsapp.so \
  --string-library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so \
  --libc lab/apk-2.26.37.71/native/qiling-rootfs/system/lib64/libc.so
```

Native-string regression execution requires `TELEWHATSAPP_FINGERPRINT_LIBRARY`,
`TELEWHATSAPP_SCHEMA_LIBRARY` and new `TELEWHATSAPP_BIONIC_LIBRARY`; otherwise
that fixture-dependent case explicitly skips. Default modeled mode still
matches its original committed artifact. This resolves the practical
ASCII-comparison primitive for the pinned comparison fixture. **Unknown:**
compatibility across other Android libc implementations and complete current
APK scheme/inventory/QR parser behavior. Production scanning and human
verification remain separate unfinished work; no account state changed.

## Explicit-input native V4 binary comparison (2026-10-07)

**Implemented/offline, grounded in confirmed static APK 2.26.37.71 traces:**
`fingerprint::qr::compare_v4` and `whatsapp-cli fingerprint-compare` compare a
binary native V4 payload against explicit known inputs. Source attribution is
unchanged: APK SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
main library SHA-256 `f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a`,
merged SHA-256 `565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa`.
Addresses above identify native descriptors, converter, wrapper and core;
the executed native cases remain the independent authority for core results.

The wrapper order is implemented from `0x714a60..0x714c70`: protobuf decode,
version presence/range, both individuals, both LID presence bits, then
local/remote conversion. Early statuses are **not remapped**. Converter
`0x7150ac..0x715264` checks ASCII LID, PN, username, hosted enum and the
version-selected key presence in that order. Versions 0/1 compare public/hash
fields respectively. Known inputs are constructed with both PN and username
retained, as params constructor `0x71916c` does; the known local holder must
not use QR serialization's local PN suppression.

Known local LID matching the scanned local side takes priority (including when
both scanned LIDs match), invokes the core and remaps its code. Matching only
the scanned remote side swaps the pair and returns the core code unchanged.
Neither matching is -13. A wrong remote LID is therefore -11 in local-first,
-12 in remote-first. Returning core zero without the wrapper orientation guard
would have been incorrect; the public comparison function includes that guard.

PN checks preserve absent versus empty and use byte equality only when both
values are present. Username checks distinguish missing/different sides and use
ASCII case-insensitive equality **with exact length**. This is independently
executed against the actual merged MCFStringCompare and pinned public Bionic
fixture, not an invented Unicode/locale normalization. Both scanned hosted
states absent take the native legacy success branch regardless of known states;
otherwise local then remote agreement is checked. An invalid enum returns -38
in conversion before the core; no native invariant abort is exposed.

Use the existing private display/QR JSON (local/remote raw key inventories,
full canonical primary lid_jid, matching display_identifier and chosen optional
PN/username/hosted inputs), plus private binary protobuf:

```sh
whatsapp-cli fingerprint-compare --config <private-known-inputs.json> \
  --scanned <private-native-payload.pb>
```

Both files must be regular, <=64 KiB and exclude group/other permissions.
Output contains `status` (native internal integer), `matches_explicit_inputs`
(status zero only), `payload_version`, nullable `orientation`,
`input_authority=caller_supplied_selection_unverified` and
`parser_authority=apk_native_schema_corpus_validated_other_edges_unverified`. No key,
contact identifier, scanned bytes, input path or raw parse failure is printed.
Mismatches are valid comparison results. Known-input/file-policy errors refuse
without claiming a comparison. No account store, network or metadata write is
performed, including on status zero. This command consumes binary protobuf;
it does not decode a QR image or assume a camera/device is available.

**Local policies:** empty input refuses instead of reproducing the APK's
empty-buffer abort; payload size is bounded before decoding. Known CLI input
uses the existing canonical primary-LID/display consistency and PN/username
presence guard. These are our operator/resource policies, not server rules.
Known library inputs keep existing bounded ASCII/key validation; scanned
strings are limited by the whole payload bound and follow the native ASCII
byte acceptance (including empty values).

**Implemented/corpus-validated:** the original prost decoder was refuted by
actual native unpacking on eight edge cases. The bounded fingerprint-schema
decoder now matches the actual 50-case corpus, including decoded field values;
see the native-parser follow-up below. Other protobuf-C edges beyond that corpus
remain **unknown/unverified**. Other
Android libc versions, current account/peer scheme and complete inventory,
QR-image decoding, live peer scanning and companion verification sync remain
unfinished. No successful explicit comparison automatically marks a peer
verified. The separate manual decision still requires its saved-pin/sequence
guards and an actual human decision.

Tests compare every **113** non-abort executed core/remapper case against the
committed native observation artifact, plus all **268** native string cases.
Independent version-0/1 wire fixtures succeed in both orientations; wrong
remote/no matching local LID, conversion ordering, missing fields, invalid
ASCII/enums, version-selected missing keys, size/empty refusals and output
redaction are covered. CLI checks require both private paths and validate
known input/redacted output. These are synthetic offline tests, not a live
scan or active inventory verification.

Command-level synthetic validation also matched version 0/1 with status zero
and local-first orientation while a deliberately unused account-store path
remained absent. Malformed bytes returned decode-stage -8; public permissions
refused with empty stdout before any store creation. Private synthetic files
remain ignored under `lab/fingerprint-comparison-synthetic-check/`.

## Actual protobuf-C parsing and bounded decoder correction (2026-10-07)

**Confirmed/static and observed/offline, APK 2.26.37.71:**
[native_fingerprint_parse_check.py](../../tools/apk-research/native_fingerprint_parse_check.py)
executes the actual merged-library generated combined unpack wrapper
`0x2d3e28..0x2d3e44`, `protobuf_c_message_unpack` at `0x703158`, its internal
scan/assignment/merge helpers, generated fingerprint/combined initializers at
`0x2d3db0..0x2d3e18`, and free wrapper `0x2d3e44` / free routine `0x7046c8`.
Library SHA-256 is `565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa`;
APK SHA-256 is `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
These are native parser instructions, not a host reimplementation of protobuf-C.

The runner maps the ELF PT_LOAD image and resolves R_AARCH64_RELATIVE plus
supported defined ABS64/GLOB_DAT/JUMP_SLOT relocations. Only identified generated
init/unpack/free/internal-helper instruction ranges execute; unexpected branches
refuse. Allocator/free/memcpy/memset are explicit host models, with synthetic TLS
and stack. There is no native library initialization, JNI, account or socket.
The command accepts a library path only, supplies built-in synthetic buffers,
and rejects missing/wrong builds before emulation. Every unpack/free invocation
must actually return within ten seconds/one million instructions. Payloads are
bounded to 64 KiB and the monotonically allocated guest heap to 1 MiB. All modeled
allocations must be freed after success/free or rejection; these bounds are our
research tooling policies, not server/Android limits.

The [50-case corpus](fingerprint-native-parser-cases.json) records synthetic
payload hex, accepted/rejected flag, version/presence, every recognized individual
field as optional hex bytes/enum, native unknown-field counts, instruction count
and modeled allocated bytes. Maximum combined unpack/free instructions were
**3173**, maximum total allocation **608 bytes**. Both independent version-0/1
wire fixtures parse with exactly their expected data. The raw corpus is synthetic
protocol data, never a real account QR or contact/key dump.

**Refuted/observed:** the first 40-case comparison against prost found eight
acceptance differences: outer/inner groups, above-64-bit scalar/unknown varints,
six-byte tags, overlong zero tags, tags above the standard 29-bit field-number
range, and six-byte empty lengths. The original parser-authority qualifier was
necessary: matching ordinary serializer bytes was not parser equivalence.
A subsequent otherwise-valid wire fixture with an unknown group confirms native
rejection; prost accepts it without changing recognized fields, so our old
comparison could have returned a **false match**. The reusable decoder is now
corrected and the regression resumes with decode-stage -8, without any account
operation. No malformed data was transmitted or presented as human verification.

**Confirmed/static rules, validated by actual corpus:**

- Native tag scan `0x7035d0..0x703640` rejects first byte <8 and allows at most
  five tag bytes. Field-id accumulation uses w26 and shifts 4/11/18/25 after
  extracting the first four id bits: up to 32 id bits, not the standard 29-bit
  cap. An overlong encoded zero tag can pass the first-byte guard and remain
  an unknown field; literal zero rejects. These unusual rules are not guessed
  standard protobuf behavior.
- Wire scan `0x703788..0x7038b8` accepts only varint/fixed64/bytes/fixed32
  (types 0/1/2/5), rejects groups 3/4 and types 6/7, and requires enough bytes.
  Varint termination must occur within ten bytes; the tenth terminal byte is
  not restricted to <=1. UInt32/enum values preserve their low 32 bits.
- Length scan `0x703808..0x703898` allows at most five bytes, accumulates in
  x10, rejects `length >> 31 != 0`, and bounds the body against remaining data.
  Lengths do not wrap to uint32: values >=2^31 reject.
- Required-member/merge helpers `0x704bcc..0x705630` preserve field presence,
  accept explicitly empty bytes, use the last scalar/byte value, merge repeated
  singular nested messages, and reject mismatched wire types for known fields.
  Unknown enum values, including negative int32, are preserved by **parsing**;
  the fingerprint converter separately rejects values outside 0/1.
- Standalone unpack accepts empty data as an empty message. This is not the
  complete comparison wrapper: that wrapper aborts for an empty buffer, while
  our public comparison intentionally refuses empty as local resource policy.

`qr/decode.rs` now implements these rules for this two-message fingerprint
schema, keeping the existing payload bound and fixed schema depth. Unknown
fields are opaque and skipped; native unknown-field storage/counts are not
round-tripped because comparison does not read them. This is a fingerprint
comparison decoder, not a replacement for every protobuf operation in the
client. `compare_v4` uses it before the existing conversion/orientation/core
logic. Output now reports
`parser_authority=apk_native_schema_corpus_validated_other_edges_unverified`.
A corpus test checks **acceptance and all recognized decoded values** for every
case, and the otherwise-valid-plus-group regression proves -8 instead of zero.
No pin/session, verification sequence, message ID or server wait changed.

Reproduce actual native unpacking:

```sh
PYTHONDONTWRITEBYTECODE=1 lab/native-analysis-tools-py312/bin/python \
  tools/apk-research/native_fingerprint_parse_check.py \
  --library lab/apk-2.26.37.71/native/qiling-rootfs/decoded/libwhatsappmerged.so
```

Three tool checks cover missing/wrong-build empty-output refusal and actual
corpus reproduction (the latter explicitly skips without
`TELEWHATSAPP_SCHEMA_LIBRARY`). Private scoped disassemblies
`fingerprint-protobuf-unpack.txt` and `fingerprint-protobuf-unpack-helpers.txt`
retain the instruction evidence. **Unknown/unverified:** arbitrary parser edges
outside this corpus, allocation-failure/OOM compatibility, other APK builds,
whole JNI/wrapper execution, current account inventory and live peer scanning.
No broader parser or human-verification equivalence is claimed.

Validation (offline): 552 workspace/all-feature tests, 240 default-feature tests
and all 25 fully configured native tooling checks passed, alongside strict
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference. A rebuilt CLI matched both independent version-0/1 fixtures and
returned -8 for the otherwise-valid-plus-group and malformed fixtures; the
unused account-store path remained absent. The corpus test also checks explicit
nested-holder presence, with its final focused rerun passing.
Repository hygiene passed with 323 local links; staged diff whitespace checks passed.

## Own-directory source, logout filtering and alias precedence (2026-10-07)

**Confirmed/static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [scoped instruction evidence](fingerprint-own-directory-evidence.txt)
resolves the own-device producers left open in the earlier inventory section.
Offsets below are method-relative bytes. This is static construction evidence,
not a current-account directory, authenticated device membership or live scan.

**Mode predicate:** exact `BNm` method-definition search across the APK found
three definitions: abstract 08Z, 08a forwarding to 08i (0000..000e), and
08i.BNm. The latter calls A09, then returns whether its `A0F:Me` is non-null
(0000..0014). A09 conditionally runs the 08t loader (0000..002a). This closes
BNm's immediate implementation; it does **not** check a directory signature,
ADV timestamp, logout time or map completeness. The complete loader/Me producer
and actual current value remain unknown. Do not describe this gate as signed
inventory freshness or derive a linked-device membership claim from it.

**Stored directory:** 1IW.A08 obtains 1IY.A07, then filters its entries
(0000..0078). 1IY.A01 lazily reads the `devices` table: 1Kt.<clinit> builds
`SELECT <1Ku.A00 columns> FROM devices` (0016..0042), with no WHERE clause in
that constructor. A01 parses `device_id` via 0DK.A04 and skips failed parses
(01ec..0218). It records the difference between rows read and map size as A00
(042c..043a), rather than refusing the entire map. Complete duplicate-row and
parser semantics are untraced; do not label that difference exclusively as
unparseable rows. The table contains more than active fingerprint devices.

**Logout filter:** A01 reads `logout_time` into v74/v75 (0074..0080,
038e..039a), supplies it as the third long argument to HxZ.<init>
(0400..0416), whose 002e..0036 stores it in HxZ.A02. HxZ.A03 compares this
field with zero and calls 1xi.A1Q (0000..0016); A1Q returns true only for a
strictly positive comparison (0000..000a). 1IW.A08 omits such rows
(004a..0056). Thus **logout_time > 0 is excluded**, while zero and negative
values pass this particular filter. This is a traced predicate, not an
assertion that negative logout times are normal or valid server inputs.
1IW.A0F independently writes HxZ.A02 to the `logout_time` column
(0192..01a8), corroborating the field mapping. It is unrelated to whether an
identity key or signed-list entry is available.

**Alias precedence before filtering:** 1IY.A07 builds a cached projection of
A01's raw directory under a monitor. If the account PN getter Aq0 is null,
it retains the raw map (0018..0024, 013e..0142). Otherwise its first pass
preserves every entry except LID-form entries belonging to self
(002e..0092). The second pass considers exactly those removed self LID entries
(0092..00e2), maps their device number onto the account PN using 0DK.A02
(00e2..00f6), and:

- Keeps the original LID row if conversion returns null (00f6..010e).
- Uses an existing entry at the mapped PN address when present; it does not
  overwrite that row (010e..011a).
- Otherwise inserts an HxZ copy with the mapped address (011a..0134).
  HxZ.A01 copies the original logout/key-index/other metadata while replacing
  the address (0000..007c).

The order matters: **PN-row precedence occurs before 1IW.A08's logout
filter**. A present logged-out PN row can suppress the corresponding LID row
in the projection and then itself be filtered out. Choosing whichever alias
looks active, unioning both key inventories or dropping logout rows before
alias selection would not reproduce this traced path. The first-pass LID
predicate 0DE.A0Z includes JID types 18, 19 and 25 (0000..002c); it is not
merely an ordinary `@lid` string suffix test. 0DE.A0e separately recognizes
type 0 (0000..0016). Hosted filtering still occurs later through KXj.A02,
as described above; the hosted feature's current value remains unknown.

**Fingerprint-facing alias sets:** 0bN.A09 uses A02's already filtered map
keys, maps type-0 devices onto own LID with the same device number, catches
conversion failures with `Failed to map to LID companion`, and directly retains
LID-form devices (0036..008e). A0A retains type-0 devices directly; a LID-form
entry is mapped onto own PN only when own PN exists and BO0 confirms self
(003a..008c). Both outputs are HashSets of device addresses. This does not
establish deduplication of identity-key bytes: JhJ's subsequent key lists remain
ArrayLists, as traced earlier. Source map filtering and address normalization
must precede identity lookup and fingerprint hashing.

**Cache mutation boundaries:** the observed add-my-device writer 1IW.A0F
replaces a `devices` row, marks its database transaction successful, then clears
both 1IY.A01 (raw) and A02 (projected) caches under the monitor
(0450..0474). The removal path 1IY.A08 expands input addresses through A06,
deletes matching `devices` rows, marks the transaction successful and likewise
clears both caches (001e..0026, 0062..0086). These two paths explain why a
cached map need not remain unchanged after those mutations; they do not prove
complete synchronization, all mutation paths, or atomicity across the additional
outer transactions and notification effects. A06 expands self-device selectors
to available PN/LID aliases (0000..00c6); its full conversion semantics remain
untraced. No deletion, reset, account mutation or linking action was executed.

**Negative results / remaining unknowns:** the exact 0bA class-definition probe
found no matching class, so it is not used as BNm implementation evidence.
The 1IY.A02 caller search found six callers, but A02 is a cache-row lookup,
not the hypothesized cache invalidator (0000..0050). 1IW.A00 instead groups
instrumentation-device identifiers (0000..00dc); A0B clears the separate
`user_device_info` and `user_device` tables (002c..00dc), not the own `devices`
source studied here. Those methods were inspected only to distinguish the
paths and were never invoked. The 1IW.A09 non-self map still branches through
B7j or Chv according to 0bq.A06 (001a..00a2); full contact-directory freshness
and alias authority remain untraced. Complete add/remove callers, account-sync
transport authentication, primary ADV ownership, current feature/contact-state/
username inputs and own identity-key lookup coverage remain unfinished. The
[account-sync authority distinction](cross-device-sync.md#refresh-response-versus-primary-adv-ownership)
still applies. Our explicit-input fingerprint commands do not acquire current
inventory or mark a peer verified from this evidence.

Reproduce with existing static CLI, for example:

```sh
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class '' --method BNm
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/1IW;' --method A08
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/1IY;' --method A07
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/HxZ;' --method A03
```

Additional scoped sources are 1IY.A01/A06/A08, HxZ.<init>/A01,
1xi.A1Q, 1Kt.<clinit>, 1IW.A0F, 0bN.A09/A0A and 0DE.A0Z/A0e.
Full raw dumps, caller lists and unsuccessful probes remain ignored in
`lab/apk-2.26.37.71/fingerprint-own-*.smali` / `*.txt`. The committed evidence
contains only relevant attributed instruction ranges and no media/account data.

Validation: the APK hash was rechecked, and an independent exact A08 method
extraction matched the scoped snapshot byte-for-byte. CLI reference regeneration
and its test passed; repository hygiene passed with 329 local links. The initial
snapshot's trailing blank line was removed; final staged whitespace checks passed.
This change adds static research evidence only; runtime code and API/CLI
capabilities are unchanged, and no live account validation is claimed.

## Fingerprint contact predicate is external-user state (2026-10-07)

**Confirmed/static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Scoped instruction evidence and exact write sites](fingerprint-contact-state-evidence.txt)
resolve the storage domain of the previously unnamed 0Da.A04 scheme input.
Offsets here are method-relative bytes; the artifact's field-reference list
separately identifies absolute DEX byte offsets.

0Da's constructor initializes unrelated boolean/string members and contains
no A04 write (0000..001a). The clone helper identifies the object as
`com.whatsapp.infra.core.data.contact.WaContactData`; field existence alone
therefore did not identify an enum or its producer. The new exact
`xtask apk-field-refs --class 'LX/0Da;' --field A04 --type I --writes-only`
search found **17 standard DEX writes in 13 methods** across the pinned APK.
A second search with reads included found 36 total sites (19 reads, 17 writes).
This counts direct iget/iput/sget/sput instructions, not native or reflective accesses.
It located the stored-field mapping without guessing neighboring class names.

**Storage mapping:** 0lO.A01 obtains the `external_user_state` cursor column,
reads its integer and writes 0Da.A04 (06aa..06c2). Conversely 0Y9.A0b reads
A04, writes the same named ContentValues column, and calls 0gt.A0S with the
contact's JID (0060..0088). The mapping is confirmed in both directions.
**Refuted:** this field is not identified by its numeric values as the saved
Signal encryption-attestation enum or the native QR HostedState. The field's
actual storage domain is external-user contact state; its producers below
include Cameo/guest and pending-invitation transitions. No server/internal
meaning is inferred just because another enum has equal numeric members.

The observed producer branches are:

| Producer | Assignment / guards | Evidence |
| --- | --- | --- |
| 0Y9.A0B | Uses value 1 on new LID contacts; existing contact conversion additionally checks 0np.A0A, chat existence and 0np.A06 before writing | 0034..007e, 008c..00b0 |
| 0Y9.A0C | Writes 4 when the existing contact does not pass 0np.A09, or creates a new contact with 4 | 003a..0082 |
| 0Y9.A0D | Writes 5 when the existing contact does not pass 0np.A0A, or creates a new contact with 5 | 003a..0082 |
| 2YX.A07 | `CameoUserJoinedNotificationHandler` logs the guest LID, loads literal 1 and later writes it to the contact | 0050..0074, 013e..0150 |
| 35v.A07 | The `CameoRegisteredNotificationHandler` branch loads the guest LID and reason, then writes literal 3 to a non-null contact | 163a..16aa |
| 2xA.A01 | `CameoUtils/transitionCameoUserToNormalWAUser` starts with literal 0, then writes 0 before saving the contact | 0000..0020, 0064..0082 |
| 2xA.A00 | Behind its additional reason/chat gates, a contact passing 0np.A06 is written with literal 2 | 0000..009e |
| 1xz.A01 | `PendingInviteUtils/deliverUnsentMessagesForPendingInvite` uses the supplied isGuest boolean as integer 0/1 when updating a qualifying contact | 002c..003a, 00b2..00c0, 010e..0128 |
| 17J.BFC | Its existing message-status path has LID/contact/0np.A06 guards; branches testing numeric codes 0x1005/0x1006 write 0 or 2 respectively | 0a90..0a92, 10b2..11a2 |

These are **branch descriptions**, not invented canonical enum names. Complete
reason definitions, notification authentication, persistence ordering and the
message-status codes' server-facing meanings remain unknown. In particular,
1xz's direct boolean assignment does not authorize setting isGuest ourselves
or sending any of its pending messages. None of these APK handlers was executed.
The remaining direct writers AW9.BqR, 33c.A00 and 3Ng.A00 are located but their
full provenance is not established; 33c's visible write takes its supplied
integer argument (0070..0078), so that site alone does not define an enum.

**Scheme consequence:** 0np.A06 still tests exactly A04 membership in {1,2,3}
(0008..0026); 26B.A00 still requires a LID-type target and an available contact
before delegating to that predicate. Thus the predicate concerns these external-
user contact states, not a generic hosted-device or human-verification flag.
The separate 0np.A09 tests 4, A0A tests 5, and A02 tests {2,3}; their existence
corroborates separate contact branches without naming an authoritative enum.
Neither this research nor the explicit-input comparator populates contact data,
sets feature 0x1d2c, or establishes this account's active scheme. Username and
complete current inventory integration remain open.

**Reusable research capability:** `xtask apk-field-refs` accepts the exact
class, field and type plus optional `--writes-only`. It scans standard
iget/iput/sget/sput variants using the shared class/method visitor, reports
DEX file, referencing class/method/shorty, opcode and absolute byte offset, and
prints the total on stderr. DEX filenames are sorted for reproducible output;
the ZIP iterator order was observed to vary between runs. Direct/virtual method indices reset independently;
instruction payloads are skipped. Results are assembled before stdout, and
missing DEX files or malformed traversed field tables/instructions refuse.
This is not a general DEX verifier or a native/reflection/quickened-offset
search. The older caller utility now shares this checked instruction visitor;
its existing per-DEX warning policy remains distinct from field-search refusal.
The pending-invitation trace also exposed a missing and-long mnemonic/operand
display: literal bytes `a0 07 07 13` at classes3.dex `0x1e0094` decode as
`and-long v7, v7, v19`, masking chat flags without changing isGuest in v1.
That reusable decoder gap is fixed with an APK-byte regression and truncated
prefix checks. This supports the boolean-to-integer flow above; it does not
define all chat-flag meanings.
Synthetic DEX tests cover read/write selection, exact class/name/type matching,
payload bytes that resemble writes, truncation and quickened-opcode exclusion.
No runtime account, field-value mutation or network capability was added.

Reproduce the field search and targeted mapping:

```sh
cargo run --locked -p xtask -- apk-field-refs ~/Downloads/WhatsApp.apk \
  --class 'LX/0Da;' --field A04 --type I --writes-only
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0lO;' --method A01
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0Y9;' --method A0b
```

Private raw method dumps and the writer list remain under
`lab/apk-2.26.37.71/fingerprint-contact-state-*` and
`fingerprint-external-state-predicates.smali`. No real contact values were read,
changed, copied into fixtures or described as current-account observations.

Validation: 555 workspace/all-feature tests and 243 default-feature tests
passed, including three new APK/DEX regression checks. Strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference passed. Final tool-only tests/Clippy also cover filename sorting.
Command-level no-DEX and malformed-DEX archives refused with empty stdout;
wrong field type produced zero sites. The shared caller search reproduced all
134 previous BNm call-site records; the first byte-for-byte check differed
because ZIP filename iteration order varies, so record sets were compared.
The new field-search command sorts DEX names to make its output reproducible.
No live account or identity-change test was performed in this change.
Two sorted field-search runs matched byte-for-byte. Repository hygiene passed
with 334 local links; final staged whitespace checks passed.

## Scheme, username and PN-sharing input branches (2026-10-07)

**Confirmed/static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [scoped selection evidence](fingerprint-input-selection-evidence.txt)
connects JhJ.A0Z's previously partial branches to the optional-string producers.
Offsets are method-relative bytes. No account inputs or APK feature values
were observed; no current-peer fingerprint was calculated.

**Scheme selection is not universally V4:** JhJ.A0Z retains the earlier
nonempty/exact-cardinality remote-device key gate (003a..006a). Its separate
Java branch calls A0V (0140..0170). For PN/LID target branches, it resolves
PN/LID aliases and optional inputs before deciding between native A0U and A0T.
The captured/contact gate at 023c selects A0U when true, or jumps to the
A0T construction at 030a when false. The later fresh feature/contact check
at 01ee..0214 chooses the PN-inclusion calculation, not the native scheme.
Its false branch at 0298..02d6 jumps **back** to shared construction at 022c. A0U's log names say
`generateV4Fingerprint` and its create selector is 2 / result selector 4
(00b2..00e2); A0T's names say `generateFingerprint` and its corresponding
selectors are 1 / 2. These connect to the already traced distinct native V4/V3
routines. A successful V4 explicit-input comparator is not a V3 comparator.
The current client still lacks integrated current-input selection and full
V3 binary comparison; substituting V4 based on convenience would change the
APK-selected scheme.

**Captured and fresh gates are separate:** JhJ.A04, captured by the earlier
KXj constructor, is combined with the target's 26B.A00 predicate at
0174..018e. That boolean controls the later external-contact special branch
and local key inclusion. A0Z additionally rereads feature 0x1d2c at
0104..0128 and 01ee..0214; it must not be reduced to one globally assumed
snapshot. The mapped remote LID's own contact predicate is computed separately
(01c6..01d2). An original PN target and its mapped LID need not produce the
same predicate result: 26B.A00 requires the LID type, as traced above.
Neither a PN/LID alias nor equal identity-key bytes proves contact-state or
feature equivalence.

**Local keys differ by branch:** A0T calls JhJ.A00(true,true) (030a..0312),
including its primary key plus found filtered own-companion keys. A0U is reached only when the captured/contact boolean is true, and calls
A00(true,!mapped_remote_external_predicate)
(023c..024c). Thus the special external-contact branch omits local companions
rather than hashing every retained identity. JhJ.A00 still reloads the own
source, applies KXj.A02 and CSC.A00, and appends available keys without the
remote branch's cardinality refusal (0048..009e). No key-byte deduplication
is introduced by this helper. This complements, rather than replaces, the
[own-directory filtering/alias rules](#own-directory-source-logout-filtering-and-alias-precedence-2026-10-07).

**Optional string helper:** 1cg.A02 returns null for null or length zero and
otherwise returns the original string (0000..0014). It does not trim, case-fold,
strip an `@`, parse a JID or infer a phone number. A0Z uses it for the own
username from Aq3 (01b6..01c6) and the remote candidate (01de..01ee).
Accordingly the selected own username is absent when empty. This differs
from the lower-level serializer's ability to represent explicitly present
empty bytes; those explicit-input fixtures remain valid primitive tests,
not evidence of UI input selection.

**Own username:** an exact Aq3 definition search locates abstract 08Z,
08a forwarding, and 08i.Aq3. The implementation first uses cached A03; otherwise
it reads `self_user_name` from 0FU's preferences, defaults null/empty to an
empty return, and caches a nonempty value (0000..0054). 08i.CU3 writes its
supplied string to that preference/cache without visible prefixing or trimming
(0000..004c). The exact preference-string xref has four sites: Aq3, CU3,
0g1.A0F and 5SX.A02. The latter two and complete setter callers remain
untraced; this does not establish whether this account's stored own username
includes `@` or how upstream validation formats it.

**Remote username-position input:** when the mapped LID is non-null, the
captured/contact boolean is true and its contact passes 26B.A00, A0Z uses
that LID's **raw JID string** as the remote candidate (01d2..01ee). This is
what the caller places in the username argument position; do not replace it
with a fabricated public username. The other usable-LID path calls
0Xr.ATB (02d6..02e4); no-LID yields null. Exact ATB definition search finds
0Xr's abstract declaration, 0Xs' cache/database adapter and 0o2's cache leaf.
0o2.ATB reads 0o3.A01 through ATA and applies the empty-to-null helper
(0008..0026). 0Xs.ATB first asks its provider; on null, it loads 0o5.A03 by
LID row ID, repopulates BJQ with the row fields and returns A01 (0008..0074).
The complete provider wiring and authenticated row producers remain unknown.

0o5.A03 selects `display_name, username FROM lid_display_name WHERE lid_row_id = ?`
(0010..0032), reads the username, maps length-zero to null, creates the row
and calls A00 (0064..00b6). A00 returns an A01 username formed by **prefixing
`@`** to a non-null nonblank supplied username; otherwise A01 is null
(0004..0068). Its blank predicate 0CM.A0p iterates UTF-16 characters and calls
0Gj.A01, which uses `Character.isWhitespace || Character.isSpaceChar`.
It does not trim a nonblank string before prefixing. Exact runtime Unicode
classification and upstream stored-string validation are unverified. This
proves a producer transformation, not permission to strip/prefix arbitrary
operator strings or normalize every native comparison input identically.

**Refuted label — PN sharing, not username availability:** the nullable Boolean
read via 0qq.A05 in A0T's branch is not a username-enable flag. A05 checks a
row-ID-indexed cache, otherwise loads 8md.A01 and caches the result
(000a..0074). 8md.A01 selects `is_pn_shared FROM lid_chat_state WHERE jid_row_id = ?`
(0010..0026); it returns false only for 0, true only for 1, and null for
other values, missing column or missing row (0028..0072). A05 similarly returns
null for its missing-ID branch. No missing value becomes false in the store
reader itself. When the fresh feature/contact check is false, in A0Z's non-PN LID branch
only an explicitly true Boolean
makes the local PN-inclusion flag true (0298..02d6). An originally PN target
retains its already true flag. That controls whether the CMn result is passed
as local PN (0312..0318). When the fresh feature/contact check is true, the flag instead becomes
whether the normalized own username is absent (0218..022c). That flag
feeds either constructor depending on the separate captured/contact gate;
A0U reads it at 024c..0252 and A0T at 0312..0318. PN-sharing privacy
state and username presence are distinct inputs; current values are unknown.
The private probe filename `fingerprint-input-username-enabled.smali` records
an earlier hypothesis, not a confirmed name; it is explicitly refuted here.

**Hosted-state authority:** J83.A1Y forwards to 0bo.A02 with its extra flag true
(0000..001c), connecting A0Z's per-side booleans to the previously traced
hosted-contact predicates. A0Z gates that lookup behind feature 0x1da3.
Both A0T and A0U additionally reread 0x1da3 and replace supplied state integers
with **-1** when disabled (0048..007e), before constructing params. Thus a
false default Boolean cannot be treated as confirmed E2EE state when this
feature is disabled. The native converter accepts -1/0/1, with absent versus
explicit enum presence already distinguished by the corpus. Upstream hosted
attestation, runtime feature values and complete account-sync authority remain
unverified; no value was synthesized to satisfy a comparison.

Reproduce using existing static commands:

```sh
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/JhJ;' --method A0Z
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class '' --method ATB
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class '' --method Aq3
target/debug/xtask apk-xref ~/Downloads/WhatsApp.apk --anchor self_user_name
```

Also inspect exact classes 1cg (A02), 0o5 (A00/A03), 0qq (A04/A05),
8md (A01), 0CM (A0p), 0Gj (A01), J83 (A1Y), 0jc (A0T/A0U) and 08i (CU3).
Full dumps stay ignored under `lab/apk-2.26.37.71/fingerprint-*`. The committed
snapshot contains attributed scoped instructions, no media or account values.

**Diagnosability correction:** the original smali formatter rendered negative
8/16-bit jumps as two's-complement hex after a literal plus sign. This made
A0Z's 02d0/02d4 jumps look forward; they are -82/-84 code units, both reaching
022c. The initial scheme interpretation was corrected before integration:
01ee's fresh check chooses PN inclusion; 023c's captured/contact gate chooses
A0U versus A0T. `apk-smali` now prints explicit signed magnitudes for goto,
goto/16, goto/32 and conditional branches. A literal APK backward-jump fixture
plus signed minima, forward conditional and truncated-prefix tests cover this
formatting correction; no instruction widths or protocol values changed.

Validation (offline): 556 workspace/all-feature tests and 244 default-feature
tests passed. Strict workspace/all-target/all-feature Clippy, warning-free
Rustdoc, formatting and CLI reference regeneration/test passed. Fresh APK
method output confirms both backward jumps and matches the scoped selection
excerpt. No account state was opened or changed and no live request was sent.
Repository hygiene passed with 339 local links; staged whitespace checks passed.

## Native V3 scan dispatch and comparison branches (2026-10-07)

**Confirmed/static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
reconstructed ARM64 `libwhatsapp.so` SHA-256
`f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a`.
[Scoped dispatch/converter/core evidence](fingerprint-v3-comparison-evidence.txt)
resolves the selector-19 and option-conversion unknowns in the earlier Java
comparison section. It does not implement V3 comparison or validate current
account inputs. Native addresses here are ELF virtual addresses; Java offsets
are method-relative bytes.

`KXj.A01` checks feature **0x1d2d** to choose native V4 versus this native V3
scan comparator. That is separate from generation's 0x1d2c captured/contact
gate. Do not assume generation and scan branch selection share one flag.
On the non-V4 branch it reads 0x1da3 (007c..0086), calls `3c8.A07`
(0088..008e), then ORs the result with 1 (0090..0094). The full `3c8.A07`
body returns **1 for zero, 2 for nonzero** (0000..000a). Thus the scan caller
passes option **1** with hosted feature off and **3** with it on. The native
core tests bit 0 for username checks and bit 1 for hosted checks; this caller
always enables username checks. Unknown runtime flags remain unknown.

**Confirmed/static dispatch:** `.rela.dyn` resolves JNI registration entry
`0xd06d88` to name `jvidispatchIIOO`, signature
`(IJLjava/lang/Object;Ljava/lang/Object;)J`, entry `0x6bc4b4`. Raw file-backed
pointer slots are zero before relocations; reading them as final pointers
produced no matching entry, and was corrected by using relocation addends.
The entry reads a uint16 jump table at `0x300df6`, whose selector-19 word is
`0x125`: base `0x6bc4f4 + 4*0x125` reaches `0x6bc988`. That branch unwraps
the known holder through `0x6b6f30`, scanned bytes through `0x6b7574`, copies
the low 32 bits of Java's long options into w2, and calls **0x71373c** at
`0x6bc9b8`. Its status is sign-extended to Java long at `0x6bc9bc`, then the
objects are released. Zero alone becomes success through the already traced
`1xm.A1P` and `KA1` result constructor.

**Confirmed/static wrapper, 0x71373c..0x713938:** nonpositive byte length
aborts, decode failure returns -8, missing version -5, version outside {0,1}
-10, and a missing individual holder -2. It converts scanned local first,
remote second through **0x713e00**, returning their conversion errors before
orientation. Unlike V4, this wrapper does not pre-require both LID fields.
The orientation helper **0x71401c** compares known local with each scanned
side: first their non-null PN values from getter `0x6c1e4c` (+0x28), then
non-null retained LID objects from `0x6e1814` (+0x30), using `MCFEqual`.
Either match suffices. If both sides match, local-first wins. Local-first
calls core **0x7140fc** then remaps through **0x7140b8**; remote-first swaps
scanned sides and calls the core without remapping. Neither match returns
-13. Object-construction failure with a zero conversion-error slot is an
explicit abort, not success.

**Confirmed/static individual converter, 0x713e00..0x71401c:** optional PN
ASCII creation fails with -7. Optional LID ASCII creation fails with -1;
its subsequent JID parse `0x71dfa4` and stable-LID conversion `0x71f2b8`
fail with -6. Optional username ASCII creation fails with -23. After these
operations, both PN and LID absent is -3, even when username exists. Present
hosted state outside {0,1} is -38; absence becomes -1. The version-selected
key field must be present (-4 otherwise): public_key for version 0,
hashed_public_key for 1. Constructor `0x718ffc` stores public/hash data at
+0x18/+0x20, PN at +0x28, retained LID at +0x30, username at +0x38 and hosted
integer at +0x40. Full JID/stable-LID acceptance and JNI normalization are
not recovered by these calls alone; no synthetic LID should be invented to
make conversion succeed.

**Confirmed/static core order, 0x7140fc..0x714358:** known remote must match
scanned remote by the same PN-or-LID helper (-12 otherwise). Version-selected
key equality then checks known local (-14) before known remote (-15), through
the shared data comparators `0x714358`/`0x7143b0`. PN comparison follows local
then remote, even after an orientation match by LID. The shared optional-value
helper **0x714450** yields 0 equal/both absent, 1 unequal, 2 known present /
scanned absent, 3 known absent / scanned present. Its equality calls
`MCFEqual`; it does not call the V4 PN/username helpers.

| Check | Unequal | Known present, scanned absent | Known absent, scanned present |
| --- | --- | --- | --- |
| Local PN (`0x714408`, side 0) | -17 | 0 (allowed) | -17 |
| Remote PN (`0x714408`, side 1) | -19 | -21 | -22 |
| Local username, options bit 0 | -24 | -26 | -28 |
| Remote username, options bit 0 | -29 | -31 | -33 |

Both-absent username comparison returns zero in this V3 helper. The traced
V4 core has additional no-PN/no-username statuses and a separate case-folding
string helper; copying that behavior into V3 would change these branches.
**Unknown:** complete MCFEqual behavior for these ASCII strings and retained
JID classes, including case sensitivity and alias equality. Calls to MCFEqual
are not proof of byte equality or V4-compatible case folding.

Hosted checks run only with options bit 1. When both scanned hosted integers
are -1, core skips matching against known values and returns zero (with an
optional diagnostic log). Otherwise known local must equal scanned local
(-34), then known remote must equal scanned remote (-36). The shown core can
abort for invalid scanned hosted enums, but the wrapper's converter rejects
such present values first. Do not treat the legacy omission branch as proof
of hosted status or manufacture E2EE=0 for absent inputs.

**Confirmed/static remapper:** `0x7140b8` accepts indices `status+36` in
0..36 selected by mask `0x100dead5ad`, and loads int32 results from table
`0x302370`. Its full accepted map is in the scoped snapshot: -36→-37,
-34→-35, -31→-32, -29→-30, -26→-27, -24→-25, -21→-20, -19→-18,
-17→-16, -12→-11; -33, -28, -22, -15, -14, -13, -10, -9 and 0 are
unchanged. Other inputs abort. This range differs from the V4 remapper;
conversion errors bypass it. These are native local scan results, not server
response reasons, and zero does not authorize marking a peer verified.

Reproduce the Java option extraction with:

```sh
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/3c8;' --method A07
```

For native code use `llvm-objdump -d --start-address=VA --stop-address=END`
with entry range 0x6bc4b4..0x6bc9c4, wrapper 0x71373c..0x713938 and
converter/helper/core 0x713e00..0x7144a0. Apply `.rela.dyn` addends when reading
JNI pointer records, not zero raw slots. Existing tooling exposes these
static reproductions; no new runtime command, HTTP route or live capability
is claimed. Ignored raw dumps are `fingerprint-v3-comparison-*.txt` under
`lab/apk-2.26.37.71/native/` and the option smali under its parent.

**Unknown / next validation:** execute the actual V3 core/remapper over an
independent synthetic corpus, follow its MCFEqual/JID semantics, and trace the
stable-LID converter before implementing binary comparison. Active feature
values, authenticated inventory and a real peer scan remain unverified.
No account state was opened or changed, verification committed, or live
message transmitted in this trace.

Validation (static/offline): APK and reconstructed-library hashes matched the
pinned values; the option extraction returned one method. Independent fresh
native extraction matched all 424 converter/helper/core instructions in the
scoped snapshot. CLI reference regeneration/test passed; repository hygiene
passed with 344 local links and staged whitespace checks passed. This changes
research/docs only; native execution and live account validation are not claimed.

## Executed V3 core and remapper corpus (2026-10-07)

**Observed/offline native execution**, same APK 37.71 and reconstructed
`libwhatsapp.so` hash as the preceding V3 trace: the new
`tools/apk-research/native_v3_comparison_check.py --library PATH` runs **264**
built-in synthetic cases. [Committed native observations](fingerprint-native-v3-comparison-cases.json)
retain exact results and instruction/model-call counts. It accepts no account
or arbitrary scanned inputs. The shared native runner retains its V4 default;
V3 executes actual instructions at core `0x7140fc..0x714358`, identity helper
`0x71401c`, optional-value helper `0x714450`, PN-presence mapper `0x714408`
and result remapper `0x7140b8` over table `0x302370`.

Coverage includes key selection for versions 0/1, UI options 1/3, both sides'
PN/username differences and presence combinations, remote PN-or-LID matching,
local-orientation omission from this core, and explicit error precedence.
All **81** valid known/scanned hosted combinations execute for each UI option.
Two internal options 0/2 cases establish the username bit can be disabled in
this core; the APK UI caller passes 1/3, so these are not available UI settings.
One invalid scanned-local hosted integer reaches the explicit native abort;
real wrapper conversion rejects unknown enums first. The largest non-abort
case executes **227 instructions**, below the per-core/remap limit of one
million instructions / ten seconds. Each call must return to its designated
return PC. Wrong-build/missing-file/unmodeled-branch/budget failures produce
no partial JSON.

**Limits / unknown:** this executes native control flow over modeled getters,
key equality and generic `MCFEqual`, whose synthetic byte tokens model equal
versus unequal objects. It does not prove case sensitivity, real LID alias or
JID class equality, stable-LID normalization, constructor acceptance or
pointer ownership. The decoder, scanned-side converter, orientation wrapper,
JNI and verification action do not execute. The V4 runner's actual merged/
Bionic case-folding mode is explicitly refused for V3, since substituting it
would conceal the untraced generic-equality dependency. Runtime V3 binary
comparison remains unimplemented; current account inputs remain unknown.

The initial execution failed with an unmapped fetch before the modeled PN
getter at `0x6c1e4c`; the runner now maps its file-backed page in V3 mode.
This was a research-harness mapping omission, not an APK/protocol refusal.
Expectations come from the literal native branches and table in the preceding
static evidence, not from a client comparison port.

Validation: all **10** V3/shared-V4 tooling tests passed with main, merged and
public Bionic fixtures configured. V3's rerun exactly matched all 264 committed
observations. The existing 115 V4 core statuses/counts and 268 native ASCII
string cases remained unchanged, including the two V4 invariant aborts.
Missing/wrong-build refusals had empty stdout; unsupported shared-runner mode
and V3/V4 string-mode mixing refused before emulation. No saved account was
opened or changed and no network message was sent.

Final validation: CLI reference regeneration/test and OpenAPI library-symbol
check passed. Repository hygiene checked 349 local links; staged whitespace
checks passed. All ten V3/shared-V4 tests passed again after restricting hooks
to the selected scheme. Rust runtime code was unchanged; no live validation
is claimed for this offline research-tool change.

## V3 equality callbacks and primary-LID conversion (2026-10-07)

**Confirmed/static**, APK 2.26.37.71, APK/main-library hashes as in the
preceding V3 sections, merged-library SHA-256
`565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa`.
[Scoped instructions and relocation-resolved constants](fingerprint-v3-equality-lid-evidence.txt)
resolve the earlier generic-equality and stable-LID questions for these traced
classes. They do not execute the object runtime, parser or constructors.
This narrows the implementation requirements beyond the modeled V3 corpus.

**Confirmed/static generic dispatch:** merged `MCFEqual` at
`0x534f34..0x534fe8` returns true for identical pointers (including both
null), false for exactly one null or different object-header type IDs, and
otherwise resolves each class descriptor through `0x534e8c`. Different resolved
descriptors or a missing descriptor abort; a missing equality callback returns
false. An existing callback at descriptor +0x28 receives both objects.
The V3 optional-value helper checks null presence before calling MCFEqual;
therefore the corpus's modeled non-null comparisons did not establish this
whole generic dispatch or need a null/null MCFEqual call.

**Confirmed/static ASCII class:** `MCFASCIIStringGetTypeID` at
`0x530fc4..0x531000` registers descriptor `0x74ee18`. Its +0x28 relocation
resolves to callback **0x53102c**, which checks/casts both objects to that class
then tail-calls **MCFASCIIStringEqual, 0x5316ac..0x5316f4**. That function
compares stored lengths at +0x18; differing lengths return false. Equal lengths
invoke `memcmp` over pointers at +0x20 with the exact length, returning true
only for zero. This is **case-sensitive byte equality**, with no lowercase,
trimming, prefix removal, parsing or Unicode normalization in this callback.
PN and username fields reach this class through the traced ASCII construction/
copy paths, so V3's generic calls do not inherit V4's case-folding username
rule. The result is a property of this build's class callback, not a claim that
all objects passed to generic MCFEqual are ASCII strings.

**Confirmed/static JID class:** main registration `0x71ecc0..0x71ecfc` uses
descriptor `0xd0afd8`; +0x28 relocates to **0x71eda4**. That callback casts
both JID objects, reads their stored strings at +0x20, and tail-calls MCFEqual
on those strings (`0x71edd8..0x71edf4`). The constructor **0x71de28** formats
user plus the selected server with literal `%s@%s` at string VA `0xce7ce`,
then copies that full ASCII result into +0x20 (`0x71de88..0x71de98`).
It separately stores the user at +0x28 and server kind at +0x30. Equality thus
uses the constructed full string, not just user bytes, not display LID-user
normalization, and not an implicit PN-to-LID mapping. Parsing may change which
string gets constructed; comparing raw scan bytes without following it can
still be wrong even with the correct callback.

**Confirmed/static primary parse:** V3's scanned individual converter first
ASCII-creates its supplied LID, then calls **0x71dfa4**. That calls the shared
parser `0x71f30c` with option 0, applies **0x71ebd8**, and accepts only resulting
JID objects whose +0x18 category is zero. Parse failure records internal error
-1; a parsed but non-primary category records -2. Both cause the individual
converter's -6, rather than exposing the parser's internal error directly.
Afterwards **0x71f2b8** requires the typed server kind returned by **0x71efe4**
to equal **5** and otherwise returns null. It returns the same pointer on
success: this last function itself does not rewrite or normalize a JID.
The typed getter allows kinds {0,4,5,7,9,10} and aborts outside its mask;
these are internal JID kinds, distinct from protobuf HostedState {0,1}.

**Confirmed/static LID routing:** `0x71f30c` separates the supplied string
using the literal `@` constant (relocation-resolved at `0xd0af98`). Exactly
two pieces pass to user/server parsing **0x71f4e0**; one piece follows a
separate server-only path, other counts fail. Server classifier **0x71fa08**
checks length, third character and exact MCFEqual against cached constants;
`0x71fb20..0x71fb3c` compares against the `lid` constant from **0x71e438**
and produces server kind 5. The user/server jump table at `0x3039fd` routes
kind 5 to **0x71f760**. This confirms the literal lowercase `lid` branch;
no case-insensitive or arbitrary-server acceptance is inferred.

With parser option 0, that branch jumps directly to **0x71f860**, validates
user through **0x7204bc**, decomposes it through **0x720d1c / 0x720bac**,
and constructs a device-LID object through **0x71da6c**. The validator's
separator is the literal `:` (`0x71e978`, constant pointer record `0xd0afb8`).
With exactly two pieces it validates base number via **0x71fdc0** and suffix
via **0x71fe0c**; otherwise it validates the full user as a base number.
These are the traced static bounds, not a general registration-number rule:

| Input part | Branch requirements |
| --- | --- |
| Base number | `0x71fd24` accepts length 1..20, first character `1`..`9`, remaining characters `0`..`9` |
| LID-specific base | `0x71fdc0` additionally rejects lengths 16..19 (`length & ~3 == 16`) |
| Device suffix | `0x71fe0c` accepts one or two digits `0`..`9`, including zero |

The parser option-1 shortcut (`0x71f764..0x71f79c`) was traced but is **not**
the option supplied by V3's primary parse. Do not replace the option-0 route
with the simpler constructor merely because it yields the same ordinary case.
The shared decomposer also knows a literal `.` separator (`0xd0afa8`), but
this LID validator rejects a dotted user before reaching it. Its broader
non-LID callers were not recovered by this trace.

**Confirmed/static decomposition/construction:** `0x720bac` obtains a base
string and device suffix, defaults to the literal `0` (`0x71eac8`, record
`0xd0af88`) when absent, parses suffix decimal with `strtol`, and returns the
base pointer plus integer. `0x71da6c` requires that integer in 0..99 and calls
**0x71d944** with server kind 5. That initializes category +0x18 to **5**,
formats the device user with `%s:%d` (VA `0x1347aa`), stores original base at
+0x38, and device integer at +0x40. It then uses the ordinary full-string
constructor for +0x20. Suffix formatting normalizes its decimal spelling;
this is not authority to manufacture a companion device or identity.

**Confirmed/static flattening:** `0x71ebd8` preserves ordinary objects and
nonzero-device objects. For device-category 5, server kind 5, device integer
zero, it reads original base +0x38 and invokes **0x71f2a4**, which constructs
category 0 with kind 5. Thus a plain valid `base@lid` goes through a temporary
device-zero object and becomes a primary `base@lid`. An explicit accepted
zero suffix also becomes primary. A nonzero device remains category 5 and
is rejected by the following primary-category guard in `0x71dfa4`.
**Inferred from these branches, not executed end-to-end:** valid synthetic
`base:0@lid` and `base:00@lid` converge to the same primary string, while
nonzero companion suffixes yield the converter's -6. Full decimal/array/
allocation behavior still needs actual native parser execution before a
runtime comparator should adopt these examples as fixtures.

**Negative / limits:** the final stable-kind gate is not the normalizer;
normalization happens earlier in the primary parse. Generic equality is not
universally byte equality; the class registration is the evidence for these
strings. The existing native V3 corpus still models object construction and
equality and accepts synthetic tokens such as `synthetic-local`; those are
control-flow fixtures, not valid LID-converter inputs. No claim is made for
all MCF classes, unrelated server kinds, allocation failure, runtime class
registry state, current authenticated inventories or active feature flags.
Full binary V3 comparison is still unimplemented. Account state was not
opened or changed, verification was not marked and no live request was sent.

Reproduce with existing `llvm-objdump -d --start-address=VA --stop-address=END`
using the scoped range labels. Use `.rela.dyn` addends for the class callback
and constant records; their raw pointer slots are zero before relocation.
The scoped snapshot carries all source/build addresses without private data.
Ignored full dumps remain under `lab/apk-2.26.37.71/native/` with prefix
`fingerprint-v3-`. No new command, runtime field or HTTP route is introduced.

Validation (static/offline): APK, main and merged hashes matched the pinned
build. Fresh extraction matched all 18 ASCII comparator instructions. CLI
reference regeneration/test passed, repository hygiene checked 354 local
links and staged whitespace checks passed. This change adds attributed research
only; it does not execute the native parser or change runtime/account behavior.
