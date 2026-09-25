//! Display name and typing indicators: the two fire-and-forget stanzas a
//! client writes while a chat is open. Neither is an `iq`, so neither has a
//! reply to correlate; the server does not acknowledge them.
//!
//! Evidence (comparison build 2.26.37.71):
//!
//! * Display name — `LX/0y0->A00`, writer case 3, logged as
//!   `WriterThread/write/push-name`: a `presence` element whose `type` is
//!   validated against `{available, unavailable}` and set to `available`, plus
//!   `name` when the string is non-null. **Confirmed.** The length check
//!   `LX/0hp->A06` runs with bounds `0..=2^53-1`, a generated-schema default, so
//!   the APK imposes no meaningful length limit on the wire; the limit below is
//!   ours, not WhatsApp's.
//! * Typing — `LX/B2G->A01` (`HandleMeComposing/sendComposing`) builds
//!   `<chatstate to=…><composing/></chatstate>`, adding `media="audio"` to
//!   `composing` while recording a voice note; `LX/B2H->handleMessage`
//!   (`HandleMeComposing/write/paused`) builds `<chatstate to=…><paused/>`.
//!   **Confirmed.** The `to` is validated against `UserJid`, `LX/1bn` and
//!   `LX/9Lm` — a user, and (inferred from the class set) a group.
//! * The receive side, `LX/1Qq->A02`, reads `from`, `participant`, and a
//!   `composing` (with optional `media`) or `paused` child. **Confirmed.**
//!
//! **Not implemented, deliberately:** the `bot` child `LX/B2G` adds for one
//! branch (`<composing><bot jid=…/></composing>`, a bot-chat case), and the
//! handler's own timing — it re-arms on a 2,500 ms timer and compares against
//! 10,000 ms (`LX/B2H` constants `0x9c4`, `0x2710`); what those windows mean in
//! the UI was not traced. The display name is also an app-state setting
//! (`pushNameSetting`, collection `critical_block`) that companion devices read;
//! a primary-only client that sends the presence is not writing that record.
use crate::{
    Error, Result,
    binary::Node,
    model::jid::{Jid, Server},
};

/// Our bound, not WhatsApp's: the APK enforces none on the wire (see the
/// module notes). 100 characters is generous for a display name and keeps an
/// accidental paste of a file from going out as one.
pub const MAX_NAME_CHARS: usize = 100;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("presence", reason)
}

/// `<presence name=<name> type="available"/>` — sets the display name other
/// people see beside our number.
pub fn push_name(name: &str) -> Result<Node> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(invalid("a display name is not empty"));
    }
    if trimmed.chars().count() > MAX_NAME_CHARS {
        return Err(invalid("a display name is at most 100 characters here"));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(invalid("a display name has no control characters"));
    }
    Ok(Node::new("presence")
        .attr("name", trimmed)
        .attr("type", "available"))
}

/// What the other side is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatState {
    /// "typing…"
    Composing,
    /// "recording audio…" — `composing` with `media="audio"`.
    Recording,
    /// Stopped.
    Paused,
}

impl ChatState {
    /// The spelling the CLI accepts.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "composing" | "typing" => Ok(Self::Composing),
            "recording" => Ok(Self::Recording),
            "paused" => Ok(Self::Paused),
            _ => Err(invalid("chat state is composing, recording or paused")),
        }
    }
}

/// Where a typing indicator may go: a person (phone or LID) or a group, never
/// a device, a broadcast list or status.
pub fn chat_target(raw: &str) -> Result<Jid> {
    let jid = if raw.contains('@') {
        Jid::parse(raw)?
    } else {
        let digits = raw.strip_prefix('+').unwrap_or(raw);
        if !(5..=15).contains(&digits.len())
            || digits.starts_with('0')
            || !digits.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(invalid("a chat is E.164 digits or a phone/LID/group JID"));
        }
        Jid::parse(&format!("{digits}@s.whatsapp.net"))?
    };
    if !matches!(jid.server, Server::User | Server::Lid | Server::Group) || jid.device != 0 {
        return Err(invalid(
            "a chat is a person or a group, with no device suffix",
        ));
    }
    Ok(jid)
}

/// `<chatstate to=<chat>><composing [media="audio"]/> | <paused/></chatstate>`.
pub fn chatstate(chat: &Jid, state: ChatState) -> Result<Node> {
    if !matches!(chat.server, Server::User | Server::Lid | Server::Group) || chat.device != 0 {
        return Err(invalid(
            "a chat is a person or a group, with no device suffix",
        ));
    }
    let child = match state {
        ChatState::Composing => Node::new("composing"),
        ChatState::Recording => Node::new("composing").attr("media", "audio"),
        ChatState::Paused => Node::new("paused"),
    };
    Ok(Node::new("chatstate")
        .attr("to", chat.clone())
        .children(vec![child]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{Decoder, Encoder, TokenDictionary};

    fn round_trip(node: &Node) {
        let dict = TokenDictionary::comparison_2_26_37_70();
        let encoded = Encoder::new(dict).encode(node).unwrap();
        let decoded = Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
            .decode_node()
            .unwrap();
        assert_eq!(&decoded, node);
    }

    /// `LX/0y0->A00` case 3: `name` then `type="available"`, nothing else.
    #[test]
    fn push_name_matches_the_traced_writer() {
        let node = push_name("  Test Name  ").unwrap();
        assert_eq!(node.description, "presence");
        assert_eq!(node.get_attr("name").unwrap().as_text(), Some("Test Name"));
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("available"));
        assert_eq!(node.attributes.len(), 2);
        assert!(node.child_nodes().is_empty());
        round_trip(&node);

        assert!(push_name("   ").is_err());
        assert!(push_name("a\nb").is_err());
        assert!(push_name(&"x".repeat(MAX_NAME_CHARS + 1)).is_err());
        assert!(
            push_name(&"é".repeat(MAX_NAME_CHARS)).is_ok(),
            "counted in characters"
        );
    }

    /// `LX/B2G->A01` and `LX/B2H->handleMessage`.
    #[test]
    fn chatstates_match_the_traced_writers() {
        let chat = chat_target("15551234567").unwrap();
        let typing = chatstate(&chat, ChatState::Composing).unwrap();
        assert_eq!(typing.description, "chatstate");
        assert_eq!(typing.get_attr("to").unwrap().as_jid(), Some(&chat));
        assert_eq!(typing.attributes.len(), 1);
        let child = &typing.child_nodes()[0];
        assert_eq!(child.description, "composing");
        assert!(child.attributes.is_empty());
        round_trip(&typing);

        let recording = chatstate(&chat, ChatState::Recording).unwrap();
        let child = &recording.child_nodes()[0];
        assert_eq!(child.description, "composing");
        assert_eq!(child.get_attr("media").unwrap().as_text(), Some("audio"));
        round_trip(&recording);

        let paused = chatstate(&chat, ChatState::Paused).unwrap();
        assert_eq!(paused.child_nodes()[0].description, "paused");
        assert_eq!(paused.child_nodes().len(), 1);
        round_trip(&paused);

        let group = chat_target("120363000000000001@g.us").unwrap();
        assert!(chatstate(&group, ChatState::Composing).is_ok());
    }

    #[test]
    fn chat_targets_are_people_or_groups() {
        assert!(chat_target("15551234567@lid").is_ok());
        assert!(chat_target("status@broadcast").is_err());
        assert!(chat_target("15551234567:2@s.whatsapp.net").is_err());
        assert!(chat_target("0123").is_err());
        assert_eq!(ChatState::parse("typing").unwrap(), ChatState::Composing);
        assert!(ChatState::parse("dancing").is_err());
    }
}
