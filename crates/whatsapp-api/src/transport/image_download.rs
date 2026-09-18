//! Explicit view-once prefetch: ciphertext only, never an ordinary file export.
use crate::{
    Error, Result,
    model::time::Timestamp,
    registration::request::LabRegistrationConfig,
    store::{ProtocolStore, Table},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("image_download", reason)
}

#[derive(Message)]
struct Image {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(string, tag = "2")]
    mime: String,
    #[prost(bytes = "vec", tag = "4")]
    plaintext_hash: Vec<u8>,
    #[prost(uint64, tag = "5")]
    length: u64,
    #[prost(bytes = "vec", tag = "8")]
    key: Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    encrypted_hash: Vec<u8>,
    #[prost(bool, tag = "25")]
    view_once: bool,
}
impl Drop for Image {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.key.zeroize();
        self.url.zeroize();
    }
}
#[derive(Message)]
struct Envelope {
    #[prost(message, optional, tag = "3")]
    image: Option<Image>,
    #[prost(message, optional, tag = "37")]
    once: Option<Wrapper>,
    #[prost(message, optional, tag = "55")]
    once_v2: Option<Wrapper>,
    #[prost(message, optional, tag = "59")]
    once_ext: Option<Wrapper>,
}
#[derive(Message)]
struct Wrapper {
    #[prost(bytes = "vec", tag = "1")]
    message: Vec<u8>,
}

fn descriptor(bytes: &[u8], depth: usize, wrapped: bool) -> Result<Option<Image>> {
    if bytes.len() > 65536 || depth > 4 {
        return Err(invalid("image envelope exceeds inspection bounds"));
    }
    let envelope = Envelope::decode(bytes).map_err(|_| invalid("invalid image message"))?;
    let wrappers = [envelope.once, envelope.once_v2, envelope.once_ext];
    if usize::from(envelope.image.is_some()) + wrappers.iter().filter(|v| v.is_some()).count() > 1 {
        return Err(invalid("ambiguous image envelope"));
    }
    if let Some(image) = envelope.image {
        return Ok((wrapped || image.view_once).then_some(image));
    }
    if let Some(wrapper) = wrappers.into_iter().flatten().next() {
        return descriptor(&Zeroizing::new(wrapper.message), depth + 1, true);
    }
    Ok(None)
}

fn url(image: &Image) -> Result<reqwest::Url> {
    if image.url.len() > 8192
        || !matches!(
            image.mime.as_str(),
            "image/jpeg" | "image/png" | "image/webp"
        )
        || image.length == 0
        || image.length > 8 * 1024 * 1024
        || image.key.len() != 32
        || image.encrypted_hash.len() != 32
    {
        return Err(invalid("missing or unsupported bounded image descriptor"));
    }
    let url = reqwest::Url::parse(&image.url).map_err(|_| invalid("invalid media URL"))?;
    if url.scheme() != "https"
        || !url
            .host_str()
            .is_some_and(super::image_upload::approved_host)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid("unapproved media URL"));
    }
    Ok(url)
}

fn state_size(state: &serde_json::Value) -> Result<u64> {
    if state["status"] != "downloaded_unopened" {
        return Err(invalid(
            "download already reserved, failed, or consumed; no replay",
        ));
    }
    state["encrypted_bytes"]
        .as_u64()
        .ok_or_else(|| invalid("invalid saved download state"))
}

/// Consume one downloaded view-once image before returning plaintext to a local
/// viewer. A crash after commit must not reopen it. This is local consumption,
/// not a server opened receipt. No plaintext is written to storage.
pub fn consume_view_once(store: &ProtocolStore) -> Result<Zeroizing<Vec<u8>>> {
    consume_view_once_with_ticket(store).map(|(_, bytes)| bytes)
}

/// Consume with an opaque ticket for recording successful display separately.
/// Consumption alone must never claim the viewer actually displayed the image.
pub fn consume_view_once_with_ticket(
    store: &ProtocolStore,
) -> Result<(String, Zeroizing<Vec<u8>>)> {
    let records = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
    if records.len() == 10000 {
        return Err(invalid("checkpoint scan limit reached"));
    }
    let mut candidate = None;
    for (key, bytes) in records {
        if !key.starts_with(b"received:") {
            continue;
        }
        let mut saved: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid inbox record"))?;
        if saved["status"] != "decrypted" {
            continue;
        }
        let protobuf: Zeroizing<Vec<u8>> = serde_json::from_value(saved["protobuf"].take())
            .map_err(|_| invalid("invalid saved protobuf"))?;
        if let Some(image) = descriptor(&protobuf, 0, false)? {
            if candidate.is_some() {
                return Err(invalid(
                    "multiple view-once images; explicit selection needed",
                ));
            }
            candidate = Some((key, image));
        }
    }
    let (key, image) = candidate
        .ok_or_else(|| invalid("no unopened view-once image; consumed images cannot reopen"))?;
    let state_key = format!("view-once-download:{}", hex::encode(Sha256::digest(&key)));
    store.write(|tx| {
        let mut state: serde_json::Value = tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1)?.ok_or_else(|| invalid("image must be downloaded first"))?;
        state_size(&state)?;
        let saved: serde_json::Value = tx.get_json(Table::Checkpoints, &key, 1)?.ok_or_else(|| invalid("inbox record disappeared"))?;
        if saved["status"] != "decrypted" { return Err(invalid("image already consumed")); }
        let body: Zeroizing<Vec<u8>> = serde_json::from_value(state["body"].take()).map_err(|_| invalid("invalid saved ciphertext"))?;
        let plaintext = crate::signal::image::decrypt(
            &body,
            image.key.as_slice().try_into().map_err(|_| invalid("invalid media key"))?,
            image.encrypted_hash.as_slice().try_into().map_err(|_| invalid("invalid encrypted hash"))?,
            image.plaintext_hash.as_slice().try_into().map_err(|_| invalid("invalid plaintext hash"))?,
            image.length,
        )?;
        // Replacing the records removes accessible media key/protobuf and body.
        // Database-page secure erasure is not claimed.
        let consumed = serde_json::json!({"status":"consumed", "at":Timestamp::now(), "opened_receipt":"awaiting_viewer", "route":saved["route"], "capture":saved["capture"]});
        tx.put_json(Table::Checkpoints, state_key.as_bytes(), 1, &consumed)?;
        tx.put_json(Table::Checkpoints, &key, 1, &consumed)?;
        Ok((hex::encode(Sha256::digest(&key)), plaintext))
    })
}

/// Download the single authenticated view-once image in this account's inbox.
/// Explicit live authorization belongs to the caller. Refuses ambiguity, normal
/// images, replay after uncertainty, and consumed state. Returns ciphertext size.
pub async fn download_view_once(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
) -> Result<u64> {
    let records = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
    if records.len() == 10000 {
        return Err(invalid("checkpoint scan limit reached"));
    }
    let mut candidate = None;
    for (key, bytes) in records {
        if !key.starts_with(b"received:") {
            continue;
        }
        let mut saved: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid inbox record"))?;
        if saved["status"] != "decrypted" {
            continue;
        }
        let protobuf: Zeroizing<Vec<u8>> = serde_json::from_value(saved["protobuf"].take())
            .map_err(|_| invalid("invalid saved protobuf"))?;
        if let Some(image) = descriptor(&protobuf, 0, false)? {
            if candidate.is_some() {
                return Err(invalid(
                    "multiple view-once images; explicit message selection needed",
                ));
            }
            candidate = Some((key, image));
        }
    }
    let (key, image) = candidate.ok_or_else(|| {
        invalid("no authenticated view-once image in saved inbox; nothing downloaded")
    })?;
    let state_key = format!("view-once-download:{}", hex::encode(Sha256::digest(&key)));
    let previous: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1))?;
    if let Some(previous) = previous {
        return state_size(&previous);
    }
    let url = url(&image)?;
    let client = super::image_upload::client(config)?;
    store.write(|tx| {
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, state_key.as_bytes(), 1)?
            .is_some()
        {
            return Err(invalid("download already reserved"));
        }
        tx.put_json(
            Table::Checkpoints,
            state_key.as_bytes(),
            1,
            &serde_json::json!({"status":"outcome_unknown","at":Timestamp::now()}),
        )
    })?;
    let mut response = client
        .get(url)
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .map_err(|_| invalid("download transport failed; reservation preserved"))?;
    let status = response.status().as_u16();
    if status != 200 {
        store.write(|tx|tx.put_json(Table::Checkpoints,state_key.as_bytes(),1,&serde_json::json!({"status":"server_refused","http_status":status,"at":Timestamp::now()})))?;
        return Err(Error::invalid(
            "image_download",
            format!("HTTP {status}; no retry"),
        ));
    }
    let expected = (image.length / 16 + 1) * 16 + 10;
    if response.content_length().is_some_and(|n| n != expected) {
        return Err(invalid("unexpected encrypted media length"));
    }
    let mut body = Zeroizing::new(Vec::new());
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| invalid("download interrupted; reservation preserved"))?
    {
        if (body.len() + chunk.len()) as u64 > expected {
            return Err(invalid("download exceeds descriptor bound"));
        }
        body.extend_from_slice(&chunk);
    }
    crate::signal::image::verify_encrypted(
        &body,
        image
            .key
            .as_slice()
            .try_into()
            .map_err(|_| invalid("invalid media key"))?,
        image
            .encrypted_hash
            .as_slice()
            .try_into()
            .map_err(|_| invalid("invalid media hash"))?,
        image.length,
    )?;
    store.write(|tx|tx.put_json(Table::Checkpoints,state_key.as_bytes(),1,&serde_json::json!({
        "status":"downloaded_unopened","body":&*body,"encrypted_bytes":body.len(),"at":Timestamp::now(),"http_status":status
    })))?;
    Ok(body.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_consumes_atomically_and_rejects_a_second_open() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("once.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap();
        let encrypted = crate::signal::image::encrypt(b"offline image").unwrap();
        let image = Image {
            url: "https://mmg.whatsapp.net/offline".into(),
            mime: "image/jpeg".into(),
            plaintext_hash: encrypted.plaintext_hash.to_vec(),
            length: encrypted.plaintext_len,
            key: encrypted.media_key.to_vec(),
            encrypted_hash: encrypted.encrypted_hash.to_vec(),
            view_once: true,
        };
        let protobuf = Envelope {
            image: Some(image),
            once: None,
            once_v2: None,
            once_ext: None,
        }
        .encode_to_vec();
        let key = b"received:offline:once";
        let state_key = format!("view-once-download:{}", hex::encode(Sha256::digest(key)));
        store.write(|tx| {
            tx.put_json(Table::Checkpoints, key, 1, &serde_json::json!({"status":"decrypted", "protobuf":protobuf}))?;
            tx.put_json(Table::Checkpoints, state_key.as_bytes(), 1, &serde_json::json!({"status":"downloaded_unopened", "body":encrypted.body, "encrypted_bytes":encrypted.body.len()}))
        }).unwrap();
        assert_eq!(&*consume_view_once(&store).unwrap(), b"offline image");
        assert!(consume_view_once(&store).is_err());
        for key in [key.as_slice(), state_key.as_bytes()] {
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key, 1))
                .unwrap()
                .unwrap();
            assert_eq!(saved["status"], "consumed");
            assert!(saved.get("body").is_none());
            assert!(saved.get("protobuf").is_none());
        }
    }
    #[test]
    fn media_url_rejects_credentials_redirect_targets_and_non_https() {
        let mut image = Image {
            url: "https://mmg.whatsapp.net/file".into(),
            mime: "image/jpeg".into(),
            plaintext_hash: vec![9; 32],
            length: 42,
            key: vec![7; 32],
            encrypted_hash: vec![8; 32],
            view_once: true,
        };
        assert!(url(&image).is_ok());
        for target in [
            "http://mmg.whatsapp.net/file",
            "https://localhost/file",
            "https://user@mmg.whatsapp.net/file",
            "https://mmg.whatsapp.net.evil.test/file",
            "https://mmg.whatsapp.net:444/file",
            "https://mmg.whatsapp.net/file#fragment",
        ] {
            image.url = target.into();
            assert!(url(&image).is_err());
        }
    }
    #[test]
    fn only_view_once_images_qualify_and_wrappers_are_bounded() {
        assert!(descriptor(&[26, 0], 0, false).unwrap().is_none());
        assert!(descriptor(&[26, 3, 200, 1, 1], 0, false).unwrap().is_some());
        assert!(
            descriptor(&[186, 3, 4, 10, 2, 26, 0], 0, false)
                .unwrap()
                .is_some()
        );
        assert!(descriptor(&[26, 0], 5, true).is_err());
        assert!(descriptor(&[26, 9], 0, false).is_err());
    }
    #[test]
    fn saved_download_state_does_not_reopen_or_replay() {
        for status in ["outcome_unknown", "server_refused", "consumed", "opened"] {
            assert!(
                state_size(&serde_json::json!({"status":status,"encrypted_bytes":42})).is_err()
            );
        }
        assert_eq!(
            state_size(&serde_json::json!({"status":"downloaded_unopened","encrypted_bytes":42}))
                .unwrap(),
            42
        );
    }
}
