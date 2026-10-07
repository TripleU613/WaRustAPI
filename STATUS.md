# Status — 2026-10-07

What is true now. The dated experiment log that led here, including every
superseded conclusion and dead end, is in
[`docs/status-history.md`](docs/status-history.md); per-area evidence is in
`docs/protocol/`. Labels: **live** = seen working against the real server,
**acked** = the server accepted it but a handset result is not confirmed,
**offline** = tested locally only.

Receive recovery (2026-10-07): the saved account recovered its blocked direct
and group texts, then delivery-confirmed both on live redelivery. The two
recovered-message read receipts received matching server ACKs. Current receipt
state: eight acknowledged reads and one acknowledged retry. Handset blue ticks
and fresh-message automatic-read validation remain unconfirmed.

The causes in our implementation were blanket refusal of changed peer pins
and group pairwise selection of a PN ratchet despite a held LID session. An
explicit APK-compatible incoming policy now commits an authenticated peer
replacement atomically with encrypted prior-key/session history and an
`IdentityChanged` event. The library default remains `pinned`; profile `l`
selects `apk_authenticated` in its preserved private corrected configuration.
No local account identity, send ID or reservation was reset. Automatic reads
also cover eligible authenticated duplicates, with once-only write-cut coverage.
Human safety-number verification remains unfinished. The legacy Java algorithm,
phone-identifier normalization, key-set ordering and QR content are now traced;
modern native schemes and active selection remain unknown.
[Safety-number evidence](docs/protocol/safety-number.md). Outgoing key-bundle
mismatches now suspend the original queued intent before ratchet use;
`outbox-suspended` inspects them offline. Independent
`transport.outgoing_identity_changes=apk_authenticated` now stages a signed
directory-bundle replacement and commits prior-state history, the identity
event and original send reservation atomically. Explicitly selected suspended
intents can use their saved reply without another key query; invalid signatures,
own-account changes, stale-pin rollback and uncertain sends remain blocked.
The canonical outgoing policy remains pinned; live replacement is unverified. `peer-identity` now inspects both namespaces offline;
the recovered direct peer has different PN/LID pins, each matching its own
held session. Authenticated history connects the PN pin to the updated LID pin.
A 25-second directory-only live check obtained a valid signed current bundle
matching the recovered LID pin and differing from the PN pin. Peer pin/session
summaries stayed unchanged. One private test draft awaits authorization; no
new chat was sent.

Private state was recovered from the former checkout in the desktop Trash,
with originals preserved. The selected profile has no eligible chat sends or
queued actions. The successful 45-second receive window and 20-second read
window sent no queued chats. Evidence: [peer identity](docs/protocol/peer-identity.md),
[group session selection](docs/protocol/groups.md#group-pairwise-pnlid-session-correction-2026-10-07),
[read recovery](docs/protocol/messaging-transport.md#authenticated-duplicate-auto-read-correction-2026-10-07).
The earlier interval with no incoming messages is still unexplained.

## Accounts

Registration uses the recipe in
[registration-lab](docs/protocol/registration-lab.md#working-recipe)
(`enc_empty_h`, 2.26.37.70 User-Agent and token material).
`lab-signup resend-code` asks for a new code on an existing pending attempt.

## Messaging

| Capability | State | Where |
| --- | --- | --- |
| Direct text, PN or LID addressed | live | [messaging-transport](docs/protocol/messaging-transport.md) |
| Direct image, voice note, view-once image | live (receipt confirmed; playback not) | same |
| Replies, reactions, polls (direct) | live | same, [chat-actions](docs/protocol/chat-actions.md) |
| Receive + decrypt direct and group messages | live | [groups](docs/protocol/groups.md) |
| Read receipts, group (`mark-read --chat …@g.us`) | acked 2026-09-25; handset "Read by" unconfirmed | [messaging-transport](docs/protocol/messaging-transport.md) |
| Automatic read receipts (`transport.mark_read`, `listen`) | offline | same |
| Group send (sender-key fan-out) | live — operator read it | [groups](docs/protocol/groups.md) |
| Group replies | live — shown on the handset (the operator reacted to one) | same |
| Group reactions | **not rendering**: payload, stanza and target match the official client's, but the handset shows nothing and the server's ack carries no `count`; cause unknown | same |
| Retry receipts: sending ours when we cannot decrypt | runtime retry server-acknowledged 2026-10-06; redelivery still undecryptable | [messaging-transport](docs/protocol/messaging-transport.md) |
| Retry receipts: answering theirs after a fan-out | offline; one live retry was received before the answer existed | [groups](docs/protocol/groups.md) |
| Status: receive | live | [status](docs/protocol/status.md) |
| Status: post styled text | live — operator saw it | same |
| Status: post image | acked as `type=text`; now `type=media`, **not yet sent live** | same |
| Status: viewers (delivered/read/played) and likes received | live (`status-viewers`) | same |
| Status: sending a like, HD dual upload, audio | not implemented | same |
| App-state writes (contact action) incl. 409 catch-up | live | [chat-actions](docs/protocol/chat-actions.md) |
| Block/unblock, contact check, disappearing, keep-in-chat | live | same |
| Two-step verification set/change/disable | live (set) | [chat-actions](docs/protocol/chat-actions.md) |
| Display name (`presence name=…`) | written live, handset unconfirmed | [presence](docs/protocol/presence.md) |
| Display name app-state record (`pushNameSetting`, `critical_block`) | live — accepted at version 1 | same |
| About text (`iq xmlns=status`, `whatsapp-cli about`) | server-accepted 2026-09-28, as reported in the session export; handset unconfirmed | [presence](docs/protocol/presence.md) |
| Typing / recording indicators (`chatstate`) | typing handset-confirmed in the session export; recording and incoming indicators unconfirmed | same |
| Presence subscribe, incoming availability / last seen | live — server answered with the peer's presence | same |
| Prekey upload and replenishment | live | [messaging-completion](docs/messaging-completion.md) |

A single undecryptable incoming message no longer ends a session; it degrades
to `Undecryptable` and we send a retry receipt. Direct messages from a LID sender now decrypt with the LID
session our group sends established; that was why one of the operator's
messages stayed unreadable for four days (fixed 2026-09-28, verified by offline
replay). A group payload addressed only to this device (such as a
`messageHistoryBundle`) is now recorded and acknowledged instead of being
redelivered at every login.

## Not done

- **Stickers and video notes.** Field numbers are decoded from the APK
  (`stickerMessage` 26, `ptvMessage` 66, `VideoMessage` inner fields);
  StickerMessage's 22-field schema and image-key domain are now traced in
  2.26.37.71. Ordinary URL-bearing WebP sticker listing, sealed references,
  authenticated retrieval and local visual playback are implemented offline.
  Lottie, directPath-only retrieval and sending remain unsupported; no incoming
  sticker/PTV stanza or sticker display has been captured. Ordinary URL-bearing
  MP4 video/video-note listing, caption/reference events and video-domain
  authenticated retrieval are now implemented offline; live playback, circular
  rendering, HD variants and sending remain unverified or unsupported. One rebuilt
  60-second receive window authenticated/activated and stopped cleanly, with the
  20-event journal and nine acknowledged receipts unchanged; no fresh sample.
  [Sticker evidence](docs/protocol/stickers.md), [video evidence](docs/protocol/video-media.md),
  [apk-analysis](docs/protocol/apk-analysis.md).
- Calls; profile picture.
- Privacy settings IQ (namespace unknown).
- Channels and QR/companion pairing — parked by the owner.
- Companion-device fan-out and own-account device sync.
- History-notice handling. APK 2.26.37.71 identifies the formerly unexplained
  49-byte `[102]` envelope as `messageHistoryNotice`; offline inspection validates
  supported metadata and reports redacted counts without applying history. Saved-account offline inspection
  confirmed the 49-byte notice metadata: one receiver, no non-receivers, context
  present, no optional bundle timestamp or bot metadata. The APK group-notice handler
  and metadata requirements are traced. Supported normal group/meta context now
  projects a distinct control event; historical group/broadcast backfill is fixed.
  Canonical offline backfill upgraded this one notice; repetition upgraded zero.
  APK feature selection, rendering, timestamp units and history processing remain
  unverified. [Cross-device evidence](docs/protocol/cross-device-sync.md).

## Offline verification

Native research now locates separate Superpack decoders and V3/V4 fingerprint
symbols in one decoded stream. The ARM64 decoder loads and initializes all
19 JNI methods locally, exiting zero. A bounded first-entry reconstruction
returns native `9/2` / `could not extract file from archive`, with no output;
real host memory metadata did not fix it. Worker tracing now identifies an
oversized-file comparison against zero requested bytes, despite the matching
125448 declared/produced sizes. This partial-range result does not establish
ordinary full extraction failure; bounded full-entry checks have reached wall
limits without output or native refusal. Hash-pinned branch/futex tracing is
available. The unmodified ordinary full decoder now succeeds under QEMU,
returning 80 entries (one duplicate) and 79 valid ARM64 ELFs; a second run
matches all 79 hashes. V3/V4 fingerprint code anchors and containing unwind
ranges are located in reconstructed `libwhatsapp.so`; the algorithms and active
scheme remain unfinished. The helper's previously ineffective threaded-runner
limits are corrected, with eight regression checks including actual guest
execution, resource guards and preflight/output preservation. These tooling
checks make no live account request.
[Evidence, setup failures and remaining warnings](docs/protocol/native-unpacking.md).

529 all-feature tests (`cargo test --locked --workspace --all-features`) and
223 default-feature tests (`cargo test --locked --workspace`) pass on
2026-10-07; Clippy with `-D warnings` is clean. Rust 1.98.1 locally and in CI. Store, codec, crash
matrix, registration state machine and transport coverage are described in
[docs/design](docs/design/README.md) and the per-area protocol docs.

Before any live check, rebuild the release binary: `cargo test` does not
refresh `target/release/whatsapp-cli`, and a stale one once produced a wrong
"cause eliminated" conclusion.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo build --release -p whatsapp-cli --features experimental-transport
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
```


Native safety-number follow-up traces the shared V3/V4 5200-round SHA-512
helper, sorted key concatenation and separate identifier-bound display seed.
A pinned ARM64 slice checker validates iteration/input ordering on synthetic
data with explicitly modeled SHA/MCF operations. Full input producers,
JNI normalization, QR semantics and current scheme/human verification remain
unfinished; no account verification state or live connection changed.
[Evidence and reproducible check](docs/protocol/safety-number.md).


The native identifier callback is now traced back into Java: PN shortening
uses HNy, LID output uses the parsed JID user, and the return conversion
requires ASCII. Native key constructors establish 32-byte sorting versus
33-byte serialized hashing inputs. Java V4 creation and result dispatch now
connect to the recovered routines; the traced result path emits QR version 0.
Current scheme selection, complete device inventory, individual QR fields,
comparison semantics and human verification remain unfinished.
[Callback and dispatch evidence](docs/protocol/safety-number.md#native-to-java-identifier-callback-and-dispatch-mapping).


Native QR descriptors and serialization are now recorded with a reproducible
metadata extractor. Version zero carries serialized keys; version one their
identifier-free digest. V4 comparison resolves peer orientation and checks
keys, LID and additional PN/username/hosted-state conditions; LID/key equality
alone cannot establish verification. Fourteen offline tooling checks pass.
Full comparator primitive/error semantics, UI selection/inventory and the
verified-state action remain unfinished; no account verification was performed.
[QR evidence and schema](docs/protocol/safety-number.md#native-qr-schema-serialization-and-v4-comparison).


Fingerprint comparison primitives now establish NUL/non-ASCII rejection, PN
byte equality, username strncasecmp and orientation-dependent native status
remapping in APK 37.71. The actual native byte leaf passed 862 synthetic
comparisons without modeled callees; all 16 offline tooling tests passed.
V4 comparison is mapped to jvidispatchIOO selector 4. Its Java caller, active
scheme/device inventory and human verification-state commit remain untraced.
No account verification or live request occurred.
[Evidence and reproducible check](docs/protocol/safety-number.md#comparison-primitives-and-orientation-dependent-statuses).


APK 37.71 Java comparison selection now connects KXj.A01 to native V4
selector 4 behind holder/feature gates and an exact zero-result success test.
All three comparison callers are located. Camera/NFC success reaches result
presentation; the traced path does not establish a persisted human-verification
bit. Delayed callbacks, the verification writer and complete scheme/device
inventory remain unfinished. No account state or live request changed.
[Java caller evidence](docs/protocol/safety-number.md#java-comparison-selection-and-scan-result-action).


APK 37.71 verification metadata writers are now traced: the local sender
rejects stale rendered identities, rechecks the stored key under lock, advances
the action sequence and writes existing metadata before companion fan-out.
Inbound peer actions are companion-only with target/key/sequence guards;
history verification has separate presence/key requirements. BWE/BT5 schemas
and ProtocolMessage field 32 are confirmed. Delayed scan callbacks only present
results. Full human-trigger wiring, active fingerprint scheme/device inventory
and client implementation remain unfinished; no account state changed.
[Writer and schema evidence](docs/protocol/peer-identity.md#verification-metadata-writers-and-rotation-guards-3771).


The APK local verification control now traces from saved metadata/rendered
identity to explicit mark confirmation and separate unmark execution. The
writer receives that captured key, allowing its rotation guards to refuse stale
confirmation. Feature/predicate gates and complete fingerprint inventory remain
unverified; client verification metadata remains unimplemented. Two missing
DEX field-write mnemonics are fixed with literal APK instruction regression
coverage. No live account operation occurred.
[Local control evidence](docs/protocol/peer-identity.md#local-verification-control-and-captured-identity-3771).

APK 37.71 manual-verification contact exclusions now resolve to a hosted-set
predicate and a saved contact-state value ==2 (semantic name still unknown).
Hosted device-account type 1 and identity attestation HYBRID_E2EE type 2 come
from separate SQL sources; neither is the native QR HostedState enum. Native
display getter tracing also confirms that QR retains the full LID JID while
its decimal seed uses the parsed LID user. Current account inventory, active
scheme and human-verification implementation remain incomplete. No account
state or live request changed. See the peer-identity contact-exclusion and
safety-number display follow-ups.

Native fingerprint display is now implemented as an offline library primitive
and `fingerprint-display --config` CLI command using explicit private normalized
identifiers and raw public key inventories. Three independent synthetic ARM64
hash-flow fixtures match, including duplicate-key preservation; result labels
state that scheme/input selection is unverified. The calculation does not open
account storage or mark a peer verified. Active scheme, complete device-key
inventory, QR comparison and human verification remain unfinished.

Fingerprint calculation validation passed 535 workspace/all-feature tests,
229 default-feature tests, strict Clippy, warning-free rustdoc, CLI reference,
formatting and repository hygiene. A command-level synthetic calculation
matched the fixture pair without creating the supplied unused store path.
No network request or current-account verification was performed.
