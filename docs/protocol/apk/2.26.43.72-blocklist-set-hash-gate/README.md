# Set-result hash: membership change and backend success are different

**Confirmed selected pinned mismatch/correction**, WhatsApp2.26.43.72
build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX/catches plus synthetic encrypted-store tests; no live request.

## Corrected interpretation of the native gates

The retained [7vV.A00 normal success](../2.26.43.72-blocklist-set-reconciliation/set-owner-success.txt)
adds/removes the requested target in1dH.A0U and calls1dH.A08 only when that
target's Set.add/remove returned true. Associated aliases have their own
selected writes; full alias reconstruction remains separate. If the target
was already in the requested state, the shown branch skips hash persistence.

The retained [1dH.A08](../2.26.43.72-blocklist-null-snapshot-hash/set-conditional-write.txt)
then calls7PB.A0Q and writes the nullable hash only on true. New complete7PB
code disproves interpreting this Boolean as a positive affected-row count:
block invokes0ai.A01 (insert/replace helper) and unblock invokes0ai.A03 (delete
helper), discarding their long results. After normal DB close it returns true.
The selected IllegalArgumentException catch logs/returns false; other throwable
and close paths remain explicit in the catch table, not blanket-success/failure.
Both helpers forward lower-level DB results, but7PB does not compare them.

Thus the outer **memory membership change** and inner **backend execution
success** are distinct. Earlier notes calling the second a backend-change gate
were imprecise and are corrected in the same change. This does not prove a DB
row changed, an insert returned a particular value, or every native layer is
transactionally consistent. It also does not authorize replay after a partial
native write or a Rust storage error.

## Rust correction and ownership limit

The adapter uses one durable blocked-row cache, not Android's separate A0U and
wa_block_list state. Within that supported projection model, it now changes the
saved hash only when the requested target changes membership: a new block or
removal of an existing blocked row. An already-satisfied request leaves hash
and cache bytes unchanged, while still recording its valid once-only inspection.
A changed membership adopts the selected-list hash; absent hash removes the
checkpoint, matching the previously retained Android null-removal contract.
The membership/hash/inspection remain one redb transaction; write failure
cannot commit only the cache or hash. No native partial-failure emulation added.

Block admission retains the existing saved-row identity validation. Unblock
uses the existing key's presence; it does not invent a new metadata decoder or
clear unrelated rows. matched=false still bypasses requested-change/hash
adoption and holds reconciliation. First-list parsing/admission is unchanged.
This does not claim that a stale Rust cache is globally equivalent to native
memory/DB/aliases, or repair already projected historical hashes.

The new12-case regression spans block/unblock, target present/absent and
missing/nonempty/empty reply hash. It seeds an unrelated contact and target
metadata, checks unchanged cache bytes when membership does not change,
nullable/empty hash semantics, final membership, journal, encrypted reopen,
idempotent inspection and no recreated pending send. The old projector fails
the changed-membership/missing-hash case by retaining the old hash; the full
matrix also covers incorrectly adopting a new hash for unchanged membership.
The earlier first-list root-only-hash control now expects removal on a newly
applied change; a root hash still cannot substitute for the selected list.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-set-hash-gate-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/7PB;' --method A0Q --signatures --exceptions > "$OUT/backend-result.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0ai;' --method A01 --signatures --exceptions > "$OUT/insert-helper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0ai;' --method A03 --signatures --exceptions > "$OUT/delete-helper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7vV;' --method A00 --signatures --exceptions > "$OUT/set-owner-success.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A08 --signatures --exceptions > "$OUT/set-conditional-write.txt"
cargo test --locked -p whatsapp-api --all-features set_hash_updates_only -- --nocapture
```

Compare the last two exports to the linked retained methods. checksums.json
binds every other bundle file. No duplicate REA record or source-body claim.
Full typed-null parser fallback, metadata stores, alias/migration state and live
acceptance remain open; this does not establish zero account restrictions.
