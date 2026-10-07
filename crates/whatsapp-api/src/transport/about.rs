//! Setting the profile "About" text.
//!
//! **Confirmed** from comparison 2.26.37.71. The About screen
//! (`com.whatsapp.profile.ui.SetAboutInfo->A5H`) saves through `LX/8lM->A04`,
//! which posts writer case 29 (`Message.obtain(…, 0x1d, …)`) to the XMPP writer,
//! and `LX/0y0->A00` case 29, logged `WriterThread/write/status-update`, builds:
//!
//! ```text
//! <iq id=<id> xmlns="status" type="set" to="s.whatsapp.net">
//!   <status>TEXT</status>
//! </iq>
//! ```
//!
//! The reply is matched by id to the screen's success and failure callbacks.
//! `LX/8lM->A04` replaces a null or empty text with a single space before
//! posting it, so "clearing" the About sends `" "`, and so does this.
//!
//! "Status" here is the old name for About; it is unrelated to the 24-hour
//! status posts in `group_send`.
use crate::{Error, Result, binary::Node, model::jid::Jid, store::ProtocolStore};

/// Durable action kind for an About change.
pub const ACTION_KIND: &str = "about-set";

/// Our bound, not WhatsApp's: the writer imposes none, and the screen's own
/// limit was not traced. Generous for an About line while keeping an accidental
/// paste of a file from going out as one.
pub const MAX_CHARS: usize = 256;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("about", reason)
}

/// Build the request. An empty text becomes `" "`, as `LX/8lM->A04` does.
pub fn set_request(id: &str, text: &str) -> Result<Node> {
    if id.is_empty() || id.len() > 128 {
        return Err(invalid("invalid request id"));
    }
    if text.chars().count() > MAX_CHARS {
        return Err(invalid("an About text is at most 256 characters here"));
    }
    if text.chars().any(|c| c.is_control() && c != '\n') {
        return Err(invalid("an About text has no control characters"));
    }
    let text = if text.is_empty() { " " } else { text };
    Ok(Node::new("iq")
        .attr("id", id)
        .attr("xmlns", "status")
        .attr("type", "set")
        .attr("to", Jid::parse("s.whatsapp.net")?)
        .children(vec![Node::new("status").text(text)]))
}

/// Queue the change on the durable action queue; a session such as `listen`
/// transmits it and records the reply.
pub fn queue_set(store: &ProtocolStore, text: &str) -> Result<String> {
    // Validate before reserving, so a refused text never occupies an action.
    set_request("validation", text)?;
    let text = text.to_owned();
    super::actions::queue(store, ACTION_KIND, "about text", move |id| {
        set_request(id, &text)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{Decoder, Encoder, NodeContent, TokenDictionary};

    /// `LX/0y0->A00` case 29: four attributes and one `status` child whose
    /// content is the text.
    #[test]
    fn set_request_matches_the_traced_writer() {
        let node = set_request("A1", "Available").unwrap();
        assert_eq!(node.description, "iq");
        assert_eq!(node.get_attr("xmlns").unwrap().as_text(), Some("status"));
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("set"));
        assert_eq!(node.get_attr("id").unwrap().as_text(), Some("A1"));
        assert!(node.get_attr("to").unwrap().as_jid().is_some());
        assert_eq!(node.attributes.len(), 4);
        let status = node.child("status").unwrap();
        assert!(matches!(&status.content, NodeContent::Text(t) if t == "Available"));

        let dict = TokenDictionary::comparison_2_26_37_71();
        let encoded = Encoder::new(dict).encode(&node).unwrap();
        let decoded = Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
            .decode_node()
            .unwrap();
        assert_eq!(
            decoded.child("status").map(|s| s.description.as_str()),
            Some("status")
        );
    }

    /// `LX/8lM->A04` sends a single space for an empty About.
    #[test]
    fn an_empty_about_is_sent_as_a_space() {
        let node = set_request("A1", "").unwrap();
        assert!(matches!(&node.child("status").unwrap().content, NodeContent::Text(t) if t == " "));
    }

    #[test]
    fn set_request_refuses_what_it_should() {
        assert!(set_request("", "x").is_err());
        assert!(set_request("A1", &"x".repeat(MAX_CHARS + 1)).is_err());
        assert!(
            set_request("A1", &"é".repeat(MAX_CHARS)).is_ok(),
            "counted in characters"
        );
        assert!(set_request("A1", "a\u{0}b").is_err());
        assert!(set_request("A1", "two\nlines").is_ok());
    }
}
