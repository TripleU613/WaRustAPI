# Strict Signal session-store failure policy

Pinned WhatsApp APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
**Confirmed selected static**, not runtime observation. Existing xtask DEX
fallback (Android REA provider unavailable); no new REA Evidence ID. Complete
selected methods retain signatures/exceptions; caller census is bounded exact
class/method matching. Instruction PCs are bytes, exception ranges code units.

## The store-failure flag has a migration-bootstrap owner

The prior [exact A0T field census](../2.26.43.72-retry-session-store-transactions/failure-policy-refs.txt)
found a static initialization and two field readers, FcP.run and Ff8.call.
[Ff8.call](strict-callable.txt) discriminator2 is the setter through that retained
ThreadLocal reference. It obtains SignalCoordinator via Eqn.A03, extracts Rn3
ciphertext/message type and SYX.A01 address, then checks0YM.A03. The
[thread predicate](executor-thread-check.txt) compares currentThread by identity
to the referent of0YM.A03 weak reference; absent/mismatched thread returns false.
The caller then throws `Strict Signal decryption must run on SignalExecutor`.

On the admitted path it saves A0T.get(), sets Boolean.TRUE (the
[helper](true-constant.txt) returns true), invokes A0O or A0N, and restores the
prior value. A null previous value uses remove(), not set(false). Both success
and catch-all exception paths restore it. This is scoped execution context,
not a persistent account capability or a flag to leave on globally.

[Constructor](callable-constructor.txt) maps supplied Rn3 to A02 and SYX to A01.
The [complete exact constructor census](callable-callers.txt) has nine records;
only the selected [TDp.invoke](migration-dispatch.txt) producer is traced here.
It passes discriminator2 at0086..0092, submits through0YM.A01 with label
`companion`, and awaits FutureTask for0x7530 milliseconds (30,000ms). Its strings
and payload validations identify Signal migration bootstrap, not ordinary chat
retry-receipt processing. The dispatch accepts EtY only with status0 and nonempty
bytes before decoding RnK and checking migration/version/bindings. Selected
ExecutionException handling recognizes Fj7 and throws `Signal migration ratchet
was not persisted`. Timeout/interruption cancel(false); timeout is not proof
that a running task stopped or any session writes rolled back. No new timeout
or cancellation policy is implemented from this evidence.

## Persistence exceptions remain distinct from decryption results

Prior [FcP session callback](../2.26.43.72-retry-session-store-transactions/session-runnable-excerpts.txt)
wraps selected IOException as Dvm only when A0T.get()==Boolean.TRUE. Otherwise
that handler logs and returns. [Dvm](persistence-exception.txt) extends Fj7.
Both [0WJ.A0N](decrypt-normal.txt) and [A0O](decrypt-prekey.txt) explicitly rethrow
Fj7, SQLiteFullException and SQLiteDatabaseCorruptException from their selected
generic Exception handler instead of converting them to EtY/-1000. Other selected
exceptions have distinct typed status mappings. This confirms that infrastructure
failure must not be flattened into an ordinary decrypt failure/result; it does
not prove all possible database faults have one treatment.

## Retry boundary and Rust comparison

The confirmed flag-setting scope is migration-bootstrap decryption. The retained
163.A02 retry-carried-key caller directly invokes0WJ.A0K; no link from that call
to this strict wrapper is established. A0K has its own exception-to-status
mapping. The exact static field census does not enumerate reflective access,
ThreadLocal aliases or every dynamic execution context. Do not claim every
retry forces strict writes, or that a native retry status proves persistence.

Confirmed Rust source comparison: transport/inbox.rs process_with_policy
propagates errors outside the explicit direct_message/app_state_key_share and
typed Signal content/decryption refusals. Its comment explicitly distinguishes
post-decryption transaction failure from undecryptable content. No new runtime
classification defect was demonstrated here; receipt_keys_unsupported remains.
This research neither weakens Rust transaction failure propagation nor adds a
migration feature, attestation/key adoption, live request or account operation.

Next proof remains concrete receipt-key cleanup/ownership, full alias/database
transactions and ExG/DDs state synchronization. Reproduce via apk-smali with
signatures/exceptions for Ff8.call/<init>, TDp.invoke,0YM.A03,2L8.A1B,0WJ.A0N/A0O
and Dvm.<init>; apk-callers for exact Ff8.<init>. No broader caller absence or
whole-APK behavioral parity is claimed. Manifest covers files except itself.
