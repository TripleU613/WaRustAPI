# Count content-error ACK boundary

**Confirmed static**, APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
xtask apk-smali exports use --signatures --exceptions; apk-callers inventories
identify their exact target in the filename discussion below. These are DEX
text, not REA Evidence IDs. All exported bodies are complete, but this audit
is bounded to the paths described here, not unrelated reader/runner branches.

## Before deferral: parser failure

Retained [15v.A09](../2.26.43.72-prekey-upload-replies/prekey-ack-count-dispatch.txt)
selects first-child count/PQ. Legacy A0H is optional: [0Tp.A0H](optional-integer.txt)
catches NumberFormatException and returns null. PQ A05 defaults absent value
but calls [A06](strict-integer.txt) for present value; A06 converts
NumberFormatException into LIL. [A05](defaulted-integer.txt) does not catch it.
Both PQ parse calls occur before A07(true) (deferred metadata) or worker enqueue.
Thus malformed PQ never gets as far as scheduling its upload worker.

## Handler exception and completion path

[0v6.A01](base-handler.txt), normal entry, stores2DT(incoming,Eoh) in A0C,
then invokes virtual A09 at0204. Its try range00ae..0210 catches LIL at0210,
logs it, then reaches0298. It takes A0C with getAndSet(null). If the context
exists and its Eoh is not in deferred map A0A, it invokes0v6.A00 with the
original incoming and Eoh at02c2, then removes Eoh.A08 from A0B. No replacement
ID, source or guessed upload is created. Count PQ parse failure precedes
A07(true), so its context remains available on this selected path.

SQLiteFullException and SQLiteDatabaseCorruptException have a separate020c
rethrow handler; they do not use the LIL completion path. Other Exception
has another handler, and tracked incoming can request a500 error through2Dx.
This does not authorize treating storage errors as harmless content errors.
Metadata-construction failures before A01 are outside this catch boundary.

0v6.BLn builds F0R.A02 metadata and invokes8bl selector13; its
[runner](notification-runner.txt) enters0368 and reaches A01 directly or via
8aj when the handler serial executor is configured. Exact A01 caller inventory
is [8aj.run and8bl.run](base-handler-callers.txt). The complete runner export
contains unrelated selectors; only selector13 is used here.

## Incoming kind and routing census

APK-wide physical method-name export for BX2 finds [three declarations](incoming-kind-predicates.txt):
interface1k2,1k3 constanttrue,1uU constantfalse. This declaration census is not
runtime coverage. Exact1uU constructor inventory finds only1s2.run. The
[reader](incoming-reader.txt) selected104a..1072 looks up a1rA handler, wraps
its node in1uU and callsA03. This establishes an untracked ingress path,
not that every count enters it. Tracked subclass selection, effective queue
configuration and count-specific bulk membership remain open. No new evidence
from uninspected reader branches is claimed.

## Rust correction and limits

Rust low_count still returns InvalidInput for malformed present PQ values.
The actor now separates that content refusal from ACK-envelope validation:
retain durable capture, construct the normal count ACK, send it, skip all
upload work. Missing/empty IDs and explicit unparsable sources still refuse
ACK and upload. A recognized mixed legacy+malformed PQ cannot fall back to
legacy upload. Store failures before this path and ACK write failures propagate.

The expanded actor regression failed on old code waiting for the missing ACK.
It now checks overflowing PQ ACK class/type/ID, no key/session/identity change,
no upload, mixed malformed refusal, invalid-envelope controls and a subsequent
valid count. This supersedes earlier documented local no-ACK policy for
malformed PQ content only. Native deferred queue/crash replay parity and valid
count upload-before-ACK-admission remain unported. No live account operation.
