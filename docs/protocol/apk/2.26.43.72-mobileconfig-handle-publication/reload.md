# Reloading the latest cache on disk

**Confirmed selected static evidence**, same pinned APK/native hash as README.
No native execution or account/network operations. StorageManager slot+0x20,
RELA **0xd55a0 -> 0x83a04**, has diagnostic
`FBMobileConfigStorageManager::reloadLatestCacheOnDisk` at **0x18bc7**.

The selected normal body obtains a string plus word+0x18 using virtual+0xe0
(RELA 0xd5660 -> 0x878e0). Empty string returns before snapshot retrieval.
0x878e0 derives a path through 0x88888 and invokes 0xc310c; the latter uses a
directory visitor and emits a string plus selected integer. The [selection follow-up](selection.md) establishes a greatest qualifying
numeric filename-prefix policy, not a latest-by-time policy; enumeration and
parser edge cases remain bounded.

For a nonempty string, reload obtains the shared snapshot via virtual+0x40,
selected 0x83c7c. A nonnull handle's string+0x28 is compared with that string
using 0x531f8. Equality joins cleanup with no translation or republication.
Null or unequal handle enters schema helper 0x8bda0.

The branch uses **bit 0 of the helper result when manager byte+0x120 is zero**,
but **bit 8 when that byte is nonzero** (not specifically equal to one).
Earlier upgrade-dispatch evidence describes the helper's bit-bearing result;
it is not a plain success enum. The selected reload behavior is:

| Selected bit | Additional condition | Behavior |
| --- | --- | --- |
| Clear | None | Store selected integer in manager+0x18; construct result with arguments (0,5,empty string); call virtual+0xe8, selected 0x87940 |
| Set | byte+0x268 bit 0 clear and word+0x26c neither 2 nor 3 | Cleanup, no translation or republication |
| Set | byte+0x268 bit 0 set or word+0x26c is 2 or 3 | Derive path with 0x88048 using selected integer; call 0x8bfe0 with that path and integer |

The mode test is `(word & ~1) == 2`. A false 0x8bfe0 result logs
`TranslationTableParamsParserError` (**0x174b7**) with the above method name
and `buildLatestConfigWithTranslationTable failed` (**0x1a90d**), then returns
through cleanup. A true result also joins cleanup. The reload body does not
call publication after either translation result; effects inside 0x8bfe0 remain
separate. Only the clear-selected-bit branch explicitly invokes publication.

The snapshot's status at output+0x10 (frame x29-0x38) is not read by this body.
It uses the handle string and the separate selected integer (x29-0x18).
Shared snapshot ownership is released at 0x83bc0. There is no mutex acquisition
in this selected method body; caller synchronization remains unknown.

**Unknown:** directory selection completeness, translation's full semantics,
all callers, concrete logger effects and final application acceptance. This
trace does not establish server-required telemetry or a Rust runtime correction.

Reproduction: retained reload-body.txt (0x83a04..0x83bf8),
reload-source.txt (0x878e0..0x87940), reload-selector.txt
(0xc310c..0xc31d4), reload-slots.txt and reload-data.json.
REA evidence `ev_a09768f945fbcbcac8d1afb705a3d4dd17053655ede56755916fc9e6f9637f4e` covers the selected schema/translation branch;
the complete bounded body is retained separately.
