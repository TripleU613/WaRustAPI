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

Group messages (`enc type="skmsg"`, `from`=group JID, `participant`=sender device
JID) are now decrypted using the Signal sender-key protocol:

**SKDM receive** (`senderKeyDistributionMessage` field 18 in a direct
`pkmsg`/`msg` payload): the decrypted payload is scanned for field 18 after
successful direct-message decryption; if present, `groupId` and the raw
`axolotlSenderKeyDistributionMessage` bytes are extracted and
`process_sender_key_distribution_message` (wacore-libsignal 0.7.0) is called,
storing the updated `SenderKeyRecord` at `sk:{group_jid}:{sender_non_ad_jid}` in
`Table::SenderKeys`, committed atomically with the direct inbox row.

**skmsg receive**: `signal::group::receive_group` calls `group_decrypt` with the
pre-loaded record; if no record exists (SKDM not yet received) the stanza is
preserved as Unsupported without acknowledgement. The updated ratchet and inbox
row commit together (OCC on the sender key).

**Event projection**: decrypted group messages project as typed `NewMessage`
events with `ChatId::Group(from)` as the chat and the `participant` JID as the
sender (`inbox_events::chat_and_sender`). The same resolver serves the text,
quoted-reply and reaction paths, so all three work in groups. A stanza with a
group `from` but no `participant`, or a `participant` with a non-group `from`,
stays opaque. Known limitation: a group quoted reply attributes the quoted
message to the stanza sender rather than to `contextInfo.participant`, so a
quote of a third member's message carries the wrong `MessageKey.sender`.

**Implementation**: `crates/whatsapp-api/src/signal/group.rs` (SKDM parsing,
receive, commit helpers); `crates/whatsapp-api/src/transport/inbox.rs`
(`group_sender` routing, `process_group`, SKDM extraction in the direct path);
`crates/whatsapp-api/src/transport/inbox_events.rs` (`chat_and_sender`). Tests
cover the DB key format, SKDM extraction (field 18 present, absent, empty), the
no-sender-key Unsupported path, group text projection and malformed group
stanzas. Not live-verified: no group message has been received on a lab account.

## Live record

2026-09-20 ~02:19 EDT: the first `create` (subject and one participant, as in
the table) was written after activation; the server answered
`stream:error code=503` and closed the stream, so the action stays `sent`
with no reply. The next login was refused with `failure reason=403` and an
appeal token (see `STATUS.md`). Whether the group request itself played a
part is unknown; nothing else here is live-verified.
