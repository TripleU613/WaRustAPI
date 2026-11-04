# Current patch-header field evidence

Static WhatsApp Android **2.26.43.72**, actual APK SHA-256 independently rechecked:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Generated on2026-11-04 using the repository's existing `xtask`, source HEAD
`136fc381a34b3756052ce7e66b51c412061286f3` (no xtask source edits).
[Audit and next-stage proposal](../../app-state-patch-header-audit.md).

These are actual selected checked DEX/descriptor exports, **not** REA Evidence
JSON envelopes, so no artificial evidence IDs or ownership sidecars are added.
The descriptor extractor follows `newMessageInfo` object-array helper bodies;
field numbers do not come from literal field order or a public schema.

| Artifact | Reproduction selector | SHA-256 |
| --- | --- | --- |
| [patch-fields.txt](patch-fields.txt) | `xtask apk-proto-fields APK --class 'LX/Dh0;'` | `75f5995276b6fd9a88df22afa0efbf208e67ab4a8d2e6117e0657aa47ea38912` |
| [version-fields.txt](version-fields.txt) | `xtask apk-proto-fields APK --class 'LX/DYK;'` | `ccce62b086315be230189b921fa2b83f06932ba9b08484be8ed0467f7bb65eee` |
| [exit-fields.txt](exit-fields.txt) | `xtask apk-proto-fields APK --class 'LX/Dba;'` | `4aff6802f2c001f83b47a9b42fe0d760445345a9cea5cd34df06337f503f629a` |
| [external-reference-fields.txt](external-reference-fields.txt) | `xtask apk-proto-fields APK --class 'LX/Dg5;'` | `d1e9effe588377c9b3aab93c54db9c1a82d6c2301569a875bedc38c6d8a1ccc8` |
| [mutation-fields.txt](mutation-fields.txt) | `xtask apk-proto-fields APK --class 'LX/Dbb;'` | `8299756cf33dd997071587de90cde07b1c6006ab816d6ee57471dd2eb9cf47fa` |
| [patch-descriptor-checked.txt](patch-descriptor-checked.txt) | `xtask apk-smali APK --class 'LX/Dh0;' --method dynamicMethod --signatures --exceptions` | `ebd2f5fd7416dd2799d27fa13e9e3637eab711bff93bf468cb86da0a71a06361` |
| [mutation-class-checked.txt](mutation-class-checked.txt) | `xtask apk-smali APK --class 'LX/Dbb;' --signatures --exceptions` | `b8389ab938a355b794f448d328edaf567a5a438fa733ff83fe275136e4e593f2` |

The existing [F2T checked bytecode](../2.26.43.72-sync-fetch-completion/response-checked.txt)
is reused, not re-exported: A00 offsets00e8..0230 establish all-header validation
before the returned collection model. Offset0052..0088 parses all selected
patch protobufs before those semantic checks. Full SHA-256 and original REA
records are retained in that bundle's README. Offsets here are method-relative
bytes. These exports establish pinned field/tag and parser facts, not runtime
execution, dynamic feature selection or authenticated mutation application.
