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
direct Signal session (`comparison-direct:<selected PN or LID>`), so the session, pinned
identity, any consumed one-time prekey and the sender key all commit together.
A direct (non-group) message carrying field 2 is handled the same way.

**Participant identity**: `participant_pn` carries the member's phone JID when
`participant` is a LID (`LX/ADw->A02` reads it as a `UserJid`, **confirmed**;
it is the group analogue of `sender_pn`). `participant_lid`, `sender_lid` and
`peer_recipient_pn` also exist in this build. When the attribute is absent, a
LID participant may use the `authenticated-lid:` checkpoint the direct path
writes. A held LID session takes priority over a PN alias; an unmapped LID can
also use its held LID session. Without either mapping or session, its SKDM child
is skipped and only a previously stored sender key can decrypt the `skmsg`.
See the dated session-selection correction below.

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

**Fresh-target retest (2026-09-25):** a `👍` to a message the operator had sent
seconds earlier (`l-react-3`) was acked the same way — no `count`,
`addressing_mode="lid"` — with no retry receipt in the 45-second window, and it
did not appear on the handset. So a stale or expired target is ruled out. In
the same session the operator's own reaction to our `l-reply-1` arrived, which
confirms that group reply rendered. The reaction cause remains **unknown**.

## A group payload addressed to one device (2026-09-28)

**Observed:** a group stanza from the operator carried a per-device `msg` and
no `skmsg`. The `msg` decrypted to `Message` fields `[35, 70]` — field 70 is
`messageHistoryBundle` (`xtask apk-proto-fields` on `LX/BZc`, the `Message`
descriptor), the group-history share a member's phone sends a newcomer. It is
not a key distribution, so the inbox found no group key to commit and returned
`Unsupported` without saving, acknowledging or receipting it — which is why the
server redelivered it at every login.

Now, when a group stanza has no `skmsg` and its per-device payload is not a
distribution, that payload is recorded as the group message: the member's
session and any consumed prekey commit, the group sender key is left as it was,
and the normal delivery receipt follows. Verified offline by replaying the
preserved capture (`lab-inbox --replay-captures`: 1 newly decrypted). The
history bundle's own contents (an encrypted media reference) are not fetched
or applied.


## Group pairwise PN/LID session correction, 2026-10-07

**Inferred from APK addressing; observed offline:** `participant_pn` is a contact
alias, not a ratchet selector. The existing direct-path authenticated recovery evidence
([LID receive note](messaging-transport.md#lid-senders-decrypt-with-the-lid-session-2026-09-28))
shows LID session addressing, while the earlier note attributes `LX/ADw.A02`
reads the group participant PN alias. The earlier group implementation always
selected that PN session, even when a different LID session already existed.

After the explicit authenticated identity policy recovered a direct capture in
an isolated store, a group pairwise capture still reported a missing/invalid
one-time prekey. Selecting its held LID session recovered it: **one newly
decrypted, 35 already committed, zero still unsupported** on replay of the
original captures. The missing-prekey result was produced by our wrong session
selection; it did not establish that the required key was globally absent.
The original account database stayed byte-for-byte unchanged. No receipt,
network request, key invention, identity reset or plaintext export occurred.
This is authentication/decryption of a captured group payload, not handset
confirmation or proof that it is a fresh human reply.

**Implemented:** LID participants prefer a held LID session; a mapped prekey
message establishes its session under the LID, as direct receive already does.
A normal `msg` falls back to PN only when no LID session is held. An unmapped
LID may use a held LID session; absent both mapping and session, it is still
skipped. PN participants retain PN selection. There is no fallback on a failed
MAC: that would risk advancing an unrelated ratchet. The selected session,
identity, SKDM and inbox still share the existing atomic commit.

Regression coverage exercises actual generated group prekey decryption with
an incompatible PN session beside the valid LID session; it also covers PN
fallback, mapped new prekeys, unmapped held sessions and malformed PN aliases.
Companion-device addressing remains as before; full device-specific receive
routing is not established by this primary-device recovery.


**Observed offline, subsequent canonical recovery:** after preserving the
original configuration/state and selecting the APK-compatible incoming policy,
the real saved account recovered the pending direct and group captures
(two newly decrypted, 34 duplicates, zero unsupported). The local account
identity and all send IDs/reservations were preserved; previous peer state is
archived atomically. No receipt/network request occurred in replay.


**Negative static result / attribution correction:** scoped disassembly of
current comparison APK 2.26.37.71 (SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`)
shows `LX/CPQ.A00(ZL)` as a cached message-policy boolean helper with fields
`0bA`, `0Fs` and a concurrent map. It does not construct a Signal session
address. The older direct note's unversioned `CPQ.A00` citation must not be
used as confirmed 37.71 addressing evidence. The original reference build
and a replacement static producer anchor remain **unknown**. The group
session-choice correction is independently **observed offline** by successful
message authentication against the original server captures; its APK routing
producer still needs re-anchoring by strings/callers rather than class-name
reuse. The inherited `ADw.A02` alias citation was not independently retraced in
this change. No claim here derives a mandatory field from its mere existence.

## Saved group notifications and inspection correction (2026-10-07)

**Confirmed/static** for APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
Fbq.<init> 0006..0014 registers literal `w:gp2` through EGu. ADw.A0G
0004..001a checks notification, reads group route and notify metadata, type
w:gp2, timestamp/offline fields, and builds 9P6/9PA at 0146..016e.
This is a parsed notification holder, not the final membership application.
9PA's four constructors are compiler-merged variants; its name alone cannot
identify one meaning. A holder trace is retained privately.

The exact groups_dirty anchor uniquely resolves 0yv.A09. Its branch at
2912..298e iterates child `group` nodes, reads each `jid`, logs/continues on a
missing JID, and accumulates JID/reason pairs. At 6548..6570 it queues
FetchTruncatedGroupsJob with that list, an empty HashSet and final params value 3 (the job constructor stores it as params, not a guessed wire reason).
Thus groups_dirty is a refresh instruction, not a complete member snapshot.
The job's exact request fields and response application remain **untraced**;
a blind generic group-info request is not established by this partial trace.
[Scoped label/parser/dirty evidence](group-notification-evidence.txt) retains
these instructions. Full raw handler (over 6,000 lines) stays in ignored lab.

**Confirmed local defect:** lab-inbox --inspect's lexical type filter excluded
all digits, so a real w:gp2 capture printed type=none. The correction permits
only this exact APK literal in addition to the existing bounded alphabetic
filter. It does not permit arbitrary numeric-containing type strings, IDs or
addresses. Tests cover recognized types, w:gp2, unknown w:gp3, numeric address,
long/Unicode/newline values and absence. CLI/API inspection metadata is updated;
no handler, ACK, query, membership write or verification decision is added.

**Observed/offline:** inspection of the saved profile l after the rebuilt CLI
exposed 26 group-notification shape lines, comprising create with no attributes,
create with key/type attribute names, and groups_dirty. That count is the
inspector's distinct shape-line output, not 26 independently delivered groups:
its set includes ID/time metadata and historical/duplicate captures. No group
address, key/type attribute value, subject, member or raw notification body was
printed. Private before/after logs are under
lab/accounts/l/document-receive-check-2026-10-07/inbox-inspection*.log.
The full saved history was inspected, not just the latest receive window.

Other preserved shapes include mex/update[op_name] and psa campaign[id,duration],
surfaces, messages[campaign_id] and psa_wakeup[psa_push_id]. Fbs.<init>
0006..0014 statically registers psa. No op_name, campaign or push ID values were
examined or applied. Their payload semantics and ACK policy remain **unknown**.
Exact GroupNotification string search found no literal reference; groups_dirty
found the real handler, refuting any inference of absence from the first search.
0yv has no A00 override in the scoped method lookup; inherited ACK construction
must still be traced before implementing notification acknowledgements.

**Current client boundary:** groups::ingest consumes settled group action
replies; connection dispatch does not apply w:gp2 notifications. The original
notifications remain encrypted captures. Generic incoming outcome=Unsupported
does not prove every side handler is absent (encrypt/devices have separate
handlers); code inspection establishes this particular group gap. Next work is
the evidenced groups_dirty job/request/commit path and create application with
route/membership/duplicate guards. No saved account keys, sessions or identities
were replaced and no network request occurred during this inspection.

**Confirmed/static job boundary:** FetchTruncatedGroupsJob.<init> stores
round, maxNumberOfRounds, params, groupsToFetch, successfullyProcessedGroups
and batchContext at 0036..004e; it requires chat connection (0018..0028).
A0G drops a stale job with null batchContext (0004..0018), calculates a
participant budget, bounds it with feature 0x187b (0082..0098), and selects
between 9x2.A00 and 9qZ paths behind feature 0x344e then 0x2a35
(01bc..0208). The alternate path's enum branches choose operation labels
per_group_dirty_recovery_truncatable, per_group_dirty_recovery,
get_participating_groups_paginated or inactive_group_migration
(0252..0274,0310..0322). Their serializer/request fields are still untraced.
Current feature values are unknown; a wire operation cannot be selected just
from its presence. Later rounds retain processed groups and queue another
job (0446..0492); implementing unbounded refresh/retries would not reproduce
that bounded state. Our general action queue's uncertain reservations must
remain intact when this protocol is added.

**Tooling unknown:** A0G's memory-budget arithmetic contains DEX opcodes cd/ce
at 003c..0044, displayed as opaque by the current disassembler. Their precise
instruction rendering/formula has not been asserted here. Raw job dump and
callers remain local; [scoped evidence](group-notification-evidence.txt)
includes constructor, relevant feature/operation branches and next-round setup.
The inherited notification ACK writer, batch context definition, concrete wire
serializers, membership/application guards and crash/deduplication behavior
are the next trace boundaries. No server query was guessed from this finding.

**Negative dispatch search:** apk-callers for the concrete 0yv.A09 returned
zero callers. This does not refute its override-based use: the inherited
0yO notification dispatch is the remaining caller/ACK trace boundary. The
scoped A00 lookup found no matching method in 0yv, consistent with inheritance,
not proof that an ACK is absent from the APK.

Validation: 568 workspace/all-feature tests, 24 default-feature CLI tests,
42 final all-feature CLI tests, strict CLI all-target/all-feature Clippy,
CLI reference regeneration/test, formatting and repository hygiene (374 local
links) passed. Rebuilt-CLI saved-account inspection exposed the previously
hidden group type without network I/O, replay or notification application.


## Dirty refresh request and inherited dispatch trace (2026-10-07)

**Confirmed/static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Scoped method evidence](group-refresh-request-evidence.txt) preserves the
context, concrete request constructors, arithmetic and inherited ACK boundary.
This follow-up resolves the preceding serializer and opaque-opcode unknowns;
it does not establish current feature settings or implement notification handling.

`8lX.<clinit>` maps ordinal 0/A04 to PER_GROUP_DIRTY_RECOVERY (0000..0014),
1/A05 to PER_GROUP_DIRTY_RECOVERY_TRUNCATABLE (0014..0028), 2/A02 to
GET_PARTICIPATING_GROUPS_PAGINATED (0028..003c), and 3/A03 to
INACTIVE_GROUP_MIGRATION (003c..0050). The previously traced `0yv.A09`
`groups_dirty` branch (6548..6570) uses A05, so its context is specifically
truncatable dirty recovery. Constructor `params=3` is a stored job argument,
not evidence of a wire attribute. Its supplied round is not traced upstream.

**Confirmed/static participant budget:** `FetchTruncatedGroupsJob.A0G`
0020..0046 converts `Runtime.freeMemory()` to double, multiplies by the
literal double 0.15, divides by double 60, then narrows to int. 0046..0052
floors this memory-derived budget at 10000; 0082..0098 caps it using feature
0x187b. The formula before the feature cap is therefore
`max(10000, int(freeMemory * 0.15 / 60))`. This is APK code, not a measurement
of our account, a known server limit or a recommendation for our runtime.
The disassembler now names cd/ce at 003c/0042; its previous opaque rendering
was a tooling limitation, not an unknown Dalvik operation.

**Confirmed/static IQ branch:** the job creates `AaH(iqId, context,
groupJids)` at 0288..028e, obtains its lazy packet at 02d2..02ea and sends via
`0h8.A0T` at 02ee..030c. `AaH.<init>` 0006..0022 stores these values and
selects `Aib` case 25; `AaH.toString` explicitly names them.
`Aib.A01` creates the lazy wrapper. `Aib.invoke` case 25 (0546..05a6)
constructs one `9QE(groupJid, 11)` per entry, then
`EJy(iqId, context, entries, 4)`. `9QE.<init>(VLI)` selector 11
(0010..0018,003e..004a, shared 0018..003e) builds `group` with `jid`.
`EJy.<init>(VLLLI)` selector 4 (0144..0214) builds:

- an `iq` root, a `to` attribute containing `9Lp.A00`, `xmlns=w:g2`,
  `type=get`, and an ID through `8ka.A0a`;
- a `query` child with `context` selected from
  `per_group_dirty_recovery`, `per_group_dirty_recovery_truncatable`,
  `get_participating_groups_paginated`, `inactive_group_migration`;
- the per-group children, after `0hp.A07(entries, 1, 10000)`.

`8kV.A0l` 0000..0010 confirms the root tag; `8kX.A0T` 0000..0012
confirms `type=get`. `9Lp.<init>` constructs an empty-user JID and
`getServer` returns `g.us`; its exact raw-string encoder has not been traced
here. `8ka.A0a` 000a..0028 calls the ID guard with 0..9007199254740991
and `false`, then emits `id` at 0028..0038 when the guard returns true.
**Unknown:** the full ID/list guard semantics (including error behavior),
actual generated ID, current list size, complete response parser and
membership/application guards. Passing bounds to a guard is not proof of its
refusal behavior. **Negative:** `9qZ` has only a constructor in the scoped
lookup; its presence as a dependency is not the concrete request serializer.
`EJy.A00` is a getter, not the constructor recovered above.

**Confirmed/static MEX branch:** feature 0x344e then 0x2a35 selects
`9x2.A00` at job 01bc..0208. `9x2.A00` 001a..0068 creates each entry with
`group_jid` and `query_context=INTERACTIVE`. Its ordinal mapping
(0068..008a,00f2..0102) selects the uppercase enum name for
`request_context`; 0098..00b2 assembles `groups` and `request_context`.
00b2..00d2 builds the `QueryBatchGetGroups` operation with caller
`whatsapp-android-mex`, class `8z9`, and dispatches through `1xk.A0a`;
00d4..00ea installs an `AjS` callback. **Unknown:** operation/document ID,
complete MEX envelope, response validation and active feature values. These
strings do not establish a HTTP route or justify guessing a GraphQL request.

**Confirmed/static inherited handler ordering:** `0yO.A01` invokes the
virtual `A09` at 0206..020c, explaining the earlier zero exact callers of
concrete `0yv.A09`. On the traced normal path it subsequently checks the
pending `CUu` and map state, then calls `0yO.A00` at 02c0..02c6.
`0yO.A00` branches on handler `BPe`: one path queues `CZF` through
`18B.A0C` (0012..0050), the other calls `0h8.A0N` (0052..006c).
`0h8.A0N` passes `1B3.BPe()` into `A07`. `A07` optionally records
`ccq_android_ack_bypassed` telemetry (0000..0058), then calls `1q7.A03`
(0058..0068) and sends through `A0S` (006e..0074).

**Negative/unknown ACK boundary:** `1q7.A03` reads `Cb1` metadata, calls
`A08`, then creates an Android message with selector 0x4c (0028..0038).
It is not the binary ACK serializer. That selector's send handler and the
`Cb1` to wire-node mapping remain untraced. Current `BPe` values, exception
paths and complete pending/map behavior are unknown. Do not implement group
notification ACK attributes from this intermediate object or infer that every
notification receives an unconditional immediate ACK.

**Current client boundary:** no dirty-refresh request, notification application
or new ACK was added here. Saved captures, identities, keys and reservations
remain intact; no account was opened and no live request was made for this
trace. The next implementation needs the actual response/application and
wire ACK evidence, with durable-before-ACK, membership and duplicate guards.
Validation: 32 xtask tests and strict all-target xtask Clippy passed; the rebuilt
tool names both instructions at the actual APK method offsets.

Final checks for this change: generated CLI reference test, all five OpenAPI
coverage/HTTP/library-symbol checks, formatting, staged whitespace and repository
hygiene (379 local links) passed. These are local checks, not live group refresh
validation.


## Batch response and current-build ACK writer (2026-10-07)

**Confirmed/static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Scoped response/ACK evidence](group-refresh-response-evidence.txt) extends the
preceding trace; no account was opened and no query/ACK was sent.

**Resolved request guard semantics:** `0hp.A06` validates Java string length,
not a numeric value. At 001c..002a it obtains `String.length()` and compares
that length to the supplied lower bound; 0054..0072 checks the upper bound.
The ID builder's 0..9007199254740991 arguments are therefore string-length
bounds. Null with optional=false logs and returns false (0000..0016).
An out-of-bound non-null string logs then returns true (002e..0054,
005c..0072). `0hp.A07` similarly logs null then returns false (0000..0012),
but an out-of-range non-null child list logs then returns true
(0022..0064,0064..0098). `00K.A0D` constructs/logs an AssertionError and
returns (0000..0016); it does not throw that error. This refutes interpreting
these writer diagnostics as mandatory refusal guards. A non-null empty list
can still produce an empty query. Receiver validation is a separate path.

**Resolved service spelling:** `Jid.getRawString` 0000..0016 delegates user
and server to `0D1.A01`. That helper returns the server directly when the user
has length zero (0010..001e), otherwise joins user, `@`, server. Combined with
`9Lp`'s empty user and `g.us` server, the batch IQ destination renders as
`g.us`, not an invented user at that domain.

**Confirmed callback selection:** the job's `Aov` selectors 10 and 11 both
reach the default batch parser branch (the packed switch has cases 0..9).
`9R2.<init>` stores the two parser callbacks as A01/A00 and delegates to
`AWW.<init>`. Inherited `AWW.C8f` calls A01 then passes a `9R6` result to its
completion callback; `BmU` calls A00 then passes `9R3`. `BjL` passes `9R4`
containing `HFb`. A search for concrete `9R2.C8h` had no method body, not
proof that the class or callback dispatch was absent. This was also a defect
in our disassembler's empty-result diagnostic, corrected below.

**Confirmed batch response parser:** `Aov.invoke` default branch
0016..0366 handles `AaH` plus the returned stanza. `8kX.A0W` requires the
`iq` tag and retrieves the request template (`EJy.A00`), not the response body.
The success branch reads template `to` and `id`, then corresponding response
attributes (007e..0118); `8lB.A0L`'s string/JID paths perform expected-value
comparison when provided (01da..01ea,0300..036e). It separately checks type
against `result` (0118..0144). It requests children at path `groups/group`
(0144..017c), with decoder `AZM` selector 15 and bounds 1..10000.

`8lB.A0O` traverses the parent path, decodes each matching child and skips
null decoder results (000a..008a), then validates the count of decoded results
(008a..00da). Missing path/count failures return null with a diagnostic;
this is stricter than the request-writer assertion behavior above. It does not
establish that every raw group child must decode successfully.

`AZM.AAK` selector 15 (041c..050c) requires `group`, reads optional
`create_ctx` from e2ee_migration/regular/rtc and `key`, then dispatches the
`GroupInfo|TruncatedGroupInfo|GroupForbidden|GroupNotExist` union using
AZK/AX9/AXA/AXB. `Aov` iterates decoded `Asn` values through `9rm`
(0182..01ca), and returns `A9t` carrying a result list and map (01cc..01ec).
The client-error path names `IQErrorBadRequest|IQErrorRateOverlimit`
(0266..02c6); the subsequent server-error path uses `AZN` selector 6
(02ee..0344). Failed parsing accumulates BatchGetGroupInfoResponseSuccess,
ClientError and ServerError diagnostics before throwing (0228..0366).
Exact codes, all union validators and server wait interpretation remain
**unknown**; the strings alone do not authorize retries or a refusal reset.

**Confirmed model boundary:** `9PN.A74` and `9PP.A74` call `ADs.A0C`
with mode 3, using Aou callbacks 23/24 and 21/22 respectively, then append
the converted result. `9PO.A74` decodes its A02 string as a group ID and
stores its A01 string in the map; the exact semantic mapping of that string
is not established here. `ADs.A0C` reads `id`, then `truncated`; a true
truncated value reads `size` and creates `A9s` without a full `AAO`
(000c..0078). The nontruncated path builds `AAO` then `A9s`
(02dc..02f0). Thus a truncated result is not a complete membership snapshot.
The complete nontruncated participant/attribute parser remains to be traced.
Creation-time arithmetic at 00be is still opaque opcode 9d in our tool;
its scale is not asserted from that rendering.

Aou cases 21/23 share 0764..0792: they call `0yw.A0W` only when the
A9s.A03 flag is false and A9s.A00 contains an AAO. Cases 22/24 report a
participant-attribute-parser diagnostic (0792..07cc). `0yw.A0W` builds
LID/PN mappings from two AAO pairs, calls A0r if nonempty, conditionally A0s
behind feature 0x36fe, and forwards a nonempty A0M map to `0r0.CRs`.
**Negative:** this method is not a complete group membership commit; no
transaction or full group snapshot update was established in its 0000..009e
body. Membership replacement, dirty-state clearing, crash/deduplication and
these feature values remain **unknown**.

**Resolved ACK writer:** the 37.70 `Cns` anchor has no matching class in
37.71 and no exact A05 callers. An exact field-read search for
`Cb1.A06:String` returned 25 sites, including `Cp2.A05` in classes7.dex
at absolute DEX offset 0x6aa2a0. There are four exact Cp2.A05 callers:
0y0.A00, 0y2.A02, Cgl.A01 and 1q7.A07. `0y0.A00`'s selector 76/0x4c
at 00f0 resolves to 11c2..11d2, calls `0y2.A02`, which passes Cb1.A04
and Cb1 into `Cp2.A05` (0008..0014), then writes via `1Rd.A04`.
The earlier `0yG.handleMessage` search was a dead end: its cases 1..8
cover callback/timeout handling, not the selector-76 outbound writer.

`Cp2.A05` builds `ack` with optional ID, destination, class, type,
participant, recipient and edit, followed by any additional Cb1 attributes
and optional child (0036..00f8). Notification type is preserved; only the
receipt/delivery combination strips type (0004..002e,0116..011c).
Participant is included whenever non-null here; do not copy the separate
receipt writer's participant-versus-destination suppression into this ACK.
Edit is omitted when empty or literal 0. Missing ID/class logs a diagnostic;
our own runtime may enforce stronger local validation, which must be named
as our policy. The inbound stanza-to-Cb1 producer is still untraced here;
`0zd.A03` is a read/played readreceipts-feature helper, not that producer.

The ACK serialization boundary is now established, but which group events
are fully and durably processed remains unfinished. Unknown notifications
must remain captured. The next implementation needs the complete group
response-to-snapshot/dirty-state path and authenticated original-route
validation before enabling application and acknowledgements.

Final local checks for this trace: generated CLI reference, five OpenAPI
coverage/HTTP/library-symbol tests, formatting, staged whitespace and repository
hygiene (383 local links) passed. No live group notification application was
validated.


## Truncated snapshot protection and completion boundary (2026-10-07)

**Confirmed/local bug and correction:** `parse_group` accepted a reply with
`truncated=true` as a complete snapshot. `record_info_reply` and queued
`apply_reply` then replaced the saved members, potentially emitting false
participant removals or replacing the subject with missing data. Both paths
now reject a truncated reply with a specific refresh diagnostic before any
snapshot/event write. `parse_group` returns None for such an element. A saved
settled action retains its response and lacks the ingestion marker; reopening
and another local ingestion remain diagnosable rather than reporting completion.
No automatic request, retry, reservation reset or notification ACK is added.

**Evidence:** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
`ADs.A0C` 002e..0078 returns id/size before building full group data, as
recorded in the preceding [response evidence](group-refresh-response-evidence.txt).
[Application evidence](group-refresh-application-evidence.txt) adds both
`A9s` constructors: `(VLL)` sets A03=true and AAO=null (000e..001a), while
`(VLLL)` sets A03=false and stores full AAO (000c..0018).
`8lQ.A01` separates A03=true entries from full ones (016a..018e); full
entries reach `8lQ.A00`, which requires AAO (0004..000c) and throws
"Expected group data info but got null" at 01b6..01c4. This independently
confirms that truncated results cannot replace full membership.

**Local policy:** absent `truncated` or case-insensitive false permits full
parsing; case-insensitive true is rejected. Any other or non-text value is
rejected as an invalid truncated flag. The latter is stronger than Java
Boolean.parseBoolean's false-for-other-strings behavior, not a server rule.
The `size` attribute is not treated as a member list. Full empty-member results
remain parseable; this guard does not infer truncation just from an empty list.
The complete APK membership/attribute parser is still unfinished.

Reproduce without an account or network:
`cargo test --locked -p whatsapp-api --all-features truncated_replies_preserve_members_events_and_saved_action_after_restart`.
This regression seeds two members, submits a flagged partial result containing
one member through both direct and saved-action paths, reopens the actual
encrypted store, and checks exact snapshot equality, one original creation
event, retained reply and absent ingestion marker. A second test covers
absent/false/mixed-case/invalid flags. Existing full-result tests continue to
cover normal replacement and events. `group list` exposes the refusal through
its existing ingestion path; `actions` inspects the saved action. Refresh must
be an explicit supported operation until the APK batch flow is implemented.

**Confirmed/static completion handoff:** the job creates `AjR` selector 12,
not the earlier guessed Au1 callback. `AjR.invoke` 0e62..0f62 handles `9R6`
with A9t, reports nonempty errors, passes its result list/context/round/params
to `8lQ.A01` (0f14..0f2e), combines the error map with returned pending map
and processed set, and completes AzY (0f30..0f5e). `8lY.<init>` stores
A00=first map, A01=second map, A02=processed set. Searches for A03.invoke
and Au1.invoke returned no method bodies. A03's recovered constructor stores
two integers; it is not evidence of an invoke callback.

`8lQ.A01` distinguishes legacy and later-round/context paths. It applies full
entries separately, preserves truncated JID/size pairs in the pending map
(02ec..0312), and constructs 8lY with pending map plus processed set
(0314..0326). Some contexts enqueue FetchTruncatedGroupsJob (029a..02bc).
The full-round and participating-groups context conditions must be retained;
this is not a reason to auto-query indefinitely or clear a partial batch.

**Confirmed/static, still partial application trace:** `8lQ.A00` passes
AAO fields into `0yw.A0U` at 0184..018a and later invokes `246.A02`
(018e..01b4). The full underlying commit/membership transaction is still
untraced. `0yw.A0t` calls 8lc.A01, 8l2.A00, 8l3.A01, per-group 8ld.A00
and 1BZ.A0L; those database semantics remain **unknown**. Do not label it
proven dirty-state deletion from the method name or call alone.
`0yw.A0u` is labelled onGroupSyncSucceeded and compares stored group JIDs
against the passed set (0034..008a), then reaches feature-gated absent-group
operations behind 0x2c53 (008a..0108). This broader reconciliation cannot be
copied onto an arbitrary partial batch. Exact deletion/state semantics and
active feature value remain **unknown**.

No saved account was opened or live query was sent for this correction. The
live occurrence of a truncated reply against our client is unverified.
Validation: 571 workspace/all-feature tests, 24 default-feature CLI tests,
strict workspace/all-target/all-feature Clippy and warning-free Rustdoc passed.

Final checks: generated CLI reference, all five OpenAPI coverage/HTTP/library
checks, formatting, staged whitespace and repository hygiene (387 local links)
passed. The added non-text-flag regression also passed independently.


## Atomic queued group ingestion (2026-10-07)

**Confirmed/local defect and correction:** queued group replies used separate
transactions for applying the snapshot/events and marking the action ingested.
A stop between those commits left the earlier action eligible for reapplication;
concurrent callers also checked that marker outside the write transaction.
A newer intervening reply could then be overwritten by reapplying an older
saved result. This is our persistence gap, not a WhatsApp requirement.

`ingest_reply_in` now checks the marker, applies the reply and saves the marker
under one store write transaction. Snapshot replacement and its journal events,
invite-code changes and leave deletion all use that same transaction. The
immutable stored request is loaded before the write lock so invite/leave do not
nest transactions. A rejected truncated response still rolls back without a
marker. Direct `record_info_reply` continues to save snapshot/events atomically;
it has no queued-action marker. No socket query, receipt, server retry or new
request ID was introduced.

**Confirmed/offline regression:**
`ingestion_snapshot_invite_leave_and_marker_rollback_together` exercises a
saved successful info, invite and leave reply in turn. It aborts the actual
transaction after both application and marker writes, reopens the encrypted
store and checks the original snapshot, original event count and missing marker.
Normal ingestion then applies the retained reply once. Another reopen preserves
the marker and does not repeat the application or event. The subject-update
case proves event rollback, the invite case proves request-derived mutation,
and leave proves deletion rollback. This is transaction-abort/reopen coverage,
not a claim of disk-full or process-kill fault injection.

Reproduce without an account/network:
`cargo test --locked -p whatsapp-api --all-features ingestion_snapshot_invite_leave_and_marker_rollback_together`.
The existing `group list` command invokes ingestion; `actions` retains the
original queued action inspection path. No additional command is needed.
Ordering between distinct replies and complete notification application remain
unfinished; an atomic commit alone does not establish freshness or membership
validation for all inbound group shapes.

**Confirmed/static helper follow-up, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Scoped helper evidence](group-completion-helper-evidence.txt) resolves two
parts of the previous `0yw.A0t` boundary. `8lc.A01` iterates supplied groups,
requires an existing 0DV, tests 0Da.A1A and `0bA.A0m`, then calls `8lc.A00`
(0008..0076). The predicates and downstream operation remain **unknown**;
this is not established as dirty-state deletion. `8l2.A00` selects group-like
JIDs via `0DE.A0n` and `0Fs.A0A == 1`, then enqueues
MemberSuggestedGroupsSyncJob for their raw JIDs (002a..009c). This is a separate
member-suggestion sync job, refuting the assumption that every A0t helper is a
simple dirty-marker clear. The value-1 predicate's meaning remains **unknown**.

`0yw.A0U` labels its entry onGroupInfoFromList (0004..000e), checks
admin/superadmin role strings from a map in the scoped 01c4..021a branch,
and includes many additional feature/state/system-message operations. Its
remaining field mapping, membership transaction and full parser are **unknown**;
this partial trace does not justify directly copying the whole method into
our smaller snapshot model. Raw full dumps remain local. Further static work
is still available; this is not an external blocker.

No saved account was opened or live request sent for this change.

Validation: 572 workspace/all-feature tests, 24 default-feature CLI tests,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated
CLI reference, five OpenAPI checks, formatting, staged whitespace and repository
hygiene (390 local links) passed. Group notification handling remains incomplete.


## Typed participant map and duplicate roles (2026-10-07)

**Confirmed/static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Scoped participant evidence](group-participant-evidence.txt) contains the map
parser, typed reader, factory and metadata helper boundaries.
`ADs.A0C` invokes `ADs.A05` for the member map at 01a6..01ac when its
mode's low bit is set; mode 3 from 9PN/9PP satisfies that branch. It derives
size from the returned map unless a size attribute parses (01ae..01d0).
This is a map-cardinality fallback, not proof size must match raw child count.

`ADs.A05` gets participant children (000a..0026), reads jid as UserJid and
skips a null typed result (002e..0046). **Refuted concern:** skipping missing
or uncastable participant JIDs is also APK behavior; it is not evidence by
itself that our snapshot must reject every such element. `0hs.A0A` uses a
cached JID attribute or the Jid companion parser, calls isProtocolCompliant,
logs noncompliance, and proceeds to the class cast (0000..00b0). Failed casts
log and return null (00b8..00f6). The compliance check is not an unconditional
refusal guard. Full companion factory and subclass compliance rules remain
**unknown** in this trace.

`1xf.A1H` constructs LinkedHashMap (0000..000c); `ADs.A05` uses Map.put at
016c..0172. Repeated typed JIDs therefore replace the prior value while
retaining first insertion position. The parser defaults missing type to an
empty string (0058..0074). No allowed-role enum check is present in that
scoped extraction; roles beyond the ordinary admin/superadmin vocabulary are
not assigned new semantics here.

**Confirmed class hierarchy:** APK class_def superclass indices establish
DeviceJid extends Jid directly (classes.dex 0x152e40), while PhoneUserJid
extends UserJid (0x152c00), which extends 0Cy (0x152be0). DeviceJid is not a
UserJid subclass through that branch. These offsets are class_def headers,
not method offsets. A local Python ZIP/DEX header read used class_defs_off
at header 0x64, 32-byte class definitions, superclass_idx at entry +8, and
type/string ID tables at header 0x44/0x3c. The output also locates 9Lm
extending 0ZX (classes6.dex 0x105c80); its full ancestry is not established.

**Confirmed/local mismatch and correction:** our Vec accepted any raw
participant jid string and retained duplicate entries. It now parses through
our existing Jid type, keeps supported user-like non-device identities, and
replaces a duplicate's role at its original position. Missing, unparseable,
non-user, empty-user or agent/device identities are skipped. This is our
supported user-level representation; it does not claim full equivalence to
every APK JID subclass or compliance predicate. Error-bearing participants
remain excluded by the separately observed create-refusal rule from 2026-09-22.
A later refused duplicate does not erase a previously accepted member.
Absent role still uses None in our model, representing the APK's empty default.

`member_map_uses_user_identities_and_last_duplicate_role` is a synthetic
fixture grounded in the typed-cast/LinkedHashMap branches. It combines a typed
PN and LID, a repeated text PN with a new role, an error-bearing duplicate,
missing/invalid/non-user/device/agent JIDs, then checks the saved unique members
and GroupCreated journal participant count. Existing refusal and full-reply
persistence tests continue to pass. No saved snapshots were rewritten or
backfilled: this behavior applies to subsequently ingested replies.
Reproduce with `cargo test --locked -p whatsapp-api --all-features
member_map_uses_user_identities_and_last_duplicate_role`. `group list` retains
the existing operator inspection path. A dedicated role-change event is not
implemented; role changes update the snapshot, while the existing event model
only describes subject and participant additions/removals.

**Confirmed/static extra fields, still incomplete:** ADs.A05 additionally
reads lid as UserJid (0074..008a), display_name with a StringUtils predicate
(008a..00a8), phone_number as UserJid (00a8..00f6), join_time accepted only
when a parsed long is positive (010e..0142), and group_history_sent through
0CM.A0P (0142..0160). A PN predicate and equality trigger a diagnostic for
phone_number with a PN jid (00bc..00f4), rather than a demonstrated refusal.
0DE.A0e/A0X, display-name filtering, history-flag conversion and the 32q
result factory remain **unknown**. Do not use these attribute names alone to
manufacture PN/LID bindings or change Signal identity pins.

**Negative producer correction:** ADs.A07 is not the primary member parser.
It gates on 8kV.A1V, builds username mappings from participant jid/username,
creator/creator_username, s_o/s_o_username and description
participant/participant_username (0026..0094). ADs.A06 gates on the same
predicate, reads creator as UserJid and creator_country_code, converts with
C9h.A00, then creates CTQ when both exist (0000..005e). The feature predicate,
country-code interpretation and complete alias application remain **unknown**.
These optional producers do not establish a mandatory registration/device field.
No alias, name, history state or country metadata is added to runtime snapshots
in this change. Full notification/batch application remains unfinished.

No saved account was opened or live request sent for this change; live
participant-map behavior against this client remains unverified.

Validation: 573 workspace/all-feature tests, 24 default-feature CLI tests,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated
CLI reference, five OpenAPI checks, formatting, staged whitespace and repository
hygiene (393 local links) passed. No live participant-map validation occurred.


## Notification metadata producer and initial refresh round (2026-10-07)

**Confirmed/static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Scoped disassembly and exact caller results](group-notification-input-evidence.txt)
connect the earlier ACK writer to the base handler's incoming metadata producer.
Reproduce with `target/debug/xtask apk-smali <apk> --class 'LX/0yO;'
--method BFC`, then `LX/CmG;`, `LX/CSu; --method A00` and
`LX/FpD; --method run`. These are offline development utilities, not network
commands or HTTP routes.

`0yO.BFC` first checks its accepted message-code array (000a..0020). It reads
the incoming stanza through `1B3.AtB` (0020..003a), obtains the
`loggableStanzaId` bundle long (003e..0052), and calls `CmG.A02` with that
stanza, null second argument, and the long (0052..005a). It constructs
`FpD` selector 9 and calls `run` (005c..0070). `FpD.run` selector 9 begins
at 0768: it reads the first child at index zero and its tag, records diagnostic
metadata by stanza ID, then directly invokes `0yO.A01` at 07e2..07e8 when
the handler's A09 executor is null. With an executor it schedules `Fpc` selector
3 (07f2..0804). **Unknown:** that alternate runnable's full body, actual message
code chosen by the upstream group parser, and current executor/feature state.
This extends the previously confirmed application-before-ACK trace without
claiming every upstream route is now recovered.

`CmG.A02` supplies class `notification`, null explicit from, null failure reason
and null default type to `CmG.A00` (0000..0016). `A00` therefore reads typed
`from` from the incoming stanza (0000..0014), required `id` (0014..0020),
typed `participant` (0020..0030), and `type` with the supplied null fallback
(0030..003c). These become CSu A02/A08/A01/A09 respectively; class becomes
A06 (0046..005a). CSu.A00 forwards these fields to Cb1's constructor
(0012..004a). A missing type is not manufactured as `w:gp2`.

**Confirmed/static distinction:** this notification wrapper does not read
`recipient`, `category` or `edit`. The separate `CmG.A03` receipt wrapper reads
those attributes, supplies class `receipt` and default type `delivery`, then
writes CSu A03/A05/A07. The common Cp2 ACK writer's ability to serialize these
fields does not prove that notifications echo them. Do not copy all root
attributes into a future group notification ACK. Exact Cb1 constructor search
found one caller, CSu.A00 (classes7.dex invoke offset 0x5b6174); CmG.A02 has
three callers, 0yO.BFC, 1RV.A06 and D34.B38. The latter two are not traced here.

**Confirmed/static normal control flow:** in `0yv.A09`, v6 is initialized to
zero at 11a6..11a8 before the fallback tag chain. A reverse normal-edge
predecessor traversal from the groups_dirty selector at 2912 visited 119
instructions and found that initialization as the sole reaching v6 definition.
The traversal uses instruction byte starts/ends, signed relative jumps in
code units (target = start + 2 * signed displacement), conditional fallthrough,
switch case/default targets, no fallthrough after return/throw/payload, and
stops a path at its first v6 destination write. It excludes exception edges;
exception-handler coverage is **unknown**, not a claim about all execution.

Each non-null group `jid` is paired with Integer.valueOf(v6) at 2976..2988;
a missing jid logs and skips (2966..2976). After iteration, 6548..6572 creates
an empty processed set, context 8lX.A05 (dirty-truncatable), null maximum rounds,
initial round v6=0, and params=3, then queues FetchTruncatedGroupsJob. The integer
in each pair is confirmed zero on this path; its semantic meaning remains
**unknown**, so it must not be described as a reason code. Static construction
also queues an empty list if all child JIDs are absent; subsequent job empty-list
behavior has not been established. No JID-validation requirement follows merely
from this raw string extraction.

**Negative search results:** 0zH.A05 is a message-class metadata producer,
not the notification wrapper. 12s.A01 constructs an outgoing location notification,
not the group receive parser. 0h8.A02 normalizes primary DeviceJid addresses in
existing Cb1 metadata; it is not the initial incoming stanza parser. These scoped
methods were inspected after exact CSu A06/A02 field-writer searches (31/34
write sites respectively). Other sites and the complete upstream dispatch remain
**unknown**. The needed runtime work is still durable group-refresh scheduling,
correlated batch response application and supported notification acknowledgement.
No account store was opened, request sent, identity changed or saved message
replayed in this research change.

Validation: all five OpenAPI checks, generated CLI reference, whitespace and
repository hygiene passed. No runtime behavior changed or live validation occurred.
