//! Independent exact-rational implementation of the Java 21 Double.toString
//! decimal-selection contract. This is a host-platform reference port, not
//! recovered APK code or verified Android-runtime equivalence. Source/provenance
//! and executed reference cases are recorded in docs/protocol/mex.md.

use std::cmp::Ordering;

// Little-endian base-2^32 unsigned limbs. Only shifts and scalar products are
// needed for exact binary/decimal scaling; no external bigint dependency.
#[derive(Clone)]
struct Natural(Vec<u32>);
impl Natural {
    fn new(value: u64) -> Self {
        let mut limbs = vec![value as u32];
        if value >> 32 != 0 {
            limbs.push((value >> 32) as u32);
        }
        if value == 0 {
            limbs.clear();
        }
        Self(limbs)
    }
    fn shifted(&self, bits: u32) -> Self {
        if self.0.is_empty() {
            return Self::new(0);
        }
        let mut limbs = vec![0; (bits / 32) as usize];
        let mut carry = 0u64;
        for &limb in &self.0 {
            let value = (u64::from(limb) << (bits % 32)) | carry;
            limbs.push(value as u32);
            carry = value >> 32;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
        Self(limbs)
    }
    fn multiplied(&self, factor: u64) -> Self {
        if factor == 0 {
            return Self::new(0);
        }
        let mut limbs = Vec::with_capacity(self.0.len() + 2);
        let mut carry = 0u128;
        for &limb in &self.0 {
            let value = u128::from(limb) * u128::from(factor) + carry;
            limbs.push(value as u32);
            carry = value >> 32;
        }
        while carry != 0 {
            limbs.push(carry as u32);
            carry >>= 32;
        }
        Self(limbs)
    }
    fn decimal_scaled(&self, exponent: u32) -> Self {
        let mut value = self.clone();
        for _ in 0..exponent {
            value = value.multiplied(10);
        }
        value
    }
    fn compare(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

fn binary_ratio(value: f64) -> (Natural, Natural) {
    // IEEE 754 binary64: 52 fraction bits, biased exponent 1023; normals
    // have an implicit bit 52, subnormals use exponent -1074.
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mantissa, exponent) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1u64 << 52), biased - 1075)
    };
    if exponent >= 0 {
        (
            Natural::new(mantissa).shifted(exponent as u32),
            Natural::new(1),
        )
    } else {
        (
            Natural::new(mantissa),
            Natural::new(1).shifted((-exponent) as u32),
        )
    }
}

fn compare_decimal_power(num: &Natural, den: &Natural, exponent: i32) -> Ordering {
    if exponent >= 0 {
        num.compare(&den.decimal_scaled(exponent as u32))
    } else {
        num.decimal_scaled((-exponent) as u32).compare(den)
    }
}

fn quotient(num: &Natural, den: &Natural) -> u64 {
    // A binary64 shortest decimal has at most 17 digits. Exact order
    // correction bounds the scaled significand below 10^17.
    let mut lower = 0;
    let mut upper = 100_000_000_000_000_001u64;
    while upper - lower > 1 {
        let middle = lower + (upper - lower) / 2;
        if den.multiplied(middle).compare(num) != Ordering::Greater {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    lower
}

fn renders_to(value: f64, coefficient: u64, exponent: i32) -> bool {
    format!("{coefficient}e{exponent}")
        .parse::<f64>()
        .is_ok_and(|parsed| parsed.to_bits() == value.to_bits())
}

/// Java platform-reference spelling. None means a local decimal-selection
/// invariant failed; callers must diagnose it rather than fabricate a value.
pub(super) fn format(value: f64) -> Option<String> {
    if value.is_nan() {
        return Some("NaN".into());
    }
    if value.is_infinite() {
        return Some(
            if value.is_sign_negative() {
                "-Infinity"
            } else {
                "Infinity"
            }
            .into(),
        );
    }
    if value == 0.0 {
        return Some(
            if value.is_sign_negative() {
                "-0.0"
            } else {
                "0.0"
            }
            .into(),
        );
    }
    let magnitude = value.abs();
    // Rust supplies only the shortest length and an initial order estimate.
    // Its selected decimal is not used as Java's selected value. Exact binary
    // arithmetic below handles near-power-of-ten order and nearest/even ties.
    let shortest = format!("{magnitude:e}");
    let (significand, estimate) = shortest.split_once('e')?;
    let digits = significand.replace('.', "");
    let precision = digits.trim_end_matches('0').len().max(2);
    if precision > 17 {
        return None;
    }
    let (num, den) = binary_ratio(magnitude);
    let mut order = estimate.parse::<i32>().ok()?;
    while compare_decimal_power(&num, &den, order) == Ordering::Less {
        order -= 1;
    }
    while compare_decimal_power(&num, &den, order + 1) != Ordering::Less {
        order += 1;
    }
    let exponent = order - precision as i32 + 1;
    let (scaled_num, scaled_den) = if exponent >= 0 {
        (num, den.decimal_scaled(exponent as u32))
    } else {
        (num.decimal_scaled((-exponent) as u32), den)
    };
    let lower = quotient(&scaled_num, &scaled_den);
    let upper = lower + 1;
    let lower_valid = renders_to(magnitude, lower, exponent);
    let upper_valid = renders_to(magnitude, upper, exponent);
    let mut coefficient = match (lower_valid, upper_valid) {
        (true, false) => lower,
        (false, true) => upper,
        (true, true) => {
            // Compare twice the exact value with the two candidates' sum.
            // At a tie choose the even coefficient, as Java specifies.
            match scaled_num
                .multiplied(2)
                .compare(&scaled_den.multiplied(lower + upper))
            {
                Ordering::Less => lower,
                Ordering::Greater => upper,
                Ordering::Equal => {
                    if lower & 1 == 0 {
                        lower
                    } else {
                        upper
                    }
                }
            }
        }
        (false, false) => return None,
    };
    let mut exponent = exponent;
    while coefficient % 10 == 0 {
        coefficient /= 10;
        exponent += 1;
    }
    let digits = coefficient.to_string();
    let order = digits.len() as i32 + exponent - 1;
    let mut output = if (-3..7).contains(&order) {
        let point = digits.len() as i32 + exponent;
        if point <= 0 {
            format!("0.{}{digits}", "0".repeat((-point) as usize))
        } else if point >= digits.len() as i32 {
            format!("{digits}{}.0", "0".repeat((point as usize) - digits.len()))
        } else {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        }
    } else {
        format!(
            "{}.{}E{order}",
            &digits[..1],
            if digits.len() == 1 { "0" } else { &digits[1..] }
        )
    };
    if value.is_sign_negative() {
        output.insert(0, '-');
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decimal_selection_matches_executed_host_java() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-double-string-reference.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let bits = u64::from_str_radix(case["bits"].as_str().unwrap(), 16).unwrap();
            assert_eq!(
                format(f64::from_bits(bits)).as_deref(),
                case["scalar"].as_str(),
                "{bits:016x}"
            );
        }
    }
}
