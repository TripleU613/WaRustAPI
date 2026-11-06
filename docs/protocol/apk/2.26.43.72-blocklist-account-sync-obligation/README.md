# account_sync blocklist: delta/refresh work before normal ACK

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No production code, account/device/APK execution or live request changed.
Complete retained [19N.A09](../../rea/2.26.43.72-notification-sync-obligations/account-sync-dex.txt)
reproduces exactly; prior complete REA source includes switch-recovery warnings,
so DEX is branch authority. No duplicate REA record/body copy is added.

## Route, parsing and admission

The exact blocklist literal census points to19N.A09, the account_sync handler,
not a separate outer blocklist notification.19N reads the **first child** with
0Tp.A0D and branches on that child's description. Its blocklist branch at02c6
reads outerid, saved09S.A0c hash, child.prev_dhash and parses6qD.A02 before
reading action. Parsing failure is not automatically a fresh-fetch branch.
The parser's default block, last-write membership map, typed identity/metadata
limits are covered by earlier blocklist evidence.

6pA.A03(notificationId,"notification") supplies admission; the
[retained method](../2.26.43.72-blocklist-set-reconciliation/reconcile-admission.txt)
consults blocklist mode and the chat-migration predicate previously traced.
No effective Rust/global role/mode authority is inferred from a cached mapping.

The refresh branch at051a is selected when action is exactmodify, or the saved
hash does not equal nullableprev_dhash, or admission is false. Equality is
nullable object equality: two nulls compare equal, unlike a guessed mandatory
nonempty hash. The complete09S.A0c export here calls getString with null
default for block_list_v2_dhash; it does not synthesize an empty string.
The selected blocklist branch has no version-field comparison;
version parsing elsewhere in this large account handler is not this rule.

Otherwise, an empty item list exits. Parsed true/false entries are separated,
then additions already present in1dH.A0U are removed and removals not present
are removed. If both effective sets are empty it exits without advancing the
hash or running the metadata worker. This is not a full snapshot replacement.

## Delta work and asynchronous persistence

For effective changes,19N calls1dH.A0A(add,true) andA0A(remove,false), then
under a monitor adds/removes the effective identities inA0U and schedules
8al(metadata,manager,remove,add,hash,1) throughEeV.A01. The retained
[shared constructor](../2.26.43.72-blocklist-null-snapshot-hash/replacement-task-init.txt)
binds these objects to the selected run case1 fields. The selected worker
export here contains the complete case plus its shared queue/return tail.

Case1 obtains1dH.A0C's set and calls7PB.A0P (whole-set persistence), saves
block_list_v2_dhash through2L8.A1U, calls7to.A00(metadata), then queues a union
of effective changes. Lower-level helper commit/error policy and handler/worker
races are not proven atomic by these calls; no native storage success is
observed. This is separate from full replacement's run case0. The A0A effects,
A0C authority and completion consumer still need their own complete tracing.

## Refresh request is selected work, not replay

051a..054c sets2D6.A01=true, constructs1sx and calls
1sv.A01(request,false,true,false). The complete builder/constructor exports
bind this flag to1sx.A01; other new-object flags remain false, including its
A00 retry-mode flag. The retained
[1sv manager](../../rea/2.26.43.72-account-privacy-refresh/refresh-manager-dex.txt)
uses request kind4 for this blocklist slot, not kind3/privacy orkind5/devices.
It passes the last Boolean false toA00, bypassing that helper's optional
60,000ms suppression. For enabled retry bookkeeping (middletrue), the false
request retry-mode chooses fresh account_sync_blocklist_num_retries=3 rather
than decrementing. This is saved bookkeeping, not proof of three transmissions.
It records kind4's timestamp and invokes1dH.A0M, the ordinary get builder.
Full retry scheduler, callback cancellation and effective provider authority
remain unported. No original per-contact block/unblock IQ is replayed here.

## ACK boundary and Rust gap

The retained [normal base dispatch/ACK](../../rea/2.26.43.72-notification-ack-audit/README.md)
and [account handler audit](../../notification-sync-obligations.md) establish
that19N does not call0v6.A07 deferral. A normal handler return may therefore be
ACKed after it schedules work, while the worker/request completes later. This
does not prove every malformed input is ACKed, nor that an ACK can replace
processing. Exception/descriptor/base routing still apply.

Rust's receive path supports selected account_privacy only; account_sync
blocklist remains durably captured without ACK or delta/refetch execution.
An ACK-only implementation would discard the obligation. A correct adapter must
map admission and hash authority, durably record/apply appropriate delta or
refresh work, preserve deduplication/uncertain writes and then implement the
traced ACK sequencing. No fake mode, blind snapshot replacement, automatic
request or generic ACK is added by this research chunk.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-account-sync-obligation-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-xref "$APK" --anchor blocklist > "$OUT/literal-xrefs.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/09S;' --method A0c --signatures --exceptions > "$OUT/saved-hash.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/19N;' --method A09 --signatures --exceptions > "$OUT/account-sync.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2D6;' --method A00 --signatures --exceptions > "$OUT/refresh-flags.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1sx;' --method '<init>' --signatures --exceptions > "$OUT/refresh-request-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1sv;' --method A00 --signatures --exceptions > "$OUT/refresh-suppression.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1sv;' --method A01 --signatures --exceptions > "$OUT/refresh-manager.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8al;' --method run --signatures --exceptions > "$OUT/delta-worker.txt"
```

Compare account-sync/refresh-manager to the linked retained complete files.
Extract the selected delta worker exactly;089e branches to shared0940:

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
root = Path('/tmp/blocklist-account-sync-obligation-repro')
lines = (root / 'delta-worker.txt').read_text().splitlines()
selected = ['# Selected DEX excerpt; not a complete method.'] + lines[:9]
for line in lines[9:]:
    match = re.match(r'    ([0-9a-f]+)\.\.', line)
    if match and (0x804 <= int(match[1], 16) < 0x8a0 or
                  0x940 <= int(match[1], 16) < 0x948):
        selected.append(line)
(root / 'delta-worker-selected.txt').write_text('\n'.join(selected) + '\n')
PYCODE
```

The selected tail's queue instruction starts0940; the preceding093a..0940
constructor is intentionally not included because case1 already constructs
its own task at0898. checksums.json binds every other bundle file. No full
notification compatibility or zero-restriction result follows from this trace.

Verify the key callback/worker branch destinations (signed code units times2):

```sh
python3 - <<'PYBRANCH'
from pathlib import Path
import re
root = Path('/tmp/blocklist-account-sync-obligation-repro')
for name, pairs in [('account-sync', [(0x33e, 0x51a), (0x34a, 0x51a),
                                     (0x34e, 0x51a), (0x36c, 0xe6),
                                     (0x42a, 0xe6)]),
                    ('delta-worker', [(0x89e, 0x940)])]:
    text = (root / (name + '.txt')).read_text()
    for source, target in pairs:
        line = next(line for line in text.splitlines()
                    if line.startswith(f'    {source:04x}..'))
        match = re.search(r'([+-])0x([0-9a-f]+)$', line)
        assert match, line
        direction = 1 if match[1] == '+' else -1
        assert source + direction * int(match[2], 16) * 2 == target, line
print('Six branch destinations match')
PYBRANCH
```
