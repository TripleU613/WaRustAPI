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

#[derive(serde::Serialize, serde::Deserialize)]
struct UploadBinding {
    key: String,
    payload_hash: [u8; 32],
}

fn public_uploaded(
    store: &ProtocolStore,
    key: &str,
) -> Result<(Zeroizing<Vec<u8>>, MediaKind, u64)> {
    use sha2::{Digest, Sha256};
    let suffix = key.strip_prefix("media-upload:").ok_or_else(bad)?;
    if suffix.len() != 64 || !suffix.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad());
    }
    let mut saved: serde_json::Value = store
        .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?
        .ok_or_else(bad)?;
    let hash = saved["hash"].as_str().ok_or_else(bad)?;
    if saved["status"] != "uploaded"
        || saved["to"] != ""
        || hash.starts_with("view-once")
        || hex::encode(Sha256::digest(hash.as_bytes())) != suffix
    {
        return Err(bad());
    }
    let kind = if hash.starts_with("ptt:") {
        MediaKind::Audio
    } else {
        MediaKind::Image
    };
    let length = saved["plaintext_len"]
        .as_u64()
        .filter(|n| *n > 0 && *n <= 8 * 1024 * 1024)
        .ok_or_else(bad)?;
    let payload = Zeroizing::new(
        serde_json::from_value::<Vec<u8>>(saved["protobuf"].take()).map_err(|_| bad())?,
    );
    if payload.is_empty() || payload.len() > 32768 {
        return Err(bad());
    }
    Ok((payload, kind, length))
}

#[cfg(any(test, feature = "comparison-client"))]
pub(crate) fn upload_reference(store: &ProtocolStore, key: &str) -> Result<MediaReference> {
    use sha2::{Digest, Sha256};
    let (payload, kind, length) = public_uploaded(store, key)?;
    let checkpoint = format!("{key}:reference");
    store.write(|tx| {
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
            mime_type: if kind == MediaKind::Audio {
                "audio/ogg; codecs=opus"
            } else {
                "image/png"
            }
            .into(),
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
    let mime = if kind == MediaKind::Audio {
        "audio/ogg; codecs=opus"
    } else {
        "image/png"
    };
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

fn uploaded(
    store: &ProtocolStore,
    id: &ClientRequestId,
    peer: &Jid,
) -> Result<(Zeroizing<Vec<u8>>, MediaKind)> {
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
    let saved: serde_json::Value = store
        .read(|tx| {
            tx.get_json(
                Table::Checkpoints,
                format!("message:{}:upload", id.as_str()).as_bytes(),
                1,
            )
        })?
        .ok_or_else(bad)?;
    let to = saved["to"].as_str().ok_or_else(bad)?;
    if saved["status"] != "uploaded" || to.strip_prefix('+').unwrap_or(to) != peer.user {
        return Err(bad());
    }
    let hash = saved["hash"].as_str().ok_or_else(bad)?;
    let kind = if hash.starts_with("ptt:") {
        MediaKind::Audio
    } else {
        MediaKind::Image
    };
    let payload = Zeroizing::new(
        serde_json::from_value::<Vec<u8>>(saved["protobuf"].clone()).map_err(|_| bad())?,
    );
    if payload.is_empty() || payload.len() > 32768 {
        return Err(bad());
    }
    Ok((payload, kind))
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
    let (payload, kind) = uploaded(store, id, peer)?;
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
        mime_type: match kind {
            MediaKind::Audio => "audio/ogg; codecs=opus",
            _ => "image/png",
        }
        .into(),
        length: None,
    };
    store.write(|tx| {
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
        let (payload, kind, length) = public_uploaded(store, &binding.key)?;
        verify_upload_binding(store, reference, &binding.key, &payload, kind, length)?;
        return Ok((
            payload,
            if kind == MediaKind::Audio {
                "ptt"
            } else {
                "image"
            },
        ));
    }
    let (uploaded, kind) = uploaded(store, id, peer)?;
    let payload = store.open_media_reference(&binding(id, peer), &reference.sealed)?;
    let (mime, media_type) = match kind {
        MediaKind::Audio => ("audio/ogg; codecs=opus", "ptt"),
        _ => ("image/png", "image"),
    };
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
