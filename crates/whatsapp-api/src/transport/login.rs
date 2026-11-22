//! Minimal primary login assembled from known account/configuration facts.
//! Comparison LX/1LC.AAU (identity), LX/1Ed.AAU (version/locale), LX/1IC.AAU
//! (active/short connection and reason). Unknown Android hardware is omitted.

use crate::proto::wa_web_protobufs_wa6::{
    ClientPayload,
    client_payload::{
        self, ConnectReason, UserAgent,
        user_agent::{AppVersion, Platform},
    },
};
use crate::{
    Error, Result, registration::request::LabRegistrationConfig,
    registration::request::TrafficAnonymization, session::AccountRecord,
};
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
    // 43.72 `X.1m5` (AnonymousClass270 case 456) always fills the Android
    // user agent from `android.os.Build` and related services. Each field is
    // filled only from configured facts: an absent fact stays absent instead of
    // becoming an invented device claim.
    let mut user_agent = UserAgent {
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
    };
    let mut payload = ClientPayload {
        username: Some(username),
        passive: Some(false),
        short_connect: Some(false),
        connect_reason: Some(ConnectReason::UserActivated as i32),
        ..Default::default()
    };
    if let Some(device) = &config.client.device {
        let text = |value: &Option<Zeroizing<String>>| value.as_ref().map(|v| v.to_string());
        user_agent.mcc = text(&device.mcc);
        user_agent.mnc = text(&device.mnc);
        user_agent.os_version = text(&device.os_version);
        user_agent.manufacturer = text(&device.manufacturer);
        user_agent.device = text(&device.device);
        user_agent.os_build_number = text(&device.os_build_number);
        user_agent.phone_id = text(&device.phone_id);
        user_agent.device_board = text(&device.board);
        user_agent.device_model_type = text(&device.model);
        // The APK marks the ordinary handset explicitly; this client never
        // claims a tablet/desktop/VR class.
        user_agent.device_type = Some(client_payload::user_agent::DeviceType::Phone as i32);
        payload.year_class = device.year_class.map(|year| year as i32);
        payload.mem_class = device.mem_class.map(|class| class as i32);
        payload.traffic_anonymization = device.traffic_anonymization.map(|mode| match mode {
            TrafficAnonymization::Off => client_payload::TrafficAnonymization::Off as i32,
            TrafficAnonymization::Standard => client_payload::TrafficAnonymization::Standard as i32,
        });
    }
    payload.user_agent = Some(user_agent);
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

    fn device_config() -> (LabRegistrationConfig, AccountRecord) {
        let (mut config, account) = version_fixture();
        config.client.user_agent = Some(Zeroizing::new(
            "WhatsApp/2.26.43.72 Android/13 Device/Google-Pixel 4a".into(),
        ));
        config.client.device = Some(crate::registration::request::LoginDevice {
            os_version: Some(Zeroizing::new("13".into())),
            manufacturer: Some(Zeroizing::new("Google".into())),
            device: Some(Zeroizing::new("sunfish".into())),
            model: Some(Zeroizing::new("Pixel 4a".into())),
            os_build_number: Some(Zeroizing::new("TQ3A.230805.001.S1".into())),
            board: Some(Zeroizing::new("sunfish".into())),
            mcc: Some(Zeroizing::new("000".into())),
            mnc: Some(Zeroizing::new("000".into())),
            phone_id: Some(Zeroizing::new(
                "0d6f0f9a-1c2b-4c3d-8e4f-5a6b7c8d9e0f".into(),
            )),
            year_class: Some(2016),
            mem_class: Some(512),
            traffic_anonymization: Some(TrafficAnonymization::Standard),
        });
        (config, account)
    }

    #[test]
    fn configured_device_facts_fill_the_android_user_agent() {
        let (config, account) = device_config();
        config.validate().unwrap();
        let payload =
            ClientPayload::decode(primary_login(&account, &config).unwrap().as_slice()).unwrap();
        let ua = payload.user_agent.unwrap();
        assert_eq!(ua.platform, Some(Platform::Android as i32));
        assert_eq!(ua.mcc.as_deref(), Some("000"));
        assert_eq!(ua.mnc.as_deref(), Some("000"));
        assert_eq!(ua.os_version.as_deref(), Some("13"));
        assert_eq!(ua.manufacturer.as_deref(), Some("Google"));
        // Build.DEVICE, not the marketing model name.
        assert_eq!(ua.device.as_deref(), Some("sunfish"));
        assert_eq!(ua.device_model_type.as_deref(), Some("Pixel 4a"));
        assert_eq!(ua.os_build_number.as_deref(), Some("TQ3A.230805.001.S1"));
        assert_eq!(ua.device_board.as_deref(), Some("sunfish"));
        assert_eq!(
            ua.phone_id.as_deref(),
            Some("0d6f0f9a-1c2b-4c3d-8e4f-5a6b7c8d9e0f")
        );
        assert_eq!(ua.device_type, Some(0), "PHONE");
        assert_eq!(payload.year_class, Some(2016));
        assert_eq!(payload.mem_class, Some(512));
        assert_eq!(payload.traffic_anonymization, Some(1), "STANDARD");
        // The official-client marker stays absent; nothing is attested here.
        assert_eq!(payload.oc, None);
        assert!(payload.device_pairing_data.is_none());
    }

    #[test]
    fn absent_or_partial_device_facts_are_not_invented() {
        let (mut config, account) = version_fixture();
        config.client.device = None;
        let payload =
            ClientPayload::decode(primary_login(&account, &config).unwrap().as_slice()).unwrap();
        assert_eq!(payload.year_class, None);
        assert_eq!(payload.mem_class, None);
        assert_eq!(payload.traffic_anonymization, None);
        // A configured-but-partial block keeps the unset fields absent too.
        config.client.user_agent = Some(Zeroizing::new(
            "WhatsApp/2.26.43.72 Android/13 Device/Google-Pixel 4a".into(),
        ));
        config.client.device = Some(crate::registration::request::LoginDevice {
            os_version: Some(Zeroizing::new("13".into())),
            ..Default::default()
        });
        let payload =
            ClientPayload::decode(primary_login(&account, &config).unwrap().as_slice()).unwrap();
        let ua = payload.user_agent.unwrap();
        assert_eq!(ua.os_version.as_deref(), Some("13"));
        assert_eq!(ua.device, None);
        assert_eq!(ua.manufacturer, None);
        assert_eq!(ua.mcc, None);
        assert_eq!(ua.phone_id, None);
        assert_eq!(payload.year_class, None);
    }

    #[test]
    fn impossible_device_facts_are_refused_by_validation() {
        let (mut config, _) = version_fixture();
        config.client.user_agent = Some(Zeroizing::new(
            "WhatsApp/2.26.43.72 Android/13 Device/Google-Pixel 4a".into(),
        ));
        let base = crate::registration::request::LoginDevice {
            os_version: Some(Zeroizing::new("13".into())),
            mcc: Some(Zeroizing::new("000".into())),
            mnc: Some(Zeroizing::new("000".into())),
            phone_id: Some(Zeroizing::new(
                "0d6f0f9a-1c2b-4c3d-8e4f-5a6b7c8d9e0f".into(),
            )),
            year_class: Some(2016),
            mem_class: Some(512),
            ..Default::default()
        };
        config.client.device = Some(base.clone());
        config.validate().unwrap();
        let cases = [
            crate::registration::request::LoginDevice {
                os_version: Some(Zeroizing::new("".into())),
                ..base.clone()
            },
            crate::registration::request::LoginDevice {
                device: Some(Zeroizing::new("sun\nfish".into())),
                ..base.clone()
            },
            crate::registration::request::LoginDevice {
                mcc: Some(Zeroizing::new("00".into())),
                ..base.clone()
            },
            crate::registration::request::LoginDevice {
                mnc: Some(Zeroizing::new("0000".into())),
                ..base.clone()
            },
            crate::registration::request::LoginDevice {
                phone_id: Some(Zeroizing::new("not-a-uuid".into())),
                ..base.clone()
            },
            crate::registration::request::LoginDevice {
                year_class: Some(2008),
                ..base.clone()
            },
            crate::registration::request::LoginDevice {
                mem_class: Some(4096),
                ..base.clone()
            },
        ];
        for device in cases {
            config.client.device = Some(device);
            assert!(config.validate().is_err());
        }
    }
}
