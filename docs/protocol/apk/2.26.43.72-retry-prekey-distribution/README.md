# Retry prekey preparation owns direct distribution — APK2.26.43.72

**Confirmed selected static / research only**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask fallback methods; PCs are bytes, branch displacements
16-bit units. No target execution, account traffic, new REA Evidence ID or Rust
runtime change. Continues the [stateless/PQ input trace](../2.26.43.72-enc-state-pq-retry-inputs/README.md).

## PQ policy depends on durable local inventory

Fresh Ex8.A01 was method-body identical to the previously retained complete
[pq-policy.txt](../2.26.43.72-sender-key-session-requirement/pq-policy.txt), so it
is reused rather than duplicated. Selected results are:

| Condition, in evaluation order | Returned02X object |
| --- | --- |
| feature0x52ce false | A0C |
| otherwise feature0x5e60 false | A0N |
| otherwise a last-resort Kyber row is sent_to_server=1 | A00 |
| otherwise any last-resort Kyber row exists | A01 |
| otherwise | A0Y |

[0WJ.A0z](uploaded-last-resort-predicate.txt) supplies the uploaded-last-resort
SQL EXISTS check. Ex8 performs its separate last-resort existence query under
Signal lock; [2L9.A1P](exists-result-predicate.txt), called with fallback false,
returns true exactly for query result1. Enum numeric values/display names and
actual feature values are not invented. The selected
[Ff4 discriminator10](../2.26.43.72-enc-state-pq-retry-inputs/retry-key-callable.txt)
loads optional0WJ.A0a only when its isPqSession input is true and Ex8 returns
A00 or A01. This does not authorize arbitrary PQ advertisement.

## Optional getter changes publication ownership before returning

[0WJ.A0a](kyber-direct-key.txt) acquires a Signal lock, calls0Yl.A06 (initialization
side effects not fully classified here), then calls0Yd.A02 with limit1.
[0Yd.A02](kyber-unsent-selector.txt) uses:

`sent_to_server = 0 AND direct_distribution = 0 AND last_resort_key = 0 LIMIT 1`

No ORDER BY is present, so lowest-id ordering is not established. The helper
parses the selected stored record through EqE.A01, builds a public envelope,
then updates that ordinary row (prekey_id plus last_resort_key=0):

- direct_distribution=true;
- upload_timestamp=local clock milliseconds divided by1000.

Only an update count exactly1 allows the envelope to survive for return.
Otherwise it logs not sending kyber pre-key with retry receipt and returns null.
Selected record-decode exceptions are logged and call0Yd.Bei(id), then return
null; deletion scope and recovery are not newly classified. This is source-only
analysis: the mutating helper was not executed.

The timestamp is publication bookkeeping; it is not proof of socket delivery or
server acceptance. Marking direct distribution happens during key preparation,
before retry envelope submission. Uncertain writes must not make such a key
eligible for another server-upload/direct-distribution path. This trace does not
establish a single SQL transaction spanning selection, update and socket write;
Signal locking and helper-local database contexts are visible, crash durability
and row-update authority require separate treatment.

## Envelope preserves stored public key/signature

[0Yj.A00](kyber-public-envelope.txt) takes the stored Dfd public/private bytes,
constructs KEMKeyPair, obtains its public A00 bytes, retains the stored signature,
and encodes the selected id via1nr.A04. It constructs ETs with those three inputs.
[ETs constructor](key-fields-constructor.txt) assigns argument1→A01(id),
argument2→A00(value),argument3→A02(signature). It does not regenerate a signature
here. Exact KEM serialization/type-prefix semantics and signing input need their
own provider binding; do not assume that raw stored public bytes can be copied
without following KEMKeyPair conversion.

[0WJ.A0c](signed-prekey-getter.txt) is the signed prekey getter (loads A0Z and
uses A05). [0WJ.A0d](ordinary-direct-key.txt) is the ordinary prekey getter; its
SQL selects unsent/not-directly-distributed prekeys, then marks direct_distribution
and upload_timestamp and requires exactly one updated row before returning.
These similarly named getters must not be swapped. Full ordinary generation,
row-decode and signature-input paths are not all classified here.

## Constructor mapping closes the selected wire shape

The retained Ff4 branch calls EkF(v5 signed,v4 ordinary,v6 optional Kyber).
[EkF constructor](key-result-constructor.txt) maps those positions into A02,A00,A01.
The retained [job run](../2.26.43.72-outgoing-retry-timestamp/job-run.txt) unpacks
A02/A00/A01 at01ea..01fe and passes them as the first/second/third ETs arguments
to [EYI](../2.26.43.72-outgoing-retry-timestamp/envelope.txt) at0302:

| Getter/result | EkF field | EYI field | F3e.A08 wire child |
| --- | --- | --- | --- |
| A0c signed | A02 | A0C | keys/skey: id,value,signature |
| A0d ordinary | A00 | A0B | keys/key: id,value |
| A0a Kyber, optional | A01 | A0A | keys/pq: id,value,signature |

[Retained writer](../2.26.43.72-outgoing-retry-timestamp/writer.txt) checks identity
and required signed/ordinary envelopes before constructing keys at0180..018c;
it appends pq only when its optional ETs is nonnull at0236..026a. The protocol
child is pq, not pq_list, pq_last_resort_key or a new session_type attribute.
This is a selected construction chain, not live delivery/acceptance evidence.

## Rust comparison and required integration

**Confirmed current source:** kem_prekeys::StoredKemPrekey already has generated
inventory, retained signature, sent and last_resort fields; upload_rows_in and
add_rows_in select unsent ordinary keys. It has no direct_distribution field.
Rust retry construction currently emits no native carried-key branch, so no
new direct-distribution double publication is demonstrated in the current runtime.
Adding a flag without a writer or emitting PQ bytes alone would not implement
this lifecycle. Historical notes that no KEM producer exists are not current
source evidence; use the actual kem_prekeys implementation.

Before carried-key retry integration, bind remaining feature applicability and
key serialization, then design account-scoped durable selection/reservation,
upload exclusion, immutable retained wire/signature and uncertain-outcome rules
as one change. Never reinterpret a successful local mark as server acceptance,
reuse a last-resort row as an ordinary directly-distributed row, or regenerate
keys/receipts to bypass an unresolved outcome. Rust's existing once-only retry
route reservation must remain intact. Full dispatch, crash recovery, policy and
server/restriction causality remain unproven. No runtime request or key mutation
is added by this research chunk.

Reproduce apk-smali with signatures/exceptions for0WJ.A0a/A0c/A0d/A0z,0Yd.A02,
0Yj.A00,2L9.A1P,EkF.<init>,ETs.<init>. Referenced methods retain their own bundle
manifests. [checksums.json](checksums.json) pins all files here.
