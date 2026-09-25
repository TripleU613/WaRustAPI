# Status — 2026-09-25

What is true now. The dated experiment log that led here, including every
superseded conclusion and dead end, is in
[`docs/status-history.md`](docs/status-history.md); per-area evidence is in
`docs/protocol/`. Labels: **live** = seen working against the real server,
**acked** = the server accepted it but a handset result is not confirmed,
**offline** = tested locally only.

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
| Group send (sender-key fan-out) | live — operator read it | [groups](docs/protocol/groups.md) |
| Group replies | acked | same |
| Group reactions | acked, now as `type=reaction` with add-on markers (2026-09-25); handset display unconfirmed | same |
| Retry receipts: sending ours when we cannot decrypt | sent live 2026-09-25; whether the sender resent is unconfirmed | [messaging-transport](docs/protocol/messaging-transport.md) |
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
| Typing / recording indicators (`chatstate`) | written live, handset unconfirmed; incoming ones reported in `--receive` (offline) | same |
| Presence subscribe, incoming availability / last seen | live — server answered with the peer's presence | same |
| Prekey upload and replenishment | live | [messaging-completion](docs/messaging-completion.md) |

A single undecryptable incoming message no longer ends a session; it degrades
to `Undecryptable` and we send a retry receipt.

## Not done

- **Stickers and video notes.** Field numbers are decoded from the APK
  (`stickerMessage` 26, `ptvMessage` 66, `VideoMessage` inner fields);
  `StickerMessage`'s inner layout is not yet decoded. No incoming sticker/PTV
  stanza has been captured. [apk-analysis](docs/protocol/apk-analysis.md).
- Calls; profile picture and "about".
- Privacy settings IQ (namespace unknown).
- Channels and QR/companion pairing — parked by the owner.
- Companion-device fan-out and own-account device sync.
- An unexplained 49-byte `[102]` payload seen on receive.

## Offline verification

475 tests pass (`cargo test --locked --workspace --all-features`); Clippy with
`-D warnings` is clean. Rust 1.98.1 locally and in CI. Store, codec, crash
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
