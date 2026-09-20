//! Spam report (`xmlns=spam`), traced from comparison APK **2.26.37.71** and
//! recorded in `docs/protocol/chat-actions.md` section 2.
//!
//! `IndividualSpamReportRpc.A00` -> `SpamRPCManager.A01` -> `LX/9QZ.<init>`
//! (confirmed shape): `<iq type=set to=s.whatsapp.net xmlns=spam><spam_list
//! jid=<reported user jid> spam_flow="<entry point>"
//! [is_known_chat="true"|"false"]/></iq>`. The reported-message children the
//! APK can attach (`LX/9QI`/`LX/BpT`, 0..210 plus 0..5 plus a third list) are
//! **not** built here: their exact attribute shape was not traced (**unknown**
//! in the source doc), so a report from this module always carries zero
//! message children. `xmlns=spam`, `to`, and `spam_list/@spam_flow` come from
//! the `LX/9QF.<init>(VLI)` case-5 fragment; `is_known_chat` is case 6 of the
//! same constructor (confirmed, optional). This shape is APK-derived and has
//! **not been live-verified**.
use crate::{Error, Result, binary::Node, model::jid::Jid, store::ProtocolStore};

/// `spam_flow` entry-point vocabulary named in chat-actions.md sections 1/2
/// (`LX/9QF.<init>(VLI)` case 5's strings that apply to an individual report).
pub const SPAM_FLOWS: &[&str] = &[
    "chat",
    "overflow_menu_report",
    "account_info_report",
    "message_menu",
    "1_1_spam_banner_report",
    "chat_list_swipe_delete_report_upsell",
    "block_dialog",
];

fn validate_flow(flow: &str) -> Result<&str> {
    if flow.is_empty() {
        return Ok("chat");
    }
    if SPAM_FLOWS.contains(&flow) {
        Ok(flow)
    } else {
        Err(Error::invalid(
            "report",
            "spam_flow is not a recognized entry point",
        ))
    }
}

/// Durably queue a spam report against `chat`, with zero reported-message
/// children (see module docs). `flow` must be one of [`SPAM_FLOWS`] or empty,
/// which defaults to `"chat"`.
pub fn queue_report(
    store: &ProtocolStore,
    chat: &Jid,
    flow: &str,
    known_chat: Option<bool>,
) -> Result<String> {
    if !chat.is_user_like() {
        return Err(Error::invalid(
            "report",
            "report target must be a phone-number or LID identity",
        ));
    }
    let flow = validate_flow(flow)?.to_owned();
    let target = chat.to_non_ad();
    let mut spam_list = Node::new("spam_list")
        .attr("jid", target)
        .attr("spam_flow", flow.as_str());
    if let Some(known) = known_chat {
        spam_list = spam_list.attr("is_known_chat", if known { "true" } else { "false" });
    }
    super::actions::queue(store, "report", "spam report", move |id| {
        Ok(Node::new("iq")
            .attr("id", id)
            .attr("type", "set")
            .attr("to", Jid::parse("s.whatsapp.net")?)
            .attr("xmlns", "spam")
            .children(vec![spam_list]))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    use std::sync::Arc;

    fn store() -> (tempfile::TempDir, ProtocolStore) {
        let dir = tempfile::tempdir().unwrap();
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([3u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, s)
    }

    fn phone() -> Jid {
        Jid::parse("15555550199@s.whatsapp.net").unwrap()
    }

    fn attr_text<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
        node.get_attr(name).and_then(|v| v.as_text())
    }

    #[test]
    fn default_flow_and_no_message_children() {
        let (_d, s) = store();
        let id = queue_report(&s, &phone(), "", None).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        assert_eq!(sent.len(), 1);
        let iq = &sent[0];
        assert_eq!(attr_text(iq, "id"), Some(id.as_str()));
        assert_eq!(attr_text(iq, "type"), Some("set"));
        assert_eq!(attr_text(iq, "xmlns"), Some("spam"));
        assert_eq!(
            iq.get_attr("to").and_then(|v| v.as_jid()),
            Some(&Jid::parse("s.whatsapp.net").unwrap())
        );
        let spam_list = iq.child("spam_list").unwrap();
        assert_eq!(
            spam_list.get_attr("jid").and_then(|v| v.as_jid()),
            Some(&phone())
        );
        assert_eq!(attr_text(spam_list, "spam_flow"), Some("chat"));
        assert!(spam_list.get_attr("is_known_chat").is_none());
        assert!(
            spam_list.child_nodes().is_empty(),
            "no reported message nodes are attached"
        );
    }

    #[test]
    fn explicit_flow_and_known_chat_flag() {
        let (_d, s) = store();
        queue_report(&s, &phone(), "block_dialog", Some(true)).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        let spam_list = sent[0].child("spam_list").unwrap();
        assert_eq!(attr_text(spam_list, "spam_flow"), Some("block_dialog"));
        assert_eq!(attr_text(spam_list, "is_known_chat"), Some("true"));
    }

    #[test]
    fn known_chat_false_is_distinguished_from_absent() {
        let (_d, s) = store();
        queue_report(&s, &phone(), "message_menu", Some(false)).unwrap();
        let sent = super::super::actions::reserve_pending(&s).unwrap();
        let spam_list = sent[0].child("spam_list").unwrap();
        assert_eq!(attr_text(spam_list, "is_known_chat"), Some("false"));
    }

    #[test]
    fn unrecognized_flow_is_rejected() {
        let (_d, s) = store();
        assert!(queue_report(&s, &phone(), "made_up_flow", None).is_err());
    }

    #[test]
    fn every_documented_flow_is_accepted() {
        let (_d, s) = store();
        for flow in SPAM_FLOWS {
            assert!(queue_report(&s, &phone(), flow, None).is_ok(), "{flow}");
        }
    }

    #[test]
    fn group_jid_is_rejected() {
        let (_d, s) = store();
        let group = Jid::parse("123-456@g.us").unwrap();
        assert!(queue_report(&s, &group, "chat", None).is_err());
    }
}
