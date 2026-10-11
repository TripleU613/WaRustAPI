# Directory identity authentication metadata

2026-10-10 follow-up: [primary attestation admission](primary-attestation-admission.md)
supersedes blanket primary attestation presence deferrals below. Structurally
valid changed-key candidates follow the native E2EE fallback; verified same-key
hybrid/non-E2EE candidates may refresh without an override child. Failed same-key
metadata and companion attestation remain deferred. Native state persistence
and non-E2EE routing remain unported; earlier validation below is historical.

2026-10-10. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Implemented/offline:** structurally valid identity-auth metadata no longer
blocks otherwise admitted classic refreshes. Native verification is log-only in
this APK. Certificate validation and identity-auth signature verification remain
unported; inspection reports that limitation explicitly.
This is separate from [non-E2EE attestation](attestation-verification.md) and
[classic identity refresh](identity-refresh-application.md).

## Parser and consumers

The retained [Bot.A03](apk/2.26.37.71-session-refresh-preview/bundle-candidate-parse.txt)
selects `identity_auth` from the directory user at `0158..0164`. It reads
`version` with 0Gv.A00 default -1 at `0168..017e`, required `signature` and
`certs` at `017e..0192`, and includes only `cert` children at `01aa..01c8`.
Their byte bodies form the list, with no certificate validation at this step.
The signature body, identity bytes and version reach CXs at `01e8..0206`;
Cai stores it in A03. Complete value-method dumps are retained for
[CXs](rea/2.26.37.71-identity-auth/authentication-value.txt) and
[Cai](rea/2.26.37.71-identity-auth/candidate-value.txt).

[Exact 0Gv.A00](rea/2.26.37.71-identity-auth/version-parser.txt) returns the
provided default on null/empty input or Integer.parseInt NumberFormatException.
It does not trim. The local shape reader reuses existing Java signed BMP-decimal
parsing and checks the int32 range before applying the -1 default.

[Physical field references](rea/2.26.37.71-identity-auth/candidate-field-references.txt)
find seven A03 instructions: constructor write, equals/hashCode/toString reads,
plus 14J.A04 and DLx.run consumers. This excludes reflection/native reachability.
The retained [14J worker](apk/2.26.37.71-companion-same-key-refresh/admitted-bundle-worker.txt)
reads this metadata after its session-processing path at `0236..0258`. The
[DLx async path](apk/2.26.37.71-status-observers/post-runnable-run.txt) reads
it at `029e..02bc` and logs the returned Boolean at `02be..02c8`. These
consumers process/schedule their classic session path before logging the result;
neither branches on that Boolean to admit or invalidate a session. The verifier
mode and failure routine below resolve the earlier admission uncertainty. Full
native scheduling and optional metadata processing remain unported.

## Native verification boundary

[Complete Cfj.A01 REA Evidence](rea/2.26.37.71-identity-auth/verifier.json),
`ev_e1b90df292eff17d595673ddc4467c0e2c6b8d4d0dfbfca396651e75a3fc2dac`,
reads property **20107**, decodes Bzb and falls back to A02. For A02 it returns
true before verification. Other modes require version 1 and a nonempty chain,
then call CgX.A01 with a current Date, the certificate list and false. With a
leaf certificate it concatenates UTF-8 decimal version with the supplied identity
bytes, takes the leaf's encoded public key and calls Cft.A01. Missing leaf,
invalid version, empty chain, verifier failure or caught exceptions report
through Cfj.A00 and return false. Effective property values remain unknown. The exact mode/failure evidence below
establishes a log-only result in either available mode. No runtime setting is
fabricated or forced.

[Complete Cft.A01 REA Evidence](rea/2.26.37.71-identity-auth/signature-primitive.json),
`ev_a0864b72664e8e01d1ef5b1bc9262dafb538ad54a6bf8218599a5280882a86b6`,
parses the encoded public key as DER, takes sequence element 1 as a bit string,
and passes signature/message through 027.A08 to its EdDSA backend. The
complete helper byte layout remains untraced. Extraction exceptions
can fall back to the supplied encoded bytes; represented extraction failures
return false. Provider/argument/other exceptions return false. This source has
a JADX duplicated-block warning. Exact backend return semantics and full
cryptographic equivalence remain unproven; Rust does not implement this path.
REA's static observations do not execute the APK or verify any live signature.

## API and CLI diagnostic

`whatsapp-cli --profile PROFILE actions --show ACTION_ID` and the existing
`identity_notifications::inspect` API add nullable **identity_auth**. A parsed
selected bundle with this child reports `version`, `certificate_count`,
`certificate_bytes_present`, `signature_bytes_present` and
`verification="unported"`. Missing metadata or unavailable bundle reports null.
Counts exclude non-cert extensions. Empty byte bodies count as present; they
do not establish validity. The existing top-level **signature_valid** validates
only the classic signed prekey, independently of identity_auth. No certificate,
signature, peer/account identifier or raw key is returned. The shape inspector
does not claim native malformed-input handling or chain parsing equivalence.

## Admission correction — 2026-10-10

[Exact Bzb](rea/2.26.37.71-identity-auth/verification-modes.txt) defines only
`NONE` (value 0, A02) and `LOG_ONLY` (value 1). Its constructor stores the supplied
value; the two-member enum is the lookup domain. Unrecognized property 20107
falls back to NONE in [exact Cfj](rea/2.26.37.71-identity-auth/auth-verifier-exact.txt)
`002a..0074`. NONE returns true; LOG_ONLY computes verification and returns its
Boolean. [Complete Cfj.A00 Evidence](rea/2.26.37.71-identity-auth/failure-logging.json)
`ev_9e10efa187f4bd122642fe27d2a5d00d07464d1afab191c97c8d572b4735eb7f`
and its exact method show logging and `bot-identity-verification-failure`
telemetry. This routine does not write pins or invalidate sessions. Combined
with the two consumers above, this supersedes the earlier blanket
`identity_auth_unported` application refusal, which invented an admission rule.

Application and read-only session preview share the required structural check:
a present identity_auth must contain a signature byte body and a certs node.
Missing/non-byte signature or absent certs defers with `malformed_identity_auth`.
Empty signature bytes and an empty certs node satisfy native parser structure;
invalid/missing versions or failed certificate verification can produce logs,
but do not authorize or block classic adoption. No auth trust is asserted.
Attestation/PQ still defer; signed-prekey verification, original reply/account/
notification guards, explicit apk_authenticated policy, own-identity exclusion,
same-key override rules and companion ADV/primary/membership checks still apply.
The shared application path also covers actor handling and startup recovery.
No new property, bypass option, network message or dependency is introduced.

Encrypted-store regressions failed before the correction. Tests cover changed
primary PN/LID, same-key primary with server override, and changed companion with
both ADV signatures bound to the independent primary. Metadata includes an empty
chain/signature and invalid/unsupported versions, which native log-only mode may
report as failure. Reapplication/reopening retain the once-only marker and old
history; unrelated tables remain unchanged. Missing/non-byte required fields
refuse in application and preview without table/journal changes. Forged prekeys,
pinned policy, absent same-key override, attestation/PQ and untracked notification
guards remain refused with well-formed auth present.

## Certificate chain evidence — static only

[Complete CgX.A01 Evidence](rea/2.26.37.71-identity-auth/chain-verifier.json)
`ev_41441e5a13ae45a037680a84fd9478fffed00b1e558b566048f0aa1f4f308170`
has JADX duplicated-block warnings. Its apparent unconditional loop return is
not reliable. [Exact CgX](rea/2.26.37.71-identity-auth/chain-manager-exact.txt)
`0260..02dc` verifies each child using the following issuer public key. A valid
issuer date at `0296` branches back to `0260`; only completing the ordered chain
returns the null error/leaf result at `02b6`. The decoded list supplies leaf then
intermediates, followed by the pinned root. Leaf and issuer date failures and
signature exceptions return error results retaining the leaf. Empty/unparseable
chains return no leaf. Cfj supplies current Java Date and false; its selected
path does not request the separate CRL branch. Log text saying server time does
not establish use of our confirmed protocol clock.

[CTx exact fields](rea/2.26.37.71-identity-auth/chain-result-exact.txt) and
[complete constructor Evidence](rea/2.26.37.71-identity-auth/chain-result.json)
`ev_26280b1fd0d1ca8f25fa89ee522a845ec1112531fd2c384bed4297a506a54add`
separate A00 error type from A01 leaf. Cfj reads only A01 at `00d6`, then uses the
leaf key for EdDSA; leaf presence is not evidence of a successful trusted chain.
Rust does not translate this behavior into a verified certificate verdict.

[Complete root-loader Evidence](rea/2.26.37.71-identity-auth/root-loader.json)
`ev_7cbda0f584e3b9c25a454488495b3406e1a601d99905d9946ef2ccd783873730`
retains the public PEM literal for Meta WA Feature Root CA. The loader caches a
parsed X.509 certificate, logs load failure and leaves a null cache for retry.
[OpenSSL static summary](rea/2.26.37.71-identity-auth/root-summary.json) records
753 DER bytes, SHA-256
`0cd2cb26acc5637ae6d6cb61aaee29aa960c739286f2f08fa2b77a1d0bdf492f`,
and validity 2025-09-04 through 2065-09-04. This public feature root is not
installed for TLS, account identity or Rust certificate trust. Native signature
math/backend equivalence and a fresh live identity-auth sample remain unverified.

## Validation

899 workspace/all-feature tests passed (758 library, four codec, nine crash,
54 CLI, one API-symbol and 73 tooling tests). Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests, CLI-reference regeneration and
the experimental CLI build passed. The default build retains the existing
unused clock::observe_login warning.

[Content-free saved-account report](identity-auth-offline.json): the rebuilt
CLI inspected one existing identity-refresh action offline. Its new identity_auth
field was present and null because that reply contains no metadata. Store
inspection summaries remained equal. This does not validate a live certificate
chain or prove every stored byte unchanged; the full table equality tests are
synthetic encrypted-store tests. Private logs remain in the ignored account
directory. That inspection preceded the admission correction. No new live validation was
performed for the correction; it does not prove fresh metadata adoption.
