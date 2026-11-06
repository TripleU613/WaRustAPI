# Jobqueue connection observer, wake and persistence admission — APK 2.26.43.72

**Confirmed selected static**, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence/runtime observations.
Research only; no live accounts or Rust behavior change.

## Concrete observer registration and manager association

RNL.<init> obtains0AI via00D.A02(0x101), stores it and registers itself through
07o.A0K(this). This is the same provider KEY used by the prior
[ChatConnectionRequirement](../2.26.43.72-chat-job-requirement/requirement.txt),
not proof of global instance identity across every provider/account lifecycle.
RNL.Cgm(manager) saves the suppliedNhR. ItsBzL andBzN each callmanager.A05 if
nonnull; BzM/BzO/BzP return directly. Existing readiness research maps selector2
toBzL; the complete BzN event producer is not newly reconstructed here.

NhR's constructor iterates its suppliedRF1 list and callsCgm(this), starts the
saved-job loader throughA03, and constructsNhN/worker structures. ExactRNL
constructor census points toRNH.A00; complete DI list assembly, RNH case and
manager provider identity remain untraced. This therefore establishes the
concrete conditional association/registration path, not universal installation
or delivery of every state change to this particular queue.

## Wake means open the selector condition, not run a send

NhR.A05 constructsQnY(this,33), then callsA03. Its constructor stores that
exact discriminator and target; QnY.run case33 at06de..06fe synchronizes on
manager.A05:NhN and opensNhM.A02'sConditionVariable. It does not add a job,
create an ID, call a job body or write a packet. The
[previous selector](../2.26.43.72-chat-job-requirement/selector.txt) wakes,
closes its condition and rechecks job requirements/group gates before handoff.

A05 is NOT guaranteed to deliver a wake: A03 returnsfalse whenmanager.A00 is
true or executor submission throwsRejectedExecutionException; A05 discards
that result. Fullflag meaning, teardown ordering and observation/reporting
policy remain untraced. Local wake submission/return is not admission or
server acceptance. No reliability claim is inferred from the method name.

A condition-field census has seven physical sites: constructor, selectorrun,
NhN.A02/A03, NhR.A04, NhS.run andQnY.run. Exactwake-call census has15 physical
sites, includingRNL callbacks, status callbacks and other producers; these
are not fifteen distinct policies or complete runtime/alias coverage.

## Other wake sources preserve distinct authorities

NhN.A02 removes a group token from its set under lock, then opens the same
condition. NhN.A03 puts the existing Job at the FRONT of its list, updates
per-group accounting and opens the condition. ExactA03 caller census has
NhO.rejectedExecution andQlq.run; the latter's prior retained exception/defer
policy must be consulted before interpreting requeue. A condition wake alone
is not evidence that either caller or another wire attempt occurred.

NhR.A04's selectedpersistent branch serializes the Job, inserts item/encrypted
columns intoqueue and assigns the returned rowID before proceeding to runtime
injection. It invokesNhf.A00, thenJob.A0J, adds the Job to the in-memory list,
updates group accounting and opens the condition. This ordering does not prove
one atomic transaction spanning SQLite/list/writer, nor every failure cleanup,
dedup, crash recovery or operation-specific admission. Selected read-lock use
is not relabeled a SQLite transaction. No native successful send is shown here.

Nhf.A00 invokesCf0 on a Job implementingRF0, then on each requirement
implementingRF0. This supplies the concrete injection loop for the prior
requirement'sCf0; complete interface/provider identity still matters. NhS.run
loads retainedqueue rows, injects loaded jobs, and its identified final block
adds them and updates group counts, then opens the selector condition. Loader decryption,
serializer compatibility, all restoration/failure branches and account binding
remain untraced. Full methods are retained; only identified admission/wake
blocks are interpreted.

## Rust comparison and remaining scope

Existing durable-receipt callback ownership and partial encrypted-write tests
pass unchanged. Native rechecking/wake mechanisms do not authorize erasing
original uncertainty, losing callback owners, rotating IDs or replaying a Rust
reservation. No confirmed Rust mismatch was changed here, no scheduler clone,
retry timer or live operation added. These are offline Rust controls, not
native execution or full lifecycle equivalence.

Next gaps: completeRF1 list/provider binding and observer dispatch lifecycle,
persistentqueue failure/dedup/restore semantics, rejection/defer handoff and
operation-specific restart/account-generation authority. The broader network
audit remains incomplete; no ban or delivery guarantee is made.

## Reproduce

From checkout root with pinned APK outside Git; compare complete outputs:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/NhN;' --signatures --exceptions > owner.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/NhR;' --signatures --exceptions > manager.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/QnY;' --method 'run' --signatures --exceptions > wake-worker.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/QnY;' --method '<init>' --signatures --exceptions > wake-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/RNL;' --signatures --exceptions > observer.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Nhf;' --signatures --exceptions > injector.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/NhS;' --method 'run' --signatures --exceptions > loader.txt
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/NhM;' --field A02 --type 'Landroid/os/ConditionVariable;' > wake-refs.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/NhN;' --method A03 > requeue-callers.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/NhR;' --method A05 > wake-callers.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/RNL;' --method '<init>' > observer-callers.txt
```

PCs are bytes; branch deltas code units. Large sharedQnY retains the fullrun;
only case33 is interpreted. Direct-reference censuses exclude reflection and
alias/global runtime completeness. No implicit server or retry authority.
