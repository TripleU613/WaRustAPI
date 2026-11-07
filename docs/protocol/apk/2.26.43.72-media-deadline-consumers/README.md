# Media deadline consumers and replacement-auth waits — APK 2.26.43.72

**Confirmed selected static / research-only boundary**, build264307238;
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods and exact field-reference results via xtask;
no target execution, account traffic or REA Evidence IDs. No Rust runtime change.

## Exact declared field consumers

[A04 field census](A04-field-refs.txt) finds two references: model constructor
write and0VM.A0T read. Thus the selected DEX route-TTL deadline reader is the
almost-expired/refresh decision, not every transfer's hard expiry predicate.
[A02 census](A02-field-refs.txt) finds ten references: one constructor write,
reads in0VM.A06/A08/A0C/A0R/A0T, MOu.A05, MtZ.Cb6 and TWO Mtv.CW7 reads.
This is exact DEX field coverage, not an exhaustive reflection/JNI/runtime census.
The [model constructor](../../rea/2.26.43.72-media-route-lifetimes/model-checked.txt)
independently multiplies ttl/auth_ttl by1000 and adds the request base, producing
A04/A02 respectively. Earlier prospective merge/callback provenance remains.

## Refresh trigger is distinct from hard auth expiry

[0VM.A0T](0VM-A0T.txt) first invokes cache restore A08 then obtains current route.
No route OR A04 <= elapsedRealtime()+120000 OR A02 <= elapsedRealtime()+120000
selects A07 request and returns true. Both deadlines strictly beyond their own
sampled now+120000 paths return false; when no Handler message0 exists they first
call A06. The120s boundary is a proactive request trigger, not proof that upload
auth becomes invalid120s before its deadline.

[0VM.A06](0VM-A06.txt) removes Handler message0 and schedules it with raw signed
A02 - elapsedRealtime() -60000. This method does not clamp that delay. Handler
behavior and the receiving message callback remain outside this selected export;
no Rust timer, periodic request or global pacing rule is added.

[0VM.A0C](0VM-A0C.txt) returns true only if candidate is nonnull, auth differs
from a supplied old string, and A02 > elapsedRealtime()+margin. A null old string
therefore imposes no old-token equality match. Same auth is false even if a new
response extended its lifetime. This is a caller-specific replacement predicate,
NOT evidence to reject every media_conn reply carrying the same auth.

[0VM.A0R](0VM-A0R.txt) requests through A0T. When refresh was requested AND
current auth is missing/expired, it calls A0B(null, margin0, budget20000).
Still-future current auth can avoid this wait even when route refresh was
triggered by the120s boundary. Strict equality at auth deadline is expired.

## Wait tracks request flight and a separate clock

[0VM.A0B](0VM-A0B.txt) captures uptimeMillis, checks A0C, and if unsatisfied
obtains1xY via A03. Nonpositive1xY.A00 under its monitor logs no request in flight
and returns. With a flight, it waits on A0I for min(1000,budget), catches/logs
InterruptedException, checks A0C again, and compares start+budget against uptime.
The expiry check is strict past-budget, with waits in chunks rather than exact
remaining time. A20000 input is a wait budget, not a proven hard wall-time upper
bound: scheduling and chunk overshoot remain. The wait uses uptime while auth
comparison uses elapsedRealtime. Do not substitute a request timeout or mix the
two native clocks. No offline busy-loop or sleep port is introduced.

## Selected transfer/supplier paths and gates

[MOu.A05](MOu-A05.txt), selected MediaUpload routine, resets A0c and reads feature
0tF.A0L. Its enabled branch calls A0T, then may wait via A0B(null,120000,20000)
before re-reading auth and checking A02 strictly future. The other branch calls
A0R (margin0 path). At01c4 onward an expired/missing-auth case has further
LnX.A1U/online/offline logic. Offline handling can record offline_skip and set
A0c at051a. These branches are not reduced to a universal early refusal; exact
feature/provider state and producer binding remain unobserved.

[MtZ.Cb6](MtZ-Cb6.txt) has additional outer4ab3/fallback decisions. On its
selected post-offline-skip path, when MOu.A0c is true, it re-reads current route,
compares current auth to request M3d.A01 and requires A02 > elapsedRealtime to
clear the flag. Otherwise an online predicate can return a local failure13
without transfer. This demonstrates selected pre-transfer auth identity/expiry
rechecking; it is not proof that this check runs unconditionally for every upload.

[Mtv.CW7](Mtv-CW7.txt) additionally requires A0N, prior auth, prior M3d and feature
0tF.A0L. It can request/wait for replacement auth; later adoption requires auth
different from its old token and A02 strictly future, then rebuilds M3d. Its120s
predicate triggers refresh; actual replacement adoption uses future expiry. Full
Mtv mode/producer/runtime scope (including possible non-upload uses), A04 observer
semantics and downstream transfer completion remain open.

[0VM.A08](0VM-A08.txt) is the complete DEX counterpart to earlier retained cache
reader evidence, preserving its gate/restore/exception branches. The new export
corroborates that earlier analysis rather than declaring a newly observed cache
state or re-running an unavailable decompiler provider.

## Rust consequence and next required work

Current Rust explicitly refreshes and retains owner/request/prior-route binding,
and upload route selection has local request-age/min(ttl,auth) eligibility with
the corrected signed multiplication. That minimum remains a LOCAL conservative
policy; A04's sole direct reader does not prove it is an unconditional native
hard upload cutoff. Automatic120s requests, the60s timer, request-aware20s wait,
old-token replacement condition, offline-skip state machine and pre-transfer
rechecks are not fully ported. These findings refute both a blanket120s auth
refusal and treating route TTL expiry as identical to hard auth expiration.

Before any runtime port: bind actual producer/mode and feature conditions, keep
request-flight/throttle/account ownership, distinguish old-token replacement from
ordinary response adoption, choose a supported elapsed/uptime/reboot policy, and
retain capture, failure and uncertain-write/no-replay semantics. The HTTP upload
preparation-to-transfer window needs a separate current-auth review; do not
silently switch URLs/tokens or invent account credentials. No global timer or
new request is authorized merely by finding these methods. No ban guarantee.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-field-refs "$APK" --class 'LX/1t5;' --field A02 --type J
target/debug/xtask apk-field-refs "$APK" --class 'LX/1t5;' --field A04 --type J
# Retained complete methods:0VM A06/A08/A0B/A0C/A0R/A0T; MOu A05; MtZ Cb6; Mtv CW7.
target/debug/xtask apk-smali "$APK" --class 'LX/0VM;' --method A0T --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0VM;' --method A0B --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/MOu;' --method A05 --signatures --exceptions
```

All selected method bodies are complete. checksums.json covers every other file.
No APK/private state/bulk tree is included. Static discovery is not server or
Android runtime acceptance; no live account operation occurred.
