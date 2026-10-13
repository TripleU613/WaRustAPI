//! The saved registration attempt as JSON, with its secrets redacted, shared
//! by `lab-signup status --json`, `registration status --json` and the
//! `GET /v1/signup/status` store route.

use whatsapp_api::Error;
use whatsapp_api::registration::RegistrationAttempt;

/// Fields of a saved attempt that must never be printed: the phone number, and
/// the consent context's intermediate login, opaque pending selector and parent
/// consent URL (`RegistrationConsent`, kept only in encrypted storage). Matched
/// at any depth, so a field nested under a blocked reason is covered too.
const REDACTED_ATTEMPT_KEYS: [&str; 4] = [
    "phone",
    "registration_login",
    "pending",
    "parent_consent_url",
];

/// Replace every present value of [`REDACTED_ATTEMPT_KEYS`] with a marker;
/// `null` stays `null`, so presence remains visible without the value.
pub(crate) fn redact_attempt(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, field) in map.iter_mut() {
                if REDACTED_ATTEMPT_KEYS.contains(&key.as_str()) {
                    if !field.is_null() {
                        *field = serde_json::Value::String("<redacted>".into());
                    }
                } else {
                    redact_attempt(field);
                }
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(redact_attempt),
        _ => {}
    }
}

/// The attempt serialised with [`redact_attempt`] applied. Its `Debug` already
/// redacts the phone; serialised JSON does not until this runs.
pub(crate) fn redacted(attempt: &RegistrationAttempt) -> Result<serde_json::Value, Error> {
    let mut value = serde_json::to_value(attempt)
        .map_err(|e| Error::invalid("registration_attempt", format!("not serialisable: {e}")))?;
    redact_attempt(&mut value);
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A saved attempt's JSON keeps its state and history but never the
    /// phone number; available without experimental features.
    #[tokio::test]
    async fn the_attempt_json_never_carries_the_phone() {
        use std::sync::Arc;
        use whatsapp_api::registration::{ChallengeMethod, RegistrationService, UnsupportedFlow};
        use whatsapp_api::store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("r.redb"),
                account_id: whatsapp_api::model::ids::AccountId::new("t").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
                create: true,
            })
            .unwrap(),
        );
        let profile = whatsapp_api::ProtocolProfile::android_reference().unwrap();
        let service = RegistrationService::new(
            store,
            &profile,
            Arc::new(UnsupportedFlow::for_profile(&profile)),
        );
        service
            .begin(
                whatsapp_api::model::phone::PhoneNumber::new(1, "5550108765").unwrap(),
                ChallengeMethod::Sms,
            )
            .await
            .unwrap();
        let attempt = service.current().await.unwrap().unwrap();
        let value = redacted(&attempt).unwrap();
        let text = value.to_string();
        assert!(!text.contains("5550108765"), "{text}");
        assert_eq!(value["phone"], "<redacted>");
        assert!(
            value.get("state").is_some() && value.get("history").is_some(),
            "{text}"
        );
    }
}
