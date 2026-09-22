# Status updates (broadcast)

Receiving WhatsApp Status. Evidence from the comparison APK build
**2.26.37.71** unless stated. Claims are **confirmed** (read from the
builder/parser), **inferred**, **observed** (seen on the wire live) or
**unknown**, as in `chat-actions.md`.

## Why this is mostly the group path

A status update is addressed `from=status@broadcast participant=<poster>` and
is encrypted with the Signal **sender-key** protocol, the same mechanism as a
group message (`docs/protocol/groups.md`). The sender-key name is built from the
**chat JID's raw string** (`LX/Cnr` factories `A00`/`A01`/`A02`, serialized by
`Cnr.A03`, **confirmed**): `<id>@g.us` for a group, `status@broadcast` for
status. Because the two are different strings they cannot collide, and the
existing `sk:{chat}:{sender}` storage key in `signal/group.rs` already keys on
the chat JID, so no new key scheme is needed.

`status@broadcast` is a canonical JID built by `LX/0hp->A00` (**confirmed**).
The incoming model `LX/Bp1` extends `LX/CjN` and compares `Bp1.A03` against the
literal `"status"` as its discriminator (**confirmed**); `CjN.A05` is the chat
JID, `CjN.A04` the sender, `CjN.A0A` the id, `CjN.A01` the timestamp.

## What is implemented

`transport/inbox.rs` `sender_key_chat` routes any stanza whose `from` is a
group **or** a broadcast, with a user/LID `participant`, into `process_group` —
the same SKDM-install-then-decrypt path groups use. Projection
(`inbox_events::chat_and_sender`) yields `ChatId::Broadcast(BroadcastId)` with
the poster in `MessageKey::sender`, mirroring how a group message carries its
sender. `BroadcastId::is_status()` distinguishes status from a user-created
broadcast list (`<id>@broadcast`), which is addressed the same way.

`ChatId` gained a `Broadcast` variant. This is additive: previously serialized
`Direct`/`Group` events still deserialize.

## What is deliberately NOT implemented

**No delivery receipt is sent for a status update.** `group_sender` is kept
groups-only precisely so `received_confirmation` cannot emit a group-shaped
receipt for a status stanza. A status receipt is a different shape
(`LX/Cp2->A06`, **confirmed**): it adds `type="sender"` and `class="status"`
where the ordinary branch leaves `type` null, with `participant` from the same
`Ayl.A0V(CjN.A04())` DeviceJid path and `to`=`status@broadcast`. It is not sent
because one input is unresolved — `A06` passes `0xffff` as the long argument
that `Cp2->A00` uses for `sts`, which would mean `sts=65535` on every status
receipt; the parameter mapping was not re-read and the tool's decoded signature
for `A00` does not reconcile with its 11-register `invoke-static/range`. Sending
a wrong receipt on a live account is worse than sending none, so this waits on
a confirmed trace. Consequence: the server may redeliver status updates; the
inbox dedupes them by `received:status@broadcast:<id>`.

## Unknown

- Which `enc` types appear on a status stanza. Assumed `skmsg` plus an
  SKDM-bearing `msg`/`pkmsg` as in a group, since the mechanism is the same —
  **inferred**, not observed. No status update has been received live.
- Whether `Bp1.A03` is read from a stanza attribute or derived from the chat
  JID.
- The decrypted payload's form. No `statusMessage`/`statusAttribution` strings
  exist in the APK, but protobuf field names compile to numeric tags so their
  absence proves nothing. Whether a status is an ordinary `Message`
  (image/video/extendedText) or a wrapper is **unknown**.
- `LX/Cnr` has a **third** name component beyond (chat, sender), defaulting to
  `""`. What ever sets it non-empty is unknown. Our two-part key worked live for
  groups, so it is empty at least there.
- Sending a status is not implemented.

## Live record

None. Status receive has not been exercised against a live account. The
routing, projection and `ChatId::Broadcast` representation are covered by
offline tests only.
