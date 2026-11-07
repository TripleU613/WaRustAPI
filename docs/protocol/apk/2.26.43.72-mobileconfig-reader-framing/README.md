# MobileConfig reader framing and hash-prefix comparison

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native reader execution, filesystem experiment or account/network operation.

## Shared construction reaches framing before table access

The [upgrade schema check](../2.26.43.72-mobileconfig-upgrade-dispatch/README.md)
constructs its reader through **0xa2a70**. Shared-reader helper **0x88144** also
calls that constructor, retaining the object in a shared allocation; it is not
a separate parser. Constructor 0xa08d0 selects a provider from flag-controlled
branches. The retained provider entry includes virtual and filesystem-backed
routes; all provider binding, readiness and ownership behavior is not resolved.

0xa2a70 installs reader address point **0xd61c0** and calls **0xa12bc**. A false
return skips its later table-pointer setup. A true return reaches root setup
0xa2bf4 and derived counters 0xa2d48. Full counter/category semantics remain
untraced. Provider existence, successful construction and a nonnull derived
pointer alone are not complete admission evidence.

0xa12bc first asks provider slot+0x20 whether input is ready, then obtains its
pointer via slot+0x10 and 32-bit size via slot+0x18. A false readiness outcome
assigns diagnostic `empty`. A null pointer or size below 2 follows the raw-buffer
handoff; it does not dereference the magic header on those selected branches.

## Framing gate is broader than the writer's canonical output

For a nonnull pointer with size at least 2, bytes **3c f1** select framing.
Otherwise the original pointer and size are passed to the body verifier.
Thus this reader supports a raw-buffer route; the header is not universally
mandatory. The framed route requires, in order:

| Check | Selected condition |
| --- | --- |
| Total size | At least 11 bytes |
| Version byte at +2 | Unsigned value <= 1 |
| u32 at +4 | Equals supplied total size |
| Last two bytes | Exactly `3c f1` |
| Body-offset byte at +3 | Strictly less than total size minus 2 |

It then supplies pointer+offset and length total−offset−2 to the verifier,
with additional defensive pointer/arithmetic handling retained in the code.
This selected framing gate does **not** require version exactly 1, offset
exactly 8, or a minimum offset of 8. The canonical
[writer](../2.26.43.72-mobileconfig-file-writer/README.md) emits version 1,
offset 8 and the same declared size/footer shape. A noncanonical offset passing
the envelope gate is not proof that the resulting body passes verification.

Framing failure records `verifyAndLoadBody`; body-verifier false records
`verifyBuffer`. The caller's return is based on diagnostic emptiness. With a
nonempty diagnostic, selected normal handling invokes provider slot+0x28 and
attempts removal through 0xc2310 using the provider string at +0x8, then returns
false. Optional logger calls are separate. The provider cleanup/removal outcome,
all exception paths and durable effects remain unresolved; this is not a
read-only rejection guarantee.

## Separate body verifier remains a distinct admission boundary

RELA **0xd61d8** binds reader slot+0x18 to **0xa2ee4**. Both raw and framed
routes invoke that selected virtual target. It requires body size at least
**12**, constructs verifier state, checks a root offset through 0x9ac7c and
continues into table/field/vector helper checks. The retained whole routine
contains many subsequent checks; their complete schema and helpers are not
modeled here. A late selected field test also compares to numeric 123456;
its field identity and relation to the earlier schema-check field remain
unproven. Do not equate numeric coincidence with the same field.

The inspector's three-byte synthetic body passes the canonical envelope shape
but is too short for this selected verifier. Its output deliberately says
**verifier handoff**, never native acceptance. This prevents a framing-only
projection from being mistaken for a valid configuration fixture.

## Previously unknown hash transformation is a colon prefix

Helper **0xa8fa4** resolves the earlier normalization question. Global 0x24a28
is a C++ short-string object (`02 3a 00`), representing `:`; it is not a plain
C-string starting at that address. Helper 0x9e308 searches the complete source
string from position zero through 0x5e804 (forward memchr/memcmp search). If a
colon is found, 0xa8fa4 copies the substring from zero up to its first position;
otherwise it copies the whole source. A leading colon produces an empty prefix.

Therefore the upper result contribution in the schema helper compares **prefixes
before the first colon**, while its separate lower logic can still compare raw
strings. For example, `abc:v1` and `abc:v2` have the same selected prefix. This
does not establish the suffix's protocol meaning, permit its omission on the
wire, or justify treating all schema/hash checks as equivalent.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-reader-framing/inspect-reader.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-reader.json
cmp /tmp/mc-reader.json docs/protocol/apk/2.26.43.72-mobileconfig-reader-framing/reader.json
```

The inspector pins member/ELF identity, strings, short-string bytes, selected
calls and verifier RELA binding. Synthetic envelope/prefix cases are analyst
projections, not executions of the parser or verifier. LLVM retains the gate,
reader construction and body verifier, plus bounded provider/search windows.
Nearest JNI_OnLoad headings are not local identities; full provider and nested
verifier semantics are not claimed.

rea-reader.json retains complete REA Evidence
`ev_a410b3282bc7685c4bcd392c74f9c07f9c90d81197d1cee672422234eefc653b`,
command `e scr.color=0; pd 32 @ 0xa14dc`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. It is a bounded framing window;
provider version/external inputs are not bound and annotations are not runtime
memory observations.

**Remaining:** complete provider identity/lifecycle, all verifier helpers and
table schema, rejection cleanup effects, upgrade/installation bodies, durable
active-state admission and retry/account ownership. No Rust runtime or saved
account state changed.
