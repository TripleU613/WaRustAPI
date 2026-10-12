# Identity refresh session preview

2026-10-11 — **Implemented/offline:** primary changed-key preview now derives
and discards a fresh classic session with zero old receiving archives, matching
the existing guarded application path. Same-key candidates continue archiving
held state. Original pin/session/account/action/receipt/outbox tables and journal
remain unchanged, including after reopen. This supersedes the historical changed
primary refusal below; full CPB admission and state persistence remain unevaluated.

The supported candidate must have a verified classic signed-prekey bundle,
a saved pin, an original matching or historical untracked read guard, no PQ
metadata and structurally valid identity-auth fields. Present primary attestation
shares the existing application invariants: changed keys allow structurally valid
metadata even when attestation verification fails; same keys require verified
metadata. Missing/non-byte/ambiguous required bodies still refuse. The existing
same-key absent-attestation preview remains cryptographic derivation rather than
proof of server override or native permission. No original request or wait is
cancelled, replayed or repaired.

The native source is unchanged: [0jc.A0L](apk/2.26.37.71-identity-refresh-application/session-processing.txt)
compares held remote identity at 0044..008a and takes the reset paths before
classic derivation on a mismatch. The exact cleanup/observer surface remains
partly unported; this preview does not execute it. Existing
[changed-key application](identity-refresh-application.md) uses the shared
classic helper with no previous session and the verified candidate identity.
The preview now selects that same fresh derivation, held in zeroizing memory.
Candidate bytes, base keys and identifiers are never exported.

Tests cover primary PN/LID changed keys with held state, a changed key without
held state, original same-key archival/error cases, both declared attestation
types and valid/invalid signatures, malformed metadata fallback, unsupported
inputs and stale guards. All saved-table snapshots and journal heads remain
equal. These are generated offline fixtures; no fresh live identity transition
or complete native cryptographic/session equivalence is claimed.

Validation passed 956 workspace/all-feature tests, strict Clippy, warnings-denied
Rustdoc, the preview regression with only TCP/Noise/Signal enabled, 26 default
CLI tests, API-symbol checks and the experimental CLI build. Formatting and
repository link/evidence hygiene passed. The existing
eight-topic identity-refresh application bundle reproduced all 13 files /
119,638 bytes against the pinned APK. No new source or effective-property claim
is inferred from this reused evidence.

2026-10-11 follow-up: [companion session preview](companion-attestation-admission.md#companion-session-preview--2026-10-11)
adds supported same/changed-key and returning-device derivation through existing
directory ADV authority. It supersedes the primary-only and blanket companion
refusals below; no candidate or authority effects are committed. Earlier tests
below describe the original scope.

2026-10-10 follow-up: [primary attestation admission](primary-attestation-admission.md)
supersedes blanket primary attestation presence deferrals below. Structurally
valid changed-key candidates follow the native E2EE fallback; verified same-key
hybrid/non-E2EE candidates may refresh without an override child. Failed same-key
metadata and companion attestation remain deferred. Native state persistence
and non-E2EE routing remain unported; earlier validation below is historical.

2026-10-09. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This continues the [durable notification key read](identity-notification-key-read.md).
It adds an invokable cryptographic preview, not automatic response adoption.

## Held identity is not a session-reuse instruction

**Confirmed static:** [0jc.A0L](apk/2.26.37.71-session-refresh-preview/session-processing.txt)
compares the held session's remote identity type and bytes at `0044..008a`.
Matching bytes branch from `0086` to `009c`: they avoid the earlier changed-key
reset, but still continue key decoding and call CQc.A00 at `0310..0324`.
They do not return with the old current ratchet. The surrounding mode/property
branches are native implementation details, not fully ported Rust behavior.

[CQc.A00](apk/2.26.37.71-session-refresh-preview/signed-bundle-derivation.txt)
checks the signed key at `0030..0064`, loads the held record and generates a
fresh ephemeral. A nonfresh held record is archived at `0124..014c`, before
fresh derivation and eventual session/identity store calls. Optional PQ branches
and complete exception/status equivalence remain unported. This trace applies
after native admission; it does not prove every returned bundle is admitted.

**Implemented/offline:** `signal::direct::prepare_session_refresh` validates
bundle identity/signature, checks a held session's current remote identity
against its pin, and invokes the existing classic Signal bundle processor even
when a session exists. It never constructs message plaintext or ciphertext.
The backend promotes a fresh state and retains the old state for receiving.
Candidate serialization is held in zeroizing memory and discarded by the preview.
The ordinary encryption API retains its existing-session reuse behavior.

A generated-key test starts from an actual encrypted-message session. The
candidate has a different current base key and one additional archived state;
the archived base key and sender counter match the original. A forged supplied
signature refuses. Original input bytes and pin remain equal. This proves the
bounded backend behavior, not complete APK cryptographic equivalence, PQ support
or a live persisted session refresh.

## Native stored admission state

**Confirmed static:** [0kc.A09](apk/2.26.37.71-session-refresh-preview/attestation-state-address.txt)
gets address components from B76.A00 and delegates to
[0kc.A01](apk/2.26.37.71-session-refresh-preview/saved-attestation-state.txt).
A01 selects `account_encryption_attestation_type` from `identities` by
`recipient_id`, `recipient_type`, and `device_id` (`0010..001e`). No matching
row returns C0F.A05 (`0028..0030`); an existing row uses its integer column
and C9D.A00 (`0032..004e`). The follow-up [stored-state trace](attestation-state.md) establishes a
NOT NULL DEFAULT 0 schema descriptor and conditional primary backfill.
Historical SQL migrations/null interpretation and Rust provenance remain unknown.

[C0F.<clinit>](apk/2.26.37.71-session-refresh-preview/attestation-state-values.txt)
passes these explicit integer values to its constructors:

| Field | Name | Value | Offsets |
| --- | --- | --- | --- |
| A05 | NOT_SET | 0 | `0000..0014` |
| A02 | E2EE | 1 | `0014..0028` |
| A03 | HYBRID_E2EE | 2 | `0028..003c` |
| A04 | NON_E2EE | 3 | `003c..0050` |

[C9D.A00](apk/2.26.37.71-session-refresh-preview/attestation-state-decode.txt)
iterates the enum adapter by `intValue`; an unmatched integer returns NOT_SET
(`0036..003c`). That is native decoding behavior, not authority to assign a
state to a historical Rust identity which never stored this field.

**Confirmed static:** [Bot.A03](apk/2.26.37.71-session-refresh-preview/bundle-candidate-parse.txt)
reads `pqkey` (`020a..020e`) and `non_e2ee_attestation` (`0248..0254`). With
property `0x6f6f` enabled and attestation present, it reads `account_type`,
required `value` and `signature` child bytes into CWS (`0278..02a6`). The names
are established by v13 at `0100` and v4 at `014c`, respectively; constructor
argument order is value then signature. Without attestation, `0258` branches
to `0354`; with the same property enabled it reads presence of
`force_override_e2ee_state` (`0370..0384`). Property disabled/present branches
also carry null CWS. These properties are not configurable bypass switches.
Actual server-assigned values and complete attestation primitives are unknown.

[CPB.A00](apk/2.26.37.71-session-refresh-preview/primary-bundle-admission.txt)
bypasses its primary attestation policy for nonprimary candidates or disabled
property `0x6f6f`, returning Bjo at `00cc`. Otherwise absent CWS selects E2EE
at `02d4`, then **rejoins `0054`** (`02d8 - 0x142*2`), so saved identity/state
checks still run. This branch does not jump straight to bundle acceptance.
Metadata `hybrid_e2ee`/`non_e2ee` select the corresponding enum, with other
strings initially classified as E2EE. Classification is not verification.

Same identity plus unchanged native state returns Bjo at `00c8..00d2`;
other paths include attestation verification and changed-key handling. At
`0226..0256`, unchanged-key E2EE with saved NOT_SET reaches Bjn (`0306`),
while saved HYBRID/NON needs the candidate's explicit override flag to reach
that branch. Other paths consult property `0x7497`: false returns Bjo, true
returns Bjp (`027a..0298`). Bjo continues processing without a new state;
Bjp skips the candidate in the downstream worker. Logged failed-attestation
fallbacks are retained evidence, not a general admission policy for Rust.
Complete matrix/primitive authority and current account property values remain
unknown. [Earlier bulk processing boundaries](bulk-prekey-processing.md)
still apply, including companion ADV verification.

**Unknown / unported:** Rust does not persist this native attestation-state
provenance. Missing historical Rust data must not be called native NOT_SET,
E2EE or proof of downgrade permission. The new preview therefore always reports
`admission_evaluated=false` and `persisted=false`, including successful derivation.

## Invoking the bounded preview

```sh
whatsapp-cli --profile PROFILE actions --show ACTION_ID --preview-session
```

**Implemented/offline:** `identity_notifications::preview_session` takes one
read transaction containing the original action, correlated reply, current
exact-namespace/device pin, session and account. It prints the existing redacted
inspection plus `outcome`, `held_session`, `archived_states`,
`admission_evaluated`, and `persisted`. No peer/action identifier, public key,
session bytes or raw stanza is exported. Unknown action kinds refuse with
`not an identity refresh action`; malformed saved sessions use fixed local
cryptographic diagnostics. `--preview-session` requires `--show`.

Only a verified classic primary bundle with the unchanged saved pin can derive.
Present attestation or PQ metadata, companions, changed/missing pins and
unverified/uncertain/refused replies report fixed reasons without derivation.
A valid pin with no held session can stage a fresh record with zero archives.
This is a local preview scope, not a claim that WhatsApp forbids these cases.

Tests cover all those paths and malformed held sessions. Before/after plaintext
snapshots of all 14 encrypted-store tables and the journal head compare equal,
including after reopening. The fixture uses generated local identities and
synthetic replies; no live server or account is needed. Repeating a preview
uses new transient entropy, but does not queue, reserve, transmit or persist
anything. It does not cancel or replay the original read.

Automatic adoption still needs native admission/provenance, notification-time
pin/version snapshots to reject stale responses, atomic current-session guards,
companion authority, mappings and the deferred notification ACK lifecycle.
A held session can advance while a directory read is in flight; freezing it at
query time would lose legitimate ratchet progress. No existing saved read is
retroactively assigned a notification-time snapshot in this change.

The [hash-pinned export recipe](apk/recipes/2.26.37.71-session-refresh-preview.json)
retains eight topics / 13 files / 103,739 bytes. Exact reproduction and cached
self-diff validate retained evidence, not a cross-release comparison.

## Retained-account offline validation

**Observed locally:** the new CLI preview consumed the correlated authenticated
primary-LID reply already saved by commit `41e66e3`. It reported
`verified_bundle`, `signature_valid=true`, `pin_matches=true`, `outcome=prepared`,
a held session and **two archived states** in the discarded candidate.
`admission_evaluated=false` and `persisted=false` were explicit.
Before/after `store inspect` JSON compared equal. No connection, new request,
chat ciphertext, pin/session commit or identity notification ACK occurred.
Summary equality alone is not a byte-level live account comparison; synthetic
tests provide that stronger local no-mutation check. The historical reply
has no new notification-time provenance attached and is not replayed.
[Redacted aggregate](identity-session-preview-validation.json) contains no
account, peer or action IDs, keys or raw wire values. Private result remains in
ignored account validation storage. This validates the invokable derivation
path against an existing live response, not live automatic refresh/admission.

## Verification

854 workspace/all-feature tests passed (API 713, codec properties 4, crash
matrix 9, experimental CLI 54, API symbols 1, research tooling 73). Strict
workspace/all-target Clippy and warnings-denied workspace Rustdoc passed.
Default CLI passed 25 unit/reference tests and one API-symbol test; experimental
CLI reference was regenerated. Formatting and staged whitespace passed. Exact
bundle regeneration verified eight topics / 13 files / 103,739 bytes;
checksum-verified self-diff found eight unchanged topics and four metadata
artifacts. Repository hygiene verified 2,309 links and 115 bundles totaling
23,735,849 bytes. Initial Clippy fixture findings were fixed and full tests/Clippy
rerun after the additional missing-pin/session-pin mismatch cases. No new
network validation is claimed.

## Original refresh guards continuation

The new [original identity guard](identity-refresh-guards.md) is included in the
inspection snapshot. Known stale reads skip derivation with stale_refresh_guard.
Historical untracked reads can still exercise cryptographic preview with no
commit or claimed provenance. Automatic admission and final session/identity OCC
rechecks remain unported; no original reply is retroactively assigned a baseline.

## Attestation verification continuation

[Selected-device attestation inspection](attestation-verification.md) now closes
prefix/protobuf/type/signature inputs. It does not evaluate the native stored-
state/property/override matrix; the preview continues to refuse present
attestation, including mathematically valid input, without a candidate commit.

## Server-authorized same-key continuation

[Same-key primary refresh](same-key-refresh.md) now applies the classic subset
with an authenticated selected-user force_override_e2ee_state child. This
supersedes the earlier blanket same-key application gap; no-child, companion
and unsupported metadata admission still remain deferred.
