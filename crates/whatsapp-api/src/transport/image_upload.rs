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

// APK 2.26.43.72 MtQ.A01, default finished/non-newsletter LKZ path.
// Cipher hash excludes this transport seal. Android JSONStringer escapes '/',
// unlike serde_json, so serialize this fixed ASCII schema explicitly in order.
// Evidence: docs/protocol/rea/2.26.43.72-media-upload-http/README.md.
fn finished_voice_seal(hash: &[u8; 32]) -> Option<Vec<u8>> {
    let hash = crate::registration::token::standard_base64_padded(hash).replace('/', "\\/");
    let json = format!("{{\"status\":\"sent\",\"hash\":\"{hash}\"}}");
    // MtQ.A01 catches its >100-byte metadata exception and returns null;
    // K2k still completes with no EOF bytes. Never print the hash/JSON.
    if json.len() + 12 > 100 {
        return None;
    }
    let mut seal = Vec::with_capacity(json.len() + 12);
    seal.extend_from_slice(b"RIFF");
    seal.extend_from_slice(json.as_bytes());
    seal.extend_from_slice(&((json.len() as u32) ^ 0x5614_050b).to_le_bytes());
    seal.extend_from_slice(b"META");
    Some(seal)
}

fn upload_wire_body(encrypted: &image::EncryptedImage, voice: bool) -> Vec<u8> {
    let mut body = encrypted.body.clone();
    if voice && let Some(seal) = finished_voice_seal(&encrypted.encrypted_hash) {
        body.extend_from_slice(&seal);
    }
    body
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
    #[prost(message, optional, tag = "7")]
    document: Option<DocumentMessage>,
    #[prost(message, optional, tag = "8")]
    audio: Option<super::voice_note::AudioMessage>,
    #[prost(message, optional, tag = "9")]
    video: Option<VideoMessage>,
}

/// `VideoMessage` (`Message` field 9). **Confirmed static, 2.26.37.71**
/// `xtask apk-proto-fields --class 'LX/BZE;'`: url 1, mimetype 2,
/// fileSha256 3, fileLength 4, seconds 5, mediaKey 6, height 9, width 10,
/// fileEncSha256 11, directPath 13, mediaKeyTimestamp 14.
#[derive(Clone, PartialEq, Message)]
struct VideoMessage {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(string, tag = "2")]
    mimetype: String,
    #[prost(bytes = "vec", tag = "3")]
    file_sha256: Vec<u8>,
    #[prost(uint64, tag = "4")]
    file_length: u64,
    #[prost(uint32, tag = "5")]
    seconds: u32,
    #[prost(bytes = "vec", tag = "6")]
    media_key: Vec<u8>,
    #[prost(uint32, tag = "9")]
    height: u32,
    #[prost(uint32, tag = "10")]
    width: u32,
    #[prost(bytes = "vec", tag = "11")]
    file_enc_sha256: Vec<u8>,
    #[prost(string, tag = "13")]
    direct_path: String,
    #[prost(int64, tag = "14")]
    media_key_timestamp: i64,
}

/// Duration (whole seconds, at least 1) and display size of an MP4, read
/// from `moov/mvhd` and the first `trak/tkhd` with a picture. Bounded walk;
/// anything else is refused.
fn mp4_info(bytes: &[u8]) -> Result<(u32, u32, u32)> {
    let bad = || invalid("expected an MP4 with moov/mvhd and a video track");
    fn boxes(data: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
        let mut at = 0usize;
        std::iter::from_fn(move || {
            let head = data.get(at..at + 8)?;
            let size = u32::from_be_bytes(head[..4].try_into().ok()?) as usize;
            if size < 8 {
                return None;
            }
            let body = data.get(at + 8..at + size)?;
            let kind = &head[4..8];
            at += size;
            Some((kind, body))
        })
    }
    if !boxes(bytes).any(|(k, _)| k == b"ftyp") {
        return Err(bad());
    }
    let moov = boxes(bytes).find(|(k, _)| *k == b"moov").ok_or_else(bad)?.1;
    let mvhd = boxes(moov).find(|(k, _)| *k == b"mvhd").ok_or_else(bad)?.1;
    let (scale, duration) = match mvhd.first() {
        Some(0) => (
            u32::from_be_bytes(mvhd.get(12..16).ok_or_else(bad)?.try_into().unwrap()) as u64,
            u32::from_be_bytes(mvhd.get(16..20).ok_or_else(bad)?.try_into().unwrap()) as u64,
        ),
        Some(1) => (
            u32::from_be_bytes(mvhd.get(20..24).ok_or_else(bad)?.try_into().unwrap()) as u64,
            u64::from_be_bytes(mvhd.get(24..32).ok_or_else(bad)?.try_into().unwrap()),
        ),
        _ => return Err(bad()),
    };
    if scale == 0 {
        return Err(bad());
    }
    let seconds = u32::try_from(duration.div_ceil(scale))
        .map_err(|_| bad())?
        .max(1);
    for (kind, trak) in boxes(moov) {
        if kind != b"trak" {
            continue;
        }
        if let Some((_, tkhd)) = boxes(trak).find(|(k, _)| *k == b"tkhd")
            && tkhd.len() >= 8
        {
            let tail = &tkhd[tkhd.len() - 8..];
            let width = u32::from_be_bytes(tail[..4].try_into().unwrap()) >> 16;
            let height = u32::from_be_bytes(tail[4..].try_into().unwrap()) >> 16;
            if width > 0 && height > 0 {
                return Ok((seconds, width, height));
            }
        }
    }
    Err(bad())
}

/// `DocumentMessage` (`Message` field 7). **Confirmed static, 2.26.37.71**
/// `xtask apk-proto-fields --class 'LX/BZ6;'`: mimetype 2, title 3,
/// fileSha256 4, fileLength 5, mediaKey 7, fileName 8, fileEncSha256 9,
/// directPath 10, mediaKeyTimestamp 11. There is no field 1 (`url`) in this
/// build, so none is sent.
#[derive(Clone, PartialEq, Message)]
struct DocumentMessage {
    #[prost(string, tag = "2")]
    mimetype: String,
    #[prost(string, tag = "3")]
    title: String,
    #[prost(bytes = "vec", tag = "4")]
    file_sha256: Vec<u8>,
    #[prost(uint64, tag = "5")]
    file_length: u64,
    #[prost(bytes = "vec", tag = "7")]
    media_key: Vec<u8>,
    #[prost(string, tag = "8")]
    file_name: String,
    #[prost(bytes = "vec", tag = "9")]
    file_enc_sha256: Vec<u8>,
    #[prost(string, tag = "10")]
    direct_path: String,
    #[prost(int64, tag = "11")]
    media_key_timestamp: i64,
}

/// File name and MIME type of a document input; the type comes from the
/// extension for a few common kinds, else `application/octet-stream`.
fn document_meta(path: &Path) -> Result<(String, String)> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty() && n.len() <= 255)
        .ok_or_else(|| invalid("document needs a UTF-8 file name"))?
        .to_owned();
    let mime = match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("csv") => "text/csv",
        Some("zip") => "application/zip",
        Some("json") => "application/json",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    };
    Ok((name, mime.to_owned()))
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
    if let Some(path) = &message.video {
        let file = std::fs::File::open(path)?;
        if !file.metadata()?.is_file() || file.metadata()?.len() > 8 * 1024 * 1024 {
            return Err(invalid("video must be a regular MP4 file of at most 8 MiB"));
        }
        let mut bytes = Zeroizing::new(Vec::new());
        file.take(8 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(invalid("video must be a regular MP4 file of at most 8 MiB"));
        }
        let (_, width, height) = mp4_info(&bytes)?;
        return Ok(Some((bytes, width, height)));
    }
    if let Some(path) = &message.document {
        let file = std::fs::File::open(path)?;
        if !file.metadata()?.is_file() || file.metadata()?.len() > 8 * 1024 * 1024 {
            return Err(invalid("document must be a regular file of at most 8 MiB"));
        }
        let mut bytes = Zeroizing::new(Vec::new());
        file.take(8 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
            return Err(invalid("document must be 1 byte to 8 MiB"));
        }
        return Ok(Some((bytes, 0, 0)));
    }
    message.image.as_deref().map(read_png).transpose()
}

fn input_hash(message: &MessageRequest, bytes: &[u8]) -> String {
    let hash = hex::encode(Sha256::digest(bytes));
    if message.view_once {
        format!("view-once-image:{hash}")
    } else if message.voice_note.is_some() {
        format!("ptt:{hash}")
    } else if message.video.is_some() {
        format!("video:{hash}")
    } else if let Some(path) = &message.document {
        // The name and type are part of what is sent, so they bind the upload.
        let (name, mime) = document_meta(path).unwrap_or_default();
        format!(
            "doc:{hash}:{}",
            hex::encode(Sha256::digest(format!("{name}\n{mime}")))
        )
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
    let latest = match super::media_routing::refresh::active_record(store)? {
        Some(saved) => saved,
        None => {
            let records = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
            if records.len() == 10000 {
                return Err(invalid("checkpoint scan limit reached"));
            }
            records
                .into_iter()
                .filter(|(key, _)| key.starts_with(b"media-route:"))
                .map(|(_, value)| {
                    serde_json::from_slice::<serde_json::Value>(&value)
                        .map_err(|_| invalid("invalid saved route"))
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .max_by_key(|v| v["at"].as_i64().unwrap_or(0))
                .ok_or_else(|| invalid("no saved media route; run --login --media-route first"))?
        }
    };
    if latest["status"] != "result" {
        return Err(invalid(
            "latest media route request is refused or uncertain",
        ));
    }
    let wire: Vec<u8> = serde_json::from_value(latest["wire"].clone())
        .map_err(|_| invalid("invalid saved route wire"))?;
    let limits = Limits::default();
    let data = unpack_frame_payload(&wire, &limits)?;
    let prospective = super::media_routing::refresh::effective_reply(&latest)?;
    let bound = prospective.is_some();
    let node = match prospective {
        Some(node) => node,
        None => {
            Decoder::new(&data, TokenDictionary::comparison_2_26_37_71(), &limits).decode_node()?
        }
    };
    let conn = node
        .child("media_conn")
        .ok_or_else(|| invalid("missing media connection"))?;
    // Prospective rows were recomputed above. Legacy standalone rows still
    // lack prior-route binding and cannot unlock a non-new response.
    if !bound && !super::media_routing::is_new_route(conn)? {
        return Err(invalid(
            "unchanged media route needs prior host binding; inspect saved response",
        ));
    }
    let conn = conn.clone();
    let attr = |name| conn.get_attr(name).and_then(|v| v.as_text());
    // APK 43.72 1xY.CK4 -> 0Tp.A07/A0L/A09 requires both fields and
    // uses Java Long.parseLong. No missing-auth-TTL fallback exists there.
    // Evidence: docs/protocol/rea/2.26.43.72-media-route-lifetimes/README.md.
    let ttl = attr("ttl")
        .and_then(super::mex::java_decimal_long)
        .ok_or_else(|| invalid("missing or invalid route lifetime"))?;
    let auth_ttl = attr("auth_ttl")
        .and_then(super::mex::java_decimal_long)
        .ok_or_else(|| invalid("missing or invalid upload auth lifetime"))?;
    let age = Timestamp::now().0.saturating_sub(
        // APK 43.72 0VM.A07 records request handoff time for 1xY.CK4.
        // Historical `at` was receipt time and cannot reconstruct that base.
        latest["requested_at"]
            .as_i64()
            .filter(|at| *at > 0)
            .ok_or_else(|| {
                invalid("saved media route lacks request time; refresh it explicitly")
            })?,
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
    if !matches!(kind, "image" | "ptt" | "document" | "video") {
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
    // APK 43.72 M0g.A00 accepts ordinary direct_path only when Uri has
    // both oe and oh. Match encoded query keys exactly; empty values count.
    // Android Uri.getQueryParameter compares raw encoded keys, not decoded
    // aliases, and excludes the fragment. No signature/expiry verification.
    let query = path
        .split('#')
        .next()
        .unwrap_or(path)
        .split_once('?')
        .map(|(_, q)| q);
    let has =
        |key| query.is_some_and(|q| q.split('&').any(|pair| pair.split('=').next() == Some(key)));
    if !has("oe") || !has("oh") {
        return Err(invalid(
            "upload direct_path missing signature or expiry parameters",
        ));
    }
    Ok((url.to_owned(), path.to_owned()))
}

fn replace_upload_state(
    store: &ProtocolStore,
    key: &str,
    expected: &serde_json::Value,
    next: &serde_json::Value,
) -> Result<()> {
    store.write(|tx| {
        let current: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
        if current.as_ref() != Some(expected) {
            return Err(invalid(
                "upload reservation changed before response settlement",
            ));
        }
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, next)
    })
}

// Observation is independent of mutable upload authority, and never authorizes retry.
fn capture_upload_response(
    store: &ProtocolStore,
    key: &str,
    reservation: &serde_json::Value,
    status: u16,
    body: Option<&[u8]>,
) -> Result<String> {
    let capture = format!("{key}:response:{}", super::messaging::new_message_id()?);
    store.write(|tx| tx.put_json(Table::Checkpoints, capture.as_bytes(), 1,
        &serde_json::json!({"http_status":status,"body":body,"encrypted_hash":reservation["encrypted_hash"],"at":Timestamp::now()})))?;
    Ok(capture)
}

// Header receipt is distinct from a complete usable upload result. Persist it
// before streaming the bounded body; interruptions must not erase known status.
fn retain_response_headers(
    store: &ProtocolStore,
    key: &str,
    status: u16,
    expected: &serde_json::Value,
) -> Result<serde_json::Value> {
    if !(100..=599).contains(&status) {
        return Err(invalid("invalid upload HTTP status"));
    }
    store.write(|tx| {
        let mut saved: serde_json::Value = tx
            .get_json(Table::Checkpoints, key.as_bytes(), 1)?
            .ok_or_else(|| invalid("upload response has no reservation"))?;
        if &saved != expected {
            return Err(invalid(
                "upload reservation changed before response headers",
            ));
        }
        if saved["status"] != "outcome_unknown" {
            return Err(invalid("upload headers cannot replace an existing outcome"));
        }
        saved["http_status"] = status.into();
        saved["status"] = "response_headers_received".into();
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)?;
        Ok(saved)
    })
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
    prepare_bound(store, message, config, message.to.to_string()).await
}

/// Normal-outbox upload with an explicit group-chat binding. This is private
/// artifact provenance, not a new field sent to the media server. The legacy
/// comparison entry point retains its original saved `to` values.
pub async fn prepare_outbox(
    store: &ProtocolStore,
    message: &MessageRequest,
    config: &LabRegistrationConfig,
) -> Result<Option<Zeroizing<Vec<u8>>>> {
    let recipient = match message.group_chat()? {
        Some(group) => group.to_string(),
        None => message.to.to_string(),
    };
    prepare_bound(store, message, config, recipient).await
}

async fn prepare_bound(
    store: &ProtocolStore,
    message: &MessageRequest,
    config: &LabRegistrationConfig,
    recipient: String,
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
            recipient,
            voice: message.voice_note.is_some(),
            document: message.document.as_deref().map(document_meta).transpose()?,
            video: message.video.as_ref().map(|_| ()),
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
    /// File name and MIME type when the upload is a document.
    document: Option<(String, String)>,
    /// Set when the upload is an MP4 video.
    video: Option<()>,
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
        document: None,
        video: None,
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
            document: None,
            video: None,
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
        document,
        video,
        view_once,
    } = input;
    let old: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?;
    if let Some(old) = old {
        return saved_payload(&old, &hash, &recipient);
    }
    let (host, auth) = route(store)?;
    let encrypted = if video.is_some() {
        image::encrypt_video(&bytes)?
    } else if document.is_some() {
        image::encrypt_document(&bytes)?
    } else if voice {
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
    let url = if video.is_some() {
        upload_url_kind(&host, &auth, &seed, &encrypted.encrypted_hash, "video")?
    } else if document.is_some() {
        upload_url_kind(&host, &auth, &seed, &encrypted.encrypted_hash, "document")?
    } else if voice {
        upload_url_kind(&host, &auth, &seed, &encrypted.encrypted_hash, "ptt")?
    } else {
        upload_url(&host, &auth, &seed, &encrypted.encrypted_hash)?
    };
    let client = client(config)?;
    let wire_body = upload_wire_body(&encrypted, voice);
    let timestamp = Timestamp::now();
    let saved = serde_json::json!({"status":"outcome_unknown", "hash":hash, "to":recipient,
        "plaintext_len":encrypted.plaintext_len,
        "media_key":encrypted.media_key.as_slice(), "encrypted_hash":encrypted.encrypted_hash,
        "body":wire_body, "encrypted_body_len":encrypted.body.len(),
        "upload_body_format":if voice { "finished_voice_seal_v1" } else { "encrypted_file_v1" },
        "upload_url":url.as_str(), "at":timestamp});
    store.write(|tx| {
        let previous: Option<serde_json::Value> =
            tx.get_json(Table::Checkpoints, key.as_bytes(), 1)?;
        if previous.is_some() {
            return Err(invalid("upload already reserved"));
        }
        tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &saved)
    })?;
    tracing::info!(
        bytes = wire_body.len(),
        kind = if voice { "ptt" } else { "image" },
        "media upload reserved; sending one POST"
    );
    let mut response = client
        .post(url)
        .header("Accept-Encoding", "identity")
        .body(wire_body)
        .send()
        .await
        .map_err(|_| invalid("upload transport failed; outcome unknown, no retry"))?;
    let status = response.status().as_u16();
    capture_upload_response(store, &key, &saved, status, None)?;
    let mut saved = retain_response_headers(store, &key, status, &saved)?;
    let headers = saved.clone();
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
    capture_upload_response(store, &key, &saved, status, Some(&body))?;
    saved["http_status"] = status.into();
    saved["response"] = serde_json::json!(&*body);
    // Persist even malformed/refused responses before inspecting them.
    saved["status"] = "response_received".into();
    replace_upload_state(store, &key, &headers, &saved)?;
    let response_state = saved.clone();
    tracing::info!(status, "media upload response saved encrypted");
    if status != 200 {
        return Err(invalid("upload not accepted; no automatic retry"));
    }
    let response: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| invalid("invalid upload JSON; no automatic retry"))?;
    let (url, direct_path) = locations(&response)?;
    let payload = if video.is_some() {
        let (seconds, width, height) = mp4_info(&bytes)?;
        Zeroizing::new(
            Envelope {
                image: None,
                audio: None,
                document: None,
                video: Some(VideoMessage {
                    url,
                    mimetype: "video/mp4".into(),
                    file_sha256: encrypted.plaintext_hash.to_vec(),
                    file_length: encrypted.plaintext_len,
                    seconds,
                    media_key: encrypted.media_key.to_vec(),
                    height,
                    width,
                    file_enc_sha256: encrypted.encrypted_hash.to_vec(),
                    direct_path,
                    media_key_timestamp: timestamp.as_unix_seconds(),
                }),
            }
            .encode_to_vec(),
        )
    } else if let Some((file_name, mimetype)) = document {
        let _ = url; // DocumentMessage has no url field in 2.26.37.71.
        Zeroizing::new(
            Envelope {
                video: None,
                image: None,
                audio: None,
                document: Some(DocumentMessage {
                    mimetype,
                    title: file_name.clone(),
                    file_sha256: encrypted.plaintext_hash.to_vec(),
                    file_length: encrypted.plaintext_len,
                    media_key: encrypted.media_key.to_vec(),
                    file_name,
                    file_enc_sha256: encrypted.encrypted_hash.to_vec(),
                    direct_path,
                    media_key_timestamp: timestamp.as_unix_seconds(),
                }),
            }
            .encode_to_vec(),
        )
    } else if voice {
        Zeroizing::new(
            Envelope {
                image: None,
                document: None,
                video: None,
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
                document: None,
                video: None,
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
    replace_upload_state(store, &key, &response_state, &saved)?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    // Existing phase tests use the current record; production passes its frozen
    // initiating reservation explicitly to the parent function.
    fn test_response_headers(
        store: &ProtocolStore,
        key: &str,
        status: u16,
    ) -> Result<serde_json::Value> {
        let current = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))?
            .unwrap_or(serde_json::Value::Null);
        retain_response_headers(store, key, status, &current)
    }

    #[test]
    fn upload_response_stages_refuse_replaced_state_and_preserve_observations_after_reopen() {
        use super::*;
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("upload-owner.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("fixture").unwrap(),
                create,
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            })
            .unwrap()
        };
        let store = open(true);
        let key = "upload";
        let expected = serde_json::json!({"status":"outcome_unknown","hash":"original","encrypted_hash":[1,2,3]});
        let replacement = serde_json::json!({"status":"outcome_unknown","hash":"replacement"});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &replacement))
            .unwrap();
        let capture =
            capture_upload_response(&store, key, &expected, 200, Some(b"private-response"))
                .unwrap();
        let journal = store.journal_head().unwrap();
        assert!(retain_response_headers(&store, key, 200, &expected).is_err());
        for phase in ["response_headers_received", "response_received"] {
            let old = serde_json::json!({"status":phase,"hash":"original"});
            assert!(
                replace_upload_state(&store, key, &old, &serde_json::json!({"status":"uploaded"}))
                    .is_err()
            );
        }
        assert_eq!(store.journal_head().unwrap(), journal);
        drop(store);
        let store = open(false);
        assert_eq!(
            store
                .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap(),
            Some(replacement)
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, capture.as_bytes(), 1))
                .unwrap()
                .is_some()
        );
        // Each exact phase can progress, but its old snapshot cannot settle twice.
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &expected))
            .unwrap();
        let headers = retain_response_headers(&store, key, 200, &expected).unwrap();
        let body = serde_json::json!({"status":"response_received","http_status":200});
        replace_upload_state(&store, key, &headers, &body).unwrap();
        assert!(replace_upload_state(&store, key, &headers, &body).is_err());
        let uploaded = serde_json::json!({"status":"uploaded","protobuf":[1,2]});
        replace_upload_state(&store, key, &body, &uploaded).unwrap();
        assert!(replace_upload_state(&store, key, &body, &uploaded).is_err());
    }

    #[test]
    fn finished_voice_seal_has_android_json_order_slash_escaping_and_length() {
        let seal = finished_voice_seal(&[0; 32]).unwrap();
        let json =
            b"{\"status\":\"sent\",\"hash\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\"}";
        assert_eq!(&seal[..4], b"RIFF");
        assert_eq!(&seal[4..seal.len() - 8], json);
        assert_eq!(
            &seal[seal.len() - 8..seal.len() - 4],
            &((json.len() as u32) ^ 0x5614_050b).to_le_bytes()
        );
        assert_eq!(&seal[seal.len() - 4..], b"META");
        let mut hash = [0; 32];
        hash[0] = 0xff;
        let escaped = finished_voice_seal(&hash).unwrap();
        assert!(escaped.windows(3).any(|w| w == b"\\/w"));
        assert!(finished_voice_seal(&[0xff; 32]).is_none());
    }

    #[test]
    fn voice_wire_seal_preserves_cipher_hash_and_other_media_bytes() {
        // Deterministic opaque cipher+MAC fixture; encryption itself has separate
        // known-vector tests in signal::image.
        let body: Vec<u8> = (0..74).collect();
        let encrypted = image::EncryptedImage {
            encrypted_hash: Sha256::digest(&body).into(),
            body,
            media_key: Zeroizing::new([0; 32]),
            plaintext_hash: [0; 32],
            plaintext_len: 48,
        };
        let wire = upload_wire_body(&encrypted, true);
        assert_eq!(&wire[..encrypted.body.len()], encrypted.body);
        assert_eq!(
            &wire[encrypted.body.len()..],
            finished_voice_seal(&encrypted.encrypted_hash).unwrap()
        );
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(&encrypted.body)),
            encrypted.encrypted_hash
        );
        assert_ne!(
            <[u8; 32]>::from(Sha256::digest(&wire)),
            encrypted.encrypted_hash
        );
        assert_eq!(upload_wire_body(&encrypted, false), encrypted.body);
    }

    #[test]
    fn required_java_auth_lifetime_never_falls_back_to_route_ttl() {
        use super::*;
        use crate::{
            binary::{Encoder, Node},
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("route-ttl.redb"),
            account_id: AccountId::new("route-ttl-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
            create: true,
        })
        .unwrap();
        for (auth_ttl, accepted) in [
            (None, false),
            (Some("3600"), true),
            (Some("+3600"), true),
            (Some("٣٦٠٠"), true),
            (Some("３６００"), true),
            (Some(""), false),
            (Some("invalid"), false),
            (Some("9223372036854775808"), false),
            (Some("0"), false),
            (Some("-1"), false),
            (Some("1"), false),
        ] {
            let mut conn = Node::new("media_conn")
                .attr("ttl", "3600")
                .attr("auth", "synthetic-auth")
                .children(vec![Node::new("host").attr("hostname", "mmg.whatsapp.net")]);
            if let Some(ttl) = auth_ttl {
                conn = conn.attr("auth_ttl", ttl);
            }
            let node = Node::new("iq").attr("type", "result").children(vec![conn]);
            let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, b"media-route:synthetic", 1,
                &serde_json::json!({"status":"result", "at":Timestamp::now().0,"requested_at":Timestamp::now().0-2000,"wire":wire}))
                })
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert_eq!(route(&store).is_ok(), accepted, "auth_ttl={auth_ttl:?}");
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
        }
    }

    #[test]
    fn delayed_route_receipt_cannot_extend_auth_lifetime_or_rebase_legacy() {
        use super::*;
        use crate::{
            binary::{Encoder, Node},
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("route-base.redb"),
            account_id: AccountId::new("route-base-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([8; 32]))),
            create: true,
        })
        .unwrap();
        let node = Node::new("iq").attr("type", "result").children(vec![
            Node::new("media_conn")
                .attr("ttl", "3600")
                .attr("auth_ttl", "1")
                .attr("auth", "synthetic-auth")
                .children(vec![Node::new("host").attr("hostname", "mmg.whatsapp.net")]),
        ]);
        let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&node)
            .unwrap();
        let now = Timestamp::now().0;
        for requested_at in [None, Some(now - 2000), Some(now + 60000), Some(0)] {
            let mut saved = serde_json::json!({"status":"result", "at":now,"wire":wire});
            if let Some(at) = requested_at {
                saved["requested_at"] = at.into();
            }
            store
                .write(|tx| tx.put_json(Table::Checkpoints, b"media-route:synthetic", 1, &saved))
                .unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert!(route(&store).is_err());
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
        }
    }

    #[test]
    fn unchanged_route_reply_cannot_replace_hosts_without_prior_binding() {
        use super::*;
        use crate::{
            binary::{Encoder, Node},
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("route-replacement.redb"),
            account_id: AccountId::new("route-replacement-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap();
        for (flag, accepted) in [
            (None, true),
            (Some("1"), true),
            (Some("+1"), true),
            (Some("١"), true),
            (Some("0"), false),
            (Some("2"), false),
            (Some("-1"), false),
            (Some("bad"), false),
            (Some("2147483648"), false),
        ] {
            let mut conn = Node::new("media_conn")
                .attr("ttl", "3600")
                .attr("auth_ttl", "3600")
                .attr("auth", "synthetic-auth")
                .children(vec![Node::new("host").attr("hostname", "mmg.whatsapp.net")]);
            if let Some(flag) = flag {
                conn = conn.attr("is_new", flag);
            }
            let wire = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&Node::new("iq").attr("type", "result").children(vec![conn]))
                .unwrap();
            store.write(|tx| tx.put_json(Table::Checkpoints,b"media-route:synthetic",1,
                &serde_json::json!({"status":"result","at":Timestamp::now(),"requested_at":Timestamp::now(),"wire":wire}))).unwrap();
            let before = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            assert_eq!(route(&store).is_ok(), accepted);
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                before
            );
        }
    }

    #[test]
    fn store_bound_non_new_projection_is_used_for_upload_route_selection() {
        use super::*;
        use crate::{
            binary::Node,
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("bound-route.redb"),
            account_id: AccountId::new("bound-route-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([1; 32]))),
            create: true,
        })
        .unwrap();
        let reply = |id: &str, host: &str, auth: &str| {
            Node::new("iq")
                .attr("id", id)
                .attr("type", "result")
                .children(vec![
                    Node::new("media_conn")
                        .attr("id", id)
                        .attr("auth", auth)
                        .attr("ttl", "3600")
                        .attr("auth_ttl", "1800")
                        .attr("is_new", "0")
                        .children(vec![Node::new("host").attr("hostname", host)]),
                ])
        };
        let coordinator = &super::super::media_routing::refresh::prepare;
        coordinator(&store, "first").unwrap();
        store
            .write(|tx| {
                super::super::media_routing::refresh::settle_in(
                    tx,
                    "first",
                    &reply("first", "old.whatsapp.net", "old-auth"),
                )
            })
            .unwrap();
        coordinator(&store, "second").unwrap();
        store
            .write(|tx| {
                super::super::media_routing::refresh::settle_in(
                    tx,
                    "second",
                    &reply("second", "ignored.whatsapp.net", "new-auth"),
                )
            })
            .unwrap();
        let (host, auth) = route(&store).unwrap();
        assert_eq!(host, "old.whatsapp.net");
        assert_eq!(auth.as_str(), "new-auth");
        coordinator(&store, "third").unwrap();
        store
            .write(|tx| {
                super::super::media_routing::refresh::settle_in(
                    tx,
                    "third",
                    &Node::new("iq").attr("id", "third").attr("type", "error"),
                )
            })
            .unwrap();
        assert_eq!(route(&store).unwrap().0, "old.whatsapp.net");
    }

    #[test]
    fn sealed_voice_reservation_survives_reopen_without_replay_or_legacy_rewrite() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sealed-voice.redb");
        let open = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("sealed-voice-test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
                create: true,
            })
            .unwrap()
        };
        let mut wire = vec![1, 2, 3];
        wire.extend_from_slice(&finished_voice_seal(&[0; 32]).unwrap());
        let reservation = serde_json::json!({"status":"outcome_unknown",
            "hash":"voice-fixture", "to":"recipient-fixture", "body":wire,
            "encrypted_hash":vec![0;32], "encrypted_body_len":3,
            "upload_body_format":"finished_voice_seal_v1"});
        let key = "message:sealed-voice:upload";
        let store = open();
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &reservation))
            .unwrap();
        let observed = test_response_headers(&store, key, 200).unwrap();
        for field in [
            "body",
            "encrypted_hash",
            "encrypted_body_len",
            "upload_body_format",
        ] {
            assert_eq!(observed[field], reservation[field]);
        }
        drop(store);
        let store = open();
        let restored: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(restored, observed);
        assert!(saved_payload(&restored, "voice-fixture", "recipient-fixture").is_err());
        assert!(test_response_headers(&store, key, 503).is_err());
        // Historical completed rows need no seal/version backfill to reuse the
        // already-created payload. Calling the resolver must not mutate storage.
        let legacy = serde_json::json!({"status":"uploaded", "hash":"voice-fixture",
            "to":"recipient-fixture", "body":[1,2,3], "protobuf":[10,0]});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &legacy))
            .unwrap();
        assert_eq!(
            &*saved_payload(&legacy, "voice-fixture", "recipient-fixture").unwrap(),
            &[10, 0]
        );
        let after: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(after, legacy);
    }

    #[test]
    fn upload_headers_survive_incomplete_body_and_never_authorize_replay() {
        use super::*;
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("upload-headers.redb");
        let open = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("upload-headers-test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
                create: true,
            })
            .unwrap()
        };
        let store = open();
        let key = "message:synthetic:upload";
        assert!(test_response_headers(&store, key, 200).is_err());
        for status in [200, 302, 403, 429, 503] {
            store.write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1,
                &serde_json::json!({"status":"outcome_unknown", "hash":"fixture-hash", "to":"fixture-recipient", "body":[1,2,3]}))).unwrap();
            let saved = test_response_headers(&store, key, status).unwrap();
            assert_eq!(saved["http_status"], status);
            assert_eq!(saved["status"], "response_headers_received");
            assert_eq!(saved["body"], serde_json::json!([1, 2, 3]));
            assert!(saved_payload(&saved, "fixture-hash", "fixture-recipient").is_err());
            assert!(test_response_headers(&store, key, 500).is_err());
        }
        drop(store);
        let store = open();
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["http_status"], 503);
        assert!(saved_payload(&saved, "fixture-hash", "fixture-recipient").is_err());
        for status in ["uploaded", "response_received", "refused"] {
            let settled = serde_json::json!({"status":status,"http_status":403});
            store
                .write(|tx| tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &settled))
                .unwrap();
            assert!(test_response_headers(&store, key, 200).is_err());
            let after: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(after, settled);
        }
    }

    use super::*;
    #[tokio::test]
    async fn legacy_group_upload_binding_is_preserved_and_normal_outbox_uses_its_explicit_chat() {
        let dir = tempfile::tempdir().unwrap();
        let store =
            super::super::group_fanout::tests::open(&dir.path().join("group-upload-binding.redb"));
        let source = dir.path().join("fixture.png");
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&1u32.to_be_bytes());
        png.extend_from_slice(&1u32.to_be_bytes());
        png.resize(33, 0);
        std::fs::write(&source, &png).unwrap();
        let mut message:MessageRequest=serde_json::from_value(serde_json::json!({"group":"120363000000000001@g.us","request_id":"legacy-group-upload","image":source})).unwrap();
        let config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        let hash = input_hash(&message, &png);
        let old = serde_json::json!({"status":"uploaded","hash":hash,"to":"","protobuf":[26,0]});
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"message:legacy-group-upload:upload",
                    1,
                    &old,
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert_eq!(
            &*prepare(&store, &message, &config).await.unwrap().unwrap(),
            &[26, 0]
        );
        assert!(prepare_outbox(&store, &message, &config).await.is_err());
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
        );
        message.request_id = "normal-group-upload".into();
        store.write(|tx|tx.put_json(Table::Checkpoints,b"message:normal-group-upload:upload",1,&serde_json::json!({"status":"uploaded","hash":hash,"to":message.group.clone().unwrap(),"protobuf":[26,0]}))).unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert_eq!(
            &*prepare_outbox(&store, &message, &config)
                .await
                .unwrap()
                .unwrap(),
            &[26, 0]
        );
        message.group = Some("120363000000000002@g.us".into());
        assert!(prepare_outbox(&store, &message, &config).await.is_err());
        assert_eq!(
            before,
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap()
        );
        assert_eq!(store.read(|tx| tx.count(Table::SignalSessions)).unwrap(), 0);
        // No route exists in this fixture: all success/refusal is cached and
        // cannot cause a real upload or a route-refresh query.
    }
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
                    quoted: None,
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
            document: None,
            video: None,
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
    fn mp4_duration_and_size_come_from_mvhd_and_tkhd() {
        fn mp4box(kind: &[u8], body: &[u8]) -> Vec<u8> {
            let mut out = ((body.len() + 8) as u32).to_be_bytes().to_vec();
            out.extend_from_slice(kind);
            out.extend_from_slice(body);
            out
        }
        let mut mvhd = vec![0u8; 100];
        mvhd[12..16].copy_from_slice(&1000u32.to_be_bytes());
        mvhd[16..20].copy_from_slice(&2500u32.to_be_bytes());
        let mut tkhd = vec![0u8; 84];
        tkhd[76..80].copy_from_slice(&(320u32 << 16).to_be_bytes());
        tkhd[80..84].copy_from_slice(&(240u32 << 16).to_be_bytes());
        let audio_tkhd = vec![0u8; 84];
        let moov = [
            mp4box(b"mvhd", &mvhd),
            mp4box(b"trak", &mp4box(b"tkhd", &audio_tkhd)),
            mp4box(b"trak", &mp4box(b"tkhd", &tkhd)),
        ]
        .concat();
        let file = [mp4box(b"ftyp", b"isom\0\0\0\0"), mp4box(b"moov", &moov)].concat();
        assert_eq!(mp4_info(&file).unwrap(), (3, 320, 240));
        assert!(mp4_info(&file[..20]).is_err());
        assert!(mp4_info(b"not an mp4 at all").is_err());
        let payload = Envelope {
            image: None,
            audio: None,
            document: None,
            video: Some(VideoMessage {
                url: "u".into(),
                mimetype: "video/mp4".into(),
                file_sha256: vec![1],
                file_length: 1,
                seconds: 3,
                media_key: vec![2],
                height: 240,
                width: 320,
                file_enc_sha256: vec![3],
                direct_path: "/d".into(),
                media_key_timestamp: 1,
            }),
        }
        .encode_to_vec();
        assert_eq!(
            crate::signal::direct::payload_fields(&payload).unwrap(),
            vec![9]
        );
    }

    #[test]
    fn document_payload_uses_the_apk_field_numbers_and_no_url() {
        let payload = Envelope {
            image: None,
            audio: None,
            video: None,
            document: Some(DocumentMessage {
                mimetype: "application/pdf".into(),
                title: "a.pdf".into(),
                file_sha256: vec![1; 32],
                file_length: 10,
                media_key: vec![2; 32],
                file_name: "a.pdf".into(),
                file_enc_sha256: vec![3; 32],
                direct_path: "/v/t62/x".into(),
                media_key_timestamp: 1,
            }),
        }
        .encode_to_vec();
        assert_eq!(
            crate::signal::direct::payload_fields(&payload).unwrap(),
            vec![7]
        );
        let inner = super::super::protobuf_enum::message_bytes(&payload, 7)
            .unwrap()
            .unwrap();
        let mut fields = crate::signal::direct::payload_fields(&inner).unwrap();
        fields.sort();
        assert_eq!(fields, vec![2, 3, 4, 5, 7, 8, 9, 10, 11]);
        assert_eq!(
            document_meta(Path::new("/tmp/report.PDF")).unwrap(),
            ("report.PDF".into(), "application/pdf".into())
        );
        let request: MessageRequest = serde_json::from_str(
            r#"{"to":"15551234567","request_id":"doc-1","document":"/tmp/report.pdf"}"#,
        )
        .unwrap();
        assert!(input_hash(&request, b"x").starts_with("doc:"));
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
                document: None,
                video: None,
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
        for status in [
            "outcome_unknown",
            "response_headers_received",
            "response_received",
            "refused",
        ] {
            value["status"] = status.into();
            assert!(saved_payload(&value, "hash-a", "test-recipient").is_err());
        }
    }
    #[test]
    fn upload_direct_path_requires_native_query_presence_without_rewriting() {
        let mut reply = serde_json::json!({"url":"https://mmg.whatsapp.net/file"});
        for path in [
            "/file?oe=1&oh=s",
            "/file?oh&oe",
            "/file?oe=&oh=",
            "/file?x=1&oh=s&oe=1#tail",
        ] {
            reply["direct_path"] = path.into();
            assert_eq!(locations(&reply).unwrap().1, path);
        }
        for path in [
            "/file",
            "/file?oe=1",
            "/file?oh=s",
            "/file#?oe=1&oh=s",
            "/file?%6fe=1&oh=s",
            "/file?oe=1&%6fh=s",
            "/file?OE=1&oh=s",
            "/file?x=oe=1&oh=s",
            "/file?oe=1;oh=s",
        ] {
            reply["direct_path"] = path.into();
            assert!(locations(&reply).is_err(), "{path}");
        }
    }

    #[test]
    fn response_locations_reject_untrusted_urls_and_absolute_paths() {
        let good = serde_json::json!({"url":"https://mmg.whatsapp.net/file", "direct_path":"/file?oe=expiry&oh=signature"});
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
            document: None,
            video: None,
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
