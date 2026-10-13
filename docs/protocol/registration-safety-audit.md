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
  cause), and the attempt returns to its state before the reservation (since
  the second review below, as the exact pre-reservation copy, history
  included): a fresh `begin` returns to `unregistered` (at first an already
  elapsed cooldown), which the next `begin` resumes with the same attempt and
  keys; a cooldown resume returns to its cooldown; a resend returns to its
  pending challenge and original method; an OTP, PIN, email-code or consent
  request returns to its prior state; a consent latch is released, and a
  code-request one-shot's latch with its restored material (review M2).
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

## Review corrections (2026-10-13)

A second read-only review of the fixes above (commits `01af56a`, `cb33e2b`,
`2624949`, `4ab7c2c`) found no path where a code, register, security, consent
or email-code request that may have reached the server is restored as not
sent, and no released latch that allows a second code send. It did find these,
now corrected:

- **M1. A sent `/v2/exist` preflight was restored as "not sent".** `begin`
  sent the preflight, then built and sent `/v2/code`; a build or connect
  failure of the second request returned `RequestNotSent` for the whole step,
  so the attempt was restored (and a `resume-preflight` latch released) though
  the server had seen the preflight. A reviewer probe repeated it three times:
  three preflights left, and the preparation counter that feeds the next
  preflight's `client_metrics.attempts` rose 1→2→3 with no code request sent.
  `ComparisonHttpFlow::begin` now builds both requests before sending either,
  and a not-sent failure after a sent preflight is recorded as an unknown
  outcome. Test: `a_failure_after_a_sent_preflight_is_not_reported_as_not_sent`.
- **M2. A not-sent correction released a latch nobody could use.** The
  correction actions (`retry-corrected-token`, `retry-corrected-request`) save
  the corrected material inside their reservation. The not-sent restore put
  the refusal back and released the latch, but kept the material, so a second
  run refused with "corrected token must differ from the saved token". The
  one-shot reservations (diagnostics, corrections, `resume-preflight`) now read
  the saved material record in the reservation transaction before `prepare`
  runs (`prepared_snapshot`, held in memory only) and a not-sent restore writes
  it back in the same transaction as the refusal (`restore_prepared`), then
  releases the latch. A flow that cannot snapshot keeps its latch spent. A crash
  while the request is in flight still leaves `preparing`, refused by every
  command, as before. A request that may have reached the server keeps the
  corrected material and the latch spent. Test:
  `a_not_sent_correction_is_undone_and_can_be_sent_later` (two not-sent runs
  reach the transport; after a `bad_token` answer to a sent correction, a third
  run is refused as already reserved).
- **M3. The email lost-request recovery could re-send an uncertain request.**
  It accepted any `outcome_unknown` reached from `email_challenge_required`,
  including a timeout or reset after the request was written. An email-code
  request whose answer arrived but could not be read (a non-JSON body, an
  oversized body) now records a marker; only with that marker can
  `resume-email-challenge` take the lost-request path. Any other unknown
  outcome is never re-sent. Test:
  `an_email_code_request_lost_after_writing_is_not_resumed`.
- **M4. The store-creation guard only checked that the store file existed.**
  `registration status`/`reset`/`reconcile` and `store init` open their store
  with `create: true`, so one run without `--profile` left an empty store at
  the default path that a later `lab-signup begin` would fill with a fresh
  identity; `--profile a --config b.json` put number B's attempt in A's store.
  A store now records the number it was created for (`store_owner` /
  `claim_store`, written by `accounts new` and `begin --create-store`). `begin`
  on a store with no attempt requires that record to name the config's number;
  an unclaimed store needs `--create-store`, another number's is refused.
  Tests: `begin_requires_a_store_created_for_this_number`,
  `only_an_explicit_begin_creates_a_store` (a pure guard test; the earlier test
  ran the real HTTPS transport with live requests allowed). The cause is
  removed too: `registration status`, `reconcile` and `reset` no longer create a
  missing store (only `registration begin` and `store init` do). Test:
  `only_registration_begin_creates_a_store`.
- **L1. The `accounts new` duplicate check could miss a number.** One malformed
  profile made the whole check pass, numbers were compared as raw strings
  (`555-123-4567` against `5551234567`), the raw number was written to the
  config, and configs used only with `--config` were never seen. Each profile is
  now read on its own, numbers are compared canonically, the canonical number is
  written, and sibling account directories' `request.json` files are checked.
  Test: `accounts_new_refuses_a_number_another_profile_configures`.
- **L2 (display). A restored first `begin` read as a server cooldown.** It was
  saved as `CoolingDown{until: now}`, shown as `cooling_down` with history
  `preparing -> cooling_down`, indistinguishable from a server wait. It now
  returns to `unregistered`, which `begin` resumes with the same attempt and
  keys. (First done through a general `preparing -> unregistered` edge; the
  second review's L1 showed any flow could use that edge, so it was replaced
  by the exact restore below.) Attempts already saved as an elapsed cooldown,
  or as `unregistered` with history, still resume. Test:
  `unsent_requests_restore_the_attempt_instead_of_an_unknown_outcome`.
- **L3. Attempts from an older pinned version could not be cleared.** The
  ordinary profile id is `android-primary/<version_name>`; after the pin
  changed, a previous version's attempt could be neither replaced (`begin`
  refuses) nor reset (`reset` refused another profile's attempt) in the same
  generation. `reset` now also clears another profile's attempt when it is
  blocked with `ProfileIncomplete`, the local answer of a flow that sends
  nothing (`UnsupportedFlow`), so it holds no server state. Any other
  attempt from another profile, including every lab refusal, is still refused;
  `begin` still never replaces one. Test:
  `a_local_block_from_an_older_version_can_be_reset`.
- **A refusal that stayed blocked lost its SMS/voice/email wait** (first
  audit, confirmed open until now; also review M3's speculative case). A refusal
  without `retry_after` but with `sms_wait`, `voice_wait` or `email_otp_wait`
  stayed `Blocked{ServerRefused}` with no wait, so the one-shot diagnostics and
  corrections did not enforce it, and `resume-email-challenge` resumed a
  `challenge` refusal of the email request with no wait. Turning it into a
  cooldown would have made it resendable through `begin`, so `ServerRefused`
  gained `not_before`: the longest of those waits, added to the time of the
  answer (serde default, omitted when absent, so older records and wait-less
  refusals keep their exact JSON). The one-shots refuse with `RateLimited`,
  before reserving or sending, until it passes; `resume-email-challenge` copies
  it into the handle's `retry_after`, which `request-email-code` enforces; the
  `exist` probe refuses during it. That the longest wait is the binding one is
  our conservative choice; it only delays. Tests:
  `one_shots_wait_for_the_refusal_method_wait`,
  `a_blocked_refusal_keeps_its_method_wait`,
  `exist_probe_is_refused_mid_request_registered_or_cooling_down`.
- **L4. The lab transport's own failures were recorded as unknown.** Adding
  configured `/v2/code` fields or encrypting the body (`DiagnosticTransport`)
  fails before anything is written; it now returns `RequestNotSent`. Test:
  `diagnostic_preparation_failures_are_not_sent`.
- **Nits.** A successful email-code answer keeps the longer of
  `email_otp_wait` and `retry_after`, as the refusal path does; the lab guide's
  "on transport failure, the outcome remains unknown" now describes
  `RequestNotSent`; the `RequestNotSent` doc states the preflight caveat.

## Second review (2026-10-13)

A read-only review of the next six commits (`72130d6` to `cf3f116`) found no
path where a request that may have reached the server is restored as not sent,
no latch released after a request may have left, and no implicit fresh
identity. Its findings, all confirmed by code trace (it could not build probes:
the container's disk was full), and what was done:

- **M1. An unreadable email-code answer under HTTP 2xx counted as
  "answered".** The marker that lets `resume-email-challenge` take the
  lost-request path was set for any unreadable or oversized answer, including
  HTTP 200 with a body strict parsing rejects (a negative wait, an uppercase
  token); that request may have sent an email and set a wait, and the resume
  then allowed it again with none. `ComparisonHttpFlow::send` now classifies an
  unreadable body under a definite client-error status (4xx except 408 and
  429, narrowed by the third review below) as `registration_refusal_body`, and
  only that sets the marker (the observed case was HTTP 404 with a non-JSON
  body). Tests: `lost_email_code_requests_resume_only_once` (now real
  404 answers), `an_unreadable_success_answer_to_an_email_request_is_not_resumed`.
- **L1, L4 and the open history item: a not-sent restore now writes back the
  exact pre-reservation attempt** (state, method, history), with the material
  and latch undo, in one transaction. The general `preparing -> unregistered`
  edge is gone, so a flow returning `Unregistered` is again an illegal
  transition (L1). A request that never left adds no history: it no longer
  counts in `code_preparation_count`, which feeds the preflight's
  `client_metrics.attempts` and the lab's `attempts == count + 1` check, so a
  rerun after a not-sent correction no longer needs a raised counter (L4), and
  no bounded history is consumed. A fresh `begin` restores its unregistered,
  history-free copy. The not-sent cause is logged before the write. Tests:
  `unsent_requests_restore_the_attempt_instead_of_an_unknown_outcome` (empty
  history, zero preparations), `a_not_sent_correction_is_undone_and_can_be_sent_later`
  (the saved attempt equals the one before the reservation).
- **L2. Resuming `begin` ignored the requested method.** A resume of an
  elapsed cooldown or an unsent first request now refuses a method other than
  the attempt's, naming it, instead of silently sending with the saved one.
- **L3. `ProfileIncomplete` did not prove nothing was sent.** A library caller
  could answer a lab attempt's pending challenge through the default flow,
  saving a local block over a server challenge, which `reset` then accepted.
  `submit`, `request_email_code` and `resume_email_challenge` now refuse another
  profile's attempt, and the reset exemption also requires a non-lab attempt
  whose history never went past `unregistered`/`preparing`/`blocked`. The
  flow trait's "not implemented" defaults for consent and the email request
  now return `RequestNotSent` (nothing was sent), not an unknown outcome.
  Test: `another_profile_cannot_answer_or_reset_a_pending_challenge`.
- **L5. A failed restore hid the not-sent cause.** The error now names both
  the cause and that the attempt stays `preparing`/`verifying`. Test:
  `a_failed_restore_reports_the_cause_and_the_stuck_state`.
- **L6. `claim_store` ignored existing account material.** An unclaimed store
  holding another number's account record is no longer claimed. Test:
  `a_store_with_another_numbers_material_is_not_claimed`.
- **L7. `accounts new` saw only `request.json` beside the new directory.** It
  now reads every `*.json` in the account directory's siblings and in its
  parent's siblings (so `lab/accounts/*` and `lab/signup-json-*`, any of the
  documented file names), bounded by the config size limit and a file count. A
  config anywhere else is still not seen.
- **Nits fixed:** a cooldown also keeps `email_otp_wait` when longer (as the
  blocked branch does); the `exist` probe refuses during a consent refusal's
  waits, as consent inspection already did; the empty-store refusal names
  `lab-signup begin --create-store` as well as `store init`.
- **Nits recorded, not changed:** a sent preflight followed by a code request
  that never left is `outcome_unknown` with no reconciliation (while the
  standalone `exist` probe sends the same request with no reservation); the
  lab transport patches and encrypts at send time, so "built before either is
  sent" holds for the factory, not that wrapper (its failures are not-sent,
  which is safe); the email marker is written after `finish_failed` in its own
  transaction (a crash in between keeps the attempt unknown, the safe
  direction); an older binary drops `not_before` when it re-saves a record; a
  refused store open still leaves the parent directory and `<store>.lock`; a
  dangling-symlink store path gets the default file mode; `resend-code
  --method <other>` never builds with the stored lab factory (its frozen method
  is checked), now an exact no-op restore each time.

## Third review (2026-10-13)

A review of `ddc443c` with probe tests (110 existing registration tests and six
probes, in a scratch clone) found the exact restore sound: between reservation
and failure the attempt sits in `preparing`/`verifying`, which every command
refuses; the restore checks id, generation and state; the only fields a
reservation changes (state, history, `updated_at`, a resend's method) are all
put back; a handle made valid again is the one current when the request was
reserved, correctly, since nothing left; nothing reads `updated_at`; records
saved under the removed edge still load and resume. Findings and what was done:

- **Medium (fixed): an unreadable 5xx or 429 email-code answer still allowed
  a second email request.** Probed for 504, 502, 500, 429 and 404: each let
  `resume-email-challenge` and a second `/v2/code method=email_otp` through. A
  gateway 5xx may follow a backend that already emailed the code; 408 and 429
  carry a wait we cannot read. Only 4xx other than 408 and 429 is now a
  definite refusal. Tests: `an_unreadable_uncertain_answer_to_an_email_request_is_not_resumed`
  (200, 500, 502, 504, 408, 429), `lost_email_code_requests_resume_only_once` (404).
- **Low (fixed): the same profile id with the default flow could still write
  a local block over a pending challenge.** A library caller reopening the
  client without its registration flow gets `UnsupportedFlow` under the same
  profile id, which `check_profile` does not distinguish. `UnsupportedFlow`
  now sends nothing and returns `RequestNotSent` when answering a challenge or
  resuming an existing attempt, so the exact restore keeps it; a fresh attempt
  is still blocked locally with `ProfileIncomplete`. Test:
  `the_default_flow_restores_a_challenge_it_did_not_create`.
- **Low (recorded): preparations counted in older records stay counted.**
  Records restored before the exact restore (as an elapsed cooldown, or as
  `unregistered` with history) keep their never-sent `-> preparing` entry, so
  `code_preparation_count`, and with it the preflight's
  `client_metrics.attempts` and the lab's `attempts == count + 1` check, stay
  one high per such restore. Rewriting saved history would fabricate records,
  so it is left as is; such a store's operator raises `attempts` by that count.
- **Nits fixed:** a failed restore says the attempt "stays in" its state only
  when the store write or the material restore failed, not when the attempt
  changed meanwhile (the restore defaults now use their own error field);
  OpenAPI states the cooldown's `email_otp_wait` and the new classification;
  the `accounts new` scan refuses, with a message, past its file bound instead
  of answering "not configured", reads only regular files through a size
  limit, and looks beside the account directory's parent only in
  `signup-json-*` directories (never the whole parent's parent, which may be
  `$HOME`); `reset` of an unsent `unregistered` attempt and `begin` with
  another method now say why they refuse.

Not acted on: no test pins reqwest's `is_connect` meaning (the review re-read
reqwest 0.12.28 / hyper-util 0.1.20: connect errors arise only from the
connector, pool checkout or a closed checkout, before any request byte is
written; handshake failures and the overall timeout are not connect errors).

## Confirmed open

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
`Retry-After` header) into `outcome_unknown`; a save failure after a server
`ok` loses the login; diagnostics print server JSON key names.

Since checked (2026-10-13):

- **Fixed: store files used the default umask.** Records are encrypted, but a
  new store file was group/world-readable under a common umask. A newly created
  store file is now 0600 (unix), created after the owner lock so a refused open
  leaves nothing behind; an existing file's mode is not changed. Test:
  `a_new_store_file_is_private`.
- **Fixed: the CLI suggested `request-email-code [--email <address>]`,** which
  is always rejected (the comparison client sends no address). The hint now
  names the command without it.
- **Confirmed open: a `/v2/code` answer of `ok` or `attached` is blocked as an
  unimplemented branch and its `login` discarded.** Our code-request handling
  (`ComparisonHttpFlow::begin`) accepts only `sent`; any other answer goes to
  `blocked_response`, which saves `Blocked{UnsupportedStep}`, or a cooldown if
  it carries `retry_after` (which a later `begin` would resume with another code
  request after an `ok`; never observed). Static evidence (2.26.37.71,
  `KotlinRegistrationBridge.A09`, the `/v2/code` parser, retained in
  [the core bundle](apk/2.26.37.71-core/registration-response-bridge.txt)):
  `attached` maps to `02S.A01` (`0254..0268`), `ok` to `02S.A0C`
  (`04da..04ee`), `sent` to `02S.A00` (`04f6..050a`), and `login` is then read
  into `IDM.A0N` for every status (`0514..0526`; an absent or unknown status
  gives `02S.A0Y`). `02S` is a shared pool of boxed `Integer` constants used
  elsewhere too, so the next trace follows `IDM`'s status-integer consumer,
  not references to `02S`. **Not traced:** what the app
  does with `02S.A0C`/`02S.A01` (whether `ok` completes registration without
  an OTP, and what `attached` means). Completing registration from it would be
  a guess, so the branch stays blocked; the `login` is a secret and is not
  persisted from this path. Never observed live.
- **Confirmed open: one wrong OTP is terminal.** A `/v2/register` answer
  `fail`/`mismatch` (or `incorrect`) goes through the generic refusal path and
  saves `Blocked{ServerRefused}` reached from `verifying`; no command returns to
  code entry (the code-request one-shots require a refusal reached from
  `preparing`, by design). Static evidence (2.26.37.71,
  `KotlinRegistrationBridge.A0D`, retained in
  [the core bundle](apk/2.26.37.71-core/registration-response-bridge.txt)):
  `mismatch` maps to `HD6.A0A` (`0280..0294`), `incorrect` to `HD6.A09`
  (`02ac..02c0`), `too_many_guesses` to `HD6.A0E` (`026a..027e`), and the reply's
  `guess_wait` is read with `optString` into `HdD.A0C` (`0338..0344`).
  **Not traced:** which screen consumes `HD6.A0A` (whether the app returns to
  code entry, and with which wait) and the unit and format of `guess_wait`; no
  retained bundle references either, and our wire parser ignores `guess_wait`.
  A re-entry path must honour that wait, so it is not built on a guess.

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

Property-tested since 2026-10-13 (`crates/whatsapp-api/tests/registration_props.rs`):
the response parser never panics on arbitrary bytes under any HTTP status, nor
on registration-shaped JSON with values of any type; well-typed answers
(including waits up to `u64::MAX` seconds, as numbers or strings) always parse;
and neither a parsed answer's `Debug` nor an error's text echoes a value
planted in any secret field (`login`, `code`, `wipe_token`,
`passkey_credential`, `appeal_token`, `edge_routing_info`, `email`). Waits
saturate when added to the clock (`Timestamp::saturating_add`).
