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

**Exposure.** CLI = `whatsapp-cli` command (see [cli.md](cli.md)). HTTP: only
the service/profile endpoints, `connect` and `registration begin` are served by
`whatsapp-cli serve`; every other operation in [openapi.json](api/openapi.json)
is `x-transport: cli` today. Library = public `whatsapp_api` items listed in each
operation's `x-library`.

## Account, registration and session

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Primary registration (SMS request, verify, 2FA PIN) | live (lab recipe) | `lab-signup …` | [registration-lab](protocol/registration-lab.md) | Default `registration begin` stays blocked until the pinned profile is observed |
| Login (Noise XX, certificate, primary payload) | live | `lab-transport`, `listen` | [messaging-transport](protocol/messaging-transport.md) | Pinned 2.26.36.75 profile equivalence unobserved |
| Prekey upload and replenishment | live | `prekeys`, `listen` | [messaging-completion](messaging-completion.md) | KEM prekey upload untraced (see PQ row) |
| Reconnect after recoverable loss | offline | `listen` | [messaging-transport](protocol/messaging-transport.md#reconnect-budget-stability-2026-10-13) | Refusals never retry by design; budget resets only after a stable connection |
| Two-step verification | live (set) | `two-factor` | [chat-actions](protocol/chat-actions.md) | Change/disable not live-verified |
| Ban appeal | partial: token recovered, nothing submitted | `ban-appeal` | [ban-appeal](protocol/ban-appeal.md) | Owner's decision |
| Account recovery / re-registration of a banned number | absent | — | — | Out of scope without owner authority |

## Encryption and identity

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Classic Signal direct/group sessions | live | `lab-transport`, `listen`, `outbox-send` | [messaging-transport](protocol/messaging-transport.md), [groups](protocol/groups.md) | — |
| ML-KEM-1024 hybrid (0x44) sessions: send to verified PQ bundles, fan-out, refresh, inbox receive | offline | via `listen`, `outbox-send`, `actions --show` | [hybrid-outbox](protocol/hybrid-outbox.md), [hybrid-fanout](protocol/hybrid-fanout.md), [hybrid-identity-refresh](protocol/hybrid-identity-refresh.md), [kem-prekey-inventory](protocol/kem-prekey-inventory.md) | No KEM prekey generation/upload (untraced); `pqsupport` producer unknown; PQ upgrade job unported; companion/attestation PQ; no live hybrid message |
| Peer identity change (incoming, refresh, outgoing replacement) | offline; directory read live | `peer-identity`, `identity-conflicts`, `actions`, `outbox-suspended` | [peer-identity](protocol/peer-identity.md), [identity-refresh-application](protocol/identity-refresh-application.md) | Native attestation state persistence; non-E2EE routing |
| Safety numbers / manual verification | partial offline | `fingerprint-display`, `fingerprint-compare`, `peer-verification` | [safety-number](protocol/safety-number.md) | Active scheme and complete inventory selection unverified |

## Messages

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Direct text (PN or LID) | live, incl. normal outbox with delivery receipt | `send`, `outbox-send` | [messaging-transport](protocol/messaging-transport.md) | Read/played receipt and handset confirmation for outbox send |
| Replies / quotes | live (text); quoted media send and receive offline | `reply`, `outbox-send` (group JSON replies) | [chat-actions](protocol/chat-actions.md), [quoted-media](protocol/quoted-media.md) | Live quoted media; direct JSON/CLI quoted media; quotes of another chat (status replies) ignore `remoteJid` |
| Reactions | direct live; group **not rendering** (cause unknown) | `react` | [reactions](protocol/reactions.md), [groups](protocol/groups.md) | Group reaction rendering |
| Polls and votes | direct live | `poll`, `vote` | [poll-votes](protocol/poll-votes.md) | Group polls unverified |
| Edits | receive own-device text/caption edits offline; outgoing **modeled** only | — | [message-edit](protocol/message-edit.md), [media-caption-edit](protocol/media-caption-edit.md) | Peer edit window needs property 2983; outbox has no edit sender ([why](protocol/unknowns.md#outgoing-revoke-and-edit-2026-10-13)) |
| Revokes (delete for everyone) | receive peer/own-device offline; outgoing **modeled** only | — | [sender-revoke](protocol/sender-revoke.md), [sender-revoke-window](protocol/sender-revoke-window.md) | Physical deletion; admin/status revokes; outbox has no revoke sender ([why](protocol/unknowns.md#outgoing-revoke-and-edit-2026-10-13)) |
| Keep in chat | live (set); own-device keep sync offline | `keep` | [own-device-keep](protocol/own-device-keep.md), [group-keep](protocol/group-keep.md) | Permission/expiry application |
| Disappearing messages | set live | `disappearing`, `disappearing-default` | [own-device-timer](protocol/own-device-timer.md) | After-read mode, full setting application; our sent messages carry no `contextInfo.expiration` (sender producer untraced) |
| Stickers, video, video notes, documents (receive) | offline | `media-list`, `media-open`, `media-save` | [stickers](protocol/stickers.md), [video-media](protocol/video-media.md), [document-media](protocol/document-media.md) | No live sample; sending unsupported |

## Receipts and delivery

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Delivery confirmations for received messages | live | `listen` | [messaging-transport](protocol/messaging-transport.md) | — |
| Read receipts (manual, automatic, batched) | acked; automatic offline | `mark-read`, `listen` | [read-receipt-batching](protocol/read-receipt-batching.md) | Handset blue ticks unconfirmed |
| Incoming receipt facts (direct, group, aggregate) | offline (live windows saw no qualifying sample) | `receipts`, `group receipts`, `status` | [direct-receipts](protocol/direct-receipts.md), [group-receipts](protocol/group-receipts.md) | Native privacy/read-self policy |
| Retry receipts we send | live (server-acked); PN/LID once-only offline | `listen` | [messaging-transport](protocol/messaging-transport.md#retry-once-only-across-pn-and-lid-2026-10-13) | PN redelivery of a LID-first message without alias attributes stays separate |
| Answering a peer's retry receipt | lab group-send window only, offline | `lab-transport` group send | [groups](protocol/groups.md) | Normal runtime does not answer; native resend policy not traced in-repo |

## Media and voice

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Image send | live (handset receipt) | `send`, `outbox-send` | [messaging-transport](protocol/messaging-transport.md) | — |
| Voice note send | live (receipt; playback unconfirmed) | same | same | — |
| View-once image send and single local opening | live send; local viewer offline | `lab-open-view-once`, `lab-download-view-once` | same | Server opened receipts |
| Video/document/sticker send, HD variants | absent | — | — | — |
| Calls (voice/video) | absent | — | — | Not researched |

## Groups, communities, channels, status, broadcast

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Group receive/decrypt | live | `listen` | [groups](protocol/groups.md) | — |
| Group send (sender-key fan-out) | live (lab); normal outbox offline | `lab-transport`, `outbox-send` | [groups](protocol/groups.md), [normal-group-outbox](protocol/normal-group-outbox.md) | Own-device inventory and feature selection |
| Group management (create, info, add/remove, promote/demote, subject, description, invite, leave) | create observed live 2026-09-22; others implemented, not live-verified | `group …` | [groups](protocol/groups.md) | Live verification per action |
| Communities | absent | — | [mex](protocol/mex.md) (searches found none) | Not researched |
| Channels (newsletters) | parked | — | — | Owner deferred |
| Status receive, viewers, likes received | live | `status-viewers` | [status](protocol/status.md) | — |
| Status post text | live | lab private JSON (see evidence) | same | — |
| Status post image | acked as text earlier; media form not yet live | lab private JSON (see evidence) | same | Live check |
| Status like send, audio, HD dual upload | absent | — | same | — |
| Broadcast lists | receive dispatch traced; runtime unported | — | [broadcast-dispatch](protocol/broadcast-dispatch.md), [broadcast-receipt](protocol/broadcast-receipt.md) | Effective property application |

## Contacts, presence, privacy, settings, app state

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Contacts (local book, check, status, sync, app-state publish) | check live; others offline | `contacts …` | [chat-counterparts](protocol/chat-counterparts.md) | — |
| Presence subscribe, availability, typing | live (typing handset-confirmed) | lab private JSON (see evidence) | [presence](protocol/presence.md) | Recording/incoming indicators unconfirmed |
| Display name, about text | written live, handset unconfirmed | `push-name`, `about` | [presence](protocol/presence.md) | — |
| Profile picture | absent | — | — | — |
| Block/unblock, block list | live | `block`, `unblock`, `blocklist` | [chat-actions](protocol/chat-actions.md) | — |
| Privacy settings (last seen, photo, groups…) | absent (no mutation command) | — | [read-setting-sync](protocol/read-setting-sync.md) | SMAX `<iq xmlns="privacy"><privacy><category name value/>` is confirmed for `readreceipts`, but property 23639 selects MEX vs SMAX (effective value unknown) and other category names are untraced |
| Per-chat privacy, report, clear chat | implemented; live status not established in this index | `chat-privacy`, `report`, `clear-chat` | [chat-actions](protocol/chat-actions.md) | Check the evidence doc before claiming live |
| App-state sync (contact action writes, 409 catch-up) | live | `contacts fetch-app-state`, `contacts publish-app-state` | [chat-actions](protocol/chat-actions.md) | Star/mute/pin/archive actions not exposed |

## Devices, history, sync

| Feature | State | CLI | Evidence | Blockers / next |
|---|---|---|---|---|
| Peer device lists and companion admission | offline | `devices`, `device-query` | [companion-directory-addition](protocol/companion-directory-addition.md) | First-seen companions, raw-ID changes |
| Own companion pairing (QR) | parked | `lab-pair` | — | Owner deferred |
| Own-device message sync (held sessions) | partial offline | `listen` | [own-device-text](protocol/own-device-text.md), [cross-device-sync](protocol/cross-device-sync.md) | Own-account device sync, history transfer |
| History sync / message history notices | partial: notice inspection and control event | `lab-inbox --inspect` | [cross-device-sync](protocol/cross-device-sync.md), [history-mode](protocol/history-mode.md) | History is not applied |

## HTTP API parity

The library and CLI cover the rows above; HTTP does not. `serve` is
deliberately store-less and unauthenticated (loopback by default), so it only
serves secret-free profile and capability data. Serving store-backed
operations, even redacted read-only ones such as prekey counts, would decrypt
the encrypted store for any local client. That requires an access-control
design (for example a per-start bearer token and explicit store/key
arguments) that the owner has not specified; it is recorded here as a
decision to make, not implemented. No document may describe a CLI-only
operation as served over HTTP.
