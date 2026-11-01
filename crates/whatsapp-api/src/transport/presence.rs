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
//!   `composing` (with optional `media`) or `paused` child. **Confirmed**;
//!   [`parse_chatstate`] reads the same.
//!
//! **Not implemented, deliberately:** the `bot` child `LX/B2G` adds for one
//! branch (`<composing><bot jid=…/></composing>`, a bot-chat case), and the
//! handler's own timing — it re-arms on a 2,500 ms timer and compares against
//! 10,000 ms (`LX/B2H` constants `0x9c4`, `0x2710`); what those windows mean in
//! the UI was not traced. The display name is also an app-state setting
//! (`pushNameSetting`, collection `critical_block`) that companion devices read;
//! `app_state_sync::queue_push_name_patch` writes that record separately.
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

const PENDING_NAME: &[u8] = b"push-name-pending";

/// Queue the display-name presence for the next session (validated now).
/// 2.26.37.71 writes it as its own writer case (`0y0.A00` case 3), separate
/// from the plain availability announcement every connection sends
/// (`1Rd.A01`), so it goes out when the name changes, not on every connect.
pub fn queue_push_name(store: &crate::store::ProtocolStore, name: &str) -> Result<()> {
    push_name(name)?;
    store.write(|tx| {
        tx.put_json(
            crate::store::Table::Checkpoints,
            PENDING_NAME,
            1,
            &name.trim().to_owned(),
        )
    })
}

/// Take the queued display-name presence, if any, for a session that has
/// just announced availability. Recorded as written under `push-name` before
/// the caller sends it: the server never acknowledges a presence, so the
/// stanza is fire-and-forget and is not repeated by later sessions.
pub fn take_push_name(store: &crate::store::ProtocolStore) -> Result<Option<Node>> {
    store.write(|tx| {
        let Some(name): Option<String> =
            tx.get_json(crate::store::Table::Checkpoints, PENDING_NAME, 1)?
        else {
            return Ok(None);
        };
        let node = push_name(&name)?;
        tx.delete(crate::store::Table::Checkpoints, PENDING_NAME)?;
        tx.put_json(
            crate::store::Table::Checkpoints,
            b"push-name",
            1,
            &serde_json::json!({"status": "written", "name": name, "at": crate::model::time::Timestamp::now()}),
        )?;
        Ok(Some(node))
    })
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

/// `<presence type="subscribe" [context=<group>] to=<chat>>[<tctoken>…</tctoken>]`
/// — ask the server to forward someone's availability and typing to us, which
/// the official client does when a chat is opened.
///
/// **Confirmed** from `LX/0y0->A00`, `WriterThread/write/subscription-request`:
/// `type="subscribe"`; `context` (a group JID parsed by `LX/1bo->A03`) only when
/// supplied; `to` validated against `LX/1bn` and `UserJid`; and the peer's
/// trusted-contact token, when held, bounded to 1..=120 bytes by
/// `LX/0hp->A04(bytes, 1, 120)`. The token is built as `<tctoken>` wrapped in
/// two `smax:any` nodes, and `LX/0ho->A04` merges an `smax:any` into the node
/// it is added to, so on the wire it is a plain `<tctoken>` child of
/// `presence` carrying the bytes.
pub fn subscribe(to: &Jid, context: Option<&Jid>, tctoken: Option<&[u8]>) -> Result<Node> {
    if !matches!(to.server, Server::User | Server::Lid) || to.device != 0 {
        return Err(invalid(
            "a subscription is to a person, with no device suffix",
        ));
    }
    let mut node = Node::new("presence").attr("type", "subscribe");
    if let Some(context) = context {
        if context.server != Server::Group {
            return Err(invalid("a subscription context is a group"));
        }
        node = node.attr("context", context.clone());
    }
    node = node.attr("to", to.clone());
    if let Some(token) = tctoken {
        if !(1..=120).contains(&token.len()) {
            return Err(invalid("a tctoken is 1 to 120 bytes (LX/0hp->A04)"));
        }
        node = node.children(vec![Node::new("tctoken").bytes(token.to_vec())]);
    }
    Ok(node)
}

/// When someone was last online, as `LX/8nc->A00` reads `last`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastSeen {
    /// No `last` attribute.
    Absent,
    /// Seconds since the epoch (the APK multiplies by 1,000).
    At(u64),
    /// `deny`, `error` or `none`: hidden or unavailable, by their setting.
    Withheld(&'static str),
}

/// An incoming availability update, as `LX/1Qw->A02` reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingPresence {
    /// Whose it is.
    pub from: Jid,
    /// `Some(true)` for `type="available"`, `Some(false)` for `unavailable`,
    /// `None` when there is no `type` (`LX/1Qw` acts on neither then).
    pub available: Option<bool>,
    /// Their last-seen, when offline.
    pub last_seen: LastSeen,
}

/// Read an incoming `<presence>`; `None` for our own outgoing shapes and
/// anything without a sender.
pub fn parse_presence(node: &Node) -> Option<IncomingPresence> {
    if node.description != "presence" {
        return None;
    }
    let from = node.get_attr("from").and_then(|v| {
        v.as_jid()
            .cloned()
            .or_else(|| Jid::parse(v.as_text()?).ok())
    })?;
    let available = match node.get_attr("type").and_then(|v| v.as_text()) {
        Some("available") => Some(true),
        Some("unavailable") => Some(false),
        _ => None,
    };
    let last_seen = match node.get_attr("last").and_then(|v| v.as_text()) {
        None => LastSeen::Absent,
        Some("deny") => LastSeen::Withheld("deny"),
        Some("error") => LastSeen::Withheld("error"),
        Some("none") => LastSeen::Withheld("none"),
        Some(raw) => raw
            .parse()
            .map(LastSeen::At)
            .unwrap_or(LastSeen::Withheld("error")),
    };
    Some(IncomingPresence {
        from,
        available,
        last_seen,
    })
}

/// The peer's saved trusted-contact token (`notifications::record` keeps it
/// under `incoming-trusted-contact:<jid>`), when one is held.
pub fn saved_tctoken(store: &crate::store::ProtocolStore, peer: &Jid) -> Result<Option<Vec<u8>>> {
    let key = format!("incoming-trusted-contact:{peer}");
    let saved: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(crate::store::Table::Checkpoints, key.as_bytes(), 1))?;
    Ok(saved.and_then(|v| serde_json::from_value(v["token"].clone()).ok()))
}

/// An incoming chat state, as `LX/1Qq->A02` reads it: the chat in `from`, the
/// member in `participant` when the chat is a group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingChatState {
    /// The chat it happened in: a person, or a group.
    pub chat: Jid,
    /// Who is typing, in a group; `None` in a direct chat, where it is `chat`.
    pub participant: Option<Jid>,
    /// What they are doing.
    pub state: ChatState,
}

/// Read a `<chatstate>`; `None` for anything else, including a child this
/// build does not name. Ephemeral by nature, so the caller decides whether to
/// keep it — nothing here writes.
pub fn parse_chatstate(node: &Node) -> Option<IncomingChatState> {
    if node.description != "chatstate" {
        return None;
    }
    let jid = |name: &str| {
        node.get_attr(name).and_then(|v| {
            v.as_jid()
                .cloned()
                .or_else(|| Jid::parse(v.as_text()?).ok())
        })
    };
    let chat = jid("from")?;
    let child = node.child_nodes().first()?;
    let state = match child.description.as_str() {
        "composing" => match child.get_attr("media").and_then(|v| v.as_text()) {
            Some("audio") => ChatState::Recording,
            _ => ChatState::Composing,
        },
        "paused" => ChatState::Paused,
        _ => return None,
    };
    Some(IncomingChatState {
        chat,
        participant: jid("participant"),
        state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{Decoder, Encoder, TokenDictionary};

    fn round_trip(node: &Node) {
        let dict = TokenDictionary::comparison_2_26_37_71();
        let encoded = Encoder::new(dict).encode(node).unwrap();
        let decoded = Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
            .decode_node()
            .unwrap();
        assert_eq!(&decoded, node);
    }

    /// `LX/0y0->A00` case 3: `name` then `type="available"`, nothing else.
    #[test]
    fn queued_push_name_is_taken_once() {
        let dir = tempfile::tempdir().unwrap();
        let store = crate::store::ProtocolStore::open(crate::store::OpenOptions {
            path: dir.path().join("name.redb"),
            account_id: crate::model::ids::AccountId::new("name").unwrap(),
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([7; 32]),
            )),
            create: true,
        })
        .unwrap();
        assert!(queue_push_name(&store, "  ").is_err());
        assert!(take_push_name(&store).unwrap().is_none());
        queue_push_name(&store, " Usher ").unwrap();
        let node = take_push_name(&store).unwrap().unwrap();
        assert_eq!(node, push_name("Usher").unwrap());
        assert!(take_push_name(&store).unwrap().is_none(), "sent once");
    }

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

    /// The shape `LX/1Qq->A02` reads.
    #[test]
    fn incoming_chatstates_are_read_like_the_apk_reads_them() {
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let member = Jid::parse("15551234567@lid").unwrap();
        let node = Node::new("chatstate")
            .attr("from", group.clone())
            .attr("participant", member.clone())
            .children(vec![Node::new("composing").attr("media", "audio")]);
        let parsed = parse_chatstate(&node).unwrap();
        assert_eq!(parsed.chat, group);
        assert_eq!(parsed.participant, Some(member));
        assert_eq!(parsed.state, ChatState::Recording);

        let direct = Node::new("chatstate")
            .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            .children(vec![Node::new("paused")]);
        let parsed = parse_chatstate(&direct).unwrap();
        assert_eq!(parsed.state, ChatState::Paused);
        assert!(parsed.participant.is_none());

        let unknown = Node::new("chatstate")
            .attr("from", Jid::parse("15551234567@s.whatsapp.net").unwrap())
            .children(vec![Node::new("dancing")]);
        assert!(parse_chatstate(&unknown).is_none());
        assert!(parse_chatstate(&Node::new("presence")).is_none());
    }

    /// `LX/0y0->A00`, subscription-request writer, after `smax:any` merging.
    #[test]
    fn subscribe_matches_the_traced_writer() {
        let peer = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let bare = subscribe(&peer, None, None).unwrap();
        assert_eq!(bare.description, "presence");
        assert_eq!(bare.get_attr("type").unwrap().as_text(), Some("subscribe"));
        assert_eq!(bare.get_attr("to").unwrap().as_jid(), Some(&peer));
        assert_eq!(bare.attributes.len(), 2);
        assert!(bare.child_nodes().is_empty());
        round_trip(&bare);

        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let full = subscribe(&peer, Some(&group), Some(&[7; 20])).unwrap();
        assert_eq!(full.get_attr("context").unwrap().as_jid(), Some(&group));
        let token = full.child("tctoken").unwrap();
        assert!(
            matches!(&token.content, crate::binary::NodeContent::Bytes(b) if b == &vec![7; 20])
        );
        round_trip(&full);

        assert!(subscribe(&peer, None, Some(&[])).is_err());
        assert!(subscribe(&peer, None, Some(&[0; 121])).is_err());
        assert!(subscribe(&group, None, None).is_err());
        assert!(subscribe(&peer, Some(&peer), None).is_err());
    }

    /// `LX/1Qw->A02` and `LX/8nc->A00`.
    #[test]
    fn incoming_presence_reads_type_and_last_seen() {
        let peer = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let online = Node::new("presence").attr("from", peer.clone());
        let parsed = parse_presence(&online).unwrap();
        assert_eq!(parsed.available, None);
        assert_eq!(parsed.last_seen, LastSeen::Absent);

        let offline = Node::new("presence")
            .attr("from", peer.clone())
            .attr("type", "unavailable")
            .attr("last", "1790000000");
        let parsed = parse_presence(&offline).unwrap();
        assert_eq!(parsed.available, Some(false));
        assert_eq!(parsed.last_seen, LastSeen::At(1_790_000_000));

        let hidden = Node::new("presence")
            .attr("from", peer)
            .attr("type", "unavailable")
            .attr("last", "deny");
        assert_eq!(
            parse_presence(&hidden).unwrap().last_seen,
            LastSeen::Withheld("deny")
        );
        assert!(parse_presence(&Node::new("presence").attr("type", "available")).is_none());
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
