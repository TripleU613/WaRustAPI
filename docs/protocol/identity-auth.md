# Directory identity authentication metadata

2026-10-10. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Confirmed static / implemented read-only shape inspection.** Certificate
validation, identity-auth signature verification and native policy are unported.
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
positions do not prove that failures have no transitive effects or authorize
skipping authentication in Rust. Complete scheduling/admission remains unported.

## Native verification boundary

[Complete Cfj.A01 REA Evidence](rea/2.26.37.71-identity-auth/verifier.json),
`ev_e1b90df292eff17d595673ddc4467c0e2c6b8d4d0dfbfca396651e75a3fc2dac`,
reads property **20107**, decodes Bzb and falls back to A02. For A02 it returns
true before verification. Other modes require version 1 and a nonempty chain,
then call CgX.A01 with a current Date, the certificate list and false. With a
leaf certificate it concatenates UTF-8 decimal version with the supplied identity
bytes, takes the leaf's encoded public key and calls Cft.A01. Missing leaf,
invalid version, empty chain, verifier failure or caught exceptions report
through Cfj.A00 and return false. Effective property values, complete Bzb mode
semantics, Cfj.A00 effects, certificate trust roots and CgX chain policy are
**unknown/unported**. No runtime setting is fabricated or forced.

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

Automatic application still defers any identity_auth child with
`identity_auth_unported`; no pin, session, request ID, gate or server wait
changes. Read-only tests compare every table and journal head across inspection
and reopening, and cover Unicode version 1, unsupported version, malformed and
overflow versions, absent metadata, missing/non-byte bodies, empty byte bodies
and ignored extensions/root metadata. The regression failed before adding this
field, then passed. No new live request was made for this chunk.

## Validation

896 workspace/all-feature tests passed (755 library, four codec, nine crash,
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
directory. No directory request, connection, replay or identity adoption occurred.
