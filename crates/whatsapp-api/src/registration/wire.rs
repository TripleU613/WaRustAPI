//! Version-scoped registration request construction.
//!
//! These facts were statically observed in the separately hashed, signed
//! WhatsApp 2.26.37.70 comparison build. They are deliberately not installed
//! into the active 2.26.36.75 [`crate::ProtocolProfile`]. This module performs
//! no I/O and cannot request an OTP.

use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{Error, Result};

/// Maximum fields accepted in one locally constructed registration form.
pub const MAX_FORM_FIELDS: usize = 256;
/// Maximum UTF-8 bytes accepted in one field name.
pub const MAX_FIELD_NAME_BYTES: usize = 128;
/// Maximum UTF-8 bytes accepted in one field value.
pub const MAX_FIELD_VALUE_BYTES: usize = 1 << 20;
/// Maximum encoded registration body size.
pub const MAX_FORM_BODY_BYTES: usize = 2 << 20;

/// Registration endpoints present in signed Android build 2.26.37.70.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RegistrationEndpoint {
    /// Registration onboarding experiment properties.
    RegOnboardAbprop,
    /// Test whether an account/number exists.
    Exist,
    /// Request or process a verification code.
    Code,
    /// Request account verification.
    AcVerifyRequest,
    /// Perform account verification.
    AcVerify,
    /// Complete registration.
    Register,
    /// Security-related registration step.
    Security,
    /// Fetch automatic configuration.
    Autoconf,
    /// Verify automatic configuration.
    AutoconfVerifier,
    /// Send a device-logout record.
    DeviceLogoutSend,
    /// Fetch device-logout records.
    DeviceLogoutFetch,
    /// Upload a registration client log.
    ClientLog,
    /// Upload a pre-phone-number client log.
    PrePnClientLog,
    /// Submit consent state.
    Consent,
    /// Registration challenge endpoint.
    Challenge,
    /// Passkey authentication endpoint.
    PasskeyAuth,
    /// WFS registration endpoint.
    Wfs,
}

impl RegistrationEndpoint {
    /// Every endpoint observed in the comparison build.
    pub const ALL: [Self; 17] = [
        Self::RegOnboardAbprop,
        Self::Exist,
        Self::Code,
        Self::AcVerifyRequest,
        Self::AcVerify,
        Self::Register,
        Self::Security,
        Self::Autoconf,
        Self::AutoconfVerifier,
        Self::DeviceLogoutSend,
        Self::DeviceLogoutFetch,
        Self::ClientLog,
        Self::PrePnClientLog,
        Self::Consent,
        Self::Challenge,
        Self::PasskeyAuth,
        Self::Wfs,
    ];

    /// Absolute path encoded in the client.
    pub const fn path(self) -> &'static str {
        match self {
            Self::RegOnboardAbprop => "/v2/reg_onboard_abprop",
            Self::Exist => "/v2/exist",
            Self::Code => "/v2/code",
            Self::AcVerifyRequest => "/v2/acverify_request",
            Self::AcVerify => "/v2/acverify",
            Self::Register => "/v2/register",
            Self::Security => "/v2/security",
            Self::Autoconf => "/v2/autoconf",
            Self::AutoconfVerifier => "/v2/autoconf_verifier",
            Self::DeviceLogoutSend => "/v2/device_logout_send",
            Self::DeviceLogoutFetch => "/v2/device_logout_fetch",
            Self::ClientLog => "/v2/client_log",
            Self::PrePnClientLog => "/v2/pre_pn_client_log",
            Self::Consent => "/v2/consent",
            Self::Challenge => "/v2/challenge",
            Self::PasskeyAuth => "/v2/passkey_auth",
            Self::Wfs => "/v2/wfs",
        }
    }
}

/// Exact APK identity and registration base associated with a wire catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RegistrationWireProfile {
    /// Android version name.
    pub version_name: &'static str,
    /// Android version code.
    pub version_code: u64,
    /// SHA-256 of the exact APK.
    pub apk_sha256: &'static str,
    /// Registration origin assembled by the client.
    pub base_url: &'static str,
}

impl RegistrationWireProfile {
    /// Assemble an endpoint URL without normalizing or guessing path segments.
    pub fn endpoint_url(self, endpoint: RegistrationEndpoint) -> String {
        format!("{}{}", self.base_url, endpoint.path())
    }

    /// Build HTTP request parts for the given ordered form.
    pub fn request(
        self,
        endpoint: RegistrationEndpoint,
        form: &RegistrationForm,
    ) -> Result<EncodedRegistrationRequest> {
        let body = form.encode()?;
        Ok(EncodedRegistrationRequest {
            url: self.endpoint_url(endpoint),
            body,
        })
    }
}

/// Signed comparison profile whose registration assembly was inspected.
pub const WHATSAPP_2_26_37_70: RegistrationWireProfile = RegistrationWireProfile {
    version_name: "2.26.37.70",
    version_code: 263_707_033,
    apk_sha256: "b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732",
    base_url: "https://v.whatsapp.net",
};

#[derive(Clone, Copy)]
enum ValueEncoding {
    Encode,
    AlreadyEncoded,
}

struct Field {
    name: String,
    value: Zeroizing<String>,
    encoding: ValueEncoding,
}

/// Ordered `application/x-www-form-urlencoded` registration fields.
///
/// Insertion order is retained because the observed Android builder uses an
/// insertion-ordered map. Values are zeroized on drop and omitted from
/// `Debug` output.
#[derive(Default)]
pub struct RegistrationForm {
    fields: Vec<Field>,
}

impl fmt::Debug for RegistrationForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationForm")
            .field("field_count", &self.fields.len())
            .finish_non_exhaustive()
    }
}

impl RegistrationForm {
    /// Create an empty form.
    pub const fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Add a field whose name and value will be UTF-8 form encoded.
    pub fn push(&mut self, name: impl Into<String>, value: impl Into<String>) -> Result<&mut Self> {
        self.push_inner(name.into(), value.into(), ValueEncoding::Encode)
    }

    /// Add a value already encoded for a form component.
    ///
    /// The name is still encoded. The value is accepted only if it consists
    /// of Java-`URLEncoder` safe bytes, `+`, or complete `%HH` escapes; raw
    /// `&` and `=` can therefore never inject another field.
    pub fn push_pre_encoded(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<&mut Self> {
        let value = value.into();
        validate_pre_encoded(&value)?;
        self.push_inner(name.into(), value, ValueEncoding::AlreadyEncoded)
    }

    fn push_inner(
        &mut self,
        name: String,
        value: String,
        encoding: ValueEncoding,
    ) -> Result<&mut Self> {
        if self.fields.len() >= MAX_FORM_FIELDS {
            return Err(Error::ResourceExhausted {
                resource: "registration form fields",
                limit: MAX_FORM_FIELDS as u64,
            });
        }
        if name.is_empty() || name.len() > MAX_FIELD_NAME_BYTES {
            return Err(Error::invalid(
                "registration_field_name",
                format!("must contain 1..={MAX_FIELD_NAME_BYTES} UTF-8 bytes"),
            ));
        }
        if value.len() > MAX_FIELD_VALUE_BYTES {
            return Err(Error::ResourceExhausted {
                resource: "registration field value bytes",
                limit: MAX_FIELD_VALUE_BYTES as u64,
            });
        }
        if self.fields.iter().any(|field| field.name == name) {
            return Err(Error::invalid("registration_field_name", "duplicate field"));
        }
        self.fields.push(Field {
            name,
            value: Zeroizing::new(value),
            encoding,
        });
        Ok(self)
    }

    /// Number of fields in insertion order.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Whether the form contains no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Encode the complete form body, zeroizing it when dropped.
    pub fn encode(&self) -> Result<Zeroizing<Vec<u8>>> {
        let mut body = Zeroizing::new(Vec::new());
        for (index, field) in self.fields.iter().enumerate() {
            if index != 0 {
                body.push(b'&');
            }
            encode_component(field.name.as_bytes(), &mut body);
            body.push(b'=');
            match field.encoding {
                ValueEncoding::Encode => encode_component(field.value.as_bytes(), &mut body),
                ValueEncoding::AlreadyEncoded => body.extend_from_slice(field.value.as_bytes()),
            }
            if body.len() > MAX_FORM_BODY_BYTES {
                return Err(Error::ResourceExhausted {
                    resource: "registration form body bytes",
                    limit: MAX_FORM_BODY_BYTES as u64,
                });
            }
        }
        Ok(body)
    }
}

/// Secret-bearing request parts ready for a reviewed HTTPS backend.
pub struct EncodedRegistrationRequest {
    url: String,
    body: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for EncodedRegistrationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncodedRegistrationRequest")
            .field("url", &self.url)
            .field("content_type", &self.content_type())
            .field("content_length", &self.body.len())
            .field("body", &"<redacted>")
            .finish()
    }
}

impl EncodedRegistrationRequest {
    /// HTTP method observed for registration requests.
    pub const fn method(&self) -> &'static str {
        "POST"
    }

    /// Full request URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// HTTP content type.
    pub const fn content_type(&self) -> &'static str {
        "application/x-www-form-urlencoded"
    }

    /// Exact body length for `Content-Length`.
    pub fn content_length(&self) -> usize {
        self.body.len()
    }

    /// Encoded body. Callers must not log it.
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

fn encode_component(input: &[u8], output: &mut Vec<u8>) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &byte in input {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' => {
                output.push(byte)
            }
            b' ' => output.push(b'+'),
            _ => {
                output.push(b'%');
                output.push(HEX[usize::from(byte >> 4)]);
                output.push(HEX[usize::from(byte & 0x0f)]);
            }
        }
    }
}

fn validate_pre_encoded(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let valid = match bytes[index] {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' | b'+' => true,
            b'%' if index + 2 < bytes.len()
                && bytes[index + 1].is_ascii_hexdigit()
                && bytes[index + 2].is_ascii_hexdigit() =>
            {
                index += 2;
                true
            }
            _ => false,
        };
        if !valid {
            return Err(Error::invalid(
                "registration_field_value",
                "invalid pre-encoded form component",
            ));
        }
        index += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn catalog_matches_committed_comparison_evidence() {
        let evidence: Value = serde_json::from_str(include_str!(
            "../../fixtures/comparisons/whatsapp-2.26.37.70.json"
        ))
        .unwrap();
        let reference = &evidence["reference_client"];
        assert_eq!(reference["version_name"], WHATSAPP_2_26_37_70.version_name);
        assert_eq!(reference["version_code"], WHATSAPP_2_26_37_70.version_code);
        assert_eq!(reference["apk_sha256"], WHATSAPP_2_26_37_70.apk_sha256);
        assert_eq!(
            evidence["observations"]["registration"]["base_url"],
            WHATSAPP_2_26_37_70.base_url
        );
        let paths: Vec<_> = RegistrationEndpoint::ALL
            .iter()
            .map(|endpoint| endpoint.path())
            .collect();
        assert_eq!(
            evidence["observations"]["registration"]["paths"],
            serde_json::json!(paths)
        );
    }

    #[test]
    fn assembles_exact_url_and_request_metadata() {
        let mut form = RegistrationForm::new();
        form.push("cc", "1")
            .unwrap()
            .push("in", "5551234567")
            .unwrap();
        let request = WHATSAPP_2_26_37_70
            .request(RegistrationEndpoint::Code, &form)
            .unwrap();
        assert_eq!(request.method(), "POST");
        assert_eq!(request.url(), "https://v.whatsapp.net/v2/code");
        assert_eq!(request.content_type(), "application/x-www-form-urlencoded");
        assert_eq!(request.body(), b"cc=1&in=5551234567");
        assert_eq!(request.content_length(), request.body().len());
        assert!(!format!("{request:?}").contains("5551234567"));
    }

    #[test]
    fn encoding_matches_java_form_rules() {
        let mut form = RegistrationForm::new();
        form.push("a b", "x+y&é*.-_").unwrap();
        assert_eq!(form.encode().unwrap().as_slice(), b"a+b=x%2By%26%C3%A9*.-_");
    }

    #[test]
    fn pre_encoded_values_are_strict_and_duplicates_fail() {
        let mut form = RegistrationForm::new();
        form.push_pre_encoded("token", "a+b%2Fc").unwrap();
        assert_eq!(form.encode().unwrap().as_slice(), b"token=a+b%2Fc");
        assert!(form.push("token", "other").is_err());
        assert!(
            RegistrationForm::new()
                .push_pre_encoded("bad", "x&injected=true")
                .is_err()
        );
        assert!(
            RegistrationForm::new()
                .push_pre_encoded("bad", "%A")
                .is_err()
        );
    }

    #[test]
    fn local_form_limits_are_enforced() {
        let mut form = RegistrationForm::new();
        assert!(form.push("", "value").is_err());
        assert!(
            form.push("name", "x".repeat(MAX_FIELD_VALUE_BYTES + 1))
                .is_err()
        );
        for index in 0..MAX_FORM_FIELDS {
            form.push(format!("f{index}"), "x").unwrap();
        }
        assert!(form.push("one_too_many", "x").is_err());
    }
}
