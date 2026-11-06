# Full blocklist snapshot: nullable hash replacement

**Confirmed selected APK mismatch/correction**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static DEX plus synthetic encrypted-store tests, no account/device/live request.

The retained [6qD.A02 parser](../2.26.43.72-blocklist-snapshot-children/parser.txt)
reads list.dhash with6oe.A18. This helper calls0Tp.A0M(name,null); the
[retained string accessor](../2.26.43.72-blocklist-identity-metadata/string-attribute.txt)
returns that default for a missing attribute. The parser/result constructor
carry the nullable hash rather than substituting the previously saved one.

The admitted fetch path reaches the retained
[1dH.A0O replacement method](../2.26.43.72-blocklist-set-reconciliation/replace-set.txt).
Its no-difference branch02de..0300 unconditionally passes the supplied nullable
hash to SharedPreferences.Editor.putString thencommit. The changed-state path
constructs8al(...,hash,0). Its constructor binds that fifth argument toA04;
run case0 readsA04 at08b0 and unconditionally passes it toputString/commit at
0910..0922 after its selected membership-storage call. The selected complete
case and dispatch header are retained, not unrelated synthetic task bodies.

The [Android Editor API contract](https://developer.android.com/reference/android/content/SharedPreferences.Editor#putString(java.lang.String,%20java.lang.String))
(accessed2026-11-06) specifies that null has the effect of removing the key.
This is a platform API contract plus APK calls, not execution of Android storage
or a claim every OEM/device write succeeded. Native commit return checking,
ordering across metadata stores and runtime gating remain separate.

## Rust correction

Previously an admitted list without dhash replaced membership and cleared the
reconciliation warning but retained the old checkpoint. Later explicit get/set
requests could therefore carry a stale hash. apply_fetch_in now deletes the
hash checkpoint when the admitted full snapshot has no hash, in the same
transaction as membership/marker/reconciliation. A supplied hash still replaces
it. No-list/unchanged fetches return before deletion; malformed lists roll back
without changing any checkpoint. No automatic request or identity reset added.

The new four-case regression covers unchanged membership with a full no-hash
list, changed-to-empty membership with a full no-hash list, no-list control and
malformed-list control. It verifies checkpoint/journal preservation on malformed
input, reconciliation state, membership, reopen, idempotency and absence of
recreated pending actions. Explicit synthetic queued get/block controls prove
future construction omits the old hash after deletion and retains it in the
no-list/malformed controls. Reservation is offline; neither request is sent.
The old projector fails the first valid case by retaining Some(stale).

## Set-result boundary is different

The separately retained7vV.A00 only calls1dH.A08 after selected in-memory set
membership changes. A08 writes the supplied nullable hash only when7PB.A0Q
returns true. That backend execution-success gate is newly retained here (the
[later backend trace](../2.26.43.72-blocklist-set-hash-gate/README.md) clarifies
that true does not mean a positive affected-row count); it is not proof
that every successful set without dhash should clear the Rust checkpoint.
This correction changes full fetch snapshots only. Full native set hash-gating,
alias/migration and metadata parity remain open. Old already-projected histories
are not rewritten. No live acceptance or zero-restriction result is claimed.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-null-snapshot-hash-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/6oe;' --method A18 --signatures --exceptions > "$OUT/nullable-attribute.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A08 --signatures --exceptions > "$OUT/set-conditional-write.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8al;' --method '<init>' --signatures --exceptions > "$OUT/replacement-task-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8al;' --method run --signatures --exceptions > "$OUT/replacement-task.txt"
cargo test --locked -p whatsapp-api --all-features full_snapshot_without_hash -- --nocapture
```

Extract the selected case exactly:

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
root = Path('/tmp/blocklist-null-snapshot-hash-repro')
lines = (root / 'replacement-task.txt').read_text().splitlines()
selected = ['# Selected DEX excerpt; not a complete method.'] + lines[:9]
for line in lines[9:]:
    match = re.match(r'    ([0-9a-f]+)\.\.', line)
    if match and 0x8a0 <= int(match[1], 16) < 0x948:
        selected.append(line)
(root / 'replacement-task-selected.txt').write_text('\n'.join(selected) + '\n')
PYCODE
```

checksums.json binds every other bundle file. No new REA record is inferred from
these DEX exports; earlier complete parser/callback Evidence is reused.
