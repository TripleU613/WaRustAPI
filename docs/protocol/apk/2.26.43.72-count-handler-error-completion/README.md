# Escaped handler errors: NACK request and queue completion

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Eleven complete method exports use xtask apk-smali APK --class CLASS --method
METHOD --signatures --exceptions:1ic.A0E/A0F,2Dx.<init>,1rg.A00/A02/A03/A04/
<clinit>,2Aa.A00,1id.A01,2A3.<clinit>. DEX text, not REA Evidence IDs.
Only the selected error/completion branches below are interpreted; this is not
full crash recovery, persistence ambiguity or count-specific execution parity.

## Error request is distinct from queue completion

The retained [2Ek.A01 router](../2.26.43.72-count-queue-processing/single-stanza-router.txt)
rethrows SQLite full/corruption exceptions from handler dispatch. Its selected
LIL, Exception and StackOverflowError paths call [1ic.A0E](unhandled-error-entry.txt).
With a stanza node, A0E calls1rg.A04 for LIL or1rg.A00 otherwise. Independently,
it creates2Dx with error487 for LIL or500 otherwise, then invokes1ic.A0F.
No-node input skips the1rg call but still constructs completion metadata.

[1rg.A04](parser-error-handler.txt) recognizes its ackable-stanza set, reports
parser diagnostics and callsA03(node,487). Unknown class can rethrow LIL; caught
reporting exceptions are logged. [A00](generic-error-handler.txt) callsA03
with500, with separate message-specific diagnostics, and logs selected errors.
[The set](ackable-stanza-set.txt) is receipt,notification,message,call,status.
This establishes notification eligibility, not successful socket delivery.

[A03](nack-request.txt) emits [metrics throughA02](nack-metrics.txt), constructs
fresh Efa with stanza class, required id, error code, marker-2, optional parsed
from/type/participant, freezes Eoh and calls0Sf.A0S. Nonackable input logs instead.
Full A0S writer/queue readiness and malformed metadata failures remain outside
this trace. These are NACK requests, not proven server acceptance or retry advice.
No ID, source or required count is fabricated by this selected builder.

[2Dx constructor](completion-descriptor.txt) maps the supplied error toA05,
null auxiliary error toA04, and the two false Booleans toA07/A06. The error
response above and completion below must not be mistaken for two normal ACKs.

## Completion state and ACK-list eligibility

[Queue enum](queue-state-enum.txt) gives the names/ordinals:
A05=QUEUED(0),A06=READY_TO_PROCESS(1),A04=PROCESSING(2),
A03=PROCESSED(3),A02=FAILED_TO_PROCESS(4).
[2Aa.A00](completed-state-predicate.txt) tests ordinal>PROCESSING: it is a
completed-state predicate, not readiness or database-commit proof. This also
names the A05->A06 memory transition in the previous selection audit.

[1ic.A0F](failure-completion-entry.txt) rejects already completed input, selects
its queue, invokes1id.A01 and uses configA0T to retain/drop that returned
pre-ACK list. It submits the shared1ic.A0A transaction path directly or via
8bT(17), with Feh(46) post-completion work. Those callbacks' error-path effects
remain untraced here; neither admission nor a completed memory state proves
successful retirement from disk.

[1id.A01](completion-selection.txt) waits for load, calls1iq.A0F to select
eligible completion descriptors, then marks the selected stanza PROCESSED for
nullA05 or FAILED_TO_PROCESS for nonnullA05. Its subsequent ElB generation
requires descriptorA07=true (plus other eligibility conditions). A0E supplied
false, so this selected escaped-error completion does not add another ElB ACK
from that list-generation branch. Full1iq.A0F membership, BUp/2DB policy and
transactional retirement remain open. A failed memory state is not replay or
key-regeneration authority.

## Count parser is an inner boundary

This outer escaped-error route does not replace the
[count-content catch](../2.26.43.72-count-content-error-ack/README.md):15v.A09 PQ
parse LIL occurs before deferral, and0v6.A01 catches it internally, dispatching
its original normal ACK context. Do not send487 for that path simply because
it is a parser exception. Likewise do not swallow SQLite storage failures into
500. Rust's recently corrected count-content ACK remains unchanged.

No Rust error NACK, automatic retry, queue replay or live account operation was
added. A future port must separately model escaped-error response, completion
selection, memory state, transaction outcome and confirmation. Concrete count
bulk ownership, cleanup callbacks, direct NACK queue behavior and crash recovery
remain open.
