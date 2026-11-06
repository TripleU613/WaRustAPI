# Status phash repair producer and resend-job boundary — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods/class through xtask fallback, not REA Evidence or
execution. No account operations. This is retained research; no Rust repair,
resend, retry or runtime policy change.

## From ACK mismatch to queued job

[1r6.A01](../2.26.43.72-status-ack-persistence/plain-success.txt) callsEcN.A00
withDwc and edit7/8 revoke predicate when participant is null,phash nonempty,
and local computed hash differs. EcN reloadsDto.A0H(key), adds ownLID device
when available, then0Dn.A0F projects a set of user recipients. Exact membership
projection/feature selection remains untraced; do not substitute all contacts.

EcN gets0yD and buildsSyncDeviceAndResendStatusJob. Empty recipients return.
Revoke path additionally callsDCC.A01(original1dW) and returns if false; its
operation/ownership predicate remains untraced. Job retainsDwc, recipient set,
statusUUID, raw remotechat and serialized raw users. The producer setsstartTimeMs
from a provider clock, expirationMs0 for revoke and another clock+0x36ee80
(3,600,000) otherwise. No claim these clock providers are interchangeable.
It submits the job through0fW.A01. This is confirmed conditional replay-related
work, not simply a local cache refresh or a new registration/account flow.

ExJ.A01 constructsDra report fields using chat classification, revokeboolean,
constantboolean andF3l-derived chat value.2LA.A11 resolves0NX and callsCSW(0FP).
That is the report-submission boundary, not evidence of a particular HTTP
endpoint or packet. No telemetry suppression/emission policy is inferred.

## Job admission and sending are stateful

The complete job class is retained, withA0L as onRun (log anchor). Its selected
entry checks positive expiration and returns when clock>expiration; equality
continues. Expiration0 skips this positive-expiry check, not all later deadlines.
A0L invokes0ob.A09(0vC.A0M,recipientSet), waits on its future, checks1xM.A01,
reloads saved status through1ti.A00(Dwc), and handles a missing saved row by
logging/returning. It uses returned user sets and saved status metadata to select
repair recipients; the full selection and operation conditions are not ported.

The selected resend path updatesDto.A0R(key,recipientSet,false), createsEY7/
Enh retaining jobstart/expiration and selected recipients, and callsFLk.A00.
FLk does further checks and dispatches lower builders; no wire stanza or write
reservation authority is established merely by reaching that call. Future
completion alone is not a server ACK or delivery fact.

## Canonical ACK wait prevents a naive retry port

A0N inspectsStatusStanzaBuildException directly or through its cause. For one
specific reason (A02, symbolic meaning not yet traced), it setswaitingForCanonicalAck,
retains exceptioncanonicalAckPartCode, sets scope1 when null or2 when nonnull,
and calls privateA01 before returningtrue. Another reasonA07 returnsfalse;
remaining cases returntrue. These native return values are not mapped to Rust
reconnect/replay categories without the jobqueue/exception contract.

PrivateA01 first cleans old wait registration, derives a deadline from positive
expiration orstart+1hour, and uses the clock before registration. It reloads
savedDyI/AQ5 and row ID. It retains an owner object plus row/part/runnable, registers
in1uS.A03 under2DW(rowId,partCode), schedules a deadline runnable with remaining
time clamped to0 and inspects existing part records. Scope1/2 and persisted part
states affect readiness. This is row/part-specific wait ownership, not id-only
ACK settlement. A00 clears owner/row/part/runnable fields, removes its exact
listener from the set/map and cancels its scheduled runnable.

A0M first checks all job requirements; unmetChatConnectionRequirement marks
connection-loss state. While waiting, it checks readiness/deadline before
clearing waiting/scope/part and allowing continuation. Exact callback Fef cases,
2DW equality,1uS ownership, exception reason enum, serialized validation and
scheduler contract still need tracing. Full onRun later failure/timeout branches
remain unported even though complete source exports are retained.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/EcN;' --method A00 --signatures --exceptions > repair-producer.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/ExJ;' --method A01 --signatures --exceptions > repair-report.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2LA;' --method A11 --signatures --exceptions > report-submit.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/jobqueue/job/SyncDeviceAndResendStatusJob;' --signatures --exceptions > sync-resend-job.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/FLk;' --method A00 --signatures --exceptions > send-builder.txt
```

PCs are bytes; deltas code units. The bounded evidence identifies actual network-
related repair producers and replay-admission prerequisites. It does not
implement them or authorize uncertain Rust sends to replay. No live/no-ban claim.
