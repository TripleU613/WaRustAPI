//! Property tests for the receive-side stanza validators: receipts,
//! notifications and prekey counts come straight from the server, and the
//! receive loop now refuses bad content without ending the session (receive
//! audit, 2026-10-13), so each validator must return an answer for any stanza
//! and never panic. Stanzas are generated close to the real shapes (tags,
//! attribute names and children the validators read) with arbitrary values.
#![cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]

use bytes::Bytes;
use proptest::prelude::*;
use whatsapp_api::binary::{AttributeValue, Node, NodeContent};
use whatsapp_api::model::jid::{Jid, Server};
use whatsapp_api::transport::{notifications, prekeys, receipts, status_receipts};

const TAGS: [&str; 20] = [
    "receipt",
    "notification",
    "list",
    "item",
    "participants",
    "user",
    "count",
    "pq_count",
    "identity",
    "devices",
    "device",
    "key-index-list",
    "retry",
    "registration",
    "keys",
    "key",
    "add",
    "remove",
    "update",
    "token",
];

const ATTRS: [&str; 16] = [
    "id",
    "from",
    "to",
    "participant",
    "recipient",
    "type",
    "t",
    "offline",
    "count",
    "value",
    "message_id",
    "jid",
    "lid",
    "v",
    "device_hash",
    "xmlns",
];

fn any_text() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(vec![
            "read",
            "played",
            "sender",
            "inactive",
            "retry",
            "enc",
            "devices",
            "encrypt",
            "privacy_token",
            "status",
            "read-self",
            "server-error",
            "0",
            "1",
            "-1",
            "",
        ])
        .prop_map(str::to_owned),
        "[0-9]{1,12}",
        "-?[0-9]{1,11}",
        "[A-F0-9]{1,24}",
        ".{0,24}",
    ]
}

fn any_jid() -> impl Strategy<Value = Jid> {
    (
        "[0-9]{1,15}",
        prop_oneof![
            Just(Server::User),
            Just(Server::Lid),
            Just(Server::Group),
            Just(Server::Broadcast),
        ],
        0u16..=99,
    )
        .prop_filter_map("valid jid", |(user, server, device)| match server {
            Server::User | Server::Lid if device > 0 => Jid::new_ad(user, 0, device, server).ok(),
            _ => Jid::new(user, server).ok(),
        })
}

fn any_attrs() -> impl Strategy<Value = Vec<(&'static str, AttributeValue)>> {
    prop::collection::vec(
        (
            prop::sample::select(ATTRS.to_vec()),
            prop_oneof![
                any_text().prop_map(AttributeValue::Text),
                any_jid().prop_map(AttributeValue::Jid),
            ],
        ),
        0..7,
    )
}

fn leaf() -> impl Strategy<Value = Node> {
    (
        prop::sample::select(TAGS.to_vec()),
        any_attrs(),
        prop_oneof![
            Just(NodeContent::Empty),
            Just(NodeContent::Children(Vec::new())),
            prop::collection::vec(any::<u8>(), 0..64)
                .prop_map(|b| NodeContent::Bytes(Bytes::from(b))),
            any_jid().prop_map(NodeContent::Jid),
        ],
    )
        .prop_map(|(tag, attrs, content)| {
            let mut node = Node::new(tag);
            for (name, value) in attrs {
                node = node.attr(name, value);
            }
            node.content = content;
            node
        })
}

/// A stanza rooted at `receipt` or `notification`, with nested children.
fn any_stanza() -> impl Strategy<Value = Node> {
    let inner = leaf().prop_recursive(3, 32, 6, |inner| {
        (
            prop::sample::select(TAGS.to_vec()),
            any_attrs(),
            prop::collection::vec(inner, 0..6),
        )
            .prop_map(|(tag, attrs, children)| {
                let mut node = Node::new(tag);
                for (name, value) in attrs {
                    node = node.attr(name, value);
                }
                node.children(children)
            })
    });
    (
        prop::sample::select(vec!["receipt", "notification"]),
        any_attrs(),
        prop::collection::vec(inner, 0..5),
    )
        .prop_map(|(tag, attrs, children)| {
            let mut node = Node::new(tag);
            for (name, value) in attrs {
                node = node.attr(name, value);
            }
            node.children(children)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// Every validator answers (accept, refuse or ignore) for any stanza.
    #[test]
    fn receive_validators_never_panic(node in any_stanza()) {
        let _ = receipts::validate(&node);
        let _ = receipts::parse_retry(&node);
        let _ = notifications::validate(&node);
        let _ = status_receipts::parse(&node);
        let _ = prekeys::low_count(&node);
        let _ = prekeys::low_count_ack(&node);
    }
}
