//! APK 37.71 LbE.A0G/Ayj.A1R plus AOSP org.json coercion reference.
//! Strict serde JSON is still the outer parser, not Android JSONTokener.

use serde_json::Value;

/// Android optInt default-zero: integral Number narrows; floating/string
/// Double truncates with saturation and NaN zero. Source links in mex.md.
pub(super) fn integer(value: Option<&Value>) -> i32 {
    match value {
        Some(Value::Number(number)) => {
            if let Some(long) = number.as_i64() {
                // Java Long.intValue retains the low 32 bits.
                long as i32
            } else {
                number.as_f64().unwrap_or_default() as i32
            }
        }
        Some(Value::String(text)) => java_double(text).unwrap_or_default() as i32,
        _ => 0,
    }
}

/// LbE.A0G: optBoolean(value, Ayj.A1R(optInt(value))); A1R means ==1.
pub(super) fn boolean(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(value)) => *value,
        Some(Value::String(value)) if value.eq_ignore_ascii_case("true") => true,
        Some(Value::String(value)) if value.eq_ignore_ascii_case("false") => false,
        _ => integer(value) == 1,
    }
}

pub(super) fn nullable_boolean(value: Option<&Value>) -> Option<bool> {
    value
        .filter(|value| !value.is_null())
        .map(|_| boolean(value))
}

pub(super) fn join_time(value: Option<&Value>) -> Option<i64> {
    let integer = integer(value);
    (integer > 0).then_some(i64::from(integer))
}

fn signed_body(text: &str) -> (&str, bool) {
    if let Some(body) = text.strip_prefix('-') {
        (body, true)
    } else {
        (text.strip_prefix('+').unwrap_or(text), false)
    }
}

fn exponent(text: &str) -> Option<i64> {
    let (body, negative) = signed_body(text);
    if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let value = body.bytes().fold(0_i64, |n, b| {
        n.saturating_mul(10).saturating_add(i64::from(b - b'0'))
    });
    Some(if negative { -value } else { value })
}

fn java_double(text: &str) -> Option<f64> {
    // Java String.trim removes <= U+0020, not all Unicode whitespace.
    let text = text.trim_matches(|c| c <= '\u{20}');
    let (body, negative) = signed_body(text);
    if body == "NaN" {
        return Some(f64::NAN);
    }
    if body == "Infinity" {
        return Some(if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    // A Java type suffix doesn't perform a float32 rounding step.
    let text = text.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(text);
    let (body, negative) = signed_body(text);
    if let Some(hex) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
        return hexadecimal(hex, negative);
    }
    let mut parts = body.split(['e', 'E']);
    let mantissa = parts.next()?;
    if let Some(exp) = parts.next() {
        exponent(exp)?;
    }
    if parts.next().is_some() {
        return None;
    }
    let mut digits = 0;
    let mut dots = 0;
    for b in mantissa.bytes() {
        if b.is_ascii_digit() {
            digits += 1;
        } else if b == b'.' {
            dots += 1;
        } else {
            return None;
        }
    }
    if digits == 0 || dots > 1 {
        return None;
    }
    text.parse().ok()
}

/// Convert hex significand using guard/sticky bits and ties-to-even, including
/// subnormal rounding. Repeated floating addition would double-round inputs
/// immediately below integral boundaries; Double.parseDouble doesn't do that.
fn hexadecimal(text: &str, negative: bool) -> Option<f64> {
    let mut parts = text.split(['p', 'P']);
    let mantissa = parts.next()?;
    let binary_exponent = exponent(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    let mut digits = 0_usize;
    let mut fraction_digits = 0_usize;
    let mut after_dot = false;
    let mut bits = 0_usize;
    let mut prefix = 0_u64;
    let mut tail_sticky = false;
    for b in mantissa.bytes() {
        if b == b'.' {
            if after_dot {
                return None;
            }
            after_dot = true;
            continue;
        }
        let nibble = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => return None,
        };
        digits += 1;
        fraction_digits += usize::from(after_dot);
        for shift in (0..4).rev() {
            let bit = (nibble >> shift) & 1;
            if bits == 0 && bit == 0 {
                continue;
            }
            bits += 1;
            if bits <= 54 {
                prefix = (prefix << 1) | u64::from(bit);
            } else {
                tail_sticky |= bit != 0;
            }
        }
    }
    if digits == 0 {
        return None;
    }
    let sign = u64::from(negative) << 63;
    if bits == 0 {
        return Some(f64::from_bits(sign));
    }
    let mut high_exponent = binary_exponent
        .saturating_add(i64::try_from(bits - 1).unwrap_or(i64::MAX))
        .saturating_sub(
            i64::try_from(fraction_digits)
                .unwrap_or(i64::MAX)
                .saturating_mul(4),
        );
    if high_exponent > 1023 {
        return Some(f64::from_bits(sign | (0x7ff << 52)));
    }
    if high_exponent < -1075 {
        return Some(f64::from_bits(sign));
    }
    let keep = if high_exponent >= -1022 {
        53
    } else {
        (high_exponent + 1075) as usize
    };
    let mut rounded = if bits <= keep {
        prefix << (keep - bits)
    } else {
        let shift = bits.min(54) - keep;
        let high = prefix >> shift;
        let guard = (prefix >> (shift - 1)) & 1 != 0;
        let sticky = tail_sticky || prefix & ((1_u64 << (shift - 1)) - 1) != 0;
        high + u64::from(guard && (sticky || high & 1 != 0))
    };
    if high_exponent < -1022 {
        return Some(f64::from_bits(sign | rounded));
    }
    if rounded == 1_u64 << 53 {
        rounded >>= 1;
        high_exponent += 1;
    }
    if high_exponent > 1023 {
        return Some(f64::from_bits(sign | (0x7ff << 52)));
    }
    Some(f64::from_bits(
        sign | (((high_exponent + 1023) as u64) << 52) | (rounded & ((1_u64 << 52) - 1)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_coercions_and_hex_rounding_match_executed_aosp_java_reference() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-coercion-reference.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let input: Value = serde_json::from_str(case["input"].as_str().unwrap()).unwrap();
            let value = input.get("value");
            let expected = &case["expected"];
            assert_eq!(
                integer(value).to_string(),
                expected["integer"].as_str().unwrap(),
                "input: {}",
                case["input"]
            );
            assert_eq!(boolean(value), expected["boolean"].as_bool().unwrap());
            assert_eq!(
                serde_json::to_value(nullable_boolean(value)).unwrap(),
                expected["nullable_boolean"]
            );
            assert_eq!(
                serde_json::to_value(join_time(value).map(|n| n.to_string())).unwrap(),
                expected["join_time"]
            );
            if let Some(Value::String(text)) = value {
                assert_eq!(
                    serde_json::to_value(java_double(text).map(|n| format!("{:x}", n.to_bits())))
                        .unwrap(),
                    expected["string_double_bits"],
                    "input: {text:?}"
                );
            }
        }
    }
}
