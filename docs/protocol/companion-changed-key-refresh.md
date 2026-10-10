# Changed companion directory identity application

2026-10-09. **Confirmed static / implemented offline**, WhatsApp APK
**2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-companion-changed-key-refresh.json) and
[source bundle](apk/2.26.37.71-companion-changed-key-refresh/targets.json)
preserve 19 topics / 24 files / 197,950 bytes, including caller, admission,
own predicates, membership, baseline and pending-time producers. Older APK
class names are not substituted for these 37.71 claims.

## Caller and remote-only guard

**Confirmed:** CPB.A00 returns Bjo directly for companions. 14J.A04 invokes
11T.A0D with candidate Cai.A09 ADV bytes, A0A identity, type, diagnostic
integer 4 and Boolean false (0120–013e), then proceeds to 0jc.A0L only on
success. The parser/constructor attribution is retained in the
[same-key trace](companion-same-key-refresh.md). That early equality branch
is separate; changed identities must traverse the signature/baseline path.

**Confirmed:** 0jV.A0a reads two nullable pending-login DeviceJids under a
monitor and compares them through A08. A08 requires equal device numbers and
BO1 true for both addresses (0018–0036). 08i.BO1 reduces a supported address
to its device user and invokes BO0. BO0 compares only own stored PN/LID
namespaces; 08a delegates these own predicates. 08i.BL6 requires BO0 true
and a nonprimary DeviceJid; its 08a proxy delegates it. Thus the login/own
branches cannot apply to an established remote namespace merely because its
device number equals our linked-device number. Rust still refuses own and
unknown-own namespaces. [Own-source provenance and remaining limits](own-identity.md)
are unchanged; no pending native login or Me state is manufactured.

## Two signatures and a current baseline

**Confirmed:** 11T.A0D parses Grz/GsB at 0160–0184. An omitted
accountSignatureKey loads the held primary key (01a0–01cc). The candidate
identity comes directly from directory bytes (026c–0274), not a synthetic
incoming envelope. Both account and device signatures are required at
030a–0374. Signature details use GsB.toByteArray and raw key bytes with the
APK domains; [the serialization correction](adv-details-serialization.md)
preserves normalized fields and closed-enum/unknown-field semantics.

**Implemented:** independently held primary is required even if the signed
ADV supplies an account key; a supplied different key refuses. The verified
candidate must be ordinary E2EE and match the current signed-list raw ID.
The list's saved primary fingerprint must still match the independent pin.
Only existing exact remote membership and a differing valid held companion
pin are supported here. Missing/duplicate membership, explicit pending
teardown, first-seen identity and malformed metadata do not create authority.
Parser singleton/size/required-field and positive Java-int index bounds are
local conservative guards, not WhatsApp refusal codes.

**Confirmed:** 0bN.A04 selects stored own metadata only for own users, otherwise
remote 0bp data or a gated Chv canonical lookup. 0bN.A0C similarly returns a
remote retained map plus primary index zero; own users receive an empty map.
11T.A0D 0494–04a0 requires matching held primary, baseline raw ID and account
type. Rust uses its current independently bound E2EE list as that baseline.
Native canonical PN/LID metadata sharing, hosted/non-E2EE transitions and
primary/raw-ID replacement remain **unknown/unported** for this application;
namespace authority is never inferred from another namespace's row.

## Freshness and add-only device handling

**Confirmed:** 04c8–050e compares the actual DeviceJid's held key index.
An old/equal timestamp with a differing index returns deleted-device refusal.
A matching index may continue with an older/equal ADV. A newer timestamp
updates expected metadata through 0bN.A05 and schedules reason 3 sync
(05fa–0628). Matching index then succeeds. A differing newer index enters
remote add-only handling; 06f2–071e removes every existing address from the
proposed additions. The existing address's index is not replaced by its
newer ADV. This refutes treating the branch as general index replacement.

**Confirmed correction:** GsB stores timestamp as protobuf uint64 but the
native cmp-long instructions compare signed Java longs (04fa–050e and
05fa–0602). 0bN.A05 also compares its expected long with signed cmp-long
(0014–001c). Rust now compares those same bit patterns as i64 both in normal
companion receive and directory admission, including the pending expectation.
A high-bit candidate must not become a bogus far-future unsigned refresh;
a negative saved expectation must not become an unreachable unsigned ceiling.
This change does not claim full native timestamp handling elsewhere.

**Implemented:** eligible newer existing companions request device sync in
the same atomic application transaction, preserving verified list time, raw
ID, existing index, notification metadata and primary pin. No membership
addition occurs in this path. The existing normal receive addition capability
remains separate. Expected-time/pending revisions preserve their existing
bounds, once-only queue behavior and server outcome reservations.

## Atomic key/session replacement and inspection

A correlated successful identity_refresh result must have a valid signed
classic prekey bundle, matching original notification guard and the explicit
apk_authenticated receive policy. Attestation/PQ/identity_auth presence still
defers. Default pinned policy is unchanged. Derivation starts with no old
ratchet for changed identity; old key and serialized session are preserved
in encrypted transition history, not copied to the new identity's session.

Commit rechecks the reply, original guard, exact old pin/session, complete
registered account, own aliases and exact companion authority snapshot
(list, primary pin and login). Existing replacement hooks emit IdentityChanged,
record the transition and clean applicable companion receipt membership;
primary-only verification/list/sender-key invalidations keep their current
scope. Pending sync and the historical application marker share this commit.
A sync-state parsing failure rolls back even the event/history and new pin.
Repeat/reopen/startup recovery applies once without replaying IQs or chats.

`actions --show ID` adds nullable **companion_changed_key_authority**. It is
null for primary, incomplete/invalid-signature replies or no differing held
pin; otherwise ready/ready_refresh_pending or a fixed local reason such as
invalid_device_identity, raw_id_mismatch, missing_list_timestamp,
invalid_companion_index or deleted_device. The existing same-key diagnostic
remains separate and reports same_identity_required for changed candidates.
Ready is baseline/ADV authority only: policy, metadata, original guard and
atomic state rechecks still may defer. Inspection is read-only and exports
no signed body, identity bytes, session or address. No HTTP route, outgoing
request field, native-state override or new CLI command is added.

## Verification and remaining work

Generated PN/LID signed-directory fixtures bind both ADV signatures directly
to the queried candidate and an independent primary. Tests verify fresh
ratchets, old encrypted history, unchanged primary/list index and unrelated
tables, pending refresh for newer ADV, once-only repeat/reopen recovery and
no key-query replay. Lower/equal/newer/mismatching-index cases and uint64
high bits cover native signed ordering. Bad account/device signatures,
foreign primary/candidate, raw-ID mismatch, invalid index, old/equal deleted
index, missing floor, unsupported metadata, pinned policy, forged prekey and
historical untracked guards preserve all rows/head. Primary/list/real-ratchet
races defer; malformed pending-sync state proves complete transaction rollback.
A real encrypted companion receive fixture separately verifies the shared
signed floor/expectation correction after successful nested MAC authentication.

Fresh live changed-companion application is **unknown**. First-seen directory
identity, new directory device admission, canonical namespace reconciliation,
primary/raw-ID/type replacement and full native metadata/session effects
remain unfinished. No qualifying server input is fabricated or historical
notification guard backfilled to exercise this branch.

Validation: all **882** workspace/all-feature tests passed (API 741, properties
4, crash matrix 9, CLI 54, API symbols 1, tooling 73). Strict Clippy and
warning-denying Rustdoc passed, along with default CLI (25 plus API-symbol),
experimental CLI-reference regeneration, formatting and whitespace. Exact
19-topic / 24-file / 197,950-byte APK export reproduces and cached self-diff
is unchanged. This proves the retained scope, not comparison with a new APK.
Repository hygiene passed (2,444 local links / 124 checksum bundles,
26,166,613 APK research bytes). Fixture compilation initially needed explicit
immutable Bytes conversions; those test-only ownership errors were corrected
before the final successful full suite. No protocol finding was inferred from
that compiler failure.

Next admission boundary (**confirmed native / not yet applied here**): a
retained companion pin may still exist after membership reconciliation removes
its address. The original notification guard can describe that held identity,
but this directory helper currently reports missing_device_membership.
11T.A0D instead permits an absent newer signed index under its remote add-only
branch, with canonical admission/device-registry effects as previously traced
in [normal receive addition](peer-identity.md#remote-newer-adv-device-application-in-rust-2026-10-08).
That is distinct from a first-seen pin: preserve the original guard and held
history when implementing the directory addition branch. Do not treat this
local membership refusal as a WhatsApp server requirement or reset an account
or request to escape it.

## Bounded saved-account validation

**Observed:** one 45-second receive-only window (60.52 seconds total)
authenticated/activated and stopped cleanly. Preflight had no queued action,
receipt or eligible send. Outbox, automatic reads and reconnects were disabled.
Actions (21), receipts (9), pending outbox (0), generation, registration and
journal head remained equal. Checkpoints rose 1,110→1,118; other table counts
were equal. These are summaries/counts, not all-record byte comparisons.
[Redacted validation](companion-changed-key-refresh-live.json).

The historical primary read remains verified_bundle/pin_matches=true,
refresh_guard=untracked, applied=false, with both companion diagnostics null.
No original guard is backfilled and no application occurs. Nine unsupported
control logs and no decrypted/duplicate/undecryptable or identity ACK log were
recorded. No qualifying companion directory reply arrived; fresh live
changed-companion application stays **unknown**. No repeated window or chat
was used to manufacture evidence. Private state, configuration, key and logs
stay outside Git.

Live-record verification: CLI-reference regeneration, whitespace and repository
hygiene passed (2,446 local links / 124 checksum bundles). This follow-up changes
research/status records only; the implementation's 882 tests and strict
Clippy/Rustdoc results remain its validation. The goal remains incomplete;
the next directory membership boundary is recorded above.
