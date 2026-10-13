# Persisted registration lab

`experimental-signal-crypto` exposes
`registration::factory::StoredComparisonRequestFactory`. It generates distinct
Noise and Signal identity keys plus a signed prekey using pinned
`wacore-libsignal`, and implements the existing comparison request-factory
boundary. This is not a live-registration command or a messaging backend.

## Guarantees tested offline

- Account keys, signed-prekey material, comparison UUID/token prerequisites,
  and preparation state use the existing encrypted transaction boundary.
- Requests cannot be built before material commits. Reopening the store and
  preparing the same attempt reuses identical keys, signatures and request bytes.
- Attempt ID, generation, phone, challenge method, configuration, and account
  identity mismatches fail instead of silently regenerating keys.
- OTP and PIN inputs are used only to build requests, never saved by the factory.
- A mock HTTP flow exercises code request, repeated begin without another
  request, OTP submission, and account completion while retaining the keys.

Run without a phone, APK, credentials, or WhatsApp network access:

```sh
cargo test --locked -p whatsapp-api --features experimental-signal-crypto registration::factory
```

## Explicit lab boundary

The factory accepts only `android-2.26.37.70-registration-lab`, not the active
pinned profile. The caller supplies the phone-bound comparison token, locale,
registration ID and 24-bit signed-prekey ID through `LabRegistrationConfig`.
The token can be derived with the existing exact-APK helper; the factory does
not attest to caller-supplied token provenance. There are no invented default
ID ranges. The same config must be supplied after restart.

Signing follows the upstream Signal convention: sign the type-prefixed
33-byte signed-prekey public value, then send raw 32-byte keys with the
separate key-type field. This is an implementation assumption, **not an
observed Android registration result**. Accepted ID ranges, signing semantics,
mandatory field timing and server acceptance still need independent evidence.
No evidence ledger entry is promoted by these mock tests.

The factory performs no network I/O and is never installed by the default
client or ordinary registration CLI. The explicit experimental CLI below pairs
it with `ComparisonHttpFlow` and real HTTPS only when requested. Resetting only the registration
state does not rotate its keys: a different attempt requires a fresh store or
account generation. Never delete an existing account to retry a transient error.

## Experimental signup CLI

### Editable request profile

The small [`request.rs`](../../crates/whatsapp-api/src/registration/request.rs)
contains the JSON starter profile, not the validation machinery. Export it once:

```sh
cargo run --locked -p whatsapp-cli --features experimental-signup -- lab-signup template
```

Save the output privately as `lab/requests.json` (mode 0600); edit that JSON
without rebuilding. The template's empty token intentionally fails validation.
Replace the example phone and key IDs with controlled experiment inputs, and
derive the token for that number. Never commit the filled-in profile.

The control panel covers phone/country, language/locale, SMS/voice, client
User-Agent, device identifiers, per-endpoint extra form fields, timeouts,
response budget and diagnostics. Null identifiers are generated once and
persisted. `fields.code`, `fields.register` and `fields.security` accept plain
string values (not pre-encoded values); core identity/key/OTP/PIN/reset inputs
cannot be overridden there. Empty maps intentionally claim no additional
requirements. Add fields only with version-scoped APK evidence and real inputs;
this is not a proven working Android fingerprint or an attestation generator.

The APK's additional-parameter map has a different encoding path. Use
`fields.code_bytes`, `register_bytes`, or `security_bytes` for UTF-8 strings
or raw byte arrays (JSON integers 0–255), never pre-encoded values. Strings
normalize to bytes, so changing only this spelling preserves saved-profile
equality; adding or changing values still does not. Persisted serialization
continues to use arrays. For example a **synthetic encoding test only**,
`"code_bytes": {"client_metrics": " ~*"}` (equivalently `[32, 126, 42]`) emits
`client_metrics=%20~%2A`. This is not a valid metrics payload. The string maps
remain ordinary form text. Duplicate names across the two maps and protected
identity/OTP/PIN fields are rejected. Known text flags require `true`/`false`,
`type` requires `0`/`1`, and binary-message strings require unpadded URL-safe
Base64. None of this supplies missing genuine device or challenge data.

```sh
target/debug/whatsapp-cli lab-signup --config lab/requests.json preview
```

Preview validates locally without opening a store or contacting WhatsApp.

The feature-gated library also exposes `EncodedRegistrationRequest::encrypt_comparison`
for offline assembly of the APK's Curve25519/AES-GCM `ENC` envelope. It generates
a fresh ephemeral encryption key, independently of persisted account keys.
It is not selected by ordinary begin or the saved factory, and does not supply the
separate `H`/Authorization attestation path. See the
[APK transport trace](http-post-analysis.md) for evidence and limits.

`lab-signup --allow-live-request diagnose-encrypted-parameter` explicitly
performs one ENC-only experiment after saved HTTP 200 / bad_param. It preserves
the inner profile, attempt and account keys. A separate durable reservation
commits before I/O and prevents repetition, including after restart; previous
diagnostic reservations are not cleared. Other refusals, cooldowns and unknown
outcomes remain ineligible. This is not a full attested Android request.

The first authorized encrypted experiment returned HTTP 200 / fail / bad_param:
884-byte outer request, 58-byte response, no recognized parameter hint or wait.
Encryption alone did not improve the observed result. No automatic retry followed.

Additional explicit, separately reserved experiments are available after a
saved HTTP 200 / bad_param only:

- `diagnose-unsigned-envelope`: `ENC` plus the APK's empty-signature `H=` fallback.
- `diagnose-additional-fields --fields <private.json>`: additive EndpointFields
  JSON on that envelope. Existing/protected fields cannot be replaced.
- `diagnose-flow-fields --fields <private.json>`: additionally requires a
  `client_metrics.attempts` value matching complete saved preparation history
  plus this request. Truncated history fails closed; this counts preparations,
  not confirmed deliveries.
- `diagnose-unknown-agent --fields <private.json>`: the same checks with the
  exact APK `0re.A00` unknown-build-property header fallback. It is an explicit
  compatibility experiment, not a claim of a real Android device.

Each command still requires `--allow-live-request`, reserves its one-shot
marker before I/O, and preserves earlier markers. Field patches are private,
size-bounded and preflighted before reservation; they do not rewrite the frozen
saved profile. No command retries `format_wrong`, cooldowns or unknown outcomes.
The unknown-agent test changed the retained refusal to `format_wrong`; no code
delivery or account registration was established.

Phone and secret values are redacted; `preview --show-phone` explicitly reveals
only the phone. Unknown settings fail fast. Live diagnostics show field names,
HTTP status, recognized server reason and wait times, never body values or
arbitrary server text. Refused states now retain HTTP status and recognized
reason after restart; historical records cannot recover details already lost.
Preview also shows missing User-Agent, the selected body encoding, and missing
attestation; these are compatibility gaps, not proven refusal causes.
Older configs remain readable through defaults.
All config inputs are frozen once an attempt is prepared: editing a saved
attempt's profile fails a consistency check instead of changing its identity.
Store path, account and master-key environment-variable name remain CLI options.
TLS validation, origin and version binding are not configurable bypass switches.

### Edit requests without changing Rust

For OTP submission, account PINs and follow-up branches, see
[After the OTP](after-otp.md).

### Consent continuation

`inspect-consent` performs one explicit `/v2/exist` request on a saved consent
refusal, preserving account keys. It does not send an OTP, SMS or consent choice.
The v2 reservation captures `pending` and intermediate login; the old inspection
reservation is retained. This bounded parser-correction inspection is not a
general retry or cooldown reset.

`submit-consent --decision <private.json>` sends one `/v2/consent` request.
The decision file is separate from the frozen identity profile, like an answer;
it must have private permissions. For the APK's `pending=dob` (or captured null)
branch, supply your real DOB as `YYYY-MM-DD` in `dob` and explicitly set
`confirmed: true`. Do not supply consent ID/version/result fields in that branch.
Malformed dates, future dates and unconfirmed decisions fail before sending.

For `pending=youth_consent`, a reviewed decision instead contains
`consent_id`, `consent_version`, `consent_result: 5`, and `confirmed: true`.
Selectors must match the saved server response. Result 5 is observed in the APK's
navigation callback; this low-level lab interface does not itself display the
consent wording and must not be treated as automatic authorization to accept it.
Parent-verification, app-store age signals and unknown branches remain blocked.

Both operations require `--allow-live-request`. Reservations commit before I/O,
concurrent calls cannot double-send, and ambiguous results cannot be replayed.
Generic/SMS/voice waits are respected. The DOB/decision is not copied to the
attempt store or logs. An intermediate login stays encrypted and is reused;
an `ok` with a pending follow-up is not treated as completed registration.

New exported templates enable `preflight: true`. This runs `/v2/exist` with
the same saved keys/identifiers before `/v2/code`. Existing JSON that omits the
switch defaults to false, preserving its frozen configuration and historical
experiments. This does not silently upgrade existing accounts or clear waits.

Configure existence-only inputs in `fields.exist` / `fields.exist_bytes`.
Do not copy code-request failure reasons or counters into them: the APK uses
a separate existence-check counter. These fields have the same size, encoding,
redaction and protected-field validation as the other endpoint maps.
When existence `client_metrics` is configured, its `attempts` value is derived
from saved preparation history, not copied from another attempt. Reservations
are counted even if transport subsequently fails; this is not a delivery count.

The initial implementation deliberately uses a conservative **lab policy**,
not a claim of full APK UI equivalence: it continues only for HTTP success,
`status: fail`, `reason: incorrect`, a recommendation containing the
operator-selected SMS/voice method (or a fallback list containing it when the
recommendation is absent), no positive generic/selected-method wait,
and no indicated device/PIN/secure-verifier branch. A same-device `ok` response
is not treated as permission to request another code or as completed signup.
The configured method is never switched automatically. Unknown methods
are redacted to `other`; lists are bounded. Other responses stop with typed
diagnostics or a cooldown. Method recommendations alone do not prove that an
account is new. A live fallback-only response with zero waits now passed this
gate; the subsequent code request returned HTTP 200 / `sent`. This establishes
server-reported dispatch, not handset receipt or completed registration.

`resume-preflight` is a one-shot migration for the old local guard's saved
missing-recommendation stop. It requires the same preflight-enabled JSON,
preserves keys and attempt identity, and repeats preflight before considering
a code request. It cannot reset a server refusal, cooldown or uncertain outcome.

Both requests run under the saved Preparing state; ambiguous transport or parse
failures enter outcome-unknown instead of automatically replaying either request.
An ordinary repeated begin does not resend a pending/blocked/unknown attempt.

Use a private `request.json` (the filename is arbitrary) with `--config`.
Change `phone.country_code` / `phone.national_number`, `language`, `locale`,
`client.user_agent`, and `fields.code` / `fields.code_bytes` there.
`transport.body_mode` selects `clear_form` (backward-compatible default),
`enc`, or `enc_empty_h`. Normal begin/verification actions use that setting;
the historical named diagnostic commands retain their explicit encoding.
`enc_empty_h` does not fabricate an Android attestation signature.
Run `lab-signup --config <private-request.json> preview` before sending.

A different phone requires its own APK-derived `token` and separate attempt
storage; do not copy another number's token or failure counters. These are
private data changes, not per-number source changes. Once prepared, a saved
attempt's JSON is frozen. Never delete its store or cooldown to repeat a test.

The sole token-correction exception is `retry-corrected-token`: supply a private
JSON differing **only** in `token`, then explicitly authorize the command with
`--allow-live-request`. It requires a saved HTTP 200 / `bad_token` refusal,
rejects unchanged tokens and unrelated edits, and commits the corrected token
and one-use reservation atomically before sending. It preserves the attempt,
keys, other fields and history. No reset or automatic retry is performed.
Use the corrected JSON for subsequent status/verification commands; keep the
original privately for comparison. This path was added for the confirmed
comparison-token salt transcription error, not token guessing.

For historical refusals with no recorded HTTP status or reason,
`retry-corrected-request` uses that same private-JSON workflow to migrate the
token, User-Agent, body encoding and additional endpoint fields together.
It shares the legacy diagnostic's one-use reservation, so using either action
prevents another. All other settings and account identity remain frozen;
known refusals, cooldowns and unknown outcomes are excluded. If metrics are
provided, their preparation count must match the saved history plus this send.
This migration is not needed for new numbers configured through JSON from the
start. Keep the previous JSON privately and use the corrected file afterward.

See the [outbound request analysis](http-post-analysis.md) for evidence and gaps.

## Working recipe

The successful registration path uses:
- **body_mode**: `enc_empty_h`
- **user_agent**: `"WhatsApp/2.26.37.70 Android/unknown Device/unknown-unknown"`
- **code_bytes**:
  - `mcc`: `"000"`
  - `mnc`: `"000"`
  - `sim_mcc`: `"000"`
  - `sim_mnc`: `"000"`
  - `education_screen_displayed`: `"false"`
  - `prefer_sms_over_flash`: `"false"`
  - `feo2_query_status`: `"did_not_query"`
  - `client_metrics`: `{"attempts":1}`
- **preflight**: `false`
- **token**: derived from 2.26.37.70 material

`lab-signup template` ships this recipe with an empty token. Replace the phone,
token (using the 2.26.37.70 material), and optional registration/prekey IDs
before first use.

## Token material (2.26.37.70)

The phone-bound token is derived from the 2.26.37.70 APK using PBKDF2-HMAC-SHA1
over the package name and `about_logo.png`, then HMAC-SHA1 over the signing
certificate DER, MD5 of `classes.dex`, and the national number.

The 2.26.37.70 APK is no longer on disk; `~/Downloads/WhatsApp.apk`
is 2.26.37.71 and produces tokens that do not match the saved working token
(verified 2026-09-20 by deriving for the working account's number and comparing).
A private archive built from the unpacked 2.26.37.70 tree reproduces the working
token exactly. Keep it as `lab/whatsapp-2.26.37.70-token-material.zip`.

Commands:

```sh
whatsapp-cli token derive --country-code <cc> --national-number <digits> \
  --material lab/whatsapp-2.26.37.70-token-material.zip

whatsapp-cli token verify --config lab/accounts/<name>/request.json \
  --material lab/whatsapp-2.26.37.70-token-material.zip
```

The first prints the token (secret); the second prints only match/mismatch.

## Onboarding a number in two commands

With a working token and material archive in place:

```sh
whatsapp-cli accounts new --name a --country-code 1 --national-number 5551234567 \
  --material lab/whatsapp-2.26.37.70-token-material.zip

export WHATSAPP_API_MASTER_KEY_A=$(cat lab/accounts/a/master.key)

whatsapp-cli --profile a lab-signup --allow-live-request begin --method sms

read -rs CODE; printf '%s\n' "$CODE" | whatsapp-cli --profile a lab-signup --allow-live-request verify-code
```

`accounts new` creates:
- `lab/accounts/<name>/master.key` (mode 0600): encryption key
- `lab/accounts/<name>/request.json` (mode 0600): working recipe + derived token
- `lab/accounts/<name>/signup.redb`: the encrypted store (since 2026-10-13)
- `lab/profiles/<name>.json`: profile linking them together

It refuses a number another saved profile already configures. `lab-signup begin`
no longer creates a missing store: a profile made before `accounts new` created
stores needs `begin --create-store` once. Without it, a forgotten `--profile` or
a mistyped path fails instead of creating a fresh identity
([registration safety audit](registration-safety-audit.md)).

The profile sets `key_env` to `WHATSAPP_API_MASTER_KEY_<NAME>` (in uppercase).

For a second number, use a different name; never share a store or key variable
between accounts.

## Live record (2026-09-20)

Two new US numbers were prepared from the old template defaults (`clear_form`,
missing User-Agent, missing code fields) with tokens derived from 2.26.37.71.

**Result**: HTTP 200 `status=fail reason=bad_param invalid_parameter=null` on
both `/v2/exist` (preflight) and `/v2/code`.

**Root cause**: wrong-build token and missing recipe — our inputs, not an
established requirement about the numbers. Both attempts are frozen as
`ServerRefused/bad_param`; no correction command accepts `bad_param` with a
changed token. A new attempt for those numbers needs a fresh store and explicit
operator authorization. Also: an `UnsupportedStep` preflight block on `bad_param`
has no resume path.

Build with `cargo build --locked -p whatsapp-cli --features experimental-signup`.
`lab-signup` is absent from the default build. It performs one operator-selected
action per invocation and never automatically submits an OTP or resets a PIN.
Mutations require `--allow-live-request`; status is local-only.

Keep the JSON `LabRegistrationConfig` in `lab/signup.json` with mode 0600,
outside Git. Its keys are `phone` (object with `country_code` and
`national_number`), `language`, `locale`, `token`, `registration_id`, and
`signed_pre_key_id`. Derive the phone-bound token from the exact comparison
APK with `xtask apk-registration-token`; do not paste it into issues or chat.
The config must stay unchanged when resuming. Supply the encrypted store's
master key through `WHATSAPP_API_MASTER_KEY`, not command-line arguments.

After independently checking the inputs and authorizing use of the test number:

```sh
target/debug/whatsapp-cli --store lab/signup.redb lab-signup \
  --config lab/signup.json --allow-live-request begin --method sms
target/debug/whatsapp-cli --store lab/signup.redb lab-signup \
  --config lab/signup.json status
```

For `verify-code` or `verify-pin`, the answer must arrive on redirected stdin;
terminal input is rejected to avoid echoing secrets. Use a local hidden-input
prompt and pipe the result, not a literal secret in shell history. Inspect the
reported state before any further action. Cooldown, blocked, and unknown-outcome
states must not be worked around by resetting stores or repeatedly requesting codes.
No live signup has been performed or shown to work by adding this command.

### Email second factor after the OTP

When `/v2/register` answers `status=fail reason=challenge`, the number needs a
second factor. The refusal carries `second_factor_methods`,
`is_device_trusted`, `email_otp_wait`, `email_otp_length` and the masked
`email` on file. In 2.26.37.71 the email one-time code is a delivery
*method* of the ordinary code request: `LX/0GB.A01` maps the method enum to
`sms`, `email_otp`, `flash`, `voice`; the dynamic method sheet lists
`sms`/`voice`/`flash`/`email_otp`/`wa_old`; `KotlinRegistrationBridge.A09`
(`/v2/code`) reads `email_otp_wait` back; and the emailed code is verified by
the same `/v2/register` request as an SMS code (`KotlinRegistrationBridge.A0D`,
reasons `mismatch`/`incorrect`/`guess_wait`). `KotlinRegistrationBridge.A0B`
(`/v2/exist`, performSameDeviceCheck) reports `email_otp_eligible`,
`wa_old_eligible` and the masked `email`, which is how the client decides.

An earlier implementation posted `/v2/challenge` with
`challenge_type=email_enter`; live on 2026-09-20 the server answered HTTP 404
(twice from the operator's session, once with diagnostics). That endpoint and
`challenge_type` belong to `ChallengeRepository.A00` (`email`, `oauth_token`,
`code`: the Gmail-OAuth email capture flow), not to the registration second
factor. Here:

```sh
target/debug/whatsapp-cli --profile b lab-signup status
target/debug/whatsapp-cli --profile b lab-signup --allow-live-request exist   # read-safe eligibility probe
target/debug/whatsapp-cli --profile b lab-signup resume-email-challenge        # only for a saved `challenge` refusal
target/debug/whatsapp-cli --profile b lab-signup --allow-live-request request-email-code
printf '%s' "$code" | target/debug/whatsapp-cli --profile b lab-signup --allow-live-request verify-email-code
```

No email address travels in the request: the server emails the address on
file, so the code only helps when that mailbox is reachable. Live on
2026-09-20 the second lab number's `exist` probe answered
`status=fail reason=incorrect` with `email_otp_eligible=false`,
`wa_old_eligible=false`, no email on file and
`fallback_methods=[other, wa_old, email_otp, other, sms, voice]`; the two
`other` entries are methods this client does not name. With neither second
factor eligible, that number cannot complete registration from this client
until the server offers one, or a different number is used. The eligibility
flags arrive as non-boolean JSON, so the parser accepts `true`/`1`/`"true"`,
and a parse failure now reports the reply's field names and value types
(never values). `wa_old` and
passkey second factors are not implemented and leave the attempt blocked.

### Diagnostic eligibility and status redaction (2026-10-13)

The one-shot code-request diagnostics, `resume-preflight` and
`retry-corrected-token` now also require the saved refusal to come from a code
request (last transition `preparing -> blocked`). An OTP submission refused with
`bad_param` saves the same shape, and answering it with a new code request
would ask for another SMS. `status --json` now redacts the consent context's
login, pending selector and parent consent URL as well as the phone number.
[Registration safety audit](registration-safety-audit.md).

### One-shot legacy diagnostic recovery

`lab-signup --allow-live-request diagnose-legacy-refusal` is an explicit lab-only
action for historical `ServerRefused` records with no saved HTTP status, code
or reason. It is not called by `begin`. Use only for an operator-authorized
number after deciding to send one more request; it may trigger SMS delivery.
It retains the same attempt, keys, configuration and transition history.
A durable account-local latch and preparation state commit before sending.
Known refusals, cooldowns, unknown outcomes and repeated diagnostics are rejected.
No existing store or latch should be deleted to repeat it. A transport failure
that may have reached the server leaves the outcome unknown; do not resend. A
request that provably never left (it could not be built, prepared or connected:
`RequestNotSent`) restores the saved refusal and the saved request material
together and releases the one-shot latch, so the same action can be run again
once the network is back (registration safety audit review M2, 2026-10-13).
Reqwest protocol-level
retries are also explicitly disabled, in addition to application-level
automatic retries.

`lab-signup --allow-live-request diagnose-bad-parameter` is a separate,
one-shot operator action restricted to an existing HTTP 200 / `bad_param`
refusal. It repeats the unchanged saved payload and keys to collect improved
diagnostics (including a recognized `param` hint if present), not to test a
claimed payload fix. A separate durable latch prevents a second use. Other
known refusals, cooldowns and unknown outcomes are ineligible. It is never
called by ordinary `begin`; do not delete its latch or store to repeat it.

## Still needed for the first real message

Validate the exact APK/profile and remaining registration assumptions, wire an
operator-controlled onboarding command, then test one authorized registration.
Authenticated login, Signal session/prekey integration, and a transmission path
remain necessary after registration. `NoBackend` still handles messaging.

## Two-step verification, and resolving an unknown outcome (2026-09-24)

Setting the PIN is `LX/0lo->A06` (`TwoFactorXmppMethods/sendSetTwoFactorAuth`),
**confirmed**: `<iq xmlns="urn:xmpp:whatsapp:account" type="set"><2fa>` with
`<code>` and/or `<email>` children, each present only when supplied. A code alone
changes the PIN, an address alone changes the recovery email, and an **empty
code turns two-step verification off**. Emitting an unwanted empty `<email/>`
alongside a PIN would clear the operator's recovery address as a side effect, so
the builder never does.

There is **no server-side read**. `LX/0lo`'s other methods show the code kept in
local preferences (`two_factor_auth_code`, `two_factor_auth_using_encryption`),
so a client cannot ask the server what the current PIN is — only set it.

**On resolving an unknown outcome.** The first set was transmitted and its reply
never arrived, because a decryption failure was ending the session and IQ replies
are not redelivered. The rule this project follows — an uncertain outcome is not
re-sent — exists to stop a second *effect*, and the thing it guards against is
duplication. Setting the same PIN to the same value has no second effect: the end
state is identical whether the first attempt landed or not. So the resolution was
to issue the identical change again, which is idempotence rather than a replay,
and it answered `result`.

That distinction matters and is worth keeping: *re-sending is unsafe when the
operation is not idempotent*. A message, a group create, a payment — those must
never be repeated on an unknown outcome. A set-to-a-known-value may be. The
original uncertain action is left recorded as `Sent` rather than rewritten,
because that is what happened to it.

## Saved account inventory (2026-10-07)

**Observed locally**, updated audit, not an APK finding: 19 saved profiles
reference 13 distinct configured numbers across 16 existing configurations.
All 16 existing stores opened with their saved keys: three account records have
completed registration and authenticated-login history; thirteen are
unregistered. Only the recently authenticated existing client is established
for continuing messaging tests. Historical registration is not evidence of
current usability or ban status, and a failed saved login alone does not
identify a ban. Three legacy profiles reference absent stores/configurations;
they establish no additional usable accounts. Deduplication was performed
privately without emitting phone numbers or credentials. This supersedes the
earlier partial audit with four locked/unknown profiles.

**Unknown:** detailed signup outcomes for the unregistered records and long-term
account reliability remain unestablished. A status placeholder does not prove
registration was never attempted. No live registration, identity reset, retry
or message transmission was performed for this count. More numbers are
unnecessary for static research/current single-account checks; a second working
account would enable controlled two-client end-to-end tests.
