# Retry connection requirement: state writers and invalidation

**Confirmed selected static DEX**, WhatsApp2.26.43.72/code264307238,
APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `2214a852`. Execution-free repository DEX inspector; no target execution,
account operation or Rust runtime change. Existing REA Android Java-path failure
is not counted as successful tool evidence.

## Reuse existing positive readiness evidence

The [chat job requirement bundle](../2.26.43.72-chat-job-requirement/README.md)
already binds ChatConnectionRequirement.BWq to0AI.A0O/A04==2, and retains the
five-site writer census. The [connected-handler trace](../2.26.43.72-client-auth-token-readiness/README.md)
already binds both0YB.A01 calls to0AI.A0M. These positive readiness conclusions
are reused, not new discoveries. Fresh [writer census](state-writers.txt),
[ready body](boolean-transition.txt) and [caller census](state-two-callers.txt)
corroborate those selectors; they do not establish a new release difference.
The new selected binding here classifies other writer values and invalidation.

## Exact writer values

| Direct write site | Value and selected effects | Requirement consequence |
| --- | --- | --- |
| [0AI constructor](initial-state.txt),0044..0058 | A04=3; creates a closed ConditionVariable | False after this initialization. |
| [0AI.A0M](boolean-transition.txt),0000..0034 | A04=2 under monitor; A07=true, A05=argument; opens condition; notifies2Gb selector2 | True at the predicate's read of this state. This can precede later connected-handler work. |
| [retained0YU.handleMessage](../2.26.43.72-iq-shutdown-eligibility/message-handler.txt),00f8..012e ->1306..132c | Uses reason `connecting`, obtains1iM.A0j's0AI; under its monitor sets v0=1 at0128 and branches to1306 to write A04; increments A03 and notifies2Gb selector1 | State1 does not satisfy the requirement. The distant shared tail must be included; its v0 is not an unknown default. |
| [1iM.A02](disconnect-state.txt),0018..0052 | A04=3 and A05=false under monitor, closes condition, notifiesFIt selector0; later logs server disconnected | Clears this local requirement after the write; no durable request outcome or server response is reset by this state assignment. |
| [1iM.A04](state-four.txt),0010..002e | A04=4 under monitor; notifiesFIt selector1 | State4 does not satisfy the requirement; complete caller event semantics are not classified here. |

The census accounts for all five exact DEX writes to this field. Reflection,
native writes, aliases and external synchronization are not covered. The
constructor and disconnect method are distinct sites even though both write3.
The method name A02 is not a state number; likewise0AI.A02 is a separate field.

## What this adds to the retry-job chain

The [retry exception/runner trace](../2.26.43.72-retry-receipt-job-failure-policy/README.md)
showed that a checked callback failure can reach the true exception handler and
then requirement recheck. That recheck can observe false for connecting1,
disconnected3 or state4; it is not permanently true after the first connection.
The selected worker can then defer instead of immediately entering another run.
This is a control-flow composition, not observed scheduling or a wire retry count.
It does not prove ordering between disconnect callback failure and the state3
write across threads/provider instances. The predicate body itself does not
acquire the transition monitor; no global atomic admission proof is inferred.

Remaining work includes requirement observer/wake registration, exact provider
identity, state4 caller classification, scheduling/deferred persistence/dedup,
and final writer/account authority. These local states do not prove complete
bootstrap or server acceptance. Rust's late writer fences and immutable durable
reservations remain unchanged; no resend, new timing rule or synthetic state is
introduced from this static trace.

## Reproduction

```
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/0AI;' --field A04 --type I --writes-only
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1iM;' --method A02 --signatures --exceptions
```

Use the exported descriptors for remaining selectors. The connecting path uses
the complete previously retained handler rather than duplicating75 KiB. Its
branch at012a targets1306 because0x012a+2*0x08ee=0x1306; printed PCs are bytes,
branch deltas and exception-table comments use code units. No runtime probe is
presented as execution. checksums.json pins this compact bundle.
