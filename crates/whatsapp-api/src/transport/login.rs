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
    let payload = ClientPayload {
        username: Some(username),
        passive: Some(false),
        short_connect: Some(false),
        connect_reason: Some(ConnectReason::UserActivated as i32),
        user_agent: Some(UserAgent {
            platform: Some(Platform::Android as i32),
            app_version: Some(AppVersion {
                primary: Some(2),
                secondary: Some(26),
                tertiary: Some(37),
                quaternary: Some(70),
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
        account.credentials[0].value = Zeroizing::new(b"not-numeric".to_vec());
        assert!(primary_login(&account, &cfg).is_err());
        account.credentials.clear();
        assert!(primary_login(&account, &cfg).is_err());
    }
}
