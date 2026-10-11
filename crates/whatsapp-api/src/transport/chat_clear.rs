//! Local "clear chat" (docs/protocol/chat-actions.md section 6).
//!
//! WhatsApp's clear chat is a local database operation plus a `clearChatAction`
//! `w:sync:app:state` mutation (`SyncActionValue.clearChatAction`, `LX/BQG`/`BQM`)
//! that carries the operation to companion devices. Writing app-state patches
//! (`CSe.A01`, the `LT-hash`/HKDF crypto in `LX/CKa`/`KmpLtHash16`) is a
//! separate, unimplemented work item (chat-actions.md section 6, confirmed
//! shape but not wired up here). What this module does is the local half
//! only: it never sends anything.
//!
//! Clearing keeps incoming `received:<chat>:<id>` and synchronized own-device
//! `received-own:<chat>:<id>` checkpoint rows so redeliveries stay deduplicated
//! (`inbox::process` treats a saved row as authoritative) and any receipt
//! correlation keyed on the same id keeps working. It removes only the
//! authenticated plaintext (`protobuf`) and marks the row `status: "cleared"`.
//! Own outgoing poll secrets (`poll-secret:<id>`, written in
//! `transport/outgoing.rs` with an explicit `chat` field) for the same chat
//! are deleted outright, since nothing needs to dedupe an outgoing send by id
//! the way incoming redelivery does.
use crate::{
    Result,
    model::{
        ids::ChatId,
        jid::{Jid, Server},
        time::Timestamp,
    },
    store::{ProtocolStore, Table},
    sync::SyncEvent,
};

fn invalid(reason: impl Into<String>) -> crate::Error {
    crate::Error::invalid("chat_clear", reason)
}

/// Counts of what a [`clear_chat`] call touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClearSummary {
    /// Incoming or synchronized own-device rows whose plaintext was removed.
    pub cleared_messages: usize,
    /// `poll-secret:` rows deleted for this chat.
    pub cleared_poll_secrets: usize,
}

/// Clear a chat's locally decrypted message bodies. Local-only: see the module
/// doc for what still needs the (unimplemented) `clearChatAction` app-state
/// mutation to reach companion devices.
///
/// Accepts a direct PN/LID chat, a group, or a
/// broadcast (`status@broadcast` and broadcast lists). All three key their
/// message rows by the chat JID — a direct message by its PN/LID,
/// a group or broadcast message by the chat JID that
/// `inbox::sender_key_chat` returns — so the prefix scan below serves all of
/// them. Anything else has no rows keyed by it and is refused rather than
/// silently clearing nothing.
pub fn clear_chat(store: &ProtocolStore, chat: &Jid) -> Result<ClearSummary> {
    if !matches!(
        chat.server,
        Server::User | Server::Lid | Server::Group | Server::Broadcast
    ) {
        return Err(invalid(
            "clear_chat supports direct chats, groups and broadcasts; received: rows are keyed by the chat JID",
        ));
    }
    let chat = chat.to_non_ad();
    let chat_id = ChatId::from_jid(&chat)?;
    let message_prefix = format!("received:{chat}:");
    let own_prefix = format!("received-own:{chat}:");
    let poll_chat = chat.to_string();
    store.write(|tx| {
        let mut summary = ClearSummary::default();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            let done = rows.len() < 256;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                if key.starts_with(message_prefix.as_bytes())
                    || key.starts_with(own_prefix.as_bytes())
                {
                    let Some(mut saved) =
                        tx.get_json::<serde_json::Value>(Table::Checkpoints, &key, 1)?
                    else {
                        continue;
                    };
                    if saved["status"].as_str() == Some("cleared") {
                        continue; // already cleared; redeliveries stay deduplicated
                    }
                    if let Some(obj) = saved.as_object_mut() {
                        obj.remove("protobuf");
                        obj.insert("status".into(), serde_json::json!("cleared"));
                    }
                    tx.put_json(Table::Checkpoints, &key, 1, &saved)?;
                    summary.cleared_messages += 1;
                } else if key.starts_with(b"poll-secret:") {
                    let Some(saved) =
                        tx.get_json::<serde_json::Value>(Table::Checkpoints, &key, 1)?
                    else {
                        continue;
                    };
                    if saved["chat"].as_str() == Some(poll_chat.as_str()) {
                        tx.delete(Table::Checkpoints, &key)?;
                        summary.cleared_poll_secrets += 1;
                    }
                }
            }
            if done {
                break;
            }
        }
        tx.append_event(SyncEvent::ChatCleared {
            chat: chat_id.clone(),
            cleared_at: Timestamp::now(),
        })?;
        Ok(summary)
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
            path: dir.path().join("cc.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([13u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, s)
    }

    #[test]
    fn clears_a_group_chat_and_leaves_other_chats_alone() {
        let (_d, s) = store();
        let group = Jid::parse("120363000000000001@g.us").unwrap();
        let other = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        s.write(|tx| {
            for (key, body) in [
                (format!("received:{group}:G1"), "group one"),
                (format!("received:{group}:G2"), "group two"),
                (format!("received:{other}:D1"), "direct one"),
            ] {
                tx.put_json(
                    Table::Checkpoints,
                    key.as_bytes(),
                    1,
                    &serde_json::json!({"status":"decrypted","protobuf":body.as_bytes()}),
                )?;
            }
            Ok(())
        })
        .unwrap();
        let summary = clear_chat(&s, &group).unwrap();
        assert_eq!(summary.cleared_messages, 2);
        let row = |key: String| {
            s.read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap()
        };
        // Group rows keep their key (so redeliveries still dedupe) but lose the
        // plaintext.
        for id in ["G1", "G2"] {
            let saved = row(format!("received:{group}:{id}"));
            assert_eq!(saved["status"], "cleared");
            assert!(saved.get("protobuf").is_none());
        }
        // The unrelated direct chat is untouched.
        let saved = row(format!("received:{other}:D1"));
        assert_eq!(saved["status"], "decrypted");
        assert!(saved.get("protobuf").is_some());
    }
    #[test]
    fn clear_chat_refuses_a_non_chat_jid() {
        let (_d, s) = store();
        assert!(clear_chat(&s, &Jid::parse("s.whatsapp.net").unwrap()).is_err());
    }
    #[test]
    fn clears_only_the_target_chats_messages_and_matching_poll_secrets() {
        let (_d, s) = store();
        let target = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                b"received:15551234567@s.whatsapp.net:MSG1",
                1,
                &serde_json::json!({
                    "status": "decrypted",
                    "protobuf": [1, 2, 3],
                    "capture": [9],
                    "route": [9],
                    "received_at": Timestamp::now(),
                }),
            )?;
            tx.put_json(
                Table::Checkpoints,
                b"received:15559999999@s.whatsapp.net:MSG2",
                1,
                &serde_json::json!({
                    "status": "decrypted",
                    "protobuf": [4, 5, 6],
                    "capture": [9],
                    "route": [9],
                    "received_at": Timestamp::now(),
                }),
            )?;
            tx.put_json(
                Table::Checkpoints,
                b"poll-secret:POLL1",
                1,
                &serde_json::json!({
                    "secret": vec![7u8; 32],
                    "options": ["Yes", "No"],
                    "name": "Lunch?",
                    "chat": "15551234567@s.whatsapp.net",
                    "creator_pn": "15550000000@s.whatsapp.net",
                    "creator_lid": null,
                }),
            )?;
            tx.put_json(
                Table::Checkpoints,
                b"poll-secret:POLL2",
                1,
                &serde_json::json!({
                    "secret": vec![8u8; 32],
                    "options": ["A", "B"],
                    "name": "Other chat",
                    "chat": "15559999999@s.whatsapp.net",
                    "creator_pn": "15550000000@s.whatsapp.net",
                    "creator_lid": null,
                }),
            )?;
            Ok(())
        })
        .unwrap();

        let summary = clear_chat(&s, &target).unwrap();
        assert_eq!(summary.cleared_messages, 1);
        assert_eq!(summary.cleared_poll_secrets, 1);

        let cleared: serde_json::Value = s
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    b"received:15551234567@s.whatsapp.net:MSG1",
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(cleared["status"], "cleared");
        assert!(cleared.get("protobuf").is_none());

        let untouched: serde_json::Value = s
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    b"received:15559999999@s.whatsapp.net:MSG2",
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(untouched["status"], "decrypted");
        assert!(untouched.get("protobuf").is_some());

        assert!(
            s.read(|tx| tx.get_json::<serde_json::Value>(
                Table::Checkpoints,
                b"poll-secret:POLL1",
                1
            ))
            .unwrap()
            .is_none()
        );
        assert!(
            s.read(|tx| tx.get_json::<serde_json::Value>(
                Table::Checkpoints,
                b"poll-secret:POLL2",
                1
            ))
            .unwrap()
            .is_some()
        );

        // Clearing again is idempotent: the already-cleared row is skipped and
        // its status/redelivery-dedup key is preserved.
        let second = clear_chat(&s, &target).unwrap();
        assert_eq!(second.cleared_messages, 0);
    }

    #[test]
    fn clears_incoming_and_synchronized_lid_rows_without_losing_route_markers() {
        let (_d, s) = store();
        s.write(|tx| {
            for key in ["received:12345@lid:IN", "received-own:12345@lid:OUT", "received-own:99999@lid:OTHER"] {
                tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &serde_json::json!({"status":"decrypted", "protobuf":[1,2,3], "source":"12345:2@lid", "route":[4,5,6]}))?;
            }
            Ok(())
        }).unwrap();
        assert_eq!(
            clear_chat(&s, &Jid::parse("12345@lid").unwrap())
                .unwrap()
                .cleared_messages,
            2
        );
        s.read(|tx| {
            for key in ["received:12345@lid:IN", "received-own:12345@lid:OUT"] {
                let row = tx
                    .get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1)?
                    .unwrap();
                assert_eq!(row["status"], "cleared");
                assert!(row["protobuf"].is_null());
                assert_eq!(row["route"], serde_json::json!([4, 5, 6]));
                assert_eq!(row["source"], "12345:2@lid");
            }
            assert_eq!(
                tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    b"received-own:99999@lid:OTHER",
                    1
                )?
                .unwrap()["status"],
                "decrypted"
            );
            Ok(())
        })
        .unwrap();
        assert_eq!(
            clear_chat(&s, &Jid::parse("12345@lid").unwrap())
                .unwrap()
                .cleared_messages,
            0
        );
    }

    /// A chat becomes clearable exactly when its messages become receivable,
    /// because both use the same `received:<chat>:<id>` keying. Groups became
    /// clearable with group receive, and broadcasts (status) with
    /// `inbox::sender_key_chat` routing — an earlier version of this test
    /// asserted `status@broadcast` must be refused, which silently became wrong
    /// two commits later and left status updates receivable but unclearable.
    #[test]
    fn accepts_every_chat_kind_that_can_receive_and_refuses_the_rest() {
        let (_d, s) = store();
        for clearable in [
            "12345-6789@g.us",            // group
            "15551234567@s.whatsapp.net", // direct
            "12345@lid",                  // direct LID
            "status@broadcast",           // status updates
            "12345@broadcast",            // broadcast list
        ] {
            assert!(
                clear_chat(&s, &Jid::parse(clearable).unwrap()).is_ok(),
                "{clearable} is receivable, so it must be clearable"
            );
        }
        // Nothing keys `received:` rows by these.
        for refused in ["s.whatsapp.net", "12345@bot"] {
            assert!(
                clear_chat(&s, &Jid::parse(refused).unwrap()).is_err(),
                "{refused} should not be clearable"
            );
        }
    }

    /// The rows a status update actually creates must be the rows clear_chat
    /// scans. This pins the two key schemes together so they cannot drift again.
    #[test]
    fn clears_status_rows_written_under_the_broadcast_chat_jid() {
        let (_d, s) = store();
        let status = Jid::parse("status@broadcast").unwrap();
        s.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                format!("received:{status}:S1").as_bytes(),
                1,
                &serde_json::json!({"status":"decrypted","protobuf":b"posted".as_slice()}),
            )
        })
        .unwrap();
        assert_eq!(clear_chat(&s, &status).unwrap().cleared_messages, 1);
        let saved = s
            .read(|tx| {
                tx.get_json::<serde_json::Value>(
                    Table::Checkpoints,
                    format!("received:{status}:S1").as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(saved["status"], "cleared");
        assert!(saved.get("protobuf").is_none());
    }
}
