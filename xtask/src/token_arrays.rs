//! Bounded straight-line evaluation of token initializer arrays, not a DEX VM.
//! APK 37.71 `1Rl.<clinit>` uses constants/new-array/aput-object/sput-object only.

#[derive(Clone, Debug)]
enum Value {
    Unknown,
    Integer(i32),
    String(String),
    Array(usize),
}

struct Array {
    pages: bool,
    slots: Vec<Value>,
}

#[derive(Debug)]
pub struct TokenArrays {
    pub single: Vec<String>,
    pub double: Vec<Vec<String>>,
}

pub fn recover(
    code: &[u8],
    register_count: usize,
    mut string: impl FnMut(u32) -> Result<String, String>,
    mut type_name: impl FnMut(u32) -> Result<String, String>,
    mut field_type: impl FnMut(u32) -> Result<String, String>,
) -> Result<TokenArrays, String> {
    if register_count == 0 || register_count > 256 {
        return Err("token initializer register count outside local bound".into());
    }
    let mut registers = vec![Value::Unknown; register_count];
    let mut arrays: Vec<Array> = Vec::new();
    let mut single = None;
    let mut double = None;
    let mut offset = 0;
    let mut returned = false;
    let get = |registers: &[Value], index: u8| {
        registers
            .get(index as usize)
            .cloned()
            .ok_or_else(|| "token initializer register out of range".to_owned())
    };
    while offset < code.len() {
        let opcode = code[offset];
        // AOSP formats 11n/21s/31i/21c/31c/22c/23x. Widths are deliberately
        // restricted; unknown code, branches and payloads refuse evaluation.
        let width = match opcode {
            0x0e | 0x12 => 2,
            0x13 | 0x1a | 0x23 | 0x4d | 0x69 => 4,
            0x14 | 0x1b => 6,
            _ => {
                return Err(format!(
                    "unsupported token initializer opcode {opcode:#x} at {offset:#x}"
                ));
            }
        };
        let instruction = code
            .get(offset..offset + width)
            .ok_or("truncated token initializer instruction")?;
        let register = instruction[1];
        let word = || u16::from_le_bytes([instruction[2], instruction[3]]);
        let value = match opcode {
            0x0e => {
                if offset + width != code.len() {
                    return Err("token initializer has trailing instructions".into());
                }
                returned = true;
                break;
            }
            0x12 => Some((
                register & 15,
                Value::Integer(i32::from((register as i8) >> 4)),
            )),
            0x13 => Some((register, Value::Integer(i32::from(word() as i16)))),
            0x14 => Some((
                register,
                Value::Integer(i32::from_le_bytes(instruction[2..6].try_into().unwrap())),
            )),
            0x1a | 0x1b => {
                let index = if opcode == 0x1a {
                    u32::from(word())
                } else {
                    u32::from_le_bytes(instruction[2..6].try_into().unwrap())
                };
                Some((register, Value::String(string(index)?)))
            }
            0x23 => {
                let Value::Integer(size) = get(&registers, register >> 4)? else {
                    return Err("token array size is not a known integer".into());
                };
                if !(1..=256).contains(&size) || arrays.len() >= 8 {
                    return Err("token array allocation outside local bound".into());
                }
                let pages = match type_name(u32::from(word()))?.as_str() {
                    "[Ljava/lang/String;" => false,
                    "[[Ljava/lang/String;" => true,
                    _ => return Err("unsupported token array element type".into()),
                };
                let index = arrays.len();
                arrays.push(Array {
                    pages,
                    slots: vec![Value::Unknown; size as usize],
                });
                Some((register & 15, Value::Array(index)))
            }
            0x4d => {
                let Value::Array(array) = get(&registers, instruction[2])? else {
                    return Err("token array write target is not an array".into());
                };
                let Value::Integer(index) = get(&registers, instruction[3])? else {
                    return Err("token array index is not a known integer".into());
                };
                let index = usize::try_from(index).map_err(|_| "negative token array index")?;
                let value = get(&registers, register)?;
                let compatible = if arrays[array].pages {
                    matches!(&value, Value::Array(page) if !arrays[*page].pages)
                } else {
                    matches!(&value, Value::String(_) | Value::Integer(0))
                };
                if !compatible {
                    return Err("token array value type mismatch".into());
                }
                let slot = arrays[array]
                    .slots
                    .get_mut(index)
                    .ok_or("token array index out of range")?;
                if !matches!(slot, Value::Unknown) {
                    return Err("repeated token array assignment".into());
                }
                *slot = value;
                None
            }
            0x69 => {
                let Value::Array(array) = get(&registers, register)? else {
                    return Err("token static field value is not an array".into());
                };
                let destination = match field_type(u32::from(word()))?.as_str() {
                    "[Ljava/lang/String;" if !arrays[array].pages => &mut single,
                    "[[Ljava/lang/String;" if arrays[array].pages => &mut double,
                    _ => return Err("unsupported token static field type".into()),
                };
                if destination.replace(array).is_some() {
                    return Err("multiple token static field assignments".into());
                }
                None
            }
            _ => unreachable!(),
        };
        if let Some((register, value)) = value {
            *registers
                .get_mut(register as usize)
                .ok_or("token initializer register out of range")? = value;
        }
        offset += width;
    }
    if !returned {
        return Err("token initializer missing final return-void".into());
    }
    let strings = |array: usize, allow_null: bool| {
        arrays[array]
            .slots
            .iter()
            .enumerate()
            .map(|(index, slot)| match slot {
                Value::String(s)
                    if !s.is_empty()
                        && s.len() <= 64
                        && s.bytes().all(|b| (0x21..0x7f).contains(&b)) =>
                {
                    Ok(s.clone())
                }
                Value::Integer(0) if allow_null && index == 0 => Ok(String::new()),
                _ => Err("incomplete or invalid token array slot".to_owned()),
            })
            .collect::<Result<Vec<_>, _>>()
    };
    let single = strings(single.ok_or("missing single-byte token field")?, true)?;
    let double = arrays[double.ok_or("missing double-byte token field")?]
        .slots
        .iter()
        .map(|slot| match slot {
            Value::Array(page) => strings(*page, false),
            _ => Err("incomplete double-byte token page".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TokenArrays { single, double })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        // Out-of-order writes: slot 2 before slot 1. The integer operand
        // contains 1a 00 01 00, which the old raw-byte scan saw as a string.
        vec![
            0x14, 4, 0x1a, 0, 1, 0, 0x13, 2, 3, 0, 0x23, 0x20, 0, 0, 0x12, 1, 0x12, 2, 0x4d, 1, 0,
            2, 0x13, 2, 2, 0, 0x1b, 1, 2, 0, 0, 0, 0x4d, 1, 0, 2, 0x13, 2, 1, 0, 0x1a, 1, 0, 0,
            0x4d, 1, 0, 2, 0x69, 0, 0, 0, 0x13, 2, 1, 0, 0x23, 0x23, 1, 0, 0x23, 0x20, 0, 0, 0x1a,
            1, 1, 0, 0x12, 2, 0x4d, 1, 0, 2, 0x4d, 0, 3, 2, 0x69, 3, 1, 0, 0x0e, 0,
        ]
    }

    fn run(code: &[u8]) -> Result<TokenArrays, String> {
        recover(
            code,
            5,
            |i| Ok(["xmlstreamstart", "active", "xmlstreamend"][i as usize].into()),
            |i| Ok(["[Ljava/lang/String;", "[[Ljava/lang/String;"][i as usize].into()),
            |i| Ok(["[Ljava/lang/String;", "[[Ljava/lang/String;"][i as usize].into()),
        )
    }

    #[test]
    fn indexed_arrays_ignore_opcode_like_operands_and_preserve_jumbo_order() {
        let result = run(&fixture()).unwrap();
        assert_eq!(result.single, ["", "xmlstreamstart", "xmlstreamend"]);
        assert_eq!(result.double, [vec!["active"]]);
    }

    #[test]
    fn unsupported_partial_and_invalid_index_inputs_refuse() {
        let valid = fixture();
        for length in 0..valid.len() {
            assert!(run(&valid[..length]).is_err());
        }
        // A const/16 operand ending in 0e 00 is not a decoded return-void.
        let mut fake_return = valid[..valid.len() - 2].to_vec();
        fake_return.extend([0x13, 4, 0x0e, 0]);
        assert!(
            run(&fake_return)
                .unwrap_err()
                .contains("missing final return")
        );
        let mut branched = valid.clone();
        branched[0] = 0x28;
        assert!(run(&branched).unwrap_err().contains("unsupported"));
        let mut bad_index = valid.clone();
        bad_index[24] = 3;
        assert!(run(&bad_index).is_err());
    }
}
