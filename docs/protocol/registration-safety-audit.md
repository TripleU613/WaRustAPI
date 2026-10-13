# Registration request safety audit (2026-10-13)

A read-only code audit of the registration (onboarding) HTTPS path:
`registration/` (attempt state, reservations, request factory, wire client,
response parsing) and the `lab-signup` CLI. It complements the
[transmission replay-safety audit](replay-safety-audit.md), which covered the
messaging socket only. Nothing was sent; no `lab/` state exists in the audit
container. Line references are to the tree at `e15c863`.

Every finding below is a claim about our tooling, not about WhatsApp. Each is
labelled **fixed** (verified in code, corrected and tested), **confirmed open**
(verified, not yet changed) or **reported** (from the audit, not yet
re-verified). Nothing here changes a request's wire content.

## Fixed

- **`lab-signup status --json` printed consent secrets.** Only `phone` was
  redacted; a consent-blocked attempt serialised `RegistrationConsent`'s
  intermediate `registration_login`, opaque `pending` selector and
  `parent_consent_url`. The JSON is now redacted for those keys at any depth
  (presence stays visible, `null` stays `null`). The non-JSON output already
  used the redacting `Debug`. Test: `status_json_redacts_consent_secrets_at_any_depth`.
- **Code-request diagnostics did not check which request was refused.** The
  `bad_param` diagnostics and `retry-corrected-token` accepted any saved
  `Blocked{ServerRefused{code: None, http_status: 200, reason}}`. An OTP
  submission, security, consent or email-code refusal saves the same shape, so
  after an OTP submission refused with `bad_param`, `diagnose-bad-parameter`
  would have sent a new preflight and `/v2/code`, asking for another SMS. They
  now also require the refusal to have come from a code request: the last
  transition is `preparing -> blocked` (`RegistrationAttempt::refused_at_code_request`).
  Test: `code_request_actions_refuse_a_refusal_from_another_request`.
- **`resume-email-challenge` could be repeated without bound.** Its local
  recovery moved a lost email-code request (`outcome_unknown` reached from
  `email_challenge_required`) back to `email_challenge_required` with no wait,
  so request, lose, resume and request again could repeat, re-sending a
  request whose outcome was uncertain; a `challenge` refusal of the email
  request itself looped the same way. Each recovery kind (a `challenge`
  refusal, a lost email-code request) is now allowed once per attempt, recorded
  in the same transaction. Two separate latches, so a challenge recovery does
  not strand a later lost request. Test: `lost_email_code_requests_resume_only_once`.
- **`email_otp_wait` on a `challenge` refusal was ignored.** The email handle
  kept only `retry_after`. It now keeps the longer of `retry_after` and
  `email_otp_wait`. That the refusal's `email_otp_wait` is the wait before an
  email code may be requested is **inferred** (the `/v2/code` answer carries the
  same field); taking the longer value can only delay a request.
- **Shorter waits replaced longer ones.** On `sent`, the handle kept
  `sms_wait.or(retry_after)` (or `voice_wait.or(retry_after)` for voice), so a
  present method wait replaced a longer `retry_after`. It now keeps the longer.
  A refusal that sets a cooldown now also keeps the longest SMS or voice wait it
  carries. (The audit's claim that `voice_wait` is never kept was wrong for voice
  attempts.) Test: `server_waits_keep_the_longest_value`.

## Confirmed open

- **A refusal that stays blocked does not keep its SMS/voice wait.** A
  `/v2/code` refusal without `retry_after` but with `sms_wait`/`voice_wait`
  stays `Blocked{ServerRefused}`, which has no wait field; the one-shot
  diagnostics and `retry-corrected-token` then do not enforce it. Turning such
  a refusal into a cooldown would make it resendable through `begin`, so the fix
  is a wait field on `ServerRefused` (a library API and saved-state change),
  not yet made. The diagnostics' refusal output already prints the waits.

## Reported, under verification

1. A new or mistyped store path (`begin` opens with `create: true`) generates
   fresh keys and IDs for a number whose real store may be cooling down, holding
   an uncertain outcome or a pending challenge; `accounts new` does not check
   whether the number already has a profile.
2. Failures before anything is sent (config mismatch after a JSON edit, a method
   switch on resend, long history with metrics, DNS/TCP/TLS connect failures) are
   saved as `outcome_unknown`, which has no way out for the HTTP flow.
3. The `exist` probe has no reservation or state check and does not save its
   waits.
4. `client_metrics.attempts` is copied from the frozen config onto every resend
   rather than derived from saved history.
5. The ordinary `registration begin`/`reset` can overwrite or delete a lab
   attempt record.
6. `accounts status` prints `registration_state: "none"` without reading the
   attempt.

Plausible lower items from the same audit: any refusal carrying `retry_after`
becomes a cooldown that a later `begin` resumes with a new code request
(`retry_after: 0` immediately); strict parsing turns some definite answers
(negative waits, uppercase reason tokens, non-JSON error bodies, an HTTP
`Retry-After` header) into `outcome_unknown`; an `ok`/`attached` code response
discards its `login`; a save failure after a server `ok` loses the login; one
wrong OTP is terminal with no lab path out; diagnostics print server JSON key
names; store files use the default umask (contents encrypted); the CLI suggests
`request-email-code --email`, which is always rejected.

## Checked and sound (from the audit)

Reservation is committed with immediate durability before every send except
`exist`; a crash after the write leaves `preparing`/`verifying`, which every
command refuses; a per-process store lock prevents concurrent sends. The HTTPS
client never retries, follows no redirects, uses no proxy, pins the origin,
keeps TLS verification on with no switch to disable it, bounds body size and
time. Timeouts, transport errors and unparseable bodies become `outcome_unknown`.
`retry-corrected-token` requires HTTP 200 `bad_token`, a changed token and no
other edit, once per store; `retry-corrected-request` shares the legacy
diagnostic's marker. Keys within a store are never regenerated. OTPs and PINs
come only from redirected stdin, are wiped after use and checked before any
reservation. Not audited: `ban-appeal` and `token_material` parsing.
