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
    /// Recipient E.164 digits (optional leading plus).
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
    /// Stable local idempotency key; reuse it after uncertain outcomes.
    pub request_id: String,
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
        value.recipient()?;
        if (usize::from(!value.text.is_empty())
            + usize::from(value.image.is_some())
            + usize::from(value.voice_note.is_some())
            != 1)
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
    /// Only direct phone recipients are currently supported.
    pub fn recipient(&self) -> crate::Result<Jid> {
        let digits = self.to.strip_prefix('+').unwrap_or(&self.to);
        if !(5..=15).contains(&digits.len())
            || digits.starts_with('0')
            || !digits.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(crate::Error::invalid(
                "message",
                "recipient must be E.164 digits",
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
    Decoder::new(&data, TokenDictionary::comparison_2_26_37_70(), &limits)
        .decode_node()
        .map_err(|_| error("invalid comparison stanza"))
}

/// Write one uncompressed encrypted stanza.
pub async fn write_node<S: AsyncRead + AsyncWrite + Unpin>(
    channel: &mut EncryptedConnection<S>,
    node: &Node,
) -> Result<(), TransportError> {
    let data = Encoder::new(TokenDictionary::comparison_2_26_37_70())
        .encode(node)
        .map_err(|_| error("cannot encode stanza"))?;
    channel.write(&data).await
}

/// Comparison LX/0y0.A00 0x1786..0x193a: iq/get/encrypt -> key -> user/jid.
pub fn key_request(recipient: &Jid, id: &str) -> Node {
    Node::new("iq")
        .attr("id", id)
        .attr("xmlns", "encrypt")
        .attr("type", "get")
        .attr(
            "to",
            Jid::parse("s.whatsapp.net").expect("constant server JID"),
        )
        .children(vec![
            Node::new("key").children(vec![Node::new("user").attr("jid", recipient.clone())]),
        ])
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

/// Comparison LX/Cns.A00/A05: primary direct delivery receipt, or receipt
/// acknowledgement. Never a read/played/opened receipt. Caller must durably
/// store the incoming frame and authenticate messages before acknowledging them.
pub fn received_confirmation(incoming: &Node) -> Option<Node> {
    if !matches!(incoming.description.as_str(), "message" | "receipt")
        || incoming.get_attr("participant").is_some()
        || incoming.get_attr("recipient").is_some()
    {
        return None;
    }
    let from = incoming.get_attr("from")?;
    let jid = from
        .as_jid()
        .cloned()
        .or_else(|| Jid::parse(from.as_text()?).ok())?;
    if jid.device != 0
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
    if incoming.description == "message" {
        if incoming.get_attr("edit").is_some() {
            return None;
        }
        return Some(Node::new("receipt").attr("to", jid).attr("id", id));
    }
    let mut ack = Node::new("ack")
        .attr("id", id)
        .attr("to", jid)
        .attr("class", "receipt");
    match incoming.get_attr("type").and_then(|v| v.as_text()) {
        None | Some("delivery") => {}
        Some(kind @ ("read" | "played")) => {
            ack = ack.attr("type", kind);
        }
        _ => return None,
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
        let dict = TokenDictionary::comparison_2_26_37_70();
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
        let dict = TokenDictionary::comparison_2_26_37_70();
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
        let dict = TokenDictionary::comparison_2_26_37_70();
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
}
