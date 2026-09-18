//! Bounded single-stream Ogg/Opus input and comparison LX/BWZ audio schema.
use crate::{Error, Result};
use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub(super) struct AudioMessage {
    #[prost(string, tag = "1")]
    pub url: String,
    #[prost(string, tag = "2")]
    pub mimetype: String,
    #[prost(bytes = "vec", tag = "3")]
    pub file_sha256: Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub file_length: u64,
    #[prost(uint32, tag = "5")]
    pub seconds: u32,
    #[prost(bool, tag = "6")]
    pub ptt: bool,
    #[prost(bytes = "vec", tag = "7")]
    pub media_key: Vec<u8>,
    #[prost(bytes = "vec", tag = "8")]
    pub file_enc_sha256: Vec<u8>,
    #[prost(string, tag = "9")]
    pub direct_path: String,
    #[prost(int64, tag = "10")]
    pub media_key_timestamp: i64,
}

pub(super) fn seconds(bytes: &[u8]) -> Result<u32> {
    let bad = || {
        Error::invalid(
            "voice_note",
            "expected bounded single-stream mono Ogg/Opus with complete pages",
        )
    };
    let mut offset = 0;
    let mut sequence = 0u32;
    let mut serial = None;
    let mut preskip = 0u64;
    let mut final_granule = None;
    while offset < bytes.len() {
        let header = bytes.get(offset..offset + 27).ok_or_else(bad)?;
        if &header[..4] != b"OggS" || header[4] != 0 || final_granule.is_some() {
            return Err(bad());
        }
        let page_serial = u32::from_le_bytes(header[14..18].try_into().expect("checked"));
        if serial.is_some_and(|s| s != page_serial)
            || u32::from_le_bytes(header[18..22].try_into().expect("checked")) != sequence
        {
            return Err(bad());
        }
        serial = Some(page_serial);
        let lacing = bytes
            .get(offset + 27..offset + 27 + usize::from(header[26]))
            .ok_or_else(bad)?;
        let start = offset + 27 + lacing.len();
        let end = start + lacing.iter().map(|v| usize::from(*v)).sum::<usize>();
        let payload = bytes.get(start..end).ok_or_else(bad)?;
        if sequence == 0 {
            if header[5] != 2
                || payload.len() != 19
                || &payload[..8] != b"OpusHead"
                || payload[8] != 1
                || payload[9] != 1
                || payload[18] != 0
            {
                return Err(bad());
            }
            preskip = u64::from(u16::from_le_bytes([payload[10], payload[11]]));
        }
        if header[5] & 4 != 0 {
            final_granule = Some(u64::from_le_bytes(
                header[6..14].try_into().expect("checked"),
            ));
        }
        offset = end;
        sequence = sequence.checked_add(1).ok_or_else(bad)?;
    }
    let samples = final_granule
        .and_then(|g| g.checked_sub(preskip))
        .filter(|s| *s > 0 && *s <= 48_000 * 600)
        .ok_or_else(bad)?;
    Ok(samples.div_ceil(48_000) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duration_from_granule_minus_preskip_and_truncation_rejected() {
        let mut header = b"OpusHead".to_vec();
        header.extend_from_slice(&[1, 1, 56, 1, 128, 187, 0, 0, 0, 0, 0]);
        fn page(flags: u8, seq: u32, granule: u64, payload: &[u8]) -> Vec<u8> {
            let mut p = b"OggS".to_vec();
            p.extend_from_slice(&[0, flags]);
            p.extend_from_slice(&granule.to_le_bytes());
            p.extend_from_slice(&1u32.to_le_bytes());
            p.extend_from_slice(&seq.to_le_bytes());
            p.extend_from_slice(&[0; 4]);
            p.push(1);
            p.push(payload.len() as u8);
            p.extend_from_slice(payload);
            p
        }
        let mut bytes = page(2, 0, 0, &header);
        bytes.extend(page(4, 1, 96_312, &[0]));
        assert_eq!(seconds(&bytes).unwrap(), 2);
        bytes.pop();
        assert!(seconds(&bytes).is_err());
    }
    #[test]
    fn audio_schema_has_ptt_and_duration_not_image_tags() {
        let audio = AudioMessage {
            seconds: 2,
            ptt: true,
            ..Default::default()
        };
        assert_eq!(audio.encode_to_vec(), [40, 2, 48, 1]);
        assert!(seconds(b"not ogg").is_err());
        assert!(seconds(&[]).is_err());
    }
}
