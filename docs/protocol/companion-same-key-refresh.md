# Known companion same-key directory refresh

2026-10-09. **Confirmed static / implemented offline** for WhatsApp APK
**2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-companion-same-key-refresh.json) and
[source bundle](apk/2.26.37.71-companion-same-key-refresh/targets.json) retain
10 topics / 15 files / 176,535 bytes. Cached comparison does not rescan the APK;
self-comparison establishes reproducibility, not parity with a new release.

## Native branch and why ADV is not parsed here

**Confirmed:** `CPB.A00` checks DeviceJid.isPrimary at 0008–0014. A companion
branches directly to Bjo at 00cc, before primary attestation-state/property
admission. `14J.A04` accepts Bjo/Bjn and invokes `11T.A0D` for a companion
(0100–013e); only a true result continues to `0jc.A0L` (01ae–01b4).
The candidate parser `Bot.A03` selects the user's device-identity child at
00c4–00d0 and its byte body at 02d2–02de, passes v36 at 031c–0320 to
`Cai.<init>`, which stores it as A09. It is not a root field or attribute.

**Confirmed:** `11T.A0D` refuses null ADV (0044–0064) or null candidate
identity (0068–0088) *before* its known-device check. It gets current membership
from `0bN.A0B(user)` at 0088–0098 and tests the actual DeviceJid at 0098–00a8.
For a member it loads the held companion key through `0jc.A0W` at 00a8–00bc,
decodes the candidate with `11T.A02`, and compares B7D at 00c4–00da.
An equal key logs "already have device" and jumps from 00e8 to 0e42,
closes the lock and returns true at 0e48. ADV parsing starts later at 0160.
Thus this branch does not inspect ADV signatures, details, index, raw ID or
timestamp and does not schedule a list refresh. A nonnull empty byte array
qualifies for this particular branch; it is not proof of a signed ADV identity.

`B7D.equals` compares A00 via 00h.areEqual (000e–0020); `B7E.equals` compares
A01 raw key arrays with Arrays.equals (000e–0020). The native equality ignores
its separate type byte. Rust retains its supported type-5 bundle restriction,
and does not generalize this finding to unsupported key types.
`0bN.A0B` selects the user's retained remote map key set (0084–009e) and adds
the primary address (009e–00b8); own PN/LID collections take different branches.

**Confirmed:** `0jc.A0L` skips changed-key reset for an equal identity but still
calls CQc.A00. CQc validates the signed prekey and archives the held current
ratchet before deriving a new session. This is session refresh, not a no-op.
[Primary same-key derivation](same-key-refresh.md) retains the shared trace.

## Bounded Rust application and local protections

Only a correlated saved identity_refresh result with a valid signed classic
bundle, matching original notification guard and equal exact companion pin
can take this path under explicit `transport.identity_changes=apk_authenticated`.
Default pinned policy stays unchanged. No force_override_e2ee_state child is
required for companions. That child's primary admission rule remains separate.
Present non_e2ee_attestation, pqkey or identity_auth still defers; the full
native metadata/session implementation is **unknown/unported** here.

The runtime also requires a registered saved account, known own namespace,
excludes own PN/LID devices, a unique Bytes device-identity child (local 16 KiB
cap), unique exact current membership, and a retained list bound to the current
independently saved primary pin. An explicit pending teardown refuses. These
are **local conservative protections**, stronger than native early return;
unknown historical removal state is not manufactured as an empty registry.
No ADV signature verification is claimed or needed for the equal-held-key
native branch. A changed companion identity remains deferred pending the full
ADV/admission trace; this does not authorize first-seen devices or list addition.

Before the single atomic write, the runtime rechecks saved reply, original
guard, exact old session and companion pin, full account fingerprint, own
aliases, and exact membership list/primary pin/login authority snapshot.
Changed authority returns companion_authority_changed_before_commit; changed
ratchet returns peer_state_changed_before_commit. Storage corruption remains
an error and rolls back. The refreshed session archives the previous receiving
state; equal pin, verification/version/transition history, sender keys, outbox,
receipts, device-list/refresh state and event head stay unchanged. An encrypted
historical application marker makes repeat/reopen/startup recovery once-only.
Recovery never reconstructs or replays the directory query.

## Read-only diagnosis and verification

`actions --show ID` adds nullable **companion_same_key_authority**. It is null
for primary, incomplete or invalid-signature replies. A supported signed
companion reply reports ready or a fixed local reason such as
missing_device_identity, missing_device_list, stale_device_list,
missing_device_membership, ambiguous_device_membership, same_identity_required,
pending_device_teardown, own_device or missing_own_identity. Ready describes
these authority prerequisites only: policy, metadata, original guard and
atomic reply/account/session rechecks may still refuse application. Inspection
never opens ADV, queues an IQ, rewrites state or proves server acceptance.
No new HTTP route, configuration override or outgoing field is introduced.
`--preview-session` remains a separate read-only primary preview; this change
does not claim companion preview support.

Synthetic PN/LID tests use independently generated signed prekeys and existing
sessions. Empty/non-protobuf ADV bytes deliberately distinguish the native
fast return from signature verification. Tests check archived old state plus
new base key, every unrelated table and the exact device-list checkpoint,
unchanged journal head, once-only recovery/repeat/reopen, and no IQ replay.
Negative tests preserve all rows for missing/nonbytes/duplicate/oversize ADV,
missing/stale/foreign/duplicate/nonmember lists, pending teardown, changed key,
own/unknown own namespace and missing primary pin. Races on primary pin, list,
login authority and real ratchet progress refuse without overwriting state.
Existing forged-signature, unsupported-metadata, pinned-policy, original guard,
account and reply-race checks remain exercised.

Fresh qualifying live companion application is **unknown**. No request or
control was fabricated to exercise it. Changed companion admission, complete
native attestation effects and cross-device receive completion remain unfinished.

Validation: all **878** workspace/all-feature tests passed (API 737, properties
4, crash matrix 9, CLI 54, API symbols 1, tooling 73). Strict Clippy and
warning-denying Rustdoc passed. Default CLI (25 plus API-symbol test),
experimental CLI-reference regeneration, formatting and whitespace passed.
The 10-topic / 15-file / 176,535-byte source export reproduces exactly and
cached self-diff is unchanged. Repository hygiene passed (2,433 local links,
123 APK bundles / 25,968,663 bytes). The initial Clippy run found a collapsible
conditional; it was fixed before the final full test/lint runs. No live branch
acceptance is claimed by these offline checks.
