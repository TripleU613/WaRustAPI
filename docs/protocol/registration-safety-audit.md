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

- **Requests that never left were recorded as uncertain, stranding the
  attempt.** Every flow error became `outcome_unknown`, which the HTTPS flow
  cannot reconcile, so the only way on was a new store: identity rotation. Two
  kinds of failure provably send nothing: a request that cannot be built (a
  config edited since the attempt froze it, a resend method the frozen config
  does not use, the metrics history bound) and a connection that fails before
  any byte is written. These now raise `Error::RequestNotSent` (wrapping the
  cause), and the attempt returns to its state before the reservation through a
  legal transition: a fresh `begin` becomes an already elapsed cooldown that the
  next `begin` resumes with the same attempt and keys; a cooldown resume returns
  to its cooldown; a resend returns to its pending challenge and original
  method; an OTP, PIN, email-code or consent request returns to its prior state;
  a one-shot diagnostic or consent latch taken by the reservation is released.
  Every other failure (timeout, a response that cannot be read, a reset after
  writing) may have reached the server and still records `outcome_unknown`.
  **Inferred:** that reqwest's `is_connect()` (DNS, TCP, TLS handshake, connect
  timeout) and `is_builder()` errors occur before the request is written; this
  follows the library's connector design, not a packet capture. A preflight sent
  before a code request that then fails to build has been sent; it is the
  read-safe `/v2/exist` probe. Tests:
  `unsent_requests_restore_the_attempt_instead_of_an_unknown_outcome`,
  `build_failures_are_classified_as_not_sent_once`.

- **A new or mistyped store path created a fresh identity.** `lab-signup begin`
  opened its store with `create: true`, so a forgotten `--profile` (defaults
  `protocol.redb`, `WHATSAPP_API_MASTER_KEY`) or a mistyped path generated new
  keys and IDs for a number whose real store might be cooling down or holding
  an uncertain outcome. `accounts new` now creates the profile's store, and
  `begin` creates one only with an explicit `--create-store`; otherwise a
  missing store is refused. `accounts new` also refuses a number another saved
  profile's request JSON configures (one number, one store), before writing
  anything. Tests: `begin_does_not_create_a_store_implicitly`,
  `accounts_new_refuses_a_number_another_profile_configures`. Store creation by
  `accounts new` needs real token material and is not covered by an offline test.
- **`begin` replaced another attempt; `reset` deleted a lab attempt.** `begin`
  silently overwrote a saved attempt for another number or profile in the same
  account generation, losing its waits, uncertain outcome or pending challenge
  and stranding its key material; the ordinary `registration reset` deleted a
  blocked lab attempt. `begin` now refuses (an attempt from an older generation
  may still be replaced), and `reset` refuses an attempt created under another
  profile. Test: `begin_and_reset_never_replace_another_attempt`.
- **`accounts status` hid the registration state.** It printed
  `registration_state: "none"` without reading the attempt, and
  `actions_queued` counted every recorded action. It now reads the saved state
  (`registration::current_state_name`) and counts only queued actions.
- **The `exist` probe ran in any state.** It has no reservation; it is now
  refused while a request is in flight or its outcome unknown, once registered,
  and during an unexpired cooldown. Repeats in other states remain possible and
  its waits are printed, not saved. Test:
  `exist_probe_is_refused_mid_request_registered_or_cooling_down`.

## Confirmed open

- **A refusal that stays blocked does not keep its SMS/voice wait.** A
  `/v2/code` refusal without `retry_after` but with `sms_wait`/`voice_wait`
  stays `Blocked{ServerRefused}`, which has no wait field; the one-shot
  diagnostics and `retry-corrected-token` then do not enforce it. Turning such
  a refusal into a cooldown would make it resendable through `begin`, so the fix
  is a wait field on `ServerRefused` (a library API and saved-state change),
  not yet made. The diagnostics' refusal output already prints the waits.

- **`code_bytes.client_metrics` is a fixed value.** The `/v2/code` request
  sends the frozen config's `{"attempts":1}` on every resend, cooldown resume,
  email-code request and diagnostic; the `/v2/exist` probe already derives
  `attempts` from saved history. Deriving it for code requests changes live
  request content, and the APK caller values of `I4D.A0H` are not inferred
  ([HTTP POST analysis](http-post-analysis.md)), so it stays a documented
  candidate, not changed here.

## Plausible, not verified

Lower items from the same audit: any refusal carrying `retry_after`
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
