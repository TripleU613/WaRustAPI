# Status builder batch and uncertainty admission — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports/direct censuses through xtask fallback, not REA
Evidence or execution. No account operations. This research adds no runtime,
status resend, ciphertext regeneration or jobqueue port.

## Bind exception reasons to actual producer context

Exact reason-field censuses show5 direct references forCANONICAL_ACK_PENDING
and3 forNON_CURRENT_BATCH. They include enum initialization, Eze result-singleton
initialization, F1y selected builder and job consumers. The constructor caller
census has22 invocation sites across EsK,F1y,Fmt,SendE2eStatusJob. Counts are
physical direct sites, not all reflective/dynamic/indirect reason producers.

StatusStanzaBuildException storesreason and nullablecanonicalAckPartCode; its
message/cause are delegated toException. Eze storesreason, selectedpartCode,
canonicalAckPartCode and target set. ItsA04 singleton carriesCANONICAL_ACK_PENDING,
A07 NON_CURRENT_BATCH, A05 TARGETS_UNRESOLVABLE. F1y.A01 (SendStatusRunnable
log anchor) creates these results while preparing/reusing stored batch parts,
then converts failures into the exception consumed by the retained repair job.
It is upstream of SendE2eStatusJob; this is not a complete wire-stanza builder.

## Canonical ACK is stronger than a SENT label

F1y.A03(part) returns true only forE8T.SENT AND nonnullackTimestamp>0. It does
not accept stateSENT with absent/zero/negative timestamp. The backfill selection
inA01 examines existing part assignments; spanning multiple parts returns
TARGETS_UNRESOLVABLE. For an assigned part that is neither canonical byA03 nor
PERMANENTLY_FAILED,1004..103a createsCANONICAL_ACK_PENDING with that part's code
ascanonicalAckPartCode. Other branches choose existing canonical/first parts or
produce a row-wide pending result. Empty/all-permanently-failed part conditions
can selectNON_CURRENT_BATCH; this is not a generic instruction to retry a
current status with a new id.

2Am.A00 is batch eligibility/selection, not all current-batch authority. It
checksAQ5.A0U, remote-group predicate, a row flag and saved row's batchstate;
1uS.A06 readsstatus.batch_state byrow_id.2A1 names are NOT_BATCHED,
BATCHED_PENDING,BATCHED_COMPLETE. AQ5 predicate meaning, flags and invalid-value
lookup behavior remain untraced; no eligibility is inferred from names alone.

## Persist uncertainty before further work

The selectedA01 backfill path at10e8..1188 opens a transaction, validates
parentrow_id ANDuuid usingSELECT1, and marks a chosen part only where
sent_timestamp IS NULL and state is PREPARING/SENDING/FAILED. The SQL operation
is explicitly MARK_PART_POSSIBLY_TRANSMITTED and includes sent_timestamp. If
UPDATE matches no row, it rereads the part and checks stored state/timestamp
before allowing continuation. Commit/close occurs before producing a successful
selected-part result. A marker is uncertainty ownership, not proof of socket
write, server ACK or recipient delivery.

Other selectedA01 paths persist target/part assignments and enqueue a targeted
backfill job. Failure to durably enqueue usesPART_ATTEMPT_PERSISTENCE_FAILED.
CompleteA01 preserves cleanup and other operations, but they are not all audited.
The EsK preparation method also refuses unresolved/incomplete device targets or
failed refreshed-target persistence. Its exact target refresh and transaction
helpers remain separate from actual ciphertext writer admission.

## Rust comparison and remaining authority

Current outgoing_group.Pending.next_distribution already rechecks membership
digest and its frozen RoutingGuard across ACK waits before returning another
ciphertext node. The existing regression membership_change_during_distribution_
cannot_release_more_ciphertext covers that local stop. It does not implement
native multipart/canonical waits. Native part markers, ciphertext stores,
current-batch identity, enqueue atomicity, serialized restart/clock domains,
SendE2eStatusJob rendering and final writer remain unported. No copying a native
conditional replay path into generic uncertain Rust retries.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/StatusStanzaBuildFailureReason;' --field A02 --type 'Lcom/whatsapp/productinfra/status/sendflow/StatusStanzaBuildFailureReason;' > pending-reason-fields.txt
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/StatusStanzaBuildFailureReason;' --field A07 --type 'Lcom/whatsapp/productinfra/status/sendflow/StatusStanzaBuildFailureReason;' > noncurrent-reason-fields.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/StatusStanzaBuildException;' --method '<init>' > exception-callers.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/StatusStanzaBuildException;' --signatures --exceptions > build-exception.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Eze;' --signatures --exceptions > build-result.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/F1y;' --method A01 --signatures --exceptions > send-status-runnable.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/F1y;' --method A03 --signatures --exceptions > canonical-part-predicate.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/EsK;' --method A00 --signatures --exceptions > target-preparation.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2Am;' --method A00 --signatures --exceptions > batch-eligibility.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2A1;' --method '<clinit>' --signatures --exceptions > batch-states.txt
```

PCs are bytes, deltas code units. Native DB/job behavior remains static evidence;
local Rust controls are not native execution or complete status parity. No live
or no-ban claim.
