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
            if value.voice_note.is_some() || value.view_once {
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
            if value.group_chat()?.is_none() {
                return Err(crate::Error::invalid(
                    "message",
                    "a quote or reaction needs the `group` it belongs to",
                ));
            }
            quote.sender()?;
            if quote.message_id.is_empty() || quote.message_id.len() > 128 {
                return Err(crate::Error::invalid(
                    "message",
                    "invalid quoted message id",
                ));
            }
            // A reaction has no body; a reply must have one.
            if quote.is_reaction() != value.text.is_empty() {
                return Err(crate::Error::invalid(
                    "message",
                    "a reaction carries no text and a reply needs one",
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
            Some(_) => {
                if value.image.is_some() || value.voice_note.is_some() || value.view_once {
                    return Err(crate::Error::invalid(
                        "message",
                        "a group send carries text only so far",
                    ));
                }
            }
            None => {
                value.recipient()?;
            }
        }
        // A reaction is the one message with no content of its own: it names an
        // existing message and an emoji, and a body would make it a reply.
        let reacting = value.quote.as_ref().is_some_and(|q| q.is_reaction());
        let bodies = usize::from(!value.text.is_empty())
            + usize::from(value.image.is_some())
            + usize::from(value.voice_note.is_some());
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
        if jid.server != crate::model::jid::Server::Group || jid.device != 0 {
            return Err(crate::Error::invalid(
                "message",
                "group must be a `…@g.us` JID",
            ));
        }
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
/// Sending one asks for a repeat of a message we already hold; it acknowledges
/// nothing and confirms no delivery.
pub fn retry_receipt(
    chat: &Jid,
    participant: Option<&Jid>,
    message_id: &str,
    count: u32,
    registration_id: u32,
    now_seconds: i64,
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
    Ok(node.children(vec![
        Node::new("retry")
            .attr("count", count.to_string().as_str())
            .attr("id", message_id)
            .attr("t", now_seconds.to_string().as_str())
            .attr("v", "1"),
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
/// Every real client announces availability after connecting and this one never
/// did, which made it the last normal post-login behaviour we lacked. It is a
/// fire-and-forget stanza, not an `iq`, so there is no reply to correlate.
pub fn available_presence() -> Node {
    Node::new("presence").attr("type", "available")
}

/// Primary delivery receipt for a direct or group message, or a receipt
/// acknowledgement. Never a read/played/opened receipt. Caller must durably
/// store the incoming frame and authenticate messages before acknowledging them.
///
/// Group form from comparison 2.26.37.71 `LX/Cp2->A00` (**confirmed**): `to` and
/// `id` always; `participant` exactly when it is non-null and differs from `to`
/// (so a direct chat, where the two coincide, never carries it); `type` omitted
/// for a plain delivery receipt; `recipient` and `sts` only when their inputs are
/// set, which they are not here.
///
/// The participant keeps the incoming stanza's own device-addressed form
/// (**confirmed**): `LX/Cp2->A06` passes `Ayl.A0V(CjN.A04())` — the message's
/// sender JID through `DeviceJid.Companion.A00` — as the value `LX/CmC->A06`
/// returns as the pair's second element. That helper returns an existing
/// `DeviceJid` unchanged and only coerces a bare `UserJid` to its primary
/// device, which renders identically. So there is no device stripping and no
/// PN/LID conversion on this path, and echoing the attribute verbatim matches.
pub fn received_confirmation(incoming: &Node) -> Option<Node> {
    if incoming.description == "receipt" {
        return receipt_ack(incoming);
    }
    if incoming.description != "message" || incoming.get_attr("recipient").is_some() {
        return None;
    }
    // Status / broadcast: a different receipt shape from an ordinary message.
    // Comparison 2.26.37.71 `LX/Cp2->A06` true branch (**confirmed**): it passes
    // `type="sender"`, an extra `class="status"` attribute, a null recipient, a
    // zero `<meta mode>` int and the long `0xffff`. `A00` emits `sts` whenever
    // that long is > 0, so a status receipt carries the literal `sts="65535"` —
    // a hardcoded constant, not a timestamp. The `<meta mode>` child is *not*
    // emitted: its gate tests the mode int first, which is zero, so the type-set
    // membership check is never reached. Attribute order follows `A00`'s append
    // order: to, id, type, participant, recipient, sts, then the extra class.
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
        return Some(
            Node::new("receipt")
                .attr("to", chat)
                .attr("id", id)
                .attr("type", "sender")
                .attr("participant", participant)
                .attr("sts", "65535")
                .attr("class", "status"),
        );
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
                .attr("participant", participant),
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
    Some(Node::new("receipt").attr("to", jid).attr("id", id))
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
/// read and played. The APK acknowledges every type after processing it; an
/// ack for a type we do not process would tell the server it was handled when
/// it was only preserved, so those stay unacknowledged.
fn receipt_ack(incoming: &Node) -> Option<Node> {
    let jid_of = |name: &str| {
        let value = incoming.get_attr(name)?;
        value
            .as_jid()
            .cloned()
            .or_else(|| Jid::parse(value.as_text()?).ok())
    };
    let from = jid_of("from")?;
    let id = incoming.get_attr("id")?.as_text()?;
    if id.is_empty() || id.len() > 128 {
        return None;
    }
    let mut ack = Node::new("ack")
        .attr("id", id)
        .attr("to", from)
        .attr("class", "receipt");
    match incoming.get_attr("type").and_then(|v| v.as_text()) {
        None | Some("delivery") => {}
        Some(kind @ ("read" | "played")) => {
            ack = ack.attr("type", kind);
        }
        _ => return None,
    }
    if let Some(participant) = jid_of("participant") {
        ack = ack.attr("participant", participant);
    }
    if let Some(recipient) = jid_of("recipient") {
        ack = ack.attr("recipient", recipient);
    }
    if let Some(edit) = incoming
        .get_attr("edit")
        .and_then(|v| v.as_text())
        .filter(|e| !e.is_empty() && *e != "0")
    {
        ack = ack.attr("edit", edit);
    }
    Some(ack)
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

/// Direct primary-device message envelope; encrypted payload only.
pub fn direct_message(
    recipient: &Jid,
    id: &str,
    kind: &str,
    ciphertext: Vec<u8>,
) -> Result<Node, TransportError> {
    direct_typed(recipient, id, kind, ciphertext, "text")
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
    if !matches!(media_type, "image" | "ptt") {
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

/// Wait for an exact stanza/id pair, with a whole-operation deadline and count
/// bound. No resend. Nonmatching frames are returned for encrypted persistence.
pub async fn wait_for<S: AsyncRead + AsyncWrite + Unpin>(
    channel: &mut EncryptedConnection<S>,
    description: &str,
    id: &str,
) -> Result<(Node, Vec<Node>), TransportError> {
    tokio::time::timeout(Duration::from_secs(30), async {
        let mut pending = Vec::new();
        for _ in 0..32 {
            let node = read_node(channel).await?;
            if node.description == description
                && node.get_attr("id").and_then(|v| v.as_text()) == Some(id)
            {
                return Ok((node, pending));
            }
            if matches!(node.description.as_str(), "failure" | "stream:error") {
                return Err(error("server ended session while awaiting response"));
            }
            pending.push(node);
        }
        Err(error("stanza count exceeded while awaiting response"))
    })
    .await
    .map_err(|_| TransportError::Deadline("correlated stanza"))?
}

#[cfg(test)]
mod retry_send_tests {
    use super::*;

    /// The shape is mirrored from the receipts the operator's handset sends us,
    /// and the registration id is the field that lets the sender decide between
    /// reusing their session and fetching a fresh bundle.
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
        assert!(received_confirmation(&receipt.clone().attr("type", "retry")).is_none());
        assert!(
            received_confirmation(&receipt.attr("type", "delivery"))
                .unwrap()
                .get_attr("type")
                .is_none()
        );
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
        let retry = group_receipt.attr("type", "retry");
        assert!(received_confirmation(&retry).is_none());
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
    /// The exact status receipt, per `LX/Cp2->A06`'s true branch: `type="sender"`,
    /// `class="status"`, the literal `sts="65535"` (a hardcoded constant, not a
    /// timestamp), no `recipient`, and no `<meta>` child because the mode int is
    /// zero and short-circuits its gate.
    #[test]
    fn status_receipt_carries_sender_type_status_class_and_the_literal_sts() {
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
        assert_eq!(receipt.get_attr("type").unwrap().as_text(), Some("sender"));
        assert_eq!(receipt.get_attr("class").unwrap().as_text(), Some("status"));
        assert_eq!(receipt.get_attr("sts").unwrap().as_text(), Some("65535"));
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
        // Both, neither, and a quote with no group are all refused.
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
                "nogroup.json",
                format!(
                    r#"{{"to":"15551234567","text":"ok","quote":{{{target}}},"request_id":"r5"}}"#
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
        // Both targets at once is ambiguous, and a group send is text only.
        assert!(
            MessageRequest::read(&write(
                "both.json",
                r#"{"to":"15551234567","group":"120363000000000001@g.us","text":"hi","request_id":"r2"}"#,
            ))
            .is_err()
        );
        assert!(
            MessageRequest::read(&write(
                "novoice.json",
                r#"{"group":"120363000000000001@g.us","voice_note":"/dev/null","request_id":"r3"}"#,
            ))
            .is_err()
        );
        // A direct send still needs digits.
        assert!(
            MessageRequest::read(&write("none.json", r#"{"text":"hi","request_id":"r4"}"#))
                .is_err()
        );
    }
}
