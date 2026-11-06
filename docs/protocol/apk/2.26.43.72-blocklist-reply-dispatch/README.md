# Blocklist reply callback dispatch and coded-error ownership

**Confirmed selected static / fixed offline**, WhatsApp2.26.43.72
build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution, live request or account-state experiment. Complete selected
DEX exports, not new REA Evidence envelopes.

## Result and error callback policies are distinct

Reuse [IQ reader admission](../2.26.43.72-iq-reader-admission/README.md),
[set/fetch CK4 callbacks](../2.26.43.72-blocklist-set-reconciliation/README.md)
and the later first-list/first-named-list corrections. CK4 processes the selected
list; unrelated root error extensions and nested user errors do not select Bvj
instead. Missing/malformed selected lists still have their existing validation
or unchanged-fetch policy; an outer result alone is not proof of a valid cache.

Both8R7.Bvj and6p9.Bvj call1ud.A00. Reused complete
[numeric helper](../2.26.43.72-status-privacy-fetch-callback/error-code.txt)
and [coded-error parser](../2.26.43.72-prekey-upload-replies/error-parser.txt)
were freshly compared byte-for-byte. A00 uses the Pair's numeric first value or
zero if absent. A01 asks0Tp.A0N(error) for children and iterates until a code
string is nonnull, then returns0HF.A00(code,0) and that same child's optional
text. It skips uncoded errors, not malformed numeric strings with a present
code. Exact0HF integer syntax/range remains untraced. New complete A0N proves
this is an ordered direct-child filter, not a descendant search.

## Error400 can queue a separate reconciliation fetch

8R7.Bvj obtains the code, calls6pA.A03(id,update) regardless of code, then
branches: only code400 **and** A03=false call7vW.A00 and return. Other codes or
A03=true log generic failure and schedule7vV.A0F. The
[retained admission helper](../2.26.43.72-blocklist-set-reconciliation/reconcile-admission.txt)
reads native blocklist mode/global migration state; Rust has no equivalent
proven state authority. Do not turn all400 responses into retries.

7vW.A00 calls1dH.A0F then A0M(null,null), resets its owner's A00=false and
schedules the failure runnable. A0F sets block_list_v2_dhash null via apply,
then separately removes block_list_receive_time via apply. This is a new full
fetch through the retained producer, not retransmission of the original set.
It is not proof of atomic preference/job persistence or full recovery. The
failure runnable's complete UI/downstream behavior remains open. Branches:
0022+0xa*2 and0026+0x8*2 both reach0036 generic failure; fallthrough002a
reaches reconciliation only for400 with false admission.

6p9.Bvj logs the numeric code and, when7sa is present, completes its FfK with a
new7gZ exception; it does not perform this set-only400 reconciliation branch.
No generic fetch retry is inferred from that exception callback.

## Confirmed Rust bugs and scoped correction

The generic action settler previously searched nested user/type=error children
before root errors, even for owned blocklist replies. Consequently an unrelated
error extension could convert a valid list result to ActionStatus::Error and
skip cache projection. On outer error, a nested401 could displace the native
first direct coded400, or an uncoded first error could hide later diagnostics.

Settlement now recognizes only blocklist/block/unblock action kinds whose saved
request passes the existing blocklist IQ ID/type/namespace/destination check.
For those requests an outer result stays Ok for the separate list projector;
outer errors select the first direct error child with a textual code, skipping
uncoded children. Unrelated action families retain their own refusal rules,
including app-state collection errors. Shared request validation is internal,
not a new library/CLI capability.

Numeric diagnostic conversion remains the existing Option<u16> parser; absent,
invalid, negative and out-of-range code/text-materialization parity is not
claimed. No automatic400 reconciliation, hash clearing, migration-mode guess,
request replay or ACK behavior is introduced. A malformed selected list remains
unprojected with its durable reply preserved. The error action remains Error;
it cannot apply the originally requested block/unblock or replace the cache.

## Regression and reproduction

The two initial new regressions failed against the previous actions.rs:
result expected Ok but got Error; coded error expected400 but got nested401.
After the correction they pass. A third regression verifies result dispatch
does not bypass malformed-snapshot refusal. Matrices cover fetch/block/unblock,
root/nested error extensions, uncoded-prefix and multiple coded errors,
encrypted-store reopen, once-only projection, preserved hash/cache on refusal,
no duplicate settlement and no requeued action. Malformed projection checks
all checkpoint bytes and journal head before/after refusal and after reopen.

```sh
cargo test --locked -p whatsapp-api --all-features blocklist_reply_
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-reply-dispatch-repro
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/8R7;' --method Bvj --signatures --exceptions > "$OUT/set-error.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6p9;' --method Bvj --signatures --exceptions > "$OUT/fetch-error.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0N --signatures --exceptions > "$OUT/error-children.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7vW;' --method A00 --signatures --exceptions > "$OUT/reconcile-fetch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0F --signatures --exceptions > "$OUT/forget.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1ud;' --method A00 --signatures --exceptions > "$OUT/error-code.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1ud;' --method A01 --signatures --exceptions > "$OUT/error-parser.txt"
```

Compare the first five exports to this bundle and the last two to the reused
files above. checksums.json binds every other file in this bundle. Native
migration, full notification processing and every other request family remain
outside this correction; no complete audit or ban-prevention guarantee.
