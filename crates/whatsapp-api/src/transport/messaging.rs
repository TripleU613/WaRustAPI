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
    pub text: zeroize::Zeroizing<String>,
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
        if value.text.is_empty()
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

/// Direct primary-device message envelope; encrypted payload only.
pub fn direct_message(
    recipient: &Jid,
    id: &str,
    kind: &str,
    ciphertext: Vec<u8>,
) -> Result<Node, TransportError> {
    if !matches!(kind, "pkmsg" | "msg") || ciphertext.is_empty() || id.is_empty() {
        return Err(error("invalid direct-message envelope"));
    }
    Ok(Node::new("message")
        .attr("to", recipient.clone())
        .attr("id", id)
        .attr("type", "text")
        .children(vec![
            Node::new("enc")
                .attr("v", "2")
                .attr("type", kind)
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
