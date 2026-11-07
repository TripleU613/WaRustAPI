# Normal group writer account ownership

## Reproduced unguarded write cut (2026-11-07)

**Implemented/offline:** NodeLink.send_group_step selected a writer guard only
when Pending exposed a device query. Membership/key queries and distribution/
final-message steps had no saved-account guard; a final message may also have
no remaining Pending. A regression queued a non-device IQ, replaced saved
registration ID31 with32 and showed Outbound.prepare still admitting encoding.
The pre-fix expected-refusal assertion failed at that IQ case.

Every normal group step now freezes the connection account fingerprint when
queued and rechecks it in Outbound.prepare. All three normal group handoff sites
pass the connection's account snapshot, including the final message with no
Pending. The binding covers connection-account/store generation, phone, own JID,
registration state/ID and identity bytes through the shared transport helper.
No raw key/digest or new field is emitted on the wire. A mismatching/missing or
stale-generation account is refused; no automatic repair, reset or resend is
introduced.

## Composition with device admission

GroupStep holds optional cloned DeviceQuery authority in addition to the account
fingerprint. Writer admission first checks the connection account, then invokes
the original query's check_write_authority when present. Thus the account guard
does not replace existing device-row ownership, pending revision or deadline
checks. Normal non-device group steps now also have a guard. A failure consumes
the queued outbound and returns its fixed local error without encoding or
writing; existing uncertain group reservations remain, not replay permission.

This is the normal writer boundary, not a transaction lock held over the socket.
It does not establish full account ownership of earlier group staging, Row
persistence, bundle application or sender-key commit; those remain separate
follow-up audit work. Direct returned-node/offline APIs, group retry-resend paths
and non-normal adapters are not silently granted this transport guard. No ACK
ordering, membership policy, payload shape or response timer is changed.

## Tests and fixture corrections

**Confirmed source/local:** connection.rs WriteGuard, Outbound.prepare,
send_group_step and its three normal driver callers. No new APK decompilation
was needed for this local handoff defect. Existing native group/device evidence
and the pinned APK2.26.43.72 remain unchanged.

queued_non_device_group_steps_refuse_changed_account_before_encoding covers
non-device IQ/message steps, unchanged and replaced account states, and final
messages without Pending. Unchanged admits exactly one write; replacement
refuses before encoding and keeps all checkpoint rows unchanged.
group_account_guard_preserves_device_reservation_admission holds a matching
account but changes the reserved device-query status. The combined group guard
must still refuse and preserve checkpoint rows.

The first full run exposed three group transport tests saving an uninitialized
fixture generation. Four instances of that shared fixture setup now assign the
actual store generation before saving Alice, preserving the new account fence.
Existing socket-cut/no-replay, cancellation and late-device-result tests continue
to exercise normal group lifecycle behavior. No live account operation, traffic
or server restriction/acceptance claim.

[Direct outbox ownership](direct-outbox-account-ownership.md), [group projection](group-projection-ownership.md)
and [network audit](network-correctness-audit.md) retain other lifecycle limits.
The broader network/protocol audit remains unfinished.

Final verification: **1,604 workspace/all-feature tests passed** after correcting
the fixture generations. Strict workspace/all-target/all-feature Clippy, fmt,
CLI reference, OpenAPI library symbols and APK coverage freshness passed.
Staged diff and repository hygiene are checked before commit. These offline
checks do not establish server acceptance or full group lifecycle parity.
