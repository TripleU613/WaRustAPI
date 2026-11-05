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
//! applied. Scheduling uses the 182-day fallback; reservation enforces the
//! mandatory 182-day lower bound confirmed in 43.72 0hu.A00/A01. Stricter
//! effective configured-bucket admission remains unported.
//! `DNf`'s raw exclusions for retained message classes `9NJ`/`Bn8` and positive
//! `178.A07` have unknown meaning and are not reproduced; only direct
//! PN/LID chats are scheduled. The call-invite producers are not ported.

use crate::{
    Error, Result,
    binary::Node,
    model::jid::{Jid, Server},
    store::{Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const ACTION_KIND: &str = "privacy-token";

/// `0q3.A00` fallback floor: now_seconds - 15,724,800 (182 days).
pub const FALLBACK_SECONDS: i64 = 15_724_800;

#[derive(Serialize, Deserialize)]
struct SendState {
    /// `sent_tc_token_timestamp`, in seconds.
    sent: i64,
    /// Native nullable real_issue_timestamp. Absent legacy fields are unknown;
    /// a new schedule uses zero and a matching completion stores null.
    #[serde(default)]
    real_issue: Option<i64>,
    /// False for legacy records that never persisted completion metadata.
    #[serde(default)]
    real_issue_known: bool,
    /// Current schedule owner, not a live callback; legacy rows omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    action_id: Option<String>,
}

/// Content-free effect of a correlated token result on saved timestamp state.
/// This is not delivery proof or permission to retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenIssueCompletion {
    /// Captured time confirmed/advanced the scheduled timestamp; real time cleared.
    ScheduledTimestampConfirmed,
    /// An older completion advanced a nonnull real time without replacing a newer schedule.
    EarlierIssueTimestampAdvanced,
    /// No state existed; the captured completed schedule was inserted.
    InsertedCompletedState,
    /// No supported timestamp change; newer/completed or unknown legacy state preserved.
    NoTimestampChange,
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

// APK43.72 infra/privacytoken/GeneratePrivacyTokenJob.A0L: metadata-only
// trusted-contact issue, distinct from client-auth nonce/token requests.
pub(crate) fn is_issue_request(node: &Node) -> bool {
    node.description == "iq"
        && node.get_attr("type").and_then(|value| value.as_text()) == Some("set")
        && node.get_attr("xmlns").and_then(|value| value.as_text()) == Some("privacy")
        && matches!(node.child_nodes(), [tokens]
            if tokens.description == "tokens"
                && matches!(tokens.child_nodes(), [token]
                    if token.description == "token"
                        && token.get_attr("type").and_then(|value| value.as_text()) == Some("trusted_contact")
                        && token.get_attr("jid").and_then(|value| value.as_jid()).is_some_and(|jid|
                            !jid.user.is_empty() && matches!(jid.server, Server::User | Server::Lid))
                        && token.get_attr("t").and_then(|value| value.as_text()).and_then(super::mex::java_decimal_long).is_some()
                        && matches!(&token.content, crate::binary::NodeContent::Empty)))
}

fn invalid(reason: &'static str) -> Error {
    Error::invalid("privacy_token", reason)
}

fn details(id: &str, node: &Node) -> Result<(Jid, i64)> {
    if id.len() != 32
        || !id.bytes().all(|byte| byte.is_ascii_hexdigit())
        || node.get_attr("id").and_then(|value| value.as_text()) != Some(id)
        || !is_issue_request(node)
    {
        return Err(invalid("invalid saved token issue shape or identity"));
    }
    let token = &node.child_nodes()[0].child_nodes()[0];
    let user = token
        .get_attr("jid")
        .and_then(|value| value.as_jid())
        .unwrap()
        .to_non_ad();
    let sent = token
        .get_attr("t")
        .and_then(|value| value.as_text())
        .and_then(super::mex::java_decimal_long)
        .unwrap();
    if *node != issue_request(id, &user, sent)? {
        return Err(invalid("saved token issue contains unsupported metadata"));
    }
    Ok((user, sent))
}

fn owner_in(tx: &WriteTx<'_>, id: &str, user: &Jid) -> Result<super::actions::ActionStatus> {
    let (outcome, _, request) = super::actions::view_in_write(tx, id)?
        .ok_or_else(|| invalid("saved token schedule owner is missing"))?;
    if outcome.kind != ACTION_KIND || details(id, &request)?.0 != *user {
        return Err(invalid(
            "saved token schedule owner does not match its user",
        ));
    }
    Ok(outcome.status)
}

fn legacy_owner_in(
    tx: &WriteTx<'_>,
    user: &Jid,
    sent: i64,
    queued_only: bool,
) -> Result<Option<String>> {
    use super::actions::ActionStatus;
    let mut queued = None;
    let mut matching = None;
    for (id, status, node) in super::actions::for_kind_in(tx, ACTION_KIND)? {
        let (candidate, timestamp) = details(&id, &node)?;
        if candidate != *user {
            continue;
        }
        if status == ActionStatus::Queued {
            if queued.is_none() {
                queued = Some(id.clone());
            }
            if timestamp == sent && matching.is_none() {
                matching = Some(id);
            }
        } else if !queued_only && timestamp == sent {
            // Legacy reserved/success/refusal ownership is not permission to
            // resend the same schedule through an older queued duplicate.
            return Ok(Some(id));
        }
    }
    Ok(matching.or(queued))
}

pub(crate) enum IssueReservation {
    Ready(Node),
    Deferred(super::actions::ActionDeferReason),
}

// Only called while the action is still Queued in the same write transaction.
pub(crate) fn prepare_in(
    tx: &mut WriteTx<'_>,
    id: &str,
    node: &Node,
    now_seconds: i64,
) -> Result<IssueReservation> {
    use super::actions::ActionDeferReason;
    let (user, _) = details(id, node)?;
    let selector = key(&user);
    let Some(mut state): Option<SendState> = tx.get_json(Table::Checkpoints, &selector, 1)? else {
        return Ok(IssueReservation::Deferred(
            ActionDeferReason::PrivacyTokenStateMissing,
        ));
    };
    // APK43.72 0hu.A00 clamps each bucket threshold to this floor; A01 takes
    // their min. Older than this floor is invalid for every configuration.
    // Passing this necessary bound does not prove configured-bucket admission.
    if state.sent < now_seconds.saturating_sub(FALLBACK_SECONDS) {
        return Ok(IssueReservation::Deferred(
            ActionDeferReason::PrivacyTokenExpired,
        ));
    }
    if state.action_id.is_none() {
        state.action_id = legacy_owner_in(tx, &user, state.sent, false)?;
        tx.put_json(Table::Checkpoints, &selector, 1, &state)?;
    }
    let owner = state
        .action_id
        .as_deref()
        .ok_or_else(|| invalid("queued token schedule has no owner"))?;
    let status = owner_in(tx, owner, &user)?;
    if owner != id || status != super::actions::ActionStatus::Queued {
        return Ok(IssueReservation::Deferred(
            ActionDeferReason::PrivacyTokenSuperseded,
        ));
    }
    // Reload the stored timestamp as A0L does. Neither now_seconds nor a
    // fabricated fresh timestamp replaces it. Sent rows never reach this path.
    Ok(IssueReservation::Ready(issue_request(
        id, &user, state.sent,
    )?))
}

/// 43.72 8R8.CK4 selector3 -> 0ht.A0V uses the captured user/time,
/// never response metadata or receipt wall time. The caller settles the action
/// in this same transaction, preserving both facts or neither on local failure.
pub(crate) fn record_success_in(
    tx: &mut WriteTx<'_>,
    id: &str,
    request: &Node,
) -> Result<TokenIssueCompletion> {
    let (user, captured) = details(id, request)?;
    let selector = key(&user);
    let Some(mut state) = tx.get_json::<SendState>(Table::Checkpoints, &selector, 1)? else {
        tx.put_json(
            Table::Checkpoints,
            &selector,
            1,
            &SendState {
                sent: captured,
                real_issue: None,
                real_issue_known: true,
                action_id: Some(id.to_owned()),
            },
        )?;
        return Ok(TokenIssueCompletion::InsertedCompletedState);
    };
    let effect = if state.sent <= captured {
        state.sent = captured;
        state.real_issue = None;
        state.real_issue_known = true;
        TokenIssueCompletion::ScheduledTimestampConfirmed
    } else if state.real_issue_known
        && state
            .real_issue
            .is_some_and(|previous| previous <= captured)
    {
        state.real_issue = Some(captured);
        TokenIssueCompletion::EarlierIssueTimestampAdvanced
    } else {
        // Native insert-on-conflict IGNORE changes nothing after both
        // conditional updates miss. Do not overwrite the newer schedule owner.
        return Ok(TokenIssueCompletion::NoTimestampChange);
    };
    tx.put_json(Table::Checkpoints, &selector, 1, &state)?;
    Ok(effect)
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
    let previous = tx.get_json::<SendState>(Table::Checkpoints, &selector, 1)?;
    if previous
        .as_ref()
        .is_some_and(|state| state.sent >= now_seconds.saturating_sub(FALLBACK_SECONDS))
    {
        return Ok(None);
    }
    // APK43.72 87x.A04 advances stored time before A05 pending admission.
    // Coalesce an existing unwritten action, never reconstruct a Sent callback.
    let reusable = if let Some(state) = &previous {
        if let Some(id) = &state.action_id {
            (owner_in(tx, id, &user)? == super::actions::ActionStatus::Queued).then(|| id.clone())
        } else {
            legacy_owner_in(tx, &user, state.sent, true)?
        }
    } else {
        None
    };
    let id = if let Some(id) = reusable {
        id
    } else {
        super::actions::queue_in(tx, ACTION_KIND, "trusted-contact token issue", |id| {
            issue_request(id, &user, now_seconds)
        })?
    };
    tx.put_json(
        Table::Checkpoints,
        &selector,
        1,
        &SendState {
            sent: now_seconds,
            // 43.72 0hv.A08 projects raw NULL as the prior sent time,
            // raw zero as no issue time, otherwise the raw value. A0W
            // carries that nonnegative projection or initializes zero.
            // Legacy Rust rows have no proven raw completion state.
            real_issue: Some(
                previous
                    .as_ref()
                    .and_then(|state| {
                        state
                            .real_issue_known
                            .then_some(state.real_issue.unwrap_or(state.sent))
                    })
                    .filter(|value| *value >= 0)
                    .unwrap_or(0),
            ),
            real_issue_known: true,
            action_id: Some(id.clone()),
        },
    )?;
    Ok(Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource};

    fn open_at(path: &std::path::Path, create: bool) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: crate::model::ids::AccountId::new("token-reservation").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create,
        })
        .unwrap()
    }
    fn token_time(node: &Node) -> i64 {
        node.child("tokens")
            .unwrap()
            .child("token")
            .unwrap()
            .get_attr("t")
            .unwrap()
            .as_text()
            .unwrap()
            .parse()
            .unwrap()
    }
    fn now() -> i64 {
        crate::model::time::Timestamp::now().as_millis() / 1000
    }

    #[test]
    fn older_success_preserves_new_schedule_owner_and_later_completion_clears_real_time() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state");
        let store = open_at(&path, true);
        let user = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let captured = 1_800_000_000;
        let old = store
            .write(|tx| schedule_after_ack_in(tx, &user, captured))
            .unwrap()
            .unwrap();
        super::super::actions::reserve_next(&store)
            .unwrap()
            .unwrap();
        let later = captured + FALLBACK_SECONDS + 60;
        let current = store
            .write(|tx| schedule_after_ack_in(tx, &user, later))
            .unwrap()
            .unwrap();
        assert_ne!(old, current);
        let response = |id: &str| Node::new("iq").attr("id", id).attr("type", "result");
        assert!(super::super::actions::record_reply(&store, &response(&old)).unwrap());
        let state = store
            .read(|tx| tx.get_json::<SendState>(Table::Checkpoints, &key(&user), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            (state.sent, state.real_issue, state.action_id),
            (later, Some(captured), Some(current.clone()))
        );
        let request = super::super::actions::reserve_next(&store)
            .unwrap()
            .unwrap();
        assert_eq!(details(&current, &request).unwrap().1, later);
        assert!(super::super::actions::record_reply(&store, &response(&current)).unwrap());
        assert!(!super::super::actions::record_reply(&store, &response(&old)).unwrap());
        let state = store
            .read(|tx| tx.get_json::<SendState>(Table::Checkpoints, &key(&user), 1))
            .unwrap()
            .unwrap();
        assert_eq!(
            (state.sent, state.real_issue, state.action_id),
            (later, None, Some(current))
        );
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/apk/2.26.43.72-token-success-state/timestamp-vectors.json"
        ))
        .unwrap();
        for case in fixture["schedule_vectors"].as_array().unwrap() {
            store
                .write(|tx| {
                    let mut before = case["before"].clone();
                    before["real_issue_known"] = serde_json::json!(true);
                    tx.put_json(Table::Checkpoints, &key(&user), 1, &before)?;
                    schedule_after_ack_in(tx, &user, captured + FALLBACK_SECONDS + 60)?;
                    Ok(())
                })
                .unwrap();
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<SendState>(Table::Checkpoints, &key(&user), 1))
                    .unwrap()
                    .unwrap()
                    .real_issue,
                case["after_real_issue"].as_i64()
            );
        }
        // Unknown legacy history is not promoted into a completed prior issue.
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    &key(&user),
                    1,
                    &serde_json::json!({"sent": captured}),
                )?;
                schedule_after_ack_in(tx, &user, captured + FALLBACK_SECONDS + 60)?;
                Ok(())
            })
            .unwrap();
        assert_eq!(
            store
                .read(|tx| tx.get_json::<SendState>(Table::Checkpoints, &key(&user), 1))
                .unwrap()
                .unwrap()
                .real_issue,
            Some(0)
        );
    }

    #[test]
    fn malformed_success_state_rolls_back_action_and_error_leaves_timestamps_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state");
        let store = open_at(&path, true);
        let user = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let id = store
            .write(|tx| schedule_after_ack_in(tx, &user, 1_800_000_000))
            .unwrap()
            .unwrap();
        super::super::actions::reserve_next(&store)
            .unwrap()
            .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    &key(&user),
                    1,
                    &serde_json::json!({"sent":"invalid saved timestamp"}),
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(
            super::super::actions::record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
            )
            .is_err()
        );
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
        );
        assert!(
            super::super::actions::record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "error")
            )
            .unwrap()
        );
        let outcome = super::super::actions::outcomes(&store)
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(outcome.status, super::super::actions::ActionStatus::Error);
        assert_eq!(outcome.token_issue_completion, None);
        assert_eq!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, &key(&user), 1))
                .unwrap()
                .unwrap(),
            serde_json::json!({"sent":"invalid saved timestamp"})
        );
    }

    #[test]
    fn later_settlement_fault_rolls_back_completed_token_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = open_at(&dir.path().join("state"), true);
        let user = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let id = store
            .write(|tx| schedule_after_ack_in(tx, &user, 1_800_000_000))
            .unwrap()
            .unwrap();
        super::super::actions::reserve_next(&store)
            .unwrap()
            .unwrap();
        store
            .write(|tx| tx.put_json(Table::Checkpoints, b"action-settlement-sequence", 1, &0u64))
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(
            super::super::actions::record_reply(
                &store,
                &Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
            )
            .is_err()
        );
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
        );
        let state = store
            .read(|tx| tx.get_json::<SendState>(Table::Checkpoints, &key(&user), 1))
            .unwrap()
            .unwrap();
        assert_eq!(state.real_issue, Some(0));
        assert_eq!(
            super::super::actions::outcomes(&store).unwrap()[0].status,
            super::super::actions::ActionStatus::Sent
        );
    }

    #[test]
    fn correlated_success_matches_independent_native_sql_vectors_and_survives_reopen() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/apk/2.26.43.72-token-success-state/timestamp-vectors.json"
        ))
        .unwrap();
        for domain in ["s.whatsapp.net", "lid"] {
            for case in fixture["vectors"].as_array().unwrap() {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("state");
                let store = open_at(&path, true);
                let user = Jid::parse(&format!("15555550101@{domain}")).unwrap();
                let captured = case["captured"].as_i64().unwrap();
                let id = store
                    .write(|tx| schedule_after_ack_in(tx, &user, captured))
                    .unwrap()
                    .unwrap();
                let request = super::super::actions::reserve_next(&store)
                    .unwrap()
                    .unwrap();
                store
                    .write(|tx| {
                        if case["before"].is_null() {
                            tx.delete(Table::Checkpoints, &key(&user))?;
                        } else {
                            let mut before = case["before"].clone();
                            before["action_id"] = serde_json::json!(id);
                            before["real_issue_known"] = serde_json::json!(true);
                            tx.put_json(Table::Checkpoints, &key(&user), 1, &before)?;
                        }
                        Ok(())
                    })
                    .unwrap();
                let response = Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .attr("t", "1999999999")
                    .children(vec![Node::new("error").attr("code", "403")]);
                assert!(super::super::actions::record_reply(&store, &response).unwrap());
                let saved = store
                    .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, &key(&user), 1))
                    .unwrap()
                    .unwrap();
                assert_eq!(saved["sent"], case["after"]["sent"]);
                assert_eq!(saved["real_issue"], case["after"]["real_issue"]);
                assert_eq!(saved["action_id"], id);
                let outcome = super::super::actions::outcomes(&store)
                    .unwrap()
                    .into_iter()
                    .find(|row| row.id == id)
                    .unwrap();
                assert_eq!(outcome.status, super::super::actions::ActionStatus::Ok);
                assert_eq!(
                    serde_json::to_value(outcome.token_issue_completion.unwrap()).unwrap(),
                    case["effect"]
                );
                let snapshot = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                assert!(!super::super::actions::record_reply(&store, &response).unwrap());
                assert_eq!(
                    snapshot,
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap()
                );
                drop(store);
                let store = open_at(&path, false);
                assert_eq!(
                    saved,
                    store
                        .read(|tx| tx.get_json::<serde_json::Value>(
                            Table::Checkpoints,
                            &key(&user),
                            1
                        ))
                        .unwrap()
                        .unwrap()
                );
                assert_eq!(
                    Some(request),
                    store
                        .write(|tx| super::super::actions::view_in_write(tx, &id))
                        .unwrap()
                        .map(|(_, _, request)| request)
                );
                assert_eq!(
                    super::super::actions::outcomes(&store)
                        .unwrap()
                        .into_iter()
                        .find(|row| row.id == id)
                        .unwrap(),
                    outcome
                );
            }
        }
    }

    #[test]
    fn queued_timestamp_reload_coalesces_reopens_and_never_rewrites_sent() {
        use super::super::actions::{self, ActionDeferReason, ActionStatus};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reload.redb");
        let store = open_at(&path, true);
        let user = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let fresh = now();
        let old = fresh - FALLBACK_SECONDS - 60;
        let id = store
            .write(|tx| schedule_after_ack_in(tx, &user, old))
            .unwrap()
            .unwrap();
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
        assert_eq!(
            actions::outcomes(&store).unwrap()[0].deferred_reason,
            Some(ActionDeferReason::PrivacyTokenExpired)
        );
        assert_eq!(
            store
                .write(|tx| schedule_after_ack_in(tx, &user, fresh))
                .unwrap(),
            Some(id.clone())
        );
        assert_eq!(actions::outcomes(&store).unwrap().len(), 1);
        assert_eq!(
            token_time(&actions::request(&store, &id).unwrap().unwrap()),
            old,
            "queued preview is not the reserved wire"
        );
        drop(store);
        let store = open_at(&path, false);
        let reserved = actions::reserve_pending(&store).unwrap();
        assert_eq!(reserved.len(), 1);
        assert_eq!(
            reserved[0].get_attr("id").unwrap().as_text(),
            Some(id.as_str())
        );
        assert_eq!(token_time(&reserved[0]), fresh);
        assert_eq!(
            token_time(&actions::request(&store, &id).unwrap().unwrap()),
            fresh
        );
        let saved = store
            .read(|tx| actions::request_wire_in(tx, &id))
            .unwrap()
            .unwrap();
        let outcome = actions::outcomes(&store).unwrap().remove(0);
        assert_eq!(outcome.status, ActionStatus::Sent);
        assert_eq!(outcome.deferred_reason, None);
        // A later legitimate scheduling window is a different intent. It may
        // not update, reset or replay the already reserved request.
        let next_time = fresh + FALLBACK_SECONDS + 1;
        let next = store
            .write(|tx| schedule_after_ack_in(tx, &user, next_time))
            .unwrap()
            .unwrap();
        assert_ne!(next, id);
        let node = store
            .write(|tx| actions::reserve_one_in(tx, &next))
            .unwrap();
        assert_eq!(token_time(&node), next_time);
        assert!(store.write(|tx| actions::reserve_one_in(tx, &id)).is_err());
        assert_eq!(
            store
                .read(|tx| actions::request_wire_in(tx, &id))
                .unwrap()
                .unwrap(),
            saved
        );
        assert!(actions::reserve_pending(&store).unwrap().is_empty());
    }

    #[test]
    fn legacy_duplicates_recover_one_owner_without_resurrecting_sent_results() {
        use super::super::actions::{self, ActionDeferReason, ActionStatus};
        for completion in [None, Some("result"), Some("error")] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("legacy.redb");
            let store = open_at(&path, true);
            let user = Jid::parse("15555550101@lid").unwrap();
            let current = now();
            let old_id = actions::queue(&store, ACTION_KIND, "old token", |id| {
                issue_request(id, &user, current - 60)
            })
            .unwrap();
            let current_id = actions::queue(&store, ACTION_KIND, "current token", |id| {
                issue_request(id, &user, current)
            })
            .unwrap();
            // Independent fixture for the pre-index schema, not Serialize of
            // the new SendState: legacy records carry only sent.
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        &key(&user),
                        1,
                        &serde_json::json!({"sent": current}),
                    )
                })
                .unwrap();
            let reserved = actions::reserve_pending(&store).unwrap();
            assert_eq!(reserved.len(), 1);
            assert_eq!(
                reserved[0].get_attr("id").unwrap().as_text(),
                Some(current_id.as_str())
            );
            if let Some(kind) = completion {
                assert!(
                    actions::record_reply(
                        &store,
                        &Node::new("iq")
                            .attr("id", current_id.as_str())
                            .attr("type", kind)
                    )
                    .unwrap()
                );
            }
            let wire = store
                .read(|tx| actions::request_wire_in(tx, &current_id))
                .unwrap()
                .unwrap();
            // Also cover legacy recovery when current ownership is already a
            // reserved, successful or refused action rather than a queued one.
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        &key(&user),
                        1,
                        &serde_json::json!({"sent": current}),
                    )
                })
                .unwrap();
            drop(store);
            let store = open_at(&path, false);
            assert!(actions::reserve_pending(&store).unwrap().is_empty());
            let outcomes = actions::outcomes(&store).unwrap();
            let old = outcomes.iter().find(|a| a.id == old_id).unwrap();
            assert_eq!(old.status, ActionStatus::Queued);
            assert_eq!(
                old.deferred_reason,
                Some(ActionDeferReason::PrivacyTokenSuperseded)
            );
            assert_eq!(
                store
                    .read(|tx| actions::request_wire_in(tx, &current_id))
                    .unwrap()
                    .unwrap(),
                wire
            );
        }
    }

    #[test]
    fn missing_state_defers_without_blocking_other_actions_and_bad_owner_rolls_back() {
        use super::super::actions::{self, ActionDeferReason, ActionStatus};
        let dir = tempfile::tempdir().unwrap();
        let store = open_at(&dir.path().join("guards.redb"), true);
        let user = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let id = actions::queue(&store, ACTION_KIND, "missing state", |id| {
            issue_request(id, &user, now())
        })
        .unwrap();
        let other = actions::queue(&store, "control", "control", |id| {
            Ok(Node::new("iq").attr("id", id).attr("type", "get"))
        })
        .unwrap();
        let reserved = actions::reserve_pending(&store).unwrap();
        assert_eq!(reserved.len(), 1);
        assert_eq!(
            reserved[0].get_attr("id").unwrap().as_text(),
            Some(other.as_str())
        );
        let held = actions::outcomes(&store)
            .unwrap()
            .into_iter()
            .find(|a| a.id == id)
            .unwrap();
        assert_eq!(held.status, ActionStatus::Queued);
        assert_eq!(
            held.deferred_reason,
            Some(ActionDeferReason::PrivacyTokenStateMissing)
        );
        let foreign = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        let foreign_id = actions::queue(&store, ACTION_KIND, "foreign owner", |id| {
            issue_request(id, &foreign, now())
        })
        .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    &key(&user),
                    1,
                    &serde_json::json!({"sent": now(), "action_id": foreign_id}),
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let error = actions::reserve_pending(&store).unwrap_err().to_string();
        assert!(
            !error.contains(&user.to_string())
                && !error.contains(&id)
                && !error.contains(&foreign_id)
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        assert!(store.write(|tx| actions::reserve_one_in(tx, &id)).is_err());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
    }

    #[test]
    fn mandatory_expiry_floor_rejects_older_and_keeps_equality() {
        let current = 1_800_000_000;
        for delta in [-1, 0, 1] {
            let dir = tempfile::tempdir().unwrap();
            let store = open_at(&dir.path().join("boundary.redb"), true);
            let user = Jid::parse("15555550101@lid").unwrap();
            let timestamp = current - FALLBACK_SECONDS + delta;
            let id = store
                .write(|tx| schedule_after_ack_in(tx, &user, timestamp))
                .unwrap()
                .unwrap();
            let node = super::super::actions::request(&store, &id)
                .unwrap()
                .unwrap();
            let result = store
                .write(|tx| prepare_in(tx, &id, &node, current))
                .unwrap();
            match result {
                IssueReservation::Ready(prepared) => {
                    assert!(delta >= 0);
                    assert_eq!(token_time(&prepared), timestamp);
                }
                IssueReservation::Deferred(reason) => {
                    assert_eq!(delta, -1);
                    assert_eq!(
                        reason,
                        super::super::actions::ActionDeferReason::PrivacyTokenExpired
                    );
                }
            }
        }
    }

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
