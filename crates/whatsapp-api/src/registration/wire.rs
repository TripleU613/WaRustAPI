//! Version-scoped registration request construction.
//!
//! These facts were statically observed in the separately hashed, signed
//! WhatsApp 2.26.37.70 comparison build. They are deliberately not installed
//! into the active 2.26.36.75 [`crate::ProtocolProfile`]. This module performs
//! no I/O and cannot request an OTP.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::model::phone::PhoneNumber;
use crate::{Error, Result};

/// Maximum fields accepted in one locally constructed registration form.
pub const MAX_FORM_FIELDS: usize = 256;
/// Maximum UTF-8 bytes accepted in one field name.
pub const MAX_FIELD_NAME_BYTES: usize = 128;
/// Maximum UTF-8 bytes accepted in one field value.
pub const MAX_FIELD_VALUE_BYTES: usize = 1 << 20;
/// Maximum encoded registration body size.
pub const MAX_FORM_BODY_BYTES: usize = 2 << 20;
/// Maximum JSON response body accepted from a registration transport.
pub const MAX_RESPONSE_BODY_BYTES: usize = 1 << 20;
/// Maximum bytes accepted in a secret-bearing response string.
pub const MAX_RESPONSE_SECRET_BYTES: usize = 256 << 10;
/// Maximum bytes accepted in one public registration key component.
pub const MAX_KEY_COMPONENT_BYTES: usize = 4 << 10;
/// Maximum raw bytes accepted in a binary registration parameter.
pub const MAX_BINARY_PARAMETER_BYTES: usize = 64 << 10;
/// Key-type byte emitted by the inspected comparison build.
pub const COMPARISON_SIGNAL_KEY_TYPE: u8 = 0x05;

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

    /// Add raw additional-parameter bytes using the comparison APK's `Hvq.A06`
    /// path: RFC 3986 percent encoding, then exempt from a second encoding pass.
    pub fn push_bytes(&mut self, name: impl Into<String>, value: &[u8]) -> Result<&mut Self> {
        validate_binary_parameter("additional_parameter", value, true)?;
        self.push_pre_encoded(name, percent_encode_binary(value))
    }

    /// Add a value already encoded for a form component.
    ///
    /// The name is still encoded. The value is accepted only if it consists
    /// of unreserved form bytes, `+`, or complete `%HH` escapes; raw `&` and
    /// `=` can therefore never inject another field.
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

    fn push_batch(
        &mut self,
        fields: Vec<(&'static str, String, ValueEncoding)>,
    ) -> Result<&mut Self> {
        if self.fields.len().saturating_add(fields.len()) > MAX_FORM_FIELDS {
            return Err(Error::ResourceExhausted {
                resource: "registration form fields",
                limit: MAX_FORM_FIELDS as u64,
            });
        }
        for (index, (name, value, encoding)) in fields.iter().enumerate() {
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
            if matches!(encoding, ValueEncoding::AlreadyEncoded) {
                validate_pre_encoded(value)?;
            }
            if self.fields.iter().any(|field| field.name == *name)
                || fields[..index]
                    .iter()
                    .any(|(previous, _, _)| previous == name)
            {
                return Err(Error::invalid("registration_field_name", "duplicate field"));
            }
        }
        self.fields
            .extend(fields.into_iter().map(|(name, value, encoding)| Field {
                name: name.to_owned(),
                value: Zeroizing::new(value),
                encoding,
            }));
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

/// Common request parameters assembled by the comparison client's
/// registration bridge before endpoint-specific fields are added.
pub struct RegistrationCommonParams {
    phone: PhoneNumber,
    language: String,
    locale: String,
    fdid: Zeroizing<String>,
    experiment_id: [u8; 16],
    access_session_id: Option<[u8; 16]>,
    device_id: Zeroizing<Vec<u8>>,
    backup_token: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for RegistrationCommonParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationCommonParams")
            .field("phone", &self.phone)
            .field("language", &self.language)
            .field("locale", &self.locale)
            .field("has_fdid", &!self.fdid.is_empty())
            .field("has_access_session_id", &self.access_session_id.is_some())
            .field("device_id_length", &self.device_id.len())
            .field("backup_token_length", &self.backup_token.len())
            .finish()
    }
}

impl RegistrationCommonParams {
    /// Validate common comparison-build parameters. UUIDs are accepted as
    /// their canonical 16-byte representation so string parsing cannot alter
    /// their byte order.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        phone: PhoneNumber,
        language: impl Into<String>,
        locale: impl Into<String>,
        fdid: impl Into<String>,
        experiment_id: [u8; 16],
        access_session_id: Option<[u8; 16]>,
        device_id: Vec<u8>,
        backup_token: Vec<u8>,
    ) -> Result<Self> {
        let language = language.into();
        let locale = locale.into();
        let fdid = fdid.into();
        for (field, value) in [
            ("language", language.as_str()),
            ("locale", locale.as_str()),
            ("fdid", fdid.as_str()),
        ] {
            if value.is_empty() || value.len() > MAX_FIELD_VALUE_BYTES {
                return Err(Error::invalid(
                    field,
                    format!("must contain 1..={MAX_FIELD_VALUE_BYTES} UTF-8 bytes"),
                ));
            }
        }
        for (field, value) in [
            ("device_id", device_id.as_slice()),
            ("backup_token", backup_token.as_slice()),
        ] {
            if value.is_empty() || value.len() > MAX_BINARY_PARAMETER_BYTES {
                return Err(Error::invalid(
                    field,
                    format!("must contain 1..={MAX_BINARY_PARAMETER_BYTES} bytes"),
                ));
            }
        }
        Ok(Self {
            phone,
            language,
            locale,
            fdid: Zeroizing::new(fdid),
            experiment_id,
            access_session_id,
            device_id: Zeroizing::new(device_id),
            backup_token: Zeroizing::new(backup_token),
        })
    }

    /// Append common fields atomically in the order used by the comparison
    /// client's `A0T`, `A0U`, and `A0V` helpers.
    pub fn append_to_form<'a>(
        &self,
        form: &'a mut RegistrationForm,
    ) -> Result<&'a mut RegistrationForm> {
        let mut fields = vec![
            (
                "cc",
                self.phone.country_code().to_string(),
                ValueEncoding::Encode,
            ),
            (
                "in",
                self.phone.national_number().to_owned(),
                ValueEncoding::Encode,
            ),
            ("lg", self.language.clone(), ValueEncoding::Encode),
            ("lc", self.locale.clone(), ValueEncoding::Encode),
            ("fdid", self.fdid.to_string(), ValueEncoding::Encode),
            (
                "expid",
                base64_url_no_pad(&self.experiment_id),
                ValueEncoding::Encode,
            ),
        ];
        if let Some(access_session_id) = self.access_session_id {
            fields.push((
                "access_session_id",
                base64_url_no_pad(&access_session_id),
                ValueEncoding::Encode,
            ));
        }
        fields.extend([
            (
                "id",
                percent_encode_binary(&self.device_id),
                ValueEncoding::AlreadyEncoded,
            ),
            (
                "backup_token",
                percent_encode_binary(&self.backup_token),
                ValueEncoding::AlreadyEncoded,
            ),
        ]);
        form.push_batch(fields)
    }
}

/// Endpoint-specific parameters observed in the comparison client's
/// generate-auth-code request (`/v2/code`).
pub struct RegistrationCodeParams {
    token: Zeroizing<String>,
    method: String,
    context: Option<Zeroizing<String>>,
    clicked_education_link: Option<bool>,
    manage_call_permission: Option<bool>,
    call_log_permission: Option<bool>,
    client_start_message: Option<Zeroizing<Vec<u8>>>,
    advertising_id: Option<Zeroizing<String>>,
    login: Option<Zeroizing<String>>,
    registration_type: Option<i32>,
}

impl fmt::Debug for RegistrationCodeParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationCodeParams")
            .field("method", &self.method)
            .field("has_context", &self.context.is_some())
            .field("clicked_education_link", &self.clicked_education_link)
            .field("manage_call_permission", &self.manage_call_permission)
            .field("call_log_permission", &self.call_log_permission)
            .field(
                "client_start_message_length",
                &self.client_start_message.as_ref().map(|value| value.len()),
            )
            .field("has_advertising_id", &self.advertising_id.is_some())
            .field("has_login", &self.login.is_some())
            .field("registration_type", &self.registration_type)
            .finish()
    }
}

impl RegistrationCodeParams {
    /// Create the required portion of a generate-auth-code request.
    pub fn new(token: impl Into<String>, method: impl Into<String>) -> Result<Self> {
        let token = token.into();
        let method = method.into();
        validate_parameter_text("token", &token, false)?;
        validate_parameter_text("method", &method, false)?;
        Ok(Self {
            token: Zeroizing::new(token),
            method,
            context: None,
            clicked_education_link: None,
            manage_call_permission: None,
            call_log_permission: None,
            client_start_message: None,
            advertising_id: None,
            login: None,
            registration_type: None,
        })
    }

    /// Derive the comparison-build token from exact APK material and create
    /// the required portion of a generate-auth-code request.
    ///
    /// This is scoped to [`WHATSAPP_2_26_37_70`]; callers must not reuse the
    /// derivation with another APK version without fresh static evidence.
    pub fn from_comparison_token_material(
        phone: &PhoneNumber,
        material: super::token::ComparisonTokenMaterial<'_>,
        method: impl Into<String>,
    ) -> Result<Self> {
        let token = super::token::derive_comparison_registration_token(phone, material)?;
        Self::new(token.as_str(), method)
    }

    /// Set the optional request context.
    pub fn with_context(mut self, context: impl Into<String>) -> Result<Self> {
        let context = context.into();
        validate_parameter_text("context", &context, true)?;
        self.context = Some(Zeroizing::new(context));
        Ok(self)
    }

    /// Set the three optional boolean fields. `None` means omit the field,
    /// matching the comparison helper's third integer state.
    pub fn with_permissions(
        mut self,
        clicked_education_link: Option<bool>,
        manage_call_permission: Option<bool>,
        call_log_permission: Option<bool>,
    ) -> Self {
        self.clicked_education_link = clicked_education_link;
        self.manage_call_permission = manage_call_permission;
        self.call_log_permission = call_log_permission;
        self
    }

    /// Set the optional binary client-start message.
    pub fn with_client_start_message(mut self, value: Vec<u8>) -> Result<Self> {
        validate_binary_parameter("client_start_message", &value, true)?;
        self.client_start_message = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the optional advertising identifier.
    pub fn with_advertising_id(mut self, value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_parameter_text("advertising_id", &value, true)?;
        self.advertising_id = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the optional prior login value.
    pub fn with_login(mut self, value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_parameter_text("login", &value, true)?;
        self.login = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the numeric registration-type value carried by the comparison
    /// client's internal enum.
    pub const fn with_registration_type(mut self, value: i32) -> Self {
        self.registration_type = Some(value);
        self
    }

    /// Append endpoint-specific fields atomically in observed request order.
    pub fn append_to_form<'a>(
        &self,
        form: &'a mut RegistrationForm,
    ) -> Result<&'a mut RegistrationForm> {
        let mut fields = vec![
            ("token", self.token.to_string(), ValueEncoding::Encode),
            ("method", self.method.clone(), ValueEncoding::Encode),
        ];
        if let Some(context) = &self.context {
            fields.push(("context", context.to_string(), ValueEncoding::Encode));
        }
        for (name, value) in [
            ("clicked_education_link", self.clicked_education_link),
            ("manage_call_permission", self.manage_call_permission),
            ("call_log_permission", self.call_log_permission),
        ] {
            if let Some(value) = value {
                fields.push((name, value.to_string(), ValueEncoding::Encode));
            }
        }
        if let Some(value) = &self.client_start_message {
            fields.push((
                "client_start_message",
                base64_url_no_pad(value),
                ValueEncoding::Encode,
            ));
        }
        if let Some(value) = &self.advertising_id {
            fields.push(("advertising_id", value.to_string(), ValueEncoding::Encode));
        }
        if let Some(value) = &self.login {
            fields.push(("login", value.to_string(), ValueEncoding::Encode));
        }
        if let Some(value) = self.registration_type {
            fields.push(("type", value.to_string(), ValueEncoding::Encode));
        }
        form.push_batch(fields)
    }
}

/// Assemble the observed clear-form generate-auth-code request without
/// sending it. The public key bundle is optional because the comparison
/// client logs missing E2E keys and still issues this request.
pub fn assemble_code_request(
    common: &RegistrationCommonParams,
    code: &RegistrationCodeParams,
    key_bundle: Option<&RegistrationKeyBundle>,
) -> Result<EncodedRegistrationRequest> {
    let mut form = RegistrationForm::new();
    common.append_to_form(&mut form)?;
    code.append_to_form(&mut form)?;
    if let Some(key_bundle) = key_bundle {
        key_bundle.append_to_form(&mut form)?;
    }
    WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Code, &form)
}

/// Endpoint-specific parameters observed in the comparison client's
/// phone-number completion request (`/v2/register`).
pub struct RegistrationCompleteParams {
    code: Zeroizing<String>,
    auth_response: Option<Zeroizing<Vec<u8>>>,
    context: Option<Zeroizing<String>>,
    method: Option<String>,
    advertising_id: Option<Zeroizing<String>>,
    login: Option<Zeroizing<String>>,
    registration_type: Option<i32>,
}

impl fmt::Debug for RegistrationCompleteParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationCompleteParams")
            .field(
                "auth_response_length",
                &self.auth_response.as_ref().map(|value| value.len()),
            )
            .field("has_context", &self.context.is_some())
            .field("method", &self.method)
            .field("has_advertising_id", &self.advertising_id.is_some())
            .field("has_login", &self.login.is_some())
            .field("registration_type", &self.registration_type)
            .finish()
    }
}

impl RegistrationCompleteParams {
    /// Create a completion request with its required verification code.
    pub fn new(code: impl Into<String>) -> Result<Self> {
        let code = code.into();
        validate_parameter_text("code", &code, false)?;
        Ok(Self {
            code: Zeroizing::new(code),
            auth_response: None,
            context: None,
            method: None,
            advertising_id: None,
            login: None,
            registration_type: None,
        })
    }

    /// Set the optional binary authentication response.
    pub fn with_auth_response(mut self, value: Vec<u8>) -> Result<Self> {
        validate_binary_parameter("auth_response", &value, true)?;
        self.auth_response = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the optional request context.
    pub fn with_context(mut self, value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_parameter_text("context", &value, true)?;
        self.context = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the optional verification method.
    pub fn with_method(mut self, value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_parameter_text("method", &value, true)?;
        self.method = Some(value);
        Ok(self)
    }

    /// Set the optional advertising identifier.
    pub fn with_advertising_id(mut self, value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_parameter_text("advertising_id", &value, true)?;
        self.advertising_id = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the optional prior login value.
    pub fn with_login(mut self, value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_parameter_text("login", &value, true)?;
        self.login = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the numeric registration-type enum value.
    pub const fn with_registration_type(mut self, value: i32) -> Self {
        self.registration_type = Some(value);
        self
    }

    /// Append endpoint-specific fields atomically in observed request order.
    pub fn append_to_form<'a>(
        &self,
        form: &'a mut RegistrationForm,
    ) -> Result<&'a mut RegistrationForm> {
        let mut fields = vec![("code", self.code.to_string(), ValueEncoding::Encode)];
        if let Some(value) = &self.auth_response {
            fields.push((
                "auth_response",
                base64_url_no_pad(value),
                ValueEncoding::Encode,
            ));
        }
        if let Some(value) = &self.context {
            fields.push(("context", value.to_string(), ValueEncoding::Encode));
        }
        if let Some(value) = &self.method {
            fields.push(("method", value.clone(), ValueEncoding::Encode));
        }
        if let Some(value) = &self.advertising_id {
            fields.push(("advertising_id", value.to_string(), ValueEncoding::Encode));
        }
        if let Some(value) = &self.login {
            fields.push(("login", value.to_string(), ValueEncoding::Encode));
        }
        if let Some(value) = self.registration_type {
            fields.push(("type", value.to_string(), ValueEncoding::Encode));
        }
        form.push_batch(fields)
    }
}

/// Assemble the observed clear-form phone-number completion request without
/// sending it.
pub fn assemble_complete_request(
    common: &RegistrationCommonParams,
    complete: &RegistrationCompleteParams,
    key_bundle: Option<&RegistrationKeyBundle>,
) -> Result<EncodedRegistrationRequest> {
    let mut form = RegistrationForm::new();
    common.append_to_form(&mut form)?;
    complete.append_to_form(&mut form)?;
    if let Some(key_bundle) = key_bundle {
        key_bundle.append_to_form(&mut form)?;
    }
    WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Register, &form)
}

/// Endpoint-specific parameters observed in the comparison client's account
/// security-code request (`/v2/security`).
pub struct RegistrationSecurityParams {
    code: Option<Zeroizing<String>>,
    reset: Option<Zeroizing<String>>,
    wipe_token: Option<Zeroizing<Vec<u8>>>,
    advertising_id: Option<Zeroizing<String>>,
}

impl fmt::Debug for RegistrationSecurityParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationSecurityParams")
            .field("has_code", &self.code.is_some())
            .field("has_reset", &self.reset.is_some())
            .field(
                "wipe_token_length",
                &self.wipe_token.as_ref().map(|value| value.len()),
            )
            .field("has_advertising_id", &self.advertising_id.is_some())
            .finish()
    }
}

impl RegistrationSecurityParams {
    /// Build a security-code verification request.
    pub fn verify(code: impl Into<String>) -> Result<Self> {
        let code = code.into();
        validate_parameter_text("code", &code, false)?;
        Ok(Self {
            code: Some(Zeroizing::new(code)),
            reset: None,
            wipe_token: None,
            advertising_id: None,
        })
    }

    /// Build a security-code reset request.
    pub fn reset(reset: impl Into<String>) -> Result<Self> {
        let reset = reset.into();
        validate_parameter_text("reset", &reset, false)?;
        Ok(Self {
            code: None,
            reset: Some(Zeroizing::new(reset)),
            wipe_token: None,
            advertising_id: None,
        })
    }

    /// Set the optional raw wipe token. The comparison helper percent-encodes
    /// its bytes and marks the value pre-encoded.
    pub fn with_wipe_token(mut self, value: Vec<u8>) -> Result<Self> {
        validate_binary_parameter("wipe_token", &value, false)?;
        self.wipe_token = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Set the optional advertising identifier.
    pub fn with_advertising_id(mut self, value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_parameter_text("advertising_id", &value, true)?;
        self.advertising_id = Some(Zeroizing::new(value));
        Ok(self)
    }

    /// Append endpoint-specific fields atomically in observed request order.
    pub fn append_to_form<'a>(
        &self,
        form: &'a mut RegistrationForm,
    ) -> Result<&'a mut RegistrationForm> {
        let mut fields = Vec::with_capacity(4);
        if let Some(value) = &self.code {
            fields.push(("code", value.to_string(), ValueEncoding::Encode));
        }
        if let Some(value) = &self.reset {
            fields.push(("reset", value.to_string(), ValueEncoding::Encode));
        }
        if let Some(value) = &self.wipe_token {
            fields.push((
                "wipe_token",
                percent_encode_binary(value),
                ValueEncoding::AlreadyEncoded,
            ));
        }
        if let Some(value) = &self.advertising_id {
            fields.push(("advertising_id", value.to_string(), ValueEncoding::Encode));
        }
        form.push_batch(fields)
    }
}

/// Assemble the observed clear-form account security-code request without
/// sending it.
pub fn assemble_security_request(
    common: &RegistrationCommonParams,
    security: &RegistrationSecurityParams,
    key_bundle: Option<&RegistrationKeyBundle>,
) -> Result<EncodedRegistrationRequest> {
    let mut form = RegistrationForm::new();
    common.append_to_form(&mut form)?;
    security.append_to_form(&mut form)?;
    if let Some(key_bundle) = key_bundle {
        key_bundle.append_to_form(&mut form)?;
    }
    WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Security, &form)
}

/// Public half of the pre-registration Noise/Signal key bundle observed in
/// the comparison client.
///
/// This type does not generate or sign keys. A reviewed Signal backend must
/// create and durably persist the corresponding private material before this
/// public bundle is constructed.
pub struct RegistrationKeyBundle {
    auth_key: Zeroizing<Vec<u8>>,
    identity_key: Zeroizing<Vec<u8>>,
    registration_id: u32,
    signed_pre_key_id: u32,
    signed_pre_key_value: Zeroizing<Vec<u8>>,
    signed_pre_key_signature: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for RegistrationKeyBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationKeyBundle")
            .field("auth_key_length", &self.auth_key.len())
            .field("identity_key_length", &self.identity_key.len())
            .field("registration_id", &self.registration_id)
            .field("signed_pre_key_id", &self.signed_pre_key_id)
            .field("signed_pre_key_length", &self.signed_pre_key_value.len())
            .field(
                "signed_pre_key_signature_length",
                &self.signed_pre_key_signature.len(),
            )
            .finish()
    }
}

impl RegistrationKeyBundle {
    /// Validate already-generated public registration material.
    pub fn new(
        auth_key: [u8; 32],
        identity_key: Vec<u8>,
        registration_id: u32,
        signed_pre_key_id: u32,
        signed_pre_key_value: Vec<u8>,
        signed_pre_key_signature: Vec<u8>,
    ) -> Result<Self> {
        if signed_pre_key_id > 0x00ff_ffff {
            return Err(Error::invalid(
                "signed_pre_key_id",
                "must fit the observed three-byte encoding",
            ));
        }
        for (field, value) in [
            ("identity_key", identity_key.as_slice()),
            ("signed_pre_key_value", signed_pre_key_value.as_slice()),
            (
                "signed_pre_key_signature",
                signed_pre_key_signature.as_slice(),
            ),
        ] {
            if value.is_empty() || value.len() > MAX_KEY_COMPONENT_BYTES {
                return Err(Error::invalid(
                    field,
                    format!("must contain 1..={MAX_KEY_COMPONENT_BYTES} bytes"),
                ));
            }
        }
        Ok(Self {
            auth_key: Zeroizing::new(auth_key.to_vec()),
            identity_key: Zeroizing::new(identity_key),
            registration_id,
            signed_pre_key_id,
            signed_pre_key_value: Zeroizing::new(signed_pre_key_value),
            signed_pre_key_signature: Zeroizing::new(signed_pre_key_signature),
        })
    }

    /// Append the seven comparison-build form parameters atomically and in
    /// the order used by the inspected client.
    pub fn append_to_form<'a>(
        &self,
        form: &'a mut RegistrationForm,
    ) -> Result<&'a mut RegistrationForm> {
        let registration_id = self.registration_id.to_be_bytes();
        let signed_pre_key_id = [
            (self.signed_pre_key_id >> 16) as u8,
            (self.signed_pre_key_id >> 8) as u8,
            self.signed_pre_key_id as u8,
        ];
        form.push_batch(vec![
            (
                "authkey",
                base64_url_no_pad(&self.auth_key),
                ValueEncoding::Encode,
            ),
            (
                "e_ident",
                base64_url_no_pad(&self.identity_key),
                ValueEncoding::Encode,
            ),
            (
                "e_keytype",
                base64_url_no_pad(&[COMPARISON_SIGNAL_KEY_TYPE]),
                ValueEncoding::Encode,
            ),
            (
                "e_regid",
                base64_url_no_pad(&registration_id),
                ValueEncoding::Encode,
            ),
            (
                "e_skey_id",
                base64_url_no_pad(&signed_pre_key_id),
                ValueEncoding::Encode,
            ),
            (
                "e_skey_val",
                base64_url_no_pad(&self.signed_pre_key_value),
                ValueEncoding::Encode,
            ),
            (
                "e_skey_sig",
                base64_url_no_pad(&self.signed_pre_key_signature),
                ValueEncoding::Encode,
            ),
        ])
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
    /// Append explicitly selected code fields without replacing saved inputs.
    /// This is an offline diagnostic transform, not a persisted profile change.
    pub fn with_additional_code_fields(
        mut self,
        fields: &super::request::EndpointFields,
    ) -> Result<Self> {
        if self.url != WHATSAPP_2_26_37_70.endpoint_url(RegistrationEndpoint::Code)
            || self.body.starts_with(b"ENC=")
            || !fields.register.is_empty()
            || !fields.register_bytes.is_empty()
            || !fields.security.is_empty()
            || !fields.security_bytes.is_empty()
        {
            return Err(Error::invalid(
                "registration_patch",
                "requires clear comparison code request and code-only fields",
            ));
        }
        let mut extra = RegistrationForm::new();
        super::request_config::append_fields(&mut extra, &fields.code)?;
        super::request_config::append_byte_fields(&mut extra, &fields.code_bytes)?;
        let encoded = extra.encode()?;
        if encoded.is_empty() {
            return Err(Error::invalid("registration_patch", "empty field patch"));
        }
        let existing_count = if self.body.is_empty() {
            0
        } else {
            self.body.split(|b| *b == b'&').count()
        };
        if existing_count.saturating_add(extra.fields.len()) > MAX_FORM_FIELDS {
            return Err(Error::ResourceExhausted {
                resource: "registration form fields",
                limit: MAX_FORM_FIELDS as u64,
            });
        }
        if self
            .body
            .len()
            .saturating_add(encoded.len())
            .saturating_add(usize::from(!self.body.is_empty()))
            > MAX_FORM_BODY_BYTES
        {
            return Err(Error::ResourceExhausted {
                resource: "registration form body",
                limit: MAX_FORM_BODY_BYTES as u64,
            });
        }
        for field in encoded.split(|b| *b == b'&') {
            let name = field.split(|b| *b == b'=').next().unwrap_or_default();
            if self
                .body
                .split(|b| *b == b'&')
                .any(|existing| existing.split(|b| *b == b'=').next() == Some(name))
            {
                return Err(Error::invalid(
                    "registration_patch",
                    "field already exists in saved request",
                ));
            }
        }
        if !self.body.is_empty() {
            self.body.push(b'&');
        }
        self.body.extend_from_slice(&encoded);
        Ok(self)
    }

    /// Observed body-builder fallback when no attestation signature is available.
    /// Sends an explicitly empty H, not a forged signature or Authorization header.
    #[cfg(feature = "experimental-signal-crypto")]
    pub fn encrypt_comparison_without_signature(self) -> Result<Self> {
        let mut request = self.encrypt_comparison()?;
        request.body.extend_from_slice(b"&H=");
        Ok(request)
    }

    /// Wrap a comparison request in the APK's `ENC` envelope, using a fresh
    /// ephemeral encryption key. Does not provide `H` or Authorization
    /// attestation, change account identity, or contact the server.
    /// Explicit opt-in: existing saved requests keep their original wire bytes.
    #[cfg(feature = "experimental-signal-crypto")]
    pub fn encrypt_comparison(mut self) -> Result<Self> {
        if !self
            .url
            .starts_with(&format!("{}/", WHATSAPP_2_26_37_70.base_url))
            || self.body.starts_with(b"ENC=")
        {
            return Err(Error::invalid(
                "registration_envelope",
                "requires an unwrapped comparison request",
            ));
        }
        let sealed = super::envelope::seal(&self.body)?;
        let encoded = Zeroizing::new(base64_url_no_pad(&sealed));
        let mut form = RegistrationForm::new();
        form.push("ENC", encoded.as_str())?;
        self.body = form.encode()?;
        Ok(self)
    }

    /// Metadata-only troubleshooting view. Field values never appear.
    pub fn diagnostic_summary(&self) -> serde_json::Value {
        let fields: Vec<_> = self
            .body
            .split(|b| *b == b'&')
            .filter_map(|field| {
                let key = field.split(|b| *b == b'=').next()?;
                std::str::from_utf8(key).ok()
            })
            .collect();
        serde_json::json!({"method": self.method(), "url": self.url(), "content_type": self.content_type(), "body_bytes": self.content_length(), "fields": fields})
    }
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

/// Future returned by a registration HTTP transport.
pub type RegistrationTransportFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RegistrationHttpResponse>> + Send + 'a>>;

/// Narrow HTTPS boundary used by a future profile-specific registration
/// driver. Implementations are responsible for certificate validation,
/// deadlines and redirect policy. They must not log request or response
/// bodies.
pub trait RegistrationTransport: Send + Sync + fmt::Debug {
    /// Send exactly one constructed request.
    fn send<'a>(&'a self, request: EncodedRegistrationRequest) -> RegistrationTransportFuture<'a>;
}

/// Lab-only HTTPS transport selected by ADR 0003.
///
/// This is feature-gated because the registration protocol remains experimental;
/// it uses normal WebPKI certificate and hostname validation, a
/// compiled Mozilla root set, HTTPS-only URLs, no redirects, no proxies, no
/// transparent decompression, no automatic retries, and a whole-request
/// deadline. It never logs request or response bodies.
#[cfg(feature = "experimental-registration-https")]
#[derive(Clone)]
pub struct ExperimentalHttpsTransport {
    client: reqwest::Client,
    timeout: Duration,
    max_response_bytes: usize,
}

#[cfg(feature = "experimental-registration-https")]
impl fmt::Debug for ExperimentalHttpsTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExperimentalHttpsTransport")
            .field("timeout", &self.timeout)
            .field("redirects", &"disabled")
            .field("automatic_retries", &"disabled")
            .finish()
    }
}

#[cfg(feature = "experimental-registration-https")]
impl ExperimentalHttpsTransport {
    /// Maximum configurable whole-request timeout.
    pub const MAX_TIMEOUT: Duration = Duration::from_secs(5 * 60);

    /// Build the constrained lab transport with an explicit ring
    /// provider. No process-global TLS provider is installed.
    pub fn new(timeout: Duration) -> Result<Self> {
        Self::build(
            timeout,
            timeout.min(Duration::from_secs(30)),
            MAX_RESPONSE_BODY_BYTES,
            None,
        )
    }

    /// Build using the central request settings, without a process-global provider.
    pub fn with_settings(
        settings: &super::request::TransportSettings,
        client: &super::request::ClientSettings,
    ) -> Result<Self> {
        settings.validate()?;
        Self::build(
            Duration::from_millis(settings.timeout_ms),
            Duration::from_millis(settings.connect_timeout_ms),
            settings.max_response_bytes,
            client.user_agent.as_ref().map(|value| value.as_str()),
        )
    }

    fn build(
        timeout: Duration,
        connect_timeout: Duration,
        max_response_bytes: usize,
        user_agent: Option<&str>,
    ) -> Result<Self> {
        if timeout.is_zero() || timeout > Self::MAX_TIMEOUT {
            return Err(Error::invalid(
                "registration_https_timeout",
                "must be greater than zero and at most five minutes",
            ));
        }
        let roots =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let tls = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(|_| {
            Error::invalid(
                "registration_https_tls",
                "ring provider has no safe protocol-version configuration",
            )
        })?
        .with_root_certificates(roots)
        .with_no_client_auth();
        let mut builder = reqwest::Client::builder()
            .use_preconfigured_tls(tls)
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_proxy()
            .referer(false)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .pool_max_idle_per_host(1);
        if let Some(value) = user_agent {
            if value.is_empty()
                || value.len() > 512
                || !value.bytes().all(|b| (32..=126).contains(&b))
            {
                return Err(Error::invalid("client.user_agent", "invalid header value"));
            }
            builder = builder.user_agent(value);
        }
        let client = builder.build().map_err(|_| {
            Error::invalid(
                "registration_https_tls",
                "failed to construct constrained HTTPS client",
            )
        })?;
        Ok(Self {
            client,
            timeout,
            max_response_bytes,
        })
    }
}

#[cfg(feature = "experimental-registration-https")]
impl RegistrationTransport for ExperimentalHttpsTransport {
    fn send<'a>(&'a self, request: EncodedRegistrationRequest) -> RegistrationTransportFuture<'a> {
        Box::pin(async move {
            if !request.url().starts_with("https://v.whatsapp.net/") {
                return Err(Error::invalid(
                    "registration_url",
                    "experimental transport accepts only the comparison registration origin",
                ));
            }
            let mut response = self
                .client
                .post(request.url())
                .header(reqwest::header::CONTENT_TYPE, request.content_type())
                .header(reqwest::header::CONTENT_LENGTH, request.content_length())
                .body(request.body().to_vec())
                .send()
                .await
                .map_err(|_| {
                    Error::Io(std::io::Error::other("registration HTTPS request failed"))
                })?;
            if response
                .content_length()
                .is_some_and(|length| length > self.max_response_bytes as u64)
            {
                return Err(Error::ResourceExhausted {
                    resource: "registration response body bytes",
                    limit: self.max_response_bytes as u64,
                });
            }
            let mut body = Zeroizing::new(Vec::with_capacity(
                response
                    .content_length()
                    .and_then(|length| usize::try_from(length).ok())
                    .unwrap_or(0)
                    .min(self.max_response_bytes),
            ));
            while let Some(chunk) = response.chunk().await.map_err(|_| {
                Error::Io(std::io::Error::other(
                    "registration HTTPS response body failed",
                ))
            })? {
                if body.len().saturating_add(chunk.len()) > self.max_response_bytes {
                    return Err(Error::ResourceExhausted {
                        resource: "registration response body bytes",
                        limit: self.max_response_bytes as u64,
                    });
                }
                body.extend_from_slice(&chunk);
            }
            RegistrationHttpResponse::new(response.status().as_u16(), body)
        })
    }
}

/// Bounded, secret-bearing HTTP response returned by a registration
/// transport.
pub struct RegistrationHttpResponse {
    status_code: u16,
    body: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for RegistrationHttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationHttpResponse")
            .field("status_code", &self.status_code)
            .field("body_length", &self.body.len())
            .field("body", &"<redacted>")
            .finish()
    }
}

impl RegistrationHttpResponse {
    /// Safe HTTP and parsed control metadata, excluding arbitrary server text.
    /// Unknown reasons remain `Other`; raw response bodies are never logged.
    pub fn diagnostic_summary(&self) -> serde_json::Value {
        let mut summary =
            serde_json::json!({"http_status": self.status_code, "body_bytes": self.body.len()});
        match RegistrationResponse::parse(Zeroizing::new(self.body.to_vec())) {
            Ok(parsed) => {
                summary["status"] = serde_json::json!(format!("{:?}", parsed.status));
                summary["reason"] = serde_json::json!(parsed.reason.map(|v| format!("{v:?}")));
                summary["invalid_parameter"] = serde_json::json!(parsed.invalid_parameter);
                if parsed.reason == Some(RegistrationResponseReason::FormatWrong) {
                    // Comparison APK: VerifyPhoneNumber.A19, dialog 53,
                    // string resource 0x7f123741. This is client interpretation,
                    // not independent proof of number assignment or eligibility.
                    summary["operator_hint"] = serde_json::json!(
                        "APK 2.26.37.70 presents this as an invalid mobile number for the country. Check the country code and national number before another attempt; this does not identify a missing protocol field."
                    );
                }
                summary["retry_after_seconds"] =
                    serde_json::json!(parsed.retry_after.map(|v| v.as_secs()));
                summary["sms_wait_seconds"] =
                    serde_json::json!(parsed.sms_wait.map(|v| v.as_secs()));
                summary["voice_wait_seconds"] =
                    serde_json::json!(parsed.voice_wait.map(|v| v.as_secs()));
            }
            Err(_) => {
                summary["parse_error"] =
                    serde_json::json!("unrecognized or invalid bounded response; body withheld");
            }
        }
        summary
    }
    /// Construct a response while enforcing the local body-size limit.
    pub fn new(status_code: u16, body: Zeroizing<Vec<u8>>) -> Result<Self> {
        if !(100..=599).contains(&status_code) {
            return Err(Error::invalid(
                "registration_http_status",
                "must be a three-digit HTTP status",
            ));
        }
        if body.len() > MAX_RESPONSE_BODY_BYTES {
            return Err(Error::ResourceExhausted {
                resource: "registration response body bytes",
                limit: MAX_RESPONSE_BODY_BYTES as u64,
            });
        }
        Ok(Self { status_code, body })
    }

    /// HTTP status code returned by the server.
    pub const fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Parse the JSON object observed in the comparison client. This consumes
    /// and zeroizes the original body buffer.
    pub fn parse_json(self) -> Result<RegistrationResponse> {
        RegistrationResponse::parse(self.body)
    }
}

/// Status strings handled by the comparison client's registration bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistrationResponseStatus {
    /// Request completed.
    Ok,
    /// A verification request was sent.
    Sent,
    /// Request failed with a reason token.
    Fail,
    /// A code or account was attached.
    Attached,
    /// A status added after the inspected build.
    Other,
}

impl RegistrationResponseStatus {
    /// Statuses explicitly handled by the comparison build.
    pub const OBSERVED: [Self; 4] = [Self::Ok, Self::Sent, Self::Fail, Self::Attached];

    /// Wire spelling for a status known to the comparison build.
    pub const fn wire_value(self) -> Option<&'static str> {
        match self {
            Self::Ok => Some("ok"),
            Self::Sent => Some("sent"),
            Self::Fail => Some("fail"),
            Self::Attached => Some("attached"),
            Self::Other => None,
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "ok" => Self::Ok,
            "sent" => Self::Sent,
            "fail" => Self::Fail,
            "attached" => Self::Attached,
            _ => Self::Other,
        }
    }
}

/// Reason strings explicitly handled by the comparison client's registration
/// bridge. Meaning and state transitions still require live validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RegistrationResponseReason {
    /// Verification guesses were submitted too quickly.
    GuessedTooFast,
    /// The operation is not allowed.
    NotAllowed,
    /// A reset was requested too soon.
    ResetTooSoon,
    /// Device confirmation or a second code is required.
    DeviceConfirmOrSecondCode,
    /// A challenge step is required.
    Challenge,
    /// Required state or input is missing.
    Missing,
    /// A consent step is required.
    Consent,
    /// A second verification code is required.
    SecondCode,
    /// The submitted state is stale.
    Stale,
    /// The operation or account is blocked.
    Blocked,
    /// Too many verification guesses were submitted.
    TooManyGuesses,
    /// An account security code is required.
    SecurityCode,
    /// The operation is unavailable for this release cohort.
    LimitedRelease,
    /// Submitted state does not match server state.
    Mismatch,
    /// The operation is temporarily unavailable.
    TemporarilyUnavailable,
    /// A submitted value is incorrect.
    Incorrect,
    /// The parent-linking consent target is already registered.
    ConsentParentLinkingAlreadyRegistered,
    /// A submitted value has the wrong format.
    FormatWrong,
    /// SMS verification is required.
    SmsRequired,
    /// A supplied token is invalid.
    BadToken,
    /// A supplied parameter is invalid.
    BadParam,
    /// The operation was attempted too recently.
    TooRecent,
    /// The server requires another delivery method.
    NextMethod,
    /// The caller should request SMS delivery.
    SendSms,
    /// The client version is too old.
    OldVersion,
    /// No delivery route is available.
    NoRoutes,
    /// The signed pre-key is invalid.
    InvalidSkey,
    /// Too many attempts were made across all methods.
    TooManyAllMethods,
    /// A generic attempt limit was reached.
    TooMany,
    /// The server is waiting for SMS verification.
    WaitingForSms,
    /// Flash-call verification is disabled.
    FlashDisabled,
    /// A required parameter is absent.
    MissingParam,
    /// The selected provider cannot route the request.
    ProviderUnroutable,
    /// The selected provider timed out.
    ProviderTimeout,
    /// A submitted value is too short.
    LengthShort,
    /// A submitted value is too long.
    LengthLong,
    /// Business registration is not allowed.
    BizNotAllowed,
    /// A reason added after the inspected build.
    Other,
}

impl RegistrationResponseReason {
    /// Reason tokens explicitly handled by the comparison build.
    pub const OBSERVED: [Self; 37] = [
        Self::GuessedTooFast,
        Self::NotAllowed,
        Self::ResetTooSoon,
        Self::DeviceConfirmOrSecondCode,
        Self::Challenge,
        Self::Missing,
        Self::Consent,
        Self::SecondCode,
        Self::Stale,
        Self::Blocked,
        Self::TooManyGuesses,
        Self::SecurityCode,
        Self::LimitedRelease,
        Self::Mismatch,
        Self::TemporarilyUnavailable,
        Self::Incorrect,
        Self::ConsentParentLinkingAlreadyRegistered,
        Self::FormatWrong,
        Self::SmsRequired,
        Self::BadToken,
        Self::BadParam,
        Self::TooRecent,
        Self::NextMethod,
        Self::SendSms,
        Self::OldVersion,
        Self::NoRoutes,
        Self::InvalidSkey,
        Self::TooManyAllMethods,
        Self::TooMany,
        Self::WaitingForSms,
        Self::FlashDisabled,
        Self::MissingParam,
        Self::ProviderUnroutable,
        Self::ProviderTimeout,
        Self::LengthShort,
        Self::LengthLong,
        Self::BizNotAllowed,
    ];

    /// Wire spelling for a reason known to the comparison build.
    pub const fn wire_value(self) -> Option<&'static str> {
        Some(match self {
            Self::GuessedTooFast => "guessed_too_fast",
            Self::NotAllowed => "not_allowed",
            Self::ResetTooSoon => "reset_too_soon",
            Self::DeviceConfirmOrSecondCode => "device_confirm_or_second_code",
            Self::Challenge => "challenge",
            Self::Missing => "missing",
            Self::Consent => "consent",
            Self::SecondCode => "second_code",
            Self::Stale => "stale",
            Self::Blocked => "blocked",
            Self::TooManyGuesses => "too_many_guesses",
            Self::SecurityCode => "security_code",
            Self::LimitedRelease => "limited_release",
            Self::Mismatch => "mismatch",
            Self::TemporarilyUnavailable => "temporarily_unavailable",
            Self::Incorrect => "incorrect",
            Self::ConsentParentLinkingAlreadyRegistered => {
                "consent_parent_linking_already_registered"
            }
            Self::FormatWrong => "format_wrong",
            Self::SmsRequired => "sms_required",
            Self::BadToken => "bad_token",
            Self::BadParam => "bad_param",
            Self::TooRecent => "too_recent",
            Self::NextMethod => "next_method",
            Self::SendSms => "send_sms",
            Self::OldVersion => "old_version",
            Self::NoRoutes => "no_routes",
            Self::InvalidSkey => "invalid_skey",
            Self::TooManyAllMethods => "too_many_all_methods",
            Self::TooMany => "too_many",
            Self::WaitingForSms => "waiting_for_sms",
            Self::FlashDisabled => "flash_disabled",
            Self::MissingParam => "missing_param",
            Self::ProviderUnroutable => "provider_unroutable",
            Self::ProviderTimeout => "provider_timeout",
            Self::LengthShort => "length_short",
            Self::LengthLong => "length_long",
            Self::BizNotAllowed => "biz_not_allowed",
            Self::Other => return None,
        })
    }

    fn parse(value: &str) -> Self {
        Self::OBSERVED
            .into_iter()
            .find(|reason| reason.wire_value() == Some(value))
            .unwrap_or(Self::Other)
    }
}

/// Secret fields extracted from a registration response. Debug output reports
/// presence only, and every value is zeroized on drop.
#[derive(Default)]
pub struct RegistrationResponseSecrets {
    login: Option<Zeroizing<String>>,
    code: Option<Zeroizing<String>>,
    wipe_token: Option<Zeroizing<String>>,
    passkey_credential: Option<Zeroizing<String>>,
    appeal_token: Option<Zeroizing<String>>,
    edge_routing_info: Option<Zeroizing<String>>,
}

impl fmt::Debug for RegistrationResponseSecrets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationResponseSecrets")
            .field("has_login", &self.login.is_some())
            .field("has_code", &self.code.is_some())
            .field("has_wipe_token", &self.wipe_token.is_some())
            .field("has_passkey_credential", &self.passkey_credential.is_some())
            .field("has_appeal_token", &self.appeal_token.is_some())
            .field("has_edge_routing_info", &self.edge_routing_info.is_some())
            .finish()
    }
}

impl RegistrationResponseSecrets {
    /// Remove and return the login credential, if present.
    pub fn take_login(&mut self) -> Option<Zeroizing<String>> {
        self.login.take()
    }

    /// Remove and return the server-provided code, if present.
    pub fn take_code(&mut self) -> Option<Zeroizing<String>> {
        self.code.take()
    }

    /// Remove and return the wipe token, if present.
    pub fn take_wipe_token(&mut self) -> Option<Zeroizing<String>> {
        self.wipe_token.take()
    }

    /// Remove and return the passkey credential, if present.
    pub fn take_passkey_credential(&mut self) -> Option<Zeroizing<String>> {
        self.passkey_credential.take()
    }

    /// Remove and return the appeal token, if present.
    pub fn take_appeal_token(&mut self) -> Option<Zeroizing<String>> {
        self.appeal_token.take()
    }

    /// Remove and return edge-routing information, if present.
    pub fn take_edge_routing_info(&mut self) -> Option<Zeroizing<String>> {
        self.edge_routing_info.take()
    }
}

/// Sanitized control fields parsed from a comparison-build registration JSON
/// response.
pub struct RegistrationResponse {
    /// Parsed response status.
    pub status: RegistrationResponseStatus,
    /// Parsed reason, when the server included one.
    pub reason: Option<RegistrationResponseReason>,
    /// Allowlisted request-field name from an optional `param` diagnostic hint.
    /// Not proof that the field is missing; arbitrary server strings are withheld.
    pub invalid_parameter: Option<&'static str>,
    /// General retry delay.
    pub retry_after: Option<Duration>,
    /// SMS retry delay.
    pub sms_wait: Option<Duration>,
    /// Voice-call retry delay.
    pub voice_wait: Option<Duration>,
    /// Flash-call retry delay.
    pub flash_wait: Option<Duration>,
    /// Account-wipe retry delay.
    pub wipe_wait: Option<Duration>,
    /// Generic verification-code length.
    pub code_length: Option<u32>,
    /// SMS verification-code length.
    pub sms_length: Option<u32>,
    /// Voice verification-code length.
    pub voice_length: Option<u32>,
    /// Whether an account security code is configured.
    pub security_code_set: bool,
    /// Whether the secure verifier flow is enabled.
    pub secure_verifier: bool,
    secrets: RegistrationResponseSecrets,
}

impl fmt::Debug for RegistrationResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationResponse")
            .field("status", &self.status)
            .field("reason", &self.reason)
            .field("invalid_parameter", &self.invalid_parameter)
            .field("retry_after", &self.retry_after)
            .field("sms_wait", &self.sms_wait)
            .field("voice_wait", &self.voice_wait)
            .field("flash_wait", &self.flash_wait)
            .field("wipe_wait", &self.wipe_wait)
            .field("code_length", &self.code_length)
            .field("sms_length", &self.sms_length)
            .field("voice_length", &self.voice_length)
            .field("security_code_set", &self.security_code_set)
            .field("secure_verifier", &self.secure_verifier)
            .field("secrets", &self.secrets)
            .finish()
    }
}

impl RegistrationResponse {
    fn parse(body: Zeroizing<Vec<u8>>) -> Result<Self> {
        let raw: RawRegistrationResponse = serde_json::from_slice(&body).map_err(|_| {
            Error::invalid(
                "registration_response",
                "expected a bounded JSON object with supported field types",
            )
        })?;
        let status = raw
            .status
            .ok_or_else(|| Error::invalid("registration_response", "missing non-empty status"))?;
        validate_response_token("status", &status)?;
        if let Some(reason) = raw.reason.as_deref() {
            validate_response_token("reason", reason)?;
        }
        for secret in [
            raw.login.as_ref(),
            raw.code.as_ref(),
            raw.wipe_token.as_ref(),
            raw.passkey_credential.as_ref(),
            raw.appeal_token.as_ref(),
            raw.edge_routing_info.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if secret.len() > MAX_RESPONSE_SECRET_BYTES {
                return Err(Error::ResourceExhausted {
                    resource: "registration response secret bytes",
                    limit: MAX_RESPONSE_SECRET_BYTES as u64,
                });
            }
        }

        Ok(Self {
            status: RegistrationResponseStatus::parse(&status),
            reason: raw.reason.as_deref().map(RegistrationResponseReason::parse),
            invalid_parameter: match raw.param {
                Some(RawDiagnosticParameter::Text(value)) => KNOWN_REQUEST_PARAMETERS
                    .iter()
                    .copied()
                    .find(|name| *name == value.as_str()),
                _ => None,
            },
            retry_after: parse_seconds("retry_after", raw.retry_after)?,
            sms_wait: parse_seconds("sms_wait", raw.sms_wait)?,
            voice_wait: parse_seconds("voice_wait", raw.voice_wait)?,
            flash_wait: parse_seconds("flash_wait", raw.flash_wait)?,
            wipe_wait: parse_seconds("wipe_wait", raw.wipe_wait)?,
            code_length: raw.length,
            sms_length: raw.sms_length,
            voice_length: raw.voice_length,
            security_code_set: raw.security_code_set,
            secure_verifier: raw.secure_verifier,
            secrets: RegistrationResponseSecrets {
                login: raw.login,
                code: raw.code,
                wipe_token: raw.wipe_token,
                passkey_credential: raw.passkey_credential,
                appeal_token: raw.appeal_token,
                edge_routing_info: raw.edge_routing_info,
            },
        })
    }

    /// Move secret-bearing fields into a separately redacted container.
    pub fn take_secrets(&mut self) -> RegistrationResponseSecrets {
        std::mem::take(&mut self.secrets)
    }
}

#[derive(Deserialize)]
struct RawRegistrationResponse {
    status: Option<String>,
    reason: Option<String>,
    param: Option<RawDiagnosticParameter>,
    retry_after: Option<SecondsValue>,
    sms_wait: Option<SecondsValue>,
    voice_wait: Option<SecondsValue>,
    flash_wait: Option<SecondsValue>,
    wipe_wait: Option<SecondsValue>,
    length: Option<u32>,
    sms_length: Option<u32>,
    voice_length: Option<u32>,
    #[serde(default)]
    security_code_set: bool,
    #[serde(default)]
    secure_verifier: bool,
    login: Option<Zeroizing<String>>,
    code: Option<Zeroizing<String>>,
    wipe_token: Option<Zeroizing<String>>,
    passkey_credential: Option<Zeroizing<String>>,
    appeal_token: Option<Zeroizing<String>>,
    edge_routing_info: Option<Zeroizing<String>>,
}

// A diagnostic extension, not a claim that captured responses included `param`.
// Ignore unexpected types without losing the primary status/reason.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawDiagnosticParameter {
    Text(Zeroizing<String>),
    Other(serde::de::IgnoredAny),
}

const KNOWN_REQUEST_PARAMETERS: &[&str] = &[
    // Server alias explicitly handled by VerifyPhoneNumber.A2B in the APK.
    "number",
    "ENC",
    "H",
    "reason",
    "mistyped",
    "hasav",
    "client_metrics",
    "mcc",
    "mnc",
    "sim_mcc",
    "sim_mnc",
    "education_screen_displayed",
    "prefer_sms_over_flash",
    "feo2_query_status",
    "recaptcha",
    "push_code",
    "new_acc_uuid",
    "cc",
    "in",
    "lg",
    "lc",
    "fdid",
    "expid",
    "access_session_id",
    "id",
    "backup_token",
    "token",
    "method",
    "context",
    "clicked_education_link",
    "manage_call_permission",
    "call_log_permission",
    "client_start_message",
    "advertising_id",
    "login",
    "type",
    "authkey",
    "e_ident",
    "e_keytype",
    "e_regid",
    "e_skey_id",
    "e_skey_val",
    "e_skey_sig",
    "code",
    "auth_response",
    "reset",
    "wipe_token",
    "ENC",
];

#[derive(Deserialize)]
#[serde(untagged)]
enum SecondsValue {
    Number(u64),
    String(String),
}

fn parse_seconds(field: &'static str, value: Option<SecondsValue>) -> Result<Option<Duration>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let seconds = match value {
        SecondsValue::Number(value) => value,
        SecondsValue::String(value) => value.parse::<u64>().map_err(|_| {
            Error::invalid("registration_response", format!("invalid {field} value"))
        })?,
    };
    Ok(Some(Duration::from_secs(seconds)))
}

fn validate_response_token(field: &'static str, value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(Error::invalid(
            "registration_response",
            format!("invalid {field} token"),
        ));
    }
    Ok(())
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

fn validate_parameter_text(field: &'static str, value: &str, allow_empty: bool) -> Result<()> {
    if (!allow_empty && value.is_empty()) || value.len() > MAX_FIELD_VALUE_BYTES {
        let minimum = usize::from(!allow_empty);
        return Err(Error::invalid(
            field,
            format!("must contain {minimum}..={MAX_FIELD_VALUE_BYTES} UTF-8 bytes"),
        ));
    }
    Ok(())
}

fn validate_binary_parameter(field: &'static str, value: &[u8], allow_empty: bool) -> Result<()> {
    if (!allow_empty && value.is_empty()) || value.len() > MAX_BINARY_PARAMETER_BYTES {
        let minimum = usize::from(!allow_empty);
        return Err(Error::invalid(
            field,
            format!("must contain {minimum}..={MAX_BINARY_PARAMETER_BYTES} bytes"),
        ));
    }
    Ok(())
}

fn base64_url_no_pad(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    let (chunks, remainder) = input.as_chunks::<3>();
    for chunk in chunks {
        output.push(TABLE[usize::from(chunk[0] >> 2)] as char);
        output.push(TABLE[usize::from(((chunk[0] & 0x03) << 4) | (chunk[1] >> 4))] as char);
        output.push(TABLE[usize::from(((chunk[1] & 0x0f) << 2) | (chunk[2] >> 6))] as char);
        output.push(TABLE[usize::from(chunk[2] & 0x3f)] as char);
    }
    match remainder {
        [first] => {
            output.push(TABLE[usize::from(first >> 2)] as char);
            output.push(TABLE[usize::from((first & 0x03) << 4)] as char);
        }
        [first, second] => {
            output.push(TABLE[usize::from(first >> 2)] as char);
            output.push(TABLE[usize::from(((first & 0x03) << 4) | (second >> 4))] as char);
            output.push(TABLE[usize::from((second & 0x0f) << 2)] as char);
        }
        [] => {}
        _ => unreachable!("as_chunks::<3>() remainder is at most two bytes"),
    }
    output
}

fn percent_encode_binary(input: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(input.len().saturating_mul(3));
    for &byte in input {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    output
}

fn validate_pre_encoded(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let valid = match bytes[index] {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' | b'~' | b'+' => {
                true
            }
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
    #[test]
    fn diagnostic_field_patch_is_additive_and_code_only() {
        let mut form = RegistrationForm::new();
        form.push("reason", "").unwrap();
        let request = || {
            WHATSAPP_2_26_37_70
                .request(RegistrationEndpoint::Code, &form)
                .unwrap()
        };
        let fields = serde_json::from_str(r#"{"code_bytes":{"mcc":"000"}}"#).unwrap();
        assert_eq!(
            request()
                .with_additional_code_fields(&fields)
                .unwrap()
                .body(),
            b"reason=&mcc=000"
        );
        for json in [
            r#"{}"#,
            r#"{"code_bytes":{"reason":""}}"#,
            r#"{"code_bytes":{"token":"x"}}"#,
            r#"{"register":{"mcc":"000"}}"#,
        ] {
            assert!(
                request()
                    .with_additional_code_fields(&serde_json::from_str(json).unwrap())
                    .is_err()
            );
        }
        let mut full = RegistrationForm::new();
        for index in 0..MAX_FORM_FIELDS {
            full.push(format!("x{index}"), "v").unwrap();
        }
        assert!(
            WHATSAPP_2_26_37_70
                .request(RegistrationEndpoint::Code, &full)
                .unwrap()
                .with_additional_code_fields(&fields)
                .is_err()
        );
    }

    #[cfg(feature = "experimental-signal-crypto")]
    #[test]
    fn unsigned_envelope_preserves_empty_signature_field() {
        let request = WHATSAPP_2_26_37_70
            .request(RegistrationEndpoint::Code, &RegistrationForm::new())
            .unwrap()
            .encrypt_comparison_without_signature()
            .unwrap();
        assert!(request.body().starts_with(b"ENC="));
        assert!(request.body().ends_with(b"&H="));
        assert_eq!(
            request.diagnostic_summary()["fields"],
            serde_json::json!(["ENC", "H"])
        );
        assert!(request.encrypt_comparison_without_signature().is_err());
    }

    #[cfg(feature = "experimental-signal-crypto")]
    #[test]
    fn comparison_envelope_is_explicit_redacted_and_not_double_wrapped() {
        let mut form = RegistrationForm::new();
        form.push("reason", "private-example").unwrap();
        let clear = WHATSAPP_2_26_37_70
            .request(RegistrationEndpoint::Code, &form)
            .unwrap();
        assert_eq!(clear.body(), b"reason=private-example");
        let encrypted = clear.encrypt_comparison().unwrap();
        assert!(encrypted.body().starts_with(b"ENC="));
        assert!(
            encrypted.body()[4..]
                .iter()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        );
        assert_eq!(
            encrypted.diagnostic_summary()["fields"],
            serde_json::json!(["ENC"])
        );
        assert!(!format!("{encrypted:?}").contains("private-example"));
        assert!(encrypted.encrypt_comparison().is_err());
    }

    use super::*;
    use serde_json::Value;

    #[test]
    fn bad_parameter_hints_are_allowlisted_and_never_echo_arbitrary_values() {
        for (hint, expected) in [
            (serde_json::json!("number"), Some("number")),
            (serde_json::json!("type"), Some("type")),
            (serde_json::json!("mcc"), Some("mcc")),
            (serde_json::json!("e_skey_sig"), Some("e_skey_sig")),
            (serde_json::json!("secret-phone-or-token"), None),
            (serde_json::json!({"secret": "secret-phone-or-token"}), None),
            (serde_json::json!(["secret-phone-or-token"]), None),
            (serde_json::json!(42), None),
            (Value::Null, None),
        ] {
            let body = serde_json::to_vec(&serde_json::json!({
                "status": "fail", "reason": "bad_param", "param": hint
            }))
            .unwrap();
            let response = RegistrationHttpResponse::new(200, Zeroizing::new(body)).unwrap();
            let summary = response.diagnostic_summary();
            assert_eq!(summary["reason"], "BadParam");
            assert_eq!(summary["invalid_parameter"], serde_json::json!(expected));
            assert!(!summary.to_string().contains("secret-phone-or-token"));
            assert_eq!(response.parse_json().unwrap().invalid_parameter, expected);
        }
    }

    #[test]
    fn additional_bytes_use_rfc3986_without_double_encoding() {
        let mut form = RegistrationForm::new();
        form.push("text", "a ~*+%&").unwrap();
        form.push_bytes("bytes", b"a ~*+%&\0\xff").unwrap();
        assert_eq!(
            form.encode().unwrap().as_slice(),
            b"text=a+%7E*%2B%25%26&bytes=a%20~%2A%2B%25%26%00%FF"
        );
        assert!(form.push_bytes("bytes", b"duplicate").is_err());
        assert!(
            RegistrationForm::new()
                .push_bytes("x", &vec![0; MAX_BINARY_PARAMETER_BYTES + 1])
                .is_err()
        );
    }

    #[test]
    fn diagnostics_withhold_response_secrets_and_malformed_bodies() {
        for body in [
            r#"{"status":"fail","reason":"private-server-text","token":"secret-token"}"#,
            "malformed secret-token",
        ] {
            let response =
                RegistrationHttpResponse::new(403, Zeroizing::new(body.as_bytes().to_vec()))
                    .unwrap();
            let diagnostic = response.diagnostic_summary();
            assert_eq!(diagnostic["http_status"], 403);
            assert!(!diagnostic.to_string().contains("secret-token"));
            assert!(!diagnostic.to_string().contains("private-server-text"));
        }
    }

    #[test]
    fn format_wrong_hint_is_static_and_reason_specific() {
        for reason in ["format_wrong", "bad_param", "unknown-private-reason"] {
            let body = serde_json::to_vec(&serde_json::json!({
                "status": "fail", "reason": reason, "number": "private-number"
            }))
            .unwrap();
            let response = RegistrationHttpResponse::new(200, Zeroizing::new(body)).unwrap();
            let summary = response.diagnostic_summary();
            assert_eq!(
                summary.get("operator_hint").is_some(),
                reason == "format_wrong"
            );
            if reason == "format_wrong" {
                assert!(
                    summary["operator_hint"]
                        .as_str()
                        .unwrap()
                        .contains("invalid mobile number")
                );
            }
            assert!(!summary.to_string().contains("private-number"));
            assert!(!summary.to_string().contains("unknown-private-reason"));
        }
    }

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
        let statuses: Vec<_> = RegistrationResponseStatus::OBSERVED
            .iter()
            .map(|status| status.wire_value().unwrap())
            .collect();
        assert_eq!(
            evidence["observations"]["registration"]["response"]["statuses"],
            serde_json::json!(statuses)
        );
        let reasons: Vec<_> = RegistrationResponseReason::OBSERVED
            .iter()
            .map(|reason| reason.wire_value().unwrap())
            .collect();
        assert_eq!(
            evidence["observations"]["registration"]["response"]["reason_values"],
            serde_json::json!(reasons)
        );
        let key_encoding = &evidence["observations"]["registration"]["key_bundle_encoding"];
        assert_eq!(key_encoding["key_type_hex"], "05");
        assert_eq!(
            key_encoding["parameter_order"],
            serde_json::json!([
                "authkey",
                "e_ident",
                "e_keytype",
                "e_regid",
                "e_skey_id",
                "e_skey_val",
                "e_skey_sig"
            ])
        );
        let common_encoding =
            &evidence["observations"]["registration"]["common_parameter_encoding"];
        assert_eq!(
            common_encoding["helper_groups"]["A0T"],
            serde_json::json!(["cc", "in"])
        );
        assert_eq!(
            common_encoding["helper_groups"]["A0U"],
            serde_json::json!(["lg", "lc", "fdid", "expid"])
        );
        assert_eq!(
            common_encoding["helper_groups"]["A0V"],
            serde_json::json!(["access_session_id", "id", "backup_token"])
        );
        let code_encoding = &evidence["observations"]["registration"]["code_request_encoding"];
        assert_eq!(code_encoding["path"], RegistrationEndpoint::Code.path());
        assert_eq!(
            code_encoding["required_fields"],
            serde_json::json!(["token", "method"])
        );
        let register_encoding =
            &evidence["observations"]["registration"]["register_request_encoding"];
        assert_eq!(
            register_encoding["path"],
            RegistrationEndpoint::Register.path()
        );
        assert_eq!(
            register_encoding["required_fields"],
            serde_json::json!(["code"])
        );
        let security_encoding =
            &evidence["observations"]["registration"]["security_request_encoding"];
        assert_eq!(
            security_encoding["path"],
            RegistrationEndpoint::Security.path()
        );
        assert_eq!(
            security_encoding["optional_field_order"],
            serde_json::json!(["code", "reset", "wipe_token", "advertising_id"])
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

    #[test]
    fn common_parameters_match_observed_group_order_and_encodings() {
        let common = RegistrationCommonParams::new(
            PhoneNumber::new(1, "5551234567").unwrap(),
            "en",
            "US",
            "fd id",
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            Some([
                16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ]),
            b"A ~*".to_vec(),
            vec![0xff, b'.', b'_'],
        )
        .unwrap();
        let mut form = RegistrationForm::new();
        common.append_to_form(&mut form).unwrap();
        assert_eq!(
            form.encode().unwrap().as_slice(),
            b"cc=1&in=5551234567&lg=en&lc=US&fdid=fd+id&expid=AAECAwQFBgcICQoLDA0ODw&access_session_id=EBESExQVFhcYGRobHB0eHw&id=A%20~%2A&backup_token=%FF._"
        );
        let debug = format!("{common:?}");
        assert!(!debug.contains("5551234567"));
        assert!(!debug.contains("fd id"));
        assert!(!debug.contains("A ~*"));
    }

    #[test]
    fn common_parameter_validation_and_append_are_atomic() {
        let phone = PhoneNumber::new(1, "5551234567").unwrap();
        assert!(
            RegistrationCommonParams::new(
                phone.clone(),
                "en",
                "US",
                "fdid",
                [0; 16],
                None,
                Vec::new(),
                vec![1],
            )
            .is_err()
        );
        assert!(
            RegistrationCommonParams::new(
                phone.clone(),
                "en",
                "US",
                "fdid",
                [0; 16],
                None,
                vec![1],
                Vec::new(),
            )
            .is_err()
        );

        let common = RegistrationCommonParams::new(
            phone,
            "en",
            "US",
            "fdid",
            [0; 16],
            None,
            vec![1],
            vec![2],
        )
        .unwrap();
        let mut form = RegistrationForm::new();
        form.push("lc", "existing").unwrap();
        assert!(common.append_to_form(&mut form).is_err());
        assert_eq!(form.len(), 1, "failed batch append must not add fields");
        assert_eq!(form.encode().unwrap().as_slice(), b"lc=existing");
    }

    #[test]
    fn code_request_uses_observed_field_order_and_encodings() {
        let common = RegistrationCommonParams::new(
            PhoneNumber::new(1, "5551234567").unwrap(),
            "en",
            "US",
            "fdid",
            [0; 16],
            None,
            vec![1],
            vec![2],
        )
        .unwrap();
        let code = RegistrationCodeParams::new("t+&", "sms")
            .unwrap()
            .with_context("ctx x")
            .unwrap()
            .with_permissions(Some(true), Some(false), None)
            .with_client_start_message(b"foo".to_vec())
            .unwrap()
            .with_advertising_id("ad id")
            .unwrap()
            .with_login("log+")
            .unwrap()
            .with_registration_type(-1);
        let request = assemble_code_request(&common, &code, None).unwrap();
        assert_eq!(request.url(), "https://v.whatsapp.net/v2/code");
        assert_eq!(
            request.body(),
            b"cc=1&in=5551234567&lg=en&lc=US&fdid=fdid&expid=AAAAAAAAAAAAAAAAAAAAAA&id=%01&backup_token=%02&token=t%2B%26&method=sms&context=ctx+x&clicked_education_link=true&manage_call_permission=false&client_start_message=Zm9v&advertising_id=ad+id&login=log%2B&type=-1"
        );
        let debug = format!("{code:?}");
        assert!(!debug.contains("t+&"));
        assert!(!debug.contains("ctx x"));
        assert!(!debug.contains("ad id"));
        assert!(!debug.contains("log+"));
    }

    #[test]
    fn code_parameter_validation_and_append_are_atomic() {
        assert!(RegistrationCodeParams::new("", "sms").is_err());
        assert!(RegistrationCodeParams::new("token", "").is_err());
        assert!(
            RegistrationCodeParams::new("token", "sms")
                .unwrap()
                .with_client_start_message(vec![0; MAX_BINARY_PARAMETER_BYTES + 1])
                .is_err()
        );

        let code = RegistrationCodeParams::new("token", "sms").unwrap();
        let mut form = RegistrationForm::new();
        form.push("method", "existing").unwrap();
        assert!(code.append_to_form(&mut form).is_err());
        assert_eq!(form.len(), 1, "failed batch append must not add fields");
        assert_eq!(form.encode().unwrap().as_slice(), b"method=existing");
    }

    #[test]
    fn code_params_can_derive_comparison_token_from_apk_material() {
        let params = RegistrationCodeParams::from_comparison_token_material(
            &PhoneNumber::new(44, "7700900123").unwrap(),
            super::super::token::ComparisonTokenMaterial {
                package_name: "com.whatsapp",
                about_logo: b"logo fixture",
                signing_certificate_der: b"certificate fixture",
                classes_dex: b"dex fixture",
            },
            "sms",
        )
        .unwrap();
        let mut form = RegistrationForm::new();
        params.append_to_form(&mut form).unwrap();
        assert_eq!(
            form.encode().unwrap().as_slice(),
            b"token=qVj97PKhWFvdbTpFsH7gSeiqQTk%3D&method=sms"
        );
    }

    #[test]
    fn completion_request_uses_observed_field_order_and_encodings() {
        let common = RegistrationCommonParams::new(
            PhoneNumber::new(44, "7700900123").unwrap(),
            "en",
            "GB",
            "fdid",
            [0; 16],
            None,
            vec![1],
            vec![2],
        )
        .unwrap();
        let complete = RegistrationCompleteParams::new("123456")
            .unwrap()
            .with_auth_response(vec![0xfb, 0xff, 0xef])
            .unwrap()
            .with_context("ctx")
            .unwrap()
            .with_method("sms")
            .unwrap()
            .with_advertising_id("ad id")
            .unwrap()
            .with_login("login-secret")
            .unwrap()
            .with_registration_type(2);
        let request = assemble_complete_request(&common, &complete, None).unwrap();
        assert_eq!(request.url(), "https://v.whatsapp.net/v2/register");
        assert_eq!(
            request.body(),
            b"cc=44&in=7700900123&lg=en&lc=GB&fdid=fdid&expid=AAAAAAAAAAAAAAAAAAAAAA&id=%01&backup_token=%02&code=123456&auth_response=-__v&context=ctx&method=sms&advertising_id=ad+id&login=login-secret&type=2"
        );
        let debug = format!("{complete:?}");
        assert!(!debug.contains("123456"));
        assert!(!debug.contains("login-secret"));
        assert!(!debug.contains("ad id"));
    }

    #[test]
    fn completion_parameter_validation_and_append_are_atomic() {
        assert!(RegistrationCompleteParams::new("").is_err());
        assert!(
            RegistrationCompleteParams::new("123456")
                .unwrap()
                .with_auth_response(vec![0; MAX_BINARY_PARAMETER_BYTES + 1])
                .is_err()
        );

        let complete = RegistrationCompleteParams::new("123456").unwrap();
        let mut form = RegistrationForm::new();
        form.push("code", "existing").unwrap();
        assert!(complete.append_to_form(&mut form).is_err());
        assert_eq!(form.len(), 1, "failed batch append must not add fields");
        assert_eq!(form.encode().unwrap().as_slice(), b"code=existing");
    }

    #[test]
    fn security_request_uses_observed_field_order_and_encodings() {
        let common = RegistrationCommonParams::new(
            PhoneNumber::new(1, "5551234567").unwrap(),
            "en",
            "US",
            "fdid",
            [0; 16],
            None,
            vec![1],
            vec![2],
        )
        .unwrap();
        let security = RegistrationSecurityParams::verify("6157")
            .unwrap()
            .with_wipe_token(b"A ~*".to_vec())
            .unwrap()
            .with_advertising_id("ad id")
            .unwrap();
        let request = assemble_security_request(&common, &security, None).unwrap();
        assert_eq!(request.url(), "https://v.whatsapp.net/v2/security");
        assert_eq!(
            request.body(),
            b"cc=1&in=5551234567&lg=en&lc=US&fdid=fdid&expid=AAAAAAAAAAAAAAAAAAAAAA&id=%01&backup_token=%02&code=6157&wipe_token=A%20~%2A&advertising_id=ad+id"
        );
        let debug = format!("{security:?}");
        assert!(!debug.contains("6157"));
        assert!(!debug.contains("A ~*"));
        assert!(!debug.contains("ad id"));
    }

    #[test]
    fn security_reset_and_atomic_validation_are_bounded() {
        assert!(RegistrationSecurityParams::verify("").is_err());
        assert!(RegistrationSecurityParams::reset("").is_err());
        assert!(
            RegistrationSecurityParams::verify("6157")
                .unwrap()
                .with_wipe_token(Vec::new())
                .is_err()
        );

        let security = RegistrationSecurityParams::reset("reset-token").unwrap();
        let mut form = RegistrationForm::new();
        form.push("reset", "existing").unwrap();
        assert!(security.append_to_form(&mut form).is_err());
        assert_eq!(form.len(), 1, "failed batch append must not add fields");
        assert_eq!(form.encode().unwrap().as_slice(), b"reset=existing");
    }

    #[test]
    fn key_bundle_uses_observed_base64url_and_integer_encodings() {
        assert_eq!(base64_url_no_pad(b""), "");
        assert_eq!(base64_url_no_pad(b"f"), "Zg");
        assert_eq!(base64_url_no_pad(b"fo"), "Zm8");
        assert_eq!(base64_url_no_pad(b"foo"), "Zm9v");
        assert_eq!(base64_url_no_pad(&[0xfb, 0xff, 0xef]), "-__v");

        let bundle = RegistrationKeyBundle::new(
            [0; 32],
            vec![0xfb, 0xff, 0xef],
            0x0102_0304,
            0x010203,
            b"foo".to_vec(),
            vec![0xff],
        )
        .unwrap();
        let mut form = RegistrationForm::new();
        bundle.append_to_form(&mut form).unwrap();
        let body = String::from_utf8(form.encode().unwrap().to_vec()).unwrap();
        assert_eq!(
            body,
            format!(
                "authkey={}&e_ident=-__v&e_keytype=BQ&e_regid=AQIDBA&e_skey_id=AQID&e_skey_val=Zm9v&e_skey_sig=_w",
                "A".repeat(43)
            )
        );
        let debug = format!("{bundle:?}");
        assert!(!debug.contains("-__v") && !debug.contains("Zm9v"));
    }

    #[test]
    fn key_bundle_validation_and_append_are_atomic() {
        assert!(RegistrationKeyBundle::new([0; 32], Vec::new(), 1, 1, vec![1], vec![2]).is_err());
        assert!(
            RegistrationKeyBundle::new([0; 32], vec![1], 1, 0x0100_0000, vec![1], vec![2]).is_err()
        );

        let bundle = RegistrationKeyBundle::new([0; 32], vec![1], 1, 1, vec![1], vec![2]).unwrap();
        let mut form = RegistrationForm::new();
        form.push("e_ident", "existing").unwrap();
        assert!(bundle.append_to_form(&mut form).is_err());
        assert_eq!(form.len(), 1, "failed batch append must not add fields");
        assert_eq!(form.encode().unwrap().as_slice(), b"e_ident=existing");
    }

    #[test]
    fn parses_control_fields_and_redacts_secrets() {
        let body = Zeroizing::new(
            br#"{
                "status":"sent",
                "reason":"waiting_for_sms",
                "retry_after":"30",
                "sms_wait":5,
                "voice_wait":"60",
                "flash_wait":0,
                "wipe_wait":"120",
                "length":6,
                "sms_length":6,
                "voice_length":6,
                "security_code_set":true,
                "secure_verifier":true,
                "login":"login-secret",
                "code":"123456",
                "wipe_token":"wipe-secret",
                "passkey_credential":"passkey-secret",
                "appeal_token":"appeal-secret",
                "edge_routing_info":"routing-secret"
            }"#
            .to_vec(),
        );
        let mut response = RegistrationHttpResponse::new(200, body)
            .unwrap()
            .parse_json()
            .unwrap();
        assert_eq!(response.status, RegistrationResponseStatus::Sent);
        assert_eq!(
            response.reason,
            Some(RegistrationResponseReason::WaitingForSms)
        );
        assert_eq!(response.retry_after, Some(Duration::from_secs(30)));
        assert_eq!(response.sms_wait, Some(Duration::from_secs(5)));
        assert_eq!(response.voice_wait, Some(Duration::from_secs(60)));
        assert_eq!(response.wipe_wait, Some(Duration::from_secs(120)));
        assert_eq!(response.code_length, Some(6));
        let debug = format!("{response:?}");
        for secret in [
            "login-secret",
            "123456",
            "wipe-secret",
            "passkey-secret",
            "appeal-secret",
            "routing-secret",
        ] {
            assert!(!debug.contains(secret));
        }

        let mut secrets = response.take_secrets();
        assert_eq!(secrets.take_login().unwrap().as_str(), "login-secret");
        assert_eq!(secrets.take_code().unwrap().as_str(), "123456");
        assert_eq!(secrets.take_wipe_token().unwrap().as_str(), "wipe-secret");
        assert_eq!(
            secrets.take_passkey_credential().unwrap().as_str(),
            "passkey-secret"
        );
        assert_eq!(
            secrets.take_appeal_token().unwrap().as_str(),
            "appeal-secret"
        );
        assert_eq!(
            secrets.take_edge_routing_info().unwrap().as_str(),
            "routing-secret"
        );
    }

    #[test]
    fn response_parser_is_bounded_and_forward_compatible() {
        let response = RegistrationHttpResponse::new(
            429,
            Zeroizing::new(br#"{"status":"new_status","reason":"new_reason"}"#.to_vec()),
        )
        .unwrap()
        .parse_json()
        .unwrap();
        assert_eq!(response.status, RegistrationResponseStatus::Other);
        assert_eq!(response.reason, Some(RegistrationResponseReason::Other));

        assert!(RegistrationHttpResponse::new(99, Zeroizing::new(Vec::new())).is_err());
        assert!(
            RegistrationHttpResponse::new(
                200,
                Zeroizing::new(vec![b'x'; MAX_RESPONSE_BODY_BYTES + 1])
            )
            .is_err()
        );
        assert!(
            RegistrationHttpResponse::new(
                200,
                Zeroizing::new(br#"{"status":"ok","retry_after":"not-a-number"}"#.to_vec())
            )
            .unwrap()
            .parse_json()
            .is_err()
        );
        assert!(
            RegistrationHttpResponse::new(200, Zeroizing::new(br#"{"reason":"blocked"}"#.to_vec()))
                .unwrap()
                .parse_json()
                .is_err()
        );
    }

    #[derive(Debug)]
    struct MockTransport;

    impl RegistrationTransport for MockTransport {
        fn send<'a>(
            &'a self,
            request: EncodedRegistrationRequest,
        ) -> RegistrationTransportFuture<'a> {
            Box::pin(async move {
                assert_eq!(request.url(), "https://v.whatsapp.net/v2/code");
                RegistrationHttpResponse::new(
                    200,
                    Zeroizing::new(br#"{"status":"sent","retry_after":"20"}"#.to_vec()),
                )
            })
        }
    }

    #[tokio::test]
    async fn transport_boundary_owns_secret_request_and_response_buffers() {
        let request = WHATSAPP_2_26_37_70
            .request(RegistrationEndpoint::Code, &RegistrationForm::new())
            .unwrap();
        let response = MockTransport.send(request).await.unwrap();
        assert_eq!(response.status_code(), 200);
        assert_eq!(
            response.parse_json().unwrap().retry_after,
            Some(Duration::from_secs(20))
        );
    }

    #[cfg(feature = "experimental-registration-https")]
    #[test]
    fn experimental_https_transport_has_bounded_configuration() {
        assert!(ExperimentalHttpsTransport::new(Duration::ZERO).is_err());
        assert!(ExperimentalHttpsTransport::new(Duration::from_secs(301)).is_err());
        let transport = ExperimentalHttpsTransport::new(Duration::from_secs(30)).unwrap();
        let debug = format!("{transport:?}");
        assert!(debug.contains("disabled"));
        assert!(!debug.contains("root"));
    }
}
