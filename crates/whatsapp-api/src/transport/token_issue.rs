//! Issue our own trusted-contact (privacy) token to people we message.
//!
//! **Confirmed static, 2.26.37.71** (docs/protocol/token-issue.md,
//! identity-token-refresh.md): when the server ACKs a message we sent to a
//! user, `17J.BFC` -> `DNf` (selector 19) -> `HxT.A04` checks the outbound
//! `wa_trusted_contacts_send` row; missing or expired state is rescheduled by
//! `0q2.A0W` (write `sent_tc_token_timestamp = now_seconds`) before
//! `GeneratePrivacyTokenJob` sends
//! `<iq to="s.whatsapp.net" xmlns="privacy" type="set"><tokens>
//! <token jid=<user> type="trusted_contact" t=<that timestamp>/></tokens></iq>`.
//! A 4xx IQ error is not retried (`A0I`).
//!
//! **Local limits:** the bucket properties 996/997 and 865/909 are not
//! applied, so the confirmed 182-day fallback floor (`0q3.A00`) is the expiry.
//! `DNf`'s raw exclusions for retained message classes `9NJ`/`Bn8` and positive
//! `178.A07` have unknown meaning and are not reproduced; only direct
//! PN/LID chats are scheduled. The call-invite producers are not ported.

use crate::{
    Result,
    binary::Node,
    model::jid::{Jid, Server},
    store::{Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// `0q3.A00` fallback floor: now_seconds - 15,724,800 (182 days).
pub const FALLBACK_SECONDS: i64 = 15_724_800;

#[derive(Serialize, Deserialize)]
struct SendState {
    /// `sent_tc_token_timestamp`, in seconds.
    sent: i64,
}

fn key(user: &Jid) -> Vec<u8> {
    format!(
        "tc-token-send:{}",
        hex::encode(Sha256::digest(user.to_string()))
    )
    .into_bytes()
}

/// The confirmed `GeneratePrivacyTokenJob.A0G` request.
pub fn issue_request(id: &str, user: &Jid, t: i64) -> Result<Node> {
    Ok(Node::new("iq")
        .attr("id", id)
        .attr("to", Jid::parse("s.whatsapp.net")?)
        .attr("xmlns", "privacy")
        .attr("type", "set")
        .children(vec![Node::new("tokens").children(vec![
            Node::new("token")
                .attr("jid", user.to_non_ad())
                .attr("type", "trusted_contact")
                .attr("t", t.to_string()),
        ])]))
}

/// After a successful server ACK of a message to `chat`, schedule our token
/// for that user unless one was issued within the window. Returns the queued
/// action ID. Groups, broadcasts and non-user chats are ignored.
pub(crate) fn schedule_after_ack_in(
    tx: &mut WriteTx<'_>,
    chat: &Jid,
    now_seconds: i64,
) -> Result<Option<String>> {
    if !matches!(chat.server, Server::User | Server::Lid) || chat.user.is_empty() {
        return Ok(None);
    }
    let user = chat.to_non_ad();
    let selector = key(&user);
    if let Some(state) = tx.get_json::<SendState>(Table::Checkpoints, &selector, 1)?
        && state.sent >= now_seconds.saturating_sub(FALLBACK_SECONDS)
    {
        return Ok(None);
    }
    // 0q2.A0W writes the scheduled time before the job; it is not proof of
    // server acceptance, and a failed issue is not retried until it expires.
    tx.put_json(
        Table::Checkpoints,
        &selector,
        1,
        &SendState { sent: now_seconds },
    )?;
    let id = super::actions::queue_in(tx, "privacy-token", "trusted-contact token issue", |id| {
        issue_request(id, &user, now_seconds)
    })?;
    Ok(Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource};

    #[test]
    fn issue_request_matches_the_traced_job() {
        let user = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let node = issue_request("ID1", &user, 1_700_000_000).unwrap();
        let text = |n: &Node, k: &str| n.get_attr(k).and_then(|v| v.as_text()).map(str::to_owned);
        assert_eq!(node.description, "iq");
        assert_eq!(text(&node, "xmlns").as_deref(), Some("privacy"));
        assert_eq!(text(&node, "type").as_deref(), Some("set"));
        let token = &node.child("tokens").unwrap().child_nodes()[0];
        assert_eq!(token.description, "token");
        assert_eq!(text(token, "type").as_deref(), Some("trusted_contact"));
        assert_eq!(text(token, "t").as_deref(), Some("1700000000"));
        assert!(
            matches!(token.content, crate::binary::NodeContent::Empty),
            "no token bytes"
        );
    }

    #[test]
    fn scheduled_once_per_window_and_only_for_people() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("tc.redb"),
            account_id: crate::model::ids::AccountId::new("tc").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap();
        let user = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let now = 1_800_000_000;
        let schedule = |chat: &Jid, at: i64| {
            store
                .write(|tx| schedule_after_ack_in(tx, chat, at))
                .unwrap()
        };
        assert!(schedule(&group, now).is_none());
        assert!(schedule(&user, now).is_some());
        assert!(schedule(&user, now + 60).is_none(), "within the window");
        assert!(
            schedule(&user, now + FALLBACK_SECONDS + 1).is_some(),
            "expired"
        );
    }
}
