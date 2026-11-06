# Receipt dispatch readiness and failure boundaries — APK 2.26.43.72

Confirmed selected static, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Selected DEX exports/catches and direct physical censuses; no account execution.
[Complete REA A05 Evidence](../../rea/2.26.43.72-receipt-dispatch-failure/dispatch.json)
cross-checks the initial branch. Other exports are repository disassembler
output, not fabricated REA envelopes. Static observations do not prove delivery.

## Dispatch can return without callback completion

Prior [played registration](../2.26.43.72-receipt-registration-completion/README.md)
registers through A0B before calling0Sf.A05 with both booleans false. In A05,
channel A0J absent plus readiness A06=false and A02=12 logs a drop and returns
at0020..002c. Other absent-channel readiness combinations throw at009a..00aa.
No callback completion/removal is present in that initial return branch. State12
is a local field value here, not an inferred server refusal or ban cause.

With a channel, played selector38 enters the A00-provider side effect at003e.
The supplied false final boolean selects0Sf.A00 preparation at0070..0076.
Retained [1i8.A07](../2.26.43.72-status-final-writer/preack-selection.txt)
returns null for selector38 (no matching constants/switch branch). Therefore
this selected preparation returns null at0008..0010; A05 uses Message.obtain
of the original message, then invokes CdY only if its reread A0J is nonnull.
If that reread is null, it returns without CdY or explicit callback completion.
This is static branch structure, not proof of an observed concurrency race.
The initial ready path, later channel reread, future completion, socket write
and server ACK are distinct boundaries.

## Failure callbacks remain separate from non-delivery

The FfK.A00 Exception census finds exactly one direct writer, BsR, and the two
get overload reads. [Retained full future](../2.26.43.72-bulk-key-job-completion/future.txt)
clears the result/success flag, saves the exception and releases the latch;
get then throws ExecutionException. The interface BsR census has19 direct
invocations across11 distinct methods (three in0m2.handleMessage). This is
not full dynamic/reflection/native interface-dispatch coverage.

Existing [size-failure path](../2.26.43.72-status-final-writer/README.md)
connects0uw.C4g ->0ln.A0B: exact full-key removal, then BsR outside the lock.
E5X/E5W lie at different encryption stages; do not call all size failures
pre-write. Existing [disconnect cleanup](../2.26.43.72-message-disconnect-callbacks/README.md)
separates full-keyA04 cleanup from directA05 cleanup. Fresh0m2 re-export matches
the retained handler byte-for-byte; these already known methods are linked,
not duplicated as new discoveries.

0Sf.A0T instead normalizes the key withA02, removesA04[key] and calls BsR
outside its lock. Two direct callers are8iv.invokeSuspend and status jobA0P;
this census does not establish an ordinary played-job failure producer. No
normalization/owner-comparison policy is inferred for every failure path.

## Rust comparison and open work

No confirmed construction/order mismatch found. Rust commits durable reservation
before writer handoff, rechecks original ownership, ends the epoch on write
error and preserves uncertainty. APK drop/exception/future completion does not
justify resetting that reservation, dropping its durable callback or retrying.
No runtime behavior change, new deadline, send or retry added. Full A05-provider
side effects, all failure producers, native scheduling/retry policy, shutdown
ordering and account-generation authority remain open. No zero-ban claim.

## Reproduction

From the repository root, with the pinned APK and current built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/receipt-dispatch-failure-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A05 --signatures --exceptions > "$OUT/dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A00 --signatures --exceptions > "$OUT/prepare.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/1ue;' --method BsR > "$OUT/failure-callers.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A0T --signatures --exceptions > "$OUT/normalized-failure.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/0Sf;' --method A0T > "$OUT/normalized-failure-callers.txt"
target/debug/xtask apk-field-refs "$APK" --class 'LX/FfK;' --field A00 --type 'Ljava/lang/Exception;' > "$OUT/future-error-refs.txt"
```

checksums.json binds every other bundle file (bytes/SHA256). Linked historical
exports retain their own bundles/checksums. REA output retains its full subject,
provider, source coverage and limitations separately.
