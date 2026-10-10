//! GsB parse/serialize projection used by APK 37.71 11T.A0D before signatures.
//! Known proto2 fields are written in tag order, then lite unknown fields in
//! encounter order. Prost alone drops unknowns and keeps unknown enum values.
use crate::proto::wa_adv::AdvDeviceIdentity;
use prost::Message;

fn read(bytes: &mut &[u8]) -> Result<u64, ()> {
    let mut value = 0u64;
    for index in 0..10 {
        let (&byte, rest) = bytes.split_first().ok_or(())?;
        *bytes = rest;
        value |= u64::from(byte & 127) << (index * 7);
        if byte < 128 {
            return Ok(value);
        }
    }
    Err(())
}
fn write(mut value: u64, out: &mut Vec<u8>) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
fn take<'a>(bytes: &mut &'a [u8], length: usize) -> Result<&'a [u8], ()> {
    let value = bytes.get(..length).ok_or(())?;
    *bytes = &bytes[length..];
    Ok(value)
}
fn unknown(bytes: &mut &[u8], tag: u32, depth: usize, out: &mut Vec<u8>) -> Result<(), ()> {
    if tag >> 3 == 0 {
        return Err(());
    }
    write(u64::from(tag), out);
    match tag & 7 {
        0 => write(read(bytes)?, out),
        1 => out.extend_from_slice(take(bytes, 8)?),
        2 => {
            let length = read(bytes)? as u32;
            if length > i32::MAX as u32 {
                return Err(());
            }
            let body = take(bytes, length as usize)?;
            write(u64::from(length), out);
            out.extend_from_slice(body);
        }
        3 => {
            // Desktop javalite oracle counts the root in its 100-level bound.
            // This is our bounded reference policy, not a proven APK limit.
            if depth >= 99 {
                return Err(());
            }
            loop {
                let nested = read(bytes)? as u32;
                if nested & 7 == 4 {
                    if nested >> 3 != tag >> 3 {
                        return Err(());
                    }
                    write(u64::from(nested), out);
                    break;
                }
                unknown(bytes, nested, depth + 1, out)?;
            }
        }
        5 => out.extend_from_slice(take(bytes, 4)?),
        _ => return Err(()),
    }
    Ok(())
}

pub(super) fn parse(mut bytes: &[u8]) -> Result<(AdvDeviceIdentity, Vec<u8>), ()> {
    let mut details = AdvDeviceIdentity::default();
    let mut extras = Vec::new();
    while !bytes.is_empty() {
        let tag = read(&mut bytes)? as u32;
        match tag {
            8 => details.raw_id = Some(read(&mut bytes)? as u32),
            16 => details.timestamp = Some(read(&mut bytes)?),
            24 => details.key_index = Some(read(&mut bytes)? as u32),
            32 | 40 => {
                // GsB uses IQg/Azc's closed enum 0..2. Unknown int32 enum
                // values become unknown VARINT fields, sign-extended to int64.
                let value = read(&mut bytes)? as i32;
                if matches!(value, 0..=2) {
                    if tag == 32 {
                        details.account_type = Some(value);
                    } else {
                        details.device_type = Some(value);
                    }
                } else {
                    write(u64::from(tag), &mut extras);
                    write(value as i64 as u64, &mut extras);
                }
            }
            _ => unknown(&mut bytes, tag, 0, &mut extras)?,
        }
    }
    let mut serialized = details.encode_to_vec();
    serialized.extend(extras);
    Ok((details, serialized))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialization_and_presence_match_independent_java_lite_oracle() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/prekeys/2.26.37.71-adv-details-oracle.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let bytes = hex::decode(case["hex"].as_str().unwrap()).unwrap();
            let parsed = parse(&bytes).ok();
            let expected = case["serialized"].as_str();
            assert_eq!(
                parsed
                    .as_ref()
                    .map(|(_, bytes)| hex::encode(bytes))
                    .as_deref(),
                expected,
                "{}",
                case["hex"]
            );
            if let Some((details, _)) = parsed {
                let fields = [
                    details.raw_id.map(u64::from),
                    details.timestamp,
                    details.key_index.map(u64::from),
                    details.account_type.map(|v| v as u64),
                    details.device_type.map(|v| v as u64),
                ];
                assert_eq!(
                    serde_json::to_value(fields).unwrap(),
                    case["fields"],
                    "{}",
                    case["hex"]
                );
            }
        }
    }
}
