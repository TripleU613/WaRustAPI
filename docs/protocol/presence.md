# Display name and typing indicators

Two fire-and-forget stanzas: neither is an `iq`, so the server sends no reply
and nothing can be correlated. What a send can establish is only that it was
written; whether it worked is seen on someone else's handset. Evidence is from
comparison build **2.26.37.71**.

## Display name (push name)

**Confirmed** — `LX/0y0->A00`, the XMPP writer's dispatch, case 3, logged as
`WriterThread/write/push-name`:

```
<presence name="<display name>" type="available"/>
```

* `type` is validated against the set `{available, unavailable}` (`LX/01d->A0A`
  over a two-element array) and is `available` here.
* `name` is added only when the string is non-null, after
  `LX/0hp->A06(name, 0, 2^53-1, true)`. That is a generated schema's length
  check, and its bounds `0..=9007199254740991` mean **the APK imposes no real
  length limit on the wire**. The 100-character limit in
  `transport::presence::push_name` is ours, and says so. (The bound was read
  once `xtask apk-smali` learned to print full-width `const-wide` literals; it
  previously printed the mnemonic with no value.)
* This is a different writer case from the plain availability announcement
  `LX/1Rd->A01` (`messaging::available_presence`), which carries only `type`.

### The app-state record (`pushNameSetting`)

The same name is also an app-state setting, which is how the account's other
devices learn it. All **confirmed** from 2.26.37.71:

| | Value | Evidence |
| --- | --- | --- |
| Collection | `critical_block` | `LX/B66.<clinit>` routes `LX/Azq.A1A` through `LX/Azs->A00` |
| Index | `["setting_pushName"]` (one element) | `LX/B66.<init>`: `Azq.A1A.value` into a one-element `String[]`; `LX/Azq.<clinit>` names it `PushNameSetting` / `setting_pushName`, ordinal 5 |
| `SyncActionData.version` | 1 | `const/4 v8, #0x1` passed as `LX/Azp.<init>`'s int argument |
| Value | `SyncActionValue.pushNameSetting` = **7**, `PushNameSetting.name` = **1** | `apk-proto-fields` on `LX/BZV` and `LX/BQd`; `LX/B66->A05` sets only `name_` |

`whatsapp-cli push-name "<name>"` queues it; `listen` transmits it. Writing a
second collection exposed a latent bug in our own code: `ingest` promoted every
accepted patch into `critical_unblock_low` and `catch_up` applied every
conflict there, so the first `critical_block` write would have overwritten the
contact collection's state. Both now use the collection the patch or conflict
names, with a regression test.

When the official client sends the push-name presence (on change only, or also
on connect) was **not traced**; we send it only when asked.

## Typing and recording indicators

**Confirmed** — `LX/B2G->A01` (`HandleMeComposing/sendComposing`):

```
<chatstate to="<chat>"><composing/></chatstate>
<chatstate to="<chat>"><composing media="audio"/></chatstate>   recording a voice note
```

**Confirmed** — `LX/B2H->handleMessage` (`HandleMeComposing/write/paused`):

```
<chatstate to="<chat>"><paused/></chatstate>
```

* `to` is validated against `UserJid`, `LX/1bn` and `LX/9Lm`. A person is
  certain; that one of the other two is a group is **inferred** from the class
  set, and the receive side below reading a `participant` supports it.
* Both writers run only while `LX/B2G->A04:LX/09Y->A06` holds. What that flag
  is was **not traced**; we announce `presence type=available` first, which is
  what every normal session has done by that point.
* One branch adds a `<bot jid=…/>` child inside `composing` — a bot-chat case,
  **not implemented**.
* Timing: `LX/B2H` re-arms with a 2,500 ms delayed message (`0x9c4`) and
  compares elapsed time against 10,000 ms (`0x2710`). What those windows mean
  on screen was **not traced**, so we write one `composing`, hold for the
  requested seconds, then write `paused`.

**Confirmed** receive side — `LX/1Qq->A02` reads `from`, `participant`, and a
`composing` child (with optional `media`) or a `paused` child, the element name
being `chatstate` (`LX/1Qq->A03`). `presence::parse_chatstate` reads the same,
and a `--receive` window reports each one (state and direct/group only; no
identities). They are ephemeral and are not written to the event journal.

## Subscribing to someone's presence

The server forwards a person's availability and typing to clients that asked.
The official client asks when a chat is opened. **Confirmed** from
`LX/0y0->A00`, logged `WriterThread/write/subscription-request`:

```
<presence type="subscribe" [context="<group>"] to="<person>">
  [<tctoken>…their trusted-contact token…</tctoken>]
</presence>
```

* `context` is a group JID (parsed by `LX/1bo->A03` into `LX/1bn`), added only
  when the caller supplies one. We never supply it; which UI path does was
  **not traced**.
* `to` is validated against `LX/1bn` and `UserJid`.
* The token comes from the message's `tctoken` bundle value and is length
  checked `1..=120` by `LX/0hp->A04`. It is built as `<tctoken>` inside two
  `smax:any` nodes; `LX/0ho->A04` treats `smax:any` as "merge into the node this
  is added to" (its error strings are `Error merging <…/> with <…/>`), so what
  goes on the wire is a plain `<tctoken>` child carrying the bytes. We send the
  peer's token saved from a `privacy_token` notification
  (`incoming-trusted-contact:<jid>`, see [messaging-transport](messaging-transport.md)),
  or none.

Incoming `presence` — **confirmed** `LX/1Qw->A02`: `from`, `type`
(`unavailable` / `available`), and `last`, which `LX/8nc->A00` reads as seconds
(multiplied by 1,000) or one of `deny`, `error`, `none`. The group branch of the
same handler reads a `count` attribute for a `PermanentGroupJid` `from`; it is
not implemented.

```
whatsapp-cli lab-transport --allow-live-request --login --receive --subscribe <person>
```

## CLI

```
whatsapp-cli lab-transport --allow-live-request --login --push-name "<name>"
whatsapp-cli lab-transport --allow-live-request --login --typing <chat> \
    [--typing-state composing|recording|paused] [--typing-seconds 1..25]
```

`--typing` combined with `--message` runs the indicator before the send.
The name is recorded locally under the `push-name` checkpoint.

## Live attempts (2026-09-25, profile `l`)

1. `--push-name` plus `--typing` into the operator's group: the name and
   `composing` were written, then the connection closed during the hold and
   `paused` was never sent. **Our bug:** the hold read the socket under a
   timeout, and cancelling a read on `EncryptedConnection` closes it by design.
   The hold now sleeps without reading.
2. `--typing` alone, 15 s: `composing` written, held, `paused` written, clean
   exit.

Neither stanza has a reply, so both are **written, not confirmed**. Whether the
name and the indicator appeared on the operator's handset is the check.

## Live: subscription answered (2026-09-25, profile `l`)

`--receive --subscribe <operator>` (no saved token for them): after
activation we wrote `presence type=available` and one subscribe, and the
server answered within the window with the operator's presence —
`type="unavailable"`, `last="deny"`. So the subscription is accepted and the
incoming shape matches `LX/1Qw`/`LX/8nc`. `deny` means their last-seen is
withheld from us (their privacy setting, or this account not being a contact
of theirs); which of the two was **not established**. No chat state arrived,
since nobody typed during the window.

## Live: the name record was accepted (2026-09-25, profile `l`)

`push-name "TeleWhatsApp"` then `listen --seconds 30`: the patch went to
`critical_block` with no `version` attribute (we held none), `order="1"`, and
the server answered `iq type=result` with
`<collection name="critical_block" version="1"/>`. First write to a second
app-state collection, accepted first time. Whether a companion device shows the
name was not checked — this account has none.
