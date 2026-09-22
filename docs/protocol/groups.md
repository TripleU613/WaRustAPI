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

**Stanza shape.** The node model permits several `enc` children of one stanza
(`LX/0hs->A0N(name)` returns a List of all children with that tag,
**confirmed**), and one `enc` node is one-of `EncTypeIndividual` (`msg`/`pkmsg`)
or `EncTypeSenderKey` (`skmsg`) — never both (`LX/ADy->A0T` schema union,
**confirmed**). `LX/ADy->A0O` selects an enc child by matching type `"skmsg"`
rather than assuming a single child (**confirmed**). Whether one incoming group
stanza carries a `pkmsg` (SKDM) *and* an `skmsg` together is **inferred**, not
confirmed: the decrypt-side loop over enc children was not located, and
`LX/ClL->A02` (QueuedStanzaFactory) reads only the first enc child. The
implementation therefore handles both shapes — it selects each child by type
and applies an SKDM found in the same stanza before decrypting that stanza's
`skmsg` — which is correct whichever shape the server actually sends.

**SKDM receive** (`senderKeyDistributionMessage` field 18): after a `pkmsg`/`msg`
child decrypts, its plaintext is scanned for field 18; if present and its
`groupId` matches the stanza's group, the raw
`axolotlSenderKeyDistributionMessage` bytes go to
`process_sender_key_distribution_message` (wacore-libsignal 0.7.0), storing the
updated `SenderKeyRecord` at `sk:{group_jid}:{sender_non_ad_jid}` in
`Table::SenderKeys`. The SKDM-bearing child advances that member's ordinary
direct Signal session (`comparison-direct:<phone>`), so the session, pinned
identity, any consumed one-time prekey and the sender key all commit together.
A direct (non-group) message carrying field 18 is handled the same way.

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

**Delivery receipts are not sent for group messages.** `received_confirmation`
refuses any stanza carrying `participant`, so nothing malformed goes out, but
the group receipt shape is **unknown**: the path reaches
`LX/0zH->A0R` (`ReadReceipts/sendDeliveryReceiptIfNotRetry`, **confirmed** by
log string) into `LX/1q7->A08`, where the attribute set is assembled deeper than
was traced. Whether a group receipt carries `participant` must be established
before one is sent. Until then the server may redeliver group messages; the
inbox dedupes them by `received:<group>:<id>`.

**Event projection**: decrypted group messages project as typed `NewMessage`
events with `ChatId::Group(from)` as the chat and the `participant` JID as the
sender (`inbox_events::chat_and_sender`). The same resolver serves the text,
quoted-reply and reaction paths, so all three work in groups. A stanza with a
group `from` but no `participant`, or a `participant` with a non-group `from`,
stays opaque. Known limitation: a group quoted reply attributes the quoted
message to the stanza sender rather than to `contextInfo.participant`, so a
quote of a third member's message carries the wrong `MessageKey.sender`.

**Implementation**: `crates/whatsapp-api/src/signal/group.rs` (SKDM parsing,
receive, padding strip, commit helpers); `crates/whatsapp-api/src/transport/inbox.rs`
(`group_sender` routing, `participant_session_jid`, `process_group`, SKDM
extraction in the direct path); `crates/whatsapp-api/src/transport/inbox_events.rs`
(`chat_and_sender`). Tests cover the DB key format, SKDM extraction (field 18
present, absent, empty), the no-sender-key Unsupported path, bad padding, group
text projection, malformed group stanzas, and a full offline SKDM→skmsg round
trip (`create_sender_key_distribution_message` + `group_encrypt` on a synthetic
sender side, through `process_skdm` → `receive_group` → unpad → field scan).

**Not live-verified**: no group message has been received on a lab account.
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
