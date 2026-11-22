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
    registration::request::TrafficAnonymization, session::AccountRecord, store::ProtocolStore,
};
use prost::Message;
use zeroize::Zeroizing;

/// Checkpoint holding the persisted connection counter.
///
/// APK 2.26.43.72 `X.1jy.A05` increments `connection_lc` in `startup_prefs`
/// after a connection attempt finishes and the filler reads it back
/// (`AnonymousClass270` case 459). This key is the same counter in this client.
const CONNECTION_COUNTER: &[u8] = b"comparison-connection-counter-v1";

/// Connection facts the APK's identity filler writes into every login payload
/// (`AnonymousClass270` case 459, `X.1n9`).
///
/// * `reason`: the filler writes `UNKNOWN` first and upgrades to
///   `USER_ACTIVATED`/`PUSH` only within ten seconds of that trigger, so an
///   automatic reconnect carries `UNKNOWN`.
/// * `attempts`/`lifetime_connections`: both count finished connection attempts.
///   The APK keeps `lc` in preferences and the thread's attempt counter in
///   memory; this client models both with the persisted counter, so after a
///   restart the second can be larger than the APK's in-memory value.
/// * `session_id`: the APK generates a random `int` when a socket connects
///   (`ConnectionThread/tryOpenSocket: socket connected; sessionId=…`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoginConnection {
    /// `connectReason` (13).
    pub reason: ConnectReason,
    /// `connectAttemptCount` (16).
    pub attempts: u32,
    /// `sessionId` (9).
    pub session_id: i32,
    /// `lc` (24).
    pub lifetime_connections: u32,
}

impl LoginConnection {
    /// Facts for one attempt. `finished` is the persisted number of attempts
    /// that already completed before this one.
    pub fn for_attempt(reason: ConnectReason, finished: u32, session_id: i32) -> Self {
        Self {
            reason,
            attempts: finished,
            session_id,
            lifetime_connections: finished,
        }
    }
}

/// Persisted number of connection attempts that have finished.
pub fn connection_attempts(store: &ProtocolStore) -> Result<u32> {
    let value: Option<serde_json::Value> =
        store.read(|tx| tx.get_json(crate::store::Table::Checkpoints, CONNECTION_COUNTER, 1))?;
    match value.and_then(|value| value["finished"].as_u64()) {
        Some(finished) => u32::try_from(finished)
            .map_err(|_| Error::invalid("login", "saved connection counter is out of range")),
        None => Ok(0),
    }
}

/// Count one finished connection attempt, wrapping like `X.1jy.A05`
/// (`Integer.MAX_VALUE` becomes 0).
pub fn record_finished_connection(store: &ProtocolStore) -> Result<()> {
    store.write(|tx| {
        let previous: Option<serde_json::Value> =
            tx.get_json(crate::store::Table::Checkpoints, CONNECTION_COUNTER, 1)?;
        let finished = previous
            .as_ref()
            .and_then(|value| value["finished"].as_u64())
            .unwrap_or(0);
        let finished = if finished >= i32::MAX as u64 {
            0
        } else {
            finished + 1
        };
        tx.put_json(
            crate::store::Table::Checkpoints,
            CONNECTION_COUNTER,
            1,
            &serde_json::json!({"finished": finished}),
        )
    })
}

/// One random `int` per socket connection, as the APK generates.
pub fn new_session_id() -> Result<i32> {
    let mut bytes = [0u8; 4];
    getrandom::fill(&mut bytes)
        .map_err(|_| Error::invalid("login", "entropy source failed for a session id"))?;
    Ok(i32::from_le_bytes(bytes))
}

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
/// Device facts come only from configuration; the display name comes only from
/// what this account already sent. This is an explicitly minimal experiment,
/// not a complete APK login payload.
pub fn primary_login(
    account: &AccountRecord,
    config: &LabRegistrationConfig,
    push_name: Option<&str>,
    connection: &LoginConnection,
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
        // 43.72 `X.1n9`: UNKNOWN unless a user trigger is under ten seconds old.
        connect_reason: Some(connection.reason as i32),
        connect_attempt_count: Some(connection.attempts),
        session_id: Some(connection.session_id),
        lc: Some(connection.lifetime_connections as i32),
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
    // 43.72 `X.1nX` (AnonymousClass270 case 461) writes the locally saved
    // display name when non-empty. It is the name this account already sent,
    // never a new or guessed one.
    if let Some(name) = push_name
        && let Ok(name) = super::presence::normalize_name(name)
    {
        payload.push_name = Some(name.to_owned());
    }
    Ok(Zeroizing::new(payload.encode_to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::time::Timestamp,
        session::{ObservedCredential, RegistrationStatus},
    };

    fn connection() -> LoginConnection {
        LoginConnection::for_attempt(ConnectReason::UserActivated, 3, 123456)
    }
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
        let bytes = primary_login(&account, &cfg, None, &connection()).unwrap();
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
        let payload = ClientPayload::decode(
            primary_login(&account, &cfg, None, &connection())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let version = payload.user_agent.unwrap().app_version.unwrap();
        assert_eq!((version.tertiary, version.quaternary), (Some(43), Some(72)));
        account.credentials.pop();
        assert_eq!(
            parse_app_version("WhatsApp/2.26.43.72 Android/unknown"),
            Some([2, 26, 43, 72])
        );
        assert_eq!(parse_app_version("WhatsApp/2.26"), None);
        account.credentials[0].value = Zeroizing::new(b"not-numeric".to_vec());
        assert!(primary_login(&account, &cfg, None, &connection()).is_err());
        account.credentials.clear();
        assert!(primary_login(&account, &cfg, None, &connection()).is_err());
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
            let error = primary_login(&account, &config, None, &connection()).unwrap_err();
            assert_eq!(
                error.to_string(),
                "invalid input for `login`: saved app version is malformed"
            );
            assert!(!format!("{error:?}").contains("private-malformed-version"));
        }
        let mut account = original;
        account.credentials.push(version_credential(b"2.26.43.72"));
        account.credentials.push(version_credential(b"2.26.37.70"));
        let error = primary_login(&account, &config, None, &connection()).unwrap_err();
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
        let payload = ClientPayload::decode(
            primary_login(&account, &config, None, &connection())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let version = payload.user_agent.unwrap().app_version.unwrap();
        assert_eq!((version.tertiary, version.quaternary), (Some(43), Some(72)));
        account
            .credentials
            .retain(|c| c.name != REGISTERED_APP_VERSION);
        let error = primary_login(&account, &config, None, &connection()).unwrap_err();
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
        let payload = ClientPayload::decode(
            primary_login(&account, &config, None, &connection())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
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
        let payload = ClientPayload::decode(
            primary_login(&account, &config, None, &connection())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
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
        let payload = ClientPayload::decode(
            primary_login(&account, &config, None, &connection())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let ua = payload.user_agent.unwrap();
        assert_eq!(ua.os_version.as_deref(), Some("13"));
        assert_eq!(ua.device, None);
        assert_eq!(ua.manufacturer, None);
        assert_eq!(ua.mcc, None);
        assert_eq!(ua.phone_id, None);
        assert_eq!(payload.year_class, None);
    }

    #[test]
    fn connection_facts_match_the_traced_filler() {
        let (config, account) = version_fixture();
        let explicit = LoginConnection::for_attempt(ConnectReason::UserActivated, 2, 0x0102_0304);
        let payload = ClientPayload::decode(
            primary_login(&account, &config, None, &explicit)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert_eq!(payload.connect_reason, Some(1), "USER_ACTIVATED");
        assert_eq!(payload.connect_attempt_count, Some(2));
        assert_eq!(payload.session_id, Some(0x0102_0304));
        assert_eq!(payload.lc, Some(2));
        assert_eq!(payload.passive, Some(false));
        assert_eq!(payload.short_connect, Some(false));
        // An automatic reconnect is the APK's UNKNOWN default, not a claim that
        // a user just opened the app.
        let retry = LoginConnection::for_attempt(ConnectReason::Unknown, 7, 5);
        let payload = ClientPayload::decode(
            primary_login(&account, &config, None, &retry)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert_eq!(payload.connect_reason, Some(6), "UNKNOWN");
        assert_eq!(payload.connect_attempt_count, Some(7));
        assert_eq!(payload.lc, Some(7));
        assert_eq!(payload.session_id, Some(5));
        // Fresh random session ids are not all equal.
        let a = new_session_id().unwrap();
        let b = new_session_id().unwrap();
        assert_ne!(a, b);
    }

    fn counter_store() -> (tempfile::TempDir, ProtocolStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(crate::store::OpenOptions {
            path: dir.path().join("counter.redb"),
            account_id: crate::model::ids::AccountId::new("counter").unwrap(),
            key_source: std::sync::Arc::new(crate::store::StaticKeySource::new(
                crate::store::MasterKey::from_bytes([4; 32]),
            )),
            create: true,
        })
        .unwrap();
        (dir, store)
    }

    #[test]
    fn finished_connection_counter_starts_at_zero_and_wraps() {
        let (_dir, store) = counter_store();
        assert_eq!(connection_attempts(&store).unwrap(), 0);
        record_finished_connection(&store).unwrap();
        assert_eq!(connection_attempts(&store).unwrap(), 1);
        // The APK's `X.1jy.A05` wraps MAX_VALUE to 0 instead of overflowing.
        store
            .write(|tx| {
                tx.put_json(
                    crate::store::Table::Checkpoints,
                    CONNECTION_COUNTER,
                    1,
                    &serde_json::json!({"finished": i32::MAX}),
                )
            })
            .unwrap();
        record_finished_connection(&store).unwrap();
        assert_eq!(connection_attempts(&store).unwrap(), 0);
        // A damaged record is refused instead of silently restarting the count.
        store
            .write(|tx| {
                tx.put_json(
                    crate::store::Table::Checkpoints,
                    CONNECTION_COUNTER,
                    1,
                    &serde_json::json!({"finished": 4_294_967_296u64}),
                )
            })
            .unwrap();
        assert!(connection_attempts(&store).is_err());
    }

    #[test]
    fn saved_display_name_is_carried_and_never_invented() {
        let (config, account) = version_fixture();
        // A name this account already sent is carried at login.
        let payload = ClientPayload::decode(
            primary_login(&account, &config, Some("  Lab Phone  "), &connection())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert_eq!(payload.push_name.as_deref(), Some("Lab Phone"));
        // No saved name means no pushName field, not an empty claim.
        let payload = ClientPayload::decode(
            primary_login(&account, &config, None, &connection())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert_eq!(payload.push_name, None);
        // An unusable stored value is dropped instead of being sent.
        for name in ["", "   ", "two\nlines"] {
            let payload = ClientPayload::decode(
                primary_login(&account, &config, Some(name), &connection())
                    .unwrap()
                    .as_slice(),
            )
            .unwrap();
            assert_eq!(payload.push_name, None, "{name:?} was sent");
        }
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
