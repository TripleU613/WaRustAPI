# Working agreement for WaRustAPI

## Recording project work

- **`docs/`** — the finding itself, in the protocol/design document it belongs
  to (`docs/protocol/<area>.md`). Carry the evidence with it: the APK build, the
  class and method it was read from, and an explicit **confirmed** /
  **inferred** / **observed** (seen live) / **unknown** label. State the
  reasoning, not just the conclusion, so the next reader can re-check it without
  redoing the trace.
- **`docs/api/openapi.json`** — every new or changed command, flag, capability,
  route, field and recognized server response, with `x-cli`, `x-transport` and
  `x-live` set for the implemented interface. `docs/api/README.md` has the checklist.
- **CLI** — expose what a human or another agent will need to inspect or
  reproduce the result, then regenerate the reference:
  `UPDATE_CLI_DOCS=1 cargo test -p whatsapp-cli --features experimental-transport cli_reference`.
- **Code** — put the evidence where the value is used. A magic field number,
  attribute name or byte offset carries a comment naming its source.
- **`STATUS.md`** and the relevant checklist — what is now true, and honestly
  what is still unverified.

## What the owner is building

- Build toward full WhatsApp onboarding and sending the first message.
- Implement what the official APK actually does, using APK-derived evidence.
- Prioritize observable results: a correct request, a useful server response,
  SMS dispatch, completed verification and messaging.

## Number changes are data changes

A new number uses a private JSON profile and the working request recipe.

- Interpret “try this number” as applying the latest working, evidence-backed
  request recipe to that number, unless the owner says otherwise.
- Change the country/national number and recompute genuinely number-dependent
  inputs such as the registration token. Keep unrelated request settings
  stable so the result is understandable.
- Reuse an existing saved attempt for that number when applicable. Preserve
  keys, identifiers, history, reservations and server-directed waits.
- A number not previously configured gets its own private configuration and
  saved account state.

## The editable request file

- The owner described `request.rs` / “requests.rs” as an env-like or Aurora-style
  JSON profile, **not a large system implementation file**. Respect that intent:
  a small values/template surface, private editable JSON, separate plumbing.
- Current source template: `crates/whatsapp-api/src/registration/request.rs`.
  Configuration types and validation live in `request_config.rs`.
- Private files may be named `request.json`, `request-corrected.json` or
  `signup.json`; use `--config`.
- Keep configurable request inputs in this surface: phone, country, language,
  locale, client/User-Agent, applicable device prerequisites, token, IDs,
  delivery method, endpoint fields, encoding mode and transport timeouts.
- Additional observed fields belong in endpoint maps where possible. Extend
  these from APK evidence without requiring Rust changes for each value.
- `transport.body_mode` supports `clear_form`, `enc`, and `enc_empty_h`.
  Preview must accurately show the selected mode. Empty H is not attestation.
- Preserve private originals when correcting a saved configuration. Keep the
  current active config path clear so subsequent verification uses that file.
- Config files containing secrets need private permissions and stay outside Git.

## Research from the APK, not a device

- **Getting an APK, including in a cloud session.** The owner publishes the
  latest official APK as an asset of the GitHub pre-release `nothing_yet`
  ("Not a release - Just the WA APK"). It is research input, not a project
  release. Download it outside the checkout and verify it before use:

  ```sh
  mkdir -p /tmp/wa-apk && gh release download nothing_yet \
    -R TripleU613/WaRustAPI -p 'WhatsApp.1.apk' -D /tmp/wa-apk
  sha256sum /tmp/wa-apk/WhatsApp.1.apk
  cargo run --locked -p xtask -- reference-manifest /tmp/wa-apk/WhatsApp.1.apk
  ```

  As of 2026-11-01 that asset is build **2.26.43.72** (version code
  264307238, SHA-256 `887a6f94…0c2e`, signing certificate `39:87:D0:43:…:5D:44`,
  the same as 2.26.37.70/71). Its comparison with what we rely on
  (token dictionary identical, token key inputs unchanged except `classes.dex`)
  is in `docs/protocol/apk-2.26.43.72-comparison.md`. If the
  hash differs, the owner replaced the asset: identify the new build before
  citing it.
- On the owner's workstation, `~/Downloads/WhatsApp.apk` is build
  **2.26.37.71** (SHA-256 `1755111a…7d5c`). Most recorded evidence cites
  2.26.37.70 or 2.26.37.71; obfuscated class names differ between builds, so
  re-anchor by string constants and attribute each claim to the build it was
  read from.
- Pinned reference version: **2.26.36.75**. Comparison versions: **2.26.37.70**,
  **2.26.37.71** and **2.26.43.72** (compared, not re-traced). Keep their
  evidence separate and identify which build supports each claim. The working
  recipe uses 2.26.37.70 material (see the new-numbers lesson below). A
  2.26.43.72 token derived from that APK, sent with the 2.26.43.72 User-Agent,
  was answered `sent` on 2026-11-02; a token must always match its User-Agent's
  build.
- Use static APK analysis. The workstation has no test Android device.
- Trace outbound POST construction: endpoint, fields, encodings, headers,
  defaults, branch conditions, producer functions and response handling.
- A field's existence is not proof that it is mandatory. Follow its callers and
  conditions; distinguish confirmed behavior, inference and unknown runtime data.
- Preserve useful reverse-engineered code/protocol information in compact,
  attributable research artifacts. Strip media such as images/audio/fonts from
  the committed research bundle.
- Raw APKs, bulk unpacked/decompiled trees and private experiment data stay local.

## Persistence and live testing

- “Continue” means resume from saved state and the last verified result, not
  restart the project or repeat finished investigations.
- For a live registration test, send one request for the selected number. Keep
  automatic HTTP/application retries disabled and preserve ambiguous outcomes.
- If a failure remains, identify its actual meaning and investigate relevant
  request construction. A `bad_param` response can reflect a missing field, invalid value or encoding.
- Report exact recognized status/reason and wait durations without logging raw
  secret-bearing request/response bodies. Respect SMS and voice waits alike.
- `status: sent` establishes server-reported dispatch, not handset receipt.
  An OTP challenge is not completed registration; registration is not proof
  that message sending works. Enter OTPs/account PINs locally without echo.
- Keep troubleshooting redacted and reproducible: selected field names, body
  mode/size, HTTP status, recognized response and preserved account state.

## Repository, docs and CI

- The README covers purpose, status and quick start. Detailed usage and research live in `docs/`.
- Run formatting, strict Clippy, tests and repository hygiene before committing to `main`. Push each completed, tested change directly to `main` and check CI.
- For Cargo dependency updates, regenerate `docs/protocol/dependency-report.md` with `cargo run --locked -p xtask -- dependency-audit`. Supersede Dependabot PRs with a direct commit including the report. RustCrypto crates on the same digest generation move together.
- Keep CI scoped by changed paths. Use APK-derived fixtures for protocol regression tests.

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
  consumed view-once content.
  `outbox-send` now sends only its selected private-JSON text intent through the
  public actor. One live test has saved server acceptance and recipient delivery;
  no read/played receipt or human handset confirmation yet. Reusing that JSON
  returns locally without another transmission.
  A separate normal-outbox PNG test now also has a matching server ACK; its
  private JSON is `lab/signup-json-PbDdYU/message-outbox-image-test.json`.
  Repeating that JSON returns locally.
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
  or advancing sessions.
  `lab-transport --login --receive` captures a bounded window; `lab-inbox`
  decrypts captures locally and skips previously committed messages. Plaintext
  stays encrypted at rest. Continuous receiving and receipt handling are incomplete.
  Consult `docs/protocol/messaging-transport.md`.
  Reuse the private message JSON's request ID: acknowledged sends return locally,
  uncertain/refused sends remain blocked.
- The APK hashes only the first Noise ephemeral, not a trailing empty payload.
  Standard snow XX message handling adds that payload hash and failed live
  decryption. The comparison classic adapter corrects sequencing over existing
  primitives.

- New numbers use the working
  recipe (body_mode `enc_empty_h`, the 2.26.37.70 User-Agent, the eight observed
  `code_bytes` fields) and derive the token from the 2.26.37.70 material; the
  2.26.37.71 APK on disk yields wrong tokens for that recipe. The
  `LabRegistrationConfig` schema in `docs/api/openapi.json` records the recipe.
  `accounts new` writes exactly this recipe; verify with `token verify`.

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
  for which action is now valid.
- Real numbers, tokens, keys and attempt files are in ignored `lab/` storage.
- The messaging runtime reconnects only after classified recoverable loss, bounded
  by private JSON `transport.reconnect` (default five attempts; `0` disables).
  Refusals, revocation, uncertain logins and worker panics never retry. Reservations are never replayed.
- 2026-10-13 cloud handoff (no APK in that container; only retained bundles
  under `docs/protocol/apk|rea` were usable). ML-KEM-1024/0x44 hybrid sessions
  are integrated offline across inbox (local KEM rows), outbox, identity refresh,
  device-key batches and group fan-out, but no KEM prekey is generated or
  uploaded and `pqsupport` is never sent, so incoming hybrid bootstrap needs a
  KEM row this client cannot yet create. Next APK traces, in order: KEM prekey
  generation and what each `pq_list`/`pq_last_resort_key` element holds (ID,
  value encoding, signature input; the element shape is already read from
  retained evidence in `docs/protocol/kem-prekey-inventory.md`), the
  `pqsupport` producer (the bodies of the `Can` predicates, chiefly
  `Can.A04(jid)`; its five retained call sites are in
  `docs/protocol/pq-bundle-boundary.md`), the
  outgoing revoke/edit `edit` attribute (per-operation values are now traced
  to the message's `178.A00` writers; still needed: the `SendE2EMessageJob`
  constructor mapping, `Civ`'s stanza writer, and the text-edit/pin writers;
  see `docs/protocol/unknowns.md`),
  sender-side `contextInfo.expiration` (via `Cn6`/`B35.A01`), the native retry-receipt answer policy, and the
  wrong-OTP path (the `HD6.A0A` consumer and `guess_wait` unit), and what the
  app does with a `/v2/code` `ok`/`attached` answer (`02S.A0C`/`02S.A01`), and
  for pin-in-chat the `BVD` descriptor plus the name helpers that
  `xtask retained-proto-fields` reports missing. Replay
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
