# Privacy MEX mutation SDK wire and intent recognition

**Confirmed selected static evidence**, WhatsApp 2.26.43.72 build264307238,
APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
These focused DEX exports reuse the [mutation producer](../2.26.43.72-privacy-mex-set/README.md)
and [SDK registration/document/trace evidence](../../rea/2.26.43.72-privacy-transport/README.md).
They add no REA Evidence IDs or runtime observations.

## Selected route and wire

0iT constructor binds input A01, response class A02, secondary class A03,
namespace A04, operation A05 and final Boolean A08. The selected mutation
supplies null secondary class. 0io.A02 rejects Pando capability for this
whatsapp-android-mex request with null A03 (0mO.A0B,
REJECTED_MEX_NOT_PANDO_CAPABLE). Retained excerpt includes the entry/selection
and rejection target, not the full method. Branch offsets are code units.

2LE.A0L -> 0ig.A01 constructs a new 0iq; 2LE.A0Q -> ARv -> AQY -> AQV -> A00
reaches its executor. With constructor-default A03=false, A00 constructs 0j3.
The alternate true branch constructs RsH; the field census has many external
writers, so this is the selected default route, not proof all callers use it.
A08=true is not the IQ get/set selector in this constructor. Its census locates
other SDK consumers; full Boolean semantics remain unknown.

0iq constructor initializes A00:J to32000; 0j3 constructor stores the supplied
long and A06 forwards that field to 0Sf.A0W, discriminator341. The duration
census has external writes: do not describe this as a universal fixed MEX
deadline, or an inline literal in A06. No Rust timeout change follows.

0mF obtains getQueryParams/Axc, converts the map to JSONObject, and serializes
queryId plus variables. 0mK constructs an IQ to the server-only JID, xmlns=w:mex,
**type=get**, id from the SDK; a direct query child carries query_id=documentID
and the JSON text. The optional trace is a sibling. For this mutation the
pinned documentID is `26887749497493182`; the input is nested setting with
feature READRECEIPTS and setting NONE. IDs stay decimal strings.

## Rust correction and limits

account_privacy now recognizes this exact direct query_id on w:mex/get alongside
classic privacy/set. Queued, sent, successful and refused saved actions retain
the existing unowned-intent hold before fresh notification adoption/ACK.
Processed redelivery may still be re-ACKed without reapplying state. This closes
a missed wire form; it does not implement native category-keyed ownership,
mutation execution, response projection, effective mode selection or recovery.
The guard is deliberately broader than native per-category reconciliation.
GetPrivacySettingsQuery, nested query IDs and body text containing the mutation
ID do not establish this setter intent. No request, retry or ACK is invented.

Regression failed against the old guard, then passed across four action outcomes
and encrypted-store reopen. The actor test covers both classic and MEX refused
setters, no fresh notification ACK/adoption, continued ping handling and no
resend. Workspace:1534 passing tests across nine suites including doc tests.
No live account operation or ban-rate claim.

## Reproduce

From repository root with the pinned APK and built xtask, run:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-privacy-wire-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/0iT;' --method '<init>' --signatures --exceptions > "$OUT/sdk-request-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iT;' --method 'getQueryParams' --signatures --exceptions > "$OUT/query-params.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2LE;' --method 'A0L' --signatures --exceptions > "$OUT/execute-helper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0ig;' --method 'A01' --signatures --exceptions > "$OUT/client-executor.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2LE;' --method 'A0Q' --signatures --exceptions > "$OUT/await-helper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iq;' --method 'ARv' --signatures --exceptions > "$OUT/suspend.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iq;' --method 'AQY' --signatures --exceptions > "$OUT/start.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iq;' --method 'AQV' --signatures --exceptions > "$OUT/run.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iq;' --method '<init>' --signatures --exceptions > "$OUT/executor-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iq;' --method 'A00' --signatures --exceptions > "$OUT/executor-route.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0io;' --method 'A01' --signatures --exceptions > "$OUT/pando-conversion.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0j3;' --method '<init>' --signatures --exceptions > "$OUT/xmpp-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0j3;' --method 'A06' --signatures --exceptions > "$OUT/xmpp-submit.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0mF;' --method '<init>' --signatures --exceptions > "$OUT/json-envelope.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0mK;' --method '<init>' --signatures --exceptions > "$OUT/iq-constructor.txt"
target/debug/xtask apk-field-refs "$APK" --class 'LX/0iT;' --field A08 --type Z > "$OUT/request-flag-refs.txt"
target/debug/xtask apk-field-refs "$APK" --class 'LX/0iq;' --field A03 --type Z > "$OUT/alternate-refs.txt"
target/debug/xtask apk-field-refs "$APK" --class 'LX/0iq;' --field A00 --type J > "$OUT/duration-refs.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0io;' --method A02 --signatures --exceptions > "$OUT/pando-rejection-full.txt"
python3 - "$OUT" <<'RECIPE'
from pathlib import Path
import re,sys
out=Path(sys.argv[1])
lines=(out/'pando-rejection-full.txt').read_text().splitlines()
selected=['# Selected DEX excerpt; not a complete method.']+lines[:6]
for line in lines[6:]:
    m=re.match(r'\s*([0-9a-f]+)\.\.',line)
    if m and (int(m[1],16)<0x54 or 0x1ac<=int(m[1],16)<0x1b2):
        selected.append(line)
(out/'pando-rejection-selected.txt').write_text('\n'.join(selected)+'\n')
RECIPE
```

Compare every retained .txt against the corresponding OUT file. The full
Pando method is local intermediate output only. checksums.json covers every retained
file except itself, including this README. No raw APK, private state or bulk
class tree is retained.
