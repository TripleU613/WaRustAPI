# Shared group fanout preparation — 2026-10-09

**Confirmed local correction:** the comparison group/status sender now uses
`transport::group_fanout::{Snapshot, Prepared}` for staged pairwise distributions,
sender-key encryption and atomic crypto reservation. This is a prerequisite for
normal group sending; the normal outbox group adapter and its query state machine
remain unfinished. The existing comparison CLI invokes the shared implementation
through `lab-transport`; no new group send is needed to inspect saved state.

## Existing APK evidence and port boundary

**Confirmed static, comparison APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
`SendE2EMessageJob.A06` constructs group sender-key messages with per-device
pairwise distributions, participant hash and addressing mode. The selected
[complete method](apk/2.26.37.71-groups/group-send-job-hash-check.txt) and
[recipe](apk/recipes/2.26.37.71-groups.json) are already retained; see
[planning evidence and limits](groups.md). Native feature selection, complete
own inventory, fresh target resolution and native scheduler parity are not
established by extracting the existing Rust sender.

**Confirmed static / local policy:** `D4x.A00` selects groups with exposed own
sender keys after primary identity changes. The
[sender-key exposure trace](peer-identity.md) supports the existing local
`sender_key_distribution` pending-rotation mechanism. Snapshot/Prepared reuse
that mechanism, retain original generations and never widen peer identity
acceptance. Rust optimistic concurrency and write-transaction composition are
local correctness requirements, not a newly traced APK transaction contract.

**Confirmed negative / remaining work:** normal `outgoing::Pending` correlates
one direct key query. Normal group sending still needs membership and device
resolution, bounded directory-key batches (the existing builder allows at most
32 targets per query), frozen plans, per-target policy handling, and a durable
final send reservation. Changing the direct recipient predicate to include groups
would not implement those operations. The CLI's explicit group path continues
resolving them through its existing routines; multi-batch group key-query support
and complete actor integration are not supplied by this change.

## The fresh-session race and correction

**Confirmed prior local bug:** at commit `1d0078b`, CLI `load_targets` replaced
the captured session with `None` when `fresh_sessions` was selected. Its commit
check then used `fresh_sessions || session_unchanged`. If a receive or another
operation advanced a held ratchet during key lookup/encryption, that check still
allowed the fresh staged ratchet to overwrite the newer state. Pin checking
alone did not detect a same-identity ratchet advance. The comparison is available
with `git show 1d0078b:crates/whatsapp-cli/src/transport_group_send.rs`.

**Confirmed correction:** Snapshot retains every actual original pairwise
ratchet and pin, the original own sender-key record and exposure snapshot in
one read transaction. Fresh mode changes only the encryption input and which
devices require bundles; it does not erase the original concurrency snapshot.
All pairwise ratchets and pins must match at reservation time in both modes.
The own sender key and distribution snapshot must also match. A late change
reports a fixed local diagnostic, preserves the newer state and rolls back
any caller writes in that transaction. It does not change the account identity,
adopt a replacement pin or retry a request.

Snapshot loading neither creates a network query nor generates crypto.
`required_keys` reports devices with no usable saved-session input under the
selected mode; fresh mode requires bundles for every addressed device. The
caller remains responsible for validated membership/device planning and for
authenticating/correlating directory responses. A held session row is not proof
that it is valid or that a send can succeed.

`prepare` stages real pairwise-encrypted sender-key distributions and `skmsg`
entirely in memory. Both encryption layers receive bare protobuf bytes and apply
padding once. Existing pins remain enforced, including fresh mode. The result
exposes frozen device ciphertext and group ciphertext, and retains the private
bare distribution in zeroizing memory for the existing bounded original-send
retry window. The distribution getter is secret-bearing and must never be logged
or exposed in a public response. Debug output includes counts and rotation state,
not identifiers, keys or payloads.

`Prepared::reserve_in` checks every original crypto snapshot, then reserves
exposure and next pairwise/own state inside the caller's send transaction.
The CLI commits this with its original `message:<request_id>:send` reservation,
including original wire and recipient devices, before writing any ciphertext.
An identity-triggered own-key rotation archives the exact previous generation
under the original per-send archive selector. No uncertain old send is replayed
or reset. Dropping a staged result does not commit anything; reusing a committed
result fails its concurrency checks.

The library's old sender-key comments incorrectly said to persist only after
acceptance. They now describe the actual pre-write atomic reservation contract:
a distributed uncertain generation must survive even if its ACK is missing.
Server acceptance, recipient delivery and handset confirmation remain distinct.

## Offline inspection

```sh
whatsapp-cli --profile NAME group fanout-inspect --chat GROUP_JID --own OWN_JID
```

The existing command now includes nullable `key_bundle_devices` and
`cached_session_devices`. With a complete saved device plan, these count missing
session inputs and stored session rows under the ordinary existing-session
policy. An incomplete device plan keeps both counts null and remains inspectable;
it is not silently treated as a partial send. No addresses are echoed. The
existing explicit `--hash` behavior is unchanged. Counts are not crypto readiness,
full own-device inventory or live feature selection. This command neither
prepares encryption nor rotates or reserves anything. The API reference remains
a CLI contract; no HTTP fanout-inspection route is served.

## Verification

**Confirmed local:** independent generated Signal identities/signed bundles
exercise primary and companion pairwise decryptors, extract field-2 sender-key
distributions and decrypt the staged group ciphertext back to the exact bare
protobuf. Store snapshots remain identical before reservation, and committed
state survives reopen. A second commit refuses; a later cached preparation needs
no directory reply and remains memory-only.

Eight concurrency scenarios cover fresh and existing-session modes against a
late valid ratchet advance, changed peer pin, changed own key and changed exposure.
Every failure rolls back a caller checkpoint and preserves all newer crypto
state and journal head. Separate tests verify exact prior-generation rotation
archives, cleared pending rotation, preserved old uncertain sends, missing-bundle
refusal, changed-pin refusal and incomplete-device-plan refusal. These fixtures
run offline and do not create account credentials or sends in the saved lab.

Live group/status sending through the refactored shared implementation remains
**unverified**; no new chat is sent merely to validate the extraction. Native
scheduler, group key batching, normal actor integration and complete device
inventory remain the next port boundaries. Saved-account inspection and full
checks are recorded in the checklist after verification.

Shared fanout validation: 753 workspace/all-feature tests passed, plus the final
CLI inspection regression, strict Clippy, warning-free Rustdoc, default CLI,
API symbol/reference, formatting and hygiene checks (1,117 local links; 15 APK
bundles). Offline projection upgraded zero events. Saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized retained inspection were
unchanged. No server connection, group/status send, request replay or saved
account crypto mutation occurred in this chunk. Shared staging/rotation is
verified offline; live group/status validation and normal actor wiring remain.

## Routing snapshot correction — 2026-10-09

**Confirmed local bug:** the initial shared snapshot (`c3a3cfc`) checked held
pairwise ratchets, pins, own sender keys and exposure before reservation, but
trusted the caller's already-built FanoutPlan. Membership/device rows could
change during a directory query or asynchronous encryption. An old target could
still receive a newly reserved distribution despite no longer being in the
current saved plan. Pin checks alone did not protect device-list authority; a
companion's primary pin can change independently of that companion's ratchet.
The planner also previously opened a separate read transaction for each list,
so its membership and device inventory were not one database snapshot.

**Confirmed correction:** group and explicit-audience status plans now read all
membership, recipient lists and their primary verification pins in one read
transaction. `Snapshot::load` reconstructs that plan in the same read transaction
as its crypto snapshot and compares every FanoutPlan field with the supplied
plan. A stale, fabricated or missing saved plan fails before any encryption,
query reservation or send reservation. Partial plans remain inspectable through
`group fanout-inspect`; complete inspection now requires matching saved routing.
No new command, field or network operation is added.

The snapshot privately captures the exact serialized group row (groups only),
each recipient's device-list row and primary pin, including absence of a pin.
`Prepared::reserve_in` compares those rows inside the caller's original send
write transaction before staging any crypto writes. Changed/deleted rows return
`group membership or device authority changed before commit`, without identifiers.
A caller write earlier in that transaction rolls back as well; newer routing,
ratchets, keys, original send history and journal head remain intact. No automatic
requery, membership downgrade, replacement identity or resend is introduced.

**Local conservative policy:** byte comparison includes timestamps, roles and
other metadata, so even a refresh with the same effective devices can block a
staged send. Unrelated contacts/groups are outside the snapshot and do not block
it. Status audiences remain explicitly supplied; group membership is not
invented for status. Their actual recipient lists/pins still must remain unchanged.
The fixed error identifies a local stale preparation, not a server refusal.

**Confirmed static evidence, APK 2.26.37.71** (same SHA-256 above):
`SendE2EMessageJob.A06` at `1300..1304` reads `groupParticipantHash` for the
outgoing envelope; `1jL.A0N/A0O -> AzR.A03` derives device hashes from stored
inventory. These existing traces support the inputs, not Rust transaction
semantics. **Negative result:** the equality at `0702..0708` in the retained
[send job](apk/2.26.37.71-groups/group-send-job-hash-check.txt) compares a TEE
node token; the next diagnostic explicitly says nodeToken mismatch/CVM changed.
It does not prove ordinary membership freshness or an APK database transaction.
Full native scheduler, cache invalidation and active hash selection remain
**unknown/unported**. No new native method export was needed.

**Confirmed offline:** real Signal fixtures cover twelve routing races across
fresh/cached preparation (member removal, changed/deleted device lists, missing
group, lost companion authority and a changed primary pin not directly targeted).
Each preserves the newer database state after rejection and reopen, rolls back
the caller intent and leaves the journal unchanged. Additional cases reject
missing plans and forged hash/devices/recipients/mode/own-inventory claims, allow
an unrelated update, and enforce status list freshness. Existing primary and
companion decrypt/rotation/race tests still exercise real crypto. These are
local database/crypto tests, not a server or handset result. Full normal group
outbox actor integration, own-device inventory and live large-group fanout
remain unfinished.
