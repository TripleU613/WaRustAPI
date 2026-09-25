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

**Not done:** the display name is also an app-state setting
(`pushNameSetting`, collection `critical_block`, see
[chat-actions](chat-actions.md)) that companion devices read. A primary-only
client that sends the presence is not writing that record. When the official
client sends the push-name presence (on change only, or also on connect) was
not traced; we send it only when asked.

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
being `chatstate` (`LX/1Qq->A03`). Our receive path does not project incoming
chat states yet.

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
