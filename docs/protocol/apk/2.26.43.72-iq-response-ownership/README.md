# IQ response ownership and token schema failure — APK 2.26.43.72

Confirmed static checked DEX and class_def metadata, not runtime observations
or REA envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `apk-field-refs APK --class LX/LIL; --field isRecoverable
--type Z --writes-only`; `python3 inspect-interfaces.py APK`.
The metadata script checks the APK digest. Registry exports select complete
methods from the checked0ln class export; all files are checksummed.

## Timer removal, callback claim and parsing are different stages

0ln.A0H processes an incoming IQ ID. UnderA0F's monitor it removes the ID
from the timer-tracking map, removes its message4 timer when tracked, and
GETS theA0H callback record. It does not remove that callback at this stage.
It logs/returns false when no sent-map callback record exists. This lookup
usesA0H, not the queuedA0I map; the selected method is not proof of all
upstream IQ root, type, address or ID admission checks.

FUT implements only0lG, whose interface declaration has no parents; it has
neither1xZ nor1xa markers. Consequently the selected token callback takes
A0H's ordinary path: post handler message2 with the IQ and iqId. The retained
[handler](../2.26.43.72-iq-promotion-shutdown/handler.txt) case2 calls0ln.A02
with false. A02 removes the callback record fromA0H underA0F's monitor,
releases it, then submits1xb to its098 executor. The runnable unwraps the
retained callback from1bW and calls0ln.A01. A01 finally dispatches exact
result toCK4 or error toBvj; other types take diagnostic handling.

Thus receipt lookup, timer cancellation, callback ownership removal and
callback parsing are not one atomic operation. Removing the timer is not
successful schema validation, durable token persistence or completion of
held-request recovery. The selected methods do not compare current wall time
against the stored elapsedRealtime timestamp to authorize parsing; A02 uses
that timestamp for a duration passed to0V4.A0C. This is not a claim about all
upstream timing controls, nor a reason to weaken Rust's explicit deadlines.

## Failure races can consume ownership before parsing

A timeout/failure or shutdown flush can remove a callback afterA0H's GET but
beforeA02's REMOVE. In that possible interleaving A02 logs missing callback
and does not enqueue its parser. IfA02 removes first, that callback is no
longer in the map for the selected failure-removal path to claim. Duplicate
queued response messages likewise do not each claim the same unchanged map
entry. These are static ownership consequences, not observed races or a
complete exactly-once guarantee across ID reuse, replacement or generation
changes. The selected [failure and shutdown removals](../2.26.43.72-iq-promotion-shutdown/README.md)
remain separate from socket delivery evidence.

Once1xb owns a removed record, it retains callback/node/ID independently of
the registry. This is not proof of restart persistence, executor completion
or successful validation. A result arriving before the deadline can wait
in handler/executor queues; native timer cancellation does not mean callback
execution already finished at that deadline.

## Schema exceptions do not restore delivery-failure state

The previously traced token CK4 throws8B6.A00'sLIL when the correlated
header conversion fails. That factory invokesLIL(String), whose exact
constructor sets isRecoverable=true. LIL extendsLV5. 0ln.A01 catchesLV5,
logs and calls the0B0.A0M reporting hook, then records an error metric forLIL.
LV5.A01 readsLIL.isRecoverable; A01 returns when this is true, and otherwise
wraps/throws RuntimeException. The exact field-write census finds only five
LIL constructors, all retained. This is a bounded DEX write inventory, not
proof that reflection/native hooks cannot mutate state or throw. The reporting
hook is abstract in this class, so its concrete effects are not asserted.

No callback reinsertion, BsS invocation, pending-nonce restoration, key rotation
or held-request dispatch appears in the selected catch body. Here recoverable
is exception routing, not network retry authorization. A null token-value
conversion can instead return normally fromCK4 without saving a token; the
surrounding normal-return bookkeeping is not proof the token was installed.
The prior [reply grammar](../2.26.43.72-client-auth-token-reply/README.md) and
[delivery-failure restoration](../2.26.43.72-client-auth-token-failure/README.md)
therefore remain distinct outcomes.

## Implementation boundary

This closes the selected ordinary response-dispatch and schema-catch chain,
not all router admission, executor/reporting behavior, concurrent ID ownership,
generation binding or durable reconciliation. No Rust sender, retry, route or
live account operation is added. Existing uncertain holds remain unchanged.
