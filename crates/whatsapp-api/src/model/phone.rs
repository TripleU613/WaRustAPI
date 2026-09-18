//! Phone-number validation with explicit country information (docs/design/README.md section 7).
//!
//! This module does not embed a country-code table and does not guess how to
//! split an E.164 number: the caller states the country calling code. The
//! protocol's required representation (digits only, no `+`) is produced by
//! [`PhoneNumber::protocol_digits`].

use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::error::{Error, Result};

/// A validated phone number in E.164 terms.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct PhoneNumber {
    country_code: u16,
    national_number: String,
}

impl PhoneNumber {
    /// Build from an explicit country calling code and national significant
    /// number (digits only).
    pub fn new(country_code: u16, national_number: &str) -> Result<Self> {
        if !(1..=999).contains(&country_code) {
            return Err(Error::invalid("country_code", "must be 1..=999"));
        }
        let digits: String = national_number
            .chars()
            .filter(|c| !matches!(c, ' ' | '-' | '(' | ')' | '.'))
            .collect();
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Error::invalid(
                "national_number",
                "must contain only digits (with optional separators)",
            ));
        }
        if digits.starts_with('0') {
            return Err(Error::invalid(
                "national_number",
                "national significant number must not start with a trunk prefix 0",
            ));
        }
        let cc_len = country_code.to_string().len();
        let total = cc_len + digits.len();
        if !(cc_len + 4..=15).contains(&total) {
            return Err(Error::invalid(
                "national_number",
                "E.164 total length must be at most 15 digits and national part at least 4",
            ));
        }
        Ok(Self {
            country_code,
            national_number: digits,
        })
    }

    /// Parse a `+<cc><national>` string given the expected country code. The
    /// prefix is verified rather than guessed.
    pub fn from_e164_with_country(country_code: u16, e164: &str) -> Result<Self> {
        let s = e164.trim();
        let Some(rest) = s.strip_prefix('+') else {
            return Err(Error::invalid("e164", "must start with `+`"));
        };
        let cc = country_code.to_string();
        let Some(national) = rest.strip_prefix(cc.as_str()) else {
            return Err(Error::invalid(
                "e164",
                "number does not start with the stated country code",
            ));
        };
        Self::new(country_code, national)
    }

    /// Country calling code.
    pub fn country_code(&self) -> u16 {
        self.country_code
    }

    /// National significant number (digits).
    pub fn national_number(&self) -> &str {
        &self.national_number
    }

    /// `+<cc><national>` form.
    pub fn e164(&self) -> String {
        format!("+{}{}", self.country_code, self.national_number)
    }

    /// Digits without `+`, the representation the protocol uses for user JIDs.
    pub fn protocol_digits(&self) -> String {
        format!("{}{}", self.country_code, self.national_number)
    }
}

impl fmt::Debug for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print the number; phone numbers are account identifiers.
        write!(
            f,
            "PhoneNumber(+{} <{} digits>)",
            self.country_code,
            self.national_number.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats() {
        let p = PhoneNumber::new(1, "555 123-4567").unwrap();
        assert_eq!(p.e164(), "+15551234567");
        assert_eq!(p.protocol_digits(), "15551234567");
        let q = PhoneNumber::from_e164_with_country(1, "+15551234567").unwrap();
        assert_eq!(p, q);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(PhoneNumber::new(0, "5551234567").is_err());
        assert!(PhoneNumber::new(1, "05551234567").is_err());
        assert!(PhoneNumber::new(1, "1234567890123456").is_err());
        assert!(PhoneNumber::new(1, "abc").is_err());
        assert!(PhoneNumber::from_e164_with_country(44, "+15551234567").is_err());
        assert!(PhoneNumber::from_e164_with_country(1, "15551234567").is_err());
    }

    #[test]
    fn debug_is_redacted() {
        let p = PhoneNumber::new(1, "5551234567").unwrap();
        assert!(!format!("{p:?}").contains("5551234567"));
    }
}
