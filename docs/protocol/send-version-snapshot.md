# Original send-time primary version snapshots

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Receipt-version evidence](receipt-version.md) traces 0Z5.A0D -> DPo.invoke
selector 1: pending device receipts bind the actual device user's nullable
primary version. Native existing-row timestamp updates retain that version.
SyncDeviceAndResendMessageJob.A0G compares prior/current versions and has a
gated absent-to-1 exception. These inputs must survive the original send.

## Rust capture and transaction boundary

**Confirmed local:** send_versions::capture_in now records one immutable
snapshot under the existing local message ID, in the normal comparison adapters.
Direct preparation captures its actual pairwise target. Group preparation uses
all frozen plan.devices. Current Rust fanout prepares a sender-key distribution
for every device; the native protocol can separate distribution recipients
from its full receipt-device target set. Unique device components determine each user's device count; wire kind
alone does not split the same user/device. Exact User/LID namespaces stay separate.
Unsupported namespaces, agents, empty users and empty target sets refuse.

Capture happens after authenticated replacement hooks. A send that replaces a
primary pin therefore captures the incremented counter in that same transaction.
Pins, ratchets, archived state, group sender-key/exposure changes, original outbox
attempt and version snapshot commit together. A failed preparation or commit
cannot leave a partial snapshot. Existing snapshots refuse overwrite; acknowledged
and uncertain reservations retain their original IDs and retry guards.

**Rust guarantee, not new native atomicity evidence:** version reads occur in
our original send write transaction. The APK's version-map read precedes its
later receipt transaction. No new native cache/transaction ordering is inferred.

## Known null versus unknown history

The snapshot privately stores actual normalized user identities, unique device
counts, optional exact device IDs and explicit nullable signed-long versions in encrypted account storage.
A known absent counter serializes as null. A missing JSON version member is
malformed and refuses; it must not silently deserialize into known-null state.
A missing entire snapshot means historical/unprepared instrumentation is unknown.
No old send is backfilled from today's counters or inferred prior key history.

Current counter reads and snapshot decoding use fixed error messages, including
malformed/unknown-field cases, so private stored values are not echoed. Store I/O
and authentication errors remain errors. No corrupted counter is reset to 1.

## Offline inspection

Use the original stable local ID from the send ticket:

```sh
whatsapp-cli --profile l outbox-versions --local-id <original-32-hex-local-id>
```

The library surface is `transport::send_versions::inspect`. It first requires an
existing outbox intent. `captured=false, users=[]` means no snapshot. Captured
rows expose only user_ticket (SHA-256 of normalized JID), devices, at_send,
current and matches. Both versions may be null; matches is nullable equality.
It is not a native keep/drop decision, a server version, safety-number proof or
permission to transmit. No user address, content, request key or ciphertext is
printed. The OpenAPI route is a CLI contract; no HTTP handler was added.

## Verification and limits

**Confirmed local:** isolated store tests cover known-null versus absent snapshots,
PN/LID separation, unique device counting, immutable capture, version change
without snapshot rewrite, reopening, rollback, unsupported targets, unknown intent,
missing version members, duplicate/unknown malformed rows and redacted failures.
An actual signed-directory replacement/prekey send fixture captures version 1,
and refusal scenarios capture nothing. Normal encrypted PN/LID group fixtures
capture all discovered devices (130 PN users and 2 LID users, one primary
device per user in these fixtures) and retain the same snapshot after reopening.
The initial helper fixture has only two devices; it is not the expanded
reservation plan and must not be used as the PN flow's expected target count.

**Unknown/unported:** native receipt rows/cache, per-device pending receipt
lifecycle, canonical counterpart mapping, effective default/hosted feature gates,
complete device-sync resend selection and recovery. This snapshot preserves a
required state input; it implements no automatic resend or reservation reset.
Compatibility can be checked offline against historical saved intents. A receive
window would not exercise new send capture, and a fresh test send requires its
own operator-approved intent. No such live send is claimed from fixture tests.

Verification completed: 795 workspace/all-feature tests passed (665 API unit,
4 property, 9 crash, 51 CLI, 1 API-symbol and 65 xtask). Strict Clippy,
warning-denying Rustdoc, 26 default CLI checks, separate library-symbol checks,
CLI-reference regeneration, formatting and experimental CLI build passed.
Repository hygiene checked 1,313 local links and 25 APK checksum bundles /
7,536,554 bytes.

**Observed offline**, retained profile: new CLI help and redacted invalid-ID /
unknown-intent refusals passed. Store generation, journal head, table/checkpoint
counts and pending-intent inspection were unchanged across these commands.
No saved normal-group preparations existed to select for historical inspection;
missing historical snapshots are exercised by isolated store fixtures, not a
claimed retained-account send. No network command or new message was run.

**Subsequent confirmed static trace:** [sync/resend](sync-resend.md) reads
receipt-device cache membership, subtracts it from refreshed candidates, then
applies version and final target filters. That cache may change through native
identity cleanup. The immutable user counts here cannot stand in for its
per-device membership or authorize any native resend selection.

## Exact original device provenance (subsequent port)

**Confirmed local/static:** new captures now include sorted, unique device IDs
for each actual normalized user namespace. These commit with the original
nullable counters and reservation. Existing count-only snapshots remain readable
with device_targets=null; no device is inferred from the count, wire list or
current membership. Mixed known/unknown rows, duplicates, unsorted IDs, count
mismatches, out-of-range/noninteger IDs and malformed rows refuse without echo.

The existing outbox-versions inspection now adds device_targets per user. Each
row exposes only device (zero for primary) and a SHA-256 device_ticket of the
normalized exact address; raw phone/LID values remain private. Wire kind alone
does not split a device, while namespace and device component do.

**Confirmed local receive correction:** group_receipts::entry_targets now uses
this complete original plan when present. It still validates the original
message's chat/ID/envelope and requires any participants/to sender-key children
to be a subset of that plan. A receipt for a planned device can therefore match
when that device has no sender-key child. Unplanned devices remain excluded;
PN/LID matches retain the existing authenticated association guard and actual
receipt actor. Old/missing target instrumentation falls back to only the known
wire distribution recipients, not invented full membership. Conflicting wire
and plan refuse atomically. Existing attempts/IDs/progress and uncertainty facts
are retained, and duplicates remain once-only after reopening.

**Negative current-builder audit:** group_fanout::Snapshot::prepare_policy loops
over every self.targets entry and prepares an SKDM for each. Its current normal
group envelopes therefore include every plan device. Missing-SKDM target
attribution is a native-shaped fixture case and future compatibility gap, not a
claimed reproduced failure of current live Rust sends.

**Evidence:** APK 37.71 0Z5.A0D/DPo.invoke seed actual device rows separately
from the distribution envelope; SyncDeviceAndResendMessageJob.A0G subtracts
receipt-cache membership before version filtering. See [the trace](sync-resend.md).
The exact immutable original plan implemented here is provenance, not the
native mutable receipt cache: receipt row timestamps/insertion/deletion and
identity-cleanup lifecycle remain unported. No live resend or new message was
authorized by the target list.

Exact-device validation: all 798 workspace/all-feature tests passed (668 API,
4 property, 9 crash, 51 CLI, 1 API-symbol and 65 xtask), alongside strict
Clippy, warning-denying Rustdoc, 26 default CLI checks, CLI-reference/API checks,
formatting and experimental CLI build. Hygiene checked 1,329 local links and
26 APK bundles / 7,808,872 bytes. Native-shaped subset/contradiction fixtures
cover planned companions, unplanned devices, immutable send state, reopening
and legacy wire-only fallback. Old count-only snapshots remain readable;
malformed device IDs refuse with redacted diagnostics.
Offline retained-profile CLI checks left store inspection and queued intents
unchanged. No retained group preparations existed to inspect; no new live
message or native pending receipt cache is claimed from these fixtures.
