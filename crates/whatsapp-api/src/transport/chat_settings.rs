//! Disappearing-message timer, keep-in-chat and advanced chat privacy ("limit
//! sharing"), traced from comparison APK **2.26.37.71** protobuf-lite
//! descriptors (`docs/protocol/chat-actions.md` §3, §4, §5). All three are
//! ordinary E2E `message type=text` payloads, using the existing ratchet.
//!
//! * `ProtocolMessage` (`Message.protocolMessage`, field 12; `LX/BZb`
//!   descriptor, confirmed): `key`=1 (reuses `super::reactions::Key`),
//!   `type`=2, `ephemeralExpiration`=4 (uint32 seconds), `ephemeralSettingTimestamp`=5,
//!   `disappearingMode`=11 (`initiator`=1, `trigger`=2, `initiatorDeviceJid`=3,
//!   `initiatedByMe`=4 — confirmed against the current BW7 descriptor), `limitSharing`=24
//!   (`sharingLimited`=1 bool, `trigger`=2 enum, `limitSharingSettingTimestamp`=3
//!   int64, `initiatedByMe`=4 bool — confirmed, `LX/BWR`), `afterReadDuration`=29,
//!   `acp2Setting`=35 (`enabled`=1 bool, `trigger`=2 enum, `settingTimestamp`=3
//!   int64, `initiatedByMe`=4 bool — confirmed, `LX/BWQ`). Type constants
//!   (confirmed, `LX/C4F.<clinit>`): `EPHEMERAL_SETTING`=3,
//!   `EPHEMERAL_SYNC_RESPONSE`=4, `LIMIT_SHARING`=27, `ACP2_SETTING`=39.
//!   Trigger `CHAT_SETTING`=1 (`LX/C3L`/`LX/C3j`, confirmed).
//! * `KeepInChatMessage` (`Message.keepInChatMessage`, field 51; `LX/BZc`
//!   descriptor, confirmed): `key`=1, `keepType`=2 (`UNKNOWN`=0,
//!   `KEEP_FOR_ALL`=1, `UNDO_KEEP_FOR_ALL`=2, `LX/C2L`), `timestampMs`=3.
//!
//! Neither `disappearing_payload` nor `limit_sharing_payload` sets `key`: both
//! settings apply to the whole chat, not to one target message (chat-actions.md
//! §3, §5). `keep_payload` follows the reaction/poll key convention: `remoteJid`
//! is the identity supplied by the caller (the contact's LID when their
//! messages arrive LID-addressed), `fromMe` is relative to the sender of this
//! keep action, matching the reaction/poll key semantics confirmed live.
use crate::{
    Error, Result,
    model::{
        ids::MessageId,
        jid::Jid,
        message::{Direction, MessageRef},
    },
};
use prost::Message as _;

/// `ProtocolMessage.type` EPHEMERAL_SETTING (per-chat disappearing timer).
pub const TYPE_EPHEMERAL_SETTING: i32 = 3;
/// `ProtocolMessage.type` EPHEMERAL_SYNC_RESPONSE (peer echoing its setting).
pub const TYPE_EPHEMERAL_SYNC_RESPONSE: i32 = 4;
/// `ProtocolMessage.type` LIMIT_SHARING (advanced chat privacy, v1).
pub const TYPE_LIMIT_SHARING: i32 = 27;
/// `ProtocolMessage.type` ACP2_SETTING (advanced chat privacy, v2).
pub const TYPE_ACP2_SETTING: i32 = 39;
/// Trigger value used for a user-initiated chat-setting change.
pub const TRIGGER_CHAT_SETTING: i32 = 1;
/// `KeepInChatMessage.keepType` KEEP_FOR_ALL.
pub const KEEP_FOR_ALL: i32 = 1;
/// `KeepInChatMessage.keepType` UNDO_KEEP_FOR_ALL.
pub const UNDO_KEEP_FOR_ALL: i32 = 2;

/// Disappearing timer values the official UI offers, plus `0` (off). Any
/// `u32` can be encoded by the raw [`disappearing_payload`] codec. Native
/// builders interpret its bits as a Java int and clamp nonpositive values
/// to zero; normal intents require the nonnegative signed range.
pub const UI_TIMER_VALUES: [u32; 4] = [0, 86_400, 604_800, 7_776_000];

fn invalid(reason: &'static str) -> Error {
    Error::invalid("chat_settings", reason)
}

#[derive(prost::Message)]
pub(crate) struct Envelope {
    #[prost(message, optional, tag = "12")]
    pub(crate) protocol: Option<ProtocolMessage>,
    #[prost(message, optional, tag = "51")]
    pub(crate) keep_in_chat: Option<KeepInChatMessage>,
}
#[derive(prost::Message)]
pub(crate) struct ProtocolMessage {
    #[prost(message, optional, tag = "1")]
    pub(crate) key: Option<super::reactions::Key>,
    #[prost(int32, optional, tag = "2")]
    pub(crate) type_: Option<i32>,
    #[prost(uint32, optional, tag = "4")]
    pub(crate) ephemeral_expiration: Option<u32>,
    #[prost(int64, optional, tag = "5")]
    pub(crate) ephemeral_setting_timestamp: Option<i64>,
    #[prost(message, optional, tag = "11")]
    pub(crate) disappearing_mode: Option<DisappearingMode>,
    #[prost(int64, optional, tag = "15")]
    pub(crate) timestamp_ms: Option<i64>,
    #[prost(message, optional, tag = "24")]
    pub(crate) limit_sharing: Option<LimitSharing>,
    #[prost(uint32, optional, tag = "29")]
    pub(crate) after_read_duration: Option<u32>,
    #[prost(message, optional, tag = "35")]
    pub(crate) acp2_setting: Option<Acp2Setting>,
}
/// `LX/BW7`; field numbers are inferred (public-schema alignment), not
/// individually confirmed against the descriptor dump.
#[derive(prost::Message)]
pub(crate) struct DisappearingMode {
    #[prost(int32, optional, tag = "1")]
    pub(crate) initiator: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub(crate) trigger: Option<i32>,
    #[prost(string, optional, tag = "3")]
    pub(crate) initiator_device_jid: Option<String>,
    #[prost(bool, optional, tag = "4")]
    pub(crate) initiated_by_me: Option<bool>,
}
/// `LX/BWR` (confirmed).
#[derive(prost::Message)]
pub(crate) struct LimitSharing {
    #[prost(bool, optional, tag = "1")]
    pub(crate) sharing_limited: Option<bool>,
    #[prost(int32, optional, tag = "2")]
    pub(crate) trigger: Option<i32>,
    #[prost(int64, optional, tag = "3")]
    pub(crate) limit_sharing_setting_timestamp: Option<i64>,
    #[prost(bool, optional, tag = "4")]
    pub(crate) initiated_by_me: Option<bool>,
}
/// `LX/BWQ` (confirmed).
#[derive(prost::Message)]
pub(crate) struct Acp2Setting {
    #[prost(bool, optional, tag = "1")]
    pub(crate) enabled: Option<bool>,
    #[prost(int32, optional, tag = "2")]
    pub(crate) trigger: Option<i32>,
    #[prost(int64, optional, tag = "3")]
    pub(crate) setting_timestamp: Option<i64>,
    #[prost(bool, optional, tag = "4")]
    pub(crate) initiated_by_me: Option<bool>,
}
#[derive(prost::Message)]
pub(crate) struct KeepInChatMessage {
    #[prost(message, optional, tag = "1")]
    pub(crate) key: Option<super::reactions::Key>,
    #[prost(int32, optional, tag = "2")]
    pub(crate) keep_type: Option<i32>,
    #[prost(int64, optional, tag = "3")]
    pub(crate) timestamp_ms: Option<i64>,
}

/// A recognised chat-setting `protocolMessage`, decoded from field 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    /// Per-chat disappearing-message timer (type `EPHEMERAL_SETTING`).
    DisappearingTimer {
        /// New timer in seconds; `0` means off.
        seconds: u32,
    },
    /// The peer echoing its own ephemeral setting (type
    /// `EPHEMERAL_SYNC_RESPONSE`). This carries the same `ephemeralExpiration`
    /// field as `EPHEMERAL_SETTING` per the `LX/BZb` descriptor, so callers
    /// that only care about the effective timer can treat the two the same
    /// way (see `docs/protocol/chat-actions.md` §3 "Implementation").
    EphemeralSyncResponse {
        /// Echoed timer in seconds.
        seconds: u32,
    },
    /// Advanced chat privacy v1 (type `LIMIT_SHARING`).
    LimitSharing {
        /// Whether sharing/forwarding out of the chat is limited.
        enabled: bool,
    },
    /// Advanced chat privacy v2 (type `ACP2_SETTING`).
    Acp2 {
        /// Whether sharing/forwarding out of the chat is limited.
        enabled: bool,
    },
}

/// Validate a legacy timer intent without silently sending a native disable value.
pub fn validate_timer(seconds: u32) -> Result<()> {
    if seconds > i32::MAX as u32 {
        return Err(invalid("timer seconds exceed the native signed range"));
    }
    Ok(())
}

// BZb/CvY/C4F: proto2 enum values 0..41 excluding seven gaps; absence stays absent.
fn protocol_type(value: u32) -> bool {
    value <= 41 && !matches!(value, 1 | 2 | 12 | 13 | 15 | 23 | 33)
}
fn decode_envelope(payload: &[u8]) -> Option<Envelope> {
    let protocol =
        super::protobuf_enum::decode_nested::<ProtocolMessage>(payload, 12, 2, protocol_type)
            .ok()?
            .map(|(mut message, value)| {
                message.type_ = value.map(|v| v as i32);
                message
            });
    let keep_in_chat =
        super::protobuf_enum::decode_nested::<KeepInChatMessage>(payload, 51, 2, |v| v <= 2)
            .ok()?
            .map(|(mut message, value)| {
                message.keep_type = value.map(|v| v as i32);
                message
            });
    Some(Envelope {
        protocol,
        keep_in_chat,
    })
}

// Bua.CF4 reads a present REVOKE type and parent id; ownership comes from CoT.
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub(super) fn decode_sender_revoke(payload: &[u8]) -> Option<MessageId> {
    let fields = crate::signal::direct::payload_fields(payload).ok()?;
    if fields.iter().any(|f| !matches!(f, 12 | 35))
        || fields.iter().filter(|&&f| f == 12).count() != 1
    {
        return None;
    }
    let protocol = decode_envelope(payload)?.protocol?;
    if protocol.type_ != Some(0) {
        return None;
    }
    let key = protocol.key?;
    // shortcut: native special participant classes need their separate revoke mapper.
    if key.participant.as_ref().is_some_and(|p| !p.is_empty()) {
        return None;
    }
    MessageId::new(key.id?).ok()
}

#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub(super) struct IncomingEdit {
    pub(super) target_id: MessageId,
    pub(super) replacement: zeroize::Zeroizing<Vec<u8>>,
    pub(super) sender_timestamp: i64,
}

// D9B.CF4/Ayo.A0S: only id comes from the embedded key; CoT owns chat/direction.
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub(super) fn decode_edit(payload: &[u8]) -> Option<IncomingEdit> {
    let fields = crate::signal::direct::payload_fields(payload).ok()?;
    if fields.iter().any(|f| !matches!(f, 12 | 35)) {
        return None;
    }
    let protocol = decode_envelope(payload)?.protocol?;
    if protocol.type_ != Some(14) {
        return None;
    }
    let target_id = MessageId::new(protocol.key?.id?).ok()?;
    let raw_protocol =
        zeroize::Zeroizing::new(super::protobuf_enum::message_bytes(payload, 12).ok()??);
    let mut replacement =
        zeroize::Zeroizing::new(super::protobuf_enum::message_bytes(&raw_protocol, 14).ok()??);
    // shortcut: other media and gated poll/event edits need their native content lifecycles.
    let fields = crate::signal::direct::payload_fields(&replacement).ok()?;
    let supported = fields.iter().all(|f| matches!(f, 1 | 6 | 35))
        && fields.iter().any(|f| matches!(f, 1 | 6))
        && super::quotes::decode(&replacement).is_some();
    #[cfg(feature = "experimental-registration-https")]
    let supported = supported
        || super::media::image_edit_caption(&replacement)
            .ok()?
            .is_some();
    if !supported {
        return None;
    }
    if !fields.contains(&35)
        && let Some(context) = super::protobuf_enum::message_bytes(payload, 35).ok()?
    {
        prost::encoding::bytes::encode(35, &context, &mut *replacement);
    }
    Some(IncomingEdit {
        target_id,
        replacement,
        // D5e stores the native proto2 default, independently of stanza time.
        sender_timestamp: protocol.timestamp_ms.unwrap_or(0),
    })
}

#[cfg(all(
    test,
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
#[test]
fn edit_decoder_preserves_merge_context_and_native_timestamp_default() {
    // Independent literal key id P, conflicting remote/fromMe/participant, MESSAGE_EDIT=14.
    let mut protocol =
        b"\x0a\x0d\x0a\x01x\x10\x00\x1a\x01P\x22\x03bad\x10\x0e\x72\x03\x0a\x01a".to_vec();
    let envelope = |protocol: &Vec<u8>, outer_context: bool| {
        let mut body = Vec::new();
        prost::encoding::bytes::encode(12, protocol, &mut body);
        if outer_context {
            body.extend_from_slice(b"\x9a\x02\x02\x08\x01");
        }
        body
    };
    let decoded = decode_edit(&envelope(&protocol, true)).unwrap();
    assert_eq!(decoded.target_id.as_str(), "P");
    assert_eq!(decoded.sender_timestamp, 0);
    assert_eq!(&*decoded.replacement, b"\x0a\x01a\x9a\x02\x02\x08\x01");
    // Repeated editedMessage submessages merge; a present empty inner context wins.
    protocol.extend_from_slice(b"\x72\x03\x9a\x02\x00\x10\x63");
    prost::encoding::int64::encode(15, &-1, &mut protocol);
    let decoded = decode_edit(&envelope(&protocol, true)).unwrap();
    assert_eq!(&*decoded.replacement, b"\x0a\x01a\x9a\x02\x00");
    assert_eq!(decoded.sender_timestamp, -1);
    let mut invalid = envelope(&protocol, false);
    invalid.extend_from_slice(b"\x0a\x01x");
    assert!(decode_edit(&invalid).is_none());
    protocol.extend_from_slice(b"\x10\x04");
    assert!(decode_edit(&envelope(&protocol, false)).is_none());
    assert!(decode_edit(b"\x62\x03\x72\x05x").is_none());
    assert_eq!(
        decode_edit(b"\x62\x0b\x0a\x03\x1a\x01P\x10\x0e\x72\x02\x1a\x00").is_some(),
        cfg!(feature = "experimental-registration-https")
    );
}

// DEy.A00 accepts empty as zero, otherwise int32 1/2/3/7/8; Bua excludes admin 8.
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub(super) fn sender_revoke_edit(node: &crate::binary::Node) -> bool {
    match node.get_attr("edit") {
        None => true,
        Some(value) => value.as_text().is_some_and(|text| {
            text.is_empty()
                || super::mex::java_decimal_long(text).is_some_and(|v| matches!(v, 1 | 2 | 3 | 7))
        }),
    }
}

/// Decode a chat-setting `protocolMessage` (field 12). Returns `None` for
/// malformed content or a `type` this module does not recognise.
pub fn decode_protocol(payload: &[u8]) -> Option<Setting> {
    let protocol = decode_envelope(payload)?.protocol?;
    if matches!(
        protocol.type_,
        Some(TYPE_EPHEMERAL_SETTING | TYPE_EPHEMERAL_SYNC_RESPONSE)
    ) && protocol.after_read_duration.unwrap_or(0) != 0
    {
        // shortcut: after-read timers need feature 25649 and duration/ordering state before projection.
        return None;
    }
    let seconds = (protocol.ephemeral_expiration.unwrap_or(0) as i32).max(0) as u32;
    match protocol.type_ {
        Some(TYPE_EPHEMERAL_SETTING) => Some(Setting::DisappearingTimer { seconds }),
        Some(TYPE_EPHEMERAL_SYNC_RESPONSE) => Some(Setting::EphemeralSyncResponse { seconds }),
        Some(TYPE_LIMIT_SHARING) => Some(Setting::LimitSharing {
            enabled: protocol.limit_sharing?.sharing_limited.unwrap_or(false),
        }),
        Some(TYPE_ACP2_SETTING) => Some(Setting::Acp2 {
            enabled: protocol.acp2_setting?.enabled.unwrap_or(false),
        }),
        _ => None,
    }
}

/// A decoded incoming keep-in-chat action, before mapping into the event model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingKeep {
    /// Wire id of the kept/un-kept message.
    pub target_id: MessageId,
    /// Whether the target message was sent by the acting party.
    pub target_from_sender: bool,
    /// `true` keeps the message for everyone; `false` undoes a previous keep.
    pub keep: bool,
    /// Original BV3 timestampMs (field 3), kept separately from stanza time.
    /// APK 37.71 D9L.A01 at 067a..0682 copies it to BnF.A02; absent wire
    /// fields stay unknown here rather than acquiring an invented clock value.
    pub sender_timestamp: Option<crate::model::time::Timestamp>,
}

/// Decode a `keepInChatMessage` payload (field 51). Returns `None` for
/// malformed content or an unrecognised `keepType`.
pub fn decode_keep(payload: &[u8]) -> Option<IncomingKeep> {
    let keep_in_chat = decode_envelope(payload)?.keep_in_chat?;
    let key = keep_in_chat.key?;
    if super::reactions::key_failure(&key).is_some() {
        return None;
    }
    let target_id = MessageId::new(key.id?).ok()?;
    let keep = match keep_in_chat.keep_type {
        Some(KEEP_FOR_ALL) => true,
        Some(UNDO_KEEP_FOR_ALL) => false,
        _ => return None,
    };
    Some(IncomingKeep {
        target_id,
        target_from_sender: key.from_me.unwrap_or(false),
        keep,
        sender_timestamp: keep_in_chat.timestamp_ms.map(crate::model::time::Timestamp),
    })
}

/// Resolve a supported group keep/unkeep reference using an authenticated
/// envelope sender. APK 37.71 D9L.A01 0682..06b6 passes BV3.key to BoQ.A00;
/// its author is independent of the keeping member. This does not authenticate
/// a supplied sender or establish permission to keep another member's message.
pub fn group_keep_target(
    chat: &Jid,
    sender: &Jid,
    payload: &[u8],
    own: &[Jid],
) -> Result<crate::model::message::MessageKey> {
    let keep = decode_envelope(payload)
        .ok_or_else(|| invalid("malformed_keep"))?
        .keep_in_chat
        .ok_or_else(|| invalid("missing_keep"))?;
    if !matches!(keep.keep_type, Some(KEEP_FOR_ALL | UNDO_KEEP_FOR_ALL)) {
        return Err(invalid("unsupported_keep_type"));
    }
    let key = keep.key.ok_or_else(|| invalid("missing_key"))?;
    super::reactions::group_key_for_sender(chat, sender, key, own).map_err(|error| match error {
        Error::InvalidInput { reason, .. } => Error::invalid("chat_settings", reason),
        other => other,
    })
}

/// Validate a keep-in-chat intent: the target message must belong to `chat`.
pub fn validate_keep(chat: &Jid, target: &MessageRef) -> Result<()> {
    if target.key.chat.jid().to_non_ad() != chat.to_non_ad() {
        return Err(invalid("keep target message belongs to another chat"));
    }
    Ok(())
}

/// Encode a per-chat disappearing-timer `protocolMessage`
/// (`type=EPHEMERAL_SETTING`). No target key: the setting applies to the
/// whole chat (chat-actions.md §3).
pub fn disappearing_payload(seconds: u32) -> Vec<u8> {
    Envelope {
        protocol: Some(ProtocolMessage {
            key: None,
            type_: Some(TYPE_EPHEMERAL_SETTING),
            ephemeral_expiration: Some(seconds),
            ephemeral_setting_timestamp: Some(
                crate::model::time::Timestamp::now().as_unix_seconds(),
            ),
            disappearing_mode: None,
            limit_sharing: None,
            after_read_duration: None,
            acp2_setting: None,
            timestamp_ms: None,
        }),
        keep_in_chat: None,
    }
    .encode_to_vec()
}

/// Encode an advanced-chat-privacy `protocolMessage` (v1, `type=LIMIT_SHARING`,
/// trigger `CHAT_SETTING`, `initiatedByMe=true`). No target key: the setting
/// applies to the whole chat (chat-actions.md §5). `now_ms` is stored directly
/// in `limitSharingSettingTimestamp`; the field's unit was not independently
/// confirmed, so this follows the millisecond convention used by the other
/// sender-timestamp fields in this codebase (reactions, polls).
pub fn limit_sharing_payload(enabled: bool, now_ms: i64) -> Vec<u8> {
    Envelope {
        protocol: Some(ProtocolMessage {
            key: None,
            type_: Some(TYPE_LIMIT_SHARING),
            ephemeral_expiration: None,
            ephemeral_setting_timestamp: None,
            disappearing_mode: None,
            limit_sharing: Some(LimitSharing {
                sharing_limited: Some(enabled),
                trigger: Some(TRIGGER_CHAT_SETTING),
                limit_sharing_setting_timestamp: Some(now_ms),
                initiated_by_me: Some(true),
            }),
            after_read_duration: None,
            acp2_setting: None,
            timestamp_ms: None,
        }),
        keep_in_chat: None,
    }
    .encode_to_vec()
}

/// Encode a `keepInChatMessage` payload (field 51). `remote` is the identity
/// to place in `key.remoteJid`, following the same rule as reactions and
/// polls (the contact's LID when their messages arrive LID-addressed).
pub fn keep_payload(remote: &Jid, target: &MessageRef, keep: bool, now_ms: i64) -> Vec<u8> {
    encode_keep(remote, None, target, keep, now_ms)
}

/// Encode an ordinary group keep/unkeep with the original message author.
/// APK 37.71 Bai.AD7/CSZ.A01 and BOE.A06 serialize that author in BZa field 4;
/// 1wk selector 2 / 0Yo.A0B preserve the ordinary group remote key.
/// This codec does not enforce native expiry/keep permissions or authenticate
/// a caller's participant. The normal actor resolves retained attribution first.
pub fn group_keep_payload(
    chat: &Jid,
    participant: &Jid,
    target: &MessageRef,
    keep: bool,
    timestamp_ms: i64,
) -> Result<Vec<u8>> {
    validate_keep(chat, target)?;
    if chat.server != crate::model::jid::Server::Group {
        return Err(Error::invalid("chat_settings", "not_group_chat"));
    }
    if !participant.is_user_like() || participant.agent != 0 {
        return Err(Error::invalid("chat_settings", "unsupported_participant"));
    }
    Ok(encode_keep(
        chat,
        Some(participant),
        target,
        keep,
        timestamp_ms,
    ))
}

fn encode_keep(
    remote: &Jid,
    participant: Option<&Jid>,
    target: &MessageRef,
    keep: bool,
    now_ms: i64,
) -> Vec<u8> {
    Envelope {
        protocol: None,
        keep_in_chat: Some(KeepInChatMessage {
            key: Some(super::reactions::Key {
                remote_jid: Some(remote.to_non_ad().to_string()),
                from_me: Some(target.key.direction == Direction::Outgoing),
                id: Some(target.key.id.as_str().to_owned()),
                participant: participant.map(|p| p.to_non_ad().to_string()),
            }),
            keep_type: Some(if keep {
                KEEP_FOR_ALL
            } else {
                UNDO_KEEP_FOR_ALL
            }),
            timestamp_ms: Some(now_ms),
        }),
    }
    .encode_to_vec()
}

/// Durably queue the account-wide default disappearing timer for new chats
/// (`xmlns=disappearing_mode`, confirmed `ChangeDMSettingActivity.A0Z`,
/// chat-actions.md §3). A live session transmits it and settles the reply
/// through the shared action outbox ([`super::actions`]).
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub fn queue_default_timer(store: &crate::store::ProtocolStore, seconds: u32) -> Result<String> {
    super::actions::queue(store, "disappearing-default", "default timer", |id| {
        Ok(crate::binary::Node::new("iq")
            .attr("id", id)
            .attr("type", "set")
            .attr("xmlns", "disappearing_mode")
            .attr("to", Jid::parse("s.whatsapp.net")?)
            .children(vec![
                crate::binary::Node::new("disappearing_mode").attr("duration", seconds.to_string()),
            ]))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        ids::{ChatId, DeviceAddress, MessageId},
        message::MessageKey,
    };
    #[test]
    fn closed_control_enum_projection_matches_independent_java_oracle() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/prekeys/2.26.37.71-control-enum-oracle.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let bytes = hex::decode(case["hex"].as_str().unwrap()).unwrap();
            let actual = match case["mode"].as_str().unwrap() {
                "p" => super::super::protobuf_enum::decode_nested::<ProtocolMessage>(
                    &bytes,
                    12,
                    2,
                    protocol_type,
                )
                .map(|v| v.and_then(|(_, value)| value)),
                "k" => super::super::protobuf_enum::decode_nested::<KeepInChatMessage>(
                    &bytes,
                    51,
                    2,
                    |v| v <= 2,
                )
                .map(|v| v.and_then(|(_, value)| value)),
                _ => panic!("unsupported oracle mode"),
            };
            let expected = if case["valid"].as_bool().unwrap() {
                Ok(case["value"].as_u64().map(|v| v as u32))
            } else {
                Err(())
            };
            assert_eq!(actual, expected, "{} {}", case["mode"], case["hex"]);
        }
    }

    #[test]
    fn protocol_types_keep_last_recognized_value_and_presence() {
        let wrap = |body: &[u8]| {
            let mut out = Vec::new();
            prost::encoding::bytes::encode(12, &body.to_vec(), &mut out);
            out
        };
        for unknown in [
            1u64,
            2,
            12,
            13,
            15,
            23,
            33,
            42,
            99,
            u32::MAX.into(),
            u64::MAX,
        ] {
            let mut body = vec![0x10, 3, 0x20, 7];
            prost::encoding::uint64::encode(2, &unknown, &mut body);
            assert_eq!(
                decode_protocol(&wrap(&body)),
                Some(Setting::DisappearingTimer { seconds: 7 }),
                "unknown {unknown}"
            );
            let mut only_unknown = Vec::new();
            prost::encoding::uint64::encode(2, &unknown, &mut only_unknown);
            assert_eq!(decode_protocol(&wrap(&only_unknown)), None);
            let mut split = wrap(&[0x10, 3, 0x20, 9]);
            only_unknown.extend_from_slice(&[0x20, 7]);
            split.extend(wrap(&only_unknown));
            assert_eq!(
                decode_protocol(&split),
                Some(Setting::DisappearingTimer { seconds: 7 })
            );
        }
        for wrong_wire in [
            vec![0x12, 1, 0xff],
            vec![0x11, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0x15, 0, 0, 0, 0],
            vec![0x13, 0x10, 0, 0x14],
        ] {
            let mut body = vec![0x10, 3, 0x20, 7];
            body.extend(wrong_wire);
            assert_eq!(
                decode_protocol(&wrap(&body)),
                Some(Setting::DisappearingTimer { seconds: 7 })
            );
        }
        for known in [
            0u8, 4, 5, 14, 18, 19, 20, 21, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 34, 35, 36, 37,
            38, 39, 40, 41,
        ] {
            let body = [0x10, 3, 0x20, 7, 0x10, known];
            assert_eq!(
                decode_protocol(&wrap(&body)),
                if known == 4 {
                    Some(Setting::EphemeralSyncResponse { seconds: 7 })
                } else {
                    None
                },
                "recognized {known}"
            );
        }
        let mut body = vec![0x10, 4, 0x20, 7];
        prost::encoding::uint64::encode(2, &0x100000003, &mut body);
        assert_eq!(
            decode_protocol(&wrap(&body)),
            Some(Setting::DisappearingTimer { seconds: 7 })
        );
    }

    #[test]
    fn keep_type_ignores_unknown_numbers_without_erasing_group_parent() {
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let actor = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let target = MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(&group).unwrap(),
                sender: Some(DeviceAddress::from_jid(&actor).unwrap()),
                direction: Direction::Incoming,
                id: MessageId::new("TARGET").unwrap(),
            },
        };
        let original = group_keep_payload(&group, &actor, &target, true, 7).unwrap();
        for unknown in [3u64, 32, 99, u32::MAX.into(), u64::MAX] {
            let mut body = Vec::new();
            prost::encoding::uint64::encode(2, &unknown, &mut body);
            let mut payload = original.clone();
            prost::encoding::bytes::encode(51, &body, &mut payload);
            assert!(decode_keep(&payload).unwrap().keep, "unknown {unknown}");
            let key = group_keep_target(
                &group,
                &actor,
                &payload,
                &[Jid::parse("15555550199@s.whatsapp.net").unwrap()],
            )
            .unwrap();
            assert_eq!(key.id, target.key.id);
            assert_eq!(key.sender, target.key.sender);
        }
        for (kind, expected) in [(0, None), (1, Some(true)), (2, Some(false))] {
            let mut payload = original.clone();
            prost::encoding::bytes::encode(51, &vec![0x10, kind], &mut payload);
            assert_eq!(decode_keep(&payload).map(|v| v.keep), expected);
        }
    }

    #[test]
    fn keep_decode_requires_native_parent_key_shape_without_using_remote_as_authority() {
        for (remote, expected) in [
            (None, false),
            (Some(""), false),
            (Some("not-a-jid"), false),
            (Some("s.whatsapp.net"), false),
            (Some("12345@unknown.invalid"), false),
            (Some("12345@lid"), true),
            (Some("15555550100@s.whatsapp.net"), true),
        ] {
            let bytes = Envelope {
                keep_in_chat: Some(KeepInChatMessage {
                    key: Some(super::super::reactions::Key {
                        id: Some("TARGET".into()),
                        remote_jid: remote.map(str::to_owned),
                        from_me: Some(true),
                        participant: None,
                    }),
                    keep_type: Some(KEEP_FOR_ALL),
                    timestamp_ms: Some(7),
                }),
                ..Default::default()
            }
            .encode_to_vec();
            assert_eq!(decode_keep(&bytes).is_some(), expected, "{remote:?}");
        }
    }

    #[test]
    fn group_keep_targets_use_descriptor_keys_and_authenticated_sender_context() {
        // Independent BV3 key=1/keepType=2 inside Message field 51.
        // D9L.A01 0682..06b6 supplies this BZa key to BoQ, not its keeper.
        fn fixture(participant: Option<&str>, from_me: bool, keep_type: u8) -> Vec<u8> {
            let chat = "120363000000000001@g.us";
            let mut key = vec![0x0a, chat.len() as u8];
            key.extend_from_slice(chat.as_bytes());
            key.extend_from_slice(&[0x10, u8::from(from_me), 0x1a, 6]);
            key.extend_from_slice(b"TARGET");
            if let Some(p) = participant {
                key.extend_from_slice(&[0x22, p.len() as u8]);
                key.extend_from_slice(p.as_bytes());
            }
            let mut keep = vec![0x0a, key.len() as u8];
            keep.extend(key);
            keep.extend_from_slice(&[0x10, keep_type]);
            let mut message = vec![0x9a, 0x03, keep.len() as u8];
            message.extend(keep);
            message
        }
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let own = [
            Jid::parse("15555550199@s.whatsapp.net").unwrap(),
            Jid::parse("222222222222222@lid").unwrap(),
        ];
        let actor = Jid::parse("15555550101:3@lid").unwrap();
        let author = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        for kind in [1, 2] {
            let bytes = fixture(Some(&author.to_string()), false, kind);
            let key = group_keep_target(&chat, &actor, &bytes, &own).unwrap();
            assert_eq!(key.direction, Direction::Incoming);
            assert_eq!(key.sender.unwrap().to_jid(), author);
            assert_eq!(decode_keep(&bytes).unwrap().keep, kind == 1);
            let key = group_keep_target(&chat, &actor, &fixture(None, true, kind), &own).unwrap();
            assert_eq!(key.direction, Direction::Incoming);
            assert_eq!(key.sender.unwrap().to_jid(), actor.to_non_ad());
            for flag in [false, true] {
                let key = group_keep_target(
                    &chat,
                    &own[1].with_device(3),
                    &fixture(Some(&own[0].to_string()), flag, kind),
                    &own,
                )
                .unwrap();
                assert_eq!(
                    key.direction,
                    if flag {
                        Direction::Outgoing
                    } else {
                        Direction::Incoming
                    }
                );
            }
        }
        for kind in [0, 3] {
            assert!(
                matches!(group_keep_target(&chat,&actor,&fixture(Some(&author.to_string()),false,kind),&own),Err(Error::InvalidInput{reason,..}) if reason=="unsupported_keep_type")
            );
        }
        assert!(
            matches!(group_keep_target(&chat,&actor,&fixture(None,false,1),&own),Err(Error::InvalidInput{reason,..}) if reason=="missing_participant")
        );
        assert!(
            matches!(group_keep_target(&chat,&actor,&fixture(Some(&author.to_string()),false,1),&[]),Err(Error::InvalidInput{reason,..}) if reason=="own_identity_missing")
        );
    }

    fn target(direction: Direction) -> MessageRef {
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        MessageRef {
            key: MessageKey {
                chat: ChatId::from_jid(&peer).unwrap(),
                sender: (direction == Direction::Incoming)
                    .then(|| DeviceAddress::from_jid(&peer).unwrap()),
                direction,
                id: MessageId::new("ABCDEF0123456789ABCDEF0123456789").unwrap(),
            },
        }
    }

    #[test]
    fn disappearing_mode_device_string_and_initiated_flag_use_current_descriptor_tags() {
        let mut mode = Vec::new();
        prost::encoding::string::encode(3, &"12345:4@lid".to_owned(), &mut mode);
        prost::encoding::bool::encode(4, &true, &mut mode);
        let mut protocol = Vec::new();
        prost::encoding::int32::encode(2, &3, &mut protocol);
        prost::encoding::uint32::encode(4, &86400, &mut protocol);
        prost::encoding::bytes::encode(11, &mode, &mut protocol);
        let mut bytes = Vec::new();
        prost::encoding::bytes::encode(12, &protocol, &mut bytes);
        assert_eq!(
            decode_protocol(&bytes),
            Some(Setting::DisappearingTimer { seconds: 86400 })
        );
        let mode = Envelope::decode(bytes.as_slice())
            .unwrap()
            .protocol
            .unwrap()
            .disappearing_mode
            .unwrap();
        assert_eq!(mode.initiator_device_jid.as_deref(), Some("12345:4@lid"));
        assert_eq!(mode.initiated_by_me, Some(true));
    }

    #[test]
    fn timer_decode_matches_native_signed_duration_and_does_not_flatten_after_read() {
        for kind in [TYPE_EPHEMERAL_SETTING, TYPE_EPHEMERAL_SYNC_RESPONSE] {
            for (raw, effective) in [
                (0, 0),
                (1, 1),
                (i32::MAX as u32, i32::MAX as u32),
                (0x80000000, 0),
                (u32::MAX, 0),
            ] {
                let bytes = Envelope {
                    protocol: Some(ProtocolMessage {
                        type_: Some(kind),
                        ephemeral_expiration: Some(raw),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
                .encode_to_vec();
                assert_eq!(
                    decode_protocol(&bytes),
                    Some(if kind == TYPE_EPHEMERAL_SETTING {
                        Setting::DisappearingTimer { seconds: effective }
                    } else {
                        Setting::EphemeralSyncResponse { seconds: effective }
                    })
                );
            }
            let bytes = Envelope {
                protocol: Some(ProtocolMessage {
                    type_: Some(kind),
                    ephemeral_expiration: Some(86400),
                    after_read_duration: Some(60),
                    ..Default::default()
                }),
                ..Default::default()
            }
            .encode_to_vec();
            assert!(decode_protocol(&bytes).is_none());
        }
    }

    #[test]
    fn disappearing_payload_roundtrips_with_apk_field_numbers() {
        let bytes = disappearing_payload(604_800);
        // Field 12, wire type 2: (12 << 3) | 2 = 98 = 0x62.
        assert_eq!(bytes[0], 0x62);
        let setting = decode_protocol(&bytes).unwrap();
        assert_eq!(setting, Setting::DisappearingTimer { seconds: 604_800 });
        let off = decode_protocol(&disappearing_payload(0)).unwrap();
        assert_eq!(off, Setting::DisappearingTimer { seconds: 0 });
        for seconds in UI_TIMER_VALUES {
            assert_eq!(
                decode_protocol(&disappearing_payload(seconds)),
                Some(Setting::DisappearingTimer { seconds })
            );
        }
    }
    #[test]
    fn ephemeral_sync_response_decodes_as_its_own_variant() {
        // ProtocolMessage{type=4 (EPHEMERAL_SYNC_RESPONSE), ephemeralExpiration=86400}.
        #[derive(prost::Message)]
        struct Fixture {
            #[prost(int32, optional, tag = "2")]
            type_: Option<i32>,
            #[prost(uint32, optional, tag = "4")]
            ephemeral_expiration: Option<u32>,
        }
        let inner = Fixture {
            type_: Some(TYPE_EPHEMERAL_SYNC_RESPONSE),
            ephemeral_expiration: Some(86_400),
        }
        .encode_to_vec();
        #[derive(prost::Message)]
        struct Outer {
            #[prost(bytes = "vec", optional, tag = "12")]
            protocol: Option<Vec<u8>>,
        }
        let bytes = Outer {
            protocol: Some(inner),
        }
        .encode_to_vec();
        assert_eq!(
            decode_protocol(&bytes),
            Some(Setting::EphemeralSyncResponse { seconds: 86_400 })
        );
    }
    #[test]
    fn limit_sharing_payload_roundtrips_v1_and_decodes_v2() {
        let bytes = limit_sharing_payload(true, 1_700_000_000_000);
        assert_eq!(bytes[0], 0x62);
        assert_eq!(
            decode_protocol(&bytes),
            Some(Setting::LimitSharing { enabled: true })
        );
        let off = limit_sharing_payload(false, 1);
        assert_eq!(
            decode_protocol(&off),
            Some(Setting::LimitSharing { enabled: false })
        );
        // v2 (ACP2_SETTING) decodes too, even though this module only sends v1.
        #[derive(prost::Message)]
        struct Acp2Fixture {
            #[prost(bool, optional, tag = "1")]
            enabled: Option<bool>,
        }
        #[derive(prost::Message)]
        struct ProtocolFixture {
            #[prost(int32, optional, tag = "2")]
            type_: Option<i32>,
            #[prost(message, optional, tag = "35")]
            acp2_setting: Option<Acp2Fixture>,
        }
        #[derive(prost::Message)]
        struct Outer {
            #[prost(message, optional, tag = "12")]
            protocol: Option<ProtocolFixture>,
        }
        let bytes = Outer {
            protocol: Some(ProtocolFixture {
                type_: Some(TYPE_ACP2_SETTING),
                acp2_setting: Some(Acp2Fixture {
                    enabled: Some(true),
                }),
            }),
        }
        .encode_to_vec();
        assert_eq!(
            decode_protocol(&bytes),
            Some(Setting::Acp2 { enabled: true })
        );
    }
    #[test]
    fn keep_payload_roundtrips_with_apk_field_numbers_and_sender_relative_from_me() {
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let bytes = keep_payload(&peer, &target(Direction::Incoming), true, 1_700_000_000_000);
        // Field 51, wire type 2: (51 << 3) | 2 = 410 = varint 9A 03.
        assert_eq!(&bytes[..2], &[0x9A, 0x03]);
        let decoded = decode_keep(&bytes).unwrap();
        assert_eq!(
            decoded,
            IncomingKeep {
                target_id: MessageId::new("ABCDEF0123456789ABCDEF0123456789").unwrap(),
                target_from_sender: false,
                keep: true,
                sender_timestamp: Some(crate::model::time::Timestamp(1_700_000_000_000)),
            }
        );
        let undo = keep_payload(&peer, &target(Direction::Outgoing), false, 1);
        let decoded = decode_keep(&undo).unwrap();
        assert!(!decoded.keep);
        assert!(decoded.target_from_sender);
        assert!(decode_keep(&[0x08, 0x01]).is_none());
    }
    #[test]
    fn group_keep_matches_independent_descriptor_bytes_and_preserves_direct_encoding() {
        let chat = Jid::parse("123@g.us").unwrap();
        let author = Jid::parse("222:3@lid").unwrap();
        let mut target = target(Direction::Incoming);
        target.key.chat = ChatId::from_jid(&chat).unwrap();
        target.key.id = MessageId::new("abc").unwrap();
        // Literal APK 37.71 BZc field 51 / BV3 1,2,3 / BZa 1,2,3,4
        // fixture: group=123@g.us, author=222@lid, id=abc, timestamp=1.
        let kept =
            hex::decode("9a03200a1a0a0831323340672e757310001a036162632207323232406c696410011801")
                .unwrap();
        assert_eq!(
            group_keep_payload(&chat, &author, &target, true, 1).unwrap(),
            kept
        );
        target.key.direction = Direction::Outgoing;
        let undo =
            hex::decode("9a03200a1a0a0831323340672e757310011a036162632207323232406c696410021801")
                .unwrap();
        assert_eq!(
            group_keep_payload(&chat, &author, &target, false, 1).unwrap(),
            undo
        );
        assert!(group_keep_payload(&chat, &chat, &target, true, 1).is_err());
        let foreign = Jid::parse("456@g.us").unwrap();
        assert!(group_keep_payload(&foreign, &author, &target, true, 1).is_err());
        assert!(group_keep_payload(&author, &author, &target, true, 1).is_err());
        let agent = Jid::new_ad("222", 2, 3, crate::model::jid::Server::Lid).unwrap();
        assert!(group_keep_payload(&chat, &agent, &target, true, 1).is_err());
        let direct = Jid::parse("222@lid").unwrap();
        target.key.chat = ChatId::from_jid(&direct).unwrap();
        // The existing direct encoder still omits participant; no identity or
        // queue migration is implied by adding the separate group codec.
        assert_eq!(
            keep_payload(&direct, &target, false, 1),
            hex::decode("9a03160a100a07323232406c696410011a0361626310021801").unwrap()
        );
    }
    #[test]
    fn keep_timestamp_preserves_wire_presence_zero_and_signed_values() {
        // Independent BZc.51/BV3.1,2,3 fixture: target abc, keep type 1.
        // No timestamp field must not acquire stanza/observation/current time.
        let prefix = hex::decode("9a03120a0e0a07323232406c69641a036162631001").unwrap();
        assert_eq!(decode_keep(&prefix).unwrap().sender_timestamp, None);
        for (literal, expected) in [
            ("9a03140a0e0a07323232406c69641a0361626310011800", 0),
            ("9a03140a0e0a07323232406c69641a036162631001182a", 42),
            (
                "9a031d0a0e0a07323232406c69641a03616263100118ffffffffffffffffff01",
                -1,
            ),
            (
                "9a031c0a0e0a07323232406c69641a03616263100118ffffffffffffffff7f",
                i64::MAX,
            ),
        ] {
            assert_eq!(
                decode_keep(&hex::decode(literal).unwrap())
                    .unwrap()
                    .sender_timestamp,
                Some(crate::model::time::Timestamp(expected))
            );
        }
        assert!(
            decode_keep(&hex::decode("9a03140a0e0a07323232406c69641a0361626310011880").unwrap())
                .is_none()
        );
    }
    #[test]
    fn validation_rejects_other_chats() {
        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let other = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        assert!(validate_keep(&peer, &target(Direction::Incoming)).is_ok());
        assert!(validate_keep(&other, &target(Direction::Incoming)).is_err());
    }
    #[cfg(all(
        feature = "experimental-signal-crypto",
        feature = "experimental-noise",
        feature = "experimental-transport"
    ))]
    #[test]
    fn default_timer_iq_shape() {
        use crate::store::{MasterKey, OpenOptions, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let store = crate::store::ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: crate::model::ids::AccountId::new("a").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([1; 32]))),
            create: true,
        })
        .unwrap();
        let id = queue_default_timer(&store, 604_800).unwrap();
        let pending = super::super::actions::reserve_pending(&store).unwrap();
        assert_eq!(pending.len(), 1);
        let node = &pending[0];
        assert_eq!(node.description, "iq");
        assert_eq!(
            node.get_attr("id").and_then(|v| v.as_text()),
            Some(id.as_str())
        );
        assert_eq!(node.get_attr("type").and_then(|v| v.as_text()), Some("set"));
        assert_eq!(
            node.get_attr("xmlns").and_then(|v| v.as_text()),
            Some("disappearing_mode")
        );
        let mode = node.child("disappearing_mode").unwrap();
        assert_eq!(
            mode.get_attr("duration").and_then(|v| v.as_text()),
            Some("604800")
        );
    }
}
