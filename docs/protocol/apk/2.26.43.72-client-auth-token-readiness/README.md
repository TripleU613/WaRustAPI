# Client-auth-token readiness producer — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-callers APK --class CLASS --method METHOD`.
All retained files are covered by checksums.json. Use full signature comments,
not the exporter's abbreviated method labels.

## State 2 is produced by the connected handler

0AI.A0M(Z)V synchronizes, writes A04=2 and A07=true, copies its boolean
argument to A05, opens A00's ConditionVariable, then invokes 07o.A00 with
2Gb selector2 before releasing the monitor. This closes the state2-producer
gap from the [trigger trace](../2.26.43.72-client-auth-token-trigger/README.md).
It does not prove that all account prerequisites can be reduced to this state.

Exact-class caller search finds two physical calls, both in 0YB.A01, whose
entry checks A1H and returns with ignored_stopped when false. At 023a, the
result of 0GS.A02 selects a branch to 07fa (nonzero) or the longer path.
Both calls pass v2 copied from boolean parameter v20. The longer path invokes
A0M at 0288; a login-back predicate earlier selects a diagnostic and optional
callback before this same call. A second call occurs at 080a on the shorter
branch. Therefore state2 is not proof that every operation in the long
connected-handler path has completed. That path logs server connected at0412.
The full selected method is retained to preserve intervening branches.
0GS.A02 semantics and the boolean parameter's upstream meaning remain open.

## Observer callback and pending-slot semantics

2Gb.CbC selector2 invokes 0AK.BzL. 1jA.BzL calls A00, which atomically takes
and clears the pending String via getAndSet(null), then calls A01 if nonnull.
A01 rechecks readiness; when false it sets the pending reference and returns.
Thus this is one replaceable in-memory slot, not a FIFO or durable retry ledger.
Repeated deferred set operations can replace an older input. Rechecking can
restore an input after readiness changes; it does not prove atomic lifecycle
ordering with concurrent push deliveries. No safe retry count is inferred.

The constructor initializes a new empty AtomicReference; it does not restore
pending input from disk in this inspected body. Other manager lifecycle hooks
BzM/BzN/BzO/BzP return directly. Exact A00 caller search also finds 0ZJ.A0z,
which remains untraced here. Observer registration, 07o scheduling/dispatch,
provider instance identity, cross-thread ordering and restart/account-generation
binding remain open. The notifier-to-interface and implementation callback
are confirmed, but actual registration/delivery is not established here.

## Runtime boundary

No Rust sender, automatic replay, route or live operation is added. Existing
uncertain holds remain. Trusted push provenance, the consumer's v31 control-flow
join, complete lifecycle eligibility and failure/backoff still block faithful
automatic token recovery. This evidence does not establish zero-ban behavior.
