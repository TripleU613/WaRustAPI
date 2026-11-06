# PSA wakeup completion and watchdog scheduler

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Eight focused complete methods/census exports and one labelled synthetic arm;
no REA Evidence record, APK execution, account operation or runtime policy change.

## Reuse the watchdog finding

The [current diagnostic-watchdog bundle](../2.26.43.72-deferred-ack-watchdog/README.md)
already establishes selector41: it clears tracking maps without sending an ACK
or consuming a held89Y token. A fresh full8bZ.run export matches that retained
file exactly. Missing map membership does not block a later89Y.A00 because its
remove result is ignored. The new PSA notes had incorrectly left this known
finding unlinked; their unknown is corrected rather than duplicating evidence.

A fresh declared override census for08Z.Cc2 includes08a andQew, each calling
Android Handler.postDelayed with the supplied long and returning a wrapper
Runnable. This binds the6000 delay to milliseconds on these compiled routes.
Both CYL implementations use Handler.removeCallbacks with the supplied Runnable.
0v6.A07's cancel/reschedule therefore tracks the returned wrapper, not a durable
per-notification timeout. postDelayed's Boolean is ignored; returning the wrapper
does not prove it was queued or ran. Concrete effective provider selection,
wrapper bodies, Looper/process lifecycle and concurrent cancellation remain open.
The census is declared ancestry only, not proof of every runtime implementation.

## Enabled wakeup worker

Reuse [19n.A09](../2.26.43.72-psa-ack-lifecycle/handler.txt), which schedules
Fc2 selector27 after the4200 feature gate. Fc2 constructor binds handler toA00, token toA01 and selector to$t.
The excerpt keeps the complete
selected arm11dc..130e and its Exception catch08f8..0978 (code units).
The worker requires a nonnull89Y; missing token throws outside that catch.
Property19043 (0x4a63) controls reading optional first psa_wakeup/psa_push_id.
It also records selected Ds1/Dr5 diagnostics; no required telemetry policy is
inferred from those event objects.

Property19827 (0x4d73) and12Q.ABu select whether19n.A0A(pushId) is called:
false property or true ABu calls it, while true property plus false ABu skips it.
Both normal joins call89Y.A01 after the selected method returns. An Exception
inside byte11f0..12f0 instead logs corruption and calls19n.A03 ->487.
Do not generalize this catch to all worker failures or missing ownership.

A0A constructs Eni/FbF context and calls1Oc.A00 with event name
whatsapp_push_notification_event and integer11231 (0x2bdf), then callsA04 with
its nullable Fy4 result.1Oc delegates toFy9.A01; that obtains a list fromFy3.A02,
filters throughFy9.A00 and returns its first result or null. The retained A04
selects quick-promotion renderers, may callF0H.A0C directly, otherwise schedules
FcV selector6 through08Z.CbD. It does not transfer89Y ownership to that queued
renderer in the inspected call. Exceptions remain inside the outer selected
worker catch when thrown before its completion call.

This is a promotion selection/rendering path, not evidence of a newly identified
network fetch. Fy3/filter/renderer internals, any indirect network work and its
completion policy remain untraced. A future port must distinguish method-return
ACK completion from queued rendering or hypothetical remote acceptance. No HTTP
request, endpoint, wait-for-fetch rule or mandatory telemetry is invented.

## Reproduce

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-psa-wakeup-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/Fc2;' --method '<init>' --signatures --exceptions > "$OUT/wakeup-init.txt"
target/debug/xtask apk-virtual-overrides "$APK" --class 'LX/08Z;' --method Cc2 --signature '(Ljava/lang/Runnable;J)Ljava/lang/Runnable;' --include-interfaces > "$OUT/delay-implementations.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/08a;' --method CYL --signatures --exceptions > "$OUT/cancel-worker.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Qew;' --method CYL --signatures --exceptions > "$OUT/cancel-alternate.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/19n;' --method A0A --signatures --exceptions > "$OUT/promotion-entry.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/19n;' --method A04 --signatures --exceptions > "$OUT/promotion-render.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1Oc;' --method A00 --signatures --exceptions > "$OUT/promotion-delegate.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Fy9;' --method A01 --signatures --exceptions > "$OUT/promotion-select.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Fc2;' --method run --signatures --exceptions > "$OUT/wakeup-full.txt"
python3 - "$OUT" <<'RECIPE'
from pathlib import Path
import re,sys
out=Path(sys.argv[1]);lines=(out/'wakeup-full.txt').read_text().splitlines()
selected=['# Selected DEX excerpt; not a complete synthetic run method.']+lines[:6]
for line in lines[6:]:
    m=re.match(r'\s*([0-9a-f]+)\.\.',line)
    if m and (int(m[1],16)<0xe or 0x11dc<=int(m[1],16)<0x130e):selected.append(line)
selected+=['    # Selected exception range only:']+[l for l in lines if '# catch Ljava/lang/Exception; code_units 08f8..0978 -> 0978' in l]
(out/'wakeup-selected.txt').write_text('\n'.join(selected)+'\n')
RECIPE
```

Compare every retained .txt exactly. The full mixed synthetic class stays local;
checksums.json covers every retained file except itself. Shared watchdog/base/
completion evidence stays under its existing identity. No raw APK, private
state, media or bulk tree. Runtime PSA policy remains the existing legacy helper;
effective flags, native work/persistence/recovery and other workers remain open.

Verification: all nine retained exports reproduce exactly, including the
separately rechecked constructor binding. The existing watchdog export matches
its retained original. fmt, OpenAPI library-symbol and CLI-reference tests,
APK coverage freshness and repository hygiene passed. Evidence-only change;
no new runtime compliance, full-workspace test run or live result is claimed.
