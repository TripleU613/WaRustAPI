# Blocklist changed-contact observer interface census

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK/device/account execution or live request. The offline development tool
changes here; production protocol/ACK behavior does not.

## Close the callback name and interface handoff

The prior [changed-union handoff](../2.26.43.72-blocklist-delta-storage-boundary/README.md)
ends1dH.A09 ->07o.A00(3XP(collection,7)). A focused3XP.invoke lookup returned
no bodies; that does not mean the callback was absent. Its declared callback
isCbC(Object). The generic Object/int constructor binds collectionA00 and$t=7;
CbC switch7 ->01c8 casts the recipient to0d0, then callsBlg(Collection).
The selected complete case and dispatch header are retained, not unrelated
synthetic cases. The constructor file also retains its other two small overloads;
they are not the selected construction here.

07o.A00 partitions registered listeners by07v/0Kn marker types and invokes or
schedules callbacks based on the supplied0Kl and current thread/context. Its
complete selected method is retained, but all scheduler paths/registration
lifetimes are not reconstructed. Dispatching this observer is not proof of a
network send, of completion, or of active listener identity.

## Opt-in interface ancestry closes a static tooling gap

Existing apk-virtual-overrides follows superclass ancestry by default. New
--include-interfaces reads class_def.interfaces_off/type_list across all DEX
entries and traverses superclass plus interface edges, including subinterfaces
and inherited interface ancestry. It still selects **declared** exact ordinary
instance methods with matching full DEX signatures; private/static/direct
methods are excluded as before. It does not perform runtime/default-method
resolution, infer active registrations, or enumerate instance classes whose
method is inherited from an unrelated implementation owner.

The reader checks interface-list alignment/extents/type indices, object-type
shape and duplicate entries/classes before output. The queried ancestry walk
rejects encountered cycles with an iterative active/completed traversal;
diamonds are allowed. This is a bounded structural reader, not a complete DEX
verifier. Failed later DEX parsing cannot emit an already-collected partial
census as complete. Default superclass-only output remains byte-for-byte equal
to the retained0bY.BQ1 negative, including its interface-dispatch limitation.

Synthetic tests cover subinterface plus superclass inheritance, unrelated
same-name declarations, a diamond, interface cycles, literal interface list
indices, duplicate/primitive/out-of-range entries, truncation, unaligned and
oversized lists; the existing later-bad-DEX test now covers both modes. Shared
exact signature/access/final/root checks remain. No app target needed for those
unit tests. The pinned real-APK control then finds37 declaredBlg(Collection)
methods (including the abstract root and no-op declarations) across114178
physical class definitions. These are not37 active observers or completed ports.

## What the census does and does not establish

All37 selected declaration bodies/ancestry are retained ininterface-census.txt.
Several have only return/no-body; others refresh views or hand off work. For
exampleDC3.Blg obtains0zB, copies the collection and queuesFef selector0x27;
3S9 has selector-dependent paths including a coroutine handoff. Therefore the
interface boundary cannot be labelled universally UI-only from the direct
callback body. These downstream jobs and registration/provider/thread policy
remain open. No new generic ACK, request replay, fabricated listener effect or
runtime blocklist notification support is introduced.

## Reproduction

From repository root with pinned APK and rebuilt xtask:

```sh
cargo build --locked -p xtask
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-observer-interface-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-virtual-overrides "$APK" --class 'LX/0d0;' --method Blg --signature '(Ljava/util/Collection;)V' --include-interfaces > "$OUT/interface-census.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/07o;' --method A00 --signatures --exceptions > "$OUT/observer-dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/3XP;' --method '<init>' --signatures --exceptions > "$OUT/callback-constructors.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/3XP;' --method CbC --signatures --exceptions > "$OUT/callback.txt"
target/debug/xtask apk-virtual-overrides "$APK" --class 'LX/0bY;' --method BQ1 --signature '(Ljava/util/Map;)V' > "$OUT/default-compat.txt"
cargo test --locked -p xtask --all-features interface_
cargo test --locked -p xtask --all-features virtual_override
```

Compare default-compat to the prior store-writes/superclass-only.txt. Extract
the selected callback exactly:

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
root = Path('/tmp/blocklist-observer-interface-repro')
lines = (root / 'callback.txt').read_text().splitlines()
selected = ['# Selected DEX excerpt; not a complete method.'] + lines[:8]
for line in lines[8:]:
    match = re.match(r'    ([0-9a-f]+)\.\.', line)
    if match and 0x1c8 <= int(match[1], 16) < 0x1e2:
        selected.append(line)
(root / 'callback-selected.txt').write_text('\n'.join(selected) + '\n')
PYCODE
```

checksums.json binds every other bundle file. No new REA record or runtime,
complete interface dispatch, ban-rate or full APK equivalence is claimed.
