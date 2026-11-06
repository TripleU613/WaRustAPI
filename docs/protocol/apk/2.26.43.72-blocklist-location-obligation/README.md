# Blocklist observer conditional identity-query and location state

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK/device execution or live account operation. Production behavior unchanged.

The preceding [interface census](../2.26.43.72-blocklist-observer-interface/README.md)
retains DC3.Blg copying the changed collection and queueing Fef selector39.
The selected job body is06ae..07e6. It looks up each changed identity in
0zB.A08 under A0c; an existing EVN's A03 list is scanned with0Dn.A0Y.
If none matches,06ce +0x5e code units reaches078a and skips the query.
A matching participant exits the monitor and calls3KB.A01 with a HashSet of
**the entire changed collection**, false, not just that participant.

This is the same A01(set,false) producer already retained in
[missing-LID preflight](../2.26.43.72-prekey-lid-preflight/README.md).
Its candidate filtering and empty-set no-query return still apply; passing false
does not bypass the query through the helper's Boolean readiness shortcut.
Queue combination, MEX/classic selection and nested timers remain governed by
that pipeline, not a new unconditional blocklist IQ. See
[route/timing evidence](../2.26.43.72-prekey-lid-query-route/README.md) and
[wire-selection evidence](../2.26.43.72-prekey-lid-mex-selection/README.md).
No claim that every blocklist change causes a network send follows.

The selected Exception catch at0750 logs the failure and constructs1xM(set,6).
At0774 the returned object is compared by reference to1xM.A05; equality logs
network unavailable and returns before later location updates. No general
success-field check is present here. No network-status enum semantics beyond
this branch are inferred. Other exceptions outside the retained try ranges
are not covered by this catch.

Otherwise078a iterates the full changed collection, logs/skips null identities,
calls0zB.A02(jid,null,null,manager,02X.A0Y), collects nonnull ESCs, and invokes
A0B(jid,null,manager). A0B conditionally removes a per-chat map entry, calls
A0G/F3L update helpers, removes the outer entry if emptied and calls A0J,
notifies Fv1.CBv,
calls A0E and posts Fef selector41. These downstream helpers/job remain
**unknown** here; its onReceiveStopSharing log does not prove an outbound
stop-sharing packet. A0I iterates collected ESCs through A0C. That selected
method builds J6g and passes it to0NX.CSW; downstream transport is untraced,
so this alone is not proof of a protocol send either.

**Unknown/unported:** active DC3 registration, provider/cache initialization,
EVN list ownership, A0Y subtype interpretation, A02 gates, final downstream
location effects, query response/mapping authority and durable recovery/ACK
ownership. The broader blocklist notification remains unsupported/captured
without ACK in Rust. No fabricated location state, automatic lookup, replay or
ACK-only shortcut is added.

## Reproduce

From the repository root with rebuilt xtask and the pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-location-repro
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/Fef;' --method run --signatures --exceptions > "$OUT/job.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0zB;' --method A0B --signatures --exceptions > "$OUT/contact-change.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0zB;' --method A0I --signatures --exceptions > "$OUT/updates.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0zB;' --method A0C --signatures --exceptions > "$OUT/finalize.txt"
```

Extract the selected body with this exact recipe; complete unrelated job cases
stay local. No new REA Evidence ID is claimed for these DEX exports.

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
root=Path('/tmp/blocklist-location-repro')
lines=(root/'job.txt').read_text().splitlines()
selected=['# Selected DEX excerpt; not a complete method.']+lines[:8]
selected += [s for s in lines[8:] if (m:=re.match(r'    ([0-9a-f]+)\.\.',s)) and 0x6ae<=int(m[1],16)<0x7e6]
selected += [s for s in lines if '# catch ' in s and any(x in s for x in ['0370..0396','03a2..03a8','03c2..03c4'])]
(root/'job-selected.txt').write_text('\n'.join(selected)+'\n')
PYCODE
```

checksums.json binds every other bundle file by bytes and SHA256. The original
A01 export was compared byte-for-byte with the retained preflight export;
reuse that evidence rather than duplicating its methods.
