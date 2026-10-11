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
or persisted. The companion preview follow-up below supersedes the original
blanket preview refusal. Existing `actions --show`
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

## Companion session preview — 2026-10-11

The existing `actions --show ID --preview-session` now derives and discards a
supported companion candidate using the same read snapshot as inspection. It
uses the existing exact-device same-key or changed-key directory authority
checks, backed by the pinned `11T.A0D` and `14J.A04` traces linked above. No new
native admission claim is inferred from successful cryptographic derivation.
Known same-key membership uses the native nonnull-ADV/equality branch; changed
keys and returning devices require independently bound signed ADV. Missing,
invalid, deleted, ambiguous or conflicting authority reports its fixed reason.
First-seen pins remain unsupported.

Metadata shape, classic signature, PQ refusal and saved account requirements
remain effective. Companion attestation follows the existing early companion
return without turning failed attestation verification into trust. Same identity
archives the held receiving session in the discarded candidate; changed identity
derives fresh with zero archived states, preserving saved old state. Primary
changed-key preview retains its existing refusal.

`persisted` and `admission_evaluated` stay false: no full native CPB stored-state
policy, application-policy approval, commit guard, list addition, pending refresh,
key change, acknowledgement or network action is performed. A ready addition or
refresh remains pending. As before, an untracked historical refresh can be
previewed; it cannot authorize application. Candidate bytes are never exported.
The existing API schema now enumerates companion refusal reasons and malformed
identity_auth; no command, route, option or dependency is added.

The previously blanket-refused companion regression failed before the change.
PN/LID fixtures cover same/changed keys and valid/invalid attestation, archive
counts and unchanged tables/journal. Returning-device fixtures cover discarded
same/changed candidates and missing/invalid ADV, signatures, independent primary,
raw ID, timestamps, teardown, index collision/overflow and invalid index refusals.
Existing primary preview/error, redaction and reopen checks remain active. Fresh
live companion preview/application and complete native parity remain unverified.

Validation: 945 workspace/all-feature tests, strict Clippy, warnings-denied
Rustdoc, 26 default CLI tests, experimental CLI build/reference and minimal
TCP/Noise/Signal companion regression passed. API enums cover the shared
directory refusal literals; formatting and repository link/evidence hygiene
passed. Existing pinned source bundles are reused without a new APK parity claim.
