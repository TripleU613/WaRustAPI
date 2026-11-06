//! Bounded comparison messaging operations; no implicit retries or account resets.
use super::{TransportError, handshake::EncryptedConnection};
use crate::{
    binary::{Decoder, Encoder, Node, TokenDictionary, unpack_frame_payload},
    model::jid::Jid,
    profile::Limits,
};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};

/// Private editable one-message intent, independent of registration identity.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageRequest {
    /// Recipient E.164 digits (optional leading plus). Absent for a group
    /// send, which names its chat in `group` instead.
    #[serde(default)]
    pub to: zeroize::Zeroizing<String>,
    /// Plaintext, never printed or stored outside encrypted state.
    #[serde(default)]
    pub text: zeroize::Zeroizing<String>,
    /// Local PNG file for an image message, mutually exclusive with text.
    #[serde(default)]
    pub image: Option<std::path::PathBuf>,
    /// Ogg/Opus voice note, mutually exclusive with text and image.
    #[serde(default)]
    pub voice_note: Option<std::path::PathBuf>,
    /// A document file to upload and send (`DocumentMessage`, Message field 7).
    #[serde(default)]
    pub document: Option<std::path::PathBuf>,
    /// An MP4 video to upload and send (`VideoMessage`, Message field 9).
    #[serde(default)]
    pub video: Option<std::path::PathBuf>,
    /// Send an image with WhatsApp's view-once marker and V2 wrapper.
    #[serde(default)]
    pub view_once: bool,
    /// Group chat JID (`…@g.us`) for a sender-key send. Mutually exclusive
    /// with `to`: a group message is fanned out to the members' devices, not
    /// addressed to one phone number.
    #[serde(default)]
    pub group: Option<String>,
    /// Discard our saved *session* with each recipient device and establish a
    /// new one from a fresh prekey bundle, keeping the pinned peer identity.
    ///
    /// Only legitimate with positive evidence that the peer never received the
    /// distribution we encrypted under the old session — a session we created
    /// but whose ciphertext never reached the peer leaves us emitting `msg`
    /// against a ratchet they never established, which can never decrypt. It
    /// resets no identity and no sender key, and a changed peer identity is
    /// still refused.
    #[serde(default)]
    pub fresh_sessions: bool,
    /// How a group send distributes its sender key: `participants` (the
    /// default — one `<enc>` per device inside the group stanza's
    /// `<participants>`) or `direct` (one ordinary direct message per device,
    /// carrying the distribution, sent before the group stanza).
    ///
    /// Both are real: `includeSenderKeysInMessage` is a per-send flag in
    /// `SendE2EMessageJob`, so the client does not always put sender keys in the
    /// message, and `SendLiveLocationKeyJob` distributes its keys entirely out
    /// of band. `direct` reuses the 1:1 path, which is live-verified.
    #[serde(default)]
    pub skdm_delivery: Option<String>,
    /// Post a status instead of sending to a chat. Mutually exclusive with
    /// `to` and `group`.
    #[serde(default)]
    pub status: Option<StatusPost>,
    /// Reply to, or react to, an existing message in the `group` named above.
    /// A reaction needs no `text`; a reply uses it as the reply body.
    #[serde(default)]
    pub quote: Option<QuotedTarget>,
    /// Stable local idempotency key; reuse it after uncertain outcomes.
    pub request_id: String,
}
/// An existing message in a group, to reply to or react to.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct QuotedTarget {
    /// Wire id of the message being replied to or reacted to.
    pub message_id: String,
    /// Who sent it. A group message id means nothing without this — the same id
    /// from two members is two different messages.
    pub participant: String,
    /// React with this emoji instead of replying. An empty string removes a
    /// previous reaction, which is a real operation and not the same as absent.
    #[serde(default)]
    pub emoji: Option<String>,
    /// The quoted message's own text, so the reply can carry it as
    /// `quotedMessage`. Absent quotes an empty body rather than inventing one.
    #[serde(default)]
    pub quoted_text: Option<zeroize::Zeroizing<String>>,
}

impl std::fmt::Debug for QuotedTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("QuotedTarget(<redacted>)")
    }
}

impl QuotedTarget {
    /// The sender of the quoted message.
    pub fn sender(&self) -> crate::Result<Jid> {
        let jid = Jid::parse(&self.participant)?;
        if !jid.is_user_like() {
            return Err(crate::Error::invalid(
                "quote",
                "participant is a phone-number or LID identity",
            ));
        }
        Ok(jid)
    }

    /// Whether this is a reaction rather than a reply.
    pub fn is_reaction(&self) -> bool {
        self.emoji.is_some()
    }
}

/// A status post: the text, who may see it, and how it is drawn.
///
/// The audience is explicit because a status has no membership list to read and
/// this client does not sync the account's status privacy list. Naming it is
/// WhatsApp's "share with only…", and it keeps the client from inferring who is
/// entitled to see something.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusPost {
    /// Status text.
    pub text: zeroize::Zeroizing<String>,
    /// Identities allowed to see it (phone-number or LID form).
    pub audience: Vec<String>,
    /// Font, `0..=8` (`LX/C44`); absent leaves the default.
    #[serde(default)]
    pub font: Option<u8>,
    /// Background colour as ARGB hex, with or without a leading `#`.
    #[serde(default)]
    pub background: Option<String>,
    /// Text colour as ARGB hex, with or without a leading `#`.
    #[serde(default)]
    pub text_color: Option<String>,
}

impl std::fmt::Debug for StatusPost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StatusPost(<redacted>)")
    }
}

impl StatusPost {
    /// The audience as parsed identities.
    pub fn recipients(&self) -> crate::Result<Vec<Jid>> {
        if self.audience.is_empty() {
            return Err(crate::Error::invalid(
                "status",
                "a status needs at least one audience identity",
            ));
        }
        self.audience
            .iter()
            .map(|raw| {
                let jid = Jid::parse(raw)?;
                if !jid.is_user_like() || jid.device != 0 {
                    return Err(crate::Error::invalid(
                        "status",
                        "an audience entry is a phone-number or LID identity",
                    ));
                }
                Ok(jid.to_non_ad())
            })
            .collect()
    }

    /// Colour and font, refusing anything this build does not enumerate rather
    /// than drawing a status some other way than asked.
    pub fn style(&self) -> crate::Result<crate::transport::quotes::TextStyle> {
        let argb = |value: &Option<String>| -> crate::Result<Option<u32>> {
            let Some(raw) = value.as_deref() else {
                return Ok(None);
            };
            let hex = raw.strip_prefix('#').unwrap_or(raw);
            u32::from_str_radix(hex, 16)
                .map(Some)
                .map_err(|_| crate::Error::invalid("status", "a colour is ARGB hex"))
        };
        Ok(crate::transport::quotes::TextStyle {
            background_argb: argb(&self.background)?,
            text_argb: argb(&self.text_color)?,
            font: self
                .font
                .map(|f| {
                    crate::transport::quotes::StatusFont::from_wire(i32::from(f))
                        .ok_or_else(|| crate::Error::invalid("status", "font is 0..=8"))
                })
                .transpose()?,
        })
    }
}

impl std::fmt::Debug for MessageRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MessageRequest(<redacted>)")
    }
}
impl MessageRequest {
    /// Read private bounded JSON without exposing parse values.
    pub fn read(path: &std::path::Path) -> crate::Result<Self> {
        use std::io::Read;
        let file = std::fs::File::open(path)?;
        let metadata = file.metadata()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(crate::Error::invalid(
                    "message",
                    "JSON must have private permissions",
                ));
            }
        }
        if !metadata.is_file() || metadata.len() > 65536 {
            return Err(crate::Error::invalid(
                "message",
                "invalid JSON file size/type",
            ));
        }
        let mut bytes = zeroize::Zeroizing::new(Vec::new());
        file.take(65537).read_to_end(&mut bytes)?;
        if bytes.len() > 65536 {
            return Err(crate::Error::invalid("message", "JSON too large"));
        }
        let value: Self = serde_json::from_slice(&bytes)
            .map_err(|_| crate::Error::invalid("message", "invalid message JSON"))?;
        if let Some(status) = &value.status {
            if !value.to.is_empty() || value.group.is_some() {
                return Err(crate::Error::invalid(
                    "message",
                    "a status post names no `to` and no `group`",
                ));
            }
            if value.voice_note.is_some()
                || value.document.is_some()
                || value.video.is_some()
                || value.view_once
            {
                return Err(crate::Error::invalid(
                    "message",
                    "a status post carries text or an image so far",
                ));
            }
            status.recipients()?;
            status.style()?;
            // An image status replaces the text rather than captioning it;
            // carrying both would silently drop one of them.
            if value.image.is_some() {
                if !status.text.is_empty() {
                    return Err(crate::Error::invalid(
                        "message",
                        "an image status carries no text",
                    ));
                }
            } else if status.text.is_empty() || status.text.len() > 4096 {
                return Err(crate::Error::invalid(
                    "message",
                    "a status text is 1..=4096 bytes",
                ));
            }
            if value.request_id.is_empty() || value.request_id.len() > 64 {
                return Err(crate::Error::invalid("message", "invalid request_id"));
            }
            return Ok(value);
        }
        if let Some(quote) = &value.quote {
            // A direct quote is valid JSON for outbox-send replies; the
            // comparison path refuses it (`require_comparison_support`).
            quote.sender()?;
            if quote.message_id.is_empty() || quote.message_id.len() > 128 {
                return Err(crate::Error::invalid(
                    "message",
                    "invalid quoted message id",
                ));
            }
            // A reaction has no body; a reply has exactly one: text, an image
            // or a voice note (outbox-send sends media replies; the comparison
            // group path refuses media separately).
            let bodies = usize::from(!value.text.is_empty())
                + usize::from(value.image.is_some())
                + usize::from(value.voice_note.is_some())
                + usize::from(value.document.is_some())
                + usize::from(value.video.is_some());
            if quote.is_reaction() && bodies != 0 {
                return Err(crate::Error::invalid(
                    "message",
                    "a reaction carries no text, image or voice note",
                ));
            }
            if !quote.is_reaction() && bodies != 1 {
                return Err(crate::Error::invalid(
                    "message",
                    "a reply carries exactly one of text, image or voice_note",
                ));
            }
        }
        match value.group_chat()? {
            Some(_) if !value.to.is_empty() => {
                return Err(crate::Error::invalid(
                    "message",
                    "set either to or group, not both",
                ));
            }
            // Group media is valid JSON for outbox-send; the comparison path
            // refuses it separately (`require_comparison_support`).
            Some(_) => {}
            None => {
                value.recipient()?;
            }
        }
        // A reaction is the one message with no content of its own: it names an
        // existing message and an emoji, and a body would make it a reply.
        let reacting = value.quote.as_ref().is_some_and(|q| q.is_reaction());
        let bodies = usize::from(!value.text.is_empty())
            + usize::from(value.image.is_some())
            + usize::from(value.voice_note.is_some())
            + usize::from(value.document.is_some())
            + usize::from(value.video.is_some());
        if (bodies != usize::from(!reacting))
            || (value.view_once && value.image.is_none())
            || value.text.len() > 4096
            || value.request_id.is_empty()
            || value.request_id.len() > 64
            || !value
                .request_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(crate::Error::invalid(
                "message",
                "invalid text or request_id bounds",
            ));
        }
        Ok(value)
    }
    /// What the comparison (`lab-transport`) path cannot send: group media
    /// (its group path sends text, replies and reactions only) and any quote
    /// in a direct chat (its direct path has no quote encoding). Both go
    /// through `outbox-send`; refusing here keeps the comparison path from
    /// sending an empty text or dropping the quote.
    pub fn require_comparison_support(&self) -> crate::Result<()> {
        let group = self.group_chat()?.is_some();
        if group
            && (self.image.is_some()
                || self.voice_note.is_some()
                || self.document.is_some()
                || self.video.is_some()
                || self.view_once)
        {
            return Err(crate::Error::invalid(
                "message",
                "a comparison group send carries text only; use outbox-send for group media",
            ));
        }
        if !group && self.quote.is_some() {
            return Err(crate::Error::invalid(
                "message",
                "the comparison direct path has no quotes; use outbox-send for direct replies",
            ));
        }
        Ok(())
    }
    /// Whether a group send puts its distribution message in `<participants>`
    /// or delivers it as separate direct messages.
    pub fn skdm_out_of_band(&self) -> crate::Result<bool> {
        match self.skdm_delivery.as_deref() {
            None | Some("") | Some("participants") => Ok(false),
            Some("direct") => Ok(true),
            Some(_) => Err(crate::Error::invalid(
                "message",
                "skdm_delivery is `participants` or `direct`",
            )),
        }
    }

    /// The group chat this message targets, when it is a group send.
    ///
    /// Validated the same way a group management request is: a `g.us` JID with
    /// no device suffix. A status post would be `status@broadcast`, which this
    /// deliberately does not yet accept — posting is a separate slice.
    pub fn group_chat(&self) -> crate::Result<Option<Jid>> {
        let Some(raw) = self.group.as_deref().filter(|v| !v.is_empty()) else {
            return Ok(None);
        };
        let jid = Jid::parse(raw)?;
        jid.require_group_id()?;
        Ok(Some(jid))
    }

    /// The direct recipient: E.164 digits, or a full `…@s.whatsapp.net` /
    /// `…@lid` JID.
    ///
    /// Authenticated original captures establish LID Signal addressing in this
    /// client (docs/protocol/groups.md). The older CPQ.A00 citation is not a
    /// 2.26.37.71 address producer: that build's method is a cached policy helper.
    /// The LID form makes direct addressing testable independently of fan-out.
    pub fn recipient(&self) -> crate::Result<Jid> {
        if self.to.contains('@') {
            let jid = Jid::parse(&self.to)?;
            if !matches!(
                jid.server,
                crate::model::jid::Server::User | crate::model::jid::Server::Lid
            ) || jid.device != 0
            {
                return Err(crate::Error::invalid(
                    "message",
                    "a direct recipient is a phone or LID identity with no device suffix",
                ));
            }
            return Ok(jid);
        }
        let digits = self.to.strip_prefix('+').unwrap_or(&self.to);
        if !(5..=15).contains(&digits.len())
            || digits.starts_with('0')
            || !digits.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(crate::Error::invalid(
                "message",
                "recipient must be E.164 digits or a phone/LID JID",
            ));
        }
        Jid::parse(&format!("{digits}@s.whatsapp.net"))
    }
}

fn error(reason: &'static str) -> TransportError {
    TransportError::Handshake(reason.into())
}

/// Fresh opaque stanza identifier, separate from the stable local request ID.
/// Random hexadecimal syntax is an experimental choice, not a device claim.
pub fn new_message_id() -> crate::Result<String> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| crate::Error::invalid("message", "message-id entropy unavailable"))?;
    Ok(hex::encode_upper(bytes))
}

/// Read one bounded encrypted binary stanza with the comparison dictionary.
pub async fn read_node<S: AsyncRead + AsyncWrite + Unpin>(
    channel: &mut EncryptedConnection<S>,
) -> Result<Node, TransportError> {
    let frame = channel
        .read()
        .await?
        .ok_or_else(|| error("EOF awaiting stanza"))?;
    let limits = Limits::default();
    let data =
        unpack_frame_payload(&frame, &limits).map_err(|_| error("invalid stanza compression"))?;
    Decoder::new(&data, TokenDictionary::comparison_2_26_37_71(), &limits)
        .decode_node()
        .map_err(|_| error("invalid comparison stanza"))
}

/// One frame read in a lab window.
pub enum FrameRead {
    /// A decoded stanza.
    Node(Node),
    /// A frame that decrypted, so the stream is intact, but did not unpack or
    /// decode; for [`super::inbox::preserve_undecodable`].
    Undecodable {
        /// The decrypted frame.
        frame: Vec<u8>,
        /// Fixed error class (`CodecError::class`).
        class: &'static str,
    },
}

impl std::fmt::Debug for FrameRead {
    /// Redacted: the stanza's name, or the undecodable frame's size and class.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(node) => f.debug_tuple("Node").field(&node.description).finish(),
            Self::Undecodable { frame, class } => f
                .debug_struct("Undecodable")
                .field("bytes", &frame.len())
                .field("class", class)
                .finish(),
        }
    }
}

/// Like [`read_node`], but a frame that decrypts and does not decode is
/// returned for capture instead of ending the caller's window, as `listen`
/// does. Login and handshake keep [`read_node`]: their reply must decode.
pub async fn read_frame<S: AsyncRead + AsyncWrite + Unpin>(
    channel: &mut EncryptedConnection<S>,
) -> Result<FrameRead, TransportError> {
    let frame = channel
        .read()
        .await?
        .ok_or_else(|| error("EOF awaiting stanza"))?;
    Ok(match super::inbox::decode_frame(&frame) {
        Ok(node) => FrameRead::Node(node),
        Err(codec) => FrameRead::Undecodable {
            frame: frame.to_vec(),
            class: codec.class(),
        },
    })
}

/// Write one uncompressed encrypted stanza.
pub async fn write_node<S: AsyncRead + AsyncWrite + Unpin>(
    channel: &mut EncryptedConnection<S>,
    node: &Node,
) -> Result<(), TransportError> {
    let data = Encoder::new(TokenDictionary::comparison_2_26_37_71())
        .encode(node)
        .map_err(|_| error("cannot encode stanza"))?;
    channel.write(&data).await
}

/// APK 2.26.37.71 0y0.A00 1786..193a: iq/get/encrypt -> key -> user/jid.
/// Source: group-ack-writer-dispatch in the versioned groups research bundle.
pub fn key_request(recipient: &Jid, id: &str) -> Node {
    key_request_many(std::slice::from_ref(recipient), id).expect("one recipient is always in range")
}

/// Local maximum number of devices in one directory query; not a server limit.
pub const KEY_QUERY_MAX_DEVICES: usize = 32;

/// The same `encrypt` key query for several recipient devices at once.
///
/// The `<key>` element holds one `<user jid=…>` per recipient; a group fan-out
/// needs a bundle for every device it has no session with, and one query per
/// device would be several round trips inside a single bounded connection.
/// The reply's `<list>` carries one `<user>` per requested JID, which
/// [`crate::signal::direct::prekey_bundle_for`] selects from.
///
/// The bound is local: the APK batches these but the exact server limit is
/// **unknown**, so this refuses an unreasonably wide query rather than
/// discovering the limit against a live account.
pub fn key_request_many(recipients: &[Jid], id: &str) -> crate::Result<Node> {
    key_request_many_with_identity_changes(recipients, &[], id)
}

/// Build the legacy directory query, marking exactly the affected devices.
/// APK 37.71 0y0.A00 1792..17da sets user/@reason=identity for membership
/// in identityJids. This flag requests a refresh; it never authorizes pin adoption.
/// PQ/context attributes and the native callback remain separate unported work.
pub fn key_request_many_with_identity_changes(
    recipients: &[Jid],
    changed: &[Jid],
    id: &str,
) -> crate::Result<Node> {
    if recipients.is_empty() || recipients.len() > KEY_QUERY_MAX_DEVICES {
        return Err(crate::Error::invalid(
            "key_request",
            "a key query covers 1..=32 recipient devices",
        ));
    }
    // Exact components keep companion devices and PN/LID namespaces distinct.
    // Encoding-kind equivalence is our existing directory correlation policy,
    // not a complete reconstruction of Java DeviceJid.equals.
    let same = super::identity_query::same_primary_address;
    if changed.len() > KEY_QUERY_MAX_DEVICES
        || changed
            .iter()
            .any(|peer| !recipients.iter().any(|v| same(v, peer)))
    {
        return Err(crate::Error::invalid(
            "key_request",
            "identity-change devices must belong to the requested device set",
        ));
    }
    Ok(Node::new("iq")
        .attr("id", id)
        .attr("xmlns", "encrypt")
        .attr("type", "get")
        .attr(
            "to",
            Jid::parse("s.whatsapp.net").expect("constant server JID"),
        )
        .children(vec![
            Node::new("key").children(
                recipients
                    .iter()
                    .map(|jid| {
                        let node = Node::new("user").attr("jid", jid.clone());
                        if changed.iter().any(|peer| same(jid, peer)) {
                            node.attr("reason", "identity")
                        } else {
                            node
                        }
                    })
                    .collect(),
            ),
        ]))
}

/// Ask a sender to send a message again, because we could not decrypt it.
///
/// This is the mirror of the retry receipts we answer. Its shape is taken from
/// the ones the operator's handset sends us (**observed** 2026-09-24):
///
/// ```text
/// <receipt to=<chat> id=<message id> [participant=<sender device>] type="retry">
///   <retry count="1" id=<message id> t=<seconds> v="1"/>
///   <registration>…</registration>
/// </receipt>
/// ```
///
/// `count` is which attempt this is, starting at 1. `<registration>` carries our
/// own registration id so the sender can tell whether their saved session for us
/// is still current — that is the field that lets them decide between reusing a
/// session and fetching a fresh bundle.
///
/// `message_seconds` is the failed message's original timestamp, not the local
/// request time (APK43.72 F3e.A08 / SendRetryReceiptJob).
///
/// Sending one asks for a repeat of a message we already hold; it acknowledges
/// nothing and confirms no delivery.
pub fn retry_receipt(
    chat: &Jid,
    participant: Option<&Jid>,
    message_id: &str,
    count: u32,
    registration_id: u32,
    message_seconds: i64,
) -> crate::Result<Node> {
    retry_receipt_with_error(
        chat,
        participant,
        message_id,
        count,
        registration_id,
        message_seconds,
        None,
    )
}

pub(crate) fn retry_receipt_with_error(
    chat: &Jid,
    participant: Option<&Jid>,
    message_id: &str,
    count: u32,
    registration_id: u32,
    message_seconds: i64,
    error: Option<u32>,
) -> crate::Result<Node> {
    if message_id.is_empty() || message_id.len() > 128 {
        return Err(crate::Error::invalid("receipt", "invalid message id"));
    }
    if count == 0 || count > 16 {
        return Err(crate::Error::invalid(
            "receipt",
            "a retry count starts at 1 and stays bounded",
        ));
    }
    let mut node = Node::new("receipt")
        .attr("to", chat.clone())
        .attr("id", message_id)
        .attr("type", "retry");
    if let Some(participant) = participant {
        node = node.attr("participant", participant.clone());
    }
    let mut retry = Node::new("retry")
        .attr("count", count.to_string().as_str())
        .attr("id", message_id)
        .attr("t", message_seconds.to_string().as_str())
        .attr("v", "1");
    if let Some(error) = error {
        retry = retry.attr("error", error.to_string());
    }
    Ok(node.children(vec![
        retry,
        Node::new("registration").bytes(registration_id.to_be_bytes().to_vec()),
    ]))
}

/// Comparison LX/Avw.A00, called with true by primary ConnectionThread
/// LX/0iS.A0e: activate this authenticated connection, not a read receipt.
pub fn active_request(id: &str) -> Node {
    Node::new("iq")
        .attr("id", id)
        .attr("xmlns", "passive")
        .attr("type", "set")
        .attr("to", Jid::parse("s.whatsapp.net").expect("constant JID"))
        .children(vec![Node::new("active")])
}

/// Comparison 2.26.37.71 `LX/1Rd->A01` (**confirmed**): announce this client as
/// available. One `presence` element carrying only `type`, whose value is
/// validated against the set `{available, unavailable}`; no `to` and no
/// children.
///
/// Current-build evidence (2.26.43.72 `LX/1rj->A01` and `LX/0ZJ->A0z`)
/// confirms this shape and a conditional startup call, not universal availability
/// on every connection. See `docs/protocol/apk/2.26.43.72-startup-availability/`.
/// This builder does not select startup policy or establish server ACK behavior;
/// it supplies no IQ request ID for correlation.
pub fn available_presence() -> Node {
    Node::new("presence").attr("type", "available")
}

/// Remote delivery receipt for a direct or group message, or a receipt
/// acknowledgement. Never a read/played/opened receipt. Caller must durably
/// store the incoming frame and authenticate messages before acknowledging them.
///
/// Group form from comparison 2.26.37.71 `LX/Cp2->A00` (**confirmed**): `to` and
/// `id` always; `participant` exactly when it is non-null and differs from `to`
/// (so a direct chat, where the two coincide, never carries it); `type` omitted
/// for a plain delivery receipt; `recipient` and `sts` only when their inputs are
/// set, which they are not here.
///
/// APK Cp2.A04/A06/A07 supply a DeviceJid to CmC.A06 for the sender route.
/// Existing companions retain their device; primary PN devices use JID_PAIR,
/// while primary LID devices use AD (0e1.getAgent=1; 1Qe.A00).
pub fn received_confirmation(incoming: &Node) -> Option<Node> {
    if incoming.description == "receipt" {
        return receipt_ack(incoming);
    }
    if incoming.description != "message" || incoming.get_attr("recipient").is_some() {
        return None;
    }
    // APK 0DK.A01 -> 0D1.A00 creates PN/LID device classes; 1Qe.A00 selects their wire form.
    let device_form = |mut jid: Jid| {
        use crate::model::jid::{JidKind, Server};
        if jid.agent == 0 && jid.device == 0 {
            match jid.server {
                Server::User => jid.kind = JidKind::Plain,
                Server::Lid => jid.kind = JidKind::AgentDevice,
                _ => {}
            }
        }
        jid
    };
    // APK 37.71 Cp2.A06 passes signed -1 to A00, so sts (>0) is omitted.
    // Remote statuses carry class=status and mode=0; sender requires own-account context.
    // APK 1RC.A06 routes ordinary lists to Bp2; Cp2.A04 supplies no class.
    if incoming.description == "message"
        && let Some((chat, participant)) = super::inbox::sender_key_chat(incoming)
        && chat.server == crate::model::jid::Server::Broadcast
    {
        if incoming.get_attr("edit").is_some() {
            return None;
        }
        let id = incoming.get_attr("id")?.as_text()?;
        if id.is_empty() || id.len() > 128 {
            return None;
        }
        let status = incoming
            .get_attr("category")
            .and_then(|value| value.as_text())
            != Some("peer")
            && (chat.user == "status"
                || incoming.child("meta").is_some_and(|meta| {
                    meta.get_attr("is_group_status")
                        .and_then(|value| value.as_text())
                        == Some("true")
                        || meta
                            .get_attr("content_type")
                            .and_then(|value| value.as_text())
                            == Some("status_notify")
                }));
        let receipt = Node::new("receipt")
            .attr("to", chat)
            .attr("id", id)
            .attr("participant", device_form(participant));
        // shortcut: status candidates keep their prior class, port effective StatusInfraConfig next.
        return Some(if status {
            receipt.attr("class", "status")
        } else {
            receipt
        });
    }
    // Group message: the chat is the group and `participant` names the sender.
    if incoming.description == "message"
        && let Some((group, participant)) = super::inbox::group_sender(incoming)
    {
        if incoming.get_attr("edit").is_some() {
            return None;
        }
        let id = incoming.get_attr("id")?.as_text()?;
        if id.is_empty() || id.len() > 128 {
            return None;
        }
        return Some(
            Node::new("receipt")
                .attr("to", group)
                .attr("id", id)
                .attr("participant", device_form(participant)),
        );
    }
    if incoming.get_attr("participant").is_some() {
        return None;
    }
    let from = incoming.get_attr("from")?;
    let jid = from
        .as_jid()
        .cloned()
        .or_else(|| Jid::parse(from.as_text()?).ok())?;
    // APK 37.71 Cp2.A00 -> CmC.A06 uses the supplied DeviceJid as
    // direct receipt destination when no alternate/broadcast route applies.
    // A companion address is not a reason to suppress delivery/read.
    if jid.agent != 0
        || !matches!(
            jid.server,
            crate::model::jid::Server::User | crate::model::jid::Server::Lid
        )
    {
        return None;
    }
    let id = incoming.get_attr("id")?.as_text()?;
    if id.is_empty() || id.len() > 128 {
        return None;
    }
    if incoming.get_attr("edit").is_some() {
        return None;
    }
    Some(
        Node::new("receipt")
            .attr("to", device_form(jid))
            .attr("id", id),
    )
}

/// Build a confirmation using the saved account's own PN/LID identity.
///
/// Receive callers must capture the node durably and authenticate/decrypt a
/// message before confirming it. Own statuses and supported committed own-device
/// text use `type=sender`; peer metadata
/// is never ownership authority. When own-LID authority is needed, malformed
/// or conflicting records return an error before a confirmation can be written.
/// This does not send or queue.
pub fn received_confirmation_for_account(
    store: &crate::store::ProtocolStore,
    account: &crate::session::AccountRecord,
    incoming: &Node,
) -> crate::Result<Option<Node>> {
    if incoming.description == "message" && incoming.get_attr("recipient").is_some() {
        return super::own_device::confirmation(store, account, incoming);
    }
    let Some(mut receipt) = received_confirmation(incoming) else {
        return Ok(None);
    };
    if receipt.description == "receipt"
        && receipt.get_attr("class").and_then(|value| value.as_text()) == Some("status")
        && receipt
            .get_attr("to")
            .and_then(|v| v.as_jid())
            .is_some_and(|jid| {
                jid.server == crate::model::jid::Server::Broadcast && jid.user == "status"
            })
    {
        let Some(sender) = super::receipts::jid(incoming, "participant") else {
            return Ok(None);
        };
        let sender = sender.to_non_ad();
        // APK 1RE.A01 / CnN.A04 -> 08i.BO0 uses stored own PN/LID, not peer aliases.
        let own = account
            .own_jid
            .as_ref()
            .is_some_and(|jid| jid.to_non_ad() == sender)
            || (sender.server == crate::model::jid::Server::Lid
                && super::group_send::own_lid(store)?.as_ref() == Some(&sender));
        if own {
            receipt.attributes.insert(
                2,
                crate::binary::Attribute {
                    name: "type".into(),
                    value: "sender".into(),
                },
            );
        }
    }
    Ok(Some(receipt))
}

/// Acknowledge an incoming receipt, as `LX/Cp2->A05` (2.26.37.71,
/// `WriterThread/write/send-message-ack`, **confirmed**) builds it from the
/// stanza: `id`; `to` = the receipt's `from`, as given; `class="receipt"`;
/// `type` unless the receipt's is absent or `delivery`; then `participant` and
/// `recipient` when present, and `edit` when present and not `"0"`.
///
/// Until 2026-09-28 a receipt carrying `participant` or `recipient`, or coming
/// from a group or a companion device, got no ack at all, so every group
/// delivery and read receipt was redelivered at each login (seen live as a
/// burst of `receipt chat=group confirmed=false` lines).
///
/// Only the receipt types this client handles are acknowledged — delivery,
/// read and played. Retry success/error handling is distinct: the evidenced
/// unsupported-version and confirmed schema-error branches get error487;
/// a separately reserved resend may receive a success ACK through the private
/// handled constructor. Other cases wait for their completion policy.
fn receipt_ack(incoming: &Node) -> Option<Node> {
    receipt_ack_after_processing(incoming, false)
}

// Only retry_answer's completed durable reservation calls this. A valid parser
// result alone is not proof that this client handled a receipt.
pub(super) fn handled_retry_confirmation(incoming: &Node) -> Option<Node> {
    let retry = super::receipts::parse_retry(incoming)?;
    if !retry.chat.is_user_like()
        || !retry.device.is_user_like()
        || !retry_context_supported(incoming)
    {
        return None;
    }
    receipt_ack_after_processing(incoming, true)
}

fn receipt_ack_after_processing(incoming: &Node, handled_retry: bool) -> Option<Node> {
    let from = super::receipts::ack_address(super::receipts::network_jid(incoming, "from")?);
    // APK43.72 distinguishes F10 null/unhandled (2Cx) from LIL parse errors
    // (1rg). Both use487, but their context fields differ. Local capability
    // refusals are not a generic parse error. docs/protocol/retry-ack-dispatch.md
    let retry_error = retry_receipt_error(incoming);
    let kind = if retry_error.is_some() || handled_retry {
        "retry"
    } else {
        super::receipts::receipt_kind(incoming)?
    };
    let id = incoming.get_attr("id")?.as_text()?;
    if id.is_empty() || id.len() > 128 {
        return None;
    }
    let mut ack = Node::new("ack")
        .attr("id", id)
        .attr("to", from)
        .attr("class", "receipt");
    if kind != "delivery" {
        ack = ack.attr("type", kind);
    }
    if let Some(participant) = super::receipts::network_jid(incoming, "participant") {
        ack = ack.attr("participant", super::receipts::ack_address(participant));
    }
    // 1rg.A03 makes a fresh parse-error context with no recipient/edit.
    // 2Cx's unhandled-version path instead copies the original Eoh context.
    if !matches!(retry_error, Some(RetryReceiptError::Parsing(_))) {
        if let Some(recipient) = super::receipts::recipient(incoming) {
            ack = ack.attr("recipient", recipient);
        }
        if let Some(edit) = incoming
            .get_attr("edit")
            .and_then(|v| v.as_text())
            .filter(|e| !e.is_empty() && *e != "0")
        {
            ack = ack.attr("edit", edit);
        }
    }
    if let Some(error) = retry_error {
        let field = match error {
            RetryReceiptError::UnsupportedVersion => "retry.v",
            RetryReceiptError::Parsing(field) => field,
        };
        tracing::debug!(field, error = 487, "retry receipt error response");
        ack = ack.attr("error", "487");
    }
    Some(ack)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RetryReceiptError {
    UnsupportedVersion,
    Parsing(&'static str),
}

fn retry_context_supported(incoming: &Node) -> bool {
    // Keep unsupported context/address grammars unacknowledged. In particular,
    // do not infer which exception precedes F10 for a context we cannot read.
    if incoming.get_attr("participant").is_some()
        && super::receipts::network_jid(incoming, "participant").is_none()
    {
        return false;
    }
    if incoming.get_attr("recipient").is_some() && super::receipts::recipient(incoming).is_none() {
        return false;
    }
    if incoming
        .get_attr("edit")
        .is_some_and(|v| v.as_text().is_none())
    {
        return false;
    }
    true
}

fn retry_receipt_error(incoming: &Node) -> Option<RetryReceiptError> {
    if incoming.get_attr("type").and_then(|v| v.as_text()) != Some("retry") {
        return None;
    }
    if !retry_context_supported(incoming) {
        return None;
    }
    // F10.A04 reads required retry, then registration (0Tp.A0G throws LIL),
    // and requires exactly four registration bytes before checking version.
    let Some(retry) = incoming.child("retry") else {
        return Some(RetryReceiptError::Parsing("retry"));
    };
    if !matches!(
        incoming.child("registration").map(|n| &n.content),
        Some(crate::binary::NodeContent::Bytes(bytes)) if bytes.len() == 4
    ) {
        return Some(RetryReceiptError::Parsing("registration"));
    }
    let version = match retry.get_attr("v") {
        None => "1",
        Some(value) => value.as_text()?,
    };
    if version != "1" {
        return Some(RetryReceiptError::UnsupportedVersion);
    }
    // F10 then reads count (default1), required signed64 t, required id. Java
    // parseInt/parseLong wrap NumberFormatException as LIL. Classify ASCII
    // only: unsupported Unicode grammar must not become a false protocol error.
    if let Some(count) = retry.get_attr("count") {
        let count = count.as_text()?;
        if !count.is_ascii() {
            return None;
        }
        if count.parse::<i32>().is_err() {
            return Some(RetryReceiptError::Parsing("retry.count"));
        }
    }
    let Some(timestamp) = retry.get_attr("t") else {
        return Some(RetryReceiptError::Parsing("retry.t"));
    };
    let timestamp = timestamp.as_text()?;
    if !timestamp.is_ascii() {
        return None;
    }
    if timestamp.parse::<i64>().is_err() {
        return Some(RetryReceiptError::Parsing("retry.t"));
    }
    if retry.get_attr("id").is_none() {
        return Some(RetryReceiptError::Parsing("retry.id"));
    }
    // Empty/oversize ids, negative/high counts and carried keys are local
    // admission/capability limits, not evidence of a native LIL exception.
    None
}

/// Comparison LX/0ip.A07: request current media routes, not an upload.
pub fn media_route_request(id: &str) -> Node {
    Node::new("iq")
        .attr("id", id)
        .attr("xmlns", "w:m")
        .attr("type", "set")
        .attr("to", Jid::parse("s.whatsapp.net").expect("constant JID"))
        .children(vec![Node::new("media_conn")])
}

/// Classification only: callers must separately correlate the request ID/address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAckOutcome {
    /// Message-class ACK without an error; correlation is the caller's duty.
    Accepted,
    /// Ordinary message-class integral error, excluding the special auth challenge.
    Refused,
    /// Numeric459 diverted by the APK before ordinary ACK dispatch.
    AuthChallenge,
    /// Other tags/classes or unsupported malformed error attributes cannot settle it.
    Unrelated,
}
impl MessageAckOutcome {
    /// Durable comparison-send status; uncertain outcomes never authorize replay.
    pub fn status(self) -> &'static str {
        match self {
            Self::Accepted => "server_acknowledged",
            Self::Refused => "server_refused",
            Self::AuthChallenge | Self::Unrelated => "outcome_unknown",
        }
    }
}

/// Classify an ordinary message ACK only for its selected expected chat.
/// The caller must separately bind the original wire ID and request ownership.
/// This performs no capture, persistence, full envelope validation or replay.
/// Expected PN/LID device addresses compare as their user address. Ordinary
/// success with a supported PN/LID participant remains uncertain; refusals
/// and challenges use their separate existing outcome rules.
pub fn message_ack_outcome_for(node: &Node, expected: &Jid) -> MessageAckOutcome {
    let expected = if matches!(
        expected.server,
        crate::model::jid::Server::User | crate::model::jid::Server::Lid
    ) {
        expected.to_non_ad()
    } else {
        expected.clone()
    };
    if super::outgoing::message_ack_chat(node).as_ref() != Some(&expected) {
        return MessageAckOutcome::Unrelated;
    }
    let outcome = message_ack_outcome(node);
    if outcome == MessageAckOutcome::Accepted && ordinary_ack_has_participant(node) {
        return MessageAckOutcome::Unrelated;
    }
    outcome
}

// APK43.72 1rH.A03 -> 0EB.A00 -> 1lZ.A10/A0H -> 196.BLn:
// ordinary success ignores a converted participant. Plain PN/LID and their
// devices convert too. Special bot-feedback handling is not implemented here.
pub(crate) fn ordinary_ack_has_participant(node: &Node) -> bool {
    if let Some(parsed) = node
        .get_attr("participant")
        .and_then(|value| value.as_text())
        .and_then(ack_user_text)
    {
        return matches!(parsed, AckUserText::User(_) | AckUserText::Device(_));
    }

    super::receipts::jid(node, "participant").is_some_and(|jid| {
        matches!(
            jid.server,
            crate::model::jid::Server::User | crate::model::jid::Server::Lid
        ) && !jid.user.is_empty()
    })
}

// APK43.72 1rH.A03 calls0Tp.A05(error,0) before class dispatch.
// A06 uses Integer.parseInt; malformed/out-of-i32 values throw LIL rather
// than becoming an ordinary ACK outcome. Nontext decoded values are outside
// this supported Rust adapter. No reader recovery policy is inferred here.
pub(crate) fn parsed_ack_error(node: &Node) -> Result<Option<i32>, ()> {
    match node.get_attr("error") {
        None => Ok(None),
        Some(value) => value
            .as_text()
            .and_then(super::mex::java_decimal_long)
            .and_then(|value| i32::try_from(value).ok())
            .map(Some)
            .ok_or(()),
    }
}

pub(crate) fn ack_error_is_valid(node: &Node) -> bool {
    parsed_ack_error(node).is_ok()
}

// APK43.72 0De.A02 maps plain lid/hosted.lid to 0Dx. Its constructor
// requires 1..15 ASCII digits and rejects leading zero; this is not Java's
// Unicode-capable Integer.parseInt grammar used for numeric ACK errors.
pub(crate) fn native_plain_lid_user_valid(user: &str) -> bool {
    (1..=15).contains(&user.len())
        && !user.starts_with('0')
        && user.bytes().all(|byte| byte.is_ascii_digit())
}
// PhoneUserJid -> 0Dl.A02 (APK43.72): protocol user grammar, not
// registration/E.164 validation. Special factory singletons bypass this.
fn native_plain_pn_user_valid(user: &str) -> bool {
    (5..=20).contains(&user.len())
        && !user.starts_with('0')
        && (!user.starts_with("10") || user.starts_with("1008"))
        && user.bytes().all(|byte| byte.is_ascii_digit())
}
pub(crate) enum AckUserText<'a> {
    Null,
    User(&'a str),
    Device(&'a str),
    // Dv7/type8 is a UserJid but not an ordinary participant-device user.
    ServiceUser(&'a str),
    // J9O/type9 extends0DY, so a UserJid recipient cast fails.
    NonUser(&'a str),
}
// Pinned 0Dd.A00 uses last dot/colon, ignores the dot suffix, parses
// device with Integer.parseInt, then DeviceJid narrows to byte before range
// checking. Dv6/Dv4 add original-int == 99 for hosted devices.
pub(crate) fn ack_user_text(text: &str) -> Option<AckUserText<'_>> {
    let (local, domain) = text.rsplit_once('@')?;
    if !matches!(domain, "lid" | "hosted.lid" | "s.whatsapp.net" | "hosted") {
        return None;
    }
    let dot = local.rfind('.');
    let colon = local.rfind(':');
    let lid = matches!(domain, "lid" | "hosted.lid");
    // APK43.72 DCF/Dv7/J9O bypass PhoneUserJid. Preserve their native
    // class rather than applying its numeric constructor to special names.
    if !lid && dot.is_none() && colon.is_none() {
        match local {
            "0" => return Some(AckUserText::User(local)),
            "Server" => return Some(AckUserText::ServiceUser(local)),
            "gdpr" if domain == "s.whatsapp.net" => return Some(AckUserText::NonUser(local)),
            _ => {}
        }
    }
    let valid_user = |user: &str| {
        if lid {
            native_plain_lid_user_valid(user)
        } else {
            // 0De.A04 -> A02 accepts DCF/Dv7 as UserJid bases too;
            // 0EE/Dv4 do not require PhoneUserJid specifically.
            matches!(user, "0" | "Server") || native_plain_pn_user_valid(user)
        }
    };
    let prefix_end = match (dot, colon) {
        (None, None) => {
            return Some(if valid_user(local) {
                AckUserText::User(local)
            } else {
                AckUserText::Null
            });
        }
        (Some(dot), Some(colon)) => dot.min(colon),
        (Some(index), None) | (None, Some(index)) => index,
    };
    if local.ends_with(['.', ':']) {
        return Some(AckUserText::Null);
    }
    let user = &local[..prefix_end];
    if !valid_user(user) {
        return Some(AckUserText::Null);
    }
    let device = match colon {
        None => 0,
        Some(colon) => {
            let Some(device) = super::mex::java_decimal_long(&local[colon + 1..])
                .and_then(|v| i32::try_from(v).ok())
            else {
                return Some(AckUserText::Null);
            };
            device
        }
    };
    if !(0..=99).contains(&(device as i8))
        || (matches!(domain, "hosted.lid" | "hosted") && device != 99)
    {
        return Some(AckUserText::Null);
    }
    Some(AckUserText::Device(user))
}

// Pinned1bc/0nW.A02 group,2U8 broadcast constructors and0De.A03 newsletter factory.
// Newsletter's error text says number, but its actual predicate only checks
// Java-trimmed nonempty/nonzero-leading text; original user is preserved.
pub(crate) fn ack_non_user_text_valid(text: &str) -> Option<bool> {
    let (user, domain) = text.rsplit_once('@')?;
    let decimal_nonzero = |value: &str, max: usize| {
        !value.is_empty()
            && value.len() <= max
            && !value.starts_with('0')
            && value.bytes().all(|b| b.is_ascii_digit())
    };
    match domain {
        "broadcast" => Some(matches!(user, "status" | "location") || decimal_nonzero(user, 20)),
        "g.us" => Some(
            decimal_nonzero(user, 20)
                || user.split_once('-').is_some_and(|(phone, time)| {
                    native_plain_pn_user_valid(phone)
                        && time.len() == 10
                        && decimal_nonzero(time, 10)
                }),
        ),
        "newsletter" => {
            let trimmed = user.trim_matches(|c: char| c <= '\u{20}');
            Some(!trimmed.is_empty() && !trimmed.starts_with('0'))
        }
        _ => None,
    }
}

// APK43.72 0Tp.A0A -> 0Dd.A02/A00: definite null results for
// the text parse path only. Do not substitute Rust's wider/narrower JID grammar.
fn call_ack_sender_known_null(node: &Node) -> bool {
    let Some(value) = node.get_attr("from") else {
        return true;
    };
    let Some(text) = value.as_text() else {
        return false;
    };
    if text.is_empty() {
        return true;
    }
    // A00's domain switch plus A02/A04 factory/cache aliases. Recognized
    // domains/bare names remain unclassified here: constructors/cache may matter.
    let recognized = |name: &str| {
        matches!(
            name,
            "s.whatsapp.net"
                | "broadcast"
                | "hosted"
                | "hosted.lid"
                | "bot"
                | "lid"
                | "call"
                | "g.us"
                | "temp"
                | "newsletter"
                | "interop"
                | "interop.g.us"
                | "status_me"
                | "lid_me"
        )
    };
    let Some((user, domain)) = text.rsplit_once('@') else {
        return !recognized(text);
    };
    // Empty-user cache keys omit @ (0Dd.A01); unknown domains cannot be
    // inserted by the selected parser or user/newsletter cache factories.
    user.is_empty()
        || !recognized(domain)
        || matches!(ack_user_text(text), Some(AckUserText::Null))
        || ack_non_user_text_valid(text) == Some(false)
}

/// Validate ACK required-ID, numeric and supported call-field admission. Stateful receivers must first capture the Node.
/// A malformed ACK ends the receiving epoch (APK43.72 1rg.A04 excludes ack;
/// the selected 1s2.run outer catch reports onError(2) and returns).
/// This performs no capture/I/O; callers must stop and discard their channel
/// on failure. It never settles an intent or authorizes reconnection/replay.
pub fn validate_received_ack(node: &Node) -> crate::Result<()> {
    // APK43.72 1rH.A03 0054..0064 calls required 0Tp.A0L(id)
    // before numeric459/class routing; missing id throws LIL.
    if node.description == "ack" && node.get_attr("id").is_none() {
        return Err(crate::Error::invalid(
            "ack",
            "missing ACK id; receive epoch consumed; preserve uncertainty and do not reconnect automatically",
        ));
    }
    if node.description == "ack" && !ack_error_is_valid(node) {
        return Err(crate::Error::invalid(
            "ack",
            "malformed ACK error; receive epoch consumed; preserve uncertainty and do not reconnect automatically",
        ));
    }
    // APK43.72 1rH.A03 0518..053c -> 05f8/0608: call ACKs
    // require nonempty textual id/type. Numeric459 returns before this branch.
    if node.description == "ack"
        && node.get_attr("class").and_then(|v| v.as_text()) == Some("call")
        && !is_auth_challenge_ack(node)
        // 04c2 null from throws IllegalStateException before call fields;
        // 1s2.run's Exception -> 1rg.A00 path returns to reading instead.
        && !call_ack_sender_known_null(node)
        && (node.get_attr("id").and_then(|v| v.as_text()) == Some("")
            || node.get_attr("type").is_none()
            || node.get_attr("type").and_then(|v| v.as_text()) == Some("")
            // DCF/Dv7/J9O types7/8/9 are nonnull but excluded by0Dn.A0R.
            || matches!(node.get_attr("from").and_then(|v| v.as_text()).and_then(ack_user_text),
                Some(AckUserText::User("0") | AckUserText::ServiceUser(_) | AckUserText::NonUser(_)))
            // Text constructor success establishes excluded types1/21/3/5/6 even if Rust's
            // generic JID parser cannot represent the native newsletter user.
            || node.get_attr("from").and_then(|v| v.as_text()).and_then(ack_non_user_text_valid) == Some(true)
            // EW4.<init> -> 0Dn.A0R rejects known group/newsletter types1/21.
            || super::receipts::jid(node, "from").is_some_and(|jid|
                matches!(jid.server, crate::model::jid::Server::Group | crate::model::jid::Server::Newsletter)))
    {
        return Err(crate::Error::invalid(
            "ack",
            "invalid call ACK id/type or unsupported sender; receive epoch consumed; preserve uncertainty and do not reconnect automatically",
        ));
    }

    Ok(())
}

/// APK43.72 1rH.A03 diverts numeric459 to1lZ.A1F before class dispatch.
/// This unsupported challenge is not an ordinary terminal send refusal.
pub(crate) fn is_auth_challenge_ack(node: &Node) -> bool {
    node.description == "ack" && matches!(parsed_ack_error(node), Ok(Some(459)))
}

/// APK43.72 1rH.A03: integral error admission and challenge459 precede class
/// dispatch. Unsupported malformed errors cannot settle an ordinary ACK.
/// This does not implement native reader error recovery or address normalization.
pub fn message_ack_outcome(node: &Node) -> MessageAckOutcome {
    if node.description != "ack" {
        return MessageAckOutcome::Unrelated;
    }
    let Ok(error) = parsed_ack_error(node) else {
        return MessageAckOutcome::Unrelated;
    };
    if error == Some(459) {
        MessageAckOutcome::AuthChallenge
    } else if node.get_attr("class").and_then(|v| v.as_text()) != Some("message") {
        MessageAckOutcome::Unrelated
    } else if error.is_some() {
        MessageAckOutcome::Refused
    } else {
        MessageAckOutcome::Accepted
    }
}

/// Direct primary-device message envelope; encrypted payload only.
pub fn direct_message(
    recipient: &Jid,
    id: &str,
    kind: &str,
    ciphertext: Vec<u8>,
) -> Result<Node, TransportError> {
    direct_typed(recipient, id, kind, ciphertext, "text")
}

/// Add the recipient's trusted-contact token to a direct message.
///
/// **Observed 2026-11-01:** without it the server refused our direct texts to
/// a person who had issued us a token with error 463, which public clients
/// name "1:1 message missing privacy token" (Baileys `SERVER_ERROR_CODES`,
/// reference only). Wrapping the `enc` in `<participants><to>` beside the
/// token, as 2.26.37.71 `Cp3.A00` does for fan-out entries, was refused 479
/// (`SmaxInvalid`, schema validation) for this top-level-`enc` direct form,
/// so the token is a `<message>` child here, the placement public clients
/// use for 1:1 messages. APK: `Cp3.A01` also gates on property 20605.
pub fn with_tctoken(stanza: Node, token: &[u8]) -> Result<Node, TransportError> {
    if !(1..=120).contains(&token.len()) {
        return Err(error("a tctoken is 1 to 120 bytes (LX/0hp->A04)"));
    }
    let mut children = stanza.child_nodes().to_vec();
    children.push(Node::new("tctoken").bytes(token.to_vec()));
    Ok(stanza.children(children))
}

/// Direct envelope with an explicit stanza `type`. `text` is the ordinary
/// message; `reaction` is the comparison kind for reaction payloads
/// (`LX/0zO.A01` kind table, observed live on incoming reactions).
pub fn direct_typed(
    recipient: &Jid,
    id: &str,
    kind: &str,
    ciphertext: Vec<u8>,
    stanza_type: &str,
) -> Result<Node, TransportError> {
    if !matches!(kind, "pkmsg" | "msg")
        || !matches!(stanza_type, "text" | "reaction")
        || ciphertext.is_empty()
        || id.is_empty()
    {
        return Err(error("invalid direct-message envelope"));
    }
    let mut enc = Node::new("enc").attr("v", "2").attr("type", kind);
    if stanza_type == "reaction" {
        // LX/BpS case 1 (2.26.37.71) builds the reaction `enc` with
        // `decrypt-fail="hide"`, also seen on the live incoming reaction.
        enc = enc.attr("decrypt-fail", "hide");
    }
    let mut children = vec![enc.bytes(ciphertext)];
    if stanza_type == "reaction" {
        // Observed on a live incoming reaction from the official client
        // (2.26.37.71): `meta content_type="add_on"` marks add-on messages; the
        // names are binary tokens (LX/1Rl) built by LX/Civ.A02. Without it the
        // recipient did not display our reaction.
        children.push(Node::new("meta").attr("content_type", "add_on"));
    }
    Ok(Node::new("message")
        .attr("to", recipient.clone())
        .attr("id", id)
        .attr("type", stanza_type)
        .children(children))
}

/// Poll envelope: `message type=poll` with `meta polltype=<kind>` (observed
/// live for `creation`; the APK's `polltype` values are creation, vote,
/// add_option, result_snapshot and quiz_creation).
pub fn poll_envelope(
    recipient: &Jid,
    id: &str,
    kind: &str,
    ciphertext: Vec<u8>,
    polltype: &str,
) -> Result<Node, TransportError> {
    if !matches!(kind, "pkmsg" | "msg")
        || !matches!(polltype, "creation" | "vote")
        || ciphertext.is_empty()
        || id.is_empty()
    {
        return Err(error("invalid poll envelope"));
    }
    // Observed live from the official client: a poll creation carries
    // `meta polltype=creation`; a vote carries `meta polltype=vote
    // content_type=add_on` and `enc decrypt-fail=hide` like other add-ons.
    let mut enc = Node::new("enc").attr("v", "2").attr("type", kind);
    let mut meta = Node::new("meta").attr("polltype", polltype);
    if polltype == "vote" {
        enc = enc.attr("decrypt-fail", "hide");
        meta = meta.attr("content_type", "add_on");
    }
    Ok(Node::new("message")
        .attr("to", recipient.clone())
        .attr("id", id)
        .attr("type", "poll")
        .children(vec![enc.bytes(ciphertext), meta]))
}

/// Comparison image envelope: media message, encrypted image payload.
pub fn direct_image(
    recipient: &Jid,
    id: &str,
    kind: &str,
    ciphertext: Vec<u8>,
) -> Result<Node, TransportError> {
    direct_media(recipient, id, kind, ciphertext, "image")
}

/// Comparison encrypted media envelope; supported media types only.
pub fn direct_media(
    recipient: &Jid,
    id: &str,
    kind: &str,
    ciphertext: Vec<u8>,
    media_type: &str,
) -> Result<Node, TransportError> {
    // ADx.A0O's mediatype table includes "document" (2.26.37.71).
    if !matches!(media_type, "image" | "ptt" | "document" | "video") {
        return Err(error("unsupported media type"));
    }
    if !matches!(kind, "pkmsg" | "msg") || ciphertext.is_empty() || id.is_empty() {
        return Err(error("invalid direct-image envelope"));
    }
    Ok(Node::new("message")
        .attr("to", recipient.clone())
        .attr("id", id)
        .attr("type", "media")
        .children(vec![
            Node::new("enc")
                .attr("v", "2")
                .attr("type", kind)
                .attr("mediatype", media_type)
                .bytes(ciphertext),
        ]))
}

/// IQ response grammar only; caller supplies its reserved ID and deadline policy.
/// APK43.72 0ln.A01 dispatches exact result/error, never request types.
pub fn iq_reply_matches(node: &Node, id: &str) -> bool {
    node.description == "iq"
        && node.get_attr("id").and_then(|v| v.as_text()) == Some(id)
        && matches!(
            node.get_attr("type").and_then(|v| v.as_text()),
            Some("result" | "error")
        )
}

fn waited_reply_matches(node: &Node, description: &str, id: &str) -> bool {
    if description == "iq" {
        iq_reply_matches(node, id)
    } else {
        node.description == description && node.get_attr("id").and_then(|v| v.as_text()) == Some(id)
    }
}

/// Wait for an exact stanza/id pair (IQ requires result/error), with a
/// whole-operation deadline and count bound. No resend. Nonmatching frames are
/// returned for persistence on success; error paths do not return them. Durable
/// operations should use [`wait_for_captured`] instead. Malformed ACK errors
/// stop the wait; callers must discard the borrowed channel on failure.
pub async fn wait_for<S: AsyncRead + AsyncWrite + Unpin>(
    channel: &mut EncryptedConnection<S>,
    description: &str,
    id: &str,
) -> Result<(Node, Vec<Node>), TransportError> {
    wait_for_observed(
        async || read_node(channel).await,
        |node| {
            validate_received_ack(node).map_err(|_| {
                error("invalid ACK admission; receive epoch consumed; discard channel")
            })
        },
        description,
        id,
        tokio::time::Instant::now() + Duration::from_secs(30),
    )
    .await
}

/// Persist decoded nodes and decrypted undecodable frames during a comparison wait.
/// Undecodable frames and capture failures stop with uncertainty; no resend.
/// Request senders should use [`write_and_wait_captured`] to include write latency.
pub async fn wait_for_captured<S: AsyncRead + AsyncWrite + Unpin>(
    store: &crate::store::ProtocolStore,
    channel: &mut EncryptedConnection<S>,
    description: &str,
    id: &str,
) -> crate::Result<(Node, Vec<Node>)> {
    wait_for_captured_until(
        store,
        channel,
        description,
        id,
        tokio::time::Instant::now() + Duration::from_secs(30),
    )
    .await
}

/// Write exactly once and await a captured response under one pre-write deadline.
/// The caller owns any required durable reservation; this helper never retries.
/// Awaited writes can delay observing expiry but cannot restart the30s window.
pub async fn write_and_wait_captured<S: AsyncRead + AsyncWrite + Unpin>(
    store: &crate::store::ProtocolStore,
    channel: &mut EncryptedConnection<S>,
    request: &Node,
    description: &str,
    id: &str,
) -> crate::Result<(Node, Vec<Node>)> {
    // APK43.72 0Sf.A0A registers through0ln.A08 before dispatch.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    write_node(channel, request).await?;
    wait_for_captured_until(store, channel, description, id, deadline).await
}

async fn wait_for_captured_until<S: AsyncRead + AsyncWrite + Unpin>(
    store: &crate::store::ProtocolStore,
    channel: &mut EncryptedConnection<S>,
    description: &str,
    id: &str,
    deadline: tokio::time::Instant,
) -> crate::Result<(Node, Vec<Node>)> {
    wait_for_observed(
        async || captured_wait_node(store, read_frame(channel).await?),
        |node| preserve_waited_node(store, node),
        description,
        id,
        deadline,
    )
    .await
}

fn preserve_waited_node(store: &crate::store::ProtocolStore, node: &Node) -> crate::Result<()> {
    super::inbox::preserve(store, node)?;
    validate_received_ack(node)
}

fn captured_wait_node(
    store: &crate::store::ProtocolStore,
    frame: FrameRead,
) -> crate::Result<Node> {
    match frame {
        FrameRead::Node(node) => Ok(node),
        FrameRead::Undecodable { frame, class } => {
            super::inbox::preserve_undecodable(store, &frame, class, None)?;
            Err(crate::Error::invalid(
                "comparison_wait",
                "undecodable decrypted frame captured; response outcome unknown; no automatic retry",
            ))
        }
    }
}

async fn wait_for_observed<R, C, E>(
    mut read: R,
    mut capture: C,
    description: &str,
    id: &str,
    deadline: tokio::time::Instant,
) -> Result<(Node, Vec<Node>), E>
where
    R: AsyncFnMut() -> Result<Node, E>,
    C: FnMut(&Node) -> Result<(), E>,
    E: From<TransportError>,
{
    if tokio::time::Instant::now() >= deadline {
        return Err(E::from(TransportError::Deadline("correlated stanza")));
    }
    tokio::time::timeout_at(deadline, async {
        let mut pending = Vec::new();
        for _ in 0..32 {
            let node = read().await?;
            // Capture terminal and matched replies too, before any settlement.
            capture(&node)?;
            if tokio::time::Instant::now() >= deadline {
                return Err(E::from(TransportError::Deadline("correlated stanza")));
            }
            if waited_reply_matches(&node, description, id) {
                return Ok((node, pending));
            }
            if matches!(node.description.as_str(), "failure" | "stream:error") {
                return Err(E::from(error(
                    "server ended session while awaiting response",
                )));
            }
            pending.push(node);
        }
        Err(E::from(error(
            "stanza count exceeded while awaiting response",
        )))
    })
    .await
    .map_err(|_| E::from(TransportError::Deadline("correlated stanza")))?
}

#[cfg(test)]
mod retry_send_tests {
    use super::*;

    /// The shape is mirrored from the receipts the operator's handset sends us,
    /// and the registration id is the field that lets the sender decide between
    /// reusing their session and fetching a fresh bundle.
    #[test]
    fn a_tctoken_is_a_message_child_after_the_enc() {
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let node = with_tctoken(
            direct_message(&peer, "M1", "msg", vec![1]).unwrap(),
            &[9; 8],
        )
        .unwrap();
        let names: Vec<_> = node
            .child_nodes()
            .iter()
            .map(|c| c.description.as_str())
            .collect();
        assert_eq!(names, ["enc", "tctoken"]);
        assert!(with_tctoken(node, &[]).is_err());
    }

    #[test]
    fn a_retry_receipt_mirrors_the_observed_shape() {
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let participant = Jid::parse("222222222222222@lid").unwrap();
        let node = retry_receipt(&chat, Some(&participant), "M1", 1, 0x0102_0304, 99).unwrap();
        assert_eq!(node.description, "receipt");
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("retry"));
        assert_eq!(node.get_attr("id").unwrap().as_text(), Some("M1"));
        assert_eq!(node.get_attr("to").unwrap().as_jid(), Some(&chat));
        assert_eq!(
            node.get_attr("participant").unwrap().as_jid(),
            Some(&participant)
        );
        let retry = node.child("retry").unwrap();
        assert_eq!(retry.get_attr("count").unwrap().as_text(), Some("1"));
        assert_eq!(retry.get_attr("v").unwrap().as_text(), Some("1"));
        let registration = node.child("registration").unwrap();
        assert!(matches!(
            &registration.content,
            crate::binary::NodeContent::Bytes(b) if b.as_ref() == [1, 2, 3, 4]
        ));
        // A direct retry names no participant.
        let direct = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        assert!(
            retry_receipt(&direct, None, "M1", 1, 7, 99)
                .unwrap()
                .get_attr("participant")
                .is_none()
        );
        // It must not be sent for a nonsense message or an unbounded count.
        assert!(retry_receipt(&chat, None, "", 1, 7, 99).is_err());
        assert!(retry_receipt(&chat, None, "M1", 0, 7, 99).is_err());
        assert!(retry_receipt(&chat, None, "M1", 99, 7, 99).is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delivery_confirmation_is_not_an_opened_receipt() {
        let message = Node::new("message")
            .attr("id", "test")
            .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap());
        let delivery = received_confirmation(&message).unwrap();
        assert_eq!(delivery.description, "receipt");
        assert!(delivery.get_attr("type").is_none());
        assert!(delivery.child_nodes().is_empty());
        assert!(received_confirmation(&message.clone().attr("participant", "x")).is_none());
        assert!(received_confirmation(&message.clone().attr("edit", "7")).is_none());
        assert!(received_confirmation(&Node::new("notification")).is_none());
        let receipt = Node {
            description: "receipt".into(),
            ..message
        };
        let ack = received_confirmation(&receipt.clone().attr("type", "read")).unwrap();
        assert_eq!(ack.description, "ack");
        assert_eq!(ack.get_attr("class").unwrap().as_text(), Some("receipt"));
        assert_eq!(ack.get_attr("type").unwrap().as_text(), Some("read"));
        assert!(
            received_confirmation(&receipt.clone().attr("type", "retry").children(vec![
                Node::new("retry").attr("id", "W1").attr("t", "1"),
                Node::new("registration").bytes(vec![1, 2, 3, 4]),
            ]))
            .is_none()
        );
        assert!(
            received_confirmation(&receipt.attr("type", "delivery"))
                .unwrap()
                .get_attr("type")
                .is_none()
        );
    }
    #[test]
    fn unsupported_retry_version_gets_native_487_without_parsing_later_fields() {
        let from = Jid::parse("15555550100:2@s.whatsapp.net").unwrap();
        let participant = Jid::parse("15555550101:3@s.whatsapp.net").unwrap();
        let recipient = Jid::parse("123456789@lid").unwrap();
        for version in ["", "0", "2", "01"] {
            // Native returns null at F10.A04+004e before required nested id/t,
            // count or carried keys. Root receipt ID remains ACK correlation.
            let node = Node::new("receipt")
                .attr("id", "ROOT")
                .attr("from", from.clone())
                .attr("type", "retry")
                .attr("participant", participant.clone())
                .attr("recipient", recipient.clone())
                .attr("edit", "7")
                .children(vec![
                    Node::new("retry").attr("v", version).attr("count", "bad"),
                    Node::new("registration").bytes(vec![1, 2, 3, 4]),
                    Node::new("keys"),
                ]);
            assert!(super::super::receipts::parse_retry(&node).is_none());
            let ack = received_confirmation(&node).unwrap();
            assert_eq!(
                ack,
                Node::new("ack")
                    .attr("id", "ROOT")
                    .attr("to", super::super::receipts::ack_address(from.clone()))
                    .attr("class", "receipt")
                    .attr("type", "retry")
                    .attr(
                        "participant",
                        super::super::receipts::ack_address(participant.clone())
                    )
                    .attr("recipient", recipient.clone())
                    .attr("edit", "7")
                    .attr("error", "487")
            );
            let dictionary = TokenDictionary::comparison_2_26_37_71();
            let encoded = Encoder::new(dictionary).encode(&ack).unwrap();
            assert_eq!(
                Decoder::new(&encoded[1..], dictionary, &Limits::default())
                    .decode_node()
                    .unwrap(),
                ack
            );
        }
    }

    #[test]
    fn retry_version_error_does_not_ack_other_local_refusals() {
        let base = Node::new("receipt")
            .attr("id", "ROOT")
            .attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap())
            .attr("type", "retry");
        let registration = || Node::new("registration").bytes(vec![1, 2, 3, 4]);
        for retry in [
            Node::new("retry").attr("id", "W1").attr("t", "1"),
            Node::new("retry")
                .attr("v", "1")
                .attr("id", "W1")
                .attr("t", "1"),
            Node::new("retry")
                .attr("v", "1")
                .attr("count", "4")
                .attr("id", "W1")
                .attr("t", "1"),
            Node::new("retry")
                .attr("count", "-1")
                .attr("id", "W1")
                .attr("t", "1"),
            Node::new("retry").attr("id", "").attr("t", "1"),
            Node::new("retry")
                .attr("count", "١")
                .attr("id", "W1")
                .attr("t", "1"),
            Node::new("retry").attr("id", "W1").attr("t", "١"),
        ] {
            assert!(
                received_confirmation(&base.clone().children(vec![
                    retry,
                    registration(),
                    Node::new("keys")
                ]))
                .is_none()
            );
        }
        let unsupported = base.children(vec![Node::new("retry").attr("v", "2"), registration()]);
        for (name, value) in [
            ("participant", "bad"),
            ("recipient", "bad"),
            ("id", ""),
            ("from", "bad"),
        ] {
            assert!(received_confirmation(&unsupported.clone().attr(name, value)).is_none());
        }
    }

    #[test]
    fn retry_schema_errors_use_fresh_native_context_in_parser_order() {
        let from = Jid::parse("15555550100:2@s.whatsapp.net").unwrap();
        let participant = Jid::parse("15555550101:3@s.whatsapp.net").unwrap();
        let base = Node::new("receipt")
            .attr("type", "retry")
            .attr("id", "ROOT")
            .attr("from", from.clone())
            .attr("participant", participant.clone())
            .attr("recipient", Jid::parse("123456789@lid").unwrap())
            .attr("edit", "7");
        let reg = || Node::new("registration").bytes(vec![1, 2, 3, 4]);
        let valid = || Node::new("retry").attr("id", "W1").attr("t", "1");
        let mut cases = vec![
            (vec![], "retry"),
            (vec![reg()], "retry"),
            (vec![valid()], "registration"),
            (vec![valid(), Node::new("registration")], "registration"),
            (
                vec![
                    valid(),
                    Node::new("registration").children(vec![Node::new("x")]),
                ],
                "registration",
            ),
            // Registration is checked before version/count, so this uses the
            // fresh parse-error context rather than the copied version context.
            (
                vec![
                    Node::new("retry").attr("v", "2").attr("count", "bad"),
                    Node::new("registration").bytes(vec![1]),
                ],
                "registration",
            ),
            (vec![Node::new("retry"), reg()], "retry.t"),
            (vec![Node::new("retry").attr("t", "1"), reg()], "retry.id"),
        ];
        for bytes in [vec![], vec![1, 2, 3], vec![1, 2, 3, 4, 5]] {
            cases.push((
                vec![valid(), Node::new("registration").bytes(bytes)],
                "registration",
            ));
        }
        for count in ["", "bad", " 1", "1.0", "2147483648", "-2147483649"] {
            cases.push((
                vec![Node::new("retry").attr("count", count), reg()],
                "retry.count",
            ));
        }
        for t in [
            "",
            "bad",
            " 1",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            cases.push((vec![Node::new("retry").attr("t", t), reg()], "retry.t"));
        }
        let expected = Node::new("ack")
            .attr("id", "ROOT")
            .attr("to", super::super::receipts::ack_address(from))
            .attr("class", "receipt")
            .attr("type", "retry")
            .attr(
                "participant",
                super::super::receipts::ack_address(participant),
            )
            .attr("error", "487");
        for (children, field) in cases {
            let node = base.clone().children(children);
            assert_eq!(
                retry_receipt_error(&node),
                Some(RetryReceiptError::Parsing(field))
            );
            let ack = received_confirmation(&node).unwrap();
            assert_eq!(ack, expected);
            let dict = TokenDictionary::comparison_2_26_37_71();
            let wire = Encoder::new(dict).encode(&ack).unwrap();
            assert_eq!(
                Decoder::new(&wire[1..], dict, &Limits::default())
                    .decode_node()
                    .unwrap(),
                expected
            );
        }
        // Java accepts signs and signed minimum; local count/id limits do not
        // justify classifying these as malformed input.
        for count in ["+1", "-1", "-2147483648", "2147483647"] {
            assert!(
                received_confirmation(&base.clone().children(
                    vec![valid().attr("count", count).attr("t", "-9223372036854775808"), reg()]
                ))
                .is_none()
            );
        }
    }

    /// `LX/Cp2->A05`: a receipt's ack mirrors the receipt — `to` its `from`,
    /// its `participant` and `recipient`, its `type` unless delivery, and its
    /// `edit` unless `"0"` — so group receipts are acknowledged too.
    #[test]
    fn receipt_acks_mirror_the_receipt_like_the_apk() {
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let member = Jid::parse("15555550100@lid").unwrap();
        let text = |node: &Node, name: &str| {
            node.get_attr(name)
                .and_then(|v| v.as_text())
                .map(str::to_owned)
        };
        let group_receipt = Node::new("receipt")
            .attr("from", group.clone())
            .attr("participant", member.clone())
            .attr("id", "R1");
        let ack = received_confirmation(&group_receipt).expect("a group receipt is acked");
        assert_eq!(ack.description, "ack");
        assert_eq!(ack.get_attr("to").unwrap().as_jid(), Some(&group));
        assert_eq!(ack.get_attr("participant").unwrap().as_jid(), Some(&member));
        assert_eq!(text(&ack, "class").as_deref(), Some("receipt"));
        assert_eq!(text(&ack, "id").as_deref(), Some("R1"));
        assert!(ack.get_attr("type").is_none(), "no type for a delivery");

        let read = group_receipt.clone().attr("type", "read");
        assert_eq!(
            text(&received_confirmation(&read).unwrap(), "type").as_deref(),
            Some("read")
        );
        let delivery = group_receipt.clone().attr("type", "delivery");
        assert!(
            received_confirmation(&delivery)
                .unwrap()
                .get_attr("type")
                .is_none()
        );

        let peer = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let with_recipient = Node::new("receipt")
            .attr("from", Jid::parse("15555550102:3@s.whatsapp.net").unwrap())
            .attr("recipient", peer.clone())
            .attr("id", "R2")
            .attr("edit", "0");
        let ack = received_confirmation(&with_recipient).expect("a device receipt is acked");
        assert_eq!(ack.get_attr("recipient").unwrap().as_jid(), Some(&peer));
        assert_eq!(ack.get_attr("to").unwrap().as_jid().unwrap().device, 3);
        assert!(ack.get_attr("edit").is_none(), "edit=0 is not copied");
        let edited = with_recipient.clone().attr("edit", "7");
        assert!(received_confirmation(&edited).is_some());

        // A type this client does not process is preserved, not acknowledged.
        let retry = group_receipt.attr("type", "retry").children(vec![
            Node::new("retry").attr("id", "W1").attr("t", "1"),
            Node::new("registration").bytes(vec![1, 2, 3, 4]),
        ]);
        assert!(received_confirmation(&retry).is_none());
    }

    #[test]
    fn binary_zero_primary_receipt_ack_uses_user_pairs_without_changing_capture() {
        let wire = b"\x00\xf8\x07\xfc\x07receipt\xfc\x04from\xf7\x01\x00\xfc\x06100001\xfc\x02id\xfc\x0aACK-ZERO-1\xfc\x0bparticipant\xf7\x00\x00\xfc\x0b15555550100";
        let decode = |wire: &[u8]| {
            Decoder::new(
                &wire[1..],
                TokenDictionary::comparison_2_26_37_71(),
                &Limits::default(),
            )
            .decode_node()
            .unwrap()
        };
        let node = decode(wire);
        let before = node.clone();
        for name in ["from", "participant"] {
            assert_eq!(
                node.get_attr(name).unwrap().as_jid().unwrap().kind,
                crate::model::jid::JidKind::AgentDevice
            );
        }
        let ack = received_confirmation(&node).unwrap();
        assert_eq!(node, before);
        assert_eq!(ack.get_attr("id").unwrap().as_text(), Some("ACK-ZERO-1"));
        assert_eq!(ack.get_attr("class").unwrap().as_text(), Some("receipt"));
        for name in ["to", "participant"] {
            let jid = ack.get_attr(name).unwrap().as_jid().unwrap();
            assert_eq!(jid.kind, crate::model::jid::JidKind::Plain, "{name}");
            assert_eq!((jid.agent, jid.device), (0, 0));
        }
        let output = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&ack)
            .unwrap();
        assert_eq!(decode(&output), ack);
        assert_eq!(
            ack.get_attr("to").unwrap().as_jid().unwrap(),
            &Jid::parse("100001@lid").unwrap()
        );
        assert_eq!(
            ack.get_attr("participant").unwrap().as_jid().unwrap(),
            &Jid::parse("15555550100@s.whatsapp.net").unwrap()
        );
    }

    #[test]
    fn receipt_ack_requires_user_recipient_but_keeps_generic_participant() {
        let base = Node::new("receipt")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "TYPED-RECIPIENT");
        for recipient in [
            "120363000000000001@g.us",
            "status@broadcast",
            "123@newsletter",
            "s.whatsapp.net",
            "123@call",
            "123@temp",
            "123@interop.g.us",
            "gdpr@s.whatsapp.net",
            "15555550100:2@s.whatsapp.net",
            "100001:0@lid",
            "15555550100:2@hosted",
            "100001:2@hosted.lid",
        ] {
            let node = base
                .clone()
                .attr("recipient", recipient)
                .attr("participant", recipient);
            let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
            let node = Decoder::new(
                &wire[1..],
                TokenDictionary::comparison_2_26_37_71(),
                &Limits::default(),
            )
            .decode_node()
            .unwrap();
            let ack = received_confirmation(&node).unwrap();
            assert!(ack.get_attr("recipient").is_none(), "{recipient}");
            assert!(ack.get_attr("participant").is_some(), "{recipient}");
        }
        for recipient in [
            "15555550100@s.whatsapp.net",
            "100001@lid",
            "15555550100@hosted",
            "100001@hosted.lid",
            "123@bot",
            "123@interop",
        ] {
            let node = base.clone().attr("recipient", recipient);
            assert!(
                received_confirmation(&node)
                    .unwrap()
                    .get_attr("recipient")
                    .is_some(),
                "{recipient}"
            );
        }
        let node = base.attr(
            "recipient",
            Jid::new_ad("100001", 0, 0, crate::model::jid::Server::Lid).unwrap(),
        );
        let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&node)
            .unwrap();
        let node = Decoder::new(
            &wire[1..],
            TokenDictionary::comparison_2_26_37_71(),
            &Limits::default(),
        )
        .decode_node()
        .unwrap();
        assert!(
            received_confirmation(&node)
                .unwrap()
                .get_attr("recipient")
                .is_some()
        );
        assert_eq!(
            received_confirmation(&node)
                .unwrap()
                .get_attr("recipient")
                .unwrap()
                .as_jid()
                .unwrap()
                .kind,
            crate::model::jid::JidKind::Plain
        );
    }

    #[test]
    fn receipt_ack_distinguishes_absent_type_and_native_namespace_failures() {
        let base = Node::new("receipt")
            .attr("from", Jid::parse("15555550100@s.whatsapp.net").unwrap())
            .attr("id", "ACK-FIXTURE");
        assert!(received_confirmation(&base).is_some());
        for node in [
            base.clone()
                .attr("type", Jid::parse("15555550100@s.whatsapp.net").unwrap()),
            base.clone().attr("from", "15555550100@unknown.example"),
            base.clone().attr("from", "unknown.example"),
            base.clone().attr("from", "lid"),
            base.clone().attr("from", "broadcast"),
            base.clone().attr("from", "@s.whatsapp.net"),
            base.clone().attr("from", "@g.us"),
            base.clone().attr("from", "@call"),
            base.clone().attr("from", "15555550100@c.us"),
            base.clone().attr("from", "15555550100@msgr"),
        ] {
            assert!(received_confirmation(&node).is_none());
        }
        for name in ["participant", "recipient"] {
            let ack =
                received_confirmation(&base.clone().attr(name, "15555550100@unknown.example"))
                    .unwrap();
            assert!(ack.get_attr(name).is_none());
            let ack = received_confirmation(&base.clone().attr(name, "lid")).unwrap();
            assert!(ack.get_attr(name).is_none());
            let ack = received_confirmation(&base.clone().attr(name, "@s.whatsapp.net")).unwrap();
            assert!(ack.get_attr(name).is_none());
        }
        for from in [
            "s.whatsapp.net",
            "g.us",
            "call",
            "status@broadcast",
            "15555550100@hosted",
            "222222222222222@hosted.lid",
            "15555550100:4@s.whatsapp.net",
        ] {
            assert!(
                received_confirmation(&base.clone().attr("from", from)).is_some(),
                "{from}"
            );
        }
    }

    /// `LX/1Rd->A01` builds exactly one `presence` element with only a `type`
    /// attribute. No `to`, no children — a stray attribute here goes out on a
    /// live account on every connect.
    #[test]
    fn available_presence_matches_the_traced_shape() {
        let node = available_presence();
        assert_eq!(node.description, "presence");
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("available"));
        assert_eq!(node.attributes.len(), 1, "presence carries only `type`");
        assert!(node.child_nodes().is_empty());
        assert!(node.get_attr("to").is_none());
        // It must survive the wire codec, since it is sent on every session.
        let dict = TokenDictionary::comparison_2_26_37_71();
        let encoded = Encoder::new(dict).encode(&node).unwrap();
        let decoded = Decoder::new(&encoded[1..], dict, &Limits::default())
            .decode_node()
            .unwrap();
        assert_eq!(decoded, node);
    }

    /// The exact shape that goes out on a live account for a group message,
    /// per comparison 2.26.37.71 `LX/Cp2->A00`: `to`=group, `id`, and
    /// `participant` (emitted only because it differs from `to`), with no
    /// `type` on a plain delivery receipt and no children.
    #[test]
    fn group_delivery_receipt_carries_the_participant_and_addresses_the_group() {
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let sender = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let message = Node::new("message")
            .attr("id", "grp1")
            .attr("from", group.clone())
            .attr("participant", sender.clone());
        let receipt = received_confirmation(&message).unwrap();
        assert_eq!(receipt.description, "receipt");
        assert_eq!(receipt.get_attr("to").unwrap().as_jid(), Some(&group));
        assert_eq!(receipt.get_attr("id").unwrap().as_text(), Some("grp1"));
        assert_eq!(
            receipt.get_attr("participant").unwrap().as_jid(),
            Some(&sender)
        );
        assert!(receipt.get_attr("type").is_none());
        assert!(receipt.get_attr("recipient").is_none());
        assert!(receipt.child_nodes().is_empty());
        // An edited group message is still refused.
        assert!(received_confirmation(&message.clone().attr("edit", "7")).is_none());
        // A group stanza with no participant is not a group message we can confirm.
        assert!(
            received_confirmation(&Node::new("message").attr("id", "x").attr("from", group))
                .is_none()
        );
        // A direct message never carries `participant`, so its receipt has none.
        let direct =
            received_confirmation(&Node::new("message").attr("id", "d1").attr("from", sender))
                .unwrap();
        assert!(direct.get_attr("participant").is_none());
    }
    #[test]
    fn ordinary_broadcast_list_confirmation_has_no_status_class() {
        for sender in ["15555550100@s.whatsapp.net", "100001@lid", "100001:7@lid"] {
            let incoming = Node::new("message")
                .attr("from", "12345@broadcast")
                .attr("participant", sender)
                .attr("id", "LIST-DELIVERY");
            let before = incoming.clone();
            let receipt = received_confirmation(&incoming).unwrap();
            assert_eq!(incoming, before);
            assert_eq!(receipt.attributes.len(), 3);
            assert!(receipt.get_attr("class").is_none());
            assert!(receipt.get_attr("type").is_none());
            assert!(receipt.get_attr("sts").is_none());
            assert_eq!(
                receipt
                    .get_attr("to")
                    .unwrap()
                    .as_jid()
                    .unwrap()
                    .to_string(),
                "12345@broadcast"
            );
            assert_eq!(
                receipt
                    .get_attr("participant")
                    .unwrap()
                    .as_jid()
                    .unwrap()
                    .user,
                Jid::parse(sender).unwrap().user
            );
            assert!(received_confirmation(&incoming.clone().attr("edit", "7")).is_none());
            assert!(received_confirmation(&incoming.attr("recipient", sender)).is_none());
        }
    }

    #[test]
    fn peer_category_uses_ordinary_receipts_even_with_status_metadata() {
        for chat in ["status@broadcast", "12345@broadcast"] {
            let incoming = Node::new("message")
                .attr("from", chat)
                .attr("participant", "100001:7@lid")
                .attr("id", "PEER-STATUS")
                .attr("category", "peer")
                .children(vec![
                    Node::new("meta").attr("content_type", "status_notify"),
                ]);
            let before = incoming.clone();
            let receipt = received_confirmation(&incoming).unwrap();
            assert_eq!(incoming, before);
            assert!(receipt.get_attr("class").is_none());
            assert!(receipt.get_attr("type").is_none());
            assert!(receipt.get_attr("sts").is_none());
            assert_eq!(receipt.attributes.len(), 3);
        }
    }

    #[test]
    fn broadcast_status_metadata_keeps_its_existing_class() {
        for (name, value, status) in [
            ("is_group_status", "true", true),
            ("content_type", "status_notify", true),
            ("is_group_status", "false", false),
            ("is_group_status", "TRUE", false),
            ("content_type", "add_on", false),
        ] {
            let incoming = Node::new("message")
                .attr("from", "12345@broadcast")
                .attr("participant", "100001@lid")
                .attr("id", "LIST-META")
                .children(vec![Node::new("meta").attr(name, value)]);
            let receipt = received_confirmation(&incoming).unwrap();
            assert_eq!(receipt.get_attr("class").is_some(), status);
        }
    }

    #[test]
    fn message_confirmation_uses_native_primary_device_wire_forms() {
        use crate::model::jid::{JidKind, Server};
        for (server, user, kind) in [
            (Server::User, "15555550100", JidKind::Plain),
            (Server::Lid, "100001", JidKind::AgentDevice),
        ] {
            let primary = Jid::parse(&format!("{user}@{}", server.as_str())).unwrap();
            for input in [
                primary.clone(),
                Jid::new_ad(user, 0, 0, server.clone()).unwrap(),
            ] {
                for chat in [
                    None,
                    Some("12345-67890@g.us"),
                    Some("status@broadcast"),
                    Some("12345@broadcast"),
                ] {
                    let mut incoming = Node::new("message").attr("id", "PRIMARY-DEVICE");
                    incoming = if let Some(chat) = chat {
                        incoming
                            .attr("from", chat)
                            .attr("participant", input.clone())
                    } else {
                        incoming.attr("from", input.clone())
                    };
                    let before = incoming.clone();
                    let receipt = received_confirmation(&incoming).unwrap();
                    assert_eq!(incoming, before);
                    let field = if chat.is_some() { "participant" } else { "to" };
                    let result = receipt.get_attr(field).unwrap().as_jid().unwrap();
                    assert_eq!(result.kind, kind, "{server:?} {chat:?} {input:?}");
                    assert_eq!(
                        (&result.user, result.agent, result.device, &result.server),
                        (&primary.user, 0, 0, &server)
                    );
                    let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                        .encode(&receipt)
                        .unwrap();
                    let decoded = Decoder::new(
                        &wire[1..],
                        TokenDictionary::comparison_2_26_37_71(),
                        &Limits::default(),
                    )
                    .decode_node()
                    .unwrap();
                    assert_eq!(decoded, receipt);
                }
            }
        }
    }

    #[test]
    fn status_sender_type_uses_saved_own_authority_and_keeps_attribute_order() {
        use super::super::{group_send, identity_notifications::tests as identities};
        let dir = tempfile::tempdir().unwrap();
        let store = crate::store::ProtocolStore::open(crate::store::OpenOptions {
            path: dir.path().join("status-ownership.redb"),
            account_id: crate::model::ids::AccountId::new("fixture").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([9; 32]),
            )),
        })
        .unwrap();
        let account = identities::account();
        store
            .write(|tx| group_send::record_own_lid(tx, &Jid::parse("100009@lid").unwrap()))
            .unwrap();
        for (participant, own) in [
            ("15555550999@s.whatsapp.net", true),
            ("15555550999:2@s.whatsapp.net", true),
            ("100009@lid", true),
            ("100009:2@lid", true),
            ("15555550100@s.whatsapp.net", false),
            ("100001@lid", false),
        ] {
            let incoming = Node::new("message")
                .attr("id", "OWN-STATUS")
                .attr("from", "status@broadcast")
                .attr("participant", participant)
                .attr("from_me", "true")
                .attr("sender_pn", "15555550999@s.whatsapp.net");
            let before = incoming.clone();
            let receipt = received_confirmation_for_account(&store, &account, &incoming)
                .unwrap()
                .unwrap();
            assert_eq!(incoming, before);
            assert_eq!(
                receipt.get_attr("type").and_then(|v| v.as_text()),
                own.then_some("sender")
            );
            let expected = if own {
                vec!["to", "id", "type", "participant", "class"]
            } else {
                vec!["to", "id", "participant", "class"]
            };
            assert_eq!(
                receipt
                    .attributes
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                receipt.get_attr("class").and_then(|v| v.as_text()),
                Some("status")
            );
            assert!(receipt.get_attr("sts").is_none());
        }
        store
            .write(|tx| {
                tx.put_json(
                    crate::store::Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"lid":"100008@lid"}),
                )
            })
            .unwrap();
        let incoming = Node::new("message")
            .attr("from", "status@broadcast")
            .attr("participant", "100009@lid")
            .attr("id", "CONFLICT");
        assert!(received_confirmation_for_account(&store, &account, &incoming).is_err());
        let peer = incoming.clone().attr("category", "peer");
        assert_eq!(
            received_confirmation_for_account(&store, &account, &peer).unwrap(),
            received_confirmation(&peer)
        );
        let direct = Node::new("message")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "DIRECT");
        assert_eq!(
            received_confirmation_for_account(&store, &account, &direct).unwrap(),
            received_confirmation(&direct)
        );
    }

    #[test]
    fn remote_status_receipt_has_status_class_without_sender_or_sts() {
        let status = Jid::parse("status@broadcast").unwrap();
        let poster = Jid::parse("15551234567:3@s.whatsapp.net").unwrap();
        let receipt = received_confirmation(
            &Node::new("message")
                .attr("id", "S1")
                .attr("from", status.clone())
                .attr("participant", poster.clone()),
        )
        .unwrap();
        assert_eq!(receipt.description, "receipt");
        assert_eq!(receipt.get_attr("to").unwrap().as_jid(), Some(&status));
        assert_eq!(receipt.get_attr("id").unwrap().as_text(), Some("S1"));
        assert!(
            receipt.get_attr("type").is_none(),
            "Cp2.A06 sender is fromMe only"
        );
        assert_eq!(receipt.get_attr("class").unwrap().as_text(), Some("status"));
        assert!(
            receipt.get_attr("sts").is_none(),
            "Cp2.A06 passes signed -1"
        );
        // The poster keeps its device-addressed form, as on the group path.
        assert_eq!(
            receipt.get_attr("participant").unwrap().as_jid(),
            Some(&poster)
        );
        assert!(receipt.get_attr("recipient").is_none());
        assert!(receipt.child_nodes().is_empty(), "no <meta> child");

        // A group receipt is still the group shape, not this one.
        let group = received_confirmation(
            &Node::new("message")
                .attr("id", "G1")
                .attr("from", Jid::parse("120363000000000001@g.us").unwrap())
                .attr("participant", poster),
        )
        .unwrap();
        assert!(group.get_attr("type").is_none());
        assert!(group.get_attr("class").is_none());
        assert!(group.get_attr("sts").is_none());

        // An edited status is refused, as an edited message is.
        assert!(
            received_confirmation(
                &Node::new("message")
                    .attr("id", "S2")
                    .attr("from", status)
                    .attr(
                        "participant",
                        Jid::parse("15551234567@s.whatsapp.net").unwrap()
                    )
                    .attr("edit", "7")
            )
            .is_none()
        );
    }

    /// `LX/Cp2->A06` passes the sender through `DeviceJid.Companion.A00`, which
    /// returns an existing device-addressed JID unchanged. So a device suffix on
    /// the incoming `participant` must survive into the receipt, rather than
    /// being stripped the way the sender-key storage key strips it.
    #[test]
    fn group_receipt_preserves_the_participant_device_suffix() {
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let device_addressed = Jid::parse("15551234567:3@s.whatsapp.net").unwrap();
        let receipt = received_confirmation(
            &Node::new("message")
                .attr("id", "grp2")
                .attr("from", group)
                .attr("participant", device_addressed.clone()),
        )
        .unwrap();
        let echoed = receipt.get_attr("participant").unwrap().as_jid().unwrap();
        assert_eq!(echoed, &device_addressed);
        assert_eq!(echoed.device, 3);
    }
    #[test]
    fn primary_connection_activation_matches_apk() {
        let node = active_request("offline-active");
        assert_eq!(node.description, "iq");
        for (key, value) in [
            ("id", "offline-active"),
            ("xmlns", "passive"),
            ("type", "set"),
        ] {
            assert_eq!(node.get_attr(key).unwrap().as_text(), Some(value));
        }
        assert_eq!(node.child_nodes(), &[Node::new("active")]);
        assert_eq!(
            node.get_attr("to"),
            key_request(&Jid::parse("15551234567@s.whatsapp.net").unwrap(), "test").get_attr("to")
        );
        let dict = TokenDictionary::comparison_2_26_37_71();
        let bytes = Encoder::new(dict).encode(&node).unwrap();
        assert_eq!(
            Decoder::new(&bytes[1..], dict, &Limits::default())
                .decode_node()
                .unwrap(),
            node
        );
    }
    #[test]
    fn image_envelope_and_media_route_roundtrip() {
        let jid = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let image = direct_image(&jid, "offline-image", "msg", vec![1, 2, 3]).unwrap();
        assert_eq!(image.get_attr("type").unwrap().as_text(), Some("media"));
        assert_eq!(
            image
                .child("enc")
                .unwrap()
                .get_attr("mediatype")
                .unwrap()
                .as_text(),
            Some("image")
        );
        let dict = TokenDictionary::comparison_2_26_37_71();
        for node in [image, media_route_request("offline-route")] {
            let bytes = Encoder::new(dict).encode(&node).unwrap();
            assert_eq!(
                Decoder::new(&bytes[1..], dict, &Limits::default())
                    .decode_node()
                    .unwrap(),
                node
            );
        }
        assert!(direct_image(&jid, "id", "plaintext", vec![1]).is_err());
    }
    #[test]
    fn private_json_accepts_exactly_one_text_or_image() {
        use std::io::Write;
        for (body, valid) in [
            (
                r#"{"to":"15551234567","request_id":"test","image":"/tmp/image.png","view_once":true}"#,
                true,
            ),
            (
                r#"{"to":"15551234567","request_id":"test","text":"hello","view_once":true}"#,
                false,
            ),
            (
                r#"{"to":"15551234567","request_id":"test","voice_note":"/tmp/beep.ogg"}"#,
                true,
            ),
            (
                r#"{"to":"15551234567","request_id":"test","voice_note":"/tmp/beep.ogg","image":"/tmp/image.png"}"#,
                false,
            ),
            (
                r#"{"to":"15551234567","request_id":"test","text":"hello"}"#,
                true,
            ),
            (
                r#"{"to":"15551234567","request_id":"test","image":"/tmp/image.png"}"#,
                true,
            ),
            (
                r#"{"to":"15551234567","request_id":"test","text":"hello","image":"/tmp/image.png"}"#,
                false,
            ),
            (r#"{"to":"15551234567","request_id":"test"}"#, false),
        ] {
            let mut file = tempfile::NamedTempFile::new().unwrap();
            file.write_all(body.as_bytes()).unwrap();
            assert_eq!(MessageRequest::read(file.path()).is_ok(), valid);
        }
    }
    #[test]
    fn key_query_roundtrip_uses_comparison_dictionary() {
        let recipient = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let node = key_request(&recipient, "offline-1");
        let dict = TokenDictionary::comparison_2_26_37_71();
        let bytes = Encoder::new(dict).encode(&node).unwrap();
        assert_eq!(
            Decoder::new(&bytes[1..], dict, &Limits::default())
                .decode_node()
                .unwrap(),
            node
        );
        assert_eq!(
            node.child("key")
                .unwrap()
                .child("user")
                .unwrap()
                .get_attr("jid")
                .unwrap()
                .as_jid(),
            Some(&recipient)
        );
    }

    #[test]
    fn identity_change_key_request_matches_transcribed_apk_and_device_bounds() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/prekeys/2.26.37.71-identity-query.json"
        ))
        .unwrap();
        let parse = |name: &str| {
            fixture[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Jid::parse(v.as_str().unwrap()).unwrap())
                .collect::<Vec<_>>()
        };
        let recipients = parse("recipients");
        let changed = parse("identity_changed");
        let node =
            key_request_many_with_identity_changes(&recipients, &changed, "fixture-query").unwrap();
        assert_eq!(node.description, "iq");
        assert_eq!(node.attributes.len(), 4);
        for (name, value) in fixture["iq"].as_object().unwrap() {
            let attr = node.get_attr(name).unwrap();
            let text = attr
                .as_text()
                .map(str::to_owned)
                .or_else(|| attr.as_jid().map(ToString::to_string))
                .unwrap();
            assert_eq!(text, value.as_str().unwrap());
        }
        assert_eq!(node.child_nodes().len(), 1);
        let key = node.child("key").unwrap();
        assert!(key.attributes.is_empty());
        assert_eq!(key.child_nodes().len(), 3);
        for (user, expected) in key
            .child_nodes()
            .iter()
            .zip(fixture["users"].as_array().unwrap())
        {
            assert_eq!(user.description, "user");
            assert_eq!(user.attributes.len(), expected.as_object().unwrap().len());
            assert_eq!(
                user.get_attr("jid").unwrap().as_jid().unwrap().to_string(),
                expected["jid"].as_str().unwrap()
            );
            assert_eq!(
                user.get_attr("reason").and_then(|v| v.as_text()),
                expected["reason"].as_str()
            );
        }
        let dict = TokenDictionary::comparison_2_26_37_71();
        let wire = Encoder::new(dict).encode(&node).unwrap();
        assert_eq!(
            Decoder::new(&wire[1..], dict, &Limits::default())
                .decode_node()
                .unwrap(),
            node
        );
        assert_eq!(
            key_request_many_with_identity_changes(&recipients, &[], "normal").unwrap(),
            key_request_many(&recipients, "normal").unwrap()
        );
        let encoded = Jid::new_ad(
            recipients[1].user.clone(),
            0,
            3,
            recipients[1].server.clone(),
        )
        .unwrap();
        assert_eq!(
            key_request_many_with_identity_changes(&recipients, &[encoded], "encoded")
                .unwrap()
                .child("key")
                .unwrap()
                .child_nodes()[1]
                .get_attr("reason")
                .and_then(|v| v.as_text()),
            Some("identity")
        );
        for other in [
            "15555550100:4@s.whatsapp.net",
            "15555550100@lid",
            "s.whatsapp.net",
        ] {
            let other = Jid::parse(other).unwrap();
            let error = key_request_many_with_identity_changes(
                &recipients,
                std::slice::from_ref(&other),
                "refuse",
            )
            .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("identity-change devices must belong")
            );
            assert!(!error.to_string().contains(&other.to_string()));
        }
        assert!(
            key_request_many_with_identity_changes(
                &recipients,
                &vec![changed[0].clone(); 33],
                "too-wide"
            )
            .is_err()
        );
    }

    /// A fan-out asks about every device it lacks a session with in one query.
    #[test]
    fn key_request_covers_several_devices_in_one_query() {
        let devices: Vec<Jid> = ["222222222222222@lid", "222222222222222:3@lid"]
            .iter()
            .map(|j| Jid::parse(j).unwrap())
            .collect();
        let node = key_request_many(&devices, "Q1").unwrap();
        assert_eq!(node.get_attr("xmlns").unwrap().as_text(), Some("encrypt"));
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("get"));
        let users = node.child("key").unwrap().child_nodes();
        assert_eq!(users.len(), 2);
        for (user, expected) in users.iter().zip(&devices) {
            assert_eq!(user.description, "user");
            assert_eq!(user.get_attr("jid").unwrap().as_jid(), Some(expected));
        }
        // The single-recipient form is the same query.
        assert_eq!(
            key_request(&devices[0], "Q1"),
            key_request_many(&devices[..1], "Q1").unwrap()
        );
        assert!(key_request_many(&[], "Q1").is_err());
        assert!(key_request_many(&vec![devices[0].clone(); 33], "Q1").is_err());
    }

    /// A LID-addressed chat encrypts under a LID Signal address, so the LID
    /// form has to be expressible on its own — otherwise it can only ever be
    /// exercised inside a group fan-out, where a failure has several possible
    /// causes.
    #[test]
    fn a_direct_recipient_may_be_digits_a_phone_jid_or_a_lid() {
        let to = |v: &str| MessageRequest {
            to: zeroize::Zeroizing::new(v.into()),
            text: zeroize::Zeroizing::new("hi".into()),
            image: None,
            voice_note: None,
            document: None,
            video: None,
            view_once: false,
            group: None,
            fresh_sessions: false,
            skdm_delivery: None,
            status: None,
            quote: None,
            request_id: "r1".into(),
        };
        assert_eq!(
            to("15551234567").recipient().unwrap(),
            Jid::parse("15551234567@s.whatsapp.net").unwrap()
        );
        assert_eq!(
            to("+15551234567").recipient().unwrap(),
            Jid::parse("15551234567@s.whatsapp.net").unwrap()
        );
        assert_eq!(
            to("15551234567@s.whatsapp.net").recipient().unwrap(),
            Jid::parse("15551234567@s.whatsapp.net").unwrap()
        );
        assert_eq!(
            to("111111111111111@lid").recipient().unwrap(),
            Jid::parse("111111111111111@lid").unwrap()
        );
        // A chat, a group, a device and a broadcast are not direct recipients.
        for bad in [
            "120363000000000001@g.us",
            "status@broadcast",
            "111111111111111:3@lid",
            "notdigits",
            "",
        ] {
            assert!(to(bad).recipient().is_err(), "{bad} must be refused");
        }
    }

    /// A reply and a reaction differ by exactly one thing in the JSON, and
    /// getting that wrong would send a reaction with a body or a reply without
    /// one — both of which a recipient renders as something else entirely.
    #[test]
    fn a_group_quote_is_a_reply_or_a_reaction_but_not_both() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, body: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, body).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
            path
        };
        let group = r#""group":"120363000000000001@g.us""#;
        let target = r#""message_id":"T1","participant":"222222222222222@lid""#;
        // A reply: text, no emoji.
        let reply = MessageRequest::read(&write(
            "reply.json",
            &format!(r#"{{{group},"text":"ok","quote":{{{target}}},"request_id":"r1"}}"#),
        ))
        .unwrap();
        assert!(!reply.quote.as_ref().unwrap().is_reaction());
        // A reaction: emoji, no text.
        let react = MessageRequest::read(&write(
            "react.json",
            &format!(r#"{{{group},"quote":{{{target},"emoji":"👍"}},"request_id":"r2"}}"#),
        ))
        .unwrap();
        assert!(react.quote.as_ref().unwrap().is_reaction());
        assert_eq!(
            react.quote.as_ref().unwrap().sender().unwrap(),
            Jid::parse("222222222222222@lid").unwrap()
        );
        // A media reply has an image or voice note as its one body. It reads
        // (outbox-send sends it); the comparison group path refuses media.
        for (name, body) in [
            ("reply-image.json", r#""image":"/tmp/reply.png""#),
            ("reply-voice.json", r#""voice_note":"/tmp/reply.ogg""#),
        ] {
            let media = MessageRequest::read(&write(
                name,
                &format!(r#"{{{group},{body},"quote":{{{target}}},"request_id":"r6"}}"#),
            ))
            .unwrap();
            assert!(!media.quote.as_ref().unwrap().is_reaction());
            assert!(media.require_comparison_support().is_err());
        }
        // A direct reply reads too (outbox-send); the comparison path refuses it.
        let direct = MessageRequest::read(&write(
            "direct-reply.json",
            &format!(
                r#"{{"to":"15551234567","text":"ok","quote":{{{target}}},"request_id":"r5"}}"#
            ),
        ))
        .unwrap();
        assert!(direct.require_comparison_support().is_err());
        assert!(reply.require_comparison_support().is_ok());
        // Both, neither, two bodies and a reaction with media are refused.
        for (name, body) in [
            (
                "both.json",
                format!(
                    r#"{{{group},"text":"ok","quote":{{{target},"emoji":"👍"}},"request_id":"r3"}}"#
                ),
            ),
            (
                "neither.json",
                format!(r#"{{{group},"quote":{{{target}}},"request_id":"r4"}}"#),
            ),
            (
                "two-bodies.json",
                format!(
                    r#"{{{group},"text":"ok","image":"/tmp/reply.png","quote":{{{target}}},"request_id":"r7"}}"#
                ),
            ),
            (
                "react-image.json",
                format!(
                    r#"{{{group},"image":"/tmp/reply.png","quote":{{{target},"emoji":"👍"}},"request_id":"r8"}}"#
                ),
            ),
        ] {
            assert!(
                MessageRequest::read(&write(name, &body)).is_err(),
                "{name} must be refused"
            );
        }
    }

    /// A group target is a JID in the private JSON, not another command.
    #[test]
    fn message_request_group_target_is_validated() {
        let request = |group: Option<&str>, to: &str| MessageRequest {
            to: zeroize::Zeroizing::new(to.into()),
            text: zeroize::Zeroizing::new("hi".into()),
            image: None,
            voice_note: None,
            document: None,
            video: None,
            view_once: false,
            group: group.map(str::to_owned),
            fresh_sessions: false,
            skdm_delivery: None,
            status: None,
            quote: None,
            request_id: "r1".into(),
        };
        assert_eq!(
            request(Some("120363000000000001@g.us"), "")
                .group_chat()
                .unwrap(),
            Some(Jid::parse("120363000000000001@g.us").unwrap())
        );
        assert_eq!(request(None, "15551234567").group_chat().unwrap(), None);
        assert_eq!(request(Some(""), "15551234567").group_chat().unwrap(), None);
        // A direct JID, a device-suffixed group and status are all refused here.
        assert!(
            request(Some("15551234567@s.whatsapp.net"), "")
                .group_chat()
                .is_err()
        );
        assert!(
            request(Some("120363000000000001:2@g.us"), "")
                .group_chat()
                .is_err()
        );
        assert!(request(Some("status@broadcast"), "").group_chat().is_err());
    }

    /// A group JSON carries no `to`; a direct one must still have digits.
    #[test]
    fn private_json_accepts_a_group_without_a_direct_recipient() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, body: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, body).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
            path
        };
        let group = MessageRequest::read(&write(
            "g.json",
            r#"{"group":"120363000000000001@g.us","text":"hi","request_id":"r1"}"#,
        ))
        .unwrap();
        assert!(group.to.is_empty());
        assert_eq!(
            group.group_chat().unwrap(),
            Some(Jid::parse("120363000000000001@g.us").unwrap())
        );
        // Both targets at once is ambiguous.
        assert!(
            MessageRequest::read(&write(
                "both.json",
                r#"{"to":"15551234567","group":"120363000000000001@g.us","text":"hi","request_id":"r2"}"#,
            ))
            .is_err()
        );
        // Group media is valid JSON (outbox-send), but the comparison group
        // path refuses it instead of sending an empty text.
        let voice = MessageRequest::read(&write(
            "voice.json",
            r#"{"group":"120363000000000001@g.us","voice_note":"/dev/null","request_id":"r3"}"#,
        ))
        .unwrap();
        assert!(voice.require_comparison_support().is_err());
        assert!(group.require_comparison_support().is_ok());
        // A direct send still needs digits.
        assert!(
            MessageRequest::read(&write("none.json", r#"{"text":"hi","request_id":"r4"}"#))
                .is_err()
        );
    }
}

#[cfg(test)]
mod message_ack_outcome_tests {
    use super::*;
    #[test]
    fn expected_chat_controls_comparison_ack_outcomes_without_replay() {
        let expected = Jid::parse("12345:7@lid").unwrap();
        let base = Node::new("ack")
            .attr("class", "message")
            .attr("id", "fixture");
        for (raw, outcome) in [
            (None, MessageAckOutcome::Accepted),
            (Some("500"), MessageAckOutcome::Refused),
            (Some("459"), MessageAckOutcome::AuthChallenge),
            (Some("bad"), MessageAckOutcome::Unrelated),
        ] {
            let mut ack = base.clone().attr("from", "12345:9@lid");
            if let Some(raw) = raw {
                ack = ack.attr("error", raw);
            }
            assert_eq!(message_ack_outcome_for(&ack, &expected), outcome);
            let wrong = ack.clone().attr("recipient", "67890@lid");
            assert_eq!(
                message_ack_outcome_for(&wrong, &expected),
                MessageAckOutcome::Unrelated
            );
            assert_eq!(
                message_ack_outcome_for(&wrong, &expected).status(),
                "outcome_unknown"
            );
            let preferred = ack.attr("from", "67890@lid").attr("recipient", "12345@lid");
            assert_eq!(message_ack_outcome_for(&preferred, &expected), outcome);
        }
        for participant in [
            "12345@lid",
            "12345:7@lid",
            "15555550100@s.whatsapp.net",
            "15555550100:7@s.whatsapp.net",
        ] {
            let ack = base
                .clone()
                .attr("from", "12345@lid")
                .attr("participant", participant);
            assert_eq!(
                message_ack_outcome_for(&ack, &expected),
                MessageAckOutcome::Unrelated
            );
            assert_eq!(
                message_ack_outcome_for(&ack, &expected).status(),
                "outcome_unknown"
            );
            assert_eq!(
                message_ack_outcome_for(&ack.clone().attr("error", "500"), &expected),
                MessageAckOutcome::Refused
            );
            assert_eq!(
                message_ack_outcome_for(&ack.attr("error", "459"), &expected),
                MessageAckOutcome::AuthChallenge
            );
        }
        for participant in ["12345-6@g.us", "not-a-jid"] {
            let ack = base
                .clone()
                .attr("from", "12345@lid")
                .attr("participant", participant);
            assert_eq!(
                message_ack_outcome_for(&ack, &expected),
                MessageAckOutcome::Accepted
            );
        }
        assert_eq!(
            message_ack_outcome_for(&base, &expected),
            MessageAckOutcome::Unrelated
        );
    }

    #[test]
    fn integer_error_admission_precedes_ordinary_ack_classification() {
        let message = Node::new("ack").attr("class", "message");
        assert_eq!(message_ack_outcome(&message), MessageAckOutcome::Accepted);
        for error in [
            "0",
            "+0",
            "-0",
            "-1",
            "2147483647",
            "-2147483648",
            "+４０３",
            "٤٠٣",
        ] {
            assert_eq!(
                message_ack_outcome(&message.clone().attr("error", error)),
                MessageAckOutcome::Refused,
                "{error}"
            );
            assert!(
                validate_received_ack(&message.clone().attr("id", "fixture").attr("error", error))
                    .is_ok()
            );
        }
        for error in [
            "",
            "bad",
            " 403",
            "403 ",
            "+",
            "-",
            "++403",
            "0x193",
            "403.0",
            "2147483648",
            "-2147483649",
            "9223372036854775808",
            "-9223372036854775809",
            "403\0",
        ] {
            let outcome = message_ack_outcome(&message.clone().attr("error", error));
            assert_eq!(outcome, MessageAckOutcome::Unrelated, "{error}");
            assert_eq!(outcome.status(), "outcome_unknown");
        }
        assert_eq!(
            message_ack_outcome(
                &message.attr("error", Jid::parse("15555550100@s.whatsapp.net").unwrap())
            ),
            MessageAckOutcome::Unrelated
        );
    }

    #[test]
    fn native_ack_family_and_challenge_diversion_control_comparison_status() {
        let message = Node::new("ack").attr("class", "message");
        assert_eq!(message_ack_outcome(&message), MessageAckOutcome::Accepted);
        assert_eq!(
            message_ack_outcome(&message.clone().attr("error", "500")),
            MessageAckOutcome::Refused
        );
        for class in ["message", "receipt", "notification", "call"] {
            let ack = Node::new("ack").attr("class", class);
            for error in ["459", "+459", "000459", "４５９"] {
                let outcome = message_ack_outcome(&ack.clone().attr("error", error));
                assert_eq!(outcome, MessageAckOutcome::AuthChallenge);
                assert_eq!(outcome.status(), "outcome_unknown");
                assert!(
                    validate_received_ack(&ack.clone().attr("id", "fixture").attr("error", error))
                        .is_ok()
                );
            }
            if class != "message" {
                assert_eq!(message_ack_outcome(&ack), MessageAckOutcome::Unrelated);
                assert_eq!(
                    message_ack_outcome(&ack.attr("error", "500")).status(),
                    "outcome_unknown"
                );
            }
        }
        assert_eq!(
            message_ack_outcome(&Node::new("ack")),
            MessageAckOutcome::Unrelated
        );
        assert_eq!(
            message_ack_outcome(
                &Node::new("iq")
                    .attr("class", "message")
                    .attr("error", "459")
            ),
            MessageAckOutcome::Unrelated
        );
    }
}

#[cfg(test)]
mod waited_reply_tests {
    use super::*;
    #[test]
    fn comparison_iq_wait_only_admits_native_result_error_replies() {
        for kind in [None, Some("get"), Some("set"), Some("Result"), Some("")] {
            let mut node = Node::new("iq").attr("id", "reserved-iq");
            if let Some(kind) = kind {
                node = node.attr("type", kind);
            }
            assert!(!waited_reply_matches(&node, "iq", "reserved-iq"));
        }
        for kind in ["result", "error"] {
            let node = Node::new("iq").attr("id", "reserved-iq").attr("type", kind);
            assert!(waited_reply_matches(&node, "iq", "reserved-iq"));
            assert!(!waited_reply_matches(&node, "iq", "another-id"));
        }
        // ACK content has its own classifier; receipt type is not an IQ type.
        assert!(waited_reply_matches(
            &Node::new("ack").attr("id", "reserved-iq"),
            "ack",
            "reserved-iq"
        ));
        assert!(!waited_reply_matches(
            &Node::new("message")
                .attr("id", "reserved-iq")
                .attr("type", "result"),
            "iq",
            "reserved-iq"
        ));
    }
}

#[cfg(test)]
mod captured_wait_tests {
    use super::*;
    use crate::store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table};

    #[test]
    fn call_ack_known_null_sender_precedes_class_field_checks() {
        for sender in [
            None,
            Some(""),
            Some("not-a-jid"),
            Some("@lid"),
            Some("1555@"),
            Some("1555@unknown.invalid"),
            Some("1555@S.WHATSAPP.NET"),
            Some("1555@c.us"),
            Some("1555@msgr"),
            Some("0@lid"),
            Some("0123@lid"),
            Some("letters@lid"),
            Some("1234567890123456@lid"),
            Some("１２３@lid"),
            Some("-1@hosted.lid"),
        ] {
            for id in ["", "fixture"] {
                for kind in [None, Some(""), Some("offer")] {
                    for error in [None, Some("0"), Some("500"), Some("459")] {
                        let mut ack = Node::new("ack").attr("class", "call").attr("id", id);
                        if let Some(sender) = sender {
                            ack = ack.attr("from", sender);
                        }
                        if let Some(kind) = kind {
                            ack = ack.attr("type", kind);
                        }
                        if let Some(error) = error {
                            ack = ack.attr("error", error);
                        }
                        assert!(validate_received_ack(&ack).is_ok());
                        let mut missing_id = ack.clone();
                        missing_id.attributes.retain(|attr| attr.name != "id");
                        assert!(validate_received_ack(&missing_id).is_err());
                        assert!(validate_received_ack(&ack.attr("error", "bad")).is_err());
                    }
                }
            }
        }
    }

    #[test]
    fn lid_device_constructor_controls_call_field_ordering_without_u16_guessing() {
        for (sender, nonnull) in [
            ("12345:0@lid", true),
            ("12345.foo@lid", true),
            ("12345.:7@lid", true),
            ("12345:99@lid", true),
            ("12345:100@lid", false),
            ("12345:128@lid", false),
            ("12345:255@lid", false),
            ("12345:256@lid", true),
            ("12345:355@lid", true),
            ("12345:356@lid", false),
            ("12345:-256@lid", true),
            ("12345:-1@lid", false),
            ("12345:+7@lid", true),
            ("12345:２５６@lid", true),
            ("12345:2147483647@lid", false),
            ("12345:-2147483648@lid", true),
            ("12345:2147483648@lid", false),
            ("12345:99@hosted.lid", true),
            ("12345:355@hosted.lid", false),
            ("12345:7@hosted.lid", false),
            ("12345.foo@hosted.lid", false),
            ("12345:7.foo@lid", false),
            ("12345..foo@lid", false),
            ("0:7@lid", false),
            ("0123:7@lid", false),
            ("12345:@lid", false),
            ("12345.@lid", false),
            ("12345:bad@lid", false),
        ] {
            let ack = Node::new("ack")
                .attr("class", "call")
                .attr("id", "")
                .attr("from", sender);
            assert_eq!(validate_received_ack(&ack).is_ok(), !nonnull, "{sender}");
            assert!(validate_received_ack(&ack.clone().attr("error", "459")).is_ok());
            assert!(validate_received_ack(&ack.attr("error", "bad")).is_err());
        }
    }

    #[test]
    fn pn_text_constructor_controls_call_ordering_and_specials_bypass_numeric_grammar() {
        for domain in ["s.whatsapp.net", "hosted"] {
            for (local, valid) in [
                ("1234", false),
                ("12345", true),
                ("12345678901234567890", true),
                ("123456789012345678901", false),
                ("01234", false),
                ("10123", false),
                ("10080", true),
                ("1008", false),
                ("１２３４５", false),
                ("١٢٣٤٥", false),
                ("+12345", false),
                ("12345:0", domain != "hosted"),
                ("12345:256", domain != "hosted"),
                ("12345:-256", domain != "hosted"),
                ("12345:99", true),
                ("12345:355", domain != "hosted"),
                ("12345:100", false),
                ("12345:2147483648", false),
                ("12345.foo", domain != "hosted"),
                ("12345:7.foo", false),
            ] {
                let ack = Node::new("ack")
                    .attr("class", "call")
                    .attr("id", "")
                    .attr("from", format!("{local}@{domain}"));
                assert_eq!(
                    validate_received_ack(&ack).is_ok(),
                    !valid,
                    "{local}@{domain}"
                );
                assert!(validate_received_ack(&ack.clone().attr("error", "459")).is_ok());
                assert!(validate_received_ack(&ack.attr("error", "bad")).is_err());
            }
        }
    }

    #[test]
    fn broadcast_call_sender_constructor_and_subtypes_control_ordering() {
        for (text, valid) in [
            ("1@broadcast", true),
            ("12345678901234567890@broadcast", true),
            ("123456789012345678901@broadcast", false),
            ("0@broadcast", false),
            ("01@broadcast", false),
            ("letters@broadcast", false),
            ("status@broadcast", true),
            ("location@broadcast", true),
            ("Status@broadcast", false),
            ("location:0@broadcast", false),
            ("1.2@broadcast", false),
            ("1:0@broadcast", false),
            ("１２３@broadcast", false),
            (" 1@broadcast", false),
            ("@broadcast", false),
        ] {
            let ack = Node::new("ack")
                .attr("class", "call")
                .attr("id", "fixture")
                .attr("type", "offer")
                .attr("from", text);
            assert_eq!(validate_received_ack(&ack).is_err(), valid, "{text}");
            assert_eq!(call_ack_sender_known_null(&ack), !valid, "{text}");
            assert!(validate_received_ack(&ack.clone().attr("error", "459")).is_ok());
            assert!(validate_received_ack(&ack.attr("error", "bad")).is_err());
        }
    }

    #[test]
    fn group_and_newsletter_call_sender_constructors_precede_later_fields() {
        for (text, valid) in [
            ("1@g.us", true),
            ("12345678901234567890@g.us", true),
            ("123456789012345678901@g.us", false),
            ("0@g.us", false),
            ("01@g.us", false),
            ("abc@g.us", false),
            ("１２３@g.us", false),
            ("12345-1234567890@g.us", true),
            ("10080-1234567890@g.us", true),
            ("10123-1234567890@g.us", false),
            ("1234-1234567890@g.us", false),
            ("12345-0234567890@g.us", false),
            ("12345-123456789@g.us", false),
            ("12345-1234567890-1@g.us", false),
            ("123:0@g.us", false),
            ("1@newsletter", true),
            ("letters@newsletter", true),
            ("-1@newsletter", true),
            ("１２３@newsletter", true),
            (" 1 @newsletter", true),
            ("\u{a0}@newsletter", true),
            ("@newsletter", false),
            ("0@newsletter", false),
            ("01@newsletter", false),
            (" 0 @newsletter", false),
            ("\t\n@newsletter", false),
            ("1:0@newsletter", true),
        ] {
            let ack = Node::new("ack")
                .attr("class", "call")
                .attr("id", "fixture")
                .attr("type", "offer")
                .attr("from", text);
            assert_eq!(validate_received_ack(&ack).is_err(), valid, "{text:?}");
            assert_eq!(call_ack_sender_known_null(&ack), !valid, "{text:?}");
            assert!(validate_received_ack(&ack.clone().attr("error", "459")).is_ok());
            assert!(validate_received_ack(&ack.attr("error", "bad")).is_err());
        }
    }

    #[test]
    fn special_pn_devices_preserve_constructor_class_and_call_ordering() {
        for user in ["0", "Server"] {
            for domain in ["s.whatsapp.net", "hosted"] {
                for (suffix, native_device) in [
                    (":0", domain != "hosted"),
                    (".foo", domain != "hosted"),
                    (":99", true),
                    (":100", false),
                    (":255", false),
                    (":256", domain != "hosted"),
                    (":355", domain != "hosted"),
                    (":-256", domain != "hosted"),
                    (":２５６", domain != "hosted"),
                    (":2147483648", false),
                    (":", false),
                    (".", false),
                    (":7.foo", false),
                    (".foo:99", true),
                ] {
                    let text = format!("{user}{suffix}@{domain}");
                    let ack = Node::new("ack")
                        .attr("class", "call")
                        .attr("id", "fixture")
                        .attr("type", "offer")
                        .attr("from", text.clone());
                    assert_eq!(
                        ordinary_ack_has_participant(
                            &Node::new("ack").attr("participant", text.clone())
                        ),
                        native_device,
                        "{text}"
                    );
                    assert!(validate_received_ack(&ack).is_ok(), "valid fields {text}");
                    let mut empty_id = ack.clone();
                    empty_id.attributes.retain(|a| a.name != "id");
                    empty_id = empty_id.attr("id", "");
                    assert_eq!(
                        validate_received_ack(&empty_id).is_ok(),
                        !native_device,
                        "{text}"
                    );
                    assert!(validate_received_ack(&empty_id.clone().attr("error", "459")).is_ok());
                    assert!(validate_received_ack(&empty_id.attr("error", "bad")).is_err());
                    assert_eq!(call_ack_sender_known_null(&ack), !native_device, "{text}");
                }
            }
        }
    }

    #[test]
    fn plain_special_pn_call_sender_types_are_rejected_after_challenge_diversion() {
        for sender in [
            "0@s.whatsapp.net",
            "0@hosted",
            "Server@s.whatsapp.net",
            "Server@hosted",
            "gdpr@s.whatsapp.net",
        ] {
            let ack = Node::new("ack")
                .attr("class", "call")
                .attr("id", "fixture")
                .attr("type", "offer")
                .attr("from", sender);
            assert!(!call_ack_sender_known_null(&ack));
            assert!(validate_received_ack(&ack).is_err(), "{sender}");
            assert!(validate_received_ack(&ack.clone().attr("error", "459")).is_ok());
            assert!(validate_received_ack(&ack.clone().attr("error", "500")).is_err());
            let mut ack = ack;
            ack.attributes.retain(|a| a.name != "class");
            assert!(validate_received_ack(&ack.attr("class", "message")).is_ok());
        }
    }

    #[test]
    fn plain_lid_sender_constructor_controls_call_field_ordering() {
        for domain in ["lid", "hosted.lid"] {
            for (user, valid) in [
                ("", false),
                ("0", false),
                ("0123", false),
                ("1", true),
                ("12345", true),
                ("123456789012345", true),
                ("1234567890123456", false),
                ("-1", false),
                ("１２３", false),
                ("١٢٣", false),
                (" 1", false),
                ("1 ", false),
            ] {
                let ack = Node::new("ack")
                    .attr("class", "call")
                    .attr("id", "")
                    .attr("from", format!("{user}@{domain}"));
                assert_eq!(
                    validate_received_ack(&ack).is_ok(),
                    !valid,
                    "{user}@{domain}"
                );
                assert!(validate_received_ack(&ack.clone().attr("error", "459")).is_ok());
                assert!(validate_received_ack(&ack.attr("error", "bad")).is_err());
            }
        }
    }

    #[test]
    fn call_ack_bare_sender_factory_results_are_not_null_guesses() {
        for sender in ["s.whatsapp.net", "g.us", "call", "status_me", "lid_me"] {
            let ack = Node::new("ack")
                .attr("class", "call")
                .attr("from", sender)
                .attr("id", "fixture");
            assert!(!call_ack_sender_known_null(&ack));
            assert!(
                validate_received_ack(&ack).is_err(),
                "nonnull sender requires type"
            );
            let ack = ack.attr("type", "offer");
            // Cache-dependent bare sender subtypes are not classified here.
            if sender == "call" {
                assert!(validate_received_ack(&ack).is_ok());
            }
            assert!(validate_received_ack(&ack.attr("error", "459")).is_ok());
        }
    }

    #[test]
    fn call_ack_requires_nonempty_text_fields_after_auth_challenge_diversion() {
        let base = Node::new("ack")
            .attr("class", "call")
            .attr("from", "15555550100@s.whatsapp.net");
        for node in [
            base.clone().attr("id", "").attr("type", "offer"),
            base.clone().attr("id", "fixture"),
            base.clone().attr("id", "fixture").attr("type", ""),
        ] {
            let error = validate_received_ack(&node).unwrap_err();
            assert!(!error.is_recoverable_by_reconnect());
            for raw in ["459", "+459", "４５９"] {
                assert!(validate_received_ack(&node.clone().attr("error", raw)).is_ok());
            }
            for raw in ["0", "500", "-1"] {
                assert!(validate_received_ack(&node.clone().attr("error", raw)).is_err());
            }
            for class in ["message", "receipt", "notification", "Call", "unknown"] {
                assert!(validate_received_ack(&node.clone().attr("class", class)).is_ok());
            }
        }
        for from in ["123456-1234567890@g.us", "123456@newsletter"] {
            let ack = base
                .clone()
                .attr("id", "fixture")
                .attr("type", "offer")
                .attr("from", from);
            assert!(validate_received_ack(&ack).is_err());
            assert!(validate_received_ack(&ack.attr("error", "459")).is_ok());
        }
        // Native String.length, not trim/semantic call-type recognition.
        for kind in ["offer", "accept", " ", "unknown"] {
            assert!(
                validate_received_ack(&base.clone().attr("id", "fixture").attr("type", kind))
                    .is_ok()
            );
        }
        assert!(
            validate_received_ack(&base.attr("error", "459")).is_err(),
            "required id still precedes459"
        );
    }

    #[test]
    fn ack_required_id_precedes_challenge_and_class_routing() {
        for class in [None, Some("message"), Some("receipt"), Some("unknown")] {
            for error in [None, Some("459"), Some("bad")] {
                let mut ack = Node::new("ack");
                if let Some(class) = class {
                    ack = ack.attr("class", class);
                }
                if let Some(error) = error {
                    ack = ack.attr("error", error);
                }
                let failure = validate_received_ack(&ack).unwrap_err();
                assert!(failure.to_string().contains("missing ACK id"));
                assert!(!failure.is_recoverable_by_reconnect());
                if error != Some("bad") {
                    // A0L rejects null, not the empty string.
                    assert!(validate_received_ack(&ack.attr("id", "")).is_ok());
                }
            }
        }
        assert!(validate_received_ack(&Node::new("notification")).is_ok());
    }

    #[tokio::test]
    async fn captured_wait_stops_on_malformed_ack_before_matching_or_next_read() {
        use sha2::Digest;
        for (matched, missing_id, call_type) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (true, false, true),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("bad-ack-wait.redb");
            let open = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: crate::model::ids::AccountId::new("fixture").unwrap(),
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [9; 32],
                    ))),
                    create: true,
                })
                .unwrap()
            };
            let store = open();
            let intent = serde_json::json!({"status":"outcome_unknown","wire_id":"held"});
            store
                .write(|tx| tx.put_json(Table::Checkpoints, b"send-fixture", 1, &intent))
                .unwrap();
            let bad = if call_type {
                Node::new("ack")
                    .attr("class", "call")
                    .attr("id", "held")
                    .attr("from", "15555550100@s.whatsapp.net")
                    .attr("type", "")
            } else if missing_id {
                Node::new("ack")
                    .attr("class", "message")
                    .attr("error", "459")
            } else {
                Node::new("ack")
                    .attr("class", "message")
                    .attr("id", if matched { "held" } else { "unrelated" })
                    .attr("error", "bad")
            };
            let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&bad)
                .unwrap();
            let key = format!(
                "inbound:sha256:{}",
                hex::encode(sha2::Sha256::digest(&wire))
            );
            let mut nodes: std::collections::VecDeque<_> = [
                bad,
                Node::new("ack").attr("class", "message").attr("id", "held"),
            ]
            .into();
            let reads = std::cell::Cell::new(0);
            let result: crate::Result<_> = wait_for_observed(
                async || {
                    reads.set(reads.get() + 1);
                    Ok(nodes.pop_front().unwrap())
                },
                |node| preserve_waited_node(&store, node),
                "ack",
                "held",
                tokio::time::Instant::now() + Duration::from_secs(1),
            )
            .await;
            let error = result.unwrap_err();
            assert!(matches!(
                error,
                crate::Error::InvalidInput { field: "ack", .. }
            ));
            assert!(!error.is_recoverable_by_reconnect());
            assert_eq!(reads.get(), 1);
            assert_eq!(nodes.len(), 1);
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"send-fixture",
                        1
                    ))
                    .unwrap(),
                Some(intent.clone())
            );
            drop(store);
            let reopened = open();
            assert!(
                reopened
                    .read(|tx| tx.get(Table::Checkpoints, key.as_bytes(), 1))
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                reopened
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"send-fixture",
                        1
                    ))
                    .unwrap(),
                Some(intent)
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn decoded_wait_captures_survive_each_exit_and_reopen_without_settlement() {
        for mode in [
            "success",
            "timeout",
            "read_error",
            "terminal",
            "count",
            "capture_error",
            "undecodable",
            "compression",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("wait.redb");
            let open = || {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: crate::model::ids::AccountId::new("fixture").unwrap(),
                    create: true,
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [9; 32],
                    ))),
                })
                .unwrap()
            };
            let store = open();
            let intent = serde_json::json!({"status":"outcome_unknown","wire_id":"reserved-iq","wire":[1,2,3]});
            store
                .write(|tx| tx.put_json(Table::Checkpoints, b"message:fixture:send", 1, &intent))
                .unwrap();
            let mut nodes: std::collections::VecDeque<_> = if mode == "count" {
                (0..32)
                    .map(|i| Node::new("receipt").attr("id", i.to_string()))
                    .collect()
            } else {
                [
                    Node::new("iq")
                        .attr("id", "reserved-iq")
                        .attr("type", "get"),
                    Node::new("receipt").attr("id", "unrelated"),
                ]
                .into()
            };
            if mode == "success" || mode == "capture_error" {
                nodes.push_back(
                    Node::new("iq")
                        .attr("id", "reserved-iq")
                        .attr("type", "result"),
                );
            } else if mode == "terminal" {
                nodes.push_back(Node::new("stream:error"));
            }
            let raw = if mode == "compression" {
                vec![2, 0] // Compressed stanza flag followed by invalid zlib data.
            } else {
                vec![
                    0,
                    crate::binary::tags::LIST_8,
                    1,
                    crate::binary::tags::JID_PAIR,
                ]
            };
            let raw_class = super::super::inbox::decode_frame(&raw).unwrap_err().class();
            let mut captures = Vec::new();
            let result: crate::Result<_> = wait_for_observed(
                async || {
                    if let Some(node) = nodes.pop_front() {
                        return Ok(node);
                    }
                    if mode == "timeout" {
                        std::future::pending().await
                    } else if matches!(mode, "undecodable" | "compression") {
                        captured_wait_node(
                            &store,
                            FrameRead::Undecodable {
                                frame: raw.clone(),
                                class: raw_class,
                            },
                        )
                    } else {
                        Err(TransportError::Closed.into())
                    }
                },
                |node| {
                    if mode == "capture_error" && captures.len() == 1 {
                        return Err(crate::Error::invalid("capture", "injected capture failure"));
                    }
                    captures.push(super::super::inbox::preserve(&store, node)?);
                    Ok(())
                },
                "iq",
                "reserved-iq",
                tokio::time::Instant::now() + Duration::from_secs(30),
            )
            .await;
            if mode == "success" {
                let (reply, pending) = result.unwrap();
                assert_eq!(
                    reply.get_attr("type").and_then(|v| v.as_text()),
                    Some("result")
                );
                assert_eq!(pending.len(), 2);
            } else {
                assert!(result.is_err(), "{mode}");
            }
            assert_eq!(
                captures.len(),
                match mode {
                    "count" => 32,
                    "success" | "terminal" => 3,
                    "capture_error" => 1,
                    _ => 2,
                }
            );
            if mode == "capture_error" {
                assert_eq!(
                    nodes.len(),
                    1,
                    "capture failure must stop before reading the matching response"
                );
            }
            drop(store);
            let store = open();
            let unreadable = super::super::inbox::undecodable_frames(&store).unwrap();
            if matches!(mode, "undecodable" | "compression") {
                assert_eq!(unreadable.len(), 1);
                assert_eq!(unreadable[0].class, raw_class);
                assert_eq!(unreadable[0].bytes, raw.len());
                assert_eq!(unreadable[0].epoch, None);
                let saved: serde_json::Value = store
                    .read(|tx| tx.get_json(Table::Checkpoints, unreadable[0].capture.as_bytes(), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(saved["frame"], serde_json::json!(raw));
            } else {
                assert!(unreadable.is_empty());
            }
            for key in captures {
                assert!(
                    store
                        .read(|tx| tx.get(Table::Checkpoints, &key, 1))
                        .unwrap()
                        .is_some(),
                    "{mode}"
                );
            }
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        b"message:fixture:send",
                        1
                    ))
                    .unwrap(),
                Some(intent)
            );
        }
    }
}

#[cfg(test)]
mod prewrite_deadline_tests {
    use super::*;
    #[tokio::test(start_paused = true)]
    async fn expired_request_deadline_is_not_restarted_when_reply_wait_begins() {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        // Model time consumed by the already-reserved request's write.
        tokio::time::advance(Duration::from_secs(31)).await;
        let reads = std::cell::Cell::new(0);
        let result: Result<_, TransportError> = wait_for_observed(
            async || {
                reads.set(reads.get() + 1);
                Ok(Node::new("iq")
                    .attr("id", "reserved")
                    .attr("type", "result"))
            },
            |_| Ok(()),
            "iq",
            "reserved",
            deadline,
        )
        .await;
        assert!(matches!(
            result,
            Err(TransportError::Deadline("correlated stanza"))
        ));
        assert_eq!(
            reads.get(),
            0,
            "expired requests cannot consume a queued late reply"
        );
    }
}
