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
owns these routines. The archive format, unpacked member selection, native
algorithm, modern QR payloads and native comparison remain unknown.

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
