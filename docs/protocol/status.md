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

## The server never queues a status for this account (observed)

The decisive measurement, 2026-09-22. The server opens a session with an
`<ib><offline_preview .../></ib>` stanza carrying per-class counts of what it is
about to deliver. `LX/1R0->A02` reads exactly these keys from it — `message`,
`receipt`, `status`, `notification`, `call` (**confirmed**) — so status is a
first-class offline-queue class and a queued status *would* be announced here.

Every `offline_preview` this account has ever received reports **`status=0`**:

```
count=11  message=1  receipt=0  notification=9   appdata=0  call=0  status=0
count=12  message=2  receipt=0  notification=9   appdata=0  call=0  status=0
count=13  message=2  receipt=0  notification=10  appdata=0  call=0  status=0
count=29  message=3  receipt=0  notification=25  appdata=0  call=0  status=0
count=29  message=5  receipt=0  notification=23  appdata=0  call=0  status=0
count=37  message=5  receipt=0  notification=31  appdata=0  call=0  status=0
count=41  message=5  receipt=0  notification=35  appdata=0  call=0  status=0
count=4   message=0  receipt=0  notification=4   appdata=0  call=0  status=0
```

The `message` and `notification` counts move as real traffic arrives, so the
preview is live and the zero is meaningful rather than a field we never
populate. **The server never had a status to give this account**, including
across the sessions following a post the owner shared with an explicit
"Only share with" list naming only this number.

This closes the receive side conclusively: the absence is upstream of our
socket, so no decryption, routing, drain or receipt work on this client can
affect it. Combined with recipient selection being client-side on the poster's
device (`status_distribution`), the remaining explanation is that the poster's
client does not treat this account as an eligible recipient.

Note also `appdata=0`: app-state data is its own offline-queue class, which is
how a companion would learn of patches this client writes.

## Refuted: privacy tokens do not gate status delivery

Traced 2026-09-22 after the owner shared a status with **only** the lab number
and it still never arrived, which eliminated the privacy-list explanation. The
hypothesis was that a poster's client skips a recipient it holds no
`trusted_contact` privacy token for, and that the lab account was missing an
outbound token because only the receive side was implemented. **Refuted on
three independent grounds, all confirmed:**

- `SendE2EMessageJob->A06`'s `privacyTokenManager` reference is a Kotlin
  `lateinit` field check (`if-nez` then `LX/00h->A0I` /
  `throwUninitializedPropertyAccess`), sitting in a run of byte-identical checks
  for `protobufTracer`, `protobufListener`, `pqMigrationChecker`, `message` and
  `e2eEncryptionRetryCounts`. The manager is then passed as one argument into
  the `LX/DEe` bundle. There is no per-recipient lookup, branch or skip in A06.
  Whether something inside `DEe` consults it is **unknown**.
- Nothing ties tokens to status. All 14 `status_distribution` references are UI
  and preferences (`TextStatusComposerFragment`, `VoiceStatusComposerFragment`,
  `ContactPickerFragment`, `LX/6c8`, `LX/7iF`, the `LX/0YK` accessor) with no
  privacy-token code near any of them. The APK has essentially no privacy-token
  logging: one `PrivacyToken` string, in `LX/IVf->getTag`.
- `GeneratePrivacyTokenJob->A0G` **cannot initiate** a relationship. It calls
  `LX/0q2->A0O(jid)` for a stored token, returns on null, and returns again if
  the stored `t` is older than `LX/0q3->A01()`. It only echoes or refreshes a
  token it already holds, so a recipient could not send one first even if we
  implemented it.

For the record, the token's shape is confirmed —
`<iq xmlns="privacy" type="set"><tokens><token jid=<UserJid> type="trusted_contact" t=<long>/></tokens></iq>`,
`t` from `LX/HYS.A00:J`, no other attributes — and received and sent tokens are
tracked in separate tables (`wa_trusted_contacts` with
`incoming_tc_token`/`incoming_tc_token_timestamp`, `wa_trusted_contacts_send`
with `sent_tc_token_timestamp`/`real_issue_timestamp`). **Implementing an
outbound token would not make status arrive**, so it was not implemented.

## Clearing

`clear_chat` accepts `status@broadcast` (and broadcast lists), because a status
update writes a `received:status@broadcast:<id>` row exactly as a group message
writes one under the group JID. A chat is clearable precisely when its messages
are receivable; the two key schemes are pinned together by a test so they cannot
drift apart again. Still local-only, like every other clear.

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

## A recipient subscribes to nothing (confirmed)

Traced 2026-09-22 after a live attempt delivered no status at all. A recipient
does **not** send anything to start receiving status updates; holding an
authenticated connection is the whole of it.

- `w:status` — **0 references**; there is no status IQ namespace.
- `StatusManager`, `StatusesViewModel`, `status_sync`, `statusSync`,
  `StatusSync`, `status_fetch` — **0 references** each.
- The only protocol `subscribe` is `LX/12s->A03`
  (`LocationSubscriptionSendMethods/subscribe`, building
  `<iq xmlns="location" type="get"><subscribe participants="true"…>`) — live
  location, not status. Other `subscribe`/`skmsg` hits are the token dictionary
  in `LX/1Rl.<clinit>`, i.e. vocabulary, not call sites.
- No status-specific offline drain: `offline_preview`/`offline_batch` are the
  generic offline-queue counters.

Recipient selection happens on the **poster's** device: `status_distribution`
(14 references, all through the local preferences accessor `LX/0YK`) holds the
privacy setting, and `statusRecipients` appears only in a local data class
(`LX/6ya->toString`). So the fan-out list is built client-side by whoever posts
(**confirmed** for where the setting lives; **inferred** that the send path
consumes it — the fan-out send itself was not traced).

The privacy token (`GeneratePrivacyTokenJob->A0G`,
`<iq xmlns="privacy" type="set"><tokens><token jid=… type="trusted_contact" t=…/>`)
is consulted from `SendE2EMessageJob->A06` on the **sending** path, so it is
generated about a contact by the party who has them saved — not something a
recipient sends to begin receiving (**inferred**).

**Consequence:** when a status does not arrive, no *subscribe* step is missing
on this client. The leading explanations are that it was posted before the
recipient was saved on the poster's handset (fan-out happens at post time and is
not replayed), or that the handset's Status privacy list excludes them.

Both of those were subsequently eliminated on 2026-09-22: the owner shared a
status with an explicit "Only share with" list naming only the lab number, and
it still did not arrive. See the refutation section below and the Live record
for where that leaves the investigation.
Whether the server additionally filters by the recipient's own server-side
address book is **unknown**, but recipient selection being client-side makes it
unlikely to matter. Our contact store is local-only regardless, so it cannot
influence delivery.

## Contact registration: the one remaining lever

This account had only ever sent `usync mode="query" context="interactive"` for a
single number — discovery ("is this number on WhatsApp"), not address-book
registration. WhatsApp's documented user-facing behaviour is that you see
someone's status only if you also have them saved, and the server can only know
this account's contacts from a sync it has actually received.

`transport::contacts::queue_full_sync` / `whatsapp-cli contacts sync` now sends
the full form. Envelope confirmed from `LX/0u7->A06`: identical
`iq xmlns="usync" type="get"` / `<usync sid index last mode context>` /
`<query>` / per-`<user><contact>+E.164</contact></user>` structure, differing
from the existence check only in mode and context. Vocabularies confirmed
(`LX/0xb`: `full`, `delta`, `query`, `download`; `LX/0xY`: `metadata`,
`registration`, `interactive`, `background`, `notification`, `message`, `add`,
`voip`, `snapshot`, `wamo`), paired by `LX/0xX`: `INTERACTIVE_FULL` is
`full`/`interactive`, and `INTERACTIVE_QUERY` is the `query`/`interactive` pair
this client already sent — which is what validates the reading.

`allow_mutation="true"` is **not** emitted: the attribute exists in the builder
but what enables it is **unknown** (the six-argument constructor hardcodes it
false).

**Live 2026-09-22:** the sync was transmitted and the server answered; the
durable action settled `Ok`.

**Refuted by measurement.** This was a hypothesis test, not an evidence-backed
fix — no APK evidence links contact sync to status delivery. After the sync was
accepted, the next session's `offline_preview` still reported `status=0`
(`count=51 message=5 receipt=0 notification=45 appdata=0 call=0 status=0`). So
registering the address book did **not** make an already-posted status
deliverable to this account.

The sync itself remains implemented and live-verified as a capability; it is
simply not the status gate.

**Why an already-live status cannot be retrieved.** A status is delivered as a
message fan-out: the poster's client computes its recipient list at post time
(`status_distribution`, client-side) and sends to those recipients. There is no
fetch or catch-up path — `w:status` has zero references and no status-specific
offline drain exists. So a recipient who becomes eligible later does not
retroactively receive a status that is still within its 24-hour lifetime, for
the same reason a group message sent before you joined is not replayed to you.
The status remaining visible on the poster's own device is its local retention,
not a server-side queue that can be re-targeted. Testing any eligibility change
therefore requires a **new** post after the change.

## Live record

2026-09-22: the owner posted a status while the lab account held a 90-second
authenticated connection. Group and direct messages sent in the same window
arrived and decrypted; **no stanza with a broadcast `from` arrived at all**, and
no sender key was attempted for `status@broadcast`. This is a delivery absence,
not a decryption failure. A `usync` contact check for the poster settled `Ok` in
the same session, so the connection and action queue were healthy. Per the trace
above the cause is on the posting handset, so no code change was made.

Status receive therefore remains **unexercised**: the routing, projection and
`ChatId::Broadcast` representation are covered by offline tests only, and the
`enc` composition and payload form of a real status stanza are still unknown.
