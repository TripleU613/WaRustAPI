# Core protocol release review: 37.70 to 37.71

**Confirmed static:** old committed APK snapshot 2.26.37.70 SHA-256
`b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732`;
available APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [old recipe](apk/recipes/2.26.37.70-core.json) records the 21 existing
snapshot topics and affected Rust paths. Its identifying constants were checked
against the existing topic files. The [new reviewed recipe](apk/recipes/2.26.37.71-core.json)
adds the checked token initializer and re-anchors changed class names.

[Raw relocation evidence](apk/remap-evidence/2.26.37.70-to-37.71-core.json)
finds fourteen unique topics, three ambiguous topics and four unanchored
helpers. A requested automatic proposal exited 1 and created no file. Adding
`(byte)0x` to the integer topic did **not** disambiguate it; that negative is
preserved in the report. There is no old-name fallback or automatic successful
proposal claim. The seven remaining selections needed the following review:

| Topic | Reviewed 37.71 source and reason |
| --- | --- |
| certificate-root | Dev.<clinit> 0000..003c builds the 32-byte WhatsAppLongTerm1 map; 1N7.A03 reads Dev.A00 at 038a and 04fc. The same name also appears in the handshake consumer, explaining the ambiguity. |
| integer-encoding | 1Nj.A00/A01 call 1Lz.A02 when building a 12-byte nonce (0016..0026 / 0018..0028); 1Lz remains the integer utility. Alternative Cjj.A02 shares the diagnostic/debug formatting strings but splits an input into three byte arrays. |
| noise-config | 19p.A05 0000..00aa reads feature configuration 0x86e8 then selects an integer mode; alternative 1NF.A01 converts an integer mode to a display string. Identical strings do not identify the producer. |
| noise-record-state | Anchored 1N7.A03 produces 1QR through 1NZ.A01 at 06b8; 1N7.A05/A06 read its A09 field and construct reader/writer adapters. |
| noise-cipher | 1QR.<init> 0026..0042 constructs two 1Nj instances from the supplied directional keys; A00/A01 have opposite cipher modes and explicit counter nonces. |
| record-writer | 1N7.A06 000e..0018 constructs 1Qc with frame output and the same 1QR state. Its constructor stores both, with no identifying strings. |
| registration-requests | Checked HrU.A0D String field references find HUG.<clinit> at classes9.dex 0x35c260 and the producer itself. HUG's complete initializer republishes all 17 path fields, matching the old HSP role without selecting it by name. |

The source topics preserve the listed methods. All renamed selectors are pinned
to the new hash. Unanchored/ambiguous topics will still block automatic recipe
writing for a future release until their dependencies are reviewed or new
supported constraints are added. Reflection, native dispatch and runtime
feature selection are **unknown**; these selections establish static roles,
not complete port equivalence.

Dev's root bytes are identical to the old Aw1 array. 1N7.A04 still constructs
`WA 06 03` in its version-six branch, with the same `WA 05 03` fallback literal.
1N7.A03 tests intermediate `issuerSerial_` for zero at 0498..04a0 and obtains
WhatsAppLongTerm1 at 04fc..0510. These selected constants agree with existing
Rust comparison behavior. Hybrid KEM/resume and all certificate branches are
not fully ported or equivalence-tested here.

The new baseline has 27 files, 731,482 bytes and 150 methods in 22 topics.
The [cached diff](apk/37.70-to-37.71-core-diff.json) reports one added,
nineteen changed and two unchanged topics. Real comparison took 0.0512 seconds.
Changed text includes obfuscated symbol churn and improved instruction operands;
the old APK cannot be regenerated with today's decoder. Do **not** treat every
changed file as an upstream behavioral change. One substantive result of review
was a bug in **our** token extractor/runtime fixture, documented in
[token extraction](token-extraction.md), rather than a server protocol change.
All source files map to Rust ownership paths for subsequent review/patching.
Full APK coverage, full control-flow/native recovery and next-release validation
remain unfinished.
