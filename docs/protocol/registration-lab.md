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
client or CLI. Pairing it with `ComparisonHttpFlow` and a real HTTPS transport
would be a separate, deliberate live action. Resetting only the registration
state does not rotate its keys: a different attempt requires a fresh store or
account generation. Never delete an existing account to retry a transient error.

## Still needed for the first real message

Validate the exact APK/profile and remaining registration assumptions, wire an
operator-controlled onboarding command, then test one authorized registration.
Authenticated login, Signal session/prekey integration, and a transmission path
remain necessary after registration. `NoBackend` still handles messaging.
