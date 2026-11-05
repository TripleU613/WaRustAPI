# Token MobileConfig specifiers and a readiness transition

**Confirmed static**, APK 2.26.43.72 code 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No execution, account operations or claimed REA Evidence envelope. This closes
specific gaps from [bucket configuration](../2.26.43.72-token-bucket-configuration/README.md)
and reuses the [integer adapter/precedence trace](../../privacy-mex-config.md#effective-integer-configuration-has-two-authorities).

## Concrete lookup and nonzero token mappings

A local static scan of class_def_item interfaces lists 0A8 as the only declared
0A7 implementation in this APK. implementations.json retains that bounded
inventory including DEX name, superclass and interface descriptors. It does
not exclude reflection, external implementations or runtime replacements.
The earlier superclass-only override tool cannot establish this inventory.
0A8.B8Q loads 0El.A00, lazily initializes via A0L, returns the indexed long
for a nonnegative in-range property, and returns zero outside the array.
A0L allocates 41080 longs under a monitor and writes 2065 literal entries.

| AB property | MobileConfig specifier | Value instruction |
| --- | --- | --- |
| 865 | 0x8201a700000065 | 1ddc |
| 909 | 0x8201bc0000006e | 1f56 |
| 996 | 0x8201e30000007d | 2214 |
| 997 | 0x8201e40000007e | 2226 |

All are nonzero. The adapters' missing-specifier zero branch therefore cannot
be selected merely because a token property is absent from a raw AB reply,
when this concrete initialized lookup is used. These longs identify SDK reads;
they are **not** the effective integer values, defaults, timestamps or durations.
Retain them as hex strings: their integers exceed JavaScript's exact range.
The APK field-write inventory finds only 0El.A0L assigning A00. This does not
prove the exposed shared array can never be mutated through an alias.

`derive-specifiers.py` checks all 2065 literal aput-wide sequences, rejects
changed coverage, duplicate/out-of-bound indices and changed token entries,
then reproduces specifiers.json from the retained full method. It is a pinned
static derivation, not a general interpreter or runtime observation.

## DI binding and readiness are no longer completely unknown

The existing 00D.A02/A03 trace resolves ID66 through 1Uw.A00. Its upper-bit
router sends 66 and 252 to A01; A01 masks low 14 bits. Selected original
switch/case lines show 66 -> new08Y and 252 -> new0A8. Thus the case66
configuration owner is now closed for this build. 08Y's previously retained
constructor requests ID252 and supplies it as 0A7 to the 00E superclass.
provider-router is complete; provider-cases is explicitly an excerpt.

0A8.BPZ is retained whole. Its 075e switch sends literal cases12/13/14 to
07ce, evaluates SDK property32343 through 0Ei.A04, obtains08Y, sets its
00E.A08 AtomicBoolean true and calls A0t. A0t resets the SDK cache A0d when
A0c is enabled. This closes **one** static ready-state producer and cache
transition. It does not establish all callers, universe semantics, success
prerequisites for reaching that branch, cancellation, or all later readiness
changes. Constructor false is not proof that native operation stays legacy.
No Rust mode or effective token setting is selected from this observation.

## Parameter assets and remaining boundary

The four packaged assets named params_map_v4_u{1,2,4,6}.txt are binary despite
the suffix: each begins bytes76 34 and a binary header. parameter-assets.json
records full identities and 30-byte prefixes only. Raw files are not committed.
BPZ includes versioned parameter-file handling. Inventory/prefixes alone do
not decode defaults, row format, active universe or effective SDK values.
The native SDK reader, selected parameter-map semantics, runtime configuration
hydration, effective readiness/migration state and complete legacy application
remain open. Full configured token scheduling/admission is still unported.
This follow-up changes evidence and the next trace target, not send behavior.

## Reproduction and verification

Existing `target/debug/xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions` exports 0A8.B8Q/BPZ, 0El.A0L, 1Uw.A00/A01 and
00E.A0t. Use apk-field-refs with class0El, fieldA00, type[J, --writes-only
for the assignment scan. The interface inventory reads each DEX class_def_item
at header class_defs_off with 32-byte entries, resolves interfaces_off+12's
u32 length/u16 type indices, and closes superclass/interface reachability from
LX/0A7;. No target code runs. APK-member hashing is ordinary ZIP read-only work.

`python3 derive-specifiers.py` must equal specifiers.json byte for byte. Full
map retention permits replay without the APK; checksums cover every other file.
