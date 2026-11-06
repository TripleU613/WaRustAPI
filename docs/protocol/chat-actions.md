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

2026-10-11 follow-up: [legacy timers and own-device synchronization](own-device-timer.md)
corrects signed duration interpretation, confirms current BW7 device/initiated
fields 3/4 and adds supported own-wrapper events. Nonzero after-read duration
remains unsupported pending native feature/state policy. Earlier generic uint32
and inferred three-field mode descriptions below are superseded; historical
validation does not cover this follow-up.

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
(`transport::actions`). (Corrected 2026-10-13: the per-chat timer was
server-accepted, delivered and read on 2026-09-20, see the top of this file;
handset display is unconfirmed. The `disappearing_mode` default-timer IQ has
never been sent live.)

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
`keep --chat <jid> --id <wire-id> [--ours] [--undo]`. (Corrected 2026-10-13:
a keep was server-accepted and delivered on 2026-09-20, see the top of this
file; handset display is unconfirmed.)

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
(--on|--off)`. (Corrected 2026-10-13: limit-sharing on was server-accepted,
delivered and read on 2026-09-20, see the top of this file; handset display
is unconfirmed.)

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

**Remaining:** `critical_unblock_low` (contacts) and, since the push-name work,
`critical_block` (`pushNameSetting`, accepted live, see
[presence](presence.md)) are written; the other collections use the same
machinery but have no queued actions yet.

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

2026-10-11: local clearing also accepts direct LID chats and strips plaintext
from `received-own:<chat>:<id>` synchronized outgoing rows alongside ordinary
incoming rows. Source/route and cleared markers stay intact, so the
[own-device text receive path](own-device-text.md) cannot republish cleared
content on redelivery. Historical journal events remain historical; this is
not journal erasure or a companion app-state mutation.

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
values `in`, `out` and `invalid` are confirmed in APK 37.71 `0u7.C8f`
04f0..056c; their complete downstream state handling is not ported).

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
`lid` read from the APK-confirmed `val` attribute after successful
`usync/result/lid` protocol metadata) or, for a top-level
`iq type=error`, records the refusal against the checked number via an
internal `contact-check-req:<action id>` mapping (since action labels must
never carry a phone number). The LID correction below was validated offline;
no fresh contact request was sent for that correction.


### Contact LID response correction (2026-10-08)

**Confirmed, static APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
[`0u7.C8f`](apk/2.26.37.71-contact-lid/contact-success-parser.txt)
0062..0070 selects `usync/result`; 0276..0298 requests the LID protocol result
only when `9uE.A09` selected that protocol. At 0e0c..0e42 it checks that result's
`A3p.A05`, then reads the optional per-user `lid` child **attribute `val`** as
plain LID class `0ZY`. The prior Rust `lid@jid` fixture and parser were incorrect;
this attribute is not a compatibility fallback established by this trace.

[`0u7.A01`](apk/2.26.37.71-contact-lid/contact-protocol-result.txt)
parses non-contact protocol results: an `error` child produces false (0166),
otherwise optional `refresh` is parsed with `Long.parseLong` and success is true
(01b4..01e6). [`A3p.<init>`](apk/2.26.37.71-contact-lid/contact-result-flag.txt)
stores that constructor boolean in `A05`. Thus an IQ result alone does not
establish LID protocol success. Missing result metadata does not authorize using
an otherwise plausible per-user LID. Malformed `refresh` cannot be interpreted
as a successful result; refresh scheduling/backoff application is not implemented.

[`0hs.A0A`](apk/2.26.37.71-contact-lid/contact-typed-attribute.txt)
uses an existing decoded JID or the text factory, checks/logs compliance, casts to
the requested class and returns null on `ClassCastException`. The existing
[plain-address evidence](mex-participant-address-evidence.txt) establishes
`0D1`/`0ZY` validation: ASCII 1..15 digits without leading zero, no device suffix,
and `hosted.lid` canonicalizes to `lid`. The Rust parser reuses this validation;
it does not normalize device addresses into accounts. Text is validated before
the generic JID parser can discard an explicitly zero device/agent suffix.
The initial implementation rejected decoded `AgentDevice` values even with zero
components. That local encoding/subtype assumption was **refuted by the live reply
below**; zero-component primary wire LIDs are now accepted, while textual suffixes
and nonzero components remain rejected.

**Implemented/offline-tested:** `contacts::ingest` stores this optional LID only
with successful protocol metadata. `contacts check` already exposes the saved
status; no new command or HTTP handler is introduced. Fixtures cover typed/text
attributes, hosted normalization, wrong attribute/subtype, explicit zero suffix,
invalid digits/length, absent/error metadata, malformed/overflow refresh and
once-only durable ingestion. Missing/rejected LIDs remain null, not proof that
an account has no LID. Existing ingested markers/statuses are preserved; no
historical reprocessing, fresh query, key change or identity mapping is performed.
Contact metadata is not authority to replace a Signal pin or authenticated alias.

**Research boundary:** a case-sensitive `Usync`/`usync` string search selected
111 strings from 453,954 before tracing `0u7`; the selected methods, rather than
the bulk class/string export, are retained in the bundle. The LID branch is
not a source of mandatory registration fields or Signal identity authority.

**Unknown:** fresh live contact replies for this correction, full contact
`in`/`out`/`invalid` state-machine parity, and refresh/backoff application.
The existence boolean still follows the retained per-user `jid` presence rule.
Do not infer account ban status from absent LID metadata. Evidence and reusable
hash-pinned recipe are in the [contact LID bundle](apk/2.26.37.71-contact-lid/manifest.json)
and [recipe](apk/recipes/2.26.37.71-contact-lid.json); raw APK/private state stay local.


Validation: 710 workspace/all-feature tests, strict workspace/all-target/all-feature
Clippy, warning-free Rustdoc, 24 default CLI checks, regenerated CLI reference,
API library-symbol check, formatting and repository hygiene passed. Exact APK
bundle re-export verified all nine files (94,899 bytes). No saved private account store was
opened, no live request sent, and no historical action reset by this chunk.


### Contact type and offline status (2026-10-08)

**Confirmed static APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
`0u7.C8f` 0484..049c selects contact versus sidelist; 04f0..0570 requires exact
`type`, maps `in`/`out`/`invalid` to `N0z.A04` values 1/2/3, and throws `B7l`
for unsupported values. It appends non-null contact text to `A0O`. With an empty
selected child list it assigns 1 at 0588..0590; this is **not** proof that a
missing contact child is a valid existence reply in our single-number subset.
`N0z.<init>` starts this field at zero, distinguishing unavailable metadata.

[`0u3.A02`](apk/2.26.37.71-contact-lid/contact-account-flag.txt)
0100..0112 skips unavailable/zero results and sets local `v7` true **only for
value 1**. At 0232..0236 that flag becomes `0DV.A0A`, independently of whether
the returned JID is non-null. It separately processes mapping/contact metadata;
these writes are not ported by the existence parser. Its missing-response and
found-invalid-contact diagnostics remain distinct. In
[`0u5.BF6`](apk/2.26.37.71-contact-lid/contact-invalid-consumer.txt)
05e2..0616, value 3 collects invalid contact strings separately, while 1 and 2
enter ordinary processing. This refutes the Rust rule that any supplied JID
establishes a positive existence result. Invalid does not establish a ban.

**Implemented/offline-tested:** `ContactStatus.contact_type` preserves the exact
recognized value. Newly ingested `on_whatsapp` is true only for `in`; `out` and
`invalid` remain distinguishable. Missing/unsupported type saves null plus fixed
local `missing_contact_type`/`unsupported_contact_type`, never the raw unknown
attribute. These are **local subset diagnostics**, not server reason strings:
Android instead raises a corrupt-stream failure for unsupported types. Other
supported per-user rows may still be ingested by our defensive parser. Existing
rows default `contact_type` to null and retain their old boolean; consumed markers
and identities are preserved. No historical reprocessing or server request occurs.

`contacts status --phone <private-number>` now inspects the saved row **without
queuing**. `--json` exposes its type and optional private address models; text
prints the masked phone/type/error without address bodies. This closes an
inspection gap: `contacts check` always queues another action before displaying
saved status and should not be used merely to inspect a prior result. The new
status command does not ingest, reserve, send, log in or migrate anything.
The API reference marks its operation CLI-only/offline, with no HTTP handler.

The full-APK [field-reference scan](contact-status-field-evidence.txt) found 38
reads/writes of `N0z.A04` (including the constructor); the consumer trace above
is scoped to `0u3.A02` and `0u5.BF6`, not all callers. The expanded seven-topic
[recipe](apk/recipes/2.26.37.71-contact-lid.json) preserves both consumers and
the default. Full sidelist/address-book mutation, contact protocol integrity,
refresh/backoff application and fresh live contact-type replies remain
**unknown/unported**. Do not promote offline evidence into live parity.


Same-APK coverage comparison against `6488d2f` completed in **0.008132 seconds**:
three added topics, four unchanged, with unchanged APK inventory/manifest/tokens.
The [cached diff](apk/contact-type-coverage-diff.json) describes coverage growth,
not a new APK release. The seven-topic bundle is 12 files / 132,816 bytes.


Validation: 713 workspace/all-feature tests, strict workspace/all-target/all-feature
Clippy, warning-free Rustdoc, 24 default CLI checks, regenerated CLI reference,
API library-symbol verification, formatting, CLI build/help and repository hygiene
passed. Exact re-export verified all 12 bundle files. Tests cover JID/type
contradictions, exact case, fixed redacted errors, historical null compatibility,
invalid result persistence across reopening and read-only status without action
changes. No saved private account was opened or live query sent in this chunk.


### Live contact projection and primary wire LID (2026-10-08)

**Confirmed local implementation:** the receive actor previously settled contact
IQ replies but never called `contacts::ingest`; only the CLI's new-query command
did so. `connection::drive` now ingests after durable reply settlement and at
session startup. Settlement and projection are separate commits: retained replies
and consumed markers repair an interrupted gap without another query or deleting
markers. Real-loop tests cover automatic status projection and a pre-settled,
unprojected reply recovered at the next session. `contacts status` stays read-only.

**Observed live**, ingestion UTC `2026-10-09T03:16:05.288000+00:00`: one original existing-peer
contact-check action received an IQ result with contact type `in` and matching
selected phone. A 20-second receive-only session authenticated/activated and
stopped cleanly, with outbox sending and reconnect disabled. The 19 prior actions
were unchanged, zero queued actions remained, two older sent/uncertain actions
were preserved, and all nine acknowledged receipts plus the empty outbox were
unchanged. No chat, SMS/registration attempt or repeated contact query was sent.

The response **did contain** successful `usync/result/lid` metadata and per-user
`lid@val`. Our first projected LID was null because `binary::Decoder::read_ad_jid`
assigns `JidKind::AgentDevice` to the zero-agent/zero-device primary LID. That
kind records **wire encoding**, not a Java `DeviceJid` subtype. Rejecting it by
kind alone discarded the valid captured account metadata. The corrected parser
accepts Plain/AgentDevice encodings with both components zero and the existing
strict plain-LID user/domain validation, storing a canonical plain LID. It still
rejects nonzero components, FB/interop encoding kinds and explicit textual
`:0`/`.0` suffixes before generic text parsing loses them. A literal synthetic
AD_JID-zero attribute round trip covers ordinary ingestion.

**Confirmed APK/static boundary:** `0hs.A0A`'s `0ZY` class cast remains the source
for logical LID type, while `0D1/0ZY` establish plain validation. A scoped
[factory/caller trace](contact-primary-wire-evidence.txt) records six `0DK`
methods, eight Java constructor callers of `0ZY` and two callers of `0D2.A02`.
`0DK.A00` can wrap UserJid as a primary DeviceJid; it is not the wire decoder.
**Unknown:** native AD_JID-to-Java object construction and exact native class
selection. The zero-component acceptance is grounded in the **observed reply
and our decoder representation**, not a claim that Android's native factory
has been fully reversed or that every encoded device is a plain account.

**Implemented local migration, not APK behavior:**
`contacts repair-lid --id <existing-action-id>` fills a missing LID only from a
sole successful contact check with saved original phone/reply, consumed marker,
one qualifying matching user and matching retained status fields. The action
history is inspected in the same write transaction, so a new same-number check
cannot race attribution. Another same-number check (even queued), unknown check
provenance, nonqualifying/duplicate/mismatched reply, conflicting existing LID,
or an incomplete prior repair refuses with a fixed local reason. It archives
the prior status and updates only `lid`, preserving `checked_at`. An already
matching LID returns false. It deletes no markers, sends nothing and changes
no authenticated alias, Signal pin, session or account identity.

**Observed offline on the live saved reply:** first repair returned true and
recovered the LID; repeat returned false. Actions, nine receipts, empty outbox
and status timestamp were unchanged. No query was repeated to repair this result.
The previous null is a resolved local parsing failure, not server LID absence.
This verifies the saved live response format and offline repair, not a fresh
post-fix network response or full cross-device mapping authority.

Operator corrections/negative results: the first receive launch used scalar
`transport.reconnect=0`, rejected before connecting because this setting is an
object. The original action remained queued; only
`transport.reconnect.max_attempts=0` was corrected in a private config copy.
The same action then ran once. Concurrent default-feature tests replaced the
shared CLI executable, causing post-session inspection commands to be unrecognized.
An isolated rebuilt experimental binary recovered all results offline; no session
or query was repeated. Future live sessions/inspection should use a private stable
copy of the experimental binary when feature-changing builds run concurrently.
Raw reply, IDs, addresses, config, keys and attempt journal remain ignored/private.

**Still unverified:** fresh post-fix replies, live `out`/`invalid` handling, native
wire factory parity, full contact integrity/sidelist/mutation and refresh/backoff
application. Contact metadata is not peer identity-replacement authority.


Validation: **716 workspace/all-feature tests**, strict workspace/all-target/all-feature
Clippy, warning-free Rustdoc, 24 default CLI checks, regenerated CLI reference,
API library-symbol checks and formatting passed. Receive-loop regressions verify
automatic projection and interrupted-projection recovery; migration regressions
verify primary AD_JID round trips, prior-status retention, idempotence, timestamp
preservation, conflicting-LID and ambiguous-history refusal, and redacted unknown-ID
errors. The first broad run found the new CLI reference stale; regeneration fixed
that check before the successful final run. Local repair errors now name the
failed prerequisite instead of a generic refusal. Repository hygiene passed.

Group keep/unkeep receive attribution (2026-10-09, **confirmed static/local**):
APK 37.71 D9L.A01 maps BV3.key through BoQ, just like ordinary group reactions.
Reception now resolves the original author independently of the keeper, uses
authenticated own-envelope context and retains matching parent devices. Incoming
updates preserve consumed bodies, outbox facts and existing event markers.
`lab-inbox --inspect` exposes fixed group keep target_validation labels using
original saved routes. Direct keep projection retains its existing behavior.
Normal group keep sending, native downstream authorization/expiry enforcement,
and fresh live group keep/handset rendering remain unimplemented or unverified.
See [group keep](group-keep.md).

Normal group keep/unkeep now has a separate original-author codec and typed
`keep --participant` / `--ours` outbox path; `--dry-run` checks references without
enqueue. Parent selection is revalidated in the reservation transaction.
[Evidence and remaining keep-policy/clock/expiry gaps](group-keep-outbox.md).
This supersedes the group send limitation above; fresh group ACK/rendering is
still unverified.

## Block-list read/change callback timing corrected — 2026-11-05

**Confirmed static / implemented offline:** current APK1dH.A05 block/unblock
uses20000ms through0Sf.A0W/A0A;1dH.A0M fetch passes0 throughA0Q/A06,
which supplies no positive-duration replacement. The normal Rust loop now
expires block/unblock live callbacks20s from the pre-write boundary, retaining
Sent uncertainty without list/hash mutation, rollback or retry. Fetch receives
no invented20/32s timeout. Reply cancellation and expiry remain competing claims.
[Complete producer, failure callback, tests and limits](apk/2.26.43.72-blocklist-callback-timing/README.md).
Paused-clock tests cover19/20s, delayed write, result/error cancellation and
fetch reply at60s. Normal-driver delayed-write/late-reply tests now cover both
changes and initial AB-properties, continued receive and no resend/projection.
The native change failure callback reaches2KT.CE5(false); its concrete downstream
side effects remain open. Full native scheduler parity and other producers'
deadlines remain unfinished. No live account operation.

## Account-default disappearing timer callback expiry — 2026-11-05

**Confirmed static / implemented offline:** current APK ChangeDMSettingActivity.A0c
set and0oy.A00 get both pass20000ms to0Sf.A0W. Normal live action ownership
now assigns20s to these account-default shapes from the pre-write boundary.
Expiry preserves durable Sent uncertainty without replay, applying the requested
value or local rollback. Result/error cancels that timer. Per-chat E2E timer
messages and usync disappearing_mode children do not inherit it.
[Producer, native success/failure distinction, tests and limits](apk/2.26.43.72-default-timer-callback/README.md).
Paused-clock tests cover19/20s, delayed write, result/error and a separate
AB-properties callback surviving until32s. The normal-driver expiry test also
covers default-timer write delay, late reply, receive ACK and no resend.
Native timeout publishes the saved duration through observers; successful
callback has a distinct preference-write path. Complete Rust default-setting
adoption, alternate-mode gates, native error/observer behavior and exact
concurrent scheduling remain open. No live account operation.

## About reply semantics and UI/network lifetime distinguished — 2026-11-05

**Confirmed static / implemented offline:** current APK FxR.A04 sends writer29,
which storesJAj in a legacy map shared by1rj/1rh/1rD. SetAboutInfo.A5f's32s
UI wait does not itself remove the callback in the selected handler. No32s
network timer is invented for About. Selected reader teardown invokes failure
callbacks then clears this map; full lifecycle/concurrency policy remains open.
Native outer result removes the callback andJAj.A02 ignores the payload, so a
synthetic nested error extension must not turn typed About success into refusal.
Rust now preserves this producer-specific meaning; outer error/code and other
producers' nested refusals remain intact. [Full alias/dispatch evidence and regression limits](apk/2.26.43.72-about-callback-lifetime/README.md).
Paused-clock regression covers empty/nonempty About at60s, independent32s
AB-properties expiry, outer result/error and duplicate claims. Normal-driver
regression verifiesOk/no refusal code, continued receipt ACK and no resend.
Saved About adoption, UI observers and full native lifecycle remain unfinished;
no live account operation or observed server extension response is claimed.

## Profile-photo callback construction and upload result — 2026-11-05

**Confirmed static / implemented offline:** current Hyd.A00's namespace helper
also returns the to=s.whatsapp.net attribute, which Hyd appends; the existing
Rust address is confirmed rather than inferred. 6p1 lookup builds that address
explicitly. Both selected producers pass duration0 through distinctA0W/A0R
wrappers, so no generic20/32s callback timer is assigned. Upload success requires
the first child picture and ignores later error extensions; Rust now preserves
this typed success shape instead of reporting a synthetic nested refusal.
Outer errors and other action refusal parsers remain intact.
[Producer/helpers, separate lookup parser, tests and limits](apk/2.26.43.72-profile-photo-callbacks/README.md).
Tests cover exact4/5 attribute counts, server address, same-connection claims at60s,
outer result/error, duplicate/mislabeled requests and normal receive continuation.
Lookup result adoption, full malformed-result admission, cache/removal/account-center,
target normalization and image transforms remain unfinished. No live traffic or
observed server extension response is claimed.

## Two-step verification callback and diagnostic privacy — 2026-11-05

**Confirmed static / implemented offline:** current1HL.A07 passes32000ms to
0Sf.A0W; FUS.CK4 schedules original supplied values without reading response
payload. Normal two-factor-set callbacks now expire32s from pre-write start,
retaining Sent uncertainty without retry or credential adoption. Typed outer
result ignores nested error extensions; outer error/code remains separate.
[Producer, timeout/success paths, tests and limits](apk/2.26.43.72-two-factor-callback/README.md).
TwoFactorChange's derived Debug exposed PIN/email; custom Debug now preserves
field presence and redacts values without changing inputs or request bytes.
Paused-clock tests cover31/32s, delayed write, PIN/disable/email-only changes,
result/error/duplicate and unchanged durable rows after expiry. Normal-driver
late-result/receive/no-resend test includes this producer. Native credential
adoption, encryption, email-status job and error/observer behavior remain open;
no live account operation or server extension response is claimed.

## Contact-check preparation and reservation atomicity —2026-11-05

Confirmed local fault: separate action/correlation commits could leave a sendable
orphan on disk. queue_check now prepares both atomically. Real full-disk and
uncertain-fsync tests verify neither/both consistency after reopen. Existing
Queued requests lacking correlation defer with contact_check_correlation_missing;
invalid/mismatched correlation refuses locally, and unrelated eligible traffic
drain continues. No automatic correlation repair, Sent reset or replay.
[Fault evidence, native caller-dependent usync timers and tests](apk/2.26.43.72-contact-queue-atomicity/README.md).
Current0pD.A07 receives a caller timeout: selected HxV lightweight call is32s,
while another0p7 call is128s. Rust contact-check's exact factory/deadline remains
unported; no universal usync timer is inferred. No live traffic was sent.

## Contact protocol metadata/backoff obligation —2026-11-05

Confirmed current0pD.CK4/A01 retains contact protocol success/integrity/error,
refresh and backoff separately from per-user in/out/invalid. Error backoff uses
seconds*1000 capped at3600000ms, with a missing-attribute default before the cap.
Selected observers persist positive deadlines using08q.A00;503 also writes
contact-executor global_backoff_time, while429 has an additional guarded scope.
Regular and lightweight query observers write different per-scope preferences.
[Bindings, full selected methods, strict deadline gate and remaining port requirements](apk/2.26.43.72-contact-protocol-backoff/README.md).
Rust still lacks full metadata/wait adoption. Outer IQ success alone is not contact
protocol success. No new wait, retry or account-state mutation is implemented here.

## Single-contact projection bound to original request —2026-11-05

A reproduced local defect let a successful contact-check reply overwrite status
for any returned phone. Projection now reloads/validates saved request provenance
and applies only a uniquely matching requested number. Unrelated/duplicate/
missing-origin/malformed cases preserve contact data and record a content-free
ActionOutcome.contact_projection through existing actions inspection. Status,
projection diagnostic and once-only marker commit atomically; settled wire/reply/
sequence and historical consumed markers stay unchanged. No query is replayed.
[Red regression, disk-fault/reopen tests, inspection and native-policy limits](contact-projection-correlation.md).
Current native protocol integrity/backoff and exact factory/deadline remain open.

## Contact factory flag comparison —2026-11-05

Current HxV lightweight A0B leavesHck.A0F false; H6w maps it toA0J and0pD
uses that flag to emit disappearing_mode. The regular A0C/common factory sets
it true alongside additional picture/business/username flags. Rust's fixed
contact/lid/disappearing_mode request is therefore not exact lightweight parity.
Both selected native phone-query dispatch paths are32s, but other usync callers
use128s; equal visible mode/context does not bind a complete factory policy.
[Full constructors, field transfer, writer and caller comparison](apk/2.26.43.72-contact-factory-selection/README.md).
Factory/configuration/observer/backoff integration remains open. No request or
callback policy changed and no live account operation occurred.

## Usync outer-refusal numeric metadata —2026-11-05

Confirmed current0pD.Bvj uses absent code0/backoff-1, unlike protocol-result
A01's absent-backoff default before its cap. Present values use signed JVM
parse and wrapping seconds*1000 capped at3600000ms. Durable usync get-action
outer errors now save optional ActionOutcome.usync_refusal with presence flags
and fixed numeric parse failures, exposed through existing actions plain/JSON.
Metadata survives projection/reopen; duplicate replies do not rewrite it.
[Full callback, independent JVM vectors, tests and scope limits](apk/2.26.43.72-usync-outer-refusal/README.md).
This does not enforce a wait, select an observer/deadline or replay historical
callbacks. Factory/read/reset/clock adoption remains unfinished; no live traffic.


## Contact backoff gates and reset callers — 2026-11-05

**Confirmed static**, APK2.26.43.72: the selected contact executor filters
protocols individually; its FULL contact/sidelist wait-key selection depends on
property37559. Exact0pQ.A02 direct callers are registration contact-sync start
and IBi selector16 scheduled by a contact `sync` notification. This event-bound
reset does not clear every wait key and is not generic retry permission.
[Evidence, delay arithmetic, branch scope and remaining port requirements](apk/2.26.43.72-contact-backoff-gates/README.md).
Rust eligibility/reset behavior is unchanged; query reachability, clock authority,
authenticated notification/reset adoption and effective configuration remain open.
No live account operations or server acceptance claims.


## Contact protocol refusal inspection — 2026-11-05

**Confirmed static and locally tested**, APK2.26.43.72: an outer usync IQ result
can carry a separate contact-protocol refusal. Actions now durably expose
`contact_protocol_refusal` with native backoff-first parsing, absentcode-1 and
absentbackoff3600000ms. Outer-IQ refusal defaults remain separate. Numeric
parse failures are fixed/redacted diagnostics, not guessed waits. Existing
plain/JSON actions inspection retains the field across projection and reopen;
duplicates cannot settle again. Legacy absent fields remain unknown.
[Source readers, JVM oracle, tests and unported adoption/wait policy](apk/2.26.43.72-contact-protocol-refusal/README.md).
Outer action status/per-user projection remain separate; no automatic retry,
wait enforcement, live operations or complete native integrity-policy claim.

## LID repair retains original request authority (2026-11-06)

**Confirmed local integrity correction:** repair_lid previously decoded its reply before opening the update transaction and checked saved phone/history without revalidating the encoded producer request. An altered request target could therefore coexist with a previously qualifying reply/status and still permit repair. It also used first-child usync/list selection rather than ordinary projection's ambiguity checks.

Repair now uses actions::view_in_write to read current action, request and reply in the same transaction as history/status writes. It requires contact-check kind, Ok status, the existing canonical single-phone request correlation validator, and an IQ result with matching ID. It reuses select_contact_reply to require unique usync/list and exactly one qualifying original-number user. Existing consumed-marker, history, protocol success, matching status, conflict and once-only rules remain. Invalid state refuses before any write; no unknown write is replayed.

Regression lid_repair_revalidates_saved_request_and_reply_without_mutation_after_reopen exercises nine saved-state mutations: request target/ID, reply ID/type/root, duplicate usync/list, kind and status. Every refusal preserves the complete checkpoint snapshot before and after reopen and emits no tested private identifiers. The prior successful AD_JID-zero repair test still verifies one repair, repeat false, history, timestamp and conflict controls. This is offline contact metadata integrity, not a new native wire requirement or authenticated alias/adoption capability. No account operation, request, retry or deadline change. The preceding APK missing-LID route research remains incomplete at final wire/profile and mapping authority; those gaps do not justify widening this local repair.

## Unmatched blocklist set results (2026-11-06)

**Confirmed current pinned mismatch / correction:** [complete set-success,
reconciliation worker/constructor, shared set sources and REA Evidence](apk/2.26.43.72-blocklist-set-reconciliation/README.md)
show8R7.CK4 routes list matched=false through migration admission and hash/set
reconciliation. It does not unconditionally apply the requested block/unblock.
Rust had applied every settled result to its immutable requested target and
adopted the reply hash, potentially fabricating a blocked/unblocked cache state.

The projector now preserves rows/hash for literal matched=false and atomically
records reconciliation_required=true plus its once-only inspection marker. It
returns zero applied changes for that action, while the generic action outcome
stays Ok for the received IQ result. Business projection, server result and
network/refusal status remain distinct. No old request is reset or resent.
CLI blocklist JSON adds reconciliation_required; text warns that the cached
list needs reconciliation. Existing --refresh --full can explicitly queue a
full fetch. The read-only library helper exposes the same Boolean.

A later admitted full fetch snapshot replaces rows/hash and clears the warning
in the same transaction. No-list/unchanged fetches, merely queuing a fetch or
forgetting the hash cannot clear it. Inspection markers prevent an old unmatched
set result being projected after that repair. A cut before commit leaves both
warning and marker absent. Stored source action/reply remain inspectable.

Regression unmatched_set_results_never_apply_the_requested_change_or_hash_after_reopen
failed before the guard (projector returned1 instead of0). It covers block and
unblock with equal/different/missing c_dhash, unchanged cache/hash, transaction
abort and journal preservation, once-only inspection, reopen, no-list warning
retention, later snapshot repair, and no resend. All actions and replies are
synthetic local fixtures; no real block, account fetch or network operation.

**Limits:** full native conditional snapshot/migration/alias reconciliation and
automatic fetch policy remain unported. Already inspected historical bad
projections are not rewound; a fresh admitted full fetch is needed to correct
such a cache. Historical root-hash/no-list set compatibility is retained; full
current first-list admission is still a separate gap. The shared native A0U
source has multiple writers/alias paths, so no new played-receipt block guard
is inferred from an unproven globally equivalent Rust cache. Source census
and positive set-success/replacement paths are retained, not called optimistic
pre-send state without evidence. Zero bans/live correctness remain unproven.

Validation:1,516 workspace tests pass (9 result suites, four test threads);
strict workspace/all-target/all-feature Clippy, formatting, CLI reference
regeneration, OpenAPI library-symbol coverage, APK coverage freshness and
repository hygiene pass. Thirteen selected DEX methods/censuses reproduce
byte-for-byte; APK subject hash, complete REA source and bundle checksums match.

## Blocklist snapshot child admission (2026-11-06)

**Confirmed pinned mismatch / correction:** [complete parser, requireTag,
constructor/unboxing checks and REA limitations](apk/2.26.43.72-blocklist-snapshot-children/README.md)
show6qD.A02 requires every child to be item. Rust's description filter silently
ignored unknown children, which could project a false empty/partial snapshot,
advance its hash, mark it applied and clear the reconciliation warning.

The projector now returns a fixed redacted InvalidInput(blocklist) error before
any cache/hash/marker/reconciliation writes can commit. The new regression
failed before the fix (malformed snapshot returned1) and covers unknown-only,
first/last unknown, nested-list, valid-empty and valid-item cases. Invalid input
preserves all checkpoint bytes and journal across reopen; valid snapshots still
replace state. No pending request is recreated and no live operation occurs.

Malformed saved snapshots remain unprojected and ingest continues to refuse
rather than quarantine/skip them. At this checkpoint native duplicates used map put while this
adapter refused duplicates; the later repeated-identity correction below
supersedes that refusal. Full typed fallback, conditional metadata/alias,
migration and previously projected bad-cache repair are separate gaps. No
complete native blocklist/played-policy equivalence or ban-rate claim follows.

Validation:1,517 workspace tests pass (9 result suites, four test threads);
strict workspace/all-target/all-feature Clippy, formatting, CLI reference
regeneration, OpenAPI library coverage, APK coverage freshness and repository
hygiene pass. Five selected DEX exports reproduce byte-for-byte; subject hash,
complete REA source with warnings and bundle checksums are retained/verified.

## Blocklist migration request and state authority (2026-11-06)

**Confirmed selected static, pinned APK43.72:** [complete worker/constructor,
mode storage/census, migration builder/response and asset export](apk/2.26.43.72-blocklist-migration-obligation/README.md)
close8bU worker0 ->1dH.A0O and expose the distinct dirty worker1 path through
lookup/merge and MigrateBlocklistLid. Its whatsapp-android-mex descriptor uses
server doc_id25028600226770432; input holds dhash, blocklist entries with nested
identifier, and a conditional dirty_ack=true setter. Concrete identifier values
come from multiple providers; full authority/null serialization remains open.

3BU stores is_blocklist_lid_based through the namespaced0aj store. A parsed
LID/mapping is not proof of that mode. The selected manager accepts7Da, but
only isRollback=false both writes mode=true and adopts returned cache data.
The true branch still reaches the completion log while skipping both writes.
Missing typed response structure or caughtGiw yields7Db; manager throws7gZ.
Parsed response, log completion and account-safe adoption are separate results.

Rust's matched=false warning/manual full-fetch path remains unchanged and is
not native migration equivalence. MigrateBlocklistLid sender, mode adoption,
lookup/alias authority, full metadata/serialization, final envelope, callback
lifetime/deadline and writer/account-generation policy remain unported. No
classic20s block/set or guessed32s MEX timeout is assigned. No action replay,
automatic fetch or live account operation. Shared helper0Ef.A06 is a union;
parsed878.A01 keys are PN-association input, not an invented local block cache.

Validation:18 selected DEX/census/asset exports reproduce byte-for-byte; the
MEX extractor's provenance check, APK subject hash, complete REA manager source
and bundle checksums match. Formatting, CLI reference regeneration, OpenAPI
library coverage, APK coverage freshness and repository hygiene pass. No Rust
behavior changed, so no new runtime-test or live-result claim is made.

## Migration values and coroutine completion (2026-11-06)

**Confirmed selected pinned static:** [complete input/object conversion,
callback discriminators/constructors and shared-operation comparison](apk/2.26.43.72-blocklist-mex-values-lifetime/README.md)
close several previously open intermediate boundaries. Direct String/Boolean
GraphQlCallInput setters append null values; A03/private conversion retains
those named values in TreeMap, with last-pair overwrite for duplicate names.
The0i1 primitive wrappers instead skip null. Final wire JSON null/omission
semantics remain unverified, so neither layer alone dictates transmitted bytes.

Migration2LE.A0Q selectsIE8 case3 through0iq.ARv. CxG case16 installs success
CxE case43 and errorCxG case15; constructors/casts bind their captures.
The policy helper2L8.A1A boxesfalse. The error wrapper therefore resumes
Giw(Hbw) failure to the waiting continuation; it is not evidence for an
intentional indefinite error wait. That matches the prior8ht catch/7Db/7gZ
path rather than7Da mode/cache adoption. All native callback races, account
ownership and lower error admission remain separate.

The current factory/constructor and direct producer path bind32000ms to the
selected non-Pando MEX send branch. The callback initializer does not alter
that default. Pando effective timeout, outer wait/cancellation and write lifetime
are not inferred as32s. The operation handle is discarded byAQY; no new retained
cancel authority is shown. Existing shared factory/adapter exports were checked
against fresh selected methods. No Rust behavior or live account operation.

Validation:15 selected DEX exports reproduce byte-for-byte; APK subject hash,
complete REA map-export source and bundle checksums match. Shared operation
constructor/ARv and callback adapter match the retained methods. Formatting,
CLI reference regeneration, OpenAPI library coverage, APK coverage freshness
and repository hygiene pass. No Rust behavior changed; runtime tests were not
rerun and no new runtime/live result is claimed for this research-only chunk.

## Migration JSON controlled reference (2026-11-06)

**Confirmed selected APK binding plus controlled external reference:**
[reused shared envelope, six source hashes and four executed cases](apk/2.26.43.72-migration-json-reference/README.md)
connect the prior migration-shaped maps to0mF's JSONObject(Map)/queryId/variables
wrapper. The same shared serializer and pinned Android source had already been
inspected for another MEX path; this chunk adds controlled execution, not a
second discovery of that static implementation. Fresh constructor/trace/guard
exports match retained source. Text-to-byte exports bind08Q.A05 toUTF8.

The oracle compiles six unchanged pinned AOSP JSON sources with annotation-only
compatibility stubs on JDK21, then tests only synthetic maps. Four cases retain
explicit null presence, Boolean types/conditional absence, escaped string/UTF8
roundtrip and empty-array semantics; complete emitted bodies are checked against
expected.json. The query ID matches the pinned migration asset. Initial explicit
public-source fetch and later offline execution both pass; no APK, device or
account was executed. Exact live/OEM/Pando serialization and positive identifier
policy remain unverified. No production sender, retry or migration state change.

Validation: four controlled reference cases pass with the pinned public-source
cache, including a fresh opt-in fetch followed by offline execution. Both retained
text/charset DEX exports reproduce byte-for-byte and the APK hash matches.
Bundle checksums, formatting, CLI reference regeneration, OpenAPI library
coverage and APK coverage freshness pass. No Rust runtime behavior changed;
workspace runtime tests were not rerun for this research-only chunk.

## Blocklist snapshot repeated-identity membership (2026-11-06)

**Confirmed pinned mismatch / correction:** the retained
[6qD.A02 and80Z methods](apk/2.26.43.72-blocklist-snapshot-children/README.md)
write every parsed item's action==block Boolean with Map.put and derive the
blocked set from final true values. A later unblock (or other non-block action)
for the same primary identity therefore removes an earlier block. Rust skipped
all non-block items before identity resolution and refused repeated block items.
It could retain the wrong blocked membership or stall a valid snapshot.

The projector now resolves supported primary identities for every item, replaces
repeated block rows, and removes prior snapshot membership/rows for a later
non-block action. Final cache/hash/inspection writes remain transactional. The
existing strict identity, unknown-child and malformed-list guards remain;
metadata continues the adapter's last-block-row policy, not full native separate
conditional metadata-map/alias parity. No sender, automatic fetch or replay added.

The synthetic repeated_snapshot_identity_uses_last_membership_after_reopen
regression covers block/unblock, block/block, unblock/block, block/other and
block/unblock/block. The old projector fails the first case by retaining a block.
The corrected projector checks final membership and supplied last-block metadata,
hash, once-only projection, unchanged journal and no recreated pending request,
including reopening the encrypted store. Missing/invalid identity admission,
conditional metadata merging, alias/migration state, malformed-history recovery
and repair of already projected histories remain separate gaps. No live result
or ban-rate claim follows.

Validation:1,517 workspace tests pass (9 result suites, four test threads),
including the five-case ordering/reopen regression. The old projector fails
block/unblock with actual blocked count1 instead of0. Strict workspace/all-target/
all-feature Clippy, formatting, CLI regeneration, OpenAPI library coverage,
APK coverage freshness and repository hygiene pass. The retained parser/result
exports reproduce byte-for-byte, bundle checksums and subject APK hash match.

## Blocklist typed identity and metadata priority (2026-11-06)

**Confirmed selected static:** [six complete DEX helpers, retained parser,
branch analysis and complete REA Evidence](apk/2.26.43.72-blocklist-identity-metadata/README.md)
resolve two previously open parser details.0Tp.A0A returns null after a failed
requested-class cast;0Dd.A02 returns null for absent/empty or caught parse
failure. Non-compliance logging inA0A is not itself rejection.6qD.A02 then
falls back from UserJid jid to PhoneUserJid pn_jid, and logs/skips when neither
resolves. Rust currently refuses invalid/missing identities and does not model
that fallback's exact requested PhoneUserJid class.

Metadata uses a type18 gate (0Dn.A0Z; concrete0Dx getter18, PhoneUserJid getter0).
Present display_name has priority, including empty suppressing later branches;
only null name reaches PN association, and only null typed PN reaches username.
These operations precede the separate action Boolean map put, so membership
last-write semantics do not prove last-row metadata semantics. All-class/type
mapping, full JID grammar and durable downstream metadata/alias adoption remain
open. Focused0Dw.getType had no body; inherited methods are not disproved.
No Rust behavior, snapshot skip policy, sender, account or live request changed.

Validation: six complete helper DEX exports reproduce byte-for-byte; APK digest,
bundle checksums and complete retained REA data match. Formatting, CLI reference
regeneration, OpenAPI library coverage and APK coverage freshness pass. Coverage
now retains964 distinct REA records,0 duplicate copies,72 explicitly mapped.
No Rust behavior changed; workspace runtime tests were not rerun for this
research-only chunk. Repository hygiene passes (421 checksum bundles).

## Blocklist metadata adoption authority (2026-11-06)

**Confirmed selected static:** [complete7to worker, map constructor, blank
predicates, selected task/comparator branches and REA Evidence](apk/2.26.43.72-blocklist-metadata-adoption/README.md)
connect the parsed metadata to a separately scheduled update task. Fresh1dH.A0O
matches the retained replacement method; it schedules8be case5 ->7to.A00 before
the membership no-difference branch.878 binds A01 associations, A02 usernames,
A00 display names. No cross-store atomicity follows from scheduling.

Associations sort active first, then lexical Jid.user. Active candidate or no
existing association selects the candidate; otherwise the existing choice is
retained. Non-chosen candidates are submitted before chosen to0Ws.A0Q; writer
semantics remain open. Username proposals go to0bY.BQ1 only if neither of the
two consulted result maps contains the key. Display-name proposals go toBPw
only if the existing12f/A00 is missing/null/blank. Blank is empty or all UTF16
chars satisfying Character.isWhitespace OR isSpaceChar, not an unverified Rust
trim equivalence. Empty maps skip; no broad metadata-clear operation is shown.

This differs from Rust's unified last-block-row metadata replacement/deletion.
Concrete0bY/0Ws backend selection, durable commits, contact-store ownership and
alias side effects are still needed for a correct adapter. No guessed merge,
production code, sender, retry, automatic fetch or live account operation added.

Validation: eight selected exports reproduce; the five complete helper files,
retained replacement method and both labelled partial-case excerpts match.
The README extraction runs exactly; bundle checksums, complete REA data and
APK digest match. Formatting, CLI reference regeneration, OpenAPI library
coverage and APK coverage freshness pass (965 distinct REA records,0 duplicate
copies,72 explicitly mapped). No Rust behavior changed; workspace runtime tests
were not rerun for this research-only chunk.
Repository hygiene passes (422 APK checksum bundles).

## Blocklist metadata concrete write boundaries (2026-11-06)

**Confirmed selected static:** [declaration census, complete writer/normalizer/
value methods and REA negative](apk/2.26.43.72-blocklist-store-writes/README.md)
find0bY abstract plus0bZ/12d concrete BQ1 declarations. The superclass-only
query cannot resolve interface dispatch; active provider binding remains open.
12d wrappers call per-entry Boolean setters without checking their results;
returned input keys are not evidence of every backend write succeeding.

0bZ normalizes usernames by Android-whitespace trim then all leading @ removal,
with no lowercasing in that selected function; null/empty maps to an empty
singleton string. It compares changed values, treats empty values separately,
filters selected unresolved row IDs and performs database, delegated writer,
invalidation-related and notification operations across separate stages. The
display writer has a DB-not-ready empty-result branch and conditional delegation.
12g's username fallback supplies an empty display_name on its insert-style
call; its positive affected-count branch does not clear the existing name.
No all-stages atomic success guarantee follows from these wrappers.

REA returned a method-exists/no-body comment for0bZ.BQ1 despite concrete DEX
instructions. The full Evidence is retained as body unavailable, not a complete
source port. Active binding,0Ws/3LI/0KU concrete effects and Rust contact metadata
ownership remain open. No production code, metadata erase, retry, sender,
automatic request or live account operation changed.

Validation: twelve selected export commands reproduce byte-for-byte, including
the three-declaration BQ1 census and the explicitly superclass-only negative.
Bundle checksums, retained REA unavailable-body data and APK digest match.
Formatting, CLI reference regeneration, OpenAPI library coverage and APK
coverage freshness pass (966 distinct REA records,0 duplicate copies,72 mapped).
No Rust behavior changed; workspace runtime tests were not rerun for this
research-only chunk.
Repository hygiene passes (423 APK checksum bundles).

## Blocklist default provider binding (2026-11-06)

**Confirmed selected static:** [provider/cache methods, exact factory cases,
constructor and caller census](apk/2.26.43.72-blocklist-provider-binding/README.md)
resolve the packaged default path from7to provider0xb43 to0bZ.05d/05h preserve
the ID;00D.A03 checks its cache then uses1Uw factories. Root selector0 and
low14-bit range0xaf7..0xc10 reachA0S key2883 ->0x430. Root/range dispatch for
0x430 reachesA04 key1072, constructing0bZ. Its delegate0x439 reaches12d and
metadata helper0x438 reaches12g. The earlier candidate declaration evidence
is now connected to this default static construction route.

This does not observe current runtime cached objects or enumerate override/
reset lifecycle. No cache miss or new construction is presumed at runtime.
Other0Ws/3LI/0Wv provider/backend effects remain separate. Rust's contact
address book has user-saved PN names, not the distinct LID username/display-name
stores shown here; those names must not be overwritten as a substitute adapter.
No Rust behavior, identity merge, sender, retry or live account operation added.

Validation: thirteen export commands reproduce; complete bridge/cache methods,
constructor caller census and three explicitly partial dispatcher excerpts match.
The README extraction executes exactly; bundle checksums and APK digest match.
Formatting, CLI reference regeneration, OpenAPI library coverage and APK coverage
freshness pass. No new REA record or Rust behavior; workspace runtime tests were
not rerun for this research-only chunk.
Repository hygiene passes (424 APK checksum bundles).

## Blocklist set first-list admission (2026-11-06)

**Confirmed pinned mismatch/correction:** [required first-child method, reused
callback/parser and regression](apk/2.26.43.72-blocklist-set-first-list/README.md)
show8R7.CK4 requires children[0] to be list and parses it before matched.
Rust now follows that first-child selection and validates through the shared
snapshot parser before any set-result projection. Only the selected list's hash
and matched value count; missing/wrong-first or malformed lists cannot apply a
requested change/hash or be marked inspected. Trailing metadata cannot redirect
the first-list branch. Existing matched=false reconciliation hold remains.

The regression covers both block/unblock across eight result shapes, immutable
settlement/reply, full checkpoint/journal preservation on refusal, encrypted
reopen, selected hash/membership and no resend. The old projector fails the
missing-list case with Ok(1). Existing positive fixtures now carry native list
shape. Fetch no-list compatibility remains; malformed old set histories refuse
without automatic quarantine/skip/repair. Strict Rust identity/metadata admission,
separate native metadata/alias/migration and previously projected history repair
remain gaps. No live account operation or new request added.

Validation:1,518 workspace tests pass (9 result suites, four test threads),
including the sixteen block/unblock shape/reopen cases in the new regression.
The old projector fails the missing-list case with Ok(1). Strict workspace/
all-target/all-feature Clippy, formatting, CLI reference regeneration, OpenAPI
library coverage, APK coverage freshness and repository hygiene pass. Fresh
callback/first-child exports reproduce and subject hash/bundle checksums match.

## Blocklist full-snapshot nullable hash (2026-11-06)

**Confirmed mismatch/correction:** [nullable accessor, replacement-task binding,
selected write case, set gating and Android API contract](apk/2.26.43.72-blocklist-null-snapshot-hash/README.md)
show admitted full snapshots replace even a missing hash: native nullputString
removes the saved key. Rust now deletes its hash checkpoint in the same snapshot
transaction. A no-list unchanged fetch still preserves it; malformed input
preserves every checkpoint and the journal. No automatic request is added.

The four-case encrypted-store regression covers same/empty membership no-hash
snapshots, no-list and malformed controls, reopen/idempotency/reconciliation,
and explicitly queued offline get/block controls proving subsequent construction
omits the stale hash only after admitted replacement. The old projector fails
by retaining Some(stale). Set-result hash mutation has a separate native memory/
backend execution-success gate (clarified below); this fix does not guess its complete parity or rewrite old
projected histories. Native metadata/alias/migration and live behavior remain open.

Validation:1,519 workspace tests pass (9 result suites, four test threads),
including four snapshot/reopen and offline request-construction cases. The old
projector fails with Some(stale) instead ofNone. Strict workspace/all-target/
all-feature Clippy, formatting, CLI reference regeneration, OpenAPI library
coverage, APK coverage freshness and repository hygiene pass. Four DEX exports,
README extraction, bundle checksums and subject APK hash match. Android null
removal is attributed to its API contract, not a device execution result.

## Blocklist outbound hash omission (2026-11-06)

**Confirmed pinned mismatch/correction:** [reused builders/predicates, authored
Java oracle and complete BMP reference](apk/2.26.43.72-blocklist-outbound-hash/README.md)
show distinct rules: get omits only null/empty; block/unblock omit null/blank
under Java UTF16 Character.isWhitespace OR isSpaceChar. Rust now preserves that
distinction and sends nonblank values unchanged, without changing saved hashes.
A Rust trim substitute would be wrong at U+001C..001F and U+0085.

The independent Java21 oracle measures every UTF16 unit; the Rust predicate is
checked against its entire true/false domain. Synthetic request/store controls
cover ten hash cases for get/block/unblock and unchanged saved bytes. The old
code fails the empty-hash get case. Host-Java reference is not Android/OEM
execution; full runtime Unicode-version parity remains unobserved. No new
sender, automatic request, state reset or live account operation added.

Validation:1,521 workspace tests pass (9 result suites, four test threads).
The two new regressions cover every UTF16 unit against the executed Java21
reference and ten get/block/unblock hash cases; the old builder fails empty get.
Strict workspace/all-target/all-feature Clippy, formatting, CLI reference
regeneration, OpenAPI library coverage, APK coverage freshness and repository
hygiene pass. Four reused DEX exports, regenerated oracle output, bundle
checksums and subject APK hash match. No live result is claimed.

## Blocklist set hash membership gate (2026-11-06)

**Confirmed pinned correction and interpretation refinement:** [complete backend
result/helpers and reused success/write chain](apk/2.26.43.72-blocklist-set-hash-gate/README.md)
show7vV only calls the hash-writing A08 when the requested target changes the
in-memory set.7PB.A0Q's true is normal execution/close, not a positive affected
row count: helper long results are discarded. Earlier backend-change wording
is corrected to execution-success. Selected exception handling is retained.

Rust's supported single durable-cache projector now changes hash only when
its requested target changes membership. Newly applied changes adopt the
selected-list nullable hash (missing deletes); already-satisfied block/unblock
preserve cache/hash bytes. Valid results still get their once-only inspection;
unmatched/malformed guards, correlation and atomic transaction remain. No
new sender, retry or live operation. The12-case regression covers both actions,
prior presence and missing/nonempty/empty hashes, unrelated contact/metadata,
reopen/idempotency/journal and no resend. Native separate memory/DB/cache/alias
partial failures, stale-state equivalence and historical repair remain open.

Validation:1,522 workspace tests pass (9 result suites, four test threads),
including the12-case membership/hash/reopen matrix and updated first-list
control. The old projector fails changed membership with a missing hash by
retaining Some(before). Strict workspace/all-target/all-feature Clippy,
formatting, CLI reference regeneration, OpenAPI library coverage, APK coverage
freshness and repository hygiene pass. Five selected DEX exports, both changed
bundle checksums and subject APK hash match. No live result is claimed.

## Blocklist fetch first named child (2026-11-06)

**Confirmed pinned mismatch/correction:** [complete selector, reused callback
and five-case regression](apk/2.26.43.72-blocklist-fetch-first-match/README.md)
show6p9.CK4 uses0Tp.A0F to select the first immediate child named list, with
no recursion or duplicate-list veto. This differs from set's required first
child. Rust now validates/applies only that first named fetch list; later lists
cannot stall a valid first snapshot. A malformed first match still refuses and
cannot fall through. Nested-only/no-match lists retain unchanged behavior.

The encrypted-store regression covers two valid lists, prefixed extension,
valid/malformed and malformed/valid ordering, plus nested-only control. It
checks selected cache/hash/reconciliation, refused checkpoint preservation,
reply/journal, reopen/idempotency and no resend. The old selector fails on two
valid lists with its invented ambiguity refusal. Previously unprojected valid
histories may now project once; already projected history is not rewritten.
Strict selected-node parser admission and native dirty/metadata/alias/migration
remain gaps. No new sender, retry, automatic fetch or live operation added.

Validation:1,523 workspace tests pass (9 result suites, four test threads),
including the five-case selector/reopen regression. The old selector fails
on two valid lists with InvalidInput(ambiguous blocklist snapshot). Strict
workspace/all-target/all-feature Clippy, formatting, CLI reference regeneration,
OpenAPI library coverage, APK coverage freshness and repository hygiene pass.
Fresh selector/callback exports, bundle checksums and subject APK hash match.
No live result is claimed.

## Blocklist dirty fetch role and admission (2026-11-06)

**Confirmed selected static:** [manager/provider bridges, complete migration
predicate/state, reused role/callback and branch table](apk/2.26.43.72-blocklist-dirty-role-admission/README.md)
resolve the post-dirty gate as account role09F.BU7, not a remote feature flag.
Its existing concrete implementations test registered device index >0 for
companion mode. After selected snapshot adoption, exact dirty=true and BU7=false
queue8bU selector1's mapping/MEX migration work. Companion mode skips that task;
this is post-adoption work, not a reason to resend the original request.

The earlier migration check uses0f5.A0H ->cached0HP.A00/A01, with readiness and
diagnostic/assertion fallback boundaries. The callback's readiness false branch
at00a2 jumps directly to00f4 adoption, bypassing mode/migration checks. With
readiness true, existing3BU mode admits; otherwise trueA0H writes mode then
admits, while false takes the invalid/unmigrated delivery path. No-list exits
before these gates. Cache writers/effective runtime authorities remain open.
Rust lacks equivalent role/migration providers and dirty post-adoption work;
this trace does not justify an unconditional mode/warning change or fabricated
state. No production sender, retry, automatic request or live operation added.

Validation: nine selected DEX exports reproduce byte-for-byte; six decisive
relative branch destinations are independently calculated by the executed
README verifier. Complete retained REA data, bundle checksums and APK digest
match. Formatting, CLI reference regeneration, OpenAPI library coverage and
APK coverage freshness pass (967 distinct REA records,0 duplicate copies,72
explicitly mapped). No Rust behavior changed; workspace runtime tests were
not rerun for this research-only chunk.
Repository hygiene passes (430 APK checksum bundles).

## Blocklist account-sync notification obligation (2026-11-06)

**Confirmed selected static:** [first-child route, prev_dhash/admission delta
conditions, worker and kind4 refresh](apk/2.26.43.72-blocklist-account-sync-obligation/README.md)
connect blocklist changes toaccount_sync, not a standalone notification type.
The parsed delta applies only with matching nullableprev_dhash, successful
6pA admission and effective membership changes. modify/mismatch/failed admission
queues a refresh; malformed parsing does not imply that fallback. Empty or
no-effective-change deltas do not advance the saved hash in this branch.

The worker persists current set/hash/metadata asynchronously; normal19N return
has no base ACK deferral. Refresh goes through2D6.A01 ->1sx.A01 ->kind4 ->1dH.A0M,
with false suppression flag and separate saved retry bookkeeping. Rust leaves
this unsupported account child captured without ACK or execution. Admission,
durable work/dedup/uncertain outcomes and remaining backend policy must be
mapped before ACK support. No blanket ACK, blind snapshot/delta adoption,
new sender, retry or live operation added.

Validation: ten selected export commands reproduce, including the reused full
account handler/refresh manager and admission/task constructor. README extraction
and six relative branch checks execute exactly; bundle checksums and subject
APK hash match. Formatting, CLI reference regeneration, OpenAPI library coverage
and APK coverage freshness pass. No new REA record or Rust behavior; workspace
runtime tests were not rerun for this research-only chunk.
Repository hygiene passes (431 APK checksum bundles).

## Blocklist delta storage and side-effect boundaries (2026-11-06)

**Confirmed selected static:** [whole-set store, current-set copy, transaction,
preference and system-message/follow-up methods](apk/2.26.43.72-blocklist-delta-storage-boundary/README.md)
show the async delta worker copies current membership at execution, then
replaces wa_block_list under its transaction wrapper. The captured hash is
written afterward via preferences apply, followed by metadata/follow-up work.
There is no established all-stages atomicity or hash/set generation binding
without executor-order evidence; no actual race or successful commit observed.

Individual insert/delete results are discarded, while selected thrown write/
close failures propagate. Per-targetA0A delegatesA0Q, which suppresses an
equivalent latest block-state system-message model or constructs and hands off
a new one.8be selector6 closes the changed-union scheduling handoff toA09 ->3XP;
consumer effects remain untraced. This is not authority to fabricate a user
message, replay a request, ACK unsupported notifications, or weaken Rust's
atomic projection. No production behavior or live account operation changed.

Validation: twelve selected DEX export commands reproduce byte-for-byte;
README selected-task extraction executes exactly. Complete retained REA data,
bundle checksums and subject APK hash match. Formatting, CLI reference
regeneration, OpenAPI library coverage and APK coverage freshness pass (968
distinct REA records,0 duplicate copies,72 explicitly mapped). No Rust behavior
changed; workspace runtime tests were not rerun for this research-only chunk.
Repository hygiene passes (432 APK checksum bundles).

## Blocklist observer interface census (2026-11-06)

**Confirmed selected static plus offline tool correction:** [CbC selector7,
complete observer dispatcher and37-declaration interface census](apk/2.26.43.72-blocklist-observer-interface/README.md)
close1dH.A09's callback to0d0.Blg(Collection). The original invoke-name lookup
had no body because this callback's actual declared method isCbC. Marker/thread
classification in07o can invoke or schedule listeners; downstream job effects
and active registration remain open, so no UI-only/network-free claim follows.

apk-virtual-overrides now offers --include-interfaces for transitive interface/
superclass **declared** method ancestry. Default output is unchanged. Literal
DEX metadata, diamond/cycle, truncation/type/duplicate checks and both-mode
later-bad-DEX tests protect the census. No active listener count, inherited-target/
default-method/runtime resolution or completed protocol port is inferred.
Production notification/ACK behavior remains unchanged; no live request.

Validation:1,525 workspace tests pass (9 result suites, four test threads).
Final xtask119 tests and strict workspace/all-target/all-feature Clippy pass,
along with formatting, CLI reference regeneration, OpenAPI library coverage
and APK coverage freshness. Five export commands reproduce;37 declared bodies,
README extraction, checksum bundle and subject APK digest match. Default
superclass-only output matches the retained negative byte-for-byte. No live
account, production protocol change or complete runtime dispatch claim.
Repository hygiene passes (433 APK checksum bundles).

## Blocklist location observer query boundary (2026-11-06)

**Confirmed selected static:** [Fef selector39 and complete location helper methods](apk/2.26.43.72-blocklist-location-obligation/README.md)
connect DC3.Blg to a conditional3KB.A01(all_changed,false) missing-LID
preflight. Existing per-chat EVN participant state gates the call; helper
filtering can still avoid a query. A specific network-unavailable sentinel
returns before later updates. Otherwise local location-state helpers/listeners
and a posted job run; their complete wire effects remain unknown. This is not
an unconditional request on block/unblock or proof every observer is UI-only.

Reuses retained pre-key identity-query construction/queue/MEX evidence rather
than inventing another IQ. Active registrations, runtime state, downstream
location effects and durable notification/ACK ownership remain open. Rust
continues capturing unsupported blocklist notifications without ACK; no new
production request, location feature or live account operation.

Validation: four fresh APK exports and selected-case extraction match; branch
targets, checksums and pinned APK digest verified. Formatting, OpenAPI library
coverage, APK coverage freshness and repository hygiene pass (434 bundles).
This evidence-only chunk changes no Rust behavior; no live test was performed.

## Blocklist observer location-disable obligation (2026-11-06)

**Confirmed selected static:** [complete cancellation, sequence, stale-job check
and outbound job methods](apk/2.26.43.72-blocklist-location-disable/README.md)
close the prior A02 boundary. No existing EVN means no cancellation/job; the
observer's null message key takes the matching-record path when an EVN exists.
Selected destination mapping may fall back to the original phone identity.
Cancellation removes the record and allocates a saved sequence, then queues
SendDisableLiveLocationJob. Local helper failures can still prevent enqueue.

The job rechecks current sharing, suppressing a stale disable when a live share
exists; an expired record can instead trigger cleanup. Admitted execution builds
notification/type=location/to=destination with a disable child whose id is the
saved decimal sequence. Its outer request ID is generated separately. A0C with
discriminator81 registers the ACK-backed future; get waits and discards the
reply object using the already-retained A01/2Gc0 result contract. Neither the
normal return nor apply-based sequence storage proves remote success/durability.

**Unknown/unported:** active observer/state authority, clock and crash recovery,
all sequence writers, downstream local helpers, job retry/dedup, complete
producer-specific ACK failure handling. Rust has no corresponding transport
share/sequence owner; adding the packet alone would invent its prerequisites.
Unsupported account_sync/blocklist remains captured without ACK. No runtime,
automatic lookup/retry, location feature or live account change.

Validation: six fresh selected-method exports reproduce byte-for-byte; bundle
checksums and selected cancellation, stale-job and sequence branch targets
match. Formatting, OpenAPI library coverage, APK coverage freshness, diff
whitespace and repository hygiene pass (435 bundles). No Rust code changed;
no APK execution or live account test was performed.
