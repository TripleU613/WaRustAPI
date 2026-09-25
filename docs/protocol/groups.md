# Groups

Evidence from the comparison APK build **2.26.37.71** (`GroupXmppMethods`
`LX/0Xm`, `CreateGroupApiHandler` `LX/AWf.A00`, `LX/9Ms.A0Z`,
`GetGroupInfoProtocolHelper.A00`), gathered 2026-09-20. Claims are
**confirmed** when read from the node builder, **inferred** when they follow
the public protocol and the APK only shows the surrounding vocabulary,
**observed** when seen on the wire live, and **unknown** otherwise. Live
results for group *messages* are recorded in this file (see "Stanza shape" and
"Live record"); the group *management* requests in the table below remain
un-live-verified except where the Live record says otherwise.

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

## Sending: the crypto half is built and round-trip verified

`signal::group` now has the sender side of the sender-key protocol, which group
sending and every outbound status operation both need:

* **`prepare_own_sender_key(chat, own, previous)`** creates (or advances) this
  device's own sender key for a chat and returns the distribution message to
  encrypt 1:1 to each recipient device. The distribution message is a `Message`
  carrying `senderKeyDistributionMessage` at **field 2** — the number confirmed
  live on the receive side, where an earlier implementation used 18 and matched
  nothing. The chat's raw JID string keys the sender-key name (`LX/Cnr`), so a
  group key and a `status@broadcast` key never collide.
* **`encrypt_group_payload(chat, own, key, plaintext)`** produces the `skmsg`
  ciphertext and the advanced record.
* **`pad(plaintext)`** applies the APK's E2E padding (1..=16 bytes, each equal
  to the pad length) — the exact inverse of the `unpad` the receive path uses.

Both return the updated record separately from the ciphertext, and the doc
comments state the rule the caller must honour: **persist only after the send is
accepted**. A distributed-but-unsent key leaves recipients holding a chain we
never use; a used-but-unpersisted key cannot decrypt our own later messages.

The round-trip test runs our send path into our receive path: mint a key,
publish its distribution message, install it via `process_skdm`, encrypt a
padded payload, then `receive_group` it and assert the plaintext is byte-identical
to the original. Encrypting per-recipient uses the existing
`direct::prepare_payload`, already live-verified for ordinary messages.

## Sending: the stanza structure (traced, built, never sent)

`transport::groups::sender_key_stanza` assembles:

```
<message to=<chat> id=<id> type=<type>>
  <participants>
    <to jid=<device>><enc v="2" type="pkmsg"|"msg">…</enc></to>   x N
  </participants>
  <enc v="2" type="skmsg">…</enc>
</message>
```

**Confirmed** (2.26.37.71):

* `<participants>` wraps an already-built child list (`LX/BpZ.<init>(VL)`,
  bounded to [1, 0x987356]).
* Each child is **`<to>`** carrying a **`jid`** attribute: `LX/Cp3->A08` builds
  and returns that list with element name `"to"` at all three of its
  construction sites, and `LX/Cp3->A0A(jid, encList)` sets `jid` then appends
  each enc as a child.
* Enc type strings come from `LX/Cp3->A06`, the outgoing mapper:
  `0=msg, 1=pkmsg, 2=skmsg, 3=frskmsg, 4=msmsg` — the exact inverse of the
  `LX/C9c->A00` reader the receive path already matches.

**Inferred:** that the top-level `skmsg` is a *sibling* of `<participants>`
rather than nested within it. `LX/Cp3->A08` returns only the `<to>` list and its
caller places the other enc; `LX/Cp3->A02` — invoked with an *empty* recipient
list, consistent with a non-per-recipient enc — is the candidate but was not
read.

**Deliberately not emitted**, because their conditions were not established:
`phash`, `addressing_mode`, `device_fanout`, `category`, `t`, `expiration`,
`recipient`/`recipient_pn`, and the per-`<to>` `peer_recipient_lid` and
`eph_setting` (`LX/Cp3->A07` gates the former on `DeviceJid.isPrimary()` plus an
`LX/0DO` device resolved through an inverse LID map). Those strings all appear
in `SendE2EMessageJob->A06`, but which apply to a plain group or status send is
unknown, and a test asserts none of them is emitted so a later guess cannot slip
in silently.

The builder is covered by a structural test and a wire-codec round trip.
**It has never been transmitted.**

## Sending: what still blocks an actual send

The crypto is done; the **envelope** is not. Still **unknown**:

1. the per-recipient child element inside `<participants>` — `LX/BpZ.<init>(VL)`
   appends an already-built list, so the element name comes from whoever builds
   that list, which was not located;
2. where the top-level `<enc type="skmsg">` sits relative to `<participants>`;
3. the `message` node's own attribute set for a group or status send;
4. how the recipient set becomes a device-level list, and whether our own other
   devices are included;
5. what sets `includeSenderKeysInMessage` — every send, or only on rotation;
6. whether `phash` is required for a status (a JID-type-5 exemption exists; see
   `status.md`).

`participant_hash` is implemented and independently vectored, so (6) is a
routing question rather than a cryptographic one.

## Sending a group message — the earlier envelope trace

### Earlier envelope findings

Researched 2026-09-22 against 2.26.37.71. Recorded here so the dead ends are
not re-walked; **nothing below is implemented** and one required input has no
known algorithm.

**`<participants>` wrapper — confirmed.** `LX/BpZ;-><init>(VL)`:
`A0m("participants")` builds the child, `LX/0hp->A07` bounds the list to
[1, 0x987356], the list is iterated and each element appended verbatim via
`LX/8kZ->A1A`, then merged as a `smax:any` fragment by `LX/8kY->A0Z`. Because it
appends an already-built `List` of nodes, **the per-recipient child element name
is not in this class and is unknown** — the caller supplies it, and the caller
was not located.

**`enc` builders — confirmed but separate.** `LX/BpT;-><init>(VI)` builds
`<enc v="2">`; `LX/BpZ;-><init>(VLLLL)` builds a second `enc` whose three
attributes are set inside `LX/9QF->A00`, `LX/Ayo->A1B` and `LX/Ayo->A1C`, none
of which were read. Each `Bp*` class emits an isolated `smax:any` fragment
(root `LX/Bph->A01()`) that something else merges, and **the assembler that
places the per-recipient children and the top-level `skmsg` together was not
found**. The obvious shape — one top-level `skmsg` beside a `<participants>`
wrapper of per-device `pkmsg`/`msg` — is therefore **inferred**, not confirmed.

**Participant hash is required — confirmed.** `SendE2EMessageJob` validation
strings: `"cannot send e2e message to a group without a participant hash"`,
`"groupParticipantHash cannot be set to an empty string"`,
`"groupParticipantHash cannot be set if participant is set"` (so `phash` and a
targeted single-`participant` send are mutually exclusive), and
`"groupParticipantHash cannot be set if the primary jid is not a group or
broadcast list"`. Emission is `LX/BpS;-><init>(VLI)`:
`A1Q(frag, "phash", value)` gated on `LX/0hp->A06(value, 0, 10, …)`, a bounds
check capping it at **10 characters** (confirmed).

**The `phash` algorithm is confirmed** (`LX/1jL->A02(Collection<Jid>)`, traced
2026-09-22) and implemented as `transport::groups::participant_hash`:

```
sorted   = sort([jid.getRawString() for jid in collection])   // natural String order
digest   = SHA1(concat(sorted))                               // no separator between entries
phash    = "1:" + Base64(digest[0..6], NO_WRAP)               // standard alphabet
```

SHA-1, not SHA-256. The six-byte truncation encodes to exactly eight Base64
characters with no padding, which with the two-character `"1:"` prefix is
exactly the ten characters `LX/0hp->A06(value, 0, 10, …)` bounds `phash` to in
the emitter — the width matching the emitter's own bound is independent
corroboration of the derivation. The implementation is tested against vectors
computed separately with Python's `hashlib`/`base64`, not by running the
implementation.

The hash is lazily cached per participant set in the APK (`A0J` caching field
`A01`, `A0L` caching `A02`), recomputed only when cleared — so it is per
member-list, not per send.

**What still blocks a group send: the membership set.** `A02` does not
normalize; it hashes whatever `getRawString()` yields, so the caller chooses the
JID form. The APK keeps two hashes over two different maps (`A0A.keySet()` and
`A0B.keySet()`), matching `SendE2EMessageJob`'s `groupParticipantHash` /
`groupParticipantUserHash` pair and its `useParticipantUserHash` flag — but
**which map is device-keyed versus user-keyed, and whether the sender itself is
included, are unknown**. A wrong membership set yields a wrong six-byte prefix
and a refused send, so `participant_hash` takes the set from its caller and
nothing here picks one.

**Sender-key re-send policy — flags confirmed, trigger unknown.**
`SendE2EMessageJob` strings: `"cannot use group encryption without including
sender key in message"` (an `skmsg` send requires sender keys in that same
send), `"includeSenderKeysInMessage cannot be set if the primary jid is not a
group or broadcast list"` (an explicit per-send flag), `"forced sender key
distribution can only be used with target devices "`, and `"missing sender key
distribution message "`. **What triggers a rotation, and whether the SKDM goes
to every device on every send or only on change, is unknown.**

**Node attributes — names confirmed, mapping inferred.** `LX/1Dz->A0H` lists,
in order: `lid`, `msgid`, `remote_chat_jid`, `participant`, `remoteJid`,
`phash`, `count`, `timestamp`, `smid`, `edit`, `counter_abuse_token`,
`addressing_mode`, `refresh_lid`. That method looks like a key/value record
rather than the XMPP node builder, so treating these as node attribute names is
**inferred**. `addressing_mode` is confirmed present in `SendE2EMessageJob->A06`
adjacent to `phash`; a `device_fanout="false"` fragment (`LX/BpS`) and a
`multicast` fragment (`LX/BpT`) also exist.

Sending would additionally need per-device fan-out, which is the open piece of
checklist item 8 — contact device lists are already fetched and verified
(`transport/devices.rs`) but are not yet used to address a send.

## Live record

2026-09-20 ~02:19 EDT: the first `create` (subject and one participant, as in
the table) was written after activation; the server answered
`stream:error code=503` and closed the stream, so the action stays `sent`
with no reply. Nothing else in the management table above is live-verified;
group *message* receive since has been — see the 2026-09-22 entry below.

## Outbound fan-out — where the recipient set comes from (2026-09-22, 2.26.37.71)

`SendE2EMessageJob` **does not compute the recipient set**. Its constructor
signature `(VLLLLLLLLLLLLLLLLLLIIIJJJJJZZZZZZZZZZ)` takes it, along with every
policy flag, as a parameter: reading the constructor's `iput` sequence gives the
field for each argument, and the caller `LX/DNu->run` supplies them all. The
useful names survived obfuscation:

| Field | Source at the `LX/DNu` call site |
| --- | --- |
| `targetDeviceRawJids: HashSet` | caller-supplied device set (empty on a normal first send) |
| `groupParticipantHash` / `groupParticipantUserHash` | caller-computed, both passed |
| `useParticipantUserHash` | `(chat instanceof LX/0c3) && targetDeviceRawJids.isEmpty()` — true for a normal group send |
| `includeSenderKeysInMessage` | `Ceo.A0d \|\| <caller boolean>`; **not** computed in the job |
| `useLidForEncryption` | `Ceo.A0h` |
| `useOneOneEncryptionOnPHashMismatch` | `LX/0DE->A0m(chat) \|\| LX/0DE->A0Q(chat)` |
| `forceSenderKeyDistribution` | caller boolean |
| `multiDeviceFanOut` | caller boolean |

Mapping method: the invoke passes `{v23..v64}`, so call-site `v55..v64` are the
ten booleans and ctor `v61..v70` receive them in the same order; the ctor
prologue moves `v63→v9`, `v65→v10`, `v66→v24`, `v68→v8`, `v69→v1` before the
`iput`s that name the fields. **Confirmed** by reading both methods.

Inside `A06` the flags are used as
`A01 = includeSenderKeysInMessage ? (v17 ? 1 : v16 ? 2 : 0) : 0` (`0b70..0b88`),
a three-way sender-key mode whose two upper inputs are still unnamed, and a
separate large-recipient branch logging `SendE2EMessageJob/one-time-sender-key`
when the recipient count reaches the `LX/AzZ->A0a` threshold and an AB gate is
on.

### Negative result: `messageRecipientDevicesProvider` is not the resolver

`LX/CQl->A00` reads `LX/0Z4->A04(key)` → `LX/0Z5`/`LX/0Z6`/`LX/0ZE`, whose own
string constants name them: tables `receipt_device` and
`message_add_on_receipt_device`, cache `MESSAGE_DEVICE_RECEIPTS_CACHE`, columns
`receipt_device_jid_row_id` / `primary_device_version`. It returns the devices
that already hold a **receipt row for this message** — prior send targets, for
retries and add-ons — and `A06` reaches it only inside the
`counterAbuseTokenGate.A01(message)` branch. **Do not re-trace it.** Its one
relevant branch is `instanceof LX/BuQ` → `E2eStatusJobParams.A02()`, which is
where a *status* post's recipient list comes from; that is the thread to pull
for posting, not for groups.

So the fresh recipient set is the ordinary one — the chat's participants
expanded to their devices — which is what
`transport::group_send::plan` builds from the saved `w:g2` snapshot and the
saved `usync devices` lists.

### Addressing mode — confirmed and observed

`A06` `12d0..1322`: `addressing_mode` is `"lid"` when
`groupParticipantsManager.A0k(chat)` or `LX/0DE->A0W(chat)` holds, `"pn"`
otherwise, and `13d8..13fc` emits it when it is non-null and the chat JID is not
itself the LID form — so a `g.us` chat always carries it.

**Observed** 2026-09-22: every stanza from the operator's group carried
`addressing_mode="lid"` with a LID `participant` and a `participant_pn`
alongside. `lab-inbox --inspect` now prints the value, so this is re-checkable
without decrypting anything.

`phash` is `groupParticipantHash` verbatim (`A06` `1300`) and is emitted only
when that caller-computed field is non-null. We omit it: the membership-set
question above is still open, and a wrong six-byte prefix is worse than an
absent one. Omitting it forfeits the server's stale-list correction; it does not
fabricate one.

### Own LID

**Confirmed** `LX/0iS->A12`: the `<success>` login stanza carries a `lid`
attribute, read through `DeviceJid`/`getUserJid`, whose absence the client
reports as `reg-lid-chatd-lid-expected-but-null`. A LID-addressed group names us
by it, so `lab-transport --login` now records it
(`transport::group_send::record_own_lid`); without it no fan-out can name its
own sender key correctly.

### What is implemented

`lab-transport --login --message <json>` with `group` instead of `to` in the
private JSON performs, on one connection: `w:g2` info query → snapshot;
`usync devices` for every other member; `encrypt` key query for the devices
without a session; per-device SKDM encryption; `skmsg` encryption of the padded
text; a single atomic commit of every session, identity and our own sender key
together with the send intent; then one write and one `ack`. It refuses when a
member has no device list rather than sending a message that would reach only
part of the group.

### Live record

2026-09-22: one fan-out was transmitted and **server-acknowledged**. The
connection reported `own LID recorded by the login result`, then
`device lists ingested=2, partial_errors=0, accepted=true`,
`addressing_mode=lid, recipients=2, devices=2, without_device_list=0`,
`key query for 2 device(s) type=result`, and finally
`fan-out server_acknowledged`. That establishes server acceptance of the stanza
shape — `<participants>` of `<to jid=…>` each carrying its own `enc`, beside a
sibling `skmsg`, with `addressing_mode="lid"` and **no** `phash` — and of the
per-device SKDM. It does **not** establish that a recipient handset displayed
the message; the operator has not confirmed that.

The live run settled the `<participants>`-sibling placement of the `skmsg`,
previously *inferred* from `LX/Cp3->A08` returning only the `<to>` list.

It also shows that a send **without** `phash` was accepted **once**, for a
three-member group whose membership we had fetched seconds earlier in the same
connection. That is not evidence that `phash` is generally optional: the server
may simply have found the `<to>` set consistent with what it knew. Treat it as
"accepted once, membership freshly fetched", not as a rule.

### The acknowledged send did not decrypt on the recipient (2026-09-22)

The operator reported **"waiting for this message"** — WhatsApp's placeholder for
a stanza it stored but could not decrypt, i.e. a device that holds no sender key
for our chain. What the saved evidence establishes:

* **The stanza was accepted, not tolerated.** The `ack` carries
  `class="message"`, no `error`, and two attributes we had never seen:
  a version 2 `phash` and `count="2"`. `lab-inbox --inspect` now prints an
  `ack`'s `class`, `phash`, `count` and `error`, since an acknowledged send that
  is undecryptable on the handset otherwise gives no clue which list was wrong.
* **`count` matched.** We addressed two devices; the server counted two.
* **The server's participant hash is version `2:`, not the `1:` form
  [`participant_hash`] implements.** Solving for it offline against the three
  known member LIDs — every subset, sorted/unsorted/reverse, bare and
  `@lid`-suffixed, joined with no separator/comma/newline, SHA-1, SHA-256 and
  MD5, with and without the group JID prefixed — reproduced **none** of it at
  first. (A wider sweep did, below.) **Not** brute force against the server: one
  known-correct target, searched locally.
* **Our distribution message is well formed.** Generated bytes are 77 bytes,
  version byte `0x33` (legacy v3), protobuf fields 1 `id`, 2 `iteration` (0),
  3 `chainKey` (32), 4 `signingKey` (33) — the same shape we successfully parse
  from real incoming stanzas.
* **No retry receipt** arrived in a 40-second listen window after the send.

**The leading explanation, and it is not yet confirmed:** every member's saved
device list holds **only the primary**, with `companions_verified: false` and
`unverified_reason: "no signed index list"` — we hold no pinned primary identity
for these LID contacts, so `LX/11T.A07` verification kept the primary and
dropped any companion. A member reading WhatsApp on a linked device (Web,
Desktop, a second phone) would therefore have received no SKDM at all and would
show exactly "waiting for this message". That would also explain a version-2
participant hash computed over a device set we cannot reproduce.

**The operator answered: the placeholder is on the phone itself.** That kills the
companion-device explanation — the primary is precisely the device we addressed.

### What the structure check then eliminated

With the recipient known to be a device we addressed, the stanza itself was
re-checked against the APK, and every remaining structural doubt was closed:

* **`<participants>` is confirmed, not inferred.** `LX/BpZ;-><init>(VL)` holds
  the literal. An earlier note claimed the name could not be found by string
  search because it is a binary token; that claim was wrong — the string has 31
  references.
* **`<to jid=…>` carrying per-participant encs is confirmed from an
  unobfuscated job.** `SendLiveLocationKeyJob->A0G` builds exactly this shape to
  distribute live-location sender keys: one attribute `jid` per child, element
  name `"to"`, children from `LX/Cp3->A03`, all wrapped in a node named
  `"participants"`. It is a `<notification type="location">` rather than a
  `<message>`, but the fan-out container is the same and it is readable without
  deobfuscation.
* **The outgoing `<enc>` attribute set is confirmed.** `LX/Cp3->A09` emits `v`
  and `type` always, then `count`, `mediatype`, `native_flow_name`, `duration`,
  `decrypt-fail="hide"`, `state="false"` and `session_type="pq"` only under
  their own conditions — none of which apply to a plain first send. Ours emits
  `v="2"` and `type` and nothing else, which is the correct minimal case.
* **The SKDM `groupId` string matches.** The receive-side guard's own comment
  records that a real incoming SKDM's group id parses to the same user and
  server as the stanza's group, differing only in the wire-encoding `kind` —
  i.e. their string is the plain `…@g.us` we also send.
* **Our sender-key chain is self-consistent across a republish**, covered by
  `a_republished_sender_key_still_decrypts_later_messages`.

So the stanza shape, the enc attributes, the distribution-message bytes and the
chain arithmetic are all accounted for. **What was left is `phash`** — and the
ack had handed us the server's own value for this group.

### The version 2 participant hash, solved

Searching locally for a construction that reproduces the server's six bytes —
every subset of the known membership, four JID renderings, four orderings, five
separators, four digests, with and without a group or count prefix — produced
exactly **one** match, and it answers all three questions this document had left
open since `participant_hash` was written:

| Question | Answer |
| --- | --- |
| Which membership set? | **Every member, the sender included.** |
| Which JID form? | The **AD form** `user.agent:device@server`, with `.0` and `:0` written out — not the display form, which omits them. |
| Which digest? | **SHA-256** for version `2:`, where version `1:` is SHA-1. |

The rest matches version 1: sort the rendered strings, concatenate with no
separator, take the first six digest bytes, Base64, prefix `"2:"`.

This was a local search against one known-correct target, not probing the
server. `transport::groups::participant_hash_v2` implements it, with vectors
computed independently over synthetic identities — the real value and the
identities behind it stay in private lab state.

`transport::group_send::plan` now computes it over the whole membership and the
fan-out sends it.

**The server confirmed the derivation independently.** A second send carried our
computed hash, and its `ack` came back with `count="2"` and **no `phash`** — the
correction the first ack had contained is gone, which is the server saying our
claimed membership matches its own. The derivation is therefore not just a
reproduction of one observed value; the server has now validated a hash we
computed ourselves.

That second send also differed in one unavoidable way: the sessions established
by the first send meant its distribution encs went out as `msg` rather than
`pkmsg`, and no key query was needed.

**It was also still "waiting for this message" on the handset.** So the phash was
not it either, and with the structure, the enc attributes, the SKDM bytes and the
hash all accounted for, the remaining difference had to be in how the `<to>` JID
itself goes on the wire.

### A `<to>` names a device, and a device is an AD-JID

**Observed** 2026-09-22, decisive: on every incoming group stanza the server
stamps, `participant` decodes as an **AD-JID** — the form carrying explicit agent
and device bytes — with `device=0`, while the group `from` beside it decodes as a
plain JID pair. `lab-inbox --inspect` now prints that distinction, so it is
re-checkable.

Our `Jid` only reaches for the AD encoding when the agent or device is non-zero
(`is_ad`), so a member's **primary** device — agent 0, device 0 — was being
encoded as a plain *user* JID. `<to jid=…>` in `A08` is built from a
`DeviceJid`, and the version 2 participant hash is likewise computed over the AD
rendering with the zeros written out, both pointing the same way: a device
address is AD-shaped everywhere. A server that cannot resolve a `<to>` to a
device has nowhere to deliver that distribution enc, which is exactly the
observed symptom — the `skmsg` arrives, the sender key never does, and the ack
is unaffected because its `count` is the server's own participant count rather
than an echo of our list.

`sender_key_stanza` now renders every `<to jid=…>` in AD form regardless of
device, asserted on the decoded stanza after a wire round trip.

**It did not fix it either.** A third fan-out, with AD-form `<to>` JIDs and
fresh `pkmsg` sessions, was acknowledged with the same clean `count="2"` and the
handset still shows the placeholder.

### Where this stands: three corrections, three eliminations

Each of the three changes below was evidence-backed and is worth keeping — the
stanza is closer to what the APK builds than it was — but **none of them was the
cause**, and the doc should not read as though any of them were:

| Change | Evidence | Result |
| --- | --- | --- |
| `addressing_mode="lid"` | confirmed in `A06`, observed on every inbound stanza | still undecryptable |
| `phash` (version 2) | solved against the server's own value, and the server stopped correcting us | still undecryptable |
| `<to jid=…>` in AD form | observed: the server stamps `participant` as an AD-JID | still undecryptable |

Constant across all three: the `ack` is clean (`class="message"`, no `error`,
`count="2"`), and **no retry receipt has ever arrived** across five receive
windows. That silence is unexplained and is itself evidence — a phone that
cannot decrypt should be asking us to re-send.

Two hypotheses remain, and they are the only unexplored ones:

1. **LID sessions have never been proven with this recipient at all.** The one
   live-verified direct message to this person was PN-addressed. `LX/CPQ->A00`
   (**confirmed**) sets `useLidForEncryption` true exactly when
   `groupParticipantsManager.A0k(chat)` holds — the same predicate that makes
   `addressing_mode` `"lid"` — so encrypting under a LID address is right for
   this group, but right and *working* are different claims. The phone already
   held a PN-keyed session with us before any of this; what it does when a known
   identity opens a second session under a different addressing form is exactly
   what `AxolotlDifferentAliceBaseKeyRequirement` exists for, and we have not
   checked it.
2. **The retry-receipt silence has a mechanical cause**, possibly the same one:
   a retry receipt is addressed to the sender, and if the phone identifies us by
   PN from that earlier session while the group stanza says LID, the receipt may
   be going somewhere we are not looking.

The discriminating experiment avoids the fan-out entirely: send a **direct**
message to the recipient's **LID** through the path that is already
live-verified for their PN. Same `prepare_payload`, same `pkmsg`, same person;
only the addressing form changes. If it decrypts, LID sessions work and the bug
is elsewhere in the fan-out. If it does not, the bug is found without touching
groups. `MessageRequest::recipient` now accepts a LID for exactly this.

### LID sessions work — the fan-out's own delivery is the remaining suspect

**Live, 2026-09-22:** a direct message addressed to the recipient's **LID**,
through the 1:1 path already verified for their phone number, with a fresh
`pkmsg` session — the operator confirmed it arrived readable.

So LID-addressed Signal sessions decrypt on that handset. Our prekey fetch by
LID, our session establishment against a LID address, our `pkmsg` and our
padding are all correct for this recipient. The first of the two remaining
hypotheses is therefore **eliminated**, and it eliminates a great deal with it:
the only thing the fan-out does that a working direct message does not is put
the distribution `<enc>` inside `<participants>` rather than sending it as its
own message.

### Out-of-band distribution

The APK supports not carrying sender keys in the message at all:
`includeSenderKeysInMessage` is a per-send constructor flag, and
`SendLiveLocationKeyJob` distributes its keys entirely out of band. So the
private message JSON now takes `skdm_delivery`:

* `participants` (default) — one `<enc>` per device inside the group stanza's
  `<participants>`, the form built so far;
* `direct` — one ordinary direct message per device carrying the distribution,
  each acknowledged before the group stanza is sent, with `<participants>`
  omitted entirely.

`sender_key_stanza` therefore accepts an empty fan-out and omits
`<participants>` rather than refusing, which is the shape a send with
`includeSenderKeysInMessage` false produces.

The point of `direct` is that it replaces the one unverified step with a step
that is now live-verified end to end. If the group message decrypts under it,
the bug is in how the server treats our `<participants>` block; if it still does
not, the distribution is arriving and the recipient is rejecting its *contents*,
which is a different search entirely.

### The receive side, read (2026-09-22, 2.26.37.71)

Out-of-band distribution did **not** work either: both distribution messages were
acknowledged individually on the verified 1:1 path, the group stanza followed,
and the handset still shows the placeholder. So the reading of `LX/Cm3->A05`
(`StanzaHandlerUtil/processSenderKeyDistributionMessage`) matters — it is the
code that has to accept what we send:

* The sender address comes from `LX/COU->A00`. If it yields null the handler
  logs `could not extract sender address` and **drops the distribution
  silently**. This is a live candidate for why the out-of-band attempt did
  nothing: a direct stanza has no `participant`, and whether `COU.A00` can
  resolve a sender from one was not established.
* Both SKDM fields are required (`bitField0_ & 1` and `& 2`, i.e. `groupId` and
  `axolotlSenderKeyDistributionMessage`).
* The `groupId` is compared against the stanza's chat **only when that chat is a
  group** (`LX/0DE->A0m`); for a non-group stanza the check is skipped and
  processing continues. A mismatch logs
  `group id on message and group id in ciphertext do not match`.
* The sender key is filed under `LX/Cnr(senderAddress, groupId, "")` — the
  group id taken from **the SKDM protobuf**, not the stanza, and an empty
  **bucket**. (Buckets are real: `LX/Cp3->A03` adds a `bucket` attribute to the
  enc when `senderKeyBucketingConfig` supplies one. We send none, so `""` on
  both sides.)
* Outcomes are logged as `StanzaHandlerUtil/processSenderKeyDistributionMessage/
  status/<n>`: 0 success, `0xfc11` incomplete, `0xfc13` legacy, `0xfc18` other.

**Our distribution message is not what is being rejected.** `LX/DRv`'s parsing
constructor accepts version `3` exactly — below it throws `Legacy message:`,
above it `Unknown version:` — and requires all four of `id`, `iteration`,
`chainKey` and `signingKey`. WhatsApp's own serializing constructor writes the
same `0x33` version byte. Ours is `0x33`, 77 bytes, with all four fields. So the
format is **eliminated**: a delivered SKDM of ours would parse.

That leaves two live candidates, and neither is guesswork any more:

1. **The `<participants>` enc is not delivered to the device.** Everything about
   our stanza that can be checked against the APK now matches, and the ack's
   `count` is the server's own participant tally rather than an echo of our
   list, so it cannot confirm delivery either way.
2. **`LX/COU->A00` cannot resolve a sender address from a non-group stanza**,
   which would make the out-of-band attempt a no-op rather than a refutation.

Both are answerable with full visibility on **both** ends — a second instrumented
account in a group with the first — which is the next thing to set up. None of
this account's spare lab numbers is registered, so that needs a decision rather
than another send.

Noted and deliberately not chased yet: the group has disappearing messages on
(inbound stanzas carry `expiration`, and `notification type=disappearing_mode`
was captured). That will not stop decryption, but a non-ephemeral payload in an
ephemeral chat is the next thing to get right once something does decrypt.

### Phantom sessions

There is a consequence to fixing this late. Each earlier send created our half of
a Signal session with those devices, but if the distribution encs were never
delivered the peer created nothing. We would keep emitting `msg` against a
ratchet they never established, which can never decrypt however correct the rest
becomes.

The private message JSON therefore takes `fresh_sessions`, which drops the
saved *session* for each fan-out target and establishes a new one from a fresh
prekey bundle. It keeps the pinned peer identity and still refuses a changed one,
so it resets a ratchet and nothing else. It is deliberately explicit rather than
automatic: it is only legitimate with positive evidence that the peer never
received what was encrypted under the old session.

**Still unknown:** the version-2 `phash` construction, whether our own companion
devices belong in the fan-out, what sets `includeSenderKeysInMessage` in
`LX/Ceo`, and the membership set behind `phash`.


## The fan-out works; one device asks for a retry (2026-09-24)

The post-send window caught what five earlier attempts never did. From a fresh
account (profile `l`) into the operator's four-member group, the send reported
`addressing_mode=lid`, a computed `phash`, `recipients=3, devices=3`, a key query
for all three, and `fan-out server_acknowledged`. Holding the connection open for
45 seconds then preserved **one** frame:

```
receipt type=retry  participant=…650@lid/device=0
  retry count=1 v=1   <registration> 4 bytes
```

**Two of the three addressed devices sent nothing back, and only the operator's
handset asked for a retry.** A recipient that cannot decrypt sends a retry
receipt; the two silent ones therefore decrypted the message. So the stanza, the
sender key, the `skmsg` and the per-device distribution are all working — what
five sends from the previous account looked like ("nobody can read it") was one
device's problem seen through a single reporter.

This also explains the silence before it: we were closing the connection the
moment the `ack` arrived, so every retry receipt was sent to a socket we had
already dropped.

### What the receipt asks for, and what we do about it

`retry count=1 v=1` with the recipient's current registration id is WhatsApp's
standard "I could not decrypt that, send it again" and the sender is expected to
answer — re-encrypting to that device, and for a group re-sending the sender-key
distribution with it (`forceSenderKeyDistribution` in `SendE2EMessageJob` is the
flag for exactly this). The registration id lets the sender notice that the
peer's identity changed and fetch a fresh bundle rather than reusing a dead
session.

**We have never answered one.** That is both why this particular message stayed
unreadable and, plausibly, part of why earlier accounts looked abnormal: a client
that is asked to re-send and never does is not behaving like a client.

Implementing the response is the next step and it is ordinary protocol
behaviour, not an extra live experiment.


## Resolved: the payload was padded twice (2026-09-24)

**Group sending is live-verified.** The operator confirmed a fan-out from
profile `l` reads correctly on their handset. Send, sender key, distribution and
`skmsg` all work.

The cause was ours, in our code, the whole time.
`direct::prepare_payload` has always applied the APK's E2E padding itself
(`LX/Cb1.A00`: 1..=16 bytes, each equal to the count). The group send padded the
sender-key distribution **again** before handing it over. The recipient strips
one layer and is left with 1..=16 junk bytes on the end of the `Message`
protobuf.

That one bug accounts for every confusing observation in this document:

| Observation | What it actually was |
| --- | --- |
| "Waiting for this message" | the leftover bytes corrupted the SKDM, so no sender key installed |
| "Your version of WhatsApp doesn't support it" | the sender key worked; the leftovers corrupted the *message* payload |
| Two of three devices decrypted one send | protobuf ignores trailing bytes that happen to spell a valid unknown field, and rejects ones that do not — the padding is **random**, so each device each send was a coin flip |
| One send worked, the next did not | same coin flip |
| Every structural correction changed nothing | `addressing_mode`, the version 2 `phash` and the AD-form `<to>` are all genuinely correct and none of them was the cause; the envelope was being fixed while the contents were corrupt |

Padding is now the encrypting layer's job on both paths:
`encrypt_group_payload` applies it exactly as `prepare_payload` does, and no
caller pads. `the_group_path_pads_exactly_once` drives a bare protobuf through
the group path and requires it back byte for byte, so a second layer cannot
return unnoticed.

**The lesson worth keeping:** an intermittent, device-specific,
"sometimes it decrypts" failure in a protobuf payload is a padding or framing
bug, not a protocol one. Two accounts were spent looking at the envelope.


## Replies and reactions in a group (2026-09-24)

Both are the same fan-out carrying a different payload, so nothing about the
stanza changes — only the `Message` inside it.

A **reply** is `extendedTextMessage` with a `contextInfo` naming the quoted
message (`stanzaId`, `participant`, `quotedMessage`), which the receive path has
decoded since the direct-chat work.

A **reaction** needed a new builder. `reactions::payload` deliberately leaves
`key.participant` unset, because in a direct chat the chat and message id
identify the message on their own. In a group they do not: the same id from two
members is two different messages, and `participant` is exactly the field the
receive path reads to attribute one. `reactions::group_payload` sets it, and
refuses a direct chat so the two cannot be confused.

The private JSON expresses both through one `quote` object, distinguished by
whether it carries an `emoji`. A reaction is also the one message with no body
of its own, so the "exactly one of text/image/voice_note" rule now expects zero
for it — a reaction with a body, or a reply without one, is rendered as
something else entirely by the recipient.

Both were transmitted live and acknowledged on 2026-09-24; handset rendering was
not separately confirmed.

### Correction: a reaction is not a text stanza (2026-09-25)

"Nothing about the stanza changes" was wrong for reactions. The direct path has
always sent a reaction as `type="reaction"` with `enc decrypt-fail="hide"` and
`<meta content_type="add_on"/>` (`LX/BpS` case 1, observed on the official
client's own reactions; without the `meta` a recipient did not display ours).
The group reaction went out as a plain `type="text"`. It was acknowledged, which
only says the server accepted the envelope, not that a handset rendered it.

`sender_key_stanza` now takes a `StanzaContent` (`Text`, `Reaction`,
`Media(kind)`) and applies the same markers as the direct envelope for that
content. **Inferred:** they go on the top-level `skmsg` `enc`, the one holding
the content; the per-device encs carry only the sender-key distribution and stay
minimal. `LX/Cp3->A09` is the shared `enc` builder that emits `mediatype` and
`decrypt-fail` under their own conditions; which of a fan-out's encs those
conditions select was **not traced**. A group reaction with these markers has not
yet been confirmed on a handset: one (`❤️`) was sent and
acknowledged on 2026-09-25 from profile `l`, with no retry receipt in the window.

### Retry answers now keep the original stanza's shape (2026-09-25)

A retry resend reuses the original message id, and the recipient matches it to
the message it is holding, so it has to be the *same* stanza apart from the
single `<to>`. It previously always carried `addressing_mode` and `phash` — also
on a status, whose original carried neither — and always `type="text"`. It now
reuses exactly the type, markers and attributes the original send used.

Two robustness fixes in the same window: frames that arrive while a retry's key
fetch is in flight (for example a second device's retry) are queued and handled
instead of discarded, and one device's failed answer is reported and skipped
rather than ending the window for the rest — the send itself was already
acknowledged. Frames that arrive while the send waits for its own replies
(membership, device lists, key query, distribution acks) are now preserved to the
receive path's content-hashed capture keys, so `lab-inbox` can open them.

### A refused participant is not a member (2026-09-25)

**Observed** 2026-09-22: a `group create` reply listed a `<participant>` carrying
an `error` attribute. `parse_group` counted it, so the snapshot said two members
while the next `group info` said one, and the fan-out then had nobody to address.
Such entries are now skipped.

### Group reactions still do not render (2026-09-25)

The operator reports that neither the `👍` (`l-react-1`, sent as `type=text`)
nor the `❤️` (`l-react-2`, sent as `type=reaction` with the add-on markers)
appears. Typing indicators into the same group do appear, so the connection and
addressing are fine. What was checked, all **observed** from saved captures:

* **Payload layout matches the official client's.** The operator's own group
  reaction decodes as `Message{46}`, `ReactionMessage{1,2,4}`,
  `MessageKey{1,2,3,4}` with a LID `participant` — the same fields ours sets
  (`reactions::shape`, printed by `lab-inbox --inspect`).
* **Stanza matches.** Their incoming reaction is `type=reaction`, `skmsg` with
  `decrypt-fail=hide`, `<meta content_type=add_on>` — what we now send.
* **Target is right.** `AC742884…` is a group text from the operator's LID
  (`participant_tail` in the inspector), 90-day disappearing timer, ~29 h old
  when reacted to, so it had not expired.
* **The ack differs.** Every group text fan-out was acked `count="3"`. The
  `type=reaction` send was acked with **no `count`** and with
  `addressing_mode="lid"` echoed. `l-react-1`, sent as `type=text`, was acked
  `count=3` like a text. What an absent `count` means for a group ack was **not
  established**: the handler first traced (`LX/DMA->run`) turned out to be the
  status ack path, which parses `addressing_mode` as `pn`/`lid` and `count` as
  an integer but says nothing about groups.
* No delivery receipt came back for either reaction; the texts and the reply
  did get receipts. Whether the official client receipts reactions at all was
  not checked, so this is not by itself evidence of non-delivery.

`decrypt-fail="hide"` means a reaction the phone cannot decrypt is dropped
silently, with no retry receipt, so decryption failure cannot be ruled out from
our side.
