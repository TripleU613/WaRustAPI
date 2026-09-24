# Chat actions: block, report, disappearing messages, keep in chat, advanced chat privacy, clear chat, contacts

Static evidence from the comparison APK build **2.26.37.71** (disassembly of
`~/Downloads/WhatsApp.apk`, 2026-09-20). Live results on the saved
lab account, 2026-09-20: block (`list dhash matched=true` reply, contact listed
by LID on a full fetch), unblock (answered once `pn_jid` was dropped, full
fetch then empty), contact existence check (`usync` reply, on WhatsApp), and
the three E2E chat-setting messages (disappearing timer 86400 s, keep in chat,
limit sharing on) all server-accepted and delivered; the timer and privacy
messages also received read receipts. Handset display of the three settings
awaits operator confirmation. While the contact was blocked every send to them
was refused with ack error 403. Class names are for that
build only. Each claim is marked **confirmed** (read from the builder or
descriptor), **inferred** (consistent reading of adjacent code or the public
schema where the APK numbering agrees) or **unknown**. Nothing below was
live-verified unless a later section of `messaging-transport.md` says so.

Node-builder vocabulary used throughout (all confirmed): `LX/8kV.A0l()` = new
`iq` builder; `LX/8kX.A0T()/A0U()` = `type=get` / `type=set` attribute;
`LX/8kY.A1H(b)` = `to=s.whatsapp.net` (`LX/1S7.A00`); `LX/8ka.A0a(attr,b,id,..)`
adds `id` plus one attribute; `LX/8kU.A0m(name)` = new child builder;
`LX/8kV.A1Q(b,name,value)` string attribute, `A1M(jid,b,name)` JID attribute,
`LX/8kW.A1H(b,name,long)` numeric attribute; `LX/8kV.A1O(child,parent)` and
`LX/8kY.A0Z(child,parent)` append a child; `LX/0ho.A04` merges a `smax:any`
fragment (attributes and children) into a node. Names that are dictionary
tokens in `fixtures/tokens/web-md-v3.json` are noted where it matters; all of
the names below encode either as tokens or plain strings with the existing
encoder, so no encoder change is needed.

## 1. Block list (`xmlns=blocklist`)

Set request, `LX/1ep.A05` (`BlocklistV2SetProtocolHelper/sendSetBlocklistRequest`,
offsets `0x03ea..0x0652`), **confirmed**:

```
<iq to="s.whatsapp.net" id=… type="set" xmlns="blocklist">
  <item action="block"|"unblock" jid=<chat jid> [pn_jid=<phone jid>] [username=…] [dhash=<current list dhash>]/>
  [<entry_point source="<entry point>"/>]      (block only; source strings below)
  [<biz_opt_out first_message="none"|… reason=… reason_description=… entry_point=… business_discovery_*/>]  (business block with reason)
</iq>
```

- `jid` is the chat JID (the contact's LID when the chat is LID-addressed,
  otherwise the phone-number JID); `pn_jid` carries the phone-number JID when
  `jid` is a LID (`0x04c4`) **on the block branch only** (`0x04a0` jumps past
  `pn_jid`/`username` for unblock). Live 2026-09-20: an unblock item that
  carried `pn_jid` was silently dropped by the server (no reply at all); the
  block with `pn_jid` was answered with `list dhash=… matched=true`.
- `dhash` is the client's saved `block_list_v2_dhash` when non-empty
  (`0x0516..0x053e`, confirmed). Omit when no list has been fetched.
- `entry_point source` values are the 97 strings in `LX/9QF.<init>(VLI)` case
  5 (`account_info`, `block_dialog`, `chat`, `chat_list_block`,
  `overflow_menu_block`, `1_1_spam_banner_block`, …). For a CLI-driven block use
  `chat` (inferred choice; the attribute is telemetry).
- Reply handling: `LX/9xs.A00` (`BlockListResponseHandler/general_request_success`)
  updates the local list; `LX/1ep.A08` stores the new `dhash` from the reply
  (`block_list_v2_dhash`, confirmed).

Get request, `LX/1ep.A0M` (`BlocklistGetProtocolHelper/sendGetBlocklistRequest`),
**confirmed**:

```
<iq to="s.whatsapp.net" id=… type="get" xmlns="blocklist">
  <item dhash=<saved dhash>/>          (only when a dhash is saved)
</iq>
```

Reply parser `LX/8nc.A02` (`ProtocolTreeNodeParserUtil/parseBlocklistFromProtocolTreeNode`),
**confirmed**: the reply's `list` child carries `dhash`; each `item` child has
`jid` (fallback `pn_jid`), `action` (default `block`), `active` (default
`false`), optional `display_name` and `username`. The client replaces its list
with the `action=block` items and saves `dhash` and `block_list_receive_time`
(`LX/1ep.A0O`, confirmed). The account-sync dirty bit `blocklist`
(`LX/17V.A09`) triggers a fetch; `BlocklistInitialSyncJob` fetches at login.

Not the contact block: `LX/1ep.A05` also builds
`<iq xmlns="w:comms:chat" type="set"><blocking action="block"|"unblock"/></iq>`
for `SetChatPsaBlockingStatusHelper` (blocking the WhatsApp PSA chat) and
`<iq xmlns="w:comms:chat" type="get"><query><blocking_status/></query></iq>`
(`LX/1ep.A0M`, confirmed). `LX/EJx`/`LX/EJy` build the `w:interop` (third-party
chat) block list. Do not use either for a user block.

**Implementation**: `crates/whatsapp-api/src/transport/blocklist.rs` builds the
set/get requests above exactly as described (LID-vs-phone `jid`/`pn_jid`
selection via `super::inbox::lid_for`, saved-`dhash` inclusion, the
block-only `entry_point source="chat"`) and durably queues them through
`transport::actions::queue`, gated behind
`experimental-signal-crypto + experimental-noise + experimental-transport`.
`ingest` applies settled fetch/block/unblock action replies into a local
`blocked:<jid>` table (idempotent per action id) and `blocked` reads it back.
CLI commands `block`, `unblock` and `blocklist [--refresh]`
(`crates/whatsapp-cli/src/main.rs`, handlers in
`crates/whatsapp-cli/src/transport_receive.rs`) queue and inspect these
actions offline; `blocklist` masks contact numbers before printing. None of
this has been exercised against a live session: the request shape is
APK-derived (confirmed except where marked above), and the reply-side `dhash`
placement for a set reply is inferred, not traced.

## 2. Report (`xmlns=spam`)

`IndividualSpamReportRpc.A00` → `SpamRPCManager.A01` → `LX/9QZ.<init>(VLLLLLLLL)`
(`.registers 20`), **confirmed** shape:

```
<iq id=… type="set" to="s.whatsapp.net" xmlns="spam">
  <spam_list jid=<reported user jid> spam_flow="<entry point>" [is_known_chat="true"|"false"]>
    <message from=<sender jid> …>…</message>   ×0..210   (the reported messages)
    …                                            ×0..5    (second message list)
    …                                                    (third list)
  </spam_list>
</iq>
```

- `xmlns=spam`, `to=s.whatsapp.net` and `spam_list/@spam_flow` come from the
  `LX/9QF.<init>(VLI)` case-5 fragment merged into the request (`0x055a..0x0592`,
  confirmed); `is_known_chat` is case 6 of the same constructor (confirmed,
  optional).
- Message nodes are `LX/9QI` (`<message from=…>` or `to=…`) wrapped with the
  message content (`LX/BpT`: `smax:any` children `plaintext`, `enc type=msg|pkmsg`,
  `device-identity`; `LX/9QF(VLLL)`: `<plaintext mediatype=…>`). The exact
  attributes of a reported `message` were not traced: **unknown**. A report with
  zero messages satisfies the size bounds (`LX/0hp.A07(list, 0, 210)` and
  `(…, 0, 5)`, confirmed).
- Reason vocabulary `LX/A5E.A00`: `no_sign_up`, `no_longer_needed`,
  `no_longer_interested`, `spam`, `offensive_messages`, `otp_did_not_request`,
  `scam_or_fraud` (confirmed strings; where they attach is **unknown**, likely
  the `LX/9QG` fragment merged into `spam_list`).
- Reply: `IndividualReportResponseSuccess` carries a report id
  (`LX/9V2` `SuccessWithData(reportId=…)`), element name **unknown**; treat any
  `type=result` as accepted.

Observed 2026-09-20: a full block-list fetch reply item carried `jid` (LID),
`pn_jid` and one attribute our decoder rendered as `B3b=true`; the comparison
token dictionary maps double-byte page 0 index 1 to `B3b` where the
web-md-v3 fixture has `active`, so this is most likely the APK parser's
`active` attribute and a dictionary-recovery defect to re-check (tracked in
`docs/protocol/unknowns.md`).

Never send a report against the lab contact during testing. This request is
implemented from the shape above and must be labelled "not live-verified".

**Implementation**: `crates/whatsapp-api/src/transport/report.rs` builds the
`iq xmlns=spam` request above with zero reported-message children (their
attribute shape is unknown, per above) and durably queues it through
`transport::actions::queue`, gated the same as `blocklist.rs`. `spam_flow` is
validated against the entry-point vocabulary named in this section and §1
(`chat`, `overflow_menu_report`, `account_info_report`, `message_menu`,
`1_1_spam_banner_report`, `chat_list_swipe_delete_report_upsell`,
`block_dialog`), defaulting to `chat` when empty. The CLI `report` command
(`crates/whatsapp-cli/src/main.rs` / `transport_receive.rs`) queues it
offline. Nothing here has been sent against a live session; the shape is
APK-derived only, and the server's exact accept/refuse response is unknown.

## 3. Disappearing messages

### Per-chat timer: E2E `protocolMessage`

`LX/D9s.AD7` (`FMessageEphemeralSettingChangeProtobuf`), **confirmed**: the
message is `Message.protocolMessage` (field 12, already live-decoded for type 4)
with `ProtocolMessage.type = EPHEMERAL_SETTING (3)` (`LX/C4F.<clinit>`),
`ephemeralExpiration` (field 4, uint32 seconds; 0 = off),
optional `afterReadDuration` (field 29, uint32), optional `disappearingMode`
(field 11, `LX/BW7`: `initiator`, `trigger`, `initiatedByMe`). Field numbers
from the `LX/BZb` descriptor (`LX/BZb.dynamicMethod`): key=1, type=2,
ephemeralExpiration=4, ephemeralSettingTimestamp=5, historySyncNotification=6,
appStateSyncKeyShare=7, appStateSyncKeyRequest=8, disappearingMode=11,
invokerJid=19, limitSharing=24, afterReadDuration=29, acp2Setting=35
(confirmed where the parse aligned; 9,10,14,15,18 garbled but match the public
schema). Type enum (confirmed): REVOKE=0, EPHEMERAL_SETTING=3,
EPHEMERAL_SYNC_RESPONSE=4, HISTORY_SYNC_NOTIFICATION=5, APP_STATE_SYNC_KEY_SHARE=6,
APP_STATE_SYNC_KEY_REQUEST=7, MSG_FANOUT_BACKFILL_REQUEST=8,
INITIAL_SECURITY_NOTIFICATION_SETTING_SYNC=9, APP_STATE_FATAL_EXCEPTION_NOTIFICATION=10,
SHARE_PHONE_NUMBER=11, MESSAGE_EDIT=14, PEER_DATA_OPERATION_REQUEST_MESSAGE=16,
PEER_DATA_OPERATION_REQUEST_RESPONSE_MESSAGE=17, LIMIT_SHARING=27, ACP2_SETTING=39.

Stanza: `LX/BoR.A02` chooses the `type` attribute by message kind and falls
through to `text` for protocol messages (confirmed: only reaction, poll, pay,
event, medianotify, scheduled-call and media get other values). Send it as an
ordinary `message type=text` E2E payload. Whether the official client adds an
`expiration` or `ephemeral_setting_timestamp` attribute on this stanza is
**unknown** (`ephemeral_setting_timestamp` is not a dictionary token and appears
only as a database column). Timer values offered by the UI: 86400, 604800,
7776000 seconds and 0 (inferred from the observed 7776000 on live traffic).

Incoming: type 3 updates the chat's timer; type 4 (`EPHEMERAL_SYNC_RESPONSE`)
is the peer echoing its setting (already decoded in `signal/direct.rs`).

**Implementation**: `crates/whatsapp-api/src/transport/chat_settings.rs` encodes
and decodes `ProtocolMessage` field 12 (`disappearing_payload`,
`decode_protocol` returning `Setting::DisappearingTimer` /
`Setting::EphemeralSyncResponse`). `model::message::MessageContent::DisappearingTimer`
is the outgoing intent; `outbox::validate` and `transport::outgoing` accept it
and send it as a plain `message type=text` payload via `prepare_payload`, per
the "falls through to text" note above. `transport::inbox_events::project`
recognises field 12 and projects both type 3 and type 4 as
`MessageContent::DisappearingTimer` (the two carry the same
`ephemeralExpiration` field, so a sync echo of the peer's own setting is
treated the same as a direct change; see the type-4 note above). The CLI
command is `disappearing --chat <jid> --seconds <n>`; `disappearing-default
--seconds <n>` durably queues the `xmlns=disappearing_mode` IQ described below
via `chat_settings::queue_default_timer` and the shared action outbox
(`transport::actions`). Nothing here is live-verified yet.

### Default timer for new chats: `xmlns=disappearing_mode`

`ChangeDMSettingActivity.A0Z`, **confirmed**:
`<iq to="s.whatsapp.net" id=… type="set" xmlns="disappearing_mode"><disappearing_mode duration=<seconds>/></iq>`;
read-back `LX/0u0.A00` (`DisappearingModeManager/getDisappearingModeSetting`):
`<iq … type="get" xmlns="disappearing_mode"/>`; notifications carry
`<disappearing_mode duration=…>` (`LX/11O.A09`). The `usync` protocol child
`disappearing_mode` returns a contact's default (`LX/0u3.A0N`).

## 4. Keep in chat

`LX/Bai.AD7` (`FMessageKeepInChatProtobuf`), **confirmed**:
`Message.keepInChatMessage` = field **51** (`LX/BZc` descriptor; numbering
matches the live-confirmed 35/46/50/64), `KeepInChatMessage` (`LX/BV3`):
`key=1` (MessageKey: remoteJid=1, fromMe=2, id=3, participant=4), `keepType=2`
(enum `LX/C2L`: UNKNOWN=0, KEEP_FOR_ALL=1, UNDO_KEEP_FOR_ALL=2), `timestampMs=3`.
Stanza type falls through to `text` (`LX/BoR.A02`). The key follows the same
LID rule as reactions (remoteJid = the contact's LID when the chat is
LID-addressed; fromMe from the sender's perspective): inferred from the
reaction/poll behaviour confirmed live. Incoming keep messages should project
as a change on the target message.

**Implementation**: `crates/whatsapp-api/src/transport/chat_settings.rs`
encodes/decodes `KeepInChatMessage` field 51 (`keep_payload`, `decode_keep`,
reusing `reactions::Key`). `model::message::MessageContent::KeepInChat` (send
intent) and `MessageChange::KeepInChat` (incoming update) carry the target
message and a `keep` flag. `outbox::validate` checks the target is in the same
chat (`chat_settings::validate_keep`); `transport::outgoing` sends it as a
plain `message type=text` payload, choosing the remote identity the same way
as reactions (`inbox::lid_for`). `transport::inbox_events::project` recognises
field 51 and projects it as `SyncEvent::MessageUpdated` with the same
`target_from_sender`/direction attribution as reactions. The CLI command is
`keep --chat <jid> --id <wire-id> [--ours] [--undo]`. Nothing here is
live-verified yet.

## 5. Advanced chat privacy ("limit sharing")

Two generations exist in 37.71 (`Acp2*` classes, `UPGRADE_TO_ACP_V2`):

- v1, `LX/D9g.AD7` (`FMessageLimitSharingChangeProtobuf`), **confirmed**:
  `ProtocolMessage{type=LIMIT_SHARING(27), limitSharing(field 24) = LimitSharing
  (LX/BWR: sharingLimited=1 bool, trigger=2 enum, limitSharingSettingTimestamp=3
  int64, initiatedByMe=4 bool)}`; trigger enum `LX/C3L`: UNKNOWN=0,
  CHAT_SETTING=1, BIZ_SUPPORTS_FB_HOSTING=2, UNKNOWN_GROUP=3. The receiver
  drops it outside 1:1 chats (`D9g.CF4`, confirmed). Regular messages then carry
  `messageContextInfo.limitSharing` (field 13 of `LX/BZU`, inferred alignment).
- v2: `ProtocolMessage{type=ACP2_SETTING(39), acp2Setting(field 35) = LX/BWQ
  {enabled=1 bool, trigger=2 enum, settingTimestamp=3 int64, initiatedByMe=4 bool}}`
  with trigger enum `LX/C3j`: UNKNOWN=0, CHAT_SETTING=1, ACCOUNT_SETTING=2,
  BULK_CHANGE=3, BIZ_SUPPORTS_FB_HOSTING=4, UNKNOWN_GROUPS=5 (descriptor and
  enum confirmed; which generation the sender picks is **unknown**).

Implement v1 for sending (trigger CHAT_SETTING, initiatedByMe=true, timestamp
now) and decode both on receive. Sent as `message type=text`.

**Implementation**: `crates/whatsapp-api/src/transport/chat_settings.rs`
encodes v1 (`limit_sharing_payload`, `ProtocolMessage` field 12/`limitSharing`
field 24) and decodes both v1 and v2 (`decode_protocol` returning
`Setting::LimitSharing` / `Setting::Acp2`). `model::message::MessageContent::ChatPrivacy`
is the outgoing/incoming domain content (`limited: bool`, collapsing both
generations); `outbox::validate` and `transport::outgoing` accept it and send
it as a plain `message type=text` payload. `transport::inbox_events::project`
recognises field 12 and projects either generation as
`MessageContent::ChatPrivacy`. The CLI command is `chat-privacy --chat <jid>
(--on|--off)`. Nothing here is live-verified yet.

## 6. Clear chat and app-state sync

Clearing a chat is a local database operation plus an app-state mutation so
companions follow. Mutation payload (`SyncActionValue`, `LX/BZV`, numbering
aligned to the public schema where the parse agreed): timestamp=1,
starAction=2, contactAction=3, muteAction=4, pinAction=5, archiveChatAction=17,
deleteMessageForMeAction=18, keyExpiration=19, markChatAsReadAction=20,
**clearChatAction=21** (`LX/BQG`/`BQM`: messageRange=1), **deleteChatAction=22**,
`SyncActionMessageRange` (`LX/BVV`): lastMessageTimestamp=1,
lastSystemMessageTimestamp=2, messages=3. **contactAction=3** (`LX/BXZ`):
fullName=1, firstName=2, lidJid=3, saveOnPrimaryAddressbook=4, pnJid=5,
username=6. `SyncActionData` (`LX/BWk`): index=1 bytes (JSON array),
value=2, padding=3, version=4.

Patch write, `CSe.A00`/`CSe.A01` (**confirmed**, re-read 2026-09-22):
`<iq to="s.whatsapp.net" type="set" xmlns="w:sync:app:state" id=…><sync data_namespace=3><collection name=<collection> order=<n> [version=<v>] [return_snapshot="true"]><patch>{SyncdPatch bytes}</patch></collection></sync></iq>`

- `data_namespace` is a numeric attribute with the literal value **3**
  (`CSe.A00`). `CSe.A00` can also append `<bot><coex mode="sync"/></bot>` when
  one of its fields is set; that branch is not used here.
- **`order` is 1 for every collection except `critical_unblock_low`, which
  emits 0.** `CSe.A01` computes it as `"critical_unblock_low".equals(name)`
  followed by `xor-int/lit8 v3, v0, #0x1`, i.e. `!isCriticalUnblockLow`. An
  earlier revision of this document stated the opposite ("0 for most
  collections, 1 for `critical_unblock_low`"); that was wrong, and the
  implementation follows the corrected direction with a test pinning it.
- `version` is a numeric attribute emitted only when the version argument is
  non-null.
- `return_snapshot="true"` is emitted only when its boolean argument is true
  **and** the version argument is null or zero — so a patch send, which always
  claims a nonzero version, never carries it. It belongs to a fetch.
- `<patch>` is a raw-bytes child, added only when the patch argument is
  non-null.
Collections (`LX/0oP.A03`): regular=1, regular_low=2, regular_high=3,
critical_block=4, critical_unblock_low=5. Wire protobufs (descriptors
confirmed): SyncdPatch `LX/BYK` version=1, mutations=2, externalMutations=3,
snapshotMac=4, patchMac=5, keyId=6 (7–9 per public schema: exitCode,
deviceIndex, clientDebugData); SyncdMutation `LX/BTI` operation=1
(SET=0, REMOVE=1), record=2; SyncdRecord `LX/BVQ` index=1, value=2, keyId=3;
SyncdSnapshot `LX/BWT` version=1, records=2, mac=3, keyId=4; ExternalBlobReference
`LX/BXY`; AppStateSyncKey `LX/BSU` keyId=1, keyData=2; AppStateSyncKeyData
`LX/BUp` keyData=1, fingerprint=2, timestamp=3; Fingerprint `LX/BVm` rawId=1,
currentIndex=2, deviceIndexes=3.

Crypto (`LX/CKa`, confirmed): HKDF-SHA256 expand of the 32-byte app-state key
with info `"WhatsApp Mutation Keys"` to 160 bytes, split into index key
[0,32), value encryption key [32,64), value MAC key [64,96), snapshot MAC key
[96,128), patch MAC key [128,160). Value encryption AES-256-CBC with a 16-byte
IV (`KmpSyncdEncryptor.A00`, confirmed size); MACs HMAC-SHA512 (`LX/0sN.A01`,
inferred). LT-hash is implemented in DEX (`KmpLtHash16.A00`, 128-byte state,
HKDF info `"WhatsApp Patch Integrity"` from `LX/0rN`), so the public recipe
applies. Writing patches is a separate work item; until then "clear chat" is
local-only and documented as such.

### App-state patch writing: built, and what still blocks using it

Implemented and offline-verified 2026-09-22 (`signal/app_state_crypto.rs`,
`signal/app_state_collection/patch.rs`, `signal/app_state.rs`,
`transport/app_state_sync.rs`):

- **Encrypt primitives** mirroring each existing verifier input for input:
  `index_mac`, `snapshot_mac`, `patch_mac`, the truncated HMAC-SHA512
  `value_mac`, and `encrypt_mutation` producing the `IV || ciphertext ||
  value-MAC` body `decrypt_mutation` reads. `MutationData.padding` is written
  empty: the APK's padding content is **unknown**.
- **`CollectionState::build_patch`**, the inverse of `verify_patch` — same
  version rule, duplicate-index and missing-REMOVE refusals, and the same
  LT-hash accounting where a replaced index subtracts its old value MAC.
- **`app_state::mint_key`**, since a primary with no companions does mint its
  own key (`LX/0oK->A05`, the empty-device-collection branch stores and returns
  it) and minting is purely local with no stanza (**confirmed**). Key id is
  big-endian `u16` device index then `u32` epoch (**confirmed**; it decodes the
  repository's existing `app-state-key:000100000002` share fixture).
- **`transport::app_state_sync::patch_request`**, the IQ above.

Tests cross-check rather than self-check: a built patch decodes through
`CollectionPatch::decode`, verifies through `CollectionState::verify_patch`,
and the verifier's resulting state is asserted equal to the builder's
prediction; a minted key is selectable by the pre-existing `select_active_key`.

### Per-action routing and index (traced 2026-09-22)

The index is **not** built with `JSONArray` at the call site, which is why an
earlier search for one missed it: each value class assembles a Java `String[]`
in its constructor, `LX/Azp->A0B` returns it, and `LX/C8y->A00` renders it with
`JSONArray(String[]).toString()` — compact, no spaces. `LX/B6M` is the
ContactAction value class and `LX/B5H` the ClearChatAction one.

**Collections (confirmed).** `LX/Azs->A00(Azq)` is a packed switch on the action
ordinal into `LX/Azr` (`A06`=regular, `A08`=regular_low, `A07`=regular_high,
`A04`=critical_block, `A05`=critical_unblock_low):

| action | collection |
|---|---|
| **contactAction** (ordinal 1) | **`critical_unblock_low`** |
| **clearChatAction** (ordinal 16) | **`regular_high`** |
| star, mute, deleteMessageForMe, deleteChat | `regular_high` |
| pin_v1, archive, sentinel, markChatAsRead | `regular_low` |
| securityNotification, pushName | `critical_block` |

A contact landing in `critical_unblock_low` is counter-intuitive and is exactly
the kind of value that must be read rather than assumed.

**Index arrays.**
* **contactAction — confirmed, two elements**: `["contact", <Jid.getRawString()>]`
  (`LX/B6M.<init>` `007a..0098` reads `Azq.A0O.value` = `"contact"` and
  `Jid.getRawString()` into a two-element `String[]`). The field is typed
  `UserJid`, so it is user-level rather than device-addressed. **Whether the
  caller passes the phone JID or the LID is unknown** — `LX/BXZ` carries both
  `lidJid` and `pnJid`, and the enqueue site was not located.
* **clearChatAction — confirmed, four elements**:
  `["clearChat", <Jid.getRawString()>, "0"|"1", "0"|"1"]` (`LX/B5H.<init>`
  `003c..0068` via `LX/1xk->A1J`). The two booleans are constructor parameters
  stored as `B5H.A02` (first) and `B5H.A01` (second); **their semantics are
  unknown**, so clearChatAction is deliberately not implemented.

**`SyncActionData.version`**: **2** for contactAction (`LX/B6M.<init>`
`const/4 v9, #0x2`), **6** for clearChatAction (`LX/B5H.<init>`
`const/4 v13, #0x6`, forwarded through `LX/BeL` → `LX/BlG` → `Azp`). That
`Azp.A03` is the `SyncActionData` version is **strongly inferred**: `Azp`
exposes exactly the `SyncActionData` trio (`A07()` index string,
`A0A()` value bytes, `A03` version) and prints it as `version=` in its
`toString`; the method assembling the protobuf itself was not found.

Also: `LX/B5H.A05` sets `BQG.messageRange_` unconditionally, so a real
clear-chat **does** populate `messageRange` (**confirmed**), even though the
field is optional in the descriptor.

### Live result: the server refused the first patch

`whatsapp-cli contacts publish-app-state` builds a `contactAction` SET for every
saved contact into `critical_unblock_low` and sends it. Live 2026-09-22 the
server answered **`iq type=error` with `code=400 text=bad-request`** — a generic
refusal naming no field.

**One evidence-backed correction was then made and retried once.** Reading
`CSe.A01`'s call sites settled the version semantics (**confirmed**, both sites
independently):

* the `<collection version=…>` attribute is **the version the client currently
  holds**, not the version the patch claims. `LX/D44` looks it up by name in the
  `CollectionVersionsTable` (`LX/0rP->A02`, `SELECT … WHERE collection_name = ?`,
  returning the stored `version` column); `LX/DUo` likewise reads stored state.
  Neither derives it from the patch.
* `LX/0rP->A02` returns **null** when no row matches, and `CSe.A01`'s `if-eqz`
  on that argument then **omits the `version` attribute entirely**. So a
  never-synced collection sends no `version` at all — not `0`, and certainly not
  the patch's claimed `1`.
* The patch's own version lives inside the `SyncdPatch` protobuf and is a
  separate value from the attribute.

The implementation was corrected (`patch_request` takes `Option<u64>` and omits
the attribute when `None`) and **one** further live attempt was made. The server
refused it again with the same `code=400 text=bad-request`.

**Live patch attempts stopped there: two sends, one correction, both refused.**
Permuting the remaining candidates until one is accepted is the brute-forcing
this project forbids, and a patch accepted for the wrong reason is worse than a
refusal. What the refusals establish is that the server validates a patch rather
than storing it opaquely, and that the version attribute was not the only cause.

Still undistinguished, and none of them worth guessing:

* a collection may need an existing **snapshot** before it accepts a patch —
  this account's `return_snapshot` fetch returned an empty `<sync/>`, so it has
  none, and whether a first patch is even legal without one is **unknown**;
* **`SyncdPatch.deviceIndex`** is set where the `BYK` protobuf is constructed,
  which was not traced; we send this device's index;
* the **index JID form** may have to be the LID rather than the phone JID
  (`LX/BXZ` carries both `lidJid` and `pnJid`, and the enqueue site that chooses
  was never located);
* some other required `SyncdPatch` field may be absent.

### The construction site, read (2026-09-22, 2.26.37.71)

`LX/CRA->A00` is where a real client assembles the outgoing `SyncdPatch`. It
sets, in order: `mutations`, `keyId` (a `LX/BQ5` blob), `snapshotMac`,
`patchMac`, `deviceIndex`, `clientDebugData`, and — conditionally —
`externalMutations`.

**It never sets `version`.** `LX/BYK->version_` has exactly nine references in
the whole APK and every one is an `iget-object`; nothing writes it. A
server-sent patch carries a version, which is why our incoming parser requires
one, but an outgoing patch does not. **We were sending it on both refused
attempts.** The claimed version still feeds both MACs and the local state; it is
simply not on the wire. `build_patch` now omits it, and
`an_outgoing_patch_carries_no_version_field` asserts that on the encoded bytes.

Field numbers for `SyncdPatch`, from `LX/BYK`'s `newMessageInfo` descriptor
(**confirmed**), matching what we already decode: 1 `version`, 2 `mutations`,
3 `externalMutations`, 4 `snapshotMac`, 5 `patchMac`, 6 `keyId`, 7 `exitCode`,
8 `deviceIndex`, 9 `clientDebugData`.

`clientDebugData` is **traced but not implemented.** `CRA->A00` builds it only
when it holds all three of a current LT hash (`A0G`), a version (`A08`) and a
new LT hash (`A0M`); otherwise the whole block is skipped, so an absent field is
a shape the client itself produces. It is a nested `LX/BYb` whose field numbers,
from its own descriptor, are: 1 `currentLthash` (bytes), 2 `newLthash` (bytes),
3 `patchVersion` (eight-byte big-endian of the version), 4 `collectionName`
(UTF-8), 5 `firstFourBytesFromAHashOfSnapshotMacKey` (never set at this site),
6 `newLthashSubtract` (bytes, optional), 7 `numberAdd`, 8 `numberRemove`,
9 `numberOverride`, 10 `senderPlatform` (Android at this site), 11
`isSenderPrimary`. **Unknown:** whether `newLthash` and `newLthashSubtract` are
the post-patch hash plus a delta or the addition and subtraction terms —
`CRA`'s constructor leaves `currentLthash` null and later setters fill it, and
those were not read. Putting wrong integrity data in a field the server may
check is worse than leaving it out.

### Resolved: `SyncdPatch.version` was the cause after all (2026-09-24)

**An app-state patch has been accepted.** `iq type=result` with
`<sync><collection name="critical_unblock_low" version="1"/></sync>` — the server
now holds version 1 for that collection.

**The section below is wrong and is kept only because the mistake is worth
seeing.** It records the third attempt as proving the `version` correction was
not the cause. It proved nothing: that attempt ran a **stale release binary**
built before the fix. The saved request settles it — its patch bytes begin
`0a 02 08 01`, field 1, `version { 1 }`. The correction had never been on the
wire.

The lesson is narrow and expensive: *a live result only tests the code that was
actually running.* Rebuild before attributing a live outcome to a change, or the
experiment silently tests the previous build and its conclusion poisons
everything after it.

### The conflict protocol, observed

A **second** patch, still claiming no held version, came back as `iq
type=result` whose `<collection>` carried `type="error"` and
`<error code="409" text="conflict"/>`, followed by `<patches><patch>…` — the
patches we are behind on, so a client can catch up. Two things fall out:

* The server's own patch bytes begin `0a 02 08 01`, i.e. they **do** carry
  `version`. That is the direct confirmation that `SyncdPatch.version` is
  server→client only, which the APK trace had inferred from `LX/BYK->version_`
  having only reads.
* App-state answers **per collection**, so an `iq type=result` can carry a
  refusal. `actions::record_reply` was reading only the outer type and recorded
  that conflict as `Ok` — an error reported as success, which is worse than an
  error. It now looks for a collection-level `type="error"` and takes its code
  and text.

`CollectionState` is now persisted between patches (`to_saved`/`from_saved`),
stashed against the action that carried it and promoted only once the server
accepts *and* reports the version we claimed, so a refused patch cannot advance
the version we believe we hold. `queue_contact_patch` promotes first, then
builds on the held state and sends `version` when it holds one.

### The full loop, live

Implemented and verified end to end on 2026-09-24:

1. patch with no held version → **accepted**, server reports `version="1"`;
2. patch still claiming nothing → **`409 conflict`**, server returns the
   `<patches>` we are behind on;
3. those patches are verified and applied — same verification an inline patch
   from a fetch gets, and the key each names must be one this account already
   holds, because a conflict is not permission to trust bytes;
4. next patch claims `version="1"` → **accepted**, server reports `version="2"`.

`app_state::keys_for` was added for step 3: `select_active_key` answers "which
key should I *write* with" and applies an age and fingerprint policy to say so,
which is the wrong question for verifying a patch that names the key it was
written with. Applying the write-side policy there would refuse to read our own
history the moment a key aged out. It still only ever returns a key this account
already stored.

**Remaining:** only `critical_unblock_low` is handled; the other four
collections use the same machinery but have no queued actions yet.

### Third live attempt: the refusal survives the correction

The `version`-free patch was sent once, live, 2026-09-22. The server answered
the **same `code=400 text=bad-request`**. Three attempts now, two
evidence-backed corrections, all refused.

That is a useful negative result rather than a wasted send: emitting
`SyncdPatch.version` was a confirmed deviation from the APK and is now
eliminated as *the* cause. The correction stays — it makes our patch match the
builder — but it is not what the server objects to.

**Live patch attempts stop here again.** What remains is what was already
listed, minus the version question:

* the **first-patch/snapshot precondition** — this account's `return_snapshot`
  fetch returned an empty `<sync/>`, so it has no snapshot for any collection,
  and whether a first patch is legal without one is still **unknown**. This is
  now the leading candidate, because it is the one condition that no amount of
  getting the patch bytes right can satisfy;
* **`clientDebugData`** — traced above, deliberately not implemented, because
  its two LT-hash fields would have to be guessed;
* the **index JID form** — `LX/BXZ` carries both `lidJid` and `pnJid`, and this
  account is now known to be LID-addressed in at least its group traffic, which
  makes the question sharper than it was; the enqueue site that chooses has
  still not been located.

**Implementation.** `crates/whatsapp-api/src/transport/chat_clear.rs`
(`clear_chat`) and the CLI's `clear-chat --chat <jid>` implement only the local
half described above. It accepts a direct chat or a group (2026-09-22: groups
became clearable once group messages could be received; both key their
`received:` rows by the chat JID, so the same prefix scan serves either).
For a direct chat it walks `received:<sender>:<id>`
checkpoint rows (`transport/inbox.rs`), strips the authenticated `protobuf`
plaintext and marks each row `status: "cleared"` (kept, not deleted, so
redeliveries and receipt correlation still dedupe by id), deletes any
`poll-secret:<id>` rows whose recorded `chat` field (`transport/outgoing.rs`)
matches, and appends `sync::SyncEvent::ChatCleared`. Writing the
`clearChatAction` app-state patch so companion devices see the clear is
**deferred and not implemented**; nothing here has been live-verified against
a WhatsApp server.

## 7. Contacts

The primary keeps contacts in the device address book and syncs them with
`usync` (`LX/0u7.A06`, `UniSyncProtocolHelper`), **confirmed** vocabulary:
protocol children `contact`, `status`, `picture`, `business`, `devices`,
`disappearing_mode`, `username`, `lid`, `sidelist`; per-user
`<user><contact [type="delete"]>+E.164</contact></user>`; the reply's `contact`
child per user (`LX/0u7.C8f`) gives the WhatsApp status; results also carry
`lid`, `status`, `picture`, `business`, `devices`, `disappearing_mode`. Modes
and contexts are dictionary tokens (`query`, `delta`, `full`, `interactive`);
the existing devices query (`transport/devices.rs`) already sends
`usync mode=query context=interactive index=0 last=true` with a `sid`. A
contact existence check therefore reuses that query with `<contact/>` (and
optionally `<lid/>`, `<status/>`, `<disappearing_mode/>`) in `query` and
`<user><contact>+E.164</contact></user>` in `list`; the reply `user` carries
`jid` when the number is on WhatsApp and `<contact type="in"|"out">` (type
values inferred from the public protocol; `in`/`out` are tokens).

Companion sync of a saved contact is the `contactAction` mutation above
(`regular_high`? collection **unknown**; public clients use `regular_high` for
contacts, `regular` for chat actions).

**Implementation.** `crates/whatsapp-api/src/transport/contacts.rs` and the
CLI's `contacts list|add|edit|remove|check` implement two independent things.
A purely local address book (`contact:<digits>` rows) with no protocol traffic
at all: syncing a saved contact to companion devices via the `contactAction`
mutation above is **not implemented**. And a `usync` existence check: `contacts
check` queues the request shape described above (`<contact/>`, `<lid/>` and
`<disappearing_mode/>` in `query`; `<user><contact>+E.164</contact></user>` in
`list`) through the durable one-shot queue in `transport/actions.rs` (kind
`contact-check`); `contacts::ingest` parses a settled reply's `usync/list/user`
nodes into a `contact-status:<digits>` row (`on_whatsapp`, `jid`, an optional
`lid` parsed defensively since its shape is **unknown**) or, for a top-level
`iq type=error`, records the refusal against the checked number via an
internal `contact-check-req:<action id>` mapping (since action labels must
never carry a phone number). Nothing here has sent a live request to a
WhatsApp server; the node shape is offline-tested only.
