//! Closed-enum projection over original protobuf bytes, with presence retained.
// Java varint32 truncates high bits and consumes continuation bytes; prost keeps
// unknown enum integers. Existing BP2 desktop-Java vectors cover these differences.
fn varint32(bytes: &mut &[u8]) -> Result<u32, ()> {
    let mut value = 0;
    let mut index = 0;
    loop {
        let (&byte, rest) = bytes.split_first().ok_or(())?;
        *bytes = rest;
        if index < 5 {
            value |= u32::from(byte & 127) << (index * 7);
        }
        if byte < 128 {
            return Ok(value);
        }
        index += 1;
    }
}
fn length(bytes: &mut &[u8]) -> Result<usize, ()> {
    let length = varint32(bytes)?;
    if length > i32::MAX as u32 {
        return Err(());
    }
    Ok(length as usize)
}
fn skip(bytes: &mut &[u8], key: u32, depth: usize) -> Result<(), ()> {
    if key >> 3 == 0 {
        return Err(());
    }
    let length = match key & 7 {
        0 => {
            varint32(bytes)?;
            0
        }
        1 => 8,
        2 => length(bytes)?,
        3 => {
            // shortcut: desktop Java counts the root toward limit 100; APK runtime limit is unverified.
            if depth >= 99 {
                return Err(());
            }
            loop {
                let nested = varint32(bytes)?;
                if nested & 7 == 4 {
                    if nested >> 3 != key >> 3 {
                        return Err(());
                    }
                    break;
                }
                skip(bytes, nested, depth + 1)?;
            }
            0
        }
        5 => 4,
        _ => return Err(()),
    };
    *bytes = bytes.get(length..).ok_or(())?;
    Ok(())
}

// APK BY9 scalar/bytes fields are last-one-wins; other wire types stay unknown.
#[cfg(feature = "experimental-signal-crypto")]
pub(crate) fn prekey_kem(mut body: &[u8]) -> Result<Option<(u32, &[u8])>, ()> {
    let (mut id, mut ciphertext) = (None, None);
    while !body.is_empty() {
        let key = varint32(&mut body)?;
        match key {
            56 => id = Some(varint32(&mut body)?),
            66 => {
                let length = length(&mut body)?;
                ciphertext = Some(body.get(..length).ok_or(())?);
                body = &body[length..];
            }
            _ => skip(&mut body, key, 0)?,
        }
    }
    match (id, ciphertext) {
        (None, None) => Ok(None),
        (Some(id), Some(ciphertext)) => Ok(Some((id, ciphertext))),
        _ => Err(()),
    }
}

// Concatenating singular submessage bodies preserves native protobuf merge order.
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub(super) fn message_bytes(mut body: &[u8], field: u32) -> Result<Option<Vec<u8>>, ()> {
    let mut result: Option<Vec<u8>> = None;
    while !body.is_empty() {
        let key = varint32(&mut body)?;
        if key == (field << 3 | 2) {
            let length = length(&mut body)?;
            let nested = body.get(..length).ok_or(())?;
            result
                .get_or_insert_with(Vec::new)
                .extend_from_slice(nested);
            body = &body[length..];
        } else {
            skip(&mut body, key, 0)?;
        }
    }
    Ok(result)
}
fn scan_enum(
    mut body: &[u8],
    field: u32,
    valid: impl Fn(u32) -> bool,
    depth: usize,
    mut metadata: impl FnMut(&[u8]) -> Result<(), ()>,
) -> Result<Option<u32>, ()> {
    let tag = field.checked_mul(8).filter(|v| *v != 0).ok_or(())?;
    let mut result = None;
    while !body.is_empty() {
        let original = body;
        let key = varint32(&mut body)?;
        if key == tag {
            let value = varint32(&mut body)?;
            if valid(value) {
                result = Some(value);
            }
        } else {
            skip(&mut body, key, depth)?;
            // Native treats an enum with another wire type as an unknown field.
            if key >> 3 != field {
                metadata(&original[..original.len() - body.len()])?;
            }
        }
    }
    Ok(result)
}
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub(super) fn value(
    body: &[u8],
    field: u32,
    valid: impl Fn(u32) -> bool,
    depth: usize,
) -> Result<Option<u32>, ()> {
    scan_enum(body, field, valid, depth, |_| Ok(()))
}

// Singular submessages merge. Borrow field slices; enum unknowns never reach prost
// and original payload bytes remain untouched for authentication/checkpoints.
pub(super) fn decode_nested<M: prost::Message + Default>(
    mut body: &[u8],
    message_field: u32,
    enum_field: u32,
    valid: impl Fn(u32) -> bool,
) -> Result<Option<(M, Option<u32>)>, ()> {
    let tag = message_field.checked_mul(8).filter(|v| *v != 0).ok_or(())? | 2;
    let mut result = None;
    while !body.is_empty() {
        let key = varint32(&mut body)?;
        if key == tag {
            let length = length(&mut body)?;
            let nested = body.get(..length).ok_or(())?;
            body = &body[length..];
            let (message, previous): &mut (M, Option<u32>) =
                result.get_or_insert_with(|| (M::default(), None));
            *previous = scan_enum(nested, enum_field, &valid, 1, |bytes| {
                message.merge(bytes).map_err(|_| ())
            })?
            .or(*previous);
        } else {
            skip(&mut body, key, 0)?;
        }
    }
    Ok(result)
}

#[cfg(all(test, feature = "experimental-signal-crypto"))]
mod kem_tests {
    use super::prekey_kem;
    #[test]
    fn kem_presence_last_values_wrong_wire_and_unknown_groups() {
        assert_eq!(prekey_kem(&[]), Ok(None));
        assert_eq!(prekey_kem(&[56, 0, 66, 0]), Ok(Some((0, &[][..]))));
        assert_eq!(
            prekey_kem(&[56, 1, 56, 2, 66, 1, 9, 66, 1, 8]),
            Ok(Some((2, &[8][..])))
        );
        assert_eq!(prekey_kem(&[58, 0, 64, 1]), Ok(None));
        assert_eq!(prekey_kem(&[75, 56, 9, 76]), Ok(None));
        assert_eq!(
            prekey_kem(&[56, 255, 255, 255, 255, 31, 66, 0]),
            Ok(Some((u32::MAX, &[][..])))
        );
        for body in [
            &[56, 0][..],
            &[66, 0][..],
            &[66, 2, 1][..],
            &[56, 128][..],
            &[75, 84][..],
            &[0][..],
        ] {
            assert!(prekey_kem(body).is_err());
        }
    }
}
