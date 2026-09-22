# Groups

Evidence from the comparison APK build **2.26.37.71** (`GroupXmppMethods`
`LX/0Xm`, `CreateGroupApiHandler` `LX/AWf.A00`, `LX/9Ms.A0Z`,
`GetGroupInfoProtocolHelper.A00`), gathered 2026-09-20. Claims are
**confirmed** when read from the node builder, **inferred** when they follow
the public protocol and the APK only shows the surrounding vocabulary, and
**unknown** otherwise. Nothing here is live-verified until
`messaging-transport.md` says so.

All requests are `iq` stanzas with `xmlns="w:g2"` and a fresh `id`. Requests
about an existing group are addressed `to=<group>@g.us`; group creation and
leaving are addressed to the group server (`LX/9Lp.A00`, the `g.us` JID).
Every element and attribute name below is either a dictionary token or a
plain string, so the existing encoder needs no change. Replies are settled by
the durable action queue (`transport/actions.rs`).

| Action | Type | Body | Evidence |
|---|---|---|---|
| Create | set, to `g.us` | `<create subject="…" key="<client id>"><participant jid="…"/>…</create>` | `LX/AWf.A00` 0x053e..0x0602: `key`, `create`, `subject`, participants list (confirmed). Optional children the client adds from settings (`description`, `ephemeral`, `member_add_mode`, `membership_approval_mode`, `announcement`, `locked`) are not sent here. The same method can instead issue the MEX `CreateGroup` query behind a flag; the classic form is used. |
| Info | get, to group | `<query request="interactive"/>` | `GetGroupInfoProtocolHelper.A00` names `interactive` and reads `group_response`; the `query`/`request` element names are tokens (inferred shape). |
| Subject | set, to group | `<subject>text</subject>` | `LX/0Xm.A0E` (confirmed). |
| Description get | get, to group | `<query request="description"/>` | `LX/0Xm.A0G` (confirmed). |
| Description set | set, to group | `<description id="<client id>" [prev="<old id>"]><body>text</body></description>`; delete: `<description delete="true" prev="…"/>` | inferred from the public protocol; the APK's `description`/`body`/`id` vocabulary appears in `LX/AWf.A00` for creation. |
| Add participants | set, to group | `<add><participant jid="…"/>…</add>` | `LX/0Xm.A03` (confirmed; it may wrap participants in `privacy` with `transparency_acknowledged="true"` for LID-addressed contacts, not sent here). |
| Remove participants | set, to group | `<remove><participant jid="…"/>…</remove>` | `remove` is a dictionary token; shape inferred from the public protocol. |
| Promote / demote | set, to group | `<promote><participant jid="…"/></promote>`, `<demote>…</demote>` | `LX/9Ms.A0Z` (confirmed). |
| Leave | set, to `g.us` | `<leave><group id="<group>@g.us"/></leave>` | `LX/0Xm.A0C` (confirmed). |
| Invite link get | get, to group | `<invite/>` | `LX/EnG.A00` queries `invite` among other children (inferred). |
| Invite link revoke | set, to group | `<invite/>` | inferred from the public protocol. |
| Join by code | set, to `g.us` | `<invite code="…" transparency_acknowledged="true"/>` | `LX/0Xm.A00` (confirmed). |

Reply parsing (`group_response`, inferred from the public protocol): a
`group` element with `id`, `subject`, `s_t` (subject time), `s_o` (subject
owner), `creator`, `creation`, and `participant` children carrying `jid` and
an optional `type` of `admin` or `superadmin`; `invite` replies carry `code`.
All of these names are dictionary tokens.

Group messages (`from`=group JID, `participant`=sender device JID) are now
decrypted using the Signal sender-key protocol.

**Stanza shape — now observed live (2026-09-22).** Two real incoming group
stanzas were captured on the lab account:

```
message type=text from=<group>(Group) participant=<sender>(Lid) participant_pn=… addressing_mode=… expiration=… sts offline notify t
  enc v=2 type=msg        <- sender key distribution (first message from that member only)
  reporting
  enc v=2 type=skmsg      <- the group message itself
```

and a later one carrying **only** `enc v=2 type=skmsg`. So a single stanza does
carry both a member's `msg` and its `skmsg` together, which the APK evidence had
only made **inferred**; it is now **observed**. Supporting APK evidence: the node
model permits several `enc` children (`LX/0hs->A0N(name)` returns a List of all
children with that tag, **confirmed**), one `enc` node is one-of
`EncTypeIndividual` (`msg`/`pkmsg`) or `EncTypeSenderKey` (`skmsg`) — never both
(`LX/ADy->A0T` schema union, **confirmed**) — and `LX/ADy->A0O` selects an enc
child by matching type `"skmsg"` rather than assuming a single child
(**confirmed**). The implementation selects each child by type and applies an
SKDM found in the same stanza before decrypting that stanza's `skmsg`.

Requiring exactly one `enc` child, as a first implementation did, drops the
combined stanza permanently: the sender key is never installed, so every later
`skmsg` from that member is undecryptable too.

**SKDM receive** (`senderKeyDistributionMessage` = **field 2**, observed live
2026-09-22: the `msg` child of a real group stanza decrypted to a payload whose
only field number was 2). An earlier implementation used field 18, which is
`liveLocationMessage`; it matched nothing, so the sender key was never installed
and no group message could decrypt. After a `pkmsg`/`msg`
child decrypts, its plaintext is scanned for field 2; if present and its
`groupId` matches the stanza's group, the raw
`axolotlSenderKeyDistributionMessage` bytes go to
`process_sender_key_distribution_message` (wacore-libsignal 0.7.0), storing the
updated `SenderKeyRecord` at `sk:{group_jid}:{sender_non_ad_jid}` in
`Table::SenderKeys`. The SKDM-bearing child advances that member's ordinary
direct Signal session (`comparison-direct:<phone>`), so the session, pinned
identity, any consumed one-time prekey and the sender key all commit together.
A direct (non-group) message carrying field 2 is handled the same way.

**Participant identity**: `participant_pn` carries the member's phone JID when
`participant` is a LID (`LX/ADw->A02` reads it as a `UserJid`, **confirmed**;
it is the group analogue of `sender_pn`). `participant_lid`, `sender_lid` and
`peer_recipient_pn` also exist in this build. When the attribute is absent, a
LID participant is mapped through the `authenticated-lid:` checkpoint the direct
path writes; an unmapped LID has no addressable session, so its SKDM child is
skipped and only a previously stored sender key can decrypt the `skmsg`.

**skmsg receive**: `signal::group::receive_group` calls `group_decrypt` with the
key in effect (the one an SKDM in the same stanza just installed, else the
stored one), then strips the APK's E2E plaintext padding exactly as the direct
path does (1..=16 bytes, each equal to the pad length). Without padding removal
the trailing bytes corrupt the protobuf field scan. If no key is available the
stanza is preserved as Unsupported without acknowledgement. One OCC check
against the pre-decryption value guards the single final sender-key write.

**Delivery receipt.** `received_confirmation` builds
`<receipt to=<group> id=<id> participant=<sender>/>` for a decrypted group
message. Shape from `LX/Cp2->A00` (**confirmed**; the earlier `LX/1q7->A08` lead
was a dead end — that method is `XmppSendMessage/validateStanzaKeyForAckOrReceipt`,
a validator that only logs): `to` and `id` always, `type` omitted for a plain
delivery receipt, `recipient`/`sts` omitted, and `participant` emitted exactly
when it is non-null and differs from `to`. That last rule is why the direct path
never carries the attribute and the group path does.

The participant keeps the incoming stanza's own **device-addressed** form
(**confirmed**): `LX/Cp2->A06` — the receipt builder for an incoming message —
passes `Ayl.A0V(CjN.A04())`, the message's sender JID through
`DeviceJid.Companion.A00` (`LX/0DK->A00`), as the value `LX/CmC->A06` returns as
its pair's second element. That helper returns an existing `DeviceJid` unchanged
and only coerces a bare `UserJid` to its primary device, which renders
identically. There is no device stripping and no PN/LID conversion on this path,
so the implementation echoes the attribute verbatim — note this differs from the
sender-key storage key, which deliberately *does* strip the device. A test pins
the suffix surviving. Not examined: `LX/Cp2->A04` also reaches `A00` through a
different pair helper (`LX/CmC->A05`); whether that path normalizes differently
is **unknown**, but it is not the incoming-message receipt path.

The receipt is gated on `Decrypted`/`Duplicate`, so an Unsupported stanza is
never confirmed and the server may redeliver it; the inbox dedupes redeliveries
by `received:<group>:<id>`.

**Event projection**: decrypted group messages project as typed `NewMessage`
events with `ChatId::Group(from)` as the chat and the `participant` JID as the
sender (`inbox_events::chat_and_sender`). The same resolver serves the text,
quoted-reply and reaction paths, so all three work in groups. A stanza with a
group `from` but no `participant`, or a `participant` with a non-group `from`,
stays opaque. A quoted reply is attributed to `contextInfo.participant` — the
quoted message's author, who in a group is often a third member rather than the
replier — falling back to the stanza sender when that field is absent. In a
direct chat the two agree, so this did not change direct behavior.

**Implementation**: `crates/whatsapp-api/src/signal/group.rs` (SKDM parsing,
receive, padding strip, commit helpers); `crates/whatsapp-api/src/transport/inbox.rs`
(`group_sender` routing, `participant_session_jid`, `process_group`, SKDM
extraction in the direct path); `crates/whatsapp-api/src/transport/inbox_events.rs`
(`chat_and_sender`). Tests cover the DB key format, SKDM extraction (field 2
present, absent, empty), the no-sender-key Unsupported path, bad padding, group
text projection, malformed group stanzas, and a full offline SKDM→skmsg round
trip (`create_sender_key_distribution_message` + `group_encrypt` on a synthetic
sender side, through `process_skdm` → `receive_group` → unpad → field scan).

**Live-verified (2026-09-22).** The operator created a group on their handset,
added the lab number and sent to it. Two group stanzas were received and, after
the field-2 correction, both decrypted: the sender key was installed from the
combined stanza's `msg` child and both messages projected as `NewMessage` events
keyed by `ChatId::Group`. Store after: `SenderKeys` 1, `Inbox` 3 -> 5.

The two stanzas were first refused by the then-current code and the server does
not redeliver a queued stanza, so they were recovered offline from their
preserved captures with `lab-inbox --replay-captures`, which re-runs preserved
message captures through the normal `inbox::process` path (dedupe, atomic
commit, no network, no receipts). Sending a group message is still not
implemented.
Storage note: sender keys use the `sk:{group}:{sender_non_ad}` schema in
`Table::SenderKeys`, keyed by group and member only — a `SenderKeyRecord` holds
multiple chains internally, so the rotation/distribution id is not part of the
key. The unused `RecordKey::SenderKey` variant in `signal/mod.rs` describes a
different (`k:…:{distribution_id}`) layout and is not what this path writes.

## Live record

2026-09-20 ~02:19 EDT: the first `create` (subject and one participant, as in
the table) was written after activation; the server answered
`stream:error code=503` and closed the stream, so the action stays `sent`
with no reply. The next login was refused with `failure reason=403` and an
appeal token (see `STATUS.md`). Whether the group request itself played a
part is unknown; nothing else here is live-verified.
