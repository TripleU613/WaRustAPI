//! Ordinary incoming image/voice-note retrieval. View-once content never enters
//! this replayable path. APK BWp/BWZ schemas and 1qB image/audio key domains.
use crate::{
    Error, Result,
    registration::request::LabRegistrationConfig,
    store::{ProtocolStore, Table},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

/// Supported ordinary attachment kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    /// Image, not view-once.
    Image,
    /// Ogg/Opus push-to-talk audio, not view-once.
    VoiceNote,
}
/// Redacted local attachment selector. Contains no phone, URL or media key.
#[derive(Debug, Clone)]
pub struct MediaEntry {
    /// Stable SHA-256 selector for the encrypted inbox record.
    pub id: String,
    /// Attachment kind.
    pub kind: MediaKind,
    /// Advertised plaintext length.
    pub bytes: u64,
}
#[derive(Message)]
struct Image {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(string, tag = "2")]
    mime: String,
    #[prost(bytes = "vec", tag = "4")]
    hash: Vec<u8>,
    #[prost(uint64, tag = "5")]
    length: u64,
    #[prost(bytes = "vec", tag = "8")]
    key: Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    enc_hash: Vec<u8>,
    #[prost(bool, tag = "25")]
    once: bool,
}
#[derive(Message)]
struct Audio {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(string, tag = "2")]
    mime: String,
    #[prost(bytes = "vec", tag = "3")]
    hash: Vec<u8>,
    #[prost(uint64, tag = "4")]
    length: u64,
    #[prost(bool, tag = "6")]
    ptt: bool,
    #[prost(bytes = "vec", tag = "7")]
    key: Vec<u8>,
    #[prost(bytes = "vec", tag = "8")]
    enc_hash: Vec<u8>,
    #[prost(bool, tag = "21")]
    once: bool,
}
#[derive(Message)]
struct Envelope {
    #[prost(message, optional, tag = "3")]
    image: Option<Image>,
    #[prost(message, optional, tag = "8")]
    audio: Option<Audio>,
    #[prost(bytes = "vec", optional, tag = "37")]
    once: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "55")]
    once_v2: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "59")]
    once_ext: Option<Vec<u8>>,
}
struct Descriptor {
    kind: MediaKind,
    url: String,
    mime: String,
    hash: Vec<u8>,
    key: Vec<u8>,
    enc_hash: Vec<u8>,
    length: u64,
}
impl Drop for Descriptor {
    fn drop(&mut self) {
        self.key.zeroize();
        self.url.zeroize();
    }
}
fn bad(reason: &'static str) -> Error {
    Error::invalid("media", reason)
}
#[derive(Message)]
struct RawEnvelope {
    #[prost(bytes = "vec", optional, tag = "3")]
    image: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "8")]
    audio: Option<Vec<u8>>,
}
#[derive(Message)]
struct ContextOnly {
    #[prost(message, optional, tag = "17")]
    context: Option<MediaContext>,
}
// Comparison BWp/BWZ contextInfo=17, BWx stanzaId=1, quotedMessage=3,
// expiration=25. Keep quote detection separate from expiry enforcement.
#[derive(Message)]
struct MediaContext {
    #[prost(string, optional, tag = "1")]
    stanza: Option<String>,
    #[prost(bytes = "vec", optional, tag = "3")]
    quoted: Option<Vec<u8>>,
    #[prost(uint32, optional, tag = "25")]
    expiration: Option<u32>,
}
fn media_context(payload: &[u8]) -> Result<Option<MediaContext>> {
    let raw = RawEnvelope::decode(payload).map_err(|_| bad("invalid media envelope"))?;
    let Some(nested) = raw.image.or(raw.audio) else {
        return Ok(None);
    };
    Ok(ContextOnly::decode(nested.as_slice())
        .map_err(|_| bad("invalid media context"))?
        .context)
}

fn unexpired(store: &ProtocolStore, record: &serde_json::Value, duration: u32) -> Result<bool> {
    let wire: Option<Vec<u8>> = if !record["route"].is_null() {
        Some(
            serde_json::from_value(record["route"].clone())
                .map_err(|_| bad("invalid saved route"))?,
        )
    } else if let Ok(capture) = serde_json::from_value::<Vec<u8>>(record["capture"].clone()) {
        store.read(|tx| tx.get_json(Table::Checkpoints, &capture, 1))?
    } else {
        None
    };
    let Some(wire) = wire else {
        return Ok(false);
    };
    let limits = crate::profile::Limits::default();
    let raw = crate::binary::unpack_frame_payload(&wire, &limits)?;
    let node = crate::binary::Decoder::new(
        &raw,
        crate::binary::TokenDictionary::comparison_2_26_37_70(),
        &limits,
    )
    .decode_node()?;
    let sent = node
        .get_attr("t")
        .and_then(|v| v.as_text())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .and_then(|v| v.checked_mul(1000));
    Ok(sent.is_some_and(|sent| {
        sent.saturating_add(i64::from(duration) * 1000) > crate::model::time::Timestamp::now().0
    }))
}
fn descriptor(bytes: &[u8]) -> Result<Option<Descriptor>> {
    if bytes.len() > 65536 {
        return Err(bad("message exceeds descriptor bound"));
    }
    let e = Envelope::decode(bytes).map_err(|_| bad("invalid media protobuf"))?;
    if e.once.is_some() || e.once_v2.is_some() || e.once_ext.is_some() {
        return Ok(None);
    }
    if e.image.is_some() && e.audio.is_some() {
        return Err(bad("ambiguous media message"));
    }
    let d = if let Some(i) = e.image {
        if i.once {
            return Ok(None);
        }
        Descriptor {
            kind: MediaKind::Image,
            url: i.url,
            mime: i.mime,
            hash: i.hash,
            key: i.key,
            enc_hash: i.enc_hash,
            length: i.length,
        }
    } else if let Some(a) = e.audio {
        if a.once || !a.ptt {
            return Ok(None);
        }
        Descriptor {
            kind: MediaKind::VoiceNote,
            url: a.url,
            mime: a.mime,
            hash: a.hash,
            key: a.key,
            enc_hash: a.enc_hash,
            length: a.length,
        }
    } else {
        return Ok(None);
    };
    if d.length == 0
        || d.length > 8 * 1024 * 1024
        || d.key.len() != 32
        || d.hash.len() != 32
        || d.enc_hash.len() != 32
    {
        return Err(bad("invalid bounded media descriptor"));
    }
    Ok(Some(d))
}
fn candidates(store: &ProtocolStore) -> Result<Vec<(Vec<u8>, Descriptor)>> {
    let rows = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
    if rows.len() == 10000 {
        return Err(bad("inbox requires pagination"));
    }
    let mut out = Vec::new();
    for (key, bytes) in rows {
        if !key.starts_with(b"received:") {
            continue;
        }
        let mut record: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| bad("invalid inbox record"))?;
        if record["status"] != "decrypted" {
            continue;
        }
        let plain: Zeroizing<Vec<u8>> = serde_json::from_value(record["protobuf"].take())
            .map_err(|_| bad("invalid protobuf record"))?;
        if let Some(d) = descriptor(&plain)? {
            if let Some(duration) = media_context(&plain)?
                .and_then(|c| c.expiration)
                .filter(|v| *v != 0)
                && !unexpired(store, &record, duration)?
            {
                continue;
            }
            out.push((key, d));
        }
    }
    Ok(out)
}
/// List ordinary attachments without opening, exporting, or contacting a server.
pub fn list(store: &ProtocolStore) -> Result<Vec<MediaEntry>> {
    Ok(candidates(store)?
        .into_iter()
        .map(|(key, d)| MediaEntry {
            id: hex::encode(Sha256::digest(key)),
            kind: d.kind,
            bytes: d.length,
        })
        .collect())
}

const REFERENCE_BINDING: &[u8] = b"incoming-ordinary-media-v1";

/// Project unwrapped ordinary media while retaining expiry and refusing quotes. The
/// caller must atomically persist the authenticated inbox record with the event.
pub(super) fn event_content(
    tx: &crate::store::WriteTx<'_>,
    key: &[u8],
    payload: &[u8],
) -> Result<Option<(crate::model::message::MessageContent, Option<u32>)>> {
    let fields = crate::signal::direct::payload_fields(payload)?;
    if fields.iter().any(|v| !matches!(v, 3 | 8 | 35))
        || fields.iter().filter(|v| matches!(v, 3 | 8)).count() != 1
    {
        return Ok(None);
    }
    #[derive(Message)]
    struct CaptionOnly {
        #[prost(string, optional, tag = "3")]
        caption: Option<String>,
    }
    let raw = RawEnvelope::decode(payload).map_err(|_| bad("invalid media envelope"))?;
    let nested = raw
        .image
        .as_ref()
        .or(raw.audio.as_ref())
        .ok_or_else(|| bad("missing media"))?;
    let context = media_context(payload)?;
    if context
        .as_ref()
        .is_some_and(|c| c.stanza.is_some() || c.quoted.is_some())
    {
        return Ok(None);
    }
    let expiration = context.and_then(|c| c.expiration).filter(|v| *v != 0);
    let Some(d) = descriptor(payload)? else {
        return Ok(None);
    };
    media_url(&d)?;
    let caption = if raw.image.is_some() {
        CaptionOnly::decode(nested.as_slice())
            .map_err(|_| bad("invalid image caption"))?
            .caption
    } else {
        None
    };
    let id = hex::encode(Sha256::digest(key));
    Ok(Some((
        crate::model::message::MessageContent::Media {
            reference: crate::model::message::MediaReference {
                sealed: tx
                    .seal_media_reference(REFERENCE_BINDING, id.as_bytes())?
                    .into(),
                kind: match d.kind {
                    MediaKind::Image => crate::model::message::MediaKind::Image,
                    MediaKind::VoiceNote => crate::model::message::MediaKind::Audio,
                },
                mime_type: d.mime.clone(),
                length: Some(d.length),
            },
            caption,
            file_name: None,
        },
        expiration,
    )))
}

/// Account-bound references to authenticated ordinary inbox attachments. This
/// operation is offline and excludes every supported view-once marker.
pub fn references(store: &ProtocolStore) -> Result<Vec<crate::model::message::MediaReference>> {
    candidates(store)?
        .into_iter()
        .map(|(key, d)| {
            media_url(&d)?;
            let id = hex::encode(Sha256::digest(key));
            Ok(crate::model::message::MediaReference {
                sealed: store
                    .seal_media_reference(REFERENCE_BINDING, id.as_bytes())?
                    .into(),
                kind: match d.kind {
                    MediaKind::Image => crate::model::message::MediaKind::Image,
                    MediaKind::VoiceNote => crate::model::message::MediaKind::Audio,
                },
                mime_type: d.mime.clone(),
                length: Some(d.length),
            })
        })
        .collect()
}

/// Validate a sealed reference against its still-present ordinary inbox record.
/// Outgoing references and references from another account cannot be opened.
pub fn reference_selector(
    store: &ProtocolStore,
    reference: &crate::model::message::MediaReference,
) -> Result<String> {
    let plain = store.open_media_reference(REFERENCE_BINDING, &reference.sealed)?;
    let id = std::str::from_utf8(&plain).map_err(|_| bad("invalid reference"))?;
    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad("invalid reference selector"));
    }
    let (_, d) = candidates(store)?
        .into_iter()
        .find(|(key, _)| hex::encode(Sha256::digest(key)) == id)
        .ok_or_else(|| bad("ordinary attachment not found"))?;
    let kind = match d.kind {
        MediaKind::Image => crate::model::message::MediaKind::Image,
        MediaKind::VoiceNote => crate::model::message::MediaKind::Audio,
    };
    if reference.kind != kind || reference.mime_type != d.mime || reference.length != Some(d.length)
    {
        return Err(bad("reference metadata mismatch"));
    }
    media_url(&d)?;
    Ok(id.to_owned())
}

/// Export explicitly requested ordinary plaintext to a new private file. Never
/// overwrites existing files/symlinks. An I/O failure truncates this owned file
/// descriptor; the empty private file may remain for operator inspection.
#[cfg(any(test, feature = "comparison-client"))]
pub(crate) fn export_new(path: &std::path::Path, plain: &[u8]) -> Result<u64> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    if let Err(error) = file.write_all(plain).and_then(|_| file.sync_all()) {
        let _ = file.set_len(0);
        return Err(error.into());
    }
    Ok(plain.len() as u64)
}
fn media_url(d: &Descriptor) -> Result<reqwest::Url> {
    let mime = d.mime.split(';').next().unwrap_or("").trim();
    let allowed = match d.kind {
        MediaKind::Image => matches!(mime, "image/jpeg" | "image/png" | "image/webp"),
        MediaKind::VoiceNote => mime == "audio/ogg",
    };
    if !allowed || d.url.len() > 8192 {
        return Err(bad("unsupported media format"));
    }
    let url = reqwest::Url::parse(&d.url).map_err(|_| bad("invalid media URL"))?;
    if url.scheme() != "https"
        || !url
            .host_str()
            .is_some_and(super::image_upload::approved_host)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err(bad("unapproved media URL"));
    }
    Ok(url)
}
fn decrypt(d: &Descriptor, body: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    let f = match d.kind {
        MediaKind::Image => crate::signal::image::decrypt,
        MediaKind::VoiceNote => crate::signal::image::decrypt_audio,
    };
    let plain = f(
        body,
        d.key.as_slice().try_into().map_err(|_| bad("key width"))?,
        d.enc_hash
            .as_slice()
            .try_into()
            .map_err(|_| bad("hash width"))?,
        d.hash
            .as_slice()
            .try_into()
            .map_err(|_| bad("hash width"))?,
        d.length,
    )?;
    if d.kind == MediaKind::VoiceNote {
        super::voice_note::seconds(&plain)?;
    }
    Ok(plain)
}
/// Explicit retrieval for one opaque selector. Existing verified ciphertext is
/// reused; failed/uncertain GETs never retry automatically. Plaintext is returned
/// in zeroizing memory, not exported or persisted.
pub async fn open(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
    id: &str,
) -> Result<(MediaKind, Zeroizing<Vec<u8>>)> {
    config.validate()?;
    let account = store
        .read(crate::session::AccountRecord::load)?
        .ok_or(Error::NotRegistered)?;
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::NotRegistered);
    }
    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad("invalid attachment selector"));
    }
    let (_, d) = candidates(store)?
        .into_iter()
        .find(|(key, _)| hex::encode(Sha256::digest(key)) == id)
        .ok_or_else(|| bad("ordinary attachment not found"))?;
    let state_key = format!("media-download:{id}");
    let saved: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1))?;
    if let Some(mut saved) = saved {
        if saved["status"] != "downloaded" {
            return Err(bad("previous download uncertain/refused; no retry"));
        }
        let body: Zeroizing<Vec<u8>> = serde_json::from_value(saved["body"].take())
            .map_err(|_| bad("invalid saved ciphertext"))?;
        let plain = decrypt(&d, &body)?;
        ensure_available(store, id)?;
        return Ok((d.kind, plain));
    }
    let url = media_url(&d)?;
    let client = super::image_upload::client(config)?;
    store.write(|tx| {
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, state_key.as_bytes(), 1)?
            .is_some()
        {
            return Err(bad("download already reserved"));
        }
        tx.put_json(
            Table::Checkpoints,
            state_key.as_bytes(),
            1,
            &serde_json::json!({"status":"outcome_unknown"}),
        )
    })?;
    let mut response = client
        .get(url)
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .map_err(|_| bad("media GET failed; reservation retained"))?;
    if response.status().as_u16() != 200 {
        return Err(bad("media GET refused; reservation retained"));
    }
    let expected = (d.length / 16 + 1) * 16 + 10;
    if response.content_length().is_some_and(|n| n != expected) {
        return Err(bad("media length mismatch"));
    }
    let mut body = Zeroizing::new(Vec::new());
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| bad("media GET interrupted"))?
    {
        if (body.len() + chunk.len()) as u64 > expected {
            return Err(bad("media exceeds size limit"));
        }
        body.extend_from_slice(&chunk);
    }
    let plain = decrypt(&d, &body)?;
    store.write(|tx| {
        tx.put_json(
            Table::Checkpoints,
            state_key.as_bytes(),
            1,
            &serde_json::json!({"status":"downloaded","body":&*body}),
        )
    })?;
    ensure_available(store, id)?;
    Ok((d.kind, plain))
}

pub(crate) fn ensure_available(store: &ProtocolStore, id: &str) -> Result<()> {
    if !candidates(store)?
        .iter()
        .any(|(key, _)| hex::encode(Sha256::digest(key)) == id)
    {
        return Err(bad("attachment expired or no longer available"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn ordinary_media_events_are_atomic_downloadable_and_exclude_context_and_view_once() {
        use super::*;
        use crate::{
            binary::Node,
            model::{
                ids::{AccountId, EventCursor},
                message::MessageContent,
            },
            store::{ConsumerId, MasterKey, OpenOptions, StaticKeySource},
            sync::SyncEvent,
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("media-events.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let image = Image {
            url: "https://mmg.whatsapp.net/fixture".into(),
            mime: "image/png".into(),
            hash: vec![1; 32],
            length: 42,
            key: vec![2; 32],
            enc_hash: vec![3; 32],
            once: false,
        };
        #[derive(Message)]
        struct RawImage {
            #[prost(bytes = "vec", tag = "3")]
            image: Vec<u8>,
        }
        let mut image_bytes = image.encode_to_vec();
        // Literal image caption tag 3, UTF-8 length 4.
        image_bytes.extend_from_slice(b"\x1a\x04test");
        let image_payload = RawImage {
            image: image_bytes.clone(),
        }
        .encode_to_vec();
        let audio_payload = Envelope {
            audio: Some(Audio {
                url: "https://mmg.whatsapp.net/fixture".into(),
                mime: "audio/ogg; codecs=opus".into(),
                hash: vec![1; 32],
                length: 42,
                ptt: true,
                key: vec![2; 32],
                enc_hash: vec![3; 32],
                once: false,
            }),
            ..Default::default()
        }
        .encode_to_vec();
        let node = Node::new("message")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "media-fixture");
        let publish = |tx: &mut crate::store::WriteTx<'_>, key: &[u8], payload: &[u8]| {
            tx.put_json(
                Table::Checkpoints,
                key,
                1,
                &serde_json::json!({"status":"decrypted", "protobuf":payload}),
            )?;
            super::super::inbox_events::publish(tx, key, &node, payload)
        };
        let rollback: Result<()> = store.write(|tx| {
            publish(tx, b"received:image", &image_payload)?;
            Err(Error::Cancelled)
        });
        assert!(rollback.is_err());
        assert_eq!(store.journal_head().unwrap(), EventCursor::START);
        assert!(references(&store).unwrap().is_empty());
        assert!(
            store
                .write(|tx| publish(tx, b"received:image", &image_payload))
                .unwrap()
        );
        assert!(
            store
                .write(|tx| publish(tx, b"received:audio", &audio_payload))
                .unwrap()
        );
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert!(
            !store
                .write(|tx| publish(tx, b"received:image", &image_payload))
                .unwrap()
        );
        let events = store
            .read_events(&ConsumerId::new("test").unwrap(), EventCursor::START, 10)
            .unwrap();
        assert_eq!(events.events.len(), 2);
        for (index, event) in events.events.iter().enumerate() {
            let SyncEvent::NewMessage(message) = &event.event else {
                panic!("media event missing")
            };
            let MessageContent::Media {
                reference, caption, ..
            } = &message.content
            else {
                panic!("reference missing")
            };
            assert_eq!(
                caption.as_deref(),
                if index == 0 { Some("test") } else { None }
            );
            assert_eq!(
                reference.kind,
                if index == 0 {
                    crate::model::message::MediaKind::Image
                } else {
                    crate::model::message::MediaKind::Audio
                }
            );
            assert!(reference_selector(&store, reference).is_ok());
            assert!(!message.ephemerality.view_once);
        }
        for suffix in [b"\xc8\x01\x01".as_slice(), b"\x8a\x01\x02\x1a\x00"] {
            let mut nested = image_bytes.clone();
            nested.extend_from_slice(suffix);
            let payload = RawImage { image: nested }.encode_to_vec();
            assert!(
                store
                    .write(|tx| event_content(tx, b"received:excluded", &payload))
                    .unwrap()
                    .is_none()
            );
        }
        // Literal BWx expiration=60 within contextInfo=17. Projecting context
        // must retain the timer, and retrieval independently enforces it.
        let mut nested = image_bytes.clone();
        nested.extend_from_slice(b"\x8a\x01\x03\xc8\x01\x3c");
        let expiring = RawImage { image: nested }.encode_to_vec();
        let now = (crate::model::time::Timestamp::now().0 / 1000).to_string();
        for (suffix, timestamp, expected) in [
            ("fresh", Some(now.as_str()), true),
            ("expired", Some("1"), false),
            ("unknown", None, false),
        ] {
            let key = format!("received:{suffix}");
            let mut route_node = node.clone();
            if let Some(timestamp) = timestamp {
                route_node = route_node.attr("t", timestamp);
            }
            let route = crate::binary::Encoder::new(
                crate::binary::TokenDictionary::comparison_2_26_37_70(),
            )
            .encode(&route_node)
            .unwrap();
            store.write(|tx| {
                tx.put_json(Table::Checkpoints, key.as_bytes(), 1, &serde_json::json!({"status":"decrypted", "protobuf":expiring, "route":route}))?;
                super::super::inbox_events::publish(tx, key.as_bytes(), &route_node, &expiring)
            }).unwrap();
            let id = hex::encode(Sha256::digest(key.as_bytes()));
            assert_eq!(list(&store).unwrap().iter().any(|v| v.id == id), expected);
        }
        let events = store
            .read_events(&ConsumerId::new("expiry").unwrap(), EventCursor::START, 20)
            .unwrap();
        let SyncEvent::NewMessage(fresh) = &events.events[2].event else {
            panic!("fresh media missing")
        };
        assert_eq!(fresh.ephemerality.expiration_seconds, Some(60));
        assert!(matches!(
            events.events[3].event,
            SyncEvent::UnsupportedMessage(_)
        ));
        assert!(matches!(
            events.events[4].event,
            SyncEvent::UnsupportedMessage(_)
        ));
        let MessageContent::Media { reference, .. } = &fresh.content else {
            panic!("media missing")
        };
        assert!(reference_selector(&store, reference).is_ok());
        // Replace only the synthetic saved timestamp with an expired one to
        // prove existing references recheck the current authenticated record.
        let old_route =
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_70())
                .encode(&node.clone().attr("t", "1"))
                .unwrap();
        store.write(|tx| tx.put_json(Table::Checkpoints, b"received:fresh", 1, &serde_json::json!({"status":"decrypted", "protobuf":expiring, "route":old_route}))).unwrap();
        assert!(reference_selector(&store, reference).is_err());
    }
    use super::*;
    #[cfg(feature = "comparison-client")]
    #[tokio::test]
    async fn public_client_downloads_cached_media_without_connecting() {
        use crate::{
            WhatsAppClient,
            model::ids::AccountId,
            session::{AccountRecord, RegistrationStatus},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("account.redb");
        let keys = std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32])));
        let store = ProtocolStore::open(OpenOptions {
            path: path.clone(),
            account_id: AccountId::new("test").unwrap(),
            key_source: keys.clone(),
            create: true,
        })
        .unwrap();
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = Zeroizing::new("offline-fixture".into());
        let encrypted = crate::signal::image::encrypt(b"synthetic image fixture").unwrap();
        let protobuf = Envelope {
            image: Some(Image {
                url: "https://mmg.whatsapp.net/offline-fixture".into(),
                mime: "image/png".into(),
                hash: encrypted.plaintext_hash.to_vec(),
                length: encrypted.plaintext_len,
                key: encrypted.media_key.to_vec(),
                enc_hash: encrypted.encrypted_hash.to_vec(),
                once: false,
            }),
            audio: None,
            once: None,
            once_v2: None,
            once_ext: None,
        }
        .encode_to_vec();
        let id = hex::encode(Sha256::digest(b"received:cached-fixture"));
        store
            .write(|tx| {
                AccountRecord {
                    phone: Some(config.phone.clone()),
                    registration: RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)?;
                tx.put_json(
                    Table::Checkpoints,
                    b"received:cached-fixture",
                    1,
                    &serde_json::json!({"status":"decrypted","protobuf":protobuf}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("media-download:{id}").as_bytes(),
                    1,
                    &serde_json::json!({"status":"downloaded","body":encrypted.body}),
                )
            })
            .unwrap();
        drop(store);
        let client = WhatsAppClient::builder()
            .store_path(path)
            .account_id(AccountId::new("test").unwrap())
            .key_source(keys)
            .create_store(false)
            .comparison_transport(config)
            .build()
            .await
            .unwrap();
        let reference = client.received_media().await.unwrap().pop().unwrap();
        let sink = dir.path().join("download.bin");
        assert_eq!(
            client
                .download_media(&reference, sink.clone())
                .await
                .unwrap(),
            encrypted.plaintext_len
        );
        assert_eq!(std::fs::read(&sink).unwrap(), b"synthetic image fixture");
        assert!(client.download_media(&reference, sink).await.is_err());
        assert!(matches!(
            client.connection_state().borrow().state,
            crate::runtime::ConnectionState::Stopped
        ));
        client.shutdown().await;
    }

    #[test]
    fn references_are_account_bound_and_view_once_cannot_be_exported() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let open = |name: &str| {
            ProtocolStore::open(OpenOptions {
                path: dir.path().join(name),
                account_id: AccountId::new("test").unwrap(),
                create: true,
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [7; 32],
                ))),
            })
            .unwrap()
        };
        let store = open("one.redb");
        let payload = |once| {
            Envelope {
                image: Some(Image {
                    url: "https://mmg.whatsapp.net/test".into(),
                    mime: "image/png".into(),
                    hash: vec![1; 32],
                    length: 4,
                    key: vec![2; 32],
                    enc_hash: vec![3; 32],
                    once,
                }),
                audio: None,
                once: None,
                once_v2: None,
                once_ext: None,
            }
            .encode_to_vec()
        };
        let save = |once| {
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        b"received:fixture",
                        1,
                        &serde_json::json!({"status":"decrypted","protobuf":payload(once)}),
                    )
                })
                .unwrap()
        };
        save(false);
        let reference = references(&store).unwrap().pop().unwrap();
        let id = reference_selector(&store, &reference).unwrap();
        assert_eq!(id, hex::encode(Sha256::digest(b"received:fixture")));
        assert!(reference_selector(&open("other.redb"), &reference).is_err());
        let mut changed = reference.clone();
        changed.length = Some(5);
        assert!(reference_selector(&store, &changed).is_err());
        changed = reference.clone();
        changed.kind = crate::model::message::MediaKind::Audio;
        assert!(reference_selector(&store, &changed).is_err());
        changed = reference.clone();
        changed.mime_type = "image/jpeg".into();
        assert!(reference_selector(&store, &changed).is_err());
        save(true);
        assert!(references(&store).unwrap().is_empty());
        assert!(reference_selector(&store, &reference).is_err());
    }

    #[test]
    fn explicit_export_is_private_and_never_clobbers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ordinary.bin");
        assert_eq!(export_new(&path, b"test").unwrap(), 4);
        assert!(export_new(&path, b"replacement").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"test");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            let link = dir.path().join("symlink");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(export_new(&link, b"replacement").is_err());
            assert_eq!(std::fs::read(&path).unwrap(), b"test");
        }
    }

    #[test]
    fn ordinary_path_rejects_all_view_once_wrappers_and_flags() {
        for bytes in [
            &[0x1a, 3, 0xc8, 1, 1][..],
            &[0x42, 3, 0xa8, 1, 1],
            &[0xba, 3, 0],
            &[0xda, 3, 0],
            &[0xaa, 2, 0],
        ] {
            assert!(descriptor(bytes).unwrap().is_none());
        }
    }
}
