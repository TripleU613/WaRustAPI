//! Minimal primary login assembled from known account/configuration facts.
//! Comparison LX/1LC.AAU (identity), LX/1Ed.AAU (version/locale), LX/1IC.AAU
//! (active/short connection and reason). Unknown Android hardware is omitted.

use crate::proto::wa_web_protobufs_wa6::{
    ClientPayload,
    client_payload::{
        ConnectReason, UserAgent,
        user_agent::{AppVersion, Platform},
    },
};
use crate::{Error, Result, registration::request::LabRegistrationConfig, session::AccountRecord};
use prost::Message;
use zeroize::Zeroizing;

/// Observed-credential name holding the app version an account registered
/// with (`2.26.43.72`), written by signup on completion.
pub const REGISTERED_APP_VERSION: &str = "registered_app_version";

/// `X.Y.Z.W` from a `WhatsApp/X.Y.Z.W …` User-Agent or a bare version.
pub fn parse_app_version(text: &str) -> Option<[u32; 4]> {
    let version = text.strip_prefix("WhatsApp/").unwrap_or(text);
    let version = version.split_whitespace().next()?;
    let parts: Vec<u32> = version
        .split('.')
        .map(|p| p.parse().ok())
        .collect::<Option<_>>()?;
    parts.try_into().ok()
}

/// Version selected by the comparison driver's saved recipe: a consistent
/// recorded registration build, else the configured User-Agent's build, else
/// the historical comparison default. This is a local selection contract, not
/// proof that matching a registration build prevents account restrictions.
/// Explicit malformed/conflicting facts must not silently become another build.
fn login_app_version(account: &AccountRecord, config: &LabRegistrationConfig) -> Result<[u32; 4]> {
    let mut saved = None;
    for credential in account
        .credentials
        .iter()
        .filter(|c| c.name == REGISTERED_APP_VERSION)
    {
        let version = std::str::from_utf8(&credential.value)
            .ok()
            .and_then(parse_app_version)
            .ok_or_else(|| Error::invalid("login", "saved app version is malformed"))?;
        if saved.is_some_and(|previous| previous != version) {
            return Err(Error::invalid("login", "saved app versions conflict"));
        }
        saved = Some(version);
    }
    if let Some(saved) = saved {
        return Ok(saved);
    }
    match config.client.user_agent.as_deref() {
        Some(ua) => parse_app_version(ua).ok_or_else(|| {
            Error::invalid("login", "configured User-Agent has no valid app version")
        }),
        // Genuine absence retains the historical comparison recipe. An
        // explicitly invalid value is not absence and never reaches this path.
        None => Ok([2, 26, 37, 70]),
    }
}

/// Build without inventing Android hardware, attestation or companion pairing.
/// This is an explicitly minimal experiment, not a complete APK login payload.
pub fn primary_login(
    account: &AccountRecord,
    config: &LabRegistrationConfig,
) -> Result<Zeroizing<Vec<u8>>> {
    config.validate()?;
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::invalid(
            "login",
            "requires matching registered account",
        ));
    }
    let login = account
        .credentials
        .iter()
        .find(|value| value.name == "login")
        .ok_or_else(|| Error::invalid("login", "no server-issued login"))?;
    let text = std::str::from_utf8(&login.value)
        .map_err(|_| Error::invalid("login", "invalid login encoding"))?;
    if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::invalid(
            "login",
            "server-issued login is not a numeric primary identity",
        ));
    }
    let username: u64 = text
        .parse()
        .map_err(|_| Error::invalid("login", "numeric login out of range"))?;
    if text != config.phone.protocol_digits() {
        return Err(Error::invalid(
            "login",
            "non-phone login needs a verified product/JID mapping",
        ));
    }
    let [primary, secondary, tertiary, quaternary] = login_app_version(account, config)?;
    let payload = ClientPayload {
        username: Some(username),
        passive: Some(false),
        short_connect: Some(false),
        connect_reason: Some(ConnectReason::UserActivated as i32),
        user_agent: Some(UserAgent {
            platform: Some(Platform::Android as i32),
            app_version: Some(AppVersion {
                primary: Some(primary),
                secondary: Some(secondary),
                tertiary: Some(tertiary),
                quaternary: Some(quaternary),
                ..Default::default()
            }),
            locale_language_iso6391: Some(config.language.clone()),
            locale_country_iso31661_alpha2: Some(config.locale.clone()),
            ..Default::default()
        }),
        ..Default::default()
    };
    Ok(Zeroizing::new(payload.encode_to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::time::Timestamp,
        session::{ObservedCredential, RegistrationStatus},
    };
    #[test]
    fn uses_saved_login_and_omits_unknown_device_claims() {
        let mut cfg: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        cfg.token = Zeroizing::new("offline-fixture".into());
        let mut account = AccountRecord {
            phone: Some(cfg.phone.clone()),
            registration: RegistrationStatus::Registered,
            credentials: vec![ObservedCredential {
                name: "login".into(),
                value: Zeroizing::new(cfg.phone.protocol_digits().into_bytes()),
                observed_at: Timestamp(0),
                source_step: None,
            }],
            ..Default::default()
        };
        let bytes = primary_login(&account, &cfg).unwrap();
        let payload = ClientPayload::decode(bytes.as_slice()).unwrap();
        assert_eq!(
            payload.username,
            Some(cfg.phone.protocol_digits().parse().unwrap())
        );
        assert!(payload.device_pairing_data.is_none());
        assert!(payload.web_info.is_none());
        let ua = payload.user_agent.unwrap();
        assert!(ua.device.is_none());
        assert!(ua.os_version.is_none());
        assert_eq!(ua.platform, Some(0));
        assert_eq!(ua.app_version.unwrap().quaternary, Some(70));
        // A recorded registration build wins over the config's User-Agent.
        account.credentials.push(ObservedCredential {
            name: REGISTERED_APP_VERSION.into(),
            value: Zeroizing::new(b"2.26.43.72".to_vec()),
            observed_at: Timestamp(0),
            source_step: None,
        });
        let payload =
            ClientPayload::decode(primary_login(&account, &cfg).unwrap().as_slice()).unwrap();
        let version = payload.user_agent.unwrap().app_version.unwrap();
        assert_eq!((version.tertiary, version.quaternary), (Some(43), Some(72)));
        account.credentials.pop();
        assert_eq!(
            parse_app_version("WhatsApp/2.26.43.72 Android/unknown"),
            Some([2, 26, 43, 72])
        );
        assert_eq!(parse_app_version("WhatsApp/2.26"), None);
        account.credentials[0].value = Zeroizing::new(b"not-numeric".to_vec());
        assert!(primary_login(&account, &cfg).is_err());
        account.credentials.clear();
        assert!(primary_login(&account, &cfg).is_err());
    }
    fn version_fixture() -> (LabRegistrationConfig, AccountRecord) {
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = Zeroizing::new("offline-fixture".into());
        let account = AccountRecord {
            phone: Some(config.phone.clone()),
            registration: RegistrationStatus::Registered,
            credentials: vec![ObservedCredential {
                name: "login".into(),
                value: Zeroizing::new(config.phone.protocol_digits().into_bytes()),
                observed_at: Timestamp(0),
                source_step: None,
            }],
            ..Default::default()
        };
        (config, account)
    }
    fn version_credential(value: &[u8]) -> ObservedCredential {
        ObservedCredential {
            name: REGISTERED_APP_VERSION.into(),
            value: Zeroizing::new(value.to_vec()),
            observed_at: Timestamp(0),
            source_step: None,
        }
    }

    #[test]
    fn explicit_saved_version_errors_never_fall_back_or_disclose_values() {
        let (config, original) = version_fixture();
        for value in [
            b"private-malformed-version".as_slice(),
            b"2.26.43",
            &[0xff, 0xfe],
        ] {
            let mut account = original.clone();
            account.credentials.push(version_credential(value));
            let error = primary_login(&account, &config).unwrap_err();
            assert_eq!(
                error.to_string(),
                "invalid input for `login`: saved app version is malformed"
            );
            assert!(!format!("{error:?}").contains("private-malformed-version"));
        }
        let mut account = original;
        account.credentials.push(version_credential(b"2.26.43.72"));
        account.credentials.push(version_credential(b"2.26.37.70"));
        let error = primary_login(&account, &config).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid input for `login`: saved app versions conflict"
        );
    }

    #[test]
    fn duplicate_identical_saved_versions_and_genuine_absence_preserve_selection() {
        let (mut config, mut account) = version_fixture();
        account.credentials.push(version_credential(b"2.26.43.72"));
        account.credentials.push(version_credential(b"2.26.43.72"));
        config.client.user_agent = Some(Zeroizing::new("private-agent-without-version".into()));
        // A valid saved fact still wins over unrelated mutable config.
        let payload =
            ClientPayload::decode(primary_login(&account, &config).unwrap().as_slice()).unwrap();
        let version = payload.user_agent.unwrap().app_version.unwrap();
        assert_eq!((version.tertiary, version.quaternary), (Some(43), Some(72)));
        account
            .credentials
            .retain(|c| c.name != REGISTERED_APP_VERSION);
        let error = primary_login(&account, &config).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid input for `login`: configured User-Agent has no valid app version"
        );
        config.client.user_agent = None;
        assert_eq!(
            login_app_version(&account, &config).unwrap(),
            [2, 26, 37, 70]
        );
        config.client.user_agent =
            Some(Zeroizing::new("WhatsApp/2.26.43.72 Android/unknown".into()));
        assert_eq!(
            login_app_version(&account, &config).unwrap(),
            [2, 26, 43, 72]
        );
    }
}
