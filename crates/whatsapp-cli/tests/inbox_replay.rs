#![cfg(feature = "experimental-transport")]

use std::{process::Command, sync::Arc};
use whatsapp_api::{
    binary::Node,
    model::{ids::AccountId, jid::Jid},
    session::AccountRecord,
    signal::group,
    store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
    transport::{group_inbox, inbox},
};

#[tokio::test]
async fn large_inbox_replay_preserves_capture_and_deduplicates_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("history.redb");
    let open = |create| {
        ProtocolStore::open(OpenOptions {
            path: path.clone(),
            create,
            account_id: AccountId::new("fixture").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([23; 32]))),
        })
        .unwrap()
    };
    let store = open(true);
    let chat = Jid::parse("120363000000000001@g.us").unwrap();
    let sender = Jid::parse("15555550100:3@lid").unwrap();
    let account = AccountRecord {
        own_jid: Some(Jid::parse("15555550199@s.whatsapp.net").unwrap()),
        ..Default::default()
    };
    let key = group::prepare_own_sender_key(&chat, &sender, None)
        .await
        .unwrap();
    let (_, skdm) = group::extract_skdm_from_payload(&key.distribution_message).unwrap();
    let installed = group::process_skdm(
        &chat,
        &sender,
        group::sender_key_db_key(&chat, &sender),
        None,
        &skdm,
    )
    .await
    .unwrap();
    store
        .write(|tx| {
            account.save(tx)?;
            assert!(group::commit_skdm(tx, &installed)?);
            for i in 0..10_000 {
                tx.put(
                    Table::Checkpoints,
                    format!("history:{i:05}").as_bytes(),
                    1,
                    b"fixture",
                )?;
            }
            Ok(())
        })
        .unwrap();
    let payload = b"\x0a\x07example";
    let (ciphertext, _) = group::encrypt_group_payload(&chat, &sender, &key.updated_key, payload)
        .await
        .unwrap();
    let id = "HISTORY";
    let received_key = group_inbox::key(&chat, &sender, id).unwrap();
    let node = Node::new("message")
        .attr("from", chat)
        .attr("participant", sender)
        .attr("id", id)
        .attr("t", "123")
        .children(vec![
            Node::new("enc")
                .attr("type", "skmsg")
                .attr("v", "2")
                .bytes(ciphertext.to_vec()),
        ]);
    let capture = inbox::preserve(&store, &node).unwrap();
    let head = store.journal_head().unwrap();
    drop(store);
    let invoke = |flags: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_whatsapp-cli"))
            .env_remove("WHATSAPP_API_PROFILE")
            .args([
                "--store",
                path.to_str().unwrap(),
                "--account",
                "fixture",
                "--key-env",
                "WA_INBOX_REPLAY_TEST",
                "lab-inbox",
            ])
            .args(flags)
            .env("WA_INBOX_REPLAY_TEST", hex::encode([23; 32]))
            .output()
            .unwrap()
    };
    let run = |flags: &[&str]| {
        let output = invoke(flags);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    assert!(run(&["--inspect"]).contains("capture shape tag=message"));
    assert_eq!(open(false).journal_head().unwrap(), head);
    assert!(run(&["--replay-captures"]).contains("1 newly decrypted, 0 already committed"));
    let state = || {
        let store = open(false);
        store
            .read(|tx| {
                Ok((
                    tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        received_key.as_bytes(),
                        1,
                    )?
                    .unwrap(),
                    tx.scan(Table::SenderKeys, 1, 5)?,
                    tx.journal_head()?,
                ))
            })
            .unwrap()
    };
    let after = state();
    assert_eq!(after.0["capture"], serde_json::json!(capture));
    assert_eq!(after.0["protobuf"], serde_json::json!(payload));
    assert_eq!(after.2.0, head.0 + 1);
    assert!(run(&["--replay-captures"]).contains("0 newly decrypted, 1 already committed"));
    assert!(run(&[]).contains("0 newly decrypted payloads"));
    assert_eq!(state(), after);

    // Explicit recovery still visits later captures even when an earlier
    // frame is damaged, and the command reports the bad frame to the operator.
    open(false)
        .write(|tx| tx.put(Table::Checkpoints, b"inbound:0-bad", 1, b"invalid"))
        .unwrap();
    let output = invoke(&["--replay-captures"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid capture"));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("0 newly decrypted, 1 already committed")
    );
    assert_eq!(state(), after);
}
