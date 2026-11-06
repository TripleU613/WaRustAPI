# Blocklist set-result reconciliation — APK 2.26.43.72

Confirmed selected static, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods/catches/censuses, with
[complete REA set-success Evidence](../../rea/2.26.43.72-blocklist-set-reconciliation/README.md).
No live accounts, block operations, fetches or APK execution.

## A result can require reconciliation instead of the requested change

8R7.CK4 selects the first child, requires list, parses80Z via6qD.A02 and
compares matched to literal false. Non-false takes00c8..00e6 ->7vV.A00.
False instead reads c_dhash and saved09S.A0c. Failed6pA.A03 admission calls
7vW.A00. Equal hashes pass parsed878/dhash/set plus false into7vW.A01;
different hashes pass null dhash plus true. REA agrees with the checked DEX.
This is not unconditional success projection from type=result.

7vW.A01 compares the requested block/unblock flag with returned set membership
and sets its owner's success flag. It schedules8ak selector1; the retained
constructor binds the fifth argument to $t and the final Boolean to A04. That worker's
true branch calls1dH.A0M to fetch, while false schedules8bU with the supplied
set/hash/metadata. Full downstream8bU, migration modes and all effective policy
remain unported; do not relabel every mismatch a failed network action or
replay the original set. The full export retains other synthetic selectors,
which are not interpreted as this path.

7vV.A00's ordinary requested-change path adds/removes the target and, depending
on its argument,0Ws.A0P aliases under the1dH monitor; later notification effects
are separate. False-matched CK4 does not unconditionally take that path.

## Receipt block-state source: bounded producer evidence

1dH.A0U has14 direct field-reference instructions. The constructor is the only
field assignment; method bodies can mutate the Set through references. A0P
adds a supplied UserJid; direct calls are both2Ic.C10. A0O computes differences
then clears/adds the supplied replacement set at0274..0280. Its three direct
callers are6p9.CK4,88m.A00 and8bU.run. A0E snapshots/clears it and posts work.
This is not exhaustive aliasing/reflection/native producer coverage or proof
of a universal initialization order. Initial wire-send code was not proven to
optimistically modify the set; no such claim is made.

6p9.CK4's no-list branch updates receive time/completion and returns without
replacingA0U. Its shown admitted list branch reachesA0O with parsed set/hash.
Migration flags can route elsewhere, so this audit does not establish a full
native loader or effective PN/LID alias authority. The prior
[played predicate](../2.26.43.72-played-privacy/README.md) reads this set through
1dI/1dH.A0T. Rust's settled-reply projection is therefore not yet a proven
complete equivalent suitable for inventing all native receipt block policy.

## Confirmed Rust projection correction

Previously apply_block_result_in applied the immutable requested block/unblock
and adopted reply hash for any settled IQ result, including matched=false.
That could falsely add/remove a cached contact or advance an unconfirmed hash.
It now leaves the blocked rows/hash intact, records a durable local
reconciliation_required flag and commits the once-only inspection marker in
one transaction. The network action remains Ok for its IQ result; local cache
projection is not claimed successful. No original action is requeued.

CLI blocklist exposes the fixed Boolean and a text warning. A later admitted
full fetch snapshot clears the warning atomically with replacing the cache;
a no-list/unchanged fetch, merely queuing a refresh or forgetting the hash does
not. Existing --refresh --full can explicitly queue that fetch; no request is
queued automatically by reconciliation. False-matched source replies remain
inspectable and are never applied later after a new snapshot. Malformed store
or snapshots retain their existing rollback/refusal behavior.

Limits: this is conservative detection/visibility, not the native conditional
snapshot application, migration reconciliation, alias expansion or automatic
fetch policy. Already marked historical false-matched projections are not
rewound; a fresh admitted full snapshot is needed to correct those caches.
Other historical no-list/root-hash set-result compatibility remains unchanged;
current native CK4's first-list admission is not claimed fully ported.

## Reproduction

From repository root with pinned APK and current built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-set-reconciliation-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/8ak;' --method '<init>' --signatures --exceptions > "$OUT/reconcile-worker-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8R7;' --method CK4 --signatures --exceptions > "$OUT/set-success.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7vV;' --method A00 --signatures --exceptions > "$OUT/set-owner-success.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7vW;' --method A01 --signatures --exceptions > "$OUT/reconcile.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8ak;' --method run --signatures --exceptions > "$OUT/reconcile-worker.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6pA;' --method A03 --signatures --exceptions > "$OUT/reconcile-admission.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0O --signatures --exceptions > "$OUT/replace-set.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0P --signatures --exceptions > "$OUT/add-set.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0E --signatures --exceptions > "$OUT/clear-set.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6p9;' --method CK4 --signatures --exceptions > "$OUT/fetch-success.txt"
target/debug/xtask apk-field-refs "$APK" --class 'LX/1dH;' --field A0U --type 'Ljava/util/Set;' > "$OUT/set-refs.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/1dH;' --method A0P > "$OUT/add-callers.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/1dH;' --method A0O > "$OUT/replace-callers.txt"
```

checksums.json binds every other file here by bytes/SHA256. Native observations
cannot establish account-restriction rates or live acceptance.
