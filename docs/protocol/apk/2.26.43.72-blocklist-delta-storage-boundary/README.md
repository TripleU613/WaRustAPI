# Account blocklist delta: execution-time set and storage boundaries

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX/catches and [complete REA store Evidence](../../rea/2.26.43.72-blocklist-delta-storage-boundary/README.md).
No Rust behavior, account/device/APK execution or live request changed.
This extends the [account_sync delta/refresh trace](../2.26.43.72-blocklist-account-sync-obligation/README.md).

## Execution-time current set, not a frozen notification snapshot

The retained8al.run selector1 calls1dH.A0C while its worker executes, then
passes that returned set to7PB.A0P. A0C synchronizes on the manager, copiesA0U
with07Z.A1S and releases the monitor. For a Collection, A1S returns empty/
singleton sets for sizes0/1 or populates a new LinkedHashSet otherwise; it does
not expose the original mutable A0U as a live view. The hash, however, is the
captured workerA04 argument from the notification. No same-generation binding
between that captured hash and the execution-time copied set is established
without executor/handler ordering. This is not proof a race actually occurs;
EeV ordering and interleaving remain open. Do not silently reinterpret this
worker as persisting only the notification's advertised delta/full snapshot.

## Whole-set table write and transaction wrapper

7PB.A0P opens0aq and1hk, deletes wa_block_list through0ai.A03 with nullwhere/
arguments, then iterates the supplied set and inserts jid values through0ai.A00.
The returned delete/insert long values are discarded. A00 forwards0KU.A06's
result; there is no per-result affected-row/row-ID check inA0P. After the loop
it calls1hk.A00 then closes the transaction and DB. Nested catches close and
rethrow, preserving suppressed close failures throughOy7. No selected catch
turns a thrown write failure into a successful A0P return.

1hk.A00 sets its A01 flag true then calls0KU.A0G. close checks its closed flag,
logs if not marked successful and invokes0KU.A0F. An end-transaction failure is
logged, retained while selected bookkeeping runs, then rethrown; normal close
marks it closed. The lower-level database wrapper, physical durability and
nested transaction behavior are not reconstructed here. A marked flag alone
is not proof that the success call/commit completed; that call can throw.

## Hash, metadata and follow-up are separate stages

After A0P returns, the retained delta worker writes block_list_v2_dhash via
2L8.A1U. This helper invokes putString followed by **apply**, notcommit. It
provides no Boolean flush result. Then the worker calls7to metadata handling
and queues8be(manager,changedUnion,6). These stages are not a single transaction
with wa_block_list; neither handler ACK nor worker normal return proves all
preference/metadata writes are physically durable. The API null-removal contract
was retained previously; it is not a new device execution result here.

8be's constructor binds managerA00, collectionA01 and discriminator$t. Its
run case6 selects the common branch000a..0022 ->1dH.A09. A09 skips empty
collections; otherwise it dispatches3XP(collection,7) through07o.A00. The
consumer's effects are still untraced, so this is a closed scheduling handoff,
not a claim of complete listener/network policy. The selected task excerpt
includes dispatch and the complete relevant body, not unrelated run cases.

## Per-target local system-message handoff precedes membership mutation

19N's calls to1dH.A0A(add,true)/A0A(remove,false) loop each supplied target into
1dH.A0Q. That method reads0hb.A06's latest model. If it is1ym with the same
BooleanA00, it returns without creating a duplicate. Otherwise it asks0yt.A00
for kind0x3a at the08q clock time, requires1ym (its diagnostic names
FMessageSystemBlockContact), writes the requested Boolean and hands it to
0wZ.A0C(message,-1). The shown method does not build the outbound blocklist IQ.
Downstream message storage/events/observer behavior is not traced by that fact;
do not fabricate a sent user chat message, receipt or network job as a substitute.
This side effect occurs before the handler's A0U add/remove and queued worker.

## Rust consequence and open obligations

Rust's blocklist request projector uses one atomic durable cache/hash/marker
transaction. It must not imitate native partial failure by committing an
inconsistent hash/cache, or claim it is equivalent to separate native memory,
wa_block_list, preferences, metadata and system-message state. Incoming blocklist
account_sync remains captured without ACK. Porting it still needs admission,
deduplication, frozen/correlated durable work, scheduler/uncertain-write policy,
remaining follow-up effects and ACK sequencing. No automatic queue, ACK,
identity merge, system-message feature or live request is added here.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-delta-storage-boundary-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/7PB;' --method A0P --signatures --exceptions > "$OUT/store-replace.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0C --signatures --exceptions > "$OUT/current-set.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/07Z;' --method A1S --signatures --exceptions > "$OUT/set-copy.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0A --signatures --exceptions > "$OUT/per-target-dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0Q --signatures --exceptions > "$OUT/system-message.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2L8;' --method A1U --signatures --exceptions > "$OUT/preference-apply.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0ai;' --method A00 --signatures --exceptions > "$OUT/insert-helper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1hk;' --method A00 --signatures --exceptions > "$OUT/transaction-success.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1hk;' --method close --signatures --exceptions > "$OUT/transaction-close.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A09 --signatures --exceptions > "$OUT/follow-up-dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8be;' --method '<init>' --signatures --exceptions > "$OUT/follow-up-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8be;' --method run --signatures --exceptions > "$OUT/follow-up-task.txt"
```

Extract the selected task exactly:

```sh
python3 - <<'PYCODE'
from pathlib import Path
import re
root = Path('/tmp/blocklist-delta-storage-boundary-repro')
lines = (root / 'follow-up-task.txt').read_text().splitlines()
selected = ['# Selected DEX excerpt; not a complete method.'] + lines[:8]
for line in lines[8:]:
    match = re.match(r'    ([0-9a-f]+)\.\.', line)
    if match and 0xa <= int(match[1], 16) < 0x22:
        selected.append(line)
(root / 'follow-up-task-selected.txt').write_text('\n'.join(selected) + '\n')
PYCODE
```

The earlier bundle reproduces8al.run; compare its case1 rather than duplicating
the full synthetic worker. checksums.json binds every other bundle file.
No live acceptance, full native equivalence or zero-restriction claim follows.
