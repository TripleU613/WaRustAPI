# Blocklist metadata adoption and existing-value policy

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No live/account/device execution. [Complete REA worker Evidence](../../rea/2.26.43.72-blocklist-metadata-adoption/README.md).
This extends [typed parser/priority evidence](../2.26.43.72-blocklist-identity-metadata/README.md),
not a new sender or claim that Rust already adopts native metadata correctly.

## Scheduling and map binding

Fresh1dH.A0O matches the retained
[complete replacement method](../2.26.43.72-blocklist-set-reconciliation/replace-set.txt).
At011c..0130 it schedules8be(this,878,5) throughEeV.A01, before the membership
no-difference branch.8be.run case5 reaches7to.A00(878). task-selected.txt retains
the exact dispatch header and complete selected case, not unrelated task bodies.
Scheduler failure/ordering and cross-store atomicity are not established.

878's three-map constructor binds parser arguments as follows:
A01 = PN-to-LID/active associations; A02 = username map; A00 = display-name map.
Those assignments matter: A02 is not the display-name map. Default constructor
supplies three empty maps.7to.A00 consumes them in that order.

## Association selection and submission

The worker queries0Ws.A0S for existing associations by PN key, sorts each
supplied candidate list with its stored comparator, and selects the first.
Empty candidate lists log/skip. Its constructor binds comparator8c3 case0:
active=true sorts before false, then Java String.compareTo on Jid.user breaks
same-active ties (lexical, not numeric). compare-selected.txt contains the
selected complete case and dispatch header.

If the selected candidate is active or the existing association is null, the
selected LID becomes chosen; otherwise the existing association remains chosen.
The worker adds every candidate unequal to chosen to its submission list, then
adds chosen last, paired with the PN. It submits that list to0Ws.A0Q. The meaning
of list order at the concrete writer, validation, events and transaction outcomes
are not traced here. This is not authority to install an arbitrary alias or merge
Rust identities. Comparator/selection does not change blocked membership.

## Username and display-name writes preserve existing authority

For A02 usernames,7to reads0Ws.A0V and0bY.AZT for the supplied keys. It submits
a key/value to0bY.BQ1 only when neither result map contains that key. Presence
checks are containsKey, not value-null/blank checks. It does not replace a known
username merely because the blocklist returned another one. Concrete result-map
semantics and interface backend persistence remain untraced.

For A00 display names, it reads0bY.AZS. Each result is12f; a proposed update is
submitted to0bY.BPw only when no row exists, its A00 string is null, orA0q says
the existing string is blank. A0q returns true for an empty CharSequence or all
UTF16 chars satisfying0Fv.A01; that predicate is Character.isWhitespace OR
Character.isSpaceChar. A nonblank existing value suppresses the update. Exact
Android Character tables/OEM behavior are not executed or replaced with Rust
trim assumptions. Empty input maps cause their entire branch to be skipped.

No shown branch clears unrelated existing names, absent keys, or metadata for a
contact no longer blocked. The worker is not parameterized by the final blocked
set; parsed metadata can have originated from non-block items. Interface calls
are evidence of submitted updates, not proof of backend commit or successful
cross-store atomic adoption.

## Rust consequence and remaining work

Rust currently replaces each unified blocked row's metadata with the last block
item and deletes rows absent from the final blocked set. It has no demonstrated
equivalent for the separate native existing-username/display-name stores and
association ordering. Changing that row to blindly overwrite or merge fields
would not port this policy. Next trace the concrete0bY/0Ws implementations and
Rust contact metadata ownership before introducing a durable adapter. No Rust
behavior, automatic fetch, retry, metadata erasure or live request is introduced.

## Reproduction

From repository root, pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-metadata-adoption-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/7to;' --method A00 --signatures --exceptions > "$OUT/metadata-worker.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/878;' --method '<init>' --signatures --exceptions > "$OUT/metadata-model.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7to;' --method '<init>' --signatures --exceptions > "$OUT/worker-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Cf;' --method A0q --signatures --exceptions > "$OUT/blank.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Fv;' --method A01 --signatures --exceptions > "$OUT/space.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0O --signatures --exceptions > "$OUT/adopt.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8be;' --method run --signatures --exceptions > "$OUT/task.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8c3;' --method compare --signatures --exceptions > "$OUT/compare.txt"
```

For the selected excerpts, run this exact extraction on the last two full temp
exports; it deliberately labels them partial and retains the dispatch header:

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
root = Path('/tmp/blocklist-metadata-adoption-repro')
for name, start, end in [('task', 0xb0a, 0xb36), ('compare', 0x7d4, 0x814)]:
    lines = (root / (name + '.txt')).read_text().splitlines()
    selected = ['# Selected DEX excerpt; not a complete method.'] + lines[:8]
    for line in lines[8:]:
        match = re.match(r'    ([0-9a-f]+)\.\.', line)
        if match and start <= int(match[1], 16) < end:
            selected.append(line)
    (root / (name + '-selected.txt')).write_text('\n'.join(selected) + '\n')
PYCODE
```

checksums.json binds every other bundle file. Full unrelated synthetic task and
comparator branches remain outside Git. Static evidence cannot prove live
protocol correctness, backend success or zero restrictions.
