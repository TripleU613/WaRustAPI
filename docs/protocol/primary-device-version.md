# Primary-device version after authenticated identity change

2026-10-09. **Confirmed static**, APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The existing [identity-receipt recipe](apk/recipes/2.26.37.71-identity-receipts.json)
retains [0Z7.A00 reads](apk/2.26.37.71-identity-receipts/primary-device-version-read.txt)
and [0Z7.A02 increments](apk/2.26.37.71-identity-receipts/primary-device-version-increment.txt).
The [observer trace](identity-receipts.md) identifies D4x.A00 0814..082a as the
primary-change producer, separately from receipt deletion and token refresh.

## Native rule and Rust port

**Confirmed:** 0Z7.A00 reads version from primary_device_version for the exact
user row, returning null when absent. Synchronized A02 calls A00, loads constant
1 at 000a and adds the old long when present at 001a. It writes that version
with the user row ID, using insert conflict mode 5, then evicts its cache entry.
DEX add-long has Java signed-long wrapping behavior; the number is not a clock.
No counterpart aggregation appears in these two methods.

**Confirmed local:** devices::after_primary_identity_change_in now keeps an
encrypted checkpoint per exact normalized primary User/LID namespace. Missing
state becomes 1; otherwise wrapping_add(1) matches the native arithmetic.
The hook is called from both authenticated incoming-message and signed-directory
archive paths. The counter, archive/event, verification invalidation, sender-key
follow-up and device-refresh intent share the caller's write transaction.
Aborting the transaction rolls all these changes back. This atomicity is a Rust
storage guarantee; it is not a newly claimed native select/delete transaction.

The existing own-account, companion, nonzero-agent and unsupported-server hook
exclusions remain. Wire kind alone does not split the same primary components.
Unchanged keys and initial pin acquisition do not call the changed-key follow-up.
Receiving a conflict observation without authenticated replacement does not
increment the counter. Malformed stored counter data refuses rather than
silently restarting at 1. No network worker, retry or account migration was added.

## Inspection and historical limits

`PeerPinState.primary_device_version` exposes the optional signed int64 through
existing offline `peer-identity --chat <PN-JID>` (phone and mapped LID states).
`peer-keys --show` reports directory matches and does not expose PeerPinState.
Missing fields in old
serialized inspection deserialize as null. A stored counter is read without
writing anything; it is not a Signal registration ID, signed device-list version,
server epoch or proof of current companion authority.

**Unknown:** historical versions before this port. Old stores are left untouched;
null means no tracked counter, not zero previous changes. The first subsequently
tracked primary change starts at 1. PN/LID counters are not merged from directory
aliases. Native receipt rows carry primary_device_version, but Rust receipt facts
and once-only send reservations do not yet use this counter. Receipt insertion,
native version-dependent cache behavior and complete canonical scope remain
unported. No acknowledged or uncertain receipt is cleared or replayed.

## Verification

**Confirmed local:** an actual encrypted direct/group prekey transition under
apk_authenticated now checks version 1. Transaction refusal and duplicate replay
after reopening retain version 1. Both archive authentication sources, key-return
transitions, independent namespace state and uncertain receipt preservation have
isolated tests. The native maximum-long case produces minimum-long, and malformed
stored types refuse. Own and companion controls create no counter checkpoints.
Legacy inspection without the new field remains readable.

Run `cargo test --locked -p whatsapp-api --all-features primary_version` and
`cargo test --locked -p whatsapp-api --all-features authenticated_replacement_is_atomic_for_direct_and_group_messages`.
A receive-only window can check current-account compatibility, but without a fresh
identity transition it cannot establish live version increments. No real peer
verification decision or synthetic live key replacement is authorized by this port.

Primary-version verification: 792 workspace/all-feature tests (662 API, four
property, nine crash, 51 CLI, one API symbol and 65 xtask), strict Clippy,
warning-free Rustdoc, default CLI/API checks, regenerated CLI reference,
formatting and build passed. Hygiene checked 1,296 local links and 24 APK
bundles / 7,401,275 bytes. Real encrypted direct/group transition fixtures
exercise the receive integration; isolated tests cover both archive sources,
rollback, reopening, namespace separation, signed-long overflow and legacy JSON.

**Observed compatibility:** one completed 45-second receive-only window
authenticated and stopped cleanly, with chat sends and automatic reconnect
disabled. Saved peer facts (apart from the new absent-version null field),
actions, receipts, outbox and event summaries were unchanged. Captures grew
502→508 with all original shapes retained; two retained group reaction targets
remain valid. No fresh identity transition occurred, so live version increments
remain unverified. The original 23-topic / 28-file / 323,094-byte APK receipt
bundle reproduced byte-for-byte.

**Observed inspection clarification:** direct `peer-identity` checks with the
previous and rebuilt binaries match after omitting only the new null counter.
Two inspected primary namespaces have absent historical versions. `peer-keys
--show` exposes directory match diagnostics, not PeerPinState; its original
compatibility comparison remains valid but cannot prove counter output. The API
and invocation description were corrected to name `peer-identity` only.
