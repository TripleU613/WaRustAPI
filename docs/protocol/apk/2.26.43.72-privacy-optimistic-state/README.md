# Privacy optimistic view versus confirmed receipt preference

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK/runtime/account execution. Production Rust behavior is unchanged.

## Setter notification is not state confirmation

The [retained setter](../2.26.43.72-privacy-setter-pending-lifecycle/README.md)
inserts its pending desired value before selecting transport, then joins0096
after the classic dispatch returns or MEX coroutine is scheduled. It invokes
0oe.A0Q(singleton category), then8OA selector2. This ordering does not wait for
successful server response; neither does it establish that no asynchronous
response can race with the observer invocation.

Complete A0Q only constructs8OL(list,14) and dispatches through07o.A00 with a
null0Kl. The constructor stores selector and captured list. CbC selector14
(01fc..021c) casts the recipient to8pp and callsCGi(list). No preference write,
network submission or response admission appears in A0Q/the selected case.
The shared dispatcher can invoke/schedule listeners; active registration,
CGi implementations and downstream effects remain unknown. This is not a
universal UI-only or network-free observer claim.

## Pending values can overlay an integer view

Complete0oe.A0L(category) obtains a stored integer/category-specific base, then
at0026 looks up A0E[category]. A present7wp replaces the candidate integer with
8AX.A00(pending.A00). It returns Math.max(0,candidate). Thus a caller of this
view can see a desired setting before server confirmation. The method itself
is a read/overlay, not a preference write or durable acceptance. Its category
branches use distinct defaults/feature authorities; those must not be copied
as account state. Invalid negative conversion can clamp to0; that display/view
fallback is not authority to transmit category all.

8Y1.BWP(82B) separately obtains0oe and returns whether the map contains a
nonnull entry under82B.A05. It does not examine that entry's value or server
response. Its lifecycle/caller domain is not newly reconstructed. Neither this
pending-existence query nor A0L is established as the complete receipt policy.

## Actual receipt Boolean remains a separate preference read

Complete09S.A1H reads read_receipts_enabled with compiled defaulttrue from its
namespaced SharedPreferences, without the pending-map overlay. Reuse the
[current read/read-self selector](../../read-receipt-policy-current.md):0xW.A05
uses this Boolean in one preference branch among other chat/account gates.
The retained0oe.A0R readreceipts branch writes this Boolean after selected
value/pending reconciliation, distinct from A0Q notification. This does not
establish every preference writer, effective defaults or full outgoing policy.
No new bridge from optimistic integer view/pending-existence query to receipt
send admission has been proven. Rust must not treat a local UI/observer change
as confirmed server preference or invent a new pending-read suppression rule.

## Literal references and guessed method names are not semantic ownership

The exact read_receipts_enabled string census has23 physical references,
including preference access, enum conversion, analytics field maps and UI code.
It is not23 writers or23 active protocol paths. Selected Bm1.A02 case0 passes
the literal as target_setting in a SettingsPrivacy Intent; the excerpt includes
the0062 common navigation handoff. Untraced helper effects remain unknown.

SettingsPrivacy.CHI contains the literal in shared key comparisons, not a
putBoolean. Its selected entry paths bind the key to privacy_status (first argument3) or
privacy_profile_photo (first argument2); the read-receipt comparison does not thereby prove
a reachable read-receipt setter. Other entry values return. Native comparisons
may remain in shared/inlined blocks; string hits alone must not be counted as
live preference writes. The full small method is retained to check those paths.

A focused0oe.A0M export was also tried while looking for another value getter;
its signature is A0M(278):void and it is the already-retained refresh protocol
selector. It exactly matches
[refresh-protocol-selector-dex.txt](../../rea/2.26.43.72-account-privacy-refresh/refresh-protocol-selector-dex.txt).
It is not an overloaded Boolean value getter. No duplicate REA record/export
is added for that reused body.

## Reproduce

From repository root with pinned APK and rebuilt xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/privacy-optimistic-state-repro
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/0oe;' --method A0Q --signatures --exceptions > "$OUT/notify.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0oe;' --method A0L --signatures --exceptions > "$OUT/integer-view.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8Y1;' --method BWP --signatures --exceptions > "$OUT/pending-query.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/09S;' --method A1H --signatures --exceptions > "$OUT/receipt-pref.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8OL;' --method '<init>' --signatures --exceptions > "$OUT/observer-constructor.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8OL;' --method CbC --signatures --exceptions > "$OUT/observer.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/settings/ui/privacy/SettingsPrivacy;' --method CHI --signatures --exceptions > "$OUT/dialog-result.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Bm1;' --method A02 --signatures --exceptions > "$OUT/navigation.txt"
target/debug/xtask apk-xref "$APK" --anchor read_receipts_enabled > "$OUT/preference-refs.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0oe;' --method A0M --signatures --exceptions > "$OUT/refresh-selector.txt"
python3 - <<'PYCODE'
from pathlib import Path
import re
root=Path('/tmp/privacy-optimistic-state-repro')
for source,name,lo,hi in [('observer','observer-selected.txt',0x1fc,0x21c),('navigation','navigation-selected.txt',0x62,0xee)]:
    lines=(root/(source+'.txt')).read_text().splitlines()
    selected=['# Selected DEX excerpt; not a complete method.']+lines[:8]
    selected += [s for s in lines[8:] if (m:=re.match(r'    ([0-9a-f]+)\.\.',s)) and lo<=int(m[1],16)<hi]
    (root/name).write_text('\n'.join(selected)+'\n')
PYCODE
```

Compare selected/complete exports with this bundle and refresh-selector to the
reused file above. checksums.json binds every other file by bytes and SHA256.
Only selected synthetic cases are retained; unrelated cases stay local.

**Remaining:** complete listener/caller ownership, every preference writer,
full pending reconciliation and durable Rust setter recovery. This closes
selected optimistic-view/confirmation boundaries, not full protocol parity.
