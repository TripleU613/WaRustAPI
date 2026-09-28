//! Implementation of the editable request profile's schema and validation.
//!
//! Add tunable inputs here, not inside the HTTP sender or CLI. Actual numbers,
//! tokens and overrides live in a private JSON file; never commit them here.
//! Unknown JSON keys fail fast so typos cannot silently change an experiment.
//! Protocol identity, TLS verification, no redirects/retries and the allowed
//! origin remain explicit invariants, not switches for bypassing safeguards.

use super::wire::{
    MAX_RESPONSE_BODY_BYTES, RegistrationCodeParams, RegistrationCommonParams, RegistrationForm,
    WHATSAPP_2_26_37_70,
};
use crate::{Error, Result, model::phone::PhoneNumber};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};
use zeroize::Zeroizing;

/// Explicit experimental profile, distinct from the pinned reference profile.
pub const LAB_PROFILE_ID: &str = "android-2.26.37.70-registration-lab";
/// Configuration-file budget, including secret inputs.
pub const MAX_CONFIG_BYTES: u64 = 64 * 1024;

/// Default delivery method; a CLI override is explicit for each begin action.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryMethod {
    /// SMS delivery.
    #[default]
    Sms,
    /// Voice call delivery.
    Voice,
}

/// All configurable signup inputs. Serialize only to private/encrypted storage.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LabRegistrationConfig {
    /// Country calling code and national digits, validated again after decoding.
    pub phone: PhoneNumber,
    /// Request language (e.g. en).
    pub language: String,
    /// Request locale (e.g. US or CA); not a GPS-location claim.
    pub locale: String,
    /// Phone-bound token derived from the exact comparison APK.
    pub token: Zeroizing<String>,
    /// Caller-selected registration ID; accepted Android range remains unverified.
    pub registration_id: u32,
    /// Signed-prekey ID, constrained by its three-byte wire encoding.
    pub signed_pre_key_id: u32,
    /// Default delivery method for the operator's begin action.
    #[serde(default)]
    pub method: DeliveryMethod,
    /// Client identification; no invented default User-Agent.
    #[serde(default)]
    pub client: ClientSettings,
    /// Request deadlines and response budget.
    #[serde(default)]
    pub transport: TransportSettings,
    /// Optional overrides for generated device prerequisites.
    #[serde(default)]
    pub device: DeviceSettings,
    /// Additional endpoint-specific form fields, encoded exactly once.
    #[serde(default)]
    pub fields: EndpointFields,
    /// Print sanitized request/response metadata, never bodies or secrets.
    #[serde(default = "enabled")]
    pub diagnostics: bool,
    /// Opt in to the comparison existence/method-selection preflight.
    /// False preserves previously frozen lab profiles.
    #[serde(default)]
    pub preflight: bool,
}

fn enabled() -> bool {
    true
}
fn profile_id() -> String {
    LAB_PROFILE_ID.into()
}

/// Identification settings. Version/hash/origin remain tied to the wire catalog.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ClientSettings {
    /// Must match the explicit lab profile.
    pub profile_id: String,
    /// Optional HTTP User-Agent, absent unless explicitly provided.
    pub user_agent: Option<Zeroizing<String>>,
}
impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            profile_id: profile_id(),
            user_agent: None,
        }
    }
}

/// Explicit comparison-lab request encoding; never supplies attestation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestBodyMode {
    /// Historical lab behavior; no envelope.
    #[default]
    ClearForm,
    /// Comparison APK encrypted envelope, without an H field.
    Enc,
    /// Comparison APK encrypted envelope with an empty H; no attestation.
    EncEmptyH,
}

/// Bounded transport configuration. TLS validation cannot be disabled.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct TransportSettings {
    /// Explicit experimental wire format, also shown in the offline preview.
    pub body_mode: RequestBodyMode,
    /// Whole-request deadline in milliseconds, including response streaming.
    pub timeout_ms: u64,
    /// Connection deadline in milliseconds, no longer than the whole request.
    pub connect_timeout_ms: u64,
    /// Response byte budget, at most the library's hard safety ceiling.
    pub max_response_bytes: usize,
    /// Bounded automatic reconnect of the messaging session after a classified
    /// recoverable connection loss. Never applies to registration requests.
    pub reconnect: ReconnectSettings,
    /// Minimum pause between consecutive action-queue sends in one heartbeat
    /// batch (milliseconds). `0` disables the delay (default). Set to a
    /// human-scale value (e.g. 5000–15000) to space out block/unblock/report/
    /// group actions rather than flushing them all at once.
    pub action_delay_ms: u64,
    /// Send a read receipt for every incoming direct or group message with
    /// readable content that a messaging session decrypts. Off by default: the
    /// official client marks messages read when a chat is on screen, and this
    /// client has no screen to infer that from.
    pub mark_read: bool,
}
impl Default for TransportSettings {
    fn default() -> Self {
        Self {
            body_mode: RequestBodyMode::default(),
            timeout_ms: 45_000,
            connect_timeout_ms: 30_000,
            max_response_bytes: MAX_RESPONSE_BODY_BYTES,
            reconnect: ReconnectSettings::default(),
            action_delay_ms: 0,
            mark_read: false,
        }
    }
}

/// Messaging-session reconnect bounds (docs/design/messaging.md, section 14).
/// Only classified recoverable failures (socket loss, keepalive timeout, I/O)
/// are retried, with capped jittered exponential backoff and a fresh epoch and
/// handshake each time. Refusals, revocation, missing registration, certificate
/// failures and uncertain login outcomes never reconnect automatically.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ReconnectSettings {
    /// Consecutive automatic attempts without reaching an active session
    /// before the runtime stops with the last error. `0` disables reconnect.
    pub max_attempts: u32,
    /// First delay in milliseconds; each further attempt doubles it.
    pub base_delay_ms: u64,
    /// Delay ceiling in milliseconds after doubling and jitter.
    pub max_delay_ms: u64,
}
impl Default for ReconnectSettings {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 2_000,
            max_delay_ms: 60_000,
        }
    }
}
impl ReconnectSettings {
    /// Hard ceiling on consecutive automatic attempts.
    pub const MAX_ATTEMPTS: u32 = 100;
    /// Hard ceiling on any single delay.
    pub const MAX_DELAY_MS: u64 = 300_000;

    /// Reject unbounded or inverted retry settings.
    pub fn validate(&self) -> Result<()> {
        if self.max_attempts > Self::MAX_ATTEMPTS
            || self.base_delay_ms == 0
            || self.base_delay_ms > self.max_delay_ms
            || self.max_delay_ms > Self::MAX_DELAY_MS
        {
            return Err(Error::invalid(
                "transport.reconnect",
                "invalid attempt/delay bounds",
            ));
        }
        Ok(())
    }

    /// Delay before automatic attempt number `attempt` (1-based): doubling from
    /// the base, capped, then reduced by up to 25% using `jitter` in `0..=255`.
    /// Deterministic given the jitter byte so tests can check the bounds.
    pub fn delay(&self, attempt: u32, jitter: u8) -> std::time::Duration {
        let exponent = attempt.saturating_sub(1).min(32);
        let scaled = self
            .base_delay_ms
            .saturating_mul(1u64 << exponent)
            .min(self.max_delay_ms);
        // jitter reduces the delay by 0..25%; never below one quarter of the base.
        let reduction = scaled / 4 * u64::from(jitter) / 255;
        std::time::Duration::from_millis(scaled - reduction)
    }
}
impl TransportSettings {
    /// Reject invalid or unsafe bounds before opening any network connection.
    pub fn validate(&self) -> Result<()> {
        if self.timeout_ms == 0
            || self.timeout_ms > 300_000
            || self.connect_timeout_ms == 0
            || self.connect_timeout_ms > self.timeout_ms
            || self.max_response_bytes == 0
            || self.max_response_bytes > MAX_RESPONSE_BODY_BYTES
        {
            return Err(Error::invalid(
                "transport",
                "invalid timeout/connect-timeout/response-size bounds",
            ));
        }
        if self.action_delay_ms > 300_000 {
            return Err(Error::invalid(
                "transport",
                "action_delay_ms exceeds 300000 ms (5 minutes)",
            ));
        }
        self.reconnect.validate()
    }
}

/// Fresh-install identifiers. None means generate once, then persist and reuse.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct DeviceSettings {
    /// Canonical UUID string; validated by the common-field assembler.
    pub facebook_device_id: Option<Zeroizing<String>>,
    /// UUID bytes for the experiment identifier.
    pub experiment_id: Option<[u8; 16]>,
    /// UUID bytes for the access session.
    pub access_session_id: Option<[u8; 16]>,
    /// Opaque recovery/device token bytes.
    pub recovery_token: Option<Zeroizing<Vec<u8>>>,
    /// Opaque backup token bytes.
    pub backup_token: Option<Zeroizing<Vec<u8>>>,
}

/// Extension point for future observed fields, without editing the sender.
/// Values are plain strings, not pre-percent-encoded. Identity, phone, token,
/// OTP/PIN and generated-key fields cannot be overridden here.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct EndpointFields {
    /// Extra `/v2/exist` fields, separate from code-request telemetry.
    pub exist: BTreeMap<String, Zeroizing<String>>,
    /// Raw byte-valued `/v2/exist` fields.
    #[serde(deserialize_with = "deserialize_byte_fields")]
    pub exist_bytes: BTreeMap<String, Zeroizing<Vec<u8>>>,
    /// Extra `/v2/code` fields (e.g. context or type).
    pub code: BTreeMap<String, Zeroizing<String>>,
    /// Extra `/v2/register` fields.
    pub register: BTreeMap<String, Zeroizing<String>>,
    /// Extra `/v2/security` fields; no PIN reset/wipe operation is exposed.
    pub security: BTreeMap<String, Zeroizing<String>>,
    /// Extra `/v2/challenge` fields (email second factor).
    #[serde(default)]
    pub challenge: BTreeMap<String, Zeroizing<String>>,
    /// Raw additional-parameter bytes for `/v2/code` (`Hvq.A06` encoding).
    /// JSON accepts UTF-8 strings or arrays of byte integers, never pre-encoding.
    #[serde(deserialize_with = "deserialize_byte_fields")]
    pub code_bytes: BTreeMap<String, Zeroizing<Vec<u8>>>,
    /// Raw additional-parameter bytes for `/v2/register`.
    #[serde(deserialize_with = "deserialize_byte_fields")]
    pub register_bytes: BTreeMap<String, Zeroizing<Vec<u8>>>,
    /// Raw additional-parameter bytes for `/v2/security`.
    #[serde(deserialize_with = "deserialize_byte_fields")]
    pub security_bytes: BTreeMap<String, Zeroizing<Vec<u8>>>,
}

fn deserialize_byte_fields<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, Zeroizing<Vec<u8>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Input {
        Text(Zeroizing<String>),
        Bytes(Zeroizing<Vec<u8>>),
    }
    let fields = BTreeMap::<String, Input>::deserialize(deserializer)?;
    Ok(fields
        .into_iter()
        .map(|(name, value)| {
            let bytes = match value {
                Input::Text(text) => Zeroizing::new(text.as_bytes().to_vec()),
                Input::Bytes(bytes) => bytes,
            };
            (name, bytes)
        })
        .collect())
}

const RESERVED: &[&str] = &[
    "cc",
    "in",
    "lc",
    "lg",
    "fdid",
    "expid",
    "access_session_id",
    "id",
    "backup_token",
    "token",
    "method",
    "code",
    "security_code",
    "reset",
    "wipe_token",
    "authkey",
    "e_ident",
    "e_keytype",
    "e_regid",
    "e_skey_id",
    "e_skey_val",
    "e_skey_sig",
];

/// Validate and append experimental fields without replacing authoritative inputs.
pub(crate) fn append_fields(
    form: &mut RegistrationForm,
    fields: &BTreeMap<String, Zeroizing<String>>,
) -> Result<()> {
    for (name, value) in fields {
        if RESERVED.contains(&name.as_str()) {
            return Err(Error::invalid(
                "fields",
                "extra field collides with a protected input",
            ));
        }
        match name.as_str() {
            "clicked_education_link" | "manage_call_permission" | "call_log_permission"
                if !matches!(value.as_str(), "true" | "false") =>
            {
                return Err(Error::invalid(
                    "fields",
                    "permission/education flags must be true or false strings; omit to leave unset",
                ));
            }
            "type" if !matches!(value.as_str(), "0" | "1") => {
                return Err(Error::invalid(
                    "fields",
                    "comparison registration type must be 0 (PHONE) or 1 (WA_TWO_FA_CONTACT_POINT)",
                ));
            }
            "client_start_message" | "auth_response" => {
                let bytes = value.as_bytes();
                if bytes.len() % 4 == 1
                    || !bytes
                        .iter()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
                {
                    return Err(Error::invalid(
                        "fields",
                        "binary form values must use unpadded URL-safe Base64, not raw or percent-encoded bytes",
                    ));
                }
            }
            _ => {}
        }
        form.push(name, value.as_str())?;
    }
    Ok(())
}

pub(crate) fn append_byte_fields(
    form: &mut RegistrationForm,
    fields: &BTreeMap<String, Zeroizing<Vec<u8>>>,
) -> Result<()> {
    for (name, value) in fields {
        if RESERVED.contains(&name.as_str()) {
            return Err(Error::invalid(
                "fields",
                "additional bytes collide with a protected input",
            ));
        }
        form.push_bytes(name, value)?;
    }
    Ok(())
}

impl LabRegistrationConfig {
    /// Offline validation shared by preview, CLI and the persisted factory.
    pub fn validate(&self) -> Result<()> {
        if PhoneNumber::new(self.phone.country_code(), self.phone.national_number())? != self.phone
        {
            return Err(Error::invalid("phone", "must use canonical digits"));
        }
        if self.signed_pre_key_id > 0x00ff_ffff {
            return Err(Error::invalid("signed_pre_key_id", "must fit 24 bits"));
        }
        if self.client.profile_id != LAB_PROFILE_ID {
            return Err(Error::invalid(
                "client.profile_id",
                "unsupported lab profile",
            ));
        }
        self.transport.validate()?;
        if let Some(value) = &self.device.facebook_device_id
            && (value.len() != 36
                || !value.bytes().enumerate().all(|(i, b)| {
                    if [8, 13, 18, 23].contains(&i) {
                        b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                }))
        {
            return Err(Error::invalid(
                "device.facebook_device_id",
                "must be a canonical UUID",
            ));
        }
        if let Some(value) = &self.client.user_agent
            && (value.is_empty()
                || value.len() > 512
                || !value.bytes().all(|b| (32..=126).contains(&b)))
        {
            return Err(Error::invalid(
                "client.user_agent",
                "must contain 1..=512 printable ASCII bytes",
            ));
        }
        RegistrationCodeParams::new(self.token.as_str(), "sms")?;
        RegistrationCommonParams::new(
            self.phone.clone(),
            self.language.clone(),
            self.locale.clone(),
            self.device
                .facebook_device_id
                .as_deref()
                .map(|s| s.as_str())
                .unwrap_or("00000000-0000-4000-8000-000000000000"),
            self.device.experiment_id.unwrap_or([0; 16]),
            Some(self.device.access_session_id.unwrap_or([0; 16])),
            self.device
                .recovery_token
                .as_ref()
                .map(|v| v.to_vec())
                .unwrap_or(vec![0; 20]),
            self.device
                .backup_token
                .as_ref()
                .map(|v| v.to_vec())
                .unwrap_or(vec![0; 20]),
        )?;
        for (fields, bytes) in [
            (&self.fields.exist, &self.fields.exist_bytes),
            (&self.fields.code, &self.fields.code_bytes),
            (&self.fields.register, &self.fields.register_bytes),
            (&self.fields.security, &self.fields.security_bytes),
        ] {
            let mut form = RegistrationForm::new();
            append_fields(&mut form, fields)?;
            append_byte_fields(&mut form, bytes)?;
        }
        Ok(())
    }

    /// Safe local overview. Showing the phone is an explicit operator choice.
    pub fn preview(&self, show_phone: bool) -> serde_json::Value {
        let observed_code_fields = [
            "reason",
            "client_metrics",
            "mcc",
            "mnc",
            "sim_mcc",
            "sim_mnc",
            "education_screen_displayed",
            "prefer_sms_over_flash",
            "feo2_query_status",
        ]
        .into_iter()
        .map(|name| {
            let status = if self.fields.code_bytes.contains_key(name) {
                "configured on observed byte path; value not verified"
            } else if self.fields.code.contains_key(name) {
                "configured on text path; APK uses byte encoding"
            } else {
                "absent"
            };
            (name, status)
        })
        .collect::<BTreeMap<_, _>>();
        serde_json::json!({
            "mode": "preview: no network, no store writes", "method": self.method, "phone": if show_phone { self.phone.e164() } else { "<redacted>".into() },
            "location": {"country_code": self.phone.country_code(), "language": self.language, "locale": self.locale},
            "client": {"profile": self.client.profile_id, "version": WHATSAPP_2_26_37_70.version_name,
                "apk_sha256": WHATSAPP_2_26_37_70.apk_sha256, "user_agent": if self.client.user_agent.is_some() { "<configured, redacted>" } else { "absent" }},
            "transport": self.transport, "origin": WHATSAPP_2_26_37_70.base_url,
            "extra_fields": {"code": self.fields.code.keys().collect::<Vec<_>>(), "register": self.fields.register.keys().collect::<Vec<_>>(), "security": self.fields.security.keys().collect::<Vec<_>>()},
            "additional_byte_fields": {"code": self.fields.code_bytes.keys().collect::<Vec<_>>(), "register": self.fields.register_bytes.keys().collect::<Vec<_>>(), "security": self.fields.security_bytes.keys().collect::<Vec<_>>()},
            "device_overrides": {"facebook_device_id": self.device.facebook_device_id.is_some(), "experiment_id": self.device.experiment_id.is_some(),
                "access_session_id": self.device.access_session_id.is_some(), "recovery_token": self.device.recovery_token.is_some(), "backup_token": self.device.backup_token.is_some()},
            "diagnostics": self.diagnostics, "preflight": self.preflight,
            "exist_fields": self.fields.exist.keys().collect::<Vec<_>>(),
            "exist_byte_fields": self.fields.exist_bytes.keys().collect::<Vec<_>>(),
            "token": "<redacted>", "key_ids": "<redacted>",
            "compatibility": {
                "observed_code_additional_fields": observed_code_fields,
                "field_coverage_note": "Observed on the inspected APK branch, not proven server requirements; do not invent values",
                "user_agent_matches_observed_presence": self.client.user_agent.is_some(),
                "body_mode": self.transport.body_mode,
                "action_delay_ms": self.transport.action_delay_ms,
                "attestation": "not implemented; no fabricated attestation",
                "server_acceptance": "unverified",
                "note": "Configured values do not prove genuine device provenance or server acceptance"
            },
            "invariants": ["TLS verification on", "fixed comparison origin", "no redirects/proxies/automatic retries", "no reset/wipe", "live actions require explicit authorization"]
        })
    }
}

macro_rules! redacted_debug {
    ($($name:ty),*) => { $(impl fmt::Debug for $name {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct(stringify!($name)).finish_non_exhaustive() }
    })* };
}
redacted_debug!(
    LabRegistrationConfig,
    ClientSettings,
    DeviceSettings,
    EndpointFields
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_distinguishes_absent_text_and_byte_evidence_without_values() {
        let mut cfg = config();
        // Clear template-populated code_bytes so observed fields start absent.
        cfg.fields.code_bytes.clear();
        cfg.fields
            .code
            .insert("reason".into(), Zeroizing::new("private-reason".into()));
        cfg.fields.code_bytes.insert(
            "client_metrics".into(),
            Zeroizing::new(b"private-metrics".to_vec()),
        );
        let preview = cfg.preview(false);
        let fields = &preview["compatibility"]["observed_code_additional_fields"];
        assert_eq!(fields["mcc"], "absent");
        assert_eq!(
            fields["reason"],
            "configured on text path; APK uses byte encoding"
        );
        assert_eq!(
            fields["client_metrics"],
            "configured on observed byte path; value not verified"
        );
        let text = preview.to_string();
        assert!(!text.contains("private-reason"));
        assert!(!text.contains("private-metrics"));
    }

    #[test]
    fn byte_field_strings_match_arrays_and_keep_wire_encoding() {
        for endpoint in ["code_bytes", "register_bytes", "security_bytes"] {
            let mut json = serde_json::to_value(config()).unwrap();
            json["fields"][endpoint] = serde_json::json!({"example": " ~*é%20"});
            let text: LabRegistrationConfig = serde_json::from_value(json.clone()).unwrap();
            json["fields"][endpoint]["example"] = serde_json::json!(b" ~*\xc3\xa9%20".to_vec());
            let bytes: LabRegistrationConfig = serde_json::from_value(json).unwrap();
            assert_eq!(text, bytes);
            text.validate().unwrap();
            let serialized = serde_json::to_value(&text).unwrap();
            assert!(serialized["fields"][endpoint]["example"].is_array());
            assert_eq!(
                serde_json::from_value::<LabRegistrationConfig>(serialized).unwrap(),
                text
            );
        }
        let fields: EndpointFields =
            serde_json::from_str(r#"{"code_bytes":{"example":" ~*é%20"}}"#).unwrap();
        let mut form = RegistrationForm::new();
        append_byte_fields(&mut form, &fields.code_bytes).unwrap();
        assert_eq!(
            form.encode().unwrap().as_slice(),
            b"example=%20~%2A%C3%A9%2520"
        );
    }

    #[test]
    fn byte_fields_reject_non_string_non_byte_inputs() {
        for invalid in [
            serde_json::json!(null),
            serde_json::json!(true),
            serde_json::json!(256),
            serde_json::json!([256]),
            serde_json::json!([-1]),
            serde_json::json!([1.5]),
            serde_json::json!({"secret": "value"}),
        ] {
            let json = serde_json::json!({"code_bytes": {"example": invalid}});
            assert!(serde_json::from_value::<EndpointFields>(json).is_err());
        }
    }

    fn config() -> LabRegistrationConfig {
        let mut value: LabRegistrationConfig =
            serde_json::from_str(super::super::request::TEMPLATE_JSON).unwrap();
        value.token = Zeroizing::new("test-secret-token".into());
        // Existing fixtures intentionally exercise backward-compatible profiles.
        value.preflight = false;
        value
    }

    #[test]
    fn template_is_parseable_but_requires_real_inputs() {
        let template: LabRegistrationConfig =
            serde_json::from_str(super::super::request::TEMPLATE_JSON).unwrap();
        assert!(template.validate().is_err());
        config().validate().unwrap();
    }

    #[test]
    fn request_body_mode_roundtrips_previews_and_rejects_typos() {
        for (name, mode) in [
            ("clear_form", RequestBodyMode::ClearForm),
            ("enc", RequestBodyMode::Enc),
            ("enc_empty_h", RequestBodyMode::EncEmptyH),
        ] {
            let mut json = serde_json::to_value(config()).unwrap();
            json["transport"]["body_mode"] = serde_json::json!(name);
            let cfg: LabRegistrationConfig = serde_json::from_value(json).unwrap();
            assert_eq!(cfg.transport.body_mode, mode);
            cfg.validate().unwrap();
            assert_eq!(cfg.preview(false)["compatibility"]["body_mode"], name);
            assert_eq!(
                serde_json::to_value(&cfg).unwrap()["transport"]["body_mode"],
                name
            );
        }
        let mut json = serde_json::to_value(config()).unwrap();
        json["transport"]
            .as_object_mut()
            .unwrap()
            .remove("body_mode");
        let old: LabRegistrationConfig = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(old.transport.body_mode, RequestBodyMode::ClearForm);
        json["transport"]["body_mode"] = serde_json::json!("enc_typo");
        assert!(serde_json::from_value::<LabRegistrationConfig>(json).is_err());
    }

    #[test]
    fn reconnect_settings_default_bound_and_reject_unknown_fields() {
        let mut json = serde_json::to_value(config()).unwrap();
        json["transport"]
            .as_object_mut()
            .unwrap()
            .remove("reconnect");
        let old: LabRegistrationConfig = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(old.transport.reconnect, ReconnectSettings::default());
        assert_eq!(old.transport.reconnect.max_attempts, 5);
        json["transport"]["reconnect"] = serde_json::json!({"max_attempts": 0});
        let disabled: LabRegistrationConfig = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(disabled.transport.reconnect.max_attempts, 0);
        assert_eq!(disabled.transport.reconnect.base_delay_ms, 2_000);
        disabled.validate().unwrap();
        json["transport"]["reconnect"] = serde_json::json!({"retries": 3});
        assert!(serde_json::from_value::<LabRegistrationConfig>(json).is_err());

        let policy = ReconnectSettings {
            max_attempts: 5,
            base_delay_ms: 1_000,
            max_delay_ms: 5_000,
        };
        assert_eq!(policy.delay(1, 0).as_millis(), 1_000);
        assert_eq!(policy.delay(2, 0).as_millis(), 2_000);
        assert_eq!(policy.delay(3, 0).as_millis(), 4_000);
        assert_eq!(policy.delay(4, 0).as_millis(), 5_000, "capped");
        assert_eq!(policy.delay(40, 0).as_millis(), 5_000, "no overflow");
        assert_eq!(policy.delay(1, 255).as_millis(), 750, "full jitter is -25%");
        for jitter in [0u8, 1, 127, 254, 255] {
            let d = policy.delay(4, jitter).as_millis();
            assert!((3_750..=5_000).contains(&d), "{jitter}: {d}");
        }
    }

    #[test]
    fn legacy_config_defaults_and_typos_fail_fast() {
        let mut json = serde_json::to_value(config()).unwrap();
        for key in ["device", "method", "diagnostics"] {
            json.as_object_mut().unwrap().remove(key);
        }
        let old: LabRegistrationConfig = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(old, config());
        json["langauge"] = serde_json::json!("en");
        assert!(serde_json::from_value::<LabRegistrationConfig>(json).is_err());
        let mut json = serde_json::to_value(config()).unwrap();
        json["client"]["useragent"] = serde_json::json!("typo");
        assert!(serde_json::from_value::<LabRegistrationConfig>(json).is_err());
    }

    #[test]
    fn protected_fields_and_unsafe_settings_are_rejected() {
        for name in RESERVED {
            let mut value = config();
            value
                .fields
                .code
                .insert((*name).into(), Zeroizing::new("override".into()));
            assert!(value.validate().is_err(), "{name}");
        }
        let mut value = config();
        value.client.user_agent = Some(Zeroizing::new("agent\r\nInjected: value".into()));
        assert!(value.validate().is_err());
        let mut value = config();
        value.transport.timeout_ms = 0;
        assert!(value.validate().is_err());
        let mut value = config();
        value.transport.max_response_bytes = MAX_RESPONSE_BODY_BYTES + 1;
        assert!(value.validate().is_err());
        let mut value = config();
        value.transport.reconnect.base_delay_ms = 0;
        assert!(value.validate().is_err());
        let mut value = config();
        value.transport.reconnect.max_delay_ms = value.transport.reconnect.base_delay_ms - 1;
        assert!(value.validate().is_err());
        let mut value = config();
        value.transport.reconnect.max_attempts = ReconnectSettings::MAX_ATTEMPTS + 1;
        assert!(value.validate().is_err());
        let mut value = config();
        value.transport.reconnect.max_delay_ms = ReconnectSettings::MAX_DELAY_MS + 1;
        assert!(value.validate().is_err());
        let mut value = config();
        value.transport.action_delay_ms = 300_001;
        assert!(value.validate().is_err());
        let mut value = config();
        value.device.facebook_device_id = Some(Zeroizing::new("invalid".into()));
        assert!(value.validate().is_err());
    }

    #[test]
    fn preview_and_debug_never_include_secret_values() {
        let mut value = config();
        value.client.user_agent = Some(Zeroizing::new("private-agent".into()));
        value
            .fields
            .code
            .insert("context".into(), Zeroizing::new("private-context".into()));
        value.validate().unwrap();
        let output = format!(
            "{} {value:?} {:?} {:?}",
            value.preview(false),
            value.client,
            value.fields
        );
        for secret in [
            "2025550100",
            "test-secret-token",
            "private-agent",
            "private-context",
        ] {
            assert!(!output.contains(secret));
        }
        assert!(value.preview(true).to_string().contains("2025550100"));
        assert!(
            !value
                .preview(true)
                .to_string()
                .contains("test-secret-token")
        );
    }

    #[test]
    fn extra_fields_encode_plain_values_once() {
        let fields = BTreeMap::from([("context".into(), Zeroizing::new("a+b &c%20".into()))]);
        let mut form = RegistrationForm::new();
        append_fields(&mut form, &fields).unwrap();
        assert_eq!(
            form.encode().unwrap().as_slice(),
            b"context=a%2Bb+%26c%2520"
        );
    }

    #[test]
    fn observed_extra_field_encodings_fail_fast() {
        for (name, value) in [
            ("manage_call_permission", "1"),
            ("clicked_education_link", "False"),
            ("call_log_permission", ""),
            ("type", "PHONE"),
            ("type", "2"),
            ("client_start_message", "YQ=="),
            ("auth_response", "a+b/"),
            ("client_start_message", "x"),
        ] {
            let mut cfg = config();
            cfg.fields
                .code
                .insert(name.into(), Zeroizing::new(value.into()));
            assert!(cfg.validate().is_err(), "{name}");
        }
        let mut cfg = config();
        for (name, value) in [
            ("manage_call_permission", "false"),
            ("type", "0"),
            ("client_start_message", "YQ"),
        ] {
            cfg.fields
                .code
                .insert(name.into(), Zeroizing::new(value.into()));
        }
        cfg.validate().unwrap();
    }

    #[test]
    fn additional_bytes_reject_collisions_and_preview_only_names() {
        let mut cfg = config();
        cfg.fields.code_bytes.insert(
            "client_metrics".into(),
            Zeroizing::new(b"private-metrics".to_vec()),
        );
        cfg.validate().unwrap();
        let preview = cfg.preview(false).to_string();
        assert!(preview.contains("client_metrics"));
        assert!(!preview.contains("private-metrics"));
        cfg.fields
            .code
            .insert("client_metrics".into(), Zeroizing::new("duplicate".into()));
        assert!(cfg.validate().is_err());
        cfg.fields.code.clear();
        for name in RESERVED {
            let mut cfg = config();
            cfg.fields
                .code_bytes
                .insert((*name).into(), Zeroizing::new(b"override".to_vec()));
            assert!(cfg.validate().is_err());
        }
    }
}
