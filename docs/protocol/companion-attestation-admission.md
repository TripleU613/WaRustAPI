# Companion attestation admission

2026-10-11, **implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Guarded companion identity-refresh application no longer treats the presence of
structurally valid `non_e2ee_attestation` as a primary-policy refusal. The existing
exact-device ADV admission still decides whether a companion session may change.

## Confirmed source and branch

The retained [CPB.A00 instruction trace](apk/2.26.37.71-attestation-verification/primary-admission.txt)
gets the candidate DeviceJid at `0000..0008`, checks `isPrimary` at
`0008..0010`, and branches from `0010` directly to Bjo at `00cc` for a
companion. This precedes the primary MobileConfig, stored C0F and attestation
verification branches. The complete [REA Java envelope](rea/2.26.37.71-attestation-admission/primary.json)
corroborates the early return; its duplicated-block warnings mean reconstructed
Boolean flow alone is insufficient. This reuses existing evidence rather than
claiming a new live observation.

The [14J.A04 worker](apk/2.26.37.71-identity-refresh-application/admitted-bundle-worker.txt)
accepts Bjo and sets its optional C0F argument to null at `00c0`. For a companion,
`0100..013e` calls `11T.A0D` with the selected candidate ADV, identity and type.
Failure refuses processing; success reaches classic session processing. The
[same-key](companion-same-key-refresh.md) and [changed-key](companion-changed-key-refresh.md)
traces retain the distinct known-member equality and signed ADV paths. Skipping
primary attestation policy does not skip ADV or make an invalid ADV valid.

Before that worker, [Bot.A03](apk/2.26.37.71-attestation-verification/attestation-candidate.txt)
constructs CWS with nullable `account_type` and required byte bodies at
`0278..02a6`. Its [constructor](apk/2.26.37.71-attestation-verification/attestation-carrier.txt)
requires nonnull byte arrays, allowing empty arrays. The shared Rust admission
helper keeps these structural checks and the existing local duplicate ambiguity
refusal, which is not claimed as an APK rule. It now distinguishes primary and
companion policy after checking those fields.

## Runtime, recovery and API

Under `transport.identity_changes=apk_authenticated`, a supported companion with
attestation metadata now reaches the same guarded session application as a
candidate without that metadata. For known same-key membership the existing
nonnull ADV/equal-key branch archives the receiving ratchet and refreshes the
session. Changed keys and returning devices still require the full independently
bound signed ADV path. First-seen and unsupported canonical changes stay deferred.

Classic signed-prekey verification, exact original refresh guards, registered
account, own-identity exclusion, held pins, membership/teardown and primary
baseline checks, PQ refusal and identity-auth required fields remain effective.
Commit still rechecks the exact saved reply, keys, ratchet, account and companion
authority atomically. Duplicate/reopen recovery cannot apply the session twice.

Inspection still reports its actual attestation verification result, including
false/null; `applied=true` is not attestation trust. No native C0F value is guessed
or persisted. Read-only preview still reports `companion_admission_unported`,
with `persisted=false` and `admission_evaluated=false`. Existing `actions --show`
and listen API descriptions document application; no endpoint, option, schema or
network request is added.

## Verification and limits

The new runtime regression failed at the blanket refusal before implementation.
It covers PN/LID companions, same/changed identities, verified hybrid and invalid
non-E2EE signatures, inspection/preview preservation, independent primary pins,
fresh versus archived ratchets, untouched unrelated tables and duplicate/reopen
idempotence. The existing signed returning-device addition regression also
applies invalid attestation for same/changed keys without bypassing membership
restoration or its primary baseline. Invalid attestation now accompanies the existing changed-companion
ADV signature, candidate, primary, raw-ID, index, deleted-device, timestamp,
classic-signature, policy, guard and other metadata refusal regressions. Empty
bytes/nullable type and missing/non-byte/duplicate fields have a focused structural
boundary test. Existing malformed attestation application tests stay effective.

Validation: 915 workspace/all-feature tests, strict Clippy, warnings-denied
Rustdoc, default CLI/API-symbol tests, CLI-reference check, CLI build and
repository formatting/link/evidence hygiene passed.

Native stored attestation transitions, non-E2EE routing, complete companion
preview/first-seen admission and fresh live application remain unfinished or
unverified. No account operation or fresh network request ran for this change.
