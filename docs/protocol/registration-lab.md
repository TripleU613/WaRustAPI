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

## Still needed for the first real message

Validate the exact APK/profile and remaining registration assumptions, wire an
operator-controlled onboarding command, then test one authorized registration.
Authenticated login, Signal session/prekey integration, and a transmission path
remain necessary after registration. `NoBackend` still handles messaging.
