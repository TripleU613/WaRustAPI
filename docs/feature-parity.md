# Feature parity tracking

2026-10-13. One row per WhatsApp feature area named in the project goal, with
what is true now. This is an index, not evidence: each row links the document
that carries the APK build, method and confirmed / inferred / observed /
unknown labels. When a row and its linked document disagree, the document wins
and this row is stale. Update the row in the same change as the work.

**States.** **live** = seen working against the real server; **acked** = the
server accepted it, handset result unconfirmed; **offline** = implemented and
tested locally only; **partial** = some paths implemented; **modeled** = types
exist but nothing sends/applies it; **absent** = not implemented;
**parked** = deferred by the owner.

**Exposure.** CLI = `whatsapp-cli` command (see [cli.md](cli.md)). HTTP: by
default `whatsapp-cli serve` serves only the service/profile endpoints, `connect`
and `registration begin`; with `--store-routes` it also serves thirteen read-only
store inspections ([HTTP API parity](#http-api-parity)). Every other operation in
[openapi.json](api/openapi.json) is `x-transport: cli`. Library = public
`whatsapp_api` items listed in each operation's `x-library`.

## Account, registration and session

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Primary registration (SMS request, verify, 2FA PIN) | live (lab recipe): code request, OTP and DOB consent; account-PIN (`verify-pin`) and email-OTP branches offline | `lab-signup …` | [registration-lab](protocol/registration-lab.md), [after-otp](protocol/after-otp.md) | Default `registration begin` stays blocked until the pinned profile is observed; device_confirm_or_second_code, wa_old and passkey unimplemented; a wrong OTP (`mismatch`) and a `/v2/code` `ok`/`attached` answer stay blocked because their APK consumers are untraced ([audit](protocol/registration-safety-audit.md#plausible-not-verified)) |
| Login (Noise XX, certificate, primary payload) | live | `lab-transport`, `listen` | [messaging-transport](protocol/messaging-transport.md) | Pinned 2.26.36.75 profile equivalence unobserved |
| Prekey upload and replenishment | live | `prekeys`, `listen` | [messaging-completion](messaging-completion.md) | KEM prekey upload untraced (see PQ row); no error backoff (local deviation) |
| Reconnect after recoverable loss | offline | `listen` | [messaging-transport](protocol/messaging-transport.md#reconnect-budget-stability-2026-10-13) | Refusals never retry by design; budget resets only after a stable connection; APK reconnect schedule untraced (local bound); no live reconnect observed |
| Two-step verification | live (set) | `two-factor` | [registration-lab](protocol/registration-lab.md#two-step-verification-and-resolving-an-unknown-outcome-2026-09-24) | Change/disable not live-verified; the first set attempt stays Sent/unknown |
| Ban appeal | partial: token recovered, nothing submitted | `ban-appeal` | [ban-appeal](protocol/ban-appeal.md) | Owner's decision; SubmitAnAppealForIP doc_id not recovered; named-operation acceptance unverified |
| Account recovery / re-registration of a banned number | absent | — | — | Out of scope without owner authority |
| Server AB properties / feature configuration | partial: one read sent live 2026-10-08, outcome unknown (stream ended); no values applied | `abprops-query` | [feature-configuration](protocol/feature-configuration.md) | A settled reply; effective values gate other rows (properties 2983, 23639) |

## Encryption and identity

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Classic Signal direct/group sessions | live | `lab-transport`, `listen`, `outbox-send` | [messaging-transport](protocol/messaging-transport.md), [groups](protocol/groups.md) | — |
| ML-KEM-1024 hybrid (0x44) sessions: send to verified PQ bundles, fan-out, refresh, inbox receive | offline | via `listen`, `outbox-send`, `actions --show` | [hybrid-outbox](protocol/hybrid-outbox.md), [hybrid-fanout](protocol/hybrid-fanout.md), [hybrid-identity-refresh](protocol/hybrid-identity-refresh.md), [kem-prekey-inventory](protocol/kem-prekey-inventory.md) | No KEM prekey generation/upload (untraced); `pqsupport` producer unknown; PQ upgrade job unported; companion/attestation PQ; no live hybrid message |
| Peer identity change (incoming, refresh, outgoing replacement) | offline; directory reads live (`peer-keys` 2026-10-07; automatic identity_refresh read 2026-10-09) | `peer-identity`, `peer-keys`, `peer-identity-query`, `identity-conflicts`, `actions`, `outbox-suspended` | [peer-identity](protocol/peer-identity.md), [identity-refresh-application](protocol/identity-refresh-application.md) | Native attestation state persistence; non-E2EE routing; no live identity change applied |
| Safety numbers / manual verification | partial offline | `fingerprint-display`, `fingerprint-compare`, `peer-verification` | [safety-number](protocol/safety-number.md) | Active scheme and complete inventory selection unverified |
| Trusted-contact (privacy) tokens | partial: incoming `privacy_token` stored and attached to presence subscribe, offline | — | [presence](protocol/presence.md), [token-issue](protocol/token-issue.md), [identity-token-refresh](protocol/identity-token-refresh.md) | Issuing and refreshing our own tokens traced statically only, not implemented |

## Messages

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Direct text (PN or LID) | live, incl. normal outbox (`send` + `listen --send-outbox`: delivery and read receipts 2026-09-19) | `send`, `outbox-send` | [messaging-transport](protocol/messaging-transport.md), [messaging-completion](messaging-completion.md) | `outbox-send` test: read receipt and handset confirmation pending |
| Replies / quotes | live (text); quoted media send and receive offline | `reply`, `outbox-send` (direct and group JSON replies with text, image or voice note) | [messaging-transport](protocol/messaging-transport.md#quoted-replies-comparison-2263771), [quoted-media](protocol/quoted-media.md) | Live quoted media; quotes of another chat (status replies) ignore `remoteJid` |
| Reactions | direct live; group **not rendering** (cause unknown) | `react` | [reactions](protocol/reactions.md), [groups](protocol/groups.md) | Group reaction rendering |
| Polls and votes | direct live | `poll`, `vote` | [poll-votes](protocol/poll-votes.md) | Group polls/votes not ported: the normal group outbox blocks them as unsupported content; group poll stanza untraced |
| Edits | receive own-device text/caption edits offline; outgoing **modeled** only | — | [message-edit](protocol/message-edit.md), [media-caption-edit](protocol/media-caption-edit.md) | Peer edit window needs property 2983; outbox has no edit sender ([why](protocol/unknowns.md#outgoing-revoke-and-edit-2026-10-13)) |
| Revokes (delete for everyone) | receive direct peer/own-device offline; outgoing **modeled** only | — | [sender-revoke](protocol/sender-revoke.md), [sender-revoke-window](protocol/sender-revoke-window.md) | Physical deletion; group/admin/status revokes; native time/type policy; outbox has no revoke sender ([why](protocol/unknowns.md#outgoing-revoke-and-edit-2026-10-13)) |
| Keep in chat | acked 2026-09-20 (delivered; handset unconfirmed); own-device keep sync and group keep send offline | `keep` | [chat-actions](protocol/chat-actions.md#4-keep-in-chat), [own-device-keep](protocol/own-device-keep.md), [group-keep](protocol/group-keep.md) | Handset display; permission/expiry application; live group keep |
| Disappearing messages | per-chat timer acked 2026-09-20 (delivered, read receipt; handset unconfirmed); default timer offline | `disappearing`, `disappearing-default` | [chat-actions](protocol/chat-actions.md#3-disappearing-messages), [own-device-timer](protocol/own-device-timer.md) | `disappearing-default` never sent live; after-read mode, full setting application; our sent messages carry no `contextInfo.expiration` (sender producer untraced) |
| Stickers, video, video notes, documents (receive) | offline | `media-list`, `media-open`, `media-save` | [stickers](protocol/stickers.md), [video-media](protocol/video-media.md), [document-media](protocol/document-media.md) | No live sample; sending unsupported; Lottie and directPath-only retrieval unsupported |
| Ordinary incoming image and voice note | image live (downloaded, authenticated, shown in mpv); voice note offline (no live sample) | `media-list`, `media-open`, `media-save` | [messaging-transport](protocol/messaging-transport.md#persistent-receiving-and-ordinary-incoming-media) | Live voice-note sample and audible playback |

## Receipts and delivery

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Delivery confirmations for received messages | live | `listen` | [messaging-transport](protocol/messaging-transport.md) | — |
| Read receipts (manual, automatic, batched) | acked (explicit direct 2026-09-19, group 2026-09-25); automatic and batched offline | `mark-read`, `listen`, `receipts` | [read-receipt-batching](protocol/read-receipt-batching.md), [messaging-completion](messaging-completion.md) | Handset blue ticks unconfirmed; fresh-message auto-read and multi-ID batch ACK unverified; native privacy/read-self policy unported |
| Incoming receipt facts (direct, group, aggregate) | direct delivery/read on our sends live (2026-09-19; outbox media `delivered=true`); device-qualified, group-participant and aggregate projection offline (no qualifying live sample) | `status`, `outbox-versions`, `group receipts`, `status-viewers` | [direct-receipts](protocol/direct-receipts.md), [group-receipts](protocol/group-receipts.md) | Native privacy/read-self policy |
| Retry receipts we send | live (server-acked); PN/LID once-only offline | `listen`, `receipts` | [messaging-transport](protocol/messaging-transport.md#retry-once-only-across-pn-and-lid-2026-10-13) | PN redelivery of a LID-first message without alias attributes stays separate |
| Answering a peer's retry receipt | lab group-send window only, offline | `lab-transport` group send | [groups](protocol/groups.md), [retry-answers](protocol/retry-answers.md) | Normal runtime does not answer; native resend policy not traced in-repo |

## Media and voice

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Image send | live (lab path handset-confirmed); normal outbox acked with delivery receipt | `lab-transport --message` (JSON `image`), `outbox-send` | [messaging-transport](protocol/messaging-transport.md), [messaging-completion](messaging-completion.md) | PNG only; no caption/thumbnail; normal-outbox handset display unconfirmed |
| Voice note send | live (receipt; playback unconfirmed) | `lab-transport --message` (JSON `voice_note`), `outbox-send` | same | Audible playback unconfirmed; Ogg/Opus only; waveform omitted |
| View-once image send and single local opening | live send; incoming ciphertext download live (one GET); local viewer and opened receipt offline | `lab-transport --message` (JSON `image` + `view_once`), `lab-download-view-once`, `lab-open-view-once` | same | Opened (played) receipt after viewing implemented but never sent live; no fresh incoming sample; group view-once refused by `outbox-send` |
| Video/document/sticker send, HD variants | absent | — | [why](protocol/unknowns.md#document-and-video-sending-2026-10-13) | Stanza `enc mediatype` for document/video untraced (key domains and schemas are known); thumbnails untraced |
| Calls (voice/video) | absent | — | — | Not researched |

## Groups, communities, channels, status, broadcast

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Group receive/decrypt | live | `listen` | [groups](protocol/groups.md) | — |
| Group send (sender-key fan-out) | live (lab); normal outbox offline | `lab-transport`, `outbox-send` | [groups](protocol/groups.md), [normal-group-outbox](protocol/normal-group-outbox.md) | Own-device inventory and feature selection |
| Group management (create, info, refresh, add/remove, promote/demote, subject, description, invite, leave) | create reply observed 2026-09-22; info and batch refresh live (2026-10-07); add/remove/promote/demote/subject/description/invite/leave not live-verified | `group …` | [groups](protocol/groups.md), [live record](protocol/groups.md#live-record), [queued replies](protocol/groups.md#apply-queued-replies-during-receiving-2026-10-07) | Live verification per action; the first create (2026-09-20) preceded stream:error 503 and the account's 403 suspension (causal role unknown); unsolicited group notifications unverified |
| Communities | absent (complete group replies classify parent/linked_subgroup/default_subgroup/general_chat) | — (`group list --json` shows group_type/linked_parent) | [groups](protocol/groups.md#complete-reply-group-classification-and-subject-alias-boundary-2026-10-07) | Community create/link/announcement not researched |
| Channels (newsletters) | parked | — | — | Owner deferred |
| Status receive, viewers, likes received | live (viewers observed 2026-09-24) | `status-viewers` | [status](protocol/status.md) | Viewer receipts were refused, ending `listen`, from 2026-10-10 until the 2026-10-13 offline fix ([details](protocol/status.md#views-status-receipts-observed-2026-09-24)); not re-verified live |
| Status post text | live | `lab-transport --message` (private JSON `status`) | same | — |
| Status post image | acked as text earlier; media form not yet live | `lab-transport --message` (private JSON `status`) | same | Live check |
| Status like send, audio, HD dual upload | absent | — | same | — |
| Broadcast lists | partial: ordinary-list delivery confirmations offline; list send/management absent | — | [broadcast-dispatch](protocol/broadcast-dispatch.md), [broadcast-receipt](protocol/broadcast-receipt.md) | Effective status-gate property application; list send/creation not researched |

## Contacts, presence, privacy, settings, app state

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Contacts (local book, check, status, sync, app-state publish) | check (2026-09-20, 2026-10-09), full sync (2026-09-22) and app-state publish (2026-09-24) live; local book and status local-only | `contacts …` | [chat-actions](protocol/chat-actions.md#7-contacts), [status](protocol/status.md#contact-registration-the-one-remaining-lever) | In/out/invalid state parity and refresh/backoff unported; fresh post-fix replies unverified |
| Presence subscribe, availability, typing | live (typing handset-confirmed) | `lab-transport --subscribe` / `--typing` / `--typing-state` | [presence](protocol/presence.md) | Recording/incoming indicators unconfirmed; `last="deny"` cause unestablished; group presence count unimplemented |
| Display name, about text | name presence written (no reply); pushNameSetting app-state acked 2026-09-25; About acked 2026-09-28 (session export); handset unconfirmed | `push-name`, `about`, `lab-transport --push-name` | [presence](protocol/presence.md), [session-export-review](protocol/session-export-review.md) | Handset/companion display unconfirmed; official client's name-presence timing untraced |
| Profile picture | absent | — | — | — |
| Block/unblock, block list | live | `block`, `unblock`, `blocklist` | [chat-actions](protocol/chat-actions.md) | — |
| Privacy settings (last seen, photo, groups…) | absent (no mutation command) | — | [read-setting-sync](protocol/read-setting-sync.md) | SMAX `<iq xmlns="privacy"><privacy><category name value/>` is confirmed for `readreceipts`, but property 23639 selects MEX vs SMAX (effective value unknown); other category/value names are listed by `LX/I4X` ([status](protocol/status.md#status-privacy-the-surface-traced-but-not-wired)) but their setter callers are untraced |
| Per-chat privacy ("limit sharing") | acked 2026-09-20 (delivered, read receipt; handset unconfirmed) | `chat-privacy` | [chat-actions](protocol/chat-actions.md#5-advanced-chat-privacy-limit-sharing) | Only v1 sent; which generation (v1/ACP2) the sender picks is unknown |
| Spam report | offline (never sent; reply shape unknown) | `report` | [chat-actions](protocol/chat-actions.md#2-report-xmlnsspam) | Reported-message children unknown; never send against the lab contact |
| Clear chat | offline, local-only | `clear-chat` | [chat-actions](protocol/chat-actions.md#6-clear-chat-and-app-state-sync) | clearChatAction app-state patch unimplemented (boolean semantics unknown) |
| App-state sync (contact action writes, 409 catch-up) | live | `contacts fetch-app-state`, `contacts publish-app-state`, `push-name` | [chat-actions](protocol/chat-actions.md) | Only contactAction (critical_unblock_low) and pushNameSetting (critical_block) written; clearChat/star/mute/pin/archive not exposed; snapshot reconciliation unfinished |

## Devices, history, sync

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Peer device lists and companion admission | device-list query live (2026-09-19, primary-only reply); companion admission offline | `devices`, `device-query` | [companion-directory-addition](protocol/companion-directory-addition.md), [messaging-transport](protocol/messaging-transport.md#contact-device-list-synchronization-comparison-2263771) | First-seen companions, raw-ID changes |
| Own companion pairing (QR) | parked | — | [cross-device-sync](protocol/cross-device-sync.md#primary-pairing-completion-and-identity-provenance) (static research only) | Owner deferred; `lab-pair` is a two-profile text check, not pairing |
| Own-device message sync (held sessions) | partial offline | `listen` | [own-device-text](protocol/own-device-text.md), [cross-device-sync](protocol/cross-device-sync.md) | Own-account device sync, history transfer |
| History sync / message history notices | partial: notice inspection and control event | `lab-inbox --inspect` | [cross-device-sync](protocol/cross-device-sync.md), [history-mode](protocol/history-mode.md) | History is not applied |

## HTTP API parity

The library and CLI cover the rows above; HTTP covers little of it. Default
`serve` is store-less and unauthenticated (loopback by default) and serves only
secret-free profile and capability data.

**2026-10-13, opt-in store routes:** `serve --store-routes --token-file <new
path>` (experimental-transport build) also serves thirteen read-only, redacted
inspections: `GET /v1/store` (the `store inspect` report, added later the same
day), `/v1/receipts`, `/v1/prekeys`, `/v1/outbox/suspended`,
`/v1/actions` (list form only), `/v1/identity-conflicts`,
`/v1/outbox/group-preparations`, `/v1/outbox/{local_id}/versions`,
`/v1/diagnostics/own-lid`, `/v1/inbox/undecodable`, and the counts-only `/v1/groups/receipts`,
`/v1/groups/key-queries` and `/v1/groups/refresh-status` (one ID query
parameter each, added the same day), the same library calls and JSON as the CLI. Inspections whose output carries raw
addresses (for example device queries, message status facts, contacts,
blocklist) or message content (events) are not served. The access-control design is the
conservative one this page previously left open: loopback only (even with
`--allow-non-loopback`), a fresh 256-bit bearer token written only to a new
mode-0600 file (never overwritten, never printed or logged, removed on
Ctrl-C/SIGTERM/SIGHUP), the store opened per request so other commands can use
it in between (a held store answers 503; a command starting during a request
can fail to open and must be rerun), and nothing that sends or changes
protocol data. Caveats and response codes: [API README](api/README.md#store-routes-opt-in). Every
other store-backed operation remains CLI-only (`x-transport: cli`); in
particular no send, queue, receipt-transmitting or live operation is served.
Extending the routes is mechanical but each one needs its output checked for
redaction first. No document may describe a CLI-only operation as served over
HTTP.
