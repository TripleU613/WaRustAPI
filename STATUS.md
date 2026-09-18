# Status — 2026-09-18

**Public ordinary-media download integrated (offline verified):** comparison
clients can list sealed incoming references with `received_media()` and explicitly
export authenticated ordinary images/voice notes with `download_media()` to new
private files. View-once content, foreign references, metadata changes and
file/symlink overwrite are refused. A cached-download client test stays offline.
Public ordinary upload now also returns stable reusable references for the normal
outbox, with offline reuse/restart/uncertainty tests. A fresh public-API network
upload and full cross-device synchronization remain unverified/incomplete.

**Normal-outbox image and voice note server-acknowledged:** private JSON image
and voice-note intents now reuse the bounded upload module and enter the public
outbox through stable account-bound sealed references. Text and media share Signal
reservation and ACK tracking. One new image test to the existing test contact
received a matching server ACK through the public outbox after an explicit expired
media-route refresh; a receive-only session then saved recipient delivery. A new
two-second synthetic voice-note test also received a matching normal-outbox ACK
and subsequent recipient delivery (no read/played receipt for either media test).
Repeating either private JSON returned its saved ACK without uploading or
connecting again. Handset display/playback confirmation and public upload/download
convenience APIs remain incomplete.

**Ten-item messaging work in progress:** the persistent listener now runs through
the public account actor (explicit comparison opt-in). A 75-second authenticated
live check completed cleanly; ordinary incoming image download/display also
passed live. Incoming primary prekey support, receipt fact tracking, a durable
post-view opened-receipt queue and ordinary voice-note playback are implemented
with offline tests. Opened-receipt, incoming voice and new-sender live checks
remain pending. Trusted-contact token notification handling and atomic key-share
storage from an already pinned own-device session are implemented and tested;
device linking, key activation and general cross-device synchronization are not.
Normal direct-text outbox sending
has a separate explicit opt-in: one selected private-JSON text received a matching
server acknowledgement through the public client/runtime. A subsequent receive
session persisted a recipient delivery receipt (no read/played receipt). Repeating
the same JSON returns locally without sending again. Human handset confirmation,
broader media/outbox and cross-device work remain incomplete. See the
[ten-item checklist and handoff](docs/messaging-completion.md).

The entries below are the historical experiment sequence; newer results above
and in the checklist supersede their then-current limitations.

**Local view-once viewer added:** `lab-open-view-once` authenticates/decrypts the
downloaded image and atomically consumes it before passing plaintext through a
pipe to a 30-second desktop viewer. No plaintext file is created. Reopening is
blocked, including after a viewer failure; accessible media body/key-bearing
protobuf records are replaced with consumed markers. Secure erasure of old
database pages is not claimed. Server opened receipts remain unimplemented.
The requested image was passed to the desktop viewer, which exited successfully
after its bounded display. It is now locally consumed, not available to reopen.

**Incoming view-once prefetch verified:** adding the APK's post-login active IQ
received a matching result, but alone produced only old messages. Delivery
receipts for previously authenticated/stored messages plus acknowledgements of
incoming receipts were then written; four additional payloads arrived and
authenticated offline. One contains ImageMessage flag 25 (`view_once=true`).
One HTTPS GET downloaded 91,562 encrypted bytes; size, hash and MAC passed.
The ciphertext is stored encrypted at rest as `downloaded_unopened`; no plaintext
export, viewing or opened receipt occurred. This sequence supports the missing
receive confirmations as the queue-progress issue, without proving a particular
server-side policy. No chats were resent or account identities changed.

**View-once sending confirmed:** the operator confirmed opening the outgoing test
picture once and being unable to reopen it. Incoming view-once download now has
an explicit ciphertext-only command, with HTTPS bounds, hash/MAC verification
and encrypted unopened storage. The initial invocation found no authenticated
descriptor; the receive correction above subsequently enabled a verified GET.
Local viewing/consumption is now available above; server opened receipts remain
unimplemented.

**First voice note server-acknowledged:** a locally generated two-second mono
Ogg/Opus beep was encrypted/uploaded and sent as `ptt` through the saved Signal
session. A matching server message acknowledgement was persisted. Repeating its
private JSON returned locally without another send. The operator confirmed
handset receipt; playback was not separately confirmed. Voice-note input is data
(`voice_note` path), not recipient-specific code.

**Incoming image message received:** after the operator corrected the recipient,
one bounded receive captured two new authenticated payloads. Offline decryption
identified one ImageMessage (Message fields 3/35, 1,227 protobuf bytes); the other
has fields 35/91: an associated-child wrapper containing another ordinary-image
descriptor, now classified from the APK. The attachment itself has not been
downloaded or decrypted. The inbox
now also scans pending/send-response captures from outgoing-message operations;
it previously scanned only dedicated receive captures. Image protobufs are now
explicitly labelled by redacted inspection. Inbound media download is not yet
implemented for ordinary images; the view-once prefetch above has not had a live
download to verify. Outbound success does not establish inbound media support.

**First image server-acknowledged:** the requested PNG was encrypted and uploaded
once (HTTP 200), then sent through the existing pinned Signal session. A matching
message acknowledgement was saved. Repeating the same private JSON returned
locally without another upload or connection. The operator confirmed receiving
the image on the handset. This is the explicit comparison lab path, not the default media
API. See [image usage](docs/protocol/messaging-transport.md#image-sending-comparison-lab).

**Inbound decryption verified:** one bounded authenticated receive connection
captured two encrypted `msg` stanzas. Offline processing used the explicit
`sender_pn` mapping on their LID envelopes and the existing pinned Signal session
to authenticate/decrypt both. Plaintext protobufs and advanced sessions were
committed atomically in encrypted storage. A second offline run skipped both
without changing the ratchet or contacting the server. No chats or receipts were
sent. Continuous receiving, inbox display and receipt handling remain incomplete.

**Payload classification correction:** those two encrypted payloads are one
extended-text message (fields 6/35) and one protocol message (field 12), type 4
`EPHEMERAL_SYNC_RESPONSE`. They are not two user replies. Offline `lab-inbox
--inspect` reads the saved plaintext structure without advancing sessions or
printing message content. Comparison APK `LX/BXF`, `LX/BXE` and `LX/C1B` establish
these field/type mappings.

**First live encrypted text message server-acknowledged:** the comparison lab
verified the server certificate (intermediate serial 3 / leaf serial 361),
received authenticated login `success`, fetched the selected recipient's key
bundle, verified its signature, committed Signal session/send intent atomically,
and sent one encrypted text. A matching `ack` / `class=message` with no error
was persisted. Repeating the same private message request returned the saved
acknowledgement without connecting or sending again. The operator subsequently
confirmed handset receipt and reported replying. Inbound decryption is now
verified above; protocol delivery/read receipt handling remains unverified. This explicit lab
flow is not the default CLI/outbox runtime.
See [transport progress and usage](docs/protocol/messaging-transport.md).

**Live registration completed:** the operator supplied the requested DOB locally
and one `/v2/consent` submission returned HTTP 200 / `ok`, with no pending branch
or wait. The same saved attempt transitioned to Registered; returned credentials
were committed in encrypted storage. Messaging authentication and server-side
message acceptance are now verified above. No DOB, phone or credential values
are recorded here.

**Consent follow-up:** an OTP submission returned HTTP 200 / `fail` / `consent`.
Static comparison-APK analysis found separate consent-result and age-verification
request shapes. The client now supports one-shot context inspection and explicit
private-JSON DOB/youth-consent submissions using saved keys and intermediate login.
Pending parent/app-store/unknown branches remain blocked. No choice is invented.
Live context refresh confirmed `pending=dob`, an intermediate login, no consent
ID/version, and no waits. The subsequent explicit DOB submission completed
registration as described above.
See the [consent trace](docs/protocol/http-post-analysis.md).

**Post-OTP hardening:** library-side answer validation preserves challenges on
local errors; PIN transitions require successful HTTP and the expected status;
PIN waits are enforced; credential provenance distinguishes register/security.
Offline response tests cover refusal families and unknown outcomes without
answer replay. [Branch coverage and remaining work](docs/protocol/after-otp.md).
Device-confirmation/second-code continuation remains missing; the later live
messaging result is recorded above.

**New onboarding patch (live dispatch verified):** new JSON templates enable an
existence preflight before code requests. The builder reuses saved identity;
typed recommendations and waits gate the selected method. Unknown/recovery
branches stop explicitly, and tests verify that they do not trigger SMS.
The fallback-only preflight response now permits the configured SMS method when
recommendations are absent and waits are zero. Resuming the saved attempt with
unchanged identity returned HTTP 200 / `sent` from `/v2/code`, with SMS wait 125
seconds and voice wait 65 seconds. Handset receipt and fresh-account signup
remain unverified. The local guard fix and continuation rule are in AGENTS.md.
One later code submission reached `device_confirm_or_second_code`, not a
registered session. No OTP or account PIN is retained in these docs.

**Latest comparison-lab result:** the operator-selected historical number was
retried once using the corrected JSON while preserving its saved keys/attempt.
HTTP 200 / `too_many_all_methods` imposed a 32417-second retry/SMS/voice wait.
The cooldown is persisted and no further send was made.

**Previous number's result:** correcting one miscopied byte in the APK-derived
token salt changed HTTP 200 / `bad_token` to HTTP 200 / `sent` on the same
attempt. The 1123-byte ENC/empty-H request returned 347 bytes in 3219 ms,
with SMS wait 125 seconds and voice wait 65 seconds. Registration now awaits
the SMS code. This confirms server-reported dispatch, not handset receipt,
completed signup, or messaging. Only the token changed; keys/history are intact.

**Previous number:** one APK-derived unknown-build-properties
User-Agent experiment changed the response from HTTP 200 / `bad_param` to
HTTP 200 / `format_wrong` (64-byte response). The saved attempt and account
keys are unchanged. This is a different refusal, not accepted signup or OTP
delivery. See the [experiment sequence](docs/protocol/http-post-analysis.md).
APK-only follow-up traced this refusal to the official client's invalid-mobile-
number-for-country dialog (resource `0x7f123741`). The request already separates
country code and national number correctly. No further live request was sent;
a justified input correction remains necessary before another test.

**Historical offline baseline:** G0 complete (repository, dependency decisions, baseline).
G1 partially complete offline (codec, framing, Noise adapter, certificate
binding, schemas, reference manifest, **static APK observations**). **G2 not
attempted**: the Android registration payload encoding, transport port/layer
and login payload values are `Unknown` (see `docs/protocol/unknowns.md`). No
live server contact had been made and no OTP had been requested at that baseline.

**Owner-approved integration update:** registration HTTPS now selects
rustls/ring instead of the alpha RustCrypto provider. The optional
`experimental-signal-crypto` feature supplies an upstream-backed XEdDSA
certificate-signature verifier. This is not a complete Signal messaging
backend; trust-root evidence, key provisioning, sessions and live interop
remain unfinished. The historical offline inventory below predates this update.

The [persisted registration lab](docs/protocol/registration-lab.md) now connects
key generation/signing, encrypted persistence and comparison request assembly.
It is explicitly lab-profile-only; default registration and messaging remain blocked.
The opt-in CLI feature `experimental-signup` now exposes `lab-signup` for
single-action operator-authorized HTTPS attempts. Ordinary commands remain
blocked. Two prior operator-authorized attempts returned `ServerRefused` without
a recorded HTTP reason; neither established signup or OTP delivery. The editable
[request profile](crates/whatsapp-api/src/registration/request.rs) now exports to
private JSON, with offline preview and redacted HTTP diagnostics for future
explicit actions. [Outbound POST analysis](docs/protocol/http-post-analysis.md)
has started; it is not a complete traffic audit. Server acceptance is unverified.

One ownership-confirmed, one-shot legacy diagnostic on 2026-09-18 UTC reused
the saved identity and returned HTTP 200 / `fail` / `bad_param`. The actual
reason now survives restart. The offending field is unknown; no OTP delivery
or successful signup is confirmed. Known refusals and cooldowns remain ineligible
for that legacy-only action, and HTTP-library retries are explicitly disabled.

**Key static findings (2026-09-17):** the pinned client contains classic
`Noise_XX`/`Noise_IK` patterns **and nine ML-KEM512 hybrid Noise patterns**.
If the server requires a hybrid handshake for primary devices, the current
`snow`-based adapter cannot be used (ADR 0004 addendum). A separately pinned,
WhatsApp-signed 2.26.37.70 comparison statically confirms `WA 06 03` and the
registration URL/form-body construction and JSON response vocabulary. Its
compiled Noise selector defaults to classic mode, though remote configuration
can select ML-KEM and remains unobserved. Its built-in socket path uses raw
TCP for primary/fallback gateways on 443/5222, and TLS only for numbered-edge
port 443; the port-80 fallback is raw TCP with a bounded HTTP/1.1 chunked
`POST /chat` wrapper. These comparison findings do not replace evidence for
the active 2.26.36.75 profile.

## Verified in this repository (offline)

| Area | Evidence |
| --- | --- |
| Toolchain and dependency contract | `rust-toolchain.toml` 1.94.1; `cargo run -p xtask -- dependency-audit`: no denied crates in `whatsapp-api`, `whatsapp-api +experimental-noise`, `whatsapp-cli` (report: `docs/protocol/dependency-report.md`) |
| Reference APK pinned | `com.whatsapp` 2.26.36.75 / 263607533, sha256 `ce7468…922e9`, signing cert `39:87:D0:…:5D:44`; reproduced by `xtask reference-manifest --check` |
| APK static observations | `xtask apk-strings`: 449,216 dex strings in pinned 2.26.36.75; hosts, 12 Noise pattern names, registration vocabulary, ClientPayload field constants. `apk-xref`/`apk-callers`/`apk-smali`: structural method and caller analysis, validated on separately hashed 2.26.37.70. |
| Token dictionary | `xtask apk-tokens --check`: ordered dictionary recovered from `classes2.dex` bytecode is identical to the committed fixture → `binary.tokens` is client-observed; version byte still ReferenceOnly |
| Store | encryption at rest, AAD row binding, wrapped data keys with wrong-key detection, master-key rotation, second owner rejected, generation fencing, atomic rollback, journal replay/ack/trim |
| Crash matrix (docs/design/README.md 19.2) | `tests/crash_matrix.rs`: rows 1, 2, 3, 9, 10 (rotation) and the second-process ownership test pass; rows 4–8 and 11 need a transmission path / Signal backend / I/O fault injection |
| Codec | independent hand-written fixture, JID kinds, packed strings, bounds, duplicate attributes, zlib bomb; `tests/codec_props.rs` property round trips (found and fixed an odd-length hex padding overflow), decoder never panics on arbitrary bytes, framing under arbitrary fragmentation |
| Transport | framing fragmentation/cancellation tests; Noise XX handshake + records against a snow responder; complete version-scoped classic/ML-KEM selector map with tested classic compiled defaults; typed comparison-only `WA 06 03` header and built-in gateway/edge raw-TCP/TLS map; cert chain binding incl. "valid-looking chain that does not bind to the remote key"; opt-in upstream Signal signature verifier; feature-gated HTTPS with explicit rustls/ring provider, validation on, no redirects/retries/proxies, deadlines and bounded streaming responses |
| Registration | state table enforced; handles bound to attempt/generation/step; answers never persisted; prerequisite persistence is atomic with the pre-request state; elapsed cooldowns resume without regenerating keys; ambiguous outcomes have a non-repeating reconciliation seam; successful account credentials commit atomically with `Registered`; version-scoped 2.26.37.70 endpoint catalog, deterministic common-field, `/v2/code`, `/v2/register`, `/v2/security`, public key-bundle encoding, APK-backed `/v2/code` token derivation, and fresh-install UUID/token prerequisite generation; conservative comparison HTTP flow including the account-PIN branch is tested end-to-end with a mock transport (not installed for the active profile) |
| Outbox | idempotent enqueue, conflict, limits, cancel rules, uncertain delivery representation |
| Client | durable enqueue → status → events → ack → restart over the same store |
| CLI / HTTP control plane | `keygen`, `store init/inspect`, `send`, `status`, `events --ack`, `connect`; loopback `serve` with health/profile/capability and comparison transport/registration wire-profile endpoints plus structured gated-operation errors |

Test count: 227 (`cargo test --locked --workspace --all-features`). Formatting,
all-feature/all-target Clippy with `-D warnings`, strict rustdoc, package
verification, and protobuf regeneration checks pass on the continuation
machine (Rust 1.98.1). The repository and CI
remain pinned to Rust 1.94.1; that version was not rerun on this machine.
The newer-toolchain chunk-iterator warnings were resolved without changing
encoding behavior.

## Blocked / not started

- Registration driver, transport connect, login: need Android-profile
  evidence (`docs/protocol/evidence-ledger.md` rows marked `Unknown`).
- Signal backend: owner approved AGPL-component evaluation (ADR 0005);
  implementation and interoperability gates remain open.
- HTTPS now uses an explicit rustls/ring provider under the owner-approved
  native TLS exception (ADR 0003). Registration remains experimental behind
  `experimental-registration-https`; no live success is claimed.
- Inbox, devices, groups, media, message operations, bridge adapter: gates
  G3–G6; modules intentionally not created yet.
- Data-key re-encryption, backup/restore, and I/O-fault injection for the
  remaining crash-matrix rows.
- `crates/tzibbur-api` and `crates/bridge` are not in this repository; the
  adapter contract could not be inspected.

## Suggested next steps (in gate order)

The [compact APK research snapshot](docs/protocol/apk/README.md) now preserves
21 mapped protocol classes (149 methods), the comparison token dictionary,
and provenance/checksums in 564,315 bytes. The media-free export is bounded
to 2 MiB and can be regenerated or verified with `xtask apk-research-bundle`.
It is comparison-only structural disassembly, not a complete decompilation
or proof of live protocol behavior.

Continuation (2026-09-17): the supplied local APK was verified as the exact
2.26.37.70 comparison hash. Its frame writer uses a three-byte big-endian
length and rejects bodies at 16 MiB; server Noise record sizes remain open.
The Rust bytecode tool now displays `const/high16` values, with a regression
test covering this bound and signed bit patterns. No extra tools were needed.

1. Static: obtain exact pinned 2.26.36.75 and reproduce the comparison
   build's dictionary-version / `WA` header and registration findings; then
   continue into handshake selection and the trust root
   (`docs/protocol/apk-analysis.md`).
2. Integrate and validate the approved Signal candidate (ADR 0005); it also gates real XEdDSA
   certificate verification, for which no independent vectors are available
   offline yet.
3. With a controlled test number and operator consent: capture one
   registration and one handshake to move `registration.*`, `transport.*`
   and `noise.*` rows to `Observed`; only then implement the registration
   driver and `connect`.
4. Resolve whether the server requires the ML-KEM512 hybrid handshake
   before investing further in the `snow` adapter.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
cargo package --locked -p whatsapp-api --allow-dirty
```
