# Chat actions: block, report, disappearing messages, keep in chat, advanced chat privacy, clear chat, contacts

Static evidence from the comparison APK build **2.26.37.71** (disassembly of
`~/Downloads/WhatsApp.apk`, 2026-09-20). Class names are for that
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
  `jid` is a LID (`0x04c4`, inferred from the branch on the JID kind).
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

Never send a report against the lab contact during testing. This request is
implemented from the shape above and must be labelled "not live-verified".

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

Patch write, `CSe.A01` (**confirmed**):
`<iq to=… type="set" xmlns="w:sync:app:state" id=…><sync><collection name=<collection> order=<n> [version=<v>] [return_snapshot="true"]><patch>{SyncdPatch bytes}</patch></collection></sync></iq>`
(the `sync` wrapper and `data_namespace` attribute come from `CSe.A00`).
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
