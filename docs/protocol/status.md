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

## Delivery receipt (implemented)

`received_confirmation` emits, for its supported decrypted status sender branch:

```
<receipt to="status@broadcast" id=<id> type="sender" participant=<poster DeviceJid> class="status"/>
```

**Confirmed static / corrected 2026-10-11:** in pinned WhatsApp 2.26.37.71,
[Cp2.A06](apk/2.26.37.71-status-receipt/status-receipt.txt) loads
`const-wide/16 v10, #0xffff (-1)` at 0048. This is a signed 16-bit literal
extended to a long, not positive 65535. The original 2026-09-22 explanation
misread that literal and incorrectly added `sts="65535"` to Rust output.
[Cp2.A00](apk/2.26.37.71-status-receipt/receipt-builder.txt) compares the supplied
long with zero at 00ca and skips its sts append when <=0 at 00ce. Rust now omits
sts. [Full evidence, reproduction and checks](status-receipt.md).

The exact DEX descriptor has eight object parameters, an int and a long;
A00's 13 registers leave two locals and put arguments in v2..v12. The old
shorty-only explanation claimed nine objects and one local; the full descriptor
corrects that uncertainty. The confirmed append gates are:

| Attribute | Gate |
|---|---|
| to / id | Always |
| type | Non-null |
| participant | Non-null and unequal to destination |
| recipient | Non-null |
| sts | Supplied signed long >0 |
| meta mode child | Supplied int !=0 and type in CBs.A00 |

A06 chooses sender type only when Bp1.A05 is true; its other branch passes null.
Both branches add class=status, mode=0, null recipient and signed long -1.
The effective Bp1.A05 producer/admission remains unported here; the current
supported Rust status sender branch retains sender type. No meta child is
emitted because mode=0. Attribute order is to, id, type, participant, then class
for this branch. Primary device class/writer normalization remains separate.

Confirmation stays gated on Decrypted/Duplicate; unsupported stanzas are not
confirmed. Fresh status delivery/server acceptance remains unverified.

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
- **Earlier interpretation, superseded below:**
  `GeneratePrivacyTokenJob->A0G` **cannot initiate** a relationship. It calls
  `LX/0q2->A0O(jid)` for a stored token, returns on null, and returns again if
  the stored `t` is older than `LX/0q3->A01()`. It only echoes or refreshes a
  token it already holds, so a recipient could not send one first even if we
  implemented it.

**Subsequent correction (2026-10-09, APK 37.71):** the job reads outbound
issue/send timestamps from wa_trusted_contacts_send, rather than incoming token
bytes. Counterpart aggregation can use a prior issue timestamp, so this does
not prove a previously acknowledged token send. The job still requires existing
outbound state and cannot derive it from our incoming token rows. Its complete
request includes to=s.whatsapp.net. See the [job trace](identity-token-refresh.md)
for thresholds, callbacks and remaining recovery/canonicalization limits.

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

## Styled text statuses (implemented, receive side)

A coloured full-screen text status is an ordinary `extendedTextMessage` with
three extra fields. Numbers and wire types read from the `LX/BZF` descriptor
(**confirmed**), not from the public schema:

| field | # | wire type |
|---|---|---|
| `text` | 1 | string |
| `matchedText` | 2 | string |
| `description` | 5 | string |
| `title` | 6 | string |
| **`textArgb`** | **7** | **FIXED32** |
| **`backgroundArgb`** | **8** | **FIXED32** |
| **`font`** | **9** | enum |
| `previewType` | 10 | enum |
| `jpegThumbnail` | 16 | bytes |
| `contextInfo` | 17 | message |

**The colours are `FIXED32`, not `int32`.** The descriptor's type character is
`U+1006`, and `& 0xFF` = 6 = `FIXED32`: unsigned, wire type 5, four bytes
little-endian. Decoding them as a varint `int32` does not merely confuse the
sign of the alpha byte — it misreads the wire entirely. `canonicalUrl` (4) is
absent in this build.

Font enum `LX/C44` (**confirmed**, `forNumber`'s packed switch covers exactly
0..=8): `0` SANS_SERIF, `1` SERIF, `2` FACEBOOKSCRIPT_WA_REGULAR,
`3` BEBASNEUE_REGULAR, `4` SYSTEM_BOLD, `5` MORNINGBREEZE_REGULAR,
`6` CALISTOGA_REGULAR, `7` EXO2_EXTRABOLD, `8` COURIERPRIME_BOLD. A value
outside that range is dropped rather than mapped to an invented font.

**Implementation**: `transport::quotes` decodes the three fields into
`TextStyle { background_argb, text_argb, font }`, carried on
`MessageContent::Text`. A plain `conversation` and an ordinary chat message both
yield the default (all `None`), so nothing changes for non-status text. Tested
against a payload assembled by hand from the descriptor — FIXED32 colours
included — rather than by running the encoder.

## Media statuses (researched, not implemented)

* **Video HD is a quality-tagged variant list**: `videoMessage` (`LX/BZE`)
  carries `processedVideos` = field **27**, repeated `LX/BY7`
  (`directPath`=1, `fileSha256`=2, `height`=3, `width`=4, `fileLength`=5,
  `bitrate`=6, `quality`=7, `capabilities`=8). Quality enum `LX/C0H`:
  `UNDEFINED=0, LD=1, SD=2, HD=3, FHD=4` (**confirmed**).
* **Image HD has no quality enum**: `imageMessage` (`LX/BZD`) instead carries
  `midQualityFileSha256`=23 and `midQualityFileEncSha256`=24 beside the primary
  `url`/`fileSha256`. So HD-versus-standard for an image is *which bytes are
  uploaded as primary*, with midQuality\* as the reduced fallback
  (**inferred** — the sender's choice was not traced).
* **The HD switch is a dual upload**: `LX/9QF` case 14 holds
  `av1_video_dual_upload`, `hd_image_dual_upload`, `hd_video_dual_upload`,
  `hevc_video_dual_upload` (**confirmed** strings), so the client uploads a
  second higher-quality rendition alongside the standard one. The pairing was
  not traced.
* **Audio on a status is an ordinary `audioMessage`** (`LX/BZ0`), with no
  status-specific wrapper. It carries **`backgroundArgb` = field 20, FIXED32** —
  the same colour field and encoding as a text status, which is what makes a
  music/voice status coloured (**confirmed**). Other fields: `url`=1,
  `mimetype`=2, `fileSha256`=3, `fileLength`=4, `seconds`=5, `ptt`=6,
  `mediaKey`=7, `fileEncSha256`=8, `directPath`=9, `mediaKeyTimestamp`=10,
  `contextInfo`=17, `streamingSidecar`=18, `waveform`=19, `viewOnce`=21,
  `accessibilityLabel`=22. A sibling class `LX/BYn` also carries `ptt` and
  `backgroundArgb` and was not disambiguated.

All of the above is send-side and therefore blocked behind the fan-out below.

## Posting, liking and resharing (researched 2026-09-22, none implemented)

All outbound status operations go through one job,
`com/whatsapp/productinfra/status/sendflow/SendE2eStatusJob`, whose kind is an
`LX/C03` enum (**confirmed**, `<clinit>`): `STATUS`=0, `STATUS_ADD_ON`=1,
`STATUS_NOTIFY`=2. `A0G` emits an envelope fragment per kind:

* `STATUS` (a plain post) carries **no** extra fragment.
* `STATUS_ADD_ON` (a like / reaction) and `STATUS_NOTIFY` carry
  `<status><meta content_type="add_on"|"status_notify"/></status>`, built by
  `LX/9QF` case 13, whose permitted `content_type` set is exactly those two
  strings (**confirmed**).

A separate `<meta session_scope="status"/>` fragment also exists (`LX/BpS`
case 3, **confirmed**).

**`status@broadcast` is a group-or-broadcast JID for send purposes**
(**confirmed**): `SendE2EMessageJob->A02` gates `participant`,
`groupParticipantHash` and `includeSenderKeysInMessage` on `instance-of LX/0c3`,
one type covering groups and broadcasts, so all three are permitted on a status
send. That the fan-out therefore takes the group shape — a `<participants>`
wrapper of per-device `enc` plus one top-level `skmsg` — is **inferred**; the
assembler was not located.

**A `phash` exemption exists but its applicability is unresolved.**
`SendE2EMessageJob->A02` throws *"cannot send e2e message to a group without a
participant hash"* unless `LX/1Yx->A04(jid, messageType)` holds, which is
`jid.getType() == 5` **and** the message type being one of
`{0x38,0x43,0x44,0x4f,0x56,0x5d,0x79,0x7d}` (**confirmed**). Whether
`status@broadcast` is type 5 is **unknown**: it is parsed by the generic
`LX/0D1->A00` dispatcher, and the JID classes constructed around its
`broadcast`/`status` branch return type 3 (`LX/9Ln`), 10 (`LX/Blh`), 1
(`LX/1bn`), 2 (`LX/9Lo`), 0x13 (`LX/0e1`) and 0x19 (`LX/Bli`) — none of them 5 —
but the string-to-class mapping in that hash switch is not positional, so this
neither confirms nor refutes the exemption.

### Why none of it is implemented

Posting and liking share the same prerequisite: creating our own sender key for
`status@broadcast`, fanning the SKDM out 1:1 to **every device of every
recipient**, then `group_encrypt`-ing the content into one stanza carrying both.
Three inputs for that are **unknown**:

1. the fan-out assembler that places the per-recipient children and the `skmsg`;
2. how `status_distribution` becomes a device-level recipient list;
3. whether a `phash` is required, and over which JID set.

The liking payload is additionally **unknown**: the `STATUS_ADD_ON` kind is
confirmed, but the protobuf it carries was not reached, so it is *not*
established that a status like is a `reactionMessage` (field 46) like a chat
reaction, nor what its `MessageKey` holds for a status target.

Building a multi-recipient sender-key stanza from inference would mean sending
an unverified fan-out from an account with suspension history. Not done.

### Resharing does not exist in this build (confirmed negative)

`repost` and `Repost` have **zero references** anywhere in the APK, as do
`forwardingScore` and `originalStatusId`. `isForwarded` appears twice, both in
`toString` of unrelated local data classes. Class and UI names are not stripped
in this build, so their total absence is strong evidence the feature is not
present — there is nothing to implement.

The nearest thing is a status *question* interaction:
`LX/BpS.<init>(VJL)` builds
`<meta interaction_type="question_reshare" [parent_server_id=<long>] [response_server_id=<string>]/>`,
with `interaction_type="question"` as the non-reshare variant (consumers
`LX/AZJ->AAK`, `LX/AZL->AAK`). That is the interactive-question feature, not a
general status repost.

### HD media

`LX/9QF` case 14 is the dual-upload flag set: `av1_video_dual_upload`,
`hd_image_dual_upload`, `hd_video_dual_upload`, `hevc_video_dual_upload`
(**confirmed** strings). "HD" is therefore a *dual upload* — the client uploads
a second, higher-quality rendition alongside the standard one — rather than a
single flag on the message. The upload pairing itself was not traced, and it is
blocked behind posting regardless.

## Live record

**2026-09-22: a status update was received, decrypted and projected.** The first
one this client has ever seen.

```
Broadcast  chat=status  sender=<poster LID>  content=Text  posted 13:31:47 UTC
```

`SenderKeys` went 1 → 2 as the poster's SKDM installed a key under
`sk:status@broadcast:<poster>`, the `skmsg` decrypted through `group_decrypt`,
the APK plaintext padding stripped cleanly, and the projection produced
`ChatId::Broadcast` with the poster in `MessageKey::sender` — the representation
built for exactly this. Every layer worked on first contact with a real status.

**What made the difference, and what did not.** The earlier finding stands: a
recipient performs no subscribe step, and there is no fetch path for a status
already in flight. Repeated `offline_preview` counts of `status=0` were accurate
— the server genuinely had nothing queued, because the poster's client computes
its recipient list at post time and this account was not on it. What changed is
that the account stopped looking unlike a normal client:

* `usync mode=full context=interactive` registered its address book
  (server-accepted), where before it had only ever sent one-number
  `query`/`interactive` existence checks;
* `<presence type="available"/>` is now announced after connecting, which this
  client had never sent at all;
* a read-only app-state fetch was performed (the server returned an empty
  `<sync/>`).

A status posted *after* those landed reached the account. Which of them was the
operative one is **not** established — they were introduced together, and
attributing the change to any single one would be a guess. What is established
is that the eligibility decision happens on the poster's device before anything
reaches the network, so it was never fixable on the receive path alone.

Note the arriving status did **not** appear in an `offline_preview` count: it
was delivered live during an open session rather than from the offline queue, so
a `status=0` preview does not mean no status can arrive in that session.



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

## Posting a status (live-verified 2026-09-24)

**A styled text status was posted from this client and appeared on the
operator's handset.** It goes out through exactly the sender-key fan-out a group
message uses, aimed at `status@broadcast`.

**The audience is named by the caller, not inferred.** A status has no
membership list to read, and `E2eStatusJobParams.A02()` does not supply one
either: it returns an explicit target set if there is one, else the devices
holding a receipt row for *this* status, else the parent status's devices for a
repost — all of which are the retry list, the same shape as the group case in
`LX/CQl`. The real audience is the account's status privacy list, which this
client does not sync, so the private JSON names it. That is WhatsApp's "share
with only…", and it keeps the client from inferring who is entitled to see
something.

The stanza carries **no `addressing_mode` and no `phash`** — neither appears on
any incoming status we have captured, and the server has not asked for one. The
own identity is included in the participant-hash membership exactly as for a
group, but never addressed.

The payload is `Message.extendedTextMessage` (**field 6**, not a bare
`conversation` — that is what distinguishes a status from a chat line), with
`text` 1, `textArgb` 7 and `backgroundArgb` 8 as **fixed32**, and `font` 9.
Those numbers come from the `LX/BZF` descriptor and were already being decoded
by the receive path, which is what `a_styled_status_round_trips_through_the_
receive_path` checks the encoder against — the decoder was written and
live-verified against a real status from the operator months before the encoder
existed, so it is an independent check rather than the encoder agreeing with
itself.

`E2eStatusJobParams` also settles a question this project carried as unknown:
its fields are named `participantUsersHash` and `participantDevicesHash`, which
identifies `SendE2EMessageJob`'s `groupParticipantUserHash` as the **users**
hash and `groupParticipantHash` as the **devices** hash.

## Correction: resharing exists, under a different name

An earlier entry in this document recorded "resharing/reposting does not exist in
this build" as a confirmed negative, on the strength of `repost`, `Repost`,
`forwardingScore` and `originalStatusId` all having zero references. **That
conclusion was wrong**: the feature is called **reshare**, and the build is full
of it — `can_be_reshared` and `audience_type` columns on `status_message_info`,
a `status_question_reshare_pointer` table, `StatusReshares(reshares=…)`,
`total_reshares_per_day` in `status_ranking`, and
`StatusPlaybackReshareHandler/reshareFailure`.

The lesson is the one the working agreement already states and this document
broke: a negative result is only as good as the names searched. "Not found" needs
the search terms recorded beside it so the next reader can see what was *not*
looked for.

## Views: status receipts (observed 2026-09-24)

After this client's first status post, the server delivered receipts of a shape
the chat path does not produce:

```
<receipt from=<viewer> id=<receipt id> [type] t=…>
  <participants message_id=<status id>/>
</receipt>
```

The status is named by `participants/@message_id`, **not** by the receipt's own
`id`. No `type` is a delivery, `read` is a view and `played` is a video watched
or a voice status heard — the same convention the chat receipts use.

`transport::status_receipts` parses and records these, keeping the furthest
state a viewer reached so that a delivery replayed from an offline queue cannot
demote a view. `whatsapp-cli status-viewers` lists them offline. Recording is
one-directional: this client still sends no read receipt of its own, so learning
that somebody viewed our status does not tell anybody that we viewed theirs.

## Status privacy: the surface, traced but not wired

`LX/I4X` maps the privacy settings this build knows. Categories:
`status`, `statusview`, `last`, `online`, `profile`, `cover_photo`, `messages`,
`pix`, `channelcreation`, `channelview`. Values: `all`, `contacts`, `none`,
`known`, `contact_blacklist` (everyone except a list), `contact_allowlist` (only
a list), `match_last_seen`, `off`, `on_standard`, `with_pin`.

`LX/FCM->A00(name, dhash, allowlist)` builds a `<category>`'s attributes —
`name`, `value` (`contact_allowlist` or `contact_blacklist`) and an optional
`dhash` over the list. The audience lists themselves live in
`status_privacy_custom_list` (`list_id`, `name`, `emoji`, `is_selected`,
`member_jids`, `source_group_jids`, `allow_list_selected`).

**Unknown, and deliberately not guessed:** the IQ that carries this. Setting now
goes through `MexSetPrivacySettingsHandler` (a MEX/GraphQL path), and whether a
classic `<iq xmlns="privacy">` still reads it was not established — the two
call sites of `FCM->A00` are inside classes whose other strings show they are
Facebook's bundled multi-`run()` shells, so the enclosing stanza was not
readable from them. Until that is traced, this client does not send a privacy
request: the status audience is named per post in the private JSON instead,
which needs no server setting at all.

## Likes: status reactions (implemented on receive)

A status like is an ordinary `Message.reactionMessage` (field 46) whose key
names **`status@broadcast`** as the `remoteJid` — that, not a separate message
type, is what distinguishes it from a reaction in a chat. In the APK it lands in
the `status_add_on` table with **`type = 1`**, which an `AFTER INSERT` trigger
counts into `status_interactions.reaction_count`; `type = 0` is a different
add-on kind that was not identified.

`status_receipts::like_from_payload` recognizes one and records it in the same
transaction as the message that carried it, so a like cannot survive a
rolled-back stanza. A reaction being *removed* arrives with empty text and is
recorded with no emoji rather than dropped, or the like would appear to persist
for ever. `whatsapp-cli status-viewers` lists receipts and likes together.

Note this already flowed through as a `MessageUpdated` event before any of this
existed, because the reaction path is generic over the chat; what was missing
was a place to *ask* who liked a status rather than replaying the journal.

## Image statuses (2026-09-24)

A picture status is the text payload swapped for the `Message.imageMessage` the
upload path already produces, through the same fan-out. The stanza is
`type="media"` with `mediatype="image"` on the `skmsg` `enc`, as the direct
image envelope is (`messaging::direct_media`); until 2026-09-25 it went out as
`type="text"`, which the server acknowledged anyway. Where the marker sits in a
fan-out is inferred — see `groups.md`, "a reaction is not a text stanza". Setting `image` on a status request replaces the text rather than
captioning it, and carrying both is refused — silently dropping one of them is
worse than saying no.

It needs a current media route (`--login --media-route`); routes expire, and the
upload refuses rather than guessing one. Transmitted and acknowledged
2026-09-24; handset rendering not separately confirmed.

**Not implemented:** sending a like (the target a reaction is addressed to for a
status — the poster directly, or `status@broadcast` — was not established), the
HD dual upload, and audio.
