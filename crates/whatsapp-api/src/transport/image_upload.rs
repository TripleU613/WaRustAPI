//! One-shot comparison MMS upload. All secret-bearing checkpoints stay encrypted.
use super::messaging::MessageRequest;
use crate::{
    Error, Result,
    binary::{Decoder, TokenDictionary, unpack_frame_payload},
    model::time::Timestamp,
    profile::Limits,
    registration::request::LabRegistrationConfig,
    signal::image,
    store::{ProtocolStore, Table},
};
use prost::Message;
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path, sync::Arc, time::Duration};
use zeroize::Zeroizing;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("image_upload", reason)
}
fn b64(bytes: &[u8]) -> String {
    crate::registration::token::standard_base64_padded(bytes)
        .replace('+', "-")
        .replace('/', "_")
}

// Comparison LX/BWp.dynamicMethod; outer LX/BXF imageMessage=3.
#[derive(Clone, PartialEq, Message)]
struct ImageMessage {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(string, tag = "2")]
    mimetype: String,
    #[prost(bytes = "vec", tag = "4")]
    file_sha256: Vec<u8>,
    #[prost(uint64, tag = "5")]
    file_length: u64,
    #[prost(uint32, tag = "6")]
    height: u32,
    #[prost(uint32, tag = "7")]
    width: u32,
    #[prost(bytes = "vec", tag = "8")]
    media_key: Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    file_enc_sha256: Vec<u8>,
    #[prost(string, tag = "11")]
    direct_path: String,
    #[prost(int64, tag = "12")]
    media_key_timestamp: i64,
    #[prost(bool, tag = "25")]
    view_once: bool,
}

// Comparison LX/BXF viewOnceMessageV2=55; LX/BM5 message=1.
#[derive(Clone, PartialEq, Message)]
struct ViewOnce {
    #[prost(message, optional, tag = "55")]
    inner: Option<WrappedMessage>,
}
#[derive(Clone, PartialEq, Message)]
struct WrappedMessage {
    #[prost(bytes = "vec", tag = "1")]
    message: Vec<u8>,
}
fn wrap_view_once(payload: Vec<u8>) -> Vec<u8> {
    ViewOnce {
        inner: Some(WrappedMessage { message: payload }),
    }
    .encode_to_vec()
}
#[derive(Clone, PartialEq, Message)]
struct Envelope {
    #[prost(message, optional, tag = "3")]
    image: Option<ImageMessage>,
    #[prost(message, optional, tag = "8")]
    audio: Option<super::voice_note::AudioMessage>,
}

type MediaInput = (Zeroizing<Vec<u8>>, u32, u32);
fn input(message: &MessageRequest) -> Result<Option<MediaInput>> {
    if let Some(path) = &message.voice_note {
        let file = std::fs::File::open(path)?;
        if !file.metadata()?.is_file() || file.metadata()?.len() > 8 * 1024 * 1024 {
            return Err(invalid("audio exceeds local file bound"));
        }
        let mut bytes = Zeroizing::new(Vec::new());
        file.take(8 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(invalid("audio exceeds local file bound"));
        }
        let seconds = super::voice_note::seconds(&bytes)?;
        return Ok(Some((bytes, seconds, 0)));
    }
    message.image.as_deref().map(read_png).transpose()
}

fn input_hash(message: &MessageRequest, bytes: &[u8]) -> String {
    let hash = hex::encode(Sha256::digest(bytes));
    if message.view_once {
        format!("view-once-image:{hash}")
    } else if message.voice_note.is_some() {
        format!("ptt:{hash}")
    } else {
        hash
    }
}

fn read_png(path: &Path) -> Result<(Zeroizing<Vec<u8>>, u32, u32)> {
    let file = std::fs::File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > 8 * 1024 * 1024 {
        return Err(invalid("PNG must be a regular file of at most 8 MiB"));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(8 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    let (width, height) = png_dimensions(&bytes)?;
    Ok((bytes, width, height))
}
fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32)> {
    if bytes.len() < 33
        || bytes.len() > 8 * 1024 * 1024
        || &bytes[..16] != b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"
    {
        return Err(invalid("only bounded PNG images are supported by this lab"));
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().expect("checked"));
    let height = u32::from_be_bytes(bytes[20..24].try_into().expect("checked"));
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 40_000_000 {
        return Err(invalid("invalid PNG dimensions"));
    }
    Ok((width, height))
}

/// Bind a saved send intent to exact source bytes, not just a mutable filename.
pub fn fingerprint(message: &MessageRequest) -> Result<serde_json::Value> {
    match input(message)? {
        None => Ok(serde_json::Value::Null),
        Some((bytes, _, _)) => Ok(serde_json::json!(input_hash(message, &bytes))),
    }
}

pub(super) fn approved_host(host: &str) -> bool {
    host.len() < 254
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        && [".whatsapp.net", ".whatsapp.com", ".fbcdn.net"]
            .iter()
            .any(|s| host.ends_with(s))
}

fn route(store: &ProtocolStore) -> Result<(String, Zeroizing<String>)> {
    let records = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
    if records.len() == 10000 {
        return Err(invalid("checkpoint scan limit reached"));
    }
    let latest = records
        .into_iter()
        .filter(|(key, _)| key.starts_with(b"media-route:"))
        .map(|(_, value)| {
            serde_json::from_slice::<serde_json::Value>(&value)
                .map_err(|_| invalid("invalid saved route"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max_by_key(|v| v["at"].as_i64().unwrap_or(0))
        .ok_or_else(|| invalid("no saved media route; run --login --media-route first"))?;
    if latest["status"] != "result" {
        return Err(invalid(
            "latest media route request is refused or uncertain",
        ));
    }
    let wire: Vec<u8> = serde_json::from_value(latest["wire"].clone())
        .map_err(|_| invalid("invalid saved route wire"))?;
    let limits = Limits::default();
    let data = unpack_frame_payload(&wire, &limits)?;
    let node =
        Decoder::new(&data, TokenDictionary::comparison_2_26_37_70(), &limits).decode_node()?;
    let conn = node
        .child("media_conn")
        .ok_or_else(|| invalid("missing media connection"))?;
    let attr = |name| conn.get_attr(name).and_then(|v| v.as_text());
    let ttl = attr("ttl")
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| invalid("missing route lifetime"))?;
    let auth_ttl = attr("auth_ttl")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(ttl);
    let age = Timestamp::now().0.saturating_sub(
        latest["at"]
            .as_i64()
            .ok_or_else(|| invalid("missing route timestamp"))?,
    );
    if age < 0 || ttl.min(auth_ttl) <= 0 || age >= ttl.min(auth_ttl).saturating_mul(1000) {
        return Err(invalid("saved media route expired; refresh it explicitly"));
    }
    let auth = attr("auth")
        .filter(|v| !v.is_empty() && v.len() <= 8192)
        .ok_or_else(|| invalid("missing upload auth"))?;
    let host = conn
        .child_nodes()
        .iter()
        .filter(|n| n.description == "host")
        .filter_map(|n| n.get_attr("hostname").and_then(|v| v.as_text()))
        .find(|h| approved_host(h))
        .ok_or_else(|| invalid("no supported HTTPS media host"))?;
    Ok((host.to_owned(), Zeroizing::new(auth.to_owned())))
}

fn upload_url(host: &str, auth: &str, seed: &[u8; 32], hash: &[u8; 32]) -> Result<reqwest::Url> {
    upload_url_kind(host, auth, seed, hash, "image")
}

fn upload_url_kind(
    host: &str,
    auth: &str,
    seed: &[u8; 32],
    hash: &[u8; 32],
    kind: &str,
) -> Result<reqwest::Url> {
    if !matches!(kind, "image" | "ptt") {
        return Err(invalid("unsupported media type"));
    }
    if !approved_host(host) {
        return Err(invalid("unapproved upload host"));
    }
    let mut digest = Sha256::new();
    digest.update(seed);
    digest.update(hash);
    let token = b64(&digest.finalize());
    let mut url = reqwest::Url::parse(&format!("https://{host}/mms/{kind}/{}", b64(hash)))
        .map_err(|_| invalid("invalid upload URL"))?;
    url.query_pairs_mut()
        .append_pair("direct_ip", "0")
        .append_pair("token", &token)
        .append_pair("auth", auth);
    Ok(url)
}

pub(super) fn client(config: &LabRegistrationConfig) -> Result<reqwest::Client> {
    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| invalid("TLS configuration failed"))?
    .with_root_certificates(roots)
    .with_no_client_auth();
    let mut builder = reqwest::Client::builder()
        .use_preconfigured_tls(tls)
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .referer(false)
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .timeout(Duration::from_millis(config.transport.timeout_ms))
        .connect_timeout(Duration::from_millis(config.transport.connect_timeout_ms));
    if let Some(ua) = &config.client.user_agent {
        builder = builder.user_agent(ua.as_str());
    }
    builder
        .build()
        .map_err(|_| invalid("cannot build upload client"))
}

fn locations(response: &serde_json::Value) -> Result<(String, String)> {
    let url = response["url"]
        .as_str()
        .filter(|s| s.len() < 8192)
        .ok_or_else(|| invalid("upload response missing URL"))?;
    let parsed = reqwest::Url::parse(url).map_err(|_| invalid("invalid returned media URL"))?;
    if parsed.scheme() != "https"
        || !parsed.host_str().is_some_and(approved_host)
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
    {
        return Err(invalid("unapproved returned media URL"));
    }
    let path = response["direct_path"]
        .as_str()
        .filter(|s| {
            s.starts_with('/')
                && !s.starts_with("//")
                && s.len() < 8192
                && !s.contains(['\r', '\n'])
        })
        .ok_or_else(|| invalid("upload response missing valid direct_path"))?;
    Ok((url.to_owned(), path.to_owned()))
}

fn saved_payload(
    old: &serde_json::Value,
    hash: &str,
    recipient: &str,
) -> Result<Zeroizing<Vec<u8>>> {
    if old["hash"] != hash || old["to"] != recipient {
        return Err(invalid(
            "request_id belongs to different image bytes or recipient",
        ));
    }
    if old["status"] != "uploaded" {
        return Err(invalid("saved upload refused or uncertain; no replay"));
    }
    let payload: Vec<u8> = serde_json::from_value(old["protobuf"].clone())
        .map_err(|_| invalid("invalid saved image protobuf"))?;
    if payload.is_empty() || payload.len() > 32768 {
        return Err(invalid("invalid saved image protobuf bounds"));
    }
    Ok(Zeroizing::new(payload))
}

/// Upload one encrypted PNG or Ogg/Opus voice note, or reuse its saved success. Never replay an uncertain
/// or refused upload. Call only after explicit live-request authorization.
pub async fn prepare(
    store: &ProtocolStore,
    message: &MessageRequest,
    config: &LabRegistrationConfig,
) -> Result<Option<Zeroizing<Vec<u8>>>> {
    let Some((bytes, width, height)) = input(message)? else {
        return Ok(None);
    };
    let hash = input_hash(message, &bytes);
    let key = format!("message:{}:upload", message.request_id);
    upload_prepared(
        store,
        config,
        PreparedUpload {
            bytes,
            width,
            height,
            hash,
            key,
            recipient: message.to.to_string(),
            voice: message.voice_note.is_some(),
            view_once: message.view_once,
        },
    )
    .await
    .map(Some)
}

struct PreparedUpload {
    bytes: Zeroizing<Vec<u8>>,
    width: u32,
    height: u32,
    hash: String,
    key: String,
    recipient: String,
    voice: bool,
    view_once: bool,
}

/// Stage an ordinary public-client upload independently of a recipient/message.
/// The content/kind checkpoint is stable across file renames and process restarts.
#[cfg(feature = "comparison-client")]
pub(crate) async fn upload_source(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
    source: &Path,
) -> Result<crate::model::message::MediaReference> {
    config.validate()?;
    let account = store
        .read(crate::session::AccountRecord::load)?
        .ok_or(Error::NotRegistered)?;
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::NotRegistered);
    }
    let mut prefix = [0; 4];
    std::fs::File::open(source)?.read_exact(&mut prefix)?;
    let voice = prefix == *b"OggS";
    let message = MessageRequest {
        to: Zeroizing::new(String::new()),
        text: Zeroizing::new(String::new()),
        image: (!voice).then(|| source.to_owned()),
        voice_note: voice.then(|| source.to_owned()),
        view_once: false,
        group: None,
        fresh_sessions: false,
        skdm_delivery: None,
        status: None,
        quote: None,
        request_id: String::new(),
    };
    let (bytes, width, height) = input(&message)?.ok_or_else(|| invalid("media source missing"))?;
    let hash = input_hash(&message, &bytes);
    let key = format!(
        "media-upload:{}",
        hex::encode(Sha256::digest(hash.as_bytes()))
    );
    upload_prepared(
        store,
        config,
        PreparedUpload {
            bytes,
            width,
            height,
            hash,
            key: key.clone(),
            recipient: String::new(),
            voice,
            view_once: false,
        },
    )
    .await?;
    super::outgoing_media::upload_reference(store, &key)
}

async fn upload_prepared(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
    input: PreparedUpload,
) -> Result<Zeroizing<Vec<u8>>> {
    let PreparedUpload {
        bytes,
        width,
        height,
        hash,
        key,
        recipient,
        voice,
        view_once,
    } = input;
    let old: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?;
    if let Some(old) = old {
        return saved_payload(&old, &hash, &recipient);
    }
    let (host, auth) = route(store)?;
    let encrypted = if voice {
        image::encrypt_audio(&bytes)?
    } else {
        image::encrypt(&bytes)?
    };
    let seed: [u8; 32] = store.write(|tx| {
        if let Some(seed) = tx.get_json(Table::Checkpoints, b"upload-token-random-bytes", 1)? {
            return Ok(seed);
        }
        let mut seed = [0; 32];
        getrandom::fill(&mut seed).map_err(|_| invalid("entropy unavailable"))?;
        tx.put_json(Table::Checkpoints, b"upload-token-random-bytes", 1, &seed)?;
        Ok(seed)
    })?;
    let url = if voice {
        upload_url_kind(&host, &auth, &seed, &encrypted.encrypted_hash, "ptt")?
    } else {
        upload_url(&host, &auth, &seed, &encrypted.encrypted_hash)?
    };
    let client = client(config)?;
    let timestamp = Timestamp::now();
    let mut saved = serde_json::json!({"status":"outcome_unknown", "hash":hash, "to":recipient,
        "plaintext_len":encrypted.plaintext_len,
        "media_key":encrypted.media_key.as_slice(), "encrypted_hash":encrypted.encrypted_hash,
        "body":encrypted.body, "upload_url":url.as_str(), "at":timestamp});
    store.write(|tx| {
        let previous: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
        if previous.is_some() {
            return Err(invalid("upload already reserved"));
        }
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)
    })?;
    tracing::info!(
        bytes = encrypted.body.len(),
        kind = if voice { "ptt" } else { "image" },
        "media upload reserved; sending one POST"
    );
    let mut response = client
        .post(url)
        .header("Accept-Encoding", "identity")
        .body(encrypted.body)
        .send()
        .await
        .map_err(|_| invalid("upload transport failed; outcome unknown, no retry"))?;
    let status = response.status().as_u16();
    let mut body = Zeroizing::new(Vec::new());
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| invalid("upload response interrupted; outcome unknown"))?
    {
        if body.len() + chunk.len() > 65536 {
            return Err(invalid(
                "upload response exceeds local bound; outcome unknown",
            ));
        }
        body.extend_from_slice(&chunk);
    }
    saved["http_status"] = status.into();
    saved["response"] = serde_json::json!(&*body);
    // Persist even malformed/refused responses before inspecting them.
    saved["status"] = "response_received".into();
    store.write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved))?;
    tracing::info!(status, "media upload response saved encrypted");
    if status != 200 {
        return Err(invalid("upload not accepted; no automatic retry"));
    }
    let response: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| invalid("invalid upload JSON; no automatic retry"))?;
    let (url, direct_path) = locations(&response)?;
    let payload = if voice {
        Zeroizing::new(
            Envelope {
                image: None,
                audio: Some(super::voice_note::AudioMessage {
                    url,
                    direct_path,
                    mimetype: "audio/ogg; codecs=opus".into(),
                    seconds: width,
                    ptt: true,
                    file_sha256: encrypted.plaintext_hash.to_vec(),
                    file_length: encrypted.plaintext_len,
                    media_key: encrypted.media_key.to_vec(),
                    file_enc_sha256: encrypted.encrypted_hash.to_vec(),
                    media_key_timestamp: timestamp.as_unix_seconds(),
                }),
            }
            .encode_to_vec(),
        )
    } else {
        Zeroizing::new(
            Envelope {
                audio: None,
                image: Some(ImageMessage {
                    url,
                    direct_path,
                    mimetype: "image/png".into(),
                    file_sha256: encrypted.plaintext_hash.to_vec(),
                    file_length: encrypted.plaintext_len,
                    height,
                    width,
                    view_once,
                    media_key: encrypted.media_key.to_vec(),
                    file_enc_sha256: encrypted.encrypted_hash.to_vec(),
                    media_key_timestamp: timestamp.as_unix_seconds(),
                }),
            }
            .encode_to_vec(),
        )
    };
    let payload = if view_once {
        Zeroizing::new(wrap_view_once(payload.to_vec()))
    } else {
        payload
    };
    saved["protobuf"] = serde_json::json!(&*payload);
    saved["status"] = "uploaded".into();
    store.write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved))?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "comparison-client")]
    #[tokio::test]
    async fn public_upload_reuses_confirmed_content_and_enters_normal_outbox_offline() {
        use crate::{
            WhatsAppClient,
            model::{
                ids::{AccountId, ChatId, ClientRequestId},
                jid::Jid,
                message::MessageContent,
                send::SendRequest,
            },
            session::{AccountRecord, RegistrationStatus},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        let source = dir.path().join("fixture.png");
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&1u32.to_be_bytes());
        png.extend_from_slice(&1u32.to_be_bytes());
        png.resize(33, 0);
        std::fs::write(&source, &png).unwrap();
        let hash = hex::encode(Sha256::digest(&png));
        let key = format!(
            "media-upload:{}",
            hex::encode(Sha256::digest(hash.as_bytes()))
        );
        let keys = Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32])));
        let options = || OpenOptions {
            path: path.clone(),
            account_id: AccountId::new("test").unwrap(),
            key_source: keys.clone(),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = Zeroizing::new("offline-fixture".into());
        store.write(|tx| {
            AccountRecord { phone: Some(config.phone.clone()), registration: RegistrationStatus::Registered, ..Default::default() }.save(tx)?;
            tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &serde_json::json!({"status":"uploaded","hash":hash,"to":"","plaintext_len":33,"protobuf":[26,0]}))
        }).unwrap();
        drop(store);
        let client = WhatsAppClient::builder()
            .store_path(path.clone())
            .account_id(AccountId::new("test").unwrap())
            .key_source(keys.clone())
            .create_store(false)
            .comparison_transport(config.clone())
            .build()
            .await
            .unwrap();
        let reference = client.upload_media(source.clone()).await.unwrap();
        assert_eq!(
            client.upload_media(source.clone()).await.unwrap(),
            reference
        );
        let peer = Jid::parse("15550000000@s.whatsapp.net").unwrap();
        let request_id = ClientRequestId::new("public-media-test").unwrap();
        let ticket = client
            .send_message(SendRequest {
                client_request_id: request_id.clone(),
                chat: ChatId::from_jid(&peer).unwrap(),
                content: MessageContent::Media {
                    reference: reference.clone(),
                    caption: None,
                    file_name: None,
                },
            })
            .await
            .unwrap();
        assert!(matches!(
            client.connection_state().borrow().state,
            crate::runtime::ConnectionState::Stopped
        ));
        client.shutdown().await;
        drop(client);
        let store = ProtocolStore::open(options()).unwrap();
        assert_eq!(
            upload_source(&store, &config, &source).await.unwrap(),
            reference
        );
        assert_eq!(
            super::super::outgoing_media::resolve(&store, &request_id, &peer, &reference)
                .unwrap()
                .1,
            "image"
        );
        assert!(
            super::super::outgoing::begin_selected(&store, Some(ticket.local_id))
                .unwrap()
                .is_some()
        );
        assert!(
            super::super::outgoing::begin_selected(&store, Some(ticket.local_id))
                .unwrap()
                .is_none()
        );
        let mut tampered = reference.clone();
        tampered.length = Some(99);
        assert!(
            super::super::outgoing_media::resolve(&store, &request_id, &peer, &tampered).is_err()
        );
        let other = ProtocolStore::open(OpenOptions {
            path: dir.path().join("other.redb"),
            ..options()
        })
        .unwrap();
        assert!(
            super::super::outgoing_media::resolve(&other, &request_id, &peer, &reference).is_err()
        );
        store
            .write(|tx| {
                let mut saved: serde_json::Value =
                    tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                saved["protobuf"] = serde_json::json!([26, 1, 0]);
                tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)
            })
            .unwrap();
        assert!(
            super::super::outgoing_media::resolve(&store, &request_id, &peer, &reference).is_err()
        );
        assert!(upload_source(&store, &config, &source).await.is_err());
        store
            .write(|tx| {
                let mut saved: serde_json::Value =
                    tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?.unwrap();
                saved["status"] = "outcome_unknown".into();
                tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)
            })
            .unwrap();
        let error = upload_source(&store, &config, &source).await.unwrap_err();
        assert!(error.to_string().contains("no replay"));
        assert!(
            super::super::outgoing_media::resolve(&store, &request_id, &peer, &reference).is_err()
        );
    }

    #[test]
    fn view_once_uses_apk_flag_and_v2_wrapper_and_distinct_intent() {
        let inner = Envelope {
            image: Some(ImageMessage {
                view_once: true,
                ..Default::default()
            }),
            audio: None,
        }
        .encode_to_vec();
        let wrapped = wrap_view_once(inner);
        assert_eq!(wrapped, [186, 3, 7, 10, 5, 26, 3, 200, 1, 1]);
        let summary = crate::signal::direct::payload_summary(&wrapped).unwrap();
        assert!(summary.contains("view-once-v2"));
        assert!(summary.contains("view_once=true"));
        let mut request: MessageRequest = serde_json::from_str(
            r#"{"to":"15551234567","request_id":"test","image":"/tmp/image.png"}"#,
        )
        .unwrap();
        let regular = input_hash(&request, b"same image");
        request.view_once = true;
        assert_ne!(regular, input_hash(&request, b"same image"));
    }
    #[test]
    fn upload_url_matches_apk_padded_urlsafe_hash_and_escaped_auth() {
        assert!(
            upload_url_kind("mmg.whatsapp.net", "auth", &[0; 32], &[0; 32], "ptt")
                .unwrap()
                .path()
                .starts_with("/mms/ptt/")
        );
        assert_eq!(
            Envelope {
                image: None,
                audio: Some(super::super::voice_note::AudioMessage {
                    seconds: 2,
                    ptt: true,
                    ..Default::default()
                })
            }
            .encode_to_vec(),
            [66, 4, 40, 2, 48, 1]
        );
        let url = upload_url("mmg.whatsapp.net", "a+/=&", &[0; 32], &[255; 32]).unwrap();
        assert_eq!(
            url.path(),
            "/mms/image/__________________________________________8="
        );
        let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(query["auth"], "a+/=&");
        assert_eq!(query["direct_ip"], "0");
        assert_eq!(query["token"].len(), 44);
        assert!(query["token"].ends_with('='));
        for host in [
            "127.0.0.1",
            "whatsapp.net.evil.test",
            "evil.test/whatsapp.net",
            "evil@mmg.whatsapp.net",
            "mmg.whatsapp.net:80",
        ] {
            assert!(upload_url(host, "auth", &[0; 32], &[0; 32]).is_err());
        }
    }
    #[test]
    fn uploaded_payload_reused_but_uncertain_refused_or_changed_inputs_block() {
        let mut value = serde_json::json!({"status":"uploaded","hash":"hash-a","to":"test-recipient","protobuf":[26,0]});
        assert_eq!(
            &*saved_payload(&value, "hash-a", "test-recipient").unwrap(),
            &[26, 0]
        );
        assert!(saved_payload(&value, "hash-b", "test-recipient").is_err());
        assert!(saved_payload(&value, "hash-a", "another-recipient").is_err());
        for status in ["outcome_unknown", "response_received", "refused"] {
            value["status"] = status.into();
            assert!(saved_payload(&value, "hash-a", "test-recipient").is_err());
        }
    }
    #[test]
    fn response_locations_reject_untrusted_urls_and_absolute_paths() {
        let good =
            serde_json::json!({"url":"https://mmg.whatsapp.net/file", "direct_path":"/file"});
        assert!(locations(&good).is_ok());
        for url in [
            "http://mmg.whatsapp.net/file",
            "https://localhost/file",
            "https://user@mmg.whatsapp.net/file",
            "https://mmg.whatsapp.net:444/file",
        ] {
            let mut bad = good.clone();
            bad["url"] = url.into();
            assert!(locations(&bad).is_err());
        }
        for path in ["https://evil.test/file", "//evil.test/file", "/file\r\n"] {
            let mut bad = good.clone();
            bad["direct_path"] = path.into();
            assert!(locations(&bad).is_err());
        }
    }
    #[test]
    fn image_proto_uses_apk_tags_and_png_bounds() {
        let image = ImageMessage {
            mimetype: "image/png".into(),
            width: 1024,
            height: 576,
            ..Default::default()
        };
        let bytes = Envelope {
            image: Some(image),
            audio: None,
        }
        .encode_to_vec();
        // Hand-encoded wire fixture: Message.image=3, MIME=2, height=6, width=7.
        assert_eq!(
            bytes,
            [
                26, 17, 18, 9, b'i', b'm', b'a', b'g', b'e', b'/', b'p', b'n', b'g', 48, 192, 4,
                56, 128, 8
            ]
        );
        assert!(png_dimensions(&[]).is_err());
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&1024u32.to_be_bytes());
        png.extend_from_slice(&576u32.to_be_bytes());
        png.resize(33, 0);
        assert_eq!(png_dimensions(&png).unwrap(), (1024, 576));
        png[16..20].fill(0);
        assert!(png_dimensions(&png).is_err());
    }
}
