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
        if (wrapped || image.view_once)
            && !super::media::supported_mime(super::media::MediaKind::Image, &image.mime)
        {
            // Pinned43.72 EwA.A03 compares the whole image MIME. A completed
            // ciphertext cache must not bypass this check during consumption.
            // See docs/protocol/media-network-outcomes.md (view-once MIME).
            return Err(invalid("unsupported view-once image MIME"));
        }
        return Ok((wrapped || image.view_once).then_some(image));
    }
    if let Some(wrapper) = wrappers.into_iter().flatten().next() {
        return descriptor(&Zeroizing::new(wrapper.message), depth + 1, true);
    }
    Ok(None)
}

fn url(image: &Image) -> Result<reqwest::Url> {
    if image.url.len() > 8192
        || !super::media::supported_mime(super::media::MediaKind::Image, &image.mime)
        || image.length == 0
        || image.length > 8 * 1024 * 1024
        || image.key.len() != 32
        || image.plaintext_hash.len() != 32
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
            candidate = Some((key, image, Zeroizing::new(bytes)));
        }
    }
    let (key, image, source) = candidate
        .ok_or_else(|| invalid("no unopened view-once image; consumed images cannot reopen"))?;
    consume_selected(store, &key, image, &source)
}

fn consume_selected(
    store: &ProtocolStore,
    key: &[u8],
    image: Image,
    source: &[u8],
) -> Result<(String, Zeroizing<Vec<u8>>)> {
    let state_key = format!("view-once-download:{}", hex::encode(Sha256::digest(key)));
    store.write(|tx| {
        // Bind consumption to the descriptor selected from this exact row,
        // before decrypting or replacing either checkpoint. See the view-once
        // consumption audit in docs/protocol/media-network-outcomes.md.
        ensure_view_once_source(tx, key, source)?;
        let mut state: serde_json::Value = tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1)?.ok_or_else(|| invalid("image must be downloaded first"))?;
        state_size(&state)?;
        let saved: serde_json::Value = tx.get_json(Table::Checkpoints, key, 1)?.ok_or_else(|| invalid("inbox record disappeared"))?;
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
        tx.put_json(Table::Checkpoints, key, 1, &consumed)?;
        Ok((hex::encode(Sha256::digest(key)), plaintext))
    })
}

/// Download the single authenticated view-once image in this account's inbox.
/// Explicit live authorization belongs to the caller. Refuses ambiguity, normal
/// images, replay after uncertainty, and consumed state. Returns ciphertext size.
pub async fn download_view_once(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
) -> Result<u64> {
    // Same pre-network account/config fence as ordinary media::open. A saved
    // inbox descriptor cannot authorize using another account's configuration.
    config.validate()?;
    let account = store
        .read(crate::session::AccountRecord::load)?
        .ok_or(Error::NotRegistered)?;
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::NotRegistered);
    }
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
            candidate = Some((key, image, Zeroizing::new(bytes)));
        }
    }
    let (key, image, source) = candidate.ok_or_else(|| {
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
    let reservation = reserve_download(store, &state_key, &key, &source)?;
    let mut response = client
        .get(url)
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .map_err(|_| invalid("download transport failed; reservation preserved"))?;
    let status = response.status().as_u16();
    if status != 200 {
        settle_download(
            store,
            &state_key,
            &reservation,
            &key,
            &source,
            &serde_json::json!({"status":"server_refused","http_status":status,"at":Timestamp::now()}),
        )?;
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
    settle_download(
        store,
        &state_key,
        &reservation,
        &key,
        &source,
        &serde_json::json!({
            "status":"downloaded_unopened","body":&*body,"encrypted_bytes":body.len(),"at":Timestamp::now(),"http_status":status
        }),
    )?;
    Ok(body.len() as u64)
}

// Local attempt ownership only; this identifier is never sent to WhatsApp.
fn reserve_download(
    store: &ProtocolStore,
    state_key: &str,
    source_key: &[u8],
    source: &[u8],
) -> Result<serde_json::Value> {
    let reservation = serde_json::json!({"status":"outcome_unknown",
        "at":Timestamp::now(),"reservation":super::messaging::new_message_id()?});
    store.write(|tx| {
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, state_key.as_bytes(), 1)?
            .is_some()
        {
            return Err(invalid("download already reserved"));
        }
        ensure_view_once_source(tx, source_key, source)?;
        tx.put_json(Table::Checkpoints, state_key.as_bytes(), 1, &reservation)
    })?;
    Ok(reservation)
}

fn settle_download(
    store: &ProtocolStore,
    state_key: &str,
    expected: &serde_json::Value,
    source_key: &[u8],
    source: &[u8],
    result: &serde_json::Value,
) -> Result<()> {
    // Match both records in the same transaction: a late HTTP result cannot
    // recreate removed state or replace another attempt/consumed outcome.
    // See docs/protocol/media-network-outcomes.md (view-once ownership audit).
    store.write(|tx| {
        let prior: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1)?;
        if expected["status"] != "outcome_unknown"
            || !expected["reservation"]
                .as_str()
                .is_some_and(|id| !id.is_empty())
            || prior.as_ref() != Some(expected)
        {
            return Err(invalid(
                "view-once download reservation changed before settlement",
            ));
        }
        ensure_view_once_source(tx, source_key, source)?;
        tx.put_json(Table::Checkpoints, state_key.as_bytes(), 1, result)
    })
}

fn ensure_view_once_source(
    tx: &crate::store::WriteTx<'_>,
    source_key: &[u8],
    source: &[u8],
) -> Result<()> {
    let current = tx
        .get(Table::Checkpoints, source_key, 1)?
        .map(Zeroizing::new);
    if current.as_ref().map(|row| row.as_slice()) != Some(source) {
        return Err(invalid("view-once source changed before state transition"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn view_once_mime_admission_precedes_cached_prefetch_and_consumption() {
        use crate::{
            model::ids::AccountId,
            session::{AccountRecord, RegistrationStatus},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let encrypted = crate::signal::image::encrypt(b"synthetic one-shot MIME fixture").unwrap();
        for wrapper in [0, 37, 55, 59] {
            for (mime, accepted) in [
                ("image/png", true),
                ("IMAGE/PNG", true),
                ("Image/Jpeg", true),
                ("image/png; extra=1", false),
                (" image/png", false),
                ("image/png ", false),
                ("application/was", false),
            ] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("once-mime.redb");
                let open_store = |create| {
                    ProtocolStore::open(OpenOptions {
                        path: path.clone(),
                        account_id: AccountId::new("fixture").unwrap(),
                        create,
                        key_source: std::sync::Arc::new(StaticKeySource::new(
                            MasterKey::from_bytes([9; 32]),
                        )),
                    })
                    .unwrap()
                };
                let store = open_store(true);
                let mut config: LabRegistrationConfig =
                    serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
                config.token = Zeroizing::new("offline-fixture".into());
                let image = Image {
                    url: "https://mmg.whatsapp.net/offline".into(),
                    mime: mime.into(),
                    plaintext_hash: encrypted.plaintext_hash.to_vec(),
                    length: encrypted.plaintext_len,
                    key: encrypted.media_key.to_vec(),
                    encrypted_hash: encrypted.encrypted_hash.to_vec(),
                    view_once: wrapper == 0,
                };
                let mut envelope = Envelope {
                    image: Some(image),
                    once: None,
                    once_v2: None,
                    once_ext: None,
                };
                if wrapper != 0 {
                    let inner = envelope.encode_to_vec();
                    envelope.image = None;
                    let wrapped = Some(Wrapper { message: inner });
                    match wrapper {
                        37 => envelope.once = wrapped,
                        55 => envelope.once_v2 = wrapped,
                        59 => envelope.once_ext = wrapped,
                        _ => unreachable!(),
                    }
                }
                let protobuf = envelope.encode_to_vec();
                if accepted {
                    assert!(url(&descriptor(&protobuf, 0, false).unwrap().unwrap()).is_ok());
                }
                let key = b"received:once-mime";
                let state_key = format!("view-once-download:{}", hex::encode(Sha256::digest(key)));
                store.write(|tx| {
                    AccountRecord {phone:Some(config.phone.clone()),registration:RegistrationStatus::Registered,
                        ..Default::default()}.save(tx)?;
                    tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":protobuf}))?;
                    tx.put_json(Table::Checkpoints,state_key.as_bytes(),1,&serde_json::json!({"status":"downloaded_unopened","body":encrypted.body,"encrypted_bytes":encrypted.body.len()}))
                }).unwrap();
                if accepted {
                    assert_eq!(
                        download_view_once(&store, &config).await.unwrap(),
                        encrypted.body.len() as u64
                    );
                    assert_eq!(
                        &*consume_view_once(&store).unwrap(),
                        b"synthetic one-shot MIME fixture"
                    );
                    assert!(consume_view_once(&store).is_err());
                    assert!(download_view_once(&store, &config).await.is_err());
                } else {
                    let rows = store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap();
                    let journal = store.journal_head().unwrap();
                    assert!(
                        download_view_once(&store, &config).await.is_err(),
                        "{wrapper} {mime}"
                    );
                    assert!(
                        consume_view_once_with_ticket(&store).is_err(),
                        "{wrapper} {mime}"
                    );
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        rows
                    );
                    assert_eq!(store.journal_head().unwrap(), journal);
                    drop(store);
                    let store = open_store(false);
                    assert!(download_view_once(&store, &config).await.is_err());
                    assert!(consume_view_once(&store).is_err());
                    assert_eq!(
                        store
                            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                            .unwrap(),
                        rows
                    );
                    assert_eq!(store.journal_head().unwrap(), journal);
                }
            }
        }
    }

    #[test]
    fn consumption_refuses_changed_source_without_consuming_either_record_after_reopen() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        for change in [
            "replacement",
            "metadata",
            "missing",
            "consumed",
            "ciphertext",
            "valid",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("consume-source.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create,
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [7; 32],
                    ))),
                })
                .unwrap()
            };
            let store = open(true);
            let encrypted = crate::signal::image::encrypt(b"synthetic view-once fixture").unwrap();
            let image = Image {
                url: "https://mmg.whatsapp.net/offline".into(),
                mime: "image/png".into(),
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
            // Select exactly as the public entry point does, before a later mutation.
            let selected = descriptor(&protobuf, 0, false).unwrap().unwrap();
            let key = b"received:offline:once";
            let state_key = format!("view-once-download:{}", hex::encode(Sha256::digest(key)));
            let inbox = serde_json::json!({"status":"decrypted","protobuf":protobuf,"route":{"id":"original"},"capture":"original"});
            store.write(|tx| {
                tx.put_json(Table::Checkpoints,key,1,&inbox)?;
                tx.put_json(Table::Checkpoints,state_key.as_bytes(),1,&serde_json::json!({"status":"downloaded_unopened","body":encrypted.body,"encrypted_bytes":encrypted.body.len()}))
            }).unwrap();
            let source = Zeroizing::new(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key, 1))
                    .unwrap()
                    .unwrap(),
            );
            store.write(|tx| {
                match change {
                    "replacement" => {
                        tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":[26,0],"sentinel":"replacement"}))?;
                    }
                    "metadata" => {
                        let mut updated = inbox.clone();updated["route"] = serde_json::json!({"id":"changed"});
                        tx.put_json(Table::Checkpoints,key,1,&updated)?;
                    }
                    "missing" => { tx.delete(Table::Checkpoints,key)?; }
                    "consumed" => { tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"consumed"}))?; }
                    "ciphertext" => {
                        tx.put_json(Table::Checkpoints,state_key.as_bytes(),1,&serde_json::json!({"status":"downloaded_unopened","body":[1,2,3],"encrypted_bytes":3}))?;
                    }
                    _ => {}
                }
                Ok(())
            }).unwrap();
            drop(store);
            let store = open(false);
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            let result = consume_selected(&store, key, selected, &source);
            if change == "valid" {
                let (ticket, plaintext) = result.unwrap();
                assert_eq!(ticket, hex::encode(Sha256::digest(key)));
                assert_eq!(&*plaintext, b"synthetic view-once fixture");
                for record_key in [key.as_slice(), state_key.as_bytes()] {
                    let saved: serde_json::Value = store
                        .read(|tx| tx.get_json(Table::Checkpoints, record_key, 1))
                        .unwrap()
                        .unwrap();
                    assert_eq!(saved["status"], "consumed");
                    assert_eq!(saved["opened_receipt"], "awaiting_viewer");
                    assert_eq!(saved["route"], inbox["route"]);
                    assert_eq!(saved["capture"], inbox["capture"]);
                    assert!(saved.get("body").is_none() && saved.get("protobuf").is_none());
                }
                drop(store);
                let reopened = open(false);
                assert!(consume_view_once(&reopened).is_err());
            } else {
                assert!(result.is_err(), "{change}");
                assert_eq!(store.journal_head().unwrap(), journal);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                drop(store);
                let reopened = open(false);
                assert_eq!(reopened.journal_head().unwrap(), journal);
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
            }
        }
    }

    #[test]
    fn late_view_once_results_preserve_reservations_and_source_after_reopen() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        use std::sync::Arc;
        for change in [
            "missing",
            "different_owner",
            "legacy_unknown",
            "consumed",
            "server_refused",
            "downloaded_unopened",
            "source_missing",
            "source_changed",
            "valid_success",
            "valid_refusal",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("view-once-owner.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create,
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
                })
                .unwrap()
            };
            let store = open(true);
            let source_key = b"received:offline:view-once";
            let inbox = serde_json::json!({"status":"decrypted","protobuf":[26,0]});
            store
                .write(|tx| tx.put_json(Table::Checkpoints, source_key, 1, &inbox))
                .unwrap();
            let source = store
                .read(|tx| tx.get(Table::Checkpoints, source_key, 1))
                .unwrap()
                .unwrap();
            let reservation = reserve_download(&store, "download", source_key, &source).unwrap();
            store.write(|tx| {
                match change {
                    "missing" => { tx.delete(Table::Checkpoints,b"download")?; }
                    "source_missing" => { tx.delete(Table::Checkpoints,source_key)?; }
                    "source_changed" => { tx.put_json(Table::Checkpoints,source_key,1,&serde_json::json!({"status":"consumed"}))?; }
                    "different_owner" => { tx.put_json(Table::Checkpoints,b"download",1,&serde_json::json!({"status":"outcome_unknown","reservation":"replacement"}))?; }
                    "legacy_unknown" => { tx.put_json(Table::Checkpoints,b"download",1,&serde_json::json!({"status":"outcome_unknown"}))?; }
                    "consumed" | "server_refused" | "downloaded_unopened" => { tx.put_json(Table::Checkpoints,b"download",1,&serde_json::json!({"status":change,"sentinel":"preserve"}))?; }
                    _ => {}
                }
                Ok(())
            }).unwrap();
            drop(store);
            let store = open(false);
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            let success = serde_json::json!({"status":"downloaded_unopened","body":[1,2,3],"encrypted_bytes":3});
            let refusal = serde_json::json!({"status":"server_refused","http_status":429});
            if change.starts_with("valid_") {
                let result = if change == "valid_success" {
                    &success
                } else {
                    &refusal
                };
                settle_download(
                    &store,
                    "download",
                    &reservation,
                    source_key,
                    &source,
                    result,
                )
                .unwrap();
                assert_eq!(
                    store
                        .read(|tx| tx.get_json::<serde_json::Value>(
                            Table::Checkpoints,
                            b"download",
                            1
                        ))
                        .unwrap()
                        .as_ref(),
                    Some(result)
                );
                let settled = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                let settled_journal = store.journal_head().unwrap();
                for late in [&success, &refusal] {
                    assert!(
                        settle_download(
                            &store,
                            "download",
                            &reservation,
                            source_key,
                            &source,
                            late
                        )
                        .is_err()
                    );
                }
                assert_eq!(store.journal_head().unwrap(), settled_journal);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    settled
                );
                drop(store);
                let reopened = open(false);
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    settled
                );
                assert!(reserve_download(&reopened, "download", source_key, &source).is_err());
            } else {
                for result in [&success, &refusal] {
                    assert!(
                        settle_download(
                            &store,
                            "download",
                            &reservation,
                            source_key,
                            &source,
                            result
                        )
                        .is_err(),
                        "{change}"
                    );
                }
                assert_eq!(store.journal_head().unwrap(), journal);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                drop(store);
                let reopened = open(false);
                assert_eq!(
                    reopened
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
            }
        }
    }

    #[test]
    fn view_once_reservation_refuses_changed_source_without_mutation() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("view-once-source.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
        })
        .unwrap();
        for prior in [None, Some(serde_json::json!({"status":"consumed"}))] {
            if let Some(prior) = prior {
                store
                    .write(|tx| tx.put_json(Table::Checkpoints, b"source", 1, &prior))
                    .unwrap();
            }
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            assert!(reserve_download(&store, "download", b"source", b"stale original").is_err());
            assert_eq!(store.journal_head().unwrap(), journal);
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                rows
            );
        }
    }

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
    fn view_once_descriptor_requires_both_hashes_before_any_get() {
        let mut image = Image {
            url: "https://mmg.whatsapp.net/offline".into(),
            mime: "image/png".into(),
            plaintext_hash: vec![1; 32],
            encrypted_hash: vec![2; 32],
            key: vec![3; 32],
            length: 42,
            view_once: true,
        };
        assert!(url(&image).is_ok());
        for length in [0, 31, 33] {
            image.plaintext_hash = vec![1; length];
            assert!(url(&image).is_err());
        }
    }

    #[tokio::test]
    async fn view_once_get_requires_valid_config_and_registered_matching_account() {
        use crate::{
            model::ids::AccountId,
            session::{AccountRecord, RegistrationStatus},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("account-fence.redb"),
            account_id: AccountId::new("view-once-test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
            create: true,
        })
        .unwrap();
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = Zeroizing::new("offline-fixture".into());
        assert!(matches!(
            download_view_once(&store, &config).await,
            Err(Error::NotRegistered)
        ));
        let other: LabRegistrationConfig = serde_json::from_str(
            &crate::registration::request::TEMPLATE_JSON.replace("2025550100", "2025550101"),
        )
        .unwrap();
        store
            .write(|tx| {
                AccountRecord {
                    phone: Some(other.phone),
                    registration: RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(matches!(
            download_view_once(&store, &config).await,
            Err(Error::NotRegistered)
        ));
        config.transport.timeout_ms = 0;
        assert!(matches!(
            download_view_once(&store, &config).await,
            Err(Error::InvalidInput { .. })
        ));
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
    }

    #[test]
    fn only_view_once_images_qualify_and_wrappers_are_bounded() {
        assert!(descriptor(&[26, 0], 0, false).unwrap().is_none());
        // Eligible one-shot descriptors now require a supported MIME. Keep
        // literal wire tags so wrapper classification is independently checked.
        let normal = [
            26, 11, 18, 9, b'i', b'm', b'a', b'g', b'e', b'/', b'p', b'n', b'g',
        ];
        assert!(descriptor(&normal, 0, false).unwrap().is_none());
        let mut once = normal.to_vec();
        once[1] += 3;
        once.extend([200, 1, 1]);
        assert!(descriptor(&once, 0, false).unwrap().is_some());
        let mut wrapped = vec![186, 3, 15, 10, 13];
        wrapped.extend(normal);
        assert!(descriptor(&wrapped, 0, false).unwrap().is_some());
        assert!(descriptor(&[26, 3, 200, 1, 1], 0, false).is_err());
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
