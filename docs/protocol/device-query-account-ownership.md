# Device query account ownership

## Reproduced local defect and correction (2026-11-07)

**Implemented and offline tested:** a reserved DeviceQuery checked its durable
IQ record, requested users and pending revisions, but not the saved account that
owned its reservation. A regression reserved under registration ID31, changed
the saved account to32, reopened the same encrypted store and demonstrated that
the old actor still passed check_write_authority. The pre-fix test failed on the
expected refusal. Matching IQ ID/revisions alone did not establish account
ownership for either writer handoff or device-list adoption.

reserve_query_in now freezes an internal SHA-256 authority digest from store
generation, saved account generation/phone/own JID/registration state/Signal
registration ID and identity public/private bytes (or explicit missing-account/
missing-identity states). No raw identity bytes, digest or extra field is added
to the stanza, durable query record or CLI output. Both the existing final
connection writer guard and reply ingestion reload current account authority;
ingestion checks inside the same write transaction before any list/pending/query
mutation. Changed/missing account refuses with fixed `device query account
changed`. Transaction faults continue propagating.

Account-less offline adapters remain usable only while their original absent
account/store-generation binding matches. Creating an account after reservation
changes authority. This is connection-local actor ownership, not a persistent
new callback protocol or permission to resume/replay an uncertain read. Old
records are not backfilled, and no settled reply is reprocessed. Restoration of
identical authority in the regression demonstrates the equality guard, not a
new account reset/recovery command. The writer check does not claim a lock over
the remote network write; it uses the existing pre-write cut. Pending-generation,
device signature, deadline and once-only record checks remain independent.

## Evidence and validation

**Confirmed source/local observation:** transport/devices.rs reservation,
DeviceQuery.check_write_authority and ingest_internal; connection.rs
WriteGuard::Devices dispatch. The test
device_query_cannot_cross_saved_account_change covers registration ID, identity
private bytes, account generation, own JID, registration status and account
removal after reservation, with a retained actor across disk reopen. Every
refusal preserves all checkpoint rows, pending work and absent device list.
Restoring the original owner admits one complete primary-only result; duplicate
settlement still refuses.

This fixes a demonstrated local persistence/ownership defect. It is not a newly
derived APK field or server restriction rule. Existing [device deadlines and
native callback evidence](device-query-deadlines.md) and [cross-device state
scope](cross-device-sync.md) remain distinct. APK2.26.43.72 remains pinned for
protocol research; no new disassembly was needed for this local defect. Full
native USync policy, AEA/business classification adoption and observed server
acceptance remain open. No live account operation or traffic occurred.

Verification: pre-fix regression failed at the changed-account writer guard;
post-fix workspace/all-feature suite passed **1,594 tests**. Strict workspace/
all-target/all-feature Clippy, fmt, CLI reference, OpenAPI library symbols and
APK coverage passed. Repository hygiene is checked on the staged chunk before
commit. These offline checks do not establish server acceptance.
