# Contact-check atomic preparation and usync timeout boundary

Native evidence: **confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Local persistence defect: **confirmed source/fault test**, parent7215df44.
No target execution, live account operation or claimed REA Evidence envelope.

## Confirmed local persistence defect and correction

contacts::queue_check committed the contact-check action through actions::queue,
then separately wrote contact-check-req:<id>. That second record is needed to
attribute a top-level IQ refusal, which has no per-user phone node. Failure at
the second commit left a Queued action eligible for later transmission without
its original correlation provenance. This is a local state defect, not a
claimed native wire requirement or established restriction cause.

The new fault regression arms a real redb storage backend when a page contains
the correlation prefix. Before the fix it failed: failed correlation commit
left a sendable orphan after disk reopen. The correction uses queue_in and
writes correlation in one encrypted-store transaction. IDs and wire construction
are unchanged. Full-disk failure leaves neither new record; failed fsync remains
uncertain, with reopen allowed to show neither or both, never a partial pair.
The implementation does not claim an fsync error means the commit did not land,
and does not retry preparation automatically.

Existing Queued contact-check rows also need protection. Shared single/bulk
reservation now verifies that correlation exists and matches the saved single
contact request. Missing correlation leaves the request Queued with optional
ActionOutcome.deferred_reason=contact_check_correlation_missing; bulk draining
continues for unrelated eligible actions. Selected reservation refuses with a
fixed local error. A present invalid/mismatched record fails the transaction
without exposing either phone or ID. Non-string stored data uses a fixed
shape diagnostic rather than a typed deserializer's raw value rendering.

No correlation is fabricated from a historical reply or current contact book.
A hold is not a server refusal, elapsed wait, terminal outcome or retry permit.
Existing actions plain/JSON exposes the reason; no new CLI flag/HTTP route.
Restoring explicit original provenance in the test permits one reservation and
clears the reason. This is not a shipped repair/reset command. Sent/settled rows
are not rewritten, reset or replayed. The local guard does not authenticate an
APK usync factory or establish full protocol shape/caller parity.

## Usync does not have one namespace-wide timeout

0pD.A07 receives a caller-supplied long, builds throughA06, retains its parsed
request helper and passes the long to shared0Sf.A0A orA06 with logical ID102
(0x66). HxV.A0B identifies itself as querySyncPhoneNumberLightweight and
supplies32000ms at0118..0122; its blocking future get also uses that interval.
A different0p7.A0O call supplies128000ms at09ba..09c6. The retained caller
inventory lists15 physical invocation sites; reflection/runtime reachability is
not asserted. other-caller-timeout.txt is explicitly a bounded excerpt.

Therefore xmlns=usync or a matching basic envelope alone does not prove a
universal32s timer. The Rust contact-check protocol selection is not yet bound
to one fully traced native factory/caller policy. Its callback deadline remains
unported; this chunk does not invent a namespace-wide timer or change device
query, full address-book sync, native job retry or contact throttling behavior.

## Verification and reproduction

The focused disk test failed before the correction and passed after it for
full-disk and fsync-only faults, verifying durable pair consistency after reopen.
Additional tests cover legacy missing-record hold, unrelated action drain,
single/bulk guards, explicit-provenance restoration, mismatched data redaction
and complete transaction rollback. Existing contact ingest/refusal/projection
and durable action tests remain in the full workspace checks.

Reproduce native selected methods with existing xtask apk-smali APK --class
CLASS --method METHOD --signatures --exceptions:0pD.A07,HxV.A0B,0p7.A0O.
Use apk-callers for0pD.A07. No APK or bulk decompiled tree is committed;
checksums cover every other file. The local fault test is
contact_request_and_error_correlation_commit_together_under_disk_faults.
Full current-build contact factory/protocol flags, caller-specific deadlines,
refresh/adoption/throttling and native multi-page error policy remain open.
