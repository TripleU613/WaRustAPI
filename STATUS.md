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
Human safety-number verification remains incomplete. Native V4 explicit-input
display, binary export/comparison and guarded local manual marks are implemented
and tested offline. Current account/peer scheme and complete inventory selection,
protobuf-C edge equivalence and companion verification synchronization remain
unverified.
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

APK peer fingerprint inventory now traces to a selected device set and a
nonempty identity map with equal cardinality. Feature 0x1da3 excludes hosted
99-device classes; the local key helper has separate primary/companion controls.
`peer-identity` now reports retained E2EE device-key coverage without claiming
complete APK inventory or fingerprint readiness. Current scheme/contact-state
values, source inventory freshness and full own-device synchronization remain
unknown. No account keys or verification metadata were changed.

The rebuilt peer inspector passed on an isolated saved-account copy: PN has
no saved device list; its mapped LID has one retained device/one pin and no
missing pins, but companions_verified=false (primary-only fallback).
Original encrypted database bytes were unchanged. These are saved-state
observations, not a live directory refresh or proof of complete inventory.
Validation passed 536 workspace/all-feature tests and strict Clippy.

Native V4 combined fingerprint protobuf export now supports QR versions 0/1
through the library and paired fingerprint-display --qr-version/--qr-output
flags. Independent synthetic wire fixtures preserve full LID, hosted-state
presence and local PN omission; version 1 uses identifier-free H(K,K).
Exports are private/create-only and do not open account storage. This is binary
comparison data, not a QR image or a verified peer comparison. Active scheme,
complete inventory, scanning/comparison and human-verification commits remain
unfinished; no current-account QR or live operation was performed.

Native QR export validation passed 540 workspace/all-feature and 233 default
feature tests, strict Clippy, warning-free rustdoc and CLI reference. The rebuilt
CLI matched independent 200/229-byte synthetic payloads for versions 0/1,
created mode-600 files and refused repeats without changing them or creating
an account store. A zero-qualified JID test exposed and fixed the new export's
canonical-input guard; the generic JID parser and account state were unchanged.

Local manual verification metadata (offline, 2026-10-07): `peer-verification`
records an explicit human mark/unmark bound to an inspected pin and sequence.
`peer-identity` reports the saved bit/sequence/pin agreement. Authenticated
incoming and signed-directory replacement clear the bit atomically, preserving
sequence; rotation back does not restore it. Tests cover restart, stale/own
refusals, unchanged session/pin and archive rollback. No actual account was
marked. Automated QR comparison and companion synchronization remain unfinished.
[Evidence and operator semantics](docs/protocol/peer-identity.md#local-manual-verification-metadata-2026-10-07).
Validation for this change: 545 workspace/all-feature tests passed; strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, formatting,
regenerated CLI reference and repository hygiene passed (304 local links).
The first suite run caught the new OpenAPI `x-cli` including flags instead of
the command name; corrected to the repository's command-name contract before
rerunning successfully.

Native V4 comparison follow-up (offline, 2026-10-07): 115 synthetic cases execute
actual APK 37.71 comparison/helper/remapper branches with explicit host-modeled
getters and string/key/LID primitives. All 81 hosted-state combinations confirm
that both scanned states absent takes the legacy success branch regardless of
known states. Missing-versus-unequal username results and version-selected key
checks are reproduced. Converter refusal codes are statically attributed.
Whole parsing/orientation/Android string behavior and production scanning remain
unimplemented; no account was marked or changed.
[Executed branches and boundaries](docs/protocol/safety-number.md#executed-native-v4-comparison-branches-2026-10-07).
Validation: all 19 pinned native tooling tests and 36 CLI/API tests passed;
CLI reference regeneration and repository hygiene passed (310 local links).
The final attributed observation artifact also regenerated identically and
passed its three focused preflight/execution checks. Rust runtime code did not
change in this research increment.

Native username primitive (offline, 2026-10-07): the V4 checker now optionally
executes actual APK 37.71 merged MCFStringCompare and the pinned public Bionic
strncasecmp fixture. All 115 core statuses agree; 268 additional accepted-ASCII,
every-position mismatch, prefix, empty and 256-byte cases confirm ASCII
case-insensitive equality plus exact length for these fixtures. Public Bionic
is not APK-bundled or proof of every Android build; object construction and
non-string dependencies remain modeled. QR parsing/orientation/current inventory
and production scanning remain unfinished. No account state changed.
[Native string evidence](docs/protocol/safety-number.md#actual-native-ascii-username-comparison-2026-10-07).
Validation for the native-string increment: all 22 native tooling tests passed
with all four fixtures configured, alongside 36 CLI/API tests, regenerated CLI
reference, formatting and repository hygiene (315 local links). Rust runtime
code did not change; the account database was not opened.

Explicit binary comparison (offline, 2026-10-07): `fingerprint-compare` now
compares private native V4 protobuf against explicit known JSON, with traced
conversion order, both scan orientations and native status remapping. Tests
match all 113 executed non-abort core cases and 268 native string cases;
independent wire fixtures cover both versions/orientations. Result reports
caller-input and bounded native-parser scope; complete protobuf-C edge equivalence,
current inventory and live scanning remain unverified. No account or human
verification metadata is changed by comparison.
[Comparison scope](docs/protocol/safety-number.md#explicit-input-native-v4-binary-comparison-2026-10-07).
Validation: 551 workspace/all-feature tests and 239 default-feature tests passed,
with strict all-target/all-feature Clippy, warning-free Rustdoc, formatting,
CLI reference and repository hygiene (317 local links). Command-level synthetic
version-0/1 fixtures matched without creating an unused account-store path;
malformed payload/private-permission checks passed. Both-scanned-LIDs-matching
priority also passed its focused regression. The initial documentation check
compiled before the new OpenAPI operation was added; completing the contract
and regenerating/rerunning resolved the coverage refusal.

Native parser correction (offline, 2026-10-07): actual APK 37.71 protobuf-C
unpack/init/merge/free now reproduces a 50-case synthetic corpus. Eight prost
acceptance differences were observed; a valid fingerprint plus an unknown group
could previously produce a false match. `fingerprint-compare` now uses a bounded
schema decoder matching native tag/wire/length/scalar/merge rules, validated for
acceptance and all recognized field values. Its parser-authority output reports
corpus validation with other edges still unverified. Unknown opaque fields are
skipped, not round-tripped; current inventory/live scanning remain unverified.
No account store, pin/session or verification metadata was changed.
[Native parser evidence](docs/protocol/safety-number.md#actual-protobuf-c-parsing-and-bounded-decoder-correction-2026-10-07).

Validation (offline): 552 workspace/all-feature tests, 240 default-feature tests
and all 25 fully configured native tooling checks passed, alongside strict
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference. A rebuilt CLI matched both independent version-0/1 fixtures and
returned -8 for the otherwise-valid-plus-group and malformed fixtures; the
unused account-store path remained absent. The corpus test also checks explicit
nested-holder presence, with its final focused rerun passing.
Repository hygiene passed with 323 local links; staged diff whitespace checks passed.

Own fingerprint inventory follow-up (static, 2026-10-07): APK 37.71 own-device
selection is now traced to the cached `devices` directory, strict positive
logout-time exclusion, self LID-to-PN projection with existing-PN precedence,
and later fingerprint-facing address remapping. BNm checks loaded Me presence,
not signed-list freshness. Add/remove paths invalidate both directory caches.
Current membership, authentication and active fingerprint inputs remain unknown;
no account state was opened or changed.
[Own-directory evidence](docs/protocol/safety-number.md#own-directory-source-logout-filtering-and-alias-precedence-2026-10-07).

Validation: the APK hash was rechecked, and an independent exact A08 method
extraction matched the scoped snapshot byte-for-byte. CLI reference regeneration
and its test passed; repository hygiene passed with 329 local links. The initial
snapshot's trailing blank line was removed; final staged whitespace checks passed.
This change adds static research evidence only; runtime code and API/CLI
capabilities are unchanged, and no live account validation is claimed.

Fingerprint contact-state follow-up (static, 2026-10-07): the reusable
`xtask apk-field-refs` command found 17 direct writes to APK 37.71 0Da.A04.
Read/write tracing identifies `external_user_state`, with guest/Cameo,
pending-invitation and normal-user transitions, rather than Signal attestation
or QR HostedState. Complete producer authority and active scheme remain open.
[Contact input evidence](docs/protocol/safety-number.md#fingerprint-contact-predicate-is-external-user-state-2026-10-07).

Validation: 555 workspace/all-feature tests and 243 default-feature tests
passed, including three new APK/DEX regression checks. Strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference passed. Final tool-only tests/Clippy also cover filename sorting.
Command-level no-DEX and malformed-DEX archives refused with empty stdout;
wrong field type produced zero sites. The shared caller search reproduced all
134 previous BNm call-site records; the first byte-for-byte check differed
because ZIP filename iteration order varies, so record sets were compared.
The new field-search command sorts DEX names to make its output reproducible.
No live account or identity-change test was performed in this change.
Two sorted field-search runs matched byte-for-byte. Repository hygiene passed
with 334 local links; final staged whitespace checks passed.

Fingerprint selection follow-up (static, 2026-10-07): APK 37.71's captured/contact
gate selects native V3/V4; the later fresh read selects PN inclusion. Own-key
companion inclusion, empty-to-null strings, stored own username, remote `@`
prefixing and nullable `is_pn_shared` storage are now traced. The earlier
username-availability hypothesis for that Boolean was refuted. Smali branch
annotations now show signed jumps, with regression coverage. Current feature
values, authenticated input producers and native V3 comparison remain open;
no current account fingerprint or verification action occurred.
[Selection evidence](docs/protocol/safety-number.md#scheme-username-and-pn-sharing-input-branches-2026-10-07).

Validation (offline): 556 workspace/all-feature tests and 244 default-feature
tests passed. Strict workspace/all-target/all-feature Clippy, warning-free
Rustdoc, formatting and CLI reference regeneration/test passed. Fresh APK
method output confirms both backward jumps and matches the scoped selection
excerpt. No account state was opened or changed and no live request was sent.
Repository hygiene passed with 339 local links; staged whitespace checks passed.

Native V3 comparison follow-up (static, 2026-10-07): selector 19 now resolves
through the relocated JNI table to 0x71373c, with Java options 1/3 and distinct
PN-or-LID orientation, conversion errors, PN/username presence checks, hosted
omission and result remapping. Generation and scan selection use separate
feature flags. Native execution, MCFEqual/JID semantics and binary comparison
implementation remain open; no account state or live request changed.
[V3 evidence](docs/protocol/safety-number.md#native-v3-scan-dispatch-and-comparison-branches-2026-10-07).

Validation (static/offline): APK and reconstructed-library hashes matched the
pinned values; the option extraction returned one method. Independent fresh
native extraction matched all 424 converter/helper/core instructions in the
scoped snapshot. CLI reference regeneration/test passed; repository hygiene
passed with 344 local links and staged whitespace checks passed. This changes
research/docs only; native execution and live account validation are not claimed.

V3 native execution follow-up (offline, 2026-10-07): 264 synthetic cases now
execute the actual V3 core/optional-value/PN-or-LID helpers and remapper,
validating the traced presence asymmetry, option bits, hosted omission and
error ordering. Ten V3/shared-V4 tests passed, preserving existing V4
observations. Generic object equality and constructors remain modeled;
JID/equality semantics, decoder/orientation integration and runtime V3 binary
comparison remain open. No account or network state changed.
[V3 corpus](docs/protocol/safety-number.md#executed-v3-core-and-remapper-corpus-2026-10-07).

Final validation: CLI reference regeneration/test and OpenAPI library-symbol
check passed. Repository hygiene checked 349 local links; staged whitespace
checks passed. All ten V3/shared-V4 tests passed again after restricting hooks
to the selected scheme. Rust runtime code was unchanged; no live validation
is claimed for this offline research-tool change.
