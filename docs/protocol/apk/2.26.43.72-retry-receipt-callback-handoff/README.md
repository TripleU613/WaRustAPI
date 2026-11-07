# Retry-receipt callback registration and job wait

**Confirmed selected static DEX**, WhatsApp2.26.43.72, code264307238,
APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Source baseline `0f61580f`. Repository offline DEX inspector, not target execution
or successful REA Android output. The previously observed [REA Java-path failure](../2.26.43.72-retry-key-envelope-bytes/rea-provider-attempt.json)
remains separate; no tool configuration changed. No live account operation.

## Selected chain

[Retained SendRetryReceiptJob.A0L](../2.26.43.72-outgoing-retry-timestamp/job-run.txt)
constructs the Eoh receipt/retry correlation key at005c..00aa. At038c..039a it
calls0Sf.A0B(Message,Eoh), then FfK.get() without a timeout, then returns normally.
This is not a local-write completion or an evidenced request deadline.

[0Sf.A0B](send-register.txt) calls0Sf.A02 on the key, creates a fresh FfK,
registers it via0Th.A01 at0024..002e, and only then dispatches0Sf.A05 with both
boolean arguments false at0050..0058. It returns the same future. Exceptions,
partial dispatch and later callback delivery remain outside this method.

[0Sf.A02](stanza-key-normalization.txt) conditionally projects device0 JIDs
in the Eoh A02/A01 positions to their userJid, rebuilding via Efa as necessary.
The A02 position first uses0Dn.A0S before a DeviceJid cast; the A01 position
uses DeviceJid.Companion.A00. This is callback-key normalization, not proof
that all wire attributes, account ownership or nonzero devices are normalized.
No Rust matching change is inferred solely from this selected method.

[0Th.A01](callback-register.txt) uses **0Th.A04**, synchronized on that map.
It checks containsKey and reports duplicate_ackable_stanza if present, then on
normal continuation still puts a new2Gc wrapper under the key. The prior value
returned by Map.put is ignored. A duplicate is not a successful deduplication
or a demonstrated cancellation of the previous future. The wrapper carries the
original1ue callback, manager, selected telemetry object and elapsedRealtime.
A01 supplies wrapper mode0. This map is in-memory; no durable reservation or
restart reconciliation is established by this registration method.

[2Gc](callback-wrapper.txt) stores the original callback in A01. BsR delegates
the exception unchanged. On BsT, mode0 forwards **null**, while nonzero mode
forwards the supplied object; both continue through elapsed-time/telemetry work.
[Alternate0Th.A02](alternate-callback-register.txt) uses the same map with mode1;
it is not the selected retry job's registration call. This negative selector
result is retained to prevent confusing the two nearly identical methods.

The [existing concrete FfK evidence](../2.26.43.72-bulk-key-job-completion/README.md#concrete-future-effects-and-response-path)
shows BsT stores success/counts down its latch, BsR stores failure/counts down,
and get() waits, returning the stored value or wrapping failure in
ExecutionException. Reuse that implementation evidence; a normal job return
through mode0 is not proof of useful response payload or server acceptance.

## Separate callback maps and unresolved consumers

The [retained0Th.A03](../2.26.43.72-retry-refetch-callback-lifetime/callback-manager.txt)
disconnect failure method iterates **A05**, calls BsR and clears it. Bulk-key
queries use that A05 map. Retry receipt registration above uses **A04** instead.
Therefore that bulk-query disconnect/failure trace cannot establish retry-receipt
callback failure or cleanup. Likewise the bulk message8 completion is not enough
to bind receipt ACK settlement. The concrete A04 ACK/removal/disconnect consumers,
full job retry decision, original-callback lifetime after duplicate replacement,
watchdogs and native crash recovery remain unresolved by this bundle.

Rust already uses immutable durable receipt callback snapshots, account authority,
exact member checks and no replay after uncertain writes. Those are separately
tested local invariants. Do not replace them with native in-memory Map.put or add
an automatic resend because this job blocks on a future. No runtime policy changes.

## Reproduction

Use the class/method/full signatures in each text file with:

```
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method A0B --signatures --exceptions
```

The wrapper export is the complete selected2Gc class (three methods), not a bulk
unpacked tree. Instruction coordinates use inspector byte rendering; exception
comments explicitly use code units. checksums.json pins the compact exports.

The initial signing question did not need a new trace: existing [PQ signing
provenance](../2.26.43.72-prekey-upload-replies/README.md#pq-signing-and-wire-byte-provenance-2026-11-04)
already binds EqE's entire generated public array to signing/storage and0Yj.A00
reuse. Native primitive equivalence and complete lifecycle remain separate.
