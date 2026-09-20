# After the OTP

Use the same private JSON, store, account and master key as the successful code
request. `lab-signup --config <private.json> --allow-live-request verify-code`
reads the answer from redirected stdin; do not put it in command arguments,
chat, JSON or logs. Account PINs use `verify-pin` and the same input mechanism.
No automatic resend, reset or account recovery is performed.

## Current branch coverage

| Result of submitting the OTP | What the client does | What remains |
| --- | --- | --- |
| HTTP success / `ok` | Atomically saves registration and returned credentials in the encrypted store | Establish the messaging transport/session; registration is not a sent message |
| HTTP success / `fail` / `security_code` | Persists a PIN challenge; accepts `verify-pin` after the generic wait | User supplies the existing account PIN locally; no reset |
| PIN submission / `ok` | Saves returned credentials with `/v2/security` provenance | Same messaging-session work as above |
| `incorrect`, `stale`, length/format errors | Preserves the recognized refusal; does not replay the answer | APK-derived retry/expiry handling is not implemented yet |
| `device_confirm_or_second_code`, `second_code` | Preserves refusal and stops | Trace the APK selector and required follow-up; neither means registration succeeded or another SMS was dispatched |
| `consent` | Saves pending selector, intermediate login, consent IDs, URL and waits encrypted | `inspect-consent` recovers context; `submit-consent` supports explicit DOB or reviewed youth decision; no automatic acceptance |
| `challenge` | Moves to `EmailChallengeRequired` when `second_factor_methods` is absent or names `email_otp` (2.26.37.71 `launchEmailChallengeFlow`); `request-email-code` sends `/v2/challenge` `challenge_type=email_enter`, `verify-email-code` sends `email_verify` with the code and an `ok` completes registration with the returned credentials. `resume-email-challenge` (local) recovers attempts refused before this branch existed | `wa_old` (confirm on the old device) and passkey second factors stay blocked; the challenge request reuses the register identity fields plus `fields.challenge` extras, live-checked 2026-09-20 on the second lab number |
| Guess limits, blocking or other refusal | Stops; a generic `retry_after` persists a cooldown | No guess loops, identity replacement or automatic recovery |
| Transport/JSON failure after reservation | Persists outcome-unknown | Reconciliation is required before repeating a mutation |
| Unknown status/reason | Stops without exposing arbitrary response text | Inspect the new branch; do not infer success |

Except for the typed consent branch, the generic cooldown representation currently loses the refusal reason and does
not distinguish code-delivery from answer-retry context. SMS/voice waits on a
blocked post-OTP branch are visible in redacted live diagnostics but are not
stored as separate deadlines. Those branches cannot currently be resumed;
retaining their complete typed context is required before implementing them.
Historical consent refusals saved before this patch lack the discarded metadata;
the patch does not recover it. See the [APK consent trace](http-post-analysis.md).

## Local input checks

The service, not only the CLI, rejects empty/non-numeric/oversized answers and
OTP/PIN type mismatches before reserving a submission. An expired challenge
cannot be submitted. Invalid local input leaves the current challenge intact.
PIN challenge waits are enforced without consuming the challenge. The delivery
wait attached to an OTP is **not** treated as a prohibition on entering it.

Offline tests cover OTP-to-PIN-to-completion with encrypted credentials, wrong
answer types, malformed input, PIN waits, refusal families and malformed
responses, stale handles and no replay after reopening the service. These tests
are not live verification of the corresponding server branches.

## After registration

The next milestone is an authenticated messaging connection using the saved
account keys and returned credentials, followed by a first-message send and
server acknowledgement. Neither a registration `ok` nor saved credentials prove
that this connection or message exchange works. See [current status](../../STATUS.md)
and [protocol unknowns](unknowns.md).
