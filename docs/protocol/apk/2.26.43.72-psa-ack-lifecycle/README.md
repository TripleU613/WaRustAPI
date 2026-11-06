# Current PSA work and ACK lifecycle

**Confirmed selected static**, APK2.26.43.72 build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete focused DEX methods and exact-string locator. No new REA Evidence
record, APK execution, account operation or production ACK-policy change.

The [canonical branch matrix and Rust gap](../../psa-ack-lifecycle.md) distinguish
native feature selection, deferred work, local one-shot ownership and wire ACK.
19n constructor supplies handler id104 (0x68), matching psa in the retained
[type map](../2.26.43.72-group-missing-identification/notification-type-map.txt).
Its Boolean superclass argument is retained without guessing its full policy.
The property provider08Y is bound in this constructor; effective values remain
unknown, not inferred from Rust's missing features or compiled defaults.

Reuse complete [0v6.A01](../2.26.43.72-identity-ack-dispatch/base-handler.txt),
[89Y.A00](../2.26.43.72-prekey-upload-replies/prekey-ack-producer.txt), and
[89Y.A01](../2.26.43.72-prekey-count-ack-source/deferred-ack-entry.txt).
Fresh exports matched each byte-for-byte. A07(true) transfers the current token
from atomic A0C to deferred A0A and creates89Y/7p2 ownership. Normal base
post-handler processing does not ACK a transferred token a second time.
A07 also schedules a6000 task when applicable. Its diagnostic-only action is
already established in the [watchdog bundle](../2.26.43.72-deferred-ack-watchdog/README.md);
the earlier omission of that retained proof is corrected. No Rust timer follows.

19n.A03 builds error=487 attributes and calls89Y.A00. That completion consumes
its owner under synchronization, removes deferred membership, overlays attributes
and invokes0v6.A00. Later completion on that same consumed token does nothing.
This is local one-shot work ownership, not durable exactly-once delivery or
proof a socket/server accepted the ACK. Lost-write/restart/retry policy remains
separate. Base/corrupt-content policy is not generalized to every worker.

Fd3.run decodes selected message bodies, constructs native messages and calls
1tj.A0e before its normal-tail89Y.A01. It has no exception handlers. Missing
message id reaches an IllegalArgumentException before that tail; protobuf/
message processing failures may also exit without reaching it. Calling A0e is
not proof of a durable commit, successful publication or full PSA application.
Enabled surfaces and revoke/status workers remain untraced beyond the
producer's selected runnable/callback binding. Wakeup completion joins are
retained in the follow-up below; full effective policy remains unported.

The previous Rust comment's universal plain-base-ACK/exactly-once wording is
not a current-build specification. Existing helper still uses its older bounded
policy; runtime behavior is deliberately unchanged in this evidence chunk.
The current gap is recorded, not called resolved, and487 is not broadened from
unknown feature flags. No native side effect or completion is fabricated.

## Reproduce

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-psa-lifecycle-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/19n;' --method 'A09' --signatures --exceptions > "$OUT/handler.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/19n;' --method '<init>' --signatures --exceptions > "$OUT/handler-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/19n;' --method 'A03' --signatures --exceptions > "$OUT/refusal.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0v6;' --method 'A07' --signatures --exceptions > "$OUT/defer.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Fd3;' --method 'run' --signatures --exceptions > "$OUT/message-worker.txt"
target/debug/xtask apk-xref "$APK" --anchor psa_wakeup > "$OUT/wakeup-xref.txt"
```

Compare every retained .txt byte-for-byte. Shared complete method reproduction
is documented under the linked bundles. checksums.json covers every retained
file except itself. No raw APK, message data, private state or bulk tree.

Verification: all six new exports reproduce exactly; the three reused base/
completion exports match their retained sources byte-for-byte. The existing
PSA helper test, fmt, OpenAPI library-symbol and CLI-reference tests, APK
coverage freshness and repository hygiene passed. These checks confirm retained
evidence and unchanged legacy behavior, not current PSA compliance. No fresh
full-workspace test count or native runtime observation is claimed.

## Wakeup follow-up (2026-11-06)

[Selected worker and scheduler](../2.26.43.72-psa-wakeup-completion/README.md)
now retain wakeup normal/Exception joins and promotion rendering selection,
without inventing remote-fetch acceptance or effective flags. Other workers,
durable application/recovery and full PSA policy remain open.
