# Dirty blocklist fetch: admission, adoption and account role

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX helpers and [complete REA A0H Evidence](../../rea/2.26.43.72-blocklist-dirty-role-admission/README.md).
No Rust behavior, account/device/APK execution or live request changed.

## Reuse the callback and separate its inputs

The retained [6p9.CK4](../2.26.43.72-blocklist-set-reconciliation/fetch-success.txt)
reads list.dirty and compares exactly to lowercase true before parsing6qD.A02.
Whitespace/case alternatives are not normalized in that selected comparison.
A no-list response exits through unchanged/receive-time completion before the
list/admission/dirty path. Malformed selected list parsing precedes admission.

For a parsed list, it queries0Gs.A0D through6pA.A06. A true readiness
result reaches mode/admission checks; false branches directly to the adoption
call setup at00f4 (00a2 +0x29 code units), bypassing those checks. It then reads3BU.A01 blocklist addressing mode;
if already true it jumps to snapshot adoption. Otherwise it calls2LA.A1S on
6pA.A03, which delegates0f5.A0H. A true result sets3BU true before adoption;
a false result reaches the selected invalid/unmigrated delivery-failure path.
Do not read the mode-write branch as proof every list has been migrated.

The later6pA.A05 gate is **not a remote feature flag**. Its constructor binds
A05 through2L7.A0B ->05d(0xed).2LE.A1T unwraps09F and callsBU7. The
[retained physical declarations and device-ID predicate](../2.26.43.72-notification-handler-census/README.md)
show09G/09O.BU7 ->09O.BU8(false): initialize, read registration_device_id with
default0, test >0. Existing [role evidence](../../server-sync-role-recovery.md)
binds true to companion mode. Fresh all-class BU7 exports match the retained
three declarations. This gate is distinct from whether any companion exists,
registration completion, LID mode and global chat migration.

## Chat-migration Boolean has initialization/readiness boundaries

6pA.A03 is bound to05d(0x434), unwrapped as0f5 by2LA.A1S.0f5.A0H checks its
0HP.A00 cached Boolean. If null it queries0Gs.A0E andA0D; when that selected
A0D is false it returns true early. Otherwise it may log a main-thread loading
diagnostic, then calls0HP.A01. With nonnull cache it directly callsA01.
Do not assume this internal readiness result is an observed equivalent of the
callback's earlier readiness query across providers/races.

0HP.A01 checks cachedA00. Null triggers00L.A0C(false,diagnostic), reports
msgStore-not-ready and stores Boolean true **if those calls return**; then it
checks nonnull and unboxes. Nonnull uses the cached value. No SQL query is shown
in this method. Assertions/diagnostics may affect control flow; no universal
runtime success or safe fallback is proven. Cache writers, reset, account
binding and effective current value remain untraced. Neither a PN/LID mapping
nor Rust's primary_login payload establishes this native authority.

## Selected branch table

Assume selected parsing and invoked methods return normally. R is the callback's
0Gs.A0D; M is3BU mode; G is0f5.A0H result (read only if needed).

| Direct list | R | M | G | Selected effect |
| --- | --- | --- | --- | --- |
| absent | any | unread | unread | unchanged completion; no dirty task |
| present | false | unread | unread | invoke adoption directly, bypass mode/migration checks |
| present | true | true | unread | adopt parsed snapshot |
| present | true | false | true | set mode true, then adopt parsed snapshot |
| present | true | false | false | invalid/unmigrated delivery path; no snapshot adoption shown |

After admitted adoption via1dH.A0O, CK4 builds a set/hash/association-key input.
Only exactdirty=true **and BU7=false** enqueue8bU selector1. BU7=true or a
non-true dirty value returns without that task. This is a post-adoption
obligation; it is not authority to pretend the snapshot failed or resend the
original fetch/block/unblock. The
[retained dirty worker](../2.26.43.72-blocklist-migration-obligation/README.md)
resolves mappings and eventually requests MigrateBlocklistLid with dirty_ack,
not an original per-contact set replay. Runtime executor/error completion and
full migration adoption remain separate.

## Rust consequence

Rust currently adopts supported parsed full lists and clears its local
reconciliation marker without equivalent native role/migration/readiness
providers. It does not execute this dirty migration request. This trace exposes
an unresolved post-adoption obligation on the selected primary-role path, not
proof every dirty list always requires the same action or every cached snapshot
is native-current. No unconditional warning/mode toggle, fabricated role,
automatic migration/refetch or guessed failure is introduced. A correct adapter
must represent proven authorities and post-adoption work separately from plain
membership/hash projection. The broad audit remains incomplete.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-dirty-role-admission-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/6pA;' --method '<init>' --signatures --exceptions > "$OUT/manager-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2LE;' --method A1T --signatures --exceptions > "$OUT/role-bridge.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2L7;' --method A0B --signatures --exceptions > "$OUT/role-provider.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2LA;' --method A1S --signatures --exceptions > "$OUT/migration-bridge.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0f5;' --method A0H --signatures --exceptions > "$OUT/migration-admission.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0HP;' --method A01 --signatures --exceptions > "$OUT/migration-state.txt"
target/debug/xtask apk-smali "$APK" --class '' --method BU7 --signatures --exceptions > "$OUT/role-bu7-definitions.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/09O;' --method BU8 --signatures --exceptions > "$OUT/role-device-id-predicate.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6p9;' --method CK4 --signatures --exceptions > "$OUT/fetch-success.txt"
```

Compare last three to linked retained files. checksums.json binds every other
bundle file. No duplicate role/callback DEX copy, live acceptance or ban-rate claim.

The callback offsets are byte offsets, but relative branch operands are signed
DEX code units (multiply by2). Verify the decisive destinations explicitly;
0054 is the common metrics-completion tail before return, not a dirty worker:

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
text = Path('/tmp/blocklist-dirty-role-admission-repro/fetch-success.txt').read_text()
for source, target in [(0xa2, 0xf4), (0xc2, 0xf4), (0xd6, 0x17e),
                       (0x17e, 0x204), (0x132, 0x54), (0x142, 0x54)]:
    line = next(line for line in text.splitlines()
                if line.startswith(f'    {source:04x}..'))
    match = re.search(r'([+-])0x([0-9a-f]+)$', line)
    assert match, line
    direction = 1 if match[1] == '+' else -1
    assert source + direction * int(match[2], 16) * 2 == target, line
print('Six branch destinations match')
PYCODE
```
