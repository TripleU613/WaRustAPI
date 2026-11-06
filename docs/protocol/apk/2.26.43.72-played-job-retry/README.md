# Played receipt job retry eligibility — APK 2.26.43.72

Confirmed selected static, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods/signatures/catches with
[complete REA hook Evidence](../../rea/2.26.43.72-played-job-retry/retry.json).
No job, account or live network operation ran.

## Concrete played-job binding

SendPlayedReceiptJobV2 constructor creates DCZ.A00, builds groupId from
`played-receipt-v2-` plus EcR.A01's raw JID, and invokes6ol.A0n. That helper
sets A01=groupId, A03=true, adds ChatConnectionRequirement, then callsDCZ.A03.
The already retained [builder](../2.26.43.72-bulk-key-job-completion/parameter-builder.txt)
initializes A00=100 and passes it unchanged, with A03, into
[JobParameters constructor](../2.26.43.72-bulk-key-job-completion/parameter-fields.txt).
Those fields bind retryCount100 and isPersistent=true for this producer.
The per-chat group key is not proof of message-ID deduplication or transmission.

A0N(Exception) invokes the private diagnostic A00 then returns true without
examining the supplied exception. REA agrees with DEX. Its diagnostic helper
formats stored JID/id/count/coexistence context (diagnostic.txt); no claim
that it cannot throw is made. This hook is not an
unconditional retry loop or a predicate over server error codes.

## Shared worker gives the hook its meaning

Fresh exports byte-match prior [Qlq.run](../2.26.43.72-bulk-key-job-completion/job-runner.txt),
[sole direct base-hook caller census](../2.26.43.72-bulk-key-job-completion/retry-callers.txt),
DCZ builder and JobParameters constructor. Shared semantics were already
established for bulk-key jobs; this audit binds the played producer to them.
The worker callsA0L while Job.A00 < parameters.retryCount. On Exception it
incrementsA00; RuntimeException bypassesA0N and is rethrown. For trueA0N,
Job.A0M rechecks requirements: satisfied loops to the budget check; unsatisfied
produces DEFERRED and requeues the same job. NormalA0L return produces SUCCESS.

Fresh ChatConnectionRequirement export matches its
[retained full implementation](../2.26.43.72-chat-job-requirement/requirement.txt):
BWq uses0AI.A0O, which checksA04==2. That predicate is separate from A02=12
in the [dispatch drop branch](../2.26.43.72-receipt-dispatch-failure/README.md).
Do not substitute a similarly named field for connection admission. Queue
requeue/wake remains as [previously traced](../2.26.43.72-jobqueue-wake/README.md),
not proof that a wire write occurs or the server accepts it.

## Rust comparison: known native retry mechanism remains unported

This establishes native played-job exception retry eligibility, rather than
leaving the entire policy unknown. Rust currently preserves uncertain receipt
reservations and does not emulate this native retry loop. That is an explicit
compatibility gap, not proof that Rust has complete APK lifecycle parity.
Copying the100 budget into a resend loop would omit native source, callback,
queue and connection authority; no such change is made.

Prior selected full-key callback completion can release the played job normally
without interpreting an outer numeric error. Future success, job SUCCESS,
receipt acceptance and local diagnostic refused are different layers. Retrying
local exceptions is not evidence that a server refusal should be replayed.
A100 attempt budget does not mean100 observed sends, a retry delay, or safe
replay after partial output. Current receipt writes retain their original IDs,
wire, durable ownership and uncertainty. No reset or new retry is introduced.

Unknown/unported: complete source-row eligibility across attempts, all played/
played-self routing and policy gates, effective provider/account generation,
restart deduplication, callback replacement and safe retry after partial writes.
The hook/budget/requirement are now established; those unresolved authorities
remain necessary before native retry behavior can be reconstructed safely.

## Reproduction

From repository root, pinned APK and current built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/played-job-retry-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/messaging/receipts/jobqueue/job/SendPlayedReceiptJobV2;' --method A00 --signatures --exceptions > "$OUT/diagnostic.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/messaging/receipts/jobqueue/job/SendPlayedReceiptJobV2;' --method '<init>' --signatures --exceptions > "$OUT/constructor.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/messaging/receipts/jobqueue/job/SendPlayedReceiptJobV2;' --method A0N --signatures --exceptions > "$OUT/retry.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6ol;' --method A0n --signatures --exceptions > "$OUT/parameters.txt"
```

checksums.json binds every other file here. Shared evidence remains in original
checksum bundles. Source evidence does not establish account restriction rates.
