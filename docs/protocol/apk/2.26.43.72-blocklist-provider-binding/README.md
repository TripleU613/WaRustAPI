# Default static blocklist metadata provider binding

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
DEX instructions/catches are authority; no application/account/device execution.
This closes the packaged default-binding gap identified in the
[previous writer trace](../2.26.43.72-blocklist-store-writes/README.md), not all
runtime override, provider lifecycle or backend atomicity questions.

## Provider -> cached registry -> concrete factory

The retained [7to initializer](../2.26.43.72-blocklist-metadata-adoption/worker-init.txt)
puts05d.A00(0xb43) into its A00 provider.05d.A00 wraps00D.A00(id)'s05h in05a;
05a stores that provider in A00;05h's constructor stores the integer.05h.get returns its cached A01 when present,
otherwise synchronizes, checks again and calls00D.A02(stored id). Exceptions
leave through the retained catch/monitor-exit path; a failed call is not shown
installing a successful object. No concurrency/volatile or full reset claim.

00D.A02 calls the global context's00D.A03. A03 first checks a static concurrent
map for this integer key. On a miss it obtains a per-ID lock, checks again,
enters the shown context plumbing and calls1Uw.A00(id,...). A nonnull return is
cached. Existing cached objects bypass the factory. This is a default static
construction route, not proof of the current runtime cache contents; map writers,
context reset, lifecycle and override enumeration remain untraced.

1Uw.A00 dispatches (id >>14)&0x7f. For0xb43 this is0 ->A01. A01 masks0x3fff
and uses range thresholds:0xaf7 <0xb43 <=0xc10 ->A0S. A0S's exact switch key
2883/0xb43 targets0954, which requests00D.A02(0x430), casting to0bY.
For0x430 the same root/range path reachesA04 (0x349 <0x430 <=0x462).
A04's exact key1072/0x430 targets0d70: new0bZ; constructor; return.
This does not depend on guessing the type from method names or declaration order.
The selected dispatcher exports are explicitly partial, retaining original
switch lines/selected instructions, not complete methods or unrelated factories.

## Concrete store dependencies

0bZ's constructor binds:

- A01 via05d(0x439); A04 key1081/0x439 ->0db8 constructs12d. The delegated
  per-entry Boolean-result-ignoring wrapper previously traced is therefore on
  this default static route, rather than merely a matching declaration.
- A05 via00D.A02(0xb42), cast0Ws. This alias's exact switch body is not retained
  by this chunk; its final factory target is not newly claimed here.
- A00 via05d(0x438); A04 key1080/0x438 ->0dac constructs12g, the metadata
  table-value helper previously traced.
- A04 via00D.A02(0x41a), cast0Wv; A02 via05d(0x1c63); A03 via05d(0x137).
  Their complete backend/provider effects are not newly reconstructed.

The focused0bZ constructor caller census finds1Uw.A04. It is a direct-call
census, not proof of absence of reflection or runtime injection. Cached provider
and registry objects may bypass construction. No active runtime selection observed.

## Rust ownership and remaining correction

Rust transport/contacts.rs is a local PN address book plus usync existence
checking; its Contact stores full_name/first_name and timestamps, not a native
LID username/display-name/alias backend. Do not overwrite a user's local saved
name as a substitute for this separate Android metadata system. The blocked-row
cache is also not demonstrated equivalent to12g/0Ws/12d ownership.

The static default worker->0bZ->12d/12g chain is now established. Remaining work
is to design/port the separate durable metadata ownership using the concrete
read/write and alias authorities, including username normalization and partial
outcomes, without assuming all native stages are atomic. No Rust behavior,
identity merge, new sender, retry, metadata erase or live request changed.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-provider-binding-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/05d;' --method A00 --signatures --exceptions > "$OUT/provider-entry.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/05a;' --method '<init>' --signatures --exceptions > "$OUT/provider-wrapper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/00D;' --method A00 --signatures --exceptions > "$OUT/provider-lookup.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/05h;' --method '<init>' --signatures --exceptions > "$OUT/provider-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/05h;' --method get --signatures --exceptions > "$OUT/provider-get.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/00D;' --method A02 --signatures --exceptions > "$OUT/provider-resolve.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/00D;' --method A03 --signatures --exceptions > "$OUT/registry-cache.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1Uw;' --method A00 --signatures --exceptions > "$OUT/root-dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0bZ;' --method '<init>' --signatures --exceptions > "$OUT/store-init.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/0bZ;' --method '<init>' > "$OUT/store-constructor-callers.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1Uw;' --method A01 --signatures --exceptions > "$OUT/range.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1Uw;' --method A0S --signatures --exceptions > "$OUT/alias.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1Uw;' --method A04 --signatures --exceptions > "$OUT/factory.txt"
```

Extract the three selected partials with this exact script:

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
root = Path('/tmp/blocklist-provider-binding-repro')
for name, ranges in [('range', [(0, 0xc), (0xcd2, 0xd86)]),
                     ('alias', [(0x954, 0x966)]),
                     ('factory', [(0xd70, 0xd88), (0xdac, 0xdc4)])]:
    lines = (root / (name + '.txt')).read_text().splitlines()
    selected = ['# Selected DEX excerpt; not a complete method.'] + lines[:6]
    for line in lines[6:]:
        match = re.match(r'    ([0-9a-f]+)\.\.', line)
        if match and (('packed-switch ' in line and name != 'range') or
                      any(a <= int(match[1], 16) < b for a, b in ranges)):
            selected.append(line)
    (root / (name + '-selected.txt')).write_text('\n'.join(selected) + '\n')
PYCODE
```

checksums.json binds every other bundle file. No new REA source-body claim or
live/ban-rate result follows from these static registry exports.
