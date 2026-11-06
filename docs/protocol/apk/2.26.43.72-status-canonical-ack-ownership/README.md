# Canonical status ACK wait ownership — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected methods/classes and exact field census via xtask fallback,
not REA Evidence or runtime execution. No account operations. Runtime unchanged.
The complete sharedFef.run retains unrelated cases because its selected25/26
branches and common tails must remain reproducible without fabricated excerpts.

## Resolve producer dedup without inventing durable authority

Previously [EcN repair producer](../2.26.43.72-status-phash-repair-job/README.md)
usesDCC.A01 only on its revoke path. DCC.A01 synchronizesA02 HashSet.add(key)
and returns whether newly added. It is a process-local duplicate marker, not
an account/operation validity oracle or durable write reservation.1dW.equals
requires same runtime class, fromMe boolean, id and nullable chat equality.
Therefore it is not id-only dedup. Chat equality internals remain separate.

ExactA02 field census finds4 direct sites: initializer/add and two other jobs'
onCanceled handlers removing their original key under the same set lock.
Those complete handlers are retained. DCC.A00 removes from DIFFERENTA03 set.
The retained status repair job'sA0K clears canonical wait state but does not
showA02 removal. This bounded finding does not prove permanent retention:
alias/reflection/native writes, other cleanup and object lifecycle remain open.
Do not copy another job's removal policy or invent an expiry for the marker.

## Exception reasons and wait key

StatusStanzaBuildFailureReasonA02 is CANONICAL_ACK_PENDING, A07 NON_CURRENT_BATCH.
The [jobA0N](../2.26.43.72-status-phash-repair-job/sync-resend-job.txt) enters a
canonical wait for the first and returnsfalse for the second. It returns true
on other selected cases; do not replace that logic with enumautoRetry alone.
The enum constructor stores its boolean asautoRetry; it is a separate field
whose use by other consumers remains untraced.

2DW equality uses savedstatusRowId(long) AND null-safe exactpartCode equality;
null differs from empty. HashCode combines these fields. This is independent
of the original1dW key and the per-wait owner object. Matching only a wire ID
cannot release this wait.

Fef.run case25 (0d40) is the completion callback created by jobA01; case26
(0290) is its deadline callback. Both synchronize the captured owner object,
compare job.A0Q by identity and setA0R only on exact match. A stale callback
cannot mark a replacement wait ready. Case25 returns after unlocking; it does
not itself wake the jobqueue. Case26's common060e tail wakes through0fW/NhR.A05
only when ownership matched. Registry completion performs its own wake below.
Cleanup in the linked job clears owner/key/runnables before removing exact
listener and canceling the deadline. Complete callbacks retain catch/monitor tails.

## Persisted part transition precedes completion wake

1uS.A05 builds ContentValues for state and optional sent/ack timestamp/error.
E8T values are0 PREPARING,1 SENDING,2 SENT,3 FAILED,4 PERMANENTLY_FAILED.
ItsA00 predicate is terminal SENT/PERMANENTLY_FAILED. Epe.A01 permits:

| Current | Permitted candidate next states before A05 extra guards |
|---|---|
| PREPARING | SENDING, FAILED, PERMANENTLY_FAILED |
| SENDING | SENDING, SENT, FAILED, PERMANENTLY_FAILED |
| SENT | SENT |
| FAILED | FAILED, SENDING, SENT, PERMANENTLY_FAILED |
| PERMANENTLY_FAILED | PERMANENTLY_FAILED |

A05 adds same-state and canonical-ACK conditions; this table is not the complete
SQL admission predicate. One selected canonical SENT upgrade requires absent/
nonpositive ack timestamp. When supplied, original status UUID adds anEXISTS
predicate binding status_send_part.status_row_id tostatus.row_id ANDstatus.uuid.
Allowed prior states are included in the SQLWHERE. Empty allowed list returns
without update. The full additional guards/caller parameter authority remain open.

Only a successful update AND terminal target sets the callback-notify flag.
After transaction commit and DB close, A05 removes both(row,null) and(row,part)
listener sets under lock, unions/deduplicates a snapshot, unlocks, runs listeners,
then wakes the jobqueue. A failed/no-row update cannot notify via this selected
path. This is committed part-transition authority, not a raw inbound ACK trigger.
Other1uS operations are retained in the complete class but not declared audited.

## Reproduce

From checkout root with pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/StatusStanzaBuildFailureReason;' --signatures --exceptions > failure-reasons.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/DCC;' --signatures --exceptions > revoke-dedup.txt
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/DCC;' --field A02 --type 'Ljava/util/Set;' > revoke-dedup-fields.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1dW;' --method equals --signatures --exceptions > original-key-equality.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2DW;' --signatures --exceptions > wait-key.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Fef;' --method run --signatures --exceptions > wait-runnables.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1uS;' --signatures --exceptions > part-registry.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/E8T;' --signatures --exceptions > part-states.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Epe;' --method A01 --signatures --exceptions > part-transitions.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/jobqueue/job/SyncDeviceAndResendMessageJob;' --method A0K --signatures --exceptions > message-cancel.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/contact/sync/jobqueue/job/SyncDevicesAndSendInvisibleMessageJob;' --method A0K --signatures --exceptions > invisible-cancel.txt
```

PCs are bytes, branch deltas code units. Remaining authority includes exception
producers/current-batch identities, part-update callers, serialized restart state,
clock/scheduler domains and actual sync/sender wire construction. No Rust repair
port, ID-only wake, automatic replay, live operation or no-ban claim.
