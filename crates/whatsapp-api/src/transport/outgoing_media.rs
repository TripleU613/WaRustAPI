//! Bridge already-uploaded comparison media to the normal outbox. No uploads,
//! plaintext media files, or speculative retries are initiated by this module.
use crate::{
    Error, Result,
    model::{
        ids::ClientRequestId,
        jid::Jid,
        message::{MediaKind, MediaReference},
    },
    store::{ProtocolStore, Table},
};
use zeroize::Zeroizing;

const UPLOAD_BINDING: &[u8] = b"outgoing-ordinary-upload-v1";

/// The upload's kind from its saved content hash prefix (image_upload::input_hash).
fn media_kind(hash: &str) -> MediaKind {
    if hash.starts_with("ptt:") {
        MediaKind::Audio
    } else if hash.starts_with("doc:") {
        MediaKind::Document
    } else if hash.starts_with("video:") {
        MediaKind::Video
    } else {
        MediaKind::Image
    }
}

/// Local reference label; a document's real type travels in its payload.
fn kind_mime(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Audio => "audio/ogg; codecs=opus",
        MediaKind::Document => "application/octet-stream",
        MediaKind::Video => "video/mp4",
        _ => "image/png",
    }
}

/// The stanza `enc mediatype` (2.26.37.71 `ADx.A0O` table).
fn stanza_media_type(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Audio => "ptt",
        MediaKind::Document => "document",
        MediaKind::Video => "video",
        _ => "image",
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UploadBinding {
    key: String,
    payload_hash: [u8; 32],
}

fn operation_owner(store: &ProtocolStore) -> Result<[u8; 32]> {
    store.read(|tx| {
        let account = crate::session::AccountRecord::load(tx)?;
        super::account_authority::fingerprint(tx.generation(), account.as_ref())
    })
}
fn owned_upload(store: &ProtocolStore, key: &str) -> Result<serde_json::Value> {
    store.read(|tx| {
        let saved = tx
            .get_json(Table::Checkpoints, key.as_bytes(), 1)?
            .ok_or_else(bad)?;
        let account = crate::session::AccountRecord::load(tx)?;
        super::image_upload::check_saved_upload_owner(&saved, tx.generation(), account.as_ref())?;
        Ok(saved)
    })
}
fn check_reference_owner_in(
    tx: &crate::store::WriteTx<'_>,
    owner: [u8; 32],
    key: &str,
) -> Result<()> {
    super::media::check_download_owner_in(tx, owner)?;
    let saved = tx
        .get_json(Table::Checkpoints, key.as_bytes(), 1)?
        .ok_or_else(bad)?;
    let account = crate::session::AccountRecord::load_in_write(tx)?;
    super::image_upload::check_saved_upload_owner(&saved, tx.generation(), account.as_ref())
}

type PublicUpload = (Zeroizing<Vec<u8>>, MediaKind, u64, serde_json::Value);
type MessageUpload = (Zeroizing<Vec<u8>>, MediaKind, serde_json::Value);

fn check_reference_source_in(
    tx: &crate::store::WriteTx<'_>,
    key: &str,
    expected: &serde_json::Value,
) -> Result<()> {
    let current: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
    if current.as_ref() != Some(expected) {
        return Err(Error::invalid(
            "outbox_media",
            "upload changed before reference creation",
        ));
    }
    Ok(())
}

fn public_uploaded(store: &ProtocolStore, key: &str) -> Result<PublicUpload> {
    use sha2::{Digest, Sha256};
    let suffix = key.strip_prefix("media-upload:").ok_or_else(bad)?;
    if suffix.len() != 64 || !suffix.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad());
    }
    let saved = owned_upload(store, key)?;
    let hash = saved["hash"].as_str().ok_or_else(bad)?;
    if saved["status"] != "uploaded"
        || saved["to"] != ""
        || hash.starts_with("view-once")
        || hex::encode(Sha256::digest(hash.as_bytes())) != suffix
    {
        return Err(bad());
    }
    let kind = media_kind(hash);
    let length = saved["plaintext_len"]
        .as_u64()
        .filter(|n| *n > 0 && *n <= 8 * 1024 * 1024)
        .ok_or_else(bad)?;
    let payload = Zeroizing::new(
        serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()).map_err(|_| bad())?,
    );
    if payload.is_empty() || payload.len() > 32768 {
        return Err(bad());
    }
    Ok((payload, kind, length, saved))
}

#[cfg(any(test, feature = "comparison-client"))]
pub(crate) fn upload_reference(store: &ProtocolStore, key: &str) -> Result<MediaReference> {
    use sha2::{Digest, Sha256};
    let owner = operation_owner(store)?;
    let (payload, kind, length, source) = public_uploaded(store, key)?;
    let checkpoint = format!("{key}:reference");
    store.write(|tx| {
        check_reference_owner_in(tx, owner, key)?;
        check_reference_source_in(tx, key, &source)?;
        if let Some(reference) = tx.get_json(Table::Checkpoints, checkpoint.as_bytes(), 1)? {
            // No nested store read/write: verify the saved envelope against this
            // confirmed upload directly while holding the transaction.
            verify_upload_binding(store, &reference, key, &payload, kind, length)?;
            return Ok(reference);
        }
        let binding = serde_json::to_vec(&UploadBinding {
            key: key.to_owned(),
            payload_hash: Sha256::digest(&*payload).into(),
        })
        .map_err(|_| bad())?;
        let reference = MediaReference {
            sealed: store.seal_media_reference(UPLOAD_BINDING, &binding)?.into(),
            kind,
            mime_type: kind_mime(kind).into(),
            length: Some(length),
        };
        tx.put_json(Table::Checkpoints, checkpoint.as_bytes(), 1, &reference)?;
        Ok(reference)
    })
}

fn verify_upload_binding(
    store: &ProtocolStore,
    reference: &MediaReference,
    key: &str,
    payload: &[u8],
    kind: MediaKind,
    length: u64,
) -> Result<()> {
    use sha2::{Digest, Sha256};
    let binding: UploadBinding =
        serde_json::from_slice(&store.open_media_reference(UPLOAD_BINDING, &reference.sealed)?)
            .map_err(|_| bad())?;
    let digest: [u8; 32] = Sha256::digest(payload).into();
    let mime = kind_mime(kind);
    if binding.key != key
        || binding.payload_hash != digest
        || reference.kind != kind
        || reference.mime_type != mime
        || reference.length != Some(length)
    {
        return Err(bad());
    }
    Ok(())
}

fn bad() -> Error {
    Error::invalid(
        "outbox_media",
        "missing, mismatched or unconfirmed uploaded media",
    )
}
fn binding(id: &ClientRequestId, peer: &Jid) -> Vec<u8> {
    format!("{}:{peer}", id.as_str()).into_bytes()
}
fn reference_key(id: &ClientRequestId) -> String {
    format!("outbox-media-reference:{}", id.as_str())
}

fn uploaded(store: &ProtocolStore, id: &ClientRequestId, peer: &Jid) -> Result<MessageUpload> {
    if store
        .read(|tx| {
            tx.get(
                Table::Checkpoints,
                format!("message:{}:send", id.as_str()).as_bytes(),
                1,
            )
        })?
        .is_some()
    {
        return Err(bad());
    }
    let saved = owned_upload(store, &format!("message:{}:upload", id.as_str()))?;
    let to = saved["to"].as_str().ok_or_else(bad)?;
    // A group upload is bound to its chat, not an empty direct `to`. Never
    // interpret the same numeric user in another namespace as the recipient.
    let matches = if peer.server == crate::model::jid::Server::User
        && to.strip_prefix('+').unwrap_or(to) == peer.user
    {
        true
    } else {
        Jid::parse(to).is_ok_and(|j| j.to_non_ad() == peer.to_non_ad())
    };
    if saved["status"] != "uploaded" || !matches {
        return Err(bad());
    }
    let hash = saved["hash"].as_str().ok_or_else(bad)?;
    let kind = media_kind(hash);
    let payload = Zeroizing::new(
        serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()).map_err(|_| bad())?,
    );
    if payload.is_empty() || payload.len() > 32768 {
        return Err(bad());
    }
    Ok((payload, kind, saved))
}

/// Create or reuse a stable sealed reference to one successful upload. Call
/// only for the same request ID and recipient; uncertain uploads cannot qualify.
pub fn reference(
    store: &ProtocolStore,
    id: &ClientRequestId,
    peer: &Jid,
) -> Result<MediaReference> {
    if store
        .read(|tx| {
            tx.get(
                Table::Checkpoints,
                format!("message:{}:send", id.as_str()).as_bytes(),
                1,
            )
        })?
        .is_some()
    {
        return Err(Error::invalid(
            "outbox_media",
            "request already belongs to a lab send; no duplicate transmission",
        ));
    }
    let owner = operation_owner(store)?;
    let (payload, kind, source) = uploaded(store, id, peer)?;
    let old: Option<MediaReference> =
        store.read(|tx| tx.get_json(Table::Checkpoints, reference_key(id).as_bytes(), 1))?;
    if let Some(old) = old {
        resolve(store, id, peer, &old)?;
        return Ok(old);
    }
    let reference = MediaReference {
        sealed: store
            .seal_media_reference(&binding(id, peer), &payload)?
            .into(),
        kind,
        mime_type: kind_mime(kind).into(),
        length: None,
    };
    store.write(|tx| {
        let upload_key = format!("message:{}:upload", id.as_str());
        check_reference_owner_in(tx, owner, &upload_key)?;
        check_reference_source_in(tx, &upload_key, &source)?;
        if tx
            .get(Table::Checkpoints, reference_key(id).as_bytes(), 1)?
            .is_some()
        {
            return Err(bad());
        }
        tx.put_json(
            Table::Checkpoints,
            reference_key(id).as_bytes(),
            1,
            &reference,
        )
    })?;
    Ok(reference)
}

pub(super) fn resolve(
    store: &ProtocolStore,
    id: &ClientRequestId,
    peer: &Jid,
    reference: &MediaReference,
) -> Result<(Zeroizing<Vec<u8>>, &'static str)> {
    if let Ok(binding) = store.open_media_reference(UPLOAD_BINDING, &reference.sealed) {
        let binding: UploadBinding = serde_json::from_slice(&binding).map_err(|_| bad())?;
        let (payload, kind, length, _) = public_uploaded(store, &binding.key)?;
        verify_upload_binding(store, reference, &binding.key, &payload, kind, length)?;
        return Ok((payload, stanza_media_type(kind)));
    }
    let (uploaded, kind, _) = uploaded(store, id, peer)?;
    let payload = store.open_media_reference(&binding(id, peer), &reference.sealed)?;
    let (mime, media_type) = (kind_mime(kind), stanza_media_type(kind));
    if payload != uploaded
        || reference.kind != kind
        || reference.mime_type != mime
        || reference.length.is_some()
    {
        return Err(bad());
    }
    Ok((payload, media_type))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };
    #[test]
    fn reference_creation_rejects_replaced_upload_snapshot_without_writing_after_reopen() {
        use sha2::{Digest, Sha256};
        for public in [false, true] {
            for change in ["same", "payload", "status", "metadata", "removed"] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("reference-snapshot.redb");
                let open = |create| {
                    ProtocolStore::open(OpenOptions {
                        path: path.clone(),
                        account_id: AccountId::new("test").unwrap(),
                        create,
                        key_source: std::sync::Arc::new(StaticKeySource::new(
                            MasterKey::from_bytes([8; 32]),
                        )),
                    })
                    .unwrap()
                };
                let store = open(true);
                let id = ClientRequestId::new("snapshot").unwrap();
                let peer = Jid::parse("15550000000@s.whatsapp.net").unwrap();
                let hash = "ptt:synthetic";
                let key = if public {
                    format!(
                        "media-upload:{}",
                        hex::encode(Sha256::digest(hash.as_bytes()))
                    )
                } else {
                    "message:snapshot:upload".to_owned()
                };
                let initial = serde_json::json!({"status":"uploaded","to":if public { "" } else { &peer.user },
                    "hash":hash,"protobuf":[66,0],"plaintext_len":2});
                store
                    .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &initial))
                    .unwrap();
                let source = if public {
                    public_uploaded(&store, &key).unwrap().3
                } else {
                    uploaded(&store, &id, &peer).unwrap().2
                };
                let owner = operation_owner(&store).unwrap();
                let mut changed = initial.clone();
                match change {
                    "payload" => changed["protobuf"] = serde_json::json!([66, 1, 0]),
                    "status" => changed["status"] = "outcome_unknown".into(),
                    "metadata" => changed["extra"] = true.into(),
                    _ => {}
                }
                store
                    .write(|tx| {
                        if change == "removed" {
                            tx.delete(Table::Checkpoints, key.as_bytes()).map(|_| ())
                        } else {
                            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &changed)
                        }
                    })
                    .unwrap();
                drop(store);
                let store = open(false);
                let before = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                let journal = store.journal_head().unwrap();
                let outcome = store.write(|tx| {
                    check_reference_owner_in(tx, owner, &key)?;
                    check_reference_source_in(tx, &key, &source)?;
                    tx.put_json(Table::Checkpoints, b"reference-proof", 1, &true)
                });
                assert_eq!(outcome.is_ok(), change == "same");
                if change != "same" {
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        before
                    );
                    assert_eq!(store.journal_head().unwrap(), journal);
                }
            }
        }
    }
    #[test]
    fn uploaded_bridge_rechecks_identity_for_references_and_resolution_after_reopen() {
        use crate::session::{AccountRecord, RegistrationStatus};
        use sha2::{Digest, Sha256};
        for change in ["same", "jid", "registration-id", "logout"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("bridge-owner.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("test").unwrap(),
                    create,
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [8; 32],
                    ))),
                })
                .unwrap()
            };
            let store = open(true);
            let config: crate::registration::request::LabRegistrationConfig =
                serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
            let original = AccountRecord {
                generation: store.generation(),
                phone: Some(config.phone),
                registration: RegistrationStatus::Registered,
                ..Default::default()
            };
            let authority =
                super::super::account_authority::fingerprint(store.generation(), Some(&original))
                    .unwrap();
            let owner = serde_json::json!({"generation":store.generation(),"phone":original.phone,"authority":authority});
            let id = ClientRequestId::new("owned-bridge").unwrap();
            let peer = Jid::parse("15550000000@s.whatsapp.net").unwrap();
            let hash = "ptt:synthetic-owned";
            let public_key = format!(
                "media-upload:{}",
                hex::encode(Sha256::digest(hash.as_bytes()))
            );
            store.write(|tx| { original.save(tx)?;
                tx.put_json(Table::Checkpoints,b"message:owned-bridge:upload",1,
                    &serde_json::json!({"status":"uploaded","to":peer.user,"hash":hash,"protobuf":[66,0],"upload_owner_version":1,"upload_owner":owner}))?;
                tx.put_json(Table::Checkpoints,public_key.as_bytes(),1,
                    &serde_json::json!({"status":"uploaded","to":"","hash":hash,"protobuf":[66,0],"plaintext_len":2,"upload_owner_version":1,"upload_owner":owner}))
            }).unwrap();
            let message_reference = reference(&store, &id, &peer).unwrap();
            let public_reference = upload_reference(&store, &public_key).unwrap();
            let mut replacement = original.clone();
            match change {
                "jid" => replacement.own_jid = Some(Jid::parse("100@s.whatsapp.net").unwrap()),
                "registration-id" => replacement.signal_registration_id = Some(42),
                "logout" => replacement.registration = RegistrationStatus::Unregistered,
                _ => {}
            }
            store.write(|tx| replacement.save(tx)).unwrap();
            drop(store);
            let store = open(false);
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            assert_eq!(reference(&store, &id, &peer).is_ok(), change == "same");
            assert_eq!(
                upload_reference(&store, &public_key).is_ok(),
                change == "same"
            );
            assert_eq!(
                resolve(&store, &id, &peer, &message_reference).is_ok(),
                change == "same"
            );
            assert_eq!(
                resolve(&store, &id, &peer, &public_reference).is_ok(),
                change == "same"
            );
            assert_eq!(
                store
                    .write(|tx| check_reference_owner_in(tx, authority, &public_key))
                    .is_ok(),
                change == "same"
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
            if change != "same" {
                assert_eq!(store.journal_head().unwrap(), journal);
            }
        }
    }
    #[test]
    fn references_are_stable_scoped_and_refuse_unknown_uploads() {
        let dir = tempfile::tempdir().unwrap();
        let open = |name: &str| {
            ProtocolStore::open(OpenOptions {
                path: dir.path().join(name),
                account_id: AccountId::new("test").unwrap(),
                create: true,
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [8; 32],
                ))),
            })
            .unwrap()
        };
        let store = open("one.redb");
        let id = ClientRequestId::new("media-test").unwrap();
        let peer = Jid::parse("15550000000@s.whatsapp.net").unwrap();
        let save = |status: &str| {
            store.write(|tx| tx.put_json(Table::Checkpoints, b"message:media-test:upload", 1,
            &serde_json::json!({"status":status,"to":peer.user,"hash":"ptt:synthetic","protobuf":[66,0]}))).unwrap()
        };
        assert!(reference(&store, &id, &peer).is_err());
        save("outcome_unknown");
        assert!(reference(&store, &id, &peer).is_err());
        save("uploaded");
        let first = reference(&store, &id, &peer).unwrap();
        assert_eq!(reference(&store, &id, &peer).unwrap(), first);
        assert_eq!(resolve(&store, &id, &peer, &first).unwrap().1, "ptt");
        let other = Jid::parse("15550000001@s.whatsapp.net").unwrap();
        assert!(resolve(&store, &id, &other, &first).is_err());
        let mut tampered = first.clone();
        tampered.sealed = vec![1, 2, 3].into();
        assert!(resolve(&store, &id, &peer, &tampered).is_err());
        assert!(
            open("two.redb")
                .open_media_reference(&binding(&id, &peer), &first.sealed)
                .is_err()
        );
        drop(store);
        let store = open("one.redb");
        assert_eq!(reference(&store, &id, &peer).unwrap(), first);
    }
}
