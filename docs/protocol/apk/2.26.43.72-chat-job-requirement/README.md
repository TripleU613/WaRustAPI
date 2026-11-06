# Chat job requirement and queue-admission boundary — APK 2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence/runtime observations.
No account operations or Rust runtime changes.

## Reuse the completed bulk-future trace

A freshBulkGetPreKeyJob.A0L export byte-compares equal to the retained
[bulk wire export](../2.26.43.72-retry-key-query-wire/bulk-prekey-job-run.txt).
[Prior concrete completion/runner research](../2.26.43.72-bulk-key-job-completion/README.md)
already binds insertion in0Th.A05, untimedFfK.get, disconnect failure, null-valued
result/error completion,100-attempt budget, exception retry/defer and regular
versusPQ-upgrade policy. None is newly inferred or reimplemented here.

## Requirement means a specific local state, not full write readiness

ChatConnectionRequirement.Cf0 obtains0AI through00D.A02(0x101) and stores it.
BWq requires that field nonnull and returns0AI.A0O, which testsA04==2 exactly.
It does not inspect callback ownership, key/pin validity, writer readiness,
queue/deferred depth, account generation or durable outcome. Global provider
identity/registration and atomic lifecycle ordering remain untraced.

[Existing readiness producer](../2.26.43.72-client-auth-token-readiness/README.md)
retains0AI.A0M writingA04=2, openingConditionVariable and notifying2Gb selector2.
Its two physical calls are in0YB.A01. As already established, it can precede
later connected-handler work; job admission must not be renamed complete
bootstrap, acceptance or safe replay. The A04 write census here has five
physical sites (initializer,A0M,0YU.handleMessage,1iM.A02,1iM.A04).
A02 is a DIFFERENT0AI field used by other queue/drop logic; it is not this
requirement predicate. Complete concurrent state transitions remain open.

1iM.A03's name/log saysready but its retained body only conditionally invokes
0oc.A00 when refresh_broadcast_lists preference is true; it does not write
A04 or callA0M. This bounded negative prevents substituting a log label for
the actual predicate producer. Full0oc effects are untraced.

## Constructor census limitation and actual allocation evidence

Exact ChatConnectionRequirement.<init> caller census is zero. That does NOT
mean this requirement is unused: the previously retainedDCZ.A02 at0000 creates
the class, then invokesObject.<init> directly before adding it to the builder.
The [complete builder](../2.26.43.72-bulk-key-job-completion/parameter-builder.txt)
and actualBulk constructor binding remain the positive evidence. Searching
only exact constructor invocation would miss this optimized allocation.
A broader ChatConnection class-substring query also returnedRestoreChatConnectionWorker;
that class name alone establishes no jobqueue requirement provider binding.

## Selected queue loop rechecks requirements without a wire guarantee

Exact Requirement.BWq invocation census finds six physical sites: baseJob.A0M,
NhM.run's blocked-requirement diagnostics and four specialized message/status
jobs. This is not a list of all runtime implementations. Existing baseA0M
iterates requirements and returnsfalse on unmetBWq.

The completeNhM.run export blocks/closes itsConditionVariable, scans saved
candidate jobs and invokesJob.A0M at00aa, with separate group-availability and
A06-dependent deterministic-selection branches. A later diagnostic branch
calls individualBWq to report blocked requirement names. Selected handoff
at0370..037a usesSynchronousQueue.put, after selection; it is not a packet
write or an ACK. The30000ms comparison at038c is ProducerPutSlow logging,
not a network timeout or retry delay. FullA06 policy, queue persistence,
observer-to-condition wake and selected job identity across runner handoff
remain untraced. No global eligible-job/reconnect ordering proof is claimed.

## Rust comparison and remaining authority

The existing durable-receipt-owner reopen regression passed unchanged. That
is a Rust control, not native job execution. A local connected predicate and
requirement recheck do not justify consuming an old durable reservation,
rotating IDs, resending after uncertainty or treating an IQ future as key
acceptance. No guessed deadline, retry loop or live operation is added.
Remaining work: actual requirement observer/wake registration, provider identity,
queue persistence/dedup/backoff, account/connection generation and atomic writer
admission. The broader outbound/inbound audit remains incomplete.

## Reproduce

From checkout root with pinned APK outside Git; compare complete outputs:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'ChatConnectionRequirement;' --signatures --exceptions > requirement.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0AI;' --method A0O --signatures --exceptions > predicate.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0AI;' --method A0M --signatures --exceptions > state-ready.txt
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/0AI;' --field A04 --type I --writes-only > state-writes.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/0AI;' --method A0M > ready-callers.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/infra/jobs/requirements/ChatConnectionRequirement;' --method '<init>' > constructor-callers.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'Lorg/whispersystems/jobqueue/requirements/Requirement;' --method BWq > check-callers.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/NhM;' --method run --signatures --exceptions > selector.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1iM;' --method A03 --signatures --exceptions > ready-named-callback.txt
```

For the negative comparison, exportBulk A0L with the same flags and cmp the
linked prior file. PCs are bytes; deltas code units. Retained complete selector
is interpreted only for identified requirement/handoff/diagnostic boundaries.
No dynamic/alias/global completeness or account-restriction guarantee.
