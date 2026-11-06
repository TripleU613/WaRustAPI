# Failed count completion: disk deletion versus cache retirement

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete xtask apk-smali exports with --signatures --exceptions:Feh.run,
both Feh constructors,1id.A0J,1ih.A04,1iq.A0F. DEX text, not REA Evidence IDs.
Only the selected paths below were audited; unrelated Feh selectors and
metrics arithmetic are not claimed understood.

## Callback identity and normal transaction boundary

The prior [1ic.A0F](../2.26.43.72-count-handler-error-completion/failure-completion-entry.txt)
constructs Feh(descriptorList,stanza,manager,46). The generic Object
[constructor](callback-constructors.txt) assigns manager toA00, stanza toA01
and descriptorList toA02. [Feh.run selector46](completion-callback.txt) enters
0250, selects the stanza's queue with1ic.A01 and invokes1id.A0J(list).
It does not resubmit the stanza to the handler or regenerate key material.

On the selected direct branch, A0F passes this Runnable as the success argument
to the existing [1ic.A0A](../2.26.43.72-prekey-upload-replies/prekey-ack-txn-bulk-transaction.txt).
That method opens0aq/1hk when it has transaction work and registers the success
Runnable using0aq.A04. The earlier
[transaction completion bridge](../2.26.43.72-prekey-upload-replies/README.md#ack-transaction-completion-and-rollback-bridge-2026-11-04)
proves normal successful outer-completion release, distinct from rollback.
A0A's no-work branch instead calls the Runnable directly without opening a
transaction. Therefore callback execution alone is not universal disk-commit
proof. A0F's alternative8bT executor branch is not re-traced here.

## Persisted stanza rows are retired by row ID

Within A0A's selected transaction scope,016c..01e6 collects nonnull stanzaA0B
row IDs and calls backend1ih.A04 when the list/backend are present. Other
replacement stanzas can be persisted by1id.A0E, and supplied pre-ACKs can be
inserted by1iV.A0I in that scope. Not every descriptor has a row ID or pre-ACK.

[1ih.A04](disk-retirement.txt) obtains its0Xp/0aq/1hk wrapper, converts supplied
numeric IDs to strings, chunks them with size0x3cf (975), and calls0KU.A04
against the backend's1ik.A05 table with an _id IN (...) clause. It marks its
transaction successful and closes both wrappers; selected exception cleanup
rethrows. SQL placeholder production by3fP(25), concrete backend/table selection,
shared provider identity with the caller's outer transaction and ambiguous disk
outcomes remain unproven. Do not claim cross-database atomicity from nesting.

This is exact supplied-row retirement, not cumulative deletion through a
server-confirmed pre-ACK watermark. These two deletion authorities differ.

## Cache retirement is a separate operation

[1id.A0J](memory-retirement.txt) maps completion descriptors to stanzas, derives
their2DG keys, removes those keys from the metadata map and auxiliary key set,
and invokes FgR(selector8) to remove matching list entries/update counts under
the cache monitor. Its later observer/metrics and CHt(size) paths do not turn
this into an SQL deletion call. Detailed FgR predicate/count semantics and CHt
consumer scheduling are still open, so no broader deduplication proof follows.

The [1iq.A0F membership filter](completion-membership.txt), used earlier in
completion selection and optionally here for stats, keeps a2Dx only if its
stanza key exists in the queue metadata map. Missing metadata is logged.
It does not compare database row IDs, account generations or exact stanza
bytes. This closes the prior filter-membership unknown without inventing
cross-generation ownership or exact-content equivalence.

## Port consequences and limits

Failed memory state, membership, SQL row retirement, cache removal, pre-ACK
persistence and server confirmation are separate facts. A future Rust port
must retain each boundary and propagate failed/uncertain writes; this source
does not authorize replaying failures, deleting captures, resetting keys or
calling cache removal a server acknowledgement. Count-specific bulk ownership,
rollback/crash recovery, exact backend transaction identity, executor branch
and cache-removal helper semantics remain open. No Rust network/recovery
behavior or live account operation changed.
