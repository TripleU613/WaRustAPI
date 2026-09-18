# Registration

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 7. Registration is its own durable state machine

Treat registration as a potentially branching protocol, not a universal two-call `request_otp` / `verify_otp` flow.

| State | Meaning and allowed behavior |
| --- | --- |
| `Unregistered` | No established account credentials |
| `Preparing` | Create and persist the exact prerequisites required by this profile |
| `ChallengePending` | Verification requested; record method, expiry, attempt identity and retry deadline |
| `NeedsUserAction` | A supported additional step is required, such as a PIN or another server-directed verification action |
| `Verifying` | A submitted answer is in progress; prevent concurrent conflicting attempts |
| `RegistrationOutcomeUnknown` | Request outcome was lost; reconcile using an observed safe flow before repeating mutations |
| `Registered` | Server-issued state and required identity material have been durably committed |
| `CoolingDown` | Observe the server's retry timing |
| `Blocked` | This profile cannot complete the required step, or the server refuses the flow |

Generate identity/prekey/device material **before whichever request first requires it**, and persist it before transmitting that request. Do not retain the original diagram's assumption that key generation necessarily follows OTP verification.

A challenge handle is opaque and bound to the account, phone number, protocol profile, registration attempt and generation. An old challenge cannot be answered for a new attempt. Support sequential challenge steps without exposing reusable secrets through `Debug` or ordinary serialization.

Required behavior:

- Validate and normalize the phone number with explicit country information; preserve the protocol's required representation.
- Separate an account registration ID, local account ID, wire message ID, device ID and challenge ID.
- Keep Noise static keys and Signal identity keys as distinct typed values.
- Persist only credentials actually observed. Do not invent a bearer token, cookie, password or fingerprint field because another API uses it.
- Honor expiry and retry constraints; do not loop through OTP requests or verification attempts.
- Do not log or durably store OTPs or user-entered PINs as ordinary state.
- Return typed required actions or a specific unsupported-registration error for unimplemented server requirements.
- Never re-register automatically to repair an expired connection, a malformed stanza, or an unknown login failure.
- Registration, logout/revocation, disconnect and local deletion are different operations.

Use controlled test numbers for live experiments. The acceptance test must show that registration succeeds on the intended supported path without importing another client's completed session. Owning a number does not imply that the server will accept a custom client.
