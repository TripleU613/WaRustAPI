# Current-build retry-answer count evidence

APK: **2.26.43.72** / SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
REA 6.5.0 / headless JADX 0.7.1. Static observations only.
[Interpretation, Rust change and remaining gaps](../../retry-enc-count.md).

| Record | Exact current selector | Role |
| --- | --- | --- |
| [receipt-parser.json](receipt-parser.json) | `X.F10.A04`, overload 0 | Reads `retry.count`, default 1 |
| [parsed-receipt-constructor.json](parsed-receipt-constructor.json) | `X.EYC.<init>`, overload 0 | Stores parsed count as `A00` |
| [receipt-handler.txt](receipt-handler.txt) | `LX/160;->A04(LX/0Tp;LX/Eoh;)Z` | Preserves parsed integer and passes it to runnable |
| [retry-runnable.txt](retry-runnable.txt) | `LX/Fe5;` constructor and `run()V` | Preserves count into send-job's second integer |
| [send-job-constructor.json](send-job-constructor.json) | `SendE2EMessageJob.<init>`, overload 0 | Stores second integer as `retryCount` |
| [count-to-enc.json](count-to-enc.json) | `X.FTy.AK7`, overload 0 | Sends job count to encryption-child builder |
| [enc-node.json](enc-node.json) | `X.F3f.A04`, overload 0 | Constructs `enc` with attribute helper |
| [enc-attributes.json](enc-attributes.json) | `X.F3f.A09`, overload 0 | Nonzero decimal `count`; zero omitted |
| [base-key-policy.json](base-key-policy.json) | `X.AnonymousClass163.A04`, overload 0 | Count-2 save / count>2 comparison policy |
| [base-key-save.json](base-key-save.json) | `X.C0WJ.A0s`, overload 0 | Per-message/device base-key persistence |
| [base-key-compare.json](base-key-compare.json) | `X.C0WJ.A15`, overload 0 | Saved vs current session base-key comparison |
| [send-job-run.json](send-job-run.json) | `SendE2EMessageJob.A0L`, overload 0 | Complete smali; count>4 abort bound |

[root-normalization-unresolved.json](root-normalization-unresolved.json) retains
current `X.FTr.ATZ` completely, including JADX duplicate-block warnings. Its
retry-root removal/addressing branches remain uninterpreted and unported; this
record has no Rust ownership mapping and does not expand the completed correction.

Two additional REA records, [receipt-handler-unavailable.json](receipt-handler-unavailable.json)
and [retry-runnable-unavailable.json](retry-runnable-unavailable.json), preserve
provider no-body results. Actual DEX code exists, so these are provider limitations,
not evidence of native/abstract methods. The checked text files fill that source gap.
Each JSON record carries its Evidence ID, APK digest, method identity and coverage.
Ownership sidecars describe relevant Rust files, not complete Rust semantic parity.

Reproduction with the exact APK outside the checkout:

```sh
target/debug/xtask apk-xref /path/to/WhatsApp.1.apk \
  --anchor 'RetryReceiptHandler/axolotl resending '
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/160;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/Fe5;' --signatures --exceptions
```

Source hashes (the text is complete stdout of the checked extractor):

- `receipt-handler.txt`: `663f73c8e4a54687495a3390440b8e1341146aacc34aefd86e5c7e3becf5210b`
- `retry-runnable.txt`: `4dfedd304182f39ec138c92b110112c4e3282e2c2adadf340cc9eafa02748be3`

The current receipt handler is found through the current runnable's incoming
constructor reference (REA `trace_android_references` for `X.Fe5.<init>`, exact
`(LX/163;Lcom/whatsapp/infra/core/jid/DeviceJid;Lcom/whatsapp/infra/core/jid/UserJid;LX/18U;LX/1dW;LX/ETs;LX/Eoh;Ljava/lang/Integer;[BIJZZZ)V`).
Neither old obfuscated names nor matching string names establish cross-release
semantic equivalence. Full `SendE2EMessageJob.A06` decompilation was partial after
JADX warning expansion and is retained only in local research output, not used as
complete evidence here.
