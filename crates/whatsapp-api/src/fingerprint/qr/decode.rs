//! Bounded native fingerprint-schema decoder, APK 2.26.37.71 protobuf-C.
//! 0x703158 unpack, 0x7035d0..0x7038b8 wire scanning, 0x704bcc..0x705630
//! singular field assignment/nested merge. Actual 50-case corpus is recorded
//! in docs/protocol/fingerprint-native-parser-cases.json. Not a general protobuf
//! decoder; opaque unknown fields are skipped, never used as verification data.
use super::{Combined, Individual, MAX_COMPARISON_PAYLOAD_BYTES};

enum Field<'a> {
    Varint(u32),
    Bytes(&'a [u8]),
    Fixed,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, size: usize) -> Result<&'a [u8], ()> {
        let end = self.offset.checked_add(size).ok_or(())?;
        let value = self.bytes.get(self.offset..end).ok_or(())?;
        self.offset = end;
        Ok(value)
    }
    fn varint(&mut self, limit: usize) -> Result<u64, ()> {
        let mut value = 0u64;
        for index in 0..limit {
            let byte = self.take(1)?[0];
            // Native wire scanning only checks termination within ten bytes,
            // not a <=1 final byte at position ten. Low 32-bit scalar decoding
            // discards higher bits. Wrapping here preserves that native rule.
            value |= u64::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(())
    }
    fn next(&mut self) -> Result<Option<(u32, Field<'a>)>, ()> {
        let Some(first) = self.bytes.get(self.offset).copied() else {
            return Ok(None);
        };
        // Native 0x7035dc rejects first byte <8, not decoded field-id zero.
        // Thus overlong zero tags differ from standard prost validation.
        if first < 8 {
            return Err(());
        }
        let key = self.varint(5)?;
        // Native accumulates field id in w26 with shifts 4,11,18,25 after
        // the first byte: up to 32 field-id bits, not the standard 29-bit cap.
        let tag = (key >> 3) as u32;
        let value = match first & 7 {
            0 => Field::Varint(self.varint(10)? as u32),
            1 => {
                self.take(8)?;
                Field::Fixed
            }
            2 => {
                // Native 0x703808..0x703898: at most five length bytes,
                // reject any value >=2^31, then check bytes remaining.
                let size = self.varint(5)?;
                if size >= 1 << 31 {
                    return Err(());
                }
                Field::Bytes(self.take(usize::try_from(size).map_err(|_| ())?)?)
            }
            5 => {
                self.take(4)?;
                Field::Fixed
            }
            // Native rejects groups (3/4) as well as unknown wire types 6/7.
            _ => return Err(()),
        };
        Ok(Some((tag, value)))
    }
}

fn merge_individual(side: &mut Individual, bytes: &[u8]) -> Result<(), ()> {
    let mut cursor = Cursor { bytes, offset: 0 };
    while let Some((tag, value)) = cursor.next()? {
        // APK native descriptor 0x740b60: bytes tags 1/2/3/4/6, enum tag 5.
        // Scalars/bytes use last value, including explicitly present empty.
        // Unknown fields remain opaque: native retains them, but comparison
        // only reads these known fields, so we do not allocate their copies.
        match (tag, value) {
            (1, Field::Bytes(value)) => side.public_key = Some(value.to_vec()),
            (2, Field::Bytes(value)) => side.pn_identifier = Some(value.to_vec()),
            (3, Field::Bytes(value)) => side.lid_identifier = Some(value.to_vec()),
            (4, Field::Bytes(value)) => side.username_identifier = Some(value.to_vec()),
            (5, Field::Varint(value)) => side.hosted_state = Some(value as i32),
            (6, Field::Bytes(value)) => side.hashed_public_key = Some(value.to_vec()),
            (1..=6, _) => return Err(()),
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn combined(bytes: &[u8]) -> Result<Combined, ()> {
    if bytes.len() > MAX_COMPARISON_PAYLOAD_BYTES {
        return Err(());
    }
    let mut result = Combined::default();
    let mut cursor = Cursor { bytes, offset: 0 };
    while let Some((tag, value)) = cursor.next()? {
        // APK combined descriptor 0x740c08: uint32 version=1, messages=2/3.
        // Native repeated singular message fields merge, not replace the
        // entire earlier holder. Existing earlier fields survive absent ones.
        match (tag, value) {
            (1, Field::Varint(value)) => result.version = Some(value),
            (2, Field::Bytes(value)) => {
                merge_individual(result.local.get_or_insert_default(), value)?
            }
            (3, Field::Bytes(value)) => {
                merge_individual(result.remote.get_or_insert_default(), value)?
            }
            (1..=3, _) => return Err(()),
            _ => {}
        }
    }
    Ok(result)
}
