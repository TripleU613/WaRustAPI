# Working agreement for TeleWhatsApp

These instructions capture the owner's directions from the project conversation.
Read this file before working here. Apply it throughout the repository, subject
to higher-priority instructions. This is a working agreement, not a transcript:
credentials, real phone numbers, OTPs and account PINs must not be copied here.

## Nothing learned may be lost (read this first)

Every finding lands in the repository in the same change that produced it.
Research that exists only in a chat reply, a terminal scrollback or a subagent
transcript is lost work: it will be paid for again, and the second answer may
silently contradict the first. If it was worth discovering, it is worth writing
down.

For anything explored, changed or discovered:

- **`docs/`** — the finding itself, in the protocol/design document it belongs
  to (`docs/protocol/<area>.md`). Carry the evidence with it: the APK build, the
  class and method it was read from, and an explicit **confirmed** /
  **inferred** / **observed** (seen live) / **unknown** label. State the
  reasoning, not just the conclusion, so the next reader can re-check it without
  redoing the trace.
- **`docs/api/openapi.json`** — every new or changed command, flag, capability,
  route, field and recognized server response, with `x-cli`, `x-transport` and
  `x-live` set honestly. Never describe something as served over HTTP before the
  route exists. `docs/api/README.md` has the checklist.
- **CLI** — expose what a human or another agent will need to inspect or
  reproduce the result, then regenerate the reference:
  `UPDATE_CLI_DOCS=1 cargo test -p whatsapp-cli --features experimental-transport cli_reference`.
  A capability nobody can invoke is not finished.
- **Code** — put the evidence where the value is used. A magic field number,
  attribute name or byte offset carries a comment naming its source. Explain the
  non-obvious *why*; do not narrate what the code plainly does.
- **`STATUS.md`** and the relevant checklist — what is now true, and honestly
  what is still unverified.

**Record negative results and unknowns too.** "This does not exist", "this
attribute is absent in this build", "this path was traced only to here" and
"this was refuted" are the most expensive things to rediscover and the easiest
to lose. Write down what was searched and what was *not* established, so nobody
repeats a dead end or mistakes an untraced assumption for a confirmed fact.

**Diagnosability is part of the work.** When something fails silently, fix the
silence as well as the failure: a refusal should say which input was missing,
and an inspection command should be able to show it. A guess that looks like a
fact costs more than an unknown that is labelled.

## What the owner is building

- Build toward full WhatsApp onboarding and sending the first message.
- Implement what the official APK actually does, using APK-derived evidence.
  The owner explicitly said: **no bypassing, breaking or hacking WhatsApp**.
- The intent is normal WhatsApp use through this client instead of the APK.
  Do not reinterpret requests for compatibility or avoiding false positives as
  permission to evade detection, bans, challenges, verification or rate limits.
- Prioritize observable results: a correct request, a useful server response,
  SMS dispatch, completed verification, then actual messaging. Do not confuse
  one milestone with another or build unrelated infrastructure instead.

## Listen and communicate

- Follow the actual request. Do not silently substitute your preferred workflow
  or turn a small data change into an engineering project.
- When the owner asks why, **answer why first**. An explanation request is not
  permission to start another implementation detour.
- Be direct, plainspoken and concise. Avoid lectures, repetitive questions,
  unnecessary approval rituals, defensive explanations and boilerplate.
- Own mistakes. Distinguish a limitation in our implementation from a constraint
  imposed by WhatsApp. Never present our design choices as server requirements.
- Explain a material obstacle or scope change **before** spending time on it.
  Say what is blocked, what proves it, and what correction is necessary.
- When a requested test exposes a fixable bug in our own guard or plumbing,
  do not end the turn merely reporting it. Explain briefly, fix the reusable
  behavior, add tests, and resume the authorized test on the saved attempt when
  safe. Do not make the owner say “fix it” again. This does not authorize
  ignoring server waits, confirmation requirements, or genuine external blockers.
- Do not repeatedly ask for facts already supplied, including the APK location,
  lack of an Android device, the selected number, or an ownership confirmation
  already given. Reuse relevant context without assuming it covers unrelated
  accounts or numbers.
- Ask only when information genuinely cannot be inferred and is needed to act
  correctly, or when new authority is necessary. Do not ask optional questions
  merely to delay work. Do not request OTPs, passwords or account PINs in chat.
- During ongoing work, give short updates about meaningful findings, tests,
  requests and blockers. Do not leave the owner wondering whether work stopped.
- Make troubleshooting easy to follow: what changed, why, what was sent, what
  came back, and what that result does or does not establish.

## Number changes are data changes

**A new number normally means editing private JSON and repeating the working
request—not adding Rust code, another command or a number-specific script.**

- Interpret “try this number” as applying the latest working, evidence-backed
  request recipe to that number, unless the owner says otherwise.
- Change the country/national number and recompute genuinely number-dependent
  inputs such as the registration token. Keep unrelated request settings
  stable so the result is understandable.
- Do not revert to an obsolete clear-form request, broken token calculation or
  missing headers simply because an old command defaults to them.
- Reuse an existing saved attempt for that number when applicable. Preserve
  keys, identifiers, history, reservations and server-directed waits.
- A number not previously configured gets its own private configuration and
  saved account state; do not copy another account's credentials or identifiers.
- Check the saved state before sending. Do not delete a store or rotate identity
  to escape a refusal, cooldown, pending challenge or uncertain outcome.
- If frozen configuration prevents applying a demonstrated correction, explain
  that **our tooling** is the obstacle. Prefer an existing explicit migration.
  Do not silently launch another framework rewrite or create fresh identity.
- If a genuinely missing capability requires code, make a reusable, bounded
  improvement—not a branch, file or command for the particular phone number.
  Explain the need before expanding the task.
- Do not copy stale failure reasons or another attempt's telemetry. Use actual
  saved history for counters; distinguish a deliberately token-only experiment
  from a normal new preparation.

## The editable request file

- The owner described `request.rs` / “requests.rs” as an env-like or Aurora-style
  JSON profile, **not a large system implementation file**. Respect that intent:
  a small values/template surface, private editable JSON, separate plumbing.
- Current source template: `crates/whatsapp-api/src/registration/request.rs`.
  Configuration types and validation live in `request_config.rs`.
- Private files may be named `request.json`, `request-corrected.json` or
  `signup.json`; use `--config`. Do not create a second incompatible format
  just to standardize the filename.
- Keep configurable request inputs in this surface: phone, country, language,
  locale, client/User-Agent, applicable device prerequisites, token, IDs,
  delivery method, endpoint fields, encoding mode and transport timeouts.
- “Everything that isn't hardcoded” means avoid scattered experimental values.
  It does not mean exposing switches to disable TLS checks or invent identity,
  attestation, location or challenge responses.
- Additional observed fields belong in endpoint maps where possible. Extend
  these from APK evidence without requiring Rust changes for each value.
- `transport.body_mode` supports `clear_form`, `enc`, and `enc_empty_h`.
  Preview must accurately show the selected mode. Empty H is not attestation.
- Preserve private originals when correcting a saved configuration. Keep the
  current active config path clear so subsequent verification uses that file.
- Config files containing secrets need private permissions and stay outside Git.

## Research from the APK, not a device

- Available comparison APK: `~/Downloads/WhatsApp.apk`. As of
  2026-09-18 that file is build **2.26.37.71** (SHA-256 `1755111a…7d5c`), not
  the 2.26.37.70 build most recorded evidence cites; its obfuscated class names
  differ, so re-anchor by string constants and attribute new claims to 37.71.
- Pinned reference version: **2.26.36.75**. Comparison versions: **2.26.37.70**
  and **2.26.37.71**. Keep their evidence separate and identify which build
  supports each claim.
- No test Android device is available. The owner explicitly wants static APK
  analysis, not traffic capture. Do not keep proposing a device or capture as
  the required next step.
- Trace outbound POST construction: endpoint, fields, encodings, headers,
  defaults, branch conditions, producer functions and response handling.
- A field's existence is not proof that it is mandatory. Follow its callers and
  conditions; distinguish confirmed behavior, inference and unknown runtime data.
- Use the APK to identify and satisfy actual protocol requirements. Do not
  invent values just to get a different error or manufacture device provenance.
- Preserve useful reverse-engineered code/protocol information in compact,
  attributable research artifacts. Strip media such as images/audio/fonts from
  the committed research bundle; do not build an unnecessary media inventory.
- Raw APKs, bulk unpacked/decompiled trees and private experiment data stay local.
  Do not delete local resources needed for derivation merely because media is
  excluded from Git.

## Persistence and live testing

- “Continue” means resume from saved state and the last verified result, not
  restart the project or repeat finished investigations.
- The owner repeatedly asked for sustained work toward a better server response:
  investigate, implement an evidence-backed correction, test it, inspect the
  result, and continue when another justified in-scope step remains.
- Do not stop merely after writing code, producing a plan, or passing an offline
  test when the requested live validation can safely be performed.
- Persistence does not authorize endless unchanged requests, brute-force token
  guesses, invented telemetry, identity rotation or ignored server cooldowns.
- When authorized, send a bounded request for the selected number. Keep
  automatic HTTP/application retries disabled and preserve ambiguous outcomes.
- If a failure remains, identify its actual meaning and investigate relevant
  request construction. Do not assume `bad_param` proves an absent field; it can
  also reflect an invalid value or encoding.
- If truly blocked, explain the specific evidence and necessary next input or
  wait. Do not claim success or pretend you will run indefinitely in background.
- Report exact recognized status/reason and wait durations without logging raw
  secret-bearing request/response bodies. Respect SMS and voice waits alike.
- `status: sent` establishes server-reported dispatch, not handset receipt.
  An OTP challenge is not completed registration; registration is not proof
  that message sending works. Enter OTPs/account PINs locally without echo.
- Keep troubleshooting redacted and reproducible: selected field names, body
  mode/size, HTTP status, recognized response and preserved account state.

## Repository, docs and CI

- Keep the README short: purpose, honest status, quick start and links.
  Put detailed design, research, troubleshooting and usage in `docs/` rather
  than allowing the README to become a long manual again.
- **Everything goes in the OpenAPI document.** Any new or changed command, flag,
  capability, route, field or recognized server response is documented in
  `docs/api/openapi.json` in the same change, with `x-cli`, `x-transport` and
  `x-live` set honestly. `docs/api/README.md` has the checklist; the CLI coverage
  test enforces command coverage. Do not describe a capability as served over HTTP
  until the route exists.
- Maintain normal repository essentials: useful docs, tests, licensing,
  contribution/security guidance, formatting, linting and predictable commands.
- CI should be selective and purposeful, not expensive full builds on every
  push. Use relevant path/change filters, job dependencies and concurrency
  cancellation; keep required checks reliable for PRs and meaningful changes.
- Add tests for fixes, request encoding, config parsing, redaction, state
  persistence and retry guards. Prefer fixtures grounded in APK evidence over
  expected values copied from the implementation being tested.
- Run verification proportionate to the change. For shared Rust changes, use
  formatting, workspace/all-feature tests, strict Clippy and repository hygiene
  when available. Report what actually ran; never invent passing checks.
- When asked to inspect/merge a PR, inspect its diff, checks and conflicts, then
  complete the authorized merge when sound. Do not force a merge or overwrite
  user changes merely to satisfy “somehow.”
- When asked to push everything local, push all relevant reviewed project work,
  not credentials, private lab data, large raw APKs, unrelated changes or
  generated build output. Report the actual commit/push result.
- The owner authorized installing needed tools. Use that for relevant tools,
  not unrelated system modifications. Never store or repeat the sudo credential
  shared in the conversation, and never add it to commands or tracked files.
- Broad encouragement to get results does not authorize destructive operations,
  purchases, resets, account recovery or other material scope expansion.

## Known lessons and handoff

- Current ten-item work and live-check boundaries are tracked in
  `docs/messaging-completion.md`; consult it before relying on the older lab
  milestones below. `listen` uses the explicit comparison public-client actor,
  without draining sends unless `--send-outbox` is given; that flag transmits
  every eligible queued intent from any namespace, so run `outbox-pending`
  first and `cancel` anything unwanted. The library opt-in is
  `comparison_outbox(true)`. Incoming supported text now reaches the
  durable event journal atomically; `events --summary` is the redacted inspection
  path. Historical projection uses original authenticated captures and skips
  consumed view-once content. Do not delete event markers to repeat projection.
  `outbox-send` now sends only its selected private-JSON text intent through the
  public actor. One live test has saved server acceptance and recipient delivery;
  no read/played receipt or human handset confirmation yet. Reusing that JSON
  returns locally without another transmission. Never give it a new ID to resend.
  A separate normal-outbox PNG test now also has a matching server ACK; its
  private JSON is `lab/signup-json-PbDdYU/message-outbox-image-test.json`.
  Repeating that JSON returns locally. Never rename its ID to repeat the image.
  The image subsequently received a delivery receipt. A normal-outbox synthetic
  voice note also received a matching ACK and delivery receipt; its JSON is
  `lab/signup-json-PbDdYU/message-outbox-voice-test.json`. Repeats return locally.
  Handset image display and audible voice-note playback remain unconfirmed.

- View-once image sends support private JSON `image` plus `view_once: true`, with
  the APK-derived image flag and V2 wrapper. One send was server-acknowledged;
  the operator confirmed one opening and refusal to reopen. Inspection is not opening.
  `lab-open-view-once` consumes a downloaded picture atomically before piping
  plaintext to a 30-second local mpv viewer. It removes accessible body/protobuf
  records and refuses reopening, with no plaintext file. A crash after consumption
  does not restore it. Server opened receipts remain unimplemented; this is a
  local single-use viewer, not full cross-device view-once synchronization.
  `lab-download-view-once` can prefetch and authenticate ciphertext into private
  unopened storage. A live receive correction subsequently yielded a qualifying
  descriptor and downloaded/authenticated 91,562 encrypted bytes, unopened.
  It does not export plaintext. Receive now sends the APK active IQ and delivery
  confirmations only for already authenticated/stored direct messages; supported
  incoming receipts receive acknowledgements. No read/opened receipt is sent.

- A two-second synthetic Ogg/Opus beep was sent as a voice note with a matching
  server acknowledgement. Use private JSON `voice_note` instead of `text`/`image`;
  keep its request ID stable. The operator confirmed handset receipt; playback
  was not separately confirmed.

- The comparison lab now uploaded one encrypted PNG (HTTP 200) and received a
  matching image-message acknowledgement; the operator confirmed handset receipt.
  Private message JSON accepts `image` (PNG path) instead of `text`. Keep the
  image request ID stable: repeat returns locally, uncertain uploads/sends block.
  Media routes expire and can be explicitly refreshed with `--login --media-route`.
  Keys, uploaded bytes, response and send intent remain in encrypted storage.

- Comparison messaging now has a live verified certificate, primary login
  `success`, and one encrypted text with a matching server message acknowledgement.
  This is the explicit `lab-transport` path, not the default outbox runtime.
  The operator confirmed handset receipt and reported replying. Two inbound
  payloads were subsequently decrypted offline with the existing pinned session:
  one extended-text reply and one `EPHEMERAL_SYNC_RESPONSE`, not two user replies.
  `lab-inbox --inspect` classifies saved payloads locally without printing content
  or advancing sessions. Never count encrypted envelopes as user replies.
  `lab-transport --login --receive` captures a bounded window; `lab-inbox`
  decrypts captures locally and skips previously committed messages. Plaintext
  stays encrypted at rest. Continuous receiving and receipt handling are incomplete.
  Consult `docs/protocol/messaging-transport.md`.
  Reuse the private message JSON's request ID: acknowledged sends return locally,
  uncertain/refused sends remain blocked. Never change the ID merely to resend.
- The APK hashes only the first Noise ephemeral, not a trailing empty payload.
  Standard snow XX message handling adds that payload hash and failed live
  decryption. The comparison classic adapter corrects sequencing over existing
  primitives; certificate verification must remain enabled.

- New numbers: never start from `lab-signup template` defaults. Copy the working
  recipe (body_mode `enc_empty_h`, the 2.26.37.70 User-Agent, the eight observed
  `code_bytes` fields) and derive the token from the 2.26.37.70 material; the
  2.26.37.71 APK on disk yields wrong tokens for that recipe. The
  `LabRegistrationConfig` schema in `docs/api/openapi.json` records the recipe.

- Read `STATUS.md`, `docs/protocol/registration-lab.md`,
  `docs/protocol/http-post-analysis.md` and `docs/protocol/evidence-ledger.md`
  as relevant, plus current Git status, before choosing the next experiment.
- Token salt bug: zero-based byte 62 was transcribed as `0xfc`; the comparison
  APK says `0xf8`. Fixing it changed one saved attempt from `bad_token` to
  `sent`. Preserve the APK-linked salt regression test and independent fixture.
- The APK maps `format_wrong` to an invalid-mobile-number-for-country dialog;
  that is its user-facing interpretation, not proof of the server's internals.
- `retry-corrected-token` permits only a changed token after HTTP 200/bad_token.
  `retry-corrected-request` migrates an opaque historical refusal using JSON
  and shares its reservation with the legacy diagnostic. Neither is a generic
  refusal-reset mechanism. Preserve their state checks and atomic reservations.
- At this handoff, one attempt had server-reported SMS dispatch and another had
  `too_many_all_methods` with a persisted cooldown. Consult current local state
  for which action is now valid; do not rely on elapsed time guessed from chat.
- Real numbers, tokens, keys and attempt files are in ignored `lab/` storage.
  Never copy their values into this file, tests, docs, commits or status reports.
- The messaging runtime reconnects only after classified recoverable loss, bounded
  by private JSON `transport.reconnect` (default five attempts; `0` disables).
  Refusals, revocation, uncertain logins and worker panics never retry. Do not
  widen recoverability to make a refusal disappear; reservations are never replayed.
- 2026-10-13 cloud handoff (no APK in that container; only retained bundles
  under `docs/protocol/apk|rea` were usable). ML-KEM-1024/0x44 hybrid sessions
  are integrated offline across inbox (local KEM rows), outbox, identity refresh,
  device-key batches and group fan-out, but no KEM prekey is generated or
  uploaded and `pqsupport` is never sent, so incoming hybrid bootstrap needs a
  KEM row this client cannot yet create. Next APK traces, in order: KEM prekey
  generation and the `pq_list`/`pq_last_resort_key` upload shape, the
  `pqsupport` producer, the outgoing revoke/edit `edit` attribute, sender-side
  `contextInfo.expiration`, and the native retry-receipt answer policy. Replay
  guarantees and open observations: `docs/protocol/replay-safety-audit.md`.
  Quoted image/voice-note sending is offline-only; per-area state is in
  `docs/feature-parity.md`. First checks for the next live receive window:
  status-view receipts are recorded and ACKed again (they ended `listen` from
  2026-10-10 to 2026-10-13), and the 2026-10-11 ACK/confirmation encodings are
  accepted (`confirmed=true` with no redelivery at the next login); see
  `docs/protocol/unknowns.md`. Refused stanza content must never end receive:
  validate before the store, let stored-row faults propagate. Registration
  (2026-10-13): `lab-signup begin` no longer creates a missing store (pass
  `--create-store` once for a profile made before `accounts new` created
  stores); a request that provably never left restores the attempt instead of
  `outcome_unknown`; see `docs/protocol/registration-safety-audit.md`.
- Most important lesson: **listen first, change data for data requests, explain
  genuine obstacles upfront, and deliver the requested result without detours.**
