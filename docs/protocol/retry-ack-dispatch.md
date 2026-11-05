# Retry receipt acknowledgement dispatch — 2026-11-05

**Confirmed static / implementation gap:** pinned APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained exports, exact branches, reproduction and limits](apk/2.26.43.72-retry-ack-dispatch/README.md).
No account traffic or runtime execution was used.

Rust currently does **not** ACK retry receipts: receipt_ack uses receipt_kind's
explicit delivery/read/played allowlist. The hypothesis that malformed retries
receive an ordinary success ACK was refuted by the current source and existing
retry-exclusion regressions. The normal receive driver calling the confirmation
helper does not itself prove a confirmation is constructed.

The APK selected base handler distinguishes true completion, false/unhandled,
and LIL parse exceptions. False invokes2Cx.A00, which adds error487; its490 value
is a local diagnostic. LIL on the non-BX2 path reaches1rg.A04, whose allowlist
explicitly contains receipt and which calls its parsing-error helper with487.
Successful handler return can submit a plain confirmation even when retry work
was merely scheduled. BX2 uses separate completion/error queues. Exact final
Message76-to-writer binding and asynchronous ordering still require tracing.
Do not turn Rust's local unsupported keys/count/recipient limits into native
malformed-input errors, or ACK every receipt indiscriminately.

The actual selected base-handler receipt context factory is now located:
14h.BLn callsF0R.A03, which delegates toF0R.A00, reads root from/id/participant/type,
defaults absent type to delivery, and adds recipient/category/edit before freeze.
This supersedes the earlier candidate-only factory search for that entrypoint;
complete registration, alias policy and full dispatch remain open.

This research chunk changes no runtime policy. Next: bind Android Message76 to
the final ACK writer, then introduce separately evidenced retry response
classification and ordering without converting local capability refusals to
protocol errors. Existing supported receipt behavior remains unchanged.

Validation: five filtered receipt-ACK tests passed with all features. APK coverage,
research checksums/local-link hygiene and staged whitespace checks passed.
No shared Rust source changed; the workspace suite was not rerun for this
research-only chunk.
