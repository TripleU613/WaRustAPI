# Retry receipt job exception eligibility and requirements

**Confirmed selected static DEX**, pinned WhatsApp2.26.43.72/code264307238,
APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `c98559e5`. Repository execution-free DEX inspector; no successful REA
Android result is claimed while its recorded Java-path failure remains. No
runtime change, account operation, timer, key generation or packet transmission.

## Exception handler and shared worker

[SendRetryReceiptJob.A0N](exception-policy.txt) checks its Exception is nonnull,
logs through DBh.A1I, and returns true on normal continuation. It does not inspect
exception subtype, ACK status, retryCount or cause in this selected method.
This is eligibility for exceptions reaching A0N, not every possible failure.

The [already traced shared runner](../2.26.43.72-bulk-key-job-completion/job-runner.txt)
invokes Job.A0L while Job.A00 is below parameters.retryCount. Normal A0L return
produces SUCCESS. On caught Exception it increments Job.A00; RuntimeException
bypasses A0N and is rethrown. For other exceptions it calls the virtual A0N,
then inherited [Job.A0M](../2.26.43.72-bulk-key-job-completion/requirements-check.txt).
If requirements hold, the selected loop returns to the budget check/A0L; if not,
it takes the DEFERRED path. False A0N reaches FAILURE, though the selected retry
job override returns true. Named results are bound by the retained
[JobResult initializer](../2.26.43.72-bulk-key-job-completion/job-results.txt).
Full outer queue persistence, defer scheduling and crash recovery remain separate.

The [previous callback consumer trace](../2.26.43.72-retry-receipt-callback-consumers/README.md)
now composes with this worker boundary: disconnect BsR fails FfK, whose get wraps
it in ExecutionException; that checked exception is eligible through A0N. A
matched error-bearing ACK instead reaches BsT/null and a normal job return in
that selected callback path. Job SUCCESS and server acceptance are distinct.
No new automatic retry of a server refusal is inferred.

## Parameters: persistent grouped job and default budget

The [retained constructor](../2.26.43.72-outgoing-retry-timestamp/job-constructor.txt)
creates DCZ.A00, builds a group string from `retry-receipt-` plus the converted
input chat, and calls6ol.A0n before the base Job constructor. It does not write
the builder's attempt budget between creation and build.
[Fresh A00 export](builder-default.txt) creates a fresh DCZ. The
[existing DCZ constructor/A03](../2.26.43.72-bulk-key-job-completion/parameter-builder.txt)
sets A00=100 and binds it to JobParameters.retryCount. The newly retained
[6ol.A0n](parameter-helper.txt) stores the group string, sets A03=true, appends
ChatConnectionRequirement and builds JobParameters. The
[existing fields constructor](../2.26.43.72-bulk-key-job-completion/parameter-fields.txt)
binds persistence=true. The fresh helper matches that retained helper body;
this is corroborated shared plumbing, not a newly discovered global default.
No OfflineProcessingCompletedRequirement is added by this selected helper.

**Do not conflate counters:** parameters.retryCount100 bounds job invocations;
Job.A00 counts caught attempts. SendRetryReceiptJob.retryCount is separately
copied from incoming encrypted-message count and the selected run encodes that
saved value+1. A repeated job invocation does not itself establish the next
protocol count, identical wire/key bytes, or100 actual network transmissions.
Carried-key preparation, native send lifecycle and effective queue policy remain
needed before interpreting repeat attempts as a client implementation recipe.

## Concrete requirement predicate

[ChatConnectionRequirement.BWq](connection-requirement.txt) requires its injected
0AI object and returns0AI.A0O. [0AI.A0O](connection-predicate.txt) reads A04 and
returns true exactly for integer state2. This closes the selected predicate body.
The state writers, session/provider lifetime, exact human meaning of state2 and
concurrent transitions are not classified here. It is not an arbitrary socket-open
check or proof of authentication/server acceptance. No Rust state number is changed.

## Reproduction and next boundary

```
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/jobqueue/job/SendRetryReceiptJob;' --method A0N --signatures --exceptions
```

Use the exact descriptors in the other text exports. Full selected constructor,
run and shared runner evidence is linked rather than duplicated. Printed
coordinates are inspector byte offsets; exception comments use code units.
checksums.json pins all local artifacts.

Next unresolved work is requirement state2 ownership/transitions, concrete queue
requeue/backoff/dedup and durable carried-key selection/upload exclusion, not
another inference from future completion. Rust's immutable reservations and
no-replay-after-uncertain-write policy remain intact. No retry limit or delay is
ported from this incomplete native lifecycle.
