# Blocklist snapshot child admission — APK 2.26.43.72

Confirmed selected static, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports/catches plus
[complete REA parser Evidence](../../rea/2.26.43.72-blocklist-snapshot-children/README.md).
No real blocklist, account, fetch or APK execution.

## Every list child is required to be item

6qD.A02 iterates every0Tp.A02 child at0030..0044 and calls0Tp.A01(child,item)
at0044..004a. It does not filter unknown tags out of that iteration.0Tp.A01
usesA02: nonnull node plus description String.equals(expected). Mismatch
constructs and throwsLIL; it cannot yield a partial/empty successful result.
All parser writes shown before its return target local maps, not the shared
block set. The caller's later state-adoption policy remains separate.

The parser tries typed UserJid jid, then typed PhoneUserJid pn_jid when the
first is null. It defaults action to block and active to false; exact equality
to block produces the stored Boolean.80Z's constructor stores that map and
buildsA03 from nonnull keys with true values, with checked2L6.A1b Boolean
unboxing. No children therefore produces a legitimate empty set. Duplicate
map-key writes use ordinary put, so native last-write behavior is distinct
from this adapter's explicit duplicate refusal; no broader parity is claimed.

Conditional display-name/username and PN-to-LID association maps are visible in
the full method. Their classifier domains, migration/adoption and alias loading
are not newly reconstructed. Earlier [set/fetch callbacks](../2.26.43.72-blocklist-set-reconciliation/README.md)
connect the parsed result to selected reconciliation/shared-set paths.

## Rust correction and regression

apply_fetch_in previously filtered child descriptions to item, silently
ignoring everything else. An unknown-only list could become an empty cache;
a valid prefix plus unknown child could become a partial cache. Both could
advance dhash, commit its marker and clear reconciliation_required.

The loop now requires each child description to equal item and returns the
fixed InvalidInput(blocklist) reason unexpected blocklist snapshot child.
No untrusted tag/address is interpolated. Validation completes before cache,
hash, marker or reconciliation writes can commit; rejection preserves all
checkpoint bytes and the journal. Valid empty/item snapshots still replace the
cache and clear reconciliation as before. No query, retry or state reset added.

Regression snapshot_rejects_unknown_children_without_erasing_cache_or_reconciliation_after_reopen
failed before the guard because the malformed snapshot returned applied=1.
It covers unknown-only, unknown-first, unknown-last, nested-list, valid-empty
and valid-item controls with real encrypted stores and synthetic settled
replies. Invalid cases preserve all checkpoints/journal and absent marker,
including after reopen; the cached list/hash and prior reconciliation warning
remain intact. All modes verify no pending request is recreated.

Limits: malformed saved snapshots remain unprojected and ingest refuses them
again; no quarantine, automatic skip/refetch or history migration is added.
Full native alias/metadata/migration policy and already projected historical
bad snapshots remain unported. A later supported full snapshot is not invented
or transmitted. The goal remains broader than this admission correction.

## Reproduction

From repository root, pinned APK and current built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-snapshot-children-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/6qD;' --method 'A02' --signatures --exceptions > "$OUT/parser.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method 'A01' --signatures --exceptions > "$OUT/require-tag.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method 'A02' --signatures --exceptions > "$OUT/tag-equals.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/80Z;' --method '<init>' --signatures --exceptions > "$OUT/parsed-result.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2L6;' --method 'A1b' --signatures --exceptions > "$OUT/boolean.txt"
```

checksums.json binds every other bundle file by bytes/SHA256. Static evidence
and local tests do not prove live server behavior or zero account restrictions.
