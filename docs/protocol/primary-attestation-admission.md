# Primary attestation admission

2026-10-10, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The guarded identity-refresh application and read-only session preview now share
the primary branches whose acceptance does not depend on missing native state.
This supersedes earlier blanket `non_e2ee_attestation` presence refusals for
primary PN/LID candidates. No new request, endpoint, property override or schema
migration is introduced.

## Native evidence

**Confirmed static:** the complete [CPB.A00 instruction trace](apk/2.26.37.71-attestation-verification/primary-admission.txt)
establishes the branch destinations below. The complete [REA Java envelope](rea/2.26.37.71-attestation-admission/primary.json)
is corroboration, with six duplicated-block warnings; its reconstructed Boolean
flow alone is insufficient evidence. Method offsets below are byte offsets.

* Main property 28527 (`0x6f6f`) disabled: `0030 -> 00cc` returns Bjo,
  continuing classic session processing without attestation policy.
* Changed primary identity: `00c2 -> 00d2` sets v12=1. Verified metadata reaches
  `0222 -> 0226 -> 0306` and returns Bjn. Failed verification reaches
  `01cc -> 01d0`, explicitly logs acceptance with key change as E2EE,
  then `01e4 -> 0222 -> 0226 -> 0306`. The failure categories include
  nullable/unrecognized declared type, bad signature width, malformed/default
  BP2, type mismatch and invalid signature. This fallback does not turn a failed
  signature into a successful verification result.
* Same primary identity, verified `hybrid_e2ee` or `non_e2ee`: after success,
  `022e` or `0236` branches to `02dc`, then Bjn at `0306`, irrespective of
  held C0F, override child or property 29847. If the stored classification
  already matches, `00c8 -> 00cc` returns Bjo before verification. Both results
  permit processing; the verified invariant does not require guessing that state.
* Same identity with failed verification and a different classification:
  `01cc -> 01ea -> 0270 -> 027a` depends on property 29847. An override child
  does not rescue this failed-signature branch. Rust leaves such candidates
  deferred because the effective property and stored classification are unknown.

[Bot.A03](apk/2.26.37.71-attestation-verification/attestation-candidate.txt),
`0278..02a6`, supplies required `value`/`signature` children and their byte
bodies. `account_type` uses 8kT.A18, whose [complete REA method](rea/2.26.37.71-attestation-admission/optional-account-type.json)
calls `A0M(name, null)`; it is optional. [CWS constructor](apk/2.26.37.71-attestation-verification/attestation-carrier.txt)
checks the two byte arrays and stores the nullable string. Empty byte arrays
are structurally valid and subsequently fail verification. Rust preserves its
existing local duplicate-node ambiguity guard, which is not claimed as an APK
rule. Missing/non-byte bodies and ambiguous candidates remain deferred.

[0jc.A0L](apk/2.26.37.71-identity-refresh-application/session-processing.txt)
uses the C0F argument only at `0326..034c`, after session processing, to call
0kc.A0F when nonnull and the main property is enabled. The argument does not
change the preceding classic prekey/session derivation. Optional native
attestation-state persistence remains separate and unported.

## Runtime and API behavior

The existing `apk_authenticated` policy still requires a registered account,
verified classic signed-prekey bundle, exact selected device, held pin and the
original matching reply/account/key/session guard. Own identities, untracked or
stale refreshes, PQ metadata, malformed identity-auth fields and unsupported
companion attestation remain refused. Pinned policy does not adopt a changed key.

Structurally valid changed-primary attestation proceeds through the existing
atomic identity/session application, even when its inspection reports failed
verification. A same-key primary requires verified attestation or the existing
absent-attestation server override branch. A present invalid attestation cannot
use that override to escape the attestation gate.

Existing actions and identity-refresh inspection expose the same redacted
verification fields and application marker. `attestation.signature_valid` remains
false/null on failed checks; `applied=true` does not claim certificate or
attestation trust. Same-key sessions archive the receiving ratchet without an
identity transition. Changed-key sessions start fresh and preserve encrypted
identity history. Duplicate application and reopen recovery do not reapply.

Read-only preview can prepare a matching-key verified attestation session;
changed-key candidates still report `identity_replacement_unported`. Neither
preview nor inspection commits anything. `admission_evaluated=false` remains
accurate: complete native state/property admission was not evaluated.

## Verification and limits

The regression failed before production edits with the blanket attestation
refusal. Tests cover primary PN/LID, same/changed keys, both account types and
valid/invalid signatures; saved-table preservation, old-ratchet archival,
duplicate/reopen idempotence; missing/unknown type, empty bytes, type mismatch,
malformed protobuf and the inspector's 64 KiB limit; absent/non-byte/duplicate
fields; classic-signature, pinned-policy, original-guard, PQ and companion
refusals, and invalid same-key metadata even with override. Preview is read-only.
The 64 KiB bound is an inspection limit, not a fabricated native admission gate
for the changed-key fallback. Existing transport/store bounds still apply.

Validation: 908 workspace all-feature tests, strict Clippy, warnings-denied
Rustdoc, default CLI/API-symbol tests and experimental CLI-reference checks
passed. The default build retains its existing `observe_login` unused warning.

No fresh qualifying live attestation reply was observed or requested for this
change. Native C0F storage, full property precedence, companion attestation,
non-E2EE send routing and complete observer effects remain unported. This change
does not claim complete native behavior, ban immunity or live interoperability.
