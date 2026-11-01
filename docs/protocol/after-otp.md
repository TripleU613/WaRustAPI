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
| `stale` (expired code) | Returns to code entry on a new step without replaying the answer; `resend-code` then asks for a new code on the same attempt (it also accepts attempts saved as blocked by `stale` before this). **Confirmed, 2.26.37.71:** `KotlinRegistrationBridge.A08` maps `stale` to result 9 (`02S.A1R`); `VerifyPhoneNumber.A5O` case 9 (+0x078e) calls `I49.A07` (removes the stored `…VerifyPhoneNumber.sms_code`/`sms_cc`/`sms_phone_number` preferences), `A16` (clears the code field) and shows an error, staying on code entry; `blocked` (case 11) instead leaves for the blocked flow. **Observed 2026-11-01:** a code entered about two hours after `status: sent` returned HTTP 200 `fail`/`stale` | The app's error text (resource `0x7f123780`) and any client-side expiry timer are not traced |
| `incorrect`, length/format errors | Preserves the recognized refusal; does not replay the answer | APK-derived wrong-code retry handling is not implemented yet |
| `device_confirm_or_second_code`, `second_code` | Preserves refusal and stops | Trace the APK selector and required follow-up; neither means registration succeeded or another SMS was dispatched |
| `consent` | Saves pending selector, intermediate login, consent IDs, URL and waits encrypted | `inspect-consent` recovers context; `submit-consent` supports explicit DOB or reviewed youth decision; no automatic acceptance |
| `challenge` | Moves to `EmailChallengeRequired` when `second_factor_methods` is absent or names `email_otp` (2.26.37.71 `launchEmailChallengeFlow`); `request-email-code` sends `/v2/code` with `method=email_otp` (`LX/0GB.A01`; the server emails the address on file), `verify-email-code` sends the emailed code to `/v2/register` and an `ok` completes registration with the returned credentials. `exist` probes `email_otp_eligible`/`wa_old_eligible` read-safely. `resume-email-challenge` (local) recovers attempts refused before this branch existed | `wa_old` (confirm on the old device) and passkey second factors stay blocked. The first implementation posted `/v2/challenge challenge_type=email_enter` and got HTTP 404 live on 2026-09-20 (that endpoint is the Gmail-OAuth email capture, `ChallengeRepository.A00`) |
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

## Wrong-code results: partial trace (2026-11-01)

**Confirmed static, 2.26.37.71:** `KotlinRegistrationBridge.A08` maps
`/v2/register` reasons to result codes (`02S` boxed integers, read from
`02S.<clinit>`): `incorrect` = 20 (`A0D`), `mismatch` = 5 (`A0j`),
`too_many_guesses` = 6 (`A0u`), `guessed_too_fast` = 7 (`A15`), `stale` = 9
(`A1R`), `blocked` = 11 (`A03`), `sms_required` = 23 (`A0G`).
`VerifyPhoneNumber.A5O` switches on that value (`packed-switch` at `067c`,
cases 0–18 only):

| Code | Target | What is established |
| --- | --- | --- |
| 9 `stale` | `078e` | forget stored code, clear input, show error, stay on code entry (implemented) |
| 5 `mismatch` | `08ca` | calls `A1v(Hts, method)`; `A1v` not yet read |
| 20 `incorrect` | default `0682` | `I49.A07` (forget stored code) first; the rest of the branch (`Ho3.A00`, `BLt`, `I1b.A01(…, -1)`, a `wa_old` method check, then a jump back) is **not** followed |

**Not established:** whether `incorrect` keeps the attempt on code entry
(likely, since it forgets only the stored code) and what `A1v` does for
`mismatch`. Until `0682`'s continuation and `A1v` are read, both stay a
preserved refusal (no replay).
