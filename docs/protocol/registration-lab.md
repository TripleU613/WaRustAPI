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

See the [outbound request analysis](http-post-analysis.md) for evidence and gaps.

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

### One-shot legacy diagnostic recovery

`lab-signup --allow-live-request diagnose-legacy-refusal` is an explicit lab-only
action for historical `ServerRefused` records with no saved HTTP status, code
or reason. It is not called by `begin`. Use only for an operator-authorized
number after deciding to send one more request; it may trigger SMS delivery.
It retains the same attempt, keys, configuration and transition history.
A durable account-local latch and preparation state commit before sending.
Known refusals, cooldowns, unknown outcomes and repeated diagnostics are rejected.
No existing store or latch should be deleted to repeat it. On transport failure,
the outcome remains unknown; do not resend. Reqwest protocol-level retries are
also explicitly disabled, in addition to application-level automatic retries.

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
