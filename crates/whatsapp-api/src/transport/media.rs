//! Ordinary incoming and synchronized own-device media retrieval. View-once content never enters
//! this replayable path. APK 37.71 BZD/BZ0/BZC schemas and 1qB image/audio key domains.
use crate::{
    Error, Result,
    registration::request::LabRegistrationConfig,
    store::{ProtocolStore, Table},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

/// Supported ordinary attachment kinds. JSON names are snake_case
/// (`voice_note`, `video_note`); CLI text output prints the variant name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    /// Image, not view-once.
    Image,
    /// Ogg/Opus push-to-talk audio, not view-once.
    VoiceNote,
    /// Ordinary WebP sticker (including animated WebP); Lottie is unsupported.
    Sticker,
    /// Ordinary MP4 video.
    Video,
    /// MP4 push-to-video note; preserves its distinct inbox selector kind.
    VideoNote,
    /// Ordinary document; CLI uses explicit plaintext export.
    Document,
}
/// Redacted local attachment selector. Contains no phone, URL or media key.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MediaEntry {
    /// Stable SHA-256 selector for the encrypted inbox record: the digest of
    /// its key `received:{chat}:{message id}` (or `received-own:…`), so it is
    /// not reversible to the chat without the message ID.
    pub id: String,
    /// Attachment kind.
    pub kind: MediaKind,
    /// Advertised plaintext length.
    pub bytes: u64,
}
// Pinned43.72 DiL.dynamicMethod and EwA.A03 confirm the core field mapping.
// See docs/protocol/apk/2.26.43.72-media-image-ingress; full HTTP parity remains open.
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
// APK 2.26.37.71 BZC.dynamicMethod 00de..00ec; names assembled by
// 6al.A1W and Ayq.A11. Sticker hashes/key use 2/3/4, unlike ImageMessage.
#[derive(Message)]
struct Sticker {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(bytes = "vec", tag = "2")]
    hash: Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    enc_hash: Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    key: Vec<u8>,
    #[prost(string, tag = "5")]
    mime: String,
    #[prost(uint64, tag = "9")]
    length: u64,
    #[prost(bool, tag = "21")]
    lottie: bool,
}
// APK 2.26.37.71 BZE.dynamicMethod 01a4..01b2 plus Ayq.A12:
// videoMessage and ptvMessage share this schema; viewOnce is wire field 20.
// Pinned43.72 DiM/F1B confirms the core video tags; current PTV dispatch is separate.
// See docs/protocol/apk/2.26.43.72-media-video-ingress.
#[derive(Message)]
struct Video {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(string, tag = "2")]
    mime: String,
    #[prost(bytes = "vec", tag = "3")]
    hash: Vec<u8>,
    #[prost(uint64, tag = "4")]
    length: u64,
    #[prost(bytes = "vec", tag = "6")]
    key: Vec<u8>,
    #[prost(bytes = "vec", tag = "11")]
    enc_hash: Vec<u8>,
    #[prost(bool, tag = "20")]
    once: bool,
}
// APK 37.71 BZB.dynamicMethod 011e..012c, with Ayp.A1K name helper:
// DocumentMessage tags differ from image/video; caption=20, fileName=8.
// Pinned43.72 Di0/Eel confirms the core tags; native parser permits zero size.
// Nonempty admission/crypto remain local limits; see
// docs/protocol/apk/2.26.43.72-media-document-ingress.
#[derive(Message)]
struct Document {
    #[prost(string, tag = "1")]
    url: String,
    #[prost(string, tag = "2")]
    mime: String,
    #[prost(bytes = "vec", tag = "4")]
    hash: Vec<u8>,
    #[prost(uint64, tag = "5")]
    length: u64,
    #[prost(bytes = "vec", tag = "7")]
    key: Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    enc_hash: Vec<u8>,
}
#[derive(Message)]
struct Envelope {
    // BZc Message descriptor: documentMessage=7, type BZB.
    #[prost(message, optional, tag = "7")]
    document: Option<Document>,
    #[prost(message, optional, tag = "3")]
    image: Option<Image>,
    #[prost(message, optional, tag = "8")]
    audio: Option<Audio>,
    // APK 2.26.37.71 BZc Message descriptor: stickerMessage=26.
    #[prost(message, optional, tag = "26")]
    sticker: Option<Sticker>,
    // APK 2.26.37.71 BZc: videoMessage=9, ptvMessage=66; both BZE.
    #[prost(message, optional, tag = "9")]
    video: Option<Video>,
    #[prost(message, optional, tag = "66")]
    ptv: Option<Video>,
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
    #[prost(bytes = "vec", optional, tag = "7")]
    document: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "3")]
    image: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "8")]
    audio: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "26")]
    sticker: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "9")]
    video: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "66")]
    ptv: Option<Vec<u8>>,
}
#[derive(Message)]
struct ContextOnly {
    #[prost(message, optional, tag = "17")]
    context: Option<MediaContext>,
}
// contextInfo=17 (2.26.37.71 BZD/BZ0 contextInfo_, type BZL; BWp/BWZ/BWx are
// another release's names), stanzaId=1, participant=2, quotedMessage=3,
// expiration=25 (the same fields text replies use).
// Keep quote detection separate from expiry enforcement.
#[derive(Message)]
struct MediaContext {
    #[prost(string, optional, tag = "1")]
    stanza: Option<String>,
    #[prost(string, optional, tag = "2")]
    participant: Option<String>,
    #[prost(bytes = "vec", optional, tag = "3")]
    quoted: Option<Vec<u8>>,
    #[prost(uint32, optional, tag = "25")]
    expiration: Option<u32>,
}
fn media_context(payload: &[u8]) -> Result<Option<MediaContext>> {
    let raw = RawEnvelope::decode(payload).map_err(|_| bad("invalid media envelope"))?;
    let Some(nested) = raw
        .image
        .or(raw.audio)
        .or(raw.sticker)
        .or(raw.video)
        .or(raw.ptv)
        .or(raw.document)
    else {
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
        crate::binary::TokenDictionary::comparison_2_26_37_71(),
        &limits,
    )
    .decode_node()?;
    // Same explicit APK CgC.A01 timestamp as event projection. Unknown historical
    // time must not gain a current-clock fallback here; see message-timestamps.md.
    let sent = super::inbox_events::stanza_timestamp(&node).map(|time| time.0);
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
    if [
        e.image.is_some(),
        e.audio.is_some(),
        e.sticker.is_some(),
        e.video.is_some(),
        e.ptv.is_some(),
        e.document.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
        > 1
    {
        return Err(bad("ambiguous media message"));
    }
    let video_kind = if e.ptv.is_some() {
        MediaKind::VideoNote
    } else {
        MediaKind::Video
    };
    let d = if let Some(i) = e.image {
        if i.once || !supported_mime(MediaKind::Image, &i.mime) {
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
        // Pinned43.72 Etl.A00 -> 0jZ.A08 admits the exact Opus MIME.
        // This receiver only supports Opus PTT; stripping codec parameters
        // would authorize a GET/cache handoff for metadata native parsing rejects.
        // See docs/protocol/apk/2.26.43.72-media-audio-ingress.
        if a.once || !a.ptt || !supported_mime(MediaKind::VoiceNote, &a.mime) {
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
    } else if let Some(v) = e.video.or(e.ptv) {
        if v.once || !supported_mime(video_kind, &v.mime) {
            return Ok(None);
        }
        Descriptor {
            kind: video_kind,
            url: v.url,
            mime: v.mime,
            hash: v.hash,
            key: v.key,
            enc_hash: v.enc_hash,
            length: v.length,
        }
    } else if let Some(doc) = e.document {
        Descriptor {
            kind: MediaKind::Document,
            url: doc.url,
            mime: doc.mime,
            hash: doc.hash,
            key: doc.key,
            enc_hash: doc.enc_hash,
            length: doc.length,
        }
    } else if let Some(s) = e.sticker {
        // Pinned43.72 FPx.A00 compares the whole non-Lottie WebP MIME
        // case-insensitively. Parameters/whitespace cannot authorize retrieval.
        // ASCII MIME scope; full Java Unicode case folding is not claimed.
        // See docs/protocol/apk/2.26.43.72-media-sticker-ingress.
        if s.lottie || !supported_mime(MediaKind::Sticker, &s.mime) {
            return Ok(None);
        }
        Descriptor {
            kind: MediaKind::Sticker,
            url: s.url,
            mime: s.mime,
            hash: s.hash,
            key: s.key,
            enc_hash: s.enc_hash,
            length: s.length,
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
type Candidate = (Vec<u8>, Descriptor, Zeroizing<Vec<u8>>);

fn candidates(store: &ProtocolStore) -> Result<Vec<Candidate>> {
    let rows = store.read(|tx| tx.scan(Table::Checkpoints, 1, 10000))?;
    if rows.len() == 10000 {
        return Err(bad("inbox requires pagination"));
    }
    let mut out = Vec::new();
    for (key, bytes) in rows {
        if !key.starts_with(b"received:") && !key.starts_with(b"received-own:") {
            continue;
        }
        let mut record: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| bad("invalid inbox record"))?;
        if record["status"] != "decrypted" {
            continue;
        }
        let plain: Zeroizing<Vec<u8>> = serde_json::from_value(record["protobuf"].take())
            .map_err(|_| bad("invalid protobuf record"))?;
        let plain = if let Some(suffix) = key.strip_prefix(b"received-own:") {
            let chat = std::str::from_utf8(suffix)
                .ok()
                .and_then(|s| s.split_once(':'))
                .and_then(|(chat, _)| crate::model::jid::Jid::parse(chat).ok())
                .ok_or_else(|| bad("invalid synchronized media chat"))?;
            super::own_device::payload(&plain, &chat)
                .ok_or_else(|| bad("invalid synchronized media wrapper"))?
        } else {
            plain
        };
        if let Some(d) = descriptor(&plain)? {
            if let Some(duration) = media_context(&plain)?
                .and_then(|c| c.expiration)
                .filter(|v| *v != 0)
                && !unexpired(store, &record, duration)?
            {
                continue;
            }
            out.push((key, d, Zeroizing::new(bytes)));
        }
    }
    Ok(out)
}
/// List ordinary attachments without opening, exporting, or contacting a server.
pub fn list(store: &ProtocolStore) -> Result<Vec<MediaEntry>> {
    Ok(candidates(store)?
        .into_iter()
        .map(|(key, d, _)| MediaEntry {
            id: hex::encode(Sha256::digest(key)),
            kind: d.kind,
            bytes: d.length,
        })
        .collect())
}

const REFERENCE_BINDING: &[u8] = b"incoming-ordinary-media-v1";

pub(super) struct MediaCaption {
    pub(super) kind: i32,
    pub(super) caption: Option<String>,
}

// 152/16P clone original media; document A0w/A0x is caption, not filename.
pub(super) fn caption_fields(payload: &[u8]) -> Result<Option<MediaCaption>> {
    let fields = crate::signal::direct::payload_fields(payload)?;
    if fields.iter().any(|f| !matches!(f, 3 | 7 | 9 | 35))
        || fields.iter().filter(|&&f| matches!(f, 3 | 7 | 9)).count() != 1
    {
        return Ok(None);
    }
    #[derive(Message)]
    struct ImageCaption {
        #[prost(string, optional, tag = "3")]
        caption: Option<String>,
        #[prost(bool, tag = "25")]
        once: bool,
    }
    #[derive(Message)]
    struct VideoCaption {
        #[prost(string, optional, tag = "7")]
        caption: Option<String>,
        #[prost(bool, tag = "8")]
        gif: bool,
        #[prost(bool, tag = "20")]
        once: bool,
    }
    #[derive(Message)]
    struct DocumentCaption {
        #[prost(string, optional, tag = "20")]
        caption: Option<String>,
    }
    let raw = RawEnvelope::decode(payload).map_err(|_| bad("invalid caption envelope"))?;
    let (kind, caption, once) = if let Some(image) = raw.image {
        let nested = Zeroizing::new(image);
        let labels =
            ImageCaption::decode(nested.as_slice()).map_err(|_| bad("invalid image caption"))?;
        (1, labels.caption, labels.once)
    } else if let Some(video) = raw.video {
        let nested = Zeroizing::new(video);
        let labels =
            VideoCaption::decode(nested.as_slice()).map_err(|_| bad("invalid video caption"))?;
        (if labels.gif { 13 } else { 3 }, labels.caption, labels.once)
    } else if let Some(document) = raw.document {
        let nested = Zeroizing::new(document);
        let labels = DocumentCaption::decode(nested.as_slice())
            .map_err(|_| bad("invalid document caption"))?;
        (9, labels.caption, false)
    } else {
        return Ok(None);
    };
    // shortcut: quoted/expiring edits need their native context lifecycle before projection.
    if once
        || media_context(payload)?.is_some_and(|c| {
            c.stanza.is_some() || c.quoted.is_some() || c.expiration.is_some_and(|v| v != 0)
        })
    {
        return Ok(None);
    }
    // The existing 64 KiB inspector bound is below StringUtils.A0F's 65536-code-point cap.
    Ok(Some(MediaCaption { kind, caption }))
}

/// The quote a media payload names, as (stanzaId, participant), when its BWx
/// context carries a stanza ID. Resolution and attribution belong to the caller.
pub(super) fn quote_context(payload: &[u8]) -> Result<Option<(String, Option<String>)>> {
    Ok(media_context(payload)?.and_then(|c| c.stanza.map(|stanza| (stanza, c.participant))))
}

/// Project unwrapped ordinary media while retaining expiry. With `allow_quote`,
/// a context naming a stanza ID projects with `quoted: None` for the caller to
/// attribute (see `quote_context`); otherwise any quote refuses. A quoted body
/// without a stanza ID names no target and always refuses, as for text. The
/// caller must atomically persist the authenticated inbox record with the event.
pub(super) fn event_content(
    tx: &crate::store::WriteTx<'_>,
    key: &[u8],
    payload: &[u8],
    allow_quote: bool,
) -> Result<Option<(crate::model::message::MessageContent, Option<u32>)>> {
    let fields = crate::signal::direct::payload_fields(payload)?;
    if fields
        .iter()
        .any(|v| !matches!(v, 3 | 7 | 8 | 9 | 26 | 35 | 66))
        || fields
            .iter()
            .filter(|v| matches!(v, 3 | 7 | 8 | 9 | 26 | 66))
            .count()
            != 1
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
        .or(raw.sticker.as_ref())
        .or(raw.video.as_ref())
        .or(raw.ptv.as_ref())
        .or(raw.document.as_ref())
        .ok_or_else(|| bad("missing media"))?;
    let context = media_context(payload)?;
    if context.as_ref().is_some_and(|c| {
        (c.stanza.is_some() || c.quoted.is_some()) && (!allow_quote || c.stanza.is_none())
    }) {
        return Ok(None);
    }
    let expiration = context.and_then(|c| c.expiration).filter(|v| *v != 0);
    let Some(d) = descriptor(payload)? else {
        return Ok(None);
    };
    media_url(&d)?;
    #[derive(Message)]
    struct VideoCaptionOnly {
        // APK BZE.dynamicMethod: caption=7 (ImageMessage uses field 3).
        #[prost(string, optional, tag = "7")]
        caption: Option<String>,
    }
    let caption = if raw.image.is_some() {
        CaptionOnly::decode(nested.as_slice())
            .map_err(|_| bad("invalid image caption"))?
            .caption
    } else if raw.video.is_some() || raw.ptv.is_some() {
        VideoCaptionOnly::decode(nested.as_slice())
            .map_err(|_| bad("invalid video caption"))?
            .caption
    } else {
        None
    };
    #[derive(Message)]
    struct DocumentLabels {
        #[prost(string, optional, tag = "8")]
        file_name: Option<String>,
        #[prost(string, optional, tag = "20")]
        caption: Option<String>,
    }
    let (caption, file_name) = if raw.document.is_some() {
        let labels = DocumentLabels::decode(nested.as_slice())
            .map_err(|_| bad("invalid document labels"))?;
        // Display metadata only. Never use sender fileName as an output path.
        (labels.caption, labels.file_name)
    } else {
        (caption, None)
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
                    MediaKind::Sticker => crate::model::message::MediaKind::Sticker,
                    MediaKind::Document => crate::model::message::MediaKind::Document,
                    MediaKind::Video | MediaKind::VideoNote => {
                        crate::model::message::MediaKind::Video
                    }
                },
                mime_type: d.mime.clone(),
                length: Some(d.length),
            },
            caption,
            file_name,
            quoted: None,
        },
        expiration,
    )))
}

/// Account-bound references to authenticated ordinary inbox attachments. This
/// operation is offline and excludes every supported view-once marker.
pub fn references(store: &ProtocolStore) -> Result<Vec<crate::model::message::MediaReference>> {
    candidates(store)?
        .into_iter()
        .map(|(key, d, _)| {
            media_url(&d)?;
            let id = hex::encode(Sha256::digest(key));
            Ok(crate::model::message::MediaReference {
                sealed: store
                    .seal_media_reference(REFERENCE_BINDING, id.as_bytes())?
                    .into(),
                kind: match d.kind {
                    MediaKind::Image => crate::model::message::MediaKind::Image,
                    MediaKind::VoiceNote => crate::model::message::MediaKind::Audio,
                    MediaKind::Sticker => crate::model::message::MediaKind::Sticker,
                    MediaKind::Document => crate::model::message::MediaKind::Document,
                    MediaKind::Video | MediaKind::VideoNote => {
                        crate::model::message::MediaKind::Video
                    }
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
    let (_, d, _) = candidates(store)?
        .into_iter()
        .find(|(key, _, _)| hex::encode(Sha256::digest(key)) == id)
        .ok_or_else(|| bad("ordinary attachment not found"))?;
    let kind = match d.kind {
        MediaKind::Image => crate::model::message::MediaKind::Image,
        MediaKind::VoiceNote => crate::model::message::MediaKind::Audio,
        MediaKind::Sticker => crate::model::message::MediaKind::Sticker,
        MediaKind::Document => crate::model::message::MediaKind::Document,
        MediaKind::Video | MediaKind::VideoNote => crate::model::message::MediaKind::Video,
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
/// Authenticate/retrieve an ordinary attachment and export to a new private file.
/// Uses the caller's output path, never the sender's filename; never opens it.
/// Existing files/symlinks refuse. Download reservations and ciphertext persist.
pub async fn save(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
    id: &str,
    output: &std::path::Path,
) -> Result<u64> {
    // Refuse an existing target before reserving any network download. The
    // create_new writer still checks atomically after retrieval.
    match std::fs::symlink_metadata(output) {
        Ok(_) => return Err(bad("output already exists")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let (_, plain) = open(store, config, id).await?;
    ensure_available(store, id)?;
    export_new(output, &plain)
}
// Pinned43.72 EwA.A03 and 0jZ.A09/A0E compare whole MIME strings.
// These are the receiver's supported formats, not all native codec/context paths.
// See docs/protocol/apk/2.26.43.72-media-visual-mime.
pub(super) fn supported_mime(kind: MediaKind, mime: &str) -> bool {
    match kind {
        MediaKind::Image => ["image/jpeg", "image/png", "image/webp"]
            .iter()
            .any(|known| mime.eq_ignore_ascii_case(known)),
        MediaKind::VoiceNote => mime == "audio/ogg; codecs=opus",
        MediaKind::Sticker => mime.eq_ignore_ascii_case("image/webp"),
        MediaKind::Video | MediaKind::VideoNote => mime == "video/mp4",
        // Export-only MIME policy is local. Preserve its existing parameter/base
        // handling; native document ingestion does not use these visual rules.
        MediaKind::Document => {
            let mime = mime.split(';').next().unwrap_or("").trim();
            let mut parts = mime.split('/');
            let token = |part: &str| {
                !part.is_empty()
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"!#$&^_.+-".contains(&b))
            };
            mime.len() <= 256
                && parts.next().is_some_and(token)
                && parts.next().is_some_and(token)
                && parts.next().is_none()
        }
    }
}
fn media_url(d: &Descriptor) -> Result<reqwest::Url> {
    let allowed = supported_mime(d.kind, &d.mime);
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
        // APK 1qB.<clinit> 02a0..02be passes the image-key label v7
        // to the sticker type constructor; there is no separate sticker domain.
        MediaKind::Image | MediaKind::Sticker => crate::signal::image::decrypt,
        MediaKind::VoiceNote => crate::signal::image::decrypt_audio,
        MediaKind::Document => crate::signal::image::decrypt_document,
        MediaKind::Video | MediaKind::VideoNote => crate::signal::image::decrypt_video,
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
    let (source_key, d, source) = candidates(store)?
        .into_iter()
        .find(|(key, _, _)| hex::encode(Sha256::digest(key)) == id)
        .ok_or_else(|| bad("ordinary attachment not found"))?;
    let state_key = format!("media-download:{id}");
    let saved: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1))?;
    if let Some(mut saved) = saved {
        if saved["status"] == "server_refused" {
            let status = saved["http_status"]
                .as_u64()
                .filter(|status| (100..=599).contains(status) && *status != 200)
                .ok_or_else(|| bad("invalid saved media refusal status"))?;
            return Err(Error::invalid(
                "media",
                format!("media GET HTTP {status}; refusal retained; no retry"),
            ));
        }
        if saved["status"] != "downloaded" {
            return Err(bad("previous download uncertain/refused; no retry"));
        }
        let body: Zeroizing<Vec<u8>> = serde_json::from_value(saved["body"].take())
            .map_err(|_| bad("invalid saved ciphertext"))?;
        let plain = decrypt(&d, &body)?;
        ensure_source_available(store, id, &source_key, &source)?;
        return Ok((d.kind, plain));
    }
    let url = media_url(&d)?;
    let client = super::image_upload::client(config)?;
    let reservation = serde_json::json!({"status":"outcome_unknown", "reservation":super::messaging::new_message_id()?});
    store.write(|tx| {
        ensure_source(tx, &source_key, &source)?;
        if tx
            .get_json::<serde_json::Value>(Table::Checkpoints, state_key.as_bytes(), 1)?
            .is_some()
        {
            return Err(bad("download already reserved"));
        }
        tx.put_json(Table::Checkpoints, state_key.as_bytes(), 1, &reservation)
    })?;
    let mut response = client
        .get(url)
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .map_err(|_| bad("media GET failed; reservation retained"))?;
    let http_status = response.status().as_u16();
    if http_status != 200 {
        retain_download_refusal_bound(store, &state_key, http_status, &reservation)?;
        return Err(Error::invalid(
            "media",
            format!("media GET HTTP {http_status}; refusal retained; no retry"),
        ));
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
        ensure_source(tx, &source_key, &source)?;
        settle_download_in(
            tx,
            &state_key,
            &reservation,
            &serde_json::json!({"status":"downloaded","body":&*body}),
        )
    })?;
    ensure_source_available(store, id, &source_key, &source)?;
    Ok((d.kind, plain))
}

// A received HTTP refusal is a known outcome, unlike a transport interruption.
// Preserve only its numeric status; response bodies/headers can contain secrets.
#[cfg(test)]
fn retain_download_refusal(store: &ProtocolStore, state_key: &str, status: u16) -> Result<()> {
    let prior = store
        .read(|tx| tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1))?
        .unwrap_or(serde_json::Value::Null);
    retain_download_refusal_bound(store, state_key, status, &prior)
}

fn settle_download(
    store: &ProtocolStore,
    state_key: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
) -> Result<()> {
    store.write(|tx| settle_download_in(tx, state_key, expected, result))
}

// Local inbox authority fence, not an APK lifecycle/schema claim.
fn ensure_source(tx: &crate::store::WriteTx<'_>, key: &[u8], source: &[u8]) -> Result<()> {
    let current = tx.get(Table::Checkpoints, key, 1)?.map(Zeroizing::new);
    if current.as_deref().map(Vec::as_slice) != Some(source) {
        return Err(bad("ordinary media source changed before state transition"));
    }
    Ok(())
}

fn ensure_source_available(
    store: &ProtocolStore,
    id: &str,
    key: &[u8],
    source: &[u8],
) -> Result<()> {
    if !candidates(store)?
        .iter()
        .any(|(current_key, _, current_source)| {
            current_key == key
                && hex::encode(Sha256::digest(current_key)) == id
                && current_source.as_slice() == source
        })
    {
        return Err(bad(
            "ordinary media source changed or expired before plaintext handoff",
        ));
    }
    Ok(())
}

fn settle_download_in(
    tx: &mut crate::store::WriteTx<'_>,
    state_key: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
) -> Result<()> {
    let prior: Option<serde_json::Value> =
        tx.get_json(Table::Checkpoints, state_key.as_bytes(), 1)?;
    if expected["status"] != "outcome_unknown" || prior.as_ref() != Some(expected) {
        return Err(bad("media download reservation changed before settlement"));
    }
    tx.put_json(Table::Checkpoints, state_key.as_bytes(), 1, result)
}

fn retain_download_refusal_bound(
    store: &ProtocolStore,
    state_key: &str,
    status: u16,
    expected: &serde_json::Value,
) -> Result<()> {
    if !(100..=599).contains(&status) || status == 200 {
        return Err(bad("invalid media refusal status"));
    }
    settle_download(
        store,
        state_key,
        expected,
        &serde_json::json!({"status":"server_refused", "http_status":status}),
    )
}

pub(crate) fn ensure_available(store: &ProtocolStore, id: &str) -> Result<()> {
    if !candidates(store)?
        .iter()
        .any(|(key, _, _)| hex::encode(Sha256::digest(key)) == id)
    {
        return Err(bad("attachment expired or no longer available"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn image_video_mime_admission_precedes_cached_plaintext_and_get_reservation() {
        use super::*;
        use crate::{
            model::ids::AccountId,
            session::{AccountRecord, RegistrationStatus},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        for kind in [MediaKind::Image, MediaKind::Video, MediaKind::VideoNote] {
            let plain = b"synthetic visual crypto fixture";
            let encrypted = if kind == MediaKind::Image {
                crate::signal::image::encrypt(plain).unwrap()
            } else {
                crate::signal::image::encrypt_video(plain).unwrap()
            };
            let cases: &[(&str, bool)] = if kind == MediaKind::Image {
                &[
                    ("image/png", true),
                    ("IMAGE/PNG", true),
                    ("Image/Jpeg", true),
                    ("image/webp", true),
                    ("image/png; extra=1", false),
                    (" image/png", false),
                    ("image/png ", false),
                    ("image/gif", false),
                ]
            } else {
                &[
                    ("video/mp4", true),
                    ("VIDEO/MP4", false),
                    ("video/mp4; codecs=avc1", false),
                    (" video/mp4", false),
                    ("video/mp4 ", false),
                    ("video/3gpp", false),
                ]
            };
            for &(mime, accepted) in cases {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("visual-mime.redb");
                let open_store = |create| {
                    ProtocolStore::open(OpenOptions {
                        path: path.clone(),
                        account_id: AccountId::new("fixture").unwrap(),
                        create,
                        key_source: std::sync::Arc::new(StaticKeySource::new(
                            MasterKey::from_bytes([7; 32]),
                        )),
                    })
                    .unwrap()
                };
                let store = open_store(true);
                let mut config: LabRegistrationConfig =
                    serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
                config.token = Zeroizing::new("offline-fixture".into());
                let mut envelope = Envelope::default();
                if kind == MediaKind::Image {
                    envelope.image = Some(Image {
                        url: "https://mmg.whatsapp.net/fixture".into(),
                        mime: mime.into(),
                        hash: encrypted.plaintext_hash.to_vec(),
                        length: encrypted.plaintext_len,
                        key: encrypted.media_key.to_vec(),
                        enc_hash: encrypted.encrypted_hash.to_vec(),
                        once: false,
                    });
                } else {
                    let video = Some(Video {
                        url: "https://mmg.whatsapp.net/fixture".into(),
                        mime: mime.into(),
                        hash: encrypted.plaintext_hash.to_vec(),
                        length: encrypted.plaintext_len,
                        key: encrypted.media_key.to_vec(),
                        enc_hash: encrypted.encrypted_hash.to_vec(),
                        once: false,
                    });
                    if kind == MediaKind::VideoNote {
                        envelope.ptv = video;
                    } else {
                        envelope.video = video;
                    }
                }
                let key = b"received:visual-mime";
                let id = hex::encode(Sha256::digest(key));
                let cache_key = format!("media-download:{id}");
                store.write(|tx| {
                    AccountRecord {phone:Some(config.phone.clone()),registration:RegistrationStatus::Registered,
                        ..Default::default()}.save(tx)?;
                    tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":envelope.encode_to_vec()}))?;
                    tx.put_json(Table::Checkpoints,cache_key.as_bytes(),1,&serde_json::json!({"status":"downloaded","body":encrypted.body}))
                }).unwrap();
                let rows = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                let journal = store.journal_head().unwrap();
                assert_eq!(
                    list(&store).unwrap().len(),
                    usize::from(accepted),
                    "{kind:?} {mime}"
                );
                assert_eq!(
                    references(&store).unwrap().len(),
                    usize::from(accepted),
                    "{kind:?} {mime}"
                );
                if accepted {
                    let (actual, body) = open(&store, &config, &id).await.unwrap();
                    assert_eq!(actual, kind);
                    assert_eq!(&*body, plain);
                } else {
                    assert!(open(&store, &config, &id).await.is_err(), "{kind:?} {mime}");
                }
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                assert_eq!(store.journal_head().unwrap(), journal);
                if !accepted {
                    // A refused host ensures no live traffic even on regression.
                    let url = "https://not-a-media-host.invalid/fixture".to_owned();
                    if let Some(image) = envelope.image.as_mut() {
                        image.url = url;
                    } else if let Some(video) = envelope.video.as_mut().or(envelope.ptv.as_mut()) {
                        video.url = url;
                    }
                    store.write(|tx| {
                        tx.delete(Table::Checkpoints,cache_key.as_bytes())?;
                        tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":envelope.encode_to_vec()}))
                    }).unwrap();
                    drop(store);
                    let store = open_store(false);
                    let rows = store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap();
                    let journal = store.journal_head().unwrap();
                    assert!(list(&store).unwrap().is_empty());
                    assert!(open(&store, &config, &id).await.is_err());
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

    #[tokio::test]
    async fn voice_note_mime_gate_precedes_cache_handoff_and_network_reservation() {
        use super::*;
        use crate::{
            model::ids::AccountId,
            session::{AccountRecord, RegistrationStatus},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        fn page(flags: u8, seq: u32, granule: u64, payload: &[u8]) -> Vec<u8> {
            let mut p = b"OggS".to_vec();
            p.extend([0, flags]);
            p.extend(granule.to_le_bytes());
            p.extend(1u32.to_le_bytes());
            p.extend(seq.to_le_bytes());
            p.extend([0; 4]);
            p.extend([1, u8::try_from(payload.len()).unwrap()]);
            p.extend(payload);
            p
        }
        let mut header = b"OpusHead".to_vec();
        header.extend([1, 1, 56, 1, 128, 187, 0, 0, 0, 0, 0]);
        let mut plain = page(2, 0, 0, &header);
        plain.extend(page(4, 1, 96_312, &[0]));
        assert_eq!(super::super::voice_note::seconds(&plain).unwrap(), 2);
        let encrypted = crate::signal::image::encrypt_audio(&plain).unwrap();
        for (mime, ptt, once, accepted) in [
            ("audio/ogg; codecs=opus", true, false, true),
            ("audio/ogg", true, false, false),
            ("audio/ogg; codecs=vorbis", true, false, false),
            ("audio/ogg; codecs=OPUS", true, false, false),
            ("audio/ogg;codecs=opus", true, false, false),
            (" audio/ogg; codecs=opus", true, false, false),
            ("audio/ogg; codecs=opus ", true, false, false),
            ("audio/ogg; codecs=opus; extra=1", true, false, false),
            ("audio/aac", true, false, false),
            ("audio/ogg; codecs=opus", false, false, false),
            ("audio/ogg; codecs=opus", true, true, false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("voice-mime.redb");
            let open_store = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("fixture").unwrap(),
                    create,
                    key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                        [8; 32],
                    ))),
                })
                .unwrap()
            };
            let store = open_store(true);
            let mut config: LabRegistrationConfig =
                serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
            config.token = Zeroizing::new("offline-fixture".into());
            let key = b"received:voice-mime";
            let id = hex::encode(Sha256::digest(key));
            let cache_key = format!("media-download:{id}");
            let mut envelope = Envelope {
                audio: Some(Audio {
                    url: "https://mmg.whatsapp.net/offline-fixture".into(),
                    mime: mime.into(),
                    hash: encrypted.plaintext_hash.to_vec(),
                    length: encrypted.plaintext_len,
                    key: encrypted.media_key.to_vec(),
                    enc_hash: encrypted.encrypted_hash.to_vec(),
                    ptt,
                    once,
                }),
                ..Default::default()
            };
            store.write(|tx| {
                AccountRecord {phone:Some(config.phone.clone()), registration:RegistrationStatus::Registered,
                    ..Default::default()}.save(tx)?;
                tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":envelope.encode_to_vec()}))?;
                tx.put_json(Table::Checkpoints,cache_key.as_bytes(),1,&serde_json::json!({"status":"downloaded","body":encrypted.body}))
            }).unwrap();
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            assert_eq!(list(&store).unwrap().len(), usize::from(accepted), "{mime}");
            assert_eq!(
                references(&store).unwrap().len(),
                usize::from(accepted),
                "{mime}"
            );
            if accepted {
                let (kind, body) = open(&store, &config, &id).await.unwrap();
                assert_eq!(kind, MediaKind::VoiceNote);
                assert_eq!(&*body, &plain);
            } else {
                assert!(open(&store, &config, &id).await.is_err(), "{mime}");
            }
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                rows
            );
            assert_eq!(store.journal_head().unwrap(), journal);
            if !accepted {
                // A regression must never contact a live CDN during this test.
                // Cache-hit cases above use real valid Opus ciphertext; uncached
                // cases additionally use a refused host as a network backstop.
                envelope.audio.as_mut().unwrap().url =
                    "https://not-a-media-host.invalid/fixture".into();
                store.write(|tx| {
                    tx.delete(Table::Checkpoints,cache_key.as_bytes())?;
                    tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":envelope.encode_to_vec()}))
                }).unwrap();
                drop(store);
                let store = open_store(false);
                let rows = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                let journal = store.journal_head().unwrap();
                assert!(list(&store).unwrap().is_empty());
                assert!(open(&store, &config, &id).await.is_err());
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

    #[test]
    fn changed_ordinary_source_cannot_adopt_ciphertext_after_reopen() {
        use super::*;
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        for change in ["same", "replaced", "metadata", "removed"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("source.redb");
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
            let image = Image {
                url: "https://mmg.whatsapp.net/test".into(),
                mime: "image/png".into(),
                hash: vec![1; 32],
                key: vec![2; 32],
                enc_hash: vec![3; 32],
                length: 32,
                once: false,
            }
            .encode_to_vec();
            let payload = RawEnvelope {
                image: Some(image),
                ..Default::default()
            }
            .encode_to_vec();
            let source =
                serde_json::to_vec(&serde_json::json!({"status":"decrypted","protobuf":payload}))
                    .unwrap();
            let source = source.as_slice();
            let reservation =
                serde_json::json!({"status":"outcome_unknown","reservation":"original"});
            store
                .write(|tx| {
                    tx.put(Table::Checkpoints, b"received:source", 1, source)?;
                    tx.put_json(Table::Checkpoints, b"download", 1, &reservation)
                })
                .unwrap();
            store
                .write(|tx| {
                    match change {
                        "replaced" => tx.put(
                            Table::Checkpoints,
                            b"received:source",
                            1,
                            br#"{"status":"decrypted","protobuf":[4,5,6]}"#,
                        )?,
                        "metadata" => {
                            let mut changed: serde_json::Value =
                                serde_json::from_slice(source).unwrap();
                            changed["local_note"] = "changed".into();
                            tx.put_json(Table::Checkpoints, b"received:source", 1, &changed)?;
                        }
                        "removed" => {
                            tx.delete(Table::Checkpoints, b"received:source")?;
                        }
                        _ => {}
                    }
                    Ok(())
                })
                .unwrap();
            drop(store);
            let store = open(false);
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            let id = hex::encode(Sha256::digest(b"received:source"));
            assert_eq!(
                ensure_source_available(&store, &id, b"received:source", source).is_ok(),
                change == "same"
            );
            let result = store.write(|tx| {
                ensure_source(tx, b"received:source", source)?;
                settle_download_in(
                    tx,
                    "download",
                    &reservation,
                    &serde_json::json!({"status":"downloaded","body":[1,2,3]}),
                )
            });
            assert_eq!(result.is_ok(), change == "same");
            if change != "same" {
                assert_eq!(store.journal_head().unwrap(), journal);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                // Refusal observations still belong to the attempt, independent
                // of whether its source remains eligible for payload adoption.
                retain_download_refusal_bound(&store, "download", 403, &reservation).unwrap();
            }
        }
    }

    #[test]
    fn late_download_result_cannot_replace_changed_or_settled_reservation_after_reopen() {
        use super::*;
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        use std::sync::Arc;
        for status in [
            "missing",
            "outcome_unknown",
            "consumed",
            "server_refused",
            "downloaded",
            "valid",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("download-owner.redb");
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
            let expected = serde_json::json!({"status":"outcome_unknown","reservation":"original"});
            let current = if status == "valid" {
                expected.clone()
            } else {
                serde_json::json!({"status":status,"reservation":"replacement"})
            };
            if status != "missing" {
                store
                    .write(|tx| tx.put_json(Table::Checkpoints, b"download", 1, &current))
                    .unwrap();
            }
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            let result = serde_json::json!({"status":"downloaded","body":[1,2,3]});
            if status == "valid" {
                settle_download(&store, "download", &expected, &result).unwrap();
                assert!(settle_download(&store, "download", &expected, &result).is_err());
                assert!(retain_download_refusal_bound(&store, "download", 403, &expected).is_err());
            } else {
                assert!(settle_download(&store, "download", &expected, &result).is_err());
                assert!(retain_download_refusal_bound(&store, "download", 403, &expected).is_err());
                assert_eq!(store.journal_head().unwrap(), journal);
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                drop(store);
                let store = open(false);
                assert!(settle_download(&store, "download", &expected, &result).is_err());
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
            }
        }
    }

    #[test]
    fn definite_http_refusal_survives_reopen_without_overwriting_other_outcomes() {
        use super::*;
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("refusal.redb");
        let open = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("media-refusal-test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [3; 32],
                ))),
                create: true,
            })
            .unwrap()
        };
        let store = open();
        let key = "media-download:synthetic-selector";
        assert!(retain_download_refusal(&store, key, 503).is_err());
        for status in [302, 403, 404, 429, 503] {
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        key.as_bytes(),
                        1,
                        &serde_json::json!({"status":"outcome_unknown"}),
                    )
                })
                .unwrap();
            retain_download_refusal(&store, key, status).unwrap();
            assert!(retain_download_refusal(&store, key, 500).is_err());
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(
                saved,
                serde_json::json!({"status":"server_refused", "http_status":status})
            );
        }
        drop(store);
        let store = open();
        let saved: serde_json::Value = store
            .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved["http_status"], 503);
        for prior in ["downloaded", "consumed", "server_refused"] {
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        key.as_bytes(),
                        1,
                        &serde_json::json!({"status":prior,"sentinel":"preserve"}),
                    )
                })
                .unwrap();
            assert!(retain_download_refusal(&store, key, 429).is_err());
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(saved["sentinel"], "preserve");
        }
        assert!(retain_download_refusal(&store, key, 200).is_err());
    }

    #[test]
    fn own_device_media_references_use_original_wrapper_and_clear_with_chat() {
        use super::*;
        use crate::{
            binary::{Encoder, Node, TokenDictionary},
            model::jid::Jid,
            sync::SyncEvent,
        };
        #[derive(Message)]
        struct Sent {
            #[prost(string, tag = "1")]
            destination: String,
            #[prost(bytes = "vec", tag = "2")]
            message: Vec<u8>,
        }
        #[derive(Message)]
        struct Wrapper {
            #[prost(message, optional, tag = "31")]
            sent: Option<Sent>,
        }
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(crate::store::OpenOptions {
            path: dir.path().join("own-media.redb"),
            account_id: crate::model::ids::AccountId::new("fixture").unwrap(),
            create: true,
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([9; 32]),
            )),
        })
        .unwrap();
        let own = Jid::parse("15555550199@s.whatsapp.net").unwrap();
        let source = Jid::parse("12345:2@lid").unwrap();
        store
            .write(|tx| {
                crate::session::AccountRecord {
                    own_jid: Some(own.clone()),
                    registration: crate::session::RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)?;
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"pn":own.to_string(),"lid":"12345@lid"}),
                )
            })
            .unwrap();
        let chat = Jid::parse("67890@lid").unwrap();
        let inner = Envelope {
            image: Some(Image {
                url: "https://mmg.whatsapp.net/fixture".into(),
                mime: "image/png".into(),
                hash: vec![1; 32],
                key: vec![2; 32],
                enc_hash: vec![3; 32],
                length: 42,
                once: false,
            }),
            ..Default::default()
        }
        .encode_to_vec();
        let wrapper = Wrapper {
            sent: Some(Sent {
                destination: chat.to_string(),
                message: inner.clone(),
            }),
        }
        .encode_to_vec();
        let key = b"received-own:67890@lid:MEDIA";
        let node = Node::new("message")
            .attr("from", "12345:2@lid")
            .attr("recipient", chat.clone())
            .attr("id", "MEDIA")
            .attr("type", "media");
        let route = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&node)
            .unwrap();
        let event = store
            .write(|tx| {
                let event = super::super::inbox_events::own_device_message(
                    tx,
                    key,
                    &chat,
                    &node,
                    &inner,
                    &[],
                )?
                .unwrap();
                tx.put_json(
                    Table::Checkpoints,
                    key,
                    1,
                    &serde_json::json!({"status":"decrypted","source":source,"protobuf":wrapper,"route":route}),
                )?;
                Ok(event)
            })
            .unwrap();
        let SyncEvent::NewMessage(message) = event else {
            panic!("media event missing")
        };
        assert_eq!(
            message.key.direction,
            crate::model::message::Direction::Outgoing
        );
        assert_eq!(message.key.chat.jid(), &chat);
        assert!(message.key.sender.is_none());
        let crate::model::message::MessageContent::Media { reference, .. } = message.content else {
            panic!("media reference missing")
        };
        let id = hex::encode(Sha256::digest(key));
        assert_eq!(reference_selector(&store, &reference).unwrap(), id);
        assert_eq!(list(&store).unwrap()[0].id, id);
        assert!(
            store
                .read(|tx| tx.scan(Table::Outbox, 1, 10))
                .unwrap()
                .is_empty()
        );
        let parent = store
            .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key, 1))
            .unwrap();
        for (index, caption) in [(0, Some("new caption")), (1, None)] {
            let mut image = Vec::new();
            // Replacement attachment metadata must not select another attachment.
            prost::encoding::string::encode(
                1,
                &"https://mmg.whatsapp.net/different".to_owned(),
                &mut image,
            );
            prost::encoding::bytes::encode(8, &vec![99; 32], &mut image);
            if let Some(text) = caption {
                prost::encoding::string::encode(3, &text.to_owned(), &mut image);
            }
            let mut replacement = Vec::new();
            prost::encoding::bytes::encode(3, &image, &mut replacement);
            let mut protocol = b"\x0a\x07\x1a\x05MEDIA\x10\x0e".to_vec();
            prost::encoding::bytes::encode(14, &replacement, &mut protocol);
            prost::encoding::int64::encode(15, &(index + 1), &mut protocol);
            let mut body = Vec::new();
            prost::encoding::bytes::encode(12, &protocol, &mut body);
            let update_key = format!("received-own:67890@lid:IMAGE-EDIT-{index}");
            let update_node = Node::new("message")
                .attr("from", source.clone())
                .attr("recipient", chat.clone())
                .attr("id", format!("IMAGE-EDIT-{index}"))
                .attr("type", "media")
                .attr("edit", "1");
            let wrapped = Wrapper {
                sent: Some(Sent {
                    destination: chat.to_string(),
                    message: body.clone(),
                }),
            }
            .encode_to_vec();
            if caption.is_none() {
                let head = store.journal_head().unwrap();
                assert!(
                    store
                        .write(|tx| super::super::inbox_events::own_device_message(
                            tx,
                            update_key.as_bytes(),
                            &chat,
                            &update_node,
                            &body,
                            &[own.clone(), source.to_non_ad()]
                        ))
                        .unwrap()
                        .is_none()
                );
                assert_eq!(store.journal_head().unwrap(), head);
                continue;
            }
            let event = store.write(|tx| {
                let event = super::super::inbox_events::own_device_message(tx,update_key.as_bytes(),&chat,&update_node,&body,&[own.clone(),source.to_non_ad()])?.unwrap();
                tx.put_json(Table::Checkpoints,update_key.as_bytes(),1,&serde_json::json!({"status":"decrypted","source":source,"protobuf":wrapped}))?;
                tx.put_json(Table::Inbox,update_key.as_bytes(),1,&serde_json::json!({"kind":"message_updated"}))?;
                Ok(event)
            }).unwrap();
            let SyncEvent::MessageUpdated(update) = event else {
                panic!("image edit missing")
            };
            let crate::model::message::MessageChange::Edited(content) = update.change else {
                panic!("not edited")
            };
            let crate::model::message::MessageContent::Media {
                reference: edited,
                caption: actual,
                ..
            } = *content
            else {
                panic!("not media")
            };
            assert_eq!(actual.as_deref(), caption);
            assert_eq!(reference_selector(&store, &edited).unwrap(), id);
            assert_eq!(update.key.id.as_str(), "MEDIA");
            assert_eq!(
                update.key.direction,
                crate::model::message::Direction::Outgoing
            );
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<serde_json::Value>(Table::Checkpoints, key, 1))
                    .unwrap(),
                parent
            );
            assert_eq!(list(&store).unwrap().len(), 1);
        }
        super::super::chat_clear::clear_chat(&store, &chat).unwrap();
        assert!(list(&store).unwrap().is_empty());
        assert!(reference_selector(&store, &reference).is_err());
    }

    #[test]
    fn image_caption_edits_refuse_view_once_quotes_and_mixed_content() {
        assert_eq!(
            caption_fields(b"\x1a\x03\x1a\x01x")
                .unwrap()
                .unwrap()
                .caption,
            Some("x".into())
        );
        // The raw reader keeps presence; MESSAGE_EDIT admission enforces nonempty text.
        assert!(
            super::super::chat_settings::decode_edit(
                b"\x62\x0b\x0a\x03\x1a\x01P\x10\x0e\x72\x02\x1a\x00"
            )
            .is_none()
        );
        assert!(caption_fields(b"\x1a\x03\xc8\x01\x01").unwrap().is_none());
        assert!(
            caption_fields(b"\x1a\x06\x8a\x01\x03\x0a\x01P")
                .unwrap()
                .is_none()
        );
        assert!(caption_fields(b"\x1a\x00\x0a\x01x").unwrap().is_none());
        assert!(caption_fields(b"\x1a\x02\x1a\x01").is_err());
    }

    #[test]
    fn own_video_and_document_edits_use_native_kinds_and_keep_original_attachment_labels() {
        use crate::{binary::Node, model::jid::Jid, sync::SyncEvent};
        for (parent_kind, edit_kind, accepted) in [
            (1, 1, true),
            (3, 3, true),
            (13, 13, true),
            (13, 3, true),
            (3, 13, false),
            (9, 9, true),
            (1, 9, false),
            (9, 1, false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = ProtocolStore::open(crate::store::OpenOptions {
                path: dir.path().join("caption.redb"),
                account_id: crate::model::ids::AccountId::new("fixture").unwrap(),
                create: true,
                key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                    crate::store::MasterKey::from_bytes([9; 32]),
                )),
            })
            .unwrap();
            let own = Jid::parse("15555550199@s.whatsapp.net").unwrap();
            let source = own.with_device(4);
            let chat = Jid::parse("67890@lid").unwrap();
            let field = |kind| match kind {
                1 => 3,
                9 => 7,
                _ => 9,
            };
            let wrap = |body: &Vec<u8>| {
                let mut sent = Vec::new();
                prost::encoding::string::encode(1, &chat.to_string(), &mut sent);
                prost::encoding::bytes::encode(2, body, &mut sent);
                let mut wrapper = Vec::new();
                prost::encoding::bytes::encode(31, &sent, &mut wrapper);
                wrapper
            };
            let mut attachment = match parent_kind {
                1 => Image {
                    url: "https://mmg.whatsapp.net/original".into(),
                    mime: "image/png".into(),
                    hash: vec![1; 32],
                    key: vec![2; 32],
                    enc_hash: vec![3; 32],
                    length: 42,
                    once: false,
                }
                .encode_to_vec(),
                9 => Document {
                    url: "https://mmg.whatsapp.net/original".into(),
                    mime: "application/pdf".into(),
                    hash: vec![1; 32],
                    key: vec![2; 32],
                    enc_hash: vec![3; 32],
                    length: 42,
                }
                .encode_to_vec(),
                _ => Video {
                    url: "https://mmg.whatsapp.net/original".into(),
                    mime: "video/mp4".into(),
                    hash: vec![1; 32],
                    key: vec![2; 32],
                    enc_hash: vec![3; 32],
                    length: 42,
                    once: false,
                }
                .encode_to_vec(),
            };
            if parent_kind == 13 {
                prost::encoding::bool::encode(8, &true, &mut attachment);
            }
            if parent_kind == 9 {
                prost::encoding::string::encode(8, &"original.pdf".to_owned(), &mut attachment);
            }
            let mut original = Vec::new();
            prost::encoding::bytes::encode(field(parent_kind), &attachment, &mut original);
            let parent_key = b"received-own:67890@lid:P";
            store.write(|tx| {
                crate::session::AccountRecord {own_jid:Some(own.clone()),registration:crate::session::RegistrationStatus::Registered,..Default::default()}.save(tx)?;
                tx.put_json(Table::Checkpoints,parent_key,1,&serde_json::json!({"status":"decrypted","source":source,"protobuf":wrap(&original)}))
            }).unwrap();
            let mut labels = Vec::new();
            prost::encoding::string::encode(
                match edit_kind {
                    1 => 3,
                    9 => 20,
                    _ => 7,
                },
                &"edited".to_owned(),
                &mut labels,
            );
            if edit_kind == 13 {
                prost::encoding::bool::encode(8, &true, &mut labels);
            }
            if edit_kind == 9 {
                prost::encoding::string::encode(8, &"changed.pdf".to_owned(), &mut labels);
                prost::encoding::string::encode(3, &"changed title".to_owned(), &mut labels);
            }
            let mut replacement = Vec::new();
            prost::encoding::bytes::encode(field(edit_kind), &labels, &mut replacement);
            let mut protocol = b"\x0a\x03\x1a\x01P\x10\x0e".to_vec();
            prost::encoding::bytes::encode(14, &replacement, &mut protocol);
            let mut body = Vec::new();
            prost::encoding::bytes::encode(12, &protocol, &mut body);
            assert!(super::super::chat_settings::decode_edit(&body).is_some());
            let node = Node::new("message")
                .attr("from", source)
                .attr("recipient", chat.clone())
                .attr("id", "EDIT")
                .attr("type", "media")
                .attr("edit", "1");
            let snapshot = || {
                store
                    .read(|tx| {
                        Table::ALL
                            .iter()
                            .map(|t| tx.scan(*t, 1, 100))
                            .collect::<Result<Vec<_>>>()
                    })
                    .unwrap()
            };
            let before = snapshot();
            let event = store
                .write(|tx| {
                    super::super::inbox_events::own_device_message(
                        tx,
                        b"received-own:67890@lid:EDIT",
                        &chat,
                        &node,
                        &body,
                        std::slice::from_ref(&own),
                    )
                })
                .unwrap()
                .unwrap();
            if accepted {
                let SyncEvent::MessageUpdated(update) = event else {
                    panic!("missing {parent_kind}/{edit_kind} edit")
                };
                let crate::model::message::MessageChange::Edited(content) = update.change else {
                    panic!("wrong change")
                };
                let crate::model::message::MessageContent::Media {
                    reference,
                    caption,
                    file_name,
                    ..
                } = *content
                else {
                    panic!("wrong content")
                };
                assert_eq!(caption.as_deref(), Some("edited"));
                assert_eq!(
                    file_name.as_deref(),
                    if parent_kind == 9 {
                        Some("original.pdf")
                    } else {
                        None
                    }
                );
                assert_eq!(
                    reference_selector(&store, &reference).unwrap(),
                    hex::encode(Sha256::digest(parent_key))
                );
                assert_eq!(update.key.id.as_str(), "P");
            } else {
                assert!(matches!(event, SyncEvent::UnsupportedMessage(_)));
            }
            assert_eq!(snapshot(), before);
        }
    }

    #[test]
    fn media_edit_caption_presence_preserves_unicode_and_keeps_existing_inspection_bound() {
        for (field, caption_field) in [(3, 3), (7, 20), (9, 7)] {
            for text in [None, Some(""), Some(" "), Some("x")] {
                let mut nested = Vec::new();
                if let Some(text) = text {
                    prost::encoding::string::encode(caption_field, &text.to_owned(), &mut nested);
                }
                let mut replacement = Vec::new();
                prost::encoding::bytes::encode(field, &nested, &mut replacement);
                let mut protocol = b"\x0a\x03\x1a\x01P\x10\x0e".to_vec();
                prost::encoding::bytes::encode(14, &replacement, &mut protocol);
                let mut body = Vec::new();
                prost::encoding::bytes::encode(12, &protocol, &mut body);
                assert_eq!(
                    super::super::chat_settings::decode_edit(&body).is_some(),
                    text.is_some_and(|s| !s.is_empty())
                );
            }
            let mut nested = Vec::new();
            prost::encoding::string::encode(caption_field, &"😀".repeat(1000), &mut nested);
            let mut body = Vec::new();
            prost::encoding::bytes::encode(field, &nested, &mut body);
            let caption = caption_fields(&body).unwrap().unwrap().caption.unwrap();
            assert_eq!(caption.chars().count(), 1000);
            assert_eq!(caption.len(), 4000);
            let mut large = Vec::new();
            prost::encoding::string::encode(caption_field, &("😀".repeat(65536) + "x"), &mut large);
            let mut body = Vec::new();
            prost::encoding::bytes::encode(field, &large, &mut body);
            assert!(caption_fields(&body).is_err());
        }
    }

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
                    .write(|tx| event_content(tx, b"received:excluded", &payload, true))
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
                crate::binary::TokenDictionary::comparison_2_26_37_71(),
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
            crate::binary::Encoder::new(crate::binary::TokenDictionary::comparison_2_26_37_71())
                .encode(&node.clone().attr("t", "1"))
                .unwrap();
        store.write(|tx| tx.put_json(Table::Checkpoints, b"received:fresh", 1, &serde_json::json!({"status":"decrypted", "protobuf":expiring, "route":old_route}))).unwrap();
        assert!(reference_selector(&store, reference).is_err());
    }
    #[test]
    fn attachment_expiry_matches_explicit_java_time_for_route_and_capture() {
        use crate::{
            binary::{Encoder, Node, TokenDictionary},
            model::{ids::AccountId, time::Timestamp},
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("media-times.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        })
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-numeric-reference.json"
        ))
        .unwrap();
        let now = Timestamp::now().0;
        let mut cases: Vec<(Option<String>, bool)> = oracle["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|case| {
                let valid = case["valid"].as_bool().unwrap();
                let scaled = case["scaled"].as_str().unwrap().parse::<i64>().unwrap();
                (
                    Some(case["input"].as_str().unwrap().to_owned()),
                    valid && scaled.saturating_add(60000) > now,
                )
            })
            .collect();
        cases.push((None, false));
        let future = (now / 1000 + 3600).to_string();
        let future_bmp: String = future
            .chars()
            .map(|c| char::from_u32('٠' as u32 + c.to_digit(10).unwrap()).unwrap())
            .collect();
        cases.push((Some(future_bmp), true));
        for (time, expected) in cases {
            let mut node = Node::new("message")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("id", "fixture");
            if let Some(time) = &time {
                node = node.attr("t", time.clone());
            }
            let route = Encoder::new(TokenDictionary::comparison_2_26_37_71())
                .encode(&node)
                .unwrap();
            let direct = serde_json::json!({"route":route});
            assert_eq!(
                unexpired(&store, &direct, 60).unwrap(),
                expected,
                "direct {time:?}"
            );
            let capture_key = b"capture:fixture";
            store
                .write(|tx| tx.put_json(Table::Checkpoints, capture_key, 1, &route))
                .unwrap();
            let historical = serde_json::json!({"capture":capture_key.to_vec()});
            assert_eq!(
                unexpired(&store, &historical, 60).unwrap(),
                expected,
                "capture {time:?}"
            );
        }
        assert!(!unexpired(&store, &serde_json::json!({}), 60).unwrap());
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
            document: None,
            audio: None,
            sticker: None,
            video: None,
            ptv: None,
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

    // Literal APK 2.26.37.71 BZC wire tags, independently of Sticker's encoder.
    fn sticker_fixture(encrypted: &crate::signal::image::EncryptedImage) -> Vec<u8> {
        let mut body = Vec::new();
        for (tag, value) in [
            (0x0a, b"https://mmg.whatsapp.net/sticker-fixture".as_slice()),
            (0x12, encrypted.plaintext_hash.as_slice()),
            (0x1a, encrypted.encrypted_hash.as_slice()),
            (0x22, encrypted.media_key.as_slice()),
            (0x2a, b"image/webp".as_slice()),
        ] {
            body.extend_from_slice(&[tag, value.len() as u8]);
            body.extend_from_slice(value);
        }
        assert!(encrypted.plaintext_len < 128);
        body.extend_from_slice(&[0x48, encrypted.plaintext_len as u8]);
        assert!(body.len() < 256);
        let mut payload = vec![0xd2, 0x01]; // Message.stickerMessage=26.
        prost::encoding::encode_varint(body.len() as u64, &mut payload);
        payload.extend(body);
        payload
    }

    #[test]
    fn sticker_literal_schema_crypto_and_exclusion_guards() {
        // Synthetic crypto plaintext: this test does not claim renderer validity.
        let encrypted = crate::signal::image::encrypt(b"sticker crypto fixture").unwrap();
        let payload = sticker_fixture(&encrypted);
        let mut d = descriptor(&payload).unwrap().unwrap();
        assert_eq!(d.kind, MediaKind::Sticker);
        assert_eq!(
            &*decrypt(&d, &encrypted.body).unwrap(),
            b"sticker crypto fixture"
        );
        media_url(&d).unwrap();
        d.hash[0] ^= 1;
        assert!(decrypt(&d, &encrypted.body).is_err());
        d.hash[0] ^= 1;
        d.length += 1;
        assert!(decrypt(&d, &encrypted.body).is_err());
        d.length -= 1;
        let mut corrupted = encrypted.body.clone();
        corrupted[0] ^= 1;
        assert!(decrypt(&d, &corrupted).is_err());
        assert!(
            crate::signal::image::decrypt_audio(
                &encrypted.body,
                &encrypted.media_key,
                &encrypted.encrypted_hash,
                &encrypted.plaintext_hash,
                encrypted.plaintext_len
            )
            .is_err()
        );
        d.mime = "image/png".into();
        assert!(media_url(&d).is_err());
        let mut mixed = payload.clone();
        mixed.extend_from_slice(b"\x1a\x00"); // ImageMessage alongside sticker.
        assert!(descriptor(&mixed).is_err());
        let mut wrapped = payload.clone();
        wrapped.extend_from_slice(b"\xaa\x02\x00"); // viewOnceMessage=37.
        assert!(descriptor(&wrapped).unwrap().is_none());
        let mut envelope = Envelope::decode(payload.as_slice()).unwrap();
        envelope.sticker.as_mut().unwrap().lottie = true;
        assert!(descriptor(&envelope.encode_to_vec()).unwrap().is_none());
    }

    #[tokio::test]
    async fn sticker_projection_reference_and_cached_retrieval() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("stickers.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap();
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = Zeroizing::new("offline-fixture".into());
        let encrypted = crate::signal::image::encrypt(b"sticker crypto fixture").unwrap();
        let payload = sticker_fixture(&encrypted);
        let key = b"received:sticker-fixture";
        let id = hex::encode(Sha256::digest(key));
        let content = store
            .write(|tx| {
                crate::session::AccountRecord {
                    phone: Some(config.phone.clone()),
                    registration: crate::session::RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)?;
                tx.put_json(
                    Table::Checkpoints,
                    key,
                    1,
                    &serde_json::json!({"status":"decrypted","protobuf":payload}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("media-download:{id}").as_bytes(),
                    1,
                    &serde_json::json!({"status":"downloaded","body":encrypted.body}),
                )?;
                event_content(tx, key, &payload, true)
            })
            .unwrap()
            .unwrap()
            .0;
        let crate::model::message::MessageContent::Media {
            reference, caption, ..
        } = content
        else {
            panic!("missing sticker");
        };
        assert_eq!(reference.kind, crate::model::message::MediaKind::Sticker);
        assert!(caption.is_none());
        assert_eq!(reference_selector(&store, &reference).unwrap(), id);
        assert_eq!(references(&store).unwrap().len(), 1);
        let serialized = serde_json::to_string(&reference).unwrap();
        assert!(!serialized.contains("whatsapp.net"));
        assert!(!serialized.contains(&hex::encode(encrypted.media_key)));
        let (kind, plain) = open(&store, &config, &id).await.unwrap();
        assert_eq!(kind, MediaKind::Sticker);
        assert_eq!(&*plain, b"sticker crypto fixture");
        // Valid cached ciphertext cannot override unsupported MIME/Lottie
        // metadata. ASCII case variants match the pinned native WebP rule.
        for (mime, lottie, accepted) in [
            ("IMAGE/WEBP", false, true),
            ("Image/WebP", false, true),
            ("image/webp; codec=other", false, false),
            (" image/webp", false, false),
            ("image/webp ", false, false),
            ("image/png", false, false),
            ("application/was", true, false),
            ("image/webp", true, false),
        ] {
            let mut envelope = Envelope::decode(payload.as_slice()).unwrap();
            let sticker = envelope.sticker.as_mut().unwrap();
            sticker.mime = mime.into();
            sticker.lottie = lottie;
            store.write(|tx| {
                tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":envelope.encode_to_vec()}))?;
                tx.put_json(Table::Checkpoints,format!("media-download:{id}").as_bytes(),1,
                    &serde_json::json!({"status":"downloaded","body":encrypted.body}))
            }).unwrap();
            let rows = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let journal = store.journal_head().unwrap();
            assert_eq!(list(&store).unwrap().len(), usize::from(accepted), "{mime}");
            assert_eq!(
                references(&store).unwrap().len(),
                usize::from(accepted),
                "{mime}"
            );
            if accepted {
                assert_eq!(
                    &*open(&store, &config, &id).await.unwrap().1,
                    b"sticker crypto fixture"
                );
            } else {
                assert!(open(&store, &config, &id).await.is_err(), "{mime}");
            }
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                rows
            );
            assert_eq!(store.journal_head().unwrap(), journal);
            if !accepted {
                // Refused host prevents a regression from issuing live traffic
                // after removing the cache; selector assertions prove MIME gating.
                envelope.sticker.as_mut().unwrap().url =
                    "https://not-a-media-host.invalid/fixture".into();
                store.write(|tx| {
                    tx.delete(Table::Checkpoints,format!("media-download:{id}").as_bytes())?;
                    tx.put_json(Table::Checkpoints,key,1,&serde_json::json!({"status":"decrypted","protobuf":envelope.encode_to_vec()}))
                }).unwrap();
                let rows = store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap();
                let journal = store.journal_head().unwrap();
                assert!(list(&store).unwrap().is_empty());
                assert!(open(&store, &config, &id).await.is_err());
                assert_eq!(
                    store
                        .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                        .unwrap(),
                    rows
                );
                assert_eq!(store.journal_head().unwrap(), journal);
            }
        }
        // Mixed messages must never publish a sticker event.
        let mut mixed = payload.clone();
        mixed.extend_from_slice(b"\x0a\x01x");
        assert!(
            store
                .write(|tx| event_content(tx, key, &mixed, true))
                .unwrap()
                .is_none()
        );
        // BZC contextInfo=17 quotedMessage without stanzaId names no target and
        // refuses even when quotes are allowed.
        let mut envelope = Envelope::decode(payload.as_slice()).unwrap();
        let mut nested = envelope.sticker.take().unwrap().encode_to_vec();
        nested.extend_from_slice(b"\x8a\x01\x02\x1a\x00");
        let mut quoted = vec![0xd2, 0x01];
        prost::encoding::encode_varint(nested.len() as u64, &mut quoted);
        quoted.extend(nested);
        assert!(
            store
                .write(|tx| event_content(tx, key, &quoted, true))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn quoted_media_replies_project_with_text_reply_attribution() {
        use super::*;
        use crate::{
            binary::Node,
            model::{
                ids::{AccountId, EventCursor},
                message::{Direction, MessageContent},
            },
            store::{ConsumerId, MasterKey, OpenOptions, StaticKeySource},
            sync::SyncEvent,
        };
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("quoted-media.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        })
        .unwrap();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"login-identity",
                    1,
                    &serde_json::json!({"pn":"15555550999@s.whatsapp.net","lid":"99999@lid"}),
                )
            })
            .unwrap();
        #[derive(Message)]
        struct RawImage {
            #[prost(bytes = "vec", tag = "3")]
            image: Vec<u8>,
        }
        // BWx contextInfo=17 with stanzaId=1, optional participant=2 and a
        // quotedMessage=3 conversation, appended to an ordinary image.
        let image = |stanza: Option<&str>, participant: Option<&str>| {
            let mut context = Vec::new();
            if let Some(stanza) = stanza {
                prost::encoding::string::encode(1, &stanza.to_owned(), &mut context);
            }
            if let Some(participant) = participant {
                prost::encoding::string::encode(2, &participant.to_owned(), &mut context);
            }
            prost::encoding::bytes::encode(3, &b"\x0a\x02hi".to_vec(), &mut context);
            let mut nested = Image {
                url: "https://mmg.whatsapp.net/fixture".into(),
                mime: "image/png".into(),
                hash: vec![1; 32],
                length: 42,
                key: vec![2; 32],
                enc_hash: vec![3; 32],
                once: false,
            }
            .encode_to_vec();
            prost::encoding::bytes::encode(17, &context, &mut nested);
            RawImage { image: nested }.encode_to_vec()
        };
        let direct = |id: &str| {
            Node::new("message")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("id", id.to_owned())
        };
        let group = |id: &str| {
            Node::new("message")
                .attr("from", "120363000000000001@g.us")
                .attr("participant", "15555550102@s.whatsapp.net")
                .attr("id", id.to_owned())
        };
        let cases = [
            // Direct, no participant: the quoted message is the sender's own.
            (
                direct("Q1"),
                image(Some("ORIG1"), None),
                Some((
                    Direction::Incoming,
                    Some("15555550100@s.whatsapp.net"),
                    "ORIG1",
                )),
            ),
            // Participant names this account: our outgoing message.
            (
                direct("Q2"),
                image(Some("ORIG2"), Some("15555550999@s.whatsapp.net")),
                Some((Direction::Outgoing, None, "ORIG2")),
            ),
            // Group: the quoted author is a third member, not the replier.
            (
                group("Q3"),
                image(Some("ORIG3"), Some("15555550103@s.whatsapp.net")),
                Some((
                    Direction::Incoming,
                    Some("15555550103@s.whatsapp.net"),
                    "ORIG3",
                )),
            ),
            // quotedMessage without stanzaId names no target: opaque.
            (direct("Q4"), image(None, None), None),
            // An unrepresentable stanza ID stays opaque too.
            (direct("Q5"), image(Some(""), None), None),
        ];
        for (node, payload, _) in &cases {
            let id = node.get_attr("id").unwrap().as_text().unwrap();
            let key = format!("received:quoted:{id}");
            store
                .write(|tx| {
                    tx.put_json(
                        Table::Checkpoints,
                        key.as_bytes(),
                        1,
                        &serde_json::json!({"status":"decrypted", "protobuf":payload}),
                    )?;
                    super::super::inbox_events::publish(tx, key.as_bytes(), node, payload)
                })
                .unwrap();
        }
        let events = store
            .read_events(&ConsumerId::new("test").unwrap(), EventCursor::START, 10)
            .unwrap();
        assert_eq!(events.events.len(), cases.len());
        for (event, (_, _, expected)) in events.events.iter().zip(&cases) {
            match expected {
                None => assert!(matches!(event.event, SyncEvent::UnsupportedMessage(_))),
                Some((direction, sender, id)) => {
                    let SyncEvent::NewMessage(message) = &event.event else {
                        panic!("quoted media event missing")
                    };
                    let MessageContent::Media {
                        quoted: Some(quoted),
                        ..
                    } = &message.content
                    else {
                        panic!("quote missing")
                    };
                    assert_eq!(quoted.key.direction, *direction);
                    assert_eq!(quoted.key.id.as_str(), *id);
                    assert_eq!(
                        quoted.key.sender.as_ref().map(|s| s.to_jid().to_string()),
                        sender.map(str::to_owned)
                    );
                    assert_eq!(quoted.key.chat, message.key.chat);
                }
            }
        }
    }

    fn document_fixture() -> crate::signal::image::EncryptedImage {
        let f: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/document-media.json")).unwrap();
        let bytes = |name: &str| hex::decode(f[name].as_str().unwrap()).unwrap();
        crate::signal::image::EncryptedImage {
            media_key: Zeroizing::new(bytes("media_key").try_into().unwrap()),
            body: bytes("body"),
            plaintext_hash: bytes("plaintext_hash").try_into().unwrap(),
            encrypted_hash: bytes("encrypted_hash").try_into().unwrap(),
            plaintext_len: f["plaintext"].as_str().unwrap().len() as u64,
        }
    }
    fn document_payload(f: &crate::signal::image::EncryptedImage, suffix: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        // Literal BZB tags: hash=4, length=5, mediaKey=7, encHash=9.
        for (tag, value) in [
            (
                0x0a,
                b"https://mmg.whatsapp.net/document-fixture".as_slice(),
            ),
            (0x12, b"text/plain".as_slice()),
            (0x22, f.plaintext_hash.as_slice()),
            (0x3a, f.media_key.as_slice()),
            (0x4a, f.encrypted_hash.as_slice()),
        ] {
            body.extend_from_slice(&[tag, value.len() as u8]);
            body.extend_from_slice(value);
        }
        body.extend_from_slice(&[0x28, f.plaintext_len as u8]);
        body.extend_from_slice(suffix);
        let mut out = vec![0x3a]; // BZc.documentMessage=7.
        prost::encoding::encode_varint(body.len() as u64, &mut out);
        out.extend(body);
        out
    }
    #[test]
    fn document_literal_schema_distinct_crypto_domain_and_export_only_mime() {
        let fixture = document_fixture();
        let payload = document_payload(&fixture, b"");
        let mut d = descriptor(&payload).unwrap().unwrap();
        assert_eq!(d.kind, MediaKind::Document);
        assert_eq!(
            &*decrypt(&d, &fixture.body).unwrap(),
            b"synthetic document domain fixture"
        );
        media_url(&d).unwrap();
        for mime in [
            "application/pdf",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/octet-stream",
            "text/plain; charset=utf-8",
        ] {
            d.mime = mime.into();
            media_url(&d).unwrap();
        }
        for mime in [
            "",
            "text",
            "/plain",
            "text/",
            "text/plain/extra",
            "text/pla\nin",
        ] {
            d.mime = mime.into();
            assert!(media_url(&d).is_err());
        }
        for f in [
            crate::signal::image::decrypt,
            crate::signal::image::decrypt_audio,
            crate::signal::image::decrypt_video,
        ] {
            assert!(
                f(
                    &fixture.body,
                    &fixture.media_key,
                    &fixture.encrypted_hash,
                    &fixture.plaintext_hash,
                    fixture.plaintext_len
                )
                .is_err()
            );
        }
        for i in 0..fixture.body.len() {
            let mut corrupted = fixture.body.clone();
            corrupted[i] ^= 1;
            assert!(decrypt(&d, &corrupted).is_err());
        }
        d.hash[0] ^= 1;
        assert!(decrypt(&d, &fixture.body).is_err());
        for wrapper in [b"\xaa\x02\x00".as_slice(), b"\xba\x03\x00", b"\xda\x03\x00"] {
            let mut wrapped = payload.clone();
            wrapped.extend(wrapper);
            assert!(descriptor(&wrapped).unwrap().is_none());
        }
        let mut mixed = payload;
        mixed.extend_from_slice(b"\x1a\x00");
        assert!(descriptor(&mixed).is_err());
    }
    #[tokio::test]
    async fn document_journal_cached_save_restart_metadata_and_overwrite_guards() {
        use crate::{
            model::ids::{AccountId, EventCursor},
            store::{ConsumerId, MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("document.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([11; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = Zeroizing::new("offline-fixture".into());
        let fixture = document_fixture();
        // Sender filename is metadata only, even if it looks like a relative path.
        let payload = document_payload(&fixture, b"\x42\x0b../file.txt\xa2\x01\x07caption");
        let key = b"received:document";
        let id = hex::encode(Sha256::digest(key));
        let node = crate::binary::Node::new("message")
            .attr("from", "15555550100@s.whatsapp.net")
            .attr("id", "document");
        store
            .write(|tx| {
                crate::session::AccountRecord {
                    phone: Some(config.phone.clone()),
                    registration: crate::session::RegistrationStatus::Registered,
                    ..Default::default()
                }
                .save(tx)?;
                tx.put_json(
                    Table::Checkpoints,
                    key,
                    1,
                    &serde_json::json!({"status":"decrypted","protobuf":payload}),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("media-download:{id}").as_bytes(),
                    1,
                    &serde_json::json!({"status":"downloaded","body":fixture.body}),
                )?;
                assert!(super::super::inbox_events::publish(
                    tx, key, &node, &payload
                )?);
                assert!(!super::super::inbox_events::publish(
                    tx, key, &node, &payload
                )?);
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        let events = store
            .read_events(&ConsumerId::new("test").unwrap(), EventCursor::START, 10)
            .unwrap();
        assert_eq!(events.events.len(), 1);
        let crate::sync::SyncEvent::NewMessage(message) = &events.events[0].event else {
            panic!("missing document event")
        };
        let crate::model::message::MessageContent::Media {
            reference,
            caption,
            file_name,
            ..
        } = &message.content
        else {
            panic!("missing document reference")
        };
        assert_eq!(reference.kind, crate::model::message::MediaKind::Document);
        assert_eq!(caption.as_deref(), Some("caption"));
        assert_eq!(file_name.as_deref(), Some("../file.txt"));
        assert_eq!(reference_selector(&store, reference).unwrap(), id);
        let output = dir.path().join("operator-choice.txt");
        assert_eq!(
            save(&store, &config, &id, &output).await.unwrap(),
            fixture.plaintext_len
        );
        assert_eq!(
            std::fs::read(&output).unwrap(),
            b"synthetic document domain fixture"
        );
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(save(&store, &config, &id, &output).await.is_err());
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&output, &link).unwrap();
        assert!(save(&store, &config, &id, &link).await.is_err());
        assert_eq!(
            std::fs::read(&output).unwrap(),
            b"synthetic document domain fixture"
        );
        let (kind, _) = open(&store, &config, &id).await.unwrap();
        assert_eq!(kind, MediaKind::Document);
        let quoted = document_payload(&fixture, b"\x8a\x01\x03\x0a\x01q");
        // Edit-parent resolution keeps refusing quotes; projection names the
        // stanza for the caller to attribute.
        assert!(
            store
                .write(|tx| event_content(tx, key, &quoted, false))
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .write(|tx| event_content(tx, key, &quoted, true))
                .unwrap()
                .is_some()
        );
        assert_eq!(quote_context(&quoted).unwrap(), Some(("q".into(), None)));
    }

    fn video_fixture() -> crate::signal::image::EncryptedImage {
        let f: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/video-media.json")).unwrap();
        let bytes = |name: &str| hex::decode(f[name].as_str().unwrap()).unwrap();
        crate::signal::image::EncryptedImage {
            media_key: Zeroizing::new(bytes("media_key").try_into().unwrap()),
            body: bytes("body"),
            plaintext_hash: bytes("plaintext_hash").try_into().unwrap(),
            encrypted_hash: bytes("encrypted_hash").try_into().unwrap(),
            plaintext_len: f["plaintext"].as_str().unwrap().len() as u64,
        }
    }
    fn video_payload(
        encrypted: &crate::signal::image::EncryptedImage,
        ptv: bool,
        suffix: &[u8],
    ) -> Vec<u8> {
        // BZE descriptor literal plus Ayq.A12 establishes these wire tags.
        let mut body = Vec::new();
        for (tag, value) in [
            (0x0a, b"https://mmg.whatsapp.net/video-fixture".as_slice()),
            (0x12, b"video/mp4".as_slice()),
            (0x1a, encrypted.plaintext_hash.as_slice()),
            (0x32, encrypted.media_key.as_slice()),
            (0x5a, encrypted.encrypted_hash.as_slice()),
        ] {
            body.extend_from_slice(&[tag, value.len() as u8]);
            body.extend(value);
        }
        assert!(encrypted.plaintext_len < 128);
        body.extend_from_slice(&[0x20, encrypted.plaintext_len as u8]);
        body.extend(suffix);
        // Message videoMessage=9 or ptvMessage=66, both of type BZE.
        let mut payload = if ptv { vec![0x92, 0x04] } else { vec![0x4a] };
        prost::encoding::encode_varint(body.len() as u64, &mut payload);
        payload.extend(body);
        payload
    }
    #[test]
    fn video_and_ptv_literal_schema_independent_crypto_and_once_guards() {
        let fixture = video_fixture();
        for (ptv, kind) in [(false, MediaKind::Video), (true, MediaKind::VideoNote)] {
            let payload = video_payload(&fixture, ptv, b"");
            let mut d = descriptor(&payload).unwrap().unwrap();
            assert_eq!(d.kind, kind);
            assert_eq!(
                &*decrypt(&d, &fixture.body).unwrap(),
                b"synthetic video domain fixture"
            );
            assert!(
                crate::signal::image::decrypt(
                    &fixture.body,
                    &fixture.media_key,
                    &fixture.encrypted_hash,
                    &fixture.plaintext_hash,
                    fixture.plaintext_len
                )
                .is_err()
            );
            assert!(
                crate::signal::image::decrypt_audio(
                    &fixture.body,
                    &fixture.media_key,
                    &fixture.encrypted_hash,
                    &fixture.plaintext_hash,
                    fixture.plaintext_len
                )
                .is_err()
            );
            media_url(&d).unwrap();
            d.mime = "video/webm".into();
            assert!(media_url(&d).is_err());
            d.hash[0] ^= 1;
            assert!(decrypt(&d, &fixture.body).is_err());
            let once = video_payload(&fixture, ptv, b"\xa0\x01\x01"); // BZE.viewOnce=20.
            assert!(descriptor(&once).unwrap().is_none());
            for wrapper in [b"\xaa\x02\x00".as_slice(), b"\xba\x03\x00", b"\xda\x03\x00"] {
                let mut wrapped = payload.clone();
                wrapped.extend(wrapper);
                assert!(descriptor(&wrapped).unwrap().is_none());
            }
        }
        let mut mixed = video_payload(&fixture, false, b"");
        mixed.extend(video_payload(&fixture, true, b""));
        assert!(descriptor(&mixed).is_err());
    }
    #[tokio::test]
    async fn video_events_commit_once_and_cached_retrieval_preserves_kind() {
        use crate::{
            model::ids::{AccountId, EventCursor},
            store::{ConsumerId, MasterKey, OpenOptions, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("video.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = Zeroizing::new("offline-fixture".into());
        let fixture = video_fixture();
        for (ptv, kind) in [(false, MediaKind::Video), (true, MediaKind::VideoNote)] {
            // BZE caption=7; the encrypted hash at 11 must not be a caption.
            let payload = video_payload(&fixture, ptv, b"\x3a\x07caption");
            let key = if ptv {
                b"received:ptv".as_slice()
            } else {
                b"received:video".as_slice()
            };
            let id = hex::encode(Sha256::digest(key));
            let node = crate::binary::Node::new("message")
                .attr("from", "15555550100@s.whatsapp.net")
                .attr("id", if ptv { "ptv" } else { "video" });
            store
                .write(|tx| {
                    crate::session::AccountRecord {
                        phone: Some(config.phone.clone()),
                        registration: crate::session::RegistrationStatus::Registered,
                        ..Default::default()
                    }
                    .save(tx)?;
                    tx.put_json(
                        Table::Checkpoints,
                        key,
                        1,
                        &serde_json::json!({"status":"decrypted","protobuf":payload}),
                    )?;
                    tx.put_json(
                        Table::Checkpoints,
                        format!("media-download:{id}").as_bytes(),
                        1,
                        &serde_json::json!({"status":"downloaded","body":fixture.body}),
                    )?;
                    assert!(super::super::inbox_events::publish(
                        tx, key, &node, &payload
                    )?);
                    assert!(!super::super::inbox_events::publish(
                        tx, key, &node, &payload
                    )?);
                    Ok(())
                })
                .unwrap();
            let (actual, plain) = open(&store, &config, &id).await.unwrap();
            assert_eq!(actual, kind);
            assert_eq!(&*plain, b"synthetic video domain fixture");
        }
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        let events = store
            .read_events(&ConsumerId::new("test").unwrap(), EventCursor::START, 10)
            .unwrap();
        assert_eq!(events.events.len(), 2);
        for event in events.events {
            let crate::sync::SyncEvent::NewMessage(message) = event.event else {
                panic!("missing video event")
            };
            let crate::model::message::MessageContent::Media {
                reference, caption, ..
            } = message.content
            else {
                panic!("missing video reference")
            };
            assert_eq!(reference.kind, crate::model::message::MediaKind::Video);
            assert_eq!(caption.as_deref(), Some("caption"));
            assert!(reference_selector(&store, &reference).is_ok());
            let json = serde_json::to_string(&reference).unwrap();
            assert!(!json.contains("whatsapp.net"));
        }
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
                document: None,
                audio: None,
                sticker: None,
                video: None,
                ptv: None,
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
        // `media-list --json` and `GET /v1/media` serialize exactly the
        // selector, snake_case kind and advertised length: no URL, MIME,
        // key, hash or chat.
        assert_eq!(
            serde_json::to_value(list(&store).unwrap()).unwrap(),
            serde_json::json!([{
                "id": hex::encode(Sha256::digest(b"received:fixture")),
                "kind": "image",
                "bytes": 4,
            }])
        );
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
