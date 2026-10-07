//! Editable request profile — values here, plumbing in `request_config.rs`.
//!
//! Export with `lab-signup template`, then edit the private JSON without rebuilding.
//! This template carries the request recipe that reached server-reported SMS dispatch with
//! the 2.26.37.70 profile (enc_empty_h body, unknown-device User-Agent, eight observed code fields).
//! The token must be derived per number from 2.26.37.70 material (`whatsapp-cli token derive`);
//! it is still not a claim of genuine device provenance.
//! Add APK-observed optional fields to the endpoint maps as evidence improves.
//! Keep real numbers/tokens out of this tracked file. Null device values mean
//! generate once and persist; they do not mean rotate on every request.
//!
//! Observed optional `/v2/code` fields (bridge A09): context, advertising_id,
//! clicked_education_link, manage_call_permission, call_log_permission,
//! client_start_message, type. Leave absent until their real producers and
//! branch conditions are understood; existence does not prove requiredness.
//! `type` is "0" for PHONE or "1" for WA_TWO_FA_CONTACT_POINT (HBS enum).
//! Permission flags are "true"/"false" strings, not "0"/"1"; absent means unset.
//! Binary client_start_message/auth_response values use unpadded URL-safe Base64.
//!
//! Caller-provided additional parameters (`I4D.A0H/A0K`, `Hvq.A06`) belong in
//! code_bytes as UTF-8 strings or raw byte arrays: reason, client_metrics,
//! mcc/mnc, sim_mcc/sim_mnc,
//! education_screen_displayed, prefer_sms_over_flash, feo2_query_status, etc.
//! Supply observed/real inputs, not invented telemetry or challenge tokens.

pub use super::request_config::{
    ClientSettings, DeliveryMethod, DeviceSettings, EndpointFields, IdentityChangePolicy,
    LAB_PROFILE_ID, LabRegistrationConfig, MAX_CONFIG_BYTES, ReconnectSettings, RequestBodyMode,
    TransportSettings,
};
#[cfg(feature = "experimental-signal-crypto")]
pub(crate) use super::request_config::{append_byte_fields, append_fields};

/// APK `0re.A00` exception fallback for unavailable build properties.
/// Explicit experiment only; not a claim that this client runs on Android.
pub const UNKNOWN_DEVICE_USER_AGENT: &str =
    "WhatsApp/2.26.37.70 Android/unknown Device/unknown-unknown";

/// Starter profile. Placeholder credentials deliberately fail live validation.
/// Locale is a locale, not GPS; no invented model, carrier or attestation values.
pub const TEMPLATE_JSON: &str = r#"{
  "phone": { "country_code": 1, "national_number": "2025550100" },
  "language": "en",
  "locale": "US",
  "token": "",
  "registration_id": 0,
  "signed_pre_key_id": 0,
  "method": "sms",
  "client": {
    "profile_id": "android-2.26.37.70-registration-lab",
    "user_agent": "WhatsApp/2.26.37.70 Android/unknown Device/unknown-unknown"
  },
  "device": {
    "facebook_device_id": null,
    "experiment_id": null,
    "access_session_id": null,
    "recovery_token": null,
    "backup_token": null
  },
  "fields": {
    "exist": {}, "exist_bytes": {},
    "code": {}, "register": {}, "security": {},
    "code_bytes": {
      "mcc": "000",
      "mnc": "000",
      "sim_mcc": "000",
      "sim_mnc": "000",
      "education_screen_displayed": "false",
      "prefer_sms_over_flash": "false",
      "feo2_query_status": "did_not_query",
      "client_metrics": "{\"attempts\":1}"
    },
    "register_bytes": {}, "security_bytes": {}
  },
  "transport": {
    "body_mode": "enc_empty_h",
    "timeout_ms": 45000,
    "connect_timeout_ms": 30000,
    "max_response_bytes": 1048576,
    "action_delay_ms": 0,
    "mark_read": false,
    "identity_changes": "pinned",
    "reconnect": { "max_attempts": 5, "base_delay_ms": 2000, "max_delay_ms": 60000 }
  },
  "diagnostics": true,
  "preflight": false
}"#;
