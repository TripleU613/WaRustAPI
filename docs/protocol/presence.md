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
on connect) was **not traced**; we send it only when asked. **Inferred from the two separate writer cases** (plain `1Rd.A01` on every
connect vs `0y0` case 3 carrying the name): the name presence is a change
event. Since 2026-11-01 `push-name` queues it and `listen` sends it once right
after its availability announcement, recorded as written under `push-name`;
`lab-signup` does the same from the config's `profile.push_name` on completion.

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

`push-name "WaRustAPI"` then `listen --seconds 30`: the patch went to
`critical_block` with no `version` attribute (we held none), `order="1"`, and
the server answered `iq type=result` with
`<collection name="critical_block" version="1"/>`. First write to a second
app-state collection, accepted first time. Whether a companion device shows the
name was not checked — this account has none.

## About text

**Confirmed** from 2.26.37.71. The About screen
(`com.whatsapp.profile.ui.SetAboutInfo->A5H`) saves through `LX/8lM->A04`,
which posts writer case 29 (`arg1 = 0x1d`); `LX/0y0->A00` case 29, logged
`WriterThread/write/status-update`, builds

```
<iq id=<counter> xmlns="status" type="set" to="s.whatsapp.net">
  <status>TEXT</status>
</iq>
```

and keys the id to the screen's success/failure callbacks. `LX/8lM->A04`
replaces a null or empty text with `" "`, so clearing the About sends a space.
`whatsapp-cli about "<text>"` queues it (`transport::about`); `listen` sends it.
The 256-character bound is ours; the writer enforces none and the screen's own
limit was not traced. "Status" is the old name for About and is unrelated to
24-hour status posts.

## Current reader/default-type audit (2026-11-06)

**Confirmed pinned static and Rust correction:** APK2.26.43.72 inherited
1rA.A03/1rF user-presence branch treats missing type as available, just like the
explicit available string. The metadata parser now does so for supported primary PN/LID users, correcting
the earlier untyped interpretation. Missing-type device/group metadata stays
untyped because those routes are separate. Empty, unknown and nontext explicit types remain
unsupported. [Complete selected methods, offsets, reproduction and limits](apk/2.26.43.72-presence-reader/README.md).

**Confirmed support gaps:** native device admission and group count/offline
routes precede this user route; the Rust metadata parser does not implement
those state transitions. Native6qD.A00 maps missing last to currentTimeMillis,
sentinels to zero and signed Java-long seconds through wrapping multiplication
by1000. Rust LastSeen preserves raw absence/nonnegative seconds/sentinel text
and still uses u64 parsing, so it is not full native timestamp projection. No
ACK, request or account-state authority depends on this metadata parser. Full
provider/consumer and exception policy remain open. No live operation occurred.

Validation:1,449 workspace/all-feature tests passed. Strict workspace/all-target/
all-feature Clippy, formatting, CLI reference regeneration, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.

## Current target types and outbound guard (2026-11-06)

**Confirmed static / implemented offline:** [APK43.72 writers and target hierarchy](apk/2.26.43.72-presence-targets/README.md)
re-anchor composing/audio/paused and the subscription writer. Current1bc is
concretely GroupJid/g.us; DeviceJid and server-only0mL are separate Jid types.
The native schema checker logs type mismatch, not a proven server ban rule.
Rust's supported named primary-chat API previously admitted empty-user server
addresses and AD forms when numeric device was0. chat_target/chatstate/subscribe
now require nonempty user and !is_ad alongside their existing server set;
subscription context gets the same named non-device restriction. Explicit
AD0/0 wire kind remains distinct from a plain primary identity. No stripping or
global JID-parser change. CLI validation occurs before transport::run.

Red/green regression covers empty PN/LID/group, agent-only/device-only/AD0/0,
invalid group contexts and valid primary/token cases. Existing codec round trips
retain supported wire construction. Native group subscription remains outside
the existing people-only Rust subset; no feature was added.

DEd's2500ms message2 pause scheduling and strictly>10000ms event suppression
comparison are retained with their0AI.A06/queued-message/bot-state gates. They
are not a mandatory heartbeat. Effective gate meaning, upstream activity/cancel,
full scheduler/restart behavior and the bot/interop branches remain unported.
No extra network request, ACK, automatic timer traffic or live operation.

Current0Tj.A03/A04 additionally confirm smax:any token wrappers merge into a
plain tctoken byte child. The existing Rust subscription wire form needs no
change; other merge branches and complete token-issuance policy remain separate.

## Incoming chatstate routing (2026-11-06)

**Confirmed static / implemented offline:** [current1rB reader and dispatch](apk/2.26.43.72-chatstate-routing/README.md)
select child0, unwrap device addresses through0Dn.A00, cast participant to
UserJid and enqueue Android messages20/21. Reused current device/type evidence
binds ordinary PN/LID devices17/19; class/factory mapping is not inferred from
arbitrary suffixes. Rust previously retained these device-qualified addresses
and known non-user participants in ephemeral metadata. Supported PN/LID now
unwrap to plain user metadata; group/server-only PN/LID authors become None,
and a supported server-only from is omitted because the metadata needs a chat.
Original Node and stored identities are unchanged. Incoming unwrapping does not
loosen the outbound device-target refusal.

Red/green regression covers PN/LID device/agent/AD0/0, group/server authors,
valid group chat, untouched input and first-child refusal despite later
composing. Other JID domains retain their prior metadata behavior pending full
class mapping; this is not a complete native admission/trust validator. Native
null-chat downstream handling and event consumer policy remain untraced.
No journal/state adoption, ACK, refresh, request, replay or live operation.

## Hosted incoming typing identities (2026-11-06)

**Confirmed static / implemented offline:** [hosted factory/constructor/reader chain](apk/2.26.43.72-hosted-chatstate/README.md)
separates the original hosted device domain from its underlying non-hosted
UserJid.0De.A02 maps hosted to PhoneUserJid and hosted.lid to0Dx; device99
constructs Dv4/Dv6 and incoming0Dn.A00 returns their parent userJid. Rust now
projects these supported incoming aliases to ordinary PN/LID metadata. Explicit
hosted device forms require99; no unsupported explicit device observation or
author is fabricated. Plain model aliases and valid device99 text are covered.

The original Node and text/typed attributes remain unchanged. This projection
must not be applied to device hashes, stored identity/alias authentication,
outbound targets or token lookup. No such paths changed. The earlier hosted
hash/domain distinction remains valid. Red/green tests cover both domains,
plain/99/group-author/text inputs, original attributes and explicit device0/2.

Full text grammar (including pre-entry zero-qualifier normalization), all JID
factories and other special families remain unported. This is ephemeral metadata,
not a trust/admission validator. No ACK, request, replay, state adoption or live
account operation.

## Hosted textual zero qualifiers (2026-11-06)

**Confirmed static / implemented offline:** [qualifier-presence trace and regression](hosted-chatstate-text.md)
show native0Dd.A00 chooses a device path on dot/colon presence before examining
numeric values. The shared Rust parser had flattened zero components to Plain,
allowing invalid hosted device0 metadata to look like an unqualified alias.
The incoming-only raw-text reader now retains AD wire kind for qualified hosted
text; the existing device99 check omits invalid observations/authors. Plain
aliases, valid99 text, typed AD handling and PN device0 projection remain.
Original frames/typed inputs, shared JID parsing, hashes and saved identities
are unchanged. Pre-collapsed typed values and full native text/agent grammar
remain an open boundary. No ACK, request, state adoption or live operation.
